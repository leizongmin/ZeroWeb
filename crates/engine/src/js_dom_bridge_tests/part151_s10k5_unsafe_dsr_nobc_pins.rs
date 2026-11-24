// js_dom_bridge 测试切片 151。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// s10-registries 刀⑤ 常驻钉测（2026-11-24）——setHTMLUnsafe DSR 物化直通面＋
/// template contents 无 BC 文档域 null registry 链（wpt-data 探针页属 gitignored
/// per-clone 刮擦面，按刀①协议用后即删，本钉测即常驻回归载体）：
/// ① `setHTMLUnsafe` 含 `shadowrootcustomelementregistry` 声明式根：镜像子
/// customElementRegistry 为 null（sticky 印——part13 `_zwSanSetHtmlImpl` 无消毒面
/// 直通臂不再经 serialize+reparse 有损中转〔mock 影子字段不进 light 序列化，
/// template 消失形〕，setHTMLUnsafe 走完整 unsafe 写链：视图 SR 挂载→引擎物化→
/// handle 重建→`_zwTransferDeclarativeSR` 转登＋decl null 印随转）；
/// ② `template.content.cloneNode(true)` 克隆宿主（R145 `_wcRebuildAsHandle` 产物，
/// `_zwCeNoBc` 无 BC 文档域印）：registry 解析 fallback 返 null（spec template
/// contents owner document 无 browsing context），attachShadow 显式 null → SR
/// innerHTML 子沿父链 null；宿主再克隆（trap 克隆 `_zwCeNoBc` 印传播＋R5433
/// imperative own 槽兜底镜像）→ cloneHost/cloneSR/克隆 SR 子全 null。
/// 根因链与 k5 探针实证见 evidence `dc2-s10k4-registries-knife4-verdict.md` 残面节
/// （刀⑤ 判读档承之）。
const S10K5_UNSAFE_DSR_NOBC_PINS_JS: &str = r#"
globalThis.__k5a = (function () {
  var c = document.createElement('div');
  c.setHTMLUnsafe('<div><template shadowrootmode="open" shadowrootcustomelementregistry><div></div></template></div>');
  var top = c.firstElementChild;
  if (!top) return 'notop';
  var sr = top.shadowRoot;
  if (!sr) return 'nosr';
  var d2 = sr.firstElementChild;
  if (!d2) return 'nomirror';
  if (d2.customElementRegistry !== null) return 'mirror-reg-not-null';
  return true;
})();
globalThis.__k5b = (function () {
  var tpl = document.createElement('template');
  tpl.innerHTML = '<div></div>';
  var host = tpl.content.cloneNode(true).firstChild;
  if (!host) return 'nohost';
  if (host.customElementRegistry !== null) return 'host-reg-not-null';
  var sr = host.attachShadow({ mode: 'open', customElementRegistry: null, clonable: true });
  if (sr.customElementRegistry !== null) return 'sr-reg-not-null';
  sr.innerHTML = '<span></span>';
  var sp = sr.querySelector('span');
  if (!sp) return 'nospan';
  if (sp.customElementRegistry !== null) return 'span-reg-not-null';
  var cloneHost = host.cloneNode(true);
  if (cloneHost.customElementRegistry !== null) return 'clone-host-reg-not-null';
  var cloneSr = cloneHost.shadowRoot;
  if (!cloneSr) return 'noclonesr';
  if (cloneSr.customElementRegistry !== null) return 'clone-sr-reg-not-null';
  var cloneSpan = cloneSr.querySelector('span');
  if (!cloneSpan) return 'noclonespan';
  if (cloneSpan.customElementRegistry !== null) return 'clone-span-reg-not-null';
  return true;
})();
"#;

#[test]
fn test_s10k5_unsafe_dsr_nobc_pins() {
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
        "<html><body><div id=\"k5page\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/custom-elements/registries/pins-k5.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox.execute(S10K5_UNSAFE_DSR_NOBC_PINS_JS).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__k5a").unwrap().value,
        "true",
        "刀⑤ ① setHTMLUnsafe DSR：镜像子 registry null（直通臂免 serialize 有损中转＋转登 decl 印）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__k5b").unwrap().value,
        "true",
        "刀⑤ ② contents 克隆链：_zwCeNoBc 印 fallback null＋attachShadow null 全链传播＋克隆三面 null"
    );
}
