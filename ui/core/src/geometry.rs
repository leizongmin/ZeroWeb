//! 几何基础类型 — 点、尺寸、矩形、向量、内边距、约束、圆角。
//!
//! 所有数值采用逻辑像素（logical px），与 device pixel 之间通过 `scale_factor` 换算。

use serde::{Deserialize, Serialize};

/// 二维点（逻辑像素）。
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub const ZERO: Point = Point { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Point {
        Point { x, y }
    }

    /// 沿向量平移。
    pub fn translate(self, dx: f32, dy: f32) -> Point {
        Point {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    pub fn scale(self, factor: f32) -> Point {
        Point { x: self.x * factor, y: self.y * factor }
    }
}

/// 二维尺寸（逻辑像素，非负）。
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };

    pub const fn new(width: f32, height: f32) -> Size {
        Size { width, height }
    }

    /// 将负分量截断为 0（布局结果不应有负尺寸）。
    pub fn clamp_nonnegative(self) -> Size {
        Size {
            width: self.width.max(0.0),
            height: self.height.max(0.0),
        }
    }
}

/// 二维向量（逻辑像素，可负）。
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };

    pub const fn new(x: f32, y: f32) -> Vec2 {
        Vec2 { x, y }
    }
}

/// 矩形（左上角 + 尺寸，逻辑像素）。
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub const ZERO: Rect = Rect {
        origin: Point::ZERO,
        size: Size::ZERO,
    };

    pub const fn from_origin_size(origin: Point, size: Size) -> Rect {
        Rect { origin, size }
    }

    pub const fn from_ltrb(left: f32, top: f32, right: f32, bottom: f32) -> Rect {
        Rect {
            origin: Point::new(left, top),
            size: Size::new(right - left, bottom - top),
        }
    }

    pub fn left(self) -> f32 {
        self.origin.x
    }
    pub fn top(self) -> f32 {
        self.origin.y
    }
    pub fn right(self) -> f32 {
        self.origin.x + self.size.width
    }
    pub fn bottom(self) -> f32 {
        self.origin.y + self.size.height
    }

    /// 点是否落在矩形内（含边界）。
    pub fn contains(self, p: Point) -> bool {
        p.x >= self.left() && p.x <= self.right() && p.y >= self.top() && p.y <= self.bottom()
    }

    /// 沿向量平移（返回新矩形）。
    pub fn translate(self, dx: f32, dy: f32) -> Rect {
        Rect {
            origin: self.origin.translate(dx, dy),
            size: self.size,
        }
    }

    /// 按 scale_factor 缩放位置与尺寸。
    pub fn scale(self, factor: f32) -> Rect {
        Rect {
            origin: self.origin.scale(factor),
            size: Size { width: self.size.width * factor, height: self.size.height * factor },
        }
    }

    /// 两个矩形的交集；无交集返回 None。
    pub fn intersect(self, other: Rect) -> Option<Rect> {
        let left = self.left().max(other.left());
        let top = self.top().max(other.top());
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right <= left || bottom <= top {
            None
        } else {
            Some(Rect::from_ltrb(left, top, right, bottom))
        }
    }
}

/// 四边内边距/外边距（逻辑像素）。
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Insets {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl Insets {
    pub const fn all(v: f32) -> Insets {
        Insets {
            left: v,
            top: v,
            right: v,
            bottom: v,
        }
    }

    pub const fn horizontal(self) -> f32 {
        self.left + self.right
    }
    pub const fn vertical(self) -> f32 {
        self.top + self.bottom
    }

    /// 收缩矩形。
    pub fn deflate_rect(self, rect: Rect) -> Rect {
        Rect::from_ltrb(
            rect.left() + self.left,
            rect.top() + self.top,
            rect.right() - self.right,
            rect.bottom() - self.bottom,
        )
    }
}

/// 圆角（四个角可独立设置，逻辑像素）。
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Rounding {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
}

impl Rounding {
    pub const fn all(v: f32) -> Rounding {
        Rounding {
            top_left: v,
            top_right: v,
            bottom_right: v,
            bottom_left: v,
        }
    }

    pub fn scale(self, factor: f32) -> Rounding {
        Rounding {
            top_left: self.top_left * factor,
            top_right: self.top_right * factor,
            bottom_right: self.bottom_right * factor,
            bottom_left: self.bottom_left * factor,
        }
    }

    pub const ZERO: Rounding = Rounding::all(0.0);
}

/// 布局约束（spec §8.4.4：constraints down / size up / position down）。
///
/// 子节点最终尺寸必须落在 `[min, max]` 区间内。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Constraints {
    pub min_width: f32,
    pub max_width: f32,
    pub min_height: f32,
    pub max_height: f32,
}

impl Constraints {
    /// 严格固定尺寸约束。
    pub const fn tight(size: Size) -> Constraints {
        Constraints {
            min_width: size.width,
            max_width: size.width,
            min_height: size.height,
            max_height: size.height,
        }
    }

    /// `[0, max]` 的宽松约束（子节点可任意小）。
    pub const fn loose(max: Size) -> Constraints {
        Constraints {
            min_width: 0.0,
            max_width: max.width,
            min_height: 0.0,
            max_height: max.height,
        }
    }

    /// 校验给定尺寸是否满足约束。
    pub fn is_satisfied(self, size: Size) -> bool {
        size.width >= self.min_width
            && size.width <= self.max_width
            && size.height >= self.min_height
            && size.height <= self.max_height
    }

    /// 收紧约束（叠加父级内边距）。
    pub fn deflate(self, insets: Insets) -> Constraints {
        Constraints {
            min_width: (self.min_width - insets.horizontal()).max(0.0),
            max_width: (self.max_width - insets.horizontal()).max(0.0),
            min_height: (self.min_height - insets.vertical()).max(0.0),
            max_height: (self.max_height - insets.vertical()).max(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rect_contains_and_intersect() {
        let a = Rect::from_ltrb(0.0, 0.0, 100.0, 100.0);
        assert!(a.contains(Point::new(50.0, 50.0)));
        assert!(!a.contains(Point::new(150.0, 50.0)));

        let b = Rect::from_ltrb(50.0, 50.0, 150.0, 150.0);
        assert_eq!(a.intersect(b), Some(Rect::from_ltrb(50.0, 50.0, 100.0, 100.0)));
        assert_eq!(a.intersect(Rect::from_ltrb(200.0, 200.0, 300.0, 300.0)), None);
    }

    #[test]
    fn rect_contains_is_edge_inclusive() {
        // 深度审查（lei-deep-review）：contains 边界含端点（<= / >=），hit-test 据此判定。
        // 锁定该语义——相邻 rect 共享边界点会同时 contains（hit_test 由 z 序 rev() 仲裁）。
        let r = Rect::from_ltrb(0.0, 0.0, 100.0, 100.0);
        // 四角（端点）含。
        assert!(r.contains(Point::new(0.0, 0.0)));
        assert!(r.contains(Point::new(100.0, 100.0)));
        assert!(r.contains(Point::new(0.0, 100.0)));
        assert!(r.contains(Point::new(100.0, 0.0)));
        // 边上含。
        assert!(r.contains(Point::new(50.0, 0.0)));
        assert!(r.contains(Point::new(100.0, 50.0)));
        // 紧贴外侧不含。
        assert!(!r.contains(Point::new(-0.001, 50.0)));
        assert!(!r.contains(Point::new(100.001, 50.0)));
    }

    #[test]
    fn rect_intersect_edge_touching_is_none() {
        // 深度审查（lei-deep-review）：intersect 用 `right <= left || bottom <= top` 判空，
        // 故**恰好共享一条边**（边相接）→ None（非零面积 rect）。此语义承载 host clip 链
        // （ui/render 审查 O1：相邻/相接节点经 intersect 得 None → clip=None）。
        let a = Rect::from_ltrb(0.0, 0.0, 100.0, 100.0);
        // 右边与 b 左边相接于 x=100 → None。
        let touch_x = Rect::from_ltrb(100.0, 0.0, 200.0, 100.0);
        assert_eq!(a.intersect(touch_x), None, "x 边相接 → None");
        // 底边与 b 顶边相接于 y=100 → None。
        let touch_y = Rect::from_ltrb(0.0, 100.0, 100.0, 200.0);
        assert_eq!(a.intersect(touch_y), None, "y 边相接 → None");
        // 1px 真重叠 → Some（非 None）。
        let overlap1 = Rect::from_ltrb(99.0, 0.0, 199.0, 100.0);
        assert_eq!(a.intersect(overlap1), Some(Rect::from_ltrb(99.0, 0.0, 100.0, 100.0)));
    }

    #[test]
    fn constraints_tight_loose_satisfied() {
        let tight = Constraints::tight(Size::new(40.0, 60.0));
        assert!(tight.is_satisfied(Size::new(40.0, 60.0)));
        assert!(!tight.is_satisfied(Size::new(41.0, 60.0)));

        let loose = Constraints::loose(Size::new(100.0, 100.0));
        assert!(loose.is_satisfied(Size::ZERO));
        assert!(loose.is_satisfied(Size::new(100.0, 100.0)));
        assert!(!loose.is_satisfied(Size::new(101.0, 0.0)));
    }

    #[test]
    fn insets_deflate_rect_and_constraints() {
        let insets = Insets::all(10.0);
        let rect = Rect::from_ltrb(0.0, 0.0, 100.0, 100.0);
        assert_eq!(insets.deflate_rect(rect), Rect::from_ltrb(10.0, 10.0, 90.0, 90.0));

        let c = Constraints::tight(Size::new(100.0, 100.0)).deflate(insets);
        assert_eq!(c.max_width, 80.0);
    }
}
