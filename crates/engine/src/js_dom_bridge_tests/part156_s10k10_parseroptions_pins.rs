// js_dom_bridge 测试切片 156。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-sanitizer 刀⑩ 常驻钉测（2026-11-24）——default policy createParserOptions 链
/// （spec：TT enforced 时每个 HTML 解析 sink 解析前经 default policy 回调取得
/// ParserOptions；语料 sethtml-with-trustedtypes.tentative.html 129F +
/// parsehtmlunsafe 6F + immutable 5F 簇的常驻回归载体）：
/// ① setHTMLUnsafe inject sanitizer config（span 移除、div 保留）；
/// ② innerHTML/outerHTML/IAH inject 同链抽查；
/// ③ ccf runScripts=false 产物 script 盖 already-started 章不执行＋default 执行对照
///    （刀⑨ 执行语义零变化）；
/// ④ createParserOptions 返回非 object（123/null）→ TypeError 且零落盘；
/// ⑤ 回调异常原样 rethrow（Error 身份保持）；
/// ⑥ createHTML 先于 createParserOptions（序断言——createHTML 抛则回调零调用）；
/// ⑦ immutable：回调变异不回写 caller（dict 深拷贝＋Sanitizer 实例 clone 隔离）；
/// ⑧ TrustedParserOptions 品牌直通：sanitizer() 方法形（每调新 clone）＋
///    parseHTMLUnsafe 第二参直通消费；
/// ⑨ parseHTMLUnsafe 无 caller options 走 default policy 回调（inject＋returning 校验）。
/// 根因链与 140F 簇见 evidence `dc2-s10k10-parseroptions-knife10-verdict.md`。
const S10K10_PARSEROPTIONS_KNIFE10_PINS_JS: &str = r#"
globalThis.__k10 = (function () {
  var bad = [];
  var doc = globalThis.document;
  // enforcement 走种子 HTML 的 CSP meta（require-trusted-types-for 'script'）——
  // 与语料页同形。不覆盖 `__zwTtEnforced`：sink 管道（part14）读闭包内 CSP 解析，
  // 只覆盖 global 会让 trap 门/ sink 短路分叉（首版 p6 实证：createHTML 未跑、
  // createParserOptions 反被调）。
  var tt = globalThis.trustedTypes;
  var _k10po = function (o) { return o; };
  var _k10html = function (h) { return h; };
  var _poCalled = 0;
  tt.createPolicy('default', {
    createHTML: function (h) { return _k10html(h); },
    createParserOptions: function (o) { _poCalled++; return _k10po(o); },
  });
  function mk() { var d = doc.createElement('div'); doc.body.appendChild(d); return d; }
  function cleanup() { _k10po = function (o) { return o; }; _k10html = function (h) { return h; }; _poCalled = 0; }

  // ① setHTMLUnsafe inject config。
  var d1 = mk();
  _k10po = function (o) { var c = {}; for (var k in o) c[k] = o[k]; c.sanitizer = { removeElements: ['span'] }; return c; };
  d1.setHTMLUnsafe("<div id='k10a'><span id='k10f'></span></div>");
  if (d1.querySelector('#k10f') !== null) bad.push('p1-span');
  if (d1.querySelector('#k10a') === null) bad.push('p1-div');
  cleanup();

  // ② innerHTML inject。
  var d2 = mk();
  _k10po = function (o) { return { sanitizer: { removeElements: ['span'] } }; };
  d2.innerHTML = "<div id='k10b'><span id='k10f2'></span></div>";
  if (d2.querySelector('#k10f2') !== null) bad.push('p2-span');
  if (d2.querySelector('#k10b') === null) bad.push('p2-div');
  cleanup();

  // ③ ccf runScripts=false 章＋default 执行对照。
  var d3 = mk();
  globalThis.__k10ran1 = false;
  _k10po = function (o) { return { runScripts: false }; };
  var r3 = doc.createRange(); r3.setStart(d3, 0); r3.setEnd(d3, 0);
  var f3 = r3.createContextualFragment("<script>globalThis.__k10ran1 = true;<\/script>");
  d3.appendChild(f3);
  if (globalThis.__k10ran1 !== false) bad.push('p3-ran:' + globalThis.__k10ran1);
  globalThis.__k10ran2 = false;
  var d3b = mk();
  _k10po = function (o) { return o; };
  var r3b = doc.createRange(); r3b.setStart(d3b, 0); r3b.setEnd(d3b, 0);
  var f3b = r3b.createContextualFragment("<script>globalThis.__k10ran2 = true;<\/script>");
  d3b.appendChild(f3b);
  if (globalThis.__k10ran2 !== true) bad.push('p3b-notran:' + globalThis.__k10ran2);
  cleanup();

  // ④ returning 非 object → TypeError＋零落盘。
  var d4 = mk();
  _k10po = function (o) { return 123; };
  var threw4 = null;
  try { d4.setHTMLUnsafe("<div id='k10d4'>x</div>"); } catch (e) { threw4 = e; }
  if (!threw4 || threw4.name !== 'TypeError') bad.push('p4-throw:' + (threw4 && threw4.name));
  if (d4.textContent !== '') bad.push('p4-landed:' + JSON.stringify(d4.textContent));
  _k10po = function (o) { return null; };
  threw4 = null;
  try { d4.innerHTML = 'y'; } catch (e) { threw4 = e; }
  if (!threw4 || threw4.name !== 'TypeError') bad.push('p4b-throw:' + (threw4 && threw4.name));
  cleanup();

  // ⑤ 回调异常原样 rethrow。
  var d5 = mk();
  var _err5 = new Error('k10 po oops');
  _k10po = function (o) { throw _err5; };
  var threw5 = null;
  try { d5.innerHTML = 'x'; } catch (e) { threw5 = e; }
  if (threw5 !== _err5) bad.push('p5-rethrow:' + (threw5 && threw5.message));
  cleanup();

  // ⑥ createHTML 先于 createParserOptions。
  var d6 = mk();
  _k10html = function (h) { throw new Error('k10 html oops'); };
  _k10po = function (o) { _poCalled = _poCalled + 100; return o; };
  var threw6 = null;
  try { d6.setHTMLUnsafe('x'); } catch (e) { threw6 = e; }
  if (!threw6) bad.push('p6-nothrow');
  if (threw6 && threw6.name !== 'Error') bad.push('p6-wrapped:' + threw6.name);
  if (_poCalled !== 0) bad.push('p6-po-called:' + _poCalled);
  cleanup();

  // ⑦ immutable：dict 变异隔离＋Sanitizer 实例隔离。part12 归一层把 caller 的
  // dict sanitizer 转成 Sanitizer 实例交付回调（JSON 深拷贝→构造器），回调侧
  // 变异走方法形 removeElement；断言面＝caller 原对象零污染＋净化生效。
  var d7 = mk();
  var opt7 = { sanitizer: {}, modified: false };
  _k10po = function (o) {
    o.modified = true;
    if (o.sanitizer instanceof globalThis.Sanitizer) o.sanitizer.removeElement('span');
    else if (o.sanitizer && typeof o.sanitizer === 'object') o.sanitizer.removeElements = ['span'];
    return o;
  };
  d7.setHTMLUnsafe("<div id='k10g'><span id='k10f7'></span></div>", opt7);
  if (opt7.modified !== false) bad.push('p7-mutated');
  if ('removeElements' in opt7.sanitizer) bad.push('p7-san-mutated');
  if (d7.querySelector('#k10f7') !== null) bad.push('p7-span');
  var d7b = mk();
  var s7 = new globalThis.Sanitizer();
  var opt7b = { sanitizer: s7 };
  _k10po = function (o) {
    if (o.sanitizer instanceof globalThis.Sanitizer) o.sanitizer.removeElement('span');
    return o;
  };
  d7b.setHTMLUnsafe("<div id='k10h'><span id='k10f7b'></span></div>", opt7b);
  var has7b = false;
  var els7b = s7.get().elements || [];
  for (var i7b = 0; i7b < els7b.length; i7b++) { if (els7b[i7b].name === 'span') { has7b = true; break; } }
  if (!has7b) bad.push('p7b-original-touched');
  if (d7b.querySelector('#k10f7b') !== null) bad.push('p7b-span');
  cleanup();

  // ⑧ TrustedParserOptions 品牌直通＋方法形。
  var pol8 = tt.createPolicy('k10p8', { createParserOptions: function (o) { return o; } });
  var opt8 = pol8.createParserOptions({ sanitizer: { removeElements: ['span'] } });
  if (typeof opt8.sanitizer !== 'function') bad.push('p8-sannotfn:' + typeof opt8.sanitizer);
  else {
    var s8a = opt8.sanitizer(); var s8b = opt8.sanitizer();
    if (!(s8a instanceof globalThis.Sanitizer)) bad.push('p8-notinst');
    if (s8a === s8b) bad.push('p8-sameclone');
  }
  var doc8 = globalThis.Document.parseHTMLUnsafe("<div id='k10i'><span id='k10f8'></span></div>", opt8);
  if (doc8.querySelector('#k10f8') !== null) bad.push('p8-span');
  if (doc8.querySelector('#k10i') === null) bad.push('p8-div');

  // ⑨ parseHTMLUnsafe 无 caller options 走 default policy 回调。
  _k10po = function (o) { return { sanitizer: { removeElements: ['span'] } }; };
  var doc9 = globalThis.Document.parseHTMLUnsafe("<div id='k10j'><span id='k10f9'></span></div>");
  if (doc9.querySelector('#k10f9') !== null) bad.push('p9-span');
  if (doc9.querySelector('#k10j') === null) bad.push('p9-div');
  _k10po = function (o) { return null; };
  var threw9 = null;
  try { globalThis.Document.parseHTMLUnsafe('<p>x</p>'); } catch (e) { threw9 = e; }
  if (!threw9 || threw9.name !== 'TypeError') bad.push('p9-throw:' + (threw9 && threw9.name));
  cleanup();

  // ⑩ outerHTML／IAH inject 抽查。
  var d10 = mk();
  var inner10 = doc.createElement('div');
  d10.appendChild(inner10);
  _k10po = function (o) { return { sanitizer: { removeElements: ['span'] } }; };
  inner10.outerHTML = "<div id='k10k'><span id='k10f10'></span></div>";
  if (d10.querySelector('#k10f10') !== null) bad.push('p10-span');
  if (d10.querySelector('#k10k') === null) bad.push('p10-div');
  var d10b = mk();
  d10b.insertAdjacentHTML('beforeend', "<div id='k10l'><span id='k10f10b'></span></div>");
  if (d10b.querySelector('#k10f10b') !== null) bad.push('p10b-span');
  if (d10b.querySelector('#k10l') === null) bad.push('p10b-div');
  cleanup();

  return bad.length === 0 ? { ok: true } : { ok: false, bad: bad.join('|') };
})();
"#;

#[test]
fn test_s10k10_parseroptions_knife10_pins() {
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
        "<html><head><meta http-equiv=\"Content-Security-Policy\" \
         content=\"require-trusted-types-for 'script'\"><title>k10</title></head>\
         <body><div id=\"k10page\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/sanitizer-api/pins-k10.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K10_PARSEROPTIONS_KNIFE10_PINS_JS).unwrap();
    let verdict = sandbox.execute("JSON.stringify(globalThis.__k10)").unwrap().value;
    assert!(
        verdict.contains("\"ok\":true"),
        "刀⑩ parser options（default policy 链/校验/副本隔离/品牌方法形/runScripts 章）：{}",
        verdict
    );
}
