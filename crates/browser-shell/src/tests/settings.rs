//! BrowserSettings 单元测试。

use crate::*;

// ── BrowserSettings 测试 ──

#[test]
fn test_settings_default() {
    let settings = BrowserSettings::new();
    assert_eq!(settings.search_engine, SearchEngine::Google);
    assert_eq!(settings.home_url, "https://example.com");
    assert!(settings.show_bookmarks_bar);
    assert!(settings.javascript_enabled);
    assert!(settings.cookies_enabled);
    assert!(settings.block_third_party_cookies);
    assert!(!settings.do_not_track);
    assert!((settings.default_zoom - 1.0).abs() < 0.01);
    assert!(settings.download_directory.is_empty());
}

#[test]
fn test_settings_search_google() {
    let settings = BrowserSettings::new();
    let url = settings.search("rust lang");
    assert!(url.contains("google.com"));
    assert!(url.contains("rust+lang"));
}

#[test]
fn test_settings_search_baidu() {
    let mut settings = BrowserSettings::new();
    settings.search_engine = SearchEngine::Baidu;
    let url = settings.search("rust 语言");
    assert!(url.contains("baidu.com"));
    assert!(url.contains("rust+"));
}

#[test]
fn test_settings_search_duckduckgo() {
    let mut settings = BrowserSettings::new();
    settings.search_engine = SearchEngine::DuckDuckGo;
    let url = settings.search("privacy");
    assert!(url.contains("duckduckgo.com"));
}

#[test]
fn test_settings_search_bing() {
    let mut settings = BrowserSettings::new();
    settings.search_engine = SearchEngine::Bing;
    let url = settings.search("test");
    assert!(url.contains("bing.com"));
}

// ── 设置验证测试 ──

#[test]
fn test_settings_default_values() {
    let settings = BrowserSettings::new();
    assert_eq!(settings.home_url, "https://example.com");
    assert!(settings.javascript_enabled);
    assert!(settings.cookies_enabled);
    assert!(!settings.do_not_track);
    assert!((settings.default_zoom - 1.0).abs() < 0.01);
}

#[test]
fn test_settings_search_engine_urls() {
    let settings = BrowserSettings::new();
    let url = settings.search("rust programming");
    assert!(
        url.contains("rust+programming") || url.contains("rust%20programming"),
        "搜索 URL 应包含查询词"
    );
}

#[test]
fn test_settings_custom_home_url() {
    let mut settings = BrowserSettings::new();
    settings.home_url = "https://zeroweb.dev".to_string();
    assert_eq!(settings.home_url, "https://zeroweb.dev");
}

// ── Settings 边界 ──

#[test]
/// 测试 SearchEngine::search_url() 对特殊字符（+, #, &）的处理。
fn test_search_url_special_chars() {
    let engine = SearchEngine::Google;
    let url = engine.search_url("a&b#c++d");
    assert!(url.contains("a&b#c++d"), "特殊字符应直接传递");
}

#[test]
/// 测试 BrowserSettings 默认搜索引擎 URL 格式。
fn test_browser_settings_search_url_format() {
    let settings = BrowserSettings::default();
    let url = settings.search_engine.search_url("test query");
    assert!(url.contains("test"), "搜索 URL 应包含查询词");
}

// ── 设置持久化测试 ──

#[test]
fn test_settings_serialize_deserialize() {
    let mut settings = BrowserSettings::new();
    settings.search_engine = SearchEngine::DuckDuckGo;
    settings.home_url = "https://custom.home".to_string();
    settings.javascript_enabled = false;
    settings.default_zoom = 1.5;

    let json = serde_json::to_string(&settings).unwrap();
    let loaded: BrowserSettings = serde_json::from_str(&json).unwrap();

    assert_eq!(loaded.search_engine, SearchEngine::DuckDuckGo);
    assert_eq!(loaded.home_url, "https://custom.home");
    assert!(!loaded.javascript_enabled);
    assert!((loaded.default_zoom - 1.5).abs() < 0.01);
}

#[test]
fn test_settings_save_and_load() {
    let dir = std::env::temp_dir().join("zeroweb-test-settings");
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("settings.json");

    let mut settings = BrowserSettings::new();
    settings.search_engine = SearchEngine::Bing;
    settings.home_url = "https://test.example.com".to_string();
    settings.do_not_track = true;

    // 保存
    settings.save(&path).unwrap();
    assert!(path.exists(), "settings file should be created");

    // 加载
    let loaded = BrowserSettings::load(&path);
    assert_eq!(loaded.search_engine, SearchEngine::Bing);
    assert_eq!(loaded.home_url, "https://test.example.com");
    assert!(loaded.do_not_track);

    // 清理
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_settings_load_missing_file_returns_default() {
    let path = std::env::temp_dir()
        .join("zeroweb-nonexistent-abc123")
        .join("missing.json");
    let loaded = BrowserSettings::load(&path);
    assert_eq!(loaded.search_engine, SearchEngine::Google);
    assert_eq!(loaded.home_url, "https://example.com");
}

#[test]
fn test_settings_load_invalid_json_returns_default() {
    let dir = std::env::temp_dir().join("zeroweb-test-invalid-json");
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("settings.json");
    std::fs::write(&path, "not valid json{{{").unwrap();

    let loaded = BrowserSettings::load(&path);
    assert_eq!(loaded.search_engine, SearchEngine::Google);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_settings_save_creates_parent_dirs() {
    let dir = std::env::temp_dir().join("zeroweb-test-nested").join("a").join("b");
    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("zeroweb-test-nested"));
    let path = dir.join("settings.json");

    let settings = BrowserSettings::new();
    settings.save(&path).unwrap();
    assert!(path.exists());

    let _ = std::fs::remove_dir_all(std::env::temp_dir().join("zeroweb-test-nested"));
}

#[test]
fn test_settings_default_config_path() {
    let path = BrowserSettings::default_config_path();
    assert!(path.to_string_lossy().contains("zeroweb"));
    assert!(path.to_string_lossy().ends_with("settings.json"));
}

#[test]
fn test_settings_roundtrip_preserves_all_fields() {
    let mut settings = BrowserSettings::new();
    settings.search_engine = SearchEngine::Baidu;
    settings.home_url = "https://home.test".to_string();
    settings.show_bookmarks_bar = false;
    settings.javascript_enabled = false;
    settings.cookies_enabled = false;
    settings.block_third_party_cookies = false;
    settings.do_not_track = true;
    settings.default_zoom = 2.0;
    settings.download_directory = "/tmp/downloads".to_string();

    let dir = std::env::temp_dir().join("zeroweb-test-roundtrip");
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("settings.json");

    settings.save(&path).unwrap();
    let loaded = BrowserSettings::load(&path);

    assert_eq!(loaded.search_engine, SearchEngine::Baidu);
    assert_eq!(loaded.home_url, "https://home.test");
    assert!(!loaded.show_bookmarks_bar);
    assert!(!loaded.javascript_enabled);
    assert!(!loaded.cookies_enabled);
    assert!(!loaded.block_third_party_cookies);
    assert!(loaded.do_not_track);
    assert!((loaded.default_zoom - 2.0).abs() < 0.01);
    assert_eq!(loaded.download_directory, "/tmp/downloads");

    let _ = std::fs::remove_dir_all(&dir);
}
