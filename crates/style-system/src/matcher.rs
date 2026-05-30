//! CSS 选择器匹配。
//!
//! 实现选择器与 DOM 元素的匹配逻辑，从右到左遍历选择器部分，
//! 检查标签名、ID、类、属性和伪类。

/// 匹配声明结果类型：(属性名, 属性值, 是否important, 特异性)
type MatchingDecl = (String, String, bool, (u32, u32, u32));

use zero_css_parser::ast::{
    AttributeMatcher, AttributeSelector, Combinator, CompoundSelector, PseudoClassSelector,
    Selector, SubclassSelector, TypeSelector,
};
use zero_dom::{Document, NodeId, NodeKind};

/// 检查选择器是否匹配指定 DOM 元素。
///
/// 从右到左遍历选择器的复合选择器链，逐个验证匹配条件。
pub fn matches_selector(doc: &Document, element: NodeId, selector: &Selector) -> bool {
    let parts = &selector.complex.parts;
    if parts.is_empty() {
        return false;
    }

    // 从最后一个复合选择器开始（最右边是目标元素）
    let last_idx = parts.len() - 1;
    let (compound, _) = &parts[last_idx];

    // 首先检查目标元素是否匹配最后一个复合选择器
    if !matches_compound(doc, element, compound) {
        return false;
    }

    // 如果只有一个复合选择器，匹配成功
    if parts.len() == 1 {
        return true;
    }

    // 递归检查前面的复合选择器
    matches_selector_recursive(doc, element, parts, last_idx)
}

/// 递归检查选择器链的其余部分。
///
/// 从目标元素开始，沿 DOM 树向上查找匹配的祖先或兄弟。
fn matches_selector_recursive(
    doc: &Document,
    current: NodeId,
    parts: &[(CompoundSelector, Option<Combinator>)],
    part_idx: usize,
) -> bool {
    if part_idx == 0 {
        return true;
    }

    let (prev_compound, combinator) = &parts[part_idx - 1];

    match combinator {
        Some(Combinator::Descendant) => {
            // 后代组合器：在任何祖先中查找匹配
            let mut ancestor = doc.parent_node(current);
            while let Some(ancestor_id) = ancestor {
                if matches_compound(doc, ancestor_id, prev_compound)
                    && matches_selector_recursive(doc, ancestor_id, parts, part_idx - 1)
                {
                    return true;
                }
                ancestor = doc.parent_node(ancestor_id);
            }
            false
        }
        Some(Combinator::Child) => {
            // 子组合器：只在直接父元素中查找
            if let Some(parent) = doc.parent_node(current)
                && matches_compound(doc, parent, prev_compound)
            {
                return matches_selector_recursive(doc, parent, parts, part_idx - 1);
            }
            false
        }
        Some(Combinator::NextSibling) => {
            // 相邻兄弟组合器：只检查前一个兄弟
            if let Some(prev) = doc.previous_sibling(current)
                && is_element(doc, prev) && matches_compound(doc, prev, prev_compound)
            {
                return matches_selector_recursive(doc, prev, parts, part_idx - 1);
            }
            false
        }
        Some(Combinator::SubsequentSibling) => {
            // 通用兄弟组合器：检查所有前面的兄弟
            let mut sibling = doc.previous_sibling(current);
            while let Some(sibling_id) = sibling {
                if is_element(doc, sibling_id)
                    && matches_compound(doc, sibling_id, prev_compound)
                    && matches_selector_recursive(doc, sibling_id, parts, part_idx - 1)
                {
                    return true;
                }
                sibling = doc.previous_sibling(sibling_id);
            }
            false
        }
        None => {
            // 没有组合器（不应该发生），尝试继续
            matches_selector_recursive(doc, current, parts, part_idx - 1)
        }
    }
}

/// 检查元素是否匹配复合选择器。
fn matches_compound(doc: &Document, element: NodeId, compound: &CompoundSelector) -> bool {
    // 检查类型选择器
    if let Some(type_sel) = &compound.type_selector
        && !matches_type(doc, element, type_sel)
    {
        return false;
    }

    // 检查所有子类选择器
    for sub in &compound.subclass_selectors {
        if !matches_subclass(doc, element, sub) {
            return false;
        }
    }

    true
}

/// 检查类型选择器是否匹配。
fn matches_type(doc: &Document, element: NodeId, type_sel: &TypeSelector) -> bool {
    let node = match doc.get(element) {
        Some(n) => n,
        None => return false,
    };

    match &node.kind {
        NodeKind::Element(elem) => match type_sel {
            TypeSelector::Universal => true,
            TypeSelector::Tag(tag) => elem.local_name().eq_ignore_ascii_case(tag),
        },
        _ => false,
    }
}

/// 检查子类选择器是否匹配。
fn matches_subclass(doc: &Document, element: NodeId, sub: &SubclassSelector) -> bool {
    match sub {
        SubclassSelector::Id(id) => matches_id(doc, element, id),
        SubclassSelector::Class(cls) => matches_class(doc, element, cls),
        SubclassSelector::Attribute(attr) => matches_attribute(doc, element, attr),
        SubclassSelector::PseudoClass(pc) => matches_pseudo_class(doc, element, pc),
        SubclassSelector::PseudoElement(_) => {
            // 伪元素不匹配 DOM 元素
            false
        }
    }
}

/// 检查 ID 选择器是否匹配。
fn matches_id(doc: &Document, element: NodeId, id: &str) -> bool {
    doc.get_attribute(element, "id").is_some_and(|v| v == id)
}

/// 检查类选择器是否匹配。
fn matches_class(doc: &Document, element: NodeId, cls: &str) -> bool {
    let node = match doc.get(element) {
        Some(n) => n,
        None => return false,
    };

    match &node.kind {
        NodeKind::Element(elem) => elem.class_list.iter().any(|c| c == cls),
        _ => false,
    }
}

/// 检查属性选择器是否匹配。
fn matches_attribute(doc: &Document, element: NodeId, attr: &AttributeSelector) -> bool {
    let value = match doc.get_attribute(element, &attr.name) {
        Some(v) => v,
        None => return false,
    };

    match &attr.matcher {
        AttributeMatcher::Exists => true,
        AttributeMatcher::Exact(v) => &value == v,
        AttributeMatcher::Includes(v) => value
            .split_whitespace()
            .any(|part| part == v),
        AttributeMatcher::DashMatch(v) => {
            value == *v || value.starts_with(&format!("{v}-"))
        }
        AttributeMatcher::Prefix(v) => value.starts_with(v),
        AttributeMatcher::Suffix(v) => value.ends_with(v),
        AttributeMatcher::Substring(v) => value.contains(v),
    }
}

/// 检查伪类选择器是否匹配。
///
/// 支持有限集：`:first-child`, `:last-child`, `:root`, `:empty`, `:nth-child()`。
fn matches_pseudo_class(doc: &Document, element: NodeId, pc: &PseudoClassSelector) -> bool {
    match pc {
        PseudoClassSelector::Simple(name) => match name.as_str() {
            "first-child" => is_first_child(doc, element),
            "last-child" => is_last_child(doc, element),
            "root" => is_root_element(doc, element),
            "empty" => is_empty_element(doc, element),
            _ => false, // 不支持的伪类
        },
        PseudoClassSelector::Not(selectors) => {
            // :not() 匹配不满足任一选择器的元素
            !selectors.iter().any(|s| matches_selector(doc, element, s))
        }
        PseudoClassSelector::Is(selectors) => {
            // :is() 匹配满足任一选择器的元素
            selectors.iter().any(|s| matches_selector(doc, element, s))
        }
        PseudoClassSelector::Where(selectors) => {
            // :where() 匹配逻辑同 :is()
            selectors.iter().any(|s| matches_selector(doc, element, s))
        }
        PseudoClassSelector::NthChild(pattern) => matches_nth_child(doc, element, pattern),
        _ => false, // 其他伪类暂不支持
    }
}

/// 检查元素是否为第一个子元素。
fn is_first_child(doc: &Document, element: NodeId) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);
    // 找到第一个元素子节点
    for &child in &children {
        if is_element(doc, child) {
            return child == element;
        }
    }
    false
}

/// 检查元素是否为最后一个子元素。
fn is_last_child(doc: &Document, element: NodeId) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);
    // 从后往前找到最后一个元素子节点
    for &child in children.iter().rev() {
        if is_element(doc, child) {
            return child == element;
        }
    }
    false
}

/// 检查元素是否为文档根元素（html）。
fn is_root_element(doc: &Document, element: NodeId) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    // 父节点是文档根节点
    if let Some(node) = doc.get(parent) {
        matches!(node.kind, NodeKind::Document(_))
    } else {
        false
    }
}

/// 检查元素是否为空（没有子元素或文本节点）。
fn is_empty_element(doc: &Document, element: NodeId) -> bool {
    let children = doc.child_nodes(element);
    if children.is_empty() {
        return true;
    }
    // 检查是否只有空文本节点
    for &child in &children {
        if let Some(node) = doc.get(child) {
            match &node.kind {
                NodeKind::Element(_) => return false,
                NodeKind::Text(data) if !data.content.trim().is_empty() => {
                    return false;
                }
                _ => {}
            }
        }
    }
    true
}

/// 检查元素是否匹配 nth-child 模式。
fn matches_nth_child(doc: &Document, element: NodeId, pattern: &zero_css_parser::ast::NthPattern) -> bool {
    let parent = match doc.parent_node(element) {
        Some(p) => p,
        None => return false,
    };
    let children = doc.child_nodes(parent);

    // 计算元素在兄弟中的位置（1-indexed，只计算元素节点）
    let mut index = 0;
    for &child in &children {
        if is_element(doc, child) {
            index += 1;
            if child == element {
                return matches_nth_pattern(index, pattern);
            }
        }
    }
    false
}

/// 检查位置是否匹配 an+b 模式。
fn matches_nth_pattern(index: i32, pattern: &zero_css_parser::ast::NthPattern) -> bool {
    let a = pattern.a;
    let b = pattern.b;

    if a == 0 {
        // 只有 b：精确匹配
        index == b
    } else {
        // an+b：检查 (index - b) 是否能被 a 整除且结果 >= 0
        let diff = index - b;
        if a > 0 {
            diff >= 0 && diff % a == 0
        } else {
            diff <= 0 && diff % a == 0
        }
    }
}

/// 检查节点是否为元素节点。
fn is_element(doc: &Document, node: NodeId) -> bool {
    doc.get(node)
        .map(|n| matches!(n.kind, NodeKind::Element(_)))
        .unwrap_or(false)
}

/// 从样式表中收集匹配指定元素的声明。
///
/// 遍历样式表中所有规则，检查每个选择器是否匹配元素，
/// 返回所有匹配的声明及其特异性。
pub fn collect_matching_declarations(
    doc: &Document,
    element: NodeId,
    stylesheets: &[zero_css_parser::Stylesheet],
) -> Vec<MatchingDecl> {
    let mut results = Vec::new();

    for stylesheet in stylesheets {
        collect_from_rules(doc, element, &stylesheet.rules, &mut results);
    }

    results
}

/// 递归从规则中收集匹配的声明。
fn collect_from_rules(
    doc: &Document,
    element: NodeId,
    rules: &[zero_css_parser::ast::Rule],
    results: &mut Vec<MatchingDecl>,
) {
    for rule in rules {
        match rule {
            zero_css_parser::ast::Rule::Style(style_rule) => {
                // 检查选择器列表中是否有匹配的选择器
                for selector in &style_rule.selectors {
                    if matches_selector(doc, element, selector) {
                        let spec = zero_css_parser::selector::specificity(selector);
                        for decl in &style_rule.declarations {
                            results.push((
                                decl.property.clone(),
                                decl.value.clone(),
                                decl.important,
                                spec,
                            ));
                        }
                        break; // 一个选择器匹配就够了
                    }
                }
            }
            zero_css_parser::ast::Rule::At(at_rule) => {
                if let zero_css_parser::ast::AtRuleBody::Block(inner_rules) = &at_rule.body {
                    collect_from_rules(doc, element, inner_rules, results);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::ast::{
        AttributeMatcher, AttributeSelector, ComplexSelector, CompoundSelector,
        Combinator, PseudoClassSelector, Selector, SubclassSelector, TypeSelector,
    };
    use zero_dom::Document;

    // ── 辅助函数 ──

    fn make_tag_selector(tag: &str) -> Selector {
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

    fn make_id_selector(id: &str) -> Selector {
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

    fn make_class_selector(cls: &str) -> Selector {
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
    fn make_test_dom() -> (Document, NodeId, NodeId, NodeId, NodeId) {
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
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("first-child".to_string()),
                        )],
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
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("root".to_string()),
                        )],
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
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("empty".to_string()),
                        )],
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
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Not(vec![make_id_selector("main")]),
                        )],
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
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Is(vec![
                                make_tag_selector("div"),
                                make_tag_selector("span"),
                            ]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, div, &sel));
        assert!(!matches_selector(&doc, p, &sel));
    }

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
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Simple("last-child".to_string()),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, span, &sel), "span (last child) should match :last-child");
        assert!(!matches_selector(&doc, div, &sel), "div (not last child) should not match :last-child");
    }

    /// 测试 :nth-child(2n) 匹配偶数位置。
    #[test]
    fn test_matches_nth_child_even() {
        let mut doc = Document::new();
        let root = doc.root();
        let body = doc.create_element("body");
        doc.append_child(root, body).unwrap();

        let items: Vec<NodeId> = (0..5).map(|_| {
            let li = doc.create_element("li");
            doc.append_child(body, li).unwrap();
            li
        }).collect();

        // :nth-child(2n) 匹配第 2、4 个
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: None,
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::NthChild(zero_css_parser::ast::NthPattern {
                                a: 2,
                                b: 0,
                            }),
                        )],
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
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Where(vec![
                                make_class_selector("container"),
                                make_class_selector("other"),
                            ]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, div, &sel), "div.container should match :where(.container, .other)");
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
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, elem, &sel), "lang='en-US' should match [lang|=en]");
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
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, elem, &sel), "data-type='button-primary' should match [data-type^=button]");
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
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, elem, &sel), "href ending with '/page' should match [href$='/page']");
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
                        })],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, elem, &sel), "href containing 'example' should match [href*=example]");
    }

    /// 测试类型选择器大小写不敏感。
    #[test]
    fn test_tag_selector_case_insensitive() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let sel = make_tag_selector("DIV");
        assert!(matches_selector(&doc, div, &sel), "DIV should match div (case insensitive)");
    }

    /// 测试空选择器不匹配任何元素。
    #[test]
    fn test_empty_selector_no_match() {
        let (doc, _html, _body, div, _p) = make_test_dom();
        let sel = Selector {
            complex: ComplexSelector {
                parts: vec![],
            },
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
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Not(vec![make_class_selector("container")]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(matches_selector(&doc, p, &sel_not_container), "p without .container should match :not(.container)");

        // p:not(.text) — p 有 text 类，不应匹配
        let sel_not_text = Selector {
            complex: ComplexSelector {
                parts: vec![(
                    CompoundSelector {
                        type_selector: Some(TypeSelector::Tag("p".to_string())),
                        subclass_selectors: vec![SubclassSelector::PseudoClass(
                            PseudoClassSelector::Not(vec![make_class_selector("text")]),
                        )],
                    },
                    None,
                )],
            },
        };
        assert!(!matches_selector(&doc, p, &sel_not_text), "p.text should not match :not(.text)");
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
        assert!(!matches_selector(&doc, div, &sel), "pseudo-element should never match DOM elements");
    }
}
