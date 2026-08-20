// Auto-generated test file — split from values.rs
use super::super::*;

#[test]
fn test_parse_tab_size_invalid() {
    assert_eq!(parse_tab_size("-1"), None);
    assert_eq!(parse_tab_size("abc"), None);
    assert_eq!(parse_tab_size(""), None);
    assert_eq!(parse_tab_size("-1px"), None);
    assert_eq!(parse_tab_size("-0.5em"), None);
    assert_eq!(parse_tab_size("50%"), None);
    assert_eq!(parse_tab_size("thin"), None);
    assert_eq!(parse_tab_size("min-content"), None);
}

// ═══════════════════════════════════════════════════════════════════
// 边界条件/错误路径边缘测试
// ═══════════════════════════════════════════════════════════════════

/// 测试 eval_calc 除以零返回 None
#[test]
fn test_eval_calc_divide_by_zero() {
    let expr = parse_calc("calc(10px / 0)").unwrap();
    let result = eval_calc(&expr, None);
    // 除以 0 应返回 None（除法分支的边界保护）
    assert_eq!(result, None);
}

/// 测试 parse_length 边界条件：负值、零值、无单位非零、科学计数法
#[test]
fn test_parse_length_boundary_conditions() {
    // 负值应正常解析
    assert_eq!(parse_length("-10px"), Some(LengthValue::Px(-10.0)));
    assert_eq!(parse_length("-5em"), Some(LengthValue::Em(-5.0)));
    // 零值（无单位）应解析为 Px(0.0)
    assert_eq!(parse_length("0"), Some(LengthValue::Px(0.0)));
    // 非零无单位值应返回 None
    assert_eq!(parse_length("5"), None);
    // 未知单位应返回 None
    assert_eq!(parse_length("10abc"), None);
    // 百分比零值
    assert_eq!(parse_length("0%"), Some(LengthValue::Percentage(0.0)));
    // 负百分比
    assert_eq!(parse_length("-50%"), Some(LengthValue::Percentage(-50.0)));
    // 科学计数法解析
    assert_eq!(parse_length("1e2px"), Some(LengthValue::Px(100.0)));
}

#[test]
fn test_parse_text_decoration_thickness_rejects_negative_length() {
    assert_eq!(
        parse_text_decoration_thickness("2px"),
        Some(TextDecorationThicknessValue::Length(LengthValue::Px(2.0)))
    );
    assert_eq!(
        parse_text_decoration_thickness("4em"),
        Some(TextDecorationThicknessValue::Length(LengthValue::Em(4.0)))
    );
    assert_eq!(
        parse_text_decoration_thickness("100%"),
        Some(TextDecorationThicknessValue::Length(LengthValue::Percentage(100.0)))
    );
    assert!(matches!(
        parse_text_decoration_thickness("calc(1em)"),
        Some(TextDecorationThicknessValue::Length(LengthValue::Calc(_)))
    ));
    assert_eq!(parse_text_decoration_thickness("-1px"), None);
    assert_eq!(parse_text_decoration_thickness("-1%"), None);
    assert_eq!(parse_text_decoration_thickness("thin"), None);
    assert_eq!(parse_text_decoration_thickness("min-content"), None);
    assert_eq!(parse_text_decoration_thickness("fit-content(10px)"), None);
    assert_eq!(parse_text_decoration_thickness("infpx"), None);
    assert_eq!(parse_text_decoration_thickness("NaNpx"), None);
}

/// 测试 parse_color 边界条件：无效十六进制长度、超出范围的 rgb 分量、空 hwb
#[test]
fn test_parse_color_edge_cases() {
    // 无效十六进制长度（2、5、7 位）应返回 None
    assert_eq!(parse_color("#12"), None);
    assert_eq!(parse_color("#12345"), None);
    assert_eq!(parse_color("#1234567"), None);
    // 仅 # 号应返回 None
    assert_eq!(parse_color("#"), None);
    // rgb 超出范围的分量（>255）应被 clamp
    let result = parse_color("rgb(300, -10, 128)");
    assert!(result.is_some());
    match result {
        Some(ColorValue::Rgba(r, g, b, _)) => {
            assert_eq!(r, 255); // 300 被钳制到 255
            assert_eq!(g, 0); // -10 被钳制到 0
            assert_eq!(b, 128);
        }
        _ => panic!("expected Rgba"),
    }
    // rgb 只有 2 个分量应返回 None
    assert_eq!(parse_color("rgb(255, 0)"), None);
    // hwb() 无效格式（缺少参数）应返回 None
    assert_eq!(parse_color("hwb(120 50%)"), None);
}

/// 测试 parse_opacity 边界条件：0、1、超出范围值、百分比边界
#[test]
fn test_parse_opacity_boundary() {
    // 0.0 和 1.0 边界
    assert_eq!(parse_opacity("0"), Some(0.0));
    assert_eq!(parse_opacity("1"), Some(1.0));
    assert_eq!(parse_opacity("0.0"), Some(0.0));
    assert_eq!(parse_opacity("1.0"), Some(1.0));
    // 超出范围值应被 clamp
    assert_eq!(parse_opacity("-0.5"), Some(0.0));
    assert_eq!(parse_opacity("2.0"), Some(1.0));
    // 百分比边界
    assert_eq!(parse_opacity("0%"), Some(0.0));
    assert_eq!(parse_opacity("100%"), Some(1.0));
    assert_eq!(parse_opacity("150%"), Some(1.0));
    assert_eq!(parse_opacity("-10%"), Some(0.0));
    // 非法输入
    assert_eq!(parse_opacity("abc"), None);
    assert_eq!(parse_opacity(""), None);
    assert_eq!(parse_opacity("inf"), None);
    assert_eq!(parse_opacity("-inf"), None);
    assert_eq!(parse_opacity("NaN"), None);
    assert_eq!(parse_opacity("inf%"), None);
}

/// 测试 parse_var 边界条件：空名称、回退值为空、嵌套 var
#[test]
fn test_parse_var_edge_cases() {
    // 基本解析带回退值
    let result = parse_var("var(--color, red)").unwrap();
    assert_eq!(result.name, "--color");
    assert_eq!(result.fallback, Some("red".to_string()));
    // 仅名称，无回退
    let result = parse_var("var(--spacing)").unwrap();
    assert_eq!(result.name, "--spacing");
    assert_eq!(result.fallback, None);
    // 不以 var( 开头应返回 None
    assert_eq!(parse_var("calc(10px)"), None);
    // 空字符串应返回 None
    assert_eq!(parse_var(""), None);
    // 缺少右括号应返回 None
    assert_eq!(parse_var("var(--color"), None);
    // CSS Values §4：函数名大小写不敏感（VAR/Var ≡ var）；自定义属性名大小写敏感（保持原样）。
    let result = parse_var("VAR(--color, red)").unwrap();
    assert_eq!(result.name, "--color");
    assert_eq!(result.fallback, Some("red".to_string()));
    let result = parse_var("Var(--MIXED-Case)").unwrap();
    assert_eq!(result.name, "--MIXED-Case");
}

// ── break-inside 测试 ──

#[test]
fn test_parse_break_inside_valid() {
    assert_eq!(parse_break_inside("auto"), Some(BreakInsideValue::Auto));
    assert_eq!(parse_break_inside("avoid"), Some(BreakInsideValue::Avoid));
    assert_eq!(parse_break_inside("avoid-page"), Some(BreakInsideValue::AvoidPage));
    assert_eq!(parse_break_inside("avoid-column"), Some(BreakInsideValue::AvoidColumn));
}

#[test]
fn test_parse_break_inside_case_insensitive() {
    assert_eq!(parse_break_inside("AVOID"), Some(BreakInsideValue::Avoid));
    assert_eq!(parse_break_inside("  Avoid-Page  "), Some(BreakInsideValue::AvoidPage));
}

#[test]
fn test_parse_break_inside_invalid() {
    assert_eq!(parse_break_inside("column"), None);
    assert_eq!(parse_break_inside("page"), None);
    assert_eq!(parse_break_inside("invalid"), None);
    assert_eq!(parse_break_inside(""), None);
}

// ── break-before / break-after 测试 ──

#[test]
fn test_parse_break_before_valid() {
    assert_eq!(parse_break_before("auto"), Some(BreakValue::Auto));
    assert_eq!(parse_break_before("avoid"), Some(BreakValue::Avoid));
    assert_eq!(parse_break_before("column"), Some(BreakValue::Column));
    assert_eq!(parse_break_before("page"), Some(BreakValue::Page));
    assert_eq!(parse_break_before("avoid-page"), Some(BreakValue::AvoidPage));
    assert_eq!(parse_break_before("avoid-column"), Some(BreakValue::AvoidColumn));
}

#[test]
fn test_parse_break_after_valid() {
    assert_eq!(parse_break_after("auto"), Some(BreakValue::Auto));
    assert_eq!(parse_break_after("avoid"), Some(BreakValue::Avoid));
    assert_eq!(parse_break_after("column"), Some(BreakValue::Column));
    assert_eq!(parse_break_after("page"), Some(BreakValue::Page));
    assert_eq!(parse_break_after("avoid-page"), Some(BreakValue::AvoidPage));
    assert_eq!(parse_break_after("avoid-column"), Some(BreakValue::AvoidColumn));
}

#[test]
fn test_parse_break_before_after_invalid() {
    assert_eq!(parse_break_before("always"), None);
    assert_eq!(parse_break_before("invalid"), None);
    assert_eq!(parse_break_after("left"), None);
    assert_eq!(parse_break_after(""), None);
}

// ── column-rule-width 测试 ──

#[test]
fn test_parse_column_rule_width_keywords() {
    assert_eq!(parse_column_rule_width("medium"), Some(ColumnRuleWidthValue::Medium));
    assert_eq!(parse_column_rule_width("thin"), Some(ColumnRuleWidthValue::Thin));
    assert_eq!(parse_column_rule_width("thick"), Some(ColumnRuleWidthValue::Thick));
}

#[test]
fn test_parse_column_rule_width_length() {
    assert_eq!(
        parse_column_rule_width("2px"),
        Some(ColumnRuleWidthValue::Length(LengthValue::Px(2.0)))
    );
    assert_eq!(
        parse_column_rule_width("0.5em"),
        Some(ColumnRuleWidthValue::Length(LengthValue::Em(0.5)))
    );
}

#[test]
fn test_parse_column_rule_width_invalid() {
    assert_eq!(parse_column_rule_width("invalid"), None);
    assert_eq!(parse_column_rule_width(""), None);
    assert_eq!(parse_column_rule_width("-1px"), None);
    assert_eq!(parse_column_rule_width("-0.5em"), None);
}

#[test]
fn test_parse_column_width_invalid_consumer_grammar() {
    assert_eq!(parse_column_width("-1px"), None);
    assert_eq!(parse_column_width("-0.5em"), None);
    assert_eq!(parse_column_width("50%"), None);
    assert_eq!(parse_column_width("thin"), None);
    assert_eq!(parse_column_width("min-content"), None);
}

// ── column-rule-style 测试 ──

#[test]
fn test_parse_column_rule_style_all_values() {
    assert_eq!(parse_column_rule_style("none"), Some(ColumnRuleStyleValue::None));
    assert_eq!(parse_column_rule_style("hidden"), Some(ColumnRuleStyleValue::Hidden));
    assert_eq!(parse_column_rule_style("dotted"), Some(ColumnRuleStyleValue::Dotted));
    assert_eq!(parse_column_rule_style("dashed"), Some(ColumnRuleStyleValue::Dashed));
    assert_eq!(parse_column_rule_style("solid"), Some(ColumnRuleStyleValue::Solid));
    assert_eq!(parse_column_rule_style("double"), Some(ColumnRuleStyleValue::Double));
    assert_eq!(parse_column_rule_style("groove"), Some(ColumnRuleStyleValue::Groove));
    assert_eq!(parse_column_rule_style("ridge"), Some(ColumnRuleStyleValue::Ridge));
    assert_eq!(parse_column_rule_style("inset"), Some(ColumnRuleStyleValue::Inset));
    assert_eq!(parse_column_rule_style("outset"), Some(ColumnRuleStyleValue::Outset));
}

#[test]
fn test_parse_column_rule_style_case_insensitive() {
    assert_eq!(parse_column_rule_style("SOLID"), Some(ColumnRuleStyleValue::Solid));
    assert_eq!(
        parse_column_rule_style("  Dotted  "),
        Some(ColumnRuleStyleValue::Dotted)
    );
}

#[test]
fn test_parse_column_rule_style_invalid() {
    assert_eq!(parse_column_rule_style("invalid"), None);
    assert_eq!(parse_column_rule_style(""), None);
}

// ── Appearance 测试 ──

#[test]
fn test_parse_appearance_none() {
    assert_eq!(parse_appearance("none"), Some(AppearanceValue::None));
}

#[test]
fn test_parse_appearance_auto() {
    assert_eq!(parse_appearance("auto"), Some(AppearanceValue::Auto));
}

#[test]
fn test_parse_appearance_widgets() {
    assert_eq!(parse_appearance("button"), Some(AppearanceValue::Button));
    assert_eq!(parse_appearance("checkbox"), Some(AppearanceValue::Checkbox));
    assert_eq!(parse_appearance("listbox"), Some(AppearanceValue::Listbox));
    assert_eq!(parse_appearance("menulist"), Some(AppearanceValue::Menulist));
    assert_eq!(parse_appearance("meter"), Some(AppearanceValue::Meter));
    assert_eq!(parse_appearance("progress-bar"), Some(AppearanceValue::ProgressBar));
    assert_eq!(parse_appearance("push-button"), Some(AppearanceValue::PushButton));
    assert_eq!(parse_appearance("radio"), Some(AppearanceValue::Radio));
    assert_eq!(parse_appearance("searchfield"), Some(AppearanceValue::Searchfield));
    assert_eq!(
        parse_appearance("slider-horizontal"),
        Some(AppearanceValue::SliderHorizontal)
    );
    assert_eq!(parse_appearance("square-button"), Some(AppearanceValue::SquareButton));
    assert_eq!(parse_appearance("textarea"), Some(AppearanceValue::Textarea));
    assert_eq!(parse_appearance("textfield"), Some(AppearanceValue::Textfield));
}

#[test]
fn test_parse_appearance_case_insensitive() {
    assert_eq!(parse_appearance("NONE"), Some(AppearanceValue::None));
    assert_eq!(parse_appearance("  Auto  "), Some(AppearanceValue::Auto));
    assert_eq!(parse_appearance("BUTTON"), Some(AppearanceValue::Button));
}

#[test]
fn test_parse_appearance_invalid() {
    assert_eq!(parse_appearance("invalid"), None);
    assert_eq!(parse_appearance(""), None);
}

// ── AccentColor 测试 ──

#[test]
fn test_parse_accent_color_auto() {
    assert_eq!(parse_accent_color("auto"), Some(AccentColorValue::Auto));
}

#[test]
fn test_parse_accent_color_named() {
    assert_eq!(
        parse_accent_color("red"),
        Some(AccentColorValue::Color(ColorValue::Rgba(255, 0, 0, 255)))
    );
    assert_eq!(
        parse_accent_color("blue"),
        Some(AccentColorValue::Color(ColorValue::Rgba(0, 0, 255, 255)))
    );
}

#[test]
fn test_parse_accent_color_hex() {
    assert_eq!(
        parse_accent_color("#ff0000"),
        Some(AccentColorValue::Color(ColorValue::Rgba(255, 0, 0, 255)))
    );
    assert_eq!(
        parse_accent_color("#0f0"),
        Some(AccentColorValue::Color(ColorValue::Rgba(0, 255, 0, 255)))
    );
}

#[test]
fn test_parse_accent_color_rgb() {
    let result = parse_accent_color("rgb(100, 200, 50)");
    assert!(result.is_some());
    match result.unwrap() {
        AccentColorValue::Color(ColorValue::Rgba(r, g, b, a)) => {
            assert_eq!(r, 100);
            assert_eq!(g, 200);
            assert_eq!(b, 50);
            assert_eq!(a, 255);
        }
        _ => panic!("expected Color variant"),
    }
}

#[test]
fn test_parse_accent_color_invalid() {
    assert_eq!(parse_accent_color("not-a-color"), None);
    assert_eq!(parse_accent_color(""), None);
}

// ── CaretColor 测试 ──

#[test]
fn test_parse_caret_color_auto() {
    assert_eq!(parse_caret_color("auto"), Some(CaretColorValue::Auto));
}

#[test]
fn test_parse_caret_color_named() {
    assert_eq!(
        parse_caret_color("green"),
        Some(CaretColorValue::Color(ColorValue::Rgba(0, 128, 0, 255)))
    );
}

#[test]
fn test_parse_caret_color_hex() {
    assert_eq!(
        parse_caret_color("#abcdef"),
        Some(CaretColorValue::Color(ColorValue::Rgba(0xAB, 0xCD, 0xEF, 255)))
    );
}

#[test]
fn test_parse_caret_color_transparent() {
    assert_eq!(
        parse_caret_color("transparent"),
        Some(CaretColorValue::Color(ColorValue::Transparent))
    );
}

#[test]
fn test_parse_caret_color_invalid() {
    assert_eq!(parse_caret_color("not-a-color"), None);
    assert_eq!(parse_caret_color(""), None);
}

// ── MixBlendMode 测试 ──

#[test]
fn test_parse_mix_blend_mode_normal() {
    assert_eq!(parse_mix_blend_mode("normal"), Some(MixBlendModeValue::Normal));
}

#[test]
fn test_parse_mix_blend_mode_all_values() {
    assert_eq!(parse_mix_blend_mode("multiply"), Some(MixBlendModeValue::Multiply));
    assert_eq!(parse_mix_blend_mode("screen"), Some(MixBlendModeValue::Screen));
    assert_eq!(parse_mix_blend_mode("overlay"), Some(MixBlendModeValue::Overlay));
    assert_eq!(parse_mix_blend_mode("darken"), Some(MixBlendModeValue::Darken));
    assert_eq!(parse_mix_blend_mode("lighten"), Some(MixBlendModeValue::Lighten));
    assert_eq!(parse_mix_blend_mode("color-dodge"), Some(MixBlendModeValue::ColorDodge));
    assert_eq!(parse_mix_blend_mode("color-burn"), Some(MixBlendModeValue::ColorBurn));
    assert_eq!(parse_mix_blend_mode("hard-light"), Some(MixBlendModeValue::HardLight));
    assert_eq!(parse_mix_blend_mode("soft-light"), Some(MixBlendModeValue::SoftLight));
    assert_eq!(parse_mix_blend_mode("difference"), Some(MixBlendModeValue::Difference));
    assert_eq!(parse_mix_blend_mode("exclusion"), Some(MixBlendModeValue::Exclusion));
    assert_eq!(parse_mix_blend_mode("hue"), Some(MixBlendModeValue::Hue));
    assert_eq!(parse_mix_blend_mode("saturation"), Some(MixBlendModeValue::Saturation));
    assert_eq!(parse_mix_blend_mode("color"), Some(MixBlendModeValue::Color));
    assert_eq!(parse_mix_blend_mode("luminosity"), Some(MixBlendModeValue::Luminosity));
}

#[test]
fn test_parse_mix_blend_mode_case_insensitive() {
    assert_eq!(parse_mix_blend_mode("NORMAL"), Some(MixBlendModeValue::Normal));
    assert_eq!(parse_mix_blend_mode("  Multiply  "), Some(MixBlendModeValue::Multiply));
    assert_eq!(parse_mix_blend_mode("COLOR-DODGE"), Some(MixBlendModeValue::ColorDodge));
}

#[test]
fn test_parse_mix_blend_mode_invalid() {
    assert_eq!(parse_mix_blend_mode("invalid"), None);
    assert_eq!(parse_mix_blend_mode(""), None);
    assert_eq!(parse_mix_blend_mode("inherit"), None);
}

// ── ScrollbarWidth 测试 ──

#[test]
fn test_parse_scrollbar_width_auto() {
    assert_eq!(parse_scrollbar_width("auto"), Some(ScrollbarWidthValue::Auto));
}

#[test]
fn test_parse_scrollbar_width_thin() {
    assert_eq!(parse_scrollbar_width("thin"), Some(ScrollbarWidthValue::Thin));
}

#[test]
fn test_parse_scrollbar_width_none() {
    assert_eq!(parse_scrollbar_width("none"), Some(ScrollbarWidthValue::None));
}

#[test]
fn test_parse_scrollbar_width_case_insensitive() {
    assert_eq!(parse_scrollbar_width("AUTO"), Some(ScrollbarWidthValue::Auto));
    assert_eq!(parse_scrollbar_width("  Thin  "), Some(ScrollbarWidthValue::Thin));
    assert_eq!(parse_scrollbar_width("NONE"), Some(ScrollbarWidthValue::None));
}

#[test]
fn test_parse_scrollbar_width_invalid() {
    assert_eq!(parse_scrollbar_width("thick"), None);
    assert_eq!(parse_scrollbar_width(""), None);
}

// ── ScrollbarGutter 测试 ──

#[test]
fn test_parse_scrollbar_gutter_auto() {
    assert_eq!(parse_scrollbar_gutter("auto"), Some(ScrollbarGutterValue::Auto));
}

#[test]
fn test_parse_scrollbar_gutter_stable() {
    assert_eq!(parse_scrollbar_gutter("stable"), Some(ScrollbarGutterValue::Stable));
}

#[test]
fn test_parse_scrollbar_gutter_stable_both_edges() {
    assert_eq!(
        parse_scrollbar_gutter("stable both-edges"),
        Some(ScrollbarGutterValue::StableBothEdges)
    );
}

#[test]
fn test_parse_scrollbar_gutter_case_insensitive() {
    assert_eq!(parse_scrollbar_gutter("AUTO"), Some(ScrollbarGutterValue::Auto));
    assert_eq!(parse_scrollbar_gutter("  Stable  "), Some(ScrollbarGutterValue::Stable));
    assert_eq!(
        parse_scrollbar_gutter("STABLE BOTH-EDGES"),
        Some(ScrollbarGutterValue::StableBothEdges)
    );
}

#[test]
fn test_parse_scrollbar_gutter_invalid() {
    assert_eq!(parse_scrollbar_gutter("both"), None);
    assert_eq!(parse_scrollbar_gutter("both-edges"), None);
    assert_eq!(parse_scrollbar_gutter(""), None);
    assert_eq!(parse_scrollbar_gutter("invalid"), None);
}

// ── text-wrap 解析测试 ──

#[test]
fn test_parse_text_wrap_wrap() {
    assert_eq!(parse_text_wrap("wrap"), Some(TextWrapValue::Wrap));
}

#[test]
fn test_parse_text_wrap_nowrap() {
    assert_eq!(parse_text_wrap("nowrap"), Some(TextWrapValue::Nowrap));
}

#[test]
fn test_parse_text_wrap_balance() {
    assert_eq!(parse_text_wrap("balance"), Some(TextWrapValue::Balance));
}

#[test]
fn test_parse_text_wrap_pretty() {
    assert_eq!(parse_text_wrap("pretty"), Some(TextWrapValue::Pretty));
}

#[test]
fn test_parse_text_wrap_stable() {
    assert_eq!(parse_text_wrap("stable"), Some(TextWrapValue::Stable));
}

#[test]
fn test_parse_text_wrap_case_insensitive() {
    assert_eq!(parse_text_wrap("Wrap"), Some(TextWrapValue::Wrap));
    assert_eq!(parse_text_wrap("NOWRAP"), Some(TextWrapValue::Nowrap));
    assert_eq!(parse_text_wrap("Balance"), Some(TextWrapValue::Balance));
}

#[test]
fn test_parse_text_wrap_invalid() {
    assert_eq!(parse_text_wrap("invalid"), None);
    assert_eq!(parse_text_wrap(""), None);
    assert_eq!(parse_text_wrap("auto"), None);
}

// ── hyphens 解析测试 ──

#[test]
fn test_parse_hyphens_none() {
    assert_eq!(parse_hyphens("none"), Some(HyphensValue::None));
}

#[test]
fn test_parse_hyphens_manual() {
    assert_eq!(parse_hyphens("manual"), Some(HyphensValue::Manual));
}

#[test]
fn test_parse_hyphens_auto() {
    assert_eq!(parse_hyphens("auto"), Some(HyphensValue::Auto));
}

#[test]
fn test_parse_hyphens_case_insensitive() {
    assert_eq!(parse_hyphens("None"), Some(HyphensValue::None));
    assert_eq!(parse_hyphens("MANUAL"), Some(HyphensValue::Manual));
    assert_eq!(parse_hyphens("Auto"), Some(HyphensValue::Auto));
}

#[test]
fn test_parse_hyphens_invalid() {
    assert_eq!(parse_hyphens("invalid"), None);
    assert_eq!(parse_hyphens(""), None);
    assert_eq!(parse_hyphens("all"), None);
}

// ── line-clamp 解析测试 ──

#[test]
fn test_parse_line_clamp_none() {
    assert_eq!(parse_line_clamp("none"), Some(LineClampValue::None));
}

#[test]
fn test_parse_line_clamp_count() {
    assert_eq!(parse_line_clamp("3"), Some(LineClampValue::Count(3)));
    assert_eq!(parse_line_clamp("1"), Some(LineClampValue::Count(1)));
    assert_eq!(parse_line_clamp("10"), Some(LineClampValue::Count(10)));
}

#[test]
fn test_parse_line_clamp_case_insensitive() {
    assert_eq!(parse_line_clamp("None"), Some(LineClampValue::None));
    assert_eq!(parse_line_clamp("NONE"), Some(LineClampValue::None));
}

#[test]
fn test_parse_line_clamp_invalid() {
    assert_eq!(parse_line_clamp("0"), None);
    assert_eq!(parse_line_clamp("-1"), None);
    assert_eq!(parse_line_clamp("1.5"), None);
    assert_eq!(parse_line_clamp("auto"), None);
    assert_eq!(parse_line_clamp(""), None);
}

// ── background-image 解析测试 ──

#[test]
fn test_parse_background_image_none() {
    assert_eq!(parse_background_image("none"), Some(BackgroundImageValue::None));
}

#[test]
fn test_parse_background_image_url() {
    assert_eq!(
        parse_background_image("url(image.png)"),
        Some(BackgroundImageValue::Url("image.png".to_string()))
    );
}

#[test]
fn test_parse_background_image_url_quoted() {
    assert_eq!(
        parse_background_image("url(\"image.png\")"),
        Some(BackgroundImageValue::Url("image.png".to_string()))
    );
    assert_eq!(
        parse_background_image("url('image.png')"),
        Some(BackgroundImageValue::Url("image.png".to_string()))
    );
    assert_eq!(
        parse_background_image("url(\"my image.png\")"),
        Some(BackgroundImageValue::Url("my image.png".to_string()))
    );
}

#[test]
fn test_parse_background_image_url_with_path() {
    assert_eq!(
        parse_background_image("url(/path/to/image.png)"),
        Some(BackgroundImageValue::Url("/path/to/image.png".to_string()))
    );
}

#[test]
fn test_parse_background_image_case_insensitive() {
    assert_eq!(parse_background_image("NONE"), Some(BackgroundImageValue::None));
    assert_eq!(parse_background_image("None"), Some(BackgroundImageValue::None));
}

#[test]
fn test_parse_background_image_invalid() {
    assert_eq!(parse_background_image(""), None);
    assert_eq!(parse_background_image("invalid"), None);
    assert_eq!(parse_background_image("url()"), None);
    assert_eq!(parse_background_image("url(my image.png)"), None);
    assert_eq!(parse_background_image("url(\"image.png\" extra)"), None);
    assert_eq!(parse_background_image("url('image.png' extra)"), None);
    assert_eq!(parse_background_image("url(\"image.png)"), None);
    assert_eq!(parse_background_image("url(image\".png)"), None);
}

// ── background-position 解析测试 ──

#[test]
fn test_parse_background_position_keywords() {
    assert_eq!(
        parse_background_position("center"),
        Some(BackgroundPositionValue::Center)
    );
    assert_eq!(parse_background_position("left"), Some(BackgroundPositionValue::Left));
    assert_eq!(parse_background_position("right"), Some(BackgroundPositionValue::Right));
    assert_eq!(parse_background_position("top"), Some(BackgroundPositionValue::Top));
    assert_eq!(
        parse_background_position("bottom"),
        Some(BackgroundPositionValue::Bottom)
    );
}

#[test]
fn test_parse_background_position_percent() {
    assert_eq!(
        parse_background_position("50%"),
        Some(BackgroundPositionValue::Percent(50.0))
    );
    assert_eq!(
        parse_background_position("0%"),
        Some(BackgroundPositionValue::Percent(0.0))
    );
    assert_eq!(
        parse_background_position("100%"),
        Some(BackgroundPositionValue::Percent(100.0))
    );
}

#[test]
fn test_parse_background_position_length() {
    assert_eq!(
        parse_background_position("10px"),
        Some(BackgroundPositionValue::Length(LengthValue::Px(10.0)))
    );
    assert_eq!(
        parse_background_position("0px"),
        Some(BackgroundPositionValue::Length(LengthValue::Px(0.0)))
    );
    // R1417：em/rem/ex 等相对单位此前被拒（parse_length 返回 Em/Rem，旧 Length(f32) 仅匹配
    // Px），现保留 LengthValue 供 style-system apply 按 font-size 解析。
    assert_eq!(
        parse_background_position("-0em"),
        Some(BackgroundPositionValue::Length(LengthValue::Em(0.0)))
    );
    assert_eq!(
        parse_background_position("2em"),
        Some(BackgroundPositionValue::Length(LengthValue::Em(2.0)))
    );
    assert_eq!(
        parse_background_position("-10px"),
        Some(BackgroundPositionValue::Length(LengthValue::Px(-10.0)))
    );
}

#[test]
fn test_parse_background_position_rejects_invalid_length_grammar() {
    for value in [
        "thin",
        "medium",
        "thick",
        "auto",
        "min-content",
        "fit-content",
        "infpx",
        "NaNpx",
        "left thin",
        "right infpx top",
    ] {
        assert_eq!(parse_background_position(value), None, "{value} should be rejected");
    }
}

#[test]
fn test_parse_background_position_two_values() {
    let result = parse_background_position("left top");
    assert!(result.is_some());
    if let Some(BackgroundPositionValue::TwoValue(h, v)) = result {
        assert_eq!(*h, BackgroundPositionValue::Left);
        assert_eq!(*v, BackgroundPositionValue::Top);
    } else {
        panic!("Expected TwoValue");
    }
}

#[test]
fn test_parse_background_position_two_values_mixed() {
    let result = parse_background_position("center 50%");
    assert!(result.is_some());
    if let Some(BackgroundPositionValue::TwoValue(h, v)) = result {
        assert_eq!(*h, BackgroundPositionValue::Center);
        assert_eq!(*v, BackgroundPositionValue::Percent(50.0));
    } else {
        panic!("Expected TwoValue");
    }
}

#[test]
fn test_parse_background_position_case_insensitive() {
    assert_eq!(
        parse_background_position("Center"),
        Some(BackgroundPositionValue::Center)
    );
    assert_eq!(parse_background_position("LEFT"), Some(BackgroundPositionValue::Left));
}

#[test]
fn test_parse_background_position_invalid() {
    assert_eq!(parse_background_position(""), None);
    assert_eq!(parse_background_position("invalid"), None);
}

// ── background-repeat 解析测试 ──

#[test]
fn test_parse_background_repeat_values() {
    assert_eq!(parse_background_repeat("repeat"), Some(BackgroundRepeatValue::Repeat));
    assert_eq!(
        parse_background_repeat("repeat-x"),
        Some(BackgroundRepeatValue::RepeatX)
    );
    assert_eq!(
        parse_background_repeat("repeat-y"),
        Some(BackgroundRepeatValue::RepeatY)
    );
    assert_eq!(
        parse_background_repeat("no-repeat"),
        Some(BackgroundRepeatValue::NoRepeat)
    );
    assert_eq!(parse_background_repeat("space"), Some(BackgroundRepeatValue::Space));
    assert_eq!(parse_background_repeat("round"), Some(BackgroundRepeatValue::Round));
}

#[test]
fn test_parse_background_repeat_case_insensitive() {
    assert_eq!(parse_background_repeat("REPEAT"), Some(BackgroundRepeatValue::Repeat));
    assert_eq!(
        parse_background_repeat("No-Repeat"),
        Some(BackgroundRepeatValue::NoRepeat)
    );
    assert_eq!(
        parse_background_repeat("REPEAT-X"),
        Some(BackgroundRepeatValue::RepeatX)
    );
}

#[test]
fn test_parse_background_repeat_invalid() {
    assert_eq!(parse_background_repeat(""), None);
    assert_eq!(parse_background_repeat("invalid"), None);
    assert_eq!(parse_background_repeat("repeat z"), None);
}

// ── background-size 解析测试 ──

#[test]
fn test_parse_background_size_keywords() {
    assert_eq!(parse_background_size("auto"), Some(BackgroundSizeValue::Auto));
    assert_eq!(parse_background_size("cover"), Some(BackgroundSizeValue::Cover));
    assert_eq!(parse_background_size("contain"), Some(BackgroundSizeValue::Contain));
}

#[test]
fn test_parse_background_size_length() {
    assert_eq!(parse_background_size("100px"), Some(BackgroundSizeValue::Length(100.0)));
    assert_eq!(parse_background_size("1.5em"), Some(BackgroundSizeValue::Length(1.5)));
    assert_eq!(parse_background_size("2rem"), Some(BackgroundSizeValue::Length(2.0)));
    assert_eq!(parse_background_size("1vh"), Some(BackgroundSizeValue::Length(1.0)));
    assert_eq!(parse_background_size("2ch"), Some(BackgroundSizeValue::Length(2.0)));
    assert_eq!(
        parse_background_size("1vh 2ch"),
        Some(BackgroundSizeValue::TwoValue(
            BgSizeComponent::Length(1.0),
            BgSizeComponent::Length(2.0)
        ))
    );
}

#[test]
fn test_parse_background_size_percent() {
    assert_eq!(parse_background_size("50%"), Some(BackgroundSizeValue::Percent(50.0)));
    assert_eq!(parse_background_size("100%"), Some(BackgroundSizeValue::Percent(100.0)));
}

#[test]
fn test_parse_background_size_case_insensitive() {
    assert_eq!(parse_background_size("AUTO"), Some(BackgroundSizeValue::Auto));
    assert_eq!(parse_background_size("Cover"), Some(BackgroundSizeValue::Cover));
    assert_eq!(parse_background_size("CONTAIN"), Some(BackgroundSizeValue::Contain));
}

#[test]
fn test_parse_background_size_invalid() {
    assert_eq!(parse_background_size(""), None);
    assert_eq!(parse_background_size("invalid"), None);
    assert_eq!(parse_background_size("-1px"), None);
    assert_eq!(parse_background_size("-50%"), None);
    assert_eq!(parse_background_size("thin"), None);
    assert_eq!(parse_background_size("auto -1px"), None);
    assert_eq!(parse_background_size("100% thin"), None);
}

// ── background-attachment 解析测试 ──

#[test]
fn test_parse_background_attachment_values() {
    assert_eq!(
        parse_background_attachment("scroll"),
        Some(BackgroundAttachmentValue::Scroll)
    );
    assert_eq!(
        parse_background_attachment("fixed"),
        Some(BackgroundAttachmentValue::Fixed)
    );
    assert_eq!(
        parse_background_attachment("local"),
        Some(BackgroundAttachmentValue::Local)
    );
}

#[test]
fn test_parse_background_attachment_case_insensitive() {
    assert_eq!(
        parse_background_attachment("SCROLL"),
        Some(BackgroundAttachmentValue::Scroll)
    );
    assert_eq!(
        parse_background_attachment("Fixed"),
        Some(BackgroundAttachmentValue::Fixed)
    );
    assert_eq!(
        parse_background_attachment("LOCAL"),
        Some(BackgroundAttachmentValue::Local)
    );
}

#[test]
fn test_parse_background_attachment_invalid() {
    assert_eq!(parse_background_attachment(""), None);
    assert_eq!(parse_background_attachment("invalid"), None);
    assert_eq!(parse_background_attachment("scroll fixed"), None);
}

// ── parse_background_clip ──

#[test]
fn test_parse_background_clip_values() {
    assert_eq!(
        parse_background_clip("border-box"),
        Some(BackgroundClipValue::BorderBox)
    );
    assert_eq!(
        parse_background_clip("padding-box"),
        Some(BackgroundClipValue::PaddingBox)
    );
    assert_eq!(
        parse_background_clip("content-box"),
        Some(BackgroundClipValue::ContentBox)
    );
    assert_eq!(parse_background_clip("text"), Some(BackgroundClipValue::Text));
}

#[test]
fn test_parse_background_clip_case_insensitive() {
    assert_eq!(
        parse_background_clip("BORDER-BOX"),
        Some(BackgroundClipValue::BorderBox)
    );
    assert_eq!(
        parse_background_clip("Padding-Box"),
        Some(BackgroundClipValue::PaddingBox)
    );
    assert_eq!(
        parse_background_clip("CONTENT-BOX"),
        Some(BackgroundClipValue::ContentBox)
    );
    assert_eq!(parse_background_clip("TEXT"), Some(BackgroundClipValue::Text));
}

#[test]
fn test_parse_background_clip_invalid() {
    assert_eq!(parse_background_clip(""), None);
    assert_eq!(parse_background_clip("invalid"), None);
    assert_eq!(parse_background_clip("border-box padding-box"), None);
}

// ── parse_background_origin ──

#[test]
fn test_parse_background_origin_values() {
    assert_eq!(
        parse_background_origin("padding-box"),
        Some(BackgroundOriginValue::PaddingBox)
    );
    assert_eq!(
        parse_background_origin("border-box"),
        Some(BackgroundOriginValue::BorderBox)
    );
    assert_eq!(
        parse_background_origin("content-box"),
        Some(BackgroundOriginValue::ContentBox)
    );
}

#[test]
fn test_parse_background_origin_case_insensitive() {
    assert_eq!(
        parse_background_origin("PADDING-BOX"),
        Some(BackgroundOriginValue::PaddingBox)
    );
    assert_eq!(
        parse_background_origin("Border-Box"),
        Some(BackgroundOriginValue::BorderBox)
    );
    assert_eq!(
        parse_background_origin("CONTENT-BOX"),
        Some(BackgroundOriginValue::ContentBox)
    );
}

#[test]
fn test_parse_background_origin_invalid() {
    assert_eq!(parse_background_origin(""), None);
    assert_eq!(parse_background_origin("invalid"), None);
    assert_eq!(parse_background_origin("text"), None);
    assert_eq!(parse_background_origin("padding-box border-box"), None);
}

// ── border-image-source ──

#[test]
fn test_parse_border_image_source_none() {
    let v = parse_border_image_source("none").unwrap();
    assert_eq!(v, BorderImageSourceValue::None);
}

#[test]
fn test_parse_border_image_source_url() {
    let v = parse_border_image_source("url(border.png)").unwrap();
    assert_eq!(v, BorderImageSourceValue::Url("border.png".to_string()));
}

#[test]
fn test_parse_border_image_source_url_quoted() {
    let v = parse_border_image_source("url('border.png')").unwrap();
    assert_eq!(v, BorderImageSourceValue::Url("border.png".to_string()));
    let v = parse_border_image_source("url(\"border image.png\")").unwrap();
    assert_eq!(v, BorderImageSourceValue::Url("border image.png".to_string()));
}

#[test]
fn test_parse_border_image_source_invalid() {
    assert_eq!(parse_border_image_source("invalid"), None);
    assert_eq!(parse_border_image_source("url()"), None);
    assert_eq!(parse_border_image_source(""), None);
    assert_eq!(parse_border_image_source("url(border image.png)"), None);
    assert_eq!(parse_border_image_source("url(\"border.png\" extra)"), None);
    assert_eq!(parse_border_image_source("url(border\".png)"), None);
}

// ── border-image-slice ──

#[test]
fn test_parse_border_image_slice_single_number() {
    let v = parse_border_image_slice("50").unwrap();
    assert_eq!(v.top, BorderImageSliceComponent::Number(50.0));
    assert_eq!(v.right, BorderImageSliceComponent::Number(50.0));
    assert_eq!(v.bottom, BorderImageSliceComponent::Number(50.0));
    assert_eq!(v.left, BorderImageSliceComponent::Number(50.0));
    assert!(!v.fill);
}

#[test]
fn test_parse_border_image_slice_percent() {
    let v = parse_border_image_slice("30%").unwrap();
    assert_eq!(v.top, BorderImageSliceComponent::Percent(30.0));
    assert_eq!(v.right, BorderImageSliceComponent::Percent(30.0));
}

#[test]
fn test_parse_border_image_slice_four_values() {
    let v = parse_border_image_slice("10 20 30 40").unwrap();
    assert_eq!(v.top, BorderImageSliceComponent::Number(10.0));
    assert_eq!(v.right, BorderImageSliceComponent::Number(20.0));
    assert_eq!(v.bottom, BorderImageSliceComponent::Number(30.0));
    assert_eq!(v.left, BorderImageSliceComponent::Number(40.0));
}

#[test]
fn test_parse_border_image_slice_fill() {
    let v = parse_border_image_slice("25 fill").unwrap();
    assert!(v.fill);
    assert_eq!(v.top, BorderImageSliceComponent::Number(25.0));
}

#[test]
fn test_parse_border_image_slice_fill_prefix() {
    let v = parse_border_image_slice("fill 10 20 30 40").unwrap();
    assert!(v.fill);
    assert_eq!(v.top, BorderImageSliceComponent::Number(10.0));
    assert_eq!(v.left, BorderImageSliceComponent::Number(40.0));
}

#[test]
fn test_parse_border_image_slice_invalid() {
    assert_eq!(parse_border_image_slice(""), None);
    assert_eq!(parse_border_image_slice("-5"), None);
    assert_eq!(parse_border_image_slice("inf"), None);
    assert_eq!(parse_border_image_slice("NaN"), None);
    assert_eq!(parse_border_image_slice("inf%"), None);
    assert_eq!(parse_border_image_slice("1 2 3 4 5"), None);
    assert_eq!(parse_border_image_slice("fill fill 10"), None);
}

// ── border-image-width ──

#[test]
fn test_parse_border_image_width_auto() {
    let v = parse_border_image_width("auto").unwrap();
    assert_eq!(v.top, BorderImageWidthComponent::Auto);
    assert_eq!(v.right, BorderImageWidthComponent::Auto);
}

#[test]
fn test_parse_border_image_width_number() {
    let v = parse_border_image_width("3").unwrap();
    assert_eq!(v.top, BorderImageWidthComponent::Number(3.0));
}

#[test]
fn test_parse_border_image_width_px() {
    let v = parse_border_image_width("10px").unwrap();
    assert_eq!(v.top, BorderImageWidthComponent::Length(LengthValue::Px(10.0)));
}

#[test]
fn test_parse_border_image_width_length_units() {
    let v = parse_border_image_width("1vh 2ch").unwrap();
    assert_eq!(v.top, BorderImageWidthComponent::Length(LengthValue::Vh(1.0)));
    assert_eq!(v.right, BorderImageWidthComponent::Length(LengthValue::Ch(2.0)));
}

#[test]
fn test_parse_border_image_width_percent() {
    let v = parse_border_image_width("25%").unwrap();
    assert_eq!(v.top, BorderImageWidthComponent::Percent(25.0));
}

#[test]
fn test_parse_border_image_width_four_values() {
    let v = parse_border_image_width("1 2 3 4").unwrap();
    assert_eq!(v.top, BorderImageWidthComponent::Number(1.0));
    assert_eq!(v.right, BorderImageWidthComponent::Number(2.0));
    assert_eq!(v.bottom, BorderImageWidthComponent::Number(3.0));
    assert_eq!(v.left, BorderImageWidthComponent::Number(4.0));
}

#[test]
fn test_parse_border_image_width_invalid() {
    assert_eq!(parse_border_image_width(""), None);
    assert_eq!(parse_border_image_width("-1"), None);
    assert_eq!(parse_border_image_width("-1px"), None);
    assert_eq!(parse_border_image_width("1px -2em"), None);
    assert_eq!(parse_border_image_width("inf"), None);
    assert_eq!(parse_border_image_width("NaN"), None);
    assert_eq!(parse_border_image_width("inf%"), None);
    assert_eq!(parse_border_image_width("infpx"), None);
    assert_eq!(parse_border_image_width("NaNpx"), None);
    assert_eq!(parse_border_image_width("1 2 3 4 5"), None);
}

// ── border-image-repeat ──

#[test]
fn test_parse_border_image_repeat_stretch() {
    let v = parse_border_image_repeat("stretch").unwrap();
    assert_eq!(v.horizontal, BorderImageRepeatMode::Stretch);
    assert_eq!(v.vertical, BorderImageRepeatMode::Stretch);
}

#[test]
fn test_parse_border_image_repeat_repeat() {
    let v = parse_border_image_repeat("repeat").unwrap();
    assert_eq!(v.horizontal, BorderImageRepeatMode::Repeat);
    assert_eq!(v.vertical, BorderImageRepeatMode::Repeat);
}

#[test]
fn test_parse_border_image_repeat_round() {
    let v = parse_border_image_repeat("round").unwrap();
    assert_eq!(v.horizontal, BorderImageRepeatMode::Round);
    assert_eq!(v.vertical, BorderImageRepeatMode::Round);
}

#[test]
fn test_parse_border_image_repeat_space() {
    let v = parse_border_image_repeat("space").unwrap();
    assert_eq!(v.horizontal, BorderImageRepeatMode::Space);
    assert_eq!(v.vertical, BorderImageRepeatMode::Space);
}

#[test]
fn test_parse_border_image_repeat_two_values() {
    let v = parse_border_image_repeat("repeat round").unwrap();
    assert_eq!(v.horizontal, BorderImageRepeatMode::Repeat);
    assert_eq!(v.vertical, BorderImageRepeatMode::Round);
}

#[test]
fn test_parse_border_image_repeat_invalid() {
    assert_eq!(parse_border_image_repeat(""), None);
    assert_eq!(parse_border_image_repeat("invalid"), None);
    assert_eq!(parse_border_image_repeat("stretch repeat round"), None);
}

// ── border-image-outset ──

#[test]
fn test_parse_border_image_outset_number() {
    let v = parse_border_image_outset("2").unwrap();
    assert_eq!(v.top, BorderImageOutsetComponent::Number(2.0));
    assert_eq!(v.right, BorderImageOutsetComponent::Number(2.0));
}

#[test]
fn test_parse_border_image_outset_px() {
    let v = parse_border_image_outset("10px").unwrap();
    assert_eq!(v.top, BorderImageOutsetComponent::Length(LengthValue::Px(10.0)));
}

#[test]
fn test_parse_border_image_outset_length_units() {
    let v = parse_border_image_outset("1vh 2ch").unwrap();
    assert_eq!(v.top, BorderImageOutsetComponent::Length(LengthValue::Vh(1.0)));
    assert_eq!(v.right, BorderImageOutsetComponent::Length(LengthValue::Ch(2.0)));
}

#[test]
fn test_parse_border_image_outset_four_values() {
    let v = parse_border_image_outset("1px 2 3px 4").unwrap();
    assert_eq!(v.top, BorderImageOutsetComponent::Length(LengthValue::Px(1.0)));
    assert_eq!(v.right, BorderImageOutsetComponent::Number(2.0));
    assert_eq!(v.bottom, BorderImageOutsetComponent::Length(LengthValue::Px(3.0)));
    assert_eq!(v.left, BorderImageOutsetComponent::Number(4.0));
}

#[test]
fn test_parse_border_image_outset_invalid() {
    assert_eq!(parse_border_image_outset(""), None);
    assert_eq!(parse_border_image_outset("-1"), None);
    assert_eq!(parse_border_image_outset("-1px"), None);
    assert_eq!(parse_border_image_outset("1px -2em"), None);
    assert_eq!(parse_border_image_outset("inf"), None);
    assert_eq!(parse_border_image_outset("NaN"), None);
    assert_eq!(parse_border_image_outset("infpx"), None);
    assert_eq!(parse_border_image_outset("NaNpx"), None);
    assert_eq!(parse_border_image_outset("1 2 3 4 5"), None);
}

// ═══════════════════════════════════════════════════════════════════
// 边缘测试补充
// ═══════════════════════════════════════════════════════════════════

/// 测试 parse_border_image_slice 混合百分比与数字值
/// 验证百分比值和纯数字值可以在同一声明中共存
#[test]
fn test_parse_border_image_slice_mixed_percent_and_number() {
    let v = parse_border_image_slice("10% 20 30% 40").unwrap();
    assert_eq!(v.top, BorderImageSliceComponent::Percent(10.0));
    assert_eq!(v.right, BorderImageSliceComponent::Number(20.0));
    assert_eq!(v.bottom, BorderImageSliceComponent::Percent(30.0));
    assert_eq!(v.left, BorderImageSliceComponent::Number(40.0));
    assert!(!v.fill);
}

/// 测试 parse_background_position 两值组合（left center）
/// 验证水平关键字 + 垂直关键字的组合正确解析为 TwoValue
#[test]
fn test_parse_background_position_left_center() {
    let result = parse_background_position("left center").unwrap();
    match result {
        BackgroundPositionValue::TwoValue(h, v) => {
            assert_eq!(*h, BackgroundPositionValue::Left);
            assert_eq!(*v, BackgroundPositionValue::Center);
        }
        _ => panic!("应为 TwoValue，实际得到 {result:?}"),
    }
}

/// 测试 parse_contain 的 "strict" 值
/// 根据 CSS 规范，strict 等价于 "size layout style paint" 四个值的组合
#[test]
fn test_parse_contain_strict() {
    let result = parse_contain("strict").unwrap();
    assert_eq!(result, ContainValue::Strict);

    // 同时验证显式写出 "size layout style paint" 等价的 Custom 标志
    let explicit = parse_contain("size layout style paint").unwrap();
    match explicit {
        ContainValue::Custom(flags) => {
            let strict_flags = ContainValue::FLAG_SIZE
                | ContainValue::FLAG_LAYOUT
                | ContainValue::FLAG_STYLE
                | ContainValue::FLAG_PAINT;
            assert_eq!(flags, strict_flags);
        }
        _ => panic!("应为 Custom，实际得到 {explicit:?}"),
    }
}

/// 测试 parse_filter 解析多个 filter 函数
/// parse_filter 每次解析单个函数，此处验证 blur、brightness、sepia 三种函数均能正确解析
#[test]
fn test_parse_filter_multiple_functions() {
    // blur(5px)
    assert_eq!(parse_filter("blur(5px)"), Some(FilterValue::Blur(5.0)));
    // brightness(1.5)
    assert_eq!(parse_filter("brightness(1.5)"), Some(FilterValue::Brightness(1.5)));
    // sepia(80%)
    assert_eq!(parse_filter("sepia(80%)"), Some(FilterValue::Sepia(0.8)));
    // hue-rotate(90deg)
    assert_eq!(parse_filter("hue-rotate(90deg)"), Some(FilterValue::HueRotate(90.0)));
}

#[test]
fn test_parse_filter_accepts_length_units() {
    assert_eq!(parse_filter("blur(1vh)"), Some(FilterValue::Blur(1.0)));
    assert_eq!(parse_filter("blur(2ch)"), Some(FilterValue::Blur(2.0)));
    assert!(parse_filter("blur(1%)").is_none());
    assert!(parse_filter("blur(thin)").is_none());

    match parse_filter("drop-shadow(1vh 2ch 3vmin red)") {
        Some(FilterValue::DropShadow(x, y, blur, color)) => {
            assert_eq!(x, 1.0);
            assert_eq!(y, 2.0);
            assert_eq!(blur, 3.0);
            assert_eq!(color, ColorValue::Rgba(255, 0, 0, 255));
        }
        other => panic!("应为 DropShadow，实际得到 {other:?}"),
    }
    assert!(parse_filter("drop-shadow(1vh 2ch -3vmin red)").is_none());
}

/// 测试 parse_text_shadow 仅有颜色无模糊半径的情况
/// 格式 "2px 3px red"：offset-x + offset-y + color，blur 默认为 0
#[test]
fn test_parse_text_shadow_color_only_no_blur() {
    let result = parse_text_shadow("2px 3px red").unwrap();
    assert_eq!(result.offset_x, LengthValue::Px(2.0));
    assert_eq!(result.offset_y, LengthValue::Px(3.0));
    assert_eq!(result.blur_radius, LengthValue::Px(0.0));
    assert_eq!(result.color, ColorValue::Rgba(255, 0, 0, 255));
}

/// 测试 text-shadow / box-shadow 省略颜色时默认 currentColor
/// （CSS Backgrounds §7.1 `<shadow>`：省略颜色 = currentColor；
///  CSS Text Decoration §3：text-shadow 同语义，省略颜色取元素 `color`）。
/// driving: R2364 — 此前默认黑色，致 `color: red; text-shadow: 2px 2px` 渲染黑阴影而非红。
#[test]
fn test_parse_shadow_omitted_color_defaults_to_currentcolor() {
    // text-shadow 无颜色
    let ts = parse_text_shadow("2px 3px").unwrap();
    assert_eq!(
        ts.color,
        ColorValue::CurrentColor,
        "text-shadow 省略颜色应默认 currentColor"
    );
    // text-shadow 有 blur 无颜色
    let ts_blur = parse_text_shadow("2px 3px 4px").unwrap();
    assert_eq!(ts_blur.color, ColorValue::CurrentColor);
    // box-shadow 无颜色
    let bs = parse_box_shadow("2px 3px").unwrap();
    assert_eq!(
        bs.color,
        ColorValue::CurrentColor,
        "box-shadow 省略颜色应默认 currentColor"
    );
    // box-shadow 有 blur/spread 无颜色
    let bs_full = parse_box_shadow("2px 3px 4px 5px").unwrap();
    assert_eq!(bs_full.color, ColorValue::CurrentColor);
    // 显式颜色仍保留（不被默认覆盖）
    assert_eq!(
        parse_text_shadow("2px 3px red").unwrap().color,
        ColorValue::Rgba(255, 0, 0, 255)
    );
    assert_eq!(
        parse_box_shadow("2px 3px red").unwrap().color,
        ColorValue::Rgba(255, 0, 0, 255)
    );
}

// ── list-style-image ──

#[test]
fn test_parse_list_style_image_none() {
    assert_eq!(parse_list_style_image("none"), Some(ListStyleImageValue::None));
}

#[test]
fn test_parse_list_style_image_url() {
    let v = parse_list_style_image("url(marker.png)").unwrap();
    assert_eq!(v, ListStyleImageValue::Url("marker.png".to_string()));
}

#[test]
fn test_parse_list_style_image_quoted() {
    let v = parse_list_style_image("url('star.gif')").unwrap();
    assert_eq!(v, ListStyleImageValue::Url("star.gif".to_string()));
    let v = parse_list_style_image("url(\"my marker.png\")").unwrap();
    assert_eq!(v, ListStyleImageValue::Url("my marker.png".to_string()));
}

#[test]
fn test_parse_list_style_image_invalid() {
    assert_eq!(parse_list_style_image("invalid"), None);
    assert_eq!(parse_list_style_image(""), None);
    assert_eq!(parse_list_style_image("url()"), None);
    assert_eq!(parse_list_style_image("url(my marker.png)"), None);
    assert_eq!(parse_list_style_image("url(\"marker.png\" extra)"), None);
    assert_eq!(parse_list_style_image("url(marker\".png)"), None);
}

// ═══════════════════════════════════════════════════════════════════
// 边缘测试补充（round 21）
// ═══════════════════════════════════════════════════════════════════

/// 测试 parse_list_style_image 双引号 URL
/// 验证 url("bullet.png") 格式能正确剥离双引号并提取 URL
#[test]
fn test_parse_list_style_image_double_quoted_url() {
    let v = parse_list_style_image("url(\"bullet.png\")").unwrap();
    assert_eq!(v, ListStyleImageValue::Url("bullet.png".to_string()));
}

/// 测试 parse_border_image_source 大写 "NONE"（大小写不敏感）
/// CSS 属性值关键字应忽略大小写，NONE / none / None 均应解析为 None
#[test]
fn test_parse_border_image_source_case_insensitive_none() {
    assert_eq!(parse_border_image_source("NONE"), Some(BorderImageSourceValue::None));
    assert_eq!(parse_border_image_source("None"), Some(BorderImageSourceValue::None));
    assert_eq!(parse_border_image_source("NoNe"), Some(BorderImageSourceValue::None));
}

/// 测试 parse_background_repeat 全部 6 个枚举值
/// 逐一验证 repeat / repeat-x / repeat-y / no-repeat / space / round 都能正确解析
#[test]
fn test_parse_background_repeat_all_values() {
    assert_eq!(parse_background_repeat("repeat"), Some(BackgroundRepeatValue::Repeat));
    assert_eq!(
        parse_background_repeat("repeat-x"),
        Some(BackgroundRepeatValue::RepeatX)
    );
    assert_eq!(
        parse_background_repeat("repeat-y"),
        Some(BackgroundRepeatValue::RepeatY)
    );
    assert_eq!(
        parse_background_repeat("no-repeat"),
        Some(BackgroundRepeatValue::NoRepeat)
    );
    assert_eq!(parse_background_repeat("space"), Some(BackgroundRepeatValue::Space));
    assert_eq!(parse_background_repeat("round"), Some(BackgroundRepeatValue::Round));
}

/// 测试 parse_filter 的 drop-shadow 函数
/// 验证 drop-shadow(2px 3px 4px red) 能正确解析为 DropShadow 变体，
/// 其中包含 x/y/blur 偏移和命名颜色。
/// R2485：CSS `<length>` 须带单位（px）；改前裸 f32::parse 仅接受 unitless 数字（非法 CSS）。
#[test]
fn test_parse_filter_drop_shadow() {
    let result = parse_filter("drop-shadow(2px 3px 4px red)");
    match result {
        Some(FilterValue::DropShadow(x, y, blur, color)) => {
            assert_eq!(x, 2.0);
            assert_eq!(y, 3.0);
            assert_eq!(blur, 4.0);
            assert_eq!(color, ColorValue::Rgba(255, 0, 0, 255));
        }
        _ => panic!("应为 DropShadow，实际得到 {result:?}"),
    }
}

/// 测试 parse_box_shadow 颜色在开头时的拒绝行为
/// 输入 "red 5px 10px" 将颜色放在首位。R2477：CSS Backgrounds §7.1
/// `<inset>? && <length>{2,4} && <color>?` 的 `&&` 允许颜色任意位置 → 合法，
/// 解析为 ox=5 oy=10 color=red（改前按固定下标 parts[0]=length 致整条丢）。
#[test]
fn test_parse_box_shadow_color_at_start() {
    let s = parse_box_shadow("red 5px 10px").expect("color-first 合法应解析");
    assert!(matches!(s.color, ColorValue::Rgba(255, 0, 0, _)));
    assert_eq!(s.offset_x, LengthValue::Px(5.0));
    assert_eq!(s.offset_y, LengthValue::Px(10.0));
}

// ── empty-cells ──

#[test]
fn test_parse_empty_cells_show() {
    assert_eq!(parse_empty_cells("show"), Some(EmptyCellsValue::Show));
}

#[test]
fn test_parse_empty_cells_hide() {
    assert_eq!(parse_empty_cells("hide"), Some(EmptyCellsValue::Hide));
}

#[test]
fn test_parse_empty_cells_case_insensitive() {
    assert_eq!(parse_empty_cells("SHOW"), Some(EmptyCellsValue::Show));
    assert_eq!(parse_empty_cells("Hide"), Some(EmptyCellsValue::Hide));
}

#[test]
fn test_parse_empty_cells_invalid() {
    assert_eq!(parse_empty_cells("invalid"), None);
    assert_eq!(parse_empty_cells(""), None);
}

// ── border-spacing ──

#[test]
fn test_parse_border_spacing_single_value() {
    let v = parse_border_spacing("2px").unwrap();
    assert_eq!(v.horizontal, LengthValue::Px(2.0));
    assert_eq!(v.vertical, LengthValue::Px(2.0));
}

#[test]
fn test_parse_border_spacing_two_values() {
    let v = parse_border_spacing("2px 4px").unwrap();
    assert_eq!(v.horizontal, LengthValue::Px(2.0));
    assert_eq!(v.vertical, LengthValue::Px(4.0));
}

#[test]
fn test_parse_border_spacing_em() {
    let v = parse_border_spacing("1em").unwrap();
    assert_eq!(v.horizontal, LengthValue::Em(1.0));
}

#[test]
fn test_parse_border_spacing_invalid() {
    assert_eq!(parse_border_spacing(""), None);
    assert_eq!(parse_border_spacing("invalid"), None);
    assert_eq!(parse_border_spacing("1px 2px 3px"), None);
    assert_eq!(parse_border_spacing("10%"), None);
    assert_eq!(parse_border_spacing("thin"), None);
    assert_eq!(parse_border_spacing("auto"), None);
    assert_eq!(parse_border_spacing("min-content"), None);
    assert_eq!(parse_border_spacing("infpx"), None);
    assert_eq!(parse_border_spacing("NaNpx"), None);
}

// ═══════════════════════════════════════════════════════════════════
// 边缘测试补充（round 22）
// ═══════════════════════════════════════════════════════════════════

/// 测试 parse_empty_cells 两个有效值的全面断言
/// 同时验证 show 和 hide 都能正确解析，包含前后空白
#[test]
fn test_parse_empty_cells_both_values_with_whitespace() {
    assert_eq!(parse_empty_cells("  show  "), Some(EmptyCellsValue::Show));
    assert_eq!(parse_empty_cells("\thide\t"), Some(EmptyCellsValue::Hide));
}

/// 测试 parse_border_spacing 单值（rem 单位）
/// 单值时 vertical 应等于 horizontal，验证 rem 单位正确解析
#[test]
fn test_parse_border_spacing_single_rem() {
    let v = parse_border_spacing("0.5rem").unwrap();
    assert_eq!(v.horizontal, LengthValue::Rem(0.5));
    assert_eq!(v.vertical, LengthValue::Rem(0.5));
}

/// 测试 parse_border_spacing 双值（混合单位 em + px）
/// 水平和垂直可以使用不同单位
#[test]
fn test_parse_border_spacing_mixed_units() {
    let v = parse_border_spacing("1em 8px").unwrap();
    assert_eq!(v.horizontal, LengthValue::Em(1.0));
    assert_eq!(v.vertical, LengthValue::Px(8.0));
}

/// 测试 parse_border_spacing 负值
/// CSS 规范要求 border-spacing 不接受负值。
#[test]
fn test_parse_border_spacing_negative_value() {
    assert_eq!(parse_border_spacing("-2px"), None);
    assert_eq!(parse_border_spacing("2px -4px"), None);
}

/// 测试 parse_list_style_image URL 中包含空格
/// URL 未加引号但含有空格时不是合法 url-token；quoted string payload 保留空格。
#[test]
fn test_parse_list_style_image_url_with_spaces() {
    assert_eq!(parse_list_style_image("url(my image.png)"), None);
    let v = parse_list_style_image("url(\"my image.png\")").unwrap();
    assert_eq!(v, ListStyleImageValue::Url("my image.png".to_string()));
}

// ── counter-set 解析测试 ──

#[test]
fn test_parse_counter_set_none() {
    assert_eq!(parse_counter_set("none"), Some(CounterSetValue::None));
}

#[test]
fn test_parse_counter_set_name_value() {
    let v = parse_counter_set("section 5").unwrap();
    match v {
        CounterSetValue::Actions(actions) => {
            assert_eq!(actions.len(), 1);
            assert_eq!(actions[0].name, "section");
            assert_eq!(actions[0].value, Some(5));
        }
        _ => panic!("expected Actions"),
    }
}

#[test]
fn test_parse_counter_set_invalid() {
    assert_eq!(parse_counter_set(""), None);
}
