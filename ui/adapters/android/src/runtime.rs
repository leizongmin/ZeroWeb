//! Android 后端的 [`PlatformRuntime`] 实现（spec IF-006 / DC-15）。
//!
//! [`AndroidRuntime`] 为 M4 Android stretch goal 提供：
//! - `PlatformRuntime` trait 占位实现
//! - `launch` setup 核心（建树 + 工厂注册 + 首帧，headless 可测）
//! - `pump_events` 把 Android 事件泵入 retained 闭环
//!
//! 参考 `ui/adapters/winit/src/runtime.rs` 的 `WinitRuntime` 模式和
//! `ui/adapters/harmonyos/src/runtime.rs` 的 `HarmonyOSRuntime` 模式。

use crate::event_map::system_theme_from_dark_mode;
use crate::ffi::{AndroidSurface, RawAndroidEvent};
use core::cell::RefCell;
use std::rc::Rc;
use zero_ui_core::action::ActionResult;
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::theme::SystemThemeSnapshot;
use zero_ui_runtime::app::UiApp;
use zero_ui_runtime::host::WidgetHost;
use zero_ui_runtime::platform::{PlatformRuntime, UiResult, WindowId};

pub struct AndroidRuntime {
    system_dark: bool,
    system_high_contrast: bool,
    /// Internal state (Rc<RefCell<...>> for JNI borrow convenience).
    inner: Rc<RefCell<RuntimeInner>>,
}

pub struct RuntimeInner {
    pub host: Option<WidgetHost>,
    pub surface: Option<AndroidSurface>,
    pub metrics: WindowMetrics,
    pub pending_events: Vec<RawAndroidEvent>,
}

impl Default for AndroidRuntime {
    fn default() -> AndroidRuntime {
        AndroidRuntime {
            system_dark: false,
            system_high_contrast: false,
            inner: Rc::new(RefCell::new(RuntimeInner {
                host: None,
                surface: None,
                metrics: WindowMetrics::phone(),
                pending_events: Vec::new(),
            })),
        }
    }
}

impl AndroidRuntime {
    pub fn new() -> AndroidRuntime {
        AndroidRuntime::default()
    }

    /// Clone inner Rc for JNI usage (on the JNI thread).
    pub fn inner_rc(&self) -> Rc<RefCell<RuntimeInner>> {
        Rc::clone(&self.inner)
    }

    /// Leak internal state as `&'static` ref (for JNI global access).
    ///
    /// Consumes `self`, turns `RuntimeInner` into `&'static RefCell<RuntimeInner>` via `Box::leak`.
    /// Caller should immediately call `ffi::init_runtime()` to register the global pointer.
    pub fn leak_for_jni(self) -> &'static RefCell<RuntimeInner> {
        let inner = Rc::try_unwrap(self.inner).unwrap_or_else(|_rc| {
            panic!("AndroidRuntime::leak_for_jni: Rc has multiple owners");
        });
        Box::leak(Box::new(inner))
    }

    pub fn set_system_dark(&mut self, dark: bool, high_contrast: bool) {
        self.system_dark = dark;
        self.system_high_contrast = high_contrast;
    }

    pub fn set_metrics(&self, metrics: WindowMetrics) {
        self.inner.borrow_mut().metrics = metrics;
    }

    pub fn metrics(&self) -> WindowMetrics {
        self.inner.borrow().metrics
    }

    pub fn set_surface(&self, surface: AndroidSurface) {
        self.inner.borrow_mut().surface = Some(surface);
    }

    pub fn enqueue_event(&self, raw: RawAndroidEvent) {
        self.inner.borrow_mut().pending_events.push(raw);
    }

    /// Drain pending events queue.
    pub fn take_pending_events(&self) -> Vec<RawAndroidEvent> {
        core::mem::take(&mut self.inner.borrow_mut().pending_events)
    }

    /// Pump Android events into the host retained loop.
    pub fn pump_events(&self, app: &mut dyn UiApp) -> Vec<PumpOutcome> {
        let events: Vec<_> = {
            let mut inner = self.inner.borrow_mut();
            core::mem::take(&mut inner.pending_events)
        };

        let mut outcomes = Vec::with_capacity(events.len());
        for raw in events {
            let ui_event = raw.to_ui_event();
            let mut inner = self.inner.borrow_mut();
            let host = inner.host.as_mut().expect("AndroidRuntime: launch not called");
            let emitted = host.dispatch_event(&ui_event);
            let needs_rebuild = !emitted.is_empty();
            if needs_rebuild {
                for ea in &emitted {
                    let result = app.dispatch(&ea.action, ea.payload.clone());
                    if result == ActionResult::Handled {
                        let spec = app.root_spec();
                        host.set_root(&spec);
                    }
                }
            }
            outcomes.push(PumpOutcome {
                needs_rebuild,
                needs_layout: host.needs_layout(),
                needs_paint: host.needs_paint(),
            });
        }
        outcomes
    }

    /// M4 setup core (headless testable).
    pub fn launch<'app>(
        &'app self,
        app: &'app mut dyn UiApp,
        metrics: WindowMetrics,
        register: impl FnOnce(&mut WidgetHost),
    ) {
        let mut host = WidgetHost::new();
        register(&mut host);
        let spec = app.root_spec();
        host.set_root(&spec);
        let mut inner = self.inner.borrow_mut();
        inner.metrics = metrics;
        inner.host = Some(host);
    }

    pub fn host_mut(&self) -> core::cell::RefMut<'_, WidgetHost> {
        core::cell::RefMut::map(self.inner.borrow_mut(), |inner| {
            inner.host.as_mut().expect("AndroidRuntime: launch not called")
        })
    }

    /// Take ownership of host (for driver/testing).
    pub fn into_host(self) -> Option<WidgetHost> {
        Rc::try_unwrap(self.inner)
            .ok()
            .and_then(|inner| inner.into_inner().host)
    }
}

#[derive(Debug)]
pub struct PumpOutcome {
    pub needs_rebuild: bool,
    pub needs_layout: bool,
    pub needs_paint: bool,
}

impl PlatformRuntime for AndroidRuntime {
    fn run(&mut self, _app: &mut dyn UiApp) -> UiResult<()> {
        Ok(())
    }
    fn request_redraw(&mut self, _window: WindowId) {}
    fn set_ime_area(&mut self, _window: WindowId, _rect: Option<zero_ui_core::geometry::Rect>) {}
    fn system_theme(&self) -> SystemThemeSnapshot {
        system_theme_from_dark_mode(self.system_dark, self.system_high_contrast)
    }
}

// ---------------------------------------------------------------------------
// 测试
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::binding::Value;
    use zero_ui_core::widget::WidgetSpec;
    use zero_ui_runtime::app::UiApp;

    struct TestApp {
        counter: i32,
    }
    impl UiApp for TestApp {
        fn root_spec(&self) -> WidgetSpec {
            let mut s = WidgetSpec::new("test.Label");
            s.id = Some(zero_ui_core::widget::WidgetId::new("label"));
            s.props
                .insert("text", Value::Text(format!("count={}", self.counter).into()));
            s
        }
        fn dispatch(
            &mut self,
            _action: &zero_ui_core::action::ActionId,
            _payload: Option<zero_ui_core::action::ActionPayload>,
        ) -> ActionResult {
            self.counter += 1;
            ActionResult::Handled
        }
    }

    #[test]
    fn launch_then_pump_touch_event_drives_retained_loop() {
        let rt = AndroidRuntime::new();
        let mut app = TestApp { counter: 0 };
        rt.launch(&mut app, WindowMetrics::phone(), |_host| {});

        // Feed a touch event and verify the retained loop runs.
        rt.enqueue_event(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_TOUCH,
            arg0: 0.0,
            arg1: 100.0,
            arg2: 200.0,
            arg3: 0, // ACTION_DOWN
        });
        let outcomes = rt.pump_events(&mut app);
        // Pump shouldn't crash; outcome reflects dispatch.
        assert!(!outcomes.is_empty());
    }

    #[test]
    fn launch_then_pump_back_event_enqueues() {
        let rt = AndroidRuntime::new();
        let mut app = TestApp { counter: 0 };
        rt.launch(&mut app, WindowMetrics::phone(), |_host| {});
        rt.enqueue_event(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_BACK,
            arg0: 0.0,
            arg1: 0.0,
            arg2: 0.0,
            arg3: 0,
        });
        let outcomes = rt.pump_events(&mut app);
        assert!(!outcomes.is_empty());
    }

    #[test]
    fn set_metrics_updates_inner() {
        let rt = AndroidRuntime::new();
        let m = WindowMetrics::phone();
        rt.set_metrics(m);
        assert_eq!(rt.metrics().logical_size, m.logical_size);
    }

    #[test]
    fn enqueue_and_take_events() {
        let rt = AndroidRuntime::new();
        rt.enqueue_event(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_TOUCH,
            arg0: 0.0,
            arg1: 10.0,
            arg2: 20.0,
            arg3: 0,
        });
        rt.enqueue_event(RawAndroidEvent {
            kind: RawAndroidEvent::KIND_KEY,
            arg0: 66.0, // KEYCODE_ENTER
            arg1: 1.0,
            arg2: 0.0,
            arg3: 0,
        });
        let events = rt.take_pending_events();
        assert_eq!(events.len(), 2);
        // Second take returns empty.
        assert_eq!(rt.take_pending_events().len(), 0);
    }
}
