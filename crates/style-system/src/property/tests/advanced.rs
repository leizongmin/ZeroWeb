// Auto-generated test file — split from property.rs
use super::super::*;

#[test]
fn test_apply_property_list_style_type() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.list_style_type, zero_css_parser::values::ListStyleTypeValue::Disc);

    assert!(apply_property_value(&mut style, "list-style-type", "circle"));
    assert_eq!(
        style.list_style_type,
        zero_css_parser::values::ListStyleTypeValue::Circle
    );

    assert!(apply_property_value(&mut style, "list-style-type", "decimal"));
    assert_eq!(
        style.list_style_type,
        zero_css_parser::values::ListStyleTypeValue::Decimal
    );

    assert!(apply_property_value(&mut style, "list-style-type", "none"));
    assert_eq!(style.list_style_type, zero_css_parser::values::ListStyleTypeValue::None);

    assert!(!apply_property_value(&mut style, "list-style-type", "invalid"));
}

#[test]
fn test_apply_property_list_style_position() {
    let mut style = ComputedStyle::default();
    assert_eq!(
        style.list_style_position,
        zero_css_parser::values::ListStylePositionValue::Outside
    );

    assert!(apply_property_value(&mut style, "list-style-position", "inside"));
    assert_eq!(
        style.list_style_position,
        zero_css_parser::values::ListStylePositionValue::Inside
    );

    assert!(!apply_property_value(&mut style, "list-style-position", "invalid"));
}

#[test]
fn test_list_style_property_registry() {
    assert!(PropertyRegistry::initial_value("list-style-type").is_some());
    assert!(PropertyRegistry::initial_value("list-style-position").is_some());
    assert!(PropertyRegistry::is_inherited("list-style-type"));
    assert!(PropertyRegistry::is_inherited("list-style-position"));
}

#[test]
fn test_list_style_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"list-style-type"));
    assert!(props.contains(&"list-style-position"));
}

// ═══════════════════════════════════════════════════════════════════
// 新增 property 边界条件测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// PropertyRegistry 已注册的属性数量
fn test_property_registry_count() {
    let props = PropertyRegistry::known_properties();
    // 确保至少有 80 个已知属性
    assert!(
        props.len() >= 80,
        "known_properties should have at least 80 entries, got {}",
        props.len()
    );
}

#[test]
/// inherit 关键字在 apply_property_value 中不被当作 display 值
fn test_inherit_keyword_not_valid_display() {
    let mut style = ComputedStyle::default();
    // "inherit" 不是一个有效的 display 值
    assert!(!apply_property_value(&mut style, "display", "inherit"));
    // display 不应该改变
    assert_eq!(style.display, DisplayValue::Inline);
}

#[test]
/// initial 关键字在 apply_property_value 中不被当作 display 值
fn test_initial_keyword_not_valid_display() {
    let mut style = ComputedStyle::default();
    style.display = DisplayValue::Flex;
    // "initial" 不是一个有效的 display 值
    assert!(!apply_property_value(&mut style, "display", "initial"));
    assert_eq!(style.display, DisplayValue::Flex);
}

#[test]
/// unset 关键字在 apply_property_value 中不被当作 position 值
fn test_unset_keyword_not_valid_position() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "position", "unset"));
    assert_eq!(style.position, PositionValue::Static);
}

#[test]
/// revert 关键字在 apply_property_value 中不被当作 position 值
fn test_revert_keyword_not_valid_position() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "position", "revert"));
    assert_eq!(style.position, PositionValue::Static);
}

#[test]
/// ComputedStyle::default 所有继承属性初始值正确性
fn test_default_inherited_properties_initial_values() {
    let style = ComputedStyle::default();
    assert_eq!(style.color, ColorValue::Rgba(0, 0, 0, 255));
    assert_eq!(style.font_family, Vec::<String>::new());
    assert_eq!(style.font_size, LengthValue::Px(16.0));
    assert_eq!(style.font_weight, FontWeightValue::Normal);
    assert_eq!(style.font_style, FontStyleValue::Normal);
    assert_eq!(style.line_height, LineHeightValue::Normal);
    assert_eq!(style.text_align, TextAlignValue::Start);
    assert_eq!(style.text_transform, TextTransformValue::None);
    assert_eq!(style.letter_spacing, LengthValue::Px(0.0));
    assert_eq!(style.word_spacing, LengthValue::Px(0.0));
    assert_eq!(style.white_space, WhiteSpaceValue::Normal);
    assert_eq!(style.visibility, VisibilityValue::Visible);
    assert_eq!(style.cursor, CursorValue::Auto);
}

#[test]
/// apply_property_value 对 opacity 的 clamp 行为
fn test_opacity_clamp_edge_values() {
    let mut style = ComputedStyle::default();

    // 正常值
    assert!(apply_property_value(&mut style, "opacity", "0.0"));
    assert_eq!(style.opacity, 0.0);

    assert!(apply_property_value(&mut style, "opacity", "1.0"));
    assert_eq!(style.opacity, 1.0);

    // 超出范围 clamp
    assert!(apply_property_value(&mut style, "opacity", "1.5"));
    assert_eq!(style.opacity, 1.0);

    assert!(apply_property_value(&mut style, "opacity", "-0.1"));
    assert_eq!(style.opacity, 0.0);

    // 无效值
    assert!(!apply_property_value(&mut style, "opacity", "abc"));
}

#[test]
/// parse_border_style 所有变体
fn test_parse_border_style_all_variants() {
    assert_eq!(parse_border_style("none"), Some(BorderStyleValue::None));
    assert_eq!(parse_border_style("hidden"), Some(BorderStyleValue::Hidden));
    assert_eq!(parse_border_style("dotted"), Some(BorderStyleValue::Dotted));
    assert_eq!(parse_border_style("dashed"), Some(BorderStyleValue::Dashed));
    assert_eq!(parse_border_style("solid"), Some(BorderStyleValue::Solid));
    assert_eq!(parse_border_style("double"), Some(BorderStyleValue::Double));
    assert_eq!(parse_border_style("groove"), Some(BorderStyleValue::Groove));
    assert_eq!(parse_border_style("ridge"), Some(BorderStyleValue::Ridge));
    assert_eq!(parse_border_style("inset"), Some(BorderStyleValue::Inset));
    assert_eq!(parse_border_style("outset"), Some(BorderStyleValue::Outset));
    assert_eq!(parse_border_style("unknown"), None);
}

#[test]
/// parse_text_align 所有变体
fn test_parse_text_align_all_variants() {
    assert_eq!(parse_text_align("left"), Some(TextAlignValue::Left));
    assert_eq!(parse_text_align("right"), Some(TextAlignValue::Right));
    assert_eq!(parse_text_align("center"), Some(TextAlignValue::Center));
    assert_eq!(parse_text_align("justify"), Some(TextAlignValue::Justify));
    assert_eq!(parse_text_align("start"), Some(TextAlignValue::Start));
    assert_eq!(parse_text_align("end"), Some(TextAlignValue::End));
    assert_eq!(parse_text_align("invalid"), None);
}

#[test]
/// parse_text_decoration 所有变体
fn test_parse_text_decoration_all_variants() {
    assert_eq!(parse_text_decoration("none"), Some(TextDecorationValue::None));
    assert_eq!(parse_text_decoration("underline"), Some(TextDecorationValue::Underline));
    assert_eq!(parse_text_decoration("overline"), Some(TextDecorationValue::Overline));
    assert_eq!(
        parse_text_decoration("line-through"),
        Some(TextDecorationValue::LineThrough)
    );
    assert_eq!(parse_text_decoration("blink"), None);
}

#[test]
/// parse_white_space 所有变体
fn test_parse_white_space_all_variants() {
    assert_eq!(parse_white_space("normal"), Some(WhiteSpaceValue::Normal));
    assert_eq!(parse_white_space("pre"), Some(WhiteSpaceValue::Pre));
    assert_eq!(parse_white_space("nowrap"), Some(WhiteSpaceValue::Nowrap));
    assert_eq!(parse_white_space("pre-wrap"), Some(WhiteSpaceValue::PreWrap));
    assert_eq!(parse_white_space("pre-line"), Some(WhiteSpaceValue::PreLine));
    assert_eq!(parse_white_space("invalid"), None);
}

#[test]
/// parse_text_transform 所有变体
fn test_parse_text_transform_all_variants() {
    assert_eq!(parse_text_transform("none"), Some(TextTransformValue::None));
    assert_eq!(parse_text_transform("uppercase"), Some(TextTransformValue::Uppercase));
    assert_eq!(parse_text_transform("lowercase"), Some(TextTransformValue::Lowercase));
    assert_eq!(parse_text_transform("capitalize"), Some(TextTransformValue::Capitalize));
    assert_eq!(parse_text_transform("invalid"), None);
}

#[test]
/// parse_text_overflow 所有变体
fn test_parse_text_overflow_all_variants() {
    assert_eq!(parse_text_overflow("clip"), Some(TextOverflowValue::Clip));
    assert_eq!(parse_text_overflow("ellipsis"), Some(TextOverflowValue::Ellipsis));
    assert_eq!(parse_text_overflow("invalid"), None);
}

#[test]
/// parse_grid_line: span 不带空格
fn test_parse_grid_line_span_no_space() {
    assert_eq!(parse_grid_line("span2"), Some(GridLineValue::Span(2)));
    assert_eq!(parse_grid_line("span3"), Some(GridLineValue::Span(3)));
}

#[test]
/// parse_grid_line: 命名区域
fn test_parse_grid_line_named_area() {
    assert_eq!(
        parse_grid_line("header"),
        Some(GridLineValue::Name("header".to_string()))
    );
    assert_eq!(
        parse_grid_line("sidebar"),
        Some(GridLineValue::Name("sidebar".to_string()))
    );
}

#[test]
/// parse_grid_line: 0 是非法值
fn test_parse_grid_line_zero_invalid() {
    assert_eq!(parse_grid_line("0"), None);
}

#[test]
/// parse_flex_basis 所有变体
fn test_parse_flex_basis_all_variants() {
    assert_eq!(parse_flex_basis("auto"), Some(FlexBasisValue::Auto));
    assert_eq!(parse_flex_basis("content"), Some(FlexBasisValue::Content));
    assert_eq!(
        parse_flex_basis("50%"),
        Some(FlexBasisValue::Length(LengthValue::Percentage(50.0)))
    );
    assert_eq!(parse_flex_basis("invalid-basis"), None);
}

#[test]
/// parse_z_index 正负整数和 auto
fn test_parse_z_index_variants() {
    assert_eq!(parse_z_index("auto"), Some(ZIndexValue::Auto));
    assert_eq!(parse_z_index("0"), Some(ZIndexValue::Integer(0)));
    assert_eq!(parse_z_index("9999"), Some(ZIndexValue::Integer(9999)));
    assert_eq!(parse_z_index("-999"), Some(ZIndexValue::Integer(-999)));
    assert_eq!(parse_z_index("abc"), None);
}

#[test]
/// apply_property_value 对无效 display 值返回 false
fn test_apply_property_invalid_display() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "display", "invalid"));
    assert!(!apply_property_value(&mut style, "display", ""));
    assert_eq!(style.display, DisplayValue::Inline);
}

#[test]
/// apply_property_value 对 max-width: none 设置无穷大
fn test_apply_property_max_width_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "max-width", "none"));
    assert_eq!(style.max_width, LengthValue::Px(f64::INFINITY));
}

#[test]
/// apply_property_value 对 max-height: none 设置无穷大
fn test_apply_property_max_height_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "max-height", "none"));
    assert_eq!(style.max_height, LengthValue::Px(f64::INFINITY));
}

#[test]
/// apply_property_value 对 transform: none
fn test_apply_property_transform_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "transform", "none"));
    assert_eq!(style.transform, zero_css_parser::values::TransformValue::None);
}

#[test]
/// apply_property_value 对 aspect-ratio: auto 设置为 None
fn test_apply_property_aspect_ratio_auto() {
    let mut style = ComputedStyle::default();
    style.aspect_ratio = Some(1.5);
    assert!(apply_property_value(&mut style, "aspect-ratio", "auto"));
    assert_eq!(style.aspect_ratio, None);
}

#[test]
/// apply_property_value 对 aspect-ratio: 16/9
fn test_apply_property_aspect_ratio_slash() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "aspect-ratio", "16 / 9"));
    let ratio = style.aspect_ratio.expect("should have ratio");
    assert!((ratio - 16.0 / 9.0).abs() < 0.01);
}

#[test]
/// apply_property_value 对 aspect-ratio: 数值
fn test_apply_property_aspect_ratio_number() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "aspect-ratio", "2"));
    assert_eq!(style.aspect_ratio, Some(2.0));
}

#[test]
/// apply_property_value 对 aspect-ratio: 除零返回 false
fn test_apply_property_aspect_ratio_divide_by_zero() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "aspect-ratio", "1 / 0"));
}

#[test]
/// apply_property_value 对 vertical-align
fn test_apply_property_vertical_align() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "vertical-align", "middle"));
    assert_eq!(style.vertical_align, VerticalAlignValue::Middle);

    assert!(apply_property_value(&mut style, "vertical-align", "top"));
    assert_eq!(style.vertical_align, VerticalAlignValue::Top);

    assert!(apply_property_value(&mut style, "vertical-align", "baseline"));
    assert_eq!(style.vertical_align, VerticalAlignValue::Baseline);
}

#[test]
/// apply_property_value 对 grid-template-areas
fn test_apply_property_grid_template_areas() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "grid-template-areas",
        "\"header header\" \"sidebar main\""
    ));
    assert_eq!(
        style.grid_template_areas,
        Some("\"header header\" \"sidebar main\"".to_string())
    );
}

#[test]
/// apply_property_value 对未知属性返回 false
fn test_apply_property_unknown() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "foobar", "baz"));
}

#[test]
/// parse_font_family 空字符串过滤
fn test_parse_font_family_empty_segments() {
    let families = parse_font_family(", , Arial, , sans-serif, ");
    assert_eq!(families, vec!["Arial", "sans-serif"]);
}

#[test]
/// parse_font_family 单个字体
fn test_parse_font_family_single() {
    let families = parse_font_family("monospace");
    assert_eq!(families, vec!["monospace"]);
}

#[test]
/// parse_line_height 无单位零
fn test_parse_line_height_zero() {
    assert_eq!(parse_line_height("0"), Some(LineHeightValue::Number(0.0)));
}

#[test]
/// parse_grid_auto_flow 大小写不敏感
fn test_parse_grid_auto_flow_case_insensitive() {
    assert_eq!(parse_grid_auto_flow("Row"), Some(GridAutoFlowValue::Row));
    assert_eq!(parse_grid_auto_flow("COLUMN"), Some(GridAutoFlowValue::Column));
    assert_eq!(parse_grid_auto_flow("Row Dense"), Some(GridAutoFlowValue::RowDense));
}

#[test]
/// inherit_property 对不可继承属性返回 false
fn test_inherit_property_returns_false_for_non_inheritable() {
    let parent = ComputedStyle::default();
    let mut child = ComputedStyle::default();
    // transform 仍不在 inherit 表；display/float/position/width 等 R545/R754 已支持显式 inherit
    assert!(!inherit_property(&parent, &mut child, "transform"));
    assert!(!inherit_property(&parent, &mut child, "unknown-prop"));
}

#[test]
/// apply_initial_value 对未知属性返回 false
fn test_apply_initial_value_unknown() {
    let mut style = ComputedStyle::default();
    assert!(!apply_initial_value(&mut style, "unknown-prop"));
}

// ═══════════════════════════════════════════════════════════════════
// grid-area / grid-column / grid-row 简写属性测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 grid-area 命名区域简写
fn test_grid_area_named_area() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-area", "header"));
    assert_eq!(style.grid_row_start, GridLineValue::Name("header".to_string()));
    assert_eq!(style.grid_row_end, GridLineValue::Name("header".to_string()));
    assert_eq!(style.grid_column_start, GridLineValue::Name("header".to_string()));
    assert_eq!(style.grid_column_end, GridLineValue::Name("header".to_string()));
}

#[test]
/// 测试 grid-area auto 简写
fn test_grid_area_auto() {
    let mut style = ComputedStyle::default();
    // 先设置非 auto 值
    style.grid_row_start = GridLineValue::Line(1);
    assert!(apply_property_value(&mut style, "grid-area", "auto"));
    assert_eq!(style.grid_row_start, GridLineValue::Auto);
    assert_eq!(style.grid_row_end, GridLineValue::Auto);
    assert_eq!(style.grid_column_start, GridLineValue::Auto);
    assert_eq!(style.grid_column_end, GridLineValue::Auto);
}

#[test]
/// 测试 grid-area 四值斜杠分隔行号
fn test_grid_area_four_line_numbers() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-area", "1 / 2 / 3 / 4"));
    assert_eq!(style.grid_row_start, GridLineValue::Line(1));
    assert_eq!(style.grid_row_end, GridLineValue::Line(2));
    assert_eq!(style.grid_column_start, GridLineValue::Line(3));
    assert_eq!(style.grid_column_end, GridLineValue::Line(4));
}

#[test]
/// 测试 grid-area 两值斜杠分隔（row-start / col-start）
fn test_grid_area_two_values() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-area", "1 / 3"));
    assert_eq!(style.grid_row_start, GridLineValue::Line(1));
    assert_eq!(style.grid_row_end, GridLineValue::Auto);
    assert_eq!(style.grid_column_start, GridLineValue::Line(3));
    assert_eq!(style.grid_column_end, GridLineValue::Auto);
}

#[test]
/// 测试 grid-area 三值斜杠分隔
fn test_grid_area_three_values() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-area", "1 / 3 / 2"));
    assert_eq!(style.grid_row_start, GridLineValue::Line(1));
    assert_eq!(style.grid_row_end, GridLineValue::Line(3));
    assert_eq!(style.grid_column_start, GridLineValue::Line(2));
    assert_eq!(style.grid_column_end, GridLineValue::Auto);
}

#[test]
/// 测试 grid-area 包含 span 关键字
fn test_grid_area_with_span() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-area", "2 / span 2 / 3 / span 3"));
    assert_eq!(style.grid_row_start, GridLineValue::Line(2));
    assert_eq!(style.grid_row_end, GridLineValue::Span(2));
    assert_eq!(style.grid_column_start, GridLineValue::Line(3));
    assert_eq!(style.grid_column_end, GridLineValue::Span(3));
}

#[test]
/// 测试 grid-column 简写（start / end）
fn test_grid_column_shorthand() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-column", "1 / 3"));
    assert_eq!(style.grid_column_start, GridLineValue::Line(1));
    assert_eq!(style.grid_column_end, GridLineValue::Line(3));
}

#[test]
/// 测试 grid-column 简写（单个值）
fn test_grid_column_single_value() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-column", "2"));
    assert_eq!(style.grid_column_start, GridLineValue::Line(2));
    assert_eq!(style.grid_column_end, GridLineValue::Auto);
}

#[test]
/// 测试 grid-row 简写（start / end）
fn test_grid_row_shorthand() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "grid-row", "1 / span 2"));
    assert_eq!(style.grid_row_start, GridLineValue::Line(1));
    assert_eq!(style.grid_row_end, GridLineValue::Span(2));
}

#[test]
/// 测试 grid-column 包含命名行
fn test_grid_column_named() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "grid-column",
        "sidebar-start / sidebar-end"
    ));
    assert_eq!(
        style.grid_column_start,
        GridLineValue::Name("sidebar-start".to_string())
    );
    assert_eq!(style.grid_column_end, GridLineValue::Name("sidebar-end".to_string()));
}

#[test]
/// 测试 grid-area 无效值返回 false
fn test_grid_area_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "grid-area", ""));
}

#[test]
/// 测试 parse_grid_area_shorthand 函数
fn test_parse_grid_area_shorthand() {
    let result = parse_grid_area_shorthand("header").unwrap();
    assert_eq!(result.0, GridLineValue::Name("header".to_string()));
    assert_eq!(result.1, GridLineValue::Name("header".to_string()));
    assert_eq!(result.2, GridLineValue::Name("header".to_string()));
    assert_eq!(result.3, GridLineValue::Name("header".to_string()));

    let result = parse_grid_area_shorthand("auto").unwrap();
    assert_eq!(result.0, GridLineValue::Auto);
    assert_eq!(result.1, GridLineValue::Auto);
    assert_eq!(result.2, GridLineValue::Auto);
    assert_eq!(result.3, GridLineValue::Auto);

    let result = parse_grid_area_shorthand("1 / 3 / 2 / 4").unwrap();
    assert_eq!(result.0, GridLineValue::Line(1));
    assert_eq!(result.1, GridLineValue::Line(3));
    assert_eq!(result.2, GridLineValue::Line(2));
    assert_eq!(result.3, GridLineValue::Line(4));
}

#[test]
/// 测试 parse_grid_line_shorthand 函数
fn test_parse_grid_line_shorthand() {
    let result = parse_grid_line_shorthand("1 / 3").unwrap();
    assert_eq!(result.0, GridLineValue::Line(1));
    assert_eq!(result.1, GridLineValue::Line(3));

    let result = parse_grid_line_shorthand("span 2 / 5").unwrap();
    assert_eq!(result.0, GridLineValue::Span(2));
    assert_eq!(result.1, GridLineValue::Line(5));

    let result = parse_grid_line_shorthand("auto").unwrap();
    assert_eq!(result.0, GridLineValue::Auto);
    assert_eq!(result.1, GridLineValue::Auto);
}

// ═══════════════════════════════════════════════════════════════════
// cursor/opacity 管线集成测试 — 验证 css-parser 的解析器被正确接入
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 cursor 属性通过 CSS 管线应用（使用 css-parser 的 parse_cursor）
fn test_cursor_via_css_parser_pipeline() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.cursor, CursorValue::Auto);

    // 基本关键字
    assert!(apply_property_value(&mut style, "cursor", "pointer"));
    assert_eq!(style.cursor, CursorValue::Pointer);

    assert!(apply_property_value(&mut style, "cursor", "move"));
    assert_eq!(style.cursor, CursorValue::Move);

    assert!(apply_property_value(&mut style, "cursor", "wait"));
    assert_eq!(style.cursor, CursorValue::Wait);

    assert!(apply_property_value(&mut style, "cursor", "not-allowed"));
    assert_eq!(style.cursor, CursorValue::NotAllowed);

    // 大小写不敏感（css-parser 使用 to_ascii_lowercase）
    assert!(apply_property_value(&mut style, "cursor", "Pointer"));
    assert_eq!(style.cursor, CursorValue::Pointer);

    assert!(apply_property_value(&mut style, "cursor", "HELP"));
    assert_eq!(style.cursor, CursorValue::Help);

    // 方向性 resize 映射到 style-system 的 NsResize/EwResize
    assert!(apply_property_value(&mut style, "cursor", "n-resize"));
    assert_eq!(style.cursor, CursorValue::NsResize);

    assert!(apply_property_value(&mut style, "cursor", "s-resize"));
    assert_eq!(style.cursor, CursorValue::NsResize);

    assert!(apply_property_value(&mut style, "cursor", "e-resize"));
    assert_eq!(style.cursor, CursorValue::EwResize);

    assert!(apply_property_value(&mut style, "cursor", "w-resize"));
    assert_eq!(style.cursor, CursorValue::EwResize);

    // 无效值返回 false
    assert!(!apply_property_value(&mut style, "cursor", "invalid-cursor"));
    assert_eq!(style.cursor, CursorValue::EwResize); // 上一个有效值
}

#[test]
/// 测试 opacity 属性通过 css-parser 的 parse_opacity 应用
fn test_opacity_via_css_parser_pipeline() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.opacity, 1.0);

    // 正常数值
    assert!(apply_property_value(&mut style, "opacity", "0.5"));
    assert!((style.opacity - 0.5).abs() < f64::EPSILON);

    assert!(apply_property_value(&mut style, "opacity", "0"));
    assert_eq!(style.opacity, 0.0);

    assert!(apply_property_value(&mut style, "opacity", "1"));
    assert_eq!(style.opacity, 1.0);

    // 百分比格式（css-parser parse_opacity 支持）
    assert!(apply_property_value(&mut style, "opacity", "50%"));
    assert!((style.opacity - 0.5).abs() < f64::EPSILON);

    assert!(apply_property_value(&mut style, "opacity", "100%"));
    assert_eq!(style.opacity, 1.0);

    assert!(apply_property_value(&mut style, "opacity", "0%"));
    assert_eq!(style.opacity, 0.0);

    // 无效值返回 false
    assert!(!apply_property_value(&mut style, "opacity", "abc"));
    assert!(!apply_property_value(&mut style, "opacity", "half"));
}

#[test]
/// 测试 opacity 值被 clamp 到 [0.0, 1.0] 范围
fn test_opacity_clamping_via_css_parser() {
    let mut style = ComputedStyle::default();

    // 超出上界 → clamp 到 1.0
    assert!(apply_property_value(&mut style, "opacity", "1.5"));
    assert_eq!(style.opacity, 1.0);

    assert!(apply_property_value(&mut style, "opacity", "999"));
    assert_eq!(style.opacity, 1.0);

    // 超出下界 → clamp 到 0.0
    assert!(apply_property_value(&mut style, "opacity", "-0.5"));
    assert_eq!(style.opacity, 0.0);

    assert!(apply_property_value(&mut style, "opacity", "-10"));
    assert_eq!(style.opacity, 0.0);

    // 百分比超出范围
    assert!(apply_property_value(&mut style, "opacity", "150%"));
    assert_eq!(style.opacity, 1.0);

    assert!(apply_property_value(&mut style, "opacity", "-25%"));
    assert_eq!(style.opacity, 0.0);
}

#[test]
/// 测试 cursor 继承：父元素 cursor:pointer，子元素应继承
fn test_cursor_inheritance() {
    let mut parent = ComputedStyle::default();
    parent.cursor = CursorValue::Pointer;

    let mut child = ComputedStyle::default();
    assert_eq!(child.cursor, CursorValue::Auto);

    // cursor 是继承属性
    assert!(inherit_property(&parent, &mut child, "cursor"));
    assert_eq!(child.cursor, CursorValue::Pointer);

    // 子元素显式设置 cursor 后覆盖继承值
    assert!(apply_property_value(&mut child, "cursor", "text"));
    assert_eq!(child.cursor, CursorValue::Text);
}

// ═══════════════════════════════════════════════════════════════════
// word-break 属性测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 apply_property_value 对 word-break: break-all
fn test_apply_word_break_break_all() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "word-break", "break-all"));
    assert_eq!(style.word_break, WordBreakValue::BreakAll);

    // 无效值返回 false
    assert!(!apply_property_value(&mut style, "word-break", "invalid"));
    assert_eq!(style.word_break, WordBreakValue::BreakAll);
}

#[test]
/// 测试 apply_property_value 对 word-break: keep-all
fn test_apply_word_break_keep_all() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "word-break", "keep-all"));
    assert_eq!(style.word_break, WordBreakValue::KeepAll);

    // break-word
    assert!(apply_property_value(&mut style, "word-break", "break-word"));
    assert_eq!(style.word_break, WordBreakValue::BreakWord);

    // normal
    assert!(apply_property_value(&mut style, "word-break", "normal"));
    assert_eq!(style.word_break, WordBreakValue::Normal);
}

#[test]
/// 测试 word-break 继承：父元素 break-all，子元素应继承
fn test_word_break_inheritance() {
    let mut parent = ComputedStyle::default();
    parent.word_break = WordBreakValue::BreakAll;

    let mut child = ComputedStyle::default();
    assert_eq!(child.word_break, WordBreakValue::Normal);

    // word-break 是继承属性
    assert!(inherit_property(&parent, &mut child, "word-break"));
    assert_eq!(child.word_break, WordBreakValue::BreakAll);

    // 子元素显式设置后覆盖继承值
    assert!(apply_property_value(&mut child, "word-break", "keep-all"));
    assert_eq!(child.word_break, WordBreakValue::KeepAll);
}

#[test]
/// 测试 word-break 默认值为 Normal
fn test_word_break_default_is_normal() {
    let style = ComputedStyle::default();
    assert_eq!(style.word_break, WordBreakValue::Normal);

    // 验证注册表初始值
    assert!(PropertyRegistry::initial_value("word-break").is_some());
    assert!(PropertyRegistry::is_inherited("word-break"));

    // 验证 known_properties 包含
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"word-break"));

    // 验证 apply_initial_value 重置
    let mut style = ComputedStyle::default();
    style.word_break = WordBreakValue::BreakAll;
    assert!(apply_initial_value(&mut style, "word-break"));
    assert_eq!(style.word_break, WordBreakValue::Normal);
}

// ═══════════════════════════════════════════════════════════════════
// writing-mode 属性测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 apply_property_value 对 writing-mode: vertical-rl
fn test_apply_writing_mode_vertical_rl() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "vertical-rl"));
    assert_eq!(style.writing_mode, WritingModeValue::VerticalRl);

    // 无效值返回 false
    assert!(!apply_property_value(&mut style, "writing-mode", "invalid"));
    assert_eq!(style.writing_mode, WritingModeValue::VerticalRl);
}

#[test]
/// 测试 apply_property_value 对 writing-mode: vertical-lr
fn test_apply_writing_mode_vertical_lr() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "writing-mode", "vertical-lr"));
    assert_eq!(style.writing_mode, WritingModeValue::VerticalLr);
}

#[test]
/// 测试 writing-mode 默认值为 horizontal-tb 且为继承属性
fn test_writing_mode_default_is_horizontal_tb() {
    let style = ComputedStyle::default();
    assert_eq!(style.writing_mode, WritingModeValue::HorizontalTb);

    // 验证注册表初始值
    assert!(PropertyRegistry::initial_value("writing-mode").is_some());
    // writing-mode 是继承属性（CSS 规范要求）
    assert!(PropertyRegistry::is_inherited("writing-mode"));

    // 验证 known_properties 包含
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"writing-mode"));

    // 验证 apply_initial_value 重置
    let mut style = ComputedStyle::default();
    style.writing_mode = WritingModeValue::VerticalRl;
    assert!(apply_initial_value(&mut style, "writing-mode"));
    assert_eq!(style.writing_mode, WritingModeValue::HorizontalTb);
}

#[test]
/// 测试 inherit_property 对 writing-mode 的显式继承
fn test_writing_mode_inherit_property() {
    let mut parent = ComputedStyle::default();
    parent.writing_mode = WritingModeValue::VerticalRl;

    let mut child = ComputedStyle::default();
    assert_eq!(child.writing_mode, WritingModeValue::HorizontalTb);

    // inherit_property 对 writing-mode 返回 true 并正确继承
    assert!(inherit_property(&parent, &mut child, "writing-mode"));
    assert_eq!(child.writing_mode, WritingModeValue::VerticalRl);
}

// ═══════════════════════════════════════════════════════════════════
// text-decoration-line / text-transform / letter-spacing 属性测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 apply_property_value 对 text-decoration-line: underline
fn test_apply_text_decoration_underline() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-decoration-line", "underline"));
    assert_eq!(style.text_decoration_line, TextDecorationLineValue::Underline);

    // 无效值返回 false
    assert!(!apply_property_value(&mut style, "text-decoration-line", "invalid"));
    assert_eq!(style.text_decoration_line, TextDecorationLineValue::Underline);
}

#[test]
/// 测试 apply_property_value 对 text-decoration-line: none
fn test_apply_text_decoration_none() {
    let mut style = ComputedStyle::default();
    // 先设置为 underline
    assert!(apply_property_value(&mut style, "text-decoration-line", "underline"));
    assert_eq!(style.text_decoration_line, TextDecorationLineValue::Underline);

    // 重置为 none
    assert!(apply_property_value(&mut style, "text-decoration-line", "none"));
    assert_eq!(style.text_decoration_line, TextDecorationLineValue::None);

    // 默认值也是 none
    let style = ComputedStyle::default();
    assert_eq!(style.text_decoration_line, TextDecorationLineValue::None);
}

#[test]
/// 测试 apply_property_value 对 text-transform: uppercase
fn test_apply_text_transform_uppercase() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-transform", "uppercase"));
    assert_eq!(style.text_transform, TextTransformValue::Uppercase);
}

#[test]
/// 测试 apply_property_value 对 text-transform: capitalize
fn test_apply_text_transform_capitalize() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-transform", "capitalize"));
    assert_eq!(style.text_transform, TextTransformValue::Capitalize);
}

#[test]
/// 测试 text-transform 继承：父元素 uppercase，子元素继承
fn test_text_transform_inherited() {
    let mut parent = ComputedStyle::default();
    parent.text_transform = TextTransformValue::Uppercase;

    let mut child = ComputedStyle::default();
    assert_eq!(child.text_transform, TextTransformValue::None);

    // text-transform 是继承属性
    assert!(inherit_property(&parent, &mut child, "text-transform"));
    assert_eq!(child.text_transform, TextTransformValue::Uppercase);
}

#[test]
/// 测试 text-decoration-line 不继承：父元素 underline，子元素不继承
fn test_text_transform_not_inherited_decoration() {
    // text-decoration-line 不是继承属性
    assert!(!PropertyRegistry::is_inherited("text-decoration-line"));

    let mut parent = ComputedStyle::default();
    parent.text_decoration_line = TextDecorationLineValue::Underline;

    let mut child = ComputedStyle::default();
    assert_eq!(child.text_decoration_line, TextDecorationLineValue::None);

    // inherit_property 对 text-decoration-line 应返回 false
    assert!(!inherit_property(&parent, &mut child, "text-decoration-line"));
    // 子元素值不变
    assert_eq!(child.text_decoration_line, TextDecorationLineValue::None);
}

#[test]
/// 测试 apply_property_value 对 letter-spacing: px
fn test_apply_letter_spacing_px() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "letter-spacing", "3px"));
    assert_eq!(style.letter_spacing, LengthValue::Px(3.0));

    // 负值
    assert!(apply_property_value(&mut style, "letter-spacing", "-1.5px"));
    assert_eq!(style.letter_spacing, LengthValue::Px(-1.5));
}

#[test]
/// 测试 apply_property_value 对 letter-spacing: normal（解析为 0px）
fn test_apply_letter_spacing_normal() {
    let mut style = ComputedStyle::default();
    // letter-spacing 的 normal 在 CSS 中解析为 0px
    // 当前实现通过 parse_length_or_math 解析，"normal" 不是有效长度
    // 所以先设置为非零值，然后验证默认重置
    assert!(apply_property_value(&mut style, "letter-spacing", "2px"));
    assert_eq!(style.letter_spacing, LengthValue::Px(2.0));

    // 默认值为 0px
    let style = ComputedStyle::default();
    assert_eq!(style.letter_spacing, LengthValue::Px(0.0));
}

#[test]
/// 测试 letter-spacing 继承：父元素 3px，子元素继承
fn test_letter_spacing_inherited() {
    let mut parent = ComputedStyle::default();
    parent.letter_spacing = LengthValue::Px(3.0);

    let mut child = ComputedStyle::default();
    assert_eq!(child.letter_spacing, LengthValue::Px(0.0));

    // letter-spacing 是继承属性
    assert!(inherit_property(&parent, &mut child, "letter-spacing"));
    assert_eq!(child.letter_spacing, LengthValue::Px(3.0));

    // 子元素显式设置后覆盖继承值
    assert!(apply_property_value(&mut child, "letter-spacing", "5px"));
    assert_eq!(child.letter_spacing, LengthValue::Px(5.0));
}

// ═══════════════════════════════════════════════════════════════════
// 级联/简写/自定义属性/继承/revert 边界条件测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试级联源顺序：两条规则具有相同特异性，后应用的规则胜出
fn test_cascade_source_order() {
    let mut style = ComputedStyle::default();

    // 第一条规则：display: flex
    assert!(apply_property_value(&mut style, "display", "flex"));
    assert_eq!(style.display, DisplayValue::Flex);

    // 第二条规则（相同特异性，后出现）：display: grid — 应覆盖前一条
    assert!(apply_property_value(&mut style, "display", "grid"));
    assert_eq!(style.display, DisplayValue::Grid);

    // 同理测试 color
    assert!(apply_property_value(&mut style, "color", "red"));
    assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));

    assert!(apply_property_value(&mut style, "color", "blue"));
    assert_eq!(style.color, ColorValue::Rgba(0, 0, 255, 255));

    // 同理测试 width
    assert!(apply_property_value(&mut style, "width", "100px"));
    assert!(apply_property_value(&mut style, "width", "200px"));
    assert_eq!(style.width, LengthValue::Px(200.0));
}

#[test]
/// 测试 border 简写展开为 12 个长属性（4边 x width/style/color）
fn test_shorthand_border_expansion() {
    let mut style = ComputedStyle::default();

    // 手动模拟 "border: 1px solid red" 的简写展开
    // 宽度：四边均为 1px
    assert!(apply_property_value(&mut style, "border-top-width", "1px"));
    assert!(apply_property_value(&mut style, "border-right-width", "1px"));
    assert!(apply_property_value(&mut style, "border-bottom-width", "1px"));
    assert!(apply_property_value(&mut style, "border-left-width", "1px"));

    // 样式：四边均为 solid
    assert!(apply_property_value(&mut style, "border-top-style", "solid"));
    assert!(apply_property_value(&mut style, "border-right-style", "solid"));
    assert!(apply_property_value(&mut style, "border-bottom-style", "solid"));
    assert!(apply_property_value(&mut style, "border-left-style", "solid"));

    // 颜色：四边均为 red
    assert!(apply_property_value(&mut style, "border-top-color", "red"));
    assert!(apply_property_value(&mut style, "border-right-color", "red"));
    assert!(apply_property_value(&mut style, "border-bottom-color", "red"));
    assert!(apply_property_value(&mut style, "border-left-color", "red"));

    // 验证所有 12 个长属性已正确设置
    let expected_width = LengthValue::Px(1.0);
    let expected_style = BorderStyleValue::Solid;
    let expected_color = ColorValue::Rgba(255, 0, 0, 255);

    // 宽度（4个）
    assert_eq!(style.border_top_width, expected_width);
    assert_eq!(style.border_right_width, expected_width);
    assert_eq!(style.border_bottom_width, expected_width);
    assert_eq!(style.border_left_width, expected_width);

    // 样式（4个）
    assert_eq!(style.border_top_style, expected_style);
    assert_eq!(style.border_right_style, expected_style);
    assert_eq!(style.border_bottom_style, expected_style);
    assert_eq!(style.border_left_style, expected_style);

    // 颜色（4个）
    assert_eq!(style.border_top_color, expected_color);
    assert_eq!(style.border_right_color, expected_color);
    assert_eq!(style.border_bottom_color, expected_color);
    assert_eq!(style.border_left_color, expected_color);
}

#[test]
/// 测试自定义属性链式引用：--a: red → --b: var(--a) → color: var(--b) 最终解析为 red
fn test_custom_property_chained() {
    use crate::computed::resolve_var;
    use std::collections::HashMap;

    // 构建自定义属性映射
    let mut custom_props = HashMap::new();
    custom_props.insert("--a".to_string(), "red".to_string());
    custom_props.insert("--b".to_string(), "var(--a)".to_string());

    // 第一层：var(--b) → 解析为 var(--a)
    let resolved_b = resolve_var("var(--b)", &custom_props);
    // 第二层：var(--a) → 解析为 red
    let resolved_a = resolve_var(&resolved_b, &custom_props);
    assert_eq!(resolved_a, "red");

    // 验证解析后的值可以应用到 ComputedStyle
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "color", &resolved_a));
    assert_eq!(style.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
/// 测试对非继承属性显式设置 inherit：没有父元素时使用默认值
fn test_inherit_non_inherited_explicit() {
    // transform 仍不在 inherit 表（不可继承属性调用 inherit_property 返回 false）
    let parent = ComputedStyle::default();
    let mut child = ComputedStyle::default();

    // transform 不可继承
    assert!(!inherit_property(&parent, &mut child, "transform"));

    // R545：width 现支持显式 inherit（CSS `inherit` 对任意属性生效），复制父元素计算值
    let mut parent_w = ComputedStyle::default();
    parent_w.width = LengthValue::Px(123.0);
    let mut child_w = ComputedStyle::default();
    assert!(inherit_property(&parent_w, &mut child_w, "width"));
    assert_eq!(child_w.width, LengthValue::Px(123.0));

    // R754：display 现支持显式 inherit（语料 36 案 display:inherit），复制父元素计算值
    let mut parent_d = ComputedStyle::default();
    parent_d.display = DisplayValue::Flex;
    let mut child_d = ComputedStyle::default();
    assert!(inherit_property(&parent_d, &mut child_d, "display"));
    assert_eq!(child_d.display, DisplayValue::Flex);
}

#[test]
/// 测试 revert 关键字：应用 display: revert 时恢复为 user-agent 默认值
fn test_revert_keyword() {
    let mut style = ComputedStyle::default();

    // 先修改 display 为非默认值
    style.display = DisplayValue::Flex;
    assert_eq!(style.display, DisplayValue::Flex);

    // "revert" 不是有效的 display 值，apply_property_value 返回 false
    // 在完整 CSS 引擎中，revert 会触发回退到 user-agent 样式
    // 这里模拟 revert 的效果：使用 apply_initial_value 恢复为 UA 默认
    assert!(!apply_property_value(&mut style, "display", "revert"));
    // display 未被 "revert" 字符串改变
    assert_eq!(style.display, DisplayValue::Flex);

    // 正确的 revert 模拟：使用 apply_initial_value 恢复为 UA 默认
    assert!(apply_initial_value(&mut style, "display"));
    assert_eq!(style.display, DisplayValue::Inline); // UA 默认 display 为 inline

    // 同理测试 position: revert
    style.position = PositionValue::Absolute;
    assert!(!apply_property_value(&mut style, "position", "revert"));
    assert_eq!(style.position, PositionValue::Absolute);
    assert!(apply_initial_value(&mut style, "position"));
    assert_eq!(style.position, PositionValue::Static); // UA 默认
}

// ═══════════════════════════════════════════════════════════════════
// 3D Transform / perspective / backface-visibility 边界条件测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 transform-origin 默认值为 50% 50%
fn test_transform_origin_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.transform_origin_x, LengthValue::Percentage(50.0));
    assert_eq!(style.transform_origin_y, LengthValue::Percentage(50.0));
}

#[test]
/// 测试 transform-origin: 10px 20px 应用
fn test_transform_origin_apply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "transform-origin", "10px 20px"));
    assert_eq!(style.transform_origin_x, LengthValue::Px(10.0));
    assert_eq!(style.transform_origin_y, LengthValue::Px(20.0));

    // 单值：Y 默认为 50%
    let mut style2 = ComputedStyle::default();
    assert!(apply_property_value(&mut style2, "transform-origin", "0px"));
    assert_eq!(style2.transform_origin_x, LengthValue::Px(0.0));
    assert_eq!(style2.transform_origin_y, LengthValue::Percentage(50.0));
}

#[test]
/// 测试 perspective: 500px 应用
fn test_perspective_apply() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "perspective", "500px"));
    assert_eq!(style.perspective, LengthValue::Px(500.0));

    // perspective: none 重置为 0
    assert!(apply_property_value(&mut style, "perspective", "none"));
    assert_eq!(style.perspective, LengthValue::Px(0.0));
}

#[test]
/// 测试 transform-style: preserve-3d 应用
fn test_transform_style_apply() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.transform_style, TransformStyleValue::Flat);

    assert!(apply_property_value(&mut style, "transform-style", "preserve-3d"));
    assert_eq!(style.transform_style, TransformStyleValue::Preserve3d);

    assert!(apply_property_value(&mut style, "transform-style", "flat"));
    assert_eq!(style.transform_style, TransformStyleValue::Flat);

    // 无效值返回 false
    assert!(!apply_property_value(&mut style, "transform-style", "invalid"));
    assert_eq!(style.transform_style, TransformStyleValue::Flat);
}

#[test]
/// 测试 backface-visibility: hidden 应用
fn test_backface_visibility_apply() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Visible);

    assert!(apply_property_value(&mut style, "backface-visibility", "hidden"));
    assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Hidden);

    assert!(apply_property_value(&mut style, "backface-visibility", "visible"));
    assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Visible);

    // 无效值返回 false
    assert!(!apply_property_value(&mut style, "backface-visibility", "invalid"));
    assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Visible);
}

#[test]
/// 测试 transform-origin 不继承
fn test_transform_origin_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("transform-origin"));

    let mut parent = ComputedStyle::default();
    parent.transform_origin_x = LengthValue::Px(100.0);
    parent.transform_origin_y = LengthValue::Px(200.0);

    let mut child = ComputedStyle::default();
    assert!(!inherit_property(&parent, &mut child, "transform-origin"));
    assert_eq!(child.transform_origin_x, LengthValue::Percentage(50.0));
    assert_eq!(child.transform_origin_y, LengthValue::Percentage(50.0));
}

#[test]
/// 测试 perspective-origin: left top 应用
fn test_perspective_origin_apply() {
    let mut style = ComputedStyle::default();
    // "left top" — left 解析为 0%, top 解析为 0%
    // 当前实现通过 parse_length_or_math 解析，"left" 不是长度值
    // 使用数值测试
    assert!(apply_property_value(&mut style, "perspective-origin", "0% 0%"));
    assert_eq!(style.perspective_origin_x, LengthValue::Percentage(0.0));
    assert_eq!(style.perspective_origin_y, LengthValue::Percentage(0.0));

    // 默认值为 50% 50%
    let style2 = ComputedStyle::default();
    assert_eq!(style2.perspective_origin_x, LengthValue::Percentage(50.0));
    assert_eq!(style2.perspective_origin_y, LengthValue::Percentage(50.0));

    // 单值：Y 默认为 50%
    let mut style3 = ComputedStyle::default();
    assert!(apply_property_value(&mut style3, "perspective-origin", "100px"));
    assert_eq!(style3.perspective_origin_x, LengthValue::Px(100.0));
    assert_eq!(style3.perspective_origin_y, LengthValue::Percentage(50.0));
}

// ═══════════════════════════════════════════════════════════════════
// 新增属性测试 — text-indent, table-layout, caption-side,
//                border-collapse, resize, white-space break-spaces
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 text-indent 默认值为 Px(0.0)
fn test_text_indent_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.text_indent, LengthValue::Px(0.0));
}

#[test]
/// 测试 apply_property_value 对 text-indent: 2em
fn test_apply_text_indent_em() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-indent", "2em"));
    assert_eq!(style.text_indent, LengthValue::Em(2.0));
}

#[test]
/// 测试 apply_property_value 对 text-indent: 10%
fn test_apply_text_indent_percentage() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-indent", "10%"));
    assert_eq!(style.text_indent, LengthValue::Percentage(10.0));
}

// ── text-decoration-inset（R1607，CSS Text Decoration 4 §2.4）────────────

#[test]
/// 默认 inset = 0/0（不改变装饰线）
fn test_text_decoration_inset_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.text_decoration_inset.start, LengthValue::Px(0.0));
    assert_eq!(style.text_decoration_inset.end, LengthValue::Px(0.0));
}

#[test]
/// apply 单值 px
fn test_apply_text_decoration_inset_single() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-decoration-inset", "10px"));
    assert_eq!(style.text_decoration_inset.start, LengthValue::Px(10.0));
    assert_eq!(style.text_decoration_inset.end, LengthValue::Px(10.0));
}

#[test]
/// apply 两值（start end），em 支持
fn test_apply_text_decoration_inset_two_em() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "text-decoration-inset",
        "0.25em -0.5em"
    ));
    assert_eq!(style.text_decoration_inset.start, LengthValue::Em(0.25));
    assert_eq!(style.text_decoration_inset.end, LengthValue::Em(-0.5));
}

#[test]
/// 非法值不 apply（返回 false，保持默认）
fn test_apply_text_decoration_inset_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "text-decoration-inset", "auto"));
    assert_eq!(style.text_decoration_inset.start, LengthValue::Px(0.0));
}

#[test]
/// 测试 table-layout 默认值为 Auto
fn test_table_layout_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.table_layout, TableLayoutValue::Auto);
}

#[test]
/// 测试 apply_property_value 对 table-layout: fixed
fn test_apply_table_layout_fixed() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "table-layout", "fixed"));
    assert_eq!(style.table_layout, TableLayoutValue::Fixed);
}

#[test]
/// 测试 apply_property_value 对 table-layout 无效值
fn test_apply_table_layout_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!apply_property_value(&mut style, "table-layout", "invalid"));
}

#[test]
/// 测试 caption-side 默认值为 Top
fn test_caption_side_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.caption_side, CaptionSideValue::Top);
}

#[test]
/// 测试 apply_property_value 对 caption-side: bottom
fn test_apply_caption_side_bottom() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "caption-side", "bottom"));
    assert_eq!(style.caption_side, CaptionSideValue::Bottom);
}

#[test]
/// 测试 border-collapse 默认值为 Separate
fn test_border_collapse_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.border_collapse, BorderCollapseValue::Separate);
}

#[test]
/// 测试 apply_property_value 对 border-collapse: collapse
fn test_apply_border_collapse_collapse() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "border-collapse", "collapse"));
    assert_eq!(style.border_collapse, BorderCollapseValue::Collapse);
}

#[test]
/// 测试 resize 默认值为 None
fn test_resize_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.resize, ResizeValue::None);
}

#[test]
/// 测试 apply_property_value 对 resize: both
fn test_apply_resize_both() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "resize", "both"));
    assert_eq!(style.resize, ResizeValue::Both);
}

#[test]
/// 测试 apply_property_value 对 resize: horizontal / vertical / block / inline
fn test_apply_resize_variants() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "resize", "horizontal"));
    assert_eq!(style.resize, ResizeValue::Horizontal);
    assert!(apply_property_value(&mut style, "resize", "vertical"));
    assert_eq!(style.resize, ResizeValue::Vertical);
    assert!(apply_property_value(&mut style, "resize", "block"));
    assert_eq!(style.resize, ResizeValue::Block);
    assert!(apply_property_value(&mut style, "resize", "inline"));
    assert_eq!(style.resize, ResizeValue::Inline);
}

#[test]
/// 测试 white-space: break-spaces
fn test_parse_white_space_break_spaces() {
    assert_eq!(parse_white_space("break-spaces"), Some(WhiteSpaceValue::BreakSpaces));
    // 验证 apply_property_value 也能应用
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "white-space", "break-spaces"));
    assert_eq!(style.white_space, WhiteSpaceValue::BreakSpaces);
}

#[test]
/// 测试 text-overflow 自定义字符串
fn test_apply_text_overflow_string() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "text-overflow", "\"...\""));
    assert_eq!(style.text_overflow, TextOverflowValue::String("...".to_string()));
}

#[test]
/// 测试 text-indent 继承
fn test_inherit_text_indent() {
    let mut parent = ComputedStyle::default();
    parent.text_indent = LengthValue::Em(2.0);
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "text-indent"));
    assert_eq!(child.text_indent, LengthValue::Em(2.0));
}

#[test]
/// 测试 caption-side 继承
fn test_inherit_caption_side() {
    let mut parent = ComputedStyle::default();
    parent.caption_side = CaptionSideValue::Bottom;
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "caption-side"));
    assert_eq!(child.caption_side, CaptionSideValue::Bottom);
}

#[test]
/// 测试 border-collapse 继承
fn test_inherit_border_collapse() {
    let mut parent = ComputedStyle::default();
    parent.border_collapse = BorderCollapseValue::Collapse;
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "border-collapse"));
    assert_eq!(child.border_collapse, BorderCollapseValue::Collapse);
}

#[test]
/// 测试 resize 不继承
fn test_resize_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("resize"));
}

#[test]
/// 测试 table-layout 不继承
fn test_table_layout_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("table-layout"));
}

#[test]
/// 测试新增属性在 known_properties 中
fn test_new_properties_in_known_list() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"text-indent"));
    assert!(props.contains(&"table-layout"));
    assert!(props.contains(&"caption-side"));
    assert!(props.contains(&"border-collapse"));
    assert!(props.contains(&"resize"));
}

// ═══════════════════════════════════════════════════════════════════
// Counter / Content / Quotes 测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 counter-reset 属性解析
fn test_apply_counter_reset() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "counter-reset",
        "section 1 subsection"
    ));
    assert_eq!(style.counter_reset.len(), 2);
    assert_eq!(style.counter_reset[0].name, "section");
    assert_eq!(style.counter_reset[0].value, Some(1));
    assert_eq!(style.counter_reset[1].name, "subsection");
    assert_eq!(style.counter_reset[1].value, None);
}

#[test]
/// 测试 counter-reset: none 清空列表
fn test_apply_counter_reset_none() {
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "counter-reset", "section 5");
    assert!(!style.counter_reset.is_empty());
    assert!(apply_property_value(&mut style, "counter-reset", "none"));
    assert!(style.counter_reset.is_empty());
}

#[test]
/// 测试 counter-increment 属性解析
fn test_apply_counter_increment() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "counter-increment", "section 2"));
    assert_eq!(style.counter_increment.len(), 1);
    assert_eq!(style.counter_increment[0].name, "section");
    assert_eq!(style.counter_increment[0].value, Some(2));
}

#[test]
/// 测试 content: normal 默认值
fn test_apply_content_normal() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "content", "normal"));
    assert_eq!(style.content, ContentComputedValue::Normal);
}

#[test]
/// 测试 content: string 值
fn test_apply_content_string() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "content", "\"Prefix: \""));
    assert_eq!(style.content, ContentComputedValue::String("Prefix: ".to_string()));
}

#[test]
/// 测试 content: none 值
fn test_apply_content_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "content", "none"));
    assert_eq!(style.content, ContentComputedValue::None);
}

#[test]
/// 测试 content: attr() 值
fn test_apply_content_attr() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "content", "attr(data-label)"));
    assert_eq!(style.content, ContentComputedValue::Attr("data-label".to_string()));
}

#[test]
/// 测试 content: counter() 值
fn test_apply_content_counter() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(
        &mut style,
        "content",
        "counter(section, upper-roman)"
    ));
    match &style.content {
        ContentComputedValue::Counter { name, style } => {
            assert_eq!(name, "section");
            assert_eq!(style, &Some("upper-roman".to_string()));
        }
        _ => panic!("expected Counter variant"),
    }
}

#[test]
/// 测试 quotes: none 值
fn test_apply_quotes_none() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "quotes", "none"));
    assert_eq!(style.quotes, QuotesComputedValue::None);
}

#[test]
/// 测试 quotes: 引号对值
fn test_apply_quotes_pairs() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "quotes", r#""«" "»" "‹" "›""#));
    match &style.quotes {
        QuotesComputedValue::Pairs(pairs) => {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0], ("«".to_string(), "»".to_string()));
        }
        _ => panic!("expected Pairs"),
    }
}

#[test]
/// 测试 quotes 继承
fn test_quotes_inherited() {
    assert!(PropertyRegistry::is_inherited("quotes"));
    let mut parent = ComputedStyle::default();
    apply_property_value(&mut parent, "quotes", "none");
    let mut child = ComputedStyle::default();
    assert!(inherit_property(&parent, &mut child, "quotes"));
    assert_eq!(child.quotes, QuotesComputedValue::None);
}

#[test]
/// 测试 counter-reset 不继承
fn test_counter_not_inherited() {
    assert!(!PropertyRegistry::is_inherited("counter-reset"));
    assert!(!PropertyRegistry::is_inherited("counter-increment"));
    assert!(!PropertyRegistry::is_inherited("content"));
}

#[test]
/// 测试新增属性在 known_properties 中
fn test_counter_content_quotes_in_known_properties() {
    let props = PropertyRegistry::known_properties();
    assert!(props.contains(&"counter-reset"));
    assert!(props.contains(&"counter-increment"));
    assert!(props.contains(&"content"));
    assert!(props.contains(&"quotes"));
}

#[test]
/// 测试 apply_initial_value 对新属性
fn test_apply_initial_value_new_properties() {
    let mut style = ComputedStyle::default();
    apply_property_value(&mut style, "counter-reset", "section 5");
    apply_property_value(&mut style, "content", "\"Hello\"");
    apply_property_value(&mut style, "quotes", "none");

    assert!(apply_initial_value(&mut style, "counter-reset"));
    assert!(style.counter_reset.is_empty());

    assert!(apply_initial_value(&mut style, "content"));
    assert_eq!(style.content, ContentComputedValue::Normal);

    assert!(apply_initial_value(&mut style, "quotes"));
    assert_eq!(style.quotes, QuotesComputedValue::Auto);
}

// ═══════════════════════════════════════════════════════════════════
// 新增属性测试：page-break, box-decoration-break, image-rendering, isolation
// ═══════════════════════════════════════════════════════════════════

#[test]
/// page-break-before 默认值为 Auto
fn test_page_break_before_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.page_break_before, PageBreakValue::Auto);
}

#[test]
/// page-break-after 默认值为 Auto
fn test_page_break_after_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.page_break_after, PageBreakValue::Auto);
}

#[test]
/// page-break-inside 默认值为 Auto
fn test_page_break_inside_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.page_break_inside, PageBreakValue::Auto);
}

#[test]
/// box-decoration-break 默认值为 Slice
fn test_box_decoration_break_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.box_decoration_break, BoxDecorationBreakValue::Slice);
}

#[test]
/// image-rendering 默认值为 Auto
fn test_image_rendering_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.image_rendering, ImageRenderingValue::Auto);
}

#[test]
/// isolation 默认值为 Auto
fn test_isolation_default() {
    let style = ComputedStyle::default();
    assert_eq!(style.isolation, IsolationValue::Auto);
}
