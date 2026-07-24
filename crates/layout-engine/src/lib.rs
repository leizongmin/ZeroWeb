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
#![allow(clippy::field_reassign_with_default)]
#![allow(clippy::len_zero)]
#![allow(clippy::collapsible_if)]

pub mod converter;
pub mod dirty;
pub mod engine;
pub mod float_positioning;
pub mod inline;
pub mod inline_block_split;
pub mod inline_finalization;
pub mod intrinsic_sizing;
pub mod margin_collapse;
pub mod multicol;
#[allow(dead_code)] // R1350 Phase 1 dormant：empirical balancing 模型，Phase 2 wiring 待定
pub mod multicol_balancing;
pub mod print_pagination;
pub mod r109;
pub mod table;
pub mod table_borders;
pub mod table_cell_content;
pub mod table_float_fix;
pub mod table_grid;
pub mod table_shrink;
pub mod table_types;
pub mod table_visibility;
pub mod tree;
pub mod types;
pub mod vertical_block_flow;

pub use converter::*;
pub use dirty::*;
pub use engine::*;
pub use inline::*;
pub use margin_collapse::*;
pub use multicol::*;
pub use table::*;
pub use tree::*;
pub use types::*;
