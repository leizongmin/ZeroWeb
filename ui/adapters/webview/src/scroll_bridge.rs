//! Scroll bridge — 把 ScrollBar/手势发出的 ScrollCommand 折算为受 scroll metrics 约束的目标 offset。
//!
//! 页面内容尺寸/scroll offset 由 WebView 管理（spec FR-006）；本桥接确保命令落在 [0, max_scroll] 内。

use zero_ui_core::scroll::{ScrollCommand, ScrollMetrics};

/// 解算 ScrollCommand → 钳制后的 (scroll_x, scroll_y) 目标。
pub fn apply_scroll_command(metrics: ScrollMetrics, cmd: ScrollCommand) -> (f32, f32) {
    let (x, y) = cmd.resolve_target(metrics);
    (
        x.clamp(0.0, metrics.max_scroll_x()),
        y.clamp(0.0, metrics.max_scroll_y()),
    )
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
            scroll_x: 100.0,
            scroll_y: 200.0,
        }
    }

    #[test]
    fn clamps_to_max_scroll() {
        let m = metrics();
        // 超出 max_scroll（x: 800, y: 1600）。
        let (x, y) = apply_scroll_command(
            m,
            ScrollCommand::To {
                x: 10_000.0,
                y: 10_000.0,
            },
        );
        assert_eq!(x, m.max_scroll_x());
        assert_eq!(y, m.max_scroll_y());
    }

    #[test]
    fn clamps_negative_to_zero() {
        let m = metrics();
        let (x, y) = apply_scroll_command(m, ScrollCommand::To { x: -50.0, y: -50.0 });
        assert_eq!((x, y), (0.0, 0.0));
    }
}
