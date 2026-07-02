//! Menu / ContextMenu — 菜单项模型（spec FR-009）。
//!
//! 菜单项触发 action（spec FR-003），由 command 层（`ui/commands`）统一注册与派发。
//! [`ContextMenu`] 在 [`Menu`] 基础上加锚点位置 + 打开状态（页面右键菜单等）。

use compact_str::CompactString;
use zero_ui_core::action::ActionId;
use zero_ui_core::geometry::Point;

/// 菜单项（可带分隔语义）。
#[derive(Debug, Clone, PartialEq)]
pub struct MenuItem {
    pub label: CompactString,
    pub action: Option<ActionId>,
    pub enabled: bool,
}

impl MenuItem {
    pub fn item(label: &str, action: &str) -> MenuItem {
        MenuItem {
            label: CompactString::new(label),
            action: Some(ActionId::new(action)),
            enabled: true,
        }
    }
    pub fn separator() -> MenuItem {
        MenuItem {
            label: CompactString::new(""),
            action: None,
            enabled: false,
        }
    }
}

/// 菜单模型。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Menu {
    pub items: Vec<MenuItem>,
}

impl Menu {
    pub fn new() -> Menu {
        Menu::default()
    }
    pub fn push(&mut self, item: MenuItem) -> &mut Menu {
        self.items.push(item);
        self
    }
}

/// 上下文菜单：`Menu` + 锚点位置 + 打开状态（页面右键菜单、次级菜单等）。
#[derive(Debug, Clone, PartialEq)]
pub struct ContextMenu {
    pub menu: Menu,
    pub anchor: Point,
    pub open: bool,
}

impl ContextMenu {
    pub fn new(anchor: Point, menu: Menu) -> ContextMenu {
        ContextMenu {
            menu,
            anchor,
            open: false,
        }
    }

    pub fn open(&mut self) {
        self.open = true;
    }
    pub fn close(&mut self) {
        self.open = false;
    }
    pub fn toggle(&mut self) {
        self.open = !self.open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_menu_with_separator() {
        let mut m = Menu::new();
        m.push(MenuItem::item("New", "file.new"))
            .push(MenuItem::separator())
            .push(MenuItem::item("Exit", "app.exit"));
        assert_eq!(m.items.len(), 3);
        assert!(m.items[1].action.is_none()); // 分隔符无 action
    }

    #[test]
    fn context_menu_open_close_toggle() {
        let mut cm = ContextMenu::new(Point::new(10.0, 20.0), Menu::new());
        assert!(!cm.open);
        cm.toggle();
        assert!(cm.open);
        cm.close();
        assert!(!cm.open);
        cm.open();
        assert!(cm.open);
        assert_eq!(cm.anchor, Point::new(10.0, 20.0));
    }
}
