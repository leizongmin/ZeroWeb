//! # zero-ui-runtime
//!
//! 通用 UI SDK 运行时（spec §8.4.1 `zero-ui-runtime` / FR-003 / IF-006）。
//!
//! app 生命周期（[`app`]）、调度器（[`scheduler`]）、UI 树运行态（[`tree`]）、
//! 平台运行时抽象（[`platform`]，不泄漏 winit 类型）、主题提供者（[`theme_provider`]）、
//! i18n 运行时（[`i18n_provider`]）、IME 控制（[`ime`]）、无障碍树（[`accessibility`]）。
//!
//! M1 提供接口与最小可测实现；具体事件循环/窗口后端在 `ui/adapters/winit`（M2/M4）。

pub mod accessibility;
pub mod app;
pub mod i18n_provider;
pub mod ime;
pub mod platform;
pub mod scheduler;
pub mod theme_provider;
pub mod tree;

pub use app::UiApp;
pub use i18n_provider::I18nRuntime;
pub use ime::ImeController;
pub use platform::{PlatformRuntime, RuntimeError, UiResult, WindowId};
pub use theme_provider::ThemeProvider;
pub use tree::UiTree;
