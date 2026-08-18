//! ZeroWeb compositor desktop process entry.

#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

use std::io;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(io::stderr)
        .init();

    let mut transport = zero_protocol::transport::stdio_transport()
        .unwrap_or_else(|error| panic!("compositor: stdio transport: {error}"));
    zero_compositor::run_role(&mut transport);
}
