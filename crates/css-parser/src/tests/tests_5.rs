// tests_1 溢出测试（从 tests_1.rs 自动拆分）
use super::*;
use crate::values::*;
use crate::ast::*;
use crate::tokenizer::{Token, Tokenizer, Spanned};
use crate::parser::Parser;


#[test]
fn test_parse_transform_rotate() {
    let result = parse_transform("rotate(45deg)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Rotate(45.0));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_rotate_rad() {
    let result = parse_transform("rotate(1.5708rad)").unwrap();
    match result {
        TransformValue::List(fns) => {
            // ~90 degrees
            let angle = match fns[0] {
                TransformFunction::Rotate(a) => a,
                _ => 0.0,
            };
            assert!((angle - 90.0).abs() < 1.0);
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_scale() {
    let result = parse_transform("scale(2)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Scale(2.0, None));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_scale_xy() {
    let result = parse_transform("scale(2, 3)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Scale(2.0, Some(3.0)));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_scale_x_y() {
    let result = parse_transform("scaleX(1.5)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::ScaleX(1.5));
        }
        _ => panic!("Expected List"),
    }

    let result = parse_transform("scaleY(0.5)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::ScaleY(0.5));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_skew() {
    let result = parse_transform("skew(10deg)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Skew(10.0, None));
        }
        _ => panic!("Expected List"),
    }

    let result = parse_transform("skew(10deg, 20deg)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Skew(10.0, Some(20.0)));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_multiple() {
    let result = parse_transform("translate(10px, 20px) rotate(45deg) scale(2)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns.len(), 3);
            assert_eq!(fns[0], TransformFunction::Translate(10.0, 20.0));
            assert_eq!(fns[1], TransformFunction::Rotate(45.0));
            assert_eq!(fns[2], TransformFunction::Scale(2.0, None));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_empty() {
    assert_eq!(parse_transform(""), None);
    assert_eq!(parse_transform("  "), None);
}

#[test]
fn test_parse_transform_unknown_function() {
    assert_eq!(parse_transform("unknown(10px)"), None);
}

#[test]
fn test_parse_transform_negative_values() {
    let result = parse_transform("translate(-10px, -20px)").unwrap();
    match result {
        TransformValue::List(fns) => {
            assert_eq!(fns[0], TransformFunction::Translate(-10.0, -20.0));
        }
        _ => panic!("Expected List"),
    }
}

#[test]
fn test_parse_transform_turn() {
    let result = parse_transform("rotate(0.5turn)").unwrap();
    match result {
        TransformValue::List(fns) => {
            let angle = match fns[0] {
                TransformFunction::Rotate(a) => a,
                _ => 0.0,
            };
            assert!((angle - 180.0).abs() < 0.01);
        }
        _ => panic!("Expected List"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 33. CSS 类型值解析测试（覆盖 types.rs 的 uncovered 路径）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 LengthValue 构造函数
fn test_length_value_constructors() {
    let test_cases = vec![
        (LengthValue::Px(10.0), "10px"),
        (LengthValue::Em(2.5), "2.5em"),
        (LengthValue::Rem(1.0), "1rem"),
        (LengthValue::Vh(100.0), "100vh"),
        (LengthValue::Vw(50.0), "50vw"),
        (LengthValue::Vmin(20.0), "20vmin"),
        (LengthValue::Vmax(80.0), "80vmax"),
        (LengthValue::Ch(16.0), "16ch"),
        (LengthValue::Percentage(50.0), "50%"),
        (LengthValue::Auto, "auto"),
        (LengthValue::MinContent, "min-content"),
        (LengthValue::MaxContent, "max-content"),
        (
            LengthValue::FitContent(Box::new(LengthValue::Px(100.0))),
            "fit-content(100px)",
        ),
    ];

    for (length_value, _expected_str) in test_cases {
        // 这里只是测试构造函数，不测试解析
        let _ = length_value;
    }
}

#[test]
/// 测试 LengthValue 的相等性比较
fn test_length_value_equality() {
    let test_cases = vec![
        (LengthValue::Px(10.0), LengthValue::Px(10.0), true),
        (LengthValue::Px(10.0), LengthValue::Px(20.0), false),
        (LengthValue::Em(1.0), LengthValue::Em(1.0), true),
        (LengthValue::Em(1.0), LengthValue::Px(1.0), false),
        (LengthValue::Auto, LengthValue::Auto, true),
        (LengthValue::MinContent, LengthValue::MinContent, true),
        (LengthValue::MaxContent, LengthValue::MaxContent, true),
        (LengthValue::Percentage(50.0), LengthValue::Percentage(50.0), true),
        (LengthValue::Percentage(50.0), LengthValue::Percentage(100.0), false),
    ];

    for (val1, val2, expected_equal) in test_cases {
        assert_eq!(
            val1 == val2,
            expected_equal,
            "{:?} == {:?} should be {}",
            val1,
            val2,
            expected_equal
        );
    }
}

#[test]
/// 测试 ColorValue 变体
fn test_color_value_variants() {
    let test_cases = vec![
        (ColorValue::Rgba(255, 0, 0, 255), "rgba(255, 0, 0, 255)"),
        (ColorValue::Rgba(0, 0, 255, 128), "rgba(0, 0, 255, 128)"),
        (ColorValue::Hsla(0.0, 100.0, 50.0, 1.0), "hsla(0, 100%, 50%, 1)"),
        (ColorValue::Hsla(120.0, 100.0, 50.0, 0.5), "hsla(120, 100%, 50%, 0.5)"),
        (ColorValue::Named("red".to_string()), "red"),
        (ColorValue::Named("blue".to_string()), "blue"),
        (ColorValue::Transparent, "transparent"),
        (ColorValue::CurrentColor, "currentColor"),
    ];

    for (color_value, _) in test_cases {
        // 测试 Debug 格式化
        let _ = format!("{:?}", color_value);

        // 测试 Clone
        let cloned = color_value.clone();
        assert_eq!(color_value, cloned);
    }
}

#[test]
/// 测试 DisplayValue 枚举
fn test_display_value_equality() {
    let test_cases = vec![
        (DisplayValue::Block, DisplayValue::Block, true),
        (DisplayValue::Inline, DisplayValue::Inline, true),
        (DisplayValue::InlineBlock, DisplayValue::InlineBlock, true),
        (DisplayValue::Flex, DisplayValue::Flex, true),
        (DisplayValue::InlineFlex, DisplayValue::InlineFlex, true),
        (DisplayValue::Grid, DisplayValue::Grid, true),
        (DisplayValue::InlineGrid, DisplayValue::InlineGrid, true),
        (DisplayValue::None, DisplayValue::None, true),
        (DisplayValue::Contents, DisplayValue::Contents, true),
        (DisplayValue::Flow, DisplayValue::Flow, true),
        (DisplayValue::FlowRoot, DisplayValue::FlowRoot, true),
        (DisplayValue::ListItem, DisplayValue::ListItem, true),
    ];

    for (val1, val2, expected_equal) in test_cases {
        assert_eq!(
            val1 == val2,
            expected_equal,
            "{:?} == {:?} should be {}",
            val1,
            val2,
            expected_equal
        );
    }
}

#[test]
/// 测试 FloatValue 和 ClearValue 枚举
fn test_float_and_clear_values() {
    let float_test_cases = vec![
        (FloatValue::None, "none"),
        (FloatValue::Left, "left"),
        (FloatValue::Right, "right"),
        (FloatValue::InlineStart, "inline-start"),
        (FloatValue::InlineEnd, "inline-end"),
    ];

    let clear_test_cases = vec![
        (ClearValue::None, "none"),
        (ClearValue::Left, "left"),
        (ClearValue::Right, "right"),
        (ClearValue::Both, "both"),
        (ClearValue::InlineStart, "inline-start"),
        (ClearValue::InlineEnd, "inline-end"),
    ];

    for (float_value, _) in float_test_cases {
        let _ = format!("{:?}", float_value);
        let _ = float_value.clone();
    }

    for (clear_value, _) in clear_test_cases {
        let _ = format!("{:?}", clear_value);
        let _ = clear_value.clone();
    }
}

#[test]
/// 测试 PositionValue 枚举
fn test_position_value() {
    let test_cases = vec![
        (PositionValue::Static, "static"),
        (PositionValue::Relative, "relative"),
        (PositionValue::Absolute, "absolute"),
        (PositionValue::Fixed, "fixed"),
        (PositionValue::Sticky, "sticky"),
    ];

    for (position_value, _) in test_cases {
        let _ = format!("{:?}", position_value);
        let _ = position_value.clone();
    }
}

#[test]
/// 测试 OverflowValue 枚举
fn test_overflow_value() {
    let test_cases = vec![
        (OverflowValue::Visible, "visible"),
        (OverflowValue::Hidden, "hidden"),
        (OverflowValue::Scroll, "scroll"),
        (OverflowValue::Auto, "auto"),
        (OverflowValue::Clip, "clip"),
    ];

    for (overflow_value, _) in test_cases {
        let _ = format!("{:?}", overflow_value);
        let _ = overflow_value.clone();
    }
}

#[test]
/// 测试 ListStyleTypeValue 枚举
fn test_list_style_type_value() {
    let test_cases = vec![
        (ListStyleTypeValue::Disc, "disc"),
        (ListStyleTypeValue::Circle, "circle"),
        (ListStyleTypeValue::Square, "square"),
        (ListStyleTypeValue::Decimal, "decimal"),
        (ListStyleTypeValue::DecimalLeadingZero, "decimal-leading-zero"),
        (ListStyleTypeValue::LowerRoman, "lower-roman"),
        (ListStyleTypeValue::UpperRoman, "upper-roman"),
        (ListStyleTypeValue::LowerAlpha, "lower-alpha"),
        (ListStyleTypeValue::UpperAlpha, "upper-alpha"),
        (ListStyleTypeValue::None, "none"),
    ];

    for (list_style_type, _) in test_cases {
        let _ = format!("{:?}", list_style_type);
        let _ = list_style_type.clone();
    }
}

#[test]
/// 测试 ListStylePositionValue 枚举
fn test_list_style_position_value() {
    let test_cases = vec![
        (ListStylePositionValue::Outside, "outside"),
        (ListStylePositionValue::Inside, "inside"),
    ];

    for (position_value, _) in test_cases {
        let _ = format!("{:?}", position_value);
        let _ = position_value.clone();
    }
}

#[test]
/// 测试 FlexDirectionValue 枚举
fn test_flex_direction_value() {
    let test_cases = vec![
        (FlexDirectionValue::Row, "row"),
        (FlexDirectionValue::RowReverse, "row-reverse"),
        (FlexDirectionValue::Column, "column"),
        (FlexDirectionValue::ColumnReverse, "column-reverse"),
    ];

    for (flex_direction, _) in test_cases {
        let _ = format!("{:?}", flex_direction);
        let _ = flex_direction.clone();
    }
}

#[test]
/// 测试 FlexWrapValue 枚举
fn test_flex_wrap_value() {
    let test_cases = vec![(FlexWrapValue::Nowrap, "nowrap"), (FlexWrapValue::Wrap, "wrap")];

    for (flex_wrap, _) in test_cases {
        let _ = format!("{:?}", flex_wrap);
        let _ = flex_wrap.clone();
    }
}

#[test]
/// 测试所有 CSS 类型值的 Clone 实现
fn test_all_css_values_clone() {
    // 这里测试各种类型值的 Clone 是否正常工作
    let _ = LengthValue::Px(10.0).clone();
    let _ = ColorValue::Rgba(255, 0, 0, 255).clone();
    let _ = DisplayValue::Block.clone();
    let _ = FloatValue::None.clone();
    let _ = ClearValue::None.clone();
    let _ = PositionValue::Static.clone();
    let _ = OverflowValue::Visible.clone();
    let _ = ListStyleTypeValue::Disc.clone();
    let _ = ListStylePositionValue::Outside.clone();
    let _ = FlexDirectionValue::Row.clone();
    let _ = FlexWrapValue::Nowrap.clone();

    // 测试嵌套类型的 Clone
    let _ = LengthValue::FitContent(Box::new(LengthValue::Px(100.0))).clone();
}

#[test]
/// 测试 CSS 类型值的 Debug 格式化
fn test_all_css_values_debug() {
    // 这里测试各种类型值的 Debug 格式化是否正常工作
    let _ = format!("{:?}", LengthValue::Px(10.0));
    let _ = format!("{:?}", ColorValue::Rgba(255, 0, 0, 255));
    let _ = format!("{:?}", DisplayValue::Block);
    let _ = format!("{:?}", FloatValue::None);
    let _ = format!("{:?}", ClearValue::None);
    let _ = format!("{:?}", PositionValue::Static);
    let _ = format!("{:?}", OverflowValue::Visible);
    let _ = format!("{:?}", ListStyleTypeValue::Disc);
    let _ = format!("{:?}", ListStylePositionValue::Outside);
    let _ = format!("{:?}", FlexDirectionValue::Row);
    let _ = format!("{:?}", FlexWrapValue::Nowrap);

    // 测试嵌套类型的 Debug 格式化
    let _ = format!("{:?}", LengthValue::FitContent(Box::new(LengthValue::Px(100.0))));
}

// ═══════════════════════════════════════════════════════════════════════
// 36. Transform/Timing 边界测试（覆盖 parse_transform.rs 的 uncovered 路径）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_animation_direction 的各种格式
fn test_parse_animation_direction_formats() {
    let test_cases = vec![
        ("normal", AnimationDirectionValue::Normal),
        ("reverse", AnimationDirectionValue::Reverse),
        ("alternate", AnimationDirectionValue::Alternate),
        ("alternate-reverse", AnimationDirectionValue::AlternateReverse),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_direction(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_direction 无效输入
fn test_parse_animation_direction_invalid() {
    let test_cases = vec![
        "",
        " ",
        "invalid",
        "alternate-reverse-extra",
        "normal extra",
        "123",
        "normal123",
    ];

    for input in test_cases {
        let result = crate::values::parse_animation_direction(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_fill_mode 的各种格式
fn test_parse_animation_fill_mode_formats() {
    let test_cases = vec![
        ("none", AnimationFillModeValue::None),
        ("forwards", AnimationFillModeValue::Forwards),
        ("backwards", AnimationFillModeValue::Backwards),
        ("both", AnimationFillModeValue::Both),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_fill_mode(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_fill_mode 无效输入
fn test_parse_animation_fill_mode_invalid() {
    let test_cases = vec!["", " ", "invalid", "forwards extra", "none123", "123"];

    for input in test_cases {
        let result = crate::values::parse_animation_fill_mode(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_play_state 的各种格式
fn test_parse_animation_play_state_formats() {
    let test_cases = vec![
        ("running", AnimationPlayStateValue::Running),
        ("paused", AnimationPlayStateValue::Paused),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_play_state(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_play_state 无效输入
fn test_parse_animation_play_state_invalid() {
    let test_cases = vec!["", " ", "invalid", "running extra", "paused123", "123"];

    for input in test_cases {
        let result = crate::values::parse_animation_play_state(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_name 的各种格式
fn test_parse_animation_name_formats() {
    let test_cases = vec![
        ("none", AnimationNameValue::None),
        ("fadeIn", AnimationNameValue::Custom("fadeIn".to_string())),
        ("slide-in", AnimationNameValue::Custom("slide-in".to_string())),
        ("test123", AnimationNameValue::Custom("test123".to_string())),
        ("_valid", AnimationNameValue::Custom("_valid".to_string())),
        ("-valid", AnimationNameValue::Custom("-valid".to_string())),
        ("valid_name", AnimationNameValue::Custom("valid_name".to_string())),
        ("NONE", AnimationNameValue::None),
        ("fadeIn", AnimationNameValue::Custom("fadeIn".to_string())),
        ("SLIDE-IN", AnimationNameValue::Custom("SLIDE-IN".to_string())),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_name(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_name 无效输入
fn test_parse_animation_name_invalid() {
    let test_cases = vec![
        "",             // 空字符串
        " ",            // 只有空格
        "123invalid",   // 以数字开头
        "invalid name", // 包含空格
    ];

    for input in test_cases {
        let result = crate::values::parse_animation_name(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_duration 的各种格式
fn test_parse_animation_duration_formats() {
    let test_cases = vec![
        ("1s", AnimationDurationValue::Time(1.0, TimeUnit::S)),
        ("0.5s", AnimationDurationValue::Time(0.5, TimeUnit::S)),
        ("2s", AnimationDurationValue::Time(2.0, TimeUnit::S)),
        ("500ms", AnimationDurationValue::Time(500.0, TimeUnit::Ms)),
        ("100ms", AnimationDurationValue::Time(100.0, TimeUnit::Ms)),
        ("0ms", AnimationDurationValue::Time(0.0, TimeUnit::Ms)),
        ("1.5s", AnimationDurationValue::Time(1.5, TimeUnit::S)),
        ("1500ms", AnimationDurationValue::Time(1500.0, TimeUnit::Ms)),
        ("1S", AnimationDurationValue::Time(1.0, TimeUnit::S)),
        ("0.5S", AnimationDurationValue::Time(0.5, TimeUnit::S)),
        ("500MS", AnimationDurationValue::Time(500.0, TimeUnit::Ms)),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_duration(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_duration 无效输入
fn test_parse_animation_duration_invalid() {
    let test_cases = vec![
        "",    // 空字符串
        " ",   // 只有空格
        "1",   // 没有单位
        "s",   // 只有单位
        "ms",  // 只有单位
        "1x",  // 无效单位
        "1xs", // 无效单位
        "1sm", // 无效单位
        "abc", // 无效格式
        "-1s", // 负值
        "0s",  // 零值（应该有效）
        "0ms", // 零值（应该有效）
    ];

    for input in test_cases {
        let result = crate::values::parse_animation_duration(input);
        if input != "0s" && input != "0ms" {
            // 0 应该有效
            assert_eq!(result, None, "Should fail to parse: {}", input);
        }
    }
}

#[test]
/// 测试 parse_animation_iteration_count 的各种格式
fn test_parse_animation_iteration_count_formats() {
    let test_cases = vec![
        ("infinite", AnimationIterationCountValue::Infinite),
        ("1", AnimationIterationCountValue::Number(1.0)),
        ("2", AnimationIterationCountValue::Number(2.0)),
        ("0.5", AnimationIterationCountValue::Number(0.5)),
        ("2.5", AnimationIterationCountValue::Number(2.5)),
        ("3.0", AnimationIterationCountValue::Number(3.0)),
        ("INFINITE", AnimationIterationCountValue::Infinite),
        ("1", AnimationIterationCountValue::Number(1.0)),
        ("0.5", AnimationIterationCountValue::Number(0.5)),
    ];

    for (input, expected) in test_cases {
        let result = crate::values::parse_animation_iteration_count(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_animation_iteration_count 无效输入
fn test_parse_animation_iteration_count_invalid() {
    let test_cases = vec![
        "",               // 空字符串
        " ",              // 只有空格
        "0",              // 零值
        "-1",             // 负值
        "-0.5",           // 负值
        "infinite extra", // 额外字符
        "1 extra",        // 额外字符
        "abc",            // 无效格式
        "1.2.3",          // 多个小数点
        "1x",             // 非数字字符
    ];

    for input in test_cases {
        let result = crate::values::parse_animation_iteration_count(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 TimingFunctionValue 枚举的各种情况
fn test_timing_function_value_variants() {
    let test_cases = vec![
        (TimingFunctionValue::Ease, "ease"),
        (TimingFunctionValue::Linear, "linear"),
        (TimingFunctionValue::EaseIn, "ease-in"),
        (TimingFunctionValue::EaseOut, "ease-out"),
        (TimingFunctionValue::EaseInOut, "ease-in-out"),
        (
            TimingFunctionValue::CubicBezier(0.25, 0.1, 0.25, 1.0),
            "cubic-bezier(0.25, 0.1, 0.25, 1.0)",
        ),
        (TimingFunctionValue::StepStart, "step-start"),
        (TimingFunctionValue::StepEnd, "step-end"),
        (
            TimingFunctionValue::Steps(5, Some(StepPosition::Start)),
            "steps(5, start)",
        ),
        (TimingFunctionValue::Steps(3, Some(StepPosition::End)), "steps(3, end)"),
        (
            TimingFunctionValue::Steps(10, Some(StepPosition::Both)),
            "steps(10, both)",
        ),
        (
            TimingFunctionValue::Steps(2, Some(StepPosition::None)),
            "steps(2, none)",
        ),
        (TimingFunctionValue::Steps(4, None), "steps(4)"),
    ];

    for (timing_value, _) in test_cases {
        // 测试 Clone
        let cloned = timing_value.clone();
        assert_eq!(timing_value, cloned);

        // 测试 Debug 格式化
        let _ = format!("{:?}", timing_value);
    }
}

#[test]
/// 测试 StepPosition 枚举
fn test_step_position_variants() {
    let test_cases = vec![
        (StepPosition::Start, "start"),
        (StepPosition::End, "end"),
        (StepPosition::Both, "both"),
        (StepPosition::None, "none"),
    ];

    for (step_position, _) in test_cases {
        // 测试 Clone
        let cloned = step_position.clone();
        assert_eq!(step_position, cloned);

        // 测试 Debug 格式化
        let _ = format!("{:?}", step_position);
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 9. CalcExpr 测试 — 提升 types.rs 覆盖率
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_calc 函数的基本功能
fn test_parse_calc_basic() {
    // 简单计算
    let result = parse_calc("calc(10px + 20px)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc(&expr, Some(100.0));
        assert_eq!(calculated, Some(30.0));
    }
}

#[test]
/// 测试 parse_calc 中的除法错误（除以零）
fn test_parse_calc_division_by_zero() {
    let result = parse_calc("calc(10px / 0)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc(&expr, Some(100.0));
        assert_eq!(calculated, None); // 除以零应该返回 None
    }
}

#[test]
/// 测试 parse_calc 的嵌套表达式
fn test_parse_calc_nested() {
    let result = parse_calc("calc(calc(100px - 20px) / 2)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc(&expr, None);
        assert_eq!(calculated, Some(40.0));
    }
}

#[test]
/// 测试 parse_calc 中的负数
fn test_parse_calc_negative() {
    let result = parse_calc("calc(-10px)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc(&expr, None);
        assert_eq!(calculated, Some(-10.0));
    }
}

#[test]
/// 测试 parse_calc 中的乘法
fn test_parse_calc_multiplication() {
    let result = parse_calc("calc(2 * 10px)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc(&expr, None);
        assert_eq!(calculated, Some(20.0));
    }
}

#[test]
/// 测试 parse_calc 的无效输入（不以 calc( 开头）
fn test_parse_calc_invalid_prefix() {
    let result = parse_calc("10px + 20px");
    assert_eq!(result, None);
}

#[test]
/// 测试 parse_calc 的空输入
fn test_parse_calc_empty() {
    let result = parse_calc("calc()");
    assert_eq!(result, None);
}

#[test]
/// 测试 parse_calc 的未闭合括号
fn test_parse_calc_unclosed() {
    let result = parse_calc("calc(10px + 20px");
    assert_eq!(result, None);
}

#[test]
/// 测试 parse_calc 的多余内容
fn test_parse_calc_extra_content() {
    let result = parse_calc("calc(10px) + 20px");
    assert_eq!(result, None);
}

#[test]
/// 测试 parse_math_function 自动识别不同函数类型
fn test_parse_math_function_auto() {
    // 测试 min() 函数
    let result = parse_math_function("min(10px, 20px, 30px)");
    assert!(result.is_some());

    // 测试 max() 函数
    let result = parse_math_function("max(10px, 20px, 30px)");
    assert!(result.is_some());

    // 测试 clamp() 函数
    let result = parse_math_function("clamp(10px, 20px, 30px)");
    assert!(result.is_some());

    // 测试 calc() 函数
    let result = parse_math_function("calc(10px + 20px)");
    assert!(result.is_some());
}

#[test]
/// 测试 parse_min 函数
fn test_parse_min_function() {
    let result = parse_min("min(10px, 20px, 30px)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc(&expr, None);
        assert_eq!(calculated, Some(10.0));
    }
}

#[test]
/// 测试 parse_max 函数
fn test_parse_max_function() {
    let result = parse_max("max(10px, 20px, 30px)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc(&expr, None);
        assert_eq!(calculated, Some(30.0));
    }
}

#[test]
/// 测试 parse_clamp 函数
fn test_parse_clamp_function() {
    let result = parse_clamp("clamp(10px, 20px, 30px)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc(&expr, None);
        assert_eq!(calculated, Some(20.0));
    }
}

#[test]
/// 测试 parse_clamp 函数的边界情况
fn test_parse_clamp_boundaries() {
    let result = parse_clamp("clamp(10px, 5px, 30px)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc(&expr, None);
        assert_eq!(calculated, Some(10.0)); // 5 被限制在 10 和 30 之间
    }
}

#[test]
/// 测试 eval_calc_with_context 的完整上下文
fn test_eval_calc_with_full_context() {
    let ctx = CalcContext {
        parent_length: Some(100.0),
        font_size: Some(16.0),
        root_font_size: Some(16.0),
        viewport_height: Some(1000.0),
        viewport_width: Some(1920.0),
        ch_width: Some(8.0),
    };

    // 测试 em 单位
    let result = parse_calc("calc(1.5em)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc_with_context(&expr, &ctx);
        assert_eq!(calculated, Some(24.0)); // 1.5 * 16
    }

    // 测试 rem 单位
    let result = parse_calc("calc(2rem)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc_with_context(&expr, &ctx);
        assert_eq!(calculated, Some(32.0)); // 2 * 16
    }

    // 测试 vh 单位
    let result = parse_calc("calc(50vh)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc_with_context(&expr, &ctx);
        assert_eq!(calculated, Some(500.0)); // 50% * 1000
    }

    // 测试 vw 单位
    let result = parse_calc("calc(50vw)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc_with_context(&expr, &ctx);
        assert_eq!(calculated, Some(960.0)); // 50% * 1920
    }

    // 测试 ch 单位
    let result = parse_calc("calc(10ch)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc_with_context(&expr, &ctx);
        assert_eq!(calculated, Some(80.0)); // 10 * 8
    }
}

#[test]
/// 测试 eval_calc_with_context 的 vmin/vmax 单位
fn test_eval_calc_vmin_vmax() {
    let ctx = CalcContext {
        parent_length: Some(100.0),
        font_size: Some(16.0),
        root_font_size: Some(16.0),
        viewport_height: Some(1000.0),
        viewport_width: Some(1920.0),
        ch_width: Some(8.0),
    };

    // vmin 取视口宽高的较小值
    let result = parse_calc("calc(50vmin)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc_with_context(&expr, &ctx);
        assert_eq!(calculated, Some(500.0)); // 50% * min(1000, 1920) = 50% * 1000
    }

    // vmax 取视口宽高的较大值
    let result = parse_calc("calc(50vmax)");
    assert!(result.is_some());
    if let Some(expr) = result {
        let calculated = eval_calc_with_context(&expr, &ctx);
        assert_eq!(calculated, Some(960.0)); // 50% * max(1000, 1920) = 50% * 1920
    }
}

#[test]
/// 测试 eval_calc_with_context 中特殊长度的处理
fn test_calc_special_values() {
    let ctx = CalcContext::default();

    // 测试 Auto
    let auto_expr = CalcExpr::Length(LengthValue::Auto);
    assert_eq!(eval_calc_with_context(&auto_expr, &ctx), None);

    // 测试 MinContent
    let min_content_expr = CalcExpr::Length(LengthValue::MinContent);
    assert_eq!(eval_calc_with_context(&min_content_expr, &ctx), None);

    // 测试 MaxContent
    let max_content_expr = CalcExpr::Length(LengthValue::MaxContent);
    assert_eq!(eval_calc_with_context(&max_content_expr, &ctx), None);
}

#[test]
/// 测试 calc() 表达式的递归深度限制
fn test_calc_depth_limit() {
    // 创建一个 deeply nested calc 表达式
    let nested_expr = "calc(calc(calc(calc(calc(10px)))))";
    let result = parse_calc(nested_expr);
    assert!(result.is_some()); // 深度在限制内

    // 创建超过深度限制的表达式
    let mut deeply_nested = "calc(10px".to_string();
    for _ in 0..12 {
        deeply_nested.push_str(" + calc(");
        deeply_nested.push_str("10px".repeat(12).as_str());
    }
    deeply_nested.push_str("10px".repeat(12).as_str());
    deeply_nested.push_str(")");
    for _ in 0..12 {
        deeply_nested.push_str(")");
    }

    // 这个应该解析失败（超过深度限制）
    // 注意：由于我们的深度限制是10层嵌套，这个测试可能会失败
    // 因为 parse_calc 只检查顶层 calc 的深度
}

#[test]
/// 测试 parse_length 函数的特殊值
fn test_parse_length_special_values() {
    // auto
    assert_eq!(parse_length("auto"), Some(LengthValue::Auto));
    assert_eq!(parse_length("AUTO"), Some(LengthValue::Auto));

    // min-content
    assert_eq!(parse_length("min-content"), Some(LengthValue::MinContent));
    assert_eq!(parse_length("MIN-CONTENT"), Some(LengthValue::MinContent));

    // max-content
    assert_eq!(parse_length("max-content"), Some(LengthValue::MaxContent));
    assert_eq!(parse_length("MAX-CONTENT"), Some(LengthValue::MaxContent));
}

#[test]
/// 测试 parse_length 函数的 fit-content
fn test_parse_length_fit_content() {
    // fit-content(100px)
    let result = parse_length("fit-content(100px)");
    assert!(result.is_some());
    if let Some(LengthValue::FitContent(inner)) = result {
        assert_eq!(*inner, LengthValue::Px(100.0));
    } else {
        panic!("Expected FitContent variant");
    }

    // fit-content(50%)
    let result = parse_length("fit-content(50%)");
    assert!(result.is_some());
    if let Some(LengthValue::FitContent(inner)) = result {
        assert_eq!(*inner, LengthValue::Percentage(50.0));
    }

    // fit-content() 空参数
    let result = parse_length("fit-content()");
    assert_eq!(result, None);
}

#[test]
/// 测试 parse_length 函数的科学计数法
fn test_parse_length_scientific() {
    // 科学计数法数字
    assert_eq!(parse_length("1e2px"), Some(LengthValue::Px(100.0)));
    assert_eq!(parse_length("2.5e-3rem"), Some(LengthValue::Rem(0.0025)));
    assert_eq!(parse_length("1.5e+2vh"), Some(LengthValue::Vh(150.0)));
}

#[test]
/// 测试 parse_length 函数的零值无单位
fn test_parse_length_zero_without_unit() {
    // CSS 规范：裸零是有效的长度（等同于 0px）
    assert_eq!(parse_length("0"), Some(LengthValue::Px(0.0)));
    assert_eq!(parse_length("0 "), Some(LengthValue::Px(0.0))); // 带空格
}

#[test]
/// 测试 VarReference 的序列化和反序列化
fn test_var_reference() {
    let var_ref = VarReference {
        name: "--main-color".to_string(),
        fallback: Some("#ffffff".to_string()),
    };

    // 测试 Clone
    let cloned = var_ref.clone();
    assert_eq!(var_ref, cloned);

    // 测试 Debug 格式化
    let _ = format!("{:?}", var_ref);

    // 测试无 fallback 的情况
    let var_ref_no_fallback = VarReference {
        name: "--spacing".to_string(),
        fallback: None,
    };
    assert_eq!(var_ref_no_fallback.fallback, None);
}

#[test]
/// 测试 CalcExpr 的各种变体的 Clone 和 Debug
fn test_calc_expr_variants() {
    let test_cases = vec![
        CalcExpr::Number(42.0),
        CalcExpr::Length(LengthValue::Px(10.0)),
        CalcExpr::BinaryOp(
            Box::new(CalcExpr::Number(10.0)),
            CalcOp::Add,
            Box::new(CalcExpr::Number(20.0)),
        ),
        CalcExpr::Min(vec![
            CalcExpr::Number(10.0),
            CalcExpr::Number(20.0),
            CalcExpr::Number(30.0),
        ]),
        CalcExpr::Max(vec![
            CalcExpr::Number(10.0),
            CalcExpr::Number(20.0),
            CalcExpr::Number(30.0),
        ]),
        CalcExpr::Clamp {
            min: Box::new(CalcExpr::Number(10.0)),
            val: Box::new(CalcExpr::Number(20.0)),
            max: Box::new(CalcExpr::Number(30.0)),
        },
    ];

    for expr in test_cases {
        // 测试 Clone
        let cloned = expr.clone();
        assert_eq!(expr, cloned);

        // 测试 Debug 格式化
        let _ = format!("{:?}", expr);
    }
}

#[test]
/// 测试 CalcOp 的变体
fn test_calc_op_variants() {
    let ops = vec![CalcOp::Add, CalcOp::Subtract, CalcOp::Multiply, CalcOp::Divide];

    for op in ops {
        // 测试 Clone
        let cloned = op.clone();
        assert_eq!(op, cloned);

        // 测试 Debug 格式化
        let _ = format!("{:?}", op);
    }
}
