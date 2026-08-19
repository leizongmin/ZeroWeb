//! apply_property_value 分支覆盖率测试（第五轮补全）
//!
//! 覆盖 apply.rs 中无效值 fall-through、background-position TwoValue、
//! border-image 非 Px 分支、columns 简写、filter 函数等未覆盖分支。

use crate::ComputedStyle;
use crate::property::apply::apply_property_value;
use crate::property::types::*;

/// 辅助：创建空的 ComputedStyle 并应用单个属性
fn apply(prop: &str, value: &str) -> (bool, ComputedStyle) {
    let mut style = ComputedStyle::default();
    let ok = apply_property_value(&mut style, prop, value);
    (ok, style)
}

// ═══════════════════════════════════════════════════════════
// 第五轮覆盖率补全：覆盖 apply.rs 中未覆盖的分支
// ═══════════════════════════════════════════════════════════

// === 无效值 fall-through 路径（覆盖各属性的 false 返回）===

#[test]
fn test_apply_invalid_layout_values() {
    // 无效的 display 值
    let (ok, _) = apply("display", "invalid-display");
    assert!(!ok, "invalid display should return false");
    // 无效的 position
    let (ok, _) = apply("position", "nowhere");
    assert!(!ok);
    // 无效的 float
    let (ok, _) = apply("float", "center");
    assert!(!ok);
    // 无效的 clear
    let (ok, _) = apply("clear", "top");
    assert!(!ok);
    // 无效的 box-sizing
    let (ok, _) = apply("box-sizing", "content");
    assert!(!ok);
}

#[test]
fn test_apply_invalid_dimension_values() {
    // 无效的 width/height
    let (ok, _) = apply("width", "abc");
    assert!(!ok);
    let (ok, _) = apply("height", "abc");
    assert!(!ok);
    let (ok, _) = apply("min-width", "abc");
    assert!(!ok);
    let (ok, _) = apply("min-height", "abc");
    assert!(!ok);
}

#[test]
fn test_apply_invalid_margin_padding() {
    for prop in [
        "margin-top",
        "margin-right",
        "margin-bottom",
        "margin-left",
        "padding-top",
        "padding-right",
        "padding-bottom",
        "padding-left",
    ] {
        let (ok, _) = apply(prop, "not-a-length");
        assert!(!ok, "{} with invalid value should return false", prop);
    }
}

#[test]
fn test_apply_invalid_border_properties() {
    // 注：thin/medium/thick 是合法 border-width 关键字（CSS §8.5.1），不再作为非法值。
    let (ok, _) = apply("border-top-color", "not-a-color");
    assert!(!ok);
    let (ok, _) = apply("border-top-style", "dotted-solid");
    assert!(!ok);
    let (ok, _) = apply("border-top-left-radius", "huge");
    assert!(!ok);
}

#[test]
fn test_apply_invalid_text_properties() {
    let (ok, _) = apply("text-align", "diagonal");
    assert!(!ok);
    let (ok, _) = apply("text-decoration", "blink-underline");
    assert!(!ok);
    let (ok, _) = apply("text-transform", "mirror");
    assert!(!ok);
    let (ok, _) = apply("white-space", "nowrap-wrap");
    assert!(!ok);
    let (ok, _) = apply("word-break", "hyphenate");
    assert!(!ok);
    let (ok, _) = apply("writing-mode", "sideways");
    assert!(!ok);
}

#[test]
fn test_apply_invalid_flex_grid() {
    let (ok, _) = apply("flex-direction", "diagonal");
    assert!(!ok);
    let (ok, _) = apply("flex-wrap", "nowrap-wrap");
    assert!(!ok);
    let (ok, _) = apply("justify-content", "space-between-center");
    assert!(!ok);
    let (ok, _) = apply("align-items", "baseline-center");
    assert!(!ok);
    let (ok, _) = apply("flex-grow", "abc");
    assert!(!ok);
    let (ok, _) = apply("flex-shrink", "abc");
    assert!(!ok);
    let (ok, _) = apply("order", "abc");
    assert!(!ok);
    let (ok, _) = apply("grid-auto-flow", "dense-column-row");
    assert!(!ok);
}

#[test]
fn test_apply_invalid_positioning() {
    let (ok, _) = apply("top", "abc");
    assert!(!ok);
    let (ok, _) = apply("right", "abc");
    assert!(!ok);
    let (ok, _) = apply("bottom", "abc");
    assert!(!ok);
    let (ok, _) = apply("left", "abc");
    assert!(!ok);
    let (ok, _) = apply("z-index", "abc");
    assert!(!ok);
    let (ok, _) = apply("overflow-x", "scroll-visible");
    assert!(!ok);
}

// === columns 简写详细覆盖 ===

#[test]
fn test_apply_columns_shorthand_count_first() {
    // count+width 顺序
    let (ok, _s) = apply("columns", "3 200px");
    assert!(ok);
    // 3 → column-count, 200px → column-width
}

#[test]
fn test_apply_columns_shorthand_width_first() {
    // width+count 顺序
    let (ok, _s) = apply("columns", "200px 3");
    assert!(ok);
}

#[test]
fn test_apply_columns_single_width() {
    let (ok, _) = apply("columns", "150px");
    assert!(ok);
}

#[test]
fn test_apply_columns_single_count() {
    let (ok, _) = apply("columns", "5");
    assert!(ok);
}

#[test]
fn test_apply_columns_invalid() {
    let (ok, _) = apply("columns", "abc def");
    assert!(!ok);
    let (ok, _) = apply("columns", "");
    assert!(!ok);
}

#[test]
fn test_apply_columns_invalid_pair_keeps_old_values() {
    let mut style = ComputedStyle::default();
    style.column_count = ColumnCountComputedValue::Number(2);
    style.column_width = ColumnWidthComputedValue::Length(LengthValue::Px(100.0));

    assert!(!apply_property_value(&mut style, "columns", "3 bogus"));
    assert_eq!(style.column_count, ColumnCountComputedValue::Number(2));
    assert_eq!(
        style.column_width,
        ColumnWidthComputedValue::Length(LengthValue::Px(100.0))
    );

    assert!(!apply_property_value(&mut style, "columns", "150px bogus"));
    assert_eq!(style.column_count, ColumnCountComputedValue::Number(2));
    assert_eq!(
        style.column_width,
        ColumnWidthComputedValue::Length(LengthValue::Px(100.0))
    );
}

#[test]
fn test_apply_columns_zero_is_width_not_count() {
    // CSS Multicol §3.2：column-count 须为正整数；0 非法。
    // 故 `columns: 0` 的单值 0 须归 column-width（zero-column-width-layout 第二 div），
    // 不可归 column-count（旧逻辑 `parse::<u32>().is_ok()` 误把 0 当 count）。
    use crate::property::types::{ColumnCountComputedValue, ColumnWidthComputedValue, LengthValue};
    let (ok, s) = apply("columns", "0");
    assert!(ok);
    assert!(
        matches!(s.column_count, ColumnCountComputedValue::Auto),
        "columns:0 must NOT set column-count (0 is not a positive integer)"
    );
    assert!(
        matches!(s.column_width, ColumnWidthComputedValue::Length(LengthValue::Px(0.0))),
        "columns:0 must set column-width to 0px"
    );
}

#[test]
fn test_apply_column_count_individual() {
    let (ok, _) = apply("column-count", "4");
    assert!(ok);
    let (ok, _) = apply("column-count", "auto");
    assert!(ok);
    let (ok, _) = apply("column-count", "abc");
    assert!(!ok);
}

#[test]
fn test_apply_column_width_individual() {
    let (ok, _) = apply("column-width", "100px");
    assert!(ok);
    let (ok, _) = apply("column-width", "auto");
    assert!(ok);
    let (ok, _) = apply("column-width", "abc");
    assert!(!ok);

    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "column-width", "120px"));
    let previous = style.column_width.clone();
    assert!(!apply_property_value(&mut style, "column-width", "-1px"));
    assert_eq!(style.column_width, previous);
    assert!(!apply_property_value(&mut style, "column-width", "50%"));
    assert_eq!(style.column_width, previous);
}

// === background-position TwoValue 分支 ===

#[test]
fn test_apply_background_position_two_value() {
    // TwoValue(left, top)
    let (ok, _) = apply("background-position", "left top");
    assert!(ok);
    // TwoValue(center, center)
    let (ok, _) = apply("background-position", "center center");
    assert!(ok);
    // TwoValue with lengths
    let (ok, _) = apply("background-position", "10px 20px");
    assert!(ok);
    // TwoValue with percent
    let (ok, _) = apply("background-position", "50% 75%");
    assert!(ok);
    // TwoValue with mixed
    let (ok, _) = apply("background-position", "left 50%");
    assert!(ok);
    let (ok, _) = apply("background-position", "right bottom");
    assert!(ok);
}

#[test]
fn test_apply_background_position_single() {
    let (ok, _) = apply("background-position", "center");
    assert!(ok);
    let (ok, _) = apply("background-position", "left");
    assert!(ok);
    let (ok, _) = apply("background-position", "right");
    assert!(ok);
    let (ok, _) = apply("background-position", "top");
    assert!(ok);
    let (ok, _) = apply("background-position", "bottom");
    assert!(ok);
    let (ok, _) = apply("background-position", "100px");
    assert!(ok);
    let (ok, _) = apply("background-position", "50%");
    assert!(ok);
}

// === background-size/attachment/clip/origin 变体 ===

#[test]
fn test_apply_background_size_variants() {
    let (ok, _) = apply("background-size", "auto");
    assert!(ok);
    let (ok, _) = apply("background-size", "cover");
    assert!(ok);
    let (ok, _) = apply("background-size", "contain");
    assert!(ok);
    let (ok, _) = apply("background-size", "200px");
    assert!(ok);
    let (ok, _) = apply("background-size", "50%");
    assert!(ok);
}

#[test]
fn test_apply_background_size_rejects_invalid_consumer_grammar() {
    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "background-size", "cover"));
    let previous = style.background_size.clone();

    for value in ["-1px", "-50%", "thin", "auto -1px", "100% thin"] {
        assert!(!apply_property_value(&mut style, "background-size", value));
        assert_eq!(style.background_size, previous, "{} should not overwrite", value);
    }
}

#[test]
fn test_apply_background_attachment_variants() {
    for v in ["scroll", "fixed", "local"] {
        let (ok, _) = apply("background-attachment", v);
        assert!(ok, "background-attachment: {} should apply", v);
    }
}

#[test]
fn test_apply_background_clip_variants() {
    for v in ["border-box", "padding-box", "content-box", "text"] {
        let (ok, _) = apply("background-clip", v);
        assert!(ok, "background-clip: {} should apply", v);
    }
}

#[test]
fn test_apply_background_origin_variants() {
    for v in ["padding-box", "border-box", "content-box"] {
        let (ok, _) = apply("background-origin", v);
        assert!(ok, "background-origin: {} should apply", v);
    }
}

// === border-image 详细分支覆盖 ===

#[test]
fn test_apply_border_image_slice_with_fill() {
    let (ok, _) = apply("border-image-slice", "30% fill");
    assert!(ok);
}

#[test]
fn test_apply_border_image_slice_four_values() {
    let (ok, _) = apply("border-image-slice", "10 20 30 40");
    assert!(ok);
}

#[test]
fn test_apply_border_image_width_auto() {
    let (ok, _) = apply("border-image-width", "auto");
    assert!(ok);
}

#[test]
fn test_apply_border_image_width_number() {
    let (ok, _) = apply("border-image-width", "2");
    assert!(ok);
}

#[test]
fn test_apply_border_image_width_percent() {
    let (ok, _) = apply("border-image-width", "10%");
    assert!(ok);
}

#[test]
fn test_apply_border_image_width_px() {
    let (ok, _) = apply("border-image-width", "5px");
    assert!(ok);
}

#[test]
fn test_apply_border_image_width_four_values() {
    let (ok, _) = apply("border-image-width", "1 2 3 4");
    assert!(ok);
}

#[test]
fn test_apply_border_image_repeat_variants() {
    for v in ["stretch", "repeat", "round", "space"] {
        let (ok, _) = apply("border-image-repeat", v);
        assert!(ok, "border-image-repeat: {} should apply", v);
    }
    // 两个值
    let (ok, _) = apply("border-image-repeat", "stretch repeat");
    assert!(ok);
}

#[test]
fn test_apply_border_image_outset_number() {
    let (ok, _) = apply("border-image-outset", "2");
    assert!(ok);
}

#[test]
fn test_apply_border_image_outset_px() {
    let (ok, _) = apply("border-image-outset", "5px");
    assert!(ok);
}

#[test]
fn test_apply_border_image_outset_four_values() {
    let (ok, _) = apply("border-image-outset", "1 2 3 4");
    assert!(ok);
}

#[test]
fn test_apply_border_image_outset_non_px() {
    // 触发 _ => Number(0.0) 分支
    let (ok, _) = apply("border-image-outset", "2em");
    assert!(ok);
}

#[test]
fn test_apply_border_image_source_variants() {
    let (ok, _) = apply("border-image-source", "none");
    assert!(ok);
    let (ok, _) = apply("border-image-source", "url(border.png)");
    assert!(ok);
}

// === box-shadow / text-shadow 边界值 ===

#[test]
fn test_apply_box_shadow_full() {
    // 带所有值的 box-shadow（含 spread 和 inset）
    let (ok, _) = apply("box-shadow", "5px 10px 3px 2px red inset");
    assert!(ok);
}

#[test]
fn test_apply_box_shadow_no_spread() {
    let (ok, _) = apply("box-shadow", "1px 2px 3px blue");
    assert!(ok);
}

#[test]
fn test_apply_text_shadow_offset_only() {
    let (ok, _) = apply("text-shadow", "2px 3px");
    assert!(ok);
}

#[test]
fn test_apply_text_shadow_with_color() {
    let (ok, _) = apply("text-shadow", "1px 1px 2px black");
    assert!(ok);
}

// === border-spacing 变体 ===

#[test]
fn test_apply_border_spacing_two_values() {
    let (ok, _) = apply("border-spacing", "5px 10px");
    assert!(ok);
}

#[test]
fn test_apply_border_spacing_single_value() {
    let (ok, _) = apply("border-spacing", "8px");
    assert!(ok);
}

// === transition-timing-function 详细分支 ===

#[test]
fn test_apply_transition_timing_function_variants() {
    let (ok, _) = apply("transition-timing-function", "ease");
    assert!(ok);
    let (ok, _) = apply("transition-timing-function", "linear");
    assert!(ok);
    let (ok, _) = apply("transition-timing-function", "ease-in");
    assert!(ok);
    let (ok, _) = apply("transition-timing-function", "ease-out");
    assert!(ok);
    let (ok, _) = apply("transition-timing-function", "ease-in-out");
    assert!(ok);
    // steps 函数
    let (ok, _) = apply("transition-timing-function", "steps(4)");
    assert!(ok);
    // cubic-bezier 函数
    let (ok, _) = apply("transition-timing-function", "cubic-bezier(0.1, 0.7, 1.0, 0.1)");
    assert!(ok);
}

#[test]
fn test_apply_timing_function_invalid_lists_keep_old_values() {
    let mut style = ComputedStyle::default();
    style.transition_timing_function = vec![zero_css_parser::values::TimingFunctionValue::Linear];
    style.animation_timing_function = vec![zero_css_parser::values::TimingFunctionValue::Linear];

    assert!(!apply_property_value(
        &mut style,
        "transition-timing-function",
        "ease, bogus"
    ));
    assert_eq!(
        style.transition_timing_function,
        vec![zero_css_parser::values::TimingFunctionValue::Linear]
    );

    assert!(!apply_property_value(&mut style, "animation-timing-function", "ease,"));
    assert_eq!(
        style.animation_timing_function,
        vec![zero_css_parser::values::TimingFunctionValue::Linear]
    );
}

// === animation 属性变体 ===

#[test]
fn test_apply_animation_direction_variants() {
    for v in ["normal", "reverse", "alternate", "alternate-reverse"] {
        let (ok, _) = apply("animation-direction", v);
        assert!(ok, "animation-direction: {} should apply", v);
    }
    let (ok, _) = apply("animation-direction", "normal, bogus");
    assert!(!ok);
}

#[test]
fn test_apply_animation_fill_mode_variants() {
    for v in ["none", "forwards", "backwards", "both"] {
        let (ok, _) = apply("animation-fill-mode", v);
        assert!(ok, "animation-fill-mode: {} should apply", v);
    }
    let (ok, _) = apply("animation-fill-mode", "both, bogus");
    assert!(!ok);
}

#[test]
fn test_apply_animation_play_state_variants() {
    let (ok, _) = apply("animation-play-state", "running");
    assert!(ok);
    let (ok, _) = apply("animation-play-state", "paused");
    assert!(ok);
    let (ok, _) = apply("animation-play-state", "running, bogus");
    assert!(!ok);
}

#[test]
fn test_apply_animation_timing_function() {
    let (ok, _) = apply("animation-timing-function", "ease-in-out");
    assert!(ok);
}

#[test]
fn test_apply_animation_delay() {
    let (ok, _) = apply("animation-delay", "0.5s");
    assert!(ok);
    let (ok, s) = apply("animation-delay", "-0.25s");
    assert!(ok);
    assert_eq!(s.animation_delay, vec![-0.25]);
    let (ok, _) = apply("animation-delay", "100ms, bogus");
    assert!(!ok);
}

// === perspective-origin 双值 ===

#[test]
fn test_apply_perspective_origin_two_values() {
    let (ok, _) = apply("perspective-origin", "100px 50px");
    assert!(ok);
    let (ok, _) = apply("perspective-origin", "25% 75%");
    assert!(ok);
}

#[test]
fn test_apply_perspective_origin_single_value() {
    let (ok, _) = apply("perspective-origin", "50%");
    assert!(ok);
    let (ok, _) = apply("perspective-origin", "100px");
    assert!(ok);
}

// === transform-origin 双值 ===

#[test]
fn test_apply_transform_origin_two_values() {
    let (ok, _) = apply("transform-origin", "100px 50px");
    assert!(ok);
    let (ok, _) = apply("transform-origin", "25% 75%");
    assert!(ok);
}

// === page-break-inside 非法值 ===

#[test]
fn test_apply_page_break_inside_invalid() {
    // page-break-inside 只接受 auto 和 avoid，其他值应 return false
    let (ok, _) = apply("page-break-inside", "always");
    assert!(!ok, "page-break-inside: always should return false");
    let (ok, _) = apply("page-break-inside", "left");
    assert!(!ok);
    let (ok, _) = apply("page-break-inside", "right");
    assert!(!ok);
}

// === justify-items / justify-self / align-content 非法值 ===

#[test]
fn test_apply_justify_items_invalid() {
    let (ok, _) = apply("justify-items", "invalid-value");
    assert!(!ok);
}

#[test]
fn test_apply_justify_self_invalid() {
    let (ok, _) = apply("justify-self", "invalid-value");
    assert!(!ok);
}

#[test]
fn test_apply_align_content_invalid() {
    let (ok, _) = apply("align-content", "invalid-value");
    assert!(!ok);
}

// === 各种 background 无效值 ===

#[test]
fn test_apply_background_invalid_values() {
    let (ok, _) = apply("background-color", "not-a-color");
    assert!(!ok);
    let (ok, _) = apply("background-image", "not-a-url");
    assert!(!ok);
    let (ok, _) = apply("background-repeat", "repeat-diagonal");
    assert!(!ok);
    let (ok, _) = apply("background-size", "not-a-size");
    assert!(!ok);
    let (ok, _) = apply("background-attachment", "not-a-attachment");
    assert!(!ok);
    let (ok, _) = apply("background-clip", "not-a-clip");
    assert!(!ok);
    let (ok, _) = apply("background-origin", "not-a-origin");
    assert!(!ok);
}

// === column-rule 变体 ===

#[test]
fn test_apply_column_rule_color() {
    let (ok, _) = apply("column-rule-color", "red");
    assert!(ok);
    let (ok, _) = apply("column-rule-color", "#00ff00");
    assert!(ok);
}

#[test]
fn test_apply_column_rule_style_variants() {
    for v in [
        "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset", "outset",
    ] {
        let (ok, _) = apply("column-rule-style", v);
        assert!(ok, "column-rule-style: {} should apply", v);
    }
}

#[test]
fn test_apply_column_rule_width_variants() {
    let (ok, _) = apply("column-rule-width", "medium");
    assert!(ok);
    let (ok, _) = apply("column-rule-width", "thin");
    assert!(ok);
    let (ok, _) = apply("column-rule-width", "thick");
    assert!(ok);
    let (ok, _) = apply("column-rule-width", "2px");
    assert!(ok);

    let mut style = ComputedStyle::default();
    assert!(apply_property_value(&mut style, "column-rule-width", "2px"));
    let previous = style.column_rule_width.clone();
    assert!(!apply_property_value(&mut style, "column-rule-width", "-1px"));
    assert_eq!(style.column_rule_width, previous);
}

// === counter-set 属性 ===

#[test]
fn test_apply_counter_set() {
    let (ok, _) = apply("counter-set", "none");
    assert!(ok);
    let (ok, _) = apply("counter-set", "section 2");
    assert!(ok);
}

// === scroll-margin/scroll-padding 完整覆盖 ===

#[test]
fn test_apply_scroll_margin_all() {
    let (ok, _) = apply("scroll-margin-top", "10px");
    assert!(ok);
    let (ok, _) = apply("scroll-margin-right", "20px");
    assert!(ok);
    let (ok, _) = apply("scroll-margin-bottom", "30px");
    assert!(ok);
    let (ok, _) = apply("scroll-margin-left", "40px");
    assert!(ok);
}

#[test]
fn test_apply_scroll_padding_all() {
    let (ok, _) = apply("scroll-padding-top", "5px");
    assert!(ok);
    let (ok, _) = apply("scroll-padding-right", "10px");
    assert!(ok);
    let (ok, _) = apply("scroll-padding-bottom", "15px");
    assert!(ok);
    let (ok, _) = apply("scroll-padding-left", "20px");
    assert!(ok);
    let (ok, _) = apply("scroll-padding-top", "auto");
    assert!(ok);
}

// === row-gap 独立属性 ===

#[test]
fn test_apply_row_gap() {
    let (ok, _) = apply("row-gap", "10px");
    assert!(ok);
    let (ok, _) = apply("row-gap", "2em");
    assert!(ok);
}

// === grid-area/grid-column/grid-row 简写 ===

#[test]
fn test_apply_grid_line_shorthands() {
    let (ok, _) = apply("grid-column", "1 / 3");
    assert!(ok);
    let (ok, _) = apply("grid-row", "2 / 4");
    assert!(ok);
    let (ok, _) = apply("grid-area", "1 / 2 / 3 / 4");
    assert!(ok);
}

// === transform-style / backface-visibility 非法值 ===

#[test]
fn test_apply_transform_style_invalid() {
    let (ok, _) = apply("transform-style", "3d");
    assert!(!ok);
}

#[test]
fn test_apply_backface_visibility_invalid() {
    let (ok, _) = apply("backface-visibility", "maybe");
    assert!(!ok);
}

// === aspect-ratio 边界值 ===

#[test]
fn test_apply_aspect_ratio_zero_height() {
    let (ok, _) = apply("aspect-ratio", "16 / 0");
    assert!(!ok, "aspect-ratio with zero height should return false");
}

#[test]
fn test_apply_aspect_ratio_invalid_value() {
    let (ok, _) = apply("aspect-ratio", "abc");
    assert!(!ok);
    let (ok, _) = apply("aspect-ratio", "16 / abc");
    assert!(!ok);
}

// === perspective none ===

#[test]
fn test_apply_perspective_none() {
    let (ok, _s) = apply("perspective", "none");
    assert!(ok);
    // none → Px(0.0)
}

#[test]
fn test_apply_perspective_length() {
    let (ok, _) = apply("perspective", "500px");
    assert!(ok);
}

// === unknown property ===

#[test]
fn test_apply_unknown_property() {
    let (ok, _) = apply("unknown-property", "value");
    assert!(!ok);
    let (ok, _) = apply("custom-foo", "bar");
    assert!(!ok);
}

// === empty-cells 变体 ===

#[test]
fn test_apply_empty_cells_hide() {
    let (ok, _) = apply("empty-cells", "hide");
    assert!(ok);
    let (ok, _) = apply("empty-cells", "show");
    assert!(ok);
}

// === object-fit 变体 ===

#[test]
fn test_apply_object_fit_variants() {
    for v in ["fill", "contain", "cover", "none", "scale-down"] {
        let (ok, _) = apply("object-fit", v);
        assert!(ok, "object-fit: {} should apply", v);
    }
}

// === object-position（CSS Images §3，R2303）===

#[test]
fn test_apply_object_position() {
    use crate::property::types::BackgroundPositionComputedValue as Bp;
    use zero_css_parser::values::LengthValue;

    // 默认 = Center（50% 50%）
    let s = ComputedStyle::default();
    assert!(
        matches!(s.object_position, Bp::Center),
        "default object-position = Center"
    );

    // 单关键字
    let (_, s) = apply("object-position", "top");
    assert!(matches!(s.object_position, Bp::Top));
    let (_, s) = apply("object-position", "left");
    assert!(matches!(s.object_position, Bp::Left));

    // 百分比（单值 → x，y 默认 center）
    let (ok, s) = apply("object-position", "25%");
    assert!(ok);
    assert!(matches!(s.object_position, Bp::Percent(25.0)));

    // 两值（关键字顺序无关：top left / left top）
    let (ok, s) = apply("object-position", "top left");
    assert!(ok);
    assert!(matches!(s.object_position, Bp::TwoValue(_, _)));

    // 两值百分比（CSS Images 默认 50% 50% 的反面：0% 100% = 左下）
    let (ok, s) = apply("object-position", "0% 100%");
    assert!(ok);
    match s.object_position {
        Bp::TwoValue(x, y) => {
            assert!(matches!(*x, Bp::Percent(0.0)), "x = 0%");
            assert!(matches!(*y, Bp::Percent(100.0)), "y = 100%");
        }
        other => panic!("expected TwoValue, got {:?}", other),
    }

    // 长度
    let (ok, s) = apply("object-position", "10px 20px");
    assert!(ok);
    match s.object_position {
        Bp::TwoValue(x, y) => {
            assert!(matches!(*x, Bp::Length(10.0)), "x = 10px");
            assert!(matches!(*y, Bp::Length(20.0)), "y = 20px");
        }
        other => panic!("expected TwoValue, got {:?}", other),
    }

    // 非法值不应用
    let (ok, _) = apply("object-position", "bogus");
    assert!(!ok);
    // LengthValue import 用于抑制未用警告（百分比/关键字路径已覆盖）
    let _ = LengthValue::Px(0.0);
}

// === filter 函数变体 ===

#[test]
fn test_apply_filter_all_functions() {
    let (ok, _) = apply("filter", "none");
    assert!(ok);
    let (ok, _) = apply("filter", "blur(5px)");
    assert!(ok);
    let (ok, _) = apply("filter", "brightness(1.5)");
    assert!(ok);
    let (ok, _) = apply("filter", "contrast(0.8)");
    assert!(ok);
    let (ok, _) = apply("filter", "grayscale(1)");
    assert!(ok);
    let (ok, _) = apply("filter", "hue-rotate(90deg)");
    assert!(ok);
    let (ok, _) = apply("filter", "invert(0.5)");
    assert!(ok);
    let (ok, _) = apply("filter", "opacity(0.3)");
    assert!(ok);
    let (ok, _) = apply("filter", "saturate(2)");
    assert!(ok);
    let (ok, _) = apply("filter", "sepia(0.8)");
    assert!(ok);
    let (ok, _) = apply("filter", "drop-shadow(1px 2px 3px red)");
    assert!(ok);
}

// === contain 变体 ===

#[test]
fn test_apply_contain_all_variants() {
    for v in ["none", "strict", "content", "size", "layout", "style", "paint"] {
        let (ok, _) = apply("contain", v);
        assert!(ok, "contain: {} should apply", v);
    }
    // 多值组合
    let (ok, _) = apply("contain", "size layout");
    assert!(ok);
}

// === accent-color / caret-color 详细 ===

#[test]
fn test_apply_accent_color_color() {
    let (ok, _) = apply("accent-color", "#ff0000");
    assert!(ok);
}

#[test]
fn test_apply_caret_color_color() {
    let (ok, _) = apply("caret-color", "#00ff00");
    assert!(ok);
}

// === 逻辑属性无效值 ===

#[test]
fn test_apply_logical_properties_valid_and_invalid() {
    for prop in [
        "margin-block-start",
        "margin-block-end",
        "margin-inline-start",
        "margin-inline-end",
        "padding-block-start",
        "padding-block-end",
        "padding-inline-start",
        "padding-inline-end",
        "inset-block-start",
        "inset-block-end",
        "inset-inline-start",
        "inset-inline-end",
    ] {
        let (ok, _) = apply(prop, "10px");
        assert!(ok, "{} should apply 10px", prop);
        let (ok, _) = apply(prop, "invalid");
        assert!(!ok, "{} should reject invalid", prop);
    }
}

// === transition 多值 ===

#[test]
fn test_apply_transition_delay() {
    let (ok, _) = apply("transition-delay", "0.5s, 1s");
    assert!(ok);
    let (ok, s) = apply("transition-delay", "-0.25s");
    assert!(ok);
    assert_eq!(s.transition_delay, vec![-0.25]);
    let (ok, _) = apply("transition-delay", "0.5s, bogus");
    assert!(!ok);
}

#[test]
fn test_apply_transition_duration_multi() {
    let (ok, _) = apply("transition-duration", "0.3s, 0.6s");
    assert!(ok);
}

// ═══════════════════════════════════════════════════════════
// property/parse.rs 覆盖率补全
// ═══════════════════════════════════════════════════════════

use crate::property::parse::*;

#[test]
fn test_parse_border_style_all_variants() {
    for v in [
        "none", "hidden", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset", "outset",
    ] {
        assert!(parse_border_style(v).is_some(), "border-style: {} should parse", v);
    }
    assert!(parse_border_style("invalid").is_none());
}

#[test]
fn test_parse_outline_style_all_variants() {
    for v in [
        "none", "dotted", "dashed", "solid", "double", "groove", "ridge", "inset", "outset",
    ] {
        assert!(parse_outline_style(v).is_some(), "outline-style: {} should parse", v);
    }
    assert!(parse_outline_style("hidden").is_none());
}

#[test]
fn test_parse_grid_auto_flow_variants() {
    assert!(parse_grid_auto_flow("row").is_some());
    assert!(parse_grid_auto_flow("column").is_some());
    assert!(parse_grid_auto_flow("dense").is_some());
    assert!(parse_grid_auto_flow("row dense").is_some());
    assert!(parse_grid_auto_flow("column dense").is_some());
    assert!(parse_grid_auto_flow("invalid").is_none());
}

#[test]
fn test_parse_grid_line_variants() {
    assert!(matches!(parse_grid_line("auto"), Some(GridLineValue::Auto)));
    assert!(matches!(parse_grid_line("1"), Some(GridLineValue::Line(1))));
    assert!(matches!(parse_grid_line("-1"), Some(GridLineValue::Line(-1))));
    assert!(matches!(parse_grid_line("span 2"), Some(GridLineValue::Span(2))));
    assert!(matches!(parse_grid_line("span3"), Some(GridLineValue::Span(3))));
    assert!(matches!(parse_grid_line("header"), Some(GridLineValue::Name(_))));
    // 0 是非法的 grid line 值
    assert!(parse_grid_line("0").is_none());
    // 以数字开头的不是命名区域
    assert!(parse_grid_line("1abc").is_none());
    // 包含 / 的不是命名区域
    assert!(parse_grid_line("a/b").is_none());
    // 空字符串
    assert!(parse_grid_line("").is_none());
}

#[test]
fn test_parse_grid_line_shorthand_no_slash() {
    // 无斜杠 → start=value, end=Auto
    let result = parse_grid_line_shorthand("2");
    assert!(result.is_some());
    let (start, end) = result.unwrap();
    assert!(matches!(start, GridLineValue::Line(2)));
    assert!(matches!(end, GridLineValue::Auto));
}

#[test]
fn test_parse_grid_line_shorthand_with_slash() {
    let result = parse_grid_line_shorthand("1 / 3");
    assert!(result.is_some());
    // 空部分
    assert!(parse_grid_line_shorthand(" / 3").is_none());
    assert!(parse_grid_line_shorthand("1 / ").is_none());
}

#[test]
fn test_parse_text_decoration_line_blink() {
    assert!(parse_text_decoration_line("blink").is_some());
    assert!(parse_text_decoration_line("invalid").is_none());
}

#[test]
fn test_parse_text_overflow_variants() {
    assert!(parse_text_overflow("clip").is_some());
    assert!(parse_text_overflow("ellipsis").is_some());
    assert!(parse_text_overflow("invalid").is_none());
}

#[test]
fn test_parse_flex_basis_content() {
    assert!(matches!(parse_flex_basis("content"), Some(FlexBasisValue::Content)));
    assert!(matches!(parse_flex_basis("auto"), Some(FlexBasisValue::Auto)));
    assert!(matches!(parse_flex_basis("100px"), Some(FlexBasisValue::Length(_))));
    assert!(parse_flex_basis("invalid").is_none());
}

#[test]
fn test_parse_z_index_integer() {
    assert!(matches!(parse_z_index("5"), Some(ZIndexValue::Integer(5))));
    assert!(matches!(parse_z_index("-3"), Some(ZIndexValue::Integer(-3))));
    assert!(parse_z_index("abc").is_none());
}

#[test]
fn test_parse_cursor_all_variants() {
    for v in [
        "auto",
        "default",
        "pointer",
        "move",
        "text",
        "wait",
        "crosshair",
        "help",
        "not-allowed",
        "grab",
        "grabbing",
        "col-resize",
        "row-resize",
        "ns-resize",
        "ew-resize",
        "none",
        "progress",
        "cell",
        "copy",
        "alias",
        "all-scroll",
        "zoom-in",
        "zoom-out",
    ] {
        assert!(parse_cursor(v).is_some(), "cursor: {} should parse", v);
    }
    assert!(parse_cursor("invalid").is_none());
}

#[test]
fn test_parse_scroll_snap_type() {
    assert!(parse_scroll_snap_type_computed("none").is_some());
    assert!(parse_scroll_snap_type_computed("mandatory").is_some());
    assert!(parse_scroll_snap_type_computed("proximity").is_some());
    assert!(parse_scroll_snap_type_computed("mandatory x").is_some());
    assert!(parse_scroll_snap_type_computed("proximity y").is_some());
    assert!(parse_scroll_snap_type_computed("invalid").is_none());
}

#[test]
fn test_parse_scroll_snap_align_variants() {
    for v in ["none", "start", "end", "center"] {
        assert!(parse_scroll_snap_align_computed(v).is_some());
    }
    assert!(parse_scroll_snap_align_computed("invalid").is_none());
}

#[test]
fn test_parse_scroll_snap_stop_variants() {
    assert!(parse_scroll_snap_stop_computed("normal").is_some());
    assert!(parse_scroll_snap_stop_computed("always").is_some());
    assert!(parse_scroll_snap_stop_computed("invalid").is_none());
}

#[test]
fn test_parse_scroll_padding_variants() {
    assert!(matches!(parse_scroll_padding("auto"), Some(ScrollPadding::Auto)));
    assert!(matches!(parse_scroll_padding("10px"), Some(ScrollPadding::Length(_))));
    // 非 Px 单位 → resolve_length_to_px 返回 0.0
    assert!(parse_scroll_padding("2em").is_some());
    assert!(parse_scroll_padding("invalid").is_none());
}

#[test]
fn test_parse_container_type_variants() {
    assert!(parse_container_type_computed("normal").is_some());
    assert!(parse_container_type_computed("size").is_some());
    assert!(parse_container_type_computed("inline-size").is_some());
    assert!(parse_container_type_computed("invalid").is_none());
}

#[test]
fn test_parse_font_family() {
    let families = parse_font_family("Arial, Helvetica, sans-serif");
    assert_eq!(families.len(), 3);
    assert_eq!(families[0], "Arial");

    // 带引号
    let families = parse_font_family("\"Times New Roman\", serif");
    assert_eq!(families[0], "\"Times New Roman\"");

    // 空值过滤
    let families = parse_font_family(",");
    assert!(families.is_empty());
}

#[test]
fn test_parse_line_height_variants() {
    assert!(matches!(parse_line_height("normal"), Some(LineHeightValue::Normal)));
    assert!(matches!(parse_line_height("1.5"), Some(LineHeightValue::Number(1.5))));
    assert!(matches!(parse_line_height("24px"), Some(LineHeightValue::Length(_))));
    assert!(matches!(parse_line_height("150%"), Some(LineHeightValue::Length(_))));
    assert!(parse_line_height("invalid").is_none());
}

#[test]
fn test_parse_text_align_variants() {
    for v in ["left", "right", "center", "justify", "start", "end"] {
        assert!(parse_text_align(v).is_some(), "text-align: {} should parse", v);
    }
    assert!(parse_text_align("invalid").is_none());
}

#[test]
fn test_parse_text_decoration_variants() {
    for v in ["none", "underline", "overline", "line-through"] {
        assert!(parse_text_decoration(v).is_some());
    }
    assert!(parse_text_decoration("invalid").is_none());
}

#[test]
fn test_parse_text_transform_variants() {
    for v in ["none", "uppercase", "lowercase", "capitalize"] {
        assert!(parse_text_transform(v).is_some());
    }
    assert!(parse_text_transform("invalid").is_none());
}

#[test]
fn test_parse_white_space_variants() {
    for v in ["normal", "pre", "nowrap", "pre-wrap", "pre-line", "break-spaces"] {
        assert!(parse_white_space(v).is_some());
    }
    assert!(parse_white_space("invalid").is_none());
}

#[test]
fn test_parse_word_break_variants() {
    for v in ["normal", "break-all", "keep-all", "break-word"] {
        assert!(parse_word_break(v).is_some());
    }
    assert!(parse_word_break("invalid").is_none());
}

#[test]
fn test_parse_writing_mode_variants() {
    assert!(parse_writing_mode("horizontal-tb").is_some());
    assert!(parse_writing_mode("vertical-rl").is_some());
    assert!(parse_writing_mode("vertical-lr").is_some());
    assert!(parse_writing_mode("invalid").is_none());
}

#[test]
fn test_parse_comma_separated_timing_functions() {
    let funcs = parse_comma_separated_timing_functions("ease, linear, ease-in-out").unwrap();
    assert_eq!(funcs.len(), 3);

    // 带 cubic-bezier（内部逗号）
    let funcs = parse_comma_separated_timing_functions("cubic-bezier(0.1, 0.7, 1.0, 0.1), ease").unwrap();
    assert_eq!(funcs.len(), 2);
}

#[test]
fn test_resolve_length_to_px() {
    assert_eq!(resolve_length_to_px(LengthValue::Px(100.0)), 100.0);
    assert_eq!(resolve_length_to_px(LengthValue::Em(2.0)), 0.0);
}
