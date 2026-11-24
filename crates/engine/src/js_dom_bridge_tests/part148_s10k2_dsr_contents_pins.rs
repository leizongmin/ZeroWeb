// js_dom_bridge 测试切片 148。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-registries 刀②续 DSR contents 常驻钉测（2026-11-24）——主文档模板 contents 的
/// 声明式 shadow root 视图三面（wpt-data 探针页属 gitignored per-clone 刮擦面，按刀①
/// 协议用后即删；本钉测即常驻回归载体）：
/// ① doc-parse 物化面——静态模板 contents 视图 host.shadowRoot 非 null（open +
/// clonable 位，template 元素已消费）；② cloneNode(true) 转登面——clonable 根随
/// contents 克隆（_wcRebuildAsHandle → _zwTransferDeclarativeSR）；③ 入文档面——
/// append 后 docfind host.shadowRoot 可用 + SR querySelector/appendChild 语义。
/// 配套引擎面（zero-dom parser create_element 空 contents 即注册 fragment）钉在
/// zero-dom `empty_template_contents_registration_tests`；innerHTML/DOMParser 视图
///（opt-in 不物化）不属本钉测域（spec 分流，见 bridge
/// `template_contents_json_doc` sr 字段注）。
const S10K2_DSR_FACES_JS: &str = r#"
globalThis.__p1 = (function () {
  var tpl = document.getElementById('tpl-p');
  if (!tpl || !tpl.content) return 'notpl';
  var host = tpl.content.querySelector('#tpl-host-p');
  if (!host) return 'nohost';
  var sr = host.shadowRoot;
  if (!sr) return 'nosr';
  return sr.mode === 'open' && sr.clonable === true;
})();
globalThis.__p2 = (function () {
  var clone = document.getElementById('tpl-p').content.cloneNode(true);
  var ch = clone.querySelector('#tpl-host-p');
  return !!(ch && ch.shadowRoot && ch.shadowRoot.clonable === true);
})();
globalThis.__p3 = (function () {
  var clone = document.getElementById('tpl-p').content.cloneNode(true);
  document.body.appendChild(clone);
  var host = document.querySelector('#tpl-host-p');
  if (!host || !host.shadowRoot) return 'nosr';
  var el = document.createElement('span');
  host.shadowRoot.appendChild(el);
  return host.shadowRoot.querySelector('span') === el;
})();
"#;

#[test]
fn test_s10k2_dsr_contents_pins() {
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
        "<html><body><template id=\"tpl-p\"><div id=\"tpl-host-p\"><template shadowrootmode=\"open\" shadowrootclonable></template></div></template></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/custom-elements/registries/pins.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K2_DSR_FACES_JS).unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__p1").unwrap().value,
        "true",
        "刀②续 ① doc-parse 物化：contents 视图 host.shadowRoot 非 null（open+clonable）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__p2").unwrap().value,
        "true",
        "刀②续 ② cloneNode 转登：clonable 根随 contents 克隆"
    );
    assert_eq!(
        sandbox.execute("globalThis.__p3").unwrap().value,
        "true",
        "刀②续 ③ 入文档：append 后 docfind shadowRoot + SR querySelector 语义"
    );
}
