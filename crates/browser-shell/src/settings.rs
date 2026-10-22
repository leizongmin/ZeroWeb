//! 浏览器设置 — 用户偏好和配置管理。

use std::collections::HashMap;
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

/// 打印页面方向偏好（T61 打印预览设置；CSS Paged Media `size` 方向分量）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrintOrientationPreference {
    /// 纵向（A4 793.7 × 1122.5 px @96dpi）。
    #[default]
    Portrait,
    /// 横向（A4 宽高互换）。
    Landscape,
}

/// 默认打印份数（settings.json 旧版本缺字段时的 serde 兜底）。
fn default_print_copies() -> u32 {
    1
}

/// 默认打印页边距 mm（0 = 与引擎 `@page` 缺省链一致，升级零行为变更）。
fn default_print_margin_mm() -> f32 {
    0.0
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
    /// 按站点记住的页面缩放（origin → zoom；Chrome per-site zoom 语义）。
    #[serde(default)]
    pub zoom_by_origin: HashMap<String, f32>,
    /// 打印页面方向（T61 打印预览设置面板）。
    #[serde(default)]
    pub print_orientation: PrintOrientationPreference,
    /// 打印份数（1–99；仅随打印作业语义，不影响预览渲染——Chrome 同）。
    #[serde(default = "default_print_copies")]
    pub print_copies: u32,
    /// 打印页边距（毫米，0–50；`@page { margin }` 显式声明时被页面覆盖）。
    #[serde(default = "default_print_margin_mm")]
    pub print_margin_mm: f32,
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
            zoom_by_origin: HashMap::new(),
            print_orientation: PrintOrientationPreference::Portrait,
            print_copies: 1,
            print_margin_mm: 0.0,
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
    /// 打印份数下限（Chrome print dialog 1–99）。
    pub const PRINT_COPIES_MIN: u32 = 1;
    /// 打印份数上限。
    pub const PRINT_COPIES_MAX: u32 = 99;
    /// 打印页边距下限（毫米）。
    pub const PRINT_MARGIN_MIN_MM: f32 = 0.0;
    /// 打印页边距上限（毫米）。
    pub const PRINT_MARGIN_MAX_MM: f32 = 50.0;
    /// 打印页边距步进（毫米）。
    pub const PRINT_MARGIN_STEP_MM: f32 = 5.0;

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
    /// 遵循 XDG 规范：`~/.config/zeroweb/settings.json`；`ZERO_CONFIG_DIR`
    /// （`--profile`）优先。
    pub fn default_config_path() -> PathBuf {
        crate::config_base_dir().join("zeroweb").join("settings.json")
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

#[cfg(test)]
mod print_settings_tests {
    use super::*;

    /// T61：settings.json 旧版本缺打印字段时 serde 兜底默认值（升级零行为变更：
    /// 纵向 / 1 份 / 0 边距，与引擎 `@page` 缺省链一致）。
    #[test]
    fn t61_legacy_settings_json_defaults() {
        let legacy = r#"{
            "search_engine": "Google",
            "home_url": "https://example.com",
            "show_bookmarks_bar": true,
            "javascript_enabled": true,
            "cookies_enabled": true,
            "block_third_party_cookies": true,
            "do_not_track": false,
            "default_zoom": 1.0,
            "download_directory": ""
        }"#;
        let settings: BrowserSettings = serde_json::from_str(legacy).expect("legacy json should parse");
        assert_eq!(settings.print_orientation, PrintOrientationPreference::Portrait);
        assert_eq!(settings.print_copies, 1);
        assert!((settings.print_margin_mm - 0.0).abs() <= f32::EPSILON);
    }

    /// T61：打印设置字段的 serde 往返（snake_case 方向枚举）。
    #[test]
    fn t61_print_settings_roundtrip() {
        let settings = BrowserSettings {
            print_orientation: PrintOrientationPreference::Landscape,
            print_copies: 3,
            print_margin_mm: 15.0,
            ..BrowserSettings::default()
        };
        let json = serde_json::to_string(&settings).expect("serialize");
        let back: BrowserSettings = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.print_orientation, PrintOrientationPreference::Landscape);
        assert_eq!(back.print_copies, 3);
        assert!((back.print_margin_mm - 15.0).abs() <= f32::EPSILON);
        assert!(
            json.contains("\"landscape\""),
            "orientation 应为 snake_case，json={json}"
        );
    }
}
