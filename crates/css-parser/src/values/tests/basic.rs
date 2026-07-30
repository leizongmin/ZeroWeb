// Auto-generated test file — split from values.rs
use super::super::*;

#[test]
fn test_parse_timing_function_keywords() {
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
fn test_parse_timing_function_cubic_bezier() {
    let result = parse_timing_function("cubic-bezier(0.25, 0.1, 0.25, 1.0)");
    assert_eq!(result, Some(TimingFunctionValue::CubicBezier(0.25, 0.1, 0.25, 1.0)));
}

#[test]
fn test_parse_timing_function_steps() {
    assert_eq!(
        parse_timing_function("steps(4)"),
        Some(TimingFunctionValue::Steps(4, None))
    );
    assert_eq!(
        parse_timing_function("steps(4, end)"),
        Some(TimingFunctionValue::Steps(4, Some(StepPosition::End)))
    );
    assert_eq!(
        parse_timing_function("steps(4, start)"),
        Some(TimingFunctionValue::Steps(4, Some(StepPosition::Start)))
    );
    assert_eq!(
        parse_timing_function("steps(2, jump-both)"),
        Some(TimingFunctionValue::Steps(2, Some(StepPosition::Both)))
    );
}

#[test]
fn test_parse_timing_function_invalid() {
    assert_eq!(parse_timing_function("invalid"), None);
}

// ── parse_time ──

#[test]
fn test_parse_time_seconds() {
    assert_eq!(parse_time("0.3s"), Some(0.3));
    assert_eq!(parse_time("1s"), Some(1.0));
    assert_eq!(parse_time("2.5s"), Some(2.5));
}

#[test]
fn test_parse_time_milliseconds() {
    assert_eq!(parse_time("200ms"), Some(0.2));
    assert_eq!(parse_time("1000ms"), Some(1.0));
    assert_eq!(parse_time("50ms"), Some(0.05));
}

#[test]
fn test_parse_time_invalid() {
    assert_eq!(parse_time("10"), None);
    assert_eq!(parse_time("abc"), None);
}

#[test]
fn test_parse_time_zero() {
    assert_eq!(parse_time("0s"), Some(0.0));
    assert_eq!(parse_time("0ms"), Some(0.0));
}

// ── parse_calc ──

#[test]
fn test_parse_calc_percentage_minus_px() {
    let expr = parse_calc("calc(100% - 20px)");
    let expr = expr.expect("should parse calc(100% - 20px)");
    match &expr {
        CalcExpr::BinaryOp(left, op, right) => {
            assert_eq!(**left, CalcExpr::Length(LengthValue::Percentage(100.0)));
            assert_eq!(*op, CalcOp::Subtract);
            assert_eq!(**right, CalcExpr::Length(LengthValue::Px(20.0)));
        }
        _ => panic!("expected BinaryOp, got {expr:?}"),
    }
}

#[test]
fn test_parse_calc_percentage_plus_px() {
    let expr = parse_calc("calc(50% + 10px)");
    let expr = expr.expect("should parse calc(50% + 10px)");
    match &expr {
        CalcExpr::BinaryOp(left, op, right) => {
            assert_eq!(**left, CalcExpr::Length(LengthValue::Percentage(50.0)));
            assert_eq!(*op, CalcOp::Add);
            assert_eq!(**right, CalcExpr::Length(LengthValue::Px(10.0)));
        }
        _ => panic!("expected BinaryOp, got {expr:?}"),
    }
}

#[test]
fn test_parse_calc_multiply() {
    let expr = parse_calc("calc(2 * 10px)");
    let expr = expr.expect("should parse calc(2 * 10px)");
    match &expr {
        CalcExpr::BinaryOp(left, op, right) => {
            assert_eq!(**left, CalcExpr::Number(2.0));
            assert_eq!(*op, CalcOp::Multiply);
            assert_eq!(**right, CalcExpr::Length(LengthValue::Px(10.0)));
        }
        _ => panic!("expected BinaryOp, got {expr:?}"),
    }
}

#[test]
fn test_parse_calc_divide() {
    let expr = parse_calc("calc(100px / 2)");
    let expr = expr.expect("should parse calc(100px / 2)");
    match &expr {
        CalcExpr::BinaryOp(left, op, right) => {
            assert_eq!(**left, CalcExpr::Length(LengthValue::Px(100.0)));
            assert_eq!(*op, CalcOp::Divide);
            assert_eq!(**right, CalcExpr::Number(2.0));
        }
        _ => panic!("expected BinaryOp, got {expr:?}"),
    }
}

#[test]
fn test_eval_calc_percentage_minus_px() {
    let expr = parse_calc("calc(100% - 20px)").unwrap();
    let result = eval_calc(&expr, Some(200.0));
    assert_eq!(result, Some(180.0));
}

#[test]
fn test_eval_calc_percentage_plus_px() {
    let expr = parse_calc("calc(50% + 10px)").unwrap();
    let result = eval_calc(&expr, Some(200.0));
    assert_eq!(result, Some(110.0));
}

#[test]
fn test_eval_calc_multiply() {
    let expr = parse_calc("calc(2 * 10px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(20.0));
}

#[test]
fn test_eval_calc_divide() {
    let expr = parse_calc("calc(100px / 2)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(50.0));
}

#[test]
fn test_parse_calc_invalid() {
    assert_eq!(parse_calc("calc()"), None);
    assert_eq!(parse_calc("calc("), None);
    assert_eq!(parse_calc("not-a-calc"), None);
    assert_eq!(parse_calc(""), None);
}

#[test]
fn test_eval_calc_percentage_without_parent() {
    let expr = parse_calc("calc(50% + 10px)").unwrap();
    // 百分比没有 parent_length，应返回 None
    assert_eq!(eval_calc(&expr, None), None);
}

// ── parse_calc 嵌套与优先级 ──

#[test]
/// 测试 calc() 基本嵌套：calc(calc(100% - 20px) / 2)
fn test_calc_nested_basic() {
    let expr = parse_calc("calc(calc(100% - 20px) / 2)");
    let expr = expr.expect("should parse nested calc");
    // 整体结构：外层除法，左操作数为内层减法
    match &expr {
        CalcExpr::BinaryOp(left, op, right) => {
            assert_eq!(*op, CalcOp::Divide);
            assert_eq!(**right, CalcExpr::Number(2.0));
            // 内层 calc(100% - 20px)
            match left.as_ref() {
                CalcExpr::BinaryOp(inner_left, inner_op, inner_right) => {
                    assert_eq!(**inner_left, CalcExpr::Length(LengthValue::Percentage(100.0)));
                    assert_eq!(*inner_op, CalcOp::Subtract);
                    assert_eq!(**inner_right, CalcExpr::Length(LengthValue::Px(20.0)));
                }
                _ => panic!("expected inner BinaryOp, got {left:?}"),
            }
        }
        _ => panic!("expected outer BinaryOp, got {expr:?}"),
    }

    // 求值验证：parent_length=200, 100%-20px=180, 180/2=90
    let result = eval_calc(&expr, Some(200.0));
    assert_eq!(result, Some(90.0));
}

#[test]
/// 测试 calc() 双重嵌套：calc(calc(10px + 5px) * calc(2))
fn test_calc_double_nesting() {
    let expr = parse_calc("calc(calc(10px + 5px) * calc(2))");
    let expr = expr.expect("should parse double nested calc");
    // 外层乘法
    match &expr {
        CalcExpr::BinaryOp(left, op, right) => {
            assert_eq!(*op, CalcOp::Multiply);
            // 左侧 calc(10px + 5px)
            match left.as_ref() {
                CalcExpr::BinaryOp(il, io, ir) => {
                    assert_eq!(**il, CalcExpr::Length(LengthValue::Px(10.0)));
                    assert_eq!(*io, CalcOp::Add);
                    assert_eq!(**ir, CalcExpr::Length(LengthValue::Px(5.0)));
                }
                _ => panic!("expected left inner BinaryOp, got {left:?}"),
            }
            // 右侧 calc(2)
            match right.as_ref() {
                CalcExpr::Number(n) => assert_eq!(*n, 2.0),
                _ => panic!("expected right Number, got {right:?}"),
            }
        }
        _ => panic!("expected outer BinaryOp, got {expr:?}"),
    }

    // 求值：(10+5)*2=30
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(30.0));
}

#[test]
/// 测试 calc() 混合运算（运算符优先级与从左到右）：
/// calc(100% - 10px + 5px) 应按从左到右顺序求值
fn test_calc_mixed_operations() {
    let expr = parse_calc("calc(100% - 10px + 5px)");
    let expr = expr.expect("should parse mixed operations");
    // + 和 - 同优先级，从左到右：(100% - 10px) + 5px
    match &expr {
        CalcExpr::BinaryOp(left, op, right) => {
            assert_eq!(*op, CalcOp::Add);
            assert_eq!(**right, CalcExpr::Length(LengthValue::Px(5.0)));
            match left.as_ref() {
                CalcExpr::BinaryOp(ll, lo, lr) => {
                    assert_eq!(**ll, CalcExpr::Length(LengthValue::Percentage(100.0)));
                    assert_eq!(*lo, CalcOp::Subtract);
                    assert_eq!(**lr, CalcExpr::Length(LengthValue::Px(10.0)));
                }
                _ => panic!("expected left BinaryOp, got {left:?}"),
            }
        }
        _ => panic!("expected BinaryOp, got {expr:?}"),
    }

    // 求值：parent_length=200, (200-10)+5=195
    let result = eval_calc(&expr, Some(200.0));
    assert_eq!(result, Some(195.0));
}

// ── min() / max() / clamp() 解析与求值 ──

/// 测试 min() 基本解析。
#[test]
fn test_parse_min_basic() {
    let expr = parse_min("min(100px, 50%)").unwrap();
    match &expr {
        CalcExpr::Min(args) => assert_eq!(args.len(), 2),
        _ => panic!("expected Min, got {expr:?}"),
    }
}

/// 测试 min() 多参数。
#[test]
fn test_parse_min_three_args() {
    let expr = parse_min("min(100px, 50%, 200px)").unwrap();
    match &expr {
        CalcExpr::Min(args) => assert_eq!(args.len(), 3),
        _ => panic!("expected Min, got {expr:?}"),
    }
}

/// 测试 min() 求值：取最小值。
#[test]
fn test_eval_min_basic() {
    let expr = parse_min("min(100px, 50%)").unwrap();
    // parent_length=300, 50%=150, min(100,150)=100
    let result = eval_calc(&expr, Some(300.0));
    assert_eq!(result, Some(100.0));
}

/// 测试 min() 求值：百分比更小。
#[test]
fn test_eval_min_percentage_smaller() {
    let expr = parse_min("min(200px, 25%)").unwrap();
    // parent_length=400, 25%=100, min(200,100)=100
    let result = eval_calc(&expr, Some(400.0));
    assert_eq!(result, Some(100.0));
}

/// 测试 min() 包含 calc() 嵌套。
#[test]
fn test_parse_min_with_calc() {
    let expr = parse_min("min(calc(100% - 20px), 300px)").unwrap();
    // parent_length=400, 100%-20px=380, min(380,300)=300
    let result = eval_calc(&expr, Some(400.0));
    assert_eq!(result, Some(300.0));
}

/// 测试 max() 基本解析。
#[test]
fn test_parse_max_basic() {
    let expr = parse_max("max(100px, 50%)").unwrap();
    match &expr {
        CalcExpr::Max(args) => assert_eq!(args.len(), 2),
        _ => panic!("expected Max, got {expr:?}"),
    }
}

/// 测试 max() 求值：取最大值。
#[test]
fn test_eval_max_basic() {
    let expr = parse_max("max(100px, 50%)").unwrap();
    // parent_length=300, 50%=150, max(100,150)=150
    let result = eval_calc(&expr, Some(300.0));
    assert_eq!(result, Some(150.0));
}

/// 测试 max() 三参数求值。
#[test]
fn test_eval_max_three_args() {
    let expr = parse_max("max(10px, 20px, 15px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(20.0));
}

/// 测试 clamp() 基本解析。
#[test]
fn test_parse_clamp_basic() {
    let expr = parse_clamp("clamp(100px, 50%, 300px)").unwrap();
    match &expr {
        CalcExpr::Clamp { min, val, max } => {
            assert_eq!(**min, CalcExpr::Length(LengthValue::Px(100.0)));
            assert_eq!(**val, CalcExpr::Length(LengthValue::Percentage(50.0)));
            assert_eq!(**max, CalcExpr::Length(LengthValue::Px(300.0)));
        }
        _ => panic!("expected Clamp, got {expr:?}"),
    }
}

/// 测试 clamp() 求值：val 在范围内。
#[test]
fn test_eval_clamp_in_range() {
    let expr = parse_clamp("clamp(100px, 50%, 300px)").unwrap();
    // parent_length=400, 50%=200, clamp(100,200,300)=200
    let result = eval_calc(&expr, Some(400.0));
    assert_eq!(result, Some(200.0));
}

/// 测试 clamp() 求值：val 小于 min，结果为 min。
#[test]
fn test_eval_clamp_below_min() {
    let expr = parse_clamp("clamp(100px, 10%, 300px)").unwrap();
    // parent_length=400, 10%=40, clamp(100,40,300)=100
    let result = eval_calc(&expr, Some(400.0));
    assert_eq!(result, Some(100.0));
}

/// 测试 clamp() 求值：val 大于 max，结果为 max。
#[test]
fn test_eval_clamp_above_max() {
    let expr = parse_clamp("clamp(100px, 80%, 300px)").unwrap();
    // parent_length=400, 80%=320, clamp(100,320,300)=300
    let result = eval_calc(&expr, Some(400.0));
    assert_eq!(result, Some(300.0));
}

/// 测试 parse_math_function 分发。
#[test]
fn test_parse_math_function_dispatch() {
    assert!(parse_math_function("calc(100px + 10px)").is_some());
    assert!(parse_math_function("min(100px, 50%)").is_some());
    assert!(parse_math_function("max(100px, 50%)").is_some());
    assert!(parse_math_function("clamp(100px, 50%, 300px)").is_some());
    assert!(parse_math_function("invalid(100px)").is_none());
}

/// 测试 min()/max()/clamp() 无效输入。
#[test]
fn test_parse_min_max_clamp_invalid() {
    assert_eq!(parse_min(""), None);
    assert_eq!(parse_min("min()"), None);
    assert_eq!(parse_min("min("), None);
    assert_eq!(parse_max(""), None);
    assert_eq!(parse_max("max()"), None);
    assert_eq!(parse_clamp(""), None);
    assert_eq!(parse_clamp("clamp()"), None);
    assert_eq!(parse_clamp("clamp(100px, 50%)"), None); // 缺少第三个参数
}

/// 测试 min()/max() 嵌套使用。
#[test]
fn test_parse_min_nested_max() {
    let expr = parse_min("min(max(100px, 50%), 300px)").unwrap();
    // parent_length=200, max(100,100)=100, min(100,300)=100
    let result = eval_calc(&expr, Some(200.0));
    assert_eq!(result, Some(100.0));
}

/// 测试 clamp() 内部使用 calc()。
#[test]
fn test_parse_clamp_with_calc() {
    let expr = parse_clamp("clamp(50px, calc(100% - 20px), 500px)").unwrap();
    // parent_length=400, 100%-20px=380, clamp(50,380,500)=380
    let result = eval_calc(&expr, Some(400.0));
    assert_eq!(result, Some(380.0));
}

// ── float/clear 解析测试 ──

#[test]
fn test_parse_float_values() {
    assert_eq!(parse_float("left"), Some(FloatValue::Left));
    assert_eq!(parse_float("right"), Some(FloatValue::Right));
    assert_eq!(parse_float("none"), Some(FloatValue::None));
    assert_eq!(parse_float("inline-start"), Some(FloatValue::InlineStart));
    assert_eq!(parse_float("inline-end"), Some(FloatValue::InlineEnd));
    assert_eq!(parse_float("center"), None);
    assert_eq!(parse_float(""), None);
}

#[test]
fn test_parse_clear_values() {
    assert_eq!(parse_clear("left"), Some(ClearValue::Left));
    assert_eq!(parse_clear("right"), Some(ClearValue::Right));
    assert_eq!(parse_clear("both"), Some(ClearValue::Both));
    assert_eq!(parse_clear("none"), Some(ClearValue::None));
    assert_eq!(parse_clear("inline-start"), Some(ClearValue::InlineStart));
    assert_eq!(parse_clear("inline-end"), Some(ClearValue::InlineEnd));
    assert_eq!(parse_clear("all"), None);
}

#[test]
fn test_parse_float_case_insensitive() {
    // CSS 关键字不区分大小写
    assert_eq!(parse_float("LEFT"), Some(FloatValue::Left));
    assert_eq!(parse_float(" Left "), Some(FloatValue::Left));
    assert_eq!(parse_float("None"), Some(FloatValue::None));
}

#[test]
fn test_parse_clear_whitespace() {
    assert_eq!(parse_clear("  both  "), Some(ClearValue::Both));
}

// ── list-style 解析测试 ──

#[test]
fn test_parse_list_style_type_values() {
    assert_eq!(parse_list_style_type("disc"), Some(ListStyleTypeValue::Disc));
    assert_eq!(parse_list_style_type("circle"), Some(ListStyleTypeValue::Circle));
    assert_eq!(parse_list_style_type("square"), Some(ListStyleTypeValue::Square));
    assert_eq!(parse_list_style_type("decimal"), Some(ListStyleTypeValue::Decimal));
    assert_eq!(
        parse_list_style_type("decimal-leading-zero"),
        Some(ListStyleTypeValue::DecimalLeadingZero)
    );
    assert_eq!(
        parse_list_style_type("lower-roman"),
        Some(ListStyleTypeValue::LowerRoman)
    );
    assert_eq!(
        parse_list_style_type("upper-roman"),
        Some(ListStyleTypeValue::UpperRoman)
    );
    assert_eq!(
        parse_list_style_type("lower-alpha"),
        Some(ListStyleTypeValue::LowerAlpha)
    );
    assert_eq!(
        parse_list_style_type("lower-latin"),
        Some(ListStyleTypeValue::LowerAlpha)
    );
    assert_eq!(
        parse_list_style_type("upper-alpha"),
        Some(ListStyleTypeValue::UpperAlpha)
    );
    assert_eq!(parse_list_style_type("none"), Some(ListStyleTypeValue::None));
    assert_eq!(parse_list_style_type("invalid"), None);
}

#[test]
fn test_parse_list_style_type_case_insensitive() {
    assert_eq!(parse_list_style_type("DISC"), Some(ListStyleTypeValue::Disc));
    assert_eq!(parse_list_style_type("Decimal"), Some(ListStyleTypeValue::Decimal));
}

#[test]
fn test_parse_list_style_position_values() {
    assert_eq!(
        parse_list_style_position("outside"),
        Some(ListStylePositionValue::Outside)
    );
    assert_eq!(
        parse_list_style_position("inside"),
        Some(ListStylePositionValue::Inside)
    );
    assert_eq!(parse_list_style_position("center"), None);
}

// ── parse_grid_area 测试 ──

#[test]
/// 测试 grid-area 单个命名区域
fn test_parse_grid_area_named_area() {
    let result = parse_grid_area("header").unwrap();
    assert_eq!(
        result,
        (
            "header".to_string(),
            "header".to_string(),
            "header".to_string(),
            "header".to_string()
        )
    );
}

#[test]
/// 测试 grid-area auto
fn test_parse_grid_area_auto() {
    let result = parse_grid_area("auto").unwrap();
    assert_eq!(
        result,
        (
            "auto".to_string(),
            "auto".to_string(),
            "auto".to_string(),
            "auto".to_string()
        )
    );
}

#[test]
/// 测试 grid-area 四值斜杠分隔
fn test_parse_grid_area_four_values() {
    let result = parse_grid_area("1 / 2 / 3 / 4").unwrap();
    assert_eq!(
        result,
        ("1".to_string(), "2".to_string(), "3".to_string(), "4".to_string())
    );
}

#[test]
/// 测试 grid-area 两值斜杠分隔（row-start / col-start）
fn test_parse_grid_area_two_values() {
    let result = parse_grid_area("1 / 3").unwrap();
    assert_eq!(
        result,
        ("1".to_string(), "auto".to_string(), "3".to_string(), "auto".to_string())
    );
}

#[test]
/// 测试 grid-area 三值斜杠分隔
fn test_parse_grid_area_three_values() {
    let result = parse_grid_area("1 / 2 / 3").unwrap();
    assert_eq!(
        result,
        ("1".to_string(), "2".to_string(), "3".to_string(), "auto".to_string())
    );
}

#[test]
/// 测试 grid-area 包含 span 关键字
fn test_parse_grid_area_span() {
    let result = parse_grid_area("span 2 / span 3 / span 1 / span 4").unwrap();
    assert_eq!(
        result,
        (
            "span 2".to_string(),
            "span 3".to_string(),
            "span 1".to_string(),
            "span 4".to_string()
        )
    );
}

#[test]
/// 测试 grid-area 带空白
fn test_parse_grid_area_whitespace() {
    let result = parse_grid_area("  header  ").unwrap();
    assert_eq!(
        result,
        (
            "header".to_string(),
            "header".to_string(),
            "header".to_string(),
            "header".to_string()
        )
    );

    let result = parse_grid_area("  1  /  2  /  3  /  4  ").unwrap();
    assert_eq!(
        result,
        ("1".to_string(), "2".to_string(), "3".to_string(), "4".to_string())
    );
}

#[test]
/// 测试 grid-area 无效输入
fn test_parse_grid_area_invalid() {
    assert_eq!(parse_grid_area(""), None);
    assert_eq!(parse_grid_area("   "), None);
}

// ── parse_word_break 测试 ──

#[test]
fn test_parse_word_break_normal() {
    assert_eq!(parse_word_break("normal"), Some(WordBreakValue::Normal));
}

#[test]
fn test_parse_word_break_break_all() {
    assert_eq!(parse_word_break("break-all"), Some(WordBreakValue::BreakAll));
}

#[test]
fn test_parse_word_break_keep_all() {
    assert_eq!(parse_word_break("keep-all"), Some(WordBreakValue::KeepAll));
}

#[test]
fn test_parse_word_break_invalid() {
    assert_eq!(parse_word_break("invalid"), None);
}

// ═══════════════════════════════════════════════════════════════════
// 3D Transform 函数解析测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// 测试 rotateX(45deg) 解析
fn test_parse_rotate_x() {
    let result = parse_transform("rotateX(45deg)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            assert_eq!(fns[0], TransformFunction::RotateX(45.0));
        }
        _ => panic!("expected TransformValue::List, got {result:?}"),
    }
}

#[test]
/// 测试 rotateY(-90deg) 使用 rad 单位
fn test_parse_rotate_y() {
    // -π/2 rad ≈ -90°
    let result = parse_transform("rotateY(-1.5708rad)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            let angle = match &fns[0] {
                TransformFunction::RotateY(a) => *a,
                other => panic!("expected RotateY, got {other:?}"),
            };
            assert!((angle - (-90.0)).abs() < 1.0, "angle should be near -90, got {angle}");
        }
        _ => panic!("expected TransformValue::List, got {result:?}"),
    }
}

#[test]
/// 测试 rotateZ(0.5turn) 解析（0.5 圈 = 180°）
fn test_parse_rotate_z() {
    let result = parse_transform("rotateZ(0.5turn)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            assert_eq!(fns[0], TransformFunction::RotateZ(180.0));
        }
        _ => panic!("expected TransformValue::List, got {result:?}"),
    }
}

#[test]
/// 测试 translate3d(10px, 20px, 30px) 解析
fn test_parse_translate_3d() {
    let result = parse_transform("translate3d(10px, 20px, 30px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            assert_eq!(fns[0], TransformFunction::Translate3d(10.0, 20.0, 30.0));
        }
        _ => panic!("expected TransformValue::List, got {result:?}"),
    }
}

#[test]
/// 测试 scale3d(1, 2, 3) 解析
fn test_parse_scale_3d() {
    let result = parse_transform("scale3d(1, 2, 3)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            assert_eq!(fns[0], TransformFunction::Scale3d(1.0, 2.0, 3.0));
        }
        _ => panic!("expected TransformValue::List, got {result:?}"),
    }
}

#[test]
/// 测试 rotate3d(1, 0, 0, 45deg) 解析
fn test_parse_rotate_3d() {
    let result = parse_transform("rotate3d(1, 0, 0, 45deg)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            assert_eq!(fns[0], TransformFunction::Rotate3d(1.0, 0.0, 0.0, 45.0));
        }
        _ => panic!("expected TransformValue::List, got {result:?}"),
    }
}

#[test]
/// 测试 perspective(500px) 解析
fn test_parse_perspective_func() {
    let result = parse_transform("perspective(500px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            assert_eq!(fns[0], TransformFunction::Perspective(500.0));
        }
        _ => panic!("expected TransformValue::List, got {result:?}"),
    }

    // perspective(0) 应返回 None（必须为正数）
    assert!(parse_transform("perspective(0)").is_none());
    // perspective(-100) 应返回 None（必须为正数）
    assert!(parse_transform("perspective(-100)").is_none());
}

#[test]
/// 测试 matrix(1, 0, 0, 1, 10, 20) 解析
fn test_parse_matrix() {
    let result = parse_transform("matrix(1, 0, 0, 1, 10, 20)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 1);
            assert_eq!(fns[0], TransformFunction::Matrix(1.0, 0.0, 0.0, 1.0, 10.0, 20.0));
        }
        _ => panic!("expected TransformValue::List, got {result:?}"),
    }

    // matrix 需要 6 个参数
    assert!(parse_transform("matrix(1, 0, 0)").is_none());
}

#[test]
/// 测试组合 3D 变换：translate3d(10px, 0, 0) rotateY(45deg)
fn test_parse_combined_3d_transforms() {
    let result = parse_transform("translate3d(10px, 0, 0) rotateY(45deg)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 2);
            assert_eq!(fns[0], TransformFunction::Translate3d(10.0, 0.0, 0.0));
            assert_eq!(fns[1], TransformFunction::RotateY(45.0));
        }
        _ => panic!("expected TransformValue::List, got {result:?}"),
    }
}

#[test]
/// 测试 transform: none 返回 None 变体
fn test_parse_transform_none() {
    let result = parse_transform("none").unwrap();
    assert_eq!(result, TransformValue::None);
}

#[test]
/// 测试 transform 无效输入
fn test_parse_transform_invalid() {
    assert!(parse_transform("").is_none());
    assert!(parse_transform("unknown(10px)").is_none());
}

// ═══════════════════════════════════════════════════════════════════
// Counter / Content / Quotes 解析测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_parse_counter_action_name_only() {
    let result = parse_counter_action("section").unwrap();
    assert_eq!(result.name, "section");
    assert_eq!(result.value, None);
}

#[test]
fn test_parse_counter_action_with_value() {
    let result = parse_counter_action("section 5").unwrap();
    assert_eq!(result.name, "section");
    assert_eq!(result.value, Some(5));
}

#[test]
fn test_parse_counter_action_negative() {
    let result = parse_counter_action("counter -3").unwrap();
    assert_eq!(result.name, "counter");
    assert_eq!(result.value, Some(-3));
}

#[test]
fn test_parse_counter_action_none_rejected() {
    assert_eq!(parse_counter_action("none"), None);
    assert_eq!(parse_counter_action(""), None);
}

#[test]
fn test_parse_counter_list_none() {
    let result = parse_counter_list("none").unwrap();
    assert!(result.is_empty());
}

#[test]
fn test_parse_counter_list_single() {
    let result = parse_counter_list("section").unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "section");
    assert_eq!(result[0].value, None);
}

#[test]
fn test_parse_counter_list_multiple() {
    let result = parse_counter_list("section 1 subsection").unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "section");
    assert_eq!(result[0].value, Some(1));
    assert_eq!(result[1].name, "subsection");
    assert_eq!(result[1].value, None);
}

#[test]
fn test_parse_counter_list_invalid() {
    assert_eq!(parse_counter_list(""), None);
}

#[test]
fn test_parse_content_normal() {
    assert_eq!(parse_content("normal"), Some(ContentValue::Normal));
}

#[test]
fn test_parse_content_none() {
    assert_eq!(parse_content("none"), Some(ContentValue::None));
}

#[test]
fn test_parse_content_string_double_quotes() {
    assert_eq!(
        parse_content("\"Hello\""),
        Some(ContentValue::String("Hello".to_string()))
    );
}

#[test]
fn test_parse_content_string_single_quotes() {
    assert_eq!(
        parse_content("'World'"),
        Some(ContentValue::String("World".to_string()))
    );
}

#[test]
fn test_parse_content_attr() {
    assert_eq!(
        parse_content("attr(data-label)"),
        Some(ContentValue::Attr("data-label".to_string()))
    );
}

#[test]
fn test_parse_content_counter_no_style() {
    let result = parse_content("counter(section)").unwrap();
    match result {
        ContentValue::Counter { name, style } => {
            assert_eq!(name, "section");
            assert_eq!(style, None);
        }
        _ => panic!("expected Counter variant"),
    }
}

#[test]
fn test_parse_content_counter_with_style() {
    let result = parse_content("counter(section, upper-roman)").unwrap();
    match result {
        ContentValue::Counter { name, style } => {
            assert_eq!(name, "section");
            assert_eq!(style, Some("upper-roman".to_string()));
        }
        _ => panic!("expected Counter variant"),
    }
}

#[test]
fn test_parse_content_invalid() {
    assert_eq!(parse_content("unknown-value"), None);
    assert_eq!(parse_content(""), None);
}

#[test]
fn test_parse_quotes_none() {
    assert_eq!(parse_quotes("none"), Some(QuotesValue::None));
}

#[test]
fn test_parse_quotes_auto() {
    assert_eq!(parse_quotes("auto"), Some(QuotesValue::Auto));
}

#[test]
fn test_parse_quotes_single_pair() {
    let result = parse_quotes(r#""«" "»""#).unwrap();
    match result {
        QuotesValue::Pairs(pairs) => {
            assert_eq!(pairs.len(), 1);
            assert_eq!(pairs[0], ("«".to_string(), "»".to_string()));
        }
        _ => panic!("expected Pairs"),
    }
}

#[test]
fn test_parse_quotes_two_pairs() {
    let result = parse_quotes(r#""«" "»" "‹" "›""#).unwrap();
    match result {
        QuotesValue::Pairs(pairs) => {
            assert_eq!(pairs.len(), 2);
            assert_eq!(pairs[0], ("«".to_string(), "»".to_string()));
            assert_eq!(pairs[1], ("‹".to_string(), "›".to_string()));
        }
        _ => panic!("expected Pairs"),
    }
}

#[test]
fn test_parse_quotes_single_quotes() {
    let result = parse_quotes("'\"' '\"'").unwrap();
    match result {
        QuotesValue::Pairs(pairs) => {
            assert_eq!(pairs.len(), 1);
            assert_eq!(pairs[0], ("\"".to_string(), "\"".to_string()));
        }
        _ => panic!("expected Pairs"),
    }
}

#[test]
fn test_parse_quotes_invalid() {
    assert_eq!(parse_quotes(""), None);
    assert_eq!(parse_quotes("random"), None);
}

// ── Page Break 测试 ──

#[test]
fn test_parse_page_break_auto() {
    assert_eq!(parse_page_break("auto"), Some(PageBreakValue::Auto));
}

#[test]
fn test_parse_page_break_always() {
    assert_eq!(parse_page_break("always"), Some(PageBreakValue::Always));
}

#[test]
fn test_parse_page_break_avoid() {
    assert_eq!(parse_page_break("avoid"), Some(PageBreakValue::Avoid));
}

#[test]
fn test_parse_page_break_left_right() {
    assert_eq!(parse_page_break("left"), Some(PageBreakValue::Left));
    assert_eq!(parse_page_break("right"), Some(PageBreakValue::Right));
}

#[test]
fn test_parse_page_break_invalid() {
    assert_eq!(parse_page_break("invalid"), None);
}

// ── BoxDecorationBreak 测试 ──

#[test]
fn test_parse_box_decoration_break() {
    assert_eq!(
        parse_box_decoration_break("slice"),
        Some(BoxDecorationBreakValue::Slice)
    );
    assert_eq!(
        parse_box_decoration_break("clone"),
        Some(BoxDecorationBreakValue::Clone)
    );
    assert_eq!(parse_box_decoration_break("invalid"), None);
}

// ── ImageRendering 测试 ──

#[test]
fn test_parse_image_rendering() {
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
    assert_eq!(parse_image_rendering("invalid"), None);
}

// ── Isolation 测试 ──

#[test]
fn test_parse_isolation() {
    assert_eq!(parse_isolation("auto"), Some(IsolationValue::Auto));
    assert_eq!(parse_isolation("isolate"), Some(IsolationValue::Isolate));
    assert_eq!(parse_isolation("invalid"), None);
}

// ── OverscrollBehavior 测试 ──

#[test]
fn test_parse_overscroll_behavior() {
    assert_eq!(parse_overscroll_behavior("auto"), Some(OverscrollBehaviorValue::Auto));
    assert_eq!(
        parse_overscroll_behavior("contain"),
        Some(OverscrollBehaviorValue::Contain)
    );
    assert_eq!(parse_overscroll_behavior("none"), Some(OverscrollBehaviorValue::None));
    assert_eq!(parse_overscroll_behavior("invalid"), None);
}

#[test]
fn test_parse_overscroll_behavior_case_insensitive() {
    assert_eq!(parse_overscroll_behavior("AUTO"), Some(OverscrollBehaviorValue::Auto));
    assert_eq!(
        parse_overscroll_behavior(" Contain "),
        Some(OverscrollBehaviorValue::Contain)
    );
}

// ── TouchAction 测试 ──

#[test]
fn test_parse_touch_action() {
    assert_eq!(parse_touch_action("auto"), Some(TouchActionValue::Auto));
    assert_eq!(parse_touch_action("none"), Some(TouchActionValue::None));
    assert_eq!(parse_touch_action("pan-x"), Some(TouchActionValue::PanX));
    assert_eq!(parse_touch_action("pan-y"), Some(TouchActionValue::PanY));
    assert_eq!(parse_touch_action("manipulation"), Some(TouchActionValue::Manipulation));
    assert_eq!(parse_touch_action("invalid"), None);
}

#[test]
fn test_parse_touch_action_pan_both() {
    assert_eq!(parse_touch_action("pan-x pan-y"), Some(TouchActionValue::PanXPanY));
    assert_eq!(parse_touch_action("pan-y pan-x"), Some(TouchActionValue::PanXPanY));
}

// ── UserSelect 测试 ──

#[test]
fn test_parse_user_select() {
    assert_eq!(parse_user_select("auto"), Some(UserSelectValue::Auto));
    assert_eq!(parse_user_select("text"), Some(UserSelectValue::Text));
    assert_eq!(parse_user_select("none"), Some(UserSelectValue::None));
    assert_eq!(parse_user_select("all"), Some(UserSelectValue::All));
    assert_eq!(parse_user_select("contain"), Some(UserSelectValue::Contain));
    assert_eq!(parse_user_select("invalid"), None);
}

// ── WillChange 测试 ──

#[test]
fn test_parse_will_change() {
    assert_eq!(parse_will_change("auto"), Some(WillChangeValue::Auto));
    assert_eq!(
        parse_will_change("scroll-position"),
        Some(WillChangeValue::ScrollPosition)
    );
    assert_eq!(parse_will_change("contents"), Some(WillChangeValue::Contents));
    assert_eq!(
        parse_will_change("transform"),
        Some(WillChangeValue::Custom("transform".to_string()))
    );
    assert_eq!(
        parse_will_change("opacity"),
        Some(WillChangeValue::Custom("opacity".to_string()))
    );
    assert_eq!(parse_will_change(""), None);
}

// ── PointerEvents 测试 ──

#[test]
fn test_parse_pointer_events() {
    assert_eq!(parse_pointer_events("auto"), Some(PointerEventsValue::Auto));
    assert_eq!(parse_pointer_events("none"), Some(PointerEventsValue::None));
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
    assert_eq!(parse_pointer_events("invalid"), None);
}

#[test]
fn test_parse_pointer_events_case_insensitive() {
    assert_eq!(parse_pointer_events("NONE"), Some(PointerEventsValue::None));
    assert_eq!(
        parse_pointer_events(" VisiblePainted "),
        Some(PointerEventsValue::VisiblePainted)
    );
}

// ── OverflowWrap 测试 ──

#[test]
fn test_parse_overflow_wrap_normal() {
    assert_eq!(parse_overflow_wrap("normal"), Some(OverflowWrapValue::Normal));
}

#[test]
fn test_parse_overflow_wrap_break_word() {
    assert_eq!(parse_overflow_wrap("break-word"), Some(OverflowWrapValue::BreakWord));
}

#[test]
fn test_parse_overflow_wrap_anywhere() {
    assert_eq!(parse_overflow_wrap("anywhere"), Some(OverflowWrapValue::Anywhere));
}

#[test]
fn test_parse_overflow_wrap_invalid() {
    assert_eq!(parse_overflow_wrap("invalid"), None);
}

#[test]
fn test_parse_overflow_wrap_case_insensitive() {
    assert_eq!(parse_overflow_wrap("BREAK-WORD"), Some(OverflowWrapValue::BreakWord));
    assert_eq!(parse_overflow_wrap(" Anywhere "), Some(OverflowWrapValue::Anywhere));
}

// ── TextAlignLast 测试 ──

#[test]
fn test_parse_text_align_last_auto() {
    assert_eq!(parse_text_align_last("auto"), Some(TextAlignLastValue::Auto));
}

#[test]
fn test_parse_text_align_last_start_end() {
    assert_eq!(parse_text_align_last("start"), Some(TextAlignLastValue::Start));
    assert_eq!(parse_text_align_last("end"), Some(TextAlignLastValue::End));
}

#[test]
fn test_parse_text_align_last_left_right_center() {
    assert_eq!(parse_text_align_last("left"), Some(TextAlignLastValue::Left));
    assert_eq!(parse_text_align_last("right"), Some(TextAlignLastValue::Right));
    assert_eq!(parse_text_align_last("center"), Some(TextAlignLastValue::Center));
}

#[test]
fn test_parse_text_align_last_justify() {
    assert_eq!(parse_text_align_last("justify"), Some(TextAlignLastValue::Justify));
}

#[test]
fn test_parse_text_align_last_invalid() {
    assert_eq!(parse_text_align_last("invalid"), None);
}

#[test]
fn test_parse_text_align_last_case_insensitive() {
    assert_eq!(parse_text_align_last("JUSTIFY"), Some(TextAlignLastValue::Justify));
    assert_eq!(parse_text_align_last(" Center "), Some(TextAlignLastValue::Center));
}

// ── FontVariantNumeric 测试 ──

#[test]
fn test_parse_font_variant_numeric_normal() {
    assert_eq!(
        parse_font_variant_numeric("normal"),
        Some(FontVariantNumericValue::Normal)
    );
}

#[test]
fn test_parse_font_variant_numeric_ordinal() {
    assert_eq!(
        parse_font_variant_numeric("ordinal"),
        Some(FontVariantNumericValue::Ordinal)
    );
}

#[test]
fn test_parse_font_variant_numeric_slashed_zero() {
    assert_eq!(
        parse_font_variant_numeric("slashed-zero"),
        Some(FontVariantNumericValue::SlashedZero)
    );
}

#[test]
fn test_parse_font_variant_numeric_num_styles() {
    assert_eq!(
        parse_font_variant_numeric("lining-nums"),
        Some(FontVariantNumericValue::LiningNums)
    );
    assert_eq!(
        parse_font_variant_numeric("oldstyle-nums"),
        Some(FontVariantNumericValue::OldstyleNums)
    );
    assert_eq!(
        parse_font_variant_numeric("proportional-nums"),
        Some(FontVariantNumericValue::ProportionalNums)
    );
    assert_eq!(
        parse_font_variant_numeric("tabular-nums"),
        Some(FontVariantNumericValue::TabularNums)
    );
}

#[test]
fn test_parse_font_variant_numeric_fractions() {
    assert_eq!(
        parse_font_variant_numeric("diagonal-fractions"),
        Some(FontVariantNumericValue::DiagonalFractions)
    );
    assert_eq!(
        parse_font_variant_numeric("stacked-fractions"),
        Some(FontVariantNumericValue::StackedFractions)
    );
}

#[test]
fn test_parse_font_variant_numeric_invalid() {
    assert_eq!(parse_font_variant_numeric("invalid"), None);
}

#[test]
fn test_parse_font_variant_numeric_case_insensitive() {
    assert_eq!(
        parse_font_variant_numeric("ORDINAL"),
        Some(FontVariantNumericValue::Ordinal)
    );
    assert_eq!(
        parse_font_variant_numeric(" Lining-Nums "),
        Some(FontVariantNumericValue::LiningNums)
    );
}

// ── Direction 测试 ──

#[test]
fn test_parse_direction_ltr() {
    assert_eq!(parse_direction("ltr"), Some(DirectionValue::Ltr));
}

#[test]
fn test_parse_direction_rtl() {
    assert_eq!(parse_direction("rtl"), Some(DirectionValue::Rtl));
}

#[test]
fn test_parse_direction_case_insensitive() {
    assert_eq!(parse_direction("LTR"), Some(DirectionValue::Ltr));
    assert_eq!(parse_direction("Rtl"), Some(DirectionValue::Rtl));
    assert_eq!(parse_direction("  ltr  "), Some(DirectionValue::Ltr));
}

#[test]
fn test_parse_direction_invalid() {
    assert_eq!(parse_direction("invalid"), None);
    assert_eq!(parse_direction(""), None);
}

// ── UnicodeBidi 测试 ──

#[test]
fn test_parse_unicode_bidi_normal() {
    assert_eq!(parse_unicode_bidi("normal"), Some(UnicodeBidiValue::Normal));
}

#[test]
fn test_parse_unicode_bidi_all_values() {
    assert_eq!(parse_unicode_bidi("embed"), Some(UnicodeBidiValue::Embed));
    assert_eq!(parse_unicode_bidi("isolate"), Some(UnicodeBidiValue::Isolate));
    assert_eq!(
        parse_unicode_bidi("bidi-override"),
        Some(UnicodeBidiValue::BidiOverride)
    );
    assert_eq!(
        parse_unicode_bidi("isolate-override"),
        Some(UnicodeBidiValue::IsolateOverride)
    );
    assert_eq!(parse_unicode_bidi("plaintext"), Some(UnicodeBidiValue::Plaintext));
}

#[test]
fn test_parse_unicode_bidi_case_insensitive() {
    assert_eq!(parse_unicode_bidi("NORMAL"), Some(UnicodeBidiValue::Normal));
    assert_eq!(parse_unicode_bidi("  Embed  "), Some(UnicodeBidiValue::Embed));
}

#[test]
fn test_parse_unicode_bidi_invalid() {
    assert_eq!(parse_unicode_bidi("invalid"), None);
    assert_eq!(parse_unicode_bidi(""), None);
}

// ── TabSize 测试 ──

#[test]
fn test_parse_tab_size_number() {
    assert_eq!(parse_tab_size("4"), Some(TabSizeValue::Number(4)));
    assert_eq!(parse_tab_size("8"), Some(TabSizeValue::Number(8)));
    assert_eq!(parse_tab_size("0"), Some(TabSizeValue::Number(0)));
}

#[test]
fn test_parse_tab_size_length() {
    assert_eq!(
        parse_tab_size("20px"),
        Some(TabSizeValue::Length(LengthValue::Px(20.0)))
    );
    assert_eq!(parse_tab_size("1em"), Some(TabSizeValue::Length(LengthValue::Em(1.0))));
}

#[test]
fn test_parse_tab_size_case_insensitive() {
    assert_eq!(parse_tab_size("  4  "), Some(TabSizeValue::Number(4)));
}

// ── parse_length_quirks ──────────────────────────────────────────────

#[test]
fn test_parse_length_quirks_standard_still_works() {
    // 标准格式在 quirks mode 下仍然正常解析
    assert_eq!(parse_length_quirks("10px"), Some(LengthValue::Px(10.0)));
    assert_eq!(parse_length_quirks("1.5em"), Some(LengthValue::Em(1.5)));
    assert_eq!(parse_length_quirks("auto"), Some(LengthValue::Auto));
    assert_eq!(parse_length_quirks("0"), Some(LengthValue::Px(0.0)));
}

#[test]
fn test_parse_length_ex_unit_to_em_ahem_xheight() {
    // CSS ex 单位 = 字体 x-height。ZeroWeb 未接入字体度量管线，按 Ahem（WPT 测试字体）
    // 实测 x-height 0.8em 取值（CSS2 values/units-004 实证：12.5ex@20px=200px=10em）。
    // ex 相对元素自身 font-size（同 em），故解析为 Em(num*0.8) 复用 em 路径。
    assert_eq!(parse_length("1ex"), Some(LengthValue::Em(0.8)));
    assert_eq!(parse_length("6ex"), Some(LengthValue::Em(6.0 * 0.8)));
    assert_eq!(parse_length("0ex"), Some(LengthValue::Em(0.0)));
}

#[test]
fn test_parse_length_quirks_unitless_number() {
    // 裸数字在 quirks mode 下视为 px
    assert_eq!(parse_length_quirks("100"), Some(LengthValue::Px(100.0)));
    assert_eq!(parse_length_quirks("50"), Some(LengthValue::Px(50.0)));
    assert_eq!(parse_length_quirks("0"), Some(LengthValue::Px(0.0)));
}

#[test]
fn test_parse_length_quirks_invalid_still_none() {
    assert_eq!(parse_length_quirks("abc"), None);
    assert_eq!(parse_length_quirks(""), None);
    assert_eq!(parse_length_quirks("10abc"), None);
}

#[test]
/// R2267：color-mix(in srgb, ...) 解析为 ColorValue::Mix（保留 currentColor 未解析）。
fn test_parse_color_mix_srgb() {
    use crate::values::ColorValue;
    // 双省略百分比 → 各 None（解析层不归一化，paint 时按 spec 默认 50/50）
    let c = super::super::parse_color("color-mix(in srgb, currentColor 50%, green)").unwrap();
    match c {
        ColorValue::Mix(spec) => {
            assert!(matches!(spec.c1.color, ColorValue::CurrentColor));
            assert_eq!(spec.c1.percentage, Some(50.0));
            // green 在 parse_color 阶段已解析为 Rgba(0,128,0)
            assert!(matches!(spec.c2.color, ColorValue::Rgba(0, 128, 0, 255)));
            assert_eq!(spec.c2.percentage, None); // 第二分量省略 → None（100-50 在 paint 算）
        }
        other => panic!("应为 Mix，实际: {:?}", other),
    }
    // 非 srgb 色彩空间 → None（defer）
    assert!(super::super::parse_color("color-mix(in lch, red, blue)").is_none());
}

#[test]
/// R2268：RCS 相对色 identity 快捷——`<func>(from currentColor <自然关键字>)` → CurrentColor。
/// driving: css-color relative-currentcolor-*（14 identity 案）。
fn test_parse_relative_color_identity() {
    use crate::values::ColorValue;
    // identity：channels = 函数自然关键字 → origin（currentColor）
    for identity in [
        "rgb(from currentColor r g b)",
        "hsl(from currentColor h s l)",
        "hwb(from currentColor h w b)",
        "lab(from currentColor l a b)",
        "lch(from currentColor l c h)",
        "oklab(from currentColor l a b)",
        "oklch(from currentColor l c h)",
        "color(from currentColor a98-rgb r g b)",
        "color(from currentColor display-p3 r g b)",
        "color(from currentColor prophoto-rgb r g b)",
        "color(from currentColor rec2020 r g b)",
        "color(from currentColor xyz-d50 x y z)",
        "color(from currentColor xyz-d65 x y z)",
    ] {
        assert!(
            matches!(super::super::parse_color(identity), Some(ColorValue::CurrentColor)),
            "identity {identity:?} 应解析为 CurrentColor"
        );
    }
    // 非 identity（channel 覆盖/swap）现由 parse_relative_color 处理（R2271，见
    // test_parse_relative_color_non_identity）；不再返回 None。
    // 非 currentColor origin + identity → origin 颜色（currentColor 经 parse_color）
    // 常规 rgb（无 from）不受影响
    assert!(matches!(
        super::super::parse_color("rgb(0, 128, 0)"),
        Some(ColorValue::Rgba(0, 128, 0, 255))
    ));
}

#[test]
/// R2271：RCS 非 identity（channel 置换/覆盖）解析为 ColorValue::RelativeColor（保留 currentColor
/// origin 未解析，paint 时按元素色解析）。driving: relative-currentcolor-rgb-02（g r b 置换）/ hsl-02（120 s l）。
fn test_parse_relative_color_non_identity() {
    use crate::values::{ColorValue, RcsAlpha, RcsChannel, RelativeColorFunc};
    // rgb 置换：`rgb(from currentColor g r b)` —— 首通道引用 origin green(=1)，次引用 red(=0)。
    let c = super::super::parse_color("rgb(from currentColor g r b)").unwrap();
    match c {
        ColorValue::RelativeColor(spec) => {
            assert_eq!(spec.func, RelativeColorFunc::Rgb);
            assert!(matches!(spec.origin, ColorValue::CurrentColor));
            assert!(matches!(spec.channels[0], RcsChannel::Ref(1)), "g → Ref(1)");
            assert!(matches!(spec.channels[1], RcsChannel::Ref(0)), "r → Ref(0)");
            assert!(matches!(spec.channels[2], RcsChannel::Ref(2)), "b → Ref(2)");
            assert_eq!(spec.alpha, RcsAlpha::Origin);
        }
        other => panic!("rgb 置换应为 RelativeColor，实际: {:?}", other),
    }
    // hsl 覆盖：`hsl(from currentColor 120 s l)` —— h=字面量 120，s/l 引用 origin。
    let c = super::super::parse_color("hsl(from currentColor 120 s l)").unwrap();
    match c {
        ColorValue::RelativeColor(spec) => {
            assert_eq!(spec.func, RelativeColorFunc::Hsl);
            assert!(matches!(spec.origin, ColorValue::CurrentColor));
            assert!(matches!(spec.channels[0], RcsChannel::Num(120.0)), "h → Num(120)");
            assert!(matches!(spec.channels[1], RcsChannel::Ref(1)), "s → Ref(1)");
            assert!(matches!(spec.channels[2], RcsChannel::Ref(2)), "l → Ref(2)");
        }
        other => panic!("hsl 覆盖应为 RelativeColor，实际: {:?}", other),
    }
    // 非 rgb/hsl 函数（lab/lch/oklab/oklch/color()）非 identity 仍 defer → None。
    assert!(super::super::parse_color("lab(from currentColor l a b / 0.5)").is_none());
    // 非 from 形式不受影响（常规 rgb）。
    assert!(matches!(
        super::super::parse_color("rgb(1 2 3)"),
        Some(ColorValue::Rgba(1, 2, 3, 255))
    ));
}

#[test]
/// R2269：lab()/lch()/oklab()/oklch() → sRGB（green #008000 各空间值经 WPT 注释验证）。
fn test_parse_lab_lch_oklab_oklch() {
    use crate::values::ColorValue;
    // green = (0,128,0)。WPT 注释提供各空间转换值。
    let g = |c: &str| super::super::parse_color(c);
    assert!(matches!(
        g("lab(46.2775% -47.5621 48.5837)"),
        Some(ColorValue::Rgba(0, 128, 0, 255))
    ));
    assert!(matches!(
        g("lch(46.2775% 67.9892 134.3912)"),
        Some(ColorValue::Rgba(0, 128, 0, 255))
    ));
    assert!(matches!(
        g("oklab(51.975% -0.1403 0.10768)"),
        Some(ColorValue::Rgba(0, 128, 0, 255))
    ));
    assert!(matches!(
        g("oklch(51.975% 0.17686 142.495)"),
        Some(ColorValue::Rgba(0, 128, 0, 255))
    ));
    // L>100/L>1 钳制：lab(150 150 20) == lab(100 150 20)（driving: lab-l-over-100-*）。
    assert_eq!(g("lab(150 150 20)"), g("lab(100 150 20)"), "L>100 应钳制到 100");
    assert_eq!(g("oklch(150% 0.17686 142.495)"), g("oklch(100% 0.17686 142.495)"));
    // 非 3 分量 → None
    assert!(g("lab(50% 40)").is_none());
}

#[test]
/// R2272：rebeccapurple（CSS Color 4 新增命名颜色）解析。driving: css-color named-001。
fn test_parse_rebeccapurple() {
    use crate::values::ColorValue;
    assert!(matches!(
        super::super::parse_color("rebeccapurple"),
        Some(ColorValue::Rgba(102, 51, 153, 255))
    ));
    // 大小写不敏感
    assert!(matches!(
        super::super::parse_color("RebeccaPurple"),
        Some(ColorValue::Rgba(102, 51, 153, 255))
    ));
}

#[test]
/// R2270：CSS Color 4 线性光变体（display-p3-linear 等）——跳过 gamma decode。
fn test_parse_linear_color_spaces() {
    use crate::values::ColorValue;
    // display-p3-linear green（WPT 线性 display-p3 值）→ (0,128,0)
    assert!(matches!(
        super::super::parse_color("color(display-p3-linear 0.0383 0.2087 0.0156)"),
        Some(ColorValue::Rgba(0, 128, 0, 255))
    ));
    // 常规 display-p3（gamma）仍工作（green 在 display-p3 略不同，但 a98-rgb green 已 R2255 验证）
    assert!(super::super::parse_color("color(display-p3 0 0.5 0)").is_some());
}
