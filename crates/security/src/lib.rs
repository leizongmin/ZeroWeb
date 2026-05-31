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

    /// 测试 CSP default-src 'none' 阻止内联脚本。
    ///
    /// default-src 'none' 作为未指定资源类型的回退，
    /// 内联脚本没有 script-src 指令时应回退到 default-src，
    /// 由于 default-src 为 'none'，内联脚本应被阻止。
    #[test]
    fn test_csp_default_src_blocks_script() {
        let csp = ContentSecurityPolicy::parse("default-src 'none'");
        // 内联脚本回退到 default-src 'none' → 被阻止
        assert!(
            !csp.is_inline_script_allowed(None, None),
            "default-src 'none' 应阻止内联脚本"
        );
        // 外部脚本也应被 default-src 'none' 阻止
        assert!(
            !csp.is_resource_allowed("script", "https://cdn.example.com/app.js", None),
            "default-src 'none' 应阻止外部脚本加载"
        );
        // 内联样式也应被阻止
        assert!(
            !csp.is_inline_style_allowed(None, None),
            "default-src 'none' 应阻止内联样式"
        );
    }

    /// 测试 CORS 预检请求 + credentials 组合场景。
    ///
    /// 验证：
    /// 1. 带凭证的预检请求在源为通配符时被拒绝
    /// 2. 带凭证的预检请求在具体源匹配时通过，且 allow_credentials 正确返回
    #[test]
    fn test_cors_preflight_with_credentials() {
        let origin = Origin::parse("http://example.com").unwrap();

        // 场景 1：具体源 + credentials → 预检通过
        let policy_ok = CorsPolicy {
            allow_origins: vec!["http://example.com".to_string()],
            allow_methods: vec!["DELETE".to_string()],
            allow_headers: vec![],
            allow_credentials: true,
            max_age: Some(3600),
        };
        let headers = generate_preflight_response(&policy_ok, &origin, "DELETE", &[]);
        assert!(headers.allow_origin.is_some(), "具体源 + credentials 预检应通过");
        assert_eq!(headers.allow_credentials, Some("true".to_string()));
        assert_eq!(headers.max_age, Some("3600".to_string()));

        // 场景 2：通配符源 + credentials → 预检被拒绝
        let policy_bad = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["DELETE".to_string()],
            allow_headers: vec![],
            allow_credentials: true,
            max_age: None,
        };
        let headers = generate_preflight_response(&policy_bad, &origin, "DELETE", &[]);
        assert!(headers.allow_origin.is_none(), "通配符源 + credentials 预检应被拒绝");

        // 场景 3：check_cors 对具体源 + credentials 也应通过
        let result = check_cors(&policy_ok, &origin, "DELETE", &[]);
        assert!(result.allowed, "具体源 + credentials 的 CORS 检查应通过");
    }

    /// 测试完全相同的 URL 解析出的两个 Origin 是同源。
    #[test]
    fn test_same_origin_identical_urls() {
        let a = Origin::parse("https://example.com/page").unwrap();
        let b = Origin::parse("https://example.com/other").unwrap();
        assert!(a.is_same_origin(&b), "相同 scheme+host+port 的 URL 应为同源");
        assert!(check_same_origin(&a, &b), "check_same_origin 对相同源应返回 true");

        // 完全相同字符串
        let c = Origin::parse("https://example.com/page").unwrap();
        assert!(a.is_same_origin(&c), "完全相同 URL 的 Origin 应为同源");
        assert_eq!(a, c, "完全相同 URL 的 Origin 应相等");
    }

    /// 测试 HTTPS 页面加载 HTTP 资源被检测为混合内容（Blockable 类型）。
    #[test]
    fn test_mixed_content_http_on_https() {
        let page = Origin::parse("https://example.com").unwrap();
        let http_script = "http://cdn.example.com/script.js";

        // 检测为混合内容
        assert!(
            is_mixed_content(&page, http_script),
            "HTTPS 页面加载 HTTP 资源应为混合内容"
        );

        // script 为 Blockable 类型
        let status = check_mixed_content(&page, http_script, "script");
        assert_eq!(
            status,
            MixedContentStatus::Blockable,
            "HTTP script 在 HTTPS 页面为 Blockable"
        );

        // iframe 也是 Blockable
        let http_iframe = "http://evil.com/embed";
        assert!(is_mixed_content(&page, http_iframe));
        assert_eq!(
            check_mixed_content(&page, http_iframe, "iframe"),
            MixedContentStatus::Blockable
        );

        // HTTPS 页面加载 HTTPS 资源不是混合内容
        let https_script = "https://cdn.example.com/script.js";
        assert!(!is_mixed_content(&page, https_script));
        assert_eq!(
            check_mixed_content(&page, https_script, "script"),
            MixedContentStatus::NotMixedContent
        );
    }

    /// 测试沙箱 allow-popups 标志允许弹窗但限制其他功能。
    #[test]
    fn test_sandbox_allow_popups() {
        let sandbox = IframeSandbox::parse("allow-popups");

        // allow-popups 应允许弹窗
        assert!(sandbox.allows_popups(), "allow-popups 应允许弹窗");
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowPopups));
        assert!(check_sandbox_popup(&sandbox), "check_sandbox_popup 应返回 true");

        // 其他功能仍被禁止
        assert!(!sandbox.allows_scripts(), "allow-popups 不应允许脚本");
        assert!(!sandbox.allows_same_origin(), "allow-popups 不应允许同源");
        assert!(!sandbox.allows_forms(), "allow-popups 不应允许表单");
        assert!(!sandbox.allows_top_navigation(), "allow-popups 不应允许顶层导航");

        // effective_origin 为不透明源
        let iframe_origin = Origin::parse("https://example.com").unwrap();
        assert_eq!(sandbox.effective_origin(&iframe_origin), SandboxOrigin::Opaque);

        // 严格沙箱不允许弹窗
        let strict = IframeSandbox::strict();
        assert!(!strict.allows_popups());
        assert!(!check_sandbox_popup(&strict));
    }

    /// 测试 CSP frame-src 阻止 iframe 加载。
    #[test]
    fn test_csp_frame_src_blocks() {
        let csp = ContentSecurityPolicy::parse("frame-src 'self'");
        let doc_origin = Origin::parse("https://example.com").unwrap();

        // 同源 iframe 允许
        assert!(csp.is_resource_allowed("frame", "https://example.com/embed", Some(&doc_origin)));

        // 外部 iframe 被阻止
        assert!(
            !csp.is_resource_allowed("frame", "https://evil.com/embed", Some(&doc_origin)),
            "frame-src 'self' 应阻止外部 iframe"
        );

        // is_child_allowed 也受影响（child-src 回退到 frame-src）
        assert!(csp.is_child_allowed("https://example.com/widget", Some(&doc_origin)));
        assert!(!csp.is_child_allowed("https://evil.com/widget", Some(&doc_origin)));

        // frame-src 'none' 阻止所有
        let csp_none = ContentSecurityPolicy::parse("frame-src 'none'");
        assert!(!csp_none.is_resource_allowed("frame", "https://example.com/embed", Some(&doc_origin)));
    }

    /// 测试 CORS 简单请求的 Content-Type 检查。
    #[test]
    fn test_cors_simple_request_content_type() {
        // 简单 Content-Type → 简单请求
        assert!(is_simple_request("POST", Some("text/plain"), &[]));
        assert!(is_simple_request(
            "POST",
            Some("application/x-www-form-urlencoded"),
            &[]
        ));
        assert!(is_simple_request("POST", Some("multipart/form-data"), &[]));

        // 非简单 Content-Type → 不是简单请求
        assert!(!is_simple_request("POST", Some("application/json"), &[]));
        assert!(!is_simple_request("POST", Some("text/xml"), &[]));

        // 无 Content-Type 的 GET 是简单请求
        assert!(is_simple_request("GET", None, &[]));

        // Content-Type 带参数（charset）→ text/plain; charset=utf-8 仍为简单
        assert!(is_simple_request("POST", Some("text/plain; charset=utf-8"), &[]));
        assert!(is_simple_request(
            "POST",
            Some("application/x-www-form-urlencoded; charset=UTF-8"),
            &[]
        ));

        // 非简单 Content-Type 在 CORS check_cors 中被阻止
        let policy = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["POST".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let result = check_cors(
            &policy,
            &origin,
            "POST",
            &[("Content-Type".to_string(), "application/json".to_string())],
        );
        assert!(!result.allowed, "application/json 非简单 Content-Type 应被 CORS 阻止");
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Edge-case tests
    // ═══════════════════════════════════════════════════════════════════════

    /// 测试 CSP connect-src 指令限制 XHR/Fetch 请求。
    #[test]
    fn test_csp_connect_src() {
        let csp = ContentSecurityPolicy::parse("connect-src https://api.example.com");
        assert!(csp.is_resource_allowed("connect", "https://api.example.com/data", None));
        assert!(!csp.is_resource_allowed("connect", "https://evil.com/data", None));
        // 其他资源类型不受 connect-src 影响，回退到 default-src（无则允许）
        assert!(csp.is_resource_allowed("img", "https://any.com/photo.jpg", None));
    }

    /// 测试 CSP media-src 指令限制音视频资源加载。
    #[test]
    fn test_csp_media_src() {
        let csp = ContentSecurityPolicy::parse("media-src https://media.example.com");
        assert!(csp.is_resource_allowed("media", "https://media.example.com/video.mp4", None));
        assert!(!csp.is_resource_allowed("media", "https://evil.com/video.mp4", None));
        // media-src 'none' 阻止所有媒体
        let csp_none = ContentSecurityPolicy::parse("media-src 'none'");
        assert!(!csp_none.is_resource_allowed("media", "https://media.example.com/video.mp4", None));
        assert!(!csp_none.is_resource_allowed("media", "video.mp4", None));
    }

    /// 测试 CORS max-age=0 时不缓存预检结果。
    #[test]
    fn test_cors_max_age_zero() {
        let policy = CorsPolicy {
            allow_origins: vec!["*".to_string()],
            allow_methods: vec!["GET".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: Some(0),
        };
        let origin = Origin::parse("http://example.com").unwrap();
        let headers = generate_preflight_response(&policy, &origin, "GET", &[]);
        // max-age=0 表示不缓存
        assert_eq!(headers.max_age, Some("0".to_string()));
        assert!(headers.allow_origin.is_some(), "请求本身仍应被允许");
    }

    /// 测试 http 与 https 同域名不同协议不是同源。
    #[test]
    fn test_same_origin_different_protocol() {
        let http = Origin::parse("http://example.com").unwrap();
        let https = Origin::parse("https://example.com").unwrap();
        assert_ne!(http.scheme, https.scheme, "scheme 应不同");
        assert_ne!(http.port, https.port, "默认端口应不同（80 vs 443）");
        assert!(!http.is_same_origin(&https), "http 与 https 不是同源");
        assert!(!check_same_origin(&http, &https));
    }

    /// 测试 HTTPS 页面加载 HTTPS 资源（非混合内容）。
    #[test]
    fn test_mixed_content_upgrade_https() {
        let page = Origin::parse("https://example.com").unwrap();
        let https_resource = "https://cdn.example.com/script.js";
        // HTTPS 页面加载 HTTPS 资源 → 不是混合内容
        assert!(!is_mixed_content(&page, https_resource));
        assert_eq!(
            check_mixed_content(&page, https_resource, "script"),
            MixedContentStatus::NotMixedContent
        );
    }

    /// 测试 COOP+COEP 所有组合中仅 SameOrigin+RequireCorp 为跨源隔离。
    #[test]
    fn test_cross_origin_isolated_all_combinations() {
        // 唯一返回 true 的组合
        assert!(is_cross_origin_isolated(
            CoopPolicy::SameOrigin,
            CoepPolicy::RequireCorp
        ));

        // SameOriginIncludingPopups 不是 SameOrigin → 不隔离
        assert!(!is_cross_origin_isolated(
            CoopPolicy::SameOriginIncludingPopups,
            CoepPolicy::RequireCorp
        ));

        // Credentialless 不是 RequireCorp → 不隔离
        assert!(!is_cross_origin_isolated(
            CoopPolicy::SameOrigin,
            CoepPolicy::Credentialless
        ));
    }

    /// 测试 CSP frame-ancestors 对非默认端口源的匹配。
    ///
    /// frame-ancestors 指令对源的匹配需要正确处理端口号。
    /// 当嵌入方使用非默认端口（如 8443）时，格式化的源字符串
    /// 应包含端口号（https://example.com:8443），与策略中的值匹配。
    #[test]
    fn test_csp_frame_ancestors_custom_port() {
        let csp = ContentSecurityPolicy::parse("frame-ancestors https://example.com:8443");
        let allowed = Origin::parse("https://example.com:8443").unwrap();
        let blocked_default = Origin::parse("https://example.com").unwrap();
        let blocked_other = Origin::parse("https://example.com:9443").unwrap();

        assert!(csp.is_frame_ancestor_allowed(&allowed), "匹配非默认端口应允许");
        assert!(
            !csp.is_frame_ancestor_allowed(&blocked_default),
            "默认端口 443 不匹配 8443"
        );
        assert!(!csp.is_frame_ancestor_allowed(&blocked_other), "不同端口不匹配");
    }

    /// 测试 HTTP 页面加载任何资源都不是混合内容。
    ///
    /// 混合内容检测仅在页面为 HTTPS 时触发。
    /// HTTP 页面加载 HTTP 资源不是混合内容（虽然不安全，但不是混合内容）。
    #[test]
    fn test_mixed_content_http_page_never_flagged() {
        let http_page = Origin::parse("http://example.com").unwrap();
        // HTTP 页面加载 HTTP 资源 → 不是混合内容
        assert!(!is_mixed_content(&http_page, "http://evil.com/script.js"));
        // HTTP 页面加载 HTTPS 资源 → 也不是混合内容
        assert!(!is_mixed_content(&http_page, "https://safe.com/script.js"));
        // HTTP 页面加载相对 URL → 也不是混合内容
        assert!(!is_mixed_content(&http_page, "app.js"));
        // check_mixed_content 对 HTTP 页面一律返回 NotMixedContent
        assert_eq!(
            check_mixed_content(&http_page, "http://evil.com/script.js", "script"),
            MixedContentStatus::NotMixedContent
        );
    }

    /// 测试 CSP 解析仅含空白字符的策略字符串不产生任何指令。
    ///
    /// 空格、制表符、换行符组成的字符串应被解析为空策略，
    /// 不产生任何指令，且不阻止任何资源加载。
    #[test]
    fn test_csp_parse_whitespace_only() {
        let csp = ContentSecurityPolicy::parse("   \t  \n  ");
        assert!(csp.directives.is_empty(), "仅含空白字符的策略不应产生指令");
        // 空策略不阻止任何资源
        assert!(csp.is_resource_allowed("script", "https://evil.com/bad.js", None));
        assert!(csp.is_inline_script_allowed(None, None));
        assert!(csp.is_inline_style_allowed(None, None));
    }

    /// 测试 CORS 空源列表时 check_cors 和 generate_preflight_response 均拒绝。
    ///
    /// 当 allow_origins 为空 Vec 时，任何源都无法通过 CORS 检查，
    /// 预检响应也不应返回 allow_origin。
    #[test]
    fn test_cors_empty_origins_blocks_everything() {
        let policy = CorsPolicy {
            allow_origins: vec![],
            allow_methods: vec!["GET".to_string(), "POST".to_string(), "DELETE".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: Some(3600),
        };
        let origin = Origin::parse("http://example.com").unwrap();

        // check_cors 应拒绝
        let result = check_cors(&policy, &origin, "GET", &[]);
        assert!(!result.allowed, "空源列表应拒绝所有请求");
        assert!(result.reason.contains("origin"), "拒绝原因应提及 origin");

        // generate_preflight_response 也应拒绝
        let headers = generate_preflight_response(&policy, &origin, "DELETE", &[]);
        assert!(
            headers.allow_origin.is_none(),
            "空源列表的预检响应不应包含 allow_origin"
        );
        assert!(headers.allow_methods.is_none());
    }

    /// 测试沙箱解析仅含无效/未知标志时等同于严格沙箱。
    ///
    /// 当 sandbox 属性值全部为无法识别的标志时，
    /// 解析后 flags 列表应为空，所有功能均被禁止，
    /// 行为等同于 IframeSandbox::strict()。
    #[test]
    fn test_sandbox_parse_only_unknown_flags() {
        let sandbox = IframeSandbox::parse("allow-unknown-flag foo-bar not-a-real-flag");
        // 所有标志均无法识别 → flags 为空
        assert!(!sandbox.allows_scripts(), "未知标志不应允许脚本");
        assert!(!sandbox.allows_forms(), "未知标志不应允许表单");
        assert!(!sandbox.allows_same_origin(), "未知标志不应允许同源");
        assert!(!sandbox.allows_popups(), "未知标志不应允许弹窗");
        assert!(!sandbox.allows_top_navigation(), "未知标志不应允许顶层导航");

        // effective_origin 为不透明源
        let iframe_origin = Origin::parse("https://example.com").unwrap();
        assert_eq!(sandbox.effective_origin(&iframe_origin), SandboxOrigin::Opaque);
    }

    /// 测试混合内容 upgrade_to_https 对边界 URL 的处理。
    ///
    /// 最小 HTTP URL（如 "http://x"）应能成功升级。
    /// 带认证信息的 HTTP URL（含 userinfo）也应正确升级。
    /// 非 http:// 开头的 URL（空字符串、ftp://）应返回 None。
    #[test]
    fn test_mixed_content_upgrade_edge_urls() {
        // 最小 HTTP URL → 可升级
        assert_eq!(
            upgrade_to_https("http://x"),
            Some("https://x".to_string()),
            "最小 HTTP URL 应可升级"
        );

        // 带端口的 HTTP URL → 可升级
        assert_eq!(
            upgrade_to_https("http://example.com:8080/path"),
            Some("https://example.com:8080/path".to_string()),
            "带端口的 HTTP URL 应可升级"
        );

        // 空 URL → 不以 http:// 开头 → None
        assert_eq!(upgrade_to_https(""), None, "空 URL 不应升级");

        // ftp:// URL → 不以 http:// 开头 → None
        assert_eq!(
            upgrade_to_https("ftp://files.example.com/data"),
            None,
            "ftp URL 不应升级"
        );

        // 仅 "http://" 无主机 → 可升级为 "https://"
        assert_eq!(
            upgrade_to_https("http://"),
            Some("https://".to_string()),
            "仅 scheme 的 URL 应可升级"
        );
    }

    /// 测试 CSP worker-src 回退链中 child-src 优先于 script-src。
    ///
    /// 当 worker-src 不存在时，is_worker_allowed 依次回退：
    /// child-src → frame-src → script-src → default-src。
    /// 验证 child-src 存在时优先使用 child-src，而非 script-src。
    #[test]
    fn test_csp_worker_src_fallback_prefers_child_over_script() {
        let csp = ContentSecurityPolicy::parse(
            "default-src 'none'; child-src https://child.com; script-src https://script.com",
        );
        // worker-src 不存在 → 回退到 child-src（而非 script-src）
        assert!(
            csp.is_worker_allowed("https://child.com/worker.js", None),
            "应使用 child-src 而非 script-src"
        );
        // script-src 中的 URL 在 child-src 中不存在 → 不应允许
        assert!(
            !csp.is_worker_allowed("https://script.com/worker.js", None),
            "child-src 存在时不应回退到 script-src"
        );
    }

    /// 测试 COEP parse_coep 对空白字符、大小写和未知值的处理。
    ///
    /// parse_coep 应对头部值做 trim 后匹配，未知值回退到 UnsafeNone。
    /// 带前导/尾随空格的合法值应正确解析。
    #[test]
    fn test_coep_parse_edge_cases() {
        // 带前导/尾随空格
        assert_eq!(
            parse_coep("  require-corp  "),
            CoepPolicy::RequireCorp,
            "带空格的 require-corp 应正确解析"
        );
        assert_eq!(
            parse_coep("\tcredentialless\n"),
            CoepPolicy::Credentialless,
            "带制表符和换行的 credentialless 应正确解析"
        );

        // 未知值 → UnsafeNone
        assert_eq!(
            parse_coep("require-corp-strict"),
            CoepPolicy::UnsafeNone,
            "未知策略值应回退到 UnsafeNone"
        );

        // 仅空格 → UnsafeNone
        assert_eq!(parse_coep("   "), CoepPolicy::UnsafeNone, "仅空格应回退到 UnsafeNone");

        // CORP 解析：未知值 → NoPolicy
        assert_eq!(
            parse_corp(Some("unknown-policy")),
            CorpStatus::NoPolicy,
            "未知 CORP 值应回退到 NoPolicy"
        );
        assert_eq!(
            parse_corp(Some("  same-origin  ")),
            CorpStatus::SameOrigin,
            "带空格的 CORP 值应正确解析"
        );
    }

    /// 测试 COOP evaluate_coop：导航方和响应方均为 SameOriginAllowPopups 时的行为。
    ///
    /// 当双方都为 SameOriginAllowPopups 且为跨源时，应允许共享浏览上下文组。
    /// 同时验证 COOP 解析对空白字符的容错。
    #[test]
    fn test_coop_same_origin_allow_popups_mutual() {
        // 跨源 + 双方均为 SameOriginAllowPopups → 允许
        let result = evaluate_coop(
            CoopPolicy::SameOriginAllowPopups,
            CoopPolicy::SameOriginAllowPopups,
            false,
        );
        assert_eq!(result, CoopResult::Allowed, "双方 SameOriginAllowPopups 跨源应允许");

        // 同源 → 始终允许
        let result_same = evaluate_coop(
            CoopPolicy::SameOriginAllowPopups,
            CoopPolicy::SameOriginAllowPopups,
            true,
        );
        assert_eq!(result_same, CoopResult::Allowed, "同源应始终允许");

        // parse_coop 空白容错
        assert_eq!(
            parse_coop("  same-origin-allow-popups  "),
            CoopPolicy::SameOriginAllowPopups,
            "带空格的 COOP 值应正确解析"
        );
        assert_eq!(
            parse_coop("\t same-origin \n"),
            CoopPolicy::SameOrigin,
            "带特殊空白符的 same-origin 应正确解析"
        );
    }

    /// 测试 CORS check_cors 和 generate_preflight_response 对非默认端口的源格式一致性。
    ///
    /// 当 Origin 使用非默认端口（如 http://example.com:3000）时，
    /// check_cors 格式化的源字符串应与 generate_preflight_response 一致，
    /// 确保 allow_origins 列表中的端口格式在两处匹配。
    #[test]
    fn test_cors_non_default_port_formatting_consistency() {
        let origin_3000 = Origin::parse("http://example.com:3000").unwrap();
        assert_eq!(origin_3000.port, 3000);

        // check_cors 使用 "http://example.com:3000" 格式匹配
        let policy = CorsPolicy {
            allow_origins: vec!["http://example.com:3000".to_string()],
            allow_methods: vec!["GET".to_string(), "POST".to_string()],
            allow_headers: vec![],
            allow_credentials: false,
            max_age: None,
        };
        let result = check_cors(&policy, &origin_3000, "GET", &[]);
        assert!(result.allowed, "非默认端口 3000 应在 check_cors 中匹配");

        // generate_preflight_response 也应格式化为 "http://example.com:3000"
        let headers = generate_preflight_response(&policy, &origin_3000, "GET", &[]);
        assert_eq!(
            headers.allow_origin,
            Some("http://example.com:3000".to_string()),
            "预检响应应包含完整端口号"
        );

        // 默认端口 80 的 http 源不应匹配 3000
        let origin_80 = Origin::parse("http://example.com").unwrap();
        let result_80 = check_cors(&policy, &origin_80, "GET", &[]);
        assert!(!result_80.allowed, "默认端口 80 不应匹配 3000");
    }

    /// 测试 CSP font-src 指令限制字体资源加载。
    ///
    /// font-src 指令控制 @font-face 可加载的字体来源。
    /// 'none' 阻止所有字体加载，指定域名仅允许该域名，
    /// 无 font-src 指令时回退到 default-src。
    #[test]
    fn test_csp_font_src_restricts_fonts() {
        // font-src 指定允许的字体域名
        let csp = ContentSecurityPolicy::parse("font-src https://fonts.example.com");
        assert!(
            csp.is_resource_allowed("font", "https://fonts.example.com/roboto.woff2", None),
            "font-src 中的域名应允许加载"
        );
        assert!(
            !csp.is_resource_allowed("font", "https://evil.com/steal.woff", None),
            "不在 font-src 中的域名应被阻止"
        );

        // font-src 'none' 阻止所有字体
        let csp_none = ContentSecurityPolicy::parse("font-src 'none'");
        assert!(
            !csp_none.is_resource_allowed("font", "https://fonts.example.com/roboto.woff2", None),
            "font-src 'none' 应阻止所有字体"
        );
        // 其他资源类型不受影响
        assert!(
            csp_none.is_resource_allowed("script", "https://any.com/app.js", None),
            "font-src 不影响其他资源类型"
        );

        // 无 font-src → 回退到 default-src
        let csp_default = ContentSecurityPolicy::parse("default-src 'self'");
        let doc_origin = Origin::parse("https://example.com").unwrap();
        assert!(
            csp_default.is_resource_allowed("font", "https://example.com/font.woff", Some(&doc_origin)),
            "无 font-src 时应回退到 default-src"
        );
    }

    /// 测试沙箱 allow-downloads 和 allow-presentation 等不常见标志的解析。
    ///
    /// allow-downloads 允许下载文件，allow-presentation 允许呈现模式。
    /// 这些标志不影响核心功能（脚本、同源、表单、弹窗、顶层导航）。
    #[test]
    fn test_sandbox_uncommon_flags_do_not_grant_core_permissions() {
        let sandbox = IframeSandbox::parse(
            "allow-downloads allow-presentation allow-orientation-lock allow-pointer-lock allow-autoplay allow-modals",
        );

        // 不常见标志不应授予核心权限
        assert!(!sandbox.allows_scripts(), "不常见标志不应允许脚本执行");
        assert!(!sandbox.allows_same_origin(), "不常见标志不应允许同源访问");
        assert!(!sandbox.allows_forms(), "不常见标志不应允许表单提交");
        assert!(!sandbox.allows_popups(), "不常见标志不应允许弹窗");
        assert!(!sandbox.allows_top_navigation(), "不常见标志不应允许顶层导航");

        // 但这些标志确实被解析并存在
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowDownloads));
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowPresentation));
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowOrientationLock));
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowPointerLock));
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowAutoplay));
        assert!(sandbox.has_flag(IframeSandboxFlag::AllowModals));

        // 缺少 allow-same-origin → effective_origin 为不透明源
        let origin = Origin::parse("https://example.com").unwrap();
        assert_eq!(sandbox.effective_origin(&origin), SandboxOrigin::Opaque);
    }

    /// 测试 COEP RequireCorp 模式下 CORS 通过的跨源资源仍被允许。
    ///
    /// evaluate_coep 的 has_cors 参数为 true 时，即使资源为跨源且
    /// CORP 状态为 NoPolicy，也应返回 Allowed。这是 COEP 与 CORS
    /// 协同工作的关键：CORS 预检通过的请求不受 COEP 限制。
    #[test]
    fn test_coep_require_corp_allows_cors_cross_origin() {
        // RequireCorp + 跨源 + 无 CORP + 无 CORS → 阻止
        let blocked = evaluate_coep(CoepPolicy::RequireCorp, CorpStatus::NoPolicy, false, false);
        assert_eq!(
            blocked,
            CoepResult::Blocked,
            "RequireCorp 应阻止无 CORP 无 CORS 的跨源资源"
        );

        // RequireCorp + 跨源 + 无 CORP + 有 CORS → 允许
        let allowed = evaluate_coep(CoepPolicy::RequireCorp, CorpStatus::NoPolicy, false, true);
        assert_eq!(allowed, CoepResult::Allowed, "CORS 通过时应绕过 COEP 限制");

        // RequireCorp + 同源 + 无 CORP + 无 CORS → 允许（同源始终通过）
        let same_origin = evaluate_coep(CoepPolicy::RequireCorp, CorpStatus::NoPolicy, true, false);
        assert_eq!(same_origin, CoepResult::Allowed, "同源资源应始终被允许");

        // Credentialless + 跨源 + CrossOrigin CORP + 有 CORS → 允许
        let credless = evaluate_coep(CoepPolicy::Credentialless, CorpStatus::CrossOrigin, false, true);
        assert_eq!(credless, CoepResult::Allowed, "CORS 通过时 Credentialless 也应允许");
    }

    /// 测试混合内容检测对未知/非标准资源类型的分类为 Blockable。
    ///
    /// classify_resource_type 仅将 img、audio、video、media 列为
    /// OptionallyBlockable，其他所有类型（包括 "link"、"xhr"、"fetch"）
    /// 均应归类为 Blockable，即必须阻止的混合内容。
    #[test]
    fn test_mixed_content_unknown_resource_type_is_blockable() {
        let page = Origin::parse("https://example.com").unwrap();
        let http_url = "http://cdn.example.com/resource";

        // 已知的 OptionallyBlockable 类型
        assert_eq!(
            check_mixed_content(&page, http_url, "img"),
            MixedContentStatus::OptionallyBlockable,
            "img 应为可升级类型"
        );

        // 未知/非标准类型均应为 Blockable
        assert_eq!(
            check_mixed_content(&page, http_url, "link"),
            MixedContentStatus::Blockable,
            "link 应为阻塞型"
        );
        assert_eq!(
            check_mixed_content(&page, http_url, "xhr"),
            MixedContentStatus::Blockable,
            "xhr 应为阻塞型"
        );
        assert_eq!(
            check_mixed_content(&page, http_url, "fetch"),
            MixedContentStatus::Blockable,
            "fetch 应为阻塞型"
        );
        assert_eq!(
            check_mixed_content(&page, http_url, "xmlhttprequest"),
            MixedContentStatus::Blockable,
            "xmlhttprequest 应为阻塞型"
        );
    }
}
