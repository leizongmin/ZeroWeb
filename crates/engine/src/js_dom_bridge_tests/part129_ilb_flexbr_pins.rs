// 编辑命令兼容 ilb I flex/grid br 元素面钉测（2026-11-17 刀，ILB-FLEXBR）：
// corpus insertlinebreak.html 残红 4 键（i7e2 刀③后基线 1674P/17F 在册，
// census ④族）——声明 pre 系（_zwCeWsSignificant 命中，非 forceLf）坍缩行断
// 垫 `<br>` 元素占位（Chrome 142 实录 `abc\n<br>`，语料 :925-932 flex／
// :987-996 grid），br 忽略容器仍须插 br 元素非文本换行；forceLf（inline div
// 无声明）维持 \n 垫。控制键现役绿形 `[]abc`→`\nabc`／`a[]bc`→`a\nbc`
//（非坍缩行断不垫）。exp 锁引擎序列化形（attr 双引号、\n 文本字面）。
#[test]
fn test_ilb_flexbr_pins() {
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
        "<html><body><div id=ce129a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐，part128 复刻）——
function nextNode129(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex129(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv129(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode129(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex129(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex129(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex129(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex129(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode129(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode129(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP129 = [];
function pin129(id, html, expOut) {
  var h = document.getElementById("ce129a");
  var err = null;
  var ok = false;
  try {
    h.innerHTML = "";
    var pts = setupDiv129(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    ok = document.execCommand("insertlinebreak", false, "");
  } catch (e129) { err = String(e129 && e129.message || e129); }
  globalThis.__zwP129.push({ id: id, ok: ok, err: err, out: h.innerHTML,
    expOut: expOut });
}

// I 族 4 键（corpus idx 925/930/987/992）——坍缩行断补 <br> 元素占位。
pin129("ilb-flexbr-flex-end", "<div style=display:flex;white-space:pre><span>abc[]</span></div>",
  "<div style=\"display:flex;white-space:pre\"><span>abc\n<br></span></div>");
pin129("ilb-flexbr-iflex-end", "<div style=display:inline-flex;white-space:pre><span>abc[]</span></div>",
  "<div style=\"display:inline-flex;white-space:pre\"><span>abc\n<br></span></div>");
pin129("ilb-flexbr-grid-end", "<div style=display:grid;white-space:pre><span>abc[]</span></div>",
  "<div style=\"display:grid;white-space:pre\"><span>abc\n<br></span></div>");
pin129("ilb-flexbr-igrid-end", "<div style=display:inline-grid;white-space:pre><span>abc[]</span></div>",
  "<div style=\"display:inline-grid;white-space:pre\"><span>abc\n<br></span></div>");
// 控制键（corpus 现役绿形 idx 905/915）——非坍缩行断不垫（\n 后有内容）。
pin129("ilb-flexbr-flex-start-control", "<div style=display:flex;white-space:pre><span>[]abc</span></div>",
  "<div style=\"display:flex;white-space:pre\"><span>\nabc</span></div>");
pin129("ilb-flexbr-flex-mid-control", "<div style=display:flex;white-space:pre><span>a[]bc</span></div>",
  "<div style=\"display:flex;white-space:pre\"><span>a\nbc</span></div>");

// 断言回路：err / ok / out 失配即 throw（execute 层 Err → Rust panic 带
// got/exp 清单）。
var bad129 = [];
for (var pi = 0; pi < globalThis.__zwP129.length; pi++) {
  var c = globalThis.__zwP129[pi];
  if (c.err || !c.ok || c.out !== c.expOut) {
    bad129.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut);
  }
}
if (bad129.length) throw "ILBFLEXBRPIN MISMATCH x" + bad129.length + " | " + bad129.join(" || ");
"##,
        )
        .expect("ilb flexbr pins: all forms must hold");
}
