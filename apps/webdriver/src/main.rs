//! ZeroWeb WebDriver 服务 — W3C WebDriver HTML 交互子集。
//!
//! 支持：
//!   POST   /session                    New Session
//!   POST   /session/{id}/url           Navigate To
//!   GET    /session/{id}/title         Get Title
//!   POST   /session/{id}/element       Find Element
//!   POST   /session/{id}/element/{ref}/click
//!   POST   /session/{id}/element/{ref}/value
//!   GET    /session/{id}/element/active
//!   POST   /session/{id}/execute/sync
//!   DELETE /session/{id}               Delete Session
//!
//! 每个 session 持有独立 `zero-renderer` 子进程；页面操作经 automation IPC
//! 在 live document 上执行。HTTP 服务保持零依赖、单线程和 loopback-only。
//!
//! 协议参考：https://w3c.github.io/webdriver/#protocol
//! 用法：zero-webdriver --port 9515

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

mod session;

use session::{Driver, DriverError, parse_webdriver_keys};

// ── HTTP 最小实现（M0：无并发、单请求-响应）──

struct HttpRequest {
    method: String,
    path: String,
    body: Vec<u8>,
}

fn read_request(stream: &mut TcpStream) -> Option<HttpRequest> {
    let mut buf = [0u8; 8192];
    let mut data = Vec::new();
    // 读到请求头结束（\r\n\r\n）——M0 只读一次（请求体小）
    loop {
        let n = stream.read(&mut buf).ok()?;
        if n == 0 {
            return None;
        }
        data.extend_from_slice(&buf[..n]);
        if data.windows(4).any(|w| w == b"\r\n\r\n") {
            break;
        }
        if data.len() > 1 << 20 {
            return None; // 1MB 上限
        }
    }
    let header_end = data.windows(4).position(|w| w == b"\r\n\r\n")? + 4;
    let head = String::from_utf8_lossy(&data[..header_end]);
    let mut lines = head.lines();
    let request_line = lines.next()?.trim();
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();

    // Content-Length
    let mut content_length = 0usize;
    for line in lines {
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_length = v.trim().parse().unwrap_or(0);
        }
    }
    if content_length > 1 << 20 {
        return None;
    }
    let body = data[header_end..].to_vec();
    let mut body = body;
    while body.len() < content_length {
        let n = stream.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&buf[..n]);
        if body.len() > 1 << 20 {
            return None;
        }
    }
    body.truncate(content_length);
    Some(HttpRequest { method, path, body })
}

fn write_response(stream: &mut TcpStream, status: u16, reason: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Content-Length: {}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, DELETE, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Connection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
    let _ = stream.flush();
}

fn json_response(stream: &mut TcpStream, value: serde_json::Value) {
    let body = serde_json::to_vec(&value).unwrap_or_default();
    write_response(stream, 200, "OK", &body);
}

fn error_response(stream: &mut TcpStream, status: u16, reason: &str, message: &str) {
    let body = serde_json::json!({ "value": { "error": reason.to_lowercase(), "message": message } });
    let body = serde_json::to_vec(&body).unwrap_or_default();
    write_response(stream, status, reason, &body);
}

/// WebDriver 元素引用键（规范固定值）。
const ELEMENT_KEY: &str = "element-6066-11e4-a52e-4f735466cecf";

fn driver_error_response(stream: &mut TcpStream, error: DriverError) {
    let status = match error.code {
        "invalid argument" => 400,
        "no such session" | "no such element" | "stale element reference" => 404,
        _ => 500,
    };
    error_response(stream, status, error.code, &error.message);
}

// ── 端点路由 ──

fn handle_request(driver: &mut Driver, req: &HttpRequest, stream: &mut TcpStream) {
    let path = req.path.trim_end_matches('/');

    // 路径解析：/session 与 /session/{id}/xxx
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    match (req.method.as_str(), segments.as_slice()) {
        ("OPTIONS", _) => {
            write_response(stream, 204, "No Content", b"");
        }
        // POST /session — New Session
        ("POST", ["session"]) => match driver.create_session() {
            Ok(id) => {
                json_response(
                    stream,
                    serde_json::json!({
                        "value": {
                            "sessionId": id,
                            "capabilities": {
                                "browserName": "zero-browser",
                                "browserVersion": zero_product_version::VERSION,
                            }
                        }
                    }),
                );
            }
            Err(error) => driver_error_response(stream, error),
        },
        // POST /session/{id}/url — Navigate To
        ("POST", ["session", id, "url"]) => {
            let url = match serde_json::from_slice::<serde_json::Value>(&req.body) {
                Ok(v) => v.get("url").and_then(|u| u.as_str()).unwrap_or("").to_string(),
                Err(_) => String::new(),
            };
            if url.is_empty() {
                error_response(stream, 400, "invalid argument", "missing url");
                return;
            }
            match driver.navigate(id, &url) {
                Ok(()) => json_response(stream, serde_json::json!({ "value": null })),
                Err(error) => driver_error_response(stream, error),
            }
        }
        // GET /session/{id}/title — Get Title
        ("GET", ["session", id, "title"]) => match driver.title(id) {
            Ok(title) => json_response(stream, serde_json::json!({ "value": title })),
            Err(error) => driver_error_response(stream, error),
        },
        // POST /session/{id}/element — Find Element。
        ("POST", ["session", id, "element"]) => {
            let body = serde_json::from_slice::<serde_json::Value>(&req.body).unwrap_or_default();
            let using = body.get("using").and_then(|value| value.as_str()).unwrap_or("");
            let selector = body.get("value").and_then(|value| value.as_str()).unwrap_or("");
            if using != "css selector" {
                error_response(stream, 400, "invalid argument", "only css selector is supported");
                return;
            }
            match driver.find_element(id, selector.to_string()) {
                Ok(reference) => {
                    json_response(stream, serde_json::json!({ "value": { ELEMENT_KEY: reference } }));
                }
                Err(error) => driver_error_response(stream, error),
            }
        }
        // GET /session/{id}/element/active — Get Active Element。
        ("GET", ["session", id, "element", "active"]) => match driver.active_element(id) {
            Ok(Some(reference)) => {
                json_response(stream, serde_json::json!({ "value": { ELEMENT_KEY: reference } }));
            }
            Ok(None) => json_response(stream, serde_json::json!({ "value": null })),
            Err(error) => driver_error_response(stream, error),
        },
        // POST /session/{id}/element/{ref}/click — Element Click。
        ("POST", ["session", id, "element", reference, "click"]) => match driver.click_element(id, reference) {
            Ok(()) => json_response(stream, serde_json::json!({ "value": null })),
            Err(error) => driver_error_response(stream, error),
        },
        // POST /session/{id}/element/{ref}/value — Element Send Keys。
        ("POST", ["session", id, "element", reference, "value"]) => {
            let body = serde_json::from_slice::<serde_json::Value>(&req.body).unwrap_or_default();
            let text = body
                .get("text")
                .and_then(|value| value.as_str())
                .map(str::to_string)
                .or_else(|| {
                    body.get("value").and_then(|value| {
                        value
                            .as_array()
                            .map(|items| items.iter().filter_map(|item| item.as_str()).collect::<String>())
                    })
                })
                .unwrap_or_default();
            if text.len() > 64 * 1024 {
                error_response(stream, 400, "invalid argument", "send keys payload exceeds 64 KiB");
                return;
            }
            match driver.send_keys(id, reference, parse_webdriver_keys(&text)) {
                Ok(()) => json_response(stream, serde_json::json!({ "value": null })),
                Err(error) => driver_error_response(stream, error),
            }
        }
        // POST /session/{id}/execute/sync — Execute Script in the live page context。
        ("POST", ["session", id, "execute", "sync"]) => {
            let body = serde_json::from_slice::<serde_json::Value>(&req.body).unwrap_or_default();
            let script = body
                .get("script")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            let arguments = body
                .get("args")
                .and_then(|value| value.as_array())
                .cloned()
                .unwrap_or_default();
            if script.is_empty() {
                error_response(stream, 400, "invalid argument", "missing script");
                return;
            }
            if script.len() > 512 * 1024 {
                error_response(stream, 400, "invalid argument", "script exceeds 512 KiB");
                return;
            }
            match driver.execute_script(id, script, arguments) {
                Ok(value) => json_response(stream, serde_json::json!({ "value": value })),
                Err(error) => driver_error_response(stream, error),
            }
        }
        // DELETE /session/{id} — Delete Session
        ("DELETE", ["session", id]) => {
            if driver.delete_session(id) {
                json_response(stream, serde_json::json!({ "value": null }));
            } else {
                error_response(stream, 404, "no such session", "session not found");
            }
        }
        _ => error_response(stream, 404, "unknown command", &format!("{} {}", req.method, req.path)),
    }
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_writer(std::io::stderr)
        .init();

    let port: u16 = std::env::args()
        .position(|a| a == "--port")
        .and_then(|i| std::env::args().nth(i + 1))
        .and_then(|p| p.parse().ok())
        .unwrap_or(9515);

    let listener = TcpListener::bind(("127.0.0.1", port)).unwrap_or_else(|e| {
        eprintln!("zero-webdriver: 绑定端口 {port} 失败: {e}");
        std::process::exit(1);
    });
    tracing::info!("zero-webdriver 监听 127.0.0.1:{port}（live renderer automation）");

    let mut driver = Driver::new().unwrap_or_else(|error| {
        tracing::error!("zero-webdriver 初始化失败: {}", error.message);
        std::process::exit(1);
    });
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Some(req) = read_request(&mut stream) {
                    handle_request(&mut driver, &req, &mut stream);
                }
            }
            Err(e) => tracing::warn!("连接错误: {e}"),
        }
    }
}
