//! WebViewWidget 数据模型与后端 trait（spec IF-004 / FR-005）。

use zero_ui_core::action::EventResult;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Rect, Size};
use zero_ui_core::scroll::ScrollMetrics;
use zero_ui_core::semantics::{SemanticsFlags, SemanticsLabel, SemanticsNode};
use zero_ui_core::theme::Theme;
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, SemanticsCtx, UpdateCtx, Widget, WidgetId};

/// WebView 布局输入（spec IF-004 `WebViewLayoutInput`）：UI SDK 分配给 WebViewWidget 的外部参数。
#[derive(Debug, Clone)]
pub struct WebViewLayoutInput {
    pub rect: Rect,
    pub scale_factor: f32,
    pub theme: Theme,
}

/// WebView 绘制输出（spec IF-004 `WebViewPaintOutput`）：WebView 上报给 UI SDK 的滚动度量。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WebViewPaintOutput {
    pub scroll_metrics: ScrollMetrics,
}

/// WebView 后端 trait：由 `zero-webview` 适配实现；UI SDK 通过它取渲染结果合成进 scene。
///
/// 方法签名引用 `zero_webview::WebViewRenderResult`，确立本 crate 与 zero-webview 的耦合边界。
pub trait WebviewBackend {
    /// 给定 layout 输入，渲染并返回 zero-webview 的原始结果。
    fn render(&mut self, input: &WebViewLayoutInput) -> zero_webview::WebViewRenderResult;
}

/// WebViewWidget — UI 树中的 WebView 自定义组件（spec FR-005）。
///
/// 只持有外部几何 + 当前 scroll 度量；DOM/页面布局完全由 zero-webview 负责。
pub struct WebViewWidget {
    pub viewport: Rect,
    pub scale_factor: f32,
    pub theme: Theme,
    pub scroll: ScrollMetrics,
    /// 宿主分配的外部表面 id（paint 记录为 `ExternalSurface` 图元；后端按 id 取回纹理合成，DC-3）。
    pub surface_id: u64,
}

impl WebViewWidget {
    pub fn new(viewport: Rect, scale_factor: f32, theme: Theme) -> WebViewWidget {
        WebViewWidget {
            viewport,
            scale_factor,
            theme,
            scroll: ScrollMetrics {
                content_width: viewport.size.width,
                content_height: viewport.size.height,
                viewport_width: viewport.size.width,
                viewport_height: viewport.size.height,
                scroll_x: 0.0,
                scroll_y: 0.0,
            },
            surface_id: 0,
        }
    }

    /// 设置外部表面 id（宿主为每个 WebViewWidget 分配唯一 id）。
    pub fn with_surface_id(mut self, surface_id: u64) -> WebViewWidget {
        self.surface_id = surface_id;
        self
    }

    /// 生成 layout 输入（交给 zero-webview）。
    pub fn layout_input(&self) -> WebViewLayoutInput {
        WebViewLayoutInput {
            rect: self.viewport,
            scale_factor: self.scale_factor,
            theme: self.theme.clone(),
        }
    }

    /// 宿主把 WebView 上报的 scroll 度量推回（页面内容尺寸/scroll offset 由 WebView 管理，spec FR-006）。
    pub fn set_scroll_metrics(&mut self, metrics: ScrollMetrics) {
        self.scroll = metrics;
    }

    /// 当前 paint 输出（含 scroll 度量，供 ScrollBar 几何计算用）。
    pub fn paint_output(&self) -> WebViewPaintOutput {
        WebViewPaintOutput {
            scroll_metrics: self.scroll,
        }
    }
}

impl Widget for WebViewWidget {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _props: &zero_ui_core::widget::Props) {}

    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        // 滚动事件：更新 scroll 度量（页面内容尺寸/offset 由 WebView 管控，spec FR-006）。
        if let UiEvent::Scroll { delta, .. } = event {
            self.scroll.scroll_x = (self.scroll.scroll_x + delta.x).clamp(0.0, self.scroll.max_scroll_x());
            self.scroll.scroll_y = (self.scroll.scroll_y + delta.y).clamp(0.0, self.scroll.max_scroll_y());
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // UI SDK 只算 WebViewWidget 外部矩形（spec FR-005）：填充分配区域；
        // DOM/layout/paint 完全由 zero-webview 负责。
        Size::new(constraints.max_width, constraints.max_height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        // 记录 ExternalSurface 图元（外部矩形 + surface_id）；真实纹理由后端按 id 合成。
        // ctx.clip.size = 节点可视尺寸（host 按绝对 origin 平移后覆盖节点 rect）。
        let size = ctx.clip.map(|r| r.size).unwrap_or(self.viewport.size);
        ctx.recorder
            .draw_external_surface(Rect::from_ltrb(0.0, 0.0, size.width, size.height), self.surface_id);
    }

    fn semantics(&self, ctx: &mut SemanticsCtx) {
        // WebView 是一个合成表面，a11y 树上为单个节点（网页内部 a11y tree 由 WebView/engine 负责）。
        ctx.nodes.push(SemanticsNode {
            id: WidgetId::new("webview"),
            rect: self.viewport,
            flags: SemanticsFlags::default(),
            label: Some(SemanticsLabel::Literal("web content".into())),
            value: None,
            children: Vec::new(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::event::{Modifiers, ScrollPhase};
    use zero_ui_core::geometry::{Point, Size, Vec2};
    use zero_ui_core::invalidation::InvalidationFlags;
    use zero_ui_core::theme::{ColorPalette, ResolvedColorScheme, ThemeId, ThemeResolver};
    use zero_ui_core::widget::WidgetSpec;
    use zero_ui_render::SceneRecorder;
    use zero_ui_render::render_node::RenderPrimitive;

    fn theme() -> Theme {
        ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::Light,
            ColorPalette::default(),
        )
    }

    #[test]
    fn widget_only_tracks_external_geometry() {
        let rect = Rect::from_origin_size(Default::default(), Size::new(800.0, 600.0));
        let mut w = WebViewWidget::new(rect, 2.0, theme());
        // 初始 scroll 度量 = viewport（无溢出）。
        assert_eq!(w.scroll.viewport_width, 800.0);
        // 宿主推回页面真实度量。
        w.set_scroll_metrics(ScrollMetrics {
            content_width: 800.0,
            content_height: 3000.0,
            viewport_width: 800.0,
            viewport_height: 600.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
        });
        assert_eq!(w.paint_output().scroll_metrics.content_height, 3000.0);
    }

    #[test]
    fn layout_input_carries_viewport_and_theme() {
        let rect = Rect::from_origin_size(Default::default(), Size::new(400.0, 300.0));
        let w = WebViewWidget::new(rect, 1.5, theme());
        let input = w.layout_input();
        assert_eq!(input.scale_factor, 1.5);
        assert_eq!(input.rect.size.width, 400.0);
    }

    #[test]
    fn paint_records_external_surface_marker() {
        // DC-3：WebViewWidget paint 把自身记录为 ExternalSurface 图元（外部矩形 + surface_id），
        // 不把网页 DOM 映射为 UI widgets。
        let rect = Rect::from_origin_size(Default::default(), Size::new(800.0, 600.0));
        let mut w = WebViewWidget::new(rect, 1.0, theme()).with_surface_id(99);
        let mut rec = SceneRecorder::new(WidgetId::new("wv"));
        rec.set_clip(Some(rect));
        let mut ctx = PaintCtx {
            recorder: &mut rec,
            clip: Some(rect),
            offset: Vec2::ZERO,
            tokens: &zero_ui_core::theme::SemanticTokens::light(),
            font_metrics: None,
            now_ms: None,
            frame_requests: &std::cell::Cell::new(0),
        };
        w.paint(&mut ctx);
        let scene = rec.finish();
        assert_eq!(scene.entries.len(), 1);
        match &scene.entries[0].primitive {
            RenderPrimitive::ExternalSurface { rect: r, surface_id } => {
                assert_eq!(*surface_id, 99);
                assert_eq!(r.size, Size::new(800.0, 600.0));
            }
            other => panic!("expected ExternalSurface, got {other:?}"),
        }
    }

    #[test]
    fn scroll_event_updates_metrics_clamped() {
        // DC-3/DC-4：滚动事件更新 WebView 内部 scroll 度量（页面 offset 由 WebView 管控）。
        let rect = Rect::from_origin_size(Default::default(), Size::new(400.0, 300.0));
        let mut w = WebViewWidget::new(rect, 1.0, theme());
        w.set_scroll_metrics(ScrollMetrics {
            content_width: 400.0,
            content_height: 1000.0,
            viewport_width: 400.0,
            viewport_height: 300.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
        });
        let mut flags = InvalidationFlags::CLEAN;
        let scroll_by = |dy: f32| UiEvent::Scroll {
            delta: Vec2::new(0.0, dy),
            phase: ScrollPhase::Discrete,
            position: Point::ZERO,
            modifiers: Modifiers::NONE,
        };
        assert_eq!(
            w.event(
                &mut EventCtx {
                    invalidation: &mut flags
                },
                &scroll_by(100.0)
            ),
            EventResult::Consumed
        );
        assert_eq!(w.scroll.scroll_y, 100.0);
        // 超过 max_scroll_y（1000-300=700）→ 钳到 700。
        let _ = w.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &scroll_by(10_000.0),
        );
        assert_eq!(w.scroll.scroll_y, 700.0);
    }

    #[test]
    fn host_paints_webview_as_external_surface() {
        // DC-3 集成：WebViewWidget 注册进 WidgetHost → layout → paint → 统一 Scene 含 ExternalSurface。
        use zero_ui_runtime::WidgetHost;
        let mut host = WidgetHost::new();
        host.register("WebView", |_spec| {
            Box::new(WebViewWidget::new(Rect::from_ltrb(0.0, 0.0, 800.0, 600.0), 1.0, theme()).with_surface_id(7))
        });
        let mut spec = WidgetSpec::new("WebView");
        spec.id = Some(WidgetId::new("wv"));
        host.set_root(&spec);
        host.layout(Constraints::loose(Size::new(800.0, 600.0)));
        let scene = host.paint().clone();
        assert!(
            scene
                .entries
                .iter()
                .any(|e| matches!(e.primitive, RenderPrimitive::ExternalSurface { surface_id: 7, .. })),
            "scene should contain the webview external surface, got {:?}",
            scene.entries
        );
    }

    #[test]
    fn scrollbar_drag_flows_to_webview_scroll() {
        // DC-4 闭环：通用 ScrollBar 拖动 → ScrollCommand → scroll_bridge::apply_scroll_command
        // → WebViewWidget.set_scroll_metrics（页面 offset 由 WebView 管理，spec FR-006）。
        use zero_ui_widgets::scrollbar::{ScrollOrientation, drag_to_command, layout_scrollbar};
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let mut metrics = ScrollMetrics {
            content_width: 200.0,
            content_height: 1000.0,
            viewport_width: 200.0,
            viewport_height: 200.0,
            scroll_x: 0.0,
            scroll_y: 100.0,
        };
        let geom = layout_scrollbar(viewport, metrics, ScrollOrientation::Vertical).unwrap();
        let on_thumb = Point::new(geom.thumb.left() + 1.0, geom.thumb.top() + 1.0);
        let cmd = drag_to_command(&geom, metrics, on_thumb, Point::new(on_thumb.x, on_thumb.y + 16.0))
            .expect("drag on thumb produces a command");
        // scroll_bridge 把命令钳制为最终 offset。
        let (_tx, ty) = crate::apply_scroll_command(metrics, cmd);
        let mut wv = WebViewWidget::new(viewport, 1.0, theme());
        metrics.scroll_y = ty;
        wv.set_scroll_metrics(metrics);
        assert!(
            wv.scroll.scroll_y > 100.0,
            "drag should advance webview scroll offset, got {}",
            wv.scroll.scroll_y
        );
        assert!(
            wv.scroll.scroll_y <= wv.scroll.max_scroll_y(),
            "must clamp to max_scroll"
        );
    }
}
