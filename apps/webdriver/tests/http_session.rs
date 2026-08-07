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

fn http_request(port: u16, method: &str, path: &str, body: Option<&str>) -> (u16, String) {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
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
