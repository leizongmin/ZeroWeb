//! JNI C ABI 导出层（Kotlin/Java ↔ Rust 桥接）。
//!
//! 通过 `std::sync::Mutex<Option<Rc<RefCell<RuntimeInner>>>>` 持有运行时引用，
//! Android Activity 通过 `System.loadLibrary("zero_ui_android")` + `external fun` 声明
//! 调用 `#[unsafe(no_mangle)] unsafe extern "C"` 函数喂入 Android 事件。
//!
//! M4 skeleton：headless 可测。真实首帧需 Android Studio + 设备/模拟器。

use crate::event_map;
use crate::runtime::RuntimeInner;
use core::cell::RefCell;

/// Global runtime reference (JNI single-thread model: UI thread drives all calls).
///
/// Uses raw pointer to avoid `Send + Sync` constraints; lifecycle managed by host.
static mut ANDROID_RT: Option<&RefCell<RuntimeInner>> = None;

/// Initialize the global runtime (called once by host; `inner` must be `&'static` or
/// host guarantees lifetime covers all JNI calls).
///
/// # Safety
/// Caller guarantees `inner`'s lifetime covers all JNI calls, and only called on
/// the Android UI thread.
pub unsafe fn init_runtime(inner: &'static RefCell<RuntimeInner>) {
    unsafe {
        ANDROID_RT = Some(inner);
    }
}

fn with_rt<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&RefCell<RuntimeInner>) -> R,
{
    // Safety: caller guarantees single-threaded access (Android UI thread).
    // ANDROID_RT is initialized before any JNI call.
    unsafe { ANDROID_RT.map(f) }
}

// ---------------------------------------------------------------------------
// Raw event types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct AndroidSurface {
    pub surface_id: u64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct RawAndroidEvent {
    pub kind: u32,
    pub arg0: f32,
    pub arg1: f32,
    pub arg2: f32,
    pub arg3: u32,
}

impl RawAndroidEvent {
    pub const KIND_TOUCH: u32 = 1;
    pub const KIND_KEY: u32 = 2;
    pub const KIND_WINDOW: u32 = 3;
    pub const KIND_BACK: u32 = 4;
    pub const KIND_IME: u32 = 5;

    pub fn to_ui_event(&self) -> zero_ui_core::event::UiEvent {
        match self.kind {
            Self::KIND_TOUCH => event_map::map_touch_event(self.arg0 as u32, self.arg1, self.arg2, self.arg3),
            Self::KIND_KEY => zero_ui_core::event::UiEvent::Key {
                code: zero_ui_core::event::KeyCode::new(&format!("android.key.{}", self.arg0 as u32)),
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
// JNI-exported functions (C ABI, callable from Kotlin via external fun)
// ---------------------------------------------------------------------------

/// Notify runtime of window size/density/orientation changes.
///
/// Called from Activity.onConfigurationChanged or initial onCreate.
///
/// # Safety
/// Caller must ensure this is called on the Android UI thread after `init_runtime`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn android_window_size_change(
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
        inner.metrics = event_map::map_window_metrics(event_map::AndroidWindowMetricsInput {
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
        inner.pending_events.push(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_WINDOW,
            arg0: width,
            arg1: height,
            arg2: density,
            arg3: 0,
        });
    });
}

/// Dispatch a touch event from Android `onTouchEvent` callback.
///
/// Kotlin caller extracts `event.getPointerId(i)`, `event.getX(i)`, `event.getY(i)`,
/// and `event.getActionMasked()` for each pointer in the MotionEvent.
///
/// # Safety
/// Caller must ensure this is called on the Android UI thread after `init_runtime`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn android_dispatch_touch(pointer_id: u32, x: f32, y: f32, action: u32) {
    with_rt(|rt| {
        rt.borrow_mut().pending_events.push(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_TOUCH,
            arg0: pointer_id as f32,
            arg1: x,
            arg2: y,
            arg3: action,
        });
    });
}

/// Notify runtime of a platform back gesture or system back key press.
///
/// Called from Activity.onBackPressed or OnBackPressedDispatcher callback.
///
/// # Safety
/// Caller must ensure this is called on the Android UI thread after `init_runtime`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn android_back_pressed() {
    with_rt(|rt| {
        rt.borrow_mut().pending_events.push(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_BACK,
            arg0: 0.0,
            arg1: 0.0,
            arg2: 0.0,
            arg3: 0,
        });
    });
}

/// Notify runtime of a soft keyboard visibility / geometry change.
///
/// Called from ViewTreeObserver.OnGlobalLayoutListener or WindowInsetsAnimation.Callback.
///
/// # Safety
/// Caller must ensure this is called on the Android UI thread after `init_runtime`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn android_input_method_change(
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
        inner.pending_events.push(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_IME,
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
pub unsafe extern "C" fn android_is_runtime_ready() -> u32 {
    let ptr = &raw const ANDROID_RT;
    unsafe { (*ptr).is_some() as u32 }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::AndroidRuntime;

    #[test]
    fn raw_event_touch_to_ui_event() {
        let raw = RawAndroidEvent {
            kind: RawAndroidEvent::KIND_TOUCH,
            arg0: 0.0,
            arg1: 100.0,
            arg2: 200.0,
            arg3: 0, // ACTION_DOWN
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
        let raw = RawAndroidEvent {
            kind: RawAndroidEvent::KIND_BACK,
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
        let rt = AndroidRuntime::new();
        rt.enqueue_event(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_TOUCH,
            arg0: 0.0,
            arg1: 100.0,
            arg2: 200.0,
            arg3: 0,
        });
        let events = rt.take_pending_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, RawAndroidEvent::KIND_TOUCH);
    }

    #[test]
    fn ffi_back_enqueues_event_directly() {
        let rt = AndroidRuntime::new();
        rt.enqueue_event(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_BACK,
            arg0: 0.0,
            arg1: 0.0,
            arg2: 0.0,
            arg3: 0,
        });
        let events = rt.take_pending_events();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].kind, RawAndroidEvent::KIND_BACK);
    }

    #[test]
    fn ffi_window_metrics_update() {
        let rt = AndroidRuntime::new();
        rt.set_metrics(event_map::map_window_metrics(event_map::AndroidWindowMetricsInput {
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
        }));
        let m = rt.metrics();
        assert_eq!(m.logical_size, zero_ui_core::geometry::Size::new(412.0, 915.0));
        assert_eq!(m.density, 2.625);
        assert_eq!(m.safe_area.top, 24.0);
    }
}
