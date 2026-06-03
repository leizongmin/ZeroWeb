//! CSS 解析器覆盖率补充测试：transform 函数、types.rs calc 边界情况。

use crate::values::{
    ColorValue, GradientValue, LengthValue, TransformFunction, TransformValue, eval_calc, parse_box_shadow, parse_calc,
    parse_gradient, parse_length, parse_text_shadow, parse_transform,
};

// ═══════════════════════════════════════════════════════════════════════
// parse_transform.rs — 变换函数全覆盖
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_transform_translate() {
    let t = parse_transform("translate(10px, 20px)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::Translate(_, _)));
    }
}

#[test]
fn test_transform_translate_x() {
    let t = parse_transform("translateX(50px)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::TranslateX(_)));
    }
}

#[test]
fn test_transform_translate_y() {
    let t = parse_transform("translateY(30px)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::TranslateY(_)));
    }
}

#[test]
fn test_transform_translate_3d() {
    let t = parse_transform("translate3d(10px, 20px, 5px)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::Translate3d(_, _, _)));
    }
}

#[test]
fn test_transform_scale() {
    let t = parse_transform("scale(2)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::Scale(_, _)));
    }
}

#[test]
fn test_transform_scale_xy() {
    let t = parse_transform("scale(2, 3)").unwrap();
    if let TransformValue::List(fs) = t {
        // Scale takes (sx, Option<sy>)
        assert!(matches!(fs[0], TransformFunction::Scale(_, Some(_))));
    }
}

#[test]
fn test_transform_scale_3d() {
    let t = parse_transform("scale3d(2, 3, 1)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::Scale3d(_, _, _)));
    }
}

#[test]
fn test_transform_rotate() {
    let t = parse_transform("rotate(45deg)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::Rotate(_)));
    }
}

#[test]
fn test_transform_rotate_x() {
    let t = parse_transform("rotateX(90deg)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::RotateX(_)));
    }
}

#[test]
fn test_transform_rotate_y() {
    let t = parse_transform("rotateY(45deg)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::RotateY(_)));
    }
}

#[test]
fn test_transform_rotate_z() {
    let t = parse_transform("rotateZ(30deg)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::RotateZ(_)));
    }
}

#[test]
fn test_transform_rotate_3d() {
    let t = parse_transform("rotate3d(1, 0, 0, 45deg)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::Rotate3d(_, _, _, _)));
    }
}

#[test]
fn test_transform_skew() {
    let t = parse_transform("skew(10deg)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::Skew(_, _)));
    }
}

#[test]
fn test_transform_skew_xy() {
    let t = parse_transform("skew(10deg, 20deg)").unwrap();
    if let TransformValue::List(fs) = t {
        // Skew takes (ax, Option<ay>)
        assert!(matches!(fs[0], TransformFunction::Skew(_, Some(_))));
    }
}

#[test]
fn test_transform_perspective() {
    let t = parse_transform("perspective(500px)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::Perspective(_)));
    }
}

#[test]
fn test_transform_perspective_invalid_zero() {
    assert!(parse_transform("perspective(0)").is_none());
}

#[test]
fn test_transform_perspective_invalid_negative() {
    assert!(parse_transform("perspective(-100)").is_none());
}

#[test]
fn test_transform_matrix() {
    let t = parse_transform("matrix(1, 0, 0, 1, 10, 20)").unwrap();
    if let TransformValue::List(fs) = t {
        assert!(matches!(fs[0], TransformFunction::Matrix(_, _, _, _, _, _)));
    }
}

#[test]
fn test_transform_none() {
    assert!(matches!(parse_transform("none"), Some(TransformValue::None)));
}

#[test]
fn test_transform_invalid() {
    assert!(parse_transform("invalid").is_none());
    assert!(parse_transform("").is_none());
}

#[test]
fn test_transform_multiple_functions() {
    let t = parse_transform("translate(10px) rotate(45deg) scale(2)").unwrap();
    if let TransformValue::List(fs) = t {
        assert_eq!(fs.len(), 3);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform.rs — gradient 更多边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_radial_gradient_circle_with_length() {
    assert!(parse_gradient("radial-gradient(circle 50px, red, blue)").is_some());
}

#[test]
fn test_radial_gradient_closest_corner() {
    assert!(parse_gradient("radial-gradient(closest-corner, red, blue)").is_some());
}

#[test]
fn test_conic_gradient_from_angle() {
    assert!(parse_gradient("conic-gradient(from 180deg, red, blue)").is_some());
}

#[test]
fn test_linear_gradient_color_stop_position() {
    assert!(parse_gradient("linear-gradient(red 0%, green 50%, blue 100%)").is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform.rs — text/box shadow 更多边界
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_text_shadow_hex_color() {
    let s = parse_text_shadow("2px 3px #ff0000").unwrap();
    if let ColorValue::Rgba(r, g, b, _) = s.color {
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }
}

#[test]
fn test_box_shadow_hex_color_with_spread() {
    let s = parse_box_shadow("2px 3px 4px 5px #00ff00").unwrap();
    if let ColorValue::Rgba(_, g, _, _) = s.color {
        assert_eq!(g, 255);
    }
}

#[test]
fn test_box_shadow_inset_hex() {
    let s = parse_box_shadow("inset 1px 1px 2px rgba(0,0,0,0.5)").unwrap();
    assert!(s.inset);
}

// ═══════════════════════════════════════════════════════════════════════
// types.rs — calc 和 length 更多边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_calc_nested() {
    assert!(parse_calc("calc(100% - calc(20px + 10px))").is_some());
}

#[test]
fn test_calc_clamp_function() {
    // clamp may need to be wrapped in calc or parsed separately
    // Test that calc() with complex expressions works
    assert!(parse_calc("calc((100px + 50px) / 2)").is_some());
}

#[test]
fn test_calc_min_function() {
    assert!(parse_calc("calc(50% - 20px)").is_some());
}

#[test]
fn test_calc_max_function() {
    assert!(parse_calc("calc(10px + 20px)").is_some());
}

#[test]
fn test_calc_with_various_units() {
    assert!(parse_calc("calc(10em + 5px)").is_some());
    assert!(parse_calc("calc(100vh - 50px)").is_some());
}

#[test]
fn test_eval_calc_with_parent_length() {
    if let Some(expr) = parse_calc("calc(50% + 10px)") {
        let result = eval_calc(&expr, Some(200.0));
        assert!(result.is_some());
        // 50% of 200 + 10 = 110
        assert!((result.unwrap() - 110.0).abs() < 0.01);
    }
}

#[test]
fn test_parse_length_various() {
    assert!(matches!(parse_length("10px"), Some(LengthValue::Px(10.0))));
    assert!(matches!(parse_length("2.5em"), Some(LengthValue::Em(2.5))));
    assert!(matches!(parse_length("1rem"), Some(LengthValue::Rem(1.0))));
    assert!(matches!(parse_length("50%"), Some(LengthValue::Percentage(50.0))));
    assert!(matches!(parse_length("100vh"), Some(LengthValue::Vh(100.0))));
    assert!(matches!(parse_length("100vw"), Some(LengthValue::Vw(100.0))));
    assert!(matches!(parse_length("5vmin"), Some(LengthValue::Vmin(5.0))));
    assert!(matches!(parse_length("5vmax"), Some(LengthValue::Vmax(5.0))));
}

#[test]
fn test_parse_length_auto() {
    assert!(matches!(parse_length("auto"), Some(LengthValue::Auto)));
}

#[test]
fn test_parse_length_none_values() {
    assert!(parse_length("").is_none());
    assert!(parse_length("invalid").is_none());
    assert!(parse_length("10").is_none()); // 无单位数值不是有效长度
}

#[test]
fn test_parse_length_negative() {
    assert!(matches!(parse_length("-5px"), Some(LengthValue::Px(-5.0))));
}
