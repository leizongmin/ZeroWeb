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

// R130（js-dom M4）：DOMImplementation.createHTMLDocument/createDocument detached doc
// 语义族——title 子树（spec 步骤 4「create a title element」）/ documentElement 惰性 getter
//（首个元素子，XML omit root 时 null）/ XMLDocument.prototype 接线 / createDocument
// WebIDL 必参 + qualifiedName root 创建 / createDocumentType 宽松校验 / A.href IDL
// percent-encode / doc.location null。
#[test]
fn test_dom_implementation_doc_family_r130() {
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
             var hd = document.implementation.createHTMLDocument('foo');\
             parts.push('htmltitle:' + hd.head.childNodes.length\
               + ':' + hd.head.firstChild.localName\
               + ':' + hd.head.firstChild.firstChild.data\
               + ':inst:' + (hd.documentElement instanceof HTMLHtmlElement\
                 && hd.head instanceof HTMLHeadElement && hd.body instanceof HTMLBodyElement\
                 && hd.head.firstChild instanceof HTMLTitleElement)\
               + ':loc:' + hd.location\
               + ':kids:' + hd.childNodes.length);\
             var nd = document.implementation.createHTMLDocument();\
             parts.push('notitle:' + nd.head.childNodes.length);\
             var xd = document.implementation.createDocument('', '');\
             parts.push('xmldoc:' + (Object.getPrototypeOf(xd) === XMLDocument.prototype)\
               + ':docel:' + xd.documentElement\
               + ':ct:' + xd.contentType);\
             var x2 = document.implementation.createDocument('http://www.w3.org/1999/xhtml', 'html');\
             parts.push('xmlroot:' + x2.documentElement.localName\
               + ':' + x2.documentElement.childNodes.length\
               + ':xhtml-ct:' + x2.contentType\
               + ':create-div:' + x2.createElement('DIV').localName);\
             parts.push('missing:' + threw(function () { document.implementation.createDocument(); })\
               + ':' + threw(function () { document.implementation.createDocument(''); })\
               + ':bad-dt:' + threw(function () { document.implementation.createDocument('', '', false); }));\
             var dt = document.implementation.createDocumentType('{', '', '');\
             parts.push('dt-ok:' + dt.name\
               + ':dt-throw:' + threw(function () { document.implementation.createDocumentType('edi:>', '', ''); })\
               + ':' + threw(function () { document.implementation.createDocumentType('edi:a ', '', ''); }));\
             var ad = hd.createElement('a');\
             ad.setAttribute('href', 'http://example.org/?\\u00E4');\
             parts.push('href:' + ad.href + ':raw:' + ad.getAttribute('href'));\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "htmltitle:1:title:foo:inst:true:loc:null:kids:2|notitle:0|xmldoc:true:docel:null:ct:application/xml|xmlroot:html:0:xhtml-ct:application/xhtml+xml:create-div:DIV|missing:TypeError:TypeError:bad-dt:TypeError|dt-ok:{:dt-throw:InvalidCharacterError:InvalidCharacterError|href:http://example.org/?%C3%A4:raw:http://example.org/?ä",
        "R130 createHTMLDocument title 子树/原型接线/location null + createDocument XMLDocument/docElement 惰性/必参校验/root 创建 + createDocumentType 校验 + A.href percent-encode"
    );
}

// R131（js-dom M4）：isEqualNode spec 逐类型字段比较（dom-node-isequalnode）——
// 元素 ns/prefix/localName/属性集（prefix 参与、属性 prefix 不参与、属性序无关）+
// PI target/data + doctype 三字段 + 子节点递归 + 合成 docEl/head/body 的 ns 标注。
#[test]
fn test_is_equal_node_spec_fields_r131() {
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
             var e1 = document.createElementNS('namespace', 'prefix:localName');\
             var e2 = document.createElementNS('namespace', 'prefix:localName');\
             var e3 = document.createElementNS('namespace2', 'prefix:localName');\
             var e4 = document.createElementNS('namespace', 'prefix2:localName');\
             parts.push('el-ns:' + e1.isEqualNode(e2) + ':' + e1.isEqualNode(e3) + ':' + e1.isEqualNode(e4));\
             var a1 = document.createElement('element');\
             a1.setAttributeNS('namespace', 'prefix:localName', 'value');\
             var a2 = document.createElement('element');\
             a2.setAttributeNS('namespace', 'prefix2:localName', 'value');\
             var a3 = document.createElement('element');\
             a3.setAttributeNS('namespace2', 'prefix:localName', 'value');\
             parts.push('attr:' + a1.isEqualNode(a2) + ':' + a1.isEqualNode(a3));\
             var p1 = document.createProcessingInstruction('target', 'data');\
             var p2 = document.createProcessingInstruction('target2', 'data');\
             var p3 = document.createProcessingInstruction('target', 'data2');\
             parts.push('pi:' + p1.isEqualNode(p1) + ':' + p1.isEqualNode(p2) + ':' + p1.isEqualNode(p3));\
             var d1 = document.implementation.createDocumentType('n', 'p', 's');\
             var d2 = document.implementation.createDocumentType('n', 'p', 's');\
             var d3 = document.implementation.createDocumentType('n2', 'p', 's');\
             var d4 = document.implementation.createDocumentType('n', 'p2', 's');\
             parts.push('dt:' + d1.isEqualNode(d2) + ':' + d1.isEqualNode(d3) + ':' + d1.isEqualNode(d4));\
             var hd = document.implementation.createHTMLDocument();\
             var d5 = document.implementation.createDocument('http://www.w3.org/1999/xhtml', 'html', document.implementation.createDocumentType('html', '', ''));\
             d5.documentElement.appendChild(d5.createElement('head'));\
             d5.documentElement.appendChild(d5.createElement('body'));\
             parts.push('docs:' + hd.isEqualNode(d5));\
             var frag1 = document.createDocumentFragment();\
             var frag2 = document.createDocumentFragment();\
             frag1.appendChild(document.createComment('data'));\
             parts.push('frag:' + frag1.isEqualNode(frag2));\
             frag2.appendChild(document.createComment('data'));\
             parts.push('frag-eq:' + frag1.isEqualNode(frag2));\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "el-ns:true:false:false|attr:true:false|pi:true:false:false|dt:true:false:false|docs:true|frag:false|frag-eq:true",
        "R131 isEqualNode：ns/prefix/localName/属性集（属性 prefix 不参与）/PI target/doctype 三字段/文档结构/子节点递归"
    );
}

// R132（js-dom M4）：importNode spec 语义（dom-document-importnode = clone + adopt）
// ——浅/深变体 childNodes 语义 + ownerDocument 递归归属 + Attr 全字段复制 + detached
// body 的 setAttributeNS/getAttributeNodeNS。
#[test]
fn test_import_node_spec_semantics_r132() {
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
             var doc = document.implementation.createHTMLDocument('Title');\
             var div = doc.body.appendChild(doc.createElement('div'));\
             div.appendChild(doc.createElement('span'));\
             var s1 = document.importNode(div);\
             parts.push('shallow:' + (s1.firstChild === null)\
               + ':' + (s1.ownerDocument === document)\
               + ':src-intact:' + (div.ownerDocument === doc) + ':' + (div.firstChild ? 'kept' : 'lost'));\
             var s2 = document.importNode(div, true);\
             parts.push('deep:' + (s2.firstChild ? s2.firstChild.ownerDocument === document : false)\
               + ':' + (s2.firstChild ? String(s2.firstChild.nodeName) : 'null'));\
             doc.body.setAttributeNS('http://example.com/', 'p:name', 'value');\
             var attr = doc.body.getAttributeNodeNS('http://example.com/', 'name');\
             parts.push('attr-found:' + (attr !== null && attr !== undefined));\
             if (attr) {\
               var imp = document.importNode(attr, true);\
               parts.push('attr:' + String(imp.prefix) + ':' + String(imp.namespaceURI) + ':' + String(imp.localName));\
             }\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "shallow:true:true:src-intact:true:kept|deep:true:SPAN|attr-found:true|attr:p:http://example.com/:name",
        "R132 importNode：浅剥子/深递归 ownerDocument/源树完整/Attr prefix+ns+localName 复制/detached body setAttributeNS+getAttributeNodeNS"
    );
}

// R133（js-dom M4）：insertAdjacentElement 入口校验（spec dom-element-insertadjacentelement
// 步骤 1-3）——非法 position 同步抛 SyntaxError DOMException（ASCII case-insensitive）；
// 非节点参数 TypeError；documentElement 的 beforebegin/afterend 抛 HierarchyRequestError。
#[test]
fn test_insert_adjacent_element_validation_r133() {
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
             var host = document.getElementById('host');\
             var p1 = document.createElement('p');\
             parts.push('syntax:' + threw(function () { host.insertAdjacentElement('test', p1); })\
               + ':case-insensitive:' + threw(function () { host.insertAdjacentElement('BeforeEnd', 'x'); }))\
             ;\
             parts.push('type:' + threw(function () { host.insertAdjacentElement('beforeend', 'notanode'); }))\
             ;\
             var docEl = document.documentElement;\
             parts.push('hre:' + threw(function () { docEl.insertAdjacentElement('beforebegin', p1); })\
               + ':' + threw(function () { docEl.insertAdjacentElement('afterend', p1); }))\
             ;\
             var ok = false;\
             try { var r = host.insertAdjacentElement('afterbegin', p1); ok = (r === p1); } catch (e) { ok = 'err:' + e.name; }\
             parts.push('legal:' + ok);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "syntax:SyntaxError:case-insensitive:TypeError|type:TypeError|hre:HierarchyRequestError:HierarchyRequestError|legal:true",
        "R133 insertAdjacentElement：非法 position SyntaxError（大小写不敏感）/非节点参数 TypeError/documentElement beforebegin+afterend HRE/合法路径返插入元素"
    );
}

// R134（js-dom M4）：matches 的 type selector NS 匹配（selectors-4 §6.1——handle-only
// 元素 JS 侧匹配）+ Element.prototype [Unscopable] 表 + on* setAttribute 的 handler
// 缓存失效重编译。
#[test]
fn test_matches_ns_and_unscopables_r134() {
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
             var e1 = document.createElementNS('', 'element');\
             parts.push('empty-ns:' + e1.matches('element'));\
             var e2 = document.createElementNS('urn:ns', 'h');\
             parts.push('ns:' + e2.matches('h') + ':star-bar:' + e2.matches('*|h')\
               + ':ns-bar:' + e2.matches('urn:ns|h') + ':wrong-ns-bar:' + e2.matches('urn:x|h')\
               + ':star:' + e2.matches('*') + ':wrong-type:' + e2.matches('x'));\
             var d = document.createElement('div');\
             parts.push('html-bare:' + d.matches('div') + ':upper:' + d.matches('DIV'));\
             parts.push('complex-false:' + d.matches('div.foo'));\
             parts.push('unscopables:' + (Element.prototype[Symbol.unscopables].before === true\
               && Element.prototype[Symbol.unscopables].remove === true\
               && Element.prototype[Symbol.unscopables].append === true));\
             parts.push('proxy-tab:' + (d[Symbol.unscopables] === Element.prototype[Symbol.unscopables]));\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "empty-ns:true|ns:true:star-bar:true:ns-bar:true:wrong-ns-bar:false:star:true:wrong-type:false|html-bare:true:upper:true|complex-false:false|unscopables:true|proxy-tab:true",
        "R134 matches NS 三形态（空 ns 裸 type/urn:ns 裸+*|+ns|+错 ns/*）+ HTML 大小写 + 复合选择器保守 false + unscopables 表 + proxy get trap 透出"
    );
}

// R135（js-dom M4）：name-validation spec regex 族——createElement/createElementNS/
// createDocument/createAttribute(NS)/setAttribute(NS)/createProcessingInstruction/
// createDocumentType 的 valid/invalid 名单语义（WPT dom/nodes/name-validation.html）。
// 关键差异点：\x0B 非 ASCII 空白（valid）；NUL invalid；attribute 名无首字符限制但禁 '='；
// NS prefix 段允许 '='；localName 段走 spec valid-name regex（':soh\x01' 的 '\x01' 非 NameChar → 抛）。
// https://dom.spec.whatwg.org/#valid-name
#[test]
fn test_name_validation_spec_regex_r135() {
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
            // createElement：字母首字符后 \x0B（垂直制表，非 ASCII 空白五字符）valid；
            // NUL / ASCII 空白 / '/' / '>' invalid；'_x' / ':x' / 非 ASCII 首 valid。
            "var parts = [];\
             var vt = false, vf1 = false, vf2 = false, vf3 = false, vf4 = false;\
             try { document.createElement('A\\x0B'); vt = true; } catch (e) { vt = e.name; }\
             try { document.createElement('null\\0'); } catch (e) { vf1 = e.name; }\
             try { document.createElement('fo o'); } catch (e) { vf2 = e.name; }\
             try { document.createElement('fo/o'); } catch (e) { vf3 = e.name; }\
             try { document.createElement('fo>o'); } catch (e) { vf4 = e.name; }\
             parts.push('ce:' + vt + ':' + vf1 + ':' + vf2 + ':' + vf3 + ':' + vf4);\
             var vu1 = false, vu2 = false;\
             try { document.createElement('_x'); vu1 = true; } catch (e) {}\
             try { document.createElement(':x'); vu2 = true; } catch (e) {}\
             parts.push('ce-valid:' + vu1 + ':' + vu2);\
             // createElementNS / createDocument：localName 'null\\0' 抛 InvalidCharacterError；
             // ':soh\\x01' local 含非 NameChar 抛（spec regex 逐段校验）；
             // 空前缀 ':div' 抛；合法 'smallEmoji🆖:div' 不抛。
             var ns1 = false, ns2 = false, ns3 = false, ns4 = true, ns5 = true;\
             try { document.createElementNS('urn:x', 'p:null\\0'); } catch (e) { ns1 = e.name; }\
             try { document.implementation.createDocument('urn:x', 'p:null\\0'); } catch (e) { ns2 = e.name; }\
             try { document.createElementNS('urn:x', ':soh\\u0001'); } catch (e) { ns3 = e.name; }\
             try { document.implementation.createDocument('urn:x', 'smallEmoji🆖:div'); } catch (e) { ns4 = 'THROW:' + e.name; }\
             try { document.createElementNS('urn:x', 'smallEmoji🆖:div'); } catch (e) { ns5 = 'THROW:' + e.name; }\
             parts.push('ns:' + ns1 + ':' + ns2 + ':' + ns3 + ':' + ns4 + ':' + ns5);\
             // createAttribute：'\\x01foo' 无首字符限制 valid；'a=b' 的 '=' invalid；
             // 'null\\0' / 空白 invalid；createAttributeNS prefix 段 '=' 合法、local 段 '=' 抛。
             var a1 = false, a2 = false, a3 = false;\
             try { document.createAttribute('\\x01foo'); a1 = true; } catch (e) { a1 = e.name; }\
             try { document.createAttribute('a=b'); } catch (e) { a2 = e.name; }\
             try { document.createAttribute('null\\0'); } catch (e) { a3 = e.name; }\
             parts.push('attr:' + a1 + ':' + a2 + ':' + a3);\
             var an1 = true, an2 = false;\
             try { document.createAttributeNS('urn:x', 'p=a:attr'); } catch (e) { an1 = 'THROW:' + e.name; }\
             try { document.createAttributeNS('urn:x', 'p:a=b'); } catch (e) { an2 = e.name; }\
             parts.push('attrns:' + an1 + ':' + an2);\
             // setAttribute / setAttributeNS 同语义（attribute 名单）。
             var d = document.createElement('div');\
             var s1 = false, s2 = false, s3 = false;\
             try { d.setAttribute('\\x01foo', 'v'); s1 = true; } catch (e) { s1 = e.name; }\
             try { d.setAttribute('a=b', 'v'); } catch (e) { s2 = e.name; }\
             try { d.setAttributeNS('urn:x', 'p:a=b', 'v'); } catch (e) { s3 = e.name; }\
             parts.push('set:' + s1 + ':' + s2 + ':' + s3);\
             // createProcessingInstruction：target spec regex（'\\x01t' 非 NameStart 抛；'A\\x0B' valid）。
             var p1 = false, p2 = true;\
             try { document.createProcessingInstruction('\\x01t', 'd'); } catch (e) { p1 = e.name; }\
             try { document.createProcessingInstruction('A\\x0B', 'd'); } catch (e) { p2 = 'THROW:' + e.name; }\
             parts.push('pi:' + p1 + ':' + p2);\
             // createDocumentType：NUL invalid（R130 旧 /\\s/ 漏）；'\\x0B' valid。
             var dt1 = false, dt2 = true;\
             try { document.implementation.createDocumentType('null\\0', '', ''); } catch (e) { dt1 = e.name; }\
             try { document.implementation.createDocumentType('A\\x0B', '', ''); } catch (e) { dt2 = 'THROW:' + e.name; }\
             parts.push('dt:' + dt1 + ':' + dt2);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "ce:true:InvalidCharacterError:InvalidCharacterError:InvalidCharacterError:InvalidCharacterError|ce-valid:true:true|\
ns:InvalidCharacterError:InvalidCharacterError:InvalidCharacterError:true:true|\
attr:true:InvalidCharacterError:InvalidCharacterError|attrns:true:InvalidCharacterError|\
set:true:InvalidCharacterError:InvalidCharacterError|\
pi:InvalidCharacterError:true|\
dt:InvalidCharacterError:true",
        "R135 name-validation spec regex 族：createElement \\x0B valid/NUL·空白·slash·gt invalid；NS localName NUL/\\x01 抛（createDocument 同步）；attribute 无首字符限制禁 '='；NS prefix 禁 ':' local 禁 '='；PI target regex；doctype NUL invalid"
    );
}

// R136（js-dom M4）：`Node.prototype.getRootNode` 泛型（spec dom-node-getrootnode，
// https://dom.spec.whatwg.org/#dom-node-getrootnode）——沿 parentNode 链上行到根 +
// composed 选项 shadow-including root。旧实现仅 proxy 元素 get trap 的 sel 版分支
// （handle-only 元素 / document / fragment / text / PI 均不可达 → "not a function"，
// WPT dom/nodes/rootNode.html 5F）。配套：shadow root 的 parentNode 恒 null（composed
// 经 host 上行）+ shadow innerHTML 解析子的 parentNode 指宿主容器 proxy（沿链上行
// 到 shadow root 而非断裂在内部假 body 快照）。
#[test]
fn test_get_root_node_generic_r136() {
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
        "<html><body><div id=\"host\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             // detached 无父 → 自身（handle-only 元素 / text / PI / fragment 四形态）。
             var e = document.createElement('div');\
             parts.push('el:' + (e.getRootNode() === e));\
             var t = document.createTextNode('x');\
             parts.push('text:' + (t.getRootNode() === t));\
             var pi = document.createProcessingInstruction('tgt', 'd');\
             parts.push('pi:' + (pi.getRootNode() === pi));\
             var f = document.createDocumentFragment();\
             parts.push('frag-self:' + (f.getRootNode() === f));\
             // fragment 子 → fragment 根（沿 parentNode 上行）。
             var c = document.createElement('span');\
             f.appendChild(c);\
             parts.push('frag-child:' + (c.getRootNode() === f));\
             // document 自身是根。
             parts.push('doc:' + (document.getRootNode() === document));\
             // 挂载元素 → document（sel 形态沿链上行）。
             var host = document.getElementById('host');\
             parts.push('mounted:' + (host.getRootNode() === document));\
             // composed: shadow 树内子 → 无 composed 返 shadowRoot，composed 顺 host 到 document。
             var sr = host.attachShadow({ mode: 'open' });\
             sr.innerHTML = '<div class=\"sc\">content</div>';\
             var sc = sr.querySelector('.sc');\
             parts.push('has-sc:' + !!sc);\
             parts.push('shadow-root:' + (sc.getRootNode() === sr));\
             parts.push('shadow-composed:' + (sc.getRootNode({ composed: true }) === document));\
             parts.push('sr-parent-null:' + (sr.parentNode === null));\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "el:true|text:true|pi:true|frag-self:true|frag-child:true|doc:true|mounted:true|\
has-sc:true|shadow-root:true|shadow-composed:true|sr-parent-null:true",
        "R136 getRootNode 泛型：四形态 detached 自根 + fragment 子沿链 + document 自根 + 挂载元素到 document + shadow 子无 composed 返 shadowRoot/composed 穿 host 到 document + shadow root parentNode 恒 null"
    );
}

// R138（js-dom M4）：native 叠加路径的事件方法族可达性 + srcElement/returnValue——
// native MouseEvent 实例（V8 FunctionTemplate 产物）没有 shim `_makeEvent` 的 own
// stopPropagation/preventDefault（own 挂工厂普通对象上），listener 内
// `event.stopPropagation()` 报 "not a function"；own data srcElement=null 遮蔽原型
// accessor；无 returnValue。修：事件方法族 + returnValue/srcElement accessor 幂等上
// Event.prototype（shim 产物 own 遮蔽零变化，native 实例经 R109 重接链可达）+
// _dispatchWithBubble 设 target 时同步 own-set srcElement。
#[test]
fn test_event_proto_methods_native_overlay_r138() {
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
        "<html><body><div id=\"a\">A</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             // Event.prototype 方法族（R138 幂等补挂——shim 产物 own 遮蔽，原型版对
             // native 实例补位；此处验证 shim 路径语义不变）。
             parts.push('proto-stop:' + (typeof Event.prototype.stopPropagation === 'function'));\
             parts.push('proto-prevent:' + (typeof Event.prototype.preventDefault === 'function'));\
             parts.push('proto-imm:' + (typeof Event.prototype.stopImmediatePropagation === 'function'));\
             parts.push('proto-rv:' + (typeof Event.prototype.returnValue));\
             parts.push('proto-src:' + (typeof Event.prototype.srcElement));\
             // dispatch：capture stopPropagation 止同元素 bubble listener（WPT
             // Event-stopPropagation-cancel-bubbling 语义）+ srcElement own-set。
             var el = document.createElement('div');\
             var bubbleRan = false;\
             el.addEventListener('click', function () { event.stopPropagation(); }, { capture: true });\
             el.addEventListener('click', function () { bubbleRan = true; });\
             var ce = new MouseEvent('click', { bubbles: true, cancelable: true });\
             el.dispatchEvent(ce);\
             parts.push('bubble-stopped:' + !bubbleRan);\
             parts.push('src-is-target:' + (ce.srcElement === el));\
             parts.push('rv-true:' + (ce.returnValue === true));\
             parts.push('phase-0:' + (ce.eventPhase === 0));\
             parts.push('ct-null:' + (ce.currentTarget === null));\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "proto-stop:true|proto-prevent:true|proto-imm:true|proto-rv:boolean|proto-src:undefined|\
bubble-stopped:true|src-is-target:true|rv-true:true|phase-0:true|ct-null:true"
            ,
        "R138 事件方法族原型补挂 + dispatch srcElement/returnValue"
    );
}


// R139（js-dom M4）：EventTarget 对象 listener（WebIDL EventListener callback）+
// Text dispatchEvent 非 bubbling 不冒泡——① EventTarget.addEventListener 旧版对非函数
// listener 直接 return（对象 listener 根本不注册，WPT EventListener-handleEvent-cross-realm
// 5F 前置根因）；② dispatch 循环补 handleEvent 分派 + 非 callable TypeError 上报 +
// revoked Proxy Get/call 抛上报（spec inner invoke 步骤 1-2 + report the exception）；
// ③ Text.dispatchEvent 无条件转父使父成新 target——非 bubbling 的 Text click 也触发父
// pre-click activation 翻 checked（WPT Event-dispatch-click "look at parents only when
// event bubbles"：bubbles=false 断言父 checked 不变）。
// R143（js-dom M4）：spec「add an event listener」步骤 4——重复 listener（同 type/callback/
// capture/槽位）静默丢弃 + window GlobalEventHandlers on* 全族 IDL 属性（onclick 等——
// setter 移旧注册新，getter 返存储 fn）。WPT dom/events/handler-count（window 变体）双 subtest。
#[test]
fn test_listener_dedup_and_window_on_family_r143() {
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
            "var parts = [];\
             var tally = 0; var listener = function() { tally++; };\
             addEventListener('click', listener, true);\
             addEventListener('click', listener, true);\
             dispatchEvent(new Event('click'));\
             parts.push('winDedup:' + tally);\
             addEventListener('ping', listener, false);\
             dispatchEvent(new Event('ping'));\
             parts.push('winCapSplit:' + tally);\
             var dt = 0; var dl = function() { dt++; };\
             document.addEventListener('info', dl);\
             document.addEventListener('info', dl);\
             document.dispatchEvent(new Event('info'));\
             parts.push('docDedup:' + dt);\
             var et = 0; var el = function() { et++; };\
             var d = document.getElementById('d');\
             d.addEventListener('go', el);\
             d.addEventListener('go', el);\
             d.dispatchEvent(new Event('go'));\
             parts.push('elDedup:' + et);\
             var c1 = 0, c2 = 0;\
             onclick = function() { c1++; };\
             onclick = function() { c2++; };\
             dispatchEvent(new Event('click'));\
             parts.push('onclickReplace:' + c1 + ':' + c2 + ':' + (typeof onclick === 'function'));\
             onclick = null;\
             dispatchEvent(new Event('click'));\
             parts.push('onclickRemoved:' + c2);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "winDedup:1|winCapSplit:2|docDedup:1|elDedup:1|onclickReplace:0:1:true|onclickRemoved:1",
        "R143 重复 listener 丢弃（window/document/element 三面）+ window onclick 替换/移除语义"
    );
}

#[test]
fn test_eventtarget_object_listener_r139() {
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
        "<html><body><div id=\"host\"><input id=\"cb\" type=\"checkbox\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             // ① 对象 listener 注册 + handleEvent 分派（this = 对象本身）。
             var et = new EventTarget;\
             var gotEvt = null, gotThis = null;\
             var objL = { handleEvent: function (e) { gotEvt = e; gotThis = this; } };\
             et.addEventListener('foo', objL);\
             et.dispatchEvent(new Event('foo'));\
             parts.push('obj-listener:' + (gotEvt && gotEvt.type === 'foo' && gotThis === objL));\
             // ② handleEvent 缺失 → TypeError 经 window 'error' 事件上报。
             var errObj = null;\
             window.addEventListener('error', function (e) { errObj = e; });\
             var et2 = new EventTarget;\
             et2.addEventListener('foo', {});\
             et2.dispatchEvent(new Event('foo'));\
             parts.push('report:' + (errObj && errObj.error && errObj.error.name === 'TypeError'));\
             // ③ revoked callable Proxy listener：call 抛 → 上报。
             var rp = Proxy.revocable(function () {}, {}); rp.revoke();\
             var errObj2 = null;\
             window.addEventListener('error', function (e) { if (!errObj2) errObj2 = e; });\
             var et3 = new EventTarget;\
             et3.addEventListener('bar', rp.proxy);\
             et3.dispatchEvent(new Event('bar'));\
             parts.push('revoked:' + (errObj2 && errObj2.error && errObj2.error.name === 'TypeError'));\
             // ④ Text dispatchEvent 非 bubbling：不触发父 pre-click activation（checked 不变）。
             var input = document.createElement('input'); input.type = 'checkbox'; document.getElementById('host').appendChild(input);\
             var textChild = new Text('x');\
             input.appendChild(textChild);\
             // （bubbling=true 时父 activation 触发——对照组）
             parts.push('before:' + input.checked);\
             textChild.dispatchEvent(new MouseEvent('click'));\
             parts.push('nonbubbling:' + input.checked);\
             textChild.dispatchEvent(new MouseEvent('click', { bubbles: true }));\
             parts.push('bubbling:' + input.checked);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "obj-listener:true|report:true|revoked:true|before:false|nonbubbling:false|bubbling:true",
        "R139 EventTarget 对象 listener + handleEvent TypeError 上报 + revoked Proxy 上报 + Text 非 bubbling 不触发父 activation"
    );
}

// R140（js-dom M4）：live childNodes（spec `dom-node-childnodes` 返 live NodeList）——
// ① 同节点重复读同一对象（caching）② append/insert/remove 后旧引用反映（live）③
// item()/迭代器（真数组承载——Array 原生 keys/values/entries/forEach/Symbol.iterator
// 的 identity 断言）④ instanceof NodeList（Symbol.hasInstance 认 __zwLiveNL 数组，不换
// 原型保迭代器）⑤ mutation 入口（_recordHandleChild/_unrecordHandleChild/appendChild/
// insertBefore/removeChild/replaceChild/remove）记账后经 _zwLiveNLSync 同步。
// WPT dom/nodes/Node-childNodes{,-cache,-cache-2}.html 8 subtest 0F 双路径。
#[test]
fn test_live_childnodes_r140() {
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
        "<html><body><ul id=\"list\"><li>1</li><li>2</li></ul></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    let out = sandbox
        .execute(
            "var parts = [];\
             // ① handle 元素 caching。
             var el = document.createElement('div');\
             parts.push('cache:' + (el.childNodes === el.childNodes));\
             // ② live：append 后旧引用反映 + item。
             var ch = el.childNodes;\
             var c1 = document.createElement('p');\
             el.appendChild(c1);\
             parts.push('liveAppend:' + (ch.length === 1 && ch[0] === c1 && ch.item(0) === c1));\
             // ③ fragment 容器同款。
             var f = document.createDocumentFragment();\
             var fc = document.createElement('span');\
             f.appendChild(fc);\
             parts.push('frag:' + (f.childNodes.length === 1 && f.childNodes.item(0) === fc));\
             // ④ sel 元素（挂载）：caching + append/remove live + instanceof NodeList。
             var ul = document.getElementById('list');\
             var lc = ul.childNodes;\
             parts.push('selCache:' + (ul.childNodes === lc) + ':pre:' + lc.length);\
             var li3 = document.createElement('li');\
             ul.appendChild(li3);\
             parts.push('selAppend:' + lc.length);\
             ul.removeChild(li3);\
             parts.push('selRemove:' + lc.length);\
             parts.push('instanceNL:' + (lc instanceof NodeList));\
             // ⑤ 迭代器 identity（真数组承载）。
             parts.push('iter:' + (lc[Symbol.iterator] === Array.prototype[Symbol.iterator]));\
             // ⑥ detached doc childNodes 的 item。
             var d = new Document();\
             var dp = document.createElement('p');\
             d.appendChild(dp);\
             parts.push('docItem:' + (d.childNodes.item(0) === dp));\
             // ⑦ live 数组经 splice 同步不破坏 data 访问（R117/R119 回归：prepend('text')
             // 后 prepend(null) 的 [null 文本, text 文本] 双子——splice 写索引须保 data 面）。
             var pr = document.createElement('div');\
             pr.prepend('text');\
             pr.prepend(null);\
             parts.push('prependText:' + pr.childNodes[0].data + ',' + pr.childNodes[1].data + ':' + pr.childNodes.length);\
             // ⑧ replaceWith handle 路径在 live 承载下的末子读取（R117 回归）。
             var rw = document.createElement('div');\
             var cx = document.createElement('x'); var cc = document.createElement('y');\
             rw.appendChild(cx); rw.appendChild(cc);\
             var rwc = rw.childNodes;\
             cx.replaceWith('r');\
             parts.push('rwLive:' + rwc.length + ':' + (rwc[0] && rwc[0].nodeName === '#text' && rwc[0].data === 'r') + ':' + (rwc[1] === cc));\
             var rwT1 = rw.childNodes[0];\
             var rwFreshOk = (rwT1 && rwT1.nodeName === '#text' && rwT1.data === 'r' && rw.childNodes[1] === cc);\
             parts.push('rwFresh:' + (rw.childNodes === rwc) + ':' + rw.childNodes.length + ':' + rwFreshOk);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "cache:true|liveAppend:true|frag:true|selCache:true:pre:2|selAppend:3|selRemove:2|instanceNL:true|iter:true|docItem:true|prependText:null,text:2|rwLive:2:true:true|rwFresh:true:2:true",
        "R140 live childNodes：caching/live/item/instanceof/迭代器/doc item 全链 + prepend/replaceWith live 回归"
    );
}
