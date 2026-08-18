//! Shared ZeroWeb renderer role entry points.

mod compositor_publish_thread;
mod error_page;
mod ipc_fetch;
mod ipc_indexed_db;
pub mod js_worker;
#[cfg(target_os = "macos")]
mod macos_app;
mod page_scripts;
mod paint_export;
#[path = "runtime.rs"]
mod runtime;
mod sandbox;
mod script_prefetch;
mod text_metrics;

pub use runtime::{parse_renderer_launch, run_desktop_role};

#[cfg(test)]
#[path = "gpu_isolation_tests.rs"]
mod gpu_isolation_tests;
