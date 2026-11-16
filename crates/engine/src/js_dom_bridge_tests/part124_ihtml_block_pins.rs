// 编辑命令兼容 ihtml 轴一块级拆分插入刀钉测（2026-11-16）：inserthtml 载荷含
// 块级/列表/表格/hr 根（或同名行内 a/nobr 拆段形）时在插入点按上下文拆分当前
// 容器插入（part06 `_zwExecCmdApplyInsertHtmlBlock`，host 级 innerHTML 字符串
// 手术；不合形返回 false 落回既有内联拼接臂零改动）。语料锚：editing/data/
// inserthtml.js 轴一族实录期望（compare innerHTML 剥 selection marker 不锁
// caret——expSel 恒 null，块级族语料 compare 不含 {} / [] marker）。覆盖：
// p 劈裂（dPS div/p）、li 降级（裸 payload `<li>abc` 未闭合补全路径）、list
// 左/右元素锚劈裂（右形须重开 list——corpus :169 实录 `</ol><p>…</p><ol>`）、
// li 内嵌 p、li 拆段、列表载荷嵌套、dir 族、dt/dd dl 劈裂、bare li 降级/
// dt·dd dl 包裹/td 溶解、td 上下文溶解、hr 入 p/b、h2 劈 h1、a/nobr 同名拆段
//（含 `<a href=/>` 未引号属性值尾 `/` 非自闭合判据）、smallcaps 非携带壳劈
// 裂、空 li 整体换载（id 形）、携带族 override carry 四形（font/span color 壳
// × swcss true/false——2026-11-16 携带族刀，swcss true → span style rgb 形、
// false → font color hex 形）、CE=false pre 链门旁路。控制三形现役
// 绿路径恒一（纯文本/行内 b 载荷走既有内联臂、xmp rawtext 壳走轴二壳臂）。
// 注：td 表格形语料期望含 tbody——bridge parser 已合成（该键可达语料真值）。
// marker 剥离/caret 语义照抄语料 include/tests.js setupDiv（part122/part123
// 同款）。

#[test]
fn test_ihtml_block_pins() {
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
        "<html><body><div id=ce124a contenteditable></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            r##"
// —— tests.js setupDiv 同款（marker 语义逐行对齐，part123 复用）——
function nextNode124(node) {
  if (node.firstChild) return node.firstChild;
  while (node) {
    if (node.nextSibling) return node.nextSibling;
    node = node.parentNode;
  }
  return null;
}
function getNodeIndex124(node) {
  var i = 0;
  while ((node = node.previousSibling)) i++;
  return i;
}
function setupDiv124(node, html) {
  node.innerHTML = html;
  var startNode = null, startOffset = 0, endNode = null, endOffset = 0;
  var cur = node;
  while (cur) {
    if (cur.nodeType != 3) { cur = nextNode124(cur); continue; }
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
    if (elStartIdx == 0) { startNode = cur.parentNode; startOffset = getNodeIndex124(cur); }
    else if (elStartIdx != -1) { startNode = cur.parentNode; startOffset = getNodeIndex124(cur) + 1; }
    if (elEndIdx == 0) { endNode = cur.parentNode; endOffset = getNodeIndex124(cur); }
    else if (elEndIdx != -1) { endNode = cur.parentNode; endOffset = getNodeIndex124(cur) + 1; }
    cur.data = cur.data.replace(/[{}]/g, "");
    if (!cur.data.length) {
      if (cur == startNode || cur == endNode) throw "bracket w/o text node";
      var oldCur = cur;
      cur = nextNode124(cur);
      oldCur.parentNode.removeChild(oldCur);
    } else {
      cur = nextNode124(cur);
    }
  }
  return [startNode, startOffset, endNode, endOffset];
}

globalThis.__zwP124 = [];
// expSel 恒 null：块级族语料 compare 仅核 innerHTML（runner 剥 marker 不锁
// caret）；swcss/dps 前置命令按语料前置序执行。
function pin124(id, html, pre, frag, expOut) {
  var h = document.getElementById("ce124a");
  var err = null;
  var ok = false;
  try {
    h.innerHTML = "";
    var pts = setupDiv124(h, html);
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
  } catch (e124) { err = String(e124 && e124.message || e124); }
  globalThis.__zwP124.push({ id: id, ok: ok, err: err, out: h.innerHTML,
    expOut: expOut });
}
var DV = ["defaultparagraphseparator", "div"];
var DP = ["defaultparagraphseparator", "p"];
// ① p 劈裂 dPS div/p（corpus :139/:144）
pin124("ihb-p-split-div", "<p>foo[bar]baz", [DV], "<p>abc",
  "<p>foo</p><p>abc</p><p>baz</p>");
pin124("ihb-p-split-p", "<p>foo[bar]baz", [DP], "<p>abc",
  "<p>foo</p><p>abc</p><p>baz</p>");
// ② li 裸 payload 未闭合补全 + 降级 dPS div/p（corpus :149/:154）
pin124("ihb-li-demote-div", "<p>foo[bar]baz", [DV], "<li>abc",
  "<p>foo</p><div>abc</div><p>baz</p>");
pin124("ihb-li-demote-p", "<p>foo[bar]baz", [DP], "<li>abc",
  "<p>foo</p><p>abc</p><p>baz</p>");
// ③ list 左元素锚劈裂（corpus :159——载荷前置、余 li 留旧壳）
pin124("ihb-list-left", "<ol>{<li>foo</li>}<li>bar</ol>", [DV], "<p>abc",
  "<p>abc</p><ol><li>bar</li></ol>");
// ④ list 右元素锚劈裂（corpus :169——旧壳闭合、载荷、list 重开持余 li）
pin124("ihb-list-right", "<ol><li>foo</li>{<li>bar</li>}<li>baz</ol>", [DV], "<p>abc",
  "<ol><li>foo</li></ol><p>abc</p><ol><li>baz</li></ol>");
// ⑤ li 内 p 嵌套（文本锚非劈裂，corpus :179）
pin124("ihb-list-nest-p", "<ol><li>[foo]</li><li>bar</ol>", [DV], "<p>abc",
  "<ol><li><p>abc</p></li><li>bar</li></ol>");
// ⑥ li 拆段（文本锚 li 内，corpus :313 同族 ol 形）
pin124("ihb-li-split", "<ol><li>f[o]o</ol>", [], "<li>abc</li>",
  "<ol><li>f</li><li>abc</li><li>o</li></ol>");
pin124("ihb-dir-li-split", "<dir><li>f[o]o</dir>", [], "<li>abc</li>",
  "<dir><li>f</li><li>abc</li><li>o</li></dir>");
// ⑦ 列表载荷嵌套 li 内（corpus :343——payload 未闭合内 li 由 parser 归位）
pin124("ihb-list-nest-payload", "<ul><li>f[o]o</ul>", [], "<ol><li>abc</ol>",
  "<ul><li>f<ol><li>abc</li></ol>o</li></ul>");
// ⑧ dt/dd dl 劈裂（corpus :265/:275——dd 项保持原位）
pin124("ihb-dt-split", "<dl><dt>f[o]o<dd>bar</dl>", [], "<dt>abc</dt>",
  "<dl><dt>f</dt><dt>abc</dt><dt>o</dt><dd>bar</dd></dl>");
pin124("ihb-dd-split", "<dl><dt>f[o]o<dd>bar</dl>", [], "<dd>abc</dd>",
  "<dl><dt>f</dt><dd>abc</dd><dt>o</dt><dd>bar</dd></dl>");
// ⑨ bare li 降级 dPS div/p（corpus :383/:388）
pin124("ihb-bare-li-div", "f[o]o", [DV], "<li>abc</li>", "f<div>abc</div>o");
pin124("ihb-bare-li-p", "f[o]o", [DP], "<li>abc</li>", "f<p>abc</p>o");
// ⑩ bare dt/dd dl 包裹（corpus :283/:293）
pin124("ihb-bare-dt", "f[o]o", [], "<dt>abc</dt>", "f<dl><dt>abc</dt></dl>o");
pin124("ihb-bare-dd", "f[o]o", [], "<dd>abc</dd>", "f<dl><dd>abc</dd></dl>o");
// ⑪ bare td 溶解（corpus :258）
pin124("ihb-bare-td", "f[o]o", [], "<td>abc</td>", "fabco");
// ⑫ td 上下文溶解（corpus table 形——bridge parser 合成 tbody，语料期望
// 形可达）
pin124("ihb-td-dissolve", "<table><tr><td>f[o]o</table>", [], "<td>abc</td>",
  "<table><tbody><tr><td>fabco</td></tr></tbody></table>");
// ⑬ hr 入 p（corpus）与入 b（swT，corpus）
pin124("ihb-hr-p", "<p>f[o]o", [DV], "<hr>", "<p>f</p><hr><p>o</p>");
pin124("ihb-hr-b", "<b>f[o]o</b>", [["stylewithcss", "true"]], "<hr>",
  "<b>f</b><hr><b>o</b>");
// ⑭ h2 劈 h1（corpus）
pin124("ihb-h2-h1", "<h1>f[o]o</h1>", [], "<h2>abc</h2>",
  "<h1>f</h1><h2>abc</h2><h1>o</h1>");
// ⑮ a 同名拆段（裸 + 属性形 `<a href=/>` 非自闭合判据，corpus）
pin124("ihb-a-split", "<a>f[o]o</a>", [], "<a>abc</a>",
  "<a>f</a><a>abc</a><a>o</a>");
pin124("ihb-a-attr-split", "<a href=.>f[o]o</a>", [], "<a href=/>abc</a>",
  "<a href=\".\">f</a><a href=\"/\">abc</a><a href=\".\">o</a>");
// ⑯ nobr 同名拆段（corpus）
pin124("ihb-nobr-split", "<nobr>f[o]o</nobr>", [], "<nobr>abc</nobr>",
  "<nobr>f</nobr><nobr>abc</nobr><nobr>o</nobr>");
// ⑰ smallcaps 非携带壳劈裂（corpus——font-variant 非携带样式放行）
pin124("ihb-smallcaps-split", "<span style=font-variant:small-caps>foo[]bar</span>",
  [["stylewithcss", "true"], DV], "<p>abc",
  "<span style=\"font-variant:small-caps\">foo</span><p>abc</p><span style=\"font-variant:small-caps\">bar</span>");
// ⑱ 空 li 整体换载（id 形，corpus——旧 list 壳与 id 保留、li0 换载荷子项）
pin124("ihb-empty-li-replace", "<ol id=\"old\"><li id=\"li0\">{}<br></ol>", [],
  "<ol id=\"new\"><li id=\"li1\">abc</li><li id=\"li2\">def</li></ol>",
  "<ol id=\"old\"><li id=\"li1\">abc</li><li id=\"li2\">def</li></ol>");
// ⑲ 携带族 override carry（2026-11-16 携带族刀——corpus inserthtml 8 键实录
// 形：swcss true → span style rgb 形、false → font color hex 形，载荷包壳在
// 块元素内）；CE=false pre 块臂旁路（tTag pre 非劈裂域）落既有轴二 pre 壳臂
// 现役 got——语料该键仍红，钉测锁现役行为防漂移。
pin124("ihb-carry-font-false", "<font color=blue>foo[]bar</font>", [DV], "<p>abc",
  "<font color=\"blue\">foo</font><p><font color=\"#0000ff\">abc</font></p><font color=\"blue\">bar</font>");
pin124("ihb-carry-span-false", "<span style=color:blue>foo[]bar</span>", [DP], "<p>abc",
  "<span style=\"color:blue\">foo</span><p><font color=\"#0000ff\">abc</font></p><span style=\"color:blue\">bar</span>");
pin124("ihb-carry-font-true", "<font color=blue>foo[]bar</font>",
  [["stylewithcss", "true"], DV], "<p>abc",
  "<font color=\"blue\">foo</font><p><span style=\"color:rgb(0, 0, 255)\">abc</span></p><font color=\"blue\">bar</font>");
pin124("ihb-carry-span-true", "<span style=color:blue>foo[]bar</span>",
  [["stylewithcss", "true"], DP], "<p>abc",
  "<span style=\"color:blue\">foo</span><p><span style=\"color:rgb(0, 0, 255)\">abc</span></p><span style=\"color:blue\">bar</span>");
pin124("ihb-bypass-ce-pre", "<pre contenteditable=\"false\"><span contenteditable>[1234]</span></pre>",
  [], "<pre>abc</pre>",
  "<pre contenteditable=\"false\"><span contenteditable=\"\"><pre>abc</pre></span></pre>");
// ㉑ 轴二元素端点臂（2026-11-16 轴二壳界选区刀——corpus inserthtml 8 键实录：
// `{}` marker 落壳边界产生元素端点 range（elStartIdx==0 → startNode=parentNode），
// 既有内联臂 sc.nodeType===3 入口门整臂旁路 no-op）。元素端点归一宿主子文本
// 空间后走溶解臂：全含选区壳溶解、载荷落 ws——期望值不含色壳（选区删除把
// 壳一并溶解，与色 carry 无关）。swcss 双态期望同形。
pin124("ihb-elm-in-span", "foo<span style=color:#aBcDeF>{bar}</span>baz",
  [["stylewithcss", "true"]], "ab<b>c</b>d", "fooab<b>c</b>dbaz");
pin124("ihb-elm-around-span", "foo{<span style=color:#aBcDeF>bar</span>}baz",
  [["stylewithcss", "true"]], "ab<b>c</b>d", "fooab<b>c</b>dbaz");
pin124("ihb-elm-cross-left", "{foo<span style=color:#aBcDeF>bar}</span>baz",
  [["stylewithcss", "true"]], "ab<b>c</b>d", "ab<b>c</b>dbaz");
pin124("ihb-elm-cross-right", "foo<span style=color:#aBcDeF>{bar</span>baz}",
  [["stylewithcss", "true"]], "ab<b>c</b>d", "fooab<b>c</b>d");
pin124("ihb-elm-in-span-false", "foo<span style=color:#aBcDeF>{bar}</span>baz",
  [["stylewithcss", "false"]], "ab<b>c</b>d", "fooab<b>c</b>dbaz");
pin124("ihb-elm-around-span-false", "foo{<span style=color:#aBcDeF>bar</span>}baz",
  [["stylewithcss", "false"]], "ab<b>c</b>d", "fooab<b>c</b>dbaz");
pin124("ihb-elm-cross-left-false", "{foo<span style=color:#aBcDeF>bar}</span>baz",
  [["stylewithcss", "false"]], "ab<b>c</b>d", "ab<b>c</b>dbaz");
pin124("ihb-elm-cross-right-false", "foo<span style=color:#aBcDeF>{bar</span>baz}",
  [["stylewithcss", "false"]], "ab<b>c</b>d", "fooab<b>c</b>d");
// ⑳ 控制三形现役绿路径恒一（块臂返 false 落既有臂）
pin124("ihb-ctrl-text", "foo[bar]baz", [], "abc", "fooabcbaz");
pin124("ihb-ctrl-inline-b", "foo[bar]baz", [], "<b>abc</b>", "foo<b>abc</b>baz");
pin124("ihb-ctrl-xmp", "<xmp>f[o]o</xmp>", [], "abc", "<xmp>fabco</xmp>");

// 断言回路：err / ok / out 失配即 throw（execute 层 Err → Rust panic 带
// got/exp 清单）。
var bad124 = [];
for (var pi = 0; pi < globalThis.__zwP124.length; pi++) {
  var c = globalThis.__zwP124[pi];
  if (c.err || !c.ok || c.out !== c.expOut) {
    bad124.push(c.id + ": err=" + c.err + " ok=" + c.ok
      + " got=" + c.out + " exp=" + c.expOut);
  }
}
if (bad124.length) throw "IHBPIN MISMATCH x" + bad124.length + " | " + bad124.join(" || ");
"##,
        )
        .expect("ihtml block split pins: all forms must hold");
}
