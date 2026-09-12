//! 绘制命令生成 — 将布局盒树转换为渲染图元。

mod color;
mod helpers;
pub(crate) mod painter;
pub(crate) mod svg_filter_taint;
pub(crate) mod svg_shape_disable;

// Re-export all public items so `pub use paint::*` in lib.rs works unchanged.
pub use color::*;
pub use helpers::*;
pub use painter::*;

#[cfg(test)]
mod tests;
