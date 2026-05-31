//! # zero-security
//!
//! 安全模型 — CORS、CSP、同源策略、沙箱、混合内容检测、COOP、COEP。

#![warn(missing_docs)]

pub mod coep;
pub mod coop;
pub mod cors;
pub mod csp;
pub mod mixed_content;
pub mod origin;
pub mod sandbox;

pub use coep::*;
pub use coop::*;
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

/// 检查当前上下文是否为跨源隔离（cross-origin isolated）。
///
/// 当 COOP 为 `SameOrigin` 且 COEP 为 `RequireCorp` 时，
/// 浏览上下文被认为是跨源隔离的，可以启用 `SharedArrayBuffer` 等高精度定时器 API。
pub fn is_cross_origin_isolated(coop: CoopPolicy, coep: CoepPolicy) -> bool {
    coop == CoopPolicy::SameOrigin && coep == CoepPolicy::RequireCorp
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cross_origin_isolated_when_both_set() {
        assert!(is_cross_origin_isolated(
            CoopPolicy::SameOrigin,
            CoepPolicy::RequireCorp
        ));
    }

    #[test]
    fn test_not_isolated_when_only_coop() {
        assert!(!is_cross_origin_isolated(
            CoopPolicy::SameOrigin,
            CoepPolicy::UnsafeNone
        ));
    }

    #[test]
    fn test_not_isolated_when_only_coep() {
        assert!(!is_cross_origin_isolated(
            CoopPolicy::UnsafeNone,
            CoepPolicy::RequireCorp
        ));
    }

    #[test]
    fn test_not_isolated_when_neither() {
        assert!(!is_cross_origin_isolated(
            CoopPolicy::UnsafeNone,
            CoepPolicy::UnsafeNone
        ));
    }

    #[test]
    fn test_not_isolated_with_coop_allow_popups() {
        assert!(!is_cross_origin_isolated(
            CoopPolicy::SameOriginAllowPopups,
            CoepPolicy::RequireCorp
        ));
    }

    #[test]
    fn test_not_isolated_with_coep_credentialless() {
        assert!(!is_cross_origin_isolated(
            CoopPolicy::SameOrigin,
            CoepPolicy::Credentialless
        ));
    }
}
