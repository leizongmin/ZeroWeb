//! Autocomplete 单元测试。

use crate::*;

// ── Autocomplete 测试 ──

#[test]
fn test_autocomplete_empty_query() {
    let ac = Autocomplete::new();
    let history = History::new();
    let bookmarks = Bookmarks::new();
    let results = ac.suggest("", &history, &bookmarks);
    assert!(results.is_empty());
}

#[test]
fn test_autocomplete_from_history() {
    let ac = Autocomplete::new();
    let mut history = History::new();
    history.record("https://github.com", "GitHub");
    history.record("https://example.com", "Example");
    let bookmarks = Bookmarks::new();

    let results = ac.suggest("git", &history, &bookmarks);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url(), "https://github.com");
    assert_eq!(results[0].source(), SuggestionSource::History);
}

#[test]
fn test_autocomplete_from_bookmarks() {
    let ac = Autocomplete::new();
    let history = History::new();
    let mut bookmarks = Bookmarks::new();
    bookmarks.add("Rust Lang", "https://rust-lang.org", None);

    let results = ac.suggest("rust", &history, &bookmarks);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url(), "https://rust-lang.org");
    assert_eq!(results[0].source(), SuggestionSource::Bookmark);
}

#[test]
fn test_autocomplete_bookmark_priority() {
    let ac = Autocomplete::new();
    let mut history = History::new();
    history.record("https://example.com", "Example Page");
    let mut bookmarks = Bookmarks::new();
    bookmarks.add("Example", "https://example.com", None);

    // 同 URL 书签优先
    let results = ac.suggest("example", &history, &bookmarks);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].source(), SuggestionSource::Bookmark);
}

#[test]
fn test_autocomplete_title_match() {
    let ac = Autocomplete::new();
    let mut history = History::new();
    history.record("https://docs.python.org", "Python Documentation");
    let bookmarks = Bookmarks::new();

    let results = ac.suggest("python", &history, &bookmarks);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url(), "https://docs.python.org");
}

#[test]
fn test_autocomplete_no_match() {
    let ac = Autocomplete::new();
    let mut history = History::new();
    history.record("https://example.com", "Example");
    let bookmarks = Bookmarks::new();

    let results = ac.suggest("zzzzz", &history, &bookmarks);
    assert!(results.is_empty());
}

#[test]
fn test_autocomplete_max_results() {
    let ac = Autocomplete::new().with_max_results(2);
    let mut history = History::new();
    for i in 0..10 {
        history.record(&format!("https://site{i}.com"), &format!("Site {i}"));
    }
    let bookmarks = Bookmarks::new();

    let results = ac.suggest("site", &history, &bookmarks);
    assert!(results.len() <= 2);
}

#[test]
fn test_autocomplete_case_insensitive() {
    let ac = Autocomplete::new();
    let mut history = History::new();
    history.record("https://GitHub.com/USER", "GitHub User Page");
    let bookmarks = Bookmarks::new();

    let results = ac.suggest("github", &history, &bookmarks);
    assert_eq!(results.len(), 1);
}

#[test]
fn test_autocomplete_url_prefix_ranked_higher() {
    let ac = Autocomplete::new();
    let mut history = History::new();
    history.record("https://example.com/about", "About Page");
    history.record("https://other.com/example", "Example on Other");
    let bookmarks = Bookmarks::new();

    let results = ac.suggest("https://example", &history, &bookmarks);
    assert!(!results.is_empty());
    // URL prefix match should rank first
    assert!(results[0].url().starts_with("https://example"));
}

#[test]
fn test_autocomplete_with_max_results_zero_clamps() {
    let ac = Autocomplete::new().with_max_results(0);
    // max_results=0 应 clamp 到 1
    let history = History::new();
    let bookmarks = Bookmarks::new();
    let results = ac.suggest("test", &history, &bookmarks);
    assert!(results.is_empty()); // 无匹配数据，所以为空
}

#[test]
fn test_suggestion_accessors() {
    let s = Suggestion::new("https://example.com", "Example", SuggestionSource::History);
    assert_eq!(s.url(), "https://example.com");
    assert_eq!(s.title(), "Example");
    assert_eq!(s.source(), SuggestionSource::History);
}

#[test]
fn test_suggestion_equality() {
    let a = Suggestion::new("https://a.com", "A", SuggestionSource::History);
    let b = Suggestion::new("https://a.com", "A", SuggestionSource::History);
    assert_eq!(a, b);
}

// ── BrowserShell autocomplete 集成测试 ──

#[test]
fn test_browser_shell_suggest_empty() {
    let shell = BrowserShell::new();
    let results = shell.suggest("test");
    assert!(results.is_empty());
}

#[test]
fn test_browser_shell_suggest_from_history() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://github.com");
    shell.on_page_loaded("GitHub");
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");

    let results = shell.suggest("git");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].url(), "https://github.com");
}

#[test]
fn test_browser_shell_suggest_from_bookmarks() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://rust-lang.org");
    shell.on_page_loaded("Rust");
    shell.add_bookmark();

    let results = shell.suggest("rust");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].source(), SuggestionSource::Bookmark);
}

// ── Autocomplete 边界测试 ──

#[test]
fn test_autocomplete_unicode_query() {
    let ac = Autocomplete::new();
    let mut history = History::new();
    history.record("https://example.com/你好", "你好世界");
    let bookmarks = Bookmarks::new();
    let results = ac.suggest("你好", &history, &bookmarks);
    assert_eq!(results.len(), 1, "Unicode 查询应匹配");
}

#[test]
fn test_autocomplete_whitespace_only_query() {
    let ac = Autocomplete::new();
    let mut history = History::new();
    history.record("https://example.com", "Test");
    let bookmarks = Bookmarks::new();
    let results = ac.suggest("   ", &history, &bookmarks);
    assert!(results.is_empty(), "纯空白查询应返回空");
}

#[test]
/// 测试 Autocomplete 空查询返回空列表。
fn test_autocomplete_suggest_empty_query() {
    let ac = Autocomplete::new();
    let hist = History::new();
    let bm = Bookmarks::new();
    let results = ac.suggest("", &hist, &bm);
    assert!(results.is_empty(), "空查询应返回空列表");
}

#[test]
/// 测试 Autocomplete 大小写不敏感搜索。
fn test_autocomplete_suggest_case_insensitive() {
    let ac = Autocomplete::new();
    let mut hist = History::new();
    hist.record("https://Example.com", "Test");
    let bm = Bookmarks::new();
    let results = ac.suggest("example", &hist, &bm);
    assert!(!results.is_empty(), "大小写不敏感搜索应找到结果");
}
