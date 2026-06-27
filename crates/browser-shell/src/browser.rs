//! 浏览器 Shell — 协调标签页、书签、历史、下载、设置的顶层控制器。

use crate::autocomplete::Autocomplete;
use crate::bookmarks::Bookmarks;
use crate::download::DownloadManager;
use crate::history::History;
use crate::session::{NavigationSnapshot, SessionState, TabInfo};
use crate::settings::BrowserSettings;
use crate::tab::{TabId, TabManager};

/// 浏览器 Shell — 顶层协调器。
///
/// 管理标签页、书签、历史记录、下载和设置，提供浏览器级别的操作接口。
pub struct BrowserShell {
    /// 标签页管理器。
    tabs: TabManager,
    /// 书签管理器。
    bookmarks: Bookmarks,
    /// 历史记录管理器。
    history: History,
    /// 下载管理器。
    downloads: DownloadManager,
    /// 浏览器设置。
    settings: BrowserSettings,
    /// 当前页面缩放级别（1.0 = 100%）。
    zoom: f32,
    /// 页面查找状态。
    find_state: FindState,
    /// 地址栏自动补全引擎。
    autocomplete: Autocomplete,
}

/// 页面查找状态。
#[derive(Debug, Clone)]
pub struct FindState {
    /// 查找关键词。
    query: String,
    /// 是否正在查找。
    active: bool,
    /// 当前匹配索引（1-based）。
    current_match: usize,
    /// 总匹配数。
    total_matches: usize,
}

impl FindState {
    /// 创建空的查找状态。
    pub fn new() -> Self {
        Self {
            query: String::new(),
            active: false,
            current_match: 0,
            total_matches: 0,
        }
    }

    /// 获取查找关键词。
    pub fn query(&self) -> &str {
        &self.query
    }

    /// 是否正在查找。
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// 当前匹配索引。
    pub fn current_match(&self) -> usize {
        self.current_match
    }

    /// 总匹配数。
    pub fn total_matches(&self) -> usize {
        self.total_matches
    }
}

impl Default for FindState {
    fn default() -> Self {
        Self::new()
    }
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
            downloads: DownloadManager::new(),
            settings: BrowserSettings::new(),
            zoom: 1.0,
            find_state: FindState::new(),
            autocomplete: Autocomplete::new(),
        }
    }

    /// 创建浏览器 Shell 并从默认路径加载持久化设置。
    ///
    /// 如果设置文件不存在，使用默认设置。
    pub fn new_with_persisted_settings() -> Self {
        let mut shell = Self::new();
        shell.settings = BrowserSettings::load_default();
        shell.bookmarks = Bookmarks::load_default();
        shell.zoom = shell.settings.default_zoom;
        shell
    }

    /// 将当前设置保存到默认路径。
    pub fn save_settings(&self) -> Result<(), String> {
        self.settings.save_default()
    }

    /// 将当前书签保存到默认路径。
    pub fn save_bookmarks(&self) -> Result<(), String> {
        self.bookmarks.save_default()
    }

    /// 更新设置并立即写入默认配置文件。
    pub fn apply_settings<F>(&mut self, update: F)
    where
        F: FnOnce(&mut BrowserSettings),
    {
        update(&mut self.settings);
        if let Err(err) = self.save_settings() {
            tracing::warn!(%err, "failed to save settings");
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

    /// 创建无痕标签页并设为活跃。
    pub fn new_private_tab(&mut self, url: Option<&str>) -> TabId {
        self.tabs.create_private_tab(url)
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

    /// 遍历所有标签页。
    pub fn tabs(&self) -> impl Iterator<Item = &crate::tab::Tab> {
        self.tabs.tabs()
    }

    /// 获取活跃标签页的可变引用。
    pub fn active_tab_mut(&mut self) -> Option<&mut crate::tab::Tab> {
        self.tabs.active_tab_mut()
    }

    /// 获取指定标签页的引用。
    pub fn tab(&self, id: TabId) -> Option<&crate::tab::Tab> {
        self.tabs.get_tab(id)
    }

    /// 设置标签页固定状态。
    pub fn set_tab_pinned(&mut self, id: TabId, pinned: bool) {
        if let Some(tab) = self.tabs.get_tab_mut(id) {
            tab.set_pinned(pinned);
        }
    }

    /// 设置标签页静音状态。
    pub fn set_tab_muted(&mut self, id: TabId, muted: bool) {
        if let Some(tab) = self.tabs.get_tab_mut(id) {
            tab.set_muted(muted);
        }
    }

    /// 设置标签页崩溃状态。
    pub fn set_tab_crashed(&mut self, id: TabId, crashed: bool) {
        if let Some(tab) = self.tabs.get_tab_mut(id) {
            tab.set_crashed(crashed);
        }
    }

    /// 设置标签页需要关注状态。
    pub fn set_tab_needs_attention(&mut self, id: TabId, needs_attention: bool) {
        if let Some(tab) = self.tabs.get_tab_mut(id) {
            tab.set_needs_attention(needs_attention);
        }
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
            if let Err(err) = self.bookmarks.save_default() {
                tracing::warn!(%err, "failed to save bookmarks");
            }
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

    // ── 下载操作 ──

    /// 下载管理器的引用。
    pub fn downloads(&self) -> &DownloadManager {
        &self.downloads
    }

    /// 下载管理器的可变引用。
    pub fn downloads_mut(&mut self) -> &mut DownloadManager {
        &mut self.downloads
    }

    // ── 设置操作 ──

    /// 浏览器设置的引用。
    pub fn settings(&self) -> &BrowserSettings {
        &self.settings
    }

    /// 浏览器设置的可变引用。
    pub fn settings_mut(&mut self) -> &mut BrowserSettings {
        &mut self.settings
    }

    // ── 缩放操作 ──

    /// 获取当前缩放级别。
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// 设置缩放级别。
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(0.25, 5.0);
    }

    /// 放大（增加 10%）。
    pub fn zoom_in(&mut self) {
        self.set_zoom(self.zoom + 0.1);
    }

    /// 缩小（减少 10%）。
    pub fn zoom_out(&mut self) {
        self.set_zoom(self.zoom - 0.1);
    }

    /// 重置缩放到 100%。
    pub fn zoom_reset(&mut self) {
        self.zoom = 1.0;
    }

    // ── 页面查找操作 ──

    /// 获取查找状态。
    pub fn find_state(&self) -> &FindState {
        &self.find_state
    }

    /// 开始页面查找。
    pub fn find_start(&mut self, query: &str) {
        self.find_state.query = query.to_string();
        self.find_state.active = true;
        self.find_state.current_match = 0;
        self.find_state.total_matches = 0;
    }

    /// 跳转到下一个匹配。
    pub fn find_next(&mut self) {
        if self.find_state.total_matches > 0 {
            self.find_state.current_match = (self.find_state.current_match % self.find_state.total_matches) + 1;
        }
    }

    /// 跳转到上一个匹配。
    pub fn find_previous(&mut self) {
        if self.find_state.total_matches > 0 {
            if self.find_state.current_match <= 1 {
                self.find_state.current_match = self.find_state.total_matches;
            } else {
                self.find_state.current_match -= 1;
            }
        }
    }

    /// 关闭页面查找。
    pub fn find_close(&mut self) {
        self.find_state.active = false;
        self.find_state.query.clear();
        self.find_state.current_match = 0;
        self.find_state.total_matches = 0;
    }

    /// 更新查找匹配数。
    pub fn find_set_matches(&mut self, total: usize) {
        self.find_state.total_matches = total;
        if total > 0 && self.find_state.current_match == 0 {
            self.find_state.current_match = 1;
        }
    }

    // ── 地址栏自动补全 ──

    /// 根据输入查询自动补全建议。
    ///
    /// 从历史记录和书签中搜索匹配的 URL 和标题。
    pub fn suggest(&self, query: &str) -> Vec<crate::autocomplete::Suggestion> {
        self.autocomplete.suggest(query, &self.history, &self.bookmarks)
    }

    // ── 会话持久化 ──

    /// 将当前标签页状态保存为会话快照。
    ///
    /// 保存所有打开标签页的 URL、标题和导航历史。
    pub fn save_session(&self) -> SessionState {
        use crate::tab::NavigationEntry;
        fn nav_to_snapshot(entry: &NavigationEntry) -> NavigationSnapshot {
            NavigationSnapshot {
                url: entry.url.clone(),
                title: entry.title.clone(),
            }
        }

        let active_id = self.tabs.active_tab_id();
        let mut tabs: Vec<TabInfo> = Vec::new();
        let mut active_tab_index = None;
        for tab in self.tabs.tabs().filter(|t| !t.is_private()) {
            if Some(tab.id()) == active_id {
                active_tab_index = Some(tabs.len());
            }
            tabs.push(TabInfo {
                url: tab.url().map(|s| s.to_string()),
                title: tab.title().map(|s| s.to_string()),
                history: tab.navigation_history().iter().map(nav_to_snapshot).collect(),
                history_index: tab.history_index(),
            });
        }
        SessionState::from_tabs(tabs.into_iter(), active_tab_index)
    }

    /// 将当前会话保存到指定路径。
    pub fn save_session_to(&self, path: &std::path::Path) -> Result<(), String> {
        self.save_session().save(path)
    }

    /// 将当前会话保存到默认路径。
    pub fn save_session_default(&self) -> Result<(), String> {
        self.save_session().save_default()
    }

    /// 从会话快照恢复标签页。
    ///
    /// 清除当前所有标签页，根据快照重新创建。
    /// 返回恢复的标签页数量。
    pub fn restore_session(&mut self, session: &SessionState) -> usize {
        // 清除现有标签页
        // 由于 TabManager 没有 clear，直接创建新的
        self.tabs = TabManager::new();

        let count = session.tabs.len();
        for tab_snap in &session.tabs {
            // 创建标签页：如果有 URL 则以该 URL 创建，否则创建空白
            let new_id = if let Some(url) = &tab_snap.url {
                self.tabs.create_tab(Some(url))
            } else {
                self.tabs.create_tab(None)
            };

            // 恢复标题
            if let Some(title) = &tab_snap.title
                && let Some(tab) = self.tabs.get_tab_mut(new_id)
            {
                tab.set_title(title);
            }

            // 恢复导航历史（如果快照中有多条记录）
            // 目前 Tab::new(url) 已经创建了一条历史记录，
            // 如果快照中有多条历史记录，需要用完整的历史替换
            if tab_snap.history.len() > 1
                && let Some(tab) = self.tabs.get_tab_mut(new_id)
            {
                // 清除当前历史，重建
                tab.clear_history();
                for (i, nav) in tab_snap.history.iter().enumerate() {
                    tab.push_navigation(&nav.url, nav.title.as_deref());
                    if i == tab_snap.history_index {
                        tab.set_url_internal(&nav.url);
                        if let Some(t) = &nav.title {
                            tab.set_title(t);
                        }
                    }
                }
            }
        }

        // 恢复活跃标签页
        if let Some(active_idx) = session.active_tab_index {
            // 先收集 ID，再切换，避免借用冲突
            let active_id = self.tabs.tabs().nth(active_idx).map(|t| t.id());
            if let Some(id) = active_id {
                self.tabs.switch_to(id);
            }
        }

        count
    }

    /// 从指定路径加载会话并恢复。
    ///
    /// 如果文件不存在或解析失败，返回 `None`。
    pub fn restore_session_from(&mut self, path: &std::path::Path) -> Option<usize> {
        let session = SessionState::load(path)?;
        Some(self.restore_session(&session))
    }

    /// 从默认路径加载会话并恢复。
    pub fn restore_session_default(&mut self) -> Option<usize> {
        let session = SessionState::load_default()?;
        Some(self.restore_session(&session))
    }
}

impl Default for BrowserShell {
    fn default() -> Self {
        Self::new()
    }
}
