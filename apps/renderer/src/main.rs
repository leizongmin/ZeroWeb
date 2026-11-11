//! ZeroWeb renderer desktop process entry.

#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

// T62/G4 alloc-trace：bin 作用域全局分配器——所有请求无条件转发 System
// （分配语义/布局/对齐逐字不变），簿记仅追加在成功之后且受进程快照开关
// 与 TLS 重入守卫双重门控。默认构建不编译本项。
#[cfg(feature = "alloc-trace")]
#[global_allocator]
static GLOBAL_ALLOCATOR: zero_renderer::alloc_trace::TracingAllocator = zero_renderer::alloc_trace::TracingAllocator;

use std::io;

use zero_protocol::ProcessRole;

/// 日志级别 env 覆盖（`ZW_LOG_LEVEL`，默认 INFO）——诊断期临时提级（如动画声明
/// 发送 debug 观测），生产零变化。
fn configured_log_level() -> tracing::Level {
    std::env::var("ZW_LOG_LEVEL")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(tracing::Level::INFO)
}

fn main() {
    // T62/G4 alloc-trace 接线：feature 构建下以 TracingAllocator 为全局分配器
    // （bin 作用域，lib 下游不带），进程最早时机快照 env 开关与 dump 配置
    // （构造期读取范式，热路径无 getenv）；默认构建本调用不存在，产品路径
    // 零变化。启用后所有分配仍无条件走 System，仪器只追加成功后的簿记。
    #[cfg(feature = "alloc-trace")]
    zero_renderer::alloc_trace::init_from_env();

    tracing_subscriber::fmt()
        .with_max_level(configured_log_level())
        .with_writer(io::stderr)
        .with_target(false)
        .init();

    let (role, renderer_id) = zero_renderer::parse_renderer_launch();
    if role != ProcessRole::Renderer {
        tracing::error!("zero-renderer must start with --type=renderer");
        std::process::exit(2);
    }
    tracing::info!("ZeroWeb renderer starting (type=renderer, instance-id={renderer_id})");

    if let Err(error) = zero_renderer::run_desktop_role(renderer_id) {
        tracing::error!("renderer exited with an error: {error}");
        std::process::exit(1);
    }
    // t2-pb1 fix#16：Browser 断连即进程级退出。js worker 可能仍在长臂中（页面脚本
    // 30s timer 回调），Drop→shutdown 的 join 会把退出拖到臂结束（bilibili 实测
    // "renderer exiting" 后挂 10min+ 仍 100% CPU，泄漏的 renderer 持续抢核干扰
    // 后续任务）；renderer 无落盘状态（状态在 browser 进程），Chrome renderer
    // 退出同此语义。
    std::process::exit(0);
}
