// js_dom_bridge_tests part11（R2962 tests 拆分后续片）——自 part10 溢出迁移的新增测试。
// 与 part01..10 同处 js_dom_bridge::tests 模块（经 include!），共享 super::* 导入。

#[test]
fn test_document_evaluate_xpath_r2981() {
    // R2981：document.evaluate / XPathResult——此前全缺（document.evaluate 与 XPathResult 零定义）→ 任何
    // XPath 查询抛 ReferenceError 中断脚本。补实用 XPath 1.0 子集：路径（//、/、相对 child）、节点测试
    //（tag/*/text()）、谓词（[n]/[last()]/[@a]/[@a='v']/[contains()]/[text()='v']/[position() op n]）、
    // 属性轴结果（@attr → 伪 Attr 节点）、多上下文 dedup、snapshot/iterateNext/singleNodeValue 表面。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // ul#list > li.item/active(A/B/C/D)；div#box > 2 p；3 a（含 class=ext 的 /c）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <ul id='list'>\
         <li class='item'>A</li>\
         <li class='item active'>B</li>\
         <li class='item'>C</li>\
         <li class='active'>D</li>\
         </ul>\
         <div id='box'><p>hello</p><p>world</p></div>\
         <a href='/a'>link-a</a>\
         <a href='/b'>link-b</a>\
         <a href='/c' class='ext'>link-c</a>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 表面存在性 + XPathResult 常量。
    assert_eq!(
        sandbox.execute("typeof document.evaluate").unwrap().value,
        "function",
        "document.evaluate 是 function"
    );
    assert_eq!(
        sandbox.execute("typeof XPathResult").unwrap().value,
        "object",
        "XPathResult 是 object"
    );
    assert_eq!(
        sandbox
            .execute("String(XPathResult.ORDERED_NODE_SNAPSHOT_TYPE)")
            .unwrap()
            .value,
        "7",
        "XPathResult.ORDERED_NODE_SNAPSHOT_TYPE = 7"
    );

    // 辅助：求值 → 快照结果。
    sandbox
        .execute(
            "globalThis.__xp = function(expr, ctx, type) {\
             return document.evaluate(expr, ctx || document, null,\
             (type == null ? XPathResult.ORDERED_NODE_SNAPSHOT_TYPE : type), null);\
             };",
        )
        .unwrap();

    // //li：descendant 全部 li = 4，resultType=7（ORDERED_NODE_SNAPSHOT）。
    assert_eq!(
        sandbox
            .execute("String(__xp('//li').resultType)")
            .unwrap()
            .value,
        "7",
        "//li resultType = ORDERED_NODE_SNAPSHOT_TYPE(7)"
    );
    assert_eq!(
        sandbox.execute("__xp('//li').snapshotLength").unwrap().value,
        "4",
        "//li snapshotLength = 4"
    );

    // //li[1]：descendant 候选集首位 = A。
    assert_eq!(
        sandbox
            .execute("__xp('//li[1]').snapshotItem(0).textContent")
            .unwrap()
            .value,
        "A",
        "//li[1] = A（descendant 候选集首位）"
    );

    // //li[last()]：末位 = D。
    assert_eq!(
        sandbox
            .execute("__xp('//li[last()]').snapshotItem(0).textContent")
            .unwrap()
            .value,
        "D",
        "//li[last()] = D"
    );

    // //li[@class='item']：精确 class=item → A、C（B 为 'item active'，D 为 'active'）= 2。
    assert_eq!(
        sandbox
            .execute("__xp(\"//li[@class='item']\").snapshotLength")
            .unwrap()
            .value,
        "2",
        "//li[@class='item'] = 2（A/C，精确匹配）"
    );

    // //li[contains(@class,'active')] → B、D = 2。
    assert_eq!(
        sandbox
            .execute("__xp(\"//li[contains(@class,'active')]\").snapshotLength")
            .unwrap()
            .value,
        "2",
        "//li[contains(@class,'active')] = 2（B/D）"
    );

    // 谓词链：//li[@class='item'][2] → class=item（A/C）中第 2 = C。
    assert_eq!(
        sandbox
            .execute("__xp(\"//li[@class='item'][2]\").snapshotItem(0).textContent")
            .unwrap()
            .value,
        "C",
        "//li[@class='item'][2] = C（谓词链顺序应用）"
    );

    // position() op：//li[position()>=3] → C、D = 2。
    assert_eq!(
        sandbox
            .execute("__xp('//li[position()>=3]').snapshotLength")
            .unwrap()
            .value,
        "2",
        "//li[position()>=3] = 2（C/D）"
    );

    // child 轴 per-parent 位置：//ul/li[2] → ul 的第 2 个 li = B。
    assert_eq!(
        sandbox
            .execute("__xp('//ul/li[2]').singleNodeValue.textContent")
            .unwrap()
            .value,
        "B",
        "//ul/li[2] = B（child 轴 per-parent 位置）"
    );

    // FIRST_ORDERED_NODE_TYPE(9) singleNodeValue。
    assert_eq!(
        sandbox
            .execute(
                "String(__xp('//ul/li[2]', null, XPathResult.FIRST_ORDERED_NODE_TYPE).resultType)"
            )
            .unwrap()
            .value,
        "9",
        "FIRST_ORDERED_NODE_TYPE resultType = 9"
    );

    // last()-n：//ul/li[last()-1] → 第 3 个 li = C。
    assert_eq!(
        sandbox
            .execute("__xp('//ul/li[last()-1]').singleNodeValue.textContent")
            .unwrap()
            .value,
        "C",
        "//ul/li[last()-1] = C"
    );

    // text() 谓词：//a[text()='link-b'] → link-b = 1。
    assert_eq!(
        sandbox
            .execute("__xp(\"//a[text()='link-b']\").snapshotLength")
            .unwrap()
            .value,
        "1",
        "//a[text()='link-b'] = 1"
    );
    // contains(@attr,'sub')：//a[contains(@href,'b')] → /b = 1。
    assert_eq!(
        sandbox
            .execute("__xp(\"//a[contains(@href,'b')]\").snapshotLength")
            .unwrap()
            .value,
        "1",
        "//a[contains(@href,'b')] = 1"
    );

    // 属性轴结果：//a/@href → 3 个伪 Attr 节点，[0].value='/a'。
    assert_eq!(
        sandbox
            .execute("__xp('//a/@href').snapshotLength")
            .unwrap()
            .value,
        "3",
        "//a/@href = 3（属性轴伪 Attr 节点）"
    );
    assert_eq!(
        sandbox
            .execute("__xp('//a/@href').snapshotItem(0).value")
            .unwrap()
            .value,
        "/a",
        "//a/@href[0].value = '/a'"
    );

    // 相对路径（child 轴 from context）：#box 子 p = 2。
    assert_eq!(
        sandbox
            .execute(
                "globalThis.__box = document.querySelector('#box');\
                 __xp('p', __box).snapshotLength"
            )
            .unwrap()
            .value,
        "2",
        "相对 'p' from #box = 2（child 轴）"
    );

    // iterateNext() 游标：//a 迭代 3 次后返 null。
    assert_eq!(
        sandbox
            .execute(
                "globalThis.__it = __xp('//a', null, XPathResult.ORDERED_NODE_ITERATOR_TYPE);\
                 globalThis.__cnt = 0;\
                 while (__it.iterateNext()) globalThis.__cnt++;\
                 String(__it.iterateNext());"
            )
            .unwrap()
            .value,
        "null",
        "iterateNext() 迭代完返 null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cnt)").unwrap().value,
        "3",
        "//a iterateNext 计数 = 3"
    );

    // 无匹配：singleNodeValue=null，snapshotLength=0。
    assert_eq!(
        sandbox
            .execute("String(__xp('//nonexistent').singleNodeValue === null)")
            .unwrap()
            .value,
        "true",
        "//nonexistent singleNodeValue = null"
    );
    assert_eq!(
        sandbox
            .execute("__xp('//nonexistent').snapshotLength")
            .unwrap()
            .value,
        "0",
        "//nonexistent snapshotLength = 0"
    );

    // 无效表达式抛 Error（spec INVALID_EXPRESSION_ERR 语义）。
    sandbox
        .execute(
            "globalThis.__threw = 'no';\
             try { document.evaluate('', document, null, 6, null); }\
             catch (e) { globalThis.__threw = (e instanceof TypeError || e instanceof Error) ? 'yes' : 'other'; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__threw)").unwrap().value,
        "yes",
        "空 XPath 表达式抛 Error（honest failure）"
    );
}

#[test]
fn test_request_body_readers_r2982() {
    // R2982：Request body 消费表面（对称 Response R2978，spec text/json/blob/arrayBuffer/formData）。
    // 此前 Request 仅有 body 字段（string|null）+ clone()，无 readers——fetch 包装库 / service worker
    // fetch handler / 请求拦截器 / 测试 mock 读 `request.text()/json()/formData()` 抛 TypeError。本切片补全，
    // 并抽出 _zwParseFormUrlencoded（Response.formData + Request.formData 共用，去重）。
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 表面存在性：5 reader 均为 function。
    sandbox
        .execute(
            "globalThis.__req = new Request('/api', { method: 'POST', body: 'hello world' });\
             globalThis.__types = ['text','json','blob','arrayBuffer','formData']\
               .map(function(m){ return typeof __req[m]; }).join(',');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__types)").unwrap().value,
        "function,function,function,function,function",
        "Request text/json/blob/arrayBuffer/formData 均为 function"
    );

    // text()：POST body 还原。
    sandbox
        .execute("globalThis.__rt = '(pending)'; __req.text().then(function(t){ globalThis.__rt = t; });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rt)").unwrap().value,
        "hello world",
        "Request.text() 还原 POST body"
    );

    // json()：合法 JSON body 解析。
    sandbox
        .execute(
            "globalThis.__rj = null;\
             new Request('/api', { method:'POST', body: '{\"a\":1,\"b\":\"two\"}' })\
               .json().then(function(o){ globalThis.__rj = o; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rj && globalThis.__rj.b)").unwrap().value,
        "two",
        "Request.json() 解析 body 对象"
    );

    // blob()：instanceof Blob + text() 还原。
    sandbox
        .execute(
            "globalThis.__rbIsBlob = false; globalThis.__rbText = '';\
             new Request('/api', { method:'POST', body: 'X' })\
               .blob().then(function(b){ globalThis.__rbIsBlob = (b instanceof Blob); return b.text(); })\
               .then(function(t){ globalThis.__rbText = t; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rbIsBlob)").unwrap().value,
        "true",
        "Request.blob() → instanceof Blob"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rbText)").unwrap().value,
        "X",
        "Request.blob().text() 还原 body"
    );

    // arrayBuffer()：Uint8Array + byteLength + 索引（'AB' → [65,66]）。
    sandbox
        .execute(
            "globalThis.__abIsU8 = false; globalThis.__abLen = -1; globalThis.__ab0 = -1;\
             new Request('/api', { method:'POST', body:'AB' })\
               .arrayBuffer().then(function(buf){\
                 globalThis.__abIsU8 = (buf instanceof Uint8Array);\
                 globalThis.__abLen = buf.length;\
                 globalThis.__ab0 = buf[0];\
               });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__abIsU8)").unwrap().value,
        "true",
        "Request.arrayBuffer() → Uint8Array"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__abLen)").unwrap().value,
        "2",
        "Request.arrayBuffer('AB') length=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ab0)").unwrap().value,
        "65",
        "Request.arrayBuffer('AB')[0]=65 ('A')"
    );

    // formData()：urlencoded 解析（+ → space，% 解码）。
    sandbox
        .execute(
            "globalThis.__fdA = null; globalThis.__fdC = null;\
             new Request('/api', { method:'POST', body:'a=1&c=hello+world' })\
               .formData().then(function(fd){ globalThis.__fdA = fd.get('a'); globalThis.__fdC = fd.get('c'); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fdA)").unwrap().value,
        "1",
        "Request.formData() 解析 a=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__fdC)").unwrap().value,
        "hello world",
        "Request.formData() c=hello+world → 'hello world'（+ → space）"
    );

    // 无体 GET 请求：text() 返 ''，arrayBuffer() 长度 0。
    sandbox
        .execute(
            "globalThis.__et = '(pending)'; globalThis.__eab = -1;\
             var get = new Request('/api');\
             get.text().then(function(t){ globalThis.__et = t; });\
             get.arrayBuffer().then(function(b){ globalThis.__eab = b.length; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__et)").unwrap().value,
        "",
        "GET（无 body）Request.text() 返 ''"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__eab)").unwrap().value,
        "0",
        "GET（无 body）Request.arrayBuffer() length=0"
    );

    // clone() 保留 body，clone 的 reader 独立读。
    sandbox
        .execute(
            "globalThis.__ct = '(pending)';\
             var orig = new Request('/api', { method:'POST', body:'cloned' });\
             orig.clone().text().then(function(t){ globalThis.__ct = t; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ct)").unwrap().value,
        "cloned",
        "Request.clone() 保留 body（reader 读到克隆体）"
    );

    // 对称性：Response.formData 经抽出 helper 仍正确（回归守卫——_zwParseFormUrlencoded 提取后 Response 不退化）。
    sandbox
        .execute(
            "globalThis.__rfd = null;\
             new Response('x=42').formData().then(function(fd){ globalThis.__rfd = fd.get('x'); });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rfd)").unwrap().value,
        "42",
        "Response.formData() 经 _zwParseFormUrlencoded 提取后仍正确（无回归）"
    );
}

#[test]
fn test_window_post_message_r2983() {
    // R2983：window.postMessage(message, targetOrigin [, transfer])——canonical 跨窗口消息 API。
    // 此前缺（MessagePort/MessageChannel/BroadcastChannel 既有，但 window.postMessage 零定义）→
    // `window.postMessage({...}, '*')` + `addEventListener('message')` 同窗口异步消息模式抛 TypeError。
    // 本切片补：structuredClone 深拷贝 payload + queueMicrotask 异步派发 MessageEvent 到自身（触发
    // window 'message' listener + onmessage），targetOrigin 安全校验（不匹配同步 throw SecurityError）。
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
    // http://test.local/ → location.origin = 'http://test.local'（targetOrigin 校验用）。
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    assert_eq!(
        sandbox.execute("typeof window.postMessage").unwrap().value,
        "function",
        "window.postMessage 是 function"
    );

    // addEventListener('message') + postMessage({a:1,b:'two'}, '*')：异步收到 cloned data。
    // microtask 在 execute 末 drain → handler 设 globalThis.__got。
    sandbox
        .execute(
            "globalThis.__got = '(none)';\
             window.addEventListener('message', function (e) {\
               globalThis.__got = JSON.stringify(e.data) + '|' + e.origin + '|' + (e.source === window);\
             });\
             window.postMessage({ a: 1, b: 'two' }, '*');",
        )
        .unwrap();
    let got = sandbox.execute("String(globalThis.__got)").unwrap().value;
    assert!(
        got.starts_with("{\"a\":1,\"b\":\"two\"}|"),
        "addEventListener('message') 收到 cloned data（got={}）",
        got
    );
    assert!(
        got.ends_with("|true"),
        "event.source === window（headless 单窗口，got={}）",
        got
    );

    // onmessage IDL handler 亦触发（postMessage 派发经 dispatchEvent → onmessage listener）。
    sandbox
        .execute(
            "globalThis.__onm = '(none)';\
             window.onmessage = function (e) { globalThis.__onm = String(e.data); };\
             window.postMessage('hello-onmessage', '*');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__onm)").unwrap().value,
        "hello-onmessage",
        "window.onmessage handler 收到 message 事件"
    );

    // structuredClone 隔离：收到的是深拷贝，原对象不被 mutate。
    sandbox
        .execute(
            "globalThis.__orig = { nested: { v: 1 } }; globalThis.__iso = '(none)';\
             window.onmessage = function (e) { e.data.nested.v = 999; globalThis.__iso = String(e.data.nested.v); };\
             window.postMessage(globalThis.__orig, '*');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__iso)").unwrap().value,
        "999",
        "收到 payload 可 mutate（深拷贝，非只读）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__orig.nested.v)")
            .unwrap()
            .value,
        "1",
        "structuredClone 隔离：原对象未被 mutate（仍 v=1）"
    );

    // targetOrigin 安全校验：'*' / '/' / 当前 origin 放行。
    sandbox
        .execute(
            "globalThis.__cnt = 0;\
             window.onmessage = function () { globalThis.__cnt++; };\
             window.postMessage('a', '*');\
             window.postMessage('b', 'http://test.local');\
             window.postMessage('c', '/');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cnt)").unwrap().value,
        "3",
        "targetOrigin '*' / 当前 origin / '/' 均放行（3 次派发）"
    );

    // targetOrigin 不匹配 → 同步 throw SecurityError。
    sandbox
        .execute(
            "globalThis.__threw = 'no'; globalThis.__errName = '';\
             try { window.postMessage('x', 'http://evil.example'); }\
             catch (e) { globalThis.__threw = 'yes'; globalThis.__errName = (e && e.name) || ''; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__threw)").unwrap().value,
        "yes",
        "targetOrigin 不匹配 → throw"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__errName)").unwrap().value,
        "SecurityError",
        "targetOrigin 不匹配 → SecurityError"
    );
}

#[test]
fn test_submit_event_submitter_r2984() {
    // R2984：SubmitEvent + event.submitter。此前 submit 事件经 __zw_dispatch_event(form,'submit',null)
    // 派发为 generic Event（无 submitter）——表单多 submit 按钮场景（"保存"/"删除"同 form）读
    // event.submitter 判激活按钮获 undefined。本切片：DomEventDetail 增 submitter 字段 + shim 新 SubmitEvent
    //（extends Event + .submitter）+ __zw_dispatch_event 在 type==='submit' 时造 SubmitEvent；
    // renderer submit_enclosing_form click 路径传被点按钮 selector，Enter 隐式提交传 None。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // form#f 含 txt input + 两个 submit 按钮（save/del）——典型多 submit 按钮 form。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><form id='f'>\
         <input id='txt' type='text'>\
         <button id='save' type='submit'>Save</button>\
         <button id='del' type='submit'>Del</button>\
         </form></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    assert_eq!(
        sandbox.execute("typeof SubmitEvent").unwrap().value,
        "function",
        "SubmitEvent 是 function（构造器就位）"
    );

    // 注册 submit listener（捕获 event 类型 / instanceof / submitter.id）。
    sandbox
        .execute(
            "globalThis.__evType = '(none)'; globalThis.__isSubmit = '(none)'; globalThis.__subId = '(none)';\
             document.querySelector('#f').addEventListener('submit', function (e) {\
               globalThis.__evType = e.type;\
               globalThis.__isSubmit = String(e instanceof SubmitEvent);\
               globalThis.__subId = (e.submitter && e.submitter.id) || 'null';\
             });",
        )
        .unwrap();

    // click #save submit button → submit 事件 + event.submitter = #save（__zw_dispatch_event 同步派发）。
    sandbox
        .execute("__zw_dispatch_event('#f', 'submit', { submitter: '#save' });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__evType)").unwrap().value,
        "submit",
        "submit 事件类型 = submit"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__isSubmit)").unwrap().value,
        "true",
        "事件 instanceof SubmitEvent"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__subId)").unwrap().value,
        "save",
        "click #save → event.submitter.id = 'save'"
    );

    // click #del submit button → submitter = #del（多 submit 按钮区分激活按钮）。
    sandbox
        .execute("__zw_dispatch_event('#f', 'submit', { submitter: '#del' });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__subId)").unwrap().value,
        "del",
        "click #del → event.submitter.id = 'del'（多 submit 按钮区分）"
    );

    // Enter 隐式提交（无 submitter）→ submitter = null（仍 instanceof SubmitEvent）。
    sandbox
        .execute("globalThis.__isSubmit2 = '(none)';\
             __zw_dispatch_event('#f', 'submit', null);")
        .unwrap();
    // 末次 listener 触发：handler 末行设 __subId，但 isSubmit2 需在 handler 内取。改用单 handler 已设 __isSubmit。
    assert_eq!(
        sandbox.execute("String(globalThis.__subId)").unwrap().value,
        "null",
        "Enter 隐式提交（无 submitter）→ event.submitter = null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__isSubmit)").unwrap().value,
        "true",
        "无 submitter 的 submit 仍 instanceof SubmitEvent"
    );

    // preventDefault：cancelable submit listener 返 false → __zw_dispatch_event 返 'prevented'。
    sandbox
        .execute(
            "document.querySelector('#f').addEventListener('submit', function (e) { e.preventDefault(); });\
             globalThis.__r = __zw_dispatch_event('#f', 'submit', { submitter: '#save' });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r)").unwrap().value,
        "prevented",
        "submit listener preventDefault → __zw_dispatch_event 返 'prevented'"
    );
}

#[test]
fn test_canvas_get_transform_dommatrix_r2985() {
    // R2985：Canvas getTransform/resetTransform + DOMMatrix/DOMPoint。此前 shim Canvas 仅有 setTransform/
    // transform，**无 getTransform（返 undefined）/ resetTransform**——读当前矩阵（hit-testing / transform-aware
    // 绘制 / save-restore 矩阵快照）失效；DOMMatrix/DOMPoint 几何类型亦全缺。本切片补全（Canvas 2D 为 Tier 1）。
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
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 表面存在性：DOMMatrix / DOMPoint 构造器 + ctx.getTransform/resetTransform。
    assert_eq!(
        sandbox.execute("typeof DOMMatrix").unwrap().value,
        "function",
        "DOMMatrix 是 function"
    );
    assert_eq!(
        sandbox.execute("typeof DOMPoint").unwrap().value,
        "function",
        "DOMPoint 是 function"
    );
    sandbox
        .execute(
            "globalThis.__cx = document.createElement('canvas').getContext('2d');\
             globalThis.__gt = typeof __cx.getTransform;\
             globalThis.__rt = typeof __cx.resetTransform;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__gt)").unwrap().value,
        "function",
        "ctx.getTransform 是 function"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rt)").unwrap().value,
        "function",
        "ctx.resetTransform 是 function"
    );

    // getTransform：setTransform(2,0,0,3,10,20) 后读回 DOMMatrix，a/b/c/d/e/f 精确。
    sandbox
        .execute(
            "__cx.setTransform(2, 0, 0, 3, 10, 20);\
             globalThis.__m = __cx.getTransform();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__m instanceof DOMMatrix)")
            .unwrap()
            .value,
        "true",
        "getTransform 返 instanceof DOMMatrix"
    );
    assert_eq!(sandbox.execute("String(globalThis.__m.a)").unwrap().value, "2", "m.a=2");
    assert_eq!(sandbox.execute("String(globalThis.__m.b)").unwrap().value, "0", "m.b=0");
    assert_eq!(sandbox.execute("String(globalThis.__m.c)").unwrap().value, "0", "m.c=0");
    assert_eq!(sandbox.execute("String(globalThis.__m.d)").unwrap().value, "3", "m.d=3");
    assert_eq!(
        sandbox.execute("String(globalThis.__m.e)").unwrap().value,
        "10",
        "m.e=10"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__m.f)").unwrap().value,
        "20",
        "m.f=20"
    );
    // m11/m22/m41/m42 别名（4×4 行主序）：m11=a=2, m22=d=3, m41=e=10, m42=f=20。
    assert_eq!(
        sandbox.execute("String(globalThis.__m.m11)").unwrap().value,
        "2",
        "m.m11=a=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__m.m41)").unwrap().value,
        "10",
        "m.m41=e=10"
    );

    // resetTransform → identity（a=1, d=1, e=0, f=0）。
    sandbox
        .execute(
            "__cx.resetTransform();\
             globalThis.__m2 = __cx.getTransform();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__m2.a)").unwrap().value,
        "1",
        "resetTransform 后 m.a=1（identity）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__m2.d)").unwrap().value,
        "1",
        "resetTransform 后 m.d=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__m2.e)").unwrap().value,
        "0",
        "resetTransform 后 m.e=0"
    );

    // DOMMatrix 构造：无参 = identity。
    sandbox.execute("globalThis.__id = new DOMMatrix();").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__id.a)").unwrap().value,
        "1",
        "new DOMMatrix() identity a=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__id.e)").unwrap().value,
        "0",
        "new DOMMatrix() identity e=0"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__id.toFloat32Array().length)")
            .unwrap()
            .value,
        "16",
        "DOMMatrix.toFloat32Array().length = 16"
    );

    // DOMMatrix from [6] 2D 数组。
    sandbox
        .execute("globalThis.__m6 = new DOMMatrix([1, 2, 3, 4, 5, 6]);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__m6.a)").unwrap().value,
        "1",
        "DOMMatrix([1,2,3,4,5,6]) a=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__m6.b)").unwrap().value,
        "2",
        "DOMMatrix([1,2,3,4,5,6]) b=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__m6.e)").unwrap().value,
        "5",
        "DOMMatrix([1,2,3,4,5,6]) e=5"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__m6.f)").unwrap().value,
        "6",
        "DOMMatrix([1,2,3,4,5,6]) f=6"
    );

    // translate：identity.translate(5,10) → e=5, f=10。
    sandbox
        .execute("globalThis.__t = new DOMMatrix().translate(5, 10);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__t.e)").unwrap().value,
        "5",
        "translate(5,10) e=5"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__t.f)").unwrap().value,
        "10",
        "translate(5,10) f=10"
    );

    // scale：identity.scale(2,3) → a=2, d=3。
    sandbox
        .execute("globalThis.__s = new DOMMatrix().scale(2, 3);")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__s.a)").unwrap().value,
        "2",
        "scale(2,3) a=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__s.d)").unwrap().value,
        "3",
        "scale(2,3) d=3"
    );

    // transformPoint：translate(5,10) 变换 (2,3) → (7,13)，返 DOMPoint。
    sandbox
        .execute(
            "globalThis.__pt = new DOMMatrix([1,0,0,1,5,10]).transformPoint({ x: 2, y: 3 });",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__pt instanceof DOMPoint)")
            .unwrap()
            .value,
        "true",
        "transformPoint 返 instanceof DOMPoint"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pt.x)").unwrap().value,
        "7",
        "transformPoint(2,3) with translate(5,10) → x=7"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__pt.y)").unwrap().value,
        "13",
        "transformPoint(2,3) with translate(5,10) → y=13"
    );

    // DOMPoint 构造 + fromPoint。
    sandbox.execute("globalThis.__p = new DOMPoint(1, 2, 3, 1);").unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__p.x)").unwrap().value,
        "1",
        "new DOMPoint(1,2,3,1) x=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__p.y)").unwrap().value,
        "2",
        "new DOMPoint(1,2,3,1) y=2"
    );
    assert_eq!(
        sandbox
            .execute("String(DOMPoint.fromPoint(globalThis.__p).z)")
            .unwrap()
            .value,
        "3",
        "DOMPoint.fromPoint 复制 z=3"
    );
}
