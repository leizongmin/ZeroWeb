//! C ABI 导出层（ArkTS ↔ Rust 桥接）。
//!
//! 通过 `std::sync::Mutex<Option<Rc<RefCell<RuntimeInner>>>>` 持有运行时引用，
//! ArkTS 侧通过 N-API/NAPI 调用 `#[unsafe(no_mangle)] unsafe extern "C"` 函数喂入 OHOS 事件。
//!
//! M4 skeleton：headless 可测。真实首帧需 DevEco Studio + Device/Emulator。

use crate::event_map;
use crate::runtime::RuntimeInner;
use core::cell::RefCell;

/// 全局运行时引用（FFI 单线程模型：ArkTS 主线程驱动所有调用）。
///
/// 使用 raw pointer 避免 `Send + Sync` 约束；生命周期由宿主管理。
static mut HARMONYOS_RT: Option<&RefCell<RuntimeInner>> = None;

/// 初始化全局运行时（由宿主调用一次；`inner` 必须为 `&'static` 或由宿主保证生命周期覆盖 FFI 调用期）。
///
/// # Safety
/// 调用方保证 `inner` 的生命周期覆盖所有 FFI 调用，且只在 ArkTS 主线程调用。
pub unsafe fn init_runtime(inner: &'static RefCell<RuntimeInner>) {
    unsafe {
        HARMONYOS_RT = Some(inner);
    }
}

fn with_rt<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&RefCell<RuntimeInner>) -> R,
{
    // Safety: caller guarantees single-threaded access (ArkTS main thread).
    // HARMONYOS_RT is initialized before any FFI call.
    unsafe { HARMONYOS_RT.map(f) }
}

// ---------------------------------------------------------------------------
// Raw 事件类型
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct HarmonyOSSurface {
    pub surface_id: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct RawHarmonyOSEvent {
    pub kind: u32,
    pub arg0: f32,
    pub arg1: f32,
    pub arg2: f32,
    pub arg3: u32,
}

impl RawHarmonyOSEvent {
    pub const KIND_TOUCH: u32 = 1;
    pub const KIND_KEY: u32 = 2;
    pub const KIND_WINDOW: u32 = 3;
    pub const KIND_BACK: u32 = 4;
    pub const KIND_IME: u32 = 5;

    pub fn to_ui_event(&self) -> zero_ui_core::event::UiEvent {
        match self.kind {
            Self::KIND_TOUCH => event_map::map_touch_event(self.arg0 as u32, self.arg1, self.arg2, self.arg3),
            Self::KIND_KEY => zero_ui_core::event::UiEvent::Key {
                code: zero_ui_core::event::KeyCode::new(&format!("ohos.key.{}", self.arg0 as u32)),
                action: event_map::map_key_action(self.arg1 > 0.0),
                modifiers: zero_ui_core::event::Modifiers::NONE,
                text: None,
            },
            Self::KIND_BACK => event_map::map_back_gesture(),
            _ => event_map::map_back_gesture(),
        }
    }
}

// ---------------------------------------------------------------------------
// FFI 函数
// ---------------------------------------------------------------------------

/// Notify the runtime of window size/density/orientation changes.
///
/// # Safety
/// Caller must ensure this is called on the ArkTS main thread after `init_runtime`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn harmonyos_window_size_change(
    width: f32,
    height: f32,
    density: f32,
    text_scale: f32,
    safe_top: f32,
    safe_right: f32,
    safe_bottom: f32,
    safe_left: f32,
    is_portrait: u32,
) {
    with_rt(|rt| {
        let mut inner = rt.borrow_mut();
        inner.metrics = event_map::map_window_metrics(event_map::OhosWindowMetricsInput {
            viewport_w: width,
            viewport_h: height,
            density,
            text_scale,
            safe_top,
            safe_right,
            safe_bottom,
            safe_left,
            keyboard_rect: None,
            is_portrait: is_portrait != 0,
        });
        inner.pending_events.push(RawHarmonyOSEvent {
            kind: RawHarmonyOSEvent::KIND_WINDOW,
            arg0: width,
            arg1: height,
            arg2: density,
            arg3: 0,
        });
    });
}

/// Dispatch a touch event from ArkTS `onTouch` callback.
///
/// # Safety
/// Caller must ensure this is called on the ArkTS main thread after `init_runtime`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn harmonyos_dispatch_touch(touch_id: u32, x: f32, y: f32, action: u32) {
    with_rt(|rt| {
        rt.borrow_mut().pending_events.push(RawHarmonyOSEvent {
            kind: RawHarmonyOSEvent::KIND_TOUCH,
            arg0: touch_id as f32,
            arg1: x,
            arg2: y,
            arg3: action,
        });
    });
}

/// Notify the runtime of a platform back gesture or system back key press.
///
/// # Safety
/// Caller must ensure this is called on the ArkTS main thread after `init_runtime`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn harmonyos_back_pressed() {
    with_rt(|rt| {
        rt.borrow_mut().pending_events.push(RawHarmonyOSEvent {
            kind: RawHarmonyOSEvent::KIND_BACK,
            arg0: 0.0,
            arg1: 0.0,
            arg2: 0.0,
            arg3: 0,
        });
    });
}

/// Notify the runtime of a soft keyboard visibility / geometry change.
///
/// # Safety
/// Caller must ensure this is called on the ArkTS main thread after `init_runtime`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn harmonyos_input_method_change(
    keyboard_x: f32,
    keyboard_y: f32,
    keyboard_w: f32,
    keyboard_h: f32,
    is_visible: u32,
) {
    with_rt(|rt| {
        let mut inner = rt.borrow_mut();
        if is_visible != 0 {
            inner.metrics.keyboard_insets = zero_ui_core::geometry::Insets {
                top: keyboard_y,
                right: inner.metrics.logical_size.width - (keyboard_x + keyboard_w),
                bottom: inner.metrics.logical_size.height - (keyboard_y + keyboard_h),
                left: keyboard_x,
            };
        } else {
            inner.metrics.keyboard_insets = zero_ui_core::geometry::Insets::all(0.0);
        }
        inner.pending_events.push(RawHarmonyOSEvent {
            kind: RawHarmonyOSEvent::KIND_IME,
            arg0: keyboard_h,
            arg1: if is_visible != 0 { 1.0 } else { 0.0 },
            arg2: 0.0,
            arg3: 0,
        });
    });
}

/// Check whether the global runtime has been initialized.
///
/// # Safety
/// May be called from any thread; the static mut access is internally synchronized.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn harmonyos_is_runtime_ready() -> u32 {
    // Safety: single read of static mut, no race condition in FFI main thread context.
    let ptr = &raw const HARMONYOS_RT;
    unsafe { (*ptr).is_some() as u32 }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::HarmonyOSRuntime;

    #[test]
    fn raw_event_touch_to_ui_event() {
        let raw = RawHarmonyOSEvent {
            kind: RawHarmonyOSEvent::KIND_TOUCH,
            arg0: 0.0,
            arg1: 100.0,
            arg2: 200.0,
            arg3: 1,
        };
        let ev = raw.to_ui_event();
        assert!(matches!(
            ev,
            zero_ui_core::event::UiEvent::Pointer {
                phase: zero_ui_core::event::PointerPhase::Pressed,
                ..
            }
        ));
    }

    #[test]
    fn raw_event_back_to_ui_event() {
        let raw = RawHarmonyOSEvent {
            kind: RawHarmonyOSEvent::KIND_BACK,
            arg0: 0.0,
            arg1: 0.0,
            arg2: 0.0,
            arg3: 0,
        };
        let ev = raw.to_ui_event();
        assert!(matches!(ev, zero_ui_core::event::UiEvent::Key { .. }));
    }

    #[test]
    fn ffi_touch_enqueues_event_directly() {
        let rt = HarmonyOSRuntime::new();
        rt.enqueue_event(RawHarmonyOSEvent {
            kind: RawHarmonyOSEvent::KIND_TOUCH,
            arg0: 0.0,
            arg1: 100.0,
            arg2: 200.0,
            arg3: 1,
        });
        let events = rt.take_pending_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, RawHarmonyOSEvent::KIND_TOUCH);
    }

    #[test]
    fn ffi_back_enqueues_event_directly() {
        let rt = HarmonyOSRuntime::new();
        rt.enqueue_event(RawHarmonyOSEvent {
            kind: RawHarmonyOSEvent::KIND_BACK,
            arg0: 0.0,
            arg1: 0.0,
            arg2: 0.0,
            arg3: 0,
        });
        let events = rt.take_pending_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, RawHarmonyOSEvent::KIND_BACK);
    }

    #[test]
    fn ffi_window_metrics_update() {
        let rt = HarmonyOSRuntime::new();
        rt.set_metrics(event_map::map_window_metrics(event_map::OhosWindowMetricsInput {
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
        }));
        let m = rt.metrics();
        assert_eq!(m.logical_size, zero_ui_core::geometry::Size::new(390.0, 844.0));
        assert_eq!(m.scale_factor, 3.0, "DPI 比 → scale_factor");
        assert_eq!(
            m.density,
            zero_ui_core::layout::DEFAULT_DENSITY,
            "Material 间距密度恒为默认"
        );
        assert_eq!(m.safe_area.top, 47.0);
    }
}
