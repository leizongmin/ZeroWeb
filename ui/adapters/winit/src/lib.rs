//! # zero-ui-adapter-winit
//!
//! winit 平台适配器（spec §8.4.1 `zero-ui-adapter-winit` / IF-006 / FR-010）。
//!
//! 职责：把 winit 原始事件转换为浏览器无关的 [`zero_ui_core::event::UiEvent`]，
//! 并为 [`zero_ui_runtime::PlatformRuntime`] 提供 winit 后端实现。
//!
//! **关键约束**：本 crate 是 winit 类型的**唯一**落点；`ui-runtime`/`ui-platform`/
//! `ui-gestures` 等的公共 API 不得暴露 winit 类型（spec §6.4 技术约束）。
//!
//! M1 skeleton：鼠标按钮/修饰键转换 + PlatformRuntime 占位实现；真实事件循环/窗口/surface 在 M2/M4。

pub mod event_map;
pub mod runtime;

pub use event_map::{map_modifiers, map_mouse_button};
pub use runtime::WinitRuntime;
