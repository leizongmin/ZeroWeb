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
