//! ZeroWeb WebDriver 服务（#1 调研建议 M0 切片）— W3C WebDriver 协议最小实现。
//!
//! M0 范围（wdspec 测试的最小可行子集）：
//!   POST   /session                    New Session
//!   POST   /session/{id}/url           Navigate To
//!   GET    /session/{id}/title         Get Title
//!   DELETE /session/{id}               Delete Session
//!
//! 架构：进程内 webview（无窗口渲染，与 wpt-runner 同路径），HTTP 服务为
//! 零依赖手写最小 HTTP/1.1（M0 无并发要求）。后续切片可扩展元素定位/
//! 交互端点、独立进程桥接（对照 Ladybird WebDriver 全端点实现，调研报告 §3.3）。
//!
//! 协议参考：https://w3c.github.io/webdriver/#protocol
//! 用法：zero-webdriver --port 9515

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use zero_webview::{WebView, WebViewConfig};

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
    let body = data[header_end..].to_vec();
    let mut body = body;
    while body.len() < content_length {
        let n = stream.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        body.extend_from_slice(&buf[..n]);
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

// ── 会话管理 ──

struct Session {
    webview: WebView,
}

struct Driver {
    sessions: HashMap<String, Session>,
    next_session_id: u64,
}

impl Driver {
    fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            next_session_id: 1,
        }
    }

    fn create_session(&mut self) -> String {
        let id = format!("{:016x}", self.next_session_id);
        self.next_session_id += 1;
        let config = WebViewConfig {
            width: 800,
            height: 600,
            url: Some("about:blank".to_string()),
            ..WebViewConfig::default()
        };
        let webview = WebView::new(config);
        self.sessions.insert(id.clone(), Session { webview });
        id
    }

    fn delete_session(&mut self, id: &str) -> bool {
        self.sessions.remove(id).is_some()
    }

    fn navigate(&mut self, id: &str, url: &str) -> Result<(), String> {
        let session = self.sessions.get_mut(id).ok_or_else(|| "no such session".to_string())?;
        session.webview.fetch_url(url).map(|_| ()).map_err(|e| e.to_string())
    }

    fn title(&self, id: &str) -> Result<String, String> {
        let session = self.sessions.get(id).ok_or_else(|| "no such session".to_string())?;
        // 进程内模式无渲染进程的 page_scripts，title 不自动设置——
        // 从缓存的 HTML 提取 <title>（M0 简化：大小写不敏感、跨行）
        let html = session.webview.html_content();
        let lower = html.to_lowercase();
        let title = lower
            .find("<title>")
            .and_then(|start| {
                let s = start + "<title>".len();
                lower[s..].find("</title>").map(|e| html[s..s + e].trim().to_string())
            })
            .unwrap_or_default();
        Ok(title)
    }
}

// ── M1：元素定位与交互（Rust 侧查询真实页面 DOM）──
//
// 能力边界（2026-08-07 实测确定）：
//   - execute_script_with_dom 的 DOM 是 JS 沙箱内的**虚拟 DOM**（dom_bridge
//     polyfill），与渲染的页面 DOM（zero-dom）无关——JS 查询页面元素不可行
//   - 因此 Find Element 用 Rust 侧查询：从 cached_html 重新 parse 页面 DOM
//     （zero_dom::query_selector）——与渲染一致（reload 后 cached_html 同步更新）
//   - Element Click 当前为**存在性验证**（事件注入需引擎级 dispatch，M2 经
//     renderer 桥接实现——对照 Ladybird 全端点，调研报告 §3.3）
//   - Execute Script 作用于 JS 沙箱（虚拟 DOM 语义），如实保留

/// WebDriver 元素引用键（规范固定值）。
const ELEMENT_KEY: &str = "element-6066-11e4-a52e-4f735466cecf";

/// 元素引用格式：`selector`（M1：query_selector 返回首个匹配，index 恒 0）。
fn encode_element_ref(selector: &str) -> String {
    selector.to_string()
}

/// 在页面的真实 DOM（cached_html 重新 parse）中查询元素。
fn find_element_in_page(session: &Session, selector: &str) -> Result<bool, String> {
    let html = session.webview.html_content();
    let doc = zero_dom::parse_html(html);
    let root = doc.root();
    Ok(doc.query_selector(root, selector).is_some())
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
        ("POST", ["session"]) => {
            let id = driver.create_session();
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
                Err(e) => error_response(stream, 404, "no such session", &e),
            }
        }
        // GET /session/{id}/title — Get Title
        ("GET", ["session", id, "title"]) => match driver.title(id) {
            Ok(title) => json_response(stream, serde_json::json!({ "value": title })),
            Err(e) => error_response(stream, 404, "no such session", &e),
        },
        // POST /session/{id}/element — Find Element（M1，css selector，Rust 侧查询真实 DOM）
        ("POST", ["session", id, "element"]) => {
            let selector = match serde_json::from_slice::<serde_json::Value>(&req.body) {
                Ok(v) => v.get("value").and_then(|u| u.as_str()).unwrap_or("").to_string(),
                Err(_) => String::new(),
            };
            if selector.is_empty() {
                error_response(stream, 400, "invalid argument", "missing value (css selector)");
                return;
            }
            match driver.sessions.get(*id) {
                Some(session) => match find_element_in_page(session, &selector) {
                    Ok(true) => {
                        let reference = encode_element_ref(&selector);
                        json_response(stream, serde_json::json!({ "value": { ELEMENT_KEY: reference } }));
                    }
                    Ok(false) => error_response(stream, 404, "no such element", "element not found"),
                    Err(e) => error_response(stream, 500, "unknown error", &e),
                },
                None => error_response(stream, 404, "no such session", "session not found"),
            }
        }
        // POST /session/{id}/element/{ref}/click — Element Click
        //（M2：真实 DOM 事件派发——run_page_scripts + __zw_dispatch_event，
        // 触发页面 addEventListener/onclick 监听器；先验证元素存在）
        ("POST", ["session", id, "element", reference, "click"]) => match driver.sessions.get_mut(*id) {
            Some(session) => match find_element_in_page(session, reference) {
                Ok(true) => match session.webview.dispatch_event(reference, "click") {
                    Ok(()) => json_response(stream, serde_json::json!({ "value": null })),
                    Err(e) => error_response(stream, 500, "unknown error", &e.to_string()),
                },
                Ok(false) => error_response(stream, 404, "no such element", "element not found"),
                Err(e) => error_response(stream, 500, "unknown error", &e),
            },
            None => error_response(stream, 404, "no such session", "session not found"),
        },
        // POST /session/{id}/execute/sync — Execute Script（M1）
        ("POST", ["session", id, "execute", "sync"]) => {
            let script = match serde_json::from_slice::<serde_json::Value>(&req.body) {
                Ok(v) => v.get("script").and_then(|u| u.as_str()).unwrap_or("").to_string(),
                Err(_) => String::new(),
            };
            if script.is_empty() {
                error_response(stream, 400, "invalid argument", "missing script");
                return;
            }
            match driver.sessions.get_mut(*id) {
                Some(session) => match session.webview.execute_script_with_dom(&script) {
                    Ok(result) => json_response(stream, serde_json::json!({ "value": result })),
                    Err(e) => error_response(stream, 500, "javascript error", &e.to_string()),
                },
                None => error_response(stream, 404, "no such session", "session not found"),
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
    tracing::info!("zero-webdriver 监听 127.0.0.1:{port}（M0 切片：New Session/Navigate/Title/Delete）");

    let mut driver = Driver::new();
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
