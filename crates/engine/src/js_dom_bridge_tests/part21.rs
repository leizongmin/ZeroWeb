// js-dom M4 R146 单元测试——events 语义收口三件：
// ① dispatchEvent 的 target/srcElement 设置（_zwMEl 节点族——detached doc createElement 产物）
// ② dispatch 结束无条件清 stop/immediate flag（同 event 二次 dispatch 恢复触发）
// ③ addEventListener 第三参的 WebIDL boolean 转换（primitive 真值 2.3/"AAAA" → capture）

/// R146：`_zwMEl` 节点的 dispatchEvent 须设 target/srcElement（spec `dom-event-dispatch`
/// 步骤——listener 读 ev.target 断言自身；WPT Event-dispatch-other-document）。
#[test]
fn test_mel_dispatch_event_target_r146() {
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
             var doc = document.implementation.createHTMLDocument('Demo');\
             var el = doc.createElement('div');\
             var targetSeen = 'unset', srcSeen = 'unset';\
             el.addEventListener('foo', function(ev) {\
               targetSeen = ev.target === el ? 'same' : 'other';\
               srcSeen = ev.srcElement === el ? 'same' : 'other';\
             });\
             doc.body.appendChild(el);\
             el.dispatchEvent(new Event('foo'));\
             parts.push('target:' + targetSeen);\
             parts.push('src:' + srcSeen);\
             parts.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "target:same|src:same",
        "R146 _zwMEl dispatchEvent 须设 target/srcElement（WPT Event-dispatch-other-document）"
    );
}

/// R146：dispatch 结束无条件清 `_propagationStopped`/`_immediateStopped`——spec
/// `concept-event-dispatch` 末步 unset stop flags 无「仅 dispatch 内设」限定；同 event
/// 二次 dispatch 恢复触发（WPT Event-propagation "After stopImmediatePropagation()"——
/// 一次 dispatch 零触发后第二次 dispatch 应正常触发）。
#[test]
fn test_dispatch_clears_stop_flags_for_redispatch_r146() {
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
             var head = document.head;\
             var tally1 = 0, tally2 = 0;\
             var cb1 = function() { tally1++; };\
             head.addEventListener('foo', cb1);\
             var ev1 = document.createEvent('Event');\
             ev1.initEvent('foo', true, false);\
             ev1.stopPropagation();\
             head.dispatchEvent(ev1);\
             parts.push('firstStopped:' + tally1);\
             head.dispatchEvent(ev1);\
             parts.push('secondFires:' + tally1);\
             var ev2 = document.createEvent('Event');\
             ev2.initEvent('foo', true, false);\
             ev2.stopImmediatePropagation();\
             var tallyPre = 0;\
             var cb2 = function() { tallyPre++; };\
             head.addEventListener('foo', cb2);\
             head.dispatchEvent(ev2);\
             parts.push('immFirstStopped:' + tallyPre);\
             head.dispatchEvent(ev2);\
             parts.push('immSecondFires:' + tallyPre);\
             parts.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "firstStopped:0|secondFires:1|immFirstStopped:0|immSecondFires:1",
        "R146 dispatch 末步无条件清 stop/immediate flag（二次 dispatch 恢复触发）"
    );
}

/// R146：addEventListener 第三参的 WebIDL boolean 转换——primitive 真值
/// （2.3/-1000.3/"AAAA"）→ true（capture）；对象形态读 .capture 字段再转换。
/// WPT EventListenerOptions-capture "Capture boolean should be honored correctly"。
#[test]
fn test_opt_capture_webidl_boolean_r146() {
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
             function probe(captureValue) {\
               var phase = -1;\
               var h = function(e) { phase = e.eventPhase; };\
               document.addEventListener('test', h, captureValue);\
               document.body.dispatchEvent(new Event('test', {bubbles: true}));\
               document.removeEventListener('test', h, captureValue);\
               return phase;\
             }\
             parts.push('num:' + (probe(2.3) === Event.CAPTURING_PHASE));\
             parts.push('negNum:' + (probe(-1000.3) === Event.CAPTURING_PHASE));\
             parts.push('nan:' + (probe(NaN) === Event.BUBBLING_PHASE));\
             parts.push('zero:' + (probe(0) === Event.BUBBLING_PHASE));\
             parts.push('emptyStr:' + (probe('') === Event.BUBBLING_PHASE));\
             parts.push('str:' + (probe('AAAA') === Event.CAPTURING_PHASE));\
             parts.push('nullV:' + (probe(null) === Event.BUBBLING_PHASE));\
             parts.push('obj2:' + (probe({capture: 2}) === Event.CAPTURING_PHASE));\
             parts.push('obj0:' + (probe({capture: 0}) === Event.BUBBLING_PHASE));\
             parts.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "num:true|negNum:true|nan:true|zero:true|emptyStr:true|str:true|nullV:true|obj2:true|obj0:true",
        "R146 WebIDL boolean 转换（primitive 真值 → capture；NaN/0/'' → bubble）"
    );
}

/// R147：classic 脚本经间接 `(0,eval)` 执行时，源内 `'use strict'` 指令使 eval 建独立
/// 变量环境——顶层 `function` 声明不落 globalThis（真实浏览器 classic 脚本即便 strict
/// 也创建全局绑定）。`script_run_classic_page` 的全局发布：行首零缩进 `function NAME(`
/// 声明在 eval 源尾拼接 `;globalThis.NAME=NAME;`（WPT 外链测试库如
/// prefixed-animation-event-tests.js 跨 `<script>` 可见性；webkit-animation 簇）。
#[test]
fn test_classic_script_strict_function_globals_r147() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    // 两段「页面脚本」按 run_page_scripts 的 strict 形态执行：第一段声明（'use strict'
    // 单引号 + 双引号两形态 + IIFE 内缩进函数不误发布），第二段跨脚本消费。
    let first = crate::js_dom_bridge::script_run_classic_page(
        "'use strict';\nfunction topFnA() { return 1; }\n\"use strict\";\nfunction topFnB() { return 2; }\n(function(){\n  function innerFn() { return 3; }\n  globalThis.__innerRef = typeof innerFn;\n})();\n",
        0,
    );
    let second = crate::js_dom_bridge::script_run_classic_page(
        "globalThis.__probe = [typeof topFnA, typeof topFnB, String(topFnA() + topFnB()), globalThis.__innerRef].join(',');",
        1,
    );
    sandbox.execute(&first).unwrap();
    sandbox.execute(&second).unwrap();
    let out = sandbox
        .execute("globalThis.__probe")
        .unwrap()
        .value;
    assert_eq!(
        out,
        "function,function,3,function",
        "R147 strict classic 脚本顶层函数声明经全局发布跨脚本可见（IIFE 内函数不受影响）"
    );
    // sentinel 干净（无抛错）。
    let err = sandbox
        .execute(&crate::js_dom_bridge::page_script_error_check())
        .unwrap()
        .value;
    assert_eq!(err, "", "R147 两段脚本均无抛错（sentinel 干净）");
}

/// R198：strict classic 脚本顶层 `const` / `let` 声明与 R147 的 function 同款全局发布
///（WPT dom/nodes/support/NodeList-static-length-tampered.js 顶层
/// `const indexOfNodeList = new Function(...)` 跨 `<script>` 不可见 → 后续脚本
/// "is not defined"）。IIFE/块内缩进 const 不误发布。
#[test]
fn test_classic_script_strict_const_let_globals_r198() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let first = crate::js_dom_bridge::script_run_classic_page(
        "'use strict';\nconst topConstA = new Function('return 41;');\nlet topLetB = 1;\n(function(){\n  const innerConst = 7;\n  globalThis.__innerConstRef = typeof innerConst;\n})();\n",
        0,
    );
    let second = crate::js_dom_bridge::script_run_classic_page(
        "globalThis.__probe = [typeof topConstA, String(topConstA() + 1), String(topLetB), globalThis.__innerConstRef].join(',');",
        1,
    );
    sandbox.execute(&first).unwrap();
    sandbox.execute(&second).unwrap();
    let err = sandbox
        .execute(&crate::js_dom_bridge::page_script_error_check())
        .unwrap()
        .value;
    assert_eq!(err, "", "R198 两段脚本均无抛错（sentinel 干净）");
    let out = sandbox.execute("globalThis.__probe").unwrap().value;
    // `__innerConstRef` 是 IIFE 内 `typeof innerConst`（局部可见 → "number"）；IIFE 的
    // const **未被发布**的验证在下一行（globalThis.innerConst undefined）。
    assert_eq!(
        out, "function,42,1,number",
        "R198 strict classic 顶层 const/let 经全局发布跨脚本可见"
    );
    assert_eq!(
        sandbox.execute("String(typeof globalThis.innerConst)").unwrap().value,
        "undefined",
        "R198 IIFE 内缩进 const 不被误发布到 globalThis"
    );
}

/// R201：strict classic 顶层 `var` 声明的 **accessor 转发导出**（WPT
/// dom/ranges/Range-mutations.js 的 `var insertDataTests = []` / dom/common.js 的
/// 多行 `var testDiv, paras,\n    foreignDoc, ...` 跨 `<script>` 可见性）。
/// 与 const/let 的值快照不同，var 有「声明后跨脚本再赋值」流——setupRangeTests
/// 在 harness 回调里赋值、后续脚本读取。accessor get/set 双向转发 eval 绑定。
/// 多行 var 的缩进续行裸声明符同属顶层（common.js 七行形态）。
#[test]
fn test_classic_script_strict_var_accessor_export_r201() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    // 第一段：多行 var（续行缩进）+ 单行 var + 带初始化首名 + 函数（R147 路径）。
    let first = crate::js_dom_bridge::script_run_classic_page(
        "'use strict';\nvar testTable = [1, 2, 3];\nvar holder, dependent,\n    continued, tailName;\nvar initialized = 42;\nfunction assignLater() { holder = 'assigned'; continued = 'cont'; tailName = 'tail'; }\n",
        0,
    );
    sandbox.execute(&first).unwrap();
    let err = sandbox
        .execute(&crate::js_dom_bridge::page_script_error_check())
        .unwrap()
        .value;
    assert_eq!(err, "", "R201 声明段无抛错（sentinel 干净）");
    // 第二段：跨脚本读（初始化值 + 未赋值 undefined）+ 跨脚本赋值后再读。
    // 跨脚本裸标识符读取本就不可见（strict eval 作用域）——消费方经 globalThis.X
    //（accessor）读写；声明脚本内的函数（assignLater）赋值 eval 绑定，accessor 读得见。
    let second = crate::js_dom_bridge::script_run_classic_page(
        "globalThis.__p1 = [String(globalThis.testTable.join('-')), String(typeof globalThis.holder), String(typeof globalThis.continued), String(globalThis.initialized)].join(',');\nglobalThis.testTable = [4];\nassignLater();\nglobalThis.__p2 = [globalThis.holder, globalThis.continued, globalThis.tailName, String(globalThis.testTable.join('-'))].join(',');",
        1,
    );
    sandbox.execute(&second).unwrap();
    let err2 = sandbox
        .execute(&crate::js_dom_bridge::page_script_error_check())
        .unwrap()
        .value;
    assert_eq!(err2, "", "R201 消费段无抛错（sentinel 干净）");
    assert_eq!(
        sandbox.execute("globalThis.__p1").unwrap().value,
        "1-2-3,undefined,undefined,42",
        "R201 初始化值快照读 + 未赋值 var 读 undefined（accessor get 转发）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__p2").unwrap().value,
        "assigned,cont,tail,4",
        "R201 跨脚本赋值 + 声明脚本内函数赋值经 accessor 双向可见（set 转发）"
    );
}

/// Cache API 初始页面表面：`caches.open()` / `Cache.put()` / `Cache.match()` 经 host bridge
/// 往返，match miss 解析为 undefined。
#[test]
fn test_cache_api_page_shim_host_roundtrip() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(|args| {
            let request = args.first().map(String::as_str).unwrap_or("");
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"v1","cache_id":7}"#.to_string();
            }
            if request.contains(r#""op":"put""#)
                && request.contains(r#""cache_name":"v1""#)
                && request.contains(r#""cache_id":7"#)
                && request.contains(r#""url":"https://example.com/data.txt""#)
                && request.contains(r#""url":"""#)
                && request.contains(r#""status":201"#)
                && request.contains(r#""body":"cached text""#)
            {
                return r#"__zw_cache_ok:{"ok":true}"#.to_string();
            }
            if request.contains(r#""op":"match""#)
                && request.contains(r#""cache_name":"v1""#)
                && request.contains(r#""cache_id":7"#)
            {
                return "__zw_cache_ok:{\"response\":\"__zwcr2:201\\u001fCreated\\u001fbasic\\u001fhttps://example.com/fetched.txt\\u001fcontent-type\\u001etext/plain\\u001fcached text\"}".to_string();
            }
            if request.contains(r#""op":"match_all""#)
                && request.contains(r#""cache_name":"v1""#)
                && request.contains(r#""cache_id":7"#)
            {
                return "__zw_cache_ok:{\"responses\":[\"__zwfr:201\\u001fCreated\\u001fcontent-type\\u001etext/plain\\u001fcached text\"]}".to_string();
            }
            if request.contains(r#""op":"cache_keys""#)
                && request.contains(r#""cache_name":"v1""#)
                && request.contains(r#""cache_id":7"#)
            {
                if request.contains(r#""method":"POST""#) {
                    return r#"__zw_cache_ok:{"requests":[]}"#.to_string();
                }
                return r#"__zw_cache_ok:{"requests":[{"url":"https://example.com/data.txt","method":"GET"}]}"#
                    .to_string();
            }
            if request.contains(r#""op":"match""#) {
                return r#"__zw_cache_ok:{"response":null}"#.to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__cacheDone = 'pending';\
             caches.open('v1').then(function (cache) {\
               return cache.put('https://example.com/data.txt', new Response('cached text', {\
                 status: 201,\
                 statusText: 'Created',\
                 headers: {'content-type': 'text/plain'}\
               })).then(function () { return Promise.all([\
                 cache.match('https://example.com/data.txt'),\
                 cache.matchAll('https://example.com/data.txt'),\
                 cache.keys(),\
                 cache.keys(new Request('https://example.com/data.txt', {method: 'POST'}))\
               ]); });\
             }).then(function (values) {\
               var response = values[0];\
               var responses = values[1];\
               var requests = values[2];\
               var filteredRequests = values[3];\
               return response.text().then(function (body) {\
                 globalThis.__cacheDone = [\
                   String(response instanceof Response),\
                   Object.prototype.toString.call(response),\
                   Object.prototype.toString.call(response.headers),\
                   String(responses.length),\
                   String(responses[0] instanceof Response),\
                   Object.prototype.toString.call(responses[0]),\
                   Object.prototype.toString.call(responses[0].headers),\
                   String(requests.length),\
                   String(requests[0] instanceof Request),\
                   Object.prototype.toString.call(requests[0]),\
                   Object.prototype.toString.call(requests[0].headers),\
                   requests[0].method,\
                   String(filteredRequests.length),\
                   String(response.status),\
                   response.statusText,\
                   response.type,\
                   response.clone().type,\
                   response.url,\
                   response.clone().url,\
                   response.headers.get('content-type'),\
                   body\
                 ].join('|');\
               });\
             }, function (error) {\
               globalThis.__cacheDone = 'error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__cachePump = {i};")).unwrap();
    }
    assert_eq!(
        sandbox.execute("globalThis.__cacheDone").unwrap().value,
        "true|[object Response]|[object Headers]|1|true|[object Response]|[object Headers]|1|true|[object Request]|[object Headers]|GET|0|201|Created|basic|basic|https://example.com/fetched.txt|https://example.com/fetched.txt|text/plain|cached text",
        "Cache API page shim should round-trip Response through host bridge"
    );
}

/// Dedicated Worker sees the same `caches` object as the window shim. This
/// fixes the upstream `cache-storage/common.https.window.js` worker/window
/// sharing path.
#[test]
fn test_cache_api_dedicated_worker_uses_window_cache_storage_bridge() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let worker_put_seen = Arc::new(Mutex::new(false));
    let worker_put_seen_for_callback = worker_put_seen.clone();
    let last_request = Arc::new(Mutex::new(String::new()));
    let last_request_for_callback = last_request.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().map(String::as_str).unwrap_or("");
            *last_request_for_callback.lock().unwrap() = request.to_string();
            if request.contains(r#""op":"open""#) && request.contains(r#""name":"shared""#) {
                return r#"__zw_cache_ok:{"name":"shared","cache_id":9}"#.to_string();
            }
            if request.contains(r#""op":"put""#)
                && request.contains(r#""cache_name":"shared""#)
                && request.contains(r#""url":"https://example.com/from-worker""#)
                && request.contains(r#""body":"from-worker""#)
            {
                *worker_put_seen_for_callback.lock().unwrap() = true;
                return r#"__zw_cache_ok:{"ok":true}"#.to_string();
            }
            if request.contains(r#""op":"match""#)
                && request.contains(r#""cache_name":"shared""#)
                && *worker_put_seen_for_callback.lock().unwrap()
            {
                return "__zw_cache_ok:{\"response\":\"__zwcr2:200\\u001fOK\\u001fbasic\\u001fhttps://example.com/from-worker\\u001fcontent-type\\u001etext/plain\\u001ffrom-worker\"}".to_string();
            }
            if request.contains(r#""op":"delete""#) && request.contains(r#""name":"shared""#) && !request.contains(r#""request":"#) {
                *worker_put_seen_for_callback.lock().unwrap() = false;
                return r#"__zw_cache_ok:{"deleted":true}"#.to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__workerCacheResult = 'pending';\
             var source = \"self.onmessage = function () {\" +\
               \"self.caches.open('shared').then(function (cache) {\" +\
               \"return cache.put('https://example.com/from-worker', new Response('from-worker'));\" +\
               \"}).then(function () { self.postMessage('ok'); }, function (error) {\" +\
               \"self.postMessage('error:' + String(error && error.message ? error.message : error));\" +\
               \"}); };\";\
             caches.delete('shared').then(function () {\
               var worker = new Worker('data:text/javascript,' + encodeURIComponent(source));\
               return new Promise(function (resolve) {\
                 worker.addEventListener('message', function listener(event) {\
                   worker.removeEventListener('message', listener);\
                   resolve(event.data);\
                 });\
                 worker.postMessage('go');\
               });\
             }).then(function (message) {\
               if (message !== 'ok') return message;\
               return caches.open('shared').then(function (cache) {\
                 return cache.match('https://example.com/from-worker');\
               }).then(function (response) {\
                 return response.text();\
               });\
             }).then(function (body) {\
               globalThis.__workerCacheResult = body;\
             }, function (error) {\
               globalThis.__workerCacheResult = 'error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..12 {
        sandbox.execute(&format!("globalThis.__workerCachePump = {i};")).unwrap();
    }
    assert_eq!(
        sandbox.execute("globalThis.__workerCacheResult").unwrap().value,
        "from-worker",
        "Dedicated Worker Cache.put should use the same CacheStorage host bridge seen by window; last request: {}",
        last_request.lock().unwrap()
    );
}

/// Worker-created nested Workers resolve relative script URLs against the
/// parent worker script URL, not the page URL.
#[test]
fn test_dedicated_worker_nested_worker_resolves_against_parent_script_url() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let fetched = Arc::new(Mutex::new(Vec::new()));
    let fetched_for_callback = fetched.clone();
    sandbox.register_callback(
        "__zw_fetch_script",
        Box::new(move |args| {
            let page = args.first().map(String::as_str).unwrap_or("");
            let src = args.get(1).map(String::as_str).unwrap_or("");
            fetched_for_callback
                .lock()
                .unwrap()
                .push(format!("{page}|{src}"));
            match (page, src) {
                ("https://example.com/tests/page.html", "workers/cache-api-nested-worker1.js") => {
                    "const worker2 = new Worker('cache-api-nested-worker2.js');\
                     worker2.onmessage = function (event) { self.postMessage(event.data); };"
                        .to_string()
                }
                (
                    "https://example.com/tests/workers/cache-api-nested-worker1.js",
                    "cache-api-nested-worker2.js",
                ) => "self.caches.keys().then(function () { postMessage('PASS'); });".to_string(),
                _ => String::new(),
            }
        }),
    );
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(|args| {
            let request = args.first().map(String::as_str).unwrap_or("");
            if request.contains(r#""op":"keys""#) {
                return r#"__zw_cache_ok:{"keys":[]}"#.to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/tests/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__nestedWorkerResult = 'pending';\
             var worker = new Worker('workers/cache-api-nested-worker1.js');\
             worker.onmessage = function (event) { globalThis.__nestedWorkerResult = event.data; };",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__nestedWorkerPump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("globalThis.__nestedWorkerResult").unwrap().value,
        "PASS",
        "nested worker should run and see CacheStorage"
    );
    assert_eq!(
        fetched.lock().unwrap().as_slice(),
        [
            "https://example.com/tests/page.html|workers/cache-api-nested-worker1.js",
            "https://example.com/tests/workers/cache-api-nested-worker1.js|cache-api-nested-worker2.js"
        ],
        "nested worker script fetch should use parent worker URL as base"
    );
}

/// Cache API query options must be projected into the host bridge for the
/// storage owner to apply matching semantics.
#[test]
fn test_cache_api_page_shim_query_options_wire() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let seen_for_callback = seen.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            seen_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"v1","cache_id":11}"#.to_string();
            }
            if request.contains(r#""op":"match_all""#) {
                return r#"__zw_cache_ok:{"responses":[]}"#.to_string();
            }
            if request.contains(r#""op":"cache_keys""#) {
                return r#"__zw_cache_ok:{"requests":[]}"#.to_string();
            }
            if request.contains(r#""op":"match""#) {
                return r#"__zw_cache_ok:{"response":null}"#.to_string();
            }
            "__zw_cache_ok:{\"deleted\":true}".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__cacheOptionsDone = 'pending';\
             caches.open('v1').then(function (cache) {\
               return Promise.all([\
                 cache.match('https://example.com/data?one', {ignoreSearch: true, ignoreMethod: true}),\
                 cache.matchAll('https://example.com/data?two', {ignoreSearch: true}),\
                 cache.keys('https://example.com/data?three', {ignoreMethod: true}),\
                 cache.delete('https://example.com/data?four', {ignoreSearch: true, ignoreMethod: true}),\
                 caches.match('https://example.com/data?five', {cacheName: 'v1', ignoreSearch: true, ignoreMethod: true})\
               ]);\
             }).then(function () {\
               globalThis.__cacheOptionsDone = 'done';\
             }, function (error) {\
               globalThis.__cacheOptionsDone = 'error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__cacheOptionsPump = {i};")).unwrap();
    }
    assert_eq!(sandbox.execute("globalThis.__cacheOptionsDone").unwrap().value, "done");

    let seen = seen.lock().unwrap();
    assert!(seen.iter().any(|request| {
        request.contains(r#""op":"match""#)
            && request.contains(r#""ignoreSearch":true"#)
            && request.contains(r#""ignoreMethod":true"#)
    }));
    assert!(seen.iter().any(|request| {
        request.contains(r#""op":"match_all""#)
            && request.contains(r#""ignoreSearch":true"#)
            && request.contains(r#""ignoreMethod":false"#)
    }));
    assert!(seen.iter().any(|request| {
        request.contains(r#""op":"cache_keys""#)
            && request.contains(r#""ignoreSearch":false"#)
            && request.contains(r#""ignoreMethod":true"#)
    }));
    assert!(seen.iter().any(|request| {
        request.contains(r#""op":"delete""#)
            && request.contains(r#""ignoreSearch":true"#)
            && request.contains(r#""ignoreMethod":true"#)
    }));
    assert!(seen.iter().any(|request| {
        request.contains(r#""op":"match""#)
            && request.contains(r#""cache_name":"v1""#)
            && request.contains(r#""ignoreSearch":true"#)
    }));
}

/// CacheStorage cache names are WebIDL DOMStrings: embedded NUL and unpaired
/// surrogate code units must survive the page shim -> host JSON bridge.
#[test]
fn test_cache_storage_name_uses_domstring_code_units_in_host_wire() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let seen: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let seen_for_callback = seen.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            let response = if request.contains(r#""name_units":"0075006e007000610069007200650064d800""#) {
                r#"__zw_cache_ok:{"name_units":"0075006e007000610069007200650064d800"}"#
            } else {
                r#"__zw_cache_ok:{"name_units":"00630061006300680065002d00730074006f0072006100670065002f0068006100730000005f0069006e005f007400680065005f006e0061006d0065"}"#
            };
            seen_for_callback.lock().unwrap().push(request);
            response.to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    sandbox
        .execute(
            "globalThis.__cacheDomStringNames = 'pending';\
             caches.open('cache-storage/has\\000_in_the_name').then(function (cache) {\
               globalThis.__cacheDomStringNames = cache._name.charCodeAt(17) + ':' + cache._name.slice(18);\
               return caches.open('unpaired\\uD800');\
             }).then(function (cache) {\
               globalThis.__cacheDomStringNames += '|' + cache._name.charCodeAt(8).toString(16);\
             }, function (error) {\
               globalThis.__cacheDomStringNames = 'error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    sandbox.execute("globalThis.__cacheNulPump = 1;").unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__cacheDomStringNames").unwrap().value,
        "0:_in_the_name|d800"
    );
    let seen = seen.lock().unwrap();
    assert_eq!(seen.len(), 2);
    assert!(
        seen[0].contains(r#""name_units":"00630061006300680065002d00730074006f0072006100670065002f0068006100730000005f0069006e005f007400680065005f006e0061006d0065""#),
        "CacheStorage host wire should carry NUL as UTF-16 code units, got {:?}",
        seen[0]
    );
    assert!(
        seen[1].contains(r#""name_units":"0075006e007000610069007200650064d800""#),
        "CacheStorage host wire should carry unpaired surrogate as UTF-16 code units, got {:?}",
        seen[1]
    );
}

/// Required Cache API arguments should reject before the host bridge is called.
#[test]
fn test_cache_api_page_shim_required_arguments_reject() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_callback = calls.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            calls_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"v1"}"#.to_string();
            }
            "__zw_cache_ok:{\"deleted\":true}".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    sandbox
        .execute(
            "globalThis.__cacheRequiredArgs = 'pending';\
             var misses = [];\
             caches.open().then(function () { misses.push('open-resolved'); }, function (error) {\
               misses.push(String(error instanceof TypeError) + ':open');\
             }).then(function () {\
               return caches.open('v1');\
             }).then(function (cache) {\
               return cache.delete().then(function () { misses.push('delete-resolved'); }, function (error) {\
                 misses.push(String(error instanceof TypeError) + ':delete');\
               });\
             }).then(function () {\
               globalThis.__cacheRequiredArgs = misses.join('|');\
             }, function (error) {\
               globalThis.__cacheRequiredArgs = 'error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__cacheRequiredArgsPump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("globalThis.__cacheRequiredArgs").unwrap().value,
        "true:open|true:delete"
    );
    let calls = calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].contains(r#""op":"open""#));
    assert!(calls[0].contains(r#""name":"v1""#));
}

/// Cache.add/addAll should fetch GET requests and store the fetched responses
/// through the existing Cache.put host bridge.
#[test]
fn test_cache_api_page_shim_add_and_add_all_wire() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetches_for_callback = fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let method = args.get(1).cloned().unwrap_or_default();
            let url = args.get(2).cloned().unwrap_or_default();
            fetches_for_callback.lock().unwrap().push(format!("{method}:{url}"));
            match url.as_str() {
                "https://example.com/a.txt" => "__zwfr:200\x1fOK\x1fcontent-type\x1etext/plain\x1falpha".to_string(),
                "https://example.com/b.txt" => "__zwfr:200\x1fOK\x1fcontent-type\x1etext/plain\x1fbeta".to_string(),
                "https://example.com/missing.txt" => "__zwfr:404\x1fNot Found\x1f\x1fmissing".to_string(),
                _ => "__zw_fetch_error:not-found".to_string(),
            }
        }),
    );

    let puts: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let puts_for_callback = puts.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            puts_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"assets"}"#.to_string();
            }
            if request.contains(r#""op":"put""#) {
                return r#"__zw_cache_ok:{"ok":true}"#.to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__cacheAddDone = 'pending';\
             caches.open('assets').then(function (cache) {\
               return cache.add('https://example.com/a.txt').then(function () {\
                 return cache.addAll(['https://example.com/b.txt']);\
               }).then(function () {\
                 return cache.add(new Request('https://example.com/post.txt', {method: 'POST'}));\
               }).then(function () {\
                 globalThis.__cacheAddDone = 'post-resolved';\
               }, function (error) {\
                 globalThis.__cacheAddDone = String(error instanceof TypeError) + ':' + String(error.message);\
               });\
             }, function (error) {\
               globalThis.__cacheAddDone = 'open-error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__cacheAddPump = {i};")).unwrap();
    }
    assert_eq!(
        sandbox.execute("globalThis.__cacheAddDone").unwrap().value,
        "true:Cache.add only supports GET requests"
    );

    assert_eq!(
        *fetches.lock().unwrap(),
        vec![
            "GET:https://example.com/a.txt".to_string(),
            "GET:https://example.com/b.txt".to_string(),
        ]
    );
    let puts = puts.lock().unwrap();
    assert!(puts.iter().any(|request| {
        request.contains(r#""op":"put""#)
            && request.contains(r#""url":"https://example.com/a.txt""#)
            && request.contains(r#""body":"alpha""#)
    }));
    assert!(puts.iter().any(|request| {
        request.contains(r#""op":"put""#)
            && request.contains(r#""url":"https://example.com/b.txt""#)
            && request.contains(r#""body":"beta""#)
    }));
    assert!(!puts.iter().any(|request| request.contains("post.txt")));
}

/// Cache.addAll must validate all fetched responses before storing any entry.
#[test]
fn test_cache_api_page_shim_add_all_rejects_without_partial_puts() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetches_for_callback = fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).cloned().unwrap_or_default();
            fetches_for_callback.lock().unwrap().push(url.clone());
            match url.as_str() {
                "https://example.com/ok.txt" => "__zwfr:200\x1fOK\x1f\x1fok".to_string(),
                "https://example.com/partial.txt" => "__zwfr:206\x1fPartial Content\x1f\x1fpartial".to_string(),
                _ => "__zw_fetch_error:not-found".to_string(),
            }
        }),
    );

    let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_callback = calls.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            calls_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"assets"}"#.to_string();
            }
            if request.contains(r#""op":"cache_keys""#) {
                return r#"__zw_cache_ok:{"requests":[]}"#.to_string();
            }
            "__zw_cache_error:unexpected put".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__cacheAddAllAtomic = 'pending';\
             caches.open('assets').then(function (cache) {\
               return cache.addAll(['https://example.com/ok.txt', 'https://example.com/partial.txt'])\
                 .then(function () {\
                   globalThis.__cacheAddAllAtomic = 'resolved';\
                 }, function (error) {\
                   return cache.keys().then(function (keys) {\
                     globalThis.__cacheAddAllAtomic = [\
                       String(error instanceof TypeError),\
                       String(error.message),\
                       String(keys.length)\
                     ].join('|');\
                   });\
                 });\
             }, function (error) {\
               globalThis.__cacheAddAllAtomic = 'open-error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__cacheAddAllAtomicPump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("globalThis.__cacheAddAllAtomic").unwrap().value,
        "true|Cache.put cannot store a 206 Partial Content response|0"
    );
    assert_eq!(
        *fetches.lock().unwrap(),
        vec![
            "https://example.com/ok.txt".to_string(),
            "https://example.com/partial.txt".to_string(),
        ]
    );
    let calls = calls.lock().unwrap();
    assert!(calls.iter().any(|request| request.contains(r#""op":"open""#)));
    assert!(calls.iter().any(|request| request.contains(r#""op":"cache_keys""#)));
    assert!(!calls.iter().any(|request| request.contains(r#""op":"put""#)));
}

/// Cache.addAll duplicate detection uses the request URL without fragment,
/// matching Cache.put replacement semantics.
#[test]
fn test_cache_api_page_shim_add_all_rejects_fragment_duplicates() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_callback = calls.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            calls_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"assets"}"#.to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__cacheAddAllDuplicate = 'pending';\
             caches.open('assets').then(function (cache) {\
               return cache.addAll(['https://example.com/a.txt#one', 'https://example.com/a.txt#two'])\
                 .then(function () {\
                   globalThis.__cacheAddAllDuplicate = 'resolved';\
                 }, function (error) {\
                   globalThis.__cacheAddAllDuplicate = [\
                     String(error.name),\
                     String(error.message)\
                   ].join('|');\
                 });\
             }, function (error) {\
               globalThis.__cacheAddAllDuplicate = 'open-error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    sandbox.execute("globalThis.__cacheAddAllDuplicatePump = 1;").unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__cacheAddAllDuplicate").unwrap().value,
        "InvalidStateError|Cache.addAll duplicate requests"
    );
    let calls = calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert!(calls[0].contains(r#""op":"open""#));
}

/// Cache.addAll duplicate checks must account for the fetched response Vary
/// headers, and invalid request list entries must reject before fetch.
#[test]
fn test_cache_api_page_shim_add_all_validates_entries_and_vary_duplicates() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetches_for_callback = fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).cloned().unwrap_or_default();
            let headers = args.get(3).cloned().unwrap_or_default();
            fetches_for_callback
                .lock()
                .unwrap()
                .push(format!("{url}|{headers}"));
            let vary = if url.contains("size-dup.txt") {
                "x-size"
            } else {
                "x-shape"
            };
            format!("__zwfr:200\x1fOK\x1fvary\x1e{vary}\x1fbody")
        }),
    );

    let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_callback = calls.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            calls_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"assets"}"#.to_string();
            }
            if request.contains(r#""op":"put""#) {
                return r#"__zw_cache_ok:{"ok":true}"#.to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__cacheAddAllVary = 'pending';\
             caches.open('assets').then(function (cache) {\
               return Promise.all([\
                 cache.addAll(['https://example.com/a.txt', undefined]).then(function () { return 'undefined-resolved'; }, function (e) { return e instanceof TypeError ? 'undefined-rejected' : 'undefined-other'; }),\
                 cache.addAll([\
                   new Request('https://example.com/vary.txt', {headers: {'x-shape': 'circle'}}),\
                   new Request('https://example.com/vary.txt', {headers: {'x-shape': 'square'}})\
                 ]).then(function () { return 'vary-distinct-resolved'; }, function (e) { return 'vary-distinct-rejected:' + e.name; }),\
                 cache.addAll([\
                   new Request('https://example.com/size-dup.txt', {headers: {'x-shape': 'circle', 'x-size': 'big'}}),\
                   new Request('https://example.com/size-dup.txt', {headers: {'x-shape': 'square', 'x-size': 'big'}})\
                 ]).then(function () { return 'vary-dup-resolved'; }, function (e) { return 'vary-dup-rejected:' + e.name; })\
               ]);\
             }).then(function (values) {\
               globalThis.__cacheAddAllVary = values.join('|');\
             }, function (error) {\
               globalThis.__cacheAddAllVary = 'outer-error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__cacheAddAllVaryPump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("globalThis.__cacheAddAllVary").unwrap().value,
        "undefined-rejected|vary-distinct-resolved|vary-dup-rejected:InvalidStateError"
    );
    let fetches = fetches.lock().unwrap();
    assert_eq!(fetches.len(), 4);
    assert!(fetches.iter().any(|request| request.contains("x-shape\u{1e}circle")));
    assert!(fetches.iter().any(|request| request.contains("x-shape\u{1e}square")));
    let calls = calls.lock().unwrap();
    assert_eq!(calls.iter().filter(|request| request.contains(r#""op":"put""#)).count(), 2);
}

/// Cache.put should pass an error filtered response through to CacheStorage.
#[test]
fn test_cache_api_page_shim_puts_error_response() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_callback = calls.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            calls_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"assets"}"#.to_string();
            }
            if request.contains(r#""op":"put""#) && request.contains(r#""type":"error""#) {
                return "__zw_cache_ok:{}".to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__cacheErrorPut = 'pending';\
             caches.open('assets').then(function (cache) {\
               return cache.put('https://example.com/error.txt', Response.error())\
                 .then(function () {\
                   globalThis.__cacheErrorPut = 'resolved';\
                 }, function (error) {\
                   globalThis.__cacheErrorPut = [\
                     String(error instanceof TypeError),\
                     String(error.message)\
                   ].join('|');\
                 });\
             }, function (error) {\
               globalThis.__cacheErrorPut = 'open-error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__cacheErrorPutPump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("globalThis.__cacheErrorPut").unwrap().value,
        "resolved"
    );
    let calls = calls.lock().unwrap();
    assert_eq!(calls.len(), 2);
    assert!(calls[0].contains(r#""op":"open""#));
    assert!(calls[1].contains(r#""op":"put""#));
    assert!(calls[1].contains(r#""type":"error""#));
}

/// Cache.put validates the WebIDL Response argument and keeps opaque filtered
/// responses cacheable even when their internal response has normally
/// uncacheable HTTP metadata.
#[test]
fn test_cache_api_page_shim_put_response_validation_and_opaque_internal_response() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let calls_for_callback = calls.clone();
    sandbox.register_callback(
        "__zw_cache_storage",
        Box::new(move |args| {
            let request = args.first().cloned().unwrap_or_default();
            calls_for_callback.lock().unwrap().push(request.clone());
            if request.contains(r#""op":"open""#) {
                return r#"__zw_cache_ok:{"name":"assets"}"#.to_string();
            }
            if request.contains(r#""op":"put""#) {
                return "__zw_cache_ok:{}".to_string();
            }
            "__zw_cache_error:unexpected request".to_string()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page.html".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__cachePutValidation = 'pending';\
             caches.open('assets').then(function (cache) {\
               var consumed = new Response('consume-me');\
               var empty = new Response();\
               var opaque = new Response('hidden');\
               opaque.type = 'opaque';\
               opaque.status = 0;\
               opaque.ok = false;\
               opaque._bodyText = '';\
               opaque._bodyNull = true;\
               opaque._zwOpaqueStatus = 206;\
               opaque._zwOpaqueStatusText = 'Partial Content';\
               opaque._zwOpaqueHeaders = new Headers({'Vary': '*'});\
               opaque._zwOpaqueBodyText = 'hidden';\
               return Promise.all([\
                 cache.put('https://example.com/bad.txt', 'not response').then(function () { return 'bad-resolved'; }, function (e) { return e instanceof TypeError ? 'bad-rejected' : 'bad-other'; }),\
                 cache.put('https://example.com/opaque.txt', opaque).then(function () { return 'opaque-resolved'; }, function (e) { return 'opaque-rejected:' + e.message; }),\
                 cache.put('https://example.com/consume.txt', consumed).then(function () {\
                   var readerResult = 'unset';\
                   try { consumed.body.getReader(); readerResult = 'reader-open'; }\
                   catch (e) { readerResult = e instanceof TypeError ? 'reader-locked' : 'reader-other'; }\
                   return 'used:' + String(consumed.bodyUsed) + '/' + readerResult;\
                 }, function (e) { return 'consume-rejected:' + e.message; }),\
                 cache.put('https://example.com/empty.txt', empty).then(function () { return 'empty-used:' + String(empty.bodyUsed); }, function (e) { return 'empty-rejected:' + e.message; })\
               ]);\
             }).then(function (values) {\
               globalThis.__cachePutValidation = values.join('|');\
             }, function (error) {\
               globalThis.__cachePutValidation = 'outer-error:' + String(error && error.message ? error.message : error);\
             });",
        )
        .unwrap();
    for i in 0..8 {
        sandbox.execute(&format!("globalThis.__cachePutValidationPump = {i};")).unwrap();
    }

    assert_eq!(
        sandbox.execute("globalThis.__cachePutValidation").unwrap().value,
        "bad-rejected|opaque-resolved|used:true/reader-locked|empty-used:false"
    );
    let calls = calls.lock().unwrap();
    let put_calls = calls
        .iter()
        .filter(|call| call.contains(r#""op":"put""#))
        .collect::<Vec<_>>();
    assert_eq!(put_calls.len(), 3);
    let opaque_put = put_calls
        .iter()
        .find(|call| call.contains(r#""type":"opaque""#))
        .expect("opaque put request should be sent to the host");
    assert!(opaque_put.contains(r#""status":206"#));
    assert!(opaque_put.contains(r#""vary\u001e*""#));
}

/// Cache API shim 在没有宿主 bridge 时应 reject，而不是悬挂 Promise。
#[test]
fn test_cache_api_page_shim_rejects_without_host_bridge() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    sandbox
        .execute(
            "globalThis.__cacheNoHost = 'pending';\
             caches.open('v1').then(function () {\
               globalThis.__cacheNoHost = 'resolved';\
             }, function (error) {\
               globalThis.__cacheNoHost = String(error instanceof TypeError) + ':' + String(error.message);\
             });",
        )
        .unwrap();
    sandbox.execute("globalThis.__cachePump = 1;").unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__cacheNoHost").unwrap().value,
        "true:CacheStorage host bridge is unavailable",
        "missing CacheStorage host bridge should reject predictably"
    );
}

/// R148：焦点所有权统一（proxy `_activeElKey` 与解析节点 `_zwMElFocused` 互斥）+
/// focus 事件 relatedTarget 的 shadow retargeting（旧焦点在 shadow 树内 → 泄露边界外
/// 以 shadow host 为 relatedTarget；WPT shadow-relatedTarget 两 subtest）。
#[test]
fn test_focus_ownership_and_related_target_r148() {
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
        Arc::new(Mutex::new("<html><body><div id=\"host\"></div><input id=\"lightInput\"></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             var host = document.getElementById('host');\
             var root = host.attachShadow({ mode: 'closed' });\
             root.innerHTML = \"<input id='shadowInput'>\";\
             var shadowInput = root.getElementById('shadowInput');\
             parts.push('shadowInput:' + (shadowInput ? 'y' : 'n'));\
             shadowInput.focus();\
             parts.push('activeIsShadow:' + (document.activeElement === shadowInput));\
             var lightInput = document.getElementById('lightInput');\
             var related = 'unset';\
             lightInput.addEventListener('focus', function(e) {\
               related = (e.relatedTarget === host) ? 'host' : (e.relatedTarget == null ? 'null' : 'other');\
             });\
             lightInput.focus();\
             parts.push('related:' + related);\
             parts.push('activeIsLight:' + (document.activeElement === lightInput));\
             parts.push('mElCleared:' + (globalThis._zwMElFocused == null));\
             parts.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "shadowInput:y|activeIsShadow:true|related:host|activeIsLight:true|mElCleared:true",
        "R148 焦点所有权互斥 + relatedTarget shadow retargeting（host 泄露边界）"
    );
}

/// R149：`customElements.define` 的既有元素自动升级（spec `custom-element-registration`
/// define 末步——文档中已存在的同名元素升级：ctor 体 + connectedCallback 立即触发；
/// WPT EventTarget-add-listener-platform-object：parser 先建 `<my-custom-click>` 后
/// define，connectedCallback 须跑使 addEventListener 注册）+ upgrade 初始 attr change
/// 仅对**存在**的 observed 属性派发（缺失不派发——真实浏览器一致，R3205 语义）。
#[test]
fn test_define_auto_upgrade_existing_elements_r149() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 既有元素带一个 pre-set observed 属性 'greet=hi'（升级初始 attr change 须派发一次）
    // + 'foo' 未设（无回调——后续 setAttribute 的 null->a 是首个 foo 回调）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><auto-ce id=\"ace\" greet=\"hi\"></auto-ce></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             globalThis.__log = [];\
             class AutoCE extends HTMLElement {\
               constructor() { super(); globalThis.__ctorRan = true; }\
               connectedCallback() { globalThis.__connectedRan = true; }\
               static get observedAttributes() { return ['greet', 'foo']; }\
               attributeChangedCallback(n, o, v) { globalThis.__log.push(n + ':' + o + '->' + v); }\
             }\
             customElements.define('auto-ce', AutoCE);\
             parts.push('ctor:' + (globalThis.__ctorRan || false));\
             parts.push('connected:' + (globalThis.__connectedRan || false));\
             var el = document.querySelector('#ace');\
             el.setAttribute('foo', 'a');\
             parts.push('log:' + globalThis.__log.join('|'));\
             parts.push('instance:' + (el instanceof AutoCE));\
             parts.join('~')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "ctor:true~connected:true~log:greet:null->hi|foo:null->a~instance:true",
        "R149 define 自动升级既有元素（ctor + connectedCallback + 存在属性的初始 attr change；缺失属性无回调）"
    );
}

/// R150①：MouseEvent offsetX/offsetY 派发期计算——spec CSSOM View §dom-mouseevent-offsetx
///（offset = client − target padding 边缘，近似 gBCR 左/上）。构造未显式给 offset init 时
/// 派发到 target 后计算；显式 init 保持构造值（WPT mouse-event-retarget：clientX 50 派发到
/// margin 8px 下的 target，offsetX 期望 42——本测试经 mock rect bridge 左缘 8 验证同语义）。
#[test]
fn test_mouse_event_offset_xy_dispatch_computed_r150() {
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
        Arc::new(Mutex::new("<html><body><div id=\"t\">x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // mock rect bridge：#t → "8,8,100,50"（padding 边缘近似 gBCR 左/上 = 8）。
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|_args: &[String]| -> String { "8,8,100,50".to_string() }),
    );

    let out = sandbox
        .execute(
            "var log = [];\
             var t = document.querySelector('#t');\
             t.addEventListener('click', function (e) {\
               log.push('computed:' + e.offsetX + ',' + e.offsetY);\
             });\
             t.dispatchEvent(new MouseEvent('click', { clientX: 50, clientY: 30 }));\
             var ev2 = new MouseEvent('click', { clientX: 50, clientY: 30, offsetX: 7, offsetY: 9 });\
             log.push('explicit:' + ev2.offsetX + ',' + ev2.offsetY);\
             log.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "computed:42,22|explicit:7,9",
        "R150 offsetX/offsetY：派发期 = client − gBCR 左/上（50−8=42 / 30−8=22）；显式 init 保持构造值"
    );
}

/// R150②：Event timeStamp 量化到 5µs（0.005ms）——定时侧信道缓解（WPT
/// Event-timestamp-safe-resolution 千样本 GCD ≥ 5µs）。断言两点：① 相邻构造事件差值
/// 的量化粒度（任意两值之差 × 200 恒整数）② 量化不破坏单调非递减（后构造 ≥ 先构造）。
#[test]
fn test_event_timestamp_quantized_5us_r150() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var ts = [];\
             for (var i = 0; i < 200; i++) {\
               ts.push(new Event('x').timeStamp);\
               for (var j = 0; j < 50; j++) { /* 忙等扰动，制造未量化差值 */ }\
             }\
             var mono = true, quant = true;\
             for (var i = 1; i < ts.length; i++) {\
               if (ts[i] < ts[i - 1]) mono = false;\
               var d = (ts[i] - ts[i - 1]) * 200;\
               if (Math.abs(d - Math.round(d)) > 1e-9) quant = false;\
             }\
             'mono:' + mono + '~quant:' + quant + '~first:' + (ts[0] > 0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "mono:true~quant:true~first:true",
        "R150 timeStamp 5µs 量化：单调保持 + 差值粒度 0.005ms 整数倍 + 首值 > 0（ceil 不归零）"
    );
}

/// R150③：GamepadEvent 构造器（spec Gamepad §gamepadevent）——WPT
/// Event-timestamp-high-resolution.https 断言 `new GamepadEvent('gamepadconnected')`
/// 可构造 + timeStamp 与 performance.now 同源单调。断言构造面 + gamepad 属性默认 null。
#[test]
fn test_gamepad_event_constructor_r150() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var ev = new GamepadEvent('gamepadconnected');\
             var ev2 = new GamepadEvent('gamepaddisconnected', { gamepad: { id: 'x' } });\
             (ev instanceof GamepadEvent) + '~' + (ev instanceof Event) + '~'\
               + (ev.gamepad === null) + '~' + (ev2.gamepad && ev2.gamepad.id)\
               + '~' + (ev.timeStamp > 0)",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "true~true~true~x~true",
        "R150 GamepadEvent：instanceof 链 + gamepad 默认 null / init 透传 + timeStamp 单调正值"
    );
}

/// R151①：`cloneNode(true)` 对含 markup 的源同步填充 JS 侧 registry（`_handleChildren[nh]`）
/// ——旧版只写 host mutation（异步 apply），克隆元素的 childNodes/children/
/// getElementsByClassName 在本 turn 内读 registry 全空（WPT Event-dispatch-single-activation
/// 的 `getContainer(parent)` 查询空 → undefined.appendChild 崩）。
#[test]
fn test_clone_node_deep_fills_child_registry_r151() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // sel 源元素带子（querySelector 路径——host 快照有 innerHTML）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"src\"><span class=\"kid\">a</span><b class=\"kid\">c</b></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var src = document.querySelector('#src');\
             var c = src.cloneNode(true);\
             var parts = [];\
             parts.push('kids:' + c.childNodes.length);\
             parts.push('children:' + c.children.length);\
             parts.push('gbc:' + c.getElementsByClassName('kid').length);\
             parts.join('~')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "kids:2~children:2~gbc:2",
        "R151 cloneNode deep：克隆元素本 turn 内 childNodes/children/getElementsByClassName 可见"
    );
}

/// R151②：`template.content` fragment 视图的 ParentNode API（children/firstElementChild/
/// childElementCount/getElementsByClassName）——WPT Event-dispatch-single-activation 的
/// `Array.from(template.content.children)` + `getElementsByClassNameInclusive`。
#[test]
fn test_template_content_parent_node_apis_r151() {
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
        "<html><body><template id=\"t\"><form class=\"a\"><input class=\"b\"></form>x</template></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var t = document.querySelector('#t');\
             var c = t.content;\
             var parts = [];\
             parts.push('children:' + c.children.length);\
             parts.push('firstEl:' + (c.firstElementChild && c.firstElementChild.tagName));\
             parts.push('count:' + c.childElementCount);\
             parts.push('gbc:' + c.getElementsByClassName('b').length);\
             parts.push('deepgbc:' + c.getElementsByClassName('a').length);\
             parts.push('childNodes:' + c.childNodes.length);\
             parts.join('~')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "children:1~firstEl:FORM~count:1~gbc:1~deepgbc:1~childNodes:2",
        "R151 template.content ParentNode API（children/元素子导航/类名子树查询）"
    );
}

/// R151③：`createEvent('KeyboardEvents')` 保持 spec 别名表语义（**抛** NotSupportedError
/// ——spec `dom-document-createevent` 复数别名仅 Events/HTMLEvents/SVGEvents/MouseEvents/
/// UIEvents；WPT Document-createEvent.https 断言）。keypress-dispatch-crash 的 vacuous
/// pass 由 runner 侧「零 test() 声明 + 脚本中断 = no-crash 达成」处理，不在 shim 加别名。
#[test]
fn test_create_event_keyboard_events_plural_throws_r151() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var r = '';\
             try { document.createEvent('KeyboardEvents'); r = 'no-throw'; }\
             catch (e) { r = e.name; }\
             var ok2 = '';\
             try { var ev = document.createEvent('KeyboardEvent'); ev.initKeyboardEvent('keypress');\
                   r += '|singular-ok:' + (ev.type === 'keypress'); }\
             catch (e2) { r += '|singular-throw'; }\
             r",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "NotSupportedError|singular-ok:true",
        "R151 createEvent：KeyboardEvents 复数抛（spec 别名表）/ 单数正常 + initKeyboardEvent"
    );
}

/// R151④：_zwMEl 解析节点（template.content 克隆子树）的 `click()` + `classList`——
/// WPT Event-dispatch-single-activation 对克隆子树 click 目标调 `.click()`、对激活元素
/// 调 `e.classList.add('test'+i)`（旧缺方法 TypeError）。
#[test]
fn test_zw_mel_click_and_classlist_r151() {
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
        "<html><body><template id=\"t\"><input class=\"c1\" type=\"checkbox\"></template></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var t = document.querySelector('#t');\
             var input = t.content.children[0].cloneNode(true);\
             var parts = [];\
             parts.push('classList:' + (typeof input.classList.add));\
             input.classList.add('x1');\
             parts.push('cls:' + input.getAttribute('class'));\
             parts.push('contains:' + input.classList.contains('x1'));\
             input.classList.remove('x1');\
             parts.push('removed:' + input.classList.contains('x1'));\
             parts.push('toggle:' + input.classList.toggle('x2'));\
             var clicked = 0;\
             input.addEventListener('click', function () { clicked++; });\
             parts.push('click:' + (typeof input.click));\
             input.click();\
             parts.push('fired:' + clicked);\
             parts.join('~')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "classList:function~cls:c1 x1~contains:true~removed:false~toggle:true~click:function~fired:1",
        "R151 _zwMEl classList（add/remove/toggle/contains 写回属性）+ click()（本地派发触发 listener）"
    );
}

// R152（js-dom M4）：inline handler `with(this)` 的 unscopables 豁免面（spec WebIDL
// §[Unscopable]——WPT remove-unscopable 六断言：bare `remove` 解析 window 全局、
// `this.remove` 是 function）。根因：R134 的 setAttribute('on*') 重编译**回读 host 快照**
// （`__zw_get_attr`），而 `__zw_set_attr` 是异步 mutation 批处理不立即落快照 → 重编译出
// 旧 fn → 六变体 dispatch 永远只跑首个 handler 体（remove/prepend/append 三 Pass 是因为
// 首个体恰好是 remove；before/after/replaceWith 的 result1 undefined = 首体未豁免这些名）。
// 修复：`_ensureInlineHandler` 加 codeOverride 参数，R134 调用点传刚写入的值 v。
#[test]
fn test_inline_handler_unscopable_full_family_r152() {
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
        "<html><body><div id=\"testDiv\" onclick=\"result1 = remove; result2 = this.remove;\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             var result1 = undefined, result2 = undefined;\
             var unscopables = ['before', 'after', 'replaceWith', 'remove', 'prepend', 'append'];\
             for (var i in unscopables) {\
               var name = unscopables[i];\
               window[name] = 'Hello there';\
               result1 = result2 = undefined;\
               var div = document.querySelector('#testDiv');\
               div.setAttribute('onclick', 'result1 = ' + name + '; result2 = this.' + name + ';');\
               div.dispatchEvent(new Event('click'));\
               parts.push(name + ':' + (typeof result1) + '/' + (typeof result2));\
             }\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "before:string/function|after:string/function|replaceWith:string/function|remove:string/function|prepend:string/function|append:string/function",
        "R152 inline handler with(this) unscopables 六方法全豁免（bare 名解析 window，this.<name> 是 function）"
    );
}

// R152（js-dom M4）：`lookupNamespaceURI`/`isDefaultNamespace`（spec
// `dom-node-lookupnamespaceuri`——沿祖先链扫 xmlns 声明 + 元素自身 prefix→ns 映射 +
// xml/xmlns 预绑定仅元素/文档分支生效）。WPT Node-lookupNamespaceURI 75 断言 0F 双路径。
#[test]
fn test_lookup_namespace_uri_family_r152() {
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
             try {\
             var XML_NS = 'http://www.w3.org/XML/1998/namespace';\
             var XMLNS_NS = 'http://www.w3.org/2000/xmlns/';\
             // ① detached fragment：非元素起点无预绑定、链空 → 恒 null；default ns 空 → isDefault(null) true
             var frag = document.createDocumentFragment();\
             parts.push('frag-xml:' + (frag.lookupNamespaceURI('xml') === null));\
             parts.push('frag-default:' + (frag.lookupNamespaceURI(null) === null));\
             parts.push('frag-isdef:' + frag.isDefaultNamespace(null));\
             // ② 元素预绑定 + 自身 prefix→ns 映射（有 prefix 元素的 ns 非 default）
             var el = document.createElementNS('fooNS', 'prefix:elem');\
             parts.push('el-xml:' + (el.lookupNamespaceURI('xml') === XML_NS));\
             parts.push('el-xmlns:' + (el.lookupNamespaceURI('xmlns') === XMLNS_NS));\
             parts.push('el-prefix:' + (el.lookupNamespaceURI('prefix') === 'fooNS'));\
             parts.push('el-default-null:' + (el.lookupNamespaceURI(null) === null));\
             // ③ xmlns 声明属性：default + 前缀
             el.setAttributeNS(XMLNS_NS, 'xmlns:bar', 'barURI');\
             el.setAttributeNS(XMLNS_NS, 'xmlns', 'bazURI');\
             parts.push('el-decl-default:' + (el.lookupNamespaceURI(null) === 'bazURI'));\
             parts.push('el-decl-bar:' + (el.lookupNamespaceURI('bar') === 'barURI'));\
             parts.push('el-isdef-baz:' + el.isDefaultNamespace('bazURI'));\
             // ④ 子元素继承 + 无 prefix 自身 ns 即 default
             var child = document.createElementNS('childNS', 'childElem');\
             el.appendChild(child);\
             parts.push('child-default:' + (child.lookupNamespaceURI(null) === 'childNS'));\
             parts.push('child-bar:' + (child.lookupNamespaceURI('bar') === 'barURI'));\
             parts.push('child-isdef-child:' + child.isDefaultNamespace('childNS'));\
             // ⑤ document：default 恒 HTML ns（不读 documentElement 声明）+ 前缀经声明 + 预绑定
             parts.push('doc-default:' + (document.lookupNamespaceURI(null) === 'http://www.w3.org/1999/xhtml'));\
             parts.push('doc-xml:' + (document.lookupNamespaceURI('xml') === XML_NS));\
             // ⑥ Attr 经 ownerElement；disconnected 恒 null
             var attr = document.createAttribute('foo');\
             parts.push('attr-disc:' + (attr.lookupNamespaceURI('xml') === null));\
             document.getElementById('d').setAttributeNode(attr);\
             parts.push('attr-conn-xml:' + (attr.lookupNamespaceURI('xml') === XML_NS));\
             // ⑦ doctype 恒 null / default 空
             parts.push('dt-null:' + (document.doctype.lookupNamespaceURI('foo') === null));\
             parts.push('dt-isdef:' + document.doctype.isDefaultNamespace(null));\
             // ⑧ new Document() 无 documentElement → xml/xmlns 无预绑定
             var d2 = new Document();\
             parts.push('newdoc-xml:' + (d2.lookupNamespaceURI('xml') === null));\
             } catch (e) { parts.push('THREW:' + e.message); }\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "frag-xml:true|frag-default:true|frag-isdef:true|el-xml:true|el-xmlns:true|el-prefix:true|el-default-null:true|el-decl-default:true|el-decl-bar:true|el-isdef-baz:true|child-default:true|child-bar:true|child-isdef-child:true|doc-default:true|doc-xml:true|attr-disc:true|attr-conn-xml:true|dt-null:true|dt-isdef:true|newdoc-xml:true",
        "R152 lookupNamespaceURI/isDefaultNamespace 全族（fragment/元素预绑定+自身映射/声明属性/子继承/document HTML ns/Attr ownerElement/doctype/new Document）"
    );
}

// R154（js-dom M4）：single-activation 的 A/AREA hash 导航诊断复现——A[click] 在
// INPUT[checkbox] 父内 click() 后，activation 应只经 window.onhashchange(newURL 字符串)
// 上报；旧版 got 元素对象（某个 activated(e) 收到元素）。
#[test]
fn test_a_click_hash_activation_in_checkbox_parent_r154() {
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
        "<html><body><div id=\"test_container\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/dom/events/x.html".to_string(),
    ));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             var activations = [];\
             globalThis.activated = function (e) { activations.push(typeof e === 'string' ? e : ('EL:' + (e && e.tagName))); };\
             window.onhashchange = function (e) {\
               if (String(e.newURL).endsWith('link')) activated(e.newURL);\
               window.location.hash = '';\
             };\
             var container = document.getElementById('test_container');\
             var parent = document.createElement('input'); parent.type = 'checkbox';\
             var a = document.createElement('a'); a.href = '#t1_link';\
             container.appendChild(parent);\
             parent.appendChild(a);\
             a.click();\
             parts.push('sync-activations:' + activations.length);\
             parts.push('parent-checked:' + parent.checked);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "sync-activations:0|parent-checked:false",
        "R154：A 在 INPUT[checkbox] 内 click——nearest activation 是 A（hash 导航），父 INPUT 不翻（single-activation a-in-input 根因）"
    );
}

// R154（js-dom M4）：single-activation 的 checkbox@FORM 簇——checkbox click 在
// detached clone 子树上须翻 checked + 派发 input 事件（oninput=activated 的链路）。
#[test]
fn test_checkbox_click_in_form_clone_r154() {
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
        "<html><body><div id=\"tc\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // ① createElement 路径：checkbox click → checked 翻转 + input 事件触发 inline
    // oninput handler（WPT single-activation 的 activation 判定链）。
    let out = sandbox
        .execute(
            "var parts = [];\
             var acts = [];\
             globalThis.__r154Act = function (e) { acts.push(e); };\
             var tc = document.getElementById('tc');\
             var form = document.createElement('form');\
             var sub = document.createElement('input'); sub.type = 'submit';\
             form.appendChild(sub);\
             var cb = document.createElement('input'); cb.type = 'checkbox';\
             cb.setAttribute('oninput', 'this.checked ? __r154Act(this) : null');\
             form.appendChild(cb);\
             tc.appendChild(form);\
             cb.click();\
             parts.push('checked:' + cb.checked);\
             parts.push('acts:' + acts.length);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "checked:true|acts:1",
        "R154：checkbox click——checked 翻转 + input 事件触发 inline oninput activated"
    );

    // ② WPT 真形态：template.content 解析产物 cloneNode(true) 的 checkbox——click 须
    // 翻 checked + 派发 input 事件触发其 inline oninput（activation 判定链在 clone
    // 子树完整闭合）。源未勾选 → 翻为 true。
    let out2 = sandbox
        .execute(
            "var parts = [];\
             var acts2 = [];\
             globalThis.__r154Act2 = function (e) { acts2.push(e); };\
             var tc = document.getElementById('tc');\
             var srcForm = document.createElement('form');\
             var srcSub = document.createElement('input'); srcSub.type = 'submit'; srcForm.appendChild(srcSub);\
             var srcCb = document.createElement('input'); srcCb.type = 'checkbox';\
             srcCb.setAttribute('oninput', 'this.checked ? __r154Act2(this) : null');\
             srcForm.appendChild(srcCb);\
             var form2 = srcForm.cloneNode(true);\
             tc.appendChild(form2);\
             var cb2 = form2.childNodes[form2.childNodes.length - 1];\
             parts.push('pre:' + cb2.checked);\
             cb2.click();\
             parts.push('post:' + cb2.checked);\
             parts.push('acts2:' + acts2.length);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out2,
        "pre:false|post:true|acts2:1",
        "R154：clone checkbox click——翻 checked + input 事件触发 inline oninput（WPT 真形态闭合）"
    );
}

// R155（js-dom M4）：single-activation 剩余 25F 的 clone 子树链——WPT 真形态是
// `template.content.children` 的 `cloneNode(true)` 产物。沙箱 createElement 单测全过
// → 差异定向：template 解析 + content.children + cloneNode 的 click 链。
#[test]
fn test_template_clone_checkbox_activation_r155() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![
        DomMutation::SetInnerHtml {
            selector: "body".into(),
            html: "<template id=\"tpl\"><form onsubmit=\"0\"><input class=\"click\" type=\"submit\"></form><input class=\"click\" type=\"checkbox\" oninput=\"0\"></template><div id=\"tc\"></div>".into(),
        },
    ]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><template id=\"tpl\"><form onsubmit=\"0\"><input class=\"click\" type=\"submit\"></form><input class=\"click\" type=\"checkbox\" oninput=\"0\"></template><div id=\"tc\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             var acts = [];\
             globalThis.__r155Act = function (e) { acts.push(e); };\
             var tpl = document.getElementById('tpl');\
             parts.push('tpl:' + (tpl ? 'yes' : 'null'));\
             var elems = tpl && tpl.content ? Array.from(tpl.content.children) : [];\
             parts.push('elems:' + elems.length);\
             if (elems.length >= 2) {\
               var form = elems[0].cloneNode(true);\
               var cb = elems[1].cloneNode(true);\
               cb.setAttribute('oninput', 'this.checked ? __r155Act(this) : null');\
               var tc = document.getElementById('tc');\
               tc.appendChild(form);\
               form.appendChild(cb);\
               parts.push('cb-pre:' + cb.checked);\
               try { cb.click(); } catch (e155c) { parts.push('threw:' + e155c.message); }\
               parts.push('cb-post:' + cb.checked);\
               parts.push('acts:' + acts.length);\
             }\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "tpl:yes|elems:2|cb-pre:false|cb-post:true|acts:1",
        "R155：template.content clone 的 checkbox 在 form 内 click——翻 checked + inline oninput activated"
    );
}

// R155（js-dom M4）：LABEL activation 转发诊断——span（LABEL 内）click 后 LABEL 的
// nearest 激活定位 + 转发链是否执行（WPT single-activation LABEL 簇 11F 定向）。
#[test]
fn test_label_span_click_forward_r155() {
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
        "<html><body><div id=\"tc\"><label id=\"lb\"><input id=\"ic\" type=\"checkbox\"><span id=\"sp\">label</span></label></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let out = sandbox
        .execute(
            "var parts = [];\
             var acts = [];\
             globalThis.__r155bAct = function (e) { acts.push(e); };\
             var ic = document.getElementById('ic');\
             ic.setAttribute('onclick', 'this.checked ? __r155bAct(this) : null');\
             var sp = document.getElementById('sp');\
             sp.click();\
             parts.push('checked:' + ic.checked);\
             parts.push('acts:' + acts.length);\
             parts.join('|');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "checked:true|acts:1",
        "R155：LABEL 内 span click——LABEL 转发激活到内部 checkbox（onclick activated 链）"
    );
}

/// R202：`_zwMText`/`_zwMComment`（foreign/detached doc 的 createTextNode/createComment、
/// `new Text()`/`new Comment()`、innerHTML 解析树子）的 CharacterData 方法面——
/// appendData/insertData/deleteData/replaceData/substringData + data/nodeValue setter
///（本地变更语义，与 R48 主文档路径的 no-parentSel 快照分支同款）。WPT Range
/// mega-case 的 foreign-doc 簇（`foreignTextNode.insertData` is not a function）。
#[test]
fn test_foreign_doc_text_characterdata_methods_r202() {
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
        Arc::new(Mutex::new("<html><body><div id='d'>x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            "var fdoc = document.implementation.createHTMLDocument('F');\
             var t = fdoc.createTextNode('xyz');\
             t.insertData(0, 'foo');\
             var a = t.data;\
             t.appendData('!');\
             var b = t.data + ':' + t.length;\
             t.deleteData(0, 3);\
             var c = t.data;\
             t.replaceData(0, 1, 'X');\
             var d = t.data + ':' + t.substringData(0, 2);\
             t.data = 'set';\
             var e = t.data + ':' + t.nodeValue + ':' + t.textContent;\
             var cm = fdoc.createComment('c');\
             cm.appendData('z');\
             var f = cm.data;\
             [a, b, c, d, e, f].join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "fooxyz|fooxyz!:7|xyz!|Xyz!:Xy|set:set:set|cz",
        "R202 foreign-doc 文本/注释节点 CharacterData 五方法 + data/nodeValue setter（本地变更语义）"
    );
}

/// R203：Range `setStart`/`setEnd` 的 **crossing 重设规则**（spec `range-set-start`
/// 步骤 3 / `range-set-end` 镜像）——新 start 在当前 end 之后（边界点比较，含跨文档）
/// 时 end 一并设为 (node, offset)；setEnd 向前穿 start 时 start 一并重设。WPT
/// Range-set 的 "must set the end node to node too" 族。
#[test]
fn test_range_set_start_end_crossing_r203() {
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
        "<html><body><div id='r'><p id='p0'>a</p><p id='p1'>b</p><p id='p2'>c</p></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            "var p0 = document.querySelector('#p0');\
             var p2 = document.querySelector('#p2');\
             var p2t = p2.firstChild;\
             var r = document.createRange();\
             r.setStart(p0, 0); r.setEnd(p0, 1);\
             var a = 'before:' + [r.startContainer === p0, r.endContainer === p0].join(',');\
             r.setStart(p2t, 0);\
             var b = 'cross:' + [r.startContainer === p2t, r.endContainer === p2t, r.endOffset].join(',');\
             var r2 = document.createRange();\
             r2.setStart(p2t, 0); r2.setEnd(p2, 1);\
             r2.setEnd(p0, 0);\
             var c = 'back:' + [r2.endContainer === p0, r2.startContainer === p0, r2.startOffset].join(',');\
             var r3 = document.createRange();\
             r3.setStart(p0, 0); r3.setEnd(p2, 1);\
             r3.setStart(p0.firstChild, 0);\
             var d = 'noncross:' + [r3.startContainer === p0.firstChild, r3.endContainer === p2].join(',');\
             [a, b, c, d].join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "before:true,true|cross:true,true,0|back:true,true,0|noncross:true,true",
        "R203 setStart 向后穿重设 end / setEnd 向前穿重设 start / 不穿越不动对侧"
    );
}

/// R204：compareBoundaryPoints 三件——① `how` 的 WebIDL unsigned short 转换前置
///（ToNumber→NaN/±0/±∞→+0；mod 2^16 负回绕；非 0-3 → NotSupportedError）；② Range
/// how 常量（START_TO_START..END_TO_START——缺常量使 WPT 合法性判定全失配）；③ 跨
/// 容器方向位（cDP 以接收者为参照：& 4 FOLLOWING → -1 / & 2 PRECEDING → +1）+
/// detached doc 的 createRange own 方法（旧落 Document.prototype 转发器自递归）。
#[test]
fn test_range_cbp_how_constants_direction_r204() {
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
        "<html><body><p id='a'>a</p><p id='b'>b</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            "var p0 = document.querySelector('#a'), p1 = document.querySelector('#b');\
             var consts = [globalThis.Range.START_TO_START, globalThis.Range.START_TO_END,\
                globalThis.Range.END_TO_END, globalThis.Range.END_TO_START].join(',');\
             var r1 = document.createRange(); r1.setStart(p0, 0); r1.setEnd(p0, 1);\
             var r2 = document.createRange(); r2.setStart(p1, 0); r2.setEnd(p1, 1);\
             var dir = r1.compareBoundaryPoints(0, r2);\
             function throws(v) { try { r1.compareBoundaryPoints(v, r2); return 'no'; } catch (e) { return e.name; } }\
             var fdoc = document.implementation.createHTMLDocument('x');\
             var fr = fdoc.createRange();\
             [consts, String(dir), throws(-1), throws(4), throws(65535), throws(NaN),\
              throws(0.5), throws('4'), throws(65536), throws(-65536), throws(null),\
              throws(undefined), throws('quasit'), String(fr.startContainer === fdoc)].join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "0,1,2,3|-1|NotSupportedError|NotSupportedError|NotSupportedError|no|no|NotSupportedError|no|no|no|no|no|true",
        "R204 how 常量 + 跨容器方向（r1 在 r2 前 → -1）+ WebIDL 转换形态 + detached createRange"
    );
}

/// R205：Range 点查询三方法的**边界点比较重写**——isPointInRange 跨容器树序
///（point 在 start 前/end 后返 false）、intersectsNode 的 (parent,i)/(parent,i+1)
/// 区间交（+ 移除 collapsed 前置 false——collapsed range 仍与边界点节点相交）、
/// comparePoint 的 -1/0/1 + doctype InvalidNodeTypeError（root 检查先行的步骤序）。
#[test]
fn test_range_point_queries_bptree_r205() {
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
        "<html><body><div id='r'><p id='p0'>abc</p><p id='p1'>def</p></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            "var p0 = document.querySelector('#p0').firstChild;\
             var p1 = document.querySelector('#p1').firstChild;\
             var dt = document.doctype;\
             var r = document.createRange();\
             r.setStart(p0, 1); r.setEnd(p1, 2);\
             var a = [\
               r.isPointInRange(p0, 0),\
               r.isPointInRange(p0, 1),\
               r.isPointInRange(p1, 3),\
               r.isPointInRange(p1, 2)\
             ].join(',');\
             var b = [\
               r.comparePoint(p0, 0),\
               r.comparePoint(p0, 1),\
               r.comparePoint(p1, 2),\
               r.comparePoint(p1, 3)\
             ].join(',');\
             function thr(fn) { try { fn(); return 'no'; } catch (e) { return e.name; } }\
             var c = [\
               thr(function () { r.isPointInRange(dt, 0); }),\
               thr(function () { r.comparePoint(dt, 0); })\
             ].join(',');\
             var r2 = document.createRange();\
             r2.setStart(p0, 1); r2.setEnd(p0, 1);\
             var d = [r2.collapsed, r2.intersectsNode(p0.parentNode)].join(',');\
             [a, b, c, d].join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "false,true,false,true|-1,0,0,1|InvalidNodeTypeError,InvalidNodeTypeError|true,true",
        "R205 点查询树序：before-start false / after-end false / 同点 true + comparePoint 三态 + doctype 抛错 + collapsed 相交"
    );
}

/// R207：iframe 工厂元素的可变面——① append/prepend/replaceChildren（R206 子文档
/// 脚本通道执行 common.js 的 `paras[5].append('9012')`——旧 only-appendChild）；
/// ② textContent setter 的 replace-all 语义（建 Text 子——旧 plain 字段使
/// firstChild 恒 null → eval 'paras[0].firstChild' 崩 ownerDocument(null)）；
/// ③ `_zwMakeIframeDoc` 根元素优先 `<html>`（通用配对 regex 首命中 `<title>` 使
/// documentElement.tagName=TITLE）+ docEl 可变面（appendChild/removeChild/append/
/// firstChild/lastChild——`refDoc.documentElement.cloneNode(true)` 链）。
#[test]
fn test_iframe_factory_mutation_face_r207() {
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
        Arc::new(Mutex::new("<html><body><div id='d'>x</div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            "var ifr = document.createElement('iframe');\
             document.body.appendChild(ifr);\
             var doc = ifr.contentDocument;\
             var p = doc.createElement('p');\
             p.textContent = 'hello';\
             var a = [typeof p.append, typeof p.prepend, typeof p.replaceChildren,\
               p.firstChild ? p.firstChild.nodeType : 'null-fc',\
               p.textContent,\
               p.firstChild && p.firstChild.parentNode === p].join(',');\
             p.append('!');\
             p.prepend('>');\
             var b = [p.childNodes.length, p.textContent].join(',');\
             p.replaceChildren('solo');\
             var c = [p.childNodes.length, p.textContent].join(',');\
             [a, b, c].join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "function,function,function,3,hello,true|3,>hello!|1,solo",
        "R207 iframe 工厂元素 append 族 + textContent replace-all + firstChild/parentNode"
    );
}

/// R208：iframe doc 查询缓存的同键失效——restoreIframe 每轮「querySelector('#test')
/// → removeChild → createElement+insertBefore 重建同构 `div#test`」，两节点
/// tag\x1fid\x1fouterHTML 键全等：移除后 `_zwQWrapCache` 仍持旧节点，后续查询
/// cache-hit 返**已移除节点**（parentNode=null → `td.parentNode.removeChild` 崩
/// "Cannot read properties of null"——Range-surroundContents/insertNode r1+ 轮
/// 919F 簇根因）。修复：body.removeChild / doc.removeChild 移除元素子时清
/// `_zwQWrapCache` + 失效 `_tree._zwNodeIdx`；`_zwMEl` 子树移除经 owner 溯源
/// 槽触发 doc 的 `_zwQWrapBump`。spec 依据 dom-node-remove：移除即脱离文档，
/// 后续查询不得命中。
/// https://dom.spec.whatwg.org/#concept-node-remove
#[test]
fn test_iframe_doc_query_cache_invalidation_r208() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            "var ifr = document.createElement('iframe');\
             document.body.appendChild(ifr);\
             ifr.setAttribute('src', 'Range-test-iframe.html');\
             var doc = ifr.contentDocument;\
             var log = [];\
             for (var round = 0; round < 3; round++) {\
               var td = doc.querySelector('#test');\
               log.push('r' + round + ':q=' + (td ? 'hit:pn=' + (td.parentNode === doc.body ? 'body' : (td.parentNode ? 'OTHER' : 'null')) : 'null'));\
               if (td && td.parentNode) { td.parentNode.removeChild(td); }\
               var nd = doc.createElement('div');\
               nd.id = 'test';\
               doc.body.insertBefore(nd, doc.body.firstChild);\
             }\
             log.join(' | ')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "r0:q=null | r1:q=hit:pn=body | r2:q=hit:pn=body",
        "R208 同键移除后查询须返 live 新节点（非已移除旧节点）"
    );
}

/// R210（js-dom M4）：surroundContents 的 spec 步骤 1/2 校验序——newParent 是
/// Document/DocumentType 抛 InvalidNodeTypeError（步骤 1，先于一切）；range 部分
/// 包含非 Text 节点抛 InvalidStateError（步骤 2——「是 start 或 end 边界容器的
/// 祖先但非双方共同祖先」的 cac 子树 DFS）。WPT 20,x 族 115F：cac=DIV 正确但
/// host 不抛 → assert_throws_dom "did not throw"。
/// https://dom.spec.whatwg.org/#dom-range-surroundcontents
#[test]
fn test_surround_invalid_state_and_step_order_r210() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    let common_js =
        // WPT dom/common.js（rev 3159769… 与 fetch-dom-subset.sh 同步，vendor 于
        // crates/engine/tests/fixtures/wpt-dom/——wpt-data/ 在 CI 会被重建，不可编译期引用）
        include_str!("../../tests/fixtures/wpt-dom/common.js").to_string();
    let iframe_html = include_str!(
        "../../tests/fixtures/wpt-dom/ranges/Range-test-iframe.html"
    )
    .to_string();
    let fetched_common = common_js.clone();
    let fetched_iframe = iframe_html.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("Range-test-iframe.html") {
                return format!("__zwfr:200\u{1f}OK\u{1f}\u{1f}{}", fetched_iframe);
            }
            "__zw_fetch_error:not-found".to_string()
        }),
    );
    sandbox.register_callback(
        "__zw_fetch_script",
        Box::new(move |args| {
            let src = args.get(1).map(String::as_str).unwrap_or("");
            if src.ends_with("common.js") {
                return fetched_common.clone();
            }
            String::new()
        }),
    );
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var ifr = document.createElement('iframe');
            document.body.appendChild(ifr);
            ifr.setAttribute('src', 'Range-test-iframe.html');
            var win = ifr.contentWindow;
            win.setupRangeTests();
            var out = [];
            // ① 跨容器 range（paras[0].firstChild → paras[1].firstChild，cac=DIV）
            // + 元素 newParent → InvalidStateError（testDiv 部分包含）
            win.testNodeInput = 'paras[0]';
            win.testRangeInput = '[paras[0].firstChild, 0, paras[1].firstChild, 0]';
            win.run();
            var r1 = win.testRange, n1 = win.testNode;
            var t1 = '';
            try { r1.surroundContents(n1); } catch (e) { t1 = e.name; }
            if (t1 !== 'InvalidStateError') out.push('cross-container:' + t1);
            // ② Document newParent → InvalidNodeTypeError（步骤 1 先于步骤 2）
            win.testNodeInput = 'document';
            win.testRangeInput = '[paras[0].firstChild, 0, paras[1].firstChild, 0]';
            win.run();
            var r2 = win.testRange, n2 = win.testNode;
            var t2 = '';
            try { r2.surroundContents(n2); } catch (e) { t2 = e.name; }
            if (t2 !== 'InvalidNodeTypeError') out.push('doc-newparent:' + t2);
            // ③ 单容器整选区（无部分包含）不抛 InvalidStateError（合法路径——
            // host 对该形态的既有行为不回归：collapsed/整元素选区不因新校验误伤）
            win.testNodeInput = 'paras[0]';
            win.testRangeInput = '[testDiv, 0, testDiv, 1]';
            win.run();
            var r3 = win.testRange, n3 = win.testNode;
            var t3 = 'no-throw';
            try { r3.surroundContents(n3); } catch (e) { t3 = e.name; }
            if (t3 === 'InvalidStateError') out.push('full-selection:false-positive');
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(out, "ALL-OK", "R210 surroundContents 步骤 1/2 校验序");
}
/// R211（js-dom M4）：extractContents 的 CharacterData 区间分支（spec
/// `dom-range-extract-contents`——start/end 容器为 Text/CDATA 同父时：
/// frag = [start 尾切片克隆, contained 子本体, end 头切片克隆]，原树
/// deleteData 掉切片 + contained 子移动；collapsed 同节点取中段）。
/// WPT Range-extractContents +24F 收口的 driving 断言。
/// https://dom.spec.whatwg.org/#dom-range-extractcontents
#[test]
fn test_extract_contents_chardata_interval_r211() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var out = [];
            // ① 主文档文本同节点中段（collapsed 检查外的基本区间）
            var p = document.createElement('p');
            p.textContent = 'abcdef';
            document.body.appendChild(p);
            var t = p.firstChild;
            var r = document.createRange();
            r.setStart(t, 1);
            r.setEnd(t, 4);
            var f = r.extractContents();
            var parts = [];
            for (var i = 0; i < f.childNodes.length; i++) parts.push(String(f.childNodes[i].data));
            out.push('mid:' + parts.join('|') + ':t-after=' + String(t.data));
            // ② 跨节点：p 内 t1('ab') t2('cd') t3('ef')，range t1[1]..t3[1]
            var p2 = document.createElement('p');
            p2.appendChild(document.createTextNode('ab'));
            p2.appendChild(document.createTextNode('cd'));
            p2.appendChild(document.createTextNode('ef'));
            document.body.appendChild(p2);
            var t1 = p2.childNodes[0], t3 = p2.childNodes[2];
            var r2 = document.createRange();
            r2.setStart(t1, 1);
            r2.setEnd(t3, 1);
            var f2 = r2.extractContents();
            var parts2 = [];
            for (var j = 0; j < f2.childNodes.length; j++) parts2.push(f2.childNodes[j].nodeType + ':' + String(f2.childNodes[j].data));
            out.push('cross:' + parts2.join('|') + ':p2kids=' + p2.childNodes.length
              + ':t1=' + String(t1.data) + ':t3=' + String(t3.data));
            // ③ extract 后 range 收缩到 (parent, si)
            out.push('r2-sc=' + (r2.startContainer === p2) + ':so=' + r2.startOffset
              + ':eo=' + r2.endOffset);
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "mid:bcd:t-after=aef\ncross:3:b|3:cd|3:e:p2kids=2:t1=a:t3=f\nr2-sc=true:so=1:eo=1",
        "R211 extractContents CharData 区间语义"
    );
}
/// R212（js-dom M4）：surroundContents 的 CharData 区间路径三件（spec
/// `dom-range-surroundcontents` 完整链）：
/// ① 步骤 1 补 nodeType 11（DocumentFragment newParent → InvalidNodeTypeError，
///   旧版漏检使 docfrag 走到 CharData 路径实际变更树——,20 族 48F）
/// ② 步骤 3「While newParent has children, remove its first child」（旧版漏，
///   wrapped 元素残留 setup 期原文本）
/// ③ 工厂元素 appendChild 的 DocumentFragment 展平（spec
///   `dom-node-append-child`——frag 子逐个 append 后清空；旧版塞 fragment 本体，
///   frag 子丢失）
/// + CDATA cloneNode nt=4 分支（成对 land——两侧对称闭合）。
///
/// <https://dom.spec.whatwg.org/#dom-range-surroundcontents>
#[test]
fn test_surround_chardata_path_r212() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    let common_js =
        // WPT dom/common.js（rev 3159769… 与 fetch-dom-subset.sh 同步，vendor 于
        // crates/engine/tests/fixtures/wpt-dom/——wpt-data/ 在 CI 会被重建，不可编译期引用）
        include_str!("../../tests/fixtures/wpt-dom/common.js").to_string();
    let iframe_html = include_str!(
        "../../tests/fixtures/wpt-dom/ranges/Range-test-iframe.html"
    )
    .to_string();
    let fetched_common = common_js.clone();
    let fetched_iframe = iframe_html.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("Range-test-iframe.html") {
                return format!("__zwfr:200\u{1f}OK\u{1f}\u{1f}{}", fetched_iframe);
            }
            "__zw_fetch_error:not-found".to_string()
        }),
    );
    sandbox.register_callback(
        "__zw_fetch_script",
        Box::new(move |args| {
            let src = args.get(1).map(String::as_str).unwrap_or("");
            if src.ends_with("common.js") {
                return fetched_common.clone();
            }
            String::new()
        }),
    );
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var ifr = document.createElement('iframe');
            document.body.appendChild(ifr);
            ifr.setAttribute('src', 'Range-test-iframe.html');
            var win = ifr.contentWindow;
            win.setupRangeTests();
            var out = [];
            // ① DocumentFragment newParent → InvalidNodeTypeError（步骤 1 三类型）
            win.testNodeInput = 'docfrag';
            win.testRangeInput = '[paras[0].firstChild, 0, paras[0].firstChild, 0]';
            win.run();
            var r1 = win.testRange, n1 = win.testNode;
            var t1 = '';
            try { r1.surroundContents(n1); } catch (e) { t1 = e.name; }
            if (t1 !== 'InvalidNodeTypeError') out.push('docfrag:' + t1);
            // ③ CDATA cloneNode(false) 保 nodeType=4（须在 surround 变更 paras[5] 前检查）
            win.testNodeInput = 'paras[5].firstChild';
            win.testRangeInput = '[paras[0].firstChild, 0, paras[0].firstChild, 0]';
            win.run();
            var cd = win.testNode;
            var cl = cd.cloneNode(false);
            if (!(cl && cl.nodeType === 4 && String(cl.data) === String(cd.data))) out.push('cdata-clone:bad');
            // ② CDATA 区间 surround：树形态（wrapped 元素 = frag 内容、父 =
            // [wrapped, 剩余 cdata 头, 空 text]）+ newParent 原子清除 + range select
            win.testNodeInput = 'paras[0]';
            win.testRangeInput = '[paras[5].firstChild, 2, paras[5].lastChild, 4]';
            win.run();
            var r2 = win.testRange, n2 = win.testNode;
            var p5 = r2.startContainer.parentNode;
            try {
              r2.surroundContents(n2);
              var s = '';
              for (var i = 0; i < p5.childNodes.length; i++) {
                var c = p5.childNodes[i];
                s += c.nodeType === 1 ? 'E[' : ('t' + c.nodeType + '(' + String(c.data) + ')');
                if (c.nodeType === 1) {
                  var w = '';
                  for (var j = 0; j < c.childNodes.length; j++)
                    w += 'nt' + c.childNodes[j].nodeType + '(' + String(c.childNodes[j].data) + ')';
                  s += w + ']';
                }
              }
              out.push('tree:' + s);
              out.push('range:' + (r2.startContainer === p5) + ':' + r2.startOffset + ':' + r2.endOffset);
            } catch (e) { out.push('threw:' + e.name); }
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "tree:t4(12)E[nt4(34)nt4(5678)nt3(9012)]t3()\nrange:true:1:2",
        "R212 surroundContents CharData 路径 + CDATA cloneNode"
    );
}
/// R213（js-dom M4）：extractContents 收缩偏移修正（spec「Set new offset to
/// one plus the index of reference node」——旧版 setStart(si) 使后续 insertNode
/// 落在削弱的 start 容器**前**，sim 落后一位：6,x positionTests 的 offsets
/// A=0,1 E=1,2 根因）+ deleteContents 的 CharData 区间删侧分支（三段：
/// start 尾段 deleteData + contained 子移除 + end 头段 deleteData；同节点中段；
/// collapse 到 (parent, si+1)）。忠实复刻 testharness 双 iframe 流程验证
/// （common.js 真源 + mySurroundContents 真函数注入）。
/// <https://dom.spec.whatwg.org/#dom-range-deletecontents>
#[test]
fn test_extract_collapse_offset_and_delete_chardata_r213() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var out = [];
            // ① extract 收缩偏移：跨节点 extract 后 startOffset = si+1（sim 对齐）
            var p = document.createElement('p');
            p.appendChild(document.createTextNode('ab'));
            p.appendChild(document.createTextNode('cd'));
            p.appendChild(document.createTextNode('ef'));
            document.body.appendChild(p);
            var t1 = p.childNodes[0], t3 = p.childNodes[2];
            var r = document.createRange();
            r.setStart(t1, 1);
            r.setEnd(t3, 1);
            r.extractContents();
            out.push('collapse:' + (r.startContainer === p) + ':' + r.startOffset + ':' + r.endOffset);
            // ② deleteContents CharData 跨节点三段：t1 保 'a'，cd 全删，t3 保 'f'
            var p2 = document.createElement('p');
            p2.appendChild(document.createTextNode('ab'));
            p2.appendChild(document.createTextNode('cd'));
            p2.appendChild(document.createTextNode('ef'));
            document.body.appendChild(p2);
            var u1 = p2.childNodes[0], u3 = p2.childNodes[2];
            var r2 = document.createRange();
            r2.setStart(u1, 1);
            r2.setEnd(u3, 1);
            r2.deleteContents();
            out.push('del-cross:' + String(u1.data) + '|' + String(u3.data)
              + '|kids=' + p2.childNodes.length + '|off=' + r2.startOffset);
            // ③ deleteContents 同节点中段
            var p3 = document.createElement('p');
            p3.textContent = 'abcdef';
            document.body.appendChild(p3);
            var r3 = document.createRange();
            r3.setStart(p3.firstChild, 1);
            r3.setEnd(p3.firstChild, 4);
            r3.deleteContents();
            out.push('del-mid:' + String(p3.firstChild.data) + '|collapsed=' + r3.collapsed);
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "collapse:true:1:1\ndel-cross:a|f|kids=2|off=1\ndel-mid:aef|collapsed=true",
        "R213 extract 收缩偏移 + deleteContents CharData 删侧"
    );
}
/// R214（js-dom M4）：iframe doc 的 documentElement 结构修复两件——
/// ① **ownerDocument 指源 doc**（两形态：mEl 解析产物 + R177 合成 html）——
/// common.js rangeFromEndpoints 经 `ownerDocument(docEl).createRange()` 建域内
/// Range（WPT Range-insertNode 12,x 138F：旧 undefined → 'reading createRange'）；
/// ② **无显式 `<html>` 标签时不再落通用配对 regex**——首个「开-闭对」是
/// `<title>…</title>` 使 docEl=TITLE（refDoc 克隆链跟着 TITLE 化）。真浏览器
/// 对无显式 html 的 HTML 文档合成 `<html>` 根；XML kind 保持通用 regex。
#[test]
fn test_iframe_docelement_structure_r214() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    let common_js =
        // WPT dom/common.js（rev 3159769… 与 fetch-dom-subset.sh 同步，vendor 于
        // crates/engine/tests/fixtures/wpt-dom/——wpt-data/ 在 CI 会被重建，不可编译期引用）
        include_str!("../../tests/fixtures/wpt-dom/common.js").to_string();
    let iframe_html = include_str!(
        "../../tests/fixtures/wpt-dom/ranges/Range-test-iframe.html"
    )
    .to_string();
    let fetched_common = common_js.clone();
    let fetched_iframe = iframe_html.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("Range-test-iframe.html") {
                return format!("__zwfr:200\u{1f}OK\u{1f}\u{1f}{}", fetched_iframe);
            }
            "__zw_fetch_error:not-found".to_string()
        }),
    );
    sandbox.register_callback(
        "__zw_fetch_script",
        Box::new(move |args| {
            let src = args.get(1).map(String::as_str).unwrap_or("");
            if src.ends_with("common.js") {
                return fetched_common.clone();
            }
            String::new()
        }),
    );
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var ifr = document.createElement('iframe');
            document.body.appendChild(ifr);
            ifr.setAttribute('src', 'Range-test-iframe.html');
            var doc = ifr.contentDocument;
            var win = ifr.contentWindow;
            var out = [];
            // ① docEl 结构：无显式 <html> 的 Range-test-iframe.html → 合成 HTML 根（非 TITLE）
            var de = doc.documentElement;
            if (!de || de.tagName !== 'HTML' || de.nodeType !== 1) out.push('docelem:' + (de ? de.tagName : 'null'));
            // ② docEl.ownerDocument === doc（rangeFromEndpoints 消费）
            if (de.ownerDocument !== doc) out.push('owner-doc:' + (de.ownerDocument === document ? 'main' : 'other'));
            // ③ clone 链：referenceDoc 重建（restoreIframe 主消费）
            var refDoc = document.implementation.createHTMLDocument('');
            refDoc.removeChild(refDoc.documentElement);
            var clone = de.cloneNode(true);
            refDoc.appendChild(clone);
            if (!refDoc.documentElement || refDoc.documentElement.tagName !== 'HTML') {
              out.push('clone-chain:' + (refDoc.documentElement ? refDoc.documentElement.tagName : 'null'));
            }
            // ④ docEl-rooted range 建立可用（12,x setup 路径）
            win.setupRangeTests();
            win.testNodeInput = 'paras[0]';
            win.testRangeInput = '[document.documentElement, 0, document.documentElement, 1]';
            win.run();
            if (win.unexpectedException) {
              out.push('range-setup:' + String(win.unexpectedException.message).slice(0, 60));
              win.unexpectedException = null;
            } else if (!win.testRange) {
              out.push('range-setup:null');
            }
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(out, "ALL-OK", "R214 iframe documentElement 结构");
}
/// R215（js-dom M4）：insertNode 的 **ensure-pre-insertion validity 前置**
/// （spec `dom-node-pre-insert` 校验族——336F HRE 簇 + 8,9/9,9 的 P→DIV→P
/// **parentNode 环**根因：旧版 splitText 路径无校验直接 insertBefore，把插入
/// 目标自身的祖先插进目标成环（upwalk 探针 101 hops 实证），后续 sim 的
/// isInclusiveAncestor 上行 walk 栈溢出）。校验四件：① parent 非
/// Element/Document/DocumentFragment → HRE；② node 是 parent 的 inclusive
/// ancestor → HRE（环检测，guard 128）；③ Text 入 Document → HRE；
/// ④ Doctype 入非 Document → HRE。surroundContents 叶子 newParent 路径经
/// `_r215NoValidate` 帧标志抑制校验（保持 R212 的先变更后抛序——sim 的
/// myExtractContents 在步骤 5 抛之前已变更树）。
/// <https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity>
#[test]
fn test_insert_node_pre_insertion_validity_r215() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var out = [];
            // ① 祖先插入自身后代 → HRE 且不建环（8,9 形态）
            var div = document.createElement('div');
            var p = document.createElement('p');
            var t = document.createTextNode('xy');
            p.appendChild(t);
            div.appendChild(p);
            var r1 = document.createRange();
            r1.setStart(t, 0);
            r1.setEnd(t, 1);
            var threw1 = '';
            try { r1.insertNode(div); } catch (e) { threw1 = e.name; }
            out.push('ancestor:' + threw1 + ':p.parent-is-div=' + (p.parentNode === div));
            // 环检测（upwalk 有界）
            var hops = 0, cur = t;
            while (cur && hops++ < 50) cur = cur.parentNode;
            out.push('upwalk:' + (hops < 50 ? 'ok' : 'CYCLE'));
            // ② Text 入 Document → HRE
            var r2 = document.createRange();
            r2.setStart(document, 0);
            r2.setEnd(document, 0);
            var threw2 = '';
            try { r2.insertNode(document.createTextNode('x')); } catch (e) { threw2 = e.name; }
            out.push('text-in-doc:' + threw2);
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "ancestor:HierarchyRequestError:p.parent-is-div=true\nupwalk:ok\ntext-in-doc:HierarchyRequestError",
        "R215 insertNode pre-insertion validity"
    );
}
/// R216（js-dom M4）：iframe doc 的 docEl **入 doc 树**（spec：documentElement
/// 是 Document 的子——WPT Range-insertNode 25,x：`[document, 0, document, 1]`
/// 的 setEnd 按 doc.childNodes 长度校验，doc 无子使 setup 抛 IndexSizeError
/// 整簇；12,x 的 upwalk 链 HTML→#document）。doctype 保持 getter-only
/// （入 childNodes 首位实测 -55——restoreIframe 清理节奏扰动面更广，R216 评估注）。
/// <https://dom.spec.whatwg.org/#document>
#[test]
fn test_iframe_docelement_in_doctree_r216() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    let common_js =
        // WPT dom/common.js（rev 3159769… 与 fetch-dom-subset.sh 同步，vendor 于
        // crates/engine/tests/fixtures/wpt-dom/——wpt-data/ 在 CI 会被重建，不可编译期引用）
        include_str!("../../tests/fixtures/wpt-dom/common.js").to_string();
    let iframe_html = include_str!(
        "../../tests/fixtures/wpt-dom/ranges/Range-test-iframe.html"
    )
    .to_string();
    let fetched_common = common_js.clone();
    let fetched_iframe = iframe_html.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("Range-test-iframe.html") {
                return format!("__zwfr:200\u{1f}OK\u{1f}\u{1f}{}", fetched_iframe);
            }
            "__zw_fetch_error:not-found".to_string()
        }),
    );
    sandbox.register_callback(
        "__zw_fetch_script",
        Box::new(move |args| {
            let src = args.get(1).map(String::as_str).unwrap_or("");
            if src.ends_with("common.js") {
                return fetched_common.clone();
            }
            String::new()
        }),
    );
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var ifr = document.createElement('iframe');
            document.body.appendChild(ifr);
            ifr.setAttribute('src', 'Range-test-iframe.html');
            var doc = ifr.contentDocument;
            var win = ifr.contentWindow;
            win.setupRangeTests();
            var out = [];
            // ① docEl.parentNode === doc（upwalk 链 HTML→#document）
            if (doc.documentElement.parentNode !== doc) out.push('parent:' + (doc.documentElement.parentNode ? 'other' : 'null'));
            // ② docEl 在 doc.childNodes 内
            var has = false;
            for (var i = 0; i < doc.childNodes.length; i++) if (doc.childNodes[i] === doc.documentElement) has = true;
            if (!has) out.push('in-childNodes:false');
            // ③ doc-rooted range setup 不抛（25,x 前置）
            win.testRangeInput = '[document, 0, document, 1]';
            win.testNodeInput = 'paras[0]';
            win.run();
            if (win.unexpectedException) out.push('setup:' + String(win.unexpectedException.message).slice(0, 50));
            else if (!win.testRange) out.push('setup:null-range');
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(out, "ALL-OK", "R216 iframe docEl 入 doc 树");
}
/// R217（js-dom M4）：insertNode 的 **Document 子位置规则**（spec
/// `dom-node-pre-insert` 的「If parent is a Document」四分支）——WPT
/// Range-insertNode 25,x 族：element 入已有 element 子的 Document → HRE；
/// fragment 多 element 子 / Text 子 → HRE；frag 单 element 子 vs 既有 element
/// 子 / 插入点与 doctype 位序 → HRE；doctype 入已有 doctype / element 子后
/// → HRE。R215 校验四件之上补 Document 专属分支。
/// <https://dom.spec.whatwg.org/#concept-node-ensure-pre-insertion-validity>
#[test]
fn test_insert_node_document_position_rules_r217() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var out = [];
            var mkRange = function (node, a, b) {
              var r = document.createRange();
              r.setStart(node, a);
              r.setEnd(node, b);
              return r;
            };
            // ① element 入已有 element 子的 detached doc（25,x 主形态）
            var d1 = document.implementation.createHTMLDocument('');
            var r1 = mkRange(d1, 0, 1);
            var el1 = d1.createElement('p');
            var t1 = '';
            try { r1.insertNode(el1); } catch (e) { t1 = e.name; }
            out.push('el-vs-el:' + t1);
            // ② fragment 多 element 子 → HRE
            var d2 = document.implementation.createDocument(null, null, null);
            var f2 = d2.createDocumentFragment();
            f2.appendChild(d2.createElement('a'));
            f2.appendChild(d2.createElement('b'));
            var r2 = mkRange(d2, 0, 0);
            var t2 = '';
            try { r2.insertNode(f2); } catch (e) { t2 = e.name; }
            out.push('frag-multi:' + t2);
            // ③ fragment Text 子 → HRE
            var d3 = document.implementation.createDocument(null, null, null);
            var f3 = d3.createDocumentFragment();
            f3.appendChild(d3.createTextNode('x'));
            var r3 = mkRange(d3, 0, 0);
            var t3 = '';
            try { r3.insertNode(f3); } catch (e) { t3 = e.name; }
            out.push('frag-text:' + t3);
            // ④ doctype 入已有 doctype → HRE
            var d4 = document.implementation.createHTMLDocument('');
            var dt4 = d4.implementation.createDocumentType('x', '', '');
            var r4 = mkRange(d4, 0, 0);
            var t4 = '';
            try { r4.insertNode(dt4); } catch (e) { t4 = e.name; }
            out.push('dt-vs-dt:' + t4);
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "el-vs-el:HierarchyRequestError\nfrag-multi:HierarchyRequestError\nfrag-text:HierarchyRequestError\ndt-vs-dt:HierarchyRequestError",
        "R217 insertNode Document 子位置规则"
    );
}
/// R218（js-dom M4）：两件方法面补齐——
/// ① **CDATASection（nt=4）的 splitText**（spec CDATASection : Text——splitText
///   经继承可达；WPT Range-insertNode 6,x：sim 对 CDATA startContainer 调
///   range.startContainer.splitText——旧 'not a function' 34F）；
/// ② **detached doc fragment 的 insertBefore**（spec `dom-node-pre-insert`——
///   38,x：range 落在 docfrag 容器时 sim 的 parent_.insertBefore——旧
///   'not a function' 17F；ref=null 等价 append + fragment 展平）。
/// <https://dom.spec.whatwg.org/#concept-node-pre-insert>
#[test]
fn test_cdata_splittext_and_fragment_insertbefore_r218() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var out = [];
            // ① CDATA splitText：offset 拆分 + parent 内插入
            var xd = document.implementation.createDocument(null, null, null);
            var cd = xd.createCDATASection('1234');
            xd.appendChild(cd);
            var tail = cd.splitText(2);
            out.push('cdata-split:' + String(cd.data) + '|' + String(tail.data)
              + ':tail-is-cdata=' + (tail.nodeType === 4));
            // ② CDATA splitText 越界 IndexSizeError
            var cd2 = xd.createCDATASection('ab');
            var threw = '';
            try { cd2.splitText(5); } catch (e) { threw = e.name; }
            out.push('cdata-oob:' + threw);
            // ③ detached-doc fragment insertBefore：ref 前插 + ref=null 尾插 + 展平
            //（R218 的修复面在 detached doc fragment——主文档 fragment 为 handle proxy 另径）
            var dd = document.implementation.createDocument(null, null, null);
            var frag = dd.createDocumentFragment();
            var a = dd.createElement('a');
            var b = dd.createElement('b');
            frag.appendChild(a);
            var c = dd.createElement('c');
            frag.insertBefore(c, a);
            out.push('frag-order:' + (frag.childNodes[0] === c && frag.childNodes[1] === a));
            frag.insertBefore(b, null);
            out.push('frag-tail:' + (frag.childNodes[2] === b));
            var frag2 = dd.createDocumentFragment();
            var inner = dd.createDocumentFragment();
            inner.appendChild(dd.createElement('x'));
            frag2.insertBefore(inner, null);
            out.push('frag-flatten:' + (frag2.childNodes.length === 1 && frag2.childNodes[0] && String(frag2.childNodes[0].nodeName).toLowerCase() === 'x')
              + ':inner-emptied=' + (inner.childNodes.length === 0));
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        "cdata-split:12|34:tail-is-cdata=true\ncdata-oob:IndexSizeError\nfrag-order:true\nfrag-tail:true\nfrag-flatten:true:inner-emptied=true",
        "R218 CDATA splitText + fragment insertBefore"
    );
}
/// R209（js-dom M4）：iframe 子文档 testNodes 方法面 + surroundContents 叶子
/// newParent 的 spec 异常链。根因（探针实证）：iframe/detached 工厂节点形态缺
/// compareDocumentPosition/hasChildNodes/cloneNode/substringData/splitText/
/// insertBefore（common.js mega-case 的 getPosition/myInsertNode 直接调）；
/// host surroundContents 对 Text/Comment/PI newParent 不抛 HRE（步骤 5 的
/// appendChild(fragment) 到叶子必抛）。spec：
/// https://dom.spec.whatwg.org/#dom-range-surroundcontents
/// https://dom.spec.whatwg.org/#dom-range-insertnode
#[test]
fn test_iframe_testnodes_method_face_r209() {
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
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    let common_js =
        // WPT dom/common.js（rev 3159769… 与 fetch-dom-subset.sh 同步，vendor 于
        // crates/engine/tests/fixtures/wpt-dom/——wpt-data/ 在 CI 会被重建，不可编译期引用）
        include_str!("../../tests/fixtures/wpt-dom/common.js").to_string();
    let iframe_html = include_str!(
        "../../tests/fixtures/wpt-dom/ranges/Range-test-iframe.html"
    )
    .to_string();
    let fetched_common = common_js.clone();
    let fetched_iframe = iframe_html.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.ends_with("Range-test-iframe.html") {
                return format!("__zwfr:200\u{1f}OK\u{1f}\u{1f}{}", fetched_iframe);
            }
            "__zw_fetch_error:not-found".to_string()
        }),
    );
    sandbox.register_callback(
        "__zw_fetch_script",
        Box::new(move |args| {
            let src = args.get(1).map(String::as_str).unwrap_or("");
            if src.ends_with("common.js") {
                return fetched_common.clone();
            }
            String::new()
        }),
    );
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let out = sandbox
        .execute(
            r#"
            var ifr = document.createElement('iframe');
            document.body.appendChild(ifr);
            ifr.setAttribute('src', 'Range-test-iframe.html');
            var doc = ifr.contentDocument;
            var win = ifr.contentWindow;
            win.setupRangeTests();
            var out = [];
            // ① testNodes 方法面：iframe doc 域节点全方法可达
            var forms = [
              ['paras[0]', 1], ['paras[0].firstChild', 3], ['detachedPara1', 1],
              ['detachedPara1.firstChild', 3], ['detachedTextNode', 3],
              ['detachedDiv', 1], ['docfrag', 11], ['doctype', 10],
              ['foreignDoctype', 10], ['processingInstruction', 7],
              ['detachedProcessingInstruction', 7], ['comment', 8], ['detachedComment', 8],
            ];
            for (var i = 0; i < forms.length; i++) {
              win.testNodeInput = forms[i][0];
              win.testRangeInput = '[paras[0].firstChild, 0, paras[0].firstChild, 0]';
              win.run();
              if (win.unexpectedException) {
                out.push(forms[i][0] + ':SETUP-ERR'); win.unexpectedException = null; continue;
              }
              var v = win.testNode;
              if (!v || v.nodeType !== forms[i][1]) { out.push(forms[i][0] + ':BAD-FORM'); continue; }
              var need = v.nodeType === 1 || v.nodeType === 11
                ? ['compareDocumentPosition', 'hasChildNodes', 'cloneNode', 'isEqualNode', 'contains']
                : v.nodeType === 3 || v.nodeType === 4 || v.nodeType === 7 || v.nodeType === 8
                ? ['compareDocumentPosition', 'hasChildNodes', 'cloneNode', 'substringData']
                : ['compareDocumentPosition', 'hasChildNodes', 'cloneNode'];
              for (var m = 0; m < need.length; m++) {
                if (typeof v[need[m]] !== 'function') out.push(forms[i][0] + ':MISSING-' + need[m]);
              }
            }
            // ② doctype 可读（Range-test-iframe 的 <!doctype html>）
            if (!(doc.doctype && doc.doctype.name === 'html' && doc.doctype.nodeType === 10)) {
              out.push('doctype:UNREADABLE');
            }
            // ③ surroundContents 叶子 newParent 抛 HRE + 树中间态（split 截断）
            win.testNodeInput = 'detachedTextNode';
            win.testRangeInput = '[paras[0].firstChild, 0, paras[0].firstChild, 0]';
            win.run();
            var r3 = win.testRange, n3 = win.testNode;
            var sc3 = r3.startContainer;
            var threw = '';
            try { r3.surroundContents(n3); } catch (e) { threw = e.name; }
            if (threw !== 'HierarchyRequestError') out.push('surround-leaf:THREW-' + threw);
            if (String(sc3.data) !== '') out.push('surround-leaf:sc-not-split');
            // ④ insertNode 的 startContainer === node → HRE 树不变
            win.testNodeInput = 'paras[0].firstChild';
            win.testRangeInput = '[paras[0].firstChild, 0, paras[0].firstChild, 0]';
            win.run();
            var r4 = win.testRange, n4 = win.testNode;
            var before4 = String(n4.data);
            var threw4 = '';
            try { r4.insertNode(n4); } catch (e) { threw4 = e.name; }
            if (threw4 !== 'HierarchyRequestError') out.push('insert-self:THREW-' + threw4);
            if (String(n4.data) !== before4) out.push('insert-self:tree-mutated');
            // ⑤ DOMException legacy code 常量在实例上可枚举（getDomExceptionName 消费）
            var de = new (win.DOMException || DOMException)('m', 'HierarchyRequestError');
            var legacy = '';
            for (var pr in de) { if (/^[A-Z_]+_ERR$/.test(pr) && de[pr] === de.code) legacy = pr; }
            if (legacy !== 'HIERARCHY_REQUEST_ERR') out.push('de-legacy:' + legacy);
            out.length ? out.join('\n') : 'ALL-OK'
            "#,
        )
        .unwrap()
        .value;
    assert_eq!(out, "ALL-OK", "R209 iframe testNodes 方法面/surroundContents 异常链");
}
