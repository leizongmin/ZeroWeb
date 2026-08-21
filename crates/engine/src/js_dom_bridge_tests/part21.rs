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
