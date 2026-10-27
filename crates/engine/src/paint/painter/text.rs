//! 文本绘制主流程；列表、multicol、ruby 与 shaping 辅助位于 `text/` 子模块。

use std::collections::HashMap;

use zero_css_parser::values::types::FontStyleValue;
use zero_css_parser::values::{ColorValue, DisplayValue, FloatValue, LengthValue, VisibilityValue};
use zero_dom::{Document, NodeId, NodeKind};
use zero_layout_engine::inline_finalization::{
    build_text_parent_override_map, resolve_tab_size_length_px, resolve_text_align, resolve_text_align_last,
    resolve_text_group_align, resolve_text_indent, resolve_word_break_mode, subtree_font_differs_from,
    subtree_has_block_elem, subtree_has_text_decoration,
};
use zero_layout_engine::{FloatExclusion, InlineFormattingContext, LayoutBox, NodeIdMap, NodeIdSet};
use zero_render_foundation::color::Color;
use zero_render_foundation::font::TextDirection;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::image_cache::ImageKey;
use zero_render_foundation::primitive::{GlyphPrimitive, ImagePrimitive, TextControlBoundary};
use zero_style_system::{
    ComputedStyle, MixBlendModeComputedValue, TabSizeValue, TextEmphasisPositionValue, TextEmphasisStyleValue,
    TextOverflowValue, TextTransformValue, WhiteSpaceValue,
};
/// R4443：ZW_DEBUG_GLYPHS 探针开关（OnceLock 缓存，热路径零 env::var 开销——
/// block_layout_1000_elements bench +39% 实证 env::var 逐次调用开销）。
fn glyph_probe_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_DEBUG_GLYPHS").as_deref() == Ok("1"))
}

/// R4558：直系文本度量种子（Path B override 缺失兜底）开关。default-on；
/// `ZW_PAINT_DIRECT_TEXT_SEED=0` 关闭（回退无种子行为）。
fn direct_text_seed_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_PAINT_DIRECT_TEXT_SEED").as_deref() != Ok("0"))
}

/// R5289：行内联内容被合成匿名 cell 接管后行级直绘抑制开关。default-on；
/// `ZW_ROW_ANON_CELL_SUPPRESS=0` 关闭（回退行盒 Path B 重跑叠绘行为）。
fn row_anon_cell_suppress_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_ROW_ANON_CELL_SUPPRESS").as_deref() != Ok("0"))
}

/// R5169：stored 片段基线锚定开关——非 Ahem 片段的 v_offset 同 is_ahem_font 臂取
/// `baseline_y_abs - frag.y`（存储 IFC 真实行基线），`frag.font_size` 近似把基线当
/// fs（infer-cells monospace 32px 行基线 37 → 字形整体高 5px 实证）。default-on；
/// `ZW_STORED_BASELINE_ANCHOR=0` 回退 fs 近似。
fn stored_baseline_anchor_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_STORED_BASELINE_ANCHOR").as_deref() != Ok("0"))
}

/// R5120：单行 inline border/padding 整盒几何开关。**默认关（dormant）**——
/// 几何已实现且自洽（`ZW_INLINE_SINGLE_FRAG_BOX_BORDER=1` 时
/// border-width-applies-to-008 15.53%→100%），但 bidi 控制页（css/CSS2/bidi-text/
/// bidi-001 族）在自源 reftest 口径下被正确几何暴露 css-writing-modes-3 bidi box
/// model 缺口（inline 不在 bidi level 边界裂解、RLO 反转不跨 run——test 页单盒
/// vs ref 页双盒 1.03~1.24% > 阈值 1.00%；HEAD 双页同错细条 0.64~0.97% 反而
/// 「绿」）。翻默认前置 = bidi 裂解 slice 落地；届时须缓存 per-(font,size) 度量
/// （fallback_line_metrics_for_paint 现每次 Face::parse，见 r5120_content_box）。
fn single_frag_box_border_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_INLINE_SINGLE_FRAG_BOX_BORDER").as_deref() == Ok("1"))
}

use super::super::color::{color_value_to_render, resolve_color_current};

/// R4525/R4549/R4553（css-backgrounds-4 §background-clip:text）：clip:text 的「恒色背景」提取。
///
/// 背景层（color + image 自下而上）合成结果为恒色 C 时，mask 管线的输出与「字形以
/// `text OVER C` 预混合色染色」逐像素等价（OVER 结合律 + coverage 可分配，R4549 证据
/// 文档推导）——无需离屏合成。任一层非恒色（多色渐变 / url 图片非纯色或非默认平铺）
/// → None，维持 bg 正常绘制，归真 mask 管线域。
///
/// url 层（R4553）：`solid_colors`（宿主从 ImageCache 预填的纯色缓存，image_sizes 同款
/// 模式）命中且该层 repeat/size/attachment 均默认时为恒色（纯色图平铺满 positioning
/// area ≡ 恒色）；未命中（未解码/生产链未注入）→ 恒色性不可判 → 禁用，与未解码时
/// chromium 不渲染该 bg 层的现状行为一致。
///
/// C 的 alpha 为 0（bg 全透明）→ None：clip:text 无视觉贡献，不激活（激活反而会以
/// 透明色覆盖字形原色）。
pub(super) fn bg_clip_text_solid_color(
    st: &ComputedStyle,
    solid_colors: &HashMap<u64, [u8; 4]>,
    document_url: Option<&str>,
) -> Option<Color> {
    use zero_style_system::property::types::{
        BackgroundAttachmentComputedValue, BackgroundClipComputedValue, BackgroundImageComputedValue,
        BackgroundRepeatComputedValue, BackgroundSizeComputedValue,
    };
    if !st
        .background_clip
        .iter()
        .any(|c| matches!(c, BackgroundClipComputedValue::Text))
    {
        return None;
    }
    // R5268：per-layer longhand 按 css-backgrounds-3 §3.1 cyclic 取值（`i % len`，
    // 与 paint 侧 R2311 同规）——旧 `.get(i)?` 在多层页（image 层数 > 声明值数）时
    // 次层取 None 直接禁用整页提取（background-clip-text-and-blend-mode box2/3）。
    let layer_ctx = |i: usize| -> Option<(
        &BackgroundRepeatComputedValue,
        &BackgroundSizeComputedValue,
        &BackgroundAttachmentComputedValue,
    )> {
        let (n_r, n_s, n_a) = (
            st.background_repeat.len(),
            st.background_size.len(),
            st.background_attachment.len(),
        );
        if n_r == 0 || n_s == 0 || n_a == 0 {
            return None;
        }
        Some((
            st.background_repeat.get(i % n_r)?,
            st.background_size.get(i % n_s)?,
            st.background_attachment.get(i % n_a)?,
        ))
    };
    // 各图层恒色提取（list 位置随行保留；None 层无贡献跳过，位置仍归 mode/repeat 配对）。
    let mut layers = Vec::new();
    for (i, layer) in st.background_image.iter().enumerate() {
        // R5021：filter() 图像函数层——clip:text 纯色提取按内层图像取色
        //（滤镜作用于最终绘制面，纯色近似不消费）。
        let layer = layer.inner();
        let c = match layer {
            BackgroundImageComputedValue::None => continue,
            BackgroundImageComputedValue::Gradient(g) => gradient_solid_color(g, &st.color)?,
            BackgroundImageComputedValue::Url(u) => {
                // R4553 门：纯色缓存命中 + 该层平铺参数全默认（默认 repeat 平铺纯色图 ≡
                // 恒色满铺；no-repeat/contain 等留空区露底色，非恒色）。
                let (r, s, a) = layer_ctx(i)?;
                let tiled = matches!(r, BackgroundRepeatComputedValue::Repeat)
                    && matches!(s, BackgroundSizeComputedValue::Auto)
                    && matches!(a, BackgroundAttachmentComputedValue::Scroll);
                let key = super::super::helpers::image_resource_key(u, document_url);
                let rgba = solid_colors.get(&key).filter(|_| tiled)?;
                Color::rgba(rgba[0], rgba[1], rgba[2], rgba[3])
            }
            // loop 顶 inner() 已剥 filter() 壳——静态穷尽防御臂（运行时不可达）。
            BackgroundImageComputedValue::Filtered { .. } => continue,
        };
        layers.push((i, c));
    }
    let mut acc = resolve_color_current(&st.background_color, &st.color);
    // R5267（css-compositing-1 §background-blend-mode）：存在非 normal 层模式时自底
    // 向上合成——图层 i 与其下方合成结果按 mode `i % m` 混合，background-color 为
    // 最底层 backdrop。全 normal（含未声明）保持既有 plain OVER 叠加（字节不变）；
    // kill-switch `ZW_BG_CLIP_TEXT_LAYER_BLEND=0` 回退旧行为。
    let layer_blend = std::env::var("ZW_BG_CLIP_TEXT_LAYER_BLEND").as_deref() != Ok("0")
        && st
            .background_blend_mode
            .iter()
            .any(|m| !matches!(m, MixBlendModeComputedValue::Normal));
    if layer_blend {
        for &(i, c) in layers.iter().rev() {
            let mode = &st.background_blend_mode[i % st.background_blend_mode.len()];
            acc = blend_solid_layer(mode, c, acc);
        }
    } else {
        // R5268：plain OVER 同样自底向上合成（css-backgrounds-3 §3.1 list 首项在最
        // 上；与下方混合路径及 paint_background 实际铺层序一致）。旧 list 序多层页
        // 提取取底层色（顶层反被盖），单层页不受影响。
        for &(_, c) in layers.iter().rev() {
            acc = color_over(c, acc);
        }
    }
    (acc.a > 0).then_some(acc)
}

/// 渐变层恒色提取：全部 stop 解析为同一 RGBA → 该色；否则 None。
/// repeating 不改变恒色性（周期常数为同一色）；radial/conic 同判（全 stop 同色即恒色）。
fn gradient_solid_color(g: &zero_css_parser::values::GradientValue, element_color: &ColorValue) -> Option<Color> {
    use zero_css_parser::values::GradientValue;
    let stops = match g {
        GradientValue::Linear(l) => &l.stops,
        GradientValue::Radial(r) => &r.stops,
        GradientValue::Conic(c) => &c.stops,
    };
    let first = stops.first().map(|s| resolve_color_current(&s.color, element_color))?;
    stops
        .iter()
        .all(|s| resolve_color_current(&s.color, element_color) == first)
        .then_some(first)
}

/// R5267：恒色层间混合单层步（css-compositing-1 §5.1 B(Cb,Cs) + §5.2 alpha 合成）。
/// normal 走既有 color_over（与 R4549 预混合等价链同路径）；其余模式复用
/// render-foundation 逐像素公式（与 mix-blend-mode 同源，`cpu::blend_rgba`）。
fn blend_solid_layer(mode: &MixBlendModeComputedValue, src: Color, dst: Color) -> Color {
    let Some(rf_mode) = blend_mode_rf(mode) else {
        return color_over(src, dst);
    };
    let out =
        zero_render_foundation::cpu::blend_rgba([src.r, src.g, src.b, src.a], [dst.r, dst.g, dst.b, dst.a], rf_mode);
    Color::rgba(out[0], out[1], out[2], out[3])
}

/// MixBlendModeComputedValue → render-foundation BlendMode（normal → None = 纯 OVER）。
fn blend_mode_rf(m: &MixBlendModeComputedValue) -> Option<zero_render_foundation::primitive::BlendMode> {
    use zero_render_foundation::primitive::BlendMode;
    Some(match m {
        MixBlendModeComputedValue::Normal => return None,
        MixBlendModeComputedValue::Multiply => BlendMode::Multiply,
        MixBlendModeComputedValue::Screen => BlendMode::Screen,
        MixBlendModeComputedValue::Overlay => BlendMode::Overlay,
        MixBlendModeComputedValue::Darken => BlendMode::Darken,
        MixBlendModeComputedValue::Lighten => BlendMode::Lighten,
        MixBlendModeComputedValue::ColorDodge => BlendMode::ColorDodge,
        MixBlendModeComputedValue::ColorBurn => BlendMode::ColorBurn,
        MixBlendModeComputedValue::HardLight => BlendMode::HardLight,
        MixBlendModeComputedValue::SoftLight => BlendMode::SoftLight,
        MixBlendModeComputedValue::Difference => BlendMode::Difference,
        MixBlendModeComputedValue::Exclusion => BlendMode::Exclusion,
        MixBlendModeComputedValue::Hue => BlendMode::Hue,
        MixBlendModeComputedValue::Saturation => BlendMode::Saturation,
        MixBlendModeComputedValue::Color => BlendMode::Color,
        MixBlendModeComputedValue::Luminosity => BlendMode::Luminosity,
    })
}

/// R4552（css-backgrounds-4 §background-clip:text）：沿 DOM 祖先链查最近 clip:text
/// 恒色祖先。paint 栈（push/pop 时序）无法覆盖「延迟绘制后代」——SC 触发后代
///（filter/transform/opacity 等）被 scope deferred 收集、在 clip:text 宿主 paint_node
/// 出口 pop **之后** flush（SC-child 案实证），栈空时字形失去染色上下文；以静态祖先
/// 链判定补栈盲区。start_id 可为文本节点（parent_node 上溯）。无命中 → None。
pub(super) fn bg_clip_text_solid_color_ancestral(
    doc: &Document,
    start_id: NodeId,
    styles: &HashMap<NodeId, ComputedStyle>,
    solid_colors: &HashMap<u64, [u8; 4]>,
    document_url: Option<&str>,
) -> Option<Color> {
    let mut cur = Some(start_id);
    while let Some(id) = cur {
        if let Some(st) = styles.get(&id)
            && let Some(c) = bg_clip_text_solid_color(st, solid_colors, document_url)
        {
            return Some(c);
        }
        cur = doc.parent_node(id);
    }
    None
}

/// src OVER dst 逐像素 alpha 合成（CSS Compositing §simple alpha compositing；
/// 非预乘 u8 出入，f32 中间精度）。
fn color_over(src: Color, dst: Color) -> Color {
    let mix = |s: u8, d: u8, sa: f32, da: f32| -> u8 {
        let out_a = sa + da * (1.0 - sa);
        if out_a <= f32::EPSILON {
            return 0;
        }
        ((s as f32 / 255.0 * sa + d as f32 / 255.0 * da * (1.0 - sa)) / out_a * 255.0).round() as u8
    };
    let (sa, da) = (src.a as f32 / 255.0, dst.a as f32 / 255.0);
    Color::rgba(
        mix(src.r, dst.r, sa, da),
        mix(src.g, dst.g, sa, da),
        mix(src.b, dst.b, sa, da),
        ((sa + da * (1.0 - sa)) * 255.0).round() as u8,
    )
}
use super::super::helpers::PrimitiveCounts;
use super::super::helpers::apply_text_transform;
use super::text_image::{compute_object_fit_rect, get_img_intrinsic_size};

// 专属 helper 与单测按职责拆入 painter/text/ 子模块。
pub(crate) mod text_list;
mod text_multicol;
mod text_ruby;
mod text_shaping;

use text_multicol::compute_multicol_info_for_paint;
use text_multicol::multicol_balance_target_height;
use text_ruby::{RubyAnnotationSegs, ruby_annotation_segments};
use text_shaping::{
    FragmentPaintWidths, collect_atomic_inline_sizes, configure_paint_ifc_advance as with_shaped_layout,
    fragment_advance_trace, fragment_font_size_adjustment, fragment_glyphs, glyph_sources, is_cc_control_char,
    logical_fragment_source, paint_ifc_baseline_offset, style_open_type_features,
};

impl super::Painter {
    /// 收集浮动子元素的排除区域（带样式映射版本）。
    ///
    /// 遍历 `box_node` 的直接子元素，找出带有 `float: left/right` 样式的子元素，
    /// 计算它们相对于容器内容区域的位置和尺寸。
    pub(super) fn collect_float_exclusions_with_styles(
        &self,
        box_node: &LayoutBox,
        styles: &HashMap<NodeId, ComputedStyle>,
    ) -> Vec<FloatExclusion> {
        let mut exclusions = Vec::new();

        // 容器内容区域的原点偏移（浮动排除区域相对于内容区域左上角计算）
        let content_offset_y = box_node.border_top + box_node.padding_top;
        let content_offset_x = box_node.border_left + box_node.padding_left;

        for child in &box_node.children {
            // 跳过绝对定位子元素（不参与浮动流）
            if child.is_absolute || child.is_fixed {
                continue;
            }

            if let Some(node_id) = child.node_id
                && let Some(child_style) = styles.get(&node_id)
            {
                let is_left = matches!(child_style.float, FloatValue::Left | FloatValue::InlineStart);
                let is_right = matches!(child_style.float, FloatValue::Right | FloatValue::InlineEnd);

                if is_left || is_right {
                    // 浮动子元素相对于容器内容区域的位置
                    let rel_y = child.y - content_offset_y;
                    exclusions.push(FloatExclusion {
                        y: rel_y,
                        height: child.height,
                        width: child.width,
                        x: child.x - content_offset_x,
                        is_left,
                    });
                }
            }
        }

        exclusions
    }

    /// 绘制列表标记（disc/circle/square/decimal 等）。
    /// 绘制 CSS `content` 属性生成的文本内容。
    ///
    /// 当元素的 `content` 属性为 `String` 或 `Counter` 时，
    /// 在元素的内容区域起始位置绘制对应的文本。
    /// 支持计数器值的十进制、小写字母、大写字母、小写罗马、大写罗马格式化。
    pub(crate) fn paint_content(&mut self, box_node: &LayoutBox, abs_x: f32, abs_y: f32, style: &ComputedStyle) {
        // R4045（CSS Content 3 §content-property：content 接受 \<image\>）：渐变 content →
        // element-becomes-replaced-with-gradient——按内容盒绘制渐变（R2439 element-replacement
        // 的非 url 分支；div 100×100 content:linear-gradient 实证页）。draw_list 语义与
        // background-image 渐变一致（gradient_to_primitive 复用）。
        // R5022：filter() 图像函数剥壳——Gradient 内层照常绘制，滤镜不施加
        //（FIXME: 渐变面滤镜 corpus 未涉及）。
        if let zero_style_system::property::types::ContentComputedValue::Gradient(g) = style.content.inner() {
            let content_x = abs_x + box_node.border_left + box_node.padding_left;
            let content_y = abs_y + box_node.border_top + box_node.padding_top;
            let rect = Rect::new(content_x, content_y, box_node.content_width, box_node.content_height);
            let font_size = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
            if let Some(prim) =
                crate::paint::helpers::gradient_to_primitive_with_font_size(g, &rect, &style.color, font_size as f32)
            {
                self.primitives.add_gradient(prim);
            }
            return;
        }
        // R2573：content 文本解析抽出为 resolve_generated_content_text（paint_content 与
        // paint_list_marker 的 ::marker content 覆盖共用）。Normal/None/Attr/Url 无文本 → None。
        let text = match self.resolve_generated_content_text(&style.content, style) {
            Some(t) => t,
            // R1988：content:url() 由 inject_pseudo_text_nodes 注入 `<img>` 元素渲染，paint_content
            //（文本路径）不处理图片；Normal/None/Attr 亦无文本 → 均返回。
            None => return,
        };

        if text.is_empty() {
            return;
        }

        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };
        if font_size <= 0.0 {
            return;
        }

        let color = super::super::color::color_value_to_render(&style.color);
        let (default_font_id, resolved_italic) = self.resolve_style_font_id(&style.font_family, style);
        let variations = crate::text_metrics::paint_font_variations(&style.font_variation_settings);
        let font_variation_id = self.primitives.intern_font_variations(&variations);
        // R2497：font-style:italic/oblique 且 resolved face 非 italic → synthetic italic shear。
        // R3248：font-synthesis:style 为 false 时禁止合成斜体。
        let synthetic_italic = matches!(style.font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_))
            && !resolved_italic
            && style.font_synthesis.style;
        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        let mut char_x = content_x;
        let char_y = content_y + font_size;
        for ch in text.chars() {
            self.primitives.add_glyph(GlyphPrimitive {
                x: char_x,
                y: char_y,
                font_size,
                color,
                glyph_id: ch as u32,
                font_glyph_index: None,
                source: None,
                font_id: default_font_id,
                font_variation_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
                synthetic_italic,
            });
            char_x += self.measure_char_cached(default_font_id.0, ch, font_size, false);
        }
    }

    /// R1660：`<input>` 的 `value` 渲染为可见文本（form-control slice-2）。
    ///
    /// `<input>` 是 void 元素（无 DOM 文本子节点），其 `value` 属性的标签/预填内容此前
    /// 不渲染——submit/reset 按钮无可见文字、text 输入框的预填值不可见。R1659 已按 value
    /// 字符数 / `size` 属性给 input 正确几何宽；本方法在 paint 侧把 value 文本绘出，对齐
    /// Chromium UA 语义：
    /// - submit/reset/button：value（submit/reset 无 value 时默认 "Submit"/"Reset"）水平居中
    ///   （按钮标签居中）
    /// - text 类（text/search/email/url/tel/number + 默认无 type）：value 左对齐于内容盒
    /// - password：每字符渲染为 `•`（密码遮罩语义）
    /// - checkbox/radio/hidden/range/file/image/color/date 等：不渲染 value 文本
    ///
    /// 几何宽由 R1659 UA sizing 决定；此处只补 paint。值超 content 宽时自然溢出（与 chromium
    /// overflow 一致，本 slice 不做 ellipsis）。
    pub(crate) fn paint_input_value(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };
        let node = match doc.get(node_id) {
            Some(n) => n,
            None => return,
        };
        let elem = match &node.kind {
            NodeKind::Element(e)
                if e.local_name().eq_ignore_ascii_case("input") || e.local_name().eq_ignore_ascii_case("textarea") =>
            {
                e
            }
            _ => return,
        };

        let itype = elem.get_attribute("type").unwrap_or_default().to_ascii_lowercase();
        let retained_value = self.form_control_values.get(&node_id).cloned();
        let default_value = if elem.local_name().eq_ignore_ascii_case("textarea") {
            doc.text_content(node_id).unwrap_or_default()
        } else {
            elem.get_attribute("value").unwrap_or_default()
        };
        let mut current_value = retained_value.unwrap_or(default_value);
        if let Some((text, start, end)) = self.form_control_compositions.get(&node_id) {
            current_value = replace_utf16_range(&current_value, *start, *end, text);
        }
        let boundary_value = current_value.clone();
        let current_value_empty = current_value.is_empty();
        // 决定渲染标签 + 是否水平居中。
        let (mut label, center): (String, bool) = if elem.local_name().eq_ignore_ascii_case("textarea") {
            if !self.form_control_values.contains_key(&node_id)
                && !self.form_control_compositions.contains_key(&node_id)
            {
                if current_value.is_empty() {
                    (elem.get_attribute("placeholder").unwrap_or_default(), false)
                } else {
                    return;
                }
            } else {
                (current_value, false)
            }
        } else {
            match itype.as_str() {
                "submit" => (
                    elem.get_attribute("value").unwrap_or_else(|| "Submit".to_string()),
                    true,
                ),
                "reset" => (elem.get_attribute("value").unwrap_or_else(|| "Reset".to_string()), true),
                "button" => (elem.get_attribute("value").unwrap_or_default(), true),
                "password" => (current_value.chars().map(|_| '\u{2022}').collect(), false),
                // 文本类（默认无 type 当 text）。
                "" | "text" | "search" | "email" | "url" | "tel" | "number" => (current_value, false),
                // checkbox/radio/hidden/range/file/image/color/date/... 不渲染 value 文本。
                _ => return,
            }
        };
        let placeholder = elem.get_attribute("placeholder");
        if label.is_empty()
            && let Some(placeholder) = placeholder.as_ref()
        {
            label = placeholder.clone();
        }
        let is_placeholder =
            current_value_empty && placeholder.as_deref().is_some_and(|placeholder| placeholder == label);
        let has_text_boundaries = elem.local_name().eq_ignore_ascii_case("textarea")
            || matches!(itype.as_str(), "" | "text" | "search" | "url" | "tel" | "password");
        if label.is_empty() && !has_text_boundaries {
            return;
        }

        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };
        if font_size <= 0.0 {
            return;
        }

        let color = if is_placeholder {
            // R5051（css-pseudo-4 §4 placeholder-pseudo）：作者 ::placeholder color 优先
            //（style-system 存 owner placeholder_pseudo，仅 color 改变时非 None），
            // 缺省回退 UA 灰（chromium #757575 同值）。
            style
                .placeholder_pseudo
                .as_ref()
                .map(|p| super::super::color::color_value_to_render(&p.color))
                .unwrap_or(zero_render_foundation::color::Color {
                    r: 117,
                    g: 117,
                    b: 117,
                    a: 255,
                })
        } else {
            super::super::color::color_value_to_render(&style.color)
        };
        // R5052（css-pseudo-4 §highlight-selectors）：::selection 选中行高亮 + 字形色。
        // 门：作者 ::selection 规则（style-system 存 owner selection_pseudo）+ harness
        // 注入的非折叠选区 + LTR + 非 password（bullet 映射歧义，两门对齐）。粒度 =
        // 整行：行 UTF-16 区间 ⊆ 选区才着色——部分选中行零变化（部分行高亮 FIXME
        // 挂账；驱动案 selection-input/textarea-011 均为 select() 全选）。
        let line_selection: Option<(usize, usize, Color, Color)> =
            match (self.text_selections.get(&node_id), style.selection_pseudo.as_ref()) {
                (Some(&(start, end)), Some(pseudo))
                    if !is_placeholder
                        && itype != "password"
                        && matches!(style.direction, zero_style_system::DirectionValue::Ltr) =>
                {
                    let sel_bg = super::super::color::resolve_color_current(&pseudo.background_color, &style.color);
                    let sel_color = super::super::color::resolve_color_current(&pseudo.color, &style.color);
                    let total = boundary_value.encode_utf16().count();
                    let (start, end) = (start.min(total), end.min(total));
                    (start < end).then_some((start, end, sel_bg, sel_color))
                }
                _ => None,
            };
        let (default_font_id, synthetic_italic) = self.resolve_style_font_id(&style.font_family, style);

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;
        // R3254-M11：textarea 值可含 '\n'——按行拆分绘制（此前把 '\n' 当普通 glyph 画成
        // 单行粘连）。line-height 简化为 1.2em；整个文本块在 content box 内垂直居中
        //（单行 input 行为与原实现一致）。font box 垂直居中于行盒：
        // https://drafts.csswg.org/css-inline-3/#line-height-property
        let line_height = font_size * 1.2;
        let lines: Vec<&str> = label.split('\n').collect();
        let boundary_lines: Vec<&str> = boundary_value.split('\n').collect();
        // 行 UTF-16 区间预表（≡ 下方 line_utf16_start 累计口径：行长 + 行间 '\n'）。
        let line_ranges: Vec<(usize, usize)> = {
            let mut off = 0usize;
            boundary_lines
                .iter()
                .map(|bl| {
                    let r = (off, off + bl.encode_utf16().count());
                    off = r.1 + 1;
                    r
                })
                .collect()
        };
        // 全行选中 → 高亮矩形铺 padding box（背景绘制区内、UA 边框内缘——≡ ref 页
        // background-color 可见区域，红底测试页由此完全被高亮覆盖）。
        if let Some((s, e, sel_bg, _)) = line_selection
            && sel_bg.a > 0
            && !line_ranges.is_empty()
            && line_ranges.iter().all(|&(a, b)| b == a || (s <= a && b <= e))
        {
            self.primitives.add_fill(
                Rect::new(
                    abs_x + box_node.border_left,
                    abs_y + box_node.border_top,
                    (box_node.width - box_node.border_left - box_node.border_right).max(0.0),
                    (box_node.height - box_node.border_top - box_node.border_bottom).max(0.0),
                ),
                sel_bg,
            );
        }
        let node_handle = crate::hit_test::node_id_to_u64(node_id);
        let text_direction = match style.direction {
            zero_style_system::DirectionValue::Ltr => TextDirection::LeftToRight,
            zero_style_system::DirectionValue::Rtl => TextDirection::RightToLeft,
        };
        let shape_features = style_open_type_features(style);
        let shape_variations = crate::text_metrics::paint_font_variations(&style.font_variation_settings);
        let font_variation_id = self.primitives.intern_font_variations(&shape_variations);
        let shape_adjustment = crate::text_metrics::font_size_adjustment(&style.font_size_adjust);
        let mut line_utf16_start = 0usize;
        let block_height = lines.len() as f32 * line_height;
        let block_top = content_y + (box_node.content_height - block_height).max(0.0) / 2.0;
        for (line_index, line) in lines.iter().enumerate() {
            let line_top = block_top + line_index as f32 * line_height;
            let baseline_y = line_top + (line_height - font_size).max(0.0) / 2.0 + font_size
                - if is_placeholder && !elem.local_name().eq_ignore_ascii_case("textarea") {
                    font_size * 0.875
                } else {
                    0.0
                };
            let shaped = (has_text_boundaries
                && !is_placeholder
                && itype != "password"
                && matches!(text_direction, TextDirection::LeftToRight))
            .then(|| {
                crate::shape_text_for_paint(
                    &[default_font_id.0],
                    line,
                    font_size,
                    text_direction,
                    &shape_features,
                    &shape_variations,
                    shape_adjustment,
                )
            })
            .flatten();
            // 居中按钮标签：先测总宽再定起始 x。
            let total_w: f32 = shaped
                .as_ref()
                .map(|glyphs| glyphs.iter().map(|glyph| glyph.advance_x).sum())
                .unwrap_or_else(|| {
                    line.chars()
                        .map(|ch| self.measure_char_cached(default_font_id.0, ch, font_size, false))
                        .sum()
                });
            let mut char_x = if center {
                content_x + (box_node.content_width - total_w).max(0.0) / 2.0
            } else {
                content_x
            };
            let boundary_line = boundary_lines.get(line_index).copied().unwrap_or_default();
            // R5052：整行选中 → 该行字形用 ::selection color（行区间 ⊆ 选区；空行不算）。
            let line_fully_selected = line_selection.is_some_and(|(s, e, _, _)| {
                line_ranges
                    .get(line_index)
                    .is_some_and(|&(a, b)| b > a && s <= a && b <= e)
            });
            let line_color = match line_selection {
                Some((_, _, _, sel_color)) if line_fully_selected => sel_color,
                _ => color,
            };
            let mut utf16_offset = line_utf16_start;
            if has_text_boundaries {
                self.primitives.add_text_control_boundary(TextControlBoundary {
                    node_handle,
                    utf16_offset: u32::try_from(utf16_offset).unwrap_or(u32::MAX),
                    x: char_x,
                    y: line_top,
                    height: line_height,
                });
            }
            if let Some(shaped) = shaped {
                let sources = glyph_sources(line, &shaped, true).unwrap_or_else(|| vec![None; shaped.len()]);
                let mut cluster = None;
                for (glyph, source) in shaped.iter().zip(sources) {
                    if cluster != source.as_ref().map(|source| (source.start, source.end))
                        && let Some((_, previous_end)) = cluster
                    {
                        utf16_offset = line_utf16_start + line[..previous_end as usize].encode_utf16().count();
                        self.primitives.add_text_control_boundary(TextControlBoundary {
                            node_handle,
                            utf16_offset: u32::try_from(utf16_offset).unwrap_or(u32::MAX),
                            x: char_x,
                            y: line_top,
                            height: line_height,
                        });
                    }
                    cluster = source.as_ref().map(|source| (source.start, source.end));
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x + glyph.x_offset,
                        y: baseline_y - glyph.y_offset,
                        font_size: glyph.font_size,
                        color: line_color,
                        glyph_id: glyph.code_point as u32,
                        font_glyph_index: u16::try_from(glyph.glyph_id).ok(),
                        source,
                        font_id: glyph.font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic,
                    });
                    char_x += glyph.advance_x;
                }
                if let Some((_, end)) = cluster {
                    utf16_offset = line_utf16_start + line[..end as usize].encode_utf16().count();
                    self.primitives.add_text_control_boundary(TextControlBoundary {
                        node_handle,
                        utf16_offset: u32::try_from(utf16_offset).unwrap_or(u32::MAX),
                        x: char_x,
                        y: line_top,
                        height: line_height,
                    });
                }
            } else {
                let mut source_chars = boundary_line.chars();
                for ch in line.chars() {
                    let advance = self.measure_char_cached(default_font_id.0, ch, font_size, false);
                    self.primitives.add_glyph(GlyphPrimitive {
                        x: char_x,
                        y: baseline_y,
                        font_size,
                        color: line_color,
                        glyph_id: ch as u32,
                        font_glyph_index: None,
                        source: None,
                        font_id: default_font_id,
                        font_variation_id,
                        bitmap_width: None,
                        bitmap_height: None,
                        rotation: 0.0,
                        synthetic_italic,
                    });
                    char_x += advance;
                    utf16_offset += source_chars.next().map(char::len_utf16).unwrap_or(1);
                    if has_text_boundaries && !is_placeholder {
                        self.primitives.add_text_control_boundary(TextControlBoundary {
                            node_handle,
                            utf16_offset: u32::try_from(utf16_offset).unwrap_or(u32::MAX),
                            x: char_x,
                            y: line_top,
                            height: line_height,
                        });
                    }
                }
            }
            line_utf16_start += boundary_line.encode_utf16().count();
            if line_index + 1 < boundary_lines.len() {
                line_utf16_start += 1;
            }
        }
    }

    /// 绘制 `<img>` 元素，根据 `object-fit` 属性决定图片如何适配容器。
    ///
    /// - `fill`：拉伸图片填满容器（默认）
    /// - `contain`：等比缩放，完整显示图片
    /// - `cover`：等比缩放，完全覆盖容器
    /// - `none`：原始尺寸
    /// - `scale-down`：取 none 和 contain 中较小的结果
    pub(crate) fn paint_img_element(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        let node_id = match box_node.node_id {
            Some(id) => id,
            None => return,
        };

        let node = match doc.get(node_id) {
            Some(n) => n,
            None => return,
        };

        // 获取图片 URL 作为键。R2439：`content:url(...)` 优先（element-becomes-replaced，
        // 覆盖任何元素含 `<img>` 的正常内容）——src 取自 style.content 的 Url；否则 `<img>`
        // 用 src 属性（src 缺失回退 srcset 首 URL，R2419）。build_subtree 已抑制 content:url
        // 元素的子节点，pipeline 已按 image 固有尺寸 sizing。
        // R5022：filter() 图像函数——剥壳取内层 url，滤镜列表随图元后随 FilterPrimitive
        //（draw_order 插入序：只滤镜已绘图像像素）。
        let content_filters: Option<&Vec<zero_style_system::property::types::FilterComputedValue>> =
            match &style.content {
                zero_style_system::property::types::ContentComputedValue::Filtered { filters, .. } => Some(filters),
                _ => None,
            };
        let src = if let zero_style_system::property::types::ContentComputedValue::Url(u) = style.content.inner() {
            u.clone()
        } else {
            match &node.kind {
                NodeKind::Element(elem) if elem.local_name() == "img" => {
                    let _ = elem;
                    // R4162：src → srcset → picture/source 有效图源（replaced-element-012）。
                    Some(doc)
                        .and_then(|d| crate::effective_img_src(d, node_id))
                        .unwrap_or_default()
                }
                // R3995（HTML §4.8 embedded content）：`<object data>` / `<embed src>` /
                // `<applet src>` 与 img 同为图片资源替换元素——paint 发 ImagePrimitive
                //（image_resource_key 同键），像素由 ImageCache 供（harness 侧已把
                // object/embed 资源解码入缓存）。此前 object/embed 全不渲染图。
                NodeKind::Element(elem) if elem.local_name() == "object" => {
                    elem.get_attribute("data").unwrap_or_default()
                }
                NodeKind::Element(elem) if elem.local_name() == "embed" || elem.local_name() == "applet" => {
                    elem.get_attribute("src").unwrap_or_default()
                }
                _ => return,
            }
        };
        if src.is_empty() {
            return;
        }

        let container_w = box_node.content_width;
        let container_h = box_node.content_height;
        if container_w <= 0.0 || container_h <= 0.0 {
            return;
        }

        // 尝试获取图片的固有尺寸（从 width/height 属性或回退到容器尺寸）
        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        let image_hash = super::super::helpers::image_resource_key(&src, self.document_url.as_deref());
        let image_key = ImageKey::new(image_hash);

        // 与布局阶段保持一致：优先使用解码后的真实图片尺寸；若图片未进入缓存，再回退到
        // HTML width/height 属性，最后才退回容器尺寸。
        let decoded_size = self.get_image_size(image_hash);
        let (intrinsic_w, intrinsic_h) = get_img_intrinsic_size(node, decoded_size, container_w, container_h);

        let (img_x, img_y, img_w, img_h) = compute_object_fit_rect(
            &style.object_fit,
            &style.object_position,
            container_w,
            container_h,
            intrinsic_w,
            intrinsic_h,
            content_x,
            content_y,
        );

        // R5023：阴影形状采样键（add_image 以 shorthand move 所有权，先克隆）。
        let shadow_image_key = image_key.clone();
        self.primitives.add_image(ImagePrimitive {
            rect: Rect::new(img_x, img_y, img_w, img_h),
            image_key,
            clip: Some(Rect::new(content_x, content_y, container_w, container_h)),
            source: None,
            clip_radius: None,
        });

        // R5022：filter() 图像函数（filter-effects-1 #FilterCSSImageValue）——滤镜图元
        // 紧随图像图元发射，rect = 图像绘制区外扩阴影/模糊偏移（css-filters §6 filter
        // region；drop-shadow 位移进滤镜 rect 内采样）。draw_order 按插入序应用 → 只
        // 滤镜已绘图像像素（含 alpha 形状的 drop-shadow 上色），不影响其后续图元。
        if let Some(filters) = content_filters {
            let kinds: Vec<_> = filters
                .iter()
                .filter_map(super::effects::filter_computed_to_kind)
                .collect();
            if !kinds.is_empty() {
                let mut ox = 0.0f32;
                let mut oy = 0.0f32;
                for f in &kinds {
                    match f {
                        zero_render_foundation::primitive::FilterKind::DropShadow(dx, dy, blur, _) => {
                            ox = ox.max(dx.abs() + blur);
                            oy = oy.max(dy.abs() + blur);
                        }
                        zero_render_foundation::primitive::FilterKind::Blur(r) => {
                            ox = ox.max(*r);
                            oy = oy.max(*r);
                        }
                        _ => {}
                    }
                }
                self.primitives
                    .add_filter(zero_render_foundation::primitive::FilterPrimitive {
                        rect: Rect::new(img_x - ox, img_y - oy, img_w + 2.0 * ox, img_h + 2.0 * oy),
                        filters: kinds,
                        // 图像函数 filter()：DropShadow 走 CPU raster alpha 形状上色（R5023）——
                        // 携带图像 key + 绘制 rect + 整图源子矩形供形状采样。
                        raster_drop_shadow_source: Some((
                            shadow_image_key,
                            Rect::new(img_x, img_y, img_w, img_h),
                            Rect::new(0.0, 0.0, 1.0, 1.0),
                        )),
                        clip_polygon: None,
                    });
            }
        }

        // R5273（css-pseudo-4 §highlight-painting + Bugzilla 1449010「::selection
        // is respected on images」）：图片替换元素被选区覆盖且作者声明非透明
        // `::selection` 背景时，图片内容上以 0.8 alpha 叠加该底色——不透明图
        // 片背后铺底不可见（corpus blue15x15 实证纯不透明 RGB），作者声明面对
        // 图片失去可见性即失义。chromium 151 活体探针（红/白/绿/灰四底，
        // /tmp/r5273）定量：红案逐通道自洽 α=0.8（R 204=255·0.8 / B 51=255·0.2，
        // 条带零通道同 α），白/灰/绿底中通道偏差为 chromium 色管理混合（非逐
        // 通道成分，精确公式不可自截图复原——保真边界记录；mismatch 判据只
        // 要求 TEST/REF 图片区域可见差异）。门序：选区 map 空（product 零选区
        // 零扫描）→ 作者声明面（selection_pseudo 仅页含 ::selection 规则时存，
        // R5054/R5065）→ kill-switch；无作者规则路径（UA 默认域）与无选区页
        // 字节不动。选区覆盖 = 整 contents 判据（≡ 盒级 anc_whole 口径：
        // selectNode(html) 形记录落祖先链；img 自身 textContent 空，任意自身
        // 记录即整选）。部分选区覆盖图片维持挂账（corpus 无案形）。
        if !self.text_selections.is_empty()
            && img_w > 0.0
            && img_h > 0.0
            && let Some(pseudo) = style.selection_pseudo.as_ref()
            && std::env::var("ZW_SELECTION_IMG_BG").as_deref() != Ok("0")
        {
            let sel_bg = super::super::color::resolve_color_current(&pseudo.background_color, &style.color);
            let selected = self
                .text_selections
                .get(&node_id)
                .map(|&(s, e)| {
                    doc.text_content(node_id)
                        .map(|t| t.encode_utf16().count())
                        .is_some_and(|n| s == 0 && e >= n)
                })
                .unwrap_or_else(|| {
                    let mut cur = doc.parent_node(node_id);
                    while let Some(aid) = cur {
                        if let Some(&(s, e)) = self.text_selections.get(&aid) {
                            // 首个有记录的祖先判定整 contents（≡ sel_owned 祖先
                            // 链臂：部分记录即否，不继续上溯）。
                            let total = doc.text_content(aid).map(|t| t.encode_utf16().count()).unwrap_or(0);
                            return s == 0 && e >= total;
                        }
                        cur = doc.parent_node(aid);
                    }
                    false
                });
            if selected && sel_bg.a > 0 {
                // 0.8 alpha 覆写（chromium 红案逐通道自洽值；css-pseudo-4 高亮
                // 面呈现须保图片内容可辨）。
                self.primitives.add_fill(
                    Rect::new(img_x, img_y, img_w, img_h),
                    Color::rgba(sel_bg.r, sel_bg.g, sel_bg.b, 204),
                );
            }
        }
    }

    /// 绘制 `<video>` 元素的当前帧（media-playback M1b 帧上屏通路）。
    ///
    /// R3268 canvas 同款两段式：painter 只按 `src` 的资源哈希发 [`ImagePrimitive`]；
    /// 像素由调用方（reftest harness / 渲染线程）经 `zero-media` 解码后注入
    /// `ImageCache`（key = `image_resource_key(src)`）。无解码像素时无图元——
    /// 元素盒装饰（背景/边框）照常绘制，保持占位行为不变。
    ///
    /// https://html.spec.whatwg.org/multipage/media.html#the-video-element
    /// （帧内容按 `object-fit` 适配容器，与 `<img>` 同一 replaced 元素语义）。
    pub(crate) fn paint_video_element(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
    ) {
        let Some(node_id) = box_node.node_id else {
            return;
        };
        let Some(node) = doc.get(node_id) else {
            return;
        };
        let NodeKind::Element(elem) = &node.kind else {
            return;
        };
        if elem.local_name() != "video" {
            return;
        }
        // R4569（HTML §4.8.6 poster frame）：无可播帧（无 src / 首帧未解码）时回退
        // poster 帧——poster 是静态图，img 同链路（ImageCache 由 runner 的 poster
        // 收集注入，image_resource_key 同键）。poster 也缺失/未解码 → 占位行为不变。
        // css-content-3 §3：`content: url()` 替换元素内容**整体替换**（R2439 img 同款）
        // ——优先于 src/poster（element-replacement-on-replaced-element：video
        // content:url(yellow) + poster=blue 应黄不应蓝）。
        let src = match &style.content {
            zero_style_system::property::types::ContentComputedValue::Url(u) => Some(u.clone()),
            _ => None,
        }
        .or_else(|| {
            elem.get_attribute("src").filter(|s| !s.is_empty()).and_then(|src| {
                let hash = super::super::helpers::image_resource_key(&src, self.document_url.as_deref());
                self.get_image_size(hash).map(|_| src)
            })
        })
        .or_else(|| elem.get_attribute("poster").filter(|s| !s.is_empty()));
        let Some(src) = src else {
            return;
        };
        let url_hash = super::super::helpers::image_resource_key(&src, self.document_url.as_deref());
        // 仅当调用方已注入解码像素（固有尺寸可用，canvas snapshot 同款 gate）时才
        // 发图元——无解码像素时保持占位行为（元素盒装饰照常，零回归）。
        let Some((decoded_w, decoded_h)) = self.get_image_size(url_hash) else {
            return;
        };
        let container_w = box_node.content_width;
        let container_h = box_node.content_height;
        if container_w <= 0.0 || container_h <= 0.0 {
            return;
        }

        let content_x = abs_x + box_node.border_left + box_node.padding_left;
        let content_y = abs_y + box_node.border_top + box_node.padding_top;

        let image_key = ImageKey::new(url_hash);
        // 固有尺寸 = 解码首帧尺寸；退化（0 维）时回退容器尺寸（1:1 填充语义）。
        let intrinsic_w = if decoded_w > 0.0 { decoded_w } else { container_w };
        let intrinsic_h = if decoded_h > 0.0 { decoded_h } else { container_h };

        let (img_x, img_y, img_w, img_h) = compute_object_fit_rect(
            &style.object_fit,
            &style.object_position,
            container_w,
            container_h,
            intrinsic_w,
            intrinsic_h,
            content_x,
            content_y,
        );

        self.primitives.add_image(ImagePrimitive {
            rect: Rect::new(img_x, img_y, img_w, img_h),
            image_key,
            clip: Some(Rect::new(content_x, content_y, container_w, container_h)),
            source: None,
            clip_radius: None,
        });
    }

    /// 绘制文本内容（生成多字符 GlyphPrimitive）。
    pub fn paint_text(
        &mut self,
        box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: Option<&Document>,
        styles: Option<&HashMap<NodeId, ComputedStyle>>,
    ) {
        if let (Some(node_id), Some(doc)) = (box_node.node_id, doc)
            && (self.form_control_values.contains_key(&node_id)
                || self.form_control_compositions.contains_key(&node_id))
            && doc.get(node_id).is_some_and(
                |node| matches!(&node.kind, NodeKind::Element(element) if element.local_name().eq_ignore_ascii_case("textarea")),
            )
        {
            return;
        }
        let abs_y = if matches!(
            style.appearance,
            zero_style_system::AppearanceComputedValue::Button
                | zero_style_system::AppearanceComputedValue::PushButton
                | zero_style_system::AppearanceComputedValue::SquareButton
        ) && box_node.node_id.is_some_and(|node_id| {
            doc.and_then(|doc| doc.get(node_id)).is_some_and(
                |node| matches!(&node.kind, NodeKind::Element(element) if element.local_name().eq_ignore_ascii_case("button")),
            )
        }) {
            abs_y + 2.0
        } else {
            abs_y
        };
        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };

        // R5292（CSS2 §16 font-size 继承）：`font-size:0` 容器的早期返回只对「自身文本
        // 全为 0px」成立——该门是 glyph-only 时代（d4d910c14）的遗留，此后 paint_text
        // 已接管 stored replay / Path B 子孙文本绘制，行内子 span 携带自有非零
        // font-size（`font-size:0` + span 正是 WPT 抑制空白的惯用法）时整盒跳过会把
        // 子孙文本留给 16px 兜底路径错绘（table-vertical-align-baseline-009 的 X
        // 50px→16px 实证）。仅当无任何 stored run 带非零字号时维持跳过（直系 0px
        // 文本不可见语义不变；store 空盒保守回退旧行为）。
        // kill-switch `ZW_PAINT_FS0_STORED_BYPASS=0`。
        static FS0_STORED_BYPASS: std::sync::LazyLock<bool> =
            std::sync::LazyLock::new(|| std::env::var("ZW_PAINT_FS0_STORED_BYPASS").as_deref() != Ok("0"));
        if font_size <= 0.0 && (!*FS0_STORED_BYPASS || box_node.text_node_font_sizes.values().all(|&fs| fs <= 0.0)) {
            return;
        }

        if style.color == ColorValue::CurrentColor {
            return;
        }

        let color = color_value_to_render(&style.color);

        let letter_spacing: f32 = match style.letter_spacing {
            LengthValue::Px(s) => s as f32,
            // R4228（css-text-4 #letter-spacing）：百分比相对 used font-size。
            LengthValue::Percentage(p) => font_size * (p as f32 / 100.0),
            ref lv => zero_style_system::computed::resolve_length(lv, font_size as f64, None, None) as f32,
        };
        let word_spacing: f32 = match style.word_spacing {
            LengthValue::Px(s) => s as f32,
            LengthValue::Percentage(p) => font_size * (p as f32 / 100.0),
            ref lv => zero_style_system::computed::resolve_length(lv, font_size as f64, None, None) as f32,
        };

        // R2305：text-shadow 多阴影列表（CSS Text Decoration §3：`none | <shadow>#`）。
        // 预解析非零阴影（offset/blur 全零 = 不可见，跳过；与既有 has_text_shadow 语义一致），
        // 颜色预解析避免逐 glyph 重复解析。空 Vec = none。
        // R2364：颜色按元素 `color` 解析 currentColor（省略颜色默认 currentColor，CSS Text Deco §3）。
        let active_text_shadows: Vec<(f32, f32, Color)> = style
            .text_shadow
            .iter()
            .filter(|ts| !(ts.offset_x == 0.0 && ts.offset_y == 0.0 && ts.blur_radius == 0.0))
            .map(|ts| (ts.offset_x, ts.offset_y, resolve_color_current(&ts.color, &style.color)))
            .collect();

        let content_x = abs_x
            + box_node.border_left
            + box_node.padding_left
            // R5264：collapse 格线模型 cell 内容原点半框修正（table.rs
            // apply_collapse_content_half_shift 设置，非 collapse 盒恒 (0,0)）。
            + box_node.collapse_half_shift.0;
        // R3835：inside 文本型 marker → 内容起点右移 marker 步进宽度（CSS Lists 3
        // §list-style-position：inside marker 是首行行盒第一个 inline，内容排其后）。
        // remove 消费一次；paint_list_marker（先于本函数执行）已按 li NodeId 填充。
        // 近似说明：IFC 起点全市偏移会使多行 li 的后续行同样缩进（chromium 仅首行），
        // 但 ZW block-IFC 无逐行 marker 模型；现状（marker 与内容重叠）远劣于此。
        // R4323：RTL（水平书写模式）行内起点在右——inside marker 占据行右端 A，
        // 内容线整体左移 A（IFC 对齐域 = [content_x, content_x + container_width]，
        // origin 平移即整线平移：Path A stored 片段与 Path B 重跑同受此坐标约定）。
        let marker_advance = match box_node.node_id {
            Some(id) => self.list_inside_marker_advance.remove(&id).unwrap_or(0.0),
            None => 0.0,
        };
        let content_x = if matches!(style.direction, zero_style_system::DirectionValue::Rtl)
            && matches!(style.writing_mode, zero_style_system::WritingModeValue::HorizontalTb)
        {
            content_x - marker_advance
        } else {
            content_x + marker_advance
        };

        // R1717：+ valign_offset — 表格单元格文本的 vertical-align 内容偏移（仅 table-cell，
        // table.rs position_cells 设置；其他盒默认 0.0，零影响）。
        // R5264：+ collapse_half_shift.1 — collapse 格线模型 cell 内容原点半框修正
        //（table.rs apply_collapse_content_half_shift 设置，非 collapse 盒恒 0.0）。
        let content_y = abs_y
            + box_node.border_top
            + box_node.padding_top
            + box_node.valign_offset
            + box_node.collapse_half_shift.1;

        // R3901：transform translate 不在此偏移——paint_node 末段对自子树全部图元统一做
        // 图元级平移（translate_primitives_since，单一平移点）。旧实现仅直属文本 +tx/ty，
        // 背景/边框/marker/后代不动（transform-overflow-001 等 4.17% 簇的根因：translate
        // 只对文字生效）。此处 tx/ty 恒 0，保留变量供下游排版度量（content_right 等 + tx
        // 语义不变——transform 本就不该影响排版度量）。
        let (tx, ty) = (0.0_f32, 0.0_f32);

        let (default_font_id, default_resolved_italic) = self.resolve_style_font_id(&style.font_family, style);
        let default_variations = crate::text_metrics::paint_font_variations(&style.font_variation_settings);
        let default_font_variation_id = self.primitives.intern_font_variations(&default_variations);
        // R2497：容器 font-style:italic/oblique → container_want_italic（macro 据 owner
        // per-fragment font_style 覆盖，缺省回落此值）。
        let container_want_italic = matches!(style.font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_));
        // R1224：Ahem font_id 供 inline 元素字体≠容器时字形位图用（如 <span font:Ahem> 在
        // default div 内）。render_fragment macro 按 owner（片段父元素）font_family 选
        // frag_font_id——is_ahem 片段用 ahem_font_id 出 Ahem 方块，非 is_ahem 用 default。
        let ahem_font_id = self.resolve_style_font_id(&["Ahem".to_string()], style).0;
        // R1464/R2497：恢复 Path B 每节点 face/list 与 synthetic-italic 状态。
        let resolved_text_fonts = self.resolve_text_node_fonts(box_node, style, doc);
        let text_node_font_ids = resolved_text_fonts.primary;
        let text_node_shaping_font_ids = resolved_text_fonts.shaping;
        let text_node_font_italic = resolved_text_fonts.italic;

        if let (Some(doc), Some(node_id)) = (doc, box_node.node_id) {
            // R109 §9.2.1.1：被 in-flow block 子元素拆分的 inline 父盒自身不渲染文本——
            // 其直接文本已由匿名块片段子盒（带 fragment_node_ids）渲染。避免与片段重叠。
            // R3893：block 容器混合内容拆分（§9.2.1.1 ②）的宿主盒同理——其直接文本由
            // Inline 匿名块片段渲染；宿主自身 Path B 重跑 IFC（空 styles）会丢失 block
            // 子分类、把 block 子树文本吸收进容器流（错位 + 丢 span 样式的 text
            // concatenation）。片段是 plain Block 不继承盒模型，故仅抑制文本（装饰守卫
            // skip_split_inline_deco 不含此旗标，宿主 bg/border 照绘）。
            if (box_node.is_r109_split || box_node.is_r109_block_mixed) && box_node.fragment_node_ids.is_none() {
                return;
            }
            // R3992：并入本容器首行的 run-in（run_in_prepended）使容器 paint_text 须跑——
            // 即便容器自身无直接文本子（run-in-block-between-001：run-in 并入**空块**，
            // run-in 文本由 IFC 前置收集提供，has_direct 早退会丢整行）。与 layout 侧
            // compute_final 的 has_text_children 放行同源。
            // R4489：匿名块片段盒（fragment_node_ids）渲染的是**片段内容**——host 直子
            // 文本判据不适用（片段可全为 inline 元素、host 直子仅空白文本：
            // insert-inline-in-blocks-n-inlines-* 族的 Inline 片段 = [span, span]）。
            // 片段仅在非空白时创建（is_whitespace_only_inline_segment 跳过），故放行。
            // R5275（css-text-3 §3）：preserve 容器（pre/pre-wrap）的纯空白文本是保留
            // 内容——has_direct_paintable_text 的 has_preserved_content 窄集合判据（R5055
            // collapse 口径）不看 white-space 上下文，pre 纯 tab div 被判无可绘制文本 →
            // 本门整跳 paint_text，行盒字形/选区高亮全缺（active-selection-063 修宽片
            // 后仍全红实证：盒 bg 已出而选区 fill 无，paint_text 在本门早退）。放行限
            // **纯文本子**容器（063 形）：含元素子时宿主 Path B 重跑会把块子树文本吸进
            // 容器流（R109 同型，outside-marker-paint-order 内嵌 pre+ol 3.48% 实证）；
            // ws + inline 元子由 has_direct_paintable_text 的 PHASEA 纯 inline 臂放行，
            // 无需本臂。collapse 语境判据不变。
            let children = doc.child_nodes(node_id);
            let has_element_child = children
                .iter()
                .any(|child_id| matches!(doc.get(*child_id).map(|n| &n.kind), Some(NodeKind::Element(_))));
            let preserve_direct_text = !has_element_child
                && matches!(
                    styles.and_then(|m| m.get(&node_id)).map(|s| &s.white_space),
                    Some(WhiteSpaceValue::Pre | WhiteSpaceValue::PreWrap)
                )
                && children.iter().any(|child_id| {
                    matches!(
                        doc.get(*child_id).map(|n| &n.kind),
                        Some(NodeKind::Text(t)) if !t.content.is_empty()
                    )
                });
            if box_node.fragment_node_ids.is_none()
                && !has_direct_paintable_text(doc, node_id, styles)
                && !preserve_direct_text
                && box_node.run_in_prepended.is_none()
            {
                return;
            }
            // R5209（CSS2 §17.2.1）：孤立 retrofit 行组不是 IFC 容器——直系裸文本已由
            // R5209 合成匿名 cell（is_anon_table_cell 旗标，借行组 node_id + fragment
            // 圈定）接管绘制。行组自身 paint_text 的 Path B 重跑会把整子树 cell 文本
            // 扁平吸进单 IFC（table-internal 盒不播种字体键（R4233/R4558 收窄）→
            // 16px 兜底度量 + 窄容器换行，161 静态 probe 实证垃圾重绘叠在 32px 正确
            // 渲染上）。抑制行组级直绘，cell 文本由各 cell 盒自绘。真 table 内行组
            //（anontbl=false）与 vertical 域维持现状零回归面；合成 cell 借行组
            // node_id 但持 is_anon_table_cell 旗标，不落入本门。
            if box_node.is_anon_table_root
                && !box_node.is_anon_table_cell
                && !box_node.writing_mode.is_vertical_block_flow()
                && styles.is_some_and(|m| {
                    m.get(&node_id).is_some_and(|s| {
                        matches!(
                            s.display,
                            DisplayValue::TableRowGroup
                                | DisplayValue::TableHeaderGroup
                                | DisplayValue::TableFooterGroup
                        )
                    })
                })
            {
                return;
            }
            // R5289（CSS2 §17.2.1）：display:table-row 的 inline 子已被合成匿名 cell
            // 接管（R5168 wrap_inline_runs_in_row：cell 借行 node_id + fragment 圈定，
            // 自带 stored/Path B 自绘）——行盒自身 Path B 重跑把整行 inline 内容扁平
            // 吸进单 IFC 重绘，跨元素空白文本节点以兜底字体计空格 advance（应随相邻
            // run 字体）→ 每跨 span 边界 advance 偏小，鬼影错位副叠在合成 cell 正确副
            // 上（table-anonymous-objects-085/086 行 1 实证：32px monospace 空格 16px
            // 兜底成 8px）。抑制行级直绘，文本由合成 cell 自绘；无合成 cell 子（run-in
            // / 纯块子等未接管形态）与 vertical 维持现状零回归面。
            if row_anon_cell_suppress_enabled()
                && !box_node.writing_mode.is_vertical_block_flow()
                && box_node.run_in_prepended.is_none()
                && matches!(style.display, DisplayValue::TableRow)
                && box_node.children.iter().any(|c| c.is_anon_table_cell)
            {
                return;
            }
            // IFC 前置渲染；自盒 Path B 仍按 DOM 子文本绘出会在块外重影
            //（run-in-breaking-001「Run-in header」块外重影实证：target PathB 79
            // runs 含前置内容，自盒 78 runs 同文本再绘）。
            if box_node.is_run_in_merged {
                return;
            }
            // R109：匿名块片段跳过 painted_inline_nodes 去重——多个片段共享 inline 的
            // node_id，首个片段渲染后会标记该 id，须放行后续片段各自渲染其片段文本。
            // R1548/R1549：vertical 容器内全 inset auto 的 abspos/fixed 盒由
            // fix_vertical_mode_abs_pos 在 postprocess 期精确定位（IFC 静态位置 + height
            // shrink-to-fit）+ 存入自身 font metrics（R1548），其文本须由自身 paint_text
            // 绘制。但其 containing-block 的 IFC 也会收集该文本片段（用于静态位置计算）并以
            // 容器色（R335/R358，常 transparent）绘在静态流位置 + 标记 node painted → 抑制
            // abspos 盒自身绘制 → 文本消失（R1547 root cause）。判据 = abspos 盒的
            // text_node_font_sizes 非空（**仅** fix_vertical_mode_abs_pos 会为 abspos 子盒
            // 填充它 = vertical 容器 + 全 inset auto + 盒位已修正；覆盖 span 继承 vertical 与
            // span 显式 horizontal-tb 两类）。非 auto inset 的 abspos 盒无此填充（定位未修正），
            // 仍走去重（旧行为）——避免绘在 taffy 错误位（vrl-038/042/044/048 top:1em）。
            let abspos_self_paint =
                (box_node.is_absolute || box_node.is_fixed) && !box_node.text_node_font_sizes.is_empty();
            if box_node.fragment_node_ids.is_none()
                && !abspos_self_paint
                && self.painted_inline_nodes.contains(&node_id)
            {
                return;
            }

            // R1099 Slice α-1（vertical-mode IFC 四层协调）：container_width WM-aware。
            // vertical-rl/lr 下 IFC 重跑须与 layout 侧（inline_finalization.rs）同取 content_height
            //（竖直 inline 尺寸 = 字符向下推进可用深度），非 content_width。horizontal-tb 零回归。
            // decoration-gate（TBD-2）：vertical 容器子树有 text-decoration/emphasis 时保持
            // content_width（旧行为），回避 Layer 4 装饰坐标耦合（α-3 未实施）。
            let is_vertical_wm = matches!(
                style.writing_mode,
                zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
            );
            let vertical_decoration_free = styles.is_some_and(|s| {
                box_node
                    .node_id
                    .is_some_and(|id| !subtree_has_text_decoration(doc, s, id))
            });
            let container_width = if is_vertical_wm && vertical_decoration_free {
                box_node.content_height
            } else {
                box_node.content_width
            };

            // 检测是否为多列容器（无块级子元素但有 inline 内容）
            // 如果是，使用列宽创建 IFC，并在渲染时将行分配到各列。
            // 条件：
            // 1. 无 inflow 子元素（纯 inline 内容）
            // 2. column-fill: balance（默认值，非 auto 顺序填充）
            // 注意：对于纯 inline 内容，有明确高度时 balance 模式仍需分配到各列。
            // column-fill: auto 的 inline 内容由 layout 层处理（有 height 限制时），
            // 此处仅处理 balance 模式（无论有无 height）。
            let has_in_flow_children = box_node
                .children
                .iter()
                .any(|c| !c.is_absolute && !c.is_fixed && c.is_block_level);
            let is_balance_mode = !matches!(style.column_fill, zero_style_system::ColumnFillComputedValue::Auto);
            // 仅对 height:auto 的纯行内 multicol 容器做列分配。明确高度的 balance 容器
            // （常见于嵌套 multicol / column-breaking 测试）涉及 column breaking，
            // 当前简单均衡分配会回归这类用例，回退到单块渲染。
            let height_auto = matches!(style.height, LengthValue::Auto);
            let multicol_info = if box_node.is_multicol_region_fragment {
                // R4499：multicol spanner 区域平衡 inline 片段——盒宽已是容器列宽，
                // 列几何不能再按盒宽求（compute_column_info(style, 列宽) 得退化 0 列宽）。
                // 列数/gap 取自宿主 style（本盒 style 即宿主 style），列宽 = 盒内容宽；
                // 下方多列分支按容器列数把行分配到各列（x = col_idx×(col_w+gap)）。
                if is_balance_mode {
                    compute_multicol_info_for_paint(style, container_width).map(|mut mc| {
                        mc.col_width = container_width;
                        mc
                    })
                } else {
                    None
                }
            } else if !has_in_flow_children && is_balance_mode && height_auto {
                compute_multicol_info_for_paint(style, container_width)
            } else {
                None
            };

            // 多列容器使用列宽创建 IFC
            let ifc_width = if let Some(ref mc) = multicol_info {
                mc.col_width
            } else {
                container_width
            };

            // R4392 临时探针（ZW_DEBUG_MEASURE=1）：paint_text 分发记账。
            if std::env::var("ZW_DEBUG_MEASURE").as_deref() == Ok("1")
                && let Some(id) = box_node.node_id
                && let Some(node) = doc.get(id)
                && let zero_dom::NodeKind::Element(e) = &node.kind
            {
                let ln = e.local_name();
                if ln == "div" || ln == "ruby" || ln == "aside" {
                    eprintln!(
                        "[paint_text] {ln} dom={id:?} stored={} w={:.2} ifc_w={:.2} children={}",
                        box_node.inline_layout.is_some(),
                        box_node.width,
                        ifc_width,
                        box_node.children.len()
                    );
                }
            }

            // R2577：word-break: break-word（CSS Text 3 legacy）≡ overflow-wrap: break-word。
            let break_word = matches!(
                style.overflow_wrap,
                zero_style_system::OverflowWrapValue::BreakWord | zero_style_system::OverflowWrapValue::Anywhere
            ) || matches!(style.word_break, zero_style_system::WordBreakValue::BreakWord);

            // 根据 white-space 属性设置换行和空白保留行为
            let (mut no_wrap, preserve_whitespace, break_at_newline) = match style.white_space {
                WhiteSpaceValue::Normal => (false, false, false),
                WhiteSpaceValue::Nowrap => (true, false, false),
                WhiteSpaceValue::Pre => (true, true, false),
                WhiteSpaceValue::PreWrap => (false, true, false),
                // pre-line：空白序列折叠但 `\n` 强制断行（CSS Text 3 §4.2）。
                // kill-switch ZW_PRELINE_NEWLINE_BREAK=0 恢复旧行为（与 inline_finalization 对称）。
                WhiteSpaceValue::PreLine => (
                    false,
                    false,
                    std::env::var("ZW_PRELINE_NEWLINE_BREAK").as_deref() != Ok("0"),
                ),
                WhiteSpaceValue::BreakSpaces => (false, true, false),
            };

            // CSS text-wrap: nowrap 覆盖换行行为
            if let Some(wrap_override) = super::Painter::resolve_text_wrap(style) {
                no_wrap = wrap_override;
            }

            // CSS line-clamp: 限制最大行数
            let max_lines = super::Painter::resolve_line_clamp(style);

            // R2191：word-break + line-break:anywhere 经共享 resolver（与 layout Path A 同源）。
            let word_break_mode = resolve_word_break_mode(style);

            // R958 双路径同源：text-align / text-align-last 经 layout 路径的共享 resolver
            //（inline_finalization::resolve_text_align[_last]）解析，消除 paint Path B 此前内联的
            // 重复 match（与 layout Path A 两份独立拷贝，曾潜伏 IFC Path A/B 分歧风险）。start/end
            // 方向感知（CSS Text 3 §6.1/§6.2）、Auto = 跟随 text-align，均由 resolver 统一实现。
            let text_align = resolve_text_align(Some(style));
            let text_align_last = resolve_text_align_last(Some(style));

            // R958 双路径同源：text-indent 经 layout 路径的共享 resolver（resolve_text_indent）解析，
            // 消除 paint Path B 此前内联的重复 match（与 layout Path A 两份独立拷贝，曾潜伏 IFC Path A/B
            // 分歧风险）。Px / Em（×font_size）/ Percentage（×container_width），CSS §10.3.1。Path B 的
            // font_size 已在 line 361 保证为 style.font_size 的 Px（非 Px 早 return），故与 resolver 内部
            // font_size_px（同取 style.font_size Px，16.0 防御回退在此不可达）等价。
            let text_indent_px = resolve_text_indent(&style.text_indent, &style.font_size, container_width);

            // CSS tab-size: Number(n) 是空格倍数，Length 是实际长度。
            let tab_size = match &style.tab_size {
                TabSizeValue::Number(n) => (*n as f32, false),
                TabSizeValue::Length(length) => (
                    resolve_tab_size_length_px(length, &style.font_size).unwrap_or(8.0),
                    true,
                ),
            };

            // 收集浮动子元素的排除区域
            let float_exclusions = styles
                .map(|s| self.collect_float_exclusions_with_styles(box_node, s))
                .unwrap_or_default();

            let is_vertical = matches!(
                style.writing_mode,
                zero_style_system::WritingModeValue::VerticalRl | zero_style_system::WritingModeValue::VerticalLr
            );
            let is_vertical_rtl = matches!(style.writing_mode, zero_style_system::WritingModeValue::VerticalRl);

            // 尝试使用布局引擎存储的行内布局结果，避免重新运行 IFC。
            // 条件：(1) 非多列模式 (2) 有存储结果 (3) 容器宽度匹配
            // 宽度验证确保 table/multicol 后处理改变宽度时回退到 paint IFC。
            let width_matches = (box_node.inline_layout_width - ifc_width).abs() < 1.0;
            let use_stored = multicol_info.is_none() && box_node.inline_layout.is_some() && width_matches;

            // 从存储结果创建的扁平化片段列表（用于非多列渲染路径）
            struct PaintFragment {
                x: f32,
                y: f32,
                // R4332：片段所属行盒顶（container-rel = line.y，horizontal）。多行 inline
                // 盒装饰（R1442 横条/竖边）的行锚；vertical 模式 line.y 是列 x，不适用
                //（R1442 块本就 !is_vertical 门控，恒 0 不消费）。
                line_top_rel: f32,
                // R817 Phase 2：片段基线绝对 y（container-rel = line.y + line.baseline_y）。
                // 供 is_ahem glyph 定位用（见 stored 渲染循环），paint 非存储路径不读。
                baseline_y_abs: f32,
                width: f32,
                font_size: f32,
                is_ahem: bool,
                is_ahem_font: bool,
                text: String,
                source: Option<zero_layout_engine::TextFragmentSource>,
                node_id: NodeId,
                // R4233：片段自身 letter-spacing（layout IFC 按 run 记账）——paint 侧
                // glyph advance 须用片段值而非容器盒值（span 声明 ls 与容器不同时，
                // 旧实现用容器值 → 字形间距/宽度与布局错位，c542-letter-sp-001 行 3/4）。
                letter_spacing: f32,
            }

            let stored_fragments: Vec<PaintFragment> = if use_stored {
                box_node
                    .inline_layout
                    .as_ref()
                    .unwrap()
                    .iter()
                    .flat_map(|line| {
                        // R355：多行存储需把行盒垂直偏移（line.y）加到片段 y 上——
                        // 存储片段 f.y 是行内相对（恒为 0），line.y 才是行盒在容器内的位置。
                        // R207 单行存储时 line.y==0 故无影响；R355 多行若不加 line.y，
                        // 所有行渲染在容器顶部 y=0 互相覆盖（ifc-008 底半红露白）。
                        let line_y = line.y;
                        line.fragments.iter().filter_map(move |f| {
                            f.node_id.map(|nid| PaintFragment {
                                x: f.x,
                                // R4332：行盒顶（horizontal = line.y；vertical 是列 x 不适用）。
                                line_top_rel: if is_vertical { 0.0 } else { line_y },
                                // R1456：垂直模式下 line.y 是**列 x 坐标**（inline/mod.rs:1551
                                // vertical_rtl 轴交换把列 x 存进 col.y/line.y），已在 f.x（= run.x
                                // = 列 x）中体现，**不可**再加到片段 y（深度）。旧行为 line_y+f.y
                                // 把列 x（如 764）误加到深度（0）→ frag_y=764 → 文本推到 viewport
                                // 外（vrl-011 等全 0 可见）。horizontal 仍 line_y+f.y（line.y 是行盒
                                // y 偏移，R355）。WM gate 零回归。
                                y: if is_vertical { f.y } else { line_y + f.y },
                                baseline_y_abs: if is_vertical {
                                    f.baseline_y
                                } else {
                                    line_y + f.baseline_y
                                },
                                width: f.width,
                                font_size: f.font_size,
                                is_ahem: f.is_ahem,
                                is_ahem_font: f.is_ahem_font,
                                text: f.text.clone(),
                                source: f.source.clone(),
                                node_id: nid,
                                letter_spacing: f.letter_spacing,
                            })
                        })
                    })
                    .collect()
            } else {
                Vec::new()
            };

            // 非存储模式下运行 IFC
            let inline_ctx = if use_stored {
                InlineFormattingContext::new(ifc_width)
            } else {
                // R72: 恢复 override maps 机制。
                // 传递真实 styles 会导致 4 个测试回归（BFC-004, font-feature-002,
                // position-absolute-in-inline-005/006），虽然修复了 float-003。
                // override maps 方式是经过 R37-R71 验证的安全路径。
                // 仅纳入文本节点片段构建父级映射。
                // text_node_* 中混入了内联元素片段（如 <img>，其 font_size=0、height=96），
                // 它们与文本片段共享同一父元素；直接 collect 时 last-write-wins，
                // 结果随 HashMap 迭代顺序（每进程随机）变化 → 渲染非确定性（flaky reftest）。
                // 过滤为纯文本节点后，同一父元素的文本节点继承一致的字号/行高，结果确定。
                let is_text = |tn: zero_dom::NodeId| matches!(doc.get(tn).map(|n| &n.kind), Some(NodeKind::Text(_)));
                let mut parent_font_sizes: NodeIdMap<f32> =
                    build_text_parent_override_map(doc, &box_node.text_node_font_sizes);
                // R4558：直系文本 font-size 兜底——盒无 stored per-fragment 键时（定高/
                // content 关键字高度盒等非 stored 路径），重跑 IFC 的直系文本 run 查不到
                // fs 覆盖会落 16px 默认，行断按错误字宽推进。以盒 id 播种兜底（R4233
                // letter-spacing 同构先例）：paint_text 入口已保证 style.font_size 为 Px。
                // 收窄（corpus A/B 定谳）：垂直书写模式（行高/字号的块轴语义经轴交换，
                // 盒自身 style 不等于文本 run 度量，text-overflow-vertical 族 6 案翻红）
                // 与 table-internal 盒（匿名盒样式继承与文本 run 实际归属分歧，
                // table-anonymous-objects 族 12 案翻红）不播种。kill-switch
                // `ZW_PAINT_DIRECT_TEXT_SEED=0`。
                let direct_text_seed = direct_text_seed_enabled()
                    && matches!(
                        style.display,
                        DisplayValue::Block
                            | DisplayValue::InlineBlock
                            | DisplayValue::FlowRoot
                            | DisplayValue::ListItem
                    )
                    && !style.writing_mode.is_vertical_block_flow();
                if direct_text_seed && let Some(nid) = box_node.node_id {
                    parent_font_sizes.entry(nid).or_insert(font_size);
                }

                let mut parent_is_ahem: NodeIdMap<bool> = box_node
                    .text_node_is_ahem
                    .iter()
                    .filter_map(|(&tn, &is_ahem)| {
                        if is_text(tn) {
                            // 文本节点：键改写为其父元素（直接文本路径）。
                            doc.parent_node(tn).map(|pid| (pid, is_ahem))
                        } else {
                            // 已是元素（如 multicol col_ctx 把 inline 元素文本扁平化为
                            // node_id=元素 的片段）：键即元素自身（paint IFC flatten 路径
                            // 按 child_id=元素 查询）。R1446：multicol-basic span 文本经
                            // flatten 收集，is_ahem 须能传到 paint IFC 测宽。
                            Some((tn, is_ahem))
                        }
                    })
                    .collect();
                // R4558：直系文本 is_ahem 兜底（同 fs/lh 种子）——覆盖缺失时测宽按非
                // Ahem 近似（~0.5em/char），Ahem 行断/行数全错（hori-1 关键字盒 4 行
                // 误成 2 行实证）。Ahem 判定与 stored 路径同源（font_family 单值比对）。
                if direct_text_seed && let Some(nid) = box_node.node_id {
                    let is_ahem_box = style
                        .font_family
                        .iter()
                        .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));
                    parent_is_ahem.entry(nid).or_insert(is_ahem_box);
                }

                // R5204：letter-spacing 与下方 word-spacing（R4133）同构——layout IFC
                // element 分支（flatten run）产出片段 node_id = 元素自身，
                // text_node_letter_spacing 按元素 id 键存；build_text_parent_override_map
                // 只保留文本节点键，元素键整条丢弃 → paint Path B 重跑 IFC 查不到覆盖落 0
                // → 声明/继承在 inline 元素的 letter-spacing 全丢（span-direct/inherit
                // probe pitch 17.6 = 纯字宽实证；驱动案 text-decoration-skip-spaces-005
                // ref 侧 span.text ls 全无）。此处理法同 R4133：文本节点 re-key 父元素，
                // 元素键原样保留。
                let mut parent_letter_spacing: NodeIdMap<f32> = box_node
                    .text_node_letter_spacing
                    .iter()
                    .filter_map(|(&tn, &ls)| {
                        if is_text(tn) {
                            doc.parent_node(tn).map(|pid| (pid, ls))
                        } else {
                            Some((tn, ls))
                        }
                    })
                    .collect();
                // R4233：Path B 直系文本兜底——盒无 stored per-fragment 键时（合成盒 /
                // 存储缺失），重跑 IFC 的直系文本 run 查不到覆盖会落 0，而容器自身 ls
                // 即直系文本 run 的 ls（继承语义）。以盒 id 播种兜底；span 内 run 的
                // 父键不受影响（paint advance 已改用片段值，见 render_fragment!）。
                if let Some(nid) = box_node.node_id {
                    parent_letter_spacing.entry(nid).or_insert(letter_spacing);
                }

                // R4133：word-spacing 声明在 inline 元素上时，layout IFC 的 element 分支
                // 产出片段 node_id = 元素自身，text_node_word_spacing 按元素 id 键存。
                // build_text_parent_override_map 只保留文本节点键（确定性过滤），会把元素
                // 键整条丢弃 → paint Path B 重跑 IFC 在该元素上查不到 ws → 声明在 span 的
                // word-spacing 丢失（word-spacing-characters-001 根因）。此处理法同上方
                // is_ahem：文本节点 re-key 父元素，元素键原样保留。
                let parent_word_spacing: NodeIdMap<f32> = box_node
                    .text_node_word_spacing
                    .iter()
                    .filter_map(|(&tn, &ws)| {
                        if is_text(tn) {
                            doc.parent_node(tn).map(|pid| (pid, ws))
                        } else {
                            Some((tn, ws))
                        }
                    })
                    .collect();

                let mut parent_line_heights: NodeIdMap<f32> =
                    build_text_parent_override_map(doc, &box_node.text_node_line_heights);
                // R4558：直系文本 line-height 兜底（与上方 font-size 种子同构）——覆盖
                // 缺失时重跑 IFC 退 19.2 (16×1.2) 默认（R632 记档的 Path B 弱点），行间距
                // 度量错（hori-1 关键字盒 PNG 取证 4 行连块）。解析语义与 layout IFC
                // `container_used_line_height_px` 同源（Number × fs / Length 直解析 /
                // Normal × 1.164 近似常数）。
                if direct_text_seed && let Some(nid) = box_node.node_id {
                    use zero_style_system::property::types::LineHeightValue;
                    let lh_px = match &style.line_height {
                        LineHeightValue::Number(n) => font_size * *n as f32,
                        LineHeightValue::Length(l) => {
                            zero_style_system::computed::resolve_length(l, font_size as f64, None, None) as f32
                        }
                        LineHeightValue::Normal => font_size * 1.164,
                    };
                    if lh_px.is_finite() && lh_px > 0.0 {
                        parent_line_heights.entry(nid).or_insert(lh_px);
                    }
                }

                // R1012：text-transform 覆盖（re-key 文本节点 → 父元素），让 paint Path B
                // 空 styles IFC 也能在 collect_inline_items 期应用 transform，使行断用
                // 转换后文本宽度（与 layout IFC / chromium 一致）。None 不插入（保持默认）。
                // R4382：扁平化 run 的 transform 按元素 id 存储——元素键直通不 re-key。
                let parent_text_transforms: NodeIdMap<TextTransformValue> = box_node
                    .text_node_text_transform
                    .iter()
                    .filter_map(|(&tn, &tt)| {
                        if matches!(tt, TextTransformValue::None) {
                            return None;
                        }
                        if is_text(tn) {
                            doc.parent_node(tn).map(|pid| (pid, tt))
                        } else {
                            Some((tn, tt))
                        }
                    })
                    .collect();

                let inline_metrics = box_node.inline_element_metrics.clone();
                let margin_overrides = box_node.inline_element_margins.clone();
                // R3837：inline 水平 padding 与 margin 同通道恢复（CSS2.1 §8.4）。
                let padding_overrides = box_node.inline_element_paddings.clone().unwrap_or_default();

                let mut ctx = InlineFormattingContext::new(ifc_width)
                    // ZRG-2026-08-15 修复 A：paint IFC 也注入 font resolver——否则
                    // run.font_id=None，行断回退 estimate（布局↔绘制宽度错位）。
                    .with_font_resolver(std::rc::Rc::new(self.font_resolver.clone()))
                    .with_text_align(text_align)
                    // R4213（CSS Text 4）：行组对齐 Path B 同源——layout 侧存 IFC 的组位移
                    // 在非存储路径重跑时不得丢失（text-group-align 转正后 Path B 漏注入
                    // 会回到容器轴对齐）。
                    .with_text_group_align(resolve_text_group_align(Some(style)))
                    .with_text_align_last(text_align_last)
                    .with_bidi_override_direction(zero_layout_engine::bidi_override_direction(style))
                    .with_plaintext_bidi(
                        matches!(style.unicode_bidi, zero_style_system::UnicodeBidiValue::Plaintext),
                        matches!(
                            style.text_align,
                            zero_style_system::property::types::TextAlignValue::Start
                        ),
                    )
                    .with_plaintext_bidi_overrides(box_node.plaintext_bidi_nodes.clone())
                    // R4310：竖排 writing-mode 元素信号——FLAT_CHILD_WALK 竖排子门在
                    // paint Path B（空 styles）下按 layout 期存储复现判定。
                    .with_vertical_walk_nodes(box_node.inline_vertical_nodes.clone())
                    // R4312：块级元素子信号——walk 块子门同通道恢复。
                    .with_block_child_walk_nodes(box_node.inline_block_child_nodes.clone())
                    // display:none 元素信号（CSS2 §9.3 / CSS Display 3 §2.1）——Path B
                    // （空 styles）下 author 规则隐藏子门，防 none 文本泄入父 IFC。
                    .with_display_none_walk_nodes(collect_display_none_walk_nodes(box_node, doc, styles))
                    // R5102（CSS2 §9.7）：out-of-flow 后代集（容器子树 ∪ run-in 元素子树）
                    // ——Path B 空 styles 下扁平化 walk 无法识别 OOF 后代，其文本被吸收
                    // 推进行宽（run-in-contains-inline-006 孙代 abspos 幽灵推进）。vertical
                    // 模式不注入（镜像 layout 期 OOF skip 臂的 vertical 例外）。
                    .with_oof_walk_nodes(collect_oof_walk_nodes(box_node, doc, styles, is_vertical))
                    // R5177（CSS 2.1 §16.1）：color 差异元素集——walk 折回门在 Path B
                    //（空 styles）下的判定信号；layout 期按 run owner 填充（见
                    // store_font_sizes_from_ifc），layout IFC 有 styles 直判同语义（见
                    // collect_flat_inline_children 折回臂）。
                    .with_color_divergent_walk_nodes(box_node.inline_color_divergent_nodes.clone())
                    // R5250（CSS2 §9.3.2）：relative inset 位移表——walk 折回门分裂信号
                    // + fragment 平移消费表，layout 期构建存盒（compute_final），Path B
                    // 注入同一张表保持两臂分段恒等（见 IFC relpos_shifts 字段文档）。
                    .with_relpos_shifts(box_node.inline_relpos_shifts.as_deref().cloned().unwrap_or_default())
                    // R3840：元素级 bidi-override 恢复（layout 期按文本节点 id 存储）。
                    .with_text_node_bidi_overrides(box_node.text_node_bidi_overrides.clone())
                    // R3778：run 级有效 white-space 覆盖——inline 包裹层声明的 pre 等在
                    // 容器级标志近似下丢失（line-clamp-014 类），layout 期按文本节点存储，
                    // 此处恢复使 paint IFC 行断与 layout 一致。
                    .with_ws_overrides(box_node.text_node_ws_overrides.clone())
                    // R4374：回退链垂直度量恢复（layout 期按文本节点/owner 存储）——
                    // paint Path B 行盒基线与 layout 一致。
                    .with_glyph_verticals_overrides(box_node.text_node_glyph_verticals.clone())
                    .with_break_word(break_word)
                    .with_no_wrap(no_wrap)
                    .with_preserve_whitespace(preserve_whitespace)
                    .with_break_at_newline(break_at_newline)
                    .with_word_break(word_break_mode)
                    .with_text_autospace(style.text_autospace)
                    .with_text_indent(text_indent_px)
                    .with_float_exclusions(float_exclusions)
                    .with_tab_size(tab_size.0)
                    .with_tab_size_is_length(tab_size.1)
                    .with_vertical(is_vertical)
                    .with_vertical_rtl(is_vertical_rtl)
                    // R4434：vertical 的 block_extent 恒取 **content_width**（物理块轴
                    // 跨度，R1122 语义）——旧非 caption 分支传 container_width（R1099 后
                    // = content_height = 行内深度），RTL 列自 x=行内深度向左堆叠、越界列
                    // 负 x 裁剪（line-box-direction-vrl-015：cell 内 25 列仅 ~7 列可见，
                    // 字形饿死左带实证）。caption 臂语义并入恒等分支。
                    .with_block_extent(if is_vertical {
                        box_node.content_width
                    } else {
                        container_width
                    })
                    .with_font_size_overrides(parent_font_sizes)
                    .with_is_ahem_overrides(parent_is_ahem)
                    .with_letter_spacing_overrides(parent_letter_spacing)
                    .with_word_spacing_overrides(parent_word_spacing)
                    .with_line_height_overrides(parent_line_heights)
                    .with_ascent_ratio_overrides(box_node.text_node_ascent_ratios.clone())
                    .with_text_transform_overrides(parent_text_transforms)
                    .with_inline_element_metrics(inline_metrics)
                    .with_margin_overrides(margin_overrides)
                    .with_padding_overrides(padding_overrides)
                    .with_inline_block_sizes(collect_atomic_inline_sizes(box_node, styles))
                    // R5235：R1576 递归臂触发集——Path B 空 styles 下 BFS 判定恒 false，
                    // 含原子后代（img）的 inline 元素落 FLAT_CHILD_WALK 后原子被丢弃，
                    // img 后文本少推进一个原子宽（before-after-positioned-001 叠盖根因）。
                    // 读 layout 期存储信号逐位镜像同一分叉（layout-engine 域修复，此处
                    // 仅注入既有 builder 通道）。
                    .with_nested_atomic_walk_nodes(box_node.inline_nested_atomic_nodes.clone());
                ctx = with_shaped_layout(
                    ctx,
                    doc,
                    styles,
                    &text_node_font_ids,
                    &text_node_shaping_font_ids,
                    &box_node.text_node_font_size_adjust,
                    &self.generic_font_ids,
                );
                // R109 §9.2.1.1：匿名块盒片段——若此盒是 inline 被 block 子元素拆分后的
                // 匿名块片段，只收集该片段的 inline 内容（而非 inline 元素的全部子节点）。
                if let Some(ref frag) = box_node.fragment_node_ids {
                    ctx.set_fragment_node_ids(frag.clone());
                }
                // R3991（CSS Display 3 §2.3）：并入本容器首行的 run-in 元素——前置收集
                // 其 inline 内容（与 layout IFC 同源，paint 重跑一致）。
                if let Some(run_in_id) = box_node.run_in_prepended {
                    ctx.set_run_in_prepended(run_in_id);
                    ctx.set_run_in_border(box_node.run_in_border);
                    // R4339：注入布局期分类的块级子集——Path B 空 styles 无法重判
                    // 块级性（页样式可把 div 转 inline），布局盒 children 的
                    // is_block_level 子为真值源（run-in-basic-005：无注入时块子
                    // text_content 扁平拼入前缀流 → "Run-inheaderStart..." 连排）。
                    let block_ids: std::collections::HashSet<zero_dom::NodeId> = box_node
                        .children
                        .iter()
                        .filter(|c| c.is_block_level)
                        .filter_map(|c| c.node_id)
                        .collect();
                    if !block_ids.is_empty() {
                        ctx.set_block_child_nodes(block_ids);
                    }
                }
                // slice10（CSS2 §9.5 float out-of-flow）：Path B 空 styles 无法识别
                // float 子——从真实 computed styles 注入 float 子真值集（box.float
                // 会被 float/margin 后处理清零，样式为权威源），使
                // collect_inline_items 对 float 子发 FloatAnchor（同 R3784 有 styles
                // 臂），不再把 float 子树 inline 内容吸收进容器 IFC（误吸收使容器把
                // float 后代文本重排到自身流位，并经 painted_inline_nodes 去重抑制
                // float 盒自身的正确绘制）。layout 侧 IFC（有 styles）行为不变。
                // 前置条件镜像 layout 期 OOF skip 臂（collect_items CSS2 §9.7 注释）：
                // position abs/fixed 交集子不发 FloatAnchor（§9.7 规定 abs 时 float
                // 计算为 none，其定位/绘制归 abspos pass，Path B 强制断行会与 layout
                // 期静默 skip 错行）；vertical 模式例外同镜像（abs 臂在 vertical 下本
                // 就不跳过，vertical-rl 的 abs 依赖 IFC 内测量）。kill-switch
                // ZW_IFC_SKIP_OOF=0 旧径不镜像（skip_oof 为 pub(super) 不可达，Path B
                // 空 styles 对 abs 的无分类属既有缺口，范围外）。
                let float_ids: std::collections::HashSet<zero_dom::NodeId> = styles
                    .map(|s| {
                        box_node
                            .children
                            .iter()
                            .filter_map(|c| {
                                let id = c.node_id?;
                                s.get(&id)
                                    .is_some_and(|cs| {
                                        !matches!(cs.float, zero_css_parser::values::FloatValue::None)
                                            && (ctx.vertical
                                                || !matches!(
                                                    cs.position,
                                                    zero_css_parser::values::PositionValue::Absolute
                                                        | zero_css_parser::values::PositionValue::Fixed
                                                ))
                                    })
                                    .then_some(id)
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                if !float_ids.is_empty() {
                    ctx.set_float_child_nodes(float_ids);
                }
                // CSS2 §9.7：OOF 直接子集（abs/fixed）——Path B 空 styles 时 OOF skip 臂
                // 不可达，abspos 子被收入行内流推进行宽（run-in-contains-abspos-001）。
                // run-in 前缀子的 OOF 语义同级：前缀收集遍历 run-in 元素的 DOM 子，
                // 其 abspos 子同样推进行宽，一并入集。vertical 模式不注入（镜像
                // layout 期 skip 臂的 vertical 例外：vertical-rl 的 abspos
                // shrink-to-fit 依赖 IFC 内测量）。
                if !ctx.vertical {
                    let oof_ids: std::collections::HashSet<zero_dom::NodeId> = styles
                        .map(|s| {
                            let is_oof = |id: zero_dom::NodeId| {
                                s.get(&id).is_some_and(|cs| {
                                    matches!(
                                        cs.position,
                                        zero_css_parser::values::PositionValue::Absolute
                                            | zero_css_parser::values::PositionValue::Fixed
                                    )
                                })
                            };
                            let mut ids: std::collections::HashSet<zero_dom::NodeId> = box_node
                                .children
                                .iter()
                                .filter_map(|c| c.node_id)
                                .filter(|&id| is_oof(id))
                                .collect();
                            if let Some(run_in_id) = box_node.run_in_prepended {
                                ids.extend(doc.child_nodes(run_in_id).iter().copied().filter(|&id| is_oof(id)));
                            }
                            ids
                        })
                        .unwrap_or_default();
                    if !oof_ids.is_empty() {
                        ctx.set_oof_child_nodes(oof_ids);
                    }
                }
                ctx.layout(doc, node_id, &HashMap::new());
                // R4417：Path B（空 styles 重跑）无行数语义（resolve_line_clamp 对
                // line-clamp:Auto 恒 None，paint 侧 style 亦不可用）——按盒存 cap 补
                // 行级 clamp，先于 fragment 展开（Line 6 片段不再产出、不产 ghost
                // glyph）。cap 由 layout 期写入（compute_final，clamped 时同镜像）。
                if let Some(cap) = box_node.line_clamp_cap {
                    ctx.apply_line_clamp_cap(cap);
                }
                if std::env::var("ZW_PROBE_4328").is_ok() {
                    let n: usize = ctx.lines.iter().map(|l| l.runs.len()).sum();
                    let first = ctx
                        .lines
                        .first()
                        .and_then(|l| l.runs.first())
                        .map(|r| (r.x, r.text.chars().take(20).collect::<String>()));
                    eprintln!(
                        "[p4328] PathB node={:?} rip={:?} runs={n} first={first:?}",
                        box_node.node_id, box_node.run_in_prepended
                    );
                }
                ctx
            };

            // 多行块的片段 y 必须包含行盒 y 偏移（line.y），否则多行文本垂直堆叠。
            // `all_fragments()` 返回行内相对 y（恒 0），`all_fragments_with_line_y()`
            // 把 line.y 加到片段 y。
            //
            // **统一使用 with_line_y（R246 限制解除，2026-06-25）**：R246 曾把此修复限定在
            // preserve_whitespace（pre 族），因 auto-wrap 多行块的 test/ref 此前都堆叠同错，
            // 修后反致同源 reftest 净 -11 回归。但实测确认 auto-wrap 多行堆叠是真实 bug
            //（layout 算对多行 h，paint 把多行画在同一 y）——用户可见的"文字堆叠看不清"。
            // 同源 -11 是「test/ref 同错用例的诚实化暴露」（DC-14 视角为进步），非真退步；
            // product-smoke（真实网站）维度此修复为正收益。故统一对所有 Path B 应用 with_line_y。
            let mut frag_line_tops: Vec<f32> = Vec::new();
            let fragments: Vec<zero_layout_engine::TextFragment> = if use_stored {
                Vec::new()
            } else {
                // R4332：片段 → 所属行盒顶（line.y）平行表——all_fragments_with_line_y
                // 按行序扁平化（行内 run 数展开即得逐片段 line.y）。多行 inline 盒装饰
                //（R1442 横条 + R4330 载荷描边）以行盒顶为锚，旧锚 = 片段基线
                //（frag.y + v_offset，R3856 契约）落字高中部。
                for line in &inline_ctx.lines {
                    frag_line_tops.extend(std::iter::repeat_n(line.y, line.runs.len()));
                }
                let mut fragments = inline_ctx.all_fragments_with_line_y();
                // R5250（CSS2 §9.3.2）：relative inset 视觉位移——纯 inline 非替换盒无
                // taffy 节点无人施加，Path B（空 styles 重跑，无 styles 可判）从盒读
                // compute_final 存的位移映射（inline_relpos_shifts）平移 fragment；装饰锚
                // 行盒顶平行表索引对齐同移（R4332 契约）。表空（无 relative inline /
                // 竖排 / kill-switch）时循环体零命中，无行为变化。
                if let Some(shifts) = box_node.inline_relpos_shifts.as_deref() {
                    for (i, frag) in fragments.iter_mut().enumerate() {
                        if let Some(&(dx, dy)) = shifts.get(&frag.node_id) {
                            frag.x += dx;
                            frag.y += dy;
                            if let Some(t) = frag_line_tops.get_mut(i) {
                                *t += dy;
                            }
                        }
                    }
                }
                fragments
            };

            let has_content = use_stored && !stored_fragments.is_empty() || !fragments.is_empty();

            let needs_ellipsis = matches!(style.text_overflow, TextOverflowValue::Ellipsis)
                && !matches!(style.overflow_x, zero_css_parser::values::OverflowValue::Visible);

            // R2237：ellipsis '.' 测宽须用容器字体判定 is_ahem（Ahem 字体 '.' = 1em 方块，
            // 非真实点宽）。旧 measure_char_for_paint('.', fs, false) 硬编码 false → Ahem 容器
            // ellipsis 宽度过小 → 定位错（text-overflow-ellipsis-001）。driving: 同 container_is_ahem
            // 模式（text.rs:853）。
            let container_is_ahem = style
                .font_family
                .iter()
                .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));

            if has_content {
                let glyphs_before_fragments = self.primitives.glyphs.len();

                // R4440 临时探针（ZW_DEBUG_GLYPHS=1）：vertical 盒 paint 装配参数。
                if is_vertical && glyph_probe_enabled() {
                    eprintln!(
                        "[vpaint] box={:?} abs=({:.0},{:.0}) content=({:.0},{:.0}) cx={:.0} cy={:.0} use_stored={} tx={:.1} ty={:.1} w={:.0} slr={}",
                        box_node.node_id,
                        abs_x,
                        abs_y,
                        box_node.content_width,
                        box_node.content_height,
                        content_x,
                        content_y,
                        use_stored,
                        tx,
                        ty,
                        box_node.width,
                        box_node.writing_mode_sideways_lr,
                    );
                    for (i, f) in fragments.iter().take(2).enumerate() {
                        eprintln!(
                            "[vpaint]   frag[{}] x={:.1} y={:.1} line_y={:.1} w={:.1} text={:?}",
                            i,
                            f.x,
                            f.y,
                            frag_line_tops.get(i).copied().unwrap_or(f32::NAN),
                            f.width,
                            f.text.chars().take(4).collect::<String>()
                        );
                    }
                }

                // writing-mode: vertical-rl/vertical-lr 时字符旋转 90°
                // R4443：sideways-lr（旁路标记）字形 −90° CCW + 列内自下而上推进
                //（spec css-writing-modes-3 §sideways-lr；R1785 规范化仅保块流方向）。
                let sideways_lr = box_node.writing_mode_sideways_lr;
                let rotation = if is_vertical {
                    if sideways_lr {
                        -std::f32::consts::FRAC_PI_2
                    } else {
                        std::f32::consts::FRAC_PI_2
                    }
                } else {
                    0.0
                };

                if let Some(ref mc) = multicol_info {
                    // 多列布局：遍历行（带 line.y），将行分配到各列
                    let total_height: f32 = inline_ctx.lines.iter().map(|l| l.height).sum();
                    let num_lines = inline_ctx.lines.len();
                    // R1424：target_h 按 ceil(行数/列数) × 平均行高（front-loaded，匹配 chromium）。
                    let target_h = multicol_balance_target_height(num_lines, mc.col_count, total_height);

                    // 预计算每列首行 y，用于把每列内容 rebase 到列内 y=0。
                    // 旧实现 col_start_y = col_idx * target_h，当 target_h 不是行高整数倍时
                    // （如 29 行 / 2 列 → target_h=14.5 行）首行不在 y=0，列内内容整体偏移。
                    // 取每列实际首行 y 作 col_start_y 可消除该 fractional offset。
                    let col_first_y: Vec<f32> = (0..mc.col_count)
                        .map(|col_idx| {
                            if target_h <= 0.0 {
                                0.0
                            } else {
                                inline_ctx
                                    .lines
                                    .iter()
                                    .find(|l| ((l.y / target_h).floor() as usize).min(mc.col_count - 1) == col_idx)
                                    .map(|l| l.y)
                                    .unwrap_or(0.0)
                            }
                        })
                        .collect();

                    // 按列分组渲染：先收集每列的行索引，再按列渲染并裁剪
                    // 这样可以对每列独立裁剪，防止内容溢出到相邻列
                    for (col_idx, &col_start_y) in col_first_y.iter().enumerate() {
                        let col_x_offset = col_idx as f32 * (mc.col_width + mc.gap);

                        // 裁剪区域：列宽 + 右半间隙，允许内容延伸到间隙
                        let clip_rect = Rect::new(
                            content_x + col_x_offset,
                            content_y,
                            mc.col_width + mc.gap / 2.0,
                            box_node.content_height.max(0.0) + 1000.0,
                        );
                        let counts_before_col = PrimitiveCounts::snapshot(&self.primitives);

                        for line in &inline_ctx.lines {
                            // 根据行的 y 位置确定所在列
                            let line_col = if target_h > 0.0 {
                                (line.y / target_h).floor() as usize
                            } else {
                                0
                            }
                            .min(mc.col_count - 1);

                            if line_col != col_idx {
                                continue;
                            }

                            for fragment in &line.runs {
                                self.painted_inline_nodes.insert(fragment.node_id);

                                // 颜色：取片段所属 inline 元素的 color，绕过 inline ownership
                                // （多列分支统一绘制全部片段）。fragment.node_id 可能是 inline 元素
                                // 也可能是文本节点——文本节点时取其父元素。同时标记 owner 元素，
                                // 使 span 自身的 paint_text 跳过（避免在非列位置重绘）。
                                let owner_id = if doc
                                    .get(fragment.node_id)
                                    .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                                {
                                    doc.parent_node(fragment.node_id).unwrap_or(fragment.node_id)
                                } else {
                                    fragment.node_id
                                };
                                self.painted_inline_nodes.insert(owner_id);
                                // R3895：扁平化片段补标 inline 包装链（同 render_fragment! 处）。
                                if let Some(styles) = styles {
                                    mark_inline_wrapper_chain_painted(
                                        &mut self.painted_inline_nodes,
                                        doc,
                                        styles,
                                        owner_id,
                                    );
                                }
                                // generated-content 注入文本节点持有伪元素自身样式（同
                                // render_fragment! 宏处；CSS Pseudo 4 §2）——文本节点片段
                                // 优先取自身 styles 条目，普通文本节点落回父元素。
                                let owner_style = if doc
                                    .get(fragment.node_id)
                                    .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                                {
                                    styles
                                        .and_then(|s| s.get(&fragment.node_id))
                                        .or_else(|| styles.and_then(|s| s.get(&owner_id)))
                                } else {
                                    styles.and_then(|s| s.get(&owner_id))
                                };
                                let owner_variations = crate::text_metrics::paint_font_variations(
                                    &owner_style.unwrap_or(style).font_variation_settings,
                                );
                                let owner_font_variation_id = self.primitives.intern_font_variations(&owner_variations);
                                // R4525/R4549：clip:text 彩字上下文——字形色 = text 色 OVER
                                // bg 恒色（恒色 bg 的 mask 管线逐像素等价预混合；text 色透明
                                // 时 = bg 恒色，与 R4525 v1 canonical 行为恒等）。C = 栈顶
                                //（paint_node push）或 owner 自身/DOM 祖先链 clip:text 恒色
                                //（栈未及 push 的 inline 扁平化形态 + R4552 deferred SC
                                // 延迟绘制后代，按静态祖先链判定）。
                                let frag_color = match self.bg_clip_text_color.last().copied().or_else(|| {
                                    owner_style
                                        .and_then(|s| {
                                            bg_clip_text_solid_color(
                                                s,
                                                &self.image_solid_colors,
                                                self.document_url.as_deref(),
                                            )
                                        })
                                        .or_else(|| {
                                            if let Some(styles) = styles {
                                                bg_clip_text_solid_color_ancestral(
                                                    doc,
                                                    owner_id,
                                                    styles,
                                                    &self.image_solid_colors,
                                                    self.document_url.as_deref(),
                                                )
                                            } else {
                                                None
                                            }
                                        })
                                }) {
                                    Some(clip_bg) => {
                                        let text_color = owner_style
                                            .filter(|s| s.color != ColorValue::CurrentColor)
                                            .map(|s| color_value_to_render(&s.color))
                                            .unwrap_or(color);
                                        color_over(text_color, clip_bg)
                                    }
                                    None => owner_style
                                        .filter(|s| s.color != ColorValue::CurrentColor)
                                        .map(|s| color_value_to_render(&s.color))
                                        .unwrap_or(color),
                                };
                                // R2523：text-emphasis-color（CSS Text Decoration 3 §3.3）。
                                // 显式色覆盖 currentColor；默认 CurrentColor → 沿用 frag_color
                                //（标记随文字色，字节不变）。
                                let emphasis_color = owner_style
                                    .filter(|s| s.text_emphasis_color != ColorValue::CurrentColor)
                                    .map(|s| color_value_to_render(&s.text_emphasis_color))
                                    .unwrap_or(frag_color);
                                // R1021：text-emphasis 标记取自片段 owner 样式（<span> 上设的属性），
                                // 非容器 style。None/Char 判定 + 位置均来自 owner。
                                let emphasis_mark: Option<char> =
                                    owner_style.and_then(|s| match s.text_emphasis_style {
                                        TextEmphasisStyleValue::Char(c) => Some(c),
                                        TextEmphasisStyleValue::None => None,
                                    });
                                let emphasis_over = owner_style
                                    .map(|s| {
                                        matches!(
                                            s.text_emphasis_position,
                                            TextEmphasisPositionValue::OverRight | TextEmphasisPositionValue::OverLeft
                                        )
                                    })
                                    .unwrap_or(true);
                                // R1689：ruby per-segment annotation —— owner 为 <ruby> 时，
                                // 每个 rt 配对其前 base 段，annotation 居中于对应 base segment。
                                // R4409：第三元 = 配对 rt 块轴间距（css-ruby #interlinear-block；零间距页逐字节不变）。
                                let ruby_segs: Option<RubyAnnotationSegs> =
                                    ruby_annotation_segments(doc, owner_id, styles);

                                let frag_base_x = content_x + fragment.x + col_x_offset + tx;
                                // 行盒顶部 = (line.y - col_start_y)；基线偏移 v_offset。
                                // R3856 基线契约：Ahem → line.baseline_y（行基线绝对 y，方块由
                                // raster 放到 [baseline−0.8em, baseline+0.2em]）；普通字体 = font_size
                                // ≈ ascent（既有启发式，非 Ahem 不受 raster 契约影响）。
                                // is_ahem 用容器 font-family 判定（多列 IFC 的 fragment.is_ahem 不可靠）。
                                let container_is_ahem = style
                                    .font_family
                                    .iter()
                                    .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));
                                let v_offset = if container_is_ahem {
                                    line.baseline_y
                                } else {
                                    fragment.font_size
                                };
                                let frag_base_y = content_y + (line.y - col_start_y) + v_offset + ty;

                                let transformed = apply_text_transform(&fragment.text, &style.text_transform);
                                let mut char_pos = frag_base_x;
                                let frag_is_ahem = fragment.is_ahem;
                                // R4133：per-fragment word-spacing——片段 owner（inline 元素）
                                // 声明的 ws 在容器盒 paint 时 style=容器（ws=0）会丢；nbsp 等
                                // word-separator 在词内不产词界（无 lead_gap），逐字符 advance
                                // 须用 owner 的 ws（word-spacing-characters-001）。
                                let frag_word_spacing: f32 = match style.word_spacing {
                                    LengthValue::Px(s) => s as f32,
                                    LengthValue::Percentage(p) => fragment.font_size * (p as f32 / 100.0),
                                    ref lv => zero_style_system::computed::resolve_length(
                                        lv,
                                        fragment.font_size as f64,
                                        None,
                                        None,
                                    ) as f32,
                                };
                                // R4233：per-fragment letter-spacing（同上 word-spacing 理由，
                                // 镜像 R4133）——片段 run 声明的 ls ≠ 容器盒值时以片段为准。
                                let frag_letter_spacing = fragment.letter_spacing;
                                let frag_word_spacing = owner_style
                                    .map(|s| match s.word_spacing {
                                        LengthValue::Px(v) => v as f32,
                                        LengthValue::Percentage(p) => fragment.font_size * (p as f32 / 100.0),
                                        ref lv => zero_style_system::computed::resolve_length(
                                            lv,
                                            fragment.font_size as f64,
                                            None,
                                            None,
                                        ) as f32,
                                    })
                                    .unwrap_or(frag_word_spacing);

                                for ch in transformed.chars() {
                                    let glyph_x = char_pos;
                                    let glyph_y = frag_base_y;

                                    for &(shadow_ox, shadow_oy, shadow_color) in &active_text_shadows {
                                        self.primitives.add_glyph(GlyphPrimitive {
                                            x: glyph_x + shadow_ox,
                                            y: glyph_y + shadow_oy,
                                            font_size: fragment.font_size,
                                            color: shadow_color,
                                            glyph_id: ch as u32,
                                            font_glyph_index: None,
                                            source: None,
                                            font_id: default_font_id,
                                            font_variation_id: owner_font_variation_id,
                                            bitmap_width: None,
                                            bitmap_height: None,
                                            rotation,
                                            synthetic_italic: false,
                                        });
                                    }

                                    self.primitives.add_glyph(GlyphPrimitive {
                                        x: glyph_x,
                                        y: glyph_y,
                                        font_size: fragment.font_size,
                                        color: frag_color,
                                        glyph_id: ch as u32,
                                        font_glyph_index: None,
                                        source: None,
                                        font_id: default_font_id,
                                        font_variation_id: owner_font_variation_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                                        synthetic_italic: false,
                                    });

                                    let advance = self.measure_char_cached(
                                        default_font_id.0,
                                        ch,
                                        fragment.font_size,
                                        frag_is_ahem,
                                    ) + frag_letter_spacing
                                        + if zero_style_system::is_word_separator(ch) {
                                            frag_word_spacing
                                        } else {
                                            0.0
                                        };
                                    char_pos += advance;

                                    // R1021：text-emphasis 标记（CSS Text Decoration 3 §3）。
                                    // 每个非空白字符上方（over）或下方（under）居中绘一个小标记字符。
                                    if !ch.is_whitespace()
                                        && let Some(mark_ch) = emphasis_mark
                                    {
                                        let mark_fs = fragment.font_size * 0.5;
                                        let mark_advance =
                                            self.measure_char_cached(default_font_id.0, mark_ch, mark_fs, frag_is_ahem);
                                        // 居中于当前字符（char_pos 已前进 advance，故字符中心 = char_pos - advance/2）
                                        let mark_x = char_pos - advance / 2.0 - mark_advance / 2.0;
                                        let mark_y = if emphasis_over {
                                            frag_base_y - fragment.font_size
                                        } else {
                                            frag_base_y + fragment.font_size * 0.35
                                        };
                                        self.primitives.add_glyph(GlyphPrimitive {
                                            x: mark_x,
                                            y: mark_y,
                                            font_size: mark_fs,
                                            color: emphasis_color,
                                            glyph_id: mark_ch as u32,
                                            font_glyph_index: None,
                                            source: None,
                                            font_id: default_font_id,
                                            font_variation_id: owner_font_variation_id,
                                            bitmap_width: None,
                                            bitmap_height: None,
                                            rotation,
                                            synthetic_italic: false,
                                        });
                                    }

                                    // R1022 per-char ruby annotation 已移至 text_width 后的
                                    // segment-centered 块（R1688）—— 整 rt 注音居中于 base segment。
                                }

                                let text_width: f32 = transformed
                                    .chars()
                                    .map(|ch| {
                                        let w = self.measure_char_cached(
                                            default_font_id.0,
                                            ch,
                                            fragment.font_size,
                                            frag_is_ahem,
                                        ) + frag_letter_spacing;
                                        if zero_style_system::is_word_separator(ch) {
                                            w + frag_word_spacing
                                        } else {
                                            w
                                        }
                                    })
                                    .sum();
                                // R1689：ruby per-segment annotation —— 每个 rt 居中于其前 base 段
                                //（替代 R1688 整 base 扁平化居中，解 per-kanji Japanese ruby）。
                                // seg_x_start 按各 base 段字符宽累积，annotation 居中于 [start, start+seg_w]。
                                if let Some(RubyAnnotationSegs { segs, span_all }) = ruby_segs.as_ref()
                                    && !segs.is_empty()
                                {
                                    let rt_fs = fragment.font_size * 0.5;
                                    let rt_y0 = frag_base_y - fragment.font_size;
                                    // R4363 临时诊断：ruby overlay 落位追踪（ZW_DEBUG_IFC=1）。
                                    if std::env::var("ZW_DEBUG_IFC").as_deref() == Ok("1") {
                                        eprintln!(
                                            "[ruby-overlay] frag_base_y={} frag.y={} frag.height={} rt_y={} fs={}",
                                            frag_base_y, fragment.y, fragment.height, rt_y0, fragment.font_size
                                        );
                                    }
                                    let mut seg_x = frag_base_x;
                                    for (base, annot, rt_spacing) in segs {
                                        // R4409：per-seg 块轴间距——rt 盒 spacing 把注音推离 base。
                                        let rt_y = rt_y0 - rt_spacing;
                                        let seg_w: f32 = base
                                            .chars()
                                            .map(|c| {
                                                self.measure_char_cached(
                                                    default_font_id.0,
                                                    c,
                                                    fragment.font_size,
                                                    frag_is_ahem,
                                                )
                                            })
                                            .sum::<f32>()
                                            + frag_letter_spacing * base.chars().count() as f32;
                                        if !annot.is_empty() {
                                            let annot_w: f32 = annot
                                                .chars()
                                                .map(|c| {
                                                    self.measure_char_cached(default_font_id.0, c, rt_fs, frag_is_ahem)
                                                })
                                                .sum();
                                            let mut ax = seg_x + (seg_w - annot_w) / 2.0;
                                            for rc in annot.chars() {
                                                self.primitives.add_glyph(GlyphPrimitive {
                                                    x: ax,
                                                    y: rt_y,
                                                    font_size: rt_fs,
                                                    color: frag_color,
                                                    glyph_id: rc as u32,
                                                    font_glyph_index: None,
                                                    source: None,
                                                    font_id: default_font_id,
                                                    font_variation_id: owner_font_variation_id,
                                                    bitmap_width: None,
                                                    bitmap_height: None,
                                                    rotation,
                                                    synthetic_italic: false,
                                                });
                                                ax += self.measure_char_cached(
                                                    default_font_id.0,
                                                    rc,
                                                    rt_fs,
                                                    frag_is_ahem,
                                                );
                                            }
                                        }
                                        seg_x += seg_w;
                                    }
                                    // R4422（css-ruby-1 #nested-pairing）：span-all 注音行——行 0 上方一个 rt 行高
                                    //（rt_fs = 0.5em），跨全部 base 居中（nested/rtc 双形态同一落位）。
                                    if let Some(span_text) = &span_all {
                                        let span_y = rt_y0 - 2.0 * rt_fs;
                                        let span_w: f32 = span_text
                                            .chars()
                                            .map(|c| {
                                                self.measure_char_cached(default_font_id.0, c, rt_fs, frag_is_ahem)
                                            })
                                            .sum();
                                        let mut sx = frag_base_x + (seg_x - frag_base_x - span_w) / 2.0;
                                        for sc in span_text.chars() {
                                            self.primitives.add_glyph(GlyphPrimitive {
                                                x: sx,
                                                y: span_y,
                                                font_size: rt_fs,
                                                color: frag_color,
                                                glyph_id: sc as u32,
                                                font_glyph_index: None,
                                                source: None,
                                                font_id: default_font_id,
                                                font_variation_id: owner_font_variation_id,
                                                bitmap_width: None,
                                                bitmap_height: None,
                                                rotation,
                                                synthetic_italic: false,
                                            });
                                            sx += self.measure_char_cached(default_font_id.0, sc, rt_fs, frag_is_ahem);
                                        }
                                    }
                                }
                                let (skip_s, skip_e) = self.decoration_skip_trims(
                                    style,
                                    &transformed,
                                    default_font_id.0,
                                    fragment.font_size,
                                    frag_is_ahem,
                                    frag_letter_spacing,
                                );
                                self.paint_text_decoration_from_style(
                                    frag_base_x,
                                    frag_base_y,
                                    fragment.font_size,
                                    text_width,
                                    frag_color,
                                    style,
                                    skip_s,
                                    skip_e,
                                );
                            }
                        }

                        // 对本列的图元应用裁剪
                        super::super::helpers::clip_all_primitives_to_rect(
                            &mut self.primitives,
                            &counts_before_col,
                            &clip_rect,
                        );
                    }
                } else {
                    // 非多列布局：统一处理存储片段和 IFC 片段
                    // 宏化渲染逻辑，避免重复代码
                    // ::first-letter（CSS2 §5.12.2 / css-pseudo-4）：块容器首个格式化行的首字母
                    // 单元按伪元素样式绘制（穿透嵌套 inline——首字母所在片段属于子 span，但伪元素
                    // 样式挂在块容器上）。首字母单元 = 前导 P* + 首字母 + 后随 P*（除 Ps/Pd），
                    // 词分隔符在 trailing 方向终止吸收（first_letter_cluster_len）。
                    // paint 侧逐字形覆色（切片范围 = color；font/metrics 类属性需 IFC 分词切片
                    // 改几何，FIXME 挂账）。多列分支暂不应用（FIXME）。
                    let first_letter_color: Option<Color> = style
                        .first_letter_pseudo
                        .as_deref()
                        .map(|s| color_value_to_render(&s.color));
                    // Some((期望字符序列, 已消费数))——仅首个含非空白字符的片段携带；
                    // 字形与期望字符不匹配（shaping 变体/分裂）即放弃，防止错染。
                    let mut first_letter_expected: Option<(Vec<char>, usize)> = None;
                    // R5299：单行整盒 band 域绘制去重（per owner+行+段一次）——band 域
                    // bg/横条覆盖段内全部片段，宏内「bg→字形」逐片段交错，逐片段重绘
                    // 会以后者 fill 盖住前者字形（chromium 为全 bg 层后全字形层；
                    // selection-012 flag-on 实证第二词 bg 盖掉首词字形）。
                    let mut r5299_band_drawn: std::collections::HashSet<(NodeId, u32, u32)> =
                        std::collections::HashSet::new();
                    let mut first_letter_applied = false;
                    let first_letter_bg_color: Option<Color> = style
                        .first_letter_pseudo
                        .as_deref()
                        .filter(|s| s.background_color != ColorValue::Transparent)
                        .map(|s| color_value_to_render(&s.background_color));
                    macro_rules! render_fragment {
                        ($frag_x:expr, $frag_y:expr, $frag_line_top:expr, $frag_edge:expr, $frag_width:expr, $baseline_offset:expr, $frag_fs:expr, $frag_text:expr, $frag_nid:expr, $is_ahem:expr, $frag_source:expr, $frag_ls:expr, $frag_sel:expr, $frag_band_map:expr, $frag_seg_map:expr) => {{
                            // R4233：per-fragment letter-spacing（镜像 R4133 per-fragment
                            // word-spacing）——片段所属 run 声明的 ls 与容器盒不同时
                            //（span{letter-spacing} / 容器内 normal 重置），glyph advance、
                            // text_width、ruby 段宽须用片段值；容器值仅作 fallback 语义保留
                            //（shaped-eligibility gate 仍读容器值，见下）。
                            let frag_letter_spacing: f32 = $frag_ls;
                            // R4332：片段在其 owner 片段序列中的首/末位（调用点按 owner
                            // 预计算——macro_rules! 卫生宏体不可见调用点局部，故作参数传入）。
                            let (r4332_is_first, r4332_is_last, r4332_is_multi_line) = $frag_edge;
                            // R4332：owner 解析（竖边判定与 per-fragment color 同源）。
                            let r4332_owner = if doc
                                .get($frag_nid)
                                .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                            {
                                doc.parent_node($frag_nid).unwrap_or($frag_nid)
                            } else {
                                $frag_nid
                            };
                            // R5297：owner 行连续 x 域 (min_x, max_end)（调用点建
                            // (owner, line_top) → 域 表，含祖先累计，见
                            // owner_line_band_extent_map）；无表项退自身 run 域。
                            // 表值/回退值均容器相对 x——与 frag_base_x 同基座
                            // （content_x + tx）换算到绘制域：缺 content_x 时整带
                            // 左移出内容盒（c42 带左移 body margin 8px 被 pair 对消
                            // 掩盖，multicol-basic-001..004 边框带实证 2.33%）。
                            // R5299：单行整盒（旗标开）改用裂解段表——按片段 x 区间
                            // 包含查找所属段；miss 回落行连续域（R5297 口径不变）。
                            // 段命中时首/末边归属改用段标记（逻辑首/末片段所在段），
                            // bidi 重排下视觉 x 序 ≠ 逻辑序。
                            let r5299_seg = if r4332_is_multi_line {
                                None
                            } else {
                                $frag_seg_map
                                    .get(&(r4332_owner, $frag_line_top.to_bits()))
                                    .and_then(|segs| {
                                        segs.iter().copied().find(|&(s, e, _, _)| {
                                            $frag_x >= s - 0.5
                                                && $frag_x + $frag_width <= e + 0.5
                                        })
                                    })
                            };
                            let r5299_seg_hit = r5299_seg.is_some();
                            let (r5297_band_min_x, r5297_band_max_end) = {
                                let (r5297_mn, r5297_mx) = match r5299_seg {
                                    Some((s, e, _, _)) => (s, e),
                                    None => $frag_band_map
                                        .get(&(r4332_owner, $frag_line_top.to_bits()))
                                        .copied()
                                        .unwrap_or(($frag_x, $frag_x + $frag_width)),
                                };
                                (content_x + r5297_mn + tx, content_x + r5297_mx + tx)
                            };
                            // R5299：band 域边归属——段命中用段标记，miss 沿用
                            // owner 逻辑首/末（owner_fragment_edge_flags 口径）。
                            let (r5299_band_first, r5299_band_last) = match r5299_seg {
                                Some((_, _, sf, sl)) => (sf, sl),
                                None => (r4332_is_first, r4332_is_last),
                            };
                            let r5299_seg_x0 = match r5299_seg {
                                Some((s, _, _, _)) => content_x + s + tx,
                                None => 0.0,
                            };
                            // CSS 2.1 §9.2.1.1: an inline-block is an atomic inline-level box.
                            // Its parent IFC emits an empty placeholder solely for positioning; the
                            // inline-block's own box must still paint its text.
                            if !$frag_text.is_empty() {
                                self.painted_inline_nodes.insert($frag_nid);
                                // R3895：扁平化片段补标 inline 包装链（内层 span/q 等不再自绘）。
                                if let Some(styles) = styles {
                                    mark_inline_wrapper_chain_painted(
                                        &mut self.painted_inline_nodes,
                                        doc,
                                        styles,
                                        $frag_nid,
                                    );
                                }
                                // R5177：color 差异拆分归因的外层 inline 载体补标在渲染
                                // 循环前统一处理（mark_split_owner_ancestor_chain_painted，
                                // 仅拆分 owner 上溯——普通 run 归因 id 即外层自身，热路径
                                // 逐片段上溯是无谓开销）。
                            }

                            // R358：per-fragment color（带 abs-pos guard）。
                            // 非多列路径此前所有片段用容器 color（丢失 span 自身 color，
                            // 如 multicol-count-computed-004 彩色 span 被渲成容器黑色）。
                            // 现解析每个片段所属元素的 color，**但 abs-pos/fixed 片段保留容器 color**——
                            // R335 实证 per-fragment color 作用于 abspos 文本会使绿色 X 更显眼地绘在
                            // 错误的 paint-IFC（正常流）位置 → abs-pos-non-replaced-vrl/vlr 4 case 回归。
                            // abspos 文本位置修复需 Phase A（R336 double-path），guard 维持当前行为。
                            // R4332：owner 解析上移至宏首（竖边首/末判定同源），此处直接复用。
                            let owner_id = r4332_owner;
                            // R3871：荷兰语 ij/IJ 双字母组语境——沿 owner 祖先链找最近 lang 属性，
                            // nl 前缀（nl / nl-NL / nl-SR…）即启用（与 style-system effective_lang
                            // 同语义）。doc 在此已为 &Document（paint_text 的 if-let 解包）。
                            // generated-content 注入文本节点（inject_pseudo_text_nodes）在 styles
                            // 中持有**伪元素自身样式**（CSS Pseudo 4 §2：::before 内容按伪元素
                            // 样式着色）——文本节点片段优先取自身 styles 条目（普通 DOM 文本节点
                            // 无条目 → 落回父元素，行为不变）。content-067/085/131：`meta:before
                            // { content: attr(X); color: green }` 注入文本按 meta 黑色渲染 → 应绿。
                            let owner_style_opt = if doc
                                .get($frag_nid)
                                .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                            {
                                styles
                                    .and_then(|s| s.get(&$frag_nid))
                                    .or_else(|| styles.and_then(|s| s.get(&owner_id)))
                            } else {
                                styles.and_then(|s| s.get(&owner_id))
                            };
                            // R5098（CSS2.1 visibility）：flat/并入 IFC 的片段 owner 盒不独立
                            // 绘制（run-in 并入 R3992 叶化、R4332 前缀收集），盒级 hidden skip
                            // 不覆盖——hidden 片段照绘（run-in-contains-abspos-001/003 族
                            // `.run-in > span[+span] { visibility: hidden }` 双 header 实证）。
                            // 片段发射前按 owner 自身样式过滤；可见后代重置继承（visibility
                            // per-element）由 owner 直判保真。kill-switch
                            // `ZW_INLINE_FRAG_VISIBILITY=0`。盒级路径（paint_node hidden skip）
                            // 不变。
                            if self.inline_frag_visibility_enabled
                                && owner_style_opt.is_some_and(|s| {
                                    matches!(s.visibility, VisibilityValue::Hidden | VisibilityValue::Collapse)
                                })
                            {
                                continue;
                            }
                            // 键盘焦点环（UA 默认）：a/span 等行内元素无独立盒、不经
                            // paint_node 盒路径的焦点环块——在此按片段行盒逐行绘制。
                            // 作者已声明 outline 的 owner 不叠加（focus_ring_applicable
                            // 与盒路径同判定）；focused_from_keyboard=false 时零开销短路。
                            if self.focused_from_keyboard
                                && !$frag_text.is_empty()
                                && self.focused_node == Some(owner_id)
                                && owner_style_opt.is_none_or(|s| {
                                    // 仅行内 display 走片段环：inline-block 等原子内联
                                    // 有独立盒，由 paint_node 盒路径焦点环覆盖，避免双环。
                                    self.focus_ring_applicable(s)
                                        && matches!(s.display, DisplayValue::Inline)
                                })
                            {
                                // 片段行盒入队（paint() 末尾按行合并成整环）：
                                // top = 行顶，bottom = 基线 + 0.25em descent。
                                let ring_top = content_y + $frag_line_top;
                                let ring_bottom = content_y + $frag_y + $baseline_offset + $frag_fs * 0.25;
                                self.focus_ring_bands.push((
                                    content_x + $frag_x,
                                    ring_top,
                                    $frag_width,
                                    (ring_bottom - ring_top).max($frag_fs),
                                ));
                            }
                            let dutch_digraph = {
                                let mut found = false;
                                let mut cur = Some(owner_id);
                                while let Some(id) = cur {
                                    if let Some(lang) = doc.get_attribute(id, "lang") {
                                        let l = lang.to_ascii_lowercase();
                                        found = l == "nl" || l.starts_with("nl-");
                                        break;
                                    }
                                    cur = doc.parent_node(id);
                                }
                                found
                            };
                            let frag_color = owner_style_opt
                                .filter(|s| {
                                    s.color != ColorValue::CurrentColor
                                        && !matches!(
                                            s.position,
                                            zero_css_parser::values::PositionValue::Absolute
                                                | zero_css_parser::values::PositionValue::Fixed
                                        )
                                })
                                .map(|s| color_value_to_render(&s.color))
                                .unwrap_or(color);
                            // R4525/R4549：clip:text 彩字上下文（同 stored 路径；owner 自身
                            // /DOM 祖先链 clip:text 恒色时按静态链判定，覆盖 R4552 deferred
                            // SC 延迟绘制后代）——字形色 = text 色 OVER bg 恒色预混合。
                            let frag_color = match self
                                .bg_clip_text_color
                                .last()
                                .copied()
                                .or_else(|| {
                                    owner_style_opt
                                        .and_then(|s| {
                                            bg_clip_text_solid_color(s, &self.image_solid_colors, self.document_url.as_deref())
                                        })
                                        .or_else(|| {
                                            if let Some(styles) = styles {
                                                bg_clip_text_solid_color_ancestral(
                                                    doc,
                                                    $frag_nid,
                                                    styles,
                                                    &self.image_solid_colors,
                                                    self.document_url.as_deref(),
                                                )
                                            } else {
                                                None
                                            }
                                        })
                                })
                            {
                                Some(clip_bg) => color_over(frag_color, clip_bg),
                                None => frag_color,
                            };
                            // R5053：整行选中 → ::selection color 覆写（owner+行盒顶查表，
                            // 调用点按活跃片段序列预算；None = 未选中行保持原色）。置于
                            // clip:text 预混之后——选中行伪色直接生效（clip:text × selection
                            // 同时态罕见，取 selection 优先语义）。
                            let frag_color = $frag_sel.unwrap_or(frag_color);

                            // R1021：text-emphasis 取自片段 owner 样式（<span> 上设）。
                            let shaping_style = owner_style_opt.unwrap_or(style);
                            let variations =
                                crate::text_metrics::paint_font_variations(&shaping_style.font_variation_settings);
                            let font_variation_id = self.primitives.intern_font_variations(&variations);
                            // R2523：text-emphasis-color（CSS Text Decoration 3 §3.3）。
                            // 显式色覆盖 currentColor；默认 CurrentColor → 沿用 frag_color
                            //（标记随文字色，字节不变）。
                            let emphasis_color = owner_style_opt
                                .filter(|s| s.text_emphasis_color != ColorValue::CurrentColor)
                                .map(|s| color_value_to_render(&s.text_emphasis_color))
                                .unwrap_or(frag_color);
                            // R1224/R1464：Path A 按 owner style 选 face；Path B 用 layout
                            // 保存的 per-run face，缺失时回退容器 face。
                            let is_ahem_frag = owner_style_opt.is_some_and(|s| {
                                s.font_family.iter().any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"))
                            });
                            // R3991（CSS Display 3 §2.3 run-in）：run-in 并入后其文本片段
                            // 的 owner（run-in 元素）font-weight 与容器不同（.run-in{font-weight:
                            // bold}），text_node_font_ids 是按**容器** style 解析的 face（resolve_
                            // text_node_fonts 无 per-node weight）→ run-in 文本丢 bold。owner 有
                            // 独立 weight/family 声明时按 owner 重解析 face（普通 inline span 的
                            // bold face 本就走此差异通道；run-in 的 owner 是块级元素，此前无
                            // per-fragment 修正路径）。仅 weight/ stretch 语义，family 沿 stored。
                            let frag_font_id = if is_ahem_frag {
                                ahem_font_id
                            } else if let Some(owner) = owner_style_opt
                                && (owner.font_weight != style.font_weight
                                    || owner.font_stretch != style.font_stretch)
                            {
                                let (fid, _) = self.resolve_font_id(&owner.font_family, &owner.font_weight, &owner.font_style, owner.font_stretch);
                                fid
                            } else {
                                text_node_font_ids
                                    .get(&$frag_nid)
                                    .copied()
                                    .unwrap_or(default_font_id)
                            };
                            // R2497：owner 请求 italic 且 resolved face 非 italic 时合成；
                            // Ahem 恒不合成，避免测试字体倾斜。
                            let frag_synthetic_italic = if is_ahem_frag {
                                false
                            } else {
                                let resolved_italic = text_node_font_italic
                                    .get(&$frag_nid)
                                    .copied()
                                    .unwrap_or(default_resolved_italic);
                                let want_it = owner_style_opt
                                    .is_some_and(|s| {
                                        matches!(s.font_style, FontStyleValue::Italic | FontStyleValue::Oblique(_))
                                    })
                                    || container_want_italic;
                                let synth_allowed = owner_style_opt
                                    .map(|s| s.font_synthesis.style)
                                    .unwrap_or(true);
                                want_it && !resolved_italic && synth_allowed
                            };
                            let emphasis_mark: Option<char> =
                                owner_style_opt.and_then(|s| match s.text_emphasis_style {
                                    TextEmphasisStyleValue::Char(c) => Some(c),
                                    TextEmphasisStyleValue::None => None,
                                });
                            let emphasis_over = owner_style_opt
                                .map(|s| matches!(
                                    s.text_emphasis_position,
                                    TextEmphasisPositionValue::OverRight | TextEmphasisPositionValue::OverLeft
                                ))
                                .unwrap_or(true);
                            // R1689：ruby per-segment annotation（替代 R1022 逐字符 + R1688 整 base 居中）。
                            // R4409：第三元 = 配对 rt 块轴间距（css-ruby #interlinear-block；零间距页逐字节不变）。
                            let ruby_segs: Option<RubyAnnotationSegs> =
                                ruby_annotation_segments(doc, owner_id, styles);

                            // R4133：per-fragment word-spacing——同 multicol 路径。片段 owner
                            // 声明的 ws 在容器 paint style（ws=0）下丢失；nbsp 等 word-separator
                            // 词内不产词界，逐字符 advance 须用 owner 的 ws
                            //（word-spacing-characters-001）。
                            let frag_word_spacing: f32 = owner_style_opt
                                .map(|s| match s.word_spacing {
                                    LengthValue::Px(v) => v as f32,
                                    LengthValue::Percentage(p) => $frag_fs * (p as f32 / 100.0),
                                    ref lv => zero_style_system::computed::resolve_length(
                                        lv,
                                        $frag_fs as f64,
                                        None,
                                        None,
                                    ) as f32,
                                })
                                .unwrap_or(word_spacing);

                            let (frag_base_x, frag_base_y, char_advance_is_y) = if is_vertical {
                                (content_x + $frag_x + tx, content_y + $frag_y + ty, true)
                            } else {
                                (
                                    content_x + $frag_x + tx,
                                    content_y + $frag_y + $baseline_offset + ty,
                                    false,
                                )
                            };
                            let mut char_pos = if char_advance_is_y {
                                frag_base_y
                            } else {
                                frag_base_x
                            };

                            let transformed = apply_text_transform(&$frag_text, &style.text_transform);

                            // R639：text_width 先算（glyph loop 之前），支持 inline bg 在 glyph 下绘制。
                            // R4233：用片段 letter-spacing（与布局片段宽度同源，span 声明
                            // ls ≠ 容器时旧容器值会使宽度错位、text-align 偏移）。
                            let text_width: f32 = transformed
                                .chars()
                                .map(|ch| {
                                    let w = self.measure_char_cached(frag_font_id.0, ch, $frag_fs, $is_ahem)
                                        + frag_letter_spacing;
                                    if zero_style_system::is_word_separator(ch) { w + frag_word_spacing } else { w }
                                })
                                .sum();
                            // R3868：::first-letter 首字母单元簇（前导 P* + 字母 + 后随 P*，除 Ps/Pd）
                            // 的 lazy 判定与伪元素 background 绘制——fill 在 glyph 之前入队（同 R639
                            // 「bg 在 glyph 下」次序）。宽 = 簇字符 advance 和；盒 = fs 上行高近似
                            //（ascent≈fs，行盒 1.164 同 R639 默认）。多列/垂直不应用（FIXME）。
                            if first_letter_color.is_some()
                                && !first_letter_applied
                                && first_letter_expected.is_none()
                                && !char_advance_is_y
                            {
                                let fl_len = text_shaping::first_letter_cluster_len_digraph(
                                    &transformed,
                                    dutch_digraph,
                                );
                                if fl_len > 0 {
                                    first_letter_expected =
                                        Some((transformed.chars().take(fl_len).collect(), 0));
                                    if let Some(bg) = first_letter_bg_color {
                                        let cluster_w: f32 = transformed
                                            .chars()
                                            .take(fl_len)
                                            .map(|ch| {
                                                self.measure_char_cached(frag_font_id.0, ch, $frag_fs, $is_ahem)
                                                    + frag_letter_spacing
                                            })
                                            .sum();
                                        self.primitives.add_fill(
                                            Rect::new(
                                                frag_base_x,
                                                frag_base_y - $frag_fs,
                                                cluster_w,
                                                $frag_fs * 1.164,
                                            ),
                                            bg,
                                        );
                                    }
                                }
                            }
                            // R1689：ruby per-segment annotation —— 每个 rt 居中于其前 base 段
                            //（替代 R1688 整 base 扁平化居中，解 per-kanji Japanese ruby）。水平 only。
                            // R4405：仅 owner **首片段**触发——Path B flatten run 经断行/分词
                            // 拆成多 fragment 时，每 fragment 都携带完整 ruby_segs，无守卫则
                            // 注音按 fragment 数重复绘制（ruby-bidi-004 三份 "123" 实证）。
                            if !char_advance_is_y
                                && r4332_is_first
                                && let Some(RubyAnnotationSegs { segs, span_all }) = ruby_segs.as_ref()
                                && !segs.is_empty()
                            {
                                let rt_fs = $frag_fs * 0.5;
                                let rt_y0 = frag_base_y - $frag_fs;
                                // R4363 临时诊断：ruby overlay 落位追踪（ZW_DEBUG_IFC=1）。
                                if std::env::var("ZW_DEBUG_IFC").as_deref() == Ok("1") {
                                    eprintln!(
                                        "[ruby-overlay-B] frag_base_y={} frag.y={} frag.height={} rt_y={} fs={}",
                                        frag_base_y, $frag_y, 0.0, rt_y0, $frag_fs
                                    );
                                }
                                let mut seg_x = frag_base_x;
                                for (base, annot, rt_spacing) in segs {
                                    // R4409：per-seg 块轴间距——rt 盒 spacing 把注音推离 base。
                                    let rt_y = rt_y0 - rt_spacing;
                                    let seg_w: f32 = base
                                        .chars()
                                        .map(|c| self.measure_char_cached(frag_font_id.0, c, $frag_fs, $is_ahem))
                                        .sum::<f32>()
                                        + frag_letter_spacing * base.chars().count() as f32;
                                    if !annot.is_empty() {
                                        let annot_w: f32 = annot
                                            .chars()
                                            .map(|c| self.measure_char_cached(frag_font_id.0, c, rt_fs, $is_ahem))
                                            .sum();
                                        let mut ax = seg_x + (seg_w - annot_w) / 2.0;
                                        for rc in annot.chars() {
                                            self.primitives.add_glyph(GlyphPrimitive {
                                                x: ax,
                                                y: rt_y,
                                                font_size: rt_fs,
                                                color: frag_color,
                                                glyph_id: rc as u32,
                                                font_glyph_index: None,
                                                source: None,
                                                font_id: frag_font_id,
                                                font_variation_id,
                                                bitmap_width: None,
                                                bitmap_height: None,
                                                rotation,
                    synthetic_italic: frag_synthetic_italic,
                                            });
                                            ax += self.measure_char_cached(frag_font_id.0, rc, rt_fs, $is_ahem);
                                        }
                                    }
                                    seg_x += seg_w;
                                }
                                // R4422（css-ruby-1 #nested-pairing）：span-all 注音行——行 0 上方一个 rt 行高
                                //（rt_fs = 0.5em），跨全部 base 居中（nested/rtc 双形态同一落位）。
                                if let Some(span_text) = &span_all {
                                    let span_y = rt_y0 - 2.0 * rt_fs;
                                    let span_w: f32 = span_text
                                        .chars()
                                        .map(|c| self.measure_char_cached(frag_font_id.0, c, rt_fs, $is_ahem))
                                        .sum();
                                    let mut sx = frag_base_x + (seg_x - frag_base_x - span_w) / 2.0;
                                    for sc in span_text.chars() {
                                        self.primitives.add_glyph(GlyphPrimitive {
                                            x: sx,
                                            y: span_y,
                                            font_size: rt_fs,
                                            color: frag_color,
                                            glyph_id: sc as u32,
                                            font_glyph_index: None,
                                            source: None,
                                            font_id: frag_font_id,
                                            font_variation_id,
                                            bitmap_width: None,
                                            bitmap_height: None,
                                            rotation,
                                            synthetic_italic: false,
                                        });
                                        sx += self.measure_char_cached(frag_font_id.0, sc, rt_fs, $is_ahem);
                                    }
                                }
                            }

                            // R639 Phase A slice：per-line-fragment inline background，仅对跨多行
                            // 的 inline 生效。关键修复（R638 锁定 blocker）：宏的 box_node 是 **IFC
                            // owner**（文本所在容器）非 inline 本身，故多行门控用 **owner inline 自身
                            // height**（self.inline_heights 按 owner_id 查），而非 box_node.height
                            //（IFC owner 的）——后者在 inline 文本处于父 IFC 时与 paint_node 抑制
                            //（inline 自身 box 上）分歧致 bg 消失。两处现均用 inline 自身 height → 一致。
                            // frag_base_x 已含 text-indent（IFC 首行 current_x=text_indent），首行从缩进后起。
                            let owner_h = self.inline_heights.get(&owner_id).copied().unwrap_or(0.0);
                            // R2160 Phase A slice 2 part2：env `ZW_PHASEA_MULTI_INLINE`（**R2198 default-on**；
                            // `=0` kill-switch。R2163 曾 REVERT default-on：orphan 丢 LayoutBox 致 hit_test.rs
                            // 漏收 `<a>`；R2197 slice 3 回填 + R2198 struct-check paint_skip-aware 后复开）
                            // 时，R639 per-fragment bg/border 也对 **orphan inline**（owner_h==0.0，即
                            // part1 skip-taffy 致 inline_heights 无条目）触发——补 part1 丢的 LayoutBox
                            // bg/border 绘制（解 R1492）。单行非 orphan（owner_h∈(0,1.5·fs]）不触发=
                            // 无双绘。orphan 信号（owner_h==0）天然耦合 part1。
                            let phasea_orphan_fire = std::env::var("ZW_PHASEA_MULTI_INLINE").as_deref() != Ok("0")
                                && owner_h == 0.0;
                            // R5296：多行判据升级（与 painter/mod.rs box-level bg 抑制门同源
                            // 同值，成对不变量：box-level 抑制 ⇔ 本臂生效）——第二输入取
                            // owner 自身 computed line-height 解析值（继承链含大 line-height
                            // 时 owner 盒 content_height 随行盒膨胀，旧 `1.5×fs` 高度代理误判
                            // 多行：content-height-001 单行 span 误入本臂，旧 frag.y 位 =
                            // baseline−lh 上溢覆盖邻块）。kill-switch 共用 ZW_INLINE_BG_ML_GATE。
                            let ml_gate_lh = if self.inline_bg_ml_gate_enabled {
                                let own_lh = owner_style_opt.map(|os| match &os.line_height {
                                    zero_style_system::LineHeightValue::Normal => {
                                        $frag_fs * 1.2
                                    }
                                    zero_style_system::LineHeightValue::Number(n) => {
                                        $frag_fs * (*n as f32)
                                    }
                                    zero_style_system::LineHeightValue::Length(l) => {
                                        zero_style_system::computed::resolve_length(
                                            l,
                                            $frag_fs as f64,
                                            None,
                                            None,
                                        ) as f32
                                    }
                                });
                                let measured_max_lh =
                                    self.inline_owner_max_lh.get(&owner_id).copied().unwrap_or(0.0);
                                $frag_fs.max(own_lh.unwrap_or(0.0)).max(measured_max_lh)
                            } else {
                                $frag_fs
                            };
                            let shaping_font_ids = self.fragment_shaping_font_ids(
                                owner_style_opt,
                                text_node_shaping_font_ids.get(&$frag_nid).map(Vec::as_slice),
                                frag_font_id,
                            );
                            if !is_vertical
                                && !box_node.is_absolute
                                && !box_node.is_fixed
                                // R5296：门值 ml_gate_lh 见上（owner 自身 line-height 入阈）。
                                && (owner_h > ml_gate_lh * 1.5 || phasea_orphan_fire)
                                && let Some(owner_style) = styles.and_then(|s| s.get(&owner_id))
                                && matches!(owner_style.display, zero_css_parser::values::DisplayValue::Inline)
                            {
                                // R1442：inline padding/border 不入 line box 高度（CSS §10.8.1）但渲染于
                                // inline box 之外，可上溢/下溢覆盖邻接 line box（border-padding-bleed
                                // driving test）。R1441 定位旧 bg 用 `frag.y`（字形 run 顶 = line.y + f.y，
                                // f.y = baseline_y − line_h）非 line box top，偏移 ~ascent。has_bleed 时改用
                                // `frag.y + baseline_offset`（= glyph 顶 = line box top，Ahem lh:1 em-box）
                                // + 外延 padding+border；并 per-fragment 绘 border-top/bottom（003 border-only）。
                                // gate 不变（仍多行 owner_h>1.5·fs）→ 不触 single-line，避 R638 双计；
                                // bg-only 多行无 padding/border（R639）保持旧 frag.y 位不变。
                                let px_of = |lv: &LengthValue| match lv {
                                    LengthValue::Px(v) => *v as f32,
                                    _ => 0.0,
                                };
                                let bt_w = if matches!(
                                    owner_style.border_top_style,
                                    zero_style_system::property::types::BorderStyleValue::None
                                ) {
                                    0.0
                                } else {
                                    px_of(&owner_style.border_top_width)
                                };
                                let bb_w = if matches!(
                                    owner_style.border_bottom_style,
                                    zero_style_system::property::types::BorderStyleValue::None
                                ) {
                                    0.0
                                } else {
                                    px_of(&owner_style.border_bottom_width)
                                };
                                let pad_top = px_of(&owner_style.padding_top);
                                let pad_bot = px_of(&owner_style.padding_bottom);
                                let pad_left = px_of(&owner_style.padding_left);
                                let pad_right = px_of(&owner_style.padding_right);
                                // R4332：竖边宽（css-break 3 §5.2 slice——左竖边仅首片段、
                                // 右竖边仅末片段；style None → 宽 0，同 bt/bb 口径）。
                                let bl_w = if matches!(
                                    owner_style.border_left_style,
                                    zero_style_system::property::types::BorderStyleValue::None
                                ) {
                                    0.0
                                } else {
                                    px_of(&owner_style.border_left_width)
                                };
                                let br_w = if matches!(
                                    owner_style.border_right_style,
                                    zero_style_system::property::types::BorderStyleValue::None
                                ) {
                                    0.0
                                } else {
                                    px_of(&owner_style.border_right_width)
                                };
                                // 首/末片段外沿水平扩展（横条含角 + 竖边贴片段外缘）：
                                // pad 垂直边随首/末片段（collect 侧 pad 只折入首/末 run，同口径）。
                                // R5299：段命中时改用段标记（逻辑首/末片段所在段）。
                                let lead_ext = if r5299_band_first { pad_left + bl_w } else { 0.0 };
                                let trail_ext = if r5299_band_last { pad_right + br_w } else { 0.0 };
                                // R4549：clip:text 恒色 owner 的 fragment bg 抑制——bg 改由
                                // 子树字形承载（R1442 per-fragment bg 站与 box-level 站同规；
                                // linebreak 案实证首片段 bg 泄漏为实心条）。
                                let has_bg = owner_style.background_color != ColorValue::Transparent
                                    && bg_clip_text_solid_color(
                                        &owner_style,
                                        &self.image_solid_colors,
                                        self.document_url.as_deref(),
                                    )
                                    .is_none();
                                let has_bleed = pad_top > 0.0 || pad_bot > 0.0 || bt_w > 0.0 || bb_w > 0.0;
                                // R5270（css-pseudo-4 §highlight-painting/§highlight-bounds）：
                                // owner 自持选区行高亮——orphan inline（无 LayoutBox，盒级背景
                                // 即本站）被选中时，高亮 fill 须与元素背景**同几何同站**（同
                                // rect 公式、紧随 bg 发射 = 画于其上、字形之下）；R5269 行级
                                // 连体 fill（词间隙 + frag.y 行顶错位）实证铺出带外细条（011
                                // 局部 +0.31pp）故废弃。判据 = owner 选区记录整 contents
                                //（start==0 且 end ≥ textContent utf16 长，≡ R5256 口径）+
                                // owner 自持 ::selection bg 非透明；部分行高亮维持挂账；
                                // clip:text 恒色 owner 与 has_bg 同款抑制（高亮域未见案形）。
                                let owner_sel_fill = if !self.text_selections.is_empty()
                                    && let Some(&(sel_start, sel_end)) = self.text_selections.get(&owner_id)
                                    && sel_start == 0
                                    && sel_end >= doc
                                        .text_content(owner_id)
                                        .map(|t| t.encode_utf16().count())
                                        .unwrap_or(0)
                                    && let Some(sel_pseudo) = owner_style.selection_pseudo.as_ref()
                                    && sel_pseudo.background_color != ColorValue::Transparent
                                    && bg_clip_text_solid_color(
                                        &owner_style,
                                        &self.image_solid_colors,
                                        self.document_url.as_deref(),
                                    )
                                    .is_none()
                                {
                                    Some(super::super::color::resolve_color_current(
                                        &sel_pseudo.background_color,
                                        &owner_style.color,
                                    ))
                                } else {
                                    None
                                };
                                // R4332：仅**真多片段**（跨行）owner 走 per-fragment 边框——
                                // 单片段 owner（含 line-height 撑大 content_height 的单行 span，
                                // bidi-003 的 line-height:3em 形态）保留 box-level border-box
                                // 绘制（R4297 口径：content_height 非行数代理，1.5×fs 门会误触发）。
                                // bg 臂维持基线行为（has_bg 即绘，门不变）。
                                // box-decoration-break: clone（css-break 3 §5.3 每片段整盒）
                                // 未实现——clone owner 保留 box-level 绘制（基线口径），不走 slice。
                                // background-image 与 per-fragment 边框的合成（background-clip:
                                // border-area 等 css-backgrounds-4 语义）未实现——带 bg-image 的
                                // owner 保留 box-level 绘制（基线口径，clip-border-area-box-
                                // decoration-break）。
                                let r4332_multi_fragment = r4332_is_multi_line
                                    && matches!(
                                        owner_style.box_decoration_break,
                                        zero_style_system::property::types::BoxDecorationBreakValue::Slice
                                    )
                                    // R5297：`background: <color>` 简写留一层 None 图像占位
                                    //（非空 vec）——门义为「无**真实**图像」（合成语义未实现），
                                    // 全 None 层视同无图（bgi=1 曾把 bg 色多行 span 挡回词宽
                                    // 旧轨，r5297 单测实证）。
                                    && owner_style
                                        .background_image
                                        .iter()
                                        .all(|img| matches!(
                                            img,
                                            zero_style_system::property::types::BackgroundImageComputedValue::None
                                        ));
                                // R5120（css-break-3 §5.3：slice 单片段 ≡ 整盒）：**单行** owner
                                // 的 border/padding bleed 与多片段 slice 同几何——整盒锚横条
                                //（单行=内容区，见 r5120_content_box；多片段=行盒；均含角外延）
                                // + 首/末竖边。旧 R4332 单片段「基线锚横条 + 无竖边」
                                // 双轨在单行 inline border 全灭：横条落字高中部 [b−bt,b]、
                                // [b+line_h,+bb]、x/宽缺 bl/br 外延、竖边缺失
                                //（border-width-applies-to-008 实测 2 fill vs ref 4 fill 15.53%）。
                                // 多行 clone / bg-image owner 仍走基线口径（合成语义未实现）。
                                // kill-switch `ZW_INLINE_SINGLE_FRAG_BOX_BORDER=1`（OnceLock 单读）
                                // 启用整盒几何；默认关 = 基线锚双轨（R4332 口径，见函数文档）。
                                // R5297：多片段 slice 不受该开关辖制——整盒几何对多片段
                                // **默认生效**（R5120 重构误把它并入此默认关开关，横条回落
                                // 基线锚+词宽：c42-ibx-ht-000 实证 BT 带落字高中部、x 只词
                                // run 宽，chromium 为内容区锚+行片段连续域）；单片段整盒仍
                                // dormant（开关默认关）。
                                let r4332_whole_box = if r4332_is_multi_line {
                                    r4332_multi_fragment
                                } else {
                                    // R5299：backdrop-filter 滤区取盒级 rect
                                    //（effects.rs apply_backdrop_filter R4296/R4297 行盒+
                                    // 外延口径），与整盒内容区锚几何不同域——合成语义未
                                    // 实现，此类 owner 保留基线口径（同 box-decoration-break:
                                    // clone / bg-image owner 先例），防滤镜区与描绘盒错位
                                    //（backdrop-filter-inline-positioning）。
                                    single_frag_box_border_enabled()
                                        && owner_style.backdrop_filter.is_empty()
                                };
                                let legacy_line_top = content_y + $frag_y + $baseline_offset + ty;
                                // R5120（CSS2.1 §10.8.1 content area）：整盒竖直锚 = **内容区**
                                // [baseline−ascent, baseline+descent]（字体原始度量），非行盒——
                                // 显式 line-height 只拉伸行盒，inline 的 border/padding/bg 仍贴内容区
                                //（bidi-003 lh:3em 实证：行盒锚把横条推出 ~0.9em，6 案翻红）。
                                // 度量缺失（宿主未注册 / `ZW_FALLBACK_LINE_METRICS=0`）回退行盒锚。
                                // R5297：多片段 slice 同贴内容区——行盒锚把横条/bg 贴 [T,T+12]
                                // 全行盒且行间无缝，chromium 内容区带 [b−a,b+d] 行间留 half-leading
                                // 缝（c42 两页 oracle 实证 10px 带 + 2px 缝，行盒锚带 12px 连片）。
                                // size-adjust 生效片段的内容区未随缩放——FIXME 遇案再修。
                                let r5120_content_box = if r4332_whole_box {
                                    zero_layout_engine::fallback_line_metrics_for_paint(
                                        &shaping_font_ids,
                                        &transformed,
                                        $frag_fs,
                                    )
                                    .map(|(a, d)| (legacy_line_top - a, a + d))
                                } else {
                                    None
                                };
                                // R5299：单行整盒 + 段命中 = band 域矩形（覆盖段内全部
                                // 片段），仅由首个触达该段的片段绘制一次；多行 slice /
                                // 逐词 legacy 轨维持逐片段绘制（矩形不跨片段，无覆盖）。
                                let r5299_band_once =
                                    r4332_whole_box && !r4332_is_multi_line && r5299_seg_hit;
                                let r5299_draw_band = !r5299_band_once
                                    || r5299_band_drawn.insert((
                                        r4332_owner,
                                        $frag_line_top.to_bits(),
                                        r5299_seg_x0.to_bits(),
                                    ));
                                if (has_bg || has_bleed || owner_sel_fill.is_some())
                                    && r5299_draw_band
                                    {
                                    let line_h = box_node
                                        .text_node_line_heights
                                        .get(&$frag_nid)
                                        .copied()
                                        .unwrap_or($frag_fs * 1.164);
                                    // 行盒顶锚（content_y + line.y + ty）——旧公式传片段基线
                                    //（frag.y + v_offset = GlyphPrimitive 基线坐标，R3856 契约）
                                    // 当 line_top，横条落 [基线-bt_w, 基线] = 字高中部；多片段
                                    // 语义 = 边框/背景贴片段所在行盒外沿（chromium 同）。
                                    let line_top = content_y + $frag_line_top + ty;
                                    // 整盒竖直域（R5120）：单行=内容区锚（度量缺失回退行盒锚），
                                    // 多片段 slice / 非整盒 = 行盒域（box_top/box_h 退化为 line 值）。
                                    let (box_top, box_h) =
                                        r5120_content_box.unwrap_or((line_top, line_h));
                                    let bleed_top = pad_top + bt_w;
                                    let bleed_bot = pad_bot + bb_w;
                                    // bg（has_bg/owner_sel_fill 时）：整盒（whole_box）= 内容区
                                    // 锚（R5120/R5297）+ owner 行连续 x 域 ± pad（首/末片段；
                                    // bg 盒 = 内容盒+pad 不含 border）；has_bleed 竖直外延对齐
                                    // 整盒边。非整盒维持 R639 旧位（frag.y）。矩形外提共享——
                                    // R5270 选区 fill 与元素 bg 同几何。
                                    // R5299：段命中用段标记（逻辑首/末片段所在段）。
                                    let bg_lead = if r5299_band_first { pad_left } else { 0.0 };
                                    let bg_trail = if r5299_band_last { pad_right } else { 0.0 };
                                    let (bg_x, bg_w) = if r4332_whole_box {
                                        (
                                            r5297_band_min_x - bg_lead,
                                            r5297_band_max_end - r5297_band_min_x + bg_lead + bg_trail,
                                        )
                                    } else {
                                        (frag_base_x, text_width)
                                    };
                                    let (bg_y, bg_h) = if has_bleed {
                                        if r4332_whole_box {
                                            (box_top - bleed_top, box_h + bleed_top + bleed_bot)
                                        } else {
                                            (legacy_line_top - bleed_top, line_h + bleed_top + bleed_bot)
                                        }
                                    } else if r4332_whole_box {
                                        (box_top, box_h)
                                    } else {
                                        (content_y + $frag_y + ty, line_h)
                                    };
                                    if has_bg {
                                        self.primitives.add_fill(
                                            Rect::new(bg_x, bg_y, bg_w, bg_h),
                                            color_value_to_render(&owner_style.background_color),
                                        );
                                    }
                                    // R5270 选区行高亮 fill：与元素 bg 同 rect（同站 = 画于其
                                    // 上），宏体次序先于本片段字形 = 字形之下（R5086 splice 同
                                    // 效：高亮不盖字）。零宽片段（text_width==0）跳过。
                                    // R5297 注：选区仍逐 run 域（A/B 校准域 R5052/R5054/R5270
                                    // 自治，不搭车改连续域——FIXME 挂账与 chromium 行片段对齐）。
                                    if let Some(sel_fill) = owner_sel_fill
                                        && text_width > 0.0
                                    {
                                        // R5299：单行整盒几何下选区 fill 与元素 bg 同 x 域
                                        //（行连续域/裂解段）——逐词域在词缝露出元素 bg
                                        //（selection-012：ref 全带绿 vs test 词缝红底）。
                                        // 多行 slice / 非整盒维持逐 run 域（R5052/R5054/
                                        // R5270 A/B 校准口径自治，FIXME 挂账与 chromium
                                        // 行片段对齐）。band 域 once 绘制由块级守卫承担。
                                        let (sel_x, sel_w) = if r4332_whole_box
                                            && !r4332_is_multi_line
                                        {
                                            (r5297_band_min_x, r5297_band_max_end - r5297_band_min_x)
                                        } else {
                                            (frag_base_x, text_width)
                                        };
                                        self.primitives.add_fill(
                                            Rect::new(sel_x, bg_y, sel_w, bg_h),
                                            sel_fill,
                                        );
                                    }
                                    // 横条：整盒几何（多片段 slice=行盒锚 / 单行=内容区锚）+ 含角；
                                    // 多行 clone 等未实现合成域 = 基线锚旧几何。x 域 = R5297
                                    // owner 行连续域（跨词 run 间隙/子 inline 空穴，chromium 行
                                    // 片段连续带），首/末角外延贴域两端。
                                    let (strip_top, strip_x, strip_w) = if r4332_whole_box {
                                        (
                                            box_top - pad_top - bt_w,
                                            r5297_band_min_x - lead_ext,
                                            r5297_band_max_end - r5297_band_min_x + lead_ext + trail_ext,
                                        )
                                    } else {
                                        (legacy_line_top - bt_w, frag_base_x, text_width)
                                    };
                                    if bt_w > 0.0 {
                                        let c = if matches!(owner_style.border_top_color, ColorValue::CurrentColor) {
                                            frag_color
                                        } else {
                                            color_value_to_render(&owner_style.border_top_color)
                                        };
                                        self.primitives.add_fill(Rect::new(strip_x, strip_top, strip_w, bt_w), c);
                                    }
                                    if bb_w > 0.0 {
                                        let c = if matches!(owner_style.border_bottom_color, ColorValue::CurrentColor) {
                                            frag_color
                                        } else {
                                            color_value_to_render(&owner_style.border_bottom_color)
                                        };
                                        let strip_y = if r4332_whole_box {
                                            box_top + box_h + pad_bot
                                        } else {
                                            legacy_line_top + line_h + pad_bot
                                        };
                                        self.primitives.add_fill(Rect::new(strip_x, strip_y, strip_w, bb_w), c);
                                    }
                                    // 竖边：整盒几何——左竖边仅首片段、右竖边仅末片段；纵跨 =
                                    // 整盒高（内容区锚，R5120/R5297）+ 垂直 pad + 顶/底边框
                                    //（与横条锚同域，覆盖邻接行）；x 贴 owner 行连续域外缘。
                                    if r4332_whole_box {
                                        // R5299：竖边归属段标记——逻辑首片段所在段画左
                                        // 竖边、逻辑末片段所在段画右竖边（bidi 重排下
                                        // 视觉 x 序 ≠ 逻辑序）。
                                        if bl_w > 0.0 && r5299_band_first {
                                            let c = if matches!(
                                                owner_style.border_left_color,
                                                ColorValue::CurrentColor
                                            ) {
                                                frag_color
                                            } else {
                                                color_value_to_render(&owner_style.border_left_color)
                                            };
                                            self.primitives.add_fill(
                                                Rect::new(
                                                    r5297_band_min_x - pad_left - bl_w,
                                                    box_top - pad_top - bt_w,
                                                    bl_w,
                                                    box_h + pad_top + pad_bot + bt_w + bb_w,
                                                ),
                                                c,
                                            );
                                        }
                                        if br_w > 0.0 && r5299_band_last {
                                            let c = if matches!(
                                                owner_style.border_right_color,
                                                ColorValue::CurrentColor
                                            ) {
                                                frag_color
                                            } else {
                                                color_value_to_render(&owner_style.border_right_color)
                                            };
                                            self.primitives.add_fill(
                                                Rect::new(
                                                    r5297_band_max_end + pad_right,
                                                    box_top - pad_top - bt_w,
                                                    br_w,
                                                    box_h + pad_top + pad_bot + bt_w + bb_w,
                                                ),
                                                c,
                                            );
                                        }
                                    }
                                }
                            }

                            let size_adjust = fragment_font_size_adjustment(
                                owner_style_opt,
                                box_node.text_node_font_size_adjust.get(&$frag_nid),
                                &style.font_size_adjust,
                                !self.generic_font_ids.contains(&frag_font_id.0),
                            );
                            let shaped_text_eligible = !char_advance_is_y
                                && !$is_ahem
                                // R4233：容器与片段任一带 ls 即走逐字符路径（与 advance
                                // 公式的 ls 来源保持同口径；片段级 ls 原被漏判 → shaped
                                // 路径 glyph 间距丢失）。
                                && letter_spacing == 0.0
                                && frag_letter_spacing == 0.0
                                && word_spacing == 0.0
                                && active_text_shadows.is_empty()
                                && emphasis_mark.is_none()
                                && ruby_segs.as_ref().is_none_or(|r| r.segs.is_empty() && r.span_all.is_none())
                                && !frag_synthetic_italic
                                && !style.text_decoration_line.has_any()
                                && owner_style_opt.is_none_or(|owner| {
                                    !owner.text_decoration_line.has_any()
                                        && (owner.background_color == ColorValue::Transparent
                                            || text_shaping::font_size_adjustment_active(size_adjust))
                                })
                                && rotation.abs() < f32::EPSILON
                                && transformed.chars().all(|ch| !is_cc_control_char(ch));
                            let style_direction = match style.direction {
                                zero_style_system::DirectionValue::Ltr => TextDirection::LeftToRight,
                                zero_style_system::DirectionValue::Rtl => TextDirection::RightToLeft,
                            };
                            let text_direction = text_shaping::fragment_shape_direction(
                                $frag_source,
                                style_direction,
                                text_shaping::shaped_uba_rtl_enabled(),
                            );
                            let logical_source = logical_fragment_source(
                                $frag_source,
                                text_direction,
                                matches!(style.text_transform, TextTransformValue::None),
                            );
                            // https://drafts.csswg.org/css-fonts/#generic-font-families
                            let generic_font = self.generic_font_ids.contains(&frag_font_id.0);
                            let shaped_advance_eligible =
                                text_shaping::fragment_shaped_advance_eligible(generic_font, size_adjust);
                            let open_type_features = style_open_type_features(shaping_style);
                            let advance_trace = fragment_advance_trace(
                                &shaping_font_ids,
                                &transformed,
                                $frag_fs,
                                text_direction,
                                logical_source.as_ref(),
                                &open_type_features,
                                &variations,
                                size_adjust,
                            );
                            // R5086（css2.1 附录 E 绘制次序）：owner 为带非透明背景的行内元素
                            // 时，在首个字形发射前记录 draw_order 位点——owner 自身 paint_node
                            // 的 box-level 背景（父 IFC 字形之后才递归发射）据此移动到字形之前，
                            // 修复背景覆盖自身文字（active-selection-027 ref 页字形消失）。
                            // 多行/orphan owner 走 R639 fragment 背景（box-level 被 skip），
                            // 位点残留不消费、无害。仅主 render_fragment 路径记录。
                            if self.inline_bg_z_order_enabled && !transformed.is_empty() {
                                let r5086_pos = self.primitives.draw_order.len();
                                // R5086：owner 自身带非透明 bg → 记录位点。
                                if let Some(r5086_owner_style) = owner_style_opt
                                    && r5086_owner_style.background_color
                                        != ColorValue::Transparent
                                    && matches!(
                                        r5086_owner_style.display,
                                        DisplayValue::Inline
                                    )
                                {
                                    self.pending_inline_bg_pos
                                        .entry(owner_id)
                                        .or_insert(r5086_pos);
                                }
                                // R5088：wrapper 链补录——带非透明 bg 的 inline 祖先（无直接
                                // 文本时非 fragment owner，owner 是否带 bg 与其无关）其
                                // box-level bg 在自身 paint_node 后置发射，须同移到本片段
                                // 字形之前（R3895 链走查同款；嵌套多层 bg 按 paint_node
                                // 访问序外先内后，splice 后次序保持）。
                                if self.inline_wrapper_bg_enabled {
                                    let mut r5086_cur = doc.parent_node(owner_id);
                                    while let Some(r5086_anc) = r5086_cur {
                                        let Some(r5086_anc_style) =
                                            styles.and_then(|s| s.get(&r5086_anc))
                                        else {
                                            break;
                                        };
                                        if !matches!(
                                            r5086_anc_style.display,
                                            DisplayValue::Inline
                                        ) {
                                            break;
                                        }
                                        if r5086_anc_style.background_color
                                            != ColorValue::Transparent
                                            && !super::in_ruby_subtree(doc, r5086_anc)
                                        {
                                            // ruby 豁免（R5088 A/B 抓 rb-display-001 /
                                            // intra-base-white-space-001 翻红）：ZW ruby 用
                                            // 专属多行片段模型（rt/base 双行同 IFC），box-level
                                            // bg 与 per-fragment bg 的次序/几何交互自洽，链
                                            // 补录 splice 反而扰动（旧路径全绿维持）。
                                            self.pending_inline_bg_pos
                                                .entry(r5086_anc)
                                                .or_insert(r5086_pos);
                                        }
                                        r5086_cur = doc.parent_node(r5086_anc);
                                    }
                                }
                            }
                            for glyph in fragment_glyphs(
                                &shaping_font_ids,
                                &transformed,
                                $frag_fs,
                                shaped_text_eligible,
                                text_direction,
                                shaped_advance_eligible,
                                logical_source,
                                &open_type_features,
                                &variations,
                                size_adjust,
                            ) {
                                let ch = glyph.code_point;
                                let glyph_font_id = glyph.font_id.unwrap_or(frag_font_id);
                                let glyph_font_size = glyph.font_size.unwrap_or($frag_fs);
                                // R4443 临时探针。
                                if char_advance_is_y && glyph_probe_enabled() {
                                    eprintln!(
                                        "[vemit] rot={:.2} slr={} adv={:.1} cy={:.1}",
                                        rotation,
                                        sideways_lr,
                                        glyph.advance_x.unwrap_or(0.0),
                                        content_y
                                    );
                                }
                                // R4443：sideways-lr 行内方向自下而上——先取本字符 advance
                                //（与下方 cursor 推进同式），列内 y 镜像到盒 content 底边锚。
                                let advance = glyph
                                    .advance_x
                                    .unwrap_or_else(|| {
                                        self.measure_char_cached(frag_font_id.0, ch, $frag_fs, $is_ahem)
                                    })
                                    + frag_letter_spacing
                                    + if zero_style_system::is_word_separator(ch) {
                                        frag_word_spacing
                                    } else {
                                        0.0
                                    };
                                let (glyph_x, glyph_y) = if char_advance_is_y {
                                    if sideways_lr {
                                        (
                                            frag_base_x,
                                            content_y + box_node.content_height
                                                - (char_pos - content_y)
                                                - advance,
                                        )
                                    } else {
                                        // R4472：CW 旋转字形的**深度位补偿**——raster
                                        // glyph_top_left 对旋转位图仍按未旋转契约放位
                                        //（start_y = anchor − y_offset − height，Ahem
                                        // y_offset=−(h−ascent)=−0.2em·fs → 方块顶 =
                                        // anchor − 0.8em）。vertical run 的 char_pos 是沿
                                        // 行内轴（物理 y）的推进位 = 字形方块**应有顶**，
                                        // 故 anchor += 0.8em·fs 使方块落 [char_pos, char_pos+fs]
                                        //（inline-table-alignment-002：A 画 3 应 51、F/L 画
                                        // 15 应 111，−0.8em 精确吻合像素实测）。
                                        (frag_base_x, char_pos + $frag_fs * 0.8)
                                    }
                                } else {
                                    (char_pos + glyph.x_offset, frag_base_y - glyph.y_offset)
                                };
                                // ::first-letter 逐字形覆色：本片段是容器首个含非空白字符的文本片段、
                                // 且该字形是其中第一个非空白字符时，用伪元素 color（CSS2 §5.12.2）。
                                let glyph_color = {
                                    let mut c = frag_color;
                                    if let Some(fl) = first_letter_color
                                        && !first_letter_applied
                                    {
                                        // 首个含非空白字符的片段：初始化期望簇（懒执行，保证是
                                        // 容器文档序第一个非空白片段而非任一空片段）。
                                        if first_letter_expected.is_none() {
                                            // R3871：簇已在上方 bg 块以 digraph 语义初始化；此处兜底
                                            // 仅处理「簇为 0」（片段无字母）放弃情形。
                                            let len = text_shaping::first_letter_cluster_len_digraph(
                                                &transformed,
                                                dutch_digraph,
                                            );
                                            if len > 0 {
                                                first_letter_expected =
                                                    Some((transformed.chars().take(len).collect(), 0));
                                            } else {
                                                // 片段全是标点/空格（无字母）→ 不染，等后续片段
                                            }
                                        }
                                        if let Some((chars, pos)) = &mut first_letter_expected {
                                            if chars.get(*pos) == Some(&ch) {
                                                c = fl;
                                                *pos += 1;
                                                if *pos == chars.len() {
                                                    first_letter_expected = None;
                                                    first_letter_applied = true;
                                                }
                                            } else {
                                                // 字形与期望字符错位（shaping 合并/变体）→ 放弃
                                                first_letter_expected = None;
                                                first_letter_applied = true;
                                            }
                                        } else if first_letter_applied {
                                            // 已完成或已放弃
                                        } else {
                                            first_letter_applied = true; // 簇为空且非空片段（防御：勿再试后续片段）
                                        }
                                    }
                                    c
                                };

                                for &(shadow_ox, shadow_oy, shadow_color) in &active_text_shadows {
                                    self.primitives.add_glyph(GlyphPrimitive {
                                        x: glyph_x + shadow_ox,
                                        y: glyph_y + shadow_oy,
                                        font_size: glyph_font_size,
                                        color: shadow_color,
                                        glyph_id: ch as u32,
                                        font_glyph_index: None,
                                        source: None,
                                        font_id: glyph_font_id,
                                        font_variation_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                    synthetic_italic: frag_synthetic_italic,
                                    });
                                }

                                self.primitives.add_glyph(GlyphPrimitive {
                                    x: glyph_x,
                                    y: glyph_y,
                                    font_size: glyph_font_size,
                                    color: glyph_color,
                                    glyph_id: ch as u32,
                                    font_glyph_index: glyph.font_glyph_index,
                                    source: glyph.source.clone(),
                                    font_id: glyph_font_id,
                                    font_variation_id,
                                    bitmap_width: None,
                                    bitmap_height: None,
                                    rotation,
                    synthetic_italic: frag_synthetic_italic,
                                });

                                // R644：Cc 控制字符可见性（CSS Text 3）——fontdue 对 Cc 无字形
                                //（.notdef 空），渲染可见占位框（em 方块），使 control-chars-* mismatch
                                // 测试 test != 空 ref（diff > min_mismatch_ratio 0.5%；fs×fs em 方块
                                // 在 4em=64px 下 ~0.85% diff，超阈值）。
                                if is_cc_control_char(ch) {
                                    self.primitives.add_fill(
                                        Rect::new(
                                            glyph_x,
                                            glyph_y - glyph_font_size,
                                            glyph_font_size,
                                            glyph_font_size,
                                        ),
                                        frag_color,
                                    );
                                }

                                // R4443：advance 已在字形定位处预计算（sideways-lr 镜像需要）。
                                char_pos += advance;

                                // R1021：text-emphasis 标记（水平书写模式；垂直暂不支持）。
                                if !char_advance_is_y
                                    && !ch.is_whitespace()
                                    && let Some(mark_ch) = emphasis_mark
                                {
                                    let mark_fs = $frag_fs * 0.5;
                                    let mark_advance = self.measure_char_cached(frag_font_id.0, mark_ch, mark_fs, $is_ahem);
                                    let mark_x = char_pos - advance / 2.0 - mark_advance / 2.0;
                                    // over：mark 基线在文本顶部之上（leading 区）；under：基线之下
                                    let mark_y = if emphasis_over {
                                        frag_base_y - $frag_fs - mark_fs * 0.4
                                    } else {
                                        frag_base_y + $frag_fs * 0.5
                                    };
                                    self.primitives.add_glyph(GlyphPrimitive {
                                        x: mark_x,
                                        y: mark_y,
                                        font_size: mark_fs,
                                        color: emphasis_color,
                                        glyph_id: mark_ch as u32,
                                        font_glyph_index: None,
                                        source: None,
                                        font_id: frag_font_id,
                                        font_variation_id,
                                        bitmap_width: None,
                                        bitmap_height: None,
                                        rotation,
                    synthetic_italic: frag_synthetic_italic,
                                    });
                                }

                                // R1022 per-char ruby annotation 已移至 text_width 后的
                                // segment-centered 块（R1688）—— 整 rt 注音居中于 base segment。
                            }

                            if let Some(trace) = advance_trace {
                                trace.emit(
                                    if use_stored { "stored" } else { "paint-ifc" },
                                    frag_font_id.0,
                                    $frag_fs,
                                    &transformed,
                                    FragmentPaintWidths {
                                        fragment: $frag_width,
                                        legacy: text_width,
                                        consumed: char_pos - frag_base_x,
                                        fragment_x: frag_base_x,
                                    },
                                );
                            }

                            let (skip_s, skip_e) = self.decoration_skip_trims(
                                style,
                                &transformed,
                                frag_font_id.0,
                                $frag_fs,
                                is_ahem_frag,
                                frag_letter_spacing,
                            );
                            self.paint_text_decoration_from_style(
                                frag_base_x,
                                frag_base_y,
                                $frag_fs,
                                text_width,
                                frag_color,
                                style,
                                skip_s,
                                skip_e,
                            );
                            // R5204：owner 自身声明的装饰补绘（CSS 2.1 §16.3.1 各元素
                            // 自设装饰独立成线）。text-decoration-line 非继承，容器无
                            // 装饰而 span 自声明时（skip-spaces-001~008 ref 页
                            // span{underline} 结构）此前整条丢失。上方容器调用已覆盖
                            // 直系文本（owner == 容器盒），此处仅补 flatten/嵌套 owner；
                            // 更深包装链（span>em>text 中 span 的装饰）仍缺（FIXME）。
                            if let Some(owner_style) = owner_style_opt
                                && owner_style.text_decoration_line.has_any()
                                && Some(owner_id) != box_node.node_id
                            {
                                let (o_skip_s, o_skip_e) = self.decoration_skip_trims(
                                    owner_style,
                                    &transformed,
                                    frag_font_id.0,
                                    $frag_fs,
                                    is_ahem_frag,
                                    frag_letter_spacing,
                                );
                                self.paint_text_decoration_from_style(
                                    frag_base_x,
                                    frag_base_y,
                                    $frag_fs,
                                    text_width,
                                    frag_color,
                                    owner_style,
                                    o_skip_s,
                                    o_skip_e,
                                );
                            }
                        }};
                    }

                    // R5053（css-pseudo-4 §highlight-selectors）：IFC 文本路径 ::selection
                    // 行级高亮——textarea 无 retained 值的可见文本走本路径（R5052 只接了
                    // paint_input_value 的 attr/retained 路径），普通元素直接文本同门。
                    // 门：本盒自身即选区 owner（harness 回流 selector 解析的 NodeId ==
                    // box_node.node_id）+ 作者 ::selection 规则 + LTR + 水平书写 + 非折叠
                    // 选区。粒度 = 整行（≡ R5052）：行内本 owner 片段 utf16 区间 ⊆ 选区
                    // → 该行字形用伪样式 color（经 render_fragment! 调用点传入覆写）。
                    // 高亮矩形：R5054 分形——表单控件 owner（input/textarea）铺 padding
                    // box（≡ chromium 文本控件选区铺满内区；全部内容行选中才发）；
                    // 其余 owner 逐行铺**片段横向范围**（≡ chromium 普通文本选区只盖
                    // 文本延伸，driving: selection-background-color-001 ref=100px 方块）。
                    // 渲染序累计 utf16 对 select-all 形态无对齐偏差；部分选区的空白折叠
                    // 偏移与后代 owner（span::selection）均不在本切片（FIXME）。
                    let mut sel_line_colors: HashMap<(NodeId, u32), Color> = HashMap::new();
                    let mut sel_padbox_fill: Option<Color> = None;
                    let mut sel_line_fills: Vec<(Rect, Color)> = Vec::new();
                    // R5256：整contents 选区的后代 owner 行（R5054 后代 owner FIXME
                    // 首片）——selectNodeContents(盒/祖先) 形下片段行可归 inline 后代
                    // owner（ref 页 `<div 选区><span>text</span></div>` 的 span 文本），
                    // 此前 owner==box_id 过滤整行跳过。整contents 判据 start==0 且
                    // end ≥ owner（自身无记录时最近祖先）textContent utf16 长（≡ shim
                    // dump branch ① 的 textContent 口径），该形全部片段行无条件选中；
                    // 部分选区维持既有 owner==box_id + 行子集检查不变。
                    let sel_owned = box_node
                        .node_id
                        .and_then(|id| self.text_selections.get(&id).map(|&(s, e)| (id, s, e)))
                        .map(|(id, s, e)| {
                            let whole = doc
                                .text_content(id)
                                .map(|t| t.encode_utf16().count())
                                .is_some_and(|n| s == 0 && e >= n);
                            ((s, e), whole)
                        })
                        .or_else(|| {
                            let mut cur = box_node.node_id.and_then(|id| doc.parent_node(id));
                            while let Some(aid) = cur {
                                if let Some(&(s, e)) = self.text_selections.get(&aid) {
                                    let total = doc.text_content(aid).map(|t| t.encode_utf16().count()).unwrap_or(0);
                                    return (s == 0 && e >= total).then_some(((s, e), true));
                                }
                                cur = doc.parent_node(aid);
                            }
                            None
                        });
                    // 行分组（活跃路径片段 → (owner, 行盒顶) 组，含行内横向范围）——
                    // R5258 起选区门与 ::first-line 门共用；两门皆不需要时跳过构建
                    //（paint_text 热路径零闲置分配）。
                    let line_ranges: Vec<((NodeId, u32), usize, usize, f32, f32)> = if style.first_line_pseudo.is_some()
                        || sel_owned.is_some()
                        // R5269（css-pseudo-4 §highlight-cascading）：owner 自持选区
                        //（inline 元素自身被 selectNodeContents 选中——文本经父块
                        // paint_text 片段路径绘制，盒级 sel_owned 查不到）也需行分组。
                        // 门 = 选区非空（product 零选区零扫描）+ 任一片段 owner 有记录。
                        || (!self.text_selections.is_empty() && {
                            let mut any = false;
                            if use_stored {
                                for f in stored_fragments.iter() {
                                    let o = if doc.get(f.node_id).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))) {
                                        doc.parent_node(f.node_id).unwrap_or(f.node_id)
                                    } else {
                                        f.node_id
                                    };
                                    if self.text_selections.contains_key(&o) {
                                        any = true;
                                        break;
                                    }
                                }
                            } else {
                                for f in fragments.iter() {
                                    let o = if doc.get(f.node_id).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))) {
                                        doc.parent_node(f.node_id).unwrap_or(f.node_id)
                                    } else {
                                        f.node_id
                                    };
                                    if self.text_selections.contains_key(&o) {
                                        any = true;
                                        break;
                                    }
                                }
                            }
                            any
                        }) {
                        // 活跃路径的 (node_id, 行盒顶, 片段文本, x, 宽) 序列（渲染序 ≈ 文档序）。
                        let active_frags: Vec<(NodeId, f32, &str, f32, f32)> = if use_stored {
                            stored_fragments
                                .iter()
                                .map(|f| (f.node_id, f.line_top_rel, f.text.as_str(), f.x, f.width))
                                .collect()
                        } else {
                            fragments
                                .iter()
                                .enumerate()
                                .map(|(gi, f)| {
                                    (
                                        f.node_id,
                                        frag_line_tops.get(gi).copied().unwrap_or(0.0),
                                        f.text.as_str(),
                                        f.x,
                                        f.width,
                                    )
                                })
                                .collect()
                        };
                        // owner 解析 ≡ render_fragment! r4332_owner（文本节点取父元素）。
                        let owner_of = |nid: NodeId| -> NodeId {
                            if doc.get(nid).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))) {
                                doc.parent_node(nid).unwrap_or(nid)
                            } else {
                                nid
                            }
                        };
                        // 逐 owner utf16 累计 + 行分组（软换行不消耗字符；同行连续片段
                        // 首片段前偏移即行起点）+ 行内横向范围。
                        let mut owner_off: HashMap<NodeId, usize> = HashMap::new();
                        let mut ranges: Vec<((NodeId, u32), usize, usize, f32, f32)> = Vec::new();
                        for (nid, line_top, text, fx, fw) in &active_frags {
                            let owner = owner_of(*nid);
                            let off = owner_off.entry(owner).or_insert(0);
                            // R5275：R1447 tab 片段 text="" 但在 textContent 占 1 个 utf16
                            // 位置（CSS Text 3 §4.1.3 preserve 保留 tab）——空文本**文本节点**
                            // 片段恒为保留 tab，按 1 计入 owner utf16 累计，行坐标与 shim
                            // dump（textContent 口径）同系；否则 preserve 空白行区间退化
                            // (0,0) 被行门整跳（active-selection-063 高亮缺失）。替换元素
                            // 片段（img 等，节点为元素）不占 textContent，len 恒 0，原行门
                            // 仍跳过（selection-image-001 图片区不得 fabrication 行 fill）。
                            let len16 = if text.is_empty()
                                && doc.get(*nid).is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                            {
                                1
                            } else {
                                text.encode_utf16().count()
                            };
                            match ranges
                                .iter_mut()
                                .find(|((o, l), _, _, _, _)| *o == owner && *l == line_top.to_bits())
                            {
                                Some((_, _, end, min_x, max_xw)) => {
                                    *end += len16;
                                    *min_x = min_x.min(*fx);
                                    *max_xw = max_xw.max(*fx + *fw);
                                }
                                None => ranges.push(((owner, line_top.to_bits()), *off, *off + len16, *fx, *fx + *fw)),
                            }
                            *off += len16;
                        }
                        ranges
                    } else {
                        Vec::new()
                    };
                    // R5258（CSS2 §5.12.1 切片 1）：::first-line 首格式行字形 color——
                    // 行组 line_top == 组内最小者为首行；组内 inline 后代解析 color ==
                    // 盒 color（≈继承、无显式覆盖）才随伪样式（span{color} 显式声明
                    // 胜出 ::first-line 继承，css-pseudo-4 §first-line）。先于选区门
                    // insert（选区行覆写胜出）。bg/font 类属性不应用（FIXME 切片 2）；
                    // 与 ::first-letter 同存时首字符 interplay 未处理（FIXME）；竖排
                    // 不应用（FIXME）。
                    if !is_vertical
                        && let Some(fl) = &style.first_line_pseudo
                        && !line_ranges.is_empty()
                    {
                        let first_ltb = line_ranges.iter().map(|((_, ltb), ..)| *ltb).min().unwrap_or(0);
                        let fl_color = super::super::color::resolve_color_current(&fl.color, &style.color);
                        for ((owner, ltb), _, _, _, _) in &line_ranges {
                            if *ltb != first_ltb {
                                continue;
                            }
                            let inherited = styles
                                .and_then(|st| st.get(owner))
                                .is_none_or(|os| os.color == style.color);
                            if inherited {
                                sel_line_colors.insert((*owner, *ltb), fl_color);
                            }
                        }
                    }
                    if let Some(box_id) = box_node.node_id
                        && !is_vertical
                        && matches!(style.direction, zero_style_system::DirectionValue::Ltr)
                        && let Some(((sel_start, sel_end), anc_whole)) = sel_owned
                        && (style.selection_pseudo.is_some()
                            || std::env::var("ZW_SELECTION_HIGHLIGHT_DEFAULT").as_deref() != Ok("0"))
                    {
                        // R5272（css-pseudo-4 §highlight-styling + css-color-4
                        // §css-system-colors）：盒/祖先链有选区记录而盒无作者
                        // ::selection 伪样式时回落 **UA 默认高亮面** Highlight 背景 +
                        // HighlightText 前景——driving: selection-image-002（页零
                        // ::selection 规则，selectNode(documentElement) 记录落 html，
                        // p 行经 anc_whole 本臂处理，此前门 `selection_pseudo` 整关）。
                        // 与 R5269 臂回落同开关同色对；作者声明面（含显式 transparent）
                        // selection_pseudo 已存不回落。
                        let (sel_color, sel_bg) = match &style.selection_pseudo {
                            Some(pseudo) => (
                                super::super::color::resolve_color_current(&pseudo.color, &style.color),
                                super::super::color::resolve_color_current(&pseudo.background_color, &style.color),
                            ),
                            None => (
                                zero_css_parser::values::parse_named_color("highlighttext")
                                    .map(|c| super::super::color::resolve_color_current(&c, &style.color))
                                    .unwrap_or_else(|| {
                                        super::super::color::resolve_color_current(&style.color, &style.color)
                                    }),
                                zero_css_parser::values::parse_named_color("highlight")
                                    .map(|c| super::super::color::resolve_color_current(&c, &style.color))
                                    .unwrap_or(zero_render_foundation::color::Color::TRANSPARENT),
                            ),
                        };
                        // 表单控件 owner（padding box 形态判据）。
                        let owner_is_text_control = doc.get(box_id).is_some_and(|n| {
                            matches!(
                                &n.kind,
                                NodeKind::Element(e) if e.local_name().eq_ignore_ascii_case("input")
                                    || e.local_name().eq_ignore_ascii_case("textarea")
                            )
                        });
                        let line_h = match &style.line_height {
                            zero_style_system::LineHeightValue::Normal => font_size * 1.2,
                            zero_style_system::LineHeightValue::Number(n) => font_size * (*n as f32),
                            zero_style_system::LineHeightValue::Length(length) => {
                                zero_style_system::computed::resolve_length(length, font_size as f64, None, None) as f32
                            }
                        };
                        let mut any_line = false;
                        let mut all_selected = true;
                        for ((owner, ltb), line_start, line_end, min_x, max_xw) in &line_ranges {
                            // 祖先整contents 形：本盒片段迭代的所有 owner（自身或 inline
                            // 后代）都在选区内，跨 owner 行不再按 owner==box_id 过滤。
                            if !anc_whole && *owner != box_id {
                                continue;
                            }
                            if line_end <= line_start {
                                continue;
                            }
                            any_line = true;
                            if anc_whole || (sel_start <= *line_start && *line_end <= sel_end) {
                                // R5257（css-pseudo-4 §highlight-cascading 后续片）：后代
                                // owner 自持 ::selection 规则时高亮样式按 owner 独立级联
                                //（universal ::selection{green/yellow} vs div#test::selection
                                // {red/black} 的 span 行应 green/yellow，R5256 整contents 形
                                // 用盒伪样式曾整段涂红——driving: cascade-highlight-002）。
                                // owner 无自持记录回落盒伪样式（R5256 判例全保持）；门仍
                                // 要求盒自持伪样式（后代-only 形维持 FIXME）。
                                let (line_color, line_bg) = if *owner == box_id {
                                    (sel_color, sel_bg)
                                } else {
                                    styles
                                        .and_then(|st| st.get(owner))
                                        .and_then(|os| {
                                            os.selection_pseudo.as_ref().map(|p| {
                                                (
                                                    super::super::color::resolve_color_current(&p.color, &os.color),
                                                    super::super::color::resolve_color_current(
                                                        &p.background_color,
                                                        &os.color,
                                                    ),
                                                )
                                            })
                                        })
                                        .unwrap_or((sel_color, sel_bg))
                                };
                                sel_line_colors.insert((*owner, *ltb), line_color);
                                if !owner_is_text_control && line_bg.a > 0 {
                                    sel_line_fills.push((
                                        Rect::new(
                                            content_x + min_x,
                                            content_y + f32::from_bits(*ltb),
                                            (max_xw - min_x).max(0.0),
                                            line_h,
                                        ),
                                        line_bg,
                                    ));
                                }
                            } else {
                                all_selected = false;
                            }
                        }
                        if owner_is_text_control && any_line && all_selected && sel_bg.a > 0 {
                            sel_padbox_fill = Some(sel_bg);
                        }
                    }
                    // R5269（css-pseudo-4 §highlight-cascading 起源元素独立级联——owner
                    // 自持选区形）：inline 元素自身被选中（selectNodeContents(inline)，
                    // active-selection-012 型）时其文本经父块 paint_text 片段路径绘制，
                    // 盒级 sel_owned（盒/祖先链）查不到 owner 记录 → 盒级臂整关、整行
                    // 零高亮。此臂按片段 owner 自身选区记录判定行选中，字形色按 owner
                    // 自持 ::selection 级联；无自持伪样式的行跳过（无 UA 默认高亮维持
                    // 既有挂账）。盒级臂已处理的盒自身行（sel_owned 命中且 owner==box_id）
                    // 不重复。行完全落在 owner 选区内才着色（部分行高亮维持挂账）；
                    // 竖排/RTL 同盒级臂维持挂账。
                    // 行高亮 fill 不在本臂——R5270 已于片段 bg 发射站（has_bg 块）按同
                    // rect 公式铺 owner 整contents 选区 fill（同几何同站、字形之下）；
                    // 部分行高亮维持挂账；非 orphan inline（有 LayoutBox、盒级 bg）的
                    // 盒级 bg 站 fill 未接——语料零案，FIXME 遇案再修。
                    if let Some(box_id) = box_node.node_id
                        && !is_vertical
                        && matches!(style.direction, zero_style_system::DirectionValue::Ltr)
                        && !self.text_selections.is_empty()
                    {
                        for ((owner, ltb), line_start, line_end, min_x, max_xw) in &line_ranges {
                            if sel_owned.is_some() && *owner == box_id {
                                continue;
                            }
                            let Some(&(sel_start, sel_end)) = self.text_selections.get(owner) else {
                                continue;
                            };
                            if line_end <= line_start || !(sel_start <= *line_start && *line_end <= sel_end) {
                                continue;
                            }
                            let Some(os) = styles.and_then(|st| st.get(owner)) else {
                                continue;
                            };
                            // R5271（css-pseudo-4 §highlight-styling/§highlight-cascading）：
                            // owner 无作者 ::selection 记录时回落 **UA 默认高亮面**
                            // Highlight 背景 + HighlightText 前景（css-color-4 §css-system-
                            // colors；ref 页同关键字静态样式 → 双侧同 RGB 必收敛）。
                            // driving: system-color-hightlights-vs-getSelection-001/002
                            //（selectNode(span) 5.25pp 合计带）。作者声明面（含显式
                            // transparent——selection_pseudo 已存）不回落；kill-switch
                            // `ZW_SELECTION_HIGHLIGHT_DEFAULT=0`（≡ R5065 同语义开关）回退
                            // 「无规则无高亮」旧口径。
                            let (line_color, line_bg) = match os.selection_pseudo.as_ref() {
                                Some(p) => (
                                    super::super::color::resolve_color_current(&p.color, &os.color),
                                    super::super::color::resolve_color_current(&p.background_color, &os.color),
                                ),
                                None if std::env::var("ZW_SELECTION_HIGHLIGHT_DEFAULT").as_deref() != Ok("0") => (
                                    zero_css_parser::values::parse_named_color("highlighttext")
                                        .map(|c| super::super::color::resolve_color_current(&c, &os.color))
                                        .unwrap_or_else(|| {
                                            super::super::color::resolve_color_current(&os.color, &os.color)
                                        }),
                                    zero_css_parser::values::parse_named_color("highlight")
                                        .map(|c| super::super::color::resolve_color_current(&c, &os.color))
                                        .unwrap_or(zero_render_foundation::color::Color::TRANSPARENT),
                                ),
                                None => continue,
                            };
                            sel_line_colors.insert((*owner, *ltb), line_color);
                            // R5271 fill 去重：orphan/多片段 owner 的**整 contents**选中行
                            // 已由 R5270 片段 bg 站 fill 覆盖（逐 run 同站同几何）——本臂
                            // 行级连体 rect（跨词间隙）会双绘（r5270 锚 greens 3 vs 2 实证）。
                            // 站 fill 判据镜像 owner_sel_fill：自持伪样式 bg 非透明 +
                            // 整 contents + 站门（orphan 或真多片段）。非 orphan 单行（站门
                            // 不触发）与部分选区下逐行整盖由本臂补位；UA 默认回落臂（无伪
                            // 样式）站恒不铺 → 恒由本臂补。clip:text 恒色 owner 与站同款
                            // 抑制（R4549 口径，r5270 锚）——本臂 fill 亦不铺。
                            let clip_ok =
                                bg_clip_text_solid_color(os, &self.image_solid_colors, self.document_url.as_deref())
                                    .is_none();
                            let station_fills = clip_ok
                                && os
                                    .selection_pseudo
                                    .as_ref()
                                    .is_some_and(|p| p.background_color != ColorValue::Transparent)
                                && sel_start == 0
                                && sel_end >= doc.text_content(*owner).map(|t| t.encode_utf16().count()).unwrap_or(0)
                                && std::env::var("ZW_PHASEA_MULTI_INLINE").as_deref() != Ok("0")
                                && match self.inline_heights.get(owner) {
                                    None => true,
                                    Some(&h) => {
                                        let fs = match os.font_size {
                                            zero_style_system::LengthValue::Px(v) => v,
                                            _ => 0.0,
                                        };
                                        h as f64 > fs * 1.5
                                    }
                                };
                            if line_bg.a > 0 && clip_ok && !station_fills {
                                // 行高 ≡ 盒级臂口径（容器 line_height 折算，3584 同式——
                                // 高亮行界随行盒非 owner 伪样式）。
                                let owner_line_h = match &style.line_height {
                                    zero_style_system::LineHeightValue::Normal => font_size * 1.2,
                                    zero_style_system::LineHeightValue::Number(n) => font_size * (*n as f32),
                                    zero_style_system::LineHeightValue::Length(length) => {
                                        zero_style_system::computed::resolve_length(
                                            length,
                                            font_size as f64,
                                            None,
                                            None,
                                        ) as f32
                                    }
                                };
                                sel_line_fills.push((
                                    Rect::new(
                                        content_x + min_x,
                                        content_y + f32::from_bits(*ltb),
                                        (*max_xw - *min_x).max(0.0),
                                        owner_line_h,
                                    ),
                                    line_bg,
                                ));
                            }
                        }
                    }
                    if let Some(sel_bg) = sel_padbox_fill {
                        // 高亮铺 padding box（背景绘制区内、UA 边框内缘，R5052 同式）。
                        self.primitives.add_fill(
                            Rect::new(
                                abs_x + box_node.border_left,
                                abs_y + box_node.border_top,
                                (box_node.width - box_node.border_left - box_node.border_right).max(0.0),
                                (box_node.height - box_node.border_top - box_node.border_bottom).max(0.0),
                            ),
                            sel_bg,
                        );
                    }
                    for (r, c) in &sel_line_fills {
                        self.primitives.add_fill(*r, *c);
                    }

                    // R5177：color 差异拆分归因片段的祖先载体补标（宏内逐片段上溯的
                    // 热点外提）——拆分归因后 frag nid 是内层元素，外层 inline 载体
                    //（has_direct_paintable_text 的 PHASEA_STORE_EXT 臂放行 paint_text）
                    // 的 Path B 重跑会重复渲染已由 IFC 片段覆盖的子树文本（span>
                    // div{display:inline} probe 双绘实证）。仅对拆分归因 owner 上溯
                    // display:Inline 补标；divergent 集空短路零开销（普通 run 归因 id
                    // 即外层自身，宏内 insert(frag_nid) 已覆盖）。
                    let r5177_divergent = &box_node.inline_color_divergent_nodes;
                    if !r5177_divergent.is_empty()
                        && let Some(styles) = styles
                    {
                        mark_split_owner_ancestor_chain_painted(
                            &mut self.painted_inline_nodes,
                            doc,
                            styles,
                            stored_fragments
                                .iter()
                                .map(|f| f.node_id)
                                .chain(fragments.iter().map(|f| f.node_id)),
                            r5177_divergent,
                        );
                    }

                    // R5297：行域累计的止步盒——块容器 IFC owner（其背景走盒级路径，
                    // 不消费行域表）止步；**inline box_node 自绘 IFC 时不可止步于自身**
                    //（span 自盒存储 IFC 的 pass：片段 owner = 自身，止步会把自身行域
                    // 全部剔除 → 查表全 miss 回落词宽，r5297 单测实证）。inline 盒的
                    // 祖先累计上溢到块祖先为止（块祖先表项无 fragment 消费，无害）。
                    let r5297_walk_stop = if matches!(style.display, zero_css_parser::values::DisplayValue::Inline) {
                        None
                    } else {
                        box_node.node_id
                    };
                    if use_stored {
                        // R4332：owner 片段序首/末旗标（宏参数传入，见宏首注释）。
                        let r4332_edges = owner_fragment_edge_flags(
                            stored_fragments.iter().map(|f| (f.node_id, f.line_top_rel)),
                            doc,
                        );
                        // R5297：owner 行连续 x 域（横条/bg 跨 run 间隙与子 inline 穴，
                        // 祖先累计止于 IFC owner 盒）。
                        let r5297_band_extents = owner_line_band_extent_map(
                            stored_fragments
                                .iter()
                                .map(|f| (f.node_id, f.line_top_rel, f.x, f.width)),
                            doc,
                            r5297_walk_stop,
                        );
                        // R5299：单行整盒路径的裂解段表（旗标关 = 空表，宏内自然回落
                        // 行连续域）。
                        let r5299_seg_extents = if single_frag_box_border_enabled() {
                            owner_line_segment_extent_map(
                                stored_fragments
                                    .iter()
                                    .map(|f| (f.node_id, f.line_top_rel, f.x, f.width)),
                                doc,
                                r5297_walk_stop,
                            )
                        } else {
                            HashMap::new()
                        };
                        for (frag_ei, frag) in stored_fragments.iter().enumerate() {
                            // R817 linebox 度量统一 Phase 2 → R3856 基线契约统一：GlyphPrimitive.y
                            // 即基线（raster 侧 y_offset = bitmap_top − height 自行放置位图，
                            // Ahem 方块由 rasterize_ahem_glyph 放到 [baseline−0.8em, baseline+0.2em]，
                            // Skia 同）。macro glyph_y = content_y + frag.y + v_offset，故 Ahem 字形
                            // v_offset = baseline_y_abs − frag.y → glyph_y = 行基线绝对 y。
                            // 旧实现把 em-box 顶（baseline−0.8·fs）当基线传，raster 再上移后
                            // 方块整体高 0.2em（blocks-020：绿色 XXX 铺 30.6..130.6 而非
                            // 50.6..150.6，露出下方红色 outer）。
                            // 仅对**真正** Ahem 字形（is_ahem_font，来自 IFC run 实际字体）应用——
                            // 容器为 Ahem 但片段实为其它字体（font-051 的 serif span）时保留旧
                            // 容器级行为（is_ahem?0:font_size），避免按 ascent=font_size 错移。
                            // R4472：vertical 的 v_offset 不进 frag_base_y（深度位直接锚，
                            // 见发射处 char_pos + 0.8em 补偿），仅 R639 inline bg 沿用旧 0 口径。
                            let v_offset = if frag.is_ahem_font {
                                frag.baseline_y_abs - frag.y
                            } else if frag.is_ahem {
                                0.0
                            } else if stored_baseline_anchor_enabled() {
                                // R5169：非 Ahem 片段同样基线锚定——存储 IFC 的 baseline_y
                                // 是 struts 真实行基线（行顶 + half-leading + ascent），旧
                                // `frag.font_size` 近似把基线当 fs（infer-cells monospace
                                // 32px 行基线 37 → 旧 32，字形整体高 5px，红绿叠层错位实证）。
                                frag.baseline_y_abs - frag.y
                            } else {
                                frag.font_size
                            };
                            render_fragment!(
                                frag.x,
                                frag.y,
                                frag.line_top_rel,
                                r4332_edges[frag_ei],
                                frag.width,
                                v_offset,
                                frag.font_size,
                                frag.text,
                                frag.node_id,
                                frag.is_ahem,
                                frag.source.as_ref(),
                                frag.letter_spacing,
                                sel_line_colors
                                    .get(&(
                                        if doc
                                            .get(frag.node_id)
                                            .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                                        {
                                            doc.parent_node(frag.node_id).unwrap_or(frag.node_id)
                                        } else {
                                            frag.node_id
                                        },
                                        frag.line_top_rel.to_bits()
                                    ))
                                    .copied(),
                                &r5297_band_extents,
                                &r5299_seg_extents
                            );
                        }
                    } else {
                        // R4332：owner 片段序首/末旗标（宏参数传入，见宏首注释）。
                        let r4332_edges = owner_fragment_edge_flags(
                            fragments
                                .iter()
                                .enumerate()
                                .map(|(gi, f)| (f.node_id, frag_line_tops.get(gi).copied().unwrap_or(0.0))),
                            doc,
                        );
                        // R5297：owner 行连续 x 域（横条/bg 跨 run 间隙与子 inline 穴，
                        // 祖先累计止于 IFC owner 盒）。多列路径**不建表**——键
                        // (owner, line_top) 跨列碰撞会把不同列的行域焊成横跨列缝的
                        // 一条带（multicol-basic-001..004 实证 2.33%），回退词 run 域
                        //（列内连续域留待键扩列索引再启）。
                        let r5297_band_extents = if multicol_info.is_some() {
                            std::collections::HashMap::new()
                        } else {
                            owner_line_band_extent_map(
                                fragments.iter().enumerate().map(|(gi, f)| {
                                    (f.node_id, frag_line_tops.get(gi).copied().unwrap_or(0.0), f.x, f.width)
                                }),
                                doc,
                                r5297_walk_stop,
                            )
                        };
                        // R5299：单行整盒路径的裂解段表（多列不建——键 (owner,
                        // line_top) 跨列碰撞，同行连续域同病；旗标关 = 空表回落）。
                        let r5299_seg_extents = if single_frag_box_border_enabled() && multicol_info.is_none() {
                            owner_line_segment_extent_map(
                                fragments.iter().enumerate().map(|(gi, f)| {
                                    (f.node_id, frag_line_tops.get(gi).copied().unwrap_or(0.0), f.x, f.width)
                                }),
                                doc,
                                r5297_walk_stop,
                            )
                        } else {
                            HashMap::new()
                        };
                        for (frag_gi, fragment) in fragments.iter().enumerate() {
                            // IFC 片段（空 styles）：frag.y 基于 16px 默认值，
                            // 使用存储的 font_size（来自 layout IFC）计算基线偏移。
                            // 如果无存储值，回退到 16px 默认值（保持原有行为）。
                            let stored_fs = box_node.text_node_font_sizes.get(&fragment.node_id).copied();
                            let baseline_fs = stored_fs.unwrap_or(fragment.font_size);
                            // GlyphPrimitive.y is a baseline coordinate. The IFC fragment y is the
                            // run top (`baseline_y - run.height`), so text adds the full fragment
                            // height. Subtracting an estimated ascent here passes the em-box top as a
                            // baseline and shifts every rasterized glyph upward by about 0.8em.
                            let baseline_offset = if fragment.font_size > 0.0 {
                                paint_ifc_baseline_offset(fragment.height)
                            } else {
                                baseline_fs
                            };
                            render_fragment!(
                                fragment.x,
                                fragment.y,
                                frag_line_tops.get(frag_gi).copied().unwrap_or(0.0),
                                r4332_edges[frag_gi],
                                fragment.width,
                                baseline_offset,
                                stored_fs.unwrap_or(fragment.font_size),
                                fragment.text,
                                fragment.node_id,
                                fragment.is_ahem,
                                fragment.source.as_ref(),
                                fragment.letter_spacing,
                                sel_line_colors
                                    .get(&(
                                        if doc
                                            .get(fragment.node_id)
                                            .is_some_and(|n| matches!(n.kind, NodeKind::Text(_)))
                                        {
                                            doc.parent_node(fragment.node_id).unwrap_or(fragment.node_id)
                                        } else {
                                            fragment.node_id
                                        },
                                        frag_line_tops.get(frag_gi).copied().unwrap_or(0.0).to_bits()
                                    ))
                                    .copied(),
                                &r5297_band_extents,
                                &r5299_seg_extents
                            );
                        }
                    }
                } // end non-multicol else block

                // R4330/R4332：run-in 分裂边框描绘（Path B）——前置内容按 css-break 3 §5.2
                // slice 语义逐片段描绘：每个片段顶/底横条（行盒顶锚，R1442 同款几何）、
                // 首片段左竖边、末片段右竖边——与 ref 页等价形态（多行 inline span）的
                // per-fragment 描绘同一几何，test/ref 双页像素收敛（run-in-breaking-001/002）。
                // 片段识别 = node_id 的父为 run-in 源元素。
                if !use_stored
                    && !is_vertical
                    && let Some(rb) = &box_node.run_in_border
                    && let Some(run_in_id) = box_node.run_in_prepended
                {
                    let bc = zero_render_foundation::color::Color {
                        r: (rb.color >> 24) as u8,
                        g: (rb.color >> 16) as u8,
                        b: (rb.color >> 8) as u8,
                        a: rb.color as u8,
                    };
                    let ri_indices: Vec<usize> = fragments
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| doc.parent_node(f.node_id) == Some(run_in_id))
                        .map(|(gi, _)| gi)
                        .collect();
                    let ri_total = ri_indices.len();
                    for (pos, gi) in ri_indices.iter().enumerate() {
                        let f = &fragments[*gi];
                        let fx = content_x + f.x;
                        let line_h = box_node
                            .text_node_line_heights
                            .get(&f.node_id)
                            .copied()
                            .unwrap_or(f.font_size * 1.164);
                        // 行盒顶锚（R4332 同款）：content_y + line.y + ty（frag_line_tops
                        // 与 fragments 平行，Path B 非 stored 恒已构建）。
                        let line_top = content_y + frag_line_tops.get(*gi).copied().unwrap_or(0.0) + ty;
                        let is_first = pos == 0;
                        let is_last = pos + 1 == ri_total;
                        let lead_ext = if is_first { rb.left } else { 0.0 };
                        let trail_ext = if is_last { rb.right } else { 0.0 };
                        // 顶/底横条（含角：首片段左伸 rb.left、末片段右伸 rb.right；无 pad）。
                        if rb.top > 0.0 {
                            self.primitives.add_fill(
                                Rect::new(fx - lead_ext, line_top - rb.top, f.width + lead_ext + trail_ext, rb.top),
                                bc,
                            );
                        }
                        if rb.bottom > 0.0 {
                            self.primitives.add_fill(
                                Rect::new(
                                    fx - lead_ext,
                                    line_top + line_h,
                                    f.width + lead_ext + trail_ext,
                                    rb.bottom,
                                ),
                                bc,
                            );
                        }
                        // 竖边：左仅首片段、右仅末片段；纵跨 = 行盒高 + 顶/底边框。
                        if rb.left > 0.0 && is_first {
                            self.primitives.add_fill(
                                Rect::new(fx - rb.left, line_top - rb.top, rb.left, line_h + rb.top + rb.bottom),
                                bc,
                            );
                        }
                        if rb.right > 0.0 && is_last {
                            self.primitives.add_fill(
                                Rect::new(fx + f.width, line_top - rb.top, rb.right, line_h + rb.top + rb.bottom),
                                bc,
                            );
                        }
                    }
                }

                // text-overflow: ellipsis 后处理
                if needs_ellipsis && container_width > 0.0 {
                    let content_right = content_x + container_width + tx;

                    let glyphs = &mut self.primitives.glyphs;
                    let fragment_glyphs = &mut glyphs[glyphs_before_fragments..];

                    let mut last_visible_idx: Option<usize> = None;
                    let mut has_overflow = false;

                    for (i, g) in fragment_glyphs.iter().enumerate() {
                        if g.font_size == 0.0 {
                            continue;
                        }
                        if g.x >= content_right {
                            has_overflow = true;
                            last_visible_idx = if i > 0 { Some(i - 1) } else { None };
                            break;
                        }
                        last_visible_idx = Some(i);
                    }

                    if has_overflow {
                        // R4116（css-overflow-3 §text-overflow）：省略号 = **单个 U+2026
                        // 字形**（chromium 同——Ahem '…' 为全盒方块占 1 个 advance；旧实现
                        // 画 3 个 '.'，合成全盒时代恰似 3 个字符位，R4115 真实轮廓下
                        // '.'/'…' 均为全盒方块，3 点 = 3 个方块 300px 覆盖 vs chromium
                        // 单 '…' 100px——text-overflow-ellipsis-001/rtl 揭示）。
                        let ellipsis_char_width =
                            crate::measure_char_for_font(default_font_id.0, '…', font_size, container_is_ahem);
                        let total_ellipsis_width = ellipsis_char_width;
                        let ellipsis_end_x = content_right;
                        let ellipsis_start_x = ellipsis_end_x - total_ellipsis_width;

                        let cutoff_start = if let Some(idx) = last_visible_idx {
                            let mut cut = idx + 1;
                            for j in (0..=idx).rev() {
                                if fragment_glyphs[j].x < ellipsis_start_x && fragment_glyphs[j].font_size > 0.0 {
                                    cut = j + 1;
                                    break;
                                }
                                cut = j;
                            }
                            cut
                        } else {
                            0
                        };

                        for g in fragment_glyphs.iter_mut().skip(cutoff_start) {
                            g.glyph_id = 0;
                            g.font_size = 0.0;
                        }

                        let first_glyph = fragment_glyphs.iter().find(|g| g.font_size > 0.0);
                        let base_y = first_glyph.map(|g| g.y).unwrap_or(content_y + font_size + ty);

                        for (i, ch) in ['…'].iter().enumerate() {
                            self.primitives.add_glyph(GlyphPrimitive {
                                x: ellipsis_start_x + ellipsis_char_width * i as f32 + letter_spacing * i as f32,
                                y: base_y,
                                font_size,
                                color,
                                glyph_id: *ch as u32,
                                font_glyph_index: None,
                                source: None,
                                font_id: default_font_id,
                                font_variation_id: default_font_variation_id,
                                bitmap_width: None,
                                bitmap_height: None,
                                rotation: 0.0,
                                synthetic_italic: false,
                            });
                        }
                    }
                }

                // CSS line-clamp 后处理：限制可见行数并在截断处添加省略号
                // R3768：跨块盒 clamp 中间子（line_clamp_cap Some(n)）——容器预算在该子
                // 用尽，行数 cap = 余量 n（paint 自身 style 无 line-clamp，max_lines
                // 为 None，须从此标志取）。
                let max = max_lines.or(box_node.line_clamp_cap.map(|n| n as u32));
                // R3770b：本盒 paint 期可能零 glyph（文本经父 IFC/owned-inline 路径绘制，
                // 本盒自身无 fragment——R3770b ellipsis host 标记只保证布局侧行数，不保证
                // 本盒 paint 期有 glyph）。line_ys 空 = 无末行可附省略号，整块跳过
                //（旧实现 line_ys[last_idx] 直接索引空 vec panic，全量 corpus 崩溃中止）。
                if let Some(max) = max.filter(|_| {
                    let glyphs = &self.primitives.glyphs;
                    glyphs[glyphs_before_fragments..].iter().any(|g| g.font_size > 0.0)
                }) {
                    let glyphs = &self.primitives.glyphs;
                    let fragment_glyphs = &glyphs[glyphs_before_fragments..];

                    // 收集唯一的行 Y 坐标（用于计算总行数）
                    // R4417：Path B（非 stored）按 IFC 行盒分带归并行 y——glyph-y 去重会
                    // 把 under rt 注音字形行误计为独立行（line-clamp-auto-with-ruby-003：
                    // cutoff 落在 base 行，"Line 5" 全行被零掉、仅剩 rt + 省略号）。带代表
                    // y = 带内首个可见字形 y（与 glyph-y 域一致，末行 ellipsis 定位/裁让
                    // 逻辑不变）。带映射为空（glyph 全在带外：abspos/valign 偏移等）回退
                    // glyph-y 去重；stored 路径 inline_ctx 空，恒回退（原行为）。
                    let glyph_line_ys = || {
                        let mut ys: Vec<f32> = fragment_glyphs
                            .iter()
                            .filter(|g| g.font_size > 0.0)
                            .map(|g| g.y)
                            .collect();
                        ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                        ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);
                        ys
                    };
                    let mut line_ys: Vec<f32> = if !use_stored && !inline_ctx.lines.is_empty() {
                        let banded: Vec<f32> = inline_ctx
                            .lines
                            .iter()
                            .filter_map(|l| {
                                // R4418：带代表 y 取带内 **最大** glyph y——under rt 的注音
                                // 字形（over 近似绘于 base 上方）在 primitives 数组里先于
                                // base 文本时 .next() 会选中注音行，clamp 省略号落到注音行
                                //（auto-with-ruby-003：省略号在 'x' 旁而非 "Line 5" 后）。
                                // max-y = base 文本行（与 ellipsis 末行文本尾定位语义一致；
                                // paint under 位真实现后 rt 在 base 下方，届时改取 base 行）。
                                // R5205：带窗口按 **绝对** glyph y 比较——重跑 IFC 的行盒 y
                                // 是内容盒局部坐标（0 起），fragments 已加盒 content 原点
                                //（border+padding [+valign]，content_y）。不锚定原点时带整体
                                // 偏移 δ：δ > 0.5 的盒（页面上方有 margin/border/padding 的
                                // 常见形态）末可见行落出末带窗口，line_ys 少一行 → ellipsis
                                // 附到 cap−1 行（line-clamp-022/023/024/025 serif 簇）。
                                fragment_glyphs
                                    .iter()
                                    .filter(|g| {
                                        g.font_size > 0.0
                                            && g.y >= content_y + l.y - 0.5
                                            && g.y <= content_y + l.y + l.height + 0.5
                                    })
                                    .map(|g| g.y)
                                    .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                            })
                            .collect();
                        if banded.is_empty() { glyph_line_ys() } else { banded }
                    } else {
                        glyph_line_ys()
                    };
                    line_ys.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    line_ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);

                    // R2467 line-clamp slice 2：触发条件双源。
                    // - `exceeded`：paint 看到的行数 > max（non-stored 路径：paint IFC 用空 styles
                    //   重跑不 cap → 全量行）。
                    // - `box_node.line_clamp_clamped`：layout 期 IFC `apply_line_clamp_cap` 真截断
                    //   （stored 路径：pure-Ahem 容器 inline_layout 已被 cap 到 max 行，paint 看到
                    //   行数 == max → exceeded=false → 须读此标志才能补 ellipsis）。
                    // max==0：CSS `line-clamp: 0` 视同 none（parser 拒绝，resolve_line_clamp
                    // 不会返回 Some(0)）；R3768 跨块盒的 line_clamp_cap 可为 0（clamp 点
                    // 落在该子盒首行前 → 0 行可见），此时须截全部 glyph，0 视为合法 cap。
                    let max_usize = max as usize;
                    let exceeded = line_ys.len() > max_usize;
                    let clamped = (max >= 1 || box_node.line_clamp_cap.is_some()) && box_node.line_clamp_clamped;
                    if exceeded || clamped {
                        // 截断：移除第 max 行之后的所有 glyph。stored 路径已被 layout cap → 无超出行 →
                        // 此处 exceeded=false 跳过；non-stored 路径 exceeded=true 主动截。
                        if exceeded {
                            let cutoff_y = line_ys[max as usize];
                            let glyphs = &mut self.primitives.glyphs;
                            for g in glyphs[glyphs_before_fragments..].iter_mut() {
                                if g.y >= cutoff_y - 0.5 {
                                    g.font_size = 0.0;
                                    g.glyph_id = 0;
                                }
                            }
                        }

                        // 在最后一可见行（第 max 行）末尾渲 U+2026 ellipsis（与 WPT line-clamp refs
                        // 一致：单字符 `…`，非 3 个 ASCII '.'）。`max.min(line_ys.len())` 防越界
                        //（stored 路径行数 == max）。max=0（跨块 cap，0 行可见）无末行 →
                        // 不渲 ellipsis。
                        let last_idx = (max as usize).min(line_ys.len()).saturating_sub(1);
                        let last_line_y = line_ys[last_idx];
                        let ellipsis_enabled = max >= 1;

                        // 末行最右可见 glyph 及其字符 advance（求末行文本 end x）。
                        let last_glyph = self.primitives.glyphs[glyphs_before_fragments..]
                            .iter()
                            .filter(|g| g.font_size > 0.0 && (g.y - last_line_y).abs() < 0.5)
                            .max_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
                        let last_adv = last_glyph
                            .and_then(|g| char::from_u32(g.glyph_id))
                            .map(|c| self.measure_char_cached(default_font_id.0, c, font_size, container_is_ahem))
                            .unwrap_or(0.0);
                        let last_text_end_x = last_glyph.map(|g| g.x + last_adv).unwrap_or(content_x + tx);

                        let default_font_id = self.resolve_style_font_id(&style.font_family, style).0;
                        let ellipsis_char = '\u{2026}';
                        let ellipsis_width =
                            self.measure_char_cached(default_font_id.0, ellipsis_char, font_size, container_is_ahem);
                        let content_right = content_x + container_width + tx;

                        // 定位：紧跟末行文本末尾；若 + ellipsis 宽超 content_right（末行已占满），
                        // 回退到 content_right 右对齐 + 截掉末行尾部 glyph 让位（镜像 text-overflow
                        // cutoff，text.rs:1464-1481）。
                        let ellipsis_x = if last_text_end_x + ellipsis_width <= content_right + 0.5 {
                            last_text_end_x
                        } else {
                            let cut_x = content_right - ellipsis_width;
                            let glyphs = &mut self.primitives.glyphs;
                            for g in glyphs[glyphs_before_fragments..].iter_mut() {
                                if g.font_size > 0.0 && (g.y - last_line_y).abs() < 0.5 && g.x + 0.5 >= cut_x {
                                    g.font_size = 0.0;
                                    g.glyph_id = 0;
                                }
                            }
                            cut_x
                        };

                        if ellipsis_enabled {
                            self.primitives.add_glyph(GlyphPrimitive {
                                x: ellipsis_x,
                                y: last_line_y,
                                font_size,
                                color,
                                glyph_id: ellipsis_char as u32,
                                font_glyph_index: None,
                                source: None,
                                font_id: default_font_id,
                                font_variation_id: default_font_variation_id,
                                bitmap_width: None,
                                bitmap_height: None,
                                rotation: 0.0,
                                synthetic_italic: false,
                            });
                        }
                    }
                }

                return;
            }
        }

        // 退化为单个占位 glyph
        let glyph_x = content_x + tx;
        let glyph_y = content_y + ty;

        for &(shadow_ox, shadow_oy, shadow_color) in &active_text_shadows {
            self.primitives.add_glyph(GlyphPrimitive {
                x: glyph_x + shadow_ox,
                y: glyph_y + font_size + shadow_oy,
                font_size,
                color: shadow_color,
                glyph_id: 0,
                font_glyph_index: None,
                source: None,
                font_id: default_font_id,
                font_variation_id: default_font_variation_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
                synthetic_italic: false,
            });
        }

        self.primitives.add_glyph(GlyphPrimitive {
            x: glyph_x,
            y: glyph_y + font_size,
            font_size,
            color,
            glyph_id: 0,
            font_glyph_index: None,
            source: None,
            font_id: default_font_id,
            font_variation_id: default_font_variation_id,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });

        self.paint_text_decoration_from_style(
            glyph_x,
            glyph_y + font_size,
            font_size,
            self.measure_char_cached(default_font_id.0, 'A', font_size, false),
            color,
            style,
            0.0,
            0.0,
        );
    }

    /// 绘制匿名文本项（flex/grid 容器中的文本节点）。
    ///
    /// 与 paint_text 不同，此方法直接渲染 node_id 指向的文本节点内容，
    /// 而非查找子文本节点。匿名文本项没有独立的 ComputedStyle，
    /// 使用父元素的样式。
    pub fn paint_anonymous_text_item(
        &mut self,
        _box_node: &LayoutBox,
        abs_x: f32,
        abs_y: f32,
        style: &ComputedStyle,
        doc: &Document,
        node_id: NodeId,
        styles: Option<&HashMap<NodeId, ComputedStyle>>,
    ) {
        let font_size: f32 = match style.font_size {
            LengthValue::Px(s) => s as f32,
            _ => return,
        };
        if font_size <= 0.0 || style.color == ColorValue::CurrentColor {
            return;
        }

        // R4525/R4549：clip:text 恒色彩字上下文——flex/grid 匿名文本项不经
        // paint_node_inner（恒色 push 站在此路径之外），按 owner 容器自身 / DOM 祖先链
        // 静态判定补链（与 paint_text 主路径同规）：字形色 = text 色 OVER bg 恒色
        //（text 色透明时 = bg 恒色，clip-text-flex 判例）。
        let color = match bg_clip_text_solid_color(style, &self.image_solid_colors, self.document_url.as_deref())
            .or_else(|| {
                styles.and_then(|s| {
                    bg_clip_text_solid_color_ancestral(
                        doc,
                        node_id,
                        s,
                        &self.image_solid_colors,
                        self.document_url.as_deref(),
                    )
                })
            }) {
            Some(clip_bg) => color_over(color_value_to_render(&style.color), clip_bg),
            None => color_value_to_render(&style.color),
        };
        let default_font_id = self.resolve_style_font_id(&style.font_family, style).0;
        let variations = crate::text_metrics::paint_font_variations(&style.font_variation_settings);
        let font_variation_id = self.primitives.intern_font_variations(&variations);
        let content_x = abs_x;
        let content_y = abs_y;

        // 直接从文本节点获取内容
        let text = match doc.get(node_id) {
            Some(node) => match &node.kind {
                NodeKind::Text(data) => data.content.trim().to_string(),
                _ => return,
            },
            None => return,
        };
        if text.is_empty() {
            return;
        }

        // 渲染文本字符为 glyph primitives
        let is_ahem = style
            .font_family
            .iter()
            .any(|f| f.trim_matches('"').eq_ignore_ascii_case("Ahem"));
        let mut char_x = content_x;
        for ch in text.chars() {
            self.primitives.add_glyph(GlyphPrimitive {
                x: char_x,
                y: content_y + font_size,
                font_size,
                color,
                glyph_id: ch as u32,
                font_glyph_index: None,
                source: None,
                font_id: default_font_id,
                font_variation_id,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
                synthetic_italic: false,
            });
            char_x += self.measure_char_cached(default_font_id.0, ch, font_size, is_ahem);
        }
    }
}

/// R3895：片段渲染后沿 inline 包装链下探补标 painted_inline_nodes。
///
/// layout IFC 对嵌套 inline 元素做文本扁平化（collect_inline_items 的
/// `text_content(child_id)` 分支把子树文本合成到外层元素 id 的单个 TextRun），
/// 内层 inline 包装元素自身永远不作为片段 node_id 出现，也就不会被
/// painted_inline_nodes 标记。其自身 paint_text 的 dedup 查不到 id → 放行 →
/// 重跑无度量覆盖的 Path B IFC（text_node_font_sizes 只存了外层键）→
/// 以默认 16px 在错误位置双绘文本（quotes-028/029/031 每层嵌套 `<q>` 双绘引号）。
///
/// CSS2 §9.4.1：嵌套 inline 盒的文本由最近 IFC 一次性排版渲染，包装层不再自绘。
/// 片段渲染后把 `$frag_nid` 元素之下的**纯 inline 元素后代链**（不跨 block/文本边界，
/// 不含 atomic inline-block 等自绘盒）一并标记，使这些包装层的 paint_text 被 dedup
/// 早退。styles 缺失时不下探（无法判定 inline 边界，保守回退旧行为）。
pub(super) fn mark_inline_wrapper_chain_painted(
    painted: &mut std::collections::HashSet<NodeId>,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    root: NodeId,
) {
    let is_inline_display = |id: NodeId| -> bool {
        styles
            .get(&id)
            .is_some_and(|s| matches!(s.display, DisplayValue::Inline))
    };
    let mut stack: Vec<NodeId> = doc.child_nodes(root);
    while let Some(id) = stack.pop() {
        // 仅沿 inline 元素下探；文本/非 inline（block、inline-block 等自绘盒）停止。
        if !is_inline_display(id) {
            continue;
        }
        painted.insert(id);
        stack.extend(doc.child_nodes(id));
    }
}

/// R5177：color 差异拆分归因片段的祖先载体补标——拆分归因后片段 nid 是内层元素
/// 自身，外层 inline 载体（has_direct_paintable_text 的 PHASEA_STORE_EXT 臂放行
/// paint_text）的 Path B 重跑会重复渲染已由 IFC 片段覆盖的子树文本；旧折回归因下
/// 外层 nid 自身被 insert 恰好覆盖此面。对每个拆分归因 owner 沿祖先链上溯
/// display:Inline 元素 insert，dedup 拦住外层重跑（抑制面与旧折回行为等价）。
/// 调用方先按 divergent 集过滤 owner；渲染循环前一次调用，非逐片段。
pub(super) fn mark_split_owner_ancestor_chain_painted(
    painted: &mut std::collections::HashSet<NodeId>,
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    owners: impl Iterator<Item = NodeId>,
    divergent: &zero_layout_engine::NodeIdSet,
) {
    for owner in owners.filter(|id| divergent.contains(id)) {
        let mut ancestor = doc.parent_node(owner);
        while let Some(id) = ancestor {
            if !styles
                .get(&id)
                .is_some_and(|s| matches!(s.display, DisplayValue::Inline))
            {
                break;
            }
            painted.insert(id);
            ancestor = doc.parent_node(id);
        }
    }
}

/// 收集 `box_node` 对应 DOM 子树内 display:none 的元素节点集——paint Path B（空
/// styles 重跑 IFC）下隐藏子门的判定信号。CSS2 §9.3 / CSS Display 3 §2.1：display:none
/// 子树不生成任何盒。须沿 **DOM 子树**遍历（盒树里 inline none 元素无独立 LayoutBox，
/// 盒遍历会漏）；none 节点整棵剪枝（collect 的 continue 本就跳过整棵）。layout 趟有
/// styles 直判，不消费此集。
fn collect_display_none_walk_nodes(
    box_node: &LayoutBox,
    doc: &Document,
    styles: Option<&HashMap<NodeId, ComputedStyle>>,
) -> NodeIdSet {
    let (Some(root), Some(styles)) = (box_node.node_id, styles) else {
        return NodeIdSet::default();
    };
    let mut out = NodeIdSet::default();
    let mut stack: Vec<NodeId> = doc.child_nodes(root);
    while let Some(id) = stack.pop() {
        if styles.get(&id).is_some_and(|s| matches!(s.display, DisplayValue::None)) {
            out.insert(id);
            continue; // 子树剪枝：hidden 后代无需入集
        }
        stack.extend(doc.child_nodes(id));
    }
    out
}

/// R5102（CSS2 §9.7）：out-of-flow 后代集——容器 DOM 子树 ∪ run-in 元素 DOM 子树中
/// position:absolute/fixed 的元素（扁平化 walk / text 收集在 Path B 空 styles 下的
/// 剪枝真值源，镜像 [`collect_display_none_walk_nodes`] 双通道模式）。run-in 前缀子树
/// 不在容器 DOM 子树内（run-in 是容器的兄弟元素），须单独并入。vertical 模式返回空集
///（镜像 layout 期 OOF skip 臂的 vertical 例外：vertical-rl 的 abspos shrink-to-fit
/// 依赖 IFC 内测量，不剪）。
fn collect_oof_walk_nodes(
    box_node: &LayoutBox,
    doc: &Document,
    styles: Option<&HashMap<NodeId, ComputedStyle>>,
    vertical: bool,
) -> NodeIdSet {
    if vertical {
        return NodeIdSet::default();
    }
    let Some(styles) = styles else {
        return NodeIdSet::default();
    };
    let mut out = NodeIdSet::default();
    let mut stack: Vec<NodeId> = Vec::new();
    if let Some(root) = box_node.node_id {
        stack.extend(doc.child_nodes(root));
    }
    // run-in 前缀子树（build_subtree 注册表：本容器并入的 run-in 源元素）。
    if let Some(run_in_id) = box_node.run_in_prepended {
        stack.extend(doc.child_nodes(run_in_id));
    }
    while let Some(id) = stack.pop() {
        if styles.get(&id).is_some_and(|s| {
            matches!(
                s.position,
                zero_css_parser::values::PositionValue::Absolute | zero_css_parser::values::PositionValue::Fixed
            )
        }) {
            out.insert(id);
            continue; // 子树剪枝：OOF 后代自身即剪枝单元
        }
        stack.extend(doc.child_nodes(id));
    }
    out
}

/// R4332（css-break 3 §5.2 / CSS2.1 §8.5.3）：按 inline owner 计算各片段的（首, 末）旗标。
///
/// owner 解析与 render_fragment 宏同源（文本节点取其父元素，元素片段取自身），
/// 返回 Vec 与片段序列平行。跨行 inline 盒的左竖边仅画在首片段、右竖边仅画在
/// 末片段；macro_rules! 卫生宏体不可见调用点局部变量，故在调用点预计算后作为
/// 宏参数传入。
/// R5297：行片段连续 x 域表（(节点, line_top) → (min_x, max_end)）——run 归属
/// owner（文本节点取父元素）并沿祖先链向**每个**中间 inline 祖先累计，止于
/// `stop_at`（IFC owner 盒，其背景走盒级路径）。词级 run 之间的空隙、子 inline
/// 穴域计入祖先的行片段域（css-break 3 §5.2 slice：行片段 = 盒在该行的连续
/// 跨度；chromium 横条/背景贴父 fragment 全域，c42 两页实证：父 bg/边框贯穿
/// 子 inline 区段，逐 run 发射留白）。查询缺失时调用点退自身 run 域。
fn owner_line_band_extent_map(
    frags: impl Iterator<Item = (NodeId, f32, f32, f32)>,
    doc: &Document,
    stop_at: Option<NodeId>,
) -> HashMap<(NodeId, u32), (f32, f32)> {
    let mut extents: HashMap<(NodeId, u32), (f32, f32)> = HashMap::new();
    for (nid, line_top, x, width) in frags {
        let lt = line_top.to_bits();
        let mut cur = if doc.get(nid).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))) {
            doc.parent_node(nid).unwrap_or(nid)
        } else {
            nid
        };
        loop {
            if Some(cur) == stop_at {
                break;
            }
            let e = extents.entry((cur, lt)).or_insert((x, x + width));
            e.0 = e.0.min(x);
            e.1 = e.1.max(x + width);
            match doc.parent_node(cur) {
                Some(p) => cur = p,
                None => break,
            }
        }
    }
    extents
}

/// R5299（css-writing-modes-3 #bidi-box-model / CSS2.1 box.html #bidi-box-model）：
/// 单行 owner 的行连续域在**盒裂解缝**处分段。bidi 重排可把 owner 的两个视觉
/// 片段拉开、其间楔入外来内容（bidi-001：RLO 把 `<p>` 直辖文本搬进 span 裂解缝，
/// 行连续域横条/bg 跨缝盖住缝内文字）。分段判据：片段间隙内楔入**非子孙**
/// owner 的片段即视作裂解边界切段；缝内仅有子孙片段（子 inline 穴，祖先带须
/// 连续覆盖）或空缝（词 run 间隙）保持连续。祖先自身楔入（bidi-001 的 `<p>`
/// 文本）同样切段——祖先带连续性由其自身表项另行维护（其视角缝内片段多为
/// 子孙，不切）。
///
/// 仅单行整盒路径（`ZW_INLINE_SINGLE_FRAG_BOX_BORDER` 开启）构建消费；多行
/// slice 路径维持 [owner_line_band_extent_map] 行连续域口径不变。段值附
/// (has_first, has_last)——该段是否含 owner 逻辑首/末片段（css-writing-modes-3
/// #bidi-box-model：start 边装饰随逻辑首片段、end 边随逻辑末片段；bidi 重排下
/// 逻辑首未必是视觉最左，边归属须按逻辑序而非 x 序）。
pub(crate) fn owner_line_segment_extent_map(
    frags: impl Iterator<Item = (NodeId, f32, f32, f32)>,
    doc: &Document,
    stop_at: Option<NodeId>,
) -> HashMap<(NodeId, u32), Vec<(f32, f32, bool, bool)>> {
    const R5299_EPS: f32 = 0.5;
    // 先收集：per (owner, line) 祖先累计的无序片段区间 + per line 全量片段
    //（含 owner，供缝内楔入检测）。插入序 = 迭代序 = 逻辑序（首/末标记依据）。
    let mut per_owner: HashMap<(NodeId, u32), Vec<(f32, f32)>> = HashMap::new();
    let mut per_line: HashMap<u32, Vec<(f32, f32, NodeId)>> = HashMap::new();
    for (nid, line_top, x, width) in frags {
        let lt = line_top.to_bits();
        let owner = if doc.get(nid).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))) {
            doc.parent_node(nid).unwrap_or(nid)
        } else {
            nid
        };
        per_line.entry(lt).or_default().push((x, x + width, owner));
        let mut cur = owner;
        loop {
            if Some(cur) == stop_at {
                break;
            }
            per_owner.entry((cur, lt)).or_default().push((x, x + width));
            match doc.parent_node(cur) {
                Some(p) => cur = p,
                None => break,
            }
        }
    }
    // 逐 owner 分段：x 序扫描，缝（> EPS）内楔入非子孙片段即切段。
    per_owner
        .into_iter()
        .map(|((owner, lt), mut ivs)| {
            // 逻辑首/末片段区间（排序前取——迭代序即 owner 片段逻辑序，
            // 与 owner_fragment_edge_flags 的 idx==0 / idx+1==count 同源）。
            let r5299_first_iv = ivs[0];
            let r5299_last_iv = *ivs.last().unwrap_or(&r5299_first_iv);
            ivs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            let line_all = per_line.get(&lt).map(Vec::as_slice).unwrap_or(&[]);
            let mut segs: Vec<(f32, f32)> = Vec::with_capacity(ivs.len());
            for &(s, e) in &ivs {
                match segs.last_mut() {
                    // 重叠/相接：无条件并入。
                    Some(last) if s <= last.1 + R5299_EPS => last.1 = last.1.max(e),
                    Some(last) => {
                        let foreign = line_all.iter().any(|&(fx0, fx1, fown)| {
                            fown != owner
                                && !r5299_is_owner_descendant(doc, fown, owner)
                                && fx0 < s - R5299_EPS
                                && fx1 > last.1 + R5299_EPS
                        });
                        if foreign {
                            segs.push((s, e));
                        } else {
                            last.1 = last.1.max(e);
                        }
                    }
                    None => segs.push((s, e)),
                }
            }
            // 段标记：逻辑首/末片段区间落入哪段（含判据与消费侧查找同 EPS）。
            let in_seg = |iv: (f32, f32)| -> Option<usize> {
                segs.iter()
                    .position(|&(s, e)| s - R5299_EPS <= iv.0 && iv.1 <= e + R5299_EPS)
            };
            let first_seg = in_seg(r5299_first_iv);
            let last_seg = in_seg(r5299_last_iv);
            (
                (owner, lt),
                segs.into_iter()
                    .enumerate()
                    .map(|(si, (s, e))| (s, e, first_seg == Some(si), last_seg == Some(si)))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

/// `node` 是否 `anc` 的子孙或自身（owner 链上溯；bidi 裂解缝「非子孙楔入」判据）。
pub(crate) fn r5299_is_owner_descendant(doc: &Document, node: NodeId, anc: NodeId) -> bool {
    if node == anc {
        return true;
    }
    let mut cur = node;
    while let Some(p) = doc.parent_node(cur) {
        if p == anc {
            return true;
        }
        cur = p;
    }
    false
}

fn owner_fragment_edge_flags(frags: impl Iterator<Item = (NodeId, f32)>, doc: &Document) -> Vec<(bool, bool, bool)> {
    let owners: Vec<(NodeId, f32)> = frags
        .map(|(nid, line_top)| {
            let owner = if doc.get(nid).is_some_and(|n| matches!(n.kind, NodeKind::Text(_))) {
                doc.parent_node(nid).unwrap_or(nid)
            } else {
                nid
            };
            (owner, line_top)
        })
        .collect();
    let mut counts: HashMap<NodeId, usize> = HashMap::new();
    let mut lo: HashMap<NodeId, f32> = HashMap::new();
    let mut hi: HashMap<NodeId, f32> = HashMap::new();
    for (o, lt) in &owners {
        *counts.entry(*o).or_insert(0) += 1;
        let e = lo.entry(*o).or_insert(*lt);
        *e = e.min(*lt);
        let e = hi.entry(*o).or_insert(*lt);
        *e = e.max(*lt);
    }
    let mut seen: HashMap<NodeId, usize> = HashMap::new();
    owners
        .iter()
        .map(|(o, _)| {
            let idx = seen.entry(*o).and_modify(|c| *c += 1).or_insert(0);
            (
                *idx == 0,
                *idx + 1 == counts[o],
                // R4332：跨行判据 = owner 片段线顶跨 ≥2 行（词级多片段单行不算，
                // split_into_words 每词一片段）。
                hi[o] - lo[o] > 0.5,
            )
        })
        .collect()
}

/// 将数字转换为罗马数字字符串（1-based）。
/// R5088：仅判**直接文本子**（R5055 preserved-content 口径）——不含
/// `has_direct_paintable_text` 的 PHASEA「inline 元素子视同可绘」扩展。box-level
/// 背景抑制门专用：抑制的前提是「本 inline 自身是 fragment owner（有直接文本），
/// 背景由 R639 per-fragment 臂施加」；无直接文本的 wrapper inline
///（`<span style=bg><b>text</b></span>`）不是 fragment owner，per-fragment 臂永不
/// 为其发背景，box-level bg 是唯一施画者——旧判据经元素子扩展误抑制 → wrapper
/// 背景整丢（多行内容器 probe-a）或后置覆盖后代字形（probe-b）。
pub(super) fn has_direct_text_child(doc: &Document, node_id: NodeId) -> bool {
    doc.child_nodes(node_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            Some(NodeKind::Text(text)) if zero_layout_engine::has_preserved_content(&text.content)
        )
    })
}

pub(super) fn has_direct_paintable_text(
    doc: &Document,
    node_id: NodeId,
    styles: Option<&HashMap<NodeId, ComputedStyle>>,
) -> bool {
    if let Some(styles) = styles
        && styles.get(&node_id).is_some_and(|style| {
            matches!(
                style.display,
                DisplayValue::Flex | DisplayValue::InlineFlex | DisplayValue::Grid | DisplayValue::InlineGrid
            )
        })
    {
        // CSS Flexbox §4 / CSS Grid §4: flex/grid children are items, not text
        // in the container's inline formatting context. Direct text is painted
        // through anonymous item boxes; element children paint through their own boxes.
        return false;
    }

    let direct = doc.child_nodes(node_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            // R5055：可绘制判据 = 含 white-space 处理后保留的内容（CSS Text §3 窄集合
            // 口径，`zero_layout_engine::has_preserved_content`）。旧 `trim().is_empty()`
            // 按 Unicode White_Space 剔除 U+00A0 → 仅含 `&nbsp;` 的文本子误判为空 →
            // 元素零渲染（driving: selection-background-color-001 `<div>&nbsp;</div>`）。
            Some(NodeKind::Text(text)) if zero_layout_engine::has_preserved_content(&text.content)
        )
    });
    if direct {
        return true;
    }
    // PHASEA stored-line-boxes 路径（默认启用；env PHASEA_STORE_EXT=0 关闭，与 compute_final 存储扩展配套）：仅对
    // **纯 inline 内容**容器（有 inline-level 元素子节点且**无 block-level 元素子节点**）返回
    // true。排除 block 子节点（独立渲染）与混合 inline+block 内容（block-in-inline / span+h4
    // 等存储路径与重跑分歧致回归：inline-box-001/002、multicol-block-no-clip-001）。
    if std::env::var("PHASEA_STORE_EXT").as_deref() != Ok("0")
        && let Some(styles) = styles
    {
        use zero_css_parser::values::DisplayValue;
        let is_inline_display = |d: &DisplayValue| {
            matches!(
                d,
                DisplayValue::Inline
                    | DisplayValue::InlineBlock
                    | DisplayValue::InlineFlex
                    | DisplayValue::InlineGrid
                    | DisplayValue::InlineTable
            )
        };
        let child_ids: Vec<zero_dom::NodeId> = doc.child_nodes(node_id);
        let child_displays: Vec<Option<&DisplayValue>> =
            child_ids.iter().map(|c| styles.get(c).map(|s| &s.display)).collect();
        let has_inline_elem = child_displays.iter().any(|d| d.is_some_and(is_inline_display));
        // R1280：float 子元素是 out-of-flow（CSS §9.5），不属于 in-flow block 内容。
        // 含 [inline 文本 + float] 的容器其 inline 内容仍经 IFC 排版并绕 float 流动。
        // 旧实现把 blockified float（display:Block + float≠none）误计为 block-level →
        // 此处返回 false → 容器 paint_text 早退 → inline 元素文本（floats-006 的 <span>X</span>）
        // 经 span 自身 Path B 在非 float-excluded 位渲染。float 子不计为 block，让容器 paint_text
        // 跑（Path A 存储见 inline_finalization，或 Path B collect_float_exclusions），IFC 正确绕
        // float，并经 painted_inline_nodes 自动抑制 inline 子 Path B（避免双绘）。
        // kill-switch `ZW_FLOAT_INLINE_PAINT=0` 回退旧行为（default-on，全 dir A/B net 0 零回归）。
        let float_not_block = std::env::var("ZW_FLOAT_INLINE_PAINT").as_deref() != Ok("0");
        // R4405：display:none 直接子不参与 in-flow 内容（CSS2 §9.3 display:none 不生成盒），
        // 不得计为块级子——否则「全 rb 包裹 base 的 ruby」（rt/rp/rbc 为 display:none 直接子）
        // predicate=false → paint_text 早退 → Path B 不运行 → base 文本零绘制（intra-base
        // REF 页空绘，R4403 定谳）。两臂同修：同一判据在 if/else 两臂的副本必须同步
        //（R4404 域认知：修一处不动另一处 = 假修复）。
        let not_none = |dd: &DisplayValue| !matches!(dd, DisplayValue::None);
        let has_block_elem = if float_not_block {
            child_ids.iter().any(|c| {
                styles.get(c).is_some_and(|s| {
                    !is_inline_display(&s.display)
                        && not_none(&s.display)
                        && matches!(s.float, zero_css_parser::values::FloatValue::None)
                })
            })
        } else {
            child_displays
                .iter()
                .any(|d| d.is_some_and(|dd| !is_inline_display(dd) && not_none(dd)))
        };
        // R207 曾要求 inline-level 子元素为**叶文本容器**（无元素子节点）——过宽守卫把
        // 「inline 子含纯 inline 后代」（嵌套 <q> 独子、<div><span>a <span>b</span> c</span></div>）
        // 也排除，容器无 stored IFC → paint 逐盒渲染 taffy 堆叠盒 → 文本三行堆叠
        //（quotes-005/013/015/019 根因）。R4043 收窄为仅排除三类深路径域：①inline 子后代树
        // 含**块级元素**（R109 block-in-inline 碎片化）；②inline 子树含 `<br>`（扁平化收集
        // 吞嵌套 br，line-break-* control 段 + outline-004 载体）；③inline 子树含与子自身
        // 字体度量不同的后代（outline-022 `#target{font-size:80px}`——扁平化字体错误）。
        // 须与 layout 侧 compute_final_inline_layouts 的守卫同步（两端一致才走 use_stored）。
        // R4392：**ruby 项移除（paint 侧）**——旧排除使「inline-block 含 ruby、无直接文本子」
        // 的容器 paint_text 早退（has_direct_paintable_text=false），Path B 不运行 → ruby
        // base 文本零绘制（ruby-intrinsic-isize-001 行 1 空盒）。R4389 已为 ruby flatten run
        // 填充 font_id（真实 serif hmtx），Path B 收集+断行语义与 layout 同源。layout 侧
        // compute_final 的 ruby 排除保留（ruby 子树仍不产 stored IFC → inline_layout=None →
        // 本侧恒走 Path B，两端 use_stored 语义仍一致）。
        let inline_child_has_deep_path = child_ids.iter().any(|c| {
            styles.get(c).is_some_and(|s| {
                is_inline_display(&s.display)
                    && (subtree_has_block_elem(doc, styles, *c)
                        || InlineFormattingContext::inline_elem_has_nested_br(doc, *c)
                        || subtree_font_differs_from(doc, styles, *c, s))
            })
        });
        has_inline_elem && !has_block_elem && !inline_child_has_deep_path
    } else {
        false
    }
}

fn replace_utf16_range(value: &str, start: usize, end: usize, replacement: &str) -> String {
    fn byte_offset(value: &str, utf16_offset: usize) -> usize {
        let mut units = 0;
        for (byte, ch) in value.char_indices() {
            if units >= utf16_offset {
                return byte;
            }
            units += ch.len_utf16();
        }
        value.len()
    }

    let range_start = start.min(end);
    let range_end = start.max(end);
    let start_byte = byte_offset(value, range_start);
    let end_byte = byte_offset(value, range_end);
    format!("{}{}{}", &value[..start_byte], replacement, &value[end_byte..])
}

/// R2303 object-position 定位测试。
#[cfg(test)]
mod r2303_object_position_tests;
