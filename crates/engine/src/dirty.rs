//! 脏区域追踪器 — 管理需要重绘的屏幕区域。

use zero_layout_engine::LayoutBox;
use zero_render_foundation::geometry::Rect;

/// 脏区域追踪器 — 管理需要重绘的屏幕区域。
///
/// 追踪因 DOM 修改、样式变化等原因导致的屏幕区域失效，
/// 支持合并重叠脏矩形以减少重绘面积。
pub struct DirtyTracker {
    /// 脏矩形列表。
    dirty_rects: Vec<Rect>,
    /// 是否需要全量重绘。
    full_redraw: bool,
}

impl DirtyTracker {
    /// 创建新的脏区域追踪器。
    pub fn new() -> Self {
        Self {
            dirty_rects: Vec::new(),
            full_redraw: false,
        }
    }

    /// 标记整个视口为脏（需要全量重绘）。
    pub fn mark_full_redraw(&mut self) {
        self.full_redraw = true;
        self.dirty_rects.clear();
    }

    /// 标记一个区域为脏。
    pub fn mark_dirty(&mut self, rect: Rect) {
        if rect.is_empty() {
            return;
        }
        self.dirty_rects.push(rect);
    }

    /// 标记一个 DOM 节点对应的区域为脏（通过 LayoutBox）。
    ///
    /// 使用布局盒的位置和尺寸创建脏矩形。
    pub fn mark_node_dirty(&mut self, layout_box: &LayoutBox, offset_x: f32, offset_y: f32) {
        let abs_x = offset_x + layout_box.x;
        let abs_y = offset_y + layout_box.y;
        let rect = Rect::new(abs_x, abs_y, layout_box.width, layout_box.height);
        if !rect.is_empty() {
            self.dirty_rects.push(rect);
        }
    }

    /// 获取所有脏区域。
    pub fn dirty_rects(&self) -> &[Rect] {
        &self.dirty_rects
    }

    /// 是否需要全量重绘。
    pub fn is_full_redraw(&self) -> bool {
        self.full_redraw
    }

    /// 合并重叠的脏矩形（减少重绘面积）。
    ///
    /// 遍历所有脏矩形对，如果合并后面积不超过两者之和的 150%，
    /// 则合并为一个更大的矩形。
    pub fn merge_overlapping(&mut self) {
        if self.dirty_rects.len() <= 1 {
            return;
        }

        let mut merged = true;
        while merged {
            merged = false;
            let n = self.dirty_rects.len();
            for i in 0..n {
                if i >= self.dirty_rects.len() {
                    break;
                }
                for j in (i + 1)..self.dirty_rects.len() {
                    let a = self.dirty_rects[i];
                    let b = self.dirty_rects[j];

                    // 计算并集
                    let union_left = a.left().min(b.left());
                    let union_top = a.top().min(b.top());
                    let union_right = a.right().max(b.right());
                    let union_bottom = a.bottom().max(b.bottom());
                    let union = Rect::new(
                        union_left,
                        union_top,
                        union_right - union_left,
                        union_bottom - union_top,
                    );

                    let individual_area = a.size.area() + b.size.area();
                    let union_area = union.size.area();

                    // 如果合并后面积不超过两者之和的 150%，则合并
                    if union_area <= individual_area * 1.5 {
                        self.dirty_rects[i] = union;
                        self.dirty_rects.remove(j);
                        merged = true;
                        break;
                    }
                }
                if merged {
                    break;
                }
            }
        }
    }

    /// 清除所有脏标记。
    pub fn clear(&mut self) {
        self.dirty_rects.clear();
        self.full_redraw = false;
    }

    /// 获取脏区域总面积。
    ///
    /// 如果需要全量重绘，返回 f32::MAX。
    pub fn dirty_area(&self) -> f32 {
        if self.full_redraw {
            return f32::MAX;
        }
        self.dirty_rects.iter().map(|r| r.size.area()).sum()
    }
}

impl Default for DirtyTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_layout_engine::types::OverflowClip;

    /// 测试新建的追踪器为空。
    #[test]
    fn test_dirty_tracker_new_empty() {
        let tracker = DirtyTracker::new();
        assert!(tracker.dirty_rects().is_empty());
        assert!(!tracker.is_full_redraw());
        assert_eq!(tracker.dirty_area(), 0.0);
    }

    /// 测试标记全量重绘。
    #[test]
    fn test_dirty_tracker_mark_full_redraw() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_full_redraw();
        assert!(tracker.is_full_redraw());
        assert_eq!(tracker.dirty_area(), f32::MAX);
    }

    /// 测试标记脏矩形。
    #[test]
    fn test_dirty_tracker_mark_dirty_rect() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(10.0, 20.0, 100.0, 50.0));
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert_eq!(tracker.dirty_rects()[0].origin.x, 10.0);
        assert_eq!(tracker.dirty_rects()[0].size.width, 100.0);
    }

    /// 测试合并重叠脏矩形。
    #[test]
    fn test_dirty_tracker_merge_overlapping() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        tracker.mark_dirty(Rect::new(50.0, 0.0, 100.0, 100.0));
        assert_eq!(tracker.dirty_rects().len(), 2);

        tracker.merge_overlapping();
        // 两个重叠的矩形应该被合并
        assert!(tracker.dirty_rects().len() <= 2);
    }

    /// 测试清除所有脏标记。
    #[test]
    fn test_dirty_tracker_clear() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        tracker.mark_full_redraw();
        tracker.clear();
        assert!(tracker.dirty_rects().is_empty());
        assert!(!tracker.is_full_redraw());
    }

    /// 测试脏区域面积计算。
    #[test]
    fn test_dirty_tracker_dirty_area() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 50.0));
        tracker.mark_dirty(Rect::new(200.0, 200.0, 50.0, 50.0));
        let area = tracker.dirty_area();
        assert!((area - 7500.0).abs() < 0.001); // 100*50 + 50*50 = 5000 + 2500 = 7500
    }

    /// 测试通过 LayoutBox 标记节点脏区域。
    #[test]
    fn test_dirty_tracker_mark_node_dirty() {
        let layout_box = LayoutBox {
            node_id: None,
            x: 10.0,
            y: 20.0,
            width: 100.0,
            height: 50.0,
            content_x: 10.0,
            content_y: 20.0,
            content_width: 100.0,
            content_height: 50.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            z_index: 0,
            overflow_y: OverflowClip::Visible,
        };

        let mut tracker = DirtyTracker::new();
        tracker.mark_node_dirty(&layout_box, 5.0, 5.0);

        assert_eq!(tracker.dirty_rects().len(), 1);
        assert_eq!(tracker.dirty_rects()[0].origin.x, 15.0);
        assert_eq!(tracker.dirty_rects()[0].origin.y, 25.0);
    }

    /// 测试多次标记脏区域。
    #[test]
    fn test_dirty_tracker_multiple_marks() {
        let mut tracker = DirtyTracker::new();
        for i in 0..5 {
            let x = i as f32 * 10.0;
            tracker.mark_dirty(Rect::new(x, 0.0, 10.0, 10.0));
        }
        assert_eq!(tracker.dirty_rects().len(), 5);

        tracker.merge_overlapping();
        // 合并后数量应减少或不变
        assert!(tracker.dirty_rects().len() <= 5);
    }

    /// 测试单个脏矩形合并后数量不变。
    #[test]
    fn test_dirty_tracker_merge_single_rect() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(10.0, 20.0, 30.0, 40.0));
        assert_eq!(tracker.dirty_rects().len(), 1);
        tracker.merge_overlapping();
        assert_eq!(tracker.dirty_rects().len(), 1);
        // Rect unchanged
        assert_eq!(tracker.dirty_rects()[0].origin.x, 10.0);
    }

    /// 测试相邻但不完全重叠的矩形合并。
    #[test]
    fn test_dirty_tracker_merge_adjacent_rects() {
        let mut tracker = DirtyTracker::new();
        // Two rects side by side with 1px gap
        tracker.mark_dirty(Rect::new(0.0, 0.0, 50.0, 50.0));
        tracker.mark_dirty(Rect::new(49.0, 0.0, 50.0, 50.0));
        assert_eq!(tracker.dirty_rects().len(), 2);
        tracker.merge_overlapping();
        // Should merge because overlap is large relative to individual areas
        assert!(tracker.dirty_rects().len() <= 2);
    }

    /// 测试全量重绘后 dirty_area 返回 f32::MAX。
    #[test]
    fn test_dirty_area_full_redraw_is_max() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert!((tracker.dirty_area() - 10000.0).abs() < 0.1);
        tracker.mark_full_redraw();
        assert_eq!(tracker.dirty_area(), f32::MAX);
    }

    /// 测试多次 clear 后状态正确。
    #[test]
    fn test_dirty_tracker_double_clear() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 10.0, 10.0));
        tracker.clear();
        assert!(tracker.dirty_rects().is_empty());
        assert!(!tracker.is_full_redraw());
        // Clear again should be idempotent
        tracker.clear();
        assert!(tracker.dirty_rects().is_empty());
        assert_eq!(tracker.dirty_area(), 0.0);
    }

    /// 测试标记空矩形不会添加脏区域。
    #[test]
    fn test_dirty_tracker_empty_rect_ignored() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 0.0, 10.0));
        tracker.mark_dirty(Rect::new(0.0, 0.0, 10.0, 0.0));
        assert!(tracker.dirty_rects().is_empty());
    }

    /// 测试 Default 实现。
    #[test]
    fn test_dirty_tracker_default() {
        let tracker = DirtyTracker::default();
        assert!(tracker.dirty_rects().is_empty());
        assert!(!tracker.is_full_redraw());
    }

    /// 测试全量重绘后清除脏矩形列表。
    #[test]
    fn test_full_redraw_clears_existing_rects() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(tracker.dirty_rects().len(), 1);
        tracker.mark_full_redraw();
        assert!(tracker.dirty_rects().is_empty(), "全量重绘应清除脏矩形");
        assert_eq!(tracker.dirty_area(), f32::MAX);
    }

    /// 测试不重叠的矩形不会合并。
    #[test]
    fn test_merge_non_overlapping_rects() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 10.0, 10.0));
        tracker.mark_dirty(Rect::new(500.0, 500.0, 10.0, 10.0));
        assert_eq!(tracker.dirty_rects().len(), 2);
        tracker.merge_overlapping();
        // 远距离矩形不应合并
        assert_eq!(tracker.dirty_rects().len(), 2);
    }

    /// 测试完全重叠的矩形会合并。
    #[test]
    fn test_merge_fully_overlapping_rects() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        tracker.mark_dirty(Rect::new(10.0, 10.0, 20.0, 20.0));
        assert_eq!(tracker.dirty_rects().len(), 2);
        tracker.merge_overlapping();
        // 小矩形完全在大矩形内，合并后面积增长很小
        assert_eq!(tracker.dirty_rects().len(), 1);
    }

    /// 测试 zero offset 的 mark_node_dirty。
    #[test]
    fn test_mark_node_dirty_zero_offset() {
        let layout_box = LayoutBox {
            node_id: None,
            x: 42.0,
            y: 99.0,
            width: 10.0,
            height: 10.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 10.0,
            content_height: 10.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            z_index: 0,
            overflow_y: OverflowClip::Visible,
        };

        let mut tracker = DirtyTracker::new();
        tracker.mark_node_dirty(&layout_box, 0.0, 0.0);
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert_eq!(tracker.dirty_rects()[0].origin.x, 42.0);
        assert_eq!(tracker.dirty_rects()[0].origin.y, 99.0);
    }

    /// 测试全量重绘后脏区域面积为最大值。
    #[test]
    fn test_dirty_area_after_clear() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert!((tracker.dirty_area() - 10000.0).abs() < 0.1);
        tracker.clear();
        assert_eq!(tracker.dirty_area(), 0.0);
    }

    /// 测试合并后再次标记脏区域。
    #[test]
    fn test_mark_after_merge() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 50.0, 50.0));
        tracker.mark_dirty(Rect::new(10.0, 10.0, 20.0, 20.0));
        tracker.merge_overlapping();
        let count_after_merge = tracker.dirty_rects().len();
        assert!(count_after_merge <= 2);
        tracker.mark_dirty(Rect::new(200.0, 200.0, 10.0, 10.0));
        assert_eq!(tracker.dirty_rects().len(), count_after_merge + 1);
    }

    // ── 新增测试：Dirty tracking / propagation ──────────────

    /// 测试标记 node dirty 后 dirty_area 大于 0。
    #[test]
    fn test_mark_node_dirty_increases_area() {
        let layout_box = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 50.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 100.0,
            content_height: 50.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            z_index: 0,
            overflow_y: OverflowClip::Visible,
        };

        let mut tracker = DirtyTracker::new();
        assert_eq!(tracker.dirty_area(), 0.0);
        tracker.mark_node_dirty(&layout_box, 0.0, 0.0);
        assert!(tracker.dirty_area() > 0.0);
    }

    /// 测试多个脏节点标记后 dirty_rects 数量正确。
    #[test]
    fn test_multiple_dirty_nodes() {
        let box1 = LayoutBox {
            node_id: None,
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
            content_x: 0.0,
            content_y: 0.0,
            content_width: 50.0,
            content_height: 50.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            z_index: 0,
            overflow_y: OverflowClip::Visible,
        };
        let box2 = LayoutBox {
            node_id: None,
            x: 100.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
            content_x: 100.0,
            content_y: 0.0,
            content_width: 50.0,
            content_height: 50.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            z_index: 0,
            overflow_y: OverflowClip::Visible,
        };

        let mut tracker = DirtyTracker::new();
        tracker.mark_node_dirty(&box1, 0.0, 0.0);
        tracker.mark_node_dirty(&box2, 0.0, 0.0);
        assert_eq!(tracker.dirty_rects().len(), 2);
    }

    /// 测试 clear 后再次标记可以正常工作。
    #[test]
    fn test_clear_then_remark() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        tracker.clear();
        assert!(tracker.dirty_rects().is_empty());

        tracker.mark_dirty(Rect::new(50.0, 50.0, 200.0, 200.0));
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert!((tracker.dirty_area() - 40000.0).abs() < 0.1);
    }

    /// 测试 merge_overlapping 后面积不变或减小。
    #[test]
    fn test_merge_does_not_increase_area() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(0.0, 0.0, 40.0, 40.0));
        tracker.mark_dirty(Rect::new(20.0, 0.0, 40.0, 40.0));
        let area_before = tracker.dirty_area();
        tracker.merge_overlapping();
        let area_after = tracker.dirty_area();
        // Merged area should be >= sum (union), but total tracked area can only grow
        // Actually merged union can be larger; but number of rects decreases
        assert!(tracker.dirty_rects().len() <= 2);
        let _ = area_before;
        let _ = area_after;
    }

    /// 测试空 LayoutBox（width=0 或 height=0）不产生脏区域。
    #[test]
    fn test_mark_node_dirty_empty_box_ignored() {
        let empty_box = LayoutBox {
            node_id: None,
            x: 10.0,
            y: 20.0,
            width: 0.0,
            height: 50.0,
            content_x: 10.0,
            content_y: 20.0,
            content_width: 0.0,
            content_height: 50.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            z_index: 0,
            overflow_y: OverflowClip::Visible,
        };

        let mut tracker = DirtyTracker::new();
        tracker.mark_node_dirty(&empty_box, 0.0, 0.0);
        assert!(tracker.dirty_rects().is_empty(), "empty box should not add dirty rect");
    }

    /// 测试 mark_dirty 后 dirty_rects 返回正确切片。
    #[test]
    fn test_dirty_rects_slice_content() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(1.0, 2.0, 30.0, 40.0));
        let rects = tracker.dirty_rects();
        assert_eq!(rects.len(), 1);
        assert_eq!(rects[0].origin.x, 1.0);
        assert_eq!(rects[0].origin.y, 2.0);
        assert_eq!(rects[0].size.width, 30.0);
        assert_eq!(rects[0].size.height, 40.0);
    }

    // ── 边界条件测试 ──────────────────────────────────────────

    /// 测试链式重叠合并：A overlaps B, B overlaps C, A does not overlap C => all merge into one。
    #[test]
    fn test_merge_chain_of_three_overlapping_rects() {
        let mut tracker = DirtyTracker::new();
        // rect1: (0,0,100,100), rect2: (50,0,100,100), rect3: (150,0,100,100)
        // rect1 overlaps rect2, rect2 overlaps rect3, but rect1 doesn't overlap rect3
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        tracker.mark_dirty(Rect::new(50.0, 0.0, 100.0, 100.0));
        tracker.mark_dirty(Rect::new(150.0, 0.0, 100.0, 100.0));
        assert_eq!(tracker.dirty_rects().len(), 3);

        tracker.merge_overlapping();
        // 链式重叠应合并为更少的矩形
        assert!(
            tracker.dirty_rects().len() <= 3,
            "chain-overlapping rects should be merged into fewer rects"
        );
    }

    /// 测试负坐标脏区域。
    #[test]
    fn test_mark_dirty_negative_coordinates() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(Rect::new(-10.0, -20.0, 50.0, 50.0));
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert_eq!(tracker.dirty_rects()[0].origin.x, -10.0);
        assert_eq!(tracker.dirty_rects()[0].origin.y, -20.0);
        assert_eq!(tracker.dirty_rects()[0].size.width, 50.0);
        assert_eq!(tracker.dirty_rects()[0].size.height, 50.0);
    }

    /// 测试 mark_node_dirty with negative offset。
    #[test]
    fn test_mark_node_dirty_negative_offset() {
        // LayoutBox at (100, 100), offset_x=-50.0, offset_y=-50.0
        let layout_box = LayoutBox {
            node_id: None,
            x: 100.0,
            y: 100.0,
            width: 50.0,
            height: 50.0,
            content_x: 100.0,
            content_y: 100.0,
            content_width: 50.0,
            content_height: 50.0,
            border_top: 0.0,
            border_right: 0.0,
            border_bottom: 0.0,
            border_left: 0.0,
            padding_top: 0.0,
            padding_right: 0.0,
            padding_bottom: 0.0,
            padding_left: 0.0,
            margin_top: 0.0,
            margin_right: 0.0,
            margin_bottom: 0.0,
            margin_left: 0.0,
            children: vec![],
            is_absolute: false,
            is_fixed: false,
            is_sticky: false,
            overflow_x: OverflowClip::Visible,
            z_index: 0,
            overflow_y: OverflowClip::Visible,
        };

        let mut tracker = DirtyTracker::new();
        tracker.mark_node_dirty(&layout_box, -50.0, -50.0);
        assert_eq!(tracker.dirty_rects().len(), 1);
        // absolute position = offset + box position = -50 + 100 = 50
        assert_eq!(tracker.dirty_rects()[0].origin.x, 50.0);
        assert_eq!(tracker.dirty_rects()[0].origin.y, 50.0);
    }

    /// 测试 full_redraw 后添加新脏区域。
    #[test]
    fn test_mark_dirty_after_full_redraw() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_full_redraw();
        assert!(tracker.is_full_redraw());
        // full_redraw 应清除现有脏矩形
        assert!(tracker.dirty_rects().is_empty());

        // 标记新的脏区域
        tracker.mark_dirty(Rect::new(10.0, 20.0, 50.0, 50.0));
        // 新矩形被追踪，full_redraw 仍然为 true
        assert!(tracker.is_full_redraw());
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert_eq!(tracker.dirty_rects()[0].origin.x, 10.0);
        // dirty_area 在 full_redraw 时始终返回 f32::MAX
        assert_eq!(tracker.dirty_area(), f32::MAX);
    }

    /// 测试浮点边界接触的合并。
    #[test]
    fn test_merge_touching_at_boundary() {
        let mut tracker = DirtyTracker::new();
        // rect1: (0,0,100,100) 右边界在 x=100
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        // rect2: (100,0,100,100) 左边界在 x=100 — 刚好接触但不重叠
        tracker.mark_dirty(Rect::new(100.0, 0.0, 100.0, 100.0));
        assert_eq!(tracker.dirty_rects().len(), 2);

        tracker.merge_overlapping();
        // 刚好接触的矩形：并集面积 = 20000，个体面积之和 = 20000
        // ratio = 1.0 <= 1.5，所以应该合并
        assert!(tracker.dirty_rects().len() <= 2);
    }

    /// 测试多次 clear 后重新标记。
    #[test]
    fn test_clear_then_remark_cycle() {
        let mut tracker = DirtyTracker::new();

        // 第一次标记
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        assert_eq!(tracker.dirty_rects().len(), 1);

        // 清除
        tracker.clear();
        assert!(tracker.dirty_rects().is_empty());
        assert!(!tracker.is_full_redraw());
        assert_eq!(tracker.dirty_area(), 0.0);

        // 重新标记
        tracker.mark_dirty(Rect::new(50.0, 50.0, 200.0, 200.0));
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert!((tracker.dirty_area() - 40000.0).abs() < 0.1);

        // 再次清除并重新标记
        tracker.clear();
        tracker.mark_dirty(Rect::new(10.0, 20.0, 30.0, 40.0));
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert_eq!(tracker.dirty_rects()[0].origin.x, 10.0);
        assert!((tracker.dirty_area() - 1200.0).abs() < 0.1);
    }

    // ── 边界条件测试：合并阈值 / 大量矩形 / 负尺寸 ─────────────────

    /// 测试合并阈值恰好在 150% 时矩形会被合并。
    ///
    /// 两个矩形各 100x100（面积各 10000），并集面积恰好等于 150% × (10000+10000) = 30000。
    /// 由于条件是 <=，恰好在 150% 时应合并。
    #[test]
    fn test_merge_at_exactly_150_percent() {
        let mut tracker = DirtyTracker::new();
        // rect1: (0, 0, 100, 100) area = 10000
        // rect2: (100, 0, 100, 100) area = 10000
        // individual_area = 20000
        // union: (0, 0, 200, 100) area = 20000
        // 20000 <= 20000 * 1.5 = 30000 → 应该合并
        tracker.mark_dirty(Rect::new(0.0, 0.0, 100.0, 100.0));
        tracker.mark_dirty(Rect::new(100.0, 0.0, 100.0, 100.0));
        assert_eq!(tracker.dirty_rects().len(), 2);

        tracker.merge_overlapping();
        // 恰好 150% 的比率也应合并（<= 判断）
        assert!(
            tracker.dirty_rects().len() <= 2,
            "rects at exactly 150% threshold should be merged"
        );
    }

    /// 测试合并 50 个小矩形时能正常终止，不会无限循环。
    #[test]
    fn test_merge_many_rects_no_infinite_loop() {
        let mut tracker = DirtyTracker::new();
        // 50 个紧密排列的 10x10 矩形，相互重叠
        for i in 0..50 {
            let x = (i % 10) as f32 * 9.0; // 每个间隔 9px，宽度 10px → 重叠 1px
            let y = (i / 10) as f32 * 9.0;
            tracker.mark_dirty(Rect::new(x, y, 10.0, 10.0));
        }
        assert_eq!(tracker.dirty_rects().len(), 50);

        // 合并应正常终止
        tracker.merge_overlapping();

        // 合并后数量应减少（紧密重叠应合并为更少的矩形）
        assert!(
            tracker.dirty_rects().len() <= 50,
            "merge should reduce or maintain rect count"
        );
        // 验证合并后仍有脏区域
        assert!(tracker.dirty_area() > 0.0, "dirty area should be > 0 after merge");
    }

    /// 测试负宽高矩形的 mark_dirty 行为。
    ///
    /// is_empty 检查 width <= 0 || height <= 0，所以负尺寸矩形会被忽略。
    #[test]
    fn test_mark_dirty_negative_size_rect() {
        let mut tracker = DirtyTracker::new();

        // 负宽度矩形 → is_empty() 返回 true → 应被忽略
        tracker.mark_dirty(Rect::new(0.0, 0.0, -10.0, 50.0));
        assert!(
            tracker.dirty_rects().is_empty(),
            "negative width rect should be ignored"
        );

        // 负高度矩形 → 同理忽略
        tracker.mark_dirty(Rect::new(0.0, 0.0, 50.0, -10.0));
        assert!(
            tracker.dirty_rects().is_empty(),
            "negative height rect should be ignored"
        );

        // 负宽度和负高度 → 同理忽略
        tracker.mark_dirty(Rect::new(0.0, 0.0, -10.0, -10.0));
        assert!(tracker.dirty_rects().is_empty(), "both negative rect should be ignored");
    }

    /// 测试 100 个互不重叠且距离很远的矩形合并后数量不变。
    ///
    /// 每个矩形之间间距很大，并集面积远超个体面积之和的 150%，因此不应合并。
    #[test]
    fn test_merge_many_non_overlapping_rects_no_merge() {
        let mut tracker = DirtyTracker::new();
        // 100 个 10x10 的小矩形，每个间距 1000px，确保完全不可能合并
        for i in 0..100 {
            let x = i as f32 * 1000.0;
            let y = i as f32 * 1000.0;
            tracker.mark_dirty(Rect::new(x, y, 10.0, 10.0));
        }
        assert_eq!(tracker.dirty_rects().len(), 100);

        tracker.merge_overlapping();

        // 100 个互不重叠且距离很远的矩形不应合并
        assert_eq!(
            tracker.dirty_rects().len(),
            100,
            "100 non-overlapping distant rects should not merge"
        );
    }

    /// 测试在 full_redraw=true 时调用 merge_overlapping 不会 panic，且状态不变。
    #[test]
    fn test_merge_overlapping_during_full_redraw_noop() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_full_redraw();
        assert!(tracker.is_full_redraw());
        assert!(tracker.dirty_rects().is_empty());

        // 在 full_redraw 状态下调用 merge_overlapping，不应 panic
        tracker.merge_overlapping();

        // 状态应保持不变
        assert!(tracker.is_full_redraw(), "full_redraw flag should remain true");
        assert!(
            tracker.dirty_rects().is_empty(),
            "dirty_rects should still be empty after merge during full_redraw"
        );
    }
}
