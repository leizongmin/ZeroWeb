#[test]
fn test_datatransfer_itemlist_r2948() {
    // R2948 DataTransferItemList + DataTransferItem：dataTransfer.items 为 live 视图（替代 R2937 空数组占位）。
    // setData → string items；length/indexed/iterator；getAsString；add(data,type)/add(file)；remove/clear；getAsFile。
    // DataTransfer 为纯 JS 构造器（无 host 回调），无需 register_dom_callbacks。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // setData → items 反映 string items（length / indexed / kind / type / iterator）。
    sandbox
        .execute(
            "globalThis.__dt = new DataTransfer();\
             __dt.setData('text/plain', 'hello');\
             __dt.setData('text/html', '<b>hi</b>');\
             globalThis.__len = __dt.items.length;\
             globalThis.__k0 = __dt.items[0].kind;\
             globalThis.__t0 = __dt.items[0].type;\
             globalThis.__kinds = [];\
             for (var it of __dt.items) { globalThis.__kinds.push(it.kind); }\
             globalThis.__kinds_csv = globalThis.__kinds.join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "2",
        "items.length"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__k0)").unwrap().value,
        "string",
        "items[0].kind"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t0)").unwrap().value,
        "text/plain",
        "items[0].type"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__kinds_csv)").unwrap().value,
        "string,string",
        "for-of 迭代 items 得 2 个 string item"
    );

    // getAsString 回调字符串内容。
    sandbox
        .execute("__dt.items[0].getAsString(function(s){ globalThis.__str = s; });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__str)").unwrap().value,
        "hello",
        "DataTransferItem.getAsString 回调传字符串内容"
    );

    // items.add(data, type) → 回写到 dataTransfer（getData 反映）+ 返 string item。
    sandbox
        .execute(
            "globalThis.__added = __dt.items.add('world', 'text/uri-list');\
             globalThis.__getData = __dt.getData('text/uri-list');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__getData)").unwrap().value,
        "world",
        "items.add(data, type) 经 setData 回写（getData 反映）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__added.kind)").unwrap().value,
        "string",
        "items.add 返 string DataTransferItem"
    );

    // items.add(file) → file item（File-like）+ getAsFile 返该对象；string item getAsFile 返 null。
    sandbox
        .execute(
            "globalThis.__file = { size: 10, type: 'image/png', name: 'a.png' };\
             globalThis.__fItem = __dt.items.add(globalThis.__file);\
             globalThis.__fKind = globalThis.__fItem.kind;\
             globalThis.__fBack = globalThis.__fItem.getAsFile() === globalThis.__file;\
             globalThis.__strFile = __dt.items[0].getAsFile();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fKind)").unwrap().value,
        "file",
        "items.add(File-like) 返 file item"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fBack)").unwrap().value,
        "true",
        "file item.getAsFile() 返原 File-like 对象"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__strFile)").unwrap().value,
        "null",
        "string item.getAsFile() 返 null"
    );

    // items.clear() → 清空（length=0；getData 返空）。
    sandbox
        .execute(
            "globalThis.__cleared = 0;\
             var snap = __dt.items; snap.clear();\
             globalThis.__afterClear = __dt.items.length;\
             globalThis.__afterClearData = __dt.getData('text/plain');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__afterClear)").unwrap().value,
        "0",
        "items.clear() 清空（length=0）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__afterClearData)").unwrap().value,
        "",
        "clear 后 getData 返空"
    );
}

#[test]
fn test_fontface_load_r2949() {
    // R2949 FontFace（CSS Font Loading API face 层）：constructor + descriptors + .status + .load() Promise
    // + .loaded getter + document.fonts.add/delete/size/values。.load() 经 host `__zw_load_font` 投递 + host
    // `resolve_async_callback(id, "ok"/"err")` 解析 Promise。本测试用 mock __zw_load_font（捕获 resolve_id
    // 经 __zw_pending 键读出）+ 直调 resolve_async_callback 模拟 runtime 完成加载。
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
    // mock __zw_load_font：no-op（runtime 侧 fetch+register 由 resolve_async_callback 模拟）。
    sandbox.register_callback("__zw_load_font", Box::new(|_args: &[String]| String::new()));

    // constructor + descriptors + 初始 status='unloaded'。
    sandbox
        .execute(
            "globalThis.__ff = new FontFace('MyFont', 'https://example.com/f.woff2',\
             { style: 'italic', weight: 'bold' });\
             globalThis.__fam = __ff.family;\
             globalThis.__st = __ff.style;\
             globalThis.__wt = __ff.weight;\
             globalThis.__status0 = __ff.status;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fam)").unwrap().value,
        "MyFont",
        "FontFace.family"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__st)").unwrap().value,
        "italic",
        "FontFace.style"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__wt)").unwrap().value,
        "bold",
        "FontFace.weight"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__status0)").unwrap().value,
        "unloaded",
        "FontFace 初始 status='unloaded'"
    );

    // .load() 投递请求（__zw_load_font mock）→ Promise pending，status='loading'，__zw_pending 多一个 '__ff_' 键。
    sandbox
        .execute(
            "globalThis.__beforeKeys = Object.keys(globalThis.__zw_pending).length;\
             __ff.load().then(function(f){ globalThis.__loaded = f.family + ':' + f.status; },\
             function(e){ globalThis.__loaded = 'reject:' + e.message; });\
             globalThis.__statusLoading = __ff.status;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__statusLoading)").unwrap().value,
        "loading",
        ".load() 后 status='loading'"
    );
    // 取 resolve_id（__zw_pending 的 '__ff_' 键）。
    let resolve_id = sandbox
        .execute(
            "String(Object.keys(globalThis.__zw_pending).filter(function(k){return k.indexOf('__ff_')===0;})[0] || '')",
        )
        .unwrap()
        .value;
    assert!(
        !resolve_id.is_empty(),
        "FontFace.load() 投递请求（__zw_pending 含 __ff_ 键）"
    );

    // host 完成加载（resolve "ok"）→ Promise resolve，status='loaded'。microtask 在下次 execute 排空。
    sandbox.resolve_async_callback(&resolve_id, "ok");
    assert_eq!(
        sandbox.execute("String(globalThis.__loaded)").unwrap().value,
        "MyFont:loaded",
        "host resolve 'ok' → FontFace.load() Promise resolve，status='loaded'"
    );

    // document.fonts.add/delete/size + values 迭代。
    sandbox
        .execute(
            "document.fonts.add(__ff);\
             globalThis.__size = document.fonts.size;\
             globalThis.__first = document.fonts.values().next().value.family;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__size)").unwrap().value,
        "1",
        "document.fonts.add → size=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__first)").unwrap().value,
        "MyFont",
        "document.fonts.values() 迭代得添加的 FontFace"
    );

    // .loaded getter 返 load Promise（已 loaded → 立即 resolve 同一 Promise）。
    sandbox
        .execute("__ff.loaded.then(function(){ globalThis.__loadedAgain = 'yes'; });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__loadedAgain)").unwrap().value,
        "yes",
        ".loaded getter 返 load Promise（已 loaded 复用同一 Promise resolve）"
    );

    // load 失败（新 FontFace，resolve "err"）→ Promise reject，status='error'。
    sandbox
        .execute(
            "globalThis.__ff2 = new FontFace('BadFont', 'https://example.com/missing.woff2');\
             globalThis.__id2 = '';\
             __ff2.load().then(function(f){ globalThis.__res2='ok:'+f.status; },\
             function(e){ globalThis.__res2='reject:'+e.message; });\
             globalThis.__id2 = Object.keys(globalThis.__zw_pending).filter(function(k){return k.indexOf('__ff_')===0;})[0];",
        )
        .unwrap();
    let id2 = sandbox.execute("String(globalThis.__id2)").unwrap().value;
    sandbox.resolve_async_callback(&id2, "err:fetch");
    assert_eq!(
        sandbox.execute("String(globalThis.__res2)").unwrap().value,
        "reject:Failed to load FontFace \"BadFont\" from https://example.com/missing.woff2",
        "host resolve 'err' → Promise reject，status='error'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ff2.status)").unwrap().value,
        "error",
        "失败 FontFace status='error'"
    );
}

#[test]
fn test_fontface_reflect_atfontface_r2950() {
    // R2950 host→JS：__zw_add_fontface 把已加载 @font-face 字体反映为 FontFace 加入 document.fonts
    //（补全 FontFaceSet 语义——set 含文档 @font-face 字体）。按 family 去重；status='loaded'/'error'。
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

    // 初始 document.fonts 为空（size=0）。
    assert_eq!(
        sandbox.execute("String(document.fonts.size)").unwrap().value,
        "0",
        "初始 document.fonts 空"
    );

    // host 反映两个 @font-face 字体（一个 loaded，一个 error）→ set 含 2 个 FontFace。
    sandbox
        .execute(
            "__zw_add_fontface('MyFont', 'loaded');\
             __zw_add_fontface('BadFont', 'error');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(document.fonts.size)").unwrap().value,
        "2",
        "反映 2 个 @font-face 字体 → size=2"
    );
    // 收集 family + status（经 values 迭代）。
    sandbox
        .execute(
            "globalThis.__families = []; globalThis.__statuses = {};\
             document.fonts.forEach(function(f){ globalThis.__families.push(f.family); globalThis.__statuses[f.family] = f.status; });",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__families.sort().join(','))")
            .unwrap()
            .value,
        "BadFont,MyFont",
        "document.fonts 迭代得反映的 family"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__statuses['MyFont'])")
            .unwrap()
            .value,
        "loaded",
        "成功加载的 FontFace status='loaded'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__statuses['BadFont'])")
            .unwrap()
            .value,
        "error",
        "失败的 FontFace status='error'"
    );

    // 按 family 去重：重复 __zw_add_fontface 同 family 不重复加。
    sandbox.execute("__zw_add_fontface('MyFont', 'loaded');").unwrap();
    assert_eq!(
        sandbox.execute("String(document.fonts.size)").unwrap().value,
        "2",
        "同 family 重复反映去重（size 不增）"
    );
}

#[test]
fn test_xmlserializer_importnode_r2818() {
    // R2818：XMLSerializer.serializeToString + document.adoptNode/importNode。serializeToString 委托节点
    // outerHTML（元素）/ nodeValue（text·comment）/ documentElement（document）；adoptNode 单文档 identity；
    // importNode 委托 cloneNode（深/浅克隆独立性）。
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
        "<html><body><div id='src' class='row'><span>hi</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // XMLSerializer 构造器 + serializeToString(元素) 含 tag + class。
    sandbox
        .execute(
            "globalThis.__xs = new XMLSerializer();\
             globalThis.__isFn = typeof XMLSerializer.prototype.serializeToString === 'function';\
             globalThis.__el = document.querySelector('#src');\
             globalThis.__ser = __xs.serializeToString(__el);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__isFn)").unwrap().value,
        "true",
        "serializeToString 为 function"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ser.indexOf('<div') >= 0)")
            .unwrap()
            .value,
        "true",
        "serializeToString(元素) 含 '<div'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ser.indexOf('class=\"row\"') >= 0 || globalThis.__ser.indexOf(\"class='row'\") >= 0)")
            .unwrap()
            .value,
        "true",
        "serializeToString(元素) 含 class 属性"
    );

    // serializeToString(text/comment) → nodeValue/data。
    sandbox
        .execute(
            "globalThis.__tn = document.createTextNode('hello');\
             globalThis.__cm = document.createComment('note');\
             globalThis.__serTn = __xs.serializeToString(__tn);\
             globalThis.__serCm = __xs.serializeToString(__cm);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__serTn)").unwrap().value,
        "hello",
        "serializeToString(text)=nodeValue"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__serCm)").unwrap().value,
        "note",
        "serializeToString(comment)=data"
    );

    // document.adoptNode → 返同对象（identity）。
    sandbox
        .execute("globalThis.__adopted = (document.adoptNode(__el) === __el);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__adopted)").unwrap().value,
        "true",
        "adoptNode 单文档 identity 返同对象"
    );

    // document.importNode 深/浅克隆：副本独立于源 + deep 含子树 span。
    sandbox
        .execute(
            "globalThis.__shallow = document.importNode(__el, false);\
             globalThis.__deep = document.importNode(__el, true);\
             globalThis.__deepHasSpan = __deep.outerHTML.indexOf('<span') >= 0;\
             globalThis.__indep = (__deep !== __el);\
             globalThis.__shallowTag = __shallow.tagName;\
             globalThis.__shallowNeqDeep = (__shallow !== __deep);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__deepHasSpan)").unwrap().value,
        "true",
        "importNode(deep=true) 含子树 span"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__indep)").unwrap().value,
        "true",
        "importNode 副本独立于源"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__shallowTag)").unwrap().value,
        "DIV",
        "importNode(浅) 仍为 DIV 元素（外层克隆）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__shallowNeqDeep)").unwrap().value,
        "true",
        "浅/深克隆互异"
    );
}

#[test]
fn test_isequalnode_r2819() {
    // R2819：node.isEqualNode——节点结构相等（node-equality 三件套最后一块）。经 _nodeSig 序列化签名比对
    //（元素 outerHTML / text·comment nodeValue）。属性序敏感（spec 序无关，实际库一致）。
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
         <div id='wrap'>\
         <div class='x'><span>hi</span></div>\
         <div class='x'><span>hi</span></div>\
         <div class='y'><span>hi</span></div>\
         <div class='x'><span>bye</span></div>\
         </div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 同结构（a==b，均无 id 冲突）true / 不同 class（a==c）false / 不同子文本（a==d）false / 自身 true / null false。
    sandbox
        .execute(
            "globalThis.__kids = document.querySelector('#wrap').children;\
             globalThis.__a = __kids[0]; globalThis.__b = __kids[1];\
             globalThis.__c = __kids[2]; globalThis.__d = __kids[3];\
             globalThis.__eq_ab = __a.isEqualNode(__b);\
             globalThis.__eq_ac = __a.isEqualNode(__c);\
             globalThis.__eq_ad = __a.isEqualNode(__d);\
             globalThis.__eq_aa = __a.isEqualNode(__a);\
             globalThis.__eq_null = __a.isEqualNode(null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_ab)").unwrap().value,
        "true",
        "同结构（class+子树）isEqualNode true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_ac)").unwrap().value,
        "false",
        "不同 class 不等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_ad)").unwrap().value,
        "false",
        "不同子文本不等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_aa)").unwrap().value,
        "true",
        "自身相等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_null)").unwrap().value,
        "false",
        "isEqualNode(null) false"
    );

    // text 节点：同 nodeValue 等 / 不同不等 / text≠comment（同 nodeValue 但 nodeType 异）。
    sandbox
        .execute(
            "globalThis.__t1 = document.createTextNode('x');\
             globalThis.__t2 = document.createTextNode('x');\
             globalThis.__t3 = document.createTextNode('y');\
             globalThis.__cm = document.createComment('x');\
             globalThis.__eq_tt = __t1.isEqualNode(__t2);\
             globalThis.__eq_t12t3 = __t1.isEqualNode(__t3);\
             globalThis.__eq_tcm = __t1.isEqualNode(__cm);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_tt)").unwrap().value,
        "true",
        "同 text nodeValue 相等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_t12t3)").unwrap().value,
        "false",
        "不同 text nodeValue 不等"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eq_tcm)").unwrap().value,
        "false",
        "text≠comment（nodeType 异）"
    );
}

#[test]
fn test_navigator_geolocation_r2820() {
    // R2820：navigator.geolocation——地理位置 API（地图/天气/本地化 feature-detect 后调 getCurrentPosition）。
    // headless 无真 GPS → fake 零坐标位置（latitude/longitude 0，accuracy Infinity = 无精度承诺），让 location
    // 脚本走 success 路径不抛。getCurrentPosition/watchPosition 经 _defer microtask 异步调 success（execute 末
    // checkpoint 派发，下 execute 可读，同 R2774/R2814）；watchPosition 返唯一 watch id；clearWatch no-op。
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

    // navigator.geolocation 存在 + 三方法为 function。
    assert_eq!(
        sandbox.execute("typeof navigator.geolocation").unwrap().value,
        "object",
        "navigator.geolocation 存在"
    );
    assert_eq!(
        sandbox
            .execute("typeof navigator.geolocation.getCurrentPosition")
            .unwrap()
            .value,
        "function",
        "getCurrentPosition 为 function"
    );
    assert_eq!(
        sandbox
            .execute("typeof navigator.geolocation.watchPosition")
            .unwrap()
            .value,
        "function",
        "watchPosition 为 function"
    );
    assert_eq!(
        sandbox
            .execute("typeof navigator.geolocation.clearWatch")
            .unwrap()
            .value,
        "function",
        "clearWatch 为 function"
    );

    // getCurrentPosition 经 microtask 调 success 携 fake 零坐标位置（__lat 初值 -999 证回调真触发）。
    sandbox
        .execute(
            "globalThis.__lat = -999;\
             navigator.geolocation.getCurrentPosition(function(p){\
               globalThis.__lat = p.coords.latitude;\
               globalThis.__lng = p.coords.longitude;\
               globalThis.__alt = String(p.coords.altitude);\
               globalThis.__acc = String(p.coords.accuracy);\
               globalThis.__hdg = String(p.coords.heading);\
               globalThis.__spd = String(p.coords.speed);\
               globalThis.__ts = p.timestamp;\
             });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__lat)").unwrap().value,
        "0",
        "getCurrentPosition success coords.latitude===0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__lng)").unwrap().value,
        "0",
        "coords.longitude===0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__alt)").unwrap().value,
        "null",
        "coords.altitude===null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__acc)").unwrap().value,
        "Infinity",
        "coords.accuracy===Infinity（无精度承诺）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hdg)").unwrap().value,
        "null",
        "coords.heading===null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__spd)").unwrap().value,
        "null",
        "coords.speed===null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ts)").unwrap().value,
        "0",
        "timestamp===0"
    );

    // watchPosition 返唯一正 watch id（首个为 1）+ 经 microtask 调 success；clearWatch no-op 不抛。
    sandbox
        .execute(
            "globalThis.__id = navigator.geolocation.watchPosition(function(p){ globalThis.__wl = p.coords.latitude; });\
             globalThis.__id2 = navigator.geolocation.watchPosition(function(){});\
             globalThis.__cleared = 'no';\
             try { navigator.geolocation.clearWatch(globalThis.__id); globalThis.__cleared = 'yes'; } catch(_e){}",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__id)").unwrap().value,
        "1",
        "watchPosition 返唯一 id（首个为 1）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__id2)").unwrap().value,
        "2",
        "watchPosition 多次返递增 id"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cleared)").unwrap().value,
        "yes",
        "clearWatch no-op 不抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__wl)").unwrap().value,
        "0",
        "watchPosition success coords.latitude===0"
    );

    // getCurrentPosition 无 success 回调静默 no-op 不抛（lenient，非真 GPS 不强求回调）。
    sandbox
        .execute("globalThis.__n = 'no'; try { navigator.geolocation.getCurrentPosition(); globalThis.__n = 'yes'; } catch(_e){}")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__n)").unwrap().value,
        "yes",
        "getCurrentPosition 无回调 lenient 不抛"
    );
}

#[test]
fn test_performance_mark_measure_observer_r2821() {
    // R2821：Performance API 扩展（performance.mark/measure + entry buffer + PerformanceObserver）。
    // analytics/RUM（web-vitals/Sentry/GA）高频。mark/measure 产 PerformanceEntry 存 entry buffer；
    // PerformanceObserver observe 匹配 entryType 时经 _defer microtask 异步派发（execute 末 checkpoint，
    // 下 execute 可读，同 R2774/R2814）。dom_bridge.rs 的 PerformanceObserver/mark/measure 为 A 代死代码
    // （generate_dom_api_polyfill 无生产调用方），生产路径仅注入本 shim——故补到 B 代 shim。
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

    // performance.mark 产 entry（entryType='mark'/duration 0）+ entry buffer 可读。
    sandbox
        .execute("globalThis.__mk = performance.mark('a'); performance.mark('b');")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__mk.entryType)").unwrap().value,
        "mark",
        "mark entry entryType='mark'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__mk.duration)").unwrap().value,
        "0",
        "mark entry duration 0"
    );
    assert_eq!(
        sandbox
            .execute("String(performance.getEntriesByType('mark').length)")
            .unwrap()
            .value,
        "2",
        "entry buffer 含 2 mark"
    );
    assert_eq!(
        sandbox
            .execute("String(performance.getEntriesByName('a').length)")
            .unwrap()
            .value,
        "1",
        "getEntriesByName('a')"
    );

    // performance.measure 计算 duration = mark(b).start - mark(a).start（>=0）；从原点 measure duration>=0；
    // 未知 mark 名抛 TypeError。
    sandbox
        .execute(
            "globalThis.__ms = performance.measure('ab', 'a', 'b');\
             globalThis.__mo = performance.measure('from-origin').duration >= 0;\
             globalThis.__err = 'no';\
             try { performance.measure('x', 'missing'); } catch(e){ globalThis.__err = (e instanceof TypeError) ? 'TypeError' : 'other'; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ms.entryType)").unwrap().value,
        "measure",
        "measure entry entryType='measure'"
    );
    let dur = sandbox.execute("String(globalThis.__ms.duration)").unwrap().value;
    let dur_n: f64 = dur.parse().unwrap_or(-1.0);
    assert!(dur_n >= 0.0, "measure duration >= 0（a 先于 b 标记）, got {}", dur);
    assert_eq!(
        sandbox.execute("String(globalThis.__mo)").unwrap().value,
        "true",
        "measure 从原点 duration>=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__err)").unwrap().value,
        "TypeError",
        "measure 引用未知 mark 名抛 TypeError"
    );

    // clearMarks / clearMeasures 清 buffer。
    sandbox
        .execute("performance.clearMarks(); performance.clearMeasures();")
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(performance.getEntries().length)")
            .unwrap()
            .value,
        "0",
        "clearMarks+clearMeasures 清空 entry buffer"
    );

    // PerformanceObserver：typeof function + supportedEntryTypes 含 mark/measure。
    assert_eq!(
        sandbox.execute("typeof PerformanceObserver").unwrap().value,
        "function",
        "PerformanceObserver 存在"
    );
    assert_eq!(
        sandbox
            .execute("String(PerformanceObserver.supportedEntryTypes.indexOf('measure') !== -1)")
            .unwrap()
            .value,
        "true",
        "supportedEntryTypes 含 'measure'"
    );

    // observe({entryTypes:['mark']}) + mark → 经 microtask 派发 list.getEntries() 含两 mark 名（排序）。
    sandbox
        .execute(
            "globalThis.__got = 'none';\
             var obs = new PerformanceObserver(function(list){\
               globalThis.__got = list.getEntries().map(function(e){ return e.name; }).sort().join(',');\
             });\
             obs.observe({ entryTypes: ['mark'] });\
             performance.mark('m1'); performance.mark('m2');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__got)").unwrap().value,
        "m1,m2",
        "observer（entryTypes）经 microtask 收两 mark"
    );

    // observe({type:'measure'}) → measure 'mz' 经 microtask 派发（单独 execute 让 flush 先于 disconnect 跑）。
    sandbox
        .execute(
            "globalThis.__g2 = 'none';\
             var obs2 = new PerformanceObserver(function(list){\
               var e = list.getEntries();\
               globalThis.__g2 = e.length + ':' + (e[0] && e[0].name);\
             });\
             obs2.observe({ type: 'measure' });\
             performance.measure('mz');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__g2)").unwrap().value,
        "1:mz",
        "observer（type form）经 microtask 收 measure 'mz'"
    );
    // disconnect 后后续 measure 'mz2' 不再派发（__g2 保持 disconnect 前值）。
    sandbox
        .execute("obs2.disconnect(); performance.measure('mz2');")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__g2)").unwrap().value,
        "1:mz",
        "disconnect 后不再派发（__g2 未变）"
    );

    // takeRecords 取并清缓冲（observe + mark 后 takeRecords 返该 entry）。
    sandbox
        .execute(
            "var obs3 = new PerformanceObserver(function(){});\
             obs3.observe({ entryTypes: ['mark'] });\
             performance.mark('tr');\
             globalThis.__rec = obs3.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rec.length)").unwrap().value,
        "1",
        "takeRecords 返缓冲 entry"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rec[0].name)").unwrap().value,
        "tr",
        "takeRecords entry name 'tr'"
    );
}

#[test]
fn test_element_replace_children_r2822() {
    // R2822：Element.replaceChildren(...nodesOrStrings)——移除全部现有子 + 追加新子（clear-and-populate
    // 原子语义，Vue3/lit/Svelte/手写代码高频）。清空经 SetInnerHtml('')，追加复用 _appendVariadic（与 append 共用）。
    // 验证经 apply_mutations_to_html_with_handles（proxy 读 stale 快照，故核 mutation 产出的 HTML）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body>\
        <div id='t'>old1<span>old2</span>old3</div>\
        <div id='u'>keep1<p>keep2</p>keep3</div>\
        <div id='v'>oldtext</div>\
        </body></html>"
        .to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 三种用法（不同元素，互不干扰）：#t 清空+追加节点+字符串；#u 无参仅清空；#v 纯字符串清空+追加。
    sandbox
        .execute(
            "var b=document.createElement('b'); var i=document.createElement('i');\
             document.querySelector('#t').replaceChildren(b, 'mid', i);\
             document.querySelector('#u').replaceChildren();\
             document.querySelector('#v').replaceChildren('hello');",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();

    // #t：清空 old1/old2/old3 + 追加 b, mid, i（参数序）。
    assert!(
        out.contains("<div id=\"t\"><b></b>mid<i></i></div>"),
        "#t 清空旧子+追加新子（b,mid,i 参数序）\n{out}"
    );
    // #u：无参 → 内容空。
    assert!(
        out.contains("<div id=\"u\"></div>"),
        "#u 无参 replaceChildren 清空\n{out}"
    );
    // #v：纯字符串清空+追加。
    assert!(out.contains("<div id=\"v\">hello</div>"), "#v 纯字符串清空+追加\n{out}");
    // 旧内容全部消失（证清空生效）。
    assert!(
        !out.contains("old1") && !out.contains("old2") && !out.contains("keep1") && !out.contains("oldtext"),
        "旧子应全清空\n{out}"
    );
}

#[test]
fn test_character_data_methods_r2823() {
    // R2823：CharacterData 数据编辑（appendData/deleteData/insertData/replaceData/substringData + length）
    // + Text.splitText。仅 handle-based 文本/注释节点（createTextNode/createComment）。读经
    // query_text_from_mutations 反向 replay 取最新值（多次编辑 compose 正确），写追加 SetTextOnHandle。
    // contentEditable 编辑库（ProseMirror/Slate/Quill）+ Range/Selection 高频。
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

    // appendData / length / substringData / deleteData / insertData / replaceData 链式 compose。
    // 'Hello' +appendData(' World')→'Hello World'(len 11) →substringData(6,5)='World'
    // →deleteData(0,6)→'World' →insertData(0,'JS ')→'JS World' →replaceData(0,3,'Hi')→'HiWorld'（'JS ' 含空格 3 字符被 'Hi' 替）
    sandbox
        .execute(
            "globalThis.__t = document.createTextNode('Hello');\
             globalThis.__t.appendData(' World');\
             globalThis.__len = globalThis.__t.length;\
             globalThis.__sub = globalThis.__t.substringData(6, 5);\
             globalThis.__t.deleteData(0, 6);\
             globalThis.__afterDel = globalThis.__t.data;\
             globalThis.__t.insertData(0, 'JS ');\
             globalThis.__t.replaceData(0, 3, 'Hi');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "11",
        "appendData 后 length=11（'Hello World'）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sub)").unwrap().value,
        "World",
        "substringData(6,5)='World'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__afterDel)").unwrap().value,
        "World",
        "deleteData(0,6)→'World'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t.data)").unwrap().value,
        "HiWorld",
        "insertData(0,'JS ')+replaceData(0,3,'Hi')→'HiWorld'（'JS ' 含空格被 'Hi' 替）"
    );

    // splitText：原节点保 [0,offset)，返新 text 节点含 [offset,)；两节点均 handle-based 可读。
    sandbox
        .execute(
            "globalThis.__t2 = document.createTextNode('abcdef');\
             globalThis.__tail = globalThis.__t2.splitText(2);\
             globalThis.__head = globalThis.__t2.data;\
             globalThis.__taildata = globalThis.__tail.data;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__head)").unwrap().value,
        "ab",
        "splitText(2) 原节点保 'ab'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__taildata)").unwrap().value,
        "cdef",
        "splitText(2) 返新节点 'cdef'"
    );

    // CharacterData mixin 亦适用 comment 节点（appendData）。
    sandbox
        .execute(
            "globalThis.__c = document.createComment('cmt');\
             globalThis.__c.appendData('!');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__c.data)").unwrap().value,
        "cmt!",
        "comment appendData（CharacterData mixin）"
    );
}

#[test]
fn test_page_visibility_and_focus_r2824() {
    // R2824：Page Visibility + 焦点状态——document.hidden / visibilityState / hasFocus()
    // （+ webkit 前缀 legacy）。analytics/RUM 高频（GA 读 visibilityState/hidden，hasFocus gate 操作）。
    // headless 恒「可见 + 已聚焦」：hidden=false / visibilityState='visible' / hasFocus=true。
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

    // 标准属性：hidden=false / visibilityState='visible' / hasFocus()=true。
    assert_eq!(
        sandbox.execute("String(document.hidden)").unwrap().value,
        "false",
        "document.hidden === false（headless 可见）"
    );
    assert_eq!(
        sandbox.execute("String(document.visibilityState)").unwrap().value,
        "visible",
        "document.visibilityState === 'visible'"
    );
    assert_eq!(
        sandbox.execute("String(document.hasFocus())").unwrap().value,
        "true",
        "document.hasFocus() === true（headless 已聚焦）"
    );
    // webkit 前缀（legacy analytics / 旧 GA feature-detect）。
    assert_eq!(
        sandbox.execute("String(document.webkitHidden)").unwrap().value,
        "false",
        "document.webkitHidden === false（legacy 前缀）"
    );
    assert_eq!(
        sandbox.execute("String(document.webkitVisibilityState)").unwrap().value,
        "visible",
        "document.webkitVisibilityState === 'visible'（legacy 前缀）"
    );
}

#[test]
fn test_constraint_validation_r2825() {
    // R2825：Constraint Validation API——checkValidity/reportValidity/setCustomValidity/validity/
    // validationMessage/willValidate。表单校验库高频（checkValidity gate submit / setCustomValidity
    // 自定义错误 / validity.valid 读）。customError 由 setCustomValidity 跟踪；原生约束 headless 不强制
    // （permissive valid）。checkValidity invalid 时派发 'invalid' 事件。
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
        "<html><body><input id='i'><input id='j'></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 默认 valid：validity.valid=true / customError=false / validationMessage='' / checkValidity=true / willValidate=true。
    sandbox
        .execute(
            "globalThis.__i = document.querySelector('#i');\
             globalThis.__defValid = globalThis.__i.validity.valid;\
             globalThis.__defCustom = globalThis.__i.validity.customError;\
             globalThis.__defMsg = globalThis.__i.validationMessage;\
             globalThis.__defCv = globalThis.__i.checkValidity();\
             globalThis.__wv = globalThis.__i.willValidate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__defValid)").unwrap().value,
        "true",
        "默认 validity.valid=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__defCustom)").unwrap().value,
        "false",
        "默认 customError=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__defMsg)").unwrap().value,
        "",
        "默认 validationMessage=''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__defCv)").unwrap().value,
        "true",
        "默认 checkValidity()=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__wv)").unwrap().value,
        "true",
        "willValidate=true"
    );

    // setCustomValidity('err') → customError=true / valid=false / validationMessage='err' / checkValidity=false。
    sandbox
        .execute(
            "globalThis.__i.setCustomValidity('err');\
             globalThis.__cvValid = globalThis.__i.validity.valid;\
             globalThis.__cvCustom = globalThis.__i.validity.customError;\
             globalThis.__cvMsg = globalThis.__i.validationMessage;\
             globalThis.__cvCheck = globalThis.__i.checkValidity();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cvValid)").unwrap().value,
        "false",
        "setCustomValidity 后 valid=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cvCustom)").unwrap().value,
        "true",
        "customError=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cvMsg)").unwrap().value,
        "err",
        "validationMessage='err'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cvCheck)").unwrap().value,
        "false",
        "checkValidity()=false"
    );

    // setCustomValidity('') 清空 → 恢复 valid。
    sandbox
        .execute(
            "globalThis.__i.setCustomValidity('');\
             globalThis.__clrValid = globalThis.__i.validity.valid;\
             globalThis.__clrCv = globalThis.__i.checkValidity();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__clrValid)").unwrap().value,
        "true",
        "setCustomValidity('') 恢复 valid=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__clrCv)").unwrap().value,
        "true",
        "清空后 checkValidity()=true"
    );

    // 'invalid' 事件在 checkValidity 失败时派发（per-element，#i 设错，监听 #i 的 invalid）。
    sandbox
        .execute(
            "globalThis.__fired = 'no';\
             globalThis.__i.addEventListener('invalid', function(){ globalThis.__fired = 'yes'; });\
             globalThis.__i.setCustomValidity('x');\
             globalThis.__i.checkValidity();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fired)").unwrap().value,
        "yes",
        "checkValidity 失败派发 'invalid' 事件"
    );

    // per-element 隔离：#j 未设 customValidity 仍 valid（#i 的 setCustomValidity 不影响 #j）。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#j').checkValidity())")
            .unwrap()
            .value,
        "true",
        "per-element 隔离：#j 仍 valid"
    );
}

#[test]
fn test_exec_command_and_select_r2826() {
    // R2826：legacy 编辑/剪贴板命令表面——document.execCommand / queryCommand* / element.select()。
    // 旧 copy 按钮 `el.select(); document.execCommand('copy')` + clipboard.js feature-detect
    // `queryCommandSupported('copy')` + contentEditable 编辑器 format 命令。headless 无真剪贴板/格式化
    // → permissive stub（execCommand→true / queryCommandSupported/Enabled→true / queryCommandValue→'' / select→undefined）。
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
        "<html><body><input id='i' value='txt'></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // execCommand / queryCommand* permissive stubs（legacy copy + feature-detect 不抛）。
    sandbox
        .execute(
            "globalThis.__copy = document.execCommand('copy');\
             globalThis.__bold = document.execCommand('bold');\
             globalThis.__sup = document.queryCommandSupported('copy');\
             globalThis.__en = document.queryCommandEnabled('copy');\
             globalThis.__val = document.queryCommandValue('fontSize');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__copy)").unwrap().value,
        "true",
        "execCommand('copy')→true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__bold)").unwrap().value,
        "true",
        "execCommand('bold')→true（format 不真应用，permissive）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sup)").unwrap().value,
        "true",
        "queryCommandSupported('copy')→true（feature-detect 通过）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__en)").unwrap().value,
        "true",
        "queryCommandEnabled→true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__val)").unwrap().value,
        "",
        "queryCommandValue→''"
    );

    // element.select() no-op 返 undefined（legacy copy 模式配对，不抛）。
    sandbox
        .execute("globalThis.__sel = document.querySelector('#i').select();")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__sel)").unwrap().value,
        "undefined",
        "element.select() no-op 返 undefined"
    );

    // 完整 legacy copy 模式不抛：select + execCommand('copy')。
    sandbox
        .execute(
            "globalThis.__ok = 'no';\
             try {\
               var el = document.querySelector('#i');\
               el.select();\
               document.execCommand('copy');\
               globalThis.__ok = 'yes';\
             } catch (e) { globalThis.__ok = 'err:' + e.message; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ok)").unwrap().value,
        "yes",
        "legacy copy 模式（select+execCommand('copy')）不抛"
    );
}

#[test]
fn test_element_animate_r2827() {
    // R2827：Element.animate（Web Animations API permissive stub）。headless 无真时间轴 → 动画瞬间完成
    //（playState 'running'→'finished' + finished Promise resolve + onfinish 触发，经 _defer microtask）。
    // modern 动画库（Framer Motion/GSAP/Lottie）feature-detect + 链式高频。关键帧不真应用（documented）。
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

    // animate 返 Animation 对象；初始 playState='running'（同步读，checkpoint 前）；duration 从 options 取。
    sandbox
        .execute(
            "globalThis.__anim = document.querySelector('#d').animate([{opacity:0},{opacity:1}], 200);\
             globalThis.__psInitial = globalThis.__anim.playState;\
             globalThis.__dur = globalThis.__anim.duration;\
             globalThis.__got = 'no';\
             globalThis.__anim.finished.then(function(a){ globalThis.__got = a.playState; });\
             globalThis.__anim.onfinish = function(){ globalThis.__of = 'fired'; };",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__psInitial)").unwrap().value,
        "running",
        "初始 playState='running'（同步读，checkpoint 前）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dur)").unwrap().value,
        "200",
        "duration 从 options（number）取 200"
    );
    // microtask checkpoint 后：playState='finished' + finished Promise resolve + onfinish 触发。
    assert_eq!(
        sandbox.execute("String(globalThis.__anim.playState)").unwrap().value,
        "finished",
        "microtask 后 playState='finished'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__got)").unwrap().value,
        "finished",
        "finished Promise resolve（携 playState='finished'）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__of)").unwrap().value,
        "fired",
        "onfinish 触发"
    );

    // options 对象形式（duration + id）+ 方法存在不抛 + cancel 切 idle。
    sandbox
        .execute(
            "globalThis.__a2 = document.querySelector('#d').animate([], { duration: 50, id: 'x' });\
             globalThis.__id = globalThis.__a2.id;\
             globalThis.__a2.cancel();\
             globalThis.__a2ps = globalThis.__a2.playState;\
             globalThis.__rev = typeof globalThis.__a2.reverse;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__id)").unwrap().value,
        "x",
        "options.id 提取"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__a2ps)").unwrap().value,
        "idle",
        "cancel() 切 idle"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rev)").unwrap().value,
        "function",
        "reverse 存在"
    );
}

#[test]
fn test_element_animate_real_playback_r2965() {
    // R2965：el.animate() 真关键帧应用（R2827 permissive stub 升级）。headless 瞬间完成模型下，finish 时按
    // fill 把末关键帧 CSS 属性写入元素 inline style（经样式→布局→渲染管线可见）；commitStyles() 显式提交当前态。
    // 验证：① fill:'forwards' → 末态应用（含多属性 + camelCase→kebab）；② commitStyles 不依赖 fill；
    // ③ fill:'none'（默认）不持久化（无 SetStyle）；④ 空关键帧 commitStyles no-op。既有语义不变（R2827 测覆盖）。
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
        "<html><body><div id='a'></div><div id='b'></div><div id='c'></div><div id='e'></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // ① fill:'forwards' + 多属性 + camelCase（backgroundColor→background-color）→ finish 后末态应用。
    sandbox
        .execute(
            "document.querySelector('#a').animate(\
               [{opacity:0, backgroundColor:'red'}, \
                {opacity:1, backgroundColor:'blue', transform:'scale(2)'}], \
               {duration:200, fill:'forwards'});",
        )
        .unwrap();
    // _defer microtask 在 execute 末 checkpoint 已 fire → SetStyle 已 push。apply_mutations_to_html 验真。
    let ms = mutations.lock().unwrap().clone();
    let out = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms).unwrap();
    assert!(out.contains("opacity: 1"), "fill:forwards 末态 opacity 应用\n{out}");
    assert!(
        out.contains("background-color: blue"),
        "camelCase backgroundColor→kebab 末态应用\n{out}"
    );
    assert!(
        out.contains("transform: scale(2)"),
        "末态 transform 应用\n{out}"
    );

    // ② commitStyles() 不依赖 fill：fill 默认 none（不自动持久化），但 commitStyles 显式提交末态。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "var a2 = document.querySelector('#b').animate([{color:'red'},{color:'blue'}], 100);\
             a2.commitStyles();",
        )
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let out2 = apply_mutations_to_html(&dom_html.lock().unwrap(), &ms2).unwrap();
    assert!(
        out2.contains("color: blue"),
        "commitStyles 提交末态 color（不依赖 fill）\n{out2}"
    );

    // ③ fill:'none'（默认）→ finish 后不持久化（无末态 SetStyle）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "document.querySelector('#c').animate([{opacity:0},{opacity:1}], {duration:50, fill:'none'});",
        )
        .unwrap();
    let set_style_none = mutations
        .lock()
        .unwrap()
        .iter()
        .filter(|m| matches!(m, DomMutation::SetStyle { .. }))
        .count();
    assert_eq!(
        set_style_none, 0,
        "fill:none 不持久化末态（无 SetStyle 变更）"
    );

    // ④ 空关键帧 → 无末态可应用（commitStyles no-op 不抛，无 SetStyle）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "var a4 = document.querySelector('#e').animate([], 30);\
             a4.commitStyles();\
             globalThis.__ok4 = 'ok';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ok4)").unwrap().value,
        "ok",
        "空关键帧 commitStyles 不抛"
    );
    let set_style_empty = mutations
        .lock()
        .unwrap()
        .iter()
        .filter(|m| matches!(m, DomMutation::SetStyle { .. }))
        .count();
    assert_eq!(set_style_empty, 0, "空关键帧无 SetStyle");
}

#[test]
fn test_element_get_client_rects_r2828() {
    // R2828：Element.getClientRects——旧返空 []（破 popper.js/tether 读 getClientRects()[0]）。
    // 现返单元素 bounding rect 数组（与 getBoundingClientRect 同源 _domRectFromId）。inline 多行收缩为
    // 单 rect（无 per-line-box，documented）；handle-only detached 无 layout → []。
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
    // mock rect bridge（rect bridge 不在 register_dom_callbacks）：selector → 固定 rect "10,20,100,50"；
    // handle（createElement，以 '__' 开头，detached）→ 空串（无 layout，匹配真实 detached 无几何语义）。
    sandbox.register_callback(
        "__zw_getBoundingClientRect",
        Box::new(|args| match args.first() {
            Some(s) if s.starts_with("__") => String::new(),
            _ => "10,20,100,50".to_string(),
        }),
    );

    // getClientRects 返数组 length=1 + [0] 含完整 DOMRect 字段（与 getBoundingClientRect 同源）。
    sandbox
        .execute(
            "globalThis.__rects = document.querySelector('#d').getClientRects();\
             globalThis.__len = globalThis.__rects.length;\
             globalThis.__r0 = globalThis.__rects[0];\
             globalThis.__keys = ['x','y','top','left','right','bottom','width','height']\
               .map(function(k){ return k + ':' + (globalThis.__r0[k] !== undefined ? 'y' : 'n'); }).join(',');\
             globalThis.__same = (function(){\
               var b = document.querySelector('#d').getBoundingClientRect();\
               return b.x === globalThis.__r0.x && b.width === globalThis.__r0.width && b.bottom === globalThis.__r0.bottom;\
             })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "1",
        "getClientRects 返数组 length=1（单 bounding rect）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__keys)").unwrap().value,
        "x:y,y:y,top:y,left:y,right:y,bottom:y,width:y,height:y",
        "[0] 含完整 DOMRect 字段（x/y/top/left/right/bottom/width/height）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__same)").unwrap().value,
        "true",
        "getClientRects[0] 与 getBoundingClientRect 同源 rect"
    );

    // spread 可迭代（[...rects] 取首元素）——现代库常用模式。
    sandbox
        .execute("globalThis.__spread = [...document.querySelector('#d').getClientRects()].length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__spread)").unwrap().value,
        "1",
        "getClientRects 可 spread 迭代（数组）"
    );

    // handle-only detached 元素（createElement，无 layout）→ []。
    sandbox
        .execute("globalThis.__detached = document.createElement('div').getClientRects().length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__detached)").unwrap().value,
        "0",
        "handle-only detached 无 layout → getClientRects 返 []"
    );
}

#[test]
fn test_form_elements_r2829() {
    // R2829：form.elements（HTMLFormControlsCollection）+ form.length + namedItem。表单序列化/校验库
    //（jQuery serialize / FormData / 校验库迭代）高频。仅 HTMLFormElement（gate）；非 form → undefined。
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
        "<html><body><form id='f'>\
         <input name='a' value='1'>\
         <select name='s'><option>x</option></select>\
         <textarea name='t'></textarea>\
         <button name='b'>go</button>\
         </form></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // form.elements：4 控件（input/select/textarea/button，tree order）+ length + 索引 + namedItem。
    sandbox
        .execute(
            "globalThis.__f = document.querySelector('#f');\
             globalThis.__els = globalThis.__f.elements;\
             globalThis.__len = globalThis.__els.length;\
             globalThis.__first = globalThis.__els[0].getAttribute('name');\
             globalThis.__last = globalThis.__els[3].getAttribute('name');\
             globalThis.__named = globalThis.__els.namedItem('s').tagName;\
             globalThis.__iter = (function(){\
               var names = [];\
               for (var i = 0; i < globalThis.__els.length; i++) names.push(globalThis.__els[i].getAttribute('name'));\
               return names.join(',');\
             })();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "4",
        "form.elements.length=4（input/select/textarea/button）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__first)").unwrap().value,
        "a",
        "elements[0]=input（tree order 首个）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__last)").unwrap().value,
        "b",
        "elements[3]=button（tree order 末个）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__named)").unwrap().value,
        "SELECT",
        "namedItem('s')=select（按 name 查）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__iter)").unwrap().value,
        "a,s,t,b",
        "迭代 form.elements 得 4 控件 name 序"
    );

    // form.length = 控件数。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#f').length)")
            .unwrap()
            .value,
        "4",
        "form.length=4"
    );

    // 非 form 元素 .elements → undefined（gate：仅 HTMLFormElement）。
    assert_eq!(
        sandbox.execute("String(document.body.elements)").unwrap().value,
        "undefined",
        "非 form 元素 .elements=undefined"
    );
}

#[test]
fn test_input_files_filelist_r2830() {
    // R2830：HTMLInputElement.files（空 FileList）。上传表单读 input.files.length / 迭代高频。
    // headless 无真文件 → 空 FileList（length 0 + item→null + 可迭代），让上传 JS 不抛（无文件→0 跳过上传）。
    // 仅 INPUT（_isTag gate）；非 input → undefined。
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
        "<html><body><input id='f' type='file'><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // input.files：空 FileList（length 0 + item(0)=null + spread 空）。
    sandbox
        .execute(
            "globalThis.__files = document.querySelector('#f').files;\
             globalThis.__len = globalThis.__files.length;\
             globalThis.__item = String(globalThis.__files.item(0));\
             globalThis.__spread = [...globalThis.__files].length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "0",
        "input.files.length=0（headless 无文件）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__item)").unwrap().value,
        "null",
        "input.files.item(0)=null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__spread)").unwrap().value,
        "0",
        "input.files 可 spread 迭代（空）"
    );

    // 非 input 元素 .files → undefined（gate：仅 INPUT）。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#d').files)")
            .unwrap()
            .value,
        "undefined",
        "非 input .files=undefined"
    );
}

#[test]
fn test_input_indeterminate_r2831() {
    // R2831：HTMLInputElement.indeterminate——JS-only IDL 布尔（非 reflected attr）。checkbox「全选」
    // tri-state UI 高频（父 checkbox 半选态）。per-element state（默认 false）；get/set round-trip。
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
        "<html><body><input id='c' type='checkbox'><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 默认 false；set true round-trip；set false 恢复。
    sandbox
        .execute(
            "globalThis.__cb = document.querySelector('#c');\
             globalThis.__def = globalThis.__cb.indeterminate;\
             globalThis.__cb.indeterminate = true;\
             globalThis.__afterTrue = globalThis.__cb.indeterminate;\
             globalThis.__cb.indeterminate = false;\
             globalThis.__afterFalse = globalThis.__cb.indeterminate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__def)").unwrap().value,
        "false",
        "默认 indeterminate=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__afterTrue)").unwrap().value,
        "true",
        "set true round-trip"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__afterFalse)").unwrap().value,
        "false",
        "set false 恢复"
    );

    // 「全选」tri-state 模式：3 子 checkbox 部分选 → 父 indeterminate。
    sandbox
        .execute(
            "globalThis.__children = [true, false, true];\
             globalThis.__all = globalThis.__children.every(function(v){ return v; });\
             globalThis.__any = globalThis.__children.some(function(v){ return v; });\
             globalThis.__cb.indeterminate = globalThis.__any && !globalThis.__all;\
             globalThis.__tri = globalThis.__cb.indeterminate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tri)").unwrap().value,
        "true",
        "tri-state：部分选 → 父 indeterminate=true"
    );

    // 非 input 元素 .indeterminate → undefined（gate：仅 INPUT）。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#d').indeterminate)")
            .unwrap()
            .value,
        "undefined",
        "非 input .indeterminate=undefined"
    );
}

#[test]
fn test_option_constructor_and_select_add_r2832() {
    // R2832：动态 select 填充表面——new Option() 构造器 + select.add() + option.text/label/defaultSelected。
    // 表单应用动态下拉（级联 select / 动态选项）高频。new Option 返 createElement('option') proxy；
    // select.add 追加 option；option.text/label/defaultSelected 读。
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
        "<html><body><select id='s'><option value='0'>zero</option></select></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // new Option(text, value, defaultSelected, selected)：tag=OPTION + text/value/selected 设置。
    sandbox
        .execute(
            "globalThis.__o = new Option('Apple', 'a', true, false);\
             globalThis.__tag = globalThis.__o.tagName;\
             globalThis.__text = globalThis.__o.text;\
             globalThis.__value = globalThis.__o.getAttribute('value');\
             globalThis.__defSel = globalThis.__o.defaultSelected;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tag)").unwrap().value,
        "OPTION",
        "new Option().tagName=OPTION"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__o.text)").unwrap().value,
        "Apple",
        "new Option text='Apple'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__value)").unwrap().value,
        "a",
        "new Option value='a'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__defSel)").unwrap().value,
        "true",
        "new Option defaultSelected=true（defaultSelected 参数）"
    );

    // select.add(option) 追加；动态填充后 select.value 可读新选项。
    sandbox
        .execute(
            "globalThis.__s = document.querySelector('#s');\
             globalThis.__s.add(new Option('Banana', 'b'));\
             globalThis.__s.add(new Option('Cherry', 'c'));",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<option value=\"b\">Banana</option>") && out.contains("<option value=\"c\">Cherry</option>"),
        "select.add 追加两 option（b=Banana, c=Cherry）\n{out}"
    );

    // option.label：有 label 属性用 label，否则回落 text。
    sandbox
        .execute(
            "globalThis.__oLab = new Option('TxtOnly');\
             globalThis.__lab1 = globalThis.__oLab.label;\
             globalThis.__oLab2 = new Option('inner');\
             globalThis.__oLab2.setAttribute('label', 'LabAttr');\
             globalThis.__lab2 = globalThis.__oLab2.label;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__lab1)").unwrap().value,
        "TxtOnly",
        "option.label 无属性回落 text"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__lab2)").unwrap().value,
        "LabAttr",
        "option.label 有属性用 label"
    );

    // new Option 无 new 调用亦可（返 proxy）。
    assert_eq!(
        sandbox.execute("String(Option('X','x').tagName)").unwrap().value,
        "OPTION",
        "Option() 无 new 亦返 OPTION proxy"
    );

    // handle-based option 的 .selected 读：4th 参数 selected=true → 设 selected 属性 → .selected=true
    //（经 __zw_has_attr_handle，句柄元素不在 HTML 快照，sel-based __zw_has_attr 对其恒 false）。
    sandbox
        .execute(
            "globalThis.__oS = new Option('Sel', 's', false, true);\
             globalThis.__selTrue = globalThis.__oS.selected;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__selTrue)").unwrap().value,
        "true",
        "new Option(...,selected=true) → .selected=true（handle has-attr 变体）"
    );
}

#[test]
fn test_element_get_animations_r3067() {
    // R3067：Element.getAnimations() / Document.getAnimations()（Web Animations API）——闭合 Element.animate()
    //（R2965）查询缺口。动画库（GSAP/Framer Motion/Lottie）调用 getAnimations() 查询/提交动画。headless 瞬间完成
    //（_defer microtask 后 playState='finished'）→ finished 动画仍含（可查询/commitStyles）；cancelled（idle）排除。
    // 验证：① el.getAnimations() per-element 计数 + 返回对象身份；② cancelled 排除；③ document.getAnimations() flat；
    // ④ 无动画元素返空数组。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='a'></div><div id='b'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // 建 3 动画：#a 两个（a2 cancel→idle），#b 一个。execute 末 microtask checkpoint 后 a1/b1=finished，a2=idle。
    sandbox
        .execute(
            "globalThis.__a1 = document.querySelector('#a').animate([{opacity:0},{opacity:1}], 100);\
             globalThis.__a2 = document.querySelector('#a').animate([{color:'red'},{color:'blue'}], 50);\
             globalThis.__a2.cancel();\
             globalThis.__b1 = document.querySelector('#b').animate([{transform:'none'},{transform:'scale(2)'}], 200);",
        )
        .unwrap();

    // ① #a.getAnimations()：返 1（a2 cancelled/idle 排除，a1 finished 含）+ 对象身份 === a1。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#a').getAnimations().length)")
            .unwrap()
            .value,
        "1",
        "#a.getAnimations() 返 1（a2 cancelled 排除）"
    );
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#a').getAnimations()[0] === globalThis.__a1)")
            .unwrap()
            .value,
        "true",
        "#a.getAnimations()[0] === a1（对象身份）"
    );

    // ② #b.getAnimations()：返 1（b1）。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#b').getAnimations().length)")
            .unwrap()
            .value,
        "1",
        "#b.getAnimations() 返 1（b1，per-element 隔离）"
    );

    // ③ document.getAnimations()：返全文档 flat（a1 + b1 = 2，a2 cancelled 排除）。
    assert_eq!(
        sandbox.execute("String(document.getAnimations().length)").unwrap().value,
        "2",
        "document.getAnimations() 返 2（a1 + b1 flat，a2 cancelled 排除）"
    );

    // ④ 无动画元素 getAnimations() 返空数组。
    assert_eq!(
        sandbox
            .execute("String(document.createElement('div').getAnimations().length)")
            .unwrap()
            .value,
        "0",
        "无动画元素 getAnimations() 返空数组"
    );
}
