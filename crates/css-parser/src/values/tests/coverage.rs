//! parse_transform.rs 覆盖率测试。
//!
//! 覆盖：
//! - animation-duration ms/s 解析
//! - timing-function cubic-bezier/steps
//! - transform 3D 函数（translate3d, scale3d, rotate3d, perspective, matrix）
//! - transform 各变体（translateX, translateY, scaleX, scaleY, skew, rotateX/Y/Z）
//! - gradient（linear-gradient 方向、radial-gradient shape/position、conic-gradient）
//! - text-shadow 多种参数组合
//! - box-shadow inset + spread
//! - grid-area 斜杠语法

use crate::values::{
    parse_animation_duration, parse_box_shadow, parse_box_shadow_list, parse_gradient, parse_grid_area,
    parse_text_shadow, parse_text_shadow_list, parse_timing_function, parse_transform,
};

// ═══════════════════════════════════════════════════════════════════════
// animation-duration
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_duration_ms() {
    let result = parse_animation_duration("500ms");
    assert!(result.is_some());
}

#[test]
fn test_animation_duration_s() {
    let result = parse_animation_duration("1.5s");
    assert!(result.is_some());
}

#[test]
fn test_animation_duration_invalid() {
    assert!(parse_animation_duration("invalid").is_none());
    assert!(parse_animation_duration("-1s").is_none());
    assert!(parse_animation_duration("infs").is_none());
    assert!(parse_animation_duration("NaNs").is_none());
    assert!(parse_animation_duration("infms").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// timing-function
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_timing_function_cubic_bezier() {
    let result = parse_timing_function("cubic-bezier(0.25, 0.1, 0.25, 1.0)");
    assert!(result.is_some());
}

#[test]
fn test_timing_function_cubic_bezier_invalid() {
    // 参数数量不对
    assert!(parse_timing_function("cubic-bezier(0.25, 0.1)").is_none());
    assert!(parse_timing_function("cubic-bezier(-0.1, 0.1, 0.25, 1.0)").is_none());
    assert!(parse_timing_function("cubic-bezier(0.25, 0.1, 1.1, 1.0)").is_none());
}

#[test]
fn test_timing_function_steps() {
    let result = parse_timing_function("steps(4, end)");
    assert!(result.is_some());
}

#[test]
fn test_timing_function_steps_no_position() {
    let result = parse_timing_function("steps(4)");
    assert!(result.is_some());
}

#[test]
fn test_timing_function_steps_start() {
    let result = parse_timing_function("steps(4, start)");
    assert!(result.is_some());
}

#[test]
fn test_timing_function_steps_invalid() {
    assert!(parse_timing_function("steps(4, start, end)").is_none());
    assert!(parse_timing_function("steps(0)").is_none());
    assert!(parse_timing_function("steps(-1)").is_none());
    assert!(parse_timing_function("steps(1, jump-none)").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// transform — 3D 函数
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_transform_translate3d() {
    let result = parse_transform("translate3d(10px, 20px, 30px)");
    assert!(result.is_some());
}

// R2294：translate(%) 不再丢弃整 transform，解析为 *Mixed 变体（per-arg 百分比标记）。
#[test]
fn test_transform_translate_percent_not_dropped() {
    use crate::values::{TransformFunction, TransformValue};
    // 修复前：parse_css_number("50%") → None → 整 transform 丢弃。
    let list = match parse_transform("translateX(50%)").unwrap() {
        TransformValue::List(f) => f,
        _ => panic!("expected list"),
    };
    assert!(matches!(list[0], TransformFunction::TranslateXMixed(50.0, true)));

    let list = match parse_transform("translate(-50%, -50%)").unwrap() {
        TransformValue::List(f) => f,
        _ => panic!("expected list"),
    };
    assert!(matches!(
        list[0],
        TransformFunction::TranslateMixed(-50.0, true, -50.0, true)
    ));

    // 混合 px + %。
    let list = match parse_transform("translate(50%, 10px)").unwrap() {
        TransformValue::List(f) => f,
        _ => panic!("expected list"),
    };
    assert!(matches!(
        list[0],
        TransformFunction::TranslateMixed(50.0, true, 10.0, false)
    ));
}

#[test]
fn test_transform_translate_px_still_uses_translate() {
    // 回归守护：纯 px 仍走既有 Translate 变体（零回归）。
    use crate::values::{TransformFunction, TransformValue};
    let list = match parse_transform("translate(10px, 20px)").unwrap() {
        TransformValue::List(f) => f,
        _ => panic!("expected list"),
    };
    assert!(matches!(list[0], TransformFunction::Translate(10.0, 20.0)));
    let list = match parse_transform("translateY(30px)").unwrap() {
        TransformValue::List(f) => f,
        _ => panic!("expected list"),
    };
    assert!(matches!(list[0], TransformFunction::TranslateY(30.0)));
}

// R2295：CSS transform 函数名大小写不敏感（CSS Syntax §3.1）。
#[test]
fn test_transform_function_name_case_insensitive() {
    use crate::values::{TransformFunction, TransformValue};
    // 全大写、混合大小写、全小写均应解析。
    assert!(parse_transform("TRANSLATEX(10px)").is_some());
    assert!(parse_transform("Translate(10px, 20px)").is_some());
    let list = match parse_transform("matrix(1,0,0,1,5,5)").unwrap() {
        TransformValue::List(f) => f,
        _ => panic!("expected list"),
    };
    assert!(matches!(
        list[0],
        TransformFunction::Matrix(1.0, 0.0, 0.0, 1.0, 5.0, 5.0)
    ));
    let list = match parse_transform("SCALE(2.0, 3.0)").unwrap() {
        TransformValue::List(f) => f,
        _ => panic!("expected list"),
    };
    assert!(matches!(list[0], TransformFunction::Scale(2.0, Some(3.0))));
}

#[test]
fn test_transform_scale3d() {
    let result = parse_transform("scale3d(1.5, 2.0, 1.0)");
    assert!(result.is_some());
}

#[test]
fn test_transform_rotate3d() {
    let result = parse_transform("rotate3d(1, 0, 0, 45deg)");
    assert!(result.is_some());
}

#[test]
fn test_transform_perspective() {
    let result = parse_transform("perspective(500px)");
    assert!(result.is_some());
}

#[test]
fn test_transform_perspective_zero_fails() {
    assert!(parse_transform("perspective(0)").is_none());
}

#[test]
fn test_transform_matrix() {
    let result = parse_transform("matrix(1, 0, 0, 1, 10, 20)");
    assert!(result.is_some());
}

#[test]
fn test_transform_matrix_invalid_count() {
    assert!(parse_transform("matrix(1, 0, 0)").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// transform — 各独立函数
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_transform_translate_x() {
    let result = parse_transform("translateX(50px)");
    assert!(result.is_some());
}

#[test]
fn test_transform_translate_y() {
    let result = parse_transform("translateY(30px)");
    assert!(result.is_some());
}

#[test]
fn test_transform_scale_x() {
    let result = parse_transform("scaleX(2.0)");
    assert!(result.is_some());
}

#[test]
fn test_transform_scale_y() {
    let result = parse_transform("scaleY(0.5)");
    assert!(result.is_some());
}

#[test]
fn test_transform_scale_two_args() {
    let result = parse_transform("scale(1.5, 2.0)");
    assert!(result.is_some());
}

#[test]
fn test_transform_skew() {
    let result = parse_transform("skew(30deg)");
    assert!(result.is_some());
}

#[test]
fn test_transform_skew_two_args() {
    let result = parse_transform("skew(30deg, 15deg)");
    assert!(result.is_some());
}

#[test]
fn test_transform_rotate_x() {
    let result = parse_transform("rotateX(45deg)");
    assert!(result.is_some());
}

#[test]
fn test_transform_rotate_y() {
    let result = parse_transform("rotateY(45deg)");
    assert!(result.is_some());
}

#[test]
fn test_transform_rotate_z() {
    let result = parse_transform("rotateZ(90deg)");
    assert!(result.is_some());
}

#[test]
fn test_transform_translate_rad() {
    let result = parse_transform("rotate(1.5708rad)");
    assert!(result.is_some());
}

#[test]
fn test_transform_translate_turn() {
    let result = parse_transform("rotate(0.25turn)");
    assert!(result.is_some());
}

#[test]
fn test_transform_empty() {
    assert!(parse_transform("").is_none());
}

#[test]
fn test_transform_none() {
    // "none" 是合法的 transform 值
    let result = parse_transform("none");
    assert!(result.is_some());
}

#[test]
fn test_transform_unknown_function() {
    assert!(parse_transform("unknownFunc(10)").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// gradient — linear-gradient 方向变体
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_linear_gradient_to_top() {
    let result = parse_gradient("linear-gradient(to top, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_linear_gradient_to_left() {
    let result = parse_gradient("linear-gradient(to left, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_linear_gradient_to_top_left() {
    let result = parse_gradient("linear-gradient(to top left, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_linear_gradient_to_bottom_right() {
    let result = parse_gradient("linear-gradient(to bottom right, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_linear_gradient_angle() {
    let result = parse_gradient("linear-gradient(45deg, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_linear_gradient_no_direction() {
    let result = parse_gradient("linear-gradient(red, blue)");
    assert!(result.is_some());
}

// R2292：calc() 色标位置（css-images gradient-infinity）。
// calc(1px / 0) 含内部空格——旧 rfind(' ') 切到 calc 内部空格 → 色标解析失败 →
// 整 gradient 被丢弃。修复：按括号深度切分 color/position + 接受 calc 为位置。
#[test]
fn test_linear_gradient_calc_stop_position_not_dropped() {
    // 修复前：calc(1px / 0) 内部空格使 rfind 误切 → gradient 被丢 → None。
    let result = parse_gradient("linear-gradient(to right, lime 100px, red calc(1px / 0))");
    assert!(result.is_some(), "calc() position must not drop the whole gradient");
    assert!(
        parse_gradient("linear-gradient(to right, lime 100px, red calc(Infinity * 1px))").is_some(),
        "calc(Infinity * 1px) position must not drop the gradient"
    );
}

#[test]
fn test_linear_gradient_calc_stop_position_is_calc() {
    use crate::values::{GradientValue, LengthValue};
    let lg = match parse_gradient("linear-gradient(to right, lime 100px, red calc(1px / 0))").unwrap() {
        GradientValue::Linear(lg) => lg,
        _ => panic!("expected linear gradient"),
    };
    assert_eq!(lg.stops.len(), 2);
    // red 色标位置应解析为 LengthValue::Calc（延迟求值）。
    assert!(
        matches!(lg.stops[1].position, Some(LengthValue::Calc(_))),
        "red stop position should be LengthValue::Calc"
    );
    // lime 色标位置保持普通 Px。
    assert!(matches!(lg.stops[0].position, Some(LengthValue::Px(100.0))));
}

#[test]
fn test_linear_gradient_rgb_color_with_position_regression() {
    // 回归守护：颜色含内部空格 rgb(0 0 0) 后随位置，深度切分仍正确（不误切颜色内空格）。
    let result = parse_gradient("linear-gradient(to right, rgb(0 0 0) 50%, white)");
    assert!(result.is_some());
    let result = parse_gradient("linear-gradient(to right, rgb(0, 0, 0) 50%, white)");
    assert!(result.is_some());
}

#[test]
fn test_linear_gradient_empty() {
    assert!(parse_gradient("linear-gradient()").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// gradient — radial-gradient shape/position 变体
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_radial_gradient_circle() {
    let result = parse_gradient("radial-gradient(circle, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_ellipse() {
    let result = parse_gradient("radial-gradient(ellipse, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_closest_side() {
    let result = parse_gradient("radial-gradient(circle closest-side, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_farthest_side() {
    let result = parse_gradient("radial-gradient(ellipse farthest-side, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_closest_corner() {
    let result = parse_gradient("radial-gradient(circle closest-corner, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_at_position() {
    let result = parse_gradient("radial-gradient(circle at center, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_at_percent() {
    let result = parse_gradient("radial-gradient(circle at 30% 70%, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_circle_with_length() {
    let result = parse_gradient("radial-gradient(circle 50px, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_shape_at_position() {
    let result = parse_gradient("radial-gradient(circle at left top, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_invalid_shape_size_or_position() {
    assert!(parse_gradient("radial-gradient(ellipse xyz, red, blue)").is_none());
    assert!(parse_gradient("radial-gradient(circle at bogus, red, blue)").is_none());
}

#[test]
fn test_radial_gradient_no_args() {
    assert!(parse_gradient("radial-gradient()").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// gradient — conic-gradient
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_conic_gradient_basic() {
    let result = parse_gradient("conic-gradient(red, blue, green)");
    assert!(result.is_some());
}

#[test]
fn test_conic_gradient_from_angle() {
    let result = parse_gradient("conic-gradient(from 45deg, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_conic_gradient_at_position() {
    let result = parse_gradient("conic-gradient(at center, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_conic_gradient_from_angle_at_position() {
    let result = parse_gradient("conic-gradient(from 90deg at 25% 75%, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_conic_gradient_invalid_config() {
    assert!(parse_gradient("conic-gradient(from bogus, red, blue)").is_none());
    assert!(parse_gradient("conic-gradient(at bogus, red, blue)").is_none());
}

#[test]
fn test_conic_gradient_empty() {
    assert!(parse_gradient("conic-gradient()").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// gradient — repeating 变体
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_repeating_linear_gradient() {
    let result = parse_gradient("repeating-linear-gradient(to right, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_repeating_radial_gradient() {
    let result = parse_gradient("repeating-radial-gradient(circle, red, blue)");
    assert!(result.is_some());
}

#[test]
fn test_repeating_conic_gradient() {
    let result = parse_gradient("repeating-conic-gradient(red, blue)");
    assert!(result.is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// gradient — unknown type
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_gradient_unknown_type() {
    assert!(parse_gradient("unknown-gradient(red, blue)").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// text-shadow — 多种参数组合
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_text_shadow_basic_two_values() {
    let result = parse_text_shadow("2px 2px");
    assert!(result.is_some());
}

#[test]
fn test_text_shadow_with_color_first() {
    // R2477：CSS Text Decoration §3 `<length>{2,3} && <color>?` 的 `&&` 允许颜色任意位置。
    // 此处测长度在前、颜色在末（blur 后）的合法语法。
    let result = parse_text_shadow("2px 2px 4px red");
    assert!(result.is_some());
}

#[test]
fn test_text_shadow_with_color_last() {
    let result = parse_text_shadow("2px 2px red");
    assert!(result.is_some());
}

#[test]
fn test_text_shadow_with_blur_and_color() {
    let result = parse_text_shadow("2px 2px 4px rgba(0,0,0,0.5)");
    assert!(result.is_some());
}

#[test]
fn test_text_shadow_color_then_blur() {
    let result = parse_text_shadow("2px 2px red 4px");
    assert!(result.is_some());
}

#[test]
fn test_text_shadow_none() {
    let result = parse_text_shadow("none");
    assert!(result.is_some());
}

#[test]
fn test_text_shadow_too_few_values() {
    assert!(parse_text_shadow("2px").is_none());
}

#[test]
fn test_text_shadow_rejects_invalid_length_grammar() {
    for value in [
        "2px 2px -1px",
        "2px 2px 10%",
        "2px 2px thin",
        "2px 2px min-content",
        "2px 2px infpx",
        "2px 2px NaNpx",
    ] {
        assert!(parse_text_shadow(value).is_none(), "{value} should be rejected");
    }
}

// ── R2305：parse_text_shadow_list — 多阴影列表（CSS Text Decoration §3：none | <shadow>#）──

#[test]
fn test_text_shadow_list_none_is_empty() {
    let list = parse_text_shadow_list("none").expect("none → Some(空 Vec)");
    assert!(list.is_empty(), "none 应解析为空阴影列表");
}

#[test]
fn test_text_shadow_list_single() {
    let list = parse_text_shadow_list("2px 2px 4px red").expect("单阴影 → Some");
    assert_eq!(list.len(), 1);
}

#[test]
fn test_text_shadow_list_multiple_comma() {
    // 顶层逗号分割：3 个独立阴影
    let list = parse_text_shadow_list("1px 1px red, 2px 2px green, 3px 3px blue").expect("多阴影 → Some");
    assert_eq!(list.len(), 3, "应拆为 3 个阴影");
}

#[test]
fn test_text_shadow_list_rgb_internal_commas_preserved() {
    // paren-aware：rgba()/rgb() 的内部逗号不应拆分 → 仍是 2 个阴影
    let list = parse_text_shadow_list("1px 1px rgb(0, 0, 0), 2px 2px rgba(255, 0, 0, 0.5)").expect("含函数逗号 → Some");
    assert_eq!(list.len(), 2, "rgb()/rgba() 内部逗号必须保持一体，应为 2 个阴影");
}

#[test]
fn test_text_shadow_list_any_invalid_is_none() {
    // 任意单个阴影解析失败 → 整列表 None
    assert!(parse_text_shadow_list("1px 1px red, bogus, 2px 2px blue").is_none());
}

#[test]
fn test_text_shadow_list_empty_is_none() {
    // 空字符串 / 纯空白 → None
    assert!(parse_text_shadow_list("").is_none());
    assert!(parse_text_shadow_list("   ").is_none());
}

#[test]
fn test_text_shadow_list_empty_comma_items_are_invalid() {
    assert!(parse_text_shadow_list("1px 1px red,").is_none());
    assert!(parse_text_shadow_list(", 1px 1px red").is_none());
    assert!(parse_text_shadow_list("1px 1px red,, 2px 2px blue").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// box-shadow — inset + spread
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_box_shadow_basic() {
    let result = parse_box_shadow("2px 2px");
    assert!(result.is_some());
}

#[test]
fn test_box_shadow_with_blur() {
    let result = parse_box_shadow("2px 2px 4px");
    assert!(result.is_some());
}

#[test]
fn test_box_shadow_with_spread() {
    let result = parse_box_shadow("2px 2px 4px 1px");
    assert!(result.is_some());
}

#[test]
fn test_box_shadow_with_color() {
    let result = parse_box_shadow("2px 2px 4px red");
    assert!(result.is_some());
}

#[test]
fn test_box_shadow_inset() {
    let result = parse_box_shadow("inset 2px 2px 4px red");
    assert!(result.is_some());
}

#[test]
fn test_box_shadow_duplicate_inset_is_invalid() {
    assert!(parse_box_shadow("inset 1px 1px inset").is_none());
    assert!(parse_box_shadow("1px 1px inset inset").is_none());
}

#[test]
fn test_box_shadow_inset_with_spread() {
    let result = parse_box_shadow("inset 2px 2px 4px 1px red");
    assert!(result.is_some());
}

#[test]
fn test_box_shadow_none() {
    let result = parse_box_shadow("none");
    assert!(result.is_some());
}

#[test]
fn test_box_shadow_too_few_values() {
    assert!(parse_box_shadow("2px").is_none());
}

#[test]
fn test_box_shadow_rejects_invalid_length_grammar() {
    assert!(parse_box_shadow("-2px -3px 4px -1px red").is_some());
    for value in [
        "2px 2px -1px",
        "2px 2px 10%",
        "2px 2px thin",
        "2px 2px min-content",
        "2px 2px infpx",
        "2px 2px NaNpx",
    ] {
        assert!(parse_box_shadow(value).is_none(), "{value} should be rejected");
    }
}

// ── R2304：parse_box_shadow_list — 多阴影列表（CSS Backgrounds §7.2：<shadow>#）──

#[test]
fn test_box_shadow_list_none_is_empty() {
    let list = parse_box_shadow_list("none").expect("none → Some(空 Vec)");
    assert!(list.is_empty(), "none 应解析为空阴影列表");
}

#[test]
fn test_box_shadow_list_single() {
    let list = parse_box_shadow_list("2px 2px 4px red").expect("单阴影 → Some");
    assert_eq!(list.len(), 1);
    assert!(list[0].inset == false);
}

#[test]
fn test_box_shadow_list_multiple_comma() {
    // 顶层逗号分割：3 个独立阴影
    let list = parse_box_shadow_list("1px 1px red, 2px 2px green, 3px 3px blue").expect("多阴影 → Some");
    assert_eq!(list.len(), 3, "应拆为 3 个阴影");
}

#[test]
fn test_box_shadow_list_rgb_internal_commas_preserved() {
    // paren-aware：rgb()/rgba() 的内部逗号不应拆分 → 仍是 2 个阴影
    let list = parse_box_shadow_list("1px 1px rgb(0, 0, 0), 2px 2px rgba(255, 0, 0, 0.5)").expect("含函数逗号 → Some");
    assert_eq!(list.len(), 2, "rgb()/rgba() 内部逗号必须保持一体，应为 2 个阴影");
}

#[test]
fn test_box_shadow_list_inset_mixed() {
    // 列表中可混入 inset
    let list = parse_box_shadow_list("inset 1px 1px red, 2px 2px blue").expect("混 inset → Some");
    assert_eq!(list.len(), 2);
    assert!(list[0].inset, "首个应为 inset");
    assert!(!list[1].inset, "第二个应为 outset");
}

#[test]
fn test_box_shadow_list_any_invalid_is_none() {
    // 任意单个阴影解析失败 → 整列表 None（CSS 错误恢复：整条声明无效）
    assert!(parse_box_shadow_list("1px 1px red, bogus, 2px 2px blue").is_none());
}

#[test]
fn test_box_shadow_list_empty_is_none() {
    // 空字符串 / 纯空白 → None（无有效阴影）
    assert!(parse_box_shadow_list("").is_none());
    assert!(parse_box_shadow_list("   ").is_none());
}

#[test]
fn test_box_shadow_list_empty_comma_items_are_invalid() {
    assert!(parse_box_shadow_list("1px 1px red,").is_none());
    assert!(parse_box_shadow_list(", 1px 1px red").is_none());
    assert!(parse_box_shadow_list("1px 1px red,, 2px 2px blue").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// grid-area — 斜杠语法
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_grid_area_single_value() {
    let result = parse_grid_area("header");
    assert!(result.is_some());
    let (rs, re, cs, ce) = result.unwrap();
    assert_eq!(rs, "header");
    assert_eq!(re, "header");
}

#[test]
fn test_grid_area_two_values() {
    let result = parse_grid_area("header / sidebar");
    assert!(result.is_some());
    let (rs, re, cs, ce) = result.unwrap();
    assert_eq!(rs, "header");
    assert_eq!(re, "auto");
    assert_eq!(cs, "sidebar");
    assert_eq!(ce, "auto");
}

#[test]
fn test_grid_area_four_values() {
    let result = parse_grid_area("1 / 2 / 3 / 4");
    assert!(result.is_some());
    let (rs, re, cs, ce) = result.unwrap();
    assert_eq!(rs, "1");
    assert_eq!(re, "2");
    assert_eq!(cs, "3");
    assert_eq!(ce, "4");
}

#[test]
fn test_grid_area_empty() {
    assert!(parse_grid_area("").is_none());
}

#[test]
fn test_grid_area_empty_after_slash() {
    assert!(parse_grid_area(" / ").is_none());
}

#[test]
fn test_grid_area_empty_before_slash() {
    assert!(parse_grid_area("/ sidebar").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// gradient — 混合 color stops
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_linear_gradient_with_stops() {
    let result = parse_gradient("linear-gradient(to right, red 0%, yellow 50%, green 100%)");
    assert!(result.is_some());
}

#[test]
fn test_radial_gradient_with_stops() {
    let result = parse_gradient("radial-gradient(circle, white 0%, black 100%)");
    assert!(result.is_some());
}

#[test]
fn test_conic_gradient_from_angle_only() {
    let result = parse_gradient("conic-gradient(from 180deg, red, blue)");
    assert!(result.is_some());
}
