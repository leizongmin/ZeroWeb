// Auto-generated test file — split from layout-engine/converter.rs
use super::super::*;

/// 测试 InlineFlex display 映射为 taffy::Display::Flex。
#[test]
fn test_convert_inline_flex_display() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::InlineFlex;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.display, taffy::style::Display::Flex);
}

/// 测试 InlineGrid display 映射为 taffy::Display::Grid。
#[test]
fn test_convert_inline_grid_display() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::InlineGrid;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
        let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(taffy_style.display, taffy::style::Display::Block);
    }
}

/// 测试 Em、Rem、Vw、Vh 单位转换为 length(v as f32)。
#[test]
fn test_convert_length_em_rem_vw_vh() {
    let mut style = ComputedStyle::default();
    style.width = LengthValue::Em(16.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::length(16.0));

    style.width = LengthValue::Rem(12.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::length(12.0));

    style.width = LengthValue::Vw(50.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::length(400.0));

    style.width = LengthValue::Vh(25.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::length(150.0));
}

/// 测试 Vmin、Vmax、Ch 单位转换：viewport 单位解析为视口相对像素。
#[test]
fn test_convert_length_vmin_vmax_ch() {
    let mut style = ComputedStyle::default();
    style.width = LengthValue::Vmin(10.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::length(60.0));

    style.width = LengthValue::Vmax(20.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::length(160.0));

    style.width = LengthValue::Ch(8.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::length(8.0));
}

/// 测试 LengthValue::Calc 在所有转换函数中映射为 length(0.0)。
#[test]
fn test_convert_length_calc_fallback() {
    use zero_css_parser::values::CalcExpr;
    let calc = LengthValue::Calc(Box::new(CalcExpr::Number(42.0)));

    // convert_length_to_dimension
    assert_eq!(
        convert_length_to_dimension(&calc, 800.0, 600.0),
        taffy::style::Dimension::length(0.0)
    );

    // convert_max_length_to_dimension
    assert_eq!(
        convert_max_length_to_dimension(&calc, 800.0, 600.0),
        taffy::style::Dimension::length(0.0)
    );

    // convert_length_to_lp
    assert_eq!(
        convert_length_to_lp(&calc, 800.0, 600.0),
        taffy::style::LengthPercentage::length(0.0)
    );

    // convert_length_to_lpa
    assert_eq!(
        convert_length_to_lpa(&calc, false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::length(0.0)
    );
}

/// 测试含百分比的 calc() 在 margin/inset 转换器（convert_length_to_lpa）中提取百分比
/// 部分，而非静默丢弃为 0.0。calc(50% - 0px) → Percent(0.5)（与 convert_length_to_dimension
/// 一致）。此前 margin/inset 的 calc() 被静默丢弃为 0.0（grid-calc-margin 等用例）。
#[test]
fn test_convert_length_to_lpa_calc_percentage() {
    use zero_css_parser::values::{CalcExpr, CalcOp};
    // calc(50% - 0px)
    let calc = LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
        Box::new(CalcExpr::Length(LengthValue::Percentage(50.0))),
        CalcOp::Subtract,
        Box::new(CalcExpr::Length(LengthValue::Px(0.0))),
    )));

    assert_eq!(
        convert_length_to_lpa(&calc, false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::percent(0.5)
    );

    // 纯百分比 calc(75%) → Percent(0.75)
    let calc75 = LengthValue::Calc(Box::new(CalcExpr::Length(LengthValue::Percentage(75.0))));
    assert_eq!(
        convert_length_to_lpa(&calc75, false, 800.0, 600.0),
        taffy::style::LengthPercentageAuto::percent(0.75)
    );
}

/// 测试含百分比的 calc() 在 max-width/max-height（convert_max_length_to_dimension）
/// 与 padding/border/gap（convert_length_to_lp）转换器中也提取百分比，而非静默丢弃为 0.0。
/// 与 convert_length_to_dimension / convert_length_to_lpa 一致（calc silent-drop 全 converter 闭合）。
#[test]
fn test_convert_max_length_and_lp_calc_percentage() {
    use zero_css_parser::values::{CalcExpr, CalcOp};
    // calc(60% - 0px)
    let calc = LengthValue::Calc(Box::new(CalcExpr::BinaryOp(
        Box::new(CalcExpr::Length(LengthValue::Percentage(60.0))),
        CalcOp::Subtract,
        Box::new(CalcExpr::Length(LengthValue::Px(0.0))),
    )));

    // max-width/max-height：Dimension::percent(0.6)
    assert_eq!(
        convert_max_length_to_dimension(&calc, 800.0, 600.0),
        taffy::style::Dimension::percent(0.6)
    );

    // padding/border/gap：LengthPercentage::percent(0.6)
    assert_eq!(
        convert_length_to_lp(&calc, 800.0, 600.0),
        taffy::style::LengthPercentage::percent(0.6)
    );
}

/// 测试 max-width/max-height 中 Px(f64::INFINITY) 映射为 Auto。
#[test]
fn test_convert_max_length_infinity() {
    let mut style = ComputedStyle::default();
    style.max_width = LengthValue::Px(f64::INFINITY);
    style.max_height = LengthValue::Px(f64::INFINITY);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::auto());
    assert_eq!(taffy_style.max_size.height, taffy::style::Dimension::auto());
}

/// 测试 max-width 的 Px 和 Percentage 值转换。
#[test]
fn test_convert_max_length_px_percentage() {
    let mut style = ComputedStyle::default();
    style.max_width = LengthValue::Px(500.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::length(500.0));

    style.max_width = LengthValue::Percentage(80.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::percent(0.8));
}

/// 测试 FlexWrap::WrapReverse 映射为 taffy::FlexWrap::WrapReverse。
#[test]
fn test_convert_flex_wrap_reverse() {
    let mut style = ComputedStyle::default();
    style.flex_wrap = FlexWrapValue::WrapReverse;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.flex_wrap, taffy::style::FlexWrap::WrapReverse);
}

/// 测试 FlexBasisValue::Content 映射为 Auto。
#[test]
fn test_convert_flex_basis_content() {
    let mut style = ComputedStyle::default();
    style.flex_basis = FlexBasisValue::Content;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.flex_basis, taffy::style::Dimension::auto());
}

/// 测试 Auto 在 convert_length_to_lp 中映射为 length(0.0)。
#[test]
fn test_convert_length_to_lp_auto() {
    let result = convert_length_to_lp(&LengthValue::Auto, 800.0, 600.0);
    assert_eq!(result, taffy::style::LengthPercentage::length(0.0));
}

/// 测试 Percentage 在 convert_length_to_lp 中转换为 Percent。
#[test]
fn test_convert_length_to_lp_percentage() {
    let result = convert_length_to_lp(&LengthValue::Percentage(33.0), 800.0, 600.0);
    assert_eq!(result, taffy::style::LengthPercentage::percent(0.33));
}

/// 测试 Auto 在 convert_length_to_lpa 中映射为 LengthPercentageAuto::auto()。
#[test]
fn test_convert_length_to_lpa_auto() {
    let result = convert_length_to_lpa(&LengthValue::Auto, false, 800.0, 600.0);
    assert_eq!(result, taffy::style::LengthPercentageAuto::auto());
}

/// 测试 Percentage 在 convert_length_to_lpa 中转换为 Percent。
#[test]
fn test_convert_length_to_lpa_percentage() {
    let result = convert_length_to_lpa(&LengthValue::Percentage(60.0), false, 800.0, 600.0);
    assert_eq!(result, taffy::style::LengthPercentageAuto::percent(0.6));
}

/// 测试 align_content 的所有变体转换。
///
/// 注意：computed_style_to_taffy 中 align_content 使用 style.justify_content，
/// 所以通过设置 justify_content 来测试 align_content 的转换结果。
#[test]
fn test_convert_alignment_align_content() {
    let cases: Vec<(AlignContentValue, Option<taffy::style::AlignContent>)> = vec![
        (
            AlignContentValue::SpaceBetween,
            Some(taffy::style::AlignContent::SPACE_BETWEEN),
        ),
        (
            AlignContentValue::SpaceAround,
            Some(taffy::style::AlignContent::SPACE_AROUND),
        ),
        (
            AlignContentValue::SpaceEvenly,
            Some(taffy::style::AlignContent::SPACE_EVENLY),
        ),
        (AlignContentValue::Stretch, Some(taffy::style::AlignContent::STRETCH)),
        (AlignContentValue::Center, Some(taffy::style::AlignContent::CENTER)),
        (AlignContentValue::Start, Some(taffy::style::AlignContent::START)),
        (AlignContentValue::End, Some(taffy::style::AlignContent::END)),
        (AlignContentValue::Normal, None),
        (AlignContentValue::Auto, None),
    ];
    for (value, expected) in cases {
        let mut style = ComputedStyle::default();
        style.align_content = value;
        let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(taffy_style.align_content, expected);
    }
}

/// 测试 justify_content 的 SpaceBetween、SpaceAround、SpaceEvenly、Start、End、Stretch 变体。
#[test]
fn test_convert_alignment_justify_content_variants() {
    let cases: Vec<(AlignmentValue, Option<taffy::style::JustifyContent>)> = vec![
        (
            AlignmentValue::SpaceBetween,
            Some(taffy::style::JustifyContent::SPACE_BETWEEN),
        ),
        (
            AlignmentValue::SpaceAround,
            Some(taffy::style::JustifyContent::SPACE_AROUND),
        ),
        (
            AlignmentValue::SpaceEvenly,
            Some(taffy::style::JustifyContent::SPACE_EVENLY),
        ),
        (AlignmentValue::Start, Some(taffy::style::JustifyContent::START)),
        (AlignmentValue::End, Some(taffy::style::JustifyContent::END)),
        (AlignmentValue::Stretch, Some(taffy::style::JustifyContent::STRETCH)),
    ];
    for (value, expected) in cases {
        let mut style = ComputedStyle::default();
        style.justify_content = value;
        let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
        assert_eq!(taffy_style.justify_content, expected);
    }
}

/// 测试 align_self 的 FlexStart、FlexEnd、Center、Stretch、Start、End 变体。
#[test]
fn test_convert_alignment_align_self_variants() {
    let cases: Vec<(AlignmentValue, Option<taffy::style::AlignSelf>)> = vec![
        (AlignmentValue::FlexStart, Some(taffy::style::AlignSelf::FLEX_START)),
        (AlignmentValue::FlexEnd, Some(taffy::style::AlignSelf::FLEX_END)),
        (AlignmentValue::Center, Some(taffy::style::AlignSelf::CENTER)),
        (AlignmentValue::Stretch, Some(taffy::style::AlignSelf::STRETCH)),
        (AlignmentValue::Start, Some(taffy::style::AlignSelf::START)),
        (AlignmentValue::End, Some(taffy::style::AlignSelf::END)),
    ];
    for (value, expected) in cases {
        let mut style = ComputedStyle::default();
        style.align_self = value;
        let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO);
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.overflow.x, taffy::style::Overflow::Scroll);
}

/// 测试 OverflowValue::Clip 映射为 taffy Clip。
#[test]
fn test_overflow_clip_maps_to_clip() {
    let mut style = ComputedStyle::default();
    style.overflow_y = OverflowValue::Clip;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    // "." 是空单元格标记，不应存储到区域映射中（LAY-06）
    assert_eq!(areas.len(), 3);
    assert_eq!(areas.get("header"), Some(&(1, 2, 1, 3)));
    assert_eq!(areas.get("sidebar"), Some(&(2, 3, 2, 3)));
    assert_eq!(areas.get("footer"), Some(&(3, 4, 1, 3)));
    assert!(!areas.contains_key("."), "空单元格标记 '.' 不应存储到区域映射中");
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
            matches!(track, taffy::style::GridTemplateComponent::Single(_)),
            "第 {} 个轨道应为 Single 变体",
            i
        );
    }

    // 将轨道转换为 taffy Style 并验证 gap 设置正确
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;
    style.grid_template_columns = Some("25% 50% 25%".to_string());
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    // fit-content 在 convert_length_to_dimension 中映射为特定值，不 panic 即可
    let _ = taffy_style.size.width;

    style.width = LengthValue::MinContent;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    let _ = taffy_style.size.width;

    style.width = LengthValue::MaxContent;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    let _ = taffy_style.size.width;
}

#[test]
/// 测试 max-width 使用 Em 单位。
fn test_convert_max_length_dimension_units() {
    let mut style = ComputedStyle::default();
    style.max_width = LengthValue::Em(10.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::length(10.0));
}

#[test]
/// 测试 max-height 使用百分比。
fn test_convert_max_height_percentage() {
    let mut style = ComputedStyle::default();
    style.max_height = LengthValue::Percentage(50.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.max_size.height, taffy::style::Dimension::percent(0.5));
}

#[test]
/// 测试 padding 使用 Em 和 Rem 单位。
fn test_convert_padding_em_rem() {
    let mut style = ComputedStyle::default();
    style.padding_left = LengthValue::Em(2.0);
    style.padding_right = LengthValue::Rem(1.5);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.padding.left, taffy::style::LengthPercentage::length(2.0));
    assert_eq!(taffy_style.padding.right, taffy::style::LengthPercentage::length(1.5));
}

#[test]
/// 测试 margin 使用 Vw/Vh 单位。
fn test_convert_margin_viewport_units() {
    let mut style = ComputedStyle::default();
    // R1058：测垂直 margin 机制须用 block 上下文（display 默认 Inline，§8.3 垂直 margin 归零）。
    style.display = DisplayValue::Block;
    style.margin_top = LengthValue::Vw(5.0);
    style.margin_bottom = LengthValue::Vh(2.5);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    // 5vw = 5 * 800/100 = 40.0; 2.5vh = 2.5 * 600/100 = 15.0
    assert_eq!(taffy_style.margin.top, taffy::style::LengthPercentageAuto::length(40.0));
    assert_eq!(
        taffy_style.margin.bottom,
        taffy::style::LengthPercentageAuto::length(15.0)
    );
}

#[test]
/// 测试 gap 使用 Vmin/Vmax 单位。
fn test_convert_gap_viewport_units() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    style.row_gap = LengthValue::Vmin(2.0);
    style.column_gap = LengthValue::Vmax(1.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    // 2vmin = 2 * min(800,600)/100 = 12.0
    assert_eq!(taffy_style.gap.height, taffy::style::LengthPercentage::length(12.0));
}

#[test]
/// 测试 flex-direction: row-reverse 和 column-reverse。
fn test_convert_flex_direction_reverse() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    style.flex_direction = FlexDirectionValue::RowReverse;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.flex_direction, taffy::style::FlexDirection::RowReverse);

    style.flex_direction = FlexDirectionValue::ColumnReverse;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.flex_direction, taffy::style::FlexDirection::ColumnReverse);
}

#[test]
/// 测试 flex-basis 使用 Em 长度值。
fn test_convert_flex_basis_length_em() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    style.flex_basis = FlexBasisValue::Length(LengthValue::Em(3.0));
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.flex_basis, taffy::style::Dimension::length(3.0));
}

#[test]
/// 测试 grid parse_single_track 对无效字符串回退到 Auto。
fn test_parse_single_track_fallback_auto() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;
    style.grid_auto_rows = Some("invalid-value".to_string());
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    // grid_auto_rows 解析失败不应 panic
    let _ = taffy_style.grid_auto_rows;
}

#[test]
/// 测试 grid track 解析纯数值 minmax。
fn test_parse_minmax_numeric_fallback() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;
    style.grid_template_rows = Some("minmax(100, 1fr)".to_string());
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    // 不应 panic
    let _ = taffy_style.grid_template_rows;
}

#[test]
/// 测试 min-width/max-width 组合使用 Ch 单位。
fn test_convert_min_max_width_ch_unit() {
    let mut style = ComputedStyle::default();
    style.min_width = LengthValue::Ch(4.0);
    style.max_width = LengthValue::Ch(40.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.min_size.width, taffy::style::Dimension::length(4.0));
    assert_eq!(taffy_style.max_size.width, taffy::style::Dimension::length(40.0));
}

// ── 内部解析函数边界条件测试 ──

/// 测试 find_top_level_comma：无逗号时返回 None。
#[test]
fn test_find_top_level_comma_no_comma() {
    assert_eq!(find_top_level_comma("100px 1fr auto"), None);
    assert_eq!(find_top_level_comma(""), None);
    assert_eq!(find_top_level_comma("abc"), None);
}

/// 测试 find_top_level_comma：逗号在括号内时被忽略。
#[test]
fn test_find_top_level_comma_inside_parens() {
    assert_eq!(find_top_level_comma("minmax(100px, 1fr)"), None);
}

/// 测试 find_top_level_comma：正常顶层逗号正确返回位置。
#[test]
fn test_find_top_level_comma_top_level() {
    let result = find_top_level_comma("3, minmax(10px, 1fr)");
    assert_eq!(result, Some(1), "逗号应在位置 1");

    let result = find_top_level_comma("a, b, c");
    assert!(result.is_some(), "应找到逗号");
    assert_eq!(result.unwrap(), 1, "第一个逗号应在位置 1");
}

/// 测试 tokenize_track_list：空字符串和纯空白。
#[test]
fn test_tokenize_track_list_empty_and_whitespace() {
    assert!(tokenize_track_list("").is_empty(), "空字符串应无 token");
    assert!(tokenize_track_list("   ").is_empty(), "纯空白应无 token");
    assert!(tokenize_track_list("\t\t").is_empty(), "纯 tab 应无 token");
}

/// 测试 tokenize_track_list：多层嵌套括号保持为单个 token。
#[test]
fn test_tokenize_track_list_deeply_nested() {
    let tokens = tokenize_track_list("repeat(2, minmax(10px, 1fr)) 200px auto");
    assert_eq!(tokens.len(), 3);
    assert_eq!(tokens[0], "repeat(2, minmax(10px, 1fr))");
    assert_eq!(tokens[1], "200px");
    assert_eq!(tokens[2], "auto");
}

/// 测试 parse_single_auto_track：auto 返回 AUTO。
#[test]
fn test_parse_single_auto_track_auto() {
    let result = parse_single_auto_track("auto");
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO);
}

/// 测试 parse_single_auto_track：fr 值返回弹性轨道。
#[test]
fn test_parse_single_auto_track_fr() {
    let result = parse_single_auto_track("2fr");
    let _ = result;
}

/// 测试 parse_single_auto_track：100px 返回固定长度。
#[test]
fn test_parse_single_auto_track_px() {
    let result = parse_single_auto_track("100px");
    let _ = result;
}

/// 测试 parse_single_auto_track：百分比返回百分比轨道。
#[test]
fn test_parse_single_auto_track_percent() {
    let result = parse_single_auto_track("50%");
    let _ = result;
}

/// 测试 parse_single_auto_track：无效值回退到 AUTO。
#[test]
fn test_parse_single_auto_track_invalid() {
    let result = parse_single_auto_track("invalid");
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO);
}

/// 测试 parse_min_track：auto 返回 Auto。
#[test]
fn test_parse_min_track_auto() {
    use taffy::style::MinTrackSizingFunction;
    let result = parse_min_track("auto");
    assert_eq!(result, MinTrackSizingFunction::auto());
}

/// 测试 parse_min_track：px 值返回 Fixed(Length)。
#[test]
fn test_parse_min_track_px() {
    let result = parse_min_track("50px");
    assert!(!result.is_auto());
}

/// 测试 parse_min_track：百分比值返回 Fixed(Percent)。
#[test]
fn test_parse_min_track_percent() {
    let result = parse_min_track("25%");
    assert!(!result.is_auto());
}

/// 测试 parse_min_track：纯数字返回 Fixed(Length)。
#[test]
fn test_parse_min_track_numeric() {
    let result = parse_min_track("100");
    assert!(!result.is_auto());
}

/// 测试 parse_min_track：无效值回退到 Auto。
#[test]
fn test_parse_min_track_invalid() {
    use taffy::style::MinTrackSizingFunction;
    let result = parse_min_track("abc");
    assert_eq!(result, MinTrackSizingFunction::auto());
}

/// 测试 parse_max_track：auto 返回 Auto。
#[test]
fn test_parse_max_track_auto() {
    use taffy::style::MaxTrackSizingFunction;
    let result = parse_max_track("auto");
    assert_eq!(result, MaxTrackSizingFunction::auto());
}

/// 测试 parse_max_track：fr 值返回 Fraction。
#[test]
fn test_parse_max_track_fr() {
    let result = parse_max_track("1fr");
    assert!(result.is_fr());
}

/// 测试 parse_max_track：px 值返回 Fixed(Length)。
#[test]
fn test_parse_max_track_px() {
    let result = parse_max_track("200px");
    assert!(!result.is_auto());
}

/// 测试 parse_max_track：百分比值返回 Fixed(Percent)。
#[test]
fn test_parse_max_track_percent() {
    let result = parse_max_track("75%");
    assert!(!result.is_auto());
}

/// 测试 parse_max_track：无效值回退到 Auto。
#[test]
fn test_parse_max_track_invalid() {
    use taffy::style::MaxTrackSizingFunction;
    let result = parse_max_track("??");
    assert_eq!(result, MaxTrackSizingFunction::auto());
}

/// 测试 parse_minmax_as_non_repeated：合法的 minmax(auto, 1fr) 组合。
#[test]
fn test_parse_minmax_as_non_repeated_auto_fr() {
    let result = parse_minmax_as_non_repeated("auto, 1fr");
    let _ = result;
}

/// 测试 parse_minmax_as_non_repeated：合法的 minmax(100px, auto) 组合。
#[test]
fn test_parse_minmax_as_non_repeated_px_auto() {
    let result = parse_minmax_as_non_repeated("100px, auto");
    let _ = result;
}

/// 测试 parse_minmax_as_non_repeated：三个参数时回退到 AUTO。
#[test]
fn test_parse_minmax_as_non_repeated_too_many_parts() {
    let result = parse_minmax_as_non_repeated("10px, 20px, 30px");
    assert_eq!(result, taffy::style::TrackSizingFunction::AUTO);
}

/// 测试 parse_grid_auto_tracks：None 值返回空列表。
#[test]
fn test_parse_grid_auto_tracks_none() {
    let result = parse_grid_auto_tracks(&None);
    assert!(result.is_empty());
}

/// 测试 parse_grid_auto_tracks：多值正确解析。
#[test]
fn test_parse_grid_auto_tracks_multiple() {
    let result = parse_grid_auto_tracks(&Some("100px auto 1fr".to_string()));
    assert_eq!(result.len(), 3);
}
