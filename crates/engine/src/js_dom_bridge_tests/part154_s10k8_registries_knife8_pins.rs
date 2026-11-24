// js_dom_bridge 测试切片 154。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-registries 刀⑧ 常驻钉测（2026-11-24）——registry 创建时点 own-stamp 模型
/// （wpt-data 探针页属 gitignored per-clone 刮擦面，按刀①协议用后即删，本钉测
/// 即常驻回归载体）：
/// ① plain createElement own-stamp＝创建文档 registry，append 进 scoped shadow 树
///    不重钉（恒 global）、移出恒 global；
/// ② initialize 条件盖印对非 null own 槽不触碰：plain（Face A 全局印）元素在
///    initialize 后 SR 内／移出／跨根恒 global（spec 步 3 仅 set null registry 元素）；
/// ③ initialize 条件盖印双臂：own null 印记（spec null registry）→ 盖为实例；
///    显式 option 他实例（语料「only upgrades」registry2 存活面）→ 不触碰；
/// ④ DSR clonable 根克隆传播：host.cloneNode(true) 后 .shadowRoot 物化＋clonable
///    真＋shadow 子可达；克隆树 initialize 盖印（no-BC 域印子盖为实例）后移出恒等
///    （element-mutation test2/test3 前置面）；
/// ⑤ createElementNS plain 臂同面（global 印）＋option 臂不受影响（实例印）；
/// ⑥ 显式 option 槽保持：{customElementRegistry: inst}→inst、null→null。
/// 根因链与 element-mutation.html 15F 簇见 evidence
/// `dc2-s10k8-registries-knife8-verdict.md`。
const S10K8_REGISTRIES_KNIFE8_PINS_JS: &str = r#"
globalThis.__k8 = (function () {
  var bad = [];
  var rn = function (r) {
    if (r === globalThis.customElements) return 'global';
    if (r === null) return 'null';
    if (r === globalThis.__k8ra) return 'ra';
    if (r === globalThis.__k8rb) return 'rb';
    return 'other';
  };
  var doc = globalThis.document;
  globalThis.__k8ra = new CustomElementRegistry();
  globalThis.__k8rb = new CustomElementRegistry();

  // ① plain createElement own-stamp＋移动不重钉（双向）。
  var host = doc.createElement('div');
  var srA = host.attachShadow({ mode: 'open', customElementRegistry: globalThis.__k8ra });
  var el = doc.createElement('div');
  if (rn(el.customElementRegistry) !== 'global') bad.push('create:' + rn(el.customElementRegistry));
  srA.appendChild(el);
  if (rn(el.customElementRegistry) !== 'global') bad.push('in-sr:' + rn(el.customElementRegistry));
  doc.body.appendChild(el);
  if (rn(el.customElementRegistry) !== 'global') bad.push('out-sr:' + rn(el.customElementRegistry));

  // ② initialize 条件盖印不触碰非 null own 槽：plain（Face A 全局印）元素
  //    SR 内／移出／跨根恒 global（spec 步 3 仅 set null registry 元素）。
  var el2 = doc.createElement('div');
  srA.appendChild(el2);
  globalThis.__k8ra.initialize(srA);
  if (rn(el2.customElementRegistry) !== 'global') bad.push('init:' + rn(el2.customElementRegistry));
  doc.body.appendChild(el2);
  if (rn(el2.customElementRegistry) !== 'global') bad.push('init-out:' + rn(el2.customElementRegistry));
  var hostB = doc.createElement('div');
  var srB = hostB.attachShadow({ mode: 'open', customElementRegistry: globalThis.__k8rb });
  srB.appendChild(el2);
  if (rn(el2.customElementRegistry) !== 'global') bad.push('init-cross:' + rn(el2.customElementRegistry));

  // ③ initialize 条件盖印双臂：own null 印记→盖为实例（spec null registry set）；
  //    显式 option 他实例→不触碰（语料「only upgrades」registry2 存活面）。
  var el3 = doc.createElement('div', { customElementRegistry: null });
  srA.appendChild(el3);
  globalThis.__k8ra.initialize(srA);
  if (rn(el3.customElementRegistry) !== 'ra') bad.push('override-null:' + rn(el3.customElementRegistry));
  var el3b = doc.createElement('div', { customElementRegistry: globalThis.__k8rb });
  srA.appendChild(el3b);
  globalThis.__k8ra.initialize(srA);
  if (rn(el3b.customElementRegistry) !== 'rb') bad.push('override-rb:' + rn(el3b.customElementRegistry));

  // ④ DSR clonable 根克隆传播（fixture 在初始 dom_html 种子）＋克隆树 initialize
  // 盖印＋移出恒等。
  var cloneHost = doc.getElementById('k8dsr').cloneNode(true);
  var cSr = cloneHost.querySelector('#k8dsr1') && cloneHost.querySelector('#k8dsr1').shadowRoot;
  if (!cSr) { bad.push('clone-sr:undefined'); }
  else {
    if (cSr.clonable !== true) bad.push('clone-clonable:' + cSr.clonable);
    var cChild = cSr.querySelector('#k8dsrchild');
    if (!cChild) bad.push('clone-child:miss');
    else {
      globalThis.__k8ra.initialize(cSr);
      if (rn(cChild.customElementRegistry) !== 'ra') bad.push('clone-init:' + rn(cChild.customElementRegistry));
      doc.body.appendChild(cChild);
      if (rn(cChild.customElementRegistry) !== 'ra') bad.push('clone-out:' + rn(cChild.customElementRegistry));
    }
  }

  // ⑤ createElementNS plain 印记＋option 臂。
  var nsEl = doc.createElementNS('http://www.w3.org/1999/xhtml', 'div');
  if (rn(nsEl.customElementRegistry) !== 'global') bad.push('ns:' + rn(nsEl.customElementRegistry));
  var nsOpt = doc.createElementNS('http://www.w3.org/1999/xhtml', 'div', { customElementRegistry: globalThis.__k8rb });
  if (rn(nsOpt.customElementRegistry) !== 'rb') bad.push('ns-opt:' + rn(nsOpt.customElementRegistry));

  // ⑥ 显式 option 槽保持。
  var optEl = doc.createElement('div', { customElementRegistry: globalThis.__k8rb });
  if (rn(optEl.customElementRegistry) !== 'rb') bad.push('opt:' + rn(optEl.customElementRegistry));
  var nullEl = doc.createElement('div', { customElementRegistry: null });
  if (rn(nullEl.customElementRegistry) !== 'null') bad.push('opt-null:' + rn(nullEl.customElementRegistry));

  return bad.length === 0 ? { ok: true } : { ok: false, bad: bad.join('|') };
})();
"#;

#[test]
fn test_s10k8_registries_knife8_pins() {
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
        "<html><body><div id=\"k8page\"></div>\
         <div id=\"k8dsr\"><div id=\"k8dsr1\">\
         <template shadowrootmode=\"open\" shadowrootclonable=\"true\" shadowrootcustomelementregistry>\
         <div id=\"k8dsrchild\"></div></template></div></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/custom-elements/registries/pins-k8.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K8_REGISTRIES_KNIFE8_PINS_JS).unwrap();
    let verdict = sandbox.execute("JSON.stringify(globalThis.__k8)").unwrap().value;
    assert!(
        verdict.contains("\"ok\":true"),
        "刀⑧ registry own-stamp（createElement 创建时点印＋initialize 条件盖印＋DSR clonable 克隆传播）：{}",
        verdict
    );
}
