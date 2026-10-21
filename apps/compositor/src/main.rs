//! ZeroWeb compositor desktop process entry.

#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

use std::io;

/// 日志级别 env 覆盖（`ZW_LOG_LEVEL`，默认 INFO）——诊断期临时提级（如动画采样
/// debug 观测），生产零变化。
fn configured_log_level() -> tracing::Level {
    std::env::var("ZW_LOG_LEVEL")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(tracing::Level::INFO)
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(configured_log_level())
        .with_writer(io::stderr)
        .init();

    let mut transport = zero_protocol::transport::stdio_transport()
        .unwrap_or_else(|error| panic!("compositor: stdio transport: {error}"));
    zero_compositor::run_role(&mut transport);
}
