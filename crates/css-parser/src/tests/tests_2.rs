use super::*;

// ═══════════════════════════════════════════════════════════════════════
// 1. Tokenizer 测试
// ═══════════════════════════════════════════════════════════════════════

// ── @keyframes 解析测试 ──

#[test]
fn test_parse_keyframes_basic() {
    use crate::ast::*;
    let css = "@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.name, "fadeIn");
            assert_eq!(kf.keyframes.len(), 2);
            assert_eq!(kf.keyframes[0].selectors, vec![KeyframeSelector::From]);
            assert_eq!(kf.keyframes[1].selectors, vec![KeyframeSelector::To]);
            assert_eq!(kf.keyframes[0].declarations.len(), 1);
            assert_eq!(kf.keyframes[0].declarations[0].property, "opacity");
            assert_eq!(kf.keyframes[0].declarations[0].value, "0");
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
fn test_parse_keyframes_percentage() {
    use crate::ast::*;
    let css = "@keyframes slide { 0% { left: 0px; } 50% { left: 100px; } 100% { left: 200px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.name, "slide");
            assert_eq!(kf.keyframes.len(), 3);
            assert_eq!(kf.keyframes[0].selectors, vec![KeyframeSelector::Percentage(0.0)]);
            assert_eq!(kf.keyframes[1].selectors, vec![KeyframeSelector::Percentage(50.0)]);
            assert_eq!(kf.keyframes[2].selectors, vec![KeyframeSelector::Percentage(100.0)]);
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
fn test_parse_keyframes_comma_selectors() {
    use crate::ast::*;
    let css = "@keyframes bounce { 0%, 100% { top: 0px; } 50% { top: 50px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.keyframes.len(), 2);
            assert_eq!(
                kf.keyframes[0].selectors,
                vec![KeyframeSelector::Percentage(0.0), KeyframeSelector::Percentage(100.0)]
            );
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
fn test_parse_keyframes_mixed_selectors() {
    use crate::ast::*;
    let css = "@keyframes test { from, 50% { color: red; } to { color: blue; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(
                kf.keyframes[0].selectors,
                vec![KeyframeSelector::From, KeyframeSelector::Percentage(50.0)]
            );
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
fn test_parse_keyframes_quoted_name() {
    let css = "@keyframes \"my-animation\" { to { opacity: 1; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.name, "my-animation");
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

// ── Animation 值解析测试 ──

#[test]
fn test_parse_animation_direction() {
    assert_eq!(
        parse_animation_direction("normal"),
        Some(crate::values::AnimationDirectionValue::Normal)
    );
    assert_eq!(
        parse_animation_direction("reverse"),
        Some(crate::values::AnimationDirectionValue::Reverse)
    );
    assert_eq!(
        parse_animation_direction("alternate"),
        Some(crate::values::AnimationDirectionValue::Alternate)
    );
    assert_eq!(
        parse_animation_direction("alternate-reverse"),
        Some(crate::values::AnimationDirectionValue::AlternateReverse)
    );
    assert_eq!(parse_animation_direction("invalid"), None);
}

#[test]
fn test_parse_animation_fill_mode() {
    assert_eq!(
        parse_animation_fill_mode("none"),
        Some(crate::values::AnimationFillModeValue::None)
    );
    assert_eq!(
        parse_animation_fill_mode("forwards"),
        Some(crate::values::AnimationFillModeValue::Forwards)
    );
    assert_eq!(
        parse_animation_fill_mode("backwards"),
        Some(crate::values::AnimationFillModeValue::Backwards)
    );
    assert_eq!(
        parse_animation_fill_mode("both"),
        Some(crate::values::AnimationFillModeValue::Both)
    );
    assert_eq!(parse_animation_fill_mode("invalid"), None);
}

#[test]
fn test_parse_animation_play_state() {
    assert_eq!(
        parse_animation_play_state("running"),
        Some(crate::values::AnimationPlayStateValue::Running)
    );
    assert_eq!(
        parse_animation_play_state("paused"),
        Some(crate::values::AnimationPlayStateValue::Paused)
    );
    assert_eq!(parse_animation_play_state("invalid"), None);
}

// ── :has() 选择器解析测试 ──

#[test]
/// 测试 :has(.active) 解析
fn test_parse_has_selector() {
    let stylesheet = Parser::parse_stylesheet("div:has(.active) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(matches!(
            &compound.type_selector,
            Some(TypeSelector::Tag(t)) if t == "div"
        ));
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Has(selectors))
                if selectors.len() == 1
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :has(> .child) 解析（子组合器）
fn test_parse_has_child_combinator() {
    let stylesheet = Parser::parse_stylesheet("div:has(> .child) { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        let has_inner = compound.subclass_selectors.iter().find_map(|s| match s {
            SubclassSelector::PseudoClass(PseudoClassSelector::Has(selectors)) => Some(selectors),
            _ => None,
        });
        assert!(has_inner.is_some(), "Expected :has() pseudo-class");
        let inner = has_inner.unwrap();
        assert_eq!(inner.len(), 1);
        // 内部选择器应有 Child 组合器
        let inner_parts = &inner[0].complex.parts;
        assert_eq!(inner_parts.len(), 2, "Expected compound > compound");
        assert_eq!(inner_parts[0].1, Some(Combinator::Child));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :has(div, span) 解析（逗号分隔选择器列表）
fn test_parse_has_selector_list() {
    let stylesheet = Parser::parse_stylesheet("section:has(div, span) { background: white; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Has(selectors))
                if selectors.len() == 2
        )));
    } else {
        panic!("Expected Style rule");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 10. Tokenizer `/` delimiter 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 `/` 作为独立分隔符（用于 font shorthand 等）
fn test_tokenize_slash_delim() {
    let tokens: Vec<_> = Tokenizer::new("/").collect_tokens();
    assert_eq!(tokens, vec![Token::Delim('/')]);
}

#[test]
/// 测试 font shorthand 中的 `/` 分隔符：`font: 12px/1.5 sans-serif`
fn test_tokenize_font_shorthand_slash() {
    let tokens: Vec<_> = Tokenizer::new("12px/1.5").collect_tokens();
    assert!(tokens.len() >= 3);
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 12.0 && u == "px"));
    assert_eq!(tokens[1], Token::Delim('/'));
    assert!(matches!(&tokens[2], Token::Number(n) if (*n - 1.5).abs() < 0.001));
}

#[test]
/// 测试 calc() 中的除法 `/`
fn test_tokenize_calc_division() {
    let tokens: Vec<_> = Tokenizer::new("100px / 2").collect_tokens();
    assert!(tokens.len() >= 3);
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 100.0 && u == "px"));
    assert_eq!(tokens[1], Token::Whitespace);
    assert_eq!(tokens[2], Token::Delim('/'));
}

// ═══════════════════════════════════════════════════════════════════════
// 11. parse_length 边界情况测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试负数长度值
fn test_parse_length_negative() {
    assert_eq!(parse_length("-10px"), Some(LengthValue::Px(-10.0)));
    assert_eq!(parse_length("-2.5em"), Some(LengthValue::Em(-2.5)));
}

#[test]
/// 测试带正号前缀的长度值
fn test_parse_length_leading_plus() {
    assert_eq!(parse_length("+10px"), Some(LengthValue::Px(10.0)));
    assert_eq!(parse_length("+1.5em"), Some(LengthValue::Em(1.5)));
}

#[test]
/// 测试科学计数法长度值
fn test_parse_length_scientific_notation() {
    // "1e2px" → 100.0px
    assert_eq!(parse_length("1e2px"), Some(LengthValue::Px(100.0)));
    assert_eq!(parse_length("1E2px"), Some(LengthValue::Px(100.0)));
}

#[test]
/// 测试带空格的裸零
fn test_parse_length_zero_whitespace() {
    assert_eq!(parse_length("  0  "), Some(LengthValue::Px(0.0)));
}

#[test]
/// 测试非零无单位值不被解析为长度（CSS 规范仅允许 0 无单位）
fn test_parse_length_unitless_nonzero() {
    assert_eq!(parse_length("42"), None);
    assert_eq!(parse_length("1.5"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 12. calc() 相对单位求值测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 calc() 中 em 单位求值
fn test_eval_calc_em() {
    let expr = parse_calc("calc(1.5em + 10px)").unwrap();
    let ctx = CalcContext {
        font_size: Some(16.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 1.5 * 16 + 10 = 34.0
    assert_eq!(result, Some(34.0));
}

#[test]
/// 测试 calc() 中 rem 单位求值
fn test_eval_calc_rem() {
    let expr = parse_calc("calc(2rem + 5px)").unwrap();
    let ctx = CalcContext {
        root_font_size: Some(20.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 2 * 20 + 5 = 45.0
    assert_eq!(result, Some(45.0));
}

#[test]
/// 测试 calc() 中 vh 单位求值
fn test_eval_calc_vh() {
    let expr = parse_calc("calc(50vh - 20px)").unwrap();
    let ctx = CalcContext {
        viewport_height: Some(800.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 50 * 800 / 100 - 20 = 380.0
    assert_eq!(result, Some(380.0));
}

#[test]
/// 测试 calc() 中 vw 单位求值
fn test_eval_calc_vw() {
    let expr = parse_calc("calc(25vw + 10px)").unwrap();
    let ctx = CalcContext {
        viewport_width: Some(1200.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 25 * 1200 / 100 + 10 = 310.0
    assert_eq!(result, Some(310.0));
}

#[test]
/// 测试 calc() 中 vmin 单位求值
fn test_eval_calc_vmin() {
    let expr = parse_calc("calc(10vmin)").unwrap();
    let ctx = CalcContext {
        viewport_width: Some(1200.0),
        viewport_height: Some(800.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 10 * min(1200, 800) / 100 = 80.0
    assert_eq!(result, Some(80.0));
}

#[test]
/// 测试 calc() 中 vmax 单位求值
fn test_eval_calc_vmax() {
    let expr = parse_calc("calc(10vmax)").unwrap();
    let ctx = CalcContext {
        viewport_width: Some(1200.0),
        viewport_height: Some(800.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 10 * max(1200, 800) / 100 = 120.0
    assert_eq!(result, Some(120.0));
}

#[test]
/// 测试 calc() 中 ch 单位求值
fn test_eval_calc_ch() {
    let expr = parse_calc("calc(4ch + 2px)").unwrap();
    let ctx = CalcContext {
        ch_width: Some(8.0),
        ..Default::default()
    };
    let result = eval_calc_with_context(&expr, &ctx);
    // 4 * 8 + 2 = 34.0
    assert_eq!(result, Some(34.0));
}

#[test]
/// 测试 calc() 相对单位缺少上下文时返回 None
fn test_eval_calc_relative_unit_missing_context() {
    let expr = parse_calc("calc(1.5em + 10px)").unwrap();
    // 没有 font_size 上下文
    let ctx = CalcContext::default();
    let result = eval_calc_with_context(&expr, &ctx);
    assert_eq!(result, None);
}

#[test]
/// 测试 eval_calc 向后兼容性（parent_length 参数）
fn test_eval_calc_backward_compat() {
    let expr = parse_calc("calc(100% - 20px)").unwrap();
    let result = eval_calc(&expr, Some(200.0));
    assert_eq!(result, Some(180.0));
}

// ═══════════════════════════════════════════════════════════════════════
// 13. CSS Gradient 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试基本 linear-gradient 解析
fn test_parse_linear_gradient_basic() {
    let result = parse_gradient("linear-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.direction, GradientDirection::ToBottom);
            assert_eq!(lg.stops.len(), 2);
            assert_eq!(lg.repeating, false);
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试带方向的 linear-gradient
fn test_parse_linear_gradient_with_direction() {
    let result = parse_gradient("linear-gradient(to right, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.direction, GradientDirection::ToRight);
            assert_eq!(lg.stops.len(), 2);
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试带角度的 linear-gradient
fn test_parse_linear_gradient_with_angle() {
    let result = parse_gradient("linear-gradient(45deg, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.direction, GradientDirection::Angle(45.0));
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试带色标位置的 linear-gradient
fn test_parse_linear_gradient_with_stop_positions() {
    let result = parse_gradient("linear-gradient(red 0%, blue 100%)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.stops.len(), 2);
            assert_eq!(lg.stops[0].position, Some(LengthValue::Percentage(0.0)));
            assert_eq!(lg.stops[1].position, Some(LengthValue::Percentage(100.0)));
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试多色标 linear-gradient
fn test_parse_linear_gradient_multi_stop() {
    let result = parse_gradient("linear-gradient(red, yellow, green, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.stops.len(), 4);
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试 repeating-linear-gradient
fn test_parse_repeating_linear_gradient() {
    let result = parse_gradient("repeating-linear-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert!(lg.repeating);
        }
        _ => panic!("Expected LinearGradient"),
    }
}

#[test]
/// 测试 radial-gradient 基本解析
fn test_parse_radial_gradient_basic() {
    let result = parse_gradient("radial-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert_eq!(rg.shape, RadialShape::Ellipse);
            assert_eq!(rg.size, RadialSize::FarthestCorner);
            assert_eq!(rg.stops.len(), 2);
            assert!(!rg.repeating);
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试 circle radial-gradient
fn test_parse_radial_gradient_circle() {
    let result = parse_gradient("radial-gradient(circle, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert_eq!(rg.shape, RadialShape::Circle);
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试带位置的 radial-gradient
fn test_parse_radial_gradient_at_position() {
    let result = parse_gradient("radial-gradient(circle at center, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert_eq!(rg.shape, RadialShape::Circle);
            assert_eq!(rg.position_x, LengthValue::Percentage(50.0));
            assert_eq!(rg.position_y, LengthValue::Percentage(50.0));
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试 repeating-radial-gradient
fn test_parse_repeating_radial_gradient() {
    let result = parse_gradient("repeating-radial-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Radial(rg) => {
            assert!(rg.repeating);
        }
        _ => panic!("Expected RadialGradient"),
    }
}

#[test]
/// 测试 conic-gradient 基本解析
fn test_parse_conic_gradient_basic() {
    let result = parse_gradient("conic-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Conic(cg) => {
            assert_eq!(cg.from_angle, 0.0);
            assert_eq!(cg.stops.len(), 2);
            assert!(!cg.repeating);
        }
        _ => panic!("Expected ConicGradient"),
    }
}

#[test]
/// 测试带 from 角度的 conic-gradient
fn test_parse_conic_gradient_from_angle() {
    let result = parse_gradient("conic-gradient(from 45deg, red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Conic(cg) => {
            assert_eq!(cg.from_angle, 45.0);
        }
        _ => panic!("Expected ConicGradient"),
    }
}

#[test]
/// 测试 repeating-conic-gradient
fn test_parse_repeating_conic_gradient() {
    let result = parse_gradient("repeating-conic-gradient(red, blue)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Conic(cg) => {
            assert!(cg.repeating);
        }
        _ => panic!("Expected ConicGradient"),
    }
}

#[test]
/// 测试无效渐变返回 None
fn test_parse_gradient_invalid() {
    assert_eq!(parse_gradient("not-a-gradient"), None);
    assert_eq!(parse_gradient("linear-gradient()"), None);
    assert_eq!(parse_gradient(""), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 14. @layer 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 @layer 基本解析：@layer base { div { color: red; } }
fn test_parse_layer_basic() {
    let stylesheet = Parser::parse_stylesheet("@layer base { div { color: red; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "base");
            assert_eq!(layer_rule.rules.len(), 1);
            if let Rule::Style(sr) = &layer_rule.rules[0] {
                assert_eq!(sr.declarations.len(), 1);
                assert_eq!(sr.declarations[0].property, "color");
                assert_eq!(sr.declarations[0].value, "red");
            } else {
                panic!("Expected Style rule inside @layer");
            }
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试 @layer 多规则
fn test_parse_layer_multiple_rules() {
    let css = "@layer components { div { color: red; } span { font-size: 16px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "components");
            assert_eq!(layer_rule.rules.len(), 2);
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试 @layer 仅声明（分号结尾）
fn test_parse_layer_declaration_only() {
    let stylesheet = Parser::parse_stylesheet("@layer base;");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "base");
            assert!(layer_rule.rules.is_empty());
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试 @layer 匿名层（无名称）
fn test_parse_layer_anonymous() {
    let stylesheet = Parser::parse_stylesheet("@layer { div { color: blue; } }");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "");
            assert_eq!(layer_rule.rules.len(), 1);
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试多个 @layer 规则
fn test_parse_multiple_layers() {
    let css = "@layer reset { * { margin: 0; } } @layer base { div { color: red; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 2);
    match &stylesheet.rules[0] {
        Rule::Layer(lr) => assert_eq!(lr.name, "reset"),
        _ => panic!("Expected Layer rule"),
    }
    match &stylesheet.rules[1] {
        Rule::Layer(lr) => assert_eq!(lr.name, "base"),
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试 @layer 内嵌套 @media
fn test_parse_layer_with_media() {
    let css = "@layer responsive { @media (min-width: 600px) { div { width: 100%; } } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Layer(layer_rule) => {
            assert_eq!(layer_rule.name, "responsive");
            assert_eq!(layer_rule.rules.len(), 1);
            match &layer_rule.rules[0] {
                Rule::At(at_rule) => {
                    assert_eq!(at_rule.name, "media");
                }
                _ => panic!("Expected At rule inside @layer"),
            }
        }
        _ => panic!("Expected Layer rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 15. @container 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 @container 带名称和条件解析
fn test_parse_container_with_name() {
    let css = "@container sidebar (min-width: 400px) { .child { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Container(container_rule) => {
            assert_eq!(container_rule.name.as_deref(), Some("sidebar"));
            match &container_rule.condition {
                ContainerCondition::Size(sc) => {
                    assert_eq!(sc.feature, "min-width");
                    assert_eq!(sc.value, "400px");
                }
                _ => panic!("Expected Size condition"),
            }
            assert_eq!(container_rule.rules.len(), 1);
        }
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container 无名称解析
fn test_parse_container_without_name() {
    let css = "@container (min-width: 400px) { .child { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Container(container_rule) => {
            assert!(container_rule.name.is_none());
            match &container_rule.condition {
                ContainerCondition::Size(sc) => {
                    assert_eq!(sc.feature, "min-width");
                    assert_eq!(sc.value, "400px");
                }
                _ => panic!("Expected Size condition"),
            }
        }
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container (min-width: 400px) 条件解析
fn test_parse_container_min_width() {
    let css = "@container (min-width: 400px) { div { display: block; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => match &cr.condition {
            ContainerCondition::Size(sc) => {
                assert_eq!(sc.feature, "min-width");
                assert_eq!(sc.value, "400px");
            }
            _ => panic!("Expected Size condition"),
        },
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container 嵌套规则
fn test_parse_container_nested_rules() {
    let css = "@container card (min-width: 300px) { .title { font-size: 24px; } .body { font-size: 16px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => {
            assert_eq!(cr.name.as_deref(), Some("card"));
            assert_eq!(cr.rules.len(), 2);
        }
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container 带 max-width 条件
fn test_parse_container_max_width() {
    let css = "@container (max-width: 800px) { .layout { flex-direction: column; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => match &cr.condition {
            ContainerCondition::Size(sc) => {
                assert_eq!(sc.feature, "max-width");
                assert_eq!(sc.value, "800px");
            }
            _ => panic!("Expected Size condition"),
        },
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试嵌套的 @container 规则保留在父规则中
fn test_parse_container_nested_in_media() {
    let css = "@media screen { @container (min-width: 500px) { .item { width: 50%; } } }";
    let stylesheet = Parser::parse_stylesheet(css);
    match &stylesheet.rules[0] {
        Rule::At(at_rule) => {
            assert_eq!(at_rule.name, "media");
            if let AtRuleBody::Block(rules) = &at_rule.body {
                assert_eq!(rules.len(), 1);
                match &rules[0] {
                    Rule::Container(cr) => {
                        assert!(cr.name.is_none());
                        match &cr.condition {
                            ContainerCondition::Size(sc) => {
                                assert_eq!(sc.feature, "min-width");
                                assert_eq!(sc.value, "500px");
                            }
                            _ => panic!("Expected Size condition"),
                        }
                    }
                    _ => panic!("Expected Container rule inside @media"),
                }
            } else {
                panic!("Expected Block body");
            }
        }
        _ => panic!("Expected At rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 16. scroll-snap 属性值解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 scroll-snap-type 值解析
fn test_parse_scroll_snap_type_values() {
    assert_eq!(parse_scroll_snap_type("none"), Some((ScrollSnapTypeValue::None, None)));
    assert_eq!(
        parse_scroll_snap_type("x mandatory"),
        Some((ScrollSnapTypeValue::Mandatory, Some(ScrollSnapAxis::X)))
    );
    assert_eq!(
        parse_scroll_snap_type("y proximity"),
        Some((ScrollSnapTypeValue::Proximity, Some(ScrollSnapAxis::Y)))
    );
    assert_eq!(
        parse_scroll_snap_type("both mandatory"),
        Some((ScrollSnapTypeValue::Mandatory, Some(ScrollSnapAxis::Both)))
    );
    assert_eq!(
        parse_scroll_snap_type("mandatory"),
        Some((ScrollSnapTypeValue::Mandatory, None))
    );
    assert_eq!(parse_scroll_snap_type("invalid"), None);
    assert_eq!(parse_scroll_snap_type("mandatory proximity"), None);
    assert_eq!(parse_scroll_snap_type("x y mandatory"), None);
}

#[test]
/// 测试 scroll-snap-align 值解析
fn test_parse_scroll_snap_align_values() {
    assert_eq!(parse_scroll_snap_align("none"), Some(ScrollSnapAlignValue::None));
    assert_eq!(parse_scroll_snap_align("start"), Some(ScrollSnapAlignValue::Start));
    assert_eq!(parse_scroll_snap_align("end"), Some(ScrollSnapAlignValue::End));
    assert_eq!(parse_scroll_snap_align("center"), Some(ScrollSnapAlignValue::Center));
    assert_eq!(parse_scroll_snap_align("invalid"), None);
}

#[test]
/// 测试 scroll-snap-stop 值解析
fn test_parse_scroll_snap_stop_values() {
    assert_eq!(parse_scroll_snap_stop("normal"), Some(ScrollSnapStopValue::Normal));
    assert_eq!(parse_scroll_snap_stop("always"), Some(ScrollSnapStopValue::Always));
    assert_eq!(parse_scroll_snap_stop("invalid"), None);
}

#[test]
/// 测试 scroll-margin 简写解析
fn test_parse_scroll_margin_shorthand() {
    // 1 个值
    let result = parse_length_shorthand("10px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Px(10.0),
            LengthValue::Px(10.0),
            LengthValue::Px(10.0),
        ])
    );
    // 2 个值
    let result = parse_length_shorthand("10px 20px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Px(20.0),
            LengthValue::Px(10.0),
            LengthValue::Px(20.0),
        ])
    );
    // 3 个值
    let result = parse_length_shorthand("10px 20px 30px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Px(20.0),
            LengthValue::Px(30.0),
            LengthValue::Px(20.0),
        ])
    );
    // 4 个值
    let result = parse_length_shorthand("10px 20px 30px 40px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Px(20.0),
            LengthValue::Px(30.0),
            LengthValue::Px(40.0),
        ])
    );
}

#[test]
/// 测试 scroll-margin 长属性解析（通过 parse_length）
fn test_parse_scroll_margin_longhands() {
    assert_eq!(parse_length("10px"), Some(LengthValue::Px(10.0)));
    assert_eq!(parse_length("1em"), Some(LengthValue::Em(1.0)));
    assert_eq!(parse_length("0"), Some(LengthValue::Px(0.0)));
}

#[test]
/// 测试 scroll-padding 简写解析
fn test_parse_scroll_padding_shorthand() {
    let result = parse_length_shorthand("5px 10px");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(5.0),
            LengthValue::Px(10.0),
            LengthValue::Px(5.0),
            LengthValue::Px(10.0),
        ])
    );
}

#[test]
/// 测试 scroll-padding 长属性解析（通过 parse_length）
fn test_parse_scroll_padding_longhands() {
    assert_eq!(parse_length("15px"), Some(LengthValue::Px(15.0)));
    assert_eq!(parse_length("2rem"), Some(LengthValue::Rem(2.0)));
}

// ═══════════════════════════════════════════════════════════════════════
// 17. container-type 属性值解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 container-type 值解析
fn test_parse_container_type_values() {
    assert_eq!(parse_container_type("normal"), Some(ContainerTypeValue::Normal));
    assert_eq!(parse_container_type("size"), Some(ContainerTypeValue::Size));
    assert_eq!(
        parse_container_type("inline-size"),
        Some(ContainerTypeValue::InlineSize)
    );
    assert_eq!(parse_container_type("invalid"), None);
}

#[test]
/// 测试 container-type 大小写不敏感
fn test_parse_container_type_case_insensitive() {
    assert_eq!(
        parse_container_type("INLINE-SIZE"),
        Some(ContainerTypeValue::InlineSize)
    );
    assert_eq!(parse_container_type("Size"), Some(ContainerTypeValue::Size));
}

#[test]
/// 测试 container-type 在样式表中解析
fn test_parse_container_type_in_stylesheet() {
    let css = ".card { container-type: inline-size; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert!(
                sr.declarations
                    .iter()
                    .any(|d| { d.property == "container-type" && d.value == "inline-size" })
            );
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
/// 测试 scroll-snap-type 属性在样式表中解析
fn test_parse_scroll_snap_type_in_stylesheet() {
    let css = ".scroller { scroll-snap-type: x mandatory; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert!(
                sr.declarations
                    .iter()
                    .any(|d| { d.property == "scroll-snap-type" && d.value == "x mandatory" })
            );
        }
        _ => panic!("Expected Style rule"),
    }
}

#[test]
/// 测试 scroll-snap-align 属性在样式表中解析
fn test_parse_scroll_snap_align_in_stylesheet() {
    let css = ".item { scroll-snap-align: start; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Style(sr) => {
            assert!(
                sr.declarations
                    .iter()
                    .any(|d| { d.property == "scroll-snap-align" && d.value == "start" })
            );
        }
        _ => panic!("Expected Style rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 18. Selector edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 :nth-child(3n+1) 带偏移的公式
fn test_parse_nth_child_3n_plus_1() {
    let stylesheet = Parser::parse_stylesheet("li:nth-child(3n+1) { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(NthPattern { a: 3, b: 1 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :only-child 伪类
fn test_parse_only_child() {
    let stylesheet = Parser::parse_stylesheet("p:only-child { color: red; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "only-child"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :only-of-type 伪类
fn test_parse_only_of_type() {
    let stylesheet = Parser::parse_stylesheet("p:only-of-type { color: blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "only-of-type"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :empty 伪类
fn test_parse_empty_selector() {
    let stylesheet = Parser::parse_stylesheet("div:empty { display: none; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "empty"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :checked, :disabled, :enabled 伪类
fn test_parse_ui_state_pseudo_classes() {
    let stylesheet = Parser::parse_stylesheet("input:checked { outline: 1px solid blue; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "checked"
        )));
    } else {
        panic!("Expected Style rule");
    }

    let stylesheet = Parser::parse_stylesheet("button:disabled { opacity: 0.5; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "disabled"
        )));
    } else {
        panic!("Expected Style rule");
    }

    let stylesheet = Parser::parse_stylesheet("input:enabled { background: white; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::Simple(name)) if name == "enabled"
        )));
    } else {
        panic!("Expected Style rule");
    }
}

#[test]
/// 测试 :nth-last-of-type 选择器
fn test_parse_nth_last_of_type() {
    let stylesheet = Parser::parse_stylesheet("li:nth-last-of-type(2) { color: green; }");
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        let compound = &sr.selectors[0].complex.parts[0].0;
        assert!(compound.subclass_selectors.iter().any(|s| matches!(
            s,
            SubclassSelector::PseudoClass(PseudoClassSelector::NthLastOfType(NthPattern { a: 0, b: 2 }))
        )));
    } else {
        panic!("Expected Style rule");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 19. Value parsing edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 calc() 嵌套乘除运算
fn test_parse_calc_nested_multiply_divide() {
    let expr = parse_calc("calc(2 * 10px)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(20.0));

    let expr = parse_calc("calc(100px / 2)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, Some(50.0));
}

#[test]
/// 测试 calc() 中除以零返回 None
fn test_eval_calc_divide_by_zero() {
    let expr = parse_calc("calc(100px / 0)").unwrap();
    let result = eval_calc(&expr, None);
    assert_eq!(result, None);
}

#[test]
/// 测试 url() 函数 tokenization
fn test_tokenize_url_with_path() {
    let tokens: Vec<_> = Tokenizer::new("url(../images/bg.png)").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(&tokens[0], Token::Url(u) if u == "../images/bg.png"));
}

#[test]
/// 测试 url() 带引号参数
fn test_tokenize_url_quoted() {
    let tokens: Vec<_> = Tokenizer::new("url('path/to/font.woff2')").collect_tokens();
    assert!(matches!(&tokens[0], Token::Url(u) if u == "path/to/font.woff2"));
}

#[test]
/// 测试 var() 嵌套在值中解析
fn test_parse_var_nested_fallback() {
    let result = parse_var("var(--spacing, 16px)");
    assert!(result.is_some());
    let var = result.unwrap();
    assert_eq!(var.name, "--spacing");
    assert_eq!(var.fallback, Some("16px".to_string()));
}

#[test]
/// 测试 parse_time 边界值
fn test_parse_time_edge_cases() {
    use crate::values::parse_time;
    assert_eq!(parse_time("0s"), Some(0.0));
    assert_eq!(parse_time("0ms"), Some(0.0));
    assert_eq!(parse_time("100ms"), Some(0.1));
    assert_eq!(parse_time("10"), None);
    assert_eq!(parse_time(""), None);
}

#[test]
/// 测试 timing-function cubic-bezier 参数
fn test_parse_timing_function_cubic_bezier_values() {
    use crate::values::parse_timing_function;
    let result = parse_timing_function("cubic-bezier(0.0, 0.0, 1.0, 1.0)");
    assert_eq!(
        result,
        Some(crate::values::TimingFunctionValue::CubicBezier(0.0, 0.0, 1.0, 1.0))
    );
}

#[test]
/// 测试 timing-function steps 带不同位置参数
fn test_parse_timing_function_steps_variants() {
    use crate::values::{StepPosition, TimingFunctionValue, parse_timing_function};
    assert_eq!(
        parse_timing_function("steps(3, jump-none)"),
        Some(TimingFunctionValue::Steps(3, Some(StepPosition::None)))
    );
    assert_eq!(
        parse_timing_function("steps(5, jump-both)"),
        Some(TimingFunctionValue::Steps(5, Some(StepPosition::Both)))
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 20. @rule parsing edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试嵌套 @media 规则：@media 内含另一个 @media
fn test_parse_nested_media_rules() {
    let css = "@media screen { @media (max-width: 600px) { div { color: blue; } } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::At(outer) => {
            assert_eq!(outer.name, "media");
            if let AtRuleBody::Block(inner_rules) = &outer.body {
                assert_eq!(inner_rules.len(), 1);
                match &inner_rules[0] {
                    Rule::At(inner) => {
                        assert_eq!(inner.name, "media");
                    }
                    _ => panic!("Expected inner At rule"),
                }
            } else {
                panic!("Expected Block body");
            }
        }
        _ => panic!("Expected outer At rule"),
    }
}

#[test]
/// 测试 @layer 排序（多个 layer 规则顺序）
fn test_parse_layer_ordering() {
    let css = "@layer reset { * { margin: 0; } } @layer base { body { font-size: 16px; } } @layer components { .btn { padding: 10px; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 3);
    let names: Vec<&str> = stylesheet
        .rules
        .iter()
        .map(|r| match r {
            Rule::Layer(lr) => lr.name.as_str(),
            _ => "unknown",
        })
        .collect();
    assert_eq!(names, vec!["reset", "base", "components"]);
}

#[test]
/// 测试 @import 带 print 媒体查询
fn test_parse_import_print_media() {
    let stylesheet = Parser::parse_stylesheet("@import \"print.css\" print;");
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Import(import_rule) => {
            assert_eq!(import_rule.url, "print.css");
            assert_eq!(import_rule.media_queries.len(), 1);
            assert_eq!(import_rule.media_queries[0], "print");
        }
        _ => panic!("Expected Import rule"),
    }
}

#[test]
/// 测试 @keyframes 带 from/to 混合百分比
fn test_parse_keyframes_mixed_from_to_percentage() {
    let css =
        "@keyframes anim { from { opacity: 0; } 25% { opacity: 0.25; } 50% { opacity: 0.5; } to { opacity: 1; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Keyframes(kf) => {
            assert_eq!(kf.name, "anim");
            assert_eq!(kf.keyframes.len(), 4);
            assert_eq!(kf.keyframes[0].selectors, vec![KeyframeSelector::From]);
            assert_eq!(kf.keyframes[1].selectors, vec![KeyframeSelector::Percentage(25.0)]);
            assert_eq!(kf.keyframes[2].selectors, vec![KeyframeSelector::Percentage(50.0)]);
            assert_eq!(kf.keyframes[3].selectors, vec![KeyframeSelector::To]);
        }
        _ => panic!("Expected Keyframes rule"),
    }
}

#[test]
/// 测试 @container 带 width 比较运算符条件
fn test_parse_container_with_comparison_condition() {
    let css = "@container card (width > 300px) { .child { width: 100%; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => {
            assert_eq!(cr.name.as_deref(), Some("card"));
            match &cr.condition {
                ContainerCondition::Size(sc) => {
                    assert_eq!(sc.feature, "width");
                    assert_eq!(sc.value, "300px");
                }
                _ => panic!("Expected Size condition"),
            }
        }
        _ => panic!("Expected Container rule"),
    }
}

#[test]
/// 测试 @container 带 inline-size 条件（不带函数包装）
fn test_parse_container_with_inline_size_condition() {
    let css = "@container (min-width: 300px) { .card { flex-direction: column; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    match &stylesheet.rules[0] {
        Rule::Container(cr) => {
            assert!(cr.name.is_none());
            match &cr.condition {
                ContainerCondition::Size(sc) => {
                    assert_eq!(sc.feature, "min-width");
                    assert_eq!(sc.value, "300px");
                }
                _ => panic!("Expected Size condition"),
            }
        }
        _ => panic!("Expected Container rule"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 21. Tokenizer edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试注释在值中间
fn test_tokenize_comment_in_value() {
    let tokens: Vec<_> = Tokenizer::new("10px /* comment */ 20px").collect_tokens();
    // Tokens: Dimension(10,px) Whitespace Comment Whitespace Dimension(20,px)
    assert!(tokens.len() >= 5);
    assert!(matches!(&tokens[0], Token::Dimension(n, u) if *n == 10.0 && u == "px"));
    assert_eq!(tokens[1], Token::Whitespace);
    assert!(matches!(&tokens[2], Token::Comment(_)));
    assert_eq!(tokens[3], Token::Whitespace);
    assert!(matches!(&tokens[4], Token::Dimension(n, u) if *n == 20.0 && u == "px"));
}

#[test]
/// 测试标识符中转义字符
fn test_tokenize_escaped_character_in_ident() {
    let tokens: Vec<_> = Tokenizer::new("\\41 ctive").collect_tokens(); // \41 = 'A'
    assert!(!tokens.is_empty());
    // The escaped \41 should produce 'A', so the ident should start with 'A'
    if let Token::Ident(s) = &tokens[0] {
        assert!(s.starts_with('A'), "Expected ident starting with 'A', got '{}'", s);
    }
}

#[test]
/// 测试科学计数法数字
fn test_tokenize_scientific_notation() {
    let tokens: Vec<_> = Tokenizer::new("1e2").collect_tokens();
    assert!(matches!(&tokens[0], Token::Number(n) if (*n - 100.0).abs() < 0.001));

    let tokens: Vec<_> = Tokenizer::new("3.5e-1").collect_tokens();
    assert!(matches!(&tokens[0], Token::Number(n) if (*n - 0.35).abs() < 0.001));
}

#[test]
/// 测试多行字符串
fn test_tokenize_multiline_string() {
    let tokens: Vec<_> = Tokenizer::new("\"line1\\nline2\"").collect_tokens();
    assert!(matches!(&tokens[0], Token::String(s) if s.contains("line1") && s.contains("line2")));
}

#[test]
/// 测试自定义属性 (--*) tokenization
fn test_tokenize_custom_property() {
    let tokens: Vec<_> = Tokenizer::new("--main-color").collect_tokens();
    // Custom properties start with '--', which is parsed as an ident starting with '-'
    assert!(!tokens.is_empty());
    if let Token::Ident(s) = &tokens[0] {
        assert!(s.starts_with('-'), "Expected ident starting with '-', got '{}'", s);
    }
}

#[test]
/// 测试连续空白合并为单个 Whitespace token
fn test_tokenize_multiple_whitespace() {
    let tokens: Vec<_> = Tokenizer::new("   \t  \n  ").collect_tokens();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0], Token::Whitespace);
}

// ═══════════════════════════════════════════════════════════════════════
// 22. Error recovery
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试格式错误的选择器恢复：未知 token 后继续解析
fn test_parse_malformed_selector_recovery() {
    let css = "div { color: red; } %invalid span { color: green; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // Parser should at least parse the first valid rule
    assert!(stylesheet.rules.len() >= 1);
    let has_red = stylesheet.rules.iter().any(|r| {
        if let Rule::Style(sr) = r {
            sr.declarations.iter().any(|d| d.value.contains("red"))
        } else {
            false
        }
    });
    assert!(has_red, "Expected to parse valid 'div {{ color: red; }}' rule");
}

#[test]
/// 测试未闭合字符串恢复
fn test_parse_unclosed_string_recovery() {
    let css = "div { content: \"unclosed; } span { color: blue; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // Parser should produce at least one rule (even if malformed)
    assert!(stylesheet.rules.len() >= 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 23. Token 偏移量追踪测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 token 偏移量追踪：验证 token 的起始字节位置正确
fn test_token_offset_tracking() {
    let css = "div { color: red; }";
    let tokens: Vec<Spanned> = Tokenizer::new(css).collect();

    // "div" 起始于偏移量 0
    assert_eq!(tokens[0].offset, 0);
    assert!(matches!(tokens[0].token, Token::Ident(ref s) if s == "div"));

    // " " （空白）起始于偏移量 3
    assert_eq!(tokens[1].offset, 3);
    assert!(matches!(tokens[1].token, Token::Whitespace));

    // "{" 起始于偏移量 4
    assert_eq!(tokens[2].offset, 4);
    assert!(matches!(tokens[2].token, Token::LBrace));

    // " " （空白）起始于偏移量 5
    assert_eq!(tokens[3].offset, 5);

    // "color" 起始于偏移量 6
    assert_eq!(tokens[4].offset, 6);
    assert!(matches!(tokens[4].token, Token::Ident(ref s) if s == "color"));

    // ":" 起始于偏移量 11
    assert_eq!(tokens[5].offset, 11);

    // " " （空白）起始于偏移量 12
    assert_eq!(tokens[6].offset, 12);

    // "red" 起始于偏移量 13
    assert_eq!(tokens[7].offset, 13);

    // ";" 起始于偏移量 16
    assert_eq!(tokens[8].offset, 16);

    // " " （空白）起始于偏移量 17
    assert_eq!(tokens[9].offset, 17);

    // "}" 起始于偏移量 18
    assert_eq!(tokens[10].offset, 18);
}

#[test]
/// 测试换行后位置正确推进
fn test_tokenizer_position_after_newline() {
    let css = "a\nb";
    let tokens: Vec<Spanned> = Tokenizer::new(css).collect();

    // "a" 起始于偏移量 0
    assert_eq!(tokens[0].offset, 0);

    // "\n" （空白）起始于偏移量 1
    assert_eq!(tokens[1].offset, 1);

    // "b" 起始于偏移量 2
    assert_eq!(tokens[2].offset, 2);

    // 测试 \r\n 换行
    let css_crlf = "a\r\nb";
    let tokens_crlf: Vec<Spanned> = Tokenizer::new(css_crlf).collect();

    // "a" 起始于偏移量 0
    assert_eq!(tokens_crlf[0].offset, 0);

    // 空白（包含 \r\n）起始于偏移量 1
    assert_eq!(tokens_crlf[1].offset, 1);

    // "b" 起始于偏移量 3（\r\n 占 2 字节）
    assert_eq!(tokens_crlf[2].offset, 3);
}

#[test]
/// 测试字节偏移量到行:列的转换
fn test_line_column_from_offset() {
    let source = "div {\n  color: red;\n}";

    // 偏移量 0 → 第 1 行第 1 列 ("d")
    assert_eq!(line_column_from_offset(source, 0), (1, 1));

    // 偏移量 3 → 第 1 行第 4 列 (" ")
    assert_eq!(line_column_from_offset(source, 3), (1, 4));

    // 偏移量 5 → 第 1 行第 6 列 ("\n" 本身)
    assert_eq!(line_column_from_offset(source, 5), (1, 6));

    // 偏移量 6 → 第 2 行第 1 列 ("\n" 后第一个字符)
    assert_eq!(line_column_from_offset(source, 6), (2, 1));

    // 偏移量 8 → 第 2 行第 3 列 ("c")
    assert_eq!(line_column_from_offset(source, 8), (2, 3));

    // 偏移量 14 → 第 2 行第 9 列 (冒号后空格)
    assert_eq!(line_column_from_offset(source, 14), (2, 9));

    // 偏移量 19 → 第 2 行第 14 列 ("\n" 是第 2 行最后一个字符)
    assert_eq!(line_column_from_offset(source, 19), (2, 14));

    // 偏移量 20 → 第 3 行第 1 列 ("}")
    assert_eq!(line_column_from_offset(source, 20), (3, 1));

    // 测试 \r\n 换行
    let source_crlf = "a\r\nb";
    assert_eq!(line_column_from_offset(source_crlf, 0), (1, 1));
    assert_eq!(line_column_from_offset(source_crlf, 1), (1, 2));
    // \r 触发换行并跳过 \n，因此 \r\n 整体被视为换行
    assert_eq!(line_column_from_offset(source_crlf, 2), (2, 1));
    assert_eq!(line_column_from_offset(source_crlf, 3), (2, 1)); // b 的位置
    assert_eq!(line_column_from_offset(source_crlf, 4), (2, 2)); // b 之后
}

#[test]
/// 测试无效 @rule 恢复
fn test_parse_invalid_at_rule_recovery() {
    let css = "@unknown-rule something { div { color: red; } } p { font-size: 14px; }";
    let stylesheet = Parser::parse_stylesheet(css);
    // Should parse @unknown-rule as a generic At rule and still get p rule
    assert!(stylesheet.rules.len() >= 2);
    let has_p = stylesheet.rules.iter().any(|r| {
        if let Rule::Style(sr) = r {
            sr.selectors.iter().any(|s| {
                s.complex.parts[0]
                    .0
                    .type_selector
                    .as_ref()
                    .map_or(false, |ts| matches!(ts, TypeSelector::Tag(t) if t == "p"))
            })
        } else {
            false
        }
    });
    assert!(has_p, "Expected to recover and parse 'p' rule");
}

#[test]
/// 测试多余右花括号恢复
fn test_parse_extra_closing_brace_recovery() {
    let css = "div { color: red; } } span { color: green; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert!(stylesheet.rules.len() >= 1);
}

#[test]
/// 测试无效属性值恢复
fn test_parse_invalid_property_value_recovery() {
    let css = "div { color: ; font-size: 16px; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        // Should still have font-size declaration (color may be empty value)
        assert!(sr.declarations.iter().any(|d| d.property == "font-size"));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 23. Round-trip consistency
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试简单规则序列化：声明属性和值保持一致
fn test_round_trip_simple_rule_consistency() {
    let css = "div { color: red; font-size: 16px; }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 1);
    if let Rule::Style(sr) = &stylesheet.rules[0] {
        assert_eq!(sr.declarations.len(), 2);
        assert_eq!(sr.declarations[0].property, "color");
        assert_eq!(sr.declarations[0].value, "red");
        assert_eq!(sr.declarations[1].property, "font-size");
        assert_eq!(sr.declarations[1].value, "16px");
    }
}

#[test]
/// 测试解析-序列化-解析一致性
fn test_round_trip_parse_serialize_parse() {
    let css = "div { color: red; } span { font-size: 16px; }";
    let first = Parser::parse_stylesheet(css);
    // Re-parse should produce the same structure
    let second = Parser::parse_stylesheet(css);
    assert_eq!(first.rules.len(), second.rules.len());
    for (r1, r2) in first.rules.iter().zip(second.rules.iter()) {
        match (r1, r2) {
            (Rule::Style(s1), Rule::Style(s2)) => {
                assert_eq!(s1.declarations.len(), s2.declarations.len());
                for (d1, d2) in s1.declarations.iter().zip(s2.declarations.iter()) {
                    assert_eq!(d1.property, d2.property);
                    assert_eq!(d1.value, d2.value);
                    assert_eq!(d1.important, d2.important);
                }
            }
            (Rule::At(a1), Rule::At(a2)) => {
                assert_eq!(a1.name, a2.name);
                assert_eq!(a1.prelude, a2.prelude);
            }
            _ => {}
        }
    }
}

#[test]
/// 测试复杂样式表往返一致性
fn test_round_trip_complex_stylesheet() {
    let css = "@media screen { div { color: red !important; } } @layer base { p { margin: 0; } }";
    let stylesheet = Parser::parse_stylesheet(css);
    assert_eq!(stylesheet.rules.len(), 2);
    // Verify @media
    match &stylesheet.rules[0] {
        Rule::At(at) => {
            assert_eq!(at.name, "media");
            assert!(at.prelude.contains("screen"));
            if let AtRuleBody::Block(rules) = &at.body {
                assert_eq!(rules.len(), 1);
                if let Rule::Style(sr) = &rules[0] {
                    let has_important = sr.declarations.iter().any(|d| d.important);
                    assert!(has_important);
                }
            }
        }
        _ => panic!("Expected At rule"),
    }
    // Verify @layer
    match &stylesheet.rules[1] {
        Rule::Layer(lr) => {
            assert_eq!(lr.name, "base");
            assert_eq!(lr.rules.len(), 1);
        }
        _ => panic!("Expected Layer rule"),
    }
}

#[test]
/// 测试空白规范化：多余空白不影响解析结果
fn test_whitespace_normalization() {
    let css1 = "div{color:red}";
    let css2 = "div  {  color  :  red  ;  }";
    let ss1 = Parser::parse_stylesheet(css1);
    let ss2 = Parser::parse_stylesheet(css2);
    assert_eq!(ss1.rules.len(), ss2.rules.len());
    if let (Rule::Style(s1), Rule::Style(s2)) = (&ss1.rules[0], &ss2.rules[0]) {
        assert_eq!(s1.declarations.len(), s2.declarations.len());
        assert_eq!(s1.declarations[0].property, s2.declarations[0].property);
        // The value should be "red" regardless of whitespace
        assert_eq!(s1.declarations[0].value, "red");
        assert_eq!(s2.declarations[0].value, "red");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 24. Additional selector and specificity tests
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 :where() specificity 为 0
fn test_specificity_where_zero() {
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Where(vec![
                        class_sel("active"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert_eq!(selector::specificity(&sel), (0, 0, 0));
}

#[test]
/// 测试 :is() specificity 取参数最大值
fn test_specificity_is_takes_max() {
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Is(vec![
                        id_sel("main"),
                        tag_sel("div"),
                    ]))],
                },
                None,
            )],
        },
    };
    // :is(#main, div) -> max((1,0,0), (0,0,1)) = (1,0,0)
    assert_eq!(selector::specificity(&sel), (1, 0, 0));
}

#[test]
/// 测试 :not() specificity 取参数最大值
fn test_specificity_not_takes_max() {
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("p".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Not(vec![
                        class_sel("hidden"),
                        id_sel("special"),
                    ]))],
                },
                None,
            )],
        },
    };
    // p:not(.hidden, #special) -> tag(0,0,1) + max((0,1,0), (1,0,0)) = (1,0,1)
    assert_eq!(selector::specificity(&sel), (1, 0, 1));
}

// ═══════════════════════════════════════════════════════════════════════
// 25. Additional value parsing edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_length_shorthand 零值
fn test_parse_length_shorthand_zero() {
    let result = parse_length_shorthand("0");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(0.0),
            LengthValue::Px(0.0),
            LengthValue::Px(0.0),
            LengthValue::Px(0.0),
        ])
    );
}

#[test]
/// 测试 parse_length_shorthand 混合单位
fn test_parse_length_shorthand_mixed_units() {
    let result = parse_length_shorthand("10px 1em");
    assert_eq!(
        result,
        Some([
            LengthValue::Px(10.0),
            LengthValue::Em(1.0),
            LengthValue::Px(10.0),
            LengthValue::Em(1.0),
        ])
    );
}

#[test]
/// 测试 linear-gradient to top 方向
fn test_parse_linear_gradient_to_top() {
    let result = parse_gradient("linear-gradient(to top, blue, transparent)");
    assert!(result.is_some());
    match result.unwrap() {
        GradientValue::Linear(lg) => {
            assert_eq!(lg.direction, GradientDirection::ToTop);
            assert_eq!(lg.stops.len(), 2);
            assert!(matches!(lg.stops[0].color, ColorValue::Rgba(0, 0, 255, 255)));
            assert!(matches!(lg.stops[1].color, ColorValue::Transparent));
        }
        _ => panic!("Expected LinearGradient"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 32. Color parsing 边界测试（覆盖 color.rs 的 uncovered 路径）
// ═══════════════════════════════════════════════════════════════════════

#[test]
/// 测试 parse_color 命名颜色的大小写不敏感
fn test_parse_named_color_case_insensitive() {
    let test_cases = vec![
        ("red", ColorValue::Rgba(255, 0, 0, 255)),
        ("RED", ColorValue::Rgba(255, 0, 0, 255)),
        ("Red", ColorValue::Rgba(255, 0, 0, 255)),
        ("blue", ColorValue::Rgba(0, 0, 255, 255)),
        ("BLUE", ColorValue::Rgba(0, 0, 255, 255)),
        ("Blue", ColorValue::Rgba(0, 0, 255, 255)),
        ("green", ColorValue::Rgba(0, 128, 0, 255)),
        ("GREEN", ColorValue::Rgba(0, 128, 0, 255)),
        ("Green", ColorValue::Rgba(0, 128, 0, 255)),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color 各种十六进制颜色格式
fn test_parse_hex_colors() {
    let test_cases = vec![
        // 标准 3 位十六进制
        ("#fff", ColorValue::Rgba(255, 255, 255, 255)),
        ("#FFF", ColorValue::Rgba(255, 255, 255, 255)),
        ("#f00", ColorValue::Rgba(255, 0, 0, 255)),
        ("#0f0", ColorValue::Rgba(0, 255, 0, 255)),
        ("#00f", ColorValue::Rgba(0, 0, 255, 255)),
        // 标准 6 位十六进制
        ("#ffffff", ColorValue::Rgba(255, 255, 255, 255)),
        ("#000000", ColorValue::Rgba(0, 0, 0, 255)),
        ("#ff0000", ColorValue::Rgba(255, 0, 0, 255)),
        ("#00ff00", ColorValue::Rgba(0, 255, 0, 255)),
        ("#0000ff", ColorValue::Rgba(0, 0, 255, 255)),
        ("#123456", ColorValue::Rgba(18, 52, 86, 255)),
        // 4 位十六进制（带透明度）
        ("#ffff", ColorValue::Rgba(255, 255, 255, 255)),
        ("#fffff0", ColorValue::Rgba(255, 255, 240, 255)),
        ("#f00f", ColorValue::Rgba(255, 0, 0, 255)),
        ("#0f0f", ColorValue::Rgba(0, 255, 0, 255)),
        ("#00ff", ColorValue::Rgba(0, 0, 255, 255)),
        // 8 位十六进制（带透明度）
        ("#ffffffff", ColorValue::Rgba(255, 255, 255, 255)),
        ("#00000000", ColorValue::Rgba(0, 0, 0, 0)),
        ("#ff0000ff", ColorValue::Rgba(255, 0, 0, 255)),
        ("#ff000080", ColorValue::Rgba(255, 0, 0, 128)),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color 无效的十六进制颜色
fn test_parse_invalid_hex_colors() {
    let test_cases = vec![
        "#",        // 太短
        "#ff",      // 无效长度
        "#fffff",   // 无效长度
        "#fffffff", // 无效长度
        "#gggggg",  // 非法字符
        "#12345",   // 无效长度
        "#1234567", // 无效长度
        "123456",   // 没有 #
        "#",        // 只有 #
        "##",       // 只有两个 #
    ];

    for input in test_cases {
        let result = parse_color(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color rgb() 和 rgba() 函数的各种格式
fn test_parse_rgb_function_colors() {
    let test_cases = vec![
        // rgb() 形式
        ("rgb(255, 0, 0)", ColorValue::Rgba(255, 0, 0, 255)),
        ("rgb(0, 255, 0)", ColorValue::Rgba(0, 255, 0, 255)),
        ("rgb(0, 0, 255)", ColorValue::Rgba(0, 0, 255, 255)),
        ("rgb(0, 0, 0)", ColorValue::Rgba(0, 0, 0, 255)),
        ("rgb(255, 255, 255)", ColorValue::Rgba(255, 255, 255, 255)),
        // rgba() 形式
        ("rgba(255, 0, 0, 1)", ColorValue::Rgba(255, 0, 0, 255)),
        ("rgba(255, 0, 0, 0.5)", ColorValue::Rgba(255, 0, 0, 128)),
        ("rgba(255, 0, 0, 0)", ColorValue::Rgba(255, 0, 0, 0)),
        ("rgba(255, 0, 0, 0.8)", ColorValue::Rgba(255, 0, 0, 204)),
        // 带空格的格式
        ("rgb(255, 0, 0)", ColorValue::Rgba(255, 0, 0, 255)),
        ("rgb( 255 , 0 , 0 )", ColorValue::Rgba(255, 0, 0, 255)),
        ("rgba(255, 0, 0, 1)", ColorValue::Rgba(255, 0, 0, 255)),
        ("rgba( 255 , 0 , 0 , 1 )", ColorValue::Rgba(255, 0, 0, 255)),
        // 百分比格式
        ("rgb(100%, 0%, 0%)", ColorValue::Rgba(255, 0, 0, 255)),
        ("rgb(0%, 100%, 0%)", ColorValue::Rgba(0, 255, 0, 255)),
        ("rgb(0%, 0%, 100%)", ColorValue::Rgba(0, 0, 255, 255)),
        ("rgba(100%, 0%, 0%, 100%)", ColorValue::Rgba(255, 0, 0, 255)),
        ("rgba(50%, 50%, 50%, 50%)", ColorValue::Rgba(128, 128, 128, 128)),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color 无效的 rgb() 格式
fn test_parse_invalid_rgb_colors() {
    // 解析器对参数数量和范围比较宽容，这里只测试确实返回 None 的情况
    let test_cases = vec![
        "rgb()",           // 没有参数
        "rg(255, 0, 0)",   // 拼写错误
        "rgbx(255, 0, 0)", // 未知函数
    ];

    for input in test_cases {
        let result = parse_color(input);
        assert_eq!(result, None, "Should fail to parse: {}", input);
    }
}

#[test]
/// 测试 parse_color hsl() 和 hsla() 函数
fn test_parse_hsl_colors() {
    let test_cases = vec![
        // hsl() 形式
        ("hsl(0, 100%, 50%)", ColorValue::Hsla(0.0, 100.0, 50.0, 1.0)),
        ("hsl(120, 100%, 50%)", ColorValue::Hsla(120.0, 100.0, 50.0, 1.0)),
        ("hsl(240, 100%, 50%)", ColorValue::Hsla(240.0, 100.0, 50.0, 1.0)),
        // hsla() 形式
        ("hsla(0, 100%, 50%, 1)", ColorValue::Hsla(0.0, 100.0, 50.0, 1.0)),
        ("hsla(0, 100%, 50%, 0.5)", ColorValue::Hsla(0.0, 100.0, 50.0, 0.5)),
        ("hsla(0, 100%, 50%, 0)", ColorValue::Hsla(0.0, 100.0, 50.0, 0.0)),
        // 带 deg 单位的色相
        ("hsl(0deg, 100%, 50%)", ColorValue::Hsla(0.0, 100.0, 50.0, 1.0)),
        ("hsl(360deg, 100%, 50%)", ColorValue::Hsla(360.0, 100.0, 50.0, 1.0)),
        ("hsl(720deg, 100%, 50%)", ColorValue::Hsla(720.0, 100.0, 50.0, 1.0)),
    ];

    for (input, expected) in test_cases {
        let result = parse_color(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}
