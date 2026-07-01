//! Bookmarks 单元测试。

use crate::*;

// ── Bookmarks 测试 ──

#[test]
fn test_bookmarks_new() {
    let bm = Bookmarks::new();
    assert!(bm.is_empty());
    assert_eq!(bm.len(), 0);
}

#[test]
fn test_bookmarks_add() {
    let mut bm = Bookmarks::new();
    let id = bm.add("Example", "https://example.com", None);
    assert!(!bm.is_empty());
    assert_eq!(bm.len(), 1);

    let bookmark = bm.get(id).unwrap();
    assert_eq!(bookmark.title(), "Example");
    assert_eq!(bookmark.url(), "https://example.com");
}

#[test]
fn test_bookmarks_add_to_folder() {
    let mut bm = Bookmarks::new();
    let folder_id = bm.create_folder("News");
    let id = bm.add("CNN", "https://cnn.com", Some(folder_id));
    let bookmark = bm.get(id).unwrap();
    assert_eq!(bookmark.folder_id(), Some(folder_id));
}

#[test]
fn test_bookmarks_remove() {
    let mut bm = Bookmarks::new();
    let id = bm.add("Example", "https://example.com", None);
    assert!(bm.remove(id));
    assert!(bm.is_empty());
}

#[test]
fn test_bookmarks_remove_nonexistent() {
    let mut bm = Bookmarks::new();
    assert!(!bm.remove(BookmarkId(99999)));
}

#[test]
fn test_bookmarks_create_folder() {
    let mut bm = Bookmarks::new();
    let id = bm.create_folder("Development");
    let folder = bm.get_folder(id).unwrap();
    assert_eq!(folder.name(), "Development");
}

#[test]
fn test_bookmarks_remove_folder() {
    let mut bm = Bookmarks::new();
    let folder_id = bm.create_folder("News");
    bm.add("CNN", "https://cnn.com", Some(folder_id));
    bm.add("BBC", "https://bbc.com", Some(folder_id));
    assert_eq!(bm.len(), 2);

    bm.remove_folder(folder_id);
    assert_eq!(bm.len(), 0);
}

#[test]
fn test_bookmarks_update_title() {
    let mut bm = Bookmarks::new();
    let id = bm.add("Old Title", "https://example.com", None);
    bm.update_title(id, "New Title");
    assert_eq!(bm.get(id).unwrap().title(), "New Title");
}

#[test]
fn test_bookmarks_list_root() {
    let mut bm = Bookmarks::new();
    bm.add("A", "https://a.com", None);
    bm.add("B", "https://b.com", None);
    let folder_id = bm.create_folder("Folder");
    bm.add("C", "https://c.com", Some(folder_id));

    let root = bm.list_root();
    assert_eq!(root.len(), 2); // Only root-level bookmarks
}

#[test]
fn test_bookmarks_list_in_folder() {
    let mut bm = Bookmarks::new();
    let folder_id = bm.create_folder("Dev");
    bm.add("GitHub", "https://github.com", Some(folder_id));
    bm.add("MDN", "https://developer.mozilla.org", Some(folder_id));
    bm.add("Other", "https://other.com", None);

    let in_folder = bm.list_in_folder(folder_id);
    assert_eq!(in_folder.len(), 2);
}

// ── Bookmarks 边界测试 ──

#[test]
fn test_bookmarks_update_nonexistent() {
    let mut bm = Bookmarks::new();
    bm.update_title(BookmarkId(99999), "New Title");
    // Should not panic
    assert!(bm.is_empty());
}

#[test]
fn test_bookmarks_get_nonexistent() {
    let bm = Bookmarks::new();
    assert!(bm.get(BookmarkId(99999)).is_none());
    assert!(bm.get_folder(BookmarkId(99999)).is_none());
}

#[test]
fn test_bookmarks_empty_folder() {
    let mut bm = Bookmarks::new();
    let folder_id = bm.create_folder("Empty");
    let in_folder = bm.list_in_folder(folder_id);
    assert!(in_folder.is_empty());
}

#[test]
fn test_bookmarks_remove_folder_nonexistent() {
    let mut bm = Bookmarks::new();
    bm.add("A", "https://a.com", None);
    bm.remove_folder(BookmarkId(99999));
    assert_eq!(bm.len(), 1, "Should not remove anything");
}

#[test]
fn test_bookmarks_multiple_folders() {
    let mut bm = Bookmarks::new();
    let f1 = bm.create_folder("Folder 1");
    let f2 = bm.create_folder("Folder 2");
    bm.add("A", "https://a.com", Some(f1));
    bm.add("B", "https://b.com", Some(f2));
    bm.add("C", "https://c.com", Some(f1));
    assert_eq!(bm.list_in_folder(f1).len(), 2);
    assert_eq!(bm.list_in_folder(f2).len(), 1);
    assert_eq!(bm.list_root().len(), 0);
    assert_eq!(bm.folders().len(), 2);
}

#[test]
fn test_bookmarks_iter() {
    let mut bm = Bookmarks::new();
    bm.add("A", "https://a.com", None);
    bm.add("B", "https://b.com", None);
    let all: Vec<_> = bm.iter().collect();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_bookmarks_update_url_via_title() {
    let mut bm = Bookmarks::new();
    let id = bm.add("Old", "https://example.com", None);
    bm.update_title(id, "New");
    assert_eq!(bm.get(id).unwrap().title(), "New");
    assert_eq!(bm.get(id).unwrap().url(), "https://example.com");
}

#[test]
fn test_bookmarks_add_duplicate_url() {
    let mut bm = Bookmarks::new();
    let _id1 = bm.add("First", "https://example.com", None);
    let _id2 = bm.add("Second", "https://example.com", None);
    // Both should exist (bookmarks allow duplicates)
    assert_eq!(bm.len(), 2);
}

#[test]
fn test_bookmarks_remove_one_from_folder() {
    let mut bm = Bookmarks::new();
    let folder_id = bm.create_folder("Dev");
    let id1 = bm.add("GitHub", "https://github.com", Some(folder_id));
    let _id2 = bm.add("MDN", "https://developer.mozilla.org", Some(folder_id));
    assert_eq!(bm.list_in_folder(folder_id).len(), 2);
    bm.remove(id1);
    assert_eq!(bm.list_in_folder(folder_id).len(), 1);
    assert_eq!(bm.get(id1), None);
}

// ── 书签嵌套文件夹测试 ──

#[test]
fn test_bookmarks_remove_folder_cascades() {
    let mut bm = Bookmarks::new();
    let folder = bm.create_folder("News");
    bm.add("Hacker News", "https://news.ycombinator.com", Some(folder));
    bm.remove_folder(folder);
    assert_eq!(bm.list_in_folder(folder).len(), 0);
}

#[test]
fn test_bookmarks_multiple_folders_separate() {
    let mut bm = Bookmarks::new();
    let f1 = bm.create_folder("News");
    let f2 = bm.create_folder("Tech");
    bm.add("HN", "https://news.ycombinator.com", Some(f1));
    bm.add("ZeroWeb", "https://zeroweb.dev", Some(f2));
    assert_eq!(bm.list_in_folder(f1).len(), 1);
    assert_eq!(bm.list_in_folder(f2).len(), 1);
    // 移除一个文件夹不影响另一个
    bm.remove_folder(f1);
    assert_eq!(bm.list_in_folder(f2).len(), 1);
}

#[test]
/// 测试 Bookmarks::iter 按 URL 过滤。
fn test_bookmarks_iter_filter_by_url() {
    let mut bm = Bookmarks::new();
    bm.add("Example", "https://example.com", None);
    bm.add("Example 2", "https://example.com", None);
    bm.add("Other", "https://other.com", None);
    let count = bm.iter().filter(|b| b.url() == "https://example.com").count();
    assert_eq!(count, 2, "应能按 URL 过滤书签");
}

#[test]
fn test_bookmarks_default() {
    let bm = Bookmarks::default();
    assert!(bm.is_empty());
    assert_eq!(bm.len(), 0);
    assert!(bm.folders().is_empty());
}

#[test]
fn test_bookmarks_save_load_roundtrip() {
    let dir = std::env::temp_dir().join(format!("zeroweb_test_bookmarks-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("bookmarks.json");

    let mut bm = Bookmarks::new();
    let folder_id = bm.create_folder("Work");
    bm.add("Example", "https://example.com", None);
    bm.add("Docs", "https://docs.example.com", Some(folder_id));
    bm.save(&path).expect("save should succeed");

    let mut loaded = Bookmarks::load(&path);
    assert_eq!(loaded.len(), 2);
    assert_eq!(loaded.folders().len(), 1);
    assert_eq!(loaded.list_root().len(), 1);
    assert_eq!(loaded.list_root()[0].title(), "Example");
    assert_eq!(loaded.list_in_folder(folder_id).len(), 1);
    assert_eq!(loaded.list_in_folder(folder_id)[0].title(), "Docs");

    let next_id = loaded.add("After Load", "https://after.example.com", None);
    assert!(next_id.0 > folder_id.0);

    let _ = std::fs::remove_dir_all(&dir);
}
