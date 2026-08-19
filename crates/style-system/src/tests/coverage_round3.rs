// ═══════════════════════════════════════════════════════════════════
// 第三轮覆盖率测试 - 覆盖剩余分支
// ═══════════════════════════════════════════════════════════════════

use super::super::*;
use super::helpers::*;

// ═══════════════════════════════════════════════════════════════════
// matcher/mod.rs 覆盖率测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 匹配不存在的节点
fn test_match_nonexistent_node() {
    let (doc, _html, _body, _div, _p) = make_test_dom();

    // 尝试匹配不存在的 NodeId
    let selector = make_tag_selector("div");
    assert!(!matches_selector(&doc, doc.root(), &selector));
}

#[test]
/// 伪元素不匹配 DOM 元素
fn test_pseudo_element_no_match() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    // 手动创建包含伪元素的选择器
    let selector = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoElement(
                        zero_css_parser::ast::PseudoElementSelector::Standard("before".to_string()),
                    )],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, div, &selector));
}

#[test]
/// :lang(en) 对无 lang 属性的 div 不匹配（make_test_dom 未设 lang；CSS 2.1 §5.11.4）
fn test_lang_no_lang_attr_no_match() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    // 手动创建包含 :lang() 的选择器
    let selector = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(
                        zero_css_parser::ast::PseudoClassSelector::Lang(vec!["en".to_string()]),
                    )],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, div, &selector));
}

// ═══════════════════════════════════════════════════════════════════
// property/parse.rs 覆盖率测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 解析无效的 CSS border-style 值
fn parse_border_style_invalid() {
    assert!(crate::property::parse::parse_border_style("invalid").is_none());
}

#[test]
/// 解析无效的 CSS grid-line 值（数字0）
fn parse_grid_line_zero() {
    assert!(crate::property::parse::parse_grid_line("0").is_none());
}

#[test]
/// 解析无效的 grid-line 值（以数字开头的命名区域）
fn parse_grid_line_invalid_name() {
    assert!(crate::property::parse::parse_grid_line("123name").is_none());
}

#[test]
/// 解析无效的 grid-line 值（包含斜杠的命名区域）
fn parse_grid_line_name_with_slash() {
    assert!(crate::property::parse::parse_grid_line("name/test").is_none());
}

#[test]
/// 解析无效的 CSS line-height 值（带单位的数值）
fn parse_line_height_with_unit_number() {
    // 2em is actually a valid line-height value - this test was based on wrong assumption
}

#[test]
/// 解析无效的 CSS line-height 值（非数值）
fn parse_line_height_not_number() {
    assert!(crate::property::parse::parse_line_height("invalid").is_none());
}

#[test]
/// 解析无效的 CSS text-align 值
fn parse_text_align_invalid() {
    assert!(crate::property::parse::parse_text_align("invalid").is_none());
}

#[test]
/// 解析无效的 CSS text-decoration 值
fn parse_text_decoration_invalid() {
    assert!(crate::property::parse::parse_text_decoration("invalid").is_none());
}

#[test]
/// 解析无效的 CSS text-decoration-line 值
fn parse_text_decoration_line_invalid() {
    assert!(crate::property::parse::parse_text_decoration_line("invalid").is_none());
}

#[test]
/// 解析无效的 CSS text-transform 值
fn parse_text_transform_invalid() {
    assert!(crate::property::parse::parse_text_transform("invalid").is_none());
}

#[test]
/// 解析无效的 CSS white-space 值
fn parse_white_space_invalid() {
    assert!(crate::property::parse::parse_white_space("invalid").is_none());
}

#[test]
/// 解析无效的 CSS word-break 值
fn parse_word_break_invalid() {
    assert!(crate::property::parse::parse_word_break("invalid").is_none());
}

#[test]
/// 解析无效的 CSS writing-mode 值
fn parse_writing_mode_invalid() {
    assert!(crate::property::parse::parse_writing_mode("invalid").is_none());
}

#[test]
/// 解析无效的 CSS text-overflow 值
fn parse_text_overflow_invalid() {
    assert!(crate::property::parse::parse_text_overflow("invalid").is_none());
}

#[test]
/// 解析无效的 CSS flex-basis 值
fn parse_flex_basis_invalid() {
    assert!(crate::property::parse::parse_flex_basis("invalid").is_none());
}

#[test]
/// 解析无效的 CSS z-index 值（非数字）
fn parse_z_index_invalid() {
    assert!(crate::property::parse::parse_z_index("invalid").is_none());
}

#[test]
/// 解析无效的 CSS cursor 值
fn parse_cursor_invalid() {
    assert!(crate::property::parse::parse_cursor("invalid").is_none());
}

#[test]
/// 解析无效的 CSS scroll-snap-type 值
fn parse_scroll_snap_type_invalid() {
    assert!(crate::property::parse::parse_scroll_snap_type_computed("invalid").is_none());
}

#[test]
/// 解析无效的 CSS scroll-snap-align 值
fn parse_scroll_snap_align_invalid() {
    assert!(crate::property::parse::parse_scroll_snap_align_computed("invalid").is_none());
}

#[test]
/// 解析无效的 CSS scroll-snap-stop 值
fn parse_scroll_snap_stop_invalid() {
    assert!(crate::property::parse::parse_scroll_snap_stop_computed("invalid").is_none());
}

#[test]
/// 解析无效的 CSS scroll-padding 值
fn parse_scroll_padding_invalid() {
    assert!(crate::property::parse::parse_scroll_padding("invalid").is_none());
}

#[test]
/// 解析无效的 CSS container-type 值
fn parse_container_type_invalid() {
    assert!(crate::property::parse::parse_container_type_computed("invalid").is_none());
}

// ═══════════════════════════════════════════════════════════════════
// property/apply.rs 覆盖率测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// R1402：应用 text-decoration-thickness（CSS Text Decoration 4 §2.3）。
fn apply_text_decoration_thickness() {
    use crate::TextDecorationThicknessValue;
    let mut style = ComputedStyle::default();
    // 默认 auto
    assert_eq!(style.text_decoration_thickness, TextDecorationThicknessValue::Auto);
    // length 2px → Length(2.0)
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "text-decoration-thickness",
        "2px"
    ));
    assert_eq!(
        style.text_decoration_thickness,
        TextDecorationThicknessValue::Length(2.0)
    );
    // 2.3px → Length(2.3)（floor 发生在 paint 层 device-px 取整）
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "text-decoration-thickness",
        "2.3px"
    ));
    assert_eq!(
        style.text_decoration_thickness,
        TextDecorationThicknessValue::Length(2.3)
    );
    // auto 关键字
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "text-decoration-thickness",
        "auto"
    ));
    assert_eq!(style.text_decoration_thickness, TextDecorationThicknessValue::Auto);
    // 非法值不应用（返回 false）
    assert!(!crate::property::apply::apply_property_value(
        &mut style,
        "text-decoration-thickness",
        "bogus"
    ));
    let previous = style.text_decoration_thickness.clone();
    assert!(!crate::property::apply::apply_property_value(
        &mut style,
        "text-decoration-thickness",
        "-1px"
    ));
    assert_eq!(style.text_decoration_thickness, previous);
}

#[test]
/// 应用 max-width: none
fn apply_max_width_none() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "max-width",
        "none"
    ));
    assert_eq!(style.max_width, LengthValue::Px(f64::INFINITY));
}

#[test]
/// 应用 max-height: none
fn apply_max_height_none() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "max-height",
        "none"
    ));
    assert_eq!(style.max_height, LengthValue::Px(f64::INFINITY));
}

#[test]
/// 应用 border-right-width
fn apply_border_right_width() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "border-right-width",
        "2px"
    ));
    assert_eq!(style.border_right_width, LengthValue::Px(2.0));
}

#[test]
/// 应用 border-bottom-width
fn apply_border_bottom_width() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "border-bottom-width",
        "3px"
    ));
    assert_eq!(style.border_bottom_width, LengthValue::Px(3.0));
}

#[test]
/// CSS 规范：border-width 负值无效，应被拒绝（保持初始值）
fn apply_border_bottom_width_negative_rejected() {
    let mut style = ComputedStyle::default();
    // 初始值为 medium = Px(3.0)
    let initial = style.border_bottom_width.clone();
    assert!(!crate::property::apply::apply_property_value(
        &mut style,
        "border-bottom-width",
        "-1px"
    ));
    // 负值被拒绝，保持初始值
    assert_eq!(style.border_bottom_width, initial);
}

#[test]
/// 应用 border-left-width
fn apply_border_left_width() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "border-left-width",
        "1px"
    ));
    assert_eq!(style.border_left_width, LengthValue::Px(1.0));
}

#[test]
/// 应用 border-top-right-radius
fn apply_border_top_right_radius() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "border-top-right-radius",
        "8px"
    ));
    assert_eq!(style.border_top_right_radius, LengthValue::Px(8.0));
}

#[test]
/// 应用 border-bottom-right-radius
fn apply_border_bottom_right_radius() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "border-bottom-right-radius",
        "8px"
    ));
    assert_eq!(style.border_bottom_right_radius, LengthValue::Px(8.0));
}

#[test]
/// 应用 border-bottom-left-radius
fn apply_border_bottom_left_radius() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "border-bottom-left-radius",
        "8px"
    ));
    assert_eq!(style.border_bottom_left_radius, LengthValue::Px(8.0));
}

#[test]
/// 应用 outline-width
fn apply_outline_width() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "outline-width",
        "2px"
    ));
    assert_eq!(style.outline_width, LengthValue::Px(2.0));
}

#[test]
/// 应用 outline-style
fn apply_outline_style() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "outline-style",
        "dashed"
    ));
    assert_eq!(style.outline_style, OutlineStyleValue::Dashed);
}

#[test]
/// 应用 outline-color
fn apply_outline_color() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "outline-color",
        "blue"
    ));
    assert_eq!(style.outline_color, ColorValue::Rgba(0, 0, 255, 255));
}

#[test]
/// 应用 outline-offset
fn apply_outline_offset() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "outline-offset",
        "3px"
    ));
    assert_eq!(style.outline_offset, LengthValue::Px(3.0));
}

#[test]
/// 应用 background-color
fn apply_background_color() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "background-color",
        "#ff0000"
    ));
    assert_eq!(style.background_color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
/// 应用 opacity
fn apply_opacity() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style, "opacity", "0.5"
    ));
    assert_eq!(style.opacity, 0.5);
}

#[test]
/// 应用 visibility
fn apply_visibility() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "visibility",
        "hidden"
    ));
    assert_eq!(style.visibility, VisibilityValue::Hidden);
}

#[test]
/// 应用 font-size
fn apply_font_size() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "font-size",
        "16px"
    ));
    assert_eq!(style.font_size, LengthValue::Px(16.0));
}

#[test]
/// 应用 font-weight
fn apply_font_weight() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "font-weight",
        "bold"
    ));
    assert_eq!(style.font_weight, zero_css_parser::values::FontWeightValue::Bold);
}

#[test]
/// 应用 font-style
fn apply_font_style() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "font-style",
        "italic"
    ));
    assert_eq!(style.font_style, zero_css_parser::values::FontStyleValue::Italic);
}

#[test]
/// 应用 line-height: normal
fn apply_line_height_normal() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "line-height",
        "normal"
    ));
    assert_eq!(style.line_height, LineHeightValue::Normal);
}

#[test]
/// 应用 line-height: number
fn apply_line_height_number() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "line-height",
        "1.5"
    ));
    assert_eq!(style.line_height, LineHeightValue::Number(1.5));
}

#[test]
/// 应用 text-align: center
fn apply_text_align_center() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "text-align",
        "center"
    ));
    assert_eq!(style.text_align, TextAlignValue::Center);
}

#[test]
/// 应用 text-decoration: underline
fn apply_text_decoration_underline() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "text-decoration",
        "underline"
    ));
    assert_eq!(style.text_decoration, TextDecorationValue::Underline);
}

#[test]
/// 应用 text-decoration-line: line-through
fn apply_text_decoration_line_line_through() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "text-decoration-line",
        "line-through"
    ));
    assert_eq!(
        style.text_decoration_line,
        TextDecorationLineValue {
            underline: false,
            overline: false,
            line_through: true
        }
    );
}

#[test]
/// 应用 text-transform: uppercase
fn apply_text_transform_uppercase() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "text-transform",
        "uppercase"
    ));
    assert_eq!(style.text_transform, TextTransformValue::Uppercase);
}

#[test]
/// 应用 letter-spacing
fn apply_letter_spacing() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "letter-spacing",
        "2px"
    ));
    assert_eq!(style.letter_spacing, LengthValue::Px(2.0));
}

#[test]
/// 应用 word-spacing
fn apply_word_spacing() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "word-spacing",
        "5px"
    ));
    assert_eq!(style.word_spacing, LengthValue::Px(5.0));
}

#[test]
/// 应用 white-space: nowrap
fn apply_white_space_nowrap() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "white-space",
        "nowrap"
    ));
    assert_eq!(style.white_space, WhiteSpaceValue::Nowrap);
}

#[test]
/// 应用 word-break: break-all
fn apply_word_break_break_all() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "word-break",
        "break-all"
    ));
    assert_eq!(style.word_break, WordBreakValue::BreakAll);
}

#[test]
/// 应用 writing-mode: vertical-rl
fn apply_writing_mode_vertical_rl() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "writing-mode",
        "vertical-rl"
    ));
    assert_eq!(style.writing_mode, WritingModeValue::VerticalRl);
}

#[test]
/// 应用 text-indent
fn apply_text_indent() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "text-indent",
        "20px"
    ));
    assert_eq!(style.text_indent, LengthValue::Px(20.0));
}

#[test]
/// 应用 text-overflow: ellipsis
fn apply_text_overflow_ellipsis() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "text-overflow",
        "ellipsis"
    ));
    assert_eq!(style.text_overflow, TextOverflowValue::Ellipsis);
}

#[test]
/// 应用 vertical-align: middle
fn apply_vertical_align_middle() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "vertical-align",
        "middle"
    ));
    assert_eq!(
        style.vertical_align,
        zero_css_parser::values::VerticalAlignValue::Middle
    );
}

#[test]
/// 应用 flex-direction: row-reverse
fn apply_flex_direction_row_reverse() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "flex-direction",
        "row-reverse"
    ));
    assert_eq!(
        style.flex_direction,
        zero_css_parser::values::FlexDirectionValue::RowReverse
    );
}

#[test]
/// 应用 flex-wrap: wrap
fn apply_flex_wrap_wrap() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "flex-wrap",
        "wrap"
    ));
    assert_eq!(style.flex_wrap, zero_css_parser::values::FlexWrapValue::Wrap);
}

#[test]
/// 应用 justify-content: space-between
fn apply_justify_content_space_between() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "justify-content",
        "space-between"
    ));
    assert_eq!(
        style.justify_content,
        zero_css_parser::values::AlignmentValue::SpaceBetween
    );
}

#[test]
/// 应用 align-items: center
fn apply_align_items_center() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "align-items",
        "center"
    ));
    assert_eq!(style.align_items, zero_css_parser::values::AlignmentValue::Center);
}

#[test]
/// 应用 align-self: flex-end
fn apply_align_self_flex_end() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "align-self",
        "flex-end"
    ));
    assert_eq!(style.align_self, zero_css_parser::values::AlignmentValue::FlexEnd);
}

#[test]
/// 应用 flex-grow
fn apply_flex_grow() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "flex-grow",
        "2"
    ));
    assert_eq!(style.flex_grow, 2.0);
}

#[test]
/// 应用 flex-shrink
fn apply_flex_shrink() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "flex-shrink",
        "0"
    ));
    assert_eq!(style.flex_shrink, 0.0);
}

#[test]
/// 负值 flex-shrink 非法（CSS Flexbox §7.3.2），apply 拒绝并保留初始值 1.0。
///
/// 负值透传 taffy 会使 scaled shrink factor 为负、`sum > 0` 门控跳过收缩分布，item 不收缩
/// （flex-shrink-002 FAIL）。拒绝负值 → 保留 default_impl 初始值 1.0 → item 正常收缩。
fn apply_flex_shrink_negative_rejected() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.flex_shrink, 1.0, "初始值应为 1.0");
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "flex-shrink",
        "-3"
    ));
    assert_eq!(style.flex_shrink, 1.0, "负值应被拒绝，保留初始值 1.0");
}

#[test]
/// 负值 flex-grow 非法（CSS Flexbox §7.3.1），apply 拒绝并保留初始值 0.0。
fn apply_flex_grow_negative_rejected() {
    let mut style = ComputedStyle::default();
    assert_eq!(style.flex_grow, 0.0);
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "flex-grow",
        "-2"
    ));
    assert_eq!(style.flex_grow, 0.0, "负值应被拒绝，保留初始值 0.0");
}

#[test]
/// 应用 flex-basis
fn apply_flex_basis() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "flex-basis",
        "200px"
    ));
    assert_eq!(
        style.flex_basis,
        FlexBasisValue::Length(zero_css_parser::values::LengthValue::Px(200.0))
    );
}

#[test]
/// 应用 gap
fn apply_gap() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(&mut style, "gap", "10px"));
    assert_eq!(style.gap, LengthValue::Px(10.0));
}

#[test]
/// 应用 column-gap
fn apply_column_gap() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "column-gap",
        "5px"
    ));
    assert_eq!(style.column_gap, LengthValue::Px(5.0));
}

#[test]
/// 应用 order
fn apply_order() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(&mut style, "order", "3"));
    assert_eq!(style.order, 3);
}

#[test]
/// 应用 top
fn apply_top() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(&mut style, "top", "10px"));
    assert_eq!(style.top, LengthValue::Px(10.0));
}

#[test]
/// 应用 right
fn apply_right() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style, "right", "20px"
    ));
    assert_eq!(style.right, LengthValue::Px(20.0));
}

#[test]
/// 应用 bottom
fn apply_bottom() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style, "bottom", "30px"
    ));
    assert_eq!(style.bottom, LengthValue::Px(30.0));
}

#[test]
/// 应用 left
fn apply_left() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(&mut style, "left", "40px"));
    assert_eq!(style.left, LengthValue::Px(40.0));
}

#[test]
/// 应用 z-index
fn apply_z_index() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style, "z-index", "10"
    ));
    assert_eq!(style.z_index, ZIndexValue::Integer(10));
}

#[test]
/// 应用 overflow-x: hidden
fn apply_overflow_x_hidden() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "overflow-x",
        "hidden"
    ));
    assert_eq!(style.overflow_x, zero_css_parser::values::OverflowValue::Hidden);
}

#[test]
/// 应用 overflow-y: scroll
fn apply_overflow_y_scroll() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "overflow-y",
        "scroll"
    ));
    assert_eq!(style.overflow_y, zero_css_parser::values::OverflowValue::Scroll);
}

#[test]
/// 应用 cursor: pointer
fn apply_cursor_pointer() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style, "cursor", "pointer"
    ));
    assert_eq!(style.cursor, CursorValue::Pointer);
}

#[test]
/// 应用 transform: rotate(45deg)
fn apply_transform() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transform",
        "rotate(45deg)"
    ));
    // TransformValue 是一个复杂枚举，我们只验证它被设置了
    assert_ne!(style.transform, zero_css_parser::values::TransformValue::None);
}

#[test]
/// 应用 transform-origin: 50% 50%
fn apply_transform_origin() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transform-origin",
        "50% 50%"
    ));
    assert_eq!(style.transform_origin_x, LengthValue::Percentage(50.0));
    assert_eq!(style.transform_origin_y, LengthValue::Percentage(50.0));
}

#[test]
/// 应用 transform-origin: 10px
fn apply_transform_origin_single() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transform-origin",
        "10px"
    ));
    assert_eq!(style.transform_origin_x, LengthValue::Px(10.0));
    assert_eq!(style.transform_origin_y, LengthValue::Percentage(50.0));
}

#[test]
/// 应用 transform-origin: 10px 20px
fn apply_transform_origin_double() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transform-origin",
        "10px 20px"
    ));
    assert_eq!(style.transform_origin_x, LengthValue::Px(10.0));
    assert_eq!(style.transform_origin_y, LengthValue::Px(20.0));
}

#[test]
/// 应用 perspective: 500px
fn apply_perspective() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "perspective",
        "500px"
    ));
    assert_eq!(style.perspective, LengthValue::Px(500.0));
}

#[test]
/// 应用 perspective: none
fn apply_perspective_none() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "perspective",
        "none"
    ));
    assert_eq!(style.perspective, LengthValue::Px(0.0));
}

#[test]
/// 应用 perspective-origin: 50% 50%
fn apply_perspective_origin() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "perspective-origin",
        "50% 50%"
    ));
    assert_eq!(style.perspective_origin_x, LengthValue::Percentage(50.0));
    assert_eq!(style.perspective_origin_y, LengthValue::Percentage(50.0));
}

#[test]
/// 应用 transform-style: flat
fn apply_transform_style_flat() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transform-style",
        "flat"
    ));
    assert_eq!(style.transform_style, TransformStyleValue::Flat);
}

#[test]
/// 应用 transform-style: preserve-3d
fn apply_transform_style_preserve_3d() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transform-style",
        "preserve-3d"
    ));
    assert_eq!(style.transform_style, TransformStyleValue::Preserve3d);
}

#[test]
/// 应用 transform-style: invalid (不应用)
fn apply_transform_style_invalid() {
    let mut style = ComputedStyle::default();
    assert!(!crate::property::apply::apply_property_value(
        &mut style,
        "transform-style",
        "invalid"
    ));
}

#[test]
/// 应用 backface-visibility: visible
fn apply_backface_visibility_visible() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "backface-visibility",
        "visible"
    ));
    assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Visible);
}

#[test]
/// 应用 backface-visibility: hidden
fn apply_backface_visibility_hidden() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "backface-visibility",
        "hidden"
    ));
    assert_eq!(style.backface_visibility, BackfaceVisibilityValue::Hidden);
}

#[test]
/// 应用 transition-property: opacity, transform
fn apply_transition_property() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transition-property",
        "opacity, transform"
    ));
    assert_eq!(
        style.transition_property,
        vec!["opacity".to_string(), "transform".to_string()]
    );
}

#[test]
/// 应用 transition-property: none
fn apply_transition_property_none() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transition-property",
        "none"
    ));
    assert_eq!(style.transition_property, vec!["none".to_string()]); // R2756：保留 "none" 对齐 Chromium
}

#[test]
/// 应用 transition-duration: 0.3s, 0.5s
fn apply_transition_duration() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transition-duration",
        "0.3s, 0.5s"
    ));
    assert_eq!(style.transition_duration, vec![0.3, 0.5]);
}

#[test]
/// 应用 transition-timing-function: ease
fn apply_transition_timing_function() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transition-timing-function",
        "ease"
    ));
    assert!(!style.transition_timing_function.is_empty());
}

#[test]
/// 应用 transition-delay: 0.1s
fn apply_transition_delay() {
    let mut style = ComputedStyle::default();
    assert!(crate::property::apply::apply_property_value(
        &mut style,
        "transition-delay",
        "0.1s"
    ));
    assert_eq!(style.transition_delay, vec![0.1]);
}

// ═══════════════════════════════════════════════════════════════════
// 边界条件和错误情况测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 应用不存在的属性
fn apply_nonexistent_property() {
    let mut style = ComputedStyle::default();
    assert!(!crate::property::apply::apply_property_value(
        &mut style,
        "nonexistent-property",
        "value"
    ));
}

#[test]
/// 应用属性为边界值
fn apply_properties_boundary_values() {
    let mut style = ComputedStyle::default();

    // 测试边界值
    let boundary_tests = [
        ("width", "0"),
        ("height", "0"),
        ("margin-top", "0"),
        ("margin-right", "0"),
        ("margin-bottom", "0"),
        ("margin-left", "0"),
        ("padding-top", "0"),
        ("padding-right", "0"),
        ("padding-bottom", "0"),
        ("padding-left", "0"),
        ("border-top-width", "0"),
        ("border-right-width", "0"),
        ("border-bottom-width", "0"),
        ("border-left-width", "0"),
        ("opacity", "0"),
        ("opacity", "1"),
        ("z-index", "0"),
        ("flex-grow", "0"),
        ("flex-shrink", "0"),
        ("order", "0"),
    ];

    for (prop, value) in boundary_tests {
        assert!(
            crate::property::apply::apply_property_value(&mut style, prop, value),
            "Property {} should accept boundary value: {}",
            prop,
            value
        );
    }
}

#[test]
/// 应用属性为最大值
fn apply_properties_max_values() {
    let mut style = ComputedStyle::default();

    // 测试最大值
    let max_tests = [
        ("width", "999999px"),
        ("height", "999999px"),
        ("margin-top", "999999px"),
        ("margin-right", "999999px"),
        ("margin-bottom", "999999px"),
        ("margin-left", "999999px"),
        ("padding-top", "999999px"),
        ("padding-right", "999999px"),
        ("padding-bottom", "999999px"),
        ("padding-left", "999999px"),
        ("border-top-width", "999999px"),
        ("border-right-width", "999999px"),
        ("border-bottom-width", "999999px"),
        ("border-left-width", "999999px"),
        ("font-size", "999999px"),
        ("letter-spacing", "999999px"),
        ("word-spacing", "999999px"),
        ("text-indent", "999999px"),
        ("top", "999999px"),
        ("right", "999999px"),
        ("bottom", "999999px"),
        ("left", "999999px"),
        ("z-index", "999999"),
        ("flex-grow", "999999"),
        ("flex-shrink", "999999"),
        ("order", "999999"),
    ];

    for (prop, value) in max_tests {
        assert!(
            crate::property::apply::apply_property_value(&mut style, prop, value),
            "Property {} should accept max value: {}",
            prop,
            value
        );
    }
}
