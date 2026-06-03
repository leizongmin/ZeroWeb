//! matcher 额外覆盖率测试：空选择器、通用选择器、组合器、ID/类匹配。

use super::super::*;
use super::helpers::*;
use crate::matcher::matches_selector;

// ═══════════════════════════════════════════════════════════════════════
// 空选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_selector_parts() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let empty_sel = Selector {
        complex: ComplexSelector { parts: vec![] },
    };
    assert!(!matches_selector(&doc, div, &empty_sel));
}

// ═══════════════════════════════════════════════════════════════════════
// 通用选择器
// ═══════════════════════════════════════════════════════════════════════

#[test]
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

// ═══════════════════════════════════════════════════════════════════════
// 后代组合器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_descendant_combinator_match() {
    let (doc, _html, _body, _div, p) = make_test_dom();
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
fn test_descendant_combinator_no_match() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("span".to_string())),
                        subclass_selectors: vec![],
                    },
                    Some(Combinator::Descendant),
                ),
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(!matches_selector(&doc, div, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// 子组合器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_child_combinator_match() {
    let (doc, _html, _body, div, p) = make_test_dom();
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
}

#[test]
fn test_child_combinator_body_to_div() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = Selector {
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
                        type_selector: Some(TypeSelector::Tag("div".to_string())),
                        subclass_selectors: vec![],
                    },
                    None,
                ),
            ],
        },
    };
    assert!(matches_selector(&doc, div, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// ID 和类选择器匹配
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_id_selector_match() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id("main".to_string())],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel));
}

#[test]
fn test_id_selector_no_match() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Id("main".to_string())],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, p, &sel));
}

#[test]
fn test_class_selector_match() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class("text".to_string())],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, p, &sel));
}

#[test]
fn test_class_selector_no_match() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Class("text".to_string())],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, div, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// 标签 + 类组合
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tag_and_class_combined() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("p".to_string())),
                    subclass_selectors: vec![SubclassSelector::Class("text".to_string())],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, p, &sel));
}

#[test]
fn test_tag_and_class_wrong_tag() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".to_string())),
                    subclass_selectors: vec![SubclassSelector::Class("text".to_string())],
                },
                None,
            )],
        },
    };
    assert!(!matches_selector(&doc, p, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// collect_matching_declarations（独立函数）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_matching_basic() {
    let (doc, _html, _body, div, _p) = make_test_dom();

    let css = "div { color: red; }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);

    let decls = crate::matcher::collect_matching_declarations(&doc, div, &[stylesheet]);
    assert!(!decls.is_empty());
}

#[test]
fn test_collect_matching_no_match() {
    let (doc, _html, _body, _div, p) = make_test_dom();

    let css = "span { color: blue; }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);

    let decls = crate::matcher::collect_matching_declarations(&doc, p, &[stylesheet]);
    assert!(decls.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 孤立元素（无父节点）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_orphaned_element() {
    let mut doc = Document::new();
    let orphan = doc.create_element("div");
    let sel = make_tag_selector("div");
    assert!(matches_selector(&doc, orphan, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// 下一个兄弟组合器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_next_sibling_combinator() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();

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
    assert!(matches_selector(&doc, child2, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// 后续兄弟组合器
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_subsequent_sibling_combinator() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child1 = doc.create_element("p");
    let child2 = doc.create_element("a");
    let child3 = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();
    doc.append_child(parent, child3).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![
                (
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
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
    assert!(matches_selector(&doc, child3, &sel));
}
