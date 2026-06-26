//! 标签页渲染快照 — UI 线程只读合成，不触碰 WebView 内部状态。

use zero_engine::HitTestCache;
use zero_render_foundation::image_cache::ImageCache;
use zero_webview::WebViewRenderResult;

/// 标签页在 UI 线程上的只读快照。
pub struct TabSnapshot {
    /// 最近一次渲染图元。
    pub last_render: Option<WebViewRenderResult>,
    /// 图片子资源缓存（绘制 `<img>` 时消费）。
    pub image_cache: ImageCache,
    /// 文档布局高度（CSS 逻辑像素）。
    pub document_height: Option<f32>,
    /// 是否仍在加载。
    pub loading: bool,
    /// 页面标题。
    pub title: Option<String>,
    /// 当前 URL。
    pub url: Option<String>,
    /// 最近一次渲染使用的 HTML 源码。
    pub html_source: Option<String>,
    /// 主线程命中测试数据（与 `last_render` 同帧）。
    pub hit_test: Option<HitTestCache>,
}

impl Default for TabSnapshot {
    fn default() -> Self {
        Self {
            last_render: None,
            image_cache: ImageCache::default(),
            document_height: None,
            loading: false,
            title: None,
            url: None,
            html_source: None,
            hit_test: None,
        }
    }
}

impl TabSnapshot {
    /// 从 WebView 状态构建快照（worker 线程内调用）。
    pub fn from_webview(wv: &zero_webview::WebView) -> Self {
        let html = wv.html_content();
        Self {
            last_render: wv.last_render().cloned(),
            image_cache: wv.snapshot_image_cache(),
            document_height: wv.document_height(),
            loading: wv.is_loading(),
            title: wv.title().map(str::to_string),
            url: wv.url().map(str::to_string),
            html_source: if html.is_empty() { None } else { Some(html.to_string()) },
            hit_test: wv.build_hit_test_cache(),
        }
    }
}
