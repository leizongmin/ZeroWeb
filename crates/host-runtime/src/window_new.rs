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
mod window_tests;
