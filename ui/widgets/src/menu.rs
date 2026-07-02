//! Menu / ContextMenu — 菜单项模型（spec FR-009）。
//!
//! 菜单项触发 action（spec FR-003），由 command 层（`ui/commands`）统一注册与派发。

use compact_str::CompactString;
use zero_ui_core::action::ActionId;

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
}
