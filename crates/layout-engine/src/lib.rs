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
//! - [`inline`] — 行内格式化上下文（文本布局、行换行）
//! - [`dirty`] — 布局脏标记追踪器（增量布局）

#![warn(missing_docs)]

pub mod converter;
pub mod dirty;
pub mod engine;
pub mod inline;
pub mod multicol;
pub mod table;
pub mod tree;
pub mod types;

pub use converter::*;
pub use dirty::*;
pub use engine::*;
pub use inline::*;
pub use multicol::*;
pub use table::*;
pub use tree::*;
pub use types::*;
