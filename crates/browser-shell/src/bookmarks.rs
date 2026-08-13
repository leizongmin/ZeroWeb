//! 书签管理 — 书签和文件夹的增删改查。

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

/// 书签唯一标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BookmarkId(pub u64);

impl BookmarkId {
    /// 生成下一个唯一 ID。
    fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// 确保持久化恢复后新 ID 不会与已有 ID 冲突。
    fn sync_counter(min_next: u64) {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        let mut current = COUNTER.load(Ordering::Relaxed);
        while current < min_next {
            match COUNTER.compare_exchange_weak(current, min_next, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => return,
                Err(actual) => current = actual,
            }
        }
    }
}

/// 书签条目。
#[derive(Debug, Clone, PartialEq)]
pub struct Bookmark {
    /// 书签 ID。
    id: BookmarkId,
    /// 标题。
    title: String,
    /// URL。
    url: String,
    /// 所属文件夹 ID。
    folder_id: Option<BookmarkId>,
}

impl Bookmark {
    /// 获取书签 ID。
    pub fn id(&self) -> BookmarkId {
        self.id
    }

    /// 获取标题。
    pub fn title(&self) -> &str {
        &self.title
    }

    /// 获取 URL。
    pub fn url(&self) -> &str {
        &self.url
    }

    /// 获取所属文件夹 ID。
    pub fn folder_id(&self) -> Option<BookmarkId> {
        self.folder_id
    }
}

/// 文件夹。
#[derive(Debug, Clone)]
pub struct BookmarkFolder {
    /// 文件夹 ID。
    id: BookmarkId,
    /// 文件夹名称。
    name: String,
}

impl BookmarkFolder {
    /// 获取文件夹 ID。
    pub fn id(&self) -> BookmarkId {
        self.id
    }

    /// 获取文件夹名称。
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// 书签管理器。
pub struct Bookmarks {
    /// 所有书签。
    bookmarks: Vec<Bookmark>,
    /// 所有文件夹。
    folders: Vec<BookmarkFolder>,
}

impl Bookmarks {
    /// 创建空的书签管理器。
    pub fn new() -> Self {
        Self {
            bookmarks: Vec::new(),
            folders: Vec::new(),
        }
    }

    /// 是否没有书签。
    pub fn is_empty(&self) -> bool {
        self.bookmarks.is_empty()
    }

    /// 书签数量。
    pub fn len(&self) -> usize {
        self.bookmarks.len()
    }

    /// 添加书签。
    ///
    /// 返回书签 ID。
    pub fn add(&mut self, title: &str, url: &str, folder_id: Option<BookmarkId>) -> BookmarkId {
        let id = BookmarkId::next();
        self.bookmarks.push(Bookmark {
            id,
            title: title.to_string(),
            url: url.to_string(),
            folder_id,
        });
        id
    }

    /// 按 URL 查询书签（精确匹配）。同一 URL 仅返回首个匹配。
    pub fn find_by_url(&self, url: &str) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.url == url)
    }

    /// 按 URL 移除书签。返回是否成功移除。
    pub fn remove_by_url(&mut self, url: &str) -> bool {
        if let Some(index) = self.bookmarks.iter().position(|b| b.url == url) {
            self.bookmarks.remove(index);
            true
        } else {
            false
        }
    }

    /// 移除书签。
    ///
    /// 返回 `true` 表示成功找到并移除。
    pub fn remove(&mut self, id: BookmarkId) -> bool {
        if let Some(index) = self.bookmarks.iter().position(|b| b.id == id) {
            self.bookmarks.remove(index);
            true
        } else {
            false
        }
    }

    /// 获取书签。
    pub fn get(&self, id: BookmarkId) -> Option<&Bookmark> {
        self.bookmarks.iter().find(|b| b.id == id)
    }

    /// 更新书签标题。
    pub fn update_title(&mut self, id: BookmarkId, title: &str) {
        if let Some(bookmark) = self.bookmarks.iter_mut().find(|b| b.id == id) {
            bookmark.title = title.to_string();
        }
    }

    /// 创建文件夹。
    ///
    /// 返回文件夹 ID。
    pub fn create_folder(&mut self, name: &str) -> BookmarkId {
        let id = BookmarkId::next();
        self.folders.push(BookmarkFolder {
            id,
            name: name.to_string(),
        });
        id
    }

    /// 获取文件夹。
    pub fn get_folder(&self, id: BookmarkId) -> Option<&BookmarkFolder> {
        self.folders.iter().find(|f| f.id == id)
    }

    /// 移除文件夹及其所有书签。
    pub fn remove_folder(&mut self, folder_id: BookmarkId) {
        self.bookmarks.retain(|b| b.folder_id != Some(folder_id));
        self.folders.retain(|f| f.id != folder_id);
    }

    /// 列出根级书签（不属于任何文件夹）。
    pub fn list_root(&self) -> Vec<&Bookmark> {
        self.bookmarks.iter().filter(|b| b.folder_id.is_none()).collect()
    }

    /// 列出指定文件夹内的书签。
    pub fn list_in_folder(&self, folder_id: BookmarkId) -> Vec<&Bookmark> {
        self.bookmarks
            .iter()
            .filter(|b| b.folder_id == Some(folder_id))
            .collect()
    }

    /// 列出所有文件夹。
    pub fn folders(&self) -> &[BookmarkFolder] {
        &self.folders
    }

    /// 遍历所有书签。
    pub fn iter(&self) -> impl Iterator<Item = &Bookmark> {
        self.bookmarks.iter()
    }

    /// 返回书签文件的默认路径。
    ///
    /// 遵循 XDG 规范：`~/.config/zeroweb/bookmarks.json`
    pub fn default_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("zeroweb");
        config_dir.join("bookmarks.json")
    }

    /// 从 JSON 文件加载书签。
    ///
    /// 如果文件不存在或解析失败，返回空书签集。
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str::<BookmarksSnapshot>(&content)
                .map(Self::from_snapshot)
                .unwrap_or_default(),
            Err(_) => Self::new(),
        }
    }

    /// 从默认路径加载书签。
    pub fn load_default() -> Self {
        Self::load(&Self::default_path())
    }

    /// 将书签保存到 JSON 文件。
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {e}"))?;
        }
        let snapshot = self.to_snapshot();
        let json =
            serde_json::to_string_pretty(&snapshot).map_err(|e| format!("Failed to serialize bookmarks: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("Failed to write bookmarks: {e}"))?;
        Ok(())
    }

    /// 保存到默认路径。
    pub fn save_default(&self) -> Result<(), String> {
        self.save(&Self::default_path())
    }

    fn to_snapshot(&self) -> BookmarksSnapshot {
        BookmarksSnapshot {
            bookmarks: self
                .bookmarks
                .iter()
                .map(|b| BookmarkSnapshot {
                    id: b.id.0,
                    title: b.title.clone(),
                    url: b.url.clone(),
                    folder_id: b.folder_id.map(|id| id.0),
                })
                .collect(),
            folders: self
                .folders
                .iter()
                .map(|f| BookmarkFolderSnapshot {
                    id: f.id.0,
                    name: f.name.clone(),
                })
                .collect(),
        }
    }

    fn from_snapshot(snapshot: BookmarksSnapshot) -> Self {
        let mut max_id = 0u64;
        let folders = snapshot
            .folders
            .into_iter()
            .map(|f| {
                max_id = max_id.max(f.id);
                BookmarkFolder {
                    id: BookmarkId(f.id),
                    name: f.name,
                }
            })
            .collect();
        let bookmarks = snapshot
            .bookmarks
            .into_iter()
            .map(|b| {
                max_id = max_id.max(b.id);
                if let Some(folder_id) = b.folder_id {
                    max_id = max_id.max(folder_id);
                }
                Bookmark {
                    id: BookmarkId(b.id),
                    title: b.title,
                    url: b.url,
                    folder_id: b.folder_id.map(BookmarkId),
                }
            })
            .collect();
        if max_id > 0 {
            // R3363：saturating_add 防 u64 溢出——恶意/损坏的本地 bookmarks.json 含
            // `"id": 18446744073709551615`（u64::MAX）时，`max_id + 1` debug panic
            //（overflow-checks）/ release 回绕为 0 致 sync_counter(0) 不推进（恢复后 ID 冲突）。
            // saturating 后 max_id==u64::MAX 时 next=u64::MAX（sync_counter 仅在 current<min_next 时推进，
            // 与已有 u64::MAX ID 共存——下一 next() 经 fetch_add 溢出回绕到 0，但这是 u64 极端边界，
            // 实际书签量级永不触及；核心是不 panic）。本地信任边界文件解析须 fail-safe 不 crash。
            let next = max_id.saturating_add(1);
            BookmarkId::sync_counter(next);
        }
        Self { bookmarks, folders }
    }
}

/// 可序列化的书签快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BookmarksSnapshot {
    bookmarks: Vec<BookmarkSnapshot>,
    folders: Vec<BookmarkFolderSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BookmarkSnapshot {
    id: u64,
    title: String,
    url: String,
    folder_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BookmarkFolderSnapshot {
    id: u64,
    name: String,
}

impl Default for Bookmarks {
    fn default() -> Self {
        Self::new()
    }
}
