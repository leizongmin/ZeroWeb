// js_dom_bridge 测试切片 158。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-sanitizer 刀⑫ 常驻钉测（2026-11-25）——setHTML 写链视图与 detached doc
/// write 物化（spec：https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html
/// #dom-element-sethtmlunsafe / #dom-document-parsehtmlunsafe；语料
/// sanitizer-basic-filtering.html text/0 ×4＋comments/0 ×2＋comments/1 ×4＋
/// dataAttributes/2 ×6 的常驻回归载体）：
/// ① handle 域写链视图恒重建（旧 `'<'` 捷径把无标记值视图清空——纯文本
///    setHTMLUnsafe/setHTML 的 childNodes 读面整丢）；
/// ② comments 开关双路径（setHTML comments:false 剥 comment 保 text；
///    comments:true 全保留）；
/// ③ dataAttributes:false 时显式允许名单仍放行（config attributes:['data-x']）；
/// ④ detached doc `write` 物化快照遍历（旧 iterate-mutate 丢约一半顶层子——
///    parseHTMLUnsafe 多子内容 T,C,T / C,C 形态保真）。
/// 根因链与判读见 evidence `dc2-s10k12-basic-filtering-writechain-knife12-verdict.md`。
const S10K12_WRITECHAIN_KNIFE12_PINS_JS: &str = r#"
globalThis.__k12 = (function () {
  var bad = [];
  var doc = globalThis.document;
  function kidsSig(n) {
    var k = n.childNodes || [];
    var out = [];
    for (var i = 0; i < k.length; i++) {
      var c = k[i];
      if (!c) { out.push('NULL'); continue; }
      if (c.nodeType === 3) out.push('T(' + c.data + ')');
      else if (c.nodeType === 8) out.push('C(' + c.data + ')');
      else out.push('E<' + String(c.tagName || c.nodeName) + '>');
    }
    return out.join('|');
  }
  function attrNames(el) {
    var out = [];
    try {
      var a = el.attributes;
      for (var i = 0; i < a.length; i++) out.push(String(a[i].name));
    } catch (e) {}
    return out.join(',');
  }

  // ① handle 域纯文本视图（text/0 四 sink 形）。
  var d1 = doc.createElement('div');
  d1.setHTMLUnsafe('text');
  if (kidsSig(d1) !== 'T(text)') bad.push('p1:' + kidsSig(d1));
  var d1b = doc.createElement('div');
  d1b.setHTMLUnsafe('a  b');
  if (kidsSig(d1b) !== 'T(a  b)') bad.push('p1b:' + kidsSig(d1b));
  var d1c = doc.createElement('div');
  d1c.setHTML('plain text');
  if (kidsSig(d1c) !== 'T(plain text)') bad.push('p1c:' + kidsSig(d1c));

  // ② comments 双路径（剥 comment 后 serialize+reparse 合并文本＝语料期望
  // "a  b" 单 text；全保留回归钉）。
  var d2 = doc.createElement('div');
  d2.setHTML('a <!-- c --> b', { sanitizer: { comments: false } });
  if (kidsSig(d2) !== 'T(a  b)') bad.push('p2:' + kidsSig(d2));
  var d2b = doc.createElement('div');
  d2b.setHTMLUnsafe('a <!-- c --> b');
  if (kidsSig(d2b) !== 'T(a )|C( c )|T( b)') bad.push('p2b:' + kidsSig(d2b));
  var d2c = doc.createElement('div');
  d2c.setHTML('a <!-- c --> b', { sanitizer: { comments: true } });
  if (kidsSig(d2c) !== 'T(a )|C( c )|T( b)') bad.push('p2c:' + kidsSig(d2c));

  // ③ dataAttributes:false ＋ 显式名单（dataAttributes/2 形）。
  var d3 = doc.createElement('div');
  d3.setHTML('<p data-x="1" data-y="2" data-z="3">', { sanitizer: { attributes: ['data-x'], dataAttributes: false } });
  var ps = d3.querySelector('p');
  if (!ps) { bad.push('p3-noel'); } else {
    if (attrNames(ps) !== 'data-x') bad.push('p3:' + attrNames(ps));
  }

  // ④ detached doc write 物化（comments/0 parse 族形＋双 comment 形）。
  var w1 = globalThis.Document.parseHTMLUnsafe('a <!-- c --> b');
  if (!w1 || !w1.body) bad.push('p4-nodoc');
  else if (kidsSig(w1.body) !== 'T(a )|C( c )|T( b)') bad.push('p4:' + kidsSig(w1.body));
  var w2 = globalThis.Document.parseHTMLUnsafe('<!-- c1 --><!-- c2 -->');
  if (!w2 || !w2.body) bad.push('p4b-nodoc');
  else if (kidsSig(w2.body) !== 'C( c1 )|C( c2 )') bad.push('p4b:' + kidsSig(w2.body));
  var w3 = globalThis.Document.parseHTMLUnsafe('<p>x</p>tail');
  if (!w3 || !w3.body) bad.push('p4c-nodoc');
  else if (kidsSig(w3.body) !== 'E<P>|T(tail)') bad.push('p4c:' + kidsSig(w3.body));

  // ⑤ ShadowRoot 域纯文本（SR.setHTML text 形——handle 域 SR 臂）。
  var h5 = doc.createElement('div');
  var sr5 = h5.attachShadow({ mode: 'open' });
  sr5.setHTML('text');
  if (kidsSig(sr5) !== 'T(text)') bad.push('p5:' + kidsSig(sr5));

  return bad.length === 0 ? { ok: true } : { ok: false, bad: bad.join('|') };
})();
"#;

#[test]
fn test_s10k12_writechain_knife12_pins() {
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
        "<html><head><title>k12</title></head><body><div id=\"k12page\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/sanitizer-api/pins-k12.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K12_WRITECHAIN_KNIFE12_PINS_JS).unwrap();
    let verdict = sandbox.execute("JSON.stringify(globalThis.__k12)").unwrap().value;
    assert!(
        verdict.contains("\"ok\":true"),
        "刀⑫ 写链视图/parse 物化/dataAttributes 名单（text 视图恒重建/comments 双路径/显式名单放行/detached write 快照遍历）：{}",
        verdict
    );
}
