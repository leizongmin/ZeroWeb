//! OHOS 原始事件 → [`UiEvent`] 纯函数转换层（spec IF-006 / FR-010）。
//!
//! 本模块是 OHOS-specific 类型的**唯一**落点——上游 ArkTS 通过 `ffi` 模块把 OHOS 触摸/键盘
//! /IME/window 数据（数值形态）传入这些函数；产出一律为浏览器无关的 [`zero_ui_core::event::UiEvent`]。
//!
//! 模拟 winit adapter `event_map.rs` 模式：纯函数、无状态、headless 可测。

use zero_ui_core::event::{KeyAction, KeyCode, Modifiers, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Insets, Point, Size};
use zero_ui_core::layout::{Orientation, WindowMetrics};
use zero_ui_core::theme::SystemThemeSnapshot;

// ---------------------------------------------------------------------------
// OHOS 触摸阶段 → PointerPhase
// ---------------------------------------------------------------------------

/// 把 OHOS `TouchAction` 数字转为 [`PointerPhase`]。
///
/// OHOS `Action` 枚举值（`@kit.InputKit`）：
/// - `0` = CANCEL → [`PointerPhase::Cancelled`]
/// - `1` = DOWN → [`PointerPhase::Pressed`]
/// - `2` = MOVE → [`PointerPhase::Moved`]
/// - `3` = UP → [`PointerPhase::Released`]
///
/// 未知值安全回落为 `Cancelled`（不 panic）。
pub fn ohos_touch_phase_to_pointer_phase(action: u32) -> PointerPhase {
    match action {
        1 => PointerPhase::Pressed,
        2 => PointerPhase::Moved,
        3 => PointerPhase::Released,
        _ => PointerPhase::Cancelled,
    }
}

// ---------------------------------------------------------------------------
// 触摸事件映射
// ---------------------------------------------------------------------------

/// 把 OHOS 单个触摸点转为 [`UiEvent::Pointer`]。
///
/// `touch_id` = OHOS `Touch.id`；`(x, y)` = 窗口坐标（OHOS `windowX`/`windowY`）；
/// `action` = OHOS `TouchAction` 数值。
pub fn map_touch_event(touch_id: u32, x: f32, y: f32, action: u32) -> UiEvent {
    let phase = ohos_touch_phase_to_pointer_phase(action);
    UiEvent::Pointer {
        phase,
        button: None,
        position: Point::new(x, y),
        modifiers: Modifiers::NONE,
        pointer_id: touch_id,
    }
}

/// 把 OHOS 多指触摸批量映射（ArkTS `TouchEvent.touches` → Vec<UiEvent>）。
pub fn map_touch_events(touches: &[(u32, f32, f32, u32)]) -> Vec<UiEvent> {
    touches
        .iter()
        .map(|&(id, x, y, action)| map_touch_event(id, x, y, action))
        .collect()
}

// ---------------------------------------------------------------------------
// 键盘事件映射
// ---------------------------------------------------------------------------

/// 把 bool 转为 KeyAction。
pub fn map_key_action(is_down: bool) -> KeyAction {
    if is_down {
        KeyAction::Pressed
    } else {
        KeyAction::Released
    }
}

/// 把 OHOS 按键名称转为 [`UiEvent::Key`]。
///
/// `key_name` = OHOS `KeyEvent.keyText` 或 `KeyCode` 的名称（如 `"Tab"`/`"Enter"`/
/// `"Back"`/`"ArrowLeft"` 等）；`is_down` = 按下/抬起。
pub fn map_key_event(key_name: &str, is_down: bool) -> UiEvent {
    UiEvent::Key {
        code: KeyCode::new(key_name),
        action: if is_down {
            KeyAction::Pressed
        } else {
            KeyAction::Released
        },
        modifiers: Modifiers::NONE,
        text: None,
    }
}

// ---------------------------------------------------------------------------
// 窗口度量映射
// ---------------------------------------------------------------------------

/// OHOS 窗口度量输入（供 FFI 调用方捆扎参数，避免 10 参数 flat 函数）。
#[derive(Debug, Clone, Copy)]
pub struct OhosWindowMetricsInput {
    pub viewport_w: f32,
    pub viewport_h: f32,
    pub density: f32,
    pub text_scale: f32,
    pub safe_top: f32,
    pub safe_right: f32,
    pub safe_bottom: f32,
    pub safe_left: f32,
    pub keyboard_rect: Option<(f32, f32, f32, f32)>,
    pub is_portrait: bool,
}

/// 把 OHOS 窗口度量捆扎值转为 [`WindowMetrics`]。
pub fn map_window_metrics(input: OhosWindowMetricsInput) -> WindowMetrics {
    let logical_size = Size::new(input.viewport_w.max(1.0), input.viewport_h.max(1.0));
    let orientation = if input.is_portrait {
        Orientation::Portrait
    } else {
        Orientation::Landscape
    };
    let keyboard_insets = input.keyboard_rect.map_or(Insets::all(0.0), |(_x, _y, _w, h)| Insets {
        top: 0.0,
        right: 0.0,
        bottom: h,
        left: 0.0,
    });

    WindowMetrics {
        logical_size,
        scale_factor: input.density.max(0.0),
        safe_area: Insets {
            top: input.safe_top,
            right: input.safe_right,
            bottom: input.safe_bottom,
            left: input.safe_left,
        },
        keyboard_insets,
        text_scale: input.text_scale.max(0.0),
        density: input.density.max(0.0),
        orientation,
    }
}

// ---------------------------------------------------------------------------
// 软键盘变化 → UiEvent（触发布局刷新）
// ---------------------------------------------------------------------------

/// 软键盘显示/隐藏变化 → 布局刷新事件。
///
/// OHOS `inputMethod.on('keyboardShow'|'keyboardHide')` 回调中，先调用
/// `map_window_metrics` 更新 `WindowMetrics.keyboard_insets`，再调本函数产生
/// 一帧布局事件供 `WidgetHost` 标记 `needs_layout`。
pub fn map_soft_keyboard(is_visible: bool, keyboard_height: f32) -> UiEvent {
    UiEvent::Key {
        code: KeyCode::new(if is_visible {
            "ohos.keyboard.show"
        } else {
            "ohos.keyboard.hide"
        }),
        action: if is_visible {
            KeyAction::Released
        } else {
            KeyAction::Pressed
        },
        modifiers: Modifiers::NONE,
        text: Some(format!("h={}", keyboard_height as u32)),
    }
}

// ---------------------------------------------------------------------------
// 平台 back gesture 映射
// ---------------------------------------------------------------------------

/// 平台 back gesture（OHOS 返回手势/系统返回键）→ [`UiEvent`]。
///
/// 宿主在 ArkTS `onBackPress` 回调中调用；handler 栈（`BackNavigationService`）
/// 仲裁消费：`Handled` → 阻止系统默认 back；`DefaultBack` → `Navigator.pop` 或退出。
pub fn map_back_gesture() -> UiEvent {
    UiEvent::Key {
        code: KeyCode::new("ohos.back_gesture"),
        action: KeyAction::Pressed,
        modifiers: Modifiers::NONE,
        text: None,
    }
}

// ---------------------------------------------------------------------------
// 系统主题快照
// ---------------------------------------------------------------------------

/// 构造系统主题快照（HarmonyOS `darkMode` 标志）。
pub fn system_theme_from_dark_mode(dark: bool, high_contrast: bool) -> SystemThemeSnapshot {
    use zero_ui_core::theme::ResolvedColorScheme;
    SystemThemeSnapshot {
        system_scheme: if dark {
            ResolvedColorScheme::Dark
        } else {
            ResolvedColorScheme::Light
        },
        high_contrast,
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::layout::ViewportClass;

    #[test]
    fn ohos_touch_phase_maps_correctly() {
        assert_eq!(ohos_touch_phase_to_pointer_phase(0), PointerPhase::Cancelled);
        assert_eq!(ohos_touch_phase_to_pointer_phase(1), PointerPhase::Pressed);
        assert_eq!(ohos_touch_phase_to_pointer_phase(2), PointerPhase::Moved);
        assert_eq!(ohos_touch_phase_to_pointer_phase(3), PointerPhase::Released);
        assert_eq!(ohos_touch_phase_to_pointer_phase(99), PointerPhase::Cancelled);
    }

    #[test]
    fn map_touch_event_pressed() {
        let ev = map_touch_event(0, 100.0, 200.0, 1);
        match ev {
            UiEvent::Pointer {
                phase,
                position,
                pointer_id,
                ..
            } => {
                assert_eq!(phase, PointerPhase::Pressed);
                assert_eq!(position, Point::new(100.0, 200.0));
                assert_eq!(pointer_id, 0);
            }
            _ => panic!("expected Pointer"),
        }
    }

    #[test]
    fn map_touch_event_moved_and_released() {
        assert_eq!(
            map_touch_event(0, 150.0, 250.0, 2),
            UiEvent::Pointer {
                phase: PointerPhase::Moved,
                button: None,
                position: Point::new(150.0, 250.0),
                modifiers: Modifiers::NONE,
                pointer_id: 0,
            }
        );
        assert_eq!(
            map_touch_event(0, 160.0, 260.0, 3),
            UiEvent::Pointer {
                phase: PointerPhase::Released,
                button: None,
                position: Point::new(160.0, 260.0),
                modifiers: Modifiers::NONE,
                pointer_id: 0,
            }
        );
    }

    #[test]
    fn map_touch_events_multi() {
        let touches = [(0, 10.0, 20.0, 1), (1, 30.0, 40.0, 2)];
        let events = map_touch_events(&touches);
        assert_eq!(events.len(), 2);
        // Verify by matching individual fields
        assert!(matches!(
            events[0],
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            }
        ));
        assert!(matches!(
            events[1],
            UiEvent::Pointer {
                phase: PointerPhase::Moved,
                ..
            }
        ));
    }

    #[test]
    fn map_key_event_down_up() {
        let down = map_key_event("Enter", true);
        let up = map_key_event("Enter", false);
        assert!(matches!(
            down,
            UiEvent::Key {
                action: KeyAction::Pressed,
                ..
            }
        ));
        assert!(matches!(
            up,
            UiEvent::Key {
                action: KeyAction::Released,
                ..
            }
        ));
    }

    #[test]
    fn map_key_event_tab_and_back() {
        let tab = map_key_event("Tab", true);
        let back = map_key_event("Back", true);
        assert!(matches!(tab, UiEvent::Key { .. }));
        assert!(matches!(back, UiEvent::Key { .. }));
    }

    #[test]
    fn map_window_metrics_phone_portrait() {
        let m = map_window_metrics(OhosWindowMetricsInput {
            viewport_w: 390.0,
            viewport_h: 844.0,
            density: 3.0,
            text_scale: 1.0,
            safe_top: 47.0,
            safe_right: 0.0,
            safe_bottom: 34.0,
            safe_left: 0.0,
            keyboard_rect: None,
            is_portrait: true,
        });
        assert_eq!(m.logical_size, Size::new(390.0, 844.0));
        assert_eq!(m.density, 3.0);
        assert_eq!(m.text_scale, 1.0);
        assert_eq!(m.safe_area.top, 47.0);
        assert_eq!(m.safe_area.bottom, 34.0);
        assert_eq!(m.orientation, Orientation::Portrait);
        assert_eq!(ViewportClass::from_width(390.0), ViewportClass::Compact);
        assert_eq!(m.keyboard_insets, Insets::all(0.0));
    }

    #[test]
    fn map_window_metrics_tablet_landscape_with_keyboard() {
        let m = map_window_metrics(OhosWindowMetricsInput {
            viewport_w: 1024.0,
            viewport_h: 768.0,
            density: 2.0,
            text_scale: 1.25,
            safe_top: 0.0,
            safe_right: 0.0,
            safe_bottom: 0.0,
            safe_left: 0.0,
            keyboard_rect: Some((0.0, 468.0, 1024.0, 300.0)),
            is_portrait: false,
        });
        assert_eq!(m.logical_size, Size::new(1024.0, 768.0));
        assert_eq!(m.orientation, Orientation::Landscape);
        assert_eq!(ViewportClass::from_width(1024.0), ViewportClass::Expanded);
        assert_eq!(m.density, 2.0);
        assert_eq!(m.text_scale, 1.25);
        assert!(m.keyboard_insets.bottom > 0.0);
    }

    #[test]
    fn map_window_metrics_defensive_clamps() {
        let m = map_window_metrics(OhosWindowMetricsInput {
            viewport_w: -10.0,
            viewport_h: -10.0,
            density: -0.5,
            text_scale: 0.0,
            safe_top: 0.0,
            safe_right: 0.0,
            safe_bottom: 0.0,
            safe_left: 0.0,
            keyboard_rect: None,
            is_portrait: true,
        });
        assert_eq!(m.logical_size, Size::new(1.0, 1.0));
        assert_eq!(m.scale_factor, 0.0);
        assert_eq!(m.density, 0.0);
        assert_eq!(m.text_scale, 0.0);
    }

    #[test]
    fn map_soft_keyboard_visible_and_hidden() {
        let show = map_soft_keyboard(true, 300.0);
        let hide = map_soft_keyboard(false, 0.0);
        assert!(matches!(show, UiEvent::Key { .. }));
        assert!(matches!(hide, UiEvent::Key { .. }));
    }

    #[test]
    fn map_back_gesture_produces_key_event() {
        let ev = map_back_gesture();
        assert!(matches!(ev, UiEvent::Key { .. }));
    }

    #[test]
    fn system_theme_from_dark_mode_light_and_dark() {
        use zero_ui_core::theme::ResolvedColorScheme;
        let light = system_theme_from_dark_mode(false, false);
        assert_eq!(light.system_scheme, ResolvedColorScheme::Light);
        assert!(!light.high_contrast);
        let dark = system_theme_from_dark_mode(true, true);
        assert_eq!(dark.system_scheme, ResolvedColorScheme::Dark);
        assert!(dark.high_contrast);
    }
}
