// 编辑命令兼容 ip 残 census C5 刀钉测（2026-11-15）：宿主唯一子 div 选区删净
// break-out——消解臂扩持行垫 br 免视（part01h `__zw_ce_dissolve_emptied_div_tail`
// 双 CharData 界放行 + ipara 续走尾随归一）→ ① 臂 A2 裸 br 垫 + sepTag 新块
// 建其后。语料锚：insertparagraph.js :2102/:2107 `<div>[abc]</div>` dPS div/p
// exp `<br><div><br></div>`/`<br><p><br></p>`（语料注释「div 应解包、<br> 置
// 于其位、新段建于其后」，after 串无 selection marker 不锁 caret——spec 参考
// 实现该面产双空块就地拆形＝Chrome 录制 delta）。控制键二形现役绿形恒一：
// `<p>[abc]</p>` 标签 p 不解包（:2117/:2124）；宿主多块
// `<div>abc</div><div>[def]</div>` 不解包（:2132/:2142）。caret/marker 剥离照
// 抄语料 include/tests.js setupDiv（part121 同款）。

#[test]
fn test_ip_c5_root_bare_br_pins() {
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
        "<html><body><div id=ce122a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐，part121 复用）——
function nextNode122(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex122(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv122(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode122(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex122(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex122(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex122(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex122(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode122(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode122(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP122 = [];
// expSel 传 null 表语料未锁 caret（:2102/:2107 after 串无 selection marker，
// compare 仅核 innerHTML）——C5 双面不断 caret（引擎落 (host,0)，Chrome 实录
// 未记，不造约束）；控制键面（:2117 族就地拆）语料注释明言 caret 入新段，
// 沿现役行为锁定。
function pin122(id, html, dps, expOut, expSel) {
  var h = document.getElementById("ce122a");
  var err = null;
  var ok = false;
  var pts = null;
  try {
    pts = setupDiv122(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    document.execCommand("defaultparagraphseparator", false, dps);
    ok = document.execCommand("insertparagraph", false, "");
  } catch (e122) { err = String(e122 && e122.message || e122); }
  var sel = "?";
  try {
    var sa = getSelection().anchorNode;
    sel = (sa ? (sa.nodeName + "@" + getSelection().anchorOffset) : "null");
  } catch (e122s) {}
  globalThis.__zwP122.push({ id: id, ok: ok, err: err, sel: sel, out: h.innerHTML,
    expOut: expOut, expSel: expSel });
}
// ① C5 break-out 本刀：宿主唯一子 div 全选删净 → 解包（corpus :2102 dPS div）
pin122("c5-div", "<div>[abc]</div>", "div",
  "<br><div><br></div>", null);
// ② 同面 separator p（corpus :2107 dPS p：新块随 sepTag）
pin122("c5-p-sep", "<div>[abc]</div>", "p",
  "<br><p><br></p>", null);
// ③ 控制键：标签 p 不解包（corpus :2117 dPS div → ② 就地拆现役绿形）
pin122("ctrl-p-tag", "<p>[abc]</p>", "div",
  "<p><br></p><p><br></p>", "P@0");
// ④ 控制键：宿主多块不解包（corpus :2132 dPS div → 现役绿形恒一）
pin122("ctrl-multiblock", "<div>abc</div><div>[def]</div>", "div",
  "<div>abc</div><div><br></div><div><br></div>", "DIV@0");

// 断言回路：err/ok/out 失配即 throw；expSel 非 null 时才核 caret（execute 层
// Err → Rust panic 带 got/exp）。
var bad122 = [];
for (var pi = 0; pi < globalThis.__zwP122.length; pi++) {
  var c = globalThis.__zwP122[pi];
  if (c.err || !c.ok || c.out !== c.expOut
      || (c.expSel !== null && c.sel !== c.expSel)) {
    bad122.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut
      + " sel=" + c.sel + " expSel=" + c.expSel);
  }
}
if (bad122.length) throw "C5PIN MISMATCH x" + bad122.length + " | " + bad122.join(" || ");
"##,
        )
        .expect("ip C5 root bare br pins: all four forms must hold");
}
