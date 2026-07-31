// Matcher nth/has/container/supports 扩展测试
use super::super::*;
use zero_css_parser::ast::{
    AttrCaseModifier, AttributeMatcher, AttributeSelector, ComplexSelector, CompoundSelector, NthPattern,
    PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
};
use zero_dom::Document;

/// 辅助：创建 NthPattern。
fn nth(a: i32, b: i32) -> NthPattern {
    NthPattern { a, b }
}

/// 辅助：创建含伪类的选择器。
fn pseudo_sel(pc: PseudoClassSelector) -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(pc)],
                },
                None,
            )],
        },
    }
}

// ── matches_nth_pattern 测试 ──

#[test]
fn test_nth_pattern_exact_match() {
    assert!(matches_nth_pattern(3, &nth(0, 3)));
    assert!(!matches_nth_pattern(2, &nth(0, 3)));
    assert!(!matches_nth_pattern(4, &nth(0, 3)));
}

#[test]
fn test_nth_pattern_odd_even() {
    assert!(matches_nth_pattern(1, &nth(2, 1)));
    assert!(matches_nth_pattern(3, &nth(2, 1)));
    assert!(!matches_nth_pattern(2, &nth(2, 1)));
    assert!(matches_nth_pattern(2, &nth(2, 0)));
    assert!(matches_nth_pattern(4, &nth(2, 0)));
    assert!(!matches_nth_pattern(1, &nth(2, 0)));
}

#[test]
fn test_nth_pattern_every_3n() {
    assert!(matches_nth_pattern(3, &nth(3, 0)));
    assert!(matches_nth_pattern(6, &nth(3, 0)));
    assert!(!matches_nth_pattern(1, &nth(3, 0)));
    assert!(!matches_nth_pattern(4, &nth(3, 0)));
}

#[test]
fn test_nth_pattern_3n_plus_1() {
    assert!(matches_nth_pattern(1, &nth(3, 1)));
    assert!(matches_nth_pattern(4, &nth(3, 1)));
    assert!(!matches_nth_pattern(2, &nth(3, 1)));
}

#[test]
fn test_nth_pattern_negative_a() {
    assert!(matches_nth_pattern(1, &nth(-1, 3)));
    assert!(matches_nth_pattern(2, &nth(-1, 3)));
    assert!(matches_nth_pattern(3, &nth(-1, 3)));
    assert!(!matches_nth_pattern(4, &nth(-1, 3)));
}

#[test]
fn test_nth_pattern_zero_index() {
    assert!(!matches_nth_pattern(0, &nth(2, 1)));
    assert!(matches_nth_pattern(0, &nth(0, 0)));
}

// ── matches_nth_child with DOM ──

#[test]
fn test_nth_child_basic() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("ul");
    let c1 = doc.create_element("li");
    let c2 = doc.create_element("li");
    let c3 = doc.create_element("li");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();
    doc.append_child(parent, c3).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::NthChild(nth(2, 1)));
    assert!(matches_selector(&doc, c1, &sel), "c1 is 1st, odd (2n+1)");
    assert!(!matches_selector(&doc, c2, &sel), "c2 is 2nd, not odd");
    assert!(matches_selector(&doc, c3, &sel), "c3 is 3rd, odd (2n+1)");
}

#[test]
fn test_nth_child_no_parent() {
    let mut doc = Document::new();
    let orphan = doc.create_element("li");
    let sel = pseudo_sel(PseudoClassSelector::NthChild(nth(0, 1)));
    assert!(!matches_selector(&doc, orphan, &sel));
}

#[test]
fn test_nth_last_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("ul");
    let c1 = doc.create_element("li");
    let c2 = doc.create_element("li");
    let c3 = doc.create_element("li");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();
    doc.append_child(parent, c3).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::NthLastChild(nth(0, 1)));
    assert!(matches_selector(&doc, c3, &sel));
    assert!(!matches_selector(&doc, c1, &sel));
}

#[test]
fn test_nth_of_type() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let span1 = doc.create_element("span");
    let p1 = doc.create_element("p");
    let span2 = doc.create_element("span");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, span1).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span2).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::NthOfType(nth(0, 2)));
    assert!(matches_selector(&doc, span2, &sel));
    assert!(!matches_selector(&doc, span1, &sel));
}

#[test]
fn test_nth_last_of_type() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let span1 = doc.create_element("span");
    let p1 = doc.create_element("p");
    let span2 = doc.create_element("span");
    let span3 = doc.create_element("span");

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, span1).unwrap();
    doc.append_child(parent, p1).unwrap();
    doc.append_child(parent, span2).unwrap();
    doc.append_child(parent, span3).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::NthLastOfType(nth(0, 1)));
    assert!(matches_selector(&doc, span3, &sel));
    assert!(matches_selector(&doc, p1, &sel));
    assert!(!matches_selector(&doc, span1, &sel));
}

// ── Simple pseudo-classes (first-child, last-child, only-child, empty, root) ──

#[test]
fn test_first_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let c1 = doc.create_element("p");
    let c2 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("first-child".to_string()));
    assert!(matches_selector(&doc, c1, &sel));
    assert!(!matches_selector(&doc, c2, &sel));
}

#[test]
fn test_last_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let c1 = doc.create_element("p");
    let c2 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("last-child".to_string()));
    assert!(!matches_selector(&doc, c1, &sel));
    assert!(matches_selector(&doc, c2, &sel));
}

#[test]
fn test_only_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let only = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, only).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("only-child".to_string()));
    assert!(matches_selector(&doc, only, &sel));
}

#[test]
fn test_only_child_with_siblings() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let c1 = doc.create_element("p");
    let c2 = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("only-child".to_string()));
    assert!(!matches_selector(&doc, c1, &sel));
}

#[test]
fn test_empty_element() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("empty".to_string()));
    assert!(matches_selector(&doc, el, &sel));
}

#[test]
fn test_not_empty_with_text() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let text = doc.create_text_node("hello");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, text).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("empty".to_string()));
    assert!(!matches_selector(&doc, div, &sel));
}

#[test]
fn test_empty_with_whitespace() {
    // CSS Selectors §:empty：纯空白文本使元素**非空**（WPT selectors-empty-001.xml test6
    // `<test6> </test6>` 在 :not(:empty) 块；与 Chromium 一致）。修复前 ZW 错把纯空白当 empty。
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let ws = doc.create_text_node("   ");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, ws).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("empty".to_string()));
    assert!(
        !matches_selector(&doc, div, &sel),
        "whitespace-only text should NOT be :empty"
    );
}

#[test]
fn test_empty_comment_or_pi_only() {
    // 仅注释或处理指令 → :empty（不计入；WPT selectors-empty-001 test3/test4）。
    let mut doc = Document::new();
    let root = doc.root();

    let div_comment = doc.create_element("div");
    doc.append_child(root, div_comment).unwrap();
    let cmt = doc.create_comment("x");
    doc.append_child(div_comment, cmt).unwrap();

    let div_empty_text = doc.create_element("div");
    doc.append_child(root, div_empty_text).unwrap();
    let empty_tn = doc.create_text_node("");
    doc.append_child(div_empty_text, empty_tn).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("empty".to_string()));
    assert!(
        matches_selector(&doc, div_comment, &sel),
        "comment-only should be :empty"
    );
    assert!(
        matches_selector(&doc, div_empty_text, &sel),
        "empty-string text node should be :empty"
    );
}

#[test]
fn test_not_empty_whitespace_plus_comment() {
    // 空白 + 注释 → 非空（有非空文本；WPT selectors-empty-001 test7/test9/test10）。
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();
    let cmt = doc.create_comment("c");
    doc.append_child(div, cmt).unwrap();
    let ws = doc.create_text_node(" ");
    doc.append_child(div, ws).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("empty".to_string()));
    assert!(
        !matches_selector(&doc, div, &sel),
        "whitespace + comment should NOT be :empty"
    );
}

#[test]
fn test_root_pseudo() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();

    let sel = pseudo_sel(PseudoClassSelector::Simple("root".to_string()));
    assert!(matches_selector(&doc, html, &sel));
}

// ── ContainerContext ──

#[test]
fn test_container_context_new() {
    let ctx = ContainerContext::new();
    assert!(ctx.container_width.is_none());
    assert!(ctx.container_height.is_none());
}

#[test]
fn test_container_context_with_size() {
    let ctx = ContainerContext::with_size(800.0, 600.0);
    assert_eq!(ctx.container_width, Some(800.0));
    assert_eq!(ctx.container_height, Some(600.0));
}

// ── evaluate_supports_condition 测试 ──

#[test]
fn test_supports_property() {
    use zero_css_parser::ast::SupportsCondition;
    let _ = evaluate_supports_condition(&SupportsCondition::Property("display".to_string(), "flex".to_string()));
}

#[test]
fn test_supports_and_condition() {
    use zero_css_parser::ast::SupportsCondition;
    let _ = evaluate_supports_condition(&SupportsCondition::And(vec![
        SupportsCondition::Property("display".to_string(), "flex".to_string()),
        SupportsCondition::Property("color".to_string(), "red".to_string()),
    ]));
}

#[test]
fn test_supports_or_condition() {
    use zero_css_parser::ast::SupportsCondition;
    let _ = evaluate_supports_condition(&SupportsCondition::Or(vec![
        SupportsCondition::Property("display".to_string(), "flex".to_string()),
        SupportsCondition::Property("display".to_string(), "grid".to_string()),
    ]));
}

#[test]
fn test_supports_not_condition() {
    use zero_css_parser::ast::SupportsCondition;
    let _ = evaluate_supports_condition(&SupportsCondition::Not(Box::new(SupportsCondition::Property(
        "display".to_string(),
        "grid".to_string(),
    ))));
}

#[test]
fn test_supports_selector() {
    use zero_css_parser::ast::SupportsCondition;
    let _ = evaluate_supports_condition(&SupportsCondition::Selector("div > p".to_string()));
}

// ── 属性选择器测试 ──

#[test]
fn test_attribute_exact_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("input");
    doc.set_attribute(el, "type", "text");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("input".to_string())),
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "type".to_string(),
                        matcher: AttributeMatcher::Exact("text".to_string()),
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

#[test]
fn test_attribute_exists() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "data-test", "value");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::Attribute(AttributeSelector {
                        name: "data-test".to_string(),
                        matcher: AttributeMatcher::Exists,
                        case: AttrCaseModifier::Default,
                    })],
                },
                None,
            )],
        },
    };
    assert!(matches_selector(&doc, el, &sel));
}

#[test]
fn test_attribute_no_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "foo");
    doc.append_child(root, el).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
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
    assert!(!matches_selector(&doc, el, &sel));
}
