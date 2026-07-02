//! BrowserMenu — 浏览器主菜单 / 上下文菜单（spec §8.4.1A）。
//!
//! 组合通用 [`Menu`] / [`ContextMenu`]（布局、键盘导航、绘制由通用控件负责）；
//! 菜单条目由 command model 生成，激活时映射到 [`BrowserAction`]。

use crate::browser_action::BrowserAction;
use zero_ui_core::geometry::Point;
use zero_ui_widgets::menu::{ContextMenu, Menu, MenuItem};

/// 菜单条目（项或分隔符）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuEntry {
    Item {
        label: String,
        action: BrowserAction,
        enabled: bool,
    },
    Separator,
}

impl MenuEntry {
    pub fn item(label: &str, action: BrowserAction) -> MenuEntry {
        MenuEntry::Item {
            label: label.to_string(),
            action,
            enabled: true,
        }
    }

    pub fn separator() -> MenuEntry {
        MenuEntry::Separator
    }

    pub fn disabled(label: &str, action: BrowserAction) -> MenuEntry {
        MenuEntry::Item {
            label: label.to_string(),
            action,
            enabled: false,
        }
    }
}

/// 浏览器菜单（props）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BrowserMenu {
    pub entries: Vec<MenuEntry>,
}

impl BrowserMenu {
    pub fn new(entries: Vec<MenuEntry>) -> BrowserMenu {
        BrowserMenu { entries }
    }

    /// 组合通用 Menu（条目按序；item 的 action id 用稳定 index 编码，便于 on_activate 回映）。
    pub fn build_menu(&self) -> Menu {
        let mut m = Menu::new();
        for (i, e) in self.entries.iter().enumerate() {
            match e {
                MenuEntry::Separator => {
                    m.push(MenuItem::separator());
                }
                MenuEntry::Item { label, enabled, .. } => {
                    let mut item = MenuItem::item(label, &format!("browser_menu.{i}"));
                    item.enabled = *enabled;
                    m.push(item);
                }
            }
        }
        m
    }

    /// 上下文菜单（页面右键等）：Menu + 锚点。
    pub fn build_context_menu(&self, anchor: Point) -> ContextMenu {
        ContextMenu::new(anchor, self.build_menu())
    }

    /// 激活第 `idx` 个条目（与 entries 同序）→ 对应 BrowserAction；
    /// 分隔符 / 禁用 / 越界 → None。
    pub fn on_activate(&self, idx: usize) -> Option<&BrowserAction> {
        match self.entries.get(idx)? {
            MenuEntry::Item {
                action, enabled: true, ..
            } => Some(action),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BrowserMenu {
        BrowserMenu::new(vec![
            MenuEntry::item("New Tab", BrowserAction::OpenTab),
            MenuEntry::separator(),
            MenuEntry::disabled("Reload", BrowserAction::Reload),
            MenuEntry::item("Close", BrowserAction::CloseTab(zero_browser_shell::TabId(7))),
        ])
    }

    #[test]
    fn build_menu_preserves_order_and_separator() {
        let m = sample().build_menu();
        assert_eq!(m.items.len(), 4);
        assert!(m.items[1].action.is_none(), "分隔符无 action");
        assert!(!m.items[2].enabled, "disabled 条目保留 enabled=false");
    }

    #[test]
    fn context_menu_carries_anchor() {
        let cm = sample().build_context_menu(Point::new(5.0, 6.0));
        assert_eq!(cm.anchor, Point::new(5.0, 6.0));
        assert!(!cm.open);
        assert_eq!(cm.menu.items.len(), 4);
    }

    #[test]
    fn on_activate_maps_enabled_only() {
        let menu = sample();
        assert_eq!(menu.on_activate(0), Some(&BrowserAction::OpenTab));
        assert!(menu.on_activate(1).is_none(), "分隔符");
        assert!(menu.on_activate(2).is_none(), "disabled");
        assert_eq!(
            menu.on_activate(3),
            Some(&BrowserAction::CloseTab(zero_browser_shell::TabId(7)))
        );
        assert!(menu.on_activate(99).is_none(), "越界");
    }
}
