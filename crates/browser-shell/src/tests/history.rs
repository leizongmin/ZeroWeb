//! History 单元测试。

use crate::*;

// ── History 测试 ──

#[test]
fn test_history_new() {
    let history = History::new();
    assert!(history.is_empty());
    assert_eq!(history.len(), 0);
}

#[test]
fn test_history_record() {
    let mut history = History::new();
    history.record("https://example.com", "Example");
    assert_eq!(history.len(), 1);

    let entry = history.iter().next().unwrap();
    assert_eq!(entry.url(), "https://example.com");
    assert_eq!(entry.title(), "Example");
}

#[test]
fn test_history_record_same_url_updates() {
    let mut history = History::new();
    history.record("https://example.com", "Title 1");
    history.record("https://example.com", "Title 2");
    assert_eq!(history.len(), 1);

    let entry = history.iter().next().unwrap();
    assert_eq!(entry.title(), "Title 2");
}

#[test]
fn test_history_search() {
    let mut history = History::new();
    history.record("https://example.com", "Example");
    history.record("https://github.com", "GitHub");
    history.record("https://developer.mozilla.org", "MDN");

    let results: Vec<_> = history.search("git").collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url(), "https://github.com");
}

#[test]
fn test_history_search_by_title() {
    let mut history = History::new();
    history.record("https://example.com", "Example Domain");
    history.record("https://other.com", "Other Site");

    let results: Vec<_> = history.search("Domain").collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url(), "https://example.com");
}

#[test]
fn test_history_search_no_match() {
    let mut history = History::new();
    history.record("https://example.com", "Example");
    let results: Vec<_> = history.search("nonexistent").collect();
    assert!(results.is_empty());
}

#[test]
fn test_history_clear() {
    let mut history = History::new();
    history.record("https://example.com", "Example");
    history.record("https://github.com", "GitHub");
    history.clear();
    assert!(history.is_empty());
}

#[test]
fn test_history_multiple_entries() {
    let mut history = History::new();
    for i in 0..10 {
        history.record(&format!("https://site{i}.com"), &format!("Site {i}"));
    }
    assert_eq!(history.len(), 10);
}

// ── History 边界测试 ──

#[test]
fn test_history_search_empty_query() {
    let mut history = History::new();
    history.record("https://example.com", "Example");
    let results: Vec<_> = history.search("").collect();
    assert_eq!(results.len(), 1, "Empty query should match all");
}

#[test]
fn test_history_search_case_insensitive() {
    let mut history = History::new();
    history.record("https://Example.COM", "Example Page");
    let results: Vec<_> = history.search("example").collect();
    assert_eq!(results.len(), 1);
    let results2: Vec<_> = history.search("PAGE").collect();
    assert_eq!(results2.len(), 1);
}

#[test]
fn test_history_record_moves_to_front() {
    let mut history = History::new();
    history.record("https://a.com", "A");
    history.record("https://b.com", "B");
    history.record("https://a.com", "A Updated");
    let first = history.iter().next().unwrap();
    assert_eq!(first.url(), "https://a.com");
    assert_eq!(first.title(), "A Updated");
    assert_eq!(history.len(), 2);
}

#[test]
fn test_history_search_partial_match() {
    let mut history = History::new();
    history.record("https://docs.rs/serde", "Serde Docs");
    let results: Vec<_> = history.search("docs").collect();
    assert_eq!(results.len(), 1);
}

#[test]
fn test_history_clear_empty() {
    let mut history = History::new();
    history.clear();
    assert!(history.is_empty());
}

#[test]
fn test_history_iter_order() {
    let mut history = History::new();
    history.record("https://a.com", "A");
    history.record("https://b.com", "B");
    history.record("https://c.com", "C");
    let urls: Vec<_> = history.iter().map(|e| e.url()).collect();
    assert_eq!(urls, vec!["https://c.com", "https://b.com", "https://a.com"]);
}

#[test]
fn test_history_special_characters() {
    let mut history = History::new();
    history.record("https://example.com/search?q=rust%20lang&lang=ja", "Rust 検索");
    let results: Vec<_> = history.search("rust").collect();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title(), "Rust 検索");
}

#[test]
/// 测试 History::clear 后搜索返回空。
fn test_history_clear_then_search_empty() {
    let mut hist = History::new();
    hist.record("https://example.com", "Example");
    hist.record("https://test.com", "Test");
    hist.clear();
    assert!(hist.search("example").next().is_none(), "清除后搜索应返回空");
    assert!(hist.search("test").next().is_none(), "清除后搜索应返回空");
}
