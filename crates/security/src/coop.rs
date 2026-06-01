//! COOP（Cross-Origin Opener Policy）模块。
//!
//! 控制顶层文档与跨源 opener 之间的关系，决定是否共享浏览上下文组。

/// Cross-Origin Opener Policy 值。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoopPolicy {
    /// 不施加限制（默认值）。
    UnsafeNone,
    /// 允许跨源弹出窗口保留 opener 引用。
    SameOriginAllowPopups,
    /// 仅同源可共享浏览上下文组。
    SameOrigin,
    /// 同源（含弹出窗口）可共享浏览上下文组。
    SameOriginIncludingPopups,
}

/// COOP 评估结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoopResult {
    /// 允许共享浏览上下文组。
    Allowed,
    /// 阻止共享，sever opener 引用。
    Blocked,
}

/// 评估 Cross-Origin Opener Policy。
///
/// 根据导航发起方（opener）的 COOP 策略和响应的 COOP 策略，
/// 决定新窗口是否可以保留 opener 引用。
///
/// `navigation_coop` 为导航发起方（opener 文档）的 COOP。
/// `response_coop` 为响应文档的 COOP。
/// `is_same_origin` 表示 opener 与新文档是否同源。
pub fn evaluate_coop(navigation_coop: CoopPolicy, response_coop: CoopPolicy, is_same_origin: bool) -> CoopResult {
    // 双方均为 UnsafeNone 时总是允许
    if navigation_coop == CoopPolicy::UnsafeNone && response_coop == CoopPolicy::UnsafeNone {
        return CoopResult::Allowed;
    }

    // 同源始终允许
    if is_same_origin {
        return CoopResult::Allowed;
    }

    // 跨源场景：检查响应方策略
    match response_coop {
        CoopPolicy::UnsafeNone => CoopResult::Allowed,
        CoopPolicy::SameOriginAllowPopups => {
            // 仅当导航方也为 UnsafeNone 或 SameOriginAllowPopups 时允许
            match navigation_coop {
                CoopPolicy::UnsafeNone | CoopPolicy::SameOriginAllowPopups => CoopResult::Allowed,
                CoopPolicy::SameOrigin | CoopPolicy::SameOriginIncludingPopups => CoopResult::Blocked,
            }
        }
        CoopPolicy::SameOrigin | CoopPolicy::SameOriginIncludingPopups => CoopResult::Blocked,
    }
}

/// 从 HTTP 响应头值解析 COOP 策略。
pub fn parse_coop(header_value: &str) -> CoopPolicy {
    match header_value.trim() {
        "same-origin-allow-popups" => CoopPolicy::SameOriginAllowPopups,
        "same-origin" => CoopPolicy::SameOrigin,
        "same-origin-including-popups" => CoopPolicy::SameOriginIncludingPopups,
        _ => CoopPolicy::UnsafeNone,
    }
}

/// 检查给定 COOP 策略是否为限制性策略（非 UnsafeNone）。
pub fn is_restrictive_coop(coop: CoopPolicy) -> bool {
    coop != CoopPolicy::UnsafeNone
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coop_unsafe_none_allows_all() {
        let result = evaluate_coop(CoopPolicy::UnsafeNone, CoopPolicy::UnsafeNone, false);
        assert_eq!(result, CoopResult::Allowed);
    }

    #[test]
    fn test_coop_same_origin_blocks_cross_origin() {
        let result = evaluate_coop(CoopPolicy::UnsafeNone, CoopPolicy::SameOrigin, false);
        assert_eq!(result, CoopResult::Blocked);
    }

    #[test]
    fn test_coop_same_origin_allows_same_origin() {
        let result = evaluate_coop(CoopPolicy::UnsafeNone, CoopPolicy::SameOrigin, true);
        assert_eq!(result, CoopResult::Allowed);
    }

    #[test]
    fn test_coop_same_origin_allow_popups() {
        // 跨源 + 导航方 UnsafeNone + 响应方 SameOriginAllowPopups → 允许
        let result = evaluate_coop(CoopPolicy::UnsafeNone, CoopPolicy::SameOriginAllowPopups, false);
        assert_eq!(result, CoopResult::Allowed);

        // 跨源 + 导航方 SameOrigin + 响应方 SameOriginAllowPopups → 阻止
        let result = evaluate_coop(CoopPolicy::SameOrigin, CoopPolicy::SameOriginAllowPopups, false);
        assert_eq!(result, CoopResult::Blocked);
    }

    #[test]
    fn test_coop_same_origin_including_popups_blocks_cross_origin() {
        let result = evaluate_coop(CoopPolicy::UnsafeNone, CoopPolicy::SameOriginIncludingPopups, false);
        assert_eq!(result, CoopResult::Blocked);
    }

    #[test]
    fn test_coop_both_restrictive_cross_origin() {
        let result = evaluate_coop(CoopPolicy::SameOrigin, CoopPolicy::SameOrigin, false);
        assert_eq!(result, CoopResult::Blocked);
    }

    #[test]
    fn test_coop_parse_header_values() {
        assert_eq!(parse_coop("same-origin"), CoopPolicy::SameOrigin);
        assert_eq!(
            parse_coop("same-origin-allow-popups"),
            CoopPolicy::SameOriginAllowPopups
        );
        assert_eq!(
            parse_coop("same-origin-including-popups"),
            CoopPolicy::SameOriginIncludingPopups
        );
        assert_eq!(parse_coop("unsafe-none"), CoopPolicy::UnsafeNone);
        assert_eq!(parse_coop(""), CoopPolicy::UnsafeNone);
    }

    #[test]
    fn test_coop_is_restrictive() {
        assert!(!is_restrictive_coop(CoopPolicy::UnsafeNone));
        assert!(is_restrictive_coop(CoopPolicy::SameOrigin));
        assert!(is_restrictive_coop(CoopPolicy::SameOriginAllowPopups));
        assert!(is_restrictive_coop(CoopPolicy::SameOriginIncludingPopups));
    }

    // ── 边界测试（round 23）──

    /// 测试 COOP SameOriginAllowPopups 在同源和跨源场景下的完整行为矩阵。
    ///
    /// SameOriginAllowPopups 策略允许跨源弹窗保留 opener 引用，
    /// 但不允许 SameOrigin/SameOriginIncludingPopups 策略的导航方
    /// 打开的弹窗保留 opener。
    #[test]
    fn test_coop_same_origin_allow_popups_full_matrix() {
        // 跨源 + UnsafeNone 导航方 + SameOriginAllowPopups 响应方 → 允许
        assert_eq!(
            evaluate_coop(CoopPolicy::UnsafeNone, CoopPolicy::SameOriginAllowPopups, false),
            CoopResult::Allowed,
            "UnsafeNone + SameOriginAllowPopups 跨源应允许"
        );

        // 跨源 + SameOriginAllowPopups 导航方 + SameOriginAllowPopups 响应方 → 允许
        assert_eq!(
            evaluate_coop(
                CoopPolicy::SameOriginAllowPopups,
                CoopPolicy::SameOriginAllowPopups,
                false
            ),
            CoopResult::Allowed,
            "双方 SameOriginAllowPopups 跨源应允许"
        );

        // 跨源 + SameOrigin 导航方 + SameOriginAllowPopups 响应方 → 阻止
        assert_eq!(
            evaluate_coop(CoopPolicy::SameOrigin, CoopPolicy::SameOriginAllowPopups, false),
            CoopResult::Blocked,
            "SameOrigin 导航方 + SameOriginAllowPopups 响应方跨源应阻止"
        );

        // 同源 + SameOriginAllowPopups → 始终允许
        assert_eq!(
            evaluate_coop(
                CoopPolicy::SameOriginAllowPopups,
                CoopPolicy::SameOriginAllowPopups,
                true
            ),
            CoopResult::Allowed,
            "同源应始终允许"
        );

        // 同源 + SameOrigin 导航方 + SameOriginAllowPopups 响应方 → 允许
        assert_eq!(
            evaluate_coop(CoopPolicy::SameOrigin, CoopPolicy::SameOriginAllowPopups, true),
            CoopResult::Allowed,
            "同源时无论策略组合均应允许"
        );
    }
}
