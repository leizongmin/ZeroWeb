// Matcher 覆盖率第 4 轮 — 聚焦未覆盖的内部函数路径和选择器匹配边界
use super::super::*;
use zero_css_parser::ast::{
    ComplexSelector, CompoundSelector, NthPattern, PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
};
use zero_dom::Document;

// ── matches_nth_pattern 更多边界值 ──

#[test]
fn test_nth_pattern_negative_a_coefficient_variants() {
    // -n+5 → 匹配 5, 4, 3, 2, 1
    let pattern = NthPattern { a: -1, b: 5 };
    assert!(matches_nth_pattern(5, &pattern));
    assert!(matches_nth_pattern(4, &pattern));
    assert!(matches_nth_pattern(1, &pattern));
    assert!(!matches_nth_pattern(6, &pattern));
    // index=0: diff=0-5=-5, -5<=0 && -5%-1==0 → true（模式允许 index=0）
    assert!(matches_nth_pattern(0, &pattern));
}

#[test]
fn test_nth_pattern_negative_a_step_2() {
    let pattern = NthPattern { a: -2, b: 10 };
    assert!(matches_nth_pattern(10, &pattern));
    assert!(matches_nth_pattern(8, &pattern));
    assert!(matches_nth_pattern(2, &pattern));
    assert!(!matches_nth_pattern(9, &pattern));
    assert!(!matches_nth_pattern(12, &pattern));
}

#[test]
fn test_nth_pattern_negative_a_step_3() {
    let pattern = NthPattern { a: -3, b: 9 };
    assert!(matches_nth_pattern(9, &pattern));
    assert!(matches_nth_pattern(6, &pattern));
    assert!(matches_nth_pattern(3, &pattern));
    assert!(!matches_nth_pattern(7, &pattern));
}

// ── get_axis_size 特性名变体 ──

#[test]
fn test_get_axis_size_all_feature_names() {
    let ctx = ContainerContext::with_size(800.0, 600.0);

    assert_eq!(get_axis_size(&ctx, "width"), Some(800.0));
    assert_eq!(get_axis_size(&ctx, "height"), Some(600.0));
    assert_eq!(get_axis_size(&ctx, "inline-size"), Some(800.0));
    assert_eq!(get_axis_size(&ctx, "block-size"), Some(600.0));
    assert_eq!(get_axis_size(&ctx, "min-width"), Some(800.0));
    assert_eq!(get_axis_size(&ctx, "max-width"), Some(800.0));
    assert_eq!(get_axis_size(&ctx, "min-height"), Some(600.0));
    assert_eq!(get_axis_size(&ctx, "max-height"), Some(600.0));
    assert_eq!(get_axis_size(&ctx, "orientation"), None);
    assert_eq!(get_axis_size(&ctx, ""), None);
}

#[test]
fn test_get_axis_size_none_context() {
    let ctx = ContainerContext::new();
    assert_eq!(get_axis_size(&ctx, "width"), None);
    assert_eq!(get_axis_size(&ctx, "height"), None);
}

// ── length_to_px 非像素单位 ──

#[test]
fn test_length_to_px_non_px_units() {
    assert_eq!(length_to_px("100px"), Some(100.0));
    assert_eq!(length_to_px("0px"), Some(0.0));
    assert_eq!(length_to_px("50.5px"), Some(50.5));
    assert_eq!(length_to_px("10em"), Some(0.0));
    assert_eq!(length_to_px("2rem"), Some(0.0));
    assert_eq!(length_to_px("invalid"), None);
    assert_eq!(length_to_px(""), None);
}

#[test]
fn test_length_to_px_whitespace() {
    assert_eq!(length_to_px("  100px  "), Some(100.0));
}

// ── ContainerContext Default 实现 ──

#[test]
fn test_container_context_default() {
    let ctx = ContainerContext::default();
    assert!(ctx.container_width.is_none());
    assert!(ctx.container_height.is_none());
}

#[test]
fn test_container_context_new() {
    let ctx = ContainerContext::new();
    assert!(ctx.container_width.is_none());
    assert!(ctx.container_height.is_none());
}

#[test]
fn test_container_context_with_size() {
    let ctx = ContainerContext::with_size(1024.0, 768.0);
    assert_eq!(ctx.container_width, Some(1024.0));
    assert_eq!(ctx.container_height, Some(768.0));
}

// ── is_element 对非元素节点 ──

#[test]
fn test_is_element_text_node() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();
    let text = doc.create_text_node("hello");
    doc.append_child(elem, text).unwrap();

    assert!(is_element(&doc, elem), "div 是元素");
    assert!(!is_element(&doc, text), "文本节点不是元素");
}

#[test]
fn test_is_element_invalid_node() {
    let doc = Document::new();
    assert!(!is_element(&doc, NodeId::default()));
}

// ── collect_descendants 测试 ──

#[test]
fn test_collect_descendants_nested() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child1 = doc.create_element("span");
    doc.append_child(parent, child1).unwrap();
    let child2 = doc.create_element("p");
    doc.append_child(parent, child2).unwrap();
    let grandchild = doc.create_element("a");
    doc.append_child(child1, grandchild).unwrap();

    let mut descendants = Vec::new();
    collect_descendants(&doc, parent, &mut descendants);
    assert_eq!(descendants.len(), 3, "应有 3 个后代元素");
    assert!(descendants.contains(&child1));
    assert!(descendants.contains(&child2));
    assert!(descendants.contains(&grandchild));
}

#[test]
fn test_collect_descendants_empty() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let mut descendants = Vec::new();
    collect_descendants(&doc, elem, &mut descendants);
    assert!(descendants.is_empty(), "无子元素时后代应为空");
}

// ── element_tag_name 边界 ──

#[test]
fn test_element_tag_name_lowercase() {
    let mut doc = Document::new();
    let elem = doc.create_element("DIV");
    doc.append_child(doc.root(), elem).unwrap();

    let name = element_tag_name(&doc, elem);
    assert_eq!(name, Some("div".to_string()));
}

#[test]
fn test_element_tag_name_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    doc.append_child(doc.root(), text).unwrap();

    assert_eq!(element_tag_name(&doc, text), None, "文本节点无标签名");
}

// ── evaluate_container_condition 无上下文 ──

#[test]
fn test_evaluate_container_condition_no_context() {
    use zero_css_parser::ast::{ContainerCondition, ContainerRule, ContainerSizeCondition};

    let rule = ContainerRule {
        name: None,
        condition: ContainerCondition::Size(ContainerSizeCondition {
            feature: "width".into(),
            value: "400px".into(),
            operator: None,
            range_min: None,
            range_max: None,
        }),
        rules: vec![],
    };

    assert!(!evaluate_container_condition(&rule, None));
}

// ── nth-child 使用真实选择器 ──

#[test]
fn test_nth_child_pattern_odd() {
    let mut doc = Document::new();
    let parent = doc.create_element("ul");
    doc.append_child(doc.root(), parent).unwrap();

    let children: Vec<_> = (0..6)
        .map(|_| {
            let li = doc.create_element("li");
            doc.append_child(parent, li).unwrap();
            li
        })
        .collect();

    let odd_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("li".into())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(
                        NthPattern { a: 2, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, children[0], &odd_sel));
    assert!(!matches_selector(&doc, children[1], &odd_sel));
    assert!(matches_selector(&doc, children[2], &odd_sel));
}

#[test]
fn test_nth_child_pattern_even() {
    let mut doc = Document::new();
    let parent = doc.create_element("ul");
    doc.append_child(doc.root(), parent).unwrap();

    let children: Vec<_> = (0..6)
        .map(|_| {
            let li = doc.create_element("li");
            doc.append_child(parent, li).unwrap();
            li
        })
        .collect();

    let even_sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("li".into())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthChild(
                        NthPattern { a: 2, b: 2 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(!matches_selector(&doc, children[0], &even_sel));
    assert!(matches_selector(&doc, children[1], &even_sel));
}

// ── nth-of-type / nth-last-of-type ──

#[test]
fn test_nth_of_type_first() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let span1 = doc.create_element("span");
    doc.append_child(parent, span1).unwrap();
    let p1 = doc.create_element("p");
    doc.append_child(parent, p1).unwrap();
    let span2 = doc.create_element("span");
    doc.append_child(parent, span2).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".into())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthOfType(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, span1, &sel));
    assert!(!matches_selector(&doc, span2, &sel));
}

#[test]
fn test_nth_last_of_type_first() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let span1 = doc.create_element("span");
    doc.append_child(parent, span1).unwrap();
    let span2 = doc.create_element("span");
    doc.append_child(parent, span2).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("span".into())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::NthLastOfType(
                        NthPattern { a: 0, b: 1 },
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, span2, &sel));
    assert!(!matches_selector(&doc, span1, &sel));
}

// ── :first-child / :last-child / :root / :empty 伪类 ──

#[test]
fn test_first_child_pseudo() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let text = doc.create_text_node("text");
    doc.append_child(parent, text).unwrap();
    let first_elem = doc.create_element("span");
    doc.append_child(parent, first_elem).unwrap();
    let second_elem = doc.create_element("p");
    doc.append_child(parent, second_elem).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Universal),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "first-child".into(),
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, first_elem, &sel));
    assert!(!matches_selector(&doc, second_elem, &sel));
}

#[test]
fn test_last_child_pseudo() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let first = doc.create_element("span");
    doc.append_child(parent, first).unwrap();
    let last = doc.create_element("p");
    doc.append_child(parent, last).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Universal),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "last-child".into(),
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, last, &sel));
    assert!(!matches_selector(&doc, first, &sel));
}

#[test]
fn test_root_pseudo() {
    let mut doc = Document::new();
    let html = doc.create_element("html");
    doc.append_child(doc.root(), html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "root".into(),
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, html, &sel));
    assert!(!matches_selector(&doc, body, &sel));
}

#[test]
fn test_empty_pseudo() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let empty_div = doc.create_element("div");
    doc.append_child(parent, empty_div).unwrap();

    let non_empty_div = doc.create_element("div");
    doc.append_child(parent, non_empty_div).unwrap();
    let text = doc.create_text_node("content");
    doc.append_child(non_empty_div, text).unwrap();

    let sel = Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: Some(TypeSelector::Tag("div".into())),
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Simple(
                        "empty".into(),
                    ))],
                },
                None,
            )],
        },
    };

    assert!(matches_selector(&doc, empty_div, &sel));
    assert!(!matches_selector(&doc, non_empty_div, &sel));
}
