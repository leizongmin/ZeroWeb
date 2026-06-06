//! 安全管线集成测试
//!
//! 测试 security crate 与 net/url/origin 等 crate 的跨 crate 协作，
//! 验证 CSP、CORS、HSTS、混合内容、沙箱、权限、站点隔离等安全机制的端到端管线。

#[cfg(test)]
use zero_net::url_parser::parse_url;
#[cfg(test)]
use zero_security::{
    CoepPolicy, ContentSecurityPolicy, CorsPolicy, HstsDirective, HstsStore, IframeSandbox, IframeSandboxFlag,
    IsolationPolicy, MixedContentStatus, Origin, PermissionManager, PermissionName, PermissionState,
    ResourceCheckResult, SecurityContext, Site, SiteIsolationManager, check_cors, check_mixed_content,
    check_sandbox_navigation, check_sandbox_popup, is_cross_origin_isolated, is_mixed_content, upgrade_to_https,
};

// ───────────────────── CSP + Origin 管线 ─────────────────────

/// CSP 解析 + 同源资源加载检查管线
#[test]
fn test_csp_parse_and_self_origin_resource() {
    let csp = ContentSecurityPolicy::parse("default-src 'self'; script-src 'self' https://cdn.example.com");
    let origin = Origin::parse("https://example.com").unwrap();

    // 同源脚本应允许
    assert!(csp.is_resource_allowed("script", "https://example.com/app.js", Some(&origin)));
    // CDN 脚本应允许
    assert!(csp.is_resource_allowed("script", "https://cdn.example.com/lib.js", Some(&origin)));
    // 其他域脚本应拒绝
    assert!(!csp.is_resource_allowed("script", "https://evil.com/steal.js", Some(&origin)));
    // 同源图片应允许（回退 default-src 'self'）
    assert!(csp.is_resource_allowed("img", "https://example.com/logo.png", Some(&origin)));
    // 跨域图片应拒绝
    assert!(!csp.is_resource_allowed("img", "https://other.com/img.png", Some(&origin)));
}

/// CSP default-src 'none' 完全阻止管线
#[test]
fn test_csp_default_none_blocks_all() {
    let csp = ContentSecurityPolicy::parse("default-src 'none'");
    let origin = Origin::parse("https://example.com").unwrap();

    assert!(!csp.is_resource_allowed("script", "https://example.com/a.js", Some(&origin)));
    assert!(!csp.is_resource_allowed("style", "https://example.com/a.css", Some(&origin)));
    assert!(!csp.is_resource_allowed("img", "https://example.com/a.png", Some(&origin)));
    assert!(!csp.is_resource_allowed("connect", "https://example.com/api", Some(&origin)));
    assert!(!csp.is_resource_allowed("font", "https://example.com/font.woff", Some(&origin)));
}

/// CSP 内联脚本 + nonce/hash 管线
#[test]
fn test_csp_inline_script_nonce_hash() {
    let csp = ContentSecurityPolicy::parse("script-src 'nonce-abc123' 'sha256-base64hash'");

    // 无 nonce/hash → 拒绝
    assert!(!csp.is_inline_script_allowed(None, None));
    // 正确 nonce → 允许
    assert!(csp.is_inline_script_allowed(Some("abc123"), None));
    // 正确 hash → 允许
    assert!(csp.is_inline_script_allowed(None, Some("base64hash")));
    // 错误 nonce → 拒绝
    assert!(!csp.is_inline_script_allowed(Some("wrong"), None));
}

/// CSP unsafe-inline 允许所有内联脚本管线
#[test]
fn test_csp_unsafe_inline_allows_all() {
    let csp = ContentSecurityPolicy::parse("script-src 'unsafe-inline'");
    assert!(csp.is_inline_script_allowed(None, None));
    assert!(csp.is_inline_script_allowed(Some("anything"), None));
}

/// CSP + URL 解析管线：data: URI 和 blob: URI 不匹配 'self'
#[test]
fn test_csp_data_blob_not_self() {
    let csp = ContentSecurityPolicy::parse("default-src 'self'");
    let origin = Origin::parse("https://example.com").unwrap();

    // data: URI 不匹配 'self'
    assert!(!csp.is_resource_allowed("img", "data:image/png;base64,abc", Some(&origin)));
    // blob: URI 不匹配 'self'
    assert!(!csp.is_resource_allowed("img", "blob:https://example.com/uuid", Some(&origin)));
}

/// CSP scheme-source 管线
#[test]
fn test_csp_scheme_source() {
    let csp = ContentSecurityPolicy::parse("default-src https:; img-src data:");
    let origin = Origin::parse("https://example.com").unwrap();

    // https: scheme 匹配所有 HTTPS URL
    assert!(csp.is_resource_allowed("script", "https://any.com/lib.js", Some(&origin)));
    // http: 被 https: scheme-source 拒绝
    assert!(!csp.is_resource_allowed("script", "http://any.com/lib.js", Some(&origin)));
    // data: 匹配 img-src 的 data: scheme
    assert!(csp.is_resource_allowed("img", "data:image/png;base64,abc", Some(&origin)));
}

/// CSP 通配符域名匹配管线
#[test]
fn test_csp_wildcard_domain() {
    let csp = ContentSecurityPolicy::parse("script-src *.example.com");
    let origin = Origin::parse("https://example.com").unwrap();

    assert!(csp.is_resource_allowed("script", "https://cdn.example.com/lib.js", Some(&origin)));
    assert!(csp.is_resource_allowed("script", "https://api.v2.example.com/lib.js", Some(&origin)));
    // 不同根域名应拒绝
    assert!(!csp.is_resource_allowed("script", "https://cdn.notexample.com/lib.js", Some(&origin)));
}

/// CSP connect-src + fetch API 管线集成
#[test]
fn test_csp_connect_src_fetch() {
    // connect-src 显式列出时不再回退到 default-src
    let csp =
        ContentSecurityPolicy::parse("default-src 'self'; connect-src https://api.example.com https://example.com");
    let origin = Origin::parse("https://example.com").unwrap();

    // API 域名允许
    assert!(csp.is_resource_allowed("connect", "https://api.example.com/v1/data", Some(&origin)));
    // 同域也允许（显式列出）
    assert!(csp.is_resource_allowed("connect", "https://example.com/data", Some(&origin)));
    // 其他域拒绝
    assert!(!csp.is_resource_allowed("connect", "https://evil.com/exfil", Some(&origin)));

    // 无 connect-src 时回退到 default-src
    let csp_fallback = ContentSecurityPolicy::parse("default-src 'self'");
    assert!(csp_fallback.is_resource_allowed("connect", "https://example.com/api", Some(&origin)));
    assert!(!csp_fallback.is_resource_allowed("connect", "https://evil.com/api", Some(&origin)));
}

/// CSP + style-src-attr 内联样式检查管线
#[test]
fn test_csp_inline_style_pipeline() {
    let csp_strict = ContentSecurityPolicy::parse("style-src 'self'");
    let csp_loose = ContentSecurityPolicy::parse("style-src 'unsafe-inline'");

    // 严格模式：无 nonce/hash 拒绝内联样式
    assert!(!csp_strict.is_inline_style_allowed(None, None));
    // 宽松模式：允许内联样式
    assert!(csp_loose.is_inline_style_allowed(None, None));
}

// ───────────────────── CORS + URL 解析管线 ─────────────────────

/// CORS + URL 解析：简单请求跨域检查
#[test]
fn test_cors_simple_request_pipeline() {
    let origin = Origin::parse("http://app.example.com").unwrap();
    let policy = CorsPolicy {
        allow_origins: vec!["http://app.example.com".to_string()],
        allow_methods: vec!["GET".to_string(), "POST".to_string()],
        allow_headers: vec![],
        allow_credentials: false,
        max_age: None,
    };

    // 同源 GET 允许
    let result = check_cors(&policy, &origin, "GET", &[]);
    assert!(result.allowed);

    // 跨域拒绝
    let evil = Origin::parse("http://evil.com").unwrap();
    let result = check_cors(&policy, &evil, "GET", &[]);
    assert!(!result.allowed);
}

/// CORS 预检请求管线
#[test]
fn test_cors_preflight_pipeline() {
    let origin = Origin::parse("http://app.example.com").unwrap();
    let policy = CorsPolicy {
        allow_origins: vec!["http://app.example.com".to_string()],
        allow_methods: vec!["GET".to_string(), "PUT".to_string(), "DELETE".to_string()],
        allow_headers: vec!["Content-Type".to_string(), "X-Custom-Header".to_string()],
        allow_credentials: true,
        max_age: Some(3600),
    };

    // PUT 方法允许
    let result = check_cors(
        &policy,
        &origin,
        "PUT",
        &[("Content-Type".into(), "application/json".into())],
    );
    assert!(result.allowed);

    // 不允许的方法拒绝
    let result = check_cors(&policy, &origin, "PATCH", &[]);
    assert!(!result.allowed);
}

/// CORS + 凭证 + 通配符冲突管线
#[test]
fn test_cors_credentials_wildcard_conflict() {
    let origin = Origin::parse("http://app.example.com").unwrap();
    let policy = CorsPolicy {
        allow_origins: vec!["*".to_string()],
        allow_methods: vec!["GET".to_string()],
        allow_headers: vec![],
        allow_credentials: true,
        max_age: None,
    };

    // credentials + wildcard 应拒绝
    let result = check_cors(&policy, &origin, "GET", &[]);
    assert!(!result.allowed);
    assert!(result.reason.contains("credentials"));
}

/// CORS + URL 解析：非默认端口处理
#[test]
fn test_cors_non_default_port() {
    let origin = Origin::parse("http://app.example.com:3000").unwrap();
    let policy = CorsPolicy {
        allow_origins: vec!["http://app.example.com:3000".to_string()],
        allow_methods: vec!["GET".to_string()],
        allow_headers: vec![],
        allow_credentials: false,
        max_age: None,
    };

    let result = check_cors(&policy, &origin, "GET", &[]);
    assert!(result.allowed);
}

/// CORS + 自定义头管线
#[test]
fn test_cors_custom_header_pipeline() {
    let origin = Origin::parse("http://app.example.com").unwrap();
    let policy = CorsPolicy {
        allow_origins: vec!["http://app.example.com".to_string()],
        allow_methods: vec!["POST".to_string()],
        allow_headers: vec!["X-Auth-Token".to_string()],
        allow_credentials: false,
        max_age: None,
    };

    // 允许的自定义头
    let result = check_cors(&policy, &origin, "POST", &[("X-Auth-Token".into(), "secret".into())]);
    assert!(result.allowed);

    // 不允许的自定义头
    let result = check_cors(&policy, &origin, "POST", &[("X-Forbidden".into(), "value".into())]);
    assert!(!result.allowed);
}

// ───────────────────── HSTS + URL 升级管线 ─────────────────────

/// HSTS 指令解析 + URL 升级管线
#[test]
fn test_hsts_parse_and_upgrade() {
    let directive = HstsDirective::parse("max-age=31536000; includeSubDomains").unwrap();
    assert_eq!(directive.max_age, 31536000);
    assert!(directive.include_subdomains);

    let mut store = HstsStore::new();
    store.register("example.com", directive);

    // HTTP URL 应升级（返回 Some(https_url)）
    let upgraded = store.should_upgrade("http://example.com/page");
    assert!(upgraded.is_some());
    assert_eq!(upgraded.unwrap(), "https://example.com/page");

    // 子域名也应升级
    let sub_upgraded = store.should_upgrade("http://sub.example.com/api");
    assert!(sub_upgraded.is_some());

    // HTTPS URL 不需要升级（返回 None）
    assert!(store.should_upgrade("https://example.com/page").is_none());

    // 未知域名不升级
    assert!(store.should_upgrade("http://other.com/page").is_none());
}

/// HSTS max-age=0 删除策略管线
#[test]
fn test_hsts_delete_policy() {
    let mut store = HstsStore::new();
    let directive = HstsDirective::parse("max-age=31536000").unwrap();
    store.register("example.com", directive);
    assert!(store.should_upgrade("http://example.com/page").is_some());

    // max-age=0 删除 HSTS 记录
    let delete = HstsDirective::parse("max-age=0").unwrap();
    store.register("example.com", delete);
    assert!(store.should_upgrade("http://example.com/page").is_none());
}

/// HSTS + URL 解析管线：upgrade_to_https 辅助函数
#[test]
fn test_hsts_upgrade_to_https_helper() {
    assert_eq!(
        upgrade_to_https("http://example.com/page"),
        Some("https://example.com/page".to_string())
    );
    assert_eq!(upgrade_to_https("https://example.com/page"), None);
    assert_eq!(upgrade_to_https("ftp://example.com/file"), None);
}

// ───────────────────── 混合内容 + Origin 管线 ─────────────────────

/// 混合内容检测：HTTPS 页面加载 HTTP 资源
#[test]
fn test_mixed_content_detection() {
    let https_origin = Origin::parse("https://example.com").unwrap();

    // HTTPS 页面 + HTTP 资源 = 混合内容
    assert!(is_mixed_content(&https_origin, "http://cdn.com/lib.js"));
    // HTTPS 页面 + HTTPS 资源 = 安全
    assert!(!is_mixed_content(&https_origin, "https://cdn.com/lib.js"));
    // HTTP 页面 + HTTP 资源 = 非混合内容
    let http_origin = Origin::parse("http://example.com").unwrap();
    assert!(!is_mixed_content(&http_origin, "http://cdn.com/lib.js"));
}

/// 混合内容分级检查管线
#[test]
fn test_mixed_content_blockable_vs_optionally_blockable() {
    let https_origin = Origin::parse("https://example.com").unwrap();

    // 阻塞型混合内容（script, connect, iframe 等）
    let script_status = check_mixed_content(&https_origin, "http://evil.com/steal.js", "script");
    assert!(matches!(script_status, MixedContentStatus::Blockable));

    // 可选阻塞型混合内容（img, audio, video 等）
    let img_status = check_mixed_content(&https_origin, "http://cdn.com/img.png", "img");
    assert!(matches!(img_status, MixedContentStatus::OptionallyBlockable));
}

// ───────────────────── 沙箱属性管线 ─────────────────────

/// iframe sandbox 属性解析 + 导航限制管线
#[test]
fn test_sandbox_navigation_restriction() {
    // 空 sandbox="" → 最严格沙箱，无任何标志
    let empty_sandbox = IframeSandbox::parse("");
    assert!(!check_sandbox_navigation(&empty_sandbox, true));
    assert!(!check_sandbox_navigation(&empty_sandbox, false));

    // 仅有 allow-scripts → 不允许导航
    let scripts_only = IframeSandbox::parse("allow-scripts");
    assert!(!check_sandbox_navigation(&scripts_only, true));

    // 允许顶部导航
    let with_top_nav = IframeSandbox::parse("allow-scripts allow-top-navigation");
    assert!(check_sandbox_navigation(&with_top_nav, true));

    // 允许用户激活导航
    let with_user_nav = IframeSandbox::parse("allow-scripts allow-top-navigation-by-user-activation");
    assert!(!check_sandbox_navigation(&with_user_nav, false)); // 非用户激活 → 拒绝
    assert!(check_sandbox_navigation(&with_user_nav, true)); // 用户激活 → 允许
}

/// iframe sandbox 弹窗限制管线
#[test]
fn test_sandbox_popup_restriction() {
    let strict = IframeSandbox::parse("allow-scripts");
    assert!(!check_sandbox_popup(&strict));

    let with_popups = IframeSandbox::parse("allow-scripts allow-popups");
    assert!(check_sandbox_popup(&with_popups));
}

/// sandbox 标志解析完整性
#[test]
fn test_sandbox_flag_parsing() {
    let sandbox = IframeSandbox::parse("allow-scripts allow-same-origin allow-forms allow-popups");
    assert!(sandbox.has_flag(IframeSandboxFlag::AllowScripts));
    assert!(sandbox.has_flag(IframeSandboxFlag::AllowSameOrigin));
    assert!(sandbox.has_flag(IframeSandboxFlag::AllowForms));
    assert!(sandbox.has_flag(IframeSandboxFlag::AllowPopups));
    assert!(!sandbox.has_flag(IframeSandboxFlag::AllowTopNavigation));
    assert!(!sandbox.has_flag(IframeSandboxFlag::AllowModals));
}

// ───────────────────── 权限模型 + Origin 隔离管线 ─────────────────────

/// 权限按 origin 隔离管线
#[test]
fn test_permission_origin_isolation() {
    let mut mgr = PermissionManager::new();
    let origin_a = Origin::parse("https://site-a.com").unwrap();
    let origin_b = Origin::parse("https://site-b.com").unwrap();

    // site-a 授予摄像头权限
    let state = mgr.grant(&origin_a, PermissionName::Camera, 1000);
    assert_eq!(state, PermissionState::Granted);

    // site-a 有摄像头权限
    assert_eq!(mgr.query(&origin_a, PermissionName::Camera), PermissionState::Granted);

    // site-b 无摄像头权限（隔离）
    assert_eq!(mgr.query(&origin_b, PermissionName::Camera), PermissionState::Prompt);
}

/// 权限授予/拒绝/撤销管线
#[test]
fn test_permission_grant_deny_revoke() {
    let mut mgr = PermissionManager::new();
    let origin = Origin::parse("https://example.com").unwrap();

    // 初始状态：Prompt
    assert_eq!(mgr.query(&origin, PermissionName::Geolocation), PermissionState::Prompt);

    // 授予
    mgr.grant(&origin, PermissionName::Geolocation, 1000);
    assert_eq!(
        mgr.query(&origin, PermissionName::Geolocation),
        PermissionState::Granted
    );

    // 拒绝
    mgr.deny(&origin, PermissionName::Geolocation, 2000);
    assert_eq!(mgr.query(&origin, PermissionName::Geolocation), PermissionState::Denied);

    // 撤销（回到 Prompt）
    mgr.revoke(&origin, PermissionName::Geolocation);
    assert_eq!(mgr.query(&origin, PermissionName::Geolocation), PermissionState::Prompt);
}

/// 多权限类型管线
#[test]
fn test_permission_multiple_types() {
    let mut mgr = PermissionManager::new();
    let origin = Origin::parse("https://example.com").unwrap();

    // 不同权限类型互不影响
    mgr.grant(&origin, PermissionName::Camera, 1000);
    mgr.deny(&origin, PermissionName::Notifications, 1000);

    assert_eq!(mgr.query(&origin, PermissionName::Camera), PermissionState::Granted);
    assert_eq!(
        mgr.query(&origin, PermissionName::Notifications),
        PermissionState::Denied
    );
    assert_eq!(mgr.query(&origin, PermissionName::Microphone), PermissionState::Prompt);
}

// ───────────────────── 站点隔离 + Origin 管线 ─────────────────────

/// 站点隔离：同站共享进程
#[test]
fn test_site_isolation_same_site() {
    let a = Origin::parse("https://sub1.example.com").unwrap();
    let b = Origin::parse("https://sub2.example.com").unwrap();

    // 同站判断
    assert!(Site::is_same_site(&a, &b));

    // 按站点隔离策略：同站共享进程
    let mut mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
    let pid1 = mgr.get_or_create_process(&a);
    let pid2 = mgr.get_or_create_process(&b);
    assert_eq!(pid1, pid2, "同站点应共享渲染进程");
}

/// 站点隔离：跨站独立进程
#[test]
fn test_site_isolation_cross_site() {
    let a = Origin::parse("https://example.com").unwrap();
    let b = Origin::parse("https://other.com").unwrap();

    // 跨站判断
    assert!(!Site::is_same_site(&a, &b));

    // 按站点隔离策略：跨站独立进程
    let mut mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);
    let pid1 = mgr.get_or_create_process(&a);
    let pid2 = mgr.get_or_create_process(&b);
    assert_ne!(pid1, pid2, "跨站点应使用独立渲染进程");
}

/// 严格源隔离：不同源必须独立进程
#[test]
fn test_strict_origin_isolation() {
    let a = Origin::parse("https://sub1.example.com").unwrap();
    let b = Origin::parse("https://sub2.example.com").unwrap();

    // 严格源隔离：即使同站，不同源也独立进程
    let mut mgr = SiteIsolationManager::new(IsolationPolicy::StrictOriginIsolated);
    let pid1 = mgr.get_or_create_process(&a);
    let pid2 = mgr.get_or_create_process(&b);
    assert_ne!(pid1, pid2, "严格源隔离下不同源应使用独立进程");
}

/// 站点隔离：跨站 iframe 需要独立进程
#[test]
fn test_site_isolation_iframe() {
    let parent = Origin::parse("https://example.com").unwrap();
    let iframe = Origin::parse("https://ads.com").unwrap();

    let mut mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);

    // 父页面进程
    let _parent_pid = mgr.get_or_create_process(&parent);
    // iframe 进程（跨站 → 独立进程）
    let _iframe_pid = mgr.get_or_create_process(&iframe);

    // 验证 iframe 需要独立进程
    assert!(mgr.needs_separate_process(&parent, &iframe));

    // 验证跨站 DOM 访问被阻止
    assert!(
        !mgr.can_access_parent_dom(&parent, &iframe),
        "跨站 iframe 不应访问父页面 DOM"
    );
}

// ───────────────────── COOP + COEP 跨源隔离管线 ─────────────────────

/// 跨源隔离状态判断管线
#[test]
fn test_cross_origin_isolation_pipeline() {
    let coop_same = zero_security::CoopPolicy::SameOrigin;
    let coep_require = CoepPolicy::RequireCorp;

    // SameOrigin + RequireCorp → 跨源隔离
    assert!(is_cross_origin_isolated(coop_same, coep_require));

    // 不满足条件 → 非跨源隔离
    assert!(!is_cross_origin_isolated(
        zero_security::CoopPolicy::UnsafeNone,
        coep_require
    ));
    assert!(!is_cross_origin_isolated(coop_same, CoepPolicy::UnsafeNone));
    assert!(!is_cross_origin_isolated(
        zero_security::CoopPolicy::UnsafeNone,
        CoepPolicy::UnsafeNone
    ));
}

// ───────────────────── 复合安全管线 ─────────────────────

/// 完整安全管线：CSP + CORS + 混合内容联合检查
#[test]
fn test_full_security_pipeline() {
    let page_origin = Origin::parse("https://example.com").unwrap();
    let csp = ContentSecurityPolicy::parse(
        "default-src 'self'; script-src 'self' https://cdn.example.com; style-src 'self' 'unsafe-inline'; connect-src 'self' https://api.example.com",
    );
    let cors_policy = CorsPolicy {
        allow_origins: vec!["https://example.com".to_string()],
        allow_methods: vec!["GET".to_string(), "POST".to_string()],
        allow_headers: vec!["Content-Type".to_string()],
        allow_credentials: true,
        max_age: Some(3600),
    };

    // 1. CSP 检查：同源脚本允许
    assert!(csp.is_resource_allowed("script", "https://example.com/app.js", Some(&page_origin)));

    // 2. CSP 检查：CDN 脚本允许
    assert!(csp.is_resource_allowed("script", "https://cdn.example.com/lib.js", Some(&page_origin)));

    // 3. CSP 检查：第三方脚本拒绝
    assert!(!csp.is_resource_allowed("script", "https://evil.com/steal.js", Some(&page_origin)));

    // 4. 混合内容检查：HTTP 脚本是混合内容
    assert!(is_mixed_content(&page_origin, "http://cdn.example.com/lib.js"));

    // 5. CORS 检查：同源请求允许
    let cors_result = check_cors(&cors_policy, &page_origin, "GET", &[]);
    assert!(cors_result.allowed);

    // 6. CORS 检查：跨域拒绝
    let evil = Origin::parse("https://evil.com").unwrap();
    let cors_result = check_cors(&cors_policy, &evil, "GET", &[]);
    assert!(!cors_result.allowed);

    // 7. CSP 内联样式允许（unsafe-inline）
    assert!(csp.is_inline_style_allowed(None, None));

    // 8. CSP 内联脚本拒绝（无 unsafe-inline）
    assert!(!csp.is_inline_script_allowed(None, None));
}

/// HSTS + 混合内容 + CSP 联合管线
#[test]
fn test_hsts_mixed_content_csp_pipeline() {
    let mut hsts_store = HstsStore::new();
    let hsts = HstsDirective::parse("max-age=31536000; includeSubDomains").unwrap();
    hsts_store.register("secure.example.com", hsts);

    let csp = ContentSecurityPolicy::parse("default-src 'self' https:; img-src 'self' data: https:");
    let origin = Origin::parse("https://secure.example.com").unwrap();

    // 1. HTTP URL 需要 HSTS 升级
    let upgraded_url = hsts_store.should_upgrade("http://secure.example.com/api");
    assert!(upgraded_url.is_some());

    // 2. 升级后是 HTTPS → 不是混合内容
    let upgraded = upgrade_to_https("http://secure.example.com/api").unwrap();
    assert!(!is_mixed_content(&origin, &upgraded));

    // 3. CSP 允许 HTTPS 资源
    assert!(csp.is_resource_allowed("script", &upgraded, Some(&origin)));

    // 4. CSP 允许 data: 图片
    assert!(csp.is_resource_allowed("img", "data:image/png;base64,abc", Some(&origin)));
}

/// 权限 + 站点隔离联合管线
#[test]
fn test_permission_site_isolation_pipeline() {
    let mut perm_mgr = PermissionManager::new();
    let mut site_mgr = SiteIsolationManager::new(IsolationPolicy::SiteIsolated);

    let main_origin = Origin::parse("https://example.com").unwrap();
    let ad_origin = Origin::parse("https://ads.com").unwrap();

    // 主站授予摄像头权限
    perm_mgr.grant(&main_origin, PermissionName::Camera, 1000);

    // 广告站点无权限（隔离）
    assert_eq!(
        perm_mgr.query(&ad_origin, PermissionName::Camera),
        PermissionState::Prompt
    );

    // 广告站点使用独立进程
    let main_pid = site_mgr.get_or_create_process(&main_origin);
    let ad_pid = site_mgr.get_or_create_process(&ad_origin);
    assert_ne!(main_pid, ad_pid);

    // 广告进程不能访问主站 DOM
    assert!(!site_mgr.can_access_parent_dom(&main_origin, &ad_origin));
}

/// Origin + URL 解析一致性管线
#[test]
fn test_origin_url_parse_consistency() {
    // 标准端口
    let origin = Origin::parse("https://example.com").unwrap();
    assert!(origin.is_secure());
    assert_eq!(origin.scheme, "https");
    assert_eq!(origin.host, "example.com");

    // 非标准端口
    let origin_port = Origin::parse("http://example.com:8080").unwrap();
    assert_eq!(origin_port.port, 8080);
    assert!(!origin_port.is_secure());

    // 同源比较
    let origin_default = Origin::parse("https://example.com:443").unwrap();
    assert!(origin.is_same_origin(&origin_default));
}

/// HSTS register_from_header 便捷管线
#[test]
fn test_hsts_register_from_header() {
    let mut store = HstsStore::new();

    // 通过 header 注册 HSTS
    assert!(store.register_from_header("example.com", "max-age=86400; includeSubDomains"));
    assert!(store.should_upgrade("http://example.com/page").is_some());
    assert!(store.should_upgrade("http://sub.example.com/page").is_some());

    // 无效 header 不注册
    assert!(!store.register_from_header("invalid.com", ""));
    assert!(store.should_upgrade("http://invalid.com/page").is_none());
}

/// CSP + 混合内容联合管线：upgrade-insecure-requests 语义
#[test]
fn test_csp_mixed_content_upgrade_semantic() {
    let https_origin = Origin::parse("https://example.com").unwrap();
    let csp = ContentSecurityPolicy::parse("default-src 'self' https:; img-src 'self' data: https:");

    // 模拟 upgrade-insecure-requests：HTTP img 被升级后检查 CSP
    let http_img = "http://cdn.example.com/img.png";
    assert!(is_mixed_content(&https_origin, http_img));

    // 升级后
    let upgraded = upgrade_to_https(http_img).unwrap();
    assert!(!is_mixed_content(&https_origin, &upgraded));
    assert!(csp.is_resource_allowed("img", &upgraded, Some(&https_origin)));
}

/// 站点隔离：无隔离策略共享进程
#[test]
fn test_site_isolation_none_policy() {
    let a = Origin::parse("https://example.com").unwrap();
    let b = Origin::parse("https://other.com").unwrap();

    let mut mgr = SiteIsolationManager::new(IsolationPolicy::None);
    let pid1 = mgr.get_or_create_process(&a);
    let pid2 = mgr.get_or_create_process(&b);

    // 无隔离策略：所有源共享同一进程
    assert_eq!(pid1, pid2, "无隔离策略下所有源应共享进程");
}

/// 权限 + revoke_all_for_origin 管线
#[test]
fn test_permission_revoke_all_for_origin() {
    let mut mgr = PermissionManager::new();
    let origin = Origin::parse("https://example.com").unwrap();

    mgr.grant(&origin, PermissionName::Camera, 1000);
    mgr.grant(&origin, PermissionName::Geolocation, 1000);
    mgr.grant(&origin, PermissionName::Notifications, 1000);

    assert_eq!(mgr.len(), 3);

    // 撤销所有权限
    mgr.revoke_all_for_origin(&origin);
    assert!(mgr.is_empty());
    assert_eq!(mgr.query(&origin, PermissionName::Camera), PermissionState::Prompt);
}

// ───────────────────── SecurityContext 集成管线 ─────────────────────

/// SecurityContext：HSTS 预加载列表加载验证
#[test]
fn test_security_context_preload_list_loaded() {
    let ctx = SecurityContext::new();
    // 预加载列表应包含 40+ 条目
    assert!(ctx.hsts_count() >= 30, "预加载列表应至少包含 30 个域名");
}

/// SecurityContext：HSTS 预加载域名自动升级
#[test]
fn test_security_context_hsts_preload_upgrade() {
    let mut ctx = SecurityContext::new();
    // github.com 在预加载列表中
    let result = ctx.check_resource_url("http://github.com/user/repo", "document");
    assert!(
        matches!(result, ResourceCheckResult::Upgraded(ref url) if url.starts_with("https://")),
        "预加载域名 HTTP 应被升级为 HTTPS"
    );
}

/// SecurityContext：HSTS 预加载子域名升级（includeSubDomains）
#[test]
fn test_security_context_hsts_subdomain_upgrade() {
    let mut ctx = SecurityContext::new();
    // cloudflare.com 预加载且 includeSubDomains=true
    let result = ctx.check_resource_url("http://cdn.cloudflare.com/resource.js", "script");
    assert!(
        matches!(result, ResourceCheckResult::Upgraded(ref url) if url.contains("https://")),
        "子域名 HTTP 应被升级为 HTTPS"
    );
}

/// SecurityContext：运行时 HSTS 注册 + 升级
#[test]
fn test_security_context_runtime_hsts_registration() {
    let mut ctx = SecurityContext::new();
    // 自定义域名不在预加载列表中
    let result1 = ctx.check_resource_url("http://custom-site.com/page", "document");
    assert_eq!(result1, ResourceCheckResult::Allow, "未注册域名应允许");

    // 从响应头注册 HSTS
    assert!(ctx.register_hsts("custom-site.com", "max-age=31536000; includeSubDomains"));

    // 注册后应被升级
    let result2 = ctx.check_resource_url("http://custom-site.com/page", "document");
    assert!(
        matches!(result2, ResourceCheckResult::Upgraded(ref url) if url.starts_with("https://")),
        "注册后 HTTP 应被升级"
    );

    // 子域名也应被升级
    let result3 = ctx.check_resource_url("http://sub.custom-site.com/page", "document");
    assert!(
        matches!(result3, ResourceCheckResult::Upgraded(ref url) if url.starts_with("https://")),
        "includeSubDomains 子域名应被升级"
    );
}

/// SecurityContext：混合内容阻止 — Blockable 类型
#[test]
fn test_security_context_mixed_content_blockable() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://secure.example.com");

    // script 应被阻止
    let r1 = ctx.check_resource_url("http://evil.com/steal.js", "script");
    assert!(matches!(r1, ResourceCheckResult::Blocked(_)), "script 应被阻止");

    // style 应被阻止
    let r2 = ctx.check_resource_url("http://evil.com/style.css", "style");
    assert!(matches!(r2, ResourceCheckResult::Blocked(_)), "style 应被阻止");

    // connect (XHR/fetch) 应被阻止
    let r3 = ctx.check_resource_url("http://evil.com/api/data", "connect");
    assert!(matches!(r3, ResourceCheckResult::Blocked(_)), "connect 应被阻止");

    // font 应被阻止
    let r4 = ctx.check_resource_url("http://evil.com/font.woff2", "font");
    assert!(matches!(r4, ResourceCheckResult::Blocked(_)), "font 应被阻止");

    // iframe 应被阻止
    let r5 = ctx.check_resource_url("http://evil.com/embed", "iframe");
    assert!(matches!(r5, ResourceCheckResult::Blocked(_)), "iframe 应被阻止");
}

/// SecurityContext：混合内容自动升级 — OptionallyBlockable 类型
#[test]
fn test_security_context_mixed_content_upgradeable() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://secure.example.com");

    // img 应被升级
    let r1 = ctx.check_resource_url("http://cdn.com/photo.jpg", "img");
    assert_eq!(
        r1,
        ResourceCheckResult::Upgraded("https://cdn.com/photo.jpg".to_string())
    );

    // audio 应被升级
    let r2 = ctx.check_resource_url("http://cdn.com/audio.mp3", "audio");
    assert_eq!(
        r2,
        ResourceCheckResult::Upgraded("https://cdn.com/audio.mp3".to_string())
    );

    // video 应被升级
    let r3 = ctx.check_resource_url("http://cdn.com/video.mp4", "video");
    assert_eq!(
        r3,
        ResourceCheckResult::Upgraded("https://cdn.com/video.mp4".to_string())
    );
}

/// SecurityContext：HSTS 升级优先于混合内容检查
#[test]
fn test_security_context_hsts_upgrade_before_mixed_content() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://secure.example.com");

    // github.com 在 HSTS 预加载列表中，HTTP script 先被 HSTS 升级
    let result = ctx.check_resource_url("http://github.com/analytics.js", "script");
    // HSTS 升级为 HTTPS → 不再是混合内容 → 应为 Upgraded
    assert!(
        matches!(result, ResourceCheckResult::Upgraded(ref url) if url == "https://github.com/analytics.js"),
        "HSTS 升级后应不再是混合内容"
    );
}

/// SecurityContext：data: 和 blob: URI 不被阻止
#[test]
fn test_security_context_safe_uri_schemes() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://secure.example.com");

    // data: URI 安全
    let r1 = ctx.check_resource_url("data:text/html,<h1>Hi</h1>", "script");
    assert_eq!(r1, ResourceCheckResult::Allow);

    // blob: URI 安全
    let r2 = ctx.check_resource_url("blob:https://example.com/abc-123", "script");
    assert_eq!(r2, ResourceCheckResult::Allow);

    // 相对 URL 安全
    let r3 = ctx.check_resource_url("scripts/app.js", "script");
    assert_eq!(r3, ResourceCheckResult::Allow);
}

/// SecurityContext：HTTP 页面不检查混合内容
#[test]
fn test_security_context_http_page_no_mixed_content_check() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("http://insecure.example.com");

    // HTTP 页面加载 HTTP 资源 — 全部允许
    let r1 = ctx.check_resource_url("http://evil.com/script.js", "script");
    assert_eq!(r1, ResourceCheckResult::Allow);

    let r2 = ctx.check_resource_url("http://evil.com/style.css", "style");
    assert_eq!(r2, ResourceCheckResult::Allow);
}

/// SecurityContext：页面源清除后不再阻止
#[test]
fn test_security_context_clear_origin_removes_blocking() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://secure.example.com");

    let r1 = ctx.check_resource_url("http://evil.com/script.js", "script");
    assert!(matches!(r1, ResourceCheckResult::Blocked(_)));

    ctx.clear_page_origin();

    let r2 = ctx.check_resource_url("http://evil.com/script.js", "script");
    assert_eq!(r2, ResourceCheckResult::Allow);
}

/// SecurityContext + HSTS + Origin 一致性管线
#[test]
fn test_security_context_hsts_origin_consistency() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://example.com");

    // 1. 预加载域名 HSTS 升级
    let r1 = ctx.check_resource_url("http://google.com/search?q=test", "connect");
    assert!(matches!(r1, ResourceCheckResult::Upgraded(_)));

    // 2. HTTPS 资源允许
    let r2 = ctx.check_resource_url("https://cdn.com/lib.js", "script");
    assert_eq!(r2, ResourceCheckResult::Allow);

    // 3. 非 HSTS HTTP + Blockable 类型 → 阻止
    let r3 = ctx.check_resource_url("http://unknown.com/script.js", "script");
    assert!(matches!(r3, ResourceCheckResult::Blocked(_)));

    // 4. 非 HSTS HTTP + OptionallyBlockable → 升级
    let r4 = ctx.check_resource_url("http://unknown.com/photo.jpg", "img");
    assert!(matches!(r4, ResourceCheckResult::Upgraded(_)));
}

/// SecurityContext + CSP 组合管线
#[test]
fn test_security_context_with_csp() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://example.com");

    // CSP 策略限制 script-src
    let csp = ContentSecurityPolicy::parse("default-src 'self'; script-src 'self' https://trusted.com");

    // SecurityContext 检查通过（HTTPS URL）+ CSP 检查通过（同源）
    let url = "https://example.com/app.js";
    let sec_result = ctx.check_resource_url(url, "script");
    assert_eq!(sec_result, ResourceCheckResult::Allow);
    let origin = Origin::parse("https://example.com").unwrap();
    assert!(csp.is_resource_allowed("script", url, Some(&origin)));

    // SecurityContext 允许 + CSP 拒绝（跨域脚本）
    let cross_url = "https://evil.com/steal.js";
    let sec_result2 = ctx.check_resource_url(cross_url, "script");
    assert_eq!(sec_result2, ResourceCheckResult::Allow);
    assert!(!csp.is_resource_allowed("script", cross_url, Some(&origin)));

    // SecurityContext 阻止（混合内容）+ CSP 不需要检查
    let http_url = "http://evil.com/steal.js";
    let sec_result3 = ctx.check_resource_url(http_url, "script");
    assert!(matches!(sec_result3, ResourceCheckResult::Blocked(_)));
}

/// SecurityContext：HSTS 过期清理管线
#[test]
fn test_security_context_hsts_cleanup() {
    let mut ctx = SecurityContext::new();
    // 注册一个自定义域名
    ctx.register_hsts("test.com", "max-age=31536000");
    let count_before = ctx.hsts_count();

    // 清理：预加载策略不过期，自定义策略刚注册也不过期
    let cleaned = ctx.cleanup_hsts();
    assert_eq!(cleaned, 0, "未过期策略不应被清理");
    assert_eq!(ctx.hsts_count(), count_before);
}

/// SecurityContext：多资源类型混合内容完整矩阵
#[test]
fn test_security_context_mixed_content_full_matrix() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://bank.com");

    // Blockable 类型全部被阻止
    for resource_type in &[
        "script", "style", "connect", "font", "iframe", "object", "xhr", "fetch", "worker",
    ] {
        let result = ctx.check_resource_url("http://attacker.com/resource", resource_type);
        assert!(
            matches!(result, ResourceCheckResult::Blocked(_)),
            "类型 '{resource_type}' 在 HTTPS 页面上加载 HTTP 资源应被阻止"
        );
    }

    // OptionallyBlockable 类型全部被升级
    for resource_type in &["img", "audio", "video", "media"] {
        let result = ctx.check_resource_url("http://cdn.com/resource", resource_type);
        assert!(
            matches!(result, ResourceCheckResult::Upgraded(ref url) if url.starts_with("https://")),
            "类型 '{resource_type}' 在 HTTPS 页面上加载 HTTP 资源应被升级"
        );
    }
}
