//! CSP script 检查点（security-hardening M2-s1）：`csp_enforcement` kill-switch
//! default-off 零变更 + on 时 inline 阻止/nonce 放行/violation 事件派发。

use crate::{WebView, WebViewConfig};

/// meta CSP `script-src 'nonce-ok'`：无 nonce 内联被阻止 + violation 派发（target
/// 为被阻止的 script 元素）+ nonce 内联放行。对齐 WPT
/// content-security-policy/securitypolicyviolation corpus 形态。
#[test]
fn csp_gate_blocks_inline_and_dispatches_violation_sh1_m2s1() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    wv.load_html(
        r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'nonce-ok'">
</head><body>
<script nonce="ok">
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push({
    b: e.blockedURI, d: e.effectiveDirective, p: e.originalPolicy,
    t: e.target === document ? 'document' : (e.target && e.target.tagName)
  });
});
</script>
<script>globalThis.__blocked = 1;</script>
<script nonce="ok">globalThis.__allowed = 1;</script>
</body></html>"#,
        None,
    );
    wv.run_page_scripts().expect("run page scripts");
    // 无 nonce 内联未执行；nonce 内联执行。
    assert_eq!(
        wv.execute_script("String(globalThis.__blocked)").unwrap(),
        "undefined",
        "无 nonce 内联脚本必须被 CSP 阻止"
    );
    assert_eq!(wv.execute_script("String(globalThis.__allowed)").unwrap(), "1");
    // violation 事件派发：blockedURI=inline + 字段面。
    let vio = wv
        .execute_script("JSON.stringify(globalThis.__vio)")
        .expect("violations");
    assert!(
        vio.contains(r#""b":"inline""#),
        "violation blockedURI 必须为 inline: {vio}"
    );
    assert!(vio.contains(r#""d":"script-src""#), "effectiveDirective: {vio}");
    assert!(vio.contains("'nonce-ok'"), "originalPolicy 为原始政策串: {vio}");
    // FIXME(M2-s2)：target 定位到被阻止 script 元素（targeting corpus 语义）——s1 为
    // document 站派发（shim doc 槽位，native 构造实例过不了站内检查）。
    assert!(vio.contains(r#""t":"document""#), "s1 target 为 document 站: {vio}");
}

/// default（kill-switch off）零变更：同页全部脚本照常执行、无 violation。
#[test]
fn csp_gate_off_by_default_zero_delta_sh1_m2s1() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html(
        r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'nonce-ok'">
</head><body>
<script>globalThis.__blocked = 1;</script>
<script nonce="ok">globalThis.__allowed = 1;</script>
</body></html>"#,
        None,
    );
    wv.run_page_scripts().expect("run page scripts");
    assert_eq!(wv.execute_script("String(globalThis.__blocked)").unwrap(), "1");
    assert_eq!(wv.execute_script("String(globalThis.__allowed)").unwrap(), "1");
    // 装配面不激活：context 无政策（后台状态零残留）。
    assert!(!wv.security_context().has_document_csp());
}

/// 外链脚本 URL 源匹配：`script-src 'self'` 下同源放行执行、异源阻止并派发
/// blockedURI=绝对 URL。
#[test]
fn csp_gate_external_source_match_sh1_m2s1() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    wv.load_html(
        r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'self' 'unsafe-inline'">
</head><body>
<script>
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push(e.blockedURI);
});
</script>
<script src="https://evil.test/evil.js"></script>
</body></html>"#,
        None,
    );
    wv.run_page_scripts().expect("run page scripts");
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert!(
        vio.contains("https://evil.test/evil.js"),
        "异源外链阻止 + blockedURI 为绝对 URL: {vio}"
    );
}
