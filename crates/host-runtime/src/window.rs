//! 窗口管理 — 创建和管理跨平台窗口
//!
//! 提供两种事件循环运行模式：
//! - `run()`: 基本模式，回调只接收 `AppEvent`
//! - `run_with_window()`: GPU 模式，回调额外接收 `Arc<Window>` 引用

use crate::event::AppEvent;
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
struct BasicApp<'a, F> {
    window_attrs: Option<winit::window::WindowAttributes>,
    window: Option<Arc<winit::window::Window>>,
    on_event: &'a mut F,
}

impl<'a, F: FnMut(AppEvent)> BasicApp<'a, F> {
    fn new_basic(window_attrs: winit::window::WindowAttributes, on_event: &'a mut F) -> Self {
        Self {
            window_attrs: Some(window_attrs),
            window: None,
            on_event,
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
                let event = if focused { AppEvent::Focused } else { AppEvent::Unfocused };
                (self.on_event)(event);
            }
            _ => {}
        }
    }
}

/// GPU 模式事件处理器（提供窗口引用）
struct GpuApp<'a, F> {
    window_attrs: Option<winit::window::WindowAttributes>,
    window: Option<Arc<winit::window::Window>>,
    on_event: &'a mut F,
}

impl<'a, F: FnMut(AppEvent, Option<Arc<winit::window::Window>>)> GpuApp<'a, F> {
    fn new_with_window(
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

impl<F: FnMut(AppEvent, Option<Arc<winit::window::Window>>)> winit::application::ApplicationHandler<()>
    for GpuApp<'_, F>
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
                    if focused { AppEvent::Focused } else { AppEvent::Unfocused },
                    win_ref,
                );
            }
            _ => {}
        }
    }
}

// Re-export winit window type for convenience
pub use winit::window::Window;

#[cfg(test)]
mod tests {
    use super::*;

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
}
