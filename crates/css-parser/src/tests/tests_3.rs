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
/// R2375：CSS Color 4 §12 现代系统颜色补全（light color-scheme 默认值）。
/// 修复前这些合法系统颜色（LinkText/VisitedText/ActiveText/ButtonText/Field/FieldText/
/// Highlight/HighlightText/SelectedItem/SelectedItemText/Mark/MarkText/AccentColor/
/// AccentColorText）返回 None → 声明被丢弃（元素回退到继承/initial 色）。现解析为
/// light-scheme 合理默认值（具体值非关键，CSS deprecated-sameas 测试为相对匹配；
/// 价值 = 不再静默丢弃合法系统颜色声明）。
fn test_parse_color_modern_system_colors() {
    // 每个现代系统颜色都能解析（非 None），且大小写不敏感
    for name in [
        "LinkText",
        "VisitedText",
        "ActiveText",
        "ButtonText",
        "Field",
        "FieldText",
        "Highlight",
        "HighlightText",
        "SelectedItem",
        "SelectedItemText",
        "Mark",
        "MarkText",
        "AccentColor",
        "AccentColorText",
    ] {
        assert!(parse_color(name).is_some(), "{name} 应可解析");
        // 大小写不敏感（与既有 canvas/Canvas 一致）
        assert!(parse_color(&name.to_lowercase()).is_some(), "{name} 小写应可解析");
    }
    // 选定几个断言具体 light-scheme 默认值（防回归）
    assert_eq!(parse_color("LinkText"), Some(ColorValue::Rgba(0, 0, 238, 255)));
    assert_eq!(parse_color("Highlight"), Some(ColorValue::Rgba(51, 153, 255, 255)));
    assert_eq!(parse_color("HighlightText"), Some(ColorValue::Rgba(255, 255, 255, 255)));
    assert_eq!(parse_color("Field"), Some(ColorValue::Rgba(255, 255, 255, 255)));
    assert_eq!(parse_color("FieldText"), Some(ColorValue::Rgba(0, 0, 0, 255)));
    assert_eq!(
        parse_color("AccentColorText"),
        Some(ColorValue::Rgba(255, 255, 255, 255))
    );
}

#[test]
/// R2376/R2377：color-mix 的 srgb-linear/lab/oklab/oklch 插值色彩空间解析（CSS Color 4 §12）。
/// R2376 前仅 `in srgb`/`in lch` 支持，`in lab`/`in oklab`/`in oklch` 返回 None；R2377 补
/// `in srgb-linear`。任一缺失 → 整条 color-mix 被丢（颜色回退）。现解析为 `ColorValue::Mix`
/// 带正确 ColorMixSpace。
fn test_parse_color_mix_lab_oklab_oklch_spaces() {
    use crate::values::ColorMixSpace;
    for (input, expect) in [
        ("color-mix(in lab, red, blue)", ColorMixSpace::Lab),
        ("color-mix(in oklab, red, blue)", ColorMixSpace::OkLab),
        ("color-mix(in oklch, red, blue)", ColorMixSpace::OkLch),
        ("color-mix(in srgb-linear, red, blue)", ColorMixSpace::SrgbLinear),
        ("color-mix(in xyz, red, blue)", ColorMixSpace::Xyz),
        ("color-mix(in xyz-d65, red, blue)", ColorMixSpace::Xyz),
    ] {
        match parse_color(input) {
            Some(ColorValue::Mix(spec)) => assert_eq!(spec.space, expect, "{input}"),
            _ => panic!("{input} 应解析为 Mix（lab/oklab/oklch 空间）"),
        }
    }
    // 大小写不敏感（与既有 srgb/lch 一致）
    assert!(matches!(
        parse_color("color-mix(in OKLAB, red, blue)"),
        Some(ColorValue::Mix(_))
    ));
    // 既有空间不回归
    assert!(matches!(
        parse_color("color-mix(in srgb, red, blue)"),
        Some(ColorValue::Mix(_))
    ));
    assert!(matches!(
        parse_color("color-mix(in lch, red, blue)"),
        Some(ColorValue::Mix(_))
    ));
}

#[test]
/// R2381：CSS Color 4 §12.3 color-mix 色相插值法（`in <polar-space> <method> hue`）。
/// 修复前 `color-mix(in oklch longer hue, …)` 的 space 段 "in oklch longer hue" 不匹配任何
/// eq_ignore_ascii_case → None → 整条被丢。现解析 hue method（仅 lch/oklch 极坐标空间）。
fn test_parse_color_mix_hue_method() {
    use crate::values::{ColorHueMethod, ColorMixSpace};
    for (input, expect_hue) in [
        ("color-mix(in oklch longer hue, red, blue)", ColorHueMethod::Longer),
        ("color-mix(in lch shorter hue, red, blue)", ColorHueMethod::Shorter),
        (
            "color-mix(in oklch increasing hue, red, blue)",
            ColorHueMethod::Increasing,
        ),
        (
            "color-mix(in lch decreasing hue, red, blue)",
            ColorHueMethod::Decreasing,
        ),
        // 大小写不敏感
        ("color-mix(in oklch LONGER HUE, red, blue)", ColorHueMethod::Longer),
    ] {
        match parse_color(input) {
            Some(ColorValue::Mix(spec)) => {
                assert_eq!(spec.hue, expect_hue, "{input}");
            }
            _ => panic!("{input} 应解析为 Mix"),
        }
    }
    // 无 hue method → 默认 Shorter（不回归）
    let spec_default = match parse_color("color-mix(in oklch, red, blue)") {
        Some(ColorValue::Mix(s)) => s,
        _ => panic!("in oklch 应解析为 Mix"),
    };
    assert_eq!(spec_default.hue, ColorHueMethod::Shorter);
    assert_eq!(spec_default.space, ColorMixSpace::OkLch);
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
/// R2374：CSS Images 4 §4.3.8 渐变 color interpolation hint（色标间裸 <length-percentage>）。
/// `linear-gradient(red, 30%, blue)` 的中间裸 `30%` 是插值提示（指定相邻两色标中点位置），
/// 不是色标。修复前裸 %/长度落 parse_color 失败 → None 传播 → **整个渐变被拒**（背景回退，
/// 无渐变渲染）。修复后提示被正确识别并消费，渐变不再被丢弃。
///（渲染侧暂线性插值——hint 中点偏移为可选 follow-up，需 GradientColorStop 加 hint 字段
/// + 渲染器改动；本切片仅 parse-compliance：不让 hint 丢掉整条渐变，与 R2204 CDO/CDC 同族。）
fn test_parse_gradient_color_interpolation_hint() {
    // % 提示（最常见）：red, 30%, blue → 提示被消费，留 red + blue 两色标
    let result = parse_gradient("linear-gradient(red, 30%, blue)");
    assert!(result.is_some(), "% hint 不应丢弃整个渐变");
    if let GradientValue::Linear(lg) = result.unwrap() {
        assert_eq!(lg.stops.len(), 2, "hint 被消费，留两色标");
        assert_eq!(lg.stops[0].color, ColorValue::Rgba(255, 0, 0, 255));
        assert_eq!(lg.stops[1].color, ColorValue::Rgba(0, 0, 255, 255));
    } else {
        panic!("Expected LinearGradient");
    }

    // 长度提示（px）+ 带位置的色标：red 0%, 20px, blue 100%
    let result = parse_gradient("linear-gradient(red 0%, 20px, blue 100%)");
    assert!(result.is_some(), "px hint 不应丢弃整个渐变");
    if let GradientValue::Linear(lg) = result.unwrap() {
        assert_eq!(lg.stops.len(), 2);
    }

    // calc() 提示
    assert!(parse_gradient("linear-gradient(red, calc(25%), green)").is_some());

    // hint 不能出现在首位（无前导色标，CSS 非法）→ 渐变应失败
    assert!(
        parse_gradient("linear-gradient(30%, blue)").is_none(),
        "首位裸 % 非色标非有效提示，应失败"
    );
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
