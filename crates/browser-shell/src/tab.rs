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
        };
        tab.navigate(url);
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
        }
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
