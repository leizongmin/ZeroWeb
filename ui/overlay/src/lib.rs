//! # zero-ui-overlay
//!
//! 浮层系统（spec §8.4.1 `zero-ui-overlay` / FR-016 / §8.4.1B 权限/下载/site info）。
//!
//! M1 提供 OverlayHost（注册/移除浮层条目）+ Toast + focus trap 标志；
//! popover/menu/dialog/sheet 的具体绘制在 M2 组合通用 widgets。

use zero_ui_core::geometry::Rect;
use zero_ui_core::widget::WidgetId;

/// 浮层条目。
#[derive(Debug, Clone, PartialEq)]
pub struct OverlayEntry {
    pub id: WidgetId,
    /// 锚定区域（None = 居中/全屏 modal）。
    pub anchor: Option<Rect>,
    /// 是否捕获焦点（modal/sheet 用）。
    pub trap_focus: bool,
    pub modal: bool,
}

/// 浮层宿主：维护当前活动的浮层列表（按 z 序）。
#[derive(Debug, Default)]
pub struct OverlayHost {
    pub entries: Vec<OverlayEntry>,
}

impl OverlayHost {
    pub fn new() -> OverlayHost {
        OverlayHost::default()
    }

    /// 在顶层插入浮层。
    pub fn push(&mut self, entry: OverlayEntry) {
        self.entries.push(entry);
    }

    /// 移除指定 id 的浮层。
    pub fn dismiss(&mut self, id: &WidgetId) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| &e.id != id);
        self.entries.len() != before
    }

    /// 是否存在任意 modal 浮层（事件路由屏障）。
    pub fn has_modal(&self) -> bool {
        self.entries.iter().any(|e| e.modal)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Toast（瞬态通知）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub message_id: String,
    pub ttl_ms: u32,
}

impl Toast {
    pub fn new(message_id: &str) -> Toast {
        Toast {
            message_id: message_id.to_string(),
            ttl_ms: 3000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_dismiss_modal() {
        let mut host = OverlayHost::new();
        host.push(OverlayEntry {
            id: WidgetId::new("perm"),
            anchor: Some(Rect::ZERO),
            trap_focus: true,
            modal: true,
        });
        assert!(host.has_modal());
        assert_eq!(host.len(), 1);
        assert!(host.dismiss(&WidgetId::new("perm")));
        assert!(!host.has_modal());
    }
}
