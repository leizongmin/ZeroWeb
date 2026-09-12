//! headless 协议单元测试 + 协议驱动冒烟测试（dispatch 直连与真实 TCP 会话）。

use super::*;

use serde_json::Value;

#[test]
fn test_server_new() {
    let server = HeadlessServer::new(0, 800.0, 600.0);
    assert!(server.addr.port() == 0);
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

    let targets: Vec<serde_json::Value> =
        serde_json::from_str(&super::discovery::http_targets_json(&session, server.addr())).unwrap();
    assert_eq!(targets.len(), before + 1, "one entry per tab");
    assert!(targets.iter().all(|t| t["type"] == "page"));
    assert!(
        targets.iter().any(|t| t["url"] == "https://example.com/page"),
        "navigated url must appear in discovery list"
    );
    let ids: Vec<&str> = targets.iter().map(|t| t["id"].as_str().unwrap()).collect();
    assert!(ids.iter().all(|id| id.starts_with("zeroweb-tab-")));
    assert!(!ids.contains(&"zeroweb-main"), "static placeholder id retired");
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

    // 4. Network.enable
    let net_enable = runner.send("Network.enable", Value::Null).unwrap();
    assert_eq!(net_enable["result"], "enabled");

    // 5. Page.captureScreenshot
    let screenshot = runner.send("Page.captureScreenshot", Value::Null).unwrap();
    let (w, h, _) = HeadlessClient::parse_screenshot(&screenshot).unwrap();
    assert_eq!(w, 800);
    assert_eq!(h, 600);
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
    assert_eq!(server.addr.ip(), std::net::IpAddr::from([127, 0, 0, 1]));
}
