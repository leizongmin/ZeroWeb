//! # zero-browser-chrome
//!
//! 浏览器专属 chrome 组件（spec §8.4.1 `zero-browser-chrome` / FR-009 / §8.4.1A）。
//!
//! 由通用 `ui/widgets` + `ui/patterns` 组合绘制，输出进入统一 UI SDK scene（不绕过 ui/render）。
//! 这是 UI SDK **唯一**允许依赖 `zero-browser-shell` 与 `ui/adapters/webview` 的浏览器耦合点
//! （spec 约束 3 / DC-1）。
//!
//! M2 已落地 §8.4.1A 全部 12 组件 + desktop/tablet/phone adaptive shell（DC-12）：
//! [`BrowserChromeModel`]（共享业务模型）+ [`AdaptiveBrowserChrome`]（按 metrics 选 shell）。

pub mod actions;
pub mod address_bar;
pub mod bookmarks_bar;
pub mod browser_action;
pub mod browser_menu;
pub mod browser_tab_strip;
pub mod chrome_model;
pub mod download_panel;
pub mod find_bar;
pub mod i18n;
pub mod navigation_buttons;
pub mod page_load_indicator;
pub mod page_viewport;
pub mod permission_prompt;
pub mod phone_demo;
pub mod render;
pub mod sdk_render;
pub mod security_badge;
pub mod shell;
pub mod shell_demo;
pub mod site_info_panel;

pub use address_bar::{AddressBar, AddressSubmission};
pub use bookmarks_bar::{BookmarkNode, BookmarksBar};
pub use browser_action::BrowserAction;
pub use browser_menu::{BrowserMenu, MenuEntry};
pub use browser_tab_strip::{BrowserTab, BrowserTabStrip};
pub use chrome_model::BrowserChromeModel;
pub use download_panel::{DownloadItemView, DownloadPanel, DownloadState};
pub use find_bar::FindBar;
pub use navigation_buttons::NavigationButtons;
pub use page_load_indicator::PageLoadIndicator;
pub use page_viewport::PageViewportFrame;
pub use permission_prompt::PermissionPrompt;
pub use render::{ChromePanel, chrome_color, register_chrome_factories, scene_texts, security_color_name};
pub use security_badge::{SecurityBadge, SecurityState};
pub use shell::{
    AdaptiveBrowserChrome, AdaptiveChromeResult, BrowserChromeShell, DesktopBrowserShell, PhoneBrowserShell, ShellKind,
    ShellLayout, TabletBrowserShell, select_shell,
};
pub use site_info_panel::{SiteInfoPanel, SitePermission};
