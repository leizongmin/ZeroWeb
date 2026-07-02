//! winit 事件 → UiEvent 基础映射（spec §8.4.3 事件分发）。

use zero_ui_core::event::{Modifiers, PointerButton};

/// winit 鼠标按钮 → UI PointerButton。
///
/// Back/Forward 映射到 `Other`（chrome 可按需识别）。
pub fn map_mouse_button(b: winit::event::MouseButton) -> PointerButton {
    match b {
        winit::event::MouseButton::Left => PointerButton::Primary,
        winit::event::MouseButton::Right => PointerButton::Secondary,
        winit::event::MouseButton::Middle => PointerButton::Middle,
        winit::event::MouseButton::Back => PointerButton::Other(3),
        winit::event::MouseButton::Forward => PointerButton::Other(4),
        winit::event::MouseButton::Other(code) => PointerButton::Other(code),
    }
}

/// winit 修饰键 → UI Modifiers。
pub fn map_modifiers(mods: winit::event::Modifiers) -> Modifiers {
    let s = mods.state();
    let mut out = Modifiers::NONE;
    if s.contains(winit::keyboard::ModifiersState::SHIFT) {
        out |= Modifiers::SHIFT;
    }
    if s.contains(winit::keyboard::ModifiersState::CONTROL) {
        out |= Modifiers::CONTROL;
    }
    if s.contains(winit::keyboard::ModifiersState::ALT) {
        out |= Modifiers::ALT;
    }
    if s.contains(winit::keyboard::ModifiersState::SUPER) {
        out |= Modifiers::SUPER;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_buttons() {
        assert_eq!(
            map_mouse_button(winit::event::MouseButton::Left),
            PointerButton::Primary
        );
        assert_eq!(
            map_mouse_button(winit::event::MouseButton::Right),
            PointerButton::Secondary
        );
        assert_eq!(
            map_mouse_button(winit::event::MouseButton::Middle),
            PointerButton::Middle
        );
        assert_eq!(
            map_mouse_button(winit::event::MouseButton::Back),
            PointerButton::Other(3)
        );
        assert_eq!(
            map_mouse_button(winit::event::MouseButton::Forward),
            PointerButton::Other(4)
        );
        assert_eq!(
            map_mouse_button(winit::event::MouseButton::Other(9)),
            PointerButton::Other(9)
        );
    }

    #[test]
    fn map_modifiers_combines() {
        let state = winit::keyboard::ModifiersState::SHIFT | winit::keyboard::ModifiersState::CONTROL;
        let mods: winit::event::Modifiers = state.into();
        let ui = map_modifiers(mods);
        assert!(ui.contains(Modifiers::SHIFT));
        assert!(ui.contains(Modifiers::CONTROL));
        assert!(!ui.contains(Modifiers::ALT));
        assert!(!ui.contains(Modifiers::SUPER));
    }
}
