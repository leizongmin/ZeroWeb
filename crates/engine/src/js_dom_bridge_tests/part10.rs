#[test]
fn test_reflected_global_attrs_inert_autocomplete_r2850() {
    // R2850：reflected 全局属性 inert（boolean attr，缺省 false）/ autocomplete（enumerated 串，缺省 "on"）。
    // 旧 fallthrough 返 undefined。inert 同 autofocus（presence）；autocomplete 缺省 → "on"（spec missing-default）。
    // 模态/无障碍（inert 隔离交互）/ 表单自动填充（autocomplete）读这些属性高频。延续 R2848 模式。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // div[inert] present；input[autocomplete="off"]；plain 无两属性。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <div id='d' inert></div>\
         <input id='a' autocomplete='off'>\
         <div id='plain'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 读：div[inert] present → inert=true；input[autocomplete="off"] → "off"；plain 缺省 inert=false / autocomplete="on"。
    sandbox
        .execute(
            "globalThis.__di = document.querySelector('#d').inert;\
             globalThis.__aa = document.querySelector('#a').autocomplete;\
             var p = document.querySelector('#plain');\
             globalThis.__pi = p.inert;\
             globalThis.__pa = p.autocomplete;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__di)").unwrap().value,
        "true",
        "div[inert] present → inert=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__aa)").unwrap().value,
        "off",
        "input[autocomplete='off'] → autocomplete='off'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pi)").unwrap().value,
        "false",
        "plain inert 缺省 → false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pa)").unwrap().value,
        "on",
        "plain autocomplete 缺省 → 'on'（spec missing-default）"
    );

    // setter：同步 set→get 优先读缓存（即时）。inert=true→presence；autocomplete='given-name'→attr 串。
    sandbox
        .execute(
            "var e = document.querySelector('#plain');\
             e.inert = true; e.autocomplete = 'given-name';\
             globalThis.__si = e.inert;\
             globalThis.__sa = e.autocomplete;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__si)").unwrap().value,
        "true",
        "setter inert=true → true（缓存即时）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sa)").unwrap().value,
        "given-name",
        "setter autocomplete='given-name' → 'given-name'（任意值写 attr）"
    );

    // apply mutations → 核验 attr 写回（inert presence / autocomplete='given-name'）。
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(out.contains("id=\"plain\" inert"), "inert setter 写 presence\n{out}");
    assert!(
        out.contains("autocomplete=\"given-name\""),
        "autocomplete setter 写 'given-name'\n{out}"
    );
}

#[test]
fn test_img_dimension_idl_width_height_natural_r2851() {
    // R2851：IMG/IFRAME width/height（reflected unsigned long，缺省/非负整数失败→0）+ IMG naturalWidth/Height
    // （固有像素尺寸，headless 无真图加载→0，spec unloaded→0）。旧 fallthrough 返 undefined。响应式/布局 JS 高频。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // img 显式 width/height；img2 无属性；iframe 显式 width。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <img id='i1' src='a.png' width='100' height='50'>\
         <img id='i2' src='b.png'>\
         <iframe id='f1' width='320'></iframe>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 读：i1 width=100/height=50；i2 缺省 width=0/height=0；naturalWidth/Height 恒 0（headless）；iframe width=320。
    sandbox
        .execute(
            "var i1 = document.querySelector('#i1');\
             globalThis.__i1w = i1.width;\
             globalThis.__i1h = i1.height;\
             globalThis.__i1nw = i1.naturalWidth;\
             globalThis.__i1nh = i1.naturalHeight;\
             var i2 = document.querySelector('#i2');\
             globalThis.__i2w = i2.width;\
             globalThis.__i2h = i2.height;\
             globalThis.__f1w = document.querySelector('#f1').width;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__i1w)").unwrap().value,
        "100",
        "img[width='100'] → width=100（reflected unsigned long）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i1h)").unwrap().value,
        "50",
        "img[height='50'] → height=50"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i1nw)").unwrap().value,
        "0",
        "img.naturalWidth=0（headless 无真图加载，spec unloaded→0）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i1nh)").unwrap().value,
        "0",
        "img.naturalHeight=0（headless）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i2w)").unwrap().value,
        "0",
        "img 无 width 属性 → width=0（缺省）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__i2h)").unwrap().value,
        "0",
        "img 无 height 属性 → height=0（缺省）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__f1w)").unwrap().value,
        "320",
        "iframe[width='320'] → width=320（IFRAME 同 reflected unsigned long）"
    );

    // setter：img.width=200 → 缓存数值即时 + apply 后 attr 写回；非负整数解析（'12px'→12 近似）。
    sandbox
        .execute(
            "var e = document.querySelector('#i2');\
             e.width = 200;\
             globalThis.__sw = e.width;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sw)").unwrap().value,
        "200",
        "setter img.width=200 → 200（缓存数值即时 sync）"
    );
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("id=\"i2\" src=\"b.png\" width=\"200\""),
        "img.width=200 setter 写 width 内容属性\n{out}"
    );
}

#[test]
fn test_document_content_type_and_node_normalize_r2853() {
    // R2853：document.contentType（'text/html'，spec HTML 文档 MIME）+ Node.normalize()（no-op，
    // snapshot 模型文本为单一串故语义正确——DOM 态已 normalized，防防御性调用抛 TypeError）。
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
        "<html><body><div id='d'>hello</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // document.contentType = 'text/html'（spec HTML 文档）。
    sandbox.execute("globalThis.__ct = document.contentType;").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ct)").unwrap().value,
        "text/html",
        "document.contentType = 'text/html'（HTML 文档 MIME）"
    );

    // Node.normalize()：可调用（不抛 TypeError），返 undefined（spec void），文本不变。
    sandbox
        .execute(
            "var d = document.querySelector('#div');\
             globalThis.__normReturn = document.querySelector('#d').normalize();\
             globalThis.__tc = document.querySelector('#d').textContent;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__normReturn)").unwrap().value,
        "undefined",
        "normalize() 返 undefined（spec void）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tc)").unwrap().value,
        "hello",
        "normalize() no-op：textContent 不变（'hello'）"
    );

    // 多元素 normalize() 均可调用（不抛）——防 rich-text 编辑器 / innerHTML 后清理的防御性调用崩溃。
    sandbox
        .execute(
            "var ok = true;\
             try { document.body.normalize(); document.documentElement.normalize(); } catch (e) { ok = false; }\
             globalThis.__allOk = ok;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__allOk)").unwrap().value,
        "true",
        "body/documentElement.normalize() 均可调用（不抛 TypeError）"
    );
}

#[test]
fn test_node_is_connected_and_has_child_nodes_r2922() {
    // R2922：Node.isConnected（只读 boolean，节点是否连入 document）+ Node.hasChildNodes()（是否有任意
    // 子节点含文本/注释）。两者为 Node 接口最高频判活 / 子存在性 API（jQuery cleanData、React commit、
    // mutation handler、树遍历 diff），旧 shim 完全缺失 → isConnected 恒 undefined（falsy）误判在档元素为
    // detached。isConnected：sel-based 经 __zw_contains('html', sel)（element_contains 自含，html 自身命中）
    // 判定在 documentElement 子树内，亦正确反映 removeChild 后 detach；handle-only（createElement 等）→ false。
    // hasChildNodes：经 _childNodeList length>0。Document literal 恒 connected + 恒有 documentElement 子。
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
        "<html><body><div id='d'>hello</div><span id='empty'></span></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ── Document 节点：nodeType=9 / nodeName='#document' / 恒 connected / 恒有子。──
    sandbox
        .execute(
            "globalThis.__docNt = document.nodeType;\
             globalThis.__docNn = document.nodeName;\
             globalThis.__docConn = document.isConnected;\
             globalThis.__docHcn = document.hasChildNodes();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__docNt)").unwrap().value,
        "9",
        "document.nodeType = 9（DOCUMENT_NODE）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__docNn)").unwrap().value,
        "#document",
        "document.nodeName = '#document'"
    );
    assert_eq!(
        sandbox.execute("globalThis.__docConn").unwrap().value,
        "true",
        "document.isConnected = true（根节点恒连入）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__docHcn").unwrap().value,
        "true",
        "document.hasChildNodes() = true（恒有 documentElement）"
    );

    // ── isConnected：sel-based 在档元素（含 documentElement/body/查询结果）= true。──
    sandbox
        .execute(
            "globalThis.__htmlConn = document.documentElement.isConnected;\
             globalThis.__bodyConn = document.body.isConnected;\
             globalThis.__headConn = document.head.isConnected;\
             globalThis.__dConn = document.querySelector('#d').isConnected;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__htmlConn").unwrap().value, "true");
    assert_eq!(sandbox.execute("globalThis.__bodyConn").unwrap().value, "true");
    assert_eq!(sandbox.execute("globalThis.__headConn").unwrap().value, "true");
    assert_eq!(
        sandbox.execute("globalThis.__dConn").unwrap().value,
        "true",
        "querySelector('#d').isConnected = true（在档）"
    );

    // ── isConnected：handle-only 节点（createElement/createTextNode/createFragment 未挂载）= false。──
    // 注：register_dom_callbacks 不注 __zw_getBoundingClientRect，故 handle-only 无 probe 路径 → false。
    sandbox
        .execute(
            "globalThis.__elConn = document.createElement('div').isConnected;\
             globalThis.__tnConn = document.createTextNode('x').isConnected;\
             globalThis.__fragConn = document.createDocumentFragment().isConnected;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__elConn").unwrap().value,
        "false",
        "createElement('div').isConnected = false（detached）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__tnConn").unwrap().value,
        "false",
        "createTextNode('x').isConnected = false（detached）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fragConn").unwrap().value,
        "false",
        "createDocumentFragment().isConnected = false（恒 detached）"
    );

    // ── hasChildNodes：有子（#d 含 'hello' 文本节点）/ body（含 #d·#empty·文本）= true；
    //    空元素（#empty）/ handle-only createElement = false。──
    sandbox
        .execute(
            "globalThis.__bodyHcn = document.body.hasChildNodes();\
             globalThis.__dHcn = document.querySelector('#d').hasChildNodes();\
             globalThis.__emptyHcn = document.querySelector('#empty').hasChildNodes();\
             globalThis.__elHcn = document.createElement('div').hasChildNodes();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__bodyHcn").unwrap().value,
        "true",
        "body.hasChildNodes() = true（含子元素 + 文本）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dHcn").unwrap().value,
        "true",
        "#d.hasChildNodes() = true（含 'hello' 文本节点）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__emptyHcn").unwrap().value,
        "false",
        "#empty.hasChildNodes() = false（无子）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__elHcn").unwrap().value,
        "false",
        "createElement('div').hasChildNodes() = false（detached 无子）"
    );

    // ── isConnected：removeChild 后 detach（sel-based 经 __zw_contains 反映在档态）。──
    // 先捕获 #d proxy（sel='#d'），再换 snapshot 为不含 #d 的 html（模拟 removeChild 已应用），
    // 旧 proxy.isConnected 应翻 false（__zw_contains('html','#d') 读新 snapshot → '0'）。**置于末尾**：
    // 换 snapshot 移除 #d，破坏后续 #d 查询，故 hasChildNodes 等须在此之前完成。
    sandbox
        .execute("globalThis.__dRef = document.querySelector('#d'); globalThis.__connBefore = globalThis.__dRef.isConnected;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__connBefore").unwrap().value, "true");
    *dom_html.lock().unwrap() = "<html><body><span id='empty'></span></body></html>".to_string();
    sandbox
        .execute("globalThis.__connAfter = globalThis.__dRef.isConnected;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__connAfter").unwrap().value,
        "false",
        "#d 移出 document 后 isConnected = false（__zw_contains 反映 detach）"
    );
}

/// R2922：`window.onload = fn` 事件处理器 IDL 语义——赋值等价注册 load 监听，
/// `__zw_dispatch_event('html','load')` 派发时触发（driving:
/// css-overflow/line-clamp/webkit-line-clamp-019 的 `window.onload` 动态改样式）。
#[test]
fn test_window_onload_assignment_registers_load_listener() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='t'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // window.onload 赋值（须在派发前注册）。
    sandbox
        .execute("window.onload = function() { document.querySelector('#t').style.color = 'red'; };")
        .unwrap();
    // 派发 load 事件 → onload 回调应执行 → 入队 SetStyle mutation。
    sandbox.execute("__zw_dispatch_event('html','load',null);").unwrap();

    let ms = mutations.lock().unwrap();
    let style_mutations: Vec<_> = ms
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetStyle {
                selector,
                property,
                value,
            } => Some((selector.as_str(), property.as_str(), value.as_str())),
            _ => None,
        })
        .collect();
    assert!(
        style_mutations.contains(&("#t", "color", "red")),
        "window.onload 回调须产生 SetStyle color=red，实际 {style_mutations:?}"
    );
}

/// R2922：`el.style.webkitLineClamp = '6'` 按 CSSOM vendor 前缀规则归一为 CSS 属性
/// `-webkit-line-clamp`（通用 camelCase→kebab 会产 `webkit-line-clamp`——丢前导 `-`，
/// CSS parser 不认 → 渲染静默失效；driving: webkit-line-clamp-019）。
#[test]
fn test_style_webkit_prefix_property_normalized_with_leading_hyphen() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body><div id='t'></div></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute("document.querySelector('#t').style.webkitLineClamp = '6';")
        .unwrap();

    let ms = mutations.lock().unwrap();
    let style_mutations: Vec<_> = ms
        .iter()
        .filter_map(|m| match m {
            DomMutation::SetStyle {
                selector,
                property,
                value,
            } => Some((property.as_str(), value.as_str())),
            _ => None,
        })
        .collect();
    assert!(
        style_mutations.contains(&("-webkit-line-clamp", "6")),
        "webkitLineClamp 须归一为 -webkit-line-clamp（带前导连字符），实际 {style_mutations:?}"
    );
}

#[test]
fn test_readable_stream_r2967() {
    // R2967：ReadableStream（Streams API）。纯 JS 控制器模型：push 源（start-enqueue-close）+
    // getReader().read() 序列（{done:false,value} × N → {done:true}）+ locked 守卫 + releaseLock +
    // async iterator（for await of）+ pull 源（queue 空 read 时触发 source.pull）。
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

    // push 源：enqueue 'a'/'b' + close；getReader 后 locked=true；read 三次 + releaseLock 后 locked=false。
    sandbox
        .execute(
            "var s = new ReadableStream({ start: function(c){ c.enqueue('a'); c.enqueue('b'); c.close(); } });\
             var r = s.getReader();\
             globalThis.__lockedA = s.locked;\
             r.read().then(function(v){ globalThis.__c1 = v; return r.read(); })\
                    .then(function(v){ globalThis.__c2 = v; return r.read(); })\
                    .then(function(v){ globalThis.__c3 = v; r.releaseLock(); globalThis.__lockedB = s.locked; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__lockedA)").unwrap().value,
        "true",
        "getReader 后 stream.locked=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c1 && globalThis.__c1.done)").unwrap().value,
        "false",
        "第 1 chunk: done=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c1 && globalThis.__c1.value)").unwrap().value,
        "a",
        "第 1 chunk: value='a'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c2 && globalThis.__c2.value)").unwrap().value,
        "b",
        "第 2 chunk: value='b'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c3 && globalThis.__c3.done)").unwrap().value,
        "true",
        "第 3 read: done=true（流关闭）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__lockedB)").unwrap().value,
        "false",
        "releaseLock 后 stream.locked=false"
    );

    // async iterator（for await of）：累加 1+2 → 3。
    sandbox
        .execute(
            "(async function(){\
               var s2 = new ReadableStream({ start: function(c){ c.enqueue(1); c.enqueue(2); c.close(); } });\
               var sum = 0;\
               for await (var x of s2) sum += x;\
               globalThis.__sum = sum;\
             })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sum)").unwrap().value,
        "3",
        "for await of 迭代流：1+2=3"
    );

    // pull 源：queue 空 read 时触发 source.pull；pull 2 次后 close。
    sandbox
        .execute(
            "var pulled = 0;\
             var s3 = new ReadableStream({ pull: function(c){ pulled++; if (pulled <= 2) c.enqueue('p'+pulled); else c.close(); } });\
             var r3 = s3.getReader();\
             r3.read().then(function(v){ globalThis.__p1 = v; return r3.read(); })\
                     .then(function(v){ globalThis.__p2 = v; return r3.read(); })\
                     .then(function(v){ globalThis.__p3 = v; globalThis.__pulledCount = pulled; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__p1 && globalThis.__p1.value)").unwrap().value,
        "p1",
        "pull 源第 1 chunk='p1'（read 触发 pull）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__p2 && globalThis.__p2.value)").unwrap().value,
        "p2",
        "pull 源第 2 chunk='p2'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__p3 && globalThis.__p3.done)").unwrap().value,
        "true",
        "pull 源第 3 read done=true（pull 第 3 次 close）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pulledCount)").unwrap().value,
        "3",
        "pull 源共触发 3 次 pull"
    );

    // locked 守卫：已 locked 时 getReader 抛 TypeError。
    sandbox
        .execute(
            "var s4 = new ReadableStream({ start: function(c){ c.close(); } });\
             s4.getReader();\
             try { s4.getReader(); globalThis.__dblLock = 'no-throw'; }\
             catch(e){ globalThis.__dblLock = String(e).indexOf('TypeError') >= 0 ? 'TypeError' : 'other'; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dblLock)").unwrap().value,
        "TypeError",
        "已 locked 时 getReader 抛 TypeError（spec）"
    );
}

#[test]
fn test_response_body_readable_stream_r2967() {
    // R2967：fetch response.body 为 ReadableStream。mock __zw_fetch 捕获 id，Rust 经
    // resolve_async_callback 投递 wire（status/statusText/headers/body），shim 包装为 Response；
    // response.body.getReader().read() 取 UTF-8 Uint8Array chunk → TextDecoder 解码 == body。
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

    // mock __zw_fetch：捕获 id（args[0]）供 Rust 侧 resolve。
    let captured_id: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let cap = Arc::clone(&captured_id);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *cap.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );

    // fetch 投递（pending）；host 异步 resolve → __resp 设置。
    sandbox
        .execute("fetch('http://test.local/data').then(function(r){ globalThis.__resp = r; });")
        .unwrap();
    let id = captured_id.lock().unwrap().clone();
    assert!(!id.is_empty(), "__zw_fetch 被调用且 id 已捕获");
    // wire：status=200 / statusText=OK / headersWire='' / body='Hello Streams'（\x1f 分隔）。
    let wire = "__zwfr:200\u{001f}OK\u{001f}\u{001f}Hello Streams";
    sandbox.resolve_async_callback(&id, wire);

    // 读 response.body 流：首 chunk 为 UTF-8 Uint8Array，TextDecoder 解码 == body；次 read done=true。
    sandbox
        .execute(
            "var r = globalThis.__resp.body.getReader();\
             globalThis.__hasBody = (globalThis.__resp.body !== null);\
             r.read().then(function(c){ globalThis.__chunk1 = c; return r.read(); })\
                    .then(function(c){ globalThis.__chunk2 = c; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hasBody)").unwrap().value,
        "true",
        "response.body 非 null（ReadableStream）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__chunk1 && globalThis.__chunk1.done)").unwrap().value,
        "false",
        "body 首 chunk: done=false"
    );
    assert_eq!(
        sandbox
            .execute("String(new TextDecoder().decode(globalThis.__chunk1.value))")
            .unwrap()
            .value,
        "Hello Streams",
        "body 首 chunk 经 TextDecoder 解码 == 'Hello Streams'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__chunk1 && globalThis.__chunk1.value && globalThis.__chunk1.value.length)")
            .unwrap()
            .value,
        "13",
        "body 首 chunk 字节长度=13（'Hello Streams' UTF-8）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__chunk2 && globalThis.__chunk2.done)").unwrap().value,
        "true",
        "body 次 read done=true（单 chunk 后 close）"
    );

    // 网络错误响应（ok:false）body=null（spec：network error 无 body）。
    let cap2: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let cap2c = Arc::clone(&cap2);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *cap2c.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute("fetch('http://test.local/err').then(function(r){ globalThis.__respErr = r; });")
        .unwrap();
    let id2 = cap2.lock().unwrap().clone();
    sandbox.resolve_async_callback(&id2, "__zw_fetch_error:network");
    sandbox
        .execute("globalThis.__errBody = (globalThis.__respErr.body === null ? 'null' : 'stream');")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__errBody)").unwrap().value,
        "null",
        "网络错误响应 body=null（spec network error）"
    );
}

#[test]
fn test_fetch_abort_signal_r3044() {
    // R3044：fetch 接通 AbortSignal。AbortController/AbortSignal 对象已就绪（part02），但 fetch 旧不消费 init.signal
    // → controller.abort() 无法中止在途 fetch。本切片接通：signal 已 aborted → 立即 reject；运行中 abort →
    // reject(signal.reason) + 清 __zw_pending[id]（host 结果到达 typeof-check no-op）。settled flag 防双 settle。
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

    // ① 运行中 abort：mock __zw_fetch 捕获 id 不 resolve；abort 后 fetch reject AbortError。
    let cap: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let capc = Arc::clone(&cap);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *capc.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute(
            "globalThis.__ctrl = new AbortController();\
             globalThis.__aborted = 'pending';\
             fetch('http://test.local/a', { signal: globalThis.__ctrl.signal })\
               .then(function(r){ globalThis.__aborted = 'resolved:' + r.status; })\
               .catch(function(e){ globalThis.__aborted = 'rejected:' + (e && e.name ? e.name : String(e)); });\
             globalThis.__ctrl.abort();",
        )
        .unwrap();
    sandbox.execute("1;").unwrap(); // drain microtask checkpoint
    assert_eq!(
        sandbox.execute("globalThis.__aborted").unwrap().value,
        "rejected:AbortError",
        "运行中 abort → fetch reject AbortError（旧：永不 reject）"
    );
    let id1 = cap.lock().unwrap().clone();
    assert!(!id1.is_empty(), "__zw_fetch 被调用且 id 已捕获");
    // abort 后 host 结果到达 → no-op（__zw_pending[id] 已清，__zwResolveCallback typeof-check）
    sandbox.resolve_async_callback(&id1, "__zwfr:200\u{001f}OK\u{001f}\u{001f}late");
    sandbox.execute("1;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__aborted").unwrap().value,
        "rejected:AbortError",
        "abort 后 host 结果到达 no-op（不覆写 reject）"
    );

    // ② 已 aborted signal → fetch 同步 reject（不调 __zw_fetch）。
    let cap2: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let cap2c = Arc::clone(&cap2);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *cap2c.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute(
            "globalThis.__pre = new AbortController();\
             globalThis.__pre.abort();\
             globalThis.__preResult = 'pending';\
             fetch('http://test.local/b', { signal: globalThis.__pre.signal })\
               .then(function(){ globalThis.__preResult = 'resolved'; })\
               .catch(function(e){ globalThis.__preResult = 'rejected:' + (e && e.name ? e.name : String(e)); });",
        )
        .unwrap();
    sandbox.execute("1;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__preResult").unwrap().value,
        "rejected:AbortError",
        "已 aborted signal → fetch reject（入口同步检查 signal.aborted）"
    );
    let id2 = cap2.lock().unwrap().clone();
    assert!(id2.is_empty(), "pre-aborted signal 不调 __zw_fetch（fetch 入口同步 reject）");

    // ③ abort(customReason) → fetch reject 传入的 reason（非 AbortError 包装，spec signal.reason 透传）。
    let cap3: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let cap3c = Arc::clone(&cap3);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *cap3c.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute(
            "globalThis.__c3 = new AbortController();\
             globalThis.__custom = 'pending';\
             fetch('http://test.local/c', { signal: globalThis.__c3.signal })\
               .catch(function(e){ globalThis.__custom = 'rejected:' + String(e); });\
             globalThis.__c3.abort('user-cancelled');",
        )
        .unwrap();
    sandbox.execute("1;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__custom").unwrap().value,
        "rejected:user-cancelled",
        "abort('user-cancelled') → fetch reject 传入 reason（signal.reason 透传，非 AbortError 包装）"
    );

    // ④ 回归守卫：signal present 但未 abort → fetch 正常 resolve（零回归）。
    let cap4: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let cap4c = Arc::clone(&cap4);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *cap4c.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute(
            "globalThis.__c4 = new AbortController();\
             globalThis.__ok = 'pending';\
             fetch('http://test.local/d', { signal: globalThis.__c4.signal })\
               .then(function(r){ globalThis.__ok = 'resolved:' + r.status; })\
               .catch(function(e){ globalThis.__ok = 'rejected:' + (e && e.name ? e.name : String(e)); });",
        )
        .unwrap();
    let id4 = cap4.lock().unwrap().clone();
    assert!(!id4.is_empty(), "signal present 未 abort → __zw_fetch 被调用");
    sandbox.resolve_async_callback(&id4, "__zwfr:200\u{001f}OK\u{001f}\u{001f}");
    sandbox.execute("1;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__ok").unwrap().value,
        "resolved:200",
        "signal present 未 abort → fetch 正常 resolve（零回归）"
    );
}

#[test]
fn test_request_signal_passthrough_r3045() {
    // R3045：Request.signal 透传。Request 构造器旧不存 signal → `fetch(new Request(url, {signal}))` 无法中止
    //（R3044 fetch 只读 init.signal，不读 Request 的 signal）。本切片 Request 存 signal + fetch 回落读 input.signal。
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

    // ① Request.signal 恒为 AbortSignal（spec 非空）——无 signal 构造 → 新建非 aborted signal。
    sandbox
        .execute(
            "globalThis.__r0 = new Request('http://test.local/x');\
             globalThis.__r0SigType = (globalThis.__r0.signal instanceof AbortSignal) ? 'AbortSignal' : 'other';\
             globalThis.__r0Aborted = String(globalThis.__r0.signal.aborted);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r0SigType").unwrap().value,
        "AbortSignal",
        "new Request(url).signal instanceof AbortSignal（spec 恒非空）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r0Aborted").unwrap().value,
        "false",
        "新 Request.signal.aborted=false"
    );

    // ② fetch(new Request(url, {signal})) + abort → reject AbortError（Request.signal 透传给 fetch）。
    let cap: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let capc = Arc::clone(&cap);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *capc.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute(
            "globalThis.__ctrl = new AbortController();\
             globalThis.__aborted = 'pending';\
             var req = new Request('http://test.local/a', { signal: globalThis.__ctrl.signal });\
             globalThis.__reqSigSame = (req.signal === globalThis.__ctrl.signal);\
             fetch(req)\
               .then(function(r){ globalThis.__aborted = 'resolved:' + r.status; })\
               .catch(function(e){ globalThis.__aborted = 'rejected:' + (e && e.name ? e.name : String(e)); });\
             globalThis.__ctrl.abort();",
        )
        .unwrap();
    sandbox.execute("1;").unwrap(); // drain microtask
    assert_eq!(
        sandbox.execute("globalThis.__reqSigSame").unwrap().value,
        "true",
        "new Request(url,{{signal}}).signal === 传入 controller.signal（透传同一对象）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__aborted").unwrap().value,
        "rejected:AbortError",
        "fetch(new Request(url,{{signal}})) + abort → reject AbortError（Request.signal 透传）"
    );

    // ③ new Request(otherRequest) 继承 signal（input.signal 回落到新 Request）。
    sandbox
        .execute(
            "globalThis.__ctrl2 = new AbortController();\
             var orig = new Request('http://test.local/b', { signal: globalThis.__ctrl2.signal });\
             var cloned = new Request(orig);\
             globalThis.__clonedSame = (cloned.signal === orig.signal);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__clonedSame").unwrap().value,
        "true",
        "new Request(otherReq) 继承 otherReq.signal（input.signal 回落）"
    );

    // ④ fetch(request, {signal: override}) —— init.signal 优先于 request.signal（spec）。
    let cap2: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let cap2c = Arc::clone(&cap2);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *cap2c.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute(
            "globalThis.__reqOnly = new AbortController();\
             globalThis.__initOnly = new AbortController();\
             var req2 = new Request('http://test.local/c', { signal: globalThis.__reqOnly.signal });\
             globalThis.__override = 'pending';\
             fetch(req2, { signal: globalThis.__initOnly.signal })\
               .catch(function(e){ globalThis.__override = 'rejected:' + (e && e.name ? e.name : String(e)); });\
             globalThis.__reqOnly.abort();\
             globalThis.__afterReqAbort = globalThis.__override;\
             globalThis.__initOnly.abort();",
        )
        .unwrap();
    sandbox.execute("1;").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__afterReqAbort").unwrap().value,
        "pending",
        "abort request.signal 不触发（init.signal override 优先，未用 request.signal）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__override").unwrap().value,
        "rejected:AbortError",
        "abort init.signal 触发 reject（init.signal 优先于 request.signal）"
    );
}

#[test]
fn test_response_request_constructors_r2968() {
    // R2968：Response / Request 全局构造器（补全 fetch API 表面）。new Response/new Request 构造 +
    // fetch 结果 instanceof Response（_makeResponseFromWire 经 new Response 路由）+ fetch(new Request) 消费。
    // R2977：headers 为 Headers 实例（.get/.has API，spec），非 plain dict。
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

    // Response 构造器：status/statusText/ok/headers(Headers 实例 .get)/text/json/clone/instanceof。
    sandbox
        .execute(
            "var r = new Response('{\"a\":1}', { status: 201, statusText: 'Created', headers: { 'X-Test': 'r2968' } });\
             globalThis.__isResp = (r instanceof Response);\
             globalThis.__ok = r.ok;\
             globalThis.__status = r.status;\
             globalThis.__statusText = r.statusText;\
             globalThis.__hdrIsHeaders = (r.headers instanceof Headers);\
             globalThis.__hdr = r.headers.get('X-Test');\
             globalThis.__hdrHas = r.headers.has('X-Test');\
             globalThis.__bodyIsStream = (r.body instanceof ReadableStream);\
             r.text().then(function(t){ globalThis.__text = t; });\
             r.json().then(function(j){ globalThis.__json = j.a; });\
             globalThis.__cloneOk = (r.clone() instanceof Response) && r.clone().status === 201;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__isResp)").unwrap().value, "true", "new Response → instanceof Response");
    assert_eq!(sandbox.execute("String(globalThis.__ok)").unwrap().value, "true", "status 201 → ok=true");
    assert_eq!(sandbox.execute("String(globalThis.__status)").unwrap().value, "201", "status 从 init");
    assert_eq!(sandbox.execute("String(globalThis.__statusText)").unwrap().value, "Created", "statusText 从 init");
    assert_eq!(
        sandbox.execute("String(globalThis.__hdrIsHeaders)").unwrap().value,
        "true",
        "R2977：Response.headers instanceof Headers（spec）"
    );
    assert_eq!(sandbox.execute("String(globalThis.__hdr)").unwrap().value, "r2968", "headers.get('X-Test') = 'r2968'（Headers API）");
    assert_eq!(sandbox.execute("String(globalThis.__hdrHas)").unwrap().value, "true", "headers.has('X-Test') = true");
    assert_eq!(
        sandbox.execute("String(globalThis.__bodyIsStream)").unwrap().value,
        "true",
        "Response.body 为 ReadableStream"
    );
    assert_eq!(sandbox.execute("String(globalThis.__text)").unwrap().value, "{\"a\":1}", "text() 返 body");
    assert_eq!(sandbox.execute("String(globalThis.__json)").unwrap().value, "1", "json() 解析 body");
    assert_eq!(sandbox.execute("String(globalThis.__cloneOk)").unwrap().value, "true", "clone() 返 Response + 保留 status");

    // 默认 new Response() → status 200 ok=true，statusText=''。
    sandbox.execute("var d = new Response('x'); globalThis.__dStatus = d.status; globalThis.__dOk = d.ok; globalThis.__dST = d.statusText;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__dStatus)").unwrap().value, "200", "默认 status=200");
    assert_eq!(sandbox.execute("String(globalThis.__dOk)").unwrap().value, "true", "默认 ok=true");
    assert_eq!(sandbox.execute("String(globalThis.__dST)").unwrap().value, "", "默认 statusText=''");

    // Request 构造器：url/method(upper)/headers(Headers 实例)/body + clone。
    sandbox
        .execute(
            "var q = new Request('http://test.local/api', { method: 'post', headers: { 'Content-Type': 'text/plain' }, body: 'hello' });\
             globalThis.__qUrl = q.url;\
             globalThis.__qMethod = q.method;\
             globalThis.__qHdr = q.headers.get('Content-Type');\
             globalThis.__qBody = q.body;\
             globalThis.__qCloneUrl = q.clone().url;\
             globalThis.__qCloneMethod = q.clone().method;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__qUrl)").unwrap().value, "http://test.local/api", "Request.url");
    assert_eq!(sandbox.execute("String(globalThis.__qMethod)").unwrap().value, "POST", "Request.method 大写归一");
    assert_eq!(sandbox.execute("String(globalThis.__qHdr)").unwrap().value, "text/plain", "Request.headers plain dict");
    assert_eq!(sandbox.execute("String(globalThis.__qBody)").unwrap().value, "hello", "Request.body");
    assert_eq!(sandbox.execute("String(globalThis.__qCloneUrl)").unwrap().value, "http://test.local/api", "Request.clone().url");
    assert_eq!(sandbox.execute("String(globalThis.__qCloneMethod)").unwrap().value, "POST", "Request.clone().method");

    // fetch 结果 instanceof Response + fetch(new Request(...)) 消费：mock __zw_fetch 捕获 method/url。
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let cap = Arc::clone(&captured);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            // args: [id, method, url, headersWire, body]
            let mut g = cap.lock().unwrap();
            g.push(args.get(1).cloned().unwrap_or_default()); // method
            g.push(args.get(2).cloned().unwrap_or_default()); // url
            g.push(args.get(4).cloned().unwrap_or_default()); // body
            "ok".to_string()
        }),
    );
    // fetch(new Request(...)) → 取 Request.method/url/body 投递 host。
    sandbox
        .execute(
            "fetch(new Request('http://test.local/r', { method: 'PUT', body: 'payload' }))\
               .then(function(resp){ globalThis.__fetchIsResp = (resp instanceof Response); });",
        )
        .unwrap();
    // 无 resolve（host 不回调）→ Promise pending；但 __zw_fetch 已同步调用，参数已捕获。
    let g = captured.lock().unwrap().clone();
    assert_eq!(g.first().map(|s| s.as_str()), Some("PUT"), "fetch(Request) 投递 Request.method=PUT");
    assert_eq!(g.get(1).map(|s| s.as_str()), Some("http://test.local/r"), "fetch(Request) 投递 Request.url");
    assert_eq!(g.get(2).map(|s| s.as_str()), Some("payload"), "fetch(Request) 投递 Request.body=payload");
    // instanceof 在 resolve 前无法验证（Promise pending）→ 改经 wire resolve 后再验。
    let captured2: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let cap2 = Arc::clone(&captured2);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *cap2.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute("fetch('http://test.local/w').then(function(r){ globalThis.__wResp = (r instanceof Response); globalThis.__wOk = r.ok; });")
        .unwrap();
    let id2 = captured2.lock().unwrap().clone();
    sandbox.resolve_async_callback(&id2, "__zwfr:200\u{001f}OK\u{001f}\u{001f}done");
    assert_eq!(
        sandbox.execute("String(globalThis.__wResp)").unwrap().value,
        "true",
        "fetch 结果 instanceof Response（_makeResponseFromWire 经 new Response 路由）"
    );
    assert_eq!(sandbox.execute("String(globalThis.__wOk)").unwrap().value, "true", "fetch 结果 status 200 → ok=true");
}

#[test]
fn test_writable_stream_r2969() {
    // R2969：WritableStream（Streams API write 侧）。sink {start, write, close, abort} 钩子 +
    // writer.write/close/abort/releaseLock/closed/ready/desiredSize + locked 守卫 + 错误传播
    //（controller.error → write reject + closed reject）。
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

    // sink 钩子：start→write×2→close 全调；getWriter 后 locked=true；close 后 closed resolve。
    sandbox
        .execute(
            "var log = [];\
             var ws = new WritableStream({\
               start: function(){ log.push('start'); },\
               write: function(chunk){ log.push('write:' + chunk); },\
               close: function(){ log.push('close'); }\
             });\
             var w = ws.getWriter();\
             globalThis.__lockedGet = ws.locked;\
             globalThis.__readyIsPromise = (w.ready instanceof Promise || typeof w.ready === 'object');\
             globalThis.__desiredSize = w.desiredSize;\
             w.write('a');\
             w.write('b');\
             w.close().then(function(){ globalThis.__closedOk = 'yes'; globalThis.__log = log.join(','); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__lockedGet)").unwrap().value, "true", "getWriter 后 locked=true");
    assert_eq!(sandbox.execute("String(globalThis.__desiredSize)").unwrap().value, "1", "writable 态 desiredSize=1");
    assert_eq!(
        sandbox.execute("String(globalThis.__log)").unwrap().value,
        "start,write:a,write:b,close",
        "sink 钩子顺序：start→write:a→write:b→close"
    );
    assert_eq!(sandbox.execute("String(globalThis.__closedOk)").unwrap().value, "yes", "writer.close 后 closed Promise resolve");

    // 错误传播：controller.error → 该 write reject + closed reject。
    sandbox
        .execute(
            "var ws2 = new WritableStream({ write: function(chunk, c){ c.error(new Error('boom')); } });\
             var w2 = ws2.getWriter();\
             w2.write('x').then(function(){ globalThis.__writeErr = 'resolved'; },\
                              function(e){ globalThis.__writeErr = String(e); });\
             w2.closed.then(function(){ globalThis.__closedErr = 'resolved'; },\
                                   function(e){ globalThis.__closedErr = String(e); });",
        )
        .unwrap();
    let write_err = sandbox.execute("String(globalThis.__writeErr)").unwrap().value;
    assert!(write_err.contains("boom"), "controller.error → write reject（含 Error 消息），got: {write_err}");
    let closed_err = sandbox.execute("String(globalThis.__closedErr)").unwrap().value;
    assert!(closed_err.contains("boom"), "controller.error → closed reject（含 Error 消息），got: {closed_err}");

    // locked 守卫：已 locked 时 getWriter 抛 TypeError。
    sandbox
        .execute(
            "var ws3 = new WritableStream({});\
             ws3.getWriter();\
             try { ws3.getWriter(); globalThis.__dbl = 'no-throw'; }\
             catch (e) { globalThis.__dbl = String(e).indexOf('TypeError') >= 0 ? 'TypeError' : 'other'; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dbl)").unwrap().value,
        "TypeError",
        "已 locked 时 getWriter 抛 TypeError（spec）"
    );

    // abort：sink.abort 调用 + closed reject。
    sandbox
        .execute(
            "var aborted = null;\
             var ws4 = new WritableStream({ abort: function(r){ aborted = String(r); } });\
             var w4 = ws4.getWriter();\
             w4.abort('stop').then(function(){ globalThis.__abortRet = 'resolved'; });\
             globalThis.__abortedReason = (function(){ return aborted; })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__abortRet)").unwrap().value,
        "resolved",
        "writer.abort 返 resolved Promise"
    );
    // aborted 在 microtask 后才设（abort 同步调 sink.abort，但同步闭包读需 re-execute 读全局）
    sandbox.execute("globalThis.__abortedReason = globalThis.__abortedReason;").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__abortedReason)").unwrap().value,
        "stop",
        "sink.abort 收到 reason='stop'"
    );
}

#[test]
fn test_transform_stream_pipe_r2969() {
    // R2969：TransformStream + pipeTo + pipeThrough。TransformStream 恒等（无 transform fn 直通）+
    // 自定义 transform（uppercase）+ flush；pipeTo readable→writable（sink 收集）；pipeThrough 管道。
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

    // TransformStream 恒等：writable.write('a'/'b') + close → readable 读 'a'/'b'/done。
    sandbox
        .execute(
            "var ts = new TransformStream();\
             var w = ts.writable.getWriter(); w.write('a'); w.write('b'); w.close();\
             var r = ts.readable.getReader();\
             r.read().then(function(c){ globalThis.__t1 = c; return r.read(); })\
                    .then(function(c){ globalThis.__t2 = c; return r.read(); })\
                    .then(function(c){ globalThis.__t3 = c; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__t1 && globalThis.__t1.value)").unwrap().value, "a", "恒等 transform 第 1 chunk='a'");
    assert_eq!(sandbox.execute("String(globalThis.__t2 && globalThis.__t2.value)").unwrap().value, "b", "恒等 transform 第 2 chunk='b'");
    assert_eq!(sandbox.execute("String(globalThis.__t3 && globalThis.__t3.done)").unwrap().value, "true", "恒等 transform 第 3 read done（writable close → readable close）");

    // 自定义 transform（uppercase）。
    sandbox
        .execute(
            "var ts2 = new TransformStream({ transform: function(chunk, c){ c.enqueue(chunk.toUpperCase()); } });\
             var w2 = ts2.writable.getWriter(); w2.write('hi'); w2.close();\
             var r2 = ts2.readable.getReader();\
             r2.read().then(function(c){ globalThis.__tu = c; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__tu && globalThis.__tu.value)").unwrap().value, "HI", "自定义 transform：'hi'→'HI'");

    // pipeTo：readable chunks → writable sink 收集 → 'xy'。
    sandbox
        .execute(
            "var collected = [];\
             var src = new ReadableStream({ start: function(c){ c.enqueue('x'); c.enqueue('y'); c.close(); } });\
             var dst = new WritableStream({ write: function(chunk){ collected.push(chunk); } });\
             src.pipeTo(dst).then(function(){ globalThis.__piped = collected.join(''); });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__piped)").unwrap().value, "xy", "pipeTo：src 两 chunk 写入 dst sink 收集为 'xy'");

    // pipeThrough：src → transform(uppercase) → readable 读 'A'/'B'/done。
    sandbox
        .execute(
            "var src2 = new ReadableStream({ start: function(c){ c.enqueue('a'); c.enqueue('b'); c.close(); } });\
             var out = src2.pipeThrough(new TransformStream({ transform: function(chunk, c){ c.enqueue(chunk.toUpperCase()); } }));\
             globalThis.__outIsReadable = (out instanceof ReadableStream);\
             var ro = out.getReader();\
             ro.read().then(function(c){ globalThis.__pt = c; return ro.read(); })\
                     .then(function(c){ globalThis.__pt2 = c; return ro.read(); })\
                     .then(function(c){ globalThis.__pt3 = c; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__outIsReadable)").unwrap().value, "true", "pipeThrough 返 ReadableStream");
    assert_eq!(sandbox.execute("String(globalThis.__pt && globalThis.__pt.value)").unwrap().value, "A", "pipeThrough + transform：'a'→'A'");
    assert_eq!(sandbox.execute("String(globalThis.__pt2 && globalThis.__pt2.value)").unwrap().value, "B", "pipeThrough + transform：'b'→'B'");
    assert_eq!(sandbox.execute("String(globalThis.__pt3 && globalThis.__pt3.done)").unwrap().value, "true", "pipeThrough 管道末 read done");
}

#[test]
fn test_text_encoder_decoder_stream_r2970() {
    // R2970：TextEncoderStream（string→UTF-8 Uint8Array）/ TextDecoderStream（Uint8Array→string）。
    // TransformStream 子类（instanceof TransformStream），encoding/fatal/ignoreBOM 属性；常与
    // response.body.pipeThrough(new TextDecoderStream()) 配对做字节→文本流解码。
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

    // TextEncoderStream：instanceof TransformStream + encoding='utf-8'；写 'Hi' → 读 Uint8Array [72,105]。
    sandbox
        .execute(
            "var tes = new TextEncoderStream();\
             globalThis.__tesEnc = tes.encoding;\
             globalThis.__tesIsTS = (tes instanceof TransformStream);\
             globalThis.__tesHasRW = (tes.readable instanceof ReadableStream) && (tes.writable instanceof WritableStream);\
             var we = tes.writable.getWriter(); we.write('Hi'); we.close();\
             var re = tes.readable.getReader();\
             re.read().then(function(c){ globalThis.__encChunk = c; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__tesEnc)").unwrap().value, "utf-8", "TextEncoderStream.encoding='utf-8'");
    assert_eq!(sandbox.execute("String(globalThis.__tesIsTS)").unwrap().value, "true", "TextEncoderStream instanceof TransformStream");
    assert_eq!(sandbox.execute("String(globalThis.__tesHasRW)").unwrap().value, "true", "TextEncoderStream 有 readable + writable");
    assert_eq!(
        sandbox.execute("String(globalThis.__encChunk && globalThis.__encChunk.done)").unwrap().value,
        "false",
        "encode chunk: done=false"
    );
    assert_eq!(
        sandbox
            .execute("String(new TextDecoder().decode(globalThis.__encChunk.value))")
            .unwrap()
            .value,
        "Hi",
        "TextEncoderStream 编码 'Hi' → 经 TextDecoder 还原 = 'Hi'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__encChunk.value && globalThis.__encChunk.value.length)")
            .unwrap()
            .value,
        "2",
        "TextEncoderStream 编码 'Hi' = 2 字节"
    );

    // TextDecoderStream：写 TextEncoder 编码的 'Hello' 字节 → 读 string 'Hello'。
    sandbox
        .execute(
            "var bytes = new TextEncoder().encode('Hello');\
             var tds = new TextDecoderStream();\
             globalThis.__tdsEnc = tds.encoding;\
             globalThis.__tdsFatal = tds.fatal;\
             var wd = tds.writable.getWriter(); wd.write(bytes); wd.close();\
             var rd = tds.readable.getReader();\
             rd.read().then(function(c){ globalThis.__decChunk = c; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__tdsEnc)").unwrap().value, "utf-8", "TextDecoderStream.encoding='utf-8'");
    assert_eq!(sandbox.execute("String(globalThis.__tdsFatal)").unwrap().value, "false", "TextDecoderStream.fatal=false（缺省）");
    assert_eq!(
        sandbox.execute("String(globalThis.__decChunk && globalThis.__decChunk.done)").unwrap().value,
        "false",
        "decode chunk: done=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__decChunk && globalThis.__decChunk.value)").unwrap().value,
        "Hello",
        "TextDecoderStream 解码 'Hello' 字节 → 'Hello'"
    );

    // pipeThrough TextDecoderStream：src 字节流 → string 流（典型 response.body 解码模式）。
    sandbox
        .execute(
            "var src = new ReadableStream({ start: function(c){ c.enqueue(new TextEncoder().encode('World')); c.close(); } });\
             var out = src.pipeThrough(new TextDecoderStream());\
             globalThis.__outReadable = (out instanceof ReadableStream);\
             var ro = out.getReader();\
             ro.read().then(function(c){ globalThis.__pipeDec = c; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__outReadable)").unwrap().value, "true", "pipeThrough(TextDecoderStream) 返 ReadableStream");
    assert_eq!(
        sandbox.execute("String(globalThis.__pipeDec && globalThis.__pipeDec.value)").unwrap().value,
        "World",
        "src 字节流 pipeThrough TextDecoderStream → 'World'"
    );

    // pipeThrough TextEncoderStream：src string 流 → 字节流。
    sandbox
        .execute(
            "var src2 = new ReadableStream({ start: function(c){ c.enqueue('Ping'); c.close(); } });\
             var out2 = src2.pipeThrough(new TextEncoderStream());\
             var ro2 = out2.getReader();\
             ro2.read().then(function(c){ globalThis.__pipeEnc = c; });",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(new TextDecoder().decode(globalThis.__pipeEnc.value))")
            .unwrap()
            .value,
        "Ping",
        "src string 流 pipeThrough TextEncoderStream → 字节，解码 = 'Ping'"
    );
}

#[test]
fn test_readable_stream_tee_r2971() {
    // R2971：ReadableStream.tee()——分叉成两独立分支共享同一源。两分支各自完整收到全部 chunk（顺序一致），
    // 独立消费（不同速率）；源 close/error 同步到两分支；源 locked 时 tee 抛 TypeError。
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

    // 源 3 chunk + close；tee → [b1, b2]，两分支独立读全部 chunk。
    sandbox
        .execute(
            "var src = new ReadableStream({ start: function(c){ c.enqueue('a'); c.enqueue('b'); c.enqueue('c'); c.close(); } });\
             var br = src.tee();\
             var b1 = br[0], b2 = br[1];\
             globalThis.__bothReadable = (b1 instanceof ReadableStream) && (b2 instanceof ReadableStream);\
             var r1 = b1.getReader();\
             r1.read().then(function(c){ globalThis.__b1a = c; return r1.read(); })\
                    .then(function(c){ globalThis.__b1b = c; return r1.read(); })\
                    .then(function(c){ globalThis.__b1c = c; return r1.read(); })\
                    .then(function(c){ globalThis.__b1d = c; });\
             var r2 = b2.getReader();\
             r2.read().then(function(c){ globalThis.__b2a = c; return r2.read(); })\
                    .then(function(c){ globalThis.__b2b = c; return r2.read(); })\
                    .then(function(c){ globalThis.__b2c = c; return r2.read(); })\
                    .then(function(c){ globalThis.__b2d = c; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__bothReadable)").unwrap().value, "true", "tee 返两 ReadableStream");
    // 分支 1 完整收到 a/b/c + done。
    assert_eq!(sandbox.execute("String(globalThis.__b1a && globalThis.__b1a.value)").unwrap().value, "a", "分支1 chunk a");
    assert_eq!(sandbox.execute("String(globalThis.__b1b && globalThis.__b1b.value)").unwrap().value, "b", "分支1 chunk b");
    assert_eq!(sandbox.execute("String(globalThis.__b1c && globalThis.__b1c.value)").unwrap().value, "c", "分支1 chunk c");
    assert_eq!(sandbox.execute("String(globalThis.__b1d && globalThis.__b1d.done)").unwrap().value, "true", "分支1 末 read done");
    // 分支 2 独立收到同样的 a/b/c + done（独立消费，无丢失）。
    assert_eq!(sandbox.execute("String(globalThis.__b2a && globalThis.__b2a.value)").unwrap().value, "a", "分支2 chunk a（独立消费）");
    assert_eq!(sandbox.execute("String(globalThis.__b2b && globalThis.__b2b.value)").unwrap().value, "b", "分支2 chunk b");
    assert_eq!(sandbox.execute("String(globalThis.__b2c && globalThis.__b2c.value)").unwrap().value, "c", "分支2 chunk c");
    assert_eq!(sandbox.execute("String(globalThis.__b2d && globalThis.__b2d.done)").unwrap().value, "true", "分支2 末 read done");

    // 源 locked 时 tee 抛 TypeError。
    sandbox
        .execute(
            "var src2 = new ReadableStream({ start: function(c){ c.close(); } });\
             src2.getReader();\
             try { src2.tee(); globalThis.__teeLock = 'no-throw'; }\
             catch (e) { globalThis.__teeLock = String(e).indexOf('TypeError') >= 0 ? 'TypeError' : 'other'; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__teeLock)").unwrap().value,
        "TypeError",
        "源 locked 时 tee 抛 TypeError（spec）"
    );

    // 源 error → 两分支 read reject（错误同步传播）。
    sandbox
        .execute(
            "var src3 = new ReadableStream({ start: function(c){ c.error(new Error('srcfail')); } });\
             var b3 = src3.tee();\
             var r3a = b3[0].getReader();\
             r3a.read().then(function(){ globalThis.__teeErr1 = 'resolved'; },\
                             function(e){ globalThis.__teeErr1 = String(e); });\
             var r3b = b3[1].getReader();\
             r3b.read().then(function(){ globalThis.__teeErr2 = 'resolved'; },\
                             function(e){ globalThis.__teeErr2 = String(e); });",
        )
        .unwrap();
    let e1 = sandbox.execute("String(globalThis.__teeErr1)").unwrap().value;
    assert!(e1.contains("srcfail"), "源 error → 分支1 read reject（含错误消息），got: {e1}");
    let e2 = sandbox.execute("String(globalThis.__teeErr2)").unwrap().value;
    assert!(e2.contains("srcfail"), "源 error → 分支2 read reject（错误同步两分支），got: {e2}");
}

#[test]
fn test_resize_observer_content_border_box_r2972() {
    // R2972：ResizeObserverEntry box-model 真值。borderBoxSize = border-box（gBCR）；contentBoxSize /
    // contentRect / devicePixelContentBoxSize = content-box（border - padding - border-width，经 getComputedStyle
    // 真值扣除）。mock gBCR 返 border-box 100x50，元素 padding:10px + border:5px solid → content 70x20。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // padding:10px + border:5px solid（border-width 5px，style 非 none→渲染宽度 5px）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='t' style='width:100px;height:50px;padding:10px;border:5px solid black'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // mock gBCR：#t 返 border-box 100x50（gBCR 恒 border-box）。
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|args| {
            let sel = args.first().cloned().unwrap_or_default();
            if sel.contains("#t") || sel == "#t" {
                "0,0,100,50".to_string()
            } else {
                "0,0,0,0".to_string()
            }
        }),
    );

    sandbox
        .execute(
            "globalThis.__entry = null;\
             new ResizeObserver(function(e){ globalThis.__entry = e[0]; }).observe(document.querySelector('#t'));",
        )
        .unwrap();
    // border-box 100x50；content = 100 - 10 - 10 - 5 - 5 = 70 wide；50 - 10 - 10 - 5 - 5 = 20 tall。
    assert_eq!(
        sandbox
            .execute("String(globalThis.__entry && globalThis.__entry.borderBoxSize[0].inlineSize)")
            .unwrap()
            .value,
        "100",
        "borderBoxSize.inlineSize = border-box 宽 100（gBCR）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__entry && globalThis.__entry.borderBoxSize[0].blockSize)")
            .unwrap()
            .value,
        "50",
        "borderBoxSize.blockSize = border-box 高 50"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__entry && globalThis.__entry.contentBoxSize[0].inlineSize)")
            .unwrap()
            .value,
        "70",
        "contentBoxSize.inlineSize = 70（100 - padding 10×2 - border 5×2）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__entry && globalThis.__entry.contentBoxSize[0].blockSize)")
            .unwrap()
            .value,
        "20",
        "contentBoxSize.blockSize = 20（50 - padding 10×2 - border 5×2）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__entry && globalThis.__entry.contentRect.width)")
            .unwrap()
            .value,
        "70",
        "contentRect.width = content-box 宽 70（spec：contentRect 为 content-box）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__entry && globalThis.__entry.contentRect.height)")
            .unwrap()
            .value,
        "20",
        "contentRect.height = content-box 高 20"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__entry && globalThis.__entry.devicePixelContentBoxSize[0].inlineSize)")
            .unwrap()
            .value,
        "70",
        "devicePixelContentBoxSize = content-box（headless 无 device pixel ratio，同 contentBoxSize）"
    );

    // 无 padding/border 元素：content = border（回归既有行为，contentRect 同 border-box）。
    let dom_html2: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='p' style='width:80px;height:40px'></div></body></html>".to_string(),
    ));
    // 切换 dom_html（新 snapshot）需重建 sandbox（cs_cache 按 html_key 缓存）。新 sandbox 复测。
    let mut sandbox2 = V8Sandbox::with_config(
        zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() },
    )
    .unwrap();
    sandbox2.execute(generate_js_dom_shim()).unwrap();
    let mutations2: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let page_url2: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox2, &mutations2, &dom_html2, &page_url2, &canvas_registry);
    sandbox2.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|args| {
            let sel = args.first().cloned().unwrap_or_default();
            if sel.contains("#p") || sel == "#p" {
                "0,0,80,40".to_string()
            } else {
                "0,0,0,0".to_string()
            }
        }),
    );
    sandbox2
        .execute(
            "globalThis.__e2 = null;\
             new ResizeObserver(function(e){ globalThis.__e2 = e[0]; }).observe(document.querySelector('#p'));",
        )
        .unwrap();
    assert_eq!(
        sandbox2.execute("String(globalThis.__e2 && globalThis.__e2.contentBoxSize[0].inlineSize)").unwrap().value,
        "80",
        "无 padding/border → contentBoxSize = border-box 80（content = border，回归）"
    );
    assert_eq!(
        sandbox2.execute("String(globalThis.__e2 && globalThis.__e2.borderBoxSize[0].blockSize)").unwrap().value,
        "40",
        "无 padding/border → borderBoxSize.blockSize = 40"
    );
}

#[test]
fn test_blob_stream_response_body_readers_r2978() {
    // R2978：Blob.stream()（字节→ReadableStream，配对 R2967）+ Response.blob()/arrayBuffer()/formData()
    //（补全 body-consumption 表面，spec text/json/blob/arrayBuffer/formData）。
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

    // Blob.stream()：instanceof ReadableStream + 单 UTF-8 chunk → TextDecoder 还原原文 + done。
    sandbox
        .execute(
            "var blob = new Blob(['Hello']);\
             var st = blob.stream();\
             globalThis.__streamIsRS = (st instanceof ReadableStream);\
             var r = st.getReader();\
             r.read().then(function(c){ globalThis.__chunk = c; return r.read(); })\
                    .then(function(c){ globalThis.__done = c; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__streamIsRS)").unwrap().value,
        "true",
        "Blob.stream() instanceof ReadableStream"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__chunk && globalThis.__chunk.done)").unwrap().value,
        "false",
        "Blob.stream 首 chunk: done=false"
    );
    assert_eq!(
        sandbox
            .execute("String(new TextDecoder().decode(globalThis.__chunk.value))")
            .unwrap()
            .value,
        "Hello",
        "Blob.stream chunk 经 TextDecoder 解码 = 'Hello'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__done && globalThis.__done.done)").unwrap().value,
        "true",
        "Blob.stream 次 read done=true（单 chunk 后 close）"
    );

    // Response.blob()：body 包成 Blob（instanceof Blob + text() 还原）。
    sandbox
        .execute(
            "new Response('hi').blob()\
               .then(function(b){ globalThis.__rbIsBlob = (b instanceof Blob); return b.text(); })\
               .then(function(t){ globalThis.__rbText = t; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rbIsBlob)").unwrap().value,
        "true",
        "Response.blob() → instanceof Blob"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rbText)").unwrap().value,
        "hi",
        "Response.blob().text() 还原 body = 'hi'"
    );

    // Response.arrayBuffer()：UTF-8 Uint8Array（byteLength + 索引）。
    sandbox
        .execute(
            "new Response('ABC').arrayBuffer().then(function(buf){\
               globalThis.__abIsU8 = (buf instanceof Uint8Array);\
               globalThis.__abLen = buf.length;\
               globalThis.__abByteLen = buf.byteLength;\
               globalThis.__ab0 = buf[0];\
             });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__abIsU8)").unwrap().value,
        "true",
        "Response.arrayBuffer() → Uint8Array"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__abLen)").unwrap().value,
        "3",
        "Response.arrayBuffer('ABC') length=3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__abByteLen)").unwrap().value,
        "3",
        "Response.arrayBuffer byteLength=3"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ab0)").unwrap().value,
        "65",
        "Response.arrayBuffer('ABC')[0]=65 ('A')"
    );

    // Response.formData()：application/x-www-form-urlencoded 解析（+ → space，% 解码）。
    sandbox
        .execute(
            "new Response('a=1&b=two&c=hello+world').formData().then(function(fd){\
               globalThis.__fdA = fd.get('a');\
               globalThis.__fdB = fd.get('b');\
               globalThis.__fdC = fd.get('c');\
             });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fdA)").unwrap().value,
        "1",
        "Response.formData() 解析 a=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fdB)").unwrap().value,
        "two",
        "Response.formData() 解析 b=two"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fdC)").unwrap().value,
        "hello world",
        "Response.formData() + → space（urlencoded 语义）"
    );
}

#[test]
fn test_window_dialog_methods_r2979() {
    // R2979：window.alert/confirm/prompt/open——此前全缺，`if (confirm(...))` / `alert(...)` / `prompt(...)`
    // 抛 ReferenceError 中断脚本。headless 无 UI 用户交互 → spec dismiss 语义：alert 返 undefined（不阻塞）、
    // confirm 返 false、prompt 返 null、open 返 null（popup-blocked）。
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

    sandbox
        .execute(
            "globalThis.__alertType = typeof alert;\
             globalThis.__confirmType = typeof confirm;\
             globalThis.__promptType = typeof prompt;\
             globalThis.__openType = typeof open;\
             globalThis.__confirmR = String(confirm('Delete?'));\
             globalThis.__promptR = (prompt('Name', 'x') === null ? 'null' : 'other');\
             globalThis.__openR = (window.open('http://test.local/popup') === null ? 'null' : 'other');\
             try { alert('hi'); globalThis.__alertNoThrow = 'ok'; } catch (e) { globalThis.__alertNoThrow = 'err'; }",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__alertType)").unwrap().value, "function", "alert 是 function");
    assert_eq!(sandbox.execute("String(globalThis.__confirmType)").unwrap().value, "function", "confirm 是 function");
    assert_eq!(sandbox.execute("String(globalThis.__promptType)").unwrap().value, "function", "prompt 是 function");
    assert_eq!(sandbox.execute("String(globalThis.__openType)").unwrap().value, "function", "open 是 function");
    assert_eq!(
        sandbox.execute("String(globalThis.__confirmR)").unwrap().value,
        "false",
        "confirm() → false（headless 无用户点 OK = dismiss）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__promptR)").unwrap().value,
        "null",
        "prompt() → null（headless 无用户输入 = dismiss）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__openR)").unwrap().value,
        "null",
        "window.open() → null（headless 弹窗被阻 = popup-blocked 语义）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__alertNoThrow)").unwrap().value,
        "ok",
        "alert() 不抛（no-op，spec 返 undefined 不阻塞）"
    );

    // 典型用法：if (confirm(...)) 守卫——confirm=false 时走 else 分支不进删除逻辑。
    sandbox
        .execute(
            "var deleted = false;\
             if (confirm('Delete item?')) { deleted = true; }\
             globalThis.__guardedDeleted = String(deleted);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__guardedDeleted)").unwrap().value,
        "false",
        "if (confirm(...)) 守卫：confirm=false 不进删除分支（headless 不误删）"
    );
}

#[test]
fn test_element_get_elements_by_r2980() {
    // R2980：Element 子树作用域 getElementsByTagName/getElementsByClassName + document.getElementsByName。
    // 此前 element 实例上两者全缺（`table.getElementsByTagName('td')` / `wrap.getElementsByClassName('item')`
    // 返 undefined / 抛），document.getElementsByName 亦全缺。现代/遗留代码高频（表单控件枚举、按类批量操作、
    // 按 name 取字段）。spec 返 live HTMLCollection，headless 近似为静态 array-like（同 querySelectorAll 模型）。
    // 关键覆盖：子树作用域（不含元素自身、不含子树外）+ `'*'` 通配（host 不支持→客户端递归）+
    // 多类交集（'.row.hot'）+ getElementsByName 经 [name="…"] 属性选择器。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // #wrap 内：span.row(a) / span.row.hot(b) / p.hot(c) / section>span.row(d)；wrap 外：span.row(out1) +
    // input[name=csrf-token] / a[name=csrf-token] / input[name=user]。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <div id='wrap'>\
         <span class='row'>a</span>\
         <span class='row hot'>b</span>\
         <p class='hot'>c</p>\
         <section><span class='row'>d</span></section>\
         </div>\
         <span class='row'>out1</span>\
         <input name='csrf-token'>\
         <a name='csrf-token'>anchor</a>\
         <input name='user'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 方法存在性（element + document）。
    assert_eq!(
        sandbox
            .execute("typeof document.querySelector('#wrap').getElementsByTagName")
            .unwrap()
            .value,
        "function",
        "el.getElementsByTagName 是 function"
    );
    assert_eq!(
        sandbox
            .execute("typeof document.querySelector('#wrap').getElementsByClassName")
            .unwrap()
            .value,
        "function",
        "el.getElementsByClassName 是 function"
    );
    assert_eq!(
        sandbox
            .execute("typeof document.getElementsByName")
            .unwrap()
            .value,
        "function",
        "document.getElementsByName 是 function"
    );

    // 元素子树 getElementsByTagName('span')：仅 wrap 后代（a/b/d），不含 out1，不含 wrap 自身。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#wrap').getElementsByTagName('span').length")
            .unwrap()
            .value,
        "3",
        "wrap.getElementsByTagName('span') = 3（a/b/d，子树作用域）"
    );
    // tree order：首元素 = a（SPAN）。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#wrap').getElementsByTagName('span')[0].tagName")
            .unwrap()
            .value,
        "SPAN",
        "首匹配为 a（SPAN）"
    );
    assert_eq!(
        sandbox
            .execute(
                "document.querySelector('#wrap').getElementsByTagName('span')[1].className"
            )
            .unwrap()
            .value,
        "row hot",
        "第二匹配 b 的 className=row hot"
    );
    // 子树作用域不含元素自身：wrap 无 div 后代 → 0。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#wrap').getElementsByTagName('div').length")
            .unwrap()
            .value,
        "0",
        "wrap.getElementsByTagName('div') = 0（不含自身，仅后代）"
    );

    // 元素子树 getElementsByClassName：单类 / 多类交集。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#wrap').getElementsByClassName('row').length")
            .unwrap()
            .value,
        "3",
        "wrap.getElementsByClassName('row') = 3（a/b/d）"
    );
    assert_eq!(
        sandbox
            .execute("document.querySelector('#wrap').getElementsByClassName('hot').length")
            .unwrap()
            .value,
        "2",
        "wrap.getElementsByClassName('hot') = 2（b/c）"
    );
    // 多类交集：'row hot' → 须同时含两类 → 仅 b。
    assert_eq!(
        sandbox
            .execute(
                "document.querySelector('#wrap').getElementsByClassName('row hot').length"
            )
            .unwrap()
            .value,
        "1",
        "wrap.getElementsByClassName('row hot') = 1（多类交集，仅 b）"
    );
    assert_eq!(
        sandbox
            .execute(
                "document.querySelector('#wrap').getElementsByClassName('row hot')[0].tagName"
            )
            .unwrap()
            .value,
        "SPAN",
        "多类交集首匹配 b 为 SPAN"
    );

    // 通配 '*'：host 不支持 → 客户端递归下降收全部元素后代（a/b/c/section/d = 5）。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#wrap').getElementsByTagName('*').length")
            .unwrap()
            .value,
        "5",
        "wrap.getElementsByTagName('*') = 5（全后代：a/b/c/section/d）"
    );
    // 空 tagName / 空 className → 空集合（spec）。
    assert_eq!(
        sandbox
            .execute("document.querySelector('#wrap').getElementsByTagName('').length")
            .unwrap()
            .value,
        "0",
        "wrap.getElementsByTagName('') = 0（空 tagName → 空集合）"
    );
    assert_eq!(
        sandbox
            .execute("document.querySelector('#wrap').getElementsByClassName('').length")
            .unwrap()
            .value,
        "0",
        "wrap.getElementsByClassName('') = 0（空 className → 空集合）"
    );

    // document 作用域 getElementsByTagName（含子树外 out1）。
    assert_eq!(
        sandbox
            .execute("document.getElementsByTagName('span').length")
            .unwrap()
            .value,
        "4",
        "document.getElementsByTagName('span') = 4（含子树外 out1）"
    );

    // document.getElementsByName：按 name 属性匹配全文档。
    assert_eq!(
        sandbox
            .execute("document.getElementsByName('csrf-token').length")
            .unwrap()
            .value,
        "2",
        "document.getElementsByName('csrf-token') = 2（input + a）"
    );
    assert_eq!(
        sandbox
            .execute("document.getElementsByName('csrf-token')[0].tagName")
            .unwrap()
            .value,
        "INPUT",
        "首 name=csrf-token 为 INPUT（tree order）"
    );
    assert_eq!(
        sandbox
            .execute("document.getElementsByName('csrf-token')[1].tagName")
            .unwrap()
            .value,
        "A",
        "次 name=csrf-token 为 A"
    );
    assert_eq!(
        sandbox
            .execute("document.getElementsByName('user').length")
            .unwrap()
            .value,
        "1",
        "document.getElementsByName('user') = 1"
    );
    assert_eq!(
        sandbox
            .execute("document.getElementsByName('nope').length")
            .unwrap()
            .value,
        "0",
        "document.getElementsByName('nope') = 0（无匹配）"
    );
}

#[test]
fn test_fetch_forbidden_headers_r3221_r3222() {
    // R3221：fetch 出口过滤 Fetch §3.4.4 禁止请求头（Host/Content-Length/Cookie/Sec-*/Proxy-* 等）——
    // JS 设的禁止头永不到达 host；R3222：response.headers 不暴露 Set-Cookie/Set-Cookie2（Fetch §3.4.5），
    // 但 getSetCookie 返数组 + Response.clone() 保 Set-Cookie + 多 Set-Cookie 经 _parseHeadersWire 累加（旧 last-wins 丢）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // —— R3221：捕获 fetch 投递的 headersWire（args[3]），验禁止头被剥离、允许头保留 ——
    let captured: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let cap = Arc::clone(&captured);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            *cap.lock().unwrap() = args.get(3).cloned().unwrap_or_default(); // headersWire
            "ok".to_string()
        }),
    );
    sandbox
        .execute(
            "fetch('http://test.local/h', { headers: {\
               'Host':'evil.com','Content-Length':'9999','Cookie':'sess=leak',\
               'Sec-Fetch-Mode':'cors','Proxy-Authorization':'Basic x',\
               'X-Custom':'keep','Content-Type':'text/plain'\
             } });",
        )
        .unwrap();
    let wire = captured.lock().unwrap().clone();
    let tokens: Vec<&str> = wire.split('\u{001e}').collect();
    let mut names: Vec<String> = Vec::new();
    let mut i = 0;
    while i + 1 < tokens.len() {
        names.push(tokens[i].to_ascii_lowercase());
        i += 2;
    }
    for forbidden in ["host", "content-length", "cookie", "sec-fetch-mode", "proxy-authorization"] {
        assert!(
            !names.iter().any(|n| n == forbidden),
            "R3221: 禁止请求头 {forbidden} 须被剥离（wire={wire:?}）"
        );
    }
    for allowed in ["x-custom", "content-type"] {
        assert!(
            names.iter().any(|n| n == allowed),
            "R3221: 非禁止头 {allowed} 须保留（wire={wire:?}）"
        );
    }

    // —— R3222：resolve 一个含双 Set-Cookie 的响应，验 response guard ——
    let captured_id: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    let capid = Arc::clone(&captured_id);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            if let Some(id) = args.first() {
                *capid.lock().unwrap() = id.clone();
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute(
            "fetch('http://test.local/s').then(function(r){\
               globalThis.__scGet = r.headers.get('Set-Cookie');\
               globalThis.__scHas = String(r.headers.has('set-cookie'));\
               globalThis.__scArr = r.headers.getSetCookie().join('|');\
               globalThis.__ctGet = r.headers.get('Content-Type');\
               globalThis.__names = [...r.headers].map(function(p){return p[0];}).join(',');\
               var cl = r.clone();\
               globalThis.__cloneArr = cl.headers.getSetCookie().join('|');\
               globalThis.__cloneGet = cl.headers.get('Set-Cookie');\
             });",
        )
        .unwrap();
    let id = captured_id.lock().unwrap().clone();
    // wire：status=200 / statusText=OK / headersWire=双 Set-Cookie + Content-Type / body=hi
    sandbox.resolve_async_callback(
        &id,
        "__zwfr:200\u{001f}OK\u{001f}Set-Cookie\u{001e}a=1\u{001e}Set-Cookie\u{001e}b=2\u{001e}Content-Type\u{001e}text/plain\u{001f}hi",
    );

    // get/has 经 response guard 不暴露 Set-Cookie（Fetch §3.4.5）。
    assert_eq!(
        sandbox.execute("String(globalThis.__scGet)").unwrap().value,
        "null",
        "R3222: response.headers.get('Set-Cookie') 须 null（forbidden response-header）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__scHas)").unwrap().value,
        "false",
        "R3222: response.headers.has('set-cookie') 须 false"
    );
    // getSetCookie 仍返数组（spec 特例）；多 Set-Cookie 经 _parseHeadersWire 累加（旧 last-wins 丢多 cookie）。
    assert_eq!(
        sandbox.execute("String(globalThis.__scArr)").unwrap().value,
        "a=1|b=2",
        "R3222: getSetCookie() 须返双 Set-Cookie 数组（多值累加）"
    );
    // 非禁止响应头正常读。
    assert_eq!(
        sandbox.execute("String(globalThis.__ctGet)").unwrap().value,
        "text/plain",
        "R3222: Content-Type 非禁止，须正常读"
    );
    // 迭代（entries）排除 Set-Cookie。
    assert_eq!(
        sandbox.execute("String(globalThis.__names)").unwrap().value,
        "content-type",
        "R3222: 迭代须排除 Set-Cookie，仅含 content-type"
    );
    // Response.clone() 保 Set-Cookie（raw _h 拷贝 + 新 Response guard）。
    assert_eq!(
        sandbox.execute("String(globalThis.__cloneArr)").unwrap().value,
        "a=1|b=2",
        "R3222: Response.clone().headers.getSetCookie() 须保双 Set-Cookie"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cloneGet)").unwrap().value,
        "null",
        "R3222: clone 的 response.headers.get('Set-Cookie') 仍 null（guard）"
    );
}

#[test]
fn test_request_headers_guard_r3223() {
    // R3223：Headers guard 系统（Fetch §5.1/§5.2/§6.2/§6.3）——request guard 在 fill 前设（§6.3 step 31-32），
    // append/set/delete 写侧阻断禁止请求头（闭合 R3222 已知限①）；response guard 写侧阻断 Set-Cookie（闭合限③）；
    // standalone new Headers 为 guard-none 不受限。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // Request guard：构造时禁止请求头被过滤（guard 先于 fill）。
    sandbox
        .execute(
            "var r = new Request('http://test.local/', { headers: {\
               'Host':'evil.com','Content-Length':'9','Cookie':'x','Sec-Fetch-Mode':'cors',\
               'Proxy-Authorization':'z','X-Custom':'keep','Content-Type':'text/plain'\
             } });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(r.headers.get('Host'))").unwrap().value,
        "null",
        "R3223: request guard 过滤 Host（构造时）"
    );
    assert_eq!(sandbox.execute("String(r.headers.has('content-length'))").unwrap().value, "false");
    assert_eq!(sandbox.execute("String(r.headers.get('cookie'))").unwrap().value, "null");
    assert_eq!(sandbox.execute("String(r.headers.has('sec-fetch-mode'))").unwrap().value, "false");
    assert_eq!(
        sandbox.execute("String(r.headers.has('proxy-authorization'))").unwrap().value,
        "false"
    );
    // 非禁止头保留。
    assert_eq!(sandbox.execute("r.headers.get('X-Custom')").unwrap().value, "keep");
    assert_eq!(sandbox.execute("r.headers.get('Content-Type')").unwrap().value, "text/plain");

    // append/set 在 request guard 上对禁止头 no-op（Fetch §5.2 step 3）。
    sandbox.execute("r.headers.append('Host','more');").unwrap();
    assert_eq!(
        sandbox.execute("String(r.headers.get('Host'))").unwrap().value,
        "null",
        "R3223: append('Host') on request guard 须 no-op"
    );
    sandbox.execute("r.headers.set('Sec-Fetch-Site','cross');").unwrap();
    assert_eq!(
        sandbox.execute("String(r.headers.has('sec-fetch-site'))").unwrap().value,
        "false",
        "R3223: set('Sec-*') on request guard 须 no-op"
    );
    // append 非禁止头仍可写。
    sandbox.execute("r.headers.append('X-More','v');").unwrap();
    assert_eq!(sandbox.execute("r.headers.get('X-More')").unwrap().value, "v");

    // response guard 写侧阻断 Set-Cookie（闭合 R3222 已知限③）。
    sandbox
        .execute("var resp = new Response('b', { headers: {'Content-Type':'text/plain'} });")
        .unwrap();
    sandbox.execute("resp.headers.append('Set-Cookie','a=1');").unwrap();
    assert_eq!(
        sandbox.execute("String(resp.headers.getSetCookie().length)").unwrap().value,
        "0",
        "R3223: response guard append('Set-Cookie') 须 no-op"
    );
    assert_eq!(
        sandbox.execute("resp.headers.get('Content-Type')").unwrap().value,
        "text/plain",
        "R3223: response guard 非禁止头可读"
    );

    // guard-none（standalone new Headers）不受限——forbidden 可写可读（spec：guard none 不过滤）。
    sandbox.execute("var h = new Headers(); h.append('Host','free');").unwrap();
    assert_eq!(
        sandbox.execute("h.get('Host')").unwrap().value,
        "free",
        "R3223: guard-none Headers 不受限（Host 可写可读）"
    );

    // Request.clone() 保留允许头 + guard 一致传递（Host 仍被过滤）。
    sandbox.execute("var rc = r.clone();").unwrap();
    assert_eq!(
        sandbox.execute("rc.headers.get('X-Custom')").unwrap().value,
        "keep",
        "R3223: Request.clone() 保留允许头"
    );
    assert_eq!(
        sandbox.execute("String(rc.headers.get('Host'))").unwrap().value,
        "null",
        "R3223: Request.clone() guard 一致（Host 仍 null）"
    );
}


