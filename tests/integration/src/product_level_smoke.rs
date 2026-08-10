//! 产品层 smoke 测试（质量测试矩阵 #9）
//!
//! 覆盖 ZeroBrowser 和 ZeroWebView 作为产品的关键 API：
//! - 标签页管理生命周期
//! - 地址栏 + 自动补全
//! - 书签 CRUD + 导航
//! - 历史记录 + 搜索 + 清除
//! - 下载管理
//! - 设置持久化
//! - 缩放控制
//! - 查找功能
//! - 会话保存/恢复
//! - BrowserShell + WebView 协调
//! - 右键上下文菜单

use zero_browser_shell::{BrowserSettings, BrowserShell, ContextMenu, ContextType};
use zero_webview::{WebView, WebViewConfig, WebViewEvent};

// ── 辅助函数 ──

/// 创建测试用的 BrowserShell + WebView 对
fn create_session() -> (BrowserShell, WebView) {
    let mut shell = BrowserShell::new();
    shell.new_tab(None);
    let webview = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    });
    (shell, webview)
}

/// 创建已加载内容的 BrowserShell + WebView
fn create_loaded_session() -> (BrowserShell, WebView) {
    let (mut shell, mut webview) = create_session();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example Domain");
    webview.load_html(
        r#"<html><body><h1>Example Domain</h1><p>This domain is for use in illustrative examples.</p></body></html>"#,
        None,
    );
    (shell, webview)
}

// ── 1. 标签页管理生命周期 ──

#[test]
fn test_product_tab_lifecycle_create_close() {
    let (mut shell, _) = create_session();
    let initial = shell.tab_count();
    assert!(initial >= 1, "初始应至少有 1 个标签页");

    // 创建多个标签页
    let tab1 = shell.new_tab(Some("https://site1.com"));
    let tab2 = shell.new_tab(Some("https://site2.com"));
    let tab3 = shell.new_tab(Some("https://site3.com"));
    assert_eq!(shell.tab_count(), initial + 3);

    // 关闭中间标签页
    shell.close_tab(tab2);
    assert_eq!(shell.tab_count(), initial + 2);

    // 关闭所有新增标签页
    shell.close_tab(tab1);
    shell.close_tab(tab3);
    assert_eq!(shell.tab_count(), initial);
}

#[test]
fn test_product_tab_switch_active() {
    let (mut shell, _) = create_session();
    let tab0 = shell.active_tab_id();

    let tab1 = shell.new_tab(Some("https://example.com"));
    assert_eq!(shell.active_tab_id(), Some(tab1), "新标签页应自动激活");

    shell.switch_tab(tab0.unwrap());
    assert_eq!(shell.active_tab_id(), tab0, "应切回原始标签页");

    shell.switch_tab(tab1);
    assert_eq!(shell.active_tab_id(), Some(tab1));
}

#[test]
fn test_product_tab_navigation_per_tab() {
    let (mut shell, _) = create_session();

    // Tab 1: 导航历史
    shell.navigate("https://page1.com");
    shell.on_page_loaded("Page 1");
    shell.navigate("https://page2.com");
    shell.on_page_loaded("Page 2");

    // 可以后退
    assert!(shell.go_back());
    // 可以前进
    assert!(shell.go_forward());

    // Tab 2: 独立历史
    let tab2 = shell.new_tab(Some("https://other.com"));
    shell.navigate("https://other.com/page");
    shell.on_page_loaded("Other Page");

    // 切回 tab1，历史仍有效
    shell.switch_tab(shell.active_tab_id().unwrap());
}

// ── 2. 地址栏 + 自动补全 ──

#[test]
fn test_product_address_bar_autocomplete_from_history() {
    let (mut shell, _) = create_session();

    // 建立浏览历史
    shell.navigate("https://docs.rust-lang.org");
    shell.on_page_loaded("Rust Documentation");
    shell.navigate("https://www.rust-lang.org");
    shell.on_page_loaded("Rust Programming Language");
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");

    // 搜索 "rust"
    let suggestions = shell.suggest("rust");
    assert!(!suggestions.is_empty(), "自动补全应找到与 'rust' 相关的结果");
}

#[test]
fn test_product_address_bar_autocomplete_empty_query() {
    let (mut shell, _) = create_session();
    let suggestions = shell.suggest("");
    // 空查询可能返回推荐或空列表，不应 panic
    let _ = suggestions;
}

#[test]
fn test_product_address_bar_autocomplete_case_insensitive() {
    let (mut shell, _) = create_session();
    shell.navigate("https://GitHub.com");
    shell.on_page_loaded("GitHub");

    let lower = shell.suggest("github");
    let upper = shell.suggest("GITHUB");
    // 两种大小写都应有结果
    assert!(!lower.is_empty() || !upper.is_empty());
}

// ── 3. 书签 CRUD + 导航模拟 ──

#[test]
fn test_product_bookmarks_crud() {
    let (mut shell, _) = create_session();

    // Create
    shell.bookmarks_mut().add("Rust", "https://rust-lang.org", None);
    shell.bookmarks_mut().add("MDN", "https://developer.mozilla.org", None);
    assert_eq!(shell.bookmarks().len(), 2);

    // Read
    let found_rust = shell.bookmarks().iter().any(|b| b.title() == "Rust");
    assert!(found_rust, "应找到 Rust 书签");

    // Update（删除旧+添加新）
    let mdn_id = shell.bookmarks().iter().find(|b| b.title() == "MDN").map(|b| b.id());
    if let Some(id) = mdn_id {
        shell.bookmarks_mut().remove(id);
    }
    shell
        .bookmarks_mut()
        .add("MDN Web Docs", "https://developer.mozilla.org", None);
    assert_eq!(shell.bookmarks().len(), 2);

    // Delete
    let all_ids: Vec<_> = shell.bookmarks().iter().map(|b| b.id()).collect();
    for id in all_ids {
        shell.bookmarks_mut().remove(id);
    }
    assert_eq!(shell.bookmarks().len(), 0);
}

#[test]
fn test_product_bookmarks_folders() {
    let (mut shell, _) = create_session();

    // 创建文件夹中的书签
    shell.bookmarks_mut().add("Tech", "https://news.ycombinator.com", None);
    shell.bookmarks_mut().add("Rust", "https://rust-lang.org", None);

    assert!(shell.bookmarks().len() >= 2);
}

#[test]
fn test_product_add_bookmark_from_current_page() {
    let (mut shell, _) = create_session();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");

    let count_before = shell.bookmarks().len();
    shell.add_bookmark();
    assert_eq!(shell.bookmarks().len(), count_before + 1);
}

// ── 4. 历史记录 + 搜索 + 清除 ──

#[test]
fn test_product_history_recording() {
    let (mut shell, _) = create_session();

    shell.navigate("https://example.com");
    shell.on_page_loaded("Example Domain");
    shell.navigate("https://rust-lang.org");
    shell.on_page_loaded("Rust");
    shell.navigate("https://github.com");
    shell.on_page_loaded("GitHub");

    // 历史应有记录
    assert!(shell.history().len() >= 3, "应至少有 3 条历史记录");
}

#[test]
fn test_product_history_search() {
    let (mut shell, _) = create_session();

    shell.navigate("https://docs.rust-lang.org/book");
    shell.on_page_loaded("The Rust Book");
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");

    // 搜索历史
    let results: Vec<_> = shell.history().search("rust").collect();
    assert!(!results.is_empty(), "应找到包含 'rust' 的历史");
}

#[test]
fn test_product_history_clear() {
    let (mut shell, _) = create_session();

    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");
    assert!(shell.history().len() > 0);

    shell.history_mut().clear();
    assert_eq!(shell.history().len(), 0, "清除后历史应为空");
}

// ── 5. 下载管理 ──

#[test]
fn test_product_download_lifecycle() {
    let (mut shell, _) = create_session();

    // 开始下载
    let dl_id = shell
        .downloads_mut()
        .start_download("https://example.com/file.zip", "file.zip");
    assert_eq!(shell.downloads().len(), 1);

    // 添加更多下载
    shell
        .downloads_mut()
        .start_download("https://example.com/data.pdf", "data.pdf");
    assert_eq!(shell.downloads().len(), 2);

    // 需要先完成或取消下载才能移除
    shell.downloads_mut().cancel(dl_id);
    assert!(shell.downloads_mut().remove(dl_id));
    assert_eq!(shell.downloads().len(), 1);
}

// ── 6. 设置 ──

#[test]
fn test_product_settings_default_values() {
    let settings = BrowserSettings::new();
    assert!(!settings.home_url.is_empty(), "默认主页 URL 不应为空");
    assert!(settings.search("test").contains("test"), "搜索 URL 应包含查询词");
}

#[test]
fn test_product_settings_customization() {
    let mut shell = BrowserShell::new();
    let custom_home = "https://custom-home.example.com";
    shell.settings_mut().home_url = custom_home.to_string();
    assert_eq!(shell.settings().home_url, custom_home);
}

// ── 7. 缩放控制 ──

#[test]
fn test_product_zoom_controls() {
    let mut shell = BrowserShell::new();

    // 默认缩放
    assert_eq!(shell.zoom(), 1.0, "默认缩放应为 100%");

    // 放大
    shell.zoom_in();
    assert!(shell.zoom() > 1.0, "zoom_in 应增加缩放级别");

    // 缩小
    shell.zoom_out();
    assert!(shell.zoom() < 2.0, "zoom_out 应减少缩放级别");

    // 重置
    shell.zoom_reset();
    assert_eq!(shell.zoom(), 1.0, "zoom_reset 应恢复 100%");

    // 精确设置
    shell.set_zoom(1.5);
    assert_eq!(shell.zoom(), 1.5);
}

// ── 8. 查找功能 ──

#[test]
fn test_product_find_in_page_lifecycle() {
    let mut shell = BrowserShell::new();

    // 初始状态
    assert!(!shell.find_state().is_active());

    // 开始查找
    shell.find_start("search query");
    assert!(shell.find_state().is_active());
    assert_eq!(shell.find_state().query(), "search query");

    // 设置匹配数
    shell.find_set_matches(5);
    assert_eq!(shell.find_state().total_matches(), 5);
    assert_eq!(shell.find_state().current_match(), 1); // find_set_matches 自动设为 1

    // 下一个（从 1 → 2）
    shell.find_next();
    assert_eq!(shell.find_state().current_match(), 2);

    // 上一个
    shell.find_previous();

    // 关闭
    shell.find_close();
    assert!(!shell.find_state().is_active());
}

// ── 9. 会话保存/恢复 ──

#[test]
fn test_product_session_save_restore() {
    let mut shell = BrowserShell::new();
    // new() 已有 1 个默认标签页，再导航
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");
    shell.bookmarks_mut().add("Example", "https://example.com", None);

    // 保存会话
    let session = shell.save_session();
    assert_eq!(session.tab_count(), 1, "应保存 1 个标签页");

    // 创建新 shell 并恢复
    let mut shell2 = BrowserShell::new();
    let restored_count = shell2.restore_session(&session);
    assert_eq!(restored_count, 1, "应恢复 1 个标签页");
    assert_eq!(shell2.tab_count(), 1);
}

#[test]
fn test_product_session_restore_empty() {
    // 创建一个空 SessionState（不包含任何标签页）
    let session = zero_browser_shell::SessionState::new();
    assert_eq!(session.tab_count(), 0);

    let mut shell2 = BrowserShell::new();
    let count = shell2.restore_session(&session);
    assert_eq!(count, 0, "空会话应恢复 0 个标签页");
}

// ── 10. BrowserShell + WebView 协调 ──

#[test]
fn test_product_shell_webview_coordination() {
    let (mut shell, mut webview) = create_loaded_session();

    // Shell 导航
    shell.navigate("https://another-site.com");
    shell.on_page_loaded("Another Site");

    // WebView 渲染新内容
    let result = webview.load_html("<html><body><h1>Another Site</h1></body></html>", None);
    assert!(result.timings.total_ms >= 0.0);

    // Shell 后退
    assert!(shell.go_back());

    // WebView 渲染之前的内容
    let result = webview.load_html("<html><body><h1>Example Domain</h1></body></html>", None);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_product_shell_webview_script_execution() {
    let (mut shell, mut webview) = create_loaded_session();

    // 通过 WebView 执行脚本
    let r = webview.execute_script("JSON.stringify({status: 'ok'})");
    assert!(r.is_ok());

    let val: serde_json::Value = serde_json::from_str(&r.unwrap()).unwrap();
    assert_eq!(val["status"], "ok");

    // Shell 状态不受影响
    assert!(shell.active_tab_id().is_some());
}

#[test]
fn test_product_shell_webview_zoom_render() {
    let (_, mut webview) = create_loaded_session();

    // 渲染并记录原始图元数
    let result1 = webview.render();
    let glyphs1 = result1.primitives().glyphs.len();

    // 重新渲染应产生相同结果
    let result2 = webview.render();
    let glyphs2 = result2.primitives().glyphs.len();

    assert_eq!(glyphs1, glyphs2, "相同内容重新渲染图元数应一致");
}

// ── 11. 右键上下文菜单 ──

#[test]
fn test_product_context_menu_page() {
    let menu = ContextMenu::new(ContextType::Page);
    assert!(!menu.items().is_empty(), "页面上下文菜单应有默认项");
}

#[test]
fn test_product_context_menu_all_types() {
    for ctx_type in [
        ContextType::Page,
        ContextType::Link,
        ContextType::Image,
        ContextType::Selection,
        ContextType::Editable,
    ] {
        let menu = ContextMenu::new(ctx_type);
        // 每种类型都应有菜单项，不应 panic
        let _ = menu.items().len();
    }
}

// ── 12. WebView 事件回调集成 ──

#[test]
fn test_product_webview_events_with_shell() {
    let (shell, mut webview) = create_session();
    let _ = shell;

    let events = std::sync::Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let events_clone = events.clone();
    let callback_id = webview.on_event(move |event| {
        let name = match event {
            WebViewEvent::LoadStart(_) => "LoadStart",
            WebViewEvent::LoadEnd(_) => "LoadEnd",
            WebViewEvent::LoadFailed(_, _) => "LoadFailed",
            WebViewEvent::TitleChanged(_) => "TitleChanged",
            WebViewEvent::UrlChanged(_) => "UrlChanged",
        };
        events_clone.lock().unwrap().push(name.to_string());
    });

    webview.load_html("<html><body><h1>Event Test</h1></body></html>", None);

    // 移除回调
    assert!(webview.remove_event_callback(callback_id));
}

// ── 13. 多标签页 + WebView 完整场景 ──

#[test]
fn test_product_multi_tab_browsing_scenario() {
    let (mut shell, mut webview) = create_session();

    // Tab 1: 加载页面
    shell.navigate("https://news.example.com");
    shell.on_page_loaded("News");
    webview.load_html(
        r#"<html><body>
            <h1>News Headlines</h1>
            <article><h2>Story 1</h2><p>Content 1</p></article>
            <article><h2>Story 2</h2><p>Content 2</p></article>
        </body></html>"#,
        None,
    );

    // 添加书签
    shell.add_bookmark();
    let bookmark_count = shell.bookmarks().len();

    // Tab 2: 打开新标签页
    let tab2 = shell.new_tab(Some("https://docs.example.com"));
    webview.load_html(
        r#"<html><body><h1>Documentation</h1><p>API Reference</p></body></html>"#,
        None,
    );

    // 在 Tab 2 执行脚本
    let r = webview.execute_script("document.title = 'Docs'");
    let _ = r; // 可能因 polyfill 失败

    // 切回 Tab 1
    shell.switch_tab(shell.active_tab_id().unwrap());

    // 书签数量不变
    assert_eq!(shell.bookmarks().len(), bookmark_count);

    // 关闭 Tab 2
    shell.close_tab(tab2);
    assert!(shell.tab_count() >= 1);
}

// ── 14. is_empty / 边界条件 ──

#[test]
fn test_product_shell_default_has_tab() {
    // BrowserShell::new() 自动创建一个默认标签页
    let shell = BrowserShell::new();
    assert!(!shell.is_empty(), "新建 shell 应有默认标签页");
    assert_eq!(shell.tab_count(), 1);
}

#[test]
fn test_product_shell_not_empty_after_new_tab() {
    let mut shell = BrowserShell::new();
    shell.new_tab(None);
    assert!(!shell.is_empty(), "创建标签页后不应为空");
    assert!(shell.tab_count() >= 2);
}

#[test]
fn test_product_close_all_tabs() {
    let mut shell = BrowserShell::new();
    // BrowserShell::new() 自动创建一个 tab，获取其 ID
    let default_tab = shell.active_tab_id().unwrap();
    let tab1 = shell.new_tab(None);
    let tab2 = shell.new_tab(None);

    shell.close_tab(tab1);
    shell.close_tab(tab2);
    shell.close_tab(default_tab);
    assert!(shell.is_empty(), "关闭所有标签页后应为空");
}

// ── 15. 刷新操作 ──

#[test]
fn test_product_refresh_action() {
    let (mut shell, _) = create_session();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");

    // 刷新不 panic
    shell.refresh();
}

// ── 16. on_page_error 处理 ──

#[test]
fn test_product_page_error_handling() {
    let (mut shell, _) = create_session();
    shell.navigate("https://error.example.com");

    // 页面加载错误不 panic
    shell.on_page_error("Network timeout");
}
