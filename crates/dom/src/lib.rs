//! # zero-dom
//!
//! DOM 树实现 — 完整的 DOM 节点类型、树操作和增量更新。
//!
//! 基于 html5ever 解析 HTML 并构建 DOM 树，支持完整的 DOM Level 2+ 核心 API。
//!
//! ## 核心概念
//!
//! - [`Document`] — DOM 文档，持有所有节点数据的容器
//! - [`NodeId`] — 节点唯一标识（稳定，O(1) 查找）
//! - [`NodeKind`] — 节点类型枚举（Element、Text、Comment 等）
//!
//! ## 示例
//!
//! ```
//! use zero_dom::parse_html;
//!
//! let doc = parse_html("<html><body><h1>Hello</h1></body></html>");
//! assert!(doc.root().is_valid());
//! ```

#![warn(missing_docs)]
#![cfg_attr(test, allow(unused_mut))]

mod attributes;
mod document;
mod event;
mod focus;
mod mutation;
mod node;
mod parser;
mod query;
mod range;
mod serializer;
mod traversal;

pub use document::*;
pub use event::*;
pub use focus::FocusManager;
pub use mutation::*;
pub use node::ShadowRootMode;
pub use node::SlotAssignment;
pub use node::*;
pub use parser::*;
pub use query::*;
pub use range::*;
pub use traversal::*;

#[cfg(test)]
mod tests;
