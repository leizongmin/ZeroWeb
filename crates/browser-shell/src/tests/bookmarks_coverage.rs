//! Bookmarks tests for uncovered paths.

use zero_browser_shell::{Bookmark, BookmarkFolder, BookmarkId, Bookmarks};

#[test]
fn test_bookmark_remove_nonexistent() {
    // Test lines 32-34 - Remove non-existent bookmark
    let mut bookmarks = Bookmarks::new();

    // Initially empty
    assert_eq!(bookmarks.len(), 0);

    // Remove non-existent bookmark
    let removed = bookmarks.remove(BookmarkId(999));
    assert!(!removed);

    // Still empty
    assert_eq!(bookmarks.len(), 0);
}
