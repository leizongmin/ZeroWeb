// 编辑命令兼容 ilb E2/E3 表格单元域钉测（2026-11-17 刀⑥表格域，ILB-SELTBL）：
// corpus insertlinebreak.html 残红 2 键（i7g0 刀⑤后基线 1689P/2F 在册）——
// (1) E2 `<td>[foo<td>bar]` 跨单元选区：delete 前导 contained-nodes 表结构排除
//（spec implementation.js :4689-4693 thead/tbody/tfoot/tr/th/td 永不入删除节点
// 表）内层文本逐个摘除＋空 cell 占位 br＋caret (td,0)；插入面 DTP-REMAP td/th
// 臂（br 合法子落 childNodes[so]，:7383 insert br 步）——exp `<td><br><br></td>`
// （新 br 占位 br 前、caret 塌 (td,1)，语料带 caret 形 `<td><br>{}<br></td>`）；
// (2) E3 `<tbody data-start=0 data-end=1>` 表节选区：ILB-SELTBL 下钻排除面（表
// 结构容器不逐层下钻，探针实录下钻改写端点致 caret 落 td 内）＋删除机制自产面
//（tr 保留、双 cell 占位 br、collapseToStart (tbody,0)）＋ILB-TBLSCT 守卫 ②
//（spec :7396-7401 start Element br 非合法子 return true，tbody 零 br no-op）——
// exp `<tbody><tr><td><br></td>…`（语料带 caret 形 `<tbody>{}<tr>…`）。控制键：
// E1 跨界 td 双 br 现役绿形恒一（part130 已钉，此处补 host 级 caret 位形）。
// setupDiv 复刻 tests.js :5313-5325 data-start/data-end 语义（元素 attr＝
// (该元素, attr 值) 边界）＋bracket 走域（part130 同款）。
#[test]
fn test_ilb_seltbl_pins() {
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
        "<html><body><div id=ce131a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（data-start 分支 :5313-5325＋bracket 走域
// part130 复刻）——
function nextNode131(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex131(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv131(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var sEl = node.querySelector("[data-start]");
  if (sEl) {
    startNode = sEl;
    startOffset = parseInt(sEl.getAttribute("data-start"));
    sEl.removeAttribute("data-start");
  }
  var eEl = node.querySelector("[data-end]");
  if (eEl) {
    endNode = eEl;
    endOffset = parseInt(eEl.getAttribute("data-end"));
    eEl.removeAttribute("data-end");
  }
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode131(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex131(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex131(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex131(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex131(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode131(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode131(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP131 = [];
function pin131(id, html, expOut) {
  var h = document.getElementById("ce131a");
  var err = null;
  var ok = false;
  try {
    h.innerHTML = "";
    var pts = setupDiv131(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    ok = document.execCommand("insertlinebreak", false, "");
  } catch (e131) { err = String(e131 && e131.message || e131); }
  globalThis.__zwP131.push({ id: id, ok: ok, err: err, out: h.innerHTML,
    expOut: expOut });
}

// E2（corpus idx 33）——跨单元选区删除＋td/th 入壳臂。
pin131("ilb-seltbl-cross-cell",
  "<table><tr><td>[foo<td>bar]<tr><td>baz<td>quz</table>",
  "<table><tbody><tr><td><br><br></td><td><br></td></tr>"
    + "<tr><td>baz</td><td>quz</td></tr></tbody></table>");
// E3（corpus idx 34）——表节选区下钻排除＋guard ② no-op。
pin131("ilb-seltbl-tbody-sel",
  "<table><tbody data-start=0 data-end=1><tr><td>foo<td>bar<tr><td>baz<td>quz</table>",
  "<table><tbody><tr><td><br></td><td><br></td></tr>"
    + "<tr><td>baz</td><td>quz</td></tr></tbody></table>");
// 控制键（现役绿形恒一）：E1 跨界 td 双 br＋块界补断（part130 同源，此处
// 锁表格域改动不翻转）＋E4 host 级 table containment 恒一。
pin131("ilb-seltbl-ctl-cross-table",
  "fo[o<table><tr><td>b]ar</table>",
  "fo<br><br><table><tbody><tr><td>ar</td></tr></tbody></table>");
pin131("ilb-seltbl-ctl-host-table",
  "{<table><tr><td>foo</table>}",
  "<br><br>");

// 断言回路：err / ok / out 失配即 throw（execute 层 Err → Rust expect 带
// got/exp 清单）。
var bad131 = [];
for (var pi = 0; pi < globalThis.__zwP131.length; pi++) {
  var c = globalThis.__zwP131[pi];
  if (c.err || !c.ok || c.out !== c.expOut) {
    bad131.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut);
  }
}
if (bad131.length) throw "ILBSELTBLPIN MISMATCH x" + bad131.length + " | " + bad131.join(" || ");
"##,
        )
        .expect("ilb seltbl pins: all forms must hold");
}
