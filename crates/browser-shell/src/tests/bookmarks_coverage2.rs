//! Bookmarks 覆盖率补充测试。

use crate::*;

#[test]
fn test_bookmarks_add_get() {
    let mut bm = Bookmarks::new();
    assert!(bm.is_empty());

    let id = bm.add("Google", "https://google.com", None);
    assert_eq!(bm.len(), 1);
    assert!(!bm.is_empty());

    let b = bm.get(id).unwrap();
    assert_eq!(b.title(), "Google");
    assert_eq!(b.url(), "https://google.com");
    assert!(b.folder_id().is_none());
}

#[test]
fn test_bookmarks_get_nonexistent() {
    let bm = Bookmarks::new();
    assert!(bm.get(BookmarkId(9999)).is_none());
}

#[test]
fn test_bookmarks_remove() {
    let mut bm = Bookmarks::new();
    let id = bm.add("Test", "https://example.com", None);
    assert!(bm.remove(id));
    assert!(bm.is_empty());
    assert!(bm.get(id).is_none());
}

#[test]
fn test_bookmarks_remove_nonexistent() {
    let mut bm = Bookmarks::new();
    assert!(!bm.remove(BookmarkId(999)));
}

#[test]
fn test_bookmarks_update_title() {
    let mut bm = Bookmarks::new();
    let id = bm.add("Old", "https://example.com", None);
    bm.update_title(id, "New");
    assert_eq!(bm.get(id).unwrap().title(), "New");
}

#[test]
fn test_bookmarks_update_title_nonexistent() {
    let mut bm = Bookmarks::new();
    bm.update_title(BookmarkId(999), "ignored");
    assert!(bm.is_empty());
}

#[test]
fn test_bookmarks_folders() {
    let mut bm = Bookmarks::new();

    // 创建文件夹
    let folder_id = bm.create_folder("Dev");
    let folder = bm.get_folder(folder_id).unwrap();
    assert_eq!(folder.name(), "Dev");

    // 添加书签到文件夹
    let id1 = bm.add("Rust", "https://rust-lang.org", Some(folder_id));
    let id2 = bm.add("MDN", "https://developer.mozilla.org", Some(folder_id));
    let id3 = bm.add("Google", "https://google.com", None);

    assert_eq!(bm.len(), 3);

    // 列出文件夹内书签
    let in_folder = bm.list_in_folder(folder_id);
    assert_eq!(in_folder.len(), 2);

    // 列出根级书签
    let root = bm.list_root();
    assert_eq!(root.len(), 1);

    // 列出所有文件夹
    assert_eq!(bm.folders().len(), 1);

    // 移除文件夹及其书签
    bm.remove_folder(folder_id);
    assert_eq!(bm.len(), 1);
    assert!(bm.get(id1).is_none());
    assert!(bm.get(id2).is_none());
    assert!(bm.get(id3).is_some());
    assert!(bm.folders().is_empty());
}

#[test]
fn test_bookmarks_get_folder_nonexistent() {
    let bm = Bookmarks::new();
    assert!(bm.get_folder(BookmarkId(999)).is_none());
}

#[test]
fn test_bookmarks_iter() {
    let mut bm = Bookmarks::new();
    bm.add("A", "https://a.com", None);
    bm.add("B", "https://b.com", None);
    bm.add("C", "https://c.com", None);
    let titles: Vec<&str> = bm.iter().map(|b| b.title()).collect();
    assert_eq!(titles, vec!["A", "B", "C"]);
}

#[test]
fn test_bookmarks_default() {
    let bm = Bookmarks::default();
    assert!(bm.is_empty());
}

#[test]
fn test_bookmark_id_accessors() {
    let mut bm = Bookmarks::new();
    let id = bm.add("Test", "https://example.com", None);
    let b = bm.get(id).unwrap();
    assert_eq!(b.id(), id);
    assert_eq!(b.url(), "https://example.com");
}

#[test]
fn test_bookmark_folder_id_accessor() {
    let mut bm = Bookmarks::new();
    let fid = bm.create_folder("F");
    let id = bm.add("InFolder", "https://x.com", Some(fid));
    let b = bm.get(id).unwrap();
    assert_eq!(b.folder_id(), Some(fid));
}

#[test]
fn test_folder_id_accessor() {
    let mut bm = Bookmarks::new();
    let fid = bm.create_folder("MyFolder");
    let f = bm.get_folder(fid).unwrap();
    assert_eq!(f.id(), fid);
}

#[test]
fn test_list_in_folder_empty() {
    let bm = Bookmarks::new();
    let result = bm.list_in_folder(BookmarkId(999));
    assert!(result.is_empty());
}

#[test]
fn test_list_root_empty() {
    let bm = Bookmarks::new();
    assert!(bm.list_root().is_empty());
}

#[test]
fn test_bookmarks_multiple_folders() {
    let mut bm = Bookmarks::new();
    let f1 = bm.create_folder("Folder1");
    let f2 = bm.create_folder("Folder2");
    bm.add("A", "a.com", Some(f1));
    bm.add("B", "b.com", Some(f2));
    bm.add("C", "c.com", Some(f1));

    assert_eq!(bm.list_in_folder(f1).len(), 2);
    assert_eq!(bm.list_in_folder(f2).len(), 1);
    assert_eq!(bm.folders().len(), 2);
}
