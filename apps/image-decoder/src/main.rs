//! ZeroWeb image-decoder desktop process entry.

#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

use std::io;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(io::stderr)
        .init();

    // fail-closed（goal ④C 拍板）：沙箱安装失败拒绝运行——隔离模式下 renderer 侧
    // 呈现为解码错误面（无内联回落），降级显式可见而非静默裸奔。
    if let Err(error) = zero_image_decoder::sandbox::apply_early_if_enabled() {
        eprintln!("image-decoder: sandbox install failed, refusing to run: {error}");
        std::process::exit(1);
    }

    let mut transport = zero_protocol::transport::stdio_transport()
        .unwrap_or_else(|error| panic!("image-decoder: stdio transport init: {error}"));
    zero_image_decoder::run_role(&mut transport);
}
