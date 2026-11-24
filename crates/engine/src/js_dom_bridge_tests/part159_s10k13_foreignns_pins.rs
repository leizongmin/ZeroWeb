// js_dom_bridge 测试切片 159。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-sanitizer 刀⑬ 常驻钉测（2026-11-25）——foreign namespace 端到端保真
/// （spec：https://html.spec.whatwg.org/multipage/parsing.html#adjust-foreign-attributes
/// ／https://dom.spec.whatwg.org/#concept-node-equals；语料
/// sanitizer-basic-filtering.html namespaces/0-9 ×44＋javascript-url 同根
/// attr/元素 ns 表示错误的常驻回归载体）：
/// ① handle 域 setHTMLUnsafe 重建保真——svg/rect 元素 ns＋adjust 命中属性
///    (xlink:href/xml:space) ns 三元组＋表外 xmlns:foo null-ns 整名形；
/// ② doc 域 parseHTMLUnsafe 物化——math/mi MathML ns＋带前缀属性三元组；
/// ③ isEqualNode (ns,local,value) 配对——期望树 createElementNS/setAttributeNS
///    构建 vs 解析树（语料 assert_subtree_equals 同形，plain/handle 双域）；
/// ④ setHTML 默认消毒链元素 ns 保真（svg/math 跨 foreign context）。
/// 根因链与判读见 evidence `dc2-s10k13-foreign-ns-writechain-knife13-verdict.md`。
const S10K13_FOREIGNNS_KNIFE13_PINS_JS: &str = r##"
globalThis.__k13 = (function () {
  var bad = [];
  var doc = globalThis.document;
  var SVG = 'http://www.w3.org/2000/svg';
  var XLINK = 'http://www.w3.org/1999/xlink';
  var XMLNS = 'http://www.w3.org/XML/1998/namespace';
  var MATHML = 'http://www.w3.org/1998/Math/MathML';

  function attrTriple(el, qname) {
    var a = el.attributes;
    for (var i = 0; i < a.length; i++) {
      if (String(a[i].name) === qname) {
        return qname + '|' + a[i].namespaceURI + '|' + a[i].prefix + '|' + a[i].localName;
      }
    }
    return 'MISS:' + qname;
  }

  // ① handle 域 setHTMLUnsafe 重建（adjust 命中形＋null-ns 形三态）。
  var d1 = doc.createElement('div');
  d1.setHTMLUnsafe('<svg><rect xlink:href="u" xml:space="default" xmlns:foo="barspace"></svg>');
  var s1 = d1.childNodes[0];
  if (!s1 || s1.namespaceURI !== SVG) bad.push('p1elns:' + (s1 && s1.namespaceURI));
  var r1 = s1 && s1.childNodes[0];
  if (!r1) bad.push('p1-norect');
  else {
    if (r1.namespaceURI !== SVG) bad.push('p1rectns:' + r1.namespaceURI);
    if (attrTriple(r1, 'xlink:href') !== 'xlink:href|' + XLINK + '|xlink|href') bad.push('p1x:' + attrTriple(r1, 'xlink:href'));
    if (attrTriple(r1, 'xml:space') !== 'xml:space|' + XMLNS + '|xml|space') bad.push('p1s:' + attrTriple(r1, 'xml:space'));
    if (attrTriple(r1, 'xmlns:foo') !== 'xmlns:foo|null|null|xmlns:foo') bad.push('p1f:' + attrTriple(r1, 'xmlns:foo'));
  }

  // ② doc 域 parseHTMLUnsafe 物化（math ns＋svg 前缀属性三元组）。
  var w2 = globalThis.Document.parseHTMLUnsafe('<math><mi>x');
  var m2 = w2 && w2.body && w2.body.childNodes[0];
  if (!m2 || m2.namespaceURI !== MATHML) bad.push('p2math:' + (m2 && m2.namespaceURI));
  var w2b = globalThis.Document.parseHTMLUnsafe('<svg><use xlink:href="#i"></svg>');
  var u2 = w2b && w2b.body && w2b.body.childNodes[0];
  if (!u2 || u2.namespaceURI !== SVG) bad.push('p2svg:' + (u2 && u2.namespaceURI));
  var uu2 = u2 && u2.childNodes[0];
  if (attrTriple(uu2, 'xlink:href') !== 'xlink:href|' + XLINK + '|xlink|href') bad.push('p2x:' + attrTriple(uu2, 'xlink:href'));

  // ③ isEqualNode 配对（语料 assert_subtree_equals 形）——handle 域。
  function expectRect() {
    var svg = doc.createElementNS(SVG, 'svg');
    var rect = doc.createElementNS(SVG, 'rect');
    rect.setAttributeNS(XLINK, 'xlink:href', 'u');
    rect.setAttributeNS(XMLNS, 'xml:space', 'default');
    svg.appendChild(rect);
    return svg;
  }
  var d3 = doc.createElement('div');
  d3.setHTMLUnsafe('<svg><rect xlink:href="u" xml:space="default"></svg>');
  if (!d3.childNodes[0].isEqualNode(expectRect())) bad.push('p3-ineq');
  // 反向钉：值不同必须判不等（守卫 isEqualNode 假恒真）。
  var d3b = doc.createElement('div');
  d3b.setHTMLUnsafe('<svg><rect xlink:href="OTHER"></svg>');
  if (d3b.childNodes[0].isEqualNode(expectRect())) bad.push('p3b-falsepos');

  // ③b isEqualNode 跨域（doc 域物化 plain vs createElementNS handle）——
  // javascript-url/parseHTML 语料族断言形。
  var w3 = globalThis.Document.parseHTMLUnsafe('<svg><use xlink:href="#i"></svg>');
  var e3 = doc.createElementNS(SVG, 'svg');
  var use3 = doc.createElementNS(SVG, 'use');
  use3.setAttributeNS(XLINK, 'xlink:href', '#i');
  e3.appendChild(use3);
  if (!(w3 && w3.body && w3.body.childNodes[0].isEqualNode(e3))) bad.push('p3b-ineq');

  // ④ setHTML 默认消毒链元素 ns 保真（svg/math 混合 foreign context）。
  // 默认配置剥 xlink:href（probe3 W3 实证）——attr 三元组由①②③钉，此处只钉元素 ns。
  var d4 = doc.createElement('div');
  d4.setHTML('<svg><rect xlink:href="u"></svg><math><mi>x');
  var s4 = d4.childNodes[0];
  var m4 = d4.childNodes[1];
  if (!s4 || s4.namespaceURI !== SVG) bad.push('p4svg:' + (s4 && s4.namespaceURI));
  if (!m4 || m4.namespaceURI !== MATHML) bad.push('p4math:' + (m4 && m4.namespaceURI));

  if (bad.length) return 'FAIL:' + bad.join(' ; ');
  return 'PASS';
})();
"##;

#[test]
fn test_s10k13_foreign_ns_knife13_pins() {
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
        "<html><head><title>k13pins</title></head><body><div id=\"k13pins\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/sanitizer-api/pins-k13.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K13_FOREIGNNS_KNIFE13_PINS_JS).unwrap();
    let verdict = sandbox.execute("globalThis.__k13").unwrap().value;
    assert_eq!(verdict, "PASS", "s10k13 foreign-ns 钉测失败：{}", verdict);
}
