//! 窗口管理 — 创建和管理跨平台窗口
//!
//! 提供两种事件循环运行模式：
//! - `run()`: 基本模式，回调只接收 `AppEvent`
//! - `run_with_window()`: GPU 模式，回调额外接收 `Arc<Window>` 引用

use crate::event::{
    convert_ime, convert_keyboard_input, convert_mouse_button, convert_scroll_delta,
    convert_touch_phase, AppEvent, TouchEvent,
};
use crate::{HostError, HostResult};
use std::sync::Arc;

/// 窗口配置
#[derive(Clone)]
pub struct WindowConfig {
    /// 窗口标题
    pub title: String,
    /// 窗口宽度
    pub width: u32,
    /// 窗口高度
    pub height: u32,
    /// 是否可调整大小
    pub resizable: bool,
}

impl WindowConfig {
    /// 创建默认窗口配置
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 800,
            height: 600,
            resizable: true,
        }
    }

    /// 设置窗口尺寸
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// 设置是否可调整大小
    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }
}

/// 宿主运行时 — 管理窗口和事件循环
///
/// 使用 winit 作为后端，提供跨平台窗口管理。
pub struct HostRuntime {
    config: WindowConfig,
}

impl HostRuntime {
    /// 创建新的宿主运行时
    pub fn new(config: WindowConfig) -> Self {
        Self { config }
    }

    /// 运行事件循环（基本模式，无窗口引用）
    pub fn run<F>(self, mut on_event: F) -> HostResult<()>
    where
        F: FnMut(AppEvent) + 'static,
    {
        let event_loop = winit::event_loop::EventLoop::new()
            .map_err(|e| HostError::EventLoopError(e.to_string()))?;

        let window_attrs = winit::window::WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.width,
                self.config.height,
            ))
            .with_resizable(self.config.resizable);

        event_loop
            .run_app(&mut BasicApp::new_basic(window_attrs, &mut on_event))
            .map_err(|e| HostError::EventLoopError(e.to_string()))?;

        Ok(())
    }

    /// 运行事件循环（GPU 模式，提供窗口引用）
    ///
    /// 回调函数接收 `(AppEvent, Option<Arc<Window>>)` — 窗口在 `resumed` 后可用。
    /// 用于需要访问 winit 窗口以创建 wgpu Surface 的场景。
    pub fn run_with_window<F>(self, mut on_event: F) -> HostResult<()>
    where
        F: FnMut(AppEvent, Option<Arc<winit::window::Window>>) + 'static,
    {
        let event_loop = winit::event_loop::EventLoop::new()
            .map_err(|e| HostError::EventLoopError(e.to_string()))?;

        let window_attrs = winit::window::WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.width,
                self.config.height,
            ))
            .with_resizable(self.config.resizable);

        event_loop
            .run_app(&mut GpuApp::new_with_window(window_attrs, &mut on_event))
            .map_err(|e| HostError::EventLoopError(e.to_string()))?;

        Ok(())
    }
}

/// 基本模式事件处理器
pub(crate) struct BasicApp<'a, F> {
    window_attrs: Option<winit::window::WindowAttributes>,
    window: Option<Arc<winit::window::Window>>,
    on_event: &'a mut F,
}

impl<'a, F: FnMut(AppEvent)> BasicApp<'a, F> {
    /// 创建基本模式事件处理器（用于测试）
    pub(crate) fn new_basic(
        window_attrs: winit::window::WindowAttributes,
        on_event: &'a mut F,
    ) -> Self {
        Self {
            window_attrs: Some(window_attrs),
            window: None,
            on_event,
        }
    }
}

impl<F: FnMut(AppEvent)> BasicApp<'_, F> {
    /// 处理单个 winit WindowEvent，转换为 AppEvent 并分发（用于测试）
    pub(crate) fn handle_window_event(&mut self, event: winit::event::WindowEvent) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                // 基本模式中无法退出事件循环，仅分发事件
                (self.on_event)(AppEvent::CloseRequested);
            }
            winit::event::WindowEvent::Resized(size) => {
                (self.on_event)(AppEvent::Resized {
                    width: size.width,
                    height: size.height,
                });
            }
            winit::event::WindowEvent::RedrawRequested => {
                (self.on_event)(AppEvent::RedrawRequested);
                if let Some(ref win) = self.window {
                    win.request_redraw();
                }
            }
            winit::event::WindowEvent::Focused(focused) => {
                let event = if focused {
                    AppEvent::Focused
                } else {
                    AppEvent::Unfocused
                };
                (self.on_event)(event);
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                (self.on_event)(convert_keyboard_input(device_id, event, is_synthetic));
            }
            winit::event::WindowEvent::CursorMoved {
                position, ..
            } => {
                (self.on_event)(AppEvent::MouseMoved {
                    x: position.x,
                    y: position.y,
                });
            }
            winit::event::WindowEvent::MouseInput {
                state, button, ..
            } => {
                (self.on_event)(AppEvent::MouseInput {
                    button: convert_mouse_button(button),
                    pressed: state == winit::event::ElementState::Pressed,
                });
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                (self.on_event)(AppEvent::MouseWheel {
                    delta: convert_scroll_delta(delta),
                });
            }
            winit::event::WindowEvent::Touch(touch) => {
                (self.on_event)(AppEvent::Touch(TouchEvent {
                    id: touch.id,
                    phase: convert_touch_phase(touch.phase),
                    x: touch.location.x,
                    y: touch.location.y,
                }));
            }
            winit::event::WindowEvent::Ime(ime) => {
                (self.on_event)(AppEvent::Ime(convert_ime(ime)));
            }
            _ => {}
        }
    }
}

impl<F: FnMut(AppEvent)> winit::application::ApplicationHandler<()> for BasicApp<'_, F> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none()
            && let Some(attrs) = self.window_attrs.take()
        {
            let win = event_loop
                .create_window(attrs)
                .expect("Failed to create window");
            self.window = Some(Arc::new(win));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            other => self.handle_window_event(other),
        }
    }
}

/// GPU 模式事件处理器（提供窗口引用）
pub(crate) struct GpuApp<'a, F> {
    window_attrs: Option<winit::window::WindowAttributes>,
    window: Option<Arc<winit::window::Window>>,
    on_event: &'a mut F,
}

impl<'a, F: FnMut(AppEvent, Option<Arc<winit::window::Window>>)> GpuApp<'a, F> {
    /// 创建 GPU 模式事件处理器（用于测试）
    pub(crate) fn new_with_window(
        window_attrs: winit::window::WindowAttributes,
        on_event: &'a mut F,
    ) -> Self {
        Self {
            window_attrs: Some(window_attrs),
            window: None,
            on_event,
        }
    }
}

impl<F: FnMut(AppEvent, Option<Arc<winit::window::Window>>)> GpuApp<'_, F> {
    /// 处理单个 winit WindowEvent，转换为 AppEvent 并分发（用于测试）
    pub(crate) fn handle_window_event(
        &mut self,
        event: winit::event::WindowEvent,
        win_ref: Option<Arc<winit::window::Window>>,
    ) {
        match event {
            winit::event::WindowEvent::CloseRequested => {
                (self.on_event)(AppEvent::CloseRequested, win_ref);
            }
            winit::event::WindowEvent::Resized(size) => {
                (self.on_event)(
                    AppEvent::Resized {
                        width: size.width,
                        height: size.height,
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::RedrawRequested => {
                (self.on_event)(AppEvent::RedrawRequested, win_ref);
                if let Some(ref win) = self.window {
                    win.request_redraw();
                }
            }
            winit::event::WindowEvent::Focused(focused) => {
                (self.on_event)(
                    if focused {
                        AppEvent::Focused
                    } else {
                        AppEvent::Unfocused
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                event,
                is_synthetic,
            } => {
                (self.on_event)(
                    convert_keyboard_input(device_id, event, is_synthetic),
                    win_ref,
                );
            }
            winit::event::WindowEvent::CursorMoved {
                position, ..
            } => {
                (self.on_event)(
                    AppEvent::MouseMoved {
                        x: position.x,
                        y: position.y,
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::MouseInput {
                state, button, ..
            } => {
                (self.on_event)(
                    AppEvent::MouseInput {
                        button: convert_mouse_button(button),
                        pressed: state == winit::event::ElementState::Pressed,
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                (self.on_event)(
                    AppEvent::MouseWheel {
                        delta: convert_scroll_delta(delta),
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::Touch(touch) => {
                (self.on_event)(
                    AppEvent::Touch(TouchEvent {
                        id: touch.id,
                        phase: convert_touch_phase(touch.phase),
                        x: touch.location.x,
                        y: touch.location.y,
                    }),
                    win_ref,
                );
            }
            winit::event::WindowEvent::Ime(ime) => {
                (self.on_event)(AppEvent::Ime(convert_ime(ime)), win_ref);
            }
            _ => {}
        }
    }
}

impl<F: FnMut(AppEvent, Option<Arc<winit::window::Window>>)>
    winit::application::ApplicationHandler<()> for GpuApp<'_, F>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none()
            && let Some(attrs) = self.window_attrs.take()
        {
            let win = event_loop
                .create_window(attrs)
                .expect("Failed to create window");
            self.window = Some(Arc::new(win));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let win_ref = self.window.clone();
        match event {
            winit::event::WindowEvent::CloseRequested => {
                (self.on_event)(AppEvent::CloseRequested, win_ref);
                event_loop.exit();
            }
            other => self.handle_window_event(other, win_ref),
        }
    }
}

// Re-export winit window type for convenience
pub use winit::window::Window;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{ImeEvent, MouseButton, MouseScrollDelta, TouchPhase};

    // === WindowConfig 测试 ===

    #[test]
    fn test_window_config_default() {
        let config = WindowConfig::new("Test");
        assert_eq!(config.title, "Test");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(config.resizable);
    }

    #[test]
    fn test_window_config_builder() {
        let config = WindowConfig::new("Test")
            .with_size(1024, 768)
            .with_resizable(false);
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
        assert!(!config.resizable);
    }

    #[test]
    fn test_host_runtime_new() {
        let config = WindowConfig::new("Test");
        let _runtime = HostRuntime::new(config);
    }

    #[test]
    fn test_window_config_new_empty_title() {
        let config = WindowConfig::new("");
        assert_eq!(config.title, "");
        assert_eq!(config.width, 800);
    }

    #[test]
    fn test_window_config_with_size_zero() {
        let config = WindowConfig::new("T").with_size(0, 0);
        assert_eq!(config.width, 0);
        assert_eq!(config.height, 0);
    }

    #[test]
    fn test_window_config_with_size_max() {
        let config = WindowConfig::new("T").with_size(u32::MAX, u32::MAX);
        assert_eq!(config.width, u32::MAX);
        assert_eq!(config.height, u32::MAX);
    }

    #[test]
    fn test_window_config_builder_only_width() {
        let config = WindowConfig::new("T").with_size(500, 600);
        assert_eq!(config.width, 500);
        assert_eq!(config.height, 600);
    }

    #[test]
    fn test_window_config_fields_public_mutable() {
        let mut config = WindowConfig::new("Original");
        config.title = "Modified".to_string();
        assert_eq!(config.title, "Modified");
    }

    #[test]
    fn test_window_config_from_string_ref() {
        let title = String::from("Test");
        let config = WindowConfig::new(&title);
        assert_eq!(config.title, "Test");
    }

    #[test]
    fn test_host_runtime_new_custom_config() {
        let config = WindowConfig::new("Custom")
            .with_size(1920, 1080)
            .with_resizable(false);
        let _runtime = HostRuntime::new(config);
    }

    // === BasicApp 事件分发测试 ===

    fn make_basic_app<F: FnMut(AppEvent)>(on_event: &'_ mut F) -> BasicApp<'_, F> {
        let attrs = winit::window::WindowAttributes::default();
        BasicApp::new_basic(attrs, on_event)
    }

    #[test]
    fn test_basic_app_resized_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        let size = winit::dpi::PhysicalSize::new(800, 600);
        app.handle_window_event(winit::event::WindowEvent::Resized(size));
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 800);
                assert_eq!(*height, 600);
            }
            _ => panic!("Expected Resized, got {:?}", received[0]),
        }
    }

    #[test]
    fn test_basic_app_resized_zero() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Resized(
            winit::dpi::PhysicalSize::new(0, 0),
        ));
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 0);
                assert_eq!(*height, 0);
            }
            _ => panic!("Expected Resized"),
        }
    }

    #[test]
    fn test_basic_app_focused_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Focused(true));
        app.handle_window_event(winit::event::WindowEvent::Focused(false));
        assert_eq!(received.len(), 2);
        assert!(matches!(received[0], AppEvent::Focused));
        assert!(matches!(received[1], AppEvent::Unfocused));
    }

    #[test]
    fn test_basic_app_redraw_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::RedrawRequested);
        assert_eq!(received.len(), 1);
        assert!(matches!(received[0], AppEvent::RedrawRequested));
    }

    #[test]
    fn test_basic_app_close_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::CloseRequested);
        assert_eq!(received.len(), 1);
        assert!(matches!(received[0], AppEvent::CloseRequested));
    }

    #[test]
    fn test_basic_app_cursor_moved_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::CursorMoved {
            device_id: winit::event::DeviceId::dummy(),
            position: winit::dpi::PhysicalPosition::new(100.0, 200.0),
        });
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseMoved { x, y } => {
                assert!((*x - 100.0).abs() < f64::EPSILON);
                assert!((*y - 200.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected MouseMoved, got {:?}", received[0]),
        }
    }

    #[test]
    fn test_basic_app_cursor_moved_negative() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::CursorMoved {
            device_id: winit::event::DeviceId::dummy(),
            position: winit::dpi::PhysicalPosition::new(-10.0, -20.0),
        });
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseMoved { x, y } => {
                assert!((*x - (-10.0)).abs() < f64::EPSILON);
                assert!((*y - (-20.0)).abs() < f64::EPSILON);
            }
            _ => panic!("Expected MouseMoved"),
        }
    }

    #[test]
    fn test_basic_app_mouse_input_press_release() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Left,
        });
        app.handle_window_event(winit::event::WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: winit::event::ElementState::Released,
            button: winit::event::MouseButton::Right,
        });
        assert_eq!(received.len(), 2);
        match &received[0] {
            AppEvent::MouseInput { button, pressed } => {
                assert_eq!(*button, MouseButton::Left);
                assert!(*pressed);
            }
            _ => panic!("Expected MouseInput"),
        }
        match &received[1] {
            AppEvent::MouseInput { button, pressed } => {
                assert_eq!(*button, MouseButton::Right);
                assert!(!pressed);
            }
            _ => panic!("Expected MouseInput"),
        }
    }

    #[test]
    fn test_basic_app_mouse_input_all_buttons() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        for btn in [
            winit::event::MouseButton::Left,
            winit::event::MouseButton::Right,
            winit::event::MouseButton::Middle,
            winit::event::MouseButton::Back,
            winit::event::MouseButton::Forward,
            winit::event::MouseButton::Other(8),
        ] {
            app.handle_window_event(winit::event::WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: winit::event::ElementState::Pressed,
                button: btn,
            });
        }
        assert_eq!(received.len(), 6);
        assert!(matches!(
            &received[0],
            AppEvent::MouseInput {
                button: MouseButton::Left,
                pressed: true
            }
        ));
        assert!(matches!(
            &received[5],
            AppEvent::MouseInput {
                button: MouseButton::Other(8),
                pressed: true
            }
        ));
    }

    #[test]
    fn test_basic_app_mouse_wheel_line() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::MouseWheel {
            device_id: winit::event::DeviceId::dummy(),
            delta: winit::event::MouseScrollDelta::LineDelta(3.0, -1.0),
            phase: winit::event::TouchPhase::Moved,
        });
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseWheel { delta } => {
                assert_eq!(*delta, MouseScrollDelta::LineDelta(3.0, -1.0));
            }
            _ => panic!("Expected MouseWheel"),
        }
    }

    #[test]
    fn test_basic_app_mouse_wheel_pixel() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::MouseWheel {
            device_id: winit::event::DeviceId::dummy(),
            delta: winit::event::MouseScrollDelta::PixelDelta(
                winit::dpi::PhysicalPosition::new(10.0, -5.0),
            ),
            phase: winit::event::TouchPhase::Moved,
        });
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseWheel { delta } => {
                assert_eq!(*delta, MouseScrollDelta::PixelDelta(10.0, -5.0));
            }
            _ => panic!("Expected MouseWheel"),
        }
    }

    #[test]
    fn test_basic_app_touch_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            phase: winit::event::TouchPhase::Started,
            location: winit::dpi::PhysicalPosition::new(50.0, 75.0),
            id: 42,
            force: None,
        }));
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::Touch(te) => {
                assert_eq!(te.id, 42);
                assert_eq!(te.phase, TouchPhase::Started);
                assert!((te.x - 50.0).abs() < f64::EPSILON);
                assert!((te.y - 75.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected Touch"),
        }
    }

    #[test]
    fn test_basic_app_touch_all_phases() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        for phase in [
            winit::event::TouchPhase::Started,
            winit::event::TouchPhase::Moved,
            winit::event::TouchPhase::Ended,
            winit::event::TouchPhase::Cancelled,
        ] {
            app.handle_window_event(winit::event::WindowEvent::Touch(winit::event::Touch {
                device_id: winit::event::DeviceId::dummy(),
                phase,
                location: winit::dpi::PhysicalPosition::new(0.0, 0.0),
                id: 0,
                force: None,
            }));
        }
        assert_eq!(received.len(), 4);
        assert!(matches!(
            &received[0],
            AppEvent::Touch(TouchEvent {
                phase: TouchPhase::Started,
                ..
            })
        ));
        assert!(matches!(
            &received[3],
            AppEvent::Touch(TouchEvent {
                phase: TouchPhase::Cancelled,
                ..
            })
        ));
    }

    #[test]
    fn test_basic_app_ime_full_lifecycle() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Ime(
            winit::event::Ime::Enabled,
        ));
        app.handle_window_event(winit::event::WindowEvent::Ime(
            winit::event::Ime::Preedit("你好".to_string(), Some((0, 2))),
        ));
        app.handle_window_event(winit::event::WindowEvent::Ime(
            winit::event::Ime::Commit("你好世界".to_string()),
        ));
        app.handle_window_event(winit::event::WindowEvent::Ime(
            winit::event::Ime::Disabled,
        ));
        assert_eq!(received.len(), 4);
        assert!(matches!(&received[0], AppEvent::Ime(ImeEvent::Enabled)));
        match &received[1] {
            AppEvent::Ime(ImeEvent::Preedit { text, cursor }) => {
                assert_eq!(text, "你好");
                assert_eq!(*cursor, Some((0, 2)));
            }
            _ => panic!("Expected Ime Preedit"),
        }
        match &received[2] {
            AppEvent::Ime(ImeEvent::Commit(s)) => assert_eq!(s, "你好世界"),
            _ => panic!("Expected Ime Commit"),
        }
        assert!(matches!(&received[3], AppEvent::Ime(ImeEvent::Disabled)));
    }

    #[test]
    fn test_basic_app_ime_preedit_no_cursor() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Ime(
            winit::event::Ime::Preedit(String::new(), None),
        ));
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::Ime(ImeEvent::Preedit { text, cursor }) => {
                assert!(text.is_empty());
                assert!(cursor.is_none());
            }
            _ => panic!("Expected Ime Preedit"),
        }
    }

    #[test]
    fn test_basic_app_multiple_events_order() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Focused(true));
        app.handle_window_event(winit::event::WindowEvent::CursorMoved {
            device_id: winit::event::DeviceId::dummy(),
            position: winit::dpi::PhysicalPosition::new(10.0, 20.0),
        });
        app.handle_window_event(winit::event::WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Left,
        });
        app.handle_window_event(winit::event::WindowEvent::MouseWheel {
            device_id: winit::event::DeviceId::dummy(),
            delta: winit::event::MouseScrollDelta::LineDelta(1.0, 0.0),
            phase: winit::event::TouchPhase::Moved,
        });
        assert_eq!(received.len(), 4);
        assert!(matches!(received[0], AppEvent::Focused));
        assert!(matches!(received[1], AppEvent::MouseMoved { .. }));
        assert!(matches!(received[2], AppEvent::MouseInput { .. }));
        assert!(matches!(received[3], AppEvent::MouseWheel { .. }));
    }

    #[test]
    fn test_basic_app_ignored_events() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Destroyed);
        app.handle_window_event(winit::event::WindowEvent::ThemeChanged(
            winit::window::Theme::Light,
        ));
        app.handle_window_event(winit::event::WindowEvent::Occluded(false));
        assert!(received.is_empty(), "Ignored events should not dispatch");
    }

    // === GpuApp 事件分发测试 ===

    fn make_gpu_app<F: FnMut(AppEvent, Option<Arc<winit::window::Window>>)>(
        on_event: &'_ mut F,
    ) -> GpuApp<'_, F> {
        let attrs = winit::window::WindowAttributes::default();
        GpuApp::new_with_window(attrs, on_event)
    }

    #[test]
    fn test_gpu_app_resized_dispatch() {
        let mut received: Vec<(AppEvent, bool)> = Vec::new();
        let mut callback =
            |e: AppEvent, w: Option<Arc<winit::window::Window>>| received.push((e, w.is_some()));
        let mut app = make_gpu_app(&mut callback);
        let size = winit::dpi::PhysicalSize::new(1024, 768);
        app.handle_window_event(winit::event::WindowEvent::Resized(size), None);
        assert_eq!(received.len(), 1);
        assert!(matches!(received[0].0, AppEvent::Resized { .. }));
        assert!(!received[0].1, "No window set yet");
    }

    #[test]
    fn test_gpu_app_focused_dispatch() {
        let mut received: Vec<(AppEvent, bool)> = Vec::new();
        let mut callback =
            |e: AppEvent, w: Option<Arc<winit::window::Window>>| received.push((e, w.is_some()));
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Focused(true), None);
        app.handle_window_event(winit::event::WindowEvent::Focused(false), None);
        assert_eq!(received.len(), 2);
        assert!(matches!(received[0].0, AppEvent::Focused));
        assert!(matches!(received[1].0, AppEvent::Unfocused));
    }

    #[test]
    fn test_gpu_app_redraw_dispatch() {
        let mut received: Vec<(AppEvent, bool)> = Vec::new();
        let mut callback =
            |e: AppEvent, w: Option<Arc<winit::window::Window>>| received.push((e, w.is_some()));
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::RedrawRequested, None);
        assert_eq!(received.len(), 1);
        assert!(matches!(received[0].0, AppEvent::RedrawRequested));
    }

    #[test]
    fn test_gpu_app_close_dispatch() {
        let mut received: Vec<(AppEvent, bool)> = Vec::new();
        let mut callback =
            |e: AppEvent, w: Option<Arc<winit::window::Window>>| received.push((e, w.is_some()));
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::CloseRequested, None);
        assert_eq!(received.len(), 1);
        assert!(matches!(received[0].0, AppEvent::CloseRequested));
    }

    #[test]
    fn test_gpu_app_cursor_moved_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(300.0, 400.0),
            },
            None,
        );
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseMoved { x, y } => {
                assert!((*x - 300.0).abs() < f64::EPSILON);
                assert!((*y - 400.0).abs() < f64::EPSILON);
            }
            _ => panic!("Expected MouseMoved"),
        }
    }

    #[test]
    fn test_gpu_app_mouse_input_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Middle,
            },
            None,
        );
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseInput { button, pressed } => {
                assert_eq!(*button, MouseButton::Middle);
                assert!(*pressed);
            }
            _ => panic!("Expected MouseInput"),
        }
    }

    #[test]
    fn test_gpu_app_mouse_wheel_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::MouseWheel {
                device_id: winit::event::DeviceId::dummy(),
                delta: winit::event::MouseScrollDelta::PixelDelta(
                    winit::dpi::PhysicalPosition::new(5.0, -3.0),
                ),
                phase: winit::event::TouchPhase::Moved,
            },
            None,
        );
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseWheel { delta } => {
                assert_eq!(*delta, MouseScrollDelta::PixelDelta(5.0, -3.0));
            }
            _ => panic!("Expected MouseWheel"),
        }
    }

    #[test]
    fn test_gpu_app_touch_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::Touch(winit::event::Touch {
                device_id: winit::event::DeviceId::dummy(),
                phase: winit::event::TouchPhase::Ended,
                location: winit::dpi::PhysicalPosition::new(123.0, 456.0),
                id: 7,
                force: None,
            }),
            None,
        );
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::Touch(te) => {
                assert_eq!(te.id, 7);
                assert_eq!(te.phase, TouchPhase::Ended);
            }
            _ => panic!("Expected Touch"),
        }
    }

    #[test]
    fn test_gpu_app_ime_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Commit("test".to_string())),
            None,
        );
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::Ime(ImeEvent::Commit(s)) => assert_eq!(s, "test"),
            _ => panic!("Expected Ime Commit"),
        }
    }

    #[test]
    fn test_gpu_app_window_ref_none_when_no_window() {
        let mut has_window = false;
        let mut callback = |_: AppEvent, w: Option<Arc<winit::window::Window>>| {
            if w.is_some() {
                has_window = true;
            }
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Focused(true), None);
        assert!(!has_window);
    }

    #[test]
    fn test_gpu_app_ignored_events() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Destroyed, None);
        app.handle_window_event(
            winit::event::WindowEvent::ThemeChanged(winit::window::Theme::Dark),
            None,
        );
        assert!(
            received.is_empty(),
            "Ignored events should not dispatch"
        );
    }

    #[test]
    fn test_gpu_app_full_ime_lifecycle() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        // Simulate full IME lifecycle: enabled -> preedit -> commit -> disabled
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Enabled),
            None,
        );
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Preedit(
                "n".to_string(),
                Some((1, 1)),
            )),
            None,
        );
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Preedit(
                "ni".to_string(),
                Some((2, 2)),
            )),
            None,
        );
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Commit("你".to_string())),
            None,
        );
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Disabled),
            None,
        );
        assert_eq!(received.len(), 5);
        assert!(matches!(&received[0], AppEvent::Ime(ImeEvent::Enabled)));
        assert!(matches!(&received[4], AppEvent::Ime(ImeEvent::Disabled)));
        match &received[3] {
            AppEvent::Ime(ImeEvent::Commit(s)) => assert_eq!(s, "你"),
            _ => panic!("Expected Ime Commit"),
        }
    }

    #[test]
    fn test_gpu_app_multiple_events() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Focused(true), None);
        app.handle_window_event(
            winit::event::WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(1.0, 2.0),
            },
            None,
        );
        app.handle_window_event(
            winit::event::WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
            },
            None,
        );
        assert_eq!(received.len(), 3);
        assert!(matches!(received[0], AppEvent::Focused));
        assert!(matches!(received[1], AppEvent::MouseMoved { .. }));
        assert!(matches!(received[2], AppEvent::MouseInput { .. }));
    }

    // === Additional coverage tests ===

    // -- WindowConfig --

    #[test]
    fn test_window_config_clone() {
        let config = WindowConfig::new("CloneTest")
            .with_size(640, 480)
            .with_resizable(false);
        let cloned = config.clone();
        assert_eq!(cloned.title, "CloneTest");
        assert_eq!(cloned.width, 640);
        assert_eq!(cloned.height, 480);
        assert!(!cloned.resizable);
    }

    #[test]
    fn test_window_config_builder_chaining_preserves_all() {
        let config = WindowConfig::new("Chain")
            .with_size(1280, 720)
            .with_resizable(true);
        assert_eq!(config.title, "Chain");
        assert_eq!(config.width, 1280);
        assert_eq!(config.height, 720);
        assert!(config.resizable);
    }

    #[test]
    fn test_window_config_with_resizable_does_not_change_size() {
        let config = WindowConfig::new("R").with_resizable(false);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(!config.resizable);
    }

    #[test]
    fn test_window_config_with_size_does_not_change_resizable() {
        let config = WindowConfig::new("S").with_size(100, 100);
        assert_eq!(config.width, 100);
        assert_eq!(config.height, 100);
        assert!(config.resizable);
    }

    #[test]
    fn test_window_config_title_unicode() {
        let config = WindowConfig::new("こんにちは世界");
        assert_eq!(config.title, "こんにちは世界");
    }

    #[test]
    fn test_window_config_title_from_string() {
        let title = String::from("OwnedTitle");
        let config = WindowConfig::new(title);
        assert_eq!(config.title, "OwnedTitle");
    }

    // -- BasicApp initial state --

    #[test]
    fn test_basic_app_window_initially_none() {
        let mut callback = |_: AppEvent| {};
        let mut app = make_basic_app(&mut callback);
        assert!(app.window.is_none());
    }

    #[test]
    fn test_basic_app_window_attrs_initially_some() {
        let mut callback = |_: AppEvent| {};
        let mut app = make_basic_app(&mut callback);
        assert!(app.window_attrs.is_some());
    }

    // -- BasicApp event dispatch edge cases --

    #[test]
    fn test_basic_app_resized_large() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Resized(
            winit::dpi::PhysicalSize::new(7680, 4320),
        ));
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 7680);
                assert_eq!(*height, 4320);
            }
            _ => panic!("Expected Resized"),
        }
    }

    #[test]
    fn test_basic_app_multiple_resizes() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Resized(
            winit::dpi::PhysicalSize::new(100, 100),
        ));
        app.handle_window_event(winit::event::WindowEvent::Resized(
            winit::dpi::PhysicalSize::new(200, 200),
        ));
        app.handle_window_event(winit::event::WindowEvent::Resized(
            winit::dpi::PhysicalSize::new(300, 300),
        ));
        assert_eq!(received.len(), 3);
        match &received[0] {
            AppEvent::Resized { width, .. } => assert_eq!(*width, 100),
            _ => panic!("Expected Resized"),
        }
        match &received[2] {
            AppEvent::Resized { width, .. } => assert_eq!(*width, 300),
            _ => panic!("Expected Resized"),
        }
    }

    #[test]
    fn test_basic_app_consecutive_focus_events() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Focused(true));
        app.handle_window_event(winit::event::WindowEvent::Focused(false));
        app.handle_window_event(winit::event::WindowEvent::Focused(true));
        assert_eq!(received.len(), 3);
        assert!(matches!(received[0], AppEvent::Focused));
        assert!(matches!(received[1], AppEvent::Unfocused));
        assert!(matches!(received[2], AppEvent::Focused));
    }

    #[test]
    fn test_basic_app_mouse_input_back_forward_buttons() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Back,
        });
        app.handle_window_event(winit::event::WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: winit::event::ElementState::Released,
            button: winit::event::MouseButton::Forward,
        });
        assert_eq!(received.len(), 2);
        match &received[0] {
            AppEvent::MouseInput { button, pressed } => {
                assert_eq!(*button, MouseButton::Back);
                assert!(*pressed);
            }
            _ => panic!("Expected MouseInput"),
        }
        match &received[1] {
            AppEvent::MouseInput { button, pressed } => {
                assert_eq!(*button, MouseButton::Forward);
                assert!(!pressed);
            }
            _ => panic!("Expected MouseInput"),
        }
    }

    #[test]
    fn test_basic_app_mouse_input_other_button() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::MouseInput {
            device_id: winit::event::DeviceId::dummy(),
            state: winit::event::ElementState::Pressed,
            button: winit::event::MouseButton::Other(9),
        });
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseInput { button, pressed } => {
                assert_eq!(*button, MouseButton::Other(9));
                assert!(*pressed);
            }
            _ => panic!("Expected MouseInput"),
        }
    }

    #[test]
    fn test_basic_app_mouse_wheel_negative_pixel() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::MouseWheel {
            device_id: winit::event::DeviceId::dummy(),
            delta: winit::event::MouseScrollDelta::PixelDelta(
                winit::dpi::PhysicalPosition::new(-100.0, -200.0),
            ),
            phase: winit::event::TouchPhase::Started,
        });
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseWheel { delta } => {
                assert_eq!(*delta, MouseScrollDelta::PixelDelta(-100.0, -200.0));
            }
            _ => panic!("Expected MouseWheel"),
        }
    }

    #[test]
    fn test_basic_app_touch_multiple_ids() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        for id in [0u64, 1, 2] {
            app.handle_window_event(winit::event::WindowEvent::Touch(winit::event::Touch {
                device_id: winit::event::DeviceId::dummy(),
                phase: winit::event::TouchPhase::Started,
                location: winit::dpi::PhysicalPosition::new(0.0, 0.0),
                id,
                force: None,
            }));
        }
        assert_eq!(received.len(), 3);
        match &received[0] {
            AppEvent::Touch(te) => assert_eq!(te.id, 0),
            _ => panic!("Expected Touch"),
        }
        match &received[2] {
            AppEvent::Touch(te) => assert_eq!(te.id, 2),
            _ => panic!("Expected Touch"),
        }
    }

    #[test]
    fn test_basic_app_mixed_event_sequence() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Focused(true));
        app.handle_window_event(winit::event::WindowEvent::Resized(
            winit::dpi::PhysicalSize::new(500, 400),
        ));
        app.handle_window_event(winit::event::WindowEvent::CursorMoved {
            device_id: winit::event::DeviceId::dummy(),
            position: winit::dpi::PhysicalPosition::new(10.0, 20.0),
        });
        app.handle_window_event(winit::event::WindowEvent::RedrawRequested);
        app.handle_window_event(winit::event::WindowEvent::CloseRequested);
        assert_eq!(received.len(), 5);
        assert!(matches!(received[0], AppEvent::Focused));
        assert!(matches!(received[1], AppEvent::Resized { .. }));
        assert!(matches!(received[2], AppEvent::MouseMoved { .. }));
        assert!(matches!(received[3], AppEvent::RedrawRequested));
        assert!(matches!(received[4], AppEvent::CloseRequested));
    }

    // -- GpuApp initial state --

    #[test]
    fn test_gpu_app_window_initially_none() {
        let mut callback = |_: AppEvent, _: Option<Arc<winit::window::Window>>| {};
        let mut app = make_gpu_app(&mut callback);
        assert!(app.window.is_none());
    }

    #[test]
    fn test_gpu_app_window_attrs_initially_some() {
        let mut callback = |_: AppEvent, _: Option<Arc<winit::window::Window>>| {};
        let mut app = make_gpu_app(&mut callback);
        assert!(app.window_attrs.is_some());
    }

    // -- GpuApp additional dispatch --

    #[test]
    fn test_gpu_app_resized_large() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(3840, 2160)),
            None,
        );
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 3840);
                assert_eq!(*height, 2160);
            }
            _ => panic!("Expected Resized"),
        }
    }

    #[test]
    fn test_gpu_app_consecutive_focus_toggle() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Focused(false), None);
        app.handle_window_event(winit::event::WindowEvent::Focused(true), None);
        assert_eq!(received.len(), 2);
        assert!(matches!(received[0], AppEvent::Unfocused));
        assert!(matches!(received[1], AppEvent::Focused));
    }

    #[test]
    fn test_gpu_app_mouse_input_all_buttons() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        let buttons = [
            winit::event::MouseButton::Left,
            winit::event::MouseButton::Right,
            winit::event::MouseButton::Middle,
            winit::event::MouseButton::Back,
            winit::event::MouseButton::Forward,
            winit::event::MouseButton::Other(7),
        ];
        for btn in buttons {
            app.handle_window_event(
                winit::event::WindowEvent::MouseInput {
                    device_id: winit::event::DeviceId::dummy(),
                    state: winit::event::ElementState::Pressed,
                    button: btn,
                },
                None,
            );
        }
        assert_eq!(received.len(), 6);
        match &received[0] {
            AppEvent::MouseInput { button, .. } => assert_eq!(*button, MouseButton::Left),
            _ => panic!("Expected MouseInput"),
        }
        match &received[5] {
            AppEvent::MouseInput { button, .. } => assert_eq!(*button, MouseButton::Other(7)),
            _ => panic!("Expected MouseInput"),
        }
    }

    #[test]
    fn test_gpu_app_touch_all_phases() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        let phases = [
            winit::event::TouchPhase::Started,
            winit::event::TouchPhase::Moved,
            winit::event::TouchPhase::Ended,
            winit::event::TouchPhase::Cancelled,
        ];
        for phase in phases {
            app.handle_window_event(
                winit::event::WindowEvent::Touch(winit::event::Touch {
                    device_id: winit::event::DeviceId::dummy(),
                    phase,
                    location: winit::dpi::PhysicalPosition::new(10.0, 20.0),
                    id: 1,
                    force: None,
                }),
                None,
            );
        }
        assert_eq!(received.len(), 4);
        assert!(matches!(&received[0], AppEvent::Touch(te) if te.phase == TouchPhase::Started));
        assert!(matches!(&received[3], AppEvent::Touch(te) if te.phase == TouchPhase::Cancelled));
    }

    #[test]
    fn test_gpu_app_ime_preedit_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Preedit(
                "abc".to_string(),
                Some((0, 3)),
            )),
            None,
        );
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::Ime(ImeEvent::Preedit { text, cursor }) => {
                assert_eq!(text, "abc");
                assert_eq!(*cursor, Some((0, 3)));
            }
            _ => panic!("Expected Ime Preedit"),
        }
    }

    #[test]
    fn test_gpu_app_mouse_wheel_line_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::MouseWheel {
                device_id: winit::event::DeviceId::dummy(),
                delta: winit::event::MouseScrollDelta::LineDelta(-2.0, 5.0),
                phase: winit::event::TouchPhase::Moved,
            },
            None,
        );
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseWheel { delta } => {
                assert_eq!(*delta, MouseScrollDelta::LineDelta(-2.0, 5.0));
            }
            _ => panic!("Expected MouseWheel"),
        }
    }

    #[test]
    fn test_gpu_app_multiple_resizes() {
        let mut received: Vec<(u32, u32)> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            if let AppEvent::Resized { width, height } = e {
                received.push((width, height));
            }
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(100, 100)),
            None,
        );
        app.handle_window_event(
            winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(200, 200)),
            None,
        );
        assert_eq!(received.len(), 2);
        assert_eq!(received[0], (100, 100));
        assert_eq!(received[1], (200, 200));
    }

    #[test]
    fn test_gpu_app_cursor_moved_large_coords() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(
            winit::event::WindowEvent::CursorMoved {
                device_id: winit::event::DeviceId::dummy(),
                position: winit::dpi::PhysicalPosition::new(99999.0, -99999.0),
            },
            None,
        );
        assert_eq!(received.len(), 1);
        match &received[0] {
            AppEvent::MouseMoved { x, y } => {
                assert!((*x - 99999.0).abs() < f64::EPSILON);
                assert!((*y - (-99999.0)).abs() < f64::EPSILON);
            }
            _ => panic!("Expected MouseMoved"),
        }
    }

    #[test]
    fn test_host_runtime_stores_config() {
        let config = WindowConfig::new("Stored").with_size(400, 300);
        let runtime = HostRuntime::new(config);
        assert_eq!(runtime.config.title, "Stored");
        assert_eq!(runtime.config.width, 400);
        assert_eq!(runtime.config.height, 300);
    }
}
