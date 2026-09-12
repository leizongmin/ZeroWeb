//! HTTP 发现端点 — /json/version、/json、/json/list（CDP 风格浏览器发现）。

use std::net::SocketAddr;

use super::HeadlessServer;
use super::session::HeadlessSession;

/// 归一化发现路径 — 容忍尾斜杠（Playwright connectOverCDP 实际请求
/// `/json/version/`，精确匹配会 404 致首连失败）。
pub(super) fn normalize_discovery_path(path: &str) -> &str {
    path.trim_end_matches('/')
}

impl HeadlessServer {
    /// 判断是否为普通 HTTP GET 请求（非 WebSocket 升级）。
    pub(super) fn is_http_get_request(data: &[u8]) -> bool {
        let s = String::from_utf8_lossy(data);
        s.starts_with("GET ") && !s.contains("Upgrade: websocket")
    }

    /// 从 HTTP 请求头中提取 Origin 值。
    pub(super) fn extract_origin_header(data: &[u8]) -> Option<String> {
        let s = String::from_utf8_lossy(data);
        for line in s.lines() {
            if let Some(value) = line.strip_prefix("Origin: ") {
                return Some(value.trim().to_string());
            }
            if let Some(value) = line.strip_prefix("origin: ") {
                return Some(value.trim().to_string());
            }
        }
        None
    }

    /// 处理 HTTP 发现请求（CDP 风格的 /json 端点）。
    pub(super) fn handle_http_discovery(&self, stream: &std::net::TcpStream, session: &HeadlessSession) {
        use std::io::{Read, Write};

        let addr = self.addr;
        let mut read_buf = [0u8; 4096];
        let raw_path = if let Ok(mut readable) = stream.try_clone() {
            let n = readable.read(&mut read_buf).unwrap_or(0);
            let request = String::from_utf8_lossy(&read_buf[..n]);
            request
                .lines()
                .next()
                .unwrap_or("")
                .split_whitespace()
                .nth(1)
                .unwrap_or("/")
                .to_string()
        } else {
            "/".to_string()
        };
        let path = normalize_discovery_path(&raw_path);

        let (status, content_type, body) = match path {
            "/json/version" => ("200 OK", "application/json", Self::http_version_json(addr)),
            "/json" | "/json/list" => ("200 OK", "application/json", http_targets_json(session, addr)),
            _ => ("404 Not Found", "text/plain", "Not Found".to_string()),
        };

        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );

        if let Ok(mut writable) = stream.try_clone() {
            let _ = writable.write_all(response.as_bytes());
            let _ = writable.flush();
        }
    }

    /// HTTP GET /json/version — CDP 风格的浏览器发现端点。
    #[allow(dead_code)]
    pub fn http_version_json(addr: SocketAddr) -> String {
        serde_json::json!({
            "Browser": format!("ZeroWeb/{}", zero_product_version::VERSION),
            "Protocol-Version": "1.3",
            "User-Agent": zero_net::HttpClient::default_user_agent(),
            "V8-Version": "12.0",
            "WebKit-Version": "0.1",
            "webSocketDebuggerUrl": format!("ws://{addr}"),
        })
        .to_string()
    }
}

/// HTTP GET /json、/json/list — 按真实标签页枚举 page target（CDP shape）。
///
/// url/title 取自 shell 模型（`BrowserShell::tabs()`），id 与 TabId 一一对应
/// （`zeroweb-tab-<n>`），供后续 Target 域把 targetId 映射回标签页。
pub(super) fn http_targets_json(session: &HeadlessSession, addr: SocketAddr) -> String {
    let targets: Vec<serde_json::Value> = session
        .shell
        .tabs()
        .map(|tab| {
            serde_json::json!({
                "description": "",
                "devtoolsFrontendUrl": format!("devtools://devtools/bundled/inspector.html?ws={addr}"),
                "id": format!("zeroweb-tab-{}", tab.id().0),
                "title": tab.title().unwrap_or("ZeroWeb"),
                "type": "page",
                "url": tab.url().unwrap_or("about:blank"),
                "webSocketDebuggerUrl": format!("ws://{addr}"),
            })
        })
        .collect();
    serde_json::to_string(&targets).unwrap_or_else(|_| "[]".to_string())
}
