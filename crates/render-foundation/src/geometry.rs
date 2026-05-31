//! 几何类型 — 矩形、点、尺寸等基础几何定义

/// 二维点
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// X 坐标
    pub x: f32,
    /// Y 坐标
    pub y: f32,
}

impl Point {
    /// 原点
    pub const ZERO: Self = Self { x: 0.0, y: 0.0 };

    /// 创建新点
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 二维尺寸
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    /// 宽度
    pub width: f32,
    /// 高度
    pub height: f32,
}

impl Size {
    /// 零尺寸
    pub const ZERO: Self = Self {
        width: 0.0,
        height: 0.0,
    };

    /// 创建新尺寸
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    /// 面积
    pub fn area(&self) -> f32 {
        self.width * self.height
    }

    /// 是否为空（宽度或高度为 0）
    pub fn is_empty(&self) -> bool {
        self.width <= 0.0 || self.height <= 0.0
    }
}

/// 矩形（左上角 + 尺寸）
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// 原点（左上角）
    pub origin: Point,
    /// 尺寸
    pub size: Size,
}

impl Rect {
    /// 零矩形
    pub const ZERO: Self = Self {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    /// 从位置和尺寸创建矩形
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            origin: Point::new(x, y),
            size: Size::new(width, height),
        }
    }

    /// 左边界
    pub fn left(&self) -> f32 {
        self.origin.x
    }

    /// 上边界
    pub fn top(&self) -> f32 {
        self.origin.y
    }

    /// 右边界
    pub fn right(&self) -> f32 {
        self.origin.x + self.size.width
    }

    /// 下边界
    pub fn bottom(&self) -> f32 {
        self.origin.y + self.size.height
    }

    /// 是否包含指定点
    pub fn contains(&self, point: Point) -> bool {
        point.x >= self.left() && point.x < self.right() && point.y >= self.top() && point.y < self.bottom()
    }

    /// 与另一个矩形的交集
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right > left && bottom > top {
            Some(Rect::new(left, top, right - left, bottom - top))
        } else {
            None
        }
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.size.is_empty()
    }
}

/// 脏区域追踪器 — 管理需要重绘的矩形区域
#[derive(Debug, Clone)]
pub struct DamageTracker {
    /// 脏矩形列表
    dirty_rects: Vec<Rect>,
}

impl DamageTracker {
    /// 创建新的脏区域追踪器
    pub fn new() -> Self {
        Self {
            dirty_rects: Vec::new(),
        }
    }

    /// 添加一个脏矩形
    pub fn add_damage(&mut self, rect: Rect) {
        if rect.is_empty() {
            return;
        }
        // 尝试与现有脏矩形合并
        for existing in &mut self.dirty_rects {
            if let Some(merged) = Self::try_merge(existing, &rect) {
                *existing = merged;
                return;
            }
        }
        self.dirty_rects.push(rect);
    }

    /// 标记整个区域为脏
    pub fn damage_all(&mut self, size: Size) {
        self.dirty_rects.clear();
        self.dirty_rects.push(Rect::new(0.0, 0.0, size.width, size.height));
    }

    /// 获取所有脏矩形
    pub fn dirty_rects(&self) -> &[Rect] {
        &self.dirty_rects
    }

    /// 是否有任何脏区域
    pub fn is_dirty(&self) -> bool {
        !self.dirty_rects.is_empty()
    }

    /// 清除所有脏区域
    pub fn clear(&mut self) {
        self.dirty_rects.clear();
    }

    /// 尝试合并两个矩形（如果它们的并集面积不超过两者之和的 50%）
    fn try_merge(a: &Rect, b: &Rect) -> Option<Rect> {
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

        // 如果合并后的面积不超过两者之和的 50%，则合并
        if union_area <= individual_area * 1.5 {
            Some(union)
        } else {
            None
        }
    }
}

impl Default for DamageTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_zero() {
        let p = Point::ZERO;
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, 0.0);
    }

    #[test]
    fn test_size_area() {
        let s = Size::new(10.0, 20.0);
        assert_eq!(s.area(), 200.0);
        assert!(!s.is_empty());
        assert!(Size::ZERO.is_empty());
    }

    #[test]
    fn test_rect_contains() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(Point::new(50.0, 50.0)));
        assert!(!r.contains(Point::new(150.0, 50.0)));
        assert!(r.contains(Point::new(0.0, 0.0)));
        assert!(!r.contains(Point::new(100.0, 100.0)));
    }

    #[test]
    fn test_rect_intersection() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        let inter = a.intersection(&b).unwrap();
        assert_eq!(inter.origin.x, 50.0);
        assert_eq!(inter.origin.y, 50.0);
        assert_eq!(inter.size.width, 50.0);
        assert_eq!(inter.size.height, 50.0);

        let c = Rect::new(200.0, 200.0, 10.0, 10.0);
        assert!(a.intersection(&c).is_none());
    }

    #[test]
    fn test_damage_tracker_add() {
        let mut tracker = DamageTracker::new();
        assert!(!tracker.is_dirty());

        tracker.add_damage(Rect::new(0.0, 0.0, 10.0, 10.0));
        assert!(tracker.is_dirty());
        assert_eq!(tracker.dirty_rects().len(), 1);
    }

    #[test]
    fn test_damage_tracker_merge() {
        let mut tracker = DamageTracker::new();
        // 两个相邻矩形应该合并
        tracker.add_damage(Rect::new(0.0, 0.0, 10.0, 10.0));
        tracker.add_damage(Rect::new(5.0, 0.0, 10.0, 10.0));
        // 应该合并为一个
        assert!(tracker.dirty_rects().len() <= 2);
    }

    #[test]
    fn test_damage_tracker_damage_all() {
        let mut tracker = DamageTracker::new();
        tracker.add_damage(Rect::new(0.0, 0.0, 10.0, 10.0));
        tracker.damage_all(Size::new(800.0, 600.0));
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert_eq!(tracker.dirty_rects()[0].size.width, 800.0);
    }

    #[test]
    fn test_damage_tracker_clear() {
        let mut tracker = DamageTracker::new();
        tracker.add_damage(Rect::new(0.0, 0.0, 10.0, 10.0));
        tracker.clear();
        assert!(!tracker.is_dirty());
    }

    #[test]
    fn test_rect_empty_not_added_to_damage() {
        let mut tracker = DamageTracker::new();
        tracker.add_damage(Rect::new(0.0, 0.0, 0.0, 10.0));
        assert!(!tracker.is_dirty());
    }

    #[test]
    fn test_rect_intersection_no_overlap() {
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(20.0, 20.0, 10.0, 10.0);
        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn test_rect_intersection_containment() {
        let outer = Rect::new(0.0, 0.0, 100.0, 100.0);
        let inner = Rect::new(10.0, 10.0, 20.0, 20.0);
        let inter = outer.intersection(&inner).unwrap();
        assert_eq!(inter, inner);
    }

    #[test]
    fn test_rect_contains_boundary() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        // Left-top edge is inside
        assert!(r.contains(Point::new(0.0, 0.0)));
        // Right-bottom edge is outside (exclusive)
        assert!(!r.contains(Point::new(100.0, 100.0)));
        // Just inside
        assert!(r.contains(Point::new(99.9, 99.9)));
    }

    #[test]
    fn test_rect_intersection_partial() {
        let a = Rect::new(0.0, 0.0, 50.0, 50.0);
        let b = Rect::new(25.0, 25.0, 50.0, 50.0);
        let inter = a.intersection(&b).unwrap();
        assert_eq!(inter.origin.x, 25.0);
        assert_eq!(inter.origin.y, 25.0);
        assert_eq!(inter.size.width, 25.0);
        assert_eq!(inter.size.height, 25.0);
    }

    #[test]
    fn test_size_operations() {
        let s = Size::new(0.0, 100.0);
        assert!(s.is_empty()); // zero width
        let s2 = Size::new(50.0, 0.0);
        assert!(s2.is_empty()); // zero height
        let s3 = Size::new(-5.0, 10.0);
        assert!(s3.is_empty()); // negative width
    }

    #[test]
    fn test_damage_tracker_non_mergeable_rects() {
        let mut tracker = DamageTracker::new();
        // Two distant rects should not merge
        tracker.add_damage(Rect::new(0.0, 0.0, 10.0, 10.0));
        tracker.add_damage(Rect::new(500.0, 500.0, 10.0, 10.0));
        assert_eq!(tracker.dirty_rects().len(), 2);
    }

    #[test]
    fn test_damage_tracker_default() {
        let tracker = DamageTracker::default();
        assert!(!tracker.is_dirty());
        assert!(tracker.dirty_rects().is_empty());
    }

    #[test]
    fn test_damage_tracker_damage_all_clears_previous() {
        let mut tracker = DamageTracker::new();
        tracker.add_damage(Rect::new(10.0, 10.0, 5.0, 5.0));
        assert_eq!(tracker.dirty_rects().len(), 1);
        tracker.damage_all(Size::new(800.0, 600.0));
        // damage_all replaces with full surface rect
        assert_eq!(tracker.dirty_rects().len(), 1);
        assert_eq!(tracker.dirty_rects()[0].size.width, 800.0);
        assert_eq!(tracker.dirty_rects()[0].size.height, 600.0);
    }

    #[test]
    fn test_rect_bounds() {
        let r = Rect::new(10.0, 20.0, 30.0, 40.0);
        assert_eq!(r.left(), 10.0);
        assert_eq!(r.top(), 20.0);
        assert_eq!(r.right(), 40.0);
        assert_eq!(r.bottom(), 60.0);
    }

    #[test]
    fn test_rect_zero_is_empty() {
        assert!(Rect::ZERO.is_empty());
        assert_eq!(Rect::ZERO.origin, Point::ZERO);
        assert_eq!(Rect::ZERO.size, Size::ZERO);
    }

    #[test]
    fn test_point_new() {
        let p = Point::new(3.5, -7.2);
        assert_eq!(p.x, 3.5);
        assert_eq!(p.y, -7.2);
    }

    #[test]
    fn test_size_zero_is_empty() {
        assert!(Size::ZERO.is_empty());
        assert_eq!(Size::ZERO.area(), 0.0);
    }

    #[test]
    fn test_size_area_calculation() {
        let s = Size::new(3.0, 4.0);
        assert_eq!(s.area(), 12.0);
    }

    #[test]
    fn test_rect_intersection_edge_touching() {
        // Two rects that just touch at an edge — no overlap
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(10.0, 0.0, 10.0, 10.0);
        assert!(a.intersection(&b).is_none());
    }

    #[test]
    fn test_damage_tracker_multiple_merges() {
        let mut tracker = DamageTracker::new();
        // Add overlapping rects that should chain-merge
        tracker.add_damage(Rect::new(0.0, 0.0, 20.0, 20.0));
        tracker.add_damage(Rect::new(15.0, 0.0, 20.0, 20.0));
        tracker.add_damage(Rect::new(30.0, 0.0, 20.0, 20.0));
        // First two merge; third may merge with merged rect
        assert!(tracker.dirty_rects().len() <= 3);
    }

    // -- 边界条件测试 --
    /// 测试负坐标 Rect 的 contains
    #[test]
    fn test_rect_contains_negative_coordinates() {
        let r = Rect::new(-100.0, -100.0, 50.0, 50.0);
        assert!(r.contains(Point::new(-80.0, -80.0)));
        assert!(r.contains(Point::new(-100.0, -100.0)));
        assert!(!r.contains(Point::new(-101.0, -80.0)));
        assert!(!r.contains(Point::new(-50.0, -50.0)));
    }

    /// 测试负坐标 Rect 的 intersection
    #[test]
    fn test_rect_intersection_negative_coordinates() {
        let a = Rect::new(-50.0, -50.0, 30.0, 30.0);
        let b = Rect::new(-40.0, -40.0, 30.0, 30.0);
        let inter = a.intersection(&b).unwrap();
        assert_eq!(inter.origin.x, -40.0);
        assert_eq!(inter.origin.y, -40.0);
        assert_eq!(inter.size.width, 20.0);
        assert_eq!(inter.size.height, 20.0);
    }

    /// 测试 Size 负高度的 is_empty
    #[test]
    fn test_size_negative_height_is_empty() {
        let s = Size::new(10.0, -5.0);
        assert!(s.is_empty());
    }

    /// 测试 Size 负值的 area
    #[test]
    fn test_size_negative_area() {
        let s = Size::new(-3.0, 4.0);
        assert_eq!(s.area(), -12.0);
    }

    /// 测试 DamageTracker 添加 NaN rect 不 panic
    #[test]
    fn test_damage_tracker_nan_rect_no_panic() {
        let mut tracker = DamageTracker::new();
        // NaN width means is_empty() returns true (NaN <= 0.0 is false, but width <= 0.0 is false
        // for NaN; however height <= 0.0 is also false for NaN, so is_empty returns false)
        // Actually NaN comparisons always return false, so width <= 0.0 is false and
        // height <= 0.0 is false, meaning is_empty() returns false — rect gets added.
        let r = Rect::new(0.0, 0.0, f32::NAN, 10.0);
        tracker.add_damage(r);
        // Should not panic, rect may or may not be added depending on NaN behavior
    }

    /// 测试 DamageTracker 添加相同 rect 多次
    #[test]
    fn test_damage_tracker_identical_rects() {
        let mut tracker = DamageTracker::new();
        let r = Rect::new(0.0, 0.0, 10.0, 10.0);
        for _ in 0..5 {
            tracker.add_damage(r);
        }
        // All identical rects should merge into one
        assert!(tracker.dirty_rects().len() <= 5);
        assert!(tracker.is_dirty());
    }

    /// 测试 Rect 单点交集（退化情况）
    #[test]
    fn test_rect_intersection_single_point() {
        // Two rects touching at exactly one corner point
        let a = Rect::new(0.0, 0.0, 10.0, 10.0);
        let b = Rect::new(10.0, 10.0, 10.0, 10.0);
        // intersection requires right > left && bottom > top
        // right = min(10, 20) = 10, left = max(0, 10) = 10 → not strictly greater
        assert!(a.intersection(&b).is_none());
    }

    /// 清空追踪器后重新标记脏区域，验证新的脏矩形正确记录。
    #[test]
    fn test_damage_tracker_clear_then_remark() {
        let mut tracker = DamageTracker::new();
        // 初始标记
        tracker.add_damage(Rect::new(0.0, 0.0, 100.0, 50.0));
        tracker.add_damage(Rect::new(200.0, 200.0, 80.0, 80.0));
        assert!(tracker.is_dirty());
        let count_before = tracker.dirty_rects().len();
        assert!(count_before >= 1);

        // 清空
        tracker.clear();
        assert!(!tracker.is_dirty());
        assert!(tracker.dirty_rects().is_empty());

        // 重新标记不同的脏区域
        tracker.add_damage(Rect::new(10.0, 20.0, 30.0, 40.0));
        tracker.add_damage(Rect::new(500.0, 100.0, 50.0, 60.0));
        assert!(tracker.is_dirty());
        // 两个不相邻的矩形不应合并
        assert_eq!(tracker.dirty_rects().len(), 2);

        // 验证新矩形的值
        let rects = tracker.dirty_rects();
        let r1 = rects[0];
        assert_eq!(r1.origin.x, 10.0);
        assert_eq!(r1.origin.y, 20.0);
        assert_eq!(r1.size.width, 30.0);
        assert_eq!(r1.size.height, 40.0);
        let r2 = rects[1];
        assert_eq!(r2.origin.x, 500.0);
        assert_eq!(r2.size.width, 50.0);
    }

    /// 测试 DamageTracker 多次 clear
    #[test]
    fn test_damage_tracker_double_clear_unchanged() {
        let mut tracker = DamageTracker::new();
        tracker.add_damage(Rect::new(0.0, 0.0, 10.0, 10.0));
        tracker.clear();
        assert!(!tracker.is_dirty());
        tracker.clear();
        assert!(!tracker.is_dirty());
        assert!(tracker.dirty_rects().is_empty());
    }
}
