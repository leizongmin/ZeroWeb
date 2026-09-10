// js_dom_bridge 测试切片 29。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。

/// WC-M1 切片 3（web-components goal）：CEReactions 反应链全语义面。
/// 覆盖：① setAttribute/反射 setter 的 attributeChanged（4 参签名带 namespace=null）；
/// ② NS 变体（setAttributeNS/removeAttributeNS）带 ns 派发；③ 跨文档
/// disconnected→adopted→connected 序（spec `concept-node-adopt` + custom-element-reactions）；
/// ④ template content 视图的 mutation 面 + contents owner document。
#[test]
fn wc_m1_ce_reactions_and_adopt() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 主沙箱（v8 native 桥）：主文档 handle proxy 面。
    let js = r#"
var out = [];
var log = [];
function L(s) { log.push(s); }
class Ce extends HTMLElement {
  static get observedAttributes() { return ['id', 'aria-label']; }
  attributeChangedCallback(n, o, v, ns) { L('attr:' + n + ':' + o + '->' + v + ':' + ns); }
  connectedCallback() { L('connected'); }
  disconnectedCallback() { L('disconnected'); }
  adoptedCallback(od, nd) { L('adopted:' + (od === document ? 'main' : 'other') + '>' + (nd === document ? 'main' : 'other')); }
}
customElements.define('wc-ce-el', Ce);
function snap(name) { out.push(name + '[' + log.join(',') + ']'); log.length = 0; }

// A: setAttribute 反应 + 4 参 namespace=null
var a = document.createElement('wc-ce-el');
a.setAttribute('id', 'foo');
a.setAttribute('data-lang', 'en'); // unobserved → 无派发
a.setAttribute('id', 'bar');
snap('A-setattr');

// B: IDL 反射 setter（id → content attribute）
a.id = 'baz';
snap('B-id-reflection');

// C: NS 变体——移除（newValue null）+ 新增（old null），未 observed 的 data-x 不派发
try {
  a.setAttributeNS(null, 'data-x', '1');           // 未 observed → 不派发
  a.removeAttributeNS(null, 'id');                 // observed 移除 → 派发（old baz, new null）
  a.setAttributeNS(null, 'id', 'nsid');            // observed 新增 → 派发（old null）
} catch (eC1) { log.push('c1:' + eC1.name); }
snap('C-ns-set');

// D: 移除派发（newValue null）
a.removeAttribute('id');
snap('D-remove');

// E: 跨文档移动——主文档插入 → createHTMLDocument 移动 → disconnected/adopted/connected
var e = document.createElement('wc-ce-el');
document.body.appendChild(e);
log.length = 0;
var fdoc = document.implementation.createHTMLDocument('f');
fdoc.documentElement.appendChild(e);
snap('E-crossdoc-move');

// F: template content 视图 mutation 面 + contents owner document
var t = document.createElementNS('http://www.w3.org/1999/xhtml', 'template');
var tdoc = t.content.ownerDocument;
out.push('F-owner[' + (tdoc && tdoc !== document) + ']');
if (!tdoc.documentElement) tdoc.appendChild(tdoc.createElement('html'));
var f = document.createElement('wc-ce-factory-el');
class CeF extends HTMLElement { connectedCallback() { L('f-connected'); } adoptedCallback() { L('f-adopted'); } disconnectedCallback() { L('f-disconnected'); } }
customElements.define('wc-ce-factory-el', CeF);
var inst = document.createElement('wc-ce-factory-el');
document.body.appendChild(inst);
log.length = 0;
tdoc.documentElement.appendChild(inst);
snap('F-tpldoc-move');

out.join(' ; ');
"#;
    let out = sandbox.execute(js).unwrap().value;
    assert_eq!(
        out,
        "A-setattr[attr:id:null->foo:null,attr:id:foo->bar:null] ; \
         B-id-reflection[attr:id:bar->baz:null] ; \
         C-ns-set[attr:id:baz->null:null,attr:id:null->nsid:null] ; \
         D-remove[attr:id:nsid->null:null] ; \
         E-crossdoc-move[disconnected,adopted:main>other,connected] ; \
         F-owner[true] ; \
         F-tpldoc-move[f-disconnected,f-adopted,f-connected]",
        "CE 反应链 + 4 参 attributeChanged + 跨文档 adopted 序（spec 对齐）"
    );
}


/// WC-M1 切片 4（web-components goal）：customized built-ins + whenDefined 真等待 +
/// registry 按 document 隔离。
/// 覆盖：① createElement(localName, {is}) 升级（ctor 体 + is 内容属性）；
/// ② connected/attributeChanged 反应链对 customized built-in 生效；③ new klass()
/// 产生真实元素（localName = extends tag，prototype = klass.prototype）；
/// ④ whenDefined define 前挂起、define 后 microtask resolve；⑤ detached doc 的
/// createElement 不触发主 registry 升级（spec look up a custom element registry）。
#[test]
fn wc_m1_ce_customized_builtins() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let js = r#"
var out = [];
var log = [];
class CbiEl extends HTMLElement {
  static get observedAttributes() { return ['alt']; }
  constructor() { super(); log.push('constructed'); }
  connectedCallback() { log.push('connected'); }
  disconnectedCallback() { log.push('disconnected'); }
  attributeChangedCallback() { log.push('attr'); }
}
customElements.define('wc-cbi-img', CbiEl, { extends: 'img' });

// A: createElement(tag, {is}) 升级 + is 内容属性
var a = document.createElement('img', { is: 'wc-cbi-img' });
out.push('A-ctor:' + (a.constructor === CbiEl));
out.push('A-is:' + a.getAttribute('is'));
out.push('A-log:' + log.join(','));
log.length = 0;

// B: connected + attributeChanged 反应
document.body.appendChild(a);
out.push('B-append:' + log.join(','));
log.length = 0;
a.setAttribute('alt', 'x');
out.push('B-set:' + log.join(','));
log.length = 0;

// C: new klass() 真实元素
var c = new CbiEl();
out.push('C-new:' + (c.nodeType === 1) + ':' + (c.tagName === 'IMG') + ':' + (Object.getPrototypeOf(c) === CbiEl.prototype) + ':' + (c.getAttribute('is') === 'wc-cbi-img'));

// D: whenDefined 真等待
var wdState = 'pending';
customElements.whenDefined('wc-wd-el').then(function () { wdState = 'resolved'; });
class WdEl extends HTMLElement {}
customElements.define('wc-wd-el', WdEl);
out.push('D-sync:' + wdState);
Promise.resolve().then(function () { out.push('D-micro:' + wdState); });

// E: detached doc 的 createElement 不升级（registry 隔离）
var ddoc = document.implementation.createHTMLDocument('d');
var de = ddoc.createElement('wc-cbi-unregistered');
out.push('E-detached:' + (de.constructor !== CbiEl) + ':tag=' + de.tagName);

// F: autonomous define 后已有同名元素升级（spec define upgrade step）
var pre = document.createElement('wc-pre-exist');
document.body.appendChild(pre);
class PreEl extends HTMLElement { constructor() { super(); log.push('pre-constructed'); } }
customElements.define('wc-pre-exist', PreEl);
out.push('F-upgrade:' + (pre.constructor === PreEl) + ':' + log.join(','));
log.length = 0;

out.join(' ; ');
"#;
    let out = sandbox.execute(js).unwrap().value;
    // D-micro：microtask 在 execute turn 结束后跑——第二段 execute 读结果。
    let micro = sandbox.execute("'D-micro:' + wdState").unwrap().value;
    assert_eq!(
        out,
        "A-ctor:true ; A-is:wc-cbi-img ; A-log:constructed ; \
         B-append:connected ; B-set:attr ; \
         C-new:true:true:true:true ; \
         D-sync:pending ; \
         E-detached:true:tag=WC-CBI-UNREGISTERED ; \
         F-upgrade:true:constructed,constructed,attr,connected,constructed,attr,connected,pre-constructed",
        "customized built-ins + whenDefined + registry 隔离（spec 对齐）"
    );
    assert_eq!(micro, "D-micro:resolved",
        "whenDefined 在 define 后的 microtask resolve");
}
