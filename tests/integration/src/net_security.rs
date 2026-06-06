#[cfg(test)]
use zero_net::url_parser::parse_url;
#[cfg(test)]
use zero_security::{
    CorsPolicy, HstsDirective, HstsStore, Origin, ResourceCheckResult, SecurityContext, check_cors,
    check_mixed_content, is_mixed_content, upgrade_to_https,
};

/// URL 解析 + 同源策略
#[test]
fn test_url_origin_same_origin_check() {
    let _url_a = parse_url("https://example.com/page1").unwrap();
    let _url_b = parse_url("https://example.com/page2?q=1").unwrap();

    let origin_a = Origin::parse("https://example.com").unwrap();
    let origin_b = Origin::parse("https://example.com").unwrap();

    assert!(origin_a.is_same_origin(&origin_b));
}

/// URL 解析 + CORS 检查
#[test]
fn test_cors_policy_with_parsed_url() {
    let origin = Origin::parse("http://evil.com").unwrap();
    let policy = CorsPolicy {
        allow_origins: vec!["http://example.com".to_string()],
        allow_methods: vec!["GET".to_string()],
        allow_headers: vec![],
        allow_credentials: false,
        max_age: None,
    };

    let result = check_cors(&policy, &origin, "GET", &[]);
    assert!(!result.allowed, "跨域请求应被拒绝");
}

/// 安全上下文判断
#[test]
fn test_url_security_context() {
    let _http_url = parse_url("http://example.com").unwrap();
    let _https_url = parse_url("https://example.com").unwrap();

    let http_origin = Origin::parse("http://example.com").unwrap();
    let https_origin = Origin::parse("https://example.com").unwrap();

    assert!(!http_origin.is_secure());
    assert!(https_origin.is_secure());
}

// ───────────────────── URL 解析 + 安全管线 ─────────────────────

/// URL 解析不同端口的源判断。
#[test]
fn test_url_origin_different_ports() {
    let origin_443 = Origin::parse("https://example.com:443").unwrap();
    let origin_8443 = Origin::parse("https://example.com:8443").unwrap();
    let origin_default = Origin::parse("https://example.com").unwrap();

    // 443 是 HTTPS 默认端口，应与无端口相同
    assert!(origin_443.is_same_origin(&origin_default));
    // 8443 不是默认端口，应为不同源
    assert!(!origin_8443.is_same_origin(&origin_default));
}

/// URL 解析不同协议的源判断。
#[test]
fn test_url_origin_different_schemes() {
    let http = Origin::parse("http://example.com").unwrap();
    let https = Origin::parse("https://example.com").unwrap();

    assert!(!http.is_same_origin(&https), "不同协议应为不同源");
}

/// URL 解析 IPv6 host。
#[test]
fn test_url_origin_ipv6() {
    let origin = Origin::parse("https://[::1]:8443").unwrap();
    assert!(origin.is_secure());
    // 解析成功即可验证 IPv6 支持正确
}

/// 混合内容 + URL 解析管线。
#[test]
fn test_mixed_content_url_pipeline() {
    let page_origin = Origin::parse("https://bank.com").unwrap();

    // HTTP 子资源
    assert!(is_mixed_content(&page_origin, "http://cdn.com/lib.js"));
    // HTTPS 子资源
    assert!(!is_mixed_content(&page_origin, "https://cdn.com/lib.js"));
    // data: URI
    assert!(!is_mixed_content(&page_origin, "data:text/html,<h1>Hi</h1>"));
    // blob: URI
    assert!(!is_mixed_content(&page_origin, "blob:https://bank.com/abc"));
    // 相对路径
    assert!(!is_mixed_content(&page_origin, "scripts/app.js"));
}

/// 混合内容升级 + URL 重构管线。
#[test]
fn test_mixed_content_upgrade_url_reconstruction() {
    let upgraded = upgrade_to_https("http://cdn.example.com:8080/path?q=1#hash").unwrap();
    assert!(upgraded.starts_with("https://"));
    assert!(upgraded.contains("cdn.example.com:8080"));
    assert!(upgraded.contains("/path?q=1#hash"));
}

/// HSTS + URL 解析管线。
#[test]
fn test_hsts_url_upgrade_pipeline() {
    let mut store = HstsStore::new();
    store.register(
        "example.com",
        HstsDirective::parse("max-age=31536000; includeSubDomains").unwrap(),
    );

    // HTTP URL 升级
    let upgraded = store.should_upgrade("http://example.com/page?q=1");
    assert!(upgraded.is_some());
    let url = upgraded.unwrap();
    assert!(url.starts_with("https://example.com/page?q=1"));

    // HTTPS URL 不升级
    assert!(store.should_upgrade("https://example.com/page").is_none());

    // 未知域名不升级
    assert!(store.should_upgrade("http://other.com/page").is_none());

    // 子域名升级（includeSubDomains）
    let sub_upgraded = store.should_upgrade("http://sub.example.com/page");
    assert!(sub_upgraded.is_some());
}

/// HSTS 子域名不继承（无 includeSubDomains 标志）。
#[test]
fn test_hsts_no_subdomain_without_flag() {
    let mut store = HstsStore::new();
    store.register("example.com", HstsDirective::parse("max-age=31536000").unwrap());

    assert!(store.should_upgrade("http://example.com/page").is_some());
    assert!(store.should_upgrade("http://sub.example.com/page").is_none());
}

/// SecurityContext + URL 完整管线。
#[test]
fn test_security_context_url_full_pipeline() {
    let mut ctx = SecurityContext::new();

    // 阶段 1: HTTP 页面加载（无页面源）
    let r1 = ctx.check_resource_url("http://unknown.com/page", "document");
    // 无 HSTS 且无页面源 → 允许
    assert_eq!(r1, ResourceCheckResult::Allow);

    // 设置 HTTPS 页面源
    ctx.set_page_origin("https://secure.bank.com");

    // 阶段 2: 混合内容阻止
    let r2 = ctx.check_resource_url("http://evil.com/steal.js", "script");
    assert!(matches!(r2, ResourceCheckResult::Blocked(_)));

    // 阶段 3: 混合内容升级
    let r3 = ctx.check_resource_url("http://cdn.com/photo.jpg", "img");
    assert!(matches!(r3, ResourceCheckResult::Upgraded(_)));

    // 阶段 4: HSTS 预加载升级（github.com 在预加载列表中）
    let r4 = ctx.check_resource_url("http://github.com/resource", "script");
    assert!(matches!(r4, ResourceCheckResult::Upgraded(_)));

    // 阶段 5: HTTPS 同源资源允许
    let r5 = ctx.check_resource_url("https://secure.bank.com/style.css", "style");
    assert_eq!(r5, ResourceCheckResult::Allow);
}

/// SecurityContext + URL 查询参数和片段。
#[test]
fn test_security_context_url_query_fragment() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://example.com");

    // 带查询参数的混合内容
    let r1 = ctx.check_resource_url("http://api.com/data?key=val&foo=bar", "connect");
    assert!(matches!(r1, ResourceCheckResult::Blocked(_)));

    // 带片段的混合内容
    let r2 = ctx.check_resource_url("http://cdn.com/page#section", "img");
    assert!(matches!(r2, ResourceCheckResult::Upgraded(ref u) if u.contains("#section")));
}

/// CORS + URL 不同端口。
#[test]
fn test_cors_different_port_origins() {
    let origin_3000 = Origin::parse("http://localhost:3000").unwrap();
    let origin_8080 = Origin::parse("http://localhost:8080").unwrap();

    let policy = CorsPolicy {
        allow_origins: vec!["http://localhost:3000".to_string()],
        allow_methods: vec!["GET".to_string()],
        allow_headers: vec![],
        allow_credentials: false,
        max_age: None,
    };

    // 3000 端口允许
    assert!(check_cors(&policy, &origin_3000, "GET", &[]).allowed);
    // 8080 端口拒绝
    assert!(!check_cors(&policy, &origin_8080, "GET", &[]).allowed);
}

/// CORS + 自定义头预检。
#[test]
fn test_cors_preflight_custom_headers() {
    let origin = Origin::parse("http://app.example.com").unwrap();
    let policy = CorsPolicy {
        allow_origins: vec!["http://app.example.com".to_string()],
        allow_methods: vec!["GET".to_string(), "POST".to_string()],
        allow_headers: vec!["Content-Type".to_string(), "X-Requested-With".to_string()],
        allow_credentials: false,
        max_age: Some(3600),
    };

    // 允许的头
    assert!(
        check_cors(
            &policy,
            &origin,
            "POST",
            &[
                ("Content-Type".into(), "application/json".into()),
                ("X-Requested-With".into(), "XMLHttpRequest".into()),
            ],
        )
        .allowed
    );

    // 不允许的头
    assert!(
        !check_cors(
            &policy,
            &origin,
            "POST",
            &[("X-Custom-Forbidden".into(), "value".into())],
        )
        .allowed
    );
}

/// SecurityContext 页面导航模拟。
#[test]
fn test_security_context_page_navigation() {
    let mut ctx = SecurityContext::new();

    // 导航到 HTTPS 页面
    ctx.set_page_origin("https://bank.com/dashboard");
    assert!(ctx.page_origin().unwrap().is_secure());

    // 子资源检查
    let r1 = ctx.check_resource_url("http://tracker.com/pixel.gif", "img");
    assert!(matches!(r1, ResourceCheckResult::Upgraded(_)));

    // 导航到 HTTP 页面
    ctx.set_page_origin("http://insecure.com/page");
    assert!(!ctx.page_origin().unwrap().is_secure());

    // HTTP 页面不再阻止混合内容
    let r2 = ctx.check_resource_url("http://evil.com/script.js", "script");
    assert_eq!(r2, ResourceCheckResult::Allow);
}

/// SecurityContext HSTS 运行时注册 + URL 升级。
#[test]
fn test_security_context_runtime_hsts_and_url() {
    let mut ctx = SecurityContext::new();
    let count_before = ctx.hsts_count();

    // 注册自定义域名
    assert!(ctx.register_hsts("my-secure-site.com", "max-age=86400"));
    assert_eq!(ctx.hsts_count(), count_before + 1);

    // HTTP URL 升级
    let r1 = ctx.check_resource_url("http://my-secure-site.com/api/data", "connect");
    assert!(matches!(r1, ResourceCheckResult::Upgraded(ref u) if u.starts_with("https://")));

    // 零 max-age 删除 HSTS 记录
    assert!(ctx.register_hsts("my-secure-site.com", "max-age=0"));
    let r2 = ctx.check_resource_url("http://my-secure-site.com/api/data", "connect");
    assert_eq!(r2, ResourceCheckResult::Allow);
}
