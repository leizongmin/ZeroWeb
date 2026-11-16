// 编辑命令兼容 ihtml 残余族钉测（2026-11-16 ihe6 刀）：corpus inserthtml.html
// census 残红 5 键（ihe5 后 iht 基线 1560P/5F）——E 空壳 ×2＋F CE-false 邻接
// ×2＋CE-false pre ×1。语料锚 editing/data/inserthtml.js :576-612 实录期望
//（compare innerHTML 剥 marker 不锁 caret，attr 引号经写读链归一——语料
// in_array/exp 形即归一后形）。marker 剥离/caret 语义照抄语料 include/tests.js
// setupDiv（part122/123/124 同款）。
//
// E 族（:584/:589）：fixture 根 div＝host 直子壳，壳内文本锚 × markup 载荷
//（含空 span 壳）——既有溶解臂对部分/坍缩选区恒 null（整命令 no-op，
// corpus got `<div>ab</div>`/`<div>ac</div>`），exp＝壳保留、载荷壳内拼接。
//
// F 族（:602/:607）：sc 在两层壳内（p 壳→b 壳→文本，壳解析仅支持一层壳
// scBase<0 → no-op，corpus got 选区文本残留+载荷零插入），exp＝选区文本
// 删除、载荷提升出 b 壳（选区侧邻接位）——语料注释「Do not delete
// non-editable when clearing the original style」域。
//
// CE-false pre（:576）：sc 在两层壳内（pre CE=false 壳→span contenteditable
// 壳→文本）× 块载荷 `<pre>abc</pre>`——块臂 CE=false 链旁路旁路、拼接臂
// scBase<0 no-op 之间，corpus got `<pre>abc</pre>` 原样入 span（未解包），
// exp＝块载荷单块解包 `abc`（pre 内块载荷解包语义同 `<pre>12[]34</pre>` 绿键）。

#[test]
fn test_ihtml_residual_pins() {
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
        "<html><body><div id=ce125a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐，part124 复用）——
function nextNode125(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex125(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv125(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode125(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex125(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex125(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex125(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex125(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode125(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode125(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP125 = [];
function pin125(id, html, pre, frag, expOut) {
  var h = document.getElementById("ce125a");
  var err = null;
  var ok = false;
  try {
    h.innerHTML = "";
    var pts = setupDiv125(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    for (var pi = 0; pi < pre.length; pi++) {
      document.execCommand(pre[pi][0], false, pre[pi][1]);
    }
    ok = document.execCommand("inserthtml", false, frag);
  } catch (e125) { err = String(e125 && e125.message || e125); }
  globalThis.__zwP125.push({ id: id, ok: ok, err: err, out: h.innerHTML,
    expOut: expOut });
}

// E 族（corpus :584/:589）——div 壳内文本锚 × markup 载荷
pin125("ihr-e-empty-span", "<div>a[]b</div>", [], "<span></span>",
  "<div>a<span></span>b</div>");
pin125("ihr-e-two-span", "<div>a[]c</div>", [],
  "<span class=\"s1\"></span>b<span class=\"s2\"></span>",
  "<div>a<span class=\"s1\"></span>b<span class=\"s2\"></span>c</div>");

// F 族（corpus :602/:607）——两层壳 × CE=false 邻接提升（exp 取 in_array 首形）
pin125("ihr-f-adj-before", "<p><b>[X]<span contenteditable=false>abc</span></b><i>def</i></p>",
  [], "<i>Z</i>",
  "<p><i>Z</i><b><span contenteditable=\"false\">abc</span></b><i>def</i></p>");
pin125("ihr-f-adj-after", "<p><b><span contenteditable=false>abc</span>[Y]</b><i>def</i></p>",
  [], "<i>Z</i>",
  "<p><b><span contenteditable=\"false\">abc</span></b><i>Z</i><i>def</i></p>");

// CE-false pre（corpus :576）——块载荷单块解包（exp 取 in_array 首形）
pin125("ihr-cef-pre", "<pre contenteditable=\"false\"><span contenteditable>[1234]</span></pre>",
  [], "<pre>abc</pre>",
  "<pre contenteditable=\"false\"><span contenteditable=\"\">abc</span></pre>");

// 断言回路：err / ok / out 失配即 throw（execute 层 Err → Rust panic 带
// got/exp 清单）。
var bad125 = [];
for (var pi = 0; pi < globalThis.__zwP125.length; pi++) {
  var c = globalThis.__zwP125[pi];
  if (c.err || !c.ok || c.out !== c.expOut) {
    bad125.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut);
  }
}
if (bad125.length) throw "IHRPIN MISMATCH x" + bad125.length + " | " + bad125.join(" || ");
"##,
        )
        .expect("ihtml residual pins: all forms must hold");
}
