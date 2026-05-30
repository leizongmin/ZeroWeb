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

pub mod tokenizer;
pub mod ast;
pub mod selector;
pub mod values;
pub mod parser;
pub mod media_query;

pub use tokenizer::{Token, Tokenizer};
pub use ast::*;
pub use selector::*;
pub use parser::Parser;
pub use media_query::*;

#[cfg(test)]
mod tests;
