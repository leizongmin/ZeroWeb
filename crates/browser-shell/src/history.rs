//! 历史记录管理 — 页面访问历史的记录、搜索、清理和持久化。

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::profile::{atomic_write, read_profile};

const MAX_ENTRIES: usize = 10_000;
const MAX_TEXT_BYTES: usize = 16 * 1024;

/// 历史记录条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// URL。
    url: String,
    /// 页面标题。
    title: String,
}

impl HistoryEntry {
    /// 获取 URL。
    pub fn url(&self) -> &str {
        &self.url
    }

    /// 获取页面标题。
    pub fn title(&self) -> &str {
        &self.title
    }
}

/// 历史记录管理器。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    /// 历史记录列表（按时间倒序排列，最新的在前）。
    entries: Vec<HistoryEntry>,
}

impl History {
    /// 创建空的历史记录管理器。
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    /// 是否没有历史记录。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 历史记录数量。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 记录一次页面访问。
    ///
    /// 如果相同 URL 已存在，更新其标题和时间（移到最前）。
    pub fn record(&mut self, url: &str, title: &str) {
        // 如果已有相同 URL，移除旧记录
        self.entries.retain(|e| e.url != url);
        // 在最前面添加新记录
        self.entries.insert(
            0,
            HistoryEntry {
                url: url.to_string(),
                title: title.to_string(),
            },
        );
        self.entries.truncate(MAX_ENTRIES);
    }

    /// 搜索历史记录（按 URL 或标题匹配）。
    ///
    /// 搜索不区分大小写。
    pub fn search<'a>(&'a self, query: &str) -> impl Iterator<Item = &'a HistoryEntry> {
        let query_lower = query.to_lowercase();
        self.entries.iter().filter(move |e| {
            e.url.to_lowercase().contains(&query_lower) || e.title.to_lowercase().contains(&query_lower)
        })
    }

    /// 清除所有历史记录。
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// 遍历所有历史记录。
    pub fn iter(&self) -> impl Iterator<Item = &HistoryEntry> {
        self.entries.iter()
    }

    /// 从指定 JSON 文件恢复历史记录；损坏或越界内容返回空历史。
    pub fn load(path: &Path) -> Self {
        let Some(content) = read_profile(path) else {
            return Self::new();
        };
        let Ok(history) = serde_json::from_str::<Self>(&content) else {
            return Self::new();
        };
        if history.entries.len() > MAX_ENTRIES
            || history
                .entries
                .iter()
                .any(|entry| entry.url.len() > MAX_TEXT_BYTES || entry.title.len() > MAX_TEXT_BYTES)
        {
            return Self::new();
        }
        history
    }

    /// 将历史记录原子保存到指定 JSON 文件。
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self).map_err(|error| format!("serialize history failed: {error}"))?;
        atomic_write(path, &json)
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
