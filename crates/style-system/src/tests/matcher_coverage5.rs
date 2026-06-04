//! style-system 覆盖率测试第五轮：matcher :has() 组合器覆盖。
//!
//! 重点覆盖 matches_has_selector_chain 中的：
//! - Descendant 组合器（lines 530-541）
//! - Child 组合器（lines 543-551）
//! - NextSibling 组合器（lines 553-560）
//! - SubsequentSibling 组合器（lines 562-576）

use super::super::*;
use super::helpers::*;
use crate::matcher::matches_selector;
use zero_css_parser::ast::{
    Combinator, ComplexSelector, CompoundSelector, PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
};

/// 构建 :has(.child) 选择器 — 内部包含后代组合器
fn make_has_descendant_selector() -> Selector {
    // :has(.child) → 内部是 [.child] 无组合器（单个 compound）
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        Selector {
                            complex: ComplexSelector {
                                parts: vec![(
                                    CompoundSelector {
                                        type_selector: None,
                                        subclass_selectors: vec![SubclassSelector::Class("child".to_string())],
                                    },
                                    None,
                                )],
                            },
                        },
                    ]))],
                },
                None,
            )],
        },
    }
}

/// 构建 :has(> .child) 选择器 — 内部 child 组合器
fn make_has_child_selector() -> Selector {
    // :has(> .child) → 两个 compound，中间有 Child 组合器
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        Selector {
                            complex: ComplexSelector {
                                parts: vec![
                                    // 第一个 compound 是空的（表示 :has 自身）
                                    (
                                        CompoundSelector {
                                            type_selector: None,
                                            subclass_selectors: vec![],
                                        },
                                        Some(Combinator::Child),
                                    ),
                                    // 第二个 compound 匹配 .child
                                    (
                                        CompoundSelector {
                                            type_selector: None,
                                            subclass_selectors: vec![SubclassSelector::Class("child".to_string())],
                                        },
                                        None,
                                    ),
                                ],
                            },
                        },
                    ]))],
                },
                None,
            )],
        },
    }
}

/// 构建 :has(.ancestor .child) 选择器 — 内部有 descendant 组合器
fn make_has_nested_descendant_selector() -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        Selector {
                            complex: ComplexSelector {
                                parts: vec![
                                    (
                                        CompoundSelector {
                                            type_selector: None,
                                            subclass_selectors: vec![SubclassSelector::Class("ancestor".to_string())],
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
                        },
                    ]))],
                },
                None,
            )],
        },
    }
}

/// 构建 :has(+ .sibling) 选择器 — 内部 next-sibling 组合器
fn make_has_next_sibling_selector() -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        Selector {
                            complex: ComplexSelector {
                                parts: vec![
                                    (
                                        CompoundSelector {
                                            type_selector: None,
                                            subclass_selectors: vec![],
                                        },
                                        Some(Combinator::NextSibling),
                                    ),
                                    (
                                        CompoundSelector {
                                            type_selector: None,
                                            subclass_selectors: vec![SubclassSelector::Class("sibling".to_string())],
                                        },
                                        None,
                                    ),
                                ],
                            },
                        },
                    ]))],
                },
                None,
            )],
        },
    }
}

/// 构建 :has(~ .sibling) 选择器 — 内部 subsequent-sibling 组合器
fn make_has_subsequent_sibling_selector() -> Selector {
    Selector {
        complex: ComplexSelector {
            parts: vec![(
                CompoundSelector {
                    type_selector: None,
                    subclass_selectors: vec![SubclassSelector::PseudoClass(PseudoClassSelector::Has(vec![
                        Selector {
                            complex: ComplexSelector {
                                parts: vec![
                                    (
                                        CompoundSelector {
                                            type_selector: None,
                                            subclass_selectors: vec![],
                                        },
                                        Some(Combinator::SubsequentSibling),
                                    ),
                                    (
                                        CompoundSelector {
                                            type_selector: None,
                                            subclass_selectors: vec![SubclassSelector::Class("sibling".to_string())],
                                        },
                                        None,
                                    ),
                                ],
                            },
                        },
                    ]))],
                },
                None,
            )],
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════
// :has() 后代组合器测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_has_descendant_matches() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let sel = make_has_descendant_selector();
    assert!(
        matches_selector(&doc, parent, &sel),
        ":has(.child) 应匹配包含 .child 子元素的 div"
    );
}

#[test]
fn test_has_descendant_no_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let child = doc.create_element("span");
    // 不设置 class="child"
    doc.append_child(parent, child).unwrap();

    let sel = make_has_descendant_selector();
    assert!(
        !matches_selector(&doc, parent, &sel),
        ":has(.child) 不应匹配无 .child 的 div"
    );
}

#[test]
fn test_has_descendant_deeply_nested() {
    let mut doc = Document::new();
    let root = doc.root();
    let grandparent = doc.create_element("div");
    doc.append_child(root, grandparent).unwrap();
    let parent = doc.create_element("section");
    doc.append_child(grandparent, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let sel = make_has_descendant_selector();
    // grandparent 也包含 .child 作为后代
    assert!(
        matches_selector(&doc, grandparent, &sel),
        ":has(.child) 应匹配深层嵌套的祖先"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// :has() 子组合器测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_has_child_matches() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let sel = make_has_child_selector();
    assert!(matches_selector(&doc, parent, &sel), ":has(> .child) 应匹配直接子元素");
}

#[test]
fn test_has_child_no_match_deeply_nested() {
    let mut doc = Document::new();
    let root = doc.root();
    let grandparent = doc.create_element("div");
    doc.append_child(root, grandparent).unwrap();
    let parent = doc.create_element("section");
    doc.append_child(grandparent, parent).unwrap();
    let child = doc.create_element("span");
    doc.set_attribute(child, "class", "child");
    doc.append_child(parent, child).unwrap();

    let sel = make_has_child_selector();
    // grandparent 的直接子元素是 section（不是 .child），不应匹配
    assert!(
        !matches_selector(&doc, grandparent, &sel),
        ":has(> .child) 不应匹配非直接子元素"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// :has() 嵌套后代组合器（两段 compound + descendant combinator）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_has_nested_descendant_matches() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let ancestor = doc.create_element("section");
    doc.set_attribute(ancestor, "class", "ancestor");
    doc.append_child(parent, ancestor).unwrap();
    let target = doc.create_element("span");
    doc.set_attribute(target, "class", "target");
    doc.append_child(ancestor, target).unwrap();

    let sel = make_has_nested_descendant_selector();
    assert!(
        matches_selector(&doc, parent, &sel),
        ":has(.ancestor .target) 应匹配包含 .ancestor > .target 结构的元素"
    );
}

#[test]
fn test_has_nested_descendant_no_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let ancestor = doc.create_element("section");
    doc.set_attribute(ancestor, "class", "ancestor");
    doc.append_child(parent, ancestor).unwrap();
    // 没有 .target 子元素
    let other = doc.create_element("span");
    doc.set_attribute(other, "class", "other");
    doc.append_child(ancestor, other).unwrap();

    let sel = make_has_nested_descendant_selector();
    assert!(
        !matches_selector(&doc, parent, &sel),
        ":has(.ancestor .target) 不应匹配无 .target 的结构"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// :has() 兄弟组合器测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_has_next_sibling_matches() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    // + .sibling 需要一个元素紧挨在 .sibling 前面
    let first = doc.create_element("span");
    doc.set_attribute(first, "class", "prev");
    doc.append_child(parent, first).unwrap();
    let second = doc.create_element("span");
    doc.set_attribute(second, "class", "sibling");
    doc.append_child(parent, second).unwrap();

    let sel = make_has_next_sibling_selector();
    // parent 包含 span.prev + span.sibling，后代 span.sibling 的前一个兄弟是 span.prev（匹配空 compound）
    assert!(
        matches_selector(&doc, parent, &sel),
        ":has(+ .sibling) 应匹配包含紧跟 .sibling 的结构"
    );
}

#[test]
fn test_has_next_sibling_no_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let first = doc.create_element("span");
    doc.append_child(parent, first).unwrap();
    let second = doc.create_element("span");
    doc.set_attribute(second, "class", "other");
    doc.append_child(parent, second).unwrap();

    let sel = make_has_next_sibling_selector();
    assert!(
        !matches_selector(&doc, first, &sel),
        ":has(+ .sibling) 不应匹配后面不跟 .sibling 的元素"
    );
}

#[test]
fn test_has_subsequent_sibling_matches() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let first = doc.create_element("span");
    doc.set_attribute(first, "class", "prev");
    doc.append_child(parent, first).unwrap();
    let middle = doc.create_element("span");
    doc.append_child(parent, middle).unwrap();
    let last = doc.create_element("span");
    doc.set_attribute(last, "class", "sibling");
    doc.append_child(parent, last).unwrap();

    let sel = make_has_subsequent_sibling_selector();
    // parent 后代中有 .sibling，且 .sibling 前面有其他兄弟元素
    assert!(
        matches_selector(&doc, parent, &sel),
        ":has(~ .sibling) 应匹配包含后续 .sibling 兄弟的结构"
    );
}

#[test]
fn test_has_subsequent_sibling_no_match() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    let first = doc.create_element("span");
    doc.append_child(parent, first).unwrap();
    let second = doc.create_element("span");
    doc.set_attribute(second, "class", "other");
    doc.append_child(parent, second).unwrap();

    let sel = make_has_subsequent_sibling_selector();
    assert!(
        !matches_selector(&doc, first, &sel),
        ":has(~ .sibling) 不应匹配后面无 .sibling 的元素"
    );
}
