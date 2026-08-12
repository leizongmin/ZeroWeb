//! 可选的 paint 阶段字符宽度测量回调。
//!
//! 浏览器在渲染帧前通过 thread-local 注入真实字体 metrics；
//! 未注入时回退到 layout 的 `estimate_char_width`。

use std::sync::OnceLock;

use zero_layout_engine::inline::{AdvanceSource, estimate_char_width};
use zero_render_foundation::font::{FontSizeAdjustment, OpenTypeFeature, ShapedGlyph, TextDirection};

static CHAR_MEASURE: OnceLock<fn(char, f32, bool) -> f32> = OnceLock::new();
/// 宿主提供的文本整形回调签名。
pub type TextShapeFn =
    fn(&[u32], &str, f32, TextDirection, &[OpenTypeFeature], FontSizeAdjustment) -> Option<Vec<ShapedGlyph>>;
static TEXT_SHAPE: OnceLock<TextShapeFn> = OnceLock::new();

/// 注册全局字符宽度测量函数（浏览器启动时调用一次）。
pub fn set_char_measure_fn(f: fn(char, f32, bool) -> f32) {
    let _ = CHAR_MEASURE.set(f);
}

/// 注册全局文本整形函数（宿主启动时调用一次）。
pub fn set_text_shape_fn(f: TextShapeFn) {
    let _ = TEXT_SHAPE.set(f);
}

/// Paint 阶段测量单个字符 advance；Ahem 字体固定为 1em 方框宽。
pub fn measure_char_for_paint(ch: char, font_size: f32, is_ahem: bool) -> f32 {
    if is_ahem {
        return font_size;
    }
    CHAR_MEASURE
        .get()
        .copied()
        .map(|measure| measure(ch, font_size, is_ahem))
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
    adjustment: FontSizeAdjustment,
) -> Option<Vec<ShapedGlyph>> {
    TEXT_SHAPE
        .get()
        .and_then(|shape| shape(font_ids, text, font_size, direction, features, adjustment))
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
pub(crate) struct ShapedAdvanceSource;

impl AdvanceSource for ShapedAdvanceSource {
    fn measure(&self, ch: char, _font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
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
        )
    }

    fn measure_text_with_font_context(
        &self,
        text: &str,
        font_ids: &[u32],
        font_size: f32,
        is_ahem: bool,
        size_adjust: &zero_style_system::FontSizeAdjustValue,
    ) -> f32 {
        let estimated: f32 = text.chars().map(|ch| estimate_char_width(ch, font_size, is_ahem)).sum();
        if is_ahem {
            return estimated;
        }
        if font_ids.is_empty() {
            return estimated;
        }
        let Some(shaped) = shape_text_for_paint(
            font_ids,
            text,
            font_size,
            TextDirection::LeftToRight,
            &[],
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
        let paint_base: f32 = text
            .chars()
            .map(|ch| measure_char_for_paint(ch, font_size, false))
            .sum();
        paint_base_with_contextual_delta(paint_base, contextual, unshaped)
    }
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
        assert_eq!(measure_char_for_paint('.', 100.0, true), 100.0);
        assert_eq!(measure_char_for_paint('p', 100.0, true), 100.0);
        assert_eq!(measure_char_for_paint('.', 16.0, true), 16.0);
    }

    /// 非 Ahem：'.' 远窄于 font_size（真实点宽，非 1em 方块）。
    #[test]
    fn test_measure_char_non_ahem_dot_is_narrow() {
        let dot_width = measure_char_for_paint('.', 100.0, false);
        assert!(
            dot_width < 100.0,
            "非 Ahem '.' 须窄于 1em（font_size），实际 {dot_width}"
        );
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
