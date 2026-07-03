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

    #[test]
    fn invisible_propagates_sticky_zero() {
        // 深度审查（lei-deep-review）：一旦某层 clip 收缩为 ZERO（不可见），
        // 其所有后继层应保持 ZERO（ZERO ∩ 任何 rect = None → unwrap_or(ZERO)）。
        // 保证不可见子树整支被裁掉，不会因后续 push “复活”。
        let mut s = ClipStack::new();
        s.push(Rect::from_ltrb(0.0, 0.0, 10.0, 10.0));
        s.push(Rect::from_ltrb(100.0, 100.0, 110.0, 110.0)); // 无交集 → ZERO
        assert_eq!(s.current(), Some(Rect::ZERO));
        // 第三层即便与第一层有交集，仍应保持 ZERO（当前有效 clip 已是 ZERO）。
        s.push(Rect::from_ltrb(5.0, 5.0, 8.0, 8.0));
        assert_eq!(s.current(), Some(Rect::ZERO), "不可见层传播：后继 push 不复活");
        // pop 回到不可见层，仍是 ZERO。
        s.pop();
        assert_eq!(s.current(), Some(Rect::ZERO));
        // 再 pop 回第一层才恢复可见。
        s.pop();
        assert_eq!(s.current(), Some(Rect::from_ltrb(0.0, 0.0, 10.0, 10.0)));
    }
}
