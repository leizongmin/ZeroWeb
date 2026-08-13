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
    /// 搜索引擎建议（用当前默认搜索引擎搜索输入词）。
    Search,
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

    /// 创建搜索建议。
    ///
    /// `query` 同时作为显示标题和导航输入（由 `normalize_url` 转换为搜索引擎 URL）。
    pub fn new_search(query: &str) -> Self {
        Self {
            url: query.to_string(),
            title: query.to_string(),
            source: SuggestionSource::Search,
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
    /// 搜索不区分大小写。**单来源内**按以下分数排序（见 [`score_match`]）：
    /// 1. URL 精确前缀匹配（40 分）
    /// 2. 标题精确前缀匹配（30 分）
    /// 3. URL 包含匹配（20 分）
    /// 4. 标题包含匹配（10 分）
    ///
    /// **跨来源**：书签条目额外加 100 分（书签是用户主动收藏，整体优先于历史项），
    /// 因此一个书签（即便仅标题包含匹配）会排在历史项（即便 URL 前缀精确匹配）之前。
    /// 同 URL 同时存在于书签与历史时，去重后只保留书签条目。
    ///
    /// 若输入不像 URL，会在列表顶部插入一条搜索建议。
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

        // 截断到 max_results，给搜索建议预留 1 个槽位
        let history_cap = self.max_results.saturating_sub(1).max(1);
        results.truncate(history_cap);

        let mut suggestions: Vec<Suggestion> = results.into_iter().map(|(s, _)| s).collect();

        // 当输入看起来不像 URL 时，在顶部插入搜索建议
        if !looks_like_url(query) {
            let search = Suggestion::new_search(query.trim());
            suggestions.insert(0, search);
        }

        suggestions
    }
}

impl Default for Autocomplete {
    fn default() -> Self {
        Self::new()
    }
}

/// 粗略判断输入是否看起来像 URL（而非搜索词）。
///
/// 命中以下任一条件视为 URL：
/// - 含 scheme（`http://`、`https://`、`ftp://` 等）
/// - 含 `localhost`
/// - 形如 `example.com` / `example.com:8080`（无空格、含点号、后缀像域名）
fn looks_like_url(input: &str) -> bool {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.contains(' ') {
        return false;
    }
    let lower = trimmed.to_lowercase();
    if lower.contains("://") {
        return true;
    }
    if lower.starts_with("localhost") {
        return true;
    }
    // 形如 host.tld 或 host.tld:port
    if let Some((host, _)) = lower.split_once(':') {
        return host_has_dot_tld(host);
    }
    host_has_dot_tld(&lower)
}

fn host_has_dot_tld(host: &str) -> bool {
    let last_dot = match host.rfind('.') {
        Some(i) => i,
        None => return false,
    };
    let tld = &host[last_dot + 1..];
    !tld.is_empty() && tld.chars().all(|c| c.is_ascii_alphabetic()) && tld.len() >= 2
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
