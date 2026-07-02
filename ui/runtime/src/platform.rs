//! 平台运行时抽象（spec IF-006 `PlatformRuntime`）。
//!
//! `ui/adapters/winit` 提供具体实现；本 trait 的公共 API **不**暴露 winit 类型
//! （spec 技术约束：runtime/platform/gestures/navigation/overlay 不得向 widgets 泄漏 winit 类型）。

use crate::app::UiApp;
use thiserror::Error;
use zero_ui_core::geometry::Rect;
use zero_ui_core::theme::SystemThemeSnapshot;

/// 窗口标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u64);

/// 运行时错误。
#[derive(Debug, Clone, Error)]
pub enum RuntimeError {
    #[error("platform error: {0}")]
    Platform(String),
    #[error("window not found: {0}")]
    WindowNotFound(u64),
}

pub type UiResult<T> = Result<T, RuntimeError>;

/// 平台运行时（spec IF-006）。
pub trait PlatformRuntime {
    fn run(&mut self, app: &mut dyn UiApp) -> UiResult<()>;
    fn request_redraw(&mut self, window: WindowId);
    fn set_ime_area(&mut self, window: WindowId, rect: Option<Rect>);
    fn system_theme(&self) -> SystemThemeSnapshot;
}
