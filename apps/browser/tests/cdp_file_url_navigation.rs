//! CDP `Page.navigate` file:// 生产链钉（renderer 信任边界 ① 验收面）。
//!
//! 缺陷史：headless 代理 fetch（`headless::session::proxy_fetch`）缺 file:// 直读臂
//! （GUI 侧 `fetch_proxy::enqueue` 一直有），文档与子资源 file:// 全部落 reqwest
//! builder error 错误页——headless CDP 导航本地文件恒失败。修复后钉两层：
//!
//! 1. `file_url_page_loads_headless`——默认环境（renderer landlock 关，当前默认）
//!    file:// 主文档 + 静态模块导入链 + 样式子资源全链可加载（错误页判别式）。
//! 2. `file_url_page_loads_with_renderer_landlock_enabled`——`ZW_RENDERER_LANDLOCK=1`
//!    同链照常加载：landlock 允许清单不含任意路径读，而主文档/子资源/静态模块
//!    依赖全部经 browser fetch_proxy（IPC）直读，「打开本地文件」产品流在默认
//!    沙箱下须保持工作（③A-renderer landlock 默认值翻开的验收前提）。
//!
//! 已知残余（记 goal 待决，此处不钉）：模块体动态 import 的 file:// 面经 renderer
//! worker 本地直读，landlock 默认开后该面被拒——取舍见
//! `docs/goal/renderer-trust-boundary/master.md` 缺口表。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tungstenite::Message;

/// ①a CDP 强制 token 面的测试常量：spawn 显式传入、连接侧 query 携带。
const TEST_CDP_TOKEN: &str = "zw-integ-test-cdp-token";

struct BrowserProcess(Child);

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// file:// fixture：index.html + main.js（静态 import dep.js 并落 window 标记）+ dep.js
/// + style.css（子资源面）。返回 (目录, index URL)。
fn write_fixture(tag: &str) -> (PathBuf, String) {
    let dir = std::env::temp_dir().join(format!("zw-file-nav-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("fixture dir");
    std::fs::write(
        dir.join("index.html"),
        "<!DOCTYPE html><html><head><title>zw-file-nav</title>\
        <link rel=\"stylesheet\" href=\"./style.css\"></head><body>\
        <script type=\"module\" src=\"./main.js\"></script></body></html>",
    )
    .expect("write index.html");
    std::fs::write(
        dir.join("main.js"),
        "import { v } from \"./dep.js\";\nwindow.__modOk = String(v);\n",
    )
    .expect("write main.js");
    std::fs::write(dir.join("dep.js"), "export const v = \"dep-42\";\n").expect("write dep.js");
    std::fs::write(dir.join("style.css"), "body { color: #111; }\n").expect("write style.css");
    let url = format!("file://{}/index.html", dir.to_string_lossy());
    (dir, url)
}

/// 启动 zero-browser --headless 并等 CDP 发现端点就绪（同 cdp_navigate_evaluate 配方）。
fn spawn_browser(vars: &[(&str, &str)]) -> (u16, BrowserProcess) {
    let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
    let port = probe.local_addr().expect("addr").port();
    drop(probe);

    let storage = std::env::temp_dir().join(format!("zw-file-nav-profile-{}", port));
    std::fs::create_dir_all(&storage).expect("storage dir");
    let browser_bin = env!("CARGO_BIN_EXE_zero-browser");
    let renderer = std::path::Path::new(browser_bin)
        .parent()
        .expect("browser bin dir")
        .join("zero-renderer");

    let mut cmd = Command::new(browser_bin);
    cmd.arg("--headless")
        .arg(format!("--remote-debugging-port={port}"))
        .arg(format!("--remote-debugging-token={TEST_CDP_TOKEN}"))
        .env("ZERO_STORAGE_DIR", &storage)
        .env("ZERO_CONFIG_DIR", &storage)
        .env("ZERO_RENDERER_PATH", &renderer);
    for (key, value) in vars {
        cmd.env(key, value);
    }
    let child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn zero-browser");

    let deadline = Instant::now() + Duration::from_secs(30);
    let mut ready = false;
    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
            let _ = stream.write_all(b"GET /json/version HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n");
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let mut buf = String::new();
            if stream.read_to_string(&mut buf).is_ok() && buf.contains("200 OK") {
                ready = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert!(ready, "zero-browser CDP endpoint not ready on port {port}");
    (port, BrowserProcess(child))
}

/// 最小 CDP 客户端（同 cdp_navigate_evaluate 形态）。
struct CdpClient {
    ws: tungstenite::WebSocket<TcpStream>,
    next_id: u64,
    pending_events: Vec<(String, Value)>,
}

impl CdpClient {
    fn connect(port: u16) -> Self {
        let stream = TcpStream::connect(("127.0.0.1", port)).expect("tcp connect");
        stream
            .set_read_timeout(Some(Duration::from_millis(50)))
            .expect("read timeout");
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .expect("write timeout");
        let (ws, _resp) = tungstenite::client(format!("ws://127.0.0.1:{port}/?auth_token={TEST_CDP_TOKEN}"), stream)
            .expect("ws handshake");
        Self {
            ws,
            next_id: 0,
            pending_events: Vec::new(),
        }
    }

    fn call(&mut self, method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        let text = json!({ "id": id, "method": method, "params": params }).to_string();
        self.ws.send(Message::Text(text.into())).expect("ws send");
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() >= deadline {
                return Err(format!("timeout waiting for response of {method}"));
            }
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let msg: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    if msg.get("id").and_then(Value::as_u64) == Some(id) {
                        return Ok(msg);
                    }
                    if let Some(m) = msg.get("method").and_then(Value::as_str) {
                        self.pending_events
                            .push((m.to_string(), msg.get("params").cloned().unwrap_or(Value::Null)));
                    }
                }
                Ok(Message::Ping(data)) => {
                    self.ws.send(Message::Pong(data)).expect("pong");
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(format!("ws read: {e}")),
            }
        }
    }

    fn wait_event(&mut self, method: &str, timeout: Duration) -> Result<Value, String> {
        if let Some(idx) = self.pending_events.iter().position(|(m, _)| m == method) {
            return Ok(self.pending_events.remove(idx).1);
        }
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() >= deadline {
                return Err(format!("timeout waiting for event {method}"));
            }
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let msg: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    if let Some(m) = msg.get("method").and_then(Value::as_str) {
                        let params = msg.get("params").cloned().unwrap_or(Value::Null);
                        if m == method {
                            return Ok(params);
                        }
                        self.pending_events.push((m.to_string(), params));
                    }
                }
                Ok(Message::Ping(data)) => {
                    self.ws.send(Message::Pong(data)).expect("pong");
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => return Err(format!("ws read: {e}")),
            }
        }
    }

    /// evaluate 并取返回值（returnByValue）。
    fn evaluate(&mut self, expression: &str) -> Result<Value, String> {
        let response = self.call(
            "Runtime.evaluate",
            json!({ "expression": expression, "returnByValue": true }),
            Duration::from_secs(10),
        )?;
        let value = response
            .pointer("/result/result/value")
            .cloned()
            .ok_or_else(|| format!("evaluate missing value: {response}"))?;
        Ok(value)
    }
}

/// 共享导航链：navigate → load 事件 → 断言 title / 静态模块链 / 非错误页。
fn assert_file_chain_loads(extra_env: &[(&str, &str)], tag: &str) {
    let (_dir, url) = write_fixture(tag);
    let (_port, _browser) = spawn_browser(extra_env);
    let mut cdp = CdpClient::connect(_port);

    cdp.call("Page.enable", json!({}), Duration::from_secs(5))
        .expect("Page.enable");
    let navigate = cdp
        .call("Page.navigate", json!({ "url": url }), Duration::from_secs(15))
        .expect("Page.navigate response");
    assert!(navigate.get("error").is_none(), "navigate error: {navigate}");
    cdp.wait_event("Page.loadEventFired", Duration::from_secs(15))
        .expect("load event");

    // 模块链完成面（静态 import → worker 注册表）可能有调度延迟：轮询至多 5s。
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut mod_ok = None;
    while Instant::now() < deadline {
        if let Ok(v) = cdp.evaluate("window.__modOk")
            && v.as_str() == Some("dep-42")
        {
            mod_ok = Some(v);
            break;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    assert_eq!(
        mod_ok.as_ref().and_then(Value::as_str),
        Some("dep-42"),
        "静态模块导入链未执行（window.__modOk={mod_ok:?}）"
    );

    let title = cdp.evaluate("document.title").expect("evaluate title");
    assert_eq!(title, json!("zw-file-nav"), "file:// 主文档标题缺失");
    let href = cdp.evaluate("location.href").expect("evaluate href");
    let href = href.as_str().unwrap_or_default().to_string();
    assert!(
        !href.starts_with("error://"),
        "file:// 导航落错误页（headless 代理缺 file:// 臂的缺陷面回潮?）: {href}"
    );
}

#[test]
fn file_url_page_loads_headless() {
    assert_file_chain_loads(&[], "default");
}

/// landlock 默认值翻转（③A-renderer 收尾）后的验收前提：file:// 产品流在允许清单
/// 沙箱下保持工作。当前 landlock 为 opt-in，本用例显式开启钉住该前提。
#[test]
fn file_url_page_loads_with_renderer_landlock_enabled() {
    assert_file_chain_loads(&[("ZW_RENDERER_LANDLOCK", "1")], "landlock");
}
