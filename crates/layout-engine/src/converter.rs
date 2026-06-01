//! ComputedStyle → taffy::Style 转换层。
//!
//! 将 [`ComputedStyle`] 的字段映射到 taffy 的 [`taffy::Style`] 结构体，
//! 这是布局引擎的关键适配层。

use zero_css_parser::values::{
    AlignmentValue, BoxSizingValue, ClearValue, DisplayValue, FlexDirectionValue, FlexWrapValue, FloatValue,
    LengthValue, OverflowValue, PositionValue,
};
use zero_style_system::{ComputedStyle, FlexBasisValue, GridAutoFlowValue, GridLineValue};

use taffy::prelude::*;

/// grid-template-areas 区域映射类型。
///
/// 键为区域名（如 "header"），值为 (row_start, row_end, col_start, col_end)，
/// 行号和列号均为 1-based，区间为 [start, end)。
pub type GridAreaMap = std::collections::HashMap<String, (i16, i16, i16, i16)>;

/// 将 ComputedStyle 转换为 taffy::Style。
///
/// 处理所有 CSS 属性到 taffy 布局属性的映射。
/// `parent_areas` 为父级 grid 容器的 grid-template-areas 区域映射，
/// 用于将子元素的 GridLineValue::Name 解析为行号。
pub fn computed_style_to_taffy(style: &ComputedStyle, parent_areas: Option<&GridAreaMap>) -> taffy::Style {
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
        aspect_ratio: style.aspect_ratio,
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
        grid_row: {
            let rs = resolve_named_area(&style.grid_row_start, parent_areas, "row-start");
            let re = resolve_named_area(&style.grid_row_end, parent_areas, "row-end");
            taffy::geometry::Line {
                start: convert_grid_line(&rs),
                end: convert_grid_line(&re),
            }
        },
        grid_column: {
            let cs = resolve_named_area(&style.grid_column_start, parent_areas, "col-start");
            let ce = resolve_named_area(&style.grid_column_end, parent_areas, "col-end");
            taffy::geometry::Line {
                start: convert_grid_line(&cs),
                end: convert_grid_line(&ce),
            }
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
///
/// - `Fixed` 映射为 `Absolute`：使元素脱离正常流，inset 相对于初始包含块（视口）。
///   后续由引擎后处理将坐标调整为视口相对。
/// - `Sticky` 映射为 `Relative`：taffy 无原生 sticky 支持，正常流布局，
///   由宿主层在滚动时动态调整偏移。
fn convert_position(value: &PositionValue) -> taffy::style::Position {
    match value {
        PositionValue::Absolute => taffy::style::Position::Absolute,
        // fixed 需要脱离正常流，使用 Absolute 让 taffy 应用 inset
        PositionValue::Fixed => taffy::style::Position::Absolute,
        // sticky 和 relative/static 一样参与正常流
        PositionValue::Sticky | PositionValue::Relative | PositionValue::Static => taffy::style::Position::Relative,
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

/// 转换 float 属性。
///
/// Taffy 0.7 不直接支持 float 布局，此函数将 FloatValue 映射为布尔值
/// 供布局引擎在构建布局树时判断元素是否需要浮动处理。
/// - `None` → 不浮动
/// - `Left` / `Right` / `InlineStart` / `InlineEnd` → 浮动
pub fn convert_float(value: &FloatValue) -> bool {
    !matches!(value, FloatValue::None)
}

/// 转换 clear 属性。
///
/// Taffy 0.7 不直接支持 clear 布局，此函数将 ClearValue 映射为布尔值
/// 供布局引擎在构建布局树时判断元素是否需要清除浮动。
/// - `None` → 不清除
/// - `Left` / `Right` / `Both` / `InlineStart` / `InlineEnd` → 清除
pub fn convert_clear(value: &ClearValue) -> bool {
    !matches!(value, ClearValue::None)
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
        // Calc 表达式应由 style-system 的 resolve_computed_style 解析为 Px
        LengthValue::Calc(_) => length(0.0),
        // fit-content() 将内部值转换为 dimension
        LengthValue::FitContent(inner) => convert_length_to_dimension(inner),
        // min-content/max-content 映射为 Auto（由 taffy 内部处理内容尺寸）
        LengthValue::MinContent | LengthValue::MaxContent => taffy::style::Dimension::Auto,
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
        LengthValue::Calc(_) => length(0.0),
        LengthValue::FitContent(inner) => convert_max_length_to_dimension(inner),
        LengthValue::MinContent | LengthValue::MaxContent => taffy::style::Dimension::Auto,
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
        LengthValue::Calc(_) => length(0.0),
        LengthValue::FitContent(inner) => convert_length_to_lp(inner),
        LengthValue::MinContent | LengthValue::MaxContent => length(0.0),
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
        LengthValue::Percentage(v) => taffy::style::LengthPercentageAuto::Percent((*v / 100.0) as f32),
        LengthValue::Auto => taffy::style::LengthPercentageAuto::Auto,
        LengthValue::Calc(_) => length(0.0),
        LengthValue::FitContent(inner) => convert_length_to_lpa(inner),
        LengthValue::MinContent | LengthValue::MaxContent => length(0.0),
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
        AlignmentValue::SpaceBetween | AlignmentValue::SpaceAround | AlignmentValue::SpaceEvenly => None,
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
        AlignmentValue::SpaceBetween | AlignmentValue::SpaceAround | AlignmentValue::SpaceEvenly => None,
    }
}

/// 转换 AlignmentValue 到 taffy JustifyContent。
fn convert_alignment_to_justify_content(value: &AlignmentValue) -> Option<taffy::style::JustifyContent> {
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
fn convert_alignment_to_align_content(value: &AlignmentValue) -> Option<taffy::style::AlignContent> {
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
/// - `repeat(auto-fill, 200px)` — 自动填充（传递给 taffy 原生 auto-fill）
fn parse_grid_tracks(value: &Option<String>) -> Vec<taffy::style::TrackSizingFunction> {
    let Some(value) = value else {
        return vec![];
    };

    let tokens = tokenize_track_list(value);
    let mut result = Vec::new();

    for token in tokens {
        if let Some(inner) = token.strip_prefix("repeat(").and_then(|s| s.strip_suffix(')')) {
            result.extend(parse_repeat(inner));
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

/// 解析 repeat() 函数内部内容为 track sizing function 列表。
///
/// 格式：`3, 100px` 或 `auto-fill, 200px` 或 `2, 1fr auto`。
///
/// 对于 auto-fill/auto-fit，生成 `TrackSizingFunction::Repeat` 变体，
/// 利用 taffy 原生的 auto-fill 支持，根据容器宽度自动计算轨道数量。
/// 对于固定次数，直接展开为对应数量的轨道。
fn parse_repeat(inner: &str) -> Vec<taffy::style::TrackSizingFunction> {
    use taffy::style::GridTrackRepetition;

    // 找到第一个不在括号内的逗号
    let comma_pos = find_top_level_comma(inner);
    let Some(comma_pos) = comma_pos else {
        return vec![taffy::style::TrackSizingFunction::AUTO];
    };

    let count_str = inner[..comma_pos].trim();
    let track_list_str = inner[comma_pos + 1..].trim();

    // 解析内部 track 列表为 NonRepeatedTrackSizingFunction
    let inner_tokens = tokenize_track_list(track_list_str);
    let inner_tracks: Vec<taffy::style::NonRepeatedTrackSizingFunction> = inner_tokens
        .iter()
        .map(|t| parse_single_track_as_non_repeated(t))
        .collect();

    if count_str.eq_ignore_ascii_case("auto-fill") {
        // 传递给 taffy 原生 auto-fill，自动根据容器宽度计算轨道数量
        return vec![taffy::style::TrackSizingFunction::Repeat(
            GridTrackRepetition::AutoFill,
            inner_tracks,
        )];
    }

    if count_str.eq_ignore_ascii_case("auto-fit") {
        // 传递给 taffy 原生 auto-fit
        return vec![taffy::style::TrackSizingFunction::Repeat(
            GridTrackRepetition::AutoFit,
            inner_tracks,
        )];
    }

    // 固定次数：展开为对应数量的轨道
    let Ok(count) = count_str.parse::<usize>() else {
        return vec![taffy::style::TrackSizingFunction::AUTO];
    };

    let mut result = Vec::with_capacity(count * inner_tracks.len());
    for _ in 0..count {
        result.extend(
            inner_tracks
                .iter()
                .map(|t| taffy::style::TrackSizingFunction::Single(*t)),
        );
    }

    result
}

/// 将单个 track 值解析为 NonRepeatedTrackSizingFunction。
///
/// 用于 repeat() 内部轨道列表的解析。
fn parse_single_track_as_non_repeated(s: &str) -> taffy::style::NonRepeatedTrackSizingFunction {
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
fn parse_grid_auto_tracks(value: &Option<String>) -> Vec<taffy::style::NonRepeatedTrackSizingFunction> {
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
///
/// 支持 auto、px、百分比（%）和纯数字。
fn parse_min_track(s: &str) -> taffy::style::MinTrackSizingFunction {
    use taffy::style::MinTrackSizingFunction;

    if s.eq_ignore_ascii_case("auto") {
        return MinTrackSizingFunction::Auto;
    }
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return MinTrackSizingFunction::Fixed(taffy::style::LengthPercentage::Percent(pct / 100.0));
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
///
/// 支持 auto、fr、px、百分比（%）和纯数字。
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
    if s.ends_with('%')
        && let Ok(pct) = s.trim_end_matches('%').parse::<f32>()
    {
        return MaxTrackSizingFunction::Fixed(taffy::style::LengthPercentage::Percent(pct / 100.0));
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
///
/// Name 变体应已由 resolve_named_area 预处理为 Line，
/// 若仍有 Name 则 fallback 到 Auto。
fn convert_grid_line(value: &GridLineValue) -> taffy::style::GridPlacement {
    match value {
        GridLineValue::Auto => taffy::style::GridPlacement::Auto,
        GridLineValue::Line(n) => taffy::style::GridPlacement::from_line_index(*n),
        GridLineValue::Span(s) => taffy::style::GridPlacement::from_span(*s),
        GridLineValue::Name(_) => taffy::style::GridPlacement::Auto,
    }
}

/// 解析 grid-template-areas CSS 字符串为区域映射。
///
/// 输入格式：'"header header" "sidebar main" "sidebar footer"'
/// 返回：HashMap<区域名, (row_start, row_end, col_start, col_end)>
///
/// 行号和列号均为 1-based。区域占据的行/列为 [start, end)，
/// 即 row_end = row_start + span_rows。
///
/// 验证规则：
/// 1. 所有行的列数必须相同（矩形检查）
/// 2. 每个命名区域必须构成一个矩形（非矩形区域会记录警告并忽略）
pub fn parse_grid_template_areas(value: &str) -> GridAreaMap {
    let mut areas = std::collections::HashMap::new();
    let mut row = 1i16;
    // 收集每行的 token 列表，用于后续矩形校验
    let mut rows_tokens: Vec<Vec<String>> = Vec::new();

    // 按引号对分割出每行
    let mut chars = value.chars().peekable();
    while let Some(&ch) = chars.peek() {
        if ch == '"' {
            chars.next(); // 消费开引号
            let mut line = String::new();
            while let Some(&c) = chars.peek() {
                if c == '"' {
                    chars.next(); // 消费闭引号
                    break;
                }
                line.push(c);
                chars.next();
            }

            // 解析行内 token
            let tokens: Vec<&str> = line.split_whitespace().collect();
            rows_tokens.push(tokens.iter().map(|s| s.to_string()).collect());

            for (col_idx, &token) in tokens.iter().enumerate() {
                let col = (col_idx + 1) as i16;

                if let Some(entry) = areas.get_mut(token) {
                    // 扩展现有区域的 row_end 和 col_end
                    let (_, ref mut re, _, ref mut ce) = *entry;
                    if row + 1 > *re {
                        *re = row + 1;
                    }
                    if col + 1 > *ce {
                        *ce = col + 1;
                    }
                } else {
                    areas.insert(token.to_string(), (row, row + 1, col, col + 1));
                }
            }

            row += 1;
        } else {
            chars.next();
        }
    }

    // ── 矩形校验 ──

    // 1. 检查所有行的列数是否一致
    if rows_tokens.len() > 1 {
        let expected_cols = rows_tokens[0].len();
        for (i, tokens) in rows_tokens.iter().enumerate() {
            if tokens.len() != expected_cols {
                tracing::warn!(
                    "grid-template-areas: row {} has {} columns but expected {}, area map may be incorrect",
                    i + 1,
                    tokens.len(),
                    expected_cols
                );
                return areas;
            }
        }
    }

    // 2. 检查每个命名区域是否构成矩形
    //    对每个区域名，记录它在 grid 中出现的所有 (row, col)，
    //    然后验证这些位置是否构成一个完整的矩形。
    if !rows_tokens.is_empty() {
        let num_rows = rows_tokens.len() as i16;
        let num_cols = rows_tokens[0].len() as i16;

        for (name, &(rs, re, cs, ce)) in &areas {
            // 计算预期占据的单元格数
            let expected_count = ((re - rs) * (ce - cs)) as usize;
            // 统计实际出现次数
            let mut actual_count = 0usize;
            for (r, tokens) in rows_tokens.iter().enumerate() {
                let r1 = (r + 1) as i16;
                if r1 < rs || r1 >= re {
                    continue;
                }
                for (c, token) in tokens.iter().enumerate() {
                    let c1 = (c + 1) as i16;
                    if c1 < cs || c1 >= ce {
                        continue;
                    }
                    if token == name {
                        actual_count += 1;
                    }
                }
            }
            if actual_count != expected_count {
                tracing::warn!(
                    "grid-template-areas: area '{}' does not form a rectangle (expected {} cells, found {}), \
                     bounds=({},{},{},{}), grid_size={}x{}",
                    name,
                    expected_count,
                    actual_count,
                    rs,
                    re,
                    cs,
                    ce,
                    num_rows,
                    num_cols
                );
            }
        }
    }

    areas
}

/// 解析子元素的命名区域引用为具体的行号。
///
/// 当子元素的 grid-row-start/end 或 grid-column-start/end 为 Name 时，
/// 查找父级区域映射，将 Name 替换为 Line（区域边界）。
/// `which` 为 "row-start"、"row-end"、"col-start"、"col-end" 之一。
fn resolve_named_area(value: &GridLineValue, parent_areas: Option<&GridAreaMap>, which: &str) -> GridLineValue {
    match value {
        GridLineValue::Name(name) => {
            if let Some(areas) = parent_areas {
                if let Some(&(rs, re, cs, ce)) = areas.get(name) {
                    match which {
                        "row-start" => GridLineValue::Line(rs),
                        "row-end" => GridLineValue::Line(re),
                        "col-start" => GridLineValue::Line(cs),
                        "col-end" => GridLineValue::Line(ce),
                        _ => GridLineValue::Auto,
                    }
                } else {
                    GridLineValue::Auto
                }
            } else {
                GridLineValue::Auto
            }
        }
        other => other.clone(),
    }
}

/// 预处理子元素的 grid line 值，将 Name 引用解析为具体行号。
///
/// 返回解析后的 (row_start, row_end, col_start, col_end)。
pub fn resolve_grid_placement(
    style: &ComputedStyle,
    parent_areas: Option<&GridAreaMap>,
) -> (GridLineValue, GridLineValue, GridLineValue, GridLineValue) {
    let rs = resolve_named_area(&style.grid_row_start, parent_areas, "row-start");
    let re = resolve_named_area(&style.grid_row_end, parent_areas, "row-end");
    let cs = resolve_named_area(&style.grid_column_start, parent_areas, "col-start");
    let ce = resolve_named_area(&style.grid_column_end, parent_areas, "col-end");
    (rs, re, cs, ce)
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
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.display, taffy::style::Display::Block);
    }

    /// 测试 Flex display 转换。
    #[test]
    fn test_convert_flex_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.display, taffy::style::Display::Flex);
    }

    /// 测试 Grid display 转换。
    #[test]
    fn test_convert_grid_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.display, taffy::style::Display::Grid);
    }

    /// 测试 None display 转换。
    #[test]
    fn test_convert_none_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::None;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.display, taffy::style::Display::None);
    }

    /// 测试 Inline display 映射为 Block。
    #[test]
    fn test_convert_inline_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Inline;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.display, taffy::style::Display::Block);

        style.display = DisplayValue::InlineBlock;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.display, taffy::style::Display::Block);
    }

    /// 测试 position: absolute 转换。
    #[test]
    fn test_convert_position_absolute() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Absolute;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.position, taffy::style::Position::Absolute);
    }

    /// 测试 position: relative 转换。
    #[test]
    fn test_convert_position_relative() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Relative;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.position, taffy::style::Position::Relative);

        style.position = PositionValue::Static;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.position, taffy::style::Position::Relative);
    }

    /// 测试 position: fixed 映射为 taffy Absolute（脱离正常流）。
    #[test]
    fn test_convert_position_fixed() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Fixed;
        style.top = LengthValue::Px(10.0);
        style.left = LengthValue::Px(20.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(
            taffy_style.position,
            taffy::style::Position::Absolute,
            "position:fixed should map to taffy Absolute"
        );
        // inset 应正确传递
        assert_eq!(taffy_style.inset.top, taffy::style::LengthPercentageAuto::Length(10.0));
        assert_eq!(taffy_style.inset.left, taffy::style::LengthPercentageAuto::Length(20.0));
    }

    /// 测试 position: sticky 映射为 taffy Relative（保持正常流）。
    #[test]
    fn test_convert_position_sticky() {
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Sticky;
        style.top = LengthValue::Px(5.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(
            taffy_style.position,
            taffy::style::Position::Relative,
            "position:sticky should map to taffy Relative"
        );
    }

    /// 测试 size px 转换。
    #[test]
    fn test_convert_size_px() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Px(200.0);
        style.height = LengthValue::Px(100.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Length(200.0));
        assert_eq!(taffy_style.size.height, taffy::style::Dimension::Length(100.0));
    }

    /// 测试 size auto 转换（Px(0.0) 表示 auto）。
    #[test]
    fn test_convert_size_auto() {
        let style = ComputedStyle::default();
        let taffy_style = computed_style_to_taffy(&style, None);
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
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.margin.top, taffy::style::LengthPercentageAuto::Length(10.0));
        assert_eq!(
            taffy_style.margin.left,
            taffy::style::LengthPercentageAuto::Length(20.0)
        );
        assert_eq!(taffy_style.padding.top, taffy::style::LengthPercentage::Length(5.0));
        assert_eq!(taffy_style.border.top, taffy::style::LengthPercentage::Length(1.0));
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
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.flex_direction, taffy::style::FlexDirection::Column);
        assert_eq!(taffy_style.flex_wrap, taffy::style::FlexWrap::Wrap);
        assert!((taffy_style.flex_grow - 2.0).abs() < 0.001);
        assert!((taffy_style.flex_shrink - 0.5).abs() < 0.001);
        assert_eq!(taffy_style.flex_basis, taffy::style::Dimension::Length(100.0));
    }

    /// 测试对齐属性转换。
    #[test]
    fn test_convert_alignment() {
        let mut style = ComputedStyle::default();
        style.justify_content = AlignmentValue::Center;
        style.align_items = AlignmentValue::FlexEnd;
        style.align_self = AlignmentValue::Baseline;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.justify_content, Some(taffy::style::JustifyContent::Center));
        assert_eq!(taffy_style.align_items, Some(taffy::style::AlignItems::FlexEnd));
        assert_eq!(taffy_style.align_self, Some(taffy::style::AlignSelf::Baseline));
    }

    /// 测试 gap 转换（column-gap 和 row-gap 独立）。
    #[test]
    fn test_convert_gap() {
        let mut style = ComputedStyle::default();
        style.gap = LengthValue::Px(10.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.gap.width, taffy::style::LengthPercentage::Length(10.0));
        // row_gap 默认 Px(0.0)
        assert_eq!(taffy_style.gap.height, taffy::style::LengthPercentage::Length(0.0));

        // 设置不同的 row-gap
        style.row_gap = LengthValue::Px(20.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.gap.width, taffy::style::LengthPercentage::Length(10.0));
        assert_eq!(taffy_style.gap.height, taffy::style::LengthPercentage::Length(20.0));
    }

    /// 测试 overflow 转换。
    #[test]
    fn test_convert_overflow() {
        let mut style = ComputedStyle::default();
        style.overflow_x = OverflowValue::Hidden;
        style.overflow_y = OverflowValue::Scroll;
        let taffy_style = computed_style_to_taffy(&style, None);
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
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.inset.top, taffy::style::LengthPercentageAuto::Length(10.0));
        assert_eq!(
            taffy_style.inset.right,
            taffy::style::LengthPercentageAuto::Length(20.0)
        );
        assert_eq!(
            taffy_style.inset.bottom,
            taffy::style::LengthPercentageAuto::Length(30.0)
        );
        assert_eq!(taffy_style.inset.left, taffy::style::LengthPercentageAuto::Length(40.0));
    }

    /// 测试 box-sizing 转换。
    #[test]
    fn test_convert_box_sizing() {
        let mut style = ComputedStyle::default();
        style.box_sizing = BoxSizingValue::BorderBox;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.box_sizing, taffy::style::BoxSizing::BorderBox);

        style.box_sizing = BoxSizingValue::ContentBox;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.box_sizing, taffy::style::BoxSizing::ContentBox);
    }

    /// 测试 grid-template-columns/rows 转换。
    #[test]
    fn test_convert_grid_template() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        style.grid_template_columns = Some("100px 200px 1fr".to_string());
        style.grid_template_rows = Some("auto 50px".to_string());
        let taffy_style = computed_style_to_taffy(&style, None);
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
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_flow, taffy::style::GridAutoFlow::Column);

        style.grid_auto_flow = GridAutoFlowValue::RowDense;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_flow, taffy::style::GridAutoFlow::RowDense);
    }

    /// 测试 row-gap 转换。
    #[test]
    fn test_convert_row_gap() {
        let mut style = ComputedStyle::default();
        style.gap = LengthValue::Px(10.0);
        style.row_gap = LengthValue::Px(20.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.gap.width, taffy::style::LengthPercentage::Length(10.0));
        assert_eq!(taffy_style.gap.height, taffy::style::LengthPercentage::Length(20.0));
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
        let taffy_style = computed_style_to_taffy(&style, None);
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
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_column.start, taffy::style::GridPlacement::from_span(2));
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

    /// 测试 repeat() auto-fill/auto-fit 生成 Repeat 变体（非展开）。
    #[test]
    fn test_parse_grid_tracks_repeat_auto_fill() {
        use taffy::style::GridTrackRepetition;

        let tracks = parse_grid_tracks(&Some("repeat(auto-fill, 200px)".to_string()));
        assert_eq!(tracks.len(), 1);
        assert!(
            matches!(
                &tracks[0],
                taffy::style::TrackSizingFunction::Repeat(GridTrackRepetition::AutoFill, _)
            ),
            "auto-fill 应生成 Repeat 变体"
        );

        let tracks = parse_grid_tracks(&Some("repeat(auto-fit, minmax(100px, 1fr))".to_string()));
        assert_eq!(tracks.len(), 1);
        assert!(
            matches!(
                &tracks[0],
                taffy::style::TrackSizingFunction::Repeat(GridTrackRepetition::AutoFit, _)
            ),
            "auto-fit 应生成 Repeat 变体"
        );
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
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_rows.len(), 2);
    }

    /// 测试 grid-auto-columns 转换。
    #[test]
    fn test_convert_grid_auto_columns() {
        let mut style = ComputedStyle::default();
        style.grid_auto_columns = Some("1fr auto".to_string());
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_columns.len(), 2);
    }

    /// 测试 grid-auto-rows/columns 默认值为空。
    #[test]
    fn test_convert_grid_auto_default() {
        let style = ComputedStyle::default();
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_rows.len(), 0);
        assert_eq!(taffy_style.grid_auto_columns.len(), 0);
    }

    /// 测试 parse_grid_template_areas 解析 2x2 区域。
    #[test]
    fn test_parse_grid_template_areas_2x2() {
        let areas = parse_grid_template_areas("\"header header\" \"sidebar main\"");
        assert_eq!(areas.len(), 3); // header, sidebar, main

        // header: row 1-2, col 1-3（跨两列）
        assert_eq!(areas.get("header"), Some(&(1, 2, 1, 3)));
        // sidebar: row 2-3, col 1-2
        assert_eq!(areas.get("sidebar"), Some(&(2, 3, 1, 2)));
        // main: row 2-3, col 2-3
        assert_eq!(areas.get("main"), Some(&(2, 3, 2, 3)));
    }

    /// 测试 parse_grid_template_areas 解析 3x3 区域。
    #[test]
    fn test_parse_grid_template_areas_3x3() {
        let areas =
            parse_grid_template_areas("\"header header header\" \"sidebar main main\" \"sidebar footer footer\"");
        assert_eq!(areas.len(), 4);

        // header: row 1-2, col 1-4（跨三列）
        assert_eq!(areas.get("header"), Some(&(1, 2, 1, 4)));
        // sidebar: row 2-4, col 1-2（跨两行）
        assert_eq!(areas.get("sidebar"), Some(&(2, 4, 1, 2)));
        // main: row 2-3, col 2-4（跨两列）
        assert_eq!(areas.get("main"), Some(&(2, 3, 2, 4)));
        // footer: row 3-4, col 2-4（跨两列）
        assert_eq!(areas.get("footer"), Some(&(3, 4, 2, 4)));
    }

    /// 测试 parse_grid_template_areas 空输入。
    #[test]
    fn test_parse_grid_template_areas_empty() {
        let areas = parse_grid_template_areas("");
        assert!(areas.is_empty());

        let areas = parse_grid_template_areas("none");
        assert!(areas.is_empty());
    }

    /// 测试 resolve_named_area 将 Name 解析为 Line。
    #[test]
    fn test_resolve_named_area_with_map() {
        use zero_style_system::GridLineValue;

        let mut areas = std::collections::HashMap::new();
        areas.insert("header".to_string(), (1, 2, 1, 3));
        areas.insert("sidebar".to_string(), (2, 3, 1, 2));

        // Name 被解析
        let val = resolve_named_area(&GridLineValue::Name("header".to_string()), Some(&areas), "row-start");
        assert_eq!(val, GridLineValue::Line(1));

        let val = resolve_named_area(&GridLineValue::Name("header".to_string()), Some(&areas), "col-end");
        assert_eq!(val, GridLineValue::Line(3));

        // 不存在的名称 → Auto
        let val = resolve_named_area(
            &GridLineValue::Name("nonexistent".to_string()),
            Some(&areas),
            "row-start",
        );
        assert_eq!(val, GridLineValue::Auto);

        // 没有 area map → Auto
        let val = resolve_named_area(&GridLineValue::Name("header".to_string()), None, "row-start");
        assert_eq!(val, GridLineValue::Auto);

        // 非 Name 值不变
        let val = resolve_named_area(&GridLineValue::Line(2), Some(&areas), "row-start");
        assert_eq!(val, GridLineValue::Line(2));
    }

    // ── float/clear 转换测试 ──

    /// 测试 float: none 不触发浮动。
    #[test]
    fn test_convert_float_none() {
        assert!(!convert_float(&FloatValue::None));
    }

    /// 测试 float: left 触发浮动。
    #[test]
    fn test_convert_float_left() {
        assert!(convert_float(&FloatValue::Left));
    }

    /// 测试 float: right 触发浮动。
    #[test]
    fn test_convert_float_right() {
        assert!(convert_float(&FloatValue::Right));
    }

    /// 测试 float: inline-start 触发浮动。
    #[test]
    fn test_convert_float_inline_start() {
        assert!(convert_float(&FloatValue::InlineStart));
    }

    /// 测试 float: inline-end 触发浮动。
    #[test]
    fn test_convert_float_inline_end() {
        assert!(convert_float(&FloatValue::InlineEnd));
    }

    /// 测试 clear: none 不触发清除浮动。
    #[test]
    fn test_convert_clear_none() {
        assert!(!convert_clear(&ClearValue::None));
    }

    /// 测试 clear: left 触发清除浮动。
    #[test]
    fn test_convert_clear_left() {
        assert!(convert_clear(&ClearValue::Left));
    }

    /// 测试 clear: right 触发清除浮动。
    #[test]
    fn test_convert_clear_right() {
        assert!(convert_clear(&ClearValue::Right));
    }

    /// 测试 clear: both 触发清除浮动。
    #[test]
    fn test_convert_clear_both() {
        assert!(convert_clear(&ClearValue::Both));
    }

    /// 测试 clear: inline-start 触发清除浮动。
    #[test]
    fn test_convert_clear_inline_start() {
        assert!(convert_clear(&ClearValue::InlineStart));
    }

    /// 测试 clear: inline-end 触发清除浮动。
    #[test]
    fn test_convert_clear_inline_end() {
        assert!(convert_clear(&ClearValue::InlineEnd));
    }

    /// 测试 ComputedStyle 中 float/clear 默认值为 None。
    #[test]
    fn test_default_float_clear_in_computed_style() {
        let style = ComputedStyle::default();
        assert_eq!(style.float, FloatValue::None);
        assert_eq!(style.clear, ClearValue::None);
        assert!(!convert_float(&style.float));
        assert!(!convert_clear(&style.clear));
    }

    // ── 新增补充测试 ──

    /// 测试 grid area name resolution — resolve_grid_placement 将 Name 解析为 Line。
    ///
    /// 当 grid-template-areas 定义了 "nav" 区域时，
    /// 子元素设置 grid-area: "nav" 应被解析为具体的行号和列号。
    #[test]
    fn test_grid_area_name_resolution() {
        use zero_style_system::GridLineValue;

        let areas = parse_grid_template_areas("\"header header\" \"nav main\" \"footer footer\"");

        // nav 区域应为 (2, 3, 1, 2) — row 2-3, col 1-2
        assert_eq!(areas.get("nav"), Some(&(2, 3, 1, 2)));

        // 创建一个 ComputedStyle 并验证 resolve_grid_placement
        let mut style = ComputedStyle::default();
        style.grid_row_start = GridLineValue::Name("nav".to_string());
        style.grid_row_end = GridLineValue::Name("nav".to_string());
        style.grid_column_start = GridLineValue::Name("nav".to_string());
        style.grid_column_end = GridLineValue::Name("nav".to_string());

        let (rs, re, cs, ce) = resolve_grid_placement(&style, Some(&areas));
        assert_eq!(rs, GridLineValue::Line(2), "row-start should be 2");
        assert_eq!(re, GridLineValue::Line(3), "row-end should be 3");
        assert_eq!(cs, GridLineValue::Line(1), "col-start should be 1");
        assert_eq!(ce, GridLineValue::Line(2), "col-end should be 2");
    }

    /// 测试 minmax() 中 auto 作为最小值和最大值。
    #[test]
    fn test_minmax_with_auto() {
        // minmax(auto, 1fr) — min=auto, max=1fr
        let tracks = parse_grid_tracks(&Some("minmax(auto, 1fr)".to_string()));
        assert_eq!(tracks.len(), 1, "应产生 1 个轨道");

        // minmax(50px, auto) — min=50px, max=auto
        let tracks = parse_grid_tracks(&Some("minmax(50px, auto)".to_string()));
        assert_eq!(tracks.len(), 1, "应产生 1 个轨道");

        // 混合使用：minmax(auto, 1fr) minmax(100px, auto)
        let tracks = parse_grid_tracks(&Some("minmax(auto, 1fr) minmax(100px, auto)".to_string()));
        assert_eq!(tracks.len(), 2, "应产生 2 个轨道");
    }

    /// 测试复杂的 grid-template-areas 模式。
    ///
    /// 3x3 区域布局：
    ///   "header header header"
    ///   "nav    main   aside"
    ///   "footer footer footer"
    /// 验证每个区域的坐标范围正确。
    #[test]
    fn test_complex_grid_template_areas_pattern() {
        let areas = parse_grid_template_areas("\"header header header\" \"nav main aside\" \"footer footer footer\"");

        assert_eq!(areas.len(), 5, "应有 5 个区域");

        // header: row 1-2, col 1-4（跨 3 列）
        assert_eq!(areas.get("header"), Some(&(1, 2, 1, 4)));

        // nav: row 2-3, col 1-2
        assert_eq!(areas.get("nav"), Some(&(2, 3, 1, 2)));

        // main: row 2-3, col 2-3
        assert_eq!(areas.get("main"), Some(&(2, 3, 2, 3)));

        // aside: row 2-3, col 3-4
        assert_eq!(areas.get("aside"), Some(&(2, 3, 3, 4)));

        // footer: row 3-4, col 1-4（跨 3 列）
        assert_eq!(areas.get("footer"), Some(&(3, 4, 1, 4)));
    }

    /// 测试 aspect-ratio 在 taffy Style 中的传递。
    #[test]
    fn test_aspect_ratio_in_taffy_style() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Px(200.0);
        style.aspect_ratio = Some(1.5); // 宽/高比 = 1.5

        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.aspect_ratio, Some(1.5));
    }

    /// 测试 aspect-ratio 为 None 时 taffy Style 中也为 None。
    #[test]
    fn test_aspect_ratio_none_in_taffy_style() {
        let style = ComputedStyle::default();
        assert_eq!(style.aspect_ratio, None);

        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.aspect_ratio, None);
    }

    /// 测试 float 元素在 flex 容器中的转换 — float 在 flex 上下文中应仍返回 true。
    #[test]
    fn test_float_in_mixed_layout_context() {
        // float: left
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        style.float = FloatValue::Left;
        let taffy_style = computed_style_to_taffy(&style, None);

        // taffy 中 float 不影响 flex 容器本身
        assert_eq!(taffy_style.display, taffy::style::Display::Flex);

        // 但 convert_float 应返回 true
        assert!(convert_float(&FloatValue::Left));

        // clear: both 也应返回 true
        assert!(convert_clear(&ClearValue::Both));
    }

    /// 测试 repeat(auto-fill, minmax(auto, 1fr)) 解析。
    ///
    /// min 侧为 auto，max 侧为 1fr，验证解析不 panic 且生成 Repeat 变体。
    #[test]
    fn test_parse_repeat_auto_fill_minmax_auto() {
        use taffy::style::GridTrackRepetition;

        let tracks = parse_grid_tracks(&Some("repeat(auto-fill, minmax(auto, 1fr))".to_string()));
        assert_eq!(tracks.len(), 1);
        assert!(
            matches!(
                &tracks[0],
                taffy::style::TrackSizingFunction::Repeat(GridTrackRepetition::AutoFill, _)
            ),
            "auto-fill + minmax(auto, 1fr) 应生成 Repeat 变体"
        );
    }

    /// 测试 grid-auto-rows 使用固定值和 fr 单位。
    #[test]
    fn test_grid_auto_rows_with_various_values() {
        let mut style = ComputedStyle::default();
        style.grid_auto_rows = Some("50px auto".to_string());
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_rows.len(), 2);

        // 单值
        style.grid_auto_rows = Some("100px".to_string());
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_rows.len(), 1);

        // fr 单位
        style.grid_auto_rows = Some("1fr".to_string());
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_rows.len(), 1);
    }

    // -- 边界条件测试 --

    /// 测试 aspect-ratio auto 不设置值
    #[test]
    fn test_aspect_ratio_auto_conversion() {
        // aspect_ratio 为 None（Auto）时，taffy style 中应为 None
        let style = ComputedStyle::default();
        assert_eq!(style.aspect_ratio, None);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.aspect_ratio, None, "auto aspect-ratio 应转换为 None");
    }

    /// 测试 grid-auto-flow dense 转换
    #[test]
    fn test_grid_auto_flow_dense_conversion() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;

        // RowDense
        style.grid_auto_flow = GridAutoFlowValue::RowDense;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_flow, taffy::style::GridAutoFlow::RowDense);

        // ColumnDense
        style.grid_auto_flow = GridAutoFlowValue::ColumnDense;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.grid_auto_flow, taffy::style::GridAutoFlow::ColumnDense);
    }

    /// 测试多个 inset 同时设置
    #[test]
    fn test_all_four_inset_values() {
        // top/right/bottom/left 全部设置，验证全部转换
        let mut style = ComputedStyle::default();
        style.position = PositionValue::Absolute;
        style.top = LengthValue::Px(10.0);
        style.right = LengthValue::Px(20.0);
        style.bottom = LengthValue::Px(30.0);
        style.left = LengthValue::Px(40.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.inset.top, taffy::style::LengthPercentageAuto::Length(10.0));
        assert_eq!(
            taffy_style.inset.right,
            taffy::style::LengthPercentageAuto::Length(20.0)
        );
        assert_eq!(
            taffy_style.inset.bottom,
            taffy::style::LengthPercentageAuto::Length(30.0)
        );
        assert_eq!(taffy_style.inset.left, taffy::style::LengthPercentageAuto::Length(40.0));
    }

    /// 测试 flex-basis: 0 转换
    #[test]
    fn test_flex_basis_zero() {
        // flex-basis: 0px 应转换为 Length(0.0)
        let mut style = ComputedStyle::default();
        style.flex_basis = FlexBasisValue::Length(LengthValue::Px(0.0));
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.flex_basis, taffy::style::Dimension::Length(0.0));
    }

    /// 测试 percentage 宽高转换
    #[test]
    fn test_percentage_size_conversion() {
        // width: 50% 应转换为 Percent(0.5)
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Percentage(50.0);
        style.height = LengthValue::Percentage(75.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Percent(0.5));
        assert_eq!(taffy_style.size.height, taffy::style::Dimension::Percent(0.75));
    }

    // ── 边界条件测试（第二批）──

    /// 测试 InlineFlex display 映射为 taffy::Display::Flex。
    #[test]
    fn test_convert_inline_flex_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::InlineFlex;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.display, taffy::style::Display::Flex);
    }

    /// 测试 InlineGrid display 映射为 taffy::Display::Grid。
    #[test]
    fn test_convert_inline_grid_display() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::InlineGrid;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.display, taffy::style::Display::Grid);
    }

    /// 测试 Flow、FlowRoot、ListItem、Contents 都映射为 taffy::Display::Block。
    #[test]
    fn test_convert_flow_variants_display() {
        let mut style = ComputedStyle::default();
        for value in [
            DisplayValue::Flow,
            DisplayValue::FlowRoot,
            DisplayValue::ListItem,
            DisplayValue::Contents,
        ] {
            style.display = value;
            let taffy_style = computed_style_to_taffy(&style, None);
            assert_eq!(taffy_style.display, taffy::style::Display::Block);
        }
    }

    /// 测试 Em、Rem、Vw、Vh 单位转换为 length(v as f32)。
    #[test]
    fn test_convert_length_em_rem_vw_vh() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Em(16.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Length(16.0));

        style.width = LengthValue::Rem(12.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Length(12.0));

        style.width = LengthValue::Vw(50.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Length(50.0));

        style.width = LengthValue::Vh(25.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Length(25.0));
    }

    /// 测试 Vmin、Vmax、Ch 单位转换为 length(v as f32)。
    #[test]
    fn test_convert_length_vmin_vmax_ch() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::Vmin(10.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Length(10.0));

        style.width = LengthValue::Vmax(20.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Length(20.0));

        style.width = LengthValue::Ch(8.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.size.width, taffy::style::Dimension::Length(8.0));
    }

    /// 测试 LengthValue::Calc 在所有转换函数中映射为 length(0.0)。
    #[test]
    fn test_convert_length_calc_fallback() {
        use zero_css_parser::values::CalcExpr;
        let calc = LengthValue::Calc(Box::new(CalcExpr::Number(42.0)));

        // convert_length_to_dimension
        assert_eq!(convert_length_to_dimension(&calc), taffy::style::Dimension::Length(0.0));

        // convert_max_length_to_dimension
        assert_eq!(
            convert_max_length_to_dimension(&calc),
            taffy::style::Dimension::Length(0.0)
        );

        // convert_length_to_lp
        assert_eq!(convert_length_to_lp(&calc), taffy::style::LengthPercentage::Length(0.0));

        // convert_length_to_lpa
        assert_eq!(
            convert_length_to_lpa(&calc),
            taffy::style::LengthPercentageAuto::Length(0.0)
        );
    }

    /// 测试 max-width/max-height 中 Px(f64::INFINITY) 映射为 Auto。
    #[test]
    fn test_convert_max_length_infinity() {
        let mut style = ComputedStyle::default();
        style.max_width = LengthValue::Px(f64::INFINITY);
        style.max_height = LengthValue::Px(f64::INFINITY);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::Auto);
        assert_eq!(taffy_style.max_size.height, taffy::style::Dimension::Auto);
    }

    /// 测试 max-width 的 Px 和 Percentage 值转换。
    #[test]
    fn test_convert_max_length_px_percentage() {
        let mut style = ComputedStyle::default();
        style.max_width = LengthValue::Px(500.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::Length(500.0));

        style.max_width = LengthValue::Percentage(80.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::Percent(0.8));
    }

    /// 测试 FlexWrap::WrapReverse 映射为 taffy::FlexWrap::WrapReverse。
    #[test]
    fn test_convert_flex_wrap_reverse() {
        let mut style = ComputedStyle::default();
        style.flex_wrap = FlexWrapValue::WrapReverse;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.flex_wrap, taffy::style::FlexWrap::WrapReverse);
    }

    /// 测试 FlexBasisValue::Content 映射为 Auto。
    #[test]
    fn test_convert_flex_basis_content() {
        let mut style = ComputedStyle::default();
        style.flex_basis = FlexBasisValue::Content;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.flex_basis, taffy::style::Dimension::Auto);
    }

    /// 测试 Auto 在 convert_length_to_lp 中映射为 length(0.0)。
    #[test]
    fn test_convert_length_to_lp_auto() {
        let result = convert_length_to_lp(&LengthValue::Auto);
        assert_eq!(result, taffy::style::LengthPercentage::Length(0.0));
    }

    /// 测试 Percentage 在 convert_length_to_lp 中转换为 Percent。
    #[test]
    fn test_convert_length_to_lp_percentage() {
        let result = convert_length_to_lp(&LengthValue::Percentage(33.0));
        assert_eq!(result, taffy::style::LengthPercentage::Percent(0.33));
    }

    /// 测试 Auto 在 convert_length_to_lpa 中映射为 LengthPercentageAuto::Auto。
    #[test]
    fn test_convert_length_to_lpa_auto() {
        let result = convert_length_to_lpa(&LengthValue::Auto);
        assert_eq!(result, taffy::style::LengthPercentageAuto::Auto);
    }

    /// 测试 Percentage 在 convert_length_to_lpa 中转换为 Percent。
    #[test]
    fn test_convert_length_to_lpa_percentage() {
        let result = convert_length_to_lpa(&LengthValue::Percentage(60.0));
        assert_eq!(result, taffy::style::LengthPercentageAuto::Percent(0.6));
    }

    /// 测试 align_content 的所有变体转换。
    ///
    /// 注意：computed_style_to_taffy 中 align_content 使用 style.justify_content，
    /// 所以通过设置 justify_content 来测试 align_content 的转换结果。
    #[test]
    fn test_convert_alignment_align_content() {
        let cases: Vec<(AlignmentValue, Option<taffy::style::AlignContent>)> = vec![
            (
                AlignmentValue::SpaceBetween,
                Some(taffy::style::AlignContent::SpaceBetween),
            ),
            (
                AlignmentValue::SpaceAround,
                Some(taffy::style::AlignContent::SpaceAround),
            ),
            (
                AlignmentValue::SpaceEvenly,
                Some(taffy::style::AlignContent::SpaceEvenly),
            ),
            (AlignmentValue::Stretch, Some(taffy::style::AlignContent::Stretch)),
            (AlignmentValue::FlexStart, Some(taffy::style::AlignContent::FlexStart)),
            (AlignmentValue::FlexEnd, Some(taffy::style::AlignContent::FlexEnd)),
            (AlignmentValue::Center, Some(taffy::style::AlignContent::Center)),
            (AlignmentValue::Start, Some(taffy::style::AlignContent::Start)),
            (AlignmentValue::End, Some(taffy::style::AlignContent::End)),
        ];
        for (value, expected) in cases {
            let mut style = ComputedStyle::default();
            style.justify_content = value;
            let taffy_style = computed_style_to_taffy(&style, None);
            assert_eq!(taffy_style.align_content, expected);
        }
    }

    /// 测试 justify_content 的 SpaceBetween、SpaceAround、SpaceEvenly、Start、End、Stretch 变体。
    #[test]
    fn test_convert_alignment_justify_content_variants() {
        let cases: Vec<(AlignmentValue, Option<taffy::style::JustifyContent>)> = vec![
            (
                AlignmentValue::SpaceBetween,
                Some(taffy::style::JustifyContent::SpaceBetween),
            ),
            (
                AlignmentValue::SpaceAround,
                Some(taffy::style::JustifyContent::SpaceAround),
            ),
            (
                AlignmentValue::SpaceEvenly,
                Some(taffy::style::JustifyContent::SpaceEvenly),
            ),
            (AlignmentValue::Start, Some(taffy::style::JustifyContent::Start)),
            (AlignmentValue::End, Some(taffy::style::JustifyContent::End)),
            (AlignmentValue::Stretch, Some(taffy::style::JustifyContent::Stretch)),
        ];
        for (value, expected) in cases {
            let mut style = ComputedStyle::default();
            style.justify_content = value;
            let taffy_style = computed_style_to_taffy(&style, None);
            assert_eq!(taffy_style.justify_content, expected);
        }
    }

    /// 测试 align_self 的 FlexStart、FlexEnd、Center、Stretch、Start、End 变体。
    #[test]
    fn test_convert_alignment_align_self_variants() {
        let cases: Vec<(AlignmentValue, Option<taffy::style::AlignSelf>)> = vec![
            (AlignmentValue::FlexStart, Some(taffy::style::AlignSelf::FlexStart)),
            (AlignmentValue::FlexEnd, Some(taffy::style::AlignSelf::FlexEnd)),
            (AlignmentValue::Center, Some(taffy::style::AlignSelf::Center)),
            (AlignmentValue::Stretch, Some(taffy::style::AlignSelf::Stretch)),
            (AlignmentValue::Start, Some(taffy::style::AlignSelf::Start)),
            (AlignmentValue::End, Some(taffy::style::AlignSelf::End)),
        ];
        for (value, expected) in cases {
            let mut style = ComputedStyle::default();
            style.align_self = value;
            let taffy_style = computed_style_to_taffy(&style, None);
            assert_eq!(taffy_style.align_self, expected);
        }
    }

    /// 测试 tokenize_track_list 正确处理嵌套括号。
    #[test]
    fn test_tokenized_track_list_nested_parens() {
        let tokens = tokenize_track_list("repeat(2, minmax(10px, 1fr)) 100px");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0], "repeat(2, minmax(10px, 1fr))");
        assert_eq!(tokens[1], "100px");
    }

    /// 测试 parse_minmax_as_non_repeated 参数不足时返回 AUTO。
    #[test]
    fn test_parse_minmax_as_non_repeated_malformed() {
        // 只有一个参数（缺少逗号分隔的第二部分），应返回 AUTO
        let result = parse_minmax_as_non_repeated("100px");
        assert_eq!(result, taffy::style::NonRepeatedTrackSizingFunction::AUTO);
    }

    /// 测试 resolve_named_area 对未知 which 参数返回 Auto。
    #[test]
    fn test_resolve_named_area_unknown_which() {
        use zero_style_system::GridLineValue;
        let mut areas = std::collections::HashMap::new();
        areas.insert("header".to_string(), (1, 2, 1, 3));

        let val = resolve_named_area(
            &GridLineValue::Name("header".to_string()),
            Some(&areas),
            "unknown-param",
        );
        assert_eq!(val, GridLineValue::Auto);
    }

    /// 测试 convert_float 对 Left、Right、InlineStart、InlineEnd 返回 true，None 返回 false。
    #[test]
    fn test_convert_float_variants() {
        assert!(convert_float(&FloatValue::Left));
        assert!(convert_float(&FloatValue::Right));
        assert!(convert_float(&FloatValue::InlineStart));
        assert!(convert_float(&FloatValue::InlineEnd));
        assert!(!convert_float(&FloatValue::None));
    }

    /// 测试 convert_clear 对 Left、Right、Both、InlineStart、InlineEnd 返回 true，None 返回 false。
    #[test]
    fn test_convert_clear_variants() {
        assert!(convert_clear(&ClearValue::Left));
        assert!(convert_clear(&ClearValue::Right));
        assert!(convert_clear(&ClearValue::Both));
        assert!(convert_clear(&ClearValue::InlineStart));
        assert!(convert_clear(&ClearValue::InlineEnd));
        assert!(!convert_clear(&ClearValue::None));
    }

    /// 测试 OverflowValue::Auto 映射为 taffy Scroll。
    #[test]
    fn test_overflow_auto_maps_to_scroll() {
        let mut style = ComputedStyle::default();
        style.overflow_x = OverflowValue::Auto;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.overflow.x, taffy::style::Overflow::Scroll);
    }

    /// 测试 OverflowValue::Clip 映射为 taffy Clip。
    #[test]
    fn test_overflow_clip_maps_to_clip() {
        let mut style = ComputedStyle::default();
        style.overflow_y = OverflowValue::Clip;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.overflow.y, taffy::style::Overflow::Clip);
    }

    // ── grid-template-areas 校验测试 ──

    /// 测试非矩形列数不一致时仍返回结果（行数不匹配）。
    #[test]
    fn test_grid_template_areas_uneven_rows() {
        // 第二行只有 1 列，第一行有 2 列
        let areas = parse_grid_template_areas("\"a a\" \"b\"");
        // 仍然返回 areas，但会有 warn 日志
        assert!(!areas.is_empty());
        assert_eq!(areas.get("a"), Some(&(1, 2, 1, 3)));
        assert_eq!(areas.get("b"), Some(&(2, 3, 1, 2)));
    }

    /// 测试非矩形区域（L 形区域触发警告）。
    #[test]
    fn test_grid_template_areas_non_rectangular() {
        // "a" 出现在 (1,1) (1,2) (2,2) — 不构成矩形（缺少 (2,1)）
        let areas = parse_grid_template_areas("\"a a\" \"b a\"");
        // 仍然返回结果（兼容性），但会有 warn 日志
        assert!(!areas.is_empty());
        // a: row 1-3, col 1-3（基于 expand 逻辑）
        assert!(areas.contains_key("a"));
        assert!(areas.contains_key("b"));
    }

    /// 测试矩形区域不触发警告。
    #[test]
    fn test_grid_template_areas_rectangular_valid() {
        let areas = parse_grid_template_areas("\"a a\" \"a b\"");
        assert_eq!(areas.len(), 2);
        // a: row 1-3, col 1-3 — 出现在 (1,1) (1,2) (2,1) 构成 2x2 矩形
        assert_eq!(areas.get("a"), Some(&(1, 3, 1, 3)));
        // b: row 2-3, col 2-3（col_idx=1 → col=2, entry=(2,3,2,3)）
        assert_eq!(areas.get("b"), Some(&(2, 3, 2, 3)));
    }

    /// 测试 dot 占位符（CSS 规范中用 . 表示空单元格）。
    #[test]
    fn test_grid_template_areas_with_dot() {
        let areas = parse_grid_template_areas("\"header header\" \". sidebar\" \"footer footer\"");
        // "." 也被视为一个 token 名称，所以共有 4 个区域
        assert_eq!(areas.len(), 4);
        assert_eq!(areas.get("header"), Some(&(1, 2, 1, 3)));
        assert_eq!(areas.get("sidebar"), Some(&(2, 3, 2, 3)));
        assert_eq!(areas.get("footer"), Some(&(3, 4, 1, 3)));
        assert!(areas.contains_key("."));
    }

    /// 测试单行区域正确解析。
    #[test]
    fn test_grid_template_areas_single_row() {
        let areas = parse_grid_template_areas("\"a b c\"");
        assert_eq!(areas.len(), 3);
        assert_eq!(areas.get("a"), Some(&(1, 2, 1, 2)));
        assert_eq!(areas.get("b"), Some(&(1, 2, 2, 3)));
        assert_eq!(areas.get("c"), Some(&(1, 2, 3, 4)));
    }

    // -- 边界条件测试（第五批）--

    /// 测试 parse_grid_tracks 传入 Some("") 空字符串时返回空轨道列表。
    ///
    /// Some("") 与 None 不同：None 返回空列表，Some("") 也应返回空列表
    /// （tokenize 后没有有效 token）。
    #[test]
    fn test_parse_grid_tracks_empty_some_string() {
        let tracks = parse_grid_tracks(&Some("".to_string()));
        assert!(
            tracks.is_empty(),
            "Some(\"\") 应返回空轨道列表，实际 {} 个",
            tracks.len()
        );

        // 纯空白字符串同样应返回空列表
        let tracks_ws = parse_grid_tracks(&Some("   ".to_string()));
        assert!(
            tracks_ws.is_empty(),
            "纯空白字符串应返回空轨道列表，实际 {} 个",
            tracks_ws.len()
        );
    }

    /// 测试 parse_grid_tracks 解析百分比轨道值。
    ///
    /// "25% 50% 25%" 应解析为三个轨道，验证轨道数量和基本属性。
    #[test]
    fn test_parse_grid_tracks_percentage_values() {
        let tracks = parse_grid_tracks(&Some("25% 50% 25%".to_string()));
        assert_eq!(tracks.len(), 3, "应有 3 个轨道");

        // 验证每个轨道都是 Single 变体（不是 Repeat）
        for (i, track) in tracks.iter().enumerate() {
            assert!(
                matches!(track, taffy::style::TrackSizingFunction::Single(_)),
                "第 {} 个轨道应为 Single 变体",
                i
            );
        }

        // 将轨道转换为 taffy Style 并验证 gap 设置正确
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        style.grid_template_columns = Some("25% 50% 25%".to_string());
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(
            taffy_style.grid_template_columns.len(),
            3,
            "taffy Style 中应有 3 列轨道"
        );
    }

    /// 测试 resolve_grid_placement 在无 parent_areas 时将所有 Name 转为 Auto。
    ///
    /// 当子元素引用 grid-area 名称但父级容器没有定义 grid-template-areas 时，
    /// 所有命名引用应安全降级为 Auto，不会 panic。
    #[test]
    fn test_resolve_grid_placement_no_parent_areas() {
        use zero_style_system::GridLineValue;

        let mut style = ComputedStyle::default();
        style.grid_row_start = GridLineValue::Name("missing".to_string());
        style.grid_row_end = GridLineValue::Name("missing".to_string());
        style.grid_column_start = GridLineValue::Name("missing".to_string());
        style.grid_column_end = GridLineValue::Name("missing".to_string());

        // parent_areas = None
        let (rs, re, cs, ce) = resolve_grid_placement(&style, None);

        assert_eq!(rs, GridLineValue::Auto, "row-start 无 area map 时应为 Auto");
        assert_eq!(re, GridLineValue::Auto, "row-end 无 area map 时应为 Auto");
        assert_eq!(cs, GridLineValue::Auto, "col-start 无 area map 时应为 Auto");
        assert_eq!(ce, GridLineValue::Auto, "col-end 无 area map 时应为 Auto");
    }

    // ── 转换路径覆盖测试 ──

    #[test]
    /// 测试 width 使用 fit-content/min-content/max-content 值。
    fn test_convert_length_dimension_content_keywords() {
        let mut style = ComputedStyle::default();
        style.width = LengthValue::FitContent(Box::new(LengthValue::Px(200.0)));
        let taffy_style = computed_style_to_taffy(&style, None);
        // fit-content 在 convert_length_to_dimension 中映射为特定值，不 panic 即可
        let _ = taffy_style.size.width;

        style.width = LengthValue::MinContent;
        let taffy_style = computed_style_to_taffy(&style, None);
        let _ = taffy_style.size.width;

        style.width = LengthValue::MaxContent;
        let taffy_style = computed_style_to_taffy(&style, None);
        let _ = taffy_style.size.width;
    }

    #[test]
    /// 测试 max-width 使用 Em 单位。
    fn test_convert_max_length_dimension_units() {
        let mut style = ComputedStyle::default();
        style.max_width = LengthValue::Em(10.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::Length(10.0));
    }

    #[test]
    /// 测试 max-height 使用百分比。
    fn test_convert_max_height_percentage() {
        let mut style = ComputedStyle::default();
        style.max_height = LengthValue::Percentage(50.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.max_size.height, taffy::style::Dimension::Percent(0.5));
    }

    #[test]
    /// 测试 padding 使用 Em 和 Rem 单位。
    fn test_convert_padding_em_rem() {
        let mut style = ComputedStyle::default();
        style.padding_left = LengthValue::Em(2.0);
        style.padding_right = LengthValue::Rem(1.5);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.padding.left, taffy::style::LengthPercentage::Length(2.0));
        assert_eq!(taffy_style.padding.right, taffy::style::LengthPercentage::Length(1.5));
    }

    #[test]
    /// 测试 margin 使用 Vw/Vh 单位。
    fn test_convert_margin_viewport_units() {
        let mut style = ComputedStyle::default();
        style.margin_top = LengthValue::Vw(5.0);
        style.margin_bottom = LengthValue::Vh(2.5);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.margin.top, taffy::style::LengthPercentageAuto::Length(5.0));
        assert_eq!(
            taffy_style.margin.bottom,
            taffy::style::LengthPercentageAuto::Length(2.5)
        );
    }

    #[test]
    /// 测试 gap 使用 Vmin/Vmax 单位。
    fn test_convert_gap_viewport_units() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        style.row_gap = LengthValue::Vmin(2.0);
        style.column_gap = LengthValue::Vmax(1.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.gap.height, taffy::style::LengthPercentage::Length(2.0));
    }

    #[test]
    /// 测试 flex-direction: row-reverse 和 column-reverse。
    fn test_convert_flex_direction_reverse() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        style.flex_direction = FlexDirectionValue::RowReverse;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.flex_direction, taffy::style::FlexDirection::RowReverse);

        style.flex_direction = FlexDirectionValue::ColumnReverse;
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.flex_direction, taffy::style::FlexDirection::ColumnReverse);
    }

    #[test]
    /// 测试 flex-basis 使用 Em 长度值。
    fn test_convert_flex_basis_length_em() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Flex;
        style.flex_basis = FlexBasisValue::Length(LengthValue::Em(3.0));
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.flex_basis, taffy::style::Dimension::Length(3.0));
    }

    #[test]
    /// 测试 grid parse_single_track 对无效字符串回退到 Auto。
    fn test_parse_single_track_fallback_auto() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        style.grid_auto_rows = Some("invalid-value".to_string());
        let taffy_style = computed_style_to_taffy(&style, None);
        // grid_auto_rows 解析失败不应 panic
        let _ = taffy_style.grid_auto_rows;
    }

    #[test]
    /// 测试 grid track 解析纯数值 minmax。
    fn test_parse_minmax_numeric_fallback() {
        let mut style = ComputedStyle::default();
        style.display = DisplayValue::Grid;
        style.grid_template_rows = Some("minmax(100, 1fr)".to_string());
        let taffy_style = computed_style_to_taffy(&style, None);
        // 不应 panic
        let _ = taffy_style.grid_template_rows;
    }

    #[test]
    /// 测试 min-width/max-width 组合使用 Ch 单位。
    fn test_convert_min_max_width_ch_unit() {
        let mut style = ComputedStyle::default();
        style.min_width = LengthValue::Ch(4.0);
        style.max_width = LengthValue::Ch(40.0);
        let taffy_style = computed_style_to_taffy(&style, None);
        assert_eq!(taffy_style.min_size.width, taffy::style::Dimension::Length(4.0));
        assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::Length(40.0));
    }
}
