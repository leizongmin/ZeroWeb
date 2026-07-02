//! # zero-browser-chrome
//!
//! 浏览器专属 chrome 组件（spec §8.4.1 `zero-browser-chrome` / FR-009 / §8.4.1A）。
//!
//! 由通用 `ui/widgets` + `ui/patterns` 组合绘制，输出进入统一 UI SDK scene（不绕过 ui/render）。
//! 这是 UI SDK **唯一**允许依赖 `zero-browser-shell` 与 `ui/adapters/webview` 的浏览器耦合点
//! （spec 约束 3 / DC-1）。
//!
//! M1 skeleton：`BrowserAction` 合约、`NavigationButtons`、`PageViewportFrame`；
//! 其余 §8.4.1A 组件（AddressBar/BrowserTabStrip/SecurityBadge/SiteInfoPanel/BookmarksBar/
//! FindBar/PermissionPrompt/DownloadPanel/BrowserMenu/PageLoadIndicator）在 M2 随迁移落地。

pub mod browser_action;
pub mod navigation_buttons;
pub mod page_viewport;

pub use browser_action::BrowserAction;
pub use navigation_buttons::NavigationButtons;
pub use page_viewport::PageViewportFrame;

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
