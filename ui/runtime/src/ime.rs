//! IME 控制（spec FR-011 / DC-8）。
//!
//! TextInput 把当前光标对应的屏幕 rect 上报给 runtime；runtime 通过
//! `PlatformRuntime::set_ime_area` 通知平台 IME。RTL/换行/选区改变光标 rect。

use zero_ui_core::geometry::Rect;

/// IME 光标区域控制器。
#[derive(Debug, Default, Clone)]
pub struct ImeController {
    current: Option<Rect>,
}

impl ImeController {
    pub fn new() -> ImeController {
        ImeController::default()
    }

    /// 更新当前 IME 区域；返回是否变化（变化时宿主需 set_ime_area）。
    pub fn update(&mut self, rect: Option<Rect>) -> bool {
        if rect == self.current {
            false
        } else {
            self.current = rect;
            true
        }
    }

    pub fn current(&self) -> Option<Rect> {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn change_detected() {
        let mut ime = ImeController::new();
        assert!(ime.update(Some(Rect::ZERO))); // None → Some
        assert!(!ime.update(Some(Rect::ZERO))); // 相同 → 不变
        assert!(ime.update(Some(Rect::from_ltrb(1.0, 1.0, 2.0, 2.0)))); // 移动
        assert!(ime.update(None)); // 隐藏
        assert!(!ime.update(None));
    }
}
