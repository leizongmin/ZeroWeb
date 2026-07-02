//! 裁剪栈（spec §8.4.1 `clip.rs`）。
//!
//! paint/layout 下钻时维护当前有效裁剪矩形（父裁剪 ∩ 自身裁剪），
//! 使 hit-test 与绘制只在可见区域内生效。

use zero_ui_core::geometry::Rect;

/// 裁剪栈。空栈表示无裁剪（无限大）。
#[derive(Debug, Clone, Default)]
pub struct ClipStack {
    stack: Vec<Rect>,
}

impl ClipStack {
    pub fn new() -> ClipStack {
        ClipStack::default()
    }

    /// 压入一层裁剪；新有效裁剪 = 旧有效 ∩ 新矩形（无交集则该子树不可见）。
    pub fn push(&mut self, rect: Rect) {
        let next = self.current().map(|c| c.intersect(rect)).unwrap_or(Some(rect));
        // intersect 失败（无交集）时压入一个面积为 0 的矩形以表示不可见。
        self.stack.push(next.unwrap_or(Rect::ZERO));
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    /// 当前有效裁剪矩形；None 表示无裁剪。
    pub fn current(&self) -> Option<Rect> {
        self.stack.last().copied()
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_narrows_clip() {
        let mut s = ClipStack::new();
        s.push(Rect::from_ltrb(0.0, 0.0, 100.0, 100.0));
        s.push(Rect::from_ltrb(50.0, 50.0, 150.0, 150.0));
        assert_eq!(s.current(), Some(Rect::from_ltrb(50.0, 50.0, 100.0, 100.0)));
    }

    #[test]
    fn disjoint_becomes_zero_rect() {
        let mut s = ClipStack::new();
        s.push(Rect::from_ltrb(0.0, 0.0, 10.0, 10.0));
        s.push(Rect::from_ltrb(100.0, 100.0, 110.0, 110.0));
        // 无交集 → 不可见（ZERO 矩形）。
        assert_eq!(s.current(), Some(Rect::ZERO));
    }
}
