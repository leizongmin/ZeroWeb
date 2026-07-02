//! 无障碍语义模型（spec FR-011 / DC-8）。
//!
//! `SemanticsNode` 是 UI SDK 暴露给 a11y 层（屏幕阅读器/自动化）的统一节点。
//! 由 Element/Render tree 在 `Widget::semantics` 中产出。

use crate::geometry::Rect;
use crate::widget::WidgetId;
use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// 语义动作（无障碍/自动化可触发的命令）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticsAction {
    Tap,
    Focus,
    SetText(String),
    /// 增量调节（slider、进度）。
    Adjust(i32),
}

/// 语义标志（角色/能力位）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SemanticsFlags(pub u16);

impl SemanticsFlags {
    pub const NONE: SemanticsFlags = SemanticsFlags(0);
    pub const FOCUSABLE: SemanticsFlags = SemanticsFlags(1 << 0);
    pub const FOCUSED: SemanticsFlags = SemanticsFlags(1 << 1);
    pub const READ_ONLY: SemanticsFlags = SemanticsFlags(1 << 2);
    pub const TEXT_FIELD: SemanticsFlags = SemanticsFlags(1 << 3);
    pub const BUTTON: SemanticsFlags = SemanticsFlags(1 << 4);
    pub const LINK: SemanticsFlags = SemanticsFlags(1 << 5);

    pub fn contains(self, f: SemanticsFlags) -> bool {
        (self.0 & f.0) == f.0
    }
}

impl std::ops::BitOr for SemanticsFlags {
    type Output = SemanticsFlags;
    fn bitor(self, rhs: SemanticsFlags) -> SemanticsFlags {
        SemanticsFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for SemanticsFlags {
    fn bitor_assign(&mut self, rhs: SemanticsFlags) {
        self.0 |= rhs.0;
    }
}

/// 可访问性标签来源（优先 message id，避免硬编码可见文案；spec FR-013）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SemanticsLabel {
    /// 直接字面量（仅限调试/无 i18n 场景）。
    Literal(CompactString),
    /// i18n message id（推荐）。
    Message(CompactString),
}

/// 语义节点。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SemanticsNode {
    pub id: WidgetId,
    pub rect: Rect,
    pub flags: SemanticsFlags,
    pub label: Option<SemanticsLabel>,
    /// 文本字段当前值（供读屏朗读）。
    pub value: Option<CompactString>,
    pub children: Vec<SemanticsNode>,
}

impl SemanticsNode {
    pub fn new(id: WidgetId, rect: Rect, flags: SemanticsFlags) -> SemanticsNode {
        SemanticsNode {
            id,
            rect,
            flags,
            label: None,
            value: None,
            children: Vec::new(),
        }
    }

    /// 收集所有带 `FOCUSABLE` 的节点（焦点/a11y 遍历用）。
    pub fn collect_focusable(&self, out: &mut Vec<WidgetId>) {
        if self.flags.contains(SemanticsFlags::FOCUSABLE) {
            out.push(self.id.clone());
        }
        for c in &self.children {
            c.collect_focusable(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn collect_focusable_descendants() {
        let root = SemanticsNode {
            id: WidgetId::new("root"),
            rect: Rect::from_origin_size(Point::ZERO, crate::geometry::Size::new(10.0, 10.0)),
            flags: SemanticsFlags::NONE,
            label: None,
            value: None,
            children: vec![
                SemanticsNode::new(
                    WidgetId::new("btn"),
                    Rect::ZERO,
                    SemanticsFlags::BUTTON | SemanticsFlags::FOCUSABLE,
                ),
                SemanticsNode::new(WidgetId::new("label"), Rect::ZERO, SemanticsFlags::NONE),
            ],
        };
        let mut f = Vec::new();
        root.collect_focusable(&mut f);
        assert_eq!(f, vec![WidgetId::new("btn")]);
    }
}
