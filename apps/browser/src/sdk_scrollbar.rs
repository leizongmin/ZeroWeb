//! SDK scrollbar 桥接适配器（DC-4：apps/browser page_scroll.rs → ui/widgets::scrollbar）。
//!
//! 本模块把浏览器的物理像素坐标系的 scrollbar 状态映射到 SDK 的 scrollbar 函数。
//! 仅替换**逻辑层**（hit-test + pointer→scroll 计算）；几何计算与渲染仍走既有路径。
//!
//! 全部函数由 `#[cfg(feature = "sdk-chrome")]` 门控——默认 feature-off 时手绘 scrollbar
//! 路径 bit-identical 不变（DC-14 零退化硬门禁）。

use crate::page_scroll::{PageScrollLayout, ScrollbarGeometry, ScrollbarHit, TabScrollState};
use zero_ui_core::geometry::{Point, Rect};
use zero_ui_core::scroll::ScrollMetrics;
use zero_ui_widgets::scrollbar::{self, ScrollBarGeometry, ScrollBarHit, ScrollOrientation};

// ── coordinate bridge ──────────────────────────────────────────────────────────

/// 把浏览器的物理像素布局 + 滚动偏移转换为 SDK `ScrollMetrics`（物理像素）。
fn to_sdk_metrics(layout: &PageScrollLayout, scroll: TabScrollState) -> ScrollMetrics {
    ScrollMetrics {
        content_width: layout.max_scroll_x + layout.viewport_w,
        content_height: layout.max_scroll_y + layout.viewport_h,
        viewport_width: layout.viewport_w,
        viewport_height: layout.viewport_h,
        scroll_x: scroll.x,
        scroll_y: scroll.y,
    }
}

/// 从浏览器既有 `ScrollbarGeometry` 抽出一个 SDK `ScrollBarGeometry`（垂直）。
///
/// 返回 `None` 表示当前不显示垂直滚动条。
fn as_sdk_vertical(geom: &ScrollbarGeometry) -> Option<ScrollBarGeometry> {
    geom.vertical_track.map(|track| {
        let thumb = geom.vertical_thumb.unwrap_or(track);
        ScrollBarGeometry {
            track: Rect::from_ltrb(track.0, track.1, track.0 + track.2, track.1 + track.3),
            thumb: Rect::from_ltrb(thumb.0, thumb.1, thumb.0 + thumb.2, thumb.1 + thumb.3),
            orientation: ScrollOrientation::Vertical,
        }
    })
}

/// 从浏览器既有 `ScrollbarGeometry` 抽出一个 SDK `ScrollBarGeometry`（水平）。
///
/// 返回 `None` 表示当前不显示水平滚动条。
fn as_sdk_horizontal(geom: &ScrollbarGeometry) -> Option<ScrollBarGeometry> {
    geom.horizontal_track.map(|track| {
        let thumb = geom.horizontal_thumb.unwrap_or(track);
        ScrollBarGeometry {
            track: Rect::from_ltrb(track.0, track.1, track.0 + track.2, track.1 + track.3),
            thumb: Rect::from_ltrb(thumb.0, thumb.1, thumb.0 + thumb.2, thumb.1 + thumb.3),
            orientation: ScrollOrientation::Horizontal,
        }
    })
}

// ── sdk-based hit test ────────────────────────────────────────────────────────

/// 用 SDK `scrollbar::hit_test` 替代原有的手写命中逻辑。
///
/// 优先垂直 → 水平；找到 Thumb 直接返回，否则取首个非 None 结果。
pub(crate) fn sdk_hit_test_scrollbar(px: f32, py: f32, geometry: &ScrollbarGeometry) -> Option<ScrollbarHit> {
    let pt = Point::new(px, py);

    // 垂直
    if let Some(sdk_geom) = as_sdk_vertical(geometry) {
        match scrollbar::hit_test(&sdk_geom, pt) {
            ScrollBarHit::Thumb => return Some(ScrollbarHit::VerticalThumb),
            ScrollBarHit::TrackBefore | ScrollBarHit::TrackAfter => return Some(ScrollbarHit::VerticalTrack),
            ScrollBarHit::None => {}
        }
    }

    // 水平
    if let Some(sdk_geom) = as_sdk_horizontal(geometry) {
        match scrollbar::hit_test(&sdk_geom, pt) {
            ScrollBarHit::Thumb => return Some(ScrollbarHit::HorizontalThumb),
            ScrollBarHit::TrackBefore | ScrollBarHit::TrackAfter => return Some(ScrollbarHit::HorizontalTrack),
            ScrollBarHit::None => {}
        }
    }

    None
}

// ── sdk-based pointer → scroll mapping ────────────────────────────────────────

/// 计算从 `from_y` 到 `to_y` 的拖动对应的内容滚动量（SDK `drag_to_command` 桥接）。
fn scroll_by_drag_y(
    layout: &PageScrollLayout,
    scroll: TabScrollState,
    content_rect: Rect,
    from_y: f32,
    to_y: f32,
) -> f32 {
    let metrics = to_sdk_metrics(layout, scroll);
    if metrics.max_scroll_y() <= 0.0 {
        return 0.0;
    }
    if let Some(sdk_geom) = scrollbar::layout_scrollbar(content_rect, metrics, ScrollOrientation::Vertical) {
        let from = Point::new(sdk_geom.thumb.left() + 1.0, from_y);
        let to = Point::new(sdk_geom.thumb.left() + 1.0, to_y);
        if let Some(cmd) = scrollbar::drag_to_command(&sdk_geom, metrics, from, to) {
            let (_tx, ty) = cmd.resolve_target(metrics);
            return ty;
        }
    }
    scroll.y.clamp(0.0, metrics.max_scroll_y())
}

/// 水平版本。
fn scroll_by_drag_x(
    layout: &PageScrollLayout,
    scroll: TabScrollState,
    content_rect: Rect,
    from_x: f32,
    to_x: f32,
) -> f32 {
    let metrics = to_sdk_metrics(layout, scroll);
    if metrics.max_scroll_x() <= 0.0 {
        return 0.0;
    }
    if let Some(sdk_geom) = scrollbar::layout_scrollbar(content_rect, metrics, ScrollOrientation::Horizontal) {
        let from = Point::new(from_x, sdk_geom.thumb.top() + 1.0);
        let to = Point::new(to_x, sdk_geom.thumb.top() + 1.0);
        if let Some(cmd) = scrollbar::drag_to_command(&sdk_geom, metrics, from, to) {
            let (tx, _ty) = cmd.resolve_target(metrics);
            return tx;
        }
    }
    scroll.x.clamp(0.0, metrics.max_scroll_x())
}

/// 用 SDK `scrollbar::drag_to_command` 替代 `scroll_y_from_pointer`。
pub(crate) fn sdk_scroll_y_from_pointer(
    layout: &PageScrollLayout,
    scroll: TabScrollState,
    content_rect: Rect,
    from_y: f32,
    to_y: f32,
) -> f32 {
    scroll_by_drag_y(layout, scroll, content_rect, from_y, to_y)
}

/// 用 SDK `scrollbar::drag_to_command` 替代 `scroll_x_from_pointer`（水平版本）。
pub(crate) fn sdk_scroll_x_from_pointer(
    layout: &PageScrollLayout,
    scroll: TabScrollState,
    content_rect: Rect,
    from_x: f32,
    to_x: f32,
) -> f32 {
    scroll_by_drag_x(layout, scroll, content_rect, from_x, to_x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page_scroll;

    fn sample_layout() -> PageScrollLayout {
        PageScrollLayout {
            viewport_x: 0.0,
            viewport_y: 100.0,
            viewport_w: 780.0,
            viewport_h: 560.0,
            show_vertical: true,
            show_horizontal: false,
            max_scroll_x: 0.0,
            max_scroll_y: 440.0,
        }
    }

    fn sample_scroll() -> TabScrollState {
        TabScrollState { x: 0.0, y: 200.0 }
    }

    #[test]
    fn sdk_hit_test_matches_handrolled() {
        // 构造浏览器原有的 ScrollbarGeometry（物理像素）。
        let layout = sample_layout();
        let scroll = sample_scroll();
        let geom = page_scroll::scrollbar_geometry(&layout, scroll, 0.0, 100.0, 800.0, 600.0, 1.0);

        // 测试点集：应覆盖 thumb 内、track 内但 thumb 外、track 外。
        let thumb = geom.vertical_thumb.unwrap();
        let track = geom.vertical_track.unwrap();

        // thumb 中心 → 两实现都返回 VerticalThumb。
        let tx = thumb.0 + thumb.2 * 0.5;
        let ty = thumb.1 + thumb.3 * 0.5;
        let old = page_scroll::hit_test_scrollbar(tx, ty, &geom);
        let new = sdk_hit_test_scrollbar(tx, ty, &geom);
        assert_eq!(old, new, "thumb center: old={old:?} new={new:?}");

        // track 空白（thumb 上方）→ 两实现都命中。
        let above_y = track.1 + 5.0; // track 顶部附近，应在 thumb 上方
        let old_above = page_scroll::hit_test_scrollbar(tx, above_y, &geom);
        let new_above = sdk_hit_test_scrollbar(tx, above_y, &geom);
        assert_eq!(old_above, new_above, "above thumb: old={old_above:?} new={new_above:?}");

        // track 外（左侧空白）→ 两实现都返回 None。
        assert_eq!(
            page_scroll::hit_test_scrollbar(5.0, ty, &geom),
            None,
            "old: outside track"
        );
        assert_eq!(sdk_hit_test_scrollbar(5.0, ty, &geom), None, "new: outside track");
    }

    fn content_rect() -> Rect {
        Rect::from_ltrb(0.0, 100.0, 800.0, 700.0)
    }

    #[test]
    fn sdk_scroll_y_from_pointer_produces_valid_range() {
        let layout = sample_layout();
        let scroll = sample_scroll();
        let cr = content_rect();
        // Drag thumb downward → scroll.y should increase (clamped to [0, max_scroll_y]).
        let new_y = sdk_scroll_y_from_pointer(&layout, scroll, cr, 300.0, 350.0);
        assert!(new_y >= 0.0, "new_y={new_y} >= 0");
        assert!(new_y <= layout.max_scroll_y, "new_y={new_y} <= {}", layout.max_scroll_y);

        // Drag upward → should decrease.
        let up_y = sdk_scroll_y_from_pointer(&layout, scroll, cr, 300.0, 250.0);
        assert!(up_y <= new_y, "drag up should decrease: {up_y} <= {new_y}");
        assert!(up_y >= 0.0, "{up_y} >= 0");
    }

    #[test]
    fn sdk_scroll_y_no_overflow_clamps() {
        // Content fits viewport → max_scroll_y=0 → returns 0.
        let flat = PageScrollLayout {
            viewport_x: 0.0,
            viewport_y: 0.0,
            viewport_w: 800.0,
            viewport_h: 600.0,
            show_vertical: false,
            show_horizontal: false,
            max_scroll_x: 0.0,
            max_scroll_y: 0.0,
        };
        let cr = Rect::from_ltrb(0.0, 0.0, 800.0, 600.0);
        let result = sdk_scroll_y_from_pointer(&flat, TabScrollState::default(), cr, 300.0, 500.0);
        assert_eq!(result, 0.0, "no overflow → no scroll");
    }
}
