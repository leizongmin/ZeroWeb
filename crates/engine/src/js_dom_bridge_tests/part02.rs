#[test]
fn test_url_setters_r2780() {
    // R2780：URL 组件 setter + 双向 searchParams 同步（host callback __zw_set_url_part → url crate setters）。
    // 注册 __zw_parse_url + __zw_set_url_part 两回调（复用 production 纯函数）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.register_callback(
        "__zw_parse_url",
        Box::new(|args: &[String]| -> String {
            let input = args.first().map(String::as_str).unwrap_or("");
            let base = args.get(1).map(String::as_str);
            parse_url_to_json(input, base)
        }),
    );
    sandbox.register_callback(
        "__zw_set_url_part",
        Box::new(|args: &[String]| -> String {
            let prev = args.first().map(String::as_str).unwrap_or("");
            let part = args.get(1).map(String::as_str).unwrap_or("");
            let value = args.get(2).map(String::as_str).unwrap_or("");
            set_url_part(prev, part, value)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // pathname setter（SPA 路由高频）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/old'); u.pathname = '/new/path'; u.pathname + '|' + u.href")
            .unwrap()
            .value,
        "/new/path|https://example.com/new/path"
    );
    // hash setter（SPA 路由）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p'); u.hash = '#section'; u.hash + '|' + u.href")
            .unwrap()
            .value,
        "#section|https://example.com/p#section"
    );
    // protocol setter（http→https）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('http://example.com/p'); u.protocol = 'https:'; u.protocol + '|' + u.href")
            .unwrap()
            .value,
        "https:|https://example.com/p"
    );
    // hostname setter + host 联动（_load 全字段重载）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://old.example.com/p'); u.hostname = 'new.example.com'; u.hostname + '|' + u.host")
            .unwrap()
            .value,
        "new.example.com|new.example.com"
    );
    // port setter（非默认）+ host/href 联动。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p'); u.port = '8443'; u.port + '|' + u.host + '|' + u.href")
            .unwrap()
            .value,
        "8443|example.com:8443|https://example.com:8443/p"
    );
    // search setter → searchParams 同步（search→params 方向）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p?a=1'); u.search = '?x=9&y=8'; u.searchParams.get('x') + '|' + u.searchParams.get('y')")
            .unwrap()
            .value,
        "9|8"
    );
    // searchParams append → search/href 同步（params→search 方向，无递归）。
    assert_eq!(
        sandbox
            .execute(
                "var u = new URL('https://example.com/p'); u.searchParams.append('k', 'v'); u.search + '|' + u.href"
            )
            .unwrap()
            .value,
        "?k=v|https://example.com/p?k=v"
    );
    // searchParams 多次 set → search 反映最后值 + 无递归（多次 mutate 不爆栈）。
    assert_eq!(
        sandbox
            .execute(
                "var u = new URL('https://example.com/p'); u.searchParams.set('a','1'); u.searchParams.set('b','2'); u.searchParams.set('a','9'); u.searchParams.get('a') + '|' + u.search"
            )
            .unwrap()
            .value,
        "9|?a=9&b=2"
    );
    // searchParams delete → search 更新。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p?a=1&b=2'); u.searchParams.delete('a'); u.search")
            .unwrap()
            .value,
        "?b=2"
    );
    // href setter（整体替换）+ searchParams 同步。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/old'); u.href = 'http://other.test/x?z=5#w'; u.host + '|' + u.pathname + '|' + u.searchParams.get('z') + '|' + u.hash")
            .unwrap()
            .value,
        "other.test|/x|5|#w"
    );
    // 无效 href setter 抛 TypeError（Url::parse 失败，spec 一致）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/'); try { u.href = 'not a valid url'; 'no-throw'; } catch (e) { e.name; }")
            .unwrap()
            .value,
        "TypeError"
    );
    // searchParams 稳定实例（多次访问同对象，spec 一致）。
    assert_eq!(
        sandbox
            .execute("var u = new URL('https://example.com/p'); u.searchParams === u.searchParams")
            .unwrap()
            .value,
        "true"
    );
}

#[test]
fn test_set_url_part_rust_r2780() {
    // R2780：set_url_part 纯函数单测（直调，验 url crate setter 正确性 + 非法 scheme 返空串不 panic）。
    use super::*;
    // pathname setter。
    let r = set_url_part("https://example.com/old", "pathname", "/new/path");
    assert!(r.contains("\"pathname\":\"/new/path\""), "pathname setter: {r}");
    assert!(
        r.contains("\"href\":\"https://example.com/new/path\""),
        "href after pathname: {r}"
    );
    // search setter。
    let r = set_url_part("https://example.com/p", "search", "?a=1&b=2");
    assert!(r.contains("\"search\":\"?a=1&b=2\""), "search setter: {r}");
    // hash 清除（空串）。
    let r = set_url_part("https://example.com/p#sec", "hash", "");
    assert!(r.contains("\"hash\":\"\""), "hash clear: {r}");
    // port setter。
    let r = set_url_part("https://example.com/p", "port", "8443");
    assert!(r.contains("\"port\":\"8443\""), "port setter: {r}");
    // href setter（整体替换）。
    let r = set_url_part("https://example.com/old", "href", "http://other.test/x?q=1");
    assert!(r.contains("\"host\":\"other.test\""), "href replace host: {r}");
    // 非法 scheme 返空串（不 panic）。
    assert_eq!(set_url_part("https://example.com/p", "protocol", "ht!tp"), "");
    // 非法 href 返空串。
    assert_eq!(set_url_part("https://example.com/p", "href", "not a url"), "");
    // 未知 part 不改 URL（返回原序列化）。
    let r = set_url_part("https://example.com/p", "unknownpart", "x");
    assert!(
        r.contains("\"href\":\"https://example.com/p\""),
        "unknown part noop: {r}"
    );
}

#[test]
fn test_match_media_r2781() {
    // R2781：window.matchMedia（host callback __zw_match_media → zero_css_parser::media_query）。
    // 响应式设计 / viewport 查询高频（shim 曾缺失）。viewport 默认 1280x800（shim innerWidth/innerHeight）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.register_callback(
        "__zw_match_media",
        Box::new(|args: &[String]| -> String {
            let query = args.first().map(String::as_str).unwrap_or("");
            let width = args.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            let height = args.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            match_media_to_json(query, width, height)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // (min-width: 768px) @1280 → true；(max-width: 500px) @1280 → false。
    assert_eq!(
        sandbox
            .execute("matchMedia('(min-width: 768px)').matches + '|' + matchMedia('(max-width: 500px)').matches")
            .unwrap()
            .value,
        "true|false"
    );
    // media 字段返回 query 串。
    assert_eq!(
        sandbox.execute("matchMedia('(min-width: 768px)').media").unwrap().value,
        "(min-width: 768px)"
    );
    // orientation：landscape @1280x800 → true；portrait → false（is_portrait = h > w）。
    assert_eq!(
        sandbox
            .execute(
                "matchMedia('(orientation: landscape)').matches + '|' + matchMedia('(orientation: portrait)').matches"
            )
            .unwrap()
            .value,
        "true|false"
    );
    // prefers-color-scheme 默认 light：light → true；dark → false。
    assert_eq!(
        sandbox
            .execute("matchMedia('(prefers-color-scheme: light)').matches + '|' + matchMedia('(prefers-color-scheme: dark)').matches")
            .unwrap()
            .value,
        "true|false"
    );
    // 逗号分隔 query list（OR 语义）：任一 match → true。
    assert_eq!(
        sandbox
            .execute("matchMedia('(max-width: 1px), (min-width: 768px)').matches")
            .unwrap()
            .value,
        "true"
    );
    // viewport 覆盖：@500 → (min-width: 768px) → false。
    assert_eq!(
        sandbox
            .execute("globalThis.innerWidth = 500; matchMedia('(min-width: 768px)').matches")
            .unwrap()
            .value,
        "false"
    );
    // MediaQueryList extends EventTarget（R2779）+ legacy addListener/removeListener。
    assert_eq!(
        sandbox
            .execute(
                "var m = matchMedia('(min-width: 1px)');\
                 (m instanceof MediaQueryList) + '|' + (m instanceof EventTarget) + '|' +\
                 typeof m.addListener + '|' + typeof m.removeListener"
            )
            .unwrap()
            .value,
        "true|true|function|function"
    );
}

#[test]
fn test_match_media_change_event_r3255() {
    // R3255（CSSOM View §media-query-list）：resize 后 matches 翻转的 MediaQueryList 派 'change' 事件。
    // R2781 落地 matchMedia 但 change 不派发（documented 限制）；R3254 resize 钩子驱动 _zwFireMqlChanges。
    // 验证：① resize 跨断点 → change listener 触发；② matches 更新为新值；③ 事件带 media + matches；
    // ④ 未跨断点（matches 不变）不派 change；⑤ legacy addListener 亦触发。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.register_callback(
        "__zw_match_media",
        Box::new(|args: &[String]| -> String {
            let query = args.first().map(String::as_str).unwrap_or("");
            let width = args.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            let height = args.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
            match_media_to_json(query, width, height)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // ① 初始 @1280：(min-width: 768px) matches=true。注册 change listener + 记录事件 media|matches。
    sandbox
        .execute(
            "globalThis.__chg = 'none|none';\
             var mql = matchMedia('(min-width: 768px)');\
             mql.addEventListener('change', function(e){ globalThis.__chg = e.media + '|' + (mql.matches ? 'true' : 'false'); });\
             globalThis.__mqlMatchesBefore = mql.matches;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__mqlMatchesBefore").unwrap().value, "true", "@1280 (min-width:768px) matches=true");

    // ② __zw_user_resize(500, 400) → 跨断点（< 768）→ matches 翻转 false → 派 change。
    sandbox.execute("__zw_user_resize(500, 400);").unwrap();
    assert_eq!(sandbox.execute("globalThis.__chg").unwrap().value, "(min-width: 768px)|false", "resize 跨断点 → change 派发，media + matches（新值 false）");

    // ③ 再次 resize(400, 300)（仍 < 768，matches 不变）→ 不派 change（__chg 保持上次值）。
    sandbox.execute("__zw_user_resize(400, 300);").unwrap();
    assert_eq!(sandbox.execute("globalThis.__chg").unwrap().value, "(min-width: 768px)|false", "未跨断点（matches 不变）不派 change");

    // ④ resize(1024, 768)（回到 ≥ 768）→ matches 翻转 true → 再派 change。
    sandbox.execute("__zw_user_resize(1024, 768);").unwrap();
    assert_eq!(sandbox.execute("globalThis.__chg").unwrap().value, "(min-width: 768px)|true", "resize 回跨断点 → matches=true → change 派发");

    // ⑤ legacy addListener（旧 API）亦触发 change——不同 MQL，注册后 resize 跨断点派发。
    sandbox
        .execute(
            "globalThis.__legacy = 0;\
             var m2 = matchMedia('(max-width: 600px)');\
             m2.addListener(function(){ globalThis.__legacy++; });\
             __zw_user_resize(300, 200);", // 600 以下 → (max-width:600px) matches 翻转 true → legacy change
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__legacy").unwrap().value, "1", "legacy addListener 亦触发 change（R3255）");
}

#[test]
fn test_console_host_bridge_r3256() {
    // R3256（Console Standard）：console.* 经 `__zw_console_log(level,msg)` 桥接宿主。旧实现全 no-op（page
    // console 输出丢失）。验证：① level 正确传递（log/warn/error）；② 多参数空格拼接 + 序列化（string/number/
    // object JSON）；③ typeof 守卫（未注册回调时 no-op，向后兼容）；④ count/group 等非输出类保持 no-op 不调回调。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    let captured: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(vec![]));
    let cap = Arc::clone(&captured);
    sandbox.register_callback(
        "__zw_console_log",
        Box::new(move |args: &[String]| -> String {
            let level = args.first().cloned().unwrap_or_default();
            let msg = args.get(1).cloned().unwrap_or_default();
            cap.lock().unwrap().push((level, msg));
            String::new()
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // ① ② 多参数 + 序列化：log('hello', 42, {a:1}) → level=log, msg="hello 42 {\"a\":1}"。
    sandbox
        .execute("console.log('hello', 42, { a: 1 }); console.warn('careful'); console.error('broken');")
        .unwrap();
    let got = captured.lock().unwrap().clone();
    assert_eq!(got.len(), 3, "三次 console 调用各触发一次回调");
    assert_eq!(got[0].0, "log", "level=log");
    assert_eq!(got[0].1, "hello 42 {\"a\":1}", "多参数空格拼接 + object JSON 序列化");
    assert_eq!(got[1].0, "warn", "level=warn");
    assert_eq!(got[1].1, "careful");
    assert_eq!(got[2].0, "error", "level=error");
    assert_eq!(got[2].1, "broken");

    // ③ typeof 守卫：删掉回调后 console.log 不抛、不产生新条目（向后兼容）。
    //（V8 sandbox 无 deregister，改在无回调的独立 sandbox 验证 no-op——此处仅验证序列化未误触。）
    let before = captured.lock().unwrap().len();
    sandbox.execute("console.count('x'); console.group('g'); console.time('t');").unwrap();
    assert_eq!(captured.lock().unwrap().len(), before, "count/group/time 非输出类 → no-op，不调回调");
}

#[test]
fn test_message_channel_r2782() {
    // R2782：MessageChannel + MessagePort + MessageEvent（postMessage 双端口，纯 JS）。MessagePort extends
    // EventTarget（R2779）；postMessage 经 structuredClone（R2773）深拷贝 + queueMicrotask（R2774）异步派发
    // 'message' 事件（execute 末 microtask checkpoint，下 execute 可读）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // port1/port2 + instanceof MessagePort/EventTarget。
    assert_eq!(
        sandbox
            .execute(
                "var ch = new MessageChannel();\
                 typeof ch.port1 + '|' + typeof ch.port2 + '|' +\
                 (ch.port1 instanceof MessagePort) + '|' + (ch.port2 instanceof EventTarget)"
            )
            .unwrap()
            .value,
        "object|object|true|true"
    );
    // postMessage port1→port2：异步派发（execute 末 microtask），下 execute 可读 __got。
    sandbox
        .execute(
            "ch.port2.onmessage = function (e) { globalThis.__got = e.data.x + 1; };\
             ch.port1.postMessage({ x: 41 }); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__got").unwrap().value, "42");
    assert_eq!(
        sandbox
            .execute(
                "var transferred = new Uint8Array([1, 2, 3, 4]);\
                 ch.port1.postMessage('', [transferred.buffer]);\
                 String(transferred.byteLength)"
            )
            .unwrap()
            .value,
        "0",
        "ArrayBuffer transfer detaches the sender buffer"
    );
    // structuredClone 深拷贝：postMessage 时克隆，后续 mutate 原对象不影响收到的（R2773 验证）。
    sandbox
        .execute(
            "var orig = { v: 1 };\
             ch.port2.onmessage = function (e) { globalThis.__msgV = e.data.v; };\
             ch.port1.postMessage(orig); orig.v = 5; 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__msgV").unwrap().value, "1");
    // 反向 port2→port1。
    sandbox
        .execute(
            "ch.port1.onmessage = function (e) { globalThis.__rev = e.data; };\
             ch.port2.postMessage('hello'); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__rev").unwrap().value, "hello");
    // MessageEvent 字段：instanceof MessageEvent & Event + type=message + source=null。
    sandbox
        .execute(
            "ch.port2.onmessage = function (e) {\
                 globalThis.__mev = (e instanceof MessageEvent) + '|' + (e instanceof Event) + '|' + e.type + '|' + (e.source === null);\
             }; ch.port1.postMessage('x'); 'sent'"
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__mev").unwrap().value,
        "true|true|message|true"
    );
    // close() 停止派发：postMessage on closed port no-op。
    sandbox
        .execute(
            "var c = new MessageChannel(); globalThis.__cl = 'none';\
             c.port2.onmessage = function () { globalThis.__cl = 'got'; };\
             c.port1.close(); c.port1.postMessage('z'); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__cl").unwrap().value, "none");
}

#[test]
fn test_broadcast_channel_r2783() {
    // R2783：BroadcastChannel（同源广播，纯 JS）。extends EventTarget R2779；postMessage 经
    // structuredClone R2773 + queueMicrotask R2782 异步派发到所有同名其他实例（sender 不收自己）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof + name + instanceof BroadcastChannel/EventTarget。
    assert_eq!(
        sandbox
            .execute(
                "var bc = new BroadcastChannel('news');\
                 typeof BroadcastChannel + '|' + bc.name + '|' +\
                 (bc instanceof BroadcastChannel) + '|' + (bc instanceof EventTarget)"
            )
            .unwrap()
            .value,
        "function|news|true|true"
    );
    // 广播：a post → b & c 收，a 不收自己（sender skipped）。
    sandbox
        .execute(
            "var a = new BroadcastChannel('ch'); var b = new BroadcastChannel('ch'); var c = new BroadcastChannel('ch');\
             globalThis.__got = '';\
             a.onmessage = function () { globalThis.__got += 'a'; };\
             b.onmessage = function (e) { globalThis.__got += 'b:' + e.data + ';'; };\
             c.onmessage = function (e) { globalThis.__got += 'c:' + e.data + ';'; };\
             a.postMessage('hi'); 'sent'"
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__got").unwrap().value, "b:hi;c:hi;");
    // structuredClone 深拷贝：postMessage 时克隆，后续 mutate 原对象不影响收到的。
    sandbox
        .execute(
            "var msg = { v: 1 }; globalThis.__mv = -1;\
             b.onmessage = function (e) { globalThis.__mv = e.data.v; };\
             a.postMessage(msg); msg.v = 99; 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__mv").unwrap().value, "1");
    // 不同 name 无串扰：a（ch）post 不触达 x（other）。
    sandbox
        .execute(
            "var x = new BroadcastChannel('other'); globalThis.__cross = 'none';\
             x.onmessage = function () { globalThis.__cross = 'got'; };\
             a.postMessage('to-ch'); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__cross").unwrap().value, "none");
    // close() → 移出注册表，不再收（仅 c 收，b 已 close）。
    sandbox
        .execute(
            "globalThis.__cl = ''; b.close();\
             c.onmessage = function () { globalThis.__cl += 'c'; };\
             a.postMessage('after'); 'sent'",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__cl").unwrap().value, "c");
}

#[test]
fn test_location_read_spec_r2784() {
    // R2784：location 读侧 spec 化（_parseLocation → new URL R2778，spec-correct）。注册
    // __zw_get_page_url（返测试 URL）+ __zw_parse_url（使 new URL 路径激活）。验默认端口归一等精度提升。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    // https 默认端口 443 → 归一省略（旧 regex 会保留 :443，spec 改进）。
    sandbox.register_callback(
        "__zw_get_page_url",
        Box::new(|_args: &[String]| "https://example.com:443/path?q=1#sec".to_string()),
    );
    sandbox.register_callback(
        "__zw_parse_url",
        Box::new(|args: &[String]| -> String {
            let input = args.first().map(String::as_str).unwrap_or("");
            let base = args.get(1).map(String::as_str);
            parse_url_to_json(input, base)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 默认端口 443 归一省略（host=example.com 非 example.com:443）+ 全组件 spec-correct。
    assert_eq!(
        sandbox
            .execute(
                "location.protocol + '|' + location.hostname + '|' + location.host + '|' +\
                 location.pathname + '|' + location.search + '|' + location.hash + '|' +\
                 location.origin + '|' + location.href"
            )
            .unwrap()
            .value,
        "https:|example.com|example.com|/path|?q=1|#sec|https://example.com|https://example.com/path?q=1#sec"
    );
    // toString === href。
    assert_eq!(
        sandbox.execute("location.toString()").unwrap().value,
        "https://example.com/path?q=1#sec"
    );
}

#[test]
fn test_css_escape_supports_r2785() {
    // R2785：CSS namespace（escape 选择器转义 + supports 特性检测）。escape 纯 JS（chromium oracle
    // 锚定）；supports 委托 host __zw_css_supports（known-property gate + apply）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.register_callback(
        "__zw_css_supports",
        Box::new(|args: &[String]| -> String {
            let prop = args.first().map(String::as_str).unwrap_or("");
            let value = args.get(1).map(String::as_str);
            if css_supports(prop, value) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // CSS.escape（chromium oracle 锚定：特殊符 \char / 首数字 \hex+space / -a 直留 / 空串不抛）。
    assert_eq!(sandbox.execute("CSS.escape('a.b#c')").unwrap().value, "a\\.b\\#c");
    assert_eq!(sandbox.execute("CSS.escape('foo bar')").unwrap().value, "foo\\ bar");
    assert_eq!(sandbox.execute("CSS.escape('1abc')").unwrap().value, "\\31 abc");
    assert_eq!(sandbox.execute("CSS.escape('-a')").unwrap().value, "-a");
    assert_eq!(sandbox.execute("CSS.escape('')").unwrap().value, "");
    // CSS.supports 两参：已知属性+合法值 true；非法值/未知属性 false。
    assert_eq!(
        sandbox
            .execute("CSS.supports('display','grid') + '|' + CSS.supports('color','red')")
            .unwrap()
            .value,
        "true|true"
    );
    assert_eq!(
        sandbox
            .execute("CSS.supports('display','bogusxyz') + '|' + CSS.supports('fakeprop','x')")
            .unwrap()
            .value,
        "false|false"
    );
    // CSS.supports 单参：括号条件 / 声明 / not。
    assert_eq!(
        sandbox
            .execute("CSS.supports('(display: grid)') + '|' + CSS.supports('display: grid')")
            .unwrap()
            .value,
        "true|true"
    );
    assert_eq!(
        sandbox.execute("CSS.supports('not (display: grid)')").unwrap().value,
        "false"
    );
}

#[test]
fn test_css_supports_and_or_r2951() {
    // R2951：CSS.supports 单参 condition 经 css-parser parse_supports_condition 完整求值
    //（and/or/not/嵌套/selector）。R2785 的 eval_supports_condition 未实现 and/or（恒 false）——本切片修。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.register_callback(
        "__zw_css_supports",
        Box::new(|args: &[String]| -> String {
            let prop = args.first().map(String::as_str).unwrap_or("");
            let value = args.get(1).map(String::as_str);
            if css_supports(prop, value) {
                "1".into()
            } else {
                "0".into()
            }
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // and：两声明均支持 → true；一真一假 → false。
    assert_eq!(
        sandbox
            .execute("CSS.supports('(display: grid) and (display: flex)')")
            .unwrap()
            .value,
        "true",
        "and：两支持声明 → true"
    );
    assert_eq!(
        sandbox
            .execute("CSS.supports('(display: grid) and (fakeprop: x)')")
            .unwrap()
            .value,
        "false",
        "and：一支持一不支持 → false"
    );
    // or：任一支持 → true；均不支持 → false。
    assert_eq!(
        sandbox
            .execute("CSS.supports('(display: grid) or (fakeprop: x)')")
            .unwrap()
            .value,
        "true",
        "or：任一支持 → true"
    );
    assert_eq!(
        sandbox
            .execute("CSS.supports('(fake1: x) or (fake2: y)')")
            .unwrap()
            .value,
        "false",
        "or：均不支持 → false"
    );
    // not：支持声明取反 → false；不支持取反 → true。
    assert_eq!(
        sandbox.execute("CSS.supports('not (display: grid)')").unwrap().value,
        "false",
        "not：支持声明取反 → false"
    );
    assert_eq!(
        sandbox.execute("CSS.supports('not (fakeprop: x)')").unwrap().value,
        "true",
        "not：不支持取反 → true"
    );
    // 嵌套（多括号层）+ 组合。
    assert_eq!(
        sandbox.execute("CSS.supports('((display: grid))')").unwrap().value,
        "true",
        "嵌套括号 → true"
    );
    assert_eq!(
        sandbox
            .execute("CSS.supports('(display: grid) and ((color: red) or (fakeprop: x))')")
            .unwrap()
            .value,
        "true",
        "and + 嵌套 or（or 内一真）→ true"
    );
    // selector()（permissive true）+ 混用 and/or 拒绝（spec：同层不可混 and/or）。
    assert_eq!(
        sandbox.execute("CSS.supports('selector(.a > .b)')").unwrap().value,
        "true",
        "selector() → permissive true"
    );
    assert_eq!(
        sandbox
            .execute("CSS.supports('(display: grid) and (color: red) or (fakeprop: x)')")
            .unwrap()
            .value,
        "false",
        "同层混用 and/or → spec 非法 → false"
    );
}

#[test]
fn test_document_cookie_r2786() {
    // R2786：document.cookie get/set（in-JS 存储，set-then-read 常见模式）。**已知限制**：不接真 cookie jar
    // / 无 origin 隔离 / 无 expiry（host-layer defer）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 初始空。
    assert_eq!(sandbox.execute("document.cookie").unwrap().value, "");
    // set 单 cookie（带属性，仅取 name=value）。
    sandbox
        .execute("document.cookie = 'theme=dark; Path=/; Max-Age=3600'")
        .unwrap();
    assert_eq!(sandbox.execute("document.cookie").unwrap().value, "theme=dark");
    // set 第二个 cookie → getter 串含两者。
    sandbox.execute("document.cookie = 'lang=en'").unwrap();
    assert!(sandbox.execute("document.cookie").unwrap().value.contains("theme=dark"));
    assert!(sandbox.execute("document.cookie").unwrap().value.contains("lang=en"));
    // 覆盖同名 cookie（name 唯一）。
    sandbox.execute("document.cookie = 'theme=light'").unwrap();
    assert_eq!(
        sandbox
            .execute("document.cookie.split('; ').sort().join('; ')")
            .unwrap()
            .value,
        "lang=en; theme=light"
    );
    // value 含 '='（split on 首 '='，value 保留后续 '='）。
    sandbox.execute("document.cookie = 'token=a=b=c'").unwrap();
    assert!(
        sandbox
            .execute("document.cookie")
            .unwrap()
            .value
            .contains("token=a=b=c")
    );
    // 无 name=value（无 '=' 串）→ 忽略，不影响存储。
    sandbox.execute("document.cookie = 'justtext'").unwrap();
    assert!(!sandbox.execute("document.cookie").unwrap().value.contains("justtext"));
}

#[test]
fn test_text_encoder_decoder_utf8_r2771() {
    // R2771：TextEncoder（str→UTF-8 Uint8Array）+ TextDecoder（bytes→str）。纯 JS UTF-8
    //（BMP + astral 代理对）。fetch body / 字符串↔字节互转高频。encode 'ZeroWeb' = ASCII 7 字节，
    // 中文 = 3 字节/字，round-trip 保真。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // TextEncoder：encoding=utf-8；encode('ZeroWeb') = 7 ASCII 字节 [90,...,98]。
    assert_eq!(
        sandbox
            .execute(
                "var a = new TextEncoder().encode('ZeroWeb');\
                 new TextEncoder().encoding + '|' + a.length + '|' + a[0] + '|' + a[6]"
            )
            .unwrap()
            .value,
        "utf-8|7|90|98"
    );
    // 中文多字节：'中' = U+4E2D → 3 字节 UTF-8。
    assert_eq!(
        sandbox.execute("new TextEncoder().encode('中').length").unwrap().value,
        "3"
    );
    // TextDecoder：字面字节序列 → 字符串。
    assert_eq!(
        sandbox
            .execute("new TextDecoder().decode(new Uint8Array([0x5a,0x65,0x72,0x6f]))")
            .unwrap()
            .value,
        "Zero"
    );
    // Round-trip（ASCII + 中文混排）保真。
    assert_eq!(
        sandbox
            .execute(
                "var e = new TextEncoder(), d = new TextDecoder();\
                 d.decode(e.encode('ZeroWeb 中文'))"
            )
            .unwrap()
            .value,
        "ZeroWeb 中文"
    );
}

#[test]
fn test_url_search_params_r2772() {
    // R2772：URLSearchParams（query 解析/序列化，location.search/fetch query 高频）。纯 JS。
    // 构造（string/?前缀/对象）+ get/getAll/has/set/append/delete + toString（space→+）+ 可迭代。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // 构造 + get（`?` 前缀可省）。
    assert_eq!(
        sandbox
            .execute("new URLSearchParams('?a=1&b=2').get('a')")
            .unwrap()
            .value,
        "1"
    );
    assert_eq!(
        sandbox
            .execute("new URLSearchParams('a=1&b=2').get('b')")
            .unwrap()
            .value,
        "2"
    );
    // 缺键 get → null；getAll 多值。
    assert_eq!(
        sandbox
            .execute("String(new URLSearchParams('a=1').get('z'))")
            .unwrap()
            .value,
        "null"
    );
    assert_eq!(
        sandbox
            .execute("new URLSearchParams('a=1&a=2').getAll('a').join(',')")
            .unwrap()
            .value,
        "1,2"
    );
    // has / append / set / delete。
    assert_eq!(
        sandbox.execute("new URLSearchParams('a=1').has('a')").unwrap().value,
        "true"
    );
    sandbox
        .execute("globalThis.__p = new URLSearchParams('a=1&b=2'); __p.append('c', '3'); __p.set('a', '9'); __p.delete('b');")
        .unwrap();
    assert_eq!(sandbox.execute("__p.get('a')").unwrap().value, "9");
    assert_eq!(sandbox.execute("String(__p.has('b'))").unwrap().value, "false");
    assert_eq!(sandbox.execute("__p.get('c')").unwrap().value, "3");
    // toString（space→+，round-trip）。
    assert_eq!(
        sandbox
            .execute("new URLSearchParams('q=hello+world&n=42').toString()")
            .unwrap()
            .value,
        "q=hello+world&n=42"
    );
    // 对象构造。
    assert_eq!(
        sandbox
            .execute("new URLSearchParams({ x: '1', y: '2' }).toString()")
            .unwrap()
            .value,
        "x=1&y=2"
    );
    // 可迭代：for...of 收集键。
    assert_eq!(
        sandbox
            .execute("var ks = []; for (var kv of new URLSearchParams('a=1&b=2')) ks.push(kv[0]); ks.join(',')")
            .unwrap()
            .value,
        "a,b"
    );
}

#[test]
fn test_structured_clone_r2773() {
    // R2773：structuredClone（深拷贝，postMessage/React state 高频）。递归 + 循环引用（WeakMap）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // primitive 原样返回。
    assert_eq!(sandbox.execute("structuredClone(42)").unwrap().value, "42");
    assert_eq!(sandbox.execute("structuredClone('hi')").unwrap().value, "hi");
    assert_eq!(sandbox.execute("String(structuredClone(null))").unwrap().value, "null");
    // 嵌套对象深拷贝独立（改 clone 不影响原）。
    assert_eq!(
        sandbox
            .execute("var a = { x: 1, n: { y: 2 } }; var b = structuredClone(a); b.n.y = 99; a.n.y")
            .unwrap()
            .value,
        "2"
    );
    // 数组深拷贝独立。
    assert_eq!(
        sandbox
            .execute("var a = [1, [2, 3]]; var b = structuredClone(a); b[1][0] = 99; a[1][0]")
            .unwrap()
            .value,
        "2"
    );
    // Date 保类型 + 值。
    assert_eq!(
        sandbox
            .execute("structuredClone(new Date(2020, 0, 1)).getTime() === new Date(2020, 0, 1).getTime()")
            .unwrap()
            .value,
        "true"
    );
    // RegExp 保 flags。
    assert_eq!(sandbox.execute("structuredClone(/abc/gi).flags").unwrap().value, "gi");
    // 循环引用不爆栈（self-ref 解到 clone 自身）。
    assert_eq!(
        sandbox
            .execute("var a = {}; a.self = a; var b = structuredClone(a); b.self === b")
            .unwrap()
            .value,
        "true"
    );
    // function 抛 DataCloneError（spec）。
    assert_eq!(
        sandbox
            .execute("try { structuredClone(function(){}); 'no-throw' } catch (e) { 'threw' }")
            .unwrap()
            .value,
        "threw"
    );
}

#[test]
fn test_queue_microtask_r2774() {
    // R2774：queueMicrotask（microtask 调度，高频）。V8 embed 未暴露全局，用 Promise.resolve().then
    // polyfill；execute 末 microtask checkpoint 派发。callback 在该 execute 末运行（下 execute 可读）。
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();

    // typeof function（全局已定义）。
    assert_eq!(sandbox.execute("typeof queueMicrotask").unwrap().value, "function");
    // callback 在 execute 末 microtask checkpoint 派发——下 execute 可读 __ran。
    sandbox
        .execute("globalThis.__ran = false; queueMicrotask(function(){ globalThis.__ran = true; });")
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__ran)").unwrap().value, "true");
    // 非 callable 抛 TypeError（spec）。
    assert_eq!(
        sandbox
            .execute("try { queueMicrotask('notfn'); 'no-throw' } catch (e) { 'threw' }")
            .unwrap()
            .value,
        "threw"
    );
}

#[test]
fn test_clone_node_e2e() {
    // cloneNode(deep) 复用既有回调组合：create(tag) + 逐属性 set_attr_handle + (deep) set_inner_html_handle。
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
        "<html><body><div id='src' class='row' data-x='1'><span>child</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // deep clone → 记 CreateElement + 复制源全部属性 + SetInnerHtmlOnHandle。
    sandbox
        .execute("document.querySelector('#src').cloneNode(true);")
        .unwrap();
    let ms = mutations.lock().unwrap();
    // CreateElement(tag=div)。
    let created_tag = ms.iter().find_map(|m| match m {
        DomMutation::CreateElement { tag, .. } => Some(tag.clone()),
        _ => None,
    });
    assert_eq!(created_tag.as_deref(), Some("div"), "cloneNode 应 CreateElement(div)");
    // SetAttrOnHandle 复制源全部 3 属性（id/class/data-x，含值）。
    let has_attr = |name: &str, value: &str| {
        ms.iter().any(|m| match m {
            DomMutation::SetAttrOnHandle { name: n, value: v, .. } => n == name && v == value,
            _ => false,
        })
    };
    assert!(has_attr("id", "src"), "应复制 id=src");
    assert!(has_attr("class", "row"), "应复制 class=row");
    assert!(has_attr("data-x", "1"), "应复制 data-x=1");
    // deep：SetInnerHtmlOnHandle 含源后代 <span>child</span>。
    let deep = ms.iter().any(|m| match m {
        DomMutation::SetInnerHtmlOnHandle { html, .. } => html.contains("<span>child</span>"),
        _ => false,
    });
    assert!(deep, "deep clone 应 SetInnerHtmlOnHandle 含源后代");
}

#[test]
fn test_collect_element_ids_dedup_preserve_order() {
    let html = "<html><body>\
                    <div id=\"container\"></div>\
                    <span id=\"target\"></span>\
                    <p id=\"container\"></p>\
                    <b></div>\
                    </body></html>";
    let ids = collect_element_ids(html);
    // 去重（首个 container 保留），保序，跳过无 id 元素。
    assert_eq!(ids, "container|target");
}

#[test]
fn test_collect_element_ids_empty() {
    let html = "<html><body><div></div><p class=\"x\"></p></body></html>";
    assert_eq!(collect_element_ids(html), "");
}

#[test]
fn test_apply_inner_html() {
    let html = "<html><body><div id=\"d\">old</div></body></html>";
    let mutations = vec![DomMutation::SetInnerHtml {
        selector: "#d".into(),
        html: "<b>new</b>".into(),
    }];
    let out = apply_mutations_to_html(html, &mutations).unwrap();
    assert!(out.contains("<b>new</b>"));
}

#[test]
fn test_shim_not_empty() {
    assert!(generate_js_dom_shim().contains("__zw_set_attr"));
    assert!(generate_js_dom_shim().contains("addEventListener"));
}

#[test]
fn test_shim_async_resolve_callback_e2e() {
    // P1b S1（方案 A）端到端：注入**生产** DOM shim（含 __zwResolveCallback + pending 表），
    // 验证 V8Sandbox::resolve_async_callback 经 shim 的 JS 契约真实 resolve Promise。
    // 宿主回调同步返「回调 ID」，JS 建 pending Promise；Rust resolve 触发 .then。
    use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};
    let config = SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    // 注入生产 shim（tab_js_worker.rs / js_worker.rs 同款）。
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox.register_callback("__zw_start_async", Box::new(|args| format!("aid:{}", args[0])));
    sandbox
        .execute(
            "var id = __zw_start_async('99');
                 new Promise(function(resolve){ globalThis.__zw_pending[id] = resolve; })
                     .then(function(v){ globalThis.__result = v; });",
        )
        .unwrap();
    // resolve 前：Promise pending。
    let before = sandbox.execute("typeof globalThis.__result").unwrap();
    assert_eq!(before.value, "undefined");
    // Rust 异步完成 → resolve（shim 的 __zwResolveCallback 触发 + microtask drain）。
    sandbox.resolve_async_callback("aid:99", "resolved!");
    let after = sandbox.execute("globalThis.__result").unwrap();
    assert_eq!(after.value, "resolved!");
}

#[test]
fn test_shim_includes_runtime_stubs() {
    let shim = generate_js_dom_shim();
    assert!(shim.contains("globalThis.setTimeout"));
    assert!(shim.contains("globalThis.navigator"));
    assert!(shim.contains("attachEvent"));
    assert!(shim.contains("__zw_get_page_url"));
    assert!(shim.contains("globalThis.screen"));
    assert!(shim.contains("parentNode"));
    // P1b S1（方案 A）异步回调 resolve 通道 JS 侧契约。
    assert!(shim.contains("globalThis.__zwResolveCallback"));
    assert!(shim.contains("globalThis.__zw_pending"));
    // P1a select：<select>.value/selectedIndex getter + setter 经 host 回调。
    assert!(shim.contains("__zw_select_value"));
    assert!(shim.contains("__zw_select_index"));
    assert!(shim.contains("__zw_select_option"));
}

#[test]
fn test_shim_includes_modern_reftest_stubs() {
    // 现代动态 reftest 的 `requestAnimationFrame(() => …; takeScreenshot())` 模式
    // 要求这两个全局存在，否则 setup mutation 永不执行（R917 未捕获的 yield gap）。
    let shim = generate_js_dom_shim();
    assert!(shim.contains("globalThis.requestAnimationFrame"));
    assert!(shim.contains("globalThis.cancelAnimationFrame"));
    assert!(shim.contains("globalThis.takeScreenshot"));
    // `Element.append(...nodesOrStrings)` 现代 API（区别于 appendChild）。
    assert!(shim.contains("if (prop === 'append')"));
    // `getBoundingClientRect()` 方法必须返回零 DOMRect，否则调用抛 TypeError
    // 中断脚本，使其后的 mutation 丢失（120 reftest 文件用作 reflow 触发器）。
    assert!(shim.contains("if (prop === 'getBoundingClientRect')"));
    // HTML 规范 named access on window（`id="x"` → 全局 `x`，257 reftest 文件）。
    assert!(shim.contains("_installNamedAccess"));
    assert!(shim.contains("__zw_collect_ids"));
    // `createElementNS`（XHTML 命名空间 alias createElement；SVG OOS 不渲染但不中断）。
    assert!(shim.contains("createElementNS:"));
    // `getComputedStyle`：动态 reftest 常作「强制 reflow」触发器调用，缺失则抛
    // ReferenceError 中断脚本丢失后续 mutation。返空 CSSStyleDeclaration 桩不抛。
    assert!(shim.contains("globalThis.getComputedStyle"));
    assert!(shim.contains("getPropertyValue"));
}

#[test]
fn test_merge_style_property() {
    let merged = merge_style_property("color: blue", "width", "10px");
    assert!(merged.contains("color: blue"));
    assert!(merged.contains("width: 10px"));
    let replaced = merge_style_property(&merged, "color", "red");
    assert!(!replaced.contains("blue"));
    assert!(replaced.contains("color: red"));
    // R3211：空值移除声明（spec setProperty/IDL setter 空值语义；`el.style.color=''` 应移除而非留
    // `color: `）。设既有声明为空 → 该声明消失；设不存在属性为空 → 无 dangling 残留。
    let cleared = merge_style_property("color: red; font-size: 10px", "color", "");
    assert!(
        !cleared.contains("color"),
        "empty value should remove the declaration, got: {cleared}"
    );
    assert!(cleared.contains("font-size: 10px"));
    let no_dangle = merge_style_property("color: red", "margin", "  ");
    assert!(
        !no_dangle.contains("margin"),
        "whitespace-only value should not leave a dangling declaration, got: {no_dangle}"
    );
}

#[test]
fn test_enclosing_form_selector() {
    // P1a form submit：input 在 form 内 → 返 form 的 stable selector。
    let html = "<html><body><form id='f'><input id='i'></form></body></html>";
    assert_eq!(enclosing_form_selector(html, "#i").as_deref(), Some("#f"));
    // input 无 enclosing form → None。
    let no_form = "<html><body><div><input id='i'></div></body></html>";
    assert_eq!(enclosing_form_selector(no_form, "#i"), None);
    // 嵌套：input 在 form 内的 div 内 → 仍解析到 form。
    let nested = "<html><body><form id='outer'><div><input id='deep'></div></form></body></html>";
    assert_eq!(enclosing_form_selector(nested, "#deep").as_deref(), Some("#outer"));
    // 未命中 selector → None。
    assert_eq!(enclosing_form_selector(html, "#missing"), None);
}

#[test]
fn test_is_submit_button() {
    // P1a form submit：submit-button 判定。
    assert!(is_submit_button(
        "<html><body><form><input id='b' type='submit'></form></body></html>",
        "#b",
    ));
    assert!(is_submit_button(
        "<html><body><form><input id='i' type='image'></form></body></html>",
        "#i",
    ));
    // button 默认 type=submit → 提交。
    assert!(is_submit_button(
        "<html><body><form><button id='btn'>Go</button></form></body></html>",
        "#btn",
    ));
    assert!(is_submit_button(
        "<html><body><form><button id='s' type='submit'>Go</button></form></body></html>",
        "#s",
    ));
    // 非提交：
    assert!(!is_submit_button(
        "<html><body><form><input id='t' type='text'></form></body></html>",
        "#t",
    ));
    assert!(!is_submit_button(
        "<html><body><form><button id='nb' type='button'>Go</button></form></body></html>",
        "#nb",
    ));
    assert!(!is_submit_button(
        "<html><body><form><button id='reset' type='reset'>Clear</button></form></body></html>",
        "#reset",
    ));
    assert!(!is_submit_button(
        "<html><body><form><div id='d'>x</div></form></body></html>",
        "#d",
    ));
}

#[test]
fn test_is_reset_button_r3050() {
    // R3050：reset-button 判定（供 renderer click 路由）。仅显式 type=reset。
    assert!(is_reset_button(
        "<html><body><form><input id='r' type='reset'></form></body></html>",
        "#r",
    ));
    assert!(is_reset_button(
        "<html><body><form><button id='br' type='reset'>Clear</button></form></body></html>",
        "#br",
    ));
    // 非 reset：
    assert!(!is_reset_button(
        "<html><body><form><input id='s' type='submit'></form></body></html>",
        "#s",
    ));
    // button 默认 type=submit（非 reset）。
    assert!(!is_reset_button(
        "<html><body><form><button id='btn'>Go</button></form></body></html>",
        "#btn",
    ));
    assert!(!is_reset_button(
        "<html><body><form><button id='nb' type='button'>Go</button></form></body></html>",
        "#nb",
    ));
    assert!(!is_reset_button(
        "<html><body><form><input id='t' type='text'></form></body></html>",
        "#t",
    ));
}

#[test]
fn test_anchor_click_target_r3052() {
    // R3052：anchor_click_target 解析 <a href> click 导航目标。
    let base = "https://example.com/dir/page";

    // ① 绝对 URL（http/https）→ Some（原样）。
    assert_eq!(
        anchor_click_target(
            "<html><body><a id='a' href='https://other.com/x'>l</a></body></html>",
            "#a",
            base
        ),
        Some("https://other.com/x".to_string()),
        "绝对 https href → Some(原样)"
    );
    // ② 相对 href（/page, page.html, ../up）→ resolve_document_url 按 base 解析为绝对。
    assert_eq!(
        anchor_click_target("<html><body><a id='r' href='/p2'>l</a></body></html>", "#r", base),
        Some("https://example.com/p2".to_string()),
        "相对 /p2 → 绝对 https://example.com/p2"
    );
    assert_eq!(
        anchor_click_target(
            "<html><body><a id='rel' href='next.html'>l</a></body></html>",
            "#rel",
            base
        ),
        Some("https://example.com/dir/next.html".to_string()),
        "相对 next.html → 按 base 目录解析"
    );
    // ③ 协议相对 //host → 用 base scheme。
    assert_eq!(
        anchor_click_target(
            "<html><body><a id='pr' href='//cdn.example.com/asset'>l</a></body></html>",
            "#pr",
            base
        )
        .as_deref(),
        Some("https://cdn.example.com/asset"),
        "协议相对 //host → 继承 base scheme"
    );

    // ④ 非导航 scheme / fragment → None。
    assert_eq!(
        anchor_click_target("<html><body><a id='h' href='#sec'>l</a></body></html>", "#h", base),
        None,
        "#hash → None（同文档锚，headless no-op）"
    );
    assert_eq!(
        anchor_click_target(
            "<html><body><a id='j' href='javascript:void(0)'>l</a></body></html>",
            "#j",
            base
        ),
        None,
        "javascript: → None（不 eval）"
    );
    assert_eq!(
        anchor_click_target(
            "<html><body><a id='m' href='mailto:a@b.com'>l</a></body></html>",
            "#m",
            base
        ),
        None,
        "mailto: → None（外部 handler）"
    );
    assert_eq!(
        anchor_click_target(
            "<html><body><a id='d' href='data:text/html,hi'>l</a></body></html>",
            "#d",
            base
        ),
        None,
        "data: → None"
    );

    // ⑤ target=_blank/_top/_parent → None（新窗口/顶层，headless no-op）；target=_self/默认 → Some。
    assert_eq!(
        anchor_click_target(
            "<html><body><a id='b' href='https://x.com/' target='_blank'>l</a></body></html>",
            "#b",
            base
        ),
        None,
        "target=_blank → None（新窗口 no-op）"
    );
    assert_eq!(
        anchor_click_target(
            "<html><body><a id='s' href='https://x.com/' target='_self'>l</a></body></html>",
            "#s",
            base
        ),
        Some("https://x.com/".to_string()),
        "target=_self → Some（同标签页导航）"
    );

    // ⑥ 非 <a> / 无 href → None。
    assert_eq!(
        anchor_click_target("<html><body><div id='d' href='https://x.com/'>l</div></body></html>", "#d", base),
        None,
        "非 <a> 元素 → None（即使有 href）"
    );
    assert_eq!(
        anchor_click_target("<html><body><a id='n'>l</a></body></html>", "#n", base),
        None,
        "<a> 无 href → None"
    );
    assert_eq!(
        anchor_click_target("<html><body><a id='e' href=''>l</a></body></html>", "#e", base),
        None,
        "<a> 空 href → None"
    );
}

#[test]
fn test_script_call_form_reset_r3050() {
    // R3050：script_call_form_reset 生成调 form.reset() 的 shim 脚本。
    // 选择器安全嵌入（引号转义）；form 不存在/reset 非函数 → no-op guard。
    let s = script_call_form_reset("#myform");
    assert!(s.contains("querySelector('#myform')"), "脚本含 querySelector('#myform')\n{s}");
    assert!(s.contains("f.reset()"), "脚本调 f.reset()\n{s}");
    assert!(s.contains("typeof f.reset==='function'"), "含 reset 函数 guard（防 throw）\n{s}");
    // 选择器转义：嵌入引号经 escape_js_string 转义（不破坏 JS 串）。
    let s2 = script_call_form_reset("#a'b");
    assert!(s2.contains("\\'#a\\'b'") || !s2.contains("'#a'b'"), "选择器引号转义（不裸含 '#a'b'）\n{s2}");
}

#[test]
fn test_script_dispatch_native_event_r3124() {
    // R3124：script_dispatch_native_event 生成经原生绑定派发的 IIFE。
    // ① typeof 守卫（__zw_native_query_selector 未定义时 no-op，防 ReferenceError）。
    // ② event 对象丰富化：{type, target:t, currentTarget:t, bubbles:true}（闭合 R3121 限制① bare {type}）。
    // ③ 选择器 / 事件类型经 escape_js_string 安全嵌入（引号转义）。
    let s = script_dispatch_native_event("#btn", "click");
    assert!(
        s.contains("typeof __zw_native_query_selector!=='function'"),
        "含 typeof 守卫（未安装绑定时 no-op）\n{s}"
    );
    assert!(
        s.contains("dispatchEvent({type:'click',target:t,currentTarget:t,bubbles:true})"),
        "dispatchEvent 调丰富 event 对象（type/target/currentTarget/bubbles）\n{s}"
    );
    assert!(
        s.contains("__zw_native_query_selector('#btn')"),
        "选择器安全嵌入\n{s}"
    );
    // 选择器转义：嵌入引号经 escape_js_string 转义（不破坏 JS 串）。
    let s2 = script_dispatch_native_event("#a'b", "input");
    assert!(
        s2.contains("\\'#a\\'b'") || !s2.contains("'#a'b'"),
        "选择器引号转义（不裸含 '#a'b'）\n{s2}"
    );
    assert!(
        s2.contains("type:'input'"),
        "事件类型嵌入\n{s2}"
    );
}

#[test]
fn test_anchor_hash_target_r3053() {
    // R3053：anchor_hash_target 解析 <a href="#..."> click 的 hash 目标（供 renderer click 路由
    // 判定是否设 location.hash，闭合 R3052 限制③）。返回 Some(hash)（含前导 '#'）当 <a> 且 href 以 '#' 开头。
    // ① 普通锚 #sec / # → Some（原样含 '#')。
    assert_eq!(
        anchor_hash_target("<html><body><a id='a' href='#sec'>l</a></body></html>", "#a"),
        Some("#sec".to_string()),
        "href='#sec' → Some('#sec')"
    );
    assert_eq!(
        anchor_hash_target("<html><body><a id='e' href='#'>l</a></body></html>", "#e"),
        Some("#".to_string()),
        "href='#'（空锚）→ Some('#')"
    );
    // ② href 带空白：trim 后仍判 '#' 开头，返回 trim 后值（mirror anchor_click_target trim；
    // shim _setLocationHash 会归一化，故去空白无副作用）。
    assert_eq!(
        anchor_hash_target("<html><body><a id='w' href='  #top  '>l</a></body></html>", "#w"),
        Some("#top".to_string()),
        "href 含空白 → trim 后 Some('#top')"
    );

    // ③ 非 hash href → None（绝对 / 相对 / 非导航 scheme）。
    assert_eq!(
        anchor_hash_target("<html><body><a id='u' href='https://x.com/'>l</a></body></html>", "#u"),
        None,
        "绝对 URL href → None"
    );
    assert_eq!(
        anchor_hash_target("<html><body><a id='r' href='/p2'>l</a></body></html>", "#r"),
        None,
        "相对 /p2 href → None"
    );
    assert_eq!(
        anchor_hash_target("<html><body><a id='j' href='javascript:void(0)'>l</a></body></html>", "#j"),
        None,
        "javascript: href → None"
    );

    // ④ 非 <a> / 无 href → None。
    assert_eq!(
        anchor_hash_target("<html><body><div id='d' href='#sec'>l</div></body></html>", "#d"),
        None,
        "非 <a> 元素 → None（即使 href='#sec'）"
    );
    assert_eq!(
        anchor_hash_target("<html><body><a id='n'>l</a></body></html>", "#n"),
        None,
        "<a> 无 href → None"
    );
    assert_eq!(
        anchor_hash_target("<html><body><a id='x' href=''>l</a></body></html>", "#x"),
        None,
        "<a> 空 href → None（不以 '#' 开头）"
    );
}

#[test]
fn test_script_call_set_location_hash_r3053() {
    // R3053：script_call_set_location_hash 生成设 location.hash 的脚本。
    // 调 shim location.hash = hash（R3006：更新 hash + history entry + 派 hashchange）。
    let s = script_call_set_location_hash("#sec");
    assert!(s.contains("location.hash='#sec'"), "脚本设 location.hash='#sec'\n{s}");
    // hash 经 escape_js_string 安全嵌入（引号 / 反斜杠转义，不破坏 JS 串）。
    let s2 = script_call_set_location_hash("#a'b\\c");
    assert!(!s2.contains("'#a'b\\c'"), "hash 引号/反斜杠转义（不裸含原值）\n{s2}");
    assert!(s2.contains("location.hash="), "转义后仍是 location.hash 赋值\n{s2}");
}

#[test]
fn test_remove_attr_and_has_attribute() {
    // P1a checkbox：RemoveAttr 真正移除属性；has_attribute 判存在性。
    let html = "<html><body><input id='c' type='checkbox' checked></body></html>";
    assert!(has_attribute(html, "#c", "checked"));
    let out = apply_mutations_to_html(
        html,
        &[DomMutation::RemoveAttr {
            selector: "#c".into(),
            name: "checked".into(),
        }],
    )
    .unwrap();
    assert!(!out.contains("checked"));
    assert!(!has_attribute(&out, "#c", "checked"));
    // 无该属性 → has_attribute false。
    assert!(!has_attribute(
        "<html><body><input id='n' type='checkbox'></body></html>",
        "#n",
        "checked",
    ));
}

#[test]
fn test_is_checkbox() {
    assert!(is_checkbox(
        "<html><body><input id='c' type='checkbox'></body></html>",
        "#c",
    ));
    assert!(!is_checkbox(
        "<html><body><input id='t' type='text'></body></html>",
        "#t",
    ));
    assert!(!is_checkbox("<html><body><div id='d'></div></body></html>", "#d",));
}

#[test]
fn test_toggle_radio_html() {
    // P1a radio：toggle target → set checked + 同 name 组兄弟 unset（直接 doc 操作）。
    let html = "<html><body><form>\
            <input id='a' type='radio' name='g' checked>\
            <input id='b' type='radio' name='g'>\
            <input id='c' type='checkbox' checked>\
            </form></body></html>";
    // toggle #b → #b checked、#a unchecked（同 name 组）；#c checkbox 不受影响。
    let out = toggle_radio_html(html, "#b").unwrap();
    assert!(has_attribute(&out, "#b", "checked"));
    assert!(!has_attribute(&out, "#a", "checked"));
    assert!(has_attribute(&out, "#c", "checked"));
    // 非 radio → None。
    assert_eq!(toggle_radio_html(html, "#c"), None);
}

#[test]
fn test_select_value_read() {
    let html = "<html><body><select id='s'>\
            <option value='a'>A</option>\
            <option value='b' selected>B</option>\
            <option value='c'>C</option>\
            </select></body></html>";
    assert!(is_select(html, "#s"));
    // selected option b → "b"。
    assert_eq!(select_value_from_html(html, "#s"), "b");
    assert_eq!(select_index_from_html(html, "#s"), 1);
    // 无 selected 属性 → 默认首个 option。
    let html2 =
        "<html><body><select id='s'><option value='x'>X</option><option value='y'>Y</option></select></body></html>";
    assert_eq!(select_value_from_html(html2, "#s"), "x");
    assert_eq!(select_index_from_html(html2, "#s"), 0);
    // option 无 value 属性 → text content。
    let html3 = "<html><body><select id='s'><option>Plain</option></select></body></html>";
    assert_eq!(select_value_from_html(html3, "#s"), "Plain");
    // 无 option → 空串 / -1。
    let html4 = "<html><body><select id='s'></select></body></html>";
    assert_eq!(select_value_from_html(html4, "#s"), "");
    assert_eq!(select_index_from_html(html4, "#s"), -1);
}

#[test]
fn test_set_selected_option_html() {
    let html = "<html><body><select id='s'>\
            <option value='a' selected>A</option>\
            <option value='b'>B</option>\
            <option value='c'>C</option>\
            </select></body></html>";
    // 设 value='c' → c selected、a/b deselect。
    let out = set_selected_option_html(html, "#s", "c").unwrap();
    assert!(has_attribute(&out, "#s > option:nth-of-type(3)", "selected") || out.contains("value=\"c\" selected"));
    assert!(!has_attribute(&out, "#s > option:nth-of-type(1)", "selected"));
    // 经 value 读回 = "c"。
    assert_eq!(select_value_from_html(&out, "#s"), "c");
    // 匹配 option 无 value 属性（按 text content）。
    let html2 = "<html><body><select id='s'><option>One</option><option>Two</option></select></body></html>";
    let out2 = set_selected_option_html(html2, "#s", "Two").unwrap();
    assert_eq!(select_value_from_html(&out2, "#s"), "Two");
    // 无匹配 value → None（不改）。
    assert_eq!(set_selected_option_html(html, "#s", "zzz"), None);
    // 非 select → None。
    assert_eq!(set_selected_option_html(html, "body", "a"), None);
}

#[test]
fn test_apply_select_option_mutation() {
    let html = "<html><body><select id='s'>\
            <option value='a' selected>A</option>\
            <option value='b'>B</option>\
            </select></body></html>";
    let out = apply_mutations_to_html(
        html,
        &[DomMutation::SelectOption {
            selector: "#s".into(),
            value: "b".into(),
        }],
    )
    .unwrap();
    assert_eq!(select_value_from_html(&out, "#s"), "b");
    // SelectOption 也参与 handle→selector map（apply_dom_mutations 末尾无新 handle，map 空）。
    let (_, handles) = apply_mutations_to_html_with_handles(
        html,
        &[DomMutation::SelectOption {
            selector: "#s".into(),
            value: "b".into(),
        }],
    )
    .unwrap();
    assert!(handles.is_empty(), "SelectOption 不创建 handle");
}

/// R3183：实证 polyfill 生产路径的 R3182 context-element 修复——`apply_dom_mutations` 经
/// `replace_inner_html`（R3182 用 html5ever `parse_fragment` + table context）应用 `SetInnerHtml`。
/// `table.innerHTML='<tr><td>x</td></tr>'` 序列化含 `tbody>tr>td`（context-element 解析），
/// 旧 body-wrap 在 body context 下 `<tr>` foster-parent → table 仅含文本 "x"。
#[test]
fn test_apply_set_inner_html_table_context_r3183() {
    let html = "<html><body><table id='t'></table></body></html>";
    let out = apply_mutations_to_html(
        html,
        &[DomMutation::SetInnerHtml {
            selector: "#t".into(),
            html: "<tr><td>x</td></tr>".into(),
        }],
    )
    .unwrap();
    // context-element（table）解析 → 隐式 tbody 包裹 tr>td（R3182 修复）。
    assert!(
        out.contains("<tbody><tr><td>x</td></tr></tbody>"),
        "table.innerHTML 经 context-element 解析应含 tbody>tr>td，got: {out}"
    );
    // 旧 body-wrap bug：table 仅含孤立文本 "x"（<table...>x</table>）。修复后无此结构。
    assert!(
        !out.contains(">x</table>"),
        "table 不应含孤立文本 x（旧 foster-parent bug），got: {out}"
    );
}

/// R3206：innerHTML setter 经 `copy_subtree_from` 重建 SVG 外部命名空间属性时保留 prefix + ns。
/// `div.innerHTML = '<svg><use xlink:href="#a"/></svg>'` round-trip（apply → 序列化）须保留 `xlink:href`
/// 前缀。旧 `copy_subtree_from` 恒走 `set_attribute(local)`，把 `xlink:href` 重建为裸 `href`（无 ns），
/// 配合 serializer（R3206 读侧）闭合 write + read 全 round-trip。
#[test]
fn test_apply_set_inner_html_svg_xlink_prefix_r3206() {
    let html = r#"<html><body><div id="t"></div></body></html>"#;
    let out = apply_mutations_to_html(
        html,
        &[DomMutation::SetInnerHtml {
            selector: "#t".into(),
            html: r##"<svg><use xlink:href="#a"/></svg>"##.into(),
        }],
    )
    .unwrap();
    // apply 后序列化保留 xlink:href 前缀（write 侧 copy_subtree_from 保留 QualName）。
    assert!(
        out.contains(r##"xlink:href="#a""##),
        "innerHTML setter 应保留 xlink:href 前缀（write side），got: {out}"
    );
    // 裸 href 是丢失前缀的症状（旧 bug），不应出现。
    assert!(
        !out.contains(r##" href="#a""##),
        "不应序列化为丢前缀的裸 href（旧 copy_subtree_from bug），got: {out}"
    );
}

/// R3207：spec `dom-element-insertadjacenthtml` position 为 ASCII 大小写不敏感。beforebegin/afterend
/// 的 context element = 目标父。旧实现 `match position`（大小写敏感）对大写 position（如 BEFOREBEGIN）
/// 错走 `_` 分支用**目标自身**作 context（应为父），致 table 等 context-sensitive 片段解析错——
/// 实测 `<tr><td>y</td></tr>` 在 caption（目标）context 下 foster-parent 丢行结构（仅剩文本 "y"），
/// 而正确 table（父）context 下解析为 `<tbody><tr><td>y</td></tr></tbody>`。修复：先规范化小写。
#[test]
fn test_insert_adjacent_html_position_case_insensitive_r3207() {
    let base = "<html><body><table id='tb'><caption id='c'>x</caption></table></body></html>";
    let frag = "<tr><td>y</td></tr>";
    let lower = apply_mutations_to_html(
        base,
        &[DomMutation::InsertAdjacentHtml {
            selector: "#c".into(),
            position: "beforebegin".into(),
            html: frag.into(),
        }],
    )
    .unwrap();
    let upper = apply_mutations_to_html(
        base,
        &[DomMutation::InsertAdjacentHtml {
            selector: "#c".into(),
            position: "BEFOREBEGIN".into(),
            html: frag.into(),
        }],
    )
    .unwrap();
    // 大小写 position 应产 identical output（spec ASCII-case-insensitive）。
    assert_eq!(
        lower, upper,
        "ASCII-case-insensitive position should produce identical output"
    );
    // 正确（父 table）context：片段解析为 tbody>tr>td（行结构保留），非 foster-parent 丢行。
    assert!(
        lower.contains("<tbody><tr><td>y</td></tr></tbody>"),
        "table context 应解析为 tbody>tr>td，got: {lower}"
    );
}

/// R3208：spec outerHTML setter——目标父为 Document（即 `<html>` 根元素）应抛 NoModificationAllowedError。
/// 旧实现只查 parent null（NotFoundError），不查 parent is Document，故 `documentElement.outerHTML=...`
/// 不报错，移除 html 后 Document 直接挂片段节点（畸形树，实测序列化为孤立 "y"）。
/// spec：https://dom.spec.whatwg.org/#dom-element-outerhtml（outerHTML setter step 3）。
#[test]
fn test_set_outer_html_root_element_errors_r3208() {
    let html = "<html><head></head><body><div id='t'>x</div></body></html>";
    let res = apply_mutations_to_html(
        html,
        &[DomMutation::SetOuterHtml {
            selector: "html".into(),
            html: "<html><body>y</body></html>".into(),
        }],
    );
    assert!(
        res.is_err(),
        "html 根元素 outerHTML 赋值应失败（parent is Document → NoModificationAllowedError）"
    );
    // 常规元素 outerHTML 不受影响（parent 为 element）——回归保护。
    let ok = apply_mutations_to_html(
        "<html><body><div id='t'><span>x</span></div></body></html>",
        &[DomMutation::SetOuterHtml {
            selector: "#t".into(),
            html: "<p>y</p>".into(),
        }],
    )
    .unwrap();
    assert!(ok.contains("<p>y</p>"), "常规元素 outerHTML 仍正常替换: {ok}");
}

#[test]
fn test_is_text_input() {
    // P1a change-on-blur：文本输入判定（textarea + input 文本类；排除 action 类型）。
    assert!(is_text_input(
        "<html><body><input id='t' type='text'></body></html>",
        "#t",
    ));
    assert!(is_text_input(
        "<html><body><input id='e' type='email'></body></html>",
        "#e",
    ));
    assert!(is_text_input(
        "<html><body><textarea id='ta'></textarea></body></html>",
        "#ta",
    ));
    // input 无 type → 默认 text。
    assert!(is_text_input("<html><body><input id='n'></body></html>", "#n"));
    // action 类型排除（change 在 click 派发）。
    assert!(!is_text_input(
        "<html><body><input id='cb' type='checkbox'></body></html>",
        "#cb",
    ));
    assert!(!is_text_input(
        "<html><body><input id='s' type='submit'></body></html>",
        "#s",
    ));
    assert!(!is_text_input("<html><body><div id='d'></div></body></html>", "#d",));
}

#[test]
fn test_next_focus_selector() {
    // P1a Tab 焦点导航：tabindex>0 升序在前（d=1, c=2），0/默认文档序在后（a, b）→ [d,c,a,b]。
    let html = "<html><body>\
            <input id='a'>\
            <button id='b'>x</button>\
            <input id='c' tabindex='2'>\
            <input id='d' tabindex='1'>\
            </body></html>";
    // 无 current → first = d（tabindex=1）。
    assert_eq!(next_focus_selector(html, None, true).as_deref(), Some("#d"));
    // current=d → c（tabindex=2）。
    assert_eq!(next_focus_selector(html, Some("#d"), true).as_deref(), Some("#c"));
    // current=c → a（文档序 tabindex=0/default）。
    assert_eq!(next_focus_selector(html, Some("#c"), true).as_deref(), Some("#a"));
    // backward：current=a → prev=c。
    assert_eq!(next_focus_selector(html, Some("#a"), false).as_deref(), Some("#c"));
    // 无 focusable → None。
    assert_eq!(
        next_focus_selector("<html><body><div>no focusable</div></body></html>", None, true),
        None
    );
}

/// R3254-H2：无 id/class 的同 tag 可聚焦元素——Tab 导航必须返回**唯一**选择器
///（此前返回歧义 `"input"` 命中文档第一个 input，纯键盘操作全部落到第一个元素）。
#[test]
fn test_next_focus_selector_returns_unique_for_same_tag_inputs() {
    let html = "<html><body><input><input><button>x</button></body></html>";
    let first = next_focus_selector(html, None, true).expect("first input");
    let second = next_focus_selector(html, Some(&first), true).expect("second input");
    let third = next_focus_selector(html, Some(&second), true).expect("button");
    assert_ne!(first, "input", "首元素选择器必须唯一，不能是歧义裸 tag");
    assert_ne!(second, "input", "第二元素选择器必须唯一，不能是歧义裸 tag");
    assert_ne!(first, second, "两个 input 的选择器必须不同");
    assert_eq!(third, "button");
    // 两个选择器各自命中且互不相同。
    let doc = parse_html(html);
    let n1 = find_by_selector(&doc, &first).expect("first resolves");
    let n2 = find_by_selector(&doc, &second).expect("second resolves");
    assert_ne!(n1, n2, "选择器必须指向不同节点");
    // 反向导航同样唯一。
    let back = next_focus_selector(html, Some(&second), false).expect("back");
    assert_eq!(back, first);
}

#[test]
fn test_insert_adjacent_html_e2e() {
    // 端到端：注入生产 shim + register_dom_callbacks，验证 insertAdjacentHTML JS 契约——
    // 调用入队 InsertAdjacentHtml mutation（sel + position + html 三参数透传）。
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
        "<html><body><ul id='list'><li>x</li></ul></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // beforeend：追加列表项。
    sandbox
        .execute("document.querySelector('#list').insertAdjacentHTML('beforeend', '<li>a</li>');")
        .unwrap();
    // afterbegin：首部插入。
    sandbox
        .execute("document.querySelector('#list').insertAdjacentHTML('afterbegin', '<li>0</li>');")
        .unwrap();
    // 非法 position：shim 不抛（host apply 时才错），但 mutation 仍入队（position 透传）。
    sandbox
        .execute("document.querySelector('#list').insertAdjacentHTML('nowhere', '<b/>');")
        .unwrap();

    let positions: Vec<String> = mutations
        .lock()
        .unwrap()
        .iter()
        .filter_map(|m| match m {
            DomMutation::InsertAdjacentHtml { position, .. } => Some(position.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        positions.len(),
        3,
        "三次 insertAdjacentHTML 均应入队 InsertAdjacentHtml mutation"
    );
    // 校验 position 透传（含非法值，host apply 时才错）。
    assert_eq!(
        positions.iter().map(String::as_str).collect::<Vec<_>>(),
        vec!["beforeend", "afterbegin", "nowhere"]
    );
}

#[test]
fn test_outer_html_e2e() {
    // 端到端：outerHTML getter 真实序列化（含自身 tag/属性/子树）+ setter 入队 SetOuterHtml。
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
        "<html><body><div id='t' class='c'>hi<span>x</span></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // getter：含自身 tag/属性 + 子树。
    sandbox
        .execute("globalThis.__o = document.querySelector('#t').outerHTML;")
        .unwrap();
    let outer = sandbox.execute("globalThis.__o").unwrap().value;
    assert!(outer.contains("<div"), "getter 含自身 tag\n{outer}");
    assert!(outer.contains("class=\"c\""), "getter 含属性\n{outer}");
    assert!(outer.contains("<span>x</span>"), "getter 含子树\n{outer}");

    // setter：入队 SetOuterHtml（selector + html 透传）。
    sandbox
        .execute("document.querySelector('#t').outerHTML = '<b>1</b>';")
        .unwrap();
    let set_mutation =
        mutations.lock().unwrap().iter().any(
            |m| matches!(m, DomMutation::SetOuterHtml { selector, html } if selector == "#t" && html == "<b>1</b>"),
        );
    assert!(set_mutation, "outerHTML setter 应入队 SetOuterHtml(#t, <b>1</b>)");
}

#[test]
fn test_prepend_order_e2e() {
    // prepend 多节点 + 字符串混合：参数序 == DOM 序（反序插入 afterbegin 保证）。
    // prepend(b, "str", i) on <div id=t>existing → <b></b>str<i></i>existing。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><div id='t'>existing</div></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    sandbox
        .execute(
            "var b = document.createElement('b');\
             var i = document.createElement('i');\
             document.querySelector('#t').prepend(b, 'str', i);",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    assert!(
        out.contains("<b></b>str<i></i>existing"),
        "prepend 应保持参数序（b,str,i）\n{out}"
    );
}

#[test]
fn test_before_after_order_e2e() {
    // before（前兄弟，正序 beforebegin）+ after（后兄弟，反序 afterend）。
    // 初始 <div id=t> 处于 body。before(x,y) → x,y 在 t 前；after(p,q) → p,q 在 t 后。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><div id='t'>x</div></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    sandbox
        .execute(
            "var x=document.createElement('x');var y=document.createElement('y');\
             var p=document.createElement('p');var q=document.createElement('q');\
             var t=document.querySelector('#t');\
             t.before(x,y); t.after(p,q);",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    // 期望 body 内顺序：x, y, t, p, q（before 正序在前、after 反序在后均保持参数序）。
    let ix = out.find("<x>").unwrap();
    let iy = out.find("<y>").unwrap();
    let it = out.find("<div id=\"t\">").unwrap();
    let ip = out.find("<p>").unwrap();
    let iq = out.find("<q>").unwrap();
    assert!(
        ix < iy && iy < it && it < ip && ip < iq,
        "before/after 应保持参数序 x<y<t<p<q\n{out}"
    );
}

#[test]
fn test_prepend_detached_noop_e2e() {
    // handle-only（detached）目标 prepend 无操作（无 parent/参考子，不抛）。
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
    // detached div.prepend(...) 不抛、不入队 InsertAdjacent*。
    sandbox
        .execute("var d=document.createElement('div'); d.prepend('x'); globalThis.__ok='done';")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__ok").unwrap().value, "done");
    let has_adj = mutations.lock().unwrap().iter().any(|m| {
        matches!(
            m,
            DomMutation::InsertAdjacentText { .. } | DomMutation::InsertAdjacentElement { .. }
        )
    });
    assert!(!has_adj, "detached 目标 prepend 不应入队 InsertAdjacent* mutation");
}

#[test]
fn test_replace_child_e2e() {
    // replaceChild(new, old)：在 old 位置替换为新节点，返回 old。父 [a,b] → replaceChild(newP,a) → [newP,b]。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><ul id='list'><li id='a'>A</li><li id='b'>B</li></ul></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    sandbox
        .execute(
            "var np = document.createElement('li'); np.id = 'new';\
             var list = document.querySelector('#list');\
             var old = list.replaceChild(np, document.querySelector('#a'));\
             globalThis.__ret = (old && old.id) || '';",
        )
        .unwrap();
    // spec：返回被替换的 old 节点（id=a）。
    assert_eq!(sandbox.execute("globalThis.__ret").unwrap().value, "a");
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    // new 在 a 原位置，a 被移除，b 保留。
    assert!(out.contains("<li id=\"new\">"), "replaceChild 应插入新节点\n{out}");
    assert!(!out.contains("<li id=\"a\">"), "replaceChild 应移除 old\n{out}");
    assert!(out.contains("<li id=\"b\">B</li>"), "replaceChild 应保留兄弟 b\n{out}");
    // 顺序：new 在 b 之前。
    let i_new = out.find("<li id=\"new\">").unwrap();
    let i_b = out.find("<li id=\"b\">").unwrap();
    assert!(i_new < i_b, "new 应在 b 之前（a 原位置）\n{out}");
}

#[test]
fn test_replace_with_e2e() {
    // replaceWith(x, y)：用 x,y 替换自身。body [t] → t.replaceWith(x,y) → [x,y]（t 移除）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><div id='t'>x</div></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);
    sandbox
        .execute(
            "var x=document.createElement('x');var y=document.createElement('y');\
             document.querySelector('#t').replaceWith(x, 'mid', y);",
        )
        .unwrap();
    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    assert!(!out.contains("<div id=\"t\">"), "replaceWith 应移除自身\n{out}");
    // 顺序：x, mid(text), y 保持参数序。
    let ix = out.find("<x>").unwrap();
    let imid = out.find("mid").unwrap();
    let iy = out.find("<y>").unwrap();
    assert!(ix < imid && imid < iy, "replaceWith 应保持参数序 x<mid<y\n{out}");
}

#[test]
fn test_node_level_traversal_e2e() {
    // 节点级遍历：childNodes/firstChild/lastChild（含文本/元素/注释）、
    // nextSibling/previousSibling（跨非元素节点）。经 JS 读属性 + 断言 nodeType/nodeValue。
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
        "<html><body><div id='t'>text1<span id='s'>x</span><!--c-->text2</div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // childNodes：4 个子（text/span/comment/text），nodeType 正确。
    sandbox
        .execute(
            "globalThis.__cn = document.querySelector('#t').childNodes;\
             globalThis.__len = __cn.length;\
             globalThis.__types = Array.prototype.map.call(__cn, function(n){return n.nodeType;}).join(',');\
             globalThis.__t0 = __cn[0].nodeValue;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__len").unwrap().value, "4");
    // nodeType: text(3), element(1), comment(8), text(3)。
    assert_eq!(sandbox.execute("globalThis.__types").unwrap().value, "3,1,8,3");
    assert_eq!(sandbox.execute("globalThis.__t0").unwrap().value, "text1");

    // firstChild/lastChild：文本节点。
    sandbox
        .execute(
            "globalThis.__fc = document.querySelector('#t').firstChild.nodeType;\
             globalThis.__fv = document.querySelector('#t').firstChild.nodeValue;\
             globalThis.__lc = document.querySelector('#t').lastChild.nodeValue;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__fc").unwrap().value, "3");
    assert_eq!(sandbox.execute("globalThis.__fv").unwrap().value, "text1");
    assert_eq!(sandbox.execute("globalThis.__lc").unwrap().value, "text2");

    // 空元素 childNodes.length=0、firstChild=null。
    sandbox
        .execute(
            "globalThis.__e = document.querySelector('#s').childNodes.length;\
             globalThis.__ef = document.querySelector('#s').firstChild;",
        )
        .unwrap();
    // #s 含文本 "x"（1 个 text 子）。
    assert_eq!(sandbox.execute("globalThis.__e").unwrap().value, "1");

    // nextSibling/previousSibling 跨非元素节点：span 的前兄弟=text1、后兄弟=comment。
    sandbox
        .execute(
            "var s = document.querySelector('#s');\
             globalThis.__ps = s.previousSibling.nodeValue;\
             globalThis.__ns = s.nextSibling.nodeType;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__ps").unwrap().value, "text1");
    assert_eq!(sandbox.execute("globalThis.__ns").unwrap().value, "8");
}

#[test]
fn test_create_document_fragment_e2e() {
    // 端到端：createDocumentFragment（nodeType 11 / nodeName）+ 建 fragment 子 + append 到 DOM
    // → flatten 子节点到目标（fragment 自身不入树）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><ul id='list'></ul></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var f = document.createDocumentFragment();\
             var a = document.createElement('li'); a.id = 'a';\
             var b = document.createElement('li'); b.id = 'b';\
             f.appendChild(a); f.appendChild(b);\
             globalThis.__nt = f.nodeType;\
             globalThis.__nn = f.nodeName;\
             document.querySelector('#list').appendChild(f);",
        )
        .unwrap();
    // fragment nodeType 11 / nodeName '#document-fragment'。
    assert_eq!(sandbox.execute("globalThis.__nt").unwrap().value, "11");
    assert_eq!(sandbox.execute("globalThis.__nn").unwrap().value, "#document-fragment");

    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    assert!(out.contains("<li id=\"a\">"), "flatten 后 li#a 应在 #list 内\n{out}");
    assert!(out.contains("<li id=\"b\">"), "flatten 后 li#b 应在 #list 内\n{out}");
    let ia = out.find("<li id=\"a\">").unwrap();
    let ib = out.find("<li id=\"b\">").unwrap();
    assert!(ia < ib, "flatten 保持子节点顺序 a<b\n{out}");

    // 入队了 AppendFragmentChildren（sel 版）。
    let has_flatten = mutations.lock().unwrap().iter().any(
        |m| matches!(m, DomMutation::AppendFragmentChildren { parent_selector, .. } if parent_selector == "#list"),
    );
    assert!(has_flatten, "appendChild(fragment) 应入队 AppendFragmentChildren");
}

#[test]
fn test_insert_before_fragment_flatten_e2e() {
    // R2688 self-review 修复验证：insertBefore(fragment, ref) 须 flatten 子节点（spec）。
    // 旧行为：插 fragment 节点本身 → childNodes 漏子（藏在被跳过的 fragment wrapper 内）+
    //   fragment 未清空。修复后：fragment 子移到 ref 前、fragment 清空。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><ul id='list'><li id='first'>F</li></ul></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    sandbox
        .execute(
            "var f = document.createDocumentFragment();\
             var a = document.createElement('li'); a.id = 'a';\
             var b = document.createElement('li'); b.id = 'b';\
             f.appendChild(a); f.appendChild(b);\
             var list = document.querySelector('#list');\
             list.insertBefore(f, list.firstChild);",
        )
        .unwrap();
    // 入队 InsertFragmentBefore（非 InsertBefore 插 fragment 节点本身）。
    let used_flatten =
        mutations.lock().unwrap().iter().any(
            |m| matches!(m, DomMutation::InsertFragmentBefore { parent_selector, .. } if parent_selector == "#list"),
        );
    assert!(
        used_flatten,
        "insertBefore(fragment, ref) 应入队 InsertFragmentBefore（flatten）"
    );

    let ms = mutations.lock().unwrap().clone();
    let (out, _handles) = apply_mutations_to_html_with_handles(&initial, &ms).unwrap();
    // flatten 后顺序：a, b, first（fragment 子在 first 之前）。
    let ia = out.find("<li id=\"a\">").unwrap();
    let ib = out.find("<li id=\"b\">").unwrap();
    let ifirst = out.find("<li id=\"first\">").unwrap();
    assert!(ia < ib && ib < ifirst, "flatten 后 a<b<first\n{out}");
}

#[test]
fn test_fragment_flatten_all_insertion_paths_e2e() {
    // R2689：闭合 fragment flatten 同类 bug——prepend/before/after/replaceChild 接 fragment
    // 须 flatten 子节点（非插 wrapper）。经 JS→mutation→apply 序列化验最终 DOM 序。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let initial = "<html><body><div id='t'>X</div></body></html>".to_string();
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(initial.clone()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // prepend(fragment)：fragment 子成为 #t 首子（在 X 前）。
    sandbox
        .execute(
            "var f1=document.createDocumentFragment();\
             var a=document.createElement('a'); var b=document.createElement('b');\
             f1.appendChild(a); f1.appendChild(b);\
             document.querySelector('#t').prepend(f1);",
        )
        .unwrap();
    let ms1 = mutations.lock().unwrap().clone();
    let (out1, _) = apply_mutations_to_html_with_handles(&initial, &ms1).unwrap();
    // #t 内：a, b, X（fragment 子在前）。
    let o1a = out1.find("<a>").unwrap();
    let o1b = out1.find("<b>").unwrap();
    let o1x = out1.find("X</div>").unwrap();
    assert!(o1a < o1b && o1b < o1x, "prepend(fragment): a<b<X\n{out1}");

    // before(fragment)：fragment 子作 #t 前兄弟。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "var f2=document.createDocumentFragment();\
             var c=document.createElement('c');\
             f2.appendChild(c);\
             document.querySelector('#t').before(f2);",
        )
        .unwrap();
    let ms2 = mutations.lock().unwrap().clone();
    let (out2, _) = apply_mutations_to_html_with_handles(&out1, &ms2).unwrap();
    let o2c = out2.find("<c>").unwrap();
    let o2t = out2.find("<div id=\"t\">").unwrap();
    assert!(o2c < o2t, "before(fragment): c 在 #t 前\n{out2}");

    // after(fragment)：fragment 子作 #t 后兄弟。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "var f3=document.createDocumentFragment();\
             var d=document.createElement('d');\
             f3.appendChild(d);\
             document.querySelector('#t').after(f3);",
        )
        .unwrap();
    let ms3 = mutations.lock().unwrap().clone();
    let (out3, _) = apply_mutations_to_html_with_handles(&out2, &ms3).unwrap();
    let o3t = out3.find("<div id=\"t\">").unwrap();
    let o3d = out3.find("<d>").unwrap();
    assert!(o3t < o3d, "after(fragment): d 在 #t 后\n{out3}");

    // replaceChild(fragment, old)：fragment 子替换 #t（old=#t）。
    mutations.lock().unwrap().clear();
    sandbox
        .execute(
            "var f4=document.createDocumentFragment();\
             var e=document.createElement('e');\
             f4.appendChild(e);\
             var body=document.querySelector('body');\
             body.replaceChild(f4, document.querySelector('#t'));",
        )
        .unwrap();
    let ms4 = mutations.lock().unwrap().clone();
    let (out4, _) = apply_mutations_to_html_with_handles(&out3, &ms4).unwrap();
    assert!(out4.contains("<e>"), "replaceChild(fragment): e 替换 #t\n{out4}");
    assert!(
        !out4.contains("<div id=\"t\">"),
        "replaceChild(fragment): #t 应被移除\n{out4}"
    );
}

#[test]
fn test_parent_node_nested_e2e() {
    // R2690：parentNode/parentElement 嵌套正确性（旧 stub 恒返 body）。
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
        "<html><body><div id='outer'><div id='inner'>x</div></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    // inner.parentNode.id === 'outer'（旧 stub 错返 body → id ''）。
    sandbox
        .execute("globalThis.__p = document.querySelector('#inner').parentNode.id;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__p").unwrap().value, "outer");
    // inner.parentElement 同。
    sandbox
        .execute("globalThis.__pe = document.querySelector('#inner').parentElement.id;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__pe").unwrap().value, "outer");
    // outer.parentNode.tagName === 'BODY'。
    sandbox
        .execute("globalThis.__op = document.querySelector('#outer').parentNode.tagName;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__op").unwrap().value, "BODY");
    // js-dom M4 R79：html.parentNode 现为 document（spec Node.parentNode：documentElement 的
    // 父是 Document——旧断言 null；contains/compareDocumentPosition 的树链前提）。
    sandbox
        .execute("globalThis.__hp = document.querySelector('html').parentNode === document;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__hp").unwrap().value, "true");
}
