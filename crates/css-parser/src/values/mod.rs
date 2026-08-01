//! CSS 属性值类型。
//!
//! 定义 CSS 属性值的类型化表示，以及解析函数。

pub mod color;
pub mod parse_extended;
pub mod parse_extended_border;
pub mod parse_extended_visual;
pub mod parse_layout;
pub mod parse_misc;
pub mod parse_transform;
pub mod types;

// Re-export all public items.
pub use color::*;
pub use parse_extended::*;
pub use parse_extended_border::*;
pub use parse_extended_visual::*;
pub use parse_layout::*;
pub use parse_misc::*;
pub use parse_transform::*;
pub use types::*;

#[cfg(test)]
mod tests;
