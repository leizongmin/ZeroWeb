//! Toolbar — 水平工具栏容器（spec FR-009）。
//!
//! 承载 IconButton/Button/分隔符；具体条目触发 action。

use compact_str::CompactString;
use zero_ui_core::action::ActionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolbarItem {
    pub id: CompactString,
    pub action: ActionId,
    pub icon: Option<CompactString>,
}

impl ToolbarItem {
    pub fn new(id: &str, action: &str) -> ToolbarItem {
        ToolbarItem {
            id: CompactString::new(id),
            action: ActionId::new(action),
            icon: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Toolbar {
    pub items: Vec<ToolbarItem>,
}

impl Toolbar {
    pub fn new() -> Toolbar {
        Toolbar::default()
    }
    pub fn push(&mut self, item: ToolbarItem) -> &mut Toolbar {
        self.items.push(item);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_toolbar() {
        let mut t = Toolbar::new();
        t.push(ToolbarItem::new("back", "browser.go_back"))
            .push(ToolbarItem::new("forward", "browser.go_forward"));
        assert_eq!(t.items.len(), 2);
    }
}
