//! # zero-browser-chrome
//!
//! 浏览器专属 chrome 组件（spec §8.4.1 `zero-browser-chrome` / FR-009 / §8.4.1A）。
//!
//! 由通用 `ui/widgets` + `ui/patterns` 组合绘制，输出进入统一 UI SDK scene（不绕过 ui/render）。
//! 这是 UI SDK **唯一**允许依赖 `zero-browser-shell` 与 `ui/adapters/webview` 的浏览器耦合点
//! （spec 约束 3 / DC-1）。
//!
//! M2 已落地 §8.4.1A 全部 12 组件：`NavigationButtons`、`PageViewportFrame`、`PageLoadIndicator`、
//! `SecurityBadge`、`BookmarksBar`、`BrowserTabStrip`、`FindBar`、`AddressBar`、`BrowserMenu`、
//! `PermissionPrompt`、`SiteInfoPanel`、`DownloadPanel`/`DownloadItemView`。

pub mod address_bar;
pub mod bookmarks_bar;
pub mod browser_action;
pub mod browser_menu;
pub mod browser_tab_strip;
pub mod download_panel;
pub mod find_bar;
pub mod navigation_buttons;
pub mod page_load_indicator;
pub mod page_viewport;
pub mod permission_prompt;
pub mod security_badge;
pub mod site_info_panel;

pub use address_bar::{AddressBar, AddressSubmission};
pub use bookmarks_bar::{BookmarkNode, BookmarksBar};
pub use browser_action::BrowserAction;
pub use browser_menu::{BrowserMenu, MenuEntry};
pub use browser_tab_strip::{BrowserTab, BrowserTabStrip};
pub use download_panel::{DownloadItemView, DownloadPanel, DownloadState};
pub use find_bar::FindBar;
pub use navigation_buttons::NavigationButtons;
pub use page_load_indicator::PageLoadIndicator;
pub use page_viewport::PageViewportFrame;
pub use permission_prompt::PermissionPrompt;
pub use security_badge::{SecurityBadge, SecurityState};
pub use site_info_panel::{SiteInfoPanel, SitePermission};

/// 浏览器 chrome 模型（M2 desktop/tablet/phone shell 共享合约的根，spec §8.4.1A）。
///
/// M1 只持有 active tab + adaptive 分支标记；真实状态从 `zero-browser-shell` 投影。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BrowserChromeModel {
    pub active_tab_id: Option<u64>,
    pub tab_count: usize,
}

impl BrowserChromeModel {
    pub fn new() -> BrowserChromeModel {
        BrowserChromeModel::default()
    }
}
