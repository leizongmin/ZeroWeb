// 编辑命令兼容 ip 残 16F C4 刀钉测（2026-11-15）：`_zwJusIsExtraneousLineBreak`
// 垫底 br 持行豁免（块级父独子臂，part01c）——容器链拆分面空块垫保全三形 +
// 控制键二形。键位对齐语料 insertparagraph.js：①:510 空 dt 拆分（splitParent
// 尾清理 `_zwJusRemoveExtraneousLineBreaksBefore` 曾误摘 `<dt><br></dt>` 垫）；
// ②:700 blockquote 尾；③:1540 ul>li>div 尾（②③隔离即绿，语料内失败属上下文
// 敏感另账）。caret/marker 剥离照抄语料 include/tests.js setupDiv。
// 修复语义锚：vendored isExtraneousLineBreak :840 height 测量（隐 br 后块塌高
// 判非冗余）——port 免布局近似 precedesLineBreak 对 br 独撑块行形过判。

#[test]
fn test_ip_c4_container_pad_pins() {
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
        "<html><body><div id=ce121a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐）——
function nextNode121(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex121(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv121(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode121(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex121(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex121(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex121(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex121(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode121(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode121(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP121 = [];
function pin121(id, html, dps, expOut, expSel) {
  var h = document.getElementById("ce121a");
  var err = null;
  var ok = false;
  var pts = null;
  try {
    pts = setupDiv121(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    document.execCommand("defaultparagraphseparator", false, dps);
    ok = document.execCommand("insertparagraph", false, "");
  } catch (e121) { err = String(e121 && e121.message || e121); }
  var sel = "?";
  try {
    var sa = getSelection().anchorNode;
    sel = (sa ? (sa.nodeName + "@" + getSelection().anchorOffset) : "null");
  } catch (e121s) {}
  globalThis.__zwP121.push({ id: id, ok: ok, err: err, sel: sel, out: h.innerHTML,
    expOut: expOut, expSel: expSel });
}
// ① 空 dt 拆分垫保全（corpus :510 dPS div：exp 选区 {} 在 dt 元素边界 → (dl,0)）
pin121("dt-pad", "<dl><dt>foo<dd>bar<dl><dt>{}<br><dd>baz</dl></dl>", "div",
  "<dl><dt>foo</dt><dd>bar</dd><dt><br></dt><dd><dl><dd>baz</dd></dl></dd></dl>", "DL@0");
// ② blockquote 尾新块（corpus :700-705 dPS div）
pin121("bq-tail", "<blockquote><p>foo[]<p>bar</blockquote>", "div",
  "<blockquote><p>foo</p><p><br></p><p>bar</p></blockquote>", "P@0");
// ③ ul>li>div 尾新 li（corpus :1540-1548 dPS p）
pin121("ul-div-tail", "<ul><li><div>foo[]</ul>", "p",
  "<ul><li><div>foo</div></li><li><div><br></div></li></ul>", "DIV@0");
// ⑦ 控制键：宿主直子 p 尾（现役绿形恒一）
pin121("ctrl-host-p", "<p>foo[]", "div", "<p>foo</p><p><br></p>", "P@0");
// ⑧ 控制键：div 包裹 p 尾（corpus :1552 现役绿形恒一）
pin121("ctrl-div-p", "<div><p>foo[]</div>", "div", "<div><p>foo</p><p><br></p></div>", "P@0");

// 断言回路：任一形 err/ok/out/sel 失配即 throw（execute 层 Err → Rust panic 带 got/exp）。
var bad121 = [];
for (var pi = 0; pi < globalThis.__zwP121.length; pi++) {
  var c = globalThis.__zwP121[pi];
  if (c.err || !c.ok || c.out !== c.expOut || c.sel !== c.expSel) {
    bad121.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut
      + " sel=" + c.sel + " expSel=" + c.expSel);
  }
}
if (bad121.length) throw "C4PIN MISMATCH x" + bad121.length + " | " + bad121.join(" || ");
"##,
        )
        .expect("ip C4 container pad pins: all five forms must hold");
}
