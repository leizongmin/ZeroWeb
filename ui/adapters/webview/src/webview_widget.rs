//! WebViewWidget 数据模型与后端 trait（spec IF-004 / FR-005）。

use zero_ui_core::geometry::Rect;
use zero_ui_core::scroll::ScrollMetrics;
use zero_ui_core::theme::Theme;

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
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::Size;
    use zero_ui_core::theme::{ColorPalette, ResolvedColorScheme, ThemeId, ThemeResolver};

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
}
