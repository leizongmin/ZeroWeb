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
    assert!(
        vio.contains(r#""d":"script-src-elem""#),
        "元素面口径（script-blocked 族 corpus 断言）: {vio}"
    );
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

/// kill-switch off（显式 false）零变更：同页全部脚本照常执行、无 violation。
/// M5 后 default-on——本测试验证宿主回退面（kill-switch 语义保留）。
#[test]
fn csp_gate_off_by_default_zero_delta_sh1_m2s1() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: false,
        ..WebViewConfig::default()
    });
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
    // 装配面不激活：kill-switch off 下 context 无政策（后台状态零残留）。
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
        vio.contains(r#""d":"style-src-elem""#),
        "元素面口径（style-blocked 族 corpus 断言）: {vio}"
    );
    assert!(vio.contains(r#""b":"inline""#), "blockedURI=inline: {vio}");
    assert!(vio.contains(r#""t":"STYLE""#), "target 为被阻止 style 元素: {vio}");
}

/// s412（html-semantics M2）：声明式 CSS module 双臂 CSP 检查——`<style type=module>`
/// 具 module script + style 双重身份，两腿镜像 driving corpus（tentative/
/// style-element-csp-*，file-Timeout→Pass 面）：
/// 腿 A `script-src 'nonce-s'; style-src 'unsafe-inline'`——无 nonce module 被
/// script 臂阻止：violation effectiveDirective=script-src-elem（恰一条）+ 元素站
/// error（addEventListener 面——onerror attr 被 script-src-attr 检查点阻止，corpus
/// 同款用 addEventListener 注册）+ 内容清空；普通 style 零位移（script 臂不适用）。
/// 腿 B `style-src 'none'`（无 script-src）——module 被 style 臂阻止：violation
/// style-src-elem + onerror attr 面派发（无 script-src 政策 attr 编译放行）。
#[test]
fn csp_style_gate_module_script_arm_blocks_and_fires_error_s412() {
    // 腿 A：script 臂阻止 + addEventListener error 面（csp-script-src-blocked 形）。
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    let html = r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'nonce-abc123'; style-src 'unsafe-inline';">
<style id="mod" type="module" specifier="foo">
    #test {color:blue}
</style>
<style id="plain">body{color:red}</style>
<script nonce="abc123">
globalThis.__vio = [];
globalThis.__modError = 0;
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push(e.effectiveDirective + '@' + (e.target && e.target.tagName));
});
document.getElementById('mod').addEventListener('error', function() {
  globalThis.__modError = 1;
});
</script>
</head><body></body></html>"#;
    let _ = wv.fetch_page_images(html, "https://wpt.test/csp/case.html");
    wv.load_html(html, None);
    wv.run_page_scripts().expect("run page scripts");
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert_eq!(
        vio, r#"["script-src-elem@STYLE"]"#,
        "module 被 script 臂阻止（style-src unsafe-inline 放行——violation 恰一条）: {vio}"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__modError)").unwrap(),
        "1",
        "被阻止声明式 module 派发元素站 error（addEventListener）"
    );
    assert!(
        wv.execute_script("document.getElementById('mod').textContent.trim()")
            .unwrap()
            .is_empty(),
        "被阻止 module 内容清空"
    );
    assert!(
        !wv.execute_script("document.getElementById('plain').textContent.trim()")
            .unwrap()
            .is_empty(),
        "普通 style 不入 script 臂（style-src unsafe-inline 放行，内容保留）"
    );

    // 腿 B：style 臂阻止 + onerror attr 面（csp-blocked 形——无 script-src 政策，
    // attr 编译放行）。
    let mut wv2 = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv2.prepare_document_state("https://wpt.test/csp/case-b.html");
    let html_b = r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="style-src 'none';">
<style id="mod" type="module" specifier="foo" onerror="globalThis.__modError = 1;">
    #test {color:blue}
</style>
<script>
globalThis.__modError = 0;
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push(e.effectiveDirective);
});
</script>
</head><body></body></html>"#;
    let _ = wv2.fetch_page_images(html_b, "https://wpt.test/csp/case-b.html");
    wv2.load_html(html_b, None);
    wv2.run_page_scripts().expect("run page scripts");
    let vio_b = wv2.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert_eq!(
        vio_b, r#"["style-src-elem"]"#,
        "module 被 style 臂阻止（violation 恰一条——无 script-src 政策不产 script 臂）: {vio_b}"
    );
    assert_eq!(
        wv2.execute_script("String(globalThis.__modError)").unwrap(),
        "1",
        "被阻止声明式 module 派发元素站 error（onerror attr）"
    );
}

/// connect-src 检查点（security-hardening M2-s5）：`connect-src 'none'` 下 fetch
/// 被阻止（reject 面）+ violation document 站派发（effectiveDirective=connect-src）。
#[test]
fn csp_connect_gate_blocks_fetch_sh1_m2s5() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        fetch_handler: Some(std::sync::Arc::new(|_req: &zero_engine::fetch_bridge::FetchRequest| {
            Ok(zero_engine::fetch_bridge::FetchResponse {
                status: 200,
                status_text: "OK".into(),
                headers: vec![("content-type".into(), "text/plain".into())],
                body: "ok".into(),
                body_bytes: None,
            })
        })),
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    let html = r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="connect-src 'none'">
</head><body>
<script>
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push({ d: e.effectiveDirective, b: e.blockedURI });
});
globalThis.__fetchOutcome = 'pending';
fetch('/content-security-policy/support/fail.txt')
  .then(function(res) { globalThis.__fetchOutcome = 'resolved:ok=' + res.ok + ':status=' + res.status; })
  .catch(function() { globalThis.__fetchOutcome = 'rejected'; });
</script>
</body></html>"#;
    let _ = wv.fetch_page_images(html, "https://wpt.test/csp/case.html");
    wv.load_html(html, None);
    wv.run_page_scripts().expect("run page scripts");
    // fetch promise reject（异步面经 probe tick 排空）。
    for _ in 0..10 {
        let _ = wv.execute_script("0;");
    }
    // shim 契约：host 错误 wire → resolve(ok:false) Response（网络错误语义走 XHR
    // error 面——connect-src-blocked corpus 的 XHR 族依赖）。
    assert_eq!(
        wv.execute_script("String(globalThis.__fetchOutcome)").unwrap(),
        "resolved:ok=false:status=0",
        "被 connect-src 阻止的 fetch 返回 ok:false（网络错误语义）"
    );
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert!(vio.contains(r#""d":"connect-src""#), "effectiveDirective: {vio}");
    assert!(
        vio.contains("https://wpt.test/content-security-policy/support/fail.txt"),
        "blockedURI 为绝对 URL: {vio}"
    );
}

/// eval 门禁（security-hardening M2-s6）：`script-src` 缺 unsafe-eval 时 eval()/
/// new Function 抛 EvalError + violation 入队（document 站 blockedURI="eval"）；
/// 'unsafe-eval' 存在时 eval 照常（unsafe_eval corpus 形态）。
#[test]
fn csp_eval_gate_blocks_eval_and_function_sh1_m2s6() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    let html = r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'self' 'unsafe-inline'">
</head><body>
<script>
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push({ d: e.effectiveDirective, b: e.blockedURI });
});
globalThis.__evalErr = 'no-throw';
globalThis.__fnErr = 'no-throw';
try { eval('1+1'); } catch (e) { globalThis.__evalErr = e.name; }
try { new Function('1+1'); } catch (e) { globalThis.__fnErr = e.name; }
</script>
</body></html>"#;
    wv.load_html(html, None);
    wv.run_page_scripts().expect("run page scripts");
    assert_eq!(
        wv.execute_script("String(globalThis.__evalErr)").unwrap(),
        "EvalError",
        "eval() 被 CSP 阻止抛 EvalError"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__fnErr)").unwrap(),
        "EvalError",
        "new Function() 被 CSP 阻止抛 EvalError"
    );
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert!(vio.contains(r#""b":"eval""#), "blockedURI=eval: {vio}");
    assert!(vio.contains(r#""d":"script-src""#), "eval 面无 -elem 子分: {vio}");
}

/// 'unsafe-eval' 存在 → eval 照常执行、零 violation（零拦截面）。
#[test]
fn csp_eval_gate_unsafe_eval_allows_sh1_m2s6() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.load_html(
        r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'unsafe-eval' 'unsafe-inline'">
</head><body>
<script>globalThis.__r = eval('1+1');</script>
</body></html>"#,
        None,
    );
    wv.run_page_scripts().expect("run page scripts");
    assert_eq!(wv.execute_script("String(globalThis.__r)").unwrap(), "2");
}

/// s261：投递内联页面脚本按 data-inline 原始 src 走**外链 URL 检查路径**——
/// `script-src 'self'` 下同源原始 src 放行执行、零 violation。runner 内联投递
///（`data-zw-inlined` + `data-inline` 记录原始 src）此前整体豁免（harness 标记
/// 塌缩），真实浏览器对外链脚本的 URL 源匹配被跳过。红态：HEAD 无 inlined_src
/// 通道 → 该脚本按内联 hash 路径检查 → nonce-only 政策无 hash/无 nonce → 阻止。
/// 监听器带 nonce 保两态可观测。
#[test]
fn csp_gate_inlined_page_script_url_path_s261() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    wv.load_html(
        r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'self' 'nonce-ok'">
</head><body>
<script nonce="ok">
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push(e.blockedURI);
});
</script>
<script data-zw-inlined data-inline="/resources/ok.js">
globalThis.__ran = 1;
</script>
</body></html>"#,
        None,
    );
    wv.run_page_scripts().expect("run page scripts");
    assert_eq!(
        wv.execute_script("String(globalThis.__ran)").unwrap(),
        "1",
        "同源原始 src 的投递内联页面脚本必须放行执行（URL 路径）"
    );
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert_eq!(vio, "[]", "同源放行零 violation: {vio}");
}

/// s261：投递内联页面脚本异源原始 src → 阻止 + blockedURI 为绝对 URL
///（与真外链同口径，csp_gate_external_source_match 镜像）。
#[test]
fn csp_gate_inlined_page_script_cross_origin_blocked_s261() {
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
<script data-zw-inlined data-inline="https://evil.test/evil.js">
globalThis.__blocked_ran = 1;
</script>
</body></html>"#,
        None,
    );
    wv.run_page_scripts().expect("run page scripts");
    assert_eq!(
        wv.execute_script("String(globalThis.__blocked_ran)").unwrap(),
        "undefined",
        "异源原始 src 的投递内联页面脚本必须被阻止"
    );
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert!(
        vio.contains("https://evil.test/evil.js"),
        "blockedURI 为原始 src 绝对 URL: {vio}"
    );
}

/// s261 伴随钉（护轨非红态——标记翻转在 runner 投递面，webview 门禁条件代码
/// 不变；红态证据承 runner delivery 钉）：投递内联页面脚本（data-zw-inlined）
/// 装 eval 包装——`script-src 'self'`（无 unsafe-eval）下页面 eval 抛 EvalError
/// + violation。s253 勘根「CSP 页外链页脚本整体豁免 eval 门禁」面的修复后契约。
#[test]
fn csp_eval_gate_wraps_inlined_page_script_s261() {
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
<script data-zw-inlined data-inline="/resources/eval-user.js">
globalThis.__evalErr = 'no-throw';
try { eval('1+1'); } catch (e) { globalThis.__evalErr = e.name; }
</script>
</body></html>"#,
        None,
    );
    wv.run_page_scripts().expect("run page scripts");
    assert_eq!(
        wv.execute_script("String(globalThis.__evalErr)").unwrap(),
        "EvalError",
        "投递内联页面脚本的 eval 必须被 eval 门禁阻止（包装须安装）"
    );
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert!(vio.contains("eval"), "blockedURI=eval: {vio}");
}

#[test]
fn csp_eval_probe_tmp() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.load_html(
        r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'self' 'unsafe-inline'">
</head><body><script>globalThis.__x=1;</script></body></html>"#,
        None,
    );
    wv.run_page_scripts().unwrap();
    let out = wv
        .execute_script(
            r#"(function(){
var out=[];
out.push('cb:'+(typeof __zwCspEvalBlocked));
out.push('ret:'+(typeof __zwCspEvalBlocked==='function'?__zwCspEvalBlocked():'n/a'));
out.push('evalIsNative:'+(eval===globalThis.eval));
try{eval('1+1');out.push('throw:no');}catch(e){out.push('throw:'+e.name);}
return out.join('|');
})()"#,
        )
        .unwrap();
    println!("EVALPROBE: {out}");
}

/// script-src-attr 检查点（security-hardening M2-s7）：unsafe-hashes + hash 匹配的
/// onclick 处理器放行执行；不匹配处理器被阻止（script-src-attr violation）。
/// script-src-event-handler-on-inline-script corpus 形态。
#[test]
fn csp_attr_gate_hash_allows_and_blocks_sh1_m2s7() {
    let hash = "KMqmvVOJ9XW5OiOAYYYPPTFk+Zj/3KrlSEyqWgqibwU=";
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    let html = format!(
        r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'nonce-dummy' 'unsafe-hashes' 'sha256-{hash}'">
</head><body>
<script nonce="dummy">
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {{
  globalThis.__vio.push({{ d: e.effectiveDirective }});
}});
window.eventHandlerExecuted = false;
var ok = document.createElement('div');
ok.setAttribute('onclick', 'window.eventHandlerExecuted = true');
ok.dispatchEvent(new Event('click'));
var bad = document.createElement('div');
bad.setAttribute('onclick', 'window.eventHandlerExecuted = "evil"');
bad.dispatchEvent(new Event('click'));
</script>
</body></html>"#
    );
    wv.load_html(&html, None);
    wv.run_page_scripts().expect("run page scripts");
    assert_eq!(
        wv.execute_script("String(window.eventHandlerExecuted)").unwrap(),
        "true",
        "hash 匹配处理器放行执行"
    );
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert!(
        vio.contains(r#""d":"script-src-attr""#),
        "不匹配处理器阻止 + script-src-attr 口径: {vio}"
    );
}

/// 内联 hash 源 = 元素 child text content 原文（s349）：带首/尾换行的内联脚本按
/// 原文摘要放行执行、零 violation（旧 trim 口径摘要失配 → 误阻 + violation——
/// 本测试在旧行为下红）。CSP source-list hash matching，corpus 形态
/// script-text-modifications-csp（pin = 含首换行原文摘要）。
/// https://w3c.github.io/webappsec-csp/#source-list-hash-matching-algorithm
#[test]
fn csp_inline_hash_matches_raw_text_content_s349() {
    // raw = "\nwindow.__zwRawHashExecuted = true;\n" 的 sha256-base64（trim 后
    // 摘要 CKQHcnGb2FyyX5+lJnraB2otESJ6kZ0q/8aQnqOadEA= 与 pin 不等——旧口径误阻）。
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    wv.load_html(
        r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="script-src 'nonce-ok' 'sha256-VYeK9Im5b0Ls9v5/mzH2XB5a0S6bNa5b2VbV3awTe1Q='">
</head><body>
<script nonce="ok">
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push({ d: e.effectiveDirective });
});
</script>

<script>
window.__zwRawHashExecuted = true;
</script>
</body></html>"#,
        None,
    );
    wv.run_page_scripts().expect("run page scripts");
    // 原文摘要放行执行（旧 trim 口径下恒 blocked → undefined）。
    assert_eq!(
        wv.execute_script("String(window.__zwRawHashExecuted)").unwrap(),
        "true",
        "带缘空白内联脚本按原文 hash 放行"
    );
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert_eq!(vio.trim(), "[]", "hash 匹配零 violation: {vio}");
}

/// 运行时 img src-set 检查点（security-hardening M2-s8）：createElement('img') +
/// src set 在 `img-src 'none'` 下被阻止 + violation 元素站派发（target = IMG）+
/// onerror。securitypolicyviolation img corpus 形态。
#[test]
fn csp_runtime_img_gate_blocks_src_set_sh1_m2s8() {
    let mut wv = WebView::new(WebViewConfig {
        csp_enforcement: true,
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/csp/case.html");
    let html = r#"<html><head>
<meta http-equiv="Content-Security-Policy" content="img-src 'none'">
</head><body>
<script>
globalThis.__vio = [];
document.addEventListener('securitypolicyviolation', function(e) {
  globalThis.__vio.push({
    b: e.blockedURI, d: e.effectiveDirective, p: e.originalPolicy,
    d: e.effectiveDirective, b: e.blockedURI
  });
});
var i = document.createElement('img');
i.onerror = function() { globalThis.__errored = 1; };
i.src = '/content-security-policy/support/fail.png';
</script>
</body></html>"#;
    wv.load_html(html, None);
    wv.run_page_scripts().expect("run page scripts");
    let vio = wv.execute_script("JSON.stringify(globalThis.__vio)").unwrap();
    assert!(
        vio.contains("https://wpt.test/content-security-policy/support/fail.png"),
        "blockedURI 为绝对 URL: {vio}"
    );
    assert!(vio.contains(r#""d":"img-src""#), "effectiveDirective: {vio}");
    // 口径注记：violation 走 document 站（corpus img 族不断言 target——
    // securitypolicyviolation-block-image 断言 documentURI/blockedURI/指令面字段）。
    assert_eq!(
        wv.execute_script("String(globalThis.__errored)").unwrap(),
        "1",
        "被阻止运行时 img 派发 error"
    );
}
