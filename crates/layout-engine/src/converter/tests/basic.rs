// Auto-generated test file — split from layout-engine/converter.rs
use super::super::*;

/// 测试 Block display 转换。
#[test]
fn test_convert_block_display() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Block;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.display, taffy::style::Display::Block);
}

/// 测试 Flex display 转换。
#[test]
fn test_convert_flex_display() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.display, taffy::style::Display::Flex);
}

/// 测试 Grid display 转换。
#[test]
fn test_convert_grid_display() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.display, taffy::style::Display::Grid);
}

/// 测试 None display 转换。
#[test]
fn test_convert_none_display() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::None;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.display, taffy::style::Display::None);
}

/// 测试 Inline display 映射为 Block。
#[test]
fn test_convert_inline_display() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Inline;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.display, taffy::style::Display::Block);

    style.display = DisplayValue::InlineBlock;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.display, taffy::style::Display::Block);
}

/// 测试 position: absolute 转换。
#[test]
fn test_convert_position_absolute() {
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Absolute;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.position, taffy::style::Position::Absolute);
}

/// 测试 position: relative 转换。
#[test]
fn test_convert_position_relative() {
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Relative;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.position, taffy::style::Position::Relative);

    style.position = PositionValue::Static;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.position, taffy::style::Position::Relative);
}

/// 测试 position: fixed 映射为 taffy Absolute（脱离正常流）。
#[test]
fn test_convert_position_fixed() {
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Fixed;
    style.top = LengthValue::Px(10.0);
    style.left = LengthValue::Px(20.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(
        taffy_style.position,
        taffy::style::Position::Absolute,
        "position:fixed should map to taffy Absolute"
    );
    // inset 应正确传递
    assert_eq!(taffy_style.inset.top, taffy::style::LengthPercentageAuto::length(10.0));
    assert_eq!(taffy_style.inset.left, taffy::style::LengthPercentageAuto::length(20.0));
}

/// 测试 position: sticky 映射为 taffy Relative（保持正常流）。
#[test]
fn test_convert_position_sticky() {
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Sticky;
    style.top = LengthValue::Px(5.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::length(200.0));
    assert_eq!(taffy_style.size.height, taffy::style::Dimension::length(100.0));
}

/// 测试 size auto 转换（Px(0.0) 表示 auto）。
#[test]
fn test_convert_size_auto() {
    let style = ComputedStyle::default();
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::auto());
    assert_eq!(taffy_style.size.height, taffy::style::Dimension::auto());
}

/// 测试 margin/padding/border 转换。
#[test]
fn test_convert_margin_padding_border() {
    let mut style = ComputedStyle::default();
    // R1058：测垂直 margin 机制须用 block 上下文（display 默认 Inline，§8.3 垂直 margin 归零）。
    style.display = DisplayValue::Block;
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
    // border-style=Solid 方能使 border-width 进入布局盒（CSS §8.5.3：style=none→width=0）
    style.border_top_style = zero_style_system::BorderStyleValue::Solid;
    style.border_right_style = zero_style_system::BorderStyleValue::Solid;
    style.border_bottom_style = zero_style_system::BorderStyleValue::Solid;
    style.border_left_style = zero_style_system::BorderStyleValue::Solid;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.margin.top, taffy::style::LengthPercentageAuto::length(10.0));
    assert_eq!(
        taffy_style.margin.left,
        taffy::style::LengthPercentageAuto::length(20.0)
    );
    assert_eq!(taffy_style.padding.top, taffy::style::LengthPercentage::length(5.0));
    assert_eq!(taffy_style.border.top, taffy::style::LengthPercentage::length(1.0));
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.flex_direction, taffy::style::FlexDirection::Column);
    assert_eq!(taffy_style.flex_wrap, taffy::style::FlexWrap::Wrap);
    assert!((taffy_style.flex_grow - 2.0).abs() < 0.001);
    assert!((taffy_style.flex_shrink - 0.5).abs() < 0.001);
    assert_eq!(taffy_style.flex_basis, taffy::style::Dimension::length(100.0));
}

/// 测试对齐属性转换。
#[test]
fn test_convert_alignment() {
    let mut style = ComputedStyle::default();
    style.justify_content = AlignmentValue::Center;
    style.align_items = AlignmentValue::FlexEnd;
    style.align_self = AlignmentValue::Baseline;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.justify_content, Some(taffy::style::JustifyContent::CENTER));
    assert_eq!(taffy_style.align_items, Some(taffy::style::AlignItems::FLEX_END));
    assert_eq!(taffy_style.align_self, Some(taffy::style::AlignSelf::BASELINE));
}

/// 测试 gap 转换（column-gap 和 row-gap 独立）。
#[test]
fn test_convert_gap() {
    let mut style = ComputedStyle::default();
    style.gap = LengthValue::Px(10.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.gap.width, taffy::style::LengthPercentage::length(10.0));
    // row_gap 未设置时回退到 gap 简写（CSS 规范：gap: 10px 等价于 row-gap: 10px column-gap: 10px）
    assert_eq!(taffy_style.gap.height, taffy::style::LengthPercentage::length(10.0));

    // 设置不同的 row-gap 时，使用显式值而非 gap 回退
    style.row_gap = LengthValue::Px(20.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.gap.width, taffy::style::LengthPercentage::length(10.0));
    assert_eq!(taffy_style.gap.height, taffy::style::LengthPercentage::length(20.0));
}

/// 测试 overflow 转换。
#[test]
fn test_convert_overflow() {
    let mut style = ComputedStyle::default();
    style.overflow_x = OverflowValue::Hidden;
    style.overflow_y = OverflowValue::Scroll;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.inset.top, taffy::style::LengthPercentageAuto::length(10.0));
    assert_eq!(
        taffy_style.inset.right,
        taffy::style::LengthPercentageAuto::length(20.0)
    );
    assert_eq!(
        taffy_style.inset.bottom,
        taffy::style::LengthPercentageAuto::length(30.0)
    );
    assert_eq!(taffy_style.inset.left, taffy::style::LengthPercentageAuto::length(40.0));
}

/// 测试 box-sizing 转换。
#[test]
fn test_convert_box_sizing() {
    let mut style = ComputedStyle::default();
    style.box_sizing = BoxSizingValue::BorderBox;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.box_sizing, taffy::style::BoxSizing::BorderBox);

    style.box_sizing = BoxSizingValue::ContentBox;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.box_sizing, taffy::style::BoxSizing::ContentBox);
}

/// 测试 grid-template-columns/rows 转换。
#[test]
fn test_convert_grid_template() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;
    style.grid_template_columns = Some("100px 200px 1fr".to_string());
    style.grid_template_rows = Some("auto 50px".to_string());
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.grid_auto_flow, taffy::style::GridAutoFlow::Column);

    style.grid_auto_flow = GridAutoFlowValue::RowDense;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.grid_auto_flow, taffy::style::GridAutoFlow::RowDense);
}

/// 测试 row-gap 转换。
#[test]
fn test_convert_row_gap() {
    let mut style = ComputedStyle::default();
    style.gap = LengthValue::Px(10.0);
    style.row_gap = LengthValue::Px(20.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.gap.width, taffy::style::LengthPercentage::length(10.0));
    assert_eq!(taffy_style.gap.height, taffy::style::LengthPercentage::length(20.0));
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    use taffy::style::RepetitionCount;

    let tracks = parse_grid_tracks(&Some("repeat(auto-fill, 200px)".to_string()));
    assert_eq!(tracks.len(), 1);
    assert!(
        matches!(
            &tracks[0],
            taffy::style::GridTemplateComponent::Repeat(rep) if rep.count == RepetitionCount::AutoFill
        ),
        "auto-fill 应生成 Repeat 变体"
    );

    let tracks = parse_grid_tracks(&Some("repeat(auto-fit, minmax(100px, 1fr))".to_string()));
    assert_eq!(tracks.len(), 1);
    assert!(
        matches!(
            &tracks[0],
            taffy::style::GridTemplateComponent::Repeat(rep) if rep.count == RepetitionCount::AutoFit
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.grid_auto_rows.len(), 2);
}

/// 测试 grid-auto-columns 转换。
#[test]
fn test_convert_grid_auto_columns() {
    let mut style = ComputedStyle::default();
    style.grid_auto_columns = Some("1fr auto".to_string());
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.grid_auto_columns.len(), 2);
}

/// 测试 grid-auto-rows/columns 默认值为空。
#[test]
fn test_convert_grid_auto_default() {
    let style = ComputedStyle::default();
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    let areas = parse_grid_template_areas("\"header header header\" \"sidebar main main\" \"sidebar footer footer\"");
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

    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.aspect_ratio, Some(1.5));
}

/// 测试 aspect-ratio 为 None 时 taffy Style 中也为 None。
#[test]
fn test_aspect_ratio_none_in_taffy_style() {
    let style = ComputedStyle::default();
    assert_eq!(style.aspect_ratio, None);

    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.aspect_ratio, None);
}

/// 测试 float 元素在 flex 容器中的转换 — float 在 flex 上下文中应仍返回 true。
#[test]
fn test_float_in_mixed_layout_context() {
    // float: left
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    style.float = FloatValue::Left;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);

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
    use taffy::style::RepetitionCount;

    let tracks = parse_grid_tracks(&Some("repeat(auto-fill, minmax(auto, 1fr))".to_string()));
    assert_eq!(tracks.len(), 1);
    assert!(
        matches!(
            &tracks[0],
            taffy::style::GridTemplateComponent::Repeat(rep) if rep.count == RepetitionCount::AutoFill
        ),
        "auto-fill + minmax(auto, 1fr) 应生成 Repeat 变体"
    );
}

/// 测试 grid-auto-rows 使用固定值和 fr 单位。
#[test]
fn test_grid_auto_rows_with_various_values() {
    let mut style = ComputedStyle::default();
    style.grid_auto_rows = Some("50px auto".to_string());
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.grid_auto_rows.len(), 2);

    // 单值
    style.grid_auto_rows = Some("100px".to_string());
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.grid_auto_rows.len(), 1);

    // fr 单位
    style.grid_auto_rows = Some("1fr".to_string());
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.grid_auto_rows.len(), 1);
}

// -- 边界条件测试 --

/// 测试 aspect-ratio auto 不设置值
#[test]
fn test_aspect_ratio_auto_conversion() {
    // aspect_ratio 为 None（Auto）时，taffy style 中应为 None
    let style = ComputedStyle::default();
    assert_eq!(style.aspect_ratio, None);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.aspect_ratio, None, "auto aspect-ratio 应转换为 None");
}

/// 测试 grid-auto-flow dense 转换
#[test]
fn test_grid_auto_flow_dense_conversion() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;

    // RowDense
    style.grid_auto_flow = GridAutoFlowValue::RowDense;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.grid_auto_flow, taffy::style::GridAutoFlow::RowDense);

    // ColumnDense
    style.grid_auto_flow = GridAutoFlowValue::ColumnDense;
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
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
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.inset.top, taffy::style::LengthPercentageAuto::length(10.0));
    assert_eq!(
        taffy_style.inset.right,
        taffy::style::LengthPercentageAuto::length(20.0)
    );
    assert_eq!(
        taffy_style.inset.bottom,
        taffy::style::LengthPercentageAuto::length(30.0)
    );
    assert_eq!(taffy_style.inset.left, taffy::style::LengthPercentageAuto::length(40.0));
}

/// 测试 flex-basis: 0 转换
#[test]
fn test_flex_basis_zero() {
    // flex-basis: 0px 应转换为 Length(0.0)
    let mut style = ComputedStyle::default();
    style.flex_basis = FlexBasisValue::Length(LengthValue::Px(0.0));
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.flex_basis, taffy::style::Dimension::length(0.0));
}

/// 测试 percentage 宽高转换
#[test]
fn test_percentage_size_conversion() {
    // width: 50% 应转换为 Percent(0.5)
    let mut style = ComputedStyle::default();
    style.width = LengthValue::Percentage(50.0);
    style.height = LengthValue::Percentage(75.0);
    let taffy_style = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(taffy_style.size.width, taffy::style::Dimension::percent(0.5));
    assert_eq!(taffy_style.size.height, taffy::style::Dimension::percent(0.75));
}

// ── 边界条件测试（第二批）──
