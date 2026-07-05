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
//! M1：鼠标按钮/修饰键转换 + PlatformRuntime 占位；M2：完整 winit 事件→UiEvent
//! 映射（指针/键盘/滚轮/IME/触摸/窗口度量）；M4（本轮）：[`WinitDriver`] —— 把事件循环的
//! 可测试核心（事件→dispatch→reducer→重建→失效→帧）从阻塞的 `EventLoop::run` 中抽出，
//! headless 可验证；真实开窗/首帧仍需 GUI。

pub mod driver;
pub mod event_map;
pub mod runtime;

pub use driver::{EventOutcome, FrameOutcome, WinitDriver};
pub use event_map::{
    LINE_HEIGHT_PX, key_text, map_cursor_moved, map_ime, map_key_action, map_key_event, map_logical_key, map_modifiers,
    map_mouse_button, map_mouse_input, map_mouse_wheel, map_pointer_phase, map_touch, map_touch_phase,
    map_window_metrics, to_logical_point, to_logical_size,
};
pub use runtime::{FontAsset, FontContainer, WinitRuntime};
