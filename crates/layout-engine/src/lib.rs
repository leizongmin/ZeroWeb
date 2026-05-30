//! # zero-layout-engine
//!
//! 布局引擎 — 基于 taffy 扩展，支持 Block/Inline/Flexbox/Grid。
//!
//! ## 核心模块
//!
//! - [`types`] — 布局输出类型（LayoutBox、LayoutResult）
//! - [`converter`] — ComputedStyle → taffy::Style 转换
//! - [`tree`] — DOM 树到 taffy 树的构建
//! - [`engine`] — LayoutEngine 协调器

#![warn(missing_docs)]

pub mod types;
pub mod converter;
pub mod tree;
pub mod engine;

pub use types::*;
pub use converter::*;
pub use tree::*;
pub use engine::*;
