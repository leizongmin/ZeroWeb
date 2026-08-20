//! Shared ZeroWeb renderer role entry points.

mod compositor_publish_thread;
mod error_page;
mod ipc_fetch;
mod ipc_indexed_db;
mod ipc_service_worker;
pub mod js_worker;
#[cfg(target_os = "macos")]
mod macos_app;
mod page_scripts;
mod paint_export;
#[path = "runtime.rs"]
mod runtime;
mod sandbox;
mod script_prefetch;
mod service_worker_host;
mod text_metrics;

pub use runtime::{parse_renderer_launch, run_desktop_role};

// macos_app 经 `super::RendererRuntime` 引用；仅 macOS 编译该模块，故 cfg 门控防 linux dead_code。
#[cfg(target_os = "macos")]
pub(crate) use runtime::RendererRuntime;

#[cfg(target_os = "android")]
pub use runtime::run_android_role;

#[cfg(test)]
#[path = "gpu_isolation_tests.rs"]
mod gpu_isolation_tests;
