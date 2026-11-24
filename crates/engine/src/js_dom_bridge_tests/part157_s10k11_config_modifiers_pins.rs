// js_dom_bridge 测试切片 157。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-sanitizer 刀⑪ 常驻钉测（2026-11-25）——SanitizerConfig 校验矩阵与
/// allowElement/allowAttribute 组合语义（spec：https://wicg.github.io/sanitizer-api/
/// validate a sanitizer config 步骤 7.1.1.2/7.1.1.3（新编号 16.2.1.4/17.1.1.5）与
/// Sanitizer.allowElement 步骤；语料 sanitizer-config.html 17F + sanitizer-modifiers.html
/// 8F 簇的常驻回归载体）：
/// ① canon 元素归一：namespace 缺席→XHTML、空串→null、null 保留；字符串形 rm:[]；
/// ② 构造校验矩阵全 TypeError（元素/属性重复含 ns 归一后相撞、16.2.1.4 subset、
///    17.1.1.5 交集、16.3 data-*、elements/removeElements 并存、17.1.1.1 双 member）；
/// ③ allowElement 替换语义全序列（modifiers 三臂腿：attrs/rm 双带让位规则、
///    重复调用零变化返 false、纯条目重置缺省形、字符串形 no-change false）；
/// ④ removeElements 方向统一允许语义（带 attrs member 或非空 rm 拒绝、
///    rm:[] 剔除、rWC 移动臂）；
/// ⑤ removeAttribute 元素条目剥空保留 []；
/// ⑥ allowProcessingInstruction remove 方向剥除语义 / allow 方向追加。
/// 根因链与判读见 evidence `dc2-s10k11-config-modifiers-knife11-verdict.md`。
const S10K11_CONFIG_MODIFIERS_KNIFE11_PINS_JS: &str = r#"
globalThis.__k11 = (function () {
  var bad = [];
  var XHTML = 'http://www.w3.org/1999/xhtml';
  var NS = 'http://example.org/';
  var S = globalThis.Sanitizer;
  function attrName(a) { return a && a.name; }
  function names(list) { return (list || []).map(attrName); }

  // ① canon 归一（get() 输出按 ns+name 排序——按 name 查找不依赖位置）。
  function elByN(list, n) { for (var i = 0; i < list.length; i++) if (list[i].name === n) return list[i]; return null; }
  var c1 = new S({ elements: [{ name: 'x' }, { name: 'y', namespace: '' }, { name: 'z', namespace: null }] }).get();
  if (elByN(c1.elements, 'x').namespace !== XHTML) bad.push('p1-undef:' + elByN(c1.elements, 'x').namespace);
  if (elByN(c1.elements, 'y').namespace !== null) bad.push('p1-empty:' + elByN(c1.elements, 'y').namespace);
  if (elByN(c1.elements, 'z').namespace !== null) bad.push('p1-null:' + elByN(c1.elements, 'z').namespace);
  var c1b = new S({ elements: ['w'] }).get();
  if (c1b.elements[0].namespace !== XHTML || JSON.stringify(c1b.elements[0].removeAttributes) !== '[]') bad.push('p1-str');

  // ② 校验矩阵（全部 TypeError）。
  var cases = [
    // 元素重复：空串与 null 归一后同名同 ns 相撞。
    [{ elements: [{ name: 'a', namespace: '' }, { name: 'a', namespace: null }] }],
    // 字符串与 member 缺省 ns 相撞。
    [{ removeElements: ['abc', { name: 'abc' }] }],
    // 16.2.1.4 subset：element.removeAttributes ⊄ config.attributes。
    [{ attributes: ['class'], elements: [{ name: 'div', removeAttributes: ['title'] }] }],
    // 17.1.1.5 交集：element.removeAttributes ∩ config.removeAttributes 非空。
    [{ removeAttributes: ['id'], elements: [{ name: 'p', removeAttributes: ['id'] }] }],
    // 16.3 data-*（config 级）。
    [{ attributes: ['data-foo'], dataAttributes: true }],
    // elements 与 removeElements 并存。
    [{ elements: [], removeElements: [] }],
    // 17.1.1.1 双 member。
    [{ removeAttributes: [], elements: [{ name: 'p', attributes: ['a'], removeAttributes: ['b'] }] }],
  ];
  for (var i = 0; i < cases.length; i++) {
    var threw = null;
    try { new S(cases[i][0]); } catch (e) { threw = e; }
    if (!threw || !(threw instanceof TypeError)) bad.push('p2-' + i + ':' + (threw && threw.name));
  }

  // ③ 替换语义全序列（modifiers 三臂腿）。
  var s3 = new S({ elements: [], removeAttributes: ['id'] });
  if (s3.allowElement({ name: 'p', attributes: ['id', 'title'] }) !== true) bad.push('p3a');
  var e3 = s3.get().elements[0];
  if (JSON.stringify(names(e3.attributes)) !== '["title"]') bad.push('p3b:' + JSON.stringify(e3));
  if ('removeAttributes' in e3) bad.push('p3c');
  if (s3.allowElement({ name: 'p', attributes: ['id', 'title'] }) !== false) bad.push('p3d');
  if (s3.allowElement({ name: 'p', removeAttributes: ['id', 'class'] }) !== true) bad.push('p3e');
  var e3b = s3.get().elements[0];
  if (JSON.stringify(names(e3b.removeAttributes)) !== '["class"]') bad.push('p3f:' + JSON.stringify(e3b));
  if ('attributes' in e3b) bad.push('p3g');
  if (s3.allowElement({ name: 'p', removeAttributes: ['id', 'class'] }) !== false) bad.push('p3h');
  if (s3.allowElement({ name: 'p', attributes: ['id', 'dir', 'lang'], removeAttributes: ['id', 'lang'] }) !== true) bad.push('p3i');
  var e3c = s3.get().elements[0];
  if (JSON.stringify(names(e3c.attributes)) !== '["dir"]') bad.push('p3j:' + JSON.stringify(e3c));
  if ('removeAttributes' in e3c) bad.push('p3k');
  if (s3.allowElement({ name: 'p', attributes: ['id', 'dir', 'lang'], removeAttributes: ['id', 'lang'] }) !== false) bad.push('p3l');
  if (s3.allowElement({ name: 'p' }) !== true) bad.push('p3m');
  var e3d = s3.get().elements[0];
  if ('attributes' in e3d) bad.push('p3n');
  if (!e3d.removeAttributes || e3d.removeAttributes.length !== 0) bad.push('p3o:' + JSON.stringify(e3d));
  if (s3.allowElement('p') !== false) bad.push('p3p');

  // ④ removeElements 方向统一允许语义。
  var s4 = new S({ removeElements: ['a'], replaceWithChildrenElements: ['b'] });
  if (s4.allowElement('span') !== false) bad.push('p4a');
  if (s4.allowElement({ name: 'a', namespace: NS }) !== false) bad.push('p4b');
  if (s4.allowElement('b') !== true) bad.push('p4c');
  if (JSON.stringify(s4.get().replaceWithChildrenElements) !== '[]') bad.push('p4d');
  if (s4.allowElement({ name: 'a', attributes: ['dir'] }) !== false) bad.push('p4e');
  if (s4.allowElement({ name: 'a', removeAttributes: ['dir'] }) !== false) bad.push('p4f');
  if (s4.allowElement({ name: 'a', removeAttributes: [] }) !== true) bad.push('p4g');
  if (JSON.stringify(s4.get().removeElements) !== '[]') bad.push('p4h');

  // ⑤ removeAttribute 元素条目剥空保留 []。
  var s5 = new S({ elements: [{ name: 'p', removeAttributes: ['x'] }] });
  if (s5.removeAttribute('x') !== true) bad.push('p5a');
  var e5 = s5.get().elements[0];
  if (!e5.removeAttributes || e5.removeAttributes.length !== 0) bad.push('p5b:' + JSON.stringify(e5));
  if (s5.removeAttribute('x') !== false) bad.push('p5c');

  // ⑥ PI 双方向。
  var s6 = new S({ removeProcessingInstructions: ['x'] });
  if (s6.allowProcessingInstruction('x') !== true) bad.push('p6a');
  if (!Array.isArray(s6.get().removeProcessingInstructions) || s6.get().removeProcessingInstructions.length !== 0) bad.push('p6b');
  if (s6.allowProcessingInstruction('y') !== false) bad.push('p6c');
  var s7 = new S({ processingInstructions: ['x'] });
  if (s7.allowProcessingInstruction('y') !== true) bad.push('p7a');
  var p7 = s7.get().processingInstructions;
  if (p7.length !== 2 || p7[0].target !== 'x' || p7[1].target !== 'y') bad.push('p7b:' + JSON.stringify(p7));
  if (s7.allowProcessingInstruction('y') !== false) bad.push('p7c');

  return bad.length === 0 ? { ok: true } : { ok: false, bad: bad.join('|') };
})();
"#;

#[test]
fn test_s10k11_config_modifiers_knife11_pins() {
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
        "<html><head><title>k11</title></head><body><div id=\"k11page\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/sanitizer-api/pins-k11.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K11_CONFIG_MODIFIERS_KNIFE11_PINS_JS).unwrap();
    let verdict = sandbox.execute("JSON.stringify(globalThis.__k11)").unwrap().value;
    assert!(
        verdict.contains("\"ok\":true"),
        "刀⑪ config/modifiers（canon 归一/校验矩阵/allowElement 替换语义/removeElements 统一允许/剥空保留/PI 双方向）：{}",
        verdict
    );
}
