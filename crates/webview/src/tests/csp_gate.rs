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
    ln: e.lineNumber, col: e.columnNumber,
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
    // M2-s2 源位置：被阻止内联脚本内容起点的 1-based 行列（第 14 行 `<script>` 8
    // 字符 → 内容起点 col 9；blockeduri-inline 15:9 同口径）。
    assert!(
        vio.contains(r#""ln":14"#) && vio.contains(r#""col":9"#),
        "lineNumber/columnNumber 为内容起点 1-based 行列: {vio}"
    );
    assert!(vio.contains("'nonce-ok'"), "originalPolicy 为原始政策串: {vio}");
    // M2-s2 元素站 target（targeting corpus 语义）：target = 被阻止 script 元素。
    assert!(
        vio.contains(r#""t":"SCRIPT""#),
        "target 为被阻止 script 元素（元素站派发）: {vio}"
    );
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

/// markup img CSP 检查点（security-hardening M2-s2）：`img-src 'none'` 下 markup img
/// 不 fetch/不 load + onerror 派发（run_page_scripts 起点）。img-src-none-blocks
/// corpus 形态。
#[test]
fn csp_img_gate_blocks_markup_img_and_fires_error_sh1_m2s2() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    let html = r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="img-src 'none'">
</head><body>
<img src="/content-security-policy/support/fail.png"
     onload="globalThis.__loaded = 1;"
     onerror="globalThis.__errored = 1;">
<script>globalThis.__setup = 1;</script>
</body></html>"#;
    // runner 调用序：fetch_page_images（img 抓取 + CSP 阻止队列入队）→ load_html →
    // run_page_scripts（shim 就绪后派发 error）。
    let _external_css = wv.fetch_page_images(html, "https://wpt.test/csp/case.html");
    wv.load_html(html, None);
    wv.run_page_scripts().expect("run page scripts");
    assert_eq!(
        wv.execute_script("String(globalThis.__loaded)").unwrap(),
        "undefined",
        "被 CSP 阻止的 markup img 不得 load"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__errored)").unwrap(),
        "1",
        "被阻止 markup img 派发 error（onerror）"
    );
}

/// inline `<style>` CSP 检查点（security-hardening M2-s3）：`style-src 'none'` 下被
/// 阻止的 style 元素内容清空（元素保留）+ violation 元素站派发（target = STYLE）。
#[test]
fn csp_style_gate_blocks_inline_style_sh1_m2s3() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    let html = r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="style-src 'none'">
<style>body{color:red}</style>
</head><body>
<script>
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push({
    d: e.effectiveDirective, b: e.blockedURI, p: e.originalPolicy,
    t: e.target === document ? 'document' : (e.target && e.target.tagName)
  });
});
</script>
</body></html>"#;
    let _ = wv.fetch_page_images(html, "https://wpt.test/csp/case.html");
    wv.load_html(html, None);
    wv.run_page_scripts().expect("run page scripts");
    // style 元素保留 + 内容清空（样式不生效面）。
    assert_eq!(
        wv.execute_script("document.getElementsByTagName('style').length")
            .unwrap(),
        "1",
        "被阻止 style 元素保留（target 可达）"
    );
    assert!(
        wv.execute_script("document.getElementsByTagName('style')[0].textContent.trim()")
            .unwrap()
            .is_empty(),
        "被阻止 style 内容清空"
    );
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert!(
        vio.contains(r#""d":"style-src""#),
        "effectiveDirective 如实（政策指令名）: {vio}"
    );
    assert!(vio.contains(r#""b":"inline""#), "blockedURI=inline: {vio}");
    assert!(vio.contains(r#""t":"STYLE""#), "target 为被阻止 style 元素: {vio}");
}
