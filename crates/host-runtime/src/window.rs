//! 窗口管理 — 创建和管理跨平台窗口

use crate::event::AppEvent;
use crate::{HostError, HostResult};

/// 窗口配置
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

    /// 运行事件循环
    ///
    /// 创建窗口并进入事件循环。回调函数在每次事件时被调用。
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

        // 事件循环 — 窗口在 resumed 回调中创建
        event_loop
            .run_app(&mut App {
                window_attrs: Some(window_attrs),
                window: None,
                on_event: &mut on_event,
            })
            .map_err(|e| HostError::EventLoopError(e.to_string()))?;

        Ok(())
    }
}

/// winit ApplicationHandler trait 实现
struct App<'a, F> {
    window_attrs: Option<winit::window::WindowAttributes>,
    window: Option<winit::window::Window>,
    on_event: &'a mut F,
}

impl<F: FnMut(AppEvent)> winit::application::ApplicationHandler<()> for App<'_, F> {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        if self.window.is_none() && let Some(attrs) = self.window_attrs.take() {
            self.window = Some(
                event_loop
                    .create_window(attrs)
                    .expect("Failed to create window"),
            );
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
            }
            winit::event::WindowEvent::Focused(true) => {
                (self.on_event)(AppEvent::Focused);
            }
            winit::event::WindowEvent::Focused(false) => {
                (self.on_event)(AppEvent::Unfocused);
            }
            _ => {}
        }
    }
}

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
}
