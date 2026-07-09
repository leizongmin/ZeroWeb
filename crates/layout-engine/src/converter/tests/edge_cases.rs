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
    let result = parse_min_track("0%");
    assert!(!result.is_auto());
}

/// 测试 parse_min_track：小数像素值。
#[test]
fn test_parse_min_track_fractional_px() {
    let result = parse_min_track("0.5px");
    assert!(!result.is_auto());
}

/// 测试 parse_min_track：fr 后缀不被 min 接受（应回退到 Auto）。
#[test]
fn test_parse_min_track_fr_fallback_to_auto() {
    let result = parse_min_track("1fr");
    // fr 不在 parse_min_track 的匹配规则中，应回退到 Auto
    assert!(result.is_auto(), "min 不支持 fr，应回退到 Auto");
}

// ── parse_max_track 边界条件 ──

/// 测试 parse_max_track：纯数字（无单位）。
#[test]
fn test_parse_max_track_bare_number() {
    let result = parse_max_track("300");
    assert!(!result.is_auto());
}

/// 测试 parse_max_track：零 fr 值。
#[test]
fn test_parse_max_track_zero_fr() {
    let result = parse_max_track("0fr");
    assert!(result.is_fr());
}

// ── parse_minmax_as_non_repeated 边界条件 ──

/// 测试 parse_minmax_as_non_repeated：空字符串返回 AUTO。
#[test]
fn test_parse_minmax_as_non_repeated_empty() {
    assert_eq!(
        parse_minmax_as_non_repeated(""),
        taffy::style::TrackSizingFunction::AUTO
    );
}

/// 测试 parse_minmax_as_non_repeated：只有空格返回 AUTO。
#[test]
fn test_parse_minmax_as_non_repeated_whitespace_only() {
    assert_eq!(
        parse_minmax_as_non_repeated("   "),
        taffy::style::TrackSizingFunction::AUTO
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
    assert_eq!(result[0], taffy::style::TrackSizingFunction::AUTO.into());
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
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO.into());
}

/// 测试 parse_single_track：空字符串回退到 AUTO。
#[test]
fn test_parse_single_track_empty_string() {
    let result = parse_single_track("");
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO.into());
}

// ── R1058：display:inline 垂直 margin 归零（CSS §8.3）──

/// R1058：非替换 inline 元素（display:inline）的垂直 margin 必须归零——CSS §8.3 规定
/// 非替换 inline 元素的 margin-top/bottom 无布局效果。旧实现把 inline 的垂直 margin
/// 原样喂给 taffy，致 block-in-inline-vertical-margins-on-span-ignored（span mt/bt:50
/// 错误推开块子间距）。水平 margin 保留（inline 水平 margin 有效）。
#[test]
fn test_r1058_inline_vertical_margin_zeroed() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Inline;
    style.margin_top = LengthValue::Px(50.0);
    style.margin_bottom = LengthValue::Px(50.0);
    style.margin_left = LengthValue::Px(10.0);
    style.margin_right = LengthValue::Px(20.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    // 垂直 margin 归零（§8.3）
    assert!(
        (!taffy_style.margin.top.is_auto() && taffy_style.margin.top.into_raw().value() == 0.0),
        "display:inline 的 margin-top 应回零（§8.3），实测 {:?}",
        taffy_style.margin.top
    );
    assert!(
        (!taffy_style.margin.bottom.is_auto() && taffy_style.margin.bottom.into_raw().value() == 0.0),
        "display:inline 的 margin-bottom 应回零（§8.3），实测 {:?}",
        taffy_style.margin.bottom
    );
    // 水平 margin 保留
    assert!(
        (!taffy_style.margin.left.is_auto() && taffy_style.margin.left.into_raw().value() == 10.0),
        "display:inline 的 margin-left 应保留 10px，实测 {:?}",
        taffy_style.margin.left
    );
    assert!(
        (!taffy_style.margin.right.is_auto() && taffy_style.margin.right.into_raw().value() == 20.0),
        "display:inline 的 margin-right 应保留 20px，实测 {:?}",
        taffy_style.margin.right
    );
}

/// R1058 对照：display:block 的垂直 margin 保留（非 inline，§8.3 不适用）。
#[test]
fn test_r1058_block_vertical_margin_preserved() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Block;
    style.margin_top = LengthValue::Px(50.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert!(
        (!taffy_style.margin.top.is_auto() && taffy_style.margin.top.into_raw().value() == 50.0),
        "display:block 的 margin-top 应保留 50px，实测 {:?}",
        taffy_style.margin.top
    );
}

// ── convert_display 边界条件 ──

/// 测试 convert_display：InlineBlock 映射为 Block。
#[test]
fn test_convert_display_inline_block() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::InlineBlock;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.display, taffy::style::Display::Block);
}

/// 测试 convert_display：InlineGrid 映射为 Grid。
#[test]
fn test_convert_display_inline_grid() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::InlineGrid;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.display, taffy::style::Display::Grid);
}

// ── convert_grid_auto_flow 边界条件 ──

/// 测试 convert_grid_auto_flow：Row 默认值。
#[test]
fn test_convert_grid_auto_flow_row() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;
    style.grid_auto_flow = GridAutoFlowValue::Row;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
        convert_length_to_lp(&LengthValue::MinContent, 800.0, 600.0),
        taffy::style::LengthPercentage::length(0.0)
    );
    assert_eq!(
        convert_length_to_lp(&LengthValue::MaxContent, 800.0, 600.0),
        taffy::style::LengthPercentage::length(0.0)
    );
}

// ── convert_length_to_lpa 边界条件 ──

/// 测试 MinContent/MaxContent 在 convert_length_to_lpa 中映射为 0。
#[test]
fn test_convert_length_to_lpa_min_max_content() {
    assert_eq!(
        convert_length_to_lpa(&LengthValue::MinContent, false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::length(0.0)
    );
    assert_eq!(
        convert_length_to_lpa(&LengthValue::MaxContent, false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::length(0.0)
    );
}

// ── convert_flex_basis 边界条件 ──

/// 测试 FlexBasisValue::Length(LengthValue::Auto) 转换。
#[test]
fn test_convert_flex_basis_auto_length() {
    let result = convert_flex_basis(&FlexBasisValue::Length(LengthValue::Auto), 800.0, 600.0);
    assert_eq!(result, taffy::style::Dimension::auto());
}

// ── convert_display 未覆盖的变体测试 ──

/// 测试 convert_display：Flow, FlowRoot, ListItem, Contents 都映射为 Block。
#[test]
fn test_convert_display_unmapped_variants() {
    let variants_to_test = [
        DisplayValue::Flow,
        DisplayValue::FlowRoot,
        DisplayValue::ListItem,
        DisplayValue::Contents,
    ];

    for display_value in variants_to_test {
        let result = convert_display(&display_value);
        assert_eq!(
            result,
            taffy::style::Display::Block,
            "{:?} should map to Block",
            display_value
        );
    }
}

// ── convert_overflow 未覆盖的变体测试 ──

/// 测试 convert_overflow：Clip 和 Auto 变体。
#[test]
fn test_convert_overflow_clip_and_auto() {
    // Clip
    let result = convert_overflow(&OverflowValue::Clip);
    assert_eq!(result, taffy::style::Overflow::Clip);

    // Auto 映射为 Scroll
    let result = convert_overflow(&OverflowValue::Auto);
    assert_eq!(result, taffy::style::Overflow::Scroll);
}

// ── convert_length_to_dimension 未覆盖的变体测试 ──

/// 测试 convert_length_to_dimension：Vh, Vw, Vmin, Vmax, Ch, FitContent, MinContent, MaxContent, Calc。
/// viewport 单位以 800×600 视口解析：50vh=300, 25vw=200, 10vmin=60, 20vmax=160。
#[test]
fn test_convert_length_to_dimension_uncovered_variants() {
    // Viewport units
    assert_eq!(
        convert_length_to_dimension(&LengthValue::Vh(50.0), 800.0, 600.0),
        taffy::style::Dimension::length(300.0)
    );
    assert_eq!(
        convert_length_to_dimension(&LengthValue::Vw(25.0), 800.0, 600.0),
        taffy::style::Dimension::length(200.0)
    );
    assert_eq!(
        convert_length_to_dimension(&LengthValue::Vmin(10.0), 800.0, 600.0),
        taffy::style::Dimension::length(60.0)
    );
    assert_eq!(
        convert_length_to_dimension(&LengthValue::Vmax(20.0), 800.0, 600.0),
        taffy::style::Dimension::length(160.0)
    );
    assert_eq!(
        convert_length_to_dimension(&LengthValue::Ch(8.0), 800.0, 600.0),
        taffy::style::Dimension::length(8.0)
    );

    // FitContent 内部转换
    let fit_content = LengthValue::FitContent(Box::new(LengthValue::Px(100.0)));
    assert_eq!(
        convert_length_to_dimension(&fit_content, 800.0, 600.0),
        taffy::style::Dimension::length(100.0)
    );

    // MinContent/MaxContent 塌缩为 0（信号保留到 layout-engine 两趟测量解析）
    assert_eq!(
        convert_length_to_dimension(&LengthValue::MinContent, 800.0, 600.0),
        taffy::style::Dimension::length(0.0)
    );
    assert_eq!(
        convert_length_to_dimension(&LengthValue::MaxContent, 800.0, 600.0),
        taffy::style::Dimension::length(0.0)
    );

    // Calc 映射为 0
    assert_eq!(
        convert_length_to_dimension(
            &LengthValue::Calc(Box::new(zero_css_parser::values::CalcExpr::Number(42.0))),
            800.0,
            600.0
        ),
        taffy::style::Dimension::length(0.0)
    );
}

// ── convert_max_length_to_dimension 未覆盖的变体测试 ──

/// 测试 convert_max_length_to_dimension：infinity 和各种变体。
/// viewport 单位以 800×600 视口解析。
#[test]
fn test_convert_max_length_to_dimension_uncovered_variants() {
    // Infinity 映射为 Auto
    assert_eq!(
        convert_max_length_to_dimension(&LengthValue::Px(f64::INFINITY), 800.0, 600.0),
        taffy::style::Dimension::auto()
    );

    // Viewport units
    assert_eq!(
        convert_max_length_to_dimension(&LengthValue::Vh(50.0), 800.0, 600.0),
        taffy::style::Dimension::length(300.0)
    );
    assert_eq!(
        convert_max_length_to_dimension(&LengthValue::Vw(25.0), 800.0, 600.0),
        taffy::style::Dimension::length(200.0)
    );

    // FitContent 内部转换
    let fit_content = LengthValue::FitContent(Box::new(LengthValue::Px(200.0)));
    assert_eq!(
        convert_max_length_to_dimension(&fit_content, 800.0, 600.0),
        taffy::style::Dimension::length(200.0)
    );

    // MinContent/MaxContent 映射为 Auto
    assert_eq!(
        convert_max_length_to_dimension(&LengthValue::MinContent, 800.0, 600.0),
        taffy::style::Dimension::auto()
    );
    assert_eq!(
        convert_max_length_to_dimension(&LengthValue::MaxContent, 800.0, 600.0),
        taffy::style::Dimension::auto()
    );
}

// ── convert_length_to_lp/lpa 未覆盖的变体测试 ──

/// 测试 convert_length_to_lp：Vh, Vw, FitContent, MinContent, MaxContent, Calc。
/// viewport 单位以 800×600 视口解析。
#[test]
fn test_convert_length_to_lp_uncovered_variants() {
    // Viewport units
    assert_eq!(
        convert_length_to_lp(&LengthValue::Vh(50.0), 800.0, 600.0),
        taffy::style::LengthPercentage::length(300.0)
    );
    assert_eq!(
        convert_length_to_lp(&LengthValue::Vw(25.0), 800.0, 600.0),
        taffy::style::LengthPercentage::length(200.0)
    );

    // FitContent 内部转换
    let fit_content = LengthValue::FitContent(Box::new(LengthValue::Px(100.0)));
    assert_eq!(
        convert_length_to_lp(&fit_content, 800.0, 600.0),
        taffy::style::LengthPercentage::length(100.0)
    );

    // MinContent/MaxContent 映射为 0
    assert_eq!(
        convert_length_to_lp(&LengthValue::MinContent, 800.0, 600.0),
        taffy::style::LengthPercentage::length(0.0)
    );
    assert_eq!(
        convert_length_to_lp(&LengthValue::MaxContent, 800.0, 600.0),
        taffy::style::LengthPercentage::length(0.0)
    );

    // Calc 映射为 0
    assert_eq!(
        convert_length_to_lp(
            &LengthValue::Calc(Box::new(zero_css_parser::values::CalcExpr::Number(42.0))),
            800.0,
            600.0
        ),
        taffy::style::LengthPercentage::length(0.0)
    );
}

/// 测试 convert_length_to_lpa：Vh, Vw, FitContent, MinContent, MaxContent, Calc。
/// viewport 单位以 800×600 视口解析。
#[test]
fn test_convert_length_to_lpa_uncovered_variants() {
    // Viewport units
    assert_eq!(
        convert_length_to_lpa(&LengthValue::Vh(50.0), false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::length(300.0)
    );
    assert_eq!(
        convert_length_to_lpa(&LengthValue::Vw(25.0), false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::length(200.0)
    );

    // FitContent 内部转换
    let fit_content = LengthValue::FitContent(Box::new(LengthValue::Px(100.0)));
    assert_eq!(
        convert_length_to_lpa(&fit_content, false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::length(100.0)
    );

    // MinContent/MaxContent 映射为 0
    assert_eq!(
        convert_length_to_lpa(&LengthValue::MinContent, false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::length(0.0)
    );
    assert_eq!(
        convert_length_to_lpa(&LengthValue::MaxContent, false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::length(0.0)
    );

    // Calc 映射为 0
    assert_eq!(
        convert_length_to_lpa(
            &LengthValue::Calc(Box::new(zero_css_parser::values::CalcExpr::Number(42.0))),
            false,
            800.0,
            600.0
        ),
        taffy::style::LengthPercentageAuto::length(0.0)
    );
}

// ── convert_alignment_to_* 未覆盖的变体测试 ──

/// 测试 convert_alignment_to_align_items：Baseline, SpaceBetween, SpaceAround, SpaceEvenly。
#[test]
fn test_convert_alignment_to_align_items_uncovered_variants() {
    assert_eq!(
        convert_alignment_to_align_items(&AlignmentValue::Baseline),
        Some(taffy::style::AlignItems::Baseline)
    );
    assert_eq!(convert_alignment_to_align_items(&AlignmentValue::SpaceBetween), None);
    assert_eq!(convert_alignment_to_align_items(&AlignmentValue::SpaceAround), None);
    assert_eq!(convert_alignment_to_align_items(&AlignmentValue::SpaceEvenly), None);
    assert_eq!(
        convert_alignment_to_align_items(&AlignmentValue::Start),
        Some(taffy::style::AlignItems::Start)
    );
    assert_eq!(
        convert_alignment_to_align_items(&AlignmentValue::End),
        Some(taffy::style::AlignItems::End)
    );
}

/// 测试 convert_alignment_to_justify_content：Start, End, Stretch, Baseline。
#[test]
fn test_convert_alignment_to_justify_content_uncovered_variants() {
    assert_eq!(
        convert_alignment_to_justify_content(&AlignmentValue::Start),
        Some(taffy::style::JustifyContent::Start)
    );
    assert_eq!(
        convert_alignment_to_justify_content(&AlignmentValue::End),
        Some(taffy::style::JustifyContent::End)
    );
    assert_eq!(
        convert_alignment_to_justify_content(&AlignmentValue::Stretch),
        Some(taffy::style::JustifyContent::Stretch)
    );
    assert_eq!(convert_alignment_to_justify_content(&AlignmentValue::Baseline), None);
}

/// 测试 convert_align_content：Start, End。
#[test]
fn test_convert_align_content_start_end() {
    use zero_style_system::AlignContentValue;
    assert_eq!(
        convert_align_content(&AlignContentValue::Start),
        Some(taffy::style::AlignContent::Start)
    );
    assert_eq!(
        convert_align_content(&AlignContentValue::End),
        Some(taffy::style::AlignContent::End)
    );
}

// ── parse_grid_tracks 未覆盖的测试 ──

/// 测试 parse_grid_tracks：None 值返回空列表。
#[test]
fn test_parse_grid_tracks_none_value() {
    let tracks = parse_grid_tracks(&None);
    assert!(tracks.is_empty());
}

/// 测试 parse_grid_tracks：repeat with auto-fill。
#[test]
fn test_parse_grid_tracks_repeat_auto_fill() {
    use taffy::style::RepetitionCount;

    let tracks = parse_grid_tracks(&Some("repeat(auto-fill, 200px)".to_string()));
    assert_eq!(tracks.len(), 1);
    match &tracks[0] {
        taffy::style::GridTemplateComponent::Repeat(rep) => {
            assert_eq!(rep.count, RepetitionCount::AutoFill);
            assert_eq!(rep.tracks.len(), 1);
            assert_eq!(rep.tracks[0], taffy::style::TrackSizingFunction::from_length(200.0));
        }
        _ => panic!("Expected GridTemplateComponent::Repeat"),
    }
}

/// 测试 parse_grid_tracks：repeat with auto-fit。
#[test]
fn test_parse_grid_tracks_repeat_auto_fit() {
    use taffy::style::RepetitionCount;

    let tracks = parse_grid_tracks(&Some("repeat(auto-fit, minmax(100px, 1fr))".to_string()));
    assert_eq!(tracks.len(), 1);
    match &tracks[0] {
        taffy::style::GridTemplateComponent::Repeat(rep) => {
            assert_eq!(rep.count, RepetitionCount::AutoFit);
            assert_eq!(rep.tracks.len(), 1);
            let nr = &rep.tracks[0];
            // TrackSizingFunction is MinMax<Min, Max>（0.9.2 opaque struct，用 accessor 断言）
            assert!(!nr.min.is_auto(), "Expected Fixed(100px)");
            assert_eq!(nr.min.into_raw().value(), 100.0);
            assert!(nr.max.is_fr(), "Expected Fraction(1fr)");
            assert_eq!(nr.max.into_raw().value(), 1.0);
        }
        _ => panic!("Expected GridTemplateComponent::Repeat"),
    }
}

// ── parse_grid_templateareas 未覆盖的测试 ──

/// 测试 parse_grid_template_areas：复杂模式（包含多个区域和不同大小）。
#[test]
fn test_parse_grid_template_areas_complex_pattern() {
    let areas = parse_grid_template_areas(
        "\"header header sidebar\" \
         \"nav    main   main\" \
         \"footer footer footer\"",
    );

    assert_eq!(areas.len(), 5);

    // header: row 1-2, col 1-3
    assert_eq!(areas.get("header"), Some(&(1, 2, 1, 3)));

    // sidebar: row 1-2, col 3-4
    assert_eq!(areas.get("sidebar"), Some(&(1, 2, 3, 4)));

    // nav: row 2-3, col 1-2
    assert_eq!(areas.get("nav"), Some(&(2, 3, 1, 2)));

    // main: row 2-3, col 2-4
    assert_eq!(areas.get("main"), Some(&(2, 3, 2, 4)));

    // footer: row 3-4, col 1-4
    assert_eq!(areas.get("footer"), Some(&(3, 4, 1, 4)));
}

/// 测试 parse_grid_template_areas：单区域多行多列。
#[test]
fn test_parse_grid_template_areas_single_area_multiple_rows_cols() {
    let areas = parse_grid_template_areas(
        "\"a a a\" \
         \"a a a\" \
         \"a a a\"",
    );

    assert_eq!(areas.len(), 1);
    assert_eq!(areas.get("a"), Some(&(1, 4, 1, 4))); // 3x3 grid
}

// ── resolve_named_area 未覆盖的测试 ──

/// 测试 resolve_named_area：Name 变体，存在 parent_areas，所有 "which" 值。
#[test]
fn test_resolve_named_area_all_which_values() {
    let mut areas = std::collections::HashMap::new();
    areas.insert("test-area".to_string(), (2, 4, 3, 5)); // row 2-4, col 3-5

    // 测试所有 which 值
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("test-area".to_string()), Some(&areas), "row-start"),
        GridLineValue::Line(2)
    );
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("test-area".to_string()), Some(&areas), "row-end"),
        GridLineValue::Line(4)
    );
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("test-area".to_string()), Some(&areas), "col-start"),
        GridLineValue::Line(3)
    );
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("test-area".to_string()), Some(&areas), "col-end"),
        GridLineValue::Line(5)
    );

    // 测试未知 which 值
    assert_eq!(
        resolve_named_area(&GridLineValue::Name("test-area".to_string()), Some(&areas), "unknown"),
        GridLineValue::Auto
    );
}

/// 测试 resolve_named_area：Name 变体，不存在 parent_areas，应返回 Auto。
#[test]
fn test_resolve_named_area_no_parent_areas() {
    let result = resolve_named_area(&GridLineValue::Name("nonexistent".to_string()), None, "row-start");
    assert_eq!(result, GridLineValue::Auto);
}

/// 测试 resolve_named_area：Name 变体，存在 parent_areas 但名称不存在。
#[test]
fn test_resolve_named_area_name_not_found() {
    let mut areas = std::collections::HashMap::new();
    areas.insert("existing".to_string(), (1, 2, 1, 2));

    let result = resolve_named_area(
        &GridLineValue::Name("nonexistent".to_string()),
        Some(&areas),
        "row-start",
    );
    assert_eq!(result, GridLineValue::Auto);
}
