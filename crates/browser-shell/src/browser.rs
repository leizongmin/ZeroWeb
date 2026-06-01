//! 浏览器 Shell — 协调标签页、书签、历史的顶层控制器。

use crate::bookmarks::Bookmarks;
use crate::history::History;
use crate::tab::{TabId, TabManager};

/// 浏览器 Shell — 顶层协调器。
///
/// 管理标签页、书签、历史记录，提供浏览器级别的操作接口。
pub struct BrowserShell {
    /// 标签页管理器。
    tabs: TabManager,
    /// 书签管理器。
    bookmarks: Bookmarks,
    /// 历史记录管理器。
    history: History,
}

impl BrowserShell {
    /// 创建新的浏览器 Shell。
    ///
    /// 默认创建一个空白标签页。
    pub fn new() -> Self {
        let mut tabs = TabManager::new();
        tabs.create_tab(None);

        Self {
            tabs,
            bookmarks: Bookmarks::new(),
            history: History::new(),
        }
    }

    /// 是否没有标签页。
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// 标签页数量。
    pub fn tab_count(&self) -> usize {
        self.tabs.len()
    }

    // ── 标签页操作 ──

    /// 创建新标签页并设为活跃。
    ///
    /// 返回新标签页的 ID。
    pub fn new_tab(&mut self, url: Option<&str>) -> TabId {
        self.tabs.create_tab(url)
    }

    /// 关闭指定标签页。
    pub fn close_tab(&mut self, id: TabId) {
        self.tabs.close_tab(id);
    }

    /// 切换到指定标签页。
    pub fn switch_tab(&mut self, id: TabId) {
        self.tabs.switch_to(id);
    }

    /// 获取活跃标签页 ID。
    pub fn active_tab_id(&self) -> Option<TabId> {
        self.tabs.active_tab_id()
    }

    /// 获取活跃标签页的引用。
    pub fn active_tab(&self) -> Option<&crate::tab::Tab> {
        self.tabs.active_tab()
    }

    /// 获取活跃标签页的可变引用。
    pub fn active_tab_mut(&mut self) -> Option<&mut crate::tab::Tab> {
        self.tabs.active_tab_mut()
    }

    /// 在活跃标签页中导航到新 URL。
    pub fn navigate(&mut self, url: &str) {
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.navigate(url);
        }
    }

    /// 刷新当前页面。
    pub fn refresh(&mut self) {
        if let Some(tab) = self.tabs.active_tab_mut()
            && tab.url().is_some()
        {
            tab.set_loading(true);
        }
    }

    /// 页面加载完成回调。
    ///
    /// 更新标签页状态并记录到历史。
    pub fn on_page_loaded(&mut self, title: &str) {
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.set_loading(false);
            tab.set_title(title);

            if let Some(url) = tab.url().map(|s| s.to_string()) {
                self.history.record(&url, title);
            }
        }
    }

    /// 页面加载失败回调。
    pub fn on_page_error(&mut self, _error: &str) {
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.set_loading(false);
        }
    }

    /// 后退。
    ///
    /// 返回 `true` 表示成功。
    pub fn go_back(&mut self) -> bool {
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.go_back()
        } else {
            false
        }
    }

    /// 前进。
    ///
    /// 返回 `true` 表示成功。
    pub fn go_forward(&mut self) -> bool {
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.go_forward()
        } else {
            false
        }
    }

    // ── 书签操作 ──

    /// 将当前页面添加到书签。
    pub fn add_bookmark(&mut self) {
        if let Some(tab) = self.tabs.active_tab()
            && let Some(url) = tab.url()
        {
            let title = tab.title().unwrap_or(url);
            self.bookmarks.add(title, url, None);
        }
    }

    /// 书签管理器的引用。
    pub fn bookmarks(&self) -> &Bookmarks {
        &self.bookmarks
    }

    /// 书签管理器的可变引用。
    pub fn bookmarks_mut(&mut self) -> &mut Bookmarks {
        &mut self.bookmarks
    }

    // ── 历史操作 ──

    /// 历史记录管理器的引用。
    pub fn history(&self) -> &History {
        &self.history
    }

    /// 历史记录管理器的可变引用。
    pub fn history_mut(&mut self) -> &mut History {
        &mut self.history
    }
}

impl Default for BrowserShell {
    fn default() -> Self {
        Self::new()
    }
}
