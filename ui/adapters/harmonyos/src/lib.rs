//! # zero-ui-adapter-harmonyos
//!
//! HarmonyOS 平台适配器（spec §8.4.1 / IF-006 / FR-010 / M4 HarmonyOS 决策）。
//!
//! 职责：把 OHOS 原始事件（触摸、键盘、IME、safe area、软键盘、back gesture）转换为
//! 浏览器无关的 [`zero_ui_core::event::UiEvent`]，并为 [`zero_ui_runtime::PlatformRuntime`]
//! 提供 HarmonyOS 后端实现。
//!
//! ## 关键约束
//!
//! - **不依赖 winit**（移动端界面不经过 winit 窗口管理）。
//! - **ArkTS 集成**：Rust 侧通过 [`ffi`] 模块导出 C ABI 函数，供 DevEco Studio Ability
//!   壳工程（ArkTS / `.ets`）调用；ArkTS 负责窗口和 surface 生命周期。
//! - **OHOS 类型不泄漏**：`ui-runtime`/`ui-platform`/`ui-gestures`/`ui-widgets` 等的公共 API
//!   不得暴露 OHOS-specific 类型（与 spec §6.4 技术约束一致）。

pub mod event_map;
pub mod ffi;
pub mod runtime;

pub use event_map::{
    OhosWindowMetricsInput, map_back_gesture, map_key_action, map_key_event, map_soft_keyboard, map_touch_event,
    map_touch_events, map_window_metrics, ohos_touch_phase_to_pointer_phase, system_theme_from_dark_mode,
};
pub use ffi::{RawHarmonyOSEvent, init_runtime};
pub use runtime::{HarmonyOSRuntime, PumpOutcome, RuntimeInner};
