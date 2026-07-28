//! Matcher 额外覆盖率测试：is_property_supported、container、supports 等。

use super::super::*;
use zero_css_parser::ast::{
    AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector, ContainerCondition,
    ContainerRule, ContainerSizeCondition, Declaration, LayerRule, NthPattern, PseudoClassSelector,
    PseudoElementSelector, Rule, Selector, StyleRule, SubclassSelector, TypeSelector,
};
use zero_css_parser::media_query::{
    MediaContext, MediaType, PointerValue, PrefersColorSchemeValue, ReducedMotionValue,
};
use zero_dom::Document;

// ═══════════════════════════════════════════════════════════════════════
// is_property_supported 边缘情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_property_supported_display() {
    assert!(is_property_supported("display", "block"));
    assert!(is_property_supported("display", "flex"));
    assert!(is_property_supported("display", "grid"));
    assert!(!is_property_supported("display", "invalid"));
}

#[test]
fn test_property_supported_position() {
    assert!(is_property_supported("position", "relative"));
    assert!(is_property_supported("position", "absolute"));
    assert!(!is_property_supported("position", "invalid"));
}

#[test]
fn test_property_supported_overflow() {
    assert!(is_property_supported("overflow", "hidden"));
    assert!(is_property_supported("overflow-x", "scroll"));
    assert!(!is_property_supported("overflow", "invalid"));
}

#[test]
fn test_property_supported_visibility() {
    assert!(is_property_supported("visibility", "visible"));
    assert!(!is_property_supported("visibility", "invalid"));
}

#[test]
fn test_property_supported_box_sizing() {
    assert!(is_property_supported("box-sizing", "border-box"));
    assert!(!is_property_supported("box-sizing", "invalid"));
}

#[test]
fn test_property_supported_flex() {
    assert!(is_property_supported("flex-direction", "row"));
    assert!(is_property_supported("flex-wrap", "wrap"));
    assert!(!is_property_supported("flex-direction", "invalid"));
}

#[test]
fn test_property_supported_alignment() {
    assert!(is_property_supported("justify-content", "center"));
    assert!(is_property_supported("align-items", "flex-start"));
    assert!(is_property_supported("align-self", "stretch"));
    assert!(is_property_supported("justify-self", "start"));
    assert!(!is_property_supported("justify-content", "invalid"));
}

#[test]
fn test_property_supported_font() {
    assert!(is_property_supported("font-weight", "bold"));
    assert!(is_property_supported("font-style", "italic"));
    assert!(!is_property_supported("font-weight", "invalid"));
}

#[test]
fn test_property_supported_color() {
    assert!(is_property_supported("color", "red"));
    assert!(is_property_supported("background-color", "#fff"));
    assert!(is_property_supported("border-color", "rgb(0,0,0)"));
    assert!(is_property_supported("border-top-color", "blue"));
    assert!(is_property_supported("border-right-color", "green"));
    assert!(is_property_supported("border-bottom-color", "yellow"));
    assert!(is_property_supported("border-left-color", "purple"));
    assert!(!is_property_supported("color", "invalid-color-xyz"));
}

#[test]
fn test_property_supported_length_properties() {
    assert!(is_property_supported("width", "100px"));
    assert!(is_property_supported("height", "50em"));
    assert!(is_property_supported("min-width", "10px"));
    assert!(is_property_supported("max-width", "200px"));
    assert!(is_property_supported("min-height", "10px"));
    assert!(is_property_supported("max-height", "200px"));
    assert!(is_property_supported("margin", "10px"));
    assert!(is_property_supported("margin-top", "5px"));
    assert!(is_property_supported("margin-right", "5px"));
    assert!(is_property_supported("margin-bottom", "5px"));
    assert!(is_property_supported("margin-left", "5px"));
    assert!(is_property_supported("padding", "10px"));
    assert!(is_property_supported("padding-top", "5px"));
    assert!(is_property_supported("padding-right", "5px"));
    assert!(is_property_supported("padding-bottom", "5px"));
    assert!(is_property_supported("padding-left", "5px"));
    assert!(is_property_supported("gap", "10px"));
    assert!(is_property_supported("top", "0px"));
    assert!(is_property_supported("right", "0px"));
    assert!(is_property_supported("bottom", "0px"));
    assert!(is_property_supported("left", "0px"));
    assert!(is_property_supported("border-top-width", "1px"));
    assert!(is_property_supported("border-right-width", "1px"));
    assert!(is_property_supported("border-bottom-width", "1px"));
    assert!(is_property_supported("border-left-width", "1px"));
    assert!(is_property_supported("border-top-left-radius", "5px"));
    assert!(is_property_supported("border-top-right-radius", "5px"));
    assert!(is_property_supported("border-bottom-right-radius", "5px"));
    assert!(is_property_supported("border-bottom-left-radius", "5px"));
    assert!(!is_property_supported("width", "invalid"));
}

#[test]
fn test_property_supported_transform() {
    assert!(is_property_supported("transform", "translate(10px, 20px)"));
    assert!(!is_property_supported("transform", "invalid-transform"));
}

#[test]
fn test_property_supported_background() {
    assert!(is_property_supported("background", "red"));
    assert!(is_property_supported("background-image", "linear-gradient(red, blue)"));
}

#[test]
fn test_property_supported_scroll_snap() {
    assert!(is_property_supported("scroll-snap-type", "mandatory"));
    assert!(is_property_supported("scroll-snap-align", "center"));
    assert!(is_property_supported("scroll-snap-stop", "always"));
    assert!(is_property_supported("scroll-margin-top", "10px"));
    assert!(is_property_supported("scroll-margin-right", "10px"));
    assert!(is_property_supported("scroll-margin-bottom", "10px"));
    assert!(is_property_supported("scroll-margin-left", "10px"));
    assert!(is_property_supported("scroll-padding-top", "auto"));
    assert!(is_property_supported("scroll-padding-right", "10px"));
    assert!(is_property_supported("scroll-padding-bottom", "10px"));
    assert!(is_property_supported("scroll-padding-left", "10px"));
    assert!(!is_property_supported("scroll-snap-type", "invalid"));
}

#[test]
fn test_property_supported_container() {
    assert!(is_property_supported("container-type", "inline-size"));
    assert!(is_property_supported("container-name", "sidebar"));
    assert!(is_property_supported("container-name", "anything"));
    assert!(!is_property_supported("container-type", "invalid"));
}

#[test]
fn test_property_supported_unknown() {
    assert!(!is_property_supported("unknown-property", "value"));
    assert!(!is_property_supported("custom-var", "red"));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_container_condition — 无容器上下文
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_condition_no_context() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "100px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    assert!(!evaluate_container_condition(&rule, None));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_container_condition — 比较运算符全覆盖
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_comparison_gte() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "300px".to_string(),
            operator: Some(">=".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
    let ctx_exact = ContainerContext::with_size(300.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx_exact)));
    let ctx_small = ContainerContext::with_size(200.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx_small)));
}

#[test]
fn test_container_comparison_lt() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "300px".to_string(),
            operator: Some("<".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(200.0, 600.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
    let ctx_equal = ContainerContext::with_size(300.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx_equal)));
}

#[test]
fn test_container_comparison_lte() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "height".to_string(),
            value: "500px".to_string(),
            operator: Some("<=".to_string()),
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(800.0, 400.0);
    assert!(evaluate_container_condition(&rule, Some(&ctx)));
}

// ═══════════════════════════════════════════════════════════════════════
// evaluate_container_condition — inline-size / block-size 特性
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_inline_size_feature() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-inline-size".to_string(),
            value: "300px".to_string(),
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
fn test_container_block_size_feature() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-block-size".to_string(),
            value: "400px".to_string(),
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
fn test_container_unknown_feature() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "unknown-feature".to_string(),
            value: "300px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(500.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

#[test]
fn test_container_invalid_length_value() {
    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".to_string(),
            value: "not-a-length".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };
    let ctx = ContainerContext::with_size(500.0, 600.0);
    assert!(!evaluate_container_condition(&rule, Some(&ctx)));
}

// ═══════════════════════════════════════════════════════════════════════
// PseudoElement 返回 false
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_pseudo_element_always_false() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoElement(PseudoElementSelector::Standard(
                        "before".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :lang() 伪类返回 false
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_lang_no_lang_attribute_does_not_match() {
    // :lang(en) 对无 lang 属性（且无 lang 祖先）的元素不匹配（CSS 2.1 §5.11.4）。
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Lang(
                        "en".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// @supports 选择器条件
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_selector_condition() {
    use zero_css_parser::ast::SupportsCondition;

    // 有效选择器
    assert!(evaluate_supports_condition(&SupportsCondition::Selector(
        "div".to_string()
    )));
    assert!(evaluate_supports_condition(&SupportsCondition::Selector(
        ".class".to_string()
    )));
    assert!(evaluate_supports_condition(&SupportsCondition::Selector(
        "#id".to_string()
    )));

    // 无效选择器（以组合器开头）
    assert!(!evaluate_supports_condition(&SupportsCondition::Selector(
        "> div".to_string()
    )));
    assert!(!evaluate_supports_condition(&SupportsCondition::Selector(
        "+ span".to_string()
    )));

    // 空选择器
    assert!(!evaluate_supports_condition(&SupportsCondition::Selector(
        "".to_string()
    )));
}

// ═══════════════════════════════════════════════════════════════════════
// @container 规则在 collect_from_rules 中
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_container_rule_matching() {
    let mut doc = Document::new();
    let root = doc.root();
    let span = doc.create_element("span");
    doc.append_child(root, span).unwrap();

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
            selectors: vec![super::uncovered_paths::make_tag_selector("span")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "blue".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Container(container_rule)],
    }];

    // 有容器上下文且匹配
    let ctx = ContainerContext::with_size(500.0, 600.0);
    let decls = collect_matching_declarations_with_media(&doc, span, &stylesheets, None, Some(&ctx));
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].0, "color");

    // 无容器上下文 → 不匹配
    let decls_no_ctx = collect_matching_declarations(&doc, span, &stylesheets);
    assert!(decls_no_ctx.is_empty());

    // 容器上下文尺寸不足 → 不匹配
    let ctx_small = ContainerContext::with_size(200.0, 600.0);
    let decls_small = collect_matching_declarations_with_media(&doc, span, &stylesheets, None, Some(&ctx_small));
    assert!(decls_small.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// @supports 规则在 collect_from_rules 中
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_supports_rule_in_collect() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    use zero_css_parser::ast::{SupportsCondition, SupportsRule};
    let supports_rule = SupportsRule {
        condition: SupportsCondition::Property("display".to_string(), "grid".to_string()),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![super::uncovered_paths::make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "display".to_string(),
                value: "grid".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(supports_rule)],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    assert_eq!(decls.len(), 1);
    assert_eq!(decls[0].0, "display");
}

#[test]
fn test_supports_rule_not_matching() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    use zero_css_parser::ast::{SupportsCondition, SupportsRule};
    let supports_rule = SupportsRule {
        condition: SupportsCondition::Property("display".to_string(), "invalid-display".to_string()),
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![super::uncovered_paths::make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "display".to_string(),
                value: "grid".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Supports(supports_rule)],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    assert!(decls.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 属性不存在 → matches_attribute 返回 false
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_selector_missing_attr() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "data-missing".to_string(),
                        matcher: AttributeMatcher::Exists,
                    })],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// 非元素节点 → matches_type 返回 false
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_matches_type_text_node() {
    let mut doc = Document::new();
    let root = doc.root();
    let text = doc.create_text_node("hello");
    doc.append_child(root, text).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Universal),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, text, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// Universal 选择器匹配
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_universal_selector_matches() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Universal),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :nth-child, :nth-last-child, :nth-of-type, :nth-last-of-type 集成
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_child_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("ul");
    let li1 = doc.create_element("li");
    let li2 = doc.create_element("li");
    let li3 = doc.create_element("li");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, li1).unwrap();
    doc.append_child(parent, li2).unwrap();
    doc.append_child(parent, li3).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(
                        NthPattern { a: 2, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, li1, &sel)); // 2*0+1=1 ✓
    assert!(!matches_selector(&doc, li2, &sel)); // 2 ≠ 2n+1
    assert!(matches_selector(&doc, li3, &sel)); // 2*1+1=3 ✓
}

#[test]
fn test_nth_last_child_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("ul");
    let li1 = doc.create_element("li");
    let li2 = doc.create_element("li");
    let li3 = doc.create_element("li");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, li1).unwrap();
    doc.append_child(parent, li2).unwrap();
    doc.append_child(parent, li3).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthLastChild(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, li1, &sel)); // 第3个从末尾
    assert!(!matches_selector(&doc, li2, &sel)); // 第2个从末尾
    assert!(matches_selector(&doc, li3, &sel)); // 第1个从末尾 ✓
}

#[test]
fn test_nth_last_of_type_integration() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let p2 = doc.create_element("p");
    let span = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, p2).unwrap();

    // p:nth-last-of-type(1) → 最后一个 p 类型
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthLastOfType(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, p1, &sel)); // 第2个 p 从末尾
    assert!(matches_selector(&doc, p2, &sel)); // 第1个 p 从末尾 ✓
}

// ═══════════════════════════════════════════════════════════════════════
// @media 规则带匹配上下文
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_media_rule_matching_context() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("span");
    doc.append_child(root, el).unwrap();

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(min-width: 400px)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![super::uncovered_paths::make_tag_selector("span")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    let media_ctx = MediaContext {
        viewport_width: 800.0,
        viewport_height: 600.0,
        media_type: MediaType::Screen,
        prefers_color_scheme: PrefersColorSchemeValue::Light,
        prefers_reduced_motion: ReducedMotionValue::NoPreference,
        pointer_type: PointerValue::Fine,
        resolution_dpi: 96.0,
    };
    let decls = collect_matching_declarations_with_media(&doc, el, &stylesheets, Some(&media_ctx), None);
    assert_eq!(decls.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 非 media 的 AtRule（如 @font-face）无条件递归
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_non_media_at_rule_unconditional_recurse() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "font-face".to_string(),
            prelude: "".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![super::uncovered_paths::make_tag_selector("div")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    // 非 @media 的通用 AtRule（未知 at-rule）body 不得作为样式应用（CSS：未知 at-rule
    // 整体忽略）。R2142 修正：旧实现无条件递归→body 内规则泄漏应用；现 body 不参与 cascade。
    // 注：真实 @font-face 经解析为 Rule::FontFace 专属变体，不进此通用 Rule::At 分支；
    // 此处构造通用 Rule::At(name="font-face") 模拟未知 at-rule 带 body 的场景。
    assert_eq!(decls.len(), 0, "unknown/non-media generic at-rule body must not apply");
}

// ═══════════════════════════════════════════════════════════════════════
// :not() 伪类
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_not_pseudo_class() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(root, span).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Not(vec![
                        super::uncovered_paths::make_tag_selector("div"),
                    ]))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, div, &sel)); // div 被 :not(div) 排除
    assert!(matches_selector(&doc, span, &sel)); // span 不被排除
}

// ═══════════════════════════════════════════════════════════════════════
// :is() 和 :where() 伪类
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_is_pseudo_class() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, div).unwrap();
    doc.append_child(root, span).unwrap();
    doc.append_child(root, p).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Is(vec![
                        super::uncovered_paths::make_tag_selector("div"),
                        super::uncovered_paths::make_tag_selector("span"),
                    ]))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, div, &sel));
    assert!(matches_selector(&doc, span, &sel));
    assert!(!matches_selector(&doc, p, &sel));
}

#[test]
fn test_where_pseudo_class() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Where(vec![
                        super::uncovered_paths::make_tag_selector("div"),
                    ]))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, div, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :first-of-type 和 :last-of-type
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_first_of_type() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let span = doc.create_element("span");
    let p2 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, p2).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "first-of-type".to_string(),
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, p1, &sel)); // 第一个 p
    assert!(!matches_selector(&doc, p2, &sel)); // 第二个 p
    assert!(matches_selector(&doc, span, &sel)); // 唯一的 span
}

#[test]
fn test_last_of_type() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let span = doc.create_element("span");
    let p2 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, p2).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "last-of-type".to_string(),
                    ))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, p1, &sel)); // 第一个 p
    assert!(matches_selector(&doc, p2, &sel)); // 最后一个 p
    assert!(matches_selector(&doc, span, &sel)); // 唯一的 span
}

// ═══════════════════════════════════════════════════════════════════════
// 属性不存在 → matches_class 返回 false（非元素节点）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_class_on_text_node() {
    let mut doc = Document::new();
    let root = doc.root();
    let text = doc.create_text_node("hello");
    doc.append_child(root, text).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class("test".to_string())],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, text, &sel));
}
