// js-dom M4 events 轮次测试（R111/R112/R113：once 调用前移除 / 派发中移除跳过 / listener
// 异常上报 onerror / checkbox·radio 激活后 input+change / detached doc·解析元素事件面 /
// parse_html_element_json path 字段 / handleEvent 非 callable TypeError 上报 / prefixed
// animation handler 别名映射 / handle-based CSSStyleSheet）。

#[test]
fn test_handle_event_not_callable_reports_typeerror_r113() {
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

    // spec inner invoke 步骤 1-2（WebIDL EventListener 非 nullable callback）：handleEvent
    // 为 null / 非 callable 值都抛 TypeError 经「report the exception」上报（fire error event
    // at window）——WPT EventListener-handleEvent「throws if `handleEvent` is falsy and not
    // callable」/「thruthy and not callable」。window 'error' listener 收 ErrorEvent（error
    // 字段是原始 TypeError），后续 listener 不受影响。
    let out = sandbox
        .execute(
            "var el = document.getElementById('d');\
             var seen = [];\
             window.addEventListener('error', function (e) {\
               seen.push(e.type + ':' + String(e.message) + ':' + (e.error instanceof TypeError));\
             });\
             el.addEventListener('foo', { handleEvent: null });\
             el.dispatchEvent(new Event('foo'));\
             var afterFoo = seen.length;\
             el.addEventListener('bar', { get handleEvent() { return 42; } });\
             el.dispatchEvent(new Event('bar'));\
             var afterBar = seen.length;\
             JSON.stringify(seen) + '|foo:' + afterFoo + '|bar:' + afterBar;",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "[\"error:Failed to execute 'addEventListener' on 'EventTarget': parameter 2's 'handleEvent' property is not a function.:true\",\"error:Failed to execute 'addEventListener' on 'EventTarget': parameter 2's 'handleEvent' property is not a function.:true\"]|foo:1|bar:2",
        "handleEvent null/42 都须各上报一个 TypeError error 事件"
    );
}

#[test]
fn test_prefixed_animation_handler_alias_r113() {
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

    // spec HTML「event handlers on elements…」表的 webkit 前缀族：`onwebkitanimationend`
    // 的 event handler event type 是 camelCase `webkitAnimationEnd`——handler setter 经
    // _ZW_PREFIXED_HANDLER_TYPES 映射注册，与 addEventListener 同 type 键触发；getter
    // 同映射读回；非别名（onanimationend 独立）。
    let out = sandbox
        .execute(
            "var el = document.getElementById('d');\
             var hits = { h: 0, l: 0, plain: 0 };\
             el.onwebkitanimationend = function () { hits.h++; };\
             el.addEventListener('webkitAnimationEnd', function () { hits.l++; });\
             el.onanimationend = function () { hits.plain++; };\
             var readback = el.onwebkitanimationend === undefined ? 'null' : 'fn';\
             el.dispatchEvent(new Event('webkitAnimationEnd'));\
             el.dispatchEvent(new Event('animationend'));\
             var cleared = el.onwebkitanimationend = null;\
             el.dispatchEvent(new Event('webkitAnimationEnd'));\
             'h:' + hits.h + ',l:' + hits.l + ',plain:' + hits.plain + ',read:' + readback + ',cleared:' + (cleared === null);",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "h:1,l:2,plain:1,read:fn,cleared:true",
        "prefixed handler 与 listener 同键触发（handler 1 次 + listener 2 次含清除后再派发不触发 handler）、非别名独立、getter 读回、置 null 清除"
    );
}

#[test]
fn test_style_sheet_from_handle_style_element_r113() {
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

    // handle-based <style>（createElement 后 append——CSS-in-JS / WPT prefixed-animation
    // 形态）：.sheet 返 CSSStyleSheet，cssRules 初始经 `__zw_style_rules_handle`（host 从
    // mutation 历史取 SetTextOnHandle 文本），selectorText/cssText 可读。规则写回走
    // SetTextOnHandle（本测只读 + length，写回路径由 shim flushToOwner 覆盖）。
    let out = sandbox
        .execute(
            "var st = document.createElement('style');\
             st.textContent = 'div { color: rgb(1, 2, 3); }';\
             document.head.appendChild(st);\
             var sheet = st.sheet;\
             var n = sheet ? sheet.cssRules.length : -1;\
             var sel0 = n > 0 ? sheet.cssRules[0].selectorText : 'none';\
             var css0 = n > 0 ? sheet.cssRules[0].cssText : 'none';\
             'has:' + (sheet !== null && sheet !== undefined) + ',n:' + n + ',sel:' + sel0 + ',css:' + css0;",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "has:true,n:1,sel:div,css:div { color: rgb(1, 2, 3) }",
        "handle-based style.sheet.cssRules 须从 mutation 历史解析出规则"
    );
}

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


#[test]
fn test_window_event_shadow_suppression_r114() {
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
        "<html><body><div id=\"host\"><span id=\"light\">x</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // spec HTML current event：listener 节点 root 是 shadow root 时 window.event 为 undefined
    //（shadow 段抑制）；composed 冒泡跨边界到 host 后恢复可见（host listener 见 event）。
    // WPT event-global "target is in a shadow tree (dispatched inside shadow tree)"。
    let out = sandbox
        .execute(
            "var host = document.getElementById('host');             var root = host.attachShadow({ mode: 'closed' });             var span = document.createElement('span');             root.appendChild(span);             var seq = [];             span.addEventListener('test', function (e) {               seq.push('shadow:' + (window.event === undefined ? 'undef' : (window.event === e ? 'ev' : 'other')));             });             host.addEventListener('test', function (e) {               seq.push('host:' + (window.event === e ? 'ev' : 'other'));             });             span.dispatchEvent(new Event('test', { composed: true, bubbles: true }));             var after = window.event === undefined ? 'undef' : 'set';             var nonComposed = [];             span.addEventListener('iso', function (e) {               nonComposed.push(window.event === undefined ? 'undef' : 'set');             });             host.addEventListener('iso', function (e) {               nonComposed.push('host-fired');             });             span.dispatchEvent(new Event('iso', { composed: false, bubbles: true }));             seq.join(',') + '|after:' + after + '|iso:' + nonComposed.join(',');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "shadow:undef,host:ev|after:undef|iso:undef",
        "shadow 段 window.event 抑制 + host 恢复 + 非 composed 不跨边界（host 不触发）"
    );
}

#[test]
fn test_eventtarget_dispatch_on_property_and_window_event_r114() {
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

    // XHR : EventTarget（原型链）+ dispatchEvent 派发 window.event（HTML current event 对
    // 非 DOM EventTarget 同样生效）+ on* 属性 handler 同 fire（WPT event-global (2)）。
    // shadow root getElementById（NonElementParentNode——innerHTML 解析子树按 id 查）。
    let out = sandbox
        .execute(
            "var x = new XMLHttpRequest();             var parts = [];             parts.push(typeof x.dispatchEvent + ':' + typeof x.addEventListener);             x.onload = function (e) { parts.push('onload:' + (e === window.event ? 'cur' : 'other')); };             x.dispatchEvent(new Event('load'));             parts.push('after:' + (window.event === undefined ? 'undef' : 'set'));             var host = document.getElementById('d');             var root = host.attachShadow({ mode: 'open' });             root.innerHTML = \"<input id='si'><b id='bi'>t</b>\";             var si = root.getElementById('si');             parts.push('gebi:' + (si && si.tagName || 'none') + ':' + (root.getElementById('bi') ? 'bi-ok' : 'bi-miss'));             parts.push('focus:' + typeof (si && si.focus));             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "function:function|onload:cur|after:undef|gebi:INPUT:bi-ok|focus:function",
        "XHR EventTarget 面 + window.event 派发 + shadow getElementById + 解析子 focus"
    );
}



#[test]
fn test_iframe_content_document_r115() {
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
        "<html><body><iframe src=\"/common/dummy.xml\"></iframe></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/dom/nodes/x.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    // R115：webview 宿主的同步 `__zw_fetch`（headless 契约）——单测 stub
    //（wire：__zwfr: status \x1f statusText \x1f headers \x1f body）。
    use zero_script_sandbox::Sandbox as _ZwSandbox;
    _ZwSandbox::register_callback(
        &mut sandbox,
        "__zw_fetch",
        Box::new(|_args: &[String]| -> String {
            "__zwfr:200\u{1f}OK\u{1f}\u{1f}<foo>Dummy XML document</foo>".to_string()
        }),
    );

    // 静态 `<iframe src>` 的 contentDocument/contentWindow：doc 加载（documentElement.textContent）、
    // XML createElement 保大小写 + namespaceURI null + instanceof win.Element、createElementNS 的
    // validate-and-extract（非法名/保留绑定抛 DOMException）、defaultView 回指 contentWindow。
    // WPT Document-createElement（147P/0F）+ Document-createElementNS（596P/0F）同语义。
    let out = sandbox
        .execute(
            "var f = document.querySelector('iframe');             var doc = f.contentDocument;             var win = f.contentWindow;             var parts = [];             parts.push('de:' + (doc && doc.documentElement ? doc.documentElement.textContent : 'null'));             parts.push('dv:' + (doc.defaultView === win));             var e1 = doc.createElement('Abc');             parts.push('ce:' + e1.localName + '/' + e1.tagName + '/' + e1.namespaceURI);             parts.push('inst:' + (e1 instanceof win.Element));             var e2 = doc.createElementNS('http://example.com/', 'p:l');             parts.push('ns:' + e2.prefix + '/' + e2.localName + '/' + e2.namespaceURI);             var threw = '';             try { doc.createElementNS(null, 'p:l'); } catch (eN) { threw = eN.name; }             parts.push('nserr:' + threw);             var threw2 = '';             try { doc.createElementNS('http://example.com/', 'xmlns'); } catch (eX) { threw2 = eX.name; }             parts.push('xmlnserr:' + threw2);             var threw3 = '';             try { doc.createElement(''); } catch (eV) { threw3 = eV.name; }             parts.push('invalid:' + threw3);             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "de:Dummy XML document|dv:true|ce:Abc/Abc/null|inst:true|ns:p/l/http://example.com/|nserr:NamespaceError|xmlnserr:NamespaceError|invalid:InvalidCharacterError",
        "iframe contentDocument：XML doc 语义（保大小写/ns null/instanceof/validate-and-extract/defaultView）"
    );
}

#[test]
fn test_attribute_case_and_ns_metadata_r116() {
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

    // R116：非 NS 属性族 HTML 小写 + 空名 InvalidCharacterError + NS 读大小写敏感 + Attr 的
    // prefix/localName/namespaceURI 元数据（setAttributeNS 登记 registry）+ createAttribute
    // 空名/文档类型大小写（WPT attributes.html / case.js / Document-createAttribute 同语义）。
    let out = sandbox
        .execute(
            "var el = document.getElementById('d');             var parts = [];             el.setAttribute('CHEESE', 'x');             parts.push('lower:' + el.hasAttribute('cheese') + ':' + el.getAttribute('cheese'));             parts.push('uppermiss:' + el.hasAttributeNS('', 'CHEESE'));             var threw = '';             try { el.setAttribute('', 'v'); } catch (eA) { threw = eA.name; }             parts.push('empty:' + threw);             el.setAttributeNS('http://FOO', 'abc:def', '1');             parts.push('nsget:' + el.getAttributeNS('http://FOO', 'def'));             parts.push('nscase:' + (el.getAttributeNS('http://FOO', 'DEF') === null));             var attr = el.attributes[el.attributes.length - 1];             parts.push('meta:' + attr.prefix + '/' + attr.localName + '/' + attr.namespaceURI);             parts.push('tc:' + attr.textContent);             var cthrew = '';             try { document.createAttribute(''); } catch (eC) { cthrew = eC.name; }             parts.push('ca-empty:' + cthrew);             var ca = document.createAttribute('MiXeD');             parts.push('ca-lower:' + ca.name);             var xdoc = document.implementation.createDocument(null, null, null);             var cax = xdoc.createAttribute('MiXeD');             parts.push('ca-xml:' + cax.name);             var t1 = 0, t2 = 0;             var tel = document.createElement('foo');             tel.toggleAttribute('tt'); t1 = tel.hasAttribute('tt');             t2 = tel.toggleAttribute('tt');             parts.push('toggle:' + t1 + ':' + t2 + ':' + tel.hasAttribute('tt'));             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "lower:true:x|uppermiss:false|empty:InvalidCharacterError|nsget:1|nscase:true|meta:abc/def/http://FOO|tc:1|ca-empty:InvalidCharacterError|ca-lower:mixed|ca-xml:MiXeD|toggle:true:false:false",
        "属性族：HTML 小写 + 空名异常 + NS 大小写敏感 + Attr NS 元数据 + createAttribute 文档类型语义 + handle toggle presence"
    );
}
