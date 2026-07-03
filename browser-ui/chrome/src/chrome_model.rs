//! BrowserChromeModel — 浏览器 chrome 业务模型投影（spec §8.4.4A / IF-009 / DC-12）。
//!
//! desktop/tablet/phone shell **共享**同一业务模型（不是同一视觉布局）：
//! 从 `zero-browser-shell` 状态投影为领域组件 props，shell 据此组装各自的 UI。
//! shell 只读消费本模型，不持有可变业务状态（spec FR-003 单向数据流）。

use crate::bookmarks_bar::BookmarkNode;
use crate::browser_tab_strip::BrowserTab;
use crate::download_panel::{DownloadItemView, DownloadState};
use crate::find_bar::FindBar;
use crate::navigation_buttons::NavigationButtons;
use crate::page_load_indicator::PageLoadIndicator;
use crate::security_badge::SecurityState;
use crate::site_info_panel::SitePermission;
use zero_browser_shell::{BrowserShell, DownloadEntry};

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

    /// 从 [`BrowserShell`] 状态投影出 chrome 业务模型（spec §8.4.4A / DC-12 / DC-14 迁移 seam）。
    ///
    /// 纯只读映射、无副作用、不触渲染：apps/browser 据此把 `zero-browser-shell` 业务状态
    /// 喂给 desktop/tablet/phone shell（shell 只读消费，spec FR-003 单向数据流）。
    /// 这是「浏览器迁移为 SDK 宿主」的首个数据层接入点——后续逐组件 chrome 灰度迁移
    /// 均从本模型取 props。
    ///
    /// **映射约定**（受限于两侧模型差异）：
    /// - security：browser-shell 不追踪页面安全状态，按 active tab URL scheme 派生
    ///   （`https://`→Secure、`http://`→Insecure、其余→Secure）。
    /// - downloads 状态：shell 6 态 → chrome 3 态
    ///   （Pending/Downloading/Paused→InProgress、Completed→Completed、Cancelled/Failed→Cancelled）。
    /// - permissions：browser-shell 当前不追踪 per-site 权限 → 空（DC-13 platform 域后续接入）。
    pub fn from_shell(shell: &BrowserShell) -> BrowserChromeModel {
        let mut model = BrowserChromeModel::new();

        // tabs + active index（shell 用 TabId，模型用 index）。
        let active_id = shell.active_tab_id();
        for (i, t) in shell.tabs().enumerate() {
            if active_id == Some(t.id()) {
                model.active_tab_index = Some(i);
            }
            model.tabs.push(BrowserTab {
                id: t.id(),
                title: t.title().unwrap_or("").to_string(),
                loading: t.is_loading(),
            });
        }

        // navigation + address + security + page_load（来自 active tab）。
        if let Some(at) = shell.active_tab() {
            let hist_len = at.history_len();
            let hist_idx = at.history_index();
            model.navigation = NavigationButtons {
                can_go_back: hist_idx > 0,
                can_go_forward: hist_idx + 1 < hist_len,
                loading: at.is_loading(),
            };
            model.address_text = at.url().unwrap_or("").to_string();
            model.security = security_from_url(at.url());
            model.page_load = PageLoadIndicator {
                loading: at.is_loading(),
                fraction: None,
            };
        }

        // bookmarks（list_root 仅返回书签项；文件夹由 bookmarks.folders 管理，当前未投影）。
        model.bookmarks = shell
            .bookmarks()
            .list_root()
            .into_iter()
            .map(|b| BookmarkNode {
                id: b.id().0.to_string(),
                title: b.title().to_string(),
                url: Some(b.url().to_string()),
            })
            .collect();

        // downloads（状态映射见上方约定）。
        model.downloads = shell.downloads().iter().map(map_download).collect();

        // find（仅活跃时投影；match_index 从 1-based 转 0-based）。
        let f = shell.find_state();
        if f.is_active() {
            let total = f.total_matches();
            model.find = Some(FindBar {
                query: f.query().to_string(),
                match_index: f.current_match().checked_sub(1).map(|x| x as u32),
                match_count: if total > 0 { Some(total as u32) } else { None },
                open: true,
            });
        }

        model
    }
}

/// 按 URL scheme 派生安全状态（browser-shell 不追踪页面安全，DC-14 seam 暂用此启发式）。
fn security_from_url(url: Option<&str>) -> SecurityState {
    match url {
        Some(u) if u.starts_with("http://") => SecurityState::Insecure,
        _ => SecurityState::Secure,
    }
}

/// 把 shell 下载项投影为 chrome 下载视图（状态映射：6→3）。
fn map_download(d: &DownloadEntry) -> DownloadItemView {
    use zero_browser_shell::DownloadState as S;
    let state = match d.state() {
        S::Completed => DownloadState::Completed,
        S::Cancelled | S::Failed => DownloadState::Cancelled,
        S::Pending | S::Downloading | S::Paused => DownloadState::InProgress,
    };
    DownloadItemView {
        id: d.id().0.to_string(),
        filename: d.filename().to_string(),
        received_bytes: d.downloaded_bytes(),
        total_bytes: d.total_bytes(),
        state,
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

    // ── from_shell 投影（DC-14 迁移 seam）──────────────────────────────

    #[test]
    fn from_shell_default_has_initial_tab() {
        // BrowserShell::new() 预建 1 个空标签（active），bookmarks/downloads 为空。
        let shell = BrowserShell::new();
        let m = BrowserChromeModel::from_shell(&shell);
        assert_eq!(m.tab_count(), 1);
        assert_eq!(m.active_tab_index, Some(0));
        assert!(m.find.is_none());
        assert!(m.bookmarks.is_empty());
        assert!(m.downloads.is_empty());
        assert_eq!(m.security, SecurityState::Secure); // 空 tab url=None → 默认 Secure
        assert_eq!(m.address_text, "");
    }

    #[test]
    fn from_shell_tabs_and_active_index() {
        let mut shell = BrowserShell::new();
        let _initial = shell.active_tab_id(); // new() 预建的空标签（index 0）
        let _a = shell.new_tab(Some("https://example.com")); // index 1
        let b = shell.new_tab(Some("https://github.com")); // index 2，new_tab 激活 → active=2
        let m = BrowserChromeModel::from_shell(&shell);
        assert_eq!(m.tab_count(), 3);
        assert_eq!(m.active_tab_index, Some(2));
        assert_eq!(m.tabs[1].id, _a);
        assert_eq!(m.tabs[2].id, b);
        // active tab 地址 + 安全（https → Secure）。
        assert_eq!(m.address_text, "https://github.com");
        assert_eq!(m.security, SecurityState::Secure);
        // 初始历史仅 1 条 → 不可后退/前进。
        assert!(!m.navigation.can_go_back);
        assert!(!m.navigation.can_go_forward);
    }

    #[test]
    fn from_shell_navigation_back_forward() {
        let mut shell = BrowserShell::new();
        shell.new_tab(Some("https://a.example.com")); // active，历史 [a]
        shell.navigate("https://b.example.com"); // 压入第二条历史 [a,b]，index=1
        shell.go_back(); // 回到 a，index=0
        let m = BrowserChromeModel::from_shell(&shell);
        // 回到历史首条 → 可前进、不可后退。
        assert!(!m.navigation.can_go_back);
        assert!(m.navigation.can_go_forward);
    }

    #[test]
    fn from_shell_http_is_insecure() {
        let mut shell = BrowserShell::new();
        shell.new_tab(Some("http://insecure.example.com")); // active
        let m = BrowserChromeModel::from_shell(&shell);
        assert_eq!(m.security, SecurityState::Insecure);
    }

    #[test]
    fn from_shell_bookmarks_and_downloads() {
        let mut shell = BrowserShell::new();
        shell.bookmarks_mut().add("Example", "https://example.com", None);
        shell
            .downloads_mut()
            .start_download("https://example.com/file.zip", "file.zip");
        let m = BrowserChromeModel::from_shell(&shell);
        assert_eq!(m.bookmarks.len(), 1);
        assert_eq!(m.bookmarks[0].title, "Example");
        assert_eq!(m.bookmarks[0].url.as_deref(), Some("https://example.com"));
        assert_eq!(m.downloads.len(), 1);
        assert_eq!(m.downloads[0].filename, "file.zip");
        // start_download → shell Downloading → chrome InProgress。
        assert_eq!(m.downloads[0].state, DownloadState::InProgress);
    }

    #[test]
    fn from_shell_find_active_projects() {
        let mut shell = BrowserShell::new();
        shell.find_start("hello");
        shell.find_set_matches(3); // 总匹配 3
        let m = BrowserChromeModel::from_shell(&shell);
        let f = m.find.expect("find active → projected");
        assert_eq!(f.query, "hello");
        assert_eq!(f.match_count, Some(3));
        assert!(f.open);
    }

    #[test]
    fn from_shell_find_inactive_is_none() {
        let shell = BrowserShell::new();
        // 未 find_start → is_active=false → model.find None。
        let m = BrowserChromeModel::from_shell(&shell);
        assert!(m.find.is_none());
    }
}
