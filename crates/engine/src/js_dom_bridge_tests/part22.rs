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
