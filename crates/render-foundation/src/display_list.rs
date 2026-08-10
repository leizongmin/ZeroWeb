//! Display list — 录制与光栅化之间的显式契约（#3 渲染线程化 RFC S1）。
//!
//! `draw_order` 位于 [`RenderPrimitives`] 内；本类型持有图元序列与本帧脏区域。

use crate::primitive::{RenderPrimitives, RenderStats};

/// 一帧的 display list：图元序列 + 本帧需重绘的脏区域（视口 CSS 像素，`(x,y,w,h)`）。
#[derive(Debug, Clone)]
pub struct DisplayList {
    /// 本帧绘制图元（含 `draw_order`）。
    pub primitives: RenderPrimitives,
    /// 脏区域列表；空或覆盖全视口时消费方走全量光栅化。
    pub dirty_rects: Vec<(f32, f32, f32, f32)>,
}

impl DisplayList {
    /// 从图元与脏区域构造。
    pub fn new(primitives: RenderPrimitives, dirty_rects: Vec<(f32, f32, f32, f32)>) -> Self {
        Self {
            primitives,
            dirty_rects,
        }
    }

    /// 全视口脏区域（全量重绘契约）。
    pub fn full_viewport(primitives: RenderPrimitives, width: f32, height: f32) -> Self {
        Self {
            primitives,
            dirty_rects: vec![(0.0, 0.0, width, height)],
        }
    }

    /// 脏区域是否覆盖整个视口（含空 = 全量）。
    pub fn is_full_viewport(&self, width: f32, height: f32) -> bool {
        if self.dirty_rects.is_empty() {
            return true;
        }
        let viewport_area = (width * height).max(1.0);
        let dirty_area: f32 = self.dirty_rects.iter().map(|(_, _, w, h)| w * h).sum();
        dirty_area >= viewport_area * 0.99
    }

    /// 与 [`RenderStats`] 同步脏区域（stats 保留计数等其它字段）。
    pub fn apply_stats_dirty_rects(&self, stats: &mut RenderStats) {
        stats.dirty_rects.clone_from(&self.dirty_rects);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitive::RenderPrimitives;

    #[test]
    fn full_viewport_dirty_covers_entire_area() {
        let dl = DisplayList::full_viewport(RenderPrimitives::default(), 800.0, 600.0);
        assert!(dl.is_full_viewport(800.0, 600.0));
    }

    #[test]
    fn partial_dirty_not_full_viewport() {
        let dl = DisplayList::new(RenderPrimitives::default(), vec![(10.0, 20.0, 50.0, 30.0)]);
        assert!(!dl.is_full_viewport(800.0, 600.0));
    }
}
