// Test file split from matcher.rs — pseudo-class selector tests
use super::super::*;
use super::core::*;
use zero_css_parser::ast::{
    AttrCaseModifier, AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector,
    PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
};
use zero_dom::{Document, NodeId};

#[test]
fn test_collect_matching_declarations() {
    use zero_css_parser::ast::{Declaration, Rule, StyleRule, Stylesheet};

    let (doc, _html, _body, div, _p) = make_test_dom();

    let stylesheets = vec![Stylesheet {
        rules: vec![Rule::Style(StyleRule {
            selectors: vec![make_tag_selector("div")],
            declarations: vec![Declaration {
                property: "color".to_string(),
                value: "red".to_string(),
                important: false,
            }],
        })],
    }];

    let results = collect_matching_declarations(&doc, div, &stylesheets);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, "color");
    assert_eq!(results[0].1, "red");
}

#[test]
fn test_next_sibling_combinator() {
    let (mut doc, _html, _body, div, _p) = make_test_dom();
    // 创建 p 的兄弟
    let span = doc.create_element("span");
    doc.append_child(div, span).unwrap();

    // p + span
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::NextSibling),
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
    assert!(matches_selector(&doc, span, &sel));
}

// ── 补充边界条件测试 ──

/// 测试通用兄弟组合器（~）：div ~ span 匹配 div 后面的 span 兄弟。
#[test]
fn test_subsequent_sibling_combinator() {
    let (mut doc, _html, _body, div, _p) = make_test_dom();
    // 在 div 后添加 span
    let body = doc.parent_node(div).unwrap();
    let span1 = doc.create_element("span");
    doc.append_child(body, span1).unwrap();
    let span2 = doc.create_element("span");
    doc.append_child(body, span2).unwrap();

    // div ~ span 应匹配 span1 和 span2
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::SubsequentSibling),
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
    assert!(matches_selector(&doc, span1, &sel), "span1 should match div ~ span");
    assert!(matches_selector(&doc, span2, &sel), "span2 should match div ~ span");
    // div 本身不应匹配
    assert!(!matches_selector(&doc, div, &sel), "div should not match div ~ span");
}

/// 测试 :last-child 伪类。
#[test]
fn test_matches_pseudo_last_child() {
    let (mut doc, _html, _body, div, _p) = make_test_dom();
    let body = doc.parent_node(div).unwrap();
    let span = doc.create_element("span");
    doc.append_child(body, span).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "last-child".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, span, &sel),
        "span (last child) should match :last-child"
    );
    assert!(
        !matches_selector(&doc, div, &sel),
        "div (not last child) should not match :last-child"
    );
}

/// 测试 :nth-child(2n) 匹配偶数位置。
#[test]
fn test_matches_nth_child_even() {
    let mut doc = Document::new();
    let root = doc.root();
    let body = doc.create_element("body");
    doc.append_child(root, body).unwrap();

    let items: Vec<NodeId> = (0..5)
        .map(|_| {
            let li = doc.create_element("li");
            doc.append_child(body, li).unwrap();
            li
        })
        .collect();

    // :nth-child(2n) 匹配第 2、4 个
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(
                        zero_css_parser::ast::NthPattern { a: 2, b: 0 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, items[0], &sel), "1st child should not match 2n");
    assert!(matches_selector(&doc, items[1], &sel), "2nd child should match 2n");
    assert!(!matches_selector(&doc, items[2], &sel), "3rd child should not match 2n");
    assert!(matches_selector(&doc, items[3], &sel), "4th child should match 2n");
}

/// 测试 :where() 伪类匹配。
#[test]
fn test_matches_where_pseudo() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Where(vec![
                        make_class_selector("container"),
                        make_class_selector("other"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, div, &sel),
        "div.container should match :where(.container, .other)"
    );
}

/// 测试属性选择器 DashMatch（lang 属性）。
#[test]
fn test_attribute_dash_match() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "lang", "en-US");
    let root = doc.root();
    doc.append_child(root, elem).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "lang".to_string(),
                        matcher: AttributeMatcher::DashMatch("en".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, elem, &sel),
        "lang='en-US' should match [lang|=en]"
    );
}

/// 测试属性选择器 Prefix。
#[test]
fn test_attribute_prefix_match() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "data-type", "button-primary");
    let root = doc.root();
    doc.append_child(root, elem).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "data-type".to_string(),
                        matcher: AttributeMatcher::Prefix("button".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, elem, &sel),
        "data-type='button-primary' should match [data-type^=button]"
    );
}

/// 测试属性选择器 Suffix。
#[test]
fn test_attribute_suffix_match() {
    let mut doc = Document::new();
    let elem = doc.create_element("a");
    doc.set_attribute(elem, "href", "https://example.com/page");
    let root = doc.root();
    doc.append_child(root, elem).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "href".to_string(),
                        matcher: AttributeMatcher::Suffix("/page".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, elem, &sel),
        "href ending with '/page' should match [href$='/page']"
    );
}

/// 测试属性选择器 Substring。
#[test]
fn test_attribute_substring_match() {
    let mut doc = Document::new();
    let elem = doc.create_element("a");
    doc.set_attribute(elem, "href", "https://example.com/docs/api");
    let root = doc.root();
    doc.append_child(root, elem).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "href".to_string(),
                        matcher: AttributeMatcher::Substring("example".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, elem, &sel),
        "href containing 'example' should match [href*=example]"
    );
}

/// 测试类型选择器大小写不敏感。
#[test]
fn test_tag_selector_case_insensitive() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = make_tag_selector("DIV");
    assert!(
        matches_selector(&doc, div, &sel),
        "DIV should match div (case insensitive)"
    );
}

/// 测试空选择器不匹配任何元素。
#[test]
fn test_empty_selector_no_match() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector { parts: vec![] },
    };
    assert!(!matches_selector(&doc, div, &sel), "empty selector should not match");
}

/// 测试 :not() 排除匹配。
#[test]
fn test_not_excludes_matching() {
    let (doc, _html, _body, _div, p) = make_test_dom();

    // p:not(.container) — p 没有 container 类，应匹配
    let sel_not_container = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("p".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Not(vec![
                        make_class_selector("container"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, p, &sel_not_container),
        "p without .container should match :not(.container)"
    );

    // p:not(.text) — p 有 text 类，不应匹配
    let sel_not_text = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("p".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Not(vec![
                        make_class_selector("text"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(
        !matches_selector(&doc, p, &sel_not_text),
        "p.text should not match :not(.text)"
    );
}

/// 测试伪元素选择器不匹配任何元素。
#[test]
fn test_pseudo_element_never_matches() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoElement(
                        zero_css_parser::ast::PseudoElementSelector::Standard("before".to_string()),
                    )],
                },
                None,
            )],
        },
    };
    assert!(
        !matches_selector(&doc, div, &sel),
        "pseudo-element should never match DOM elements"
    );
}

// ── :has() 伪类匹配测试 ──

/// 测试 :has(.child) 匹配拥有 .child 后代的父元素。
#[test]
fn test_has_descendant_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    // div:has(.child)
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        make_class_selector("child"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, parent, &sel),
        "div with .child descendant should match :has(.child)"
    );
}

/// 测试 :has(> .direct) 匹配拥有 .direct 直接子元素的父元素。
#[test]
fn test_has_direct_child_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "direct");
    doc.append_child(parent, child).unwrap();

    // div:has(> .direct) — parsed as * > .direct
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
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::Class("direct".to_string())],
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
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![inner_sel]))],
                },
                None,
            )],
        },
    };
    assert!(
        matches_selector(&doc, parent, &sel),
        "div with .direct child should match :has(> .direct)"
    );
}

/// 测试 :has(.absent) 不匹配没有 .absent 后代的父元素。
#[test]
fn test_has_no_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "other");
    doc.append_child(parent, child).unwrap();

    // div:has(.absent)
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        make_class_selector("absent"),
                    ]))],
                },
                None,
            )],
        },
    };
    assert!(
        !matches_selector(&doc, parent, &sel),
        "div without .absent descendant should not match :has(.absent)"
    );
}

// ===== 伪元素声明路由（R486：:before/:after generated-content 基础）=====

/// 端到端验证：CSS2 单冒号 `:before` 经解析器归为伪元素，matcher 把
/// `div:before { content: "X"; color: red }` 的声明路由到 div 的 before 伪元素槽，
/// 且不会落到 div 元素自身的样式上。
#[test]
fn test_pseudo_element_declaration_routing() {
    let (_doc, _html, _body, div, _p) = make_test_dom();
    let css = r#"div:before { content: "X"; color: red; }"#;
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);

    // 1. 解析器：:before 归为伪元素（尾部伪元素名 == "before"）
    use zero_css_parser::ast::Rule;
    let style_rule = match &stylesheet.rules[0] {
        Rule::Style(sr) => sr,
        _ => panic!("expected style rule"),
    };
    assert_eq!(selector_pseudo_element(&style_rule.selectors[0]), Some("before"));

    let stylesheets = [stylesheet];

    // 2. 伪元素声明收集：div 的 before 应收到 content + color
    let pseudo_decls = collect_pseudo_declarations_with_media(
        &_doc,
        div,
        &stylesheets,
        &build_stylesheet_index(&stylesheets),
        None,
        None,
        "before",
    );
    let mut got_content = false;
    let mut got_color = false;
    for (prop, val, _, _, _) in &pseudo_decls {
        if prop == "content" {
            assert!(val.contains('X'), "content value: {val}");
            got_content = true;
        }
        if prop == "color" {
            got_color = true;
        }
    }
    assert!(got_content, "before 伪元素应收到 content 声明");
    assert!(got_color, "before 伪元素应收到 color 声明");

    // 3. 元素自身不应收到这些声明（伪元素规则不作用于元素本体）
    let own_decls = collect_matching_declarations_with_media(
        &_doc,
        div,
        &stylesheets,
        &build_stylesheet_index(&stylesheets),
        None,
        None,
    );
    for (prop, _, _, _, _) in &own_decls {
        assert!(prop != "content", "元素本体不应收到伪元素的 content 声明");
    }

    // 4. after 槽不应收到 before 的声明
    let after_decls = collect_pseudo_declarations_with_media(
        &_doc,
        div,
        &stylesheets,
        &build_stylesheet_index(&stylesheets),
        None,
        None,
        "after",
    );
    assert!(after_decls.is_empty(), "after 槽应为空（规则只匹配 before）");
}

/// 双冒号 `::before` 与单冒号等价，且特异性含伪元素贡献。
#[test]
fn test_double_colon_pseudo_element_routing() {
    let (_doc, _html, _body, div, _p) = make_test_dom();
    let css = r#"div::after { content: "Z"; }"#;
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);
    use zero_css_parser::ast::Rule;
    let style_rule = match &stylesheet.rules[0] {
        Rule::Style(sr) => sr,
        _ => panic!("expected style rule"),
    };
    assert_eq!(selector_pseudo_element(&style_rule.selectors[0]), Some("after"));
    let stylesheets = [stylesheet];
    let after_decls = collect_pseudo_declarations_with_media(
        &_doc,
        div,
        &stylesheets,
        &build_stylesheet_index(&stylesheets),
        None,
        None,
        "after",
    );
    assert_eq!(after_decls.len(), 1);
    assert_eq!(after_decls[0].0, "content");
}
