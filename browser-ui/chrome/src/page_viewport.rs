//! PageViewportFrame — 页面视口框架（spec §8.4.1A）。
//!
//! 组合 `WebViewWidget` + 外层 `ScrollBar`；UI SDK 布局其外部矩形，WebViewWidget 绘制网页，
//! ScrollBar 显示外层滚动反馈。WebView 内容尺寸/scroll offset 由 WebView 管理（spec FR-006）。

use zero_ui_adapter_webview::WebViewLayoutInput;
use zero_ui_core::geometry::Rect;
use zero_ui_core::scroll::ScrollMetrics;
use zero_ui_core::theme::Theme;

/// 页面视口框架（M1 skeleton：持有外部 rect + scale + scroll 度量）。
#[derive(Debug, Clone)]
pub struct PageViewportFrame {
    pub rect: Rect,
    pub scale_factor: f32,
    pub scroll: ScrollMetrics,
}

impl PageViewportFrame {
    pub fn new(rect: Rect, scale_factor: f32) -> PageViewportFrame {
        let scroll = ScrollMetrics {
            content_width: rect.size.width,
            content_height: rect.size.height,
            viewport_width: rect.size.width,
            viewport_height: rect.size.height,
            scroll_x: 0.0,
            scroll_y: 0.0,
        };
        PageViewportFrame {
            rect,
            scale_factor,
            scroll,
        }
    }

    /// 生成传给 WebViewWidget 的 layout 输入（复用 adapter-webview 类型，确立耦合边界）。
    pub fn webview_layout_input(&self, theme: Theme) -> WebViewLayoutInput {
        WebViewLayoutInput {
            rect: self.rect,
            scale_factor: self.scale_factor,
            theme,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::theme::{ColorPalette, ResolvedColorScheme, ThemeId, ThemeResolver};

    #[test]
    fn layout_input_carries_geometry() {
        let frame = PageViewportFrame::new(Rect::from_ltrb(0.0, 0.0, 800.0, 600.0), 2.0);
        let theme = ThemeResolver::build_theme(
            ThemeId::new("zero"),
            "Zero",
            ResolvedColorScheme::Light,
            ColorPalette::default(),
        );
        let input = frame.webview_layout_input(theme);
        assert_eq!(input.rect.size.width, 800.0);
        assert_eq!(input.scale_factor, 2.0);
    }
}
