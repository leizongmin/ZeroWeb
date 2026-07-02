//! Popup — 短暂浮层（非 modal，spec FR-009）。
//!
//! 用于 tooltip 瞬态、下拉建议等；不设焦点陷阱。modal/带遮罩见 `popover`/`overlay`。

use zero_ui_core::geometry::Rect;

#[derive(Debug, Clone, PartialEq)]
pub struct Popup {
    pub open: bool,
    pub anchor_rect: Rect,
}

impl Popup {
    pub fn new(anchor_rect: Rect) -> Popup {
        Popup {
            open: false,
            anchor_rect,
        }
    }
    pub fn open(&mut self) {
        self.open = true;
    }
    pub fn close(&mut self) {
        self.open = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_close_toggle() {
        let mut p = Popup::new(Rect::ZERO);
        assert!(!p.open);
        p.open();
        assert!(p.open);
        p.close();
        assert!(!p.open);
    }
}
