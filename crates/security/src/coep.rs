//! COEP（Cross-Origin Embedder Policy）模块。
//!
//! 控制文档可以加载哪些跨源资源，配合 COOP 实现跨源隔离。

/// Cross-Origin Embedder Policy 值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoepPolicy {
    /// 不施加限制（默认值）。
    UnsafeNone,
    /// 要求所有跨源资源提供 CORP 头或 CORS。
    RequireCorp,
    /// 跨源资源无需 CORP 头，但凭证不会被发送。
    Credentialless,
}

/// Cross-Origin Resource Policy（CORP）状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorpStatus {
    /// 资源与请求方同源。
    SameOrigin,
    /// 资源跨源。
    CrossOrigin,
    /// 资源未设置 Cross-Origin-Resource-Policy 头。
    NoPolicy,
}

/// COEP 评估结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoepResult {
    /// 允许加载资源。
    Allowed,
    /// 阻止加载资源。
    Blocked,
}

/// 评估 Cross-Origin Embedder Policy。
///
/// 根据文档的 COEP 策略和资源的 CORP 状态，决定是否允许加载该资源。
///
/// `document_coep` 为发起请求的文档的 COEP 策略。
/// `resource_corp` 为响应资源的 Cross-Origin-Resource-Policy 状态。
/// `is_same_origin` 表示文档与资源是否同源。
/// `has_cors` 表示资源是否通过 CORS 检查。
pub fn evaluate_coep(
    document_coep: CoepPolicy,
    resource_corp: CorpStatus,
    is_same_origin: bool,
    has_cors: bool,
) -> CoepResult {
    // UnsafeNone 不施加限制
    if document_coep == CoepPolicy::UnsafeNone {
        return CoepResult::Allowed;
    }

    // 同源资源始终允许
    if is_same_origin {
        return CoepResult::Allowed;
    }

    // 通过 CORS 检查的资源始终允许
    if has_cors {
        return CoepResult::Allowed;
    }

    // 跨源 + 无 CORS 的资源
    match document_coep {
        CoepPolicy::UnsafeNone => CoepResult::Allowed,
        CoepPolicy::RequireCorp => {
            // 要求 CORP 头明确允许
            match resource_corp {
                CorpStatus::SameOrigin => CoepResult::Allowed,
                CorpStatus::CrossOrigin | CorpStatus::NoPolicy => CoepResult::Blocked,
            }
        }
        CoepPolicy::Credentialless => {
            // Credentialless 模式：CORP 无策略时也允许（不发送凭证）
            match resource_corp {
                CorpStatus::SameOrigin | CorpStatus::NoPolicy => CoepResult::Allowed,
                CorpStatus::CrossOrigin => CoepResult::Blocked,
            }
        }
    }
}

/// 从 HTTP 响应头值解析 COEP 策略。
pub fn parse_coep(header_value: &str) -> CoepPolicy {
    match header_value.trim() {
        "require-corp" => CoepPolicy::RequireCorp,
        "credentialless" => CoepPolicy::Credentialless,
        _ => CoepPolicy::UnsafeNone,
    }
}

/// 从 Cross-Origin-Resource-Policy 头值解析 CORP 状态。
pub fn parse_corp(header_value: Option<&str>) -> CorpStatus {
    match header_value {
        None => CorpStatus::NoPolicy,
        Some(v) => match v.trim() {
            "same-origin" => CorpStatus::SameOrigin,
            "cross-origin" => CorpStatus::CrossOrigin,
            _ => CorpStatus::NoPolicy,
        },
    }
}

/// 检查给定 COEP 策略是否为限制性策略（非 UnsafeNone）。
pub fn is_restrictive_coep(coep: CoepPolicy) -> bool {
    coep != CoepPolicy::UnsafeNone
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coep_unsafe_none_allows_all() {
        let result = evaluate_coep(
            CoepPolicy::UnsafeNone,
            CorpStatus::NoPolicy,
            false,
            false,
        );
        assert_eq!(result, CoepResult::Allowed);
    }

    #[test]
    fn test_coep_require_corp_blocks_without_corp() {
        let result = evaluate_coep(
            CoepPolicy::RequireCorp,
            CorpStatus::NoPolicy,
            false,
            false,
        );
        assert_eq!(result, CoepResult::Blocked);
    }

    #[test]
    fn test_coep_require_corp_allows_with_corp_same_origin() {
        let result = evaluate_coep(
            CoepPolicy::RequireCorp,
            CorpStatus::SameOrigin,
            false,
            false,
        );
        assert_eq!(result, CoepResult::Allowed);
    }

    #[test]
    fn test_coep_require_corp_blocks_cross_origin_corp() {
        let result = evaluate_coep(
            CoepPolicy::RequireCorp,
            CorpStatus::CrossOrigin,
            false,
            false,
        );
        assert_eq!(result, CoepResult::Blocked);
    }

    #[test]
    fn test_coep_credentialless_behavior() {
        // Credentialless: NoPolicy → 允许（无凭证加载）
        let result = evaluate_coep(
            CoepPolicy::Credentialless,
            CorpStatus::NoPolicy,
            false,
            false,
        );
        assert_eq!(result, CoepResult::Allowed);

        // Credentialless: CrossOrigin → 阻止
        let result = evaluate_coep(
            CoepPolicy::Credentialless,
            CorpStatus::CrossOrigin,
            false,
            false,
        );
        assert_eq!(result, CoepResult::Blocked);

        // Credentialless: SameOrigin → 允许
        let result = evaluate_coep(
            CoepPolicy::Credentialless,
            CorpStatus::SameOrigin,
            false,
            false,
        );
        assert_eq!(result, CoepResult::Allowed);
    }

    #[test]
    fn test_coep_require_corp_allows_same_origin_resource() {
        let result = evaluate_coep(
            CoepPolicy::RequireCorp,
            CorpStatus::NoPolicy,
            true,
            false,
        );
        assert_eq!(result, CoepResult::Allowed);
    }

    #[test]
    fn test_coep_require_corp_allows_cors() {
        let result = evaluate_coep(
            CoepPolicy::RequireCorp,
            CorpStatus::NoPolicy,
            false,
            true,
        );
        assert_eq!(result, CoepResult::Allowed);
    }

    #[test]
    fn test_coep_parse_header_values() {
        assert_eq!(parse_coep("require-corp"), CoepPolicy::RequireCorp);
        assert_eq!(parse_coep("credentialless"), CoepPolicy::Credentialless);
        assert_eq!(parse_coep("unsafe-none"), CoepPolicy::UnsafeNone);
        assert_eq!(parse_coep(""), CoepPolicy::UnsafeNone);
    }

    #[test]
    fn test_coep_parse_corp_header() {
        assert_eq!(parse_corp(Some("same-origin")), CorpStatus::SameOrigin);
        assert_eq!(parse_corp(Some("cross-origin")), CorpStatus::CrossOrigin);
        assert_eq!(parse_corp(None), CorpStatus::NoPolicy);
        assert_eq!(parse_corp(Some("")), CorpStatus::NoPolicy);
    }

    #[test]
    fn test_coep_is_restrictive() {
        assert!(!is_restrictive_coep(CoepPolicy::UnsafeNone));
        assert!(is_restrictive_coep(CoepPolicy::RequireCorp));
        assert!(is_restrictive_coep(CoepPolicy::Credentialless));
    }
}
