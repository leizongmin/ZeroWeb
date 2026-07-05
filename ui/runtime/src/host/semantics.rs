//! Semantics tree — a11y 语义树构建（DC-8 phase-3，P0-2 拆分）。
//!
//! 入口：[`build_semantics`] / [`self_semantics`]。遍历 retained 树，向每个有 widget 的
//! 节点索要自描述 `SemanticsNode`，覆盖 host 已知信息（id/rect），OR 进焦点标志
//! （FOCUSABLE / FOCUSED）。纯容器节点（无 widget）做 semantics merge（子节点上浮），
//! 避免无内容中间节点污染读屏树。

use zero_ui_core::semantics::{SemanticsFlags, SemanticsNode};
use zero_ui_core::widget::{SemanticsCtx, WidgetId};

use super::HostNode;

/// 由一个 retained 节点产出其自身 `SemanticsNode`（不含 children）。
pub(super) fn self_semantics(node: &HostNode, focused: Option<&WidgetId>) -> SemanticsNode {
    let mut pushed: Vec<SemanticsNode> = Vec::new();
    if let Some(w) = node.widget.as_ref() {
        w.semantics(&mut SemanticsCtx { nodes: &mut pushed });
    }
    let mut s = pushed
        .pop()
        .unwrap_or_else(|| SemanticsNode::new(node.id.clone(), node.cached_rect, SemanticsFlags::NONE));
    s.id = node.id.clone();
    s.rect = node.cached_rect;
    if node.focusable {
        s.flags |= SemanticsFlags::FOCUSABLE;
    }
    if focused == Some(&node.id) {
        s.flags |= SemanticsFlags::FOCUSED;
    }
    s
}

/// 递归构建 a11y 树：有 widget 或可聚焦的节点产出独立语义节点；纯容器节点（无 widget）
/// 把子节点合并进父级（semantics merge）。
pub(super) fn build_semantics(node: &HostNode, focused: Option<&WidgetId>, out: &mut Vec<SemanticsNode>) {
    if node.widget.is_some() || node.focusable {
        let mut s = self_semantics(node, focused);
        for child in &node.children {
            build_semantics(child, focused, &mut s.children);
        }
        out.push(s);
    } else {
        for child in &node.children {
            build_semantics(child, focused, out);
        }
    }
}
