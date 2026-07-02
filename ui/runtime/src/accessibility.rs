//! 无障碍运行时（spec FR-011 / DC-8）。
//!
//! 持有从 element/render tree 产出的 `SemanticsNode` 树；标记 `needs_semantics` 时重建。

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
}
