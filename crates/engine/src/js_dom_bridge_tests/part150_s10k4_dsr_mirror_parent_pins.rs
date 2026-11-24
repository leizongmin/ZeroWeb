// js_dom_bridge 测试切片 150。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-registries 刀④ 常驻钉测（2026-11-24）——DSR 镜像子 parentNode 接线两针
/// （part05 `_zwDeclarativeShadowEnsureMirrors` 镜像子 pending-parent 槽同步；wpt-data
/// 探针页属 gitignored per-clone 刮擦面，按刀①协议用后即删，本钉测即常驻回归载体）：
/// ① DSR-attr 根（`shadowrootcustomelementregistry`）：镜像子 parentNode 语义父＝
/// ShadowRoot 容器 wrapper（spec concept-node-pre-insert 步骤 6「set parent」；真实浏览器
/// `sr.firstElementChild.parentNode === sr`），registry 解析父链上行达盖 decl 印的
/// wrapper → null（ShadowRoot-init-declarative test1/test2 expected-null-but-got-object
/// 回归面——修前 parentNode 落引擎父视图折叠 host，解析直达文档默认 global）；
/// ② 无属性 DSR 根：镜像子 parentNode 同接 SR wrapper，registry 解析经 wrapper 无槽越界
/// host → 文档默认 live（window.customElements）——walk 语义与刀③矩阵一致。
/// 根因链与 k4 探针实证见 evidence `dc2-s10k3-registries-knife3-verdict.md` 残面节。
const S10K4_DSR_MIRROR_PARENT_PINS_JS: &str = r#"
globalThis.__k4a = (function () {
  var ha = document.getElementById('h4a');
  if (!ha || !ha.shadowRoot) return 'nosr';
  var sr = ha.shadowRoot;
  var ce = sr.firstElementChild, dv = sr.lastElementChild;
  if (!ce || !dv) return 'nokids';
  if (ce.parentNode !== sr) return 'ce-pn-not-sr';
  if (dv.parentNode !== sr) return 'dv-pn-not-sr';
  if (ce.customElementRegistry !== null) return 'ce-reg-not-null';
  if (dv.customElementRegistry !== null) return 'dv-reg-not-null';
  return true;
})();
globalThis.__k4b = (function () {
  var hb = document.getElementById('h4b');
  if (!hb || !hb.shadowRoot) return 'nosr';
  var sr = hb.shadowRoot;
  var ce = sr.firstElementChild, dv = sr.lastElementChild;
  if (!ce || !dv) return 'nokids';
  if (ce.parentNode !== sr) return 'ce-pn-not-sr';
  if (dv.parentNode !== sr) return 'dv-pn-not-sr';
  if (ce.customElementRegistry !== window.customElements) return 'ce-reg-not-live';
  if (dv.customElementRegistry !== window.customElements) return 'dv-reg-not-live';
  return true;
})();
"#;

#[test]
fn test_s10k4_dsr_mirror_parent_pins() {
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
        "<html><body><div id=\"h4a\"><template shadowrootmode=\"open\" shadowrootcustomelementregistry><custom-element></custom-element><div></div></template></div><div id=\"h4b\"><template shadowrootmode=\"open\"><custom-element></custom-element><div></div></template></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/custom-elements/registries/pins-k4.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K4_DSR_MIRROR_PARENT_PINS_JS).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__k4a").unwrap().value,
        "true",
        "刀④ ① DSR-attr 镜像子：parentNode===SR wrapper＋registry null（父链上行达 decl 印 wrapper）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__k4b").unwrap().value,
        "true",
        "刀④ ② 无属性 DSR 镜像子：parentNode===SR wrapper＋registry live（wrapper 无槽越界 host→doc default）"
    );
}
