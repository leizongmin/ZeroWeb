//! headless 代理 fetch 接入共享 HTTP 缓存的进程级回归钉（T25）。
//!
//! 既有缺陷：headless 会话 `proxy_fetch` 的 GET 直发网络调度器，零缓存查询/
//! 存储/条件请求——同 profile 热导航对可缓存子资源全量 200 重传（GUI 路径
//! fetch_proxy/ResourceLoader 已接线同一 `zero_net::shared_http_cache()`，
//! headless 旁路）。
//!
//! 钉面（真实进程：zero-browser + zero-renderer + 本地 HTTP，两次导航同页）：
//! 1. `max-age` 新鲜资源第二次导航零网络——服务器只收到 1 次请求（base 收 2 次）。
//! 2. `no-cache`+ETag 资源第二次导航带 `If-None-Match` 条件请求、服务器回 304
//!    （base 重发无条件请求拿全量 200）。
//! 3. renderer 侧经 CDP `Network.responseReceived` 观察到的该资源状态恒为 200
//!    ——304 在 browser 进程内由缓存收口为「存储体+更新头」，不得透传 renderer。
//!
//! `/marker`（`no-store`）每轮必上网，作为「第二轮页面加载与图片通道活跃」的
//! 确定性证据，防止「第二轮压根没请求」造成缓存钉假绿。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde_json::Value;
use tungstenite::Message;

/// 1×1 透明 PNG（图片通道最小合法载荷）。
const PNG_1X1: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00,
    0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49,
    0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00,
    0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

/// 测试站点：线程化 HTTP（进程生命周期 = 测试函数），按路径分缓存策略并计数。
struct TestSite {
    port: u16,
    #[allow(dead_code)]
    shutdown: Arc<std::sync::atomic::AtomicBool>,
    /// `/fresh`（max-age=60）收到的请求次数。
    fresh_hits: Arc<AtomicUsize>,
    /// `/marker`（no-store）收到的请求次数——每轮导航必上网。
    marker_hits: Arc<AtomicUsize>,
    /// `/reval`（no-cache+ETag）每次请求的（是否带 If-None-Match，响应状态）。
    reval_requests: Arc<Mutex<Vec<(bool, u16)>>>,
    /// 全部请求的 `(时刻, 请求行)` 日志（诊断加载节奏）。
    request_log: Arc<Mutex<Vec<(String, String)>>>,
}

impl TestSite {
    fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test site");
        let port = listener.local_addr().expect("addr").port();
        let shutdown = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let fresh_hits = Arc::new(AtomicUsize::new(0));
        let marker_hits = Arc::new(AtomicUsize::new(0));
        let reval_requests = Arc::new(Mutex::new(Vec::new()));
        let request_log = Arc::new(Mutex::new(Vec::new()));
        let started = Instant::now();
        let shutdown_clone = shutdown.clone();
        let fresh_hits_thread = fresh_hits.clone();
        let marker_hits_thread = marker_hits.clone();
        let reval_requests_thread = reval_requests.clone();
        let request_log_thread = request_log.clone();
        std::thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            for stream in listener.incoming() {
                if shutdown_clone.load(Ordering::Relaxed) {
                    break;
                }
                let Ok(mut stream) = stream else { continue };
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
                let req_lower = req.to_ascii_lowercase();
                request_log_thread
                    .lock()
                    .expect("request log")
                    .push((format!("{:.3}", started.elapsed().as_secs_f64()), path.clone()));
                let mut head_extra = String::new();
                let (status_line, body): (&str, Vec<u8>) = match path.split('?').next().unwrap_or("/") {
                    "/page" => (
                        "HTTP/1.1 200 OK",
                        "<!DOCTYPE html><html><head><title>T25 Cache</title>\
                         <link rel=\"stylesheet\" href=\"/reval\"></head><body>\
                         <img src=\"/fresh\"><img src=\"/marker\"></body></html>"
                            .to_string()
                            .into_bytes(),
                    ),
                    "/fresh" => {
                        fresh_hits_thread.fetch_add(1, Ordering::SeqCst);
                        head_extra.push_str("Cache-Control: public, max-age=60\r\n");
                        head_extra.push_str("ETag: \"t25-fresh-1\"\r\n");
                        ("HTTP/1.1 200 OK", PNG_1X1.to_vec())
                    }
                    "/marker" => {
                        marker_hits_thread.fetch_add(1, Ordering::SeqCst);
                        head_extra.push_str("Cache-Control: no-store\r\n");
                        ("HTTP/1.1 200 OK", PNG_1X1.to_vec())
                    }
                    "/reval" => {
                        // no-cache + ETag：每次使用前须再验证；带匹配 If-None-Match 回 304
                        //（RFC 9111 §4.3.4 条件请求收口）。
                        let had_validator = req_lower.contains("if-none-match:");
                        let status = if had_validator { 304 } else { 200 };
                        reval_requests_thread
                            .lock()
                            .expect("reval log")
                            .push((had_validator, status));
                        head_extra.push_str("Cache-Control: no-cache\r\n");
                        head_extra.push_str("ETag: \"t25-reval-1\"\r\n");
                        if status == 304 {
                            ("HTTP/1.1 304 Not Modified", Vec::new())
                        } else {
                            ("HTTP/1.1 200 OK", b"body{margin:0}\n".to_vec())
                        }
                    }
                    _ => ("HTTP/1.1 200 OK", b"not found".to_vec()),
                };
                let content_type = match path.split('?').next().unwrap_or("/") {
                    "/page" => "text/html",
                    "/reval" => "text/css",
                    _ => "image/png",
                };
                let response = format!(
                    "{status_line}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\n{head_extra}Connection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.write_all(&body);
                let _ = stream.flush();
            }
        });
        Self {
            port,
            shutdown,
            fresh_hits,
            marker_hits,
            reval_requests,
            request_log,
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }

    fn fresh_hits(&self) -> usize {
        self.fresh_hits.load(Ordering::SeqCst)
    }

    fn marker_hits(&self) -> usize {
        self.marker_hits.load(Ordering::SeqCst)
    }

    fn reval_requests(&self) -> Vec<(bool, u16)> {
        self.reval_requests.lock().expect("reval log").clone()
    }

    fn request_log(&self) -> Vec<(String, String)> {
        self.request_log.lock().expect("request log").clone()
    }

    /// 三计数快照（fresh / marker / reval 条数）。
    fn counts(&self) -> (usize, usize, usize) {
        (self.fresh_hits(), self.marker_hits(), self.reval_requests().len())
    }

    /// 等到 `min_marker` 达标且全部计数在 `quiet` 窗口内无新请求（加载节奏稳定，
    /// 兼容单次导航触发多轮资源请求的引擎行为）。
    fn wait_quiet(&self, min_marker: usize, quiet: Duration, timeout: Duration, what: &str) {
        let deadline = Instant::now() + timeout;
        let mut last = self.counts();
        let mut last_change = Instant::now();
        loop {
            assert!(
                Instant::now() < deadline,
                "timeout waiting for {what}: log={:?}",
                self.request_log()
            );
            std::thread::sleep(Duration::from_millis(50));
            let now = self.counts();
            if now != last {
                last = now;
                last_change = Instant::now();
                continue;
            }
            if self.marker_hits() >= min_marker && last_change.elapsed() >= quiet {
                return;
            }
        }
    }
}

/// 被测浏览器进程（RAII 关停）。
struct BrowserProcess(Child);

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// 启动 zero-browser --headless；`ZERO_CACHE_DIR` 指向全新临时目录（磁盘缓存层
/// 每用例隔离，测试条目不落真实用户缓存目录），等 CDP 发现端点就绪。
fn spawn_browser() -> (u16, BrowserProcess) {
    // 端口预分配：先占住再释放（窗口期极小；发现端点轮询兜底）。
    let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
    let port = probe.local_addr().expect("addr").port();
    drop(probe);

    let storage = std::env::temp_dir().join(format!("zw-t25-storage-{}-{}", std::process::id(), port));
    std::fs::create_dir_all(&storage).expect("storage dir");
    let cache_dir = std::env::temp_dir().join(format!("zw-t25-cache-{}-{}", std::process::id(), port));
    std::fs::create_dir_all(&cache_dir).expect("cache dir");
    // renderer 与 browser 同 target 目录（dev profile），取同级；与
    // cdp_navigate_evaluate.rs 同一查找约定（测试前须显式构建 peer binary）。
    let browser_bin = env!("CARGO_BIN_EXE_zero-browser");
    let renderer = std::path::Path::new(browser_bin)
        .parent()
        .expect("browser bin dir")
        .join("zero-renderer");

    let mut cmd = Command::new(browser_bin);
    cmd.arg("--headless")
        .arg(format!("--remote-debugging-port={port}"))
        .env("ZERO_STORAGE_DIR", &storage)
        .env("ZERO_CACHE_DIR", &cache_dir)
        .env("ZERO_RENDERER_PATH", &renderer);
    let child = cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn zero-browser");

    // 等 /json/version 就绪（首次启动含 renderer 子进程拉起）。
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

/// 最小 CDP 客户端（同步 ws；与 cdp_navigate_evaluate.rs 同构）。
struct CdpClient {
    ws: tungstenite::WebSocket<TcpStream>,
    next_id: u64,
    /// 等响应途中途经的事件 `(method, params)`。
    pending_events: Vec<(String, Value)>,
}

impl CdpClient {
    fn connect(port: u16) -> Self {
        let stream = TcpStream::connect(("127.0.0.1", port)).expect("tcp connect");
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .expect("read timeout");
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .expect("write timeout");
        let (ws, _resp) = tungstenite::client(format!("ws://127.0.0.1:{port}"), stream).expect("ws handshake");
        ws.get_ref()
            .set_read_timeout(Some(Duration::from_millis(50)))
            .expect("read timeout");
        Self {
            ws,
            next_id: 0,
            pending_events: Vec::new(),
        }
    }

    /// 发送并等待同名命令响应（读超时 = 断言失败）。
    fn call(&mut self, method: &str, params: Value, timeout: Duration) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let text = serde_json::json!({ "id": id, "method": method, "params": params }).to_string();
        self.ws.send(Message::Text(text.into())).expect("ws send");
        let deadline = Instant::now() + timeout;
        loop {
            assert!(Instant::now() < deadline, "timeout waiting for response id {id}");
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let msg: Value = serde_json::from_str(&text).expect("cdp json");
                    if msg.get("id").and_then(Value::as_u64) == Some(id) {
                        return msg;
                    }
                    if let Some(method) = msg.get("method").and_then(Value::as_str) {
                        self.pending_events
                            .push((method.to_string(), msg.get("params").cloned().unwrap_or(Value::Null)));
                    }
                }
                Ok(Message::Ping(data)) => {
                    self.ws.send(Message::Pong(data)).expect("pong");
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(e) => panic!("ws read: {e}"),
            }
        }
    }

    /// 在 `window` 内排空事件流，收集 `Network.responseReceived` 的 `(url, status)`
    ///（含途经缓冲；headless 事件经 transport 逐命令排空，调用侧先发廉价命令触发）。
    fn collect_response_received(&mut self, window: Duration) -> Vec<(String, u16)> {
        let mut collected = Vec::new();
        let mut buffered = std::mem::take(&mut self.pending_events);
        for (method, params) in buffered.drain(..) {
            if method == "Network.responseReceived" {
                collected.push(Self::response_url_status(&params));
            }
        }
        let deadline = Instant::now() + window;
        while Instant::now() < deadline {
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let Ok(msg) = serde_json::from_str::<Value>(&text) else {
                        continue;
                    };
                    if msg.get("method").and_then(Value::as_str) == Some("Network.responseReceived") {
                        let params = msg.get("params").cloned().unwrap_or(Value::Null);
                        collected.push(Self::response_url_status(&params));
                    }
                }
                Ok(Message::Ping(data)) => {
                    self.ws.send(Message::Pong(data)).expect("pong");
                }
                Ok(_) => {}
                Err(tungstenite::Error::Io(e))
                    if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {}
                Err(_) => break,
            }
        }
        collected
    }

    fn response_url_status(params: &Value) -> (String, u16) {
        (
            params
                .pointer("/response/url")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            params.pointer("/response/status").and_then(Value::as_u64).unwrap_or(0) as u16,
        )
    }
}

/// 两次导航：max-age 资源第二轮零网络；no-cache+ETag 资源第二轮走条件再验证
/// 且 renderer 侧恒见 200；no-store 资源每轮均上网（通道活跃证据）。
///
/// 计数断言全部用「导航间增量」：引擎单次导航可能触发多批资源请求（实测首航
/// 图片双请求），绝对计数不可依赖；以 no-store marker 计数上升 + 1.5s 静默窗
/// 判定「该轮资源请求已收敛」，再比对各资源相对基线的增量。
#[test]
fn second_navigation_reuses_http_cache() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Network.enable", serde_json::json!({}), Duration::from_secs(10));

    // 第一轮导航：三资源全部上网，等待请求流收敛。
    let nav1 = cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/page?r=1") }),
        Duration::from_secs(20),
    );
    println!("nav1 response: {nav1}");
    site.wait_quiet(
        1,
        Duration::from_millis(1500),
        Duration::from_secs(20),
        "first navigation subresources",
    );
    let (fresh_base, _marker_base, reval_base) = site.counts();
    assert!(
        fresh_base >= 1 && reval_base >= 1,
        "first navigation must fetch all subresources: log={:?}",
        site.request_log()
    );

    // 第二轮导航（同进程同 profile）：no-store 资源必上网（页面加载与 fetch 通道
    // 活跃证据），reval 资源必上网（条件再验证是网络请求）。
    let nav2 = cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/page?r=2") }),
        Duration::from_secs(20),
    );
    println!("nav2 response: {nav2}");
    site.wait_quiet(
        _marker_base + 1,
        Duration::from_millis(1500),
        Duration::from_secs(20),
        "second navigation subresources",
    );

    // 钉 1：max-age 新鲜资源第二轮零网络——缓存新鲜命中，服务器不得收到新请求。
    // base（无缓存接线）：第二轮全量 200 重传，fresh 增量 ≥1。
    let fresh_after = site.fresh_hits();
    assert_eq!(
        fresh_after,
        fresh_base,
        "max-age subresource must be served from cache on second navigation (zero new requests); log={:?}",
        site.request_log()
    );

    // 钉 2：no-cache+ETag 资源第二轮必须带 If-None-Match 条件请求且服务器回 304。
    // base：无条件请求，全量 200。
    let reval = site.reval_requests();
    let reval_round2 = &reval[reval_base..];
    assert!(
        !reval_round2.is_empty(),
        "second navigation must revalidate the no-cache+ETag resource; log={:?}",
        site.request_log()
    );
    for (i, (had_validator, status)) in reval_round2.iter().enumerate() {
        assert!(*had_validator, "round-2 reval request #{i} must carry If-None-Match");
        assert_eq!(*status, 304, "server must answer round-2 reval request #{i} with 304");
    }

    // 钉 3：renderer 观察面（Network.responseReceived）恒 200——304 的「存储体+更新头」
    // 收口发生在 browser 进程缓存层，不得把 304 状态透传 renderer。
    // headless 事件经 transport 逐命令排空：先发廉价命令触发排空再收集。
    cdp.call(
        "Runtime.evaluate",
        serde_json::json!({ "expression": "1" }),
        Duration::from_secs(5),
    );
    let responses = cdp.collect_response_received(Duration::from_secs(2));
    let reval_statuses: Vec<u16> = responses
        .iter()
        .filter(|(url, _)| url.contains("/reval"))
        .map(|(_, status)| *status)
        .collect();
    assert!(
        reval_statuses.len() >= 2,
        "expected responseReceived for /reval in both navigations, got {reval_statuses:?} (all: {responses:?})"
    );
    assert!(
        reval_statuses.iter().all(|status| *status == 200),
        "renderer must observe 200 (cached body rebuilt from 304), got {reval_statuses:?}"
    );
}
