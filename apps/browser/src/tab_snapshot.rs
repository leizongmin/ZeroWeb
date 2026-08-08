//! 标签页渲染快照 — UI 线程只读合成，不触碰 WebView 内部状态。

use zero_engine::HitTestCache;
use zero_render_foundation::image_cache::ImageCache;
use zero_webview::WebViewRenderResult;

/// 标签页在 UI 线程上的只读快照。
#[derive(Default)]
pub struct TabSnapshot {
    /// 最近一次渲染图元。
    pub last_render: Option<WebViewRenderResult>,
    /// 图片子资源缓存（绘制 `<img>` 时消费）。
    pub image_cache: ImageCache,
    /// 文档布局高度（CSS 逻辑像素）。
    pub document_height: Option<f32>,
    /// 文档内容宽度估计（CSS 逻辑像素，图元下界）。
    ///
    /// 性能门禁优化 S3（2026-08-08）：在快照到达时计算一次并缓存，
    /// 避免 `document_size_physical` 在每次 mousemove/wheel 上对全部图元
    /// 做 O(P) 扫描（旧实现 `primitives_content_width` 每事件扫全量 fills+glyphs+images）。
    pub document_width: Option<f32>,
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
    /// 导航世代：每次 `begin_navigation` 递增，用于丢弃 stale ViewPainted。
    pub navigation_epoch: u64,
}

impl TabSnapshot {
    /// 从 WebView 状态构建快照（worker 线程内调用）。
    pub fn from_webview(wv: &zero_webview::WebView) -> Self {
        let html = wv.html_content();
        let last_render = wv.last_render().cloned();
        Self {
            document_width: last_render
                .as_ref()
                .map(|r| crate::page_scroll::primitives_content_width(&r.primitives)),
            last_render,
            image_cache: wv.snapshot_image_cache(),
            document_height: wv.document_height(),
            loading: wv.is_loading(),
            title: wv.title().map(str::to_string),
            url: wv.url().map(str::to_string),
            html_source: if html.is_empty() { None } else { Some(html.to_string()) },
            hit_test: wv.build_hit_test_cache(),
            navigation_epoch: 0,
        }
    }

    /// 导航开始：丢弃上一页绘制结果，避免 compositor 继续显示 stale 帧。
    pub fn begin_navigation(&mut self, url: String) {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
        self.clear_paint();
        self.loading = true;
        self.url = Some(url);
    }

    /// 清除 paint 与命中数据（保留 url/title/loading 由调用方设置）。
    pub fn clear_paint(&mut self) {
        self.last_render = None;
        self.document_height = None;
        self.document_width = None;
        self.hit_test = None;
        self.html_source = None;
        self.image_cache.clear();
    }

    /// 是否将 `last_render` 合成到屏幕（loading 期间即使有帧也不绘制 stale 内容）。
    pub fn should_composite_paint(&self) -> bool {
        self.last_render.is_some() && !self.loading
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};
    use zero_webview::WebViewRenderResult;

    fn blue_render() -> WebViewRenderResult {
        WebViewRenderResult {
            primitives: RenderPrimitives {
                fills: vec![FillPrimitive {
                    rect: Rect::new(0.0, 0.0, 10.0, 10.0),
                    color: Color::rgb(0, 0, 255),
                }],
                ..RenderPrimitives::new()
            },
            timings: Default::default(),
        }
    }

    #[test]
    fn begin_navigation_clears_paint_and_sets_loading() {
        let mut snap = TabSnapshot {
            last_render: Some(blue_render()),
            loading: false,
            url: Some("https://old.example".into()),
            ..Default::default()
        };

        snap.begin_navigation("https://new.example".into());
        assert_eq!(snap.navigation_epoch, 1);

        assert!(snap.last_render.is_none());
        assert!(snap.loading);
        assert!(!snap.should_composite_paint());
        assert_eq!(snap.url.as_deref(), Some("https://new.example"));
    }

    #[test]
    fn should_composite_paint_false_while_loading_even_with_frame() {
        let snap = TabSnapshot {
            last_render: Some(blue_render()),
            loading: true,
            ..Default::default()
        };
        assert!(!snap.should_composite_paint());
    }

    #[test]
    fn clear_paint_removes_render_but_not_url() {
        let mut snap = TabSnapshot {
            last_render: Some(blue_render()),
            url: Some("https://example.com".into()),
            ..Default::default()
        };
        snap.clear_paint();
        assert!(snap.last_render.is_none());
        assert_eq!(snap.url.as_deref(), Some("https://example.com"));
    }
}
