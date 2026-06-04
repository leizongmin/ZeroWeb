//! Autocomplete tests for uncovered paths.

use crate::*;

#[test]
fn test_autocomplete_empty_query() {
    let autocomplete = Autocomplete::new();
    let history = History::new();
    let bookmarks = Bookmarks::new();

    let suggestions = autocomplete.suggest("", &history, &bookmarks);
    assert_eq!(suggestions.len(), 0);

    let suggestions = autocomplete.suggest("   ", &history, &bookmarks);
    assert_eq!(suggestions.len(), 0);
}

#[test]
fn test_autocomplete_with_bookmark_match() {
    let autocomplete = Autocomplete::new();
    let history = History::new();
    let mut bookmarks = Bookmarks::new();

    bookmarks.add("Example", "https://example.com", None);
    let suggestions = autocomplete.suggest("ex", &history, &bookmarks);
    assert!(!suggestions.is_empty());
}

#[test]
fn test_autocomplete_with_history_match() {
    let autocomplete = Autocomplete::new();
    let mut history = History::new();
    let bookmarks = Bookmarks::new();

    history.record("https://example.com", "Example");
    let suggestions = autocomplete.suggest("ex", &history, &bookmarks);
    assert!(!suggestions.is_empty());
}

// ── 新增覆盖率测试 ──

#[test]
fn test_suggestion_accessors() {
    let s = Suggestion::new("https://example.com", "Example", SuggestionSource::Bookmark);
    assert_eq!(s.url(), "https://example.com");
    assert_eq!(s.title(), "Example");
    assert_eq!(s.source(), SuggestionSource::Bookmark);
}

#[test]
fn test_suggestion_equality() {
    let s1 = Suggestion::new("https://example.com", "Example", SuggestionSource::History);
    let s2 = Suggestion::new("https://example.com", "Example", SuggestionSource::History);
    assert_eq!(s1, s2);
}

#[test]
fn test_autocomplete_default() {
    let autocomplete = Autocomplete::default();
    let history = History::new();
    let bookmarks = Bookmarks::new();
    let suggestions = autocomplete.suggest("test", &history, &bookmarks);
    assert!(suggestions.is_empty());
}

#[test]
fn test_autocomplete_with_max_results() {
    let autocomplete = Autocomplete::new().with_max_results(2);
    let history = History::new();
    let mut bookmarks = Bookmarks::new();

    // 添加多个匹配项
    bookmarks.add("Example 1", "https://example1.com", None);
    bookmarks.add("Example 2", "https://example2.com", None);
    bookmarks.add("Example 3", "https://example3.com", None);

    let suggestions = autocomplete.suggest("example", &history, &bookmarks);
    assert_eq!(suggestions.len(), 2); // max_results = 2
}

#[test]
fn test_autocomplete_with_max_results_zero_clamped() {
    let autocomplete = Autocomplete::new().with_max_results(0);
    let mut history = History::new();
    let bookmarks = Bookmarks::new();
    history.record("https://example.com", "Example");
    // max_results 被限制为最小 1
    let suggestions = autocomplete.suggest("ex", &history, &bookmarks);
    assert_eq!(suggestions.len(), 1);
}

#[test]
fn test_autocomplete_bookmark_priority() {
    let autocomplete = Autocomplete::new();
    let mut history = History::new();
    let mut bookmarks = Bookmarks::new();

    // 同一个 URL 同时在历史和书签中
    history.record("https://example.com", "Example");
    bookmarks.add("Example", "https://example.com", None);

    let suggestions = autocomplete.suggest("ex", &history, &bookmarks);
    // 书签应优先（只出现一次）
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].source(), SuggestionSource::Bookmark);
}

#[test]
fn test_autocomplete_url_prefix_match() {
    let autocomplete = Autocomplete::new();
    let mut history = History::new();
    let bookmarks = Bookmarks::new();

    history.record("https://example.com/page1", "Page 1");
    history.record("https://other.com/page", "example title");

    let suggestions = autocomplete.suggest("https://ex", &history, &bookmarks);
    assert!(!suggestions.is_empty());
    // URL 前缀匹配应排在前面
    assert!(suggestions[0].url().starts_with("https://example.com"));
}

#[test]
fn test_autocomplete_title_contains_match() {
    let autocomplete = Autocomplete::new();
    let mut history = History::new();
    let bookmarks = Bookmarks::new();

    history.record("https://random.com", "A random page about rust");

    let suggestions = autocomplete.suggest("rust", &history, &bookmarks);
    assert!(!suggestions.is_empty());
}

#[test]
fn test_autocomplete_no_match() {
    let autocomplete = Autocomplete::new();
    let mut history = History::new();
    let bookmarks = Bookmarks::new();

    history.record("https://example.com", "Example");

    let suggestions = autocomplete.suggest("xyz", &history, &bookmarks);
    assert!(suggestions.is_empty());
}

#[test]
fn test_autocomplete_case_insensitive() {
    let autocomplete = Autocomplete::new();
    let mut history = History::new();
    let bookmarks = Bookmarks::new();

    history.record("https://Example.COM", "EXAMPLE Title");

    let suggestions = autocomplete.suggest("example", &history, &bookmarks);
    assert!(!suggestions.is_empty());

    let suggestions = autocomplete.suggest("TITLE", &history, &bookmarks);
    assert!(!suggestions.is_empty());
}
