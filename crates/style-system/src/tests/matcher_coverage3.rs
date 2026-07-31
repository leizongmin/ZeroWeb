//! matcher 覆盖率测试第三轮：通过 matches_selector 间接测试内部函数。
//!
//! 覆盖：:has() 子/兄弟组合器、:empty、:first/:last-child、
//! :first/:last-of-type、:nth-of-type、:nth-last-of-type、
//! :root、容器查询、属性选择器、@layer 嵌套。

use super::super::*;
use super::helpers::*;
use crate::matcher::matches_selector;
use zero_css_parser::ast::{
    AttrCaseModifier, AttributeMatcher, AttributeSelector, Combinator, CompoundSelector, NthPattern,
    PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
};

/// Helper: 创建带伪类的选择器。
fn make_pseudo_selector(pseudo: PseudoClassSelector) -> Selector {
    Selector {
        complex: zero_css_parser::ast::ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(pseudo)],
                },
                None,
            )],
        },
    }
}

/// Helper: 创建带标签+伪类的选择器。
fn make_tag_pseudo_selector(tag: &str, pseudo: PseudoClassSelector) -> Selector {
    Selector {
        complex: zero_css_parser::ast::ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag(tag.to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(pseudo)],
                },
                None,
            )],
        },
    }
}

/// Helper: 创建 :has() 选择器。
fn make_has_selector(outer_tag: &str, inner_sel: Selector) -> Selector {
    Selector {
        complex: zero_css_parser::ast::ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag(outer_tag.to_string())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![inner_sel]))],
                },
                None,
            )],
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════
// :empty 伪类
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_empty_element_no_children() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("empty".to_string()));
    // div 有子元素 p，所以不是 empty
    assert!(!matches_selector(&doc, div, &sel));
}

#[test]
fn test_empty_element_truly_empty() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let empty_div = doc.create_element("div");
    doc.append_child(root, empty_div).unwrap();
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("empty".to_string()));
    assert!(matches_selector(&doc, empty_div, &sel));
}

#[test]
fn test_empty_element_with_text() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let text = doc.create_text_node("hello");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, text).unwrap();
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("empty".to_string()));
    assert!(!matches_selector(&doc, div, &sel));
}

#[test]
fn test_empty_element_with_whitespace_text() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let text = doc.create_text_node("   ");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, text).unwrap();
    // 任何非空文本（含纯空白）都使元素非空（CSS Selectors §:empty，与 Chromium 一致）
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("empty".to_string()));
    assert!(!matches_selector(&doc, div, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :first-child / :last-child
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_first_child_match() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    // p 是 div 的唯一子元素 → first-child
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("first-child".to_string()));
    assert!(matches_selector(&doc, p, &sel));
}

#[test]
fn test_first_child_with_multiple_siblings() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let first = doc.create_element("span");
    let second = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, first).unwrap();
    doc.append_child(parent, second).unwrap();
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("first-child".to_string()));
    assert!(matches_selector(&doc, first, &sel));
    assert!(!matches_selector(&doc, second, &sel));
}

#[test]
fn test_last_child_match() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let first = doc.create_element("span");
    let second = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, first).unwrap();
    doc.append_child(parent, second).unwrap();
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("last-child".to_string()));
    assert!(!matches_selector(&doc, first, &sel));
    assert!(matches_selector(&doc, second, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :root
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_root_element_match() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("root".to_string()));
    // div 的父元素是 body，body 的父元素是 html → div 不是根
    assert!(!matches_selector(&doc, div, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :first-of-type / :last-of-type
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_first_of_type_multiple_same() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let span = doc.create_element("span");
    let p2 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, p2).unwrap();

    let sel = make_pseudo_selector(PseudoClassSelector::Simple("first-of-type".to_string()));
    assert!(matches_selector(&doc, p1, &sel));
    assert!(!matches_selector(&doc, p2, &sel));
    // span 是唯一 span → 也是 first-of-type
    assert!(matches_selector(&doc, span, &sel));
}

#[test]
fn test_last_of_type_multiple_same() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let span = doc.create_element("span");
    let p2 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, p2).unwrap();

    let sel = make_pseudo_selector(PseudoClassSelector::Simple("last-of-type".to_string()));
    assert!(!matches_selector(&doc, p1, &sel));
    assert!(matches_selector(&doc, p2, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :nth-of-type / :nth-last-of-type
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_of_type_basic() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let p2 = doc.create_element("p");
    let p3 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, p2).unwrap();
    doc.append_child(parent, p3).unwrap();

    let sel = make_pseudo_selector(PseudoClassSelector::NthOfType(NthPattern { a: 0, b: 2 }));
    assert!(!matches_selector(&doc, p1, &sel));
    assert!(matches_selector(&doc, p2, &sel));
    assert!(!matches_selector(&doc, p3, &sel));
}

#[test]
fn test_nth_last_of_type_basic() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p1 = doc.create_element("p");
    let p2 = doc.create_element("p");
    let p3 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, p2).unwrap();
    doc.append_child(parent, p3).unwrap();

    let sel = make_pseudo_selector(PseudoClassSelector::NthLastOfType(NthPattern { a: 0, b: 1 }));
    // 从后往前数第 1 个 → p3
    assert!(!matches_selector(&doc, p1, &sel));
    assert!(!matches_selector(&doc, p2, &sel));
    assert!(matches_selector(&doc, p3, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :has() with Child combinator
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_has_child_combinator_match() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    let grandchild = doc.create_element("a");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();
    doc.append_child(child, grandchild).unwrap();

    // div:has(> span) — 直接子元素有 span
    let inner_sel = Selector {
        complex: zero_css_parser::ast::ComplexSelector {
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
    let sel = make_has_selector("div", inner_sel);
    assert!(matches_selector(&doc, parent, &sel));
}

#[test]
fn test_has_child_combinator_no_match() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    // div:has(> span) — 直接子元素只有 p，没有 span
    let inner_sel = Selector {
        complex: zero_css_parser::ast::ComplexSelector {
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
    let sel = make_has_selector("div", inner_sel);
    assert!(!matches_selector(&doc, parent, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :has() with SubsequentSibling combinator
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_has_subsequent_sibling() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let p = doc.create_element("p");
    let span = doc.create_element("span");
    let a = doc.create_element("a");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, p).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, a).unwrap();

    // div:has(p ~ span) — p 后面有 span
    let inner_sel = Selector {
        complex: zero_css_parser::ast::ComplexSelector {
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
    let sel = make_has_selector("div", inner_sel);
    assert!(matches_selector(&doc, parent, &sel));
}

#[test]
fn test_has_subsequent_sibling_no_match() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, p).unwrap();

    // div:has(p ~ span) — p 在 span 后面，不满足
    let inner_sel = Selector {
        complex: zero_css_parser::ast::ComplexSelector {
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
    let sel = make_has_selector("div", inner_sel);
    assert!(!matches_selector(&doc, parent, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// 属性选择器匹配
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attribute_exists_match() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = Selector {
        complex: zero_css_parser::ast::ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "id".to_string(),
                        matcher: AttributeMatcher::Exists,
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel));
}

#[test]
fn test_attribute_exact_match() {
    let (doc, _html, _body, div, _p) = make_test_dom();
    let sel = Selector {
        complex: zero_css_parser::ast::ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "id".to_string(),
                        matcher: AttributeMatcher::Exact("main".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, div, &sel));
    assert!(!matches_selector(&doc, _p, &sel));
}

#[test]
fn test_attribute_includes_match() {
    let (doc, _html, _body, _div, p) = make_test_dom();
    let sel = Selector {
        complex: zero_css_parser::ast::ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "class".to_string(),
                        matcher: AttributeMatcher::Includes("text".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, p, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// :nth-child with negative coefficient
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_nth_child_negative_coefficient() {
    let mut doc = zero_dom::Document::new();
    let root = doc.root();
    let parent = doc.create_element("ul");
    let li1 = doc.create_element("li");
    let li2 = doc.create_element("li");
    let li3 = doc.create_element("li");
    let li4 = doc.create_element("li");
    let li5 = doc.create_element("li");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, li1).unwrap();
    doc.append_child(parent, li2).unwrap();
    doc.append_child(parent, li3).unwrap();
    doc.append_child(parent, li4).unwrap();
    doc.append_child(parent, li5).unwrap();

    // -n+3 → 匹配 1, 2, 3
    let sel = make_pseudo_selector(PseudoClassSelector::NthChild(NthPattern { a: -1, b: 3 }));
    assert!(matches_selector(&doc, li1, &sel));
    assert!(matches_selector(&doc, li2, &sel));
    assert!(matches_selector(&doc, li3, &sel));
    assert!(!matches_selector(&doc, li4, &sel));
    assert!(!matches_selector(&doc, li5, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// 孤立元素（无父节点）的伪类
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_orphaned_element_first_child() {
    let mut doc = zero_dom::Document::new();
    let orphan = doc.create_element("div");
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("first-child".to_string()));
    // 孤立元素没有父元素 → 不是 first-child
    assert!(!matches_selector(&doc, orphan, &sel));
}

#[test]
fn test_orphaned_element_last_child() {
    let mut doc = zero_dom::Document::new();
    let orphan = doc.create_element("div");
    let sel = make_pseudo_selector(PseudoClassSelector::Simple("last-child".to_string()));
    assert!(!matches_selector(&doc, orphan, &sel));
}

// ═══════════════════════════════════════════════════════════════════════
// collect_matching_declarations with @layer
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_from_layer_rules() {
    let (doc, _html, _body, _div, p) = make_test_dom();

    let css = "@layer base { p { color: blue; } }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);

    let decls = crate::matcher::collect_matching_declarations(&doc, p, &[stylesheet]);
    assert!(!decls.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// collect_matching_declarations_with_media (无媒体上下文)
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_with_media_no_context() {
    let (doc, _html, _body, _div, p) = make_test_dom();

    let css = "p { color: red; } @media (min-width: 400px) { p { font-weight: bold; } }";
    let stylesheet = zero_css_parser::Parser::parse_stylesheet(css);

    // 无媒体上下文 → 只有非 @media 规则匹配
    let decls = crate::matcher::collect_matching_declarations_with_media(&doc, p, &[stylesheet], None, None);
    assert!(!decls.is_empty());
    // 应该只有 color: red，不应该有 font-weight: bold
    let has_bold = decls.iter().any(|d| d.0 == "font-weight");
    assert!(!has_bold);
}
