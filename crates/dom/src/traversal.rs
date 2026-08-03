//! DOM 树遍历迭代器（从 `document.rs` 抽出，run-rules §5 文件大小控制）。
//!
//! 含 `TreeWalker` 与 `NodeIterator` —— WHATWG DOM 规范的深度优先遍历接口，
//! 基于 `NodeId` + `&Document`（与 crate slotmap 架构一致）。

use crate::{Document, NodeId};

// ── TreeWalker ──────────────────────────────────────────────────────

/// DOM TreeWalker — 提供深度优先遍历 DOM 子树的能力。
///
/// 遵循 WHATWG DOM 规范中 `TreeWalker` 接口的核心语义。
/// 使用 `NodeId` 和 `&Document` 进行遍历，与 crate 的 slotmap 架构一致。
pub struct TreeWalker {
    /// 遍历的根节点。
    root: NodeId,
    /// 当前节点位置。
    current: NodeId,
    /// 节点类型过滤位掩码（0xFFFFFFFF = 显示所有节点）。
    #[expect(dead_code)]
    what_to_show: u32,
}

impl TreeWalker {
    /// 创建新的 TreeWalker。
    pub fn new(root: NodeId, what_to_show: u32) -> Self {
        let current = root;
        Self {
            root,
            current,
            what_to_show,
        }
    }

    /// 移动到下一个节点（深度优先，文档顺序，前序遍历）。
    ///
    /// 遍历顺序：先尝试第一个子节点，然后下一个兄弟节点，
    /// 最后向上回溯到父节点并尝试父节点的下一个兄弟节点。
    /// 当到达根节点的父级时停止。
    pub fn next_node(&mut self, doc: &Document) -> Option<NodeId> {
        // 尝试第一个子节点
        if let Some(child) = doc.first_child(self.current) {
            self.current = child;
            return Some(self.current);
        }

        // 尝试下一个兄弟节点，或向上回溯
        let mut node = self.current;
        loop {
            if node == self.root {
                return None;
            }
            if let Some(sibling) = doc.next_sibling(node) {
                self.current = sibling;
                return Some(self.current);
            }
            // 回溯到父节点继续查找
            node = doc.parent_node(node)?;
        }
    }

    /// 移动到当前节点的第一个子节点。
    pub fn first_child(&mut self, doc: &Document) -> Option<NodeId> {
        let child = doc.first_child(self.current)?;
        self.current = child;
        Some(self.current)
    }

    /// 移动到当前节点的下一个兄弟节点。
    pub fn next_sibling(&mut self, doc: &Document) -> Option<NodeId> {
        let sibling = doc.next_sibling(self.current)?;
        self.current = sibling;
        Some(self.current)
    }

    /// 获取当前节点。
    pub fn current_node(&self) -> NodeId {
        self.current
    }

    /// 获取根节点。
    pub fn root(&self) -> NodeId {
        self.root
    }
}

// ── NodeIterator ─────────────────────────────────────────────────────

/// DOM NodeIterator — 提供遍历 DOM 子树中节点列表的能力。
///
/// 遵循 WHATWG DOM 规范中 `NodeIterator` 接口的核心语义。
/// 与 [`TreeWalker`] 不同，`NodeIterator` 不维护子树层级导航方法，
/// 仅支持向前/向后遍历，且使用 `done` 标志标记遍历结束。
pub struct NodeIterator {
    /// 遍历的根节点。
    root: NodeId,
    /// 当前节点位置。
    current: NodeId,
    /// 节点类型过滤位掩码（0xFFFFFFFF = 显示所有节点）。
    #[expect(dead_code)]
    what_to_show: u32,
    /// 是否已遍历完毕。
    done: bool,
}

impl NodeIterator {
    /// 创建新的 NodeIterator。
    pub fn new(root: NodeId, what_to_show: u32) -> Self {
        let current = root;
        Self {
            root,
            current,
            what_to_show,
            done: false,
        }
    }

    /// 移动到下一个节点并返回它（深度优先，文档顺序，前序遍历）。
    ///
    /// 遍历顺序与 [`TreeWalker::next_node`] 相同：
    /// 先尝试第一个子节点，然后下一个兄弟节点，
    /// 最后向上回溯到父节点并尝试父节点的下一个兄弟节点。
    /// 当遍历完根节点的所有后代后标记为 `done`，返回 `None`。
    pub fn next_node(&mut self, doc: &Document) -> Option<NodeId> {
        if self.done {
            return None;
        }

        // 尝试第一个子节点
        if let Some(child) = doc.first_child(self.current) {
            self.current = child;
            return Some(self.current);
        }

        // 尝试下一个兄弟节点，或向上回溯
        let mut node = self.current;
        loop {
            if node == self.root {
                self.done = true;
                return None;
            }
            if let Some(sibling) = doc.next_sibling(node) {
                self.current = sibling;
                return Some(self.current);
            }
            // 回溯到父节点继续查找
            node = doc.parent_node(node)?;
        }
    }

    /// 移动到上一个节点并返回它。
    ///
    /// 按文档顺序的反方向移动：
    /// 先尝试上一个兄弟节点的最后一个后代，
    /// 然后尝试上一个兄弟节点本身，
    /// 最后回退到父节点。
    /// 如果已在根节点，返回 `None`。
    pub fn previous_node(&mut self, doc: &Document) -> Option<NodeId> {
        if self.done {
            // 如果之前遍历完毕，重置 done 并从最后一个位置回退
            self.done = false;
        }

        // 尝试上一个兄弟节点的最深后代
        if self.current != self.root
            && let Some(sibling) = doc.previous_sibling(self.current)
        {
            // 找到 sibling 的最深最后一个后代
            let mut deepest = sibling;
            while let Some(last_child) = doc.last_child(deepest) {
                deepest = last_child;
            }
            self.current = deepest;
            return Some(self.current);
        }

        // 回退到父节点
        if self.current == self.root {
            return None;
        }
        if let Some(parent) = doc.parent_node(self.current) {
            if parent == self.root {
                // 根节点不返回
                return None;
            }
            self.current = parent;
            return Some(self.current);
        }

        None
    }

    /// 获取当前节点。
    pub fn current_node(&self) -> NodeId {
        self.current
    }

    /// 获取根节点。
    pub fn root(&self) -> NodeId {
        self.root
    }

    /// 是否已遍历完毕。
    pub fn is_done(&self) -> bool {
        self.done
    }
}
