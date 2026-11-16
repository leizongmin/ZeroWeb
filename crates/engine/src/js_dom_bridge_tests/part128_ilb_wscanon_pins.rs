// 编辑命令兼容 ilb H+J 归一域钉测（2026-11-17 刀，ILB-WSCANON-V1）：corpus
// insertlinebreak.html 残红 5 键（i7d0 刀②后基线 1669P/22F 在册，census ③族
// 全量 7 ws 邻接形中 Fail 5 即本刀靶）——行断面空白归一三则：
// (a) 左段尾随普通空格 run 删（H1/H3/H4），(b) 右段全折叠空白整删（H1/H2），
// (c) 右段首部 [空格|nbsp] run 首 nbsp 翻——仅头侧结构内容放行（J 形
// CE-false 壳邻接；`[] abc` 退化头形保形绝不 nbsp——ip 同判据 :13009 注）。
// 语料锚 editing/data/insertlinebreak.js H idx 838/843/860/866、J idx 1002；
// 控制键现役绿形 `<div>[] abc</div>`（退化头恒一）与 ` <span…><br>[]&nbsp;; …`
// （nbsp-first run 翻转恒等，ip "nbsp 首原样" 接受形）。exp 锁引擎序列化形
//（attr 双引号、nbsp 输出 &nbsp; 实体——census 现役绿键 :1009 实证）。
#[test]
fn test_ilb_wscanon_pins() {
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
        "<html><body><div id=ce128a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐，part126/127 复刻）——
function nextNode128(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex128(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv128(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode128(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex128(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex128(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex128(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex128(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode128(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode128(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP128 = [];
function pin128(id, html, expOut) {
  var h = document.getElementById("ce128a");
  var err = null;
  var ok = false;
  try {
    h.innerHTML = "";
    var pts = setupDiv128(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    ok = document.execCommand("insertlinebreak", false, "");
  } catch (e128) { err = String(e128 && e128.message || e128); }
  globalThis.__zwP128.push({ id: id, ok: ok, err: err, out: h.innerHTML,
    expOut: expOut });
}

// H 族（corpus idx 838/843）——左段尾随空格删＋右段 ws-only 删＋双垫。
pin128("ilb-wscanon-h1-tail-ws", "<div>abc [] </div>",
  "<div>abc<br><br></div>");
pin128("ilb-wscanon-h2-wsonly", "<div>abc[]  </div>",
  "<div>abc<br><br></div>");
// H 族（corpus idx 860/866）——退化全 ws 头清空，右段保形落接受变体
//（exp 数组取 `<div><br> abc`/`<div><br>abc`，绝不 nbsp 形）。
pin128("ilb-wscanon-h3-lead-raw", "<div> [] abc</div>",
  "<div><br> abc</div>");
pin128("ilb-wscanon-h4-degen", "<div>  []abc</div>",
  "<div><br>abc</div>");
// J 族（corpus idx 1002）——CE-false 壳邻接结构头放行右段首空格 nbsp 翻
//（exp 唯一变体 `<br>&nbsp;;`）；外段首尾空格与后续 span 面恒一。
pin128("ilb-wscanon-j-cepseudo",
  " <span contenteditable=false>A</span>[] ; <span contenteditable=false>B</span> ; <span contenteditable=false>C</span> ",
  " <span contenteditable=\"false\">A</span><br>&nbsp;; <span contenteditable=\"false\">B</span> ; <span contenteditable=\"false\">C</span> ");
// 控制键（corpus 现役绿形）——退化头 `[] abc`：右段保形不翻 nbsp。
pin128("ilb-wscanon-lead-raw-control", "<div>[] abc</div>",
  "<div><br> abc</div>");
// 控制键（corpus 现役绿形）——br 后 nbsp-first run：翻转恒等（nbsp 首原样）。
pin128("ilb-wscanon-nbsp-after-br-control",
  " <span contenteditable=false>A</span><br>[]&nbsp;; <span contenteditable=false>B</span> ; <span contenteditable=false>C</span> ",
  " <span contenteditable=\"false\">A</span><br><br>&nbsp;; <span contenteditable=\"false\">B</span> ; <span contenteditable=\"false\">C</span> ");

// 断言回路：err / ok / out 失配即 throw（execute 层 Err → Rust panic 带
// got/exp 清单）。
var bad128 = [];
for (var pi = 0; pi < globalThis.__zwP128.length; pi++) {
  var c = globalThis.__zwP128[pi];
  if (c.err || !c.ok || c.out !== c.expOut) {
    bad128.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut);
  }
}
if (bad128.length) throw "ILBWSCANONPIN MISMATCH x" + bad128.length + " | " + bad128.join(" || ");
"##,
        )
        .expect("ilb wscanon pins: all forms must hold");
}
