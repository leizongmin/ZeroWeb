//! ScrollBar — 通用滚动条几何与视觉（spec FR-006 / DC-4）。
//!
//! 页面内容尺寸/scroll offset 由 WebView 管理；本控件只算**外部**滚动条的几何与绘制。
//! 拖动滚动条转为 [`ScrollCommand`]（不直接改业务状态）；hit-test 判定是否在 thumb/track 上。

use zero_ui_core::geometry::{Point, Rect};
use zero_ui_core::scroll::{ScrollCommand, ScrollMetrics};

/// 滚动条朝向。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollOrientation {
    Vertical,
    Horizontal,
}

/// 滚动条几何。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScrollBarGeometry {
    pub track: Rect,
    pub thumb: Rect,
    pub orientation: ScrollOrientation,
}

const THICKNESS: f32 = 10.0;

/// 由视口矩形 + 滚动度量算滚动条几何。
///
/// 返回 `None` 表示内容不超出视口（无需滚动条）。
pub fn layout_scrollbar(
    viewport: Rect,
    metrics: ScrollMetrics,
    orientation: ScrollOrientation,
) -> Option<ScrollBarGeometry> {
    let track = match orientation {
        ScrollOrientation::Vertical => Rect::from_ltrb(
            viewport.right() - THICKNESS,
            viewport.top(),
            viewport.right(),
            viewport.bottom(),
        ),
        ScrollOrientation::Horizontal => Rect::from_ltrb(
            viewport.left(),
            viewport.bottom() - THICKNESS,
            viewport.right(),
            viewport.bottom(),
        ),
    };

    let (content_extent, viewport_extent, scroll, max_scroll) = match orientation {
        ScrollOrientation::Vertical => (
            metrics.content_height,
            metrics.viewport_height,
            metrics.scroll_y,
            metrics.max_scroll_y(),
        ),
        ScrollOrientation::Horizontal => (
            metrics.content_width,
            metrics.viewport_width,
            metrics.scroll_x,
            metrics.max_scroll_x(),
        ),
    };

    if content_extent <= viewport_extent || max_scroll <= 0.0 {
        return None;
    }

    let track_start = match orientation {
        ScrollOrientation::Vertical => track.top(),
        ScrollOrientation::Horizontal => track.left(),
    };
    let track_extent = match orientation {
        ScrollOrientation::Vertical => track.size.height,
        ScrollOrientation::Horizontal => track.size.width,
    };

    let thumb_extent = (track_extent * viewport_extent / content_extent).max(24.0);
    let ratio = scroll / max_scroll;
    let thumb_start = track_start + (track_extent - thumb_extent) * ratio;

    let thumb = match orientation {
        ScrollOrientation::Vertical => {
            Rect::from_ltrb(track.left(), thumb_start, track.right(), thumb_start + thumb_extent)
        }
        ScrollOrientation::Horizontal => {
            Rect::from_ltrb(thumb_start, track.top(), thumb_start + thumb_extent, track.bottom())
        }
    };

    Some(ScrollBarGeometry {
        track,
        thumb,
        orientation,
    })
}

/// 把滚动条上的指针拖动（从 `from` 到 `to`）转成 ScrollCommand。
pub fn drag_to_command(geom: &ScrollBarGeometry, from: Point, to: Point) -> Option<ScrollCommand> {
    // 仅当起始点落在 thumb 上才视为有效拖动。
    if !geom.thumb.contains(from) {
        return None;
    }
    let max_scroll = 1000.0_f32; // 占位；真实值由宿主 metrics 提供（M2 接入）
    match geom.orientation {
        ScrollOrientation::Vertical => {
            let track_extent = geom.track.size.height.max(1.0);
            let dy = to.y - from.y;
            let frac = dy / track_extent;
            Some(ScrollCommand::By {
                dx: 0.0,
                dy: frac * max_scroll,
            })
        }
        ScrollOrientation::Horizontal => {
            let track_extent = geom.track.size.width.max(1.0);
            let dx = to.x - from.x;
            let frac = dx / track_extent;
            Some(ScrollCommand::By {
                dx: frac * max_scroll,
                dy: 0.0,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metrics(scroll_y: f32) -> ScrollMetrics {
        ScrollMetrics {
            content_width: 200.0,
            content_height: 1000.0,
            viewport_width: 200.0,
            viewport_height: 200.0,
            scroll_x: 0.0,
            scroll_y,
        }
    }

    #[test]
    fn vertical_thumb_ratio_and_position() {
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let geom = layout_scrollbar(viewport, metrics(400.0), ScrollOrientation::Vertical).unwrap();
        // viewport/content = 200/1000 = 0.2 → thumb 高度 = track 高 * 0.2。
        let track_h = geom.track.size.height;
        assert!((geom.thumb.size.height - track_h * 0.2).abs() < 0.5);
        // scroll 400 / max 800 = 0.5 → thumb 顶端 = (track_h - thumb_h) * 0.5 = 80。
        assert!((geom.thumb.top() - 80.0).abs() < 0.5);
    }

    #[test]
    fn no_scrollbar_when_content_fits() {
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let small = ScrollMetrics {
            content_height: 100.0,
            viewport_height: 200.0,
            content_width: 200.0,
            viewport_width: 200.0,
            scroll_x: 0.0,
            scroll_y: 0.0,
        };
        assert!(layout_scrollbar(viewport, small, ScrollOrientation::Vertical).is_none());
    }

    #[test]
    fn drag_only_when_start_on_thumb() {
        let viewport = Rect::from_ltrb(0.0, 0.0, 200.0, 200.0);
        let geom = layout_scrollbar(viewport, metrics(400.0), ScrollOrientation::Vertical).unwrap();
        let thumb_center = Point::new(geom.thumb.left(), geom.thumb.top() + geom.thumb.size.height / 2.0);
        // 重新构造一个 thumb 在原位的 geom 用于测试（thumb.top≈ 非零）。
        let on_thumb = Point::new(geom.thumb.left() + 1.0, geom.thumb.top() + 1.0);
        assert!(drag_to_command(&geom, on_thumb, Point::new(on_thumb.x, on_thumb.y + 20.0)).is_some());
        // 起点不在 thumb 上 → None。
        assert!(drag_to_command(&geom, Point::new(0.0, 0.0), thumb_center).is_none());
    }
}
