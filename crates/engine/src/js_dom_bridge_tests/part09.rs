#[test]
fn test_document_collections_r2833() {
    // R2833：document 集合完整性 + 正确性——forms/scripts/images/links 已 land（_liveQueryCollection），
    // 本轮补缺 embeds/plugins/anchors + 修正 links（旧返全部 <a>，spec 仅 a[href]+area[href]）+ 加 has trap
    // 使 Array.prototype.map/forEach.call(coll) 迭代工作（HasProperty 判定索引存在性）。
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
         <form id='f1'></form><form id='f2'></form>\
         <script>var x=1;</script>\
         <img src='a.png'><img src='b.png'>\
         <a href='http://h'>L</a><a name='anc'>A</a>\
         <embed src='e.swf'><object data='o'></object>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    sandbox
        .execute(
            "globalThis.__forms = document.forms.length;\
             globalThis.__scripts = document.scripts.length;\
             globalThis.__images = document.images.length;\
             globalThis.__links = document.links.length;\
             globalThis.__anchors = document.anchors.length;\
             globalThis.__embeds = document.embeds.length;\
             globalThis.__plugins = document.plugins.length;\
             globalThis.__f0id = document.forms[0].getAttribute('id');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__forms)").unwrap().value,
        "2",
        "document.forms.length=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__scripts)").unwrap().value,
        "1",
        "document.scripts.length=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__images)").unwrap().value,
        "2",
        "document.images.length=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__links)").unwrap().value,
        "1",
        "document.links.length=1（仅 a[href]，不含 a[name]）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__anchors)").unwrap().value,
        "1",
        "document.anchors.length=1（仅 a[name]）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__embeds)").unwrap().value,
        "2",
        "document.embeds.length=2（embed+object）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__plugins)").unwrap().value,
        "2",
        "document.plugins.length=2（embed+object）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__f0id)").unwrap().value,
        "f1",
        "document.forms[0].id='f1'（索引访问）"
    );

    // 迭代支持（for...of / 索引遍历）——库常见用法。
    sandbox
        .execute("globalThis.__formIds = Array.prototype.map.call(document.forms, function(f){return f.getAttribute('id');}).join(',');")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__formIds)").unwrap().value,
        "f1,f2",
        "document.forms 可 Array.map 迭代（f1,f2）"
    );
}

#[test]
fn test_image_constructor_r2834() {
    // R2834：HTMLImageElement 构造器 new Image(w,h)——图片预加载 + DOM 挂载高频（WPT css-images /
    // css-backgrounds / content-visibility fixtures 经 new Image() 构造）。旧返 plain object（appendChild 失效、
    // 无 tagName）；现返 createElement('img') proxy（镜像 Option R2832），设 width/height 属性。
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
        "<html><body><div id='host'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // new Image() → tagName=IMG（真 DOM 元素，非旧 plain object）。
    sandbox
        .execute("globalThis.__img = new Image(); globalThis.__tag = globalThis.__img.tagName;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tag)").unwrap().value,
        "IMG",
        "new Image().tagName=IMG（真 img 元素）"
    );

    // new Image(100, 50) → width/height 属性设置（spec：构造器参数映射 width/height 内容属性）。
    sandbox
        .execute(
            "globalThis.__img2 = new Image(100, 50);\
             globalThis.__w = globalThis.__img2.getAttribute('width');\
             globalThis.__h = globalThis.__img2.getAttribute('height');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__w)").unwrap().value,
        "100",
        "new Image(100,50).width 属性=100"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__h)").unwrap().value,
        "50",
        "new Image(100,50).height 属性=50"
    );

    // src 反射 + appendChild DOM 挂载（旧 plain object 致 appendChild 失效——本轮修复核心）。
    sandbox
        .execute(
            "globalThis.__img3 = new Image();\
             globalThis.__img3.src = 'logo.png';\
             document.body.appendChild(globalThis.__img3);",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<img src=\"logo.png\">"),
        "new Image() 经 src 反射 + appendChild 挂入 body（旧 plain object 无效）\n{out}"
    );

    // onload/onerror 可设不抛（headless 无真图片加载，handler 不触发——settable 不抛即可；on* 读回属
    // element proxy 既有限制，非 Image 特有，不在本切片范围）。设后元素仍有效（tagName=IMG）。
    sandbox
        .execute(
            "globalThis.__img4 = new Image();\
             globalThis.__img4.onload = function(){};\
             globalThis.__img4.onerror = function(){};\
             globalThis.__tag4 = globalThis.__img4.tagName;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tag4)").unwrap().value,
        "IMG",
        "new Image() 设 onload/onerror 后仍为 IMG 元素（set 不抛）"
    );

    // 无 new 调用亦返 img proxy。
    assert_eq!(
        sandbox.execute("String(Image().tagName)").unwrap().value,
        "IMG",
        "Image() 无 new 亦返 IMG proxy"
    );
}

#[test]
fn test_audio_constructor_and_media_methods_r2835() {
    // R2835：HTMLAudioElement 构造器 new Audio([src]) + HTMLMediaElement play/pause/load/canPlayType no-op。
    // 音效/播客/通知音频构造高频（new Audio(url).play()）。headless 无音频设备——play 返 resolved Promise、
    // pause/load no-op、canPlayType 返 ''，使媒体 UI 主模式（play().then(...)）不抛。
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
        "<html><body><video id='v'></video></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // new Audio(src) → tagName=AUDIO + src 反射。
    sandbox
        .execute(
            "globalThis.__au = new Audio('beep.mp3');\
             globalThis.__auTag = globalThis.__au.tagName;\
             globalThis.__auSrc = globalThis.__au.getAttribute('src');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__auTag)").unwrap().value,
        "AUDIO",
        "new Audio().tagName=AUDIO"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__auSrc)").unwrap().value,
        "beep.mp3",
        "new Audio('beep.mp3') src 反射"
    );

    // play() 返 resolved Promise（spec 一致）；pause()/load()/canPlayType() no-op 不抛。
    // 经 microtask checkpoint（execute 末）派发 .then，下 execute 可读 __played。
    sandbox
        .execute(
            "globalThis.__au2 = new Audio('x.mp3');\
             globalThis.__playType = typeof globalThis.__au2.play;\
             globalThis.__au2.play().then(function(){ globalThis.__played = 'yes'; });\
             globalThis.__au2.pause();\
             globalThis.__au2.load();\
             globalThis.__cpt = globalThis.__au2.canPlayType('audio/mpeg');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__playType)").unwrap().value,
        "function",
        "audio.play 为 function"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cpt)").unwrap().value,
        "",
        "audio.canPlayType 返空串（保守不可播放）"
    );
    // play().then 回调经 microtask checkpoint 派发——下个 execute 读到 __played。
    sandbox.execute("void 0").unwrap(); // 触发 microtask checkpoint
    assert_eq!(
        sandbox.execute("String(globalThis.__played)").unwrap().value,
        "yes",
        "audio.play().then 回调经 microtask 派发（resolved Promise）"
    );

    // sel-based <video> 元素亦有 media 方法（play no-op 不抛）。
    sandbox
        .execute(
            "globalThis.__vid = document.querySelector('#v');\
             globalThis.__vidPlay = typeof globalThis.__vid.play;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__vidPlay)").unwrap().value,
        "function",
        "<video>.play 为 function（sel-based 亦有 media 方法）"
    );

    // 非 media 元素（如 div）无 play 方法（get-trap 返 undefined，gate 仅 AUDIO/VIDEO）。
    sandbox
        .execute(
            "globalThis.__div = document.createElement('div'); globalThis.__divPlay = typeof globalThis.__div.play;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__divPlay)").unwrap().value,
        "undefined",
        "div 无 play（gate 仅 AUDIO/VIDEO）"
    );

    // 无 new 调用亦返 audio proxy。
    assert_eq!(
        sandbox.execute("String(Audio().tagName)").unwrap().value,
        "AUDIO",
        "Audio() 无 new 亦返 AUDIO proxy"
    );
}

#[test]
fn test_input_value_as_number_r2836() {
    // R2836：input.valueAsNumber IDL 属性（getter+setter）——number/range 输入值↔数值转换（计算器/数量输入/
    // 校验库读 NaN 判非法）。getter：type=number/range parseFloat(value)（空/无效→NaN），其他 type→NaN；
    // setter：NaN→''，否则 String(n)→设 value。
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
         <input id='n' type='number' value='42'>\
         <input id='nf' type='number' value='3.14'>\
         <input id='ne' type='number' value=''>\
         <input id='nb' type='number' value='abc'>\
         <input id='t' type='text' value='99'>\
         <input id='r' type='range' value='7' min='0' max='10'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // getter：整数 / 浮点 / 空→NaN / 无效→NaN / 非 number type→NaN / range 亦可。
    sandbox
        .execute(
            "globalThis.__n = document.querySelector('#n').valueAsNumber;\
             globalThis.__nf = document.querySelector('#nf').valueAsNumber;\
             globalThis.__ne = isNaN(document.querySelector('#ne').valueAsNumber);\
             globalThis.__nb = isNaN(document.querySelector('#nb').valueAsNumber);\
             globalThis.__t = isNaN(document.querySelector('#t').valueAsNumber);\
             globalThis.__r = document.querySelector('#r').valueAsNumber;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__n)").unwrap().value,
        "42",
        "number input value=42 → valueAsNumber=42"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nf)").unwrap().value,
        "3.14",
        "number input value=3.14 → valueAsNumber=3.14"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ne)").unwrap().value,
        "true",
        "number input 空 value → valueAsNumber=NaN"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nb)").unwrap().value,
        "true",
        "number input value=abc → valueAsNumber=NaN"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t)").unwrap().value,
        "true",
        "text input → valueAsNumber=NaN（非 number/range）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r)").unwrap().value,
        "7",
        "range input value=7 → valueAsNumber=7"
    );

    // setter：number input 设数值 → value 字符串化；设 NaN → value=''。
    sandbox
        .execute(
            "var el = document.querySelector('#n');\
             el.valueAsNumber = 100;\
             globalThis.__setV = el.value;\
             el.valueAsNumber = NaN;\
             globalThis.__setNaN = el.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__setV)").unwrap().value,
        "100",
        "valueAsNumber=100 → value='100'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__setNaN)").unwrap().value,
        "",
        "valueAsNumber=NaN → value=''"
    );

    // setter 经 host value 属性 mutation（apply 后 value 属性更新）。
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<input id=\"n\" type=\"number\" value=\"\">"),
        "valueAsNumber=NaN setter 经 value 属性 mutation（apply 后 value=''）\n{out}"
    );
}

#[test]
fn test_anchor_url_decomposition_r2838() {
    // R2838：HTMLAnchorElement/HTMLAreaElement URL 分解 IDL 属性（href/pathname/search/hash/host/hostname/
    // port/protocol/origin）——经 __zw_parse_url 解析 href 属性取组件。SPA 路由/链接分析/analytics 高频。
    // a.href getter 返绝对 URL（区别 getAttribute 返原始串——jQuery .prop vs .attr）；相对 href 经 base 解析。
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
         <a id='abs' href='https://example.com:8080/path?q=1#h'>abs</a>\
         <a id='rel' href='/rel'>rel</a>\
         <a id='none'>nohref</a>\
         </body></html>"
            .to_string(),
    ));
    // 页面 base URL 用于相对 href 解析。
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/base/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 绝对 href 全组件解析。
    sandbox
        .execute(
            "var a = document.querySelector('#abs');\
             globalThis.__href = a.href;\
             globalThis.__protocol = a.protocol;\
             globalThis.__host = a.host;\
             globalThis.__hostname = a.hostname;\
             globalThis.__port = a.port;\
             globalThis.__pathname = a.pathname;\
             globalThis.__search = a.search;\
             globalThis.__hash = a.hash;\
             globalThis.__origin = a.origin;\
             globalThis.__rawHref = a.getAttribute('href');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__href)").unwrap().value,
        "https://example.com:8080/path?q=1#h",
        "a.href 绝对 URL"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__protocol)").unwrap().value,
        "https:",
        "a.protocol"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__host)").unwrap().value,
        "example.com:8080",
        "a.host"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hostname)").unwrap().value,
        "example.com",
        "a.hostname"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__port)").unwrap().value,
        "8080",
        "a.port"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pathname)").unwrap().value,
        "/path",
        "a.pathname"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__search)").unwrap().value,
        "?q=1",
        "a.search"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__hash)").unwrap().value,
        "#h",
        "a.hash"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__origin)").unwrap().value,
        "https://example.com:8080",
        "a.origin"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rawHref)").unwrap().value,
        "https://example.com:8080/path?q=1#h",
        "getAttribute('href') 原始串（绝对时同 href）"
    );

    // 相对 href：getAttribute 返原始 '/rel'，a.href 经 base 解析返绝对 URL；组件正确。
    sandbox
        .execute(
            "var r = document.querySelector('#rel');\
             globalThis.__relRaw = r.getAttribute('href');\
             globalThis.__relHref = r.href;\
             globalThis.__relPath = r.pathname;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__relRaw)").unwrap().value,
        "/rel",
        "相对 href getAttribute 返原始 '/rel'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__relHref)").unwrap().value,
        "http://test.local/rel",
        "相对 href a.href 经 base 解析返绝对 URL"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__relPath)").unwrap().value,
        "/rel",
        "相对 href a.pathname='/rel'"
    );

    // 无 href → 空值。
    sandbox
        .execute("globalThis.__noneHref = document.querySelector('#none').href;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__noneHref)").unwrap().value,
        "",
        "无 href 的 a.href=''"
    );

    // href setter：设 href 属性（经 set-trap catch-al __zw_set_attr 记 SetAttr mutation）。SetAttr 异步 apply，
    // 无 href 客户端缓存故同 execute 内 getAttribute 读 stale 快照——apply 后验 HTML 含新 href 属性。
    sandbox
        .execute("document.querySelector('#none').href = 'https://set.example.org/x';")
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let (out2, _h2) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms2).unwrap();
    assert!(
        out2.contains("<a id=\"none\" href=\"https://set.example.org/x\">"),
        "a.href setter 经 SetAttr mutation（apply 后 href 属性写入）\n{out2}"
    );
}

#[test]
fn test_form_reflected_idl_attrs_r2839() {
    // R2839：HTMLFormElement 反射 IDL 属性（action/method/enctype/target）——form 序列化 / AJAX 提交库
    // 读 form.action/form.method 构提交请求。action/target 纯串反射；method/enctype 小写归一 + spec 默认。
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
         <form id='f1' action='/submit' method='POST' enctype='multipart/form-data' target='_blank'></form>\
         <form id='f2' action='https://api.example.org/api' method='dialog'></form>\
         <form id='f3'></form>\
         <div id='notform' action='/x'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // f1：显式 action/method(POST→post 小写)/enctype/target 全反射。
    sandbox
        .execute(
            "var f = document.querySelector('#f1');\
             globalThis.__action = f.action;\
             globalThis.__method = f.method;\
             globalThis.__enctype = f.enctype;\
             globalThis.__target = f.target;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__action)").unwrap().value,
        "/submit",
        "form.action 反射（原始串）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__method)").unwrap().value,
        "post",
        "form.method POST→post 小写归一"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__enctype)").unwrap().value,
        "multipart/form-data",
        "form.enctype 反射"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__target)").unwrap().value,
        "_blank",
        "form.target 反射"
    );

    // f2：method=dialog（合法 enum 值）；action 绝对串反射。
    sandbox
        .execute(
            "var f2 = document.querySelector('#f2');\
             globalThis.__action2 = f2.action;\
             globalThis.__method2 = f2.method;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__action2)").unwrap().value,
        "https://api.example.org/api",
        "form.action 绝对串反射"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__method2)").unwrap().value,
        "dialog",
        "form.method=dialog 合法 enum"
    );

    // f3：无属性 → method 默认 'get'，enctype 默认 'application/x-www-form-urlencoded'，action/target 空。
    sandbox
        .execute(
            "var f3 = document.querySelector('#f3');\
             globalThis.__methodDef = f3.method;\
             globalThis.__enctypeDef = f3.enctype;\
             globalThis.__actionDef = f3.action;\
             globalThis.__targetDef = f3.target;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__methodDef)").unwrap().value,
        "get",
        "form.method 无属性→默认 'get'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__enctypeDef)").unwrap().value,
        "application/x-www-form-urlencoded",
        "form.enctype 无属性→默认"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__actionDef)").unwrap().value,
        "",
        "form.action 无属性→''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__targetDef)").unwrap().value,
        "",
        "form.target 无属性→''"
    );

    // 非 form 元素（div 带 action 属性）不返 form IDL（gate 仅 FORM）——div.action 非 form 默认行为。
    sandbox
        .execute("globalThis.__notformAction = String(document.querySelector('#notform').action);")
        .unwrap();
    // div.action 不应得 form 的默认 'get'-style 处理；接受 catch-all 任一返值（undefined/空/原始串）。
    let _nf = sandbox.execute("String(globalThis.__notformAction)").unwrap().value;
}

#[test]
fn test_reflected_idl_htmlfor_defaultvalue_r2840() {
    // R2840：反射属性 IDL——label.htmlFor（for 属性）、input.defaultValue（初始 value 属性，区别 .value
    // 当前态）、input.defaultChecked（checked 属性存在性）。form reset / 校验库读这些判「值/选中态是否改过」。
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
         <label id='l' for='nameInput'>Name</label>\
         <input id='nameInput' type='text' value='initial'>\
         <input id='chk' type='checkbox' checked>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // label.htmlFor 反射 for 属性。
    sandbox
        .execute("globalThis.__htmlFor = document.querySelector('#l').htmlFor;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__htmlFor)").unwrap().value,
        "nameInput",
        "label.htmlFor 反射 for 属性"
    );

    // input.defaultValue = 初始 value 属性（'initial'）；.value 当前态可独立改变，defaultValue 不变。
    sandbox
        .execute(
            "var i = document.querySelector('#nameInput');\
             globalThis.__dv0 = i.defaultValue;\
             globalThis.__val0 = i.value;\
             i.value = 'changed';\
             globalThis.__dv1 = i.defaultValue;\
             globalThis.__val1 = i.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dv0)").unwrap().value,
        "initial",
        "input.defaultValue=初始 value 属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__val0)").unwrap().value,
        "initial",
        "input.value 初始=defaultValue"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dv1)").unwrap().value,
        "initial",
        "改 .value 后 defaultValue 不变（区别当前态）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__val1)").unwrap().value,
        "changed",
        "改 .value 后 .value=changed"
    );

    // input.defaultChecked = checked 属性存在性（true）。.checked 当前态同（shim 无独立 toggle 态）。
    sandbox
        .execute(
            "var c = document.querySelector('#chk');\
             globalThis.__dc = c.defaultChecked;\
             globalThis.__ck = c.checked;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dc)").unwrap().value,
        "true",
        "input.defaultChecked=checked 属性存在"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ck)").unwrap().value,
        "true",
        "input.checked=true（同 defaultChecked）"
    );

    // setter：label.htmlFor = x 设 for 属性（attr 名映射）；input.defaultValue = x 设 value 属性。
    sandbox
        .execute(
            "document.querySelector('#l').htmlFor = 'emailInput';\
             document.querySelector('#nameInput').defaultValue = 'reset';",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<label id=\"l\" for=\"emailInput\">"),
        "label.htmlFor setter 设 for 属性（attr 名映射 htmlFor→for）\n{out}"
    );
    assert!(
        out.contains("value=\"reset\""),
        "input.defaultValue setter 设 value 属性（attr 名映射 defaultValue→value）\n{out}"
    );
}

#[test]
fn test_input_form_owner_r2841() {
    // R2841：.form（form-associated 控件）——返所属 <form> 元素。spec 顺序：① form 属性关联优先
    // （<input form="id"> → getElementById）；② 否则最近 ancestor <form>。校验/序列化库读 input.form 高频。
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
         <form id='fA'>\
           <input id='nested' type='text'>\
           <select id='sel'><option>x</option></select>\
         </form>\
         <input id='orphan' type='text'>\
         <form id='fB'></form>\
         <input id='attr' type='text' form='fB'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ancestor-based：nested input.form → fA（最近 ancestor form）。
    sandbox
        .execute(
            "globalThis.__nestedForm = document.querySelector('#nested').form;\
             globalThis.__nestedFormId = globalThis.__nestedForm ? globalThis.__nestedForm.id : null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__nestedFormId)").unwrap().value,
        "fA",
        "嵌套 input.form → ancestor form fA"
    );
    // 同 form proxy identity：input.form === document.querySelector('#fA')。
    assert_eq!(
        sandbox
            .execute("String(document.querySelector('#nested').form === document.querySelector('#fA'))")
            .unwrap()
            .value,
        "true",
        "input.form === ancestor form proxy（identity）"
    );

    // select.form 亦返 ancestor form（form-associated 控件 gate 含 SELECT）。
    sandbox
        .execute("globalThis.__selForm = document.querySelector('#sel').form.id;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__selForm)").unwrap().value,
        "fA",
        "select.form → ancestor form fA"
    );

    // orphan input（无 ancestor form）→ null。
    sandbox
        .execute("globalThis.__orphanForm = document.querySelector('#orphan').form;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__orphanForm)").unwrap().value,
        "null",
        "orphan input.form=null（无 ancestor form）"
    );

    // form 属性关联优先：<input form='fB'>（无 ancestor form）→ fB（getElementById）。
    sandbox
        .execute(
            "globalThis.__attrForm = document.querySelector('#attr').form;\
             globalThis.__attrFormId = globalThis.__attrForm ? globalThis.__attrForm.id : null;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__attrFormId)").unwrap().value,
        "fB",
        "input form='fB' → form 属性关联优先（getElementById fB）"
    );

    // 非 form 控件（如 div）的 .form 不走本 gate（返 undefined/其他，非 form owner 逻辑）。
    sandbox
        .execute("globalThis.__divForm = String(document.createElement('div').form);")
        .unwrap();
    // div.form 非 form owner 逻辑——接受 undefined（String(undefined)='undefined'）。
    let _df = sandbox.execute("String(globalThis.__divForm)").unwrap().value;
}

#[test]
fn test_table_row_cell_index_r2842() {
    // R2842：<tr>.rowIndex（行在 table 中位置，跨 thead/tbody/tfoot document order）+ <td>/<th>.cellIndex
    // （单元格在行中位置，td+th 混计）。data-table / 表格操作库读这些定位高频。client-side 经
    // _ancestorChain + 元素作用域 querySelectorAll + proxy identity 计位；无 owner → -1。
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
         <table id='t1'>\
           <thead><tr id='hr'><th>A</th><th>B</th></tr></thead>\
           <tbody>\
             <tr id='r0'><td id='c00'>1</td><td id='c01'>2</td></tr>\
             <tr id='r1'><td id='c10'>3</td><th id='h10'>4</th></tr>\
           </tbody>\
         </table>\
         <table id='t2'><tr id='r0b'><td>x</td></tr></table>\
         <tr id='orphan'><td>no-table</td></tr>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // rowIndex：跨 thead+tbody document order——hr=0, r0=1, r1=2。
    sandbox
        .execute(
            "globalThis.__hr = document.querySelector('#hr').rowIndex;\
             globalThis.__r0 = document.querySelector('#r0').rowIndex;\
             globalThis.__r1 = document.querySelector('#r1').rowIndex;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hr)").unwrap().value,
        "0",
        "thead 行 rowIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r0)").unwrap().value,
        "1",
        "tbody 首行 rowIndex=1（跨 thead 计）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r1)").unwrap().value,
        "2",
        "tbody 次行 rowIndex=2"
    );
    // 不同 table 的 r0b 在 t2 中 rowIndex=0（各 table 独立计）。
    sandbox
        .execute("globalThis.__r0b = document.querySelector('#r0b').rowIndex;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r0b)").unwrap().value,
        "0",
        "t2 中行 rowIndex=0（各 table 独立）"
    );
    // detached tr（createElement，未挂入 table）→ -1。注：HTML 解析器丢弃 table 外的 <tr>，
    // 故无法用 orphan tr 测；用 createElement('tr') detached 测 -1。
    sandbox
        .execute("globalThis.__detached = document.createElement('tr').rowIndex;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__detached)").unwrap().value,
        "-1",
        "detached tr（无 table）rowIndex=-1"
    );

    // cellIndex：行内 td+th document order——c00=0, c01=1, c10=0, h10=1（td+th 混计）。
    sandbox
        .execute(
            "globalThis.__c00 = document.querySelector('#c00').cellIndex;\
             globalThis.__c01 = document.querySelector('#c01').cellIndex;\
             globalThis.__c10 = document.querySelector('#c10').cellIndex;\
             globalThis.__h10 = document.querySelector('#h10').cellIndex;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__c00)").unwrap().value,
        "0",
        "td cellIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c01)").unwrap().value,
        "1",
        "td cellIndex=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c10)").unwrap().value,
        "0",
        "r1 首格 cellIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__h10)").unwrap().value,
        "1",
        "th cellIndex=1（td+th 混计 document order）"
    );
}

#[test]
fn test_table_section_index_and_collections_r2843() {
    // R2843：<tr>.sectionRowIndex（行在 thead/tbody/tfoot section 内位置）+ <table>.rows / <table>.tBodies
    //（table 内全部行 / tbody 集合，返真数组）。延续 R2842 表格表面。data-table 库迭代 table.rows 高频。
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
         <table id='t1'>\
           <thead><tr id='h1'><th>H1</th></tr><tr id='h2'><th>H2</th></tr></thead>\
           <tbody><tr id='b1'><td>B1</td></tr></tbody>\
           <tbody><tr id='b2'><td>B2</td></tr><tr id='b3'><td>B3</td></tr></tbody>\
         </table>\
         <table id='t2'><tbody><tr id='x1'><td>x</td></tr></tbody></table>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // sectionRowIndex：行在所属 section 内位置——thead 的 h1=0/h2=1；tbody1 的 b1=0；tbody2 的 b2=0/b3=1。
    sandbox
        .execute(
            "globalThis.__h1 = document.querySelector('#h1').sectionRowIndex;\
             globalThis.__h2 = document.querySelector('#h2').sectionRowIndex;\
             globalThis.__b1 = document.querySelector('#b1').sectionRowIndex;\
             globalThis.__b2 = document.querySelector('#b2').sectionRowIndex;\
             globalThis.__b3 = document.querySelector('#b3').sectionRowIndex;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__h1)").unwrap().value,
        "0",
        "thead h1 sectionRowIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__h2)").unwrap().value,
        "1",
        "thead h2 sectionRowIndex=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b1)").unwrap().value,
        "0",
        "tbody1 b1 sectionRowIndex=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b2)").unwrap().value,
        "0",
        "tbody2 b2 sectionRowIndex=0（新 section 重计）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b3)").unwrap().value,
        "1",
        "tbody2 b3 sectionRowIndex=1"
    );

    // table.rows：t1 全部行（跨 thead+2 tbody，document order，5 行）。
    sandbox
        .execute(
            "globalThis.__t1Rows = document.querySelector('#t1').rows.length;\
             globalThis.__t2Rows = document.querySelector('#t2').rows.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t1Rows)").unwrap().value,
        "5",
        "t1.rows.length=5（h1,h2,b1,b2,b3）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t2Rows)").unwrap().value,
        "1",
        "t2.rows.length=1（各 table 独立）"
    );

    // table.rows 真数组：可 Array.map 迭代 + 索引访问。
    sandbox
        .execute(
            "globalThis.__rowsMap = Array.prototype.map.call(document.querySelector('#t1').rows, function(r){return r.id;}).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rowsMap)").unwrap().value,
        "h1,h2,b1,b2,b3",
        "table.rows 可 Array.map 迭代（document order）"
    );

    // table.tBodies：t1 有 2 个 tbody。
    sandbox
        .execute("globalThis.__t1Bodies = document.querySelector('#t1').tBodies.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t1Bodies)").unwrap().value,
        "2",
        "t1.tBodies.length=2"
    );
}

#[test]
fn test_text_control_selection_r2844() {
    // R2844：text-control（input text-type / textarea）选区 IDL——selectionStart / selectionEnd /
    // selectionDirection getter + setSelectionRange + select + 属性 setter。Chromium 150 oracle 锚定：
    // 默认 {0, 0, 'forward'}（未聚焦 text control 选区折叠在 0，非值末）；select()→{0, len, forward}；
    // setSelectionRange clamp [0,len]，end<start 折叠到 end，direction 缺省 forward；属性 setter 保持 0≤start≤end≤len。
    // 文本编辑器 / 自动选择 / Range 算法读选区状态高频。number/checkbox 非选区 type → undefined（Chrome null）。
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
         <input id='i' type='text' value='world'>\
         <textarea id='ta'>hello</textarea>\
         <input id='num' type='number' value='42'>\
         <input id='chk' type='checkbox'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 默认选区 = {0, 0, 'forward'}（text control 未设/未聚焦）；非选区 type（number/checkbox）→ undefined。
    sandbox
        .execute(
            "var i = document.querySelector('#i');\
             var ta = document.querySelector('#ta');\
             globalThis.__d_ss = i.selectionStart;\
             globalThis.__d_se = i.selectionEnd;\
             globalThis.__d_dir = i.selectionDirection;\
             globalThis.__ta_ss = ta.selectionStart;\
             globalThis.__ta_se = ta.selectionEnd;\
             globalThis.__num = document.querySelector('#num').selectionStart;\
             globalThis.__chk = document.querySelector('#chk').selectionStart;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__d_ss)").unwrap().value,
        "0",
        "input 默认 selectionStart=0（折叠在 0，非值末）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__d_se)").unwrap().value,
        "0",
        "input 默认 selectionEnd=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__d_dir)").unwrap().value,
        "forward",
        "input 默认 selectionDirection='forward'"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__ta_ss) + ',' + String(globalThis.__ta_se)")
            .unwrap()
            .value,
        "0,0",
        "textarea 默认选区 {{0,0}}"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__num)").unwrap().value,
        "undefined",
        "number input 非选区 type → selectionStart undefined（Chrome null）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__chk)").unwrap().value,
        "undefined",
        "checkbox 非选区 type → selectionStart undefined"
    );

    // select() → {0, value.length, 'forward'}（input 5 / textarea 5）。
    sandbox
        .execute(
            "i.select();\
             globalThis.__sel_ss = i.selectionStart;\
             globalThis.__sel_se = i.selectionEnd;\
             globalThis.__sel_dir = i.selectionDirection;\
             ta.select();\
             globalThis.__ta_sel_se = ta.selectionEnd;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__sel_ss) + ',' + String(globalThis.__sel_se)")
            .unwrap()
            .value,
        "0,5",
        "input select() → {{0, 5}}（world 长度）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sel_dir)").unwrap().value,
        "forward",
        "input select() direction='forward'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ta_sel_se)").unwrap().value,
        "5",
        "textarea select() → selectionEnd=5（hello 长度）"
    );

    // setSelectionRange：正常 / end<start 折叠 / clamp 超界 / direction。
    // 注：Chrome 对**负数** start 的 setSelectionRange 有古怪归一（如 setSR(-5,-1)→{5,5}），属病态边角、
    // 无真实代码依赖；本实现按 spec 合理 clamp [0,len]，仅负数输入与 Chrome 古怪行为分歧（documented）。
    sandbox
        .execute(
            "i.setSelectionRange(1, 3, 'backward');\
             globalThis.__a = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.setSelectionRange(4, 2);\
             globalThis.__b = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.setSelectionRange(3, 9999);\
             globalThis.__c = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.setSelectionRange(0, 9999);\
             globalThis.__d = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__a)").unwrap().value,
        "1,3,backward",
        "setSelectionRange(1,3,'backward')"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__b)").unwrap().value,
        "2,2,forward",
        "setSelectionRange(4,2) end<start 折叠到 {{2,2}}，direction 缺省 forward"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__c)").unwrap().value,
        "3,5,forward",
        "setSelectionRange(3,9999) end clamp 到值长度 5"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__d)").unwrap().value,
        "0,5,forward",
        "setSelectionRange(0,9999) start=0 / end clamp 到 5"
    );

    // 属性 setter：start 超 end → end 跟升；end 低于 start → end 升回 start；direction 仅接受合法值。
    sandbox
        .execute(
            "i.setSelectionRange(1, 4);\
             i.selectionDirection = 'backward';\
             globalThis.__s1 = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.selectionStart = 99;\
             globalThis.__s2 = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;\
             i.selectionEnd = -5;\
             globalThis.__s3 = i.selectionStart + ',' + i.selectionEnd + ',' + i.selectionDirection;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__s1)").unwrap().value,
        "1,4,backward",
        "属性设 selectionDirection='backward'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s2)").unwrap().value,
        "5,5,backward",
        "selectionStart=99 → clamp 5，end 跟升到 5（保 0≤start≤end≤len）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s3)").unwrap().value,
        "5,5,backward",
        "selectionEnd=-5 → clamp 0 后升回 start=5（end 不低于 start）"
    );
}

#[test]
fn test_table_caption_thead_tfoot_section_rows_r2845() {
    // R2845：table.caption/tHead/tFoot（首个 caption/thead/tfoot 子元素或 null）+ section.rows（thead/tbody/tfoot
    // 作用域内行）。延续 R2843 表格表面。表格分析 / 序列化库读结构高频。Chromium 150 oracle 锚定。
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
         <table id='t1'>\
           <caption id='cap'>My Caption</caption>\
           <thead id='th'><tr id='h1'><th>H</th></tr></thead>\
           <tfoot id='tf'><tr id='f1'><td>F</td></tr></tfoot>\
           <tbody id='tb1'><tr id='b1'><td>B1</td></tr><tr id='b2'><td>B2</td></tr></tbody>\
         </table>\
         <table id='t2'><tbody><tr id='x1'><td>x</td></tr></tbody></table>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // table.caption：t1 首 caption（id=cap）；t2 无 → null。
    sandbox
        .execute(
            "globalThis.__cap = document.querySelector('#t1').caption ? document.querySelector('#t1').caption.id : 'null';\
             globalThis.__cap2 = String(document.querySelector('#t2').caption);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cap)").unwrap().value,
        "cap",
        "t1.caption 返首个 caption 元素（id=cap）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cap2)").unwrap().value,
        "null",
        "t2 无 caption → null"
    );

    // table.tHead / table.tFoot：t1 有 thead/tfoot（id）；t2 无 → null。
    sandbox
        .execute(
            "globalThis.__th = document.querySelector('#t1').tHead ? document.querySelector('#t1').tHead.id : 'null';\
             globalThis.__th2 = String(document.querySelector('#t2').tHead);\
             globalThis.__tf = document.querySelector('#t1').tFoot ? document.querySelector('#t1').tFoot.id : 'null';\
             globalThis.__tf2 = String(document.querySelector('#t2').tFoot);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__th)").unwrap().value,
        "th",
        "t1.tHead 返首个 thead（id=th）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__th2)").unwrap().value,
        "null",
        "t2 无 thead → null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tf)").unwrap().value,
        "tf",
        "t1.tFoot 返首个 tfoot（id=tf）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tf2)").unwrap().value,
        "null",
        "t2 无 tfoot → null"
    );

    // section.rows：tbody#tb1 作用域内行（b1,b2，2 行，section-scoped）；thead/tfoot 同。
    sandbox
        .execute(
            "globalThis.__tbRows = document.querySelector('#tb1').rows.length;\
             globalThis.__tbRowsMap = Array.prototype.map.call(document.querySelector('#tb1').rows, function(r){return r.id;}).join(',');\
             globalThis.__thRows = document.querySelector('#th').rows.length;\
             globalThis.__tfRows = document.querySelector('#tf').rows.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tbRows)").unwrap().value,
        "2",
        "tbody#tb1.rows.length=2（section-scoped b1,b2）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tbRowsMap)").unwrap().value,
        "b1,b2",
        "tbody.rows 迭代 document order（b1,b2）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__thRows)").unwrap().value,
        "1",
        "thead.rows.length=1（h1）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tfRows)").unwrap().value,
        "1",
        "tfoot.rows.length=1（f1）"
    );

    // table.rows 仍跨全 section（4 行：h1/f1/b1/b2），与 R2843 一致——rows gate 同时支持 TABLE 与 section。
    sandbox
        .execute("globalThis.__t1Rows = document.querySelector('#t1').rows.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t1Rows)").unwrap().value,
        "4",
        "table.rows.length=4（跨 thead/tfoot/tbody 全行，R2843 行为不变）"
    );
}

#[test]
fn test_output_value_default_value_r2846() {
    // R2846：HTMLOutputElement.value（getter=textContent，setter 同步 textContent）+ defaultValue（初始文本内容，
    // lazy 捕获一次，跨 value 变更保持稳定）。表单计算器 `<output>` 显示结果高频。Chromium 150 oracle 锚定。
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
         <output id='o1'>12</output>\
         <output id='o2'></output>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // value getter = textContent；defaultValue getter = 初始 textContent（同 value，未变更时相等）。
    // 每 execute 内声明局部元素 var（_proxyCache identity-stable）。
    sandbox
        .execute(
            "var a = document.querySelector('#o1');\
             globalThis.__v1 = a.value;\
             globalThis.__dv1 = a.defaultValue;\
             globalThis.__v2 = document.querySelector('#o2').value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__v1)").unwrap().value,
        "12",
        "o1.value=textContent '12'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dv1)").unwrap().value,
        "12",
        "o1.defaultValue=初始 textContent '12'"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__v2)").unwrap().value,
        "",
        "o2 空 output → value=''"
    );

    // value setter 仅更新 dirty 当前值（client 缓存即时）；spec：value 独立于 textContent——
    // 设 .value 不触碰 DOM text（<output> 按 children 渲染非 value），故 textContent 仍='12'。
    // defaultValue 不被 value 变更影响（捕获稳定）。每 execute 内声明局部元素 var（_proxyCache identity-stable）。
    sandbox
        .execute(
            "var o = document.querySelector('#o1');\
             o.value = 99;\
             globalThis.__v1b = o.value;\
             globalThis.__tc1 = o.textContent;\
             globalThis.__dv1b = o.defaultValue;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__v1b)").unwrap().value,
        "99",
        "o1.value=99 → value='99'（client 缓存即时）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tc1)").unwrap().value,
        "12",
        "o1.value setter 不触碰 textContent（仍='12'，spec value 独立于 text）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dv1b)").unwrap().value,
        "12",
        "defaultValue 跨 value 变更保持稳定（仍='12'）"
    );

    // defaultValue setter 更新捕获值；value（dirty）不受影响。
    sandbox
        .execute(
            "var d = document.querySelector('#o1');\
             d.defaultValue = 'dd';\
             globalThis.__dv1c = d.defaultValue;\
             globalThis.__v1c = d.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__dv1c)").unwrap().value,
        "dd",
        "defaultValue='dd' setter 更新捕获值"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__v1c)").unwrap().value,
        "99",
        "dirty 时设 defaultValue 不改 value（仍='99'）"
    );

    // value setter 不写 DOM text（spec：value 独立于 textContent）——apply 后 output 仍含初值 '12'，无 text mutation。
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("<output id=\"o1\">12</output>"),
        "output.value=99 不写 DOM text（apply 后 textContent 仍='12'，value 独立）\n{out}"
    );
}

#[test]
fn test_mutation_record_instanceof_spec_fields_r2847() {
    // R2847：MutationObserver 回调收到的 record 须 `instanceof MutationRecord` + `[object MutationRecord]`
    // toStringTag + 完整 spec 字段（previousSibling/nextSibling/attributeNamespace/oldValue 缺省 null，
    // addedNodes/removedNodes 缺省 []）。库做 instanceof 特征检测 / 读 record.previousSibling 须得 null 非 undefined。
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // execute 1：observe handle-based parent，appendChild child（childList mutation）。
    // 回调经 execute 末 microtask checkpoint 派发 → globalThis.__recs 在本 execute 末就绪。
    sandbox
        .execute(
            "var obs = new MutationObserver(function(records){ globalThis.__recs = records; });\
             var parent = document.createElement('div');\
             obs.observe(parent, { childList: true });\
             var child = document.createElement('span');\
             parent.appendChild(child);",
        )
        .unwrap();

    // execute 2：读捕获 record + 断言 instanceof / toStringTag / spec 字段缺省值。
    sandbox
        .execute(
            "var r = globalThis.__recs && globalThis.__recs[0];\
             globalThis.__len = globalThis.__recs ? globalThis.__recs.length : -1;\
             globalThis.__isMR = r instanceof MutationRecord;\
             globalThis.__tag = Object.prototype.toString.call(r);\
             globalThis.__type = r && r.type;\
             globalThis.__addedLen = r && r.addedNodes.length;\
             globalThis.__prevSib = r && r.previousSibling;\
             globalThis.__nextSib = r && r.nextSibling;\
             globalThis.__attrName = r && r.attributeName;\
             globalThis.__attrNs = r && r.attributeNamespace;\
             globalThis.__oldVal = r && r.oldValue;\
             globalThis.__removedLen = r && r.removedNodes.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__len)").unwrap().value,
        "1",
        "1 childList record（appendChild 触发）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__isMR)").unwrap().value,
        "true",
        "record instanceof MutationRecord（R2847）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tag)").unwrap().value,
        "[object MutationRecord]",
        "toStringTag = [object MutationRecord]"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__type)").unwrap().value,
        "childList",
        "type = childList"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__addedLen)").unwrap().value,
        "1",
        "addedNodes 含 1（span）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__prevSib)").unwrap().value,
        "null",
        "previousSibling 缺省 null（spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__nextSib)").unwrap().value,
        "null",
        "nextSibling 缺省 null（spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__attrName)").unwrap().value,
        "null",
        "attributeName 缺省 null（childList record，spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__attrNs)").unwrap().value,
        "null",
        "attributeNamespace 缺省 null（spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__oldVal)").unwrap().value,
        "null",
        "oldValue 缺省 null（spec）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__removedLen)").unwrap().value,
        "0",
        "removedNodes 缺省 []（spec，length 0）"
    );
}

#[test]
fn test_reflected_global_attrs_autofocus_draggable_spellcheck_translate_r2848() {
    // R2848：reflected 布尔/枚举全局属性 autofocus/draggable/spellcheck/translate——旧 fallthrough 返 undefined
    // （spec 须布尔）。spec 默认：autofocus=false / draggable=false / spellcheck=true / translate=true。
    // autofocus=boolean attr（presence）；draggable/spellcheck="true"/"false"；translate="yes"/"no"。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // autofocus 默认缺省 / draggable="true" attr / spellcheck="false" attr / translate="no" attr。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <input id='a' autofocus>\
         <div id='d' draggable='true'></div>\
         <div id='s' spellcheck='false'></div>\
         <div id='t' translate='no'></div>\
         <div id='plain'></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 读：autofocus(a, present)→true；draggable(d,"true")→true；spellcheck(s,"false")→false；
    // translate(t,"no")→false；plain 全缺省：autofocus=false / draggable=false / spellcheck=true / translate=true。
    sandbox
        .execute(
            "globalThis.__af = document.querySelector('#a').autofocus;\
             globalThis.__dg = document.querySelector('#d').draggable;\
             globalThis.__sc = document.querySelector('#s').spellcheck;\
             globalThis.__tr = document.querySelector('#t').translate;\
             var p = document.querySelector('#plain');\
             globalThis.__paf = p.autofocus;\
             globalThis.__pdg = p.draggable;\
             globalThis.__psc = p.spellcheck;\
             globalThis.__ptr = p.translate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__af)").unwrap().value,
        "true",
        "a[autofocus] present → autofocus=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__dg)").unwrap().value,
        "true",
        "div[draggable='true'] → draggable=true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sc)").unwrap().value,
        "false",
        "div[spellcheck='false'] → spellcheck=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tr)").unwrap().value,
        "false",
        "div[translate='no'] → translate=false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__paf)").unwrap().value,
        "false",
        "plain autofocus 缺省 → false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pdg)").unwrap().value,
        "false",
        "plain draggable 缺省 → false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__psc)").unwrap().value,
        "true",
        "plain spellcheck 缺省 → true（spec 默认）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ptr)").unwrap().value,
        "true",
        "plain translate 缺省 → true（spec 默认）"
    );

    // setter：同步 set→get 优先读缓存（即时）。autofocus=true 设 presence；draggable=true→attr "true"；
    // spellcheck=false→attr "false"；translate=true→attr "yes"。apply 后 attr 写回核验。
    sandbox
        .execute(
            "var e = document.querySelector('#plain');\
             e.autofocus = true; e.draggable = true; e.spellcheck = false; e.translate = true;\
             globalThis.__saf = e.autofocus;\
             globalThis.__sdg = e.draggable;\
             globalThis.__ssc = e.spellcheck;\
             globalThis.__str = e.translate;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__saf)").unwrap().value,
        "true",
        "setter autofocus=true → true（缓存即时）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__sdg)").unwrap().value,
        "true",
        "setter draggable=true → true"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ssc)").unwrap().value,
        "false",
        "setter spellcheck=false → false"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__str)").unwrap().value,
        "true",
        "setter translate=true → true"
    );

    // apply mutations → 核验 attr 写回（autofocus presence / draggable="true" / spellcheck="false" / translate="yes"）。
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&dom_html.lock().unwrap().clone(), &ms).unwrap();
    assert!(
        out.contains("id=\"plain\" autofocus"),
        "autofocus setter 写 presence\n{out}"
    );
    assert!(out.contains("draggable=\"true\""), "draggable setter 写 'true'\n{out}");
    assert!(
        out.contains("spellcheck=\"false\""),
        "spellcheck setter 写 'false'\n{out}"
    );
    assert!(out.contains("translate=\"yes\""), "translate setter 写 'yes'\n{out}");
}

#[test]
fn test_option_index_r2849() {
    // R2849：`<option>`.index（HTMLOptionElement）——option 在其 select 中的 0-based 位置（document order）；
    // 0 若不在 select（detached / handle-based，与 Chromium detached→0 一致）。form 库读 option.index 高频。
    // 同 R2842 rowIndex 模式：_ancestorChain 找 owning SELECT + 元素作用域 querySelectorAll('option') + identity。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 3 options in #s1；#s2 第一个 option 为 target（index 0）；含 optgroup（option 仍按 document order 计）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <select id='s1'>\
           <option id='a'>A</option>\
           <option id='b'>B</option>\
           <optgroup><option id='c'>C</option></optgroup>\
         </select>\
         <select id='s2'><option id='x'>X</option></select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 读 option.index：a=0 / b=1 / c=2（optgroup 内仍 document order）/ x=0（另一 select）。
    sandbox
        .execute(
            "globalThis.__ia = document.querySelector('#a').index;\
             globalThis.__ib = document.querySelector('#b').index;\
             globalThis.__ic = document.querySelector('#c').index;\
             globalThis.__ix = document.querySelector('#x').index;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ia)").unwrap().value,
        "0",
        "#a 为 s1 首个 option → index=0"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ib)").unwrap().value,
        "1",
        "#b 为 s1 第二个 option → index=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ic)").unwrap().value,
        "2",
        "#c 在 optgroup 内但 document order 仍 → index=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ix)").unwrap().value,
        "0",
        "#x 为 s2 首个 option → index=0（另一 select 作用域）"
    );

    // detached option（createElement，不在 select）→ 0（Chromium detached→0 一致）。
    sandbox
        .execute("globalThis.__d = document.createElement('option').index;")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__d)").unwrap().value,
        "0",
        "detached option（createElement，不在 select）→ index=0"
    );
}
