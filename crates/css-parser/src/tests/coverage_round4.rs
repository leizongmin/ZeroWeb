// Coverage round 4 - targeting uncovered branches in color.rs, parse_transform.rs, parser.rs, tokenizer.rs

use crate::ast::*;
use crate::parser::Parser;
use crate::tokenizer::{Token, Tokenizer};
use crate::values::{
    AnimationDirectionValue, AnimationDurationValue, AnimationFillModeValue, AnimationIterationCountValue,
    AnimationNameValue, AnimationPlayStateValue, ColorValue, GradientDirection, GradientValue, StepPosition, TimeUnit,
    TimingFunctionValue, TransformFunction, TransformValue, hwb_to_rgba, parse_animation_direction,
    parse_animation_duration, parse_animation_fill_mode, parse_animation_iteration_count, parse_animation_name,
    parse_animation_play_state, parse_color, parse_gradient, parse_time, parse_timing_function, parse_transform,
};

// ═══════════════════════════════════════════════════════════════════════
// color.rs coverage
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_color_hex_invalid_length() {
    assert!(parse_color("#12").is_none());
    assert!(parse_color("#12345").is_none());
    assert!(parse_color("#1234567").is_none());
}

#[test]
fn test_color_hex_3_digit() {
    assert_eq!(parse_color("#f00"), Some(ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
fn test_color_hex_4_digit() {
    assert_eq!(parse_color("#f00f"), Some(ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
fn test_color_hex_6_digit() {
    assert_eq!(parse_color("#ff8000"), Some(ColorValue::Rgba(255, 128, 0, 255)));
}

#[test]
fn test_color_hex_8_digit() {
    assert_eq!(parse_color("#ff000080"), Some(ColorValue::Rgba(255, 0, 0, 128)));
}

#[test]
fn test_color_hex_invalid_chars() {
    let c = parse_color("#ggg");
    // R3344：`g` 非 hex digit，非法 hex 颜色须拒绝（与 6/8 位路径一致）。旧实现
    // hex_char_to_byte 用 unwrap_or(0) 把 #ggg 误转为黑色——已修正为 Option<u8>。
    assert!(c.is_none());
}

#[test]
fn test_color_rgb_basic() {
    assert_eq!(
        parse_color("rgb(255, 128, 0)"),
        Some(ColorValue::Rgba(255, 128, 0, 255))
    );
}

#[test]
/// CSS Values §4：颜色函数名大小写不敏感（RGB/RGBA/HSL/HWB/LAB/OKLAB ≡ 小写形式）。
fn test_color_function_names_case_insensitive() {
    // 已知解析结果的函数用显式期望值。
    assert_eq!(
        parse_color("RGB(255, 128, 0)"),
        Some(ColorValue::Rgba(255, 128, 0, 255))
    );
    assert_eq!(
        parse_color("RGBA(255, 128, 0, 0.5)"),
        Some(ColorValue::Rgba(255, 128, 0, 128))
    );
    // 其余用「大写 ≡ 小写」断言（同时守护小写基线非 None）。
    let (hsl_lo, hsl_up) = (parse_color("hsl(120, 100%, 50%)"), parse_color("HSL(120, 100%, 50%)"));
    assert!(hsl_lo.is_some());
    assert_eq!(hsl_up, hsl_lo);
    let (hwb_lo, hwb_up) = (parse_color("hwb(120 30% 20%)"), parse_color("HWB(120 30% 20%)"));
    assert!(hwb_lo.is_some());
    assert_eq!(hwb_up, hwb_lo);
    let (lab_lo, lab_up) = (parse_color("lab(50% 40 30)"), parse_color("LAB(50% 40 30)"));
    assert!(lab_lo.is_some());
    assert_eq!(lab_up, lab_lo);
    let (oklab_lo, oklab_up) = (parse_color("oklab(0.5 0.1 0.1)"), parse_color("OKLAB(0.5 0.1 0.1)"));
    assert!(oklab_lo.is_some());
    assert_eq!(oklab_up, oklab_lo);
}

#[test]
fn test_color_rgba_with_alpha() {
    assert_eq!(
        parse_color("rgba(255, 128, 0, 0.5)"),
        Some(ColorValue::Rgba(255, 128, 0, 128))
    );
}

#[test]
fn test_color_rgb_percentage() {
    assert_eq!(
        parse_color("rgb(100%, 50%, 0%)"),
        Some(ColorValue::Rgba(255, 128, 0, 255))
    );
}

#[test]
fn test_color_rgb_too_few_args() {
    assert!(parse_color("rgb(255, 128)").is_none());
}

#[test]
fn test_color_rgb_non_numeric() {
    assert!(parse_color("rgb(a, b, c)").is_none());
}

#[test]
fn test_color_rgb_empty_string() {
    assert!(parse_color("").is_none());
}

#[test]
fn test_color_hsl_basic() {
    assert_eq!(
        parse_color("hsl(0, 100%, 50%)"),
        Some(ColorValue::Hsla(0.0, 100.0, 50.0, 1.0))
    );
}

#[test]
fn test_color_hsla_with_alpha() {
    assert_eq!(
        parse_color("hsla(120, 50%, 75%, 0.5)"),
        Some(ColorValue::Hsla(120.0, 50.0, 75.0, 0.5))
    );
}

#[test]
fn test_color_hsl_too_few_args() {
    assert!(parse_color("hsl(0, 100%)").is_none());
}

#[test]
fn test_color_hsl_deg_suffix() {
    let c = parse_color("hsl(180deg, 50%, 50%)");
    assert!(c.is_some());
}

#[test]
fn test_color_hwb_basic() {
    let c = parse_color("hwb(0 0% 0%)").unwrap();
    assert!(matches!(c, ColorValue::Rgba(r, _, _, 255) if r == 255));
}

#[test]
fn test_color_hwb_with_alpha() {
    let c = parse_color("hwb(120 20% 30% / 0.5)").unwrap();
    assert!(matches!(c, ColorValue::Rgba(_, _, _, 128)));
}

#[test]
fn test_color_hwb_alpha_percentage() {
    let c = parse_color("hwb(240 0% 100% / 50%)");
    assert!(c.is_some());
}

#[test]
fn test_color_hwb_too_few_args() {
    assert!(parse_color("hwb(0 0%)").is_none());
}

#[test]
fn test_color_named_case_insensitive() {
    let c1 = parse_color("RED").unwrap();
    let c2 = parse_color("red").unwrap();
    let c3 = parse_color("Red").unwrap();
    assert_eq!(c1, c2);
    assert_eq!(c2, c3);
}

#[test]
fn test_color_named_transparent() {
    assert_eq!(parse_color("transparent"), Some(ColorValue::Transparent));
    assert_eq!(parse_color("TRANSPARENT"), Some(ColorValue::Transparent));
}

#[test]
fn test_color_named_current_color() {
    assert_eq!(parse_color("currentColor"), Some(ColorValue::CurrentColor));
    assert_eq!(parse_color("CURRENTCOLOR"), Some(ColorValue::CurrentColor));
}

#[test]
fn test_color_unknown_name() {
    assert!(parse_color("notacolor").is_none());
}

#[test]
fn test_color_named_aliases() {
    // cyan/aqua alias
    assert_eq!(parse_color("cyan"), parse_color("aqua"));
    // gray/grey alias
    assert_eq!(parse_color("gray"), parse_color("grey"));
}

#[test]
fn test_hwb_to_rgba_all_sectors() {
    let r0 = hwb_to_rgba(0.0, 0.0, 0.0, 1.0);
    assert_eq!(r0.0, 255);
    let r1 = hwb_to_rgba(90.0, 0.0, 0.0, 1.0);
    assert!(r1.0 > 0 || r1.1 > 0);
    let r2 = hwb_to_rgba(150.0, 0.0, 0.0, 1.0);
    assert!(r2.1 > 0);
    let r3 = hwb_to_rgba(210.0, 0.0, 0.0, 1.0);
    assert!(r3.2 > 0);
    let r4 = hwb_to_rgba(270.0, 0.0, 0.0, 1.0);
    assert!(r4.2 > 0);
    let r5 = hwb_to_rgba(330.0, 0.0, 0.0, 1.0);
    assert!(r5.0 > 0);
}

#[test]
fn test_hwb_to_rgba_wb_exceeds_1() {
    let r = hwb_to_rgba(0.0, 0.8, 0.8, 1.0);
    assert!(r.0 > 100 && r.0 < 200);
}

#[test]
fn test_hwb_to_rgba_clamp() {
    let r = hwb_to_rgba(0.0, 1.5, -0.5, 2.0);
    assert_eq!(r.0, 255);
    assert_eq!(r.1, 255);
    assert_eq!(r.2, 255);
}

#[test]
fn test_hwb_to_rgba_alpha_clamp() {
    let r = hwb_to_rgba(0.0, 0.0, 0.0, -1.0);
    assert_eq!(r.3, 0);
}

// ═══════════════════════════════════════════════════════════════════════
// parse_transform.rs coverage
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_transform_none() {
    assert_eq!(parse_transform("none"), Some(TransformValue::None));
    assert_eq!(parse_transform("NONE"), Some(TransformValue::None));
}

#[test]
fn test_transform_translate_single_arg() {
    let t = parse_transform("translate(10px)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    let TransformFunction::Translate(tx, ty) = &fs[0] else {
        panic!("Expected Translate")
    };
    assert!((*tx - 10.0).abs() < 0.01);
    assert!((*ty - 0.0).abs() < 0.01);
}

#[test]
fn test_transform_translate_two_args() {
    let t = parse_transform("translate(10px, 20px)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    let TransformFunction::Translate(tx, ty) = &fs[0] else {
        panic!("Expected Translate")
    };
    assert!((*tx - 10.0).abs() < 0.01);
    assert!((*ty - 20.0).abs() < 0.01);
}

#[test]
fn test_transform_translate_x_y() {
    let t1 = parse_transform("translateX(15px)").unwrap();
    let TransformValue::List(fs1) = &t1 else {
        panic!("Expected List")
    };
    let TransformFunction::TranslateX(tx) = &fs1[0] else {
        panic!("Expected TranslateX")
    };
    assert!((*tx - 15.0).abs() < 0.01);

    let t2 = parse_transform("translateY(25px)").unwrap();
    let TransformValue::List(fs2) = &t2 else {
        panic!("Expected List")
    };
    let TransformFunction::TranslateY(ty) = &fs2[0] else {
        panic!("Expected TranslateY")
    };
    assert!((*ty - 25.0).abs() < 0.01);
}

#[test]
fn test_transform_rotate_deg() {
    let t = parse_transform("rotate(45deg)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    let TransformFunction::Rotate(a) = &fs[0] else {
        panic!("Expected Rotate")
    };
    assert!((*a - 45.0).abs() < 0.01);
}

#[test]
fn test_transform_rotate_rad() {
    let t = parse_transform("rotate(1.5708rad)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    let TransformFunction::Rotate(a) = &fs[0] else {
        panic!("Expected Rotate")
    };
    assert!((*a - 90.0).abs() < 1.0);
}

#[test]
fn test_transform_rotate_turn() {
    let t = parse_transform("rotate(0.25turn)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    let TransformFunction::Rotate(a) = &fs[0] else {
        panic!("Expected Rotate")
    };
    assert!((*a - 90.0).abs() < 0.01);
}

#[test]
fn test_transform_scale_single() {
    let t = parse_transform("scale(2)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    let TransformFunction::Scale(sx, sy) = &fs[0] else {
        panic!("Expected Scale")
    };
    assert!((*sx - 2.0).abs() < 0.01);
    assert!(sy.is_none());
}

#[test]
fn test_transform_scale_two() {
    let t = parse_transform("scale(2, 3)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    let TransformFunction::Scale(sx, sy) = &fs[0] else {
        panic!("Expected Scale")
    };
    assert!((*sx - 2.0).abs() < 0.01);
    assert_eq!(sy.unwrap(), 3.0);
}

#[test]
fn test_transform_scale_x_y() {
    let t1 = parse_transform("scaleX(1.5)").unwrap();
    let TransformValue::List(fs1) = &t1 else {
        panic!("Expected List")
    };
    let TransformFunction::ScaleX(sx) = &fs1[0] else {
        panic!("Expected ScaleX")
    };
    assert!((*sx - 1.5).abs() < 0.01);

    let t2 = parse_transform("scaleY(0.5)").unwrap();
    let TransformValue::List(fs2) = &t2 else {
        panic!("Expected List")
    };
    let TransformFunction::ScaleY(sy) = &fs2[0] else {
        panic!("Expected ScaleY")
    };
    assert!((*sy - 0.5).abs() < 0.01);
}

#[test]
fn test_transform_skew() {
    let t1 = parse_transform("skew(10deg)").unwrap();
    let TransformValue::List(fs1) = &t1 else {
        panic!("Expected List")
    };
    let TransformFunction::Skew(ax, ay) = &fs1[0] else {
        panic!("Expected Skew")
    };
    assert!((*ax - 10.0).abs() < 0.01);
    assert!(ay.is_none());

    let t2 = parse_transform("skew(10deg, 20deg)").unwrap();
    let TransformValue::List(fs2) = &t2 else {
        panic!("Expected List")
    };
    let TransformFunction::Skew(ax2, ay2) = &fs2[0] else {
        panic!("Expected Skew")
    };
    assert!((*ax2 - 10.0).abs() < 0.01);
    assert_eq!(ay2.unwrap(), 20.0);
}

#[test]
fn test_transform_3d_functions() {
    let t1 = parse_transform("translate3d(10, 20, 30)").unwrap();
    let TransformValue::List(fs1) = &t1 else {
        panic!("Expected List")
    };
    let TransformFunction::Translate3d(tx, ty, tz) = &fs1[0] else {
        panic!("Expected Translate3d")
    };
    assert!((*tx - 10.0).abs() < 0.01);

    let t2 = parse_transform("scale3d(1, 2, 3)").unwrap();
    let TransformValue::List(fs2) = &t2 else {
        panic!("Expected List")
    };
    let TransformFunction::Scale3d(_, _, _) = &fs2[0] else {
        panic!("Expected Scale3d")
    };

    let t3 = parse_transform("rotate3d(1, 0, 0, 45deg)").unwrap();
    let TransformValue::List(fs3) = &t3 else {
        panic!("Expected List")
    };
    let TransformFunction::Rotate3d(_, _, _, a) = &fs3[0] else {
        panic!("Expected Rotate3d")
    };
    assert!((*a - 45.0).abs() < 0.01);
}

#[test]
fn test_transform_rotate_xyz() {
    let t1 = parse_transform("rotateX(30deg)").unwrap();
    let TransformValue::List(fs1) = &t1 else {
        panic!("Expected List")
    };
    let TransformFunction::RotateX(a) = &fs1[0] else {
        panic!("Expected RotateX")
    };
    assert!((*a - 30.0).abs() < 0.01);

    let t2 = parse_transform("rotateY(60deg)").unwrap();
    let TransformValue::List(fs2) = &t2 else {
        panic!("Expected List")
    };
    let TransformFunction::RotateY(a) = &fs2[0] else {
        panic!("Expected RotateY")
    };
    assert!((*a - 60.0).abs() < 0.01);

    let t3 = parse_transform("rotateZ(90deg)").unwrap();
    let TransformValue::List(fs3) = &t3 else {
        panic!("Expected List")
    };
    let TransformFunction::RotateZ(a) = &fs3[0] else {
        panic!("Expected RotateZ")
    };
    assert!((*a - 90.0).abs() < 0.01);
}

#[test]
fn test_transform_perspective() {
    let t = parse_transform("perspective(500px)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    let TransformFunction::Perspective(v) = &fs[0] else {
        panic!("Expected Perspective")
    };
    assert!((*v - 500.0).abs() < 0.01);
}

#[test]
fn test_transform_perspective_invalid() {
    assert!(parse_transform("perspective(0)").is_none());
    assert!(parse_transform("perspective(-100)").is_none());
}

#[test]
fn test_transform_matrix() {
    let t = parse_transform("matrix(1, 0, 0, 1, 10, 20)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    let TransformFunction::Matrix(a, _b, _c, _d, e, _f) = &fs[0] else {
        panic!("Expected Matrix")
    };
    assert!((*a - 1.0).abs() < 0.01 && (*e - 10.0).abs() < 0.01);
}

#[test]
fn test_transform_matrix_wrong_args() {
    assert!(parse_transform("matrix(1, 0, 0, 1)").is_none());
}

#[test]
fn test_transform_wrong_arg_counts() {
    assert!(parse_transform("translate3d(10, 20)").is_none());
    assert!(parse_transform("rotate3d(1, 0, 0)").is_none());
    assert!(parse_transform("scale3d(1, 2)").is_none());
}

#[test]
fn test_transform_unknown_function() {
    assert!(parse_transform("fooBar(10px)").is_none());
}

#[test]
fn test_transform_empty() {
    assert!(parse_transform("").is_none());
}

#[test]
fn test_transform_multiple_functions() {
    let t = parse_transform("translate(10px, 20px) rotate(45deg) scale(2)").unwrap();
    let TransformValue::List(fs) = &t else {
        panic!("Expected List")
    };
    assert_eq!(fs.len(), 3);
}

#[test]
fn test_transform_no_paren() {
    assert!(parse_transform("translate").is_none());
}

// ── timing function coverage ──

#[test]
fn test_timing_function_keywords() {
    assert_eq!(parse_timing_function("ease"), Some(TimingFunctionValue::Ease));
    assert_eq!(parse_timing_function("linear"), Some(TimingFunctionValue::Linear));
    assert_eq!(parse_timing_function("ease-in"), Some(TimingFunctionValue::EaseIn));
    assert_eq!(parse_timing_function("ease-out"), Some(TimingFunctionValue::EaseOut));
    assert_eq!(
        parse_timing_function("ease-in-out"),
        Some(TimingFunctionValue::EaseInOut)
    );
    assert_eq!(
        parse_timing_function("step-start"),
        Some(TimingFunctionValue::StepStart)
    );
    assert_eq!(parse_timing_function("step-end"), Some(TimingFunctionValue::StepEnd));
}

#[test]
fn test_timing_function_cubic_bezier() {
    let cb = parse_timing_function("cubic-bezier(0.1, 0.9, 0.2, 1.0)").unwrap();
    assert_eq!(cb, TimingFunctionValue::CubicBezier(0.1, 0.9, 0.2, 1.0));
}

#[test]
fn test_timing_function_cubic_bezier_wrong_args() {
    assert!(parse_timing_function("cubic-bezier(0.1, 0.9)").is_none());
}

#[test]
fn test_timing_function_steps() {
    assert_eq!(
        parse_timing_function("steps(4)"),
        Some(TimingFunctionValue::Steps(4, None))
    );
    assert_eq!(
        parse_timing_function("steps(4, start)"),
        Some(TimingFunctionValue::Steps(4, Some(StepPosition::Start)))
    );
    assert_eq!(
        parse_timing_function("steps(4, end)"),
        Some(TimingFunctionValue::Steps(4, Some(StepPosition::End)))
    );
    assert_eq!(
        parse_timing_function("steps(4, jump-start)"),
        Some(TimingFunctionValue::Steps(4, Some(StepPosition::Start)))
    );
    assert_eq!(
        parse_timing_function("steps(4, jump-end)"),
        Some(TimingFunctionValue::Steps(4, Some(StepPosition::End)))
    );
    assert_eq!(
        parse_timing_function("steps(4, jump-both)"),
        Some(TimingFunctionValue::Steps(4, Some(StepPosition::Both)))
    );
    assert_eq!(
        parse_timing_function("steps(4, jump-none)"),
        Some(TimingFunctionValue::Steps(4, Some(StepPosition::None)))
    );
}

#[test]
fn test_timing_function_invalid() {
    assert!(parse_timing_function("unknown-func").is_none());
    assert!(parse_timing_function("steps(4, invalid)").is_none());
}

// ── animation functions ──

#[test]
fn test_animation_duration() {
    assert_eq!(
        parse_animation_duration("1s"),
        Some(AnimationDurationValue::Time(1.0, TimeUnit::S))
    );
    assert_eq!(
        parse_animation_duration("500ms"),
        Some(AnimationDurationValue::Time(500.0, TimeUnit::Ms))
    );
    assert!(parse_animation_duration("-1s").is_none());
    assert!(parse_animation_duration("abc").is_none());
}

#[test]
fn test_animation_iteration_count() {
    assert_eq!(
        parse_animation_iteration_count("infinite"),
        Some(AnimationIterationCountValue::Infinite)
    );
    assert_eq!(
        parse_animation_iteration_count("INFINITE"),
        Some(AnimationIterationCountValue::Infinite)
    );
    assert_eq!(
        parse_animation_iteration_count("3"),
        Some(AnimationIterationCountValue::Number(3.0))
    );
    assert_eq!(
        parse_animation_iteration_count("0"),
        Some(AnimationIterationCountValue::Number(0.0))
    );
    assert!(parse_animation_iteration_count("-1").is_none());
}

#[test]
fn test_animation_name() {
    assert_eq!(parse_animation_name("none"), Some(AnimationNameValue::None));
    assert_eq!(
        parse_animation_name("fadeIn"),
        Some(AnimationNameValue::Custom("fadeIn".to_string()))
    );
    assert!(parse_animation_name("").is_none());
    assert!(parse_animation_name("123abc").is_none());
    assert!(parse_animation_name("fade in").is_none());
}

#[test]
fn test_animation_fill_mode() {
    assert_eq!(parse_animation_fill_mode("none"), Some(AnimationFillModeValue::None));
    assert_eq!(
        parse_animation_fill_mode("forwards"),
        Some(AnimationFillModeValue::Forwards)
    );
    assert_eq!(
        parse_animation_fill_mode("backwards"),
        Some(AnimationFillModeValue::Backwards)
    );
    assert_eq!(parse_animation_fill_mode("both"), Some(AnimationFillModeValue::Both));
    assert!(parse_animation_fill_mode("invalid").is_none());
}

#[test]
fn test_animation_play_state() {
    assert_eq!(
        parse_animation_play_state("running"),
        Some(AnimationPlayStateValue::Running)
    );
    assert_eq!(
        parse_animation_play_state("paused"),
        Some(AnimationPlayStateValue::Paused)
    );
    assert!(parse_animation_play_state("invalid").is_none());
}

#[test]
fn test_animation_direction() {
    assert_eq!(
        parse_animation_direction("normal"),
        Some(AnimationDirectionValue::Normal)
    );
    assert_eq!(
        parse_animation_direction("reverse"),
        Some(AnimationDirectionValue::Reverse)
    );
    assert_eq!(
        parse_animation_direction("alternate"),
        Some(AnimationDirectionValue::Alternate)
    );
    assert_eq!(
        parse_animation_direction("alternate-reverse"),
        Some(AnimationDirectionValue::AlternateReverse)
    );
    assert!(parse_animation_direction("invalid").is_none());
}

#[test]
fn test_parse_time() {
    let t1 = parse_time("0.3s").unwrap();
    assert!((t1 - 0.3).abs() < f64::EPSILON);
    let t2 = parse_time("200ms").unwrap();
    assert!((t2 - 0.2).abs() < f64::EPSILON);
    assert!(parse_time("abc").is_none());
}

#[test]
/// R2361：parse_time 时间单位大小写不敏感（CSS Values §：单位大小写不敏感）。
/// `500MS`/`2S` 此前落 None 丢 transition/animation duration/delay 声明。
fn test_parse_time_case_insensitive() {
    assert!((parse_time("500MS").unwrap() - 0.5).abs() < f64::EPSILON, "MS");
    assert!((parse_time("2S").unwrap() - 2.0).abs() < f64::EPSILON, "S");
    assert!((parse_time("200Ms").unwrap() - 0.2).abs() < f64::EPSILON, "Ms mixed");
}

// ═══════════════════════════════════════════════════════════════════════
// parser.rs coverage
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_stylesheet_empty() {
    let stylesheet = Parser::parse_stylesheet("");
    assert!(stylesheet.rules.is_empty());
}

#[test]
fn test_parse_stylesheet_whitespace_only() {
    let stylesheet = Parser::parse_stylesheet("   \n\t  ");
    assert!(stylesheet.rules.is_empty());
}

#[test]
fn test_parse_declaration_missing_semicolon() {
    let stylesheet = Parser::parse_stylesheet("div { color: red }");
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_multiple_rules() {
    let stylesheet = Parser::parse_stylesheet("div { color: red; } span { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 2);
}

#[test]
fn test_parse_empty_declaration_block() {
    let stylesheet = Parser::parse_stylesheet("div {}");
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_at_media() {
    let stylesheet = Parser::parse_stylesheet("@media (max-width: 600px) { div { color: red; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    assert!(matches!(&stylesheet.rules[0], Rule::At(_)));
}

#[test]
fn test_parse_at_keyframes() {
    let stylesheet = Parser::parse_stylesheet("@keyframes slide { from { left: 0; } to { left: 100%; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    assert!(matches!(&stylesheet.rules[0], Rule::Keyframes(_)));
}

#[test]
fn test_parse_at_import() {
    let stylesheet = Parser::parse_stylesheet("@import url('style.css');");
    assert_eq!(stylesheet.rules.len(), 1);
    assert!(matches!(&stylesheet.rules[0], Rule::Import(_)));
}

#[test]
fn test_parse_anonymous_layer() {
    let stylesheet = Parser::parse_stylesheet("@layer { .test { color: red; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Layer(layer) = &stylesheet.rules[0] {
        assert_eq!(layer.name, "");
        assert_eq!(layer.rules.len(), 1);
    } else {
        panic!("Expected Layer rule");
    }
}

#[test]
fn test_parse_named_layer() {
    let stylesheet = Parser::parse_stylesheet("@layer base { .test { color: red; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Layer(layer) = &stylesheet.rules[0] {
        assert_eq!(layer.name, "base");
    } else {
        panic!("Expected Layer rule");
    }
}

#[test]
fn test_parse_important_declaration() {
    let stylesheet = Parser::parse_stylesheet("div { color: red !important; }");
    if let Rule::Style(style) = &stylesheet.rules[0] {
        assert!(style.declarations[0].important);
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
fn test_parse_selector_with_id() {
    let stylesheet = Parser::parse_stylesheet("#myid { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_selector_with_class() {
    let stylesheet = Parser::parse_stylesheet(".myclass { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_selector_with_attribute() {
    let stylesheet = Parser::parse_stylesheet("[data-test] { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_selector_with_pseudo_class() {
    let stylesheet = Parser::parse_stylesheet("div:hover { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
}

#[test]
fn test_parse_unterminated_comment() {
    let stylesheet = Parser::parse_stylesheet("div /* unterminated");
    assert!(stylesheet.rules.len() <= 1);
}

#[test]
fn test_parse_at_supports() {
    let stylesheet = Parser::parse_stylesheet("@supports (display: grid) { .test { display: grid; } }");
    // @supports may be parsed as At rule or another rule type
    assert!(!stylesheet.rules.is_empty());
}

#[test]
fn test_parse_at_container() {
    let stylesheet = Parser::parse_stylesheet("@container (width > 100px) { .test { color: red; } }");
    assert_eq!(stylesheet.rules.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// tokenizer.rs coverage
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tokenize_empty() {
    let tokens: Vec<_> = Tokenizer::new("").collect_tokens();
    // May be empty or contain EOF
    assert!(tokens.len() <= 1);
}

#[test]
fn test_tokenize_simple_ident() {
    let tokens: Vec<_> = Tokenizer::new("div").collect_tokens();
    assert!(matches!(&tokens[0], Token::Ident(s) if s == "div"));
}

#[test]
fn test_tokenize_number() {
    let tokens: Vec<_> = Tokenizer::new("42").collect_tokens();
    assert!(matches!(&tokens[0], Token::Number(n) if (*n - 42.0).abs() < f64::EPSILON));
}

#[test]
fn test_tokenize_float() {
    let tokens: Vec<_> = Tokenizer::new("3.14").collect_tokens();
    assert!(matches!(&tokens[0], Token::Number(n) if (*n - 3.14).abs() < 0.001));
}

#[test]
fn test_tokenize_dimension() {
    let tokens: Vec<_> = Tokenizer::new("10px").collect_tokens();
    assert!(matches!(&tokens[0], Token::Dimension(_, s) if s == "px"));
}

#[test]
fn test_tokenize_string_double() {
    let tokens: Vec<_> = Tokenizer::new("\"hello\"").collect_tokens();
    assert!(matches!(&tokens[0], Token::String(s) if s == "hello"));
}

#[test]
fn test_tokenize_string_single() {
    let tokens: Vec<_> = Tokenizer::new("'world'").collect_tokens();
    assert!(matches!(&tokens[0], Token::String(s) if s == "world"));
}

#[test]
fn test_tokenize_comment() {
    let tokens: Vec<_> = Tokenizer::new("/* comment */").collect_tokens();
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenize_function() {
    let tokens: Vec<_> = Tokenizer::new("rgb(").collect_tokens();
    assert!(matches!(&tokens[0], Token::Function(s) if s == "rgb"));
}

#[test]
fn test_tokenize_url() {
    let tokens: Vec<_> = Tokenizer::new("url(test.png)").collect_tokens();
    assert!(tokens.len() >= 1);
}

#[test]
fn test_tokenize_at_keyword() {
    let tokens: Vec<_> = Tokenizer::new("@media").collect_tokens();
    assert!(matches!(&tokens[0], Token::AtKeyword(s) if s == "media"));
}

#[test]
fn test_tokenize_hash() {
    let tokens: Vec<_> = Tokenizer::new("#fff").collect_tokens();
    assert!(matches!(&tokens[0], Token::Hash(_)));
}

#[test]
fn test_tokenize_delim() {
    let tokens: Vec<_> = Tokenizer::new("{").collect_tokens();
    assert!(matches!(&tokens[0], Token::LBrace));
}

#[test]
fn test_tokenize_colon() {
    let tokens: Vec<_> = Tokenizer::new(":").collect_tokens();
    assert!(matches!(&tokens[0], Token::Colon));
}

#[test]
fn test_tokenize_semicolon() {
    let tokens: Vec<_> = Tokenizer::new(";").collect_tokens();
    assert!(matches!(&tokens[0], Token::Semicolon));
}

#[test]
fn test_tokenize_comma() {
    let tokens: Vec<_> = Tokenizer::new(",").collect_tokens();
    assert!(matches!(&tokens[0], Token::Comma));
}

#[test]
fn test_tokenize_parentheses() {
    let tokens: Vec<_> = Tokenizer::new("()").collect_tokens();
    assert!(matches!(&tokens[0], Token::LParen));
    assert!(matches!(&tokens[1], Token::RParen));
}

#[test]
fn test_tokenize_brackets() {
    let tokens: Vec<_> = Tokenizer::new("[]").collect_tokens();
    assert!(matches!(&tokens[0], Token::LBracket));
    assert!(matches!(&tokens[1], Token::RBracket));
}

#[test]
fn test_tokenize_braces() {
    let tokens: Vec<_> = Tokenizer::new("{}").collect_tokens();
    assert!(matches!(&tokens[0], Token::LBrace));
    assert!(matches!(&tokens[1], Token::RBrace));
}

#[test]
fn test_tokenize_negative_number() {
    let tokens: Vec<_> = Tokenizer::new("-10").collect_tokens();
    // Could be Number(-10) or Delim('-') + Number(10)
    assert!(!tokens.is_empty());
}

#[test]
fn test_tokenize_percentage() {
    let tokens: Vec<_> = Tokenizer::new("50%").collect_tokens();
    assert!(matches!(&tokens[0], Token::Percentage(n) if (*n - 50.0).abs() < f64::EPSILON));
}

#[test]
fn test_tokenize_unicode_escape() {
    let tokens: Vec<_> = Tokenizer::new("\\20AC").collect_tokens();
    // Unicode escape may produce Ident or other token
    assert!(!tokens.is_empty());
}

#[test]
fn test_tokenize_whitespace() {
    let tokens: Vec<_> = Tokenizer::new("  ").collect_tokens();
    assert!(tokens.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// gradient coverage
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_gradient_linear_basic() {
    let g = parse_gradient("linear-gradient(red, blue)").unwrap();
    assert!(matches!(g, GradientValue::Linear(_)));
}

#[test]
fn test_gradient_linear_with_direction() {
    let g = parse_gradient("linear-gradient(to right, red, blue)").unwrap();
    let GradientValue::Linear(lg) = &g else {
        panic!("Expected Linear")
    };
    assert_eq!(lg.direction, GradientDirection::ToRight);
}

#[test]
fn test_gradient_linear_angle() {
    let g = parse_gradient("linear-gradient(45deg, red, blue)").unwrap();
    let GradientValue::Linear(lg) = &g else {
        panic!("Expected Linear")
    };
    let GradientDirection::Angle(a) = &lg.direction else {
        panic!("Expected Angle")
    };
    assert!((*a - 45.0).abs() < 0.01);
}

#[test]
fn test_gradient_radial_basic() {
    assert!(matches!(
        parse_gradient("radial-gradient(red, blue)"),
        Some(GradientValue::Radial(_))
    ));
}

#[test]
fn test_gradient_conic_basic() {
    assert!(matches!(
        parse_gradient("conic-gradient(red, blue)"),
        Some(GradientValue::Conic(_))
    ));
}

#[test]
fn test_gradient_repeating_linear() {
    let g = parse_gradient("repeating-linear-gradient(red, blue)").unwrap();
    let GradientValue::Linear(lg) = &g else {
        panic!("Expected Linear")
    };
    assert!(lg.repeating);
}

#[test]
fn test_gradient_repeating_radial() {
    let g = parse_gradient("repeating-radial-gradient(red, blue)").unwrap();
    let GradientValue::Radial(rg) = &g else {
        panic!("Expected Radial")
    };
    assert!(rg.repeating);
}

#[test]
fn test_gradient_invalid() {
    assert!(parse_gradient("not-a-gradient").is_none());
    assert!(parse_gradient("linear-gradient()").is_none());
}

#[test]
fn test_gradient_linear_direction_corners() {
    let dirs = [
        ("to top left", GradientDirection::ToTopLeft),
        ("to top right", GradientDirection::ToTopRight),
        ("to bottom left", GradientDirection::ToBottomLeft),
        ("to bottom right", GradientDirection::ToBottomRight),
    ];
    for (dir_str, expected) in dirs {
        let g = parse_gradient(&format!("linear-gradient({}, red, blue)", dir_str)).unwrap();
        let GradientValue::Linear(lg) = &g else {
            panic!("Expected Linear")
        };
        assert_eq!(lg.direction, expected, "Failed for direction: {}", dir_str);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// R2357/R2358：枚举/关键字大小写不敏感（CSS Syntax §：所有关键字大小写不敏感）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_animation_enum_keywords_case_insensitive() {
    // 全大写应与全小写等价（apply_advanced.rs 用原始值调用，大小写敏感会丢失声明）。
    assert_eq!(
        parse_animation_direction("ALTERNATE-REVERSE"),
        Some(AnimationDirectionValue::AlternateReverse)
    );
    assert_eq!(
        parse_animation_fill_mode("BACKWARDS"),
        Some(AnimationFillModeValue::Backwards)
    );
    assert_eq!(
        parse_animation_play_state("PAUSED"),
        Some(AnimationPlayStateValue::Paused)
    );
}

#[test]
fn test_timing_function_keywords_case_insensitive() {
    assert_eq!(parse_timing_function("EASE-IN"), Some(TimingFunctionValue::EaseIn));
    assert_eq!(parse_timing_function("LINEAR"), Some(TimingFunctionValue::Linear));
    // 函数名前缀大小写不敏感（数值不变）
    let cb = parse_timing_function("CUBIC-BEZIER(0.1, 0.9, 0.2, 1.0)").unwrap();
    let TimingFunctionValue::CubicBezier(x1, y1, x2, y2) = cb else {
        panic!("Expected CubicBezier");
    };
    assert_eq!((x1, y1, x2, y2), (0.1, 0.9, 0.2, 1.0));
    // steps() 函数名 + 位置关键字大小写不敏感
    assert_eq!(
        parse_timing_function("STEPS(4, JUMP-START)"),
        Some(TimingFunctionValue::Steps(4, Some(StepPosition::Start)))
    );
}
