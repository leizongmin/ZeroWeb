//! 标签页管理 — Tab 数据模型和 TabManager。

use std::sync::atomic::{AtomicU64, Ordering};

/// 标签页唯一标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TabId(pub u64);

impl TabId {
    /// 生成下一个唯一 ID。
    fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// 标签页 — 代表一个浏览器标签页的状态。
#[derive(Debug, Clone)]
pub struct Tab {
    /// 标签页 ID。
    id: TabId,
    /// 当前 URL。
    url: Option<String>,
    /// 页面标题。
    title: Option<String>,
    /// 是否正在加载。
    loading: bool,
    /// 导航历史索引（用于前进/后退）。
    history_index: usize,
    /// 导航历史列表。
    history: Vec<NavigationEntry>,
    /// 无痕模式（不写磁盘缓存、不写入会话）。
    private: bool,
    /// 是否固定标签页。
    pinned: bool,
    /// 是否静音。
    muted: bool,
    /// 是否崩溃。
    crashed: bool,
    /// 是否需要用户注意（例如完成通知）。
    needs_attention: bool,
}

/// 导航历史条目。
#[derive(Debug, Clone)]
pub struct NavigationEntry {
    /// URL。
    pub(crate) url: String,
    /// 页面标题。
    pub(crate) title: Option<String>,
}

impl Tab {
    /// 创建新的标签页。
    pub fn new(url: &str) -> Self {
        let mut tab = Self {
            id: TabId::next(),
            url: None,
            title: None,
            loading: false,
            history_index: 0,
            history: Vec::new(),
            private: false,
            pinned: false,
            muted: false,
            crashed: false,
            needs_attention: false,
        };
        tab.navigate(url);
        tab
    }

    /// 创建无痕标签页。
    pub fn new_private(url: &str) -> Self {
        let mut tab = Self::new(url);
        tab.private = true;
        tab
    }

    /// 创建空白无痕标签页。
    pub fn new_empty_private() -> Self {
        let mut tab = Self::new_empty();
        tab.private = true;
        tab
    }

    /// 创建空白标签页（无 URL）。
    pub fn new_empty() -> Self {
        Self {
            id: TabId::next(),
            url: None,
            title: None,
            loading: false,
            history_index: 0,
            history: Vec::new(),
            private: false,
            pinned: false,
            muted: false,
            crashed: false,
            needs_attention: false,
        }
    }

    /// 创建当前标签页的副本（新 ID，保留 URL/标题/历史/固定/无痕等状态，重置加载与崩溃状态）。
    pub fn duplicate(&self) -> Self {
        Self {
            id: TabId::next(),
            url: self.url.clone(),
            title: self.title.clone(),
            loading: false,
            history_index: self.history_index,
            history: self.history.clone(),
            private: self.private,
            pinned: self.pinned,
            muted: self.muted,
            crashed: false,
            needs_attention: false,
        }
    }

    /// 是否无痕标签页。
    pub fn is_private(&self) -> bool {
        self.private
    }

    /// 是否固定标签页。
    pub fn is_pinned(&self) -> bool {
        self.pinned
    }

    /// 设置固定状态。
    pub fn set_pinned(&mut self, pinned: bool) {
        self.pinned = pinned;
    }

    /// 是否静音。
    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// 设置静音状态。
    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    /// 是否崩溃。
    pub fn is_crashed(&self) -> bool {
        self.crashed
    }

    /// 设置崩溃状态。
    pub fn set_crashed(&mut self, crashed: bool) {
        self.crashed = crashed;
    }

    /// 是否需要用户注意。
    pub fn needs_attention(&self) -> bool {
        self.needs_attention
    }

    /// 设置是否需要用户注意。
    pub fn set_needs_attention(&mut self, needs_attention: bool) {
        self.needs_attention = needs_attention;
    }

    /// 获取标签页 ID。
    pub fn id(&self) -> TabId {
        self.id
    }

    /// 获取当前 URL。
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// 获取页面标题。
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// 设置页面标题。
    pub fn set_title(&mut self, title: &str) {
        self.title = Some(title.to_string());
        // 更新历史中当前条目的标题
        if let Some(entry) = self.history.get_mut(self.history_index) {
            entry.title = Some(title.to_string());
        }
    }

    /// 是否正在加载。
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// 设置加载状态。
    pub fn set_loading(&mut self, loading: bool) {
        self.loading = loading;
    }

    /// 导航到新 URL。
    pub fn navigate(&mut self, url: &str) {
        // 截断前进历史
        self.history.truncate(self.history_index + 1);
        self.history.push(NavigationEntry {
            url: url.to_string(),
            title: None,
        });
        self.history_index = self.history.len() - 1;
        self.url = Some(url.to_string());
        self.loading = true;
        self.title = None;
    }

    /// 设置 URL（不记录到历史）。
    pub fn set_url(&mut self, url: &str) {
        self.url = Some(url.to_string());
        self.loading = true;
    }

    /// 前进。
    ///
    /// 返回 `true` 表示成功，`false` 表示已无前进历史。
    pub fn go_forward(&mut self) -> bool {
        if !self.history.is_empty() && self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            let entry = &self.history[self.history_index];
            self.url = Some(entry.url.clone());
            self.title = entry.title.clone();
            self.loading = false;
            true
        } else {
            false
        }
    }

    /// 后退。
    ///
    /// 返回 `true` 表示成功，`false` 表示已无后退历史。
    pub fn go_back(&mut self) -> bool {
        if self.history_index > 0 {
            self.history_index -= 1;
            let entry = &self.history[self.history_index];
            self.url = Some(entry.url.clone());
            self.title = entry.title.clone();
            self.loading = false;
            true
        } else {
            false
        }
    }

    /// 获取导航历史长度。
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// 获取当前历史索引。
    pub fn history_index(&self) -> usize {
        self.history_index
    }

    /// 获取导航历史条目的引用。
    pub fn navigation_history(&self) -> &[NavigationEntry] {
        &self.history
    }

    /// 清除导航历史（会话恢复内部使用）。
    pub(crate) fn clear_history(&mut self) {
        self.history.clear();
        self.history_index = 0;
    }

    /// 向导航历史追加一条记录（会话恢复内部使用）。
    pub(crate) fn push_navigation(&mut self, url: &str, title: Option<&str>) {
        self.history.push(NavigationEntry {
            url: url.to_string(),
            title: title.map(|s| s.to_string()),
        });
    }

    /// 设置 URL 而不触发加载状态（会话恢复内部使用）。
    pub(crate) fn set_url_internal(&mut self, url: &str) {
        self.url = Some(url.to_string());
        self.loading = false;
    }
}

/// 标签页管理器 — 管理多个标签页的创建、关闭、切换。
pub struct TabManager {
    /// 所有标签页。
    tabs: Vec<Tab>,
    /// 当前活跃标签页索引。
    active_index: Option<usize>,
}

impl TabManager {
    /// 创建空的标签页管理器。
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_index: None,
        }
    }

    /// 是否没有标签页。
    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    /// 标签页数量。
    pub fn len(&self) -> usize {
        self.tabs.len()
    }

    /// 创建新标签页并设为活跃。
    ///
    /// 返回新标签页的 ID。
    pub fn create_tab(&mut self, url: Option<&str>) -> TabId {
        let tab = match url {
            Some(url) => Tab::new(url),
            None => Tab::new_empty(),
        };
        let id = tab.id();
        self.tabs.push(tab);
        self.active_index = Some(self.tabs.len() - 1);
        id
    }

    /// 创建标签页但**不**切换为活跃（后台打开），返回新标签 id。
    /// 用于 Ctrl+点击链接等"在新标签打开但不离开当前页"场景。
    pub fn create_tab_background(&mut self, url: Option<&str>) -> TabId {
        let tab = match url {
            Some(url) => Tab::new(url),
            None => Tab::new_empty(),
        };
        let id = tab.id();
        self.tabs.push(tab);
        // active_index 保持不变，新标签在后台。
        id
    }

    /// 创建无痕标签页并设为活跃。
    pub fn create_private_tab(&mut self, url: Option<&str>) -> TabId {
        let tab = match url {
            Some(url) => Tab::new_private(url),
            None => Tab::new_empty_private(),
        };
        let id = tab.id();
        self.tabs.push(tab);
        self.active_index = Some(self.tabs.len() - 1);
        id
    }

    /// 切换到指定标签页。
    pub fn switch_to(&mut self, id: TabId) {
        if let Some(index) = self.tabs.iter().position(|t| t.id() == id) {
            self.active_index = Some(index);
        }
    }

    /// 关闭指定标签页。
    ///
    /// 如果关闭的是活跃标签页，自动切换到相邻标签页。
    pub fn close_tab(&mut self, id: TabId) {
        if let Some(index) = self.tabs.iter().position(|t| t.id() == id) {
            self.tabs.remove(index);

            if self.tabs.is_empty() {
                self.active_index = None;
            } else {
                // 切换到之前的标签页，或最后一个
                let new_index = if index > 0 { index - 1 } else { 0 };
                self.active_index = Some(new_index.min(self.tabs.len() - 1));
            }
        }
    }

    /// 复制指定标签页，副本插入到原标签页之后并设为活跃。
    ///
    /// 返回新标签页的 ID；若指定标签页不存在则返回 `None`。
    pub fn duplicate_tab(&mut self, id: TabId) -> Option<TabId> {
        let index = self.tabs.iter().position(|t| t.id() == id)?;
        let copy = self.tabs[index].duplicate();
        let new_id = copy.id();
        self.tabs.insert(index + 1, copy);
        self.active_index = Some(index + 1);
        Some(new_id)
    }

    /// 关闭除指定标签页外的所有标签页，并将活跃标签页切换为该标签页。
    pub fn close_other_tabs(&mut self, id: TabId) {
        let Some(keep_index) = self.tabs.iter().position(|t| t.id() == id) else {
            return;
        };
        self.tabs.retain(|t| t.id() == id);
        self.active_index = if self.tabs.is_empty() { None } else { Some(0) };
        let _ = keep_index;
    }

    /// 关闭指定标签页右侧的所有标签页。
    pub fn close_tabs_to_right(&mut self, id: TabId) {
        let Some(index) = self.tabs.iter().position(|t| t.id() == id) else {
            return;
        };
        self.tabs.truncate(index + 1);
        if self.tabs.is_empty() {
            self.active_index = None;
        } else {
            self.active_index = Some(self.active_index.unwrap_or(0).min(self.tabs.len() - 1));
        }
    }

    /// 获取活跃标签页 ID。
    pub fn active_tab_id(&self) -> Option<TabId> {
        self.active_index.and_then(|i| self.tabs.get(i).map(|t| t.id()))
    }

    /// 获取活跃标签页的引用。
    pub fn active_tab(&self) -> Option<&Tab> {
        self.active_index.and_then(|i| self.tabs.get(i))
    }

    /// 获取活跃标签页的可变引用。
    pub fn active_tab_mut(&mut self) -> Option<&mut Tab> {
        self.active_index.and_then(|i| self.tabs.get_mut(i))
    }

    /// 获取指定标签页的引用。
    pub fn get_tab(&self, id: TabId) -> Option<&Tab> {
        self.tabs.iter().find(|t| t.id() == id)
    }

    /// 获取指定标签页的可变引用。
    pub fn get_tab_mut(&mut self, id: TabId) -> Option<&mut Tab> {
        self.tabs.iter_mut().find(|t| t.id() == id)
    }

    /// 遍历所有标签页。
    pub fn tabs(&self) -> impl Iterator<Item = &Tab> {
        self.tabs.iter()
    }

    /// 获取活跃标签页索引。
    pub fn active_index(&self) -> Option<usize> {
        self.active_index
    }

    /// 移动标签页位置（拖拽排序）。
    ///
    /// 将指定 ID 的标签页移动到目标索引位置。
    /// 返回 `true` 表示成功移动。
    pub fn move_tab(&mut self, id: TabId, to_index: usize) -> bool {
        let Some(from_index) = self.tabs.iter().position(|t| t.id() == id) else {
            return false;
        };
        if from_index == to_index || to_index >= self.tabs.len() {
            return false;
        }
        // 保存活跃标签页 ID（move 前记录）
        let active_id = self.active_index.and_then(|i| self.tabs.get(i).map(|t| t.id()));
        let tab = self.tabs.remove(from_index);
        // 插入位置需要调整：如果从前面移到后面，索引会偏移
        let insert_at = if from_index < to_index {
            to_index.min(self.tabs.len())
        } else {
            to_index
        };
        self.tabs.insert(insert_at, tab);
        // 重新定位活跃索引
        if let Some(aid) = active_id {
            self.active_index = self.tabs.iter().position(|t| t.id() == aid);
        }
        true
    }
}

impl Default for TabManager {
    fn default() -> Self {
        Self::new()
    }
}
