//! CSP（内容安全策略）单元测试。

use crate::csp::*;

// ---- 解析测试 ----

#[test]
fn test_csp_parse_default_src() {
    let csp = ContentSecurityPolicy::parse("default-src 'self'");
    assert_eq!(csp.directives.len(), 1);
    assert_eq!(csp.directives[0].name, "default-src");
    assert_eq!(csp.directives[0].values, vec!["'self'"]);
}

#[test]
fn test_csp_parse_multiple_directives() {
    let csp = ContentSecurityPolicy::parse("default-src 'self'; script-src 'unsafe-inline'");
    assert_eq!(csp.directives.len(), 2);
    assert_eq!(csp.directives[0].name, "default-src");
    assert_eq!(csp.directives[1].name, "script-src");
}

#[test]
fn test_csp_parse_empty_values() {
    let csp = ContentSecurityPolicy::parse("upgrade-insecure-requests");
    assert_eq!(csp.directives.len(), 1);
    assert_eq!(csp.directives[0].name, "upgrade-insecure-requests");
    assert!(csp.directives[0].values.is_empty());
}

#[test]
fn test_csp_parse_empty_input() {
    let csp = ContentSecurityPolicy::parse("");
    assert!(csp.directives.is_empty());
}

// ---- 源列表匹配测试 ----

#[test]
fn test_is_resource_allowed_no_directive() {
    let csp = ContentSecurityPolicy::parse("img-src https://example.com");
    assert!(csp.is_resource_allowed("script", "https://evil.com/evil.js", None));
}

#[test]
fn test_is_resource_allowed_default_src() {
    let csp = ContentSecurityPolicy::parse("default-src https://example.com");
    assert!(csp.is_resource_allowed("script", "https://example.com/script.js", None));
    assert!(!csp.is_resource_allowed("script", "https://evil.com/evil.js", None));
}

#[test]
fn test_is_resource_allowed_self() {
    let csp = ContentSecurityPolicy::parse("script-src 'self'");
    let origin = crate::origin::Origin::parse("https://example.com").unwrap();
    assert!(csp.is_resource_allowed("script", "https://example.com/script.js", Some(&origin)));
}

#[test]
fn test_is_resource_allowed_none() {
    let csp = ContentSecurityPolicy::parse("script-src 'none'");
    assert!(!csp.is_resource_allowed("script", "https://example.com/script.js", None));
}

#[test]
fn test_is_resource_allowed_wildcard() {
    let csp = ContentSecurityPolicy::parse("script-src *");
    assert!(csp.is_resource_allowed("script", "https://example.com/script.js", None));
}

#[test]
fn test_is_resource_allowed_wildcard_domain() {
    let csp = ContentSecurityPolicy::parse("script-src *.example.com");
    assert!(csp.is_resource_allowed("script", "https://sub.example.com/script.js", None));
    assert!(!csp.is_resource_allowed("script", "https://notexample.com/script.js", None));
}

/// R3342：CSP 源表达式须按主机名精确/通配符匹配，禁止纯字符串前缀匹配。
///
/// 修复前 `check_source_list` 用 `url.starts_with(value)` 做前缀匹配——攻击者注册
/// `example.com.evil.com` 域名，`script-src https://example.com` 策略下其脚本
/// `https://example.com.evil.com/x.js` 因 `starts_with("https://example.com")` 为 true
/// 被错误允许（CSP 绕过，可加载任意跨源脚本）。CSP 规范要求源表达式按 host 匹配，
/// `https://example.com` 只匹配 host 恰为 example.com（或显式 `*.example.com` 子域）。
#[test]
fn test_is_resource_allowed_no_prefix_bypass_r3342() {
    // 策略仅允许 https://example.com。
    let csp = ContentSecurityPolicy::parse("script-src https://example.com");

    // 合法：host 恰为 example.com。
    assert!(csp.is_resource_allowed("script", "https://example.com/script.js", None));
    // 绕过尝试 1：攻击者域 example.com.evil.com（修复前 starts_with 误匹配）。
    assert!(
        !csp.is_resource_allowed("script", "https://example.com.evil.com/x.js", None),
        "CSP 源表达式不得前缀匹配，example.com.evil.com 不应被 example.com 策略允许"
    );
    // 绕过尝试 2：攻击者域 evilexample.com（前缀含 example.com 但 host 不同）。
    assert!(
        !csp.is_resource_allowed("script", "https://evilexample.com/x.js", None),
        "evilexample.com 不应被 example.com 策略允许"
    );
    // 合法子路径仍允许（host 不变）。
    assert!(csp.is_resource_allowed("script", "https://example.com/a/b/c.js", None));

    // 裸主机名源表达式（无 scheme）：`script-src example.com`。
    let csp2 = ContentSecurityPolicy::parse("script-src example.com");
    assert!(csp2.is_resource_allowed("script", "https://example.com/s.js", None));
    assert!(csp2.is_resource_allowed("script", "http://example.com/s.js", None));
    assert!(
        !csp2.is_resource_allowed("script", "https://example.com.evil.com/s.js", None),
        "裸主机名源表达式同样不得前缀匹配"
    );
}

// ---- 内联脚本/样式测试 ----

#[test]
fn test_inline_script_unsafe_inline() {
    let csp = ContentSecurityPolicy::parse("script-src 'unsafe-inline'");
    assert!(csp.is_inline_script_allowed(None, None));
}

#[test]
fn test_inline_script_nonce() {
    let csp = ContentSecurityPolicy::parse("script-src 'nonce-abc123'");
    assert!(csp.is_inline_script_allowed(Some("abc123"), None));
    assert!(!csp.is_inline_script_allowed(Some("wrong"), None));
}

#[test]
fn test_inline_script_hash() {
    let csp = ContentSecurityPolicy::parse("script-src 'sha256-base64hash'");
    assert!(csp.is_inline_script_allowed(None, Some("base64hash")));
}

#[test]
fn test_inline_style_unsafe_inline() {
    let csp = ContentSecurityPolicy::parse("style-src 'unsafe-inline'");
    assert!(csp.is_inline_style_allowed(None, None));
}

#[test]
fn test_inline_style_nonce() {
    let csp = ContentSecurityPolicy::parse("style-src 'nonce-abc123'");
    assert!(csp.is_inline_style_allowed(Some("abc123"), None));
}

// ---- 导航/文档指令测试 ----

#[test]
fn test_base_uri_allowed() {
    let csp = ContentSecurityPolicy::parse("base-uri 'self'");
    let origin = crate::origin::Origin::parse("https://example.com").unwrap();
    assert!(csp.is_base_uri_allowed("https://example.com/base", Some(&origin)));
}

#[test]
fn test_base_uri_no_directive() {
    let csp = ContentSecurityPolicy::parse("default-src 'none'");
    assert!(csp.is_base_uri_allowed("https://example.com/base", None));
}

#[test]
fn test_form_action_allowed() {
    let csp = ContentSecurityPolicy::parse("form-action 'self'");
    let origin = crate::origin::Origin::parse("https://example.com").unwrap();
    assert!(csp.is_form_action_allowed("https://example.com/submit", Some(&origin)));
}

#[test]
fn test_frame_ancestors_none() {
    let csp = ContentSecurityPolicy::parse("frame-ancestors 'none'");
    let embedder = crate::origin::Origin::parse("https://evil.com").unwrap();
    assert!(!csp.is_frame_ancestor_allowed(&embedder));
}

#[test]
fn test_frame_ancestors_self() {
    let csp = ContentSecurityPolicy::parse("frame-ancestors 'self'");
    let embedder = crate::origin::Origin::parse("https://example.com").unwrap();
    assert!(csp.is_frame_ancestor_allowed(&embedder));
}

#[test]
fn test_sandbox_flags() {
    let csp = ContentSecurityPolicy::parse("sandbox allow-scripts allow-forms");
    let flags = csp.sandbox_flags().unwrap();
    assert_eq!(flags.len(), 2);
}

#[test]
fn test_sandbox_empty() {
    let csp = ContentSecurityPolicy::parse("sandbox");
    let flags = csp.sandbox_flags().unwrap();
    assert!(flags.is_empty());
}

// R3389：CSP directive name / sandbox token 须 ASCII 大小写不敏感匹配，空源列表等价 'none'。
// 三处 CSP spec 合规修复回归锁定。

#[test]
fn test_sandbox_token_case_insensitive_r3389() {
    // 全大写 token 须识别（CSP §6.3.2 sandbox token 大小写不敏感，与 HTML iframe 同根）
    let csp = ContentSecurityPolicy::parse("sandbox ALLOW-SCRIPTS ALLOW-FORMS");
    let flags = csp.sandbox_flags().unwrap();
    assert_eq!(flags.len(), 2, "全大写 sandbox token 须识别");
    // 首字母大写（HTML/CSP 常见书写）
    let csp = ContentSecurityPolicy::parse("sandbox Allow-Same-Origin Allow-Top-Navigation");
    let flags = csp.sandbox_flags().unwrap();
    assert_eq!(flags.len(), 2, "首字母大写 sandbox token 须识别");
    // 小写回归保护
    let csp = ContentSecurityPolicy::parse("sandbox allow-scripts allow-forms");
    assert_eq!(csp.sandbox_flags().unwrap().len(), 2);
}

#[test]
fn test_directive_name_case_insensitive_r3389() {
    // mixed-case 指令名须按大小写不敏感匹配（CSP §2.2.1 parse step 4 小写化 directive name）。
    // 旧实现：`Script-Src 'none'` 被当未知指令丢弃 → 回退 default-src（缺省）→ 放行 = CSP 绕过。
    let csp = ContentSecurityPolicy::parse("Script-Src 'none'");
    assert!(
        !csp.is_resource_allowed("script", "https://evil.com/evil.js", None),
        "mixed-case 指令名须识别，'none' 应阻断脚本"
    );
    // 首字母大写 + 小写混合（用 host 而非 'self'，避免依赖 document_origin）
    let csp = ContentSecurityPolicy::parse("DEFAULT-SRC https://example.com");
    assert!(csp.is_resource_allowed("script", "https://example.com/s.js", None));
    assert!(!csp.is_resource_allowed("script", "https://evil.com/s.js", None));
    // 解析后指令名须规范化为小写
    let csp = ContentSecurityPolicy::parse("Script-Src 'none'");
    assert_eq!(csp.directives[0].name, "script-src");
}

#[test]
fn test_empty_source_list_blocks_all_r3389() {
    // 空源列表等价只含 'none'，须阻断全部（CSP §6.7.2.7）。
    // 旧实现：`script-src`（无值）空列表返回 true（放行全部）= CSP 绕过。
    let csp = ContentSecurityPolicy::parse("script-src");
    assert!(
        !csp.is_resource_allowed("script", "https://example.com/s.js", None),
        "空源列表应阻断全部资源"
    );
    assert!(
        !csp.is_resource_allowed("script", "https://evil.com/s.js", None),
        "空源列表应阻断全部资源"
    );
}

#[test]
fn test_none_ignored_when_other_source_present_r3389() {
    // 'none' 与其它源共存时须被忽略（CSP §6.7.2.7 注：'none' ignored if other source present）。
    // 旧实现：`script-src 'none' 'self'` 因 'none' 在场而阻断 self（过度阻断，非 spec 语义）。
    let origin = crate::origin::Origin::parse("https://example.com").unwrap();
    let csp = ContentSecurityPolicy::parse("script-src 'none' 'self'");
    assert!(
        csp.is_resource_allowed("script", "https://example.com/s.js", Some(&origin)),
        "'none' 与 'self' 共存时应忽略 'none'，放行 self"
    );
    assert!(
        !csp.is_resource_allowed("script", "https://evil.com/s.js", Some(&origin)),
        "非 self 仍应阻断"
    );
    // 'none' 独占时仍阻断全部（回归保护）
    let csp = ContentSecurityPolicy::parse("script-src 'none'");
    assert!(!csp.is_resource_allowed("script", "https://example.com/s.js", None));
}

#[test]
fn test_navigate_to_allowed() {
    let csp = ContentSecurityPolicy::parse("navigate-to 'self'");
    let origin = crate::origin::Origin::parse("https://example.com").unwrap();
    assert!(csp.is_navigate_to_allowed("https://example.com/page", Some(&origin)));
}

// ---- 资源类型便捷方法测试 ----

#[test]
fn test_connect_src() {
    let csp = ContentSecurityPolicy::parse("connect-src https://api.example.com");
    assert!(csp.is_connect_allowed("https://api.example.com/data", None));
    assert!(!csp.is_connect_allowed("https://evil.com/api", None));
}

#[test]
fn test_font_src() {
    let csp = ContentSecurityPolicy::parse("font-src https://fonts.example.com");
    assert!(csp.is_font_allowed("https://fonts.example.com/font.woff", None));
}

#[test]
fn test_media_src() {
    let csp = ContentSecurityPolicy::parse("media-src https://media.example.com");
    assert!(csp.is_media_allowed("https://media.example.com/video.mp4", None));
}

#[test]
fn test_object_src() {
    let csp = ContentSecurityPolicy::parse("object-src 'none'");
    assert!(!csp.is_object_allowed("https://example.com/flash.swf", None));
}

#[test]
fn test_frame_src_fallback() {
    let csp = ContentSecurityPolicy::parse("child-src https://child.example.com");
    assert!(csp.is_frame_allowed("https://child.example.com/frame.html", None));
}

#[test]
fn test_worker_src_fallback() {
    let csp = ContentSecurityPolicy::parse("script-src https://scripts.example.com");
    assert!(csp.is_worker_allowed("https://scripts.example.com/worker.js", None));
}

#[test]
fn test_manifest_src() {
    let csp = ContentSecurityPolicy::parse("manifest-src https://app.example.com");
    assert!(csp.is_manifest_allowed("https://app.example.com/manifest.json", None));
}

#[test]
fn test_is_script_element_allowed() {
    let csp = ContentSecurityPolicy::parse("script-src https://scripts.example.com");
    assert!(csp.is_script_element_allowed("https://scripts.example.com/app.js", None));
}

#[test]
fn test_is_script_element_allowed_fallback() {
    let csp = ContentSecurityPolicy::parse("default-src https://example.com");
    assert!(csp.is_script_element_allowed("https://example.com/script.js", None));
}

#[test]
fn test_is_style_element_allowed() {
    let csp = ContentSecurityPolicy::parse("style-src https://styles.example.com");
    assert!(csp.is_style_element_allowed("https://styles.example.com/style.css", None));
}

#[test]
fn test_is_style_element_allowed_fallback() {
    let csp = ContentSecurityPolicy::parse("default-src https://example.com");
    assert!(csp.is_style_element_allowed("https://example.com/style.css", None));
}

// ---- upgrade-insecure-requests / report 测试 ----

#[test]
fn test_upgrade_insecure_requests() {
    let csp = ContentSecurityPolicy::parse("upgrade-insecure-requests");
    assert!(csp.has_upgrade_insecure_requests());
}

#[test]
fn test_report_uri() {
    let csp = ContentSecurityPolicy::parse("report-uri /csp-report");
    assert_eq!(csp.report_uri(), Some("/csp-report"));
}

#[test]
fn test_report_to() {
    let csp = ContentSecurityPolicy::parse("report-to csp-endpoint");
    assert_eq!(csp.report_to(), Some("csp-endpoint"));
}

// ---- script-src-attr / style-src-attr 测试 ----

#[test]
fn test_script_src_attr_allows_unsafe_inline() {
    let csp = ContentSecurityPolicy::parse("script-src 'unsafe-inline'");
    assert!(csp.is_script_attr_allowed(None, None));
}

#[test]
fn test_script_src_attr_blocks_no_directive() {
    let csp = ContentSecurityPolicy::parse("default-src 'none'");
    assert!(!csp.is_script_attr_allowed(None, None));
}

#[test]
fn test_script_src_attr_allows_nonce() {
    let csp = ContentSecurityPolicy::parse("script-src 'nonce-abc123'");
    assert!(csp.is_script_attr_allowed(Some("abc123"), None));
}

#[test]
fn test_script_src_attr_blocks_wrong_nonce() {
    let csp = ContentSecurityPolicy::parse("script-src 'nonce-abc123'");
    assert!(!csp.is_script_attr_allowed(Some("wrong"), None));
}

#[test]
fn test_script_src_attr_unsafe_hashes_allows_hash() {
    let csp = ContentSecurityPolicy::parse("script-src 'sha256-base64hash'");
    assert!(!csp.is_script_attr_allowed(None, Some("base64hash")));

    let csp2 = ContentSecurityPolicy::parse("script-src 'unsafe-hashes' 'sha256-base64hash'");
    assert!(csp2.is_script_attr_allowed(None, Some("base64hash")));
}

#[test]
fn test_script_src_attr_fallback_to_script_src() {
    let csp = ContentSecurityPolicy::parse("script-src 'unsafe-inline'");
    assert!(csp.is_script_attr_allowed(None, None));
}

#[test]
fn test_script_src_attr_explicit_directive() {
    let csp = ContentSecurityPolicy::parse("script-src 'unsafe-inline'; script-src-attr 'none'");
    assert!(!csp.is_script_attr_allowed(None, None));
}

#[test]
fn test_style_src_attr_allows_unsafe_inline() {
    let csp = ContentSecurityPolicy::parse("style-src 'unsafe-inline'");
    assert!(csp.is_style_attr_allowed(None, None));
}

#[test]
fn test_style_src_attr_blocks_default_none() {
    let csp = ContentSecurityPolicy::parse("default-src 'none'");
    assert!(!csp.is_style_attr_allowed(None, None));
}

#[test]
fn test_style_src_attr_allows_nonce() {
    let csp = ContentSecurityPolicy::parse("style-src 'nonce-xyz789'");
    assert!(csp.is_style_attr_allowed(Some("xyz789"), None));
}

#[test]
fn test_style_src_attr_unsafe_hashes() {
    let csp = ContentSecurityPolicy::parse("style-src 'unsafe-hashes' 'sha256-abc123'");
    assert!(csp.is_style_attr_allowed(None, Some("abc123")));
}

// ---- unsafe-eval / wasm-unsafe-eval 测试 ----

#[test]
fn test_eval_blocked_with_self() {
    let csp = ContentSecurityPolicy::parse("default-src 'self'");
    assert!(!csp.is_eval_allowed());
}

#[test]
fn test_eval_allowed_with_unsafe_eval() {
    let csp = ContentSecurityPolicy::parse("script-src 'unsafe-eval'");
    assert!(csp.is_eval_allowed());
}

#[test]
fn test_eval_blocked_without_unsafe_eval() {
    let csp = ContentSecurityPolicy::parse("script-src 'self'");
    assert!(!csp.is_eval_allowed());
}

#[test]
fn test_eval_allowed_with_wildcard() {
    let csp = ContentSecurityPolicy::parse("script-src *");
    assert!(csp.is_eval_allowed());
}

#[test]
fn test_wasm_eval_allowed_by_unsafe_eval() {
    let csp = ContentSecurityPolicy::parse("script-src 'unsafe-eval'");
    assert!(csp.is_wasm_eval_allowed());
}

#[test]
fn test_wasm_eval_allowed_by_wasm_unsafe_eval() {
    let csp = ContentSecurityPolicy::parse("script-src 'wasm-unsafe-eval'");
    assert!(csp.is_wasm_eval_allowed());
    assert!(!csp.is_eval_allowed());
}

#[test]
fn test_wasm_eval_blocked_by_default() {
    let csp = ContentSecurityPolicy::parse("script-src 'self'");
    assert!(!csp.is_wasm_eval_allowed());
}

#[test]
fn test_wasm_eval_allowed_with_wildcard() {
    let csp = ContentSecurityPolicy::parse("script-src *");
    assert!(csp.is_wasm_eval_allowed());
}

#[test]
fn test_wasm_eval_fallback_to_default_src() {
    let csp = ContentSecurityPolicy::parse("default-src 'unsafe-eval'");
    assert!(csp.is_wasm_eval_allowed());
}

// ---- strict-dynamic 测试 ----

#[test]
fn test_strict_dynamic_not_set_by_default() {
    let csp = ContentSecurityPolicy::parse("script-src 'self'");
    assert!(!csp.has_strict_dynamic());
}

#[test]
fn test_strict_dynamic_detected() {
    let csp = ContentSecurityPolicy::parse("script-src 'strict-dynamic' 'nonce-abc'");
    assert!(csp.has_strict_dynamic());
}

#[test]
fn test_strict_dynamic_in_default_src() {
    let csp = ContentSecurityPolicy::parse("default-src 'strict-dynamic'");
    assert!(csp.has_strict_dynamic());
}

#[test]
fn test_strict_dynamic_not_in_other_directives() {
    let csp = ContentSecurityPolicy::parse("style-src 'strict-dynamic'");
    assert!(!csp.has_strict_dynamic());
}

#[test]
fn test_strict_dynamic_with_nonce_trust_chain() {
    let csp = ContentSecurityPolicy::parse("script-src 'strict-dynamic' 'nonce-abc123'");
    assert!(csp.has_strict_dynamic());
    assert!(csp.is_inline_script_allowed(Some("abc123"), None));
}

#[test]
fn test_strict_dynamic_url_matching() {
    let csp = ContentSecurityPolicy::parse("script-src 'strict-dynamic' https://trusted.com");
    assert!(csp.is_resource_allowed("script", "https://trusted.com/script.js", None));
}

// ---- report-sample 测试 ----

#[test]
fn test_report_sample_not_set_by_default() {
    let csp = ContentSecurityPolicy::parse("script-src 'self'");
    assert!(!csp.has_report_sample());
}

#[test]
fn test_report_sample_in_script_src() {
    let csp = ContentSecurityPolicy::parse("script-src 'report-sample' 'self'");
    assert!(csp.has_report_sample());
}

#[test]
fn test_report_sample_in_style_src() {
    let csp = ContentSecurityPolicy::parse("style-src 'report-sample'");
    assert!(csp.has_report_sample());
}

#[test]
fn test_report_sample_in_default_src() {
    let csp = ContentSecurityPolicy::parse("default-src 'report-sample'");
    assert!(csp.has_report_sample());
}

// ---- report-only 模式正确区分测试 ----

#[test]
fn test_report_only_never_blocks_resource() {
    let csp_only = ContentSecurityPolicyReportOnly::parse("script-src 'none'");
    assert!(csp_only.check_resource("script", "https://evil.com/evil.js", None, None));
}

#[test]
fn test_report_only_calls_callback_on_violation() {
    use std::sync::{Arc, Mutex};
    let violations = Arc::new(Mutex::new(Vec::new()));
    let violations_clone = violations.clone();
    let callback = move |url: &str, directive: &str, blocked: &str| {
        violations_clone
            .lock()
            .unwrap()
            .push((url.to_string(), directive.to_string(), blocked.to_string()));
    };
    let csp_only = ContentSecurityPolicyReportOnly::parse("script-src 'none'");
    let allowed = csp_only.check_resource("script", "https://evil.com/evil.js", None, Some(&callback));
    assert!(allowed);
    let v = violations.lock().unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].0, "https://evil.com/evil.js");
    assert_eq!(v[0].1, "script-src");
}

#[test]
fn test_report_only_no_callback_when_allowed() {
    use std::sync::{Arc, Mutex};
    let violations = Arc::new(Mutex::new(Vec::new()));
    let violations_clone = violations.clone();
    let callback = move |url: &str, directive: &str, blocked: &str| {
        violations_clone
            .lock()
            .unwrap()
            .push((url.to_string(), directive.to_string(), blocked.to_string()));
    };
    let csp_only = ContentSecurityPolicyReportOnly::parse("script-src 'self'");
    let allowed = csp_only.check_resource("script", "/local.js", None, Some(&callback));
    assert!(allowed);
    assert!(violations.lock().unwrap().is_empty());
}

// ---- CSP 多策略组合和 scheme-source 测试 ----

#[test]
fn test_multiple_csp_policies_most_restrictive_wins() {
    let csp1 = ContentSecurityPolicy::parse("script-src https://a.com");
    let csp2 = ContentSecurityPolicy::parse("script-src https://b.com");
    assert!(csp1.is_resource_allowed("script", "https://a.com/lib.js", None));
    assert!(!csp2.is_resource_allowed("script", "https://a.com/lib.js", None));
    assert!(!csp1.is_resource_allowed("script", "https://b.com/lib.js", None));
    assert!(csp2.is_resource_allowed("script", "https://b.com/lib.js", None));
}

#[test]
fn test_scheme_source_https() {
    let csp = ContentSecurityPolicy::parse("script-src https:");
    assert!(csp.is_resource_allowed("script", "https://any.com/script.js", None));
    assert!(!csp.is_resource_allowed("script", "http://any.com/script.js", None));
}

#[test]
fn test_data_uri_blocked_by_default() {
    let csp = ContentSecurityPolicy::parse("default-src 'self'");
    assert!(!csp.is_resource_allowed("img", "data:image/png;base64,abc", None));
}

#[test]
fn test_blob_uri_blocked_by_default() {
    let csp = ContentSecurityPolicy::parse("default-src 'self'");
    assert!(!csp.is_resource_allowed("script", "blob:https://example.com/uuid", None));
}
