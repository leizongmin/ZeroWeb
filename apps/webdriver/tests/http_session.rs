//! WebDriver M0 集成测试：spawn zero-webdriver 服务，经 HTTP 协议
//! 验证 New Session / Navigate / Get Title / Delete Session 全链路。

use std::io::{Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};

fn free_port() -> u16 {
    // 简单探测一个空闲端口
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    listener.local_addr().expect("addr").port()
}

/// RAII 包装：所有退出路径（含断言 panic）都 kill + wait 子进程。
struct DriverProcess(Child);

impl Drop for DriverProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn spawn_driver() -> (DriverProcess, u16) {
    let port = free_port();
    // lint 不追踪 DriverProcess 的 Drop（kill+wait）——RAII 保证所有路径收尾
    #[allow(clippy::zombie_processes)]
    let child = Command::new(env!("CARGO_BIN_EXE_zero-webdriver"))
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn zero-webdriver");
    // 等待服务就绪
    for _ in 0..50 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return (DriverProcess(child), port);
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    panic!("zero-webdriver 未就绪");
}

/// 本地测试 HTTP 服务器：固定返回带标题 + 按钮（onclick 改标题）的 HTML 页。
fn spawn_test_page_server_with_button() -> (std::thread::JoinHandle<()>, u16) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let handle = std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = "<html><head><title>WebDriver Test</title></head><body>\
                        <button id=\"btn\" onclick=\"document.title='Clicked!'\">Click me</button>\
                        </body></html>";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    (handle, port)
}

#[test]
fn webdriver_element_interaction() {
    let (_driver, port) = spawn_driver();
    let (_page_server, page_port) = spawn_test_page_server_with_button();

    // New Session
    let (status, body) = http_request(port, "POST", "/session", Some("{}"));
    assert_eq!(status, 200);
    let v: serde_json::Value = serde_json::from_str(&body).expect("json");
    let session_id = v["value"]["sessionId"].as_str().expect("sessionId").to_string();

    // Navigate
    let url = format!("http://127.0.0.1:{page_port}/");
    let (status, _) = http_request(
        port,
        "POST",
        &format!("/session/{session_id}/url"),
        Some(&serde_json::json!({ "url": url }).to_string()),
    );
    assert_eq!(status, 200);

    // Find Element（css selector）
    let (status, body) = http_request(
        port,
        "POST",
        &format!("/session/{session_id}/element"),
        Some(&serde_json::json!({ "using": "css selector", "value": "#btn" }).to_string()),
    );
    assert_eq!(status, 200, "Find Element 应 200: {body}");
    let v: serde_json::Value = serde_json::from_str(&body).expect("json");
    let element_key = "element-6066-11e4-a52e-4f735466cecf";
    let reference = v["value"][element_key].as_str().expect("element reference").to_string();

    // 未找到元素 → no such element
    let (status, _) = http_request(
        port,
        "POST",
        &format!("/session/{session_id}/element"),
        Some(&serde_json::json!({ "using": "css selector", "value": "#missing" }).to_string()),
    );
    assert_eq!(status, 404, "不存在的元素应 404");

    // Execute Script（JS 沙箱语义——如实断言：表达式求值可用；
    // 页面 DOM 操作为 M2 renderer 桥接能力）
    let (status, body) = http_request(
        port,
        "POST",
        &format!("/session/{session_id}/execute/sync"),
        Some(&serde_json::json!({ "script": "1 + 1", "args": [] }).to_string()),
    );
    assert_eq!(status, 200);
    let v: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(v["value"], "2");

    // Element Click（M1：存在性验证；onclick 事件注入为 M2 引擎级能力）
    let (status, body) = http_request(
        port,
        "POST",
        &format!("/session/{session_id}/element/{reference}/click"),
        None,
    );
    assert_eq!(status, 200, "Element Click 应 200: {body}");

    // Click 不存在的引用 → no such element
    let (status, _) = http_request(
        port,
        "POST",
        &format!("/session/{session_id}/element/not-there/click"),
        None,
    );
    assert_eq!(status, 404, "不存在的元素引用应 404");
}

/// 本地测试 HTTP 服务器：固定返回带标题的 HTML 页。
fn spawn_test_page_server() -> (std::thread::JoinHandle<()>, u16) {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    let handle = std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { break };
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            let body = "<html><head><title>WebDriver Test</title></head><body>hi</body></html>";
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(resp.as_bytes());
            let _ = stream.flush();
        }
    });
    (handle, port)
}

/// 连接 zero-webdriver，重试以桥接并发负载下子进程首次 accept 滞后于 TCP 握手就绪的窗口。
///
/// `spawn_driver` 的 readiness 仅检测 TCP 握手（OS backlog 接受即返），但子进程应用层 accept
/// 可能稍后才就绪；全量 `cargo test --workspace` 并发下首条 `http_request` 偶发 Connection refused。
/// 镜像 net client R3086 整请求重试的 deflake 模式（test-infra，非驱动行为变更）。
fn connect_with_retry(port: u16) -> TcpStream {
    for attempt in 0..30u32 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(s) => return s,
            Err(_) if attempt < 29 => std::thread::sleep(std::time::Duration::from_millis(100)),
            Err(e) => panic!("connect to zero-webdriver:{port} failed after retries: {e}"),
        }
    }
    unreachable!()
}

fn http_request(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    let mut stream = connect_with_retry(port);
    let body = body.unwrap_or("");
    let req = format!(
        "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(req.as_bytes()).expect("write");
    stream.flush().expect("flush");
    let mut resp = String::new();
    stream.read_to_string(&mut resp).expect("read");
    let status: u16 = resp
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let body_start = resp.find("\r\n\r\n").map(|i| i + 4).unwrap_or(resp.len());
    (status, resp[body_start..].to_string())
}

#[test]
fn webdriver_session_lifecycle() {
    let (_driver, port) = spawn_driver();

    // 1. New Session
    let (status, body) = http_request(port, "POST", "/session", Some("{}"));
    assert_eq!(status, 200, "New Session 应 200: {body}");
    let v: serde_json::Value = serde_json::from_str(&body).expect("json");
    let session_id = v["value"]["sessionId"].as_str().expect("sessionId").to_string();
    assert!(!session_id.is_empty());
    assert_eq!(v["value"]["capabilities"]["browserName"], "zero-browser");
    assert_eq!(
        v["value"]["capabilities"]["browserVersion"],
        zero_product_version::VERSION
    );

    // 2. Navigate To（本地测试页服务器）
    let (_page_server, page_port) = spawn_test_page_server();
    let url = format!("http://127.0.0.1:{page_port}/");
    let (status, body) = http_request(
        port,
        "POST",
        &format!("/session/{session_id}/url"),
        Some(&serde_json::json!({ "url": url }).to_string()),
    );
    assert_eq!(status, 200, "Navigate 应 200: {body}");

    // 3. Get Title
    let (status, body) = http_request(port, "GET", &format!("/session/{session_id}/title"), None);
    assert_eq!(status, 200, "Get Title 应 200: {body}");
    let v: serde_json::Value = serde_json::from_str(&body).expect("json");
    assert_eq!(v["value"], "WebDriver Test", "标题应来自页面: {body}");

    // 4. Delete Session
    let (status, body) = http_request(port, "DELETE", &format!("/session/{session_id}"), None);
    assert_eq!(status, 200, "Delete Session 应 200: {body}");

    // 5. 删除后访问应 no such session
    let (status, _) = http_request(port, "GET", &format!("/session/{session_id}/title"), None);
    assert_eq!(status, 404, "删除后应 404");

    // 6. 未知命令
    let (status, _) = http_request(port, "GET", "/bogus", None);
    assert_eq!(status, 404, "未知命令应 404");

    // DriverProcess Drop 自动 kill + wait
}
