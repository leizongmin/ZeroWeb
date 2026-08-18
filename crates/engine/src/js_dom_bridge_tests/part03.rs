#[test]
fn test_tag_name_real_not_div_heuristic() {
    // R2691：tagName/nodeName 真实化（旧 _tagFromSel 对 #id 选择器恒猜 DIV）。
    // sel-based：id-bearing 非 DIV 元素返真实 tag；handle-based：detached createElement 返真实 tag。
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
        "<html><body><span id=\"s\">x</span><input id=\"i\"></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // sel-based：#s 是 <span>（旧 stub 错返 DIV），#i 是 <input>。
    sandbox
        .execute("globalThis.__s = document.querySelector('#s').tagName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__s").unwrap().value, "SPAN");
    sandbox
        .execute("globalThis.__i = document.querySelector('#i').tagName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__i").unwrap().value, "INPUT");
    // nodeName 同 tagName（元素节点）。
    sandbox
        .execute("globalThis.__sn = document.querySelector('#s').nodeName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__sn").unwrap().value, "SPAN");
    // 大小写：tagName 在 HTML 命名空间须大写（createElement('svg')→'SVG'）。
    sandbox
        .execute("globalThis.__tr = document.createElement('tr').tagName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__tr").unwrap().value, "TR");
}

#[test]
fn parsed_fragment_elements_expose_geometry_api() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var parsed = new DOMParser().parseFromString('<div><span></span></div>', 'text/html');\
             var element = parsed.body.firstChild.firstChild;\
             globalThis.__parsedGeometry = typeof element.getBoundingClientRect + ',' + element.getBoundingClientRect().width;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__parsedGeometry").unwrap().value,
        "function,0",
        "parsed fragment elements provide a zero-rect geometry fallback"
    );
}

#[test]
fn test_event_bubbling_to_ancestor() {
    // R2692：事件冒泡。旧 dispatchEvent/__zw_dispatch_event 仅派发 target 自身 listener，
    // 不冒泡到祖先——事件委托（document/body 上注册的 listener 捕获子元素事件）失效。
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
        "<html><body><div id=\"p\"><span id=\"c\">x</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 祖先 #p 与 document(html key) 各注册 click listener。
    sandbox
        .execute(
            "document.querySelector('#p').addEventListener('click', function(e){ globalThis.__p = e.currentTarget.id; });",
        )
        .unwrap();
    sandbox
        .execute("document.addEventListener('click', function(){ globalThis.__doc = true; });")
        .unwrap();
    // 在子 #c 上派发 click → 应冒泡到 #p 和 document（html）。
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__p").unwrap().value,
        "p",
        "#p listener 应经冒泡触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__doc").unwrap().value,
        "true",
        "document listener 应经冒泡触发（事件委托）"
    );

    // currentTarget 在 target 阶段 = target 自身。
    sandbox
        .execute(
            "document.querySelector('#c').addEventListener('click', function(e){ globalThis.__ct = e.currentTarget.id; });",
        )
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ct").unwrap().value,
        "c",
        "target 阶段 currentTarget = target"
    );
}

#[test]
fn test_event_bubbling_stop_and_nonbubble() {
    // R2692 续：stopPropagation 中断冒泡；bubbles:false 的事件不冒泡。
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
        "<html><body><div id=\"a\"><div id=\"b\"><i id=\"c\">x</i></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // #b stopPropagation → #a 不应触发。注册顺序：先 #a 后 #b（冒泡 #c→#b→#a）。
    sandbox
        .execute("document.querySelector('#a').addEventListener('click', function(){ globalThis.__a = true; });")
        .unwrap();
    sandbox
        .execute("document.querySelector('#b').addEventListener('click', function(e){ globalThis.__b = true; e.stopPropagation(); });")
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__b === true").unwrap().value,
        "true",
        "#b 应触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__a === true").unwrap().value,
        "false",
        "#a 不应触发（stopPropagation 中断冒泡）"
    );

    // bubbles:false 的事件不冒泡：dispatchEvent 自定义非冒泡事件到 #c，#b 不触发。
    sandbox
        .execute("document.querySelector('#b').addEventListener('foo', function(){ globalThis.__foo = true; });")
        .unwrap();
    sandbox
        .execute("document.querySelector('#c').dispatchEvent(new Event('foo', { bubbles: false }));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__foo === true").unwrap().value,
        "false",
        "bubbles:false 事件不应冒泡到 #b"
    );
}

#[test]
fn test_event_capture_phase() {
    // R2693：capture 阶段。祖先 capture listener（addEventListener 第三参 true）在 root→target
    // 捕获期触发，先于 target（AT_TARGET）与 bubble。旧实现祖先 capture listener 永不触发。
    // 同时验证 legacy 布尔第三参 `addEventListener(t, fn, true)` 注册 capture（_optCapture 修复）。
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
        "<html><body><div id=\"a\"><div id=\"b\"><i id=\"c\">x</i></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // #a capture（legacy 布尔第三参 true）→ #c target（非 capture）→ #b bubble，记录派发顺序。
    sandbox
        .execute(
            "document.querySelector('#a').addEventListener('click', function(e){ globalThis.__order = (globalThis.__order||'') + 'capA:' + e.currentTarget.id + ';'; }, true);",
        )
        .unwrap();
    sandbox
        .execute(
            "document.querySelector('#c').addEventListener('click', function(e){ globalThis.__order += 'tgt:' + e.currentTarget.id + ';'; });",
        )
        .unwrap();
    sandbox
        .execute(
            "document.querySelector('#b').addEventListener('click', function(e){ globalThis.__order += 'bubB:' + e.currentTarget.id + ';'; });",
        )
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    // 捕获期 #a（root 方向）先于 target #c，先于冒泡期 #b。
    assert_eq!(
        sandbox.execute("globalThis.__order").unwrap().value,
        "capA:a;tgt:c;bubB:b;",
        "capture(#a) → target(#c) → bubble(#b) 顺序"
    );
}

#[test]
fn test_event_capture_stop_propagation() {
    // R2693 续：capture 期 stopPropagation → target 与 bubble 阶段不触发。
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
        "<html><body><div id=\"a\"><div id=\"b\"><i id=\"c\">x</i></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // #a capture stopPropagation；#c target；#b bubble。
    sandbox
        .execute("document.querySelector('#a').addEventListener('click', function(e){ globalThis.__cap = true; e.stopPropagation(); }, { capture: true });")
        .unwrap();
    sandbox
        .execute("document.querySelector('#c').addEventListener('click', function(){ globalThis.__tgt = true; });")
        .unwrap();
    sandbox
        .execute("document.querySelector('#b').addEventListener('click', function(){ globalThis.__bub = true; });")
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#c', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cap === true").unwrap().value,
        "true",
        "#a capture 应触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__tgt === true").unwrap().value,
        "false",
        "capture stopPropagation 后 target 不应触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__bub === true").unwrap().value,
        "false",
        "capture stopPropagation 后 bubble 不应触发"
    );
}

#[test]
fn test_event_listener_once() {
    // R2694：`once` 选项。`{once:true}` 注册的 listener 派发一次后自动移除（再次派发不触发）。
    // 旧实现完全忽略 once → listener 重复触发。
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
        "<html><body><button id=\"b\">x</button></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "document.querySelector('#b').addEventListener('click', function(){ globalThis.__n = (globalThis.__n|0) + 1; }, { once: true });",
        )
        .unwrap();
    sandbox.execute("__zw_dispatch_event('#b', 'click', null);").unwrap();
    sandbox.execute("__zw_dispatch_event('#b', 'click', null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__n").unwrap().value,
        "1",
        "once listener 应仅触发一次（第二次派发不触发）"
    );
}

#[test]
fn test_remove_event_listener_capture_aware() {
    // R2694：capture-aware removeEventListener。spec：useCapture 须匹配才移除——
    // `addEventListener(t, fn, true)`（capture）仅 `removeEventListener(t, fn, true)` 能移除；
    // `removeEventListener(t, fn)`（capture=false）不应动 capture 注册。旧实现按 fn 误删。
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
        "<html><body><div id=\"p\"><i id=\"c\">x</i></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // #p 上注册 capture listener（fn），随后 removeEventListener 不带 capture → 不应移除。
    sandbox
        .execute(
            "globalThis.__fn = function(){ globalThis.__cap = (globalThis.__cap|0) + 1; };\n\
             document.querySelector('#p').addEventListener('click', globalThis.__fn, true);\n\
             document.querySelector('#p').removeEventListener('click', globalThis.__fn);\n\
             __zw_dispatch_event('#c', 'click', null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cap").unwrap().value,
        "1",
        "removeEventListener(fn) 不带 capture 不应移除 capture 注册（仍触发）"
    );
    // 现在 removeEventListener 带 capture=true → 应移除，再次派发不触发。
    sandbox
        .execute(
            "document.querySelector('#p').removeEventListener('click', globalThis.__fn, true);\n\
             __zw_dispatch_event('#c', 'click', null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cap").unwrap().value,
        "1",
        "removeEventListener(fn, true) 应移除 capture 注册（再次派发不触发）"
    );
}

#[test]
fn test_style_proxy_methods() {
    // R2695：style 代理 API。getPropertyValue 读初始 style；setProperty 经 SetStyle 应用；
    // per-property get/set 保留；cssText get 读原始串、set 经 SetAttr 整体替换。
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
        "<html><body><div id=\"d\" style=\"color: red\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // getPropertyValue 读初始 style 快照（'color: red' → 'red'）。
    sandbox
        .execute("globalThis.__gv = document.querySelector('#d').style.getPropertyValue('color');")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__gv").unwrap().value,
        "red",
        "getPropertyValue 读初始 color"
    );
    // per-property get 保留（'color' → 'red'）。
    sandbox
        .execute("globalThis.__pg = document.querySelector('#d').style.color;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__pg").unwrap().value,
        "red",
        "per-property style.color 保留"
    );
    // cssText get 读原始串。
    sandbox
        .execute("globalThis.__ct = document.querySelector('#d').style.cssText;")
        .unwrap();
    assert!(
        sandbox.execute("globalThis.__ct").unwrap().value.contains("color: red"),
        "cssText getter 读原始 style 串"
    );

    // setProperty（dashed 名）+ per-property set → 应用后验证序列化。
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.style.setProperty('background-color', 'blue');\n\
             d.style.fontSize = '10px';",
        )
        .unwrap();
    let ms1 = mutations.lock().unwrap().clone();
    let out1 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms1).unwrap();
    assert!(out1.contains("background-color: blue"), "setProperty 应用\n{out1}");
    assert!(
        out1.contains("font-size: 10px"),
        "per-property style.fontSize 须归一为 kebab-case 应用\n{out1}"
    );

    // cssText set → 整体替换（原 color: red 应消失）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute("document.querySelector('#d').style.cssText = 'margin: 0; padding: 5px';")
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    assert!(out2.contains("margin: 0"), "cssText setter 应用 margin\n{out2}");
    assert!(out2.contains("padding: 5px"), "cssText setter 应用 padding\n{out2}");
    assert!(
        !out2.contains("color: red"),
        "cssText setter 应整体替换（原 color 消失）\n{out2}"
    );
}

#[test]
fn test_style_remove_property() {
    // R2695：removeProperty 真移除 style 声明（SetStyle 空值仍 push 'prop: '，不移除）。
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
        "<html><body><div id=\"d\" style=\"color: red; font-size: 10px\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute("document.querySelector('#d').style.removeProperty('color');")
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        !out.contains("color"),
        "removeProperty('color') 应真移除 color 声明\n{out}"
    );
    assert!(
        out.contains("font-size: 10px"),
        "removeProperty 不应影响其他属性\n{out}"
    );
}

#[test]
fn test_style_set_empty_value_removes_r3211() {
    // R3211：`el.style.color = ''`（IDL setter 空值）+ `setProperty('x','')`（空值）应移除声明，
    // 而非留 `prop: ` dangling 空值（spec `dom-cssstyledeclaration-setproperty` + 浏览器一致行为）。
    // `el.style.display = ''` 是 reset inline 样式事实标准高频用法。旧实现 merge_style_property 恒 push
    // 致 dangling（与 setProperty/named-setter 三处同病，本片 host 路径闭合）。
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
        "<html><body><div id=\"d\" style=\"color: red; font-size: 10px\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // IDL setter 空值移除 color；setProperty 空值移除 font-size。
    sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.style.color='';\
             d.style.setProperty('font-size','');",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        !out.contains("color"),
        "el.style.color='' 应移除 color 声明（非 dangling 空值）\n{out}"
    );
    assert!(
        !out.contains("font-size"),
        "setProperty('font-size','') 应移除 font-size 声明\n{out}"
    );
}

#[test]
fn test_style_duplicate_prop_last_wins_r3213() {
    // R3213：duplicate inline prop 末值胜（spec getPropertyValue/getPropertyPriority 返 LAST 声明——
    // CSSOM「get a CSS declaration」末值胜，与 native parse_style dedup 末值胜对称）。polyfill readDeclValue
    // 取末次非空匹配。旧 first-match 返首值（错）。`display:-webkit-flex;display:flex` 等 fallback 模式命中。
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
        "<html><body><div id=\"d\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // setAttribute 注入 duplicate prop（cssText 旁路，直接存原始串）；shim 读 getPropertyValue 取末次。
    let out = sandbox
        .execute(
            "var d=document.querySelector('#d');\
             d.setAttribute('style','color: red; color: blue');\
             d.style.getPropertyValue('color')+'|'+d.style.getPropertyPriority('color');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out.trim(),
        "blue|",
        "duplicate prop 应末值胜（blue），priority 非空 important → ''；got: {out}"
    );

    // 末次为空值 → 回落前一个非空（与 R3212 parse 丢空值对称）：`color:red;color:` → red。
    let out2 = sandbox
        .execute(
            "d.setAttribute('style','color: red; color: ');\
             d.style.getPropertyValue('color');",
        )
        .unwrap()
        .value;
    assert_eq!(
        out2.trim(),
        "red",
        "末次空值应回落前一个非空值（R3212 对称）；got: {out2}"
    );
}

#[test]
fn test_style_camel_to_kebab() {
    // R2696：per-property camelCase style 须归一为 kebab-case 存 style 属性（CSS parser 不认
    // camelCase → 渲染静默失效）。覆盖 backgroundColor / WebkitTransform（vendor 前缀）/ cssFloat
    // （→float）/ per-property camelCase 读 kebab 属性。
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
        "<html><body><div id=\"d\" style=\"font-size: 10px\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // per-property camelCase 读 kebab 属性（font-size → fontSize 读出 '10px'）。
    sandbox
        .execute("globalThis.__fs = document.querySelector('#d').style.fontSize;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__fs").unwrap().value,
        "10px",
        "camelCase 读 kebab 属性"
    );

    // camelCase set → kebab 存储（不残留 camelCase）。
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.style.backgroundColor = 'red';\n\
             d.style.WebkitTransform = 'scale(2)';\n\
             d.style.cssFloat = 'left';",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        out.contains("background-color: red"),
        "backgroundColor → background-color\n{out}"
    );
    assert!(
        !out.contains("backgroundColor"),
        "不应残留 camelCase backgroundColor\n{out}"
    );
    assert!(
        out.contains("-webkit-transform: scale(2)"),
        "WebkitTransform → -webkit-transform\n{out}"
    );
    assert!(out.contains("float: left"), "cssFloat → float\n{out}");
    assert!(!out.contains("cssFloat"), "不应残留 cssFloat\n{out}");
}

#[test]
fn test_classlist_consecutive_ops() {
    // R2697：classList 连续操作不丢类。旧实现每次读 stale snapshot + SetAttr 整体替换，
    // 同脚本 add('a');add('b');add('c') 仅保留末个（base c）。客户端缓存累积全量后修复。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mk = || -> (V8Sandbox, Arc<Mutex<Vec<DomMutation>>>, Arc<Mutex<String>>) {
        let mut sb = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sb.execute(generate_js_dom_shim()).unwrap();
        let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
        let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
            "<html><body><div id=\"d\" class=\"base\"></div></body></html>".to_string(),
        ));
        let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
        let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sb, &mutations, &dom_html, &page_url, &canvas_registry);
        (sb, mutations, dom_html)
    };

    // ① 连续 add 三类 → apply 后 class 含 base/a/b/c 全部（旧实现仅 'base c'）。
    let (mut sandbox, mutations, dom_html) = mk();
    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.classList.add('a');\n\
             d.classList.add('b');\n\
             d.classList.add('c');",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    let class_val: String = out
        .split("class=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap_or("")
        .to_string();
    for cls in ["base", "a", "b", "c"] {
        assert!(
            class_val.split_whitespace().any(|t| t == cls),
            "class 应含 {cls}（got '{class_val}'）\n{out}"
        );
    }

    // ② className set + classList add 协作（className 写缓存、classList 读缓存累加）。
    let (mut sandbox, mutations, dom_html) = mk();
    sandbox
        .execute(
            "var e = document.querySelector('#d');\n\
             e.className = 'x';\n\
             e.classList.add('y');",
        )
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    assert!(
        out2.contains("class=\"x y\""),
        "className=x 后 classList.add(y) → 'x y'\n{out2}"
    );

    // ③ toggle 首次加（true）/ contains 反映 / 二次移除（false），双 toggle 后 on 消失。
    let (mut sandbox, mutations, dom_html) = mk();
    sandbox
        .execute(
            "globalThis.__t1 = document.querySelector('#d').classList.toggle('on');\n\
             globalThis.__has = document.querySelector('#d').classList.contains('on');\n\
             globalThis.__t2 = document.querySelector('#d').classList.toggle('on');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__t1").unwrap().value,
        "true",
        "toggle 首次加返 true"
    );
    assert_eq!(
        sandbox.execute("globalThis.__has").unwrap().value,
        "true",
        "toggle 后 contains(on) 反映缓存"
    );
    assert_eq!(
        sandbox.execute("globalThis.__t2").unwrap().value,
        "false",
        "toggle 二次移除返 false"
    );
    let ms3 = mutations.lock().unwrap().clone();
    let out3 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms3).unwrap();
    let class_val3: String = out3
        .split("class=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .unwrap_or("")
        .to_string();
    assert!(
        !class_val3.split_whitespace().any(|t| t == "on"),
        "双 toggle 后 on 应移除（got '{class_val3}'）\n{out3}"
    );
}

#[test]
fn test_remove_attribute_truly_removes() {
    // R2698：removeAttribute 真移除。旧 set-empty 残留 `checked=""`（present）→ el.checked 仍 true。
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
        "<html><body><input id=\"i\" checked></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute("document.querySelector('#i').removeAttribute('checked');")
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(
        !out.contains("checked"),
        "removeAttribute('checked') 应真移除（不残留 checked=\"\"）\n{out}"
    );
}

#[test]
fn test_attribute_query_api() {
    // R2698：hasAttribute/hasAttributes/getAttributeNames/toggleAttribute。
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
        "<html><body><input id=\"i\" type=\"text\" disabled></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // hasAttribute（present/absent）。
    sandbox
        .execute(
            "globalThis.__hd = document.querySelector('#i').hasAttribute('disabled');\n\
             globalThis.__hid = document.querySelector('#i').hasAttribute('id');\n\
             globalThis.__no = document.querySelector('#i').hasAttribute('checked');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hd").unwrap().value,
        "true",
        "hasAttribute(disabled)"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hid").unwrap().value,
        "true",
        "hasAttribute(id)"
    );
    assert_eq!(
        sandbox.execute("globalThis.__no").unwrap().value,
        "false",
        "hasAttribute(checked) absent"
    );

    // hasAttributes + getAttributeNames。
    sandbox
        .execute(
            "globalThis.__hs = document.querySelector('#i').hasAttributes();\n\
             globalThis.__names = document.querySelector('#i').getAttributeNames().join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hs").unwrap().value,
        "true",
        "hasAttributes"
    );
    assert_eq!(
        sandbox.execute("globalThis.__names").unwrap().value,
        "id,type,disabled",
        "getAttributeNames 顺序"
    );
}

#[test]
fn test_toggle_attribute() {
    // R2701：toggleAttribute 经 server-side mutation（apply 时决策），连续 toggle 正确复合。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id=\"d\"></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 单次 toggle 加 → 返 true；force=false 移除（即便刚加，server-side 不受 stale 影响）。
    sandbox
        .execute(
            "globalThis.__r1 = document.querySelector('#d').toggleAttribute('hidden');\n\
             document.querySelector('#d').toggleAttribute('hidden', false);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "true",
        "toggle 加返 true"
    );
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    // toggle(hidden) 加 → SetAttr(want=true)（R3192 enqueue-时解析）；toggle(hidden,false) → RemoveAttr。
    // apply 顺序：先加 hidden，再移除 → net 无 hidden。
    assert!(!out.contains("hidden"), "force=false 应移除（net 无 hidden）\n{out}");

    // 连续双 toggle（无 force）：朴素实现都读 stale 都加 → 残留；enqueue-时解析正确复合 → net 移除。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "document.querySelector('#d').toggleAttribute('x');\n\
             document.querySelector('#d').toggleAttribute('x');",
        )
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    // 两次 toggle(x)：apply 时第一次无 x→加，第二次有 x→移除 → net 无 x（朴素实现都加会残留 x）。
    assert!(
        !out2.contains("x"),
        "连续双 toggle(x) server-side 决策 → net 移除（无 x）\n{out2}"
    );

    // force=true 强加（即便存在也保留）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute("document.querySelector('#d').toggleAttribute('aria-label', true);")
        .unwrap();
    let ms3 = mutations.lock().unwrap().clone();
    let out3 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms3).unwrap();
    assert!(out3.contains("aria-label"), "force=true 强加 aria-label\n{out3}");
}

#[test]
fn test_get_computed_style_display_position_visibility_opacity() {
    // R2704：getComputedStyle 计算值（首批 display/position/visibility/opacity）。旧全属性返 '' →
    // visibility/hidden 分支全断。现经 __zw_get_computed_style 返真实计算值（UA builtin + <style>）。
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
         <div id=\"d\"></div>\
         <span id=\"s\" style=\"display:none\"></span>\
         <style>#d { position: relative; opacity: 0.5 }</style>\
         <p id=\"hid\" style=\"visibility:hidden\"></p>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // div：UA display=block；<style> 设 position=relative、opacity=0.5。
    sandbox
        .execute(
            "globalThis.__dd = getComputedStyle(document.querySelector('#d')).display;\n\
             globalThis.__dp = getComputedStyle(document.querySelector('#d')).position;\n\
             globalThis.__do = getComputedStyle(document.querySelector('#d')).opacity;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dd").unwrap().value,
        "block",
        "div UA display=block"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dp").unwrap().value,
        "relative",
        "<style> position=relative"
    );
    assert_eq!(
        sandbox.execute("globalThis.__do").unwrap().value,
        "0.5",
        "<style> opacity=0.5"
    );
    // span inline display:none；getPropertyValue(kebab) 路径。
    sandbox
        .execute("globalThis.__sd = getComputedStyle(document.querySelector('#s')).getPropertyValue('display');")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__sd").unwrap().value,
        "none",
        "inline style display:none（getPropertyValue kebab 路径）"
    );
    // p inline visibility:hidden。
    sandbox
        .execute("globalThis.__pv = getComputedStyle(document.querySelector('#hid')).visibility;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__pv").unwrap().value,
        "hidden",
        "inline visibility:hidden"
    );
}

#[test]
fn test_get_computed_style_colors() {
    // R2705：getComputedStyle 颜色族（color/background-color/border-*-color）。compute_styles 保留
    // 颜色未解析（Named/CurrentColor），经 paint 层 resolve_color_current 解析为 rgb/rgba 串。
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
         <div id=\"a\" style=\"color: red; background-color: rgb(0, 128, 0)\"></div>\
         <div id=\"b\" style=\"border: 1px solid blue\"></div>\
         <div id=\"c\" style=\"color: transparent\"></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // color: red（named → rgb）+ background-color: rgb(0,128,0)。
    sandbox
        .execute(
            "globalThis.__col = getComputedStyle(document.querySelector('#a')).color;\n\
             globalThis.__bg = getComputedStyle(document.querySelector('#a')).backgroundColor;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__col").unwrap().value,
        "rgb(255, 0, 0)",
        "color: red → rgb(255,0,0)"
    );
    assert_eq!(
        sandbox.execute("globalThis.__bg").unwrap().value,
        "rgb(0, 128, 0)",
        "background-color: rgb(0,128,0)"
    );
    // border: 1px solid blue → border-color (4 边) = rgb(0,0,255)。
    sandbox
        .execute(
            "globalThis.__bt = getComputedStyle(document.querySelector('#b')).getPropertyValue('border-top-color');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__bt").unwrap().value,
        "rgb(0, 0, 255)",
        "border shorthand 的 blue → border-top-color rgb(0,0,255)"
    );
    // color: transparent → rgba(0,0,0,0)。
    sandbox
        .execute("globalThis.__tc = getComputedStyle(document.querySelector('#c')).color;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__tc").unwrap().value,
        "rgba(0, 0, 0, 0)",
        "color: transparent → rgba(0,0,0,0)"
    );
}

#[test]
fn test_computed_style_cache_reuse_composition() {
    // R2706：getComputedStyle per-snapshot 缓存 = compute_document_styles（一次）+
    // lookup_computed_property（多次）。验证「build (doc, styles) once → query N 属性」与无缓存
    // computed_style_property 逐次等价——锁缓存命中路径返回值正确（缓存复用不改变结果）。
    let html = "<html><body>\
        <div id=\"d\" style=\"color: red; display: none; opacity: 0.25\"></div>\
        <style>#d { position: relative }</style>\
        </body></html>";
    let (doc, styles) = compute_document_styles(html);
    // 同一 (doc, styles) 连续查 4 个属性（缓存命中场景）。
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "color"), "rgb(255, 0, 0)");
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "display"), "none");
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "opacity"), "0.25");
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "position"), "relative");
    // 与无缓存参考实现逐属性等价。
    assert_eq!(computed_style_property(html, "#d", "color"), "rgb(255, 0, 0)");
    assert_eq!(computed_style_property(html, "#d", "display"), "none");
    assert_eq!(computed_style_property(html, "#d", "position"), "relative");
    // 未命中选择器 → ''；margin-top R2707 起已覆盖（长度族）→ div 默认 0px。
    assert_eq!(lookup_computed_property(&doc, &styles, "#missing", "color"), "");
    assert_eq!(lookup_computed_property(&doc, &styles, "#d", "margin-top"), "0px");
}

#[test]
fn test_get_computed_style_cache_invalidation() {
    // R2706：getComputedStyle per-snapshot 缓存失效（核心正确性风险）。同一会话内首查填缓存后
    // 改 dom_html snapshot，再查须反映新 html（不返 stale 缓存值）。缓存 keyed on html。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body><div id=\"d\"></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 首查：div UA display=block（填缓存）。
    sandbox
        .execute("globalThis.__v1 = getComputedStyle(document.querySelector('#d')).display;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__v1").unwrap().value, "block");

    // 改 snapshot：注入 <style>#d{display:none}</style>。缓存 keyed on html → 失效 → 重算。
    *dom_html.lock().unwrap() = "<html><body><div id=\"d\"></div>\
        <style>#d { display: none }</style></body></html>"
        .to_string();
    sandbox
        .execute("globalThis.__v2 = getComputedStyle(document.querySelector('#d')).display;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__v2").unwrap().value,
        "none",
        "html snapshot 变 → 缓存失效重算，返新 display=none（非 stale 的 block）"
    );
}

#[test]
fn test_get_computed_style_lengths() {
    // R2707：getComputedStyle 长度族（width/height/min-max/margin/padding/border-width/
    // border-radius/outline-width/font-size/gap/letter-spacing/text-indent 等）。compute_styles
    // 已把相对单位解析为 Px，故 px 指定值精确；百分比/auto 保留（无 layout 不解析为 used 值）。
    // border-width 在 style:none 时返 "0px"（used=0）；outline-width 则保留 computed medium→3px（R2754）；max-*:none → "none"。
    let html = "<html><body>\
        <div id=\"box\" style=\"\
            width: 100px; height: 50%; \
            margin-top: 10px; margin-right: 20px; margin-bottom: 10px; margin-left: 20px; \
            padding: 5px; \
            border-top-width: 3px; border-top-style: solid; \
            border-top-left-radius: 8px; \
            outline-width: 2px; outline-style: solid; \
            max-width: 500px; min-width: auto; \
            font-size: 20px; \
            gap: 12px; letter-spacing: 0.1em; \
        \"></div>\
        <div id=\"plain\"></div>\
        </body></html>";

    // px 指定 → 精确（Chrome 一致）。
    assert_eq!(computed_style_property(html, "#box", "width"), "100px");
    assert_eq!(computed_style_property(html, "#box", "margin-top"), "10px");
    assert_eq!(computed_style_property(html, "#box", "margin-right"), "20px");
    assert_eq!(computed_style_property(html, "#box", "margin-bottom"), "10px");
    assert_eq!(computed_style_property(html, "#box", "margin-left"), "20px");
    assert_eq!(computed_style_property(html, "#box", "padding-top"), "5px");
    assert_eq!(computed_style_property(html, "#box", "padding-left"), "5px");
    // 百分比 → 保留（计算值，无 layout 不解析 used 值）。
    assert_eq!(computed_style_property(html, "#box", "height"), "50%");
    // em → 解析为 px（letter-spacing 0.1em @ font-size 20px = 2px）。
    assert_eq!(computed_style_property(html, "#box", "letter-spacing"), "2px");
    assert_eq!(computed_style_property(html, "#box", "font-size"), "20px");
    assert_eq!(computed_style_property(html, "#box", "gap"), "12px");
    // border-width：style=solid → 真宽；border-radius px。
    assert_eq!(computed_style_property(html, "#box", "border-top-width"), "3px");
    assert_eq!(computed_style_property(html, "#box", "border-top-left-radius"), "8px");
    // outline-width：style=solid → 真宽。
    assert_eq!(computed_style_property(html, "#box", "outline-width"), "2px");
    // max-width 指定 → px；min-width auto → "auto"。
    assert_eq!(computed_style_property(html, "#box", "max-width"), "500px");
    assert_eq!(computed_style_property(html, "#box", "min-width"), "auto");

    // 默认 div（无 border）：border-width 返 "0px"（border-style:none → used=0，对齐 Chromium）。
    assert_eq!(computed_style_property(html, "#plain", "border-top-width"), "0px");
    // R2754：outline-width 不套 border 的 none→0 规则——outline-style:none 时 outline-width 仍保留
    // computed 值（medium→3px），Chromium getComputedStyle 返 "3px"（与 border-width 行为不同）。
    assert_eq!(computed_style_property(html, "#plain", "outline-width"), "3px");
    // 默认 max-width/max-height:none → "none"；默认 margin:0 → "0px"；默认 width:auto → "auto"。
    assert_eq!(computed_style_property(html, "#plain", "max-width"), "none");
    assert_eq!(computed_style_property(html, "#plain", "max-height"), "none");
    assert_eq!(computed_style_property(html, "#plain", "margin-top"), "0px");
    assert_eq!(computed_style_property(html, "#plain", "width"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "font-size"), "16px");
}

#[test]
fn test_get_computed_style_keywords() {
    // R2708：getComputedStyle 关键字/枚举族（float/clear/box-sizing/overflow/text-align/
    // white-space/font-weight/font-style/line-height/z-index/cursor/text-transform/text-overflow/
    // direction/border-collapse/table-layout/caption-side/border-*-style/outline-style）。
    let html = "<html><body>\
        <div id=\"k\" style=\"\
            float: left; clear: both; box-sizing: border-box; \
            overflow: hidden; text-align: center; white-space: pre-wrap; \
            font-weight: bold; font-style: italic; line-height: 1.5; \
            z-index: 10; cursor: pointer; text-transform: uppercase; \
            text-overflow: ellipsis; direction: rtl; \
            border: 2px dashed red; outline: 3px dotted blue; \
        \"></div>\
        <table id=\"t\" style=\"border-collapse: collapse; table-layout: fixed;\
            \"><caption id=\"cap\"></caption><tr><td></td></tr></table>\
        <div id=\"plain\"></div>\
        </body></html>";

    // 显式设置的关键字直映。
    assert_eq!(computed_style_property(html, "#k", "float"), "left");
    assert_eq!(computed_style_property(html, "#k", "clear"), "both");
    assert_eq!(computed_style_property(html, "#k", "box-sizing"), "border-box");
    assert_eq!(computed_style_property(html, "#k", "overflow-x"), "hidden");
    assert_eq!(computed_style_property(html, "#k", "overflow-y"), "hidden");
    assert_eq!(computed_style_property(html, "#k", "text-align"), "center");
    assert_eq!(computed_style_property(html, "#k", "white-space"), "pre-wrap");
    // font-weight bold→700（对齐 Chromium 绝对值）、font-style italic、line-height number→used px
    // （1.5 × 默认 font-size 16px = 24px，R2761 对齐 Chromium getComputedStyle used 值）。
    assert_eq!(computed_style_property(html, "#k", "font-weight"), "700");
    assert_eq!(computed_style_property(html, "#k", "font-style"), "italic");
    assert_eq!(computed_style_property(html, "#k", "line-height"), "24px");
    assert_eq!(computed_style_property(html, "#k", "z-index"), "10");
    assert_eq!(computed_style_property(html, "#k", "cursor"), "pointer");
    assert_eq!(computed_style_property(html, "#k", "text-transform"), "uppercase");
    assert_eq!(computed_style_property(html, "#k", "text-overflow"), "ellipsis");
    assert_eq!(computed_style_property(html, "#k", "direction"), "rtl");
    // border/outline shorthand → longhand style。
    assert_eq!(computed_style_property(html, "#k", "border-top-style"), "dashed");
    assert_eq!(computed_style_property(html, "#k", "outline-style"), "dotted");
    // 表格属性。
    assert_eq!(computed_style_property(html, "#t", "border-collapse"), "collapse");
    assert_eq!(computed_style_property(html, "#t", "table-layout"), "fixed");

    // 默认值（initial 关键字）——验证关键字族 fallback 正确。
    assert_eq!(computed_style_property(html, "#plain", "float"), "none");
    assert_eq!(computed_style_property(html, "#plain", "box-sizing"), "content-box");
    assert_eq!(computed_style_property(html, "#plain", "overflow-x"), "visible");
    assert_eq!(computed_style_property(html, "#plain", "text-align"), "start");
    assert_eq!(computed_style_property(html, "#plain", "white-space"), "normal");
    assert_eq!(computed_style_property(html, "#plain", "font-weight"), "400");
    assert_eq!(computed_style_property(html, "#plain", "font-style"), "normal");
    assert_eq!(computed_style_property(html, "#plain", "line-height"), "normal");
    assert_eq!(computed_style_property(html, "#plain", "z-index"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "cursor"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "text-transform"), "none");
    assert_eq!(computed_style_property(html, "#plain", "text-overflow"), "clip");
    assert_eq!(computed_style_property(html, "#plain", "direction"), "ltr");
    assert_eq!(computed_style_property(html, "#plain", "border-top-style"), "none");
    assert_eq!(computed_style_property(html, "#plain", "outline-style"), "none");
}

#[test]
fn test_get_computed_style_composite() {
    // R2710：getComputedStyle 复合/列表族（font-family/flex-*/justify-content/align-*
    // /writing-mode/object-fit/isolation/mix-blend-mode/pointer-events/user-select/list-style-*）。
    let html = "<html><body>\
        <div id=\"c\" style=\"\
            font-family: 'Helvetica Neue', Arial, sans-serif; \
            flex-direction: column; flex-wrap: wrap; \
            justify-content: space-between; align-items: center; align-self: flex-end; \
            writing-mode: vertical-rl; object-fit: cover; isolation: isolate; \
            mix-blend-mode: multiply; pointer-events: none; user-select: all; \
        \"></div>\
        <ul id=\"l\" style=\"list-style-type: lower-alpha; list-style-position: inside;\
            \"><li></li></ul>\
        <div id=\"plain\"></div>\
        </body></html>";

    // font-family：逗号分隔，带空格的族名加引号，简单 ident（Arial/sans-serif）不引号。
    assert_eq!(
        computed_style_property(html, "#c", "font-family"),
        "\"Helvetica Neue\", Arial, sans-serif"
    );
    // flex / alignment / writing-mode / object-fit / 隔离·混合·交互。
    assert_eq!(computed_style_property(html, "#c", "flex-direction"), "column");
    assert_eq!(computed_style_property(html, "#c", "flex-wrap"), "wrap");
    assert_eq!(computed_style_property(html, "#c", "justify-content"), "space-between");
    assert_eq!(computed_style_property(html, "#c", "align-items"), "center");
    assert_eq!(computed_style_property(html, "#c", "align-self"), "flex-end");
    assert_eq!(computed_style_property(html, "#c", "writing-mode"), "vertical-rl");
    assert_eq!(computed_style_property(html, "#c", "object-fit"), "cover");
    assert_eq!(computed_style_property(html, "#c", "isolation"), "isolate");
    assert_eq!(computed_style_property(html, "#c", "mix-blend-mode"), "multiply");
    assert_eq!(computed_style_property(html, "#c", "pointer-events"), "none");
    assert_eq!(computed_style_property(html, "#c", "user-select"), "all");
    // list-style。
    assert_eq!(computed_style_property(html, "#l", "list-style-type"), "lower-alpha");
    assert_eq!(computed_style_property(html, "#l", "list-style-position"), "inside");

    // 默认值（ZeroWeb initial：justify-content=flex-start、align-items=stretch、align-self=auto；
    // 注：Chromium Box Align 3 initial 为 normal，ZeroWeb default 取 flex-start/stretch，diverge）。
    assert_eq!(computed_style_property(html, "#plain", "flex-direction"), "row");
    assert_eq!(computed_style_property(html, "#plain", "flex-wrap"), "nowrap");
    assert_eq!(computed_style_property(html, "#plain", "justify-content"), "flex-start");
    assert_eq!(computed_style_property(html, "#plain", "align-items"), "stretch");
    assert_eq!(computed_style_property(html, "#plain", "align-self"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "writing-mode"), "horizontal-tb");
    assert_eq!(computed_style_property(html, "#plain", "object-fit"), "fill");
    assert_eq!(computed_style_property(html, "#plain", "isolation"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "mix-blend-mode"), "normal");
    assert_eq!(computed_style_property(html, "#plain", "pointer-events"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "user-select"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "list-style-type"), "disc");
    assert_eq!(
        computed_style_property(html, "#plain", "list-style-position"),
        "outside"
    );
}

#[test]
fn test_get_computed_style_numeric_special() {
    // R2711：getComputedStyle 数值/special 族（flex-grow/flex-shrink/order/flex-basis/aspect-ratio）。
    let html = "<html><body>\
        <div id=\"n\" style=\"\
            flex-grow: 2.5; flex-shrink: 0; order: 3; \
            flex-basis: 120px; aspect-ratio: 16 / 9; \
        \"></div>\
        <div id=\"plain\"></div>\
        </body></html>";

    // 显式数值/special。
    assert_eq!(computed_style_property(html, "#n", "flex-grow"), "2.5");
    assert_eq!(computed_style_property(html, "#n", "flex-shrink"), "0");
    assert_eq!(computed_style_property(html, "#n", "order"), "3");
    assert_eq!(computed_style_property(html, "#n", "flex-basis"), "120px");
    // aspect-ratio: ZeroWeb 只存合并比值 → 数值（Chrome 返 "16 / 9"，diverge，已记 known-limitation）。
    assert_eq!(computed_style_property(html, "#n", "aspect-ratio"), "1.778");

    // 默认值（ZeroWeb initial：flex-grow=0、flex-shrink=1、order=0、flex-basis=auto、aspect-ratio=auto）。
    assert_eq!(computed_style_property(html, "#plain", "flex-grow"), "0");
    assert_eq!(computed_style_property(html, "#plain", "flex-shrink"), "1");
    assert_eq!(computed_style_property(html, "#plain", "order"), "0");
    assert_eq!(computed_style_property(html, "#plain", "flex-basis"), "auto");
    assert_eq!(computed_style_property(html, "#plain", "aspect-ratio"), "auto");
}

#[test]
fn test_get_computed_style_transform() {
    // R2715：getComputedStyle transform 序列化（CSS Transforms L1/L2 计算值 = 函数列表）。
    // Chromium 返 resolved matrix（diverge）；ZeroWeb 返 parsed 函数列表（spec-correct + Firefox 一致）。
    let html = "<html><body>\
        <div id=\"t\" style=\"transform: translate(10px, 20px) rotate(45deg) scale(2);\"></div>\
        <div id=\"pct\" style=\"transform: translateX(50%);\"></div>\
        <div id=\"none\"></div>\
        </body></html>";
    // 组合：translate + rotate + scale（空格分隔函数列表）。
    assert_eq!(
        computed_style_property(html, "#t", "transform"),
        "translate(10px, 20px) rotate(45deg) scale(2)"
    );
    // 百分比 translate 保留（border-box 相对，须 layout 故保 %）。
    assert_eq!(computed_style_property(html, "#pct", "transform"), "translateX(50%)");
    // 默认 none。
    assert_eq!(computed_style_property(html, "#none", "transform"), "none");
}

#[test]
fn test_get_computed_style_transform_origin() {
    // R2716：getComputedStyle transform-origin 序列化（2 LengthValue，空格连接）。
    // Chromium 返 used 值（border-box 中心绝对 px，diverge）；ZeroWeb 返计算值（spec-correct + Firefox 一致）。
    let html = "<html><body>\
        <div id=\"px\" style=\"transform-origin: 10px 20px;\"></div>\
        <div id=\"pct\" style=\"transform-origin: 25% 75%;\"></div>\
        <div id=\"center\" style=\"transform-origin: center;\"></div>\
        <div id=\"single\" style=\"transform-origin: 0px;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // 显式 px（computed == used，与 real browser 一致）。
    assert_eq!(computed_style_property(html, "#px", "transform-origin"), "10px 20px");
    // 显式百分比保留为计算值（Chromium 返 used px，diverge）。
    assert_eq!(computed_style_property(html, "#pct", "transform-origin"), "25% 75%");
    // 关键字 center 计算值 = 50% 50%（apply 未解析关键字降级为默认，恰等于 center 计算值，行为正确）。
    assert_eq!(computed_style_property(html, "#center", "transform-origin"), "50% 50%");
    // 单值：x 指定，y 默认 50%。
    assert_eq!(computed_style_property(html, "#single", "transform-origin"), "0px 50%");
    // 默认值 50% 50%。
    assert_eq!(computed_style_property(html, "#def", "transform-origin"), "50% 50%");
}

#[test]
fn test_get_computed_style_contain() {
    // R2717：getComputedStyle contain 序列化（CSS Containment L1/L2 计算值）。
    // Strict/Content 保留 shorthand 不展开（与 Chromium 一致）；组合值按 spec 语法序 size/layout/paint/style。
    let html = "<html><body>\
        <div id=\"none\" style=\"contain: none;\"></div>\
        <div id=\"strict\" style=\"contain: strict;\"></div>\
        <div id=\"content\" style=\"contain: content;\"></div>\
        <div id=\"single\" style=\"contain: layout;\"></div>\
        <div id=\"combo\" style=\"contain: layout paint;\"></div>\
        <div id=\"size-style\" style=\"contain: size style;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // none（默认）。
    assert_eq!(computed_style_property(html, "#none", "contain"), "none");
    // shorthand 保留。
    assert_eq!(computed_style_property(html, "#strict", "contain"), "strict");
    assert_eq!(computed_style_property(html, "#content", "contain"), "content");
    // 单关键字。
    assert_eq!(computed_style_property(html, "#single", "contain"), "layout");
    // 组合：位掩码解码，spec 语法序（layout paint）。
    assert_eq!(computed_style_property(html, "#combo", "contain"), "layout paint");
    // 组合：size + style（非连续位）按语法序 size 在前。
    assert_eq!(computed_style_property(html, "#size-style", "contain"), "size style");
    // 默认 none。
    assert_eq!(computed_style_property(html, "#def", "contain"), "none");
}

#[test]
fn test_get_computed_style_filter() {
    // R2718：getComputedStyle filter 序列化（CSS Filter Effects 函数列表，空格分隔）。
    let html = "<html><body>\
        <div id=\"none\" style=\"filter: none;\"></div>\
        <div id=\"blur\" style=\"filter: blur(5px);\"></div>\
        <div id=\"combo\" style=\"filter: brightness(1.5) contrast(0.8);\"></div>\
        <div id=\"hue\" style=\"filter: hue-rotate(90deg);\"></div>\
        <div id=\"shadow\" style=\"filter: drop-shadow(2px 4px 6px red);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // none（显式与默认均为空 Vec）。
    assert_eq!(computed_style_property(html, "#none", "filter"), "none");
    // 单函数：blur 长度为 px。
    assert_eq!(computed_style_property(html, "#blur", "filter"), "blur(5px)");
    // 多函数组合：空格分隔，数值函数无单位。
    assert_eq!(
        computed_style_property(html, "#combo", "filter"),
        "brightness(1.5) contrast(0.8)"
    );
    // hue-rotate 角度为 deg。
    assert_eq!(computed_style_property(html, "#hue", "filter"), "hue-rotate(90deg)");
    // drop-shadow：3 长度 px + 颜色解析为 rgb()。
    assert_eq!(
        computed_style_property(html, "#shadow", "filter"),
        "drop-shadow(2px 4px 6px rgb(255, 0, 0))"
    );
    // 默认 none。
    assert_eq!(computed_style_property(html, "#def", "filter"), "none");
}

#[test]
fn test_get_computed_style_transform_family() {
    // R2719：getComputedStyle 3D transform 簇（transform-style / backface-visibility / perspective /
    // perspective-origin，完成 R2715/R2716 启动的 transform 簇）。
    let html = "<html><body>\
        <div id=\"ts-3d\" style=\"transform-style: preserve-3d;\"></div>\
        <div id=\"bv-hidden\" style=\"backface-visibility: hidden;\"></div>\
        <div id=\"persp\" style=\"perspective: 800px;\"></div>\
        <div id=\"po\" style=\"perspective-origin: 25% 75%;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // transform-style：默认 flat，显式 preserve-3d。
    assert_eq!(
        computed_style_property(html, "#ts-3d", "transform-style"),
        "preserve-3d"
    );
    assert_eq!(computed_style_property(html, "#def", "transform-style"), "flat");
    // backface-visibility：默认 visible，显式 hidden。
    assert_eq!(
        computed_style_property(html, "#bv-hidden", "backface-visibility"),
        "hidden"
    );
    assert_eq!(computed_style_property(html, "#def", "backface-visibility"), "visible");
    // perspective：默认 none（Px(0.0)），显式 px。
    assert_eq!(computed_style_property(html, "#persp", "perspective"), "800px");
    assert_eq!(computed_style_property(html, "#def", "perspective"), "none");
    // perspective-origin：默认 50% 50%，显式百分比保留。
    assert_eq!(computed_style_property(html, "#po", "perspective-origin"), "25% 75%");
    assert_eq!(computed_style_property(html, "#def", "perspective-origin"), "50% 50%");
}

#[test]
fn test_get_computed_style_will_change() {
    // R2720：getComputedStyle will-change 序列化（CSS Will Change 列表，perf hint 常查）。
    let html = "<html><body>\
        <div id=\"auto\" style=\"will-change: auto;\"></div>\
        <div id=\"scroll\" style=\"will-change: scroll-position;\"></div>\
        <div id=\"contents\" style=\"will-change: contents;\"></div>\
        <div id=\"custom\" style=\"will-change: transform;\"></div>\
        <div id=\"multi\" style=\"will-change: transform opacity;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // auto（显式与默认均为空 Vec）。
    assert_eq!(computed_style_property(html, "#auto", "will-change"), "auto");
    assert_eq!(computed_style_property(html, "#def", "will-change"), "auto");
    // 关键字标识符。
    assert_eq!(
        computed_style_property(html, "#scroll", "will-change"),
        "scroll-position"
    );
    assert_eq!(computed_style_property(html, "#contents", "will-change"), "contents");
    // 自定义属性名原样。
    assert_eq!(computed_style_property(html, "#custom", "will-change"), "transform");
    // 多属性组合：空格分隔。
    assert_eq!(
        computed_style_property(html, "#multi", "will-change"),
        "transform opacity"
    );
}

#[test]
fn test_get_computed_style_clip_path() {
    // R2721：getComputedStyle clip-path 序列化（CSS Masking basic-shape 函数）。
    let html = "<html><body>\
        <div id=\"none\" style=\"clip-path: none;\"></div>\
        <div id=\"inset1\" style=\"clip-path: inset(10%);\"></div>\
        <div id=\"inset2\" style=\"clip-path: inset(10% 20%);\"></div>\
        <div id=\"inset-round\" style=\"clip-path: inset(5px round 10px);\"></div>\
        <div id=\"circle\" style=\"clip-path: circle(50px at 25% 75%);\"></div>\
        <div id=\"circle-def\" style=\"clip-path: circle();\"></div>\
        <div id=\"polygon\" style=\"clip-path: polygon(0% 0%, 100% 0%, 50% 100%);\"></div>\
        <div id=\"polygon-ee\" style=\"clip-path: polygon(evenodd, 0% 0%, 100% 0%, 50% 100%);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // none。
    assert_eq!(computed_style_property(html, "#none", "clip-path"), "none");
    assert_eq!(computed_style_property(html, "#def", "clip-path"), "none");
    // inset 单值折叠（解析展开 4 值全等 → 重新折叠为 1 值）。
    assert_eq!(computed_style_property(html, "#inset1", "clip-path"), "inset(10%)");
    // inset 双值（top==bottom, left==right）。
    assert_eq!(computed_style_property(html, "#inset2", "clip-path"), "inset(10% 20%)");
    // inset + round（圆角半径）。
    assert_eq!(
        computed_style_property(html, "#inset-round", "clip-path"),
        "inset(5px round 10px)"
    );
    // circle 半径 + at 位置。
    assert_eq!(
        computed_style_property(html, "#circle", "clip-path"),
        "circle(50px at 25% 75%)"
    );
    // circle() 空（默认 closest-side，无位置）。
    assert_eq!(
        computed_style_property(html, "#circle-def", "clip-path"),
        "circle(closest-side)"
    );
    // polygon 默认 nonzero 省略填充规则，顶点逗号分隔。
    assert_eq!(
        computed_style_property(html, "#polygon", "clip-path"),
        "polygon(0% 0%, 100% 0%, 50% 100%)"
    );
    // polygon evenodd 输出填充规则。
    assert_eq!(
        computed_style_property(html, "#polygon-ee", "clip-path"),
        "polygon(evenodd, 0% 0%, 100% 0%, 50% 100%)"
    );
}

#[test]
fn test_get_computed_style_content() {
    // R2722：getComputedStyle content 序列化（CSS Generated Content，::before/::after 生成内容）。
    let html = "<html><body>\
        <div id=\"none\" style=\"content: none;\"></div>\
        <div id=\"str\" style=\"content: 'hello';\"></div>\
        <div id=\"counter\" style=\"content: counter(c);\"></div>\
        <div id=\"counter-style\" style=\"content: counter(c, upper-roman);\"></div>\
        <div id=\"counters\" style=\"content: counters(n, '.');\"></div>\
        <div id=\"url\" style=\"content: url(x.png);\"></div>\
        <div id=\"list\" style=\"content: 'Chapter ' counter(c);\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // none / normal（默认）。
    assert_eq!(computed_style_property(html, "#none", "content"), "none");
    assert_eq!(computed_style_property(html, "#def", "content"), "normal");
    // 字符串：双引号包裹。
    assert_eq!(computed_style_property(html, "#str", "content"), "\"hello\"");
    // counter(name) / counter(name, style)。
    assert_eq!(computed_style_property(html, "#counter", "content"), "counter(c)");
    assert_eq!(
        computed_style_property(html, "#counter-style", "content"),
        "counter(c, upper-roman)"
    );
    // counters(name, "sep")：分隔符引号串化。
    assert_eq!(
        computed_style_property(html, "#counters", "content"),
        "counters(n, \".\")"
    );
    // url(...)。
    assert_eq!(computed_style_property(html, "#url", "content"), "url(x.png)");
    // 多 component value 列表：空格连接。
    assert_eq!(
        computed_style_property(html, "#list", "content"),
        "\"Chapter \" counter(c)"
    );
}

#[test]
fn test_get_computed_style_font_weight_bolder_lighter() {
    // R2723：getComputedStyle bolder/lighter 按父链 resolved 绝对值解析（CSS Fonts 3 §5.2，
    // 对齐 Chromium；ZeroWeb 保关键字供 paint 二值 want_bold，仅 gCS 路径解析）。
    let html = "<html><body>\
        <b id=\"bolder-normal\" style=\"font-weight: bolder\"></b>\
        <div style=\"font-weight: bold\"><b id=\"bolder-bold\" style=\"font-weight: bolder\"></b></div>\
        <div style=\"font-weight: bold\"><span id=\"lighter-bold\" style=\"font-weight: lighter\"></span></div>\
        <span id=\"lighter-normal\" style=\"font-weight: lighter\"></span>\
        <div id=\"explicit\" style=\"font-weight: 500\"></div>\
        </body></html>";
    // bolder on normal(400) parent → 700。
    assert_eq!(computed_style_property(html, "#bolder-normal", "font-weight"), "700");
    // bolder on bold(700) parent → 900。
    assert_eq!(computed_style_property(html, "#bolder-bold", "font-weight"), "900");
    // lighter on bold(700) parent → 400。
    assert_eq!(computed_style_property(html, "#lighter-bold", "font-weight"), "400");
    // lighter on normal(400) parent → 100。
    assert_eq!(computed_style_property(html, "#lighter-normal", "font-weight"), "100");
    // 非 bolder/lighter 不受影响（显式数值原样）。
    assert_eq!(computed_style_property(html, "#explicit", "font-weight"), "500");
}

#[test]
fn test_get_computed_style_background_position() {
    // R2724：getComputedStyle background-position 序列化（CSS Backgrounds <bg-position># 多层）。
    // Chromium 解析关键字为百分比（WPT background-computed.html），单关键字按轴展开（缺省轴 center 50%）。
    let html = "<html><body>\
        <div id=\"center\" style=\"background-position: center;\"></div>\
        <div id=\"lt\" style=\"background-position: left top;\"></div>\
        <div id=\"rb\" style=\"background-position: right bottom;\"></div>\
        <div id=\"px\" style=\"background-position: 10px 20px;\"></div>\
        <div id=\"pct\" style=\"background-position: 25% 75%;\"></div>\
        <div id=\"multi\" style=\"background-position: center, left top;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // 默认 0% 0%（TwoValue(Percent 0, Percent 0)）。
    assert_eq!(computed_style_property(html, "#def", "background-position"), "0% 0%");
    // 单关键字 center → 两轴展开 50% 50%。
    assert_eq!(
        computed_style_property(html, "#center", "background-position"),
        "50% 50%"
    );
    // TwoValue 关键字 → 解析为 %。
    assert_eq!(computed_style_property(html, "#lt", "background-position"), "0% 0%");
    assert_eq!(computed_style_property(html, "#rb", "background-position"), "100% 100%");
    // TwoValue 长度 → px。
    assert_eq!(computed_style_property(html, "#px", "background-position"), "10px 20px");
    // TwoValue 百分比 → %。
    assert_eq!(computed_style_property(html, "#pct", "background-position"), "25% 75%");
    // 多背景层：逗号分隔。
    assert_eq!(
        computed_style_property(html, "#multi", "background-position"),
        "50% 50%, 0% 0%"
    );
}

#[test]
fn test_get_computed_style_background_size_repeat() {
    // R2725：getComputedStyle background-size + background-repeat 序列化（CSS Backgrounds 多层）。
    let html = "<html><body>\
        <div id=\"size-cover\" style=\"background-size: cover;\"></div>\
        <div id=\"size-px\" style=\"background-size: 100px;\"></div>\
        <div id=\"size-multi\" style=\"background-size: 50%, auto;\"></div>\
        <div id=\"repeat-x\" style=\"background-repeat: repeat-x;\"></div>\
        <div id=\"repeat-multi\" style=\"background-repeat: no-repeat, space;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // background-size 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "background-size"), "auto");
    assert_eq!(computed_style_property(html, "#size-cover", "background-size"), "cover");
    assert_eq!(computed_style_property(html, "#size-px", "background-size"), "100px");
    // 多层逗号分隔。
    assert_eq!(
        computed_style_property(html, "#size-multi", "background-size"),
        "50%, auto"
    );
    // background-repeat 默认 repeat。
    assert_eq!(computed_style_property(html, "#def", "background-repeat"), "repeat");
    assert_eq!(
        computed_style_property(html, "#repeat-x", "background-repeat"),
        "repeat-x"
    );
    // 多层逗号分隔。
    assert_eq!(
        computed_style_property(html, "#repeat-multi", "background-repeat"),
        "no-repeat, space"
    );
}

#[test]
fn test_get_computed_style_background_attachment_clip_origin() {
    // R2726：getComputedStyle background-attachment/clip/origin 序列化（单值 box-model 枚举）。
    let html = "<html><body>\
        <div id=\"att-fixed\" style=\"background-attachment: fixed;\"></div>\
        <div id=\"att-local\" style=\"background-attachment: local;\"></div>\
        <div id=\"clip-pad\" style=\"background-clip: padding-box;\"></div>\
        <div id=\"clip-content\" style=\"background-clip: content-box;\"></div>\
        <div id=\"clip-text\" style=\"background-clip: text;\"></div>\
        <div id=\"origin-border\" style=\"background-origin: border-box;\"></div>\
        <div id=\"origin-content\" style=\"background-origin: content-box;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // background-attachment 默认 scroll。
    assert_eq!(computed_style_property(html, "#def", "background-attachment"), "scroll");
    assert_eq!(
        computed_style_property(html, "#att-fixed", "background-attachment"),
        "fixed"
    );
    assert_eq!(
        computed_style_property(html, "#att-local", "background-attachment"),
        "local"
    );
    // background-clip 默认 border-box。
    assert_eq!(computed_style_property(html, "#def", "background-clip"), "border-box");
    assert_eq!(
        computed_style_property(html, "#clip-pad", "background-clip"),
        "padding-box"
    );
    assert_eq!(
        computed_style_property(html, "#clip-content", "background-clip"),
        "content-box"
    );
    assert_eq!(computed_style_property(html, "#clip-text", "background-clip"), "text");
    // background-origin 默认 padding-box（注意：与 clip 的 border-box 默认不同）。
    assert_eq!(
        computed_style_property(html, "#def", "background-origin"),
        "padding-box"
    );
    assert_eq!(
        computed_style_property(html, "#origin-border", "background-origin"),
        "border-box"
    );
    assert_eq!(
        computed_style_property(html, "#origin-content", "background-origin"),
        "content-box"
    );
}

#[test]
fn test_get_computed_style_alignment_cluster() {
    // R2727：getComputedStyle align-content/justify-items/justify-self 序列化（Box Alignment 簇补齐）。
    let html = "<html><body>\
        <div id=\"ac-center\" style=\"align-content: center;\"></div>\
        <div id=\"ac-between\" style=\"align-content: space-between;\"></div>\
        <div id=\"ji-start\" style=\"justify-items: start;\"></div>\
        <div id=\"ji-right\" style=\"justify-items: right;\"></div>\
        <div id=\"js-end\" style=\"justify-self: end;\"></div>\
        <div id=\"js-stretch\" style=\"justify-self: stretch;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // align-content 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "align-content"), "normal");
    assert_eq!(computed_style_property(html, "#ac-center", "align-content"), "center");
    assert_eq!(
        computed_style_property(html, "#ac-between", "align-content"),
        "space-between"
    );
    // justify-items 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "justify-items"), "normal");
    assert_eq!(computed_style_property(html, "#ji-start", "justify-items"), "start");
    assert_eq!(computed_style_property(html, "#ji-right", "justify-items"), "right");
    // justify-self 默认 auto（注意：与 justify-items 的 normal 默认不同）。
    assert_eq!(computed_style_property(html, "#def", "justify-self"), "auto");
    assert_eq!(computed_style_property(html, "#js-end", "justify-self"), "end");
    assert_eq!(computed_style_property(html, "#js-stretch", "justify-self"), "stretch");
}

#[test]
fn test_get_computed_style_text_break_cluster() {
    // R2728：getComputedStyle word-break/overflow-wrap/hyphens/line-break 序列化（CSS Text 换行/断词簇）。
    let html = "<html><body>\
        <div id=\"wb-all\" style=\"word-break: break-all;\"></div>\
        <div id=\"wb-keep\" style=\"word-break: keep-all;\"></div>\
        <div id=\"ow-word\" style=\"overflow-wrap: break-word;\"></div>\
        <div id=\"ow-any\" style=\"overflow-wrap: anywhere;\"></div>\
        <div id=\"hyph-auto\" style=\"hyphens: auto;\"></div>\
        <div id=\"hyph-manual\" style=\"hyphens: manual;\"></div>\
        <div id=\"lb-strict\" style=\"line-break: strict;\"></div>\
        <div id=\"lb-anywhere\" style=\"line-break: anywhere;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // word-break 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "word-break"), "normal");
    assert_eq!(computed_style_property(html, "#wb-all", "word-break"), "break-all");
    assert_eq!(computed_style_property(html, "#wb-keep", "word-break"), "keep-all");
    // overflow-wrap 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "overflow-wrap"), "normal");
    assert_eq!(computed_style_property(html, "#ow-word", "overflow-wrap"), "break-word");
    assert_eq!(computed_style_property(html, "#ow-any", "overflow-wrap"), "anywhere");
    // hyphens：ZeroWeb 默认 none（diverge：CSS 规范/Chromium 初值 manual）。
    assert_eq!(computed_style_property(html, "#def", "hyphens"), "none");
    assert_eq!(computed_style_property(html, "#hyph-auto", "hyphens"), "auto");
    assert_eq!(computed_style_property(html, "#hyph-manual", "hyphens"), "manual");
    // line-break 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "line-break"), "auto");
    assert_eq!(computed_style_property(html, "#lb-strict", "line-break"), "strict");
    assert_eq!(computed_style_property(html, "#lb-anywhere", "line-break"), "anywhere");
}

#[test]
fn test_get_computed_style_va_bidi_empty() {
    // R2729：getComputedStyle vertical-align/unicode-bidi/empty-cells 序列化（单值关键字枚举）。
    let html = "<html><body>\
        <div id=\"va-middle\" style=\"vertical-align: middle;\"></div>\
        <div id=\"va-text-top\" style=\"vertical-align: text-top;\"></div>\
        <div id=\"va-sub\" style=\"vertical-align: sub;\"></div>\
        <div id=\"ub-isolate\" style=\"unicode-bidi: isolate;\"></div>\
        <div id=\"ub-plaintext\" style=\"unicode-bidi: plaintext;\"></div>\
        <div id=\"ec-hide\" style=\"empty-cells: hide;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // vertical-align 默认 baseline。
    assert_eq!(computed_style_property(html, "#def", "vertical-align"), "baseline");
    assert_eq!(computed_style_property(html, "#va-middle", "vertical-align"), "middle");
    assert_eq!(
        computed_style_property(html, "#va-text-top", "vertical-align"),
        "text-top"
    );
    assert_eq!(computed_style_property(html, "#va-sub", "vertical-align"), "sub");
    // unicode-bidi 默认 normal。
    assert_eq!(computed_style_property(html, "#def", "unicode-bidi"), "normal");
    assert_eq!(computed_style_property(html, "#ub-isolate", "unicode-bidi"), "isolate");
    assert_eq!(
        computed_style_property(html, "#ub-plaintext", "unicode-bidi"),
        "plaintext"
    );
    // empty-cells 默认 show。
    assert_eq!(computed_style_property(html, "#def", "empty-cells"), "show");
    assert_eq!(computed_style_property(html, "#ec-hide", "empty-cells"), "hide");
}

#[test]
fn test_get_computed_style_caret_accent_color() {
    // R2730：getComputedStyle caret-color + accent-color 序列化（CSS UI 颜色 auto | <color>）。
    let html = "<html><body>\
        <div id=\"cc-red\" style=\"caret-color: red;\"></div>\
        <div id=\"cc-cc\" style=\"color: blue; caret-color: currentcolor;\"></div>\
        <div id=\"ac-green\" style=\"accent-color: #00ff00;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // caret-color 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "caret-color"), "auto");
    assert_eq!(
        computed_style_property(html, "#cc-red", "caret-color"),
        "rgb(255, 0, 0)"
    );
    // currentcolor 解析为元素自身 color（blue → rgb(0,0,255)）。
    assert_eq!(computed_style_property(html, "#cc-cc", "caret-color"), "rgb(0, 0, 255)");
    // accent-color 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "accent-color"), "auto");
    assert_eq!(
        computed_style_property(html, "#ac-green", "accent-color"),
        "rgb(0, 255, 0)"
    );
}

#[test]
fn test_get_computed_style_misc_ui() {
    // R2731：getComputedStyle text-wrap/text-align-last/resize/appearance 序列化（misc 单值关键字枚举）。
    let html = "<html><body>\
        <div id=\"tw-balance\" style=\"text-wrap: balance;\"></div>\
        <div id=\"tw-pretty\" style=\"text-wrap: pretty;\"></div>\
        <div id=\"tal-justify\" style=\"text-align-last: justify;\"></div>\
        <div id=\"tal-right\" style=\"text-align-last: right;\"></div>\
        <div id=\"rz-both\" style=\"resize: both;\"></div>\
        <div id=\"rz-horizontal\" style=\"resize: horizontal;\"></div>\
        <div id=\"ap-none\" style=\"appearance: none;\"></div>\
        <div id=\"ap-textfield\" style=\"appearance: textfield;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // text-wrap 默认 wrap。
    assert_eq!(computed_style_property(html, "#def", "text-wrap"), "wrap");
    assert_eq!(computed_style_property(html, "#tw-balance", "text-wrap"), "balance");
    assert_eq!(computed_style_property(html, "#tw-pretty", "text-wrap"), "pretty");
    // text-align-last 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "text-align-last"), "auto");
    assert_eq!(
        computed_style_property(html, "#tal-justify", "text-align-last"),
        "justify"
    );
    assert_eq!(computed_style_property(html, "#tal-right", "text-align-last"), "right");
    // resize 默认 none。
    assert_eq!(computed_style_property(html, "#def", "resize"), "none");
    assert_eq!(computed_style_property(html, "#rz-both", "resize"), "both");
    assert_eq!(computed_style_property(html, "#rz-horizontal", "resize"), "horizontal");
    // appearance 默认 auto；CamelCase→kebab（textfield 不变，slider-horizontal 会变）。
    assert_eq!(computed_style_property(html, "#def", "appearance"), "auto");
    assert_eq!(computed_style_property(html, "#ap-none", "appearance"), "none");
    assert_eq!(
        computed_style_property(html, "#ap-textfield", "appearance"),
        "textfield"
    );
}

#[test]
fn test_get_computed_style_container_ui() {
    // R2732：getComputedStyle box-decoration-break/scrollbar-*/touch-action 序列化（容器交互/UI 枚举）。
    let html = "<html><body>\
        <div id=\"bdb-clone\" style=\"box-decoration-break: clone;\"></div>\
        <div id=\"sw-thin\" style=\"scrollbar-width: thin;\"></div>\
        <div id=\"sg-stable\" style=\"scrollbar-gutter: stable;\"></div>\
        <div id=\"sg-both\" style=\"scrollbar-gutter: stable both-edges;\"></div>\
        <div id=\"ta-panx\" style=\"touch-action: pan-x;\"></div>\
        <div id=\"ta-manip\" style=\"touch-action: manipulation;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // box-decoration-break 默认 slice。
    assert_eq!(computed_style_property(html, "#def", "box-decoration-break"), "slice");
    assert_eq!(
        computed_style_property(html, "#bdb-clone", "box-decoration-break"),
        "clone"
    );
    // scrollbar-width 默认 auto。
    assert_eq!(computed_style_property(html, "#def", "scrollbar-width"), "auto");
    assert_eq!(computed_style_property(html, "#sw-thin", "scrollbar-width"), "thin");
    // scrollbar-gutter 默认 auto；stable / stable both-edges。
    assert_eq!(computed_style_property(html, "#def", "scrollbar-gutter"), "auto");
    assert_eq!(
        computed_style_property(html, "#sg-stable", "scrollbar-gutter"),
        "stable"
    );
    assert_eq!(
        computed_style_property(html, "#sg-both", "scrollbar-gutter"),
        "stable both-edges"
    );
    // touch-action 默认 auto；pan-x / manipulation。
    assert_eq!(computed_style_property(html, "#def", "touch-action"), "auto");
    assert_eq!(computed_style_property(html, "#ta-panx", "touch-action"), "pan-x");
    assert_eq!(
        computed_style_property(html, "#ta-manip", "touch-action"),
        "manipulation"
    );
}

#[test]
fn test_get_computed_style_outline_break() {
    // R2733：getComputedStyle outline-offset + break-* 序列化（补齐 outline 簇 + Fragmentation 簇）。
    let html = "<html><body>\
        <div id=\"oo-px\" style=\"outline-offset: 4px;\"></div>\
        <div id=\"oo-neg\" style=\"outline-offset: -2px;\"></div>\
        <div id=\"bb-avoid\" style=\"break-before: avoid;\"></div>\
        <div id=\"bb-column\" style=\"break-before: column;\"></div>\
        <div id=\"ba-avoid-page\" style=\"break-after: avoid-page;\"></div>\
        <div id=\"bi-avoid\" style=\"break-inside: avoid;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // outline-offset 默认 0px。
    assert_eq!(computed_style_property(html, "#def", "outline-offset"), "0px");
    assert_eq!(computed_style_property(html, "#oo-px", "outline-offset"), "4px");
    assert_eq!(computed_style_property(html, "#oo-neg", "outline-offset"), "-2px");
    // break-before 默认 auto；avoid / column。
    assert_eq!(computed_style_property(html, "#def", "break-before"), "auto");
    assert_eq!(computed_style_property(html, "#bb-avoid", "break-before"), "avoid");
    assert_eq!(computed_style_property(html, "#bb-column", "break-before"), "column");
    // break-after 默认 auto；avoid-page（CamelCase→kebab）。
    assert_eq!(computed_style_property(html, "#def", "break-after"), "auto");
    assert_eq!(
        computed_style_property(html, "#ba-avoid-page", "break-after"),
        "avoid-page"
    );
    // break-inside 默认 auto；avoid。
    assert_eq!(computed_style_property(html, "#def", "break-inside"), "auto");
    assert_eq!(computed_style_property(html, "#bi-avoid", "break-inside"), "avoid");
}

#[test]
fn test_get_computed_style_grid_container() {
    // R2734：getComputedStyle grid-auto-flow + container-type/name + tab-size 序列化。
    let html = "<html><body>\
        <div id=\"gaf-col\" style=\"grid-auto-flow: column;\"></div>\
        <div id=\"gaf-dense\" style=\"grid-auto-flow: dense;\"></div>\
        <div id=\"ct-size\" style=\"container-type: size;\"></div>\
        <div id=\"ct-inline\" style=\"container-type: inline-size;\"></div>\
        <div id=\"cn-named\" style=\"container-name: sidebar;\"></div>\
        <div id=\"ts-px\" style=\"tab-size: 24px;\"></div>\
        <div id=\"ts-num\" style=\"tab-size: 4;\"></div>\
        <div id=\"def\"></div>\
        </body></html>";
    // grid-auto-flow 默认 row；column / dense（ZeroWeb 解析 dense→RowDense 多词）。
    assert_eq!(computed_style_property(html, "#def", "grid-auto-flow"), "row");
    assert_eq!(computed_style_property(html, "#gaf-col", "grid-auto-flow"), "column");
    assert_eq!(
        computed_style_property(html, "#gaf-dense", "grid-auto-flow"),
        "row dense"
    );
    // container-type 默认 normal；size / inline-size。
    assert_eq!(computed_style_property(html, "#def", "container-type"), "normal");
    assert_eq!(computed_style_property(html, "#ct-size", "container-type"), "size");
    assert_eq!(
        computed_style_property(html, "#ct-inline", "container-type"),
        "inline-size"
    );
    // container-name 默认 none；显式字符串。
    assert_eq!(computed_style_property(html, "#def", "container-name"), "none");
    assert_eq!(computed_style_property(html, "#cn-named", "container-name"), "sidebar");
    // tab-size 默认 8（CSS 规范初值）；px / number。
    assert_eq!(computed_style_property(html, "#def", "tab-size"), "8");
    assert_eq!(computed_style_property(html, "#ts-px", "tab-size"), "24px");
    assert_eq!(computed_style_property(html, "#ts-num", "tab-size"), "4");
}

#[test]
fn test_io_cross_threshold_host_tick_r3062() {
    // R3062：IntersectionObserver cross-threshold host-tick 验证。元素从视口外移入 → host tick
    //（`__zw_observers_tick`，renderer render 后调）复算 rect → `_crossed`（threshold 越界）→ 派发后续通知。
    // lazy-load / infinite-scroll 高频 hook。经共享可变 mock rect 模拟元素移动。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='t'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    // 可控 rect：target #t 经共享状态控制（模拟视口外→内移动）。初始视口外（y=200 > vh=100）。
    let rect: Arc<Mutex<String>> = Arc::new(Mutex::new("0,200,100,50".to_string()));
    {
        let r = Arc::clone(&rect);
        sandbox.register_callback(
            "__zw_getBoundingClientRect",
            Box::new(move |args| {
                let sel = args.first().cloned().unwrap_or_default();
                if sel.contains('t') {
                    r.lock().map(|s| s.clone()).unwrap_or_default()
                } else {
                    "0,0,0,0".to_string()
                }
            }),
        );
    }
    sandbox.execute(
        "globalThis.innerWidth=100; globalThis.innerHeight=100;\
         globalThis.__count=0; globalThis.__fired=null;\
         new IntersectionObserver(function(e){ globalThis.__count++; globalThis.__fired=e[0].isIntersecting; }, {threshold:[0.5]})\
           .observe(document.querySelector('#t'));",
    )
    .unwrap();
    // observe → initial fire（视口外，ratio=0，isIntersecting=false）。
    assert_eq!(sandbox.execute("String(globalThis.__count)").unwrap().value, "1", "observe -> initial fire");
    assert_eq!(sandbox.execute("String(globalThis.__fired)").unwrap().value, "false", "initial isIntersecting=false (out of view)");

    // tick（rect 仍视口外）→ 不派（_crossed=false，prev ratio=0 当前 ratio=0）。
    sandbox.execute("__zw_observers_tick();").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__count)").unwrap().value, "1", "rect 未变 -> tick 不派 (crossed=false)");

    // 元素移入视口（rect 改为 y=0，ratio=1）→ tick → cross-threshold fire（isIntersecting=true）。
    *rect.lock().unwrap() = "0,0,100,50".to_string();
    sandbox.execute("__zw_observers_tick();").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__count)").unwrap().value, "2", "rect 变入视口 -> tick cross-threshold fire");
    assert_eq!(sandbox.execute("String(globalThis.__fired)").unwrap().value, "true", "cross-threshold 后 isIntersecting=true");

    // tick（rect 不变）→ 不派（_crossed=false，prev=1 当前=1）。
    sandbox.execute("__zw_observers_tick();").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__count)").unwrap().value, "2", "rect 未变 -> tick 不派");
}

#[test]
fn test_ro_size_change_host_tick_r3063() {
    // R3063：ResizeObserver size-change host-tick 验证（镜像 R3062 IO cross-threshold）。元素尺寸变化 →
    // host tick（`__zw_observers_tick`，renderer render 后调）复算 rect → `_schedule` 内 size-diff 检测
    //（`prev.w !== r.w || prev.h !== r.h`）→ 派发后续通知。响应式设计 / 元素查询 / 动态内容测量高频 hook。
    // 经共享可变 mock rect 模拟元素尺寸变化。test-only：锁定 cross-tick 行为，未来回归（size-diff / tick /
    // gBCR 复算任一环坏）会被此测试捕获。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='t'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    // 可控 rect：target #t 经共享状态控制（模拟尺寸变化）。初始 100x50（无 padding/border → content=border）。
    let rect: Arc<Mutex<String>> = Arc::new(Mutex::new("0,0,100,50".to_string()));
    {
        let r = Arc::clone(&rect);
        sandbox.register_callback(
            "__zw_getBoundingClientRect",
            Box::new(move |args| {
                let sel = args.first().cloned().unwrap_or_default();
                if sel.contains("#t") {
                    r.lock().map(|s| s.clone()).unwrap_or_default()
                } else {
                    "0,0,0,0".to_string()
                }
            }),
        );
    }
    sandbox.execute(
        "globalThis.__count=0; globalThis.__entry=null;\
         new ResizeObserver(function(e){ globalThis.__count++; globalThis.__entry=e[0]; })\
           .observe(document.querySelector('#t'));",
    )
    .unwrap();
    // observe → initial fire（首次 prev==null → 必派；border-box 100x50）。
    assert_eq!(sandbox.execute("String(globalThis.__count)").unwrap().value, "1", "observe -> initial fire");
    assert_eq!(
        sandbox.execute("String(globalThis.__entry.borderBoxSize[0].inlineSize)").unwrap().value,
        "100",
        "initial borderBoxSize.inlineSize = 100"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__entry.borderBoxSize[0].blockSize)").unwrap().value,
        "50",
        "initial borderBoxSize.blockSize = 50"
    );

    // tick（rect 不变 100x50）→ 不派（size-diff=false，prev.w=100/h=50 当前同）。
    sandbox.execute("__zw_observers_tick();").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__count)").unwrap().value, "1", "rect 未变 -> tick 不派 (size-diff=false)");

    // 元素尺寸变化（rect 改为 200x80）→ tick → size-change fire（border-box 200x80）。
    *rect.lock().unwrap() = "0,0,200,80".to_string();
    sandbox.execute("__zw_observers_tick();").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__count)").unwrap().value,
        "2",
        "rect 尺寸变化 -> tick size-change fire"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__entry.borderBoxSize[0].inlineSize)").unwrap().value,
        "200",
        "size-change 后 borderBoxSize.inlineSize = 200"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__entry.borderBoxSize[0].blockSize)").unwrap().value,
        "80",
        "size-change 后 borderBoxSize.blockSize = 80"
    );

    // tick（rect 不变 200x80）→ 不派（size-diff=false，prev.w=200/h=80 当前同）。
    sandbox.execute("__zw_observers_tick();").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__count)").unwrap().value, "2", "rect 未变 -> tick 不派");
}

#[test]
fn test_window_event_current_event_global_r33() {
    // R33：`Window.event`（HTML spec `current event`，legacy IE 全局）。Window 须 own `event` 属性，初值
    // undefined；dispatch 期 = 正在派发的 event（innermost，嵌套 dispatch 后恢复外层）；dispatch 后回 undefined。
    // 裸 `event` 全局（listener 内 event.stopPropagation() 等 legacy 写法）依赖此。spec `window-event`。
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
        "<html><body><div id=\"p\"><span id=\"c\">x</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① Window own `event` 属性 + 初值 undefined（WPT event-global "event exists on window, initially undefined"）。
    assert_eq!(
        sandbox
            .execute("String(Object.prototype.hasOwnProperty.call(globalThis, 'event'))")
            .unwrap()
            .value,
        "true",
        "window 须 own event 属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.event)").unwrap().value,
        "undefined",
        "dispatch 前 window.event === undefined"
    );

    // ② dispatch 期 window.event === 正在派发的 event；dispatch 后回 undefined。
    sandbox
        .execute(
            "var __captured;\n\
             document.querySelector('#c').addEventListener('click', function(e){\n\
               __captured = (window.event === e) + '/' + (typeof event !== 'undefined' && event === e);\n\
             });",
        )
        .unwrap();
    sandbox
        .execute("document.querySelector('#c').dispatchEvent(new Event('click'));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__captured").unwrap().value,
        "true/true",
        "dispatch 期 window.event === e（含裸 event 全局）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.event)").unwrap().value,
        "undefined",
        "dispatch 后 window.event === undefined"
    );

    // ③ 嵌套 dispatch（redispatch）：内层 dispatch 后恢复外层 event，外层 listener 仍见外层 event。
    // 内层 listener 设 __inner，外层 listener 派发内层后再读 window.event 应仍 === 外层 event。
    sandbox
        .execute(
            "var __outerEventEq = 'unset', __innerEventEq = 'unset';\n\
             document.querySelector('#p').addEventListener('outer', function(eOuter){\n\
               __innerEventEq = 'pre';\n\
               // 内层 dispatch：listener 期 window.event 应 === 内层 event\n\
               document.querySelector('#c').dispatchEvent(new Event('inner'));\n\
               // 内层 dispatch 结束后 window.event 应恢复 === 外层 event\n\
               __outerEventEq = (window.event === eOuter);\n\
             });\n\
             document.querySelector('#c').addEventListener('inner', function(eInner){\n\
               __innerEventEq = (window.event === eInner);\n\
             });",
        )
        .unwrap();
    sandbox
        .execute("document.querySelector('#p').dispatchEvent(new Event('outer'));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__innerEventEq").unwrap().value,
        "true",
        "嵌套 dispatch 内层 window.event === 内层 event"
    );
    assert_eq!(
        sandbox.execute("globalThis.__outerEventEq").unwrap().value,
        "true",
        "嵌套 dispatch 内层结束后恢复外层 window.event === 外层 event"
    );
}

#[test]
fn test_at_target_stop_propagation_halts_same_element_r34() {
    // R34：AT_TARGET（target 阶段，element 无祖先）capture-listener 调 stopPropagation 须止**同元素**的
    // non-capture listener。根因有两层：① `_dispatchToListeners` 'all' 模式 non-capture 循环未检查 stop flag；
    // ② polyfill Event 设 `_propagationStopped`，但叠加路径（ZW_NATIVE_DOM=1，`new MouseEvent` 走 native Event
    // 构造器）stopPropagation 设 `__zw_stop`，polyfill dispatch 须同认两 flag（未解问题 #9：dispatch 走 polyfill）。
    // WPT Event-stopPropagation-cancel-bubbling：capture 内 stopPropagation 止同元素 bubble handler。
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
        "<html><body><div id=\"t\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① 同元素 capture（设 _propagationStopped，polyfill Event）止同元素 non-capture。
    sandbox
        .execute(
            "document.querySelector('#t').addEventListener('click', function(e){ globalThis.__cap1 = true; e.stopPropagation(); }, { capture: true });\n\
             document.querySelector('#t').addEventListener('click', function(){ globalThis.__bub1 = true; });\n\
             document.querySelector('#t').dispatchEvent(new Event('click', { bubbles: true }));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cap1 === true").unwrap().value,
        "true",
        "同元素 capture listener 应触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__bub1 === true").unwrap().value,
        "false",
        "AT_TARGET capture stopPropagation 须止同元素 non-capture listener"
    );

    // ② 反向：non-capture 先注册，capture 后注册仍先触发并 stopPropagation 止 non-capture（注册序无关，
    // capture 总在 non-capture 前）。验证 __bub2 仍 false。
    sandbox
        .execute(
            "document.querySelector('#t').addEventListener('foo', function(){ globalThis.__bub2 = true; });\n\
             document.querySelector('#t').addEventListener('foo', function(e){ globalThis.__cap2 = true; e.stopPropagation(); }, { capture: true });\n\
             document.querySelector('#t').dispatchEvent(new Event('foo', { bubbles: true }));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__bub2 === true").unwrap().value,
        "false",
        "capture 总先于 non-capture 触发并 stopPropagation 止之（注册序无关）"
    );

    // ③ native flag 兼容：模拟叠加路径下 stopPropagation 设 `__zw_stop`（native Event 构造器行为），
    // polyfill dispatch 须认此 flag 止同元素 non-capture。
    sandbox
        .execute(
            "document.querySelector('#t').addEventListener('bar', function(e){ globalThis.__cap3 = true; e.__zw_stop = true; }, { capture: true });\n\
             document.querySelector('#t').addEventListener('bar', function(){ globalThis.__bub3 = true; });\n\
             document.querySelector('#t').dispatchEvent(new Event('bar', { bubbles: true }));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__bub3 === true").unwrap().value,
        "false",
        "native __zw_stop flag 须被 polyfill dispatch 认（叠加路径对齐）"
    );

    // ④ 无 stopPropagation 的正常场景：同元素 capture + non-capture 都应触发（_propagationStopped 默认 false
    // 不误伤）。防 R34 修改过度止息。
    sandbox
        .execute(
            "document.querySelector('#t').addEventListener('baz', function(){ globalThis.__cap4 = true; }, { capture: true });\n\
             document.querySelector('#t').addEventListener('baz', function(){ globalThis.__bub4 = true; });\n\
             document.querySelector('#t').dispatchEvent(new Event('baz', { bubbles: true }));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cap4 === true && globalThis.__bub4 === true").unwrap().value,
        "true",
        "无 stopPropagation 时 capture + non-capture 都应触发（不误伤）"
    );
}

#[test]
fn test_event_phase_during_dispatch_r35() {
    // R35：spec `concept-event-dispatch`——派发期 event.eventPhase 反映当前阶段：祖先 capture→
    // CAPTURING_PHASE(1)、target（AT_TARGET）→ 2（target 的 capture 与 non-capture listener 都为 AT_TARGET）、
    // 祖先 bubble→ BUBBLING_PHASE(3)；dispatch 完全结束后复位 NONE(0) + currentTarget→null。
    // WPT Event-dispatch-order-at-target：target 阶段 capture/bubble listener 都 eventPhase===AT_TARGET(2)。
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
        "<html><body><div id=\"a\"><div id=\"b\"><i id=\"c\">x</i></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 三阶段 eventPhase：#a capture(1) → #c target AT_TARGET(2，capture+non-capture 都 2) → #a bubble(3)。
    // 收集每个 listener 触发时的 eventPhase。
    sandbox
        .execute(
            "var phases = [];\n\
             document.querySelector('#a').addEventListener('click', function(e){ phases.push('a-cap:'+e.eventPhase); }, true);\n\
             document.querySelector('#a').addEventListener('click', function(e){ phases.push('a-bub:'+e.eventPhase); }, false);\n\
             document.querySelector('#c').addEventListener('click', function(e){ phases.push('c-cap:'+e.eventPhase); }, true);\n\
             document.querySelector('#c').addEventListener('click', function(e){ phases.push('c-bub:'+e.eventPhase); }, false);\n\
             var ev = new Event('click', { bubbles: true });\n\
             document.querySelector('#c').dispatchEvent(ev);\n\
             globalThis.__phases = phases.join(',');\n\
             globalThis.__postPhase = ev.eventPhase;\n\
             globalThis.__postCT = String(ev.currentTarget);",
        )
        .unwrap();
    // capture 倒序：#a capture 先（1）；target #c：capture(2) + non-capture(2) 都 AT_TARGET；bubble 正序：#a(3)。
    assert_eq!(
        sandbox.execute("globalThis.__phases").unwrap().value,
        "a-cap:1,c-cap:2,c-bub:2,a-bub:3",
        "三阶段 eventPhase：capture(1)/AT_TARGET(2，target 的 cap+bub)/bubble(3)"
    );
    // dispatch 完全结束后 eventPhase 复位 NONE(0)。
    assert_eq!(
        sandbox.execute("globalThis.__postPhase").unwrap().value,
        "0",
        "dispatch 后 eventPhase 复位 NONE(0)"
    );
    // dispatch 后 currentTarget 复位 null（spec concept-event-dispatch 末尾）。
    assert_eq!(
        sandbox.execute("globalThis.__postCT").unwrap().value,
        "null",
        "dispatch 后 currentTarget 复位 null"
    );
}

#[test]
fn test_preset_stop_flag_zero_dispatch_r39() {
    // R39：spec `concept-event-dispatch` 步骤 2——dispatch 开始时若 stop propagation flag **已设**
    // （dispatch 前外部调 stopPropagation()/stopImmediatePropagation()/设 cancelBubble=true，R29 setter
    // 等同 stopPropagation），跳过全部 listener 触发（capture/target/bubble 三阶段全不进）。
    // WPT Event-dispatch-propagation-stopped（dispatch 前 stopPropagation → 零触发）+
    // Event-dispatch-bubble-canceled（dispatch 前 cancelBubble=true → 零触发）。
    // 旧实现各阶段循环先派发后才查 flag → html capture 先触发 2 次才止（wrong）。
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
        "<html><body><div id=\"a\"><i id=\"c\">x</i></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 三种 pre-set flag 形态（stopPropagation / stopImmediatePropagation / cancelBubble=true）都零触发；
    // dispatch 后 flag 被 finally 重置（R29 spec 步骤14），同 event 再派发恢复正常三阶段。
    sandbox
        .execute(
            "var hits = [];\n\
             function arm() { hits = [];\n\
               document.querySelector('#a').addEventListener('click', function(e){ hits.push('a-cap'); }, true);\n\
               document.querySelector('#a').addEventListener('click', function(e){ hits.push('a-bub'); }, false);\n\
               document.querySelector('#c').addEventListener('click', function(e){ hits.push('c'); }, false);\n\
               window.addEventListener('click', function(e){ hits.push('win'); }); }\n\
             function fresh() { var ev = new Event('click', { bubbles: true }); return ev; }\n\
             var e1 = fresh(); e1.stopPropagation(); document.querySelector('#c').dispatchEvent(e1);\n\
             globalThis.__h1 = hits.join(',');\n\
             var e2 = fresh(); e2.stopImmediatePropagation(); document.querySelector('#c').dispatchEvent(e2);\n\
             globalThis.__h2 = hits.join(',');\n\
             var e3 = fresh(); e3.cancelBubble = true; document.querySelector('#c').dispatchEvent(e3);\n\
             globalThis.__h3 = hits.join(',');\n\
             arm();\n\
             var e4 = fresh(); document.querySelector('#c').dispatchEvent(e4);\n\
             globalThis.__h4 = hits.join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__h1").unwrap().value,
        "",
        "dispatch 前 stopPropagation → 零触发（含 window/html 共享 key listener）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__h2").unwrap().value,
        "",
        "dispatch 前 stopImmediatePropagation → 零触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__h3").unwrap().value,
        "",
        "dispatch 前 cancelBubble=true（R29 setter 置 flag）→ 零触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__h4").unwrap().value,
        "a-cap,c,a-bub,win",
        "无 pre-set flag 的正常 dispatch 不受影响（capture/target/bubble + window listener）"
    );
}

#[test]
fn test_doc_win_dispatch_chain_slots_r40() {
    // R40：document/window 入派发链（spec 结构 html → document → window）+ 槽位身份。
    // ① 元素 target 连入文档：完整链 = [元素祖先链…, document, window]，document/window listener 以
    //    document/window 本体为 currentTarget 触发（不再是 html proxy）；capture 反序 win→doc 在元素链前。
    // ② document.dispatchEvent：path = [document, window]——doc AT_TARGET 一次 + win 冒泡一次。
    // ③ window.dispatchEvent：path = [window] 仅 win AT_TARGET。
    // ④ stopPropagation 在元素祖先链止住后，document/window 虚站不再触发。
    // ⑤ detached 元素（createElement 未挂载）不经 document/window 虚站。
    // WPT Event-dispatch-multiple-stopPropagation / omitted-capture / bubbles-true 主路径。
    // https://dom.spec.whatwg.org/#concept-event-dispatch
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
        "<html><body><div id=\"a\"><i id=\"c\">x</i></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var log = [];\n\
             function tag(e){ return e === document ? 'DOC' : e === window ? 'WIN' : (e && e.id) || String(e); }\n\
             document.querySelector('#c').addEventListener('k', function(e){ log.push('c:'+tag(e.currentTarget)+':'+e.eventPhase); }, false);\n\
             document.addEventListener('k', function(e){ log.push('doc@doc:'+tag(e.currentTarget)+':'+e.eventPhase); }, false);\n\
             window.addEventListener('k', function(e){ log.push('win@win:'+tag(e.currentTarget)+':'+e.eventPhase); }, false);\n\
             // ① 元素派发：完整链 target(#c:2) → doc 站(3, DOC 身份) → win 站(3, WIN 身体)\n\
             var e1 = new Event('k', { bubbles: true });\n\
             document.querySelector('#c').dispatchEvent(e1);\n\
             globalThis.__r1 = log.join(','); log = [];\n\
             // ② document.dispatchEvent：DOC:2（AT_TARGET 一次）→ WIN:3（冒泡一次）\n\
             var e2 = new Event('k', { bubbles: true });\n\
             document.dispatchEvent(e2);\n\
             globalThis.__r2 = log.join(','); log = [];\n\
             // ③ window.dispatchEvent：仅 WIN:2\n\
             var e3 = new Event('k', { bubbles: true });\n\
             window.dispatchEvent(e3);\n\
             globalThis.__r3 = log.join(','); log = [];\n\
             // ④ 祖先链 stopPropagation 止住后 doc/win 虚站不触发\n\
             document.querySelector('#a').addEventListener('k', function(e){ e.stopPropagation(); }, false);\n\
             var e4 = new Event('k', { bubbles: true });\n\
             document.querySelector('#c').dispatchEvent(e4);\n\
             globalThis.__r4 = log.join(','); log = [];\n\
             // ⑤ detached 元素不经 doc/win 虚站\n\
             var d = document.createElement('div');\n\
             d.addEventListener('k', function(e){ log.push('det:'+tag(e.currentTarget)); }, false);\n\
             var e5 = new Event('k', { bubbles: true });\n\
             d.dispatchEvent(e5);\n\
             globalThis.__r5 = log.join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "c:c:2,doc@doc:DOC:3,win@win:WIN:3",
        "① 元素派发完整链：target(#c AT_TARGET) → document 站（DOC 身份，bubble 3）→ window 站（WIN 身份，bubble 3）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r2").unwrap().value,
        "doc@doc:DOC:2,win@win:WIN:3",
        "② document.dispatchEvent = [document(AT_TARGET 一次), window(bubble 一次)]，doc 不重复触发"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r3").unwrap().value,
        "win@win:WIN:2",
        "③ window.dispatchEvent = [window] 仅 AT_TARGET，无 doc 站（doc 是 window 的后代不在 path）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r4").unwrap().value,
        "c:c:2",
        "④ 祖先 #a stopPropagation 止住后 document/window 虚站不再触发（spec path 后续节点全止）"
    );
    sandbox
        .execute(
            "globalThis.__r5has = log.join(',').indexOf('DOC') >= 0 || log.join(',').indexOf('WIN') >= 0 || log.join(',').indexOf('doc@doc') >= 0 || log.join(',').indexOf('win@win') >= 0;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5has").unwrap().value,
        "false",
        "⑤ detached 元素（未挂载）path 止于自身，不经 document/window 虚站（无 DOC/WIN 触发）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5.split(':').length >= 2").unwrap().value,
        "true",
        "⑤ detached 元素自身 listener 正常触发（target 站）"
    );
}

#[test]
fn test_treewalker_api_surface_r41() {
    // R41：dom/traversal 导入建基线驱动的 TreeWalker/NodeIterator API 面修复。
    // ① NodeFilter 常量对齐上游全表（SHOW_PROCESSING_INSTRUCTION 0x40 修正 + SHOW_ATTRIBUTE/ENTITY/
    //    ENTITY_REFERENCE/NOTATION 补齐，WPT NodeFilter-constants.html）
    // ② createTreeWalker root 缺省/无效抛 TypeError（spec document-createtreewalker 步骤 1）
    // ③ root/whatToShow/filter readonly（defineProperty getter-only）
    // ④ whatToShow 显式 null → 0（ToUint32(null)，区别缺省 → SHOW_ALL）
    // ⑤ toString 接口 branding（[object TreeWalker] / [object NodeIterator]）
    // ⑥ currentNode 赋非 Node 抛 TypeError；赋合法 Node 接受
    // https://dom.spec.whatwg.org/#interface-treewalker
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
        "<html><body><div id=\"a\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var body = document.body;\n\
             // ① 常量全表（上游 NodeFilter-constants.html 断言集）\n\
             globalThis.__c1 = [NodeFilter.SHOW_ALL, NodeFilter.SHOW_ELEMENT, NodeFilter.SHOW_ATTRIBUTE,\n\
               NodeFilter.SHOW_TEXT, NodeFilter.SHOW_CDATA_SECTION, NodeFilter.SHOW_ENTITY_REFERENCE,\n\
               NodeFilter.SHOW_ENTITY, NodeFilter.SHOW_PROCESSING_INSTRUCTION, NodeFilter.SHOW_COMMENT,\n\
               NodeFilter.SHOW_DOCUMENT, NodeFilter.SHOW_DOCUMENT_TYPE, NodeFilter.SHOW_DOCUMENT_FRAGMENT,\n\
               NodeFilter.SHOW_NOTATION, NodeFilter.FILTER_ACCEPT, NodeFilter.FILTER_REJECT, NodeFilter.FILTER_SKIP].join(',');\n\
             // ② root 无效抛 TypeError\n\
             try { document.createTreeWalker(); globalThis.__c2 = 'no-throw'; }\n\
             catch (e) { globalThis.__c2 = (e instanceof TypeError) ? 'TypeError' : String(e); }\n\
             try { document.createNodeIterator(null); globalThis.__c3 = 'no-throw'; }\n\
             catch (e) { globalThis.__c3 = (e instanceof TypeError) ? 'TypeError' : String(e); }\n\
             // ③④⑤ readonly + null→0 + toString\n\
             var w = document.createTreeWalker(body);\n\
             var w2 = document.createTreeWalker(body, null, null);\n\
             var it = document.createNodeIterator(body);\n\
             globalThis.__c4 = [String(w), String(it)].join('|');\n\
             globalThis.__c5 = w.whatToShow;\n\
             globalThis.__c6 = w2.whatToShow;\n\
             // readonly 判定与 testharness assert_readonly 同口径：accessor 属性 set===undefined（getter-only）。\n\
             globalThis.__c7 = Object.getOwnPropertyDescriptor(w, 'root').set === undefined\n\
               && Object.getOwnPropertyDescriptor(w, 'whatToShow').set === undefined\n\
               && Object.getOwnPropertyDescriptor(w, 'filter').set === undefined;\n\
             // ⑥ currentNode setter 校验\n\
             try { w.currentNode = null; globalThis.__c8 = 'no-throw'; }\n\
             catch (e) { globalThis.__c8 = (e instanceof TypeError) ? 'TypeError' : String(e); }\n\
             try { w.currentNode = {}; globalThis.__c9 = 'no-throw'; }\n\
             catch (e) { globalThis.__c9 = (e instanceof TypeError) ? 'TypeError' : String(e); }\n\
             var div = document.querySelector('#a');\n\
             w.currentNode = div;\n\
             globalThis.__c10 = (w.currentNode === div);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__c1").unwrap().value,
        "4294967295,1,2,4,8,16,32,64,128,256,512,1024,2048,1,2,3",
        "① NodeFilter 常量上游全表（含 SHOW_PI=0x40 修正 + 4 个补齐常量）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c2").unwrap().value,
        "TypeError",
        "② createTreeWalker() 无 root 抛 TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c3").unwrap().value,
        "TypeError",
        "② createNodeIterator(null) 抛 TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c4").unwrap().value,
        "[object TreeWalker]|[object NodeIterator]",
        "⑤ toString 接口 branding"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c5").unwrap().value,
        "4294967295",
        "④ 缺省 whatToShow → SHOW_ALL"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c6").unwrap().value,
        "0",
        "④ 显式 null whatToShow → 0（ToUint32(null)，非缺省语义）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c7").unwrap().value,
        "true",
        "③ root/whatToShow/filter readonly"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c8").unwrap().value,
        "TypeError",
        "⑥ currentNode = null 抛 TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c9").unwrap().value,
        "TypeError",
        "⑥ currentNode 赋普通对象抛 TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c10").unwrap().value,
        "true",
        "⑥ currentNode 赋合法 Node 接受"
    );
}

#[test]
fn test_range_apis_r42() {
    // R42：dom/ranges 导入建基线驱动的 Range/StaticRange/Attr 端点 API 面修复。
    // ① `new Range()` 返真实实例（旧空函数 stub → setStart 抛 TypeError）
    // ② `StaticRange(init)` 构造器：readonly 四属性 + collapsed 派生 + 非 Node 容器抛 TypeError
    // ③ `element.getAttributeNode(name)` 返 Attr 节点（instanceof Attr / value / ownerElement），缺省 null
    // ④ Range setStart/setEnd spec 校验：仅拒 DocumentType（Attr 允许作容器，length=0）；offset 越界
    //    （文本/注释/PI data length 或 Attr 0）抛 IndexSizeError；setStartBefore 族 parent=null 抛
    //    InvalidNodeTypeError；selectNode 无 parent 同抛
    // https://dom.spec.whatwg.org/#interface-range / #dom-staticrange
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
        "<html><body><div id=\"a\">hello</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var results = [];\n\
             // ① new Range() 真实实例\n\
             var r1 = new Range();\n\
             globalThis.__n1 = typeof r1.setStart;\n\
             // ② StaticRange 构造器\n\
             var div = document.querySelector('#a');\n\
             var sr = new StaticRange({ startContainer: div, startOffset: 1, endContainer: div, endOffset: 2 });\n\
             globalThis.__n2 = [sr.startContainer === div, sr.startOffset, sr.endOffset, sr.collapsed].join(',');\n\
             try { new StaticRange({ startContainer: {}, startOffset: 0, endContainer: div, endOffset: 0 }); globalThis.__n3 = 'no-throw'; }\n\
             catch (e) { globalThis.__n3 = e instanceof TypeError ? 'TypeError' : String(e); }\n\
             var sr2 = new StaticRange({ startContainer: div, startOffset: 1, endContainer: div, endOffset: 1 });\n\
             globalThis.__n3b = sr2.collapsed;\n\
             // ③ getAttributeNode\n\
             div.setAttribute('x', 'abc');\n\
             var attr = div.getAttributeNode('x');\n\
             globalThis.__n4 = [attr instanceof Attr, attr.value, attr.name, attr.ownerElement === div].join(',');\n\
             globalThis.__n5 = div.getAttributeNode('nonexistent');\n\
             // ④ Range 校验\n\
             var r2 = new Range();\n\
             r2.setStart(attr, 0);\n\
             globalThis.__n6 = r2.startContainer === attr;\n\
             try { r2.setStart(attr, 1); globalThis.__n7 = 'no-throw'; }\n\
             catch (e) { globalThis.__n7 = (e && e.name) || String(e); }\n\
             var text = document.createTextNode('hello');\n\
             try { r2.setStart(text, 6); globalThis.__n8 = 'no-throw'; }\n\
             catch (e) { globalThis.__n8 = (e && e.name) || String(e); }\n\
             try { r2.setStartBefore(attr); globalThis.__n9 = 'no-throw'; }\n\
             catch (e) { globalThis.__n9 = (e && e.name) || String(e); }\n\
             try { r2.selectNode(attr); globalThis.__n10 = 'no-throw'; }\n\
             catch (e) { globalThis.__n10 = (e && e.name) || String(e); }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__n1").unwrap().value,
        "function",
        "① new Range() 返真实实例（setStart 可用）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n2").unwrap().value,
        "true,1,2,false",
        "② StaticRange 四属性 + collapsed=false"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n3").unwrap().value,
        "TypeError",
        "② StaticRange 非 Node 容器抛 TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n3b").unwrap().value,
        "true",
        "② StaticRange start===end collapsed=true"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n4").unwrap().value,
        "true,abc,x,true",
        "③ getAttributeNode 返 Attr（instanceof/value/name/ownerElement）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n5").unwrap().value,
        "null",
        "③ getAttributeNode 缺省属性返 null"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n6").unwrap().value,
        "true",
        "④ Attr 允许作 setStart 容器（offset 0）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n7").unwrap().value,
        "IndexSizeError",
        "④ Attr offset>0（length=0）抛 IndexSizeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n8").unwrap().value,
        "IndexSizeError",
        "④ 文本节点 offset>length 抛 IndexSizeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n9").unwrap().value,
        "InvalidNodeTypeError",
        "④ setStartBefore 无 parent（Attr）抛 InvalidNodeTypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__n10").unwrap().value,
        "InvalidNodeTypeError",
        "④ selectNode 无 parent 抛 InvalidNodeTypeError"
    );
}

#[test]
fn test_htmlcollection_indexed_named_props_r43() {
    // R43：spec legacy platform object（HTMLCollection）属性语义（WPT HTMLCollection-delete +
    // getElementsByClassName-32 "does not get confused by numeric IDs"）。
    // ① indexed 属性不可配置：delete c[0] no-op（loose，值不丢——普通数组 delete 挖洞致永久
    //    undefined）；strict 模式 delete 抛 TypeError
    // ② named getter：c.<id> 命中带 id 元素，同样不可配置（delete c.foo no-op）
    // ③ 纯数字 id 不经 named 暴露（数组 defineProperty 数字键会把 length 推到 index+1——
    //    "does not get confused by numeric IDs" 断言 collection.length 与 map ids 不受扰）
    // https://dom.spec.whatwg.org/#interface-htmlcollection（legacy platform object）
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
        "<html><body><i id=\"foo\"></i><div class=\"k\" id=\"1\"></div><div class=\"k\" id=\"2\"></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var c = document.getElementsByTagName('i');\n\
             var e = document.getElementById('foo');\n\
             // ① indexed delete no-op（loose）——值保留\n\
             var before = c[0] === e;\n\
             delete c[0];\n\
             globalThis.__d1 = before && (c[0] === e);\n\
             // ① strict delete 抛 TypeError\n\
             try { (function(){ 'use strict'; delete c[0]; })(); globalThis.__d2 = 'no-throw'; }\n\
             catch (err) { globalThis.__d2 = err instanceof TypeError ? 'TypeError' : String(err); }\n\
             // ② named getter + delete no-op\n\
             globalThis.__d3 = c.foo === e;\n\
             delete c.foo;\n\
             globalThis.__d3b = c.foo === e;\n\
             try { (function(){ 'use strict'; delete c.foo; })(); globalThis.__d4 = 'no-throw'; }\n\
             catch (err) { globalThis.__d4 = err instanceof TypeError ? 'TypeError' : String(err); }\n\
             // ③ 数字 id 不经 named 暴露（length 不被推大）\n\
             var k = document.getElementsByClassName('k');\n\
             globalThis.__d5 = k.length;\n\
             var ids = [];\n\
             for (var i = 0; i < k.length; i++) ids.push(k[i] && k[i].id);\n\
             globalThis.__d6 = ids.join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__d1").unwrap().value,
        "true",
        "① loose delete c[0] no-op——元素仍可读（无挖洞）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__d2").unwrap().value,
        "TypeError",
        "① strict delete c[0] 抛 TypeError（不可配置）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__d3").unwrap().value,
        "true",
        "② c.foo named getter 命中带 id 元素"
    );
    assert_eq!(
        sandbox.execute("globalThis.__d3b").unwrap().value,
        "true",
        "② delete c.foo no-op——named 属性保留"
    );
    assert_eq!(
        sandbox.execute("globalThis.__d4").unwrap().value,
        "TypeError",
        "② strict delete c.foo 抛 TypeError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__d5").unwrap().value,
        "2",
        "③ 数字 id 不推大 length（2 个 .k 元素）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__d6").unwrap().value,
        "1,2",
        "③ indexed 顺序读不受 named 干扰（ids=1,2）"
    );
}

#[test]
fn test_namednodemap_own_enumeration_r44() {
    // R44：spec `dom-namednodemap-supported-property-names`——NamedNodeMap own keys =
    // 数值索引（"0","1",…）+ 属性名（id/class/…）。WPT namednodemap-supported-property-names
    // 断言 `Object.getOwnPropertyNames(el.attributes)` === [indices..., names...]。旧实现
    // Proxy({}) 无 ownKeys/getOwnPropertyDescriptor trap → 恒 []。
    // ① getOwnPropertyNames 返 [indices + names] 文档序
    // ② named 索引访问（.attributes.id.value）经 descriptor 可见
    // ③ for-in 枚举到全部（enumerable descriptor）
    // ④ 移除属性后枚举收缩
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
        "<html><body><div id=\"d\" class=\"c1\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var d = document.querySelector('#d');\n\
             d.setAttribute('data-x', 'v1');\n\
             var attrs = d.attributes;\n\
             // ① getOwnPropertyNames：indices + names（文档序：id, class, data-x）\n\
             globalThis.__o1 = Object.getOwnPropertyNames(attrs).join(',');\n\
             // ② named 访问经 descriptor\n\
             globalThis.__o2 = attrs.id ? attrs.id.value : 'none';\n\
             // ③ for-in 枚举\n\
             var seen = [];\n\
             for (var k in attrs) seen.push(k);\n\
             globalThis.__o3 = seen.join(',');\n\
             // ④ 移除后收缩\n\
             d.removeAttribute('data-x');\n\
             globalThis.__o4 = Object.getOwnPropertyNames(attrs).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__o1").unwrap().value,
        "0,1,2,id,class,data-x",
        "① getOwnPropertyNames = [indices + names] 文档序"
    );
    assert_eq!(
        sandbox.execute("globalThis.__o2").unwrap().value,
        "d",
        "② named 访问（attrs.id.value）经 descriptor 可见"
    );
    assert_eq!(
        sandbox.execute("globalThis.__o3").unwrap().value,
        "0,1,2",
        "③ for-in 只枚举数值索引（R96：named 属性 descriptor enumerable:false——spec named \
        properties 平台对象枚举语义，WPT attributes.html getEnumerableOwnProps1 仲裁；\
        getOwnPropertyNames ① 仍含 named，不依赖 enumerability）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__o4").unwrap().value,
        "0,1,id,class",
        "④ removeAttribute 后枚举收缩（indices + 剩余 names）"
    );
}

#[test]
fn test_mutation_observer_id_chain_and_oldvalue_r45() {
    // R45：MutationObserver 语义修复双件：
    // ① 同批 id 重命名追链（Rust apply_dom_mutations）：`el.id="abc"; el.className="x"` 两条 mutation
    //    的 selector 均取自 proxy 建立时（#old），第一条应用后 #old 消失——rewrite_pending_id_selectors
    //    把剩余队列 #old → #abc，第二批不再整批崩（WPT MutationObserver-attributes 曾整用例
    //    "set_attr: no match" 崩）。非 id 的 stale selector 仍走原错误路径（不掩盖真 bug）。
    // ② attributeOldValue：IDL 反射 setter（id/className/title/lang/type）与 classList write 在
    //    **写入前**捕获 old value（WPT "oldValue didn't match" 全族）。旧实现 notify 恒不带 old。
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
        "<html><body><div id=\"old\" class=\"c0\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var recs = [];
             var mo = new MutationObserver(function(rs) { recs = rs; });
             var el = document.querySelector('#old');
             mo.observe(el, { attributes: true, attributeOldValue: true });
             // ① id 改名 + 同元素后续 mutation（同批追链）
             el.id = 'newid';
             el.className = 'c1';",
        )
        .unwrap();
    // flush 经 `_defer`（microtask）。沙箱无 host 事件循环——V8 microtask 在 execute 返回后
    // 由 sandbox 内部泵（persistent context 每次 execute 后跑 pending microtasks）。多轮 execute
    // 轮询直到 recs 填充（上限 50 轮）。
    let mut filled = false;
    for _ in 0..50 {
        if sandbox.execute("recs.length").unwrap().value == "2" {
            filled = true;
            break;
        }
        let _ = sandbox.execute("0");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(filled, "MO 回调应经 microtask flush 填充 recs（2 条）");
    assert_eq!(
        sandbox
            .execute("recs.map(function(r) { return r.attributeName + '=' + r.oldValue; }).join(',')")
            .unwrap()
            .value,
        "id=old,class=c0",
        "② attributeOldValue 写入前捕获（id=old / class=c0）+ ① 两条都入 record"
    );
}

#[test]
fn test_mutation_observer_ns_and_no_mutation_r46() {
    // R46：MutationObserver record 语义三件（WPT MutationObserver-attributes 0→38P/0F 驱动）：
    // ① setAttributeNS record：attributeName=**localName** + attributeNamespace=ns（旧委托
    //    setAttribute 使 record 带限定名 "xml:lang" 且 namespace null）
    // ② removeAttribute 缺失属性**不发 record**（spec：queue record 仅当已存在属性被移除；
    //    旧无条件 notify 致 "removal no mutation" 多一条）
    // ③ classList.add 已存在 token 仍发 record（spec update 步骤 8 仍 set attribute）+
    //    classList.remove 到空集且原属性缺失不写不 notify（remove 不得创建空属性）
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
        "<html><body><div id=\"a\" class=\"c1\">x</div><div id=\"b\">y</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var recs = [];\n\
             var mo = new MutationObserver(function(rs) { recs = rs; });\n\
             var a = document.querySelector('#a');\n\
             mo.observe(a, { attributes: true, attributeOldValue: true });\n\
             // ① setAttributeNS：localName + namespace\n\
             a.setAttributeNS('http://example.org/', 'xml:lang', 'en');\n\
             a.setAttributeNS('http://example.org/ns2', 'title2', 'v');",
        )
        .unwrap();
    let mut filled = false;
    for _ in 0..50 {
        if sandbox.execute("recs.length").unwrap().value == "2" {
            filled = true;
            break;
        }
        let _ = sandbox.execute("0");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert!(filled, "NS 两条 record 应 flush");
    assert_eq!(
        sandbox
            .execute("recs.map(function(r) { return r.attributeName + '|' + String(r.attributeNamespace); }).join(',')")
            .unwrap()
            .value,
        "lang|http://example.org/,title2|http://example.org/ns2",
        "① setAttributeNS record：attributeName=localName + attributeNamespace=ns（prefixed 限定名拆解）"
    );
    // ② removeAttribute 缺失属性 + ③ classList 语义
    sandbox
        .execute(
            "recs = [];\n\
             var b = document.querySelector('#b');\n\
             var mo2 = new MutationObserver(function(rs) { recs = rs; });\n\
             mo2.observe(b, { attributes: true, attributeOldValue: true });\n\
             b.removeAttribute('class');\n\
             b.classList.remove('nonexistent');\n\
             globalThis.__phase2 = true;",
        )
        .unwrap();
    for _ in 0..50 {
        if sandbox.execute("recs.length").unwrap().value != "0" || sandbox.execute("globalThis.__phase2").unwrap().value == "true" {
            break;
        }
        let _ = sandbox.execute("0");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    // flush 后 recs 应仍 0（缺失移除 + 空集 remove 都不发）
    std::thread::sleep(std::time::Duration::from_millis(50));
    let _ = sandbox.execute("0");
    assert_eq!(
        sandbox.execute("recs.length").unwrap().value,
        "0",
        "②③ removeAttribute 缺失 + classList.remove 空集（原无 class）均不发 record"
    );
    // ③ classList.add 已存在 token 仍发
    sandbox
        .execute(
            "recs = [];\n\
             a.classList.add('c1');\n\
             globalThis.__phase3 = true;",
        )
        .unwrap();
    for _ in 0..50 {
        if sandbox.execute("recs.length").unwrap().value == "1" {
            break;
        }
        let _ = sandbox.execute("0");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        sandbox.execute("recs.length").unwrap().value,
        "1",
        "③ classList.add 已存在 token 仍发 attributes record（spec update 步骤 8 仍 set）"
    );
}

#[test]
fn test_mutation_observer_childlist_fragment_r47() {
    // R47：childList record 语义（WPT MutationObserver-childList 10P→15P/1F 驱动）：
    // ① appendChild/insertBefore(fragment) 的 addedNodes = fragment **子节点**（flatten 前快照；
    //    spec fragment 自身不入树不出现在 record）
    // ② record.previousSibling/nextSibling：appendChild prev=写入前容器 lastChild / next=null；
    //    insertBefore prev=refNode 前兄弟 / next=refNode
    // ③ el.remove() 发 childList removed record（旧缺——surroundContents 多 record 依赖）
    // ④ surroundContents 逐 removed 正序 record（每 child 一条）+ added 一条
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
        "<html><body><div id=\"host\"><span id=\"s0\">a</span></div><div id=\"rm\"><i id=\"r1\">x</i><i id=\"r2\">y</i></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var recs = [];\n\
             var host = document.querySelector('#host');\n\
             var mo = new MutationObserver(function(rs) { recs = rs; });\n\
             mo.observe(host, { childList: true });\n\
             // ① fragment append：addedNodes = 子节点数组\n\
             var f = document.createDocumentFragment();\n\
             var b1 = document.createElement('b'); b1.id = 'b1';\n\
             var b2 = document.createElement('b'); b2.id = 'b2';\n\
             f.appendChild(b1); f.appendChild(b2);\n\
             host.appendChild(f);",
        )
        .unwrap();
    for _ in 0..50 {
        if sandbox.execute("recs.length").unwrap().value == "1" {
            break;
        }
        let _ = sandbox.execute("0");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        sandbox
            .execute("recs[0] && recs[0].addedNodes.length + ',' + String(recs[0].addedNodes[0] === b1) + ',' + String(recs[0].addedNodes[1] === b2)")
            .unwrap()
            .value,
        "2,true,true",
        "① appendChild(fragment) record.addedNodes = [b1, b2]（fragment 子节点，非 fragment 自身）"
    );
    assert_eq!(
        sandbox.execute("String(recs[0].previousSibling === document.querySelector('#s0'))").unwrap().value,
        "true",
        "② appendChild record.previousSibling = 写入前容器 lastChild（#s0）"
    );
    // ③ el.remove() record
    sandbox
        .execute(
            "recs = [];\n\
             var rmDiv = document.querySelector('#rm');\n\
             mo.observe(rmDiv, { childList: true });\n\
             document.querySelector('#r1').remove();",
        )
        .unwrap();
    for _ in 0..50 {
        if sandbox.execute("recs.length").unwrap().value == "1" {
            break;
        }
        let _ = sandbox.execute("0");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        sandbox
            .execute("recs[0] && recs[0].type + ',' + String(recs[0].removedNodes[0] === document.querySelector('#r1'))")
            .unwrap()
            .value,
        "childList,true",
        "③ el.remove() 发 childList removed record（removedNodes=[自身]）"
    );
}

#[test]
fn test_parsed_text_characterdata_r48() {
    // R48：parsed DOM 文本节点的 CharacterData 编辑 + MutationObserver record（WPT
    // MutationObserver-characterData 4P/12F→18P/0F 驱动）：
    // ① parsed 文本节点（<p>firstChild，_wrapNodeEntry 普通对象）具备 appendData/insertData/
    //    deleteData/replaceData/substringData + data/nodeValue setter
    // ② 编辑经「父 selector + childNodes 索引」写入 host（SetChildText mutation→__zw_set_child_text）
    // ③ observe(文本节点) 落到父元素 id；characterData record 携带 characterDataOldValue 的写前 oldValue
    // ④ 本地 data 同步（同块读不 stale）
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
        "<html><body><p id=\"t\">CHAN</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var recs = [];\n\
             var tn = document.querySelector('#t').firstChild;\n\
             var mo = new MutationObserver(function(rs) { recs = rs; });\n\
             mo.observe(tn, { characterData: true, characterDataOldValue: true });\n\
             tn.appendData('GED');",
        )
        .unwrap();
    for _ in 0..50 {
        if sandbox.execute("recs.length").unwrap().value == "1" {
            break;
        }
        let _ = sandbox.execute("0");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        sandbox.execute("recs[0] && recs[0].type + ',' + recs[0].oldValue").unwrap().value,
        "characterData,CHAN",
        "③ record type=characterData + oldValue 写前捕获（CHAN）"
    );
    assert_eq!(
        sandbox.execute("tn.data").unwrap().value,
        "CHANGED",
        "④ 本地 data 同步（appendData 后同块读不 stale）"
    );
    // ② mutation 队列有 SetChildText（父 sel + 索引 + 新文本）
    sandbox
        .execute(
            "tn.deleteData(0, 6);\n\
             tn.insertData(0, 'X');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("tn.data").unwrap().value,
        "XD",
        "① deleteData/insertData 组合（CHANGED 删 [0,6) 剩 D → 前插 X → XD）"
    );
    assert_eq!(
        sandbox.execute("tn.substringData(0, 1)").unwrap().value,
        "X",
        "① substringData 读"
    );
}

#[test]
fn test_mo_observe_options_and_textcontent_records_r49() {
    // R49：MutationObserver 语义四件（WPT sanity 11→16P/0F + takeRecords 1→3P/0F + attributes
    // 38→40P/0F + childList 15→18P/1F 驱动）：
    // ① observe options 校验（全缺抛/oldValue 矛盾抛/filter 矛盾抛）
    // ② 隐含启用——attributeOldValue/attributeFilter/characterDataOldValue 存在即隐含对应观测
    // ③ attributeFilter alone 不提供 oldValue（仅 attributeOldValue===true）
    // ④ textContent=——异值发 childList（removed=旧子 + added=[新文本节点]）不发 characterData；
    //    firstChild 立即可见且 data= 可编辑（characterData oldValue=写前值 target=文本节点）；
    //    同值 no-op（本地注册文本优先判等）
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
        "<html><body><p id=\"t\">old</p><p id=\"e\"></p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var t = document.querySelector('#t');\n\
             var mk = function() { return new MutationObserver(function(){}); };\n\
             try { mk().observe(t, {}); globalThis.__v1 = 'no-throw'; }\n\
             catch (e) { globalThis.__v1 = e instanceof TypeError ? 'TypeError' : String(e); }\n\
             try { mk().observe(t, { childList: true, attributeOldValue: true, attributes: false }); globalThis.__v2 = 'no-throw'; }\n\
             catch (e) { globalThis.__v2 = e instanceof TypeError ? 'TypeError' : String(e); }\n\
             try { mk().observe(t, { childList: true, attributeFilter: ['a'], attributes: false }); globalThis.__v3 = 'no-throw'; }\n\
             catch (e) { globalThis.__v3 = e instanceof TypeError ? 'TypeError' : String(e); }\n\
             try { mk().observe(t, { attributeOldValue: true }); mk().observe(t, { attributeFilter: ['a'] }); mk().observe(t, { characterDataOldValue: true }); globalThis.__v4 = 'ok'; }\n\
             catch (e) { globalThis.__v4 = 'threw:' + String(e); }",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__v1").unwrap().value, "TypeError", "① 全缺 options 抛 TypeError");
    assert_eq!(sandbox.execute("globalThis.__v2").unwrap().value, "TypeError", "① attributeOldValue true + attributes false 抛");
    assert_eq!(sandbox.execute("globalThis.__v3").unwrap().value, "TypeError", "① attributeFilter + attributes false 抛");
    assert_eq!(sandbox.execute("globalThis.__v4").unwrap().value, "ok", "② 隐含启用三形态不抛");

    sandbox
        .execute(
            "var recs = [];\n\
             var e = document.querySelector('#e');\n\
             var mo = new MutationObserver(function(rs) { recs = rs; });\n\
             mo.observe(e, { childList: true, characterData: true, characterDataOldValue: true });\n\
             e.textContent = 'old data';\n\
             e.firstChild.data = 'new data';",
        )
        .unwrap();
    for _ in 0..50 {
        if sandbox.execute("recs.length").unwrap().value == "2" {
            break;
        }
        let _ = sandbox.execute("0");
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    assert_eq!(
        sandbox.execute("recs.map(function(r) { return r.type; }).join(',')").unwrap().value,
        "childList,characterData",
        "④ textContent= 发 childList；firstChild.data= 发 characterData（各一条）"
    );
    assert_eq!(
        sandbox.execute("recs[1].oldValue + '/' + String(recs[1].target === e.firstChild)").unwrap().value,
        "old data/true",
        "④ characterData oldValue=写前值 + target=文本节点"
    );
    // 同值 no-op（data= 后 textContent=同值）
    sandbox.execute("recs = []; e.textContent = 'new data';").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(60));
    let _ = sandbox.execute("0");
    assert_eq!(
        sandbox.execute("recs.length").unwrap().value,
        "0",
        "④ 同值 textContent= 不发 record（本地注册文本优先判等）"
    );
}

#[test]
fn test_event_subclass_constructors_r109() {
    // R109（WPT Event-subclasses-constructors）：① `class X extends Event` 的 super() 须以
    // [[Construct]] 语义填充 new.target 的 this——旧工厂返对象形态下子类 ctor 体 `this.customProp=5`
    // 抛 TypeError / 实例 instanceof 子类 false。② 拷贝须含非枚举 accessor（cancelBubble/
    // returnValue/srcElement——for-in 漏它们会读 undefined）。③ UIEvent view 非 WindowProxy 抛
    // TypeError（WebIDL dictionary 校验）。
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
        "<html><body><div id=\"p\">x</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① class extends Event：super() 后 this 可用、instanceof 双向、子类 getter 落到实例。
    sandbox
        .execute(
            "self.SubE = class SubE extends Event {\
               constructor(n, p) {\
                 super(n, p);\
                 this.customProp = (p && typeof p == 'object' && 'customProp' in p) ? p.customProp : 5;\
               }\
               get fixedProp() { return 17; }\
             };\
             var ev = new SubE('type', { customProp: 8 });\
             globalThis.__r109a = [ev instanceof SubE, ev instanceof Event, ev.customProp, ev.fixedProp, ev.type].join(',');\
             var ev2 = new SubE('t');\
             globalThis.__r109b = ev2.customProp;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r109a").unwrap().value,
        "true,true,8,17,type",
        "① class extends Event：instanceof 子类/父类 + customProp/fixedProp"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r109b").unwrap().value,
        "5",
        "① 缺省 customProp = 5"
    );

    // ② 拷贝含非枚举 accessor：cancelBubble/returnValue/srcElement 在子类实例上可读（非 undefined）。
    sandbox
        .execute(
            "var ev3 = new SubE('t');\
             globalThis.__r109c = [typeof ev3.cancelBubble, typeof ev3.returnValue, typeof ev3.srcElement].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r109c").unwrap().value,
        "boolean,boolean,object",
        "② cancelBubble(bool)/returnValue(bool)/srcElement(null→object) 非 undefined"
    );
    // stopPropagation → cancelBubble getter 联动（accessor 后端 _propagationStopped 已搬运）。
    sandbox
        .execute("ev3.stopPropagation(); globalThis.__r109d = String(ev3.cancelBubble);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r109d").unwrap().value,
        "true",
        "② stopPropagation 后 cancelBubble=true（accessor 联动）"
    );

    // ③ UIEvent view 非 WindowProxy（数字 7）→ TypeError（WPT "view argument with wrong type"）。
    sandbox
        .execute(
            "globalThis.__r109e = (function(){\
               try { new UIEvent('x', { view: 7 }); return 'no-throw'; }\
               catch (e) { return e instanceof TypeError ? 'TypeError' : String(e); }\
             })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r109e").unwrap().value,
        "TypeError",
        "③ UIEvent view=7 抛 TypeError"
    );
    // view 合法值（window / null / 缺省）不抛。
    sandbox
        .execute(
            "globalThis.__r109f = (function(){\
               try {\
                 new UIEvent('x', { view: null });\
                 new UIEvent('x');\
                 new UIEvent('x', { view: window });\
                 return 'ok';\
               } catch (e) { return String(e); }\
             })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r109f").unwrap().value,
        "ok",
        "③ view=null/缺省/window 均不抛"
    );

    // ④ 事件基类回归：new Event() 仍 instanceof Event、dispatch 语义不破坏。
    sandbox
        .execute(
            "var ev4 = new Event('e', { bubbles: true, cancelable: true });\
             globalThis.__r109g = [ev4 instanceof Event, ev4.bubbles, ev4.cancelable, ev4.cancelBubble].join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r109g").unwrap().value,
        "true,true,true,false",
        "④ new Event() 基类行为不回归"
    );
}
