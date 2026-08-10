//! 无头浏览器协议集成测试
//!
//! 验证 BrowserShell + WebView 集成、浏览上下文管理、
//! 脚本执行和渲染管线。

use zero_browser_shell::BrowserShell;
use zero_webview::{WebView, WebViewConfig};

/// 创建测试用的 BrowserShell + WebView 对。
fn create_test_session() -> (BrowserShell, WebView) {
    let mut shell = BrowserShell::new();
    shell.new_tab(None);
    let config = WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    };
    let webview = WebView::new(config);
    (shell, webview)
}

#[test]
fn test_browser_shell_new_tab_and_close() {
    let (mut shell, _) = create_test_session();
    let initial_count = shell.tab_count();
    assert!(initial_count >= 1);

    let tab_id = shell.new_tab(Some("https://example.com"));
    assert_eq!(shell.tab_count(), initial_count + 1);

    shell.close_tab(tab_id);
    assert_eq!(shell.tab_count(), initial_count);
}

#[test]
fn test_browser_shell_navigation_history() {
    let (mut shell, _) = create_test_session();
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");

    // go_back 返回 false 表示无法后退（首页）
    assert!(!shell.go_back());
    assert!(!shell.go_forward());

    shell.navigate("https://example.org");
    shell.on_page_loaded("Example Org");
    // 现在可以后退
    assert!(shell.go_back());
    // 后退后可以前进
    assert!(shell.go_forward());
}

#[test]
fn test_browser_shell_bookmarks() {
    let (mut shell, _) = create_test_session();
    let count_before = shell.bookmarks().len();

    shell.bookmarks_mut().add("Example", "https://example.com", None);
    assert_eq!(shell.bookmarks().len(), count_before + 1);

    let found = shell.bookmarks().iter().any(|b| b.title() == "Example");
    assert!(found, "bookmark should be added");
}

#[test]
fn test_browser_shell_tabs_switch() {
    let (mut shell, _) = create_test_session();
    let tab1 = shell.active_tab_id();

    let tab2 = shell.new_tab(Some("https://example.org"));
    assert_eq!(shell.active_tab_id(), Some(tab2));

    if let Some(t1) = tab1 {
        shell.switch_tab(t1);
        assert_eq!(shell.active_tab_id(), Some(t1));
    }
}

#[test]
fn test_webview_creation_and_render() {
    let (_, mut webview) = create_test_session();
    let result = webview.render();
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_load_html() {
    let (_, mut webview) = create_test_session();
    let result = webview.load_html(
        "<html><body><div style=\"width:100px;height:50px;background:red;\"></div></body></html>",
        None,
    );
    assert!(
        !result.primitives().fills.is_empty(),
        "should have fill primitives after loading HTML"
    );
}

#[test]
fn test_webview_execute_script() {
    let (_, mut webview) = create_test_session();
    let result = webview.execute_script("1 + 1");
    assert!(result.is_ok(), "script execution should succeed");
}

#[test]
fn test_webview_execute_script_json() {
    let (_, mut webview) = create_test_session();
    let result = webview.execute_script("JSON.stringify({a: 1})");
    assert!(result.is_ok());
    let val: serde_json::Value = serde_json::from_str(&result.unwrap()).unwrap();
    assert_eq!(val["a"], 1);
}

#[test]
fn test_webview_execute_script_error() {
    let (_, mut webview) = create_test_session();
    let result = webview.execute_script("throw new Error('test')");
    assert!(result.is_err(), "script error should be returned as Err");
}

#[test]
fn test_tab_id_monotonic() {
    let (mut shell, _) = create_test_session();
    let id1 = shell.new_tab(None);
    let id2 = shell.new_tab(None);
    assert!(id2.0 > id1.0, "TabId should be monotonically increasing");
}

#[test]
fn test_webview_inject_css() {
    let (_, mut webview) = create_test_session();
    webview.load_html("<html><body><p>Hello</p></body></html>", None);
    let result = webview.inject_css("p { color: red; }");
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_render_primitives_structure() {
    let (_, mut webview) = create_test_session();
    webview.load_html(
        "<html><body>\
         <div style=\"background:blue;width:100px;height:50px;\"></div>\
         <div style=\"background:red;width:50px;height:25px;\"></div>\
         </body></html>",
        None,
    );
    let result = webview.render();
    assert!(
        result.primitives().fills.len() >= 2,
        "should have at least 2 fills for 2 divs"
    );
}

#[test]
fn test_browser_shell_context_menu() {
    use zero_browser_shell::{ContextMenu, ContextType};
    let menu = ContextMenu::new(ContextType::Page);
    assert!(!menu.items().is_empty(), "default page menu should have items");
}

#[test]
fn test_browser_shell_download_manager() {
    let (mut shell, _) = create_test_session();
    let count_before = shell.downloads().len();
    shell
        .downloads_mut()
        .start_download("https://example.com/file.zip", "file.zip");
    assert!(shell.downloads().len() > count_before);
}

#[test]
fn test_browser_shell_autocomplete() {
    let (mut shell, _) = create_test_session();
    // 添加历史记录以建立自动补全数据
    shell.navigate("https://example.com");
    shell.on_page_loaded("Example");
    shell.navigate("https://example.org");
    shell.on_page_loaded("Example Org");

    let results = shell.suggest("example");
    assert!(!results.is_empty(), "autocomplete should find matches");
}

#[test]
fn test_browser_shell_settings() {
    use zero_browser_shell::BrowserSettings;
    let settings = BrowserSettings::new();
    assert!(!settings.home_url.is_empty());
    assert!(settings.search("test").contains("test"));
}
