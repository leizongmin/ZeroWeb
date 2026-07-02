//! winit 后端的 PlatformRuntime 占位实现（spec IF-006）。
//!
//! M1 提供空实现以满足 trait 边界与编译期依赖隔离验证；真实事件循环/窗口/surface/IME
//! 在 M2（桌面）与 M4（移动）落地。

use zero_ui_core::geometry::Rect;
use zero_ui_core::theme::{ResolvedColorScheme, SystemThemeSnapshot};
use zero_ui_runtime::app::UiApp;
use zero_ui_runtime::platform::{PlatformRuntime, UiResult, WindowId};

/// winit 平台运行时（M1 占位）。
pub struct WinitRuntime {
    system_scheme: ResolvedColorScheme,
}

impl Default for WinitRuntime {
    fn default() -> WinitRuntime {
        WinitRuntime {
            system_scheme: ResolvedColorScheme::Light,
        }
    }
}

impl WinitRuntime {
    pub fn new() -> WinitRuntime {
        WinitRuntime::default()
    }

    /// 测试/注入用：设置系统主题快照。
    pub fn set_system_scheme(&mut self, scheme: ResolvedColorScheme) {
        self.system_scheme = scheme;
    }
}

impl PlatformRuntime for WinitRuntime {
    fn run(&mut self, _app: &mut dyn UiApp) -> UiResult<()> {
        // M1 占位：真实 run loop 在 M2 接入 winit EventLoop。
        Ok(())
    }
    fn request_redraw(&mut self, _window: WindowId) {}
    fn set_ime_area(&mut self, _window: WindowId, _rect: Option<Rect>) {}
    fn system_theme(&self) -> SystemThemeSnapshot {
        SystemThemeSnapshot {
            system_scheme: self.system_scheme,
            high_contrast: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_stubs_compile_and_default() {
        let mut rt = WinitRuntime::new();
        rt.set_system_scheme(ResolvedColorScheme::Dark);
        rt.request_redraw(WindowId(0));
        rt.set_ime_area(WindowId(0), None);
        assert_eq!(rt.system_theme().system_scheme, ResolvedColorScheme::Dark);
    }

    #[test]
    fn run_returns_ok() {
        use zero_ui_runtime::platform::RuntimeError;
        let mut rt = WinitRuntime::new();
        // 用空 app 指针验证 run 签名；不真正驱动循环。
        let result: Result<(), RuntimeError> = rt.run(&mut _Noop);
        assert!(result.is_ok());
    }

    struct _Noop;
    impl UiApp for _Noop {
        fn root_spec(&self) -> zero_ui_core::widget::WidgetSpec {
            zero_ui_core::widget::WidgetSpec::new("Empty")
        }
        fn dispatch(
            &mut self,
            _action: &zero_ui_core::action::ActionId,
            _payload: Option<zero_ui_core::action::ActionPayload>,
        ) -> zero_ui_core::action::ActionResult {
            zero_ui_core::action::ActionResult::Handled
        }
    }
}
