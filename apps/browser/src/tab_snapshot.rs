//! 标签页渲染快照 — UI 线程只读合成，不触碰 WebView 内部状态。

use zero_engine::HitTestCache;
use zero_render_foundation::image_cache::{ImageCache, ImageData, ImageKey};
use zero_render_foundation::primitive::TextControlBoundary;
use zero_webview::WebViewRenderResult;

/// Browser 已接收并提交给 compositor 的最新页面帧标识。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompositorSubmission {
    /// Renderer surface 标识。
    pub surface_id: u64,
    /// 导航世代。
    pub navigation_epoch: u64,
    /// Renderer 帧序号。
    pub frame_id: u64,
}

/// Browser 可显示的最新 compositor RGBA 位图。
///
/// RGBA 字节由同一 [`TabSnapshot`] 的 `image_cache` 持有，`image_key` 指向该缓存，
/// 从 compositor 接收后只发生一次所有权移动。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositorFrame {
    /// Renderer surface 标识。
    pub surface_id: u64,
    /// 导航世代。
    pub navigation_epoch: u64,
    /// Renderer 帧序号。
    pub frame_id: u64,
    /// 位图宽度（像素）。
    pub width: u32,
    /// 位图高度（像素）。
    pub height: u32,
    /// RGBA 位图在当前 Tab 图片缓存中的键。
    pub image_key: ImageKey,
    /// Linux：GPU 直接导入（跳过 image_cache CPU 上传）。
    #[cfg(target_os = "linux")]
    pub gpu_direct: bool,
}

/// Linux compositor dma-buf 待 GPU 导入。
#[cfg(target_os = "linux")]
pub struct CompositorDmabufPending {
    pub fd: std::os::fd::OwnedFd,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub drm_fourcc: u32,
    pub drm_modifier: u64,
    pub dst_x: f32,
    pub dst_y: f32,
}

/// 标签页在 UI 线程上的只读快照。
#[derive(Default)]
pub struct TabSnapshot {
    /// 最近一次渲染图元。
    pub last_render: Option<WebViewRenderResult>,
    /// 文本控件 caret 边界交互元数据（compositor 模式也保留）。
    pub text_control_boundaries: Vec<TextControlBoundary>,
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
    /// 当前导航内的 Document 世代。
    pub document_generation: u64,
    /// 已提交给 compositor 的最新页面帧。
    pub compositor_submission: Option<CompositorSubmission>,
    /// compositor 已完成且可显示的最新页面位图。
    pub compositor_frame: Option<CompositorFrame>,
    /// compositor 合成的全窗口 present 位图（RFC 4.4-S3；`ZW_COMPOSITOR_PRESENT=1`）。
    pub compositor_present: Option<CompositorFrame>,
    /// compositor 回读的滚动偏移（RFC 4.2；异步滚动默认开时用于显示）。
    pub compositor_scroll: Option<(f32, f32)>,
    /// Linux：待 Browser GPU 导入的 compositor dma-buf。
    #[cfg(target_os = "linux")]
    pub compositor_dmabuf: Option<CompositorDmabufPending>,
}

impl TabSnapshot {
    /// 从 WebView 状态构建快照（worker 线程内调用）。
    pub fn from_webview(wv: &zero_webview::WebView) -> Self {
        let html = wv.html_content();
        let last_render = wv.last_render().cloned();
        let text_control_boundaries = last_render
            .as_ref()
            .map(|render| render.primitives.text_control_boundaries.clone())
            .unwrap_or_default();
        Self {
            document_width: last_render
                .as_ref()
                .map(|r| crate::page_scroll::primitives_content_width(&r.primitives)),
            last_render,
            text_control_boundaries,
            image_cache: wv.snapshot_image_cache(),
            document_height: wv.document_height(),
            loading: wv.is_loading(),
            title: wv.title().map(str::to_string),
            url: wv.url().map(str::to_string),
            html_source: if html.is_empty() { None } else { Some(html.to_string()) },
            hit_test: wv.build_hit_test_cache(),
            navigation_epoch: 0,
            document_generation: 1,
            compositor_submission: None,
            compositor_frame: None,
            compositor_present: None,
            compositor_scroll: None,
            #[cfg(target_os = "linux")]
            compositor_dmabuf: None,
        }
    }

    /// 导航开始：丢弃上一页绘制结果，避免 compositor 继续显示 stale 帧。
    pub fn begin_navigation(&mut self, url: String) {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
        self.document_generation = 0;
        self.clear_paint();
        self.loading = true;
        self.url = Some(url);
    }

    /// 清除 paint 与命中数据（保留 url/title/loading 由调用方设置）。
    pub fn clear_paint(&mut self) {
        self.last_render = None;
        self.text_control_boundaries.clear();
        self.compositor_submission = None;
        self.compositor_frame = None;
        self.compositor_present = None;
        self.compositor_scroll = None;
        self.document_height = None;
        self.document_width = None;
        self.hit_test = None;
        self.html_source = None;
        self.image_cache.clear();
    }

    /// 清除 compositor 提交、完成位图及其图片缓存，保留 legacy 页面快照。
    pub fn clear_compositor_state(&mut self) {
        self.compositor_submission = None;
        self.compositor_frame = None;
        self.compositor_present = None;
        self.compositor_scroll = None;
        self.image_cache.clear();
    }

    /// 是否将 `last_render` 合成到屏幕（loading 期间即使有帧也不绘制 stale 内容）。
    pub fn should_composite_paint(&self) -> bool {
        self.last_render.is_some() && !self.loading
    }

    /// 记录准备提交给 compositor 的 renderer 帧；拒绝错误世代和倒序帧。
    pub fn record_compositor_submission(&mut self, submission: CompositorSubmission) -> bool {
        if submission.navigation_epoch != self.navigation_epoch {
            return false;
        }
        if let Some(current) = self.compositor_submission
            && current.surface_id == submission.surface_id
            && current.navigation_epoch == submission.navigation_epoch
            && current.frame_id >= submission.frame_id
        {
            return false;
        }
        self.compositor_submission = Some(submission);
        true
    }

    /// 接收 compositor 完成位图；只接受与最新提交完全匹配且不旧于当前显示帧的结果。
    pub fn commit_compositor_frame(
        &mut self,
        submission: CompositorSubmission,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
        scroll_x: f32,
        scroll_y: f32,
    ) -> bool {
        if self.compositor_submission != Some(submission)
            || self.compositor_frame.as_ref().is_some_and(|current| {
                current.surface_id == submission.surface_id
                    && (current.navigation_epoch, current.frame_id) > (submission.navigation_epoch, submission.frame_id)
            })
        {
            return false;
        }
        let Ok(image) = ImageData::from_rgba(rgba, width, height) else {
            return false;
        };
        self.image_cache.clear();
        let image_key = self.image_cache.insert(image);
        self.compositor_present = None;
        self.compositor_frame = Some(CompositorFrame {
            surface_id: submission.surface_id,
            navigation_epoch: submission.navigation_epoch,
            frame_id: submission.frame_id,
            width,
            height,
            image_key,
            #[cfg(target_os = "linux")]
            gpu_direct: false,
        });
        self.compositor_scroll = Some((scroll_x, scroll_y));
        true
    }

    /// Linux：接收 compositor dma-buf（GPU 导入路径）。
    ///
    /// `shadow_rgba`：dmabuf 的 CPU 影子副本——headless GPU 捕获渲染器（冒烟/
    /// parity 采集）没有导入纹理，需回退位图绘制；窗口渲染仍走 compositor_import，
    /// 不双绘。`None` 时保留占位键（gpu_direct 帧在捕获场景外不消费位图）。
    #[cfg(target_os = "linux")]
    #[allow(clippy::too_many_arguments)]
    pub fn commit_compositor_dmabuf(
        &mut self,
        submission: CompositorSubmission,
        width: u32,
        height: u32,
        scroll_x: f32,
        scroll_y: f32,
        dmabuf: CompositorDmabufPending,
        shadow_rgba: Option<Vec<u8>>,
    ) -> bool {
        if self.compositor_submission != Some(submission)
            || self.compositor_frame.as_ref().is_some_and(|current| {
                current.surface_id == submission.surface_id
                    && (current.navigation_epoch, current.frame_id)
                        >= (submission.navigation_epoch, submission.frame_id)
            })
        {
            return false;
        }
        self.image_cache.clear();
        let image_key = shadow_rgba
            .and_then(|rgba| {
                zero_render_foundation::image_cache::ImageData::from_rgba(rgba, width, height)
                    .ok()
                    .map(|image| self.image_cache.insert(image))
            })
            .unwrap_or_else(|| ImageKey::new(0));
        self.compositor_present = None;
        self.compositor_frame = Some(CompositorFrame {
            surface_id: submission.surface_id,
            navigation_epoch: submission.navigation_epoch,
            frame_id: submission.frame_id,
            width,
            height,
            image_key,
            gpu_direct: true,
        });
        self.compositor_dmabuf = Some(dmabuf);
        self.compositor_scroll = Some((scroll_x, scroll_y));
        true
    }

    /// 取走待导入 dma-buf（每帧消费一次）。
    #[cfg(target_os = "linux")]
    pub fn take_compositor_dmabuf(&mut self) -> Option<CompositorDmabufPending> {
        self.compositor_dmabuf.take()
    }

    /// 接收 compositor 全窗口 present 位图；须与当前 page surface 匹配。
    pub fn commit_compositor_present_frame(
        &mut self,
        page_surface_id: u64,
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    ) -> bool {
        let Some(page) = self.compositor_frame.as_ref() else {
            return false;
        };
        if page.surface_id != page_surface_id {
            return false;
        }
        let Ok(image) = ImageData::from_rgba(rgba, width, height) else {
            return false;
        };
        let image_key = self.image_cache.insert(image);
        self.compositor_present = Some(CompositorFrame {
            surface_id: page_surface_id,
            navigation_epoch: page.navigation_epoch,
            frame_id: page.frame_id,
            width,
            height,
            image_key,
            #[cfg(target_os = "linux")]
            gpu_direct: false,
        });
        true
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
            dirty_rects: Vec::new(),
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

    #[test]
    fn compositor_frame_accepts_only_latest_submission_and_moves_pixels_to_cache() {
        let mut snap = TabSnapshot {
            navigation_epoch: 3,
            ..Default::default()
        };
        let first = CompositorSubmission {
            surface_id: 41,
            navigation_epoch: 3,
            frame_id: 7,
        };
        let latest = CompositorSubmission { frame_id: 8, ..first };
        assert!(snap.record_compositor_submission(first));
        assert!(snap.record_compositor_submission(latest));
        assert!(!snap.commit_compositor_frame(first, 1, 1, vec![255, 0, 0, 255], 0.0, 0.0));
        assert!(snap.commit_compositor_frame(latest, 1, 1, vec![0, 0, 255, 255], 0.0, 0.0));

        let frame = snap.compositor_frame.as_ref().unwrap();
        assert_eq!((frame.surface_id, frame.navigation_epoch, frame.frame_id), (41, 3, 8));
        assert_eq!(snap.image_cache.get(&frame.image_key).unwrap().pixels, [0, 0, 255, 255]);
    }

    #[test]
    fn compositor_frame_accepts_refreshed_pixels_for_the_same_submission() {
        let mut snap = TabSnapshot {
            navigation_epoch: 3,
            ..Default::default()
        };
        let submission = CompositorSubmission {
            surface_id: 41,
            navigation_epoch: 3,
            frame_id: 8,
        };
        assert!(snap.record_compositor_submission(submission));
        assert!(snap.commit_compositor_frame(submission, 1, 1, vec![255, 0, 0, 255], 0.0, 0.0));

        // 滚动不生成新的 renderer 帧；合成器重绘后须能替换相同 submission 的位图。
        assert!(snap.commit_compositor_frame(submission, 1, 1, vec![0, 0, 255, 255], 0.0, 0.0));
        let frame = snap.compositor_frame.as_ref().unwrap();
        assert_eq!(snap.image_cache.get(&frame.image_key).unwrap().pixels, [0, 0, 255, 255]);
    }

    #[test]
    fn compositor_frames_are_isolated_per_tab_snapshot() {
        let mut first = TabSnapshot {
            navigation_epoch: 1,
            ..Default::default()
        };
        let mut second = TabSnapshot {
            navigation_epoch: 1,
            ..Default::default()
        };
        let first_key = CompositorSubmission {
            surface_id: 10,
            navigation_epoch: 1,
            frame_id: 1,
        };
        let second_key = CompositorSubmission {
            surface_id: 20,
            navigation_epoch: 1,
            frame_id: 1,
        };
        assert!(first.record_compositor_submission(first_key));
        assert!(second.record_compositor_submission(second_key));
        assert!(first.commit_compositor_frame(first_key, 1, 1, vec![255, 0, 0, 255], 0.0, 0.0));
        assert!(second.commit_compositor_frame(second_key, 1, 1, vec![0, 255, 0, 255], 0.0, 0.0));

        assert_eq!(first.compositor_frame.as_ref().unwrap().surface_id, 10);
        assert_eq!(second.compositor_frame.as_ref().unwrap().surface_id, 20);
        let first_frame = first.compositor_frame.as_ref().unwrap();
        let second_frame = second.compositor_frame.as_ref().unwrap();
        assert_eq!(first.image_cache.get(&first_frame.image_key).unwrap().pixels[0], 255);
        assert_eq!(second.image_cache.get(&second_frame.image_key).unwrap().pixels[1], 255);
    }

    #[test]
    fn compositor_state_clear_preserves_legacy_render() {
        let mut snap = TabSnapshot {
            navigation_epoch: 1,
            last_render: Some(blue_render()),
            ..Default::default()
        };
        let submission = CompositorSubmission {
            surface_id: 8,
            navigation_epoch: 1,
            frame_id: 2,
        };
        assert!(snap.record_compositor_submission(submission));
        assert!(snap.commit_compositor_frame(submission, 1, 1, vec![1, 2, 3, 4], 0.0, 0.0));

        snap.clear_compositor_state();

        assert!(snap.compositor_submission.is_none());
        assert!(snap.compositor_frame.is_none());
        assert!(snap.last_render.is_some());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn commit_compositor_dmabuf_accepts_latest_submission() {
        use std::os::fd::{FromRawFd, IntoRawFd};

        let mut snap = TabSnapshot {
            navigation_epoch: 3,
            ..Default::default()
        };
        let first = CompositorSubmission {
            surface_id: 41,
            navigation_epoch: 3,
            frame_id: 7,
        };
        let latest = CompositorSubmission { frame_id: 8, ..first };
        assert!(snap.record_compositor_submission(first));
        assert!(snap.record_compositor_submission(latest));

        let null_fd = std::fs::File::open("/dev/null").expect("open /dev/null");
        let raw_fd = null_fd.into_raw_fd();
        assert!(snap.commit_compositor_dmabuf(
            latest,
            2,
            2,
            0.0,
            0.0,
            CompositorDmabufPending {
                fd: unsafe { std::os::fd::OwnedFd::from_raw_fd(raw_fd) },
                width: 2,
                height: 2,
                stride: 8,
                drm_fourcc: 0x3432_4241,
                drm_modifier: 0,
                dst_x: 0.0,
                dst_y: 0.0,
            },
            // 影子路径：gpu_direct 帧的位图 key 指向缓存中的 RGBA 副本。
            Some(vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255]),
        ));

        let frame = snap.compositor_frame.as_ref().unwrap();
        assert!(frame.gpu_direct);
        assert!(
            snap.image_cache.get(&frame.image_key).is_some(),
            "影子 RGBA 应以帧位图键入缓存（捕获路径回退绘制）"
        );
        assert_eq!((frame.surface_id, frame.navigation_epoch, frame.frame_id), (41, 3, 8));
        assert!(snap.compositor_dmabuf.is_some());
        assert!(snap.take_compositor_dmabuf().is_some());
    }
}
