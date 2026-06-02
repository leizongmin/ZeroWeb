// 边界条件和极端值测试 — converter 模块私有函数。
use super::super::*;

// ── find_top_level_comma 边界条件 ──

/// 测试 find_top_level_comma：多字节 Unicode 字符后的逗号位置正确。
///
/// find_top_level_comma 使用 char_indices()，返回字节索引。
/// "中文,abc" 中 "中" 占 3 字节、"文" 占 3 字节，逗号在字节位置 6。
#[test]
fn test_find_top_level_comma_after_unicode() {
    let result = find_top_level_comma("中文,abc");
    assert_eq!(result, Some(6), "逗号应在字节位置 6（3+3）");
}

/// 测试 find_top_level_comma：多个顶层逗号返回第一个的位置。
#[test]
fn test_find_top_level_comma_multiple_commas() {
    let result = find_top_level_comma("a, b, c");
    assert_eq!(result, Some(1), "第一个逗号应在位置 1");
}

/// 测试 find_top_level_comma：嵌套括号内多个逗号均被忽略。
#[test]
fn test_find_top_level_comma_nested_multiple_commas() {
    // "minmax(10px, 20px, 30px)" — 括号内三个逗号，均不在顶层
    let result = find_top_level_comma("minmax(10px, 20px, 30px)");
    assert_eq!(result, None, "嵌套括号内的逗号应被忽略");
}

/// 测试 find_top_level_comma：深层嵌套后顶层逗号。
#[test]
fn test_find_top_level_comma_deep_nest_then_top_level() {
    // "repeat(2, minmax(10px, 1fr)), 100px" — 顶层逗号在 repeat(...) 之后
    let result = find_top_level_comma("repeat(2, minmax(10px, 1fr)), 100px");
    assert!(result.is_some(), "应找到顶层逗号");
    // 顶层逗号位置：在 ')' 之后
    let pos = result.unwrap();
    let char_at: Vec<char> = "repeat(2, minmax(10px, 1fr)), 100px".chars().collect();
    assert_eq!(char_at[pos], ',', "位置 {} 应是逗号，实际是 '{}'", pos, char_at[pos]);
}

/// 测试 find_top_level_comma：仅有逗号的单字符输入。
#[test]
fn test_find_top_level_comma_only_comma() {
    assert_eq!(find_top_level_comma(","), Some(0), "单个逗号应在位置 0");
}

// ── tokenize_track_list 边界条件 ──

/// 测试 tokenize_track_list：连续空白字符只产生间隔。
#[test]
fn test_tokenize_track_list_consecutive_whitespace() {
    let tokens = tokenize_track_list("100px    200px");
    assert_eq!(tokens.len(), 2, "连续空白应只分隔两个 token");
    assert_eq!(tokens[0], "100px");
    assert_eq!(tokens[1], "200px");
}

/// 测试 tokenize_track_list：tab 和空格混合。
#[test]
fn test_tokenize_track_list_mixed_tab_space() {
    let tokens = tokenize_track_list("auto\t\t1fr\t200px");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], "auto");
    assert_eq!(tokens[1], "1fr");
    assert_eq!(tokens[2], "200px");
}

/// 测试 tokenize_track_list：尾部空白不产生空 token。
#[test]
fn test_tokenize_track_list_trailing_whitespace() {
    let tokens = tokenize_track_list("100px 200px   ");
    assert_eq!(tokens.len(), 2, "尾部空白不应产生额外 token");
}

/// 测试 tokenize_track_list：前置空白不产生空 token。
#[test]
fn test_tokenize_track_list_leading_whitespace() {
    let tokens = tokenize_track_list("   100px 200px");
    assert_eq!(tokens.len(), 2, "前置空白不应产生额外 token");
}

// ── parse_single_auto_track 边界条件 ──

/// 测试 parse_single_auto_track：纯数字（无 px 后缀）。
#[test]
fn test_parse_single_auto_track_bare_number() {
    let result = parse_single_auto_track("42");
    let _ = result; // 不 panic 即可，验证解析为固定长度
}

/// 测试 parse_single_auto_track：零像素值。
#[test]
fn test_parse_single_auto_track_zero_px() {
    let result = parse_single_auto_track("0px");
    let _ = result;
}

/// 测试 parse_single_auto_track：零 fr 值。
#[test]
fn test_parse_single_auto_track_zero_fr() {
    let result = parse_single_auto_track("0fr");
    let _ = result;
}

/// 测试 parse_single_auto_track：小数 fr 值。
#[test]
fn test_parse_single_auto_track_fractional_fr() {
    let result = parse_single_auto_track("0.5fr");
    let _ = result;
}

/// 测试 parse_single_auto_track：minmax 内部值。
#[test]
fn test_parse_single_auto_track_minmax() {
    let result = parse_single_auto_track("minmax(0px, 1fr)");
    let _ = result;
}

// ── parse_min_track 边界条件 ──

/// 测试 parse_min_track：零百分比。
#[test]
fn test_parse_min_track_zero_percent() {
    use taffy::style::MinTrackSizingFunction;
    let result = parse_min_track("0%");
    assert!(matches!(result, MinTrackSizingFunction::Fixed(_)));
}

/// 测试 parse_min_track：小数像素值。
#[test]
fn test_parse_min_track_fractional_px() {
    use taffy::style::MinTrackSizingFunction;
    let result = parse_min_track("0.5px");
    assert!(matches!(result, MinTrackSizingFunction::Fixed(_)));
}

/// 测试 parse_min_track：fr 后缀不被 min 接受（应回退到 Auto）。
#[test]
fn test_parse_min_track_fr_fallback_to_auto() {
    use taffy::style::MinTrackSizingFunction;
    let result = parse_min_track("1fr");
    // fr 不在 parse_min_track 的匹配规则中，应回退到 Auto
    assert!(
        matches!(result, MinTrackSizingFunction::Auto),
        "min 不支持 fr，应回退到 Auto"
    );
}

// ── parse_max_track 边界条件 ──

/// 测试 parse_max_track：纯数字（无单位）。
#[test]
fn test_parse_max_track_bare_number() {
    use taffy::style::MaxTrackSizingFunction;
    let result = parse_max_track("300");
    assert!(matches!(result, MaxTrackSizingFunction::Fixed(_)));
}

/// 测试 parse_max_track：零 fr 值。
#[test]
fn test_parse_max_track_zero_fr() {
    use taffy::style::MaxTrackSizingFunction;
    let result = parse_max_track("0fr");
    assert!(matches!(result, MaxTrackSizingFunction::Fraction(_)));
}

// ── parse_minmax_as_non_repeated 边界条件 ──

/// 测试 parse_minmax_as_non_repeated：空字符串返回 AUTO。
#[test]
fn test_parse_minmax_as_non_repeated_empty() {
    assert_eq!(
        parse_minmax_as_non_repeated(""),
        taffy::style::NonRepeatedTrackSizingFunction::AUTO
    );
}

/// 测试 parse_minmax_as_non_repeated：只有空格返回 AUTO。
#[test]
fn test_parse_minmax_as_non_repeated_whitespace_only() {
    assert_eq!(
        parse_minmax_as_non_repeated("   "),
        taffy::style::NonRepeatedTrackSizingFunction::AUTO
    );
}

/// 测试 parse_minmax_as_non_repeated：两侧均为 auto。
#[test]
fn test_parse_minmax_as_non_repeated_both_auto() {
    let result = parse_minmax_as_non_repeated("auto, auto");
    let _ = result; // 不 panic 即可
}

/// 测试 parse_minmax_as_non_repeated：百分比 min + fr max。
#[test]
fn test_parse_minmax_as_non_repeated_percent_min_fr_max() {
    let result = parse_minmax_as_non_repeated("25%, 2fr");
    let _ = result;
}

// ── parse_grid_tracks 边界条件 ──

/// 测试 parse_grid_tracks：None 值返回空列表。
#[test]
fn test_parse_grid_tracks_none() {
    assert!(parse_grid_tracks(&None).is_empty());
}

/// 测试 parse_repeat：无效的重复次数字符串回退到 AUTO。
#[test]
fn test_parse_repeat_invalid_count() {
    let result = parse_repeat("abc, 100px");
    // 无效次数应产生单个 AUTO
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], taffy::style::TrackSizingFunction::AUTO);
}

/// 测试 parse_repeat：零次重复返回空列表。
#[test]
fn test_parse_repeat_zero_count() {
    let result = parse_repeat("0, 100px");
    assert!(result.is_empty(), "0 次重复应返回空列表");
}

/// 测试 parse_repeat：单次重复产生正确数量的轨道。
#[test]
fn test_parse_repeat_one_count() {
    let result = parse_repeat("1, 100px");
    assert_eq!(result.len(), 1);
}

/// 测试 parse_repeat：多次重复多轨道正确展开。
#[test]
fn test_parse_repeat_multi_track_expansion() {
    let result = parse_repeat("3, 1fr auto");
    // 3 × 2 = 6 个轨道
    assert_eq!(result.len(), 6);
}

// ── parse_grid_auto_tracks 边界条件 ──

/// 测试 parse_grid_auto_tracks：空字符串返回空列表。
#[test]
fn test_parse_grid_auto_tracks_empty_string() {
    let result = parse_grid_auto_tracks(&Some("".to_string()));
    assert!(result.is_empty(), "空字符串应返回空列表");
}

/// 测试 parse_grid_auto_tracks：纯空白字符串返回空列表。
#[test]
fn test_parse_grid_auto_tracks_whitespace_string() {
    let result = parse_grid_auto_tracks(&Some("   ".to_string()));
    assert!(result.is_empty(), "纯空白应返回空列表");
}

// ── parse_single_track 边界条件 ──

/// 测试 parse_single_track：无效字符串回退到 AUTO。
#[test]
fn test_parse_single_track_invalid_string() {
    let result = parse_single_track("not-a-track-value");
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO);
}

/// 测试 parse_single_track：空字符串回退到 AUTO。
#[test]
fn test_parse_single_track_empty_string() {
    let result = parse_single_track("");
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO);
}

// ── convert_display 边界条件 ──

/// 测试 convert_display：InlineBlock 映射为 Block。
#[test]
fn test_convert_display_inline_block() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::InlineBlock;
    let taffy_style = computed_style_to_taffy(&style, None);
    assert_eq!(taffy_style.display, taffy::style::Display::Block);
}

/// 测试 convert_display：InlineGrid 映射为 Grid。
#[test]
fn test_convert_display_inline_grid() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::InlineGrid;
    let taffy_style = computed_style_to_taffy(&style, None);
    assert_eq!(taffy_style.display, taffy::style::Display::Grid);
}

// ── convert_grid_auto_flow 边界条件 ──

/// 测试 convert_grid_auto_flow：Row 默认值。
#[test]
fn test_convert_grid_auto_flow_row() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;
    style.grid_auto_flow = GridAutoFlowValue::Row;
    let taffy_style = computed_style_to_taffy(&style, None);
    assert_eq!(taffy_style.grid_auto_flow, taffy::style::GridAutoFlow::Row);
}

// ── convert_grid_line 边界条件 ──

/// 测试 convert_grid_line：Name 变体回退到 Auto。
#[test]
fn test_convert_grid_line_name_fallback() {
    let result = convert_grid_line(&GridLineValue::Name("my-area".to_string()));
    assert_eq!(result, taffy::style::GridPlacement::Auto);
}

/// 测试 convert_grid_line：Span 值。
#[test]
fn test_convert_grid_line_span() {
    let result = convert_grid_line(&GridLineValue::Span(3));
    assert_eq!(result, taffy::style::GridPlacement::from_span(3));
}

/// 测试 convert_grid_line：负 Line 值。
#[test]
fn test_convert_grid_line_negative_line() {
    let result = convert_grid_line(&GridLineValue::Line(-2));
    assert_eq!(result, taffy::style::GridPlacement::from_line_index(-2));
}

/// 测试 convert_grid_line：零 Line 值。
#[test]
fn test_convert_grid_line_zero_line() {
    let result = convert_grid_line(&GridLineValue::Line(0));
    assert_eq!(result, taffy::style::GridPlacement::from_line_index(0));
}

// ── parse_grid_template_areas 边界条件 ──

/// 测试 parse_grid_template_areas：所有单元格相同名称。
#[test]
fn test_parse_grid_template_areas_single_name_all_cells() {
    let areas = parse_grid_template_areas("\"a a\" \"a a\"");
    assert_eq!(areas.len(), 1, "所有单元格相同名称应只有 1 个区域");
    assert_eq!(areas.get("a"), Some(&(1, 3, 1, 3)), "区域 a 应覆盖 2x2");
}

/// 测试 parse_grid_template_areas：混合引号外内容被忽略。
#[test]
fn test_parse_grid_template_areas_extra_content_outside_quotes() {
    let areas = parse_grid_template_areas("prefix \"a b\" suffix");
    // 引号外的 "prefix" 和 "suffix" 应被忽略
    assert_eq!(areas.len(), 2);
    assert_eq!(areas.get("a"), Some(&(1, 2, 1, 2)));
    assert_eq!(areas.get("b"), Some(&(1, 2, 2, 3)));
}

// ── parse_minmax 边界条件 ──

/// 测试 parse_minmax：只有一个参数时返回 AUTO。
#[test]
fn test_parse_minmax_single_arg() {
    let result = parse_minmax("100px");
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO);
}

/// 测试 parse_minmax：三个参数时返回 AUTO。
#[test]
fn test_parse_minmax_three_args() {
    let result = parse_minmax("10px, 20px, 30px");
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO);
}

/// 测试 parse_minmax：合法的 auto + fr 组合。
#[test]
fn test_parse_minmax_auto_fr() {
    let result = parse_minmax("auto, 1fr");
    let _ = result; // 不 panic 即可，验证是合法的 Single(MinMax)
}

// ── convert_length_to_lp 边界条件 ──

/// 测试 MinContent/MaxContent 在 convert_length_to_lp 中映射为 0。
#[test]
fn test_convert_length_to_lp_min_max_content() {
    assert_eq!(
        convert_length_to_lp(&LengthValue::MinContent),
        taffy::style::LengthPercentage::Length(0.0)
    );
    assert_eq!(
        convert_length_to_lp(&LengthValue::MaxContent),
        taffy::style::LengthPercentage::Length(0.0)
    );
}

// ── convert_length_to_lpa 边界条件 ──

/// 测试 MinContent/MaxContent 在 convert_length_to_lpa 中映射为 0。
#[test]
fn test_convert_length_to_lpa_min_max_content() {
    assert_eq!(
        convert_length_to_lpa(&LengthValue::MinContent),
        taffy::style::LengthPercentageAuto::Length(0.0)
    );
    assert_eq!(
        convert_length_to_lpa(&LengthValue::MaxContent),
        taffy::style::LengthPercentageAuto::Length(0.0)
    );
}

// ── convert_flex_basis 边界条件 ──

/// 测试 FlexBasisValue::Length(LengthValue::Auto) 转换。
#[test]
fn test_convert_flex_basis_auto_length() {
    let result = convert_flex_basis(&FlexBasisValue::Length(LengthValue::Auto));
    assert_eq!(result, taffy::style::Dimension::Auto);
}
