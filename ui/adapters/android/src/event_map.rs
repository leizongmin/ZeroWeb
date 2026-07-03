//! Android 原始事件 → [`UiEvent`] 纯函数转换层（spec IF-006 / FR-010）。
//!
//! 本模块是 Android-specific 类型的**唯一**落点——上游 Kotlin/Java Activity 通过 `ffi` 模块
//! 把 Android `MotionEvent` / `KeyEvent` / `WindowInsets` 数据（数值形态）传入这些函数；
//! 产出一律为浏览器无关的 [`zero_ui_core::event::UiEvent`]。
//!
//! 模拟 winit adapter `event_map.rs` 和 harmonyos adapter 的 `event_map.rs` 模式：
//! 纯函数、无状态、headless 可测。

use zero_ui_core::event::{KeyAction, KeyCode, Modifiers, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Insets, Point, Size};
use zero_ui_core::layout::{DEFAULT_DENSITY, Orientation, WindowMetrics};
use zero_ui_core::theme::SystemThemeSnapshot;

// ---------------------------------------------------------------------------
// Android 触摸动作 → PointerPhase
// ---------------------------------------------------------------------------

/// 把 Android `MotionEvent.ACTION_*` 数字转为 [`PointerPhase`]。
///
/// Android `MotionEvent` action masked 值：
/// - `0` = `ACTION_DOWN` → [`PointerPhase::Pressed`]
/// - `1` = `ACTION_UP` → [`PointerPhase::Released`]
/// - `2` = `ACTION_MOVE` → [`PointerPhase::Moved`]
/// - `3` = `ACTION_CANCEL` → [`PointerPhase::Cancelled`]
/// - `5` = `ACTION_POINTER_DOWN`（多指按下，非主指）→ [`PointerPhase::Pressed`]
/// - `6` = `ACTION_POINTER_UP`（多指抬起，非主指）→ [`PointerPhase::Released`]
///
/// 未知值安全回落为 `Cancelled`（不 panic）。
pub fn android_touch_action_to_pointer_phase(action: u32) -> PointerPhase {
    // Android action 编码：低 8 位是动作类型；`ACTION_MASK = 0xff`。
    let kind = action & 0xff;
    match kind {
        0 => PointerPhase::Pressed,   // ACTION_DOWN
        1 => PointerPhase::Released,  // ACTION_UP
        2 => PointerPhase::Moved,     // ACTION_MOVE
        5 => PointerPhase::Pressed,   // ACTION_POINTER_DOWN
        6 => PointerPhase::Released,  // ACTION_POINTER_UP
        _ => PointerPhase::Cancelled, // ACTION_CANCEL(3) + unknown
    }
}

// ---------------------------------------------------------------------------
// 触摸事件映射
// ---------------------------------------------------------------------------

/// 把 Android 单个触摸点转为 [`UiEvent::Pointer`]。
///
/// `pointer_id` = Android `MotionEvent.getPointerId(i)`；
/// `(x, y)` = 窗口坐标（`MotionEvent.getX(i)` / `getY(i)`）；
/// `action` = `MotionEvent.getActionMasked()`（主指动作）或 `getAction()`（含 pointer index）。
///
/// 多指场景：调用方对 `MotionEvent` 的每个 pointer 分别调本函数，pointer_id 带入。
pub fn map_touch_event(pointer_id: u32, x: f32, y: f32, action: u32) -> UiEvent {
    let phase = android_touch_action_to_pointer_phase(action);
    UiEvent::Pointer {
        phase,
        button: None,
        position: Point::new(x, y),
        modifiers: Modifiers::NONE,
        pointer_id,
    }
}

/// 把 Android 多指触摸批量映射（Kotlin `MotionEvent` 遍历 → Vec<UiEvent>）。
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

/// 把 Android 按键转为 [`UiEvent::Key`]。
///
/// `key_name` = Android `KeyEvent.keyCodeToString(keyCode)` 或按键名称
/// （如 `"KEYCODE_TAB"`/`"KEYCODE_ENTER"`/`"KEYCODE_BACK"`/`"KEYCODE_DPAD_LEFT"` 等）；
/// `is_down` = 按下/抬起。
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

/// Android 窗口度量输入（供 FFI 调用方捆扎参数，避免 10 参数 flat 函数）。
///
/// 参数对齐 Android `DisplayMetrics` + `WindowInsets`（API 30+）：
/// - `viewport_w/h` = 窗口逻辑像素（`Configuration.screenWidthDp` × density）
/// - `density` = `DisplayMetrics.density`（dp→px 缩放，非 DPI bucket）
/// - `text_scale` = `Configuration.fontScale`
/// - `safe_*` = `WindowInsets.getInsets(Type.systemBars())` —— status bar / navigation bar / cutout
/// - `keyboard_rect` = 软键盘窗口坐标 `(x, y, w, h)`（None = 键盘隐藏）
/// - `is_portrait` = 竖屏（`resources.configuration.orientation == ORIENTATION_PORTRAIT`）
#[derive(Debug, Clone, Copy)]
pub struct AndroidWindowMetricsInput {
    pub viewport_w: f32,
    pub viewport_h: f32,
    /// 平台 DPI 比（HiDPI 设备像素比，Android `displayMetrics.density`，如 phone 3.0）。
    /// 映射到 [`WindowMetrics::scale_factor`]；**非** Material 间距密度（后者恒为 `DEFAULT_DENSITY`）。
    pub density: f32,
    pub text_scale: f32,
    pub safe_top: f32,
    pub safe_right: f32,
    pub safe_bottom: f32,
    pub safe_left: f32,
    pub keyboard_rect: Option<(f32, f32, f32, f32)>,
    pub is_portrait: bool,
}

/// 把 Android 窗口度量捆扎值转为 [`WindowMetrics`]。
pub fn map_window_metrics(input: AndroidWindowMetricsInput) -> WindowMetrics {
    let logical_size = Size::new(input.viewport_w.max(1.0), input.viewport_h.max(1.0));
    let orientation = if input.is_portrait {
        Orientation::Portrait
    } else {
        Orientation::Landscape
    };
    // 软键盘 insets：仅取底部高度（Android 键盘通常全宽贴合底部）。
    let keyboard_insets = input.keyboard_rect.map_or(Insets::all(0.0), |(_x, _y, _w, h)| Insets {
        top: 0.0,
        right: 0.0,
        bottom: h,
        left: 0.0,
    });

    WindowMetrics {
        logical_size,
        // `input.density` = 平台 DPI 比（HiDPI，Android `displayMetrics.density`，如 phone 3.0）
        // → scale_factor（设备像素比）。
        scale_factor: input.density.max(0.0),
        safe_area: Insets {
            top: input.safe_top,
            right: input.safe_right,
            bottom: input.safe_bottom,
            left: input.safe_left,
        },
        keyboard_insets,
        text_scale: input.text_scale.max(0.0),
        // **density = Material 间距密度（compact/comfortable），非 DPI 比**（DC-12 决策：
        // density 与 scale_factor 正交；DEFAULT_DENSITY=1.0）。Android 无独立「间距密度」API，
        // 故回落默认 1.0——此前误把 DPI 比塞进 density 会 3× 放大所有 spacing token（phone 上布局爆）。
        density: DEFAULT_DENSITY,
        orientation,
    }
}

// ---------------------------------------------------------------------------
// 软键盘变化 → UiEvent（触发布局刷新）
// ---------------------------------------------------------------------------

/// 软键盘显示/隐藏变化 → 布局刷新事件。
///
/// Android `ViewTreeObserver.OnGlobalLayoutListener` 或 `WindowInsetsAnimation.Callback`
/// 回调中，先调用 `map_window_metrics` 更新 `WindowMetrics.keyboard_insets`，再调本函数
/// 产生一帧布局事件供 `WidgetHost` 标记 `needs_layout`。
pub fn map_soft_keyboard(is_visible: bool, keyboard_height: f32) -> UiEvent {
    UiEvent::Key {
        code: KeyCode::new(if is_visible {
            "android.keyboard.show"
        } else {
            "android.keyboard.hide"
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

/// 平台 back 动作（Android 返回键或手势导航返回）→ [`UiEvent`]。
///
/// 宿主在 `Activity.onBackPressed()` 或 `OnBackPressedDispatcher.addCallback` 中调用；
/// handler 栈（`BackNavigationService`）仲裁消费：`Handled` → 阻止系统默认 back；
/// `DefaultBack` → `Activity.finish()` 或 `Navigator.pop`。
pub fn map_back_gesture() -> UiEvent {
    UiEvent::Key {
        code: KeyCode::new("android.back_gesture"),
        action: KeyAction::Pressed,
        modifiers: Modifiers::NONE,
        text: None,
    }
}

// ---------------------------------------------------------------------------
// 系统主题快照
// ---------------------------------------------------------------------------

/// 构造系统主题快照（Android `UiModeManager` 夜间模式标志）。
///
/// Android `Configuration.uiMode & UI_MODE_NIGHT_MASK`：
/// - `UI_MODE_NIGHT_YES` → dark = true
/// - `UI_MODE_NIGHT_NO` / `UI_MODE_NIGHT_UNDEFINED` → dark = false
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
    fn android_touch_action_maps_correctly() {
        assert_eq!(android_touch_action_to_pointer_phase(0), PointerPhase::Pressed);
        assert_eq!(android_touch_action_to_pointer_phase(1), PointerPhase::Released);
        assert_eq!(android_touch_action_to_pointer_phase(2), PointerPhase::Moved);
        assert_eq!(android_touch_action_to_pointer_phase(3), PointerPhase::Cancelled);
        // POINTER_DOWN(5)/UP(6) → same as primary
        assert_eq!(android_touch_action_to_pointer_phase(5), PointerPhase::Pressed);
        assert_eq!(android_touch_action_to_pointer_phase(6), PointerPhase::Released);
        assert_eq!(android_touch_action_to_pointer_phase(99), PointerPhase::Cancelled);
    }

    #[test]
    fn map_touch_event_down_move_up_cancel() {
        let down = map_touch_event(0, 100.0, 200.0, 0);
        assert!(matches!(
            down,
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            }
        ));

        let mv = map_touch_event(0, 150.0, 250.0, 2);
        assert!(matches!(
            mv,
            UiEvent::Pointer {
                phase: PointerPhase::Moved,
                ..
            }
        ));

        let up = map_touch_event(0, 160.0, 260.0, 1);
        assert!(matches!(
            up,
            UiEvent::Pointer {
                phase: PointerPhase::Released,
                ..
            }
        ));
    }

    #[test]
    fn map_touch_event_multi_pointer() {
        let touches = [(0, 10.0, 20.0, 0), (1, 30.0, 40.0, 5)];
        let events = map_touch_events(&touches);
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0],
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                pointer_id: 0,
                ..
            }
        ));
        assert!(matches!(
            events[1],
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                pointer_id: 1,
                ..
            }
        ));
    }

    #[test]
    fn map_key_event_down_up() {
        let down = map_key_event("KEYCODE_ENTER", true);
        let up = map_key_event("KEYCODE_ENTER", false);
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
    fn map_key_event_android_specific_keys() {
        let back = map_key_event("KEYCODE_BACK", true);
        let home = map_key_event("KEYCODE_HOME", true);
        assert!(matches!(back, UiEvent::Key { .. }));
        assert!(matches!(home, UiEvent::Key { .. }));
    }

    #[test]
    fn map_window_metrics_phone_portrait() {
        let m = map_window_metrics(AndroidWindowMetricsInput {
            viewport_w: 412.0,
            viewport_h: 915.0,
            density: 2.625,
            text_scale: 1.0,
            safe_top: 24.0,
            safe_right: 0.0,
            safe_bottom: 48.0,
            safe_left: 0.0,
            keyboard_rect: None,
            is_portrait: true,
        });
        assert_eq!(m.logical_size, Size::new(412.0, 915.0));
        assert_eq!(m.scale_factor, 2.625, "DPI 比 → scale_factor");
        assert_eq!(m.density, DEFAULT_DENSITY, "Material 间距密度恒为默认，非 DPI 比");
        assert_eq!(m.text_scale, 1.0);
        assert_eq!(m.safe_area.top, 24.0);
        assert_eq!(m.safe_area.bottom, 48.0);
        assert_eq!(m.orientation, Orientation::Portrait);
        assert_eq!(ViewportClass::from_width(412.0), ViewportClass::Compact);
        assert_eq!(m.keyboard_insets, Insets::all(0.0));
    }

    #[test]
    fn map_window_metrics_tablet_landscape_with_keyboard() {
        let m = map_window_metrics(AndroidWindowMetricsInput {
            viewport_w: 1024.0,
            viewport_h: 768.0,
            density: 2.0,
            text_scale: 1.15,
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
        assert_eq!(m.scale_factor, 2.0, "DPI 比 → scale_factor");
        assert_eq!(m.density, DEFAULT_DENSITY, "Material 间距密度恒为默认");
        assert_eq!(m.text_scale, 1.15);
        assert!(m.keyboard_insets.bottom > 0.0);
    }

    #[test]
    fn map_window_metrics_defensive_clamps() {
        let m = map_window_metrics(AndroidWindowMetricsInput {
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
        assert_eq!(m.density, DEFAULT_DENSITY, "density 恒为 Material 默认（不随 DPI 比）");
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
