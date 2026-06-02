//! 布局脏标记追踪器。
//!
//! [`LayoutDirtyTracker`] 追踪哪些 DOM 节点需要重新布局。
//! 当 DOM 变更（添加/删除/修改节点、样式变更）时，标记受影响的节点为"脏"，
//! 增量布局只重算脏节点及其祖先路径上的节点。
//!
//! ## 使用方式
//!
//! ```ignore
//! let mut tracker = LayoutDirtyTracker::new();
//! tracker.mark_dirty(node_id);              // DOM 变更时标记
//! tracker.mark_subtree_dirty(node_id);      // 子树整体变更
//! let dirty_nodes = tracker.drain_dirty();  // 获取并清除脏标记
//! ```

use std::collections::HashSet;
use zero_dom::NodeId;

/// 布局脏标记追踪器。
///
/// 维护一组需要重新布局的 DOM 节点 ID。
/// 布局引擎根据脏标记决定哪些子树需要重新计算。
#[derive(Debug, Clone, Default)]
pub struct LayoutDirtyTracker {
    /// 脏节点集合。
    dirty_nodes: HashSet<NodeId>,
    /// 是否需要全量重算（如视口大小变化）。
    full_recalc: bool,
}

impl LayoutDirtyTracker {
    /// 创建空的追踪器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 标记单个节点为脏（需要重新布局）。
    ///
    /// 当节点的样式或内容发生局部变更时调用。
    pub fn mark_dirty(&mut self, node_id: NodeId) {
        self.dirty_nodes.insert(node_id);
    }

    /// 标记节点及其所有子节点为脏。
    ///
    /// 当节点被替换、样式发生结构性变化时调用。
    /// 注意：调用方需负责遍历 DOM 子树并调用 `mark_dirty`。
    pub fn mark_subtree_dirty(&mut self, node_ids: &[NodeId]) {
        for &id in node_ids {
            self.dirty_nodes.insert(id);
        }
    }

    /// 标记需要全量重算。
    ///
    /// 在视口大小变化等影响全局布局的场景调用。
    pub fn mark_full_recalc(&mut self) {
        self.full_recalc = true;
    }

    /// 检查是否需要全量重算。
    pub fn is_full_recalc(&self) -> bool {
        self.full_recalc
    }

    /// 检查指定节点是否为脏。
    pub fn is_dirty(&self, node_id: NodeId) -> bool {
        self.full_recalc || self.dirty_nodes.contains(&node_id)
    }

    /// 检查是否有任何脏节点。
    pub fn has_dirty(&self) -> bool {
        self.full_recalc || !self.dirty_nodes.is_empty()
    }

    /// 获取所有脏节点 ID（消耗性，获取后清空）。
    pub fn drain_dirty(&mut self) -> Vec<NodeId> {
        let result: Vec<NodeId> = self.dirty_nodes.drain().collect();
        self.full_recalc = false;
        result
    }

    /// 获取脏节点数量。
    pub fn dirty_count(&self) -> usize {
        self.dirty_nodes.len()
    }

    /// 清除所有脏标记。
    pub fn clear(&mut self) {
        self.dirty_nodes.clear();
        self.full_recalc = false;
    }

    /// 检查追踪器是否为空（无脏标记）。
    pub fn is_empty(&self) -> bool {
        !self.full_recalc && self.dirty_nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_dom::Document;

    /// 辅助函数：创建测试用的 NodeId。
    fn make_ids(n: usize) -> Vec<NodeId> {
        let mut doc = Document::new();
        let mut ids = Vec::with_capacity(n);
        for _ in 0..n {
            ids.push(doc.create_element("div"));
        }
        ids
    }

    #[test]
    fn test_new_tracker_is_empty() {
        let tracker = LayoutDirtyTracker::new();
        assert!(tracker.is_empty());
        assert!(!tracker.has_dirty());
        assert_eq!(tracker.dirty_count(), 0);
    }

    #[test]
    fn test_mark_dirty() {
        let ids = make_ids(2);
        let mut tracker = LayoutDirtyTracker::new();
        tracker.mark_dirty(ids[0]);
        assert!(tracker.has_dirty());
        assert!(tracker.is_dirty(ids[0]));
        assert!(!tracker.is_dirty(ids[1]));
        assert_eq!(tracker.dirty_count(), 1);
    }

    #[test]
    fn test_mark_subtree_dirty() {
        let ids = make_ids(3);
        let mut tracker = LayoutDirtyTracker::new();
        tracker.mark_subtree_dirty(&ids);
        assert_eq!(tracker.dirty_count(), 3);
        assert!(tracker.is_dirty(ids[0]));
        assert!(tracker.is_dirty(ids[1]));
        assert!(tracker.is_dirty(ids[2]));
    }

    #[test]
    fn test_mark_full_recalc() {
        let ids = make_ids(1);
        let mut tracker = LayoutDirtyTracker::new();
        tracker.mark_full_recalc();
        assert!(tracker.is_full_recalc());
        assert!(tracker.has_dirty());
        // 全量重算时所有节点都视为脏
        assert!(tracker.is_dirty(ids[0]));
    }

    #[test]
    fn test_drain_dirty() {
        let ids = make_ids(2);
        let mut tracker = LayoutDirtyTracker::new();
        tracker.mark_dirty(ids[0]);
        tracker.mark_dirty(ids[1]);
        let dirty = tracker.drain_dirty();
        assert_eq!(dirty.len(), 2);
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_drain_dirty_clears_full_recalc() {
        let mut tracker = LayoutDirtyTracker::new();
        tracker.mark_full_recalc();
        let _ = tracker.drain_dirty();
        assert!(!tracker.is_full_recalc());
        assert!(tracker.is_empty());
    }

    #[test]
    fn test_clear() {
        let ids = make_ids(1);
        let mut tracker = LayoutDirtyTracker::new();
        tracker.mark_dirty(ids[0]);
        tracker.mark_full_recalc();
        tracker.clear();
        assert!(tracker.is_empty());
        assert!(!tracker.is_full_recalc());
    }

    #[test]
    fn test_duplicate_mark_dirty() {
        let ids = make_ids(1);
        let mut tracker = LayoutDirtyTracker::new();
        tracker.mark_dirty(ids[0]);
        tracker.mark_dirty(ids[0]);
        assert_eq!(tracker.dirty_count(), 1);
    }

    #[test]
    fn test_mark_dirty_then_check_not_dirty_after_drain() {
        let ids = make_ids(1);
        let mut tracker = LayoutDirtyTracker::new();
        tracker.mark_dirty(ids[0]);
        assert!(tracker.is_dirty(ids[0]));
        let _ = tracker.drain_dirty();
        assert!(!tracker.is_dirty(ids[0]));
    }
}
