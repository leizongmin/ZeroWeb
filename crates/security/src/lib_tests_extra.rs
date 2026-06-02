use super::*;
// ═══════════════════════════════════════════════════════════════════════
// Additional edge-case tests (round 15)
// ═══════════════════════════════════════════════════════════════════════

/// 测试 CORS check_cors 对 allow_origins 列表中大小写不同的源字符串的匹配。
///
/// check_cors 使用 eq_ignore_ascii_case 进行源字符串匹配，
/// 因此 "HTTP://EXAMPLE.COM" 应与 "http://example.com" 匹配。
/// 验证 Origin 格式化后的源字符串与 allow_origins 中的不同大小写
/// 仍能通过 CORS 检查。
#[test]
fn test_cors_origin_case_insensitive_matching() {
    let policy = CorsPolicy {
        allow_origins: vec!["HTTP://EXAMPLE.COM".to_string()],
        allow_methods: vec!["GET".to_string()],
        allow_headers: vec![],
        allow_credentials: false,
        max_age: None,
    };
    let origin = Origin::parse("http://example.com").unwrap();
    // check_cors 使用 eq_ignore_ascii_case → 大写 allow_origins 应匹配
    let result = check_cors(&policy, &origin, "GET", &[]);
    assert!(result.allowed, "allow_origins 中大写的源字符串应忽略大小写匹配");

    // 反向：allow_origins 为小写，Origin 格式化也为小写 → 应匹配
    let policy_lower = CorsPolicy {
        allow_origins: vec!["http://example.com".to_string()],
        allow_methods: vec!["GET".to_string()],
        allow_headers: vec![],
        allow_credentials: false,
        max_age: None,
    };
    let result_lower = check_cors(&policy_lower, &origin, "GET", &[]);
    assert!(result_lower.allowed, "小写 allow_origins 应正常匹配");
}

/// 测试 CSP navigate-to 指令在存在时正确限制导航目标。
///
/// navigate-to 指令限制页面可以导航到哪些地址。
/// 当指令存在且值为 'self' 时，只有同源地址允许导航。
/// 当指令不存在时，导航不受限制（不回退到 default-src）。
#[test]
fn test_csp_navigate_to_restriction_with_self() {
    let csp = ContentSecurityPolicy::parse("navigate-to 'self'");
    let doc_origin = Origin::parse("https://example.com").unwrap();

    // 同源 URL → 允许
    assert!(
        csp.is_navigate_to_allowed("https://example.com/page", Some(&doc_origin)),
        "navigate-to 'self' 应允许同源导航"
    );
    // 跨源 URL → 拒绝
    assert!(
        !csp.is_navigate_to_allowed("https://evil.com/page", Some(&doc_origin)),
        "navigate-to 'self' 应阻止跨源导航"
    );
    // 相对 URL → 视为同源
    assert!(
        csp.is_navigate_to_allowed("/local", Some(&doc_origin)),
        "相对 URL 在 navigate-to 'self' 下应视为同源"
    );
    // 无 navigate-to 指令 → 不限制
    let csp_no_nav = ContentSecurityPolicy::parse("default-src 'none'");
    assert!(
        csp_no_nav.is_navigate_to_allowed("https://evil.com", None),
        "无 navigate-to 指令时导航不受限制"
    );
}

/// 测试沙箱 allow-popups-to-escape-sandbox 标志：弹窗不受父沙箱限制。
///
/// 当 iframe 沙箱设置了 allow-popups-to-escape-sandbox 时，
/// 从该 iframe 打开的弹窗不继承沙箱限制。
/// 该标志本身不影响当前 iframe 的权限（脚本、表单等仍需各自标志）。
#[test]
fn test_sandbox_popups_escape_flag() {
    let sandbox = IframeSandbox::parse("allow-scripts allow-popups allow-popups-to-escape-sandbox");

    // 核心权限不受此标志影响
    assert!(sandbox.allows_scripts(), "allow-scripts 应允许脚本");
    assert!(sandbox.allows_popups(), "allow-popups 应允许弹窗");
    assert!(sandbox.has_flag(IframeSandboxFlag::AllowPopupsToEscapeSandbox));

    // 其他权限仍被禁止
    assert!(!sandbox.allows_same_origin(), "不应允许同源访问");
    assert!(!sandbox.allows_forms(), "不应允许表单提交");
    assert!(!sandbox.allows_top_navigation(), "不应允许顶层导航");

    // 缺少 allow-same-origin → 不透明源
    let origin = Origin::parse("https://example.com").unwrap();
    assert_eq!(sandbox.effective_origin(&origin), SandboxOrigin::Opaque);

    // 对比：无此标志的弹窗沙箱
    let sandbox_no_escape = IframeSandbox::parse("allow-popups");
    assert!(!sandbox_no_escape.has_flag(IframeSandboxFlag::AllowPopupsToEscapeSandbox));
}

/// 测试 COOP 跨源场景下 SameOriginIncludingPopups 与 UnsafeNone 的组合。
///
/// 当响应方为 SameOriginIncludingPopups 时，跨源请求始终被阻止。
/// 当导航方为 SameOriginIncludingPopups 且响应方为 SameOrigin 时，
/// 跨源也始终被阻止。
#[test]
fn test_coop_same_origin_including_popups_cross_origin_variants() {
    // 跨源 + 导航方 UnsafeNone + 响应方 SameOriginIncludingPopups → 阻止
    assert_eq!(
        evaluate_coop(CoopPolicy::UnsafeNone, CoopPolicy::SameOriginIncludingPopups, false),
        CoopResult::Blocked,
        "跨源 + 响应方 SameOriginIncludingPopups 应阻止"
    );

    // 跨源 + 导航方 SameOriginIncludingPopups + 响应方 SameOrigin → 阻止
    assert_eq!(
        evaluate_coop(CoopPolicy::SameOriginIncludingPopups, CoopPolicy::SameOrigin, false),
        CoopResult::Blocked,
        "跨源 + 导航方 SameOriginIncludingPopups + 响应方 SameOrigin 应阻止"
    );

    // 跨源 + 双方均为 SameOriginIncludingPopups → 阻止
    assert_eq!(
        evaluate_coop(
            CoopPolicy::SameOriginIncludingPopups,
            CoopPolicy::SameOriginIncludingPopups,
            false
        ),
        CoopResult::Blocked,
        "跨源 + 双方 SameOriginIncludingPopups 应阻止"
    );

    // 同源 + SameOriginIncludingPopups → 始终允许
    assert_eq!(
        evaluate_coop(CoopPolicy::SameOriginIncludingPopups, CoopPolicy::SameOrigin, true),
        CoopResult::Allowed,
        "同源应始终允许"
    );

    // parse_coop 验证
    assert_eq!(
        parse_coop("same-origin-including-popups"),
        CoopPolicy::SameOriginIncludingPopups
    );
}

/// 测试 CSP img-src 不存在时回退到 default-src 'self'，
/// 且无 document_origin 时绝对 URL 被 'self' 拒绝但相对 URL 仍通过。
///
/// 当资源类型没有对应指令时回退到 default-src。如果 default-src 为 'self'，
/// 相对 URL（不以 http:// 或 https:// 开头）通过 is_self_match 判定为同源，
/// 而绝对 URL 在无 document_origin 时无法匹配 'self'，应被阻止。
#[test]
fn test_csp_img_src_fallback_to_default_self_without_origin() {
    let csp = ContentSecurityPolicy::parse("default-src 'self'");
    // 相对 URL → is_self_match 判定为同源（不以 http/https 开头）
    assert!(
        csp.is_resource_allowed("img", "photo.jpg", None),
        "相对 URL 在 default-src 'self' 下应允许"
    );
    // 绝对 URL 无 document_origin → 无法匹配 'self' → 被阻止
    assert!(
        !csp.is_resource_allowed("img", "https://cdn.example.com/photo.jpg", None),
        "绝对 URL 无 document_origin 时 default-src 'self' 应阻止"
    );
    // 有 document_origin 后同源绝对 URL 可通过
    let doc_origin = Origin::parse("https://cdn.example.com").unwrap();
    assert!(
        csp.is_resource_allowed("img", "https://cdn.example.com/photo.jpg", Some(&doc_origin)),
        "同源绝对 URL 有 document_origin 时应允许"
    );
    // 跨源绝对 URL 仍被阻止
    assert!(
        !csp.is_resource_allowed("img", "https://evil.com/photo.jpg", Some(&doc_origin)),
        "跨源绝对 URL 在 default-src 'self' 下应被阻止"
    );
}

/// 测试 CORS check_cors 对多个自定义头部分允许时的严格拒绝。
///
/// 当请求携带两个自定义头（X-Allowed 和 X-Forbidden），
/// 但策略仅允许 X-Allowed 时，整个请求应被拒绝。
/// CORS 的头部检查是"全部允许才算通过"，而非部分通过。
#[test]
fn test_cors_partial_custom_header_rejection() {
    let policy = CorsPolicy {
        allow_origins: vec!["*".to_string()],
        allow_methods: vec!["GET".to_string()],
        allow_headers: vec!["X-Allowed".to_string()],
        allow_credentials: false,
        max_age: None,
    };
    let origin = Origin::parse("http://example.com").unwrap();

    // 两个自定义头，仅一个在 allow_headers 中 → 拒绝
    let result = check_cors(
        &policy,
        &origin,
        "GET",
        &[
            ("X-Allowed".to_string(), "ok".to_string()),
            ("X-Forbidden".to_string(), "bad".to_string()),
        ],
    );
    assert!(!result.allowed, "部分自定义头未允许时应拒绝整个请求");
    assert!(result.reason.contains("X-Forbidden"), "拒绝原因应包含未允许的头名");

    // 仅发送已允许的头 → 通过
    let result_ok = check_cors(&policy, &origin, "GET", &[("X-Allowed".to_string(), "ok".to_string())]);
    assert!(result_ok.allowed, "所有自定义头均在 allow_headers 中时应通过");

    // 无自定义头 → 通过
    let result_no_headers = check_cors(&policy, &origin, "GET", &[]);
    assert!(result_no_headers.allowed, "无自定义头时应通过");
}

/// 测试混合内容对所有 OptionallyBlockable 资源类型的完整分类覆盖。
///
/// classify_resource_type 将 img、audio、video、media 归为 OptionallyBlockable，
/// 其他所有类型归为 Blockable。验证四种可选阻塞类型均正确分类，
/// 以及空字符串资源类型的边界处理。
#[test]
fn test_mixed_content_all_optionally_blockable_types() {
    let page = Origin::parse("https://example.com").unwrap();
    let http_url = "http://cdn.example.com/resource";

    // 四种 OptionallyBlockable 类型均应返回 OptionallyBlockable
    for resource_type in &["img", "audio", "video", "media"] {
        assert_eq!(
            check_mixed_content(&page, http_url, resource_type),
            MixedContentStatus::OptionallyBlockable,
            "资源类型 '{resource_type}' 应为 OptionallyBlockable"
        );
    }

    // 空字符串资源类型不在可选阻塞列表中 → Blockable
    assert_eq!(
        check_mixed_content(&page, http_url, ""),
        MixedContentStatus::Blockable,
        "空字符串资源类型应为 Blockable"
    );

    // 所有四种类型均可通过 upgrade_to_https 升级
    let upgraded = upgrade_to_https(http_url);
    assert!(upgraded.is_some(), "HTTP URL 应可升级");
    let upgraded_url = upgraded.unwrap();
    for resource_type in &["img", "audio", "video", "media"] {
        assert_eq!(
            check_mixed_content(&page, &upgraded_url, resource_type),
            MixedContentStatus::NotMixedContent,
            "升级后资源类型 '{resource_type}' 不再是混合内容"
        );
    }
}

/// 测试沙箱 IframeSandbox::parse 对前导/尾随空格和多连续空格分隔符的容错。
///
/// parse 使用 split_whitespace 分割标志，因此应正确处理：
/// - 前导和尾随空格
/// - 多个连续空格（包括 tab 和换行符）
/// - 混合空白符分隔
/// 无效标志在空格处理后仍被过滤。
#[test]
fn test_sandbox_parse_whitespace_tolerance() {
    // 前导和尾随空格
    let sandbox = IframeSandbox::parse("  allow-scripts allow-forms  ");
    assert!(sandbox.allows_scripts(), "前导/尾随空格不应影响标志解析");
    assert!(sandbox.allows_forms());
    assert!(!sandbox.allows_same_origin());

    // 多个连续空格 + tab + 换行符分隔
    let sandbox_mixed = IframeSandbox::parse("allow-scripts\t\tallow-popups\nallow-forms");
    assert!(sandbox_mixed.allows_scripts(), "tab 分隔应被正确处理");
    assert!(sandbox_mixed.allows_popups(), "换行分隔应被正确处理");
    assert!(sandbox_mixed.allows_forms(), "多空白符分隔应被正确处理");

    // 仅空格字符串等同于严格沙箱
    let sandbox_blank = IframeSandbox::parse("   \t  \n  ");
    assert!(!sandbox_blank.allows_scripts(), "仅空白字符串应等同于严格沙箱");
    assert!(!sandbox_blank.allows_forms());
    assert!(!sandbox_blank.allows_popups());
    assert!(!sandbox_blank.allows_same_origin());

    // 空字符串也等同于严格沙箱
    let sandbox_empty = IframeSandbox::parse("");
    assert!(!sandbox_empty.allows_scripts(), "空字符串应等同于严格沙箱");
}

/// 测试 COEP Credentialless 模式下不同 CORP 状态的差异化处理。
///
/// Credentialless 模式的核心语义：
/// - NoPolicy → 允许（无凭证加载，不发送 cookies）
/// - SameOrigin → 允许（CORP 头明确允许同源）
/// - CrossOrigin → 阻止（CORP 头明确拒绝）
/// 验证三种 CORP 状态在跨源 + 无 CORS 场景下的完整行为矩阵。
#[test]
fn test_coep_credentialless_corp_status_differentiation() {
    // 跨源 + 无 CORS 场景
    let result_nopolicy = evaluate_coep(CoepPolicy::Credentialless, CorpStatus::NoPolicy, false, false);
    assert_eq!(
        result_nopolicy,
        CoepResult::Allowed,
        "Credentialless + NoPolicy → 允许（无凭证加载）"
    );

    let result_sameorigin = evaluate_coep(CoepPolicy::Credentialless, CorpStatus::SameOrigin, false, false);
    assert_eq!(
        result_sameorigin,
        CoepResult::Allowed,
        "Credentialless + SameOrigin CORP → 允许"
    );

    let result_crossorigin = evaluate_coep(CoepPolicy::Credentialless, CorpStatus::CrossOrigin, false, false);
    assert_eq!(
        result_crossorigin,
        CoepResult::Blocked,
        "Credentialless + CrossOrigin CORP → 阻止"
    );

    // 对比 RequireCorp：NoPolicy 和 CrossOrigin 都被阻止
    let require_nopolicy = evaluate_coep(CoepPolicy::RequireCorp, CorpStatus::NoPolicy, false, false);
    assert_eq!(
        require_nopolicy,
        CoepResult::Blocked,
        "RequireCorp + NoPolicy → 阻止（与 Credentialless 不同）"
    );

    // parse_corp None → NoPolicy
    assert_eq!(parse_corp(None), CorpStatus::NoPolicy);
}

/// 测试混合内容 upgrade_to_https 对带查询参数和片段的 URL 的处理。
///
/// HTTP URL 包含查询字符串（?key=value）或片段（#section）时，
/// upgrade_to_https 应仅替换 scheme 前缀，保留其余部分完整。
/// 带认证信息的 URL（user:pass@host）也应正确升级。
#[test]
fn test_mixed_content_upgrade_preserves_query_and_fragment() {
    let page = Origin::parse("https://example.com").unwrap();

    // 带查询参数的 HTTP URL → 升级后保留查询参数
    let url_with_query = "http://api.example.com/data?key=value&sort=asc";
    assert!(
        is_mixed_content(&page, url_with_query),
        "带查询参数的 HTTP URL 应为混合内容"
    );
    let upgraded = upgrade_to_https(url_with_query);
    assert_eq!(
        upgraded,
        Some("https://api.example.com/data?key=value&sort=asc".to_string()),
        "升级后应保留查询参数"
    );

    // 带片段的 HTTP URL → 升级后保留片段
    let url_with_fragment = "http://example.com/page#section-1";
    assert!(is_mixed_content(&page, url_with_fragment));
    let upgraded_frag = upgrade_to_https(url_with_fragment);
    assert_eq!(
        upgraded_frag,
        Some("https://example.com/page#section-1".to_string()),
        "升级后应保留片段标识符"
    );

    // 带查询参数和片段的 HTTP URL → 两者均保留
    let url_full = "http://cdn.example.com/api/v2?key=val#result";
    let upgraded_full = upgrade_to_https(url_full);
    assert_eq!(
        upgraded_full,
        Some("https://cdn.example.com/api/v2?key=val#result".to_string()),
        "升级后应同时保留查询参数和片段"
    );

    // 升级后的 URL 不再是混合内容
    if let Some(upgraded_url) = upgraded_full {
        assert!(!is_mixed_content(&page, &upgraded_url));
    }
}

// ═══════════════════════════════════════════════════════════════════════
// Additional edge-case tests (round 19)
// ═══════════════════════════════════════════════════════════════════════

/// 测试 CORS 自定义请求头不在 Access-Control-Allow-Headers 中时被拒绝。
///
/// 当请求携带自定义头（如 X-Secret-Token），
/// 但服务端 CORS 策略的 allow_headers 仅包含 Authorization 时，
/// check_cors 应拒绝该请求，且原因应包含未允许的头名。
/// 同时验证 generate_preflight_response 在请求头不匹配时返回空响应。
#[test]
fn test_cors_custom_header_not_in_allow_headers() {
    let policy = CorsPolicy {
        allow_origins: vec!["http://example.com".to_string()],
        allow_methods: vec!["GET".to_string(), "POST".to_string()],
        allow_headers: vec!["Authorization".to_string()],
        allow_credentials: false,
        max_age: None,
    };
    let origin = Origin::parse("http://example.com").unwrap();

    // 场景 1：X-Secret-Token 不在 allow_headers 中 → check_cors 拒绝
    let result = check_cors(
        &policy,
        &origin,
        "GET",
        &[
            ("Authorization".to_string(), "Bearer token".to_string()),
            ("X-Secret-Token".to_string(), "abc".to_string()),
        ],
    );
    assert!(!result.allowed, "X-Secret-Token 未在 allow_headers 中应被拒绝");
    assert!(result.reason.contains("X-Secret-Token"), "拒绝原因应包含未允许的头名");

    // 场景 2：仅有 Authorization 在 allow_headers 中 → 通过
    let result_ok = check_cors(
        &policy,
        &origin,
        "GET",
        &[("Authorization".to_string(), "Bearer token".to_string())],
    );
    assert!(result_ok.allowed, "Authorization 在 allow_headers 中应通过");

    // 场景 3：generate_preflight_response 对未允许的头返回空响应
    let headers = generate_preflight_response(
        &policy,
        &origin,
        "GET",
        &["Authorization".to_string(), "X-Secret-Token".to_string()],
    );
    assert!(headers.allow_origin.is_none(), "预检请求中有未允许的头时应返回空响应");
}

/// 测试 CSP script-src 'none' 阻止 data: URI 脚本加载。
///
/// data: URI 可用于内联嵌入脚本内容（如 data:text/javascript,...）。
/// 当 script-src 为 'none' 时，应无条件阻止所有脚本来源，包括 data: URI。
/// 当前实现中 is_self_match 将非 http/https 开头的 URL 视为相对路径（同源），
/// 因此 script-src 'self' 不会阻止 data: URI——这是已知行为。
/// 此测试验证 'none' 策略能正确阻止 data: URI。
#[test]
fn test_csp_script_src_blocks_data_uri() {
    // script-src 'none' → 所有脚本均被阻止，包括 data: URI
    let csp_none = ContentSecurityPolicy::parse("script-src 'none'");
    assert!(
        !csp_none.is_resource_allowed("script", "data:text/javascript,alert(1)", None),
        "script-src 'none' 应阻止 data: URI"
    );
    // 内联脚本也被阻止
    assert!(
        !csp_none.is_inline_script_allowed(None, None),
        "script-src 'none' 应阻止内联脚本"
    );

    // script-src data: → data: 作为前缀匹配允许 data: URI
    let csp_data = ContentSecurityPolicy::parse("script-src data:");
    assert!(
        csp_data.is_resource_allowed("script", "data:text/javascript,alert(1)", None),
        "script-src data: 应允许 data: URI（前缀匹配）"
    );

    // default-src 'none' 回退也应阻止 data: URI
    let csp_default = ContentSecurityPolicy::parse("default-src 'none'");
    assert!(
        !csp_default.is_resource_allowed("script", "data:text/javascript,alert(1)", None),
        "default-src 'none' 应阻止 data: URI 脚本"
    );
    assert!(
        !csp_default.is_inline_script_allowed(None, None),
        "default-src 'none' 应阻止内联脚本"
    );

    // 当前实现行为记录：script-src 'self' 不会阻止 data: URI，
    // 因为 is_self_match 将非 http/https 开头的 URL 视为相对路径（同源）。
    // 根据 CSP 规范，data: URI 不应匹配 'self'，这是需要改进的地方。
    let csp_self = ContentSecurityPolicy::parse("script-src 'self'");
    let doc_origin = Origin::parse("https://example.com").unwrap();
    let data_allowed = csp_self.is_resource_allowed("script", "data:text/javascript,alert(1)", Some(&doc_origin));
    // 记录当前行为：data: URI 通过 'self' 检查（is_self_match 的相对路径逻辑）
    assert!(
        data_allowed,
        "当前实现：data: URI 被 is_self_match 视为相对路径，通过 'self' 检查"
    );
}

/// 测试同源策略对 http 与 https 不同协议的严格区分。
///
/// http://example.com 与 https://example.com 虽然主机名相同，
/// 但协议不同（http vs https），且默认端口也不同（80 vs 443），
/// 因此不是同源。验证 is_same_origin、check_same_origin 和
/// is_secure 三个维度的一致性。
#[test]
fn test_same_origin_http_vs_https_protocol() {
    let http = Origin::parse("http://example.com").unwrap();
    let https = Origin::parse("https://example.com").unwrap();

    // 协议不同
    assert_ne!(http.scheme, https.scheme, "http 与 https 的 scheme 应不同");
    assert_eq!(http.scheme, "http");
    assert_eq!(https.scheme, "https");

    // 默认端口不同：http=80, https=443
    assert_ne!(http.port, https.port, "默认端口应不同（80 vs 443）");
    assert_eq!(http.port, 80);
    assert_eq!(https.port, 443);

    // 主机名相同
    assert_eq!(http.host, https.host, "主机名应相同");

    // 不同源
    assert!(
        !http.is_same_origin(&https),
        "http://example.com 与 https://example.com 不是同源"
    );
    assert!(
        !check_same_origin(&http, &https),
        "check_same_origin 对 http vs https 应返回 false"
    );

    // 反向也成立
    assert!(!https.is_same_origin(&http), "反向同源检查也应返回 false");

    // 安全上下文判断
    assert!(!http.is_secure(), "http 不是安全上下文");
    assert!(https.is_secure(), "https 是安全上下文");

    // 显式端口版本也验证
    let http_80 = Origin::parse("http://example.com:80").unwrap();
    let https_443 = Origin::parse("https://example.com:443").unwrap();
    assert!(!http_80.is_same_origin(&https_443), "显式默认端口版本也不是同源");
    assert!(http.is_same_origin(&http_80), "http 默认端口应与隐式端口同源");
    assert!(https.is_same_origin(&https_443), "https 默认端口应与隐式端口同源");
}

/// 测试沙箱 allow-popups 标志允许弹窗但与其他标志组合时的行为。
///
/// allow-popups 允许 iframe 内通过 window.open 等方式打开弹窗。
/// 验证：
/// 1. 仅有 allow-popups 时允许弹窗但其他功能受限
/// 2. allow-popups + allow-scripts 组合允许脚本和弹窗
/// 3. check_sandbox_popup 函数与 allows_popups 方法一致
/// 4. 无 allow-popups 时弹窗被禁止
#[test]
fn test_sandbox_allow_popups_flag_behavior() {
    // 场景 1：仅有 allow-popups
    let sandbox = IframeSandbox::parse("allow-popups");
    assert!(sandbox.allows_popups(), "allow-popups 应允许弹窗");
    assert!(sandbox.has_flag(IframeSandboxFlag::AllowPopups));
    assert!(check_sandbox_popup(&sandbox), "check_sandbox_popup 应返回 true");
    // 其他功能受限
    assert!(!sandbox.allows_scripts(), "不应允许脚本");
    assert!(!sandbox.allows_forms(), "不应允许表单");
    assert!(!sandbox.allows_same_origin(), "不应允许同源访问");
    assert!(!sandbox.allows_top_navigation(), "不应允许顶层导航");

    // 场景 2：allow-popups + allow-scripts 组合
    let sandbox_combo = IframeSandbox::parse("allow-popups allow-scripts");
    assert!(sandbox_combo.allows_popups(), "组合中弹窗应允许");
    assert!(sandbox_combo.allows_scripts(), "组合中脚本应允许");
    assert!(!sandbox_combo.allows_forms(), "组合中表单仍被禁止");
    assert!(!sandbox_combo.allows_same_origin(), "组合中同源仍被禁止");

    // 场景 3：严格沙箱不允许弹窗
    let strict = IframeSandbox::strict();
    assert!(!strict.allows_popups(), "严格沙箱不允许弹窗");
    assert!(
        !check_sandbox_popup(&strict),
        "check_sandbox_popup 对严格沙箱应返回 false"
    );

    // 场景 4：弹窗权限不影响 effective_origin
    let iframe_origin = Origin::parse("https://example.com").unwrap();
    assert_eq!(
        sandbox.effective_origin(&iframe_origin),
        SandboxOrigin::Opaque,
        "allow-popups 不影响 effective_origin（缺少 allow-same-origin 仍为不透明源）"
    );
}

/// 测试混合内容检测对 ws: WebSocket URL 在 HTTPS 页面上的处理。
///
/// HTTPS 页面打开 ws://（非加密 WebSocket）连接属于混合内容，
/// 因为 ws:// 使用明文传输。当前 is_mixed_content 仅检测
/// starts_with("http://")，ws:// 前缀不同，因此不会被检测为混合内容。
/// 但 wss://（加密 WebSocket）在 HTTPS 页面上是安全的。
/// 此测试记录当前行为并验证升级函数对 ws:// 的处理。
#[test]
fn test_mixed_content_ws_url_on_https_page() {
    let page = Origin::parse("https://example.com").unwrap();

    // ws:// WebSocket URL 在 HTTPS 页面上
    let ws_url = "ws://api.example.com/socket";
    // 当前 is_mixed_content 仅检测 http:// 前缀，ws:// 不以 http:// 开头
    // 因此当前实现不将其识别为混合内容
    // 这记录了当前行为——ws:// 理论上应被视为混合内容（需要扩展检测逻辑）
    let is_detected = is_mixed_content(&page, ws_url);
    // ws:// 不以 http:// 开头，当前不会被检测为混合内容
    assert!(
        !is_detected,
        "当前 is_mixed_content 不检测 ws:// 前缀（仅检测 http://）"
    );

    // wss://（加密 WebSocket）不是混合内容
    let wss_url = "wss://api.example.com/socket";
    assert!(!is_mixed_content(&page, wss_url), "wss:// 在 HTTPS 页面上不是混合内容");

    // upgrade_to_https 不处理 ws://（仅处理 http:// 前缀）
    assert_eq!(upgrade_to_https(ws_url), None, "upgrade_to_https 不应处理 ws:// URL");

    // 对比：http:// 在 HTTPS 页面上应被检测为混合内容
    let http_url = "http://api.example.com/data";
    assert!(
        is_mixed_content(&page, http_url),
        "http:// 在 HTTPS 页面上应被检测为混合内容"
    );

    // check_mixed_content 对 ws:// 返回 NotMixedContent（当前行为）
    assert_eq!(
        check_mixed_content(&page, ws_url, "connect"),
        MixedContentStatus::NotMixedContent,
        "当前实现对 ws:// 不检测为混合内容"
    );
}

// ═══════════════════════════════════════════════════════════════════════
// Additional edge-case tests (round 21)
// ═══════════════════════════════════════════════════════════════════════

/// 测试 CSP 空指令列表策略不阻止任何资源加载。
///
/// ContentSecurityPolicy::parse 接收空字符串时，directives 列表应为空。
/// 空策略等同于未设置 CSP，所有资源类型（script、style、img、connect 等）
/// 的加载均应被允许，内联脚本和内联样式也不受限制。
#[test]
fn test_csp_empty_directive_list_allows_all() {
    let csp = ContentSecurityPolicy::parse("");
    assert!(csp.directives.is_empty(), "空字符串解析后 directives 应为空");

    // 所有资源类型均应允许
    assert!(
        csp.is_resource_allowed("script", "https://evil.com/bad.js", None),
        "空策略不应阻止脚本加载"
    );
    assert!(
        csp.is_resource_allowed("style", "https://evil.com/bad.css", None),
        "空策略不应阻止样式加载"
    );
    assert!(
        csp.is_resource_allowed("img", "https://evil.com/bad.png", None),
        "空策略不应阻止图片加载"
    );
    assert!(
        csp.is_resource_allowed("connect", "https://api.example.com/data", None),
        "空策略不应阻止 XHR/Fetch 请求"
    );
    assert!(
        csp.is_resource_allowed("font", "https://cdn.example.com/font.woff2", None),
        "空策略不应阻止字体加载"
    );
    assert!(
        csp.is_resource_allowed("frame", "https://evil.com/embed", None),
        "空策略不应阻止 iframe 加载"
    );

    // 内联脚本和样式也不受限制
    assert!(csp.is_inline_script_allowed(None, None), "空策略不应阻止内联脚本");
    assert!(csp.is_inline_style_allowed(None, None), "空策略不应阻止内联样式");
}

/// 测试 CORS 通配符源（*）配合凭证（credentials）时请求被拒绝。
///
/// 根据 CORS 规范，Access-Control-Allow-Origin: * 与
/// Access-Control-Allow-Credentials: true 不能同时使用。
/// 当 allow_origins 包含 "*" 且 allow_credentials 为 true 时，
/// check_cors 应拒绝请求，generate_preflight_response 也应返回空响应。
#[test]
fn test_cors_wildcard_origin_with_credentials_rejected() {
    let policy = CorsPolicy {
        allow_origins: vec!["*".to_string()],
        allow_methods: vec!["GET".to_string(), "POST".to_string()],
        allow_headers: vec![],
        allow_credentials: true,
        max_age: None,
    };
    let origin = Origin::parse("http://example.com").unwrap();

    // check_cors 应拒绝（通配符 + 凭证不允许）
    let result = check_cors(&policy, &origin, "GET", &[]);
    assert!(!result.allowed, "通配符源 + credentials 应被 check_cors 拒绝");
    assert!(
        result.reason.contains("credential") || result.reason.contains("origin"),
        "拒绝原因应提及 credential 或 origin"
    );

    // generate_preflight_response 也应拒绝
    let headers = generate_preflight_response(&policy, &origin, "POST", &[]);
    assert!(
        headers.allow_origin.is_none(),
        "通配符源 + credentials 的预检响应不应包含 allow_origin"
    );
    assert!(
        headers.allow_credentials.is_none(),
        "通配符源 + credentials 时不应返回 allow_credentials"
    );

    // 对比：通配符源 + 无凭证 → 允许
    let policy_no_cred = CorsPolicy {
        allow_origins: vec!["*".to_string()],
        allow_methods: vec!["GET".to_string()],
        allow_headers: vec![],
        allow_credentials: false,
        max_age: None,
    };
    let result_ok = check_cors(&policy_no_cred, &origin, "GET", &[]);
    assert!(result_ok.allowed, "通配符源 + 无凭证应被允许");
}

/// 测试混合内容检测对 data: URI 的处理：不以 http:// 开头故不检测为混合内容。
///
/// data: URI 是不透明来源（opaque origin），不使用 http:// 或 https:// 协议。
/// 当前 is_mixed_content 仅检测 starts_with("http://")，
/// 因此 data: URI 不会被识别为混合内容。upgrade_to_https 对
/// 非 http:// 开头的 URL 也返回 None。
#[test]
fn test_mixed_content_data_uri_not_detected() {
    let page = Origin::parse("https://example.com").unwrap();

    // data: URI 不以 http:// 开头 → 不被检测为混合内容
    let data_uri = "data:text/html,<script>alert(1)</script>";
    assert!(
        !is_mixed_content(&page, data_uri),
        "data: URI 不以 http:// 开头，不应被检测为混合内容"
    );

    // check_mixed_content 返回 NotMixedContent
    assert_eq!(
        check_mixed_content(&page, data_uri, "script"),
        MixedContentStatus::NotMixedContent,
        "data: URI 的混合内容状态应为 NotMixedContent"
    );

    // upgrade_to_https 对 data: URI 返回 None
    assert_eq!(upgrade_to_https(data_uri), None, "upgrade_to_https 不应处理 data: URI");

    // data: image URI
    let data_img = "data:image/png;base64,iVBORw0KGgo=";
    assert!(
        !is_mixed_content(&page, data_img),
        "data: 图片 URI 也不应被检测为混合内容"
    );
    assert_eq!(
        check_mixed_content(&page, data_img, "img"),
        MixedContentStatus::NotMixedContent
    );
    assert_eq!(upgrade_to_https(data_img), None);
}

/// 测试沙箱空标志（等同于严格沙箱）的所有功能均被禁止。
///
/// IframeSandbox::strict() 创建的沙箱 flags 列表为空，
/// 所有核心功能（脚本、同源、表单、弹窗、顶层导航）均被禁止。
/// effective_origin 为不透明源（Opaque），
/// IframeSandbox::parse("") 也应产生相同的严格行为。
#[test]
fn test_sandbox_empty_flags_is_strict() {
    let strict = IframeSandbox::strict();
    let from_empty = IframeSandbox::parse("");

    // 所有核心功能均被禁止
    assert!(!strict.allows_scripts(), "严格沙箱不应允许脚本");
    assert!(!strict.allows_same_origin(), "严格沙箱不应允许同源");
    assert!(!strict.allows_forms(), "严格沙箱不应允许表单");
    assert!(!strict.allows_popups(), "严格沙箱不应允许弹窗");
    assert!(!strict.allows_top_navigation(), "严格沙箱不应允许顶层导航");

    // parse("") 与 strict() 行为一致
    assert!(!from_empty.allows_scripts(), "parse(\"\") 不应允许脚本");
    assert!(!from_empty.allows_same_origin(), "parse(\"\") 不应允许同源");
    assert!(!from_empty.allows_forms(), "parse(\"\") 不应允许表单");
    assert!(!from_empty.allows_popups(), "parse(\"\") 不应允许弹窗");
    assert!(!from_empty.allows_top_navigation(), "parse(\"\") 不应允许顶层导航");

    // effective_origin 为不透明源
    let origin = Origin::parse("https://example.com").unwrap();
    assert_eq!(
        strict.effective_origin(&origin),
        SandboxOrigin::Opaque,
        "严格沙箱的 effective_origin 应为 Opaque"
    );
    assert_eq!(
        from_empty.effective_origin(&origin),
        SandboxOrigin::Opaque,
        "parse(\"\") 的 effective_origin 应为 Opaque"
    );

    // 无任何标志
    assert!(!strict.has_flag(IframeSandboxFlag::AllowScripts));
    assert!(!strict.has_flag(IframeSandboxFlag::AllowSameOrigin));
    assert!(!strict.has_flag(IframeSandboxFlag::AllowForms));
    assert!(!strict.has_flag(IframeSandboxFlag::AllowPopups));
    assert!(!strict.has_flag(IframeSandboxFlag::AllowTopNavigation));
}

/// 测试同源策略：完全相同的源（scheme+host+port 一致）返回 true。
///
/// 验证 is_same_origin 和 check_same_origin 在以下场景均返回 true：
/// 1. 相同完整 URL 字符串解析出的两个 Origin
/// 2. 不同路径（/a vs /b）但相同三元组的 Origin
/// 3. 显式默认端口与隐式默认端口
/// 4. http 和 https 各自的同源判断
#[test]
fn test_same_origin_identical_origins_returns_true() {
    // 场景 1：相同完整 URL
    let a = Origin::parse("https://example.com/page").unwrap();
    let b = Origin::parse("https://example.com/page").unwrap();
    assert!(a.is_same_origin(&b), "相同 URL 解析出的 Origin 应为同源");
    assert!(check_same_origin(&a, &b), "check_same_origin 对相同 Origin 应返回 true");
    assert_eq!(a, b, "相同 URL 的 Origin 应相等");

    // 场景 2：不同路径但相同三元组
    let c = Origin::parse("https://example.com/a").unwrap();
    let d = Origin::parse("https://example.com/b").unwrap();
    assert!(c.is_same_origin(&d), "不同路径但相同 scheme+host+port 应为同源");
    assert!(check_same_origin(&c, &d));
    assert_eq!(c, d, "三元组相同的 Origin 应相等");

    // 场景 3：显式默认端口与隐式默认端口
    let implicit = Origin::parse("https://example.com").unwrap();
    let explicit = Origin::parse("https://example.com:443").unwrap();
    assert!(implicit.is_same_origin(&explicit), "隐式 443 与显式 443 应为同源");
    assert!(check_same_origin(&implicit, &explicit));
    assert_eq!(implicit, explicit);

    // 场景 4：http 同源判断
    let http_a = Origin::parse("http://example.com").unwrap();
    let http_b = Origin::parse("http://example.com:80/path").unwrap();
    assert!(http_a.is_same_origin(&http_b), "http 默认端口 80 应为同源");
    assert!(check_same_origin(&http_a, &http_b));
    assert_eq!(http_a, http_b);

    // 验证三元组字段值
    assert_eq!(implicit.scheme, "https");
    assert_eq!(implicit.host, "example.com");
    assert_eq!(implicit.port, 443);
}

// ── 新增边界测试 ──

/// 测试 CSP report-only 不阻止实际请求。
#[test]
fn test_csp_report_only_allows_all() {
    let policy = csp::ContentSecurityPolicy::parse("Content-Security-Policy-Report-Only: default-src 'self'");
    // report-only 模式下应允许所有资源
    assert!(policy.is_resource_allowed("script", "http://evil.com", None));
    assert!(policy.is_resource_allowed("style", "http://evil.com", None));
}

/// 测试不同协议的源判定。
#[test]
fn test_origin_different_scheme_not_same() {
    let http = Origin::parse("http://example.com").unwrap();
    let https = Origin::parse("https://example.com").unwrap();
    assert!(!http.is_same_origin(&https), "http 和 https 应为不同源");
}

/// 测试 CORS 简单请求不包含自定义 header。
#[test]
fn test_cors_simple_request_no_custom_header() {
    assert!(cors::is_simple_request("GET", Some("text/plain"), &[]));
    assert!(cors::is_simple_request(
        "POST",
        Some("application/x-www-form-urlencoded"),
        &[]
    ));
}

/// 测试混合内容阻止 http: 协议但不阻止 https:。
#[test]
fn test_mixed_content_blocks_http_on_https() {
    let origin = Origin::parse("https://example.com").unwrap();
    assert!(mixed_content::is_mixed_content(&origin, "http://example.com/script.js"));
    assert!(!mixed_content::is_mixed_content(
        &origin,
        "https://example.com/script.js"
    ));
    // 非安全源（http）不触发混合内容检查
    let http_origin = Origin::parse("http://example.com").unwrap();
    assert!(!mixed_content::is_mixed_content(
        &http_origin,
        "http://other.com/script.js"
    ));
}

/// 测试 sandbox 允许脚本运行。
#[test]
fn test_sandbox_allows_scripts() {
    let sandbox = sandbox::IframeSandbox::parse("allow-scripts");
    assert!(sandbox.allows_scripts());
    assert!(!sandbox.allows_same_origin());
}
