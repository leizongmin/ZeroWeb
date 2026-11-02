//! file:// 直读臂范围策略的进程级回归钉（renderer 信任边界 DC-2，2026-11-02）。
//!
//! 缺陷史：browser 侧 file:// 直读臂（GUI `fetch_proxy::enqueue` 与 headless
//! `proxy_fetch`）对 renderer 的读取请求无 origin/范围校验——http 页面 `<img
//! src=file://>` 可读本地文件、页面 JS 导航 file:// 可渲染本地文件全文（headless
//! 实证在册，见 goal evidence 2026-11-02-file-url-ipc-arm-scope-finding）。
//!
//! 修复后钉三面（锚点 = browser 发起导航登记 + 最近实际服务文档，renderer 不可伪造）：
//! 1. `http_page_cannot_read_local_files`——http 文档下 file:// 子资源拒绝
//!    （正控 http 图片正常加载，排除通道失真）、页面 JS 导航 file:// 拒绝
//!    （location 不落 file、正文无目标文件内容）。
//! 2. `file_page_subtree_confines_subresources_and_allows_file_nav`——file 文档下
//!    同目录子资源放行（锚点已钉 + same-dir-or-below 对齐 Chrome）、目录树外子资源
//!    拒绝（PathBuf 组件级前缀，`../` 越界）；file→file 页面 JS 导航放行
//!    （Document 面「最近文档为 file://」语义，防过约回归）。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use tungstenite::Message;

/// ①a CDP 强制 token 面的测试常量：spawn 显式传入、连接侧 query 携带。
const TEST_CDP_TOKEN: &str = "zw-integ-test-cdp-token";

/// 1×1 透明 PNG（图片通道最小合法载荷，同 cdp_fetch_cache）。
const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49,
    0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00,
    0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// 目标文件内容标记（越权读取判别式：正文出现即读得本地文件）。
const SECRET_MARKER: &str = "SECRET-MARKER-9137";

/// file:// fixture 布局：
/// `site/`（file 文档目录树）+ `outside/`（树外）。返回 (目录, 关键 URL)。
fn write_fixture(tag: &str) -> (PathBuf, String, String, String) {
    let dir = std::env::temp_dir().join(format!("zw-file-scope-{tag}-{}", std::process::id()));
    let site = dir.join("site");
    let outside = dir.join("outside");
    std::fs::create_dir_all(&site).expect("fixture site dir");
    std::fs::create_dir_all(&outside).expect("fixture outside dir");
    // file 文档：同目录 img（放行腿）+ ../outside img（树外拒绝腿）。
    std::fs::write(
        site.join("page.html"),
        "<!DOCTYPE html><html><head><title>zw-file-scope</title></head><body>\
        <img id=\"ok\" src=\"./pixel.png\">\
        <img id=\"leak\" src=\"../outside/secret.png\"></body></html>",
    )
    .expect("write page.html");
    std::fs::write(site.join("pixel.png"), PNG_1X1).expect("write pixel.png");
    std::fs::write(outside.join("secret.png"), PNG_1X1).expect("write secret.png");
    std::fs::write(
        outside.join("secret.html"),
        format!("<!DOCTYPE html><html><body>{SECRET_MARKER}</body></html>"),
    )
    .expect("write secret.html");
    let path = |rel: &str| format!("file://{}", dir.join(rel).to_string_lossy());
    let site_url = path("site/page.html");
    let secret_png = path("outside/secret.png");
    let secret_html = path("outside/secret.html");
    (dir, site_url, secret_png, secret_html)
}

/// 测试站点：线程化 HTTP（同 cdp_fetch_cache 形态）。`/probe` = http 文档
/// （含 file:// img 越权腿 + http 正控 img），`/img` = 正控 PNG。
struct TestSite {
    port: u16,
    shutdown: Arc<AtomicBool>,
}

impl TestSite {
    fn spawn(secret_png_url: &str) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test site");
        let port = listener.local_addr().expect("addr").port();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_thread = shutdown.clone();
        let secret_png = secret_png_url.to_string();
        std::thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            for stream in listener.incoming() {
                if shutdown_thread.load(Ordering::Relaxed) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
                let (status_line, content_type, body): (&str, &str, Vec<u8>) =
                    match path.split('?').next().unwrap_or("/") {
                        "/probe" => (
                            "HTTP/1.1 200 OK",
                            "text/html",
                            format!(
                                "<!DOCTYPE html><html><head><title>zw-scope-http</title></head><body>\
                                <img id=\"local\" src=\"{secret_png}\">\
                                <img id=\"httpimg\" src=\"/img\"></body></html>"
                            )
                            .into_bytes(),
                        ),
                        "/img" => ("HTTP/1.1 200 OK", "image/png", PNG_1X1.to_vec()),
                        _ => ("HTTP/1.1 200 OK", "text/plain", b"not found".to_vec()),
                    };
                let response = format!(
                    "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&body);
                let _ = stream.flush();
            }
        });
        Self { port, shutdown }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }
}

impl Drop for TestSite {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

/// 启动 zero-browser --headless 并等 CDP 发现端点就绪（同 cdp_file_url_navigation 配方）。
/// stderr 落临时文件：策略拒绝的 `tracing::warn`（file fetch blocked headless）
/// 是服务端拒绝面的直接证据，用例断言其出现。
fn spawn_browser() -> (u16, BrowserProcess, PathBuf) {
    let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
    let port = probe.local_addr().expect("addr").port();
    drop(probe);

    let storage = std::env::temp_dir().join(format!("zw-file-scope-profile-{}", port));
    std::fs::create_dir_all(&storage).expect("storage dir");
    let stderr_path = std::env::temp_dir().join(format!("zw-file-scope-stderr-{port}.log"));
    let browser_bin = env!("CARGO_BIN_EXE_zero-browser");
    let renderer = std::path::Path::new(browser_bin)
        .parent()
        .expect("browser bin dir")
        .join("zero-renderer");

    let child = Command::new(browser_bin)
        .arg("--headless")
        .arg(format!("--remote-debugging-port={port}"))
        .arg(format!("--remote-debugging-token={TEST_CDP_TOKEN}"))
        .env("ZERO_STORAGE_DIR", &storage)
        .env("ZERO_CONFIG_DIR", &storage)
        .env("ZERO_RENDERER_PATH", &renderer)
        .stdout(Stdio::null())
        .stderr(std::fs::File::create(&stderr_path).expect("stderr log file"))
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
    (port, BrowserProcess(child), stderr_path)
}

struct BrowserProcess(Child);

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// 最小 CDP 客户端（同 cdp_file_url_navigation 形态）。
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

/// 轮询求值直到 `pred` 成立；命中返回 Some(值)，超时未命中返回 None。
fn poll_until(
    cdp: &mut CdpClient,
    expression: &str,
    timeout: Duration,
    pred: impl Fn(&Value) -> bool,
) -> Option<Value> {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(v) = cdp.evaluate(expression)
            && pred(&v)
        {
            return Some(v);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    None
}

/// img 加载成功判别：轮询至 naturalWidth != 0（图片解码成功，1×1 PNG → 1）。
/// CDP 数值序列化可能落 f64（`1.0`），比较走 as_f64。
fn wait_img_loaded(cdp: &mut CdpClient, id: &str) -> Value {
    let expression = format!("document.getElementById('{id}')?.naturalWidth ?? 'missing'");
    poll_until(cdp, &expression, Duration::from_secs(10), |v| {
        v.as_f64().is_some_and(|w| w > 0.0)
    })
    .unwrap_or_else(|| panic!("img #{id} 未加载（正控通道失真或放行腿被误拒）"))
}

/// img 加载拒绝判别：轮询观察窗口内 naturalWidth 恒 0，窗口结束复测仍 0。
fn assert_img_blocked(cdp: &mut CdpClient, id: &str, what: &str) {
    let expression = format!("document.getElementById('{id}')?.naturalWidth ?? 'missing'");
    let leaked = poll_until(cdp, &expression, Duration::from_secs(3), |v| {
        v.as_f64().is_some_and(|w| w > 0.0)
    });
    assert!(
        leaked.is_none(),
        "{what}: 本地文件被读入（naturalWidth={leaked:?}，范围策略失效回潮）"
    );
    let final_width = cdp.evaluate(&expression).expect("final naturalWidth");
    assert!(
        final_width.as_f64() == Some(0.0),
        "{what}: 观察窗结束后 img 仍不得加载（naturalWidth={final_width}）"
    );
}

/// 页面 JS 导航 file:// 拒绝判别：正文不得出现目标文件内容（内容判别是安全边界；
/// 被拒导航与 Chrome 对齐地停在目标 URL——空错误文档 commit，不含文件内容）。
/// 导航确实发生的证据 = location 落在目标 URL 且 stderr 出现策略拒绝 warn。
/// （CDP evaluate 的 location 赋值同为 renderer 侧发起导航，无 browser 登记凭证。）
fn assert_file_nav_blocked(cdp: &mut CdpClient, target: &str, what: &str) {
    cdp.evaluate(&format!("location.href = '{target}'"))
        .expect("trigger nav");
    // 导航尝试面：location 须落在目标 URL（renderer 侧 commit），否则触发本身失真。
    let landed = poll_until(cdp, "location.href", Duration::from_secs(5), |v| {
        v.as_str().is_some_and(|href| href.contains(target))
    });
    assert!(landed.is_some(), "{what}: JS 导航未发生（触发失真，判别无效）");
    // 内容面：正文不得含目标文件内容（观察窗 + 终态复测）。
    let leaked = poll_until(cdp, "document.body?.innerText ?? ''", Duration::from_secs(3), |v| {
        v.as_str().is_some_and(|t| t.contains(SECRET_MARKER))
    });
    assert!(
        leaked.is_none(),
        "{what}: 本地文件内容被渲染（{leaked:?}，范围策略失效回潮）"
    );
    let final_body = cdp.evaluate("document.body?.innerText ?? ''").expect("final body");
    assert!(
        !final_body.as_str().unwrap_or_default().contains(SECRET_MARKER),
        "{what}: 终态正文含目标文件内容: {final_body}"
    );
}

#[test]
fn http_page_cannot_read_local_files() {
    let (_dir, _site_url, secret_png, secret_html) = write_fixture("http");
    let site = TestSite::spawn(&secret_png);
    let (_port, _browser, stderr_path) = spawn_browser();
    let mut cdp = CdpClient::connect(_port);

    cdp.call("Page.enable", json!({}), Duration::from_secs(5))
        .expect("Page.enable");
    cdp.call(
        "Page.navigate",
        json!({ "url": site.url("/probe") }),
        Duration::from_secs(15),
    )
    .expect("Page.navigate response");
    cdp.wait_event("Page.loadEventFired", Duration::from_secs(15))
        .expect("load event");

    // 正控：http 图片正常加载（证明 renderer 图片通道本身可用，负控不失真）。
    let http_width = wait_img_loaded(&mut cdp, "httpimg");
    assert_eq!(http_width.as_f64(), Some(1.0), "http 正控 img 应为 1×1 解码成功");

    // 负控 1：http 文档下 file:// 子资源拒绝。
    assert_img_blocked(&mut cdp, "local", "http 页面 file:// 子资源");

    // 负控 2：http 文档页面 JS 导航 file:// 拒绝（无 browser 登记凭证）。
    assert_file_nav_blocked(&mut cdp, &secret_html, "http 页面 JS 导航 file://");

    // 服务端拒绝证据：browser stderr 须出现策略拒绝 warn（两条负控各 ≥1 条）。
    let stderr = std::fs::read_to_string(&stderr_path).unwrap_or_default();
    assert!(
        stderr.contains("file fetch blocked headless"),
        "stderr 缺策略拒绝证据（deny 未走 check_file_url_scope 面?）:\n{stderr}"
    );
    let deny_count = stderr.matches("file fetch blocked headless").count();
    assert!(
        deny_count >= 2,
        "策略拒绝 warn 不足 2 条（子资源+导航腿）: {deny_count}"
    );
}

#[test]
fn file_page_subtree_confines_subresources_and_allows_file_nav() {
    let (_dir, site_url, _secret_png, secret_html) = write_fixture("file");
    let (_port, _browser, _stderr_path) = spawn_browser();
    let mut cdp = CdpClient::connect(_port);

    // CDP 导航 = browser 发起（登记凭证面），file:// 文档放行（同 round-4 钉面）。
    cdp.call("Page.enable", json!({}), Duration::from_secs(5))
        .expect("Page.enable");
    cdp.call("Page.navigate", json!({ "url": site_url }), Duration::from_secs(15))
        .expect("Page.navigate response");
    cdp.wait_event("Page.loadEventFired", Duration::from_secs(15))
        .expect("load event");

    // 放行腿：同目录子资源（same-dir-or-below；同时证明 document 锚点已服务端钉住）。
    let ok_width = wait_img_loaded(&mut cdp, "ok");
    assert_eq!(ok_width.as_f64(), Some(1.0), "file 文档同目录 img 应加载");

    // 拒绝腿：`../` 越出文档目录树（PathBuf 组件级前缀）。
    assert_img_blocked(&mut cdp, "leak", "file 文档树外子资源");

    // 防过约腿：file→file 页面 JS 导航放行（Document 面「最近文档为 file://」语义，
    // Chrome 对齐——已提交 file 文档可在本地文件间导航）。
    cdp.evaluate(&format!("location.href = '{secret_html}'"))
        .expect("trigger nav");
    let committed = poll_until(
        &mut cdp,
        "location.href + '|' + (document.body?.innerText ?? '')",
        Duration::from_secs(5),
        |v| v.as_str().is_some_and(|t| t.contains(SECRET_MARKER)),
    );
    let state = committed.unwrap_or_else(|| panic!("file→file 导航未落地（Document 面过约回归）"));
    let text = state.as_str().unwrap_or_default();
    assert!(text.contains(SECRET_MARKER), "file→file 导航正文缺目标内容: {text}");
    assert!(text.contains(&secret_html), "location 未落在目标 file URL: {text}");
}
