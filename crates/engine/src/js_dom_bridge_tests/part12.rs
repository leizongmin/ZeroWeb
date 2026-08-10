// R3006+ 新测试（part01-11 满载后扩展，控制单文件 <2000 行）。经 js_dom_bridge_tests.rs include。

#[test]
fn test_location_hash_setter_hashchange_r3006() {
    // R3006：`location.hash = v` 须更新 hash + 新 history entry + 派发 hashchange（SPA hash 路由核心）。
    // 旧 _makeLocation 仅 getter（无 setter）→ location.hash = '#foo' 静默 no-op，hash router 全失效。
    // HashChangeEvent（R2812）已有 oldURL/newURL 字段，本切片补 setter + 派发。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/path".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 安装 hashchange listener 捕获 newURL/oldURL。
    sandbox
        .execute(
            "globalThis.__hc = null;\
             addEventListener('hashchange', function(e){ globalThis.__hc = { newURL: e.newURL, oldURL: e.oldURL }; });",
        )
        .unwrap();

    // location.hash = '#foo'：hash 更新 + history entry + hashchange 派发（_defer 异步）。
    sandbox.execute("location.hash = '#foo';").unwrap();
    sandbox
        .execute(
            "globalThis.__h1 = location.hash;\
             globalThis.__href1 = location.href;\
             globalThis.__hcNew1 = globalThis.__hc ? globalThis.__hc.newURL : '(none)';\
             globalThis.__hcOld1 = globalThis.__hc ? globalThis.__hc.oldURL : '(none)';",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__h1").unwrap().value, "#foo", "location.hash='#foo' 后 hash='#foo'");
    assert_eq!(sandbox.execute("globalThis.__href1").unwrap().value, "https://example.com/path#foo", "location.href 含 #foo");
    assert_eq!(sandbox.execute("globalThis.__hcNew1").unwrap().value, "https://example.com/path#foo", "hashchange.newURL 含 #foo");
    assert_eq!(sandbox.execute("globalThis.__hcOld1").unwrap().value, "https://example.com/path", "hashchange.oldURL 无 hash");

    // 无 '#' 前缀：spec 自动补 '#'。
    sandbox
        .execute(
            "globalThis.__hc = null;\
             location.hash = 'bar';\
             globalThis.__h2 = location.hash;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__h2").unwrap().value, "#bar", "location.hash='bar'（无#）spec 自动补 → '#bar'");
    sandbox.execute("globalThis.__hcNew2 = globalThis.__hc ? globalThis.__hc.newURL : '(none)';").unwrap();
    assert_eq!(sandbox.execute("globalThis.__hcNew2").unwrap().value, "https://example.com/path#bar", "hashchange.newURL='#bar'");

    // 设同 hash：no-op（不派 hashchange，spec）。
    sandbox
        .execute(
            "globalThis.__hc = null;\
             location.hash = '#bar';\
             globalThis.__hcSame = globalThis.__hc;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__hcSame)").unwrap().value, "null", "设同 hash no-op（不派 hashchange）");

    // history.length 反映 hash entry 累积（初始 1 + #foo + #bar = 3）。
    sandbox.execute("globalThis.__len = history.length;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len)").unwrap().value, "3", "hash setter 推 history entry：length=3");

    // back() 回 #foo：location.hash 反映（R3005 location 读 history entry url）。
    sandbox.execute("history.back(); globalThis.__hBack = location.hash;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__hBack").unwrap().value, "#foo", "back() 后 location.hash='#foo'（history entry 反映）");
}

#[test]
fn test_back_forward_hashchange_r3007() {
    // R3007：back/forward/go 跨 hash entry 须同时派 hashchange（spec：hash 变更的导航派 popstate + hashchange）。
    // 旧 _hist_dispatchPopState 仅派 popstate → hash router 的 back 按钮处理失效。本切片闭合。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/path".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 建 hash 序列：#foo → #bar（cursor 在 #bar）。装 hashchange + popstate listener。
    sandbox
        .execute(
            "location.hash = '#foo';\
             location.hash = '#bar';\
             globalThis.__hc = null; globalThis.__popFired = false;\
             addEventListener('hashchange', function(e){ globalThis.__hc = { newURL: e.newURL, oldURL: e.oldURL }; });\
             addEventListener('popstate', function(){ globalThis.__popFired = true; });",
        )
        .unwrap();

    // back()：cursor 回 #foo，须派 hashchange（newURL 含 #foo）+ popstate。
    sandbox.execute("history.back();").unwrap();
    sandbox
        .execute(
            "globalThis.__hcNew = globalThis.__hc ? globalThis.__hc.newURL : '(none)';\
             globalThis.__hcOld = globalThis.__hc ? globalThis.__hc.oldURL : '(none)';",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__hcNew").unwrap().value, "https://example.com/path#foo", "back() 跨 hash 派 hashchange.newURL='#foo'");
    assert_eq!(sandbox.execute("globalThis.__hcOld").unwrap().value, "https://example.com/path#bar", "hashchange.oldURL='#bar'（back 前 entry）");
    assert_eq!(sandbox.execute("String(globalThis.__popFired)").unwrap().value, "true", "back() 同时派 popstate");

    // forward()：cursor 前进回 #bar，须派 hashchange（newURL #bar）。
    sandbox.execute("globalThis.__hc = null; history.forward();").unwrap();
    sandbox.execute("globalThis.__hcFwd = globalThis.__hc ? globalThis.__hc.newURL : '(none)';").unwrap();
    assert_eq!(sandbox.execute("globalThis.__hcFwd").unwrap().value, "https://example.com/path#bar", "forward() 跨 hash 派 hashchange.newURL='#bar'");

    // 跨非 hash 变更的 back（两 entry 均无 hash）不应派 hashchange。
    sandbox
        .execute(
            "history.pushState({}, '', '/p1');\
             history.pushState({}, '', '/p2');\
             globalThis.__hc = null;\
             history.back();",
        )
        .unwrap();
    sandbox.execute("globalThis.__hcNoHash = globalThis.__hc;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__hcNoHash)").unwrap().value, "null", "跨非 hash 变更的 back 不派 hashchange（两 entry 均无 hash）");
}

#[test]
fn test_location_part_setters_r3008() {
    // R3008：location.pathname/search/href 旧无 setter（静默 no-op，仅 hash 有 setter per R3006）→ URL 变更路由
    // / 测试场景失效。补 setter：经 URL part setter 计算新 href，push history entry（navigation 语义，R3005 location 读之反映），
    // hash 变化时派 hashchange（与 hash setter / back-forward 对称）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/old?q=1".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // location.pathname = '/new'：pathname 替换，search/hash 保留。
    sandbox
        .execute(
            "globalThis.__p0 = location.pathname;\
             location.pathname = '/new';\
             globalThis.__p1 = location.pathname; globalThis.__href1 = location.href; globalThis.__s1 = location.search;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__p0").unwrap().value, "/old", "初始 pathname=/old");
    assert_eq!(sandbox.execute("globalThis.__p1").unwrap().value, "/new", "location.pathname='/new' 后 pathname=/new");
    assert_eq!(sandbox.execute("globalThis.__s1").unwrap().value, "?q=1", "pathname 替换保留 search=?q=1");
    assert_eq!(sandbox.execute("globalThis.__href1").unwrap().value, "https://example.com/new?q=1", "href 反映新 pathname");

    // location.search = '?q=2'：search 替换。
    sandbox.execute("location.search = '?q=2'; globalThis.__s2 = location.search; globalThis.__href2 = location.href;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__s2").unwrap().value, "?q=2", "location.search='?q=2' 后 search=?q=2");
    assert_eq!(sandbox.execute("globalThis.__href2").unwrap().value, "https://example.com/new?q=2", "href 反映新 search");

    // location.href = 绝对 URL（含 hash）：整体替换 + hash 变化派 hashchange。
    sandbox
        .execute(
            "globalThis.__hc = null;\
             addEventListener('hashchange', function(e){ globalThis.__hc = e.newURL; });\
             location.href = 'https://example.com/full#h';",
        )
        .unwrap();
    sandbox
        .execute(
            "globalThis.__href3 = location.href; globalThis.__hash3 = location.hash;\
             globalThis.__hc3 = globalThis.__hc;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__href3").unwrap().value, "https://example.com/full#h", "location.href 绝对 URL 整体替换");
    assert_eq!(sandbox.execute("globalThis.__hash3").unwrap().value, "#h", "hash='#h'");
    assert_eq!(sandbox.execute("globalThis.__hc3").unwrap().value, "https://example.com/full#h", "href 引入 hash 变化派 hashchange.newURL");

    // history.length 反映导航 entry 累积（初始 1 + pathname + search + href = 4）。
    sandbox.execute("globalThis.__len = history.length;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len)").unwrap().value, "4", "location setter 推 history entry：length=4");

    // 设同值 no-op（不增 entry）。
    sandbox.execute("location.href = 'https://example.com/full#h'; globalThis.__len2 = history.length;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len2)").unwrap().value, "4", "设同 href no-op（不增 entry）");
}

#[test]
fn test_location_assign_replace_reload_r3009() {
    // R3009：location.assign/replace/reload 旧为静默 no-op stub → redirect / 基于 location 的导航模式失效。
    // spec：assign(url) 功能等价 location.href = url（MDN）——resolve url + push history entry + location 反映
    // + hash 变化派 hashchange；replace(url) replace 当前 entry（back 不回旧 url）+ hash 变化派 hashchange；
    // reload() headless 无真文档重载 no-op（synthesized page 无原始 fetch 可重取）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/start".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 初始 location.href = page_url（_hist_current().url 为空 → 回落 __zw_get_page_url）。
    sandbox.execute("globalThis.__h0 = location.href;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__h0").unwrap().value, "https://example.com/start", "初始 location.href=page_url");

    // assign('/a')：相对 URL 解析为绝对 + push entry + location 反映。
    sandbox
        .execute("location.assign('/a'); globalThis.__h1 = location.href; globalThis.__p1 = location.pathname;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__h1").unwrap().value, "https://example.com/a", "assign('/a') 相对 URL resolve + location.href 反映");
    assert_eq!(sandbox.execute("globalThis.__p1").unwrap().value, "/a", "assign('/a') pathname=/a");

    // 装 hashchange listener，assign('#sec')：hash 变化 push entry + 派 hashchange。
    sandbox
        .execute("globalThis.__hc = null; addEventListener('hashchange', function(e){ globalThis.__hc = e.newURL; });")
        .unwrap();
    sandbox.execute("location.assign('#sec'); globalThis.__h2 = location.href;").unwrap();
    // hashchange 经 _defer microtask 在 execute 末尾派发 → 下一次 execute 读到。
    sandbox.execute("globalThis.__hc2 = globalThis.__hc;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__h2").unwrap().value, "https://example.com/a#sec", "assign('#sec') href 反映新 hash");
    assert_eq!(sandbox.execute("globalThis.__hc2").unwrap().value, "https://example.com/a#sec", "assign hash 变化派 hashchange.newURL");

    // history.length 反映 assign 累积（初始 1 + /a + #sec = 3）。
    sandbox.execute("globalThis.__len = history.length;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len)").unwrap().value, "3", "assign 推 history entry：length=3");

    // replace(绝对 URL)：replace 当前 entry（不增 length）+ location 反映。
    sandbox
        .execute("location.replace('https://example.com/final'); globalThis.__h3 = location.href; globalThis.__len3 = history.length;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__h3").unwrap().value, "https://example.com/final", "replace(url) location.href 反映");
    assert_eq!(sandbox.execute("String(globalThis.__len3)").unwrap().value, "3", "replace 替换当前 entry 不增 length（替换 #sec）：length=3");

    // replace 后 back 不回 #sec（被替换）→ 回 /a。
    sandbox.execute("history.back(); globalThis.__hBack = location.href;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__hBack").unwrap().value, "https://example.com/a", "replace 后 back 回 /a（#sec entry 被替换）");

    // assign 同当前 url no-op（不增 entry）。
    sandbox
        .execute("history.forward(); globalThis.__len4 = history.length; location.assign('https://example.com/final'); globalThis.__len5 = history.length;")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len5)").unwrap().value, "3", "assign 同当前 url no-op（不增 entry）");

    // reload()：headless no-op（不抛，location 不变）。
    sandbox
        .execute("globalThis.__err = 'none'; try { location.reload(); } catch(e){ globalThis.__err = String(e); } globalThis.__hRel = location.href;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__err").unwrap().value, "none", "reload() 不抛");
    assert_eq!(sandbox.execute("globalThis.__hRel").unwrap().value, "https://example.com/final", "reload() headless no-op，location 不变");
}

#[test]
fn test_streams_backpressure_r3010() {
    // R3010：Streams 背压 spec 化。旧 desiredSize 恒 1 / ready 立即 resolve（无 highWaterMark 追踪）→
    // 流控库（按 desiredSize 节流 / await writer.ready）失效。spec：desiredSize = highWaterMark - queueTotalSize，
    // controller/writer.desiredSize 反映队列压力；writer.ready 在 desiredSize<=0 挂起、>0 resolve（背压门控）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ReadableStream controller.desiredSize：默认 hwm=1，enqueue 一 chunk 后 0（旧恒 1）。
    sandbox
        .execute(
            "globalThis.__rc = null;\
             new ReadableStream({ start: function(c){ globalThis.__rc = c; } });\
             globalThis.__ds0 = globalThis.__rc.desiredSize;\
             globalThis.__rc.enqueue('x');\
             globalThis.__ds1 = globalThis.__rc.desiredSize;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ds0)").unwrap().value, "1", "ReadableStream 默认 hwm=1 初始 desiredSize=1");
    assert_eq!(sandbox.execute("String(globalThis.__ds1)").unwrap().value, "0", "enqueue 1 chunk 后 desiredSize=hwm-queueTotalSize=0");

    // 自定义 highWaterMark + size 函数（byte 计量）：hwm=10，enqueue 'hello'（size 5）→ desiredSize=5。
    sandbox
        .execute(
            "globalThis.__rc2 = null;\
             new ReadableStream({ start: function(c){ globalThis.__rc2 = c; } }, { highWaterMark: 10, size: function(c){ return c.length; } });\
             globalThis.__dsc0 = globalThis.__rc2.desiredSize;\
             globalThis.__rc2.enqueue('hello');\
             globalThis.__dsc1 = globalThis.__rc2.desiredSize;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__dsc0)").unwrap().value, "10", "自定义 hwm=10 初始 desiredSize=10");
    assert_eq!(sandbox.execute("String(globalThis.__dsc1)").unwrap().value, "5", "enqueue 'hello'(size 5) 后 desiredSize=10-5=5");

    // ReadableStream close → desiredSize=0；read 后 desiredSize 回升（drain 释放余量）。
    sandbox
        .execute(
            "globalThis.__rc3 = null;\
             var rs3 = new ReadableStream({ start: function(c){ globalThis.__rc3 = c; c.enqueue('a'); } });\
             globalThis.__dsBeforeRead = globalThis.__rc3.desiredSize;\
             globalThis.__rc3.close(); globalThis.__dsClose = globalThis.__rc3.desiredSize;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__dsBeforeRead)").unwrap().value, "0", "enqueue 1 chunk 后 desiredSize=0（背压）");
    assert_eq!(sandbox.execute("String(globalThis.__dsClose)").unwrap().value, "0", "close 后 desiredSize=0");

    // WritableStream desiredSize = hwm - queueTotalSize：hwm=2，写前 2、写 'a' 后 1、写 'b' 后 0。
    sandbox
        .execute(
            "globalThis.__log = [];\
             globalThis.__ws = new WritableStream({ write: function(c){ globalThis.__log.push(c); } }, { highWaterMark: 2 });\
             globalThis.__w = globalThis.__ws.getWriter();\
             globalThis.__wds0 = globalThis.__w.desiredSize;\
             globalThis.__w.write('a'); globalThis.__wds1 = globalThis.__w.desiredSize;\
             globalThis.__w.write('b'); globalThis.__wds2 = globalThis.__w.desiredSize;\
             globalThis.__w.ready.then(function(){ globalThis.__readyFired = 'drained'; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__wds0)").unwrap().value, "2", "WritableStream hwm=2 初始 desiredSize=2");
    assert_eq!(sandbox.execute("String(globalThis.__wds1)").unwrap().value, "1", "写 'a' 后 desiredSize=2-1=1");
    assert_eq!(sandbox.execute("String(globalThis.__wds2)").unwrap().value, "0", "写 'b' 后 desiredSize=2-2=0（背压）");
    // 同步 sink：microtask checkpoint drain 后 queueTotalSize 归零、desiredSize 回 hwm、ready resolve。
    sandbox.execute("globalThis.__wdsAfter = globalThis.__w.desiredSize;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__wdsAfter)").unwrap().value, "2", "同步 sink drain 后 desiredSize 回 hwm=2");
    assert_eq!(sandbox.execute("String(globalThis.__readyFired)").unwrap().value, "drained", "ready 在 desiredSize>0 后 resolve（背压释放）");

    // 异步 sink + ready 背压门控：写超 hwm 后 desiredSize<=0（背压），手控 resolver 释放后 ready resolve。
    sandbox
        .execute(
            "globalThis.__defer = null;\
             globalThis.__ws2 = new WritableStream({\
               write: function(c){ return new Promise(function(res){ globalThis.__defer = res; }); }\
             }, { highWaterMark: 1 });\
             globalThis.__w2 = globalThis.__ws2.getWriter();\
             globalThis.__w2.write('x');\
             globalThis.__bp = (globalThis.__w2.desiredSize <= 0) ? 'yes' : 'no';\
             globalThis.__w2.ready.then(function(){ globalThis.__readyFired2 = 'resumed'; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__bp)").unwrap().value, "yes", "异步 sink 写超 hwm 后 desiredSize<=0（背压挂起）");
    assert_eq!(sandbox.execute("String(globalThis.__readyFired2 === undefined)").unwrap().value, "true", "背压态 ready 挂起（未 resolve）");
    // 释放 pending write → 背压解除 → ready resolve（_defer microtask 在 execute 末尾 drain）。
    sandbox.execute("globalThis.__defer();").unwrap();
    sandbox.execute("globalThis.__rf2 = globalThis.__readyFired2;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__rf2)").unwrap().value, "resumed", "pending write 完成释放背压 → ready resolve");
}

#[test]
fn test_blob_real_bytes_r3011() {
    // R3011：Blob 真字节级物化。旧 slice() 浅拷 _parts + clamp size（slice().text() 返**全**内容非字节范围）；
    // arrayBuffer()/stream() 经 text() UTF-8 往返——**二进制 TypedArray part 被 UTF-8 解码-再编码损坏**。
    // 本切片：同步 _zw_partBytes 字节拼接，slice 返真字节范围、arrayBuffer/stream 返真字节（二进制不损）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // slice()：返真字节范围（旧返全内容）。new Blob(['ZeroWeb']).slice(1,4).text() === 'ero'。
    sandbox
        .execute("globalThis.__sl = '(pending)'; new Blob(['ZeroWeb']).slice(1,4).text().then(function(s){ globalThis.__sl = s; });")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__sl)").unwrap().value, "ero", "slice(1,4) 返真字节范围 'ero'（旧返全 'ZeroWeb'）");
    // slice().size 仍 clamp（既有断言不破）。
    assert_eq!(sandbox.execute("new Blob(['ZeroWeb']).slice(1,4).size").unwrap().value, "3", "slice size clamp 保留");

    // slice 跨 part 边界：['abc','def']（6 字节）slice(2,5) = 'cde'（跨 abc|def）。
    sandbox
        .execute("globalThis.__sl2 = '(pending)'; new Blob(['abc','def']).slice(2,5).text().then(function(s){ globalThis.__sl2 = s; });")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__sl2)").unwrap().value, "cde", "slice 跨 part 边界返 'cde'");

    // arrayBuffer() 二进制保真：TypedArray 含非 UTF-8 字节（0xff 0xfe 0x00）→ 原样返回（旧经 UTF-8 往返损坏）。
    sandbox
        .execute(
            "globalThis.__bin = '(pending)';\
             new Blob([new Uint8Array([0xff,0xfe,0x00,0x41])]).arrayBuffer()\
               .then(function(a){ globalThis.__bin = a[0] + ',' + a[1] + ',' + a[2] + ',' + a[3]; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__bin)").unwrap().value, "255,254,0,65", "arrayBuffer 二进制保真（0xff 0xfe 0x00 0x41，旧 UTF-8 往返损坏）");

    // string arrayBuffer 不变（'AB' → [65,66]，既有行为）。
    sandbox
        .execute("globalThis.__ab2 = '(pending)'; new Blob(['AB']).arrayBuffer().then(function(a){ globalThis.__ab2 = a.length + ':' + a[0] + ',' + a[1]; });")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ab2)").unwrap().value, "2:65,66", "string arrayBuffer 不变（'AB'→[65,66]）");

    // stream() 二进制保真：字节经 reader 读出原样（不 UTF-8 往返）。
    sandbox
        .execute(
            "globalThis.__stbin = '(pending)';\
             var st = new Blob([new Uint8Array([0x01,0x02,0xff])]).stream();\
             var rd = st.getReader();\
             rd.read().then(function(c){ globalThis.__stbin = c.value[0] + ',' + c.value[1] + ',' + c.value[2]; });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__stbin)").unwrap().value, "1,2,255", "stream 二进制保真（0x01 0x02 0xff 原样）");

    // File 继承 slice 真字节：File(['ZeroWeb']).slice(1,4).text() === 'ero'。
    sandbox
        .execute("globalThis.__fsl = '(pending)'; new File(['ZeroWeb'],'f').slice(1,4).text().then(function(s){ globalThis.__fsl = s; });")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__fsl)").unwrap().value, "ero", "File 继承 slice 真字节 'ero'");
}

#[test]
fn test_text_decoder_streaming_r3012() {
    // R3012：TextDecoder 流式跨 chunk 多字节状态。旧 decode 每 chunk 独立解码（无 carry）→ 多字节 char 跨 chunk
    // 边界切断时损坏（残余字节被独立解码为 U+FFFD/垃圾）。且 _zw_utf8_decode 读越界（truncated 序列读 undefined）。
    // 本切片：_zw_utf8_decode_stream 返回 {s, tail}（不完整尾部缓存），decode({stream:true}) 跨调用 carry + flush。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 单次 decode（valid）行为不变：'ZeroWeb 中文' round-trip 保真。
    assert_eq!(
        sandbox
            .execute("var d0 = new TextDecoder(); d0.decode(new TextEncoder().encode('ZeroWeb 中文'))")
            .unwrap()
            .value,
        "ZeroWeb 中文",
        "单次 decode round-trip 保真（valid 输入行为不变）"
    );

    // stream:true 跨 chunk 重组：'中' = [0xe4,0xb8,0xad] 拆 [0xe4,0xb8] + [0xad]。
    sandbox
        .execute(
            "var d = new TextDecoder();\
             globalThis.__part1 = d.decode(new Uint8Array([0xe4,0xb8]), { stream: true });\
             globalThis.__part2 = d.decode(new Uint8Array([0xad]), { stream: true });\
             globalThis.__flush = d.decode();",
        )
        .unwrap();
    assert_eq!(sandbox.execute("JSON.stringify(globalThis.__part1)").unwrap().value, "\"\"", "首 chunk 不完整 → 空串（缓存尾部）");
    assert_eq!(sandbox.execute("JSON.stringify(globalThis.__part2)").unwrap().value, "\"中\"", "次 chunk 补全 → '中'（跨 chunk 重组）");
    assert_eq!(sandbox.execute("JSON.stringify(globalThis.__flush)").unwrap().value, "\"\"", "flush 无残余 → 空串");

    // astral（4 字节）跨 chunk：'🌍' = U+1F30D → [0xf0,0x9f,0x8c,0x8d]，拆 1+3。
    sandbox
        .execute(
            "var d2 = new TextDecoder();\
             globalThis.__a1 = d2.decode(new Uint8Array([0xf0,0x9f]), { stream: true });\
             globalThis.__a2 = d2.decode(new Uint8Array([0x8c,0x8d]), { stream: true });",
        )
        .unwrap();
    assert_eq!(sandbox.execute("JSON.stringify(globalThis.__a1)").unwrap().value, "\"\"", "astral 首 chunk 不完整 → 空");
    assert_eq!(sandbox.execute("JSON.stringify(globalThis.__a2)").unwrap().value, "\"🌍\"", "astral 次 chunk 补全 → '🌍'（4 字节跨 chunk 重组）");

    // stream:false（缺省）flush：truncated 尾部 → U+FFFD（旧读越界产垃圾）。
    sandbox.execute("globalThis.__trunc = new TextDecoder().decode(new Uint8Array([0xe4,0xb8])); globalThis.__truncLen = globalThis.__trunc.length; globalThis.__truncCode = globalThis.__trunc.charCodeAt(0);").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__truncLen)").unwrap().value, "1", "truncated 单次 decode → 1 char");
    assert_eq!(sandbox.execute("String(globalThis.__truncCode)").unwrap().value, "65533", "truncated 单次 decode → U+FFFD（0xFFFD=65533，flush 容错）");

    // TextDecoderStream 跨 chunk 重组：分两 write chunk 写入拆分的 '中文' 字节。
    sandbox
        .execute(
            "var all = new TextEncoder().encode('中文');\
             var first = all.slice(0, 4), second = all.slice(4);\
             var tds = new TextDecoderStream();\
             var wd = tds.writable.getWriter(); wd.write(first); wd.write(second); wd.close();\
             var rd = tds.readable.getReader();\
             globalThis.__chunks = [];\
             (function pump(){ rd.read().then(function(c){ if(c.done){ return; } globalThis.__chunks.push(c.value); pump(); }); })();",
        )
        .unwrap();
    sandbox.execute("globalThis.__tdsJoined = globalThis.__chunks.join('');").unwrap();
    assert_eq!(sandbox.execute("JSON.stringify(globalThis.__tdsJoined)").unwrap().value, "\"中文\"", "TextDecoderStream 跨 chunk 重组 '中文'（旧各 chunk 独立解码损坏）");
}

#[test]
fn test_detached_document_query_r3013() {
    // R3013：document.implementation.createHTMLDocument 旧返 hollow doc（querySelector 恒 null、body 无 innerHTML
    // setter）→ jQuery `$.parseHTML` / DOMPurify feature-detect / 模板引擎「detached 解析后查询」模式失效。
    // 本切片：body 经 __zw_parse_html_query 支持可写可查（innerHTML setter + querySelector 族）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 既有行为保留：body.tagName=BODY + title 透传（R2815 断言不破）。
    sandbox
        .execute(
            "var doc = document.implementation.createHTMLDocument('T');\
             globalThis.__bodyTag = doc.body.tagName;\
             globalThis.__title = doc.title;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__bodyTag)").unwrap().value, "BODY", "createHTMLDocument body.tagName=BODY（保留）");
    assert_eq!(sandbox.execute("String(globalThis.__title)").unwrap().value, "T", "createHTMLDocument title 透传（保留）");

    // body.innerHTML setter：存解析源；getter 返存值。
    sandbox.execute("doc.body.innerHTML = '<div id=\"a\">A</div><div class=\"x\">X</div><span class=\"b\">B</span>';").unwrap();
    assert_eq!(sandbox.execute("String(doc.body.innerHTML)").unwrap().value, "<div id=\"a\">A</div><div class=\"x\">X</div><span class=\"b\">B</span>", "body.innerHTML getter 返存值");

    // body.querySelector：id / class 选择器查解析树。
    sandbox
        .execute(
            "globalThis.__qa = doc.body.querySelector('#a');\
             globalThis.__qx = doc.body.querySelector('.x');\
             globalThis.__qb = doc.body.querySelector('.b');\
             globalThis.__qnone = doc.body.querySelector('.nope');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__qa && globalThis.__qa.textContent)").unwrap().value, "A", "body.querySelector('#a').textContent='A'");
    assert_eq!(sandbox.execute("String(globalThis.__qx && globalThis.__qx.tagName)").unwrap().value, "DIV", "body.querySelector('.x').tagName=DIV");
    assert_eq!(sandbox.execute("String(globalThis.__qb && globalThis.__qb.tagName)").unwrap().value, "SPAN", "body.querySelector('.b').tagName=SPAN");
    assert_eq!(sandbox.execute("String(globalThis.__qnone)").unwrap().value, "null", "body.querySelector 无匹配 → null");

    // body.querySelectorAll：多元素集合（div×2 / span×1）。
    assert_eq!(sandbox.execute("String(doc.body.querySelectorAll('div').length)").unwrap().value, "2", "body.querySelectorAll('div').length=2");
    assert_eq!(sandbox.execute("String(doc.body.querySelectorAll('span').length)").unwrap().value, "1", "body.querySelectorAll('span').length=1");

    // body.getElementById / getElementsByTagName / getElementsByClassName。
    assert_eq!(sandbox.execute("String(doc.body.getElementById('a').tagName)").unwrap().value, "DIV", "body.getElementById('a').tagName=DIV");
    assert_eq!(sandbox.execute("String(doc.body.getElementsByTagName('span').length)").unwrap().value, "1", "body.getElementsByTagName('span').length=1");
    assert_eq!(sandbox.execute("String(doc.body.getElementsByClassName('x').length)").unwrap().value, "1", "body.getElementsByClassName('x').length=1");

    // document 级 query 委托 body（同样解析树）。
    assert_eq!(sandbox.execute("String(doc.querySelector('#a').textContent)").unwrap().value, "A", "doc.querySelector('#a') 委托 body 解析树");
    assert_eq!(sandbox.execute("String(doc.querySelectorAll('div').length)").unwrap().value, "2", "doc.querySelectorAll('div').length=2");

    // createElement + createTextNode 仍可（feature-detection 常用）。
    sandbox.execute("globalThis.__el = doc.createElement('section'); globalThis.__tn = doc.createTextNode('hi');").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__el.tagName)").unwrap().value, "SECTION", "doc.createElement('section').tagName=SECTION");
    assert_eq!(sandbox.execute("String(globalThis.__tn.nodeValue)").unwrap().value, "hi", "doc.createTextNode('hi').nodeValue='hi'");
}

#[test]
fn test_form_data_multipart_r3014() {
    // R3014：FormData multipart 序列化 + Blob/File 值保真 + fetch FormData body 接线。旧 append/set 经
    // String(value) 归一（Blob/File 被字符串化），且无 multipart 序列化 + fetch POST FormData body 为
    // '[object FormData]' → 表单提交 / 文件上传链断。本切片闭合（Blob/File→FormData→multipart→fetch POST）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 值保真：append(File) 后 get 返 File 实例（旧返 String(file)）。字符串值不变（R2788 保留）。
    sandbox
        .execute(
            "var fd = new FormData();\
             fd.append('name', 'Alice');\
             fd.append('file', new File(['hello'], 'g.txt', { type: 'text/plain' }));\
             globalThis.__nameVal = fd.get('name');\
             globalThis.__fileIsFile = (fd.get('file') instanceof File);\
             globalThis.__fileName = fd.get('file').name;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__nameVal)").unwrap().value, "Alice", "字符串值 get 不变（R2788 保留）");
    assert_eq!(sandbox.execute("String(globalThis.__fileIsFile)").unwrap().value, "true", "append(File) 后 get 返 File 实例（旧返字符串）");
    assert_eq!(sandbox.execute("String(globalThis.__fileName)").unwrap().value, "g.txt", "保真的 File 保留 name");

    // _zwMultipart() 序列化：boundary 在 contentType；body 含 Content-Disposition（name + filename）+
    // Content-Type（file）+ 值；以 boundary 起/收。
    sandbox
        .execute(
            "var mp = fd._zwMultipart();\
             globalThis.__ct = mp.contentType;\
             globalThis.__boundary = mp.contentType.split('boundary=')[1];\
             globalThis.__text = new TextDecoder().decode(mp.body);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ct.indexOf('multipart/form-data; boundary=') === 0)").unwrap().value,
        "true",
        "contentType = multipart/form-data; boundary=..."
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__text.indexOf('--' + globalThis.__boundary + '\\r\\n') === 0)").unwrap().value,
        "true",
        "multipart body 以 --boundary\\r\\n 起"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__text.indexOf(globalThis.__boundary + '--') > 0)").unwrap().value,
        "true",
        "multipart body 以 boundary-- 收"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__text.indexOf('Content-Disposition: form-data; name=\"name\"') >= 0)")
            .unwrap()
            .value,
        "true",
        "body 含字符串字段的 Content-Disposition"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__text.indexOf('Content-Disposition: form-data; name=\"file\"; filename=\"g.txt\"') >= 0)")
            .unwrap()
            .value,
        "true",
        "body 含文件字段的 Content-Disposition + filename"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__text.indexOf('Content-Type: text/plain') >= 0)").unwrap().value,
        "true",
        "body 含文件字段 Content-Type（Blob.type）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__text.indexOf('Alice') > 0 && globalThis.__text.indexOf('hello') > 0)").unwrap().value,
        "true",
        "body 含字段值（Alice + hello）"
    );

    // fetch FormData body 接线：mock __zw_fetch 捕获 method/headersWire/body，验 Content-Type multipart + body multipart 标记。
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let cap = Arc::clone(&captured);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let mut g = cap.lock().unwrap();
            g.clear();
            for a in args.iter() {
                g.push(a.clone());
            }
            "ok".to_string()
        }),
    );
    sandbox
        .execute(
            "var fd2 = new FormData();\
             fd2.append('q', 'search');\
             fetch('http://test.local/upload', { method: 'POST', body: fd2 });",
        )
        .unwrap();
    let cap_guard = captured.lock().unwrap();
    // args = [id, method, url, headersWire, body]
    assert!(cap_guard.len() >= 5, "__zw_fetch 须收到 5 args（id/method/url/headers/body）");
    assert_eq!(cap_guard[1], "POST", "fetch FormData body → method=POST");
    let headers_wire = cap_guard[3].to_lowercase();
    assert!(
        headers_wire.contains("content-type") && headers_wire.contains("multipart/form-data"),
        "fetch FormData → headers 含 Content-Type: multipart/form-data，got headers: {headers_wire}"
    );
    let body = &cap_guard[4];
    // R3020：multipart body 经 byte-wire（__zw_bytes: csv-decimal）——解码为字节再 UTF-8 文本断言 multipart 标记 + 值。
    let body_text = String::from_utf8_lossy(&crate::decode_body_bytes_raw(body).unwrap_or_default()).to_string();
    assert!(
        body_text.contains("Content-Disposition: form-data; name=\"q\"") && body_text.contains("search"),
        "fetch FormData → body 含 multipart 标记 + 值，got body: {body_text}"
    );
}

#[test]
fn test_fetch_body_types_r3015() {
    // R3015：fetch body 类型扩展——URLSearchParams（urlencoded + Content-Type）/ Blob（字节 + Content-Type）。
    // 旧 fetch 经 String(body) 归一：URLSearchParams body 缺 Content-Type、Blob body 为 '[object Blob]'。
    // 本切片：instanceof 链分发，缺省 Content-Type 时设（用户显式 Content-Type 保留不覆写）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // mock __zw_fetch：每次调用 clear + 捕获全部 args（id/method/url/headersWire/body）。
    let captured: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let cap = Arc::clone(&captured);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let mut g = cap.lock().unwrap();
            g.clear();
            for a in args.iter() {
                g.push(a.clone());
            }
            "ok".to_string()
        }),
    );

    // URLSearchParams body：body=urlencoded + Content-Type application/x-www-form-urlencoded。
    sandbox
        .execute("fetch('http://t/u', { method: 'POST', body: new URLSearchParams({ q: 'hello', n: '1' }) });")
        .unwrap();
    {
        let g = captured.lock().unwrap();
        assert_eq!(g[1], "POST", "URLSearchParams body → method=POST");
        assert_eq!(g[4], "q=hello&n=1", "URLSearchParams body → urlencoded（toString）");
        let hw = g[3].to_lowercase();
        assert!(
            hw.contains("application/x-www-form-urlencoded"),
            "URLSearchParams → Content-Type application/x-www-form-urlencoded，got headers: {hw}"
        );
    }

    // Blob body：body=byte-wire（R3020）+ Content-Type blob.type（旧 '[object Blob]'）。
    sandbox
        .execute("fetch('http://t/b', { method: 'POST', body: new Blob(['payload'], { type: 'text/plain' }) });")
        .unwrap();
    {
        let g = captured.lock().unwrap();
        // R3020：Blob 字节经 __zw_bytes: csv-decimal wire（二进制保真）——解码为字节再 UTF-8 文本断言。
        let blob_bytes = crate::decode_body_bytes_raw(&g[4]).expect("Blob body 须为 __zw_bytes: byte-wire");
        assert_eq!(
            String::from_utf8_lossy(&blob_bytes),
            "payload",
            "Blob body → 字节 byte-wire，解码为 'payload'（旧 String(blob)='[object Blob]'）"
        );
        let hw = g[3].to_lowercase();
        assert!(hw.contains("text/plain"), "Blob → Content-Type blob.type=text/plain，got headers: {hw}");
    }

    // string body + 显式 Content-Type：body 原样，用户 Content-Type 保留（不被覆写）。
    sandbox
        .execute(
            "fetch('http://t/s', { method: 'POST', body: '{\"a\":1}', headers: { 'Content-Type': 'application/json' } });",
        )
        .unwrap();
    {
        let g = captured.lock().unwrap();
        assert_eq!(g[4], "{\"a\":1}", "string body 原样（String()）");
        let hw = g[3].to_lowercase();
        assert!(
            hw.contains("application/json") && !hw.contains("application/x-www-form-urlencoded"),
            "用户显式 Content-Type 保留不覆写，got headers: {hw}"
        );
    }

    // FormData body 仍 multipart（R3014 非回归）。
    sandbox
        .execute("var fd = new FormData(); fd.append('k', 'v'); fetch('http://t/f', { method: 'POST', body: fd });")
        .unwrap();
    {
        let g = captured.lock().unwrap();
        let hw = g[3].to_lowercase();
        assert!(hw.contains("multipart/form-data"), "FormData body 仍 multipart（R3014 非回归）");
        // R3020：multipart body 经 byte-wire（__zw_bytes: csv-decimal）——解码为字节再 UTF-8 文本断言 multipart 标记。
        let fd_text = String::from_utf8_lossy(&crate::decode_body_bytes_raw(&g[4]).unwrap_or_default()).to_string();
        assert!(
            fd_text.contains("Content-Disposition: form-data; name=\"k\""),
            "FormData body multipart 标记（R3014 非回归），got: {fd_text}"
        );
    }
}

#[test]
fn test_detached_document_traversal_r3016() {
    // R3016：detached document body.childNodes 递归遍历（DOMPurify.sanitize 核心阻塞）。R3013 让 detached doc
    // 可 querySelector，但 body.childNodes 恒空（hollow）→ DOMPurify 设 dom.body.innerHTML 后无法递归 walk 清洗。
    // 本切片：__zw_parse_html_child_nodes + _zwDetachedEl 递归 element/text/comment proxy。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 嵌套 HTML：div.a > (b>text + tail text) + 文本 + 注释。
    sandbox
        .execute(
            "var doc = document.implementation.createHTMLDocument('');\
             doc.body.innerHTML = '<div class=\"a\"><b>bold</b>tail</div>mid<!--cmt-->';",
        )
        .unwrap();

    // body.childNodes：[div.a, text 'mid', comment 'cmt']（3 节点）。
    sandbox.execute("globalThis.__cn = doc.body.childNodes; globalThis.__len = doc.body.childNodes.length;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len)").unwrap().value, "3", "body.childNodes.length=3（div + text + comment）");

    // [0] div.a：element，tag/attr/递归 childNodes。
    sandbox
        .execute(
            "globalThis.__n0 = globalThis.__cn[0];\
             globalThis.__n0Type = globalThis.__n0.nodeType;\
             globalThis.__n0Tag = globalThis.__n0.tagName;\
             globalThis.__n0Cls = globalThis.__n0.getAttribute('class');\
             globalThis.__n0ChildLen = globalThis.__n0.childNodes.length;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__n0Type)").unwrap().value, "1", "body.childNodes[0] nodeType=1（element）");
    assert_eq!(sandbox.execute("String(globalThis.__n0Tag)").unwrap().value, "DIV", "body.childNodes[0] tagName=DIV");
    assert_eq!(sandbox.execute("String(globalThis.__n0Cls)").unwrap().value, "a", "body.childNodes[0] getAttribute('class')='a'");
    assert_eq!(sandbox.execute("String(globalThis.__n0ChildLen)").unwrap().value, "2", "div.a.childNodes.length=2（b + text 'tail'）");

    // div.a > [0] b：递归子元素，textContent='bold'。
    sandbox
        .execute(
            "globalThis.__b = globalThis.__n0.childNodes[0];\
             globalThis.__bTag = globalThis.__b.tagName;\
             globalThis.__bText = globalThis.__b.textContent;\
             globalThis.__bChildLen = globalThis.__b.childNodes.length;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__bTag)").unwrap().value, "B", "div.a.childNodes[0] tagName=B（递归子）");
    assert_eq!(sandbox.execute("String(globalThis.__bText)").unwrap().value, "bold", "b.textContent='bold'");
    assert_eq!(sandbox.execute("String(globalThis.__bChildLen)").unwrap().value, "1", "b.childNodes.length=1（text 'bold'）");

    // b > [0] text 'bold'：叶文本节点。
    sandbox
        .execute("globalThis.__bt = globalThis.__b.childNodes[0]; globalThis.__btType = globalThis.__bt.nodeType; globalThis.__btVal = globalThis.__bt.nodeValue;")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__btType)").unwrap().value, "3", "b.childNodes[0] nodeType=3（text）");
    assert_eq!(sandbox.execute("String(globalThis.__btVal)").unwrap().value, "bold", "text nodeValue='bold'");

    // [1] text 'mid' + [2] comment 'cmt'：body 直接子。
    sandbox
        .execute("globalThis.__n1 = globalThis.__cn[1]; globalThis.__n2 = globalThis.__cn[2];")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__n1.nodeType)").unwrap().value, "3", "body.childNodes[1] nodeType=3（text）");
    assert_eq!(sandbox.execute("String(globalThis.__n1.nodeValue)").unwrap().value, "mid", "text nodeValue='mid'");
    assert_eq!(sandbox.execute("String(globalThis.__n2.nodeType)").unwrap().value, "8", "body.childNodes[2] nodeType=8（comment）");
    assert_eq!(sandbox.execute("String(globalThis.__n2.nodeValue)").unwrap().value, "cmt", "comment nodeValue='cmt'");

    // body.children：仅元素子（div），不含 text/comment。
    assert_eq!(sandbox.execute("String(doc.body.children.length)").unwrap().value, "1", "body.children.length=1（仅元素，不含 text/comment）");
    assert_eq!(sandbox.execute("String(doc.body.children[0].tagName)").unwrap().value, "DIV", "body.children[0]=div");

    // body.firstChild = div（首子）。
    assert_eq!(sandbox.execute("String(doc.body.firstChild.tagName)").unwrap().value, "DIV", "body.firstChild=div");

    // R3013 query 非回归：querySelector 仍工作。
    assert_eq!(sandbox.execute("String(doc.querySelector('.a').tagName)").unwrap().value, "DIV", "querySelector 仍工作（R3013 非回归）");
}

#[test]
fn test_detached_mutable_tree_r3017() {
    // R3017：detached document 可变树（DOMPurify.sanitize 真跑通核心）。R3016 让 body.childNodes 可递归遍历（只读），
    // 但 DOMPurify 核心是 `node.parentNode.removeChild(node)` 清洗 + 读 `body.innerHTML`——须 parentNode + removeChild
    // mutation + 序列化反映。本切片：lazy-snapshot → cached mutable JS tree（节点 relink，无 selector 重算）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 建树：div > (span + p) + 文本。访问 body.childNodes 触发建 mutable tree。
    sandbox
        .execute(
            "var doc = document.implementation.createHTMLDocument('');\
             doc.body.innerHTML = '<div><span>x</span><p>y</p></div>tail';\
             globalThis.__cn = doc.body.childNodes;",
        )
        .unwrap();

    // parentNode：span 的父是 div（DOMPurify 经 node.parentNode.removeChild 取 parent）。
    sandbox
        .execute(
            "globalThis.__div = globalThis.__cn[0];\
             globalThis.__span = globalThis.__div.childNodes[0];\
             globalThis.__spanParentTag = globalThis.__span.parentNode.tagName;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__spanParentTag)").unwrap().value, "DIV", "span.parentNode.tagName=DIV");
    // body 的 parentNode 为 null（detached root）。
    assert_eq!(sandbox.execute("String(doc.body.parentNode)").unwrap().value, "null", "body.parentNode=null（detached root）");

    // removeChild：从 div 移除 span → div.childNodes 不含 span + body.innerHTML 序列化反映移除。
    sandbox
        .execute(
            "globalThis.__div.removeChild(globalThis.__span);\
             globalThis.__divChildLen = globalThis.__div.childNodes.length;\
             globalThis.__html1 = doc.body.innerHTML;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__divChildLen)").unwrap().value, "1", "removeChild(span) 后 div.childNodes.length=1（仅 p）");
    assert_eq!(
        sandbox.execute("String(globalThis.__html1.indexOf('<span') >= 0)").unwrap().value,
        "false",
        "removeChild 后 body.innerHTML 不含 <span>（序列化反映移除）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__html1.indexOf('<p>y</p>') >= 0)").unwrap().value,
        "true",
        "removeChild 后 body.innerHTML 仍含 <p>（兄弟保留）"
    );

    // DOMPurify 核心模式：node.parentNode.removeChild(node) 移除整 div（body 直接子）。
    sandbox
        .execute(
            "globalThis.__div.parentNode.removeChild(globalThis.__div);\
             globalThis.__bodyLen = doc.body.childNodes.length;\
             globalThis.__html2 = doc.body.innerHTML;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__bodyLen)").unwrap().value, "1", "removeChild(div) 后 body.childNodes.length=1（仅 text 'tail'）");
    assert_eq!(
        sandbox.execute("String(globalThis.__html2.indexOf('<div') >= 0)").unwrap().value,
        "false",
        "removeChild(div) 后 body.innerHTML 不含 <div>"
    );

    // appendChild reparent：建新树，把 p 从 div 移到 body。
    sandbox
        .execute(
            "var d2 = document.implementation.createHTMLDocument('');\
             d2.body.innerHTML = '<div><p>z</p></div>';\
             var p = d2.body.childNodes[0].childNodes[0];\
             d2.body.appendChild(p);\
             globalThis.__d2bodyLen = d2.body.childNodes.length;\
             globalThis.__d2pParent = p.parentNode.tagName;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__d2bodyLen)").unwrap().value, "2", "appendChild(p) 后 body.childNodes.length=2（div + p）");
    assert_eq!(sandbox.execute("String(globalThis.__d2pParent)").unwrap().value, "BODY", "appendChild reparent：p.parentNode=BODY");

    // R3016 只读遍历非回归：嵌套 walk 仍正确。
    sandbox
        .execute(
            "var d3 = document.implementation.createHTMLDocument('');\
             d3.body.innerHTML = '<a><b>deep</b>tail</a>after<!--c-->';\
             globalThis.__n0 = d3.body.childNodes[0];",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__n0.tagName)").unwrap().value, "A", "R3016 非回归：body.childNodes[0]=A");
    assert_eq!(sandbox.execute("String(globalThis.__n0.childNodes[0].tagName)").unwrap().value, "B", "R3016 非回归：递归子 B");
    assert_eq!(sandbox.execute("String(globalThis.__n0.childNodes[0].textContent)").unwrap().value, "deep", "R3016 非回归：b.textContent='deep'");
    assert_eq!(sandbox.execute("String(d3.body.childNodes[2].nodeType)").unwrap().value, "8", "R3016 非回归：comment nodeType=8");
}

#[test]
fn test_detached_attribute_mutation_r3018() {
    // R3018：detached mutable tree 属性 mutation + 兄弟导航。R3017 闭合 removeChild/appendChild/parentNode
    // （DOMPurify 移元素核心），但 setAttribute/removeAttribute/previousSibling/nextSibling/insertBefore/replaceChild
    // 缺失（R3017 known limitation ① 属性 mutation defer、② previousSibling/nextSibling 未接）→ DOMPurify 去
    // on*/style 属性 + 节点重定位/替换全失效。本切片补齐 attribute mutation（入 attrs 数组，序列化反映 + id/class
    // IDL 反射同步）+ 兄弟/末子导航（经 parentNode 动态求值）+ insertBefore/replaceChild。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // setAttribute（新增 + 更新）+ removeAttribute + id/class IDL 反射同步 + 序列化反映。
    sandbox
        .execute(
            "var doc = document.implementation.createHTMLDocument('');\
             doc.body.innerHTML = '<div id=\"d\" class=\"c\"><span>a</span><b>x</b><i>y</i></div>';\
             var div = doc.body.childNodes[0];\
             div.setAttribute('data-n', '42');\
             div.setAttribute('id', 'd2');\
             div.removeAttribute('class');\
             globalThis.__hasNew = div.hasAttribute('data-n');\
             globalThis.__getNew = div.getAttribute('data-n');\
             globalThis.__idReflect = div.id;\
             globalThis.__idAttr = div.getAttribute('id');\
             globalThis.__clsReflect = div.className;\
             globalThis.__hasCls = div.hasAttribute('class');\
             globalThis.__outer = div.outerHTML;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__hasNew)").unwrap().value, "true", "setAttribute('data-n') 后 hasAttribute=true");
    assert_eq!(sandbox.execute("globalThis.__getNew").unwrap().value, "42", "getAttribute('data-n')='42'（新增属性）");
    assert_eq!(sandbox.execute("globalThis.__idReflect").unwrap().value, "d2", "setAttribute('id','d2') 后 div.id='d2'（IDL 反射同步）");
    assert_eq!(sandbox.execute("globalThis.__idAttr").unwrap().value, "d2", "getAttribute('id')='d2'（latest-wins 更新）");
    assert_eq!(sandbox.execute("globalThis.__clsReflect").unwrap().value, "", "removeAttribute('class') 后 div.className=''（IDL 反射清空）");
    assert_eq!(sandbox.execute("String(globalThis.__hasCls)").unwrap().value, "false", "removeAttribute('class') 后 hasAttribute=false");
    let outer = sandbox.execute("globalThis.__outer").unwrap().value;
    assert!(outer.contains("data-n=\"42\""), "serialize 含 data-n=\"42\"：{outer}");
    assert!(outer.contains("id=\"d2\""), "serialize 含 id=\"d2\"（setAttribute 更新）：{outer}");
    assert!(!outer.contains("class="), "serialize 不含 class（removeAttribute 反映）：{outer}");

    // 兄弟导航（previousSibling/nextSibling）+ lastChild。
    sandbox
        .execute(
            "var span = div.childNodes[0];\
             var b = div.childNodes[1];\
             var i = div.childNodes[2];\
             globalThis.__spanPrev = span.previousSibling;\
             globalThis.__spanNextTag = span.nextSibling.tagName;\
             globalThis.__bPrevTag = b.previousSibling.tagName;\
             globalThis.__iNext = i.nextSibling;\
             globalThis.__lastTag = div.lastChild.tagName;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__spanPrev)").unwrap().value, "null", "首子 span.previousSibling=null");
    assert_eq!(sandbox.execute("globalThis.__spanNextTag").unwrap().value, "B", "span.nextSibling.tagName=B");
    assert_eq!(sandbox.execute("globalThis.__bPrevTag").unwrap().value, "SPAN", "b.previousSibling.tagName=SPAN");
    assert_eq!(sandbox.execute("String(globalThis.__iNext)").unwrap().value, "null", "末子 i.nextSibling=null");
    assert_eq!(sandbox.execute("globalThis.__lastTag").unwrap().value, "I", "div.lastChild.tagName=I（末子）");

    // insertBefore：在 b 前插文本节点 → [span, t, b, i]，序列化反映插入位置。
    sandbox
        .execute(
            "var t = doc.createTextNode('INS');\
             div.insertBefore(t, b);\
             globalThis.__len = div.childNodes.length;\
             globalThis.__idx1 = div.childNodes[1].nodeValue;\
             globalThis.__tPrevTag = t.previousSibling.tagName;\
             globalThis.__tNextTag = t.nextSibling.tagName;\
             globalThis.__insHtml = div.outerHTML;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len)").unwrap().value, "4", "insertBefore 后 childNodes.length=4");
    assert_eq!(sandbox.execute("globalThis.__idx1").unwrap().value, "INS", "insertBefore(t,b) 后 childNodes[1]=t（落 b 前）");
    assert_eq!(sandbox.execute("globalThis.__tPrevTag").unwrap().value, "SPAN", "t.previousSibling=SPAN（兄弟反映插入位）");
    assert_eq!(sandbox.execute("globalThis.__tNextTag").unwrap().value, "B", "t.nextSibling=B（兄弟反映插入位）");
    assert!(sandbox.execute("globalThis.__insHtml").unwrap().value.contains("INS"), "serialize 含 INS（序列化反映插入）");

    // replaceChild：用末子 i 替换首子 span（i 在 span 之后，验证 adopt-then-index 顺序）→ [i, t, b]。
    sandbox
        .execute(
            "div.replaceChild(i, span);\
             globalThis.__len2 = div.childNodes.length;\
             globalThis.__firstTag = div.childNodes[0].tagName;\
             globalThis.__spanParent = span.parentNode;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len2)").unwrap().value, "3", "replaceChild 后 childNodes.length=3（i 移位非新增）");
    assert_eq!(sandbox.execute("globalThis.__firstTag").unwrap().value, "I", "replaceChild(i,span) 后 childNodes[0]=I（i adopt 到首位）");
    assert_eq!(sandbox.execute("String(globalThis.__spanParent)").unwrap().value, "null", "被替换的 span.parentNode=null（脱链）");
}

#[test]
fn test_sanitize_dompurify_style_r3018() {
    // R3018：DOMPurify 式 sanitize 端到端 driving test。R3013（query）+ R3016（traverse）+ R3017（mutable remove）
    // + R3018（attribute mutation + sibling）闭合后，detached document 成可清洗解析容器。本测试复刻 DOMPurify
    // 核心算法形态——设 body.innerHTML → 递归 walk childNodes（取 copy 防 mutate-during-iterate）→ 移禁元素
    // （script/iframe/object，node.parentNode.removeChild）→ 去 on*/style 属性（遍历 attributes copy + removeAttribute）
    // → 读 body.innerHTML 取清洗结果。证明 R3013-R3018 链路对真实清洗库的支撑能力。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 注入 DOMPurify 式 sanitize（复刻核心算法：移禁元素 + 去 on*/style 属性 + 递归子）。
    sandbox
        .execute(
            "function zwSanitize(html) {\
               var doc = document.implementation.createHTMLDocument('');\
               doc.body.innerHTML = html;\
               zwWalk(doc.body);\
               return doc.body.innerHTML;\
             }\
             function zwWalk(node) {\
               var kids = Array.prototype.slice.call(node.childNodes);\
               for (var i = 0; i < kids.length; i++) {\
                 var c = kids[i];\
                 if (c.nodeType !== 1) continue;\
                 var tag = c.tagName.toLowerCase();\
                 if (tag === 'script' || tag === 'iframe' || tag === 'object') {\
                   c.parentNode.removeChild(c);\
                   continue;\
                 }\
                 var ac = Array.prototype.slice.call(c.attributes);\
                 for (var j = 0; j < ac.length; j++) {\
                   var an = ac[j].name;\
                   if (/^on/i.test(an) || an === 'style') c.removeAttribute(an);\
                 }\
                 zwWalk(c);\
               }\
             }",
        )
        .unwrap();

    // 输入：含 script/iframe 禁元素 + onclick/onerror/style 危险属性，夹保留属性 class/title 与正文。
    sandbox
        .execute(
            "globalThis.__out = zwSanitize('<div onclick=\"evil()\" class=\"keep\"><script>alert(1)</script>\
             <p style=\"color:red\" title=\"t\">hi</p><iframe src=\"x\"></iframe>\
             <img onerror=\"steal()\" src=\"a.png\" alt=\"ok\"></div>');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__out").unwrap().value;
    assert!(!out.contains("<script"), "sanitize 移除 <script>：{out}");
    assert!(!out.contains("<iframe"), "sanitize 移除 <iframe>：{out}");
    assert!(!out.contains("onclick"), "sanitize 剥离 onclick 属性：{out}");
    assert!(!out.contains("onerror"), "sanitize 剥离 onerror 属性：{out}");
    assert!(!out.contains("style="), "sanitize 剥离 style 属性：{out}");
    assert!(out.contains("class=\"keep\""), "sanitize 保留安全属性 class：{out}");
    assert!(out.contains("title=\"t\""), "sanitize 保留安全属性 title：{out}");
    assert!(out.contains("alt=\"ok\""), "sanitize 保留 img 安全属性 alt：{out}");
    assert!(out.contains("<p title=\"t\">hi</p>"), "sanitize 保留正文 p：{out}");
    assert!(out.contains("src=\"a.png\""), "sanitize 保留 img src：{out}");

    // 第二例：嵌套禁元素（script 套在合法 section 内）+ 深层 on* 属性，验证递归 walk 到深处。
    sandbox
        .execute(
            "globalThis.__out2 = zwSanitize('<section><article onmouseover=\"bad()\"><h2>title</h2>\
             <object data=\"x\"></object><p>body <a href=\"/ok\" onclick=\"x()\">link</a></p></article></section>');",
        )
        .unwrap();
    let out2 = sandbox.execute("globalThis.__out2").unwrap().value;
    assert!(!out2.contains("<object"), "递归移除嵌套 <object>：{out2}");
    assert!(!out2.contains("onmouseover"), "递归剥离深层 onmouseover：{out2}");
    assert!(!out2.contains("onclick"), "递归剥离深层 <a> onclick：{out2}");
    assert!(out2.contains("<h2>title</h2>"), "递归保留嵌套正文 h2：{out2}");
    assert!(out2.contains("href=\"/ok\""), "递归保留 <a> 安全属性 href：{out2}");
    assert!(out2.contains(">link</a>"), "递归保留 <a> 正文：{out2}");

    // 第三例：纯文本 + 无危险内容 → 原样返回（idempotent，无误伤）。
    sandbox
        .execute(
            "globalThis.__out3 = zwSanitize('<p>plain <b>bold</b> text</p>');",
        )
        .unwrap();
    let out3 = sandbox.execute("globalThis.__out3").unwrap().value;
    assert_eq!(out3, "<p>plain <b>bold</b> text</p>", "无危险内容原样返回（idempotent）：{out3}");
}



#[test]
fn test_sanitize_dompurify_real_r3019() {
    // R3019：真实 DOMPurify 库端到端实测（R3018 下一步①）。加载真实 DOMPurify 3.2.7（fixture 66KB，
    // Apache-2.0/MPL-2.0 双许可，许可证头保留于 fixture）→ sanitize 各类 dirty 输入验证清洗生效。
    // 承接 R3018 driving test（复刻核心算法形态）升级为真实库验证：库内 feature-detect 路径
    // （lookupGetter Element.prototype 成员固化、instanceof HTMLFormElement/NamedNodeMap、cross-document
    // getElementsByTagName.call(doc)、NodeIterator 遍历、hasChildNodes mXSS 检查、FORBID_CONTENTS 整节点
    // 移除、keep-content clone 子节点插回）全链路真实执行。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 加载真实 DOMPurify（fixture 保留原始许可证头）。加载失败 = shim 基础面不满足库加载 → panic。
    let dp = include_str!("../../tests/fixtures/dompurify.js");
    sandbox.execute(dp).unwrap();
    let loaded = sandbox.execute(
        "typeof DOMPurify === 'function' && DOMPurify.version === '3.2.7' && DOMPurify.isSupported === true",
    ).unwrap().value;
    assert_eq!(loaded, "true", "DOMPurify 3.2.7 加载 + isSupported 须为 true（feature-detect 全链路）：{loaded}");

    // ① 禁元素移除 + 危险属性剥离：script/iframe 整节点移除（FORBID_CONTENTS），on*/style 属性剥离，
    //    安全属性（src/class/title/alt/href）保留，正文保留。
    let cases1 = [
        // (输入, 期望结果, 断言说明)
        ("<img src=x onerror=alert(1)>", "<img src=\"x\">", "剥离 onerror 保留 src"),
        ("<script>alert(1)</script><p>hi</p>", "<p>hi</p>", "移除 script（host parse 亦剥，双路径）"),
        ("<div onclick=evil()>text</div>", "<div>text</div>", "剥离 onclick 保留正文"),
        ("<b>hello</b>", "<b>hello</b>", "安全输入 idempotent"),
        ("<div><iframe src=\"x\"></iframe>keep</div>", "<div>keep</div>", "iframe FORBID_CONTENTS 整节点移除（R3019b 回归：旧实现 removed 记录但未真移除）"),
        ("<a href='/ok' onclick='bad()'>link</a>", "<a href=\"/ok\">link</a>", "剥离 onclick 保留 href"),
        ("<p>plain <strong>bold</strong> text</p>", "<p>plain <strong>bold</strong> text</p>", "安全输入原样"),
        ("<p onmouseover=x()>hover <b>bold</b></p>", "<p>hover <b>bold</b></p>", "剥离深层 onmouseover 保留子元素"),
    ];
    for (i, (input, expected, why)) in cases1.iter().enumerate() {
        let js = format!("try {{ globalThis.__r = DOMPurify.sanitize({input:?}); }} catch(e) {{ globalThis.__r = 'THROW:' + e; }}");
        sandbox.execute(&js).unwrap();
        let r = sandbox.execute("globalThis.__r").unwrap().value;
        assert_eq!(r, *expected, "case{i}（{why}）：input={input:?} expected={expected:?} got={r:?}");
    }

    // ② keep-content 路径：非 allowed 且非 FORBID_CONTENTS 标签 → 移除元素但 clone 子节点插回父节点
    //    （DOMPurify 默认 KEEP_CONTENT=true；custom-tag 不在白名单）。克隆须复制子 span 且 relink parentNode。
    sandbox
        .execute("globalThis.__r = DOMPurify.sanitize('<div><custom-tag><span>t</span></custom-tag></div>');")
        .unwrap();
    let r = sandbox.execute("globalThis.__r").unwrap().value;
    assert_eq!(r, "<div><span>t</span></div>", "keep-content 移除 custom-tag 保留子 span（clone 插回）：{r}");

    // ③ FORBID_CONTENTS 整节点移除不留内容（noscript 在 DEFAULT_FORBID_CONTENTS）：
    //    KEEP_CONTENT 不适用 → 整节点移除（含子内容），与真实 DOMPurify 语义一致。
    sandbox
        .execute("globalThis.__r = DOMPurify.sanitize('<div><noscript><p>ns</p></noscript></div>');")
        .unwrap();
    let r = sandbox.execute("globalThis.__r").unwrap().value;
    assert_eq!(r, "<div></div>", "noscript FORBID_CONTENTS 整节点移除不留子内容：{r}");

    // ④ removed 数组真实记录移除（R3019b 回归：旧实现 removed 有记录但节点未从树中移除——_forceRemove
    //    的 getParentNode fallback 恒 null → removeChild 抛错 → catch 走空 remove() 静默失败）。
    sandbox
        .execute("DOMPurify.removed = []; globalThis.__r = DOMPurify.sanitize('<div><iframe src=\"x\"></iframe>keep</div>'); globalThis.__removed = DOMPurify.removed.map(function(x){ return x.element ? x.element.nodeName : '?'; }).join(',');")
        .unwrap();
    let r = sandbox.execute("globalThis.__r").unwrap().value;
    let removed = sandbox.execute("globalThis.__removed").unwrap().value;
    assert_eq!(r, "<div>keep</div>", "removed 记录 + 真移除一致：{r}");
    assert_eq!(removed, "IFRAME", "removed 数组记录 IFRAME 且结果无 iframe（移除真生效）：{removed}");
}

#[test]
fn test_fetch_binary_body_byte_wire_r3020() {
    // R3020：fetch 二进制 body byte-wire（csv-decimal）。旧路径 Blob/FormData body 经 TextDecoder.decode
    // 对非 UTF-8 字节 lossy（0xFF→U+FFFD 等），破坏二进制上传（文件 / Canvas.toBlob / multipart 二进制内容）。
    // 本切片 shim 把 Blob/FormData 字节编码为 `__zw_bytes:` + csv-decimal 传 host，host（fetch_bridge）解码为
    // Vec<u8>（body_bytes），app default_fetch_handler 优先用 body_bytes 联网——全链路二进制保真。
    // 本 driving test 验证 shim 编码侧（host 解码侧见 fetch_bridge::tests::body_bytes_wire_*）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // mock __zw_fetch 捕获 method（args[1]）+ body wire（args[4]）。
    let captured: Arc<Mutex<(String, String)>> = Arc::new(Mutex::new((String::new(), String::new())));
    let cap = Arc::clone(&captured);
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let m = args.get(1).cloned().unwrap_or_default();
            let b = args.get(4).cloned().unwrap_or_default();
            if let Ok(mut c) = cap.lock() {
                *c = (m, b);
            }
            "ok".to_string()
        }),
    );

    // ① 二进制 Blob body（含非 UTF-8 字节 0xFF/0x00/0x80）→ __zw_bytes: csv-decimal。
    sandbox
        .execute(
            "var blob = new Blob([new Uint8Array([255,0,128,72,105])], {type:'application/octet-stream'});\
             fetch('http://test.local/up', {method:'POST', body: blob});",
        )
        .unwrap();
    let (m, b) = captured.lock().unwrap().clone();
    assert_eq!(m, "POST", "Blob body fetch 用 POST 方法");
    assert_eq!(b, "__zw_bytes:255,0,128,72,105", "二进制 Blob body 编码为 __zw_bytes: csv-decimal（0xFF/0x00 保真）：{b}");

    // ② 文本 body（string）→ 原样，不带 __zw_bytes: 前缀（无歧义，按 body 类型决定，非内容匹配）。
    sandbox
        .execute("fetch('http://test.local/up', {method:'POST', body: 'plain text'});")
        .unwrap();
    let (_, b2) = captured.lock().unwrap().clone();
    assert_eq!(b2, "plain text", "文本 body 原样传递（无 __zw_bytes: 前缀）：{b2}");

    // ③ FormData 含二进制 Blob file → multipart body 经 __zw_bytes: 编码（含二进制文件内容字节）。
    sandbox
        .execute(
            "var fd = new FormData(); fd.append('f', new Blob([new Uint8Array([255,0,128])]));\
             fetch('http://test.local/up', {method:'POST', body: fd});",
        )
        .unwrap();
    let (_, b3) = captured.lock().unwrap().clone();
    assert!(b3.starts_with("__zw_bytes:"), "FormData multipart body 经 byte-wire 编码：{b3}");
    // 解码后须含二进制字节序列 [255,0,128]（append 的二进制 Blob 文件内容保真，嵌 multipart 体内）。
    let csv = &b3["__zw_bytes:".len()..];
    let decoded: Vec<u8> = csv.split(',').filter_map(|s| s.parse::<u8>().ok()).collect();
    let has_binary = decoded.windows(3).any(|w| w == [255, 0, 128]);
    assert!(has_binary, "FormData multipart 解码后含二进制字节序列 [255,0,128]（文件内容保真）：{csv}");
}

#[test]
fn test_fetch_response_binary_body_r3021() {
    // R3021：fetch 响应侧二进制 body byte-wire（与请求侧 R3020 对称）。host 对非 UTF-8 response body 经
    // `__zw_bytes:` csv-decimal wire 传 JS（serialize_response），shim _makeResponseFromWire 解码为 Uint8Array
    // 存 Response._bodyBytes，response.arrayBuffer()/blob() 取保真字节（旧 String::from_utf8_lossy 破坏 0xFF 等）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // mock __zw_fetch 捕获 id（args[0]）供 Rust 侧异步 resolve 二进制 body wire。
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

    // ① 二进制 response body wire（__zw_bytes: csv-decimal）→ response.arrayBuffer() 取保真字节。
    sandbox
        .execute(
            "fetch('http://t/bin').then(function(r){\
               globalThis.__status = r.status;\
               r.arrayBuffer().then(function(ab){ globalThis.__ab = ab; });\
             });",
        )
        .unwrap();
    let id = captured_id.lock().unwrap().clone();
    assert!(!id.is_empty(), "__zw_fetch 被调用且 id 已捕获");
    // wire：status=200 / statusText=OK / headersWire='' / body=__zw_bytes:255,0,128,72,105（非 UTF-8 字节）。
    let wire = "__zwfr:200\u{001f}OK\u{001f}\u{001f}__zw_bytes:255,0,128,72,105";
    sandbox.resolve_async_callback(&id, wire);
    // 多级 .then 链须额外 microtask flush。
    sandbox.execute("0;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__status").unwrap().value, "200", "二进制 body response status=200");
    sandbox.execute(
        "globalThis.__len = globalThis.__ab.length;\
         globalThis.__bytes = Array.prototype.slice.call(globalThis.__ab).join(',');",
    ).unwrap();
    assert_eq!(sandbox.execute("globalThis.__len").unwrap().value, "5", "response.arrayBuffer() 字节长度=5");
    assert_eq!(
        sandbox.execute("globalThis.__bytes").unwrap().value,
        "255,0,128,72,105",
        "response.arrayBuffer() 返原始非 UTF-8 字节 [255,0,128,72,105]（二进制保真）"
    );

    // ② response.blob().arrayBuffer() 同样保真（Blob 包二进制 _bodyBytes 字节）。
    sandbox
        .execute(
            "globalThis.__ab2 = null;\
             fetch('http://t/bin2').then(function(r){ r.blob().then(function(b){ b.arrayBuffer().then(function(ab){ globalThis.__ab2 = ab; }); }); });",
        )
        .unwrap();
    let id2 = captured_id.lock().unwrap().clone();
    sandbox.resolve_async_callback(&id2, wire);
    sandbox.execute("0;").unwrap();
    sandbox.execute("0;").unwrap(); // 三级 .then 链须再多一次 flush
    sandbox.execute("globalThis.__bytes2 = globalThis.__ab2 ? Array.prototype.slice.call(globalThis.__ab2).join(',') : '(unset)';").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__bytes2").unwrap().value,
        "255,0,128,72,105",
        "response.blob().arrayBuffer() 返保真二进制字节（Blob 包 _bodyBytes）"
    );

    // ③ response.text() 仍可读（TextDecoder 解码二进制字节，0xFF→U+FFFD 等——文本视图 best-effort），
    //    status/headers 等非 body 字段不受 byte-wire 影响。
    sandbox
        .execute(
            "globalThis.__t = null;\
             fetch('http://t/bin3').then(function(r){ globalThis.__t = r.text(); });",
        )
        .unwrap();
    let id3 = captured_id.lock().unwrap().clone();
    sandbox.resolve_async_callback(&id3, wire);
    sandbox.execute("globalThis.__tval = ''; globalThis.__t && globalThis.__t.then(function(v){ globalThis.__tval = v; });").unwrap();
    // text() 解码二进制字节为字符串（非 UTF-8 → 替换字符），非空即说明 text() 路径走通（best-effort 文本视图）。
    assert!(
        sandbox.execute("String(globalThis.__tval.length > 0)").unwrap().value == "true",
        "response.text() 对二进制 body 返非空文本（TextDecoder best-effort 解码）"
    );
}

#[test]
fn test_namednodemap_setnameditem_main_dom_r3022() {
    // R3022：NamedNodeMap setNamedItem(attr)/removeNamedItem(name) 真 mutation（主 DOM 路径）。
    // R3003 闭合 attributes 读路径（getNamedItem/Attr.value latest-wins），但 setNamedItem/removeNamedItem
    // 仍为只读 no-op → lib 经 element.attributes.setNamedItem(attr) 改属性静默失效。本切片委托元素 setAttribute/
    // removeAttribute host 路径，返旧/移除 Attr。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d' class='c'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // setNamedItem 新属性 → getAttribute / getNamedItem 反映；返旧 Attr=null（新属性）。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             var old1 = d.attributes.setNamedItem({ name: 'data-x', value: 'v1' });\
             globalThis.__old1 = String(old1 === null);\
             globalThis.__gv = d.getAttribute('data-x');\
             globalThis.__nv = d.attributes.getNamedItem('data-x').value;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__old1").unwrap().value, "true", "setNamedItem 新属性返 null（无旧 Attr）");
    assert_eq!(sandbox.execute("globalThis.__gv").unwrap().value, "v1", "setNamedItem 后 getAttribute='v1'（经 setAttribute host 路径）");
    assert_eq!(sandbox.execute("globalThis.__nv").unwrap().value, "v1", "setNamedItem 后 attributes.getNamedItem('data-x').value='v1'（latest-wins）");

    // setNamedItem 改既有属性 → 返旧 Attr {name,value}；新值反映。
    sandbox
        .execute(
            "var old2 = d.attributes.setNamedItem({ name: 'class', value: 'newc' });\
             globalThis.__old2Name = old2 ? old2.name : '(null)';\
             globalThis.__old2Val = old2 ? old2.value : '(null)';\
             globalThis.__newClass = d.getAttribute('class');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__old2Name").unwrap().value, "class", "setNamedItem 改既有属性返旧 Attr.name=class");
    assert_eq!(sandbox.execute("globalThis.__old2Val").unwrap().value, "c", "setNamedItem 返旧 Attr.value='c'（原值）");
    assert_eq!(sandbox.execute("globalThis.__newClass").unwrap().value, "newc", "setNamedItem 改 class 后 getAttribute='newc'");

    // removeNamedItem 既存 → 返移除 Attr；hasAttribute=false。
    sandbox
        .execute(
            "var rem = d.attributes.removeNamedItem('data-x');\
             globalThis.__remName = rem ? rem.name : '(null)';\
             globalThis.__hasX = String(d.hasAttribute('data-x'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__remName").unwrap().value, "data-x", "removeNamedItem('data-x') 返移除 Attr.name=data-x");
    assert_eq!(sandbox.execute("globalThis.__hasX").unwrap().value, "false", "removeNamedItem 后 hasAttribute('data-x')=false");

    // removeNamedItem 缺失 → null（best-effort 不抛 NOT_FOUND_ERR）。
    sandbox.execute("globalThis.__remMissing = String(d.attributes.removeNamedItem('nope') === null);").unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__remMissing").unwrap().value,
        "true",
        "removeNamedItem 缺失属性返 null（best-effort，不抛）"
    );
}

#[test]
fn test_namednodemap_setnameditem_mutable_tree_r3022() {
    // R3022：NamedNodeMap setNamedItem/removeNamedItem mutable tree 路径（detached document _zwMEl attrs +
    // DOMParser lazy bridge 经 _ensureMutTree 复用 _zwMEl attrs）。R3018 闭合 setAttribute/removeAttribute
    // 入树但 NamedNodeMap mutation 缺失。本切片补 _zwMEl attrs.setNamedItem/removeNamedItem，序列化反映。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // detached document _zwMEl attrs：setNamedItem 新/改 + removeNamedItem + 序列化反映。
    sandbox
        .execute(
            "var doc = document.implementation.createHTMLDocument('');\
             doc.body.innerHTML = '<div id=\"d\" class=\"c\"></div>';\
             var div = doc.body.childNodes[0];\
             var old1 = div.attributes.setNamedItem({ name: 'data-x', value: '1' });\
             var old2 = div.attributes.setNamedItem({ name: 'id', value: 'd2' });\
             var rem = div.attributes.removeNamedItem('class');\
             globalThis.__old1 = String(old1 === null);\
             globalThis.__old2Val = old2 ? old2.value : '(null)';\
             globalThis.__remName = rem ? rem.name : '(null)';\
             globalThis.__outer = div.outerHTML;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__old1").unwrap().value, "true", "mutable tree setNamedItem 新属性返 null");
    assert_eq!(sandbox.execute("globalThis.__old2Val").unwrap().value, "d", "mutable tree setNamedItem 改 id 返旧 value='d'");
    assert_eq!(sandbox.execute("globalThis.__remName").unwrap().value, "class", "mutable tree removeNamedItem('class') 返 Attr.name=class");
    let outer = sandbox.execute("globalThis.__outer").unwrap().value;
    assert!(outer.contains("data-x=\"1\""), "mutable tree 序列化含 data-x=\"1\"（setNamedItem 反映）：{outer}");
    assert!(outer.contains("id=\"d2\""), "mutable tree 序列化含 id=\"d2\"（setNamedItem 改 id）：{outer}");
    assert!(!outer.contains("class="), "mutable tree 序列化不含 class（removeNamedItem 反映）：{outer}");

    // DOMParser lazy bridge（_zwParseEl → _ensureMutTree → _zwMEl attrs）同受益。
    sandbox
        .execute(
            "var d2 = new DOMParser().parseFromString('<p title=\"t\"></p>', 'text/html');\
             var p = d2.body.childNodes[0];\
             p.attributes.setNamedItem({ name: 'data-y', value: '2' });\
             globalThis.__pOuter = p.outerHTML;",
        )
        .unwrap();
    let pouter = sandbox.execute("globalThis.__pOuter").unwrap().value;
    assert!(pouter.contains("data-y=\"2\""), "DOMParser lazy bridge setNamedItem 序列化含 data-y=\"2\"：{pouter}");
}

#[test]
fn test_create_attribute_and_attr_instance_r3023() {
    // R3023：document.createAttribute + 真 Attr 实例（闭合 R3022 限制①「返 Attr 为 plain {name,value}」）。
    // createAttribute(name) 建 Attr 节点（nodeType 2 + 全字段）；NamedNodeMap 各方法返值（getNamedItem/item/
    // setNamedItem/removeNamedItem）经 _zwMakeAttr 返真 Attr（nodeType 2 + localName/namespaceURI/prefix/
    // specified/ownerElement），非 plain object。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d' class='c'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // document.createAttribute：nodeType=2 + 全字段（name/nodeName/value/nodeValue/localName/namespaceURI=
    // null/prefix=null/specified/ownerElement=null）。
    sandbox
        .execute(
            "var a = document.createAttribute('data-x');\
             globalThis.__nt = String(a.nodeType);\
             globalThis.__name = a.name;\
             globalThis.__nodeName = a.nodeName;\
             globalThis.__val = a.value;\
             globalThis.__localName = a.localName;\
             globalThis.__ns = String(a.namespaceURI);\
             globalThis.__prefix = String(a.prefix);\
             globalThis.__specified = String(a.specified);\
             globalThis.__owner = String(a.ownerElement);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__nt").unwrap().value, "2", "createAttribute 返 Attr nodeType=2");
    assert_eq!(sandbox.execute("globalThis.__name").unwrap().value, "data-x", "Attr.name=data-x");
    assert_eq!(sandbox.execute("globalThis.__nodeName").unwrap().value, "data-x", "Attr.nodeName=data-x（同 name）");
    assert_eq!(sandbox.execute("globalThis.__val").unwrap().value, "", "createAttribute Attr.value=''（初始空）");
    assert_eq!(sandbox.execute("globalThis.__localName").unwrap().value, "data-x", "Attr.localName=data-x");
    assert_eq!(sandbox.execute("globalThis.__ns").unwrap().value, "null", "Attr.namespaceURI=null");
    assert_eq!(sandbox.execute("globalThis.__prefix").unwrap().value, "null", "Attr.prefix=null");
    assert_eq!(sandbox.execute("globalThis.__specified").unwrap().value, "true", "Attr.specified=true");
    assert_eq!(sandbox.execute("globalThis.__owner").unwrap().value, "null", "createAttribute Attr.ownerElement=null（游离）");

    // createAttribute + setNamedItem 流：a.value='v' 后 setNamedItem → 属性生效。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             a.value = 'v';\
             d.attributes.setNamedItem(a);\
             globalThis.__gv = d.getAttribute('data-x');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__gv").unwrap().value, "v", "createAttribute + setNamedItem 后 getAttribute='v'");

    // 主 DOM getNamedItem 返真 Attr（nodeType=2 + localName + ownerElement 非空）。
    sandbox
        .execute(
            "var ga = d.attributes.getNamedItem('class');\
             globalThis.__gnt = String(ga.nodeType);\
             globalThis.__glocal = ga.localName;\
             globalThis.__gownerTag = ga.ownerElement ? ga.ownerElement.tagName : '(null)';",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__gnt").unwrap().value, "2", "主 DOM getNamedItem 返 Attr nodeType=2（真实例）");
    assert_eq!(sandbox.execute("globalThis.__glocal").unwrap().value, "class", "主 DOM Attr.localName=class");
    assert_eq!(sandbox.execute("globalThis.__gownerTag").unwrap().value, "DIV", "主 DOM Attr.ownerElement=元素（tagName=DIV）");

    // mutable tree getNamedItem / setNamedItem / removeNamedItem 返真 Attr（nodeType=2）。
    sandbox
        .execute(
            "var doc = document.implementation.createHTMLDocument('');\
             doc.body.innerHTML = '<div id=\"m\"></div>';\
             var div = doc.body.childNodes[0];\
             div.setAttribute('k','vv');\
             var mga = div.attributes.getNamedItem('k');\
             var msa = div.attributes.setNamedItem({name:'k',value:'vv2'});\
             var mra = div.attributes.removeNamedItem('k');\
             globalThis.__mgaNt = String(mga.nodeType);\
             globalThis.__msaNt = String(msa.nodeType);\
             globalThis.__mraNt = String(mra.nodeType);\
             globalThis.__msaOldVal = msa.value;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__mgaNt").unwrap().value, "2", "mutable tree getNamedItem 返 Attr nodeType=2（真实例，非 plain）");
    assert_eq!(sandbox.execute("globalThis.__msaNt").unwrap().value, "2", "mutable tree setNamedItem 返旧 Attr nodeType=2");
    assert_eq!(sandbox.execute("globalThis.__mraNt").unwrap().value, "2", "mutable tree removeNamedItem 返移除 Attr nodeType=2");
    assert_eq!(sandbox.execute("globalThis.__msaOldVal").unwrap().value, "vv", "mutable tree setNamedItem 返旧 Attr.value='vv'");
}

#[test]
fn test_attr_instanceof_and_namespace_attrs_r3024() {
    // R3024：Attr 构造器占位（instanceof Attr）+ 命名空间属性族（setAttributeNS/getAttributeNS/
    // hasAttributeNS/removeAttributeNS）+ createAttributeNS。闭合 R3023 限制①（Attr instanceof）②（namespace 属性）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // createAttribute + getNamedItem 返值 instanceof Attr（R3023 限制①）。
    sandbox
        .execute(
            "var a = document.createAttribute('data-x');\
             var d = document.getElementById('d');\
             d.setAttribute('class','c');\
             var ga = d.attributes.getNamedItem('class');\
             globalThis.__aInst = String(a instanceof Attr);\
             globalThis.__gaInst = String(ga instanceof Attr);",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__aInst").unwrap().value, "true", "createAttribute 返值 instanceof Attr（Object.create(Attr.prototype)）");
    assert_eq!(sandbox.execute("globalThis.__gaInst").unwrap().value, "true", "getNamedItem 返值 instanceof Attr（真 Attr 实例，非 plain）");

    // setAttributeNS / getAttributeNS / hasAttributeNS / removeAttributeNS round-trip（null ns + 简单名）。
    sandbox
        .execute(
            "d.setAttributeNS(null, 'data-ns', 'v1');\
             globalThis.__nsGet = d.getAttributeNS(null, 'data-ns');\
             globalThis.__nsHas = String(d.hasAttributeNS(null, 'data-ns'));\
             globalThis.__plainGet = d.getAttribute('data-ns');",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__nsGet").unwrap().value, "v1", "setAttributeNS(null,'data-ns','v1') 后 getAttributeNS='v1'");
    assert_eq!(sandbox.execute("globalThis.__nsHas").unwrap().value, "true", "hasAttributeNS(null,'data-ns')=true");
    assert_eq!(sandbox.execute("globalThis.__plainGet").unwrap().value, "v1", "setAttributeNS 经 setAttribute 落地（getAttribute 同值）");

    // removeAttributeNS → hasAttribute false。
    sandbox
        .execute(
            "d.removeAttributeNS(null, 'data-ns');\
             globalThis.__nsHasAfter = String(d.hasAttributeNS(null, 'data-ns'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__nsHasAfter").unwrap().value, "false", "removeAttributeNS 后 hasAttributeNS=false");

    // R3217：xlink 命名空间 setAttributeNS('xlink:href') ↔ getAttributeNS('href') 一致性。旧实现读 local 'href'
    // 查不到存为 'xlink:href' 的属性（spec 按 ns+local 匹配，本 shim ns→prefix 重构限定名）。SVG 图标库高频。
    sandbox
        .execute(
            "d.setAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href', '#icon');\
             globalThis.__xlinkGet = d.getAttributeNS('http://www.w3.org/1999/xlink', 'href');\
             globalThis.__xlinkHas = String(d.hasAttributeNS('http://www.w3.org/1999/xlink', 'href'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__xlinkGet").unwrap().value, "#icon", "setAttributeNS(xlink,'xlink:href','#icon') 后 getAttributeNS(xlink,'href')='#icon'");
    assert_eq!(sandbox.execute("globalThis.__xlinkHas").unwrap().value, "true", "hasAttributeNS(xlink,'href')=true");
    // removeAttributeNS(xlink,'href') 真移除。
    sandbox
        .execute(
            "d.removeAttributeNS('http://www.w3.org/1999/xlink', 'href');\
             globalThis.__xlinkHasAfter = String(d.hasAttributeNS('http://www.w3.org/1999/xlink', 'href'));",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__xlinkHasAfter").unwrap().value, "false", "removeAttributeNS(xlink,'href') 后 hasAttributeNS=false");

    // createAttributeNS：解析 prefix:local + namespaceURI（SVG xlink:href 用法）。
    sandbox
        .execute(
            "var na = document.createAttributeNS('http://www.w3.org/1999/xlink', 'xlink:href');\
             globalThis.__naInst = String(na instanceof Attr);\
             globalThis.__naNs = na.namespaceURI;\
             globalThis.__naPrefix = na.prefix;\
             globalThis.__naLocal = na.localName;\
             globalThis.__naName = na.name;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__naInst").unwrap().value, "true", "createAttributeNS 返值 instanceof Attr");
    assert_eq!(
        sandbox.execute("globalThis.__naNs").unwrap().value,
        "http://www.w3.org/1999/xlink",
        "createAttributeNS 设 namespaceURI=xlink ns"
    );
    assert_eq!(sandbox.execute("globalThis.__naPrefix").unwrap().value, "xlink", "解析 qualifiedName prefix=xlink");
    assert_eq!(sandbox.execute("globalThis.__naLocal").unwrap().value, "href", "解析 qualifiedName localName=href");
    assert_eq!(sandbox.execute("globalThis.__naName").unwrap().value, "xlink:href", "Attr.name=完整 qualifiedName");
}

#[test]
fn test_mutation_observer_attr_filter_and_old_value_r3025() {
    // R3025：MutationObserver attributeFilter + attributeOldValue（observe options spec 合规）。
    // 旧 _mo_notify 仅按 opts.attributes/childList 过滤——attributeFilter（仅观测列表内属性）+ attributeOldValue
    // （rec.oldValue 报告旧值）未支持，框架库按 spec 用这两个 option 时记录错误/冗余。本切片闭合。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='d'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① attributeFilter：仅观测列表内属性（data-x fire，class 被滤掉）。
    sandbox
        .execute(
            "var d = document.getElementById('d');\
             var mo = new MutationObserver(function(){});\
             mo.observe(d, { attributes: true, attributeFilter: ['data-x'] });\
             d.setAttribute('data-x', '1');\
             d.setAttribute('class', 'c');\
             globalThis.__fRecs = mo.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__fRecs.length)").unwrap().value,
        "1",
        "attributeFilter=['data-x']：仅 data-x 变更记录（class 被滤掉）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__fRecs[0].attributeName").unwrap().value,
        "data-x",
        "attributeFilter 命中记录的 attributeName=data-x"
    );

    // ② attributeOldValue：新属性 oldValue=null；后续变更 oldValue=前值。
    sandbox
        .execute(
            "mo.disconnect();\
             var mo2 = new MutationObserver(function(){});\
             mo2.observe(d, { attributes: true, attributeOldValue: true });\
             d.setAttribute('data-z', 'v1');\
             globalThis.__r1 = mo2.takeRecords();\
             d.setAttribute('data-z', 'v2');\
             globalThis.__r2 = mo2.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r1[0].oldValue)").unwrap().value,
        "null",
        "attributeOldValue：新属性 oldValue=null（absent）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r2[0].oldValue").unwrap().value,
        "v1",
        "attributeOldValue：变更 oldValue=前值 'v1'"
    );

    // ③ 未请求 attributeOldValue → oldValue 恒 null（spec）。
    sandbox
        .execute(
            "mo2.disconnect();\
             var mo3 = new MutationObserver(function(){});\
             mo3.observe(d, { attributes: true });\
             d.setAttribute('data-z', 'v3');\
             globalThis.__r3 = mo3.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__r3[0].oldValue)").unwrap().value,
        "null",
        "未请求 attributeOldValue → oldValue=null（即使有旧值）"
    );

    // ④ attributeFilter 命中亦报告 oldValue（spec：filter 命中 == 请求 old value），无须 attributeOldValue。
    sandbox
        .execute(
            "mo3.disconnect();\
             var mo4 = new MutationObserver(function(){});\
             mo4.observe(d, { attributes: true, attributeFilter: ['data-z'] });\
             d.setAttribute('data-z', 'v4');\
             globalThis.__r4 = mo4.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r4[0].oldValue").unwrap().value,
        "v3",
        "attributeFilter 命中报告 oldValue='v3'（spec：filter 命中 == 请求 old value）"
    );
}

#[test]
fn test_mutation_observer_subtree_r3026() {
    // R3026：MutationObserver subtree（ancestor 解析）。observe(container,{...,subtree:true}) 时后代 mutation
    // 冒泡到 container observer（record.target=container）；非 subtree observer 不收后代 mutation。框架「观测整个子树」
    // 第一高频用法。经 _ancestorChain（__zw_parent 父链）上行，_mo_any_subtree guard 无 subtree observer 时零开销。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='container'><div id='inner'><span id='leaf'></span></div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① subtree childList：后代（leaf）appendChild → container observer 收记录（target=container）。
    sandbox
        .execute(
            "var container = document.getElementById('container');\
             var leaf = document.getElementById('leaf');\
             var mo = new MutationObserver(function(){});\
             mo.observe(container, { childList: true, subtree: true });\
             leaf.appendChild(document.createElement('b'));\
             globalThis.__recs = mo.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__recs.length)").unwrap().value,
        "1",
        "subtree childList：后代 leaf appendChild → container observer 收 1 记录"
    );
    assert_eq!(
        sandbox.execute("globalThis.__recs[0].type").unwrap().value,
        "childList",
        "subtree 记录 type=childList"
    );
    assert_eq!(
        sandbox.execute("globalThis.__recs[0].target.id").unwrap().value,
        "container",
        "subtree 记录 target=container（祖先 observer 的 target，非 leaf）"
    );

    // ② 非 subtree observer 不收后代 mutation（仅收 container 自身直接 childList）。
    sandbox
        .execute(
            "mo.disconnect();\
             var mo2 = new MutationObserver(function(){});\
             mo2.observe(container, { childList: true });\
             leaf.appendChild(document.createElement('i'));\
             globalThis.__recs2 = mo2.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__recs2.length)").unwrap().value,
        "0",
        "非 subtree observer 不收后代 leaf 的 childList mutation（仅 container 直接子）"
    );

    // ③ subtree attributes：后代 leaf.setAttribute → container observer 收记录（target=container）。
    sandbox
        .execute(
            "mo2.disconnect();\
             var mo3 = new MutationObserver(function(){});\
             mo3.observe(container, { attributes: true, subtree: true });\
             leaf.setAttribute('data-x', '1');\
             globalThis.__recs3 = mo3.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__recs3.length)").unwrap().value,
        "1",
        "subtree attributes：后代 leaf setAttribute → container observer 收 1 记录"
    );
    assert_eq!(
        sandbox.execute("globalThis.__recs3[0].attributeName").unwrap().value,
        "data-x",
        "subtree 属性记录 attributeName=data-x"
    );
    assert_eq!(
        sandbox.execute("globalThis.__recs3[0].target.id").unwrap().value,
        "container",
        "subtree 属性记录 target=container"
    );

    // ④ 子树内多层深度（leaf 在 inner 在 container）：仍冒泡到 container（ancestor 链长度无关）。
    sandbox
        .execute(
            "mo3.disconnect();\
             var mo4 = new MutationObserver(function(){});\
             mo4.observe(container, { childList: true, subtree: true });\
             leaf.appendChild(document.createElement('u'));\
             globalThis.__recs4 = mo4.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__recs4.length)").unwrap().value,
        "1",
        "多层深度（leaf←inner←container）subtree 仍冒泡到 container"
    );
}

#[test]
fn test_mutation_observer_character_data_r3027() {
    // R3027：MutationObserver characterData emission + characterDataOldValue。textContent 变更发射 characterData
    // 记录（target=元素，pragmatic——文本节点无 selector 不能直接作 target）；observe(el,{characterData,subtree})
    // + 后代 textContent 经 ancestor 冒泡覆盖。characterDataOldValue 请求时 rec.oldValue=旧文本。闭合 MO 最后类型。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='a'></div><div id='container'><span id='leaf'></span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // ① 直接观测：observe(a,{characterData,subtree}) + a.textContent → characterData 记录（target=a）。
    sandbox
        .execute(
            "var a = document.getElementById('a');\
             var mo = new MutationObserver(function(){});\
             mo.observe(a, { characterData: true, subtree: true });\
             a.textContent = 'hello';\
             globalThis.__recs = mo.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__recs.length)").unwrap().value,
        "1",
        "characterData：a.textContent 变更 → 1 记录"
    );
    assert_eq!(
        sandbox.execute("globalThis.__recs[0].type").unwrap().value,
        "characterData",
        "记录 type=characterData"
    );
    assert_eq!(
        sandbox.execute("globalThis.__recs[0].target.id").unwrap().value,
        "a",
        "characterData 记录 target=a（元素，pragmatic）"
    );

    // ② subtree 后代：observe(container,{characterData,subtree}) + leaf.textContent → 冒泡到 container。
    sandbox
        .execute(
            "mo.disconnect();\
             var container = document.getElementById('container');\
             var leaf = document.getElementById('leaf');\
             var mo2 = new MutationObserver(function(){});\
             mo2.observe(container, { characterData: true, subtree: true });\
             leaf.textContent = 'deep';\
             globalThis.__recs2 = mo2.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__recs2.length)").unwrap().value,
        "1",
        "subtree characterData：后代 leaf.textContent → container observer 收 1 记录"
    );
    assert_eq!(
        sandbox.execute("globalThis.__recs2[0].target.id").unwrap().value,
        "container",
        "subtree characterData 记录 target=container（经 ancestor 冒泡）"
    );

    // ③ 未观测 characterData → textContent 变更不产 characterData 记录。
    sandbox
        .execute(
            "mo2.disconnect();\
             var mo3 = new MutationObserver(function(){});\
             mo3.observe(leaf, { attributes: true });\
             leaf.textContent = 'third';\
             globalThis.__recs3 = mo3.takeRecords();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__recs3.length)").unwrap().value,
        "0",
        "未观测 characterData → textContent 变更不产记录（仅 attributes 观测）"
    );
}
