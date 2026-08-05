//! getComputedStyle 计算与序列化——把 [`zero_style_system::ComputedStyle`] 的单属性序列化为
//! CSS 字符串（kebab-case 属性名）。从 `js_dom_bridge` 拆出（R2709）以控制主文件行数。
//!
//! 覆盖：display/position/visibility/opacity + 颜色族 + 长度族 + 关键字/枚举族；未覆盖属性返 ''。

use std::collections::HashMap;

use zero_css_parser::values::{
    AlignmentValue, BoxSizingValue, ClearValue, DisplayValue, FlexDirectionValue, FlexWrapValue, FloatValue,
    FontStyleValue, FontWeightValue, LengthValue, ListStylePositionValue, ListStyleTypeValue, OverflowValue,
    PositionValue, TransformFunction, TransformValue, VisibilityValue,
};
use zero_style_system::{
    BorderCollapseValue, BorderStyleValue, CaptionSideValue, ComputedStyle, CursorValue, DirectionValue,
    FlexBasisValue, IsolationValue, LineHeightValue, MixBlendModeComputedValue, ObjectFitComputedValue,
    OutlineStyleValue, PointerEventsValue, StyleSystem, TableLayoutValue, TextAlignValue, TextOverflowValue,
    TextTransformValue, UserSelectValue, WhiteSpaceValue, WritingModeValue, ZIndexValue,
};
use zero_dom::{Document, NodeId, parse_html};

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
    let styles = sys.compute_styles(&doc, &sheets);
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
///   （flex-grow/flex-shrink/order/flex-basis/aspect-ratio）。未覆盖属性返 ''。
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
        "color" => color_to_css(&crate::resolve_color_current(
            &style.color,
            element_color,
        )),
        "background-color" => color_to_css(&crate::resolve_color_current(
            &style.background_color,
            element_color,
        )),
        "border-top-color" => color_to_css(&crate::resolve_color_current(
            &style.border_top_color,
            element_color,
        )),
        "border-right-color" => color_to_css(&crate::resolve_color_current(
            &style.border_right_color,
            element_color,
        )),
        "border-bottom-color" => color_to_css(&crate::resolve_color_current(
            &style.border_bottom_color,
            element_color,
        )),
        "border-left-color" => color_to_css(&crate::resolve_color_current(
            &style.border_left_color,
            element_color,
        )),
        "outline-color" => color_to_css(&crate::resolve_color_current(
            &style.outline_color,
            element_color,
        )),
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
        "border-top-width" => {
            border_width_to_css(&style.border_top_width, &style.border_top_style, font_size_px)
        }
        "border-right-width" => border_width_to_css(
            &style.border_right_width,
            &style.border_right_style,
            font_size_px,
        ),
        "border-bottom-width" => border_width_to_css(
            &style.border_bottom_width,
            &style.border_bottom_style,
            font_size_px,
        ),
        "border-left-width" => border_width_to_css(
            &style.border_left_width,
            &style.border_left_style,
            font_size_px,
        ),
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
        TransformValue::List(fns) => fns
            .iter()
            .map(transform_function_to_css)
            .collect::<Vec<_>>()
            .join(" "),
    }
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
