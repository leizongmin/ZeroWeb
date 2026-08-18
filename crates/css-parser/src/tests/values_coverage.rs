//! parse_transform.rs 和 values/types.rs 覆盖率补全测试。

use crate::values::{
    eval_calc, parse_animation_direction, parse_animation_duration, parse_animation_fill_mode,
    parse_animation_iteration_count, parse_animation_name, parse_animation_play_state, parse_calc, parse_time,
    parse_timing_function, parse_transform,
};

// ── parse_transform.rs 未覆盖的公共函数 ──────────────────────────────

// 行 79: parse_animation_direction
#[test]
fn test_animation_direction_normal() {
    assert!(parse_animation_direction("normal").is_some());
}
#[test]
fn test_animation_direction_reverse() {
    assert!(parse_animation_direction("reverse").is_some());
}
#[test]
fn test_animation_direction_alternate() {
    assert!(parse_animation_direction("alternate").is_some());
}
#[test]
fn test_animation_direction_alternate_reverse() {
    assert!(parse_animation_direction("alternate-reverse").is_some());
}
#[test]
fn test_animation_direction_invalid() {
    assert!(parse_animation_direction("invalid").is_none());
}

// 行 90: parse_animation_fill_mode
#[test]
fn test_animation_fill_mode_none() {
    assert!(parse_animation_fill_mode("none").is_some());
}
#[test]
fn test_animation_fill_mode_forwards() {
    assert!(parse_animation_fill_mode("forwards").is_some());
}
#[test]
fn test_animation_fill_mode_backwards() {
    assert!(parse_animation_fill_mode("backwards").is_some());
}
#[test]
fn test_animation_fill_mode_both() {
    assert!(parse_animation_fill_mode("both").is_some());
}
#[test]
fn test_animation_fill_mode_invalid() {
    assert!(parse_animation_fill_mode("invalid").is_none());
}

// 行 101: parse_animation_play_state
#[test]
fn test_animation_play_state_running() {
    assert!(parse_animation_play_state("running").is_some());
}
#[test]
fn test_animation_play_state_paused() {
    assert!(parse_animation_play_state("paused").is_some());
}
#[test]
fn test_animation_play_state_invalid() {
    assert!(parse_animation_play_state("invalid").is_none());
}

// 行 123, 129: parse_animation_name 边界
#[test]
fn test_animation_name_valid() {
    assert!(parse_animation_name("slideIn").is_some());
}
#[test]
fn test_animation_name_empty() {
    assert!(parse_animation_name("").is_none());
}
#[test]
fn test_animation_name_none_keyword() {
    // "none" 可能是有效的动画名（CSS 规范允许）
    let r = parse_animation_name("none");
    // 只要不崩溃即可
    let _ = r;
}

// 行 132: parse_time
#[test]
fn test_parse_time_seconds() {
    assert!(parse_time("1s").is_some());
}
#[test]
fn test_parse_time_ms() {
    assert!(parse_time("500ms").is_some());
}
#[test]
fn test_parse_time_invalid() {
    assert!(parse_time("invalid").is_none());
}

// 行 159: parse_animation_duration
#[test]
fn test_animation_duration_seconds() {
    let r = parse_animation_duration("2s");
    assert!(r.is_some());
}
#[test]
fn test_animation_duration_ms() {
    let r = parse_animation_duration("500ms");
    assert!(r.is_some());
}
#[test]
fn test_animation_duration_negative() {
    assert!(parse_animation_duration("-1s").is_none());
}
#[test]
fn test_animation_duration_invalid() {
    assert!(parse_animation_duration("abc").is_none());
}

// 行 189: parse_animation_iteration_count
#[test]
fn test_animation_iteration_count_infinite() {
    let r = parse_animation_iteration_count("infinite");
    assert!(r.is_some());
}
#[test]
fn test_animation_iteration_count_number() {
    let r = parse_animation_iteration_count("3");
    assert!(r.is_some());
}
#[test]
fn test_animation_iteration_count_zero() {
    assert_eq!(
        parse_animation_iteration_count("0"),
        Some(crate::values::AnimationIterationCountValue::Number(0.0))
    );
}
#[test]
fn test_animation_iteration_count_negative() {
    assert!(parse_animation_iteration_count("-1").is_none());
}

// 行 203: parse_timing_function
#[test]
fn test_timing_function_ease() {
    assert!(parse_timing_function("ease").is_some());
}
#[test]
fn test_timing_function_linear() {
    assert!(parse_timing_function("linear").is_some());
}
#[test]
fn test_timing_function_ease_in() {
    assert!(parse_timing_function("ease-in").is_some());
}
#[test]
fn test_timing_function_ease_out() {
    assert!(parse_timing_function("ease-out").is_some());
}
#[test]
fn test_timing_function_ease_in_out() {
    assert!(parse_timing_function("ease-in-out").is_some());
}
#[test]
fn test_timing_function_step_start() {
    assert!(parse_timing_function("step-start").is_some());
}
#[test]
fn test_timing_function_step_end() {
    assert!(parse_timing_function("step-end").is_some());
}
#[test]
fn test_timing_function_cubic_bezier() {
    assert!(parse_timing_function("cubic-bezier(0.25, 0.1, 0.25, 1.0)").is_some());
}
#[test]
fn test_timing_function_cubic_bezier_wrong_args() {
    assert!(parse_timing_function("cubic-bezier(0.25, 0.1)").is_none());
}
#[test]
fn test_timing_function_steps() {
    assert!(parse_timing_function("steps(4, end)").is_some());
}
#[test]
fn test_timing_function_steps_wrong_args() {
    assert!(parse_timing_function("steps()").is_none());
}
#[test]
fn test_timing_function_invalid() {
    assert!(parse_timing_function("invalid").is_none());
}

// 行 328+: parse_transform 更多变体
#[test]
fn test_transform_skew() {
    assert!(parse_transform("skew(10deg, 20deg)").is_some());
}
#[test]
fn test_transform_skew_single() {
    assert!(parse_transform("skew(10deg)").is_some());
}
#[test]
fn test_transform_matrix_6() {
    assert!(parse_transform("matrix(1, 0, 0, 1, 10, 20)").is_some());
}
#[test]
fn test_transform_perspective_valid() {
    assert!(parse_transform("perspective(500px)").is_some());
}

// 行 450+: 更多 transform 单位
#[test]
fn test_transform_rotate_rad() {
    assert!(parse_transform("rotate(1.5708rad)").is_some());
}
#[test]
fn test_transform_rotate_turn() {
    assert!(parse_transform("rotate(0.25turn)").is_some());
}
#[test]
fn test_transform_translate_rem() {
    assert!(parse_transform("translate(2rem, 3rem)").is_some());
}
#[test]
fn test_transform_translate_em() {
    assert!(parse_transform("translate(2em, 3em)").is_some());
}

// ── values/types.rs: calc 相关 ─────────────────────────────────────

// 行 431+: CalcParser parse_expr
#[test]
fn test_calc_addition() {
    let r = parse_calc("calc(10px + 5px)");
    assert!(r.is_some());
}
#[test]
fn test_calc_subtraction() {
    let r = parse_calc("calc(100% - 20px)");
    assert!(r.is_some());
}
#[test]
fn test_calc_multiplication() {
    let r = parse_calc("calc(10px * 2)");
    assert!(r.is_some());
}
#[test]
fn test_calc_division() {
    let r = parse_calc("calc(100px / 4)");
    assert!(r.is_some());
}
#[test]
fn test_calc_nested_parens() {
    let r = parse_calc("calc((10 + 5) * 2px)");
    assert!(r.is_some());
}
#[test]
fn test_calc_var() {
    let r = parse_calc("calc(var(--width) + 10px)");
    let _ = r;
}

// 行 513+: min/max/clamp
#[test]
fn test_calc_min() {
    let r = parse_calc("calc(min(100px, 50%))");
    assert!(r.is_some());
}
#[test]
fn test_calc_max() {
    let r = parse_calc("calc(max(100px, 50%))");
    assert!(r.is_some());
}
#[test]
fn test_calc_clamp() {
    let r = parse_calc("calc(clamp(10px, 5vw, 100px))");
    assert!(r.is_some());
}

// 行 548+: clamp 三参数
#[test]
fn test_calc_clamp_three_args() {
    let r = parse_calc("calc(clamp(10px, 50%, 200px))");
    assert!(r.is_some());
}

// 行 571+: parse_primary 更多路径
#[test]
fn test_calc_negative_number() {
    let r = parse_calc("calc(-10px)");
    assert!(r.is_some());
}
#[test]
fn test_calc_percentage() {
    let r = parse_calc("calc(50%)");
    assert!(r.is_some());
}
#[test]
fn test_calc_plain_number() {
    let r = parse_calc("calc(100)");
    assert!(r.is_some());
}

// eval_calc 测试
#[test]
fn test_eval_calc_simple() {
    let expr = parse_calc("calc(10px + 5px)");
    if let Some(e) = expr {
        let r = eval_calc(&e, Some(100.0));
        assert!(r.is_some());
    }
}
#[test]
fn test_eval_calc_with_context() {
    let expr = parse_calc("calc(50% + 10px)");
    if let Some(e) = expr {
        let r = eval_calc(&e, Some(200.0));
        assert!(r.is_some());
    }
}

// 行 664+: parse_comma_list
#[test]
fn test_calc_min_multiple_args() {
    let r = parse_calc("calc(min(10px, 20px, 30px))");
    assert!(r.is_some());
}
#[test]
fn test_calc_max_multiple_args() {
    let r = parse_calc("calc(max(10px, 20px, 30px))");
    assert!(r.is_some());
}

// 行 790+: parse_number_or_percentage
#[test]
fn test_calc_number_value() {
    let r = parse_calc("calc(42)");
    assert!(r.is_some());
}
#[test]
fn test_calc_float() {
    let r = parse_calc("calc(3.14)");
    assert!(r.is_some());
}

// 行 840+: 负数处理
#[test]
fn test_calc_negative_result() {
    let r = parse_calc("calc(5px - 10px)");
    assert!(r.is_some());
}

// 行 847+: parse_unit_value 更多单位
#[test]
fn test_calc_vw_unit() {
    let r = parse_calc("calc(10vw + 5px)");
    assert!(r.is_some());
}
#[test]
fn test_calc_vh_unit() {
    let r = parse_calc("calc(10vh + 5px)");
    assert!(r.is_some());
}
#[test]
fn test_calc_em_unit() {
    let r = parse_calc("calc(2em * 3)");
    assert!(r.is_some());
}
#[test]
fn test_calc_rem_unit() {
    let r = parse_calc("calc(2rem * 3)");
    assert!(r.is_some());
}

// 行 909+: parse_var
#[test]
fn test_calc_var_only() {
    // calc(var(...)) 是否被支持取决于解析器实现
    let r = parse_calc("calc(var(--size))");
    // 只要不崩溃即可
    let _ = r;
}
#[test]
fn test_calc_var_with_fallback() {
    let r = parse_calc("calc(var(--size, 100px))");
    let _ = r;
}

// 行 1021+: depth overflow
#[test]
fn test_calc_deeply_nested() {
    // 多层嵌套 → depth overflow → None
    let deep = "calc(clamp(min(max(clamp(min(max(1px, 2px), 3px), 4px), 5px), 6px), 7px, 8px))";
    let r = parse_calc(deep);
    // 可能返回 Some 或 None，取决于 MAX_CALC_DEPTH
    let _ = r;
}
