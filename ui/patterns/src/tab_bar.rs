//! TabBar — 标签栏组合模式（通用；浏览器领域 BrowserTabStrip 在 `browser-ui/chrome`，spec FR-009）。
//!
//! 组合 `ui/widgets::Tabs` 的 tab item + 关闭/新增语义。

use zero_ui_widgets::tabs::{TabItem, Tabs};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabBar {
    pub tabs: Tabs,
    pub show_new_tab: bool,
}

impl TabBar {
    pub fn new(items: Vec<TabItem>) -> TabBar {
        TabBar {
            tabs: Tabs::new(items),
            show_new_tab: true,
        }
    }

    /// 返回当前激活 tab 的引用。
    pub fn active(&self) -> Option<&TabItem> {
        self.tabs.active_index.and_then(|i| self.tabs.items.get(i))
    }

    pub fn close_active(&mut self) {
        if let Some(idx) = self.tabs.active_index
            && idx < self.tabs.items.len()
        {
            self.tabs.items.remove(idx);
            self.tabs.active_index = if self.tabs.items.is_empty() {
                None
            } else {
                Some(idx.min(self.tabs.items.len() - 1))
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn close_active_adjusts_index() {
        let mut bar = TabBar::new(vec![
            TabItem::new("a", "A"),
            TabItem::new("b", "B"),
            TabItem::new("c", "C"),
        ]);
        bar.tabs.activate(1);
        bar.close_active();
        assert_eq!(bar.tabs.items.len(), 2);
        assert_eq!(bar.tabs.active_index, Some(1));
        assert_eq!(
            bar.active().map(|t| t.label.as_str().to_string()),
            Some("C".to_string())
        );
    }
}
