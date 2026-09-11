//! WebView 测试模块。

mod advanced;
mod basic;
mod builder_nav;
mod cache_storage;
mod coverage;
mod coverage_improvements;
mod edge;
mod even_more_coverage;
mod event_dispatch;
mod final_coverage;
mod indexed_db_owner;
mod integration;
mod more_coverage;
mod opfs_owner;
mod service_worker_fetch;
mod service_worker_iframe;
mod service_worker_runtime;
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
