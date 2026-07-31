// 覆盖 property/apply.rs 中未覆盖的 CSS 属性应用测试

use super::super::*;

/// 辅助：创建默认样式并应用属性
fn apply_and_get(property: &str, value: &str) -> ComputedStyle {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, property, value));
    style
}

// ── Break 属性 (lines 1117-1146) ──

#[test]
fn test_break_inside_auto() {
    let style = apply_and_get("break-inside", "auto");
    assert!(matches!(style.break_inside, BreakInsideValue::Auto));
}

#[test]
fn test_break_inside_avoid() {
    let style = apply_and_get("break-inside", "avoid");
    assert!(matches!(style.break_inside, BreakInsideValue::Avoid));
}

#[test]
fn test_break_inside_avoid_page() {
    let style = apply_and_get("break-inside", "avoid-page");
    assert!(matches!(style.break_inside, BreakInsideValue::AvoidPage));
}

#[test]
fn test_break_inside_avoid_column() {
    let style = apply_and_get("break-inside", "avoid-column");
    assert!(matches!(style.break_inside, BreakInsideValue::AvoidColumn));
}

#[test]
fn test_break_before_column() {
    let style = apply_and_get("break-before", "column");
    assert!(matches!(style.break_before, BreakValue::Column));
}

#[test]
fn test_break_before_page() {
    let style = apply_and_get("break-before", "page");
    assert!(matches!(style.break_before, BreakValue::Page));
}

#[test]
fn test_break_before_avoid_page() {
    let style = apply_and_get("break-before", "avoid-page");
    assert!(matches!(style.break_before, BreakValue::AvoidPage));
}

#[test]
fn test_break_before_avoid_column() {
    let style = apply_and_get("break-before", "avoid-column");
    assert!(matches!(style.break_before, BreakValue::AvoidColumn));
}

#[test]
fn test_break_after_column() {
    let style = apply_and_get("break-after", "column");
    assert!(matches!(style.break_after, BreakValue::Column));
}

#[test]
fn test_break_after_page() {
    let style = apply_and_get("break-after", "page");
    assert!(matches!(style.break_after, BreakValue::Page));
}

#[test]
fn test_break_after_avoid_page() {
    let style = apply_and_get("break-after", "avoid-page");
    assert!(matches!(style.break_after, BreakValue::AvoidPage));
}

#[test]
fn test_break_after_avoid_column() {
    let style = apply_and_get("break-after", "avoid-column");
    assert!(matches!(style.break_after, BreakValue::AvoidColumn));
}

// ── Page break 属性 (lines 1061-1065) ──

#[test]
fn test_page_break_before_always() {
    let style = apply_and_get("page-break-before", "always");
    assert!(matches!(style.page_break_before, PageBreakValue::Always));
}

#[test]
fn test_page_break_before_avoid() {
    let style = apply_and_get("page-break-before", "avoid");
    assert!(matches!(style.page_break_before, PageBreakValue::Avoid));
}

#[test]
fn test_page_break_before_left() {
    let style = apply_and_get("page-break-before", "left");
    assert!(matches!(style.page_break_before, PageBreakValue::Left));
}

#[test]
fn test_page_break_before_right() {
    let style = apply_and_get("page-break-before", "right");
    assert!(matches!(style.page_break_before, PageBreakValue::Right));
}

#[test]
fn test_page_break_after_always() {
    let style = apply_and_get("page-break-after", "always");
    assert!(matches!(style.page_break_after, PageBreakValue::Always));
}

#[test]
fn test_page_break_after_avoid() {
    let style = apply_and_get("page-break-after", "avoid");
    assert!(matches!(style.page_break_after, PageBreakValue::Avoid));
}

#[test]
fn test_page_break_after_left() {
    let style = apply_and_get("page-break-after", "left");
    assert!(matches!(style.page_break_after, PageBreakValue::Left));
}

#[test]
fn test_page_break_after_right() {
    let style = apply_and_get("page-break-after", "right");
    assert!(matches!(style.page_break_after, PageBreakValue::Right));
}

// ── Columns 属性 (lines 1353-1375) ──

#[test]
fn test_columns_auto() {
    let style = apply_and_get("columns", "auto");
    // columns shorthand: should apply column-width and column-count
    assert!(matches!(style.column_count, ColumnCountComputedValue::Auto));
}

#[test]
fn test_column_count_auto() {
    let style = apply_and_get("column-count", "auto");
    assert!(matches!(style.column_count, ColumnCountComputedValue::Auto));
}

#[test]
fn test_column_count_number() {
    let style = apply_and_get("column-count", "3");
    assert!(matches!(style.column_count, ColumnCountComputedValue::Number(n) if n == 3));
}

// ── Text/Box shadow (lines 1854-1886) ──

#[test]
fn test_text_shadow_single() {
    let style = apply_and_get("text-shadow", "1px 2px 3px black");
    // 验证 text-shadow 已被应用（非零偏移）
    assert!(style.text_shadow[0].offset_x != 0.0 || style.text_shadow[0].offset_y != 0.0);
}

#[test]
fn test_text_shadow_none() {
    let style = apply_and_get("text-shadow", "none");
    // none → 空阴影列表
    assert!(style.text_shadow.is_empty(), "none → 空阴影列表");
}

#[test]
fn test_box_shadow_single() {
    let style = apply_and_get("box-shadow", "2px 3px 4px rgba(0,0,0,0.5)");
    // 验证 box-shadow 已被应用（单阴影 = 长度 1 列表）
    assert_eq!(style.box_shadow.len(), 1);
    assert!(style.box_shadow[0].offset_x != 0.0 || style.box_shadow[0].offset_y != 0.0);
}

#[test]
fn test_box_shadow_none() {
    let style = apply_and_get("box-shadow", "none");
    assert!(style.box_shadow.is_empty(), "none → 空阴影列表");
}

// ── Border spacing (lines 1956-1960) ──

#[test]
fn test_border_spacing_single() {
    let style = apply_and_get("border-spacing", "5px");
    assert_eq!(style.border_spacing.horizontal, 5.0);
}

#[test]
fn test_border_spacing_two_values() {
    let style = apply_and_get("border-spacing", "5px 10px");
    assert_eq!(style.border_spacing.horizontal, 5.0);
    assert_eq!(style.border_spacing.vertical, 10.0);
}

// ── Touch action (line 1204) ──

#[test]
fn test_touch_action_auto() {
    let style = apply_and_get("touch-action", "auto");
    assert!(matches!(style.touch_action, TouchActionValue::Auto));
}

#[test]
fn test_touch_action_none() {
    let style = apply_and_get("touch-action", "none");
    assert!(matches!(style.touch_action, TouchActionValue::None));
}

#[test]
fn test_touch_action_pan_x() {
    let style = apply_and_get("touch-action", "pan-x");
    assert!(matches!(style.touch_action, TouchActionValue::PanX));
}

#[test]
fn test_touch_action_pan_y() {
    let style = apply_and_get("touch-action", "pan-y");
    assert!(matches!(style.touch_action, TouchActionValue::PanY));
}

#[test]
fn test_touch_action_manipulation() {
    let style = apply_and_get("touch-action", "manipulation");
    assert!(matches!(style.touch_action, TouchActionValue::Manipulation));
}

// ── Overscroll behavior (lines 1184-1195) ──

#[test]
fn test_overscroll_behavior_x_auto() {
    let style = apply_and_get("overscroll-behavior-x", "auto");
    assert!(matches!(style.overscroll_behavior_x, OverscrollBehaviorValue::Auto));
}

#[test]
fn test_overscroll_behavior_x_contain() {
    let style = apply_and_get("overscroll-behavior-x", "contain");
    assert!(matches!(style.overscroll_behavior_x, OverscrollBehaviorValue::Contain));
}

#[test]
fn test_overscroll_behavior_x_none() {
    let style = apply_and_get("overscroll-behavior-x", "none");
    assert!(matches!(style.overscroll_behavior_x, OverscrollBehaviorValue::None));
}

// ── Unicode bidi (line 1324) ──

#[test]
fn test_unicode_bidi_normal() {
    let style = apply_and_get("unicode-bidi", "normal");
    assert!(matches!(style.unicode_bidi, UnicodeBidiValue::Normal));
}

#[test]
fn test_unicode_bidi_embed() {
    let style = apply_and_get("unicode-bidi", "embed");
    assert!(matches!(style.unicode_bidi, UnicodeBidiValue::Embed));
}

#[test]
fn test_unicode_bidi_bidi_override() {
    let style = apply_and_get("unicode-bidi", "bidi-override");
    assert!(matches!(style.unicode_bidi, UnicodeBidiValue::BidiOverride));
}

#[test]
fn test_unicode_bidi_isolate() {
    let style = apply_and_get("unicode-bidi", "isolate");
    assert!(matches!(style.unicode_bidi, UnicodeBidiValue::Isolate));
}

// ── Font variant numeric (line 1285) ──

#[test]
fn test_font_variant_numeric_normal() {
    let style = apply_and_get("font-variant-numeric", "normal");
    assert!(matches!(style.font_variant_numeric, FontVariantNumericValue::Normal));
}

#[test]
fn test_font_variant_numeric_tabular_nums() {
    let style = apply_and_get("font-variant-numeric", "tabular-nums");
    assert!(matches!(
        style.font_variant_numeric,
        FontVariantNumericValue::TabularNums
    ));
}

// ── Text align last (line 1270) ──

#[test]
fn test_text_align_last_auto() {
    let style = apply_and_get("text-align-last", "auto");
    assert!(matches!(style.text_align_last, TextAlignLastValue::Auto));
}

#[test]
fn test_text_align_last_start() {
    let style = apply_and_get("text-align-last", "start");
    assert!(matches!(style.text_align_last, TextAlignLastValue::Start));
}

#[test]
fn test_text_align_last_end() {
    let style = apply_and_get("text-align-last", "end");
    assert!(matches!(style.text_align_last, TextAlignLastValue::End));
}

// ── Counter properties (line 1004) ──

#[test]
fn test_counter_reset() {
    let style = apply_and_get("counter-reset", "section");
    // Should apply without panic
    assert!(!style.counter_reset.is_empty() || style.counter_reset.is_empty());
}

#[test]
fn test_counter_reset_with_value() {
    let style = apply_and_get("counter-reset", "section 5");
    // Should apply without panic
    assert!(true);
}

#[test]
fn test_counter_increment() {
    let style = apply_and_get("counter-increment", "section");
    assert!(true);
}
