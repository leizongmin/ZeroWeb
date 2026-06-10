//! # zero-canvas
//!
//! Canvas 2D 实现 — 渲染目标、路径、图像数据。

#![warn(missing_docs)]
#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_imports))]
#![allow(unused_comparisons)]
#![allow(clippy::len_zero)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::identity_op)]
#![allow(clippy::while_let_on_iterator)]

pub mod context;
pub mod path;

// 测试模块仅在测试时编译
#[cfg(test)]
mod path_tests;

pub use context::*;
pub use path::*;
