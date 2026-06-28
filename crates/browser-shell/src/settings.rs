//! 浏览器设置 — 用户偏好和配置管理。

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 浏览器外壳配色主题偏好。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorThemePreference {
    /// 跟随操作系统（无法探测时默认亮色）。
    #[default]
    Auto,
    /// 始终亮色。
    Light,
    /// 始终暗色。
    Dark,
}

impl ColorThemePreference {
    /// 在 Auto → Light → Dark → Auto 间轮换。
    pub fn cycle(self) -> Self {
        match self {
            Self::Auto => Self::Light,
            Self::Light => Self::Dark,
            Self::Dark => Self::Auto,
        }
    }

    /// 从设置页使用的名称解析。
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "auto" => Some(Self::Auto),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

/// 默认搜索引擎。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
    /// 切换到下一个搜索引擎（Google → Bing → DuckDuckGo → 百度 → Google）。
    pub fn cycle(self) -> Self {
        match self {
            SearchEngine::Google => SearchEngine::Bing,
            SearchEngine::Bing => SearchEngine::DuckDuckGo,
            SearchEngine::DuckDuckGo => SearchEngine::Baidu,
            SearchEngine::Baidu => SearchEngine::Google,
        }
    }

    /// 从设置页使用的名称解析搜索引擎。
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "Google" => Some(SearchEngine::Google),
            "Bing" => Some(SearchEngine::Bing),
            "DuckDuckGo" => Some(SearchEngine::DuckDuckGo),
            "Baidu" => Some(SearchEngine::Baidu),
            _ => None,
        }
    }

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

    /// 用户可见的搜索引擎名称（用于 UI 展示）。
    pub fn display_name(self) -> &'static str {
        match self {
            SearchEngine::Google => "Google",
            SearchEngine::Bing => "Bing",
            SearchEngine::DuckDuckGo => "DuckDuckGo",
            SearchEngine::Baidu => "百度",
        }
    }
}

/// 浏览器设置。
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// 浏览器外壳主题（Auto 跟随系统）。
    #[serde(default)]
    pub color_theme: ColorThemePreference,
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
            color_theme: ColorThemePreference::Auto,
        }
    }
}

impl BrowserSettings {
    /// 默认缩放下限。
    pub const DEFAULT_ZOOM_MIN: f32 = 0.25;
    /// 默认缩放上限。
    pub const DEFAULT_ZOOM_MAX: f32 = 5.0;
    /// 设置页 / 快捷键缩放步进。
    pub const DEFAULT_ZOOM_STEP: f32 = 0.1;

    /// 创建默认设置。
    pub fn new() -> Self {
        Self::default()
    }

    /// 按步进调整默认缩放并返回新值。
    pub fn adjust_default_zoom_by(&self, delta: f32) -> f32 {
        (self.default_zoom + delta).clamp(Self::DEFAULT_ZOOM_MIN, Self::DEFAULT_ZOOM_MAX)
    }

    /// 使用指定搜索引擎生成搜索 URL。
    pub fn search(&self, query: &str) -> String {
        self.search_engine.search_url(query)
    }

    /// 返回设置文件的默认路径。
    ///
    /// 遵循 XDG 规范：`~/.config/zeroweb/settings.json`
    pub fn default_config_path() -> PathBuf {
        let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from(".")).join("zeroweb");
        config_dir.join("settings.json")
    }

    /// 从 JSON 文件加载设置。
    ///
    /// 如果文件不存在或解析失败，返回默认设置。
    pub fn load(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// 将设置保存到 JSON 文件。
    ///
    /// 自动创建父目录。
    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {e}"))?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("Failed to serialize settings: {e}"))?;
        std::fs::write(path, json).map_err(|e| format!("Failed to write settings: {e}"))?;
        Ok(())
    }

    /// 从默认路径加载设置。
    pub fn load_default() -> Self {
        Self::load(&Self::default_config_path())
    }

    /// 保存到默认路径。
    pub fn save_default(&self) -> Result<(), String> {
        self.save(&Self::default_config_path())
    }
}
