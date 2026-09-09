#[test]
fn wc_m1_factory_shadow_mutation_and_clone_template_content() {
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
        "<html><body><div id=\"test_basic\"><div id=\"host\"><template data-mode=\"open\"><slot id=\"s1\" name=\"slot1\"></slot></template><div id=\"c1\" slot=\"slot1\"></div></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // WPT shadow-dom/resources/shadow-dom.js createTestTree 固定装配（slots-basic）：
    // detached 克隆树 → removeChild(template) → attachShadow → importNode(content) →
    // shadowRoot.appendChild → qSA 走 shadow 树。
    let js = r#"
var __o = [];
try {
  var test_basic = document.getElementById('test_basic');
  var n = {};
  function walk(root) {
    if (root.id) n[root.id] = root;
    var ids = root.querySelectorAll ? root.querySelectorAll('[id]') : [];
    for (var i = 0; i < ids.length; i++) n[ids[i].id] = ids[i];
    var tpls = root.querySelectorAll ? root.querySelectorAll('template') : [];
    for (var t = 0; t < tpls.length; t++) {
      var template = tpls[t];
      var parent = template.parentNode;
      parent.removeChild(template);
      var shadowRoot = parent.attachShadow({mode: template.getAttribute('data-mode')});
      if (template.id) { shadowRoot.id = template.id; n[template.id] = shadowRoot; }
      shadowRoot.appendChild(document.importNode(template.content, true));
      walk(shadowRoot);
    }
  }
  walk(test_basic.cloneNode(true));
  __o.push('s1:' + (n.s1 ? n.s1.id : 'null'));
  __o.push('inShadow:' + (n.s1 ? (n.s1.closest ? 'n/a' : 'ok') : '-'));
  __o.push('shadowKids:' + (n.host && n.host.shadowRoot ? n.host.shadowRoot.childNodes.length : 'n/a'));
} catch (e) { __o.push('ERR:' + (e && e.message)); }
__o.join('|');
"#;
    let out = sandbox.execute(js).unwrap().value;
    assert_eq!(out, "s1:s1|inShadow:ok|shadowKids:1", "createTestTree 全装配不抛 + slot 可达 + shadow 子树挂接");
}

#[test]
fn wc_m1_cycle_guard_no_regression_on_acyclic_trees() {
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
        "<html><body><div id=\"a\"><p id=\"p1\">x<span id=\"s1\">y</span></p></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = r#"
var __o = [];
try {
  var frag = document.createDocumentFragment();
  var d = document.createElement('div'); d.id = 'k';
  var p = document.createElement('p'); p.id = 'q';
  d.appendChild(p); p.appendChild(document.createElement('span'));
  frag.appendChild(d);
  var c = frag.cloneNode(true);
  __o.push('deep-clone:' + c.childNodes.length + ':' + c.childNodes[0].childNodes[0].tagName);
  var imp = document.importNode(c, true);
  __o.push('import:' + imp.childNodes[0].id);
} catch (e) { __o.push('ERR:' + (e && e.message)); }
__o.join('|');
"#;
    let out = sandbox.execute(js).unwrap().value;
    assert_eq!(out, "deep-clone:1:P|import:k", "环守卫对无环树零回归（fragment 深克隆 + importNode）");
}
