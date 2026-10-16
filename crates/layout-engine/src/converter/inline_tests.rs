use super::*;
use zero_css_parser::values::{
    AlignmentValue, BoxSizingValue, ClearValue, DisplayValue, FlexDirectionValue, FlexWrapValue, FloatValue,
    LengthValue, OverflowValue, PositionValue, VisibilityValue,
};
use zero_style_system::{ComputedStyle, FlexBasisValue, GridAutoFlowValue, GridLineValue};

// ── computed_style_to_taffy ─────────────────────────────────────────

#[test]
fn test_computed_style_to_taffy_default() {
    let style = ComputedStyle::default();
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.display, taffy::style::Display::Block);
}

#[test]
fn test_computed_style_to_taffy_flex() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.display, taffy::style::Display::Flex);
}

#[test]
fn test_computed_style_to_taffy_grid() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.display, taffy::style::Display::Grid);
}

#[test]
fn test_computed_style_to_taffy_position_relative() {
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Relative;
    style.top = LengthValue::Px(10.0);
    style.left = LengthValue::Px(20.0);
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.position, taffy::style::Position::Relative);
    assert_eq!(result.inset.top, taffy::style::LengthPercentageAuto::length(10.0));
    assert_eq!(result.inset.left, taffy::style::LengthPercentageAuto::length(20.0));
}

#[test]
fn test_computed_style_to_taffy_position_absolute() {
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Absolute;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.position, taffy::style::Position::Absolute);
}

#[test]
fn test_computed_style_to_taffy_position_fixed() {
    let mut style = ComputedStyle::default();
    style.position = PositionValue::Fixed;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.position, taffy::style::Position::Absolute); // taffy maps fixed to absolute
}

#[test]
fn test_computed_style_to_taffy_padding() {
    let mut style = ComputedStyle::default();
    // 垂直 padding 机制须用 block 上下文（display 默认 Inline，§10.8.1 垂直 padding 归零）。
    style.display = DisplayValue::Block;
    style.padding_top = LengthValue::Px(10.0);
    style.padding_right = LengthValue::Px(20.0);
    style.padding_bottom = LengthValue::Px(30.0);
    style.padding_left = LengthValue::Px(40.0);
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.padding.top, taffy::style::LengthPercentage::length(10.0));
    assert_eq!(result.padding.right, taffy::style::LengthPercentage::length(20.0));
    assert_eq!(result.padding.bottom, taffy::style::LengthPercentage::length(30.0));
    assert_eq!(result.padding.left, taffy::style::LengthPercentage::length(40.0));
}

#[test]
fn test_table_padding_zeroed_in_border_collapse() {
    // CSS 2.1 §17.6.2：border-collapse:collapse 模式下 table 盒的 padding 不应用。
    // display:table + border-collapse:collapse → padding 归零（不论显式 padding 多大）。
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Table;
    style.border_collapse = BorderCollapseValue::Collapse;
    style.padding_top = LengthValue::Px(100.0);
    style.padding_right = LengthValue::Px(100.0);
    style.padding_bottom = LengthValue::Px(100.0);
    style.padding_left = LengthValue::Px(100.0);
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    let zero = taffy::style::LengthPercentage::length(0.0_f32);
    assert_eq!(result.padding.top, zero);
    assert_eq!(result.padding.right, zero);
    assert_eq!(result.padding.bottom, zero);
    assert_eq!(result.padding.left, zero);
}

#[test]
fn test_table_padding_kept_in_border_separate() {
    // border-collapse:separate（默认）→ table padding 正常应用（§17.5：仅 collapse 模式禁用 table padding）。
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Table;
    // border_collapse 默认 Separate
    style.padding_top = LengthValue::Px(10.0);
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.padding.top, taffy::style::LengthPercentage::length(10.0));
}

#[test]
fn test_computed_style_to_taffy_margin_auto() {
    let mut style = ComputedStyle::default();
    style.margin_left = LengthValue::Auto;
    style.margin_right = LengthValue::Auto;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.margin.left, taffy::style::LengthPercentageAuto::auto());
    assert_eq!(result.margin.right, taffy::style::LengthPercentageAuto::auto());
}

#[test]
fn test_computed_style_to_taffy_size_percentage() {
    let mut style = ComputedStyle::default();
    style.width = LengthValue::Percentage(50.0);
    style.height = LengthValue::Percentage(75.0);
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.size.width, taffy::style::Dimension::percent(0.5));
    assert_eq!(result.size.height, taffy::style::Dimension::percent(0.75));
}

#[test]
fn test_computed_style_to_taffy_size_auto() {
    let mut style = ComputedStyle::default();
    style.width = LengthValue::Auto;
    style.height = LengthValue::Auto;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.size.width, taffy::style::Dimension::auto());
    assert_eq!(result.size.height, taffy::style::Dimension::auto());
}

#[test]
fn test_computed_style_to_taffy_min_max_size() {
    let mut style = ComputedStyle::default();
    style.min_width = LengthValue::Px(100.0);
    style.max_width = LengthValue::Px(500.0);
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.min_size.width, taffy::style::Dimension::length(100.0));
    assert_eq!(result.max_size.width, taffy::style::Dimension::length(500.0));
}

#[test]
fn test_computed_style_to_taffy_overflow() {
    let mut style = ComputedStyle::default();
    style.overflow_x = OverflowValue::Hidden;
    style.overflow_y = OverflowValue::Scroll;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.overflow.x, taffy::style::Overflow::Hidden);
    assert_eq!(result.overflow.y, taffy::style::Overflow::Scroll);
}

#[test]
fn test_computed_style_to_taffy_overflow_auto_no_scrollbar_reservation() {
    // R4265：overflow:auto（两轴）零滚动条预留——taffy 仅对 Scroll 轴预留，auto
    // 容器空内容时零槽位（chromium auto 滚动条仅内容溢出出现）。
    let mut style = ComputedStyle::default();
    style.overflow_x = OverflowValue::Auto;
    style.overflow_y = OverflowValue::Auto;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.scrollbar_width, 0.0);
    // 任一轴显式 scroll 维持预留（classic 恒预留）。
    style.overflow_x = OverflowValue::Scroll;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.scrollbar_width, 15.0);
}

#[test]
fn test_computed_style_to_taffy_flex_direction() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    style.flex_direction = FlexDirectionValue::RowReverse;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.flex_direction, taffy::style::FlexDirection::RowReverse);
}

#[test]
fn test_computed_style_to_taffy_flex_wrap() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    style.flex_wrap = FlexWrapValue::Wrap;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.flex_wrap, taffy::style::FlexWrap::Wrap);
}

#[test]
fn test_computed_style_to_taffy_flex_basis_auto() {
    let mut style = ComputedStyle::default();
    style.flex_basis = FlexBasisValue::Auto;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.flex_basis, taffy::style::Dimension::auto());
}

#[test]
fn test_computed_style_to_taffy_flex_basis_length() {
    let mut style = ComputedStyle::default();
    style.flex_basis = FlexBasisValue::Length(LengthValue::Px(200.0));
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.flex_basis, taffy::style::Dimension::length(200.0));
}

#[test]
fn test_visibility_collapse_zeros_flex_main_size() {
    // CSS Flexbox §10.1：visibility:collapse 的 flex item 主尺寸归零（strut），
    // 即使声明了显式 flex-basis / flex-grow / flex-shrink 也被覆盖。
    // 对应上游 reftest: flexbox-collapsed-item-horiz-001/002。
    let mut style = ComputedStyle::default();
    style.visibility = VisibilityValue::Collapse;
    style.flex_basis = FlexBasisValue::Length(LengthValue::Px(200.0));
    style.flex_grow = 1.0;
    style.flex_shrink = 1.0;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.flex_basis, taffy::style::Dimension::length(0.0_f32));
    assert_eq!(result.flex_grow, 0.0);
    assert_eq!(result.flex_shrink, 0.0);
}

#[test]
fn test_visibility_visible_preserves_flex() {
    // 非 collapse 时 flex 属性正常透传。
    let mut style = ComputedStyle::default();
    style.flex_basis = FlexBasisValue::Length(LengthValue::Px(200.0));
    style.flex_grow = 2.0;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.flex_basis, taffy::style::Dimension::length(200.0));
    assert_eq!(result.flex_grow, 2.0);
}

#[test]
fn test_computed_style_to_taffy_align_items_center() {
    let mut style = ComputedStyle::default();
    style.align_items = AlignmentValue::Center;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.align_items, Some(taffy::style::AlignItems::CENTER));
}

#[test]
fn test_computed_style_to_taffy_justify_content_space_between() {
    let mut style = ComputedStyle::default();
    style.justify_content = AlignmentValue::SpaceBetween;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(
        result.justify_content,
        Some(taffy::style::JustifyContent::SPACE_BETWEEN)
    );
}

#[test]
fn test_computed_style_to_taffy_gap() {
    let mut style = ComputedStyle::default();
    style.gap = LengthValue::Px(20.0); // gap.width = style.gap
    style.row_gap = LengthValue::Px(10.0); // gap.height = style.row_gap
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.gap.width, taffy::style::LengthPercentage::length(20.0));
    assert_eq!(result.gap.height, taffy::style::LengthPercentage::length(10.0));
}

#[test]
fn test_computed_style_to_taffy_box_sizing() {
    let mut style = ComputedStyle::default();
    style.box_sizing = BoxSizingValue::BorderBox;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.box_sizing, taffy::style::BoxSizing::BorderBox);
}

#[test]
fn test_computed_style_to_taffy_display_none() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::None;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.display, taffy::style::Display::None);
}

// ── parse_grid_template_areas ───────────────────────────────────────

#[test]
fn test_parse_grid_template_areas_simple() {
    let input = r#""a a" "b b""#;
    let areas = parse_grid_template_areas(input);
    assert_eq!(areas.len(), 2);
    assert_eq!(areas.get("a"), Some(&(1, 2, 1, 3)));
    assert_eq!(areas.get("b"), Some(&(2, 3, 1, 3)));
}

#[test]
fn test_parse_grid_template_areas_single_cell() {
    let areas = parse_grid_template_areas(r#""main""#);
    assert_eq!(areas.get("main"), Some(&(1, 2, 1, 2)));
}

#[test]
fn test_parse_grid_template_areas_dot_is_skipped() {
    // "." is an empty cell marker and should not be stored in the area map
    let areas = parse_grid_template_areas(r#""a ." "b b""#);
    assert!(areas.contains_key("a"));
    assert!(areas.contains_key("b"));
    assert!(!areas.contains_key("."), "空单元格标记 '.' 不应存储到区域映射中");
}

#[test]
fn test_parse_grid_template_areas_empty() {
    let areas = parse_grid_template_areas("");
    assert!(areas.is_empty());
}

#[test]
fn test_parse_grid_template_areas_3x3() {
    let input = r#""h h h" "s m m" "s m m""#;
    let areas = parse_grid_template_areas(input);
    let h = areas.get("h").unwrap();
    assert_eq!(*h, (1, 2, 1, 4)); // row 1, cols 1-3
    let s = areas.get("s").unwrap();
    assert_eq!(*s, (2, 4, 1, 2)); // rows 2-3, col 1
    let m = areas.get("m").unwrap();
    assert_eq!(*m, (2, 4, 2, 4)); // rows 2-3, cols 2-3
}

// ── resolve_grid_placement ──────────────────────────────────────────

#[test]
fn test_resolve_grid_placement_auto() {
    let style = ComputedStyle::default();
    let (rs, re, cs, ce) = resolve_grid_placement(&style, None);
    assert_eq!(rs, GridLineValue::Auto);
    assert_eq!(re, GridLineValue::Auto);
    assert_eq!(cs, GridLineValue::Auto);
    assert_eq!(ce, GridLineValue::Auto);
}

#[test]
fn test_resolve_grid_placement_with_line_numbers() {
    let mut style = ComputedStyle::default();
    style.grid_row_start = GridLineValue::Line(2);
    style.grid_row_end = GridLineValue::Line(4);
    style.grid_column_start = GridLineValue::Line(1);
    style.grid_column_end = GridLineValue::Line(3);
    let (rs, re, cs, ce) = resolve_grid_placement(&style, None);
    assert_eq!(rs, GridLineValue::Line(2));
    assert_eq!(re, GridLineValue::Line(4));
    assert_eq!(cs, GridLineValue::Line(1));
    assert_eq!(ce, GridLineValue::Line(3));
}

#[test]
fn test_resolve_grid_placement_with_span() {
    let mut style = ComputedStyle::default();
    style.grid_row_start = GridLineValue::Line(1);
    style.grid_row_end = GridLineValue::Span(2);
    style.grid_column_start = GridLineValue::Span(3);
    style.grid_column_end = GridLineValue::Line(5);
    let (rs, re, cs, ce) = resolve_grid_placement(&style, None);
    assert_eq!(rs, GridLineValue::Line(1));
    assert_eq!(re, GridLineValue::Span(2));
    assert_eq!(cs, GridLineValue::Span(3));
    assert_eq!(ce, GridLineValue::Line(5));
}

#[test]
fn test_resolve_grid_placement_named_area() {
    let mut style = ComputedStyle::default();
    style.grid_row_start = GridLineValue::Name("header".to_string());
    style.grid_row_end = GridLineValue::Name("header".to_string());
    style.grid_column_start = GridLineValue::Name("header".to_string());
    style.grid_column_end = GridLineValue::Name("header".to_string());

    let mut areas = GridAreaMap::new();
    areas.insert("header".to_string(), (1, 2, 1, 4));

    let (rs, re, cs, ce) = resolve_grid_placement(&style, Some(&areas));
    assert_eq!(rs, GridLineValue::Line(1)); // row-start
    assert_eq!(re, GridLineValue::Line(2)); // row-end
    assert_eq!(cs, GridLineValue::Line(1)); // col-start
    assert_eq!(ce, GridLineValue::Line(4)); // col-end
}

#[test]
fn test_resolve_grid_placement_unknown_area_preserves_named_line() {
    let mut style = ComputedStyle::default();
    style.grid_row_start = GridLineValue::Name("nonexistent".to_string());

    let areas = GridAreaMap::new();
    let (rs, _, _, _) = resolve_grid_placement(&style, Some(&areas));
    assert_eq!(rs, GridLineValue::Name("nonexistent".to_string()));
}

// ── convert_float / convert_clear ───────────────────────────────────

#[test]
fn test_convert_float_none() {
    assert!(!convert_float(&FloatValue::None));
}

#[test]
fn test_convert_float_inline_end() {
    assert!(convert_float(&FloatValue::InlineEnd));
}

#[test]
fn test_convert_clear_both() {
    assert!(convert_clear(&ClearValue::Both));
}

#[test]
fn test_convert_clear_inline_start() {
    assert!(convert_clear(&ClearValue::InlineStart));
}

// ── grid auto flow ──────────────────────────────────────────────────

#[test]
fn test_computed_style_grid_auto_flow() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Grid;
    style.grid_auto_flow = GridAutoFlowValue::ColumnDense;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.grid_auto_flow, taffy::style::GridAutoFlow::ColumnDense);
}

// ── computed_style_to_taffy with border ─────────────────────────────

#[test]
fn test_computed_style_border() {
    let mut style = ComputedStyle::default();
    style.border_top_width = LengthValue::Px(1.0);
    style.border_right_width = LengthValue::Px(2.0);
    style.border_bottom_width = LengthValue::Px(3.0);
    style.border_left_width = LengthValue::Px(4.0);
    // border-style=Solid 方能使 border-width 进入布局盒（CSS §8.5.3：style=none→width=0）
    style.border_top_style = BorderStyleValue::Solid;
    style.border_right_style = BorderStyleValue::Solid;
    style.border_bottom_style = BorderStyleValue::Solid;
    style.border_left_style = BorderStyleValue::Solid;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.border.top, taffy::style::LengthPercentage::length(1.0));
    assert_eq!(result.border.right, taffy::style::LengthPercentage::length(2.0));
    assert_eq!(result.border.bottom, taffy::style::LengthPercentage::length(3.0));
    assert_eq!(result.border.left, taffy::style::LengthPercentage::length(4.0));
}

// ── ComputedStyle with percentage padding/margin ────────────────────

#[test]
fn test_computed_style_padding_percentage() {
    let mut style = ComputedStyle::default();
    // 垂直 padding 机制须用 block 上下文（display 默认 Inline，§10.8.1 垂直 padding 归零）。
    style.display = DisplayValue::Block;
    style.padding_top = LengthValue::Percentage(10.0);
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.padding.top, taffy::style::LengthPercentage::percent(0.1));
}

#[test]
fn test_computed_style_margin_percentage() {
    let mut style = ComputedStyle::default();
    // R1058：测垂直 margin 机制须用 block 上下文（display 默认 Inline，§8.3 垂直 margin 归零）。
    style.display = DisplayValue::Block;
    style.margin_bottom = LengthValue::Percentage(25.0);
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.margin.bottom, taffy::style::LengthPercentageAuto::percent(0.25));
}

// CSS 2.1 §10.8.1 + §8.4：非替换 inline 的垂直 padding 不影响 line box 高度——taffy 盒
// 垂直 padding 归零（与 R1058 垂直 margin 归零同构）。水平 padding 保留。
#[test]
fn test_computed_style_inline_vertical_padding_zeroed() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Inline;
    style.padding_top = LengthValue::Px(16.0);
    style.padding_bottom = LengthValue::Px(16.0);
    style.padding_left = LengthValue::Px(8.0);
    style.padding_right = LengthValue::Px(8.0);
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.padding.top, taffy::style::LengthPercentage::length(0.0));
    assert_eq!(result.padding.bottom, taffy::style::LengthPercentage::length(0.0));
    // 水平 padding 有效（作用于 IFC 内 inline 片段宽度）
    assert_eq!(result.padding.left, taffy::style::LengthPercentage::length(8.0));
    assert_eq!(result.padding.right, taffy::style::LengthPercentage::length(8.0));
    // block 上下文不受影响（对照）
    style.display = DisplayValue::Block;
    let result = computed_style_to_taffy(&style, None, 800.0, 600.0);
    assert_eq!(result.padding.top, taffy::style::LengthPercentage::length(16.0));
    assert_eq!(result.padding.bottom, taffy::style::LengthPercentage::length(16.0));
}
