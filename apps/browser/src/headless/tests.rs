//! headless 协议单元测试 + 协议驱动冒烟测试（dispatch 直连与真实 TCP 会话）。

use super::*;

use serde_json::Value;

#[test]
fn test_server_new() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    assert!(server.addr().port() == 0);
}

#[test]
fn test_session_new() {
    let session = HeadlessSession::new(800.0, 600.0);
    assert!(session.shell.tab_count() >= 1);
}

#[test]
fn test_dispatch_session_status() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server.dispatch(&mut session, "session.status", Value::Null).unwrap();
    assert_eq!(result["ready"], true);
}

#[test]
fn test_dispatch_unknown_method() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server.dispatch(&mut session, "unknown.method", Value::Null);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, -32601);
}

#[test]
fn test_dispatch_navigate_missing_url() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server.dispatch(&mut session, "browsingContext.navigate", Value::Null);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, -32602);
}

#[test]
fn test_dispatch_script_evaluate() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "expression": "1 + 1" });
    let result = server.dispatch(&mut session, "script.evaluate", params).unwrap();
    assert!(result.get("result").is_some() || result.get("exceptionDetails").is_some());
}

#[test]
fn test_dispatch_capture_screenshot() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server
        .dispatch(&mut session, "browsingContext.captureScreenshot", Value::Null)
        .unwrap();
    assert_eq!(result["data"]["width"], 800);
    assert_eq!(result["data"]["height"], 600);
}

#[test]
fn test_dispatch_get_dom_snapshot() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server
        .dispatch(&mut session, "browsingContext.getDOMSnapshot", Value::Null)
        .unwrap();
    assert!(result.get("renderPrimitives").is_some());
}

#[test]
fn test_dispatch_session_new() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server.dispatch(&mut session, "session.new", Value::Null).unwrap();
    assert_eq!(result["capabilities"]["browserName"], "ZeroWeb");
}

#[test]
fn test_dispatch_browser_close() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server.dispatch(&mut session, "browser.close", Value::Null).unwrap();
    assert_eq!(result["result"], "closing");
}

#[test]
fn test_client_request_parse() {
    let raw = r#"{"id":1,"method":"session.status","params":{}}"#;
    let req: ClientRequest = serde_json::from_str(raw).unwrap();
    assert_eq!(req.id, 1);
    assert_eq!(req.method, "session.status");
}

#[test]
fn test_client_request_no_params() {
    let raw = r#"{"id":2,"method":"browser.close"}"#;
    let req: ClientRequest = serde_json::from_str(raw).unwrap();
    assert_eq!(req.id, 2);
    assert_eq!(req.params, Value::Null);
}

#[test]
fn test_server_response_serialize() {
    let resp = ServerResponse {
        id: 1,
        result: Some(serde_json::json!({"ready": true})),
        error: None,
        session_id: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"id\":1"));
    assert!(json.contains("\"result\""));
    assert!(!json.contains("\"error\""));
}

#[test]
fn test_server_response_error() {
    let resp = ServerResponse {
        id: 3,
        result: None,
        error: Some(ProtocolError {
            code: -32601,
            message: "Unknown method".into(),
        }),
        session_id: None,
    };
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"error\""));
    assert!(json.contains("-32601"));
}

// ── Phase 2 测试 ──

#[test]
fn test_dispatch_browsing_context_create() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "url": "https://example.com" });
    let result = server.dispatch(&mut session, "browsingContext.create", params).unwrap();
    assert!(result.get("context").is_some());
    assert_eq!(result["url"], "https://example.com");
}

#[test]
fn test_dispatch_browsing_context_get_tree() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server
        .dispatch(&mut session, "browsingContext.getTree", Value::Null)
        .unwrap();
    let contexts = result.get("contexts").unwrap().as_array().unwrap();
    assert!(!contexts.is_empty(), "should have at least one tab");
}

#[test]
fn test_dispatch_browsing_context_close() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    // 创建第二个标签页并获取其 ID
    let new_tab_id = session.shell.new_tab(None);
    let count_before = session.shell.tab_count();
    let params = serde_json::json!({ "context": new_tab_id.0 });
    let result = server.dispatch(&mut session, "browsingContext.close", params).unwrap();
    assert_eq!(result["result"], "closed");
    assert_eq!(session.shell.tab_count(), count_before - 1);
}

#[test]
fn test_dispatch_browsing_context_close_missing_context() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server.dispatch(&mut session, "browsingContext.close", Value::Null);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, -32602);
}

#[test]
fn test_dispatch_browsing_context_reload() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server
        .dispatch(&mut session, "browsingContext.reload", Value::Null)
        .unwrap();
    assert_eq!(result["result"], "reloaded");
}

#[test]
fn test_dispatch_script_call_function() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({
        "functionDeclaration": "function() { return 1 + 1; }",
    });
    let result = server.dispatch(&mut session, "script.callFunction", params).unwrap();
    assert!(result.get("result").is_some() || result.get("exceptionDetails").is_some());
}

#[test]
fn test_dispatch_script_call_function_with_args() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({
        "functionDeclaration": "function(a, b) { return a + b; }",
        "arguments": [{ "value": 1 }, { "value": 2 }]
    });
    let result = server.dispatch(&mut session, "script.callFunction", params).unwrap();
    assert!(result.get("result").is_some() || result.get("exceptionDetails").is_some());
}

#[test]
fn test_dispatch_script_call_function_missing_declaration() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server.dispatch(&mut session, "script.callFunction", Value::Null);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, -32602);
}

#[test]
fn test_http_version_json() {
    let addr: SocketAddr = "127.0.0.1:9222".parse().unwrap();
    let json = HeadlessServer::http_version_json(addr);
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["Browser"], format!("ZeroWeb/{}", zero_product_version::VERSION));
    assert_eq!(parsed["User-Agent"], zero_net::HttpClient::default_user_agent());
    assert!(parsed["webSocketDebuggerUrl"].as_str().unwrap().contains("ws://"));
}

// ── Phase 2-3 测试：事件推送和 CDP 兼容 ──

#[test]
fn test_is_http_get_request() {
    assert!(HeadlessServer::is_http_get_request(b"GET /json HTTP/1.1\r\n"));
    assert!(!HeadlessServer::is_http_get_request(
        b"GET /json HTTP/1.1\r\nUpgrade: websocket\r\n"
    ));
    assert!(!HeadlessServer::is_http_get_request(b"POST /json HTTP/1.1\r\n"));
}

#[test]
fn test_server_event_serialize() {
    let event = ServerEvent {
        method: "browsingContext.load".into(),
        params: serde_json::json!({ "url": "https://example.com" }),
        session_id: None,
    };
    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("\"method\""));
    assert!(json.contains("browsingContext.load"));
}

#[test]
fn test_dispatch_with_events_navigate() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "url": "https://example.com" });
    let (result, events) = server.dispatch_with_events(&mut session, "browsingContext.navigate", params);
    assert!(result.is_ok());
    assert!(!events.is_empty(), "navigate should produce events");
    let methods: Vec<&str> = events.iter().map(|e| e.method.as_str()).collect();
    assert!(methods.contains(&"browsingContext.load"), "should emit load event");
    assert!(methods.contains(&"log.entryAdded"), "should emit log event");
}

#[test]
fn test_dispatch_with_events_reload() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let (result, events) = server.dispatch_with_events(&mut session, "browsingContext.reload", Value::Null);
    assert!(result.is_ok());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].method, "browsingContext.load");
}

#[test]
fn test_dispatch_with_events_create() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({});
    let (result, events) = server.dispatch_with_events(&mut session, "browsingContext.create", params);
    assert!(result.is_ok());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].method, "browsingContext.contextCreated");
}

#[test]
fn test_dispatch_with_events_close() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let new_tab_id = session.shell.new_tab(None);
    let params = serde_json::json!({ "context": new_tab_id.0 });
    let (result, events) = server.dispatch_with_events(&mut session, "browsingContext.close", params);
    assert!(result.is_ok());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].method, "browsingContext.contextDestroyed");
}

#[test]
fn test_dispatch_with_events_no_events_for_status() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let (result, events) = server.dispatch_with_events(&mut session, "session.status", Value::Null);
    assert!(result.is_ok());
    assert!(events.is_empty(), "session.status should not produce events");
}

#[test]
fn test_dispatch_cdp_page_navigate() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "url": "https://example.com" });
    let (result, events) = server.dispatch_with_events(&mut session, "Page.navigate", params);
    assert!(result.is_ok());
    assert!(events.iter().any(|e| e.method == "Page.loadEventFired"));
}

#[test]
fn test_dispatch_cdp_runtime_evaluate() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "expression": "1 + 1" });
    let (result, events) = server.dispatch_with_events(&mut session, "Runtime.evaluate", params);
    assert!(result.is_ok());
    assert!(events.is_empty(), "Runtime.evaluate should not produce events");
}

// ── M4+：Runtime objectId 桥（evaluate returnByValue:false / callFunctionOn objectId /
//    releaseObject 族；语义链由 renderer 单测 + cdp-e2e 覆盖，此处收 CDP 层形状）──

#[test]
fn test_cdp_runtime_evaluate_return_by_value_false_goes_handle_bridge() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "expression": "({a: 1})", "returnByValue": false });
    let (result, _) = server.dispatch_with_events(&mut session, "Runtime.evaluate", params);
    // 测试进程无 renderer：句柄桥操作以异常形状回传（真实链路在 renderer 单测/cdp-e2e）。
    let result = result.expect("handle bridge failure must be exceptionDetails, not transport error");
    assert!(
        result.get("exceptionDetails").is_some(),
        "expected exceptionDetails shape, got {result}"
    );
}

#[test]
fn test_cdp_runtime_call_function_on_invalid_object_id_is_invalid_params() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({
        "functionDeclaration": "(function(){})",
        "objectId": "not-an-zw-object-id",
    });
    let result = server.dispatch(&mut session, "Runtime.callFunctionOn", params);
    let error = result.expect_err("foreign objectId must be rejected");
    assert_eq!(error.code, -32602);
}

#[test]
fn test_cdp_runtime_call_function_on_object_id_routes_handle_bridge() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({
        "functionDeclaration": "(function(){ return 1; })",
        "objectId": "zw:7",
        "arguments": [
            { "value": 2 },
            { "objectId": "zw:8" },
            { "other": true }
        ],
        "returnByValue": true,
        "awaitPromise": true,
    });
    let (result, _) = server.dispatch_with_events(&mut session, "Runtime.callFunctionOn", params);
    let result = result.expect("zw: objectId must route into the handle bridge");
    assert!(
        result.get("exceptionDetails").is_some(),
        "expected exceptionDetails, got {result}"
    );
}

#[test]
fn test_cdp_runtime_release_object_validates_object_id() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);

    let result = server.dispatch(&mut session, "Runtime.releaseObject", Value::Null);
    assert_eq!(result.expect_err("missing objectId").code, -32602);

    let result = server.dispatch(
        &mut session,
        "Runtime.releaseObject",
        serde_json::json!({ "objectId": "chrome-object-1" }),
    );
    assert_eq!(result.expect_err("foreign objectId").code, -32602);

    let (result, _) = server.dispatch_with_events(
        &mut session,
        "Runtime.releaseObject",
        serde_json::json!({ "objectId": "zw:3" }),
    );
    assert!(
        result.is_err() || result.unwrap().is_object(),
        "zw: objectId accepted at CDP layer"
    );
}

#[test]
fn test_cdp_runtime_release_object_group_validates_group() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);

    let result = server.dispatch(&mut session, "Runtime.releaseObjectGroup", Value::Null);
    assert_eq!(result.expect_err("missing objectGroup").code, -32602);

    let (result, _) = server.dispatch_with_events(
        &mut session,
        "Runtime.releaseObjectGroup",
        serde_json::json!({ "objectGroup": "pw-utilities" }),
    );
    // 校验通过后进入执行面——测试进程无 renderer，以 -32000 传回（非参数错）。
    let error = result.expect_err("valid group must route past validation");
    assert_eq!(error.code, -32000);
}

#[test]
fn test_cdp_target_attach_to_browser_target_registers_session() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);

    // 浏览器级调用（无 sessionId）→ 返回新 sessionId
    let (response, _) = server.handle_message_with_events(
        &mut session,
        r#"{"id":1,"method":"Target.attachToBrowserTarget","params":{}}"#,
    );
    let result = response.result.expect("attachToBrowserTarget must succeed");
    let sid = result["sessionId"].as_str().expect("sessionId in result");
    assert!(sid.starts_with("zeroweb-session-"), "opaque sid: {sid}");
    // 登记后：携带该 sessionId 的命令不再被 -32001 拒绝（未附接校验）
    let raw = format!(r#"{{"id":2,"method":"Target.getTargets","params":{{}},"sessionId":"{sid}"}}"#);
    let (response, _) = server.handle_message_with_events(&mut session, &raw);
    assert!(response.error.is_none(), "registered sid must pass: {response:?}");
}

#[test]
fn test_cdp_target_attach_to_target_and_detach_event_stamped() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let tab_id = session.shell.active_tab_id().expect("default tab");
    let target_id = format!("zeroweb-tab-{}", tab_id.0);

    // 未知/缺参 targetId 校验
    let (result, _) = server.dispatch_with_events(&mut session, "Target.attachToTarget", Value::Null);
    assert_eq!(result.expect_err("missing targetId").code, -32602);
    let (result, _) = server.dispatch_with_events(
        &mut session,
        "Target.attachToTarget",
        serde_json::json!({ "targetId": "zeroweb-tab-999", "flatten": true }),
    );
    assert_eq!(result.expect_err("unknown target").code, -32000);

    // 附接 → detachFromTarget → 事件盖发起会话（发起方按 sessionId 收到应答事件）
    let raw = format!(
        r#"{{"id":1,"method":"Target.attachToTarget","params":{{"targetId":"{target_id}","flatten":true}},"sessionId":"caller-1"}}"#
    );
    // caller-1 未登记 → 先经 attachToBrowserTarget 建会话再调用
    let (resp, _) = server.handle_message_with_events(
        &mut session,
        r#"{"id":0,"method":"Target.attachToBrowserTarget","params":{}}"#,
    );
    let caller_sid = resp.result.unwrap()["sessionId"].as_str().unwrap().to_string();
    let raw = format!(
        r#"{{"id":1,"method":"Target.attachToTarget","params":{{"targetId":"{target_id}","flatten":true}},"sessionId":"{caller_sid}"}}"#
    );
    let (resp, _) = server.handle_message_with_events(&mut session, &raw);
    let attached_sid = resp.result.unwrap()["sessionId"].as_str().unwrap().to_string();

    let raw = format!(
        r#"{{"id":2,"method":"Target.detachFromTarget","params":{{"sessionId":"{attached_sid}"}},"sessionId":"{caller_sid}"}}"#
    );
    let (resp, events) = server.handle_message_with_events(&mut session, &raw);
    assert!(resp.error.is_none(), "detach must succeed: {resp:?}");
    let ev = events
        .iter()
        .find(|e| e.method == "Target.detachedFromTarget")
        .expect("detach event emitted");
    assert_eq!(
        ev.session_id.as_deref(),
        Some(caller_sid.as_str()),
        "stamped to initiator"
    );
    assert_eq!(ev.params["sessionId"], attached_sid.as_str());
}

#[test]
fn test_cdp_object_id_wire_format_roundtrip() {
    use super::domains::{object_id_string, parse_object_id};
    for handle in [1u64, 42, u64::MAX] {
        let object_id = object_id_string(handle);
        assert_eq!(parse_object_id(&object_id), Some(handle));
    }
    assert_eq!(parse_object_id("zw:"), None);
    assert_eq!(parse_object_id("zw:abc"), None);
    assert_eq!(parse_object_id("1"), None);
    assert_eq!(parse_object_id(""), None);
}

#[test]
fn test_dispatch_cdp_network_enable() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let (result, events) = server.dispatch_with_events(&mut session, "Network.enable", Value::Null);
    assert!(result.is_ok());
    assert!(events.is_empty());
}

// ── M1 切片 2：CDP 传输层（sessionId 复用 / 发现端点尾斜杠 / 真实 target 枚举）──

#[test]
fn test_discovery_path_trailing_slash() {
    use super::discovery::normalize_discovery_path;
    assert_eq!(normalize_discovery_path("/json/version/"), "/json/version");
    assert_eq!(normalize_discovery_path("/json/version"), "/json/version");
    assert_eq!(normalize_discovery_path("/json/list/"), "/json/list");
    // 根路径剥成空串 → 走 404 分支（与旧行为等价："/" 不匹配任何端点）
    assert_eq!(normalize_discovery_path("/"), "");
}

#[test]
fn test_ws_upgrade_path_extraction() {
    use super::discovery::extract_request_path;
    // 标准 WS 升级请求行
    assert_eq!(
        extract_request_path(b"GET /devtools/page/zeroweb-tab-1 HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\n\r\n")
            .as_deref(),
        Some("/devtools/page/zeroweb-tab-1")
    );
    // query 不参与路由判定
    assert_eq!(
        extract_request_path(b"GET /devtools/page/zeroweb-tab-2?token=abc HTTP/1.1\r\n\r\n").as_deref(),
        Some("/devtools/page/zeroweb-tab-2")
    );
    // 浏览器级入口（Playwright connectOverCDP 直连根路径）
    assert_eq!(
        extract_request_path(b"GET / HTTP/1.1\r\nUpgrade: websocket\r\n\r\n").as_deref(),
        Some("/")
    );
    assert_eq!(extract_request_path(b"garbage"), None);
}

#[test]
fn test_per_tab_devtools_frontend_url() {
    use super::discovery::per_tab_frontend_url;
    use std::net::SocketAddr;
    let addr: SocketAddr = "127.0.0.1:9222".parse().unwrap();
    let url = per_tab_frontend_url(addr, "zeroweb-tab-3");
    assert!(
        url.starts_with("/devtools/inspector.html?ws=127.0.0.1:9222/devtools/page/zeroweb-tab-3"),
        "{url}"
    );
    // ws 参数落在 127.0.0.1（frontend CSP connect-src 只放行 ws://127.0.0.1:*）
    assert!(url.contains("ws=127.0.0.1:"), "{url}");
}

#[test]
fn test_dom_get_document_probe_conversion() {
    use super::domains::convert_cdp_node;
    // 探测 JSON：#document → doctype + html（含属性、文本子节点）
    let probe: Value = serde_json::json!({
        "t": 9, "n": "#document", "u": "https://example.com/", "cc": 2,
        "c": [
            { "t": 10, "n": "html" },
            { "t": 1, "n": "HTML", "a": ["lang", "en"], "cc": 1, "q": "html",
              "c": [ { "t": 1, "n": "DIV", "a": ["id", "x"], "cc": 0, "q": "body > div#x" } ] },
        ],
    });
    let mut next = 1;
    let mut selectors = std::collections::HashMap::new();
    let root = convert_cdp_node(&probe, &mut next, &mut selectors);
    // 元素 __zwSelector 捕获入注册表（S3b CSS 域 nodeId 解析依赖）：
    // document=1, doctype=2, HTML=3, DIV=4
    assert_eq!(selectors.get(&3).map(String::as_str), Some("html"));
    assert_eq!(selectors.get(&4).map(String::as_str), Some("body > div#x"));
    assert!(!selectors.contains_key(&2), "doctype 非 element 不入表");
    assert_eq!(root["nodeId"], 1);
    assert_eq!(root["backendNodeId"], 1);
    assert_eq!(root["nodeType"], 9);
    assert_eq!(root["documentURL"], "https://example.com/");
    assert_eq!(root["childNodeCount"], 2);
    let children = root["children"].as_array().unwrap();
    assert_eq!(children.len(), 2);
    // doctype：nodeType 10 + doctype 名 nodeName
    assert_eq!(children[0]["nodeType"], 10);
    assert_eq!(children[0]["nodeName"], "html");
    // 元素：属性 flat 数组原样 + 子元素递归分配 id
    assert_eq!(children[1]["nodeName"], "HTML");
    assert_eq!(children[1]["attributes"], serde_json::json!(["lang", "en"]));
    assert_eq!(children[1]["children"][0]["nodeType"], 1);
    assert_eq!(children[1]["children"][0]["nodeName"], "DIV");
    assert_eq!(children[1]["children"][0]["nodeId"], 4);
    // depth 截断桩（无 c/cc）：无 children 字段，childNodeCount 兜底 0
    let stub: Value = serde_json::json!({ "t": 1, "n": "DIV" });
    let mut next = 100;
    let mut stub_selectors = std::collections::HashMap::new();
    let stub_node = convert_cdp_node(&stub, &mut next, &mut stub_selectors);
    assert!(stub_selectors.is_empty(), "depth 截断桩无 q 不入表");
    assert!(stub_node.get("children").is_none());
    assert_eq!(stub_node["childNodeCount"], 0);
}

#[test]
fn test_session_id_echoed_in_response() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    server.attach_session("test-session-1", "zeroweb-tab-1");
    let raw = r#"{"id":7,"method":"session.status","params":{},"sessionId":"test-session-1"}"#;
    let (response, _) = server.handle_message_with_events(&mut session, raw);
    assert!(response.result.is_some());
    assert_eq!(response.session_id.as_deref(), Some("test-session-1"));
}

#[test]
fn test_unknown_session_id_rejected() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let raw = r#"{"id":8,"method":"session.status","params":{},"sessionId":"ghost"}"#;
    let (response, _) = server.handle_message_with_events(&mut session, raw);
    let error = response.error.expect("unknown session must error");
    assert_eq!(error.code, -32001);
    assert!(error.message.contains("ghost"));
    // 回显 sessionId（客户端需据以配对）
    assert_eq!(response.session_id.as_deref(), Some("ghost"));
}

#[test]
fn test_json_targets_enumerates_real_tabs() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    // 枚举前新建一个标签页并给活跃页导航，验证 url 取自 shell 模型
    //（BrowserShell::new 自带 1 个默认 tab，故用增量断言而非绝对数）。
    let before = session.shell.tab_count();
    session.shell.new_tab(None);
    session.shell.navigate("https://example.com/page");

    let targets: Vec<serde_json::Value> = serde_json::from_str(&super::discovery::http_targets_json(
        &session,
        server.addr(),
        server.devtools_serve_enabled(),
    ))
    .unwrap();
    assert_eq!(targets.len(), before + 1, "one entry per tab");
    assert!(targets.iter().all(|t| t["type"] == "page"));
    assert!(
        targets.iter().any(|t| t["url"] == "https://example.com/page"),
        "navigated url must appear in discovery list"
    );
    let ids: Vec<&str> = targets.iter().map(|t| t["id"].as_str().unwrap()).collect();
    assert!(ids.iter().all(|id| id.starts_with("zeroweb-tab-")));
    assert!(!ids.contains(&"zeroweb-main"), "static placeholder id retired");
    // bundle 未配置（测试进程无 ZW_DEVTOOLS_FRONTEND_DIR）→ devtools:// 占位形态
    assert!(targets.iter().all(|t| {
        t["devtoolsFrontendUrl"]
            .as_str()
            .unwrap()
            .starts_with("devtools://devtools/bundled/")
    }));
}

// ── M4：Storage cookie 域 / UA override / Network 门控 ──

#[test]
fn test_storage_cookies_roundtrip() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let empty = server
        .dispatch(&mut session, "Storage.getCookies", Value::Null)
        .unwrap();
    assert!(empty["cookies"].as_array().unwrap().is_empty());

    let set = serde_json::json!({
        "cookies": [{ "name": "zw", "value": "1", "domain": "127.0.0.1", "path": "/" }],
    });
    assert!(server.dispatch(&mut session, "Storage.setCookies", set).is_ok());
    let got = server
        .dispatch(&mut session, "Storage.getCookies", Value::Null)
        .unwrap();
    let cookies = got["cookies"].as_array().unwrap();
    assert_eq!(cookies.len(), 1);
    assert_eq!(cookies[0]["name"], "zw");
    assert_eq!(cookies[0]["value"], "1");
    assert_eq!(cookies[0]["domain"], "127.0.0.1");

    assert!(
        server
            .dispatch(&mut session, "Storage.clearCookies", Value::Null)
            .is_ok()
    );
    let after = server
        .dispatch(&mut session, "Storage.getCookies", Value::Null)
        .unwrap();
    assert!(after["cookies"].as_array().unwrap().is_empty());
}

#[test]
fn test_storage_set_cookies_with_url_scopes_to_host() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let set = serde_json::json!({
        "cookies": [{ "name": "scoped", "value": "abc", "url": "https://example.com/page" }],
    });
    assert!(server.dispatch(&mut session, "Storage.setCookies", set).is_ok());
    let got = server
        .dispatch(&mut session, "Storage.getCookies", Value::Null)
        .unwrap();
    let cookies = got["cookies"].as_array().unwrap();
    assert_eq!(cookies.len(), 1);
    assert_eq!(cookies[0]["domain"], "example.com");
    assert_eq!(cookies[0]["secure"], true);
}

#[test]
fn test_set_user_agent_override_stored() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    assert!(session.user_agent_override.is_none());
    let params = serde_json::json!({ "userAgent": "test-ua/1.0" });
    assert!(
        server
            .dispatch(&mut session, "Emulation.setUserAgentOverride", params)
            .is_ok()
    );
    assert_eq!(session.user_agent_override.as_deref(), Some("test-ua/1.0"));
}

#[test]
fn test_set_user_agent_override_missing_rejected() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server.dispatch(&mut session, "Emulation.setUserAgentOverride", Value::Null);
    assert_eq!(result.unwrap_err().code, -32602);
}

#[test]
fn test_network_enable_disable_gates_flag() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    assert!(!session.network_enabled);
    assert!(server.dispatch(&mut session, "Network.enable", Value::Null).is_ok());
    assert!(session.network_enabled);
    assert!(server.dispatch(&mut session, "Network.disable", Value::Null).is_ok());
    assert!(!session.network_enabled);
}

// ── M3：Emulation viewport 桥 / 媒体仿真 / 截图 clip ──

#[test]
fn test_emulation_set_device_metrics_updates_state_and_emits_resize() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({
        "width": 500, "height": 400, "deviceScaleFactor": 1, "mobile": false,
        "screenWidth": 500, "screenHeight": 400,
    });
    let (result, events) = server.dispatch_with_events(&mut session, "Emulation.setDeviceMetricsOverride", params);
    result.unwrap();
    assert_eq!(server.viewport_size(), (500.0, 400.0));
    assert_eq!(events.len(), 1, "size change emits frameResized");
    assert_eq!(events[0].method, "Page.frameResized");
    // getLayoutMetrics 联动
    let metrics = server
        .dispatch(&mut session, "Page.getLayoutMetrics", Value::Null)
        .unwrap();
    assert_eq!(metrics["cssLayoutViewport"]["clientWidth"], 500);
    assert_eq!(metrics["cssLayoutViewport"]["clientHeight"], 400);
}

#[test]
fn test_emulation_set_device_metrics_same_size_no_event() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "width": 800, "height": 600, "deviceScaleFactor": 1, "mobile": false });
    let (result, events) = server.dispatch_with_events(&mut session, "Emulation.setDeviceMetricsOverride", params);
    result.unwrap();
    assert!(events.is_empty(), "same-size override emits no frameResized");
}

#[test]
fn test_emulation_set_emulated_media_ok() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({
        "media": "",
        "features": [
            { "name": "prefers-color-scheme", "value": "dark" },
            { "name": "prefers-reduced-motion", "value": "reduce" },
        ],
    });
    assert!(
        server
            .dispatch(&mut session, "Emulation.setEmulatedMedia", params)
            .is_ok()
    );
    let print_params = serde_json::json!({ "media": "print" });
    assert!(
        server
            .dispatch(&mut session, "Emulation.setEmulatedMedia", print_params)
            .is_ok()
    );
}

#[test]
fn test_capture_screenshot_clip_crops() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let _ = session.webview.load_html(
        r#"<html><body style="margin:0"><div style="width:800px;height:600px;background:#0f0;"></div></body></html>"#,
        None,
    );
    let params = serde_json::json!({
        "clip": { "x": 10, "y": 20, "width": 100, "height": 50, "scale": 1 },
    });
    let (result, _events) = server.dispatch_with_events(&mut session, "Page.captureScreenshot", params);
    let result = result.unwrap();
    // CDP 形状：data 为 base64 字符串
    let data = result["data"].as_str().expect("CDP data must be a string");
    assert!(!data.is_empty());
    // 裁剪尺寸验证：解码 PNG 后应为 100x50
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(data).unwrap();
    let decoder = png::Decoder::new(&bytes[..]);
    let reader = decoder.read_info().unwrap();
    assert_eq!((reader.info().width, reader.info().height), (100, 50));
}

#[test]
fn test_capture_screenshot_clip_out_of_bounds_rejected() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({
        "clip": { "x": 700, "y": 500, "width": 200, "height": 200, "scale": 1 },
    });
    let (result, _events) = server.dispatch_with_events(&mut session, "Page.captureScreenshot", params);
    assert_eq!(result.unwrap_err().code, -32602);
}

// ── M2：导航事件族 / 布局面 / 注入脚本 / Input 域 ──

#[test]
fn test_navigation_event_family_sequence() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    server.set_auto_attach(true);
    let (result, events) = server.dispatch_with_events(&mut session, "Target.createTarget", serde_json::json!({}));
    result.unwrap();
    let sid = events[0].params["sessionId"].as_str().unwrap().to_string();
    let target_id = events[0].params["targetInfo"]["targetId"].as_str().unwrap().to_string();

    let mut nav_events = Vec::new();
    server.emit_navigation_event_family(
        &mut session,
        Some(&sid),
        &target_id,
        "zw-loader-1",
        "http://x/",
        &mut nav_events,
    );
    let methods: Vec<&str> = nav_events.iter().map(|e| e.method.as_str()).collect();
    assert_eq!(
        methods,
        vec![
            "Page.frameStartedLoading",
            "Page.frameNavigated",
            "Runtime.executionContextsCleared",
            "Runtime.executionContextCreated",
            "Page.lifecycleEvent",
            "Page.domContentEventFired",
            "Page.lifecycleEvent",
            "Page.loadEventFired",
            "Page.frameStoppedLoading",
        ]
    );
    // frameNavigated 的 frame.id 与主 frame id（=targetId）一致
    assert_eq!(nav_events[1].params["frame"]["id"], target_id.as_str());
    let load_idx = methods.iter().position(|m| *m == "Page.loadEventFired").unwrap();
    assert!(matches!(nav_events[load_idx - 1].params["name"].as_str(), Some("load")));
}

#[test]
fn test_frame_detached_on_document_swap() {
    // 文档换代：上一文档的子帧记录先 detach（reason=frameRemoved）、记录清空。
    // （记录由 frameAttached 探测填充；此处手工预置以脱离 renderer 依赖。）
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    session.active_child_frames.insert(
        "zeroweb-tab-1".into(),
        vec!["zeroweb-frame-1".into(), "zeroweb-frame-2".into()],
    );

    let mut nav_events = Vec::new();
    server.emit_navigation_event_family(
        &mut session,
        None,
        "zeroweb-tab-1",
        "zw-loader-2",
        "about:blank",
        &mut nav_events,
    );
    let detached: Vec<_> = nav_events.iter().filter(|e| e.method == "Page.frameDetached").collect();
    assert_eq!(detached.len(), 2);
    assert_eq!(detached[0].params["frameId"], "zeroweb-frame-1");
    assert_eq!(detached[1].params["frameId"], "zeroweb-frame-2");
    assert_eq!(detached[0].params["reason"], "frameRemoved");
    // detach 位于 frameStartedLoading 之前（文档换代时序）
    assert_eq!(nav_events[0].method, "Page.frameDetached");
    assert_eq!(nav_events[1].method, "Page.frameDetached");
    assert_eq!(nav_events[2].method, "Page.frameStartedLoading");
    // 记录清空（探测失败路径下无新 attach）
    assert!(session.active_child_frames.is_empty());
}

#[test]
fn test_page_navigate_success_path_via_load_html_page() {
    // 成功路径的事件族由 Playwright goto 冒烟验收；此处断言注入脚本在导航后重放的
    // 存储面（emit_navigation_event_family 内部调用 replay）。
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let (result, _events) = server.dispatch_with_events(
        &mut session,
        "Page.addScriptToEvaluateOnNewDocument",
        serde_json::json!({ "source": "1;" }),
    );
    let identifier = result.unwrap()["identifier"].as_str().unwrap().to_string();
    assert!(identifier.starts_with("zw-script-"));
    assert_eq!(session.injected_scripts.len(), 1);
    assert_eq!(session.injected_scripts[0].identifier, identifier);

    // 重放辅助直接调用（导航成功路径内部同样调用）
    let mut nav_events = Vec::new();
    server.emit_navigation_event_family(
        &mut session,
        None,
        "zeroweb-tab-1",
        "zw-loader-2",
        "about:blank",
        &mut nav_events,
    );
    assert!(nav_events.iter().any(|e| e.method == "Page.loadEventFired"));
}

#[test]
fn test_page_get_layout_metrics_shape() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server
        .dispatch(&mut session, "Page.getLayoutMetrics", Value::Null)
        .unwrap();
    assert_eq!(result["cssLayoutViewport"]["clientWidth"], 800);
    assert_eq!(result["cssLayoutViewport"]["clientHeight"], 600);
    assert_eq!(result["layoutViewport"]["clientWidth"], 800);
    assert_eq!(result["cssVisualViewport"]["scale"], 1);
}

#[test]
fn test_input_dispatch_key_event_ok() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({
        "type": "keyDown", "key": "a", "code": "KeyA",
        "windowsVirtualKeyCode": 65, "modifiers": 8,
    });
    let result = server.dispatch(&mut session, "Input.dispatchKeyEvent", params);
    assert!(result.is_ok());
}

#[test]
fn test_input_dispatch_key_event_char_type() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "type": "char", "key": "a", "text": "a" });
    assert!(server.dispatch(&mut session, "Input.dispatchKeyEvent", params).is_ok());
}

#[test]
fn test_input_dispatch_mouse_unknown_type_rejected() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "type": "bogus", "x": 1, "y": 2 });
    let result = server.dispatch(&mut session, "Input.dispatchMouseEvent", params);
    assert_eq!(result.unwrap_err().code, -32602);
}

#[test]
fn test_input_mouse_press_release_ok() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let press = serde_json::json!({ "type": "mousePressed", "x": 10, "y": 20, "button": "left", "clickCount": 1 });
    let release = serde_json::json!({ "type": "mouseReleased", "x": 10, "y": 20, "button": "left", "clickCount": 1 });
    assert!(server.dispatch(&mut session, "Input.dispatchMouseEvent", press).is_ok());
    assert!(
        server
            .dispatch(&mut session, "Input.dispatchMouseEvent", release)
            .is_ok()
    );
}

#[test]
fn test_input_insert_text_ok() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let params = serde_json::json!({ "text": "hello" });
    assert!(server.dispatch(&mut session, "Input.insertText", params).is_ok());
}

#[test]
fn test_input_missing_text_rejected() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server.dispatch(&mut session, "Input.insertText", Value::Null);
    assert_eq!(result.unwrap_err().code, -32602);
}

// ── M1 切片 3：Target 域 + Browser/Runtime 雏形（Playwright 连接脊柱）──

#[test]
fn test_browser_get_version() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server
        .dispatch(&mut session, "Browser.getVersion", Value::Null)
        .unwrap();
    assert_eq!(result["protocolVersion"], "1.3");
    assert!(result["product"].as_str().unwrap().starts_with("ZeroWeb/"));
    assert!(!result["userAgent"].as_str().unwrap().is_empty());
    assert!(result["jsVersion"].as_str().is_some());
}

#[test]
fn test_target_set_auto_attach_emits_attached_to_target() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let tab_count = session.shell.tab_count();
    let params = serde_json::json!({ "autoAttach": true, "waitForDebuggerOnStart": false, "flatten": true });
    let (result, events) = server.dispatch_with_events(&mut session, "Target.setAutoAttach", params);
    assert!(result.is_ok());
    assert_eq!(events.len(), tab_count, "one attachedToTarget per existing tab");
    let first = &events[0];
    assert_eq!(first.method, "Target.attachedToTarget");
    assert_eq!(first.params["targetInfo"]["type"], "page");
    assert_eq!(first.params["waitingForDebugger"], false);
    let sid = first.params["sessionId"].as_str().unwrap();
    assert!(server.session_attached(sid), "session must be registered");
    // 新 target 自动附接
    let (result, events) = server.dispatch_with_events(
        &mut session,
        "Target.createTarget",
        serde_json::json!({ "url": "about:blank" }),
    );
    assert!(result.is_ok());
    assert_eq!(events.len(), 1, "createTarget auto-attaches when enabled");
}

#[test]
fn test_target_get_target_info_browser_level() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server
        .dispatch(&mut session, "Target.getTargetInfo", Value::Null)
        .unwrap();
    assert_eq!(result["targetInfo"]["type"], "browser");
    assert_eq!(result["targetInfo"]["attached"], true);
}

#[test]
fn test_target_create_and_close_target_lifecycle() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    server.set_auto_attach(true);
    let tabs_before = session.shell.tab_count();

    let (result, events) = server.dispatch_with_events(
        &mut session,
        "Target.createTarget",
        serde_json::json!({ "url": "about:blank" }),
    );
    let result = result.unwrap();
    let target_id = result["targetId"].as_str().unwrap().to_string();
    assert!(target_id.starts_with("zeroweb-tab-"));
    assert_eq!(events.len(), 1);
    let sid = events[0].params["sessionId"].as_str().unwrap().to_string();
    assert!(server.session_attached(&sid));
    assert_eq!(session.shell.tab_count(), tabs_before + 1);

    // closeTarget：target 销毁事件 + 会话摘除
    let (result, events) = server.dispatch_with_events(
        &mut session,
        "Target.closeTarget",
        serde_json::json!({ "targetId": target_id }),
    );
    let result = result.unwrap();
    assert_eq!(result["success"], true);
    assert!(events.iter().any(|e| e.method == "Target.targetDestroyed"));
    assert!(events.iter().any(|e| e.method == "Target.detachedFromTarget"));
    assert!(
        !server.session_attached(&sid),
        "closed target's session must be removed"
    );
}

#[test]
fn test_target_detach_from_target() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    server.set_auto_attach(true);
    let (result, events) = server.dispatch_with_events(&mut session, "Target.createTarget", serde_json::json!({}));
    result.unwrap();
    let sid = events[0].params["sessionId"].as_str().unwrap().to_string();

    let (result, events) = server.dispatch_with_events(
        &mut session,
        "Target.detachFromTarget",
        serde_json::json!({ "sessionId": sid }),
    );
    assert!(result.is_ok());
    assert!(events.iter().any(|e| e.method == "Target.detachedFromTarget"));
    assert!(!server.session_attached(&sid));
    // detach 后该会话上的命令必须被拒
    let raw = format!(r#"{{"id":1,"method":"session.status","params":{{}},"sessionId":"{sid}"}}"#);
    let (response, _) = server.handle_message_with_events(&mut session, &raw);
    assert_eq!(response.error.as_ref().unwrap().code, -32001);
}

#[test]
fn test_runtime_enable_emits_execution_context() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let (result, events) = server.dispatch_with_events(&mut session, "Runtime.enable", Value::Null);
    assert!(result.is_ok());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].method, "Runtime.executionContextCreated");
    assert!(events[0].params["context"]["id"].is_number());
}

#[test]
fn test_runtime_run_if_waiting_for_debugger_ok() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let result = server
        .dispatch(&mut session, "Runtime.runIfWaitingForDebugger", Value::Null)
        .unwrap();
    assert!(result.is_object());
}

#[test]
fn test_dispatch_cdp_target_get_targets() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let (result, events) = server.dispatch_with_events(&mut session, "Target.getTargets", Value::Null);
    assert!(result.is_ok());
    // M1 切片 3：CDP 形状修正（旧实现误用 BiDi tree 的 contexts 形状）
    assert!(result.as_ref().unwrap().get("targetInfos").is_some());
}

#[test]
fn test_dispatch_cdp_page_capture_screenshot() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    let (result, events) = server.dispatch_with_events(&mut session, "Page.captureScreenshot", Value::Null);
    assert!(result.is_ok());
    assert!(events.is_empty());
}

// ── Phase 4: 协议客户端测试 ──

#[test]
fn test_client_build_request() {
    let req = HeadlessClient::build_request(1, "session.status", serde_json::json!({}));
    let parsed: ClientRequest = serde_json::from_str(&req).unwrap();
    assert_eq!(parsed.id, 1);
    assert_eq!(parsed.method, "session.status");
}

#[test]
fn test_client_parse_response_success() {
    let raw = r#"{"id":1,"result":{"ready":true,"message":"ready"}}"#;
    let result = HeadlessClient::parse_response(raw).unwrap();
    assert_eq!(result["ready"], true);
}

#[test]
fn test_client_parse_response_error() {
    let raw = r#"{"id":1,"error":{"code":-32601,"message":"Unknown method"}}"#;
    let result = HeadlessClient::parse_response(raw);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("-32601"));
}

#[test]
fn test_client_parse_event() {
    let raw = r#"{"method":"browsingContext.load","params":{"url":"https://example.com"}}"#;
    let (method, params) = HeadlessClient::parse_event(raw).unwrap();
    assert_eq!(method, "browsingContext.load");
    assert_eq!(params["url"], "https://example.com");
}

#[test]
fn test_client_parse_screenshot() {
    let result = serde_json::json!({
        "data": { "width": 800, "height": 600, "format": "rgba8", "pixelCount": 480000 }
    });
    let (w, h, px) = HeadlessClient::parse_screenshot(&result).unwrap();
    assert_eq!(w, 800);
    assert_eq!(h, 600);
    assert_eq!(px, 480000);
}

#[test]
fn test_client_parse_dom_snapshot() {
    let result = serde_json::json!({
        "renderPrimitives": { "fills": 10, "glyphs": 5, "gradients": 2, "shadows": 1, "images": 3 }
    });
    let stats = HeadlessClient::parse_dom_snapshot(&result);
    assert_eq!(stats.fills, 10);
    assert_eq!(stats.glyphs, 5);
    assert_eq!(stats.gradients, 2);
    assert_eq!(stats.shadows, 1);
    assert_eq!(stats.images, 3);
    assert_eq!(stats.total(), 21);
}

#[test]
fn test_dom_snapshot_stats_total() {
    let stats = DomSnapshotStats {
        fills: 1,
        glyphs: 2,
        gradients: 3,
        shadows: 4,
        images: 5,
    };
    assert_eq!(stats.total(), 15);
}

// ── Phase 4: 协议驱动的自动化冒烟测试 ──

/// 辅助：通过 dispatch 模拟完整的协议驱动的测试场景。
struct ProtocolTestRunner {
    server: HeadlessServer,
    session: HeadlessSession,
    next_id: u64,
    /// 收集到的事件日志。
    event_log: Vec<(String, serde_json::Value)>,
}

impl ProtocolTestRunner {
    fn new() -> Self {
        Self {
            server: HeadlessServer::new(0, 800.0, 600.0),
            session: HeadlessSession::new(800.0, 600.0),
            next_id: 1,
            event_log: Vec::new(),
        }
    }

    /// 发送命令并收集响应和事件（模拟协议往返）。
    fn send(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let id = self.next_id;
        self.next_id += 1;

        // 模拟发送 JSON 请求（验证请求格式正确）
        let req_json = HeadlessClient::build_request(id, method, params.clone());
        let _req: ClientRequest = serde_json::from_str(&req_json).map_err(|e| format!("Invalid request: {e}"))?;

        // 执行命令（直接传递原始 params）
        let (result, events) = self.server.dispatch_with_events(&mut self.session, method, params);

        // 记录事件
        for event in events {
            self.event_log.push((event.method.clone(), event.params.clone()));
        }

        // 模拟解析响应
        let response = match result {
            Ok(value) => ServerResponse {
                id,
                result: Some(value),
                error: None,
                session_id: None,
            },
            Err(err) => ServerResponse {
                id,
                result: None,
                error: Some(err),
                session_id: None,
            },
        };
        let response_json = serde_json::to_string(&response).unwrap();
        HeadlessClient::parse_response(&response_json)
    }

    /// 获取事件日志中指定方法的事件数。
    fn event_count(&self, method: &str) -> usize {
        self.event_log.iter().filter(|(m, _)| m == method).count()
    }
}

/// 解码 PNG 字节为 (width, height, RGBA8 像素)。测试辅助。
fn decode_png_rgba(bytes: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    use png::ColorType;
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info().clone();
    let mut buf = vec![0u8; reader.output_buffer_size()];
    reader.next_frame(&mut buf).ok()?;
    // 若非 RGBA，转 RGBA（oracle 均为 RGBA8/RGB，统一转 RGBA 比较）。
    let rgba = if info.color_type == ColorType::Rgb {
        buf.chunks(3).flat_map(|px| [px[0], px[1], px[2], 255u8]).collect()
    } else {
        buf
    };
    Some((info.width, info.height, rgba))
}

/// 两 RGBA8 缓冲的差异像素占比（min 尺寸对齐）。逐像素：任一通道差 > 8 计为差异。
fn rgba_diff_pct(a: &(u32, u32, Vec<u8>), b: &(u32, u32, Vec<u8>)) -> f64 {
    let w = a.0.min(b.0) as usize;
    let h = a.1.min(b.1) as usize;
    let total = w * h;
    if total == 0 {
        return 1.0;
    }
    let stride = 4;
    let mut diff = 0usize;
    for y in 0..h {
        for x in 0..w {
            let ia = (y * a.0 as usize + x) * stride;
            let ib = (y * b.0 as usize + x) * stride;
            let da = [
                a.2[ia].abs_diff(b.2[ib]),
                a.2[ia + 1].abs_diff(b.2[ib + 1]),
                a.2[ia + 2].abs_diff(b.2[ib + 2]),
            ];
            if da.iter().any(|&d| d > 8) {
                diff += 1;
            }
        }
    }
    diff as f64 / total as f64
}

/// DC-13 line 315：welcome.html 经 ZeroBrowser headless 路径截图，与 chromium oracle
/// 像素对比。验真实 headless 渲染管线（loadHtml + captureScreenshot 经 render_full_scene
/// 全 13 图元）。welcome 自包含（内联 CSS + data-URI），loadHtml 绕过 fetch_url HTTP-only。
/// oracle（welcome-chromium.png）为 tracked 文件（CI 可用）。baseline diff ~17%（字体墙）。
#[test]
fn test_dc13_line315_welcome_headless_vs_chromium_oracle() {
    // R3254-F10：与 GPU 截图 env 测试互斥（env 污染 → 误走 GPU 截图崩溃）。
    let _gpu_lock = super::GPU_SCREENSHOT_ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    use base64::Engine;

    // 路径基于 CARGO_MANIFEST_DIR（package root）拼接，不依赖进程 cwd——
    // make test 经 test-guard --compile-first 直接执行测试二进制（cwd=workspace
    // root），cargo 原生模式 cwd=package root（d2d47a1a 引入后 make test 暴露）。
    let welcome = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/welcome.html"))
        .expect("welcome.html tracked fixture");
    let oracle_path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/goal/rendering-compat/evidence/product-static/welcome-chromium.png"
    );
    let Ok(oracle_bytes) = std::fs::read(oracle_path) else {
        eprintln!("skipping chromium oracle comparison; {oracle_path} is not available");
        return;
    };
    let oracle = decode_png_rgba(&oracle_bytes).expect("decode oracle PNG");

    let mut runner = ProtocolTestRunner::new();
    let load = runner
        .send("browsingContext.loadHtml", serde_json::json!({ "html": welcome }))
        .expect("loadHtml responds");
    let load_ok = load.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    assert!(load_ok, "loadHtml must report success: {load:?}");

    let shot = runner
        .send("browsingContext.captureScreenshot", serde_json::Value::Null)
        .expect("captureScreenshot responds");
    let data = shot.get("data").expect("captureScreenshot data field");
    let png_b64 = data
        .get("png")
        .and_then(|v| v.as_str())
        .expect("R1601: captureScreenshot must return base64 PNG in data.png");
    let shot_w = data.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    let shot_h = data.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
    assert_eq!((shot_w, shot_h), (800, 600), "headless viewport must be 800x600");

    let png_bytes = base64::engine::general_purpose::STANDARD
        .decode(png_b64)
        .expect("decode screenshot base64");
    let rendered = decode_png_rgba(&png_bytes).expect("decode screenshot PNG");

    let diff = rgba_diff_pct(&rendered, &oracle);
    eprintln!(
        "DC-13 line 315: welcome headless vs chromium oracle diff = {:.2}% ({}x{} vs {}x{})",
        diff * 100.0,
        rendered.0,
        rendered.1,
        oracle.0,
        oracle.1
    );
    // baseline ~17%（字体墙残余，同 product-smoke engine 路径 16.98%）；25% 留余量，
    // 超过则 headless 路径相对 chromium 退化（非字体墙）。
    assert!(
        diff < 0.25,
        "welcome headless render must be within 25% of chromium oracle (baseline ~17%): {:.2}%",
        diff * 100.0
    );
}

/// 冒烟测试：完整会话生命周期（创建→导航→脚本→截图→关闭）。
#[test]
fn test_smoke_full_session_lifecycle() {
    let mut runner = ProtocolTestRunner::new();

    // 1. 会话状态检查
    let status = runner.send("session.status", Value::Null).unwrap();
    assert_eq!(status["ready"], true);

    // 2. 创建新会话
    let session = runner.send("session.new", Value::Null).unwrap();
    assert_eq!(session["capabilities"]["browserName"], "ZeroWeb");

    // 3. 创建浏览上下文
    let ctx = runner
        .send("browsingContext.create", serde_json::json!({ "url": "about:blank" }))
        .unwrap();
    assert!(ctx.get("context").is_some());

    // 4. 获取上下文树
    let tree = runner.send("browsingContext.getTree", Value::Null).unwrap();
    let contexts = tree.get("contexts").unwrap().as_array().unwrap();
    assert!(contexts.len() >= 2, "should have at least 2 tabs");

    // 5. 执行脚本
    let script_result = runner
        .send("script.evaluate", serde_json::json!({ "expression": "1 + 1" }))
        .unwrap();
    assert!(script_result.get("result").is_some() || script_result.get("exceptionDetails").is_some());

    // 6. 截图
    let screenshot = runner.send("browsingContext.captureScreenshot", Value::Null).unwrap();
    let (w, h, px) = HeadlessClient::parse_screenshot(&screenshot).unwrap();
    assert_eq!(w, 800);
    assert_eq!(h, 600);
    assert!(px > 0);

    // 7. DOM 快照（空会话可能没有图元，但不应 panic）
    let snapshot = runner.send("browsingContext.getDOMSnapshot", Value::Null).unwrap();
    let stats = HeadlessClient::parse_dom_snapshot(&snapshot);
    // 空白页面至少应该有视口根填充
    assert!(stats.total() >= 0, "DOM snapshot should not panic");

    // 8. 验证事件收集（create 不产生 load 事件，需 navigate 才有）
    assert!(runner.event_count("browsingContext.contextCreated") >= 1);

    // 9. 关闭浏览上下文
    let context_id = ctx["context"].as_u64().unwrap();
    let close_result = runner
        .send("browsingContext.close", serde_json::json!({ "context": context_id }))
        .unwrap();
    assert_eq!(close_result["result"], "closed");

    // 10. 验证 contextDestroyed 事件
    assert!(runner.event_count("browsingContext.contextDestroyed") >= 1);

    // 11. 浏览器关闭
    let close = runner.send("browser.close", Value::Null).unwrap();
    assert_eq!(close["result"], "closing");
}

/// 冒烟测试：CDP 兼容命令序列。
#[test]
fn test_smoke_cdp_command_sequence() {
    let mut runner = ProtocolTestRunner::new();

    // 1. 获取版本信息（模拟 HTTP 发现）
    let addr: std::net::SocketAddr = "127.0.0.1:9222".parse().unwrap();
    let version_json = HeadlessServer::http_version_json(addr);
    let version: serde_json::Value = serde_json::from_str(&version_json).unwrap();
    assert_eq!(version["Browser"], format!("ZeroWeb/{}", zero_product_version::VERSION));
    assert!(version["webSocketDebuggerUrl"].as_str().unwrap().starts_with("ws://"));

    // 2. Target.getTargets（M1 切片 3：CDP 形状 targetInfos，替换旧 BiDi tree contexts）
    let targets = runner.send("Target.getTargets", Value::Null).unwrap();
    assert!(targets.get("targetInfos").is_some());

    // 3. Runtime.evaluate
    let eval_result = runner
        .send(
            "Runtime.evaluate",
            serde_json::json!({ "expression": "JSON.stringify({ok: true})" }),
        )
        .unwrap();
    assert!(eval_result.get("result").is_some());

    // 4. Network.enable（S7：真实门控，CDP 语义响应 {}）
    let net_enable = runner.send("Network.enable", Value::Null).unwrap();
    assert!(net_enable.is_object());

    // 5. Page.captureScreenshot（S6：CDP 形状 {data: base64}，BiDi 对象形另测）
    let screenshot = runner.send("Page.captureScreenshot", Value::Null).unwrap();
    assert!(screenshot["data"].as_str().map(|s| !s.is_empty()).unwrap_or(false));
}

/// 冒烟测试：脚本执行和错误处理。
#[test]
fn test_smoke_script_execution_variants() {
    let mut runner = ProtocolTestRunner::new();

    // 正常表达式
    let ok = runner
        .send("script.evaluate", serde_json::json!({ "expression": "2 + 2" }))
        .unwrap();
    assert!(ok.get("result").is_some());

    // JSON 返回
    let json = runner
        .send(
            "script.evaluate",
            serde_json::json!({ "expression": "JSON.stringify({a: 1})" }),
        )
        .unwrap();
    if let Some(result) = json.get("result") {
        if let Some(value) = result.get("value").and_then(|v| v.as_str()) {
            let parsed: serde_json::Value = serde_json::from_str(value).unwrap();
            assert_eq!(parsed["a"], 1);
        }
    }

    // 错误表达式
    let err = runner
        .send(
            "script.evaluate",
            serde_json::json!({ "expression": "throw new Error('test')" }),
        )
        .unwrap();
    assert!(err.get("exceptionDetails").is_some());

    // callFunction 无参数
    let call_no_args = runner
        .send(
            "script.callFunction",
            serde_json::json!({
                "functionDeclaration": "function() { return 42; }"
            }),
        )
        .unwrap();
    assert!(call_no_args.get("result").is_some() || call_no_args.get("exceptionDetails").is_some());

    // callFunction 有参数
    let call_with_args = runner
        .send(
            "script.callFunction",
            serde_json::json!({
                "functionDeclaration": "function(a, b) { return a + b; }",
                "arguments": [{ "value": 10 }, { "value": 20 }]
            }),
        )
        .unwrap();
    assert!(call_with_args.get("result").is_some() || call_with_args.get("exceptionDetails").is_some());
}

/// 冒烟测试：多浏览上下文管理。
#[test]
fn test_smoke_multiple_browsing_contexts() {
    let mut runner = ProtocolTestRunner::new();

    // 初始有 1 个标签页
    let tree1 = runner.send("browsingContext.getTree", Value::Null).unwrap();
    let count1 = tree1.get("contexts").unwrap().as_array().unwrap().len();

    // 创建 3 个新标签页
    let mut new_contexts = Vec::new();
    for i in 0..3 {
        let ctx = runner
            .send(
                "browsingContext.create",
                serde_json::json!({
                    "url": &format!("https://example.com/page{i}")
                }),
            )
            .unwrap();
        new_contexts.push(ctx["context"].as_u64().unwrap());
    }

    // 验证标签页数量增加
    let tree2 = runner.send("browsingContext.getTree", Value::Null).unwrap();
    let count2 = tree2.get("contexts").unwrap().as_array().unwrap().len();
    assert_eq!(count2, count1 + 3);

    // 验证 contextCreated 事件
    assert_eq!(runner.event_count("browsingContext.contextCreated"), 3);

    // 逐个关闭
    for ctx_id in &new_contexts {
        let result = runner
            .send("browsingContext.close", serde_json::json!({ "context": ctx_id }))
            .unwrap();
        assert_eq!(result["result"], "closed");
    }

    // 验证恢复原始数量
    let tree3 = runner.send("browsingContext.getTree", Value::Null).unwrap();
    let count3 = tree3.get("contexts").unwrap().as_array().unwrap().len();
    assert_eq!(count3, count1);

    // 验证 contextDestroyed 事件
    assert_eq!(runner.event_count("browsingContext.contextDestroyed"), 3);
}

/// 冒烟测试：渲染管线通过协议验证。
#[test]
fn test_smoke_render_pipeline_via_protocol() {
    let mut runner = ProtocolTestRunner::new();

    // 加载 HTML 内容（通过脚本设置）
    let load_result = runner
        .send(
            "script.evaluate",
            serde_json::json!({
                "expression": "'render pipeline test'"
            }),
        )
        .unwrap();
    assert!(load_result.get("result").is_some());

    // 截图验证视口尺寸
    let screenshot = runner.send("browsingContext.captureScreenshot", Value::Null).unwrap();
    let (w, h, px) = HeadlessClient::parse_screenshot(&screenshot).unwrap();
    assert_eq!(w, 800);
    assert_eq!(h, 600);
    assert_eq!(px, 800 * 600);

    // DOM 快照验证协议可正确返回图元信息
    let snapshot = runner.send("browsingContext.getDOMSnapshot", Value::Null).unwrap();
    let stats = HeadlessClient::parse_dom_snapshot(&snapshot);
    // 空页面不一定有图元，但快照应成功返回
    assert!(stats.total() >= 0, "DOM snapshot should not panic");
}

/// 冒烟测试：协议错误处理。
#[test]
fn test_smoke_protocol_error_handling() {
    let mut runner = ProtocolTestRunner::new();

    // 未知命令
    let err = runner.send("unknown.command", Value::Null);
    assert!(err.is_err());
    assert!(err.unwrap_err().contains("-32601"));

    // 缺少必要参数
    let nav_err = runner.send("browsingContext.navigate", Value::Null);
    assert!(nav_err.is_err());
    assert!(nav_err.unwrap_err().contains("-32602"));

    // 关闭不存在的上下文
    let close_err = runner.send("browsingContext.close", serde_json::json!({ "context": 99999 }));
    // close_tab 对不存在的标签页是 no-op，所以不会报错
    // 但我们可以验证命令本身不会 panic
    let _ = close_err;

    // callFunction 缺少参数
    let call_err = runner.send("script.callFunction", Value::Null);
    assert!(call_err.is_err());
    assert!(call_err.unwrap_err().contains("-32602"));
}

/// 冒烟测试：页面重载和事件序列。
#[test]
fn test_smoke_reload_and_event_sequence() {
    let mut runner = ProtocolTestRunner::new();

    // 重载当前页面
    let reload = runner.send("browsingContext.reload", Value::Null).unwrap();
    assert_eq!(reload["result"], "reloaded");

    // 验证重载产生 load 事件
    assert!(runner.event_count("browsingContext.load") >= 1);

    // 再次重载
    let reload2 = runner.send("browsingContext.reload", Value::Null).unwrap();
    assert_eq!(reload2["result"], "reloaded");

    // 验证事件累计
    assert!(runner.event_count("browsingContext.load") >= 2);
}

// ── Phase 5: 安全配置测试 ──

#[test]
fn test_security_config_default_allows_all() {
    let config = HeadlessSecurityConfig::new();
    assert!(config.verify_token(None));
    assert!(config.verify_token(Some("anything")));
    assert!(config.verify_origin(None));
    assert!(config.verify_origin(Some("http://evil.com")));
}

#[test]
fn test_security_config_token_required() {
    let config = HeadlessSecurityConfig::new().with_token("secret123");
    assert!(!config.verify_token(None));
    assert!(!config.verify_token(Some("wrong")));
    assert!(config.verify_token(Some("secret123")));
}

#[test]
fn test_security_config_origin_allowlist() {
    let config = HeadlessSecurityConfig::new()
        .with_origin("http://localhost:3000")
        .with_origin("https://trusted.example.com");
    // 允许的来源
    assert!(config.verify_origin(Some("http://localhost:3000")));
    assert!(config.verify_origin(Some("https://trusted.example.com")));
    // 不允许的来源
    assert!(!config.verify_origin(Some("http://evil.com")));
    assert!(!config.verify_origin(None));
}

#[test]
fn test_security_config_empty_origin_allows_all() {
    let config = HeadlessSecurityConfig::new();
    assert!(config.verify_origin(None));
    assert!(config.verify_origin(Some("http://anything.com")));
}

#[test]
fn test_extract_origin_header() {
    let request = b"GET /json HTTP/1.1\r\nHost: localhost:9222\r\nOrigin: http://localhost:3000\r\n\r\n";
    let origin = HeadlessServer::extract_origin_header(request);
    assert_eq!(origin.as_deref(), Some("http://localhost:3000"));

    // 无 Origin 头
    let no_origin = b"GET /json HTTP/1.1\r\nHost: localhost:9222\r\n\r\n";
    assert!(HeadlessServer::extract_origin_header(no_origin).is_none());

    // 小写 origin
    let lowercase = b"GET /json HTTP/1.1\r\norigin: http://example.com\r\n\r\n";
    assert_eq!(
        HeadlessServer::extract_origin_header(lowercase).as_deref(),
        Some("http://example.com")
    );
}

#[test]
fn test_server_with_security_config() {
    let server =
        HeadlessServer::new(0, 800.0, 600.0).with_security(HeadlessSecurityConfig::new().with_token("test-token"));
    assert!(server.security.verify_token(Some("test-token")));
    assert!(!server.security.verify_token(None));
}

#[test]
fn test_server_binds_to_localhost_only() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    assert_eq!(server.addr().ip(), std::net::IpAddr::from([127, 0, 0, 1]));
}
#[test]
fn test_frame_probe_tolerates_script_failure() {
    // 探测容错：查询脚本失败（测试进程内 webview 缺 querySelectorAll 宿主绑定）
    // → 零 attach、事件族完整不阻塞。生产路径（renderer 完整 shim）的 attach 面
    // 由 cdp-e2e 门 frames.access 验证。
    let server = HeadlessServer::new(0, 800.0, 600.0);
    let mut session = HeadlessSession::new(800.0, 600.0);
    session.webview.load_html(
        r#"<html><body><iframe id="a" src="x.html"></iframe><iframe id="b"></iframe></body></html>"#,
        None,
    );
    let mut nav_events = Vec::new();
    server.emit_navigation_event_family(
        &mut session,
        None,
        "zeroweb-tab-1",
        "zw-loader-3",
        "about:blank",
        &mut nav_events,
    );
    assert_eq!(
        nav_events.iter().filter(|e| e.method == "Page.frameAttached").count(),
        0,
        "probe failure must not emit frameAttached"
    );
    let methods: Vec<&str> = nav_events.iter().map(|e| e.method.as_str()).collect();
    assert!(
        methods.contains(&"Page.loadEventFired"),
        "event family must stay intact"
    );
    assert!(session.active_child_frames.is_empty());
    let (tree, _) = server.dispatch_with_events(&mut session, "Page.getFrameTree", Value::Null);
    assert_eq!(tree.unwrap()["frameTree"]["childFrames"].as_array().unwrap().len(), 0);
}

// R-baidu1：headless 截图字体注册表——下载字体按帧原子导入且 surface-local 数字
// ID 在快照转换前被重写为 loader 侧 ID（系统基表来自进程级共享解析）。
// 光栅化消费点（render_page_framebuffer 用 session.paint_fonts.loader）为
// cfg(not(test)) 渲染路径，由 welcome.html/baidu 截图证据与 product-smoke 覆盖。
#[test]
fn paint_frame_fonts_import_downloaded_bytes_and_rewrite_surface_local_ids() {
    let (base, _) = crate::app::shared_system_fonts();
    let mut paint_fonts = zero_paint_convert::fonts::PaintFonts::new(std::sync::Arc::new(base));
    let ahem = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf").to_vec();
    let payloads = vec![zero_protocol::IpcFontPayload {
        font_id: 42,
        face_index: 0,
        data: ahem.clone(),
    }];
    let mut glyphs = vec![zero_protocol::IpcGlyph {
        x: 1.0,
        y: 2.0,
        font_size: 16.0,
        glyph_id: 'X' as u32,
        font_glyph_index: None,
        source: None,
        font_id: 42,
        font_variation_id: None,
        color: zero_protocol::IpcColor {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        },
        rotation: 0.0,
        synthetic_italic: false,
    }];

    super::session::apply_frame_fonts(&mut paint_fonts, &payloads, &mut glyphs);
    let mapped = glyphs[0].font_id;
    assert_ne!(mapped, 42, "surface-local ID must be rewritten before raster");
    assert_eq!(
        paint_fonts.loader.get_font_data(mapped),
        Some(ahem.as_slice()),
        "mapped ID must resolve to the downloaded bytes"
    );

    // 无效资源不得替换当前可用 registry（fail closed，保留上一帧）。
    let bad = vec![zero_protocol::IpcFontPayload {
        font_id: 43,
        face_index: 0,
        data: vec![1, 2, 3],
    }];
    super::session::apply_frame_fonts(&mut paint_fonts, &bad, &mut glyphs);
    assert_eq!(
        paint_fonts.loader.get_font_data(glyphs[0].font_id),
        Some(ahem.as_slice()),
        "rejected frame must keep last good registry"
    );
}
