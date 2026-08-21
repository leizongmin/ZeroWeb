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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

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
