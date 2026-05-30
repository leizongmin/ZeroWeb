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
}
