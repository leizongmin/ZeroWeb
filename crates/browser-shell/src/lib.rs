//! # zero-browser-shell
//!
//! 浏览器应用层 — 多标签页、收藏夹、地址栏、历史。
//!
//! 提供 UI-agnostic 的浏览器 shell 数据模型和协调逻辑，
//! 可被任何 UI 框架消费。实际渲染由 render-foundation 完成。

#![warn(missing_docs)]

mod autocomplete;
mod bookmarks;
mod browser;
mod context_menu;
mod download;
mod history;
mod settings;
mod tab;

pub use autocomplete::*;
pub use bookmarks::*;
pub use browser::*;
pub use context_menu::*;
pub use download::*;
pub use history::*;
pub use settings::*;
pub use tab::*;

#[cfg(test)]
mod tests;
