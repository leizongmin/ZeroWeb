//! HTTP 发现端点 — /json/version、/json、/json/list（CDP 风格浏览器发现）。

use std::net::SocketAddr;

use super::HeadlessServer;
use super::devtools_serve;
use super::session::HeadlessSession;

/// 归一化发现路径 — 容忍尾斜杠（Playwright connectOverCDP 实际请求
/// `/json/version/`，精确匹配会 404 致首连失败）。
pub(super) fn normalize_discovery_path(path: &str) -> &str {
    path.trim_end_matches('/')
}

/// 从 HTTP/WS 升级请求字节中提取请求行路径（不含 query；解析不出返回 `None`）。
/// WS 升级不走 `handle_http_discovery`（那里只认 GET 非升级），路由判定需要原始路径。
pub(super) fn extract_request_path(data: &[u8]) -> Option<String> {
    let s = String::from_utf8_lossy(data);
    let raw = s.lines().next()?.split_whitespace().nth(1)?.to_string();
    let path = raw.split(['?', '#']).next().unwrap_or("");
    Some(path.to_string())
}

/// DevTools frontend 每标签页入口 URL（bundle 已配置时）。
///
/// ws 参数指向 per-page 路径（Chrome 同款），frontend 据此建立页面直连 socket。
pub(super) fn per_tab_frontend_url(addr: SocketAddr, target_id: &str) -> String {
    format!(
        "{}/inspector.html?ws={addr}{}{target_id}",
        devtools_serve::SERVE_PREFIX,
        devtools_serve::PAGE_WS_PREFIX
    )
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

        // devtools goal M0-P2：`/devtools/...` 前缀 → bundle 静态 serve（含 query 串，
        // frontend 入口 URL 携带 `?ws=` 参数）。
        if raw_path.starts_with(devtools_serve::SERVE_PREFIX) {
            devtools_serve::handle_request(self.devtools_frontend_dir.as_deref(), stream, &raw_path);
            return;
        }

        let (status, content_type, body) = match path {
            "/json/version" => ("200 OK", "application/json", Self::http_version_json(addr)),
            "/json" | "/json/list" => (
                "200 OK",
                "application/json",
                http_targets_json(session, addr, self.devtools_serve_enabled()),
            ),
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
/// `devtools_serve_enabled` = bundle 已配置：devtoolsFrontendUrl 指向本地 serve 的
/// frontend 并携带 per-page ws 路径；未配置时维持 `devtools://` 占位形态。
pub(super) fn http_targets_json(session: &HeadlessSession, addr: SocketAddr, devtools_serve_enabled: bool) -> String {
    let targets: Vec<serde_json::Value> = session
        .shell
        .tabs()
        .map(|tab| {
            let id = format!("zeroweb-tab-{}", tab.id().0);
            let devtools_frontend_url = if devtools_serve_enabled {
                per_tab_frontend_url(addr, &id)
            } else {
                format!("devtools://devtools/bundled/inspector.html?ws={addr}")
            };
            serde_json::json!({
                "description": "",
                "devtoolsFrontendUrl": devtools_frontend_url,
                "id": id,
                "title": tab.title().unwrap_or("ZeroWeb"),
                "type": "page",
                "url": tab.url().unwrap_or("about:blank"),
                "webSocketDebuggerUrl": format!("ws://{addr}"),
            })
        })
        .collect();
    serde_json::to_string(&targets).unwrap_or_else(|_| "[]".to_string())
}
