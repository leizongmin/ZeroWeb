//! BrowserChromeModel — 浏览器 chrome 业务模型投影（spec §8.4.4A / IF-009 / DC-12）。
//!
//! desktop/tablet/phone shell **共享**同一业务模型（不是同一视觉布局）：
//! 从 `zero-browser-shell` 状态投影为领域组件 props，shell 据此组装各自的 UI。
//! shell 只读消费本模型，不持有可变业务状态（spec FR-003 单向数据流）。

use crate::bookmarks_bar::BookmarkNode;
use crate::browser_tab_strip::BrowserTab;
use crate::download_panel::DownloadItemView;
use crate::find_bar::FindBar;
use crate::navigation_buttons::NavigationButtons;
use crate::page_load_indicator::PageLoadIndicator;
use crate::security_badge::SecurityState;
use crate::site_info_panel::SitePermission;

/// 浏览器 chrome 业务模型（desktop/tablet/phone 共享，spec §8.4.4A）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BrowserChromeModel {
    /// 标签列表（投影自 browser-shell tab model）。
    pub tabs: Vec<BrowserTab>,
    pub active_tab_index: Option<usize>,
    /// 导航状态（前进/后退/加载）。
    pub navigation: NavigationButtons,
    /// 地址栏文本（当前 active tab 的 URL/搜索词）。
    pub address_text: String,
    /// 当前站点安全状态。
    pub security: SecurityState,
    /// 书签栏根节点。
    pub bookmarks: Vec<BookmarkNode>,
    /// 下载项。
    pub downloads: Vec<DownloadItemView>,
    /// 站点权限状态（SiteInfoPanel 展示/切换）。
    pub permissions: Vec<SitePermission>,
    /// 页面查找会话（None = 未打开）。
    pub find: Option<FindBar>,
    /// 页面加载进度。
    pub page_load: PageLoadIndicator,
}

impl BrowserChromeModel {
    pub fn new() -> BrowserChromeModel {
        BrowserChromeModel::default()
    }

    /// active tab 的稳定 id（兼容 M1 skeleton 字段语义）。
    pub fn active_tab_id(&self) -> Option<u64> {
        self.active_tab_index.and_then(|i| self.tabs.get(i)).map(|t| t.id.0)
    }

    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_browser_shell::TabId;

    #[test]
    fn default_is_empty() {
        let m = BrowserChromeModel::new();
        assert_eq!(m.tab_count(), 0);
        assert!(m.active_tab_id().is_none());
        assert!(m.find.is_none());
    }

    #[test]
    fn active_tab_id_resolves() {
        let mut m = BrowserChromeModel::new();
        m.tabs = vec![
            BrowserTab {
                id: TabId(11),
                title: "A".into(),
                loading: false,
            },
            BrowserTab {
                id: TabId(22),
                title: "B".into(),
                loading: false,
            },
        ];
        m.active_tab_index = Some(1);
        assert_eq!(m.active_tab_id(), Some(22));
        assert_eq!(m.tab_count(), 2);
    }

    #[test]
    fn active_index_out_of_range_is_none() {
        let mut m = BrowserChromeModel::new();
        m.tabs = vec![BrowserTab {
            id: TabId(1),
            title: "A".into(),
            loading: false,
        }];
        m.active_tab_index = Some(9);
        assert!(m.active_tab_id().is_none());
    }
}
