// Test file split from matcher.rs — core selector matching tests
use super::super::*;
use zero_css_parser::ast::{
    AttributeMatcher, AttributeSelector, Combinator, ComplexSelector, CompoundSelector, PseudoClassSelector,
    Selector, SubclassSelector, TypeSelector,
};
use zero_dom::Document;

// ── 辅助函数 ──

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

pub(super) fn make_id_selector(id: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id(id.to_string())],
                },
                None,
            )],
        },
    }
}

pub(super) fn make_class_selector(cls: &str) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class(cls.to_string())],
                },
                None,
            )],
        },
    }
}

/// 创建一个简单的测试 DOM：html > body > div#main.container > p.text
pub(super) fn make_test_dom() -> (Document, NodeId, NodeId, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();

    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let div = doc.create_element("div");
    doc.set_attribute(div, "id", "main");
    doc.set_attribute(div, "class", "container");
    doc.append_child(body, div).unwrap();

    let p = doc.create_element("p");
    doc.set_attribute(p, "class", "text");
    doc.append_child(div, p).unwrap();

    (doc, html, body, div, p)
}
#[test]
fn test_matches_tag_selector() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = make_tag_selector("div");
    assert!(matches_selector(&doc, div, &sel));

    let sel_p = make_tag_selector("p");
    assert!(!matches_selector(&doc, div, &sel_p));
}

#[test]
fn test_matches_id_selector() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = make_id_selector("main");
    assert!(matches_selector(&doc, div, &sel));

    let sel_not_found = make_id_selector("other");
    assert!(!matches_selector(&doc, div, &sel_not_found));
}

#[test]
fn test_matches_class_selector() {
    let (doc, _html, _body, div, p) = make_test_dom();
    let sel = make_class_selector("container");
    assert!(matches_selector(&doc, div, &sel));

    let sel_text = make_class_selector("text");
    assert!(matches_selector(&doc, p, &sel_text));
    assert!(!matches_selector(&doc, div, &sel_text));
}

#[test]
fn test_matches_universal_selector() {
    let (doc, _html, _body, div, _p) = make_test_dom();
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
}

#[test]
fn test_matches_descendant_combinator() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    // div p
    let sel = Selector {
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
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(matches_selector(&doc, p, &sel));
}

#[test]
fn test_matches_child_combinator() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    // div > p
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Child),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(matches_selector(&doc, p, &sel));

    // body > p 不应该匹配（p 是 div 的子元素，不是 body 的直接子元素）
    let sel2 = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("body".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Child),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(!matches_selector(&doc, p, &sel2));
}

#[test]
fn test_matches_attribute_selector() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    // [id]
    let sel_exists = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "id".to_string(),
                        matcher: AttributeMatcher::Exists,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel_exists));

    // [id=main]
    let sel_exact = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "id".to_string(),
                        matcher: AttributeMatcher::Exact("main".to_string()),
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel_exact));
}

#[test]
fn test_matches_pseudo_first_child() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "first-child".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    // div 是 body 的第一个子元素
    assert!(matches_selector(&doc, div, &sel));
}

#[test]
fn test_matches_pseudo_root() {
    let (doc, html, _body, _div, _p) = make_test_dom();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "root".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    // html 是文档根元素
    assert!(matches_selector(&doc, html, &sel));
}

#[test]
fn test_matches_pseudo_empty() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "empty".to_string(),
                    ))],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel));

    // 添加文本节点后不再是 empty
    let text = doc.create_text_node("hello");
    doc.append_child(div, text).unwrap();
    assert!(!matches_selector(&doc, div, &sel));
}

#[test]
fn test_matches_not_pseudo() {
    let (doc, _html, _body, div, p) = make_test_dom();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Not(vec![
                        make_id_selector("main"),
                    ]))],
                },
                None,
            )],
        },
    };
    // div#main 不匹配 :not(#main)
    assert!(!matches_selector(&doc, div, &sel));
    // p 匹配 :not(#main)
    assert!(matches_selector(&doc, p, &sel));
}

#[test]
fn test_matches_is_pseudo() {
    let (doc, _html, _body, div, p) = make_test_dom();

    let sel = Selector {
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
    assert!(matches_selector(&doc, div, &sel));
    assert!(!matches_selector(&doc, p, &sel));
}
