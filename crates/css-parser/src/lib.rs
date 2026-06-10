//! # zero-css-parser
//!
//! 自建 CSS 解析器 — tokenizer + parser，支持完整选择器和属性解析。
//!
//! 不依赖任何 MPL 许可的 CSS 解析库（rust-cssparser、lightningcss），完全自建。
//!
//! ## 核心模块
//!
//! - [`tokenizer`] — CSS 词法分析器，将字符流转换为 token 流
//! - [`parser`] — CSS 语法解析器，将 token 流转换为 AST
//! - [`selector`] — 选择器解析和匹配
//! - [`values`] — CSS 属性值类型
//! - [`ast`] — CSS AST 数据结构
//! - [`media_query`] — CSS 媒体查询解析与评估

#![warn(missing_docs)]
#![cfg_attr(test, allow(unused_variables))]
#![cfg_attr(test, allow(unused_imports))]
#![allow(clippy::len_zero)]
#![allow(unused_comparisons)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::approx_constant)]
#![allow(clippy::bool_comparison)]
#![allow(clippy::bool_assert_comparison)]
#![allow(clippy::single_match)]
#![allow(clippy::comparison_to_empty)]
#![allow(clippy::redundant_pattern_matching)]
#![allow(clippy::unnecessary_map_or)]

pub mod ast;
pub mod media_query;
pub mod parser;
pub mod selector;
pub mod supports_condition;
pub mod tokenizer;
pub mod values;

pub use ast::*;
pub use media_query::*;
pub use parser::Parser;
pub use selector::*;
pub use supports_condition::*;
pub use tokenizer::{Spanned, Token, Tokenizer, line_column_from_offset};

#[cfg(test)]
mod tests;
