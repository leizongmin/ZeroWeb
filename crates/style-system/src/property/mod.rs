//! CSS 属性定义和计算样式结构。
//!
//! 定义 `ComputedStyle` 结构体，包含所有 Tier 1 CSS 属性的 typed 字段，
//! 以及 `PropertyRegistry` 用于查询初始值和继承性。

pub mod apply;
mod apply_advanced;
mod computed_style;
mod default_impl;
pub mod inherit;
pub mod parse;
pub mod registry;
pub mod types;

// Re-export all public items so `pub use property::*` in lib.rs works unchanged.
pub use apply::*;
pub use computed_style::*;
pub use inherit::*;
pub use parse::*;
pub use registry::*;
pub use types::*;

#[cfg(test)]
mod tests;
