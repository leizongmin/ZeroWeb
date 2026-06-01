//! Canvas 2D 渲染上下文 — 实现 CanvasRenderingContext2D API。

mod context_impl;
mod offscreen;
mod raster;
mod types;

// Re-export all public items so `pub use context::*` in lib.rs works unchanged.
pub use offscreen::*;
pub use types::*;

#[cfg(test)]
mod tests;
