//! # zero-security
//!
//! 安全模型 — CORS、CSP、同源策略、沙箱、混合内容检测。

#![warn(missing_docs)]

pub mod cors;
pub mod csp;
pub mod mixed_content;
pub mod origin;
pub mod sandbox;

pub use cors::*;
pub use csp::*;
pub use mixed_content::*;
pub use origin::*;
pub use sandbox::*;

use thiserror::Error;

/// 安全错误类型。
#[derive(Error, Debug)]
pub enum SecurityError {
    /// 源解析错误。
    #[error("Origin parse error: {0}")]
    OriginParse(String),
    /// CORS 错误。
    #[error("CORS error: {0}")]
    Cors(String),
    /// CSP 违规。
    #[error("CSP violation: {0}")]
    CspViolation(String),
}
