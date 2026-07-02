//! NavigationButtons — 导航按钮组（spec §8.4.1A）。
//!
//! 由通用 `IconButton`/`Toolbar` 组合绘制（输出进入统一 UI scene，不绕过 ui/render）。
//! 点击只发出 `BrowserAction`；`can_go_back/can_go_forward/loading` 来自 browser-shell navigation state。

use crate::browser_action::BrowserAction;
use zero_ui_widgets::toolbar::{Toolbar, ToolbarItem};

/// 导航按钮组状态（props）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NavigationButtons {
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub loading: bool,
}

impl NavigationButtons {
    pub fn new(can_go_back: bool, can_go_forward: bool, loading: bool) -> NavigationButtons {
        NavigationButtons {
            can_go_back,
            can_go_forward,
            loading,
        }
    }

    /// 组合通用 toolbar（M1：条目构造；enabled 由 nav 状态决定；绘制走 ui/widgets）。
    pub fn build_toolbar(&self) -> Toolbar {
        let mut tb = Toolbar::new();
        tb.push(ToolbarItem::new("back", "browser.go_back"));
        tb.push(ToolbarItem::new("forward", "browser.go_forward"));
        // loading 时显示 stop，否则 reload（M2 用状态切图标）。
        tb.push(ToolbarItem::new("reload_stop", "browser.reload_stop"));
        tb
    }

    /// toolbar item id → BrowserAction；不满足条件时返回 None（按钮 disabled）。
    pub fn on_activate(&self, action_id: &str) -> Option<BrowserAction> {
        match action_id {
            "browser.go_back" if self.can_go_back => Some(BrowserAction::GoBack),
            "browser.go_forward" if self.can_go_forward => Some(BrowserAction::GoForward),
            "browser.reload_stop" => {
                if self.loading {
                    Some(BrowserAction::Stop)
                } else {
                    Some(BrowserAction::Reload)
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_when_no_history() {
        let nb = NavigationButtons::new(false, false, false);
        assert_eq!(nb.on_activate("browser.go_back"), None);
        assert_eq!(nb.on_activate("browser.go_forward"), None);
        // reload 在非 loading 时可用。
        assert_eq!(nb.on_activate("browser.reload_stop"), Some(BrowserAction::Reload));
    }

    #[test]
    fn enabled_emits_action() {
        let nb = NavigationButtons::new(true, false, true);
        assert_eq!(nb.on_activate("browser.go_back"), Some(BrowserAction::GoBack));
        // loading → stop。
        assert_eq!(nb.on_activate("browser.reload_stop"), Some(BrowserAction::Stop));
    }

    #[test]
    fn build_toolbar_has_items() {
        let nb = NavigationButtons::new(true, true, false);
        let tb = nb.build_toolbar();
        assert!(tb.items.len() >= 3);
    }
}
