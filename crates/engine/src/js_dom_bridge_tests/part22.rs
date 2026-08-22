// js-dom M1 L2-d3a 单元测试——identity 桥基建（R166）：
use super::*;
// ① `_zwMEl` 工厂出口登记（cloneNode/树建产物入桥，幂等首登记者胜）
// ② doc 级 queryBody 真实节点直出点登记（L2-d1 纯 tag 路径产物 === 桥内对象）
// ③ fragment QSA 真实节点优先产物登记（R163 路径）
// ④ 零行为变化：d3a 只登记不归一——各面返回对象与桥前一致（同对象引用）

/// R166（d3a）：`_zwMEl` 产物经桥登记——同节点多次「暴露」桥内恒同一对象；工厂
/// 两次调用产不同节点（不同 key）。用 `document.implementation.createHTMLDocument`
/// 的 createElement/树建路径驱动（createElement 走 handle 域不登记——桥只覆盖
/// mutTree 域；cloneNode/`_zwMBuildBodyTree`/fragment 解析是登记入口）。
#[test]
fn test_node_bridge_registers_muttree_nodes_r166() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id=\"d\">x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             var doc = document.implementation.createHTMLDocument('B');\
             doc.body.innerHTML = '<div id=\"a\"><span id=\"s\">t</span></div>';\
             var first = doc.body.querySelector('div');\
             var again = doc.body.querySelector('div');\
             parts.push('same:' + (first === again));\
             var kids = doc.body.childNodes;\
             parts.push('kids:' + (kids && kids.length ? 'y' : 'n'));\
             parts.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "same:true|kids:y",
        "R166 d3a：L2-d1 真实节点直出 identity 保持（同查询同对象——桥登记不破坏 R158/R164 语义）"
    );
}

/// R166（d3a）：桥 API 直接断言——`_zwBridgeSet` 首登记者胜（重复登记不覆盖）、
/// `_zwBridgeGet` 命中/miss 语义、非对象入参防御（不抛）。
#[test]
fn test_node_bridge_api_semantics_r166() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    let out = sandbox
        .execute(
            "var parts = [];\
             var n1 = { nodeType: 1, nodeName: 'DIV' };\
             var expose1 = { tag: 'first' };\
             var expose2 = { tag: 'second' };\
             _zwBridgeSet(n1, expose1);\
             _zwBridgeSet(n1, expose2);\
             parts.push('first-wins:' + (_zwBridgeGet(n1) === expose1));\
             var miss = _zwBridgeGet({ nodeType: 1 });\
             parts.push('miss:' + (miss === undefined));\
             _zwBridgeSet(null, expose1); _zwBridgeSet(n1, null);\
             parts.push('defensive:ok');\
             parts.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "first-wins:true|miss:true|defensive:ok",
        "R166 d3a：桥 API 首登记者胜 + miss 语义 + 非对象防御"
    );
}

/// R166（d3a）：fragment QSA 真实节点优先路径（R163）登记桥——querySelectorAll
/// 产物 identity 跨查询稳定（R158/R163 语义零回归）。**实证发现记录**：createElement
/// 产物（handle 域）appendChild 进 fragment 后，QSA 键匹配（tag+id+outer）对其
/// miss——handle 元素的 outerHTML 序列化与 mutTree 键形态不同源，回落 wrapper →
/// `q[0] === frag.childNodes[0]` 为 false。这是四对象域 identity 分歧的活样本
///（RFC §1.1），跨域归一由 d3b 消费桥实现，d3a 断言限于零回归面。
#[test]
fn test_node_bridge_fragment_real_nodes_r166() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id=\"d\">x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             var doc = document.implementation.createHTMLDocument('F');\
             var frag = doc.createDocumentFragment();\
             var el = doc.createElement('div'); el.id = 'in-frag';\
             frag.appendChild(el);\
             var q = frag.querySelectorAll('div');\
             parts.push('hit:' + (q && q.length === 1));\
             var q2 = frag.querySelectorAll('div');\
             parts.push('stable:' + (q2[0] === q[0]));\
             parts.push('qs:' + (frag.querySelector('div') === q[0]));\
             parts.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "hit:true|stable:true|qs:true",
        "R166 d3a：fragment QSA identity 跨查询稳定（R158/R163 语义零回归；createElement 跨域分歧记 RFC d3b 面）"
    );
}

/// R167（d3b）：查询产物归一——doc 级 getElementById 产物与树遍历产物（childNodes
/// 下行）**同对象**（旧行为：D 域 wrapper vs C 域节点割裂）。归一经桥消费：
/// `_zwWrapCached` 前置 `_zwMFindRealNode` + `_zwBridgeGet`。
#[test]
fn test_query_unification_doc_getelementby_id_r167() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id=\"d\">x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             var doc = document.implementation.createHTMLDocument('U');\
             doc.body.innerHTML = '<div id=\"a\"><span id=\"s\">t</span></div>';\
             var byId = doc.getElementById('s');\
             var byQsa = doc.querySelectorAll('#s')[0];\
             var byTraverse = doc.body.childNodes[0].childNodes[0];\
             parts.push('idEqQsa:' + (byId === byQsa));\
             parts.push('idEqTraverse:' + (byId === byTraverse));\
             var inner = doc.getElementById('a').querySelector('span');\
             parts.push('elEqTraverse:' + (inner === byTraverse));\
             parts.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "idEqQsa:true|idEqTraverse:true|elEqTraverse:true",
        "R167 d3b：doc/element/遍历三面查询产物 identity 归一（桥消费）"
    );
}

/// R167（d3b）：链派发三阶段——mutTree 节点 dispatchEvent 的祖先链 capture/bubble
///（WPT Event-dispatch-bubbles "In DOMImplementation.createHTMLDocument()" 语义面，
/// R112 单测同源）。**proxy 止链**：B 域 proxy（createElement host）不进入 C 域链
///（lit e2e 双触发教训）。
#[test]
fn test_mel_chain_dispatch_phases_r167() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id=\"d\">x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             var doc = document.implementation.createHTMLDocument('C');\
             doc.body.innerHTML = '<div id=\"wrap\"><span id=\"leaf\">t</span></div>';\
             var leaf = doc.getElementById('leaf');\
             var order = [];\
             doc.addEventListener('ping', function () { order.push('doc'); });\
             doc.documentElement.addEventListener('ping', function (e) { order.push('html:' + e.eventPhase); }, true);\
             doc.body.addEventListener('ping', function (e) { order.push('body:' + e.eventPhase); }, true);\
             leaf.addEventListener('ping', function (e) { order.push('leaf:' + e.eventPhase); });\
             leaf.dispatchEvent(new Event('ping', { bubbles: true }));\
             parts.push(order.join(','));\
             var noBubble = [];\
             var leaf2 = doc.getElementById('wrap');\
             leaf2.addEventListener('x', function () { noBubble.push('t'); });\
             doc.body.addEventListener('x', function () { noBubble.push('body'); });\
             leaf2.dispatchEvent(new Event('x', { bubbles: false }));\
             parts.push('nb:' + noBubble.join(','));\
             parts.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "html:1,body:1,leaf:2,doc|nb:t",
        "R167 d3b：mutTree 链派发三阶段（capture html→body / AT_TARGET / bubble doc）+ 非 bubbles 不上行"
    );
}
