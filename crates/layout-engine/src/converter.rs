//! ComputedStyle → taffy::Style 转换层。
//!
//! 将 [`ComputedStyle`] 的字段映射到 taffy 的 [`taffy::Style`] 结构体，
//! 这是布局引擎的关键适配层。

use zero_css_parser::values::{
    AlignmentValue, BoxSizingValue, DisplayValue, FlexDirectionValue, FlexWrapValue, LengthValue,
    OverflowValue, PositionValue,
};
use zero_style_system::{ComputedStyle, FlexBasisValue, GridAutoFlowValue, GridLineValue};

use taffy::prelude::*;

/// 将 ComputedStyle 转换为 taffy::Style。
///
/// 处理所有 CSS 属性到 taffy 布局属性的映射。
pub fn computed_style_to_taffy(style: &ComputedStyle) -> taffy::Style {
    taffy::Style {
        display: convert_display(&style.display),
        box_sizing: convert_box_sizing(&style.box_sizing),
        overflow: taffy::geometry::Point {
            x: convert_overflow(&style.overflow_x),
            y: convert_overflow(&style.overflow_y),
        },
        scrollbar_width: 0.0,
        position: convert_position(&style.position),
        inset: taffy::geometry::Rect {
            left: convert_length_to_lpa(&style.left),
            right: convert_length_to_lpa(&style.right),
            top: convert_length_to_lpa(&style.top),
            bottom: convert_length_to_lpa(&style.bottom),
        },
        size: taffy::geometry::Size {
            width: convert_length_to_dimension(&style.width),
            height: convert_length_to_dimension(&style.height),
        },
        min_size: taffy::geometry::Size {
            width: convert_length_to_dimension(&style.min_width),
            height: convert_length_to_dimension(&style.min_height),
        },
        max_size: taffy::geometry::Size {
            width: convert_max_length_to_dimension(&style.max_width),
            height: convert_max_length_to_dimension(&style.max_height),
        },
        aspect_ratio: None,
        margin: taffy::geometry::Rect {
            left: convert_length_to_lpa(&style.margin_left),
            right: convert_length_to_lpa(&style.margin_right),
            top: convert_length_to_lpa(&style.margin_top),
            bottom: convert_length_to_lpa(&style.margin_bottom),
        },
        padding: taffy::geometry::Rect {
            left: convert_length_to_lp(&style.padding_left),
            right: convert_length_to_lp(&style.padding_right),
            top: convert_length_to_lp(&style.padding_top),
            bottom: convert_length_to_lp(&style.padding_bottom),
        },
        border: taffy::geometry::Rect {
            left: convert_length_to_lp(&style.border_left_width),
            right: convert_length_to_lp(&style.border_right_width),
            top: convert_length_to_lp(&style.border_top_width),
            bottom: convert_length_to_lp(&style.border_bottom_width),
        },
        align_items: convert_alignment_to_align_items(&style.align_items),
        align_self: convert_alignment_to_align_self(&style.align_self),
        align_content: convert_alignment_to_align_content(&style.justify_content),
        justify_content: convert_alignment_to_justify_content(&style.justify_content),
        gap: taffy::geometry::Size {
            width: convert_length_to_lp(&style.gap),
            height: convert_length_to_lp(&style.row_gap),
        },
        grid_template_rows: parse_grid_tracks(&style.grid_template_rows),
        grid_template_columns: parse_grid_tracks(&style.grid_template_columns),
        grid_auto_flow: convert_grid_auto_flow(&style.grid_auto_flow),
        grid_auto_rows: parse_grid_auto_tracks(&style.grid_auto_rows),
        grid_auto_columns: parse_grid_auto_tracks(&style.grid_auto_columns),
        grid_row: taffy::geometry::Line {
            start: convert_grid_line(&style.grid_row_start),
            end: convert_grid_line(&style.grid_row_end),
        },
        grid_column: taffy::geometry::Line {
            start: convert_grid_line(&style.grid_column_start),
            end: convert_grid_line(&style.grid_column_end),
        },
        flex_direction: convert_flex_direction(&style.flex_direction),
        flex_wrap: convert_flex_wrap(&style.flex_wrap),
        flex_basis: convert_flex_basis(&style.flex_basis),
        flex_grow: style.flex_grow as f32,
        flex_shrink: style.flex_shrink as f32,
        ..taffy::Style::default()
    }
}

/// 转换 display 属性。
fn convert_display(value: &DisplayValue) -> taffy::style::Display {
    match value {
        DisplayValue::Block => taffy::style::Display::Block,
        DisplayValue::Flex => taffy::style::Display::Flex,
        DisplayValue::InlineFlex => taffy::style::Display::Flex,
        DisplayValue::Grid => taffy::style::Display::Grid,
        DisplayValue::InlineGrid => taffy::style::Display::Grid,
        DisplayValue::None => taffy::style::Display::None,
        // inline, inline-block, flow, flow-root, list-item, contents 都映射为 Block
        DisplayValue::Inline
        | DisplayValue::InlineBlock
        | DisplayValue::Flow
        | DisplayValue::FlowRoot
        | DisplayValue::ListItem
        | DisplayValue::Contents => taffy::style::Display::Block,
    }
}

/// 转换 position 属性。
fn convert_position(value: &PositionValue) -> taffy::style::Position {
    match value {
        PositionValue::Absolute => taffy::style::Position::Absolute,
        // taffy 没有 Fixed/Sticky，映射为 Relative
        PositionValue::Fixed
        | PositionValue::Sticky
        | PositionValue::Relative
        | PositionValue::Static => taffy::style::Position::Relative,
    }
}

/// 转换 overflow 属性。
fn convert_overflow(value: &OverflowValue) -> taffy::style::Overflow {
    match value {
        OverflowValue::Visible => taffy::style::Overflow::Visible,
        OverflowValue::Hidden => taffy::style::Overflow::Hidden,
        OverflowValue::Clip => taffy::style::Overflow::Clip,
        OverflowValue::Scroll | OverflowValue::Auto => taffy::style::Overflow::Scroll,
    }
}

/// 转换 box-sizing 属性。
fn convert_box_sizing(value: &BoxSizingValue) -> taffy::style::BoxSizing {
    match value {
        BoxSizingValue::ContentBox => taffy::style::BoxSizing::ContentBox,
        BoxSizingValue::BorderBox => taffy::style::BoxSizing::BorderBox,
    }
}

/// 将 LengthValue 转换为 taffy 的 Dimension。
///
/// em/rem 单位已由 style-system 解析为 px，所以统一用 Length。
/// Auto 映射为 Auto，Percentage 映射为 Percent。
fn convert_length_to_dimension(value: &LengthValue) -> taffy::style::Dimension {
    match value {
        LengthValue::Px(v) => length(*v as f32),
        LengthValue::Em(v) => length(*v as f32),
        LengthValue::Rem(v) => length(*v as f32),
        LengthValue::Vh(v) => length(*v as f32),
        LengthValue::Vw(v) => length(*v as f32),
        LengthValue::Vmin(v) => length(*v as f32),
        LengthValue::Vmax(v) => length(*v as f32),
        LengthValue::Ch(v) => length(*v as f32),
        LengthValue::Percentage(v) => taffy::style::Dimension::Percent((*v / 100.0) as f32),
        LengthValue::Auto => taffy::style::Dimension::Auto,
    }
}

/// 将 max-width/max-height 的 LengthValue 转换为 Dimension。
///
/// max-width/max-height 默认值为 INFINITY，映射为 Auto。
fn convert_max_length_to_dimension(value: &LengthValue) -> taffy::style::Dimension {
    match value {
        LengthValue::Px(v) => {
            let v = *v as f32;
            if v.is_infinite() {
                taffy::style::Dimension::Auto
            } else {
                length(v)
            }
        }
        LengthValue::Em(v) => length(*v as f32),
        LengthValue::Rem(v) => length(*v as f32),
        LengthValue::Vh(v) => length(*v as f32),
        LengthValue::Vw(v) => length(*v as f32),
        LengthValue::Vmin(v) => length(*v as f32),
        LengthValue::Vmax(v) => length(*v as f32),
        LengthValue::Ch(v) => length(*v as f32),
        LengthValue::Percentage(v) => taffy::style::Dimension::Percent((*v / 100.0) as f32),
        LengthValue::Auto => taffy::style::Dimension::Auto,
    }
}

/// 将 LengthValue 转换为 taffy 的 LengthPercentage。
///
/// 用于 padding、border、gap 等不接受 auto 的属性。
fn convert_length_to_lp(value: &LengthValue) -> taffy::style::LengthPercentage {
    match value {
        LengthValue::Px(v) => length(*v as f32),
        LengthValue::Em(v) => length(*v as f32),
        LengthValue::Rem(v) => length(*v as f32),
        LengthValue::Vh(v) => length(*v as f32),
        LengthValue::Vw(v) => length(*v as f32),
        LengthValue::Vmin(v) => length(*v as f32),
        LengthValue::Vmax(v) => length(*v as f32),
        LengthValue::Ch(v) => length(*v as f32),
        LengthValue::Percentage(v) => taffy::style::LengthPercentage::Percent((*v / 100.0) as f32),
        LengthValue::Auto => length(0.0), // 不接受 auto 的属性，auto 视为 0
    }
}

/// 将 LengthValue 转换为 taffy 的 LengthPercentageAuto。
///
/// 用于 margin、inset 等接受 auto 的属性。
fn convert_length_to_lpa(value: &LengthValue) -> taffy::style::LengthPercentageAuto {
    match value {
        LengthValue::Px(v) => length(*v as f32),
        LengthValue::Em(v) => length(*v as f32),
        LengthValue::Rem(v) => length(*v as f32),
        LengthValue::Vh(v) => length(*v as f32),
        LengthValue::Vw(v) => length(*v as f32),
        LengthValue::Vmin(v) => length(*v as f32),
        LengthValue::Vmax(v) => length(*v as f32),
        LengthValue::Ch(v) => length(*v as f32),
        LengthValue::Percentage(v) => {
            taffy::style::LengthPercentageAuto::Percent((*v / 100.0) as f32)
        }
        LengthValue::Auto => taffy::style::LengthPercentageAuto::Auto,
    }
}

/// 转换 flex-direction 属性。
fn convert_flex_direction(value: &FlexDirectionValue) -> taffy::style::FlexDirection {
    match value {
        FlexDirectionValue::Row => taffy::style::FlexDirection::Row,
        FlexDirectionValue::RowReverse => taffy::style::FlexDirection::RowReverse,
        FlexDirectionValue::Column => taffy::style::FlexDirection::Column,
        FlexDirectionValue::ColumnReverse => taffy::style::FlexDirection::ColumnReverse,
    }
}

/// 转换 flex-wrap 属性。
fn convert_flex_wrap(value: &FlexWrapValue) -> taffy::style::FlexWrap {
    match value {
        FlexWrapValue::Nowrap => taffy::style::FlexWrap::NoWrap,
        FlexWrapValue::Wrap => taffy::style::FlexWrap::Wrap,
        FlexWrapValue::WrapReverse => taffy::style::FlexWrap::WrapReverse,
    }
}

/// 转换 flex-basis 属性。
fn convert_flex_basis(value: &FlexBasisValue) -> taffy::style::Dimension {
    match value {
        FlexBasisValue::Auto => taffy::style::Dimension::Auto,
        FlexBasisValue::Content => taffy::style::Dimension::Auto, // taffy 无 content，映射为 Auto
        FlexBasisValue::Length(lv) => convert_length_to_dimension(lv),
    }
}

/// 转换 AlignmentValue 到 taffy AlignItems。
fn convert_alignment_to_align_items(value: &AlignmentValue) -> Option<taffy::style::AlignItems> {
    match value {
        AlignmentValue::FlexStart => Some(taffy::style::AlignItems::FlexStart),
        AlignmentValue::FlexEnd => Some(taffy::style::AlignItems::FlexEnd),
        AlignmentValue::Center => Some(taffy::style::AlignItems::Center),
        AlignmentValue::Stretch => Some(taffy::style::AlignItems::Stretch),
        AlignmentValue::Baseline => Some(taffy::style::AlignItems::Baseline),
        AlignmentValue::Start => Some(taffy::style::AlignItems::Start),
        AlignmentValue::End => Some(taffy::style::AlignItems::End),
        // space-between, space-around, space-evenly 不适用于 align-items
        AlignmentValue::SpaceBetween
        | AlignmentValue::SpaceAround
        | AlignmentValue::SpaceEvenly => None,
    }
}

/// 转换 AlignmentValue 到 taffy AlignSelf。
fn convert_alignment_to_align_self(value: &AlignmentValue) -> Option<taffy::style::AlignSelf> {
    // AlignSelf 是 AlignItems 的 type alias
    match value {
        AlignmentValue::FlexStart => Some(taffy::style::AlignSelf::FlexStart),
        AlignmentValue::FlexEnd => Some(taffy::style::AlignSelf::FlexEnd),
        AlignmentValue::Center => Some(taffy::style::AlignSelf::Center),
        AlignmentValue::Stretch => Some(taffy::style::AlignSelf::Stretch),
        AlignmentValue::Baseline => Some(taffy::style::AlignSelf::Baseline),
        AlignmentValue::Start => Some(taffy::style::AlignSelf::Start),
        AlignmentValue::End => Some(taffy::style::AlignSelf::End),
        AlignmentValue::SpaceBetween
        | AlignmentValue::SpaceAround
        | AlignmentValue::SpaceEvenly => None,
    }
}

/// 转换 AlignmentValue 到 taffy JustifyContent。
fn convert_alignment_to_justify_content(
    value: &AlignmentValue,
) -> Option<taffy::style::JustifyContent> {
    match value {
        AlignmentValue::FlexStart => Some(taffy::style::JustifyContent::FlexStart),
        AlignmentValue::FlexEnd => Some(taffy::style::JustifyContent::FlexEnd),
        AlignmentValue::Center => Some(taffy::style::JustifyContent::Center),
        AlignmentValue::SpaceBetween => Some(taffy::style::JustifyContent::SpaceBetween),
        AlignmentValue::SpaceAround => Some(taffy::style::JustifyContent::SpaceAround),
        AlignmentValue::SpaceEvenly => Some(taffy::style::JustifyContent::SpaceEvenly),
        AlignmentValue::Start => Some(taffy::style::JustifyContent::Start),
        AlignmentValue::End => Some(taffy::style::JustifyContent::End),
        AlignmentValue::Stretch => Some(taffy::style::JustifyContent::Stretch),
        AlignmentValue::Baseline => None, // baseline 不适用于 justify-content
    }
}

/// 转换 AlignmentValue 到 taffy AlignContent。
fn convert_alignment_to_align_content(
    value: &AlignmentValue,
) -> Option<taffy::style::AlignContent> {
    match value {
        AlignmentValue::FlexStart => Some(taffy::style::AlignContent::FlexStart),
        AlignmentValue::FlexEnd => Some(taffy::style::AlignContent::FlexEnd),
        AlignmentValue::Center => Some(taffy::style::AlignContent::Center),
        AlignmentValue::SpaceBetween => Some(taffy::style::AlignContent::SpaceBetween),
        AlignmentValue::SpaceAround => Some(taffy::style::AlignContent::SpaceAround),
        AlignmentValue::SpaceEvenly => Some(taffy::style::AlignContent::SpaceEvenly),
        AlignmentValue::Stretch => Some(taffy::style::AlignContent::Stretch),
        AlignmentValue::Start => Some(taffy::style::AlignContent::Start),
        AlignmentValue::End => Some(taffy::style::AlignContent::End),
        AlignmentValue::Baseline => None,
    }
}

/// 解析 CSS grid track 定义字符串为 taffy TrackSizingFunction 列表。
///
/// 支持的值格式：
/// - `100px` — 固定长度
/// - `1fr` — 弹性轨道
/// - `auto` — 自动轨道
/// - `50%` — 百分比
/// - `minmax(100px, 1fr)` — 最小最大
/// - `repeat(3, 100px)` — 重复
/// - `repeat(auto-fill, 200px)` — 自动填充（展开为单个值）
fn parse_grid_tracks(value: &Option<String>) -> Vec<taffy::style::TrackSizingFunction> {
    let Some(value) = value else {
        return vec![];
    };

    let tokens = tokenize_track_list(value);
    let mut result = Vec::new();

    for token in tokens {
        if let Some(inner) = token
            .strip_prefix("repeat(")
            .and_then(|s| s.strip_suffix(')'))
        {
            result.extend(expand_repeat(inner));
        } else {
            result.push(parse_single_track(&token));
        }
    }

    result
}

/// 将 grid track 列表字符串拆分为独立的 token。
///
/// 与 `split_whitespace` 不同，此函数会识别括号边界，
/// 将 `repeat(...)` 和 `minmax(...)` 保持为单个 token。
fn tokenize_track_list(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for ch in value.chars() {
        match ch {
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth -= 1;
                current.push(ch);
            }
            ' ' | '\t' if depth == 0 => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// 展开 repeat() 函数内部内容为多个 track sizing function。
///
/// 格式：`3, 100px` 或 `auto-fill, 200px` 或 `2, 1fr auto`。
fn expand_repeat(inner: &str) -> Vec<taffy::style::TrackSizingFunction> {
    // 找到第一个不在括号内的逗号
    let comma_pos = find_top_level_comma(inner);
    let Some(comma_pos) = comma_pos else {
        return vec![taffy::style::TrackSizingFunction::AUTO];
    };

    let count_str = inner[..comma_pos].trim();
    let track_list_str = inner[comma_pos + 1..].trim();

    // 解析重复次数：auto-fill/auto-fit → 1（taffy 不原生支持）
    let count: usize = if count_str.eq_ignore_ascii_case("auto-fill")
        || count_str.eq_ignore_ascii_case("auto-fit")
    {
        1
    } else if let Ok(n) = count_str.parse::<usize>() {
        n
    } else {
        return vec![taffy::style::TrackSizingFunction::AUTO];
    };

    // 解析内部 track 列表
    let inner_tokens = tokenize_track_list(track_list_str);
    let inner_tracks: Vec<taffy::style::TrackSizingFunction> =
        inner_tokens.iter().map(|t| parse_single_track(t)).collect();

    // 重复 count 次
    let mut result = Vec::with_capacity(count * inner_tracks.len());
    for _ in 0..count {
        result.extend(inner_tracks.iter().cloned());
    }

    result
}

/// 找到字符串中第一个不在括号内的逗号位置。
fn find_top_level_comma(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => return Some(i),
            _ => {}
        }
    }
    None
}

/// 解析 grid-auto-rows/columns 的 track 定义为 NonRepeatedTrackSizingFunction 列表。
///
/// 与 parse_grid_tracks 类似，但返回 NonRepeatedTrackSizingFunction
/// （不包含 repeat 变体），用于 taffy 的 grid_auto_rows/grid_auto_columns 字段。
fn parse_grid_auto_tracks(
    value: &Option<String>,
) -> Vec<taffy::style::NonRepeatedTrackSizingFunction> {
    let Some(value) = value else {
        return vec![];
    };

    value
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(parse_single_auto_track)
        .collect()
}

/// 解析单个 NonRepeatedTrackSizingFunction 值。
fn parse_single_auto_track(s: &str) -> taffy::style::NonRepeatedTrackSizingFunction {
    use taffy::style::NonRepeatedTrackSizingFunction;

    let s = s.trim();

    if s.eq_ignore_ascii_case("auto") {
        return NonRepeatedTrackSizingFunction::AUTO;
    }
    if s.ends_with("fr")
        && let Ok(flex) = s.trim_end_matches("fr").parse::<f32>()
    {
        return NonRepeatedTrackSizingFunction::from_flex(flex);
    }
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return NonRepeatedTrackSizingFunction::from_percent(pct / 100.0);
    }
    if s.starts_with("minmax(") && s.ends_with(')') {
        return parse_minmax_as_non_repeated(&s[7..s.len() - 1]);
    }
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return NonRepeatedTrackSizingFunction::from_length(px);
    }
    if let Ok(px) = s.parse::<f32>() {
        return NonRepeatedTrackSizingFunction::from_length(px);
    }

    NonRepeatedTrackSizingFunction::AUTO
}

/// 解析 minmax() 函数内部，返回 NonRepeatedTrackSizingFunction。
fn parse_minmax_as_non_repeated(inner: &str) -> taffy::style::NonRepeatedTrackSizingFunction {
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 2 {
        return taffy::style::NonRepeatedTrackSizingFunction::AUTO;
    }

    let min = parse_min_track(parts[0].trim());
    let max = parse_max_track(parts[1].trim());

    taffy::geometry::MinMax { min, max }
}

/// 解析单个 grid track 值。
fn parse_single_track(s: &str) -> taffy::style::TrackSizingFunction {
    use taffy::style::TrackSizingFunction;

    let s = s.trim();

    if s.eq_ignore_ascii_case("auto") {
        return TrackSizingFunction::AUTO;
    }
    if s.ends_with("fr")
        && let Ok(flex) = s.trim_end_matches("fr").parse::<f32>()
    {
        return TrackSizingFunction::from_flex(flex);
    }
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return TrackSizingFunction::from_percent(pct / 100.0);
    }
    if s.starts_with("minmax(") && s.ends_with(')') {
        return parse_minmax(&s[7..s.len() - 1]);
    }
    // 默认尝试解析为长度
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return TrackSizingFunction::from_length(px);
    }
    if let Ok(px) = s.parse::<f32>() {
        return TrackSizingFunction::from_length(px);
    }

    // 无法解析，默认 auto
    TrackSizingFunction::AUTO
}

/// 解析 minmax() 函数内部。
fn parse_minmax(inner: &str) -> taffy::style::TrackSizingFunction {
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 2 {
        return taffy::style::TrackSizingFunction::AUTO;
    }

    let min = parse_min_track(parts[0].trim());
    let max = parse_max_track(parts[1].trim());

    taffy::style::TrackSizingFunction::Single(taffy::geometry::MinMax { min, max })
}

/// 解析 minmax 的最小值。
fn parse_min_track(s: &str) -> taffy::style::MinTrackSizingFunction {
    use taffy::style::MinTrackSizingFunction;

    if s.eq_ignore_ascii_case("auto") {
        return MinTrackSizingFunction::Auto;
    }
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return MinTrackSizingFunction::Fixed(taffy::style::LengthPercentage::Length(px));
    }
    if let Ok(px) = s.parse::<f32>() {
        return MinTrackSizingFunction::Fixed(taffy::style::LengthPercentage::Length(px));
    }

    MinTrackSizingFunction::Auto
}

/// 解析 minmax 的最大值。
fn parse_max_track(s: &str) -> taffy::style::MaxTrackSizingFunction {
    use taffy::style::MaxTrackSizingFunction;

    if s.eq_ignore_ascii_case("auto") {
        return MaxTrackSizingFunction::Auto;
    }
    if s.ends_with("fr")
        && let Ok(flex) = s.trim_end_matches("fr").parse::<f32>()
    {
        return MaxTrackSizingFunction::Fraction(flex);
    }
    if s.ends_with("px")
        && let Ok(px) = s.trim_end_matches("px").parse::<f32>()
    {
        return MaxTrackSizingFunction::Fixed(taffy::style::LengthPercentage::Length(px));
    }
    if let Ok(px) = s.parse::<f32>() {
        return MaxTrackSizingFunction::Fixed(taffy::style::LengthPercentage::Length(px));
    }

    MaxTrackSizingFunction::Auto
}

/// 转换 grid-auto-flow 值。
fn convert_grid_auto_flow(value: &GridAutoFlowValue) -> taffy::style::GridAutoFlow {
    match value {
        GridAutoFlowValue::Row => taffy::style::GridAutoFlow::Row,
        GridAutoFlowValue::Column => taffy::style::GridAutoFlow::Column,
        GridAutoFlowValue::RowDense => taffy::style::GridAutoFlow::RowDense,
        GridAutoFlowValue::ColumnDense => taffy::style::GridAutoFlow::ColumnDense,
    }
}

/// 转换 GridLineValue 到 taffy GridPlacement。
fn convert_grid_line(value: &GridLineValue) -> taffy::style::GridPlacement {
    match value {
        GridLineValue::Auto => taffy::style::GridPlacement::Auto,
        GridLineValue::Line(n) => taffy::style::GridPlacement::from_line_index(*n),
        GridLineValue::Span(s) => taffy::style::GridPlacement::from_span(*s),
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use zero_css_parser::values::LengthValue;
    use zero_style_system::ComputedStyle;

    /// 测试 Block display 转换。
    #[test]
    fn test_convert_block_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Block;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.display, taffy::style::Display::Block);
    }

    /// 测试 Flex display 转换。
    #[test]
    fn test_convert_flex_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.display, taffy::style::Display::Flex);
    }

    /// 测试 Grid display 转换。
    #[test]
    fn test_convert_grid_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.display, taffy::style::Display::Grid);
    }

    /// 测试 None display 转换。
    #[test]
    fn test_convert_none_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::None;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.display, taffy::style::Display::None);
    }

    /// 测试 Inline display 映射为 Block。
    #[test]
    fn test_convert_inline_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Inline;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.display, taffy::style::Display::Block);

        style.display = DisplayValue::InlineBlock;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.display, taffy::style::Display::Block);
    }

    /// 测试 position: absolute 转换。
    #[test]
    fn test_convert_position_absolute() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Absolute;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.position, taffy::style::Position::Absolute);
    }

    /// 测试 position: relative 转换。
    #[test]
    fn test_convert_position_relative() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Relative;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.position, taffy::style::Position::Relative);

        style.position = PositionValue::Static;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.position, taffy::style::Position::Relative);
    }

    /// 测试 size px 转换。
    #[test]
    fn test_convert_size_px() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Px(200.0);
        style.height = LengthValue::Px(100.0);
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.size.width,
            taffy::style::Dimension::Length(200.0)
        );
        assert_eq!(
            taffy_style.size.height,
            taffy::style::Dimension::Length(100.0)
        );
    }

    /// 测试 size auto 转换（Px(0.0) 表示 auto）。
    #[test]
    fn test_convert_size_auto() {
        let style = ComputedStyle::default();
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Auto);
        assert_eq!(taffy_style.size.height, taffy::style::Dimension::Auto);
    }

    /// 测试 margin/padding/border 转换。
    #[test]
    fn test_convert_margin_padding_border() {
        let mut style = ComputedStyle::default();
        style.margin_top = LengthValue::Px(10.0);
        style.margin_right = LengthValue::Px(20.0);
        style.margin_bottom = LengthValue::Px(10.0);
        style.margin_left = LengthValue::Px(20.0);
        style.padding_top = LengthValue::Px(5.0);
        style.padding_right = LengthValue::Px(10.0);
        style.padding_bottom = LengthValue::Px(5.0);
        style.padding_left = LengthValue::Px(10.0);
        style.border_top_width = LengthValue::Px(1.0);
        style.border_right_width = LengthValue::Px(2.0);
        style.border_bottom_width = LengthValue::Px(1.0);
        style.border_left_width = LengthValue::Px(2.0);
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.margin.top,
            taffy::style::LengthPercentageAuto::Length(10.0)
        );
        assert_eq!(
            taffy_style.margin.left,
            taffy::style::LengthPercentageAuto::Length(20.0)
        );
        assert_eq!(
            taffy_style.padding.top,
            taffy::style::LengthPercentage::Length(5.0)
        );
        assert_eq!(
            taffy_style.border.top,
            taffy::style::LengthPercentage::Length(1.0)
        );
    }

    /// 测试 flex 相关属性转换。
    #[test]
    fn test_convert_flex_properties() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        style.flex_direction = FlexDirectionValue::Column;
        style.flex_wrap = FlexWrapValue::Wrap;
        style.flex_grow = 2.0;
        style.flex_shrink = 0.5;
        style.flex_basis = FlexBasisValue::Length(LengthValue::Px(100.0));
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.flex_direction,
            taffy::style::FlexDirection::Column
        );
        assert_eq!(taffy_style.flex_wrap, taffy::style::FlexWrap::Wrap);
        assert!((taffy_style.flex_grow - 2.0).abs() < 0.001);
        assert!((taffy_style.flex_shrink - 0.5).abs() < 0.001);
        assert_eq!(
            taffy_style.flex_basis,
            taffy::style::Dimension::Length(100.0)
        );
    }

    /// 测试对齐属性转换。
    #[test]
    fn test_convert_alignment() {
        let mut style = ComputedStyle::default();
        style.justify_content = AlignmentValue::Center;
        style.align_items = AlignmentValue::FlexEnd;
        style.align_self = AlignmentValue::Baseline;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.justify_content,
            Some(taffy::style::JustifyContent::Center)
        );
        assert_eq!(
            taffy_style.align_items,
            Some(taffy::style::AlignItems::FlexEnd)
        );
        assert_eq!(
            taffy_style.align_self,
            Some(taffy::style::AlignSelf::Baseline)
        );
    }

    /// 测试 gap 转换（column-gap 和 row-gap 独立）。
    #[test]
    fn test_convert_gap() {
        let mut style = ComputedStyle::default();
        style.gap = LengthValue::Px(10.0);
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.gap.width,
            taffy::style::LengthPercentage::Length(10.0)
        );
        // row_gap 默认 Px(0.0)
        assert_eq!(
            taffy_style.gap.height,
            taffy::style::LengthPercentage::Length(0.0)
        );

        // 设置不同的 row-gap
        style.row_gap = LengthValue::Px(20.0);
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.gap.width,
            taffy::style::LengthPercentage::Length(10.0)
        );
        assert_eq!(
            taffy_style.gap.height,
            taffy::style::LengthPercentage::Length(20.0)
        );
    }

    /// 测试 overflow 转换。
    #[test]
    fn test_convert_overflow() {
        let mut style = ComputedStyle::default();
        style.overflow_x = OverflowValue::Hidden;
        style.overflow_y = OverflowValue::Scroll;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.overflow.x, taffy::style::Overflow::Hidden);
        assert_eq!(taffy_style.overflow.y, taffy::style::Overflow::Scroll);
    }

    /// 测试绝对定位的 inset 转换。
    #[test]
    fn test_convert_absolute_position_inset() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Absolute;
        style.top = LengthValue::Px(10.0);
        style.right = LengthValue::Px(20.0);
        style.bottom = LengthValue::Px(30.0);
        style.left = LengthValue::Px(40.0);
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.inset.top,
            taffy::style::LengthPercentageAuto::Length(10.0)
        );
        assert_eq!(
            taffy_style.inset.right,
            taffy::style::LengthPercentageAuto::Length(20.0)
        );
        assert_eq!(
            taffy_style.inset.bottom,
            taffy::style::LengthPercentageAuto::Length(30.0)
        );
        assert_eq!(
            taffy_style.inset.left,
            taffy::style::LengthPercentageAuto::Length(40.0)
        );
    }

    /// 测试 box-sizing 转换。
    #[test]
    fn test_convert_box_sizing() {
        let mut style = ComputedStyle::default();
        style.box_sizing = BoxSizingValue::BorderBox;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.box_sizing, taffy::style::BoxSizing::BorderBox);

        style.box_sizing = BoxSizingValue::ContentBox;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.box_sizing, taffy::style::BoxSizing::ContentBox);
    }

    /// 测试 grid-template-columns/rows 转换。
    #[test]
    fn test_convert_grid_template() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        style.grid_template_columns = Some("100px 200px 1fr".to_string());
        style.grid_template_rows = Some("auto 50px".to_string());
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.display, taffy::style::Display::Grid);
        assert_eq!(taffy_style.grid_template_columns.len(), 3);
        assert_eq!(taffy_style.grid_template_rows.len(), 2);
    }

    /// 测试 grid-auto-flow 转换。
    #[test]
    fn test_convert_grid_auto_flow() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        style.grid_auto_flow = GridAutoFlowValue::Column;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.grid_auto_flow,
            taffy::style::GridAutoFlow::Column
        );

        style.grid_auto_flow = GridAutoFlowValue::RowDense;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.grid_auto_flow,
            taffy::style::GridAutoFlow::RowDense
        );
    }

    /// 测试 row-gap 转换。
    #[test]
    fn test_convert_row_gap() {
        let mut style = ComputedStyle::default();
        style.gap = LengthValue::Px(10.0);
        style.row_gap = LengthValue::Px(20.0);
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.gap.width,
            taffy::style::LengthPercentage::Length(10.0)
        );
        assert_eq!(
            taffy_style.gap.height,
            taffy::style::LengthPercentage::Length(20.0)
        );
    }

    /// 测试 grid-column/row 转换。
    #[test]
    fn test_convert_grid_placement() {
        use zero_style_system::GridLineValue;
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        style.grid_column_start = GridLineValue::Line(1);
        style.grid_column_end = GridLineValue::Line(3);
        style.grid_row_start = GridLineValue::Line(2);
        style.grid_row_end = GridLineValue::Auto;
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.grid_column.start,
            taffy::style::GridPlacement::from_line_index(1)
        );
        assert_eq!(
            taffy_style.grid_column.end,
            taffy::style::GridPlacement::from_line_index(3)
        );
        assert_eq!(
            taffy_style.grid_row.start,
            taffy::style::GridPlacement::from_line_index(2)
        );
        assert_eq!(taffy_style.grid_row.end, taffy::style::GridPlacement::Auto);
    }

    /// 测试 grid span 转换。
    #[test]
    fn test_convert_grid_span() {
        use zero_style_system::GridLineValue;
        let mut style = ComputedStyle::default();
        style.grid_column_start = GridLineValue::Span(2);
        style.grid_row_start = GridLineValue::Line(-1);
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(
            taffy_style.grid_column.start,
            taffy::style::GridPlacement::from_span(2)
        );
        assert_eq!(
            taffy_style.grid_row.start,
            taffy::style::GridPlacement::from_line_index(-1)
        );
    }

    /// 测试 repeat() 固定次数展开。
    #[test]
    fn test_parse_grid_tracks_repeat_fixed() {
        let tracks = parse_grid_tracks(&Some("repeat(3, 100px)".to_string()));
        assert_eq!(tracks.len(), 3);

        let tracks = parse_grid_tracks(&Some("repeat(2, 1fr auto)".to_string()));
        assert_eq!(tracks.len(), 4);
    }

    /// 测试 repeat() auto-fill/auto-fit 降级为单次展开。
    #[test]
    fn test_parse_grid_tracks_repeat_auto_fill() {
        let tracks = parse_grid_tracks(&Some("repeat(auto-fill, 200px)".to_string()));
        assert_eq!(tracks.len(), 1);

        let tracks = parse_grid_tracks(&Some("repeat(auto-fit, minmax(100px, 1fr))".to_string()));
        assert_eq!(tracks.len(), 1);
    }

    /// 测试 repeat() 与普通 track 值混用。
    #[test]
    fn test_parse_grid_tracks_repeat_mixed() {
        let tracks = parse_grid_tracks(&Some("50px repeat(2, 1fr) 100px".to_string()));
        assert_eq!(tracks.len(), 4); // 50px + 1fr + 1fr + 100px
    }

    /// 测试 grid-auto-rows 转换。
    #[test]
    fn test_convert_grid_auto_rows() {
        let mut style = ComputedStyle::default();
        style.grid_auto_rows = Some("100px 200px".to_string());
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.grid_auto_rows.len(), 2);
    }

    /// 测试 grid-auto-columns 转换。
    #[test]
    fn test_convert_grid_auto_columns() {
        let mut style = ComputedStyle::default();
        style.grid_auto_columns = Some("1fr auto".to_string());
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.grid_auto_columns.len(), 2);
    }

    /// 测试 grid-auto-rows/columns 默认值为空。
    #[test]
    fn test_convert_grid_auto_default() {
        let style = ComputedStyle::default();
        let taffy_style = computed_style_to_taffy(&style);
        assert_eq!(taffy_style.grid_auto_rows.len(), 0);
        assert_eq!(taffy_style.grid_auto_columns.len(), 0);
    }
}
