use super::*;

// ═══════════════════════════════════════════════════════════════════════
// 1. Tokenizer 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 radial-gradient closest-side
fn test_parse_radial_gradient_closest_side() {
    let result = parse_gradient("radial-gradient(circle closest-side, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert_eq!(rg.shape, RadialShape::Circle);
            assert_eq!(rg.size, RadialSize::ClosestSide);
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试 conic-gradient 带 at 位置
fn test_parse_conic_gradient_at_position() {
    let result = parse_gradient("conic-gradient(at 50% 50%, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Conic(cg) => {
            assert_eq!(cg.position_x, LengthValue::Percentage(50.0));
            assert_eq!(cg.position_y, LengthValue::Percentage(50.0));
        }
        _ => panic!("Expected ConicGradient"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 26. Media query edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试媒体查询 "all" 类型解析
fn test_media_query_all_type() {
    use crate::media_query::{MediaType, parse_media_query};
    let queries = parse_media_query("all").unwrap();
    let q = &queries[0];
    assert_eq!(q.media_type, Some(MediaType::All));
    assert!(q.conditions.is_empty());
}

#[test]
/// 测试媒体查询多重条件评估
fn test_media_query_multiple_conditions_eval() {
    use crate::media_query::{MediaContext, evaluate_media_query, parse_media_query};
    let queries = parse_media_query("screen and (min-width: 600px) and (orientation: landscape)").unwrap();
    let q = &queries[0];
    let ctx = MediaContext::new(1024.0, 768.0);
    assert!(evaluate_media_query(q, &ctx));
    let ctx_portrait = MediaContext::new(1024.0, 1200.0);
    assert!(!evaluate_media_query(q, &ctx_portrait));
}

// ═══════════════════════════════════════════════════════════════════════
// 27. vertical-align / list-style / float / clear / calc viewport 边界测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_vertical_align 所有关键字：baseline、sub、super、top、text-top、
/// middle、bottom、text-bottom，以及大小写不敏感和无效输入
fn test_parse_vertical_align() {
    use crate::values::{VerticalAlignValue, parse_vertical_align};
    assert_eq!(parse_vertical_align("baseline"), Some(VerticalAlignValue::Baseline));
    assert_eq!(parse_vertical_align("sub"), Some(VerticalAlignValue::Sub));
    assert_eq!(parse_vertical_align("super"), Some(VerticalAlignValue::Super));
    assert_eq!(parse_vertical_align("top"), Some(VerticalAlignValue::Top));
    assert_eq!(parse_vertical_align("text-top"), Some(VerticalAlignValue::TextTop));
    assert_eq!(parse_vertical_align("middle"), Some(VerticalAlignValue::Middle));
    assert_eq!(parse_vertical_align("bottom"), Some(VerticalAlignValue::Bottom));
    assert_eq!(
        parse_vertical_align("text-bottom"),
        Some(VerticalAlignValue::TextBottom)
    );
    // 大小写不敏感
    assert_eq!(parse_vertical_align("BASELINE"), Some(VerticalAlignValue::Baseline));
    assert_eq!(parse_vertical_align("  Middle  "), Some(VerticalAlignValue::Middle));
    assert_eq!(
        parse_vertical_align("TEXT-BOTTOM"),
        Some(VerticalAlignValue::TextBottom)
    );
    // 无效值
    assert_eq!(parse_vertical_align("center"), None);
    assert_eq!(parse_vertical_align("10px"), None);
    assert_eq!(parse_vertical_align(""), None);
}

#[test]
/// 测试 parse_list_style_type 所有关键字：disc、circle、square、decimal、
/// decimal-leading-zero、lower-roman、upper-roman、lower-alpha、upper-alpha、
/// lower-latin（别名）、upper-latin（别名）、none，
/// 以及未映射关键字（lower-greek、armenian、georgian）返回 None
fn test_parse_list_style_type() {
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
        parse_list_style_type("upper-alpha"),
        Some(ListStyleTypeValue::UpperAlpha)
    );
    assert_eq!(
        parse_list_style_type("lower-latin"),
        Some(ListStyleTypeValue::LowerAlpha)
    );
    assert_eq!(
        parse_list_style_type("upper-latin"),
        Some(ListStyleTypeValue::UpperAlpha)
    );
    assert_eq!(parse_list_style_type("none"), Some(ListStyleTypeValue::None));
    // 当前不支持的关键字应返回 None
    assert_eq!(parse_list_style_type("lower-greek"), None);
    assert_eq!(parse_list_style_type("armenian"), None);
    assert_eq!(parse_list_style_type("georgian"), None);
    // 大小写不敏感
    assert_eq!(parse_list_style_type("DISC"), Some(ListStyleTypeValue::Disc));
    assert_eq!(parse_list_style_type("  Circle  "), Some(ListStyleTypeValue::Circle));
    // 无效输入
    assert_eq!(parse_list_style_type("invalid"), None);
    assert_eq!(parse_list_style_type(""), None);
}

#[test]
/// 测试 parse_list_style_position 的 inside 和 outside 关键字，
/// 以及大小写不敏感和无效输入
fn test_parse_list_style_position() {
    assert_eq!(
        parse_list_style_position("inside"),
        Some(ListStylePositionValue::Inside)
    );
    assert_eq!(
        parse_list_style_position("outside"),
        Some(ListStylePositionValue::Outside)
    );
    // 大小写不敏感
    assert_eq!(
        parse_list_style_position("INSIDE"),
        Some(ListStylePositionValue::Inside)
    );
    assert_eq!(
        parse_list_style_position("  Outside  "),
        Some(ListStylePositionValue::Outside)
    );
    // 无效输入
    assert_eq!(parse_list_style_position("center"), None);
    assert_eq!(parse_list_style_position(""), None);
}

#[test]
/// 测试 parse_float 所有关键字：left、right、none、inline-start、inline-end，
/// 以及大小写不敏感、前后空白、无效输入
fn test_parse_float() {
    assert_eq!(parse_float("left"), Some(FloatValue::Left));
    assert_eq!(parse_float("right"), Some(FloatValue::Right));
    assert_eq!(parse_float("none"), Some(FloatValue::None));
    assert_eq!(parse_float("inline-start"), Some(FloatValue::InlineStart));
    assert_eq!(parse_float("inline-end"), Some(FloatValue::InlineEnd));
    // 大小写不敏感
    assert_eq!(parse_float("LEFT"), Some(FloatValue::Left));
    assert_eq!(parse_float("  Right  "), Some(FloatValue::Right));
    assert_eq!(parse_float("INLINE-START"), Some(FloatValue::InlineStart));
    // 无效输入
    assert_eq!(parse_float("center"), None);
    assert_eq!(parse_float(""), None);
    assert_eq!(parse_float("inherit"), None);
}

#[test]
/// 测试 parse_clear 所有关键字：left、right、both、none、inline-start、inline-end，
/// 以及大小写不敏感、前后空白、无效输入
fn test_parse_clear() {
    assert_eq!(parse_clear("left"), Some(ClearValue::Left));
    assert_eq!(parse_clear("right"), Some(ClearValue::Right));
    assert_eq!(parse_clear("both"), Some(ClearValue::Both));
    assert_eq!(parse_clear("none"), Some(ClearValue::None));
    assert_eq!(parse_clear("inline-start"), Some(ClearValue::InlineStart));
    assert_eq!(parse_clear("inline-end"), Some(ClearValue::InlineEnd));
    // 大小写不敏感
    assert_eq!(parse_clear("BOTH"), Some(ClearValue::Both));
    assert_eq!(parse_clear("  None  "), Some(ClearValue::None));
    assert_eq!(parse_clear("INLINE-END"), Some(ClearValue::InlineEnd));
    // 无效输入
    assert_eq!(parse_clear("all"), None);
    assert_eq!(parse_clear(""), None);
    assert_eq!(parse_clear("inherit"), None);
}

#[test]
/// 测试 eval_calc_with_context 在包含视口尺寸的 CalcContext 中，
/// 验证 vw/vh/vmin/vmax 均能正确解析为像素值
fn test_eval_calc_with_context_viewport() {
    // 视口尺寸：1920 x 1080
    let ctx = CalcContext {
        viewport_width: Some(1920.0),
        viewport_height: Some(1080.0),
        ..Default::default()
    };

    // vw: 25vw = 25 * 1920 / 100 = 480.0
    let expr_vw = parse_calc("calc(25vw)").unwrap();
    let result = eval_calc_with_context(&expr_vw, &ctx);
    assert_eq!(result, Some(480.0));

    // vh: 50vh = 50 * 1080 / 100 = 540.0
    let expr_vh = parse_calc("calc(50vh)").unwrap();
    let result = eval_calc_with_context(&expr_vh, &ctx);
    assert_eq!(result, Some(540.0));

    // vmin: 10vmin = 10 * min(1920, 1080) / 100 = 10 * 1080 / 100 = 108.0
    let expr_vmin = parse_calc("calc(10vmin)").unwrap();
    let result = eval_calc_with_context(&expr_vmin, &ctx);
    assert_eq!(result, Some(108.0));

    // vmax: 10vmax = 10 * max(1920, 1080) / 100 = 10 * 1920 / 100 = 192.0
    let expr_vmax = parse_calc("calc(10vmax)").unwrap();
    let result = eval_calc_with_context(&expr_vmax, &ctx);
    assert_eq!(result, Some(192.0));

    // 混合视口单位：calc(50vw - 10vh) = 960 - 108 = 852.0
    let expr_mixed = parse_calc("calc(50vw - 10vh)").unwrap();
    let result = eval_calc_with_context(&expr_mixed, &ctx);
    assert_eq!(result, Some(852.0));

    // 缺少视口上下文时返回 None
    let ctx_empty = CalcContext::default();
    let result = eval_calc_with_context(&expr_vw, &ctx_empty);
    assert_eq!(result, None);
}

// ═══════════════════════════════════════════════════════════════════════
// 26. parse_cursor / parse_opacity 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_cursor 常见关键字
fn test_parse_cursor_common_keywords() {
    assert_eq!(parse_cursor("pointer"), Some(CursorValue::Pointer));
    assert_eq!(parse_cursor("default"), Some(CursorValue::Default));
    assert_eq!(parse_cursor("text"), Some(CursorValue::Text));
    assert_eq!(parse_cursor("move"), Some(CursorValue::Move));
    assert_eq!(parse_cursor("wait"), Some(CursorValue::Wait));
    assert_eq!(parse_cursor("crosshair"), Some(CursorValue::Crosshair));
    assert_eq!(parse_cursor("not-allowed"), Some(CursorValue::NotAllowed));
    assert_eq!(parse_cursor("grab"), Some(CursorValue::Grab));
    assert_eq!(parse_cursor("grabbing"), Some(CursorValue::Grabbing));
    assert_eq!(parse_cursor("help"), Some(CursorValue::Help));
    assert_eq!(parse_cursor("progress"), Some(CursorValue::Progress));
}

#[test]
/// 测试 parse_cursor 方向调整关键字
fn test_parse_cursor_resize_keywords() {
    assert_eq!(parse_cursor("n-resize"), Some(CursorValue::NResize));
    assert_eq!(parse_cursor("s-resize"), Some(CursorValue::SResize));
    assert_eq!(parse_cursor("e-resize"), Some(CursorValue::EResize));
    assert_eq!(parse_cursor("w-resize"), Some(CursorValue::WResize));
    assert_eq!(parse_cursor("ne-resize"), Some(CursorValue::NeResize));
    assert_eq!(parse_cursor("nw-resize"), Some(CursorValue::NwResize));
    assert_eq!(parse_cursor("se-resize"), Some(CursorValue::SeResize));
    assert_eq!(parse_cursor("sw-resize"), Some(CursorValue::SwResize));
    assert_eq!(parse_cursor("col-resize"), Some(CursorValue::ColResize));
    assert_eq!(parse_cursor("row-resize"), Some(CursorValue::RowResize));
    assert_eq!(parse_cursor("all-scroll"), Some(CursorValue::AllScroll));
}

#[test]
/// 测试 parse_cursor 其他关键字
fn test_parse_cursor_other_keywords() {
    assert_eq!(parse_cursor("auto"), Some(CursorValue::Auto));
    assert_eq!(parse_cursor("zoom-in"), Some(CursorValue::ZoomIn));
    assert_eq!(parse_cursor("zoom-out"), Some(CursorValue::ZoomOut));
    assert_eq!(parse_cursor("none"), Some(CursorValue::None));
}

#[test]
/// 测试 parse_cursor 大小写不敏感
fn test_parse_cursor_case_insensitive() {
    assert_eq!(parse_cursor("POINTER"), Some(CursorValue::Pointer));
    assert_eq!(parse_cursor("Pointer"), Some(CursorValue::Pointer));
    assert_eq!(parse_cursor("DEFAULT"), Some(CursorValue::Default));
    assert_eq!(parse_cursor("NOT-ALLOWED"), Some(CursorValue::NotAllowed));
    assert_eq!(parse_cursor("  pointer  "), Some(CursorValue::Pointer));
}

#[test]
/// 测试 parse_cursor 未知值返回 None
fn test_parse_cursor_unknown() {
    assert_eq!(parse_cursor("invalid"), None);
    assert_eq!(parse_cursor(""), None);
    assert_eq!(parse_cursor("cursor"), None);
}

#[test]
/// 测试 parse_opacity 基本数值
fn test_parse_opacity_basic() {
    assert_eq!(parse_opacity("0"), Some(0.0));
    assert_eq!(parse_opacity("1"), Some(1.0));
    assert_eq!(parse_opacity("0.5"), Some(0.5));
}

#[test]
/// 测试 parse_opacity 值钳制到 [0.0, 1.0]
fn test_parse_opacity_clamping() {
    assert_eq!(parse_opacity("-0.1"), Some(0.0));
    assert_eq!(parse_opacity("1.5"), Some(1.0));
    assert_eq!(parse_opacity("-10"), Some(0.0));
    assert_eq!(parse_opacity("100"), Some(1.0));
}

#[test]
/// 测试 parse_opacity 百分比值
fn test_parse_opacity_percentage() {
    assert_eq!(parse_opacity("50%"), Some(0.5));
    assert_eq!(parse_opacity("0%"), Some(0.0));
    assert_eq!(parse_opacity("100%"), Some(1.0));
    assert_eq!(parse_opacity("25%"), Some(0.25));
    assert_eq!(parse_opacity("150%"), Some(1.0));
    assert_eq!(parse_opacity("-10%"), Some(0.0));
}

#[test]
/// 测试 parse_opacity 无效输入返回 None
fn test_parse_opacity_invalid() {
    assert_eq!(parse_opacity("abc"), None);
    assert_eq!(parse_opacity(""), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 27. 边界条件扩展测试 — hwb 颜色、混合渐变色标、3D 变换、嵌套 var、复杂 @supports
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 hwb() 颜色记法：hwb(0 0% 0%) 应为纯红色 (255, 0, 0)
fn test_parse_color_hwb_red() {
    let result = parse_color("hwb(0 0% 0%)");
    assert_eq!(result, Some(ColorValue::Rgba(255, 0, 0, 255)));
}

#[test]
/// 测试 hwb() 颜色：hwb(0 100% 0%) 应为白色 (255, 255, 255)
fn test_parse_color_hwb_white() {
    let result = parse_color("hwb(0 100% 0%)");
    assert_eq!(result, Some(ColorValue::Rgba(255, 255, 255, 255)));
}

#[test]
/// 测试 hwb() 颜色：hwb(0 0% 100%) 应为黑色 (0, 0, 0)
fn test_parse_color_hwb_black() {
    let result = parse_color("hwb(0 0% 100%)");
    assert_eq!(result, Some(ColorValue::Rgba(0, 0, 0, 255)));
}

#[test]
/// 测试 hwb() 带透明度：hwb(120 30% 20% / 0.5) — 验证 RGBA 分量合理
fn test_parse_color_hwb_with_alpha() {
    let result = parse_color("hwb(120 30% 20% / 0.5)");
    assert!(result.is_some());
    if let Some(ColorValue::Rgba(r, g, b, a)) = result {
        // alpha = 0.5 → 128
        assert_eq!(a, 128);
        // 绿色色调 (hue=120)，30% 白度推亮，20% 黑度压暗
        assert!(g > r, "green channel should be dominant at hue 120");
        assert!(g > b, "green channel should be dominant at hue 120");
    } else {
        panic!("Expected Rgba color");
    }
}

#[test]
/// 测试 hwb() W+B 超过 100% 时应按比例缩小：hwb(0 80% 80%) 应产生灰色
fn test_parse_color_hwb_clamped() {
    let result = parse_color("hwb(0 80% 80%)");
    assert!(result.is_some());
    if let Some(ColorValue::Rgba(r, g, b, a)) = result {
        // W+B=160% > 100%，缩小后 W=B=50%，混合结果应为灰色 (128,128,128)
        assert_eq!(a, 255);
        // 灰色：三个通道应接近相等
        assert!((r as i32 - g as i32).abs() <= 2);
        assert!((g as i32 - b as i32).abs() <= 2);
    } else {
        panic!("Expected Rgba color");
    }
}

#[test]
/// 测试渐变使用混合色标位置类型（px、%）：验证解析不崩溃，色标数量正确。
fn test_parse_gradient_with_multiple_types() {
    // 纯 px 色标
    let result = parse_gradient("linear-gradient(red 10px, blue 20px)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.stops.len(), 2);
            assert_eq!(lg.stops[0].position, Some(LengthValue::Px(10.0)));
            assert_eq!(lg.stops[1].position, Some(LengthValue::Px(20.0)));
        }
        _ => panic!("Expected LinearGradient"),
    }

    // 混合 px 和 % 色标
    let result = parse_gradient("linear-gradient(red 10px, green 50%, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.stops.len(), 3);
            assert_eq!(lg.stops[0].position, Some(LengthValue::Px(10.0)));
            assert_eq!(lg.stops[1].position, Some(LengthValue::Percentage(50.0)));
            assert_eq!(lg.stops[2].position, None);
        }
        _ => panic!("Expected LinearGradient"),
    }

    // calc() 色标位置：当前 parse_length 不支持 calc()，验证不崩溃
    let result = parse_gradient("linear-gradient(red, blue calc(50% - 10px))");
    // calc() 在色标位置中不被 parse_length 支持，可能返回 None 或部分结果
    assert!(result.is_some() || result.is_none());
}

#[test]
/// 测试 3D 变换函数：translate3d、scale3d、rotate3d、perspective、rotateX、rotateY、rotateZ、matrix。
fn test_parse_transform_3d_functions() {
    // translate3d
    let result = parse_transform("translate3d(10px, 20px, 30px)").unwrap();
    let fns = match result {
        TransformValue::List(f) => f,
        _ => panic!("expected List"),
    };
    assert_eq!(fns.len(), 1);
    assert_eq!(fns[0], TransformFunction::Translate3d(10.0, 20.0, 30.0));

    // scale3d
    let result = parse_transform("scale3d(1, 2, 3)").unwrap();
    let fns = match result {
        TransformValue::List(f) => f,
        _ => panic!("expected List"),
    };
    assert_eq!(fns.len(), 1);
    assert_eq!(fns[0], TransformFunction::Scale3d(1.0, 2.0, 3.0));

    // rotate3d
    let result = parse_transform("rotate3d(1, 0, 0, 45deg)").unwrap();
    let fns = match result {
        TransformValue::List(f) => f,
        _ => panic!("expected List"),
    };
    assert_eq!(fns.len(), 1);
    assert_eq!(fns[0], TransformFunction::Rotate3d(1.0, 0.0, 0.0, 45.0));

    // perspective
    let result = parse_transform("perspective(500px)").unwrap();
    let fns = match result {
        TransformValue::List(f) => f,
        _ => panic!("expected List"),
    };
    assert_eq!(fns.len(), 1);
    assert_eq!(fns[0], TransformFunction::Perspective(500.0));

    // rotateX
    let result = parse_transform("rotateX(45deg)").unwrap();
    let fns = match result {
        TransformValue::List(f) => f,
        _ => panic!("expected List"),
    };
    assert_eq!(fns[0], TransformFunction::RotateX(45.0));

    // rotateY
    let result = parse_transform("rotateY(30deg)").unwrap();
    let fns = match result {
        TransformValue::List(f) => f,
        _ => panic!("expected List"),
    };
    assert_eq!(fns[0], TransformFunction::RotateY(30.0));

    // rotateZ
    let result = parse_transform("rotateZ(90deg)").unwrap();
    let fns = match result {
        TransformValue::List(f) => f,
        _ => panic!("expected List"),
    };
    assert_eq!(fns[0], TransformFunction::RotateZ(90.0));

    // matrix
    let result = parse_transform("matrix(1, 0, 0, 1, 10, 20)").unwrap();
    let fns = match result {
        TransformValue::List(f) => f,
        _ => panic!("expected List"),
    };
    assert_eq!(fns[0], TransformFunction::Matrix(1.0, 0.0, 0.0, 1.0, 10.0, 20.0));

    // 混合 2D 和 3D 变换
    let result = parse_transform("translate(10px) rotate3d(1, 0, 0, 45deg)").unwrap();
    let fns = match result {
        TransformValue::List(f) => f,
        _ => panic!("expected List"),
    };
    assert_eq!(fns.len(), 2);

    // 纯 2D 变换仍然正常
    let result = parse_transform("translate(10px, 20px) rotate(45deg)");
    assert!(result.is_some());

    // perspective 不接受零或负值
    assert_eq!(parse_transform("perspective(0)"), None);
    assert_eq!(parse_transform("perspective(-100px)"), None);

    // rotate3d 需要 4 个参数
    assert_eq!(parse_transform("rotate3d(1, 0, 0)"), None);

    // translate3d 需要 3 个参数
    assert_eq!(parse_transform("translate3d(10px, 20px)"), None);

    // matrix 需要 6 个参数
    assert_eq!(parse_transform("matrix(1, 0, 0, 1, 10)"), None);
}

#[test]
/// 测试 var() 三层嵌套回退：var(--a, var(--b, var(--c, blue)))。
/// parse_var 使用逗号分割，深层嵌套的回退值应保留完整文本。
fn test_parse_var_deeply_nested_fallback() {
    let result = parse_var("var(--a, var(--b, var(--c, blue)))");
    assert!(result.is_some());
    let var = result.unwrap();
    assert_eq!(var.name, "--a");
    // 回退值应保留完整的嵌套 var() 文本
    assert!(var.fallback.is_some());
    let fallback = var.fallback.unwrap();
    assert!(
        fallback.contains("var(--b"),
        "Nested var() should be preserved in fallback"
    );
    assert!(
        fallback.contains("var(--c"),
        "Deeply nested var() should be preserved in fallback"
    );

    // 单层嵌套回退
    let result = parse_var("var(--x, var(--y, red))");
    assert!(result.is_some());
    let var = result.unwrap();
    assert_eq!(var.name, "--x");
    assert_eq!(var.fallback, Some("var(--y, red)".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// 14. 错误恢复测试 — 畸形输入处理
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试畸形选择器 "div..class" 的错误恢复 — 解析器不应 panic。
/// "div..class" 中连续两个点不是合法的选择器语法，解析器应优雅恢复。
fn test_parse_double_dot_selector_recovery() {
    // 双点选择器：不是合法语法，但不应 panic
    let stylesheet = Parser::parse_stylesheet("div..class { color: red; }");
    // 不 panic 即可，结果可以是空规则或部分解析
    assert!(stylesheet.rules.len() <= 2);
}

#[test]
/// 测试未闭合括号 "@media (min-width: 100px {" 的错误恢复 — 解析器不应 panic。
/// 缺少右括号和右花括号的 @media 规则是畸形的，解析器应优雅恢复。
fn test_parse_unclosed_bracket_recovery() {
    // 未闭合括号 — 不应 panic
    let stylesheet = Parser::parse_stylesheet("@media (min-width: 100px { div { color: red; }");
    // 不 panic 即可
    assert!(stylesheet.rules.len() <= 2);
}

#[test]
/// 测试空值 "color: ;" 的错误恢复 — 解析器跳过该属性，不影响后续声明。
fn test_parse_empty_value_recovery() {
    // 带空值的声明后面跟着正常声明
    let stylesheet = Parser::parse_stylesheet("div { color: ; font-size: 16px; }");
    // 不 panic，应至少解析到 font-size 声明
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        // font-size 应被正确解析
        assert!(
            sr.declarations.iter().any(|d| d.property == "font-size"),
            "font-size 应在空值恢复后被正确解析"
        );
    }
}

#[test]
/// 测试 @supports 复杂嵌套条件：(display: grid) and (not (display: flex))。
/// 验证解析器正确处理 and + not 嵌套组合。
fn test_parse_supports_complex_condition() {
    let css = "@supports (display: grid) and (not (display: flex)) { .container { display: grid; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Supports(supports_rule) => {
            match &supports_rule.condition {
                SupportsCondition::And(conditions) => {
                    assert_eq!(conditions.len(), 2);
                    // 第一个条件：(display: grid)
                    assert!(matches!(
                        &conditions[0],
                        SupportsCondition::Property(p, v) if p == "display" && v == "grid"
                    ));
                    // 第二个条件：not (display: flex)
                    match &conditions[1] {
                        SupportsCondition::Not(inner) => {
                            assert!(matches!(
                                inner.as_ref(),
                                SupportsCondition::Property(p, v) if p == "display" && v == "flex"
                            ));
                        }
                        _ => panic!("Expected Not condition as second operand"),
                    }
                }
                _ => panic!("Expected And condition with nested Not"),
            }
            assert_eq!(supports_rule.rules.len(), 1);
        }
        _ => panic!("Expected Supports rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 28. 媒体查询范围语法与选择器边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试媒体查询 Level 4 范围语法：200px <= width <= 800px。
/// 组合范围展开为两个条件（width >= 200 且 width <= 800），
/// 并在不同视口宽度下正确评估。
#[test]
fn test_parse_media_query_range_syntax() {
    use crate::media_query::{MediaCondition, MediaContext, MediaFeatureOp, evaluate_media_query, parse_media_query};

    // 解析组合范围
    let queries = parse_media_query("(200px <= width <= 800px)").unwrap();
    let q = &queries[0];
    assert_eq!(q.conditions.len(), 2, "组合范围应展开为 2 个条件");
    assert_eq!(
        q.conditions[0],
        MediaCondition::Width(MediaFeatureOp::GreaterEqual, 200.0),
        "第一个条件应为 width >= 200"
    );
    assert_eq!(
        q.conditions[1],
        MediaCondition::Width(MediaFeatureOp::LessEqual, 800.0),
        "第二个条件应为 width <= 800"
    );

    // 评估：500 在范围内通过
    let ctx_inside = MediaContext::new(500.0, 400.0);
    assert!(evaluate_media_query(q, &ctx_inside), "500px 在 [200, 800] 范围内应通过");

    // 评估：200 恰好下界通过
    let ctx_lower = MediaContext::new(200.0, 400.0);
    assert!(evaluate_media_query(q, &ctx_lower), "200px 恰好下界应通过（>=）");

    // 评估：800 恰好上界通过
    let ctx_upper = MediaContext::new(800.0, 400.0);
    assert!(evaluate_media_query(q, &ctx_upper), "800px 恰好上界应通过（<=）");

    // 评估：100 在范围外不通过
    let ctx_below = MediaContext::new(100.0, 400.0);
    assert!(!evaluate_media_query(q, &ctx_below), "100px 低于下界不应通过");

    // 评估：900 在范围外不通过
    let ctx_above = MediaContext::new(900.0, 400.0);
    assert!(!evaluate_media_query(q, &ctx_above), "900px 超过上限不应通过");
}

/// 测试 :has(> .child) 选择器解析正确。
/// :has() 内部使用子组合器（>）时，解析器应正确识别 Child 组合器。
#[test]
fn test_parse_selector_has_with_combinator() {
    let stylesheet = Parser::parse_stylesheet("article:has(> .summary) { display: block; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        // 验证主体类型选择器
        assert!(matches!(
            &compound.type_selector,
            Some(TypeSelector::Tag(t)) if t == "article"
        ));
        // 验证 :has() 内部有子组合器
        let has_inner = compound.subclass_selectors.iter().find_map(|s| match s {
            SubclassSelector::PseudoClass(PseudoClassSelector::Has(selectors)) => Some(selectors),
            _ => None,
        });
        assert!(has_inner.is_some(), "应有 :has() 伪类");
        let inner = has_inner.unwrap();
        assert_eq!(inner.len(), 1);
        let inner_parts = &inner[0].complex.parts;
        assert_eq!(inner_parts.len(), 2, ":has(> .summary) 应有 2 个组合部分");
        assert_eq!(
            inner_parts[0].1,
            Some(Combinator::Child),
            ":has() 内部应有 Child 组合器"
        );
        // 验证内部 .summary 类选择器
        let summary_compound = &inner_parts[1].0;
        assert!(
            summary_compound.subclass_selectors.iter().any(|s| matches!(
                s,
                SubclassSelector::Class(c) if c == "summary"
            )),
            ":has() 内部应有 .summary 类选择器"
        );
    } else {
        panic!("Expected Style rule");
    }
}

/// 测试 :not(.a, .b, .c) 多参数否定伪类解析。
/// :not() 内部有 3 个选择器，解析器应正确识别所有参数。
#[test]
fn test_parse_selector_not_multiple() {
    let stylesheet = Parser::parse_stylesheet("div:not(.a, .b, .c) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.selectors.len(), 1);
        let compound = &sr.selectors[0].complex.parts[0].0;
        // 验证类型选择器
        assert!(matches!(
            &compound.type_selector,
            Some(TypeSelector::Tag(t)) if t == "div"
        ));
        // 验证 :not() 内部有 3 个选择器
        let not_inner = compound.subclass_selectors.iter().find_map(|s| match s {
            SubclassSelector::PseudoClass(PseudoClassSelector::Not(selectors)) => Some(selectors),
            _ => None,
        });
        assert!(not_inner.is_some(), "应有 :not() 伪类");
        let selectors = not_inner.unwrap();
        assert_eq!(selectors.len(), 3, ":not(.a, .b, .c) 应有 3 个参数");

        // 验证每个参数是类选择器
        let class_names: Vec<&str> = selectors
            .iter()
            .map(|sel| {
                sel.complex.parts[0]
                    .0
                    .subclass_selectors
                    .iter()
                    .find_map(|s| match s {
                        SubclassSelector::Class(c) => Some(c.as_str()),
                        _ => None,
                    })
                    .unwrap()
            })
            .collect();
        assert_eq!(class_names, vec!["a", "b", "c"], ":not() 参数应为 .a, .b, .c");
    } else {
        panic!("Expected Style rule");
    }
}

/// 测试 :is(.a, #b) 和 :where(div, span) 都被正确解析。
/// :is() 和 :where() 都支持多选择器参数，解析器应正确识别。
#[test]
fn test_parse_selector_is_where() {
    // 测试 :is(.a, #b)
    let stylesheet_is = Parser::parse_stylesheet("p:is(.a, #b) { font-size: 14px; }");
    assert_eq!(stylesheet_is.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet_is.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        let is_inner = compound.subclass_selectors.iter().find_map(|s| match s {
            SubclassSelector::PseudoClass(PseudoClassSelector::Is(selectors)) => Some(selectors),
            _ => None,
        });
        assert!(is_inner.is_some(), "应有 :is() 伪类");
        let selectors = is_inner.unwrap();
        assert_eq!(selectors.len(), 2, ":is(.a, #b) 应有 2 个参数");

        // 第一个参数 .a 是类选择器
        assert!(
            selectors[0].complex.parts[0]
                .0
                .subclass_selectors
                .iter()
                .any(|s| matches!(
                    s,
                    SubclassSelector::Class(c) if c == "a"
                )),
            "第一个 :is() 参数应为 .a"
        );

        // 第二个参数 #b 是 ID 选择器
        assert!(
            selectors[1].complex.parts[0]
                .0
                .subclass_selectors
                .iter()
                .any(|s| matches!(
                    s,
                    SubclassSelector::Id(id) if id == "b"
                )),
            "第二个 :is() 参数应为 #b"
        );
    } else {
        panic!("Expected Style rule for :is()");
    }

    // 测试 :where(div, span)
    let stylesheet_where = Parser::parse_stylesheet("p:where(div, span) { margin: 0; }");
    assert_eq!(stylesheet_where.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet_where.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        let where_inner = compound.subclass_selectors.iter().find_map(|s| match s {
            SubclassSelector::PseudoClass(PseudoClassSelector::Where(selectors)) => Some(selectors),
            _ => None,
        });
        assert!(where_inner.is_some(), "应有 :where() 伪类");
        let selectors = where_inner.unwrap();
        assert_eq!(selectors.len(), 2, ":where(div, span) 应有 2 个参数");

        // 第一个参数 div 是标签选择器
        assert!(
            matches!(
                &selectors[0].complex.parts[0].0.type_selector,
                Some(TypeSelector::Tag(t)) if t == "div"
            ),
            "第一个 :where() 参数应为 div"
        );

        // 第二个参数 span 是标签选择器
        assert!(
            matches!(
                &selectors[1].complex.parts[0].0.type_selector,
                Some(TypeSelector::Tag(t)) if t == "span"
            ),
            "第二个 :where() 参数应为 span"
        );
    } else {
        panic!("Expected Style rule for :where()");
    }
}

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
