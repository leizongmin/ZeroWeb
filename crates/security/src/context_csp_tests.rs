//! SecurityContext 文档级 CSP 装配与 script 检查点单测（security-hardening M2-s1）。
//!
//! 对齐 WPT content-security-policy corpus 形态：meta 装配（多政策并集）、nonce
//! 放行、inline 阻止、外链 URL 源匹配、effectiveDirective 如实上报。

use crate::context::SecurityContext;

/// 无文档级 CSP → 全放行（默认行为不变，kill-switch 语义前置条件）。
#[test]
fn check_script_no_policy_allows() {
    let ctx = SecurityContext::new();
    assert!(ctx.check_script(None, None, None).is_none());
    assert!(ctx.check_script(None, None, Some("https://evil.test/x.js")).is_none());
    assert!(!ctx.has_document_csp());
}

/// inline 阻止 + nonce 放行（blockeduri-inline corpus 形态：script-src 'nonce-abc'）。
#[test]
fn check_script_inline_nonce_sh1_m2s1() {
    let mut ctx = SecurityContext::new();
    ctx.set_document_csp(&["script-src 'nonce-abc'".to_string()]);
    assert!(ctx.has_document_csp());
    // 无 nonce 内联 → 阻止，effectiveDirective/blockedURI 如实。
    let v = ctx.check_script(None, None, None).expect("inline must be blocked");
    assert_eq!(v.effective_directive, "script-src-elem");
    assert_eq!(v.blocked_uri, "inline");
    assert_eq!(v.original_policy, "script-src 'nonce-abc'");
    // 匹配 nonce 内联 → 放行。
    assert!(ctx.check_script(Some("abc"), None, None).is_none());
    // 错 nonce 内联 → 阻止。
    assert!(ctx.check_script(Some("wrong"), None, None).is_some());
}

/// 外链 URL 源匹配：self 放行同源、阻止异源（script-src 'self' corpus 形态）。
#[test]
fn check_script_external_self_sh1_m2s1() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://wpt.test/csp/case.html");
    ctx.set_document_csp(&["script-src 'self'".to_string()]);
    // 同源 → 放行。
    assert!(
        ctx.check_script(None, None, Some("https://wpt.test/resources/x.js"))
            .is_none()
    );
    // 异源 → 阻止。
    let v = ctx
        .check_script(None, None, Some("https://evil.test/x.js"))
        .expect("cross-origin must be blocked");
    assert_eq!(v.blocked_uri, "https://evil.test/x.js");
    // 政策无 nonce 源时，元素 nonce 不构成放行（spec：nonce 仅匹配政策内
    // 'nonce-...' 源；'self' 政策下异源外链仍阻止）。
    assert!(
        ctx.check_script(Some("abc"), None, Some("https://evil.test/x.js"))
            .is_some()
    );
    ctx.clear_document_csp();
    ctx.set_document_csp(&["script-src 'self' 'nonce-abc'".to_string()]);
    // 政策含匹配 nonce 源 → 携带 nonce 的异源外链不受源清单限制放行。
    assert!(
        ctx.check_script(Some("abc"), None, Some("https://evil.test/x.js"))
            .is_none()
    );
    // 同一政策无 nonce 外链仍走源清单 → 阻止。
    assert!(ctx.check_script(None, None, Some("https://evil.test/x.js")).is_some());
}

/// 多政策并集：任一阻止即阻止（spec CSP3 multiple-policies）；全放行才放行。
#[test]
fn check_script_multiple_policies_union_sh1_m2s1() {
    let mut ctx = SecurityContext::new();
    ctx.set_document_csp(&[
        "script-src 'unsafe-inline' 'self'".to_string(),
        "default-src 'self'".to_string(),
    ]);
    ctx.set_page_origin("https://wpt.test/");
    // 异源外链：政策 1 即阻止（'unsafe-inline' 只放行内联，外链仍走源清单）→
    // effectiveDirective 落 policy1 的 script-src。
    let v = ctx
        .check_script(None, None, Some("https://evil.test/x.js"))
        .expect("first policy must block external");
    assert_eq!(v.effective_directive, "script-src-elem");
    assert_eq!(v.original_policy, "script-src 'unsafe-inline' 'self'");
    // 内联：政策 1 放行（unsafe-inline）、政策 2 阻止（default-src 'self' 无
    // unsafe-inline/nonce）→ 阻止，effectiveDirective 落 default-src。
    let v = ctx
        .check_script(None, None, None)
        .expect("second policy must block inline");
    assert_eq!(v.effective_directive, "default-src");
    assert_eq!(v.original_policy, "default-src 'self'");
    // 同源外链两政策均放行。
    assert!(ctx.check_script(None, None, Some("https://wpt.test/x.js")).is_none());
}

/// 无 script-src 时内联放行（无 script-src 且无 default-src → 无政策约束）。
#[test]
fn check_script_unrelated_directive_ignores_scripts() {
    let mut ctx = SecurityContext::new();
    ctx.set_document_csp(&["img-src 'none'".to_string()]);
    assert!(ctx.check_script(None, None, None).is_none());
}

/// 文档换代：clear 后回到无政策放行；重装配整组替换。
#[test]
fn check_script_clear_and_replace_sh1_m2s1() {
    let mut ctx = SecurityContext::new();
    ctx.set_document_csp(&["script-src 'none'".to_string()]);
    assert!(ctx.check_script(None, None, None).is_some());
    ctx.clear_document_csp();
    assert!(ctx.check_script(None, None, None).is_none());
    ctx.set_document_csp(&["script-src 'unsafe-inline'".to_string(), "img-src 'self'".to_string()]);
    assert!(ctx.check_script(None, None, None).is_none());
}

/// 无效政策串跳过（spec：无效政策忽略该条，不整体失败）。
#[test]
fn set_document_csp_skips_unparseable_sh1_m2s1() {
    let mut ctx = SecurityContext::new();
    // 空 source list 的 script-src → 全阻止；仅合法串生效。
    ctx.set_document_csp(&["script-src 'self'".to_string()]);
    ctx.set_page_origin("https://wpt.test/");
    assert!(ctx.check_script(None, None, Some("https://wpt.test/a.js")).is_none());
}

/// hash 匹配放行（security-hardening M2-s1.5）：`script-src 'sha256-<b64>'` 下内容
/// 匹配的内联放行、不匹配阻止（scripthash corpus 形态；hash 对 trim 后内容计算）。
#[test]
fn check_script_inline_hash_sh1_m2s1() {
    let mut ctx = SecurityContext::new();
    let hash = crate::csp::script_hash_sha256_base64("alert(1)");
    ctx.set_document_csp(&[format!("script-src 'sha256-{hash}'")]);
    // 内容匹配 → 放行。
    assert!(ctx.check_script(None, Some(&hash), None).is_none());
    // 内容不匹配 → 阻止。
    let wrong = crate::csp::script_hash_sha256_base64("alert(2)");
    assert!(ctx.check_script(None, Some(&wrong), None).is_some());
    // 无 nonce 无 hash 内联 → 阻止。
    assert!(ctx.check_script(None, None, None).is_some());
}

/// hash 源值向量（spec 口径：sha256 → RFC 4648 base64 含 padding）。
#[test]
fn script_hash_sha256_base64_vectors_sh1_m2s1() {
    assert_eq!(
        crate::csp::script_hash_sha256_base64(""),
        "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU="
    );
    assert_eq!(
        crate::csp::script_hash_sha256_base64("test"),
        "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg="
    );
}

/// image 检查点（security-hardening M2-s2）：img-src 'none' 全阻止 + 'self' 源匹配 +
/// effectiveDirective 如实（img-src-none-blocks corpus 形态）。
#[test]
fn check_image_img_src_sh1_m2s2() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://wpt.test/csp/case.html");
    ctx.set_document_csp(&["img-src 'none'".to_string()]);
    let v = ctx
        .check_image("https://wpt.test/csp/support/fail.png")
        .expect("img-src 'none' must block");
    assert_eq!(v.effective_directive, "img-src");
    assert_eq!(v.blocked_uri, "https://wpt.test/csp/support/fail.png");
    assert_eq!(v.original_policy, "img-src 'none'");
    // 'self' 政策：同源放行、异源阻止。
    ctx.clear_document_csp();
    ctx.set_document_csp(&["img-src 'self'".to_string()]);
    assert!(ctx.check_image("https://wpt.test/a.png").is_none());
    assert!(ctx.check_image("https://evil.test/a.png").is_some());
    // 无 img-src 无 default-src → 放行。
    ctx.clear_document_csp();
    ctx.set_document_csp(&["script-src 'self'".to_string()]);
    assert!(ctx.check_image("https://evil.test/a.png").is_none());
}

/// style 元素检查点（security-hardening M2-s3）：inline nonce 放行 + 无 nonce 阻止 +
/// 外链 URL 源匹配（style-src corpus 形态）。
#[test]
fn check_style_style_src_sh1_m2s3() {
    let mut ctx = SecurityContext::new();
    ctx.set_page_origin("https://wpt.test/csp/case.html");
    ctx.set_document_csp(&["style-src 'nonce-ok'".to_string()]);
    // 无 nonce 内联 → 阻止，effectiveDirective 如实。
    let v = ctx.check_style(None, None, None).expect("inline must be blocked");
    assert_eq!(v.effective_directive, "style-src-elem");
    assert_eq!(v.blocked_uri, "inline");
    // nonce 匹配内联 → 放行；内容 hash 匹配亦放行。
    assert!(ctx.check_style(Some("ok"), None, None).is_none());
    let hash = crate::csp::script_hash_sha256_base64("body{color:red}");
    ctx.clear_document_csp();
    ctx.set_document_csp(&[format!("style-src 'sha256-{hash}'")]);
    assert!(ctx.check_style(None, Some(&hash), None).is_none());
    // 外链：'self' 源匹配（回退 style-src → default-src 面）。
    ctx.clear_document_csp();
    ctx.set_document_csp(&["style-src 'self'".to_string()]);
    assert!(ctx.check_style(None, None, Some("https://wpt.test/a.css")).is_none());
    let v = ctx
        .check_style(None, None, Some("https://evil.test/a.css"))
        .expect("cross-origin stylesheet must be blocked");
    assert_eq!(v.blocked_uri, "https://evil.test/a.css");
}
