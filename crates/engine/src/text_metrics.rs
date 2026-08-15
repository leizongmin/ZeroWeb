//! 可选的 paint 阶段字符宽度测量回调。
//!
//! 浏览器在渲染帧前通过 thread-local 注入真实字体 metrics；
//! 未注入时回退到 layout 的 `estimate_char_width`。

use std::sync::OnceLock;

use zero_layout_engine::inline::{AdvanceSource, estimate_char_width};
use zero_render_foundation::font::{
    FontSizeAdjustment, OpenTypeFeature, OpenTypeVariation, ShapedGlyph, TextDirection,
};

static CHAR_MEASURE: OnceLock<fn(u32, char, f32, bool) -> f32> = OnceLock::new();
/// 宿主提供的 hmtx 批量测量回调（布局 estimate 替换，ZRG-2026-08-15 修复 A）。
pub type HmtxMeasureFn = fn(&[u32], &str, f32) -> Option<f32>;
static HMTX_MEASURE: OnceLock<HmtxMeasureFn> = OnceLock::new();
/// 宿主提供的文本整形回调签名。
pub type TextShapeFn = fn(
    &[u32],
    &str,
    f32,
    TextDirection,
    &[OpenTypeFeature],
    &[OpenTypeVariation],
    FontSizeAdjustment,
) -> Option<Vec<ShapedGlyph>>;
static TEXT_SHAPE: OnceLock<TextShapeFn> = OnceLock::new();

/// 注册全局字符宽度测量函数（浏览器启动时调用一次）。
///
/// 回调签名 `fn(font_id, ch, font_size, is_ahem)`：`font_id` 为字形实际解析的
/// 字体（ZRG-2026-08-15 起显式传入——此前用 thread-local primary 字体测量，
/// 与 @font-face webfont 字形脱节致字距错乱）。
pub fn set_char_measure_fn(f: fn(u32, char, f32, bool) -> f32) {
    let _ = CHAR_MEASURE.set(f);
}

/// 注册全局文本整形函数（宿主启动时调用一次）。
pub fn set_text_shape_fn(f: TextShapeFn) {
    let _ = TEXT_SHAPE.set(f);
}

/// 注册全局 hmtx 批量测量函数（宿主启动时调用一次）。
///
/// 签名 `fn(font_ids, text, font_size) -> Option<f32>`：按字体链读 hmtx 求和，
/// 与 rustybuzz 的 `unshaped_advance_x` 同源。布局侧 estimate 启发式的替换
/// （ZRG-2026-08-15 修复 A）；宿主未注册或无字体上下文时回退 estimate。
pub fn set_hmtx_measure_fn(f: HmtxMeasureFn) {
    let _ = HMTX_MEASURE.set(f);
}

/// 布局侧 hmtx 测量：宿主注册且字体链有效时返回真实宽度，否则 None。
pub fn measure_text_hmtx_for_layout(font_ids: &[u32], text: &str, font_size: f32) -> Option<f32> {
    if font_ids.is_empty() {
        return None;
    }
    HMTX_MEASURE
        .get()
        .copied()
        .and_then(|measure| measure(font_ids, text, font_size))
}

/// 是否启用 production variable-font axis 消费。
///
/// Shaping、字形 raster 与 IPC 已携带同一坐标；默认仍关闭，等待 Chromium Oracle 裁决。
pub fn font_variations_enabled() -> bool {
    std::env::var("ZW_FONT_VARIATIONS").as_deref() == Ok("1")
}

/// Paint 阶段按字形实际字体测量单个字符 advance；Ahem 字体固定为 1em 方框宽。
///
/// `font_id` 为字形实际解析的字体（webfont/fallback），保证测量与 shaping 同源
/// （ZRG-2026-08-15：此前用 thread-local primary 字体测所有字形，多字体页面
/// 字距与 Chrome 差 ~1px/词）。
pub fn measure_char_for_font(font_id: u32, ch: char, font_size: f32, is_ahem: bool) -> f32 {
    if is_ahem {
        return font_size;
    }
    CHAR_MEASURE
        .get()
        .copied()
        .map(|measure| measure(font_id, ch, font_size, is_ahem))
        .unwrap_or_else(|| estimate_char_width(ch, font_size, is_ahem))
}

/// 与 layout IFC 一致的字符宽度估计（无真实字体回调时使用）。
pub fn layout_estimate_char_width(ch: char, font_size: f32, is_ahem: bool) -> f32 {
    estimate_char_width(ch, font_size, is_ahem)
}

/// Paint 阶段按实际字体 ID 整形文本；宿主未注册或当前无字体上下文时返回 `None`。
pub fn shape_text_for_paint(
    font_ids: &[u32],
    text: &str,
    font_size: f32,
    direction: TextDirection,
    features: &[OpenTypeFeature],
    variations: &[OpenTypeVariation],
    adjustment: FontSizeAdjustment,
) -> Option<Vec<ShapedGlyph>> {
    let variations = if font_variations_enabled() { variations } else { &[] };
    TEXT_SHAPE
        .get()
        .and_then(|shape| shape(font_ids, text, font_size, direction, features, variations, adjustment))
}

pub(crate) fn font_size_adjustment(value: &zero_style_system::FontSizeAdjustValue) -> FontSizeAdjustment {
    match value {
        zero_style_system::FontSizeAdjustValue::None => FontSizeAdjustment::None,
        zero_style_system::FontSizeAdjustValue::Adjust { metric, basis } => {
            let metric = match metric.unwrap_or(zero_style_system::FontSizeAdjustMetric::ExHeight) {
                zero_style_system::FontSizeAdjustMetric::ExHeight => {
                    zero_render_foundation::font::FontSizeAdjustMetric::ExHeight
                }
                zero_style_system::FontSizeAdjustMetric::CapHeight => {
                    zero_render_foundation::font::FontSizeAdjustMetric::CapHeight
                }
                zero_style_system::FontSizeAdjustMetric::ChWidth => {
                    zero_render_foundation::font::FontSizeAdjustMetric::ChWidth
                }
                zero_style_system::FontSizeAdjustMetric::IcWidth => {
                    zero_render_foundation::font::FontSizeAdjustMetric::IcWidth
                }
                zero_style_system::FontSizeAdjustMetric::IcHeight => {
                    zero_render_foundation::font::FontSizeAdjustMetric::IcHeight
                }
            };
            let target = match basis {
                zero_style_system::FontSizeAdjustBasis::Number(value) => Some(*value as f32),
                zero_style_system::FontSizeAdjustBasis::FromFont => None,
            };
            FontSizeAdjustment::Adjust { metric, target }
        }
    }
}

pub(crate) fn font_variations(value: &zero_style_system::FontVariationSettingsValue) -> Vec<OpenTypeVariation> {
    match value {
        zero_style_system::FontVariationSettingsValue::Normal => Vec::new(),
        zero_style_system::FontVariationSettingsValue::Settings(settings) => settings
            .iter()
            .map(|setting| OpenTypeVariation::new(setting.tag, setting.value))
            .collect(),
    }
}

pub(crate) fn paint_font_variations(value: &zero_style_system::FontVariationSettingsValue) -> Vec<OpenTypeVariation> {
    if font_variations_enabled() {
        font_variations(value)
    } else {
        Vec::new()
    }
}

/// 判断 shaping 输出是否与源 Unicode 标量一一对应。
pub(crate) fn one_to_one_source_mapping(text: &str, glyphs: &[ShapedGlyph]) -> bool {
    glyphs.len() == text.chars().count()
        && text.chars().zip(glyphs).all(|(source, glyph)| {
            source == glyph.code_point && glyph.glyph_id > 0 && u16::try_from(glyph.glyph_id).is_ok()
        })
}

/// 一一映射中是否存在源码标量共享 shaping cluster。
pub(crate) fn source_mapping_requires_offsets(text: &str, glyphs: &[ShapedGlyph]) -> bool {
    text.char_indices()
        .zip(glyphs)
        .any(|((byte_offset, _), glyph)| glyph.cluster != byte_offset as u32)
}

/// 判断 shaping 是否将多个源码标量合并为较少的有效 glyph。
pub(crate) fn many_to_one_source_mapping(text: &str, glyphs: &[ShapedGlyph]) -> bool {
    let text_len = text.len();
    !glyphs.is_empty()
        && glyphs.len() < text.chars().count()
        && glyphs.iter().any(|glyph| glyph.cluster == 0)
        && glyphs.iter().all(|glyph| {
            glyph.glyph_id > 0
                && u16::try_from(glyph.glyph_id).is_ok()
                && usize::try_from(glyph.cluster)
                    .ok()
                    .is_some_and(|cluster| cluster < text_len && text.is_char_boundary(cluster))
        })
}

/// `ZW_SHAPED_TEXT` 使用的 layout advance source。
///
/// 策略（ZRG-2026-08-15 修复 A）：显式 author face run 走 rustybuzz shaping
/// （R3424-F 现状）；generic/系统字体 run 走 hmtx 批量测量（替换 estimate
/// 启发式——布局宽度与绘制宽度错位 15-20% 的根源）。`generic_font_ids` 由
/// pipeline 从 resolver 构建传入。
pub(crate) struct ShapedAdvanceSource {
    generic_font_ids: std::collections::HashSet<u32>,
}

impl ShapedAdvanceSource {
    pub(crate) fn new(generic_font_ids: std::collections::HashSet<u32>) -> Self {
        Self { generic_font_ids }
    }

    /// generic/系统字体 run 的 hmtx 测量（`ZW_HMTX_LAYOUT` 默认开；`"0"` 回退
    /// estimate——与 R3424-F 之前语义一致）。
    fn measure_generic_hmtx(&self, font_ids: &[u32], text: &str, font_size: f32, is_ahem: bool) -> Option<f32> {
        if is_ahem {
            return None;
        }
        if std::env::var("ZW_HMTX_LAYOUT").as_deref() == Ok("0") {
            return None;
        }
        crate::text_metrics::measure_text_hmtx_for_layout(font_ids, text, font_size)
    }
}

impl AdvanceSource for ShapedAdvanceSource {
    fn measure(&self, ch: char, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        // ZRG-2026-08-15 修复 A：单字符测量（悬挂标点/空格/tab 等换行与对齐宽度）
        // 与 measure_text 同源（generic 字体走 hmtx）——否则行断/对齐与布局宽度
        // 不一致（hanging-punctuation 等 reftest 回归）。
        if let Some(id) = font_id
            && self.generic_font_ids.contains(&id)
            && let Some(hmtx) =
                self.measure_generic_hmtx(std::slice::from_ref(&id), &ch.to_string(), font_size, is_ahem)
        {
            return hmtx;
        }
        estimate_char_width(ch, font_size, is_ahem)
    }

    fn measure_text(&self, text: &str, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        let Some(font_id) = font_id else {
            return text.chars().map(|ch| estimate_char_width(ch, font_size, is_ahem)).sum();
        };
        self.measure_text_with_fonts(text, &[font_id], font_size, is_ahem)
    }

    fn measure_text_with_fonts(&self, text: &str, font_ids: &[u32], font_size: f32, is_ahem: bool) -> f32 {
        self.measure_text_with_font_context(
            text,
            font_ids,
            font_size,
            is_ahem,
            &zero_style_system::FontSizeAdjustValue::None,
            &zero_style_system::FontVariationSettingsValue::Normal,
        )
    }

    fn measure_text_with_font_context(
        &self,
        text: &str,
        font_ids: &[u32],
        font_size: f32,
        is_ahem: bool,
        size_adjust: &zero_style_system::FontSizeAdjustValue,
        variations: &zero_style_system::FontVariationSettingsValue,
    ) -> f32 {
        let estimated: f32 = text.chars().map(|ch| estimate_char_width(ch, font_size, is_ahem)).sum();
        if is_ahem {
            return estimated;
        }
        if font_ids.is_empty() {
            return estimated;
        }
        // ZRG-2026-08-15 修复 A：generic/系统字体 run 走 hmtx（无 shaping 开销，
        // 与 paint 的 rustybuzz unshaped 同源）——布局宽度从 estimate 启发式
        // （偏差 15-20%）变为真实 hmtx。宿主未注册 hmtx 时回 estimate，绝不
        // 让 generic run 落入 shaping（R3234-F 37x 回归教训）。复杂 shaping 文本
        //（阿拉伯/印度系等）回退 shaping（量少，perf 可接受）——hmtx 无连字/变体语义。
        if font_ids.iter().all(|id| self.generic_font_ids.contains(id)) && !is_complex_shaping_text(text) {
            return self
                .measure_generic_hmtx(font_ids, text, font_size, is_ahem)
                .unwrap_or(estimated);
        }
        let variation_input = if font_variations_enabled() {
            font_variations(variations)
        } else {
            Vec::new()
        };
        let Some(shaped) = shape_text_for_paint(
            font_ids,
            text,
            font_size,
            TextDirection::LeftToRight,
            &[],
            &variation_input,
            font_size_adjustment(size_adjust),
        ) else {
            return estimated;
        };
        let contextual: f32 = shaped.iter().map(|glyph| glyph.advance_x).sum();
        // https://drafts.csswg.org/css-text-3/#line-breaking
        // R3278-F：complex paint 直接消费 ligature/控制字符折叠后的 glyph run，
        // layout intrinsic advance 也必须使用同一 run 的总 advance。
        if many_to_one_source_mapping(text, &shaped) {
            return contextual;
        }
        if !one_to_one_source_mapping(text, &shaped) || source_mapping_requires_offsets(text, &shaped) {
            return estimated;
        }
        if !matches!(size_adjust, zero_style_system::FontSizeAdjustValue::None)
            || glyph_sizes_adjusted(font_size, &shaped)
        {
            return contextual;
        }
        let unshaped: f32 = shaped.iter().map(|glyph| glyph.unshaped_advance_x).sum();
        // ZRG-2026-08-15：paint_base 按 shaping 实际字体（font_ids 首 face）测量，
        // 与 glyph 字形同源——此前用 primary 字体测，webfont run 的布局↔绘制错位。
        let paint_font_id = font_ids.first().copied().unwrap_or(0);
        let paint_base: f32 = text
            .chars()
            .map(|ch| measure_char_for_font(paint_font_id, ch, font_size, false))
            .sum();
        paint_base_with_contextual_delta(paint_base, contextual, unshaped)
    }
}

/// 文本是否包含复杂 shaping 需求（阿拉伯/希伯来/印度系/高棉等）——hmtx 无
/// 连字/变体语义，这类 run 的布局宽度须走 shaping（文本量少，perf 可接受）。
fn is_complex_shaping_text(text: &str) -> bool {
    text.chars().any(|c| {
        matches!(
            c,
            // 希伯来/阿拉伯/叙利亚/他加禄等 RTL 与复杂脚本区块
            '\u{0590}'..='\u{08FF}'
                | '\u{0E00}'..='\u{0E7F}' // 泰文
                | '\u{0900}'..='\u{0DFF}' // 印度系
                | '\u{1780}'..='\u{17FF}' // 高棉
                | '\u{FB1D}'..='\u{FDFF}' // 阿拉伯呈现形式 A
                | '\u{FE70}'..='\u{FEFF}' // 阿拉伯呈现形式 B
        )
    })
}

pub(crate) fn paint_base_with_contextual_delta(paint_base: f32, shaped: f32, unshaped: f32) -> f32 {
    paint_base + shaped - unshaped
}

pub(crate) fn glyph_sizes_adjusted(font_size: f32, glyphs: &[ShapedGlyph]) -> bool {
    glyphs
        .iter()
        .any(|glyph| (glyph.font_size - font_size).abs() > f32::EPSILON)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::primitive::FontId;

    /// R2237：Ahem 字体的字符宽度 = font_size（1em 完美方块），非真实点宽。
    /// ellipsis 测宽须传 is_ahem=true（Ahem 容器），否则 '.' 宽度过小致 ellipsis 定位错。
    /// driving: WPT css-overflow text-overflow-ellipsis-001（font:100px/1 Ahem）。
    #[test]
    fn test_measure_char_ahem_returns_font_size() {
        // Ahem: 任意字符（含 '.'）= font_size（1em 方块）。
        assert_eq!(measure_char_for_font(0, '.', 100.0, true), 100.0);
        assert_eq!(measure_char_for_font(0, 'p', 100.0, true), 100.0);
        assert_eq!(measure_char_for_font(0, '.', 16.0, true), 16.0);
    }

    /// 非 Ahem：'.' 远窄于 font_size（真实点宽，非 1em 方块）。
    #[test]
    fn test_measure_char_non_ahem_dot_is_narrow() {
        let dot_width = measure_char_for_font(0, '.', 100.0, false);
        assert!(
            dot_width < 100.0,
            "非 Ahem '.' 须窄于 1em（font_size），实际 {dot_width}"
        );
    }

    #[test]
    fn computed_variations_convert_to_shaping_axes() {
        let value = zero_style_system::FontVariationSettingsValue::Settings(vec![
            zero_style_system::FontVariationSetting {
                tag: *b"wdth",
                value: 125.0,
            },
            zero_style_system::FontVariationSetting {
                tag: *b"wght",
                value: 600.7,
            },
        ]);
        assert_eq!(
            font_variations(&value),
            vec![
                OpenTypeVariation::new(*b"wdth", 125.0),
                OpenTypeVariation::new(*b"wght", 600.7),
            ]
        );
        assert!(font_variations(&zero_style_system::FontVariationSettingsValue::Normal).is_empty());
    }

    fn glyph(code_point: char, glyph_id: u32) -> ShapedGlyph {
        ShapedGlyph {
            glyph_id,
            font_id: FontId(1),
            font_size: 16.0,
            advance_x: 8.0,
            unshaped_advance_x: 8.0,
            x_offset: 0.0,
            y_offset: 0.0,
            cluster: 0,
            code_point,
        }
    }

    #[test]
    fn one_to_one_mapping_rejects_ligature_or_wrong_cluster() {
        assert!(one_to_one_source_mapping("AV", &[glyph('A', 3), glyph('V', 4)]));
        assert!(!one_to_one_source_mapping("fi", &[glyph('f', 7)]));
        assert!(!one_to_one_source_mapping("éA", &[glyph('é', 8), glyph('é', 9)]));
    }

    #[test]
    fn many_to_one_mapping_accepts_ligature_and_ignorable_control() {
        assert!(many_to_one_source_mapping("fi", &[glyph('f', 7)]));

        let first = glyph('f', 7);
        let mut second = glyph('i', 8);
        second.cluster = 4;
        assert!(many_to_one_source_mapping("f\u{200c}i", &[first, second]));
        assert!(!many_to_one_source_mapping("fi", &[glyph('f', 0)]));
        let mut invalid_cluster = glyph('f', 7);
        invalid_cluster.cluster = 2;
        assert!(!many_to_one_source_mapping("fi", &[invalid_cluster]));
    }

    #[test]
    fn shared_cluster_requires_offsets() {
        let base = glyph('A', 8);
        let mut mark = glyph('\u{301}', 9);
        mark.cluster = 0;
        assert!(one_to_one_source_mapping("A\u{301}", &[base.clone(), mark.clone()]));
        assert!(source_mapping_requires_offsets("A\u{301}", &[base, mark]));

        let base = glyph('A', 8);
        let mut next = glyph('V', 9);
        next.cluster = 1;
        assert!(!source_mapping_requires_offsets("AV", &[base, next]));
    }

    #[test]
    fn contextual_layout_advance_keeps_the_paint_base() {
        assert_eq!(paint_base_with_contextual_delta(20.0, 18.5, 19.0), 19.5);
    }

    #[test]
    fn adjusted_glyph_size_requires_absolute_shaped_advance() {
        let mut glyph = glyph('A', 1);
        assert!(!glyph_sizes_adjusted(16.0, std::slice::from_ref(&glyph)));
        glyph.font_size = 24.0;
        assert!(glyph_sizes_adjusted(16.0, std::slice::from_ref(&glyph)));
    }
}
