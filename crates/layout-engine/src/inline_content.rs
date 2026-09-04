//! Inline-content predicates shared by layout postprocessing passes.

use std::collections::HashMap;

use zero_css_parser::values::DisplayValue;
use zero_dom::{Document, NodeId, NodeKind};
use zero_style_system::ComputedStyle;

pub(crate) fn has_direct_text(doc: &Document, dom_id: NodeId) -> bool {
    doc.child_nodes(dom_id).iter().any(|child_id| {
        matches!(
            doc.get(*child_id).map(|node| &node.kind),
            // CSS Text §3：空白判定须用可折叠白空间集合（排除 U+00A0 nbsp）——
            // Rust `trim()` 按 White_Space 属性剔除 nbsp，会把仅含 `&nbsp;` 的元素
            // 误判为无 inline 内容 → IFC 不跑 → 行盒消失高度 0
            //（line-height-applies-to-004/014 行高 192 塌 0 实证）。
            // R1085 修 IFC 分词时已同语义（is_collapsible_ws 排除 nbsp），本处补齐 gate 面。
            // 注意：不可用 `trim()` 或 `is_whitespace()` 兜底——两者都把 nbsp 视为空白，
            // 等于恢复原 bug。
            Some(NodeKind::Text(text))
                if text.content.chars().any(|ch| !crate::inline::is_collapsible_ws(ch))
        )
    })
}

/// Checks for direct text or inline-level element children.
pub(crate) fn has_inline_content(doc: &Document, styles: &HashMap<NodeId, ComputedStyle>, dom_id: NodeId) -> bool {
    if has_direct_text(doc, dom_id) {
        return true;
    }

    doc.child_nodes(dom_id).iter().any(|child_id| {
        matches!(
            (doc.get(*child_id), styles.get(child_id)),
            (Some(node), Some(style))
                if matches!(&node.kind, NodeKind::Element(_))
                    && matches!(style.display, DisplayValue::Inline | DisplayValue::InlineBlock)
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// R4034：nbsp（U+00A0）是 preserved 内容——仅含 `&nbsp;` 的元素**有** inline 内容。
    /// 旧实现 `text.content.trim().is_empty()`（Rust trim 按 White_Space 剔除 nbsp）把
    /// nbsp-only 元素误判为无 inline 内容 → measure 回 0 → 行盒消失（CSS §10.8.1 strut
    /// 缺失，line-height-applies-to-004 行高 192 塌 0 实证，corpus +10 修复面）。
    #[test]
    fn r4034_nbsp_only_element_has_inline_content() {
        let mut doc = Document::new();
        let root = doc.root();
        let div = doc.create_element("div");
        let _ = doc.append_child(root, div);
        let text = doc.create_text_node("\u{00A0}");
        let _ = doc.append_child(div, text);

        let styles: HashMap<NodeId, ComputedStyle> = HashMap::new();
        assert!(
            has_direct_text(&doc, div),
            "nbsp-only 元素应有直接文本（nbsp 是 preserved 内容）"
        );
        assert!(
            has_inline_content(&doc, &styles, div),
            "nbsp-only 元素应有 inline 内容（须跑 IFC 产生行盒）"
        );
    }

    /// R4034 对照锚：纯可折叠空白（空格/换行）仍判无 inline 内容——不改变既有行为
    ///（test_has_direct_text_whitespace_only 的 nbsp-面补充）。
    #[test]
    fn r4034_collapsible_ws_only_still_has_no_text() {
        let mut doc = Document::new();
        let root = doc.root();
        let div = doc.create_element("div");
        let _ = doc.append_child(root, div);
        let text = doc.create_text_node(" \n\t ");
        let _ = doc.append_child(div, text);

        assert!(!has_direct_text(&doc, div), "纯可折叠空白仍应判无直接文本");
    }
}
