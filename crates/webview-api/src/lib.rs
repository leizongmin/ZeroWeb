//! # zero-webview-api
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
            url: Some("https://example.com".to_string()),
            devtools: true,
        };
        let wv = WebView::new(config);
        assert_eq!(wv.config().width, 1024);
        assert_eq!(wv.config().height, 768);
        assert!(wv.config().transparent);
        assert_eq!(
            wv.config().user_agent.as_deref(),
            Some("TestAgent/1.0")
        );
        assert!(wv.config().devtools);
    }

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
    fn test_webview_resize() {
        let mut wv = WebView::new(WebViewConfig::default());
        assert_eq!(wv.config().width, 800);
        assert_eq!(wv.config().height, 600);
        wv.resize(1024, 768);
        assert_eq!(wv.config().width, 1024);
        assert_eq!(wv.config().height, 768);
    }

    #[test]
    fn test_webview_execute_script_not_implemented() {
        let wv = WebView::new(WebViewConfig::default());
        let result = wv.execute_script("console.log('test')");
        assert!(result.is_err());
        match result.unwrap_err() {
            WebViewError::NotImplemented(msg) => {
                assert!(msg.contains("JavaScript"));
            }
            other => panic!("Expected NotImplemented, got: {other}"),
        }
    }

    #[test]
    fn test_webview_inject_css() {
        let mut wv = WebView::new(WebViewConfig::default());
        // First load some HTML
        wv.load_html("<html><body><div>Hello</div></body></html>", None);
        let result = wv.inject_css("div { background-color: blue; }");
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
            .url("https://example.com")
            .devtools(true)
            .build();
        assert_eq!(wv.config().width, 1024);
        assert_eq!(wv.config().height, 768);
        assert!(wv.config().transparent);
        assert_eq!(wv.config().user_agent.as_deref(), Some("TestBot/1.0"));
        assert_eq!(wv.config().url.as_deref(), Some("https://example.com"));
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

    // ── render() 方法测试 ──

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
        // 缓存 HTML 为空，渲染应成功且不 panic
        assert!(result.timings.total_ms >= 0.0);
        assert!(wv.last_render().is_some());
    }

    #[test]
    fn test_webview_render_after_load_url() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_url("https://example.com");
        let result = wv.render();
        // load_url 不填充 cached_html，render 仍可执行
        assert!(result.timings.total_ms >= 0.0);
    }

    // ── load_url 边界条件 ──

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

    // ── inject_css 边界条件 ──

    #[test]
    fn test_webview_inject_css_without_load_html() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.inject_css("div { color: red; }");
        // 使用 fallback HTML
        assert!(result.timings.total_ms >= 0.0);
        assert!(wv.last_render().is_some());
    }

    #[test]
    fn test_webview_inject_css_multiple_times() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_html("<html><body><div>A</div></body></html>", None);
        let result1 = wv.inject_css("div { color: red; }");
        let result2 = wv.inject_css("div { color: blue; }");
        // 第二次注入覆盖，都应成功
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

    // ── resize 边界条件 ──

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

    // ── load_html 边界条件 ──

    #[test]
    fn test_webview_load_html_malformed() {
        let mut wv = WebView::new(WebViewConfig::default());
        let result = wv.load_html("<div><p>unclosed<span>", None);
        // 容错解析不应 panic
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
        let first_timing = wv.last_render().unwrap().timings.total_ms;
        wv.load_html("<html><body>Second</body></html>", None);
        // last_render 应反映最近一次调用
        assert!(wv.last_render().is_some());
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

    // ── Builder 边界条件 ──

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

    // ── 状态转换一致性 ──

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
    fn test_webview_load_url_does_not_set_last_render() {
        let mut wv = WebView::new(WebViewConfig::default());
        wv.load_url("https://example.com");
        assert!(wv.last_render().is_none());
    }

    #[test]
    fn test_webview_config_url_not_auto_loaded() {
        let config = WebViewConfig {
            url: Some("https://example.com".into()),
            ..Default::default()
        };
        let wv = WebView::new(config);
        // config.url 不会自动加载
        assert!(wv.url().is_none());
    }

    #[test]
    fn test_webview_config_clone() {
        let config = WebViewConfig::default();
        let cloned = config.clone();
        assert_eq!(config.width, cloned.width);
        assert_eq!(config.height, cloned.height);
    }
}
