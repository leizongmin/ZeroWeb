//! Canvas 2D 渲染上下文 — 实现 CanvasRenderingContext2D API。

mod context_impl;
mod offscreen;
// R3355：raster 模块的 shadow_blur_geom / SHADOW_BLUR_MAX_RADIUS 经 #[cfg(test)] 测试直接断言，
// 故 crate 内可见。生产代码仍经 context_impl 内 super::raster 路径消费。
#[cfg(test)]
pub(crate) mod raster;
#[cfg(not(test))]
mod raster;
mod types;

// Re-export all public items so `pub use context::*` in lib.rs works unchanged.
pub use offscreen::*;
pub use types::*;

#[cfg(test)]
mod tests;
