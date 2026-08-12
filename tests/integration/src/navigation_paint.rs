//! 导航与绘制快照契约：浏览器侧快照 + 渲染进程 WebView 状态。

use zero_protocol::message::{IpcMessage, IpcMessageKind};
use zero_protocol::transport::shared_channel_pair;
use zero_protocol::{IpcChannel, IpcColor, IpcFill, IpcRect, PaintSnapshotParams};
use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{FillPrimitive, RenderPrimitives};
use zero_webview::{WebView, WebViewConfig, WebViewRenderResult};

/// 浏览器侧最小快照（与 `zero-browser` 中 `TabSnapshot` 契约一致）。
struct BrowserPaintSnapshot {
    last_render: Option<WebViewRenderResult>,
    loading: bool,
    url: Option<String>,
    navigation_epoch: u64,
    document_generation: u64,
}

impl BrowserPaintSnapshot {
    fn begin_navigation(&mut self, url: String) {
        self.navigation_epoch = self.navigation_epoch.wrapping_add(1);
        self.last_render = None;
        self.loading = true;
        self.url = Some(url);
        self.document_generation = 0;
    }

    fn should_composite_paint(&self) -> bool {
        self.last_render.is_some() && !self.loading
    }

    fn apply_view_painted(&mut self, params: PaintSnapshotParams) {
        if params.navigation_epoch != self.navigation_epoch {
            return;
        }
        self.document_generation = params.document_generation;
        let mut primitives = RenderPrimitives::new();
        for fill in params.fills {
            primitives.fills.push(FillPrimitive {
                rect: Rect::new(fill.rect.x, fill.rect.y, fill.rect.width, fill.rect.height),
                color: Color::rgba(fill.color.r, fill.color.g, fill.color.b, fill.color.a),
            });
        }
        self.last_render = Some(WebViewRenderResult {
            primitives,
            dirty_rects: Vec::new(),
            timings: Default::default(),
        });
        if self.loading {
            self.loading = false;
        }
    }
}

fn red_paint_snapshot(epoch: u64) -> PaintSnapshotParams {
    PaintSnapshotParams {
        viewport_width: 800,
        viewport_height: 600,
        document_height: 400.0,
        navigation_epoch: epoch,
        document_generation: 1,
        fills: vec![IpcFill {
            rect: IpcRect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            color: IpcColor {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        }],
        rounded_rects: vec![],
        gradients: vec![],
        shadows: vec![],
        images: vec![],
        image_payloads: vec![],
        strokes: vec![],
        path_fills: vec![],
        path_strokes: vec![],
        clips: vec![],
        transforms: vec![],
        filters: vec![],
        blend_modes: vec![],
        glyph_text_runs: vec![],
        glyphs: vec![],
        draw_order: vec![],
        dirty_rects: vec![],
        hit_test: None,
    }
}

fn blue_render() -> WebViewRenderResult {
    WebViewRenderResult {
        primitives: RenderPrimitives {
            fills: vec![FillPrimitive {
                rect: Rect::new(0.0, 0.0, 50.0, 50.0),
                color: Color::rgb(0, 0, 255),
            }],
            ..RenderPrimitives::new()
        },
        dirty_rects: Vec::new(),
        timings: Default::default(),
    }
}

/// 导航后浏览器丢弃旧帧；首帧 ViewPainted 到达后才可合成新页。
#[test]
fn navigation_replaces_stale_browser_paint_with_view_painted() {
    let mut snap = BrowserPaintSnapshot {
        last_render: Some(blue_render()),
        loading: false,
        url: Some("https://old.example".into()),
        navigation_epoch: 0,
        document_generation: 7,
    };
    assert!(snap.should_composite_paint());

    snap.begin_navigation("https://new.example".into());
    assert!(!snap.should_composite_paint(), "loading 中不应合成旧帧");
    assert_eq!(snap.document_generation, 0);

    snap.apply_view_painted(red_paint_snapshot(snap.navigation_epoch));
    assert!(snap.should_composite_paint());
    assert_eq!(snap.document_generation, 1);
    let fill = &snap.last_render.as_ref().unwrap().primitives().fills[0];
    assert_eq!(fill.color.r, 255);
    assert_eq!(fill.color.g, 0);
}

/// 渲染进程 WebView：prepare_document_state 必须丢弃上一页 last_render。
#[test]
fn renderer_prepare_document_state_clears_stale_last_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body style='background:red'>A</body></html>", None);
    assert!(wv.last_render().is_some());
    assert!(!wv.is_loading());

    wv.prepare_document_state("https://new.example");
    assert!(wv.last_render().is_none());
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://new.example"));
}

/// 滞后的上一页 ViewPainted 不应覆盖新导航。
#[test]
fn stale_epoch_view_painted_is_ignored() {
    let mut snap = BrowserPaintSnapshot {
        last_render: None,
        loading: true,
        url: Some("https://new.example".into()),
        navigation_epoch: 2,
        document_generation: 0,
    };
    snap.apply_view_painted(red_paint_snapshot(1));
    assert!(snap.last_render.is_none());
    assert!(snap.loading);
}

/// IPC 通道：Navigate 后 ViewPainted 携带新绘制数据（不依赖真实子进程）。
#[test]
fn ipc_navigate_then_view_painted_lifecycle() {
    let (mut browser_ch, mut renderer_ch) = shared_channel_pair();
    let epoch = 3_u64;

    browser_ch
        .send(IpcMessage {
            id: 1,
            kind: IpcMessageKind::Navigate(zero_protocol::message::NavigateParams {
                url: "https://example.com".into(),
                referrer: None,
                navigation_epoch: epoch,
            }),
        })
        .unwrap();

    let nav = renderer_ch.recv().unwrap();
    assert!(matches!(nav.kind, IpcMessageKind::Navigate(_)));

    renderer_ch
        .send(IpcMessage {
            id: 2,
            kind: IpcMessageKind::ViewPainted(Box::new(red_paint_snapshot(epoch))),
        })
        .unwrap();

    let painted = browser_ch.recv().unwrap();
    if let IpcMessageKind::ViewPainted(params) = painted.kind {
        assert_eq!(params.fills[0].color.r, 255);
    } else {
        panic!("期望 ViewPainted");
    }
}
