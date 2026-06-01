//! 地址栏自动补全 — URL 和标题搜索建议。
//!
//! 根据用户输入从历史记录和书签中搜索匹配项，返回按相关度排序的建议列表。

/// 自动补全建议。
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    /// 建议 URL。
    url: String,
    /// 显示标题。
    title: String,
    /// 建议来源。
    source: SuggestionSource,
}

/// 建议来源。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuggestionSource {
    /// 来自历史记录。
    History,
    /// 来自书签。
    Bookmark,
}

impl Suggestion {
    /// 创建新的建议。
    pub fn new(url: &str, title: &str, source: SuggestionSource) -> Self {
        Self {
            url: url.to_string(),
            title: title.to_string(),
            source,
        }
    }

    /// 获取建议 URL。
    pub fn url(&self) -> &str {
        &self.url
    }

    /// 获取显示标题。
    pub fn title(&self) -> &str {
        &self.title
    }

    /// 获取建议来源。
    pub fn source(&self) -> SuggestionSource {
        self.source
    }
}

/// 自动补全引擎。
pub struct Autocomplete {
    /// 最大返回建议数。
    max_results: usize,
}

impl Autocomplete {
    /// 创建默认自动补全引擎。
    pub fn new() -> Self {
        Self { max_results: 10 }
    }

    /// 设置最大返回建议数。
    pub fn with_max_results(mut self, max: usize) -> Self {
        self.max_results = max.max(1);
        self
    }

    /// 从历史记录和书签中搜索匹配建议。
    ///
    /// 搜索不区分大小写，按以下优先级排序：
    /// 1. URL 精确前缀匹配（最高优先级）
    /// 2. 标题精确前缀匹配
    /// 3. URL 包含匹配
    /// 4. 标题包含匹配
    ///
    /// 书签优先于历史记录中的同 URL 条目。
    pub fn suggest(&self, query: &str, history: &crate::History, bookmarks: &crate::Bookmarks) -> Vec<Suggestion> {
        if query.trim().is_empty() {
            return Vec::new();
        }

        let query_lower = query.to_lowercase();
        let mut seen_urls = std::collections::HashSet::new();
        let mut results: Vec<(Suggestion, u32)> = Vec::new();

        // 收集书签匹配
        for bm in bookmarks.iter() {
            let score = score_match(bm.url(), bm.title(), &query_lower);
            if score > 0 {
                seen_urls.insert(bm.url().to_string());
                results.push((
                    Suggestion::new(bm.url(), bm.title(), SuggestionSource::Bookmark),
                    score + 100, // 书签加分
                ));
            }
        }

        // 收集历史匹配（跳过已作为书签出现的 URL）
        for entry in history.iter() {
            if seen_urls.contains(entry.url()) {
                continue;
            }
            let score = score_match(entry.url(), entry.title(), &query_lower);
            if score > 0 {
                seen_urls.insert(entry.url().to_string());
                results.push((
                    Suggestion::new(entry.url(), entry.title(), SuggestionSource::History),
                    score,
                ));
            }
        }

        // 按分数降序排序
        results.sort_by_key(|b| std::cmp::Reverse(b.1));

        // 截断到 max_results
        results.truncate(self.max_results);
        results.into_iter().map(|(s, _)| s).collect()
    }
}

impl Default for Autocomplete {
    fn default() -> Self {
        Self::new()
    }
}

/// 计算匹配分数（0 表示不匹配）。
fn score_match(url: &str, title: &str, query_lower: &str) -> u32 {
    let url_lower = url.to_lowercase();
    let title_lower = title.to_lowercase();

    let url_score = if url_lower.starts_with(query_lower) {
        40 // URL 前缀精确匹配
    } else if url_lower.contains(query_lower) {
        20 // URL 包含匹配
    } else {
        0
    };

    let title_score = if title_lower.starts_with(query_lower) {
        30 // 标题前缀匹配
    } else if title_lower.contains(query_lower) {
        10 // 标题包含匹配
    } else {
        0
    };

    url_score + title_score
}
