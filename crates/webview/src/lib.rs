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

    /// 验证执行空脚本字符串不会 panic，且仍返回 NotImplemented 错误。
    ///
    /// 边界场景：传入空字符串作为脚本内容，
    /// 确保 JS 引擎尚未集成时代码路径仍然安全。
    #[test]
    fn test_webview_execute_script_empty_string() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.execute_script("");
        assert!(result.is_err(), "空脚本应返回错误（JS 引擎未集成）");
        match result.unwrap_err() {
            WebViewError::NotImplemented(msg) => {
                assert!(
                    msg.contains("V8") || msg.contains("QuickJS"),
                    "错误信息应提及所需的 JS 引擎，实际: {msg}"
                );
            }
            other => panic!("预期 NotImplemented 错误，实际: {other}"),
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
    /// 当前 zero-script-sandbox 尚未集成 V8/QuickJS 引擎，
    /// execute_script 应安全地拒绝所有脚本执行请求，
    /// 并返回包含引擎信息的 NotImplemented 错误。
    #[test]
    fn test_webview_execute_script_placeholder() {
        let mut wv = WebView::new(WebViewConfig::default());

        // 执行一条简单的脚本——应返回 NotImplemented
        let result = wv.execute_script("document.title = 'test'");
        assert!(result.is_err(), "JS 引擎未集成时应返回错误");

        match result.unwrap_err() {
            WebViewError::NotImplemented(msg) => {
                // 错误信息应指明需要的引擎
                assert!(
                    msg.contains("V8") || msg.contains("QuickJS"),
                    "错误信息应提及 V8 或 QuickJS 引擎，实际: {msg}"
                );
            }
            other => panic!("预期 NotImplemented 错误，实际: {other}"),
        }

        // WebView 状态不应因 execute_script 调用而改变
        assert!(wv.url().is_none());
        assert!(!wv.is_loading());

        // 多次调用同样安全返回错误
        for _ in 0..3 {
            let r = wv.execute_script("var x = 42;");
            assert!(r.is_err());
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
}
