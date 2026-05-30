//! WebView 主类型 — 可嵌入的网页渲染表面。

use zero_engine::{PipelineTimings, RenderPipeline};
use zero_render_foundation::primitive::RenderPrimitives;

use crate::WebViewError;

/// WebView 配置。
#[derive(Debug, Clone)]
pub struct WebViewConfig {
    /// 视口宽度。
    pub width: u32,
    /// 视口高度。
    pub height: u32,
    /// 是否透明背景。
    pub transparent: bool,
    /// 用户代理字符串。
    pub user_agent: Option<String>,
    /// 初始 URL。
    pub url: Option<String>,
    /// 是否启用开发者工具。
    pub devtools: bool,
}

impl Default for WebViewConfig {
    fn default() -> Self {
        Self {
            width: 800,
            height: 600,
            transparent: false,
            user_agent: None,
            url: None,
            devtools: false,
        }
    }
}

/// WebView 渲染结果。
#[derive(Debug, Clone)]
pub struct WebViewRenderResult {
    /// 渲染图元。
    pub primitives: RenderPrimitives,
    /// 管线耗时。
    pub timings: PipelineTimings,
}

/// WebView 事件回调。
#[derive(Debug, Clone)]
pub enum WebViewEvent {
    /// 页面开始加载。
    LoadStart(String),
    /// 页面加载完成。
    LoadEnd(String),
    /// 页面加载失败。
    LoadFailed(String, String),
    /// 标题变更。
    TitleChanged(String),
    /// URL 变更。
    UrlChanged(String),
}

/// WebView — 可嵌入的网页渲染表面。
pub struct WebView {
    /// 配置。
    config: WebViewConfig,
    /// 渲染管线。
    pipeline: RenderPipeline,
    /// 当前 URL。
    current_url: Option<String>,
    /// 页面标题。
    title: Option<String>,
    /// 是否正在加载。
    loading: bool,
    /// 上次渲染结果。
    last_render: Option<WebViewRenderResult>,
    /// 缓存的 HTML（用于 inject_css 重新渲染）。
    cached_html: String,
}

impl WebView {
    /// 创建新的 WebView。
    pub fn new(config: WebViewConfig) -> Self {
        let pipeline = RenderPipeline::new(config.width as f32, config.height as f32);
        Self {
            config,
            pipeline,
            current_url: None,
            title: None,
            loading: false,
            last_render: None,
            cached_html: String::new(),
        }
    }

    /// 加载 HTML 内容。
    pub fn load_html(&mut self, html: &str, css: Option<&str>) -> WebViewRenderResult {
        self.cached_html = html.to_string();
        let css_str = css.unwrap_or("");
        let result = self.pipeline.render_html(html, css_str);
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result.clone());
        render_result
    }

    /// 加载 URL（需要网络栈，暂返回占位结果）。
    pub fn load_url(&mut self, url: &str) {
        self.current_url = Some(url.to_string());
        self.loading = true;
    }

    /// 重新渲染（用于 resize 等场景）。
    pub fn render(&mut self) -> WebViewRenderResult {
        let css = "";
        let result = self.pipeline.render_html(&self.cached_html, css);
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result.clone());
        render_result
    }

    /// 获取当前 URL。
    pub fn url(&self) -> Option<&str> {
        self.current_url.as_deref()
    }

    /// 获取页面标题。
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// 获取配置。
    pub fn config(&self) -> &WebViewConfig {
        &self.config
    }

    /// 是否正在加载。
    pub fn is_loading(&self) -> bool {
        self.loading
    }

    /// 获取上次渲染结果。
    pub fn last_render(&self) -> Option<&WebViewRenderResult> {
        self.last_render.as_ref()
    }

    /// 调整大小。
    pub fn resize(&mut self, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.pipeline = RenderPipeline::new(width as f32, height as f32);
    }

    /// 执行 JavaScript（占位 — 需要 V8）。
    pub fn execute_script(&self, _script: &str) -> Result<(), WebViewError> {
        Err(WebViewError::NotImplemented(
            "JavaScript execution requires V8 engine".to_string(),
        ))
    }

    /// 注入 CSS（重新渲染）。
    pub fn inject_css(&mut self, css: &str) -> WebViewRenderResult {
        let html = if self.cached_html.is_empty() {
            "<html><body></body></html>"
        } else {
            &self.cached_html
        };
        let result = self.pipeline.render_html(html, css);
        let render_result = WebViewRenderResult {
            primitives: result.primitives,
            timings: result.timings,
        };
        self.last_render = Some(render_result.clone());
        render_result
    }
}
