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
}
