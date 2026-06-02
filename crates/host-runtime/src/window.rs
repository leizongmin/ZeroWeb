//! 窗口管理 — 创建和管理跨平台窗口
//!
//! 提供两种事件循环运行模式：
//! - `run()`: 基本模式，回调只接收 `AppEvent`
//! - `run_with_window()`: GPU 模式，回调额外接收 `Arc<Window>` 引用

use crate::event::{
    AppEvent, TouchEvent, convert_ime, convert_keyboard_input, convert_mouse_button, convert_scroll_delta,
    convert_touch_phase,
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
        let event_loop = winit::event_loop::EventLoop::new().map_err(|e| HostError::EventLoopError(e.to_string()))?;

        let window_attrs = winit::window::WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.config.width, self.config.height))
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
        let event_loop = winit::event_loop::EventLoop::new().map_err(|e| HostError::EventLoopError(e.to_string()))?;

        let window_attrs = winit::window::WindowAttributes::default()
            .with_title(&self.config.title)
            .with_inner_size(winit::dpi::LogicalSize::new(self.config.width, self.config.height))
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
    pub(crate) fn new_basic(window_attrs: winit::window::WindowAttributes, on_event: &'a mut F) -> Self {
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
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                (self.on_event)(AppEvent::ScaleFactorChanged { scale_factor });
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
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                (self.on_event)(AppEvent::MouseMoved {
                    x: position.x,
                    y: position.y,
                });
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
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
            let win = event_loop.create_window(attrs).expect("Failed to create window");
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
    pub(crate) fn new_with_window(window_attrs: winit::window::WindowAttributes, on_event: &'a mut F) -> Self {
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
            winit::event::WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                (self.on_event)(AppEvent::ScaleFactorChanged { scale_factor }, win_ref);
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
                (self.on_event)(convert_keyboard_input(device_id, event, is_synthetic), win_ref);
            }
            winit::event::WindowEvent::CursorMoved { position, .. } => {
                (self.on_event)(
                    AppEvent::MouseMoved {
                        x: position.x,
                        y: position.y,
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
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

impl<F: FnMut(AppEvent, Option<Arc<winit::window::Window>>)> winit::application::ApplicationHandler<()>
    for GpuApp<'_, F>
{
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none()
            && let Some(attrs) = self.window_attrs.take()
        {
            let win = event_loop.create_window(attrs).expect("Failed to create window");
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
        let config = WindowConfig::new("Test").with_size(1024, 768).with_resizable(false);
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
        let config = WindowConfig::new("Custom").with_size(1920, 1080).with_resizable(false);
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
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(0, 0)));
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
            delta: winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(10.0, -5.0)),
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
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Enabled));
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Preedit(
            "你好".to_string(),
            Some((0, 2)),
        )));
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Commit(
            "你好世界".to_string(),
        )));
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Disabled));
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
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Preedit(
            String::new(),
            None,
        )));
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
        app.handle_window_event(winit::event::WindowEvent::ThemeChanged(winit::window::Theme::Light));
        app.handle_window_event(winit::event::WindowEvent::Occluded(false));
        assert!(received.is_empty(), "Ignored events should not dispatch");
    }

    // === GpuApp 事件分发测试 ===

    fn make_gpu_app<F: FnMut(AppEvent, Option<Arc<winit::window::Window>>)>(on_event: &'_ mut F) -> GpuApp<'_, F> {
        let attrs = winit::window::WindowAttributes::default();
        GpuApp::new_with_window(attrs, on_event)
    }

    #[test]
    fn test_gpu_app_resized_dispatch() {
        let mut received: Vec<(AppEvent, bool)> = Vec::new();
        let mut callback = |e: AppEvent, w: Option<Arc<winit::window::Window>>| received.push((e, w.is_some()));
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
        let mut callback = |e: AppEvent, w: Option<Arc<winit::window::Window>>| received.push((e, w.is_some()));
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
        let mut callback = |e: AppEvent, w: Option<Arc<winit::window::Window>>| received.push((e, w.is_some()));
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::RedrawRequested, None);
        assert_eq!(received.len(), 1);
        assert!(matches!(received[0].0, AppEvent::RedrawRequested));
    }

    #[test]
    fn test_gpu_app_close_dispatch() {
        let mut received: Vec<(AppEvent, bool)> = Vec::new();
        let mut callback = |e: AppEvent, w: Option<Arc<winit::window::Window>>| received.push((e, w.is_some()));
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
                delta: winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(5.0, -3.0)),
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
        assert!(received.is_empty(), "Ignored events should not dispatch");
    }

    #[test]
    fn test_gpu_app_full_ime_lifecycle() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);
        // Simulate full IME lifecycle: enabled -> preedit -> commit -> disabled
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Enabled), None);
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Preedit("n".to_string(), Some((1, 1)))),
            None,
        );
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Preedit("ni".to_string(), Some((2, 2)))),
            None,
        );
        app.handle_window_event(
            winit::event::WindowEvent::Ime(winit::event::Ime::Commit("你".to_string())),
            None,
        );
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Disabled), None);
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
        let config = WindowConfig::new("CloneTest").with_size(640, 480).with_resizable(false);
        let cloned = config.clone();
        assert_eq!(cloned.title, "CloneTest");
        assert_eq!(cloned.width, 640);
        assert_eq!(cloned.height, 480);
        assert!(!cloned.resizable);
    }

    #[test]
    fn test_window_config_builder_chaining_preserves_all() {
        let config = WindowConfig::new("Chain").with_size(1280, 720).with_resizable(true);
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
        let app = make_basic_app(&mut callback);
        assert!(app.window.is_none());
    }

    #[test]
    fn test_basic_app_window_attrs_initially_some() {
        let mut callback = |_: AppEvent| {};
        let app = make_basic_app(&mut callback);
        assert!(app.window_attrs.is_some());
    }

    // -- BasicApp event dispatch edge cases --

    #[test]
    fn test_basic_app_resized_large() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            7680, 4320,
        )));
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
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            100, 100,
        )));
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            200, 200,
        )));
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            300, 300,
        )));
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
            delta: winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(-100.0, -200.0)),
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
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            500, 400,
        )));
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
        let app = make_gpu_app(&mut callback);
        assert!(app.window.is_none());
    }

    #[test]
    fn test_gpu_app_window_attrs_initially_some() {
        let mut callback = |_: AppEvent, _: Option<Arc<winit::window::Window>>| {};
        let app = make_gpu_app(&mut callback);
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
            winit::event::WindowEvent::Ime(winit::event::Ime::Preedit("abc".to_string(), Some((0, 3)))),
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

    /// 验证窗口 resize 后回调中维护的状态保持一致性。
    ///
    /// 模拟窗口从 800x600 resize 到 1024x768 再到 400x300，
    /// 回调中累积所有尺寸变更事件，验证：
    /// 1. 每次事件携带正确的尺寸
    /// 2. 事件数量与 resize 次数匹配
    /// 3. 最终记录的尺寸为最后一次 resize 的值
    #[test]
    fn test_window_resize_preserves_state() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| {
            received.push(e);
        };
        let mut app = make_basic_app(&mut callback);

        // 第一次 resize：800x600
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            800, 600,
        )));

        // 第二次 resize：1024x768
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            1024, 768,
        )));

        // 第三次 resize：400x300
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            400, 300,
        )));

        // 验证完整历史：3 次 resize，每次尺寸正确
        assert_eq!(received.len(), 3, "应记录 3 次 resize 事件");

        match &received[0] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 800);
                assert_eq!(*height, 600);
            }
            _ => panic!("第 1 个事件应为 Resized"),
        }
        match &received[1] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 1024);
                assert_eq!(*height, 768);
            }
            _ => panic!("第 2 个事件应为 Resized"),
        }
        match &received[2] {
            AppEvent::Resized { width, height } => {
                assert_eq!(*width, 400, "最终 resize 宽度应为 400");
                assert_eq!(*height, 300, "最终 resize 高度应为 300");
            }
            _ => panic!("第 3 个事件应为 Resized"),
        }
    }

    /// Window minimize/maximize event fields.
    /// 验证窗口 resize 到 0x0（最小化）和恢复/最大化时事件字段正确。
    #[test]
    fn test_window_event_min_max() {
        // 模拟最小化：resize 到 0x0
        let mut received: Vec<(u32, u32)> = Vec::new();
        let mut callback = |e: AppEvent| {
            if let AppEvent::Resized { width, height } = e {
                received.push((width, height));
            }
        };
        let mut app = make_basic_app(&mut callback);

        // 最小化：0x0
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(0, 0)));
        // 最大化：1920x1080
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            1920, 1080,
        )));
        // 恢复：800x600
        app.handle_window_event(winit::event::WindowEvent::Resized(winit::dpi::PhysicalSize::new(
            800, 600,
        )));

        assert_eq!(received.len(), 3, "应收到 3 个 Resized 事件");
        assert_eq!(received[0], (0, 0), "最小化尺寸应为 0x0");
        assert_eq!(received[1], (1920, 1080), "最大化尺寸应为 1920x1080");
        assert_eq!(received[2], (800, 600), "恢复尺寸应为 800x600");
    }

    /// 验证 WindowConfig::with_size 链式调用多次时，最后一次调用生效。
    /// 模拟用户先设置 800x600 再覆盖为 1024x768，最终尺寸应为 1024x768。
    #[test]
    fn test_window_config_with_size_chained_overwrite() {
        let config = WindowConfig::new("Overwrite").with_size(800, 600).with_size(1024, 768);
        assert_eq!(config.width, 1024, "最终宽度应为 1024，后设置的值应覆盖前值");
        assert_eq!(config.height, 768, "最终高度应为 768，后设置的值应覆盖前值");
    }

    /// 验证 GpuApp 分发 CloseRequested 事件时，回调同时接收到事件和窗口引用。
    /// 由于测试环境未创建实际窗口，窗口引用应为 None。
    #[test]
    fn test_gpu_app_close_dispatch_with_window_ref() {
        let mut received: Vec<(AppEvent, bool)> = Vec::new();
        let mut callback = |e: AppEvent, w: Option<Arc<winit::window::Window>>| {
            received.push((e, w.is_some()));
        };
        let mut app = make_gpu_app(&mut callback);
        app.handle_window_event(winit::event::WindowEvent::CloseRequested, None);
        assert_eq!(received.len(), 1, "应收到 1 个 CloseRequested 事件");
        assert!(
            matches!(received[0].0, AppEvent::CloseRequested),
            "事件应为 CloseRequested"
        );
        assert!(!received[0].1, "未创建窗口时，窗口引用应为 None");
    }

    /// 验证通过 BasicApp 分发的触摸事件在 u64::MAX 极端 id 值下正确传递。
    /// 模拟高 id 触摸点的 Started 和 Ended 阶段，确保 id 不被截断或溢出。
    #[test]
    fn test_basic_app_touch_max_id_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);

        // 按下阶段
        app.handle_window_event(winit::event::WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            phase: winit::event::TouchPhase::Started,
            location: winit::dpi::PhysicalPosition::new(50.0, 75.0),
            id: u64::MAX,
            force: None,
        }));
        // 释放阶段
        app.handle_window_event(winit::event::WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            phase: winit::event::TouchPhase::Ended,
            location: winit::dpi::PhysicalPosition::new(55.0, 80.0),
            id: u64::MAX,
            force: None,
        }));

        assert_eq!(received.len(), 2, "应收到 2 个 Touch 事件");

        match &received[0] {
            AppEvent::Touch(te) => {
                assert_eq!(te.id, u64::MAX, "触摸点 id 应为 u64::MAX，不应被截断");
                assert_eq!(te.phase, TouchPhase::Started);
                assert!((te.x - 50.0).abs() < f64::EPSILON);
                assert!((te.y - 75.0).abs() < f64::EPSILON);
            }
            _ => panic!("第 1 个事件应为 Touch"),
        }
        match &received[1] {
            AppEvent::Touch(te) => {
                assert_eq!(te.id, u64::MAX, "释放事件的触摸点 id 应保持 u64::MAX");
                assert_eq!(te.phase, TouchPhase::Ended);
                assert!((te.x - 55.0).abs() < f64::EPSILON);
                assert!((te.y - 80.0).abs() < f64::EPSILON);
            }
            _ => panic!("第 2 个事件应为 Touch"),
        }
    }

    /// 验证通过 BasicApp 分发的鼠标滚轮事件在 PixelDelta 极端 f64 值下正确传递。
    /// 使用 f64::MAX 和 f64::MIN 模拟极端滚动增量，确保不丢失精度。
    #[test]
    fn test_basic_app_mouse_wheel_pixel_extreme_delta() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);

        app.handle_window_event(winit::event::WindowEvent::MouseWheel {
            device_id: winit::event::DeviceId::dummy(),
            delta: winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(f64::MAX, f64::MIN)),
            phase: winit::event::TouchPhase::Moved,
        });

        assert_eq!(received.len(), 1, "应收到 1 个 MouseWheel 事件");
        match &received[0] {
            AppEvent::MouseWheel { delta } => {
                assert_eq!(
                    *delta,
                    MouseScrollDelta::PixelDelta(f64::MAX, f64::MIN),
                    "极端 PixelDelta 值应精确传递"
                );
            }
            _ => panic!("应为 MouseWheel 事件"),
        }
    }

    /// 验证 BasicApp 对多种未处理窗口事件的静默忽略行为。
    /// 这些事件不在 handle_window_event 的 match 分支中（落入 `_ => {}`），
    /// 分发后回调不应收到任何事件，确保未处理事件不触发回调。
    /// 忽略事件后，正常事件仍能正确分发，验证忽略不影响后续处理。
    #[test]
    fn test_basic_app_unhandled_events_silently_ignored() {
        // 阶段 1：验证未处理事件不产生回调
        {
            let mut received: Vec<AppEvent> = Vec::new();
            let mut callback = |e: AppEvent| received.push(e);
            let mut app = make_basic_app(&mut callback);

            // 以下事件均不在 handle_window_event 的 match 分支中
            app.handle_window_event(winit::event::WindowEvent::Destroyed);
            app.handle_window_event(winit::event::WindowEvent::ThemeChanged(winit::window::Theme::Light));
            app.handle_window_event(winit::event::WindowEvent::Occluded(false));

            assert!(received.is_empty(), "未处理的事件应被静默忽略，不应产生任何回调事件");
        }

        // 阶段 2：验证忽略事件后，正常事件仍能正确分发
        {
            let mut received: Vec<AppEvent> = Vec::new();
            let mut callback = |e: AppEvent| received.push(e);
            let mut app = make_basic_app(&mut callback);

            app.handle_window_event(winit::event::WindowEvent::Destroyed);
            app.handle_window_event(winit::event::WindowEvent::Focused(true));

            assert_eq!(received.len(), 1, "忽略未处理事件后，正常事件应能正确分发");
            assert!(matches!(received[0], AppEvent::Focused), "应为 Focused 事件");
        }
    }

    /// 验证 GpuApp 分发 IME Enabled/Disabled 事件时，事件和窗口引用均正确传递。
    /// 模拟输入法激活和停用的完整生命周期，确保 GpuApp 回调接收到正确的 Ime 变体。
    #[test]
    fn test_gpu_app_ime_enabled_disabled_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);

        // IME 激活
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Enabled), None);
        // IME 停用
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Disabled), None);

        assert_eq!(received.len(), 2, "应收到 2 个 IME 事件");
        assert!(
            matches!(&received[0], AppEvent::Ime(ImeEvent::Enabled)),
            "第 1 个事件应为 Ime(Enabled)"
        );
        assert!(
            matches!(&received[1], AppEvent::Ime(ImeEvent::Disabled)),
            "第 2 个事件应为 Ime(Disabled)"
        );
    }

    /// 验证 WindowConfig::with_resizable 连续多次调用时，最后一次调用生效。
    /// 模拟用户先设置 resizable=true 再覆盖为 false 再覆盖为 true，
    /// 最终 resizable 标志应为 true。
    #[test]
    fn test_window_config_with_resizable_chained_overwrite() {
        let config = WindowConfig::new("Resizable")
            .with_resizable(true)
            .with_resizable(false)
            .with_resizable(true);
        assert!(config.resizable, "连续多次 with_resizable 调用后，最终值应为 true");

        let config2 = WindowConfig::new("R2").with_resizable(false).with_resizable(true);
        assert!(config2.resizable, "先 false 后 true，最终应为 true");
    }

    /// 验证 GpuApp 分发鼠标滚轮事件时，LineDelta 为零值的边界情况正确传递。
    /// 零值滚动增量可能在触控板恰好回到静止位置时产生。
    #[test]
    fn test_gpu_app_mouse_wheel_zero_line_delta() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);

        app.handle_window_event(
            winit::event::WindowEvent::MouseWheel {
                device_id: winit::event::DeviceId::dummy(),
                delta: winit::event::MouseScrollDelta::LineDelta(0.0, 0.0),
                phase: winit::event::TouchPhase::Moved,
            },
            None,
        );

        assert_eq!(received.len(), 1, "应收到 1 个 MouseWheel 事件");
        match &received[0] {
            AppEvent::MouseWheel { delta } => {
                assert_eq!(
                    *delta,
                    MouseScrollDelta::LineDelta(0.0, 0.0),
                    "零值 LineDelta 应精确传递"
                );
            }
            _ => panic!("应为 MouseWheel 事件"),
        }
    }

    /// 验证 BasicApp 分发 IME Enabled 和 Disabled 事件的正确性。
    /// 这两个事件变体没有额外数据字段，确保通过分发路径正确传递变体类型。
    #[test]
    fn test_basic_app_ime_enabled_disabled_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);

        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Enabled));
        app.handle_window_event(winit::event::WindowEvent::Ime(winit::event::Ime::Disabled));

        assert_eq!(received.len(), 2, "应收到 2 个 IME 事件");
        assert!(
            matches!(&received[0], AppEvent::Ime(ImeEvent::Enabled)),
            "第 1 个事件应为 Ime(Enabled)"
        );
        assert!(
            matches!(&received[1], AppEvent::Ime(ImeEvent::Disabled)),
            "第 2 个事件应为 Ime(Disabled)"
        );
    }

    /// 验证 WindowConfig::new 能正确存储包含 emoji 和特殊 Unicode 字符的窗口标题。
    /// 测试标题中包含多字节 emoji 序列、零宽字符和混合中日韩字符，
    /// 确保标题字段不会因编码问题而截断或乱码。
    #[test]
    fn test_window_config_title_emoji_and_special_unicode() {
        // 包含 emoji 的标题
        let config = WindowConfig::new("🌐 ZeroWeb 浏览器 🚀");
        assert_eq!(config.title, "🌐 ZeroWeb 浏览器 🚀", "emoji 标题应完整保留");

        // 包含零宽连接符的 emoji 序列（家庭 emoji）
        let family_emoji = "👨‍👩‍👧‍👦";
        let config2 = WindowConfig::new(family_emoji);
        assert_eq!(config2.title, family_emoji, "ZWJ emoji 序列应完整保留");

        // 混合中日韩字符
        let mixed = "日本語テスト한글中文";
        let config3 = WindowConfig::new(mixed);
        assert_eq!(config3.title, mixed, "中日韩混合字符应完整保留");

        // 验证 clone 后标题一致
        let cloned = config.clone();
        assert_eq!(cloned.title, config.title, "clone 后标题应一致");
    }

    /// 验证 ImeEvent::Preedit 在光标范围超出文本长度时的边界行为。
    /// 某些输入法实现可能报告不一致的 cursor 范围（end > text.len()），
    /// 此测试确保 Preedit 变体仍能正确存储和读取这些值。
    #[test]
    fn test_ime_preedit_cursor_beyond_text_length() {
        use crate::event::ImeEvent;

        // 光标范围 (0, 10) 但文本仅 3 字节
        let preedit = ImeEvent::Preedit {
            text: "abc".to_string(),
            cursor: Some((0, 10)),
        };
        if let ImeEvent::Preedit { text, cursor } = &preedit {
            assert_eq!(text, "abc", "文本应为 'abc'");
            assert_eq!(*cursor, Some((0, 10)), "光标范围应保留原始值 (0, 10)");
        } else {
            panic!("Expected Preedit variant");
        }

        // 光标 start > end（逻辑错误值，但结构应能存储）
        let inverted = ImeEvent::Preedit {
            text: "测试".to_string(),
            cursor: Some((5, 2)),
        };
        if let ImeEvent::Preedit { text, cursor } = &inverted {
            assert_eq!(text, "测试");
            assert_eq!(*cursor, Some((5, 2)), "反转的光标范围应原样存储");
        } else {
            panic!("Expected Preedit variant");
        }

        // 通过 winit 转换路径验证极端光标值不丢失
        let converted = crate::event::convert_ime(winit::event::Ime::Preedit("x".to_string(), Some((0, 99))));
        assert_eq!(
            converted,
            ImeEvent::Preedit {
                text: "x".to_string(),
                cursor: Some((0, 99)),
            },
            "通过 winit 转换后极端光标值应保留"
        );
    }

    /// 验证 GpuApp 在分发多个 RedrawRequested 事件时，
    /// 每次回调都能正确接收事件（无窗口时 window 引用始终为 None）。
    /// 模拟连续重绘场景（如动画循环），确保回调不被吞没。
    #[test]
    fn test_gpu_app_consecutive_redraw_events_without_window() {
        let mut received: Vec<(AppEvent, bool)> = Vec::new();
        let mut callback = |e: AppEvent, w: Option<Arc<winit::window::Window>>| {
            received.push((e, w.is_some()));
        };
        let mut app = make_gpu_app(&mut callback);

        // 模拟动画循环中连续 10 次 redraw
        for _ in 0..10 {
            app.handle_window_event(winit::event::WindowEvent::RedrawRequested, None);
        }

        assert_eq!(received.len(), 10, "应收到 10 个 RedrawRequested 事件");
        for (i, (event, has_window)) in received.iter().enumerate() {
            assert!(
                matches!(event, AppEvent::RedrawRequested),
                "第 {} 个事件应为 RedrawRequested",
                i + 1
            );
            assert!(!has_window, "第 {} 个事件：未创建窗口时引用应为 None", i + 1);
        }
    }

    /// 验证 GpuApp 分发 MouseWheel 事件时 PixelDelta 极端值的正确传递。
    /// 使用 (0.0, f64::MAX) 这种不对称极端组合，确保 x 和 y 独立传递无精度损失。
    #[test]
    fn test_gpu_app_mouse_wheel_pixel_delta_asymmetric_extreme() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);

        app.handle_window_event(
            winit::event::WindowEvent::MouseWheel {
                device_id: winit::event::DeviceId::dummy(),
                delta: winit::event::MouseScrollDelta::PixelDelta(winit::dpi::PhysicalPosition::new(0.0, f64::MAX)),
                phase: winit::event::TouchPhase::Moved,
            },
            None,
        );

        assert_eq!(received.len(), 1, "应收到 1 个 MouseWheel 事件");
        match &received[0] {
            AppEvent::MouseWheel { delta } => {
                assert_eq!(
                    *delta,
                    MouseScrollDelta::PixelDelta(0.0, f64::MAX),
                    "不对称极端 PixelDelta 应精确传递"
                );
            }
            _ => panic!("应为 MouseWheel 事件"),
        }
    }

    /// 验证 BasicApp 分发触摸事件时负数坐标的正确传递。
    /// 虽然正常触摸坐标应为正值，但在某些平台坐标系统异常或窗口映射错误时，
    /// 负坐标应能无损传递而非被裁剪为零。
    #[test]
    fn test_basic_app_touch_negative_coordinates_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);

        // 触摸点携带负坐标
        app.handle_window_event(winit::event::WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            phase: winit::event::TouchPhase::Started,
            location: winit::dpi::PhysicalPosition::new(-42.5, -99.75),
            id: 3,
            force: None,
        }));

        assert_eq!(received.len(), 1, "应收到 1 个 Touch 事件");
        match &received[0] {
            AppEvent::Touch(te) => {
                assert_eq!(te.id, 3, "触摸点 id 应为 3");
                assert_eq!(te.phase, TouchPhase::Started, "阶段应为 Started");
                assert!((te.x - (-42.5)).abs() < f64::EPSILON, "x 坐标应为 -42.5，不应被裁剪");
                assert!((te.y - (-99.75)).abs() < f64::EPSILON, "y 坐标应为 -99.75，不应被裁剪");
            }
            _ => panic!("应为 Touch 事件"),
        }
    }

    /// 验证 GpuApp 分发 Other(u16::MAX) 鼠标按钮事件时正确传递。
    /// Other 变体携带 u16 值，测试边界值 u16::MAX 确保不发生截断。
    #[test]
    fn test_gpu_app_mouse_input_other_max_button() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
            received.push(e);
        };
        let mut app = make_gpu_app(&mut callback);

        app.handle_window_event(
            winit::event::WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Other(u16::MAX),
            },
            None,
        );
        app.handle_window_event(
            winit::event::WindowEvent::MouseInput {
                device_id: winit::event::DeviceId::dummy(),
                state: winit::event::ElementState::Released,
                button: winit::event::MouseButton::Other(u16::MAX),
            },
            None,
        );

        assert_eq!(received.len(), 2, "应收到 2 个 MouseInput 事件");
        match &received[0] {
            AppEvent::MouseInput { button, pressed } => {
                assert_eq!(*button, MouseButton::Other(u16::MAX), "按钮应为 Other(u16::MAX)");
                assert!(*pressed, "应为按下状态");
            }
            _ => panic!("第 1 个事件应为 MouseInput"),
        }
        match &received[1] {
            AppEvent::MouseInput { button, pressed } => {
                assert_eq!(
                    *button,
                    MouseButton::Other(u16::MAX),
                    "释放事件的按钮应为 Other(u16::MAX)"
                );
                assert!(!pressed, "应为释放状态");
            }
            _ => panic!("第 2 个事件应为 MouseInput"),
        }
    }

    /// 验证 WindowConfig::new 接受 &str 字面量（impl Into<String> 的零拷贝路径）。
    /// 确保字符串字面量无需显式 .to_string() 即可传入构造函数。
    #[test]
    fn test_window_config_new_from_str_literal() {
        let config = WindowConfig::new("PlainLiteral");
        assert_eq!(config.title, "PlainLiteral", "&str 字面量应通过 Into<String> 正确转换");

        // 验证默认字段不变
        assert_eq!(config.width, 800, "默认宽度应为 800");
        assert_eq!(config.height, 600, "默认高度应为 600");
        assert!(config.resizable, "默认 resizable 应为 true");
    }

    /// 验证 GpuApp 在忽略事件后继续分发现觉焦点事件的正确性。
    /// 某些平台在窗口生命周期中会产生 ScaleFactorChanged 等未处理事件，
    /// 确保这些事件不会影响后续 Focused 事件的分发。
    #[test]
    fn test_gpu_app_ignored_then_focused_dispatch() {
        // 阶段 1：忽略事件不产生回调
        {
            let mut received: Vec<AppEvent> = Vec::new();
            let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
                received.push(e);
            };
            let mut app = make_gpu_app(&mut callback);
            app.handle_window_event(winit::event::WindowEvent::Destroyed, None);
            assert!(received.is_empty(), "Destroyed 事件应被忽略");
        }

        // 阶段 2：忽略事件与正常事件交替分发
        {
            let mut received: Vec<AppEvent> = Vec::new();
            let mut callback = |e: AppEvent, _: Option<Arc<winit::window::Window>>| {
                received.push(e);
            };
            let mut app = make_gpu_app(&mut callback);
            app.handle_window_event(winit::event::WindowEvent::Destroyed, None);
            app.handle_window_event(winit::event::WindowEvent::Focused(true), None);
            app.handle_window_event(winit::event::WindowEvent::Destroyed, None);
            app.handle_window_event(winit::event::WindowEvent::Focused(false), None);

            assert_eq!(received.len(), 2, "应只收到 2 个焦点事件（忽略的事件不产生回调）");
            assert!(matches!(received[0], AppEvent::Focused), "第 1 个应为 Focused");
            assert!(matches!(received[1], AppEvent::Unfocused), "第 2 个应为 Unfocused");
        }
    }

    /// 验证鼠标光标移动事件在 f64 次正规（subnormal）极小坐标值下的精确传递。
    /// 次正规浮点数具有降低的精度特征，某些高 DPI 输入设备可能产生此类值。
    #[test]
    fn test_basic_app_cursor_moved_subnormal_coordinates() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);

        let subnormal_x = f64::from_bits(1u64); // 最小正次正规数 ~4.9e-324
        let subnormal_y = f64::from_bits(0x8000_0000_0000_0001u64); // 最小负次正规数

        app.handle_window_event(winit::event::WindowEvent::CursorMoved {
            device_id: winit::event::DeviceId::dummy(),
            position: winit::dpi::PhysicalPosition::new(subnormal_x, subnormal_y),
        });

        assert_eq!(received.len(), 1, "应收到 1 个 MouseMoved 事件");
        match &received[0] {
            AppEvent::MouseMoved { x, y } => {
                assert_eq!(*x, subnormal_x, "次正规 x 坐标应精确传递");
                assert_eq!(*y, subnormal_y, "次正规 y 坐标应精确传递");
                assert!(x.is_subnormal(), "x 应为次正规数");
                assert!(y.is_subnormal(), "y 应为次正规数");
            }
            _ => panic!("应为 MouseMoved 事件"),
        }
    }

    /// 验证连续快速交替的焦点事件（Focused/Unfocused）不会丢失事件。
    /// 模拟用户快速在窗口间切换的场景，每次切换都应独立产生正确的事件。
    #[test]
    fn test_basic_app_rapid_focus_toggle() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);

        // 模拟快速交替 20 次
        for i in 0..20 {
            let focused = i % 2 == 0;
            app.handle_window_event(winit::event::WindowEvent::Focused(focused));
        }

        assert_eq!(received.len(), 20, "应收到 20 个焦点事件");
        for (i, event) in received.iter().enumerate() {
            let expected_focused = i % 2 == 0;
            if expected_focused {
                assert!(
                    matches!(event, AppEvent::Focused),
                    "第 {} 个事件应为 Focused（偶数索引）",
                    i + 1
                );
            } else {
                assert!(
                    matches!(event, AppEvent::Unfocused),
                    "第 {} 个事件应为 Unfocused（奇数索引）",
                    i + 1
                );
            }
        }
    }

    /// 验证通过 BasicApp 分发触摸事件时，f64::INFINITY 坐标的正确传递。
    /// 某些平台或驱动在触摸屏校准异常时可能报告无穷大坐标，
    /// 确保分发路径不会因无穷值而产生 panic 或数据截断。
    #[test]
    fn test_basic_app_touch_infinity_coordinates_dispatch() {
        let mut received: Vec<AppEvent> = Vec::new();
        let mut callback = |e: AppEvent| received.push(e);
        let mut app = make_basic_app(&mut callback);

        app.handle_window_event(winit::event::WindowEvent::Touch(winit::event::Touch {
            device_id: winit::event::DeviceId::dummy(),
            phase: winit::event::TouchPhase::Moved,
            location: winit::dpi::PhysicalPosition::new(f64::INFINITY, f64::NEG_INFINITY),
            id: 100,
            force: None,
        }));

        assert_eq!(received.len(), 1, "应收到 1 个 Touch 事件");
        match &received[0] {
            AppEvent::Touch(te) => {
                assert_eq!(te.id, 100, "触摸点 id 应为 100");
                assert_eq!(te.phase, TouchPhase::Moved, "阶段应为 Moved");
                assert!(te.x.is_infinite() && te.x.is_sign_positive(), "x 应为正无穷");
                assert!(te.y.is_infinite() && te.y.is_sign_negative(), "y 应为负无穷");
            }
            _ => panic!("应为 Touch 事件"),
        }
    }
}
