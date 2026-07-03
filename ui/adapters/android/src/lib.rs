//! # zero-ui-adapter-android
//!
//! Android 平台适配器（spec §8.4.1 / IF-006 / FR-010 / M4 Android 决策）。
//!
//! 职责：把 Android 原始事件（触摸、键盘、IME、safe area、软键盘、back gesture）转换为
//! 浏览器无关的 [`zero_ui_core::event::UiEvent`]，并为 [`zero_ui_runtime::PlatformRuntime`]
//! 提供 Android 后端实现。
//!
//! ## 关键约束
//!
//! - **不依赖 winit**（移动端界面不经过 winit 窗口管理）。
//! - **JNI 集成**：Rust 侧通过 [`ffi`] 模块导出 C ABI 函数，供 Android Activity
//!   （Kotlin/Java）通过 `System.loadLibrary("zero_ui_android")` 调用。
//! - **Android 类型不泄漏**：`ui-runtime`/`ui-platform`/`ui-gestures`/`ui-widgets` 等的公共 API
//!   不得暴露 Android-specific 类型（与 spec §6.4 技术约束一致）。
//!
//! ## 与 HarmonyOS adapter 的关系
//!
//! 本 crate 与 [`zero_ui_adapter_harmonyos`] 对位，提供等价的 Android 事件映射和运行时抽象。
//! 两者共享同一套通用 SDK 接口（core/runtime/gestures/platform），差异仅在平台原生事件语义。
//! HarmonyOS 是 M4/DONE 硬指标；Android 是 stretch goal（不阻塞 DONE，见
//! `docs/goal/ui-sdk/decisions/m4-add-android-backend.md`）。

pub mod event_map;
pub mod ffi;
pub mod runtime;

pub use event_map::{
    AndroidWindowMetricsInput, android_touch_action_to_pointer_phase, map_back_gesture, map_key_action, map_key_event,
    map_soft_keyboard, map_touch_event, map_touch_events, map_window_metrics, system_theme_from_dark_mode,
};
pub use ffi::{RawAndroidEvent, init_runtime};
pub use runtime::{AndroidRuntime, PumpOutcome, RuntimeInner};
