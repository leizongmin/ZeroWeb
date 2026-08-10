//! BrowserShell 集成测试（含 Find/Zoom）。

use crate::*;

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
fn test_browser_shell_reopen_last_closed_tab() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");
    let id = shell.active_tab_id().unwrap();
    shell.close_tab(id);
    assert_eq!(shell.tab_count(), 0, "关闭唯一标签后应为 0");

    // Ctrl+Shift+T 恢复
    let reopened = shell.reopen_last_closed_tab();
    assert!(reopened.is_some(), "应能恢复刚关闭的标签");
    assert_eq!(shell.tab_count(), 1, "恢复后标签数应为 1");
    // 恢复的标签应导航到原 URL。
    let active_url = shell.active_tab().and_then(|t| t.url()).unwrap_or("");
    assert_eq!(active_url, "https://example.com");

    // 队列空后再恢复返回 None。
    assert!(shell.reopen_last_closed_tab().is_none(), "无历史时返回 None");
}

#[test]
fn test_browser_shell_recently_closed_max_size() {
    let mut shell = BrowserShell::new();
    // 关闭 15 个标签，队列上限 10。
    for i in 0..15 {
        let id = shell.new_tab(Some(&format!("https://example.com/{i}")));
        shell.close_tab(id);
    }
    // 应只能恢复 10 次（队列上限）。
    let mut count = 0;
    while shell.reopen_last_closed_tab().is_some() {
        count += 1;
    }
    assert_eq!(count, 10, "recently_closed 队列上限应为 10");
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
fn test_browser_shell_remove_bookmark_by_url() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");
    shell.add_bookmark();
    assert_eq!(shell.bookmarks().len(), 1);

    // 删除存在的书签返回 true，并清空。
    assert!(shell.remove_bookmark_by_url("https://example.com"));
    assert_eq!(shell.bookmarks().len(), 0);

    // 删除不存在的书签返回 false，无副作用。
    assert!(!shell.remove_bookmark_by_url("https://example.com"));
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

// ── BrowserShell 新功能测试 ──

#[test]
fn test_browser_shell_downloads() {
    let shell = BrowserShell::new();
    assert!(shell.downloads().is_empty());
}

#[test]
fn test_browser_shell_downloads_mut() {
    let mut shell = BrowserShell::new();
    shell
        .downloads_mut()
        .start_download("https://example.com/file.zip", "file.zip");
    assert_eq!(shell.downloads().len(), 1);
}

#[test]
fn test_browser_shell_settings() {
    let shell = BrowserShell::new();
    assert_eq!(shell.settings().home_url, "https://example.com");
}

#[test]
fn test_browser_shell_settings_mut() {
    let mut shell = BrowserShell::new();
    shell.settings_mut().home_url = "https://google.com".to_string();
    assert_eq!(shell.settings().home_url, "https://google.com");
}

#[test]
fn test_browser_shell_zoom() {
    let mut shell = BrowserShell::new();
    assert!((shell.zoom() - 1.0).abs() < 0.01);

    shell.zoom_in();
    assert!((shell.zoom() - 1.1).abs() < 0.01);

    shell.zoom_out();
    assert!((shell.zoom() - 1.0).abs() < 0.01);

    shell.zoom_reset();
    assert!((shell.zoom() - 1.0).abs() < 0.01);
}

#[test]
fn test_browser_shell_zoom_clamp() {
    let mut shell = BrowserShell::new();
    shell.set_zoom(10.0);
    assert!((shell.zoom() - 5.0).abs() < 0.01, "Should clamp to max 5.0");
    shell.set_zoom(0.01);
    assert!((shell.zoom() - 0.25).abs() < 0.01, "Should clamp to min 0.25");
}

#[test]
fn test_zoom_is_per_tab_independent() {
    let mut shell = BrowserShell::new();
    let id_a = shell.new_tab(Some("https://a.example/"));
    let id_b = shell.new_tab(Some("https://b.example/"));

    // 切到 A，放大
    shell.switch_tab(id_a);
    shell.zoom_in();
    assert!((shell.zoom() - 1.1).abs() < 0.01, "tab A should be zoomed in");

    // 切到 B，应仍是 100%
    shell.switch_tab(id_b);
    assert!((shell.zoom() - 1.0).abs() < 0.01, "tab B should remain at 100%");

    // 切回 A，应保留 1.1
    shell.switch_tab(id_a);
    assert!((shell.zoom() - 1.1).abs() < 0.01, "tab A should retain its zoom");
}

#[test]
fn test_find_options_toggle_and_reset() {
    let mut shell = BrowserShell::new();
    shell.find_start("hello");
    shell.find_set_matches(5);
    assert!(!shell.find_state().case_sensitive());
    assert!(!shell.find_state().whole_word());

    // 切换大小写：选项翻转，匹配计数重置（等待渲染层重算）
    shell.find_toggle_case_sensitive();
    assert!(shell.find_state().case_sensitive());
    assert_eq!(shell.find_state().total_matches(), 0);
    assert_eq!(shell.find_state().current_match(), 0);

    // 再次切换关闭
    shell.find_toggle_case_sensitive();
    assert!(!shell.find_state().case_sensitive());

    // 全字匹配同理
    shell.find_set_matches(3);
    shell.find_toggle_whole_word();
    assert!(shell.find_state().whole_word());
    assert_eq!(shell.find_state().total_matches(), 0);
}

#[test]
fn test_browser_shell_find() {
    let mut shell = BrowserShell::new();
    assert!(!shell.find_state().is_active());

    shell.find_start("hello");
    assert!(shell.find_state().is_active());
    assert_eq!(shell.find_state().query(), "hello");

    shell.find_set_matches(5);
    assert_eq!(shell.find_state().total_matches(), 5);
    assert_eq!(shell.find_state().current_match(), 1);

    shell.find_next();
    assert_eq!(shell.find_state().current_match(), 2);

    shell.find_next();
    assert_eq!(shell.find_state().current_match(), 3);

    shell.find_previous();
    assert_eq!(shell.find_state().current_match(), 2);

    shell.find_close();
    assert!(!shell.find_state().is_active());
    assert!(shell.find_state().query().is_empty());
}

#[test]
fn test_browser_shell_find_wrap_around() {
    let mut shell = BrowserShell::new();
    shell.find_start("test");
    shell.find_set_matches(3);
    // At match 1
    assert_eq!(shell.find_state().current_match(), 1);

    // Go to 2, then 3
    shell.find_next();
    shell.find_next();
    assert_eq!(shell.find_state().current_match(), 3);

    // Wrap around to 1
    shell.find_next();
    assert_eq!(shell.find_state().current_match(), 1);
}

#[test]
fn find_next_sets_wrap_hint_when_wrapping_to_start() {
    let mut shell = BrowserShell::new();
    shell.find_start("test");
    shell.find_set_matches(3);
    // 1 → 2 → 3（无环绕）
    shell.find_next();
    shell.find_next();
    assert_eq!(shell.find_state().last_wrap(), None);
    // 3 → 1（环绕到开头）
    shell.find_next();
    assert_eq!(shell.find_state().last_wrap(), Some(FindWrapHint::WrappedToStart));
}

#[test]
fn find_previous_sets_wrap_hint_when_wrapping_to_end() {
    let mut shell = BrowserShell::new();
    shell.find_start("test");
    shell.find_set_matches(3);
    // 1 → 3（环绕到末尾）
    shell.find_previous();
    assert_eq!(shell.find_state().last_wrap(), Some(FindWrapHint::WrappedToEnd));
    // 3 → 2（非环绕，清除 hint）
    shell.find_previous();
    assert_eq!(shell.find_state().last_wrap(), None);
}

#[test]
fn find_start_clears_wrap_hint() {
    let mut shell = BrowserShell::new();
    shell.find_start("test");
    shell.find_set_matches(2);
    shell.find_next(); // 1 → 2
    shell.find_next(); // 2 → 1 wrap
    assert!(shell.find_state().last_wrap().is_some());
    shell.find_start("new");
    assert_eq!(shell.find_state().last_wrap(), None);
}

#[test]
fn test_browser_shell_find_previous_at_start() {
    let mut shell = BrowserShell::new();
    shell.find_start("test");
    shell.find_set_matches(3);
    assert_eq!(shell.find_state().current_match(), 1);

    // Go previous should wrap to 3
    shell.find_previous();
    assert_eq!(shell.find_state().current_match(), 3);
}

#[test]
fn test_browser_shell_find_no_matches() {
    let mut shell = BrowserShell::new();
    shell.find_start("nothing");
    // No matches set
    assert_eq!(shell.find_state().total_matches(), 0);
    assert_eq!(shell.find_state().current_match(), 0);
    // find_next/find_previous should be no-ops
    shell.find_next();
    assert_eq!(shell.find_state().current_match(), 0);
    shell.find_previous();
    assert_eq!(shell.find_state().current_match(), 0);
}

// ── FindState 边界测试 ──

#[test]
fn test_find_next_with_zero_matches_is_noop() {
    let mut shell = BrowserShell::new();
    shell.new_tab(Some("https://example.com"));
    shell.find_start("test");
    shell.find_set_matches(0);
    shell.find_next();
    assert_eq!(shell.find_state().current_match(), 0);
}

#[test]
fn test_find_previous_with_zero_matches_is_noop() {
    let mut shell = BrowserShell::new();
    shell.new_tab(Some("https://example.com"));
    shell.find_start("test");
    shell.find_previous();
    assert_eq!(shell.find_state().current_match(), 0);
}

#[test]
fn test_find_close_resets_state() {
    let mut shell = BrowserShell::new();
    shell.new_tab(Some("https://example.com"));
    shell.find_start("hello");
    shell.find_set_matches(5);
    shell.find_next();
    assert!(shell.find_state().is_active());
    shell.find_close();
    assert!(!shell.find_state().is_active());
    assert!(shell.find_state().query().is_empty());
    assert_eq!(shell.find_state().current_match(), 0);
    assert_eq!(shell.find_state().total_matches(), 0);
}

#[test]
fn test_find_next_wraps_around() {
    let mut shell = BrowserShell::new();
    shell.new_tab(Some("https://example.com"));
    shell.find_start("test");
    shell.find_set_matches(3);
    assert_eq!(shell.find_state().current_match(), 1);
    shell.find_next(); // 1 → 2
    assert_eq!(shell.find_state().current_match(), 2);
    shell.find_next(); // 2 → 3
    assert_eq!(shell.find_state().current_match(), 3);
    shell.find_next(); // 3 → 1（环绕）
    assert_eq!(shell.find_state().current_match(), 1);
}

#[test]
fn test_find_previous_at_start_wraps() {
    let mut shell = BrowserShell::new();
    shell.new_tab(Some("https://example.com"));
    shell.find_start("test");
    shell.find_set_matches(3);
    assert_eq!(shell.find_state().current_match(), 1);
    shell.find_previous(); // 1 → 3（环绕）
    assert_eq!(shell.find_state().current_match(), 3);
}

#[test]
fn test_find_previous_mid_range() {
    let mut shell = BrowserShell::new();
    shell.new_tab(Some("https://example.com"));
    shell.find_start("test");
    shell.find_set_matches(5);
    // current_match = 1, 先跳到 3
    shell.find_next(); // 2
    shell.find_next(); // 3
    shell.find_previous(); // 3 → 2
    assert_eq!(shell.find_state().current_match(), 2);
}

// ── 缩放边界测试 ──

#[test]
fn test_zoom_in_max_clamped() {
    let mut shell = BrowserShell::new();
    for _ in 0..100 {
        shell.zoom_in();
    }
    assert_eq!(shell.zoom(), 5.0, "缩放不应超过 500%");
}

#[test]
fn test_zoom_out_min_clamped() {
    let mut shell = BrowserShell::new();
    for _ in 0..100 {
        shell.zoom_out();
    }
    assert!((shell.zoom() - 0.25).abs() < 0.01, "缩放不应低于 25%");
}

#[test]
fn test_zoom_reset() {
    let mut shell = BrowserShell::new();
    shell.zoom_in();
    shell.zoom_in();
    assert!(shell.zoom() > 1.0);
    shell.zoom_reset();
    assert_eq!(shell.zoom(), 1.0);
}

#[test]
fn test_set_zoom_direct_clamp() {
    let mut shell = BrowserShell::new();
    shell.set_zoom(0.1);
    assert!((shell.zoom() - 0.25).abs() < 0.01);
    shell.set_zoom(10.0);
    assert_eq!(shell.zoom(), 5.0);
}

// ── 浏览器 Shell 页面加载回调测试 ──

#[test]
fn test_on_page_loaded_records_history() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example Domain");
    assert_eq!(shell.history().iter().count(), 1);
    let entry = shell.history().iter().next().unwrap();
    assert_eq!(entry.url(), "https://example.com");
    assert_eq!(entry.title(), "Example Domain");
}

#[test]
fn test_on_page_loaded_updates_tab_title() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://example.com");
    shell.on_page_loaded("My Title");
    let tab = shell.active_tab().unwrap();
    assert_eq!(tab.title(), Some("My Title"));
    assert!(!tab.is_loading());
}

#[test]
fn test_on_page_error_stops_loading() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://example.com");
    assert!(shell.active_tab().unwrap().is_loading());
    shell.on_page_error("Network timeout");
    assert!(!shell.active_tab().unwrap().is_loading());
}

#[test]
fn test_navigate_multiple_records_history() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://a.com");
    shell.on_page_loaded("A");
    shell.navigate("https://b.com");
    shell.on_page_loaded("B");
    shell.navigate("https://c.com");
    shell.on_page_loaded("C");
    assert_eq!(shell.history().iter().count(), 3);
}

#[test]
fn test_go_back_then_navigate_clears_forward() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://a.com");
    shell.on_page_loaded("A");
    shell.navigate("https://b.com");
    shell.on_page_loaded("B");
    assert!(shell.go_back()); // 回到 A
    shell.navigate("https://c.com"); // 新导航应清空前进历史
    assert!(!shell.go_forward(), "新导航后前进历史应清空");
}

#[test]
/// 测试 BrowserShell 多次导航后前进历史被清空。
fn test_browser_shell_navigation_clears_forward() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://a.com");
    shell.on_page_loaded("A");
    shell.navigate("https://b.com");
    shell.on_page_loaded("B");
    // 后退
    let back = shell.go_back();
    assert!(back);
    // 从 A 导航到 C，B 应从前进历史消失
    shell.navigate("https://c.com");
    let forward = shell.go_forward();
    assert!(!forward, "新导航后前进历史应被清空");
}

// ── 设置持久化集成测试 ──

#[test]
fn test_browser_shell_new_with_persisted_settings() {
    let dir = std::env::temp_dir().join(format!("zeroweb-test-persisted-shell-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("settings.json");
    let expected = BrowserSettings {
        search_engine: SearchEngine::Bing,
        default_zoom: 1.25,
        ..BrowserSettings::default()
    };
    expected.save(&path).expect("save isolated settings");

    let shell = BrowserShell::new_with_persisted_settings_at(&path);
    assert!(!shell.is_empty());
    assert_eq!(shell.tab_count(), 1);
    assert_eq!(shell.settings().search_engine, SearchEngine::Bing);
    assert_eq!(shell.settings().default_zoom, 1.25);
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn test_browser_shell_save_and_reload_settings() {
    use std::path::PathBuf;

    let dir = std::env::temp_dir().join(format!("zeroweb-test-shell-settings-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("settings.json");

    // 手动创建一个自定义设置文件
    let _ = std::fs::create_dir_all(&dir);
    let custom = r#"{"search_engine":"DuckDuckGo","home_url":"https://custom.test","show_bookmarks_bar":false,"javascript_enabled":true,"cookies_enabled":true,"block_third_party_cookies":true,"do_not_track":true,"default_zoom":1.5,"download_directory":"/tmp/dl"}"#;
    std::fs::write(&path, custom).unwrap();

    // 通过 BrowserSettings::load 验证可以读取
    let loaded = BrowserSettings::load(&path);
    assert_eq!(loaded.search_engine, SearchEngine::DuckDuckGo);
    assert_eq!(loaded.home_url, "https://custom.test");
    assert!(!loaded.show_bookmarks_bar);
    assert!(loaded.do_not_track);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_browser_shell_apply_settings() {
    let mut shell = BrowserShell::new();
    assert!(shell.settings().show_bookmarks_bar);
    shell.apply_settings(|settings| settings.show_bookmarks_bar = false);
    assert!(!shell.settings().show_bookmarks_bar);
}

#[test]
fn test_browser_shell_save_settings() {
    let mut shell = BrowserShell::new();
    shell.settings_mut().home_url = "https://test-save.test".to_string();

    // save_settings 写到默认路径（~/.config/zeroweb/settings.json）
    // 这里仅验证方法不 panic，不实际验证文件（避免影响用户系统）
    let _ = shell.settings().save(
        &std::env::temp_dir()
            .join(format!("zeroweb-shell-save-test-{}", std::process::id()))
            .join("s.json"),
    );
}

// ── 会话持久化集成测试 ──

#[test]
fn test_session_save_basic() {
    let mut shell = BrowserShell::new();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");
    shell.new_tab(Some("https://github.com"));
    shell.on_page_loaded("GitHub");

    let session = shell.save_session();
    assert_eq!(session.tab_count(), 2); // 1 default (navigated to example) + 1 new tab (github)
}

#[test]
fn test_session_save_and_restore() {
    let dir = std::env::temp_dir().join(format!("zeroweb-test-session-restore-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("session.json");

    // 创建并保存会话
    let mut shell = BrowserShell::new();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");
    shell.new_tab(Some("https://github.com"));
    shell.on_page_loaded("GitHub");

    shell.save_session_to(&path).expect("save should succeed");

    // 恢复会话到新的 shell
    let mut shell2 = BrowserShell::new();
    let count = shell2.restore_session_from(&path).expect("restore should succeed");
    assert_eq!(count, 2);
    assert_eq!(shell2.tab_count(), 2);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_session_restore_preserves_urls() {
    let dir = std::env::temp_dir().join(format!("zeroweb-test-session-urls-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("session.json");

    let mut shell = BrowserShell::new();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");
    shell.new_tab(Some("https://github.com"));
    shell.on_page_loaded("GitHub");
    shell.save_session_to(&path).expect("save");

    let mut shell2 = BrowserShell::new();
    shell2.restore_session_from(&path);

    // 验证所有标签页的 URL
    let urls: Vec<Option<&str>> = shell2.tabs().map(|t| t.url()).collect();
    assert_eq!(urls.len(), 2);
    assert!(urls.iter().any(|u| *u == Some("https://example.com")));
    assert!(urls.iter().any(|u| *u == Some("https://github.com")));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_session_restore_from_missing_file() {
    let mut shell = BrowserShell::new();
    let result = shell.restore_session_from(std::path::Path::new("/nonexistent/session.json"));
    assert!(result.is_none());
}

#[test]
fn test_session_restore_empty_session() {
    let session = SessionState::new();
    let mut shell = BrowserShell::new();
    let count = shell.restore_session(&session);
    assert_eq!(count, 0);
    assert!(shell.is_empty());
}

#[test]
fn test_session_save_empty_tab() {
    let shell = BrowserShell::new(); // 默认有一个空标签页
    let session = shell.save_session();
    assert_eq!(session.tab_count(), 1);
    assert!(session.tabs[0].url.is_none());
}

#[test]
fn test_session_restore_with_active_tab() {
    let dir = std::env::temp_dir().join(format!("zeroweb-test-session-active-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("session.json");

    let mut shell = BrowserShell::new();
    let _id1 = shell.active_tab_id(); // first tab
    shell.navigate("https://a.com");
    shell.on_page_loaded("A");
    let _id2 = shell.new_tab(Some("https://b.com")); // second tab → active
    shell.on_page_loaded("B");
    let _id3 = shell.new_tab(Some("https://c.com")); // third tab → active

    shell.save_session_to(&path).expect("save");

    let mut shell2 = BrowserShell::new();
    shell2.restore_session_from(&path);

    // 活跃标签页应该是最后一个（index 2）
    assert_eq!(shell2.active_tab().unwrap().url(), Some("https://c.com"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_session_roundtrip_with_titles() {
    let dir = std::env::temp_dir().join(format!("zeroweb-test-session-titles-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let path = dir.join("session.json");

    let mut shell = BrowserShell::new();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example Site");
    shell.save_session_to(&path).expect("save");

    let mut shell2 = BrowserShell::new();
    shell2.restore_session_from(&path);

    // 验证标题被恢复
    let has_example_title = shell2.tabs().any(|t| t.title() == Some("Example Site"));
    assert!(has_example_title, "should restore tab title");

    let _ = std::fs::remove_dir_all(&dir);
}
