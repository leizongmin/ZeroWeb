//! CSS 布局辅助：BFC 检测 + 外边距折叠工具。
//!
//! ## 外边距折叠
//!
//! Taffy 0.7 已内置 CSS 块级外边距折叠（`CollapsibleMarginSet`），
//! 不需要额外的后处理步骤。此模块提供折叠计算工具函数供测试使用。
//!
//! ## BFC（Block Formatting Context）
//!
//! 根据 CSS 2.1 §9.4.1，以下条件建立新的 BFC：
//! - `overflow` 不为 `visible`
//! - `display: flow-root`
//! - `float` 不为 `none`
//! - `position: absolute` 或 `fixed`
//! - `display: inline-block`
//!
//! BFC 影响：
//! - 包含浮动元素（float containment）
//! - 阻止外边距折叠（margin collapse isolation）
//! - 隔离布局上下文

use crate::types::{LayoutBox, OverflowClip};
use zero_css_parser::values::FloatValue;

/// 判断一个 LayoutBox 是否建立了新的 BFC（Block Formatting Context）。
///
/// 根据 CSS 2.1 §9.4.1，以下条件建立 BFC：
/// - `overflow` 不为 `visible`
/// - `float` 不为 `none`
/// - `position: absolute` 或 `fixed`
/// - `display: flow-root` / `inline-block`（需要 computed style，暂不支持）
/// - 根元素
pub fn establishes_bfc(box_node: &LayoutBox) -> bool {
    // overflow != visible → BFC
    if box_node.overflow_x != OverflowClip::Visible || box_node.overflow_y != OverflowClip::Visible {
        return true;
    }

    // float != none → BFC
    if !matches!(box_node.float, FloatValue::None) {
        return true;
    }

    // position: absolute/fixed → BFC
    if box_node.is_absolute || box_node.is_fixed {
        return true;
    }

    // display: flow-root → BFC
    if box_node.is_flow_root {
        return true;
    }

    false
}

/// 折叠两个 margin 值（CSS 2.1 §8.3.1 折叠规则）。
///
/// - 两个正 margin → 取较大值
/// - 两个负 margin → 取更负的值
/// - 一正一负 → 相加
pub fn collapse_two_margins(m1: f32, m2: f32) -> f32 {
    if m1 >= 0.0 && m2 >= 0.0 {
        m1.max(m2)
    } else if m1 < 0.0 && m2 < 0.0 {
        // 取更负的值
        if m1 < m2 { m1 } else { m2 }
    } else {
        // 一正一负：相加
        m1 + m2
    }
}

/// 判断一个 LayoutBox 是否为空块（无内容、无 border/padding/height）。
///
/// 空块的 margin-top 和 margin-bottom 会自折叠（CSS 2.1 §8.3.1 第4条）。
pub fn is_empty_block(box_node: &LayoutBox) -> bool {
    box_node.height == 0.0
        && box_node.border_top == 0.0
        && box_node.border_bottom == 0.0
        && box_node.padding_top == 0.0
        && box_node.padding_bottom == 0.0
        && box_node.content_height == 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collapse_two_positive_margins() {
        assert_eq!(collapse_two_margins(20.0, 30.0), 30.0);
        assert_eq!(collapse_two_margins(10.0, 10.0), 10.0);
        assert_eq!(collapse_two_margins(0.0, 15.0), 15.0);
    }

    #[test]
    fn test_collapse_two_negative_margins() {
        assert_eq!(collapse_two_margins(-20.0, -30.0), -30.0);
        assert_eq!(collapse_two_margins(-10.0, -10.0), -10.0);
    }

    #[test]
    fn test_collapse_mixed_margins() {
        assert_eq!(collapse_two_margins(20.0, -10.0), 10.0);
        assert_eq!(collapse_two_margins(-20.0, 30.0), 10.0);
        assert_eq!(collapse_two_margins(10.0, -20.0), -10.0);
    }

    #[test]
    fn test_establishes_bfc_default() {
        let bx = LayoutBox::default();
        assert!(!establishes_bfc(&bx), "default LayoutBox should not establish BFC");
    }

    #[test]
    fn test_establishes_bfc_overflow_hidden() {
        let mut bx = LayoutBox::default();
        bx.overflow_x = OverflowClip::Hidden;
        assert!(establishes_bfc(&bx));
    }

    #[test]
    fn test_establishes_bfc_overflow_scroll() {
        let mut bx = LayoutBox::default();
        bx.overflow_y = OverflowClip::Scroll;
        assert!(establishes_bfc(&bx));
    }

    #[test]
    fn test_establishes_bfc_float() {
        let mut bx = LayoutBox::default();
        bx.float = FloatValue::Left;
        assert!(establishes_bfc(&bx));
        bx.float = FloatValue::Right;
        assert!(establishes_bfc(&bx));
    }

    #[test]
    fn test_establishes_bfc_absolute() {
        let mut bx = LayoutBox::default();
        bx.is_absolute = true;
        assert!(establishes_bfc(&bx));
    }

    #[test]
    fn test_establishes_bfc_fixed() {
        let mut bx = LayoutBox::default();
        bx.is_fixed = true;
        assert!(establishes_bfc(&bx));
    }

    #[test]
    fn test_is_empty_block_default() {
        let bx = LayoutBox::default();
        assert!(is_empty_block(&bx));
    }

    #[test]
    fn test_is_not_empty_with_height() {
        let mut bx = LayoutBox::default();
        bx.height = 10.0;
        assert!(!is_empty_block(&bx));
    }

    #[test]
    fn test_is_not_empty_with_border() {
        let mut bx = LayoutBox::default();
        bx.border_top = 1.0;
        assert!(!is_empty_block(&bx));
    }

    #[test]
    fn test_is_not_empty_with_padding() {
        let mut bx = LayoutBox::default();
        bx.padding_bottom = 5.0;
        assert!(!is_empty_block(&bx));
    }
}
