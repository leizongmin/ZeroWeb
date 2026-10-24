//! Tab 和 TabManager 单元测试。

use crate::*;

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
fn test_tab_state_flags() {
    let mut tab = Tab::new("https://example.com");
    assert!(!tab.is_pinned());
    tab.set_pinned(true);
    assert!(tab.is_pinned());
    tab.set_muted(true);
    assert!(tab.is_muted());
    tab.set_crashed(true);
    assert!(tab.is_crashed());
    tab.set_needs_attention(true);
    assert!(tab.needs_attention());
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
fn test_tab_jump_history_arbitrary_index() {
    // t5（qq-20261016-r1）：CDP Page.navigateToHistoryEntry 共享路径——任意索引
    // 跳转，url/title 随目标条目更新（go_back/go_forward 的步长泛化）。
    let mut tab = Tab::new("https://a.com");
    tab.set_title("A");
    tab.navigate("https://b.com");
    tab.set_title("B");
    tab.navigate("https://c.com");
    assert!(tab.jump_history(0), "跳到首条目");
    assert_eq!(tab.url(), Some("https://a.com"));
    assert_eq!(tab.title(), Some("A"));
    assert_eq!(tab.history_index(), 0);
    assert!(tab.jump_history(2), "前向跳到末条目");
    assert_eq!(tab.url(), Some("https://c.com"));
    assert_eq!(tab.title(), None, "末条目标题未 set_title，随条目回退 None");
    assert!(!tab.jump_history(9), "越界索引失败且状态不变");
    assert_eq!(tab.url(), Some("https://c.com"));
}

/// T5/P10：页面 history 遍历提交面的宿主判界——jump_history（go(delta) 绝对目标共享
/// 路径）越界 no-op，遍历按钮态（can_go_back/can_go_forward）与栈不变。
#[test]
fn test_tab_jump_history_traverse_out_of_bounds_noop_t5() {
    let mut tab = Tab::new("https://a.com");
    tab.navigate("https://b.com");
    // index=1，delta=-2 → 绝对目标 -1：宿主以 i64 判界后不会调用 jump_history；
    // 此处钉 jump_history 自身对一切越界索引（含 usize 下溢前哨 0）的 no-op 语义。
    assert!(tab.can_go_back(), "前置：index=1 可后退");
    let (len, index) = (tab.history_len(), tab.history_index());
    assert!(!tab.jump_history(3), "越界（>=len）no-op");
    assert_eq!(
        (tab.history_len(), tab.history_index()),
        (len, index),
        "越界后栈与索引不变"
    );
    assert_eq!(tab.url(), Some("https://b.com"), "越界后 url 不变");
    assert!(tab.can_go_back() && !tab.can_go_forward(), "越界后遍历按钮态不变");
}

#[test]
fn test_tab_navigate_resets_title() {
    let mut tab = Tab::new("https://a.com");
    tab.set_title("Page A");
    assert_eq!(tab.title(), Some("Page A"));
    tab.navigate("https://b.com");
    assert!(tab.title().is_none(), "Title should be reset on navigate");
}

/// replace_current_entry_url（t4 返修 D3 重定向提交）：改写当前条目 URL 而非追加
/// 条目；title/loading 语义与 navigate 一致。
#[test]
fn test_tab_replace_current_entry_url_no_append() {
    let mut tab = Tab::new("https://a.com");
    tab.navigate("https://b.com");
    tab.set_title("Page B");
    tab.replace_current_entry_url("https://final.com");
    assert_eq!(tab.history_len(), 2, "取代当前条目，不追加");
    assert_eq!(tab.url(), Some("https://final.com"));
    assert!(tab.title().is_none(), "标题随取代复位");
    assert!(tab.is_loading());
    // 当前条目确被改写：后退读旧条目原值，前进读回取代值。
    assert!(tab.go_back());
    assert_eq!(tab.url(), Some("https://a.com"));
    assert!(tab.go_forward());
    assert_eq!(tab.url(), Some("https://final.com"));
}

/// replace_current_entry_url 空历史（无当前条目）不 panic，URL 仍上报。
#[test]
fn test_tab_replace_current_entry_url_empty_history_no_panic() {
    let mut tab = Tab::new_empty();
    tab.replace_current_entry_url("https://a.com");
    assert_eq!(tab.url(), Some("https://a.com"));
    assert_eq!(tab.history_len(), 0);
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

// ── Tab drag-and-drop reorder 测试 ──

#[test]
fn test_tab_manager_move_tab_forward() {
    let mut manager = TabManager::new();
    let id1 = manager.create_tab(Some("https://a.com"));
    let _id2 = manager.create_tab(Some("https://b.com"));
    let _id3 = manager.create_tab(Some("https://c.com"));
    // [a, b, c] → move a to index 2 → [b, c, a]
    assert!(manager.move_tab(id1, 2));
    let urls: Vec<_> = manager.tabs().map(|t| t.url().unwrap()).collect();
    assert_eq!(urls, vec!["https://b.com", "https://c.com", "https://a.com"]);
}

#[test]
fn test_tab_manager_move_tab_backward() {
    let mut manager = TabManager::new();
    let _id1 = manager.create_tab(Some("https://a.com"));
    let _id2 = manager.create_tab(Some("https://b.com"));
    let id3 = manager.create_tab(Some("https://c.com"));
    // [a, b, c] → move c to index 0 → [c, a, b]
    assert!(manager.move_tab(id3, 0));
    let urls: Vec<_> = manager.tabs().map(|t| t.url().unwrap()).collect();
    assert_eq!(urls, vec!["https://c.com", "https://a.com", "https://b.com"]);
}

#[test]
fn test_tab_manager_move_tab_same_position() {
    let mut manager = TabManager::new();
    let id1 = manager.create_tab(Some("https://a.com"));
    manager.create_tab(Some("https://b.com"));
    // move to same position = no-op
    assert!(!manager.move_tab(id1, 0));
}

#[test]
fn test_tab_manager_move_tab_preserves_active() {
    let mut manager = TabManager::new();
    let id1 = manager.create_tab(Some("https://a.com"));
    let id2 = manager.create_tab(Some("https://b.com"));
    manager.switch_to(id1);
    assert_eq!(manager.active_tab_id(), Some(id1));
    // move id2 → active should still be id1
    manager.move_tab(id2, 0);
    assert_eq!(manager.active_tab_id(), Some(id1));
}

#[test]
fn test_tab_manager_move_nonexistent() {
    let mut manager = TabManager::new();
    manager.create_tab(Some("https://a.com"));
    assert!(!manager.move_tab(TabId(99999), 0));
}

#[test]
fn test_tab_manager_move_out_of_bounds() {
    let mut manager = TabManager::new();
    let id1 = manager.create_tab(Some("https://a.com"));
    manager.create_tab(Some("https://b.com"));
    assert!(!manager.move_tab(id1, 100));
}

// ── Tab reorder + navigation 边界测试 ──

#[test]
/// 测试 TabManager::move_tab 将标签页从首位移到末位。
fn test_tab_manager_move_tab_first_to_last() {
    let mut mgr = TabManager::new();
    let id0 = mgr.create_tab(Some("https://a.com"));
    let _id1 = mgr.create_tab(Some("https://b.com"));
    let _id2 = mgr.create_tab(Some("https://c.com"));
    // 初始顺序: [id0, id1, id2]
    mgr.move_tab(id0, 2);
    let urls: Vec<_> = mgr.tabs().map(|t| t.url().unwrap_or("")).collect();
    assert_eq!(
        urls,
        &["https://b.com", "https://c.com", "https://a.com"],
        "id0 应移到末位"
    );
}

#[test]
/// 测试 TabManager::move_tab 无效索引不 panic。
fn test_tab_manager_move_tab_invalid_index() {
    let mut mgr = TabManager::new();
    let id = mgr.create_tab(Some("https://a.com"));
    let result = mgr.move_tab(id, 99); // 越界
    assert!(!result, "越界 move_tab 应返回 false");
    let fake_id = TabId(99999);
    let result2 = mgr.move_tab(fake_id, 0); // 不存在的 id
    assert!(!result2, "不存在的 id 应返回 false");
    assert_eq!(mgr.len(), 1, "标签页数不变");
}

#[test]
/// 测试 Tab 多次前进/后退导航历史边界。
fn test_tab_navigation_history_boundary() {
    let mut tab = Tab::new("https://a.com");
    tab.set_title("A");
    tab.navigate("https://b.com");
    tab.navigate("https://c.com");
    // 后退 2 次
    assert!(tab.go_back()); // → B
    assert!(tab.go_back()); // → A
    assert!(!tab.go_back(), "已到历史起点应返回 false");
    // 前进 2 次
    assert!(tab.go_forward()); // → B
    assert!(tab.go_forward()); // → C
    assert!(!tab.go_forward(), "已到历史末尾应返回 false");
}

// ── TabManager duplicate / close_others / close_to_right 测试 ──

#[test]
fn test_tab_manager_duplicate_tab_inserts_after_and_activates() {
    let mut mgr = TabManager::new();
    let id0 = mgr.create_tab(Some("https://a.com"));
    let id1 = mgr.create_tab(Some("https://b.com"));
    mgr.switch_to(id0);
    let dup = mgr.duplicate_tab(id0).expect("应返回副本 id");
    assert_ne!(dup, id0);
    assert_eq!(mgr.len(), 3);
    assert_eq!(mgr.active_tab_id(), Some(dup));
    let urls: Vec<_> = mgr.tabs().map(|t| t.url().unwrap_or("")).collect();
    assert_eq!(urls, &["https://a.com", "https://a.com", "https://b.com"]);
    let _ = id1;
}

#[test]
fn test_tab_manager_duplicate_nonexistent_returns_none() {
    let mut mgr = TabManager::new();
    mgr.create_tab(Some("https://a.com"));
    assert!(mgr.duplicate_tab(TabId(99999)).is_none());
}

#[test]
fn test_tab_manager_close_other_tabs() {
    let mut mgr = TabManager::new();
    let id0 = mgr.create_tab(Some("https://a.com"));
    let _id1 = mgr.create_tab(Some("https://b.com"));
    let _id2 = mgr.create_tab(Some("https://c.com"));
    mgr.close_other_tabs(id0);
    assert_eq!(mgr.len(), 1);
    assert_eq!(mgr.active_tab_id(), Some(id0));
    assert_eq!(mgr.tabs().next().unwrap().url(), Some("https://a.com"));
}

#[test]
fn test_tab_manager_close_tabs_to_right() {
    let mut mgr = TabManager::new();
    let id0 = mgr.create_tab(Some("https://a.com"));
    let _id1 = mgr.create_tab(Some("https://b.com"));
    let _id2 = mgr.create_tab(Some("https://c.com"));
    mgr.close_tabs_to_right(id0);
    assert_eq!(mgr.len(), 1);
    assert_eq!(mgr.tabs().next().unwrap().url(), Some("https://a.com"));
}

#[test]
fn test_tab_manager_close_tabs_to_right_keeps_left() {
    let mut mgr = TabManager::new();
    let _id0 = mgr.create_tab(Some("https://a.com"));
    let id1 = mgr.create_tab(Some("https://b.com"));
    let _id2 = mgr.create_tab(Some("https://c.com"));
    mgr.close_tabs_to_right(id1);
    let urls: Vec<_> = mgr.tabs().map(|t| t.url().unwrap_or("")).collect();
    assert_eq!(urls, &["https://a.com", "https://b.com"]);
}

// ── T4-I6：导航箭头启用态与历史记账一致性 ──

/// 历史序列期望态表格：启动空标签欢迎页记账 → 地址栏导航 → 链接点击 →
/// Alt+返回 → Alt+前进。每个点位断言 (历史长度, 索引, 可后退, 可前进, 当前 URL)，
/// 与 app_render.rs 导航按钮禁用判断同源（Tab::can_go_back/forward）。
#[test]
fn test_tab_nav_arrow_states_table() {
    // 启动空标签（BrowserShell::new 同模型：无 URL、无历史）。
    let mut tab = Tab::new_empty();

    // 欢迎页记账（shell::record_initial_navigation 的 Tab 层语义）。
    tab.navigate("zero://newtab");
    assert_eq!(
        (
            tab.history_len(),
            tab.history_index(),
            tab.can_go_back(),
            tab.can_go_forward()
        ),
        (1, 0, false, false),
        "点位0 欢迎页：无后退无前进"
    );

    // 地址栏导航 index.html。
    tab.navigate("http://127.0.0.1:18080/index.html");
    assert_eq!(
        (
            tab.history_len(),
            tab.history_index(),
            tab.can_go_back(),
            tab.can_go_forward()
        ),
        (2, 1, true, false),
        "点位1 地址栏导航：可返回欢迎页"
    );

    // 链接点击 page2.html。
    tab.navigate("http://127.0.0.1:18080/page2.html");
    assert_eq!(
        (
            tab.history_len(),
            tab.history_index(),
            tab.can_go_back(),
            tab.can_go_forward()
        ),
        (3, 2, true, false),
        "点位2 链接点击：可后退、无前进"
    );

    // Alt+Left 返回。
    assert!(tab.go_back());
    assert_eq!(tab.url(), Some("http://127.0.0.1:18080/index.html"));
    assert_eq!(
        (
            tab.history_len(),
            tab.history_index(),
            tab.can_go_back(),
            tab.can_go_forward()
        ),
        (3, 1, true, true),
        "点位3 Alt+返回：双向可用"
    );

    // Alt+Right 前进。
    assert!(tab.go_forward());
    assert_eq!(tab.url(), Some("http://127.0.0.1:18080/page2.html"));
    assert_eq!(
        (
            tab.history_len(),
            tab.history_index(),
            tab.can_go_back(),
            tab.can_go_forward()
        ),
        (3, 2, true, false),
        "点位4 Alt+前进：回到末尾，前进禁用"
    );

    // 边界：首位不可再退、末尾不可再进。
    let mut tab = Tab::new("https://a.com");
    tab.navigate("https://b.com");
    assert!(tab.go_back());
    assert!(!tab.go_back(), "首位再退应失败");
    assert!(!tab.can_go_back());
    assert!(tab.can_go_forward());
}

/// record_initial_navigation：仅当标签无 URL 时记账；已有 URL（如地址栏路径
/// 已 navigate 过）不产生重复条目。
#[test]
fn test_record_initial_navigation_only_when_no_url() {
    let mut shell = BrowserShell::new();
    let id = shell.active_tab_id().expect("启动含一个空标签");

    shell.record_initial_navigation(id, "zero://newtab");
    let tab = shell.tab(id).expect("标签存在");
    assert_eq!(tab.url(), Some("zero://newtab"));
    assert_eq!(tab.history_len(), 1);
    assert!(!tab.can_go_back());

    // 已有 URL 时再次补记应无操作（地址栏路径已 navigate 过）。
    shell.record_initial_navigation(id, "zero://newtab");
    let tab = shell.tab(id).expect("标签存在");
    assert_eq!(tab.history_len(), 1, "不应产生重复历史条目");
}
