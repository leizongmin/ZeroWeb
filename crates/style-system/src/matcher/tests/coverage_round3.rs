//! Matcher 覆盖率测试第三轮：@layer、@supports、@container 集成、
//! matches_nth_pattern 负系数、length_to_px 非px单位、get_axis_size 等。

use super::super::*;
use zero_css_parser::ast::{
    AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector, ContainerCondition,
    ContainerRule, ContainerSizeCondition, Declaration, LayerRule, NthPattern, PseudoClassSelector, Rule, Selector,
    StyleRule, SubclassSelector, SupportsCondition, SupportsRule, TypeSelector,
};
use zero_css_parser::media_query::MediaContext;
use zero_dom::Document;

// ═══════════════════════════════════════════════════════════════════════
// matches_nth_pattern — 负系数 a
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_pattern_negative_a_matches() {
    // -2n+5: a=-2, b=5
    // index=5: diff=5-5=0, 0<=0 && 0%-2==0 → true
    let pattern = NthPattern { a: -2, b: 5 };
    assert!(matches_nth_pattern(5, &pattern));
}

#[test]
fn test_nth_pattern_negative_a_no_match() {
    // -2n+5: index=6 → diff=6-5=1, 1<=0? false
    let pattern = NthPattern { a: -2, b: 5 };
    assert!(!matches_nth_pattern(6, &pattern));
}

#[test]
fn test_nth_pattern_negative_a_index_3() {
    // -2n+5: index=3 → diff=3-5=-2, -2<=0 && -2%-2==0 → true
    let pattern = NthPattern { a: -2, b: 5 };
    assert!(matches_nth_pattern(3, &pattern));
}

#[test]
fn test_nth_pattern_negative_a_index_1() {
    // -2n+5: index=1 → diff=1-5=-4, -4<=0 && -4%-2==0 → true
    let pattern = NthPattern { a: -2, b: 5 };
    assert!(matches_nth_pattern(1, &pattern));
}

#[test]
fn test_nth_pattern_a_zero_exact_match() {
    // a=0, b=3 → 只有 index==3 才匹配
    let pattern = NthPattern { a: 0, b: 3 };
    assert!(matches_nth_pattern(3, &pattern));
    assert!(!matches_nth_pattern(2, &pattern));
    assert!(!matches_nth_pattern(4, &pattern));
}

#[test]
fn test_nth_pattern_positive_a() {
    // 2n+1: a=2, b=1
    let pattern = NthPattern { a: 2, b: 1 };
    assert!(matches_nth_pattern(1, &pattern)); // diff=0, 0>=0 && 0%2==0
    assert!(matches_nth_pattern(3, &pattern)); // diff=2, 2>=0 && 2%2==0
    assert!(matches_nth_pattern(5, &pattern)); // diff=4
    assert!(!matches_nth_pattern(2, &pattern)); // diff=1, 1%2!=0
    assert!(!matches_nth_pattern(4, &pattern)); // diff=3, 3%2!=0
}

// ═══════════════════════════════════════════════════════════════════════
// length_to_px — 非 px 单位返回 0.0
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_length_to_px_px_value() {
    assert_eq!(length_to_px("100px"), Some(100.0));
}

#[test]
fn test_length_to_px_zero() {
    assert_eq!(length_to_px("0"), Some(0.0));
}

#[test]
fn test_length_to_px_em_returns_zero() {
    // 非 px 单位返回 0.0（但 Some）
    assert_eq!(length_to_px("2em"), Some(0.0));
}

#[test]
fn test_length_to_px_rem_returns_zero() {
    assert_eq!(length_to_px("1.5rem"), Some(0.0));
}

#[test]
fn test_length_to_px_percentage_returns_zero() {
    assert_eq!(length_to_px("50%"), Some(0.0));
}

#[test]
fn test_length_to_px_invalid() {
    assert_eq!(length_to_px("invalid"), None);
}

#[test]
fn test_length_to_px_empty() {
    assert_eq!(length_to_px(""), None);
}

// ═══════════════════════════════════════════════════════════════════════
// get_axis_size
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_get_axis_size_width() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert_eq!(get_axis_size(&ctx, "width"), Some(800.0));
}

#[test]
fn test_get_axis_size_inline_size() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert_eq!(get_axis_size(&ctx, "inline-size"), Some(800.0));
}

#[test]
fn test_get_axis_size_height() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert_eq!(get_axis_size(&ctx, "height"), Some(600.0));
}

#[test]
fn test_get_axis_size_block_size() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert_eq!(get_axis_size(&ctx, "block-size"), Some(600.0));
}

#[test]
fn test_get_axis_size_min_width() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert_eq!(get_axis_size(&ctx, "min-width"), Some(800.0));
}

#[test]
fn test_get_axis_size_max_height() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert_eq!(get_axis_size(&ctx, "max-height"), Some(600.0));
}

#[test]
fn test_get_axis_size_unknown() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert_eq!(get_axis_size(&ctx, "unknown-feature"), None);
}

#[test]
fn test_get_axis_size_none_context() {
    let ctx = ContainerContext::new();
    assert_eq!(get_axis_size(&ctx, "width"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// ContainerContext methods
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_context_new() {
    let ctx = ContainerContext::new();
    assert!(ctx.container_width.is_none());
    assert!(ctx.container_height.is_none());
}

#[test]
fn test_container_context_default() {
    let ctx = ContainerContext::default();
    assert!(ctx.container_width.is_none());
    assert!(ctx.container_height.is_none());
}

#[test]
fn test_container_context_with_size() {
    let ctx = ContainerContext::with_size(1024.0, 768.0);
    assert_eq!(ctx.container_width, Some(1024.0));
    assert_eq!(ctx.container_height, Some(768.0));
}

#[test]
fn test_container_context_clone() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    let cloned = ctx.clone();
    assert_eq!(cloned.container_width, Some(800.0));
    assert_eq!(cloned.container_height, Some(600.0));
}

#[test]
fn test_container_context_debug() {
    let ctx = ContainerContext::with_size(100.0, 200.0);
    let debug_str = format!("{:?}", ctx);
    assert!(debug_str.contains("100"));
    assert!(debug_str.contains("200"));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_container_condition — 范围语法
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_range_width() {
    // 200px <= width <= 500px — 容器宽度 400 满足
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: String::new(),
            operator: None,
            range_min: Some("200px".to_string()),
            range_max: Some("500px".to_string()),
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_range_too_narrow() {
    // 200px <= width <= 500px — 容器宽度 100 不满足
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: String::new(),
            operator: None,
            range_min: Some("200px".to_string()),
            range_max: Some("500px".to_string()),
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(100.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_range_too_wide() {
    // 200px <= width <= 500px — 容器宽度 800 不满足
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: String::new(),
            operator: None,
            range_min: Some("200px".to_string()),
            range_max: Some("500px".to_string()),
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_range_invalid_min() {
    // 范围 min 不是有效 px — evaluate 返回 false
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: String::new(),
            operator: None,
            range_min: Some("invalid".to_string()),
            range_max: Some("500px".to_string()),
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_container_condition — 比较运算符
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_gt() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "300px".to_string(),
            operator: Some(">".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_gt_fail() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "300px".to_string(),
            operator: Some(">".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(200.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_lt() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "height".to_string(),
            value: "800px".to_string(),
            operator: Some("<".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_lte() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "400px".to_string(),
            operator: Some("<=".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_gte() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "400px".to_string(),
            operator: Some(">=".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_unknown_operator() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "400px".to_string(),
            operator: Some("!=".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_container_condition — 冒号语法
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_min_width_colon() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            value: "400px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(500.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_min_width_colon_fail() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            value: "400px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(300.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_max_height_colon() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "max-height".to_string(),
            value: "600px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(800.0, 500.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_exact_width_colon() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "800px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_exact_width_colon_fail() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "800px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(801.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_no_context() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            value: "400px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    assert!(!evaluate_container_condition(&rule, None));
}

#[test]
fn test_container_context_missing_axis() {
    // container_width is None, should return false for width queries
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            value: "400px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext {
        container_width: None,
        container_height: Some(600.0),
    };
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_supports_condition — logical operators
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_and_both_true() {
    let cond = SupportsCondition::And(vec![
        SupportsCondition::Property("display".to_string(), "block".to_string()),
        SupportsCondition::Property("color".to_string(), "red".to_string()),
    ]);
    assert!(evaluate_supports_condition(&cond));
}

#[test]
fn test_supports_and_one_false() {
    let cond = SupportsCondition::And(vec![
        SupportsCondition::Property("display".to_string(), "block".to_string()),
        SupportsCondition::Property("unknown-prop".to_string(), "value".to_string()),
    ]);
    assert!(!evaluate_supports_condition(&cond));
}

#[test]
fn test_supports_or_one_true() {
    let cond = SupportsCondition::Or(vec![
        SupportsCondition::Property("unknown-prop".to_string(), "value".to_string()),
        SupportsCondition::Property("display".to_string(), "block".to_string()),
    ]);
    assert!(evaluate_supports_condition(&cond));
}

#[test]
fn test_supports_or_all_false() {
    let cond = SupportsCondition::Or(vec![
        SupportsCondition::Property("unknown1".to_string(), "v".to_string()),
        SupportsCondition::Property("unknown2".to_string(), "v".to_string()),
    ]);
    assert!(!evaluate_supports_condition(&cond));
}

#[test]
fn test_supports_not_true() {
    let cond = SupportsCondition::Not(Box::new(SupportsCondition::Property(
        "display".to_string(),
        "block".to_string(),
    )));
    assert!(!evaluate_supports_condition(&cond));
}

#[test]
fn test_supports_not_false() {
    let cond = SupportsCondition::Not(Box::new(SupportsCondition::Property(
        "unknown-prop".to_string(),
        "value".to_string(),
    )));
    assert!(evaluate_supports_condition(&cond));
}

// ═══════════════════════════════════════════════════════════════════════
// is_valid_selector_parse — more edge cases
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_valid_selector_single_gt() {
    // 单个 > 应该无效（以组合器开头）
    assert!(!is_valid_selector_parse(">", &[]));
}

#[test]
fn test_valid_selector_plus_tilde() {
    assert!(!is_valid_selector_parse("+ ~", &[]));
}

#[test]
fn test_valid_selector_normal_descendant() {
    let css = "div p { }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    if let Some(Rule::Style(style_rule)) = stylesheet.rules.first() {
        assert!(is_valid_selector_parse("div p", &style_rule.selectors));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// collect_from_rules — @layer integration
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_from_layer_rule() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let layer_rule = LayerRule {
        name: "base".to_string(),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![Selector {
                complex: ComplexSelector {
                    parts: vec![(
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    )],
                },
            }],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheet = zero_css_parser::Stylesheet {
        rules: vec![Rule::Layer(layer_rule)],
    };

    let results = collect_matching_declarations(&doc, el, &[stylesheet]);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "color");
    assert_eq!(results[0].1, "red");
    // layer index should be assigned
    assert!(results[0].4.is_some());
}

#[test]
fn test_collect_from_nested_layers() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("p");
    doc.append_child(root, el).unwrap();

    let layer1 = LayerRule {
        name: "base".to_string(),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![Selector {
                complex: ComplexSelector {
                    parts: vec![(
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("p".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    )],
                },
            }],
            declarations: vec![Declaration {
                property: "margin".to_string(),
                value: "0".to_string(),
                important: false,
            }],
        })],
    };

    let layer2 = LayerRule {
        name: "components".to_string(),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![Selector {
                complex: ComplexSelector {
                    parts: vec![(
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("p".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    )],
                },
            }],
            declarations: vec![Declaration {
                property: "padding".to_string(),
                value: "10px".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheet = zero_css_parser::Stylesheet {
        rules: vec![Rule::Layer(layer1), Rule::Layer(layer2)],
    };

    let results = collect_matching_declarations(&doc, el, &[stylesheet]);
    assert_eq!(results.len(), 2);
    // Each layer gets a different index
    assert_ne!(results[0].4, results[1].4);
}

// ═══════════════════════════════════════════════════════════════════════
// collect_from_rules — @supports integration
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_from_supports_rule_true() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let supports_rule = SupportsRule {
        condition: SupportsCondition::Property("display".to_string(), "block".to_string()),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![Selector {
                complex: ComplexSelector {
                    parts: vec![(
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    )],
                },
            }],
            declarations: vec![Declaration {
                property: "display".to_string(),
                value: "block".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheet = zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(supports_rule)],
    };

    let results = collect_matching_declarations(&doc, el, &[stylesheet]);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_collect_from_supports_rule_false() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let supports_rule = SupportsRule {
        condition: SupportsCondition::Property("nonexistent-prop".to_string(), "value".to_string()),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![Selector {
                complex: ComplexSelector {
                    parts: vec![(
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    )],
                },
            }],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheet = zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(supports_rule)],
    };

    let results = collect_matching_declarations(&doc, el, &[stylesheet]);
    assert!(
        results.is_empty(),
        "Unsupported property should not produce declarations"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// collect_from_rules — @container integration
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_from_container_rule_matching() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let container_rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            value: "400px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![Selector {
                complex: ComplexSelector {
                    parts: vec![(
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    )],
                },
            }],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheet = zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(container_rule)],
    };

    let ctx = ContainerContext::with_size(500.0, 600.0);
    let results = collect_matching_declarations_with_media(
        &doc,
        el,
        std::slice::from_ref(&stylesheet),
        &build_stylesheet_index(std::slice::from_ref(&stylesheet)),
        None,
        Some(&ctx),
    );
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, "blue");
}

#[test]
fn test_collect_from_container_rule_not_matching() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let container_rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            value: "800px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![Selector {
                complex: ComplexSelector {
                    parts: vec![(
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    )],
                },
            }],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheet = zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(container_rule)],
    };

    let ctx = ContainerContext::with_size(500.0, 600.0);
    let results = collect_matching_declarations_with_media(
        &doc,
        el,
        std::slice::from_ref(&stylesheet),
        &build_stylesheet_index(std::slice::from_ref(&stylesheet)),
        None,
        Some(&ctx),
    );
    assert!(
        results.is_empty(),
        "Container condition not met, should not produce declarations"
    );
}

#[test]
fn test_collect_from_container_rule_no_context() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let container_rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            value: "400px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![Selector {
                complex: ComplexSelector {
                    parts: vec![(
                        CompoundSelector {
                            type_selector: Some(TypeSelector::Tag("div".to_string())),
                            subclass_selectors: vec![],
                        },
                        None,
                    )],
                },
            }],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheet = zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(container_rule)],
    };

    // No container context — should not apply
    let results = collect_matching_declarations(&doc, el, &[stylesheet]);
    assert!(results.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// collect_from_rules — @import and @keyframes skipped
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_skips_import() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let stylesheet = zero_css_parser::Stylesheet {
        rules: vec![Rule::Import(zero_css_parser::ast::ImportRule {
            url: "style.css".to_string(),
            media_queries: vec![],
        })],
    };

    let results = collect_matching_declarations(&doc, el, &[stylesheet]);
    assert!(results.is_empty());
}

#[test]
fn test_collect_skips_keyframes() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let stylesheet = zero_css_parser::Stylesheet {
        rules: vec![Rule::Keyframes(zero_css_parser::ast::KeyframesRule {
            name: "test".to_string(),
            keyframes: vec![],
        })],
    };

    let results = collect_matching_declarations(&doc, el, &[stylesheet]);
    assert!(results.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// is_property_supported — remaining property checks
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_property_supported_container_type() {
    assert!(is_property_supported("container-type", "inline-size"));
    assert!(!is_property_supported("container-type", "invalid"));
}

#[test]
fn test_property_supported_container_name() {
    assert!(is_property_supported("container-name", "sidebar"));
    assert!(is_property_supported("container-name", "anything"));
}

#[test]
fn test_property_supported_transform() {
    assert!(is_property_supported("transform", "translateX(10px)"));
    assert!(is_property_supported("transform", "none"));
    assert!(!is_property_supported("transform", "invalid"));
}

#[test]
fn test_property_supported_background() {
    assert!(is_property_supported("background", "red"));
    assert!(is_property_supported("background-image", "linear-gradient(red, blue)"));
}

#[test]
fn test_property_supported_scroll_snap() {
    assert!(is_property_supported("scroll-snap-type", "x mandatory"));
    assert!(is_property_supported("scroll-snap-align", "start"));
    assert!(is_property_supported("scroll-snap-stop", "always"));
}

#[test]
fn test_property_supported_scroll_margin() {
    assert!(is_property_supported("scroll-margin-top", "10px"));
    assert!(is_property_supported("scroll-margin-right", "5px"));
}

#[test]
fn test_property_supported_scroll_padding() {
    assert!(is_property_supported("scroll-padding-top", "10px"));
    assert!(is_property_supported("scroll-padding-bottom", "auto"));
}

#[test]
fn test_property_supported_unknown() {
    assert!(!is_property_supported("unknown-property", "value"));
    assert!(!is_property_supported("custom-prop", "something"));
}

// ═══════════════════════════════════════════════════════════════════════
// is_element helper
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_is_element_with_element() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();
    assert!(is_element(&doc, el));
}

#[test]
fn test_is_element_with_text() {
    let mut doc = Document::new();
    let root = doc.root();
    let text = doc.create_text_node("hello");
    doc.append_child(root, text).unwrap();
    assert!(!is_element(&doc, text));
}

#[test]
fn test_is_element_with_root() {
    let doc = Document::new();
    assert!(!is_element(&doc, doc.root()));
}
