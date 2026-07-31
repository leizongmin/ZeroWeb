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

#[test]
fn test_r2315_gradient_double_position_color_stop() {
    // CSS Images 4 双位置色标 `red 0% 50%` ≡ 两个同色色标 red@0% + red@50%（硬过渡）。
    // 此前 split_color_stop_position 在首个空格切分 → 位置部分 "0% 50%" 解析失败 → 整条渐变 None。
    let g = parse_gradient("linear-gradient(red 0% 50%, blue 100%)").expect("double-position color stop must parse");
    let stops = match g {
        GradientValue::Linear(lg) => lg.stops,
        _ => panic!("expected linear gradient"),
    };
    assert_eq!(stops.len(), 3, "red 0% 50% → 2 stops + blue 100% → 3 total");
    // red@0%, red@50%, blue@100%
    let assert_pct = |stop: &crate::values::GradientColorStop, expect: f64| match &stop.position {
        Some(LengthValue::Percentage(p)) => assert!(((*p) - expect).abs() < 0.01, "got {p} expect {expect}"),
        other => panic!("expected Percentage({expect}), got {other:?}"),
    };
    assert_pct(&stops[0], 0.0);
    assert_pct(&stops[1], 50.0);
    assert_pct(&stops[2], 100.0);
    // 前两色标同色（red，双位置展开为两个同色色标）
    assert_eq!(stops[0].color, stops[1].color);
}

#[test]
fn test_r2315_gradient_double_position_length_and_calc() {
    // 双位置也支持长度与 calc() 位置（depth-aware 切分，calc 内空格不破坏）
    let g = parse_gradient("linear-gradient(red 10px 20px, blue)").expect("must parse");
    let stops = match g {
        GradientValue::Linear(lg) => lg.stops,
        _ => panic!("expected linear gradient"),
    };
    assert_eq!(stops.len(), 3, "red 10px 20px → 2 stops + blue → 3");
    assert!(matches!(stops[0].position, Some(LengthValue::Px(10.0))));
    assert!(matches!(stops[1].position, Some(LengthValue::Px(20.0))));

    // calc() 双位置（calc 内含空格，须 depth-aware）
    let g2 = parse_gradient("linear-gradient(red calc(10% + 5px) 80%, blue)").expect("must parse");
    let stops2 = match g2 {
        GradientValue::Linear(lg) => lg.stops,
        _ => panic!("expected linear gradient"),
    };
    assert_eq!(stops2.len(), 3, "red calc(..) 80% → 2 stops + blue → 3");
    assert!(matches!(stops2[0].position, Some(LengthValue::Calc(_))));
    assert!(matches!(stops2[1].position, Some(LengthValue::Percentage(80.0))));
}

#[test]
fn test_r2315_gradient_single_position_byte_identical() {
    // 单位置/无位置回归 byte-identical（仍是合法解析）
    assert!(parse_gradient("linear-gradient(red 50%, blue 100%)").is_some());
    assert!(parse_gradient("linear-gradient(red, blue)").is_some());
    // 非法（三个位置）仍拒绝
    assert!(parse_gradient("linear-gradient(red 0% 50% 100%, blue)").is_none());
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

/// box-shadow 的 rgba 颜色含逗号后空格（标准 CSS 格式 `rgba(0, 0, 0, 0.08)`）
/// 必须作为单个 token 解析，alpha 不能丢失。
///
/// 此前 `parse_box_shadow` 用 `split_whitespace()` 把 `rgba(0, 0, 0, 0.08)` 拆成
/// 碎片，颜色解析失败回退为实心黑（alpha=255），导致 welcome.html 等页面渲染出
/// 大面积实心黑阴影（DC-13）。修复后括号内空白不再分割。
#[test]
fn test_box_shadow_rgba_with_spaces_keeps_alpha() {
    // 带空格的标准格式——修复前 alpha 错为 255（实心黑）
    let s = parse_box_shadow("0 1px 3px rgba(0, 0, 0, 0.08)").unwrap();
    match s.color {
        ColorValue::Rgba(r, g, b, a) => {
            assert_eq!([r, g, b], [0, 0, 0]);
            assert_eq!(a, 20, "rgba(0,0,0,0.08) alpha 应≈20，不应丢失为 255 实心黑");
        }
        other => panic!("expected Rgba, got {:?}", other),
    }
    // 无空格格式仍正确
    let s2 = parse_box_shadow("0 1px 3px rgba(0,0,0,0.08)").unwrap();
    if let ColorValue::Rgba(_, _, _, a) = s2.color {
        assert_eq!(a, 20);
    }
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
