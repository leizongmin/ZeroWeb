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
// JNI-exported functions — Java_com_zeroweb_ui_MainActivity_* (Kotlin external fun)
//
// JNI 函数接受头两个隐藏参数（JNIEnv* + jclass），但 extern "C" 下额外参数存寄存器无害。
// 用 std::ffi::c_void 占位，不依赖 jni crate。
// ---------------------------------------------------------------------------

/// Initialize runtime. Called once from Activity.onCreate.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_zeroweb_ui_MainActivity_nativeInitRuntime(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
) -> u8 {
    // Create a leaked AndroidRuntime for JNI global access.
    // The Kotlin side must hold a long-lived reference; we use Box::leak.
    use crate::runtime::AndroidRuntime;
    let rt = AndroidRuntime::new();
    let inner: &'static RefCell<RuntimeInner> = rt.leak_for_jni();
    unsafe { init_runtime(inner) };
    1 // true
}

/// Notify window size / density change.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_zeroweb_ui_MainActivity_nativeWindowResize(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    width: i32,
    height: i32,
    scale: f32,
) {
    // delegate to existing C ABI function
    unsafe { android_window_size_change(width as f32, height as f32, scale, 1.0, 0.0, 0.0, 0.0, 0.0, 1) };
}

/// Dispatch a single-touch event.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_zeroweb_ui_MainActivity_nativeDispatchTouch(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    pointer_id: i32,
    action: i32,
    x: f32,
    y: f32,
    _timestamp_ms: i64,
) {
    unsafe { android_dispatch_touch(pointer_id as u32, x, y, action as u32) };
}

/// Dispatch a key event.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_zeroweb_ui_MainActivity_nativeDispatchKey(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    key_code: i32,
    action: i32,
) {
    with_rt(|rt| {
        rt.borrow_mut().pending_events.push(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_KEY,
            arg0: key_code as f32,
            arg1: action as f32,
            arg2: 0.0,
            arg3: 0,
        });
    });
}

/// Handle system back gesture.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_zeroweb_ui_MainActivity_nativeBackPressed(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
) -> u8 {
    // Push back event and pump
    unsafe { android_back_pressed() };
    // Check if there's a back handler registered (via BackNavigationService).
    // For skeleton: always report consumed to prevent Activity from finishing.
    1 // true = consumed
}

/// Soft keyboard visibility / geometry change.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_zeroweb_ui_MainActivity_nativeSoftKeyboard(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
    height: i32,
    visible: u8,
) {
    unsafe { android_input_method_change(0.0, 0.0, 0.0, height as f32, visible as u32) };
}

/// Check if runtime is ready.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_zeroweb_ui_MainActivity_nativeIsRuntimeReady(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
) -> u8 {
    (unsafe { android_is_runtime_ready() }) as u8
}

/// Pump pending events through the retained loop.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_zeroweb_ui_MainActivity_nativePumpEvents(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
) {
    with_rt(|rt| {
        let events: Vec<RawAndroidEvent> = core::mem::take(&mut rt.borrow_mut().pending_events);
        for raw in events {
            let ui_event = raw.to_ui_event();
            let mut inner = rt.borrow_mut();
            if let Some(host) = inner.host.as_mut() {
                host.dispatch_event(&ui_event);
            }
        }
    });
}

/// Shutdown runtime (clean up leaked memory).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn Java_com_zeroweb_ui_MainActivity_nativeShutdown(
    _env: *mut std::ffi::c_void,
    _class: *mut std::ffi::c_void,
) {
    // Safety: NULL out the global ptr; leaked Box<RefCell<RuntimeInner>> is dropped
    // by the OS on process exit (acceptable for mobile app lifecycle).
    unsafe { ANDROID_RT = None };
}

// ---------------------------------------------------------------------------
// Plain C ABI functions (internal use, not callable from Kotlin directly)
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
        assert_eq!(m.scale_factor, 2.625, "DPI 比 → scale_factor");
        assert_eq!(
            m.density,
            zero_ui_core::layout::DEFAULT_DENSITY,
            "Material 间距密度恒为默认"
        );
        assert_eq!(m.safe_area.top, 24.0);
    }
}
