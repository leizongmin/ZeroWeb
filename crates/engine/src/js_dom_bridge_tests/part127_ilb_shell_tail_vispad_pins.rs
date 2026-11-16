// 编辑命令兼容 ilb A 壳尾计数族钉测（2026-11-17 刀，ILB-VISPAD-V2）：corpus
// insertlinebreak.html 残红 5 键（i7c0 刀①后基线 1664P/27F 在册，A 族逐键
// 在 census 十族分类）——inline 壳尾行断 br 误双垫＋注释邻接反形漏垫。
// 语料锚 editing/data/insertlinebreak.js idx 458/468/518/528/538 实录期望
//（compare innerHTML 剥 marker 不锁 caret）；控制键 p 块尾双垫（corpus idx
// 203 现役绿形＝补断路径恒一）与 p 注释邻接块起点（corpus idx 523 现役绿形
// ＝前向扫描注释跳过）证扫描重写对既有臂零扰动。
//
// 修复面：part01 `_zwCeLineBreakAt` 可见性补断 v2——br 型按文档序块界扫描
//（_zwCeBrVisibleAfter：非折叠空白文本可见、注释不可见、元素含可见后代
// 可见、空子树仅替换型元素可见；inline 父链上爬至块界）定补断。spec
// implementation.js :795 isCollapsedLineBreak（zwsp 布局探针）结构近似；
// :7459 "If br is a collapsed line break" 补第二条。wsSig（\n 型）维持旧
// 同父兄弟规则（H+J 归一域另案）。
#[test]
fn test_ilb_shell_tail_vispad_pins() {
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
        "<html><body><div id=ce127a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐，part126 复刻）——
function nextNode127(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex127(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv127(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode127(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex127(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex127(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex127(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex127(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode127(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode127(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP127 = [];
function pin127(id, html, expOut) {
  var h = document.getElementById("ce127a");
  var err = null;
  var ok = false;
  try {
    h.innerHTML = "";
    var pts = setupDiv127(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    ok = document.execCommand("insertlinebreak", false, "");
  } catch (e127) { err = String(e127 && e127.message || e127); }
  globalThis.__zwP127.push({ id: id, ok: ok, err: err, out: h.innerHTML,
    expOut: expOut });
}

// A 族（corpus idx 458/468/518）——inline 壳尾 × 块内可见后续：单 br 不双垫。
pin127("ilb-vispad-b-tail-text", "<b>foo[]</b>bar",
  "<b>foo<br></b>bar");
pin127("ilb-vispad-b-tail-shell", "<b>foo[]</b><i>bar</i>",
  "<b>foo<br></b><i>bar</i>");
pin127("ilb-vispad-a-tail-text", "<a href=foo>foo[]</a>bar",
  "<a href=\"foo\">foo<br></a>bar");
// A 族（corpus idx 528）——注释邻接反形：注释不可见 → 补第二条（双垫）。
pin127("ilb-vispad-comment-adjacent", "<p>foo[]<!--bar-->",
  "<p>foo<br><br><!--bar--></p>");
// A 族（corpus idx 538）——非塌缩选区壳内删后壳尾：删选区＋单 br。style
// attr 锁引擎原始形（hex 直传；语料 rgb(171, 205, 239) 归一在 harness
// 比对面 "after normalizing inline style"，此处钉引擎序列化语义）。
pin127("ilb-vispad-sel-shell-tail", "<p>foo<span style=color:#aBcDeF>[bar]</span>baz",
  "<p>foo<span style=\"color:#aBcDeF\"><br></span>baz</p>");
// 控制键（corpus idx 203 现役绿形）——块尾文本：无可见后续 → 双垫恒一。
pin127("ilb-vispad-block-tail-pad", "<p>foo[]</p>",
  "<p>foo<br><br></p>");
// 控制键（corpus idx 523 现役绿形）——注释邻接块起点：br 前注释不入扫描，
// br 后可见文本 → 单 br 不补。
pin127("ilb-vispad-comment-before-caret", "<p><!--foo-->[]bar",
  "<p><!--foo--><br>bar</p>");

// 断言回路：err / ok / out 失配即 throw（execute 层 Err → Rust panic 带
// got/exp 清单）。
var bad127 = [];
for (var pi = 0; pi < globalThis.__zwP127.length; pi++) {
  var c = globalThis.__zwP127[pi];
  if (c.err || !c.ok || c.out !== c.expOut) {
    bad127.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut);
  }
}
if (bad127.length) throw "ILBVISPADPIN MISMATCH x" + bad127.length + " | " + bad127.join(" || ");
"##,
        )
        .expect("ilb shell-tail vispad pins: all forms must hold");
}
