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
    // 无匹配数据时，非 URL 输入返回单条搜索建议
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].source(), SuggestionSource::Search);
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
    // max_results=2：1 条历史/书签 + 1 条搜索建议
    assert_eq!(suggestions.len(), 2);
}

#[test]
fn test_autocomplete_with_max_results_zero_clamped() {
    let autocomplete = Autocomplete::new().with_max_results(0);
    let mut history = History::new();
    let bookmarks = Bookmarks::new();
    history.record("https://example.com", "Example");
    // max_results 被限制为最小 1：history_cap=1，1 历史 + 1 搜索建议 = 2
    let suggestions = autocomplete.suggest("ex", &history, &bookmarks);
    assert_eq!(suggestions.len(), 2);
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
    // 顶部搜索建议 + 1 条书签（书签优先于同 URL 历史）
    assert_eq!(suggestions.len(), 2);
    assert_eq!(suggestions[0].source(), SuggestionSource::Search);
    assert_eq!(suggestions[1].source(), SuggestionSource::Bookmark);
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
    // 输入像 URL，不插入搜索建议；URL 前缀匹配排第一
    assert_eq!(suggestions[0].source(), SuggestionSource::History);
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
fn test_autocomplete_no_match_yields_search() {
    let autocomplete = Autocomplete::new();
    let mut history = History::new();
    let bookmarks = Bookmarks::new();

    history.record("https://example.com", "Example");

    let suggestions = autocomplete.suggest("xyz", &history, &bookmarks);
    // 无匹配 + 非 URL 输入 → 单条搜索建议
    assert_eq!(suggestions.len(), 1);
    assert_eq!(suggestions[0].source(), SuggestionSource::Search);
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

// ── 搜索建议行为测试 ──

#[test]
fn search_suggestion_inserted_for_plain_word() {
    let ac = Autocomplete::new();
    let history = History::new();
    let bookmarks = Bookmarks::new();
    // 纯词、无点号 → 不像 URL → 顶部应有搜索建议
    let results = ac.suggest("hello world", &history, &bookmarks);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].source(), SuggestionSource::Search);
    assert_eq!(results[0].title(), "hello world");
}

#[test]
fn search_suggestion_skipped_for_url_like_input() {
    let ac = Autocomplete::new();
    let history = History::new();
    let bookmarks = Bookmarks::new();
    // 形如 host.tld → 像 URL → 不插入搜索建议
    let results = ac.suggest("example.com", &history, &bookmarks);
    assert!(results.is_empty(), "URL 形式输入且无匹配应返回空");
}

#[test]
fn search_suggestion_url_equals_query_for_normalize_handoff() {
    let ac = Autocomplete::new();
    let history = History::new();
    let bookmarks = Bookmarks::new();
    let results = ac.suggest("rust async", &history, &bookmarks);
    let s = &results[0];
    // url 与 title 一致，交给 normalize_url 转换为搜索引擎 URL
    assert_eq!(s.url(), s.title());
}

// ── R3366：跨来源排序行为锁定 ──

#[test]
/// R3366：书签 +100 加分整体优先于历史项，即便历史项是 URL 前缀精确匹配。
///
/// 文档（autocomplete.rs suggest docstring）描述的单来源 4 级分数仅作用于同一来源内部；
/// 书签的 +100 跨来源加分是独立提升，使书签条目整体排在历史项之前。
/// 此测试锁定该跨来源语义，防止未来误把「URL 前缀(40) 应高于 书签标题包含(10+100)」
/// 当作 bug 而改动评分逻辑（行为是设计意图——书签是用户主动收藏，应优先）。
fn bookmark_bonus_outranks_history_url_prefix_r3366() {
    let ac = Autocomplete::new().with_max_results(10);
    let mut history = History::new();
    let mut bookmarks = Bookmarks::new();
    // 历史：query "rust" 是该 URL 的精确前缀匹配（score 40）
    history.record("rust-lang.org", "Other Title");
    // 书签：query "rust" 仅在标题中包含匹配（raw score 10，+100 加分 = 110）
    bookmarks.add("a rust example", "https://zzz-bm.com", None);

    let results = ac.suggest("rust", &history, &bookmarks);
    // "rust" 非纯 URL 形式（无点号 TLD）→ 顶部插入搜索建议，其后为书签，再后为历史
    assert_eq!(results.len(), 3, "搜索建议 + 书签 + 历史");
    assert_eq!(results[0].source(), SuggestionSource::Search);
    assert_eq!(
        results[1].source(),
        SuggestionSource::Bookmark,
        "书签 +100 加分应排在历史 URL 前缀匹配之前"
    );
    assert_eq!(results[1].url(), "https://zzz-bm.com");
    assert_eq!(results[2].source(), SuggestionSource::History);
    assert_eq!(results[2].url(), "rust-lang.org");
}

#[test]
/// R3366：同 URL 同时存在于书签与历史时，去重后只保留书签来源。
fn same_url_dedup_keeps_bookmark_source_r3366() {
    let ac = Autocomplete::new().with_max_results(10);
    let mut history = History::new();
    let mut bookmarks = Bookmarks::new();
    history.record("https://example.com", "Example Page");
    bookmarks.add("Example", "https://example.com", None);

    let results = ac.suggest("ex", &history, &bookmarks);
    // 搜索建议 + 单条去重后的书签
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].source(), SuggestionSource::Search);
    assert_eq!(results[1].source(), SuggestionSource::Bookmark);
}
