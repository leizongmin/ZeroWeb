//! winit 事件 → [`UiEvent`] 映射（spec §8.4.3 事件分发 / IF-006 / FR-010）。
//!
//! 本模块是 winit 原始事件到浏览器无关 [`UiEvent`] 的**纯函数转换层**——无窗口、
//! 无副作用、完全可单测。`ui-runtime` / `ui-widgets` 等只消费 `UiEvent`，永不接触
//! winit 类型（spec §6.4：winit 类型的唯一落点为本 adapter）。
//!
//! 真实 winit `EventLoop` 驱动（窗口/surface/run loop）在 M4；本模块覆盖桌面端所需的
//! 全部事件种类：指针（鼠标按下/释放/移动/触摸）、键盘、滚轮、IME、窗口度量。
//!
//! ## 单位与符号约定
//! - 位置一律 **逻辑像素**；winit 给的是物理像素，调用方经 [`to_logical_point`] /
//!   [`to_logical_size`] 或带 `scale_factor` 的映射函数转换。
//! - 滚动增量 `Vec2`：x 向右为正、y **向下为正**（与 [`UiEvent::Scroll`] 消费端
//!   `scroll_y += delta.y` 一致：正 y = 向下滚动看更下方内容）。winit 的
//!   `LineDelta` / `PixelDelta` y 向上为正，故映射时**取反 y**。

use zero_ui_core::event::{ImeEvent, KeyAction, KeyCode, Modifiers, PointerButton, PointerPhase, ScrollPhase, UiEvent};
use zero_ui_core::geometry::{Insets, Point, Size, Vec2};
use zero_ui_core::layout::WindowMetrics;

/// 行高（逻辑像素）—— `MouseScrollDelta::LineDelta` → 像素的换算因子
/// （与浏览器 chrome 行高量级一致；spec §8.4.3 滚动约定）。
pub const LINE_HEIGHT_PX: f32 = 20.0;

// ---------------------------------------------------------------------------
// 修饰键 / 鼠标按钮
// ---------------------------------------------------------------------------

/// winit 鼠标按钮 → UI [`PointerButton`]。
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

/// winit 修饰键 → UI [`Modifiers`]。
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

// ---------------------------------------------------------------------------
// 物理像素 → 逻辑像素
// ---------------------------------------------------------------------------

/// 物理坐标点 → 逻辑坐标点。`scale_factor <= 0` 视为 1.0（防御除零）。
pub fn to_logical_point(physical: winit::dpi::PhysicalPosition<f64>, scale_factor: f32) -> Point {
    let s = if scale_factor > 0.0 { scale_factor as f64 } else { 1.0 };
    Point::new((physical.x / s) as f32, (physical.y / s) as f32)
}

/// 物理尺寸 → 逻辑尺寸。`scale_factor <= 0` 视为 1.0（防御除零）。
pub fn to_logical_size(physical: winit::dpi::PhysicalSize<u32>, scale_factor: f32) -> Size {
    let s = if scale_factor > 0.0 { scale_factor as f64 } else { 1.0 };
    Size::new((physical.width as f64 / s) as f32, (physical.height as f64 / s) as f32)
}

/// winit 窗口物理尺寸 + scale → UI [`WindowMetrics`]（resize 事件桥）。
///
/// `safe_area` / `keyboard_insets` 初始为 0（桌面无刘海/软键盘遮挡；移动端由 M4
/// 运行时按平台 inset 填充）。`text_scale`/`density` 固定为默认（桌面不放大；移动端由 M4
/// runtime 从系统字号/密度设置探测后覆盖）。`orientation` 由逻辑尺寸派生。
pub fn map_window_metrics(physical_size: winit::dpi::PhysicalSize<u32>, scale_factor: f32) -> WindowMetrics {
    let logical_size = to_logical_size(physical_size, scale_factor);
    WindowMetrics {
        logical_size,
        scale_factor,
        safe_area: Insets::all(0.0),
        keyboard_insets: Insets::all(0.0),
        text_scale: zero_ui_core::layout::DEFAULT_TEXT_SCALE,
        density: zero_ui_core::layout::DEFAULT_DENSITY,
        orientation: zero_ui_core::layout::Orientation::from_size(logical_size),
    }
}

// ---------------------------------------------------------------------------
// 键盘
// ---------------------------------------------------------------------------

/// winit `ElementState` + repeat 标志 → UI [`KeyAction`]。
///
/// `repeat == true` 一律为 [`KeyAction::Repeat`]（winit 仅在 Pressed 时置 repeat）。
pub fn map_key_action(state: winit::event::ElementState, repeat: bool) -> KeyAction {
    if repeat {
        KeyAction::Repeat
    } else {
        match state {
            winit::event::ElementState::Pressed => KeyAction::Pressed,
            winit::event::ElementState::Released => KeyAction::Released,
        }
    }
}

/// winit 逻辑键 [`winit::keyboard::Key`] → UI [`KeyCode`]（稳定字符串）。
///
/// 命名与 `crates/host-runtime` 既有转换一致：`Named` 取 `Debug` 形态
/// （`NamedKey::Tab` → `"Tab"`、`ArrowLeft` → `"ArrowLeft"`），`Character` 取字面字符，
/// `Dead` 取 `"Dead"` 或 `"Dead({ch})"`，`Unidentified` → `"Unidentified"`。
/// `ui-runtime` 焦点遍历按 `"Tab"` 识别 Tab 键。
pub fn map_logical_key(key: &winit::keyboard::Key) -> KeyCode {
    match key {
        winit::keyboard::Key::Named(named) => KeyCode::new(&format!("{named:?}")),
        winit::keyboard::Key::Character(ch) => KeyCode::new(ch.as_str()),
        winit::keyboard::Key::Unidentified(_) => KeyCode::new("Unidentified"),
        winit::keyboard::Key::Dead(dead) => match dead {
            Some(ch) => KeyCode::new(&format!("Dead({ch})")),
            None => KeyCode::new("Dead"),
        },
    }
}

/// winit `KeyEvent.text`（`Option<SmolStr>`）→ UI 可打印文本（空串视为无文本）。
///
/// 抽出为独立纯函数以便单测（`winit::event::KeyEvent` 含 `pub(crate)` 字段，
/// 仓外无法构造，故 [`map_key_event`] 的文本路径经本函数验证）。
pub fn key_text(text: Option<&winit::keyboard::SmolStr>) -> Option<String> {
    text.filter(|t| !t.is_empty()).map(|t| t.to_string())
}

/// winit `KeyEvent` → UI [`UiEvent::Key`]。
///
/// 组合 [`map_logical_key`] / [`map_key_action`] / [`key_text`]；`KeyEvent` 含
/// `pub(crate)` 字段无法在 winit 仓外构造，故本函数本身不直接单测，正确性由上述
/// 三个子函数的单测覆盖。
pub fn map_key_event(event: &winit::event::KeyEvent, modifiers: Modifiers) -> UiEvent {
    UiEvent::Key {
        code: map_logical_key(&event.logical_key),
        action: map_key_action(event.state, event.repeat),
        modifiers,
        text: key_text(event.text.as_ref()),
    }
}

// ---------------------------------------------------------------------------
// 指针（鼠标 / 触摸）
// ---------------------------------------------------------------------------

/// winit `ElementState` → UI [`PointerPhase`]（按下/释放）。
pub fn map_pointer_phase(state: winit::event::ElementState) -> PointerPhase {
    match state {
        winit::event::ElementState::Pressed => PointerPhase::Pressed,
        winit::event::ElementState::Released => PointerPhase::Released,
    }
}

/// winit 鼠标按下/释放（`MouseInput`）→ UI [`UiEvent::Pointer`]。
///
/// `position` 须为逻辑像素（由调用方经 [`to_logical_point`] 转换）。
pub fn map_mouse_input(
    button: winit::event::MouseButton,
    state: winit::event::ElementState,
    position: Point,
    modifiers: Modifiers,
) -> UiEvent {
    UiEvent::Pointer {
        phase: map_pointer_phase(state),
        button: Some(map_mouse_button(button)),
        position,
        modifiers,
        pointer_id: 0,
    }
}

/// winit 光标移动（`CursorMoved`）→ UI [`UiEvent::Pointer`]（Moved，无按键）。
///
/// `position` 须为逻辑像素。
pub fn map_cursor_moved(position: Point, modifiers: Modifiers) -> UiEvent {
    UiEvent::Pointer {
        phase: PointerPhase::Moved,
        button: None,
        position,
        modifiers,
        pointer_id: 0,
    }
}

/// winit `TouchPhase` → UI [`PointerPhase`]。
pub fn map_touch_phase(phase: winit::event::TouchPhase) -> PointerPhase {
    match phase {
        winit::event::TouchPhase::Started => PointerPhase::Pressed,
        winit::event::TouchPhase::Moved => PointerPhase::Moved,
        winit::event::TouchPhase::Ended => PointerPhase::Released,
        winit::event::TouchPhase::Cancelled => PointerPhase::Cancelled,
    }
}

/// 触摸事件 → UI [`UiEvent::Pointer`]。
///
/// `winit::event::Touch` 含 `pub(crate) DeviceId` 无法仓外构造，故取可构造的
/// `(phase, location, touch_id)` 入参；调用方传 `touch.phase` / `touch.location` /
/// `touch.id as u32` 即可（`winit::event::TouchEvent.id: u64` 为各手指稳定 id）。
/// 触摸 `Started` 视为主按键按下（等同鼠标左键，供 click-to-focus / tap），
/// 其余阶段 `button = None`（phase 已编码按下/释放）。
pub fn map_touch(
    phase: winit::event::TouchPhase,
    location: winit::dpi::PhysicalPosition<f64>,
    touch_id: u32,
    scale_factor: f32,
    modifiers: Modifiers,
) -> UiEvent {
    let pointer_phase = map_touch_phase(phase);
    UiEvent::Pointer {
        phase: pointer_phase,
        button: if matches!(pointer_phase, PointerPhase::Pressed) {
            Some(PointerButton::Primary)
        } else {
            None
        },
        position: to_logical_point(location, scale_factor),
        modifiers,
        pointer_id: touch_id,
    }
}

// ---------------------------------------------------------------------------
// 滚轮
// ---------------------------------------------------------------------------

/// winit `MouseScrollDelta` → UI [`UiEvent::Scroll`]。
///
/// - `LineDelta(x, y)` → `(x * [`LINE_HEIGHT_PX`], -y * [`LINE_HEIGHT_PX`])`。
/// - `PixelDelta(physical)` → 物理像素除 `scale_factor` 得逻辑像素，y 取反。
///
/// 取反 y：winit 滚动 y 向上为正，而 UI `delta.y` 正 = 向下（见模块约定）。
/// `position` 须为逻辑像素。
pub fn map_mouse_wheel(
    delta: winit::event::MouseScrollDelta,
    scale_factor: f32,
    position: Point,
    modifiers: Modifiers,
) -> UiEvent {
    let (dx, dy) = match delta {
        winit::event::MouseScrollDelta::LineDelta(x, y) => (x * LINE_HEIGHT_PX, -y * LINE_HEIGHT_PX),
        winit::event::MouseScrollDelta::PixelDelta(px) => {
            let s = if scale_factor > 0.0 { scale_factor as f64 } else { 1.0 };
            ((px.x / s) as f32, (-px.y / s) as f32)
        }
    };
    UiEvent::Scroll {
        delta: Vec2::new(dx, dy),
        phase: ScrollPhase::Discrete,
        position,
        modifiers,
    }
}

// ---------------------------------------------------------------------------
// 输入法（IME）
// ---------------------------------------------------------------------------

/// winit [`winit::event::Ime`] → UI [`UiEvent::Ime`]。
///
/// `Preedit` 的 winit 光标是 `Option<(start, end)>` 选择区；UI [`ImeEvent`]
/// 用单光标位，取 `start` 作为锚点。
pub fn map_ime(ime: winit::event::Ime) -> UiEvent {
    let ev = match ime {
        winit::event::Ime::Enabled => ImeEvent::Enabled,
        winit::event::Ime::Disabled => ImeEvent::Disabled,
        winit::event::Ime::Preedit(text, cursor) => ImeEvent::Preedit {
            text,
            cursor: cursor.map(|(start, _)| start),
        },
        winit::event::Ime::Commit(text) => ImeEvent::Commit(text),
    };
    UiEvent::Ime(ev)
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

    // ---- 物理像素 → 逻辑像素 ----

    #[test]
    fn to_logical_point_and_size_scale() {
        let p = to_logical_point(winit::dpi::PhysicalPosition::new(200.0, 100.0), 2.0);
        assert_eq!(p, Point::new(100.0, 50.0));

        let s = to_logical_size(winit::dpi::PhysicalSize::new(1600, 900), 1.25);
        assert_eq!(s, Size::new(1280.0, 720.0));
    }

    #[test]
    fn to_logical_handles_zero_scale() {
        // scale<=0 视为 1.0，不除零。
        let p = to_logical_point(winit::dpi::PhysicalPosition::new(42.0, 17.0), 0.0);
        assert_eq!(p, Point::new(42.0, 17.0));
    }

    #[test]
    fn window_metrics_from_physical() {
        let m = map_window_metrics(winit::dpi::PhysicalSize::new(1920, 1080), 1.5);
        assert_eq!(m.logical_size, Size::new(1280.0, 720.0));
        assert!((m.scale_factor - 1.5).abs() < 1e-6);
        assert_eq!(m.safe_area, Insets::all(0.0));
        assert_eq!(m.keyboard_insets, Insets::all(0.0));
    }

    // ---- 键盘 ----

    #[test]
    fn key_action_maps_state_and_repeat() {
        use winit::event::ElementState;
        assert_eq!(map_key_action(ElementState::Pressed, false), KeyAction::Pressed);
        assert_eq!(map_key_action(ElementState::Released, false), KeyAction::Released);
        assert_eq!(map_key_action(ElementState::Pressed, true), KeyAction::Repeat);
        // repeat 仅在 Pressed 出现；Released+repeat 仍按 Repeat（winit 不会发）。
        assert_eq!(map_key_action(ElementState::Released, true), KeyAction::Repeat);
    }

    #[test]
    fn logical_key_named_uses_debug_form() {
        use winit::keyboard::{Key, NamedKey};
        assert_eq!(map_logical_key(&Key::Named(NamedKey::Tab)), KeyCode::new("Tab"));
        assert_eq!(map_logical_key(&Key::Named(NamedKey::Enter)), KeyCode::new("Enter"));
        assert_eq!(
            map_logical_key(&Key::Named(NamedKey::ArrowLeft)),
            KeyCode::new("ArrowLeft")
        );
        assert_eq!(
            map_logical_key(&Key::Named(NamedKey::Backspace)),
            KeyCode::new("Backspace")
        );
        assert_eq!(map_logical_key(&Key::Named(NamedKey::Space)), KeyCode::new("Space"));
    }

    #[test]
    fn logical_key_character_and_dead() {
        use winit::keyboard::{Key, SmolStr};
        assert_eq!(map_logical_key(&Key::Character(SmolStr::new("a"))), KeyCode::new("a"));
        assert_eq!(map_logical_key(&Key::Character(SmolStr::new("A"))), KeyCode::new("A"));
        assert_eq!(map_logical_key(&Key::Dead(Some('^'))), KeyCode::new("Dead(^)"));
        assert_eq!(map_logical_key(&Key::Dead(None)), KeyCode::new("Dead"));
    }

    #[test]
    fn logical_key_matches_browser_critical_contract() {
        // 浏览器输入（apps/browser）与 host-runtime 按**精确字符串**匹配这些命名键
        // （Escape 关菜单 / Space 滚屏 / Home·End·PageUp·PageDown 翻页 / 方向键 caret·scroll /
        // Backspace·Delete 编辑 / F1·F5 快捷键）。本测锁定 map_logical_key 的 Debug 形态产出
        // 这些串——`format!("{named:?}")` 是脆弱的跨 crate 契约（winit NamedKey Debug 若变，
        // 浏览器输入会静默失效）。任何不匹配 = 该键在浏览器已 silently broken。
        use winit::keyboard::{Key, NamedKey};
        let cases: &[(NamedKey, &str)] = &[
            (NamedKey::Tab, "Tab"),
            (NamedKey::Enter, "Enter"),
            (NamedKey::Space, "Space"),
            (NamedKey::Backspace, "Backspace"),
            (NamedKey::Escape, "Escape"),
            (NamedKey::Delete, "Delete"),
            (NamedKey::Home, "Home"),
            (NamedKey::End, "End"),
            (NamedKey::PageUp, "PageUp"),
            (NamedKey::PageDown, "PageDown"),
            (NamedKey::ArrowUp, "ArrowUp"),
            (NamedKey::ArrowDown, "ArrowDown"),
            (NamedKey::ArrowLeft, "ArrowLeft"),
            (NamedKey::ArrowRight, "ArrowRight"),
            (NamedKey::F1, "F1"),
            (NamedKey::F5, "F5"),
        ];
        for (named, expected) in cases.iter().copied() {
            assert_eq!(
                map_logical_key(&Key::Named(named)),
                KeyCode::new(expected),
                "NamedKey::{named:?} → 浏览器期望 \"{expected}\"，实际 {:?}",
                map_logical_key(&Key::Named(named))
            );
        }
        // 注：`Key::Unidentified(NativeKey)` 的 NativeKey 为 winit `pub(crate)`，仓外不可构造，
        // 故 Unidentified/Dead(None) 外的分支无法直接单测（与 map_key_event 同理，见其 docstring）。
    }

    #[test]
    fn key_text_filters_empty_and_none() {
        use winit::keyboard::SmolStr;
        assert_eq!(key_text(Some(&SmolStr::new("a"))), Some("a".to_string()));
        assert_eq!(key_text(Some(&SmolStr::new(""))), None);
        assert_eq!(key_text(None), None);
    }

    // ---- 指针 ----

    #[test]
    fn pointer_phase_from_element_state() {
        use winit::event::ElementState;
        assert_eq!(map_pointer_phase(ElementState::Pressed), PointerPhase::Pressed);
        assert_eq!(map_pointer_phase(ElementState::Released), PointerPhase::Released);
    }

    #[test]
    fn mouse_input_to_pointer_pressed_released() {
        let pos = Point::new(10.0, 20.0);
        let pressed = map_mouse_input(
            winit::event::MouseButton::Left,
            winit::event::ElementState::Pressed,
            pos,
            Modifiers::NONE,
        );
        match pressed {
            UiEvent::Pointer {
                phase,
                button,
                position,
                modifiers,
                pointer_id,
            } => {
                assert_eq!(phase, PointerPhase::Pressed);
                assert_eq!(button, Some(PointerButton::Primary));
                assert_eq!(position, pos);
                assert_eq!(modifiers, Modifiers::NONE);
                assert_eq!(pointer_id, 0, "鼠标 pointer_id 恒为 0");
            }
            _ => panic!("expected Pointer"),
        }

        let released = map_mouse_input(
            winit::event::MouseButton::Right,
            winit::event::ElementState::Released,
            pos,
            Modifiers::CONTROL,
        );
        if let UiEvent::Pointer {
            phase,
            button,
            modifiers,
            ..
        } = released
        {
            assert_eq!(phase, PointerPhase::Released);
            assert_eq!(button, Some(PointerButton::Secondary));
            assert!(modifiers.contains(Modifiers::CONTROL));
        } else {
            panic!("expected Pointer");
        }
    }

    #[test]
    fn cursor_moved_is_moved_no_button() {
        let ev = map_cursor_moved(Point::new(5.0, 6.0), Modifiers::SHIFT);
        match ev {
            UiEvent::Pointer {
                phase,
                button,
                position,
                modifiers,
                ..
            } => {
                assert_eq!(phase, PointerPhase::Moved);
                assert_eq!(button, None);
                assert_eq!(position, Point::new(5.0, 6.0));
                assert!(modifiers.contains(Modifiers::SHIFT));
            }
            _ => panic!("expected Pointer"),
        }
    }

    #[test]
    fn touch_phase_mapping() {
        use winit::event::TouchPhase;
        assert_eq!(map_touch_phase(TouchPhase::Started), PointerPhase::Pressed);
        assert_eq!(map_touch_phase(TouchPhase::Moved), PointerPhase::Moved);
        assert_eq!(map_touch_phase(TouchPhase::Ended), PointerPhase::Released);
        assert_eq!(map_touch_phase(TouchPhase::Cancelled), PointerPhase::Cancelled);
    }

    #[test]
    fn touch_started_is_primary_press_others_no_button() {
        let started = map_touch(
            winit::event::TouchPhase::Started,
            winit::dpi::PhysicalPosition::new(200.0, 100.0),
            7,
            2.0,
            Modifiers::NONE,
        );
        if let UiEvent::Pointer {
            phase,
            button,
            position,
            pointer_id,
            ..
        } = started
        {
            assert_eq!(phase, PointerPhase::Pressed);
            assert_eq!(button, Some(PointerButton::Primary));
            assert_eq!(position, Point::new(100.0, 50.0)); // 物理→逻辑
            assert_eq!(pointer_id, 7, "触摸 pointer_id 取自平台 touch id");
        } else {
            panic!("expected Pointer");
        }

        let moved = map_touch(
            winit::event::TouchPhase::Moved,
            winit::dpi::PhysicalPosition::new(220.0, 110.0),
            7,
            2.0,
            Modifiers::NONE,
        );
        if let UiEvent::Pointer { phase, button, .. } = moved {
            assert_eq!(phase, PointerPhase::Moved);
            assert_eq!(button, None);
        } else {
            panic!("expected Pointer");
        }
    }

    // ---- 滚轮 ----

    #[test]
    fn wheel_line_delta_negates_y_and_scales_lines() {
        // winit LineDelta(0, 1) = 向上滚一格 → UI delta.y = -20（向上）。
        let up = map_mouse_wheel(
            winit::event::MouseScrollDelta::LineDelta(0.0, 1.0),
            1.0,
            Point::ZERO,
            Modifiers::NONE,
        );
        if let UiEvent::Scroll { delta, .. } = up {
            assert_eq!(delta, Vec2::new(0.0, -LINE_HEIGHT_PX));
        } else {
            panic!("expected Scroll");
        }

        // 向下滚两格 → delta.y = +40。
        let down = map_mouse_wheel(
            winit::event::MouseScrollDelta::LineDelta(0.0, -2.0),
            1.0,
            Point::ZERO,
            Modifiers::NONE,
        );
        if let UiEvent::Scroll { delta, .. } = down {
            assert_eq!(delta, Vec2::new(0.0, 2.0 * LINE_HEIGHT_PX));
        } else {
            panic!("expected Scroll");
        }
    }

    #[test]
    fn wheel_pixel_delta_scales_and_negates_y() {
        // 物理 (0, 100) @ scale 2.0 → 逻辑 (0, 50)，y 取反 → (0, -50)。
        let ev = map_mouse_wheel(
            winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(0.0, 100.0)),
            2.0,
            Point::new(3.0, 4.0),
            Modifiers::NONE,
        );
        if let UiEvent::Scroll { delta, position, .. } = ev {
            assert_eq!(delta, Vec2::new(0.0, -50.0));
            assert_eq!(position, Point::new(3.0, 4.0));
        } else {
            panic!("expected Scroll");
        }
    }

    // ---- IME ----

    #[test]
    fn ime_commit_and_preedit_cursor_start() {
        let commit = map_ime(winit::event::Ime::Commit("hello".to_string()));
        assert!(matches!(
            commit,
            UiEvent::Ime(ImeEvent::Commit(ref t)) if t == "hello"
        ));

        // 选择区 (2, 5) → 单光标位 start=2。
        let preedit = map_ime(winit::event::Ime::Preedit("abc".to_string(), Some((2, 5))));
        match preedit {
            UiEvent::Ime(ImeEvent::Preedit { text, cursor }) => {
                assert_eq!(text, "abc");
                assert_eq!(cursor, Some(2));
            }
            _ => panic!("expected Ime Preedit"),
        }

        let preedit_none = map_ime(winit::event::Ime::Preedit("x".to_string(), None));
        if let UiEvent::Ime(ImeEvent::Preedit { cursor, .. }) = preedit_none {
            assert_eq!(cursor, None);
        } else {
            panic!("expected Ime Preedit");
        }

        assert!(matches!(
            map_ime(winit::event::Ime::Enabled),
            UiEvent::Ime(ImeEvent::Enabled)
        ));
        assert!(matches!(
            map_ime(winit::event::Ime::Disabled),
            UiEvent::Ime(ImeEvent::Disabled)
        ));
    }
}

/// adapter → `WidgetHost` retained 运行态的端到端契约（DC-2「接 winit 事件循环」核心证据）。
///
/// 验证 winit 原始事件经本模块映射为 `UiEvent` 后，能正确驱动 `ui-runtime::WidgetHost`
/// 的指针命中 / click-to-focus / emit 闭环——即 adapter↔runtime 的 phase/button/位置约定
/// 端到端正确（任何映射符号/相位错误都会让 host 不命中、不聚焦、不 emit）。
#[cfg(test)]
mod adapter_runtime_integration {
    use super::*;
    use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
    use zero_ui_core::geometry::{Constraints, Size};
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::widget::{
        EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, UpdateCtx, Widget, WidgetId, WidgetSpec,
    };
    use zero_ui_runtime::{EmittedAction, UiApp, WidgetHost};

    /// 可聚焦、点击 emit `app.click` 的最小叶子控件（仅供本集成测试）。
    #[derive(Default)]
    struct ClickBox;

    impl Widget for ClickBox {
        fn mount(&mut self, _ctx: &mut MountCtx) {}
        fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
        fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
            if let UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            } = event
            {
                EventResult::Emit(ActionId::new("app.click"))
            } else {
                EventResult::Ignored
            }
        }
        fn layout(&mut self, _ctx: &mut LayoutCtx, _c: Constraints) -> Size {
            Size::new(100.0, 40.0)
        }
        fn paint(&mut self, _ctx: &mut PaintCtx) {}
        fn focusable(&self) -> bool {
            true
        }
    }

    /// 注册 ClickBox 工厂、建根树、布局（ClickBox 落在 (0,0)–(100,40)）。
    fn host_with_clickbox() -> WidgetHost {
        let mut host = WidgetHost::new();
        host.register("ClickBox", |_spec| Box::new(ClickBox));
        let mut spec = WidgetSpec::new("ClickBox");
        spec.id = Some(WidgetId::new("click"));
        host.set_root(&spec);
        host.layout(Constraints::loose(Size::new(800.0, 600.0)));
        host
    }

    #[test]
    fn winit_left_click_drives_host_focus_and_emit() {
        let mut host = host_with_clickbox();
        // 模拟 winit MouseInput：左键按下在 ClickBox 中心 (50, 20)。
        let ui_event = map_mouse_input(
            winit::event::MouseButton::Left,
            winit::event::ElementState::Pressed,
            Point::new(50.0, 20.0),
            Modifiers::NONE,
        );
        let emitted: Vec<EmittedAction> = host.dispatch_event(&ui_event);
        // adapter 映射的事件被 host 当作点击：emit app.click + click-to-focus 命中点最深 focusable。
        assert!(
            emitted.iter().any(|e| e.action == ActionId::new("app.click")),
            "winit 左键点击应驱动 host emit app.click，got {emitted:?}"
        );
        assert_eq!(
            host.focused_id(),
            Some(&WidgetId::new("click")),
            "点击应聚焦命中点的 ClickBox"
        );
    }

    #[test]
    fn winit_cursor_move_does_not_emit() {
        let mut host = host_with_clickbox();
        let ui_event = map_cursor_moved(Point::new(50.0, 20.0), Modifiers::NONE);
        let emitted = host.dispatch_event(&ui_event);
        assert!(emitted.is_empty(), "光标移动不应触发 emit，got {emitted:?}");
    }

    #[test]
    fn winit_click_outside_widget_no_emit() {
        let mut host = host_with_clickbox();
        // 点击 ClickBox 之外 (500, 500)。
        let ui_event = map_mouse_input(
            winit::event::MouseButton::Left,
            winit::event::ElementState::Pressed,
            Point::new(500.0, 500.0),
            Modifiers::NONE,
        );
        let emitted = host.dispatch_event(&ui_event);
        assert!(emitted.is_empty(), "widget 外点击不应 emit，got {emitted:?}");
    }

    /// 计数 "app.click" 的最小 UiApp（证明 driver → reducer 路径）。
    struct ClickApp {
        clicks: u32,
    }

    impl ClickApp {
        fn new() -> ClickApp {
            ClickApp { clicks: 0 }
        }
        fn clicks(&self) -> u32 {
            self.clicks
        }
    }

    impl UiApp for ClickApp {
        fn root_spec(&self) -> WidgetSpec {
            let mut spec = WidgetSpec::new("ClickBox");
            spec.id = Some(WidgetId::new("click"));
            spec
        }
        fn dispatch(&mut self, action: &ActionId, _payload: Option<ActionPayload>) -> ActionResult {
            if action.0.as_str() == "app.click" {
                self.clicks += 1;
                ActionResult::Handled
            } else {
                ActionResult::UnknownAction(action.clone())
            }
        }
    }

    #[test]
    fn winit_raw_event_drives_driver_reducer_and_rebuild() {
        // DC-2 真实 EventLoop::run 的 per-event 契约：winit 原始 MouseInput → event_map 映射 →
        // WinitDriver.pump_event → host dispatch（ClickBox emit "app.click"）→ app.dispatch reducer
        // （Handled）→ driver 重建声明树。串联 winit-raw → driver → reducer 这条路径（既有
        // event_map 测止于 host.dispatch_event；driver 测用合成 UiEvent；本测把二者合一）。
        let mut app = ClickApp::new();
        {
            let mut driver = crate::WinitDriver::new(&mut app, WindowMetrics::desktop());
            driver.host_mut().register("ClickBox", |_spec| Box::new(ClickBox));
            driver.begin();
            // 模拟 winit MouseInput：左键按下在 ClickBox 中心 (50, 20)。
            let ui_event = map_mouse_input(
                winit::event::MouseButton::Left,
                winit::event::ElementState::Pressed,
                Point::new(50.0, 20.0),
                Modifiers::NONE,
            );
            let out = driver.pump_event(&ui_event);
            // driver 内部 dispatch_event → ClickBox emit "app.click" → app.dispatch Handled → 重建 spec。
            assert_eq!(out.emitted_actions, 1, "winit 点击经 driver 派发出 1 action");
            assert!(out.spec_rebuilt, "Handled → driver 重建声明树");
            assert!(out.needs_redraw, "重建 → 需要重绘");
            driver.pump_frame(); // 落盘（按 invalidation 重绘）。
        }
        // driver 释放 &mut app 后读最终状态：reducer 被 winit 原始事件经 driver 真正驱动。
        assert_eq!(app.clicks(), 1, "winit 原始事件经 driver 驱动了应用 reducer");
    }
}
