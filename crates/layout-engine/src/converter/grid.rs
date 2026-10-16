use taffy::prelude::*;
use zero_css_parser::values::LengthValue;
use zero_css_parser::{Token, Tokenizer};
use zero_style_system::{GridAutoFlowValue, GridLineValue};

#[cfg(test)]
pub(super) fn parse_grid_tracks(value: &Option<String>) -> Vec<taffy::style::GridTemplateComponent<String>> {
    parse_grid_tracks_with_context(value, 16.0, 0.0, 0.0)
}

#[cfg(test)]
pub(super) fn parse_grid_tracks_with_context(
    value: &Option<String>,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> Vec<taffy::style::GridTemplateComponent<String>> {
    parse_grid_template_with_context(value, font_size, vw, vh).0
}

/// 将轨道尺寸与同一显式网格线上定义的名称一并转换。
///
/// CSS Grid named lines are annotations at track boundaries. Preserve them in Taffy's
/// parallel line-name list so its named-line resolver can place items by name.
// https://drafts.csswg.org/css-grid-2/#typedef-line-names
pub(super) fn parse_grid_template_with_context(
    value: &Option<String>,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> (Vec<taffy::style::GridTemplateComponent<String>>, Vec<Vec<String>>) {
    let Some(value) = value else {
        return (vec![], vec![]);
    };

    let tokens = tokenize_track_list(value);
    let mut tracks = Vec::new();
    let mut line_names = vec![Vec::new()];

    for token in tokens {
        if is_grid_line_names_token(&token) {
            if let Some(names) = parse_grid_line_names(&token) {
                line_names.last_mut().expect("initial line name set").extend(names);
            }
            continue;
        }
        if let Some(inner) = grid_track_function_inner(&token, "repeat") {
            let (repeat_tracks, repeat_line_names) = parse_repeat_with_context(inner, font_size, vw, vh);
            let auto_repeat = repeat_tracks.len() == 1
                && matches!(
                    repeat_tracks.first(),
                    Some(taffy::style::GridTemplateComponent::Repeat(_))
                );
            tracks.extend(repeat_tracks);
            if auto_repeat {
                // The repetition owns its internal line names; this set is the line after it.
                line_names.push(Vec::new());
            } else if let Some((first, rest)) = repeat_line_names.split_first() {
                line_names
                    .last_mut()
                    .expect("initial line name set")
                    .extend(first.iter().cloned());
                line_names.extend(rest.iter().cloned());
            } else if !tracks.is_empty() {
                line_names.push(Vec::new());
            }
        } else {
            tracks.push(parse_single_track_with_context(&token, font_size, vw, vh));
            line_names.push(Vec::new());
        }
    }

    (tracks, line_names)
}

pub(super) fn parse_grid_line_names(token: &str) -> Option<Vec<String>> {
    let names = token.strip_prefix('[')?.strip_suffix(']')?;
    let mut parsed = Vec::new();
    for token in Tokenizer::new(names).collect_tokens() {
        match token {
            Token::Ident(name) => parsed.push(name.to_string()),
            Token::Whitespace | Token::Comment(_) | Token::Eof => {}
            _ => return None,
        }
    }
    (!parsed.is_empty()).then_some(parsed)
}

pub(super) fn parse_grid_line_name_sets(tokens: &[String]) -> Vec<Vec<String>> {
    let mut line_names = vec![Vec::new()];
    for token in tokens {
        if is_grid_line_names_token(token) {
            if let Some(names) = parse_grid_line_names(token) {
                line_names.last_mut().expect("initial line name set").extend(names);
            }
        } else {
            line_names.push(Vec::new());
        }
    }
    line_names
}

/// 将 grid track 列表字符串拆分为独立的 token。
///
/// 与 `split_whitespace` 不同，此函数会识别括号边界，
/// 将 `repeat(...)` 和 `minmax(...)` 保持为单个 token。
pub(super) fn tokenize_track_list(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth: u32 = 0;
    let mut line_names = false;

    for ch in value.chars() {
        match ch {
            '[' if !line_names => {
                line_names = true;
                current.push(ch);
            }
            '[' => current.push(ch),
            ']' if line_names => {
                line_names = false;
                current.push(ch);
            }
            ']' => current.push(ch),
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.saturating_sub(1);
                current.push(ch);
            }
            c if c.is_whitespace() && depth == 0 && !line_names => {
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

pub(super) fn is_grid_line_names_token(token: &str) -> bool {
    token.starts_with('[') && token.ends_with(']')
}

/// 解析 repeat() 函数内部内容为 track sizing function 列表。
///
/// 格式：`3, 100px` 或 `auto-fill, 200px` 或 `2, 1fr auto`。
///
/// 对于 auto-fill/auto-fit，生成 `TrackSizingFunction::Repeat` 变体，
/// 利用 taffy 原生的 auto-fill 支持，根据容器宽度自动计算轨道数量。
/// 对于固定次数，直接展开为对应数量的轨道。
#[cfg(test)]
pub(super) fn parse_repeat(inner: &str) -> Vec<taffy::style::GridTemplateComponent<String>> {
    parse_repeat_with_context(inner, 16.0, 0.0, 0.0).0
}

pub(super) fn parse_repeat_with_context(
    inner: &str,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> (Vec<taffy::style::GridTemplateComponent<String>>, Vec<Vec<String>>) {
    use taffy::style::{GridTemplateComponent, GridTemplateRepetition, RepetitionCount, TrackSizingFunction};

    // 找到第一个不在括号内的逗号
    let comma_pos = find_top_level_comma(inner);
    let Some(comma_pos) = comma_pos else {
        return (vec![TrackSizingFunction::AUTO.into()], vec![Vec::new(), Vec::new()]);
    };

    let count_str = inner[..comma_pos].trim();
    let track_list_str = inner[comma_pos + 1..].trim();

    // 解析内部 track 列表为 TrackSizingFunction（非重复型，0.9.2 = MinMax）
    let inner_tokens = tokenize_track_list(track_list_str);
    let inner_tracks: Vec<TrackSizingFunction> = inner_tokens
        .iter()
        .filter(|token| !is_grid_line_names_token(token))
        .map(|t| parse_single_track_as_non_repeated(t, font_size, vw, vh))
        .collect();
    let inner_line_names = parse_grid_line_name_sets(&inner_tokens);

    if count_str.eq_ignore_ascii_case("auto-fill") {
        // 传递给 taffy 原生 auto-fill，自动根据容器宽度计算轨道数量
        return (
            vec![GridTemplateComponent::Repeat(GridTemplateRepetition {
                count: RepetitionCount::AutoFill,
                tracks: inner_tracks,
                line_names: inner_line_names.clone(),
            })],
            inner_line_names,
        );
    }

    if count_str.eq_ignore_ascii_case("auto-fit") {
        // 传递给 taffy 原生 auto-fit
        return (
            vec![GridTemplateComponent::Repeat(GridTemplateRepetition {
                count: RepetitionCount::AutoFit,
                tracks: inner_tracks,
                line_names: inner_line_names.clone(),
            })],
            inner_line_names,
        );
    }

    // 固定次数：展开为对应数量的轨道
    let Ok(count) = count_str.parse::<usize>() else {
        return (vec![TrackSizingFunction::AUTO.into()], vec![Vec::new(), Vec::new()]);
    };

    let mut result = Vec::with_capacity(count * inner_tracks.len());
    let mut line_names = vec![Vec::new()];
    for _ in 0..count {
        result.extend(inner_tracks.iter().map(|t| GridTemplateComponent::Single(*t)));
        if let Some(first) = inner_line_names.first() {
            line_names
                .last_mut()
                .expect("initial line name set")
                .extend(first.iter().cloned());
        }
        line_names.extend(inner_line_names.iter().skip(1).cloned());
    }

    (result, line_names)
}

/// 将单个 track 值解析为 TrackSizingFunction。
///
/// 用于 repeat() 内部轨道列表的解析。
pub(super) fn parse_single_track_as_non_repeated(
    s: &str,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> taffy::style::TrackSizingFunction {
    use taffy::style::TrackSizingFunction;

    let s = s.trim();

    if s.eq_ignore_ascii_case("auto") {
        return TrackSizingFunction::AUTO;
    }
    // R5038：内容关键字轨道（同 parse_single_track——min-content ≡ minmax(min-content,
    // max-content)、max-content ≡ minmax(max-content, max-content)）。
    if s.eq_ignore_ascii_case("min-content") {
        return taffy::geometry::MinMax {
            min: taffy::style::MinTrackSizingFunction::MIN_CONTENT,
            max: taffy::style::MaxTrackSizingFunction::MAX_CONTENT,
        };
    }
    if s.eq_ignore_ascii_case("max-content") {
        return taffy::geometry::MinMax {
            min: taffy::style::MinTrackSizingFunction::MAX_CONTENT,
            max: taffy::style::MaxTrackSizingFunction::MAX_CONTENT,
        };
    }
    if let Some(flex) = parse_non_negative_number_with_suffix(s, "fr") {
        return TrackSizingFunction::from_fr(flex);
    }
    if let Some(pct) = parse_non_negative_number_with_suffix(s, "%") {
        return TrackSizingFunction::from_percent(pct / 100.0);
    }
    if let Some(inner) = grid_track_function_inner(s, "minmax") {
        return parse_minmax_as_non_repeated_with_context(inner, font_size, vw, vh);
    }
    // fit-content() 函数
    if let Some(inner) = grid_track_function_inner(s, "fit-content") {
        if let Some((val, is_pct)) = parse_length_percentage(inner.trim(), font_size, vw, vh) {
            return TrackSizingFunction {
                min: taffy::style::MinTrackSizingFunction::auto(),
                max: if is_pct {
                    taffy::style::MaxTrackSizingFunction::fit_content_percent(val)
                } else {
                    taffy::style::MaxTrackSizingFunction::fit_content_px(val)
                },
            };
        }
    }
    if let Some((value, is_percent)) = parse_length_percentage(s, font_size, vw, vh) {
        return if is_percent {
            TrackSizingFunction::from_percent(value)
        } else {
            TrackSizingFunction::from_length(value)
        };
    }

    TrackSizingFunction::AUTO
}

/// 找到字符串中第一个不在括号内的逗号位置。
pub(super) fn find_top_level_comma(s: &str) -> Option<usize> {
    let mut depth: u32 = 0;
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

/// 解析 grid-auto-rows/columns 的 track 定义为 TrackSizingFunction 列表。
///
/// 与 parse_grid_tracks 类似，但返回 TrackSizingFunction
/// （不包含 repeat 变体），用于 taffy 的 grid_auto_rows/grid_auto_columns 字段。
#[cfg(test)]
pub(super) fn parse_grid_auto_tracks(value: &Option<String>) -> Vec<taffy::style::TrackSizingFunction> {
    parse_grid_auto_tracks_with_context(value, 16.0, 0.0, 0.0)
}

pub(super) fn parse_grid_auto_tracks_with_context(
    value: &Option<String>,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> Vec<taffy::style::TrackSizingFunction> {
    let Some(value) = value else {
        return vec![];
    };

    value
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|token| parse_single_auto_track_with_context(token, font_size, vw, vh))
        .collect()
}

/// 解析单个 TrackSizingFunction 值。
#[cfg(test)]
pub(super) fn parse_single_auto_track(s: &str) -> taffy::style::TrackSizingFunction {
    parse_single_auto_track_with_context(s, 16.0, 0.0, 0.0)
}

pub(super) fn parse_single_auto_track_with_context(
    s: &str,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> taffy::style::TrackSizingFunction {
    use taffy::style::TrackSizingFunction;

    let s = s.trim();

    if s.eq_ignore_ascii_case("auto") {
        return TrackSizingFunction::AUTO;
    }
    if let Some(flex) = parse_non_negative_number_with_suffix(s, "fr") {
        return TrackSizingFunction::from_fr(flex);
    }
    if let Some(pct) = parse_non_negative_number_with_suffix(s, "%") {
        return TrackSizingFunction::from_percent(pct / 100.0);
    }
    if let Some(inner) = grid_track_function_inner(s, "minmax") {
        return parse_minmax_as_non_repeated_with_context(inner, font_size, vw, vh);
    }
    if let Some((value, is_percent)) = parse_length_percentage(s, font_size, vw, vh) {
        return if is_percent {
            TrackSizingFunction::from_percent(value)
        } else {
            TrackSizingFunction::from_length(value)
        };
    }

    TrackSizingFunction::AUTO
}

/// 解析 minmax() 函数内部，返回 TrackSizingFunction。
#[cfg(test)]
pub(super) fn parse_minmax_as_non_repeated(inner: &str) -> taffy::style::TrackSizingFunction {
    parse_minmax_as_non_repeated_with_context(inner, 16.0, 0.0, 0.0)
}

pub(super) fn parse_minmax_as_non_repeated_with_context(
    inner: &str,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> taffy::style::TrackSizingFunction {
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 2 {
        return taffy::style::TrackSizingFunction::AUTO;
    }

    let min = parse_min_track_with_context(parts[0].trim(), font_size, vw, vh);
    let max = parse_max_track_with_context(parts[1].trim(), font_size, vw, vh);

    taffy::geometry::MinMax { min, max }
}

/// 解析单个 grid track 值。
#[cfg(test)]
pub(super) fn parse_single_track(s: &str) -> taffy::style::GridTemplateComponent<String> {
    parse_single_track_with_context(s, 16.0, 0.0, 0.0)
}

pub(super) fn parse_single_track_with_context(
    s: &str,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> taffy::style::GridTemplateComponent<String> {
    use taffy::style::TrackSizingFunction;

    let s = s.trim();

    if s.eq_ignore_ascii_case("auto") {
        return TrackSizingFunction::AUTO.into();
    }
    // R5038（css-grid-2 §7.2.3）：内容关键字轨道——`min-content` ≡ minmax(min-content,
    // max-content)、`max-content` ≡ minmax(max-content, max-content)。
    if s.eq_ignore_ascii_case("min-content") {
        return taffy::geometry::MinMax {
            min: taffy::style::MinTrackSizingFunction::MIN_CONTENT,
            max: taffy::style::MaxTrackSizingFunction::MAX_CONTENT,
        }
        .into();
    }
    if s.eq_ignore_ascii_case("max-content") {
        return taffy::geometry::MinMax {
            min: taffy::style::MinTrackSizingFunction::MAX_CONTENT,
            max: taffy::style::MaxTrackSizingFunction::MAX_CONTENT,
        }
        .into();
    }
    if let Some(flex) = parse_non_negative_number_with_suffix(s, "fr") {
        return TrackSizingFunction::from_fr(flex).into();
    }
    if let Some(pct) = parse_non_negative_number_with_suffix(s, "%") {
        return TrackSizingFunction::from_percent(pct / 100.0).into();
    }
    if let Some(inner) = grid_track_function_inner(s, "minmax") {
        return parse_minmax_with_context(inner, font_size, vw, vh).into();
    }
    // fit-content() 函数：映射为 taffy 的 FitContent 轨道尺寸
    if let Some(inner) = grid_track_function_inner(s, "fit-content") {
        if let Some((val, is_pct)) = parse_length_percentage(inner.trim(), font_size, vw, vh) {
            return taffy::style::GridTemplateComponent::Single(taffy::geometry::MinMax {
                min: taffy::style::MinTrackSizingFunction::auto(),
                max: if is_pct {
                    taffy::style::MaxTrackSizingFunction::fit_content_percent(val)
                } else {
                    taffy::style::MaxTrackSizingFunction::fit_content_px(val)
                },
            });
        }
    }
    // 默认尝试解析为长度
    if let Some((value, is_percent)) = parse_length_percentage(s, font_size, vw, vh) {
        return if is_percent {
            TrackSizingFunction::from_percent(value).into()
        } else {
            TrackSizingFunction::from_length(value).into()
        };
    }

    // 无法解析，默认 auto
    TrackSizingFunction::AUTO.into()
}

/// 解析并解析 CSS Grid 轨道的 length-percentage，返回 (值, 是否百分比)。
///
/// em/rem 与 viewport 单位按现有样式系统计算值转换，再交给 Taffy。
/// https://drafts.csswg.org/css-grid-2/#typedef-track-breadth
/// https://drafts.csswg.org/css-values-4/#lengths
/// fit-content 轨道使用同一转换（Taffy 0.12.1 接受 px/percent 的 f32）。
pub(super) fn parse_length_percentage(s: &str, font_size: f32, vw: f32, vh: f32) -> Option<(f32, bool)> {
    let Some(value) = zero_css_parser::values::parse_length(s.trim()) else {
        return Some((parse_non_negative_number(s)?, false));
    };
    if let LengthValue::Percentage(percent) = value {
        return (percent.is_finite() && percent >= 0.0).then_some((percent as f32 / 100.0, true));
    }
    if !matches!(
        value,
        LengthValue::Px(_)
            | LengthValue::Em(_)
            | LengthValue::Rem(_)
            | LengthValue::Vh(_)
            | LengthValue::Vw(_)
            | LengthValue::Vmin(_)
            | LengthValue::Vmax(_)
    ) {
        return None;
    }
    let resolved = zero_style_system::resolve_length(&value, font_size as f64, Some(vw as f64), Some(vh as f64)) as f32;
    (resolved.is_finite() && resolved >= 0.0).then_some((resolved, false))
}

#[cfg(test)]
pub(super) fn parse_minmax(inner: &str) -> taffy::style::TrackSizingFunction {
    parse_minmax_with_context(inner, 16.0, 0.0, 0.0)
}

pub(super) fn parse_minmax_with_context(
    inner: &str,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> taffy::style::TrackSizingFunction {
    let parts: Vec<&str> = inner.split(',').collect();
    if parts.len() != 2 {
        return taffy::style::TrackSizingFunction::AUTO;
    }

    let min = parse_min_track_with_context(parts[0].trim(), font_size, vw, vh);
    let max = parse_max_track_with_context(parts[1].trim(), font_size, vw, vh);

    taffy::geometry::MinMax { min, max }
}

/// 解析 minmax 的最小值。
///
/// 支持 auto、内容关键字及 CSS length-percentage 值。
#[cfg(test)]
pub(super) fn parse_min_track(s: &str) -> taffy::style::MinTrackSizingFunction {
    parse_min_track_with_context(s, 16.0, 0.0, 0.0)
}

pub(super) fn parse_min_track_with_context(
    s: &str,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> taffy::style::MinTrackSizingFunction {
    use taffy::style::MinTrackSizingFunction;

    if s.eq_ignore_ascii_case("auto") {
        return MinTrackSizingFunction::auto();
    }
    // R5038（css-grid-2 §7.2 `<inflexible-breadth>`）：minmax() min 侧接受内容关键字。
    if s.eq_ignore_ascii_case("min-content") {
        return MinTrackSizingFunction::MIN_CONTENT;
    }
    if s.eq_ignore_ascii_case("max-content") {
        return MinTrackSizingFunction::MAX_CONTENT;
    }
    if let Some((value, is_percent)) = parse_length_percentage(s, font_size, vw, vh) {
        return if is_percent {
            MinTrackSizingFunction::percent(value)
        } else {
            MinTrackSizingFunction::length(value)
        };
    }

    MinTrackSizingFunction::auto()
}

/// 解析 minmax 的最大值。
///
/// 支持 auto、fr、内容关键字及 CSS length-percentage 值。
#[cfg(test)]
pub(super) fn parse_max_track(s: &str) -> taffy::style::MaxTrackSizingFunction {
    parse_max_track_with_context(s, 16.0, 0.0, 0.0)
}

pub(super) fn parse_max_track_with_context(
    s: &str,
    font_size: f32,
    vw: f32,
    vh: f32,
) -> taffy::style::MaxTrackSizingFunction {
    use taffy::style::MaxTrackSizingFunction;

    if s.eq_ignore_ascii_case("auto") {
        return MaxTrackSizingFunction::auto();
    }
    // R5038（css-grid-2 §7.2 `<track-breadth>`）：minmax() max 侧接受内容关键字。
    if s.eq_ignore_ascii_case("min-content") {
        return MaxTrackSizingFunction::MIN_CONTENT;
    }
    if s.eq_ignore_ascii_case("max-content") {
        return MaxTrackSizingFunction::MAX_CONTENT;
    }
    if let Some(flex) = parse_non_negative_number_with_suffix(s, "fr") {
        return MaxTrackSizingFunction::from_fr(flex);
    }
    if let Some((value, is_percent)) = parse_length_percentage(s, font_size, vw, vh) {
        return if is_percent {
            MaxTrackSizingFunction::percent(value)
        } else {
            MaxTrackSizingFunction::length(value)
        };
    }

    MaxTrackSizingFunction::auto()
}

pub(super) fn parse_non_negative_number_with_suffix(value: &str, suffix: &str) -> Option<f32> {
    // https://drafts.csswg.org/css-values-4/#numeric-types
    // CSS unit identifiers are ASCII case-insensitive.
    let number = strip_ascii_case_suffix(value.trim(), suffix)?;
    let parsed = number.parse::<f32>().ok()?;
    (parsed.is_finite() && parsed >= 0.0).then_some(parsed)
}

pub(super) fn parse_non_negative_number(value: &str) -> Option<f32> {
    let parsed = value.trim().parse::<f32>().ok()?;
    (parsed.is_finite() && parsed >= 0.0).then_some(parsed)
}

pub(super) fn strip_ascii_case_suffix<'a>(value: &'a str, suffix: &str) -> Option<&'a str> {
    let suffix_start = value.len().checked_sub(suffix.len())?;
    let tail = value.get(suffix_start..)?;
    if !tail.eq_ignore_ascii_case(suffix) {
        return None;
    }
    value.get(..suffix_start).map(str::trim)
}

pub(super) fn grid_track_function_inner<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    let value = value.trim();
    let prefix_len = name.len();
    let prefix = value.get(..prefix_len)?;
    if value.len() <= prefix_len + 1
        || !prefix.eq_ignore_ascii_case(name)
        || value.as_bytes().get(prefix_len) != Some(&b'(')
        || !value.ends_with(')')
    {
        return None;
    }
    Some(&value[prefix_len + 1..value.len() - 1])
}

/// 转换 grid-auto-flow 值。
pub(super) fn convert_grid_auto_flow(value: &GridAutoFlowValue) -> taffy::style::GridAutoFlow {
    match value {
        GridAutoFlowValue::Row => taffy::style::GridAutoFlow::Row,
        GridAutoFlowValue::Column => taffy::style::GridAutoFlow::Column,
        GridAutoFlowValue::RowDense => taffy::style::GridAutoFlow::RowDense,
        GridAutoFlowValue::ColumnDense => taffy::style::GridAutoFlow::ColumnDense,
    }
}

/// 转换 GridLineValue 到 taffy GridPlacement。
///
/// 显式命名线保留为 Taffy 的 NamedLine，供父级 grid-template line-name 列表解析。
// https://drafts.csswg.org/css-grid-2/#typedef-grid-line
pub(super) fn convert_grid_line(value: &GridLineValue) -> taffy::style::GridPlacement {
    match value {
        GridLineValue::Auto => taffy::style::GridPlacement::Auto,
        GridLineValue::Line(n) => taffy::style::GridPlacement::from_line_index(*n),
        GridLineValue::Span(s) => taffy::style::GridPlacement::from_span(*s),
        GridLineValue::Name(name) => taffy::style::GridPlacement::NamedLine(name.clone(), 0),
    }
}
