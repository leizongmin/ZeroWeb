//! 无障碍运行时（spec FR-011 / DC-8）。
//!
//! 持有从 element/render tree 产出的 `SemanticsNode` 树；标记 `needs_semantics` 时重建。
//!
//! DC-8 平台桥接（本轮）：[`AccessibilityBackend`] trait 把语义树推送到平台屏幕阅读器后端
//! （Windows UI Automation / macOS NSAccessibility / Linux AT-SPI 等在 M4 实现）；本模块提供
//! 平台无关 trait + [`RecordingAccessibilityBackend`] 测试 mock，[`AccessibilityTree`] 经
//! [`update_backend`](AccessibilityTree::update_backend) 驱动后端。

use zero_ui_core::semantics::SemanticsNode;
use zero_ui_core::widget::WidgetId;

#[derive(Debug, Default, Clone)]
pub struct AccessibilityTree {
    root: Option<SemanticsNode>,
}

impl AccessibilityTree {
    pub fn new() -> AccessibilityTree {
        AccessibilityTree::default()
    }

    pub fn set_root(&mut self, node: SemanticsNode) {
        self.root = Some(node);
    }

    /// 收集所有可聚焦节点（供焦点遍历/读屏）。
    pub fn focusables(&self) -> Vec<WidgetId> {
        let mut out = Vec::new();
        if let Some(root) = &self.root {
            root.collect_focusable(&mut out);
        }
        out
    }

    pub fn root(&self) -> Option<&SemanticsNode> {
        self.root.as_ref()
    }

    /// 把当前语义树推送到平台 a11y 后端（DC-8 平台桥接）。
    ///
    /// 宿主在语义树重建后调用；后端据此更新平台无障碍树（真实平台实现见 M4）。
    pub fn update_backend(&self, backend: &mut dyn AccessibilityBackend) {
        backend.update_tree(self.root.as_ref());
    }
}

/// 平台无障碍后端（spec FR-011 / DC-8 平台桥接）。
///
/// 把 SDK 语义树 + 焦点/通告事件桥接到平台屏幕阅读器 API。SDK 侧只定义契约；
/// 真实平台实现（Windows UI Automation / macOS NSAccessibility / Linux AT-SPI / 移动端 TalkBack 等）
/// 在 M4 runtime adapter 落地。测试用 [`RecordingAccessibilityBackend`] mock。
pub trait AccessibilityBackend {
    /// 语义树更新（每帧/失效后推送）；`None` = 空树（清空平台侧）。
    fn update_tree(&mut self, root: Option<&SemanticsNode>);
    /// 焦点移动（`None` = 失焦）；平台据此朗读焦点控件。
    fn focus_moved(&mut self, focused: Option<WidgetId>);
    /// 短暂通告（如「页面已加载」「已复制」）；平台屏幕阅读器朗读。
    fn announce(&mut self, message: &str);
}

/// 计算语义树节点总数（含根；DFS）。
pub fn node_count(root: Option<&SemanticsNode>) -> usize {
    match root {
        None => 0,
        Some(n) => 1 + n.children.iter().map(|c| node_count(Some(c))).sum::<usize>(),
    }
}

/// 记录型 a11y 后端（测试用 mock）。
///
/// 记录 `update_tree`/`focus_moved`/`announce` 调用，供测试断言 SDK→后端桥接正确。
#[derive(Debug, Default, Clone)]
pub struct RecordingAccessibilityBackend {
    /// 每次 update_tree 推送的节点数（None = 空树）。
    pub tree_updates: Vec<Option<usize>>,
    /// 每次 focus_moved 的焦点 id。
    pub focus_moves: Vec<Option<WidgetId>>,
    /// 每次 announce 的文案。
    pub announcements: Vec<String>,
}

impl RecordingAccessibilityBackend {
    pub fn new() -> RecordingAccessibilityBackend {
        RecordingAccessibilityBackend::default()
    }
}

impl AccessibilityBackend for RecordingAccessibilityBackend {
    fn update_tree(&mut self, root: Option<&SemanticsNode>) {
        self.tree_updates.push(root.map(|_| node_count(root)));
    }

    fn focus_moved(&mut self, focused: Option<WidgetId>) {
        self.focus_moves.push(focused);
    }

    fn announce(&mut self, message: &str) {
        self.announcements.push(message.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::Rect;
    use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
    use zero_ui_core::widget::WidgetId;

    fn leaf(id: &str, label: &str, focusable: bool) -> SemanticsNode {
        let flags = if focusable {
            SemanticsFlags::FOCUSABLE
        } else {
            SemanticsFlags::NONE
        };
        let mut n = SemanticsNode::new(WidgetId::new(id), Rect::from_ltrb(0.0, 0.0, 10.0, 10.0), flags);
        n.label = Some(SemanticsLabel::Literal(label.into()));
        n
    }

    fn sample_tree() -> SemanticsNode {
        // root(a) → [b(focusable), c → [d(focusable)]]
        let mut c = leaf("c", "C", false);
        c.children.push(leaf("d", "D", true));
        let mut root = SemanticsNode::new(
            WidgetId::new("a"),
            Rect::from_ltrb(0.0, 0.0, 100.0, 100.0),
            SemanticsFlags::NONE,
        );
        root.children = vec![leaf("b", "B", true), c];
        root
    }

    #[test]
    fn node_count_walks_full_tree() {
        assert_eq!(node_count(None), 0);
        assert_eq!(node_count(Some(&sample_tree())), 4);
    }

    #[test]
    fn update_backend_pushes_tree_to_backend() {
        // DC-8 平台桥接：AccessibilityTree.update_backend → backend.update_tree（节点数）。
        let mut tree = AccessibilityTree::new();
        let mut rec = RecordingAccessibilityBackend::new();
        // 空树 → None。
        tree.update_backend(&mut rec);
        assert_eq!(rec.tree_updates, vec![None]);
        // 非空树 → Some(4)。
        tree.set_root(sample_tree());
        tree.update_backend(&mut rec);
        assert_eq!(rec.tree_updates, vec![None, Some(4)]);
    }

    #[test]
    fn recording_backend_records_focus_and_announcements() {
        let mut rec = RecordingAccessibilityBackend::new();
        let id = WidgetId::new("b");
        rec.focus_moved(Some(id.clone()));
        rec.focus_moved(None);
        rec.announce("Page loaded");
        assert_eq!(rec.focus_moves, vec![Some(id), None]);
        assert_eq!(rec.announcements, vec!["Page loaded".to_string()]);
    }

    #[test]
    fn focusables_match_pushed_tree() {
        // 一致性：tree.focusables() 与推送给后端的树的 collect_focusable 一致。
        let mut tree = AccessibilityTree::new();
        tree.set_root(sample_tree());
        let focusable = tree.focusables();
        assert_eq!(focusable.len(), 2, "b + d are focusable");
        let mut rec = RecordingAccessibilityBackend::new();
        tree.update_backend(&mut rec);
        assert_eq!(rec.tree_updates.last().unwrap(), &Some(4));
    }
}
