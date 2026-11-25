// s10k22 刀㉒ secondary-document 常驻钉测（2026-11-25）
// corpus 形蒸馏：WPT trusted-types/trusted-types-secondary-document 30F 收敛
//（97/0）——`document.implementation.createHTMLDocument` 产物是 MEl plain 世界
//（R125 按 tag 接口原型：script→HTMLScriptElement.prototype、iframe→
// HTMLIFrameElement.prototype、miss→HTMLElement.prototype），旧整层缺 TT 门/API
// ＋stub 意外蒙混。钉面九向（enforced 静态 meta 页）＋负面一页：
//   V1 setAttribute 双向＋值断言（DIV onclick → TrustedScript）；
//   V2 setAttributeNS ns 印记（SVG script xlink:href → TrustedScriptURL，
//     qualified/local 双读视图）；
//   V3 node 绑定族——detached `.value=` IDL DOMString 边界不查（Trusted 品牌
//     在此 String 化失传）＋setAttributeNode(NS) 绑定域复检拒字符串（corpus
//     acceptTrustedTypeArgumentInIDL 仅直设 setter 有＝node 绑定族无 works 面
//     的真实浏览器语义）；
//   V4 innerHTML／V5 srcdoc（值断言）／V6 script text 三名——textContent
//     own 直门＋text/innerText 原型门→`__zwTtAccBypass` 委托让位（防双门
//     回归面：委托值已 String 化失品牌，二进门误抛）；
//   V7 setHTMLUnsafe（方法存在＋双向）；
//   V8 insertAdjacentHTML works/throws＋插入位置实证；
//   V9 execCommand——inserthtml 门内双向＋非 inserthtml 命令门外直返 false；
//   N 负面页——无 meta enforcement off：全 sink plain 直通零抛＋ns 印记
//     与强制无关（独立沙箱两页互不污染闩锁闭包）。
// https://w3c.github.io/webappsec-trusted-types/dist/spec/#enforcement-in-documents
// https://w3c.github.io/webappsec-trusted-types/dist/spec/#enforcement-in-scripts
const S10K22_SD_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function throwsT(fn, m) {
    try { fn(); fails.push('FAIL:' + m + ' 未抛'); }
    catch (e) { if (!(e instanceof TypeError)) fails.push('FAIL:' + m + ' 抛非 TypeError:' + (e && e.name)); }
  }
  function noThrow(fn, m) { try { fn(); } catch (e) { fails.push('FAIL:' + m + ' 不应抛:' + (e && e.message || e)); } }
  var pp = trustedTypes.createPolicy('sdpolicy', {
    createScript: function (s) { return s; },
    createHTML: function (s) { return s; },
    createScriptURL: function (s) { return s; }
  });
  var sdoc = document.implementation.createHTMLDocument('sd');
  var th = pp.createHTML('<b>hi</b>');
  var ts = pp.createScript('somevalue');
  var tu = pp.createScriptURL('somevalue');
  // V1 setAttribute 双向 + 值断言（DIV onclick → TrustedScript）
  var d1 = sdoc.createElement('div');
  noThrow(function () { d1.setAttribute('onclick', ts); }, 'V1 setAttribute works');
  chk(eq(d1.getAttribute('onclick'), 'somevalue', 'V1 setAttribute 值'));
  throwsT(function () { d1.setAttribute('onclick', 'plain'); }, 'V1 setAttribute plain');
  // V2 setAttributeNS ns 印记（SVG script xlink:href → TrustedScriptURL）
  var sv = sdoc.createElementNS('http://www.w3.org/2000/svg', 'script');
  noThrow(function () { sv.setAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href', tu); }, 'V2 setAttributeNS works');
  chk(eq(sv.getAttributeNS('http://www.w3.org/1999/xlink', 'href'), 'somevalue', 'V2 getAttributeNS 印记'));
  chk(eq(sv.getAttribute('xlink:href'), 'somevalue', 'V2 qualified 读'));
  throwsT(function () { sv.setAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href', 'plain'); }, 'V2 setAttributeNS plain');
  // V3 node 绑定族（spec-true 形）：detached attr `.value=` 是 IDL DOMString
  // 边界——Trusted 品牌**在此 String 化失传**（corpus acceptTrustedTypeArgument
  // InIDL 仅直设 setter 有＝node 绑定族无 works 面的真实浏览器语义根因）；
  // detached 写不查 enforcement（attr ownerElement null 不调 set-an-attribute-
  // value），绑定域（setAttributeNode(NS) → setAttributeNS 单咽喉）复检字符串。
  // https://w3c.github.io/webappsec-trusted-types/dist/spec/#enforcement-in-documents
  var e3 = sdoc.createElement('div');
  var a3 = sdoc.createAttribute('onclick');
  noThrow(function () { a3.value = 'plain'; }, 'V3 detached 写不查');
  throwsT(function () { e3.setAttributeNode(a3); }, 'V3 绑定 plain 复检');
  chk(eq(e3.getAttribute('onclick'), null, 'V3 拒后不落'));
  var e3b = sdoc.createElement('div');
  var a3b = sdoc.createAttribute('onclick');
  noThrow(function () { a3b.value = ts; }, 'V3 detached Trusted 写不抛（IDL 转换）');
  throwsT(function () { e3b.setAttributeNode(a3b); }, 'V3 绑定 Trusted 亦拒（品牌已失）');
  var sv3 = sdoc.createElementNS('http://www.w3.org/2000/svg', 'script');
  var a3c = sdoc.createAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href');
  noThrow(function () { a3c.value = 'plain'; }, 'V3 NS detached 写不查');
  throwsT(function () { sv3.setAttributeNodeNS(a3c); }, 'V3 NS 绑定 plain 复检');
  chk(eq(sv3.getAttributeNS('http://www.w3.org/1999/xlink', 'href'), null, 'V3 NS 拒后不落'));
  // V4 innerHTML / V5 srcdoc（值断言）
  var d4 = sdoc.createElement('div');
  noThrow(function () { d4.innerHTML = th; }, 'V4 innerHTML works');
  throwsT(function () { d4.innerHTML = '<i>x</i>'; }, 'V4 innerHTML plain');
  var f5 = sdoc.createElement('iframe');
  noThrow(function () { f5.srcdoc = th; }, 'V5 srcdoc works');
  chk(eq(f5.srcdoc, '<b>hi</b>', 'V5 srcdoc 值'));
  throwsT(function () { f5.srcdoc = '<p>x</p>'; }, 'V5 srcdoc plain');
  // V6 script text 三名——textContent own 直门 + text/innerText 原型门→
  // AccBypass 委托让位（双门回归面）
  var s6 = sdoc.createElement('script');
  noThrow(function () { s6.textContent = ts; }, 'V6 textContent works');
  chk(eq(s6.textContent, 'somevalue', 'V6 textContent 值'));
  throwsT(function () { s6.textContent = 'plain'; }, 'V6 textContent plain');
  var s6b = sdoc.createElement('script');
  noThrow(function () { s6b.text = ts; }, 'V6 text works（原型门→委托让位）');
  chk(eq(s6b.text, 'somevalue', 'V6 text 值'));
  throwsT(function () { s6b.text = 'plain'; }, 'V6 text plain');
  var s6c = sdoc.createElement('script');
  noThrow(function () { s6c.innerText = ts; }, 'V6 innerText works');
  throwsT(function () { s6c.innerText = 'plain'; }, 'V6 innerText plain');
  // V7 setHTMLUnsafe 存在 + 双向
  var d7 = sdoc.createElement('div');
  chk(isTrue(typeof d7.setHTMLUnsafe === 'function', 'V7 setHTMLUnsafe 存在'));
  noThrow(function () { d7.setHTMLUnsafe(th); }, 'V7 setHTMLUnsafe works');
  throwsT(function () { d7.setHTMLUnsafe('<b>x</b>'); }, 'V7 setHTMLUnsafe plain');
  // V8 IAH works/throws + 插入位置实证
  var d8 = sdoc.createElement('div');
  noThrow(function () { d8.insertAdjacentHTML('afterbegin', th); }, 'V8 IAH works');
  chk(isTrue(d8.firstChild && d8.firstChild.tagName === 'B', 'V8 IAH 插入'));
  throwsT(function () { d8.insertAdjacentHTML('afterbegin', '<b>x</b>'); }, 'V8 IAH plain');
  // V9 execCommand：inserthtml 门内双向 + 非 inserthtml 门外直返 false
  noThrow(function () {
    chk(eq(sdoc.execCommand('insertHTML', false, th), false, 'V9 返 false'));
  }, 'V9 works');
  throwsT(function () { sdoc.execCommand('insertHTML', false, '<b>x</b>'); }, 'V9 plain');
  noThrow(function () {
    chk(eq(sdoc.execCommand('bold', false, 'plain'), false, 'V9 非 inserthtml'));
  }, 'V9 bold 不应抛');
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

// 负面页：无 meta——enforcement off，plain 直通零抛；ns 印记与强制无关
const S10K22_SD_NEG_PIN_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + JSON.stringify(a) + '!=' + JSON.stringify(b) + ')'; return null; }
  function isTrue(a, m) { if (a !== true) return 'FAIL:' + m + '(' + a + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  function noThrow(fn, m) { try { fn(); } catch (e) { fails.push('FAIL:' + m + ' 不应抛:' + (e && e.message || e)); } }
  chk(isTrue(globalThis.__zwTtEnforced() === false, 'N0 无 meta 页 enforcement false'));
  var sdoc = document.implementation.createHTMLDocument('sdn');
  var d1 = sdoc.createElement('div');
  noThrow(function () { d1.setAttribute('onclick', 'plain'); }, 'N1 setAttribute plain');
  chk(eq(d1.getAttribute('onclick'), 'plain', 'N1 值'));
  var sv = sdoc.createElementNS('http://www.w3.org/2000/svg', 'script');
  noThrow(function () { sv.setAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href', 'u'); }, 'N2 setAttributeNS plain');
  chk(eq(sv.getAttributeNS('http://www.w3.org/1999/xlink', 'href'), 'u', 'N2 NS 印记与强制无关'));
  var d3 = sdoc.createElement('div');
  noThrow(function () { d3.innerHTML = '<i>x</i>'; }, 'N3 innerHTML plain');
  var s4 = sdoc.createElement('script');
  noThrow(function () { s4.textContent = 'p1'; s4.text = 'p2'; s4.innerText = 'p3'; }, 'N4 script 三名 plain');
  var f5 = sdoc.createElement('iframe');
  noThrow(function () { f5.srcdoc = '<p>x</p>'; }, 'N5 srcdoc plain');
  var d6 = sdoc.createElement('div');
  noThrow(function () { d6.setHTMLUnsafe('<b>x</b>'); d6.insertAdjacentHTML('afterbegin', '<b>y</b>'); }, 'N6 setHTMLUnsafe/IAH plain');
  noThrow(function () {
    chk(eq(sdoc.execCommand('insertHTML', false, '<b>x</b>'), false, 'N7 返 false'));
  }, 'N7 execCommand plain');
  var d8 = sdoc.createElement('div');
  var a8 = sdoc.createAttribute('onclick');
  noThrow(function () { a8.value = 'x'; d8.setAttributeNode(a8); }, 'N8 setAttributeNode plain');
  chk(eq(d8.getAttribute('onclick'), 'x', 'N8 值'));
  return fails.length ? fails.join('; ') : 'PASS';
})()
"##;

#[test]
fn test_s10k22_secondary_doc_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // secondary-document 同构基座：CSP meta 在场（enforced 静态形）
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>k22sd</title><meta http-equiv=\"Content-Security-Policy\" content=\"require-trusted-types-for 'script';\"></head><body><div id=\"k22host\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k22.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K22_SD_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k22 sd execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k22 secondary-doc 钉测失败：{}", verdict);
}

#[test]
fn test_s10k22_secondary_doc_negative_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head><title>k22sdneg</title></head><body><div id=\"k22neg\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/trusted-types/pins-k22-neg.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let verdict = match sandbox.execute(S10K22_SD_NEG_PIN_JS) {
        Ok(v) => v.value,
        Err(e) => panic!("s10k22 sd-neg execute error: {}", e),
    };
    assert_eq!(verdict, "PASS", "s10k22 secondary-doc 负面钉测失败：{}", verdict);
}
