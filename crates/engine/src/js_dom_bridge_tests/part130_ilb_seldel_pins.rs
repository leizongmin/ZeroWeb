// 编辑命令兼容 ilb B+G+E 选区删除域钉测（2026-11-17 刀⑤，ILB-SELDEL）：
// corpus insertlinebreak.html 残红 11 键（i7f2 刀④后基线 1678P/13F 在册，
// census ⑤族）三子机制——(1) 同节点元素容器选区下钻（`{bar}`→(span,0)-
// (span,1)／`{<span>bar</span>}`→(p,1)-(p,2) 形：Chrome 元素端点选区按壳内
// 内容语义删，壳留内容删，br 落壳内 `<span><br>{}</span>`）；(2) 后删元素
// caret 入壳（前导删完 caret 塌进 inline 壳 (span,0)：br 插壳内非 (父,sc)
// 外迁）＋块界补断细化（br 后随块界元素 table/p 补第二 br，替换型 br 仍不
// 垫）；(3) rawtext no-op 守卫（spec :7396-7409 br 非合法子 return true，
// xmp/script 零变更）。控制键：text 容器切片现役绿形、跨节点前导现役绿形、
// br-后随既有 br 不垫现役绿形、块尾补断现役绿形。exp 锁引擎序列化形
//（attr 双引号、color attr 原样——harness 比对面已做 inline-style 归一）。
#[test]
fn test_ilb_seldel_pins() {
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
        "<html><body><div id=ce130a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐，part129 复刻）——
function nextNode130(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex130(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv130(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode130(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex130(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex130(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex130(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex130(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode130(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode130(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP130 = [];
function pin130(id, html, expOut, swcss) {
  var h = document.getElementById("ce130a");
  var err = null;
  var ok = false;
  try {
    h.innerHTML = "";
    if (swcss) document.execCommand("stylewithcss", false, "true");
    var pts = setupDiv130(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    ok = document.execCommand("insertlinebreak", false, "");
    if (swcss) document.execCommand("stylewithcss", false, "false");
  } catch (e130) { err = String(e130 && e130.message || e130); }
  globalThis.__zwP130.push({ id: id, ok: ok, err: err, out: h.innerHTML,
    expOut: expOut });
}

// B 族 6 键（corpus idx 543/548/573/578/588/583）——同节点元素容器下钻＋
// 后删元素 caret 入壳＋块界/壳内补断。
pin130("ilb-seldel-span-textsel", "<p>foo<span style=color:#aBcDeF>{bar}</span>baz",
  "<p>foo<span style=\"color:#aBcDeF\"><br></span>baz</p>", false);
pin130("ilb-seldel-p-elem", "<p>foo{<span style=color:#aBcDeF>bar</span>}baz",
  "<p>foo<span style=\"color:#aBcDeF\"><br></span>baz</p>", false);
pin130("ilb-seldel-textstart", "<p>foo<span style=color:#aBcDeF>[bar</span>baz]",
  "<p>foo<span style=\"color:#aBcDeF\"><br><br></span></p>", false);
pin130("ilb-seldel-elemstart", "<p>foo<span style=color:#aBcDeF>{bar</span>baz}",
  "<p>foo<span style=\"color:#aBcDeF\"><br><br></span></p>", false);
pin130("ilb-seldel-twocolor", "<p>foo<span style=color:#aBcDeF>[bar</span><span style=color:#fEdCbA>baz]</span>quz",
  "<p>foo<span style=\"color:#aBcDeF\"><br></span>quz</p>", false);
pin130("ilb-seldel-twocolor-swcss", "<p>foo<span style=color:#aBcDeF>[bar</span><span style=color:#fEdCbA>baz]</span>quz",
  "<p>foo<span style=\"color:#aBcDeF\"><br></span>quz</p>", true);
// G/E 族（corpus idx 378/33/8）——host 级元素容器前导整元素摘除＋块界补断。
pin130("ilb-seldel-h1-elem", "<p>foo</p>{<h1>bar</h1>}<p>baz</p>",
  "<p>foo</p><br><br><p>baz</p>", false);
pin130("ilb-seldel-table-elem", "{<table><tr><td>foo</table>}", "<br><br>", false);
pin130("ilb-seldel-cross-table", "fo[o<table><tr><td>b]ar</table>",
  "fo<br><br><table><tbody><tr><td>ar</td></tr></tbody></table>", false);
// D 族（corpus rawtext 2 键）——rawtext no-op 守卫。
pin130("ilb-seldel-xmp", "<xmp>foo[]bar</xmp>", "<xmp>foobar</xmp>", false);
pin130("ilb-seldel-script", "<script>foo[]bar</script>baz", "<script>foobar</script>baz", false);
// 控制键（现役绿形恒一）。
pin130("ilb-seldel-ctl-text-caret", "<p>foo<span style=color:#aBcDeF>[bar]</span>baz",
  "<p>foo<span style=\"color:#aBcDeF\"><br></span>baz</p>", false);
pin130("ilb-seldel-ctl-cross-node", "<p>[foo<span style=color:#aBcDeF>bar]</span>baz",
  "<p><br>baz</p>", false);
pin130("ilb-seldel-ctl-br-then-br", "<p>foo</p>{}<br><h1>bar</h1>",
  "<p>foo</p><br><br><h1>bar</h1>", false);
pin130("ilb-seldel-ctl-block-tail", "<div>foo[]</div>", "<div>foo<br><br></div>", false);
pin130("ilb-seldel-ctl-h1-br", "<h1>foo[]<br></h1>", "<h1>foo<br><br></h1>", false);

// 断言回路：err / ok / out 失配即 throw（execute 层 Err → Rust expect 带
// got/exp 清单）。
var bad130 = [];
for (var pi = 0; pi < globalThis.__zwP130.length; pi++) {
  var c = globalThis.__zwP130[pi];
  if (c.err || !c.ok || c.out !== c.expOut) {
    bad130.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut);
  }
}
if (bad130.length) throw "ILBSELDELPIN MISMATCH x" + bad130.length + " | " + bad130.join(" || ");
"##,
        )
        .expect("ilb seldel pins: all forms must hold");
}
