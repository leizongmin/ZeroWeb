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

#[test]
fn test_compression_stream_r2986() {
    // R2986：CompressionStream/DecompressionStream（gzip/deflate/deflate-raw）。Compression Streams API
    // 此前全缺——`response.body.pipeThrough(new DecompressionStream('gzip'))` 解压服务端 gzip 流 / 压缩上传
    // 载荷不可用。本切片经 flate2（既有 workspace crate）补 gzip/deflate/deflate-raw，buffer-then-process
    //（transform 累积 chunk，flush 整体压缩/解压），完成 Streams API 转换流表面。
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

    assert_eq!(
        sandbox.execute("typeof CompressionStream").unwrap().value,
        "function",
        "CompressionStream 是 function"
    );
    assert_eq!(
        sandbox.execute("typeof DecompressionStream").unwrap().value,
        "function",
        "DecompressionStream 是 function"
    );
    assert_eq!(
        sandbox
            .execute("String(new CompressionStream('gzip') instanceof TransformStream)")
            .unwrap()
            .value,
        "true",
        "CompressionStream instanceof TransformStream"
    );

    // gzip 往返：源字节流 → CompressionStream('gzip') → DecompressionStream('gzip') → 还原文本。
    // 经 pipeThrough 串联两个转换流；ReadableStream pipeTo Writable（transform 的 writable）异步 drain。
    sandbox
        .execute(
            "globalThis.__out = '(none)';\
             var src = new ReadableStream({\
               start: function (c) {\
                 c.enqueue(new TextEncoder().encode('hello gzip compression round trip payload repeat repeat'));\
                 c.close();\
               }\
             });\
             var comp = new CompressionStream('gzip');\
             var decomp = new DecompressionStream('gzip');\
             // src → comp.writable（压缩），comp.readable → decomp.writable（解压），decomp.readable 读取。
             src.pipeTo(comp.writable);\
             comp.readable.pipeTo(decomp.writable);\
             var reader = decomp.readable.getReader();\
             reader.read().then(function (c) {\
               if (c.done) { globalThis.__out = '(empty)'; return; }\
               globalThis.__out = new TextDecoder().decode(c.value);\
               return reader.read();\
             });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__out)").unwrap().value,
        "hello gzip compression round trip payload repeat repeat",
        "gzip 压缩→解压往返还原原文"
    );

    // deflate 往返（zlib 包装）。
    sandbox
        .execute(
            "globalThis.__out2 = '(none)';\
             var src2 = new ReadableStream({\
               start: function (c) {\
                 c.enqueue(new TextEncoder().encode('deflate zlib wrapped payload hello hello'));\
                 c.close();\
               }\
             });\
             var c2 = new CompressionStream('deflate');\
             var d2 = new DecompressionStream('deflate');\
             src2.pipeTo(c2.writable);\
             c2.readable.pipeTo(d2.writable);\
             var r2 = d2.readable.getReader();\
             r2.read().then(function (c) {\
               if (c.done) { globalThis.__out2 = '(empty)'; return; }\
               globalThis.__out2 = new TextDecoder().decode(c.value);\
             });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__out2)").unwrap().value,
        "deflate zlib wrapped payload hello hello",
        "deflate 压缩→解压往返还原原文"
    );

    // deflate-raw 往返（裸 deflate，无 zlib 头）。
    sandbox
        .execute(
            "globalThis.__out3 = '(none)';\
             var src3 = new ReadableStream({\
               start: function (c) {\
                 c.enqueue(new TextEncoder().encode('raw deflate payload no wrapper data'));\
                 c.close();\
               }\
             });\
             var c3 = new CompressionStream('deflate-raw');\
             var d3 = new DecompressionStream('deflate-raw');\
             src3.pipeTo(c3.writable);\
             c3.readable.pipeTo(d3.writable);\
             var r3 = d3.readable.getReader();\
             r3.read().then(function (c) {\
               if (c.done) { globalThis.__out3 = '(empty)'; return; }\
               globalThis.__out3 = new TextDecoder().decode(c.value);\
             });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__out3)").unwrap().value,
        "raw deflate payload no wrapper data",
        "deflate-raw 压缩→解压往返还原原文"
    );

    // 不支持 format → 构造抛 DOMException NotSupportedError。
    sandbox
        .execute(
            "globalThis.__threw = 'no'; globalThis.__errName = '';\
             try { new CompressionStream('brotli'); }\
             catch (e) { globalThis.__threw = 'yes'; globalThis.__errName = (e && e.name) || ''; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__threw)").unwrap().value,
        "yes",
        "不支持 format（brotli）→ 构造抛"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__errName)").unwrap().value,
        "NotSupportedError",
        "不支持 format → NotSupportedError"
    );
}

#[test]
fn test_decompression_stream_error_r2991() {
    // R2991：DecompressionStream 损坏输入端到端错误契约（R2986 driving test 仅覆盖 happy-path 往返，
    // corrupt 路径未测）。DecompressionStream 消费任意不可信字节（服务端响应 / 上传载荷），corrupt 输入是
    // 常态：host decompress_bytes 对非法字节返空串（flate2 Err，已由 compress::tests 锁定不 panic），
    // shim flush 见「输入非空但输出空」→ controller.error(DataError) → readable 出错 → reader.read() reject。
    // 本测试验证整条链路：reader.read() 以 DataError reject（不静默吞 / 不挂起），且错误经 reader 可观察。
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

    // 垃圾字节（无合法 gzip magic）→ DecompressionStream('gzip') → reader.read() reject DataError。
    sandbox
        .execute(
            "globalThis.__res = '(none)'; globalThis.__errName = '';\
             var garbage = new Uint8Array([0, 1, 2, 3, 4, 0xfe, 0xfd, 0xfc]);\
             var src = new ReadableStream({\
               start: function (c) { c.enqueue(garbage); c.close(); }\
             });\
             var ds = new DecompressionStream('gzip');\
             src.pipeTo(ds.writable);\
             var reader = ds.readable.getReader();\
             reader.read().then(\
               function (c) { globalThis.__res = c.done ? 'done' : 'data'; },\
               function (e) { globalThis.__res = 'err'; globalThis.__errName = (e && e.name) || ''; }\
             );",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__res)").unwrap().value,
        "err",
        "垃圾字节 → reader.read() reject（不静默吞 / 不挂起）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__errName)").unwrap().value,
        "DataError",
        "损坏 gzip 流 → DataError"
    );

    // 格式错配：gzip 压缩字节喂给 DecompressionStream('deflate')（zlib 期望 0x78 头）→ reject DataError。
    // 先用 CompressionStream('gzip') 产出合法 gzip 字节，再喂错 deflate 解码器。
    sandbox
        .execute(
            "globalThis.__res2 = '(none)'; globalThis.__errName2 = '';\
             var payload = new TextEncoder().encode('cross-format mismatch probe');\
             var csrc = new ReadableStream({\
               start: function (c) { c.enqueue(payload); c.close(); }\
             });\
             var comp = new CompressionStream('gzip');\
             csrc.pipeTo(comp.writable);\
             var cr = comp.readable.getReader();\
             cr.read().then(function (chunk) {\
               if (chunk.done) { globalThis.__res2 = 'src-done'; return; }\
               var dsrc = new ReadableStream({\
                 start: function (c) { c.enqueue(chunk.value); c.close(); }\
               });\
               var ds = new DecompressionStream('deflate');\
               dsrc.pipeTo(ds.writable);\
               var reader = ds.readable.getReader();\
               reader.read().then(\
                 function (cc) { globalThis.__res2 = cc.done ? 'done' : 'data'; },\
                 function (e) { globalThis.__res2 = 'err'; globalThis.__errName2 = (e && e.name) || ''; }\
               );\
             });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__res2)").unwrap().value,
        "err",
        "gzip 字节 → DecompressionStream('deflate') → reject（格式错配）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__errName2)").unwrap().value,
        "DataError",
        "格式错配 → DataError"
    );
}

#[test]
fn test_window_context_globals_r2987() {
    // R2987：window.isSecureContext / crossOriginIsolated / reportError。库 feature-detect 后再使用
    // secure-only API（crypto.subtle / SharedArrayBuffer / Service Worker）或经 reportError 转错误事件。
    // 此前三者全缺 → feature-detect 走「不可用」分支、reportError 抛 ReferenceError。isSecureContext 取协议
    //（http/ws 不安全，余皆安全）；crossOriginIsolated=false（headless 无 COOP/COEP）；reportError 派发
    // ErrorEvent 到 window 'error' listener。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };

    // http: 协议 → isSecureContext = false。
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("http://test.local/".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    assert_eq!(
        sandbox.execute("String(globalThis.isSecureContext)").unwrap().value,
        "false",
        "http: 协议 → isSecureContext = false"
    );

    // about:blank（继承安全上下文）→ isSecureContext = true。
    let mut sandbox2 = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox2.execute(generate_js_dom_shim()).unwrap();
    let mutations2: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html2: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url2: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox2, &mutations2, &dom_html2, &page_url2);
    assert_eq!(
        sandbox2
            .execute("String(globalThis.isSecureContext)")
            .unwrap()
            .value,
        "true",
        "about:blank → isSecureContext = true（secure context）"
    );

    // crossOriginIsolated = false（headless 无 COOP/COEP）。
    assert_eq!(
        sandbox
            .execute("String(globalThis.crossOriginIsolated)")
            .unwrap()
            .value,
        "false",
        "crossOriginIsolated = false（无 COOP/COEP）"
    );

    // reportError：派发 ErrorEvent 到 window 'error' listener（message 字段）+ onerror IDL handler。
    sandbox
        .execute(
            "globalThis.__errMsg = '(none)'; globalThis.__onerr = '(none)';\
             window.addEventListener('error', function (e) { globalThis.__errMsg = e.message; });\
             window.onerror = function (msg) { globalThis.__onerr = String(msg); };\
             reportError(new Error('boom-failure'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__errMsg)").unwrap().value,
        "boom-failure",
        "reportError → window 'error' listener 收 ErrorEvent.message"
    );
    // typeof reportError === 'function'（防 ReferenceError）。
    assert_eq!(
        sandbox.execute("typeof reportError").unwrap().value,
        "function",
        "reportError 是 function"
    );
}

#[test]
fn test_navigator_env_info_r2988() {
    // R2988：navigator.deviceMemory / connection（Network Information API）/ userAgentData
    //（UA Client Hints）。RUM/analytics（GA）/ 自适应加载库 feature-detect 读这些决定上报/资源质量，
    // 此前三者全缺 → feature-detect 走「不可用」分支。headless 取静态 '4g'/8GB/Chromium-120 近似
    //（real 浏览器桌面默认亦 '4g'）。
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

    // deviceMemory：number（GB，spec 离散值之一）。
    assert_eq!(
        sandbox.execute("typeof navigator.deviceMemory").unwrap().value,
        "number",
        "navigator.deviceMemory 是 number"
    );
    assert!(
        sandbox
            .execute("String(navigator.deviceMemory > 0)")
            .unwrap()
            .value
            == "true",
        "navigator.deviceMemory > 0"
    );

    // connection（Network Information API）：effectiveType='4g' + downlink/rtt/saveData + EventTarget no-op。
    assert_eq!(
        sandbox
            .execute("String(navigator.connection.effectiveType)")
            .unwrap()
            .value,
        "4g",
        "navigator.connection.effectiveType = '4g'"
    );
    assert_eq!(
        sandbox
            .execute("typeof navigator.connection.downlink")
            .unwrap()
            .value,
        "number",
        "navigator.connection.downlink 是 number"
    );
    assert_eq!(
        sandbox
            .execute("String(navigator.connection.saveData)")
            .unwrap()
            .value,
        "false",
        "navigator.connection.saveData = false"
    );
    // addEventListener 注册有效（不抛）。
    sandbox
        .execute(
            "globalThis.__ok = 'no';\
             navigator.connection.addEventListener('change', function () {});\
             globalThis.__ok = 'yes';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ok)").unwrap().value,
        "yes",
        "navigator.connection.addEventListener 注册不抛"
    );

    // userAgentData（UA Client Hints）：brands/mobile/platform + getHighEntropyValues Promise。
    assert_eq!(
        sandbox
            .execute("String(navigator.userAgentData.mobile)")
            .unwrap()
            .value,
        "false",
        "navigator.userAgentData.mobile = false"
    );
    assert_eq!(
        sandbox
            .execute("String(navigator.userAgentData.platform)")
            .unwrap()
            .value,
        "Windows",
        "navigator.userAgentData.platform = 'Windows'"
    );
    assert_eq!(
        sandbox
            .execute("String(navigator.userAgentData.brands.length > 0)")
            .unwrap()
            .value,
        "true",
        "navigator.userAgentData.brands 非空"
    );

    // getHighEntropyValues：返 Promise，resolve 含请求的高熵字段（platformVersion/architecture）。
    sandbox
        .execute(
            "globalThis.__hev = '(none)';\
             navigator.userAgentData\
               .getHighEntropyValues(['platformVersion', 'architecture'])\
               .then(function (v) { globalThis.__hev = v.platformVersion + '|' + v.architecture; });",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__hev)").unwrap().value,
        "15.0.0|x86",
        "getHighEntropyValues resolve 含 platformVersion + architecture"
    );

    // toJSON：含 brands/mobile/platform。
    sandbox
        .execute("globalThis.__json = JSON.stringify(navigator.userAgentData.toJSON());")
        .unwrap();
    assert!(
        sandbox
            .execute("String(globalThis.__json.indexOf('brands') >= 0)")
            .unwrap()
            .value
            == "true",
        "userAgentData.toJSON() 含 brands"
    );
}

#[test]
fn test_dommatrix_matrix_math_r2989() {
    // R2989：DOMMatrix 矩阵运算覆盖加固。R2985 shipped DOMMatrix 但 driving test 仅覆盖 identity/
    // from-array/translate/scale/transformPoint(translate)，**multiply/inverse/rotate/multiplySelf/
    // fromMatrix 未测**——矩阵运算（column-major multiply / Gauss-Jordan inverse / rotate Z）为最易藏
    // subtle bug 的算法代码，补 known-answer 覆盖锁定正确性（Done Criteria §5 测试质量）。
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

    // multiply：A.scale(2,2) × B.translate(5,0) = {a=2, d=2, e=10}（A(B(p)) = 2·(p+(5,0))）。
    sandbox
        .execute("globalThis.__m = new DOMMatrix().scale(2, 2).multiply(new DOMMatrix().translate(5, 0));")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__m.a)").unwrap().value,
        "2",
        "scale(2,2)×translate(5,0) → a=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__m.d)").unwrap().value,
        "2",
        "scale(2,2)×translate(5,0) → d=2"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__m.e)").unwrap().value,
        "10",
        "scale(2,2)×translate(5,0) → e=10（2·5）"
    );

    // multiply 非交换验证：translate(5,0)×scale(2,2) ≠ 上（B(A(p)) = 2·p+(5,0)，e=5 非 10）。
    sandbox
        .execute("globalThis.__m2 = new DOMMatrix().translate(5, 0).multiply(new DOMMatrix().scale(2, 2));")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__m2.e)").unwrap().value,
        "5",
        "translate(5,0)×scale(2,2) → e=5（非交换，序敏感）"
    );

    // inverse：translate(5,10).inverse() = translate(-5,-10)。
    sandbox
        .execute("globalThis.__inv = new DOMMatrix([1,0,0,1,5,10]).inverse();")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__inv.e)").unwrap().value,
        "-5",
        "translate(5,10).inverse() → e=-5"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__inv.f)").unwrap().value,
        "-10",
        "translate(5,10).inverse() → f=-10"
    );

    // inverse scale：scale(2,4).inverse() = scale(0.5,0.25)。
    sandbox
        .execute("globalThis.__invs = new DOMMatrix().scale(2, 4).inverse();")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__invs.a)").unwrap().value,
        "0.5",
        "scale(2,4).inverse() → a=0.5"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__invs.d)").unwrap().value,
        "0.25",
        "scale(2,4).inverse() → d=0.25"
    );

    // multiply × inverse 往返 = identity（数值容差 < 1e-6）。
    sandbox
        .execute(
            "var orig = new DOMMatrix([2,0,0,3,5,7]);\
             var rt = orig.multiply(orig.inverse());\
             globalThis.__rtClose = String(Math.abs(rt.a - 1) < 1e-6 && Math.abs(rt.d - 1) < 1e-6 && Math.abs(rt.e) < 1e-6);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rtClose)").unwrap().value,
        "true",
        "M × M.inverse() ≈ identity（数值容差）"
    );

    // rotate(90°)：a=cos90≈0, b=sin90=1, c=-sin90=-1, d=cos90≈0。
    sandbox
        .execute(
            "var r = new DOMMatrix().rotate(90);\
             globalThis.__rA = String(Math.abs(r.a) < 1e-6);\
             globalThis.__rB = String(Math.round(r.b));\
             globalThis.__rC = String(Math.round(r.c));\
             globalThis.__rD = String(Math.abs(r.d) < 1e-6);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__rA)").unwrap().value,
        "true",
        "rotate(90) a=cos90≈0（容差）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rB)").unwrap().value,
        "1",
        "rotate(90) b=sin90=1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rC)").unwrap().value,
        "-1",
        "rotate(90) c=-sin90=-1"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rD)").unwrap().value,
        "true",
        "rotate(90) d=cos90≈0（容差）"
    );

    // multiplySelf：identity.multiplySelf(translate(5,0)) → e=5，原对象 mutated。
    sandbox
        .execute(
            "var ms = new DOMMatrix();\
             var ret = ms.multiplySelf(new DOMMatrix().translate(5, 0));\
             globalThis.__msRetIsSelf = String(ret === ms);\
             globalThis.__msE = String(ms.e);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__msRetIsSelf)").unwrap().value,
        "true",
        "multiplySelf 返 this（mutate 原对象）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__msE)").unwrap().value,
        "5",
        "identity.multiplySelf(translate(5,0)) → e=5"
    );

    // fromMatrix：独立副本（改原不影响副本）。
    sandbox
        .execute(
            "var src = new DOMMatrix([1,2,3,4,5,6]);\
             var cp = DOMMatrix.fromMatrix(src);\
             src.e = 999;\
             globalThis.__cpE = String(cp.e);\
             globalThis.__cpF = String(cp.f);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__cpE)").unwrap().value,
        "5",
        "fromMatrix 独立副本（改原 src.e=999 不影响 cp.e=5）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cpF)").unwrap().value,
        "6",
        "fromMatrix 副本 f=6"
    );

    // transformPoint with scale：scale(2,3).transformPoint({x:3,y:4}) = (6,12)。
    sandbox
        .execute("globalThis.__tp = new DOMMatrix().scale(2, 3).transformPoint({ x: 3, y: 4 });")
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__tp.x)").unwrap().value,
        "6",
        "scale(2,3).transformPoint(3,4) → x=6"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__tp.y)").unwrap().value,
        "12",
        "scale(2,3).transformPoint(3,4) → y=12"
    );
}

// TEMP PROBE removed — replaced by real assertions below (R2990).

#[test]
fn test_document_evaluate_xpath_edges_r2990() {
    // R2990：XPath 实用子集边界覆盖 + 两个 spec bug 修复（R2981 shipped 但 driving test 仅覆盖主干路径）。
    //
    // Bug A（part06.js _xpathPred @attr 分支）：`@attr != 'val'` 对【缺失该属性】的节点，旧代码将 null 归一
    //   为 '' 再比较 → `'' != 'val'` 为 true → 无该属性的节点被错误命中。XPath 1.0 §3.4 存在量词语义：
    //   `@attr` 是节点集，当其为空（属性缺失）时，`=`/`!=` 比较皆为 false（不存在满足的节点）。修复：
    //   av==null 且带比较运算符时直接 return false。
    //
    // Bug B（part06.js _xpathParseStep/@​ 轴 + _xpathApplyStep 属性分支）：`//@name`（descendant 属性轴）
    //   旧代码返空——`_xpathParseStep` 对 '@' 开头 token 硬编码 axis='attribute'，丢弃了 `//` 传入的
    //   'descendant' 轴；`_xpathApplyStep` 属性分支仅检查 ctx 自身。修复：保留 fromDesc 标志，属性分支
    //   fromDesc 时扩展到 ctx + 全部后代元素（descendant-or-self 语义）。
    //
    // 另覆盖此前未测但已正确的路径：`..` parent 轴 + dedup、`.` self 轴、`not()` 谓词、`text()`/`node()`
    //   直接步、`comment()` 节点测试、`@*` 通配属性、`position()<1` / `last()-0` 边界。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // ul#list > li(item/item+active/item/active)；1 注释节点；3 a（/a 无 class、/b class=x、/c class=ext）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <ul id='list'>\
         <li class='item'>A</li>\
         <li class='item active'>B</li>\
         <li class='item'>C</li>\
         <li class='active'>D</li>\
         </ul>\
         <!-- a comment node -->\
         <a href='/a'>link-a</a>\
         <a href='/b' class='x'>link-b</a>\
         <a href='/c' class='ext'>link-c</a>\
         <div id='box'><p>hello</p><p>world</p></div>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);
    sandbox
        .execute(
            "globalThis.__xp = function(expr, ctx, type) {\
             return document.evaluate(expr, ctx || document, null,\
             (type == null ? XPathResult.ORDERED_NODE_SNAPSHOT_TYPE : type), null);\
             };",
        )
        .unwrap();

    // ── Bug A：`@class != 'ext'` 不得命中无 class 属性的节点 ─────────────────────────
    // a 集：/a（无 class）、/b（class=x）、/c（class=ext）。仅 /b 有 class 且 != 'ext' → spec = 1。
    // 旧 bug：/a 无 class → ''!='ext' → true → 错误命中 /a → 返 2。
    assert_eq!(
        sandbox
            .execute("__xp(\"//a[@class!='ext']\").snapshotLength")
            .unwrap()
            .value,
        "1",
        "Bug A：//a[@class!='ext'] = 1（仅 /b；无 class 的 /a 不应命中）"
    );
    // 对照：`@class = 'x'` → /b = 1。
    assert_eq!(
        sandbox
            .execute("__xp(\"//a[@class='x']\").snapshotLength")
            .unwrap()
            .value,
        "1",
        "//a[@class='x'] = 1（/b）"
    );
    // 对照：`[@class]` 存在性 → 有 class 的 a = /b、/c = 2。
    assert_eq!(
        sandbox.execute("__xp('//a[@class]').snapshotLength").unwrap().value,
        "2",
        "//a[@class] = 2（/b、/c 存在 class 属性）"
    );
    // `@href != '/a'` 应排除 /a 自身但也不应额外命中——此处所有 a 都有 href，正常 = 2（/b、/c）。
    assert_eq!(
        sandbox
            .execute("__xp(\"//a[@href!='/a']\").snapshotLength")
            .unwrap()
            .value,
        "2",
        "//a[@href!='/a'] = 2（/b、/c；所有 a 均有 href）"
    );

    // ── Bug B：`//@href` descendant 属性轴应收集所有后代元素的 href ─────────────────
    // 旧 bug：_xpathParseStep 丢弃 descendant 轴 → 仅检查 documentElement → 返 0。
    assert_eq!(
        sandbox
            .execute("__xp('//@href').snapshotLength")
            .unwrap()
            .value,
        "3",
        "Bug B：//@href = 3（descendant 属性轴收集全部 a 的 href）"
    );
    // 文档序首项 value = '/a'。
    assert_eq!(
        sandbox
            .execute("__xp('//@href').snapshotItem(0).value")
            .unwrap()
            .value,
        "/a",
        "//@href 文档序首项 value = '/a'"
    );
    // 对照：`//*[@href]`（descendant 元素 + 谓词，非属性轴）亦 = 3，确认两条路径一致。
    assert_eq!(
        sandbox
            .execute("__xp('//*[@href]').snapshotLength")
            .unwrap()
            .value,
        "3",
        "//*[@href] = 3（descendant 元素谓词路径）"
    );

    // ── parent 轴 `..` + 多上下文 dedup ────────────────────────────────────────────
    // //li 的 4 个 li 同属 ul#list → `..` 全指向同一 ul → dedup = 1。
    assert_eq!(
        sandbox.execute("__xp('//li/..').snapshotLength").unwrap().value,
        "1",
        "//li/.. = 1（4 个 li 的 parent 同为 ul → dedup）"
    );
    assert_eq!(
        sandbox
            .execute("__xp('//li/..').snapshotItem(0).id")
            .unwrap()
            .value,
        "list",
        "//li/.. 命中 ul#list"
    );

    // ── self 轴 `.` ───────────────────────────────────────────────────────────────
    assert_eq!(
        sandbox.execute("__xp('//li/.').snapshotLength").unwrap().value,
        "4",
        "//li/. = 4（self 轴保留全部 li）"
    );

    // ── not() 谓词 ────────────────────────────────────────────────────────────────
    // not(@class='item')：A/C（class=item）→ false；B（'item active'!='item'→谓词 false→not true）；D 同理 → B、D = 2。
    assert_eq!(
        sandbox
            .execute("__xp(\"//li[not(@class='item')]\").snapshotLength")
            .unwrap()
            .value,
        "2",
        "//li[not(@class='item')] = 2（B、D）"
    );

    // ── text() / node() 直接步 ─────────────────────────────────────────────────────
    assert_eq!(
        sandbox
            .execute("__xp('//ul/li/text()').snapshotLength")
            .unwrap()
            .value,
        "4",
        "//ul/li/text() = 4（每个 li 一个文本子节点）"
    );
    assert_eq!(
        sandbox
            .execute("__xp('//ul/li/text()').snapshotItem(0).nodeValue")
            .unwrap()
            .value,
        "A",
        "//ul/li/text()[0].nodeValue = 'A'"
    );
    assert_eq!(
        sandbox
            .execute("__xp('//ul/li/node()').snapshotLength")
            .unwrap()
            .value,
        "4",
        "//ul/li/node() = 4（文本节点亦匹配 node()）"
    );

    // ── comment() 节点测试 ─────────────────────────────────────────────────────────
    assert_eq!(
        sandbox
            .execute("__xp('//comment()').snapshotLength")
            .unwrap()
            .value,
        "1",
        "//comment() = 1（注释节点经 html5ever 解析入 DOM）"
    );
    assert_eq!(
        sandbox
            .execute("String(__xp('//comment()').snapshotItem(0).nodeType)")
            .unwrap()
            .value,
        "8",
        "//comment() 命中节点 nodeType = 8"
    );

    // ── @* 通配属性轴 ──────────────────────────────────────────────────────────────
    assert_eq!(
        sandbox.execute("__xp('//ul/@*').snapshotLength").unwrap().value,
        "1",
        "//ul/@* = 1（ul 仅有 id 属性）"
    );
    assert_eq!(
        sandbox
            .execute("__xp('//ul/@*').snapshotItem(0).name")
            .unwrap()
            .value,
        "id",
        "//ul/@*[0].name = 'id'"
    );

    // ── 谓词边界：position()<1（永不命中）/ last()-0 ───────────────────────────────
    assert_eq!(
        sandbox
            .execute("__xp('//li[position()<1]').snapshotLength")
            .unwrap()
            .value,
        "0",
        "//li[position()<1] = 0（position 1-based，<1 永不命中）"
    );
    assert_eq!(
        sandbox
            .execute("__xp('//li[last()-0]').snapshotItem(0).textContent")
            .unwrap()
            .value,
        "D",
        "//li[last()-0] = D（last()-0 等价 last()）"
    );
}

#[test]
fn test_custom_elements_attr_changed_callback_r2992() {
    // R2992：custom element lifecycle slice——attributeChangedCallback + observedAttributes（R2813 仅 shipped
    // registry bookkeeping，lifecycle 回调全 defer）。element 为 generic Proxy 非 ctor 实例（upgrade/ctor
    // 调用仍 defer），本 slice 落地 CE 最常用的可观察行为：setAttribute/removeAttribute 命中 observedAttributes
    // 时，分派 ctor.prototype.attributeChangedCallback.call(element, name, old, new)。old 经变更前 getAttribute
    // 读（首次 set old=null，remove new=null）；值真变才入队（set/remove 同值无 change）。
    // connectedCallback/disconnectedCallback/upgrade/adoptedCallback 仍 defer（独立 slice）。
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

    // 定义带 observedAttributes + attributeChangedCallback 的 custom element，记录每次调用。
    sandbox
        .execute(
            "globalThis.__calls = [];\
             class MyCounter extends HTMLElement {\
               static get observedAttributes() { return ['count', 'label']; }\
               attributeChangedCallback(name, oldVal, newVal) {\
                 globalThis.__calls.push(name + ':' + oldVal + '->' + newVal);\
               }\
             }\
             customElements.define('my-counter', MyCounter);\
             var el = document.createElement('my-counter');\
             document.body.appendChild(el);\
             el.setAttribute('count', '5');\
             el.setAttribute('count', '10');\
             el.setAttribute('label', 'hi');\
             el.setAttribute('data-x', '1');\
             el.removeAttribute('count');\
             el.setAttribute('label', 'hi');",
        )
        .unwrap();
    // 期望序列：count:null->5（首次 set old=null）| count:5->10 | label:null->hi |（data-x 未观察，跳过）
    // | count:10->null（remove → new=null）|（label 同值 'hi'，无 change，跳过）。
    // 注：remove 后再 setAttribute 的 old 值受 handle 元素 removeAttribute「set-empty 而非真移除」既有限制
    // 影响（hasAttribute 仍 true、old=''），故本切片不测该复合边角——handle true-removal（RemoveAttrOnHandle
    // 变体）为独立 follow-up（涉 DomMutation apply 管线，engine 共享面）。
    assert_eq!(
        sandbox.execute("globalThis.__calls.join('|')").unwrap().value,
        "count:null->5|count:5->10|label:null->hi|count:10->null",
        "attributeChangedCallback 按 observedAttributes 过滤分派，old(null on first set)/new(null on remove) 值正确，未观察/同值跳过"
    );

    // 非自定义元素（<div>）setAttribute 不触发任何 CE 回调（registry 无 'div'）。
    sandbox
        .execute(
            "globalThis.__before = globalThis.__calls.length;\
             var d = document.createElement('div');\
             document.body.appendChild(d);\
             d.setAttribute('count', '1');\
             d.setAttribute('label', '2');\
             globalThis.__after = globalThis.__calls.length;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__before === globalThis.__after)").unwrap().value,
        "true",
        "非自定义元素 setAttribute 不分派 attributeChangedCallback"
    );

    // 多观察属性 + 连续变更：count 与 label 交替。
    sandbox
        .execute(
            "globalThis.__calls2 = [];\
             class MyTag extends HTMLElement {\
               static get observedAttributes() { return ['a', 'b', 'c']; }\
               attributeChangedCallback(name, oldVal, newVal) {\
                 globalThis.__calls2.push(name + '=' + newVal);\
               }\
             }\
             customElements.define('my-tag', MyTag);\
             var t = document.createElement('my-tag');\
             document.body.appendChild(t);\
             t.setAttribute('b', '1');\
             t.setAttribute('a', '2');\
             t.setAttribute('c', '3');\
             t.setAttribute('b', '4');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__calls2.join(',')").unwrap().value,
        "b=1,a=2,c=3,b=4",
        "多观察属性交替变更均分派（顺序 = setAttribute 调用序）"
    );

    // hasAttribute 对 handle（createElement）元素生效（latent gap 修复——此前 handle-only 恒 false）。
    // 注：removeAttribute 对 handle 元素为「set-empty 而非真移除」（既有限制，同上），故 remove 后
    // hasAttribute 仍 true；该 true-removal 为独立 follow-up，此处只测 set 前/后 存在性。
    sandbox
        .execute(
            "var hg = document.createElement('my-tag');\
             document.body.appendChild(hg);\
             globalThis.__hgBefore = String(hg.hasAttribute('a'));\
             hg.setAttribute('a', 'x');\
             globalThis.__hgAfter = String(hg.hasAttribute('a'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hgBefore").unwrap().value,
        "false",
        "createElement 元素 setAttribute 前 hasAttribute=false"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hgAfter").unwrap().value,
        "true",
        "createElement 元素 setAttribute 后 hasAttribute=true（handle 路径生效，latent gap 修复）"
    );
}

#[test]
fn test_handle_remove_attribute_true_removal_r2993() {
    // R2993：handle（createElement）元素 removeAttribute 真移除（RemoveAttrOnHandle 变体）。
    // R2992 发现的 latent gap：handle 元素 removeAttribute 旧实现 set-empty（__zw_set_attr_handle ''），
    // 致 remove 后 hasAttribute 仍 true、custom element post-remove setAttribute 的 old='' 而非 null。
    // 本切片加 RemoveAttrOnHandle 变体 + latest-wins query，闭合三处：
    //   ① hasAttribute-after-remove = false；② getAttribute-after-remove = absent（空串）；
    //   ③ CE attributeChangedCallback set→remove→set 第二次 set old=null（R2992 限于既存 gap 未测此复合）。
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
        "<html><body><div id='d' class='c'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // handle 元素 set→remove→set：CE old=null 闭合 + hasAttribute/getAttribute 反映真移除。
    sandbox
        .execute(
            "globalThis.__calls = [];\
             class C extends HTMLElement {\
               static get observedAttributes() { return ['count']; }\
               attributeChangedCallback(n, o, v) { globalThis.__calls.push(n + ':' + o + '->' + v); }\
             }\
             customElements.define('r2993-el', C);\
             var e = document.createElement('r2993-el');\
             document.body.appendChild(e);\
             e.setAttribute('count', '5');\
             e.removeAttribute('count');\
             globalThis.__hasAfterRemove = String(e.hasAttribute('count'));\
             globalThis.__getAfterRemove = String(e.getAttribute('count'));\
             e.setAttribute('count', '10');\
             globalThis.__hasAfterReset = String(e.hasAttribute('count'));\
             globalThis.__getAfterReset = String(e.getAttribute('count'));",
        )
        .unwrap();
    // CE 序列：count:null->5（首 set）| count:5->null（remove）| count:null->10（remove 后再 set，old=null 而非 ''）。
    assert_eq!(
        sandbox.execute("globalThis.__calls.join('|')").unwrap().value,
        "count:null->5|count:5->null|count:null->10",
        "handle 元素 set→remove→set：第二次 set old=null（RemoveAttrOnHandle 真移除闭合）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasAfterRemove").unwrap().value,
        "false",
        "handle 元素 removeAttribute 后 hasAttribute=false（真移除，非 set-empty 残留）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__getAfterRemove").unwrap().value,
        "",
        "handle 元素 removeAttribute 后 getAttribute=空串（absent）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasAfterReset").unwrap().value,
        "true",
        "handle 元素 remove 后再 setAttribute → hasAttribute=true"
    );
    assert_eq!(
        sandbox.execute("globalThis.__getAfterReset").unwrap().value,
        "10",
        "handle 元素 remove 后再 setAttribute('10') → getAttribute=10"
    );

    // 注：sel-based removeAttribute 路径未变（分支逻辑 `else if __zw_remove_attr(sel,n)` 与 R2657 一致），
    // 不在此重复断言——sel-based hasAttribute 读 HTML 快照（非 mutation 列表），removeAttribute 后快照
    // 仍 stale（render apply 后才反映），故 JS 层 hasAttribute-after-remove 对 sel 元素恒 true（既存限制，
    // 与本切片 handle 真移除无关）。handle 元素经 mutation 列表 latest-wins 读，故无此 stale。
}

#[test]
fn test_custom_elements_connected_callback_r2994() {
    // R2994：custom element lifecycle slice——connectedCallback / disconnectedCallback。
    // R2992 落地 attributeChangedCallback；本切片落地连入/断开回调（spec HTML §4.13）。element 为 generic
    // Proxy 非 ctor 实例（upgrade/ctor 仍 defer），以 element proxy 为 this 分派 ctor.prototype 上的回调。
    // 连入态由 JS 端 _ceConn 追踪（host 快照不实时反映 handle 元素挂载），appendChild/insertBefore/
    // removeChild/remove/replaceChild/append/replaceChildren/prepend/before/after 单点 hook：插入到已连入父
    // → 子树连入（connectedCallback），移除 → 断开（disconnectedCallback）；状态真转才调（再连再调、同态跳过）。
    // 已知限制（既存）：parsed HTML 源中的 <my-el> 不经 JS appendChild → 初始 connectedCallback 不触发。
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

    // 基础连入/断开/再连：connectedCallback 与 disconnectedCallback 双向均可重复触发。
    sandbox
        .execute(
            "globalThis.__log = [];\
             class CE extends HTMLElement {\
               connectedCallback() { globalThis.__log.push('+'); }\
               disconnectedCallback() { globalThis.__log.push('-'); }\
             }\
             customElements.define('r2994-a', CE);\
             var el = document.createElement('r2994-a');\
             document.body.appendChild(el);\
             document.body.removeChild(el);\
             document.body.appendChild(el);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__log.join('|')").unwrap().value,
        "+|-|+",
        "connectedCallback/disconnectedCallback 双向触发，再连再调（appendChild→+ / removeChild→- / appendChild→+）"
    );

    // el.remove()（self 级）断开 → disconnectedCallback（独立 class/log，避免与 CE 共享 __log）。
    sandbox
        .execute(
            "globalThis.__log2 = [];\
             class CE2 extends HTMLElement {\
               connectedCallback() { globalThis.__log2.push('+'); }\
               disconnectedCallback() { globalThis.__log2.push('-'); }\
             }\
             customElements.define('r2994-b', CE2);\
             var e2 = document.createElement('r2994-b');\
             document.body.appendChild(e2);\
             e2.remove();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__log2.join('|')").unwrap().value,
        "+|-",
        "el.remove() 触发 disconnectedCallback（self 级断连）"
    );

    // detached 容器内 append 不触发；容器连入时子树（含 custom element 后代）一并连入。
    sandbox
        .execute(
            "globalThis.__log3 = [];\
             class Sub extends HTMLElement {\
               connectedCallback() { globalThis.__log3.push('c'); }\
               disconnectedCallback() { globalThis.__log3.push('d'); }\
             }\
             customElements.define('r2994-sub', Sub);\
             var box = document.createElement('div');\
             var sub = document.createElement('r2994-sub');\
             box.appendChild(sub);\
             globalThis.__afterDetach = globalThis.__log3.slice();\
             document.body.appendChild(box);\
             globalThis.__afterAttach = globalThis.__log3.slice();\
             document.body.removeChild(box);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__afterDetach.join('|')").unwrap().value,
        "",
        "detached 容器内 appendChild 不触发 connectedCallback（父未连入）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__afterAttach.join('|')").unwrap().value,
        "c",
        "容器连入时其 custom element 后代随之连入（子树传播）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__log3.join('|')").unwrap().value,
        "c|d",
        "容器移除时其 custom element 后代随之断开（removeChild 子树断连）"
    );

    // insertBefore 连入 + 同树内移动不重复触发（状态未变）。
    sandbox
        .execute(
            "globalThis.__log4 = [];\
             class CE4 extends HTMLElement {\
               connectedCallback() { globalThis.__log4.push('+'); }\
               disconnectedCallback() { globalThis.__log4.push('-'); }\
             }\
             customElements.define('r2994-c', CE4);\
             var a = document.createElement('r2994-c');\
             var b = document.createElement('r2994-c');\
             document.body.appendChild(a);\
             document.body.appendChild(b);\
             globalThis.__beforeMove = globalThis.__log4.length;\
             document.body.insertBefore(a, b);\
             globalThis.__moveDelta = globalThis.__log4.length - globalThis.__beforeMove;",
        )
        .unwrap();
    // a、b 各连入一次（++）；insertBefore 把已连入的 a 移到 b 前——状态未变（仍连入）→ 无新回调。
    assert_eq!(
        sandbox.execute("globalThis.__log4.join('|')").unwrap().value,
        "+|+",
        "insertBefore 连入新节点（a、b 各一次 +）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__moveDelta)").unwrap().value,
        "0",
        "同树内移动已连入元素不重复触发 connectedCallback（状态未变）"
    );

    // 非自定义元素 append 不触发任何回调；仅定义 connectedCallback（无 disconnectedCallback）移除不抛。
    sandbox
        .execute(
            "globalThis.__log5 = [];\
             globalThis.__before = globalThis.__log5.length;\
             var d = document.createElement('div');\
             document.body.appendChild(d);\
             document.body.removeChild(d);\
             globalThis.__after = globalThis.__log5.length;\
             class OnlyConn extends HTMLElement {\
               connectedCallback() { globalThis.__log5.push('+'); }\
             }\
             customElements.define('r2994-only', OnlyConn);\
             var oc = document.createElement('r2994-only');\
             document.body.appendChild(oc);\
             oc.remove();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__before === globalThis.__after)").unwrap().value,
        "true",
        "非自定义元素 append/remove 不触发 custom element 回调"
    );
    // 仅 connectedCallback：append 触发 +，remove 无 disconnectedCallback 定义 → 不抛、不记（graceful）。
    assert_eq!(
        sandbox.execute("globalThis.__log5.join('|')").unwrap().value,
        "+",
        "仅定义 connectedCallback 时移除 graceful（无 disconnectedCallback 不抛、不记）"
    );
}

#[test]
fn test_sel_based_attr_latest_wins_r2995() {
    // R2995：sel-based（parsed / getElementById）元素 getAttribute/hasAttribute snapshot-stale 闭合。
    // R2993 latent gap：sel 元素旧读 HTML 快照（render apply 前不反映同批 SetAttr/RemoveAttr）→ removeAttribute
    // 后 hasAttribute 恒 true、getAttribute 仍返旧值。本切片让 __zw_get_attr/__zw_has_attr 先 consult 变更列表
    //（selector-keyed SetAttr/RemoveAttr latest-wins），无命中再回落快照——与 handle 路径（R2993）对称。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // div#d 带 data-x/class；input#cb checkbox checked（boolean 反射读 __zw_has_attr）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <div id='d' data-x='orig' class='c'></div>\
         <input id='cb' type='checkbox' checked>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // setAttribute → getAttribute 反映新值（旧实现读 stale 快照返 'orig'）。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             globalThis.__origGet = d.getAttribute('data-x');\
             d.setAttribute('data-x', 'changed');\
             globalThis.__afterSetGet = d.getAttribute('data-x');\
             globalThis.__afterSetHas = String(d.hasAttribute('data-x'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__origGet").unwrap().value,
        "orig",
        "初始 getAttribute 读快照原值（无变更覆盖）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__afterSetGet").unwrap().value,
        "changed",
        "setAttribute 后 getAttribute 反映新值（latest-wins 覆盖 stale 快照）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__afterSetHas").unwrap().value,
        "true",
        "setAttribute 后 hasAttribute=true"
    );

    // removeAttribute → hasAttribute false / getAttribute ''（旧实现 stale：true / 'changed'）。
    sandbox
        .execute(
            "d.removeAttribute('data-x');\
             globalThis.__afterRemHas = String(d.hasAttribute('data-x'));\
             globalThis.__afterRemGet = d.getAttribute('data-x');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__afterRemHas").unwrap().value,
        "false",
        "removeAttribute 后 hasAttribute=false（latest-wins RemoveAttr 覆盖 stale 快照 true）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__afterRemGet").unwrap().value,
        "",
        "removeAttribute 后 getAttribute=空串（absent）"
    );

    // remove 后再 set → 恢复（latest-wins：最近 SetAttr 命中）。
    sandbox
        .execute(
            "d.setAttribute('data-x', 'back');\
             globalThis.__resetGet = d.getAttribute('data-x');\
             globalThis.__resetHas = String(d.hasAttribute('data-x'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__resetGet").unwrap().value,
        "back",
        "remove 后再 setAttribute → getAttribute=新值（最近 SetAttr 命中）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__resetHas").unwrap().value,
        "true",
        "remove 后再 setAttribute → hasAttribute=true"
    );

    // hasAttribute（latest-wins 方法）对 boolean 属性 removeAttribute 后正确返 false（区别于 el.checked
    // 反射 getter 仍读纯快照 __zw_has_attr——其 stale 为独立 follow-up，本切片仅闭合 getAttribute/hasAttribute）。
    sandbox
        .execute(
            "var cb = document.getElementById('cb');\
             globalThis.__cbBefore = String(cb.hasAttribute('checked'));\
             cb.removeAttribute('checked');\
             globalThis.__cbAfter = String(cb.hasAttribute('checked'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cbBefore").unwrap().value,
        "true",
        "checkbox 初始 hasAttribute('checked')=true（快照）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__cbAfter").unwrap().value,
        "false",
        "removeAttribute('checked') 后 hasAttribute=false（latest-wins 方法，修正 stale true）"
    );

    // 关键不变量：.value= 不应污染 defaultValue（defaultValue 读纯快照 __zw_get_attr，不被 value setter 的
    // SetAttr{value} 覆盖——R2840 语义）。即 latest-wins 仅作用于 getAttribute/hasAttribute 方法，不波及反射 getter。
    sandbox
        .execute(
            "var vi = document.getElementById('d');\
             vi.setAttribute('value', 'attrval');\
             globalThis.__dvViaAttr = vi.getAttribute('value');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dvViaAttr").unwrap().value,
        "attrval",
        "setAttribute('value') 后 getAttribute('value') 反映新值（latest-wins 方法路径）"
    );
}

#[test]
fn test_input_value_defaultvalue_independence_r2996() {
    // R2996：INPUT `.value=` 与 `.defaultValue` 独立（spec：`.value=` 改 dirty 当前态，**不**改 defaultValue=
    // 初始 value 属性）。R2995 暴露的既存 imperfection：shim `.value=` 写 value 属性供 render（paint_input_value
    // 读属性），致 defaultValue（亦读 value 属性）被污染。本切片单独追踪「真默认值」：首次 `.value=` 前捕获
    // value 属性，setAttribute('value')/defaultValue=/removeAttribute('value') 重同步。
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
        "<html><body><input id='i' type='text' value='initial'></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // `.value=` 改当前态，defaultValue 不变（旧实现 defaultValue 被 .value= 污染为 'changed'）。
    sandbox
        .execute(
            "var i = document.getElementById('i');\
             globalThis.__dv0 = i.defaultValue;\
             globalThis.__val0 = i.value;\
             i.value = 'changed';\
             globalThis.__dv1 = i.defaultValue;\
             globalThis.__val1 = i.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dv0").unwrap().value,
        "initial",
        ".value= 前 defaultValue=初始 value 属性"
    );
    assert_eq!(
        sandbox.execute("globalThis.__val0").unwrap().value,
        "initial",
        ".value= 前 .value=defaultValue"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dv1").unwrap().value,
        "initial",
        ".value='changed' 后 defaultValue 仍='initial'（不被 dirty 当前态污染）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__val1").unwrap().value,
        "changed",
        ".value='changed' 后 .value='changed'（dirty 当前态）"
    );

    // 多次 .value= defaultValue 恒为初值（捕获仅一次，不重捕被污染的属性）。
    sandbox
        .execute(
            "i.value = 'second';\
             globalThis.__dv2 = i.defaultValue;\
             globalThis.__val2 = i.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dv2").unwrap().value,
        "initial",
        "二次 .value= 后 defaultValue 仍='initial'（捕获仅首次）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__val2").unwrap().value,
        "second",
        "二次 .value= 后 .value='second'"
    );

    // setAttribute('value') 重同步 defaultValue（spec：setAttribute 改默认值）+ .value 跟随。
    sandbox
        .execute(
            "i.setAttribute('value', 'reset');\
             globalThis.__dv3 = i.defaultValue;\
             globalThis.__val3 = i.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dv3").unwrap().value,
        "reset",
        "setAttribute('value','reset') 后 defaultValue='reset'（重同步默认值）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__val3").unwrap().value,
        "reset",
        "setAttribute('value') 后 .value 跟随='reset'"
    );

    // setAttribute 后再 .value=：defaultValue 锚定新默认值 'reset'（捕获 setAttribute 后的属性）。
    sandbox
        .execute(
            "i.value = 'dirty again';\
             globalThis.__dv4 = i.defaultValue;\
             globalThis.__val4 = i.value;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dv4").unwrap().value,
        "reset",
        "setAttribute 后 .value= 不改 defaultValue（锚定 setAttribute 后的 'reset'）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__val4").unwrap().value,
        "dirty again",
        "setAttribute 后 .value='dirty again'"
    );

    // .value= 后 getAttribute('value') 仍反映 dirty 写入的属性（render 车辆不变）。
    assert_eq!(
        sandbox.execute("i.getAttribute('value')").unwrap().value,
        "dirty again",
        ".value= 后 getAttribute('value')=dirty 写入值（属性仍供 render，latest-wins）"
    );

    // defaultValue= 显式设默认值（重同步），不联动已被 dirty 的 .value。
    sandbox
        .execute(
            "i.defaultValue = 'brandnew';\
             globalThis.__dv5 = i.defaultValue;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dv5").unwrap().value,
        "brandnew",
        "defaultValue='brandnew' 后 defaultValue='brandnew'（显式设默认值）"
    );
}

#[test]
fn test_boolean_reflected_getter_latest_wins_r2997() {
    // R2997：sel-based boolean 反射 getter（checked/hidden/disabled）旧读纯快照 __zw_has_attr（非 _lw）→
    // removeAttribute / .checked=false / .hidden=true / .disabled=false 后 stale（同 R2995 getAttribute/hasAttribute
    // 方法路径闭合前的 stale gap，但本 getter 路径漏了）。本切片闭合：sel getter 改 latest-wins。
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
         <input id='cb' type='checkbox' checked>\
         <input id='dis' type='text' disabled>\
         <input id='plain' type='text'>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // removeAttribute('checked') 后 el.checked 应 false（旧 stale 返 true——getter 读快照未反映同批 RemoveAttr）。
    sandbox
        .execute(
            "var cb = document.getElementById('cb');\
             globalThis.__cbBefore = String(cb.checked);\
             cb.removeAttribute('checked');\
             globalThis.__cbAfter = String(cb.checked);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cbBefore").unwrap().value,
        "true",
        "checkbox 初始 checked=true（快照）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__cbAfter").unwrap().value,
        "false",
        "removeAttribute('checked') 后 el.checked=false（latest-wins getter，修正 stale true）"
    );

    // .checked = false 后 el.checked 应 false（旧 stale：setter 推 RemoveAttr{checked}，getter 读快照仍 true）。
    sandbox
        .execute(
            "var cb2 = document.getElementById('cb');\
             cb2.setAttribute('checked', '');\
             globalThis.__cbReSet = String(cb2.checked);\
             cb2.checked = false;\
             globalThis.__cbUnset = String(cb2.checked);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__cbReSet").unwrap().value,
        "true",
        "setAttribute('checked') 后 el.checked=true（latest-wins getter 反映同批 SetAttr）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__cbUnset").unwrap().value,
        "false",
        ".checked=false 后 el.checked=false（latest-wins getter 反映 setter 推的 RemoveAttr）"
    );

    // .disabled = false（预置 disabled）后 el.disabled 应 false（旧 stale 返 true）。
    sandbox
        .execute(
            "var dis = document.getElementById('dis');\
             globalThis.__disBefore = String(dis.disabled);\
             dis.disabled = false;\
             globalThis.__disAfter = String(dis.disabled);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__disBefore").unwrap().value,
        "true",
        "disabled 初始=true（快照）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__disAfter").unwrap().value,
        "false",
        ".disabled=false 后 el.disabled=false（latest-wins getter 反映 RemoveAttr）"
    );

    // .hidden = true（无预置 hidden）后 el.hidden 应 true（旧 stale 返 false——getter 读快照未反映同批 SetAttr）。
    sandbox
        .execute(
            "var plain = document.getElementById('plain');\
             globalThis.__hidBefore = String(plain.hidden);\
             plain.hidden = true;\
             globalThis.__hidAfter = String(plain.hidden);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hidBefore").unwrap().value,
        "false",
        "hidden 初始=false（无预置）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hidAfter").unwrap().value,
        "true",
        ".hidden=true 后 el.hidden=true（latest-wins getter 反映 SetAttr）"
    );
}

#[test]
fn test_default_checked_selected_independence_r2998() {
    // R2998：defaultChecked（INPUT）/ defaultSelected（OPTION）独立于 .checked= / .selected= dirty 态
    // （spec：.checked=/.selected= 改 dirty 当前态、不改 default*=初始属性）。承接 R2997 记录的限制 ①。
    // R2997 已让 .checked getter latest-wins（.checked=false 移除属性→.checked=false 正确）；但 defaultChecked
    // 旧读纯快照 → setAttribute/removeAttribute 后 stale-wrong；且若朴素改 latest-wins 又会被 .checked= 污染
    // （.checked=false 移除属性→latest-wins defaultChecked=false，错，应保持初始 true）。故须 R2996 类 dirty 缓存：
    // 首次 .checked=/.selected= 前捕获初始属性存在性，setAttribute/removeAttribute/default*=重同步。
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
         <input id='on' type='checkbox' checked>\
         <input id='off' type='checkbox'>\
         <input id='off2' type='checkbox'>\
         <select id='s'>\
         <option id='o1' selected>A</option>\
         <option id='o2'>B</option>\
         </select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ── A：removeAttribute('checked') 后 defaultChecked 应 false（旧 stale 读快照仍 true）。──
    sandbox
        .execute(
            "var on = document.getElementById('on');\
             globalThis.__dcRmBefore = String(on.defaultChecked);\
             on.removeAttribute('checked');\
             globalThis.__dcRmAfter = String(on.defaultChecked);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dcRmBefore").unwrap().value,
        "true",
        "checked 初始 defaultChecked=true"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dcRmAfter").unwrap().value,
        "false",
        "removeAttribute('checked') 后 defaultChecked=false（重同步，修正 stale true）"
    );

    // ── B：setAttribute('checked') 在无预置上后 defaultChecked 应 true（旧 stale 读快照仍 false）。──
    sandbox
        .execute(
            "var off = document.getElementById('off');\
             globalThis.__dcSetBefore = String(off.defaultChecked);\
             off.setAttribute('checked', '');\
             globalThis.__dcSetAfter = String(off.defaultChecked);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dcSetBefore").unwrap().value,
        "false",
        "无预置 checked defaultChecked=false"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dcSetAfter").unwrap().value,
        "true",
        "setAttribute('checked') 后 defaultChecked=true（重同步，修正 stale false）"
    );

    // ── C：.checked= dirty 不污染 defaultChecked（朴素 latest-wins 会在此返 false，错）。──
    //     on 当前已 removeAttribute（见 A），先 setAttribute 复原 checked，再 .checked=false 验捕获。
    sandbox
        .execute(
            "on.setAttribute('checked', '');\
             globalThis.__dcDirtyBefore = String(on.defaultChecked);\
             on.checked = false;\
             globalThis.__dcDirtyAfter = String(on.defaultChecked);\
             globalThis.__ckDirtyAfter = String(on.checked);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dcDirtyBefore").unwrap().value,
        "true",
        ".checked= 前 defaultChecked=true（setAttribute 复原）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dcDirtyAfter").unwrap().value,
        "true",
        ".checked=false 后 defaultChecked 仍 true（dirty 缓存保护，不被属性移除污染）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ckDirtyAfter").unwrap().value,
        "false",
        ".checked=false 后 .checked=false（dirty 当前态，R2997 latest-wins）"
    );

    // ── D：.checked=true 在无预置上 dirty，defaultChecked 保持初始 false（捕获），不跟随属性添加。──
    //     用干净的 off2（未被前序 setAttribute 污染）验「真无预置」捕获。
    sandbox
        .execute(
            "var off2 = document.getElementById('off2');\
             off2.checked = true;\
             globalThis.__dcDirtyOff = String(off2.defaultChecked);\
             globalThis.__ckDirtyOff = String(off2.checked);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dcDirtyOff").unwrap().value,
        "false",
        "无预置 .checked=true 后 defaultChecked 仍 false（捕获初始，不跟随 dirty 添加）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__ckDirtyOff").unwrap().value,
        "true",
        "无预置 .checked=true 后 .checked=true（dirty 当前态）"
    );

    // ── E：.defaultChecked= 显式设默认值（重同步），生效。──
    sandbox
        .execute(
            "off2.defaultChecked = false;\
             globalThis.__dcExplicit = String(off2.defaultChecked);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dcExplicit").unwrap().value,
        "false",
        ".defaultChecked=false 显式设后 defaultChecked=false（重同步生效，清 dirty 回落属性）"
    );

    // ── F：defaultSelected（OPTION）同 pattern——removeAttribute('selected') 后 defaultSelected=false。──
    sandbox
        .execute(
            "var o1 = document.getElementById('o1');\
             globalThis.__dsRmBefore = String(o1.defaultSelected);\
             o1.removeAttribute('selected');\
             globalThis.__dsRmAfter = String(o1.defaultSelected);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dsRmBefore").unwrap().value,
        "true",
        "selected 初始 defaultSelected=true"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dsRmAfter").unwrap().value,
        "false",
        "removeAttribute('selected') 后 defaultSelected=false（重同步，修正 stale true）"
    );

    // ── G：.selected= dirty 不污染 defaultSelected（朴素 latest-wins 会返 false，错）。──
    //     o2 无预置 selected，先 .selected=true（dirty 捕获 false），defaultSelected 应保持 false。
    //     （.selected getter 本身 stale 为 R2997 限制 ② 独立 slice，此处只验 defaultSelected 缓存。）
    sandbox
        .execute(
            "var o2 = document.getElementById('o2');\
             o2.selected = true;\
             globalThis.__dsDirtyOff = String(o2.defaultSelected);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__dsDirtyOff").unwrap().value,
        "false",
        "无预置 .selected=true 后 defaultSelected 仍 false（dirty 缓存保护）"
    );
}

#[test]
fn test_option_selected_getter_latest_wins_r2999() {
    // R2999：option.selected 当前态 getter sel 路径仍读纯快照 __zw_has_attr（R2997 限制 ②）→
    // removeAttribute('selected') / .selected= / setAttribute('selected') 后 stale。本切片闭合（同 R2997 .checked 模式）。
    // .selected 与属性耦合（.selected= set/remove attr，无独立 dirty 态），故 getter latest-wins 与 setter 一致。
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
         <select id='s'>\
         <option id='o1' selected>A</option>\
         <option id='o2'>B</option>\
         </select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // removeAttribute('selected') 后 .selected 应 false（旧 stale 返 true——getter 读快照未反映同批 RemoveAttr）。
    sandbox
        .execute(
            "var o1 = document.getElementById('o1');\
             globalThis.__selRmBefore = String(o1.selected);\
             o1.removeAttribute('selected');\
             globalThis.__selRmAfter = String(o1.selected);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__selRmBefore").unwrap().value,
        "true",
        "option 初始 selected=true（快照）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__selRmAfter").unwrap().value,
        "false",
        "removeAttribute('selected') 后 .selected=false（latest-wins getter，修正 stale true）"
    );

    // setAttribute('selected') 复原后 .selected 应 true（latest-wins 反映同批 SetAttr）。
    sandbox
        .execute(
            "o1.setAttribute('selected', '');\
             globalThis.__selReSet = String(o1.selected);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__selReSet").unwrap().value,
        "true",
        "setAttribute('selected') 后 .selected=true（latest-wins getter 反映同批 SetAttr）"
    );

    // .selected = false 后 .selected 应 false（setter 推 RemoveAttr{selected}，getter latest-wins 反映）。
    sandbox
        .execute(
            "o1.selected = false;\
             globalThis.__selUnset = String(o1.selected);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__selUnset").unwrap().value,
        "false",
        ".selected=false 后 .selected=false（latest-wins getter 反映 RemoveAttr）"
    );

    // 无预置 option：.selected = true 后 .selected 应 true（旧 stale 返 false——快照无 selected）。
    sandbox
        .execute(
            "var o2 = document.getElementById('o2');\
             globalThis.__selOffBefore = String(o2.selected);\
             o2.selected = true;\
             globalThis.__selOffAfter = String(o2.selected);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__selOffBefore").unwrap().value,
        "false",
        "无预置 option 初始 selected=false"
    );
    assert_eq!(
        sandbox.execute("globalThis.__selOffAfter").unwrap().value,
        "true",
        ".selected=true 后 .selected=true（latest-wins getter 反映 SetAttr，修正 stale false）"
    );
}

#[test]
fn test_select_value_programmatic_reflection_r3000() {
    // R3000：`select.value = x` 编程选中须反映到 select.value/selectedIndex getter + option.selected（matched/
    // unmatched）。audit（R2999 限制 ②）：select.value setter 记 SelectOption mutation，但 __zw_select_value/
    // __zw_select_index/option.selected 读路径均不 consult SelectOption → 全 stale。本切片闭合：三读路径 consult
    // latest SelectOption mutation。
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
         <select id='s'>\
         <option id='a' value='A'>A</option>\
         <option id='b' value='B'>B</option>\
         <option id='c' value='C'>C</option>\
         </select>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 初始：默认选首个 option（无 selected 属性）→ value='A', index=0, a.selected=true（snapshot selected_option
    // 返首个）。先验初始态。
    sandbox
        .execute(
            "globalThis.__v0 = document.getElementById('s').value;\
             globalThis.__i0 = String(document.getElementById('s').selectedIndex);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__v0").unwrap().value, "A", "初始 select.value='A'（首个 option）");

    // select.value = 'B' 后四读路径须反映编程选中。
    sandbox
        .execute(
            "var s = document.getElementById('s');\
             s.value = 'B';\
             globalThis.__v1 = s.value;\
             globalThis.__i1 = String(s.selectedIndex);\
             globalThis.__selA = String(document.getElementById('a').selected);\
             globalThis.__selB = String(document.getElementById('b').selected);\
             globalThis.__selC = String(document.getElementById('c').selected);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__v1").unwrap().value,
        "B",
        "select.value='B' 后 select.value='B'（consult SelectOption，旧 stale 'A'）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__i1").unwrap().value,
        "1",
        "select.value='B' 后 selectedIndex=1（consult SelectOption，旧 stale 0）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__selB").unwrap().value,
        "true",
        "select.value='B' 后 optionB.selected=true（consult SelectOption，旧 stale false）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__selA").unwrap().value,
        "false",
        "select.value='B' 后 optionA.selected=false（被 deselect，旧 stale true）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__selC").unwrap().value,
        "false",
        "select.value='B' 后 optionC.selected=false（未匹配）"
    );
}

#[test]
fn test_get_attribute_names_dataset_latest_wins_r3002() {
    // R3002：getAttributeNames / hasAttributes / dataset 旧经 `__zw_attr_names`（纯快照）+ dataset 值/存在性
    // 经 `__zw_get_attr`/`__zw_has_attr`（纯快照）→ 同批 setAttribute/removeAttribute / dataset 设/删 后 stale
    // （R2995 限制 ③，属性反射矩阵最后一项 stale）。data-* 为纯反射属性（无 dirty 态），latest-wins 正确。
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
        "<html><body><div id='d' data-existing='e' class='c'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // setAttribute('data-new','x') 后 getAttributeNames 应含 'data-new'（旧 stale 缺）；dataset.new === 'x'。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             d.setAttribute('data-new', 'x');\
             globalThis.__names1 = d.getAttributeNames().join(',');\
             globalThis.__dv1 = d.dataset.new;\
             globalThis.__hasNew1 = String('new' in d.dataset);",
        )
        .unwrap();
    assert!(
        sandbox.execute("globalThis.__names1").unwrap().value.contains("data-new"),
        "setAttribute('data-new') 后 getAttributeNames 含 data-new（旧 stale 缺）got: {}",
        sandbox.execute("globalThis.__names1").unwrap().value
    );
    assert_eq!(
        sandbox.execute("globalThis.__dv1").unwrap().value,
        "x",
        "setAttribute('data-new','x') 后 dataset.new='x'（旧 stale undefined）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasNew1").unwrap().value,
        "true",
        "setAttribute('data-new') 后 'new' in dataset=true（旧 stale false）"
    );

    // removeAttribute('data-existing') 后 getAttributeNames 不含 'data-existing'；dataset.existing===undefined。
    sandbox
        .execute(
            "globalThis.__names0 = d.getAttributeNames().join(',');\
             globalThis.__dv0 = String(d.dataset.existing);\
             d.removeAttribute('data-existing');\
             globalThis.__names2 = d.getAttributeNames().join(',');\
             globalThis.__dv2 = String(d.dataset.existing);\
             globalThis.__hasExisting2 = String('existing' in d.dataset);",
        )
        .unwrap();
    assert!(
        sandbox.execute("globalThis.__names0").unwrap().value.contains("data-existing"),
        "removeAttribute 前 getAttributeNames 含 data-existing"
    );
    assert!(
        !sandbox.execute("globalThis.__names2").unwrap().value.contains("data-existing"),
        "removeAttribute('data-existing') 后 getAttributeNames 不含 data-existing（旧 stale 仍含）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dv2").unwrap().value,
        "undefined",
        "removeAttribute('data-existing') 后 dataset.existing=undefined（旧 stale 'e'）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__hasExisting2").unwrap().value,
        "false",
        "removeAttribute('data-existing') 后 'existing' in dataset=false（旧 stale true）"
    );

    // dataset.assigned = 'y'（setter 推 SetAttr{data-assigned}）后 getAttributeNames 含 'data-assigned'；
    // dataset.assigned === 'y'。
    sandbox
        .execute(
            "d.dataset.assigned = 'y';\
             globalThis.__names3 = d.getAttributeNames().join(',');\
             globalThis.__dv3 = d.dataset.assigned;",
        )
        .unwrap();
    assert!(
        sandbox.execute("globalThis.__names3").unwrap().value.contains("data-assigned"),
        "dataset.assigned='y' 后 getAttributeNames 含 data-assigned（旧 stale 缺）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dv3").unwrap().value,
        "y",
        "dataset.assigned='y' 后 dataset.assigned='y'（旧 stale undefined）"
    );

    // delete dataset.assigned（推 RemoveAttr{data-assigned}）后 getAttributeNames 不含；dataset.assigned===undefined。
    sandbox
        .execute(
            "delete d.dataset.assigned;\
             globalThis.__names4 = d.getAttributeNames().join(',');\
             globalThis.__dv4 = String(d.dataset.assigned);",
        )
        .unwrap();
    assert!(
        !sandbox.execute("globalThis.__names4").unwrap().value.contains("data-assigned"),
        "delete dataset.assigned 后 getAttributeNames 不含 data-assigned（旧 stale 仍含）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__dv4").unwrap().value,
        "undefined",
        "delete dataset.assigned 后 dataset.assigned=undefined（旧 stale 'y'）"
    );

    // hasAttributes() 始终 true（div 有 class/data-existing 残留或新增）；验非空元素返 true。
    sandbox.execute("globalThis.__hasAttrs = String(d.hasAttributes());").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hasAttrs").unwrap().value,
        "true",
        "hasAttributes()=true（div 有属性）"
    );
}
