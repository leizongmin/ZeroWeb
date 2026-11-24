// js_dom_bridge 测试切片 155。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-domparsing 刀⑨ 常驻钉测（2026-11-24）——IAH 兄弟链/校验/NMA、ccf isEqualNode、
/// ccf 产物 appendChild 展平执行、html context 剥壳（wpt-data 探针页属 gitignored
/// per-clone 刮擦面，按刀①协议用后即删，本钉测即常驻回归载体）：
/// ① Fix A（兄弟链视图）：IAH beforebegin/afterend/afterbegin/beforeend 二深链＋script
///    不执行＋连续两次插入不回落 stale 基底（R3254-K3 清基底行拆除防回归）；
/// ② Fix B（校验恒开）：非法 position（"bar"/土耳其 "beforebegİn"）SyntaxError；
///    detached 元素 beforebegin/afterend NMA；documentElement（父为 Document）NMA；
///    对照 beforeend/afterbegin 不受 NMA 臂影响（corpus "foobar" 次序面）；
/// ③ Fix C：ccf 产物 isEqualNode 成员（等价输入 true / 异输入 false）＋ownerDocument；
/// ④ Fix D：ccf 产物（plain nodeType 11）appendChild 进 body → 子节点父反链＋内联
///    script 执行；innerHTML 产物带 `__zwFragNoExec` 章同流免执行（单通道分流对照）；
/// ⑤ html context 剥壳：body.afterend / head.beforebegin 的 `<p>` 面不产生重复
///    head/body；documentElement.beforeend 显式 `<head>/<body>` 壳输入壳不落文档；
/// ⑥ testEquivalence 精确镜像："<body><p>" vs "<p>" 等价（ccf spec 步骤 6 html
///    context 重定向 body——既有回落臂防回归）。
/// 根因链与 18F/2F/11F 簇见 evidence `dc2-s10k9-domparsing-knife9-verdict.md`。
const S10K9_DOMPARSING_KNIFE9_PINS_JS: &str = r#"
globalThis.__k9 = (function () {
  var bad = [];
  var doc = globalThis.document;

  // ① Fix A 兄弟链。
  var a1 = doc.getElementById('k9page');
  a1.insertAdjacentHTML('beforebegin', '<script>globalThis.__k9ranA = true;<\/script><i id="k9i1"></i>');
  if (!a1.previousSibling || a1.previousSibling.localName !== 'i') bad.push('a1-prev:' + (a1.previousSibling && a1.previousSibling.localName));
  if (!a1.previousSibling || !a1.previousSibling.previousSibling || a1.previousSibling.previousSibling.localName !== 'script') bad.push('a1-prev2:miss');
  if (globalThis.__k9ranA === true) bad.push('a1-script-ran');
  a1.insertAdjacentHTML('beforebegin', '<b id="k9b1"></b>');
  if (!a1.previousSibling || a1.previousSibling.localName !== 'b') bad.push('a1-second:' + (a1.previousSibling && a1.previousSibling.localName));

  var a2 = doc.getElementById('k9page');
  a2.insertAdjacentHTML('afterend', '<a id="k9a2"></a><u id="k9u2"></u>');
  if (!a2.nextSibling || a2.nextSibling.localName !== 'a') bad.push('a2-next:' + (a2.nextSibling && a2.nextSibling.localName));
  if (!a2.nextSibling || !a2.nextSibling.nextSibling || a2.nextSibling.nextSibling.localName !== 'u') bad.push('a2-next2:miss');

  var a3 = doc.getElementById('k9ab');
  a3.insertAdjacentHTML('afterbegin', '<b id="k9abb"></b><script>globalThis.__k9ranAB = true;<\/script>');
  if (!a3.firstChild || a3.firstChild.localName !== 'b') bad.push('a3-first:' + (a3.firstChild && a3.firstChild.localName));
  if (!a3.firstChild || !a3.firstChild.nextSibling || a3.firstChild.nextSibling.localName !== 'script') bad.push('a3-second:miss');
  if (globalThis.__k9ranAB === true) bad.push('a3-script-ran');

  var a4 = doc.getElementById('k9be');
  a4.insertAdjacentHTML('beforeend', '<script>globalThis.__k9ranBE = true;<\/script><u id="k9u4"></u>');
  if (!a4.lastChild || a4.lastChild.localName !== 'u') bad.push('a4-last:' + (a4.lastChild && a4.lastChild.localName));
  if (!a4.lastChild || !a4.lastChild.previousSibling || a4.lastChild.previousSibling.localName !== 'script') bad.push('a4-penult:miss');
  if (globalThis.__k9ranBE === true) bad.push('a4-script-ran');

  // ② Fix B 校验。
  var b1 = doc.createElement('div');
  var threw = null;
  try { b1.insertAdjacentHTML('bar', 'x'); } catch (e) { threw = e; }
  if (!threw || threw.name !== 'SyntaxError') bad.push('b1-syntax:' + (threw && threw.name));
  threw = null;
  try { b1.insertAdjacentHTML('beforebegİn', 'x'); } catch (e) { threw = e; }
  if (!threw || threw.name !== 'SyntaxError') bad.push('b1-tt:' + (threw && threw.name));
  threw = null;
  try { b1.insertAdjacentHTML('beforebegin', 'x'); } catch (e) { threw = e; }
  if (!threw || threw.name !== 'NoModificationAllowedError') bad.push('b1-nma:' + (threw && threw.name));
  threw = null;
  try { b1.insertAdjacentHTML('afterend', 'x'); } catch (e) { threw = e; }
  if (!threw || threw.name !== 'NoModificationAllowedError') bad.push('b1-nma2:' + (threw && threw.name));
  threw = null;
  try { doc.documentElement.insertAdjacentHTML('afterend', 'x'); } catch (e) { threw = e; }
  if (!threw || threw.name !== 'NoModificationAllowedError') bad.push('b1-docnma:' + (threw && threw.name));
  b1.insertAdjacentHTML('beforeend', 'foo');
  b1.insertAdjacentHTML('afterBegin', 'bar');
  if (b1.textContent !== 'barfoo') bad.push('b1-order:' + JSON.stringify(b1.textContent));

  // ③ Fix C isEqualNode。
  var c1r1 = doc.createRange(); c1r1.selectNodeContents(doc.documentElement);
  var c1r2 = doc.createRange(); c1r2.selectNodeContents(doc.documentElement);
  var c1f1 = c1r1.createContextualFragment('<p>x');
  var c1f2 = c1r2.createContextualFragment('<p>x');
  if (typeof c1f1.isEqualNode !== 'function') bad.push('c1-member');
  else if (!c1f1.isEqualNode(c1f2)) bad.push('c1-eq:false');
  var c1f3 = c1r1.createContextualFragment('<p>y');
  if (typeof c1f1.isEqualNode === 'function' && c1f1.isEqualNode(c1f3)) bad.push('c1-neq:true');
  if (c1f1.firstChild && c1f1.firstChild.ownerDocument !== doc) bad.push('c1-owner');

  // ④ Fix D ccf 产物 appendChild 展平＋script 执行；innerHTML 章机制对照。
  globalThis.__k9ranD = false;
  var d1r = doc.createRange(); d1r.selectNodeContents(doc.documentElement);
  var d1f = d1r.createContextualFragment('<div id="k9d1"></div><script>globalThis.__k9ranD = true;<\/script>');
  doc.body.appendChild(d1f);
  if (globalThis.__k9ranD !== true) bad.push('d1-script:' + globalThis.__k9ranD);
  if (!d1f.firstChild || d1f.firstChild.parentNode !== doc.body) bad.push('d1-parent');
  globalThis.__k9ranD2 = false;
  var d2 = doc.createElement('div');
  d2.innerHTML = '<script>globalThis.__k9ranD2 = true;<\/script>';
  doc.body.appendChild(d2);
  if (globalThis.__k9ranD2 === true) bad.push('d2-innerhtml-script-ran');

  // ⑤ html context 剥壳。
  doc.body.insertAdjacentHTML('afterend', '<p id="k9p5"></p>');
  var eHead = doc.getElementsByTagName('head').length;
  var eBody = doc.getElementsByTagName('body').length;
  if (eHead !== 1) bad.push('e1-head:' + eHead);
  if (eBody !== 1) bad.push('e1-body:' + eBody);
  doc.head.insertAdjacentHTML('beforebegin', '<p id="k9p5b"></p>');
  if (doc.getElementsByTagName('head').length !== 1) bad.push('e2-head:' + doc.getElementsByTagName('head').length);
  doc.documentElement.insertAdjacentHTML('beforeend', '<head><meta id="k9m5"></head><body><p id="k9p5c"></p></body>');
  if (doc.getElementsByTagName('head').length !== 1) bad.push('e3-head:' + doc.getElementsByTagName('head').length);
  if (doc.getElementsByTagName('body').length !== 1) bad.push('e3-body:' + doc.getElementsByTagName('body').length);

  // ⑥ testEquivalence 精确镜像（html context → body 回落既有臂；corpus 无 id 等价形）。
  var f6r = doc.createRange(); f6r.selectNodeContents(doc.documentElement);
  var f6a = f6r.createContextualFragment('<body><p>');
  var f6b = f6r.createContextualFragment('<p>');
  if (!f6a.isEqualNode(f6b)) bad.push('f6-equiv');

  return bad.length === 0 ? { ok: true } : { ok: false, bad: bad.join('|') };
})();
"#;

#[test]
fn test_s10k9_domparsing_knife9_pins() {
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
        "<html><head><title>k9</title></head><body><div id=\"k9page\"></div>\
         <div id=\"k9ab\"></div><div id=\"k9be\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/domparsing/pins-k9.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K9_DOMPARSING_KNIFE9_PINS_JS).unwrap();
    let verdict = sandbox.execute("JSON.stringify(globalThis.__k9)").unwrap().value;
    assert!(
        verdict.contains("\"ok\":true"),
        "刀⑨ domparsing（IAH 兄弟链/校验/NMA＋ccf isEqualNode/appendChild 执行＋html context 剥壳）：{}",
        verdict
    );
}
