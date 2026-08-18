//! 表单控件激活事务所需的纯 DOM 查询。

use super::*;

/// details 首个 summary 激活所需状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryActivationSnapshot {
    /// owning details 的唯一选择器。
    pub details_selector: String,
    /// 激活前 open 状态。
    pub open: bool,
}

/// 解析 summary 默认动作；仅 details 的首个 summary 元素子可激活。
/// https://html.spec.whatwg.org/multipage/interactive-elements.html#the-summary-element
pub fn summary_activation_snapshot(html: &str, selector: &str) -> Option<SummaryActivationSnapshot> {
    let doc = parse_html(html);
    let summary = find_by_selector(&doc, selector)?;
    if element_local_name(&doc, summary) != "summary" {
        return None;
    }
    let details = doc.parent_node(summary)?;
    if element_local_name(&doc, details) != "details" {
        return None;
    }
    let first_summary = doc
        .child_nodes(details)
        .iter()
        .copied()
        .find(|child| element_local_name(&doc, *child) == "summary")?;
    if first_summary != summary {
        return None;
    }
    Some(SummaryActivationSnapshot {
        details_selector: unique_selector_for_node(&doc, details)?,
        open: doc.get_attribute(details, "open").is_some(),
    })
}

/// 返回 owner 为指定 form 的 listed controls 唯一选择器，保持文档序。
/// https://html.spec.whatwg.org/multipage/forms.html#category-listed
///
/// FV M3：`input[type=image]` 排除（WPT form-requestsubmit 的 oracle——
/// "<input type=image> is not in form.elements"，与 Chromium 一致）。
pub fn form_control_selectors(html: &str, form_selector: &str) -> Vec<String> {
    let doc = parse_html(html);
    form_control_selectors_doc(&doc, form_selector)
}

/// [`form_control_selectors`] 的 doc 版（js-dom M1 L2 R102：调用方经
/// `with_query_doc_live_aware` 供 doc——live 读 or 快照缓存）。
pub fn form_control_selectors_doc(doc: &Document, form_selector: &str) -> Vec<String> {
    let Some(form) = find_by_selector(doc, form_selector) else {
        return Vec::new();
    };
    if element_local_name(doc, form) != "form" {
        return Vec::new();
    }
    doc.collect_descendants(doc.root())
        .into_iter()
        .filter(|node| {
            let tag = element_local_name(doc, *node);
            matches!(
                tag,
                "button" | "fieldset" | "input" | "object" | "output" | "select" | "textarea"
            ) && form_owner_node(doc, *node) == Some(form)
                && !(tag == "input"
                    && doc
                        .get_attribute(*node, "type")
                        .is_some_and(|value| value.eq_ignore_ascii_case("image")))
        })
        .filter_map(|node| unique_selector_for_node(doc, node))
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
