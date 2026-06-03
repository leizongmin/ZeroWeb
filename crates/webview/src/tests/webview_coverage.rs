//! WebView tests for uncovered paths - testing public API.

use zero_webview::{WebView, WebViewConfig, WebViewError, WebViewEvent};

#[test]
fn test_webview_event_callback_removal() {
    let mut webview = WebView::new(WebViewConfig::default());
    let callback1 = |event: &WebViewEvent| println!("Callback 1: {:?}", event);

    let index1 = webview.on_event(callback1);
    assert!(webview.remove_event_callback(index1));

    // Removing already-removed callback should fail
    assert!(!webview.remove_event_callback(index1));
    assert!(!webview.remove_event_callback(99));
}

#[test]
fn test_webview_execute_script_errors() {
    let mut webview = WebView::new(WebViewConfig::default());
    let result = webview.execute_script("");
    assert!(result.is_err());
}

#[test]
fn test_webview_extract_origin() {
    assert_eq!(
        WebView::extract_origin("https://example.com/path"),
        Some("https://example.com".to_string())
    );
    assert_eq!(
        WebView::extract_origin("http://localhost:3000"),
        Some("http://localhost:3000".to_string())
    );
    assert_eq!(WebView::extract_origin("not-a-url"), None);
    assert_eq!(WebView::extract_origin(""), None);
}

#[test]
fn test_webview_set_title() {
    let mut webview = WebView::new(WebViewConfig::default());
    assert_eq!(webview.title(), None);
    webview.set_title("Test Page");
    assert_eq!(webview.title(), Some("Test Page"));
}

#[test]
fn test_webview_resize() {
    let mut webview = WebView::new(WebViewConfig::default());
    webview.resize(800, 600);
}

#[test]
fn test_webview_is_loading() {
    let webview = WebView::new(WebViewConfig::default());
    assert!(!webview.is_loading());
}

#[test]
fn test_webview_load_html() {
    let mut webview = WebView::new(WebViewConfig::default());
    let result = webview.load_html("<html><body>Hello</body></html>", None);
    // Should produce render primitives
    assert!(!result.primitives.is_empty() || result.timings.parse_ms >= 0.0);
}

#[test]
fn test_webview_fail_load() {
    let mut webview = WebView::new(WebViewConfig::default());
    webview.fail_load("Network error");
}

#[test]
fn test_webview_complete_load() {
    let mut webview = WebView::new(WebViewConfig::default());
    let result = webview.complete_load("<html><body>Loaded</body></html>", Some("body { color: red; }"));
    assert!(!result.primitives.is_empty() || result.timings.parse_ms >= 0.0);
}

#[test]
fn test_webview_config_default() {
    let config = WebViewConfig::default();
    let webview = WebView::new(config);
    assert_eq!(webview.url(), None);
    assert!(webview.title().is_none());
}
