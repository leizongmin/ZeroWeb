//! 下载管理器 — 文件下载的状态追踪和管理。

use std::sync::atomic::{AtomicU64, Ordering};

/// 下载唯一标识符。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DownloadId(pub u64);

impl DownloadId {
    /// 生成下一个唯一 ID。
    fn next() -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// 下载状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadState {
    /// 等待开始。
    Pending,
    /// 正在下载。
    Downloading,
    /// 已暂停。
    Paused,
    /// 已完成。
    Completed,
    /// 已取消。
    Cancelled,
    /// 下载失败。
    Failed,
}

/// 下载条目。
#[derive(Debug, Clone)]
pub struct DownloadEntry {
    /// 下载 ID。
    id: DownloadId,
    /// 下载 URL。
    url: String,
    /// 文件名。
    filename: String,
    /// 已下载字节数。
    downloaded_bytes: u64,
    /// 总字节数（未知为 None）。
    total_bytes: Option<u64>,
    /// 下载状态。
    state: DownloadState,
}

impl DownloadEntry {
    /// 获取下载 ID。
    pub fn id(&self) -> DownloadId {
        self.id
    }

    /// 获取下载 URL。
    pub fn url(&self) -> &str {
        &self.url
    }

    /// 获取文件名。
    pub fn filename(&self) -> &str {
        &self.filename
    }

    /// 获取已下载字节数。
    pub fn downloaded_bytes(&self) -> u64 {
        self.downloaded_bytes
    }

    /// 获取总字节数。
    pub fn total_bytes(&self) -> Option<u64> {
        self.total_bytes
    }

    /// 获取下载状态。
    pub fn state(&self) -> DownloadState {
        self.state
    }

    /// 获取下载进度（0.0 ~ 1.0），总大小未知时返回 0.0。
    pub fn progress(&self) -> f32 {
        match self.total_bytes {
            Some(total) if total > 0 => (self.downloaded_bytes as f32 / total as f32).min(1.0),
            _ => 0.0,
        }
    }

    /// 是否已完成。
    pub fn is_completed(&self) -> bool {
        self.state == DownloadState::Completed
    }

    /// 是否正在下载。
    pub fn is_active(&self) -> bool {
        matches!(self.state, DownloadState::Downloading | DownloadState::Pending)
    }
}

/// 下载管理器。
pub struct DownloadManager {
    /// 所有下载条目。
    downloads: Vec<DownloadEntry>,
}

impl DownloadManager {
    /// 创建空的下载管理器。
    pub fn new() -> Self {
        Self { downloads: Vec::new() }
    }

    /// 是否没有下载。
    pub fn is_empty(&self) -> bool {
        self.downloads.is_empty()
    }

    /// 下载数量。
    pub fn len(&self) -> usize {
        self.downloads.len()
    }

    /// 创建新的下载任务。
    ///
    /// 返回下载 ID。
    pub fn start_download(&mut self, url: &str, filename: &str) -> DownloadId {
        let id = DownloadId::next();
        self.downloads.push(DownloadEntry {
            id,
            url: url.to_string(),
            filename: filename.to_string(),
            downloaded_bytes: 0,
            total_bytes: None,
            state: DownloadState::Pending,
        });
        id
    }

    /// 更新下载进度。
    pub fn update_progress(&mut self, id: DownloadId, downloaded: u64, total: Option<u64>) {
        if let Some(entry) = self.downloads.iter_mut().find(|d| d.id == id) {
            entry.downloaded_bytes = downloaded;
            entry.total_bytes = total;
            if entry.state == DownloadState::Pending {
                entry.state = DownloadState::Downloading;
            }
        }
    }

    /// 标记下载为完成。
    pub fn mark_completed(&mut self, id: DownloadId) {
        if let Some(entry) = self.downloads.iter_mut().find(|d| d.id == id) {
            entry.state = DownloadState::Completed;
            if let Some(total) = entry.total_bytes {
                entry.downloaded_bytes = total;
            }
        }
    }

    /// 暂停下载。
    pub fn pause(&mut self, id: DownloadId) {
        if let Some(entry) = self.downloads.iter_mut().find(|d| d.id == id)
            && (entry.state == DownloadState::Downloading || entry.state == DownloadState::Pending)
        {
            entry.state = DownloadState::Paused;
        }
    }

    /// 恢复下载。
    pub fn resume(&mut self, id: DownloadId) {
        if let Some(entry) = self.downloads.iter_mut().find(|d| d.id == id)
            && entry.state == DownloadState::Paused
        {
            entry.state = DownloadState::Downloading;
        }
    }

    /// 取消下载。
    pub fn cancel(&mut self, id: DownloadId) {
        if let Some(entry) = self.downloads.iter_mut().find(|d| d.id == id)
            && entry.state != DownloadState::Completed
        {
            entry.state = DownloadState::Cancelled;
        }
    }

    /// 标记下载失败。
    pub fn mark_failed(&mut self, id: DownloadId) {
        if let Some(entry) = self.downloads.iter_mut().find(|d| d.id == id) {
            entry.state = DownloadState::Failed;
        }
    }

    /// 移除下载记录（仅限已完成/已取消/失败的）。
    pub fn remove(&mut self, id: DownloadId) -> bool {
        if let Some(index) = self.downloads.iter().position(|d| d.id == id) {
            let entry = &self.downloads[index];
            if matches!(
                entry.state,
                DownloadState::Completed | DownloadState::Cancelled | DownloadState::Failed
            ) {
                self.downloads.remove(index);
                return true;
            }
        }
        false
    }

    /// 清除所有已完成的下载记录。
    pub fn clear_completed(&mut self) {
        self.downloads.retain(|d| {
            !matches!(
                d.state,
                DownloadState::Completed | DownloadState::Cancelled | DownloadState::Failed
            )
        });
    }

    /// 获取指定下载条目。
    pub fn get(&self, id: DownloadId) -> Option<&DownloadEntry> {
        self.downloads.iter().find(|d| d.id == id)
    }

    /// 遍历所有下载。
    pub fn iter(&self) -> impl Iterator<Item = &DownloadEntry> {
        self.downloads.iter()
    }

    /// 获取活跃下载数量。
    pub fn active_count(&self) -> usize {
        self.downloads.iter().filter(|d| d.is_active()).count()
    }
}

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new()
    }
}
