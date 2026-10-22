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

// T40 书签管理页：search（搜索框过滤）与 update（编辑表单提交）。
#[test]
fn test_bookmarks_search_matches_title_or_url_case_insensitive() {
    let mut bm = Bookmarks::new();
    bm.add("Example", "https://example.com/a", None);
    bm.add("Rust Blog", "https://www.rust-lang.org/", None);

    let by_title: Vec<_> = bm.search("rust").collect();
    assert_eq!(by_title.len(), 1, "标题子串命中（大小写不敏感）");
    assert_eq!(by_title[0].url(), "https://www.rust-lang.org/");

    let by_url: Vec<_> = bm.search("EXAMPLE.COM").collect();
    assert_eq!(by_url.len(), 1, "URL 子串命中（大小写不敏感）");
    assert_eq!(by_url[0].title(), "Example");

    assert_eq!(bm.search("nomatch-xyz").count(), 0);
}

#[test]
fn test_bookmarks_update_title_and_url() {
    let mut bm = Bookmarks::new();
    let id = bm.add("Old", "https://old.example.com", None);
    assert!(bm.update(id, "New", "https://new.example.com"));
    let bookmark = bm.get(id).unwrap();
    assert_eq!(bookmark.title(), "New");
    assert_eq!(bookmark.url(), "https://new.example.com");
}

#[test]
fn test_bookmarks_update_nonexistent_returns_false() {
    let mut bm = Bookmarks::new();
    assert!(!bm.update(BookmarkId(99999), "New", "https://new.example.com"));
}

#[test]
fn test_bookmarks_update_rejects_oversize_text() {
    // 快照校验上限（MAX_TEXT_BYTES = 16KiB）以上拒写：超限书签会让整个
    // bookmarks.json 在下次加载时被 is_valid 整体丢弃（防数据丢失）。
    let mut bm = Bookmarks::new();
    let id = bm.add("Keep", "https://example.com", None);
    let oversize = "x".repeat(16 * 1024 + 1);
    assert!(!bm.update(id, &oversize, "https://example.com"));
    assert!(!bm.update(id, "Title", &oversize));
    let bookmark = bm.get(id).unwrap();
    assert_eq!(bookmark.title(), "Keep", "拒写时不改动原值");
    assert_eq!(bookmark.url(), "https://example.com");
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

// 三测试共用的计数器窗口互斥（见 roundtrip 测试头部说明）。
static BOOKMARK_ID_COUNTER_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// BookmarkId 计数器（bookmarks.rs `COUNTER`）是进程级单例，cargo test 并行共享：
/// 下方两个 R3363 溢出测试加载 u64::MAX id 后 `sync_counter` 把计数器顶到 u64::MAX，
/// 此后任意 `next()` 经 fetch_add 回绕到 0——与本测试早先持有的 folder_id 跨窗口
/// 比较即假红（`next_id.0 > folder_id.0` 失败）。三测试互斥串行隔离污染窗口；
/// 生产侧 saturating 语义维持 R3363 决议不动。
#[test]
fn test_bookmarks_save_load_roundtrip() {
    let _counter_guard = BOOKMARK_ID_COUNTER_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    // 预热一笔：若前序溢出测试刚释放锁且计数器停在 u64::MAX，本笔触发 fetch_add
    // 回绕到 0，后续本测试内 ID 恢复单调低位；计数器健康时仅多耗一个 ID（无副作用）。
    let _warmup = Bookmarks::new().add("counter-warmup", "about:blank", None);
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

// R3363：本地 bookmarks.json 含 `"id": u64::MAX` 时不再 panic——`from_snapshot` 的
// `max_id + 1` 旧实现溢出 panic（debug overflow-checks）/ release 回绕为 0 致 ID 冲突。
// 本地信任边界文件（~/.config/zeroweb/bookmarks.json）解析须 fail-safe 不 crash 浏览器。
#[test]
fn test_bookmarks_load_max_u64_id_no_overflow_r3363() {
    // 加载 u64::MAX id 会把全局计数器顶到 u64::MAX（污染窗口起点），须与 roundtrip
    // 等跨窗口比较 ID 的测试互斥（见 roundtrip 测试头部说明）。
    let _counter_guard = BOOKMARK_ID_COUNTER_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join(format!("zeroweb_test_bookmarks_ovf-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("bookmarks.json");

    // 手工构造含 u64::MAX id 的 bookmarks.json（模拟损坏/恶意本地文件）。
    // bookmarks.json 格式：{"bookmarks":[{id,title,url,folder_id}], "folders":[{id,name}]}
    let crafted = r#"{
        "bookmarks": [
            {"id": 18446744073709551615, "title": "Max", "url": "https://max.example.com", "folder_id": null}
        ],
        "folders": []
    }"#;
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&path, crafted).unwrap();

    // 旧实现 from_snapshot 的 `max_id + 1`（max_id=u64::MAX）→ debug panic
    //（attempt to add with overflow）/ release 回绕 sync_counter(0)。
    // 修复后 saturating_add 不 panic。
    let loaded = Bookmarks::load(&path);
    assert_eq!(loaded.len(), 1, "损坏文件含 u64::MAX id 仍应加载（fail-safe）");
    assert_eq!(loaded.list_root()[0].title(), "Max");

    let _ = std::fs::remove_dir_all(&dir);
}

// R3363：folder id 为 u64::MAX 时同样不 panic（from_snapshot 的 max_id 经 folder_id 分支也取 max）。
#[test]
fn test_bookmarks_load_max_u64_folder_id_no_overflow_r3363() {
    // 同上：u64::MAX folder id 同样顶满全局计数器，须与 roundtrip 互斥。
    let _counter_guard = BOOKMARK_ID_COUNTER_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = std::env::temp_dir().join(format!("zeroweb_test_bookmarks_fovf-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("bookmarks.json");

    let crafted = r#"{
        "bookmarks": [],
        "folders": [
            {"id": 18446744073709551615, "name": "MaxFolder"}
        ]
    }"#;
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(&path, crafted).unwrap();

    let loaded = Bookmarks::load(&path);
    assert_eq!(loaded.folders().len(), 1);
    assert_eq!(loaded.folders()[0].name(), "MaxFolder");

    let _ = std::fs::remove_dir_all(&dir);
}

// ── BrowserShell::import_bookmarks（T46b 书签导入合并）──

#[test]
fn test_shell_import_bookmarks_merges_dedups_and_reuses_folder() {
    let mut shell = BrowserShell::new();
    shell.add_bookmark_with_url("https://example.com/old");

    let entries = vec![
        BookmarkImportEntry {
            title: "Old".to_string(),
            url: "https://example.com/old".to_string(),
            folder: None,
        },
        BookmarkImportEntry {
            title: "Rust".to_string(),
            url: "https://www.rust-lang.org/".to_string(),
            folder: Some("Dev".to_string()),
        },
        BookmarkImportEntry {
            title: "Rust Again".to_string(),
            url: "https://www.rust-lang.org/".to_string(),
            folder: Some("Dev".to_string()),
        },
    ];
    let (added, skipped) = shell.import_bookmarks(&entries);
    assert_eq!((added, skipped), (1, 2), "URL 重复与既有条目都计入跳过");
    assert_eq!(shell.bookmarks().len(), 2);

    // 再次导入同名文件夹：复用既有文件夹，不新建。
    let more = vec![BookmarkImportEntry {
        title: "Crates".to_string(),
        url: "https://crates.io/".to_string(),
        folder: Some("Dev".to_string()),
    }];
    let (added_more, skipped_more) = shell.import_bookmarks(&more);
    assert_eq!((added_more, skipped_more), (1, 0));
    assert_eq!(shell.bookmarks().folders().len(), 1, "同名文件夹必须复用");
    let folder_id = shell.bookmarks().folders()[0].id();
    assert_eq!(shell.bookmarks().list_in_folder(folder_id).len(), 2);
}

#[test]
fn test_shell_import_bookmarks_skips_over_limit_text() {
    let mut shell = BrowserShell::new();
    // 超过 MAX_TEXT_BYTES 的标题：快照校验会整体丢弃超限文件，导入必须跳过。
    let entries = vec![BookmarkImportEntry {
        title: "x".repeat(MAX_TEXT_BYTES + 1),
        url: "https://example.com/big".to_string(),
        folder: None,
    }];
    let (added, skipped) = shell.import_bookmarks(&entries);
    assert_eq!((added, skipped), (0, 1));
    assert!(shell.bookmarks().is_empty());
}

#[test]
fn test_shell_import_bookmarks_stops_at_store_capacity() {
    let mut shell = BrowserShell::new();
    // 预填到上限（直写存储，不经 add_bookmark_with_url 的逐条落盘）：
    // 随后整批导入全部跳过，不得写出超限快照。
    for index in 0..MAX_ENTRIES {
        shell
            .bookmarks_mut()
            .add("Filler", &format!("https://example.com/{index}"), None);
    }
    let entries = vec![BookmarkImportEntry {
        title: "Overflow".to_string(),
        url: "https://example.com/overflow".to_string(),
        folder: None,
    }];
    let (added, skipped) = shell.import_bookmarks(&entries);
    assert_eq!((added, skipped), (0, 1));
    assert_eq!(shell.bookmarks().len(), MAX_ENTRIES);
}
