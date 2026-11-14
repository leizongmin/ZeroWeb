//! Shared ZeroWeb renderer role entry points.

mod compositor_publish_thread;
mod error_page;
mod font_payloads;
mod ipc_fetch;
mod ipc_image_batch;
mod ipc_indexed_db;
mod ipc_service_worker;
pub mod js_worker;
#[cfg(target_os = "macos")]
mod macos_app;
pub mod mem_reclaim;
mod page_scripts;
mod paint_export;
mod query_overrides;
#[path = "runtime.rs"]
mod runtime;
mod sandbox;
mod script_prefetch;
mod service_worker_host;
mod text_metrics;
// M3-S1a：页面真线程 worker 宿主（ZW_THREADED_WORKERS flag 门控装填）。
mod worker_host;

pub use runtime::{parse_renderer_launch, run_desktop_role};
// 生产编码入口导出：集成测试钉住「renderer 编码 → IPC 图元快照 → browser 解码」
// 边界保真（多进程/单进程一致性常驻断言，见 tests/integration clearfix_multiprocess_parity）。
pub use paint_export::paint_snapshot_from_primitives;

// macos_app 经 `super::RendererRuntime` 引用；仅 macOS 编译该模块，故 cfg 门控防 linux dead_code。
#[cfg(target_os = "macos")]
pub(crate) use runtime::RendererRuntime;

#[cfg(target_os = "android")]
pub use runtime::run_android_role;

// T62/G4 先导分配仪器：默认不编译（非默认 feature `alloc-trace`），产品构建
// 零代码、零数据、分配路径不变；接线（#[global_allocator] + env 快照初始化）
// 在 bin 入口 main.rs——bin 作用域，lib 下游与 lib 测试不带包装分配器。
#[cfg(feature = "alloc-trace")]
pub mod alloc_trace;

#[cfg(test)]
#[path = "gpu_isolation_tests.rs"]
mod gpu_isolation_tests;

#[cfg(test)]
#[path = "identity_bridge_tests_s48.rs"]
mod identity_bridge_tests_s48;
