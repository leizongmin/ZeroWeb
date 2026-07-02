//! # zero-ui-examples
//!
//! UI SDK 示例应用（DC-14）—— 验证通用 UI SDK 可被**外部程序**复用。
//!
//! 本 crate **不依赖任何浏览器 crate**（`zero-browser-shell`/`zero-webview`/`zero-engine`/`zero-net`），
//! 仅依赖 `ui/core` + `ui/render` + `ui/runtime` + `ui/widgets`。
//!
//! 当前示例：[`counter`](counter) —— retained 运行时闭环（事件→Action→AppState→重建→re-layout/paint）。

pub mod counter;

pub use counter::{CounterApp, Label, register_counter_factories};
