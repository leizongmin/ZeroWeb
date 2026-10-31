//! WebView 测试模块。

mod advanced;
mod async_drain_bound;
mod basic;
mod builder_nav;
mod cache_storage;
mod coverage;
mod coverage_improvements;
mod csp_gate;
mod edge;
mod even_more_coverage;
mod event_dispatch;
mod execution_timing_css;
mod load_contiguous_advance;

/// T32：进程级 env 串行锁——`ZW_LOAD_CONTIGUOUS_ADVANCE` 用例互斥（进程级全局
/// 资源跨用例串行化纪律）。
///
/// DF-F3 口径：与 `async_load` 模块既有的 `ENV_TEST_LOCK` 是两把独立锁（后者
/// 覆盖 async_load 自身 env 用例的断言面，先于本纪律存在）。持本锁改 env 的窗口
/// 内，async_load 用例仍可能并发读 env——但其用例持自有锁且断言的是各自分支
/// 行为，读到的值无论新旧都回落有效路径（连续推进或逐拍），断言安全。
pub(crate) fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}
// s174（html-semantics-compat M2）：静态外链 CSS entries 化——级联文档序钉测。
mod external_css_cascade_order;
mod final_coverage;
mod indexed_db_owner;
mod integration;
// security-hardening M3：Mixed Content 分级阻止 + HSTS 响应注册（kill-switch 面）。
mod mixed_content_gate;
mod more_coverage;
mod opfs_owner;
// T67（TV-1 遗留）：打印设置 → 渲染输出几何的程序化断言（webview render 入口级）。
mod print_setup_geometry;
mod service_worker_fetch;
mod service_worker_iframe;
mod service_worker_runtime;
mod stylesheet_urls;
mod uncovered_paths;
mod user_actions;
// event-loop-spec M2 MO-S1：host 侧 mutation 通知排空（双引擎 native 绑定域——fragment
// 用例单测 v8 门控，quickjs 绑定面缺 create_document_fragment 工厂）。
#[cfg(any(feature = "v8", feature = "quickjs"))]
mod mo_host_trigger;
#[cfg(feature = "v8")]
mod wasm_bridge;
mod webview_coverage_final;
mod worker_integration;
