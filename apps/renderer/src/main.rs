//! ZeroWeb renderer desktop process entry.

#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

use std::io;

use zero_protocol::ProcessRole;

fn main() {
    tracing_subscriber::fmt()
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
