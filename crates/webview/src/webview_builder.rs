//! WebView 构建器 — Builder pattern 创建 WebView。

use crate::webview::{WebView, WebViewConfig};

/// WebView 构建器。
pub struct WebViewBuilder {
    config: WebViewConfig,
}

impl WebViewBuilder {
    /// 创建新的 WebView 构建器。
    pub fn new() -> Self {
        Self {
            config: WebViewConfig::default(),
        }
    }

    /// 设置视口宽度。
    pub fn width(mut self, w: u32) -> Self {
        self.config.width = w;
        self
    }

    /// 设置视口高度。
    pub fn height(mut self, h: u32) -> Self {
        self.config.height = h;
        self
    }

    /// 设置是否透明背景。
    pub fn transparent(mut self, t: bool) -> Self {
        self.config.transparent = t;
        self
    }

    /// 设置用户代理字符串。
    pub fn user_agent(mut self, ua: &str) -> Self {
        self.config.user_agent = Some(ua.to_string());
        self
    }

    /// 设置初始 URL。
    pub fn url(mut self, url: &str) -> Self {
        self.config.url = Some(url.to_string());
        self
    }

    /// 设置是否启用开发者工具。
    pub fn devtools(mut self, enable: bool) -> Self {
        self.config.devtools = enable;
        self
    }

    /// 设置 HTTP 请求超时（秒）；`None` 使用默认 30s。
    pub fn http_timeout(mut self, secs: Option<u64>) -> Self {
        self.config.http_timeout_secs = secs;
        self
    }

    /// 使用外部 JS 执行器（专用 JS 线程），不在 WebView 内初始化 V8。
    pub fn external_script(mut self, executor: crate::ExternalScriptExecutor) -> Self {
        self.config.external_script = Some(executor);
        self
    }

    /// 构建 WebView 实例。
    ///
    /// 如果 `config.url` 已设置，会自动调用 `load_url` 将 WebView
    /// 置为加载状态（但不会同步发起网络请求）。
    pub fn build(self) -> WebView {
        let mut wv = WebView::new(self.config);
        if let Some(ref url) = wv.config().url {
            let url = url.clone();
            wv.load_url(&url);
        }
        wv
    }
}

impl Default for WebViewBuilder {
    fn default() -> Self {
        Self::new()
    }
}
