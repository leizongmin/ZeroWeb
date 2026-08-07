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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

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
