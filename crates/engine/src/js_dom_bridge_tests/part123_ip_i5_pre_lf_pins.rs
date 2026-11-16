// 编辑命令兼容 ip 残 census I5 刀钉测（2026-11-16）：pre 劈裂空右块垫 br 臂
// 认「左块尾字面 \n＝持行已足」（part01f pre/address/listing 支路免视守卫，
// white-space 感知）→ 单 br 垫新空行免三行形。语料锚：insertparagraph.js
// :367 `<pre>foo&#10;[]</pre>` exp 三选一（`foo<br>\n`/`foo\n<br>`/
// `foo<br><br>`，after 串无 selection marker 不锁 caret——尾 \n 隐行，语料
// 注释 "trailing linefeed is invisible"）。控制键四形现役绿形恒一：裸尾无
// \n 双 br 维持（:340）；br 后有 \n 文本不经免视臂（:375）；左尾 \n 但 nxt
// 存在守卫不误伤（:380）；br 后有旧 br 元素（:345）。caret/marker 剥离照抄
// 语料 include/tests.js setupDiv（part122 同款）。

#[test]
fn test_ip_i5_pre_lf_pins() {
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
        "<html><body><div id=ce123a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐，part122 复用）——
function nextNode123(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex123(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv123(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode123(cur); continue; }
    var data = cur.data.replace(/\]/g, "");
    var startIdx = data.indexOf("[");
    data = cur.data.replace(/\[/g, "");
    var endIdx = data.indexOf("]");
    cur.data = cur.data.replace(/[\[\]]/g, "");
    if (startIdx != -1) { startNode = cur; startOffset = startIdx; }
    if (endIdx != -1) { endNode = cur; endOffset = endIdx; }
    data = cur.data.replace(/\}/g, "");
    var elStartIdx = data.indexOf("{");
    data = cur.data.replace(/\{/g, "");
    var elEndIdx = data.indexOf("}");
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex123(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex123(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex123(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex123(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode123(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode123(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP123 = [];
// expSel 传 null 表语料未锁 caret（:367 exp 三选一无 selection marker，
// compare 仅核 innerHTML——I5 键不断 caret）；控制键面语料锁 caret 的取
// 现役绿位（实测捕获后锁定）。
function pin123(id, html, dps, expOut, expSel) {
  var h = document.getElementById("ce123a");
  var err = null;
  var ok = false;
  var pts = null;
  try {
    pts = setupDiv123(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    document.execCommand("defaultparagraphseparator", false, dps);
    ok = document.execCommand("insertparagraph", false, "");
  } catch (e123) { err = String(e123 && e123.message || e123); }
  var sel = "?";
  try {
    var sa = getSelection().anchorNode;
    sel = (sa ? (sa.nodeName + "@" + getSelection().anchorOffset) : "null");
  } catch (e123s) {}
  globalThis.__zwP123.push({ id: id, ok: ok, err: err, sel: sel, out: h.innerHTML,
    expOut: expOut, expSel: expSel });
}
// ① I5 本刀：pre 尾字面 \n 已持行免双 br（corpus :367 exp 三选一取定形）
pin123("i5-pre-lf", "<pre>foo&#10;[]</pre>", "div",
  "<pre>foo\n<br></pre>", null);
// ② 控制：裸尾无 \n 双 br 维持（corpus :340 现役绿形，caret PRE@2 实测锁定）
pin123("ctrl-pre-bare-end", "<pre>foo[]</pre>", "div",
  "<pre>foo<br><br></pre>", "PRE@2");
// ③ 控制：br 后有 \n 文本不经免视臂（corpus :375 现役绿形，caret #text@3
// ——collapse 原文本节点 offset+1 越界钳制落 "foo" 尾，现役 corpus 绿位）
pin123("ctrl-pre-lf-after", "<pre>foo[]&#10;</pre>", "div",
  "<pre>foo<br>\n</pre>", "#text@3");
// ④ 控制：左尾 \n 但 nxt 存在守卫不误伤（corpus :380 现役绿形，caret
// #text@4 同 ③ 钳制语义）
pin123("ctrl-pre-lf-both", "<pre>foo&#10;[]&#10;</pre>", "div",
  "<pre>foo\n<br>\n</pre>", "#text@4");
// ⑤ 控制：br 后有旧 br 元素（corpus :345 现役绿形，caret PRE@2 实测锁定）
pin123("ctrl-pre-br-after", "<pre>foo[]<br></pre>", "div",
  "<pre>foo<br><br></pre>", "PRE@2");

// 断言回路：err/ok/out 失配即 throw；expSel 非 null 时才核 caret（execute 层
// Err → Rust panic 带 got/exp）。
var bad123 = [];
for (var pi = 0; pi < globalThis.__zwP123.length; pi++) {
  var c = globalThis.__zwP123[pi];
  if (c.err || !c.ok || c.out !== c.expOut
      || (c.expSel !== null && c.sel !== c.expSel)) {
    bad123.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut
      + " sel=" + c.sel + " expSel=" + c.expSel);
  }
}
if (bad123.length) throw "I5PIN MISMATCH x" + bad123.length + " | " + bad123.join(" || ");
"##,
        )
        .expect("ip I5 pre linefeed pins: all five forms must hold");
}
