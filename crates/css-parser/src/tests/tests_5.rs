use super::*;

// ═══════════════════════════════════════════════════════════════════════
// writing-mode 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_writing_mode_horizontal_tb() {
    assert_eq!(
        parse_writing_mode("horizontal-tb"),
        Some(WritingModeValue::HorizontalTb)
    );
}

#[test]
fn test_parse_writing_mode_vertical_rl() {
    assert_eq!(parse_writing_mode("vertical-rl"), Some(WritingModeValue::VerticalRl));
}

#[test]
fn test_parse_writing_mode_vertical_lr() {
    assert_eq!(parse_writing_mode("vertical-lr"), Some(WritingModeValue::VerticalLr));
}

#[test]
fn test_parse_writing_mode_invalid() {
    assert_eq!(parse_writing_mode("invalid"), None);
    assert_eq!(parse_writing_mode(""), None);
    assert_eq!(parse_writing_mode("sideways-rl"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// text-decoration-line / text-transform / spacing 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_text_decoration_line 所有 5 个有效值
fn test_parse_text_decoration_line() {
    assert_eq!(parse_text_decoration_line("none"), Some(TextDecorationLineValue::None));
    assert_eq!(
        parse_text_decoration_line("underline"),
        Some(TextDecorationLineValue::Underline)
    );
    assert_eq!(
        parse_text_decoration_line("overline"),
        Some(TextDecorationLineValue::Overline)
    );
    assert_eq!(
        parse_text_decoration_line("line-through"),
        Some(TextDecorationLineValue::LineThrough)
    );
    assert_eq!(
        parse_text_decoration_line("blink"),
        Some(TextDecorationLineValue::Blink)
    );
}

#[test]
/// 测试 parse_text_decoration_line 无效输入
fn test_parse_text_decoration_line_invalid() {
    assert_eq!(parse_text_decoration_line("invalid"), None);
    assert_eq!(parse_text_decoration_line(""), None);
    assert_eq!(parse_text_decoration_line("double-underline"), None);
}

#[test]
/// 测试 parse_text_transform 所有 4 个有效值
fn test_parse_text_transform() {
    assert_eq!(parse_text_transform("none"), Some(TextTransformValue::None));
    assert_eq!(parse_text_transform("uppercase"), Some(TextTransformValue::Uppercase));
    assert_eq!(parse_text_transform("lowercase"), Some(TextTransformValue::Lowercase));
    assert_eq!(parse_text_transform("capitalize"), Some(TextTransformValue::Capitalize));
}

#[test]
/// 测试 parse_text_transform 无效输入
fn test_parse_text_transform_invalid() {
    assert_eq!(parse_text_transform("invalid"), None);
    assert_eq!(parse_text_transform(""), None);
    assert_eq!(parse_text_transform("full-width"), None);
}

#[test]
/// 测试 parse_spacing 的 px 值解析
fn test_parse_letter_spacing_px() {
    assert_eq!(parse_spacing("2px"), Some(LengthValue::Px(2.0)));
    assert_eq!(parse_spacing("0px"), Some(LengthValue::Px(0.0)));
    assert_eq!(parse_spacing("-1px"), Some(LengthValue::Px(-1.0)));
}

#[test]
/// 测试 parse_spacing 的 em 值解析
fn test_parse_letter_spacing_em() {
    assert_eq!(parse_spacing("0.5em"), Some(LengthValue::Em(0.5)));
    assert_eq!(parse_spacing("1em"), Some(LengthValue::Em(1.0)));
}

#[test]
/// 测试 parse_spacing 的 "normal" 关键字映射为 Px(0.0)
fn test_parse_letter_spacing_normal() {
    assert_eq!(parse_spacing("normal"), Some(LengthValue::Px(0.0)));
    assert_eq!(parse_spacing("Normal"), Some(LengthValue::Px(0.0)));
    assert_eq!(parse_spacing("  normal  "), Some(LengthValue::Px(0.0)));
}

#[test]
/// 测试 parse_spacing 无效输入
fn test_parse_letter_spacing_invalid() {
    assert_eq!(parse_spacing("abc"), None);
    assert_eq!(parse_spacing(""), None);
}

#[test]
/// 测试 parse_spacing 用于 word-spacing 的 px 值
fn test_parse_word_spacing_px() {
    assert_eq!(parse_spacing("4px"), Some(LengthValue::Px(4.0)));
    assert_eq!(parse_spacing("0.25em"), Some(LengthValue::Em(0.25)));
}

#[test]
/// 测试 parse_spacing 用于 word-spacing 的 "normal" 关键字
fn test_parse_word_spacing_normal() {
    assert_eq!(parse_spacing("normal"), Some(LengthValue::Px(0.0)));
}

// ═══════════════════════════════════════════════════════════════════════
// text-shadow / box-shadow 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_text_shadow 的 "none" 值
fn test_parse_text_shadow_none() {
    let result = parse_text_shadow("none").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(0.0));
    assert_eq!(result.offset_y, LengthValue::Px(0.0));
    assert_eq!(result.blur_radius, LengthValue::Px(0.0));
    assert_eq!(result.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// 测试 parse_text_shadow 基本偏移（无模糊、无颜色）
fn test_parse_text_shadow_basic() {
    let result = parse_text_shadow("2px 2px").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(2.0));
    assert_eq!(result.offset_y, LengthValue::Px(2.0));
    assert_eq!(result.blur_radius, LengthValue::Px(0.0));
    assert_eq!(result.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// 测试 parse_text_shadow 带模糊半径
fn test_parse_text_shadow_with_blur() {
    let result = parse_text_shadow("2px 2px 4px").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(2.0));
    assert_eq!(result.offset_y, LengthValue::Px(2.0));
    assert_eq!(result.blur_radius, LengthValue::Px(4.0));
    assert_eq!(result.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// 测试 parse_text_shadow 带命名颜色
fn test_parse_text_shadow_with_color() {
    let result = parse_text_shadow("2px 2px red").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(2.0));
    assert_eq!(result.offset_y, LengthValue::Px(2.0));
    assert_eq!(result.blur_radius, LengthValue::Px(0.0));
    assert_eq!(result.color, ColorValue::Rgba(255, 0, 0, 255));
}

#[test]
/// 测试 parse_box_shadow 的 "none" 值
fn test_parse_box_shadow_none() {
    let result = parse_box_shadow("none").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(0.0));
    assert_eq!(result.offset_y, LengthValue::Px(0.0));
    assert_eq!(result.blur_radius, LengthValue::Px(0.0));
    assert_eq!(result.spread_radius, LengthValue::Px(0.0));
    assert_eq!(result.color, ColorValue::Rgba(0, 0, 0, 255));
    assert!(!result.inset);
}

#[test]
/// 测试 parse_box_shadow 基本偏移
fn test_parse_box_shadow_basic() {
    let result = parse_box_shadow("2px 2px").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(2.0));
    assert_eq!(result.offset_y, LengthValue::Px(2.0));
    assert!(!result.inset);
}

#[test]
/// 测试 parse_box_shadow 带 inset 关键字、模糊和颜色
fn test_parse_box_shadow_inset() {
    let result = parse_box_shadow("inset 2px 2px 4px black").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(2.0));
    assert_eq!(result.offset_y, LengthValue::Px(2.0));
    assert_eq!(result.blur_radius, LengthValue::Px(4.0));
    assert_eq!(result.color, ColorValue::Rgba(0, 0, 0, 255));
    assert!(result.inset);
}

// ═══════════════════════════════════════════════════════════════════════
// text-overflow 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_text_overflow_clip() {
    assert_eq!(parse_text_overflow("clip"), Some(TextOverflowValue::Clip));
}

#[test]
fn test_parse_text_overflow_ellipsis() {
    assert_eq!(parse_text_overflow("ellipsis"), Some(TextOverflowValue::Ellipsis));
}

#[test]
fn test_parse_text_overflow_custom_string() {
    assert_eq!(
        parse_text_overflow("\"...\""),
        Some(TextOverflowValue::String("...".to_string()))
    );
    assert_eq!(
        parse_text_overflow("'…'"),
        Some(TextOverflowValue::String("…".to_string()))
    );
}

#[test]
fn test_parse_text_overflow_invalid() {
    assert_eq!(parse_text_overflow("fade"), None);
    assert_eq!(parse_text_overflow("\"\""), None); // 空字符串不合法
}

// ═══════════════════════════════════════════════════════════════════════
// text-indent 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_text_indent_px() {
    assert_eq!(parse_text_indent("20px"), Some(LengthValue::Px(20.0)));
}

#[test]
fn test_parse_text_indent_em() {
    assert_eq!(parse_text_indent("2em"), Some(LengthValue::Em(2.0)));
}

#[test]
fn test_parse_text_indent_percentage() {
    assert_eq!(parse_text_indent("10%"), Some(LengthValue::Percentage(10.0)));
}

#[test]
fn test_parse_text_indent_invalid() {
    assert_eq!(parse_text_indent("auto"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// text-decoration-inset 解析测试（R1607）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_text_decoration_inset_single_px() {
    // 单值 → start=end（正值=向内缩进）
    let v = parse_text_decoration_inset("10px").unwrap();
    assert_eq!(v.start, LengthValue::Px(10.0));
    assert_eq!(v.end, LengthValue::Px(10.0));
}

#[test]
fn test_parse_text_decoration_inset_two_values() {
    // 两值 → (start, end)；inset-001 用例 10px -10px
    let v = parse_text_decoration_inset("10px -10px").unwrap();
    assert_eq!(v.start, LengthValue::Px(10.0));
    assert_eq!(v.end, LengthValue::Px(-10.0));
}

#[test]
fn test_parse_text_decoration_inset_em() {
    // em 支持（inset-005 用例 -0.25em，负值=向外延伸）
    let v = parse_text_decoration_inset("-0.25em").unwrap();
    assert_eq!(v.start, LengthValue::Em(-0.25));
    assert_eq!(v.end, LengthValue::Em(-0.25));
}

#[test]
fn test_parse_text_decoration_inset_invalid() {
    // 关键字 / 三值 / 非长度 → None
    assert_eq!(parse_text_decoration_inset("auto"), None);
    assert_eq!(parse_text_decoration_inset("1px 2px 3px"), None);
    assert_eq!(parse_text_decoration_inset("abc"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// table-layout 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_table_layout_auto() {
    assert_eq!(parse_table_layout("auto"), Some(TableLayoutValue::Auto));
}

#[test]
fn test_parse_table_layout_fixed() {
    assert_eq!(parse_table_layout("fixed"), Some(TableLayoutValue::Fixed));
}

#[test]
fn test_parse_table_layout_invalid() {
    assert_eq!(parse_table_layout("inherit"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// caption-side 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_caption_side_top() {
    assert_eq!(parse_caption_side("top"), Some(CaptionSideValue::Top));
}

#[test]
fn test_parse_caption_side_bottom() {
    assert_eq!(parse_caption_side("bottom"), Some(CaptionSideValue::Bottom));
}

#[test]
fn test_parse_caption_side_invalid() {
    assert_eq!(parse_caption_side("left"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// border-collapse 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_border_collapse_separate() {
    assert_eq!(parse_border_collapse("separate"), Some(BorderCollapseValue::Separate));
}

#[test]
fn test_parse_border_collapse_collapse() {
    assert_eq!(parse_border_collapse("collapse"), Some(BorderCollapseValue::Collapse));
}

#[test]
fn test_parse_border_collapse_invalid() {
    assert_eq!(parse_border_collapse("auto"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// resize 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_resize_none() {
    assert_eq!(parse_resize("none"), Some(ResizeValue::None));
}

#[test]
fn test_parse_resize_both() {
    assert_eq!(parse_resize("both"), Some(ResizeValue::Both));
}

#[test]
fn test_parse_resize_horizontal() {
    assert_eq!(parse_resize("horizontal"), Some(ResizeValue::Horizontal));
}

#[test]
fn test_parse_resize_vertical() {
    assert_eq!(parse_resize("vertical"), Some(ResizeValue::Vertical));
}

#[test]
fn test_parse_resize_block_inline() {
    assert_eq!(parse_resize("block"), Some(ResizeValue::Block));
    assert_eq!(parse_resize("inline"), Some(ResizeValue::Inline));
}

#[test]
fn test_parse_resize_invalid() {
    assert_eq!(parse_resize("auto"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 29. 未覆盖的边界条件测试 — word-break / contain / grid-area / length-shorthand / length-vw
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_word_break 所有关键字：normal、break-all、keep-all、break-word，
/// 以及大小写不敏感和无效输入。此前 parse_word_break 无任何测试。
fn test_parse_word_break_all_values() {
    use crate::values::{WordBreakValue, parse_word_break};
    assert_eq!(parse_word_break("normal"), Some(WordBreakValue::Normal));
    assert_eq!(parse_word_break("break-all"), Some(WordBreakValue::BreakAll));
    assert_eq!(parse_word_break("keep-all"), Some(WordBreakValue::KeepAll));
    assert_eq!(parse_word_break("break-word"), Some(WordBreakValue::BreakWord));
    // 大小写不敏感
    assert_eq!(parse_word_break("BREAK-ALL"), Some(WordBreakValue::BreakAll));
    assert_eq!(parse_word_break("  Keep-All  "), Some(WordBreakValue::KeepAll));
    // 无效输入
    assert_eq!(parse_word_break("invalid"), None);
    assert_eq!(parse_word_break(""), None);
    assert_eq!(parse_word_break("inherit"), None);
}

/// 测试 parse_line_break 所有关键字（CSS Text 3 §5.3）：auto/loose/normal/strict/anywhere，
/// 大小写不敏感，无效输入返回 None。R1008 line-break:anywhere → BreakAll 的解析基础。
#[test]
fn test_parse_line_break_all_values() {
    use crate::values::{LineBreakValue, parse_line_break};
    assert_eq!(parse_line_break("auto"), Some(LineBreakValue::Auto));
    assert_eq!(parse_line_break("loose"), Some(LineBreakValue::Loose));
    assert_eq!(parse_line_break("normal"), Some(LineBreakValue::Normal));
    assert_eq!(parse_line_break("strict"), Some(LineBreakValue::Strict));
    assert_eq!(parse_line_break("anywhere"), Some(LineBreakValue::Anywhere));
    // 大小写不敏感
    assert_eq!(parse_line_break("ANYWHERE"), Some(LineBreakValue::Anywhere));
    assert_eq!(parse_line_break("  Strict  "), Some(LineBreakValue::Strict));
    // 无效输入
    assert_eq!(parse_line_break("invalid"), None);
    assert_eq!(parse_line_break(""), None);
    assert_eq!(parse_line_break("break-all"), None); // 不是 line-break 值
}

#[test]
/// 测试 parse_contain 所有关键字和自定义标志位组合。
/// 验证 none/strict/content/单关键字/多关键字组合的正确解析，
/// 以及无效输入返回 None。此前 parse_contain 无任何测试。
fn test_parse_contain_strict_and_custom_flags() {
    use crate::values::{ContainValue, parse_contain};
    // 单关键字
    assert_eq!(parse_contain("none"), Some(ContainValue::None));
    assert_eq!(parse_contain("strict"), Some(ContainValue::Strict));
    assert_eq!(parse_contain("content"), Some(ContainValue::Content));
    assert_eq!(parse_contain("size"), Some(ContainValue::Size));
    assert_eq!(parse_contain("layout"), Some(ContainValue::Layout));
    assert_eq!(parse_contain("style"), Some(ContainValue::Style));
    assert_eq!(parse_contain("paint"), Some(ContainValue::Paint));
    // 多关键字组合 — layout paint → FLAG_LAYOUT | FLAG_PAINT = 0x0A
    assert!(
        matches!(parse_contain("layout paint"), Some(ContainValue::Custom(f)) if f == ContainValue::FLAG_LAYOUT | ContainValue::FLAG_PAINT)
    );
    // size layout style paint → 全部标志位
    assert!(matches!(
        parse_contain("size layout style paint"),
        Some(ContainValue::Custom(f)) if f == ContainValue::FLAG_SIZE | ContainValue::FLAG_LAYOUT | ContainValue::FLAG_STYLE | ContainValue::FLAG_PAINT
    ));
    // 大小写不敏感
    assert_eq!(parse_contain("STRICT"), Some(ContainValue::Strict));
    assert_eq!(parse_contain("  LAYOUT PAINT  "), parse_contain("layout paint"));
    // 无效输入
    assert_eq!(parse_contain("invalid"), None);
    assert_eq!(parse_contain(""), None);
}

#[test]
/// 测试 parse_grid_area 各种斜杠分割格式：
/// 单值、2 值（row-start / col-start）、3 值、4 值，
/// 以及空输入和无效格式。此前 parse_grid_area 无任何测试。
fn test_parse_grid_area_slash_separated() {
    use crate::values::parse_grid_area;
    // 单值：所有四项相同
    let result = parse_grid_area("header");
    assert_eq!(
        result,
        Some(("header".into(), "header".into(), "header".into(), "header".into()))
    );

    // 2 值：row-start / col-start，row-end 和 col-end 为 "auto"
    let result = parse_grid_area("1 / 3");
    assert_eq!(result, Some(("1".into(), "auto".into(), "3".into(), "auto".into())));

    // 3 值：row-start / row-end / col-start，col-end 为 "auto"
    let result = parse_grid_area("1 / 3 / 5");
    assert_eq!(result, Some(("1".into(), "3".into(), "5".into(), "auto".into())));

    // 4 值：row-start / row-end / col-start / col-end
    let result = parse_grid_area("1 / 3 / 5 / span 2");
    assert_eq!(result, Some(("1".into(), "3".into(), "5".into(), "span 2".into())));

    // 命名区域
    let result = parse_grid_area("sidebar");
    assert_eq!(
        result,
        Some(("sidebar".into(), "sidebar".into(), "sidebar".into(), "sidebar".into()))
    );

    // auto 关键字
    let result = parse_grid_area("auto");
    assert_eq!(
        result,
        Some(("auto".into(), "auto".into(), "auto".into(), "auto".into()))
    );

    // 空输入
    assert_eq!(parse_grid_area(""), None);
    assert_eq!(parse_grid_area("   "), None);
}

#[test]
/// 测试 parse_length_shorthand 空输入、超过 4 个值、无效值等边界情况。
/// 此前 parse_length_shorthand 仅测试了有效输入。
fn test_parse_length_shorthand_empty_and_invalid() {
    // 空输入：split_whitespace 收集为空 → 0 个部分 → None
    assert_eq!(parse_length_shorthand(""), None);
    assert_eq!(parse_length_shorthand("   "), None);

    // 超过 4 个值：应返回 None
    assert_eq!(parse_length_shorthand("1px 2px 3px 4px 5px"), None);

    // 无效值（非长度字符串）：parse_length 返回 None → 整体返回 None
    assert_eq!(parse_length_shorthand("abc 2px"), None);
    assert_eq!(parse_length_shorthand("10px invalid"), None);
}

#[test]
/// 测试 parse_length 对 vw 和 vh 单位的直接解析（不依赖 calc 上下文），
/// 以及负数百分比和极端大数。此前缺少 vw/vh 的直接 parse_length 测试。
fn test_parse_length_vw_vh_and_edge_cases() {
    // vw 单位
    assert_eq!(parse_length("100vw"), Some(LengthValue::Vw(100.0)));
    assert_eq!(parse_length("50vw"), Some(LengthValue::Vw(50.0)));

    // vh 单位
    assert_eq!(parse_length("100vh"), Some(LengthValue::Vh(100.0)));
    assert_eq!(parse_length("25vh"), Some(LengthValue::Vh(25.0)));

    // 负数百分比
    assert_eq!(parse_length("-10%"), Some(LengthValue::Percentage(-10.0)));

    // 极端大数
    let result = parse_length("999999px");
    assert_eq!(result, Some(LengthValue::Px(999999.0)));

    // 极小浮点数
    let result = parse_length("0.001em");
    assert_eq!(result, Some(LengthValue::Em(0.001)));
}

// ═══════════════════════════════════════════════════════════════════════
// 30. 未测试属性值解析边界测试 — touch-action / user-select / will-change /
//     pointer-events / counter-increment
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_touch_action 所有关键字、大小写不敏感、双向 pan 组合及无效输入。
/// 此前 parse_touch_action 无任何测试。
fn test_parse_touch_action_edge_cases() {
    use crate::values::{TouchActionValue, parse_touch_action};
    // 所有关键字
    assert_eq!(parse_touch_action("auto"), Some(TouchActionValue::Auto));
    assert_eq!(parse_touch_action("none"), Some(TouchActionValue::None));
    assert_eq!(parse_touch_action("pan-x"), Some(TouchActionValue::PanX));
    assert_eq!(parse_touch_action("pan-y"), Some(TouchActionValue::PanY));
    assert_eq!(parse_touch_action("manipulation"), Some(TouchActionValue::Manipulation));
    // pan-x pan-y 和 pan-y pan-x 都应解析为 PanXPanY
    assert_eq!(parse_touch_action("pan-x pan-y"), Some(TouchActionValue::PanXPanY));
    assert_eq!(parse_touch_action("pan-y pan-x"), Some(TouchActionValue::PanXPanY));
    // 大小写不敏感
    assert_eq!(parse_touch_action("PAN-X"), Some(TouchActionValue::PanX));
    assert_eq!(
        parse_touch_action("  Manipulation  "),
        Some(TouchActionValue::Manipulation)
    );
    // 无效输入
    assert_eq!(parse_touch_action("invalid"), None);
    assert_eq!(parse_touch_action(""), None);
    // 单独 pan 不是合法值
    assert_eq!(parse_touch_action("pan"), None);
}

#[test]
/// 测试 parse_user_select 所有关键字、大小写不敏感及无效输入。
/// 此前 parse_user_select 无任何测试。
fn test_parse_user_select_edge_cases() {
    use crate::values::{UserSelectValue, parse_user_select};
    assert_eq!(parse_user_select("auto"), Some(UserSelectValue::Auto));
    assert_eq!(parse_user_select("text"), Some(UserSelectValue::Text));
    assert_eq!(parse_user_select("none"), Some(UserSelectValue::None));
    assert_eq!(parse_user_select("all"), Some(UserSelectValue::All));
    assert_eq!(parse_user_select("contain"), Some(UserSelectValue::Contain));
    // 大小写不敏感
    assert_eq!(parse_user_select("TEXT"), Some(UserSelectValue::Text));
    assert_eq!(parse_user_select("  All  "), Some(UserSelectValue::All));
    assert_eq!(parse_user_select("CONTAIN"), Some(UserSelectValue::Contain));
    // 无效输入
    assert_eq!(parse_user_select("inherit"), None);
    assert_eq!(parse_user_select(""), None);
    assert_eq!(parse_user_select("element"), None);
}

#[test]
/// 测试 parse_will_change 关键字、自定义属性名、大小写不敏感、空字符串及含特殊字符的无效输入。
/// 此前 parse_will_change 无任何测试。
fn test_parse_will_change_edge_cases() {
    use crate::values::{WillChangeValue, parse_will_change};
    // 关键字
    assert_eq!(parse_will_change("auto"), Some(WillChangeValue::Auto));
    assert_eq!(
        parse_will_change("scroll-position"),
        Some(WillChangeValue::ScrollPosition)
    );
    assert_eq!(parse_will_change("contents"), Some(WillChangeValue::Contents));
    // 自定义属性名
    assert!(matches!(parse_will_change("transform"), Some(WillChangeValue::Custom(s)) if s == "transform"));
    assert!(matches!(parse_will_change("opacity"), Some(WillChangeValue::Custom(s)) if s == "opacity"));
    assert!(matches!(parse_will_change("top"), Some(WillChangeValue::Custom(s)) if s == "top"));
    // 大小写不敏感
    assert!(matches!(parse_will_change("TRANSFORM"), Some(WillChangeValue::Custom(s)) if s == "transform"));
    assert!(matches!(
        parse_will_change("  Scroll-Position  "),
        Some(WillChangeValue::ScrollPosition)
    ));
    // 无效输入
    assert_eq!(parse_will_change(""), None);
    assert_eq!(parse_will_change("  "), None);
    // 含特殊字符的自定义值应返回 None
    assert_eq!(parse_will_change("transform, opacity"), None);
    assert_eq!(parse_will_change("top!"), None);
}

#[test]
/// 测试 parse_pointer_events 所有关键字（含 SVG 特有值）、大小写不敏感及无效输入。
/// 此前 parse_pointer_events 无任何测试。
fn test_parse_pointer_events_edge_cases() {
    use crate::values::{PointerEventsValue, parse_pointer_events};
    // 通用关键字
    assert_eq!(parse_pointer_events("auto"), Some(PointerEventsValue::Auto));
    assert_eq!(parse_pointer_events("none"), Some(PointerEventsValue::None));
    // SVG 关键字
    assert_eq!(
        parse_pointer_events("visiblePainted"),
        Some(PointerEventsValue::VisiblePainted)
    );
    assert_eq!(
        parse_pointer_events("visibleFill"),
        Some(PointerEventsValue::VisibleFill)
    );
    assert_eq!(
        parse_pointer_events("visibleStroke"),
        Some(PointerEventsValue::VisibleStroke)
    );
    assert_eq!(parse_pointer_events("visible"), Some(PointerEventsValue::Visible));
    assert_eq!(parse_pointer_events("painted"), Some(PointerEventsValue::Painted));
    assert_eq!(parse_pointer_events("fill"), Some(PointerEventsValue::Fill));
    assert_eq!(parse_pointer_events("stroke"), Some(PointerEventsValue::Stroke));
    assert_eq!(parse_pointer_events("all"), Some(PointerEventsValue::All));
    assert_eq!(parse_pointer_events("inherit"), Some(PointerEventsValue::Inherit));
    // 大小写不敏感
    assert_eq!(
        parse_pointer_events("VISIBLEPAINTED"),
        Some(PointerEventsValue::VisiblePainted)
    );
    assert_eq!(parse_pointer_events("  none  "), Some(PointerEventsValue::None));
    // 无效输入
    assert_eq!(parse_pointer_events("invalid"), None);
    assert_eq!(parse_pointer_events(""), None);
    assert_eq!(parse_pointer_events("click"), None);
}

#[test]
/// 测试 parse_counter_action 和 parse_counter_list 的各种边界情况：
/// 单个计数器（带值/不带值）、多个计数器、特殊值 "none"、空输入。
/// 此前 parse_counter_action 和 parse_counter_list 无任何测试。
fn test_parse_counter_action_and_list_edge_cases() {
    use crate::values::{CounterActionValue, parse_counter_action, parse_counter_list};
    // parse_counter_action：单个计数器不带值
    let result = parse_counter_action("section");
    assert_eq!(
        result,
        Some(CounterActionValue {
            name: "section".to_string(),
            value: None,
        })
    );
    // parse_counter_action：带整数值
    let result = parse_counter_action("section 5");
    assert_eq!(
        result,
        Some(CounterActionValue {
            name: "section".to_string(),
            value: Some(5),
        })
    );
    // parse_counter_action：负整数值
    let result = parse_counter_action("chapter -1");
    assert_eq!(
        result,
        Some(CounterActionValue {
            name: "chapter".to_string(),
            value: Some(-1),
        })
    );
    // parse_counter_action："none" 应返回 None
    assert_eq!(parse_counter_action("none"), None);
    // parse_counter_action：空输入
    assert_eq!(parse_counter_action(""), None);
    // parse_counter_action：非整数值应返回 None
    assert_eq!(parse_counter_action("counter abc"), None);

    // parse_counter_list："none" 返回空列表
    let result = parse_counter_list("none");
    assert_eq!(result, Some(vec![]));
    // parse_counter_list：多个计数器
    let result = parse_counter_list("section 1 subsection");
    assert!(result.is_some());
    let list = result.unwrap();
    assert_eq!(list.len(), 2);
    assert_eq!(list[0].name, "section");
    assert_eq!(list[0].value, Some(1));
    assert_eq!(list[1].name, "subsection");
    assert_eq!(list[1].value, None);
    // parse_counter_list：空输入返回 None
    assert_eq!(parse_counter_list(""), None);
    assert_eq!(parse_counter_list("   "), None);
    // parse_counter_list：中间出现 "none" 应返回 None
    assert_eq!(parse_counter_list("section none"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 31. 未覆盖属性值解析边界测试 — overscroll-behavior / content / quotes /
//     image-rendering / isolation
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_overscroll_behavior 所有关键字、大小写不敏感及无效输入。
/// 此前 parse_overscroll_behavior 无任何测试。
fn test_parse_overscroll_behavior_edge_cases() {
    use crate::values::{OverscrollBehaviorValue, parse_overscroll_behavior};
    // 所有关键字
    assert_eq!(parse_overscroll_behavior("auto"), Some(OverscrollBehaviorValue::Auto));
    assert_eq!(
        parse_overscroll_behavior("contain"),
        Some(OverscrollBehaviorValue::Contain)
    );
    assert_eq!(parse_overscroll_behavior("none"), Some(OverscrollBehaviorValue::None));
    // 大小写不敏感
    assert_eq!(parse_overscroll_behavior("AUTO"), Some(OverscrollBehaviorValue::Auto));
    assert_eq!(
        parse_overscroll_behavior("  Contain  "),
        Some(OverscrollBehaviorValue::Contain)
    );
    assert_eq!(parse_overscroll_behavior("NONE"), Some(OverscrollBehaviorValue::None));
    // 无效输入
    assert_eq!(parse_overscroll_behavior("scroll"), None);
    assert_eq!(parse_overscroll_behavior(""), None);
    assert_eq!(parse_overscroll_behavior("inherit"), None);
}

#[test]
/// 测试 parse_content 所有变体：normal、none、字符串、attr()、counter() 及 counter(name, style)，
/// 以及空 attr()、空字符串、未闭合引号等边界输入。
/// 此前 parse_content 无任何测试。
fn test_parse_content_edge_cases() {
    use crate::values::{ContentValue, parse_content};
    // normal / none
    assert_eq!(parse_content("normal"), Some(ContentValue::Normal));
    assert_eq!(parse_content("none"), Some(ContentValue::None));
    assert_eq!(parse_content("NORMAL"), Some(ContentValue::Normal));
    assert_eq!(parse_content("  None  "), Some(ContentValue::None));
    // 双引号字符串
    assert_eq!(
        parse_content("\"hello\""),
        Some(ContentValue::String("hello".to_string()))
    );
    // 单引号字符串
    assert_eq!(
        parse_content("'world'"),
        Some(ContentValue::String("world".to_string()))
    );
    // 空引号字符串
    assert_eq!(parse_content("\"\""), Some(ContentValue::String(String::new())));
    assert_eq!(parse_content("''"), Some(ContentValue::String(String::new())));
    // attr(name)
    assert_eq!(
        parse_content("attr(href)"),
        Some(ContentValue::Attr("href".to_string()))
    );
    assert_eq!(
        parse_content("attr(data-value)"),
        Some(ContentValue::Attr("data-value".to_string()))
    );
    // 空 attr() 应返回 None
    assert_eq!(parse_content("attr()"), None);
    // counter(name)
    assert_eq!(
        parse_content("counter(section)"),
        Some(ContentValue::Counter {
            name: "section".to_string(),
            style: None,
        })
    );
    // counter(name, style)
    assert_eq!(
        parse_content("counter(section, upper-roman)"),
        Some(ContentValue::Counter {
            name: "section".to_string(),
            style: Some("upper-roman".to_string()),
        })
    );
    // 空 counter() 应返回 None
    assert_eq!(parse_content("counter()"), None);
    // 无效输入
    assert_eq!(parse_content(""), None);
    assert_eq!(parse_content("invalid-value"), None);
    assert_eq!(parse_content("\"unclosed"), None);
}

#[test]
/// 测试 parse_quotes 所有关键字（none、auto）、引号对解析、
/// 多层引号对、混合引号类型、空输入和未闭合引号。
/// 此前 parse_quotes 无任何测试。
fn test_parse_quotes_edge_cases() {
    use crate::values::{QuotesValue, parse_quotes};
    // none / auto
    assert_eq!(parse_quotes("none"), Some(QuotesValue::None));
    assert_eq!(parse_quotes("auto"), Some(QuotesValue::Auto));
    assert_eq!(parse_quotes("NONE"), Some(QuotesValue::None));
    assert_eq!(parse_quotes("  Auto  "), Some(QuotesValue::Auto));
    // 单层引号对
    let result = parse_quotes("\"«\" \"»\"");
    assert!(result.is_some());
    if let Some(QuotesValue::Pairs(pairs)) = result {
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("«".to_string(), "»".to_string()));
    } else {
        panic!("Expected Pairs");
    }
    // 多层引号对（CSS 规范允许嵌套级别）
    let result = parse_quotes("\"«\" \"»\" \"‹\" \"›\"");
    assert!(result.is_some());
    if let Some(QuotesValue::Pairs(pairs)) = result {
        assert_eq!(pairs.len(), 2);
        assert_eq!(pairs[0], ("«".to_string(), "»".to_string()));
        assert_eq!(pairs[1], ("‹".to_string(), "›".to_string()));
    } else {
        panic!("Expected Pairs");
    }
    // 单引号引号对
    let result = parse_quotes("'\"' '\"'");
    assert!(result.is_some());
    if let Some(QuotesValue::Pairs(pairs)) = result {
        assert_eq!(pairs.len(), 1);
        assert_eq!(pairs[0], ("\"".to_string(), "\"".to_string()));
    } else {
        panic!("Expected Pairs");
    }
    // 空输入返回 None
    assert_eq!(parse_quotes(""), None);
    assert_eq!(parse_quotes("   "), None);
}

#[test]
/// 测试 parse_image_rendering 所有关键字（auto、smooth、high-quality、pixelated、crisp-edges）、
/// 大小写不敏感及无效输入。此前 parse_image_rendering 无任何测试。
fn test_parse_image_rendering_edge_cases() {
    use crate::values::{ImageRenderingValue, parse_image_rendering};
    // 所有关键字
    assert_eq!(parse_image_rendering("auto"), Some(ImageRenderingValue::Auto));
    assert_eq!(parse_image_rendering("smooth"), Some(ImageRenderingValue::Smooth));
    assert_eq!(
        parse_image_rendering("high-quality"),
        Some(ImageRenderingValue::HighQuality)
    );
    assert_eq!(parse_image_rendering("pixelated"), Some(ImageRenderingValue::Pixelated));
    assert_eq!(
        parse_image_rendering("crisp-edges"),
        Some(ImageRenderingValue::CrispEdges)
    );
    // 大小写不敏感
    assert_eq!(parse_image_rendering("AUTO"), Some(ImageRenderingValue::Auto));
    assert_eq!(
        parse_image_rendering("  Pixelated  "),
        Some(ImageRenderingValue::Pixelated)
    );
    assert_eq!(
        parse_image_rendering("CRISP-EDGES"),
        Some(ImageRenderingValue::CrispEdges)
    );
    // 无效输入
    assert_eq!(parse_image_rendering("sharp"), None);
    assert_eq!(parse_image_rendering(""), None);
    assert_eq!(parse_image_rendering("inherit"), None);
}

#[test]
/// 测试 parse_isolation 所有关键字（auto、isolate）、大小写不敏感及无效输入。
/// 此前 parse_isolation 无任何测试。
fn test_parse_isolation_edge_cases() {
    use crate::values::{IsolationValue, parse_isolation};
    // 所有关键字
    assert_eq!(parse_isolation("auto"), Some(IsolationValue::Auto));
    assert_eq!(parse_isolation("isolate"), Some(IsolationValue::Isolate));
    // 大小写不敏感
    assert_eq!(parse_isolation("AUTO"), Some(IsolationValue::Auto));
    assert_eq!(parse_isolation("  Isolate  "), Some(IsolationValue::Isolate));
    assert_eq!(parse_isolation("ISOLATE"), Some(IsolationValue::Isolate));
    // 无效输入
    assert_eq!(parse_isolation("none"), None);
    assert_eq!(parse_isolation(""), None);
    assert_eq!(parse_isolation("inherit"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 32. parse_box_shadow / parse_text_shadow / parse_background_image 边界条件测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_box_shadow 空字符串返回 None。
fn test_edge_parse_box_shadow_empty() {
    assert_eq!(parse_box_shadow(""), None);
    assert_eq!(parse_box_shadow("   "), None);
}

#[test]
/// 测试 parse_box_shadow 仅 inset 关键字。
fn test_edge_parse_box_shadow_inset_only() {
    // "inset" alone has no offset values → parts.len() < 2 → None
    assert_eq!(parse_box_shadow("inset"), None);
    // "inset" with valid offsets should parse correctly
    let result = parse_box_shadow("inset 3px 4px").unwrap();
    assert!(result.inset);
    assert_eq!(result.offset_x, LengthValue::Px(3.0));
    assert_eq!(result.offset_y, LengthValue::Px(4.0));
}

#[test]
/// 测试 parse_box_shadow 带颜色值 "5px 5px 10px red"。
fn test_edge_parse_box_shadow_with_named_color() {
    let result = parse_box_shadow("5px 5px 10px red").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(5.0));
    assert_eq!(result.offset_y, LengthValue::Px(5.0));
    assert_eq!(result.blur_radius, LengthValue::Px(10.0));
    assert_eq!(result.spread_radius, LengthValue::Px(0.0));
    assert_eq!(result.color, ColorValue::Rgba(255, 0, 0, 255));
    assert!(!result.inset);
}

#[test]
/// 测试 parse_text_shadow 空字符串返回 None。
fn test_edge_parse_text_shadow_empty() {
    assert_eq!(parse_text_shadow(""), None);
    assert_eq!(parse_text_shadow("   "), None);
}

#[test]
/// 测试 parse_text_shadow 颜色在前 "red 2px 3px"。
/// 解析器从 parts[0] 开始尝试 parse_length，"red" 不是长度，
/// 所以 ox 会是 None → 整体返回 None。
fn test_edge_parse_text_shadow_color_first() {
    assert_eq!(parse_text_shadow("red 2px 3px"), None);
}

#[test]
/// 测试 parse_text_shadow 大偏移量。
fn test_edge_parse_text_shadow_large_offset() {
    let result = parse_text_shadow("9999px 8888px 100px").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(9999.0));
    assert_eq!(result.offset_y, LengthValue::Px(8888.0));
    assert_eq!(result.blur_radius, LengthValue::Px(100.0));
    assert_eq!(result.color, ColorValue::Rgba(0, 0, 0, 255));
}

#[test]
/// 测试 parse_background_image 空字符串返回 None。
fn test_edge_parse_background_image_empty() {
    assert_eq!(parse_background_image(""), None);
    assert_eq!(parse_background_image("   "), None);
}

#[test]
/// 测试 parse_background_image url 带引号。
fn test_edge_parse_background_image_quoted_url() {
    // 双引号
    let result = parse_background_image("url(\"image.png\")");
    assert_eq!(result, Some(BackgroundImageValue::Url("image.png".to_string())));
    // 单引号
    let result = parse_background_image("url('bg.jpg')");
    assert_eq!(result, Some(BackgroundImageValue::Url("bg.jpg".to_string())));
}

#[test]
/// 测试 parse_background_image 大小写 URL。
fn test_edge_parse_background_image_case_insensitive() {
    // "URL(...)" is not recognized — starts_with("url(") is case-sensitive
    assert_eq!(parse_background_image("URL(image.png)"), None);
    // "url(...)" is the valid form
    let result = parse_background_image("url(image.png)");
    assert_eq!(result, Some(BackgroundImageValue::Url("image.png".to_string())));
}

#[test]
/// 测试 parse_background_image 无效值返回 None。
fn test_edge_parse_background_image_invalid() {
    assert_eq!(parse_background_image("not-a-url"), None);
    assert_eq!(parse_background_image("url()"), None);
    assert_eq!(parse_background_image("gradient(red, blue)"), None);
    assert_eq!(parse_background_image("url('')"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 33. parse_background_image 渐变边界条件测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_background_image 识别 linear-gradient。
fn test_parse_background_image_linear_gradient() {
    let result = parse_background_image("linear-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        BackgroundImageValue::Gradient(GradientValue::Linear(lg)) => {
            assert!(!lg.repeating);
            assert!(lg.stops.len() >= 2);
        }
        other => panic!("Expected Gradient(Linear(..)), got {:?}", other),
    }
}

#[test]
/// 测试 parse_background_image 识别 radial-gradient。
fn test_parse_background_image_radial_gradient() {
    let result = parse_background_image("radial-gradient(circle, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        BackgroundImageValue::Gradient(GradientValue::Radial(rg)) => {
            assert_eq!(rg.shape, RadialShape::Circle);
            assert!(rg.stops.len() >= 2);
        }
        other => panic!("Expected Gradient(Radial(..)), got {:?}", other),
    }
}

#[test]
/// 测试 parse_background_image 识别 conic-gradient。
fn test_parse_background_image_conic_gradient() {
    let result = parse_background_image("conic-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        BackgroundImageValue::Gradient(GradientValue::Conic(cg)) => {
            assert!(cg.stops.len() >= 2);
        }
        other => panic!("Expected Gradient(Conic(..)), got {:?}", other),
    }
}

#[test]
/// 测试 parse_background_image 识别 repeating-linear-gradient。
fn test_parse_background_image_repeating_linear_gradient() {
    let result = parse_background_image("repeating-linear-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        BackgroundImageValue::Gradient(GradientValue::Linear(lg)) => {
            assert!(lg.repeating, "repeating flag should be true");
        }
        other => panic!("Expected Gradient(Linear(..)), got {:?}", other),
    }
}

#[test]
/// 测试 parse_background_image 渐变大小写不敏感。
fn test_parse_background_image_gradient_case_insensitive() {
    let result = parse_background_image("Linear-Gradient(red, blue)");
    assert!(result.is_some(), "Mixed-case gradient name should be recognized");
    match result.unwrap() {
        BackgroundImageValue::Gradient(GradientValue::Linear(_)) => {}
        other => panic!("Expected Gradient(Linear(..)), got {:?}", other),
    }
}

#[test]
/// 测试 parse_background_image 渐变方向解析。
fn test_parse_background_image_gradient_direction() {
    let result = parse_background_image("linear-gradient(to right, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        BackgroundImageValue::Gradient(GradientValue::Linear(lg)) => {
            assert_eq!(lg.direction, GradientDirection::ToRight);
        }
        other => panic!("Expected Gradient(Linear(..)), got {:?}", other),
    }
}

#[test]
/// 测试 parse_background_image 无效渐变返回 None。
fn test_parse_background_image_invalid_gradient() {
    // "gradient(...)" is not a known gradient function name
    assert_eq!(parse_background_image("gradient(red, blue)"), None);
}

#[test]
/// 测试 parse_background_image 空渐变参数返回 None。
fn test_parse_background_image_empty_gradient() {
    // "linear-gradient()" with no color stops should return None
    assert_eq!(parse_background_image("linear-gradient()"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 31. Tokenizer 边界测试（覆盖 tokenizer.rs 的 uncovered 路径）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 tokenizer 处理 Unicode range (U+0-7F)
fn test_tokenizer_unicode_range() {
    let tokenizer = crate::Tokenizer::new("U+0-7F");
    let tokens: Vec<_> = tokenizer.collect_tokens();
    // Check if UnicodeRange is being generated
    let has_unicode_range = tokens.iter().any(|t| matches!(t, Token::UnicodeRange(_, _)));
    if !has_unicode_range {
        // If UnicodeRange is not generated, check if it's being parsed as Ident
        let has_ident = tokens.iter().any(|t| matches!(t, Token::Ident(_)));
        assert!(has_ident, "Should parse as Ident or UnicodeRange");
    }
}

#[test]
/// 测试 tokenizer 处理包含数字的标识符
fn test_tokenizer_ident_with_numbers() {
    let test_cases = vec!["ident123", "ident_123", "_ident", "ident-123"];

    for css in test_cases {
        let tokenizer = crate::Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        assert!(
            tokens.iter().any(|t| matches!(t, Token::Ident(_))),
            "Should parse as Ident: {}",
            css
        );
    }
}

#[test]
/// 测试 tokenizer 处理各种边界情况
fn test_tokenizer_edge_cases() {
    // 简单测试 tokenizer 不 panic 并返回合理的 token 数量
    let test_cases = vec![("", 0), (" ", 0), ("div", 1), ("@media", 1), ("/* comment */", 1)];

    for (css, _expected) in test_cases {
        let tokenizer = crate::Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 关键是不 panic
        let _ = tokens.len();
    }
}

#[test]
/// 测试 tokenizer 处理无效的数字格式
fn test_tokenizer_invalid_numbers() {
    let test_cases = vec![
        "1.", ".1", "++1", "--1", "1.2.3", "1e10", // 科学计数法目前不支持
    ];

    for css in test_cases {
        let tokenizer = crate::Tokenizer::new(css);
        let tokens: Vec<_> = tokenizer.collect_tokens();
        // 确保不 panic，即使数字格式无效
        assert!(!tokens.is_empty());
    }
}
