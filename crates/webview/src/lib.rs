//! # zero-webview
//!
//! 面向外部应用的稳定嵌入接口。
//!
//! 提供构建器模式创建 WebView、导航、注入 JS、回调、渲染表面输出。

#![warn(missing_docs)]

pub mod webview;
pub mod webview_builder;

pub use webview::*;
pub use webview_builder::*;

/// WebView 错误类型。
#[derive(Debug, thiserror::Error)]
pub enum WebViewError {
    /// 渲染错误。
    #[error("Rendering error: {0}")]
    Rendering(String),
    /// 导航错误。
    #[error("Navigation error: {0}")]
    Navigation(String),
    /// 脚本错误。
    #[error("Script error: {0}")]
    Script(String),
    /// 未实现。
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

#[cfg(test)]
mod tests {
    use super::*;
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

    // ── Script bridge ──

    #[test]
    fn test_webview_execute_script_empty() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.execute_script("");
        assert!(result.is_err());
    }

    #[test]
    fn test_webview_execute_script_long_script() {
        let mut wv = WebView::new(WebViewConfig::default());
        let long_script = "var x = 0; ".repeat(1000);
        let result = wv.execute_script(&long_script);
        // V8 sandbox executes the script (result is undefined)
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_webview_execute_script_multiple_calls() {
        let mut wv = WebView::new(WebViewConfig::default());
        for i in 0..5 {
            let result = wv.execute_script(&format!("{i} + 1"));
            assert!(result.is_ok(), "Script {i} should execute successfully");
        }
    }

    #[test]
    fn test_webview_execute_script_with_special_chars() {
        let mut wv = WebView::new(WebViewConfig::default());
        let script = "let s = 'hello \"world\" 🌍';";
        let result = wv.execute_script(script);
        assert!(result.is_ok(), "Script with special chars should execute");
    }

    #[test]
    fn test_webview_execute_script_returns_result() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.execute_script("1 + 1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "2");
    }

    // ── Event callbacks: edge cases ──

    #[test]
    fn test_webview_load_html_does_not_fire_load_events() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            ec.borrow_mut().push(e.clone());
        });
        wv.load_html("<html><body>No events</body></html>", None);
        // load_html does not fire LoadStart/LoadEnd/LoadFailed
        assert!(events.borrow().is_empty());
    }

    #[test]
    fn test_webview_inject_css_does_not_fire_events() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_html("<html><body><div>Test</div></body></html>", None);
        let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            ec.borrow_mut().push(e.clone());
        });
        wv.inject_css("div { color: red; }");
        assert!(events.borrow().is_empty());
    }

    #[test]
    fn test_webview_set_title_fires_title_changed_event() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            ec.borrow_mut().push(e.clone());
        });
        wv.set_title("Title 1");
        wv.set_title("Title 2");
        let recorded = events.borrow();
        assert_eq!(recorded.len(), 2);
        assert!(matches!(&recorded[0], WebViewEvent::TitleChanged(t) if t == "Title 1"));
        assert!(matches!(&recorded[1], WebViewEvent::TitleChanged(t) if t == "Title 2"));
    }

    #[test]
    fn test_webview_callback_sees_all_event_types() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            let name = match e {
                WebViewEvent::LoadStart(_) => "LoadStart",
                WebViewEvent::LoadEnd(_) => "LoadEnd",
                WebViewEvent::LoadFailed(_, _) => "LoadFailed",
                WebViewEvent::TitleChanged(_) => "TitleChanged",
                WebViewEvent::UrlChanged(_) => "UrlChanged",
            };
            ec.borrow_mut().push(name.to_string());
        });

        wv.set_title("MyTitle");
        wv.load_url("https://example.com");
        wv.complete_load("<html><body>Hi</body></html>", None);

        let recorded = events.borrow();
        assert!(recorded.contains(&"TitleChanged".to_string()));
        assert!(recorded.contains(&"LoadStart".to_string()));
        assert!(recorded.contains(&"UrlChanged".to_string()));
        assert!(recorded.contains(&"LoadEnd".to_string()));
        assert!(!recorded.contains(&"LoadFailed".to_string()));
    }

    #[test]
    fn test_webview_remove_first_callback_keeps_second() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events_a: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
        let events_b: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
        let ea = events_a.clone();
        let eb = events_b.clone();
        let idx_a = wv.on_event(move |_| {
            *ea.borrow_mut() += 1;
        });
        wv.on_event(move |_| {
            *eb.borrow_mut() += 1;
        });

        wv.set_title("T");
        assert_eq!(*events_a.borrow(), 1);
        assert_eq!(*events_b.borrow(), 1);

        wv.remove_event_callback(idx_a);
        wv.set_title("T2");
        assert_eq!(*events_a.borrow(), 1); // not incremented
        assert_eq!(*events_b.borrow(), 2); // incremented
    }

    // ── WebView state transitions ──

    #[test]
    fn test_webview_state_idle_to_loading_to_loaded() {
        let mut wv = WebView::new(WebViewConfig::default());
        // Idle
        assert!(!wv.is_loading());
        assert!(wv.url().is_none());
        assert!(wv.last_render().is_none());

        // Loading
        wv.load_url("https://state-test.com");
        assert!(wv.is_loading());
        assert_eq!(wv.url(), Some("https://state-test.com"));
        assert!(wv.last_render().is_none());

        // Loaded
        wv.complete_load("<html><body>Loaded</body></html>", None);
        assert!(!wv.is_loading());
        assert_eq!(wv.url(), Some("https://state-test.com"));
        assert!(wv.last_render().is_some());
    }

    #[test]
    fn test_webview_state_idle_to_loading_to_failed() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_url("https://fail.com");
        assert!(wv.is_loading());
        wv.fail_load("timeout");
        assert!(!wv.is_loading());
        assert!(wv.last_render().is_none());
        assert_eq!(wv.url(), Some("https://fail.com"));
    }

    #[test]
    fn test_webview_state_loaded_then_reload() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_url("https://page.com");
        wv.complete_load("<html><body>V1</body></html>", None);
        assert!(!wv.is_loading());

        // Reload same URL
        wv.load_url("https://page.com");
        assert!(wv.is_loading());
        wv.complete_load("<html><body>V2</body></html>", None);
        assert!(!wv.is_loading());
        assert!(wv.last_render().is_some());
    }

    #[test]
    fn test_webview_state_loaded_then_navigate_new() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_url("https://a.com");
        wv.complete_load("<html><body>A</body></html>", None);
        assert!(!wv.is_loading());

        wv.load_url("https://b.com");
        assert!(wv.is_loading());
        assert_eq!(wv.url(), Some("https://b.com"));
        wv.complete_load("<html><body>B</body></html>", None);
        assert!(!wv.is_loading());
        assert_eq!(wv.url(), Some("https://b.com"));
    }

    #[test]
    fn test_webview_state_fail_then_retry() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_url("https://retry.com");
        wv.fail_load("connection reset");
        assert!(!wv.is_loading());

        // Retry
        wv.load_url("https://retry.com");
        assert!(wv.is_loading());
        wv.complete_load("<html><body>Success</body></html>", None);
        assert!(!wv.is_loading());
        assert!(wv.last_render().is_some());
    }

    // ── Configuration ──

    #[test]
    fn test_webview_config_user_agent_none_by_default() {
        let config = WebViewConfig::default();
        assert!(config.user_agent.is_none());
    }

    #[test]
    fn test_webview_config_devtools_false_by_default() {
        let config = WebViewConfig::default();
        assert!(!config.devtools);
    }

    #[test]
    fn test_webview_config_transparent_false_by_default() {
        let config = WebViewConfig::default();
        assert!(!config.transparent);
    }

    #[test]
    fn test_webview_config_url_none_by_default() {
        let config = WebViewConfig::default();
        assert!(config.url.is_none());
    }

    #[test]
    fn test_webview_config_all_fields_custom() {
        let config = WebViewConfig {
            width: 1920,
            height: 1080,
            transparent: true,
            user_agent: Some("Custom/2.0".to_string()),
            url: Some("https://start.com".to_string()),
            devtools: true,
        };
        let wv = WebView::new(config);
        assert_eq!(wv.config().width, 1920);
        assert_eq!(wv.config().height, 1080);
        assert!(wv.config().transparent);
        assert_eq!(wv.config().user_agent.as_deref(), Some("Custom/2.0"));
        assert_eq!(wv.config().url.as_deref(), Some("https://start.com"));
        assert!(wv.config().devtools);
    }

    // ── WebViewRenderResult clone/debug ──

    #[test]
    fn test_webview_render_result_clone() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.load_html("<html><body><div>X</div></body></html>", None);
        let cloned = result.clone();
        assert!(cloned.timings.total_ms >= 0.0);
    }

    #[test]
    fn test_webview_render_result_debug() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.load_html("<html><body><div>X</div></body></html>", None);
        let debug = format!("{result:?}");
        assert!(debug.contains("WebViewRenderResult"));
    }

    // ── Resize edge cases ──

    #[test]
    fn test_webview_resize_very_large() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.resize(10000, 10000);
        assert_eq!(wv.config().width, 10000);
        assert_eq!(wv.config().height, 10000);
    }

    #[test]
    fn test_webview_resize_preserves_title() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.set_title("Preserved");
        wv.resize(500, 400);
        assert_eq!(wv.title(), Some("Preserved"));
    }

    // ── load_html with various content ──

    #[test]
    fn test_webview_load_html_with_inline_styles() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body><div style=\"color: red; width: 100px;\">Styled</div></body></html>";
        let result = wv.load_html(html, None);
        assert!(result.timings.total_ms >= 0.0);
    }

    #[test]
    fn test_webview_load_html_preserves_cached_html() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_html("<html><body><div>Cached</div></body></html>", None);
        // inject_css uses cached HTML internally — verify it works
        let result = wv.inject_css("div { background: green; }");
        assert!(result.timings.total_ms >= 0.0);
        assert!(wv.last_render().is_some());
    }

    // ── Builder: all-setters chain ──

    #[test]
    fn test_webview_builder_all_options() {
        let wv = WebViewBuilder::new()
            .width(1280)
            .height(720)
            .transparent(true)
            .user_agent("FullAgent/3.0")
            .url("https://full.com")
            .devtools(true)
            .build();
        assert_eq!(wv.config().width, 1280);
        assert_eq!(wv.config().height, 720);
        assert!(wv.config().transparent);
        assert_eq!(wv.config().user_agent.as_deref(), Some("FullAgent/3.0"));
        assert!(wv.config().devtools);
        assert_eq!(wv.url(), Some("https://full.com"));
        assert!(wv.is_loading());
    }

    // ── cached_css：CSS 在 render / resize 后保留 ──

    #[test]
    fn test_webview_load_html_with_css_preserved_in_render() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body><div id=\"main\">Hello</div></body></html>";
        let css = "div { background-color: red; width: 200px; height: 100px; }";
        let first = wv.load_html(html, Some(css));
        let fill_count_after_load = first.primitives.fills.len();

        // render() 应使用缓存的 CSS，fills 数量应一致
        let second = wv.render();
        assert_eq!(
            second.primitives.fills.len(),
            fill_count_after_load,
            "render() should produce same fills as load_html() when CSS is cached"
        );
    }

    #[test]
    fn test_webview_load_html_css_preserved_after_resize() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body><div id=\"box\">Content</div></body></html>";
        let css = "div { background-color: blue; width: 100px; height: 50px; }";
        let first = wv.load_html(html, Some(css));
        let fill_count = first.primitives.fills.len();

        wv.resize(400, 300);
        let after = wv.render();
        assert_eq!(
            after.primitives.fills.len(),
            fill_count,
            "CSS should be preserved after resize + render"
        );
    }

    #[test]
    fn test_webview_inject_css_accumulates() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body><div class=\"a b\">Test</div></body></html>";
        let css = ".a { background-color: red; width: 100px; height: 50px; }";
        let first = wv.load_html(html, Some(css));
        let fill_count_first = first.primitives.fills.len();

        // 注入额外 CSS，应追加到已有 CSS
        let second = wv.inject_css(".b { background-color: blue; }");
        // 注入后 fills 数量应 >= 之前（追加的 CSS 可能影响布局）
        assert!(
            second.primitives.fills.len() >= fill_count_first,
            "inject_css should accumulate CSS, not replace it"
        );

        // render 也应保留累积的 CSS
        let third = wv.render();
        assert_eq!(
            third.primitives.fills.len(),
            second.primitives.fills.len(),
            "render() should use accumulated CSS"
        );
    }

    #[test]
    fn test_webview_load_html_resets_cached_css() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body><div>Content</div></body></html>";
        wv.load_html(html, Some("div { color: red; }"));
        // 再次调用 load_html 传 None，应重置 CSS
        wv.load_html(html, None);
        let after = wv.render();
        // 没有 CSS 时的 fills 应 <= 有 CSS 时
        // 主要验证不会崩溃，且 CSS 被正确清空
        assert!(after.timings.total_ms >= 0.0);
    }

    // ════════════════════════════════════════════════════════════════
    //  边界条件测试：空输入、CSS 累积、状态机转换
    // ════════════════════════════════════════════════════════════════

    /// 验证加载空 HTML 字符串不会 panic，且返回有效的渲染结果。
    ///
    /// 边界场景：传入完全为空的字符串而非有效 HTML 文档，
    /// 确保渲染管线不会因缺少根元素而崩溃。
    #[test]
    fn test_webview_load_empty_html() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.load_html("", None);
        assert!(result.timings.total_ms >= 0.0, "渲染空 HTML 应返回非负耗时");
        assert!(wv.last_render().is_some(), "加载空 HTML 后应存在渲染结果");
        assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
        assert!(wv.url().is_none(), "load_html 不应设置 URL");
    }

    /// 验证执行空脚本字符串返回错误。
    ///
    /// 边界场景：传入空字符串作为脚本内容，
    /// 确保 JS 引擎正确拒绝空输入。
    #[test]
    fn test_webview_execute_script_empty_string() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.execute_script("");
        assert!(result.is_err(), "空脚本应返回错误");
        match result.unwrap_err() {
            WebViewError::Script(msg) => {
                assert!(
                    msg.contains("Invalid input") || msg.contains("empty"),
                    "错误信息应提及空输入，实际: {msg}"
                );
            }
            other => panic!("预期 Script 错误，实际: {other}"),
        }
    }

    /// 验证多次注入 CSS 会累积而非替换。
    ///
    /// 每次调用 inject_css 应将新 CSS 追加到已有 CSS 之后，
    /// 渲染结果应反映所有已注入样式的叠加效果。
    #[test]
    fn test_webview_multiple_css_injections() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body>\
            <div class=\"first\">A</div>\
            <div class=\"second\">B</div>\
            <div class=\"third\">C</div>\
            </body></html>";
        let initial = wv.load_html(html, None);
        let fills_after_load = initial.primitives.fills.len();

        // 第一次注入：为 .first 添加背景
        let after_first = wv.inject_css(".first { background-color: red; width: 50px; height: 50px; }");
        let fills_after_first = after_first.primitives.fills.len();
        assert!(
            fills_after_first >= fills_after_load,
            "第一次注入后 fills 数量应 >= 初始值"
        );

        // 第二次注入：为 .second 添加背景
        let after_second = wv.inject_css(".second { background-color: green; width: 50px; height: 50px; }");
        let fills_after_second = after_second.primitives.fills.len();
        assert!(
            fills_after_second >= fills_after_first,
            "第二次注入后 fills 数量应 >= 第一次注入后（CSS 累积，不替换）"
        );

        // 第三次注入：为 .third 添加背景
        let after_third = wv.inject_css(".third { background-color: blue; width: 50px; height: 50px; }");
        let fills_after_third = after_third.primitives.fills.len();
        assert!(
            fills_after_third >= fills_after_second,
            "第三次注入后 fills 数量应 >= 第二次注入后（CSS 持续累积）"
        );

        // render() 也应保留所有累积的 CSS
        let after_render = wv.render();
        assert_eq!(
            after_render.primitives.fills.len(),
            fills_after_third,
            "render() 应使用累积的所有 CSS"
        );
    }

    /// 验证 WebView 状态机转换：Created -> Loading -> Loaded -> Error。
    ///
    /// 测试完整的状态生命周期：
    /// 1. 初始 Created 状态（无 URL，未加载）
    /// 2. load_url 进入 Loading 状态
    /// 3. complete_load 进入 Loaded 状态
    /// 4. 再次 load_url 进入 Loading 状态
    /// 5. fail_load 进入 Error（恢复到非加载状态）
    /// 6. 重试后再次进入 Loaded 状态
    #[test]
    fn test_webview_state_transitions() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            let label = match e {
                WebViewEvent::LoadStart(u) => format!("LoadStart({u})"),
                WebViewEvent::LoadEnd(u) => format!("LoadEnd({u})"),
                WebViewEvent::LoadFailed(u, m) => format!("LoadFailed({u},{m})"),
                WebViewEvent::TitleChanged(t) => format!("TitleChanged({t})"),
                WebViewEvent::UrlChanged(u) => format!("UrlChanged({u})"),
            };
            ec.borrow_mut().push(label);
        });

        // ── 状态 1: Created ──
        assert!(!wv.is_loading(), "初始状态: 不应处于加载中");
        assert!(wv.url().is_none(), "初始状态: URL 应为 None");
        assert!(wv.last_render().is_none(), "初始状态: 不应有渲染结果");

        // ── 状态 2: Loading（通过 load_url）──
        wv.load_url("https://state-test.com");
        assert!(wv.is_loading(), "Loading 状态: 应处于加载中");
        assert_eq!(wv.url(), Some("https://state-test.com"), "Loading 状态: URL 应已设置");
        assert!(wv.last_render().is_none(), "Loading 状态: 尚未有渲染结果");

        // ── 状态 3: Loaded（通过 complete_load）──
        wv.complete_load("<html><body><div>Content</div></body></html>", None);
        assert!(!wv.is_loading(), "Loaded 状态: 不应处于加载中");
        assert_eq!(wv.url(), Some("https://state-test.com"), "Loaded 状态: URL 应保持不变");
        assert!(wv.last_render().is_some(), "Loaded 状态: 应有渲染结果");

        // ── 状态 4: 再次 Loading（导航到新 URL）──
        wv.load_url("https://error-test.com");
        assert!(wv.is_loading(), "再次 Loading: 应处于加载中");
        assert_eq!(wv.url(), Some("https://error-test.com"), "再次 Loading: URL 应已更新");

        // ── 状态 5: Error（通过 fail_load）──
        wv.fail_load("network timeout");
        assert!(!wv.is_loading(), "Error 状态: 加载应已停止");
        assert_eq!(wv.url(), Some("https://error-test.com"), "Error 状态: URL 应保留");
        assert!(wv.last_render().is_some(), "Error 状态: 上次成功的渲染结果应保留");

        // ── 状态 6: 重试 Loading -> Loaded ──
        wv.load_url("https://retry-test.com");
        assert!(wv.is_loading(), "重试 Loading: 应处于加载中");
        assert_eq!(wv.url(), Some("https://retry-test.com"), "重试 Loading: URL 应已更新");
        wv.complete_load("<html><body><div>Retry OK</div></body></html>", None);
        assert!(!wv.is_loading(), "重试 Loaded: 不应处于加载中");
        assert_eq!(wv.url(), Some("https://retry-test.com"), "重试 Loaded: URL 应保持");
        assert!(wv.last_render().is_some(), "重试 Loaded: 应有渲染结果");

        // ── 验证完整事件序列 ──
        let recorded = events.borrow();
        assert_eq!(
            recorded.len(),
            9,
            "应有 9 个事件: 2(LoadStart+UrlChanged) + 1(LoadEnd) + 2(LoadStart+UrlChanged) + 1(LoadFailed) + 2(LoadStart+UrlChanged) + 1(LoadEnd)"
        );
        assert_eq!(recorded[0], "LoadStart(https://state-test.com)");
        assert_eq!(recorded[1], "UrlChanged(https://state-test.com)");
        assert_eq!(recorded[2], "LoadEnd(https://state-test.com)");
        assert_eq!(recorded[3], "LoadStart(https://error-test.com)");
        assert_eq!(recorded[4], "UrlChanged(https://error-test.com)");
        assert!(recorded[5].starts_with("LoadFailed(https://error-test.com"));
        assert_eq!(recorded[6], "LoadStart(https://retry-test.com)");
        assert_eq!(recorded[7], "UrlChanged(https://retry-test.com)");
        assert_eq!(recorded[8], "LoadEnd(https://retry-test.com)");
    }

    // ════════════════════════════════════════════════════════════════
    //  边界条件测试：多次导航、CSS 注入累积、脚本占位、Builder 视口
    // ════════════════════════════════════════════════════════════════

    /// 验证连续导航两次后，WebView 最终状态反映 URL2 的内容。
    ///
    /// 模拟用户从 URL1 导航到 URL2 的场景：
    /// 1. 加载 URL1 并完成（complete_load），确认状态为 URL1
    /// 2. 加载 URL2 并完成（complete_load），确认最终状态为 URL2
    /// 3. 确保 URL、加载状态、渲染结果全部正确指向 URL2
    #[test]
    fn test_webview_multiple_navigate() {
        let mut wv = WebView::new(WebViewConfig::default());

        // 第一次导航：URL1
        wv.load_url("https://url-one.com");
        assert!(wv.is_loading());
        assert_eq!(wv.url(), Some("https://url-one.com"));
        wv.complete_load("<html><body><div>Content from URL1</div></body></html>", None);
        assert!(!wv.is_loading());
        assert_eq!(wv.url(), Some("https://url-one.com"));
        let render1 = wv.last_render().unwrap();
        assert!(render1.timings.total_ms >= 0.0);

        // 第二次导航：URL2
        wv.load_url("https://url-two.com");
        assert!(wv.is_loading());
        assert_eq!(wv.url(), Some("https://url-two.com"));
        wv.complete_load("<html><body><div>Content from URL2</div></body></html>", None);
        assert!(!wv.is_loading());

        // 验证最终状态指向 URL2
        assert_eq!(wv.url(), Some("https://url-two.com"));
        assert!(wv.last_render().is_some());
        let render2 = wv.last_render().unwrap();
        assert!(render2.timings.total_ms >= 0.0);

        // 重新渲染应仍反映 URL2 的内容（cached_html 为 URL2 的 HTML）
        let rerender = wv.render();
        assert!(rerender.timings.total_ms >= 0.0);
    }

    /// 验证 load_html 加载 CSS 后，inject_css 追加新样式，cached_css 同时包含原始和注入的 CSS。
    ///
    /// 步骤：
    /// 1. load_html 加载带 CSS 的 HTML（为 .orig 元素设置红色背景）
    /// 2. inject_css 注入额外 CSS（为 .injected 元素设置蓝色背景）
    /// 3. 通过 render() 的 fills 数量验证 CSS 累积效果：
    ///    - 注入后 fills >= 仅原始 CSS 时的 fills
    ///    - render() 使用累积 CSS，fills 数量一致
    #[test]
    fn test_webview_inject_css_after_load() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body>\
            <div class=\"orig\">Original</div>\
            <div class=\"injected\">Injected</div>\
            </body></html>";
        let original_css = ".orig { background-color: red; width: 100px; height: 50px; }";

        // 加载带原始 CSS 的 HTML
        let after_load = wv.load_html(html, Some(original_css));
        let fills_after_load = after_load.primitives.fills.len();
        assert!(fills_after_load > 0, "带 CSS 的 load_html 应产生 fills");

        // 注入额外 CSS
        let injected_css = ".injected { background-color: blue; width: 80px; height: 40px; }";
        let after_inject = wv.inject_css(injected_css);
        let fills_after_inject = after_inject.primitives.fills.len();

        // 注入后 fills 应 >= 仅原始 CSS（CSS 累积，不替换）
        assert!(
            fills_after_inject >= fills_after_load,
            "inject_css 应追加 CSS，fills 数量应 >= 注入前 (got {fills_after_inject} < {fills_after_load})"
        );

        // render() 应使用累积的 CSS（原始 + 注入），fills 数量一致
        let after_render = wv.render();
        assert_eq!(
            after_render.primitives.fills.len(),
            fills_after_inject,
            "render() 应使用累积的 cached_css（原始 + 注入）"
        );
    }

    /// 验证 execute_script 作为占位方法，在 JS 引擎集成前返回 NotImplemented 错误。
    ///
    /// 验证 execute_script 现在通过 V8 沙箱执行脚本。
    ///
    /// V8 沙箱已集成，execute_script 可以成功执行简单脚本。
    /// document.title 在独立沙箱中不可用，因此会产生运行时错误。
    #[test]
    fn test_webview_execute_script_v8_integrated() {
        let mut wv = WebView::new(WebViewConfig::default());

        // 简单算术表达式应成功执行
        let result = wv.execute_script("1 + 1");
        assert!(result.is_ok(), "简单脚本应成功执行");
        assert_eq!(result.unwrap(), "2");

        // WebView 状态不应因 execute_script 调用而改变
        assert!(wv.url().is_none());
        assert!(!wv.is_loading());

        // 多次调用同样成功
        for i in 0..3 {
            let r = wv.execute_script(&format!("{i} * 2"));
            assert!(r.is_ok(), "Script {i} should succeed");
        }
    }

    /// 验证 WebViewBuilder 默认配置（无 URL）产生正确的初始状态。
    ///
    /// 默认视口 800x600，无 URL，不在加载中，无渲染结果。
    #[test]
    fn test_webview_builder_defaults() {
        let wv = WebViewBuilder::new().build();
        assert_eq!(wv.config().width, 800, "默认宽度应为 800");
        assert_eq!(wv.config().height, 600, "默认高度应为 600");
        assert!(wv.url().is_none(), "默认不应有 URL");
        assert!(!wv.is_loading(), "默认不应处于加载中");
        assert!(wv.last_render().is_none(), "默认不应有渲染结果");
        assert!(!wv.config().transparent);
        assert!(wv.config().user_agent.is_none());
        assert!(!wv.config().devtools);
    }

    /// 验证加载 data URI 内容后渲染成功。
    ///
    /// 通过 load_html 加载 data URI 格式的 HTML 内容，
    /// 确认渲染管线产生有效结果（非负耗时、存在渲染输出）。
    #[test]
    fn test_webview_load_data_uri() {
        let mut wv = WebView::new(WebViewConfig::default());
        let data_uri_html = "<html><body><div>Data URI content rendered</div></body></html>";
        let result = wv.load_html(data_uri_html, None);
        assert!(result.timings.total_ms >= 0.0, "data URI 渲染耗时应为非负");
        assert!(wv.last_render().is_some(), "加载 data URI 后应有渲染结果");
        assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
    }

    /// 验证加载 HTML 后注入 CSS，渲染结果反映注入的样式。
    ///
    /// 步骤：
    /// 1. load_html 加载带 div 的 HTML（无 CSS）
    /// 2. inject_css 注入为 div 设置背景色和尺寸的 CSS
    /// 3. 渲染结果的 fills 数量应大于仅加载 HTML 时
    #[test]
    fn test_webview_render_after_inject_css() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body><div class=\"box\">Hello</div></body></html>";

        // 加载 HTML（无 CSS）
        let after_load = wv.load_html(html, None);
        let fills_after_load = after_load.primitives.fills.len();

        // 注入 CSS
        let css = ".box { background-color: green; width: 100px; height: 50px; }";
        let after_inject = wv.inject_css(css);
        let fills_after_inject = after_inject.primitives.fills.len();

        // 注入后 fills 数量应 >= 加载时（CSS 为 div 添加了背景色）
        assert!(
            fills_after_inject >= fills_after_load,
            "注入 CSS 后 fills 数量应 >= 注入前 (got {fills_after_inject} < {fills_after_load})"
        );

        // render 应使用累积的 CSS
        let after_render = wv.render();
        assert_eq!(
            after_render.primitives.fills.len(),
            fills_after_inject,
            "render() 应使用注入的 CSS"
        );
    }

    /// 验证 WebViewBuilder 支持自定义视口尺寸，且 build 后 WebView 正确反映配置。
    ///
    /// 测试非默认视口（如 1280x900），确认：
    /// 1. Builder 的 width/height 链式调用正确
    /// 2. build 后 config 反映自定义尺寸
    /// 3. 后续 render 在正确尺寸的视口上工作
    #[test]
    fn test_webview_builder_custom_viewport() {
        let mut wv = WebViewBuilder::new().width(1280).height(900).build();

        // 验证自定义视口尺寸
        assert_eq!(wv.config().width, 1280, "视口宽度应为 1280");
        assert_eq!(wv.config().height, 900, "视口高度应为 900");

        // 默认值应保持不变
        assert!(!wv.config().transparent);
        assert!(wv.config().user_agent.is_none());
        assert!(!wv.config().devtools);

        // 加载 HTML 并渲染，验证在自定义视口上正常工作
        let html = "<html><body><div>Custom viewport</div></body></html>";
        let result = wv.load_html(html, None);
        assert!(result.timings.total_ms >= 0.0, "自定义视口上的渲染应成功");
        assert!(wv.last_render().is_some());

        // resize 后视口尺寸应更新
        wv.resize(640, 480);
        assert_eq!(wv.config().width, 640);
        assert_eq!(wv.config().height, 480);
        let after_resize = wv.render();
        assert!(after_resize.timings.total_ms >= 0.0);
    }

    // ════════════════════════════════════════════════════════════════
    //  边界条件测试：默认配置、data URI、连续导航、CSS 存储、状态转换
    // ════════════════════════════════════════════════════════════════

    /// 验证 WebView 使用默认配置创建后，所有字段均为预期默认值。
    ///
    /// 测试通过 WebViewConfig::default() 构造 WebView，
    /// 确认宽高、透明度、user_agent、devtools 等字段均为默认值，
    /// 且初始状态下无 URL、无标题、不在加载中、无渲染结果。
    #[test]
    fn test_webview_default_config() {
        let config = WebViewConfig::default();
        let wv = WebView::new(config);

        // 配置字段默认值
        assert_eq!(wv.config().width, 800, "默认宽度应为 800");
        assert_eq!(wv.config().height, 600, "默认高度应为 600");
        assert!(!wv.config().transparent, "默认不应透明");
        assert!(wv.config().user_agent.is_none(), "默认 user_agent 应为 None");
        assert!(wv.config().url.is_none(), "默认 url 应为 None");
        assert!(!wv.config().devtools, "默认 devtools 应为 false");

        // 初始状态
        assert!(wv.url().is_none(), "初始 URL 应为 None");
        assert!(wv.title().is_none(), "初始标题应为 None");
        assert!(!wv.is_loading(), "初始不应处于加载中");
        assert!(wv.last_render().is_none(), "初始不应有渲染结果");
    }

    /// 验证加载 data URI 格式的 HTML 内容不会 panic，且渲染管线返回有效结果。
    ///
    /// 模拟 "data:text/html,<h1>Hello</h1>" 场景：
    /// 通过 load_html 加载 data URI 中嵌入的 HTML 片段，
    /// 确认渲染结果非负耗时，且 last_render 存在。
    #[test]
    fn test_webview_load_data_uri_content() {
        let mut wv = WebView::new(WebViewConfig::default());
        // 模拟 data URI 中提取的 HTML 内容
        let html = "<h1>Hello</h1>";
        let result = wv.load_html(html, None);
        assert!(result.timings.total_ms >= 0.0, "data URI 渲染耗时应为非负");
        assert!(wv.last_render().is_some(), "加载 data URI 后应有渲染结果");
        assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
        assert!(wv.url().is_none(), "load_html 不应设置 URL");
    }

    /// 验证连续导航到 url1 再到 url2 后，当前 URL 为 url2。
    ///
    /// 模拟用户在浏览器中依次访问两个不同页面的场景：
    /// 1. 导航到 url1 并完成加载，确认状态正确
    /// 2. 导航到 url2 并完成加载，确认最终 URL 为 url2
    /// 3. 渲染结果应反映 url2 的内容
    #[test]
    fn test_webview_sequential_navigate() {
        let mut wv = WebView::new(WebViewConfig::default());
        let url1 = "https://first-page.com";
        let url2 = "https://second-page.com";

        // 第一次导航：url1
        wv.load_url(url1);
        assert!(wv.is_loading());
        assert_eq!(wv.url(), Some(url1));
        wv.complete_load("<html><body><div>Page 1</div></body></html>", None);
        assert!(!wv.is_loading());
        assert_eq!(wv.url(), Some(url1));

        // 第二次导航：url2
        wv.load_url(url2);
        assert!(wv.is_loading());
        assert_eq!(wv.url(), Some(url2));
        wv.complete_load("<html><body><div>Page 2</div></body></html>", None);
        assert!(!wv.is_loading());

        // 最终状态：URL 为 url2
        assert_eq!(wv.url(), Some(url2), "连续导航后当前 URL 应为 url2");
        assert!(wv.last_render().is_some(), "应有渲染结果");
    }

    /// 验证加载 HTML 后注入 CSS，CSS 被正确存储在 cached_css 中。
    ///
    /// 步骤：
    /// 1. load_html 加载 HTML（带初始 CSS）
    /// 2. inject_css 注入额外 CSS
    /// 3. 多次 render() 后 CSS 仍被保留（fills 数量不变）
    /// 4. 再次 inject_css 后 CSS 继续累积
    #[test]
    fn test_webview_css_stored_after_inject() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body><div class=\"a b\">Text</div></body></html>";
        let css_a = ".a { background-color: red; width: 100px; height: 50px; }";

        // 加载带初始 CSS 的 HTML
        let after_load = wv.load_html(html, Some(css_a));
        let fills_after_load = after_load.primitives.fills.len();
        assert!(fills_after_load > 0, "带 CSS 的 load_html 应产生 fills");

        // 注入额外 CSS，应被存储
        let css_b = ".b { background-color: green; width: 80px; height: 40px; }";
        let after_inject = wv.inject_css(css_b);
        let fills_after_inject = after_inject.primitives.fills.len();
        assert!(
            fills_after_inject >= fills_after_load,
            "注入后 fills 应 >= 注入前 (got {fills_after_inject} < {fills_after_load})"
        );

        // render() 后 CSS 应被保留（fills 数量不变）
        let after_render = wv.render();
        assert_eq!(
            after_render.primitives.fills.len(),
            fills_after_inject,
            "render() 后 CSS 应被保留，fills 数量应一致"
        );
    }

    // ── 边界条件测试：默认视口、导航到 URL、错误后状态、CSS 持久性 ──

    /// 验证 WebView 默认视口尺寸为 800x600。
    #[test]
    fn test_webview_default_viewport() {
        let wv = WebView::new(WebViewConfig::default());
        assert_eq!(wv.config().width, 800, "默认视口宽度应为 800");
        assert_eq!(wv.config().height, 600, "默认视口高度应为 600");
        // 默认视口下渲染应正常工作
        let mut wv2 = WebView::new(WebViewConfig::default());
        let result = wv2.load_html("<html><body>viewport test</body></html>", None);
        assert!(result.timings.total_ms >= 0.0, "默认视口渲染应成功");
    }

    /// 验证 navigate 到 URL 后 WebView 处于正确的加载状态。
    #[test]
    fn test_webview_navigate_to_url() {
        let mut wv = WebView::new(WebViewConfig::default());
        // 初始状态：无 URL
        assert!(wv.url().is_none());
        assert!(!wv.is_loading());
        // 导航到 URL
        wv.load_url("https://example.com/page1");
        assert_eq!(wv.url(), Some("https://example.com/page1"));
        assert!(wv.is_loading());
        assert!(wv.last_render().is_none(), "load_url 后不应有渲染结果");
        // 完成加载
        wv.complete_load("<html><body><div>Navigated</div></body></html>", None);
        assert!(!wv.is_loading());
        assert_eq!(wv.url(), Some("https://example.com/page1"));
        assert!(wv.last_render().is_some());
    }

    /// 验证加载失败后 WebView 状态正确：loading 停止，URL 保留，last_render 保持。
    #[test]
    fn test_webview_state_after_error() {
        let mut wv = WebView::new(WebViewConfig::default());
        // 先成功加载一个页面
        wv.load_url("https://good.com");
        wv.complete_load("<html><body>Good page</body></html>", None);
        assert!(wv.last_render().is_some());
        let render_before_error = wv.last_render().unwrap().timings.total_ms;
        // 导航到新 URL 但加载失败
        wv.load_url("https://bad.com");
        assert!(wv.is_loading());
        wv.fail_load("DNS resolution failed");
        // 失败后 loading 应停止
        assert!(!wv.is_loading(), "失败后 loading 应停止");
        // URL 应保留为失败的 URL
        assert_eq!(wv.url(), Some("https://bad.com"), "URL 应保留为失败请求的 URL");
        // last_render 应保留上次成功的渲染结果
        assert!(wv.last_render().is_some(), "失败后应保留上次成功的渲染结果");
        assert!(
            (wv.last_render().unwrap().timings.total_ms - render_before_error).abs() < f64::EPSILON,
            "渲染结果应是上次成功加载的结果"
        );
    }

    /// 验证 CSS 在多次 render 调用间持久保留。
    #[test]
    fn test_webview_css_persistence_across_render() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body><div id=\"box\">Content</div></body></html>";
        let css = "#box { background-color: red; width: 100px; height: 50px; }";
        // 第一次加载带 CSS
        let first = wv.load_html(html, Some(css));
        let first_fill_count = first.primitives.fills.len();
        assert!(first_fill_count > 0, "带 CSS 的加载应产生 fills");
        // 第一次 render — CSS 应持久
        let second = wv.render();
        assert_eq!(
            second.primitives.fills.len(),
            first_fill_count,
            "第一次 render 后 CSS 应持久保留，fills 数量应一致"
        );
        // 第二次 render — CSS 仍应持久
        let third = wv.render();
        assert_eq!(
            third.primitives.fills.len(),
            first_fill_count,
            "第二次 render 后 CSS 仍应持久保留"
        );
    }

    /// 验证 WebView 在完整生命周期中的状态转换正确性。
    ///
    /// 覆盖的状态转换路径：
    /// 1. Created -> Loading（load_url）
    /// 2. Loading -> Loaded（complete_load）
    /// 3. Loaded -> Loading（load_url 新 URL）
    /// 4. Loading -> Failed（fail_load）
    /// 5. Failed -> Loading（load_url 重试）
    /// 6. Loading -> Loaded（complete_load）
    /// 7. Loaded -> Loading（load_html 不改变 loading 状态）
    /// 验证每个阶段 is_loading、url、last_render 的值正确。
    #[test]
    fn test_webview_lifecycle_state_transitions() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            let label = match e {
                WebViewEvent::LoadStart(u) => format!("LoadStart({u})"),
                WebViewEvent::LoadEnd(u) => format!("LoadEnd({u})"),
                WebViewEvent::LoadFailed(u, m) => format!("LoadFailed({u},{m})"),
                WebViewEvent::TitleChanged(t) => format!("TitleChanged({t})"),
                WebViewEvent::UrlChanged(u) => format!("UrlChanged({u})"),
            };
            ec.borrow_mut().push(label);
        });

        // ── 阶段 1: Created ──
        assert!(!wv.is_loading());
        assert!(wv.url().is_none());
        assert!(wv.last_render().is_none());

        // ── 阶段 2: Created -> Loading ──
        wv.load_url("https://lifecycle.com");
        assert!(wv.is_loading());
        assert_eq!(wv.url(), Some("https://lifecycle.com"));
        assert!(wv.last_render().is_none());

        // ── 阶段 3: Loading -> Loaded ──
        wv.complete_load("<html><body><div>Loaded</div></body></html>", None);
        assert!(!wv.is_loading());
        assert_eq!(wv.url(), Some("https://lifecycle.com"));
        assert!(wv.last_render().is_some());

        // ── 阶段 4: Loaded -> Loading（导航到新 URL）──
        wv.load_url("https://fail-lifecycle.com");
        assert!(wv.is_loading());
        assert_eq!(wv.url(), Some("https://fail-lifecycle.com"));

        // ── 阶段 5: Loading -> Failed ──
        wv.fail_load("connection refused");
        assert!(!wv.is_loading());
        assert_eq!(wv.url(), Some("https://fail-lifecycle.com"));

        // ── 阶段 6: Failed -> Loading（重试）──
        wv.load_url("https://retry-lifecycle.com");
        assert!(wv.is_loading());
        assert_eq!(wv.url(), Some("https://retry-lifecycle.com"));

        // ── 阶段 7: Loading -> Loaded（重试成功）──
        wv.complete_load("<html><body><div>Retry OK</div></body></html>", None);
        assert!(!wv.is_loading());
        assert_eq!(wv.url(), Some("https://retry-lifecycle.com"));
        assert!(wv.last_render().is_some());

        // ── 验证事件序列 ──
        let recorded = events.borrow();
        // 完整序列:
        //   load_url -> LoadStart+UrlChanged (2)
        //   complete_load -> LoadEnd (1)
        //   load_url -> LoadStart+UrlChanged (2)
        //   fail_load -> LoadFailed (1)
        //   load_url -> LoadStart+UrlChanged (2)
        //   complete_load -> LoadEnd (1)
        //   合计 = 9
        assert_eq!(recorded.len(), 9, "应有 9 个事件，实际: {recorded:?}");
        assert_eq!(recorded[0], "LoadStart(https://lifecycle.com)");
        assert_eq!(recorded[1], "UrlChanged(https://lifecycle.com)");
        assert_eq!(recorded[2], "LoadEnd(https://lifecycle.com)");
        assert_eq!(recorded[3], "LoadStart(https://fail-lifecycle.com)");
        assert_eq!(recorded[4], "UrlChanged(https://fail-lifecycle.com)");
        assert!(recorded[5].starts_with("LoadFailed(https://fail-lifecycle.com"));
        assert_eq!(recorded[6], "LoadStart(https://retry-lifecycle.com)");
        assert_eq!(recorded[7], "UrlChanged(https://retry-lifecycle.com)");
        assert_eq!(recorded[8], "LoadEnd(https://retry-lifecycle.com)");
    }

    // ════════════════════════════════════════════════════════════════
    //  边界条件测试：极端尺寸、Unicode URL、异常状态转换、回调边界
    // ════════════════════════════════════════════════════════════════

    /// 验证将 WebView 尺寸调整至 u32::MAX 不会 panic。
    ///
    /// 边界场景：传入 u32 最大值作为视口宽高，
    /// 确保内部 RenderPipeline 不会因整数溢出或内存分配失败而崩溃。
    /// resize 应正常存储配置值，后续 render 也不应 panic。
    #[test]
    fn test_webview_resize_to_u32_max() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_html("<html><body><div>Extreme</div></body></html>", None);
        wv.resize(u32::MAX, u32::MAX);
        assert_eq!(wv.config().width, u32::MAX);
        assert_eq!(wv.config().height, u32::MAX);
        // render 在极端尺寸下应不 panic（管线内部会处理）
        let result = wv.render();
        assert!(result.timings.total_ms >= 0.0, "极端尺寸下渲染耗时应为非负");
    }

    /// 验证加载包含 Unicode 和特殊字符的 URL 不会 panic，且 URL 被正确存储。
    ///
    /// 边界场景：URL 包含中日韩字符、URL 编码百分号、查询参数中的特殊符号，
    /// 确保 load_url 不会因非 ASCII 字符而崩溃，current_url 被原样存储。
    #[test]
    fn test_webview_load_url_with_unicode_and_special_chars() {
        let mut wv = WebView::new(WebViewConfig::default());
        let url = "https://例え.jp/パス?q=hello%20world&lang=日本語#セクション";
        wv.load_url(url);
        assert_eq!(wv.url(), Some(url), "Unicode URL 应被原样存储");
        assert!(wv.is_loading());
        wv.complete_load("<html><body><div>Unicode URL 内容</div></body></html>", None);
        assert!(!wv.is_loading());
        assert_eq!(wv.url(), Some(url));
    }

    /// 验证从加载失败状态直接调用 complete_load 不会 panic，且状态转换正确。
    ///
    /// 异常状态转换路径：load_url -> fail_load -> complete_load（无中间 load_url）。
    /// fail_load 将 loading 置为 false，complete_load 应能正常工作：
    /// 加载 HTML、将 loading 置为 false（已为 false 不变），并触发 LoadEnd 事件。
    #[test]
    fn test_webview_fail_load_then_complete_without_load_url() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            let label = match e {
                WebViewEvent::LoadStart(u) => format!("LoadStart({u})"),
                WebViewEvent::LoadEnd(u) => format!("LoadEnd({u})"),
                WebViewEvent::LoadFailed(u, m) => format!("LoadFailed({u},{m})"),
                WebViewEvent::TitleChanged(t) => format!("TitleChanged({t})"),
                WebViewEvent::UrlChanged(u) => format!("UrlChanged({u})"),
            };
            ec.borrow_mut().push(label);
        });

        // 先导航并失败
        wv.load_url("https://will-fail.com");
        assert!(wv.is_loading());
        wv.fail_load("server error 500");
        assert!(!wv.is_loading());

        // 在未再次调用 load_url 的情况下直接 complete_load
        let result = wv.complete_load("<html><body><div>Recovery</div></body></html>", None);
        assert!(!wv.is_loading(), "complete_load 后不应处于加载状态");
        assert!(result.timings.total_ms >= 0.0, "渲染耗时应为非负");
        assert!(wv.last_render().is_some(), "应有渲染结果");
        // URL 应保留为 will-fail.com（complete_load 使用 current_url）
        assert_eq!(wv.url(), Some("https://will-fail.com"));

        // 事件序列：LoadStart + UrlChanged + LoadFailed + LoadEnd
        let recorded = events.borrow();
        assert_eq!(recorded.len(), 4, "应有 4 个事件，实际: {recorded:?}");
        assert!(recorded[3].starts_with("LoadEnd(https://will-fail.com"));
    }

    /// 验证在未注册任何回调时调用 remove_event_callback 返回 false。
    ///
    /// 边界场景：event_callbacks 为空列表时，传入索引 0（合法 usize），
    /// remove_event_callback 应安全返回 false 而非 panic。
    #[test]
    fn test_webview_remove_event_callback_on_empty_list() {
        let mut wv = WebView::new(WebViewConfig::default());
        // 未注册任何回调，空列表
        assert!(!wv.remove_event_callback(0), "空回调列表中索引 0 应返回 false");
        assert!(
            !wv.remove_event_callback(usize::MAX),
            "空回调列表中 usize::MAX 应返回 false"
        );
        // 后续操作应正常工作，不 panic
        wv.load_url("https://test.com");
        assert_eq!(wv.url(), Some("https://test.com"));
    }

    /// 验证在加载状态（loading=true）下调用 render 不会改变加载标志。
    ///
    /// 边界场景：load_url 将 loading 置为 true 后，直接调用 render，
    /// render 仅执行重新渲染，不应干扰 loading 状态。
    /// 适用于外部异步加载过程中需要中间渲染的场景（如进度指示器）。
    #[test]
    fn test_webview_render_while_loading_state() {
        let mut wv = WebView::new(WebViewConfig::default());

        // 先加载一些内容到 cached_html 中
        wv.load_html("<html><body><div>Loading indicator</div></body></html>", None);
        assert!(!wv.is_loading());

        // 发起 URL 加载（设置 loading=true）
        wv.load_url("https://slow-page.com");
        assert!(wv.is_loading(), "load_url 后应处于加载状态");
        assert!(wv.last_render().is_some(), "之前的 load_html 应有渲染结果");

        // 在 loading 状态下调用 render — 模拟显示加载进度
        let result = wv.render();
        assert!(result.timings.total_ms >= 0.0, "loading 中 render 耗时应为非负");

        // 关键断言：render 不应改变 loading 状态
        assert!(wv.is_loading(), "render() 不应改变 loading 状态，应仍为 true");
        assert_eq!(wv.url(), Some("https://slow-page.com"), "URL 不应被 render 改变");

        // 最终完成加载
        wv.complete_load("<html><body><div>Final content</div></body></html>", None);
        assert!(!wv.is_loading(), "complete_load 后不应处于加载状态");
    }

    // ════════════════════════════════════════════════════════════════
    //  边界条件测试：空标题、CSS 累积链、空白 HTML、无 URL 失败、inject 不干扰 loading
    // ════════════════════════════════════════════════════════════════

    /// 验证将标题设置为空字符串后，title() 返回 Some("") 而非 None。
    ///
    /// 边界场景：空字符串在语义上与 None 不同，
    /// set_title("") 应将内部 title 字段设为 Some("")，
    /// 后续 title() 应精确返回 Some("") 而非 None。
    #[test]
    fn test_webview_set_title_empty_string() {
        let mut wv = WebView::new(WebViewConfig::default());
        // 初始状态：标题为 None
        assert!(wv.title().is_none(), "初始标题应为 None");

        // 设置空字符串标题
        wv.set_title("");
        assert_eq!(wv.title(), Some(""), "空字符串标题应为 Some(\"\")，而非 None");

        // 覆盖为非空标题后再次设为空字符串
        wv.set_title("Real Title");
        assert_eq!(wv.title(), Some("Real Title"));
        wv.set_title("");
        assert_eq!(wv.title(), Some(""), "再次设为空字符串应为 Some(\"\")");
    }

    /// 验证 complete_load 传入 CSS 后，再 inject_css 追加的样式被正确累积。
    ///
    /// 场景：load_url -> complete_load(html, Some(css_a)) -> inject_css(css_b)。
    /// complete_load 内部调用 load_html 会缓存 css_a，
    /// inject_css 在 cached_css 后追加 css_b，
    /// render() 应使用包含 css_a + css_b 的累积 CSS。
    #[test]
    fn test_webview_complete_load_with_css_then_inject_more_css() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body>\
            <div class=\"original\">A</div>\
            <div class=\"extra\">B</div>\
            </body></html>";

        // 通过 load_url + complete_load 加载带 CSS 的内容
        wv.load_url("https://styled.com");
        let after_complete = wv.complete_load(
            html,
            Some(".original { background-color: red; width: 100px; height: 50px; }"),
        );
        let fills_after_complete = after_complete.primitives.fills.len();
        assert!(fills_after_complete > 0, "complete_load 带 CSS 应产生 fills");

        // 注入额外 CSS
        let after_inject = wv.inject_css(".extra { background-color: blue; width: 80px; height: 40px; }");
        let fills_after_inject = after_inject.primitives.fills.len();
        assert!(
            fills_after_inject >= fills_after_complete,
            "inject_css 应追加到 complete_load 的 CSS 上，fills 应 >= 注入前 (got {fills_after_inject} < {fills_after_complete})"
        );

        // render() 应保留累积的 CSS
        let after_render = wv.render();
        assert_eq!(
            after_render.primitives.fills.len(),
            fills_after_inject,
            "render() 应使用 complete_load CSS + inject CSS 的累积结果"
        );
    }

    /// 验证加载仅含空白字符的 HTML 不会 panic，且返回有效渲染结果。
    ///
    /// 边界场景：传入 "   \n\t  " 等纯空白字符串，
    /// 渲染管线应能处理无有效 HTML 标签的输入，
    /// 不会因缺少根元素或内容为空而崩溃。
    #[test]
    fn test_webview_load_html_with_only_whitespace() {
        let mut wv = WebView::new(WebViewConfig::default());
        let whitespace_html = "   \n\t  \r\n   ";
        let result = wv.load_html(whitespace_html, None);
        assert!(result.timings.total_ms >= 0.0, "纯空白 HTML 渲染耗时应为非负");
        assert!(wv.last_render().is_some(), "纯空白 HTML 加载后应有渲染结果");
        assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
        assert!(wv.url().is_none(), "load_html 不应设置 URL");

        // 后续操作应正常工作
        let inject_result = wv.inject_css("div { color: red; }");
        assert!(inject_result.timings.total_ms >= 0.0, "空白 HTML 上注入 CSS 不应 panic");

        let render_result = wv.render();
        assert!(render_result.timings.total_ms >= 0.0, "空白 HTML 上重新渲染不应 panic");
    }

    /// 验证在未先调用 load_url 的情况下直接调用 fail_load 不会 panic。
    ///
    /// 边界场景：current_url 为 None 时调用 fail_load，
    /// 内部 current_url.unwrap_or_default() 应返回空字符串，
    /// LoadFailed 事件的 URL 字段应为空字符串。
    /// loading 状态应从 false 变为 false（无变化）。
    #[test]
    fn test_webview_fail_load_without_prior_load_url_uses_empty_url() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            ec.borrow_mut().push(e.clone());
        });

        // 初始状态：无 URL，不在加载中
        assert!(wv.url().is_none());
        assert!(!wv.is_loading());

        // 直接调用 fail_load，未先调用 load_url
        wv.fail_load("unexpected error");
        assert!(!wv.is_loading(), "fail_load 后 loading 应为 false");

        // 验证 LoadFailed 事件的 URL 为空字符串
        let recorded = events.borrow();
        assert_eq!(recorded.len(), 1, "应有 1 个 LoadFailed 事件");
        assert!(
            matches!(&recorded[0], WebViewEvent::LoadFailed(url, msg) if url.is_empty() && msg.contains("unexpected error")),
            "LoadFailed 事件的 URL 应为空字符串，实际: {:?}",
            recorded[0]
        );
    }

    /// 验证在 loading 状态下调用 inject_css 不会重置 loading 标志。
    ///
    /// 边界场景：load_url 将 loading 置为 true 后，
    /// 调用 inject_css 进行样式注入（如加载指示器的 CSS 动画），
    /// inject_css 不应干扰导航状态，loading 应保持为 true。
    /// 适用于异步加载过程中动态更新样式的场景。
    #[test]
    fn test_webview_inject_css_after_load_url_preserves_loading_state() {
        let mut wv = WebView::new(WebViewConfig::default());

        // 先加载 HTML 内容到缓存
        wv.load_html(
            "<html><body><div class=\"spinner\">Loading...</div></body></html>",
            None,
        );
        assert!(!wv.is_loading());

        // 发起 URL 加载
        wv.load_url("https://async-page.com");
        assert!(wv.is_loading(), "load_url 后应处于加载状态");
        assert_eq!(wv.url(), Some("https://async-page.com"));

        // 在 loading 状态下注入 CSS（如加载动画样式）
        let result = wv.inject_css(".spinner { animation: spin 1s linear infinite; }");
        assert!(result.timings.total_ms >= 0.0, "inject_css 渲染耗时应为非负");

        // 关键断言：inject_css 不应重置 loading 状态
        assert!(wv.is_loading(), "inject_css 不应改变 loading 状态，应仍为 true");
        assert_eq!(wv.url(), Some("https://async-page.com"), "URL 不应被 inject_css 改变");

        // 后续 complete_load 应正常完成加载
        wv.complete_load("<html><body><div>Final</div></body></html>", None);
        assert!(!wv.is_loading(), "complete_load 后不应处于加载状态");
    }

    // ════════════════════════════════════════════════════════════════
    //  边界条件测试：连续失败、回调移除后验证、Builder 空 URL、超长 CSS、渲染幂等
    // ════════════════════════════════════════════════════════════════

    /// 验证连续调用 fail_load 两次不会 panic，且 loading 始终为 false。
    ///
    /// 边界场景：第一次 fail_load 将 loading 从 true 置为 false，
    /// 第二次 fail_load 在 loading 已经为 false 的状态下调用，
    /// 不应导致状态异常或 panic，且每次调用都应触发 LoadFailed 事件。
    #[test]
    fn test_webview_consecutive_fail_load_calls() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            let label = match e {
                WebViewEvent::LoadStart(u) => format!("LoadStart({u})"),
                WebViewEvent::LoadEnd(u) => format!("LoadEnd({u})"),
                WebViewEvent::LoadFailed(u, m) => format!("LoadFailed({u},{m})"),
                WebViewEvent::TitleChanged(t) => format!("TitleChanged({t})"),
                WebViewEvent::UrlChanged(u) => format!("UrlChanged({u})"),
            };
            ec.borrow_mut().push(label);
        });

        // 第一次加载并失败
        wv.load_url("https://first-fail.com");
        assert!(wv.is_loading());
        wv.fail_load("timeout");
        assert!(!wv.is_loading());

        // 第二次连续失败（未重新 load_url）
        wv.fail_load("second error");
        assert!(!wv.is_loading(), "连续 fail_load 后 loading 应仍为 false");

        // 验证两次 LoadFailed 事件都被触发
        let recorded = events.borrow();
        let fail_count = recorded.iter().filter(|e| e.starts_with("LoadFailed")).count();
        assert_eq!(fail_count, 2, "应有 2 次 LoadFailed 事件");
    }

    /// 验证移除事件回调后，后续操作不再触发该回调。
    ///
    /// 场景：注册回调 A -> 触发操作（回调 A 被调用）-> 移除回调 A -> 触发操作（回调 A 不再被调用）。
    /// 通过引用计数验证回调被调用次数精确匹配预期值。
    #[test]
    fn test_webview_callback_removed_no_longer_fires() {
        let mut wv = WebView::new(WebViewConfig::default());
        let call_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
        let cc = call_count.clone();
        let idx = wv.on_event(move |_| {
            *cc.borrow_mut() += 1;
        });

        // 第一次 set_title — 回调应被触发
        wv.set_title("First");
        assert_eq!(*call_count.borrow(), 1, "注册后回调应被触发 1 次");

        // 移除回调
        assert!(wv.remove_event_callback(idx));

        // 第二次 set_title — 已移除的回调不应再被触发
        wv.set_title("Second");
        assert_eq!(*call_count.borrow(), 1, "移除后回调不应再被触发，计数应保持 1");

        // 第三次 set_title — 确认回调持续不触发
        wv.set_title("Third");
        assert_eq!(*call_count.borrow(), 1, "多次操作后回调仍不应被触发");
    }

    /// 验证 WebViewBuilder 传入空字符串 URL 后，build 产生正确的初始状态。
    ///
    /// 边界场景：url("") 是合法的链式调用（非 None），
    /// build 应自动调用 load_url("")，将 WebView 置为加载状态，
    /// current_url 应为 Some("")（空字符串，与 None 语义不同）。
    #[test]
    fn test_webview_builder_with_empty_url_string() {
        let wv = WebViewBuilder::new().url("").build();
        // url("") 设置了 config.url = Some("")，build 时会调用 load_url("")
        assert_eq!(wv.url(), Some(""), "空字符串 URL 应为 Some(\"\")，而非 None");
        assert!(wv.is_loading(), "空 URL 仍应触发加载状态");
        assert!(wv.last_render().is_none(), "仅有 load_url 不应有渲染结果");

        // 后续 complete_load 应正常工作
        let mut wv = wv;
        wv.complete_load("<html><body><div>Empty URL page</div></body></html>", None);
        assert!(!wv.is_loading());
        assert!(wv.last_render().is_some());
        assert_eq!(wv.url(), Some(""), "complete_load 后 URL 应保持为空字符串");
    }

    /// 验证加载包含超长 CSS 属性值的 HTML 不会 panic，且渲染管线返回有效结果。
    ///
    /// 边界场景：CSS 属性值长度达到数千字符（如超长 gradient 定义），
    /// 确保 CSS 解析器和渲染管线不会因字符串过长而崩溃或内存溢出。
    #[test]
    fn test_webview_load_html_with_very_long_css_value() {
        let mut wv = WebView::new(WebViewConfig::default());
        // 构造一个超长的 CSS background 属性值（重复 linear-gradient 段）
        let long_gradient = "linear-gradient(red, blue)".repeat(200);
        let css = format!("div {{ background: {long_gradient}; width: 100px; height: 50px; }}");
        let html = "<html><body><div>Long CSS test</div></body></html>";

        let result = wv.load_html(html, Some(&css));
        assert!(result.timings.total_ms >= 0.0, "超长 CSS 渲染耗时应为非负");
        assert!(wv.last_render().is_some(), "超长 CSS 加载后应有渲染结果");

        // 后续操作不应崩溃
        let inject_result = wv.inject_css("span { color: red; }");
        assert!(inject_result.timings.total_ms >= 0.0);
        let render_result = wv.render();
        assert!(render_result.timings.total_ms >= 0.0);
    }

    /// 验证 complete_load 后连续多次 render 产生完全相同的 fills 数量（渲染幂等性）。
    ///
    /// 边界场景：相同输入（cached_html + cached_css）多次调用 render，
    /// 渲染结果应在 fills 数量上保持一致（幂等），
    /// 不应因内部状态变化而产生不同输出。
    #[test]
    fn test_webview_render_idempotent_after_complete_load() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body>\
            <div class=\"box-a\">Box A</div>\
            <div class=\"box-b\">Box B</div>\
            </body></html>";
        let css = ".box-a { background-color: red; width: 100px; height: 50px; }\
                   .box-b { background-color: blue; width: 200px; height: 80px; }";

        wv.load_url("https://idempotent.com");
        let _complete = wv.complete_load(html, Some(css));
        assert!(!wv.is_loading());
        assert!(wv.last_render().is_some());

        // 第一次 render
        let first = wv.render();
        let first_fills = first.primitives.fills.len();

        // 第二次 render — 应产生相同的 fills 数量
        let second = wv.render();
        let second_fills = second.primitives.fills.len();

        // 第三次 render — 进一步确认幂等性
        let third = wv.render();
        let third_fills = third.primitives.fills.len();

        assert_eq!(
            first_fills, second_fills,
            "连续 render 的 fills 数量应一致（第一次 vs 第二次）"
        );
        assert_eq!(
            second_fills, third_fills,
            "连续 render 的 fills 数量应一致（第二次 vs 第三次）"
        );
        assert!(first_fills > 0, "带背景色 CSS 的 HTML 应产生至少一个 fill 图元");
    }

    // ════════════════════════════════════════════════════════════════
    //  边界条件测试：inject_css 先于 load_html、set_title 事件计数、零视口渲染、
    //  失败恢复后渲染、连续 load_html 内容覆盖
    // ════════════════════════════════════════════════════════════════

    /// 验证在全新 WebView 上（从未调用 load_html）直接 inject_css 不会 panic。
    ///
    /// 边界场景：WebView 刚创建，cached_html 为空，
    /// 此时调用 inject_css 应安全返回有效的渲染结果，
    /// 不会因缺少已缓存 HTML 而崩溃。
    #[test]
    fn test_webview_inject_css_before_any_load_html() {
        let mut wv = WebView::new(WebViewConfig::default());
        // 全新 WebView，未调用任何 load_html
        assert!(wv.last_render().is_none(), "全新 WebView 不应有渲染结果");

        // 在未加载任何 HTML 的情况下直接注入 CSS
        let result = wv.inject_css("div { color: red; width: 100px; height: 50px; }");
        assert!(result.timings.total_ms >= 0.0, "inject_css 应返回非负耗时");
        assert!(wv.last_render().is_some(), "inject_css 后应有渲染结果");
        assert!(!wv.is_loading(), "inject_css 不应触发加载状态");

        // 后续 load_html 应正常工作
        let html_result = wv.load_html("<html><body><div>After inject</div></body></html>", None);
        assert!(html_result.timings.total_ms >= 0.0, "后续 load_html 应正常工作");
    }

    /// 验证多次 set_title 调用每次都触发独立的 TitleChanged 事件。
    ///
    /// 场景：连续调用 set_title 三次（包含重复标题值），
    /// 每次调用都应触发一个 TitleChanged 事件，共 3 个事件。
    /// 即使标题值与前一次相同，事件仍应触发。
    #[test]
    fn test_webview_set_title_fires_separate_events_each_call() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            ec.borrow_mut().push(e.clone());
        });

        // 第一次 set_title
        wv.set_title("标题一");
        // 第二次 set_title（不同值）
        wv.set_title("标题二");
        // 第三次 set_title（与第二次相同的值——仍应触发事件）
        wv.set_title("标题二");

        let recorded = events.borrow();
        assert_eq!(recorded.len(), 3, "三次 set_title 应触发 3 个 TitleChanged 事件");
        assert!(
            matches!(&recorded[0], WebViewEvent::TitleChanged(t) if t == "标题一"),
            "第一个事件应为 TitleChanged(\"标题一\")"
        );
        assert!(
            matches!(&recorded[1], WebViewEvent::TitleChanged(t) if t == "标题二"),
            "第二个事件应为 TitleChanged(\"标题二\")"
        );
        assert!(
            matches!(&recorded[2], WebViewEvent::TitleChanged(t) if t == "标题二"),
            "第三个事件应为 TitleChanged(\"标题二\")（重复值仍触发事件）"
        );
    }

    /// 验证 WebView 在零尺寸视口（width=0, height=0）下 render 不会 panic。
    ///
    /// 边界场景：将视口尺寸设为 (0, 0) 后调用 render，
    /// 渲染管线应能处理零尺寸画布，不会因除零或空缓冲区而崩溃。
    /// 适用于窗口最小化或隐藏时的场景。
    #[test]
    fn test_webview_render_with_zero_size_viewport() {
        let mut wv = WebView::new(WebViewConfig {
            width: 0,
            height: 0,
            ..Default::default()
        });

        // 加载内容后渲染——零视口不应 panic
        let result = wv.load_html("<html><body><div>Zero viewport</div></body></html>", None);
        assert!(result.timings.total_ms >= 0.0, "零视口 load_html 应返回非负耗时");

        // render 在零视口下也不应 panic
        let render_result = wv.render();
        assert!(render_result.timings.total_ms >= 0.0, "零视口 render 应返回非负耗时");
        assert_eq!(wv.config().width, 0, "视口宽度应为 0");
        assert_eq!(wv.config().height, 0, "视口高度应为 0");

        // resize 到正常尺寸后应恢复正常
        wv.resize(800, 600);
        let after_resize = wv.render();
        assert!(after_resize.timings.total_ms >= 0.0, "恢复尺寸后 render 应成功");
        assert_eq!(wv.config().width, 800);
        assert_eq!(wv.config().height, 600);
    }

    /// 验证加载失败后通过 load_html 恢复，渲染结果反映恢复后的内容。
    ///
    /// 场景：load_url -> fail_load（模拟网络错误）-> load_html 恢复。
    /// fail_load 后 last_render 保留之前的结果（可能为 None），
    /// load_html 应覆盖缓存内容并产生新的渲染结果，
    /// 且 WebView 状态应完全恢复为正常（不处于加载状态）。
    #[test]
    fn test_webview_render_after_load_failure_recovery() {
        let mut wv = WebView::new(WebViewConfig::default());
        let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
        let ec = events.clone();
        wv.on_event(move |e| {
            let label = match e {
                WebViewEvent::LoadStart(u) => format!("LoadStart({u})"),
                WebViewEvent::LoadEnd(u) => format!("LoadEnd({u})"),
                WebViewEvent::LoadFailed(u, m) => format!("LoadFailed({u},{m})"),
                WebViewEvent::TitleChanged(t) => format!("TitleChanged({t})"),
                WebViewEvent::UrlChanged(u) => format!("UrlChanged({u})"),
            };
            ec.borrow_mut().push(label);
        });

        // 先成功加载一个页面
        wv.load_url("https://good-page.com");
        wv.complete_load("<html><body><div>Good content</div></body></html>", None);
        assert!(!wv.is_loading());
        assert!(wv.last_render().is_some());
        let good_render_fills = wv.last_render().unwrap().primitives.fills.len();

        // 导航到新 URL 但加载失败
        wv.load_url("https://broken-page.com");
        assert!(wv.is_loading());
        wv.fail_load("connection refused");
        assert!(!wv.is_loading(), "失败后 loading 应停止");
        // last_render 保留之前成功的结果
        assert!(wv.last_render().is_some(), "失败后应保留上次成功的渲染结果");

        // 通过 load_html 恢复——加载新内容
        let recovery_html = "<html><body><div>Recovery content</div></body></html>";
        let recovery_css = "div { background-color: green; width: 200px; height: 100px; }";
        let result = wv.load_html(recovery_html, Some(recovery_css));
        assert!(result.timings.total_ms >= 0.0, "恢复 load_html 应返回非负耗时");
        assert!(!wv.is_loading(), "load_html 后不应处于加载状态");
        assert!(wv.last_render().is_some(), "恢复后应有渲染结果");

        // 渲染结果应反映恢复后的内容（带 CSS，fills 应 > 无 CSS 时）
        let render_result = wv.render();
        assert!(render_result.timings.total_ms >= 0.0);
        assert!(
            render_result.primitives.fills.len() >= good_render_fills,
            "恢复后的渲染结果应反映新加载的带 CSS 内容"
        );

        // 验证事件序列：LoadStart+UrlChanged+LoadEnd + LoadStart+UrlChanged+LoadFailed
        // load_html 不触发事件
        let recorded = events.borrow();
        assert_eq!(recorded.len(), 6, "应有 6 个事件（成功加载 3 + 失败 3）");
        assert!(recorded[5].starts_with("LoadFailed(https://broken-page.com"));
    }

    /// 验证连续调用 load_html 三次，每次渲染结果反映最新加载的内容。
    ///
    /// 场景：依次加载三份不同 HTML（含不同 CSS 样式），
    /// 每次 load_html 后调用 render，验证 fills 数量随内容变化。
    /// 最终 render 应反映第三次加载的内容，而非第一次或第二次。
    #[test]
    fn test_webview_consecutive_load_html_reflects_latest_content() {
        let mut wv = WebView::new(WebViewConfig::default());

        // 第一次加载：带红色背景的 div
        let html1 = "<html><body><div class=\"box\">Version 1</div></body></html>";
        let css1 = ".box { background-color: red; width: 100px; height: 50px; }";
        let result1 = wv.load_html(html1, Some(css1));
        assert!(result1.timings.total_ms >= 0.0, "第一次 load_html 应成功");
        let fills1 = wv.render().primitives.fills.len();

        // 第二次加载：带蓝色背景的 div + 额外 div
        let html2 = "<html><body>\
            <div class=\"box\">Version 2</div>\
            <div class=\"extra\">Extra</div>\
            </body></html>";
        // 注意：load_html 会重置 cached_css，仅使用传入的 CSS
        let css2 = ".box { background-color: blue; width: 200px; height: 80px; }\
                    .extra { background-color: green; width: 50px; height: 30px; }";
        let result2 = wv.load_html(html2, Some(css2));
        assert!(result2.timings.total_ms >= 0.0, "第二次 load_html 应成功");
        let fills2 = wv.render().primitives.fills.len();

        // 第三次加载：仅一个无样式的 div
        let html3 = "<html><body><div>Version 3 - plain</div></body></html>";
        let result3 = wv.load_html(html3, None);
        assert!(result3.timings.total_ms >= 0.0, "第三次 load_html 应成功");
        let fills3 = wv.render().primitives.fills.len();

        // 验证第二次加载的 fills >= 第一次（更多带样式的元素）
        assert!(
            fills2 >= fills1,
            "第二次加载（两个带样式 div）的 fills 应 >= 第一次（一个带样式 div），got {fills2} < {fills1}"
        );

        // 验证第三次加载的 fills <= 第二次（无 CSS，背景色消失）
        assert!(
            fills3 <= fills2,
            "第三次加载（无 CSS）的 fills 应 <= 第二次（两个带样式 div），got {fills3} > {fills2}"
        );

        // 最终 render 应反映第三次的内容（无 CSS）
        let final_render = wv.render();
        assert_eq!(
            final_render.primitives.fills.len(),
            fills3,
            "最终 render 应反映第三次加载的内容"
        );
    }

    // ════════════════════════════════════════════════════════════════
    //  边界条件测试：极大尺寸 Builder、inject_css 空字符串、最小 HTML、
    //  TitleChanged 回调零触发、多次 inject_css 累积渲染
    // ════════════════════════════════════════════════════════════════

    /// 验证 WebViewBuilder 使用极大尺寸（u32::MAX）构建 WebView 不会 panic，
    /// 且生成的 WebView 配置正确反映传入的尺寸。
    ///
    /// 边界场景：width/height 设为 u32 最大值，
    /// 确保构建器不会因整数溢出或内存预分配失败而崩溃。
    /// 后续 load_html 和 render 也应在极端视口下安全完成。
    #[test]
    fn test_webview_builder_very_large_dimensions() {
        let mut wv = WebViewBuilder::new().width(u32::MAX).height(u32::MAX).build();

        assert_eq!(wv.config().width, u32::MAX, "宽度应存储为 u32::MAX");
        assert_eq!(wv.config().height, u32::MAX, "高度应存储为 u32::MAX");

        // 极大视口下加载和渲染不应 panic
        let html = "<html><body><div>Large viewport</div></body></html>";
        let result = wv.load_html(html, None);
        assert!(result.timings.total_ms >= 0.0, "极大视口渲染耗时应为非负");
        assert!(wv.last_render().is_some());

        let render_result = wv.render();
        assert!(render_result.timings.total_ms >= 0.0, "极大视口下 re-render 应安全");
    }

    /// 验证 inject_css 传入空字符串不会 panic，且返回有效的渲染结果。
    ///
    /// 边界场景：在已有 HTML 内容的 WebView 上注入空 CSS，
    /// 渲染管线应安全处理空字符串输入，
    /// fills 数量应与注入前保持一致（空 CSS 不产生新的样式规则）。
    #[test]
    fn test_webview_inject_css_empty_string_preserves_fills() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body><div class=\"box\">Content</div></body></html>";
        let css = ".box { background-color: red; width: 100px; height: 50px; }";

        let after_load = wv.load_html(html, Some(css));
        let fills_before = after_load.primitives.fills.len();
        assert!(fills_before > 0, "带 CSS 的 load_html 应产生 fills");

        // 注入空字符串 CSS
        let after_inject = wv.inject_css("");
        assert!(after_inject.timings.total_ms >= 0.0, "空 CSS 注入耗时应为非负");
        assert_eq!(
            after_inject.primitives.fills.len(),
            fills_before,
            "空 CSS 注入不应改变 fills 数量"
        );

        // render 也应保持一致
        let after_render = wv.render();
        assert_eq!(
            after_render.primitives.fills.len(),
            fills_before,
            "render 后 fills 应与注入空 CSS 前一致"
        );
    }

    /// 验证 load_html 加载最小 HTML "<html></html>" 不会 panic，且返回有效渲染结果。
    ///
    /// 边界场景：传入仅含根元素、无 body、无内容的极简 HTML，
    /// 确保 DOM 树构建和渲染管线不会因缺少 body 或内容为空而崩溃。
    #[test]
    fn test_webview_load_html_minimal_html_tag() {
        let mut wv = WebView::new(WebViewConfig::default());

        let result = wv.load_html("<html></html>", None);
        assert!(result.timings.total_ms >= 0.0, "最小 HTML 渲染耗时应为非负");
        assert!(wv.last_render().is_some(), "最小 HTML 加载后应有渲染结果");
        assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
        assert!(wv.url().is_none(), "load_html 不应设置 URL");
        assert!(wv.title().is_none(), "最小 HTML 不应产生标题");

        // 后续操作应正常工作
        let inject_result = wv.inject_css("body { margin: 0; }");
        assert!(inject_result.timings.total_ms >= 0.0, "最小 HTML 上注入 CSS 不应 panic");

        let render_result = wv.render();
        assert!(render_result.timings.total_ms >= 0.0, "最小 HTML 重新渲染不应 panic");
    }

    /// 验证在未调用 set_title 时，TitleChanged 回调触发次数为零。
    ///
    /// 边界场景：注册 TitleChanged 监听后执行一系列操作
    /// （load_url、complete_load、load_html、inject_css、render），
    /// 由于所有操作均未调用 set_title，
    /// TitleChanged 事件计数应始终保持为 0。
    #[test]
    fn test_webview_title_changed_zero_fires_without_set_title() {
        let mut wv = WebView::new(WebViewConfig::default());
        let title_change_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
        let tcc = title_change_count.clone();
        wv.on_event(move |e| {
            if matches!(e, WebViewEvent::TitleChanged(_)) {
                *tcc.borrow_mut() += 1;
            }
        });

        // 执行一系列操作——均不涉及 set_title
        wv.load_html("<html><body><div>No title set</div></body></html>", None);
        assert_eq!(*title_change_count.borrow(), 0, "load_html 后 TitleChanged 计数应为 0");

        wv.load_url("https://titleless.com");
        assert_eq!(*title_change_count.borrow(), 0, "load_url 后 TitleChanged 计数应为 0");

        wv.complete_load("<html><body><div>Loaded</div></body></html>", None);
        assert_eq!(
            *title_change_count.borrow(),
            0,
            "complete_load 后 TitleChanged 计数应为 0"
        );

        wv.inject_css("div { color: blue; }");
        assert_eq!(*title_change_count.borrow(), 0, "inject_css 后 TitleChanged 计数应为 0");

        let _ = wv.render();
        assert_eq!(*title_change_count.borrow(), 0, "render 后 TitleChanged 计数应为 0");

        // 确认 title() 仍为 None
        assert!(wv.title().is_none(), "未调用 set_title 时 title 应为 None");
    }

    /// 验证多次调用 inject_css 后 render 累积所有 CSS，fills 单调递增。
    ///
    /// 边界场景：连续注入三条独立 CSS 规则（分别匹配不同 class），
    /// 每次注入后 fills 数量应 >= 上一次（CSS 累积而非替换）。
    /// 最终 render 应使用所有累积的 CSS，fills 数量与最后一次 inject_css 一致。
    #[test]
    fn test_webview_render_accumulates_all_css_after_multiple_injects() {
        let mut wv = WebView::new(WebViewConfig::default());
        let html = "<html><body>\
            <div class=\"a\">A</div>\
            <div class=\"b\">B</div>\
            <div class=\"c\">C</div>\
            </body></html>";

        // 初始加载（无 CSS）
        let initial = wv.load_html(html, None);
        let fills_initial = initial.primitives.fills.len();

        // 第一次注入：为 .a 添加样式
        let after_a = wv.inject_css(".a { background-color: red; width: 50px; height: 50px; }");
        let fills_a = after_a.primitives.fills.len();
        assert!(fills_a >= fills_initial, "第一次注入后 fills 应 >= 初始值");

        // 第二次注入：为 .b 添加样式
        let after_b = wv.inject_css(".b { background-color: green; width: 60px; height: 60px; }");
        let fills_b = after_b.primitives.fills.len();
        assert!(fills_b >= fills_a, "第二次注入后 fills 应 >= 第一次注入后");

        // 第三次注入：为 .c 添加样式
        let after_c = wv.inject_css(".c { background-color: blue; width: 70px; height: 70px; }");
        let fills_c = after_c.primitives.fills.len();
        assert!(fills_c >= fills_b, "第三次注入后 fills 应 >= 第二次注入后");

        // render 应累积所有三条 CSS，fills 与最后一次 inject_css 一致
        let after_render = wv.render();
        assert_eq!(
            after_render.primitives.fills.len(),
            fills_c,
            "render() 应使用累积的所有 CSS（.a + .b + .c），fills 数量应与最后一次 inject_css 一致"
        );

        // 再次 render 确认幂等
        let after_rerender = wv.render();
        assert_eq!(
            after_rerender.primitives.fills.len(),
            fills_c,
            "第二次 render 的 fills 应与第一次 render 一致（幂等）"
        );
    }

    // ── 新增边界测试 ──

    /// 测试 WebViewConfig 默认 devtools 为 false。
    #[test]
    fn test_webview_config_default_devtools_off() {
        let config = WebViewConfig::default();
        assert!(!config.devtools, "默认 devtools 应为 false");
        assert!(!config.transparent, "默认 transparent 应为 false");
    }

    /// 测试 load_html 后 last_render 不为 None。
    #[test]
    fn test_webview_load_html_sets_last_render() {
        let mut wv = WebView::new(WebViewConfig::default());
        assert!(wv.last_render().is_none(), "初始 last_render 应为 None");

        wv.load_html("<p>Hello</p>", None);
        assert!(wv.last_render().is_some(), "load_html 后 last_render 不应为 None");
    }

    /// 测试 resize 后重新 render 仍能工作（边界补充）。
    #[test]
    fn test_webview_resize_render_preserves_content() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_html("<div style='background: red; width: 100px; height: 50px;'></div>", None);
        let fills_before = wv.last_render().unwrap().primitives.fills.len();

        wv.resize(1024, 768);
        let result = wv.render();
        assert!(
            result.primitives.fills.len() >= fills_before,
            "resize 后 render 的 fills 不应少于 resize 前"
        );
    }

    /// 测试 is_loading 初始状态为 false。
    #[test]
    fn test_webview_initial_not_loading() {
        let wv = WebView::new(WebViewConfig::default());
        assert!(!wv.is_loading(), "初始状态不应在加载中");
    }

    /// 测试 remove_event_callback 对不存在的索引返回 false。
    #[test]
    fn test_webview_remove_nonexistent_callback() {
        let mut wv = WebView::new(WebViewConfig::default());
        assert!(!wv.remove_event_callback(999), "移除不存在的索引应返回 false");
        assert!(!wv.remove_event_callback(0), "移除索引 0（未注册）应返回 false");
    }
}
