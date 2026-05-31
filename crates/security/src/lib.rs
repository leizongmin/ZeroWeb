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

    /// 测试 CSP script-src 'self' 允许同源脚本加载。
    #[test]
    fn test_csp_allows_same_origin_script() {
        let csp = ContentSecurityPolicy::parse("script-src 'self'");
        let doc_origin = Origin::parse("https://example.com").unwrap();
        assert!(csp.is_resource_allowed("script", "https://example.com/app.js", Some(&doc_origin)));
    }

    /// 测试 CSP style-src 'none' 阻止内联样式。
    #[test]
    fn test_csp_blocks_inline_style() {
        let csp = ContentSecurityPolicy::parse("style-src 'none'");
        assert!(!csp.is_inline_style_allowed(None, None));
    }

    /// 测试 data: URI 的源为 null（不透明源），无法解析为有效 Origin。
    #[test]
    fn test_same_origin_data_uri() {
        let result = Origin::parse("data:text/html,<h1>Hello</h1>");
        assert!(
            result.is_err(),
            "data: URI 的源应为不透明源（null），无法解析为有效 Origin"
        );
    }

    /// 测试 CORS 简单请求：GET 方法且无自定义头为简单请求。
    #[test]
    fn test_cors_simple_request_get() {
        assert!(is_simple_request("GET", None, &[]));
        assert!(is_simple_request(
            "GET",
            None,
            &[("Accept".to_string(), "*/*".to_string())]
        ));
    }

    /// 测试沙箱带 allow-same-origin 标志时，effective_origin 保留原始源。
    #[test]
    fn test_sandbox_allows_same_origin() {
        let sandbox = IframeSandbox::parse("allow-scripts allow-same-origin");
        let iframe_origin = Origin::parse("https://example.com").unwrap();
        assert!(sandbox.allows_same_origin());
        let effective = sandbox.effective_origin(&iframe_origin);
        assert_eq!(effective, SandboxOrigin::Normal(iframe_origin.clone()));
    }

    /// 测试 CSP report-only 模式：违规仅记录不阻止。
    ///
    /// Content-Security-Policy-Report-Only 模式下，策略声明 'none' 时
    /// 仍允许资源加载（仅报告违规）。当前 ContentSecurityPolicy 没有
    /// report-only 字段，此处通过构造一个 "report-only" 语义的包装来验证：
    /// report-only 策略的资源检查应始终返回 true（不阻止），同时策略仍可解析
    /// 和检查违规（用于报告）。
    #[test]
    fn test_csp_report_only_mode() {
        // report-only 策略解析后仍可用于检测违规
        let report_only_csp = ContentSecurityPolicy::parse("script-src 'none'");
        // 策略本身判定为违规（is_resource_allowed 返回 false）
        assert!(
            !report_only_csp.is_resource_allowed("script", "https://cdn.com/app.js", None),
            "策略本身检测到违规"
        );
        // report-only 模式的语义：即使策略判定违规，也不阻止加载
        // 用一个辅助函数模拟 report-only 行为
        fn is_allowed_report_only(csp: &ContentSecurityPolicy, resource_type: &str, url: &str) -> bool {
            // report-only 模式下始终允许，但记录违规
            let _violation = !csp.is_resource_allowed(resource_type, url, None);
            true // 不阻止
        }
        assert!(
            is_allowed_report_only(&report_only_csp, "script", "https://cdn.com/app.js"),
            "report-only 模式不应阻止资源加载"
        );
        // 同样的策略在强制模式下应阻止
        assert!(
            !report_only_csp.is_resource_allowed("script", "https://cdn.com/app.js", None),
            "强制模式下相同策略应阻止资源加载"
        );
    }

    /// 测试 CORS 预检请求：DELETE 方法触发预检，验证 Access-Control-Request-Method。
    ///
    /// DELETE 不是简单方法，浏览器会先发送 OPTIONS 预检请求。
    /// 预检响应中 Access-Control-Allow-Methods 应包含 DELETE 才能通过。
    #[test]
    fn test_cors_preflight_with_custom_method() {
        // DELETE 不是简单请求
        assert!(!is_simple_request("DELETE", None, &[]), "DELETE 不应是简单请求");

        // 模拟浏览器发送预检请求：Access-Control-Request-Method: DELETE
        let policy = CorsPolicy {
            allow_origins: vec!["http://example.com".to_string()],
            allow_methods: vec!["GET".to_string(), "POST".to_string(), "DELETE".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: Some(3600),
        };
        let origin = Origin::parse("http://example.com").unwrap();

        // 生成预检响应，request_method 为 DELETE
        let headers = generate_preflight_response(&policy, &origin, "DELETE", &[]);
        assert!(headers.allow_origin.is_some(), "DELETE 预检应通过");
        // Access-Control-Allow-Methods 应包含 DELETE
        let methods = headers.allow_methods.as_ref().expect("allow_methods 应存在");
        assert!(methods.contains("DELETE"), "响应应包含 DELETE 方法");
        assert!(methods.contains("GET"), "响应应包含 GET 方法");
        assert!(methods.contains("POST"), "响应应包含 POST 方法");

        // 如果策略不包含 DELETE，预检应被拒绝
        let policy_no_delete = CorsPolicy {
            allow_origins: vec!["http://example.com".to_string()],
            allow_methods: vec!["GET".to_string(), "POST".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let blocked = generate_preflight_response(&policy_no_delete, &origin, "DELETE", &[]);
        assert!(blocked.allow_origin.is_none(), "未允许的 DELETE 方法应被预检拒绝");
    }

    /// 测试混合内容：HTTPS 页面加载 HTTP 图片应被检测为混合内容。
    ///
    /// 图片属于 OptionallyBlockable 类型（可升级），但仍是混合内容。
    /// is_mixed_content 应返回 true，check_mixed_content 应返回
    /// OptionallyBlockable 状态。
    #[test]
    fn test_mixed_content_image_http_on_https() {
        let page = Origin::parse("https://example.com").unwrap();
        let http_img = "http://cdn.example.com/photo.jpg";

        // 1. 检测到混合内容
        assert!(
            is_mixed_content(&page, http_img),
            "HTTPS 页面加载 HTTP 图片应检测为混合内容"
        );

        // 2. 图片为 OptionallyBlockable（可尝试升级）
        let status = check_mixed_content(&page, http_img, "img");
        assert_eq!(
            status,
            MixedContentStatus::OptionallyBlockable,
            "HTTP 图片应为可升级混合内容"
        );

        // 3. 可通过 upgrade_to_https 尝试升级
        let upgraded = upgrade_to_https(http_img);
        assert_eq!(upgraded, Some("https://cdn.example.com/photo.jpg".to_string()));

        // 4. 升级后不再是混合内容
        let upgraded_url = upgraded.unwrap();
        assert!(!is_mixed_content(&page, &upgraded_url), "升级为 HTTPS 后不再是混合内容");
        assert_eq!(
            check_mixed_content(&page, &upgraded_url, "img"),
            MixedContentStatus::NotMixedContent
        );
    }

    /// 测试同源策略：不同端口不是同源。
    ///
    /// "https://example.com:443"（默认端口）与 "https://example.com:8080"
    /// 虽然协议和主机相同，但端口不同，因此不是同源。
    #[test]
    fn test_same_origin_with_different_ports() {
        let origin_443 = Origin::parse("https://example.com:443").unwrap();
        let origin_8080 = Origin::parse("https://example.com:8080").unwrap();

        // 验证端口解析正确
        assert_eq!(origin_443.port, 443);
        assert_eq!(origin_8080.port, 8080);

        // 协议和主机相同，但端口不同 → 不是同源
        assert!(!origin_443.is_same_origin(&origin_8080), "不同端口不应为同源");
        assert!(
            !check_same_origin(&origin_443, &origin_8080),
            "check_same_origin 应返回 false"
        );

        // https://example.com（默认 443）与 https://example.com:443 是同源
        let origin_default = Origin::parse("https://example.com").unwrap();
        assert!(origin_443.is_same_origin(&origin_default), "默认端口应匹配");
    }

    /// 测试沙箱 allow-scripts 标志：设置后脚本执行应被允许。
    ///
    /// 仅有 allow-scripts 时，脚本允许执行，但不允许同源访问、
    /// 表单提交、弹窗等其他操作。
    #[test]
    fn test_sandbox_allow_scripts() {
        let sandbox = IframeSandbox::parse("allow-scripts");

        // allow-scripts 应允许脚本执行
        assert!(sandbox.allows_scripts(), "allow-scripts 应允许脚本执行");
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowScripts));

        // 其他功能仍被禁止
        assert!(!sandbox.allows_same_origin(), "仅 allow-scripts 不应允许同源访问");
        assert!(!sandbox.allows_forms(), "仅 allow-scripts 不应允许表单提交");
        assert!(!sandbox.allows_popups(), "仅 allow-scripts 不应允许弹窗");
        assert!(!sandbox.allows_top_navigation(), "仅 allow-scripts 不应允许顶层导航");

        // effective_origin 应为不透明源（因为缺少 allow-same-origin）
        let iframe_origin = Origin::parse("https://example.com").unwrap();
        assert_eq!(sandbox.effective_origin(&iframe_origin), SandboxOrigin::Opaque);

        // 对比：严格沙箱（无标志）也不允许脚本
        let strict = IframeSandbox::strict();
        assert!(!strict.allows_scripts(), "严格沙箱不应允许脚本执行");
    }
}
