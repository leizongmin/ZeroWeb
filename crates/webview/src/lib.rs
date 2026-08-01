//! # zero-webview
//!
//! 面向外部应用的稳定嵌入接口。
//!
//! 提供构建器模式创建 WebView、导航、注入 JS、回调、渲染表面输出。

#![warn(missing_docs)]
#![cfg_attr(test, allow(unused_imports))]
#![cfg_attr(test, allow(unused_variables))]
#![allow(clippy::single_match)]
#![allow(clippy::assertions_on_constants)]
#![allow(unused_comparisons)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::doc_lazy_continuation)]

mod async_load;
mod net_pool;
pub mod webview;
pub mod webview_builder;

pub use async_load::{AsyncPageLoad, InProcessFetchHost, PageLoadStage, live_fontface_enabled};
pub use net_pool::{fetch_bytes_async, fetch_text_async};
pub use webview::*;
pub use webview_builder::*;

/// WebView 错误类型。
#[derive(Debug, thiserror::Error)]
pub enum WebViewError {
    /// 渲染错误。
    #[error("Rendering error: {0}")]
    Rendering(String),
    /// 导航错误。
    #[error("Navigation error: {0}")]
    Navigation(String),
    /// 脚本错误。
    #[error("Script error: {0}")]
    Script(String),
    /// 未实现。
    #[error("Not implemented: {0}")]
    NotImplemented(String),
}

#[cfg(test)]
mod tests;
