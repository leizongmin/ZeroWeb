// Auto-generated test file — split from webview/lib.rs
use super::super::*;
use std::cell::RefCell;
use std::rc::Rc;

// ── 基础创建与配置 ──

#[test]
fn test_webview_config_default() {
    let config = WebViewConfig::default();
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(!config.transparent);
    assert!(config.user_agent.is_none());
    assert!(config.url.is_none());
    assert!(!config.devtools);
}

#[test]
fn test_webview_new_default() {
    let wv = WebView::new(WebViewConfig::default());
    assert!(wv.url().is_none());
    assert!(wv.title().is_none());
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_none());
    assert_eq!(wv.config().width, 800);
    assert_eq!(wv.config().height, 600);
}

#[test]
fn test_webview_new_custom_config() {
    let config = WebViewConfig {
        width: 1024,
        height: 768,
        transparent: true,
        user_agent: Some("TestAgent/1.0".to_string()),
        url: None,
        devtools: true,
        external_script: None,
        ..Default::default()
    };
    let wv = WebView::new(config);
    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);
    assert!(wv.config().transparent);
    assert_eq!(wv.config().user_agent.as_deref(), Some("TestAgent/1.0"));
    assert!(wv.config().devtools);
}

// ── load_html ──

#[test]
fn test_webview_load_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.load_html("<html><body><div>Hello</div></body></html>", None);
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());
}

#[test]
fn test_webview_load_html_with_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div id=\"main\">Hello</div></body></html>";
    let css = "div { background-color: red; width: 200px; height: 100px; }";
    let result = wv.load_html(html, Some(css));
    assert!(!result.primitives.fills.is_empty());
}

#[test]
fn test_webview_load_html_empty() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.load_html("", None);
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());
}

#[test]
fn test_webview_load_html_does_not_set_url() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>Hi</body></html>", None);
    assert!(wv.url().is_none());
}

#[test]
fn test_webview_load_html_does_not_set_loading() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>Hi</body></html>", None);
    assert!(!wv.is_loading());
}

#[test]
fn test_webview_load_html_malformed() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.load_html("<div><p>unclosed<span>", None);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_load_html_unicode() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body>こんにちは世界 🌍</body></html>";
    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_load_html_updates_last_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>First</body></html>", None);
    let _first_timing = wv.last_render().unwrap().timings.total_ms;
    wv.load_html("<html><body>Second</body></html>", None);
    assert!(wv.last_render().is_some());
}

// ── load_url / complete_load ──

#[test]
fn test_webview_url_after_load() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(wv.url().is_none());
    wv.load_url("https://example.com");
    assert_eq!(wv.url(), Some("https://example.com"));
    assert!(wv.is_loading());
}

#[test]
fn test_webview_is_loading() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(!wv.is_loading());
    wv.load_url("https://example.com");
    assert!(wv.is_loading());
}

#[test]
fn test_webview_load_url_does_not_set_last_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://example.com");
    assert!(wv.last_render().is_none());
}

#[test]
fn test_webview_load_url_multiple_times() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://a.com");
    assert_eq!(wv.url(), Some("https://a.com"));
    wv.load_url("https://b.com");
    assert_eq!(wv.url(), Some("https://b.com"));
}

#[test]
fn test_webview_load_url_empty_string() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("");
    assert_eq!(wv.url(), Some(""));
    assert!(wv.is_loading());
}

#[test]
fn test_webview_title_always_none() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(wv.title().is_none());
    wv.load_html("<html><body>Hi</body></html>", None);
    assert!(wv.title().is_none());
    wv.load_url("https://example.com");
    assert!(wv.title().is_none());
}

#[test]
fn test_webview_complete_load_transitions_loading() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://example.com");
    assert!(wv.is_loading());
    assert!(wv.last_render().is_none());

    let result = wv.complete_load("<html><body>Hello</body></html>", None);
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_some());
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_complete_load_with_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://example.com");
    let result = wv.complete_load(
        "<html><body><div id='x'>Hi</div></body></html>",
        Some("div { background-color: blue; width: 100px; height: 50px; }"),
    );
    assert!(!wv.is_loading());
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_fail_load_resets_loading() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://example.com");
    assert!(wv.is_loading());
    wv.fail_load("connection refused");
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_none());
}

#[test]
fn test_webview_load_then_complete_load_cycle() {
    let mut wv = WebView::new(WebViewConfig::default());
    // First cycle
    wv.load_url("https://a.com");
    assert!(wv.is_loading());
    wv.complete_load("<html><body>A</body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://a.com"));

    // Second cycle
    wv.load_url("https://b.com");
    assert!(wv.is_loading());
    wv.complete_load("<html><body>B</body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://b.com"));
}

#[test]
fn test_webview_complete_load_without_load_url() {
    let mut wv = WebView::new(WebViewConfig::default());
    // complete_load without prior load_url should still work
    let result = wv.complete_load("<html><body>X</body></html>", None);
    assert!(!wv.is_loading());
    assert!(result.timings.total_ms >= 0.0);
}

// ── render ──

#[test]
fn test_webview_render_after_load_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><div>Hello</div></body></html>", None);
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());
}

#[test]
fn test_webview_render_without_load_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());
}

#[test]
fn test_webview_render_after_load_url() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://example.com");
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_last_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(wv.last_render().is_none());
    wv.load_html("<html><body>Test</body></html>", None);
    assert!(wv.last_render().is_some());
    let render = wv.last_render().unwrap();
    assert!(render.timings.total_ms >= 0.0);
}

// ── resize ──

#[test]
fn test_webview_resize() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert_eq!(wv.config().width, 800);
    assert_eq!(wv.config().height, 600);
    wv.resize(1024, 768);
    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);
}

#[test]
fn test_webview_resize_then_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><div>Test</div></body></html>", None);
    wv.resize(400, 300);
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0);
    assert_eq!(wv.config().width, 400);
    assert_eq!(wv.config().height, 300);
}

#[test]
fn test_webview_resize_to_zero() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.resize(0, 0);
    assert_eq!(wv.config().width, 0);
    assert_eq!(wv.config().height, 0);
}

#[test]
fn test_webview_resize_preserves_url_and_loading() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://example.com");
    wv.resize(500, 400);
    assert_eq!(wv.url(), Some("https://example.com"));
    assert!(wv.is_loading());
}

// ── execute_script ──

#[test]
fn test_webview_execute_script_success() {
    let mut wv = WebView::new(WebViewConfig::default());
    // V8 sandbox is now integrated — scripts execute successfully
    let result = wv.execute_script("1 + 1");
    assert!(result.is_ok(), "execute_script should succeed with V8");
    assert_eq!(result.unwrap(), "2");
}

// ── inject_css ──

#[test]
fn test_webview_inject_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><div>Hello</div></body></html>", None);
    let result = wv.inject_css("div { background-color: blue; }");
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_inject_css_without_load_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.inject_css("div { color: red; }");
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());
}

#[test]
fn test_webview_inject_css_multiple_times() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><div>A</div></body></html>", None);
    let result1 = wv.inject_css("div { color: red; }");
    let result2 = wv.inject_css("div { color: blue; }");
    assert!(result1.timings.total_ms >= 0.0);
    assert!(result2.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_inject_css_empty_string() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><div>Hello</div></body></html>", None);
    let result = wv.inject_css("");
    assert!(result.timings.total_ms >= 0.0);
}

// ── set_title ──

#[test]
fn test_webview_set_title() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(wv.title().is_none());
    wv.set_title("Test Page");
    assert_eq!(wv.title(), Some("Test Page"));
}

#[test]
fn test_webview_set_title_overwrite() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.set_title("First");
    wv.set_title("Second");
    assert_eq!(wv.title(), Some("Second"));
}

// ── WebViewEvent 回调系统 ──

#[test]
fn test_webview_event_load_start_on_load_url() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        events_clone.borrow_mut().push(e.clone());
    });

    wv.load_url("https://example.com");
    let recorded = events.borrow();
    assert_eq!(recorded.len(), 2);
    assert!(matches!(&recorded[0], WebViewEvent::LoadStart(u) if u == "https://example.com"));
    assert!(matches!(&recorded[1], WebViewEvent::UrlChanged(u) if u == "https://example.com"));
}

#[test]
fn test_webview_event_url_changed_only_on_change() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        events_clone.borrow_mut().push(e.clone());
    });

    wv.load_url("https://a.com");
    assert_eq!(events.borrow().len(), 2); // LoadStart + UrlChanged

    // Same URL again — UrlChanged should NOT fire
    wv.load_url("https://a.com");
    assert_eq!(events.borrow().len(), 3); // LoadStart only

    // Different URL — UrlChanged fires
    wv.load_url("https://b.com");
    assert_eq!(events.borrow().len(), 5); // LoadStart + UrlChanged
}

#[test]
fn test_webview_event_load_end_on_complete_load() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        events_clone.borrow_mut().push(e.clone());
    });

    wv.load_url("https://example.com");
    wv.complete_load("<html><body>Hi</body></html>", None);

    let recorded = events.borrow();
    // LoadStart, UrlChanged, LoadEnd
    assert_eq!(recorded.len(), 3);
    assert!(matches!(&recorded[2], WebViewEvent::LoadEnd(u) if u == "https://example.com"));
}

#[test]
fn test_webview_event_load_failed_on_fail_load() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        events_clone.borrow_mut().push(e.clone());
    });

    wv.load_url("https://example.com");
    wv.fail_load("connection refused");

    let recorded = events.borrow();
    // LoadStart, UrlChanged, LoadFailed
    assert_eq!(recorded.len(), 3);
    assert!(matches!(
        &recorded[2],
        WebViewEvent::LoadFailed(url, msg) if url == "https://example.com" && msg.contains("connection refused")
    ));
}

#[test]
fn test_webview_event_title_changed() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        events_clone.borrow_mut().push(e.clone());
    });

    wv.set_title("My Page");
    let recorded = events.borrow();
    assert_eq!(recorded.len(), 1);
    assert!(matches!(&recorded[0], WebViewEvent::TitleChanged(t) if t == "My Page"));
}

#[test]
fn test_webview_multiple_callbacks() {
    let mut wv = WebView::new(WebViewConfig::default());
    let count_a: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let count_b: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let ca = count_a.clone();
    let cb = count_b.clone();
    wv.on_event(move |_| {
        *ca.borrow_mut() += 1;
    });
    wv.on_event(move |_| {
        *cb.borrow_mut() += 1;
    });

    wv.load_url("https://example.com");
    assert_eq!(*count_a.borrow(), 2); // LoadStart + UrlChanged
    assert_eq!(*count_b.borrow(), 2);
}

#[test]
fn test_webview_remove_event_callback() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let e1 = events.clone();
    let idx = wv.on_event(move |e| {
        e1.borrow_mut().push(format!("cb1:{e:?}"));
    });
    let e2 = events.clone();
    wv.on_event(move |e| {
        e2.borrow_mut().push(format!("cb2:{e:?}"));
    });

    wv.load_url("https://a.com");
    assert_eq!(events.borrow().len(), 4); // 2 callbacks x 2 events

    let removed = wv.remove_event_callback(idx);
    assert!(removed);

    wv.load_url("https://b.com");
    // Only cb2 remains — 1 callback x 2 events (LoadStart + UrlChanged)
    assert_eq!(events.borrow().len(), 6);
}

#[test]
fn test_webview_remove_invalid_callback_index() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(!wv.remove_event_callback(99));
}

#[test]
fn test_webview_no_events_without_callback() {
    let mut wv = WebView::new(WebViewConfig::default());
    // Should not panic
    wv.load_url("https://example.com");
    wv.complete_load("<html><body>X</body></html>", None);
    assert_eq!(wv.url(), Some("https://example.com"));
}

// ── fetch_url (uses real HTTP — test error path only) ──

#[test]
fn test_webview_fetch_url_invalid_host() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        events_clone.borrow_mut().push(e.clone());
    });

    let result = wv.fetch_url("http://0.0.0.0:1/nonexistent");
    assert!(result.is_err());
    assert!(!wv.is_loading());

    // Should have fired LoadStart, UrlChanged, LoadFailed
    let recorded = events.borrow();
    assert!(recorded.len() >= 2);
    assert!(matches!(&recorded[0], WebViewEvent::LoadStart(_)));
    let has_failed = recorded.iter().any(|e| matches!(e, WebViewEvent::LoadFailed(_, _)));
    assert!(has_failed);
}

#[test]
fn test_webview_fetch_url_resets_loading_on_error() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(!wv.is_loading());
    let _ = wv.fetch_url("http://0.0.0.0:1/fail");
    assert!(!wv.is_loading());
}

// ── config.url auto-load via builder ──

#[test]
fn test_webview_builder_auto_loads_config_url() {
    let wv = WebViewBuilder::new().url("https://example.com").build();
    // builder 应自动调用 load_url
    assert_eq!(wv.url(), Some("https://example.com"));
    assert!(wv.is_loading());
}

#[test]
fn test_webview_builder_no_auto_load_without_url() {
    let wv = WebViewBuilder::new().build();
    assert!(wv.url().is_none());
    assert!(!wv.is_loading());
}

#[test]
fn test_webview_config_url_auto_loaded_via_builder() {
    let config = WebViewConfig {
        url: Some("https://example.com".into()),
        ..Default::default()
    };
    // WebView::new does NOT auto-load; WebViewBuilder::build does
    let wv_direct = WebView::new(config.clone());
    assert!(wv_direct.url().is_none());

    let wv_builder = WebViewBuilder::new().url("https://example.com").build();
    assert_eq!(wv_builder.url(), Some("https://example.com"));
}
