#[test]
fn test_event_subclasses2_r2812() {
    // R2812：Event 子类簇 #2——HashChangeEvent / PopStateEvent / StorageEvent / ProgressEvent /
    // TransitionEvent / AnimationEvent（均 extends Event）。SPA hash/history 路由 + 跨标签页 storage 同步 +
    // XHR/资源加载进度 + CSS 过渡/动画回调高频。复用 R2811 _defineEventSubclass 工厂 + createEvent map 扩展。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // feature-detection：6 构造器均为 function + instanceof Event。
    sandbox
        .execute(
            "globalThis.__fns = ['HashChangeEvent','PopStateEvent','StorageEvent','ProgressEvent',\
               'TransitionEvent','AnimationEvent']\
               .map(function(n){ return typeof globalThis[n] === 'function'; }).every(Boolean);\
             globalThis.__chain = new ProgressEvent('load') instanceof Event;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fns)").unwrap().value,
        "true",
        "6 构造器须均为 function"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__chain)").unwrap().value,
        "true",
        "子类 instanceof Event"
    );

    // HashChangeEvent: oldURL/newURL（SPA hash 路由）。
    sandbox
        .execute(
            "globalThis.__he = new HashChangeEvent('hashchange', { oldURL: '#/a', newURL: '#/b' });\
             globalThis.__heOld = __he.oldURL; globalThis.__heNew = __he.newURL;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__heOld)").unwrap().value,
        "#/a",
        "HashChangeEvent oldURL"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__heNew)").unwrap().value,
        "#/b",
        "HashChangeEvent newURL"
    );

    // PopStateEvent: state（history 路由）。
    sandbox
        .execute("globalThis.__ps = new PopStateEvent('popstate', { state: { page: 2 } }); globalThis.__psS = __ps.state.page;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__psS)").unwrap().value,
        "2",
        "PopStateEvent state"
    );

    // StorageEvent: key/newValue/oldValue/url/storageArea（跨标签页 storage 同步）。
    sandbox
        .execute(
            "globalThis.__se = new StorageEvent('storage', { key: 'k', newValue: 'v2', oldValue: 'v1', url: 'http://x' });\
             globalThis.__seKey = __se.key; globalThis.__seNew = __se.newValue; globalThis.__seOld = __se.oldValue;\
             globalThis.__seUrl = __se.url; globalThis.__seArea = __se.storageArea;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__seKey)").unwrap().value,
        "k",
        "StorageEvent key"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__seNew)").unwrap().value,
        "v2",
        "StorageEvent newValue"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__seOld)").unwrap().value,
        "v1",
        "StorageEvent oldValue"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__seUrl)").unwrap().value,
        "http://x",
        "StorageEvent url"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__seArea)").unwrap().value,
        "null",
        "StorageEvent storageArea 默认 null"
    );

    // ProgressEvent: lengthComputable/loaded/total + 默认（XHR/资源加载进度）。
    sandbox
        .execute(
            "globalThis.__pe = new ProgressEvent('progress', { lengthComputable: true, loaded: 50, total: 100 });\
             globalThis.__peLC = __pe.lengthComputable; globalThis.__peL = __pe.loaded; globalThis.__peT = __pe.total;\
             globalThis.__peDef = new ProgressEvent('load').lengthComputable;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__peLC)").unwrap().value,
        "true",
        "ProgressEvent lengthComputable"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__peL)").unwrap().value,
        "50",
        "ProgressEvent loaded"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__peT)").unwrap().value,
        "100",
        "ProgressEvent total"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__peDef)").unwrap().value,
        "false",
        "ProgressEvent lengthComputable 默认 false"
    );

    // TransitionEvent / AnimationEvent: propertyName/animationName + elapsedTime + pseudoElement。
    sandbox
        .execute(
            "globalThis.__te = new TransitionEvent('transitionend', { propertyName: 'opacity', elapsedTime: 0.5 });\
             globalThis.__teP = __te.propertyName; globalThis.__teE = __te.elapsedTime;\
             globalThis.__ae = new AnimationEvent('animationend', { animationName: 'fade', elapsedTime: 1.2 });\
             globalThis.__aeN = __ae.animationName; globalThis.__aeE = __ae.elapsedTime;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__teP)").unwrap().value,
        "opacity",
        "TransitionEvent propertyName"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__teE)").unwrap().value,
        "0.5",
        "TransitionEvent elapsedTime"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__aeN)").unwrap().value,
        "fade",
        "AnimationEvent animationName"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__aeE)").unwrap().value,
        "1.2",
        "AnimationEvent elapsedTime"
    );

    // createEvent 映射（map）含 6 新 type：createEvent('StorageEvent') instanceof StorageEvent。
    sandbox
        .execute(
            "globalThis.__cse = document.createEvent('StorageEvent') instanceof StorageEvent;\
             globalThis.__cpr = document.createEvent('ProgressEvent') instanceof ProgressEvent;\
             globalThis.__cuk = document.createEvent('UnknownEvent') instanceof Event;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cse)").unwrap().value,
        "true",
        "createEvent('StorageEvent') instanceof StorageEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cpr)").unwrap().value,
        "true",
        "createEvent('ProgressEvent') instanceof ProgressEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cuk)").unwrap().value,
        "true",
        "createEvent(未知 type) 回落 instanceof Event"
    );
}

#[test]
fn test_custom_elements_r2813() {
    // R2813：customElements (CustomElementRegistry) scoped registry slice——web components 生态门控。
    // define/get/getName/whenDefined（同步 bookkeeping + whenDefined Promise）+ upgrade stub defer。
    // 诚实 defer element upgrade + lifecycle 回调（element proxy 非 ctor 实例 + 需 mutation 观察——深项）。
    // Promise.then 经 execute 末 microtask checkpoint 派发（同 R2774），下 execute 可读。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // feature-detection：typeof customElements === 'object'。
    assert_eq!(
        sandbox.execute("typeof customElements").unwrap().value,
        "object",
        "window.customElements 须存在（object）"
    );

    // define（class extends HTMLElement）+ get 返同 ctor + getName 反查。
    sandbox
        .execute(
            "globalThis.MyEl = class MyEl extends HTMLElement {};\
             customElements.define('my-el', globalThis.MyEl);\
             globalThis.__same = (customElements.get('my-el') === globalThis.MyEl);\
             globalThis.__name = customElements.getName(globalThis.MyEl);\
             globalThis.__missing = customElements.get('no-such');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__same)").unwrap().value,
        "true",
        "get(name) 返已注册 ctor"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__name)").unwrap().value,
        "my-el",
        "getName(ctor) 反查 name"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__missing)").unwrap().value,
        "undefined",
        "get(未注册) 返 undefined"
    );

    // 无效名抛：'div'（无连字符）/ 'MyEl'（大写）/ 'font-face'（reserved）。
    sandbox
        .execute(
            "function _try(fn){ try { fn(); return 'no-throw'; } catch(e){ return 'threw'; } }\
             globalThis.__bad1 = _try(function(){ customElements.define('div', function(){}); });\
             globalThis.__bad2 = _try(function(){ customElements.define('MyEl', function(){}); });\
             globalThis.__bad3 = _try(function(){ customElements.define('font-face', function(){}); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__bad1)").unwrap().value,
        "threw",
        "无连字符名须抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__bad2)").unwrap().value,
        "threw",
        "大写名须抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__bad3)").unwrap().value,
        "threw",
        "reserved 名须抛"
    );

    // 重复名抛 / 重复 ctor 抛 / ctor 非 function 抛。
    sandbox
        .execute(
            "globalThis.__dupName = _try(function(){ customElements.define('my-el', function(){}); });\
             globalThis.__dupCtor = _try(function(){ customElements.define('other-el', globalThis.MyEl); });\
             globalThis.__notFn = _try(function(){ customElements.define('ok-el', 42); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dupName)").unwrap().value,
        "threw",
        "重复名须抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dupCtor)").unwrap().value,
        "threw",
        "重复 ctor 须抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__notFn)").unwrap().value,
        "threw",
        "ctor 非 function 须抛"
    );

    // whenDefined 已定义 → Promise<ctor> resolve（execute 末 microtask 派发，下 execute 可读）。
    sandbox
        .execute(
            "globalThis.__wdCtor = null; globalThis.__wd = false;\
             customElements.whenDefined('my-el').then(function(c){ globalThis.__wdCtor = (c === globalThis.MyEl); globalThis.__wd = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__wd)").unwrap().value,
        "true",
        "whenDefined(已定义) resolve"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__wdCtor)").unwrap().value,
        "true",
        "whenDefined resolve 值为 ctor"
    );

    // whenDefined pending → define 触发 resolve（先挂起，下 execute define，再下 execute 读）。
    sandbox
        .execute(
            "globalThis.__later = false; globalThis.__laterCtor = null;\
             customElements.whenDefined('pending-el').then(function(c){ globalThis.__laterCtor = c; globalThis.__later = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__later)").unwrap().value,
        "false",
        "未 define 前 whenDefined pending 不 resolve"
    );
    sandbox
        .execute("globalThis.PendingEl = class extends HTMLElement {}; customElements.define('pending-el', globalThis.PendingEl);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__later)").unwrap().value,
        "true",
        "define 触发挂起的 whenDefined resolve"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__laterCtor === globalThis.PendingEl)")
            .unwrap()
            .value,
        "true",
        "挂起 resolve 值为 define 的 ctor"
    );

    // whenDefined 无效名 → Promise reject（.catch）。
    sandbox
        .execute(
            "globalThis.__rej = false;\
             customElements.whenDefined('BadName').catch(function(){ globalThis.__rej = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rej)").unwrap().value,
        "true",
        "whenDefined(无效名) reject"
    );

    // upgrade(root) no-op 不抛（defer，element proxy 非 ctor 实例）。
    sandbox
        .execute("globalThis.__up = _try(function(){ customElements.upgrade(document.body); });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__up)").unwrap().value,
        "no-throw",
        "upgrade no-op 不抛"
    );
}

#[test]
fn test_history_pushstate_r2814() {
    // R2814：history session history stack——SPA 路由核心（react-router / vue-router 等）。原 stub no-op，
    // 现实现 in-memory entries + cursor：pushState/replaceState 维护 state/length，back/forward/go 移 cursor
    // + _defer 异步派发 popstate（window listener，复用 R2812 PopStateEvent）。popstate 经 execute 末
    // microtask 派发，下 execute 可读（同 R2774）。已知限制：仅 in-memory（不更新 location / 不接 host 导航）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 初始 length=1 / state=null；pushState 推进 length + state；replaceState 原地替换 state 不增 length。
    sandbox
        .execute(
            "globalThis.__initLen = history.length;\
             globalThis.__initState = history.state;\
             history.pushState({ page: 1 }, '', '/a');\
             globalThis.__len2 = history.length; globalThis.__st2 = history.state.page;\
             history.pushState({ page: 2 }, '', '/b');\
             globalThis.__len3 = history.length; globalThis.__st3 = history.state.page;\
             history.replaceState({ page: 20 }, '', '/b2');\
             globalThis.__len3b = history.length; globalThis.__st3b = history.state.page;\
             globalThis.__sr = history.scrollRestoration;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__initLen)").unwrap().value,
        "1",
        "初始 length=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__initState)").unwrap().value,
        "null",
        "初始 state=null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len2)").unwrap().value,
        "2",
        "pushState 后 length=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__st2)").unwrap().value,
        "1",
        "pushState state.page=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len3)").unwrap().value,
        "3",
        "二次 pushState length=3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__st3)").unwrap().value,
        "2",
        "state.page=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__len3b)").unwrap().value,
        "3",
        "replaceState 不增 length=3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__st3b)").unwrap().value,
        "20",
        "replaceState state.page=20"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sr)").unwrap().value,
        "auto",
        "scrollRestoration='auto'"
    );

    // 安装 popstate listener + back() → cursor 回退到 {page:1}（execute 末 microtask 派发 popstate）。
    sandbox
        .execute(
            "globalThis.__popState = null;\
             addEventListener('popstate', function(e){ globalThis.__popState = e.state; });\
             history.back();",
        )
        .unwrap();
    sandbox
        .execute(
            "globalThis.__popPage = globalThis.__popState ? globalThis.__popState.page : null;\
             globalThis.__curState = history.state.page; globalThis.__curLen = history.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__popPage)").unwrap().value,
        "1",
        "back() popstate 携带 state.page=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__curState)").unwrap().value,
        "1",
        "back() 后 history.state 回到 entry page=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__curLen)").unwrap().value,
        "3",
        "back() 不改 length=3"
    );

    // forward() → cursor 前进到 {page:20}，popstate 携带 {page:20}。
    sandbox.execute("history.forward();").unwrap();
    sandbox
        .execute("globalThis.__fwdPop = globalThis.__popState.page; globalThis.__fwdCur = history.state.page;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fwdPop)").unwrap().value,
        "20",
        "forward() popstate state.page=20"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fwdCur)").unwrap().value,
        "20",
        "forward() 后 state.page=20"
    );

    // go(-2) → cursor 回到 idx0（state=null）；go(0) 不动。
    sandbox.execute("history.go(-2);").unwrap();
    sandbox
        .execute("globalThis.__goStateIsNull = (history.state === null);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__goStateIsNull)").unwrap().value,
        "true",
        "go(-2) 回到初始 entry state=null"
    );

    // 截断：cursor 在 idx0 时 pushState → forward entries 截断 + 新 entry → length=2。
    sandbox
        .execute(
            "history.pushState({ page: 9 }, '', '/x');\
             globalThis.__truncLen = history.length; globalThis.__truncSt = history.state.page;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__truncLen)").unwrap().value,
        "2",
        "back 后 pushState 截断 forward → length=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__truncSt)").unwrap().value,
        "9",
        "截断后 state.page=9"
    );

    // go(99) 越界 clamp 到末尾不抛（state 末 entry）。
    sandbox
        .execute(
            "globalThis.__oob = (function(){ try { history.go(99); return 'ok'; } catch(e){ return 'threw'; } })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__oob)").unwrap().value,
        "ok",
        "go(越界) 不抛（no-op）"
    );
}

#[test]
fn test_history_go_out_of_range_noop_r3004() {
    // R3004：history.go(delta) 越界须为 **no-op**（不动 cursor、不派发 popstate）——spec/MDN：「out-of-range
    // delta does nothing」。旧实现 clamp target 到 [0,len-1] 后移动 + 派发 popstate（SPA router 计算的 delta
    // 过冲时误导航到边界）。经 history.state 同步可观测：越界 go 后 state 不变；in-range go 正常移动。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 建 3 entry（init null + a + b），back 到中间（cursor 在 a，state.page=1）。
    sandbox
        .execute(
            "history.pushState({ page: 1 }, '', '/a');\
             history.pushState({ page: 2 }, '', '/b');\
             history.back();\
             globalThis.__midState = history.state.page;\
             globalThis.__midLen = history.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__midState)").unwrap().value,
        "1",
        "back 后 cursor 在中间 entry state.page=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__midLen)").unwrap().value,
        "3",
        "length=3"
    );

    // go(-100) 越界（仅能回退 1）→ no-op：state 不变（仍 page=1），length 不变。
    sandbox
        .execute(
            "history.go(-100);\
             globalThis.__afterBack = history.state.page;\
             globalThis.__afterBackLen = history.length;\
             globalThis.__oobBack = (function(){ try { return 'ok'; } catch(e){ return 'threw'; } })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__oobBack)").unwrap().value,
        "ok",
        "go(-100) 越界不抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__afterBack)").unwrap().value,
        "1",
        "go(-100) 越界 no-op：state 不变 page=1（旧 clamp 会移到 idx0 state=null）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__afterBackLen)").unwrap().value,
        "3",
        "go(-100) 越界 length 不变=3"
    );

    // go(100) 越界（仅能前进 1）→ no-op：state 不变。
    sandbox
        .execute(
            "history.go(100);\
             globalThis.__afterFwd = history.state.page;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__afterFwd)").unwrap().value,
        "1",
        "go(100) 越界 no-op：state 不变 page=1（旧 clamp 会移到 idx2 state.page=2）"
    );

    // in-range go(-1) 正常移动（state→null，回 init entry）。
    sandbox
        .execute(
            "history.go(-1);\
             globalThis.__inRangeBack = (history.state === null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__inRangeBack)").unwrap().value,
        "true",
        "in-range go(-1) 正常回退到 init entry state=null"
    );

    // in-range go(2) 正常前进（state→page=2，b entry）。
    sandbox
        .execute(
            "history.go(2);\
             globalThis.__inRangeFwd = history.state.page;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__inRangeFwd)").unwrap().value,
        "2",
        "in-range go(2) 正常前进到 b entry state.page=2"
    );

    // go(0) 不移动（headless 无真 reload，近似 no-op：state 不变）。
    sandbox
        .execute(
            "history.go(0);\
             globalThis.__goZero = history.state.page;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__goZero)").unwrap().value,
        "2",
        "go(0) 不移动：state 仍 page=2（headless 近似 reload no-op）"
    );
}

#[test]
fn test_node_relation_implementation_r2815() {
    // R2815：document.implementation (DOMImplementation) + 节点关系方法（getRootNode/compareDocumentPosition/
    // isSameNode）+ Node.DOCUMENT_POSITION_* 常量。compareDocumentPosition bitmask 经 _ancestorChain + LCA +
    // __zw_element_children 子序比较。createComment defer（需 host DomMutation 桥）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='parent'><div id='a'>A</div><div id='b'>B</div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // document.implementation.hasFeature 恒 true + createHTMLDocument 返 hollow doc（body/title）。
    sandbox
        .execute(
            "globalThis.__hf = document.implementation.hasFeature('HTML', '1.0');\
             globalThis.__hdoc = document.implementation.createHTMLDocument('hi');\
             globalThis.__hbody = __hdoc.body.tagName; globalThis.__htitle = __hdoc.title;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hf)").unwrap().value,
        "true",
        "hasFeature 恒 true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hbody)").unwrap().value,
        "BODY",
        "createHTMLDocument doc.body.tagName BODY"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__htitle)").unwrap().value,
        "hi",
        "createHTMLDocument title 透传"
    );

    // getRootNode：#a 的根为 html（documentElement）。
    sandbox
        .execute(
            "globalThis.__a = document.querySelector('#a');\
             globalThis.__root = __a.getRootNode().tagName;\
             globalThis.__rootIsDocEl = (__a.getRootNode() === document.documentElement);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__root)").unwrap().value,
        "HTML",
        "getRootNode 返根 html"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rootIsDocEl)").unwrap().value,
        "true",
        "getRootNode() === document.documentElement"
    );

    // isSameNode：自身 true / 他节点 false。
    sandbox
        .execute(
            "globalThis.__b = document.querySelector('#b');\
             globalThis.__same = __a.isSameNode(document.querySelector('#a'));\
             globalThis.__diff = __a.isSameNode(__b);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__same)").unwrap().value,
        "true",
        "isSameNode 自身 true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__diff)").unwrap().value,
        "false",
        "isSameNode 他节点 false"
    );

    // compareDocumentPosition bitmask + Node 常量。
    sandbox
        .execute(
            "globalThis.__F = Node.DOCUMENT_POSITION_FOLLOWING;\
             globalThis.__Ct = Node.DOCUMENT_POSITION_CONTAINS;\
             globalThis.__self = __a.compareDocumentPosition(__a);\
             globalThis.__htmlBody = document.documentElement.compareDocumentPosition(document.body);\
             globalThis.__bodyHtml = document.body.compareDocumentPosition(document.documentElement);\
             globalThis.__parent = document.querySelector('#parent');\
             globalThis.__ab = __a.compareDocumentPosition(__b);\
             globalThis.__ba = __b.compareDocumentPosition(__a);\
             globalThis.__parentA = __parent.compareDocumentPosition(__a);\
             globalThis.__aParent = __a.compareDocumentPosition(__parent);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__F)").unwrap().value,
        "4",
        "Node.DOCUMENT_POSITION_FOLLOWING=4"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__Ct)").unwrap().value,
        "8",
        "Node.DOCUMENT_POSITION_CONTAINS=8"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__self)").unwrap().value,
        "0",
        "compareDocumentPosition(自身)=0"
    );
    // html 含 body，body 跟随 html → CONTAINED_BY(16)|FOLLOWING(4)=20。
    assert_eq!(
        sandbox.execute("String(globalThis.__htmlBody)").unwrap().value,
        "20",
        "html.cDP(body)=CONTAINED_BY|FOLLOWING=20"
    );
    // body 看 html：html 含 body + html 先于 body → CONTAINS(8)|PRECEDING(2)=10。
    assert_eq!(
        sandbox.execute("String(globalThis.__bodyHtml)").unwrap().value,
        "10",
        "body.cDP(html)=CONTAINS|PRECEDING=10"
    );
    // a 先于 b（兄弟）→ b 跟随 a → FOLLOWING(4)。
    assert_eq!(
        sandbox.execute("String(globalThis.__ab)").unwrap().value,
        "4",
        "a.cDP(b)=FOLLOWING=4（a 先于 b）"
    );
    // b 看 a → a 先于 b → PRECEDING(2)。
    assert_eq!(
        sandbox.execute("String(globalThis.__ba)").unwrap().value,
        "2",
        "b.cDP(a)=PRECEDING=2"
    );
    // parent 含 a，a 跟随 → CONTAINED_BY|FOLLOWING=20。
    assert_eq!(
        sandbox.execute("String(globalThis.__parentA)").unwrap().value,
        "20",
        "parent.cDP(a)=CONTAINED_BY|FOLLOWING=20"
    );
    // a 看 parent → CONTAINS|PRECEDING=10。
    assert_eq!(
        sandbox.execute("String(globalThis.__aParent)").unwrap().value,
        "10",
        "a.cDP(parent)=CONTAINS|PRECEDING=10"
    );
}

#[test]
fn test_create_comment_r2816() {
    // R2816：document.createComment——注释节点（nodeType 8）。host DomMutation::CreateComment 变体 + apply
    //（doc.create_comment）+ __zw_create_comment callback；shim _commentHandles 标识 nodeType/nodeName +
    // textContent/nodeValue/data 经 query_text_from_mutations（CreateComment arm）读回。框架 placeholder/anchor。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // createComment 返节点：nodeType=8 / nodeName '#comment' / tagName undefined / nodeValue=data=textContent=文本。
    sandbox
        .execute(
            "globalThis.__c = document.createComment('hi there');\
             globalThis.__nt = __c.nodeType;\
             globalThis.__nn = __c.nodeName;\
             globalThis.__tag = __c.tagName;\
             globalThis.__nv = __c.nodeValue;\
             globalThis.__data = __c.data;\
             globalThis.__tc = __c.textContent;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nt)").unwrap().value,
        "8",
        "createComment nodeType=8"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nn)").unwrap().value,
        "#comment",
        "nodeName '#comment'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tag)").unwrap().value,
        "undefined",
        "comment tagName undefined"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nv)").unwrap().value,
        "hi there",
        "nodeValue=注释文本"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__data)").unwrap().value,
        "hi there",
        "data=注释文本"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tc)").unwrap().value,
        "hi there",
        "textContent=注释文本"
    );

    // 区别于 createTextNode（nodeType=3）。
    sandbox
        .execute(
            "globalThis.__t = document.createTextNode('txt');\
             globalThis.__tnt = __t.nodeType;\
             globalThis.__cnt = __c.nodeType;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tnt)").unwrap().value,
        "3",
        "createTextNode nodeType=3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cnt)").unwrap().value,
        "8",
        "createComment 仍 nodeType=8（区别 text）"
    );

    // host 记 CreateComment mutation（验证 host 桥接）。
    let muts = mutations.lock().unwrap();
    let has_comment = muts
        .iter()
        .any(|m| matches!(m, DomMutation::CreateComment { text, .. } if text == "hi there"));
    drop(muts);
    assert!(
        has_comment,
        "createComment 须经 __zw_create_comment 记 DomMutation::CreateComment"
    );

    // 空串/数字参数 lenient 转 string 不抛。
    sandbox
        .execute(
            "globalThis.__ok = (function(){ try { document.createComment(''); document.createComment(42); return 'ok'; } catch(e){ return 'threw'; } })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ok)").unwrap().value,
        "ok",
        "createComment lenient 不抛"
    );
}

#[test]
fn test_modern_interaction_stubs_r2817() {
    // R2817：现代交互 API stubs 簇——navigator.clipboard/permissions + element.requestFullscreen +
    // document.fullscreen/exitFullscreen + element/window scroll。headless 无真剪贴板/全屏/滚动 → resolving
    // Promise（clipboard/fullscreen）或 no-op（scroll）。高频 feature-detection 点不抛。Promise 经 execute
    // 末 microtask checkpoint 派发，下 execute 可读（同 R2774/R2814）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // navigator.clipboard（R2817 stubs + R2964 真实化）：typeof object + writeText/readText 返 Promise +
    // writeText→readText 进程内 store 往返（writeText 同步写 store，readText 读之；execute 末 microtask 派发）。
    assert_eq!(
        sandbox.execute("typeof navigator.clipboard").unwrap().value,
        "object",
        "navigator.clipboard 存在"
    );
    sandbox
        .execute(
            "globalThis.__wb = false; globalThis.__rt = 'X';\
             navigator.clipboard.writeText('hi').then(function(){ globalThis.__wb = true; });\
             navigator.clipboard.readText().then(function(t){ globalThis.__rt = t; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__wb)").unwrap().value,
        "true",
        "clipboard.writeText Promise resolves"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rt)").unwrap().value,
        "hi",
        "clipboard.readText 返 writeText 写入值（R2964 进程内 store 往返）"
    );

    // navigator.permissions.query → Promise<PermissionStatus state 'prompt'>。
    sandbox
        .execute(
            "globalThis.__perm = null;\
             navigator.permissions.query({ name: 'clipboard' }).then(function(s){ globalThis.__perm = s.state + ':' + s.name; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__perm)").unwrap().value,
        "prompt:clipboard",
        "permissions.query → state 'prompt' + name 透传"
    );

    // element.requestFullscreen → Promise resolves + 设 fullscreenElement=body；exitFullscreen 清 + resolve。
    // （R2817 时 fullscreenElement 恒 null；R2938 升级为 spec-alike 状态追踪，详见 test_fullscreen_api_r2938。）
    sandbox
        .execute(
            "globalThis.__fs = false;\
             document.body.requestFullscreen().then(function(){ globalThis.__fs = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fs)").unwrap().value,
        "true",
        "requestFullscreen Promise resolves"
    );
    assert_eq!(
        sandbox
            .execute("String(document.fullscreenElement === document.body)")
            .unwrap()
            .value,
        "true",
        "R2938 fullscreenElement 反映全屏元素（body）"
    );
    sandbox
        .execute("globalThis.__ef = false; document.exitFullscreen().then(function(){ globalThis.__ef = true; });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ef)").unwrap().value,
        "true",
        "exitFullscreen Promise resolves"
    );
    assert_eq!(
        sandbox.execute("String(document.fullscreenElement)").unwrap().value,
        "null",
        "R2938 exitFullscreen 后 fullscreenElement 复 null"
    );

    // element scroll 方法 no-op 返 undefined；window scroll 同；scrollX/pageXOffset 恒 0。
    sandbox
        .execute(
            "globalThis.__siv = document.body.scrollIntoView();\
             globalThis.__sto = document.body.scrollTo(0, 0);\
             globalThis.__wst = window.scrollTo(0, 0);\
             globalThis.__sX = window.scrollX; globalThis.__pXO = window.pageXOffset;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__siv)").unwrap().value,
        "undefined",
        "scrollIntoView no-op 返 undefined"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sto)").unwrap().value,
        "undefined",
        "scrollTo no-op 返 undefined"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__wst)").unwrap().value,
        "undefined",
        "window.scrollTo no-op 返 undefined"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sX)").unwrap().value,
        "0",
        "scrollX 恒 0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pXO)").unwrap().value,
        "0",
        "pageXOffset 恒 0"
    );
}

#[test]
fn test_clipboard_write_read_round_trip_r2964() {
    // R2964：navigator.clipboard.writeText/readText 真实化（进程内 store 往返）。覆盖写入→读、覆盖写、
    // 非字符串归一、空默认。headless 无 OS 剪贴板——store 同页/同进程 write→read 通（复制按钮 + 粘贴检查）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 新 sandbox readText 默认空。
    sandbox.execute("globalThis.__e='X'; navigator.clipboard.readText().then(function(t){globalThis.__e=t;});").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__e)").unwrap().value, "");
    // writeText→readText 往返。
    sandbox
        .execute(
            "globalThis.__r='X';\
             navigator.clipboard.writeText('hello').then(function(){ return navigator.clipboard.readText(); })\
               .then(function(t){ globalThis.__r = t; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__r)").unwrap().value, "hello");
    // 覆盖写：writeText('second') → readText()='second'（非首值残留）。
    sandbox
        .execute(
            "globalThis.__r2='X';\
             navigator.clipboard.writeText('second').then(function(){ return navigator.clipboard.readText(); })\
               .then(function(t){ globalThis.__r2 = t; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__r2)").unwrap().value, "second");
    // 非字符串归一（数字 → '42'，null → 'null'... 实际 null→'' 因 String(null)=='null'... 验 number 归一）。
    sandbox
        .execute(
            "globalThis.__r3='X';\
             navigator.clipboard.writeText(42).then(function(){ return navigator.clipboard.readText(); })\
               .then(function(t){ globalThis.__r3 = t; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__r3)").unwrap().value, "42");
    // read/write（ClipboardItem 富 MIME）仍 best-effort stub（不抛，read 返 []）。
    sandbox
        .execute(
            "globalThis.__rw='X';\
             Promise.all([navigator.clipboard.read(), navigator.clipboard.write([])]).then(function(r){ globalThis.__rw = String(r[0].length); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__rw)").unwrap().value, "0");
}

#[test]
fn test_fullscreen_api_r2938() {
    // R2938 Fullscreen API（spec-alike）：element.requestFullscreen() 返 Promise——grant 路径设 fullscreenElement +
    // 派 fullscreenchange + resolve；deny 路径（fullscreenEnabled=false）派 fullscreenerror + reject TypeError。
    // document.exitFullscreen() 清状态 + 派 fullscreenchange + resolve；非全屏态 resolve 不派事件。
    // fullscreenElement/fullscreenEnabled 反映状态；fullscreenchange/fullscreenerror 经 document listener +
    // document.onfullscreenchange/onfullscreenerror IDL handler 触发。headless 无真 OS 全屏，但语义可观察。
    // https://fullscreen.spec.whatwg.org/
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='d'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // fullscreenEnabled 默认 true；fullscreenElement 初值 null。
    assert_eq!(
        sandbox.execute("String(document.fullscreenEnabled)").unwrap().value,
        "true",
        "fullscreenEnabled 默认 true"
    );
    assert_eq!(
        sandbox.execute("String(document.fullscreenElement)").unwrap().value,
        "null",
        "fullscreenElement 初值 null"
    );

    // grant 路径：requestFullscreen 设 fullscreenElement + 派 fullscreenchange（listener 内读 fullscreenElement）+ resolve。
    sandbox
        .execute(
            "globalThis.__fc = 0; globalThis.__fe = 'x';\
             document.addEventListener('fullscreenchange', function(){\
               globalThis.__fc++;\
               globalThis.__fe = document.fullscreenElement ? document.fullscreenElement.id : '(null)';\
             });\
             globalThis.__ok = false;\
             document.getElementById('d').requestFullscreen().then(function(){ globalThis.__ok = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fc)").unwrap().value,
        "1",
        "requestFullscreen 派发一次 fullscreenchange"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fe)").unwrap().value,
        "d",
        "fullscreenchange handler 内 fullscreenElement === 全屏元素（id='d'）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ok)").unwrap().value,
        "true",
        "requestFullscreen Promise resolves"
    );

    // 相同元素重复 requestFullscreen → no-op（不重复派 fullscreenchange，仍 resolve）。
    sandbox
        .execute(
            "globalThis.__ok2 = false;\
             document.getElementById('d').requestFullscreen().then(function(){ globalThis.__ok2 = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fc)").unwrap().value,
        "1",
        "相同元素重复 requestFullscreen 不再派 fullscreenchange（no-op）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ok2)").unwrap().value,
        "true",
        "重复 requestFullscreen 仍 resolve"
    );

    // exitFullscreen → 清状态 + 派 fullscreenchange + resolve。
    sandbox
        .execute(
            "globalThis.__ef = false;\
             document.exitFullscreen().then(function(){ globalThis.__ef = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fc)").unwrap().value,
        "2",
        "exitFullscreen 派发 fullscreenchange（第二次）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ef)").unwrap().value,
        "true",
        "exitFullscreen Promise resolves"
    );
    assert_eq!(
        sandbox.execute("String(document.fullscreenElement)").unwrap().value,
        "null",
        "exitFullscreen 后 fullscreenElement 复 null"
    );

    // 非全屏态 exitFullscreen → resolve，不派事件（计数不变）。
    sandbox
        .execute(
            "globalThis.__ef2 = 'p';\
             document.exitFullscreen().then(function(){ globalThis.__ef2 = 'resolved'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ef2)").unwrap().value,
        "resolved",
        "非全屏态 exitFullscreen 仍 resolve"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fc)").unwrap().value,
        "2",
        "非全屏态 exitFullscreen 不派 fullscreenchange"
    );

    // document.onfullscreenchange IDL handler：注册后由 fullscreenchange 触发。
    sandbox
        .execute(
            "globalThis.__ofc = 0;\
             document.onfullscreenchange = function(){ globalThis.__ofc++; };\
             document.body.requestFullscreen();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ofc)").unwrap().value,
        "1",
        "document.onfullscreenchange IDL handler 触发"
    );
    sandbox.execute("document.exitFullscreen();").unwrap(); // 清理全屏态

    // deny 路径：host `__zw_fullscreen_enabled` 返 '0' → fullscreenEnabled=false → requestFullscreen reject
    // TypeError + 派 fullscreenerror（document listener + document.onfullscreenerror IDL handler 触发）。
    sandbox.register_callback("__zw_fullscreen_enabled", Box::new(|_args: &[String]| "0".to_string()));
    assert_eq!(
        sandbox.execute("String(document.fullscreenEnabled)").unwrap().value,
        "false",
        "host 禁用后 fullscreenEnabled=false"
    );
    sandbox
        .execute(
            "globalThis.__ferr = 0; globalThis.__rej = null;\
             document.addEventListener('fullscreenerror', function(){ globalThis.__ferr++; });\
             document.onfullscreenerror = function(){ globalThis.__ferr += 10; };\
             document.body.requestFullscreen().then(function(){ globalThis.__rej = 'resolved'; },\
               function(err){ globalThis.__rej = (err instanceof TypeError) ? 'TypeError' : 'other'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ferr)").unwrap().value,
        "11",
        "deny 路径派 fullscreenerror（document listener + document.onfullscreenerror）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rej)").unwrap().value,
        "TypeError",
        "deny 路径 reject TypeError"
    );
    assert_eq!(
        sandbox.execute("String(document.fullscreenElement)").unwrap().value,
        "null",
        "deny 路径不设 fullscreenElement"
    );
}

#[test]
fn test_pointer_lock_api_r2939() {
    // R2939 Pointer Lock API（spec-alike，镜像 R2938 Fullscreen）：element.requestPointerLock() 返 Promise
    //（grant→resolve + 设 pointerLockElement + 派 pointerlockchange；deny→reject TypeError + 派 pointerlockerror）；
    // document.exitPointerLock() 返 **void**（undefined，与 exitFullscreen 返 Promise 不同）+ 清状态 + 派
    // pointerlockchange；pointerLockElement 反映状态；pointerlockchange/pointerlockerror 经 document listener +
    // document.onpointerlockchange/onpointerlockerror IDL handler 触发。
    // https://w3c.github.io/pointerlock/
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><canvas id='c'></canvas></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // pointerLockElement 初值 null。
    assert_eq!(
        sandbox.execute("String(document.pointerLockElement)").unwrap().value,
        "null",
        "pointerLockElement 初值 null"
    );

    // grant 路径：requestPointerLock 设 pointerLockElement + 派 pointerlockchange + resolve。
    sandbox
        .execute(
            "globalThis.__plc = 0; globalThis.__ple = 'x';\
             document.addEventListener('pointerlockchange', function(){\
               globalThis.__plc++;\
               globalThis.__ple = document.pointerLockElement ? document.pointerLockElement.id : '(null)';\
             });\
             globalThis.__ok = false;\
             document.getElementById('c').requestPointerLock().then(function(){ globalThis.__ok = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__plc)").unwrap().value,
        "1",
        "requestPointerLock 派发一次 pointerlockchange"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ple)").unwrap().value,
        "c",
        "pointerlockchange handler 内 pointerLockElement === 锁定元素（id='c'）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ok)").unwrap().value,
        "true",
        "requestPointerLock Promise resolves"
    );

    // 相同元素重复 requestPointerLock → no-op（计数不变仍 resolve）。
    sandbox
        .execute(
            "globalThis.__ok2 = false;\
             document.getElementById('c').requestPointerLock().then(function(){ globalThis.__ok2 = true; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__plc)").unwrap().value,
        "1",
        "相同元素重复 requestPointerLock 不再派 pointerlockchange（no-op）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ok2)").unwrap().value,
        "true",
        "重复 requestPointerLock 仍 resolve"
    );

    // exitPointerLock → 清状态 + 派 pointerlockchange；返 void（undefined，与 exitFullscreen 返 Promise 不同）。
    sandbox
        .execute("globalThis.__ex = typeof document.exitPointerLock();")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ex)").unwrap().value,
        "undefined",
        "exitPointerLock 返 void（undefined，spec 非 Promise）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__plc)").unwrap().value,
        "2",
        "exitPointerLock 派发 pointerlockchange（第二次）"
    );
    assert_eq!(
        sandbox.execute("String(document.pointerLockElement)").unwrap().value,
        "null",
        "exitPointerLock 后 pointerLockElement 复 null"
    );

    // 非锁定态 exitPointerLock → no-op，不派事件（计数不变）。
    sandbox.execute("document.exitPointerLock();").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__plc)").unwrap().value,
        "2",
        "非锁定态 exitPointerLock 不派 pointerlockchange"
    );

    // document.onpointerlockchange IDL handler：注册后由 pointerlockchange 触发。
    sandbox
        .execute(
            "globalThis.__oplc = 0;\
             document.onpointerlockchange = function(){ globalThis.__oplc++; };\
             document.body.requestPointerLock();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__oplc)").unwrap().value,
        "1",
        "document.onpointerlockchange IDL handler 触发"
    );
    sandbox.execute("document.exitPointerLock();").unwrap(); // 清理锁定态

    // deny 路径：host `__zw_pointer_lock_enabled` 返 '0' → requestPointerLock reject TypeError + 派 pointerlockerror
    //（document listener + document.onpointerlockerror IDL handler 触发）。
    sandbox.register_callback(
        "__zw_pointer_lock_enabled",
        Box::new(|_args: &[String]| "0".to_string()),
    );
    sandbox
        .execute(
            "globalThis.__plerr = 0; globalThis.__rej = null;\
             document.addEventListener('pointerlockerror', function(){ globalThis.__plerr++; });\
             document.onpointerlockerror = function(){ globalThis.__plerr += 10; };\
             document.body.requestPointerLock().then(function(){ globalThis.__rej = 'resolved'; },\
               function(err){ globalThis.__rej = (err instanceof TypeError) ? 'TypeError' : 'other'; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__plerr)").unwrap().value,
        "11",
        "deny 路径派 pointerlockerror（document listener + document.onpointerlockerror）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rej)").unwrap().value,
        "TypeError",
        "deny 路径 reject TypeError"
    );
    assert_eq!(
        sandbox.execute("String(document.pointerLockElement)").unwrap().value,
        "null",
        "deny 路径不设 pointerLockElement"
    );
}

#[test]
fn test_window_onerror_report_r2940() {
    // R2940 onerror host 集成：ErrorEvent 构造器 + createEvent('ErrorEvent') + __zw_report_error hook。
    // hook 派发 window 'error' 事件（addEventListener 接 ErrorEvent 读 .message/.filename/.lineno/.colno）+
    // 调 legacy window.onerror（spec 特殊 5-arg 签名 msg/src/line/col/err），不重复触发 onerror（dispatch 前
    // 暂移除 onerror listener、legacy 调、dispatch 完装回）。onerror 返 true → defaultPrevented（错误已处理）。
    // host（tab_scripts）执行页面 <script> 出错时经 zero_engine::script_report_error 生成调用串执行此 hook。
    // https://html.spec.whatwg.org/#runtime-script-errors
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://test.local/page".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ErrorEvent 构造器（字段 message/filename/lineno/colno/error）+ createEvent('ErrorEvent') 返 ErrorEvent 实例。
    sandbox
        .execute(
            "globalThis.__ev = new ErrorEvent('error', {message:'boom', filename:'a.js', lineno:7, colno:3});\
             globalThis.__ce = document.createEvent('ErrorEvent');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.message)").unwrap().value,
        "boom",
        "ErrorEvent.message"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.filename)").unwrap().value,
        "a.js",
        "ErrorEvent.filename"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.lineno)").unwrap().value,
        "7",
        "ErrorEvent.lineno"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.colno)").unwrap().value,
        "3",
        "ErrorEvent.colno"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ev.error)").unwrap().value,
        "null",
        "ErrorEvent.error（headless 无真 Error 对象 → null）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ev instanceof ErrorEvent)")
            .unwrap()
            .value,
        "true",
        "ErrorEvent instanceof"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ce instanceof ErrorEvent)")
            .unwrap()
            .value,
        "true",
        "createEvent('ErrorEvent') 返 ErrorEvent 实例"
    );

    // __zw_report_error：window.addEventListener('error') 接 ErrorEvent + window.onerror legacy 5-arg，不重复触发。
    sandbox
        .execute(
            "globalThis.__ael = null; globalThis.__oe = null; globalThis.__oeCount = 0;\
             window.addEventListener('error', function(e){\
               globalThis.__ael = e.message + '|' + e.filename + '|' + e.lineno + '|' + e.colno;\
             });\
             window.onerror = function(msg, src, line, col, err){\
               globalThis.__oeCount++;\
               globalThis.__oe = String(msg) + '|' + String(src) + '|' + line + '|' + col + '|' + String(err);\
               return false;\
             };\
             __zw_report_error('TypeError: x is undefined', 'https://test.local/a.js', 42, 9);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ael)").unwrap().value,
        "TypeError: x is undefined|https://test.local/a.js|42|9",
        "addEventListener('error') listener 接 ErrorEvent（字段透传）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__oe)").unwrap().value,
        "TypeError: x is undefined|https://test.local/a.js|42|9|null",
        "window.onerror legacy 5-arg 签名（msg/src/line/col/err）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__oeCount)").unwrap().value,
        "1",
        "onerror 仅触发一次（不与 event 派发重复）"
    );

    // onerror 返 true → defaultPrevented（错误「已处理」，spec：抑制默认动作）。
    sandbox
        .execute(
            "globalThis.__dp = 'unset';\
             window.onerror = function(){ return true; };\
             window.addEventListener('error', function(e){ globalThis.__dp = String(e.defaultPrevented); });\
             __zw_report_error('handled', 'b.js', 1, 1);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dp)").unwrap().value,
        "true",
        "onerror 返 true → ErrorEvent.defaultPrevented"
    );

    // 仅 addEventListener('error')（无 onerror）也能触发——hook 不依赖 onerror 存在。
    sandbox
        .execute(
            "window.onerror = null;\
             globalThis.__only2 = null;\
             window.addEventListener('error', function(e){ globalThis.__only2 = e.message; });\
             __zw_report_error('solo', 'c.js', 5, 5);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__only2)").unwrap().value,
        "solo",
        "无 onerror 时 addEventListener('error') 仍触发"
    );
}

#[test]
fn test_page_lifecycle_load_r2941() {
    // R2941 页面生命周期事件派发：host（tab_scripts::finish）在页面脚本阶段完成后依次派发
    // DOMContentLoaded + load（均经 __zw_dispatch_event('html', type, null) → document/window listener 同键）。
    // 触发 document.addEventListener('DOMContentLoaded') / window.addEventListener('load') / window.onload
    //（R2932 IDL）/ document.onDOMContentLoaded（R2941 IDL）。DOMContentLoaded 先于 load（spec）。
    // https://html.spec.whatwg.org/#the-end
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // document.onDOMContentLoaded（R2941 新增 IDL handler）可读写（无值初 null）。
    assert_eq!(
        sandbox.execute("String(document.onDOMContentLoaded)").unwrap().value,
        "null",
        "document.onDOMContentLoaded 初值 null"
    );

    // 注册四类 hook：document.addEListener('DOMContentLoaded') / window.onload / window.addEventListener('load') /
    // document.onDOMContentLoaded。记录触发顺序进 __order。
    sandbox
        .execute(
            "globalThis.__order = [];\
             document.addEventListener('DOMContentLoaded', function(){ globalThis.__order.push('dcl-ael'); });\
             document.onDOMContentLoaded = function(){ globalThis.__order.push('dcl-idl'); };\
             window.onload = function(){ globalThis.__order.push('load-idl'); };\
             window.addEventListener('load', function(){ globalThis.__order.push('load-ael'); });",
        )
        .unwrap();

    // host 模拟：finish() 依次派发 DOMContentLoaded + load（DOMContentLoaded 先于 load）。
    sandbox
        .execute(
            "__zw_dispatch_event('html', 'DOMContentLoaded', null);\
             __zw_dispatch_event('html', 'load', null);",
        )
        .unwrap();
    // DOMContentLoaded 触发 document.addEventListener + document.onDOMContentLoaded（均 html 键）。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__order.indexOf('dcl-ael') >= 0)")
            .unwrap()
            .value,
        "true",
        "DOMContentLoaded 派发触发 document.addEventListener('DOMContentLoaded')"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__order.indexOf('dcl-idl') >= 0)")
            .unwrap()
            .value,
        "true",
        "DOMContentLoaded 派发触发 document.onDOMContentLoaded（R2941 IDL）"
    );
    // load 触发 window.onload（R2932 IDL）+ window.addEventListener('load')。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__order.indexOf('load-idl') >= 0)")
            .unwrap()
            .value,
        "true",
        "load 派发触发 window.onload（R2932 IDL）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__order.indexOf('load-ael') >= 0)")
            .unwrap()
            .value,
        "true",
        "load 派发触发 window.addEventListener('load')"
    );
    // DOMContentLoaded 整体先于 load（spec：DOMContentLoaded → load）：最后一条 dcl 记录在首条 load 记录前。
    sandbox.execute(
        "globalThis.__dclFirst = (function(){ var s = globalThis.__order.join(','); return s.lastIndexOf('dcl') < s.indexOf('load'); })();",
    ).unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dclFirst)").unwrap().value,
        "true",
        "DOMContentLoaded 先于 load"
    );
}

#[test]
fn test_img_element_events_r2943() {
    // R2943 img 元素级 onload/onerror：`__zw_dispatch_img_event(absUrl, type)` 按 src 绝对 URL 匹配 `<img>`
    // proxy，用其自身 selector 派发 load/error（保证 listener store key 与 page JS 经 querySelectorAll
    // 获取 proxy 时一致 → img.onload/onerror + addEventListener('load'/'error') 触发）。模拟 host 在 img
    // fetch 完成（'load'）/ 失败（'error'）时调用。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <img id='i1' src='https://example.com/a.png'>\
         <img src='https://example.com/b.png'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // img#i1：addEventListener('load') + onload IDL + onerror；img b.png：onload。
    sandbox
        .execute(
            "globalThis.__hit = [];\
             var imgs = document.querySelectorAll('img');\
             imgs[0].addEventListener('load', function(){ globalThis.__hit.push('i1-load-ael'); });\
             imgs[0].onload = function(){ globalThis.__hit.push('i1-load-idl'); };\
             imgs[0].onerror = function(){ globalThis.__hit.push('i1-error'); };\
             imgs[1].onload = function(){ globalThis.__hit.push('b-load'); };",
        )
        .unwrap();

    // host 派发：i1 load + i1 error + b load（绝对 URL；src 已绝对故不经 parse_url 解析）。
    sandbox
        .execute(
            "__zw_dispatch_img_event('https://example.com/a.png', 'load');\
             __zw_dispatch_img_event('https://example.com/a.png', 'error');\
             __zw_dispatch_img_event('https://example.com/b.png', 'load');",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('i1-load-ael') >= 0)")
            .unwrap()
            .value,
        "true",
        "img#i1 addEventListener('load') 触发（元素自身 selector 派发）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('i1-load-idl') >= 0)")
            .unwrap()
            .value,
        "true",
        "img#i1 onload IDL 触发"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('i1-error') >= 0)")
            .unwrap()
            .value,
        "true",
        "img#i1 onerror 触发（fetch/decode 失败派 error）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('b-load') >= 0)")
            .unwrap()
            .value,
        "true",
        "img b.png onload 触发（多 img 按 src 区分）"
    );
    // 未匹配 src 不派发（计数不变）。
    sandbox
        .execute(
            "globalThis.__before = globalThis.__hit.length;\
             __zw_dispatch_img_event('https://example.com/missing.png', 'load');",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.length === globalThis.__before)")
            .unwrap()
            .value,
        "true",
        "未匹配 src 的派发不触发任何 img listener"
    );
}

#[test]
fn test_link_script_element_events_r2944() {
    // R2944 link/script 元素级 onload/onerror：`__zw_dispatch_link_event(href,type)` / `__zw_dispatch_script_event(src,type)`
    // 经通用 `__zw_dispatch_element_event(tag,attr,url,type)` 按 href/src 绝对 URL 匹配元素 proxy，用其自身
    // selector 派发（保证 listener key 匹配）。模拟 host 在样式表/外部脚本 fetch 成功（load）/ 失败（error）时调用。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head>\
         <link rel='stylesheet' href='https://example.com/a.css'>\
         <script src='https://example.com/s.js'></script>\
         </head><body></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // link.onload/onerror + script.onload/onerror 注册（querySelectorAll 取唯一 selector proxy）。
    sandbox
        .execute(
            "globalThis.__hit = [];\
             var link = document.querySelectorAll('link')[0];\
             link.onload = function(){ globalThis.__hit.push('link-load'); };\
             link.onerror = function(){ globalThis.__hit.push('link-error'); };\
             var sc = document.querySelectorAll('script')[0];\
             sc.onload = function(){ globalThis.__hit.push('script-load'); };\
             sc.onerror = function(){ globalThis.__hit.push('script-error'); };",
        )
        .unwrap();

    // host 派发：link load + link error + script load + script error（绝对 URL）。
    sandbox
        .execute(
            "__zw_dispatch_link_event('https://example.com/a.css', 'load');\
             __zw_dispatch_link_event('https://example.com/a.css', 'error');\
             __zw_dispatch_script_event('https://example.com/s.js', 'load');\
             __zw_dispatch_script_event('https://example.com/s.js', 'error');",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('link-load') >= 0)")
            .unwrap()
            .value,
        "true",
        "link onload 触发"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('link-error') >= 0)")
            .unwrap()
            .value,
        "true",
        "link onerror 触发"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('script-load') >= 0)")
            .unwrap()
            .value,
        "true",
        "script onload 触发（外部脚本 fetch+执行成功）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.indexOf('script-error') >= 0)")
            .unwrap()
            .value,
        "true",
        "script onerror 触发（外部脚本 fetch 失败）"
    );
    // 未匹配 href/src 不派发。
    sandbox
        .execute(
            "globalThis.__before = globalThis.__hit.length;\
             __zw_dispatch_link_event('https://example.com/missing.css', 'load');\
             __zw_dispatch_script_event('https://example.com/missing.js', 'load');",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__hit.length === globalThis.__before)")
            .unwrap()
            .value,
        "true",
        "未匹配 href/src 的派发不触发任何 listener"
    );
}

#[test]
fn test_body_onload_reflection_r2946() {
    // R2946 <body on*> → window.on* 反射（HTML spec body→window event handler reflection）。
    // <body onload="..."> 经 __zw_reflect_body_handlers（__zw_begin_script 内 / lifecycle 派发前调用）
    // 编译为 window.onload，使 __zw_dispatch_event('html','load') 触发——与 window.addEventListener('load')
    // / window.onload = fn 路径合一。每页一次（page URL 去重），JS 设值优先（window.onload = custom 不被覆盖）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body onload=\"globalThis.__hit='body-onload'\" onresize=\"globalThis.__resize='body-resize'\"></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 反射（__zw_begin_script 内调）+ 派 load → body onload 触发。
    sandbox
        .execute("__zw_begin_script && __zw_begin_script(); __zw_dispatch_event('html','load',null);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hit)").unwrap().value,
        "body-onload",
        "<body onload> 反射为 window.onload，load 派发触发"
    );

    // 另一 on* 类型（onresize）也反射；派 resize 触发。
    sandbox.execute("__zw_dispatch_event('html','resize',null);").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__resize)").unwrap().value,
        "body-resize",
        "<body onresize> 反射为 window.onresize，resize 派发触发"
    );

    // JS 设值优先：window.onload = custom 后再反射（导航新 URL 触发重反射）不应覆盖。
    sandbox
        .execute(
            "globalThis.__hit = null;\
             globalThis.onload = function(){ globalThis.__hit = 'js-onload'; };",
        )
        .unwrap();
    // 模拟导航：换 page URL → 反射去重失效 → 重反射，但 window.onload 已是 JS 设值 → 不覆盖。
    *page_url.lock().unwrap() = "https://example.com/page2".to_string();
    sandbox.execute("__zw_begin_script && __zw_begin_script();").unwrap();
    sandbox.execute("__zw_dispatch_event('html','load',null);").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hit)").unwrap().value,
        "js-onload",
        "JS 设值的 window.onload 优先于 body onload 反射（不被覆盖）"
    );
}

#[test]
fn test_fontface_set_events_r2947() {
    // R2947 CSS Font Loading API：document.fonts FontFaceSet——.ready Promise（settle 时解析）、
    // 'loadingdone'/'loadingerror' 事件（addEventListener + IDL handler）、status、check()。
    // 宿主经 __zw_font_settle(hadLoaded, hadError) 触发（finish_page_load 调用）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // document.fonts 存在 + .ready 是 Promise + status 初始 'loaded'。
    assert_eq!(
        sandbox
            .execute("String(document.fonts instanceof Object)")
            .unwrap()
            .value,
        "true",
        "document.fonts 存在"
    );
    assert_eq!(
        sandbox.execute("String(document.fonts.status)").unwrap().value,
        "loaded",
        "FontFaceSet 初始 status='loaded'"
    );
    assert_eq!(
        sandbox
            .execute("String(document.fonts.check('1em Foo'))")
            .unwrap()
            .value,
        "true",
        "FontFaceSet.check() 返 true"
    );

    // 注册 ready.then + loadingdone/error listener + IDL handler。
    sandbox
        .execute(
            "globalThis.__hit = [];\
             document.fonts.ready.then(function(){ globalThis.__hit.push('ready'); });\
             document.fonts.addEventListener('loadingdone', function(){ globalThis.__hit.push('ael-done'); });\
             document.fonts.onloadingdone = function(){ globalThis.__hit.push('idl-done'); };\
             document.fonts.addEventListener('loadingerror', function(){ globalThis.__hit.push('ael-err'); });",
        )
        .unwrap();

    // settle(true, false)：有成功加载 → loadingdone（ael + IDL）+ ready 解析。
    sandbox.execute("globalThis.__zw_font_settle(true, false);").unwrap();
    let has = |sandbox: &mut V8Sandbox, s: &str| -> bool {
        sandbox
            .execute(&format!("String(globalThis.__hit.indexOf('{s}') >= 0)"))
            .unwrap()
            .value
            == "true"
    };
    assert!(has(&mut sandbox, "ael-done"), "loadingdone addEventListener 触发");
    assert!(has(&mut sandbox, "idl-done"), "onloadingdone IDL 触发");
    assert!(has(&mut sandbox, "ready"), "ready Promise 解析");
    // ready 仅 resolve 一次：再 settle 不重复触发 then。
    sandbox.execute("globalThis.__zw_font_settle(false, false);").unwrap();
    let ready_count = sandbox
        .execute("String(globalThis.__hit.filter(function(x){return x==='ready';}).length)")
        .unwrap()
        .value;
    assert_eq!(
        ready_count, "1",
        "ready Promise 仅 resolve 一次（spec：单 Promise settle 一次）"
    );

    // settle(_, true)：有失败 → loadingerror。
    sandbox.execute("globalThis.__zw_font_settle(false, true);").unwrap();
    assert!(
        has(&mut sandbox, "ael-err"),
        "loadingerror 事件派发（@font-face 加载失败）"
    );
}

#[test]
fn test_location_reflects_pushstate_replacestate_r3005() {
    // R3005：pushState/replaceState 须更新 location（SPA router 读 location.pathname 路由高频）。旧实现 location
    // 读 __zw_get_page_url（host 页面 URL），history 仅 in-memory → location 永不反映 pushState。修复：pushState/
    // replaceState 的 url 经 new URL(rel, base) 解析为绝对（相对当前 location.href）存入 entry；location.href getter
    // 返 _hist_current().url（非空绝对）否则 host 页面 URL。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/start".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "globalThis.__p0 = location.pathname;\
             history.pushState({ p: 1 }, '', '/a'); globalThis.__p1 = location.pathname;\
             history.pushState({ p: 2 }, '', '/b'); globalThis.__p2 = location.pathname;\
             history.back(); globalThis.__pBack = location.pathname;\
             history.replaceState({ p: 3 }, '', '/c'); globalThis.__pRep = location.pathname; globalThis.__sRep = history.state.p;\
             history.pushState({ p: 9 }); globalThis.__pNoUrl = location.pathname;\
             globalThis.__href1 = location.href;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__p0").unwrap().value, "/start", "初始 location.pathname=/start（host 页面 URL）");
    assert_eq!(sandbox.execute("globalThis.__p1").unwrap().value, "/a", "pushState('/a') 后 location.pathname=/a（旧 stale /start）");
    assert_eq!(sandbox.execute("globalThis.__p2").unwrap().value, "/b", "pushState('/b') 后 location.pathname=/b");
    assert_eq!(sandbox.execute("globalThis.__pBack").unwrap().value, "/a", "back() 后 location.pathname=/a");
    assert_eq!(sandbox.execute("globalThis.__pRep").unwrap().value, "/c", "replaceState('/c') 后 location.pathname=/c（替换当前 entry URL）");
    assert_eq!(sandbox.execute("globalThis.__sRep").unwrap().value, "3", "replaceState state.p=3");
    assert_eq!(sandbox.execute("globalThis.__pNoUrl").unwrap().value, "/c", "pushState 无 url 不改 location（保持 /c）");
    assert_eq!(sandbox.execute("globalThis.__href1").unwrap().value, "https://example.com/c", "location.href 绝对 URL");
}

