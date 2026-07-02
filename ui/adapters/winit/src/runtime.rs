//! winit 后端的 PlatformRuntime 占位实现（spec IF-006）+ [`WinitRuntime::launch`]（DC-2
//! 终端阻塞壳的可测试 setup 核心）。
//!
//! M1 提供空 `run` 满足 trait 边界与编译期依赖隔离验证；真实事件循环/窗口/surface/IME
//! 在 M2（桌面）与 M4（移动）落地。`launch` 把「建树 + 工厂注册 + 首帧」从阻塞的 GUI
//! run loop 中抽离为可单测的 setup 核心（headless 可验证）；GUI-gated 的 `EventLoop::new`/
//! `Window`/surface + `event_loop.run` 包壳是剩余运行时件（需 GUI 验证首帧）。

use crate::driver::WinitDriver;
use zero_ui_core::geometry::Rect;
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::theme::{ResolvedColorScheme, SystemThemeSnapshot};
use zero_ui_runtime::app::UiApp;
use zero_ui_runtime::host::WidgetHost;
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

    /// 真实 `EventLoop::run` 的**可测试 setup 核心**（DC-2 终端阻塞壳前置）。
    ///
    /// 构造 [`WinitDriver`]、经 `register` 闭包注册应用控件工厂、`begin` 产出首帧，返回
    /// driver 供真实 run loop 继续喂事件（`pump_event`/`pump_frame`）。这把建树、工厂注册、
    /// 首帧三步从阻塞的 GUI run loop 中抽离，使其可在无窗口环境单测——解决了「driver 需在
    /// `begin` 前注册工厂」的设计 blocker（`register` 闭包承载应用特定工厂注册）。
    ///
    /// GUI-gated 的 `EventLoop::new` / `Window` / GPU surface 与 `event_loop.run` 包壳是剩余
    /// 运行时件（需 GUI 验证首帧）。真实 `run` 实现会是：
    /// ```text
    /// let mut driver = WinitRuntime::launch(app, metrics, register);
    /// event_loop.run(|ev, _| {
    ///     driver.pump_event(&event_map::map_*(ev));
    ///     driver.pump_frame();
    ///     render(driver.host().scene());  // 经 zero-ui-adapter-render-foundation 光栅
    /// });
    /// ```
    pub fn launch<'app>(
        app: &'app mut dyn UiApp,
        metrics: WindowMetrics,
        register: impl FnOnce(&mut WidgetHost),
    ) -> WinitDriver<'app> {
        let mut driver = WinitDriver::new(app, metrics);
        register(driver.host_mut());
        driver.begin();
        driver
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
    use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
    use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase, UiEvent};
    use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
    use zero_ui_core::layout::WindowMetrics;
    use zero_ui_core::theme::Color;
    use zero_ui_core::widget::{
        EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget, WidgetId, WidgetSpec,
    };

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

    // ── launch() 测试用品：最小可交互控件 + 计数 UiApp ──

    /// 占位叶子控件：Pressed emit "box.click"；paint 一个 fill（证明经工厂挂载）。
    struct _Box;
    impl Widget for _Box {
        fn mount(&mut self, _ctx: &mut MountCtx) {}
        fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
        fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
            if let UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                ..
            } = event
            {
                EventResult::Emit(ActionId::new("box.click"))
            } else {
                EventResult::Ignored
            }
        }
        fn layout(&mut self, _ctx: &mut LayoutCtx, _c: Constraints) -> Size {
            Size::new(100.0, 40.0)
        }
        fn paint(&mut self, ctx: &mut PaintCtx) {
            ctx.recorder
                .fill_rect(Rect::from_ltrb(0.0, 0.0, 100.0, 40.0), Color::rgb(0.3, 0.5, 0.7));
        }
        fn semantics(&self, _ctx: &mut SemanticsCtx) {}
        fn focusable(&self) -> bool {
            true
        }
    }

    /// 计数 "box.click" 的最小 UiApp（证明 launch 返回的 driver 能驱动 reducer）。
    struct _App {
        clicks: u32,
    }
    impl UiApp for _App {
        fn root_spec(&self) -> WidgetSpec {
            let mut s = WidgetSpec::new("Box");
            s.id = Some(WidgetId::new("box"));
            s
        }
        fn dispatch(&mut self, action: &ActionId, _payload: Option<ActionPayload>) -> ActionResult {
            if action.0.as_str() == "box.click" {
                self.clicks += 1;
                ActionResult::Handled
            } else {
                ActionResult::UnknownAction(action.clone())
            }
        }
    }

    fn press_at(x: f32, y: f32) -> UiEvent {
        UiEvent::Pointer {
            phase: PointerPhase::Pressed,
            button: Some(PointerButton::Primary),
            position: Point::new(x, y),
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        }
    }

    #[test]
    fn launch_registers_factories_before_begin_and_returns_drivable_driver() {
        // DC-2 终端阻塞壳的可测试 setup 核心：launch = driver 构造 + register 工厂 + begin 首帧。
        let mut app = _App { clicks: 0 };
        // launch 返回的 driver 持 &mut app → app.clicks 在作用域外读（下方最终断言）。
        {
            let mut driver = WinitRuntime::launch(&mut app, WindowMetrics::desktop(), |host| {
                host.register("Box", |_spec| Box::new(_Box));
            });
            // register 在 begin 前调用 → _Box 挂载 + 首帧 paint → scene 非空。
            assert!(
                !driver.host().scene().entries.is_empty(),
                "launch 注册工厂在 begin 前 → 首帧非空 scene"
            );
            // 返回的 driver 可继续驱动：点击 → emit → reducer → Handled → 重建。
            let out = driver.pump_event(&press_at(10.0, 10.0));
            assert_eq!(out.emitted_actions, 1);
            assert!(out.spec_rebuilt, "Handled → driver 重建声明树");
            driver.pump_frame();
        }
        assert_eq!(app.clicks, 1, "reducer 被 launch 返回的 driver 驱动");
    }
}
