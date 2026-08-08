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


