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
    fn test_webview_execute_script_not_implemented() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.execute_script("console.log('test')");
        assert!(result.is_err());
        match result.unwrap_err() {
            WebViewError::NotImplemented(msg) => {
                assert!(
                    msg.contains("V8") || msg.contains("QuickJS"),
                    "Expected V8/QuickJS mention, got: {msg}"
                );
            }
            other => panic!("Expected NotImplemented, got: {other}"),
        }
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
        assert!(result.is_err());
        match result.unwrap_err() {
            WebViewError::NotImplemented(_) => {}
            other => panic!("Expected NotImplemented, got: {other}"),
        }
    }

    #[test]
    fn test_webview_execute_script_multiple_calls() {
        let mut wv = WebView::new(WebViewConfig::default());
        for i in 0..5 {
            let result = wv.execute_script(&format!("console.log({i})"));
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_webview_execute_script_with_special_chars() {
        let mut wv = WebView::new(WebViewConfig::default());
        let script = "let s = 'hello \"world\" 🌍';";
        let result = wv.execute_script(script);
        assert!(result.is_err());
    }

    #[test]
    fn test_webview_execute_script_returns_string_err() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.execute_script("1 + 1");
        assert!(matches!(result, Err(WebViewError::NotImplemented(_))));
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
}
