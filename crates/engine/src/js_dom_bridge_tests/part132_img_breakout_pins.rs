// 编辑命令兼容 insertimage 壳形域钉测（2026-11-17 img 出壳刀，IMG-BREAKOUT/
// IMG-SWCSS-GUARD）：corpus insertimage.html 残红 23 键（ilb 全清后基线在册）
// 两机制——R1 出壳落位（21 壳形：折叠插入点提离 inline 壳链，腰点右半壳分裂，
// 插入后邻接空 inline 壳摘除；stripWrappers:false 留壳＝Chrome 出壳口径反差
// 面）＋R2 跨块值载体 swcss 保真（img apply 的 deleteSelection 守卫包 InnerDef
// 见 part01g，_zwSync F1'-B 清除跳过 → restoreValues 跨块载体 font/span 按模
// 式选形）。exp 全部取自 wpt-data editing/data/insertimage.js 语料录形；控制
// 键＝现役绿形（块层插入点、sole-br 步、根层直插）恒一锁改动不翻转。
#[test]
fn test_img_breakout_pins() {
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
        "<html><body><div id=ce132a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（bracket 走域复刻 part131 同款，换名防撞）——
function nextNode132(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex132(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv132(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode132(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex132(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex132(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex132(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex132(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode132(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode132(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP132 = [];
// pin132(id, html, expOut [, swcss])——swcss true 时在选区 setup 后、
// insertimage 前置 stylewithcss true（语料命令表序：setup 选区变更先清、
// 命令表再置位）。断言面＝innerHTML 字节相等（载体形 10 号钉为结构断言，
// 见钉注）。
function pin132(id, html, expOut, swcss) {
  var h = document.getElementById("ce132a");
  var err = null;
  var ok = false;
  try {
    h.innerHTML = "";
    var pts = setupDiv132(h, html);
    var r = document.createRange();
    r.setStart(pts[0], pts[1]);
    r.setEnd(pts[2], pts[3]);
    if (r.collapsed) r.setEnd(pts[0], pts[1]);
    getSelection().removeAllRanges();
    getSelection().addRange(r);
    if (swcss) document.execCommand("stylewithcss", false, "true");
    ok = document.execCommand("insertimage", false, "/img/lion.svg");
  } catch (e132) { err = String(e132 && e132.message || e132); }
  globalThis.__zwP132.push({ id: id, ok: ok, err: err, out: h.innerHTML,
    expOut: expOut, structural: !!swcss });
}

// R1 族——壳内落位（got 形＝corpus 23F got 侧）→ exp 语料录形。
// [9]/[10]：caret (b,0) 空壳内，出壳偏移 0 提界＋空壳摘。
pin132("img-bo-caret-in-empty-shell",
  "foo<b>[bar]</b>baz", "foo<img src=\"/img/lion.svg\">baz");
// [22]：腰点右半壳分裂（b）。
pin132("img-bo-mid-shell-split-b",
  "<div><b>A[]B</b></div>",
  "<div><b>A</b><img src=\"/img/lion.svg\"><b>B</b></div>");
// [23]：腰点右半壳分裂（span）。
pin132("img-bo-mid-shell-split-span",
  "<div><span>A[]B</span></div>",
  "<div><span>A</span><img src=\"/img/lion.svg\"><span>B</span></div>");
// [1]：文本尾边界提壳界，双壳保留零摘。
pin132("img-bo-boundary-between-shells",
  "<span>foo[</span><span>]bar</span>",
  "<span>foo</span><img src=\"/img/lion.svg\"><span>bar</span>");
// [20]：空壳含 br 非空保留（语料 accept 数组第二形）。
pin132("img-bo-empty-br-shell-kept",
  "<div><b>{}<br></b></div>",
  "<div><img src=\"/img/lion.svg\"><b><br></b></div>");
// [21]：span 变体同形。
pin132("img-bo-empty-br-shell-kept-span",
  "<div><span>{}<br></span></div>",
  "<div><img src=\"/img/lion.svg\"><span><br></span></div>");
// [11]：块层 caret（整壳删空 restoreValues 不重发裸 b）前邻空壳摘。
pin132("img-bo-whole-shell-prune",
  "foo{<b>bar</b>}baz", "foo<img src=\"/img/lion.svg\">baz");
// [4]：样式壳删空重发形，空壳摘（无 style 属性残留）。
pin132("img-bo-styled-empty-prune",
  "foo{<span style=color:#aBcDeF>bar</span>}baz",
  "foo<img src=\"/img/lion.svg\">baz");
// [15]：跨壳界（b 尾→i 头）落壳间。
pin132("img-bo-cross-bi-boundary",
  "<b>foo[bar</b><i>baz]quz</i>",
  "<b>foo</b><img src=\"/img/lion.svg\"><i>quz</i>");
// [7]/[8]：双空样式壳链逐个摘（swcss 两态同 exp，单态钉）。
pin132("img-bo-double-empty-span-prune",
  "foo<span style=color:#aBcDeF>[bar</span><span style=color:#fEdCbA>baz]</span>quz",
  "foo<img src=\"/img/lion.svg\">quz");
// [12]：裸 span 空壳摘。
pin132("img-bo-plain-span-prune",
  "foo<span>[bar]</span>baz", "foo<img src=\"/img/lion.svg\">baz");

// R2 族——跨块值载体（font/span 按模式选形）＋img 落载体壳前块层。
// [17]/[19]：swcss false → font 载体（字节日节形可字节断言）。
pin132("img-bo-cross-block-font-carry",
  "<p>foo[bar<p style=color:blue>baz]quz",
  "<p>foo<img src=\"/img/lion.svg\"><font color=\"#0000ff\">quz</font></p>");
// [16]/[18]：swcss true → span 载体。裸 innerHTML 的 color 串形随写入路径
// （hex/rgb）波动，字节断言改结构断言：载体必 span 非 font、img 在载体前、
// quz 在载体内（字节级恒一由语料腿 normalize 口径承载）。
pin132("img-bo-cross-block-span-carry",
  "<p>foo[bar<p style=color:blue>baz]quz",
  "__STRUCTURAL__", true);

// 控制键（现役绿形恒一）。
// foo[]bar 根层直插。
pin132("img-ctl-root",
  "foo[]bar", "foo<img src=\"/img/lion.svg\">bar");
// 块层壳间插入点恒等（出壳臂 no-op）。
pin132("img-ctl-between-spans",
  "<span>foo</span>{}<span>bar</span>",
  "<span>foo</span><img src=\"/img/lion.svg\"><span>bar</span>");
// sole-br 步（块独子 br 摘除）绿形恒一。
pin132("img-ctl-div-sole-br",
  "<div>{}<br></div>", "<div><img src=\"/img/lion.svg\"></div>");

// 断言回路：err / ok / out 失配即 throw（execute 层 Err → Rust expect 带
// got/exp 清单）；结构钉单独判。
var bad132 = [];
for (var pi = 0; pi < globalThis.__zwP132.length; pi++) {
  var c = globalThis.__zwP132[pi];
  var bad = false;
  if (c.err || !c.ok) {
    bad = true;
  } else if (c.structural) {
    bad = !(c.out.indexOf('<font') === -1
      && c.out.indexOf('<span') >= 0
      && c.out.indexOf('<img src="/img/lion.svg">') >= 0
      && c.out.indexOf('<img src="/img/lion.svg">') < c.out.indexOf('<span')
      && c.out.indexOf('quz') >= 0);
  } else {
    bad = c.out !== c.expOut;
  }
  if (bad) {
    bad132.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut);
  }
}
if (bad132.length) throw "IMGBOPIN MISMATCH x" + bad132.length + " | " + bad132.join(" || ");
"##,
        )
        .expect("img breakout pins: all forms must hold");
}
