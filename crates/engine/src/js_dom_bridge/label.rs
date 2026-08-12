//! HTML label 与 labelable control 的关联解析。

use super::*;

/// 返回 `<label>` 关联的首个 labelable control 唯一选择器。
///
/// `for` 属性优先；缺少 `for` 时查找首个 labelable 后代。hidden input 不可关联。
/// https://html.spec.whatwg.org/multipage/forms.html#labeled-control
pub fn associated_label_control_selector(html: &str, label_selector: &str) -> Option<String> {
    let doc = parse_html(html);
    let label = find_by_selector(&doc, label_selector)?;
    let is_label = doc.get(label).is_some_and(|node| match &node.kind {
        NodeKind::Element(element) => element.local_name().eq_ignore_ascii_case("label"),
        _ => false,
    });
    if !is_label {
        return None;
    }
    if let Some(target_id) = doc.get_attribute(label, "for").filter(|id| !id.is_empty()) {
        return doc
            .query_selector_all(doc.root(), "[id]")
            .into_iter()
            .find(|node| doc.get_attribute(*node, "id").as_deref() == Some(target_id.as_str()))
            .filter(|node| is_labelable_control(&doc, *node))
            .and_then(|node| unique_selector_for_node(&doc, node));
    }
    first_labelable_descendant(&doc, label).and_then(|node| unique_selector_for_node(&doc, node))
}

fn first_labelable_descendant(doc: &Document, root: NodeId) -> Option<NodeId> {
    for child in doc.child_nodes(root) {
        if is_labelable_control(doc, child) {
            return Some(child);
        }
        if let Some(found) = first_labelable_descendant(doc, child) {
            return Some(found);
        }
    }
    None
}

fn is_labelable_control(doc: &Document, node: NodeId) -> bool {
    let Some(tag) = doc.get(node).and_then(|node| match &node.kind {
        NodeKind::Element(element) => Some(element.local_name()),
        _ => None,
    }) else {
        return false;
    };
    if tag.eq_ignore_ascii_case("input") {
        return !doc
            .get_attribute(node, "type")
            .is_some_and(|value| value.eq_ignore_ascii_case("hidden"));
    }
    matches!(tag, "button" | "meter" | "output" | "progress" | "select" | "textarea")
}
