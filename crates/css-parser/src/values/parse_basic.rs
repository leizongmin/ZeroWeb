//! CSS 基础属性解析函数（布局、文本、字体等）。

use super::types::*;

pub fn parse_length(value: &str) -> Option<LengthValue> {
    let value = value.trim();

    // 处理 auto 关键字
    if value.eq_ignore_ascii_case("auto") {
        return Some(LengthValue::Auto);
    }

    // 处理 border-width 关键字（CSS 2.1 §8.5.1）：
    // thin = 1px, medium = 3px, thick = 5px
    // 这些关键字在 border 简写展开后作为 border-*--width 的值出现
    if value.eq_ignore_ascii_case("thin") {
        return Some(LengthValue::Px(1.0));
    }
    if value.eq_ignore_ascii_case("medium") {
        return Some(LengthValue::Px(3.0));
    }
    if value.eq_ignore_ascii_case("thick") {
        return Some(LengthValue::Px(5.0));
    }

    // 处理 min-content/max-content 关键字
    if value.eq_ignore_ascii_case("min-content") {
        return Some(LengthValue::MinContent);
    }
    if value.eq_ignore_ascii_case("max-content") {
        return Some(LengthValue::MaxContent);
    }

    // 处理 fit-content() 函数
    if value.starts_with("fit-content(") && value.ends_with(')') {
        let inner = &value["fit-content(".len()..value.len() - 1];
        let inner = inner.trim();
        // fit-content() 不接受空参数
        if inner.is_empty() {
            return None;
        }
        let arg = parse_length(inner)?;
        return Some(LengthValue::FitContent(Box::new(arg)));
    }

    // 从字符串末尾扫描，找到单位部分的起始位置。
    // 单位部分由字母组成（可能以 '%' 结尾）；数字部分在单位之前。
    // 这样可以正确处理科学计数法（如 "1e2px"），因为 'e' 在数字部分内。
    let unit_start = find_unit_start(value);

    let num_str = &value[..unit_start];
    let unit = &value[unit_start..];

    let num: f64 = num_str.parse().ok()?;

    match unit {
        "px" => Some(LengthValue::Px(num)),
        "em" => Some(LengthValue::Em(num)),
        "rem" => Some(LengthValue::Rem(num)),
        "vh" => Some(LengthValue::Vh(num)),
        "vw" => Some(LengthValue::Vw(num)),
        "vmin" => Some(LengthValue::Vmin(num)),
        "vmax" => Some(LengthValue::Vmax(num)),
        "ch" => Some(LengthValue::Ch(num)),
        "%" => Some(LengthValue::Percentage(num)),
        // CSS 绝对长度单位（按 CSS 规范 96 DPI 转换为 px）
        "in" => Some(LengthValue::Px(num * 96.0)),
        "pt" => Some(LengthValue::Px(num * 96.0 / 72.0)),
        "pc" => Some(LengthValue::Px(num * 96.0 / 6.0)), // 1pc = 12pt
        "cm" => Some(LengthValue::Px(num * 96.0 / 2.54)),
        "mm" => Some(LengthValue::Px(num * 96.0 / 25.4)),
        "Q" => Some(LengthValue::Px(num * 96.0 / 101.6)), // 1Q = 1/4mm
        // Per CSS spec, a bare zero without units is a valid length (0px).
        "" if num == 0.0 => Some(LengthValue::Px(0.0)),
        _ => None,
    }
}

/// 从字符串末尾找到单位部分的起始索引。
///
/// 从右向左扫描：跳过 '%'（如果有），然后跳过连续的字母字符，
/// 剩下的就是数字部分的结束位置。
fn find_unit_start(s: &str) -> usize {
    let bytes = s.as_bytes();
    let mut i = bytes.len();

    // 跳过末尾的 '%'
    if i > 0 && bytes[i - 1] == b'%' {
        i -= 1;
        return i;
    }

    // 从末尾向前跳过连续的 ASCII 字母（单位名）
    while i > 0 && bytes[i - 1].is_ascii_alphabetic() {
        i -= 1;
    }

    i
}

/// 解析 CSS display 属性值。
pub fn parse_display(value: &str) -> Option<DisplayValue> {
    match value.trim() {
        "block" => Some(DisplayValue::Block),
        "inline" => Some(DisplayValue::Inline),
        "inline-block" => Some(DisplayValue::InlineBlock),
        "flex" => Some(DisplayValue::Flex),
        "inline-flex" => Some(DisplayValue::InlineFlex),
        "grid" => Some(DisplayValue::Grid),
        "inline-grid" => Some(DisplayValue::InlineGrid),
        "none" => Some(DisplayValue::None),
        "contents" => Some(DisplayValue::Contents),
        "flow" => Some(DisplayValue::Flow),
        "flow-root" => Some(DisplayValue::FlowRoot),
        "list-item" => Some(DisplayValue::ListItem),
        "table" => Some(DisplayValue::Table),
        "inline-table" => Some(DisplayValue::InlineTable),
        "table-row" => Some(DisplayValue::TableRow),
        "table-cell" => Some(DisplayValue::TableCell),
        "table-caption" => Some(DisplayValue::TableCaption),
        "table-column" => Some(DisplayValue::TableColumn),
        "table-column-group" => Some(DisplayValue::TableColumnGroup),
        "table-row-group" => Some(DisplayValue::TableRowGroup),
        "table-header-group" => Some(DisplayValue::TableHeaderGroup),
        "table-footer-group" => Some(DisplayValue::TableFooterGroup),
        _ => None,
    }
}

/// 解析 CSS position 属性值。
pub fn parse_position(value: &str) -> Option<PositionValue> {
    match value.trim() {
        "static" => Some(PositionValue::Static),
        "relative" => Some(PositionValue::Relative),
        "absolute" => Some(PositionValue::Absolute),
        "fixed" => Some(PositionValue::Fixed),
        "sticky" => Some(PositionValue::Sticky),
        _ => None,
    }
}

/// 解析 CSS overflow 属性值。
pub fn parse_overflow(value: &str) -> Option<OverflowValue> {
    match value.trim() {
        "visible" => Some(OverflowValue::Visible),
        "hidden" => Some(OverflowValue::Hidden),
        "scroll" => Some(OverflowValue::Scroll),
        "auto" => Some(OverflowValue::Auto),
        "clip" => Some(OverflowValue::Clip),
        _ => None,
    }
}

/// 解析 CSS float 属性值。
pub fn parse_float(value: &str) -> Option<FloatValue> {
    match value.trim().to_lowercase().as_str() {
        "none" => Some(FloatValue::None),
        "left" => Some(FloatValue::Left),
        "right" => Some(FloatValue::Right),
        "inline-start" => Some(FloatValue::InlineStart),
        "inline-end" => Some(FloatValue::InlineEnd),
        _ => None,
    }
}

/// 解析 CSS clear 属性值。
pub fn parse_clear(value: &str) -> Option<ClearValue> {
    match value.trim().to_lowercase().as_str() {
        "none" => Some(ClearValue::None),
        "left" => Some(ClearValue::Left),
        "right" => Some(ClearValue::Right),
        "both" => Some(ClearValue::Both),
        "inline-start" => Some(ClearValue::InlineStart),
        "inline-end" => Some(ClearValue::InlineEnd),
        _ => None,
    }
}

/// 解析 CSS list-style-type 属性值。
pub fn parse_list_style_type(value: &str) -> Option<ListStyleTypeValue> {
    match value.trim().to_lowercase().as_str() {
        "disc" => Some(ListStyleTypeValue::Disc),
        "circle" => Some(ListStyleTypeValue::Circle),
        "square" => Some(ListStyleTypeValue::Square),
        "decimal" => Some(ListStyleTypeValue::Decimal),
        "decimal-leading-zero" => Some(ListStyleTypeValue::DecimalLeadingZero),
        "lower-roman" => Some(ListStyleTypeValue::LowerRoman),
        "upper-roman" => Some(ListStyleTypeValue::UpperRoman),
        "lower-alpha" | "lower-latin" => Some(ListStyleTypeValue::LowerAlpha),
        "upper-alpha" | "upper-latin" => Some(ListStyleTypeValue::UpperAlpha),
        "none" => Some(ListStyleTypeValue::None),
        _ => None,
    }
}

/// 解析 CSS list-style-position 属性值。
pub fn parse_list_style_position(value: &str) -> Option<ListStylePositionValue> {
    match value.trim().to_lowercase().as_str() {
        "outside" => Some(ListStylePositionValue::Outside),
        "inside" => Some(ListStylePositionValue::Inside),
        _ => None,
    }
}

/// 解析 CSS flex-direction 属性值。
pub fn parse_flex_direction(value: &str) -> Option<FlexDirectionValue> {
    match value.trim() {
        "row" => Some(FlexDirectionValue::Row),
        "row-reverse" => Some(FlexDirectionValue::RowReverse),
        "column" => Some(FlexDirectionValue::Column),
        "column-reverse" => Some(FlexDirectionValue::ColumnReverse),
        _ => None,
    }
}

/// 解析 CSS flex-wrap 属性值。
pub fn parse_flex_wrap(value: &str) -> Option<FlexWrapValue> {
    match value.trim() {
        "nowrap" => Some(FlexWrapValue::Nowrap),
        "wrap" => Some(FlexWrapValue::Wrap),
        "wrap-reverse" => Some(FlexWrapValue::WrapReverse),
        _ => None,
    }
}

/// 解析 CSS justify-content / align-items / align-self 属性值。
pub fn parse_alignment(value: &str) -> Option<AlignmentValue> {
    match value.trim() {
        "auto" => Some(AlignmentValue::Auto),
        "flex-start" => Some(AlignmentValue::FlexStart),
        "flex-end" => Some(AlignmentValue::FlexEnd),
        "center" => Some(AlignmentValue::Center),
        "space-between" => Some(AlignmentValue::SpaceBetween),
        "space-around" => Some(AlignmentValue::SpaceAround),
        "space-evenly" => Some(AlignmentValue::SpaceEvenly),
        "stretch" => Some(AlignmentValue::Stretch),
        "start" => Some(AlignmentValue::Start),
        "end" => Some(AlignmentValue::End),
        "baseline" => Some(AlignmentValue::Baseline),
        _ => None,
    }
}

/// 解析 CSS box-sizing 属性值。
pub fn parse_box_sizing(value: &str) -> Option<BoxSizingValue> {
    match value.trim() {
        "content-box" => Some(BoxSizingValue::ContentBox),
        "border-box" => Some(BoxSizingValue::BorderBox),
        _ => None,
    }
}

/// 解析 CSS visibility 属性值。
pub fn parse_visibility(value: &str) -> Option<VisibilityValue> {
    match value.trim() {
        "visible" => Some(VisibilityValue::Visible),
        "hidden" => Some(VisibilityValue::Hidden),
        "collapse" => Some(VisibilityValue::Collapse),
        _ => None,
    }
}

/// 解析 CSS word-break 属性值。
pub fn parse_word_break(value: &str) -> Option<WordBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(WordBreakValue::Normal),
        "break-all" => Some(WordBreakValue::BreakAll),
        "keep-all" => Some(WordBreakValue::KeepAll),
        "break-word" => Some(WordBreakValue::BreakWord),
        _ => None,
    }
}

/// 解析 CSS writing-mode 属性值。
pub fn parse_writing_mode(value: &str) -> Option<WritingModeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "horizontal-tb" => Some(WritingModeValue::HorizontalTb),
        "vertical-rl" => Some(WritingModeValue::VerticalRl),
        "vertical-lr" => Some(WritingModeValue::VerticalLr),
        // R1785：sideways-rl/lr 规范化为 vertical-rl/lr（block-flow 等价，见 color.rs 注释）。
        "sideways-rl" => Some(WritingModeValue::VerticalRl),
        "sideways-lr" => Some(WritingModeValue::VerticalLr),
        _ => None,
    }
}

/// 解析 CSS text-decoration-line 值。
pub fn parse_text_decoration_line(value: &str) -> Option<TextDecorationLineValue> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(TextDecorationLineValue::None),
        "underline" => Some(TextDecorationLineValue::Underline),
        "overline" => Some(TextDecorationLineValue::Overline),
        "line-through" => Some(TextDecorationLineValue::LineThrough),
        "blink" => Some(TextDecorationLineValue::Blink),
        _ => None,
    }
}

/// 解析 CSS text-decoration-style 值。
pub fn parse_text_decoration_style(value: &str) -> Option<TextDecorationStyleValue> {
    match value.to_ascii_lowercase().as_str() {
        "solid" => Some(TextDecorationStyleValue::Solid),
        "double" => Some(TextDecorationStyleValue::Double),
        "dotted" => Some(TextDecorationStyleValue::Dotted),
        "dashed" => Some(TextDecorationStyleValue::Dashed),
        "wavy" => Some(TextDecorationStyleValue::Wavy),
        _ => None,
    }
}

/// 解析 CSS text-transform 值。
pub fn parse_text_transform(value: &str) -> Option<TextTransformValue> {
    match value.to_ascii_lowercase().as_str() {
        "none" => Some(TextTransformValue::None),
        "uppercase" => Some(TextTransformValue::Uppercase),
        "lowercase" => Some(TextTransformValue::Lowercase),
        "capitalize" => Some(TextTransformValue::Capitalize),
        _ => None,
    }
}

/// 解析 CSS text-indent 属性值。
///
/// 支持长度值（如 `2em`、`20px`）和百分比值（如 `10%`）。
/// 不支持 `auto` 关键字。
pub fn parse_text_indent(value: &str) -> Option<LengthValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return None;
    }
    parse_length(v)
}

/// 解析 CSS letter-spacing / word-spacing 值。
/// "normal" 映射为 LengthValue::Px(0.0)。
pub fn parse_spacing(value: &str) -> Option<LengthValue> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("normal") {
        return Some(LengthValue::Px(0.0));
    }
    parse_length(v)
}

/// 解析 CSS font-weight 属性值。
pub fn parse_font_weight(value: &str) -> Option<FontWeightValue> {
    match value.trim() {
        "bold" => Some(FontWeightValue::Bold),
        "normal" => Some(FontWeightValue::Normal),
        "bolder" => Some(FontWeightValue::Bolder),
        "lighter" => Some(FontWeightValue::Lighter),
        s => {
            let w: u16 = s.parse().ok()?;
            if (100..=900).contains(&w) {
                Some(FontWeightValue::Absolute(w))
            } else {
                None
            }
        }
    }
}

/// 解析 CSS font-style 属性值。
pub fn parse_font_style(value: &str) -> Option<FontStyleValue> {
    // CSS 关键字大小写不敏感（NORMAL/Italic/OBLIQUE ≡ normal/italic/oblique）。归一化小写后匹配。
    let value = value.trim().to_ascii_lowercase();
    if value == "normal" {
        Some(FontStyleValue::Normal)
    } else if value == "italic" {
        Some(FontStyleValue::Italic)
    } else if value.starts_with("oblique") {
        let angle_str = value.strip_prefix("oblique")?.trim();
        if angle_str.is_empty() {
            Some(FontStyleValue::Oblique(None))
        } else {
            // 处理 "(angle)" 或 "(angledeg)" 形式
            let angle_str = angle_str
                .strip_prefix('(')
                .unwrap_or(angle_str)
                .strip_suffix(')')
                .unwrap_or(angle_str);
            let angle: f64 = angle_str.trim_end_matches("deg").trim().parse().ok()?;
            Some(FontStyleValue::Oblique(Some(angle)))
        }
    } else {
        None
    }
}

// ── CSS Scroll Snap 值类型 ──────────────────────────────────────────

/// CSS scroll-snap-type 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapTypeValue {
    /// none。
    None,
    /// mandatory（必须吸附）。
    Mandatory,
    /// proximity（接近时吸附）。
    Proximity,
}

/// CSS scroll-snap-type 轴。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapAxis {
    /// x 轴。
    X,
    /// y 轴。
    Y,
    /// 两个轴。
    Both,
}

/// CSS scroll-snap-align 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapAlignValue {
    /// none。
    None,
    /// start。
    Start,
    /// end。
    End,
    /// center。
    Center,
}

/// CSS scroll-snap-stop 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollSnapStopValue {
    /// normal。
    Normal,
    /// always。
    Always,
}

/// CSS container-type 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerTypeValue {
    /// normal。
    Normal,
    /// size。
    Size,
    /// inline-size。
    InlineSize,
}

/// 解析 CSS scroll-snap-type 属性值。
///
/// 支持格式如 `"none"`、`"x mandatory"`、`"y proximity"`、`"both mandatory"`。
/// 返回 (strictness, axis) 元组。
pub fn parse_scroll_snap_type(value: &str) -> Option<(ScrollSnapTypeValue, Option<ScrollSnapAxis>)> {
    let value = value.trim().to_ascii_lowercase();

    if value == "none" {
        return Some((ScrollSnapTypeValue::None, None));
    }

    let parts: Vec<&str> = value.split_whitespace().collect();
    let mut strictness = None;
    let mut axis = None;

    for part in parts {
        match part {
            "mandatory" => strictness = Some(ScrollSnapTypeValue::Mandatory),
            "proximity" => strictness = Some(ScrollSnapTypeValue::Proximity),
            "x" => axis = Some(ScrollSnapAxis::X),
            "y" => axis = Some(ScrollSnapAxis::Y),
            "both" => axis = Some(ScrollSnapAxis::Both),
            _ => return None,
        }
    }

    strictness.map(|s| (s, axis))
}

/// 解析 CSS scroll-snap-align 属性值。
pub fn parse_scroll_snap_align(value: &str) -> Option<ScrollSnapAlignValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(ScrollSnapAlignValue::None),
        "start" => Some(ScrollSnapAlignValue::Start),
        "end" => Some(ScrollSnapAlignValue::End),
        "center" => Some(ScrollSnapAlignValue::Center),
        _ => None,
    }
}

/// 解析 CSS scroll-snap-stop 属性值。
pub fn parse_scroll_snap_stop(value: &str) -> Option<ScrollSnapStopValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(ScrollSnapStopValue::Normal),
        "always" => Some(ScrollSnapStopValue::Always),
        _ => None,
    }
}

/// 解析 CSS container-type 属性值。
pub fn parse_container_type(value: &str) -> Option<ContainerTypeValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(ContainerTypeValue::Normal),
        "size" => Some(ContainerTypeValue::Size),
        "inline-size" => Some(ContainerTypeValue::InlineSize),
        _ => None,
    }
}

/// CSS vertical-align 值。
#[derive(Debug, Clone, PartialEq)]
pub enum VerticalAlignValue {
    /// baseline（默认值）— 元素基线与父元素基线对齐。
    Baseline,
    /// top — 元素顶部与行盒顶部对齐。
    Top,
    /// middle — 元素中部与父元素基线 + 半 x-height 处对齐。
    Middle,
    /// bottom — 元素底部与行盒底部对齐。
    Bottom,
    /// text-top — 元素顶部与父元素字体的顶部对齐。
    TextTop,
    /// text-bottom — 元素底部与父元素字体的底部对齐。
    TextBottom,
    /// sub — 元素基线下移至适合下标的位置。
    Sub,
    /// super — 元素基线上移至适合上标的位置。
    Super,
}

/// CSS cursor 值。
#[derive(Debug, Clone, PartialEq)]
pub enum CursorValue {
    /// auto。
    Auto,
    /// default。
    Default,
    /// pointer。
    Pointer,
    /// move。
    Move,
    /// text。
    Text,
    /// wait。
    Wait,
    /// crosshair。
    Crosshair,
    /// not-allowed。
    NotAllowed,
    /// grab。
    Grab,
    /// grabbing。
    Grabbing,
    /// help。
    Help,
    /// progress。
    Progress,
    /// n-resize。
    NResize,
    /// s-resize。
    SResize,
    /// e-resize。
    EResize,
    /// w-resize。
    WResize,
    /// ne-resize。
    NeResize,
    /// nw-resize。
    NwResize,
    /// se-resize。
    SeResize,
    /// sw-resize。
    SwResize,
    /// col-resize。
    ColResize,
    /// row-resize。
    RowResize,
    /// all-scroll。
    AllScroll,
    /// zoom-in。
    ZoomIn,
    /// zoom-out。
    ZoomOut,
    /// none。
    None,
}

/// 解析 CSS cursor 属性值。
pub fn parse_cursor(value: &str) -> Option<CursorValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(CursorValue::Auto),
        "default" => Some(CursorValue::Default),
        "pointer" => Some(CursorValue::Pointer),
        "move" => Some(CursorValue::Move),
        "text" => Some(CursorValue::Text),
        "wait" => Some(CursorValue::Wait),
        "crosshair" => Some(CursorValue::Crosshair),
        "not-allowed" => Some(CursorValue::NotAllowed),
        "grab" => Some(CursorValue::Grab),
        "grabbing" => Some(CursorValue::Grabbing),
        "help" => Some(CursorValue::Help),
        "progress" => Some(CursorValue::Progress),
        "n-resize" => Some(CursorValue::NResize),
        "s-resize" => Some(CursorValue::SResize),
        "e-resize" => Some(CursorValue::EResize),
        "w-resize" => Some(CursorValue::WResize),
        "ne-resize" => Some(CursorValue::NeResize),
        "nw-resize" => Some(CursorValue::NwResize),
        "se-resize" => Some(CursorValue::SeResize),
        "sw-resize" => Some(CursorValue::SwResize),
        "col-resize" => Some(CursorValue::ColResize),
        "row-resize" => Some(CursorValue::RowResize),
        "all-scroll" => Some(CursorValue::AllScroll),
        "zoom-in" => Some(CursorValue::ZoomIn),
        "zoom-out" => Some(CursorValue::ZoomOut),
        "none" => Some(CursorValue::None),
        _ => None,
    }
}

/// 解析 CSS opacity 属性值。
///
/// 支持数值（0.0-1.0）和百分比（如 `50%` → 0.5）。
/// 结果限制在 [0.0, 1.0] 范围内。
pub fn parse_opacity(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.ends_with('%') {
        let pct: f64 = value.trim_end_matches('%').parse().ok()?;
        Some((pct / 100.0).clamp(0.0, 1.0))
    } else {
        let num: f64 = value.parse().ok()?;
        Some(num.clamp(0.0, 1.0))
    }
}

/// 解析 CSS vertical-align 属性值。
pub fn parse_vertical_align(value: &str) -> Option<VerticalAlignValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "baseline" => Some(VerticalAlignValue::Baseline),
        "top" => Some(VerticalAlignValue::Top),
        "middle" => Some(VerticalAlignValue::Middle),
        "bottom" => Some(VerticalAlignValue::Bottom),
        "text-top" => Some(VerticalAlignValue::TextTop),
        "text-bottom" => Some(VerticalAlignValue::TextBottom),
        "sub" => Some(VerticalAlignValue::Sub),
        "super" => Some(VerticalAlignValue::Super),
        _ => None,
    }
}

/// 解析 1-4 个长度值的简写属性（如 scroll-margin、scroll-padding）。
///
/// 返回 [top, right, bottom, left]（按 CSS 简写规则展开）。
pub fn parse_length_shorthand(value: &str) -> Option<[LengthValue; 4]> {
    let parts: Vec<&str> = value.split_whitespace().collect();
    match parts.len() {
        1 => {
            let v = parse_length(parts[0])?;
            Some([v.clone(), v.clone(), v.clone(), v])
        }
        2 => {
            let tb = parse_length(parts[0])?;
            let lr = parse_length(parts[1])?;
            Some([tb.clone(), lr.clone(), tb, lr])
        }
        3 => {
            let top = parse_length(parts[0])?;
            let lr = parse_length(parts[1])?;
            let bottom = parse_length(parts[2])?;
            Some([top, lr.clone(), bottom, lr])
        }
        4 => {
            let top = parse_length(parts[0])?;
            let right = parse_length(parts[1])?;
            let bottom = parse_length(parts[2])?;
            let left = parse_length(parts[3])?;
            Some([top, right, bottom, left])
        }
        _ => None,
    }
}

/// 找到字符串中第一个不在嵌套括号内的逗号位置。
fn find_top_level_comma(s: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// 解析 CSS var() 函数引用。
///
/// 支持格式如 `var(--name)` 和 `var(--name, fallback)`。
pub fn parse_var(value: &str) -> Option<VarReference> {
    let value = value.trim();

    // CSS Values §4：函数名大小写不敏感（VAR ≡ Var ≡ var）。自定义属性名（--x）大小写敏感，
    // 故仅前缀大小写不敏感检查，内容（变量名/回退）按原样提取。
    if !(value.len() >= 4 && value[..4].eq_ignore_ascii_case("var(")) || !value.ends_with(')') {
        return None;
    }

    // 提取括号内的内容
    let inner = value.get(4..value.len() - 1)?.trim();

    // 找到第一个不在嵌套括号内的逗号
    if let Some(comma_pos) = find_top_level_comma(inner) {
        let name = inner[..comma_pos].trim().to_string();
        let fallback = inner[comma_pos + 1..].trim().to_string();
        Some(VarReference {
            name,
            fallback: Some(fallback),
        })
    } else {
        Some(VarReference {
            name: inner.to_string(),
            fallback: None,
        })
    }
}

// ── CSS Page Break 值类型 ──────────────────────────────────────────────

/// CSS page-break 属性值（page-break-before、page-break-after、page-break-inside）。
#[derive(Debug, Clone, PartialEq)]
pub enum PageBreakValue {
    /// auto。
    Auto,
    /// always。
    Always,
    /// avoid。
    Avoid,
    /// left。
    Left,
    /// right。
    Right,
}

/// 解析 CSS page-break 属性值。
pub fn parse_page_break(value: &str) -> Option<PageBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(PageBreakValue::Auto),
        "always" => Some(PageBreakValue::Always),
        "avoid" => Some(PageBreakValue::Avoid),
        "left" => Some(PageBreakValue::Left),
        "right" => Some(PageBreakValue::Right),
        _ => None,
    }
}

/// CSS box-decoration-break 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum BoxDecorationBreakValue {
    /// slice。
    Slice,
    /// clone。
    Clone,
}

/// 解析 CSS box-decoration-break 属性值。
pub fn parse_box_decoration_break(value: &str) -> Option<BoxDecorationBreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "slice" => Some(BoxDecorationBreakValue::Slice),
        "clone" => Some(BoxDecorationBreakValue::Clone),
        _ => None,
    }
}

/// CSS image-rendering 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum ImageRenderingValue {
    /// auto。
    Auto,
    /// smooth。
    Smooth,
    /// high-quality。
    HighQuality,
    /// pixelated。
    Pixelated,
    /// crisp-edges。
    CrispEdges,
}

/// 解析 CSS image-rendering 属性值。
pub fn parse_image_rendering(value: &str) -> Option<ImageRenderingValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(ImageRenderingValue::Auto),
        "smooth" => Some(ImageRenderingValue::Smooth),
        "high-quality" => Some(ImageRenderingValue::HighQuality),
        "pixelated" => Some(ImageRenderingValue::Pixelated),
        "crisp-edges" => Some(ImageRenderingValue::CrispEdges),
        _ => None,
    }
}

/// CSS isolation 属性值。
#[derive(Debug, Clone, PartialEq)]
pub enum IsolationValue {
    /// auto。
    Auto,
    /// isolate。
    Isolate,
}

/// 解析 CSS isolation 属性值。
pub fn parse_isolation(value: &str) -> Option<IsolationValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(IsolationValue::Auto),
        "isolate" => Some(IsolationValue::Isolate),
        _ => None,
    }
}

/// CSS break-inside 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BreakInsideValue {
    /// auto。
    Auto,
    /// avoid。
    Avoid,
    /// avoid-page。
    AvoidPage,
    /// avoid-column。
    AvoidColumn,
}

/// CSS break-before / break-after 值。
#[derive(Debug, Clone, PartialEq)]
pub enum BreakValue {
    /// auto。
    Auto,
    /// avoid。
    Avoid,
    /// column。
    Column,
    /// page。
    Page,
    /// avoid-page。
    AvoidPage,
    /// avoid-column。
    AvoidColumn,
}

/// CSS column-rule-width 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnRuleWidthValue {
    /// medium。
    Medium,
    /// thin。
    Thin,
    /// thick。
    Thick,
    /// 长度值。
    Length(LengthValue),
}

/// CSS column-rule-style 值。
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnRuleStyleValue {
    /// none。
    None,
    /// hidden。
    Hidden,
    /// dotted。
    Dotted,
    /// dashed。
    Dashed,
    /// solid。
    Solid,
    /// double。
    Double,
    /// groove。
    Groove,
    /// ridge。
    Ridge,
    /// inset。
    Inset,
    /// outset。
    Outset,
}

/// 解析 CSS break-inside 属性值。
pub fn parse_break_inside(value: &str) -> Option<BreakInsideValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(BreakInsideValue::Auto),
        "avoid" => Some(BreakInsideValue::Avoid),
        "avoid-page" => Some(BreakInsideValue::AvoidPage),
        "avoid-column" => Some(BreakInsideValue::AvoidColumn),
        _ => None,
    }
}

/// 解析 CSS break-before 属性值。
pub fn parse_break_before(value: &str) -> Option<BreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(BreakValue::Auto),
        "avoid" => Some(BreakValue::Avoid),
        "column" => Some(BreakValue::Column),
        "page" => Some(BreakValue::Page),
        "avoid-page" => Some(BreakValue::AvoidPage),
        "avoid-column" => Some(BreakValue::AvoidColumn),
        _ => None,
    }
}

/// 解析 CSS break-after 属性值。
pub fn parse_break_after(value: &str) -> Option<BreakValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(BreakValue::Auto),
        "avoid" => Some(BreakValue::Avoid),
        "column" => Some(BreakValue::Column),
        "page" => Some(BreakValue::Page),
        "avoid-page" => Some(BreakValue::AvoidPage),
        "avoid-column" => Some(BreakValue::AvoidColumn),
        _ => None,
    }
}

/// 解析 CSS column-rule-width 属性值。
pub fn parse_column_rule_width(value: &str) -> Option<ColumnRuleWidthValue> {
    let v = value.trim().to_ascii_lowercase();
    match v.as_str() {
        "medium" => Some(ColumnRuleWidthValue::Medium),
        "thin" => Some(ColumnRuleWidthValue::Thin),
        "thick" => Some(ColumnRuleWidthValue::Thick),
        _ => parse_length(&v).map(ColumnRuleWidthValue::Length),
    }
}

/// 解析 CSS column-rule-style 属性值。
pub fn parse_column_rule_style(value: &str) -> Option<ColumnRuleStyleValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "none" => Some(ColumnRuleStyleValue::None),
        "hidden" => Some(ColumnRuleStyleValue::Hidden),
        "dotted" => Some(ColumnRuleStyleValue::Dotted),
        "dashed" => Some(ColumnRuleStyleValue::Dashed),
        "solid" => Some(ColumnRuleStyleValue::Solid),
        "double" => Some(ColumnRuleStyleValue::Double),
        "groove" => Some(ColumnRuleStyleValue::Groove),
        "ridge" => Some(ColumnRuleStyleValue::Ridge),
        "inset" => Some(ColumnRuleStyleValue::Inset),
        "outset" => Some(ColumnRuleStyleValue::Outset),
        _ => None,
    }
}

/// CSS direction 值。
#[derive(Debug, Clone, PartialEq)]
pub enum DirectionValue {
    /// ltr（默认值）— 从左到右。
    Ltr,
    /// rtl — 从右到左。
    Rtl,
}

/// 解析 CSS direction 属性值。
pub fn parse_direction(value: &str) -> Option<DirectionValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "ltr" => Some(DirectionValue::Ltr),
        "rtl" => Some(DirectionValue::Rtl),
        _ => None,
    }
}

/// CSS unicode-bidi 值。
#[derive(Debug, Clone, PartialEq)]
pub enum UnicodeBidiValue {
    /// normal（默认值）。
    Normal,
    /// embed。
    Embed,
    /// isolate。
    Isolate,
    /// bidi-override。
    BidiOverride,
    /// isolate-override。
    IsolateOverride,
    /// plaintext。
    Plaintext,
}

/// 解析 CSS unicode-bidi 属性值。
pub fn parse_unicode_bidi(value: &str) -> Option<UnicodeBidiValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(UnicodeBidiValue::Normal),
        "embed" => Some(UnicodeBidiValue::Embed),
        "isolate" => Some(UnicodeBidiValue::Isolate),
        "bidi-override" => Some(UnicodeBidiValue::BidiOverride),
        "isolate-override" => Some(UnicodeBidiValue::IsolateOverride),
        "plaintext" => Some(UnicodeBidiValue::Plaintext),
        _ => None,
    }
}

/// CSS tab-size 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TabSizeValue {
    /// 数字值（空格数）。
    Number(u32),
    /// 长度值（如 px、em）。
    Length(LengthValue),
}

/// 解析 CSS tab-size 属性值。
///
/// 支持整数（如 `4`）和长度值（如 `20px`、`1em`）。
pub fn parse_tab_size(value: &str) -> Option<TabSizeValue> {
    let value = value.trim();
    // 先尝试解析为整数
    if let Ok(n) = value.parse::<u32>() {
        return Some(TabSizeValue::Number(n));
    }
    // 再尝试解析为长度值
    parse_length(value).map(TabSizeValue::Length)
}

/// CSS overflow-wrap 值。
#[derive(Debug, Clone, PartialEq)]
pub enum OverflowWrapValue {
    /// normal。
    Normal,
    /// break-word。
    BreakWord,
    /// anywhere。
    Anywhere,
}

/// 解析 CSS overflow-wrap 属性值。
pub fn parse_overflow_wrap(value: &str) -> Option<OverflowWrapValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(OverflowWrapValue::Normal),
        "break-word" => Some(OverflowWrapValue::BreakWord),
        "anywhere" => Some(OverflowWrapValue::Anywhere),
        _ => None,
    }
}

/// CSS text-align-last 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TextAlignLastValue {
    /// auto。
    Auto,
    /// start。
    Start,
    /// end。
    End,
    /// left。
    Left,
    /// right。
    Right,
    /// center。
    Center,
    /// justify。
    Justify,
}

/// 解析 CSS text-align-last 属性值。
pub fn parse_text_align_last(value: &str) -> Option<TextAlignLastValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "auto" => Some(TextAlignLastValue::Auto),
        "start" => Some(TextAlignLastValue::Start),
        "end" => Some(TextAlignLastValue::End),
        "left" => Some(TextAlignLastValue::Left),
        "right" => Some(TextAlignLastValue::Right),
        "center" => Some(TextAlignLastValue::Center),
        "justify" => Some(TextAlignLastValue::Justify),
        _ => None,
    }
}

/// CSS font-variant-numeric 值。
#[derive(Debug, Clone, PartialEq)]
pub enum FontVariantNumericValue {
    /// normal。
    Normal,
    /// ordinal。
    Ordinal,
    /// slashed-zero。
    SlashedZero,
    /// lining-nums。
    LiningNums,
    /// oldstyle-nums。
    OldstyleNums,
    /// proportional-nums。
    ProportionalNums,
    /// tabular-nums。
    TabularNums,
    /// diagonal-fractions。
    DiagonalFractions,
    /// stacked-fractions。
    StackedFractions,
}

/// 解析 CSS font-variant-numeric 属性值。
pub fn parse_font_variant_numeric(value: &str) -> Option<FontVariantNumericValue> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some(FontVariantNumericValue::Normal),
        "ordinal" => Some(FontVariantNumericValue::Ordinal),
        "slashed-zero" => Some(FontVariantNumericValue::SlashedZero),
        "lining-nums" => Some(FontVariantNumericValue::LiningNums),
        "oldstyle-nums" => Some(FontVariantNumericValue::OldstyleNums),
        "proportional-nums" => Some(FontVariantNumericValue::ProportionalNums),
        "tabular-nums" => Some(FontVariantNumericValue::TabularNums),
        "diagonal-fractions" => Some(FontVariantNumericValue::DiagonalFractions),
        "stacked-fractions" => Some(FontVariantNumericValue::StackedFractions),
        _ => None,
    }
}

// ── CSS Transition 值类型 ──────────────────────────────────────────────

/// CSS transition-timing-function / animation-timing-function 值。
#[derive(Debug, Clone, PartialEq)]
pub enum TimingFunctionValue {
    /// ease。
    Ease,
    /// linear。
    Linear,
    /// ease-in。
    EaseIn,
    /// ease-out。
    EaseOut,
    /// ease-in-out。
    EaseInOut,
    /// cubic-bezier(x1, y1, x2, y2)。
    CubicBezier(f64, f64, f64, f64),
    /// step-start。
    StepStart,
    /// step-end。
    StepEnd,
    /// steps(n, position)。
    Steps(i32, Option<StepPosition>),
}

/// steps() 的位置参数。
#[derive(Debug, Clone, PartialEq)]
pub enum StepPosition {
    /// jump-start / start。
    Start,
    /// jump-end / end（默认）。
    End,
    /// jump-both。
    Both,
    /// jump-none。
    None,
}

/// CSS animation-direction 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationDirectionValue {
    /// normal。
    Normal,
    /// reverse。
    Reverse,
    /// alternate。
    Alternate,
    /// alternate-reverse。
    AlternateReverse,
}

/// CSS animation-fill-mode 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationFillModeValue {
    /// none。
    None,
    /// forwards。
    Forwards,
    /// backwards。
    Backwards,
    /// both。
    Both,
}

/// CSS animation-play-state 值。
#[derive(Debug, Clone, PartialEq)]
pub enum AnimationPlayStateValue {
    /// running。
    Running,
    /// paused。
    Paused,
}

/// 解析 CSS animation-direction 值。
