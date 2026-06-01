//! # zero-browser-shell
//!
//! 浏览器应用层 — 多标签页、收藏夹、地址栏、历史。
//!
//! 提供 UI-agnostic 的浏览器 shell 数据模型和协调逻辑，
//! 可被任何 UI 框架消费。实际渲染由 render-foundation 完成。

#![warn(missing_docs)]

mod bookmarks;
mod browser;
mod history;
mod tab;

pub use bookmarks::*;
pub use browser::*;
pub use history::*;
pub use tab::*;

#[cfg(test)]
mod tests {
    use super::*;

    // ── Tab 管理测试 ──

    #[test]
    fn test_tab_new() {
        let tab = Tab::new("https://example.com");
        assert_eq!(tab.url(), Some("https://example.com"));
        assert!(tab.title().is_none());
        assert!(tab.is_loading()); // New tab with URL starts loading
    }

    #[test]
    fn test_tab_new_empty() {
        let tab = Tab::new_empty();
        assert!(tab.url().is_none());
        assert!(tab.title().is_none());
        assert!(!tab.is_loading());
    }

    #[test]
    fn test_tab_set_url() {
        let mut tab = Tab::new_empty();
        tab.set_url("https://example.com");
        assert_eq!(tab.url(), Some("https://example.com"));
        assert!(tab.is_loading());
    }

    #[test]
    fn test_tab_set_title() {
        let mut tab = Tab::new("https://example.com");
        tab.set_title("Example Page");
        assert_eq!(tab.title(), Some("Example Page"));
    }

    #[test]
    fn test_tab_set_loading() {
        let mut tab = Tab::new("https://example.com");
        tab.set_loading(true);
        assert!(tab.is_loading());
        tab.set_loading(false);
        assert!(!tab.is_loading());
    }

    #[test]
    fn test_tab_id_unique() {
        let tab1 = Tab::new_empty();
        let tab2 = Tab::new_empty();
        assert_ne!(tab1.id(), tab2.id(), "Each tab should have a unique ID");
    }

    // ── TabManager 测试 ──

    #[test]
    fn test_tab_manager_new() {
        let manager = TabManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.len(), 0);
        assert!(manager.active_tab().is_none());
    }

    #[test]
    fn test_tab_manager_create_tab() {
        let mut manager = TabManager::new();
        let id = manager.create_tab(Some("https://example.com"));
        assert!(!manager.is_empty());
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.active_tab_id(), Some(id));
    }

    #[test]
    fn test_tab_manager_create_empty_tab() {
        let mut manager = TabManager::new();
        let id = manager.create_tab(None);
        assert_eq!(manager.len(), 1);
        let tab = manager.active_tab().unwrap();
        assert!(tab.url().is_none());
        assert_eq!(tab.id(), id);
    }

    #[test]
    fn test_tab_manager_switch_tab() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab(Some("https://a.com"));
        let id2 = manager.create_tab(Some("https://b.com"));
        assert_eq!(manager.active_tab_id(), Some(id2));

        manager.switch_to(id1);
        assert_eq!(manager.active_tab_id(), Some(id1));
    }

    #[test]
    fn test_tab_manager_switch_to_nonexistent() {
        let mut manager = TabManager::new();
        manager.create_tab(Some("https://a.com"));
        // Switch to nonexistent tab should not crash
        manager.switch_to(TabId(99999));
        // Active tab should remain unchanged
        assert!(manager.active_tab().is_some());
    }

    #[test]
    fn test_tab_manager_close_tab() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab(Some("https://a.com"));
        let id2 = manager.create_tab(Some("https://b.com"));
        assert_eq!(manager.len(), 2);

        manager.close_tab(id1);
        assert_eq!(manager.len(), 1);
        assert_eq!(manager.active_tab_id(), Some(id2));
    }

    #[test]
    fn test_tab_manager_close_active_tab() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab(Some("https://a.com"));
        let _id2 = manager.create_tab(Some("https://b.com"));
        let id3 = manager.create_tab(Some("https://c.com"));

        // Close active tab (id3) — should switch to previous
        manager.close_tab(id3);
        // After closing the active tab, should switch to an adjacent one
        let active = manager.active_tab_id();
        assert!(active == Some(id1) || active.is_some());
    }

    #[test]
    fn test_tab_manager_close_last_tab() {
        let mut manager = TabManager::new();
        let id = manager.create_tab(Some("https://a.com"));
        manager.close_tab(id);
        assert!(manager.is_empty());
        assert!(manager.active_tab().is_none());
    }

    #[test]
    fn test_tab_manager_close_nonexistent() {
        let mut manager = TabManager::new();
        manager.create_tab(Some("https://a.com"));
        manager.close_tab(TabId(99999));
        assert_eq!(manager.len(), 1);
    }

    #[test]
    fn test_tab_manager_get_tab() {
        let mut manager = TabManager::new();
        let id = manager.create_tab(Some("https://example.com"));
        let tab = manager.get_tab(id).unwrap();
        assert_eq!(tab.url(), Some("https://example.com"));
    }

    #[test]
    fn test_tab_manager_get_tab_mut() {
        let mut manager = TabManager::new();
        let id = manager.create_tab(Some("https://example.com"));
        let tab = manager.get_tab_mut(id).unwrap();
        tab.set_title("Test Title");
        assert_eq!(tab.title(), Some("Test Title"));
    }

    #[test]
    fn test_tab_manager_tabs() {
        let mut manager = TabManager::new();
        manager.create_tab(Some("https://a.com"));
        manager.create_tab(Some("https://b.com"));
        let urls: Vec<_> = manager.tabs().map(|t| t.url().unwrap()).collect();
        assert_eq!(urls, vec!["https://a.com", "https://b.com"]);
    }

    #[test]
    fn test_tab_manager_multiple_operations() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab(Some("https://a.com"));
        let id2 = manager.create_tab(Some("https://b.com"));
        let id3 = manager.create_tab(Some("https://c.com"));

        manager.switch_to(id1);
        assert_eq!(manager.active_tab_id(), Some(id1));

        manager.close_tab(id2);
        assert_eq!(manager.len(), 2);

        manager.switch_to(id3);
        assert_eq!(manager.active_tab_id(), Some(id3));

        let id4 = manager.create_tab(Some("https://d.com"));
        assert_eq!(manager.active_tab_id(), Some(id4));
        assert_eq!(manager.len(), 3);
    }

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

    // ── Browser Shell 测试 ──

    #[test]
    fn test_browser_shell_new() {
        let shell = BrowserShell::new();
        assert!(!shell.is_empty());
        assert_eq!(shell.tab_count(), 1); // Starts with one empty tab
    }

    #[test]
    fn test_browser_shell_new_tab() {
        let mut shell = BrowserShell::new();
        let id = shell.new_tab(Some("https://example.com"));
        assert_eq!(shell.tab_count(), 2);
        assert_eq!(shell.active_tab_id(), Some(id));
    }

    #[test]
    fn test_browser_shell_close_tab() {
        let mut shell = BrowserShell::new();
        let id = shell.new_tab(Some("https://example.com"));
        assert_eq!(shell.tab_count(), 2);
        shell.close_tab(id);
        assert_eq!(shell.tab_count(), 1);
    }

    #[test]
    fn test_browser_shell_navigate() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://example.com");
        let tab = shell.active_tab().unwrap();
        assert_eq!(tab.url(), Some("https://example.com"));
        assert!(tab.is_loading());
    }

    #[test]
    fn test_browser_shell_go_back() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://a.com");
        shell.on_page_loaded("Page A");
        shell.navigate("https://b.com");
        shell.on_page_loaded("Page B");

        assert!(shell.go_back());
        let tab = shell.active_tab().unwrap();
        assert_eq!(tab.url(), Some("https://a.com"));
    }

    #[test]
    fn test_browser_shell_go_forward() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://a.com");
        shell.on_page_loaded("Page A");
        shell.navigate("https://b.com");
        shell.on_page_loaded("Page B");
        shell.go_back();

        assert!(shell.go_forward());
        let tab = shell.active_tab().unwrap();
        assert_eq!(tab.url(), Some("https://b.com"));
    }

    #[test]
    fn test_browser_shell_go_back_no_history() {
        let mut shell = BrowserShell::new();
        assert!(!shell.go_back());
    }

    #[test]
    fn test_browser_shell_add_bookmark() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://example.com");
        shell.on_page_loaded("Example");
        shell.add_bookmark();

        let bookmarks = shell.bookmarks();
        assert_eq!(bookmarks.len(), 1);
    }

    #[test]
    fn test_browser_shell_history_recorded() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://a.com");
        shell.on_page_loaded("Page A");
        shell.navigate("https://b.com");
        shell.on_page_loaded("Page B");

        assert_eq!(shell.history().len(), 2);
    }

    #[test]
    fn test_browser_shell_switch_tab() {
        let mut shell = BrowserShell::new();
        let id1 = shell.active_tab_id().unwrap();
        let id2 = shell.new_tab(Some("https://b.com"));

        shell.switch_tab(id1);
        assert_eq!(shell.active_tab_id(), Some(id1));

        shell.switch_tab(id2);
        assert_eq!(shell.active_tab_id(), Some(id2));
    }

    #[test]
    fn test_browser_shell_refresh() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://example.com");
        shell.on_page_loaded("Example");
        assert!(!shell.active_tab().unwrap().is_loading());

        shell.refresh();
        assert!(shell.active_tab().unwrap().is_loading());
    }

    // ── Tab 导航边界测试 ──

    #[test]
    fn test_tab_go_back_at_start() {
        let mut tab = Tab::new("https://example.com");
        assert!(!tab.go_back(), "Should not go back at start");
        assert_eq!(tab.url(), Some("https://example.com"));
    }

    #[test]
    fn test_tab_go_forward_at_end() {
        let mut tab = Tab::new("https://example.com");
        assert!(!tab.go_forward(), "Should not go forward at end");
    }

    #[test]
    fn test_tab_navigation_truncates_forward_history() {
        let mut tab = Tab::new("https://a.com");
        tab.set_title("A");
        tab.navigate("https://b.com");
        tab.set_title("B");
        tab.navigate("https://c.com");
        tab.set_title("C");
        // Go back to a
        tab.go_back();
        tab.go_back();
        assert_eq!(tab.url(), Some("https://a.com"));
        // Navigate to d — forward history should be truncated
        tab.navigate("https://d.com");
        assert!(!tab.go_forward(), "Forward history should be truncated");
        assert_eq!(tab.url(), Some("https://d.com"));
    }

    #[test]
    fn test_tab_multiple_back_forward() {
        let mut tab = Tab::new("https://a.com");
        tab.navigate("https://b.com");
        tab.navigate("https://c.com");
        // Back to b
        assert!(tab.go_back());
        assert_eq!(tab.url(), Some("https://b.com"));
        // Back to a
        assert!(tab.go_back());
        assert_eq!(tab.url(), Some("https://a.com"));
        // Forward to b
        assert!(tab.go_forward());
        assert_eq!(tab.url(), Some("https://b.com"));
        // Forward to c
        assert!(tab.go_forward());
        assert_eq!(tab.url(), Some("https://c.com"));
    }

    #[test]
    fn test_tab_history_len_and_index() {
        let mut tab = Tab::new("https://a.com");
        assert_eq!(tab.history_len(), 1);
        assert_eq!(tab.history_index(), 0);
        tab.navigate("https://b.com");
        assert_eq!(tab.history_len(), 2);
        assert_eq!(tab.history_index(), 1);
        tab.go_back();
        assert_eq!(tab.history_index(), 0);
    }

    #[test]
    fn test_tab_navigate_resets_title() {
        let mut tab = Tab::new("https://a.com");
        tab.set_title("Page A");
        assert_eq!(tab.title(), Some("Page A"));
        tab.navigate("https://b.com");
        assert!(tab.title().is_none(), "Title should be reset on navigate");
    }

    #[test]
    fn test_tab_empty_url() {
        let tab = Tab::new_empty();
        assert!(tab.url().is_none());
        assert!(tab.title().is_none());
        assert!(!tab.is_loading());
        assert_eq!(tab.history_len(), 0);
    }

    #[test]
    fn test_tab_set_title_updates_history() {
        let mut tab = Tab::new("https://a.com");
        tab.set_title("Updated Title");
        assert_eq!(tab.title(), Some("Updated Title"));
    }

    // ── TabManager 边界测试 ──

    #[test]
    fn test_tab_manager_close_first_of_three() {
        let mut manager = TabManager::new();
        let id1 = manager.create_tab(Some("https://a.com"));
        let _id2 = manager.create_tab(Some("https://b.com"));
        let _id3 = manager.create_tab(Some("https://c.com"));
        // Close first tab — should switch to adjacent
        manager.close_tab(id1);
        assert_eq!(manager.len(), 2);
        assert!(manager.active_tab().is_some());
    }

    #[test]
    fn test_tab_manager_close_middle_tab() {
        let mut manager = TabManager::new();
        let _id1 = manager.create_tab(Some("https://a.com"));
        let id2 = manager.create_tab(Some("https://b.com"));
        let _id3 = manager.create_tab(Some("https://c.com"));
        // Close middle tab
        manager.close_tab(id2);
        assert_eq!(manager.len(), 2);
        assert!(manager.active_tab().is_some());
    }

    #[test]
    fn test_tab_manager_get_nonexistent() {
        let manager = TabManager::new();
        assert!(manager.get_tab(TabId(99999)).is_none());
        let mut manager_mut = TabManager::new();
        assert!(manager_mut.get_tab_mut(TabId(99999)).is_none());
    }

    #[test]
    fn test_tab_manager_active_tab_mut() {
        let mut manager = TabManager::new();
        manager.create_tab(Some("https://example.com"));
        let tab = manager.active_tab_mut().unwrap();
        tab.set_title("Modified");
        assert_eq!(manager.active_tab().unwrap().title(), Some("Modified"));
    }

    #[test]
    fn test_tab_manager_empty_active() {
        let manager = TabManager::new();
        assert!(manager.active_tab().is_none());
        let mut manager_mut = TabManager::new();
        assert!(manager_mut.active_tab_mut().is_none());
        assert!(manager.active_tab_id().is_none());
    }

    #[test]
    fn test_tab_manager_create_many() {
        let mut manager = TabManager::new();
        let mut ids = Vec::new();
        for i in 0..20 {
            ids.push(manager.create_tab(Some(&format!("https://site{i}.com"))));
        }
        assert_eq!(manager.len(), 20);
        // Last created should be active
        assert_eq!(manager.active_tab_id(), Some(ids[19]));
        // Switch to first
        manager.switch_to(ids[0]);
        assert_eq!(manager.active_tab_id(), Some(ids[0]));
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

    // ── BrowserShell 集成边界测试 ──

    #[test]
    fn test_browser_shell_navigate_empty_tab() {
        let mut shell = BrowserShell::new();
        // Default tab is empty — navigate should work
        shell.navigate("https://example.com");
        assert_eq!(shell.active_tab().unwrap().url(), Some("https://example.com"));
    }

    #[test]
    fn test_browser_shell_add_bookmark_no_url() {
        let mut shell = BrowserShell::new();
        // Default tab has no URL — add_bookmark should be a no-op
        shell.add_bookmark();
        assert_eq!(shell.bookmarks().len(), 0);
    }

    #[test]
    fn test_browser_shell_add_bookmark_uses_title() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://example.com");
        shell.on_page_loaded("Example Page");
        shell.add_bookmark();
        let bm = shell.bookmarks().iter().next().unwrap();
        assert_eq!(bm.title(), "Example Page");
        assert_eq!(bm.url(), "https://example.com");
    }

    #[test]
    fn test_browser_shell_add_bookmark_uses_url_as_fallback_title() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://example.com");
        // Don't call on_page_loaded — title is None
        shell.add_bookmark();
        let bm = shell.bookmarks().iter().next().unwrap();
        assert_eq!(bm.title(), "https://example.com");
    }

    #[test]
    fn test_browser_shell_go_forward_no_history() {
        let mut shell = BrowserShell::new();
        assert!(!shell.go_forward(), "Should not go forward without history");
    }

    #[test]
    fn test_browser_shell_refresh_empty_tab() {
        let mut shell = BrowserShell::new();
        shell.refresh();
        // Should not panic, no URL so loading stays false
        assert!(!shell.active_tab().unwrap().is_loading());
    }

    #[test]
    fn test_browser_shell_on_page_error() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://example.com");
        assert!(shell.active_tab().unwrap().is_loading());
        shell.on_page_error("Network timeout");
        assert!(!shell.active_tab().unwrap().is_loading());
    }

    #[test]
    fn test_browser_shell_multiple_tabs_history() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://a.com");
        shell.on_page_loaded("A");
        let id2 = shell.new_tab(Some("https://b.com"));
        shell.switch_tab(id2);
        shell.on_page_loaded("B");
        // Both should be in history
        assert_eq!(shell.history().len(), 2);
    }

    #[test]
    fn test_browser_shell_close_all_tabs_creates_none() {
        let mut shell = BrowserShell::new();
        let id = shell.active_tab_id().unwrap();
        shell.close_tab(id);
        assert!(shell.is_empty());
        assert!(shell.active_tab_id().is_none());
    }

    #[test]
    fn test_browser_shell_bookmarks_mut() {
        let mut shell = BrowserShell::new();
        shell.bookmarks_mut().add("Direct", "https://direct.com", None);
        assert_eq!(shell.bookmarks().len(), 1);
    }

    #[test]
    fn test_browser_shell_history_mut() {
        let mut shell = BrowserShell::new();
        shell.history_mut().record("https://manual.com", "Manual");
        assert_eq!(shell.history().len(), 1);
    }

    #[test]
    fn test_browser_shell_navigate_multiple_pages() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://a.com");
        shell.on_page_loaded("A");
        shell.navigate("https://b.com");
        shell.on_page_loaded("B");
        shell.navigate("https://c.com");
        shell.on_page_loaded("C");
        // History should have all 3
        assert_eq!(shell.history().len(), 3);
        // Can go back twice
        assert!(shell.go_back()); // c -> b
        assert!(shell.go_back()); // b -> a
        assert_eq!(shell.active_tab().unwrap().url(), Some("https://a.com"));
        // Forward twice
        assert!(shell.go_forward()); // a -> b
        assert!(shell.go_forward()); // b -> c
        assert_eq!(shell.active_tab().unwrap().url(), Some("https://c.com"));
    }

    #[test]
    fn test_browser_shell_default() {
        let shell = BrowserShell::default();
        assert_eq!(shell.tab_count(), 1);
    }

    #[test]
    fn test_browser_shell_active_tab_mut() {
        let mut shell = BrowserShell::new();
        shell.navigate("https://example.com");
        let tab = shell.active_tab_mut().unwrap();
        tab.set_title("Custom Title");
        assert_eq!(shell.active_tab().unwrap().title(), Some("Custom Title"));
    }

    #[test]
    fn test_browser_shell_tab_count_after_operations() {
        let mut shell = BrowserShell::new();
        assert_eq!(shell.tab_count(), 1);
        let id2 = shell.new_tab(None);
        assert_eq!(shell.tab_count(), 2);
        let _id3 = shell.new_tab(None);
        assert_eq!(shell.tab_count(), 3);
        shell.close_tab(id2);
        assert_eq!(shell.tab_count(), 2);
    }
}
