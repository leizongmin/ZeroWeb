// style-system property/parse.rs 覆盖率补充测试。
use super::super::parse::*;
use super::super::*;

#[test]
fn test_parse_grid_line_named_area_various() {
    // 命名区域
    assert!(matches!(parse_grid_line("sidebar"), Some(GridLineValue::Name(_))));
    // 数字开头不合法 → None
    assert!(parse_grid_line("2col").is_none());
    // 包含 / 不合法 → None
    assert!(parse_grid_line("a/b").is_none());
    // 空字符串 → None
    assert!(parse_grid_line("").is_none());
}

#[test]
fn test_parse_grid_line_negative() {
    assert!(matches!(parse_grid_line("-1"), Some(GridLineValue::Line(-1))));
    assert!(matches!(parse_grid_line("-5"), Some(GridLineValue::Line(-5))));
}

#[test]
fn test_parse_grid_line_span_various() {
    assert!(matches!(parse_grid_line("span 3"), Some(GridLineValue::Span(3))));
    assert!(matches!(parse_grid_line("span3"), Some(GridLineValue::Span(3))));
    // span 0 解析为 Span(0)，虽然语义上无意义但解析不拒绝
    assert!(matches!(parse_grid_line("span 0"), Some(GridLineValue::Span(0))));
    // 非数字 span 解析失败
    assert!(parse_grid_line("span abc").is_none());
}

#[test]
fn test_parse_grid_line_shorthand_no_slash() {
    let result = parse_grid_line_shorthand("2");
    assert!(result.is_some());
    let (start, end) = result.unwrap();
    assert!(matches!(start, GridLineValue::Line(2)));
    assert!(matches!(end, GridLineValue::Auto));
}

#[test]
fn test_parse_grid_line_shorthand_empty_parts() {
    assert!(parse_grid_line_shorthand(" / 3").is_none());
    assert!(parse_grid_line_shorthand("2 / ").is_none());
}

#[test]
fn test_parse_grid_line_shorthand_with_slash() {
    let result = parse_grid_line_shorthand("1 / 3");
    assert!(result.is_some());
    let (start, end) = result.unwrap();
    assert!(matches!(start, GridLineValue::Line(1)));
    assert!(matches!(end, GridLineValue::Line(3)));
}

#[test]
fn test_parse_grid_area_shorthand_various() {
    let result = parse_grid_area_shorthand("1 / 2 / 3 / 4");
    assert!(result.is_some());

    let result = parse_grid_area_shorthand("header");
    assert!(result.is_some());

    let result = parse_grid_area_shorthand("auto / auto / span 2 / sidebar");
    assert!(result.is_some());
}

#[test]
fn test_parse_line_height_unitless() {
    assert!(matches!(parse_line_height("1.5"), Some(LineHeightValue::Number(1.5))));
    assert!(matches!(parse_line_height("2"), Some(LineHeightValue::Number(2.0))));
}

#[test]
fn test_parse_line_height_with_unit() {
    assert!(matches!(parse_line_height("24px"), Some(LineHeightValue::Length(_))));
    assert!(matches!(parse_line_height("1.5em"), Some(LineHeightValue::Length(_))));
    assert!(matches!(parse_line_height("120%"), Some(LineHeightValue::Length(_))));
}

#[test]
fn test_parse_line_height_normal() {
    assert!(matches!(parse_line_height("normal"), Some(LineHeightValue::Normal)));
}

#[test]
fn test_parse_line_height_invalid() {
    assert!(parse_line_height("invalid").is_none());
}

#[test]
fn test_parse_word_break_variants() {
    assert!(matches!(parse_word_break("normal"), Some(WordBreakValue::Normal)));
    assert!(matches!(parse_word_break("break-all"), Some(WordBreakValue::BreakAll)));
    assert!(matches!(parse_word_break("keep-all"), Some(WordBreakValue::KeepAll)));
    assert!(matches!(
        parse_word_break("break-word"),
        Some(WordBreakValue::BreakWord)
    ));
    assert!(parse_word_break("invalid").is_none());
}

#[test]
fn test_parse_writing_mode_variants() {
    assert!(matches!(
        parse_writing_mode("horizontal-tb"),
        Some(WritingModeValue::HorizontalTb)
    ));
    assert!(matches!(
        parse_writing_mode("vertical-rl"),
        Some(WritingModeValue::VerticalRl)
    ));
    assert!(matches!(
        parse_writing_mode("vertical-lr"),
        Some(WritingModeValue::VerticalLr)
    ));
    assert!(parse_writing_mode("invalid").is_none());
}

#[test]
fn test_parse_outline_style() {
    assert!(matches!(parse_outline_style("none"), Some(OutlineStyleValue::None)));
    assert!(matches!(parse_outline_style("dotted"), Some(OutlineStyleValue::Dotted)));
    assert!(matches!(parse_outline_style("dashed"), Some(OutlineStyleValue::Dashed)));
    assert!(matches!(parse_outline_style("solid"), Some(OutlineStyleValue::Solid)));
    assert!(matches!(parse_outline_style("double"), Some(OutlineStyleValue::Double)));
    assert!(matches!(parse_outline_style("groove"), Some(OutlineStyleValue::Groove)));
    assert!(matches!(parse_outline_style("ridge"), Some(OutlineStyleValue::Ridge)));
    assert!(matches!(parse_outline_style("inset"), Some(OutlineStyleValue::Inset)));
    assert!(matches!(parse_outline_style("outset"), Some(OutlineStyleValue::Outset)));
    assert!(parse_outline_style("invalid").is_none());
}

#[test]
fn test_parse_cursor_all_values() {
    assert!(matches!(parse_cursor("auto"), Some(CursorValue::Auto)));
    assert!(matches!(parse_cursor("default"), Some(CursorValue::Default)));
    assert!(matches!(parse_cursor("pointer"), Some(CursorValue::Pointer)));
    assert!(matches!(parse_cursor("move"), Some(CursorValue::Move)));
    assert!(matches!(parse_cursor("text"), Some(CursorValue::Text)));
    assert!(matches!(parse_cursor("wait"), Some(CursorValue::Wait)));
    assert!(matches!(parse_cursor("crosshair"), Some(CursorValue::Crosshair)));
    assert!(matches!(parse_cursor("help"), Some(CursorValue::Help)));
    assert!(matches!(parse_cursor("not-allowed"), Some(CursorValue::NotAllowed)));
    assert!(matches!(parse_cursor("grab"), Some(CursorValue::Grab)));
    assert!(matches!(parse_cursor("grabbing"), Some(CursorValue::Grabbing)));
    assert!(matches!(parse_cursor("col-resize"), Some(CursorValue::ColResize)));
    assert!(matches!(parse_cursor("row-resize"), Some(CursorValue::RowResize)));
    assert!(matches!(parse_cursor("ns-resize"), Some(CursorValue::NsResize)));
    assert!(matches!(parse_cursor("ew-resize"), Some(CursorValue::EwResize)));
    assert!(matches!(parse_cursor("none"), Some(CursorValue::None)));
    assert!(matches!(parse_cursor("progress"), Some(CursorValue::Progress)));
    assert!(matches!(parse_cursor("cell"), Some(CursorValue::Cell)));
    assert!(matches!(parse_cursor("copy"), Some(CursorValue::Copy)));
    assert!(matches!(parse_cursor("alias"), Some(CursorValue::Alias)));
    assert!(matches!(parse_cursor("all-scroll"), Some(CursorValue::AllScroll)));
    assert!(matches!(parse_cursor("zoom-in"), Some(CursorValue::ZoomIn)));
    assert!(matches!(parse_cursor("zoom-out"), Some(CursorValue::ZoomOut)));
    assert!(parse_cursor("invalid-cursor").is_none());
}

#[test]
fn test_parse_grid_auto_flow_all() {
    assert!(matches!(parse_grid_auto_flow("row"), Some(GridAutoFlowValue::Row)));
    assert!(matches!(
        parse_grid_auto_flow("column"),
        Some(GridAutoFlowValue::Column)
    ));
    assert!(matches!(
        parse_grid_auto_flow("dense"),
        Some(GridAutoFlowValue::RowDense)
    ));
    assert!(matches!(
        parse_grid_auto_flow("row dense"),
        Some(GridAutoFlowValue::RowDense)
    ));
    assert!(matches!(
        parse_grid_auto_flow("column dense"),
        Some(GridAutoFlowValue::ColumnDense)
    ));
    assert!(parse_grid_auto_flow("invalid").is_none());
}

#[test]
fn test_parse_scroll_padding_auto() {
    assert!(matches!(parse_scroll_padding("auto"), Some(ScrollPadding::Auto)));
}

#[test]
fn test_parse_scroll_padding_length() {
    let result = parse_scroll_padding("10px");
    assert!(result.is_some());
}

#[test]
fn test_parse_scroll_padding_invalid() {
    assert!(parse_scroll_padding("invalid").is_none());
}

#[test]
fn test_parse_scroll_snap_type_computed() {
    let result = parse_scroll_snap_type_computed("mandatory x");
    assert!(result.is_some());
    let result = parse_scroll_snap_type_computed("proximity y");
    assert!(result.is_some());
    let result = parse_scroll_snap_type_computed("none");
    assert!(result.is_some());
}

#[test]
fn test_parse_scroll_snap_align_computed() {
    assert!(parse_scroll_snap_align_computed("start").is_some());
    assert!(parse_scroll_snap_align_computed("end").is_some());
    assert!(parse_scroll_snap_align_computed("center").is_some());
    assert!(parse_scroll_snap_align_computed("none").is_some());
}

#[test]
fn test_parse_scroll_snap_stop_computed() {
    assert!(parse_scroll_snap_stop_computed("normal").is_some());
    assert!(parse_scroll_snap_stop_computed("always").is_some());
}

#[test]
fn test_parse_container_type_computed() {
    assert!(parse_container_type_computed("normal").is_some());
    assert!(parse_container_type_computed("size").is_some());
    assert!(parse_container_type_computed("inline-size").is_some());
}

#[test]
fn test_parse_text_decoration_line_all() {
    assert!(matches!(
        parse_text_decoration_line("none"),
        Some(TextDecorationLineValue::NONE)
    ));
    assert!(matches!(
        parse_text_decoration_line("underline"),
        Some(TextDecorationLineValue {
            underline: true,
            overline: false,
            line_through: false
        })
    ));
    assert!(matches!(
        parse_text_decoration_line("overline"),
        Some(TextDecorationLineValue {
            underline: false,
            overline: true,
            line_through: false
        })
    ));
    assert!(matches!(
        parse_text_decoration_line("line-through"),
        Some(TextDecorationLineValue {
            underline: false,
            overline: false,
            line_through: true
        })
    ));
    assert!(matches!(
        parse_text_decoration_line("blink"),
        Some(TextDecorationLineValue::NONE)
    ));
    assert!(parse_text_decoration_line("invalid").is_none());
}

#[test]
fn test_parse_text_decoration_line_multi_value() {
    // 多值组合（CSS Text Decoration §3）—— driving: css-text-decor 010/011/012/013。
    // 两条组合。
    let v = parse_text_decoration_line("underline overline").unwrap();
    assert!(v.underline && v.overline && !v.line_through);
    // 三条全组合（顺序无关）。
    let v = parse_text_decoration_line("overline line-through underline").unwrap();
    assert!(v.underline && v.overline && v.line_through);
    // 大小写不敏感 + underline line-through（最常见组合）。
    let v = parse_text_decoration_line("Underline Line-Through").unwrap();
    assert!(v.underline && !v.overline && v.line_through);
    // 任一非法关键字 → 整值无效。
    assert!(parse_text_decoration_line("underline bogus").is_none());
    // none 与其他组合 → none 取消（全 false）。
    let v = parse_text_decoration_line("underline none").unwrap();
    assert!(!v.has_any());
    assert!(v == TextDecorationLineValue::NONE);
}

#[test]
fn test_parse_font_family_multiple() {
    let families = parse_font_family("Arial, Helvetica, sans-serif");
    assert_eq!(families.len(), 3);
    assert_eq!(families[0], "Arial");
    assert_eq!(families[2], "sans-serif");
}

#[test]
fn test_parse_font_family_quoted() {
    let families = parse_font_family("\"Times New Roman\", serif");
    assert_eq!(families[0], "\"Times New Roman\"");
    assert_eq!(families[1], "serif");
}

#[test]
fn test_parse_font_family_single_quotes() {
    let families = parse_font_family("'Courier New', monospace");
    assert_eq!(families[0], "\"Courier New\"");
}
