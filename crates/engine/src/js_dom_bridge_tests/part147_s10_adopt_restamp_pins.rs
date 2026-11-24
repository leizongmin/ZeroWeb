// js_dom_bridge 测试切片 147。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-registries 刀② adopt 重钉常驻钉测（2026-11-24）——探针页 faces 的桥内等价
/// （探针页在 wpt-data 属 gitignored per-clone 刮擦面，按刀① 协议用后即删；本钉测
/// 即常驻回归载体）。修复面：R225 frag 展平清空直子后 stamp/R191 走查双漏章
/// （appendChild 体内展平前快照直子、展平后补章，跨文档域限定）＋ shadow root 挂
/// host 影子槽非 childNodes、直子展平走查恒不可达（经 host.shadowRoot 入口续走，
/// 显式 slot 走 SR 臂重钉）。
///
/// 断言面：① frag 展平补章——跨文档 appendChild(frag) 后 frag 直子 global-own 翻
/// null-reg；② SR global-own 重钉——adopt 后 sr.customElementRegistry === null；
/// ③ scoped-own 持久（spec：显式 scoped registry adopt 不重钉）；④ null-own 保持；
/// ⑤ 同文档 frag 搬移零移（无 adopt 步，域限定门——global-own 原样）。
const S10_FACES_JS: &str = r#"
globalThis.__f1 = (function () {
  var d2 = document.implementation.createHTMLDocument('t');
  var el = document.createElement('a-b', {customElementRegistry: customElements});
  var frag = document.createDocumentFragment();
  frag.appendChild(el);
  d2.body.appendChild(frag);
  return el.customElementRegistry === null;
})();
globalThis.__f2 = (function () {
  var d2 = document.implementation.createHTMLDocument('t');
  var host = document.createElement('div');
  var sr = host.attachShadow({mode: 'open', customElementRegistry: customElements});
  d2.body.appendChild(host);
  return sr.customElementRegistry === null;
})();
globalThis.__f3 = (function () {
  var scopedRegistry = new CustomElementRegistry();
  var host = document.createElement('div');
  var sr = host.attachShadow({mode: 'open', customElementRegistry: scopedRegistry});
  var d2 = document.implementation.createHTMLDocument('t');
  d2.body.appendChild(host);
  return sr.customElementRegistry === scopedRegistry;
})();
globalThis.__f4 = (function () {
  var d2 = document.implementation.createHTMLDocument('t');
  var el = document.createElement('a-b', {customElementRegistry: null});
  d2.body.appendChild(el);
  return el.customElementRegistry === null;
})();
globalThis.__f5 = (function () {
  var el = document.createElement('a-b', {customElementRegistry: customElements});
  var frag = document.createDocumentFragment();
  frag.appendChild(el);
  document.body.appendChild(frag);
  return el.customElementRegistry === customElements;
})();
"#;

#[test]
fn test_s10_adopt_restamp_pins() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10_FACES_JS).unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__f1").unwrap().value,
        "true",
        "刀② ① frag 展平补章：跨文档 appendChild(frag) 后 global-own 翻 null-reg"
    );
    assert_eq!(
        sandbox.execute("globalThis.__f2").unwrap().value,
        "true",
        "刀② ② SR global-own 重钉：adopt 后 sr.customElementRegistry === null"
    );
    assert_eq!(
        sandbox.execute("globalThis.__f3").unwrap().value,
        "true",
        "刀② ③ scoped-own 持久：adopt 后 scoped registry 原样"
    );
    assert_eq!(
        sandbox.execute("globalThis.__f4").unwrap().value,
        "true",
        "刀② ④ null-own 保持：adopt 后仍 null"
    );
    assert_eq!(
        sandbox.execute("globalThis.__f5").unwrap().value,
        "true",
        "刀② ⑤ 同文档 frag 搬移零移：global-own 原样（域限定门）"
    );
}
