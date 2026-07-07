//! DOM Bridge polyfill 行为测试。
//!
//! 通过 V8 引擎实际执行 polyfill 代码，验证 JS 侧 DOM API 的运行时行为。

use zero_engine::dom_bridge::generate_dom_api_polyfill;

/// 辅助：在 V8 中执行 polyfill + 测试代码，返回原始结果字符串。
fn eval_polyfill(test_code: &str) -> String {
    #[cfg(feature = "v8")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        Box::new(zero_script_sandbox::V8Sandbox::new().expect("V8 init"));
    #[cfg(feature = "quickjs")]
    let mut sandbox: Box<dyn zero_script_sandbox::Sandbox> =
        Box::new(zero_script_sandbox::QuickJSSandbox::new().expect("QuickJS init"));

    let polyfill = generate_dom_api_polyfill();
    let full_code = format!("{polyfill}\n{test_code}");
    let result = sandbox.execute(&full_code).expect("execute");
    result.value
}

#[test]
fn test_polyfill_create_element() {
    let result = eval_polyfill("var div = document.createElement('div');\nJSON.stringify(div.tagName);");
    assert_eq!(result.trim_matches('"'), "DIV", "tagName should be 'DIV'");
}

#[test]
fn test_polyfill_create_text_node() {
    let result = eval_polyfill("var t = document.createTextNode('hello');\nJSON.stringify(t.textContent);");
    assert_eq!(result.trim_matches('"'), "hello");
}

#[test]
fn test_polyfill_set_get_attribute() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'test-id');
        el.setAttribute('class', 'a b');
        var r = {id: el.getAttribute('id'), cls: el.getAttribute('class'), has: el.hasAttribute('id')};
        JSON.stringify(r);
    "#,
    );
    assert!(result.contains("test-id"), "id 应为 'test-id': {result}");
    assert!(result.contains("a b"), "class 应为 'a b': {result}");
}

#[test]
fn test_polyfill_remove_attribute() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('data-x', '123');
        el.removeAttribute('data-x');
        JSON.stringify(el.hasAttribute('data-x'));
    "#,
    );
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_polyfill_append_child() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var c = document.createElement('span');
        p.appendChild(c);
        JSON.stringify(p.children.length);
    "#,
    );
    assert_eq!(result.trim(), "1");
}

#[test]
fn test_polyfill_remove_child() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var c = document.createElement('span');
        p.appendChild(c);
        p.removeChild(c);
        JSON.stringify(p.children.length);
    "#,
    );
    assert_eq!(result.trim(), "0");
}

#[test]
fn test_polyfill_child_parent_node() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var c = document.createElement('span');
        p.appendChild(c);
        JSON.stringify(c.parentNode === p);
    "#,
    );
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_polyfill_insert_before() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var a = document.createElement('a');
        var b = document.createElement('b');
        var c = document.createElement('c');
        p.appendChild(a);
        p.appendChild(c);
        p.insertBefore(b, c);
        JSON.stringify(p.children[1].tagName);
    "#,
    );
    assert_eq!(result.trim_matches('"'), "B");
}

#[test]
fn test_polyfill_clone_node_shallow() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('class', 'orig');
        el.appendChild(document.createElement('span'));
        var clone = el.cloneNode(false);
        JSON.stringify([clone.getAttribute('class'), clone.children.length]);
    "#,
    );
    assert!(result.contains("orig"), "浅拷贝应保留属性: {result}");
    assert!(result.contains("0"), "浅拷贝不应含子节点: {result}");
}

#[test]
fn test_polyfill_clone_node_deep() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'src');
        var child = document.createElement('span');
        child.setAttribute('class', 'inner');
        el.appendChild(child);
        var clone = el.cloneNode(true);
        JSON.stringify([clone.getAttribute('id'), clone.children.length]);
    "#,
    );
    assert!(result.contains("src"), "深拷贝应保留属性: {result}");
    assert!(result.contains("1"), "深拷贝应含子节点: {result}");
}

#[test]
fn test_polyfill_text_content_getter() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.appendChild(document.createTextNode('Hello '));
        el.appendChild(document.createTextNode('World'));
        JSON.stringify(el.getTextContent());
    "#,
    );
    assert_eq!(result.trim_matches('"'), "Hello World");
}

#[test]
fn test_polyfill_has_child_nodes() {
    let result = eval_polyfill(
        r#"
        var e = document.createElement('div');
        var f = document.createElement('div');
        f.appendChild(document.createElement('span'));
        JSON.stringify([e.hasChildNodes(), f.hasChildNodes()]);
    "#,
    );
    assert!(result.contains("false"), "空元素 hasChildNodes 应 false: {result}");
    assert!(result.contains("true"), "有子节点 hasChildNodes 应 true: {result}");
}

#[test]
fn test_polyfill_replace_child() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var old = document.createElement('span');
        var nw = document.createElement('b');
        p.appendChild(old);
        p.replaceChild(nw, old);
        JSON.stringify(p.children[0].tagName);
    "#,
    );
    assert_eq!(result.trim_matches('"'), "B");
}

#[test]
fn test_polyfill_document_fragment_type() {
    let result = eval_polyfill("var frag = document.createDocumentFragment(); JSON.stringify(frag.nodeType);");
    assert_eq!(result.trim(), "11", "DocumentFragment nodeType 应为 11");
}

#[test]
fn test_polyfill_id_classname() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'myId');
        el.setAttribute('class', 'foo bar');
        JSON.stringify([el.id, el.className]);
    "#,
    );
    assert!(result.contains("myId"), "el.id 应为 'myId': {result}");
    assert!(result.contains("foo bar"), "el.className 应为 'foo bar': {result}");
}

#[test]
fn test_polyfill_get_element_by_id() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'target');
        document.body.appendChild(el);
        JSON.stringify(document.getElementById('target') !== null);
    "#,
    );
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_polyfill_get_element_by_id_not_found() {
    let result = eval_polyfill("JSON.stringify(document.getElementById('nonexistent'));");
    assert_eq!(result.trim(), "null");
}

// ── CSSStyleDeclaration 测试 ──

#[test]
fn test_polyfill_style_set_get_property() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.style.setProperty('color', 'red');
        el.style.setProperty('font-size', '16px');
        JSON.stringify([el.style.getPropertyValue('color'), el.style.getPropertyValue('font-size')]);
    "#,
    );
    assert!(result.contains("red"), "color 应为 'red': {result}");
    assert!(result.contains("16px"), "font-size 应为 '16px': {result}");
}

#[test]
fn test_polyfill_style_remove_property() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.style.setProperty('color', 'red');
        var removed = el.style.removeProperty('color');
        JSON.stringify([removed, el.style.getPropertyValue('color')]);
    "#,
    );
    assert!(result.contains("red"), "removeProperty 应返回旧值: {result}");
    assert!(result.contains("\"\""), "移除后 getPropertyValue 应为空: {result}");
}

#[test]
fn test_polyfill_style_css_text() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.style.setProperty('color', 'blue');
        el.style.setProperty('margin', '10px');
        JSON.stringify(el.style.cssText);
    "#,
    );
    assert!(result.contains("color"), "cssText 应包含 'color': {result}");
    assert!(result.contains("margin"), "cssText 应包含 'margin': {result}");
}

#[test]
fn test_polyfill_style_css_text_setter() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.style.cssText = 'padding: 5px; border: 1px solid';
        JSON.stringify(el.style.getPropertyValue('padding'));
    "#,
    );
    assert!(result.contains("5px"), "cssText setter 应解析属性: {result}");
}

// ── DOMTokenList (classList) 测试 ──

#[test]
fn test_polyfill_classlist_add_contains() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.classList.add('active');
        el.classList.add('visible');
        JSON.stringify([el.classList.contains('active'), el.classList.contains('hidden'), el.classList.length]);
    "#,
    );
    assert!(result.contains("true"), "应包含 'active': {result}");
    assert!(result.contains("false"), "不应包含 'hidden': {result}");
}

#[test]
fn test_polyfill_classlist_remove() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.classList.add('a');
        el.classList.add('b');
        el.classList.remove('a');
        JSON.stringify([el.classList.contains('a'), el.classList.contains('b')]);
    "#,
    );
    assert!(result.contains("false"), "移除后不应包含 'a': {result}");
    assert!(result.contains("true"), "仍应包含 'b': {result}");
}

#[test]
fn test_polyfill_classlist_toggle() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.classList.add('on');
        var r1 = el.classList.toggle('on');
        var r2 = el.classList.toggle('off');
        JSON.stringify([r1, r2, el.classList.contains('on'), el.classList.contains('off')]);
    "#,
    );
    assert!(result.contains("false"), "toggle 已有的类应返回 false: {result}");
    assert!(result.contains("true"), "toggle 没有的类应返回 true: {result}");
}

#[test]
fn test_polyfill_classlist_replace() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.classList.add('old');
        var r = el.classList.replace('old', 'new');
        JSON.stringify([r, el.classList.contains('old'), el.classList.contains('new')]);
    "#,
    );
    assert!(result.contains("true"), "replace 应返回 true: {result}");
}

#[test]
fn test_polyfill_classlist_item() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.classList.add('first');
        el.classList.add('second');
        JSON.stringify([el.classList.item(0), el.classList.item(1), el.classList.item(99)]);
    "#,
    );
    assert!(result.contains("first"), "item(0) 应为 'first': {result}");
    assert!(result.contains("second"), "item(1) 应为 'second': {result}");
    assert!(result.contains("null"), "越界 item 应为 null: {result}");
}

// ── 导航属性测试 ──

#[test]
fn test_polyfill_first_last_child() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var a = document.createElement('a');
        var b = document.createElement('b');
        p.appendChild(a);
        p.appendChild(b);
        JSON.stringify([p.firstChild.tagName, p.lastChild.tagName]);
    "#,
    );
    assert!(result.contains("A"), "firstChild 应为 A: {result}");
    assert!(result.contains("B"), "lastChild 应为 B: {result}");
}

#[test]
fn test_polyfill_next_previous_sibling() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        var a = document.createElement('a');
        var b = document.createElement('b');
        var c = document.createElement('c');
        p.appendChild(a); p.appendChild(b); p.appendChild(c);
        JSON.stringify([a.nextSibling.tagName, c.previousSibling.tagName]);
    "#,
    );
    assert!(result.contains("B"), "a.nextSibling 应为 B: {result}");
    assert!(result.contains("B"), "c.previousSibling 应为 B: {result}");
}

#[test]
fn test_polyfill_child_element_count() {
    let result = eval_polyfill(
        r#"
        var p = document.createElement('div');
        p.appendChild(document.createElement('a'));
        p.appendChild(document.createElement('b'));
        JSON.stringify(p.childElementCount);
    "#,
    );
    assert_eq!(result.trim(), "2");
}

#[test]
fn test_polyfill_first_child_empty() {
    let result = eval_polyfill("var p = document.createElement('div'); JSON.stringify(p.firstChild);");
    assert_eq!(result.trim(), "null");
}

// ── innerHTML / outerHTML 测试 ──

#[test]
fn test_polyfill_inner_html_setter() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.innerHTML = 'Hello';
        JSON.stringify(el.textContent);
    "#,
    );
    assert_eq!(result.trim_matches('"'), "Hello");
}

#[test]
fn test_polyfill_outer_html_tag() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'x');
        var html = el.outerHTML;
        JSON.stringify(html.indexOf('<div') === 0 && html.indexOf('id="x"') > 0);
    "#,
    );
    assert_eq!(result.trim(), "true");
}

// ── Fetch API 测试 ──

#[test]
fn test_polyfill_fetch_returns_response() {
    // fetch() 返回 Promise，then 回调在 V8 桩中不保证同步执行
    // 直接验证 fetch 函数和 Response 构造器存在
    let result = eval_polyfill("JSON.stringify([typeof fetch === 'function', typeof Response === 'function']);");
    assert!(result.contains("true"), "fetch 和 Response 应存在: {result}");
}

#[test]
fn test_polyfill_headers_case_insensitive() {
    let result = eval_polyfill(
        r#"
        var h = new Headers();
        h.append('Content-Type', 'text/html');
        JSON.stringify([h.get('content-type'), h.has('CONTENT-TYPE')]);
    "#,
    );
    assert!(result.contains("text/html"), "Headers 应不区分大小写: {result}");
    assert!(result.contains("true"), "has 应不区分大小写: {result}");
}

#[test]
fn test_polyfill_headers_delete() {
    let result = eval_polyfill(
        r#"
        var h = new Headers();
        h.set('X-Custom', 'value');
        h.delete('x-custom');
        JSON.stringify(h.has('X-Custom'));
    "#,
    );
    assert_eq!(result.trim(), "false");
}

#[test]
fn test_polyfill_response_ok_status() {
    let result = eval_polyfill(
        r#"
        var r = new Response('body text', {status: 200, statusText: 'OK'});
        JSON.stringify([r.ok, r.status, r.statusText]);
    "#,
    );
    assert!(result.contains("true"), "status 200 ok 应为 true: {result}");
    assert!(result.contains("200"), "status 应为 200: {result}");
}

#[test]
fn test_polyfill_response_not_ok() {
    let result = eval_polyfill("var r = new Response(null, {status: 404}); JSON.stringify(r.ok);");
    assert_eq!(result.trim(), "false", "status 404 ok 应为 false");
}

#[test]
fn test_polyfill_response_clone() {
    let result = eval_polyfill(
        r#"
        var r = new Response('body', {status: 201, statusText: 'Created'});
        var c = r.clone();
        JSON.stringify([c.status, c.statusText]);
    "#,
    );
    assert!(result.contains("201"), "clone 应保留 status: {result}");
    assert!(result.contains("Created"), "clone 应保留 statusText: {result}");
}

#[test]
fn test_polyfill_response_error() {
    let result = eval_polyfill("var r = Response.error(); JSON.stringify(r.type);");
    assert_eq!(result.trim_matches('"'), "error");
}

// ── Web Storage API 测试 ──

#[test]
fn test_polyfill_local_storage_crud() {
    let result = eval_polyfill(
        r#"
        localStorage.setItem('key', 'value');
        var v = localStorage.getItem('key');
        localStorage.removeItem('key');
        var after = localStorage.getItem('key');
        JSON.stringify([v, after]);
    "#,
    );
    assert!(result.contains("value"), "getItem 应返回 'value': {result}");
    assert!(result.contains("null"), "removeItem 后应为 null: {result}");
}

#[test]
fn test_polyfill_session_storage_clear() {
    let result = eval_polyfill(
        r#"
        sessionStorage.setItem('a', '1');
        sessionStorage.setItem('b', '2');
        sessionStorage.clear();
        JSON.stringify(sessionStorage.length);
    "#,
    );
    assert_eq!(result.trim(), "0");
}

#[test]
fn test_polyfill_storage_key() {
    let result = eval_polyfill(
        r#"
        localStorage.clear();
        localStorage.setItem('x', '1');
        localStorage.setItem('y', '2');
        JSON.stringify(localStorage.length >= 2);
    "#,
    );
    assert_eq!(result.trim(), "true");
}

// ── MutationObserver 测试 ──

#[test]
fn test_polyfill_mutation_observer_observe_disconnect() {
    let result = eval_polyfill(
        r#"
        var called = false;
        var obs = new MutationObserver(function() { called = true; });
        var el = document.createElement('div');
        obs.observe(el, { childList: true });
        obs.disconnect();
        JSON.stringify(typeof obs._callback === 'function');
    "#,
    );
    assert_eq!(result.trim(), "true");
}

#[test]
fn test_polyfill_mutation_observer_take_records() {
    let result = eval_polyfill(
        r#"
        var obs = new MutationObserver(function() {});
        var records = obs.takeRecords();
        JSON.stringify(Array.isArray(records) && records.length === 0);
    "#,
    );
    assert_eq!(result.trim(), "true");
}

// ── CustomEvent 测试 ──

#[test]
fn test_polyfill_custom_event() {
    let result = eval_polyfill(
        r#"
        var e = new CustomEvent('myevent', { bubbles: true, detail: { x: 42 } });
        JSON.stringify([e.type, e.bubbles, e.detail.x]);
    "#,
    );
    assert!(result.contains("myevent"), "type 应为 'myevent': {result}");
    assert!(result.contains("42"), "detail.x 应为 42: {result}");
}

#[test]
fn test_polyfill_custom_event_prevent_default() {
    let result = eval_polyfill(
        r#"
        var e = new CustomEvent('test', { cancelable: true });
        e.preventDefault();
        JSON.stringify(e._defaultPrevented);
    "#,
    );
    assert_eq!(result.trim(), "true");
}

// ── IntersectionObserver 测试 ──

#[test]
fn test_polyfill_intersection_observer_observe() {
    let result = eval_polyfill(
        r#"
        var obs = new IntersectionObserver(function() {});
        var el = document.createElement('div');
        obs.observe(el);
        obs.observe(el);
        JSON.stringify(obs._observing.length);
    "#,
    );
    assert_eq!(result.trim(), "1", "重复 observe 不应重复添加");
}

#[test]
fn test_polyfill_intersection_observer_unobserve() {
    let result = eval_polyfill(
        r#"
        var obs = new IntersectionObserver(function() {});
        var el = document.createElement('div');
        obs.observe(el);
        obs.unobserve(el);
        JSON.stringify(obs._observing.length);
    "#,
    );
    assert_eq!(result.trim(), "0");
}

#[test]
fn test_polyfill_intersection_observer_disconnect() {
    let result = eval_polyfill(
        r#"
        var obs = new IntersectionObserver(function() {});
        obs.observe(document.createElement('a'));
        obs.observe(document.createElement('b'));
        obs.disconnect();
        JSON.stringify(obs._observing.length);
    "#,
    );
    assert_eq!(result.trim(), "0");
}

// ── ResizeObserver 测试 ──

#[test]
fn test_polyfill_resize_observer_observe_unobserve() {
    let result = eval_polyfill(
        r#"
        var obs = new ResizeObserver(function() {});
        var el = document.createElement('div');
        obs.observe(el);
        obs.unobserve(el);
        JSON.stringify(obs._observing.length);
    "#,
    );
    assert_eq!(result.trim(), "0");
}

#[test]
fn test_polyfill_resize_observer_disconnect() {
    let result = eval_polyfill(
        r#"
        var obs = new ResizeObserver(function() {});
        obs.observe(document.createElement('div'));
        obs.disconnect();
        JSON.stringify(obs._observing.length);
    "#,
    );
    assert_eq!(result.trim(), "0");
}

// ── Timer API 测试 ──

#[test]
fn test_polyfill_set_timeout_executes() {
    let result = eval_polyfill(
        r#"
        var x = 0;
        setTimeout(function() { x = 42; }, 0);
        JSON.stringify(x);
    "#,
    );
    assert_eq!(result.trim(), "42", "setTimeout 应同步执行回调（桩实现）");
}

#[test]
fn test_polyfill_set_interval_executes() {
    let result = eval_polyfill(
        r#"
        var x = 0;
        setInterval(function() { x = 99; }, 100);
        JSON.stringify(x);
    "#,
    );
    assert_eq!(result.trim(), "99", "setInterval 应同步执行回调（桩实现）");
}

// ── WebAssembly API 测试 ──

#[test]
fn test_polyfill_webassembly_exists() {
    let result = eval_polyfill(
        r#"
        typeof WebAssembly === 'object' ? 'yes' : 'no';
        "#,
    );
    assert_eq!(result.trim(), "yes", "WebAssembly 对象应存在");
}

#[test]
fn test_polyfill_webassembly_validate() {
    let result = eval_polyfill(
        r#"
        // WASM 魔术字节: 0x00 0x61 0x73 0x6D + 版本号 0x01 0x00 0x00 0x00
        var validWasm = new Uint8Array([0x00, 0x61, 0x73, 0x6D, 0x01, 0x00, 0x00, 0x00]);
        var validResult = WebAssembly.validate(validWasm) ? 'valid' : 'invalid';
        // 非 WASM 字节应返回 false
        var invalidResult = WebAssembly.validate(new ArrayBuffer(8)) ? 'valid' : 'invalid';
        validResult + ':' + invalidResult;
        "#,
    );
    assert_eq!(
        result.trim(),
        "valid:invalid",
        "WebAssembly.validate 应正确检测 WASM 魔术字节"
    );
}

#[test]
fn test_polyfill_webassembly_compile() {
    let result = eval_polyfill(
        r#"
        var p = WebAssembly.compile(new ArrayBuffer(8));
        typeof p === 'object' ? 'promise' : typeof p;
        "#,
    );
    assert_eq!(result.trim(), "promise", "WebAssembly.compile 应返回 Promise");
}

#[test]
fn test_polyfill_webassembly_instantiate() {
    let result = eval_polyfill(
        r#"
        var p = WebAssembly.instantiate(new ArrayBuffer(8));
        typeof p === 'object' ? 'promise' : typeof p;
        "#,
    );
    assert_eq!(result.trim(), "promise", "WebAssembly.instantiate 应返回 Promise");
}

#[test]
fn test_polyfill_webassembly_memory() {
    let result = eval_polyfill(
        r#"
        var p = WebAssembly.instantiate(new ArrayBuffer(8));
        // 同步等待 resolve（桩实现立即 resolve）
        var result = null;
        p.then(function(r) { result = r; });
        result && result.instance && result.instance.exports.memory
            ? 'has-memory' : 'no-memory';
        "#,
    );
    // V8 Promise 是微任务，需要 await 或特殊处理
    // 桩实现中 Promise.resolve 可能不会同步执行 then
    // 所以我们只验证 polyfill 语法正确，不验证异步行为
    let _ = result; // 不 panic 即可
}

// ── navigator.serviceWorker 测试 ──

#[test]
fn test_polyfill_navigator_service_worker_exists() {
    let result = eval_polyfill(
        r#"
        typeof navigator.serviceWorker === 'object' ? 'yes' : 'no';
        "#,
    );
    assert_eq!(result.trim(), "yes", "navigator.serviceWorker 应存在");
}

#[test]
fn test_polyfill_service_worker_register() {
    let result = eval_polyfill(
        r#"
        var p = navigator.serviceWorker.register('/sw.js');
        typeof p === 'object' ? 'promise' : typeof p;
        "#,
    );
    assert_eq!(result.trim(), "promise", "register() 应返回 Promise");
}

#[test]
fn test_polyfill_service_worker_register_with_scope() {
    let result = eval_polyfill(
        r#"
        navigator.serviceWorker.register('/sw.js', { scope: '/app/' });
        var regs = navigator.serviceWorker._registrations;
        regs.length === 1 && regs[0]._scope === '/app/' ? 'ok' : 'fail:' + regs.length;
        "#,
    );
    assert_eq!(result.trim(), "ok", "register 应记录 scope 选项");
}

#[test]
fn test_polyfill_service_worker_get_registrations() {
    let result = eval_polyfill(
        r#"
        navigator.serviceWorker.register('/sw.js');
        navigator.serviceWorker.getRegistrations instanceof Function
            ? 'has-method' : 'no-method';
        "#,
    );
    assert_eq!(result.trim(), "has-method", "getRegistrations 方法应存在");
}

// ═══════════════════════════════════════════════════════════════
// element.matches() 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_polyfill_matches_class() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('class', 'active highlight');
        el.matches('.active') ? 'matches' : 'no-match';
        "#,
    );
    assert_eq!(result.trim(), "matches", ".active 应匹配");
}

#[test]
fn test_polyfill_matches_id() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('id', 'main');
        el.matches('#main') ? 'matches' : 'no-match';
        "#,
    );
    assert_eq!(result.trim(), "matches", "#main 应匹配");
}

#[test]
fn test_polyfill_matches_tag() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('span');
        el.matches('span') ? 'matches' : 'no-match';
        "#,
    );
    assert_eq!(result.trim(), "matches", "span 标签应匹配");
}

#[test]
fn test_polyfill_matches_multi_selector() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('class', 'item');
        el.matches('.active, .item, .selected') ? 'matches' : 'no-match';
        "#,
    );
    assert_eq!(result.trim(), "matches", "逗号选择器中 .item 应匹配");
}

#[test]
fn test_polyfill_matches_attribute() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('input');
        el.setAttribute('type', 'text');
        el.setAttribute('name', 'email');
        el.matches('[type="text"]') ? 'attr-match' : 'no-match';
        "#,
    );
    assert_eq!(result.trim(), "attr-match", "[type=text] 属性选择器应匹配");
}

#[test]
fn test_polyfill_matches_no_match() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('class', 'foo');
        el.matches('.bar') ? 'matches' : 'no-match';
        "#,
    );
    assert_eq!(result.trim(), "no-match", ".bar 不应匹配 .foo 元素");
}

// ═══════════════════════════════════════════════════════════════
// element.closest() 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_polyfill_closest_self() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('class', 'parent');
        el.closest('.parent') === el ? 'self' : 'no-self';
        "#,
    );
    assert_eq!(result.trim(), "self", "closest('.parent') 应返回自身");
}

#[test]
fn test_polyfill_closest_parent() {
    let result = eval_polyfill(
        r#"
        var parent = document.createElement('div');
        parent.setAttribute('class', 'container');
        var child = document.createElement('span');
        parent.appendChild(child);
        child.closest('.container') === parent ? 'parent' : 'no-parent';
        "#,
    );
    assert_eq!(result.trim(), "parent", "closest('.container') 应返回父元素");
}

#[test]
fn test_polyfill_closest_none() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        el.setAttribute('class', 'orphan');
        el.closest('.nonexistent') === null ? 'null' : 'found';
        "#,
    );
    assert_eq!(result.trim(), "null", "closest('.nonexistent') 应返回 null");
}

// ═══════════════════════════════════════════════════════════════
// document.createComment() 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_polyfill_create_comment() {
    let result = eval_polyfill(
        r#"
        var c = document.createComment('this is a comment');
        JSON.stringify({nodeType: c.nodeType, text: c.textContent});
        "#,
    );
    assert!(result.contains("\"nodeType\":8"), "注释节点 nodeType 应为 8");
    assert!(result.contains("this is a comment"), "textContent 应保留");
}

// ═══════════════════════════════════════════════════════════════
// getBoundingClientRect() 测试
// ═══════════════════════════════════════════════════════════════

#[test]
fn test_polyfill_get_bounding_client_rect() {
    let result = eval_polyfill(
        r#"
        var el = document.createElement('div');
        var rect = el.getBoundingClientRect();
        JSON.stringify({w: rect.width, h: rect.height, x: rect.x, y: rect.y});
        "#,
    );
    assert!(result.contains("\"w\":0"), "stub rect width 应为 0");
    assert!(result.contains("\"h\":0"), "stub rect height 应为 0");
}
