// net-api 二期 M2-S10 记账 1/2 清偿钉：请求头名首见原始大小写线上保真补全——
// Headers 实例作 fetch init 头源（fill 经 entries sort-and-combine 只产小写名）
// 与 XHR 异步路（fetchFn dict init）两形态。同步 XHR caseMap / 数组 / record 三
// 形态既有覆盖（request-headers-case 语料），本钉锁旁路补齐面。
// https://fetch.spec.whatwg.org/#dom-headers（fill 对 Headers 对象迭代 header
// list 本就保 pair 名；浏览器同型——API 面归一小写、网络面保用户 case）。

#[test]
fn test_fetch_headers_instance_preserves_first_seen_wire_case() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    // 相对 URL 形态需可解析 base（同 part23 credentials 测试——shim fetch 构造校验
    // 链经 `new URL` 验解析后 URL）。
    sandbox.register_callback(
        "__zw_get_page_url",
        Box::new(|_args: &[String]| "https://wpt.test/case-page".to_string()),
    );
    sandbox.register_callback(
        "__zw_parse_url",
        Box::new(|args: &[String]| -> String {
            let input = args.first().map(String::as_str).unwrap_or("");
            let base = args.get(1).map(String::as_str);
            crate::js_dom_bridge::parse_url_to_json(input, base)
        }),
    );
    sandbox.execute(crate::js_dom_bridge::generate_js_dom_shim()).unwrap();
    let wires: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let wires_for_callback = wires.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            // __zw_fetch 同步契约参数位：id|method|url|headersWire|body|...（part01
            // 主发调用序）；捕获 headersWire 供大小写断言。
            wires_for_callback
                .lock()
                .unwrap()
                .push(args.get(3).cloned().unwrap_or_default());
            "__zwfr:200\x1fOK\x1f\x1fbody".to_string()
        }),
    );

    sandbox
        .execute(
            "globalThis.__caseDone = 'pending';\
             fetch('/case-a', { headers: new Headers([['THIS-is-A-test', '1']]) })\
               .then(function () { globalThis.__caseDone = 'done'; });",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(sandbox.execute("String(globalThis.__caseDone)").unwrap().value, "done");
    let wire = wires.lock().unwrap().last().cloned().unwrap_or_default();
    assert!(
        wire.contains("THIS-is-A-test\x1e1"),
        "Headers 实例源线上应保首见原始大小写，got: {wire}"
    );
    // API 面恒小写（spec HeaderList 归一语义零扰动）。
    assert_eq!(
        sandbox
            .execute("var __hh = new Headers([['THIS-is-A-test', '1']]); __hh.get('this-is-a-test')")
            .unwrap()
            .value,
        "1"
    );
}

#[test]
fn test_request_ctor_headers_instance_round_trips_wire_case() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.register_callback(
        "__zw_get_page_url",
        Box::new(|_args: &[String]| "https://wpt.test/case-page".to_string()),
    );
    sandbox.register_callback(
        "__zw_parse_url",
        Box::new(|args: &[String]| -> String {
            let input = args.first().map(String::as_str).unwrap_or("");
            let base = args.get(1).map(String::as_str);
            crate::js_dom_bridge::parse_url_to_json(input, base)
        }),
    );
    sandbox.execute(crate::js_dom_bridge::generate_js_dom_shim()).unwrap();
    let wires: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let wires_for_callback = wires.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            wires_for_callback
                .lock()
                .unwrap()
                .push(args.get(3).cloned().unwrap_or_default());
            "__zwfr:200\x1fOK\x1f\x1fbody".to_string()
        }),
    );

    sandbox
        .execute(
            "globalThis.__reqCaseDone = 'pending';\
             var __req = new Request('/case-c', {\
               headers: new Headers([['THIS-is-A-test', '1']])\
             });\
             fetch(__req).then(function () { globalThis.__reqCaseDone = 'done'; });",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(sandbox.execute("String(globalThis.__reqCaseDone)").unwrap().value, "done");
    let wire = wires.lock().unwrap().last().cloned().unwrap_or_default();
    assert!(
        wire.contains("THIS-is-A-test\x1e1"),
        "Request ctor 经 Headers 实例源分发线上应保首见原始大小写，got: {wire}"
    );
}

#[test]
fn test_async_xhr_set_request_header_preserves_wire_case() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.register_callback(
        "__zw_get_page_url",
        Box::new(|_args: &[String]| "https://wpt.test/case-page".to_string()),
    );
    sandbox.register_callback(
        "__zw_parse_url",
        Box::new(|args: &[String]| -> String {
            let input = args.first().map(String::as_str).unwrap_or("");
            let base = args.get(1).map(String::as_str);
            crate::js_dom_bridge::parse_url_to_json(input, base)
        }),
    );
    sandbox.execute(crate::js_dom_bridge::generate_js_dom_shim()).unwrap();
    let wires: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let wires_for_callback = wires.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            wires_for_callback
                .lock()
                .unwrap()
                .push(args.get(3).cloned().unwrap_or_default());
            "__zwfr:200\x1fOK\x1f\x1fbody".to_string()
        }),
    );

    sandbox
        .execute(
            "globalThis.__xhrCaseDone = 'pending';\
             var __x = new XMLHttpRequest();\
             __x.open('GET', '/case-b');\
             __x.setRequestHeader('X-MiXeD-Case', 'v1');\
             __x.setRequestHeader('X-MiXeD-case', 'v2');\
             __x.addEventListener('load', function () { globalThis.__xhrCaseDone = 'done'; });\
             __x.send();",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }

    assert_eq!(sandbox.execute("String(globalThis.__xhrCaseDone)").unwrap().value, "done");
    let wire = wires.lock().unwrap().last().cloned().unwrap_or_default();
    // 首见原始名 `X-MiXeD-Case` 上线，同名值合并（spec setRequestHeader combine）。
    assert!(
        wire.contains("X-MiXeD-Case\x1ev1, v2"),
        "XHR 异步路 setRequestHeader 线上应保首见原始大小写，got: {wire}"
    );
    // 响应面同步 XHR 语义保持（DONE + responseText 填充）。
    assert_eq!(
        sandbox.execute("__x.responseText").unwrap().value,
        "body"
    );
}
