//! 浏览器设置 — 用户偏好和配置管理。

/// 默认搜索引擎。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchEngine {
    /// Google。
    Google,
    /// Bing。
    Bing,
    /// DuckDuckGo。
    DuckDuckGo,
    /// 百度。
    Baidu,
}

impl SearchEngine {
    /// 获取搜索 URL 模板（`{query}` 为占位符）。
    pub fn search_url(&self, query: &str) -> String {
        let encoded = query.replace(' ', "+");
        match self {
            SearchEngine::Google => format!("https://www.google.com/search?q={encoded}"),
            SearchEngine::Bing => format!("https://www.bing.com/search?q={encoded}"),
            SearchEngine::DuckDuckGo => format!("https://duckduckgo.com/?q={encoded}"),
            SearchEngine::Baidu => format!("https://www.baidu.com/s?wd={encoded}"),
        }
    }
}

/// 浏览器设置。
#[derive(Debug, Clone)]
pub struct BrowserSettings {
    /// 默认搜索引擎。
    pub search_engine: SearchEngine,
    /// 主页 URL。
    pub home_url: String,
    /// 是否显示书签栏。
    pub show_bookmarks_bar: bool,
    /// 是否允许 JavaScript。
    pub javascript_enabled: bool,
    /// 是否允许 Cookie。
    pub cookies_enabled: bool,
    /// 是否阻止第三方 Cookie。
    pub block_third_party_cookies: bool,
    /// 是否发送 Do Not Track 头。
    pub do_not_track: bool,
    /// 默认缩放级别（1.0 = 100%）。
    pub default_zoom: f32,
    /// 下载目录（空表示使用系统默认）。
    pub download_directory: String,
}

impl Default for BrowserSettings {
    fn default() -> Self {
        Self {
            search_engine: SearchEngine::Google,
            home_url: "https://example.com".to_string(),
            show_bookmarks_bar: true,
            javascript_enabled: true,
            cookies_enabled: true,
            block_third_party_cookies: true,
            do_not_track: false,
            default_zoom: 1.0,
            download_directory: String::new(),
        }
    }
}

impl BrowserSettings {
    /// 创建默认设置。
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用指定搜索引擎生成搜索 URL。
    pub fn search(&self, query: &str) -> String {
        self.search_engine.search_url(query)
    }
}
