//! 绘制命令生成 — 将布局盒树转换为渲染图元。

mod border_shape;
mod clip;
mod color;
mod gradient;
mod helpers;
pub(crate) mod painter;
pub(crate) mod svg_displacement_scale;
pub(crate) mod svg_filter_chain;
pub(crate) mod svg_filter_functions;
pub(crate) mod svg_filter_taint;
pub(crate) mod svg_shape_disable;
pub(crate) mod svg_style_sanitize;
mod transform;

// Re-export all public items so `pub use paint::*` in lib.rs works unchanged.
pub use color::*;
pub use helpers::*;
pub use painter::*;

#[cfg(test)]
mod tests;
