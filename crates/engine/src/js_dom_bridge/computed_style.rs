//! getComputedStyle 计算与序列化——把 [`zero_style_system::ComputedStyle`] 的单属性序列化为
//! CSS 字符串（kebab-case 属性名）。从 `js_dom_bridge` 拆出（R2709）以控制主文件行数。
//!
//! 覆盖：display/position/visibility/opacity + 颜色族（color/background-color/border-*-color/outline-color/caret-color/accent-color）+ 长度族 + 关键字/枚举族 + font-family/复合族
//! + Transforms 全簇 + contain + filter + will-change + clip-path + content + background 簇（position/size/repeat/attachment/clip/origin）+ Box Alignment 簇（align-items/self、justify-content/items/self、align-content）+ CSS Text 换行/断词（word-break/overflow-wrap/hyphens/line-break）+ vertical-align/unicode-bidi/empty-cells；未覆盖属性返 ''。

use std::collections::HashMap;

use zero_css_parser::values::{
    AlignmentValue, BackgroundEdge, BoxSizingValue, ClearValue, ClipPathRadius, ClipPathValue, ColorValue,
    ContentListItem, DisplayValue, FlexDirectionValue, FlexWrapValue, FloatValue, FontStyleValue, FontWeightValue,
    LengthValue, ListStylePositionValue, ListStyleTypeValue, OverflowValue, PolygonFillRule, PositionValue,
    TransformFunction, TransformValue, VisibilityValue,
};
use zero_dom::{Document, NodeId, parse_html};
use zero_style_system::{
    AccentColorComputedValue, AlignContentValue, BackfaceVisibilityValue, BackgroundAttachmentComputedValue,
    BackgroundClipComputedValue, BackgroundOriginComputedValue, BackgroundPositionComputedValue,
    BackgroundRepeatComputedValue, BackgroundSizeComputedValue, BorderCollapseValue, BorderStyleValue,
    CaptionSideValue, CaretColorComputedValue, ComputedStyle, ContainComputedValue, ContentComputedValue, CursorValue,
    DirectionValue, EmptyCellsComputedValue, FilterComputedValue, FlexBasisValue, HyphensComputedValue, IsolationValue,
    JustifyItemsValue, JustifySelfValue, LineBreakValue, LineHeightValue, MixBlendModeComputedValue,
    ObjectFitComputedValue, OutlineStyleValue, OverflowWrapValue, PointerEventsValue, StyleSystem, TableLayoutValue,
    TextAlignValue, TextOverflowValue, TextTransformValue, TransformStyleValue, UnicodeBidiValue, UserSelectValue,
    VerticalAlignValue, WhiteSpaceValue, WillChangeValue, WordBreakValue, WritingModeValue, ZIndexValue,
};

use super::find_by_selector;

/// `getComputedStyle(el).getPropertyValue(prop)` 的 host 实现：解析 html → 收集 `<style>`（+
/// color-scheme meta）→ 计算样式（UA 默认 builtin via `ua_default_display`，故 `<div>` 等的
/// display 无须外链 UA stylesheet）→ 序列化查询属性。供 `__zw_get_computed_style` 回调 → shim
/// `getComputedStyle`。
///
/// **覆盖范围**：`display`/`position`/`visibility`/`opacity`（visibility/hidden 检查 + position
/// 查询主导用例）+ 颜色族（color/background-color/border-*-color/outline-color）。其余属性返 ''。
///
/// **限制**：外链 `<link>` CSS 不在 dom_html snapshot 内（snapshot 限制，同 gBCR）。**每次调用
/// 重跑 parse+cascade**——作为无缓存的参考实现；生产回调 `__zw_get_computed_style` 用
/// [`compute_document_styles`] + [`lookup_computed_property`] 配 per-snapshot 缓存复用 (doc, styles)。
pub fn computed_style_property(html: &str, selector: &str, prop: &str) -> String {
    let (doc, styles) = compute_document_styles(html);
    lookup_computed_property(&doc, &styles, selector, prop)
}

/// 解析 html 并计算全文档样式，返回 `(Document, NodeId→ComputedStyle)`。供 getComputedStyle
/// 回调的 per-snapshot 缓存：html 未变时复用此结果，避免每次属性查询重跑 parse+cascade
/// （O(文档) + O(规则×节点)）。viewport 固定 1280×800（同 getComputedStyle 既有默认）。
pub fn compute_document_styles(html: &str) -> (Document, HashMap<NodeId, ComputedStyle>) {
    let doc = parse_html(html);
    let sheets = crate::pipeline::collect_stylesheets(&doc, "");
    let mut sys = StyleSystem::new();
    // 设默认 viewport（length 属性 % 解析需要；首批属性 viewport 无关，但为后续 length 扩展设）。
    sys.set_viewport(1280.0, 800.0);
    let mut styles = sys.compute_styles(&doc, &sheets);
    // R2723：getComputedStyle 对齐 Chromium——`bolder`/`lighter` 按父链 resolved 绝对值解析
    //（CSS Fonts 3 §5.2 计算值语义）。style-system computed 值保关键字供 paint 二值 want_bold
    // 消费；本 getComputedStyle 专用后处理仅改 gCS 路径的副本，paint/render 零影响。
    resolve_font_weight_bolder_lighter(&doc, &mut styles);
    (doc, styles)
}

/// 在已计算的 `(doc, styles)` 上按选择器查询单个属性并序列化。缓存命中路径仅此步
/// （find_by_selector + HashMap lookup + serialize，O(节点)）。
pub fn lookup_computed_property(
    doc: &Document,
    styles: &HashMap<NodeId, ComputedStyle>,
    selector: &str,
    prop: &str,
) -> String {
    let Some(node) = find_by_selector(doc, selector) else {
        return String::new();
    };
    let Some(style) = styles.get(&node) else {
        return String::new();
    };
    serialize_computed_property(style, prop)
}

/// 把 [`ComputedStyle`] 的单个属性序列化为 CSS 字符串（kebab-case 属性名）。覆盖：
/// display/position/visibility/opacity + 颜色族（color/background-color/border-*-color/outline-color）
/// + 长度族（width/height/min-max/margin/padding/border-*-width/border-*-radius/outline-width/
///   font-size/top/right/bottom/left/gap/letter-spacing/word-spacing/text-indent 等，经
///   [`length_to_css`]）+ 关键字/枚举族（float/clear/box-sizing/overflow-x·y/text-align/
///   white-space/font-weight/font-style/line-height/z-index/cursor/text-transform/text-overflow/
///   direction/border-collapse/table-layout/caption-side/border-*-style/outline-style）+ 复合/列表族
///   （font-family/flex-direction/wrap/justify-content/align-items/align-self/writing-mode/object-fit/
///   isolation/mix-blend-mode/pointer-events/user-select/list-style-type·position）+ 数值族
///   （flex-grow/flex-shrink/order/flex-basis/aspect-ratio）+ Transforms（transform 函数列表 /
///   transform-origin 两长度 / transform-style / backface-visibility / perspective /
///   perspective-origin）+ contain（关键字 / 位掩码组合）+ filter（函数列表）+ will-change（列表）
///   + clip-path（basic-shape 函数）+ content（生成内容）+ background 簇（position/size/repeat 多层）。未覆盖属性返 ''。
///
/// **长度族返回计算值**（非 used 值）：`compute_styles` 已把 em/rem/vw/vh/非 % calc 解析为 Px，
/// 故 px 指定值与 real browser getComputedStyle 精确一致；百分比/auto 保留为 `N%`/`auto`
///（real browser 对 width/height/margin 等 geometric 属性返 used 值——需 layout 解析百分比，
/// 此处无 layout 故返计算值；对 font-size/border-radius/gap/letter-spacing 等非 geometric 属性
/// 与 real browser 一致）。
pub fn serialize_computed_property(style: &ComputedStyle, prop: &str) -> String {
    let p = prop.trim().to_ascii_lowercase();
    // 元素自身计算 color，作 currentColor 解析上下文（color 属性本身则 currentColor→自解析）。
    let element_color = &style.color;
    // 元素 font-size（resolve 阶段已为 Px）供长度族兜底解析残余相对单位（em/rem/vw 等）。
    let font_size_px = match &style.font_size {
        LengthValue::Px(v) => *v,
        _ => 16.0,
    };
    let length = |lv: &LengthValue| length_to_css(lv, font_size_px);
    match p.as_str() {
        "display" => display_value_str(&style.display),
        "position" => position_value_str(&style.position),
        "visibility" => visibility_value_str(&style.visibility),
        "opacity" => {
            let o = style.opacity;
            // 整数 opacity 打印无小数点（"1"/"0"），与 real browser getComputedStyle 一致。
            if o == o.trunc() {
                format!("{}", o as i32)
            } else {
                format!("{o}")
            }
        }
        "color" => color_to_css(&crate::resolve_color_current(&style.color, element_color)),
        "background-color" => color_to_css(&crate::resolve_color_current(&style.background_color, element_color)),
        "border-top-color" => color_to_css(&crate::resolve_color_current(&style.border_top_color, element_color)),
        "border-right-color" => color_to_css(&crate::resolve_color_current(&style.border_right_color, element_color)),
        "border-bottom-color" => color_to_css(&crate::resolve_color_current(&style.border_bottom_color, element_color)),
        "border-left-color" => color_to_css(&crate::resolve_color_current(&style.border_left_color, element_color)),
        "outline-color" => color_to_css(&crate::resolve_color_current(&style.outline_color, element_color)),
        // ── 长度族（计算值；resolve_computed_style 已把主要相对单位解析为 Px）──
        "width" => length(&style.width),
        "height" => length(&style.height),
        "min-width" => length(&style.min_width),
        "min-height" => length(&style.min_height),
        "max-width" => max_size_to_css(&style.max_width, font_size_px),
        "max-height" => max_size_to_css(&style.max_height, font_size_px),
        "margin-top" => length(&style.margin_top),
        "margin-right" => length(&style.margin_right),
        "margin-bottom" => length(&style.margin_bottom),
        "margin-left" => length(&style.margin_left),
        "padding-top" => length(&style.padding_top),
        "padding-right" => length(&style.padding_right),
        "padding-bottom" => length(&style.padding_bottom),
        "padding-left" => length(&style.padding_left),
        "border-top-width" => border_width_to_css(&style.border_top_width, &style.border_top_style, font_size_px),
        "border-right-width" => border_width_to_css(&style.border_right_width, &style.border_right_style, font_size_px),
        "border-bottom-width" => {
            border_width_to_css(&style.border_bottom_width, &style.border_bottom_style, font_size_px)
        }
        "border-left-width" => border_width_to_css(&style.border_left_width, &style.border_left_style, font_size_px),
        "border-top-left-radius" => length(&style.border_top_left_radius),
        "border-top-right-radius" => length(&style.border_top_right_radius),
        "border-bottom-right-radius" => length(&style.border_bottom_right_radius),
        "border-bottom-left-radius" => length(&style.border_bottom_left_radius),
        "outline-width" => match &style.outline_style {
            OutlineStyleValue::None => "0px".to_string(),
            _ => length(&style.outline_width),
        },
        "font-size" => length(&style.font_size),
        "top" => length(&style.top),
        "right" => length(&style.right),
        "bottom" => length(&style.bottom),
        "left" => length(&style.left),
        "gap" => length(&style.gap),
        "row-gap" => length(&style.row_gap),
        "column-gap" => length(&style.column_gap),
        "letter-spacing" => length(&style.letter_spacing),
        "word-spacing" => length(&style.word_spacing),
        "text-indent" => length(&style.text_indent),
        // ── 关键字/枚举族 ──
        "float" => float_value_str(&style.float),
        "clear" => clear_value_str(&style.clear),
        "box-sizing" => box_sizing_str(&style.box_sizing),
        "overflow-x" => overflow_value_str(&style.overflow_x),
        "overflow-y" => overflow_value_str(&style.overflow_y),
        "text-align" => text_align_str(&style.text_align),
        "white-space" => white_space_str(&style.white_space),
        "font-weight" => font_weight_str(&style.font_weight),
        "font-style" => font_style_str(&style.font_style),
        "line-height" => line_height_str(&style.line_height, font_size_px),
        "z-index" => z_index_str(&style.z_index),
        "cursor" => cursor_str(&style.cursor),
        "text-transform" => text_transform_str(&style.text_transform),
        "text-overflow" => text_overflow_str(&style.text_overflow),
        "direction" => direction_str(&style.direction),
        "border-collapse" => border_collapse_str(&style.border_collapse),
        "table-layout" => table_layout_str(&style.table_layout),
        "caption-side" => caption_side_str(&style.caption_side),
        "border-top-style" => border_style_str(&style.border_top_style),
        "border-right-style" => border_style_str(&style.border_right_style),
        "border-bottom-style" => border_style_str(&style.border_bottom_style),
        "border-left-style" => border_style_str(&style.border_left_style),
        "outline-style" => outline_style_str(&style.outline_style),
        // ── 复合枚举/列表族（R2710）──
        "font-family" => font_family_to_css(&style.font_family),
        "flex-direction" => flex_direction_str(&style.flex_direction),
        "flex-wrap" => flex_wrap_str(&style.flex_wrap),
        "justify-content" => alignment_str(&style.justify_content),
        "align-items" => alignment_str(&style.align_items),
        "align-self" => alignment_str(&style.align_self),
        "writing-mode" => writing_mode_str(&style.writing_mode),
        "object-fit" => object_fit_str(&style.object_fit),
        "isolation" => isolation_str(&style.isolation),
        "mix-blend-mode" => mix_blend_mode_str(&style.mix_blend_mode),
        "pointer-events" => pointer_events_str(&style.pointer_events),
        "user-select" => user_select_str(&style.user_select),
        "list-style-type" => list_style_type_str(&style.list_style_type),
        "list-style-position" => list_style_position_str(&style.list_style_position),
        // ── 数值/special 族（R2711）──
        "flex-grow" => format_num(style.flex_grow, ""),
        "flex-shrink" => format_num(style.flex_shrink, ""),
        "order" => style.order.to_string(),
        "flex-basis" => flex_basis_str(&style.flex_basis, font_size_px),
        "aspect-ratio" => aspect_ratio_str(style.aspect_ratio, style.aspect_ratio_auto),
        // ── transform（R2715）── CSS Transforms L1/L2 计算值 = 函数列表（Chromium 返 resolved matrix，diverge）。
        "transform" => transform_to_css(&style.transform),
        // ── transform-origin（R2716）── 2 LengthValue 经 length_to_css，空格连接。
        "transform-origin" => {
            transform_origin_to_css(&style.transform_origin_x, &style.transform_origin_y, font_size_px)
        }
        // ── contain（R2717）── CSS Containment L1/L2 计算值（关键字 / 位掩码组合）。
        "contain" => contain_to_css(&style.contain),
        // ── filter（R2718）── CSS Filter Effects 函数列表（空 Vec / None → none）。
        "filter" => filter_to_css(&style.filter, element_color),
        // ── 3D Transforms 簇（R2719）── transform-style / backface-visibility 枚举 + perspective /
        // perspective-origin（与 transform / transform-origin 同族，完成 3D transform 簇）。
        "transform-style" => transform_style_str(&style.transform_style),
        "backface-visibility" => backface_visibility_str(&style.backface_visibility),
        "perspective" => perspective_to_css(&style.perspective, font_size_px),
        "perspective-origin" => format!(
            "{} {}",
            length_to_css(&style.perspective_origin_x, font_size_px),
            length_to_css(&style.perspective_origin_y, font_size_px)
        ),
        // ── will-change（R2720）── CSS Will Change 列表（空 Vec / Auto → auto）。
        "will-change" => will_change_to_css(&style.will_change),
        // ── clip-path（R2721）── CSS Masking basic-shape 函数（none / inset / circle / ellipse / polygon）。
        "clip-path" => clip_path_to_css(&style.clip_path, font_size_px),
        // ── content（R2722）── CSS Generated Content（::before/::after 生成内容，多 component value）。
        "content" => content_to_css(&style.content),
        // ── background-position（R2724）── CSS Backgrounds <bg-position># 多层（逗号分隔）。
        "background-position" => background_position_to_css(&style.background_position),
        // ── background-size / background-repeat（R2725）── CSS Backgrounds 多层（逗号分隔）。
        "background-size" => background_size_to_css(&style.background_size),
        "background-repeat" => background_repeat_to_css(&style.background_repeat),
        // ── background-attachment / clip / origin（R2726）── CSS Backgrounds 单值 box-model 枚举。
        "background-attachment" => background_attachment_to_css(&style.background_attachment),
        "background-clip" => background_clip_to_css(&style.background_clip),
        "background-origin" => background_origin_to_css(&style.background_origin),
        // ── align-content / justify-items / justify-self（R2727）── CSS Box Alignment 单值枚举
        // （补齐 align-items/align-self/justify-content R2710 后的 alignment 簇缺口）。
        "align-content" => align_content_to_css(&style.align_content),
        "justify-items" => justify_items_to_css(&style.justify_items),
        "justify-self" => justify_self_to_css(&style.justify_self),
        // ── word-break / overflow-wrap / hyphens / line-break（R2728）── CSS Text 换行/断词单值枚举。
        "word-break" => word_break_to_css(&style.word_break),
        "overflow-wrap" => overflow_wrap_to_css(&style.overflow_wrap),
        "hyphens" => hyphens_to_css(&style.hyphens),
        "line-break" => line_break_to_css(&style.line_break),
        // ── vertical-align / unicode-bidi / empty-cells（R2729）── 单值关键字枚举。
        "vertical-align" => vertical_align_to_css(&style.vertical_align),
        "unicode-bidi" => unicode_bidi_to_css(&style.unicode_bidi),
        "empty-cells" => empty_cells_to_css(&style.empty_cells),
        // ── caret-color / accent-color（R2730）── CSS UI 颜色（auto | <color>，复用 R2705 颜色解析）。
        "caret-color" => caret_color_to_css(&style.caret_color, element_color),
        "accent-color" => accent_color_to_css(&style.accent_color, element_color),
        _ => String::new(),
    }
}

/// 把解析后的 [`Color`]（render-foundation，u8 通道）序列化为 CSS 颜色串。不透明 →
/// `rgb(r, g, b)`；含透明度 → `rgba(r, g, b, a)`（a 为 0-1 小数，对齐 real browser）。
fn color_to_css(c: &zero_render_foundation::color::Color) -> String {
    if c.a == 255 {
        format!("rgb({}, {}, {})", c.r, c.g, c.b)
    } else {
        let alpha = ((c.a as f64 / 255.0) * 1000.0).round() / 1000.0;
        format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, alpha)
    }
}

/// caret-color：CSS UI 光标颜色（auto | <color>）。Auto→auto（CSS UI 4 初值，Chromium 返 auto）；
/// Color→经 [`crate::resolve_color_current`] 解析 currentcolor 后 rgb/rgba（复用 R2705 颜色路径）。
fn caret_color_to_css(c: &CaretColorComputedValue, element_color: &ColorValue) -> String {
    match c {
        CaretColorComputedValue::Auto => "auto".to_string(),
        CaretColorComputedValue::Color(col) => color_to_css(&crate::resolve_color_current(col, element_color)),
    }
}

/// accent-color：CSS UI 表单控件强调色（auto | <color>）。Auto→auto；Color→rgb/rgba（复用 R2705）。
fn accent_color_to_css(a: &AccentColorComputedValue, element_color: &ColorValue) -> String {
    match a {
        AccentColorComputedValue::Auto => "auto".to_string(),
        AccentColorComputedValue::Color(col) => color_to_css(&crate::resolve_color_current(col, element_color)),
    }
}

/// 把数值序列化为带后缀的 CSS 量（对齐 real browser getComputedStyle 数值串）。
/// 整数无小数点（`16px`/`50%`/`0px`），小数去尾零（`19.2px`）。四舍五入到 1e-3 抑制
/// f64 累积噪声（如 em 解析 `1.2 * 16 = 19.200000000000003` → `19.2px`）。
fn format_num(v: f64, suffix: &str) -> String {
    let r = (v * 1000.0).round() / 1000.0;
    if r == r.trunc() {
        format!("{}{}", r as i64, suffix)
    } else {
        format!("{r}{suffix}")
    }
}

/// 把已计算的 [`LengthValue`] 序列化为 CSS 字符串（getComputedStyle 长度族）。
///
/// `compute_styles` 经 `resolve_computed_style` 已把主要字段的 em/rem/vw/vh/vmin/vmax/ch/非 % calc
/// 解析为 `Px`；此处对残余相对单位（resolve 未覆盖的字段如 text-indent 的 em）用 `font_size_px`
/// 兜底解析为 px。百分比/auto/关键字保留为计算值（`N%`/`auto`/`min-content`/…）；含 % 的 calc
/// 无容器尺寸无法解析为绝对值 → ''。
fn length_to_css(lv: &LengthValue, font_size_px: f64) -> String {
    // viewport 与 compute_document_styles 一致（getComputedStyle 默认 1280×800）。
    const VW: f64 = 1280.0;
    const VH: f64 = 800.0;
    match lv {
        LengthValue::Px(v) => format_num(*v, "px"),
        LengthValue::Percentage(v) => format_num(*v, "%"),
        LengthValue::Auto => "auto".to_string(),
        LengthValue::MinContent => "min-content".to_string(),
        LengthValue::MaxContent => "max-content".to_string(),
        LengthValue::FitContent(_) => "fit-content".to_string(),
        // 含 % 的 calc 无容器尺寸无法解析（非 % calc 已被 resolve_computed_style 转 Px）→ ''。
        LengthValue::Calc(_) => String::new(),
        // em/rem/vw/vh/vmin/vmax/ch：兜底解析残余相对单位为 px。
        other => format_num(
            zero_style_system::resolve_length(other, font_size_px, Some(VW), Some(VH)),
            "px",
        ),
    }
}

/// 序列化 `border-*-width` 的计算值。real browser getComputedStyle 对 border-width 返 used 值：
/// `border-style: none/hidden` → `"0px"`（ZeroWeb computed 值保留 specified 宽度供 inherit，
/// 见 `computed.rs` csswg #2768 注；gCS 须对齐 Chromium 的 used 行为，故 none/hidden 归零）。
fn border_width_to_css(width: &LengthValue, style: &BorderStyleValue, font_size_px: f64) -> String {
    match style {
        BorderStyleValue::None | BorderStyleValue::Hidden => "0px".to_string(),
        _ => length_to_css(width, font_size_px),
    }
}

/// 序列化 `max-width`/`max-height` 的计算值。ZeroWeb 用 `Px(f64::INFINITY)` 表示 initial 值
/// `none`（见 `default_impl.rs`），real browser getComputedStyle 对 max-size:none 返 `"none"`。
fn max_size_to_css(lv: &LengthValue, font_size_px: f64) -> String {
    match lv {
        LengthValue::Px(v) if v.is_infinite() => "none".to_string(),
        _ => length_to_css(lv, font_size_px),
    }
}

fn display_value_str(d: &DisplayValue) -> String {
    match d {
        DisplayValue::Block => "block",
        DisplayValue::Inline => "inline",
        DisplayValue::InlineBlock => "inline-block",
        DisplayValue::Flex => "flex",
        DisplayValue::InlineFlex => "inline-flex",
        DisplayValue::Grid => "grid",
        DisplayValue::InlineGrid => "inline-grid",
        DisplayValue::None => "none",
        DisplayValue::Contents => "contents",
        DisplayValue::Flow => "flow",
        DisplayValue::FlowRoot => "flow-root",
        DisplayValue::ListItem => "list-item",
        DisplayValue::Table => "table",
        DisplayValue::InlineTable => "inline-table",
        DisplayValue::TableRow => "table-row",
        DisplayValue::TableCell => "table-cell",
        DisplayValue::TableCaption => "table-caption",
        DisplayValue::TableColumn => "table-column",
        DisplayValue::TableColumnGroup => "table-column-group",
        DisplayValue::TableRowGroup => "table-row-group",
        DisplayValue::TableHeaderGroup => "table-header-group",
        DisplayValue::TableFooterGroup => "table-footer-group",
    }
    .to_string()
}

fn position_value_str(p: &PositionValue) -> String {
    match p {
        PositionValue::Static => "static",
        PositionValue::Relative => "relative",
        PositionValue::Absolute => "absolute",
        PositionValue::Fixed => "fixed",
        PositionValue::Sticky => "sticky",
    }
    .to_string()
}

fn visibility_value_str(v: &VisibilityValue) -> String {
    match v {
        VisibilityValue::Visible => "visible",
        VisibilityValue::Hidden => "hidden",
        VisibilityValue::Collapse => "collapse",
    }
    .to_string()
}

// ── getComputedStyle 关键字/枚举族序列化（R2708）── 多为 variant→kebab-case 关键字直映。

fn float_value_str(f: &FloatValue) -> String {
    match f {
        FloatValue::None => "none",
        FloatValue::Left => "left",
        FloatValue::Right => "right",
        FloatValue::InlineStart => "inline-start",
        FloatValue::InlineEnd => "inline-end",
    }
    .to_string()
}

fn clear_value_str(c: &ClearValue) -> String {
    match c {
        ClearValue::None => "none",
        ClearValue::Left => "left",
        ClearValue::Right => "right",
        ClearValue::Both => "both",
        ClearValue::InlineStart => "inline-start",
        ClearValue::InlineEnd => "inline-end",
    }
    .to_string()
}

fn overflow_value_str(o: &OverflowValue) -> String {
    match o {
        OverflowValue::Visible => "visible",
        OverflowValue::Hidden => "hidden",
        OverflowValue::Scroll => "scroll",
        OverflowValue::Auto => "auto",
        OverflowValue::Clip => "clip",
    }
    .to_string()
}

fn box_sizing_str(b: &BoxSizingValue) -> String {
    match b {
        BoxSizingValue::ContentBox => "content-box",
        BoxSizingValue::BorderBox => "border-box",
    }
    .to_string()
}

fn text_align_str(a: &TextAlignValue) -> String {
    match a {
        TextAlignValue::Left => "left",
        TextAlignValue::Right => "right",
        TextAlignValue::Center => "center",
        TextAlignValue::Justify => "justify",
        TextAlignValue::Start => "start",
        TextAlignValue::End => "end",
        TextAlignValue::MatchParent => "match-parent",
    }
    .to_string()
}

fn white_space_str(w: &WhiteSpaceValue) -> String {
    match w {
        WhiteSpaceValue::Normal => "normal",
        WhiteSpaceValue::Pre => "pre",
        WhiteSpaceValue::Nowrap => "nowrap",
        WhiteSpaceValue::PreWrap => "pre-wrap",
        WhiteSpaceValue::PreLine => "pre-line",
        WhiteSpaceValue::BreakSpaces => "break-spaces",
    }
    .to_string()
}

/// real browser getComputedStyle 把 font-weight 解析为绝对值（normal=400、bold=700）。
/// bolder/lighter 须父链解析，此处保关键字（计算值，与 Chromium 对这些值有 diverge）。
fn font_weight_str(w: &FontWeightValue) -> String {
    match w {
        FontWeightValue::Absolute(n) => n.to_string(),
        FontWeightValue::Normal => "400".to_string(),
        FontWeightValue::Bold => "700".to_string(),
        FontWeightValue::Bolder => "bolder".to_string(),
        FontWeightValue::Lighter => "lighter".to_string(),
    }
}

fn font_style_str(s: &FontStyleValue) -> String {
    match s {
        FontStyleValue::Normal => "normal".to_string(),
        FontStyleValue::Italic => "italic".to_string(),
        FontStyleValue::Oblique(None) => "oblique".to_string(),
        FontStyleValue::Oblique(Some(deg)) => format!("oblique {deg}deg"),
    }
}

/// line-height：normal→`normal`；number→无单位数（resolve 阶段 em 已转 Px）。
fn line_height_str(lh: &LineHeightValue, font_size_px: f64) -> String {
    match lh {
        LineHeightValue::Normal => "normal".to_string(),
        LineHeightValue::Number(n) => format_num(*n, ""),
        LineHeightValue::Length(lv) => length_to_css(lv, font_size_px),
    }
}

fn z_index_str(z: &ZIndexValue) -> String {
    match z {
        ZIndexValue::Auto => "auto".to_string(),
        ZIndexValue::Integer(n) => n.to_string(),
    }
}

fn cursor_str(c: &CursorValue) -> String {
    match c {
        CursorValue::Auto => "auto",
        CursorValue::Default => "default",
        CursorValue::Pointer => "pointer",
        CursorValue::Move => "move",
        CursorValue::Text => "text",
        CursorValue::Wait => "wait",
        CursorValue::Crosshair => "crosshair",
        CursorValue::Help => "help",
        CursorValue::NotAllowed => "not-allowed",
        CursorValue::Grab => "grab",
        CursorValue::Grabbing => "grabbing",
        CursorValue::ColResize => "col-resize",
        CursorValue::RowResize => "row-resize",
        CursorValue::NsResize => "ns-resize",
        CursorValue::EwResize => "ew-resize",
        CursorValue::None => "none",
        CursorValue::Progress => "progress",
        CursorValue::Cell => "cell",
        CursorValue::Copy => "copy",
        CursorValue::Alias => "alias",
        CursorValue::AllScroll => "all-scroll",
        CursorValue::ZoomIn => "zoom-in",
        CursorValue::ZoomOut => "zoom-out",
    }
    .to_string()
}

fn text_transform_str(t: &TextTransformValue) -> String {
    match t {
        TextTransformValue::None => "none",
        TextTransformValue::Uppercase => "uppercase",
        TextTransformValue::Lowercase => "lowercase",
        TextTransformValue::Capitalize => "capitalize",
        TextTransformValue::FullWidth => "full-width",
        TextTransformValue::FullSizeKana => "full-size-kana",
    }
    .to_string()
}

fn text_overflow_str(t: &TextOverflowValue) -> String {
    match t {
        TextOverflowValue::Clip => "clip".to_string(),
        TextOverflowValue::Ellipsis => "ellipsis".to_string(),
        TextOverflowValue::String(s) => s.clone(),
    }
}

fn direction_str(d: &DirectionValue) -> String {
    match d {
        DirectionValue::Ltr => "ltr",
        DirectionValue::Rtl => "rtl",
    }
    .to_string()
}

fn border_collapse_str(b: &BorderCollapseValue) -> String {
    match b {
        BorderCollapseValue::Separate => "separate",
        BorderCollapseValue::Collapse => "collapse",
    }
    .to_string()
}

fn table_layout_str(t: &TableLayoutValue) -> String {
    match t {
        TableLayoutValue::Auto => "auto",
        TableLayoutValue::Fixed => "fixed",
    }
    .to_string()
}

fn caption_side_str(c: &CaptionSideValue) -> String {
    match c {
        CaptionSideValue::Top => "top",
        CaptionSideValue::Bottom => "bottom",
    }
    .to_string()
}

fn border_style_str(s: &BorderStyleValue) -> String {
    match s {
        BorderStyleValue::None => "none",
        BorderStyleValue::Hidden => "hidden",
        BorderStyleValue::Dotted => "dotted",
        BorderStyleValue::Dashed => "dashed",
        BorderStyleValue::Solid => "solid",
        BorderStyleValue::Double => "double",
        BorderStyleValue::Groove => "groove",
        BorderStyleValue::Ridge => "ridge",
        BorderStyleValue::Inset => "inset",
        BorderStyleValue::Outset => "outset",
    }
    .to_string()
}

fn outline_style_str(s: &OutlineStyleValue) -> String {
    match s {
        OutlineStyleValue::None => "none",
        OutlineStyleValue::Dotted => "dotted",
        OutlineStyleValue::Dashed => "dashed",
        OutlineStyleValue::Solid => "solid",
        OutlineStyleValue::Double => "double",
        OutlineStyleValue::Groove => "groove",
        OutlineStyleValue::Ridge => "ridge",
        OutlineStyleValue::Inset => "inset",
        OutlineStyleValue::Outset => "outset",
        OutlineStyleValue::Auto => "auto",
    }
    .to_string()
}

// ── 复合枚举/列表族序列化（R2710）──

/// font-family：逗号分隔，每个族名按 CSS ident 规则决定是否加引号（对齐 Chromium：
/// `"Helvetica Neue", Arial, sans-serif`）。空 Vec → ''（UA 默认字体未入 computed）。
fn font_family_to_css(families: &[String]) -> String {
    families
        .iter()
        .map(|f| quote_font_family(f))
        .collect::<Vec<_>>()
        .join(", ")
}

/// 族名是合法 CSS 标识符（非空、首字符非数字/`-数字`、仅 ident 字符）→ 不引号；否则双引号
///（转义内嵌 `"`、`\`）。对齐 real browser 的 font-family 序列化。
fn quote_font_family(name: &str) -> String {
    if is_css_ident(name) {
        name.to_string()
    } else {
        let escaped = name.replace('\\', "\\\\").replace('"', "\\\"");
        format!("\"{escaped}\"")
    }
}

/// 判断字符串是否为合法 CSS 标识符（font-family 不引号条件）。非空；首字符为字母/下划线/
/// 连字符（但非 `-` 后跟数字，如 `-1`）；其余字符为字母/数字/`-`/`_`。
fn is_css_ident(s: &str) -> bool {
    let mut chars = s.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    // 首字符：字母/_；或 `-` 后非数字（CSS：`-` 须后跟 ident 字符，不能直接跟数字）。
    let first_ok = first.is_ascii_alphabetic()
        || first == '_'
        || (first == '-' && !matches!(chars.clone().next(), Some(c) if c.is_ascii_digit()));
    first_ok && chars.all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

fn flex_direction_str(d: &FlexDirectionValue) -> String {
    match d {
        FlexDirectionValue::Row => "row",
        FlexDirectionValue::RowReverse => "row-reverse",
        FlexDirectionValue::Column => "column",
        FlexDirectionValue::ColumnReverse => "column-reverse",
    }
    .to_string()
}

fn flex_wrap_str(w: &FlexWrapValue) -> String {
    match w {
        FlexWrapValue::Nowrap => "nowrap",
        FlexWrapValue::Wrap => "wrap",
        FlexWrapValue::WrapReverse => "wrap-reverse",
    }
    .to_string()
}

/// justify-content / align-items / align-self 共用（CSS Box Align 3）。
fn alignment_str(a: &AlignmentValue) -> String {
    match a {
        AlignmentValue::Auto => "auto",
        AlignmentValue::Normal => "normal",
        AlignmentValue::FlexStart => "flex-start",
        AlignmentValue::FlexEnd => "flex-end",
        AlignmentValue::Center => "center",
        AlignmentValue::SpaceBetween => "space-between",
        AlignmentValue::SpaceAround => "space-around",
        AlignmentValue::SpaceEvenly => "space-evenly",
        AlignmentValue::Stretch => "stretch",
        AlignmentValue::Start => "start",
        AlignmentValue::End => "end",
        AlignmentValue::Baseline => "baseline",
    }
    .to_string()
}

fn writing_mode_str(w: &WritingModeValue) -> String {
    match w {
        WritingModeValue::HorizontalTb => "horizontal-tb",
        WritingModeValue::VerticalRl => "vertical-rl",
        WritingModeValue::VerticalLr => "vertical-lr",
    }
    .to_string()
}

fn object_fit_str(o: &ObjectFitComputedValue) -> String {
    match o {
        ObjectFitComputedValue::Fill => "fill",
        ObjectFitComputedValue::Contain => "contain",
        ObjectFitComputedValue::Cover => "cover",
        ObjectFitComputedValue::None => "none",
        ObjectFitComputedValue::ScaleDown => "scale-down",
    }
    .to_string()
}

fn isolation_str(i: &IsolationValue) -> String {
    match i {
        IsolationValue::Auto => "auto",
        IsolationValue::Isolate => "isolate",
    }
    .to_string()
}

fn mix_blend_mode_str(m: &MixBlendModeComputedValue) -> String {
    match m {
        MixBlendModeComputedValue::Normal => "normal",
        MixBlendModeComputedValue::Multiply => "multiply",
        MixBlendModeComputedValue::Screen => "screen",
        MixBlendModeComputedValue::Overlay => "overlay",
        MixBlendModeComputedValue::Darken => "darken",
        MixBlendModeComputedValue::Lighten => "lighten",
        MixBlendModeComputedValue::ColorDodge => "color-dodge",
        MixBlendModeComputedValue::ColorBurn => "color-burn",
        MixBlendModeComputedValue::HardLight => "hard-light",
        MixBlendModeComputedValue::SoftLight => "soft-light",
        MixBlendModeComputedValue::Difference => "difference",
        MixBlendModeComputedValue::Exclusion => "exclusion",
        MixBlendModeComputedValue::Hue => "hue",
        MixBlendModeComputedValue::Saturation => "saturation",
        MixBlendModeComputedValue::Color => "color",
        MixBlendModeComputedValue::Luminosity => "luminosity",
    }
    .to_string()
}

fn pointer_events_str(p: &PointerEventsValue) -> String {
    match p {
        PointerEventsValue::Auto => "auto",
        PointerEventsValue::None => "none",
        PointerEventsValue::VisiblePainted => "visiblePainted",
        PointerEventsValue::VisibleFill => "visibleFill",
        PointerEventsValue::VisibleStroke => "visibleStroke",
        PointerEventsValue::Visible => "visible",
        PointerEventsValue::Painted => "painted",
        PointerEventsValue::Fill => "fill",
        PointerEventsValue::Stroke => "stroke",
        PointerEventsValue::All => "all",
        PointerEventsValue::Inherit => "inherit",
    }
    .to_string()
}

fn user_select_str(u: &UserSelectValue) -> String {
    match u {
        UserSelectValue::Auto => "auto",
        UserSelectValue::Text => "text",
        UserSelectValue::None => "none",
        UserSelectValue::All => "all",
        UserSelectValue::Contain => "contain",
    }
    .to_string()
}

/// list-style-type：~28 builtin 关键字 + Custom(name)（`@counter-style` 名，不引号）+
/// String(s)（`<string>` 标记，双引号）。对齐 Chromium getComputedStyle 序列化。
fn list_style_type_str(t: &ListStyleTypeValue) -> String {
    match t {
        ListStyleTypeValue::Disc => "disc".into(),
        ListStyleTypeValue::Circle => "circle".into(),
        ListStyleTypeValue::Square => "square".into(),
        ListStyleTypeValue::Decimal => "decimal".into(),
        ListStyleTypeValue::DecimalLeadingZero => "decimal-leading-zero".into(),
        ListStyleTypeValue::LowerRoman => "lower-roman".into(),
        ListStyleTypeValue::UpperRoman => "upper-roman".into(),
        ListStyleTypeValue::LowerAlpha => "lower-alpha".into(),
        ListStyleTypeValue::UpperAlpha => "upper-alpha".into(),
        ListStyleTypeValue::LowerGreek => "lower-greek".into(),
        ListStyleTypeValue::Persian => "persian".into(),
        ListStyleTypeValue::Armenian => "armenian".into(),
        ListStyleTypeValue::LowerArmenian => "lower-armenian".into(),
        ListStyleTypeValue::Georgian => "georgian".into(),
        ListStyleTypeValue::Hebrew => "hebrew".into(),
        ListStyleTypeValue::ArabicIndic => "arabic-indic".into(),
        ListStyleTypeValue::Devanagari => "devanagari".into(),
        ListStyleTypeValue::Bengali => "bengali".into(),
        ListStyleTypeValue::Gujarati => "gujarati".into(),
        ListStyleTypeValue::Gurmukhi => "gurmukhi".into(),
        ListStyleTypeValue::Kannada => "kannada".into(),
        ListStyleTypeValue::Malayalam => "malayalam".into(),
        ListStyleTypeValue::Tamil => "tamil".into(),
        ListStyleTypeValue::Telugu => "telugu".into(),
        ListStyleTypeValue::Lao => "lao".into(),
        ListStyleTypeValue::Khmer => "khmer".into(),
        ListStyleTypeValue::Myanmar => "myanmar".into(),
        ListStyleTypeValue::CjkDecimal => "cjk-decimal".into(),
        ListStyleTypeValue::None => "none".into(),
        ListStyleTypeValue::Custom(name) => name.clone(),
        ListStyleTypeValue::String(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{escaped}\"")
        }
    }
}

fn list_style_position_str(p: &ListStylePositionValue) -> String {
    match p {
        ListStylePositionValue::Outside => "outside",
        ListStylePositionValue::Inside => "inside",
    }
    .to_string()
}

// ── 数值/special 族序列化（R2711）──

/// flex-basis：auto/content/length（em 已在 resolve 阶段转 Px）。
fn flex_basis_str(b: &FlexBasisValue, font_size_px: f64) -> String {
    match b {
        FlexBasisValue::Auto => "auto".to_string(),
        FlexBasisValue::Content => "content".to_string(),
        FlexBasisValue::Length(lv) => length_to_css(lv, font_size_px),
    }
}

/// aspect-ratio：None/auto → `auto`；Some(r) 无 auto → 数值；Some(r) + auto → `auto <r>`。
///
/// **已知 diverge**：ZeroWeb 只存合并比值（`Option<f32>`），不保留原始 `w / h`——故
/// `aspect-ratio: 16 / 9`（Chrome 返 `"16 / 9"`）序列化为 `"1.7778"`。单数值（`2`）与 auto
/// 路径与 Chromium 一致。
fn aspect_ratio_str(ratio: Option<f32>, auto: bool) -> String {
    match (ratio, auto) {
        (None, _) => "auto".to_string(),
        (Some(r), false) => format_num(r as f64, ""),
        (Some(r), true) => format!("auto {}", format_num(r as f64, "")),
    }
}

/// transform：按 CSS Transforms L1/L2 计算值序列化为**函数列表**（none / 空格分隔函数）。
///
/// **已知 diverge**：Chromium getComputedStyle 对非 none transform 返 **resolved matrix**
///（`matrix(...)` / `matrix3d(...)`）；CSS 规范（L1/L2）+ Firefox 返函数列表（长度解析为 px、
/// 百分比保留——border-box 相对，须 layout 故保 `%`）。ZeroWeb 返函数列表（spec-correct）。
fn transform_to_css(t: &TransformValue) -> String {
    match t {
        TransformValue::None => "none".to_string(),
        TransformValue::List(fns) => fns.iter().map(transform_function_to_css).collect::<Vec<_>>().join(" "),
    }
}

/// transform-origin：按 CSS Transforms L1 计算值序列化为两个长度（空格连接）。
///
/// 存储为 `transform_origin_x`/`transform_origin_y` 两个 `LengthValue`，默认均为 `50%`
///（即关键字 `center` 的计算值）。经 [`length_to_css`]：px 指定值精确、百分比保留为 `N%`。
///
/// **已知 diverge（同 transform / width·height used-value）**：Chromium getComputedStyle 对
/// transform-origin 返 **used** 值（border-box 中心绝对 px，如 `100px 50px`）；CSS 规范 + Firefox
/// 返**计算值**（百分比/指定 px）。ZeroWeb 返计算值（spec-correct）。单值 `center`/`top`/`left` 等
/// 轴感知 `<position>` 关键字语法解析为独立 follow-up（当前 apply 仅解析长度，关键字降级为默认
/// `50% 50%`，恰好等于 `center` 计算值，故 `center` 行为正确）。
fn transform_origin_to_css(x: &LengthValue, y: &LengthValue, font_size_px: f64) -> String {
    format!("{} {}", length_to_css(x, font_size_px), length_to_css(y, font_size_px))
}

/// contain：按 CSS Containment L1/L2 计算值序列化。
///
/// `None`/`Strict`/`Content`/`Size`/`Layout`/`Style`/`Paint` → 对应关键字（`strict`/`content`
/// 保留 shorthand 不展开，与 Chromium getComputedStyle 一致）；`Custom(u8)` 位掩码按 spec 语法序
///（`size || layout || paint || style`）解码为空格分隔关键字列表。空位掩码→`none`（防御性，
/// parser 正常不产 Custom(0)）。
fn contain_to_css(c: &ContainComputedValue) -> String {
    use ContainComputedValue as C;
    match c {
        C::None => "none".to_string(),
        C::Strict => "strict".to_string(),
        C::Content => "content".to_string(),
        C::Size => "size".to_string(),
        C::Layout => "layout".to_string(),
        C::Style => "style".to_string(),
        C::Paint => "paint".to_string(),
        C::Custom(flags) => {
            let mut parts: Vec<&str> = Vec::new();
            if (flags & C::FLAG_SIZE) != 0 {
                parts.push("size");
            }
            if (flags & C::FLAG_LAYOUT) != 0 {
                parts.push("layout");
            }
            if (flags & C::FLAG_PAINT) != 0 {
                parts.push("paint");
            }
            if (flags & C::FLAG_STYLE) != 0 {
                parts.push("style");
            }
            if parts.is_empty() {
                "none".to_string()
            } else {
                parts.join(" ")
            }
        }
    }
}

/// filter：按 CSS Filter Effects 计算值序列化为**函数列表**（none / 空格分隔函数）。
///
/// 空 `Vec`（= `filter: none`，见 `parse_filter_list`）→ `none`；否则各函数空格连接。
/// 长度函数 `blur` 为 px；数值函数（brightness/contrast/...）无单位；`hue-rotate` 为 `deg`；
/// `drop-shadow` 颜色经 [`crate::resolve_color_current`] 解析（currentcolor→元素计算 color）后串行化。
fn filter_to_css(filters: &[FilterComputedValue], element_color: &ColorValue) -> String {
    if filters.is_empty() {
        return "none".to_string();
    }
    filters
        .iter()
        .map(|f| filter_function_to_css(f, element_color))
        .collect::<Vec<_>>()
        .join(" ")
}

/// 序列化单个 [`FilterComputedValue`] 函数。
fn filter_function_to_css(f: &FilterComputedValue, element_color: &ColorValue) -> String {
    use FilterComputedValue as F;
    match f {
        F::None => "none".to_string(),
        F::Blur(n) => format!("blur({})", format_num(*n as f64, "px")),
        F::Brightness(n) => format!("brightness({})", format_num(*n as f64, "")),
        F::Contrast(n) => format!("contrast({})", format_num(*n as f64, "")),
        F::Grayscale(n) => format!("grayscale({})", format_num(*n as f64, "")),
        F::HueRotate(n) => format!("hue-rotate({}deg)", format_num(*n as f64, "")),
        F::Invert(n) => format!("invert({})", format_num(*n as f64, "")),
        F::Opacity(n) => format!("opacity({})", format_num(*n as f64, "")),
        F::Saturate(n) => format!("saturate({})", format_num(*n as f64, "")),
        F::Sepia(n) => format!("sepia({})", format_num(*n as f64, "")),
        F::DropShadow(x, y, blur, color) => format!(
            "drop-shadow({} {} {} {})",
            format_num(*x as f64, "px"),
            format_num(*y as f64, "px"),
            format_num(*blur as f64, "px"),
            color_to_css(&crate::resolve_color_current(color, element_color)),
        ),
    }
}

/// transform-style：CSS Transforms 2 计算值（flat / preserve-3d）。
fn transform_style_str(t: &TransformStyleValue) -> String {
    match t {
        TransformStyleValue::Flat => "flat",
        TransformStyleValue::Preserve3d => "preserve-3d",
    }
    .to_string()
}

/// backface-visibility：CSS Transforms 2 计算值（visible / hidden）。
fn backface_visibility_str(b: &BackfaceVisibilityValue) -> String {
    match b {
        BackfaceVisibilityValue::Visible => "visible",
        BackfaceVisibilityValue::Hidden => "hidden",
    }
    .to_string()
}

/// perspective：CSS Transforms 2 计算值。ZeroWeb 用 `Px(0.0)` 表示 initial 值 `none`
///（见 `default_impl.rs` / `apply` 的 `none→Px(0.0)`），real browser getComputedStyle 对
/// perspective:none 返 `"none"`（对齐 [`max_size_to_css`] 的 INFINITY→none 模式）。
fn perspective_to_css(lv: &LengthValue, font_size_px: f64) -> String {
    match lv {
        LengthValue::Px(v) if *v == 0.0 => "none".to_string(),
        _ => length_to_css(lv, font_size_px),
    }
}

/// will-change：CSS Will Change 计算值。空 `Vec`（默认 + `will-change: auto`，见
/// `parse_will_change_list`）→ `auto`；否则各标识符空格连接（`scroll-position` / `contents` /
/// 自定义属性名原样）。对齐 Chromium getComputedStyle。
fn will_change_to_css(list: &[WillChangeValue]) -> String {
    if list.is_empty() {
        return "auto".to_string();
    }
    list.iter()
        .map(|v| match v {
            WillChangeValue::Auto => "auto",
            WillChangeValue::ScrollPosition => "scroll-position",
            WillChangeValue::Contents => "contents",
            WillChangeValue::Custom(s) => s.as_str(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// clip-path：按 CSS Masking 计算值序列化 basic-shape 函数。
///
/// `None`→`none`；`inset()` 四内缩按 box 简写折叠（1/2/3/4 值，同 margin 语法）+ 可选 `round`；
/// `circle()`/`ellipse()` 半径 + 可选 ` at <pos>`；`polygon()` 填充规则（仅 evenodd 输出）+ 逗号分隔顶点。
/// 长度经 [`length_to_css`]；半径关键字 closest-side/farthest-side 原样。
fn clip_path_to_css(c: &ClipPathValue, font_size_px: f64) -> String {
    use ClipPathValue as C;
    match c {
        C::None => "none".to_string(),
        C::Inset {
            top,
            right,
            bottom,
            left,
            round,
        } => {
            let mut inner = box_4_to_css(top, right, bottom, left, font_size_px);
            if let Some(r) = round {
                inner.push_str(&format!(" round {}", clip_path_radius_to_css(r, font_size_px)));
            }
            format!("inset({inner})")
        }
        C::Circle { radius, position } => {
            let mut inner = clip_path_radius_to_css(radius, font_size_px);
            if let Some((x, y)) = position {
                inner.push_str(&format!(
                    " at {} {}",
                    length_to_css(x, font_size_px),
                    length_to_css(y, font_size_px)
                ));
            }
            format!("circle({inner})")
        }
        C::Ellipse { rx, ry, position } => {
            let mut inner = format!(
                "{} {}",
                clip_path_radius_to_css(rx, font_size_px),
                clip_path_radius_to_css(ry, font_size_px)
            );
            if let Some((x, y)) = position {
                inner.push_str(&format!(
                    " at {} {}",
                    length_to_css(x, font_size_px),
                    length_to_css(y, font_size_px)
                ));
            }
            format!("ellipse({inner})")
        }
        C::Polygon { fill_rule, points } => {
            let pts = points
                .iter()
                .map(|(x, y)| format!("{} {}", length_to_css(x, font_size_px), length_to_css(y, font_size_px)))
                .collect::<Vec<_>>()
                .join(", ");
            match fill_rule {
                PolygonFillRule::NonZero => format!("polygon({pts})"),
                PolygonFillRule::EvenOdd => format!("polygon(evenodd, {pts})"),
            }
        }
    }
}

/// 序列化 clip-path 半径（circle/ellipse）。具体长度经 [`length_to_css`]；关键字原样。
fn clip_path_radius_to_css(r: &ClipPathRadius, font_size_px: f64) -> String {
    match r {
        ClipPathRadius::Length(lv) => length_to_css(lv, font_size_px),
        ClipPathRadius::ClosestSide => "closest-side".to_string(),
        ClipPathRadius::FarthestSide => "farthest-side".to_string(),
    }
}

/// 把 4 个 box 维度（top right bottom left，如 inset 内缩 / margin）序列化为 CSS box 简写：
/// 全等→1 值；top==bottom && left==right→2 值；left==right→3 值；否则 4 值（同 margin 语法）。
/// 比较按序列化后的字符串（等价 LengthValue 必序列化等价）。
fn box_4_to_css(
    top: &LengthValue,
    right: &LengthValue,
    bottom: &LengthValue,
    left: &LengthValue,
    font_size_px: f64,
) -> String {
    let t = length_to_css(top, font_size_px);
    let r = length_to_css(right, font_size_px);
    let b = length_to_css(bottom, font_size_px);
    let l = length_to_css(left, font_size_px);
    if t == r && r == b && b == l {
        t
    } else if t == b && r == l {
        format!("{t} {r}")
    } else if r == l {
        format!("{t} {r} {b}")
    } else {
        format!("{t} {r} {b} {l}")
    }
}

/// content：按 CSS Generated Content 计算值序列化（::before/::after 生成内容）。
///
/// `Normal`/`None`→关键字；`String`→CSS 双引号串（[`css_string_to_css`]）；`Attr`→`attr(name)`；
/// `Counter`/`Counters`→计数器函数；`Url`→`url(...)`；`List`→多 component value 空格连接。
fn content_to_css(c: &ContentComputedValue) -> String {
    use ContentComputedValue as C;
    match c {
        C::Normal => "normal".to_string(),
        C::None => "none".to_string(),
        C::String(s) => css_string_to_css(s),
        C::Attr(name) => format!("attr({name})"),
        C::Counter { name, style } => counter_fn_to_css(name, style.as_deref()),
        C::Counters { name, separator, style } => counters_fn_to_css(name, separator, style.as_deref()),
        C::Url(u) => format!("url({u})"),
        C::List(items) => items.iter().map(content_list_item_to_css).collect::<Vec<_>>().join(" "),
    }
}

/// 序列化 content 多 component value 序列的单项 [`ContentListItem`]（Str/Counter/Counters）。
fn content_list_item_to_css(item: &ContentListItem) -> String {
    use ContentListItem as I;
    match item {
        I::Str(s) => css_string_to_css(s),
        I::Counter { name, style } => counter_fn_to_css(name, style.as_deref()),
        I::Counters { name, separator, style } => counters_fn_to_css(name, separator, style.as_deref()),
    }
}

/// `counter(name[, style])` 函数序列化。
fn counter_fn_to_css(name: &str, style: Option<&str>) -> String {
    match style {
        Some(s) => format!("counter({name}, {s})"),
        None => format!("counter({name})"),
    }
}

/// `counters(name, "sep"[, style])` 函数序列化（separator 经 CSS 引号串化）。
fn counters_fn_to_css(name: &str, separator: &str, style: Option<&str>) -> String {
    let sep = css_string_to_css(separator);
    match style {
        Some(s) => format!("counters({name}, {sep}, {s})"),
        None => format!("counters({name}, {sep})"),
    }
}

/// 把字符串字面量序列化为 CSS 双引号串：转义 `\` / `"` / 换行（CSS escape `\A `）。
fn css_string_to_css(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\A "),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// getComputedStyle 专用后处理：把 `bolder`/`lighter` font-weight 按父链 resolved 绝对值解析
///（CSS Fonts 3 §5.2 计算值语义；Chromium getComputedStyle 返绝对数）。
///
/// 自顶向下遍历文档树（栈式 DFS，父先于子出栈处理），保证解析子节点时父节点已解析为绝对值。
/// 仅改本函数返回的 styles 副本——style-system computed 值仍保关键字供 paint 二值 want_bold，
/// 故 render/reftest 零影响，只有 getComputedStyle 输出对齐 Chromium。
fn resolve_font_weight_bolder_lighter(doc: &Document, styles: &mut HashMap<NodeId, ComputedStyle>) {
    let mut stack = vec![doc.root()];
    while let Some(id) = stack.pop() {
        let children = doc.child_nodes(id);
        // 先以 immutable borrow 决定是否解析 + 方向（释放后再 get_mut 写）。
        let new_weight = styles.get(&id).and_then(|s| match s.font_weight {
            FontWeightValue::Bolder => Some(FontWeightValue::Absolute(bolder_of(parent_font_weight_base(
                doc, styles, id,
            )))),
            FontWeightValue::Lighter => Some(FontWeightValue::Absolute(lighter_of(parent_font_weight_base(
                doc, styles, id,
            )))),
            _ => None,
        });
        if let Some(w) = new_weight
            && let Some(style) = styles.get_mut(&id)
        {
            style.font_weight = w;
        }
        stack.extend(children);
    }
}

/// 元素父节点 resolved font-weight 的绝对基数（Normal→400 / Bold→700 / Absolute(n)→n）。
/// 根元素无父 → 默认 normal = 400。
fn parent_font_weight_base(doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, id: NodeId) -> u16 {
    doc.parent_node(id)
        .and_then(|p| styles.get(&p))
        .map(|s| font_weight_base_absolute(&s.font_weight))
        .unwrap_or(400)
}

/// font-weight 的绝对基数。Bolder/Lighter 防御性返 400（父经自顶向下解析后不应出现）。
fn font_weight_base_absolute(w: &FontWeightValue) -> u16 {
    match w {
        FontWeightValue::Absolute(n) => *n,
        FontWeightValue::Normal => 400,
        FontWeightValue::Bold => 700,
        FontWeightValue::Bolder | FontWeightValue::Lighter => 400,
    }
}

/// CSS Fonts 3 §5.2 `bolder` 映射表（标准 100-900 经此表；非标准值按区间合理映射）。
fn bolder_of(parent: u16) -> u16 {
    if parent < 400 {
        400
    } else if parent < 600 {
        700
    } else {
        900
    }
}

/// CSS Fonts 3 §5.2 `lighter` 映射表。
fn lighter_of(parent: u16) -> u16 {
    if parent < 600 {
        100
    } else if parent < 800 {
        400
    } else {
        700
    }
}

/// background-position：CSS Backgrounds `<bg-position>#` 多层序列化（逗号分隔层）。
///
/// 关键字解析为百分比（center 50% / left 0% / right 100% / top 0% / bottom 100%），对齐 Chromium
/// getComputedStyle（WPT background-computed.html）；单关键字/单长度按轴展开（缺省轴 = center 50%）。
fn background_position_to_css(layers: &[BackgroundPositionComputedValue]) -> String {
    if layers.is_empty() {
        return String::new();
    }
    layers
        .iter()
        .map(bg_position_layer_to_css)
        .collect::<Vec<_>>()
        .join(", ")
}

/// 序列化单层 `<bg-position>`。单值按轴展开（缺省轴 center 50%）；TwoValue 两轴；EdgeOffset 边缘+偏移。
fn bg_position_layer_to_css(p: &BackgroundPositionComputedValue) -> String {
    use BackgroundPositionComputedValue as P;
    match p {
        // 单关键字：按轴展开（水平关键字→垂直 center 50%；垂直关键字→水平 center 50%）。
        P::Center => "50% 50%".to_string(),
        P::Left => "0% 50%".to_string(),
        P::Right => "100% 50%".to_string(),
        P::Top => "50% 0%".to_string(),
        P::Bottom => "50% 100%".to_string(),
        // 单长度/百分比：水平 = 值，垂直缺省 center 50%。
        P::Length(f) => format!("{} 50%", format_num(*f as f64, "px")),
        P::Percent(f) => format!("{} 50%", format_num(*f as f64, "%")),
        // % calc 无容器尺寸不可解析（非 % calc 已 resolve）→ ''。
        P::Calc(_) => String::new(),
        P::TwoValue(h, v) => format!("{} {}", bg_position_axis_to_css(h), bg_position_axis_to_css(v)),
        P::EdgeOffset(edge, offset) => {
            format!("{} {}", bg_edge_to_str(edge), bg_position_axis_to_css(offset))
        }
    }
}

/// 序列化 `<bg-position>` 单轴值（关键字→% / Length→px / Percent→%）。TwoValue/EdgeOffset 不应出现于轴位。
fn bg_position_axis_to_css(v: &BackgroundPositionComputedValue) -> String {
    use BackgroundPositionComputedValue as P;
    match v {
        P::Center => "50%".to_string(),
        P::Left => "0%".to_string(),
        P::Right => "100%".to_string(),
        P::Top => "0%".to_string(),
        P::Bottom => "100%".to_string(),
        P::Length(f) => format_num(*f as f64, "px"),
        P::Percent(f) => format_num(*f as f64, "%"),
        P::Calc(_) => String::new(),
        // 防御：轴位不应嵌套 TwoValue/EdgeOffset。
        P::TwoValue(..) | P::EdgeOffset(..) => String::new(),
    }
}

/// `<bg-position>` 边缘关键字（3/4 值 EdgeOffset 的 side）。
fn bg_edge_to_str(e: &BackgroundEdge) -> &'static str {
    match e {
        BackgroundEdge::Left => "left",
        BackgroundEdge::Right => "right",
        BackgroundEdge::Top => "top",
        BackgroundEdge::Bottom => "bottom",
    }
}

/// background-size：CSS Backgrounds `<bg-size>#` 多层序列化（逗号分隔）。
/// Auto/Cover/Contain→关键字；Length→px；Percent→%。
fn background_size_to_css(layers: &[BackgroundSizeComputedValue]) -> String {
    if layers.is_empty() {
        return String::new();
    }
    layers
        .iter()
        .map(|s| match s {
            BackgroundSizeComputedValue::Auto => "auto".to_string(),
            BackgroundSizeComputedValue::Cover => "cover".to_string(),
            BackgroundSizeComputedValue::Contain => "contain".to_string(),
            BackgroundSizeComputedValue::Length(f) => format_num(*f as f64, "px"),
            BackgroundSizeComputedValue::Percent(f) => format_num(*f as f64, "%"),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// background-repeat：CSS Backgrounds `<repeat-style>#` 多层序列化（逗号分隔）。
fn background_repeat_to_css(layers: &[BackgroundRepeatComputedValue]) -> String {
    if layers.is_empty() {
        return String::new();
    }
    layers
        .iter()
        .map(|r| match r {
            BackgroundRepeatComputedValue::Repeat => "repeat",
            BackgroundRepeatComputedValue::RepeatX => "repeat-x",
            BackgroundRepeatComputedValue::RepeatY => "repeat-y",
            BackgroundRepeatComputedValue::NoRepeat => "no-repeat",
            BackgroundRepeatComputedValue::Space => "space",
            BackgroundRepeatComputedValue::Round => "round",
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// background-attachment：CSS Backgrounds `<attachment>` 单值序列化。
/// Scroll/Fixed/Local → scroll/fixed/local。ZeroWeb 存单值（非多层 Vec），与解析侧一致。
fn background_attachment_to_css(a: &BackgroundAttachmentComputedValue) -> String {
    match a {
        BackgroundAttachmentComputedValue::Scroll => "scroll",
        BackgroundAttachmentComputedValue::Fixed => "fixed",
        BackgroundAttachmentComputedValue::Local => "local",
    }
    .to_string()
}

/// background-clip：CSS Backgrounds `<visual-box>` 单值序列化。
/// border-box/padding-box/content-box/text。ZeroWeb 存单值（非多层 Vec）。
fn background_clip_to_css(c: &BackgroundClipComputedValue) -> String {
    match c {
        BackgroundClipComputedValue::BorderBox => "border-box",
        BackgroundClipComputedValue::PaddingBox => "padding-box",
        BackgroundClipComputedValue::ContentBox => "content-box",
        BackgroundClipComputedValue::Text => "text",
    }
    .to_string()
}

/// background-origin：CSS Backgrounds `<geometry-box>` 单值序列化。
/// padding-box/border-box/content-box。ZeroWeb 存单值（非多层 Vec）。
fn background_origin_to_css(o: &BackgroundOriginComputedValue) -> String {
    match o {
        BackgroundOriginComputedValue::PaddingBox => "padding-box",
        BackgroundOriginComputedValue::BorderBox => "border-box",
        BackgroundOriginComputedValue::ContentBox => "content-box",
    }
    .to_string()
}

/// align-content：CSS Box Alignment `<content-distribution>` 单值序列化。
/// Chromium getComputedStyle 初值 = normal（CSS Align 3）。ZeroWeb computed 默认 Normal。
fn align_content_to_css(a: &AlignContentValue) -> String {
    match a {
        AlignContentValue::Auto => "auto",
        AlignContentValue::Normal => "normal",
        AlignContentValue::Start => "start",
        AlignContentValue::End => "end",
        AlignContentValue::Center => "center",
        AlignContentValue::Stretch => "stretch",
        AlignContentValue::Baseline => "baseline",
        AlignContentValue::SpaceBetween => "space-between",
        AlignContentValue::SpaceAround => "space-around",
        AlignContentValue::SpaceEvenly => "space-evenly",
    }
    .to_string()
}

/// justify-items：CSS Box Alignment 单值序列化。初值 = normal（CSS Align 3）。
fn justify_items_to_css(j: &JustifyItemsValue) -> String {
    match j {
        JustifyItemsValue::Auto => "auto",
        JustifyItemsValue::Normal => "normal",
        JustifyItemsValue::Start => "start",
        JustifyItemsValue::End => "end",
        JustifyItemsValue::Center => "center",
        JustifyItemsValue::Stretch => "stretch",
        JustifyItemsValue::Baseline => "baseline",
        JustifyItemsValue::Left => "left",
        JustifyItemsValue::Right => "right",
    }
    .to_string()
}

/// justify-self：CSS Box Alignment 单值序列化。初值 = auto（CSS Align 3）。
fn justify_self_to_css(j: &JustifySelfValue) -> String {
    match j {
        JustifySelfValue::Auto => "auto",
        JustifySelfValue::Normal => "normal",
        JustifySelfValue::Start => "start",
        JustifySelfValue::End => "end",
        JustifySelfValue::Center => "center",
        JustifySelfValue::Stretch => "stretch",
        JustifySelfValue::Baseline => "baseline",
        JustifySelfValue::Left => "left",
        JustifySelfValue::Right => "right",
    }
    .to_string()
}

/// word-break：CSS Text `<word-break>` 单值序列化。初值 normal。
fn word_break_to_css(w: &WordBreakValue) -> String {
    match w {
        WordBreakValue::Normal => "normal",
        WordBreakValue::BreakAll => "break-all",
        WordBreakValue::KeepAll => "keep-all",
        WordBreakValue::BreakWord => "break-word",
    }
    .to_string()
}

/// overflow-wrap：CSS Text `<overflow-wrap>` 单值序列化。初值 normal。
fn overflow_wrap_to_css(w: &OverflowWrapValue) -> String {
    match w {
        OverflowWrapValue::Normal => "normal",
        OverflowWrapValue::BreakWord => "break-word",
        OverflowWrapValue::Anywhere => "anywhere",
    }
    .to_string()
}

/// hyphens：CSS Text 单值序列化。ZeroWeb 初值 None（diverge：CSS 规范初值 manual，Chromium 返 manual）。
fn hyphens_to_css(h: &HyphensComputedValue) -> String {
    match h {
        HyphensComputedValue::None => "none",
        HyphensComputedValue::Manual => "manual",
        HyphensComputedValue::Auto => "auto",
    }
    .to_string()
}

/// line-break：CSS Text 单值序列化。初值 auto。
fn line_break_to_css(l: &LineBreakValue) -> String {
    match l {
        LineBreakValue::Auto => "auto",
        LineBreakValue::Loose => "loose",
        LineBreakValue::Normal => "normal",
        LineBreakValue::Strict => "strict",
        LineBreakValue::Anywhere => "anywhere",
    }
    .to_string()
}

/// vertical-align：CSS 行内/表格单元格垂直对齐单值序列化。初值 baseline。
/// ZeroWeb enum 仅关键字（无 `<length>`/`<percentage>` 变体）——diverge：CSS 规范允许，Chromium 对
/// `5px`/`50%` 返 used 值，ZeroWeb 无 length 变体存储故无法表达（解析层丢弃，本序列化如实反映枚举）。
fn vertical_align_to_css(v: &VerticalAlignValue) -> String {
    match v {
        VerticalAlignValue::Baseline => "baseline",
        VerticalAlignValue::Top => "top",
        VerticalAlignValue::Middle => "middle",
        VerticalAlignValue::Bottom => "bottom",
        VerticalAlignValue::TextTop => "text-top",
        VerticalAlignValue::TextBottom => "text-bottom",
        VerticalAlignValue::Sub => "sub",
        VerticalAlignValue::Super => "super",
    }
    .to_string()
}

/// unicode-bidi：CSS Writing Modes 双向文本算法单值序列化。初值 normal。
fn unicode_bidi_to_css(u: &UnicodeBidiValue) -> String {
    match u {
        UnicodeBidiValue::Normal => "normal",
        UnicodeBidiValue::Embed => "embed",
        UnicodeBidiValue::Isolate => "isolate",
        UnicodeBidiValue::BidiOverride => "bidi-override",
        UnicodeBidiValue::IsolateOverride => "isolate-override",
        UnicodeBidiValue::Plaintext => "plaintext",
    }
    .to_string()
}

/// empty-cells：CSS 表格空单元格边框单值序列化。初值 show。
fn empty_cells_to_css(e: &EmptyCellsComputedValue) -> String {
    match e {
        EmptyCellsComputedValue::Show => "show",
        EmptyCellsComputedValue::Hide => "hide",
    }
    .to_string()
}

/// 序列化单个 [`TransformFunction`]。translate 类长度为 px（混合百分比分支保 `%`）；rotate/skew
/// 角度为 `deg`；scale 系数无单位；matrix 6 分量无单位。
fn transform_function_to_css(f: &TransformFunction) -> String {
    use TransformFunction as Tf;
    match f {
        Tf::Translate(tx, ty) => {
            format!("translate({}, {})", format_num(*tx, "px"), format_num(*ty, "px"))
        }
        Tf::TranslateMixed(tx, txp, ty, typ) => {
            format!("translate({}, {})", mixed_len(*tx, *txp), mixed_len(*ty, *typ))
        }
        Tf::TranslateXMixed(v, pct) => format!("translateX({})", mixed_len(*v, *pct)),
        Tf::TranslateYMixed(v, pct) => format!("translateY({})", mixed_len(*v, *pct)),
        Tf::TranslateX(v) => format!("translateX({})", format_num(*v, "px")),
        Tf::TranslateY(v) => format!("translateY({})", format_num(*v, "px")),
        Tf::Rotate(deg) => format!("rotate({}deg)", format_num(*deg, "")),
        Tf::Scale(sx, sy) => match sy {
            Some(sy) => format!("scale({}, {})", format_num(*sx, ""), format_num(*sy, "")),
            None => format!("scale({})", format_num(*sx, "")),
        },
        Tf::ScaleX(sx) => format!("scaleX({})", format_num(*sx, "")),
        Tf::ScaleY(sy) => format!("scaleY({})", format_num(*sy, "")),
        Tf::Skew(ax, ay) => match ay {
            Some(ay) => format!("skew({}deg, {}deg)", format_num(*ax, ""), format_num(*ay, "")),
            None => format!("skew({}deg)", format_num(*ax, "")),
        },
        Tf::RotateX(deg) => format!("rotateX({}deg)", format_num(*deg, "")),
        Tf::RotateY(deg) => format!("rotateY({}deg)", format_num(*deg, "")),
        Tf::RotateZ(deg) => format!("rotateZ({}deg)", format_num(*deg, "")),
        Tf::Translate3d(tx, ty, tz) => format!(
            "translate3d({}, {}, {})",
            format_num(*tx, "px"),
            format_num(*ty, "px"),
            format_num(*tz, "px")
        ),
        Tf::Scale3d(sx, sy, sz) => format!(
            "scale3d({}, {}, {})",
            format_num(*sx, ""),
            format_num(*sy, ""),
            format_num(*sz, "")
        ),
        Tf::Rotate3d(x, y, z, angle) => format!(
            "rotate3d({}, {}, {}, {}deg)",
            format_num(*x, ""),
            format_num(*y, ""),
            format_num(*z, ""),
            format_num(*angle, "")
        ),
        Tf::Perspective(len) => format!("perspective({})", format_num(*len, "px")),
        Tf::Matrix(a, b, c, d, e, f) => format!(
            "matrix({}, {}, {}, {}, {}, {})",
            format_num(*a, ""),
            format_num(*b, ""),
            format_num(*c, ""),
            format_num(*d, ""),
            format_num(*e, ""),
            format_num(*f, "")
        ),
    }
}

/// translate 类混合长度：`is_pct` → `N%`，否则 `Npx`。
fn mixed_len(v: f64, is_pct: bool) -> String {
    if is_pct {
        format_num(v, "%")
    } else {
        format_num(v, "px")
    }
}
