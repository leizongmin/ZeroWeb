//! 书签管理 — 书签和文件夹的增删改查。

use std::sync::atomic::{AtomicU64, Ordering};

/// 书签唯一标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BookmarkId(pub u64);

impl BookmarkId {
    /// 生成下一个唯一 ID。
    fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
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
}

impl Default for Bookmarks {
    fn default() -> Self {
        Self::new()
    }
}
