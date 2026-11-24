// js_dom_bridge 测试切片 149。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-registries 刀③ 常驻钉测（2026-11-24）——effective-global-registry 残面修复四针
/// （wpt-data 探针页属 gitignored per-clone 刮擦面，按刀①协议用后即删；本钉测即
/// 常驻回归载体）：
/// ① no-src iframe 首访问面（part04 identity-chain k3 臂）——contentDocument/
/// contentWindow 首读即物化 fallback win + R365 realm registry 槽接线（doc 级
/// customElementRegistry 与 win.customElements 同源）；
/// ② DSR-attr transfer 印记（part05 `_zwTransferDeclarativeSR` 槽传播）——
/// shadowrootcustomelementregistry 模板 contents 克隆入主文档后 SR getter null
///（sticky null），无属性对照根 live（window.customElements）；
/// ③ 跨文档 adopt/append 矩阵（part03 `_zwCeAdoptStampSubtree` SR 臂）——DSR-attr
/// 子 sticky null（adopt 不重钉）、DSR-no-attr 子重钉 iframe doc default、
/// imperative null 重钉（非 sticky）；SR 内子元素解析 null（resolver wrapper 槽臂）；
/// ④ 主文档 hydrate 面（bridge `declarative_shadow_deep_json_doc` `r` 键）——
/// 直接 body 标记 DSR-attr 根 getter null、无属性根 live。
/// 语义矩阵与上游语料实证见 evidence `dc2-s10k3-registries-knife3-verdict.md`。
const S10K3_IFRAME_REG_PINS_JS: &str = r#"
globalThis.__p1 = (function () {
  var f = document.createElement('iframe');
  document.body.appendChild(f);
  var d = f.contentDocument;
  if (!d) return 'nodoc';
  var w = f.contentWindow;
  if (!w || !w.customElements) return 'nowin';
  if (f.contentDocument !== d) return 'identity-broken';
  return d.customElementRegistry === w.customElements;
})();
globalThis.__p2 = (function () {
  var ca = document.getElementById('tp').content.cloneNode(true);
  document.body.appendChild(ca);
  var ha = document.querySelector('#hp');
  if (!ha || !ha.shadowRoot) return 'nosr-a';
  if (ha.shadowRoot.customElementRegistry !== null) return 'attr-not-null';
  var cq = document.getElementById('tq').content.cloneNode(true);
  document.body.appendChild(cq);
  var hq = document.querySelector('#hq');
  if (!hq || !hq.shadowRoot) return 'nosr-b';
  return hq.shadowRoot.customElementRegistry === window.customElements;
})();
globalThis.__p3 = (function () {
  // adopt walk 门激活（_zwCeAdoptStampSubtree 的 __zwCeScopedActive 前置——语料域
  // 均先建过 scoped registry；本钉测同步开门，一次 scoped attachShadow 即置位）。
  try {
    var _k3act = new CustomElementRegistry();
    document.createElement('div').attachShadow({ mode: 'open', customElementRegistry: _k3act });
  } catch (_eK3act) {}
  var f3 = document.createElement('iframe');
  document.body.appendChild(f3);
  var fd = f3.contentDocument;
  if (!fd) return 'noiframe';
  var root = fd.body || fd.documentElement;
  if (!root) return 'noroot';
  var ca = document.getElementById('tp').content.cloneNode(true);
  root.appendChild(ca);
  var ha = fd.querySelector('#hp');
  if (!ha) return 'nohost-a';
  if (!ha.shadowRoot) return 'nosr-adopt-a';
  if (ha.shadowRoot.customElementRegistry !== null) return 'attr-repinned';
  var kid = fd.createElement('x-k');
  ha.shadowRoot.appendChild(kid);
  if (kid.customElementRegistry !== null) return 'kid-not-null';
  var ih = document.createElement('div');
  var isr = ih.attachShadow({ mode: 'open', customElementRegistry: null });
  if (isr.customElementRegistry !== null) return 'imp-null-main-doc-broken';
  root.appendChild(ih);
  if (isr.customElementRegistry !== fd.customElementRegistry) {
    return 'imp-not-repinned;fdslot-vs-fddr=' + (fd._zwCERegistry === fd.customElementRegistry ? 'same' : 'diff')
      + ';isrreg-vs-fdslot=' + (isr.customElementRegistry === fd._zwCERegistry ? 'same' : 'diff')
      + ';isrreg-vs-isrwrap=' + (isr.customElementRegistry === isr._zwCERegistry ? 'same' : 'diff')
      + ';isrwrap-vs-fdslot=' + (isr._zwCERegistry === fd._zwCERegistry ? 'same' : 'diff')
      + ';fddr-type=' + (typeof fd.customElementRegistry)
      + ';fdslot-type=' + (typeof fd._zwCERegistry)
      + ';isrreg-type=' + (typeof isr.customElementRegistry);
  }
  var cq = document.getElementById('tq').content.cloneNode(true);
  root.appendChild(cq);
  var hq = fd.querySelector('#hq');
  if (!hq || !hq.shadowRoot) return 'nosr-adopt-b';
  if (hq.shadowRoot.customElementRegistry !== fd.customElementRegistry) {
    return 'noattr-not-repinned;slot=' + (hq.shadowRoot._zwCERegistry === undefined ? 'undef' : 'set')
      + ';reg=' + (hq.shadowRoot.customElementRegistry === fd.customElementRegistry ? 'fd' : (hq.shadowRoot.customElementRegistry === globalThis.customElements ? 'global' : 'other'))
      + ';owndocfd=' + (hq.ownerDocument === fd)
      + ';owndocmain=' + (hq.ownerDocument === globalThis.document);
  }
  return true;
})();
globalThis.__p4 = (function () {
  var hs = document.getElementById('hs');
  if (!hs || !hs.shadowRoot) return 'nosr';
  if (hs.shadowRoot.customElementRegistry !== null) return 'attr-not-null';
  var hl = document.getElementById('hl');
  if (!hl || !hl.shadowRoot) return 'nosr2';
  return hl.shadowRoot.customElementRegistry === window.customElements;
})();
"#;

#[test]
fn test_s10k3_iframe_registry_pins() {
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
        "<html><body><div id=\"hs\"><template shadowrootmode=\"open\" shadowrootcustomelementregistry></template></div><div id=\"hl\"><template shadowrootmode=\"open\"></template></div><template id=\"tp\"><div id=\"hp\"><template shadowrootmode=\"open\" shadowrootclonable shadowrootcustomelementregistry></template></div></template><template id=\"tq\"><div id=\"hq\"><template shadowrootmode=\"open\" shadowrootclonable></template></div></template></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/custom-elements/registries/pins-k3.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K3_IFRAME_REG_PINS_JS).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__p1").unwrap().value,
        "true",
        "刀③ ① no-src iframe 首访问：fallback win 物化 + R365 doc 槽接线（doc.customElementRegistry === win.customElements）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__p2").unwrap().value,
        "true",
        "刀③ ② DSR-attr transfer 印记：克隆入主文档 SR getter null sticky，无属性根 live"
    );
    assert_eq!(
        sandbox.execute("globalThis.__p3").unwrap().value,
        "true",
        "刀③ ③ adopt 矩阵：DSR-attr sticky null＋SR 子解析 null，DSR-no-attr/imperative null 重钉 iframe doc default"
    );
    assert_eq!(
        sandbox.execute("globalThis.__p4").unwrap().value,
        "true",
        "刀③ ④ 主文档 hydrate：declarative_shadow_deep_json_doc r 键 → DSR-attr 根 getter null，无属性根 live"
    );
}
