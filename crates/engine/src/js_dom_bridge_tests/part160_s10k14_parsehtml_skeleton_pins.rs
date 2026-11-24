// s10k14 parseHTML 文档骨架保真钉测（刀⑭，2026-11-25）
// corpus 形蒸馏：WPT sanitizer-api/sanitizer-parseHTML.html 22F 单根因（文档级
// childNodes 从不挂载骨架→主文档迭代器只达根节点）修复面的常驻回归钉：
// 骨架挂载遍历／head 无子（工厂 title 清）／跨域 isEqualNode 配对／DOCTYPE 捕获
// （双路径）／漏斗骨架 verdict 过滤（head 剥／html 剥空文档）／注释剥离＋并文
// （safe）／注释保留（unsafe）／工厂流零扰守卫。
const S10K14_PARSEHTML_SKELETON_PINS_JS: &str = r##"
(function () {
  function eq(a, b, m) { if (a !== b) return 'FAIL:' + m + '(' + a + '!=' + b + ')'; return null; }
  var fails = [];
  function chk(r) { if (r) fails.push(r); }
  // corpus assert_subtree_equals 蒸馏（跳根对＋isEqualNode 逐对＋尾查）
  function subEq(a, b) {
    var i1 = document.createNodeIterator(a);
    var i2 = document.createNodeIterator(b);
    var c1 = i1.nextNode(), c2 = i2.nextNode();
    var bad = null;
    do {
      c1 = i1.nextNode(); c2 = i2.nextNode();
      if (!c1) break;
      var ok = false;
      try { ok = c1.isEqualNode(c2); } catch (e) { ok = false; }
      if (!ok) {
        bad = 'pair ' + c1.nodeType + (c1.tagName ? ':' + c1.tagName : '') + ' vs ' +
          (c2 ? c2.nodeType + (c2.tagName ? ':' + c2.tagName : '') : 'null');
        break;
      }
    } while (c1);
    if (!bad && c2) bad = 'extra nodes in node2';
    return bad;
  }
  function el(tag, attrs, kids) {
    var e = document.createElement(tag);
    if (attrs) for (var i = 0; i < attrs.length; i++) e.setAttribute(attrs[i][0], attrs[i][1]);
    if (kids) for (var j = 0; j < kids.length; j++) e.appendChild(kids[j]);
    return e;
  }
  function txt(s) { return document.createTextNode(s); }
  function com(s) { return document.createComment(s); }
  function wrap(kids) { var w = document.createElement('div'); if (kids) for (var i = 0; i < kids.length; i++) w.appendChild(kids[i]); return w; }
  function cfg(s) { return { sanitizer: s }; }
  function htmlTree(bodyKids) {
    var b = el('body');
    if (bodyKids) for (var i = 0; i < bodyKids.length; i++) b.appendChild(bodyKids[i]);
    return el('html', null, [el('head'), b]);
  }
  // ① 骨架挂载遍历（单根因面）：raw parse 文档经主文档迭代器可达骨架＋内容
  var d1 = Document.parseHTMLUnsafe('text');
  chk(eq(d1.childNodes.length, 1, '① doc kids'));
  chk(eq(d1.childNodes[0].nodeType, 1, '① doc kid tag'));
  chk(subEq(d1, wrap([htmlTree([txt('text')])])), '① subtree text');
  // ② head 无子（工厂 title 清）＋head/body 活树视图（body 内容 = 解析产物）
  chk(eq(d1.documentElement.childNodes[0].childNodes.length, 0, '② head empty'));
  // ③ 跨域 isEqualNode 配对（mock 骨架 vs 真 handle 树，双向）
  var real = htmlTree([txt('text')]);
  chk(eq(d1.documentElement.isEqualNode(real), true, '③ html fwd'));
  chk(eq(real.isEqualNode(d1.documentElement), true, '③ html rev'));
  // ④ DOCTYPE 捕获（raw 路径）——nodeType/name/publicId/systemId＋期望侧
  //   createDocumentType 真 handle 配对
  var ed4 = new Document();
  ed4.appendChild(document.implementation.createDocumentType('html', '', ''));
  ed4.appendChild(htmlTree([txt('text')]));
  var d4 = Document.parseHTMLUnsafe('<!DOCTYPE html>text', {});
  chk(eq(d4.childNodes.length, 2, '④ doc kids'));
  chk(eq(d4.childNodes[0].nodeType, 10, '④ doctype nodeType'));
  chk(eq(d4.childNodes[0].name, 'html', '④ doctype name'));
  chk(eq(d4.childNodes[0].publicId + '|' + d4.childNodes[0].systemId, '|', '④ doctype ids'));
  chk(subEq(d4, ed4), '④ subtree doctype doc');
  // ⑤ DOCTYPE（漏斗 safe 路径同形）
  var d5 = Document.parseHTML('<!DOCTYPE html>text', {});
  chk(eq(d5.childNodes.length, 2, '⑤ doc kids'));
  chk(subEq(d5, ed4), '⑤ subtree doctype doc');
  // ⑥ 漏斗骨架 verdict：elements 未列 head→剥（tc1 形）
  var d6 = Document.parseHTML('<div>text', cfg({ elements: ['html', 'body', 'div'] }));
  chk(subEq(d6, wrap([(function () {
    var b = el('body'); var dv = el('div'); dv.appendChild(txt('text')); b.appendChild(dv);
    return el('html', null, [b]);
  })()])), '⑥ head stripped');
  // ⑦ html 未列→空文档（tc2 形——假通过转真判定的 P2F 防线）
  var d7 = Document.parseHTML('<div>text', cfg({ elements: ['body', 'div'] }));
  chk(eq(d7.childNodes.length, 0, '⑦ empty doc'));
  // ⑧ head+body 未列→html-only（tc3 形）
  var d8 = Document.parseHTML('<div>text', cfg({ elements: ['html', 'div'] }));
  chk(subEq(d8, wrap([el('html')])), '⑧ html only');
  // ⑨ safe 路径注释剥＋相邻文本并文（tc3 safe 形）
  var d9 = Document.parseHTML('<div>a<!-- xx -->b', cfg({}));
  chk(subEq(d9, wrap([htmlTree([(function () {
    var dv = el('div'); dv.appendChild(txt('ab')); return dv;
  })()])])), '⑨ comment stripped merged');
  // ⑩ unsafe 路径注释保留（tc3 unsafe 形——配置忽略语义）
  var d10 = Document.parseHTMLUnsafe('<div>a<!-- xx -->b', cfg({}));
  chk(subEq(d10, wrap([htmlTree([(function () {
    var dv = el('div'); dv.appendChild(txt('a')); dv.appendChild(com(' xx ')); dv.appendChild(txt('b'));
    return dv;
  })()])])), '⑩ comment kept unsafe');
  // ⑪ 属性面：unsafe 配置忽略（data-xyz+id 留，title 剥）；body 起始标签属性
  //   不并入（tc5 形——expected 无 onload）
  var d11 = Document.parseHTMLUnsafe('<div data-xyz="1" id="2" title="3">', cfg({ attributes: ['id'] }));
  chk(subEq(d11, wrap([htmlTree([el('div', [['data-xyz', '1'], ['id', '2']])])])), '⑪ attrs unsafe');
  var d11b = Document.parseHTMLUnsafe('<body><body onload="alert(1)">Hello', cfg({ attributes: [] }));
  chk(subEq(d11b, wrap([htmlTree([txt('Hello')])])), '⑪ body attr not merged');
  // ⑫ 工厂流零扰守卫：createHTMLDocument title 元素仍在（title 清只作用解析流）
  var cd = document.implementation.createHTMLDocument('T');
  chk(eq(cd.head.childNodes.length, 1, '⑫ createHTMLDocument title intact'));
  // ⑬ 无输入边界：空串解析返回挂载骨架文档（write 跳过但骨架照挂）
  var d13 = Document.parseHTMLUnsafe('');
  chk(eq(d13.childNodes.length, 1, '⑬ empty input skeleton'));
  return fails.length ? fails.join('; ') : 'PASS';
})();
"##;

#[test]
fn test_s10k14_parsehtml_skeleton_knife14_pins() {
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
        "<html><head><title>k14pins</title></head><body><div id=\"k14pins\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/sanitizer-api/pins-k14.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    let verdict = sandbox.execute(S10K14_PARSEHTML_SKELETON_PINS_JS).unwrap().value;
    assert_eq!(verdict, "PASS", "s10k14 parseHTML 骨架钉测失败：{}", verdict);
}
