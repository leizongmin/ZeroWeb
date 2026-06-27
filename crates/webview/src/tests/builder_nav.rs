// Auto-generated test file — split from webview/lib.rs
use super::super::*;
use std::cell::RefCell;
use std::rc::Rc;

// ── Builder ──

#[test]
fn test_webview_builder_default() {
    let builder = WebViewBuilder::new();
    let wv = builder.build();
    assert_eq!(wv.config().width, 800);
    assert_eq!(wv.config().height, 600);
}

#[test]
fn test_webview_builder_custom() {
    let wv = WebViewBuilder::new()
        .width(1024)
        .height(768)
        .transparent(true)
        .user_agent("TestBot/1.0")
        .devtools(true)
        .build();
    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);
    assert!(wv.config().transparent);
    assert_eq!(wv.config().user_agent.as_deref(), Some("TestBot/1.0"));
    assert!(wv.config().devtools);
}

#[test]
fn test_webview_builder_chained() {
    let wv = WebViewBuilder::new()
        .width(640)
        .height(480)
        .user_agent("Chain/2.0")
        .transparent(false)
        .build();
    assert_eq!(wv.config().width, 640);
    assert_eq!(wv.config().height, 480);
    assert!(!wv.config().transparent);
    assert_eq!(wv.config().user_agent.as_deref(), Some("Chain/2.0"));
}

#[test]
fn test_webview_builder_default_trait() {
    let wv = WebViewBuilder::default().build();
    assert_eq!(wv.config().width, 800);
    assert_eq!(wv.config().height, 600);
}

#[test]
fn test_webview_builder_setter_overwrite() {
    let wv = WebViewBuilder::new().width(100).width(200).build();
    assert_eq!(wv.config().width, 200);
}

#[test]
fn test_webview_builder_partial_width_only() {
    let wv = WebViewBuilder::new().width(500).build();
    assert_eq!(wv.config().width, 500);
    assert_eq!(wv.config().height, 600);
}

#[test]
fn test_webview_builder_partial_height_only() {
    let wv = WebViewBuilder::new().height(400).build();
    assert_eq!(wv.config().width, 800);
    assert_eq!(wv.config().height, 400);
}

// ── Error 变体显示格式 ──

#[test]
fn test_webview_error_display_rendering() {
    let err = WebViewError::Rendering("gpu oom".into());
    let msg = err.to_string();
    assert!(msg.contains("Rendering error"), "message: {msg}");
    assert!(msg.contains("gpu oom"));
}

#[test]
fn test_webview_error_display_navigation() {
    let err = WebViewError::Navigation("timeout".into());
    let msg = err.to_string();
    assert!(msg.contains("Navigation error"), "message: {msg}");
}

#[test]
fn test_webview_error_display_script() {
    let err = WebViewError::Script("syntax error".into());
    let msg = err.to_string();
    assert!(msg.contains("Script error"), "message: {msg}");
}

#[test]
fn test_webview_error_display_not_implemented() {
    let err = WebViewError::NotImplemented("todo".into());
    let msg = err.to_string();
    assert!(msg.contains("Not implemented"), "message: {msg}");
}

// ── WebViewEvent Debug/Clone ──

#[test]
fn test_webview_event_debug_format() {
    let e = WebViewEvent::LoadStart("https://example.com".to_string());
    let debug = format!("{e:?}");
    assert!(debug.contains("LoadStart"));
}

#[test]
fn test_webview_event_clone() {
    let e = WebViewEvent::LoadFailed("https://a.com".to_string(), "timeout".to_string());
    let cloned = e.clone();
    assert!(matches!(cloned, WebViewEvent::LoadFailed(u, m) if u == "https://a.com" && m == "timeout"));
}

#[test]
fn test_webview_config_clone() {
    let config = WebViewConfig::default();
    let cloned = config.clone();
    assert_eq!(config.width, cloned.width);
    assert_eq!(config.height, cloned.height);
}

// ── load_url end-to-end with complete_load ──

#[test]
fn test_webview_load_url_e2e_full_cycle() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        let label = match e {
            WebViewEvent::LoadStart(u) => format!("LoadStart({u})"),
            WebViewEvent::LoadEnd(u) => format!("LoadEnd({u})"),
            WebViewEvent::LoadFailed(u, m) => format!("LoadFailed({u},{m})"),
            WebViewEvent::TitleChanged(t) => format!("TitleChanged({t})"),
            WebViewEvent::UrlChanged(u) => format!("UrlChanged({u})"),
        };
        events_clone.borrow_mut().push(label);
    });

    // Step 1: load_url
    wv.load_url("https://example.com");
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://example.com"));
    assert!(wv.last_render().is_none());

    // Step 2: complete_load with HTML
    let html = "<html><head><title>Example</title></head><body><div>Content</div></body></html>";
    wv.complete_load(html, None);
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_some());
    assert!(wv.last_render().unwrap().timings.total_ms >= 0.0);

    // Verify event sequence
    let recorded = events.borrow();
    assert_eq!(recorded.len(), 3);
    assert_eq!(recorded[0], "LoadStart(https://example.com)");
    assert_eq!(recorded[1], "UrlChanged(https://example.com)");
    assert_eq!(recorded[2], "LoadEnd(https://example.com)");
}

#[test]
fn test_webview_load_url_e2e_fail_cycle() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        let label = match e {
            WebViewEvent::LoadStart(u) => format!("LoadStart({u})"),
            WebViewEvent::LoadEnd(u) => format!("LoadEnd({u})"),
            WebViewEvent::LoadFailed(u, m) => format!("LoadFailed({u},{m})"),
            WebViewEvent::TitleChanged(t) => format!("TitleChanged({t})"),
            WebViewEvent::UrlChanged(u) => format!("UrlChanged({u})"),
        };
        events_clone.borrow_mut().push(label);
    });

    wv.load_url("https://unreachable.invalid");
    wv.fail_load("DNS resolution failed");
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_none());

    let recorded = events.borrow();
    assert_eq!(recorded.len(), 3);
    assert_eq!(recorded[0], "LoadStart(https://unreachable.invalid)");
    assert_eq!(recorded[1], "UrlChanged(https://unreachable.invalid)");
    assert!(recorded[2].starts_with("LoadFailed(https://unreachable.invalid,DNS"));
}

// ════════════════════════════════════════════════════════════════
//  新增测试：Builder / Navigation / Script / Callback / State / Config
// ════════════════════════════════════════════════════════════════

// ── Builder: HTML content via builder + build-then-load ──

#[test]
fn test_webview_builder_then_load_html() {
    let mut wv = WebViewBuilder::new().width(1024).height(768).build();
    let result = wv.load_html("<html><body><p>Builder HTML</p></body></html>", None);
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());
    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);
}

#[test]
fn test_webview_builder_with_url_then_complete() {
    let mut wv = WebViewBuilder::new().url("https://builder-test.com").build();
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://builder-test.com"));
    wv.complete_load("<html><body>Loaded</body></html>", None);
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_some());
}

#[test]
fn test_webview_builder_transparent_with_url() {
    let mut wv = WebViewBuilder::new()
        .transparent(true)
        .url("https://transparent.com")
        .build();
    assert!(wv.config().transparent);
    assert_eq!(wv.url(), Some("https://transparent.com"));
    wv.complete_load("<html><body>Transparent</body></html>", None);
    assert!(!wv.is_loading());
}

#[test]
fn test_webview_builder_devtools_enabled() {
    let wv = WebViewBuilder::new().devtools(true).build();
    assert!(wv.config().devtools);
}

#[test]
fn test_webview_builder_user_agent_with_unicode() {
    let wv = WebViewBuilder::new().user_agent("ZeroBrowser/1.0 (日本語)").build();
    assert_eq!(wv.config().user_agent.as_deref(), Some("ZeroBrowser/1.0 (日本語)"));
}

// ── Navigation: reload semantics, navigation cycles ──

#[test]
fn test_webview_reload_via_load_url_same() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let ec = events.clone();
    wv.on_event(move |e| {
        let label = match e {
            WebViewEvent::LoadStart(u) => format!("LoadStart({u})"),
            WebViewEvent::LoadEnd(u) => format!("LoadEnd({u})"),
            WebViewEvent::UrlChanged(u) => format!("UrlChanged({u})"),
            WebViewEvent::LoadFailed(u, m) => format!("LoadFailed({u},{m})"),
            WebViewEvent::TitleChanged(t) => format!("TitleChanged({t})"),
        };
        ec.borrow_mut().push(label);
    });

    // First load + complete
    wv.load_url("https://page.com");
    wv.complete_load("<html><body>V1</body></html>", None);
    assert!(!wv.is_loading());

    // "Reload" — same URL
    wv.load_url("https://page.com");
    // Should be loading again; same URL => no UrlChanged
    assert!(wv.is_loading());
    wv.complete_load("<html><body>V2</body></html>", None);
    assert!(!wv.is_loading());

    let recorded = events.borrow();
    // Cycle 1: LoadStart, UrlChanged, LoadEnd
    // Cycle 2: LoadStart (no UrlChanged because same URL), LoadEnd
    assert_eq!(recorded.len(), 5);
    assert_eq!(recorded[0], "LoadStart(https://page.com)");
    assert_eq!(recorded[1], "UrlChanged(https://page.com)");
    assert_eq!(recorded[2], "LoadEnd(https://page.com)");
    assert_eq!(recorded[3], "LoadStart(https://page.com)");
    assert_eq!(recorded[4], "LoadEnd(https://page.com)");
}

#[test]
fn test_webview_navigate_forward_back_sequence() {
    let mut wv = WebView::new(WebViewConfig::default());
    let urls: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let uc = urls.clone();
    wv.on_event(move |e| {
        if let WebViewEvent::UrlChanged(u) = e {
            uc.borrow_mut().push(u.clone());
        }
    });

    wv.load_url("https://a.com");
    wv.complete_load("<html><body>A</body></html>", None);
    wv.load_url("https://b.com");
    wv.complete_load("<html><body>B</body></html>", None);
    // "Navigate back" — same as loading the old URL
    wv.load_url("https://a.com");
    wv.complete_load("<html><body>A</body></html>", None);

    assert_eq!(wv.url(), Some("https://a.com"));
    let recorded = urls.borrow();
    // UrlChanged fires when URL changes: a.com, b.com, a.com
    assert_eq!(recorded.len(), 3);
    assert_eq!(recorded[0], "https://a.com");
    assert_eq!(recorded[1], "https://b.com");
    assert_eq!(recorded[2], "https://a.com");
}

#[test]
fn test_webview_current_url_updated_after_load_url() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://first.com");
    assert_eq!(wv.url(), Some("https://first.com"));
    wv.complete_load("<html><body>A</body></html>", None);
    assert_eq!(wv.url(), Some("https://first.com"));

    wv.load_url("https://second.com");
    assert_eq!(wv.url(), Some("https://second.com"));
}

#[test]
fn prepare_document_state_clears_last_render_for_new_navigation() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>old</body></html>", None);
    assert!(wv.last_render().is_some());

    wv.prepare_document_state("https://next.example");
    assert!(wv.last_render().is_none());
    assert!(wv.is_loading());
}

#[test]
fn test_webview_multiple_load_url_without_complete() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://a.com");
    assert!(wv.is_loading());
    // Overwrite with a new URL before completing
    wv.load_url("https://b.com");
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://b.com"));
    // Last URL wins
    wv.complete_load("<html><body>B</body></html>", None);
    assert_eq!(wv.url(), Some("https://b.com"));
}
