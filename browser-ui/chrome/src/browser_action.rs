//! BrowserAction — 浏览器 chrome 的统一 action 合约（spec §8.4.1A / M2 desktop·tablet·phone shell 共享）。
//!
//! chrome 组件只产出 `BrowserAction`；由 `apps/browser` reducer 映射到 `zero-browser-shell` 状态变更
//! 与 `BrowserCommandId`（spec §8.4.1B 同 command 多入口）。

use zero_browser_shell::TabId;

/// 浏览器 action（覆盖 §8.4.1A 全部 chrome 组件的输入语义）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserAction {
    // 导航
    GoBack,
    GoForward,
    Reload,
    Stop,
    Home,
    Navigate(String),
    Search(String),
    // 标签
    OpenTab,
    CloseTab(TabId),
    ActivateTab(TabId),
    ReorderTab {
        from: usize,
        to: usize,
    },
    // 书签/历史
    OpenBookmark(String),
    // 权限/下载
    GrantPermission(String),
    DenyPermission(String),
    /// 打开/运行已下载文件（download id）。
    OpenDownload(String),
    /// 取消进行中的下载（download id）。
    CancelDownload(String),
    /// 在文件夹中显示已下载文件（download id）。
    ShowDownload(String),
    // 查找
    FindNext,
    FindPrev,
    FindClose,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_variants_distinct() {
        assert_ne!(BrowserAction::GoBack, BrowserAction::GoForward);
        assert_eq!(BrowserAction::CloseTab(TabId(3)), BrowserAction::CloseTab(TabId(3)));
        assert_ne!(BrowserAction::CloseTab(TabId(3)), BrowserAction::CloseTab(TabId(4)));
    }
}
