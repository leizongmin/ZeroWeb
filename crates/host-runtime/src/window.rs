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
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, Default)]
struct PointerTracker {
    x: f64,
    y: f64,
}

impl PointerTracker {
    fn set(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    fn coords(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum UnixBackendPreference {
    X11,
    Wayland,
}

#[cfg(target_os = "linux")]
fn parse_unix_backend_preference(raw: &str) -> Option<UnixBackendPreference> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "x11" => Some(UnixBackendPreference::X11),
        "wayland" => Some(UnixBackendPreference::Wayland),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn unix_backend_preference_from_env() -> Option<UnixBackendPreference> {
    std::env::var("WINIT_UNIX_BACKEND")
        .ok()
        .as_deref()
        .and_then(parse_unix_backend_preference)
}

fn build_event_loop() -> HostResult<winit::event_loop::EventLoop<()>> {
    let mut builder = winit::event_loop::EventLoop::builder();

    #[cfg(target_os = "linux")]
    {
        match unix_backend_preference_from_env() {
            Some(UnixBackendPreference::X11) => {
                use winit::platform::x11::EventLoopBuilderExtX11;

                tracing::info!("Forcing winit backend: x11");
                builder.with_x11();
            }
            Some(UnixBackendPreference::Wayland) => {
                use winit::platform::wayland::EventLoopBuilderExtWayland;

                tracing::info!("Forcing winit backend: wayland");
                builder.with_wayland();
            }
            None => {}
        }
    }

    builder.build().map_err(|e| HostError::EventLoopError(e.to_string()))
}

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
    /// 是否显示系统窗口装饰
    pub decorations: bool,
    /// 启动时是否全屏（无边框全屏）
    pub fullscreen: bool,
    /// 启动时是否最大化
    pub maximized: bool,
    /// macOS：透明标题栏 + 全尺寸内容视图（自绘标签栏与系统 traffic lights 同排）
    pub unified_titlebar: bool,
}

impl WindowConfig {
    /// 创建默认窗口配置
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            width: 800,
            height: 600,
            resizable: true,
            decorations: true,
            fullscreen: false,
            maximized: false,
            unified_titlebar: false,
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

    /// 设置是否显示系统窗口装饰
    pub fn with_decorations(mut self, decorations: bool) -> Self {
        self.decorations = decorations;
        self
    }

    /// 设置启动时是否全屏
    pub fn with_fullscreen(mut self, fullscreen: bool) -> Self {
        self.fullscreen = fullscreen;
        self
    }

    /// 设置启动时是否最大化
    pub fn with_maximized(mut self, maximized: bool) -> Self {
        self.maximized = maximized;
        self
    }

    /// macOS 一体化标题栏（系统 traffic lights + 自绘标签栏同排）
    pub fn with_unified_titlebar(mut self, unified_titlebar: bool) -> Self {
        self.unified_titlebar = unified_titlebar;
        self
    }
}

fn window_attributes_from_config(config: &WindowConfig) -> winit::window::WindowAttributes {
    let mut attrs = winit::window::WindowAttributes::default()
        .with_title(&config.title)
        .with_inner_size(winit::dpi::LogicalSize::new(config.width, config.height))
        .with_resizable(config.resizable)
        .with_decorations(config.decorations);
    // Windows / Linux：设置任务栏与标题栏图标。macOS 由 .app bundle 的 .icns 提供。
    if let Some(icon) = crate::app_icon::window_icon() {
        attrs = attrs.with_window_icon(Some(icon));
    }
    if config.fullscreen {
        attrs = attrs.with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
    }
    if config.maximized {
        attrs = attrs.with_maximized(true);
    }
    #[cfg(target_os = "macos")]
    if config.unified_titlebar {
        use winit::platform::macos::WindowAttributesExtMacOS;
        // 勿用 with_titlebar_hidden：会切到 Borderless，系统红黄绿按钮会消失。
        attrs = attrs
            .with_titlebar_transparent(true)
            .with_title_hidden(true)
            .with_fullsize_content_view(true);
    }
    attrs
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
        let event_loop = build_event_loop()?;

        let window_attrs = window_attributes_from_config(&self.config);

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
        let event_loop = build_event_loop()?;

        let window_attrs = window_attributes_from_config(&self.config);

        event_loop
            .run_app(&mut GpuApp::new_with_window(window_attrs, &mut on_event))
            .map_err(|e| HostError::EventLoopError(e.to_string()))?;

        Ok(())
    }

    /// 运行事件循环，并在 `poll_active` 为真时按指定间隔派发 [`AppEvent::Poll`]。
    ///
    /// Poll 事件仅用于消费后台 IPC/网络结果，不会自行触发窗口重绘。
    pub fn run_with_window_polling<F>(
        self,
        poll_interval: Duration,
        poll_active: Arc<AtomicBool>,
        mut on_event: F,
    ) -> HostResult<()>
    where
        F: FnMut(AppEvent, Option<Arc<winit::window::Window>>) + 'static,
    {
        let event_loop = build_event_loop()?;
        let window_attrs = window_attributes_from_config(&self.config);
        let mut app = GpuApp::new_with_window(window_attrs, &mut on_event);
        app.polling = Some((poll_interval, poll_active));

        event_loop
            .run_app(&mut app)
            .map_err(|e| HostError::EventLoopError(e.to_string()))?;

        Ok(())
    }
}

/// 基本模式事件处理器
pub(crate) struct BasicApp<'a, F> {
    window_attrs: Option<winit::window::WindowAttributes>,
    window: Option<Arc<winit::window::Window>>,
    on_event: &'a mut F,
    pointer: PointerTracker,
}

impl<'a, F: FnMut(AppEvent)> BasicApp<'a, F> {
    /// 创建基本模式事件处理器（用于测试）
    pub(crate) fn new_basic(window_attrs: winit::window::WindowAttributes, on_event: &'a mut F) -> Self {
        Self {
            window_attrs: Some(window_attrs),
            window: None,
            on_event,
            pointer: PointerTracker::default(),
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
                self.pointer.set(position.x, position.y);
                (self.on_event)(AppEvent::MouseMoved {
                    x: position.x,
                    y: position.y,
                });
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let (x, y) = self.pointer.coords();
                (self.on_event)(AppEvent::MouseInput {
                    button: convert_mouse_button(button),
                    pressed: state == winit::event::ElementState::Pressed,
                    x,
                    y,
                });
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = self.pointer.coords();
                (self.on_event)(AppEvent::MouseWheel {
                    delta: convert_scroll_delta(delta),
                    x,
                    y,
                });
            }
            winit::event::WindowEvent::PanGesture { delta, .. } => {
                let (x, y) = self.pointer.coords();
                (self.on_event)(AppEvent::PanGesture {
                    delta_x: delta.x,
                    delta_y: delta.y,
                    x,
                    y,
                });
            }
            winit::event::WindowEvent::Touch(touch) => {
                self.pointer.set(touch.location.x, touch.location.y);
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
            winit::event::WindowEvent::ThemeChanged(theme) => {
                (self.on_event)(AppEvent::ThemeChanged {
                    dark: theme == winit::window::Theme::Dark,
                });
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
            let win = Arc::new(win);
            win.request_redraw();
            self.window = Some(win);
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
    pointer: PointerTracker,
    polling: Option<(Duration, Arc<AtomicBool>)>,
}

impl<'a, F: FnMut(AppEvent, Option<Arc<winit::window::Window>>)> GpuApp<'a, F> {
    /// 创建 GPU 模式事件处理器（用于测试）
    pub(crate) fn new_with_window(window_attrs: winit::window::WindowAttributes, on_event: &'a mut F) -> Self {
        Self {
            window_attrs: Some(window_attrs),
            window: None,
            on_event,
            pointer: PointerTracker::default(),
            polling: None,
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
                self.pointer.set(position.x, position.y);
                (self.on_event)(
                    AppEvent::MouseMoved {
                        x: position.x,
                        y: position.y,
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => {
                let (x, y) = self.pointer.coords();
                (self.on_event)(
                    AppEvent::MouseInput {
                        button: convert_mouse_button(button),
                        pressed: state == winit::event::ElementState::Pressed,
                        x,
                        y,
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::MouseWheel { delta, .. } => {
                let (x, y) = self.pointer.coords();
                (self.on_event)(
                    AppEvent::MouseWheel {
                        delta: convert_scroll_delta(delta),
                        x,
                        y,
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::PanGesture { delta, .. } => {
                let (x, y) = self.pointer.coords();
                (self.on_event)(
                    AppEvent::PanGesture {
                        delta_x: delta.x,
                        delta_y: delta.y,
                        x,
                        y,
                    },
                    win_ref,
                );
            }
            winit::event::WindowEvent::Touch(touch) => {
                self.pointer.set(touch.location.x, touch.location.y);
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
            winit::event::WindowEvent::ThemeChanged(theme) => {
                (self.on_event)(
                    AppEvent::ThemeChanged {
                        dark: theme == winit::window::Theme::Dark,
                    },
                    win_ref,
                );
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
            let win = Arc::new(win);
            win.request_redraw();
            self.window = Some(win);
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

    fn about_to_wait(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let Some((interval, active)) = self.polling.as_ref() else {
            return;
        };
        if !active.load(Ordering::Acquire) {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
            return;
        }

        (self.on_event)(AppEvent::Poll, self.window.clone());
        if active.load(Ordering::Acquire) {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(Instant::now() + *interval));
        } else {
            event_loop.set_control_flow(winit::event_loop::ControlFlow::Wait);
        }
    }
}

// Re-export winit window type for convenience
pub use winit::window::Window;

#[cfg(test)]
#[path = "window_tests.rs"]
mod tests;

#[cfg(test)]
mod inline_tests {
    use super::*;

    // ── WindowConfig builder ────────────────────────────────────────────

    #[test]
    fn test_window_config_default_values() {
        let config = WindowConfig::new("Test");
        assert_eq!(config.title, "Test");
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert!(config.resizable);
        assert!(config.decorations);
    }

    #[test]
    fn test_window_config_with_size() {
        let config = WindowConfig::new("Test").with_size(1024, 768);
        assert_eq!(config.width, 1024);
        assert_eq!(config.height, 768);
    }

    #[test]
    fn test_window_config_with_resizable_false() {
        let config = WindowConfig::new("Test").with_resizable(false);
        assert!(!config.resizable);
    }

    #[test]
    fn test_window_config_with_decorations_false() {
        let config = WindowConfig::new("Test").with_decorations(false);
        assert!(!config.decorations);
    }

    #[test]
    fn test_window_config_builder_chain() {
        let config = WindowConfig::new("My App")
            .with_size(1920, 1080)
            .with_resizable(false)
            .with_decorations(false);
        assert_eq!(config.title, "My App");
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert!(!config.resizable);
        assert!(!config.decorations);
    }

    #[test]
    fn test_window_config_clone() {
        let config = WindowConfig::new("Test").with_size(640, 480);
        let cloned = config.clone();
        assert_eq!(cloned.title, config.title);
        assert_eq!(cloned.width, config.width);
        assert_eq!(cloned.height, config.height);
        assert_eq!(cloned.resizable, config.resizable);
        assert_eq!(cloned.decorations, config.decorations);
    }

    #[test]
    fn test_host_runtime_new() {
        let config = WindowConfig::new("Test");
        let _runtime = HostRuntime::new(config);
    }

    #[test]
    fn test_window_config_empty_title() {
        let config = WindowConfig::new("");
        assert_eq!(config.title, "");
    }

    #[test]
    fn test_window_config_zero_size() {
        let config = WindowConfig::new("Test").with_size(0, 0);
        assert_eq!(config.width, 0);
        assert_eq!(config.height, 0);
    }
}
