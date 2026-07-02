//! 滚动语义（spec FR-006 / IF-004）。
//!
//! 页面内容尺寸/scroll offset 由 WebView 管理；通用 ScrollBar 几何由 `ui/widgets` 提供。
//! ScrollBar 拖动发出 [`ScrollCommand`]，不直接改业务状态（spec FR-006 / DC-4）。
//! ScrollCommand/ScrollMetrics 同时被 `ui/widgets::ScrollBar` 与 `ui/adapters/webview` 消费，
//! 故置于浏览器无关的 core 层。

use serde::{Deserialize, Serialize};

/// 滚动度量（spec IF-004 `ScrollMetrics`）：WebView 上报的内容/视口/偏移。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScrollMetrics {
    pub content_width: f32,
    pub content_height: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl ScrollMetrics {
    pub fn max_scroll_x(self) -> f32 {
        (self.content_width - self.viewport_width).max(0.0)
    }
    pub fn max_scroll_y(self) -> f32 {
        (self.content_height - self.viewport_height).max(0.0)
    }
}

/// 滚动命令（spec IF-004 `ScrollCommand`）：ScrollBar/手势 → 宿主 → WebView。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ScrollCommand {
    /// 相对增量。
    By { dx: f32, dy: f32 },
    /// 绝对定位。
    To { x: f32, y: f32 },
    /// 按页翻动（pages 可为小数/负数）。
    Page { pages_x: f32, pages_y: f32 },
}

impl ScrollCommand {
    /// 把命令折叠成最终 scroll 目标（相对当前 metrics）。
    pub fn resolve_target(self, current: ScrollMetrics) -> (f32, f32) {
        match self {
            ScrollCommand::By { dx, dy } => (current.scroll_x + dx, current.scroll_y + dy),
            ScrollCommand::To { x, y } => (x, y),
            ScrollCommand::Page { pages_x, pages_y } => (
                current.scroll_x + pages_x * current.viewport_width,
                current.scroll_y + pages_y * current.viewport_height,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics() -> ScrollMetrics {
        ScrollMetrics {
            content_width: 1000.0,
            content_height: 2000.0,
            viewport_width: 200.0,
            viewport_height: 400.0,
            scroll_x: 50.0,
            scroll_y: 100.0,
        }
    }

    #[test]
    fn max_scroll_clamps_negative() {
        let m = ScrollMetrics {
            content_width: 100.0,
            content_height: 100.0,
            viewport_width: 200.0,
            viewport_height: 200.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
        };
        // 内容小于视口 → 无可滚动空间。
        assert_eq!(m.max_scroll_x(), 0.0);
        assert_eq!(m.max_scroll_y(), 0.0);
    }

    #[test]
    fn command_resolve_targets() {
        let m = metrics();
        assert_eq!(
            ScrollCommand::By { dx: 10.0, dy: 20.0 }.resolve_target(m),
            (60.0, 120.0)
        );
        assert_eq!(ScrollCommand::To { x: 0.0, y: 0.0 }.resolve_target(m), (0.0, 0.0));
        assert_eq!(
            ScrollCommand::Page {
                pages_x: 1.0,
                pages_y: -1.0
            }
            .resolve_target(m),
            (250.0, -300.0)
        );
    }
}
