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
