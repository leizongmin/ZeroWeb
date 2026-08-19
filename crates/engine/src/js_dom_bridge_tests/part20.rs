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

#[test]
fn test_child_parent_node_mutation_family_r117() {
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

    // R117：ChildNode.before/after 的 spec viable-sibling 顺序 pre-insert（self 参数移动语义 +
    // 兄弟参数先移除再插不复制）+ replaceWith（handle 路径 + self 作参数重插不丢）+ null/undefined
    // WebIDL 文本转换 + 层级校验（doc 插 Text/Document → HRE；doctype 插元素 → HRE）+
    // Node.prototype 泛型 replaceChild 校验顺序（parent-type → ancestor → NotFound → node 类型）。
    let out = sandbox
        .execute(
            "var parts = [];             var parent = document.createElement('div');             var c = document.createComment('test');             parent.appendChild(c);             c.after('text', c);             parts.push('after-self:' + parent.childNodes.map(function(n){return n.nodeName === '#comment' ? '<!--' : n.data;}).join(''));             var p2 = document.createElement('div');             var x = document.createElement('x'); var y = document.createElement('y');             var c2 = document.createComment('m');             p2.appendChild(x); p2.appendChild(c2); p2.appendChild(y);             c2.before(y, x);             parts.push('before-move:' + p2.childNodes.map(function(n){return n.nodeName === '#comment' ? 'C' : n.tagName;}).join(','));             c2.replaceWith('r');             parts.push('replaceWith:' + p2.childNodes.map(function(n){return n.nodeName === '#comment' ? 'C' : (n.tagName || n.data);}).join(','));             var el = document.createElement('a');             el.append(null, undefined);             parts.push('null-text:' + el.textContent);             var threw = '';             try { document.append(document.createTextNode('t')); } catch (eH) { threw = eH.name; }             parts.push('doc-text:' + threw);             var threw2 = '';             try { el.appendChild(document.implementation.createDocumentType('h','','')); } catch (eD) { threw2 = eD.name; }             parts.push('dt-into-el:' + threw2);             var rf = Node.prototype.replaceChild;             var threw3 = '';             try { rf.call(document.createComment('c'), x, y); } catch (eP) { threw3 = eP.name; }             parts.push('nonparent:' + threw3);             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "after-self:text<!--|before-move:Y,X,C|replaceWith:Y,X,r|null-text:nullundefined|doc-text:HierarchyRequestError|dt-into-el:|nonparent:HierarchyRequestError",
        "mutation 族：viable-sibling 顺序 pre-insert + 移动语义 + replaceWith self 重插 + null 文本 + doc/doctyp 校验 + 泛型校验顺序"
    );
}

// js-dom M4 R118：querySelector CSS 转义（CSS Syntax 4.3.4「consume an escaped code point」）。
// 驱动用例 WPT dom/nodes/ParentNode-querySelector-escapes.html（17P→64P 的四类修复面）：
// ① hex 转义 + 空白终止符（`\30 next` 的终止空格不是组合器边界——_splitComplex 转义感知）
// ② 转义字符属段内（`\,` 不是逗号组边界——_splitSelectorListOf 转义感知；`\.` 字面进 id 值）
// ③ CSS 空白 = 5 ASCII 字符（JS /\s/ 含 U+2003 等 Unicode 空白会把 `# ` 误切分）
// ④ EOF 反斜杠 → U+FFFD；CRLF 序列整体是单个终止符。
// 已知限制（wire 协议深结构，R118 记档）：孤立代理 id 与字面 NUL selector 经
// `to_rust_string_lossy`（WTF-16→UTF-8）必然替换 U+FFFD，never-match 用例不可达。
// https://drafts.csswg.org/css-syntax/#consume-escaped-code-point
#[test]
fn test_query_selector_css_escapes_r118() {
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

    // JS 以 raw string 内嵌（Rust 不二次转义，\ 等由 JS 解释——与 WPT 用例源码同构）。
    let out = sandbox
        .execute(
            r#"var parts = [];
            function q(id, sel) {
              var c = document.createElement('div');
              var k = document.createElement('span');
              k.id = id;
              c.appendChild(k);
              return c.querySelector(sel) === k ? 'HIT' : 'MISS';
            }
            parts.push(q('spaces', '#spaces'));
            parts.push(q('0nextIsWhiteSpace', '#\\30 nextIsWhiteSpace'));
            parts.push(q('0spaceMoreThan6Hex', '#\\000030 spaceMoreThan6Hex'));
            parts.push(q('aBMPRegular', '#\\61 BMPRegular'));
            parts.push(q('spaces', '#spac\\65\r\ns'));
            parts.push(q('hello', '#hel\\6C o'));
            parts.push(q('.comma', '#\\.comma'));
            parts.push(q('.,:!', '#\\.\\,\\:\\!'));
            parts.push(q('-m', '#-\\6d'));
            parts.push(q('test', '#te\\s\\t'));
            parts.push(q('null�', '#null\\0'));
            parts.push(q('null�', '#null\\0000'));
                                    parts.push(q(' id', '#\\2003 id'));
            parts.join('|');"#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "HIT|HIT|HIT|HIT|HIT|HIT|HIT|HIT|HIT|HIT|HIT|HIT|HIT",
        "CSS 转义：hex+空白终止符 / 转义字符属段内（\\, \\.）/ CSS 空白 ASCII 集 / EOF→FFFD / CRLF 单终止符 / 非 ASCII 空白是 ident 字符"
    );
}

// js-dom M4 R119：prepend/replaceChildren 的 handle 容器路径（createElement 元素 /
// DocumentFragment / cloneNode 产物）+ detached doc replaceChildren 三缺口。
// 驱动用例 WPT dom/nodes/ParentNode-prepend.html（8F→21P 全绿）+ ParentNode-replaceChildren.html
// （13F→29P 全绿）。四层修复：① handle 容器 prepend 此前无实现（仅 sel-based）——
// `_prependHandleVariadic` registry 头插（参数序）+ R101 全 handle wire；② handle 容器
// replaceChildren 清空 + 移动记账（旧父每子 removed record、本容器单条合成 record）；
// ③ detached doc replaceChildren：清空 firstChild-while、校验在清空后（whatwg/dom#1045）、
// 字符串参数抛 HRE；④ doc prepend 的 doctype-vs-doctype 校验（spec pre-insert 步骤 6 II）。
// https://dom.spec.whatwg.org/#dom-parentnode-prepend
#[test]
fn test_prepend_replace_children_handle_paths_r119() {
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

    let out = sandbox
        .execute(
            r#"var parts = [];
            var a = document.createElement('div');
            a.prepend('text');
            parts.push('A:' + a.childNodes[0].textContent);
            a.prepend(null);
            parts.push('A-null:' + a.childNodes[0].textContent + ':' + a.childNodes.length);
            var e = document.createElement('div');
            var ec = document.createElement('test');
            e.appendChild(ec);
            e.prepend('t1', 't2');
            parts.push('E:' + e.childNodes[0].textContent + ',' + e.childNodes[1].textContent + ',' + e.childNodes[2].tagName);
            var f = document.createElement('div');
            f.appendChild(document.createElement('test'));
            f.replaceChildren();
            parts.push('F:' + f.childNodes.length);
            var g = document.createElement('div');
            g.appendChild(document.createElement('test'));
            g.replaceChildren(null);
            parts.push('G:' + g.childNodes.length + ':' + g.childNodes[0].textContent);
            var pp = document.createElement('div');
            var moved = document.createElement('m');
            pp.appendChild(moved);
            var g2 = document.createElement('div');
            g2.replaceChildren(moved);
            parts.push('MOVE:' + g2.childNodes.length + ':' + pp.childNodes.length + ':' + (g2.childNodes[0] === moved));
            var doc = document.implementation.createHTMLDocument('title');
            doc.replaceChildren();
            parts.push('DOC-CLEAR:' + doc.childNodes.length);
            var doc2 = document.implementation.createHTMLDocument('title');
            var el = doc2.createElement('a');
            doc2.replaceChildren(el);
            parts.push('DOC-EL:' + doc2.childNodes.length + ':' + (doc2.childNodes[0] === el));
            var threwText = '';
            try { doc2.replaceChildren('text'); } catch (eT) { threwText = eT.name; }
            parts.push('DOC-TEXT:' + threwText);
            parts.join('|');"#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "A:text|A-null:null:2|E:t1,t2,TEST|F:0|G:1:null|MOVE:1:0:true|DOC-CLEAR:0|DOC-EL:1:true|DOC-TEXT:HierarchyRequestError",
        "prepend/replaceChildren handle 路径：文本/null、参数序+identity、清空、移动记账（旧父剔除断链）、doc 清空/doc 元素替换/字符串 HRE"
    );
}

// js-dom M4 R120：getElementsBy* 族 NS 感知匹配 + live collection + named 暴露不对称。
// 驱动用例 WPT dom/nodes 四文件全 100%（Element/Document-getElementsByTagName(NS)）：
// ① `_zwFilterByTagNameNS` 统一匹配器（非 NS 变体：qualifiedName 双方 ASCII 小写 + HTML ns
// 元素 localName 非纯小写永不匹配「uppercase never matches」；NS 变体：localName **原样
// 精确**——createElementNS(HTMLNS,'ABC') 只被 ('HTMLNS','ABC') 命中）② document 级
// `_zwDocAllElements` 枚举（快照 '*' ∪ pending 动态子）③ liveSpec 作用域放行（detached
// 容器上的 element 级集合）④ named 暴露不对称（id 全元素 / name 仅 HTML ns）⑤
// NodeList/HTMLCollection 构造器 + prototype item/namedItem（expando identity 断言）。
// https://dom.spec.whatwg.org/#concept-getelementsbytagname
#[test]
fn test_get_elements_by_tag_name_family_r120() {
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

    let out = sandbox
        .execute(
            r#"var parts = [];
            var el = document.createElement('div');
            var s1 = el.appendChild(document.createElementNS('test', 'st'));
            parts.push('ST-orig:' + el.getElementsByTagName('ST').length);
            parts.push('st:' + el.getElementsByTagName('st').length);
            parts.push('st-identity:' + (el.getElementsByTagName('st')[0] === s1));
            parts.push('ns-star:' + el.getElementsByTagNameNS('test', '*').length);
            parts.push('ns-st:' + el.getElementsByTagNameNS('test', 'st').length);
            parts.push('ns-null:' + el.getElementsByTagNameNS(null, 'st').length);
            var up = el.appendChild(document.createElementNS('http://www.w3.org/1999/xhtml', 'I'));
            parts.push('upper-I:' + el.getElementsByTagName('I').length + ':' + el.getElementsByTagName('i').length);
            parts.push('tagName-ascii:' + el.appendChild(document.createElementNS('http://www.w3.org/1999/xhtml', 'ä')).tagName);
            var l = el.getElementsByTagName('st');
            var live1 = l.length;
            el.appendChild(document.createElementNS('test', 'st'));
            parts.push('live:' + live1 + '->' + l.length);
            var pre = el.appendChild(document.createElement('pre'));
            pre.id = 'px';
            var preNs = el.appendChild(document.createElementNS('', 'pre'));
            preNs.setAttribute('name', 'pn');
            var coll = el.getElementsByTagName('pre');
            parts.push('named-id:' + (coll.namedItem('px') === pre));
            parts.push('named-name-ns:' + (coll.namedItem('pn') === null));
            parts.push('instanceof:' + (l instanceof globalThis.HTMLCollection));
            parts.push('proto-item:' + (globalThis.HTMLCollection.prototype.item !== undefined));
            parts.join('|');"#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "ST-orig:0|st:1|st-identity:true|ns-star:1|ns-st:1|ns-null:0|upper-I:0:0|tagName-ascii:ä|live:1->2|named-id:true|named-name-ns:true|instanceof:true|proto-item:true",
        "getElementsBy*：非 HTML ns 大小写敏感 / NS localName 原样 / uppercase-never-match / ASCII tagName / live collection / id-name 不对称暴露 / 接口构造器"
    );
}

// js-dom M4 R121：Text/Comment 真构造器 + CharacterData 孤立代理保真（JS 侧 data 覆盖缓存）。
// 驱动用例 WPT dom/nodes Text-constructor.html（13F→15P 100%）+ Comment-constructor.html
// （13F→15P 100%）+ CharacterData-surrogates.html（6F→8P 100%）：
// ① new Text(data)/new Comment(data) 真构造（data/nodeValue String 转换 + ownerDocument=
//   document + 原型链 Text.prototype→CharacterData→Node + instanceof 三层）——旧空 stub
//   使 object.data undefined 全簇 fail。
// ② `_zwTextDataCache`（JS Map，键 handle）：wire 层（to_rust_string_lossy，WTF-16→UTF-8）
//   把孤立代理替换 U+FFFD，而 spec 允许 CharacterData 方法按 UTF-16 code unit 偏移**切开
//   代理对**（replaceData/deleteData/insertData 的切开半对在读回时保真）。缓存写双写
//   （JS 保真 + wire 尽力供 host 渲染），读缓存优先（miss 回落 wire）——data/nodeValue/
//   textContent/wholeText/length/appendData/deleteData/insertData/replaceData/substringData
//   十处统一接线。
// https://dom.spec.whatwg.org/#dom-text
#[test]
fn test_text_comment_constructors_and_surrogates_r121() {
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

    let out = sandbox
        .execute(
            r#"var parts = [];
            var t0 = new Text();
            parts.push('t0:' + JSON.stringify(t0.data) + ':' + (t0.ownerDocument === document));
            var t1 = new Text('hi');
            parts.push('t1:' + t1.data + ':' + (t1 instanceof Text) + ':' + (t1 instanceof CharacterData) + ':' + (t1 instanceof Node));
            var c1 = new Comment(null);
            parts.push('c1:' + c1.data + ':' + (c1 instanceof Comment) + ':' + c1.nodeType);
            // 孤立代理保真：createTextNode 走 wire 会 FFFD——new Text 纯 JS 无 wire；
            // createTextNode + replaceData 切开代理对经覆盖缓存保真。
            var meta = String.fromCharCode(0xD83C, 0xDF20);
            var t2 = document.createTextNode(meta + ' test');
            t2.replaceData(1, 4, '--');
            parts.push('surrogate-split:' + JSON.stringify(t2.data).indexOf('ufffd') >= 0 ? 'FFFD' : 'KEPT');
            var t3 = document.createTextNode('abcdef');
            t3.deleteData(1, 2);
            t3.insertData(0, '<');
            t3.appendData('!');
            parts.push('ops:' + t3.data);
            parts.push('substr:' + t3.substringData(1, 3));
            parts.join('|');"#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "t0:\"\":true|t1:hi:true:true:true|c1:null:true:8|KEPT|ops:<adef!|substr:ade",
        "Text/Comment 构造器（data 转换/ownerDocument/原型链）+ CharacterData 孤立代理保真（覆盖缓存）+ 方法族"
    );
}

// js-dom M4 R122：Attr identity 绑定表 + 同名多实例属性覆盖层 + setAttributeNS 校验。
// 驱动用例 WPT dom/nodes attributes.html（36F→全 100%）+ attributes-namednodemap.html
// （2F→8P 100%）：
// ① `_zwAttrBindings`（elKey → Map(限定名 → Attr 对象)）：setAttributeNode/getAttributeNode/
//   setNamedItem/removeNamedItem 的 identity 契约（同一 Attr 对象往返 + ownerElement 绑定/
//   解绑 + removeAttribute 解绑防 InUse 误抛）。
// ② `_zwAttrInstances`（elKey → 有序实例）：host 属性存储按限定名扁平（同 local 多 ns 无法
//   共存），spec 允许 setAttributeNS('ab','attr') + setAttributeNS('kl','attr') 两实例并存；
//   非 NS getAttribute 返第一个 local 匹配；setAttribute 按限定名原位更新。
// ③ `Attr.prototype.value/nodeValue` accessor（_r122V 存储）：attr.value = v 写回传播到
//   ownerElement（spec `dom-attr-value`「change an attribute」）。
// ④ NamedNodeMap 原型方法 identity（`map.item === NamedNodeMap.prototype.item`）+ per-element
//   Proxy 缓存。
// ⑤ setAttributeNS validate-and-extract（InvalidCharacterError/NamespaceError 全分支）+
//   HTML-ns 判定的小写化收窄（非 HTML ns 元素限定名大小写敏感保留）。
// https://dom.spec.whatwg.org/#dom-element-setattributenode
#[test]
fn test_attr_identity_and_multi_instance_r122() {
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

    let out = sandbox
        .execute(
            r#"var parts = [];
            // ① identity：attributes[0] === getAttributeNode；setAttributeNode 绑定往返。
            var el = document.createElement('div');
            el.setAttribute('foo', 'bar');
            var attr = el.attributes[0];
            parts.push('id1:' + (attr === el.getAttributeNode('foo')));
            parts.push('id2:' + (attr === el.getAttributeNodeNS('', 'foo')));
            // removeAttribute 解绑（ownerElement null）→ setAttributeNode 不再 InUse。
            var el2 = document.createElement('div');
            el.removeAttribute('foo');
            parts.push('owner-null:' + (attr.ownerElement === null));
            el2.setAttributeNode(attr);
            parts.push('rebind:' + (attr === el2.getAttributeNode('foo')) + ':' + (attr.ownerElement === el2) + ':' + attr.value);
            // ② 多实例：同 local 不同 ns 并存；非 NS 读返第一 local 匹配。
            var e3 = document.createElement('baz');
            e3.setAttributeNS('ab', 'attr', 'fail');
            e3.setAttributeNS('kl', 'attr', 'pass');
            e3.setAttribute('attr', 'pass');
            parts.push('multi:' + e3.getAttribute('attr') + ':' + e3.getAttributeNS('kl', 'attr')
              + ':' + e3.attributes.length + ':' + e3.attributes[0].namespaceURI + ':' + e3.attributes[1].namespaceURI);
            // ③ Attr.value setter 写回元素。
            var e4 = document.createElement('foo');
            e4.setAttribute('x', 'y');
            var a4 = e4.attributes[0];
            a4.value = 'Y&lt;';
            parts.push('setprop:' + e4.getAttribute('x') + ':' + a4.nodeValue);
            // ④ NamedNodeMap 方法 identity + named 不遮蔽方法。
            var e5 = document.createElement('div');
            var map5 = e5.attributes;
            var foo5 = document.createAttribute('foo');
            map5.setNamedItem(foo5);
            var item5 = document.createAttribute('item');
            map5.setNamedItem(item5);
            parts.push('nnm:' + (map5.foo === foo5) + ':' + (map5.item === globalThis.NamedNodeMap.prototype.item)
              + ':' + (map5.length === 2));
            var rm5 = map5.removeNamedItem('item');
            parts.push('rm:' + (rm5 === item5) + ':' + (map5.length === 1));
            // ⑤ setAttributeNS 校验（NamespaceError / InvalidCharacterError）+ 非 HTML ns 大小写保留。
            var e6 = document.createElement('div');
            var threwN = false, threwI = false;
            try { e6.setAttributeNS('', 'p:l', 'v'); } catch (e) { threwN = e.name === 'NamespaceError'; }
            try { e6.setAttributeNS('a', '1abc', 'v'); } catch (e) { threwI = e.name === 'InvalidCharacterError'; }
            parts.push('valid:' + threwN + ':' + threwI);
            var e7 = document.createElementNS('http://www.example.com', 'foo');
            e7.setAttribute('A', 'test');
            parts.push('case:' + e7.getAttribute('A') + ':' + e7.hasAttribute('A') + ':' + e7.hasAttributeNS('', 'A') + ':' + e7.hasAttributeNS('foo', 'A'));
            // getAttributeNames 多实例合成名剥离。
            var e8 = document.createElement('div');
            e8.setAttribute('foo', 'bar');
            e8.setAttributeNS('', 'FOO', 'bar');
            e8.setAttributeNS('dummy1', 'foo', 'bar');
            e8.setAttributeNS('dummy2', 'dummy:foo', 'bar');
            var gn = e8.getAttributeNames();
            parts.push('names:' + gn.length + ':' + gn.join(',').indexOf(String.fromCharCode(0)) + ':');
            parts.join('|');"#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "id1:true|id2:true|owner-null:true|rebind:true:true:bar|multi:pass:pass:2:ab:kl|setprop:Y&lt;:Y&lt;|nnm:true:true:true|rm:true:true|valid:true:true|case:test:true:true:false|names:4:-1:",
        "Attr identity 绑定 + 多实例 NS 覆盖层 + value 写回 + NamedNodeMap 方法 identity + NS 校验/大小写"
    );
}

// R122 调试探针（诊断 setNamedItem 真 Attr 形态用）——已定位并删除，正式断言在
// test_namednodemap_setnameditem_main_dom_r3022（R122 spec 语义版）。

// js-dom M4 R123：ProcessingInstruction 属性层（WICG declarative-partial-updates——WPT
// dom/nodes/processing-instruction-attributes.html 7P/133F→140P/0F 驱动）。PI 的 data 即
// 属性序列化源：`a="b" x="yy"` ⇔ [['a','b'],['x','yy']]，读写双向同步（setAttribute 族改
// 属性后 data 重序列化；data= 重新解析属性）。断言组：
// ① createPI 属性五件套（hasAttributes/getAttributeNames/getAttribute/hasAttribute/setAttribute/
//    removeAttribute/toggleAttribute）+ data 重序列化（改值原位、新增尾追、移除收缩）
// ② data= 重解析（清空 → 无属性；'blabla=""' → getAttribute('blabla')=''）
// ③ Name production 校验（'=' '>' '/' 空白 → InvalidCharacterError；'$' '_' 合法）
// ④ 大小写敏感（ABC ≠ abc distinct）+ 值转义往返（& " < > 与 element outerHTML 同款全集）
// ⑤ bogus comment innerHTML 派生 PI 视图（html-parser source）+ 主文档 parse 同款
// https://github.com/WICG/declarative-partial-updates
#[test]
fn test_pi_attribute_layer_r123() {
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
        "<html><body><div id=\"d\">x</div><p id=\"n50\"><?processing data?></p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             var pi = document.createProcessingInstruction('t', 'a=\"b\" x=\"yy\"');\
             parts.push('init:' + pi.nodeType + ':' + pi.hasAttributes() + ':' + pi.getAttributeNames().join(',') + ':' + pi.getAttribute('a') + ':' + pi.getAttribute('X'));\
             pi.setAttribute('x', 'yy2');\
             parts.push('mod:' + pi.data + ':' + pi.getAttribute('x'));\
             pi.setAttribute('z', '1');\
             parts.push('add:' + pi.data + ':' + pi.getAttributeNames().join(','));\
             pi.removeAttribute('x');\
             parts.push('rm:' + pi.data + ':' + pi.getAttribute('x') + ':' + pi.getAttributeNames().join(','));\
             pi.data = '';\
             parts.push('clear:' + pi.hasAttributes() + ':' + pi.getAttributeNames().length);\
             pi.data = 'blabla=\"\"';\
             parts.push('reparse:' + pi.getAttribute('blabla') + ':' + pi.getAttribute('BLABLA') + ':' + pi.getAttributeNames().join(','));\
             var threw = '';\
             try { pi.setAttribute('a=', 'v'); } catch (e) { threw = e.name; }\
             parts.push('invalid:' + threw);\
             pi.setAttribute('$', 'd'); pi.setAttribute('a_b', 'e');\
             parts.push('valid:' + pi.getAttribute('$') + ':' + pi.getAttribute('a_b'));\
             pi.setAttribute('ABC', 'up');\
             parts.push('case:' + pi.getAttribute('ABC') + ':' + pi.getAttribute('abc'));\
             pi.setAttribute('esc', 'a<b>&\"c');\
             parts.push('esc:' + pi.data);\
             var tg = pi.toggleAttribute('tk');\
             parts.push('toggle1:' + tg + ':' + pi.getAttribute('tk'));\
             var tg2 = pi.toggleAttribute('tk');\
             parts.push('toggle2:' + tg2 + ':' + pi.hasAttribute('tk'));\
             var div = document.createElement('div');\
             div.innerHTML = '<?t a=\"b\" x=\"yy\"?>';\
             var f = div.firstChild;\
             parts.push('htmlp:' + f.nodeType + ':' + f.target + ':' + f.getAttribute('a'));\
             var n50 = document.getElementById('n50').firstChild;\
             parts.push('mainp:' + n50.nodeType + ':' + n50.target + ':' + n50.getAttribute('a') + ':' + n50.data);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "init:7:true:a,x:b:null|mod:a=\"b\" x=\"yy2\":yy2|add:a=\"b\" x=\"yy2\" z=\"1\":a,x,z|rm:a=\"b\" z=\"1\":null:a,z|clear:false:0|reparse::null:blabla|invalid:InvalidCharacterError|valid:d:e|case:up:null|esc:blabla=\"\" $=\"d\" a_b=\"e\" ABC=\"up\" esc=\"a&lt;b&gt;&amp;&quot;c\"|toggle1:true:|toggle2:false:false|htmlp:7:t:b|mainp:7:processing:null:data",
        "PI 属性层：五件套 + data 双向同步 + 校验 + 大小写敏感 + 转义 + toggle + 双 parse 派生视图"
    );
}

// js-dom M4 R124：class 域 ASCII whitespace 分词语义（spec html-infrastructure
// 「ascii whitespace」——分隔符仅 space/\t/\n/\f/r 五字符，U+00A0/U+2000 系/U+3000 等
// Unicode 空白是**字面类名字符**非分隔符）。WPT
// dom/nodes/getElementsByClassName-whitespace-class-names.html 19F 簇驱动
//（`<span class="&#x00A0;">` 的 class 是合法单字符类名）。断言组：
// ① gEBCN 以单个 Unicode 空白字符为类名可命中（shim 侧 _zwSplitClassList）
// ② classList.contains 对单个 Unicode 空白字符 token 返 true（旧 /\s/ 误拒）
// ③ ~= 属性选择器以 Unicode 空白为字面字符匹配（Rust 侧 split_ascii_whitespace 同源）
// https://html.spec.whatwg.org/multipage/infrastructure.html#ascii-whitespace
#[test]
fn test_ascii_whitespace_class_domain_r124() {
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
        // data-k / data-w 的 Unicode 空白属性值直接以字面字符进入初始 parse 快照（__zw_matches
        // 读快照——同 execute setAttribute 不入快照，与 WPT 用例的静态标记形态一致）。
        "<html><body><span id=\"s1\" class=\"&#x00A0;\">nb</span><span id=\"s2\" class=\"a&#x2003;b c\">em</span><div id=\"d1\" data-k=\"x&#x2003;y\">y</div><div id=\"d2\" data-w=\"n&#x00A0;o\">z</div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             var NBSP = String.fromCharCode(0xA0), EM = String.fromCharCode(0x2003), VT = String.fromCharCode(0x0B);\
             parts.push('gEBCN-nbsp:' + document.getElementsByClassName(NBSP).length + ':' + document.getElementsByClassName(NBSP)[0].id);\
             var s2 = document.getElementById('s2');\
             parts.push('classList-len:' + s2.classList.length + ':' + (s2.classList.item(0) === 'a' + EM + 'b'));\
             parts.push('contains-em-token:' + s2.classList.contains('a' + EM + 'b'));\
             parts.push('contains-nbsp:' + document.getElementById('s1').classList.contains(NBSP));\
             parts.push('contains-ascii-space:' + s2.classList.contains(' '));\
             var d1 = document.getElementById('d1');\
             parts.push('hat-em:' + d1.matches('[data-k~=x' + EM + 'y]'));\
             var d2 = document.getElementById('d2');\
             parts.push('hat-word:' + d2.matches('[data-w~=n' + NBSP + 'o]'));\
             parts.push('hat-ascii-sep-miss:' + d2.matches('[data-w~=o]'));\
             parts.push('sel-vt-class:' + (function () { try { return document.querySelector('.' + VT) === null; } catch (e) { return 'threw:' + e.name; } })());\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "gEBCN-nbsp:1:s1|classList-len:2:true|contains-em-token:true|contains-nbsp:true|contains-ascii-space:false|hat-em:true|hat-word:true|hat-ascii-sep-miss:false|sel-vt-class:true",
        "ASCII whitespace 分词：U+00A0/U+2003 是字面类名字符，gEBCN/classList/~= 三面一致"
    );
}

// js-dom M4 R125：Document.getElementById 动态 id 语义（WPT
// dom/nodes/Document-getElementById.html 12F 簇驱动——spec
// https://dom.specwg.org/#dom-nonelementparentnode-getelementbyid：按**tree order**
// 首个 tree 中 in-document 的 id 命中）。断言组：
// ① setAttribute/removeAttribute 更新 id 后新旧 id 查询即时反映
// ② innerHTML 增删 id 元素可见性
// ③ detached 父容器（未入 doc）内的 id 元素不可见（in-document 门）
// ④ Attr.value 修改 id 同步（element.attributes[0].value = …）
// ⑤ getElementById('') 返 null（tree 中无空 id 元素时）
#[test]
fn test_get_element_by_id_dynamic_id_r125() {
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
        "<html><body><div id=\"log\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             parts.push('t0:' + (document.getElementById('') === null));\
             var t3 = document.createElement('div');\
             t3.setAttribute('id', 'test3');\
             document.body.appendChild(t3);\
             t3.setAttribute('id', 'test3-updated');\
             parts.push('t3-new:' + (document.getElementById('test3-updated') === t3));\
             parts.push('t3-old:' + (document.getElementById('test3') === null));\
             t3.removeAttribute('id');\
             parts.push('t3-rm:' + (document.getElementById('test3-updated') === null));\
             var t6s = document.createElement('div');\
             t6s.setAttribute('id', 'test6');\
             document.createElement('div').appendChild(t6s);\
             parts.push('t6:' + (document.getElementById('test6') === null));\
             var t7 = document.createElement('div');\
             t7.setAttribute('id', 'test7');\
             document.body.appendChild(t7);\
             parts.push('t7-before:' + (document.getElementById('test7') === t7));\
             try {\
               t7.attributes[0].value = 'test7-updated';\
               parts.push('t7-old:' + (document.getElementById('test7') === null));\
               parts.push('t7-new:' + (document.getElementById('test7-updated') === t7));\
             } catch (e) { parts.push('t7-threw:' + e.name); }\
             var t8 = document.createElement('div');\
             t8.setAttribute('id', 'test8-fixture');\
             document.body.appendChild(t8);\
             t8.innerHTML = \"<div id='test8'></div>\";\
             parts.push('t8:' + (document.getElementById('test8') === t8.firstChild));\
             var t9f = document.createElement('div');\
             t9f.setAttribute('id', 'test9-fixture');\
             document.body.appendChild(t9f);\
             var t9 = document.createElement('div');\
             t9.setAttribute('id', 'test9');\
             t9f.appendChild(t9);\
             parts.push('t9-before:' + (document.getElementById('test9') === t9));\
             t9f.innerHTML = '';\
             parts.push('t9-after:' + (document.getElementById('test9') === null));\
             var t8b = document.createElement('div');\
             document.body.appendChild(t8b);\
             t8b.innerHTML = '<div ' + 'id=' + String.fromCharCode(34) + 'test8b' + String.fromCharCode(34) + '></div>';\
             parts.push('t8b:' + (document.getElementById('test8b') === t8b.firstChild));\
             var t15o = document.getElementById('log');\
             var t15m = document.createElement('div');\
             t15o.appendChild(t15m);\
             var t15i = document.createElement('span');\
             t15i.id = 'test15i';\
             t15m.appendChild(t15i);\
             parts.push('t15-in:' + (document.getElementById('test15i') === t15i));\
             t15o.removeChild(t15m);\
             parts.push('t15-out:' + (document.getElementById('test15i') === null));\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "t0:true|t3-new:true|t3-old:true|t3-rm:true|t6:true|t7-before:true|t7-old:true|t7-new:true|t8:true|t9-before:true|t9-after:true|t8b:true|t15-in:true|t15-out:true",
        "getElementById 动态 id：setAttribute/Attr.value/innerHTML 三路更新即时反映 + detached in-doc 门"
    );
}




// js-dom M4 R126：removeChild 的 spec `dom-node-pre-remove` 校验族（WPT
// dom/nodes/Node-removeChild.html 28F 簇驱动）。断言组：
// ① WebIDL 类型校验：null / 非 Node（无 nodeType）→ TypeError
// ② detached 子（createElement 未挂）→ NotFoundError + ownerDocument 不变
// ③ 非本父的子（挂 documentElement 下，body.removeChild）→ NotFoundError
// ④ s.removeChild(document)（handle-only 父，registry 空）→ NotFoundError
// ⑤ 合成文档（createHTMLDocument）同族：docEl.appendChild 后主文档 remove 抛、
//    body 内 s.removeChild(doc) 抛 + DOMException identity（(doc.defaultView||self)
//    .DOMException —— defaultView undefined 回落 self，R126 _zwDomException globalThis 化）
// ⑥ 文本/注释叶子（无自身 removeChild，childNodes 恒 []）→ NotFoundError 非
//    HierarchyRequestError（spec pre-remove 步骤 1 是包含检查，先于父类型检查）
// ⑦ 合法移除仍生效（R87 注册文本子路径不受校验影响）
#[test]
fn test_remove_child_not_found_validation_r126() {
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
        "<html><body><div id=\"host\"><p id=\"a\">A</p></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             function threw(fn) { try { fn(); return 'none'; } catch (e) { return e.name; } }\
             parts.push('null:' + threw(function () { document.body.removeChild(null); }));\
             parts.push('obj:' + threw(function () { document.body.removeChild({'a':'b'}); }));\
             var s = document.createElement('a');\
             parts.push('detached:' + threw(function () { document.body.removeChild(s); })\
               + ':od:' + (s.ownerDocument === document));\
             var s2 = document.createElement('b');\
             document.documentElement.appendChild(s2);\
             parts.push('other-parent:' + threw(function () { document.body.removeChild(s2); })\
               + ':od:' + (s2.ownerDocument === document));\
             var s3 = document.createElement('test');\
             document.body.appendChild(s3);\
             parts.push('handle-empty:' + threw(function () { s3.removeChild(document); }));\
             var sd = document.implementation.createHTMLDocument();\
             var se = sd.createElement('b');\
             sd.documentElement.appendChild(se);\
             parts.push('syn-el:' + threw(function () { document.body.removeChild(se); })\
               + ':od:' + (se.ownerDocument === sd));\
             var se2 = sd.createElement('test');\
             sd.body.appendChild(se2);\
             var domExc = (sd.defaultView || self).DOMException;\
             var e2n = 'none', e2ctor = 'none';\
             try { se2.removeChild(sd); } catch (e2) { e2n = e2.name; e2ctor = String(e2.constructor === domExc); }\
             parts.push('syn-child:' + e2n + ':ctor:' + e2ctor);\
             var st = sd.createTextNode('t');\
             sd.body.appendChild(st);\
             parts.push('syn-text:' + threw(function () { st.removeChild(sd); }));\
             var sc = sd.createComment('c');\
             sd.body.appendChild(sc);\
             parts.push('syn-comment:' + threw(function () { sc.removeChild(sd); }));\
             var sp = document.createElement('p');\
             document.getElementById('host').appendChild(sp);\
             sp.textContent = 'inner';\
             parts.push('legal-text:' + (threw(function () { sp.removeChild(sp.firstChild); }) === 'none')\
               + ':fc-null:' + (sp.firstChild === null));\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "null:TypeError|obj:TypeError|detached:NotFoundError:od:true|other-parent:NotFoundError:od:true|handle-empty:NotFoundError|syn-el:NotFoundError:od:true|syn-child:NotFoundError:ctor:true|syn-text:NotFoundError|syn-comment:NotFoundError|legal-text:true:fc-null:true",
        "removeChild 校验族：TypeError/NotFoundError/ownerDocument 不变/合成文档 DOMException identity/合法移除不受影响"
    );
}

// js-dom M4 R127：replaceChild 的 spec `dom-node-replace-child` 校验族 + replace 语义
//（WPT dom/nodes/Node-replaceChild.html 15F 簇 + MO-childList 两个悬挂 async_test 驱动）。
// 断言组：
// ① NotFound：非本父的子（detached createElement 三件）→ NotFoundError
// ② Document pre-insert step 6：fragment 多元素/text、element 重复/doctype 前、doctype
//    重复/element 后 → HierarchyRequestError（detached doc 路径）
// ③ replace-with-sibling：[b,c] → replaceChild(c,b) → childNodes=[c]（adopt 先于定位）
// ④ replace-with-self：replaceChild(b,b) 不动树
// ⑤ fragment flatten：doc.replaceChild(df, docEl) → df 子展开进 doc.childNodes
// ⑥ adopt ownerDocument：doc.replaceChild(doc2.doctype, doc.doctype) 后
//    doctype2.ownerDocument === doc、doc2.childNodes 少 1
#[test]
fn test_replace_child_validation_and_semantics_r127() {
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
        "<html><body><div id=\"host\"><p id=\"a\">A</p></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             function threw(fn) { try { fn(); return 'none'; } catch (e) { return e.name; } }\
             var a = document.createElement('div');\
             var b = document.createElement('div');\
             var c = document.createElement('div');\
             parts.push('nf:' + threw(function () { a.replaceChild(b, c); }));\
             var doc = document.implementation.createHTMLDocument('T');\
             var df2 = doc.createDocumentFragment();\
             df2.appendChild(doc.createElement('a'));\
             df2.appendChild(doc.createElement('b'));\
             parts.push('frag-multi:' + threw(function () { doc.replaceChild(df2, doc.documentElement); }));\
             var df3 = doc.createDocumentFragment();\
             df3.appendChild(doc.createTextNode('t'));\
             parts.push('frag-text:' + threw(function () { doc.replaceChild(df3, doc.documentElement); }));\
             var cm = doc.appendChild(doc.createComment('foo'));\
             parts.push('el-dup:' + threw(function () { doc.replaceChild(doc.createElement('a'), cm); }));\
             var docB = document.implementation.createHTMLDocument('B');\
             var cmB = docB.appendChild(docB.createComment('foo'));\
             var dtNew = document.implementation.createDocumentType('html', '', '');\
             parts.push('dt-dup:' + threw(function () { docB.replaceChild(dtNew, cmB); }));\
             a.appendChild(b);\
             a.appendChild(c);\
             a.replaceChild(c, b);\
             parts.push('sib:' + (a.childNodes.length === 1 && a.childNodes[0] === c));\
             var a2 = document.createElement('div');\
             var b2 = document.createElement('div');\
             var cc2 = document.createElement('div');\
             a2.appendChild(b2);\
             a2.appendChild(cc2);\
             a2.replaceChild(b2, b2);\
             parts.push('self:' + (a2.childNodes.length === 2 && a2.childNodes[0] === b2));\
             var docC = document.implementation.createHTMLDocument('C');\
             var dfC = docC.createDocumentFragment();\
             var elC = docC.createElement('x');\
             dfC.appendChild(elC);\
             docC.replaceChild(dfC, docC.documentElement);\
             parts.push('flatten:' + (docC.childNodes.length === 2\
               && docC.childNodes[0].nodeType === 10 && docC.childNodes[1] === elC));\
             var docD = document.implementation.createHTMLDocument('D');\
             var docE = document.implementation.createHTMLDocument('E');\
             var dtE = docE.doctype;\
             docD.replaceChild(docE.doctype, docD.doctype);\
             parts.push('adopt-od:' + (dtE.ownerDocument === docD)\
               + ':d2kids:' + docE.childNodes.length\
               + ':dkids:' + docD.childNodes.length);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "nf:NotFoundError|frag-multi:HierarchyRequestError|frag-text:HierarchyRequestError|el-dup:HierarchyRequestError|dt-dup:HierarchyRequestError|sib:true|self:true|flatten:true|adopt-od:true:d2kids:1:dkids:2",
        "replaceChild 校验族 + replace 语义：NotFound/step6 HRE/sibling/self/fragment flatten/adopt ownerDocument"
    );
}


// js-dom M4 R128：Node-cloneNode 全节点形态族（WPT dom/nodes/Node-cloneNode.html 14F 簇 +
// XMLDocument/document-with-doctype 连带驱动）。断言组：
// ① text/comment/PI/fragment clone 保 nodeType（旧全落元素克隆返 1）
// ② Attr clone 四字段（ns/prefix/localName/value）+ 与源 value 独立
// ③ doctype clone instanceof DocumentType + 三字段
// ④ detached doc clone instanceof Document + XMLDocument constructor + metadata 相等
// ⑤ deep doc clone 带 doctype 子（createDocument(ns,'',dt) 形态）
// ⑥ createElement(unknown) clone instanceof HTMLUnknownElement
// ⑦ createElementNS clone 保 prefix（nodeName 'FOO:DIV'）
// ⑧ 自定义原型不传染 clone + 用户原型 setPrototypeOf 生效
#[test]
fn test_clone_node_all_node_kinds_r128() {
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
        "<html><body><div id=\"host\"><p id=\"a\">A</p></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             var t = document.createTextNode('tx');\
             parts.push('text:' + (t.cloneNode().nodeType === 3 && t.cloneNode().data === 'tx'));\
             var c = document.createComment('cm');\
             parts.push('comment:' + (c.cloneNode().nodeType === 8));\
             var pi = document.createProcessingInstruction('tg', 'dt');\
             var pic = pi.cloneNode();\
             parts.push('pi:' + (pic.nodeType === 7 && pic.target === 'tg' && pic.data === 'dt'));\
             var fr = document.createDocumentFragment();\
             parts.push('frag:' + (fr.cloneNode().nodeType === 11));\
             var at = document.createAttributeNS('http://www.w3.org/1999/xhtml', 'foo:class');\
             at.value = 'v1';\
             var atc = at.cloneNode();\
             at.value = 'v2';\
             parts.push('attr:' + (atc.nodeType === 2 && atc.prefix === 'foo'\
               && atc.localName === 'class' && atc.namespaceURI === 'http://www.w3.org/1999/xhtml'\
               && atc.value === 'v1' && at.value === 'v2'));\
             var dt = document.implementation.createDocumentType('html', 'pub', 'sys');\
             var dtc = dt.cloneNode();\
             parts.push('dt:' + (dtc instanceof DocumentType\
               && dtc.name === 'html' && dtc.publicId === 'pub' && dtc.systemId === 'sys'));\
             var doc = document.implementation.createDocument(null, null, null);\
             var dc = doc.cloneNode();\
             parts.push('doc:' + (dc instanceof Document && dc.constructor === XMLDocument\
               && dc.contentType === 'application/xml' && dc.URL === 'about:blank'\
               && dc.compatMode === 'CSS1Compat'));\
             var doc2 = document.implementation.createDocument('ns', '', dt);\
             var dc2 = doc2.cloneNode(true);\
             parts.push('doc-deep:' + (dc2.childNodes.length === 1\
               && dc2.childNodes[0].nodeType === 10 && dc2.childNodes[0].name === 'html'));\
             var hd = document.implementation.createHTMLDocument('T');\
             parts.push('htmldoc:' + (hd.cloneNode().title === ''));\
             var unk = document.createElement('zz-unknown');\
             parts.push('unknown:' + (unk.cloneNode() instanceof HTMLUnknownElement));\
             var nsEl = document.createElementNS('http://www.w3.org/1999/xhtml', 'foo:div');\
             parts.push('ns:' + (nsEl.cloneNode().nodeName === 'FOO:DIV'));\
             var proto = Object.create(HTMLElement.prototype);\
             var node = document.createElement('hi');\
             Object.setPrototypeOf(node, proto);\
             var nclone = node.cloneNode();\
             parts.push('custproto:' + (proto.isPrototypeOf(node)\
               && !proto.isPrototypeOf(nclone)\
               && HTMLUnknownElement.prototype.isPrototypeOf(nclone)));\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "text:true|comment:true|pi:true|frag:true|attr:true|dt:true|doc:true|doc-deep:true|htmldoc:true|unknown:true|ns:true|custproto:true",
        "cloneNode 全节点形态：text/comment/PI/fragment/Attr/doctype/doc deep/unknown/NS prefix/自定义原型不传染"
    );
}

// js-dom M4 R129：CharacterData 方法族 spec 校验 + WebIDL 转换语义（WPT
// dom/nodes/CharacterData-*.html 44F 簇驱动）。断言组：
// ① appendChild/insertBefore/replaceChild on Text/Comment → HierarchyRequestError
// ② appendData 缺参 TypeError；null/undefined 显式 String 转换
// ③ substringData/insertData/deleteData/replaceData offset 越界 IndexSizeError
// ④ count 负值 unsigned long 回绕 + clamp 余量（非抛）
// ⑤ data = null → ''（LegacyNullToEmptyString）；= undefined → 'undefined'
// ⑥ text.remove() 真移除（父 childNodes 剔除 + parentNode null）
// ⑦ 'remove' in textNode 方法存在性
#[test]
fn test_character_data_method_family_r129() {
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
        "<html><body><div id=\"host\"><p id=\"a\">A</p></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             function threw(fn) { try { fn(); return 'none'; } catch (e) { return e.name; } }\
             var t1 = document.createTextNode('test');\
             var t2 = document.createTextNode('other');\
             parts.push('nochild:' + threw(function () { t1.appendChild(t2); })\
               + ':' + threw(function () { t1.insertBefore(t2, null); })\
               + ':' + threw(function () { t1.replaceChild(t2, t1); }));\
             var t3 = document.createTextNode('test');\
             parts.push('appmissing:' + threw(function () { t3.appendData(); }));\
             t3.appendData(null);\
             var afterNull = t3.data;\
             t3.appendData(undefined);\
             parts.push('appnull:' + afterNull + ':appundef:' + t3.data);\
             var t4 = document.createTextNode('test');\
             parts.push('oob:' + threw(function () { t4.substringData(5, 0); })\
               + ':' + threw(function () { t4.insertData(-1, 'x'); })\
               + ':' + threw(function () { t4.deleteData(6, 1); })\
               + ':' + threw(function () { t4.replaceData(-1, 1, 'y'); }));\
             var t5 = document.createTextNode('test');\
             parts.push('clamp-sub:' + t5.substringData(0, -1)\
               + ':' + (function () { t5.deleteData(2, -1); return t5.data; })()\
               + ':' + (function () { t5.replaceData(1, -1, 'yo'); return t5.data; })());\
             var t6 = document.createTextNode('test');\
             t6.data = null;\
             var dn = t6.data;\
             t6.data = undefined;\
             parts.push('data-null:' + JSON.stringify(dn) + ':undef-here:' + t6.data);\
             var parent = document.createElement('div');\
             var t7 = document.createTextNode('mid');\
             parent.appendChild(document.createComment('before'));\
             parent.appendChild(t7);\
             parent.appendChild(document.createComment('after'));\
             t7.remove();\
             parts.push('remove:' + (t7.parentNode === null)\
               + ':kids:' + parent.childNodes.length\
               + ':inop:' + ('remove' in t7 && 'appendData' in t7));\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "nochild:HierarchyRequestError:HierarchyRequestError:HierarchyRequestError|appmissing:TypeError|appnull:testnull:appundef:testnullundefined|oob:IndexSizeError:IndexSizeError:IndexSizeError:IndexSizeError|clamp-sub:test:te:tyo|data-null:\"\":undef-here:undefined|remove:true:kids:2:inop:true",
        "CharacterData 方法族：叶子节点 HRE/缺参 TypeError/offset IndexSizeError/count 回绕 clamp/LegacyNullToEmptyString/remove 真移除/方法存在性"
    );
}
