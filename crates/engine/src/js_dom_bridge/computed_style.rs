//! getComputedStyle 计算与序列化——把 [`zero_style_system::ComputedStyle`] 的单属性序列化为
//! CSS 字符串（kebab-case 属性名）。从 `js_dom_bridge` 拆出（R2709）以控制主文件行数。
//!
//! 覆盖：display/position/visibility/opacity + 颜色族 + 长度族 + 关键字/枚举族；未覆盖属性返 ''。

use std::collections::HashMap;

use zero_css_parser::values::{
    BoxSizingValue, ClearValue, DisplayValue, FloatValue, FontStyleValue, FontWeightValue, LengthValue,
    OverflowValue, PositionValue, VisibilityValue,
};
use zero_style_system::{
    BorderCollapseValue, BorderStyleValue, CaptionSideValue, ComputedStyle, CursorValue, DirectionValue,
    LineHeightValue, OutlineStyleValue, StyleSystem, TableLayoutValue, TextAlignValue, TextOverflowValue,
    TextTransformValue, WhiteSpaceValue, ZIndexValue,
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
///   direction/border-collapse/table-layout/caption-side/border-*-style/outline-style）。未覆盖属性返 ''。
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
