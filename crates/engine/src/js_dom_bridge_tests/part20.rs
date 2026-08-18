// js-dom M4 events 轮次测试（R111/R112：once 调用前移除 / 派发中移除跳过 / listener
// 异常上报 onerror / checkbox·radio 激活后 input+change / detached doc·解析元素事件面 /
// parse_html_element_json path 字段）。

#[test]
fn test_event_once_removed_before_invoke_r111() {
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
        "<html><body><button id=\"b\">x</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // spec inner invoke 步骤 4：once listener 调用前移除（remove then call）——listener 内
    // 嵌套 dispatchEvent 不重入本 listener（WPT remove-all-listeners "Nested usage of once
    // listeners"）。深度守卫：重入会 1000 次自派发超时/爆栈，前置移除使恰好 1 次。
    sandbox
        .execute(
            "globalThis.__n = 0;\
             var el = document.getElementById('b');\
             el.addEventListener('tick', function self() {\
               __n++;\
               if (__n < 5) el.dispatchEvent(new Event('tick'));\
             }, { once: true });\
             el.dispatchEvent(new Event('tick'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("__n").unwrap().value, "1");
}

#[test]
fn test_event_listener_removed_during_dispatch_skipped_r111() {
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // spec inner invoke「if listener's removed is true, continue」——listener1 内移除
    // listener2，listener2 不得触发（WPT remove-all-listeners "Removing all listeners and
    // then adding a new one"）。
    sandbox
        .execute(
            "globalThis.__seen = [];\
             var el = document.getElementById('d');\
             var l2 = function () { __seen.push('l2'); };\
             el.addEventListener('go', function () { __seen.push('l1'); el.removeEventListener('go', l2); });\
             el.addEventListener('go', l2);\
             el.dispatchEvent(new Event('go'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("__seen.join(',')").unwrap().value, "l1");
}

#[test]
fn test_click_activation_fires_input_and_change_when_attached_r112() {
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
        "<html><body><input id=\"cb\" type=\"checkbox\"></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // spec HTML input activation behavior 末段：fire input + change at el（attached 才派发，
    // detached 不派发——WPT Event-dispatch-detached-input-and-change）。
    sandbox
        .execute(
            "globalThis.__fired = [];\
             var cb = document.getElementById('cb');\
             cb.addEventListener('input', function () { __fired.push('input'); });\
             cb.addEventListener('change', function () { __fired.push('change'); });\
             cb.click();\
             var detached = document.createElement('input');\
             detached.type = 'checkbox';\
             detached.addEventListener('input', function () { __fired.push('d-input'); });\
             detached.addEventListener('change', function () { __fired.push('d-change'); });\
             detached.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("__fired.join(',')").unwrap().value,
        "input,change"
    );
}

#[test]
fn test_click_activation_input_change_bubbles_and_checked_rolled_back_on_prevent_r112() {
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
        "<html><body><form id=\"f\"><input id=\"r1\" type=\"radio\" name=\"g\"></form></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // input/change 冒泡到 form（bubbles:true）+ dispatchEvent(new MouseEvent('click')) 同样
    // 触发激活链；preventDefault 回滚 checked 后不再派发（canceled activation）。
    sandbox
        .execute(
            "globalThis.__bubbled = [];\
             var form = document.getElementById('f');\
             form.addEventListener('input', function () { __bubbled.push('f-input'); });\
             form.addEventListener('change', function () { __bubbled.push('f-change'); });\
             var r1 = document.getElementById('r1');\
             r1.dispatchEvent(new MouseEvent('click'));\
             var ok = r1.checked === true;\
             var cb = document.createElement('input');\
             cb.type = 'checkbox';\
             document.body.appendChild(cb);\
             var evs = [];\
             cb.addEventListener('click', function (e) { e.preventDefault(); });\
             cb.addEventListener('input', function () { evs.push('input'); });\
             cb.click();\
             globalThis.__rollback = (cb.checked === false) + ':' + evs.length;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("__bubbled.join(',')").unwrap().value, "f-input,f-change");
    // preventDefault：checked 回滚 false + input/change 不派发。
    assert_eq!(sandbox.execute("__rollback").unwrap().value, "true:0");
}

#[test]
fn test_detached_document_event_surface_r112() {
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
        "<html><body><p id=\"p\">x</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // detached doc（new Document / createHTMLDocument）三面：doc 级 getElementsByTagName/
    // getElementById + doc/docEl/body addEventListener/dispatchEvent（本地 capture/bubble）。
    sandbox
        .execute(
            "globalThis.__order = [];\
             var d = document.implementation.createHTMLDocument();\
             d.body.innerHTML = '<div id=\"x\">hi</div>';\
             var x = d.getElementById('x');\
             d.addEventListener('ping', function () { __order.push('doc'); });\
             d.documentElement.addEventListener('ping', function (e) { __order.push('html:' + e.eventPhase); }, true);\
             d.body.addEventListener('ping', function (e) { __order.push('body:' + e.eventPhase); }, true);\
             x.addEventListener('ping', function (e) { __order.push('x:' + e.eventPhase); });\
             x.dispatchEvent(new Event('ping', { bubbles: true }));\
             globalThis.__x = x ? x.tagName : 'null';",
        )
        .unwrap();
    // 捕获 doc→html→body（eventPhase 1，远→近——spec capture 沿祖先链从最外层向 target），
    // target x（2），bubble 末端 doc 站（非捕获 listener——doc 是链最外层，bubble 最后触发）。
    // doc 站 capture 期无 listener（只注册了非捕获），故序从 html:1 起。
    assert_eq!(
        sandbox.execute("__order.join(',')").unwrap().value,
        "html:1,body:1,x:2,doc"
    );
    assert_eq!(sandbox.execute("__x").unwrap().value, "DIV");
}

#[test]
fn test_parse_html_query_path_field_r112() {
    // parse_html_element_json 的 path 字段（R112）：祖先身份键数组（根→父，\x1f 分隔）——
    // detached 解析元素事件派发的祖先链来源。直接测 Rust 侧序列化（不经沙箱；engine 无
    // serde_json 依赖，JSON 串按字面断言）。
    let json = crate::js_dom_bridge::parse_html_element_json(
        "<html><body><table id=\"t\"><tbody id=\"tb\"><tr id=\"r\"><td id=\"c\">x</td></tr></tbody></table></body></html>",
        "#c",
        false,
    );
    // path 在 JSON 里是单串（\x1f 分隔——json_str 转义为 ）：html/body（sig——无 id）
    // → id:t → id:tb → id:r。按字面断言（引擎测试无 serde_json 依赖）。
    assert!(json.contains("sig:HTML|"), "html sig key missing: {json}");
    assert!(json.contains("sig:BODY|"), "body sig key missing: {json}");
    assert!(
        json.contains("id:t\\u001fid:tb\\u001fid:r\""),
        "id chain missing: {json}"
    );
}
