//! 浏览器 Shell — 协调标签页、书签、历史、下载、设置的顶层控制器。

use std::collections::VecDeque;
use std::path::Path;

use crate::autocomplete::Autocomplete;
use crate::bookmarks::Bookmarks;
use crate::download::DownloadManager;
use crate::history::History;
use crate::session::{NavigationSnapshot, SessionState, TabInfo};
use crate::settings::BrowserSettings;
use crate::tab::{TabId, TabManager};

/// 已关闭标签的快照，用于 Ctrl+Shift+T 恢复。
#[derive(Debug, Clone)]
pub struct ClosedTab {
    /// 关闭时的 URL（若有）。
    pub url: Option<String>,
    /// 关闭时的标题（若有）。
    pub title: Option<String>,
}

/// recently_closed 队列最大长度。
const MAX_RECENTLY_CLOSED: usize = 10;

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
    /// 最近关闭的标签队列（最新在尾部），用于 Ctrl+Shift+T 恢复。
    recently_closed: VecDeque<ClosedTab>,
}

/// 页面查找循环提示方向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindWrapHint {
    /// 已从末尾回到开头。
    WrappedToStart,
    /// 已从开头跳到末尾。
    WrappedToEnd,
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
    /// 上一次 find_next/previous 是否发生了循环环绕（用于 UI 提示）。
    last_wrap: Option<FindWrapHint>,
    /// 是否区分大小写。
    case_sensitive: bool,
    /// 是否全字匹配。
    whole_word: bool,
}

impl FindState {
    /// 创建空的查找状态。
    pub fn new() -> Self {
        Self {
            query: String::new(),
            active: false,
            current_match: 0,
            total_matches: 0,
            last_wrap: None,
            case_sensitive: false,
            whole_word: false,
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

    /// 上一次 find 操作的循环提示（若有）。
    pub fn last_wrap(&self) -> Option<FindWrapHint> {
        self.last_wrap
    }

    /// 是否区分大小写。
    pub fn case_sensitive(&self) -> bool {
        self.case_sensitive
    }

    /// 是否全字匹配。
    pub fn whole_word(&self) -> bool {
        self.whole_word
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
            recently_closed: VecDeque::new(),
        }
    }

    /// 创建浏览器 Shell 并从默认路径加载持久化设置。
    ///
    /// 如果设置文件不存在，使用默认设置。
    pub fn new_with_persisted_settings() -> Self {
        Self::new_with_persisted_settings_at(&BrowserSettings::default_config_path())
    }

    /// 创建浏览器 Shell 并从指定路径加载设置。
    pub(crate) fn new_with_persisted_settings_at(path: &Path) -> Self {
        let mut shell = Self::new();
        shell.settings = BrowserSettings::load(path);
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

    /// 创建后台标签页（不切换活跃），用于 Ctrl+点击链接等场景。
    pub fn new_tab_background(&mut self, url: Option<&str>) -> TabId {
        self.tabs.create_tab_background(url)
    }

    /// 创建无痕标签页并设为活跃。
    pub fn new_private_tab(&mut self, url: Option<&str>) -> TabId {
        self.tabs.create_private_tab(url)
    }

    /// 关闭指定标签页。
    pub fn close_tab(&mut self, id: TabId) {
        // 先收集快照，避免对 self 的共享/可变借用冲突。
        let snapshot = self.tabs.tabs().find(|t| t.id() == id).map(|tab| ClosedTab {
            url: tab.url().map(str::to_string),
            title: tab.title().map(str::to_string),
        });
        if let Some(c) = snapshot {
            self.push_recently_closed(c);
        }
        self.tabs.close_tab(id);
    }

    /// 复制指定标签页（副本插入其后并设为活跃）。
    pub fn duplicate_tab(&mut self, id: TabId) -> Option<TabId> {
        self.tabs.duplicate_tab(id)
    }

    /// 关闭除指定标签页外的所有标签页。
    pub fn close_other_tabs(&mut self, id: TabId) {
        // 先收集所有将被关闭的标签快照（保留指定 id 的那个）。
        let to_close: Vec<ClosedTab> = self
            .tabs
            .tabs()
            .filter(|t| t.id() != id)
            .map(|t| ClosedTab {
                url: t.url().map(str::to_string),
                title: t.title().map(str::to_string),
            })
            .collect();
        for c in to_close {
            self.push_recently_closed(c);
        }
        self.tabs.close_other_tabs(id);
    }

    /// 关闭指定标签页右侧的所有标签页。
    pub fn close_tabs_to_right(&mut self, id: TabId) {
        // 先收集要关闭的标签快照，避免 borrow 冲突。
        let to_close: Vec<ClosedTab> = {
            let mut iter = self.tabs.tabs().peekable();
            // 跳过到 id 之后。
            while let Some(t) = iter.peek() {
                if t.id() == id {
                    iter.next();
                    break;
                }
                iter.next();
            }
            iter.map(|t| ClosedTab {
                url: t.url().map(str::to_string),
                title: t.title().map(str::to_string),
            })
            .collect()
        };
        for c in to_close {
            self.push_recently_closed(c);
        }
        self.tabs.close_tabs_to_right(id);
    }

    /// 推入 recently_closed 队列，超出上限时丢弃最旧的一条。
    fn push_recently_closed(&mut self, tab: ClosedTab) {
        if self.recently_closed.len() >= MAX_RECENTLY_CLOSED {
            self.recently_closed.pop_front();
        }
        self.recently_closed.push_back(tab);
    }

    /// 恢复最近关闭的一个标签：弹出最新一条快照，新建标签并导航到原 URL。
    /// 返回新标签页 ID；若无历史返回 None。
    pub fn reopen_last_closed_tab(&mut self) -> Option<TabId> {
        let closed = self.recently_closed.pop_back()?;
        // create_tab 已将新标签设为活跃，直接用 self.navigate 导航。
        let new_id = self.tabs.create_tab(None);
        if let Some(url) = closed.url.as_deref() {
            self.navigate(url);
            if let (Some(title), Some(tab)) = (closed.title.as_deref(), self.tabs.active_tab_mut()) {
                tab.set_title(title);
            }
        }
        Some(new_id)
    }

    /// 最近关闭的标签快照（最新在尾部），只读访问，用于菜单展示。
    pub fn recently_closed(&self) -> impl Iterator<Item = &ClosedTab> {
        self.recently_closed.iter().rev()
    }

    /// 恢复指定 URL 对应的最近关闭标签（从队列中移除该条）。
    /// 用于从菜单列表中点击恢复特定标签。返回新标签 ID。
    pub fn reopen_closed_by_url(&mut self, url: &str) -> Option<TabId> {
        // 从尾部查找最新的匹配项。
        let pos = self
            .recently_closed
            .iter()
            .rposition(|c| c.url.as_deref() == Some(url))?;
        let closed = self.recently_closed.remove(pos)?;
        let new_id = self.tabs.create_tab(None);
        if let Some(url) = closed.url.as_deref() {
            self.navigate(url);
            if let (Some(title), Some(tab)) = (closed.title.as_deref(), self.tabs.active_tab_mut()) {
                tab.set_title(title);
            }
        }
        Some(new_id)
    }

    /// 切换到指定标签页。
    pub fn switch_tab(&mut self, id: TabId) {
        self.tabs.switch_to(id);
    }

    /// 移动标签到指定索引位置（拖拽重排序用）。
    pub fn move_tab(&mut self, id: TabId, to_index: usize) -> bool {
        self.tabs.move_tab(id, to_index)
    }

    /// 按索引切换到对应标签页（Ctrl+1~8 / Ctrl+9 用）。
    /// index 超出范围时无操作。返回是否成功切换。
    pub fn switch_to_index(&mut self, index: usize) -> bool {
        let id = self.tabs.tabs().nth(index).map(|t| t.id());
        if let Some(id) = id {
            self.tabs.switch_to(id);
            true
        } else {
            false
        }
    }

    /// 切换到最后一个标签页（Ctrl+9 用）。无标签时返回 false。
    pub fn switch_to_last(&mut self) -> bool {
        let id = self.tabs.tabs().last().map(|t| t.id());
        if let Some(id) = id {
            self.tabs.switch_to(id);
            true
        } else {
            false
        }
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

    /// 停止当前活动标签页的加载（Esc 或刷新按钮在 loading 时点击）。
    pub fn stop_loading(&mut self) {
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.set_loading(false);
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

    /// 用指定 URL 添加书签（用于"将链接添加为书签"）。
    pub fn add_bookmark_with_url(&mut self, url: &str) {
        self.bookmarks.add(url, url, None);
        if let Err(err) = self.bookmarks.save_default() {
            tracing::warn!(%err, "failed to save bookmarks");
        }
    }
    /// 切换当前页面书签状态：未收藏则添加，已收藏则移除。返回最终是否已收藏。
    pub fn toggle_current_bookmark(&mut self) -> bool {
        if let Some(tab) = self.tabs.active_tab()
            && let Some(url) = tab.url()
        {
            if self.bookmarks.find_by_url(url).is_some() {
                self.bookmarks.remove_by_url(url);
                if let Err(err) = self.bookmarks.save_default() {
                    tracing::warn!(%err, "failed to save bookmarks");
                }
                false
            } else {
                let title = tab.title().unwrap_or(url);
                self.bookmarks.add(title, url, None);
                if let Err(err) = self.bookmarks.save_default() {
                    tracing::warn!(%err, "failed to save bookmarks");
                }
                true
            }
        } else {
            false
        }
    }

    /// 当前活跃标签页的 URL 是否已收藏。
    pub fn is_current_page_bookmarked(&self) -> bool {
        self.tabs
            .active_tab()
            .and_then(|t| t.url())
            .is_some_and(|url| self.bookmarks.find_by_url(url).is_some())
    }

    /// 书签管理器的引用。
    pub fn bookmarks(&self) -> &Bookmarks {
        &self.bookmarks
    }

    /// 书签管理器的可变引用。
    pub fn bookmarks_mut(&mut self) -> &mut Bookmarks {
        &mut self.bookmarks
    }

    /// 按 URL 删除书签，并持久化。返回是否确实删除了一条记录。
    pub fn remove_bookmark_by_url(&mut self, url: &str) -> bool {
        let removed = self.bookmarks.remove_by_url(url);
        if removed && let Err(err) = self.bookmarks.save_default() {
            tracing::warn!(%err, "failed to save bookmarks");
        }
        removed
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
    // 缩放是每标签独立的：以下方法作用于当前活动标签页。
    // 新标签页创建时 zoom 初始化为 1.0；settings.default_zoom 作为设置页展示的默认值。

    /// 获取当前缩放级别（活动标签页的 zoom；无活动标签时返回全局默认）。
    pub fn zoom(&self) -> f32 {
        self.tabs.active_tab().map(|t| t.zoom()).unwrap_or(self.zoom)
    }

    /// 设置当前缩放级别（作用于活动标签页）。
    pub fn set_zoom(&mut self, zoom: f32) {
        let clamped = zoom.clamp(0.25, 5.0);
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.set_zoom(clamped);
        }
        // 同步全局默认值，供设置页与新标签场景使用。
        self.zoom = clamped;
    }

    /// 放大（增加 10%）。
    pub fn zoom_in(&mut self) {
        let cur = self.zoom();
        self.set_zoom(cur + 0.1);
    }

    /// 缩小（减少 10%）。
    pub fn zoom_out(&mut self) {
        let cur = self.zoom();
        self.set_zoom(cur - 0.1);
    }

    /// 重置缩放到 100%（作用于活动标签页）。
    pub fn zoom_reset(&mut self) {
        if let Some(tab) = self.tabs.active_tab_mut() {
            tab.set_zoom(1.0);
        }
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
        self.find_state.last_wrap = None;
    }

    /// 跳转到下一个匹配。
    pub fn find_next(&mut self) {
        if self.find_state.total_matches > 0 {
            let next = (self.find_state.current_match % self.find_state.total_matches) + 1;
            // 从末尾回到开头视为循环
            if self.find_state.current_match == self.find_state.total_matches {
                self.find_state.last_wrap = Some(FindWrapHint::WrappedToStart);
            } else {
                self.find_state.last_wrap = None;
            }
            self.find_state.current_match = next;
        }
    }

    /// 跳转到上一个匹配。
    pub fn find_previous(&mut self) {
        if self.find_state.total_matches > 0 {
            if self.find_state.current_match <= 1 {
                self.find_state.current_match = self.find_state.total_matches;
                self.find_state.last_wrap = Some(FindWrapHint::WrappedToEnd);
            } else {
                self.find_state.current_match -= 1;
                self.find_state.last_wrap = None;
            }
        }
    }

    /// 关闭页面查找。
    pub fn find_close(&mut self) {
        self.find_state.active = false;
        self.find_state.query.clear();
        self.find_state.current_match = 0;
        self.find_state.total_matches = 0;
        self.find_state.last_wrap = None;
    }

    /// 更新查找匹配数。
    pub fn find_set_matches(&mut self, total: usize) {
        self.find_state.total_matches = total;
        if total > 0 && self.find_state.current_match == 0 {
            self.find_state.current_match = 1;
        }
    }

    /// 切换「区分大小写」选项。切换后重置匹配计数（实际匹配由渲染层重新计算）。
    pub fn find_toggle_case_sensitive(&mut self) {
        self.find_state.case_sensitive = !self.find_state.case_sensitive;
        self.find_state.current_match = 0;
        self.find_state.total_matches = 0;
        self.find_state.last_wrap = None;
    }

    /// 切换「全字匹配」选项。
    pub fn find_toggle_whole_word(&mut self) {
        self.find_state.whole_word = !self.find_state.whole_word;
        self.find_state.current_match = 0;
        self.find_state.total_matches = 0;
        self.find_state.last_wrap = None;
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
