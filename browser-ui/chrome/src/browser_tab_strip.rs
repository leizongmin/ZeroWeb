//! BrowserTabStrip — 标签栏（spec §8.4.1A）。
//!
//! 组合通用 [`TabBar`]（+ favicon/loading 图标由 shell 在 widget 层叠加）；
//! browser-shell 提供 tab list，组件输出 activate/close/new/reorder action。

use crate::browser_action::BrowserAction;
use zero_browser_shell::TabId;
use zero_ui_patterns::tab_bar::TabBar;
use zero_ui_widgets::tabs::TabItem;

/// 单个浏览器标签状态（props，从 browser-shell tab model 投影）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserTab {
    pub id: TabId,
    pub title: String,
    pub loading: bool,
}

/// 标签栏（props）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrowserTabStrip {
    pub tabs: Vec<BrowserTab>,
    pub active_index: Option<usize>,
}

impl BrowserTabStrip {
    pub fn new(tabs: Vec<BrowserTab>) -> BrowserTabStrip {
        BrowserTabStrip {
            tabs,
            active_index: None,
        }
    }

    pub fn with_active(mut self, idx: usize) -> BrowserTabStrip {
        self.active_index = Some(idx);
        self
    }

    /// 组合通用 TabBar：每个 tab 一个 TabItem（label = 标题；id 稳定）。
    pub fn build_tab_bar(&self) -> TabBar {
        let items: Vec<TabItem> = self
            .tabs
            .iter()
            .map(|t| TabItem::new(&t.id.0.to_string(), &t.title))
            .collect();
        let mut bar = TabBar::new(items);
        if let Some(idx) = self.active_index {
            bar.tabs.activate(idx);
        }
        bar
    }

    /// 激活第 `idx` 个标签 → ActivateTab。
    pub fn on_activate(&self, idx: usize) -> Option<BrowserAction> {
        self.tabs.get(idx).map(|t| BrowserAction::ActivateTab(t.id))
    }

    /// 关闭第 `idx` 个标签 → CloseTab。
    pub fn on_close(&self, idx: usize) -> Option<BrowserAction> {
        self.tabs.get(idx).map(|t| BrowserAction::CloseTab(t.id))
    }

    /// "新建标签"按钮 → OpenTab。
    pub fn on_new_tab(&self) -> BrowserAction {
        BrowserAction::OpenTab
    }

    /// 拖拽重排 → ReorderTab。
    pub fn on_reorder(&self, from: usize, to: usize) -> Option<BrowserAction> {
        if from < self.tabs.len() && to < self.tabs.len() && from != to {
            Some(BrowserAction::ReorderTab { from, to })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BrowserTabStrip {
        BrowserTabStrip::new(vec![
            BrowserTab {
                id: TabId(1),
                title: "A".into(),
                loading: false,
            },
            BrowserTab {
                id: TabId(2),
                title: "B".into(),
                loading: true,
            },
        ])
        .with_active(0)
    }

    #[test]
    fn tab_bar_has_item_per_tab_and_activates() {
        let strip = sample();
        let bar = strip.build_tab_bar();
        assert_eq!(bar.tabs.items.len(), 2);
        assert_eq!(bar.tabs.active_index, Some(0));
    }

    #[test]
    fn actions_map_to_correct_tab() {
        let strip = sample();
        assert_eq!(strip.on_activate(1), Some(BrowserAction::ActivateTab(TabId(2))));
        assert_eq!(strip.on_close(0), Some(BrowserAction::CloseTab(TabId(1))));
        assert_eq!(strip.on_new_tab(), BrowserAction::OpenTab);
    }

    #[test]
    fn out_of_range_and_noop_reorder_is_none() {
        let strip = sample();
        assert!(strip.on_activate(5).is_none());
        assert!(strip.on_close(5).is_none());
        assert!(strip.on_reorder(0, 0).is_none(), "同位重排为 no-op");
        assert_eq!(
            strip.on_reorder(0, 1),
            Some(BrowserAction::ReorderTab { from: 0, to: 1 })
        );
    }
}
