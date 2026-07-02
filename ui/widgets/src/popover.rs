//! Popover — 锚定浮层（spec FR-009）。
//!
//! 相对锚矩形定位的浮层（地址栏建议、site info 面板等）；可含焦点陷阱（modal 见 `overlay`）。

use zero_ui_core::geometry::Rect;

/// 相对锚的定位偏好。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopoverPlacement {
    Below,
    Above,
    Left,
    Right,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Popover {
    pub open: bool,
    pub anchor_rect: Rect,
    pub placement: PopoverPlacement,
}

impl Popover {
    pub fn new(anchor_rect: Rect, placement: PopoverPlacement) -> Popover {
        Popover {
            open: false,
            anchor_rect,
            placement,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placement_default() {
        let p = Popover::new(Rect::ZERO, PopoverPlacement::Below);
        assert_eq!(p.placement, PopoverPlacement::Below);
        assert!(!p.open);
    }
}
