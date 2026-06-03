// Matcher module - uncovered paths test
use super::super::*;
use zero_css_parser::ast::{
    AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector, PseudoClassSelector, Selector,
    SubclassSelector, TypeSelector,
};
use zero_css_parser::media_query::{
    MediaContext, MediaType, PointerValue, PrefersColorSchemeValue, ReducedMotionValue,
};
use zero_dom::Document;

// ── is_valid_selector_parse edge cases ──

/// 测试无效选择器：连续组合器（>>>）
#[test]
fn test_valid_selector_parse_consecutive_combinators() {
    assert!(!is_valid_selector_parse(">>>", &[]), ">>> should be invalid");
}

/// 测试无效选择器：> + 组合器
#[test]
fn test_valid_selector_parse_gt_plus() {
    assert!(!is_valid_selector_parse("> + div", &[]), "> + should be invalid");
}

/// 测试无效选择器：~ > 组合器
#[test]
fn test_valid_selector_parse_gt_tilde() {
    assert!(!is_valid_selector_parse("~ > span", &[]), "~ > should be invalid");
}

/// 测试无效选择器：以组合器开头的选择器
#[test]
fn test_valid_selector_parse_starts_with_combinator() {
    assert!(!is_valid_selector_parse("> div", &[]), "> div should be invalid");
    assert!(!is_valid_selector_parse("+ span", &[]), "+ span should be invalid");
    assert!(!is_valid_selector_parse("~ p", &[]), "~ p should be invalid");
}

/// 测试有效选择器
#[test]
fn test_valid_selector_parse_valid() {
    let css = "div { }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    if let Some(zero_css_parser::ast::Rule::Style(style_rule)) = stylesheet.rules.first() {
        assert!(
            is_valid_selector_parse("div", &style_rule.selectors),
            "div should be valid"
        );
    }
}

/// 测试空选择器
#[test]
fn test_valid_selector_parse_empty() {
    assert!(!is_valid_selector_parse("", &[]), "empty selector should be invalid");
}

/// 测试带括号的有效选择器
#[test]
fn test_valid_selector_parse_with_parens() {
    let css = "div:not(.foo) > span { }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    if let Some(zero_css_parser::ast::Rule::Style(style_rule)) = stylesheet.rules.first() {
        assert!(
            is_valid_selector_parse("div:not(.foo) > span", &style_rule.selectors),
            "div:not(.foo) > span should be valid"
        );
    }
}

// ── 复杂选择器组合 ──

/// 测试带多个子类选择器的复合选择器
#[test]
fn test_multiple_subclass_selectors() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "id", "test");
    doc.set_attribute(el, "class", "foo bar");
    doc.set_attribute(el, "data-x", "y");
    doc.append_child(root, el).unwrap();

    // div#test.foo.bar[data-x="y"]
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![
                        SubclassSelector::Id("test".to_string()),
                        SubclassSelector::Class("foo".to_string()),
                        SubclassSelector::Class("bar".to_string()),
                        SubclassSelector::Attribute(AttributeSelector {
                            name: "data-x".to_string(),
                            matcher: AttributeMatcher::Exact("y".to_string()),
                        }),
                    ],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

/// 测试空类列表
#[test]
fn test_empty_class_list() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "id", "test");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::Id("test".to_string())],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

// ── :has() 复杂情况测试 ──

/// 测试 :has() 带直接子元素选择器
#[test]
fn test_has_direct_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    // div:has(> span)
    let inner_sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Universal),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Child),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        inner_sel.clone(),
                    ]))],
                },
                None,
            )],
        },
    };

    // 首先检查基础匹配
    let inner_matches = matches_selector(&doc, child, &inner_sel);
    assert!(inner_matches, "> span should match span");

    assert!(matches_selector(&doc, parent, &sel), "div:has(> span) should match");
}

/// 测试 :has() 带多个后代选择器（OR 关系）
#[test]
fn test_has_multiple_descendants() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child1 = doc.create_element("span");
    let child2 = doc.create_element("p");
    let grandchild = doc.create_element("a");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();
    doc.append_child(child1, grandchild).unwrap();

    // div:has(span, p)
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        make_tag_selector("span"),
                        make_tag_selector("p"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, parent, &sel));
}

// ── :only-child 和 :only-of-type 测试 ──

/// 测试 :only-child
#[test]
fn test_only_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let only = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, only).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "only-child".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, only, &sel));
}

/// 测试 :only-of-type
#[test]
fn test_only_of_type() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let only = doc.create_element("p");
    let other_type = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, only).unwrap();
    doc.append_child(parent, other_type).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "only-of-type".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, only, &sel));
}

// ── 属性选择器 Includes 测试 ──

/// 测试属性选择器 Includes（空格分隔）
#[test]
fn test_attribute_includes() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "foo bar baz");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "class".to_string(),
                        matcher: AttributeMatcher::Includes("bar".to_string()),
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

/// 测试属性选择器 Includes（不匹配）
#[test]
fn test_attribute_includes_no_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "foo baz");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "class".to_string(),
                        matcher: AttributeMatcher::Includes("bar".to_string()),
                    })],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ── 匹配函数边界条件 ──

/// 测试 matches_selector 空选择器
#[test]
fn test_matches_selector_empty() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector { parts: vec![] },
    };
    assert!(!matches_selector(&doc, el, &sel));
}

// ── collect_from_rules 边界条件 ──

// ── @media 规则的特殊情况测试 ──

/// 测试 @media 规则没有媒体上下文时不应用
#[test]
fn test_media_rule_no_context() {
    let (doc, nodes) = super::coverage::make_nested_dom();
    let child1 = nodes[2]; // span

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(min-width: 400px)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("span")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "red".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    // 不提供媒体上下文，@media 规则不应该应用
    let decls = collect_matching_declarations(&doc, child1, &stylesheets);
    assert!(decls.is_empty());
}

/// 测试 @media 规则逗号分隔的查询（OR 关系）
#[test]
fn test_media_rule_comma_queries() {
    let (doc, nodes) = super::coverage::make_nested_dom();
    let child1 = nodes[2]; // span

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::At(zero_css_parser::ast::AtRule {
            name: "media".to_string(),
            prelude: "(max-width: 400px), (prefers-color-scheme: dark)".to_string(),
            body: zero_css_parser::ast::AtRuleBody::Block(vec![Rule::Style(StyleRule {
                selectors: vec![make_tag_selector("span")],
                declarations: vec![Declaration {
                    property: "color".to_string(),
                    value: "blue".to_string(),
                    important: false,
                }],
            })]),
        })],
    }];

    // 视口宽度为 800px，不是 dark 模式 - 两个查询都不匹配
    let media_ctx = MediaContext {
        viewport_width: 800.0,
        viewport_height: 600.0,
        media_type: MediaType::Screen,
        prefers_color_scheme: PrefersColorSchemeValue::Light,
        prefers_reduced_motion: ReducedMotionValue::NoPreference,
        pointer_type: PointerValue::Fine,
        resolution_dpi: 96.0,
    };
    let decls = collect_matching_declarations_with_media(&doc, child1, &stylesheets, Some(&media_ctx), None);
    assert!(decls.is_empty());
}

// ── collect_from_rules 边界条件 ──

/// 测试空的规则列表
#[test]
fn test_collect_from_empty_rules() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let decls = collect_matching_declarations(&doc, el, &[]);
    assert!(decls.is_empty());
}

/// 测试没有匹配选择器的规则
#[test]
fn test_collect_no_matching_selector() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    use zero_css_parser::ast::{Declaration, Rule, StyleRule};
    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("span")], // 不匹配 div
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    }];

    let decls = collect_matching_declarations(&doc, el, &stylesheets);
    assert!(decls.is_empty());
}

// 辅助函数
pub(super) fn make_tag_selector(tag: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag(tag.to_string())),
                    subclass_selectors: vec![],
                },
                None,
            )],
        },
    }
}
