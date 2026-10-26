//! P10 宿主历史桥 E2E 钉（headless IPC 面）：页面 `history.back()/forward()` 越出
//! renderer 本地栈后移交宿主 shell 会话历史栈的完整链路——shim 越界探测 →
//! `__zw_request_history_traverse` → `HistoryTraverse` IPC → `HeadlessSession`
//! 宿主分发（jump_history + navigate_renderer）。真实进程：zero-browser +
//! zero-renderer + 本地静态 HTTP。
//!
//! 宿主分发缺失（该 IPC 落入兜底被丢弃）时 back/forward 不产生任何导航，
//! location 停留原文档——本钉据此翻红。边界（R3004）：栈端越界请求宿主判界
//! 静默 no-op，不得 wedge 命令通道。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use tungstenite::Message;

/// 测试页服务：线程化静态 HTTP（进程生命周期 = 测试函数）。
struct TestSite {
    port: u16,
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl TestSite {
    /// 启动静态服务。路径集固定：/list 与 /detail（跨文档历史两页）。
    fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test site");
        let port = listener.local_addr().expect("addr").port();
        let shutdown = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        std::thread::spawn(move || {
            listener.set_nonblocking(true).ok();
            for stream in listener.incoming() {
                if shutdown_clone.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
                let Ok(stream) = stream else { continue };
                // 每连接一线程：read 有超时且不得阻塞 accept 循环——浏览器会开
                // 预连接（不发数据），单线程同步读会卡死后续全部请求。线程寿命
                // 由读超时兜底，无需感知 shutdown。
                std::thread::spawn(move || {
                    let mut stream = stream;
                    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
                    let mut buf = [0u8; 2048];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let Ok(request) = std::str::from_utf8(&buf[..n]) else {
                        return;
                    };
                    let path = request.split_whitespace().nth(1).unwrap_or("/");
                    let body = match path {
                        "/list" | "/list?x=1" => page("list"),
                        "/detail" | "/detail?x=1" => page("detail"),
                        _ => page("other"),
                    };
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
                    let _ = stream.write_all(response.as_bytes());
                });
            }
        });
        Self { port, shutdown }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }
}

fn page(marker: &str) -> String {
    format!(
        "<!DOCTYPE html><html><head><title>{marker}</title></head><body><h1 id=\"marker\">{marker}</h1></body></html>"
    )
}

impl Drop for TestSite {
    fn drop(&mut self) {
        self.shutdown.store(true, std::sync::atomic::Ordering::Relaxed);
        // 唤醒 accept 循环以便观察到 shutdown（进程退出即回收，容错连接）。
        let _ = TcpStream::connect(("127.0.0.1", self.port));
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

/// 启动 zero-browser --headless 并等待 CDP 发现端点就绪。
fn spawn_browser() -> (u16, BrowserProcess) {
    // 端口预分配：先占住再释放（窗口期极小；发现端点轮询兜底）。
    let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
    let port = probe.local_addr().expect("addr").port();
    drop(probe);

    let storage = std::env::temp_dir().join(format!("zw-histpin-{}-{}", std::process::id(), port));
    std::fs::create_dir_all(&storage).expect("storage dir");
    // renderer 与 browser 同 target 目录（dev profile），取同级。
    let browser_bin = env!("CARGO_BIN_EXE_zero-browser");
    let renderer = std::path::Path::new(browser_bin)
        .parent()
        .expect("browser bin dir")
        .join("zero-renderer");

    let mut cmd = Command::new(browser_bin);
    cmd.arg("--headless")
        .arg(format!("--remote-debugging-port={port}"))
        .env("ZERO_STORAGE_DIR", &storage)
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

/// 最小 CDP 客户端（同步 ws；单连接独占）。
struct CdpClient {
    ws: tungstenite::WebSocket<TcpStream>,
    next_id: u64,
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
        Self { ws, next_id: 0 }
    }

    /// 发送并等待同名命令响应（读超时 = 断言失败）。
    fn call(&mut self, method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
        self.next_id += 1;
        let id = self.next_id;
        let text = serde_json::json!({ "id": id, "method": method, "params": params }).to_string();
        self.ws.send(Message::Text(text.into())).expect("ws send");
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() >= deadline {
                return Err(format!("timeout waiting for response to {method}"));
            }
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let msg: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    if msg.get("id").and_then(Value::as_u64) == Some(id) {
                        return Ok(msg);
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

    /// evaluate 表达式并取 returnByValue 结果值。
    fn evaluate(&mut self, expression: &str, timeout: Duration) -> Result<Value, String> {
        let response = self.call(
            "Runtime.evaluate",
            serde_json::json!({ "expression": expression, "returnByValue": true }),
            timeout,
        )?;
        Ok(response.pointer("/result/result/value").cloned().unwrap_or(Value::Null))
    }
}

/// 轮询 evaluate 直到表达式为真（跨文档导航就绪/遍历回程的统一等待面）。
fn wait_until(cdp: &mut CdpClient, expression: &str, timeout: Duration) {
    let deadline = Instant::now() + timeout;
    let mut last: String = "no-sample".into();
    loop {
        if Instant::now() >= deadline {
            panic!("timeout waiting for `{expression}` (last: {last})");
        }
        match cdp.evaluate(expression, Duration::from_secs(5)) {
            Ok(Value::Bool(true)) => return,
            Ok(v) => last = v.to_string(),
            Err(e) => last = format!("eval-error: {e}"),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn ready_at(url_fragment: &str) -> String {
    format!(
        "document.readyState === 'complete' && location.href.includes('{url_fragment}') && !!document.querySelector('#marker')"
    )
}

/// 主链钉：list → detail（跨文档）→ history.back() 回 list → history.forward()
/// 回 detail。两步都必须是真实导航（shell 栈条目切换 + renderer 重载），而非
/// shim 本地栈回放——跨文档后 renderer 本地栈无跨文档条目，回程只能经宿主。
#[test]
fn history_back_forward_traverse_host_session_stack() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Runtime.enable");

    // 前往 list → 跨文档 detail（两跳均走会话命令导航：浏览器级连接上，在
    // 渲染侧导航在途窗口发 evaluate 会停摆命令应答——既有问题（P15 待查），
    // 与本钉无关，钉内避免该窗口；跨文档前提仍成立：两文档两历史条目）。
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/list") }),
        Duration::from_secs(20),
    )
    .expect("navigate list");
    wait_until(&mut cdp, &ready_at("/list"), Duration::from_secs(20));
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/detail") }),
        Duration::from_secs(20),
    )
    .expect("navigate detail");
    wait_until(&mut cdp, &ready_at("/detail"), Duration::from_secs(20));

    // J3：history.back() 越界移交宿主 → 回 list。遍历由宿主同步执行（arm 内
    // navigate_renderer 阻塞复用循环至 DCL），先留调度窗再轮询。
    cdp.evaluate("history.back()", Duration::from_secs(10))
        .expect("evaluate history.back");
    std::thread::sleep(Duration::from_millis(1500));
    wait_until(&mut cdp, &ready_at("/list"), Duration::from_secs(20));

    // J4：history.forward() → 回 detail（delta=+1 分支）。
    cdp.evaluate("history.forward()", Duration::from_secs(10))
        .expect("evaluate history.forward");
    std::thread::sleep(Duration::from_millis(1500));
    wait_until(&mut cdp, &ready_at("/detail"), Duration::from_secs(20));
}

/// 边界钉（R3004）：栈端越界请求宿主判界静默 no-op——list（首条目）上
/// history.back() 与远超上界的 history.go(99) 均不产生导航，且命令通道保持
/// 可用（后续 evaluate 正常应答，不分发不 wedge）。
#[test]
fn history_traverse_out_of_bounds_is_silent_noop() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Runtime.enable");

    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/list") }),
        Duration::from_secs(20),
    )
    .expect("navigate list");
    wait_until(&mut cdp, &ready_at("/list"), Duration::from_secs(20));

    cdp.evaluate("history.back()", Duration::from_secs(10))
        .expect("evaluate history.back at first entry");
    cdp.evaluate("history.go(99)", Duration::from_secs(10))
        .expect("evaluate history.go(99)");
    // 越界不导航：留出调度窗口后确认仍在原页，且通道存活。
    std::thread::sleep(Duration::from_secs(2));
    let href = cdp
        .evaluate("location.href", Duration::from_secs(10))
        .expect("evaluate location.href after out-of-bounds");
    assert!(
        href.as_str().is_some_and(|u| u.contains("/list")),
        "out-of-bounds traverse must stay on /list, got {href}"
    );
    let two = cdp
        .evaluate("1 + 1", Duration::from_secs(10))
        .expect("evaluate 1 + 1 after out-of-bounds traverse");
    assert_eq!(
        two.as_f64(),
        Some(2.0),
        "command channel must stay responsive after out-of-bounds traverse"
    );
}
