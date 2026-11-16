// 编辑命令兼容 ilb C+F 外迁族钉测（2026-11-17 刀，ILB-DTP-REMAP）：corpus
// insertlinebreak.html 残红 6 键（i7b0 刀前 fresh 基线 1658P/33F 键集与
// i5p1 census 恒一，C+F 族逐键在册）——dt/dd/pre 元素 caret 行断 br
// 外迁容器外。语料锚 editing/data/insertlinebreak.js idx 48/61-65 实录
// 期望（compare innerHTML 剥 marker 不锁 caret）；loose 语料形（`<dd>{}<br>`
// 隐式闭合）以归一化等形落钉——钉引擎落位语义，parser 归一面归语料门。
// marker 剥离/caret 语义照抄语料 include/tests.js setupDiv（part125 同款）。
//
// 修复面：part01 `__zw_ce_line_break_mutate` → `_zwCeParagraphMutate`
// lineBreakOnly 块 caret 重映射臂谓词扩 dt/dd/pre（共享窄表
// `_zwCeIsBlockTagName` 不动——ip 拆分边界语义不变）。spec
// implementation.js :7383 insertlinebreak：br 插 active range start
// 所在元素内（childNodes[so] 前），不外迁。
//
// 控制键 li 元素 caret（语料 idx 55 族形）＝窄表现役绿形，证谓词扩展
// 对既有臂零扰动。
#[test]
fn test_ilb_dt_dd_pre_pins() {
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
        "<html><body><div id=ce126a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐，part125 复刻）——
function nextNode126(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex126(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv126(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode126(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex126(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex126(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex126(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex126(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode126(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode126(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP126 = [];
function pin126(id, html, expOut) {
  var h = document.getElementById("ce126a");
  var err = null;
  var ok = false;
  try {
    h.innerHTML = "";
    var pts = setupDiv126(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    ok = document.execCommand("insertlinebreak", false, "");
  } catch (e126) { err = String(e126 && e126.message || e126); }
  globalThis.__zwP126.push({ id: id, ok: ok, err: err, out: h.innerHTML,
    expOut: expOut });
}

// F 族（corpus idx 48）——pre 坍缩 caret × 既有 br
pin126("ilb-dtp-pre-caret", "<pre>foo<br>{}<br></pre>",
  "<pre>foo<br><br><br></pre>");
// C 族（corpus idx 61-65）——dt/dd 元素 caret × 既有 br（嵌套 dl ×2；
// loose 语料形以归一化等形落钉）
pin126("ilb-dtp-dt-caret", "<dl><dt>{}<br></dt></dl>",
  "<dl><dt><br><br></dt></dl>");
pin126("ilb-dtp-dd-caret", "<dl><dt>foo</dt><dd>{}<br></dd></dl>",
  "<dl><dt>foo</dt><dd><br><br></dd></dl>");
pin126("ilb-dtp-dt-dd", "<dl><dt>{}<br></dt><dd>bar</dd></dl>",
  "<dl><dt><br><br></dt><dd>bar</dd></dl>");
pin126("ilb-dtp-dt-nested", "<dl><dt>foo</dt><dd>bar<dl><dt>{}<br></dt><dd>baz</dd></dl></dd></dl>",
  "<dl><dt>foo</dt><dd>bar<dl><dt><br><br></dt><dd>baz</dd></dl></dd></dl>");
pin126("ilb-dtp-dd-nested", "<dl><dt>foo</dt><dd>bar<dl><dt>baz</dt><dd>{}<br></dd></dl></dd></dl>",
  "<dl><dt>foo</dt><dd>bar<dl><dt>baz</dt><dd><br><br></dd></dl></dd></dl>");
// 控制键：li 元素 caret（corpus idx 55 族形，窄表现役绿形）——谓词扩展
// 对既有臂零扰动。
pin126("ilb-dtp-li-control", "<ol><li>{}<br></li></ol>",
  "<ol><li><br><br></li></ol>");

// 断言回路：err / ok / out 失配即 throw（execute 层 Err → Rust panic 带
// got/exp 清单）。
var bad126 = [];
for (var pi = 0; pi < globalThis.__zwP126.length; pi++) {
  var c = globalThis.__zwP126[pi];
  if (c.err || !c.ok || c.out !== c.expOut) {
    bad126.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut);
  }
}
if (bad126.length) throw "ILBDTPPIN MISMATCH x" + bad126.length + " | " + bad126.join(" || ");
"##,
        )
        .expect("ilb dt/dd/pre pins: all forms must hold");
}
