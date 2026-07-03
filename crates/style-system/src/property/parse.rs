//! CSS 属性解析函数。

use super::types::*;
use zero_css_parser::values;

/// 解析 CSS border-style 值。
pub fn parse_border_style(value: &str) -> Option<BorderStyleValue> {
    match value.trim() {
        "none" => Some(BorderStyleValue::None),
        "hidden" => Some(BorderStyleValue::Hidden),
        "dotted" => Some(BorderStyleValue::Dotted),
        "dashed" => Some(BorderStyleValue::Dashed),
        "solid" => Some(BorderStyleValue::Solid),
        "double" => Some(BorderStyleValue::Double),
        "groove" => Some(BorderStyleValue::Groove),
        "ridge" => Some(BorderStyleValue::Ridge),
        "inset" => Some(BorderStyleValue::Inset),
        "outset" => Some(BorderStyleValue::Outset),
        _ => None,
    }
}

/// 解析 CSS outline-style 值。
pub fn parse_outline_style(value: &str) -> Option<OutlineStyleValue> {
    match value.trim() {
        "none" => Some(OutlineStyleValue::None),
        "dotted" => Some(OutlineStyleValue::Dotted),
        "dashed" => Some(OutlineStyleValue::Dashed),
        "solid" => Some(OutlineStyleValue::Solid),
        "double" => Some(OutlineStyleValue::Double),
        "groove" => Some(OutlineStyleValue::Groove),
        "ridge" => Some(OutlineStyleValue::Ridge),
        "inset" => Some(OutlineStyleValue::Inset),
        "outset" => Some(OutlineStyleValue::Outset),
        _ => None,
    }
}

/// 解析 CSS grid-auto-flow 值。
pub fn parse_grid_auto_flow(value: &str) -> Option<GridAutoFlowValue> {
    let value = value.trim().to_ascii_lowercase();
    match value.as_str() {
        "row" => Some(GridAutoFlowValue::Row),
        "column" => Some(GridAutoFlowValue::Column),
        "dense" | "row dense" => Some(GridAutoFlowValue::RowDense),
        "column dense" => Some(GridAutoFlowValue::ColumnDense),
        _ => None,
    }
}

/// 解析 CSS grid line 值（用于 grid-column/row-start/end）。
///
/// 支持格式：`auto`、`1`（行号）、`-1`（从末尾）、`span 2`。
pub fn parse_grid_line(value: &str) -> Option<GridLineValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(GridLineValue::Auto);
    }
    if let Some(span_str) = value.strip_prefix("span ") {
        let span: u16 = span_str.trim().parse().ok()?;
        return Some(GridLineValue::Span(span));
    }
    if let Some(span_str) = value.strip_prefix("span") {
        let span: u16 = span_str.trim().parse().ok()?;
        return Some(GridLineValue::Span(span));
    }
    if let Ok(line) = value.parse::<i16>() {
        if line == 0 {
            return None; // 0 是非法的 grid line 值
        }
        return Some(GridLineValue::Line(line));
    }
    // 非数字值视为命名区域（如 "header"、"sidebar"）
    // 合法的命名区域标识符：非空，不含 / 和数字开头
    if !value.is_empty() && !value.starts_with(|c: char| c.is_ascii_digit()) && !value.contains('/') {
        return Some(GridLineValue::Name(value.to_string()));
    }
    None
}

/// 解析 CSS grid-area 简写并展开为四个 GridLineValue。
///
/// 返回 `(row_start, row_end, col_start, col_end)`。
/// 解析失败返回 `None`。
pub fn parse_grid_area_shorthand(value: &str) -> Option<(GridLineValue, GridLineValue, GridLineValue, GridLineValue)> {
    let (rs, re, cs, ce) = values::parse_grid_area(value)?;
    let row_start = parse_grid_line(&rs)?;
    let row_end = parse_grid_line(&re)?;
    let col_start = parse_grid_line(&cs)?;
    let col_end = parse_grid_line(&ce)?;
    Some((row_start, row_end, col_start, col_end))
}

/// 解析 CSS grid-column / grid-row 简写（`<start> / <end>` 格式）。
///
/// 返回 `(start, end)`。
/// 无斜杠时，`<start>` 作为 start，end 为 Auto。
pub fn parse_grid_line_shorthand(value: &str) -> Option<(GridLineValue, GridLineValue)> {
    let value = value.trim();
    if let Some(slash_pos) = value.find('/') {
        let start_str = value[..slash_pos].trim();
        let end_str = value[slash_pos + 1..].trim();
        if start_str.is_empty() || end_str.is_empty() {
            return None;
        }
        let start = parse_grid_line(start_str)?;
        let end = parse_grid_line(end_str)?;
        Some((start, end))
    } else {
        let start = parse_grid_line(value)?;
        Some((start, GridLineValue::Auto))
    }
}

/// 解析逗号分隔的 transition-timing-function 列表。
///
/// 需要处理 cubic-bezier() 和 steps() 内部的逗号。
pub(crate) fn parse_comma_separated_timing_functions(value: &str) -> Vec<zero_css_parser::values::TimingFunctionValue> {
    let mut result = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (i, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let part = value[start..i].trim();
                if let Some(tf) = values::parse_timing_function(part) {
                    result.push(tf);
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    // 处理最后一个
    let last = value[start..].trim();
    if let Some(tf) = values::parse_timing_function(last) {
        result.push(tf);
    }

    result
}

/// 解析 CSS line-height 值。
pub fn parse_line_height(value: &str) -> Option<LineHeightValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("normal") {
        return Some(LineHeightValue::Normal);
    }
    // 尝试解析为无单位数值
    if let Ok(num) = value.parse::<f64>() {
        // 如果值不含单位后缀，视为无单位数值
        if !value.contains("px")
            && !value.contains("em")
            && !value.contains("rem")
            && !value.contains("%")
            && !value.contains("vh")
            && !value.contains("vw")
        {
            return Some(LineHeightValue::Number(num));
        }
    }
    // 尝试解析为长度
    if let Some(length) = values::parse_length(value) {
        return Some(LineHeightValue::Length(length));
    }
    None
}

/// 解析 CSS text-align 值。
pub fn parse_text_align(value: &str) -> Option<TextAlignValue> {
    match value.trim() {
        "left" => Some(TextAlignValue::Left),
        "right" => Some(TextAlignValue::Right),
        "center" => Some(TextAlignValue::Center),
        "justify" => Some(TextAlignValue::Justify),
        "start" => Some(TextAlignValue::Start),
        "end" => Some(TextAlignValue::End),
        _ => None,
    }
}

/// 解析 CSS text-decoration 值。
pub fn parse_text_decoration(value: &str) -> Option<TextDecorationValue> {
    match value.trim() {
        "none" => Some(TextDecorationValue::None),
        "underline" => Some(TextDecorationValue::Underline),
        "overline" => Some(TextDecorationValue::Overline),
        "line-through" => Some(TextDecorationValue::LineThrough),
        _ => None,
    }
}

/// 解析 CSS text-decoration-line 值。
pub fn parse_text_decoration_line(value: &str) -> Option<TextDecorationLineValue> {
    match value.trim() {
        "none" => Some(TextDecorationLineValue::None),
        "underline" => Some(TextDecorationLineValue::Underline),
        "overline" => Some(TextDecorationLineValue::Overline),
        "line-through" => Some(TextDecorationLineValue::LineThrough),
        "blink" => Some(TextDecorationLineValue::Blink),
        _ => None,
    }
}

/// 解析 CSS text-transform 值。
pub fn parse_text_transform(value: &str) -> Option<TextTransformValue> {
    match value.trim() {
        "none" => Some(TextTransformValue::None),
        "uppercase" => Some(TextTransformValue::Uppercase),
        "lowercase" => Some(TextTransformValue::Lowercase),
        "capitalize" => Some(TextTransformValue::Capitalize),
        _ => None,
    }
}

/// 解析 CSS white-space 值。
pub fn parse_white_space(value: &str) -> Option<WhiteSpaceValue> {
    match value.trim() {
        "normal" => Some(WhiteSpaceValue::Normal),
        "pre" => Some(WhiteSpaceValue::Pre),
        "nowrap" => Some(WhiteSpaceValue::Nowrap),
        "pre-wrap" => Some(WhiteSpaceValue::PreWrap),
        "pre-line" => Some(WhiteSpaceValue::PreLine),
        "break-spaces" => Some(WhiteSpaceValue::BreakSpaces),
        _ => None,
    }
}

/// 解析 CSS word-break 值。
pub fn parse_word_break(value: &str) -> Option<WordBreakValue> {
    match value.trim() {
        "normal" => Some(WordBreakValue::Normal),
        "break-all" => Some(WordBreakValue::BreakAll),
        "keep-all" => Some(WordBreakValue::KeepAll),
        "break-word" => Some(WordBreakValue::BreakWord),
        _ => None,
    }
}

/// 解析 CSS line-break 值（CSS Text 3 §5.3）。
pub fn parse_line_break(value: &str) -> Option<LineBreakValue> {
    match value.trim() {
        "auto" => Some(LineBreakValue::Auto),
        "loose" => Some(LineBreakValue::Loose),
        "normal" => Some(LineBreakValue::Normal),
        "strict" => Some(LineBreakValue::Strict),
        "anywhere" => Some(LineBreakValue::Anywhere),
        _ => None,
    }
}

/// 解析 CSS writing-mode 值。
pub fn parse_writing_mode(value: &str) -> Option<WritingModeValue> {
    match value.trim() {
        "horizontal-tb" => Some(WritingModeValue::HorizontalTb),
        "vertical-rl" => Some(WritingModeValue::VerticalRl),
        "vertical-lr" => Some(WritingModeValue::VerticalLr),
        _ => None,
    }
}

/// 解析 CSS text-overflow 值。
pub fn parse_text_overflow(value: &str) -> Option<TextOverflowValue> {
    let v = value.trim();
    if let Some(parsed) = values::parse_text_overflow(v) {
        return match parsed {
            zero_css_parser::values::TextOverflowValue::Clip => Some(TextOverflowValue::Clip),
            zero_css_parser::values::TextOverflowValue::Ellipsis => Some(TextOverflowValue::Ellipsis),
            zero_css_parser::values::TextOverflowValue::String(s) => Some(TextOverflowValue::String(s)),
        };
    }
    None
}

/// 解析 CSS flex-basis 值。
pub fn parse_flex_basis(value: &str) -> Option<FlexBasisValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(FlexBasisValue::Auto);
    }
    if value.eq_ignore_ascii_case("content") {
        return Some(FlexBasisValue::Content);
    }
    if let Some(length) = values::parse_length(value) {
        return Some(FlexBasisValue::Length(length));
    }
    None
}

/// 解析 CSS z-index 值。
pub fn parse_z_index(value: &str) -> Option<ZIndexValue> {
    let value = value.trim();
    if value.eq_ignore_ascii_case("auto") {
        return Some(ZIndexValue::Auto);
    }
    let int: i32 = value.parse().ok()?;
    Some(ZIndexValue::Integer(int))
}

/// 解析 CSS cursor 值。
pub fn parse_cursor(value: &str) -> Option<CursorValue> {
    match value.trim() {
        "auto" => Some(CursorValue::Auto),
        "default" => Some(CursorValue::Default),
        "pointer" => Some(CursorValue::Pointer),
        "move" => Some(CursorValue::Move),
        "text" => Some(CursorValue::Text),
        "wait" => Some(CursorValue::Wait),
        "crosshair" => Some(CursorValue::Crosshair),
        "help" => Some(CursorValue::Help),
        "not-allowed" => Some(CursorValue::NotAllowed),
        "grab" => Some(CursorValue::Grab),
        "grabbing" => Some(CursorValue::Grabbing),
        "col-resize" => Some(CursorValue::ColResize),
        "row-resize" => Some(CursorValue::RowResize),
        "ns-resize" => Some(CursorValue::NsResize),
        "ew-resize" => Some(CursorValue::EwResize),
        "none" => Some(CursorValue::None),
        "progress" => Some(CursorValue::Progress),
        "cell" => Some(CursorValue::Cell),
        "copy" => Some(CursorValue::Copy),
        "alias" => Some(CursorValue::Alias),
        "all-scroll" => Some(CursorValue::AllScroll),
        "zoom-in" => Some(CursorValue::ZoomIn),
        "zoom-out" => Some(CursorValue::ZoomOut),
        _ => None,
    }
}

/// 将 css-parser 的 CursorValue 映射为 style-system 的 CursorValue。
pub(crate) fn map_css_cursor(v: zero_css_parser::values::CursorValue) -> CursorValue {
    use zero_css_parser::values::CursorValue as Cv;
    match v {
        Cv::Auto => CursorValue::Auto,
        Cv::Default => CursorValue::Default,
        Cv::Pointer => CursorValue::Pointer,
        Cv::Move => CursorValue::Move,
        Cv::Text => CursorValue::Text,
        Cv::Wait => CursorValue::Wait,
        Cv::Crosshair => CursorValue::Crosshair,
        Cv::NotAllowed => CursorValue::NotAllowed,
        Cv::Grab => CursorValue::Grab,
        Cv::Grabbing => CursorValue::Grabbing,
        Cv::Help => CursorValue::Help,
        Cv::Progress => CursorValue::Progress,
        Cv::NResize => CursorValue::NsResize,
        Cv::SResize => CursorValue::NsResize,
        Cv::EResize => CursorValue::EwResize,
        Cv::WResize => CursorValue::EwResize,
        Cv::NeResize => CursorValue::NsResize,
        Cv::NwResize => CursorValue::NsResize,
        Cv::SeResize => CursorValue::NsResize,
        Cv::SwResize => CursorValue::NsResize,
        Cv::ColResize => CursorValue::ColResize,
        Cv::RowResize => CursorValue::RowResize,
        Cv::AllScroll => CursorValue::AllScroll,
        Cv::ZoomIn => CursorValue::ZoomIn,
        Cv::ZoomOut => CursorValue::ZoomOut,
        Cv::None => CursorValue::None,
    }
}

/// 解析 CSS scroll-snap-type 值。
///
/// 格式：none | [ mandatory | proximity ] [ x | y | both ]?
pub fn parse_scroll_snap_type_computed(value: &str) -> Option<ScrollSnapType> {
    let parsed = values::parse_scroll_snap_type(value)?;
    let strictness = match parsed.0 {
        ScrollSnapTypeValue::None => ScrollSnapStrictness::None,
        ScrollSnapTypeValue::Mandatory => ScrollSnapStrictness::Mandatory,
        ScrollSnapTypeValue::Proximity => ScrollSnapStrictness::Proximity,
    };
    let axis = parsed.1.unwrap_or(ScrollSnapAxis::Both);
    Some(ScrollSnapType { strictness, axis })
}

/// 解析 CSS scroll-snap-align 值。
pub fn parse_scroll_snap_align_computed(value: &str) -> Option<ScrollSnapAlign> {
    match values::parse_scroll_snap_align(value)? {
        ScrollSnapAlignValue::None => Some(ScrollSnapAlign::None),
        ScrollSnapAlignValue::Start => Some(ScrollSnapAlign::Start),
        ScrollSnapAlignValue::End => Some(ScrollSnapAlign::End),
        ScrollSnapAlignValue::Center => Some(ScrollSnapAlign::Center),
    }
}

/// 解析 CSS scroll-snap-stop 值。
pub fn parse_scroll_snap_stop_computed(value: &str) -> Option<ScrollSnapStop> {
    match values::parse_scroll_snap_stop(value)? {
        ScrollSnapStopValue::Normal => Some(ScrollSnapStop::Normal),
        ScrollSnapStopValue::Always => Some(ScrollSnapStop::Always),
    }
}

/// 解析 CSS scroll-padding 值。
pub fn parse_scroll_padding(value: &str) -> Option<ScrollPadding> {
    let v = value.trim();
    if v.eq_ignore_ascii_case("auto") {
        return Some(ScrollPadding::Auto);
    }
    values::parse_length(v).map(|l| {
        let px = match l {
            LengthValue::Px(n) => n as f32,
            other => resolve_length_to_px(other),
        };
        ScrollPadding::Length(px)
    })
}

/// 将 LengthValue 转换为 f32 px（简单近似，非相对单位返回 0.0）。
pub(crate) fn resolve_length_to_px(l: LengthValue) -> f32 {
    match l {
        LengthValue::Px(n) => n as f32,
        _ => 0.0,
    }
}

/// 解析 CSS container-type 值。
pub fn parse_container_type_computed(value: &str) -> Option<ContainerType> {
    match values::parse_container_type(value)? {
        ContainerTypeValue::Normal => Some(ContainerType::Normal),
        ContainerTypeValue::Size => Some(ContainerType::Size),
        ContainerTypeValue::InlineSize => Some(ContainerType::InlineSize),
    }
}

/// 解析 font-family 值。
///
/// 简单实现：按逗号分割，去除引号和空格。
pub fn parse_font_family(value: &str) -> Vec<String> {
    let mut result = Vec::new();
    for part in value.split(',') {
        let s = part.trim();
        if s.is_empty() {
            continue;
        }
        // 带引号的名称直接使用
        if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
            result.push(s[1..s.len() - 1].to_string());
            continue;
        }
        // 未引号的名称必须仅包含 CSS 标识符字符（字母、数字、连字符、下划线、空格）
        // 包含 !@#$% 等无效字符的名称应使整个声明无效（CSS 规范 § 3.1）
        if s.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == ' ')
        {
            result.push(s.to_string());
        } else {
            // 无效字符 → 整个声明无效
            return Vec::new();
        }
    }
    result
}
