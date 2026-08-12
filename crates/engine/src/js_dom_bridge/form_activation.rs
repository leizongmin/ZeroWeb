//! 表单控件激活事务所需的纯 DOM 查询。

use super::*;

/// 返回 owner 为指定 form 的 listed controls 唯一选择器，保持文档序。
/// https://html.spec.whatwg.org/multipage/forms.html#category-listed
pub fn form_control_selectors(html: &str, form_selector: &str) -> Vec<String> {
    let doc = parse_html(html);
    let Some(form) = find_by_selector(&doc, form_selector) else {
        return Vec::new();
    };
    if element_local_name(&doc, form) != "form" {
        return Vec::new();
    }
    doc.collect_descendants(doc.root())
        .into_iter()
        .filter(|node| {
            matches!(
                element_local_name(&doc, *node),
                "button" | "fieldset" | "input" | "object" | "output" | "select" | "textarea"
            ) && form_owner_node(&doc, *node) == Some(form)
        })
        .filter_map(|node| unique_selector_for_node(&doc, node))
        .collect()
}

/// 返回目标 radio 同组中当前 checked 项的唯一选择器。
///
/// 无 name 的 radio 只与自身成组；无选中项返回 `None`。
/// https://html.spec.whatwg.org/multipage/input.html#radio-button-group
pub fn checked_radio_group_selector(html: &str, target_selector: &str) -> Option<String> {
    let doc = parse_html(html);
    let target = find_by_selector(&doc, target_selector)?;
    if !is_radio_node(&doc, target) {
        return None;
    }
    let name = doc.get_attribute(target, "name");
    doc.query_selector_all(doc.root(), "input[type=radio]")
        .into_iter()
        .filter(|node| {
            if name.is_none() {
                *node == target
            } else {
                doc.get_attribute(*node, "name") == name
            }
        })
        .find(|node| doc.get_attribute(*node, "checked").is_some())
        .and_then(|node| unique_selector_for_node(&doc, node))
}

fn is_radio_node(doc: &Document, node: NodeId) -> bool {
    doc.get(node).is_some_and(|node| match &node.kind {
        NodeKind::Element(element) => {
            element.local_name().eq_ignore_ascii_case("input")
                && element
                    .get_attribute("type")
                    .is_some_and(|value| value.eq_ignore_ascii_case("radio"))
        }
        _ => false,
    })
}
