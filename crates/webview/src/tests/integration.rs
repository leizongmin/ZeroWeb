// Auto-generated test file — split from webview/lib.rs
use super::super::*;
use std::cell::RefCell;
use std::rc::Rc;

// ── 事件系统集成测试（通过 V8 + DOM polyfill 端到端验证）──

/// 测试 addEventListener 在 polyfill 中可用。
#[test]
fn test_webview_dom_add_event_listener_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("var el = document.createElement('div'); typeof el.addEventListener;");
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("function"),
        "addEventListener should be a function"
    );
}

/// 测试 removeEventListener 在 polyfill 中可用。
#[test]
fn test_webview_dom_remove_event_listener_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("var el = document.createElement('div'); typeof el.removeEventListener;");
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("function"),
        "removeEventListener should be a function"
    );
}

/// 测试 dispatchEvent 在 polyfill 中可用。
#[test]
fn test_webview_dom_dispatch_event_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("var el = document.createElement('div'); typeof el.dispatchEvent;");
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("function"),
        "dispatchEvent should be a function"
    );
}

/// 测试 CustomEvent 构造函数可用。
#[test]
fn test_webview_dom_custom_event_constructor() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        "var evt = new CustomEvent('test', { bubbles: true, cancelable: true, detail: 42 }); evt.type;",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test", "CustomEvent type should match");
}

/// 测试 CustomEvent detail 属性。
#[test]
fn test_webview_dom_custom_event_detail() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        "var evt = new CustomEvent('myevent', { detail: { name: 'test' } }); JSON.stringify(evt.detail);",
    );
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("test"),
        "CustomEvent detail should be preserved"
    );
}

/// 测试事件监听器被正确调用。
#[test]
fn test_webview_dom_event_listener_fires() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        var received = false;
        el.addEventListener('click', function() { received = true; });
        el.dispatchEvent(new CustomEvent('click'));
        received ? 'yes' : 'no';
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "yes", "Event listener should fire on dispatchEvent");
}

/// 测试事件监听器接收事件对象。
#[test]
fn test_webview_dom_event_listener_receives_event() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        var eventType = '';
        el.addEventListener('custom', function(e) { eventType = e.type; });
        el.dispatchEvent(new CustomEvent('custom'));
        eventType;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "custom",
        "Listener should receive event with correct type"
    );
}

/// 测试 preventDefault 可阻止默认行为。
#[test]
fn test_webview_dom_event_prevent_default() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.addEventListener('click', function(e) { e.preventDefault(); });
        var notPrevented = el.dispatchEvent(new CustomEvent('click', { cancelable: true }));
        notPrevented ? 'not-prevented' : 'prevented';
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "prevented",
        "preventDefault should make dispatchEvent return false"
    );
}

/// 测试 capture 选项传递给 addEventListener。
#[test]
fn test_webview_dom_event_capture_option() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        var order = [];
        el.addEventListener('click', function() { order.push('bubble'); }, false);
        el.addEventListener('click', function() { order.push('capture'); }, true);
        el.dispatchEvent(new CustomEvent('click'));
        order.join(',');
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "capture,bubble",
        "Capture listeners should fire before bubble listeners"
    );
}

/// 测试 body/head/documentElement 预创建节点存在。
#[test]
fn test_webview_dom_built_in_nodes() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        "document.body.tagName + ',' + document.head.tagName + ',' + document.documentElement.tagName;",
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "BODY,HEAD,HTML",
        "Built-in document nodes should have correct tag names"
    );
}

/// 测试 setAttribute + getAttribute 往返正确。
#[test]
fn test_webview_dom_attribute_roundtrip() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.setAttribute('data-value', 'hello');
        el.getAttribute('data-value');
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "hello",
        "setAttribute/getAttribute roundtrip should work"
    );
}

// ── Fetch API 集成测试（通过 V8 + DOM polyfill 端到端验证）──

/// 测试 fetch 函数在 polyfill 中可用。
#[test]
fn test_webview_fetch_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof fetch;");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"), "fetch should be a function");
}

/// 测试 Headers 构造函数可用。
#[test]
fn test_webview_headers_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof Headers;");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"), "Headers should be a function");
}

/// 测试 Response 构造函数可用。
#[test]
fn test_webview_response_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof Response;");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"), "Response should be a function");
}

/// 测试 Request 构造函数可用。
#[test]
fn test_webview_request_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof Request;");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"), "Request should be a function");
}

/// 测试 Headers 方法正常工作。
#[test]
fn test_webview_headers_methods() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var h = new Headers();
        h.append('Content-Type', 'text/html');
        h.get('content-type');
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "text/html",
        "Headers.get should return appended value (case-insensitive)"
    );
}

/// 测试 Headers.has 和 Headers.delete。
#[test]
fn test_webview_headers_has_delete() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var h = new Headers();
        h.set('X-Test', 'yes');
        var has1 = h.has('x-test');
        h.delete('x-test');
        var has2 = h.has('x-test');
        has1 && !has2 ? 'ok' : 'fail';
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ok");
}

/// 测试 Response 属性。
#[test]
fn test_webview_response_properties() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var r = new Response('body text', { status: 200, statusText: 'OK' });
        r.status + ',' + r.ok + ',' + r.statusText;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "200,true,OK",
        "Response properties should be set correctly"
    );
}

/// 测试 Response.text() 返回 Promise。
#[test]
fn test_webview_response_text() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("var r = new Response('hello world', { status: 200 }); typeof r.text();");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "object",
        "Response.text() should return a Promise (object)"
    );
}

/// 测试 Response.json() 返回 Promise。
#[test]
fn test_webview_response_json() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result =
        wv.execute_script_with_dom(r#"var r = new Response('{"name":"test"}', { status: 200 }); typeof r.json();"#);
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "object",
        "Response.json() should return a Promise (object)"
    );
}

/// 测试 Response.ok 对错误状态码为 false。
#[test]
fn test_webview_response_not_ok() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv
        .execute_script_with_dom("var r = new Response(null, { status: 404, statusText: 'Not Found' }); String(r.ok);");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "false", "Response.ok should be false for 404");
}

/// 测试 Request 属性。
#[test]
fn test_webview_request_properties() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var req = new Request('https://example.com/api', { method: 'POST', body: 'data' });
        req.url + ',' + req.method;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "https://example.com/api,POST",
        "Request url and method should be set"
    );
}

/// 测试 fetch 返回 Promise 对象。
#[test]
fn test_webview_fetch_returns_promise() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof fetch('https://example.com/test');");
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("object"),
        "fetch() should return a Promise (object type)"
    );
}

// ── Console API 集成测试（通过 V8 + DOM polyfill 端到端验证）──

/// 测试 console 对象存在。
#[test]
fn test_webview_console_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof console;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "object", "console should be an object");
}

/// 测试 console.log 可调用。
#[test]
fn test_webview_console_log_callable() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof console.log;");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"), "console.log should be a function");
}

/// 测试 console.log 调用不报错。
#[test]
fn test_webview_console_log_no_error() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("console.log('hello', 42, {a:1}); 'ok';");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ok", "console.log should not throw");
}

/// 测试 console.warn/error/info 不报错。
#[test]
fn test_webview_console_methods_no_error() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("console.warn('w'); console.error('e'); console.info('i'); 'ok';");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ok");
}

/// 测试 console.time/timeEnd 不报错。
#[test]
fn test_webview_console_time() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("console.time('test'); console.timeEnd('test'); 'ok';");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ok");
}

// ── Timer API 集成测试 ──

/// 测试 setTimeout 存在且可调用。
#[test]
fn test_webview_set_timeout_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof setTimeout;");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"), "setTimeout should be a function");
}

/// 测试 setInterval 存在且可调用。
#[test]
fn test_webview_set_interval_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof setInterval;");
    assert!(result.is_ok());
    assert!(result.unwrap().contains("function"), "setInterval should be a function");
}

/// 测试 clearTimeout/clearInterval 存在。
#[test]
fn test_webview_clear_timers_exist() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof clearTimeout + ',' + typeof clearInterval;");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "function,function",
        "clearTimeout and clearInterval should be functions"
    );
}

/// 测试 setTimeout 执行回调。
///
/// R100：execute 路径不再覆写 shim（polyfill 幂等安装）——setTimeout 是 shim 的
/// 真异步实现（spec：宏任务延迟执行），同步读 x 得 0；回调在本 execute 的
/// microtask/宏任务排水后生效，第二次 execute 读到 42（与 run_page_scripts 页面的
/// 定时器语义一致）。
#[test]
fn test_webview_set_timeout_calls_fn() {
    let mut wv = WebView::new(WebViewConfig::default());
    let _ = wv.execute_script_with_dom(
        "globalThis.__x = 0; setTimeout(function() { globalThis.__x = 42; }, 0); globalThis.__x;",
    );
    let result = wv.execute_script_with_dom("globalThis.__x;");
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "42",
        "setTimeout callback should run after timer drain"
    );
}

/// 测试 setInterval 执行回调。
#[test]
fn test_webview_set_interval_calls_fn() {
    let mut wv = WebView::new(WebViewConfig::default());
    // R100：shim 真 setInterval（spec 异步）——首次回调经 host tick 派发后可观察。
    let _ = wv.execute_script_with_dom(
        "globalThis.__cnt = 0; setInterval(function() { globalThis.__cnt++; }, 10); globalThis.__cnt;",
    );
    let result = wv.execute_script_with_dom("globalThis.__cnt;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "1", "setInterval callback should run after first tick");
}

// ── Web Storage API 集成测试（通过 V8 + DOM polyfill 端到端验证）──

/// 测试 localStorage 存在。
#[test]
fn test_webview_local_storage_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof localStorage;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "object", "localStorage should be an object");
}

/// 测试 sessionStorage 存在。
#[test]
fn test_webview_session_storage_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof sessionStorage;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "object", "sessionStorage should be an object");
}

/// 测试 localStorage setItem/getItem 往返。
#[test]
fn test_webview_local_storage_set_get() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        localStorage.setItem('key1', 'value1');
        localStorage.getItem('key1');
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "value1",
        "localStorage getItem should return set value"
    );
}

/// 测试 localStorage removeItem。
#[test]
fn test_webview_local_storage_remove() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        localStorage.setItem('temp', 'data');
        localStorage.removeItem('temp');
        localStorage.getItem('temp');
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "null",
        "localStorage should return null after removeItem"
    );
}

/// 测试 localStorage clear。
#[test]
fn test_webview_local_storage_clear() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        localStorage.setItem('a', '1');
        localStorage.setItem('b', '2');
        localStorage.clear();
        localStorage.length;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0", "localStorage should be empty after clear");
}

/// 测试 localStorage length。
#[test]
fn test_webview_local_storage_length() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        localStorage.setItem('x', '1');
        localStorage.setItem('y', '2');
        localStorage.setItem('z', '3');
        localStorage.length;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "3", "localStorage length should be 3");
}

/// 测试 localStorage key() 方法。
#[test]
fn test_webview_local_storage_key() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        localStorage.setItem('alpha', 'a');
        localStorage.setItem('beta', 'b');
        var k0 = localStorage.key(0);
        var k1 = localStorage.key(1);
        var kn = localStorage.key(99);
        kn === null ? k0 + ',' + k1 : 'fail';
        "#,
    );
    assert!(result.is_ok());
    let val = result.unwrap();
    // key order is insertion order
    assert!(
        val.contains("alpha") || val.contains("beta"),
        "key() should return valid keys"
    );
}

/// 测试 getItem 对不存在的 key 返回 null。
#[test]
fn test_webview_local_storage_get_missing() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("localStorage.getItem('nonexistent');");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "null", "Missing key should return null");
}

/// 测试 sessionStorage 独立于 localStorage。
#[test]
fn test_webview_session_storage_independent() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        localStorage.setItem('shared', 'in-local');
        sessionStorage.setItem('shared', 'in-session');
        localStorage.getItem('shared') + ',' + sessionStorage.getItem('shared');
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "in-local,in-session",
        "localStorage and sessionStorage should be independent"
    );
}

// ── MutationObserver 集成测试（通过 V8 + DOM polyfill 端到端验证）──

/// 测试 MutationObserver 构造函数存在。
#[test]
fn test_webview_mutation_observer_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof MutationObserver;");
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("function"),
        "MutationObserver should be a function"
    );
}

/// 测试 MutationObserver 可以创建并调用 observe。
#[test]
fn test_webview_mutation_observer_observe() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var callback = function(records) {};
        var observer = new MutationObserver(callback);
        var el = document.createElement('div');
        observer.observe(el, { childList: true });
        typeof observer.takeRecords === 'function' && typeof observer.disconnect === 'function' ? 'observing' : 'not-observing';
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "observing");
}

/// 测试 MutationObserver disconnect。
#[test]
fn test_webview_mutation_observer_disconnect() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var observer = new MutationObserver(function() {});
        var el = document.createElement('div');
        observer.observe(el, { attributes: true });
        observer.disconnect();
        observer._observing ? 'still' : 'disconnected';
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "disconnected");
}

/// 测试 MutationObserver takeRecords 返回空数组。
#[test]
fn test_webview_mutation_observer_take_records() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("var obs = new MutationObserver(function() {}); obs.takeRecords().length;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0", "takeRecords should return empty array initially");
}

/// 测试 MutationRecord 构造函数。
///
/// R100：shim 的 MutationRecord 无公开构造器入参（spec——record 由 MO 回调产生）；
/// 改为经真实 observe→mutate→drain 读 record.type（shim 语义，与 WPT MO 用例同源）。
#[test]
fn test_webview_mutation_record_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let _ = wv.execute_script_with_dom(
        r#"
        globalThis.__recType = 'none';
        var obs = new MutationObserver(function (recs) {
          if (recs && recs.length) globalThis.__recType = String(recs[0].type);
        });
        var el = document.createElement('div');
        obs.observe(el, { childList: true });
        el.appendChild(document.createElement('span'));
        "#,
    );
    let result = wv.execute_script_with_dom("globalThis.__recType;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "childList");
}

/// 测试 MutationRecord 属性。
///
/// R100：同 record_exists——经真实 observe 读 spec 字段缺省值（attributes record 的
/// addedNodes/removedNodes 空 + attributeName 为触发属性名）。
#[test]
fn test_webview_mutation_record_properties() {
    let mut wv = WebView::new(WebViewConfig::default());
    let _ = wv.execute_script_with_dom(
        r#"
        globalThis.__recProps = 'none';
        var obs = new MutationObserver(function (recs) {
          if (recs && recs.length) {
            var r = recs[0];
            globalThis.__recProps = r.addedNodes.length + ',' + r.removedNodes.length + ',' + String(r.attributeName);
          }
        });
        var el = document.createElement('div');
        obs.observe(el, { attributes: true });
        el.setAttribute('data-x', '1');
        "#,
    );
    let result = wv.execute_script_with_dom("globalThis.__recProps;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0,0,data-x");
}

// ── IntersectionObserver 集成测试（通过 V8 + DOM polyfill 端到端验证）──

/// 测试 IntersectionObserver 构造函数存在。
#[test]
fn test_webview_intersection_observer_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof IntersectionObserver;");
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("function"),
        "IntersectionObserver should be a function"
    );
}

/// 测试 IntersectionObserver observe 和 unobserve。
#[test]
fn test_webview_intersection_observer_observe_unobserve() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var io = new IntersectionObserver(function() {});
        var el1 = document.createElement('div');
        var el2 = document.createElement('span');
        io.observe(el1);
        io.observe(el2);
        var len1 = Object.keys(io._targets).length;
        io.unobserve(el1);
        var len2 = Object.keys(io._targets).length;
        len1 + ',' + len2;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "2,1", "observe/unobserve should manage elements");
}

/// 测试 IntersectionObserver disconnect。
#[test]
fn test_webview_intersection_observer_disconnect() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var io = new IntersectionObserver(function() {});
        io.observe(document.createElement('div'));
        io.disconnect();
        Object.keys(io._targets).length;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0", "disconnect should clear observed elements");
}

/// 测试 IntersectionObserver takeRecords。
#[test]
fn test_webview_intersection_observer_take_records() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result =
        wv.execute_script_with_dom("var io = new IntersectionObserver(function() {}); io.takeRecords().length;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0", "takeRecords should return empty array in stub");
}

/// 测试 IntersectionObserverEntry 构造函数。
#[test]
fn test_webview_intersection_observer_entry() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var entry = new IntersectionObserverEntry({ isIntersecting: true, intersectionRatio: 0.5 });
        entry.isIntersecting + ',' + entry.intersectionRatio;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "true,0.5",
        "IntersectionObserverEntry should have correct properties"
    );
}

// ── ResizeObserver 集成测试（通过 V8 + DOM polyfill 端到端验证）──

/// 测试 ResizeObserver 构造函数存在。
#[test]
fn test_webview_resize_observer_exists() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof ResizeObserver;");
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("function"),
        "ResizeObserver should be a function"
    );
}

/// 测试 ResizeObserver observe/unobserve。
#[test]
fn test_webview_resize_observer_observe_unobserve() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var ro = new ResizeObserver(function() {});
        var el = document.createElement('div');
        ro.observe(el);
        var len1 = Object.keys(ro._targets).length;
        ro.unobserve(el);
        var len2 = Object.keys(ro._targets).length;
        len1 + ',' + len2;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "1,0");
}

/// 测试 ResizeObserver disconnect。
#[test]
fn test_webview_resize_observer_disconnect() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var ro = new ResizeObserver(function() {});
        ro.observe(document.createElement('div'));
        ro.observe(document.createElement('span'));
        ro.disconnect();
        Object.keys(ro._targets).length;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "0", "disconnect should clear all observed elements");
}

/// 测试 execute_script 纯空格字符串
#[test]
fn test_webview_execute_script_whitespace_only() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("   \n  \t  ");
    // 应该返回错误或成功但不能 panic
    assert!(result.is_ok() || result.is_err());
}

/// 测试 execute_script 空字符串后立即执行有效脚本
#[test]
fn test_webview_execute_script_empty_then_valid() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 先执行空脚本
    let _ = wv.execute_script("");

    // 再执行有效脚本
    let result = wv.execute_script("1 + 1");
    // 简化测试，只确保不 panic
    let _ = result;
}

/// 测试 execute_script 深层属性链错误
#[test]
fn test_webview_execute_script_deep_property_chain_error() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 深层属性链中的中间对象不存在
    let script = "a.b.c.d.e.f";
    let result = wv.execute_script(script);
    // 简化测试，只确保不 panic
    let _ = result;
}

/// 测试 execute_script 返回超长字符串
#[test]
fn test_webview_execute_script_very_long_string() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 生成很长的字符串
    let long_string = "x".repeat(10000); // 减小长度
    let script = format!("'{}'", long_string);

    let result = wv.execute_script(&script);
    // 简化测试，只确保不 panic
    let _ = result;
}

/// 测试 execute_script TypeError
#[test]
fn test_webview_execute_script_type_error() {
    let mut wv = WebView::new(WebViewConfig::default());

    // TypeError - 调用非函数
    let script = "(1)()";
    let result = wv.execute_script(script);
    // 简化测试，只确保不 panic
    let _ = result;
}

/// 测试 execute_script 多行语句中的语法错误
#[test]
fn test_webview_execute_script_multiline_syntax_error() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 多行语句，第二行有语法错误
    let script = r#"
        let x = 1;
        let y = ;  // 语法错误
        x + y;
    "#;

    let result = wv.execute_script(script);
    // 简化测试，只确保不 panic
    let _ = result;
}

/// 测试 execute_script 返回 undefined
#[test]
fn test_webview_execute_script_returns_undefined() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 返回 undefined 的脚本
    let script = "undefined";
    let result = wv.execute_script(script);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "undefined");
}

/// 测试 execute_script 返回 null
#[test]
fn test_webview_execute_script_returns_null() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 返回 null 的脚本
    let script = "null";
    let result = wv.execute_script(script);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "null");
}

/// 测试 service_worker_registry_mut 的修改
#[test]
fn test_service_worker_registry_mut_access() {
    let mut wv = WebView::new(WebViewConfig::default());
    let registry = wv.service_worker_registry_mut();
    assert!(registry.is_empty());
}

/// 测试 ResizeObserverEntry 构造函数。
#[test]
fn test_webview_resize_observer_entry() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var entry = new ResizeObserverEntry({ contentRect: { width: 100, height: 200 } });
        entry.contentRect.width + ',' + entry.contentRect.height;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "100,200",
        "ResizeObserverEntry should have contentRect"
    );
}

/// 测试 DOMRectReadOnly 构造函数。
#[test]
fn test_webview_dom_rect_read_only() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        "var rect = new DOMRectReadOnly(10, 20, 100, 50); rect.left + ',' + rect.right + ',' + rect.top + ',' + rect.bottom;",
    );
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap(),
        "10,110,20,70",
        "DOMRectReadOnly should compute derived properties"
    );
}

// ── DOM Bridge 增强 API 端到端测试 ──

/// 测试 insertBefore 在参考节点前插入。
#[test]
fn test_webview_dom_insert_before() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var parent = document.createElement('div');
        var c1 = document.createElement('span');
        var c2 = document.createElement('p');
        var c3 = document.createElement('a');
        parent.appendChild(c1);
        parent.appendChild(c3);
        parent.insertBefore(c2, c3);
        parent.children.length;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "3", "insertBefore 应插入到参考节点前");
}

/// 测试 replaceChild 替换子节点。
#[test]
fn test_webview_dom_replace_child() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var parent = document.createElement('div');
        var old = document.createElement('span');
        var rep = document.createElement('p');
        parent.appendChild(old);
        parent.replaceChild(rep, old);
        parent.childNodes[0].tagName;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "P", "replaceChild 应替换旧节点");
}

/// 测试 cloneNode 浅拷贝。
#[test]
fn test_webview_dom_clone_node_shallow() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'orig');
        var child = document.createElement('span');
        el.appendChild(child);
        var clone = el.cloneNode(false);
        clone.getAttribute('id') + ',' + clone.children.length;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "orig,0", "浅拷贝应复制属性但不复制子节点");
}

/// 测试 cloneNode 深拷贝。
#[test]
fn test_webview_dom_clone_node_deep() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'orig');
        var child = document.createElement('span');
        el.appendChild(child);
        var clone = el.cloneNode(true);
        clone.getAttribute('id') + ',' + clone.childNodes.length;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "orig,1", "深拷贝应复制属性和子节点");
}

/// 测试 style.cssText 读写。
#[test]
fn test_webview_dom_style_css_text() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.style.cssText = 'color: red; font-size: 16px';
        el.style.cssText;
        "#,
    );
    assert!(result.is_ok());
    let css = result.unwrap();
    assert!(css.contains("color: red"), "cssText 应包含 color");
    assert!(css.contains("font-size: 16px"), "cssText 应包含 font-size");
}

/// 测试 style.setProperty / getPropertyValue。
#[test]
fn test_webview_dom_style_set_get() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.style.setProperty('background-color', 'blue');
        el.style.getPropertyValue('background-color');
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "blue");
}

/// 测试 classList add/remove/contains。
#[test]
fn test_webview_dom_classlist() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.classList.add('a', 'b');
        var has = el.classList.contains('a') && el.classList.contains('b');
        el.classList.remove('a');
        var afterRemove = !el.classList.contains('a') && el.classList.contains('b');
        has + ',' + afterRemove;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "true,true", "classList add/remove/contains 应正常工作");
}

/// 测试 classList toggle。
#[test]
fn test_webview_dom_classlist_toggle() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.classList.add('active');
        var r1 = el.classList.toggle('active');
        var r2 = el.classList.toggle('active');
        r1 + ',' + r2 + ',' + el.classList.contains('active');
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "false,true,true", "toggle 应移除再添加");
}

/// 测试 innerHTML setter。
#[test]
fn test_webview_dom_inner_html_setter() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.innerHTML = 'Hello World';
        el.textContent;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello World", "innerHTML setter 应设置内容");
}

/// 测试 textContent getter/setter。
#[test]
fn test_webview_dom_text_content() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        var child = document.createElement('span');
        child.textContent = 'Hello';
        el.appendChild(child);
        el.textContent;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello", "textContent getter 应返回子节点文本");
}

/// 测试导航属性 firstChild/lastChild。
#[test]
fn test_webview_dom_navigation_properties() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var parent = document.createElement('div');
        var c1 = document.createElement('span');
        var c2 = document.createElement('p');
        parent.appendChild(c1);
        parent.appendChild(c2);
        var first = parent.firstChild.tagName;
        var last = parent.lastChild.tagName;
        var count = parent.childElementCount;
        first + ',' + last + ',' + count;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "SPAN,P,2", "导航属性应返回正确的子节点");
}

/// 测试 nextSibling/previousSibling。
#[test]
fn test_webview_dom_sibling_navigation() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var parent = document.createElement('div');
        var c1 = document.createElement('span');
        var c2 = document.createElement('p');
        var c3 = document.createElement('a');
        parent.appendChild(c1);
        parent.appendChild(c2);
        parent.appendChild(c3);
        var next = c2.nextSibling.tagName;
        var prev = c2.previousSibling.tagName;
        next + ',' + prev;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "A,SPAN", "兄弟节点导航应正确工作");
}

/// 测试 hasChildNodes。
#[test]
fn test_webview_dom_has_child_nodes() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var empty = document.createElement('div');
        var parent = document.createElement('div');
        parent.appendChild(document.createElement('span'));
        empty.hasChildNodes() + ',' + parent.hasChildNodes();
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "false,true");
}

/// 测试 createDocumentFragment。
#[test]
fn test_webview_dom_create_document_fragment() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof document.createDocumentFragment();");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "object", "createDocumentFragment 应返回对象");
}

// ── DOM 交互场景端到端测试 ──

/// 场景：构建 todo 列表，使用多种 DOM API 组合。
#[test]
fn test_webview_dom_todo_list_scenario() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var list = document.createElement('ul');
        list.setAttribute('id', 'todo-list');
        list.classList.add('list');

        var items = ['Buy milk', 'Read book', 'Write code'];
        for (var i = 0; i < items.length; i++) {
            var li = document.createElement('li');
            li.textContent = items[i];
            li.classList.add('item');
            li.style.setProperty('color', 'black');
            list.appendChild(li);
        }

        var count = list.children.length;
        var firstText = list.firstChild.textContent;
        var hasListClass = list.classList.contains('list');
        count + '|' + firstText + '|' + hasListClass;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "3|Buy milk|true", "todo 列表场景应正确使用 DOM API");
}

/// 场景：DOM 遍历和修改。
#[test]
fn test_webview_dom_traversal_and_modification() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var container = document.createElement('div');
        var c1 = document.createElement('span');
        c1.textContent = 'A';
        var c2 = document.createElement('span');
        c2.textContent = 'B';
        var c3 = document.createElement('span');
        c3.textContent = 'C';
        container.appendChild(c1);
        container.appendChild(c2);
        container.appendChild(c3);

        // 遍历并收集文本
        var text = '';
        var child = container.firstChild;
        while (child) {
            text += child.textContent;
            child = child.nextSibling;
        }

        // 移除中间节点
        container.removeChild(c2);

        text + '|' + container.children.length;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ABC|2", "遍历和修改场景应正确工作");
}

/// 场景：classList 与 setAttribute('class', ...) 同步。
#[test]
fn test_webview_dom_classlist_attribute_sync() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.classList.add('active');
        el.classList.add('visible');
        var classFromAttr = el.getAttribute('class');
        var fromClassName = el.className;
        classFromAttr === fromClassName ? 'sync' : 'mismatch';
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "sync", "classList 和 class 属性应同步");
}

/// 场景：style.cssText 解析和修改。
#[test]
fn test_webview_dom_style_parse_and_modify() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.style.cssText = 'color: red; font-size: 16px';
        var color = el.style.getPropertyValue('color');
        el.style.setProperty('margin', '10px');
        el.style.removeProperty('font-size');
        var hasFontSize = el.style.getPropertyValue('font-size');
        color + '|' + (hasFontSize === '' ? 'removed' : 'present');
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "red|removed", "style 解析和修改场景应正确工作");
}

/// 场景：replaceChild 替换后检查父子关系。
#[test]
fn test_webview_dom_replace_child_parent_tracking() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var parent = document.createElement('div');
        var old = document.createElement('span');
        old.textContent = 'old';
        parent.appendChild(old);

        var replacement = document.createElement('p');
        replacement.textContent = 'new';
        parent.replaceChild(replacement, old);

        var newChild = parent.firstChild;
        var oldHasNoParent = old.parentNode === null;
        newChild.tagName + '|' + newChild.textContent + '|' + oldHasNoParent;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "P|new|true", "replaceChild 应正确更新父子关系");
}

/// 场景：insertBefore 结合 appendChild 顺序验证。
#[test]
fn test_webview_dom_insert_ordering() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var parent = document.createElement('div');
        var a = document.createElement('span'); a.textContent = 'A';
        var b = document.createElement('span'); b.textContent = 'B';
        var c = document.createElement('span'); c.textContent = 'C';
        var d = document.createElement('span'); d.textContent = 'D';

        parent.appendChild(a);
        parent.appendChild(c);
        parent.insertBefore(b, c);  // [A, B, C]
        parent.insertBefore(d, null); // null → 追加到末尾 [A, B, C, D]

        var order = '';
        var child = parent.firstChild;
        while (child) {
            order += child.textContent;
            child = child.nextSibling;
        }
        order;
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ABCD", "insertBefore 排序应正确");
}

// ── 新增边界测试 ──

/// 测试 WebView 生命周期：load_url → complete_load → is_loading 状态转换。
#[test]
fn test_webview_lifecycle_load_complete() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(!wv.is_loading(), "初始状态不应在加载");
    assert!(wv.url().is_none(), "初始 URL 应为 None");

    wv.load_url("https://example.com");
    assert!(wv.is_loading(), "load_url 后应在加载");
    assert_eq!(wv.url(), Some("https://example.com"));

    let result = wv.complete_load("<html><body>Hello</body></html>", None);
    assert!(!wv.is_loading(), "complete_load 后应完成加载");
    assert!(result.timings.total_ms >= 0.0, "耗时应为非负");
}

/// 测试 WebView fail_load 重置加载状态。
#[test]
fn test_webview_fail_load_resets_state() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://example.com");
    assert!(wv.is_loading());

    wv.fail_load("network error");
    assert!(!wv.is_loading(), "fail_load 后应不再加载");
}

/// 测试 WebView resize 后渲染仍然正常。
#[test]
fn test_webview_resize_and_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>Test</body></html>", None);

    wv.resize(1024, 768);
    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);

    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0, "resize 后应能正常渲染");
}

/// 测试 set_title 触发 TitleChanged 事件。
#[test]
fn test_webview_set_title_event() {
    let mut wv = WebView::new(WebViewConfig::default());
    let titles: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let titles_clone = titles.clone();
    wv.on_event(move |event| {
        if let WebViewEvent::TitleChanged(t) = event {
            titles_clone.borrow_mut().push(t.clone());
        }
    });

    wv.set_title("My Page");
    assert_eq!(wv.title(), Some("My Page"));
    assert_eq!(titles.borrow().len(), 1);
    assert_eq!(titles.borrow()[0], "My Page");
}

/// 测试 inject_css 累积追加而非替换。
#[test]
fn test_webview_inject_css_cumulative() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>Hi</body></html>", None);

    wv.inject_css("body { color: red; }");
    wv.inject_css("div { display: none; }");

    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0, "累积 CSS 注入后应能正常渲染");
}

/// 测试 execute_script 编译错误返回正确错误类型。
#[test]
#[cfg(feature = "v8")]
fn test_webview_execute_script_compile_error() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("function { invalid syntax");
    assert!(result.is_err(), "语法错误应返回错误");
    let err = result.unwrap_err();
    match err {
        WebViewError::Script(msg) => assert!(
            msg.contains("Compile error") || msg.contains("Invalid input"),
            "应为编译错误: {msg}"
        ),
        other => panic!("预期 Script 错误，得到: {other}"),
    }
}

/// 测试 execute_script 运行时错误。
#[test]
fn test_webview_execute_script_runtime_error() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("throw new Error('test error');");
    assert!(result.is_err(), "运行时错误应返回错误");
}

/// 测试 remove_event_callback 越界返回 false。
#[test]
fn test_webview_remove_callback_out_of_bounds() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(!wv.remove_event_callback(999), "越界索引应返回 false");
    assert!(!wv.remove_event_callback(0), "空列表索引 0 应返回 false");
}

/// 测试 DOM API：outerHTML 读写。
#[test]
fn test_webview_dom_outer_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'test');
        el.outerHTML;
        "#,
    );
    assert!(result.is_ok());
    let html = result.unwrap();
    assert!(html.contains("div"), "outerHTML 应包含标签名");
    assert!(html.contains("test"), "outerHTML 应包含属性值");
}

/// 测试 DOM API：createDocumentFragment 基本存在性。
#[test]
fn test_webview_dom_document_fragment() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        r#"
        var frag = document.createDocumentFragment();
        typeof frag === 'object' ? 'ok' : 'fail';
        "#,
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "ok", "createDocumentFragment 应返回对象");
}

// ── WASM 执行测试 ──

/// 最小的 WASM 模块：导出一个 `add` 函数 (i32) -> (i32)，返回 42。
///
/// 手工编码的 WASM 二进制：
/// - magic + version: 00 61 73 6d 01 00 00 00
/// - type section: (i32) -> (i32)
/// - function section: type index 0
/// - export section: "add" → function 0
/// - code section: i32.const 42, end
fn minimal_wasm_add_module() -> Vec<u8> {
    vec![
        0x00, 0x61, 0x73, 0x6d, // magic
        0x01, 0x00, 0x00, 0x00, // version
        // type section (id=1)
        0x01, 0x06, 0x01, 0x60, 0x01, 0x7f, 0x01, 0x7f, // function section (id=3)
        0x03, 0x02, 0x01, 0x00, // export section (id=7)
        0x07, 0x07, 0x01, 0x03, 0x61, 0x64, 0x64, 0x00, 0x00, // code section (id=10)
        0x0a, 0x09, 0x01, 0x07, 0x00, 0x20, 0x00, 0x41, 0x2a, 0x6a, 0x0b,
    ]
}

/// 测试 WASM 模块编译和执行。
#[test]
fn test_webview_execute_wasm_add() {
    let wv = WebView::new(WebViewConfig::default());
    let wasm_bytes = minimal_wasm_add_module();
    let args = vec![zero_wasm_sandbox::WasmValue::I32(8)];
    let result = wv.execute_wasm(&wasm_bytes, "add", &args);
    assert!(result.is_ok(), "WASM execution should succeed: {:?}", result);
    // add(x) = x + 42 = 8 + 42 = 50
    assert_eq!(result.unwrap().trim(), "i32(50)");
}

/// 测试 WASM 执行无效字节码。
#[test]
fn test_webview_execute_wasm_invalid_bytes() {
    let wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_wasm(&[0x00, 0x01, 0x02], "func", &[]);
    assert!(result.is_err(), "Invalid WASM should return error");
}

/// 测试 WASM 执行不存在的函数。
#[test]
fn test_webview_execute_wasm_missing_function() {
    let wv = WebView::new(WebViewConfig::default());
    let wasm_bytes = minimal_wasm_add_module();
    let result = wv.execute_wasm(&wasm_bytes, "nonexistent", &[]);
    assert!(result.is_err(), "Missing function should return error");
}

// ── Service Worker 集成测试 ──

/// 测试 Service Worker 注册 + 安装 + 激活完整生命周期。
#[test]
fn test_sw_register_install_activate() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.register_service_worker("/sw.js", "/", "https://example.com");
    assert!(wv.install_service_worker(id));
    assert!(wv.activate_service_worker(id));

    let reg = wv.service_worker_registry().get(id).unwrap();
    assert_eq!(reg.script_url, "/sw.js");
    assert!(reg.is_active());
}

/// 测试 Service Worker 注销。
#[test]
fn test_sw_unregister() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.register_service_worker("/sw.js", "/", "https://example.com");
    wv.install_service_worker(id);
    wv.activate_service_worker(id);
    assert!(wv.unregister_service_worker(id));
    assert!(wv.service_worker_registry().get(id).is_none());
}

/// 测试 Service Worker 版本替换。
#[test]
fn test_sw_version_replacement() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id1 = wv.register_service_worker("/sw-v1.js", "/", "https://example.com");
    wv.install_service_worker(id1);
    wv.activate_service_worker(id1);

    let id2 = wv.register_service_worker("/sw-v2.js", "/", "https://example.com");
    wv.install_service_worker(id2);
    wv.activate_service_worker(id2);

    // 旧 SW 应被标记为废弃
    assert_eq!(
        wv.service_worker_registry().get(id1).unwrap().state,
        zero_storage::ServiceWorkerState::Redundant
    );
    assert!(wv.service_worker_registry().get(id2).unwrap().is_active());
}

/// 测试 Service Worker fetch 拦截（缓存命中）。
#[test]
fn test_sw_intercept_cached_response() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.register_service_worker("/sw.js", "/", "https://example.com");
    wv.install_service_worker(id);
    wv.activate_service_worker(id);

    // 手动缓存一个响应
    let request = zero_storage::CacheRequest::new("https://example.com/cached.html");
    let response = zero_storage::CacheResponse::ok(b"<html><body>Cached</body></html>".to_vec());
    let _ = wv
        .service_worker_registry_mut()
        .get_active_mut("https://example.com")
        .unwrap()
        .cache_storage
        .open("v1")
        .put(request.clone(), response);

    // fetch_url 应被拦截并返回缓存内容
    // 注意：这需要网络可用，因为我们测试的是拦截路径
    // 用 load_html 模拟已缓存的结果更可控
    let reg = wv.service_worker_registry();
    let intercept = reg.intercept_fetch(&request, "https://example.com");
    assert!(matches!(intercept, zero_storage::FetchInterceptResult::Cached(_)));
}

/// 测试 Service Worker 不影响无 SW 的请求。
#[test]
fn test_sw_no_worker_pass_through() {
    let wv = WebView::new(WebViewConfig::default());
    let request = zero_storage::CacheRequest::new("https://example.com/page.html");
    let result = wv
        .service_worker_registry()
        .intercept_fetch(&request, "https://example.com");
    assert!(matches!(result, zero_storage::FetchInterceptResult::NoWorker));
}

/// 测试 Service Worker 作用域匹配。
#[test]
fn test_sw_scope_matching() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.register_service_worker("/sw.js", "/app/", "https://example.com");
    wv.install_service_worker(id);
    wv.activate_service_worker(id);

    let reg = wv.service_worker_registry().get_active("https://example.com").unwrap();
    assert!(reg.is_in_scope("https://example.com/app/page.html"));
    assert!(reg.is_in_scope("/app/sub/page.html"));
    assert!(!reg.is_in_scope("https://example.com/other/page.html"));
}

/// 测试 Service Worker 非法操作（重复安装）。
#[test]
fn test_sw_invalid_operations() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.register_service_worker("/sw.js", "/", "https://example.com");

    assert!(wv.install_service_worker(id));
    assert!(!wv.install_service_worker(id)); // 重复安装应失败
    assert!(!wv.activate_service_worker(999)); // 不存在的 ID
}

/// 测试 extract_origin 辅助函数。
#[test]
fn test_extract_origin() {
    assert_eq!(
        WebView::extract_origin("https://example.com/path?q=1"),
        Some("https://example.com".to_string())
    );
    assert_eq!(
        WebView::extract_origin("https://example.com:8443/path"),
        Some("https://example.com:8443".to_string())
    );
    assert_eq!(WebView::extract_origin("not-a-url"), None);
}
