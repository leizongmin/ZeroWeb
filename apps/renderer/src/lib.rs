//! Shared ZeroWeb renderer role entry points.

mod compositor_publish_thread;
mod error_page;
mod font_payloads;
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
// 生产编码入口导出：集成测试钉住「renderer 编码 → IPC 图元快照 → browser 解码」
// 边界保真（多进程/单进程一致性常驻断言，见 tests/integration clearfix_multiprocess_parity）。
pub use paint_export::paint_snapshot_from_primitives;

// macos_app 经 `super::RendererRuntime` 引用；仅 macOS 编译该模块，故 cfg 门控防 linux dead_code。
#[cfg(target_os = "macos")]
pub(crate) use runtime::RendererRuntime;

#[cfg(target_os = "android")]
pub use runtime::run_android_role;

#[cfg(test)]
#[path = "gpu_isolation_tests.rs"]
mod gpu_isolation_tests;

#[cfg(test)]
#[path = "identity_bridge_tests_s48.rs"]
mod identity_bridge_tests_s48;
