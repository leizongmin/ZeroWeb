// js_dom_bridge 测试切片 28。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

#[test]
fn wc_m1_ce_define_domexception_types() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"d\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // WC-M1 切片 2b（web-components goal）：define/whenDefined 异常类型 DOMException 化
    //（spec `dom-customelementregistry-define`——SyntaxError/NotSupportedError）+
    // PCEN 产生式（非 ASCII 码点名合法）+ CustomElementRegistry 接口对象。
    let js = r#"
var __o = [];
class A extends HTMLElement {}
try { customElements.define('BAD', A); }
catch (e) { __o.push('invalid:' + e.name + ':' + (e instanceof DOMException)); }
try { customElements.define('wc-ok-el', A); customElements.define('wc-ok-el', A); }
catch (e2) { __o.push('dup:' + e2.name + ':' + (e2 instanceof DOMException)); }
try {
  // 非 ASCII PCENChar（spec 允许）——旧 ASCII-only regex 误拒。
  class B extends HTMLElement {}
  customElements.define('wc-\u00e9l\u00e9ment', B);
  __o.push('pcen:' + (customElements.get('wc-\u00e9l\u00e9ment') === B));
} catch (e4) { __o.push('pcen:THROW:' + e4.name); }
__o.push('iface:' + (customElements instanceof CustomElementRegistry));
__o.push('proto-define:' + typeof CustomElementRegistry.prototype.define);
try { new CustomElementRegistry(); __o.push('new:THREW-NOTHING'); }
catch (e5) { __o.push('new:' + (e5 instanceof TypeError)); }
__o.join('|');
"#;
    let out = sandbox.execute(js).unwrap().value;
    assert_eq!(
        out, "invalid:SyntaxError:true|dup:NotSupportedError:true|pcen:true|iface:true|proto-define:function|new:true",
        "define/whenDefined 异常类型 + PCEN + CustomElementRegistry 接口对象（spec 对齐）"
    );
}
