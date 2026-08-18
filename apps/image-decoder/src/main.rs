//! ZeroWeb image-decoder desktop process entry.

#![cfg_attr(all(windows, not(test)), windows_subsystem = "windows")]

use std::io;

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .with_writer(io::stderr)
        .init();

    let mut transport = zero_protocol::transport::stdio_transport()
        .unwrap_or_else(|error| panic!("image-decoder: stdio transport init: {error}"));
    zero_image_decoder::run_role(&mut transport);
}
