//! # zero-canvas
//!
//! Canvas 2D 实现 — 渲染目标、路径、图像数据。

#![warn(missing_docs)]

pub mod context;
pub mod path;

// 测试模块仅在测试时编译
#[cfg(test)]
mod path_tests;

pub use context::*;
pub use path::*;
