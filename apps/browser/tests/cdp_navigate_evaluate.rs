//! CDP `Page.navigate` → `Runtime.evaluate` 通道钉（slice21 eval-wedge）。
//!
//! 既有主支缺陷：`Page.navigate` 的导航事件族在响应前同步执行 iframe 探测求值——
//! renderer 单 JS worker 忙于页面脚本臂时该求值排在其后，导航命令阻塞整个单线程
//! 复用循环，冻结期间所有连接的命令/事件停摆（导航后跨连接 evaluate 全数饿死，
//! s20-eval-wedge-ab-verdict.txt 双实证）。
//!
//! 钉面（真实进程：zero-browser + zero-renderer + 本地静态 HTTP）：
//! 1. `navigate_busy_page_returns_promptly`——含同步忙脚本的页面，navigate 响应
//!    不得等脚本臂（base ~臂长，修复 ~数百 ms）。
//! 2. `first_connection_evaluate_works`——进程启动后首个连接 evaluate 可用
//!    （「首连窗口」语义防修复时无意破坏）。
//! 3. `evaluate_without_navigation_works`——负控制：无导航直连 evaluate 正常。
//! 4. `multiple_navigations_evaluate_works`——相邻变体：两次导航后 evaluate 正常
//!    （连续导航不残留旧文档状态）。
//! 5. `iframe_page_frame_attached_events`——iframe 页 frameAttached 即时路径。
//! 6. `busy_iframe_page_evaluate_works`——忙臂 iframe 页导航即时返回 + 臂中发送
//!    evaluate（臂后应答）通道必答（忙臂页放弃 frame 元数据，evaluate 语义不丢）。
//! 7. `kill_switch_env_off_keeps_frame_attached_on_busy_iframe_page`——kill-switch
//!    `ZW_CDP_NAV_IFRAME_PROBE_ASYNC=0`（子进程 env 注入）回落旧阻塞探测：忙臂
//!    iframe 页 frameAttached 事件在（旧行为面常驻判别）。
//! 8. `default_async_probe_gives_up_frame_attached_on_busy_iframe_page`——默认
//!    异步探测忙臂页放弃 frame 元数据：frameAttached 不发（负向判别，base 上
//!    此断言翻红——旧路径事件在）。
//! 9. `cross_connection_stays_responsive_during_busy_navigation`——跨连接停摆
//!    症状面：conn A 忙导航占位期间，conn B 新连接 Page.enable 在 1.5s 窗内可达
//!    （base 上复用循环被冻结 ~臂长，B 不可达翻红——s20 原始症状防回归）。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use tungstenite::Message;

/// 测试页服务：线程化静态 HTTP（进程生命周期 = 测试函数）。
struct TestSite {
    port: u16,
    #[allow(dead_code)]
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl TestSite {
    /// 启动静态服务。路径集固定：/busy?ms=<n>（同步忙臂 n ms）、/iframes（2 iframe）、
    /// /busyframes?ms=<n>（忙臂 + 1 iframe）、/plain（普通页）。
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
                let Ok(mut stream) = stream else { continue };
                let mut buf = [0u8; 2048];
                let n = stream.read(&mut buf).unwrap_or(0);
                let req = String::from_utf8_lossy(&buf[..n]).to_string();
                let path = req.split_whitespace().nth(1).unwrap_or("/").to_string();
                let body = match path.split('?').next().unwrap_or("/") {
                    "/plain" => plain_page("S21 Plain"),
                    "/iframes" => "<!DOCTYPE html><html><head><title>S21 Iframes</title></head><body>\
                        <iframe src=\"/plain\"></iframe><iframe src=\"/plain\"></iframe></body></html>"
                        .to_string(),
                    p if p.starts_with("/busyframes") => {
                        let ms = path
                            .split("ms=")
                            .nth(1)
                            .and_then(|v| v.parse::<u64>().ok())
                            .unwrap_or(3000);
                        format!(
                            "<!DOCTYPE html><html><head><title>S21 BusyFrames</title></head><body>\
                             <iframe src=\"/plain\"></iframe>\
                             <script>(function(){{var t0=Date.now();while(Date.now()-t0<{ms}){{}}}})();</script>\
                             </body></html>"
                        )
                    }
                    p if p.starts_with("/busy") => {
                        let ms = path
                            .split("ms=")
                            .nth(1)
                            .and_then(|v| v.parse::<u64>().ok())
                            .unwrap_or(3000);
                        format!(
                            "<!DOCTYPE html><html><head><title>S21 Busy {ms}</title></head><body>\
                             <script>(function(){{var t0=Date.now();while(Date.now()-t0<{ms}){{}}}})();</script>\
                             </body></html>"
                        )
                    }
                    _ => plain_page("S21 NotFound"),
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });
        Self { port, shutdown }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{}{path}", self.port)
    }
}

fn plain_page(title: &str) -> String {
    format!("<!DOCTYPE html><html><head><title>{title}</title></head><body><h1>ok</h1></body></html>")
}

/// 被测浏览器进程（RAII 关停）。
struct BrowserProcess(Child);

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// 启动 zero-browser --headless 并等待 CDP 发现端点就绪（默认环境）。
fn spawn_browser() -> (u16, BrowserProcess) {
    spawn_browser_with_env(&[])
}

/// 启动 zero-browser --headless 并等待 CDP 发现端点就绪；`vars` 注入子进程环境
///（kill-switch 等行为开关用子进程注入，不用 `std::env::set_var`——edition 2024
/// unsafe 且与并行测试竞态）。
fn spawn_browser_with_env(vars: &[(&str, &str)]) -> (u16, BrowserProcess) {
    // 端口预分配：先占住再释放（窗口期极小；发现端点轮询兜底）。
    let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
    let port = probe.local_addr().expect("addr").port();
    drop(probe);

    let storage = std::env::temp_dir().join(format!("zw-s21-pin-{}-{}", std::process::id(), port));
    std::fs::create_dir_all(&storage).expect("storage dir");
    // renderer 与 browser 同 target 目录（dev profile），取同级；ZERO_RENDERER_PATH 可覆盖。
    let browser_bin = env!("CARGO_BIN_EXE_zero-browser");
    let renderer = std::env::var("ZERO_S21_RENDERER_PATH")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::Path::new(browser_bin)
                .parent()
                .expect("browser bin dir")
                .join("zero-renderer")
        });

    let mut cmd = Command::new(browser_bin);
    cmd.arg("--headless")
        .arg(format!("--remote-debugging-port={port}"))
        .env("ZERO_STORAGE_DIR", &storage)
        .env("ZERO_RENDERER_PATH", &renderer);
    for (key, value) in vars {
        cmd.env(key, value);
    }
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

/// 最小 CDP 客户端（同步 ws；每连接独占——与 s20 矩阵形态一致）。
struct CdpClient {
    ws: tungstenite::WebSocket<TcpStream>,
    next_id: u64,
    /// 等响应途中途经的事件 `(method, params)`（wait_response 不丢弃，wait_event 先查缓冲）。
    pending_events: Vec<(String, Value)>,
}

impl CdpClient {
    fn connect(port: u16) -> Self {
        Self::try_connect(port, Duration::from_millis(1400)).expect("ws handshake")
    }

    /// 带握手超时的连接：`handshake_timeout` 只约束 ws 握手阶段（mux 可能正被其他
    /// 连接的命令占用——如跨连接停摆钉的判别窗），握手完成后回到 50ms 轮询读超时。
    fn try_connect(port: u16, handshake_timeout: Duration) -> Result<Self, String> {
        // 预设读超时的 TCP 流交给 tungstenite（MaybeTlsStream 不透出 set_read_timeout）。
        let stream = TcpStream::connect(("127.0.0.1", port)).map_err(|e| format!("tcp connect: {e}"))?;
        stream.set_read_timeout(Some(handshake_timeout)).expect("read timeout");
        stream
            .set_write_timeout(Some(Duration::from_secs(10)))
            .expect("write timeout");
        let (ws, _resp) =
            tungstenite::client(format!("ws://127.0.0.1:{port}"), stream).map_err(|e| format!("ws handshake: {e}"))?;
        ws.get_ref()
            .set_read_timeout(Some(Duration::from_millis(50)))
            .expect("read timeout");
        Ok(Self {
            ws,
            next_id: 0,
            pending_events: Vec::new(),
        })
    }

    /// 发送命令（不等待——竞速/时序断言用）。
    fn send(&mut self, method: &str, params: Value) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        let text = serde_json::json!({ "id": id, "method": method, "params": params }).to_string();
        self.ws.send(Message::Text(text.into())).expect("ws send");
        id
    }

    /// 发送并等待同名命令响应（读超时 = 断言失败）。
    fn call(&mut self, method: &str, params: Value, timeout: Duration) -> Result<Value, String> {
        let id = self.send(method, params);
        self.wait_response(id, timeout)
    }

    /// 等指定 id 的响应；途经的其他消息（事件/其他响应）一并返回 None 跳过。
    fn wait_response(&mut self, id: u64, timeout: Duration) -> Result<Value, String> {
        let deadline = Instant::now() + timeout;
        loop {
            if Instant::now() >= deadline {
                return Err(format!("timeout waiting for response id {id}"));
            }
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let msg: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
                    if msg.get("id").and_then(Value::as_u64) == Some(id) {
                        return Ok(msg);
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
                Err(e) => return Err(format!("ws read: {e}")),
            }
        }
    }

    /// 等待指定事件在 `timeout` 内出现（消费途经消息），返回其 params。
    fn wait_event(&mut self, method: &str, timeout: Duration) -> Result<Value, String> {
        // 途经缓冲里已有目标事件则直接取（早于导航响应到达的事件）。
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

/// 钉 1：含同步忙脚本的页面，`Page.navigate` 响应不得等页面脚本臂。
///
/// base 行为：导航事件族在响应前同步跑 iframe 探测求值 → 排在忙臂后 → 响应
/// ~臂长（3000ms 臂实测 3.2s+）；修复后探测有界等待 → 响应数百 ms。
#[test]
fn navigate_busy_page_returns_promptly() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Runtime.enable");

    let t0 = Instant::now();
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/busy?ms=3000") }),
        Duration::from_secs(15),
    )
    .expect("navigate response");
    let elapsed = t0.elapsed();
    // 忙臂 3000ms：base ~3.2s 落入断言失败区间；修复 ~0.3s。阈值取臂长一半。
    assert!(
        elapsed < Duration::from_millis(1500),
        "Page.navigate blocked {:?} (must not wait for page script arms)",
        elapsed
    );
}

/// 钉 2：进程启动后首个连接 evaluate 可用（「首连窗口」语义）。
#[test]
fn first_connection_evaluate_works() {
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    let value = cdp
        .evaluate("1+1", Duration::from_secs(10))
        .expect("first-connection evaluate must respond");
    assert_eq!(value.as_f64(), Some(2.0));
}

/// 负控制：无导航直连 evaluate 正常（钉修复不破坏直连路径）。
#[test]
fn evaluate_without_navigation_works() {
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Runtime.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Runtime.enable");
    cdp.call("Page.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Page.enable");
    let value = cdp.evaluate("6*7", Duration::from_secs(10)).expect("evaluate");
    assert_eq!(value.as_f64(), Some(42.0));
}

/// 相邻变体：两次导航后 evaluate 正常（连续导航不残留旧文档状态）。
#[test]
fn multiple_navigations_evaluate_works() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Page.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Page.enable");
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/busy?ms=1500") }),
        Duration::from_secs(15),
    )
    .expect("first navigate");
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/plain") }),
        Duration::from_secs(15),
    )
    .expect("second navigate");
    let value = cdp
        .evaluate("String(document.title)", Duration::from_secs(10))
        .expect("evaluate after two navigations");
    assert_eq!(value, Value::from("S21 Plain"));
}

/// 相邻变体：iframe 页导航 → frameAttached 即时路径（renderer 空闲，探测有界等待内应答）。
#[test]
fn iframe_page_frame_attached_events() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Page.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Page.enable");
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/iframes") }),
        Duration::from_secs(15),
    )
    .expect("navigate");
    // 2 个子帧的 frameAttached（导航响应前后即可见，不等脚本臂——本页无脚本）。
    cdp.wait_event("Page.frameAttached", Duration::from_secs(5))
        .expect("frameAttached #1");
    cdp.wait_event("Page.frameAttached", Duration::from_secs(5))
        .expect("frameAttached #2");
    let value = cdp
        .evaluate("String(document.title)", Duration::from_secs(10))
        .expect("evaluate on iframe page");
    assert_eq!(value, Value::from("S21 Iframes"));
}

/// 忙臂 iframe 页：导航即时返回（同钉 1 语义）+ 臂结束后 evaluate 通道可用。
///
/// 忙臂页的 iframe 探测在界内未应答时按 0 子帧放弃（迟发求值与导航管线 worker
/// 快照安装竞态，迟发 frameAttached 不可靠）——本页不产出 frame 元数据，但
/// evaluate 语义不丢（base 上此形态 evaluate 直接楔死）。
#[test]
fn busy_iframe_page_evaluate_works() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Page.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Page.enable");
    let t0 = Instant::now();
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/busyframes?ms=2500") }),
        Duration::from_secs(15),
    )
    .expect("navigate");
    // 导航不被 2500ms 忙臂阻塞（同钉 1 语义）。
    assert!(
        t0.elapsed() < Duration::from_millis(1250),
        "navigate blocked {:?} on busy iframe page",
        t0.elapsed()
    );
    // 臂中发送 evaluate（2500ms 臂未结束，求值排在忙臂后）：应答在臂后返回——通道必答语义。
    std::thread::sleep(Duration::from_millis(1500));
    let value = cdp
        .evaluate("String(document.title)", Duration::from_secs(10))
        .expect("evaluate after busy arm");
    assert_eq!(value, Value::from("S21 BusyFrames"));
}

/// kill-switch 常驻钉（env=0 侧）：`ZW_CDP_NAV_IFRAME_PROBE_ASYNC=0` 子进程注入
/// 回落旧无界阻塞探测——忙臂 iframe 页的 frameAttached 事件在（旧行为面）。
///
/// 无严格计时断言（旧路径 navigate 会等臂长，属旧行为的一部分，不作断言）。
/// 判别方向：若 kill-switch 失效（env=0 未回落旧路径），本钉在修复代码上翻红
///（事件不再发出）。
#[test]
fn kill_switch_env_off_keeps_frame_attached_on_busy_iframe_page() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser_with_env(&[("ZW_CDP_NAV_IFRAME_PROBE_ASYNC", "0")]);
    let mut cdp = CdpClient::connect(port);
    cdp.call("Page.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Page.enable");
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/busyframes?ms=2500") }),
        Duration::from_secs(15),
    )
    .expect("navigate");
    // 旧行为：探测在臂后同步应答（本页 1 iframe）→ frameAttached 必在。
    cdp.wait_event("Page.frameAttached", Duration::from_secs(15))
        .expect("frameAttached present under kill-switch (legacy blocking probe)");
}

/// kill-switch 常驻钉（默认侧，负向判别）：默认异步探测在忙臂页超界放弃——
/// frameAttached 不发（迟发求值与导航管线 worker 快照安装竞态，宁缺勿错）。
///
/// 无严格计时断言；等待窗 5s 覆盖臂长（2500ms）+ 余量。base 上本钉翻红
///（旧路径事件在）——翻红方向在 base 侧实证归档。
#[test]
fn default_async_probe_gives_up_frame_attached_on_busy_iframe_page() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut cdp = CdpClient::connect(port);
    cdp.call("Page.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("Page.enable");
    cdp.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/busyframes?ms=2500") }),
        Duration::from_secs(15),
    )
    .expect("navigate");
    assert!(
        cdp.wait_event("Page.frameAttached", Duration::from_secs(5)).is_err(),
        "frameAttached must not be emitted when the bounded probe gives up on a busy arm"
    );
}

/// 跨连接停摆症状面钉（s20 原始症状防回归）：conn A 忙导航占位期间，conn B
/// 新连接在 1.5s 窗内可达（Page.enable 应答）。
///
/// base 行为：conn A 的 navigate 命令冻结单线程复用循环 ~臂长（3000ms 臂实测
/// 3.2s+），conn B 的 ws 握手/Page.enable 全部排队 → 1.5s 窗内不可达翻红。
/// 修复后 mux 不冻结：B 握手 + Page.enable 毫秒级。evaluate 仅作通道端到端
/// 必答核对（臂后应答，无严格计时）；1.5s 窗为判别边界非性能断言。
#[test]
fn cross_connection_stays_responsive_during_busy_navigation() {
    let site = TestSite::spawn();
    let (port, _browser) = spawn_browser();
    let mut conn_a = CdpClient::connect(port);
    conn_a
        .call("Page.enable", serde_json::json!({}), Duration::from_secs(10))
        .expect("conn A Page.enable");
    conn_a.send("Page.navigate", serde_json::json!({ "url": site.url("/busy?ms=3000") }));

    // conn B 在 A 忙导航占位期新连接：握手（1.4s 上限）+ Page.enable（1.5s 判别窗）。
    let t0 = Instant::now();
    let mut conn_b = CdpClient::try_connect(port, Duration::from_millis(1400))
        .expect("conn B handshake within the 1.5s window (mux not frozen)");
    conn_b
        .call("Page.enable", serde_json::json!({}), Duration::from_millis(1500))
        .expect("conn B must answer Page.enable within the 1.5s window");
    assert!(
        t0.elapsed() < Duration::from_millis(1500),
        "conn B setup took {:?} (mux frozen by conn A navigation?)",
        t0.elapsed()
    );

    // 通道端到端必答：evaluate 臂后应答命中新文档（无严格计时断言）。
    let value = conn_b
        .evaluate("String(document.title)", Duration::from_secs(15))
        .expect("conn B evaluate must answer");
    assert_eq!(value, Value::from("S21 Busy 3000"));
    conn_a
        .wait_response(2, Duration::from_secs(15))
        .expect("conn A navigate response");
}
