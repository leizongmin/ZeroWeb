// Test file split from matcher.rs — container, supports, and edge case tests
use super::super::*;
use super::core::*;
use zero_css_parser::ast::{
    AttrCaseModifier, AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector,
    PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
};
use zero_dom::Document;

// ═══════════════════════════════════════════════════════════════════
// 扩展测试 — 新增伪类选择器匹配
// ═══════════════════════════════════════════════════════════════════

/// 测试 :only-child 匹配。
#[test]
fn test_only_child() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let only = doc.create_element("span");
    let _ = doc.append_child(parent, only);

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "only-child".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, only, &sel));

    // 添加第二个子元素后不再匹配
    let sibling = doc.create_element("p");
    let _ = doc.append_child(parent, sibling);
    assert!(!matches_selector(&doc, only, &sel));
}

/// 测试 :first-of-type 匹配。
#[test]
fn test_first_of_type() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let first = doc.create_element("span");
    let second = doc.create_element("span");
    let p = doc.create_element("p");
    let _ = doc.append_child(parent, first);
    let _ = doc.append_child(parent, p);
    let _ = doc.append_child(parent, second);

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "first-of-type".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, first, &sel));
    assert!(!matches_selector(&doc, second, &sel));
}

/// 测试 :last-of-type 匹配。
#[test]
fn test_last_of_type() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let first = doc.create_element("span");
    let second = doc.create_element("span");
    let p = doc.create_element("p");
    let _ = doc.append_child(parent, first);
    let _ = doc.append_child(parent, second);
    let _ = doc.append_child(parent, p);

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "last-of-type".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, first, &sel));
    assert!(matches_selector(&doc, second, &sel));
}

/// 测试 :only-of-type 匹配。
#[test]
fn test_only_of_type() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let only_p = doc.create_element("p");
    let span = doc.create_element("span");
    let _ = doc.append_child(parent, only_p);
    let _ = doc.append_child(parent, span);

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("p".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "only-of-type".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, only_p, &sel));

    // 添加第二个 p 后不再匹配
    let second_p = doc.create_element("p");
    let _ = doc.append_child(parent, second_p);
    assert!(!matches_selector(&doc, only_p, &sel));
}

/// 测试 :nth-last-child() 匹配（从末尾计数）。
#[test]
fn test_nth_last_child() {
    use zero_css_parser::ast::NthPattern;
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let c1 = doc.create_element("span");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("span");
    let _ = doc.append_child(parent, c1);
    let _ = doc.append_child(parent, c2);
    let _ = doc.append_child(parent, c3);

    // :nth-last-child(1) 应匹配最后一个
    let sel_last = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthLastChild(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, c1, &sel_last));
    assert!(!matches_selector(&doc, c2, &sel_last));
    assert!(matches_selector(&doc, c3, &sel_last));

    // :nth-last-child(2) 应匹配倒数第二个
    let sel_second_last = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthLastChild(
                        NthPattern { a: 0, b: 2 },
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, c1, &sel_second_last));
    assert!(matches_selector(&doc, c2, &sel_second_last));
    assert!(!matches_selector(&doc, c3, &sel_second_last));
}

/// 测试 :nth-of-type() 匹配（按类型计数）。
#[test]
fn test_nth_of_type() {
    use zero_css_parser::ast::NthPattern;
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let s1 = doc.create_element("span");
    let p1 = doc.create_element("p");
    let s2 = doc.create_element("span");
    let p2 = doc.create_element("p");
    let _ = doc.append_child(parent, s1);
    let _ = doc.append_child(parent, p1);
    let _ = doc.append_child(parent, s2);
    let _ = doc.append_child(parent, p2);

    // :nth-of-type(2) 在 span 中应匹配 s2（第二个 span）
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthOfType(
                        NthPattern { a: 0, b: 2 },
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, s1, &sel));
    assert!(matches_selector(&doc, s2, &sel));

    // :nth-of-type(1) 在 p 中应匹配 p1
    let sel_p = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("p".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthOfType(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, p1, &sel_p));
    assert!(!matches_selector(&doc, p2, &sel_p));
}

/// 测试 :nth-last-of-type() 匹配（从末尾按类型计数）。
#[test]
fn test_nth_last_of_type() {
    use zero_css_parser::ast::NthPattern;
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let s1 = doc.create_element("span");
    let p1 = doc.create_element("p");
    let s2 = doc.create_element("span");
    let _ = doc.append_child(parent, s1);
    let _ = doc.append_child(parent, p1);
    let _ = doc.append_child(parent, s2);

    // :nth-last-of-type(1) 在 span 中应匹配 s2（最后一个 span）
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthLastOfType(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, s1, &sel));
    assert!(matches_selector(&doc, s2, &sel));
}

/// 测试 :nth-of-type(odd) 匹配奇数位置的同类型元素。
#[test]
fn test_nth_of_type_odd() {
    use zero_css_parser::ast::NthPattern;
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let s1 = doc.create_element("span");
    let s2 = doc.create_element("span");
    let s3 = doc.create_element("span");
    let s4 = doc.create_element("span");
    let _ = doc.append_child(parent, s1);
    let _ = doc.append_child(parent, s2);
    let _ = doc.append_child(parent, s3);
    let _ = doc.append_child(parent, s4);

    // odd = 2n+1
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthOfType(
                        NthPattern { a: 2, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, s1, &sel)); // 1st
    assert!(!matches_selector(&doc, s2, &sel)); // 2nd
    assert!(matches_selector(&doc, s3, &sel)); // 3rd
    assert!(!matches_selector(&doc, s4, &sel)); // 4th
}

// ═══════════════════════════════════════════════════════════════════
// ContainerContext 和 @container 规则匹配测试
// ═══════════════════════════════════════════════════════════════════

#[test]
fn test_container_context_creation() {
    let ctx = ContainerContext::new();
    assert_eq!(ctx.container_width, None);
    assert_eq!(ctx.container_height, None);

    let ctx = ContainerContext::with_size(400.0, 600.0);
    assert_eq!(ctx.container_width, Some(400.0));
    assert_eq!(ctx.container_height, Some(600.0));
}

#[test]
fn test_container_context_default() {
    let ctx = ContainerContext::default();
    assert_eq!(ctx.container_width, None);
    assert_eq!(ctx.container_height, None);
}

#[test]
fn test_container_rule_collects_declarations() {
    use zero_css_parser::ast::{ContainerCondition, ContainerRule, ContainerSizeCondition, Declaration, StyleRule};

    let (doc, _html, _body, _div, p) = make_test_dom();

    let container_rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "min-width".to_string(),
            value: "400px".to_string(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![zero_css_parser::ast::Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("p")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    };

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![zero_css_parser::ast::Rule::Container(container_rule)],
    }];

    // 无容器上下文时，@container 规则不应用
    let results = collect_matching_declarations(&doc, p, &stylesheets);
    assert_eq!(results.len(), 0, "@container should not apply without context");

    // 容器宽度 >= 400px 时，规则应用
    let ctx = ContainerContext::with_size(500.0, 600.0);
    let results = collect_matching_declarations_with_media(&doc, p, &stylesheets, None, Some(&ctx));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "color");
    assert_eq!(results[0].1, "red");

    // 容器宽度 < 400px 时，规则不应用
    let ctx_small = ContainerContext::with_size(300.0, 600.0);
    let results = collect_matching_declarations_with_media(&doc, p, &stylesheets, None, Some(&ctx_small));
    assert_eq!(results.len(), 0, "@container min-width:400px should not apply at 300px");
}

// ═══════════════════════════════════════════════════════════════════
// 新增匹配器边界条件测试
// ═══════════════════════════════════════════════════════════════════

#[test]
/// :has() 带后代组合器
fn test_has_with_descendant_combinator() {
    let mut doc = Document::new();
    let root = doc.root();
    let grandparent = doc.create_element("section");
    doc.append_child(root, grandparent).unwrap();
    let parent = doc.create_element("div");
    doc.append_child(grandparent, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "target");
    doc.append_child(parent, child).unwrap();

    // section:has(div .target)
    let inner_sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Descendant),
                ),
                (
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Class("target".to_string())],
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
                    type_selector: Some(TypeSelector::Tag("section".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![inner_sel]))],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, grandparent, &sel),
        "section containing div .target should match :has(div .target)"
    );
}

#[test]
/// :not() 带多个选择器
fn test_not_with_multiple_selectors() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    // :not(div, span)
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Not(vec![
                        make_tag_selector("div"),
                        make_tag_selector("span"),
                    ]))],
                },
                None,
            )],
        },
    };

    // div 不匹配 :not(div, span)
    assert!(!matches_selector(&doc, div, &sel));
}

#[test]
/// :is() 匹配 vs :where() 匹配（两者匹配逻辑相同，区别在特异性）
fn test_is_and_where_matching_logic() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    let is_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Is(vec![
                        make_tag_selector("div"),
                        make_tag_selector("span"),
                    ]))],
                },
                None,
            )],
        },
    };

    let where_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Where(vec![
                        make_tag_selector("div"),
                        make_tag_selector("span"),
                    ]))],
                },
                None,
            )],
        },
    };

    // 两者的匹配逻辑相同
    assert!(matches_selector(&doc, div, &is_sel));
    assert!(matches_selector(&doc, div, &where_sel));
}

#[test]
/// 通用选择器匹配所有元素
fn test_universal_selector_matches_all() {
    let (doc, _html, _body, div, p) = make_test_dom();
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
    assert!(matches_selector(&doc, div, &sel));
    assert!(matches_selector(&doc, p, &sel));
}

#[test]
/// 属性 Includes matcher 匹配空格分隔值
fn test_attribute_includes_match() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "foo bar baz");
    let root = doc.root();
    doc.append_child(root, elem).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "class".to_string(),
                        matcher: AttributeMatcher::Includes("bar".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, elem, &sel));
}

#[test]
/// :nth-child(1) 匹配第一个子元素
fn test_nth_child_first() {
    let mut doc = Document::new();
    let parent = doc.create_element("ul");
    let li1 = doc.create_element("li");
    let li2 = doc.create_element("li");
    let _ = doc.append_child(parent, li1);
    let _ = doc.append_child(parent, li2);

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("li".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(
                        zero_css_parser::ast::NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, li1, &sel));
    assert!(!matches_selector(&doc, li2, &sel));
}

// ═══════════════════════════════════════════════════════════════════
// justify-all declaration 展开（CSS Text 3 §7.1）
// ═══════════════════════════════════════════════════════════════════

/// 辅助：构建 tag selector。
fn tag_sel(tag: &str) -> Selector {
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

/// `text-align: justify-all` 在 declaration 收集层展开为两个 author declaration：
/// `text-align: justify` + `text-align-last: justify`（R957）。
/// apply 层单点特判会被 cascade「text-align-last 无 author declaration → 继承」覆盖（R956）。
#[test]
fn test_text_align_justify_all_expands_to_two_declarations() {
    let mut doc = Document::new();
    let p = doc.create_element("p");

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![zero_css_parser::ast::Rule::Style(zero_css_parser::ast::StyleRule {
            selectors: vec![tag_sel("p")],
            declarations: vec![zero_css_parser::ast::Declaration {
                property: "text-align".to_string(),
                value: "justify-all".to_string(),
                important: false,
            }],
        })],
    }];

    let results = collect_matching_declarations(&doc, p, &stylesheets);
    // 展开为 2 个 declaration
    assert_eq!(results.len(), 2, "justify-all should expand to 2 declarations");
    assert_eq!(results[0].0, "text-align");
    assert_eq!(results[0].1, "justify");
    assert_eq!(results[1].0, "text-align-last");
    assert_eq!(results[1].1, "justify");
    // 两者同 specificity / layer（来自同一源 declaration）
    assert_eq!(results[0].3, results[1].3, "same specificity");
    assert_eq!(results[0].4, results[1].4, "same layer");
}

/// `text-align: justify-all` 大小写不敏感 + 容忍空白。
#[test]
fn test_text_align_justify_all_case_insensitive_and_trim() {
    let mut doc = Document::new();
    let p = doc.create_element("p");

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![zero_css_parser::ast::Rule::Style(zero_css_parser::ast::StyleRule {
            selectors: vec![tag_sel("p")],
            declarations: vec![zero_css_parser::ast::Declaration {
                property: "TEXT-ALIGN".to_string(),
                value: "  JUSTIFY-ALL  ".to_string(),
                important: false,
            }],
        })],
    }];

    let results = collect_matching_declarations(&doc, p, &stylesheets);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, "text-align");
    assert_eq!(results[1].0, "text-align-last");
}

/// 普通 `text-align: justify` 不展开（仅 justify-all 展开）。
#[test]
fn test_text_align_justify_not_expanded() {
    let mut doc = Document::new();
    let p = doc.create_element("p");

    let stylesheets = vec![zero_css_parser::Stylesheet {
        rules: vec![zero_css_parser::ast::Rule::Style(zero_css_parser::ast::StyleRule {
            selectors: vec![tag_sel("p")],
            declarations: vec![zero_css_parser::ast::Declaration {
                property: "text-align".to_string(),
                value: "justify".to_string(),
                important: false,
            }],
        })],
    }];

    let results = collect_matching_declarations(&doc, p, &stylesheets);
    assert_eq!(results.len(), 1, "plain justify should not expand");
    assert_eq!(results[0].0, "text-align");
    assert_eq!(results[0].1, "justify");
}
