//! T27-P35 真进程红绿钉：在途阻塞 fetch（FontFace.load）冻结渲染主循环期间，
//! 点击必须在有界等待内派发并立即触发导航，而非扣压到 fetch 链走完。
//!
//! 缺陷定谳（perf-mem-20261023 P35，Phase A 插桩证据
//! `evidence/t27-run-browser.log` run-5）：renderer 阻塞 fetch 等待用无界
//! `inbound_rx.recv()`，点击 DispatchDomEvent 被扣进局部 skipped、fetch 完成才
//! 回填放行——12s POST 臂扣压点击 8.1s、20s 字体串行窗冻结期间输入全部滞留，
//! 30s×N 链 ≈ 分钟级「点了没反应」。修复：`ipc_fetch_wait_step` 优先通道——
//! 点击/输入类消息即时 Yield 给主循环派发（对齐 t2-pb1 fix#14 Navigate 抢占），
//! 其余消息仍扣压回填；等待总上限 60s。
//!
//! 钉面（真实 zero-browser + zero-renderer + 本地延迟 fixture，CDP ws 逐连接独占，
//! harness 同 `cdp_input_events.rs`）：页面在 load+200ms 发起 15s 字体取（渲染
//! 主循环冻结窗），t≈+1.5s CDP 点击 `/p2` 链接，以 **fixture 服务端 `/p2` 首次
//! 被请求** 的时刻为派发反馈口径（服务端计数器 + 测试本地时钟，不依赖主循环
//! 内的 CDP evaluate——该通道在冻结窗内同样被扣压，红绿两态都不可用作观测）。
//! 绿：导航在点击后 8s 内开始（实测 ≤3s）；红：扣压至字体完成（实测 ≈13.7s），
//! 断言翻红。8s 阈值距绿 ≥2×、不足红一半，双向余量充分。
//!
//! 前提：`cargo build -p zero-renderer`（renderer 二进制与源码陈旧断言同 s23）。
//! 点击导航命令到达 browser 后由其 zero-net 取 `/p2`；renderer 侧 LoadHtml 处理
//! 仍排在字体 fetch 之后（非输入类不 Yield，T27 范围外），故本钉只断言「导航
//! 开始」不被点击扣压，不断言导航绘制完成。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use serde_json::Value;
use tungstenite::Message;

/// 字体臂持流时长：显著长于绿阈值（8s），红态扣压量可判别。
const FONT_DELAY: Duration = Duration::from_secs(15);
/// 绿阈值：点击→fixture 见到 `/p2` 请求。修复后有界等待（250ms 切片）+ 导航发起
/// ≈3s 内；红态实测 ≈13.7s（字体 15s 窗扣压），单向越界即翻红。
const NAV_START_BOUND: Duration = Duration::from_secs(8);
/// 最小 ttf 头 + 零填充（合法性不影响阻塞臂计时，t27-fixture 同源字节）。
const FONT_BYTES: &[u8] = &[
    0x00, 0x00, 0x01, 0x00, 0x09, 0x00, 0x10, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

/// 延迟 fixture：`/` 链接页（load+200ms 发起 15s 字体取）、`/slowfont`（15s 持流）、
/// `/p2`（链接目标，即时应答）。每连接一线程——浏览器会开预连接（不发数据），
/// 单线程同步读会卡死 accept 循环（image_batch_e2e BurstSite 同型）。
struct DelaySite {
    port: u16,
    index_served: Arc<AtomicUsize>,
    font_started: Arc<AtomicUsize>,
    font_done: Arc<AtomicUsize>,
    p2_served: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    accept_loop: Option<std::thread::JoinHandle<()>>,
}

impl DelaySite {
    fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture");
        let port = listener.local_addr().expect("fixture address").port();
        listener.set_nonblocking(true).expect("nonblocking fixture");
        let index_served = Arc::new(AtomicUsize::new(0));
        let font_started = Arc::new(AtomicUsize::new(0));
        let font_done = Arc::new(AtomicUsize::new(0));
        let p2_served = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));
        let accept_loop = {
            let index_served = Arc::clone(&index_served);
            let font_started = Arc::clone(&font_started);
            let font_done = Arc::clone(&font_done);
            let p2_served = Arc::clone(&p2_served);
            let shutdown = Arc::clone(&shutdown);
            std::thread::spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let index_served = Arc::clone(&index_served);
                            let font_started = Arc::clone(&font_started);
                            let font_done = Arc::clone(&font_done);
                            let p2_served = Arc::clone(&p2_served);
                            std::thread::spawn(move || {
                                serve(stream, &index_served, &font_started, &font_done, &p2_served);
                            });
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            std::thread::sleep(Duration::from_millis(2));
                        }
                        Err(error) => panic!("fixture accept failed: {error}"),
                    }
                }
            })
        };
        Self {
            port,
            index_served,
            font_started,
            font_done,
            p2_served,
            shutdown,
            accept_loop: Some(accept_loop),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://127.0.0.1:{port}{path}", port = self.port)
    }
}

impl Drop for DelaySite {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(accept_loop) = self.accept_loop.take() {
            let _ = accept_loop.join();
        }
    }
}

/// 单连接服务：`/slowfont` 计 started → 持流 → 计 done；`/p2` 计数即时应答；其余
/// 按链接页应答（load+200ms 后发起字体取——绝对 URL 用 `location.origin` 拼，
/// 相对 URL 在 zero-net FontFace src 解析已证不通）。
fn serve(
    mut stream: TcpStream,
    index_served: &AtomicUsize,
    font_started: &AtomicUsize,
    font_done: &AtomicUsize,
    p2_served: &AtomicUsize,
) {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let Some(head) = read_request_head(&mut stream) else {
        return;
    };
    let request = String::from_utf8_lossy(&head);
    let path = request.split_whitespace().nth(1).unwrap_or("/").to_string();
    if path.starts_with("/slowfont") {
        font_started.fetch_add(1, Ordering::Relaxed);
        std::thread::sleep(FONT_DELAY);
        write_response(&mut stream, "font/ttf", FONT_BYTES);
        font_done.fetch_add(1, Ordering::Relaxed);
    } else if path.starts_with("/p2") {
        p2_served.fetch_add(1, Ordering::Relaxed);
        let body = "<!DOCTYPE html><html><head><title>T27P35 p2</title></head><body>p2</body></html>";
        write_response(&mut stream, "text/html; charset=utf-8", body.as_bytes());
    } else {
        index_served.fetch_add(1, Ordering::Relaxed);
        let body = concat!(
            "<!DOCTYPE html><html><head><title>T27P35 index</title></head>",
            r#"<body style="margin:0">"#,
            r#"<a id="lnk" href="/p2" style="position:fixed;left:10px;top:10px;width:300px;height:60px;"#,
            r#"background:#dde;display:block;line-height:60px">go-p2</a>"#,
            r#"<script>setTimeout(function () {"#,
            r#"  const f = new FontFace('T27P35Font', location.origin + '/slowfont');"#,
            r#"  document.fonts.add(f);"#,
            r#"  f.load().then(function () {}).catch(function () {});"#,
            r#"}, 200);</script>"#,
            "</body></html>"
        );
        write_response(&mut stream, "text/html; charset=utf-8", body.as_bytes());
    }
}

/// 读到请求头结束标记（`\r\n\r\n`）或 4KB 上限；预连接/提前断开返回 None。
fn read_request_head(stream: &mut TcpStream) -> Option<Vec<u8>> {
    let mut head = Vec::new();
    let mut buf = [0_u8; 1024];
    loop {
        if head.windows(4).any(|window| window == b"\r\n\r\n") || head.len() > 4096 {
            return Some(head);
        }
        let read = stream.read(&mut buf).ok()?;
        if read == 0 {
            return None;
        }
        head.extend_from_slice(&buf[..read]);
    }
}

fn write_response(stream: &mut TcpStream, content_type: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

/// 被测浏览器进程（RAII 关停；renderer 随 IPC 管道断连自退出）。
struct BrowserProcess(Child);

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// 启动 zero-browser --headless 直达 `start_url` 并等待 CDP 发现端点就绪
/// （harness 同 cdp_input_events；start_url 直传——导航经渲染进程，冻结窗内
/// CDP Page.navigate 同样被扣压，不作加载手段）。
fn spawn_browser(start_url: &str) -> (u16, BrowserProcess) {
    // 端口预分配：先占住再释放（窗口期极小；发现端点轮询兜底）。
    let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
    let port = probe.local_addr().expect("addr").port();
    drop(probe);

    let storage = std::env::temp_dir().join(format!("zw-t27p35-{}-{}", std::process::id(), port));
    std::fs::create_dir_all(&storage).expect("storage dir");
    let browser_bin = env!("CARGO_BIN_EXE_zero-browser");
    let renderer = std::path::Path::new(browser_bin)
        .parent()
        .expect("browser bin dir")
        .join("zero-renderer");
    assert_renderer_bin_current(&renderer);

    let child = Command::new(browser_bin)
        .arg("--headless")
        .arg(format!("--remote-debugging-port={port}"))
        .arg(start_url)
        .env("ZERO_STORAGE_DIR", &storage)
        .env("ZERO_RENDERER_PATH", &renderer)
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

/// renderer bin spawn 前存在性 + 陈旧断言（同 slice22 T-S3——陈旧 peer-bin 假绿面消除）。
fn assert_renderer_bin_current(renderer: &std::path::Path) {
    assert!(
        renderer.exists(),
        "renderer binary missing at {} — build it explicitly (cargo build -p zero-renderer) before running this suite",
        renderer.display()
    );
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/renderer/src");
    let bin_mtime = std::fs::metadata(renderer)
        .and_then(|m| m.modified())
        .expect("renderer bin mtime readable");
    if let Some(newest) = newest_source_mtime(&src_dir) {
        assert!(
            bin_mtime >= newest,
            "renderer binary at {} is stale (bin mtime {bin_mtime:?} < newest renderer source mtime {newest:?}) — rebuild zero-renderer",
            renderer.display()
        );
    }
}

/// `dir` 下递归最新 `.rs` 源码 mtime（无目录/无源码 → None——跳过陈旧判定）。
fn newest_source_mtime(dir: &std::path::Path) -> Option<std::time::SystemTime> {
    let mut newest: Option<std::time::SystemTime> = None;
    for entry in std::fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        let candidate = if path.is_dir() {
            newest_source_mtime(&path)
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            entry.metadata().and_then(|m| m.modified()).ok()
        } else {
            None
        };
        if let Some(time) = candidate {
            newest = Some(match newest {
                Some(prev) => prev.max(time),
                None => time,
            });
        }
    }
    newest
}

/// 最小 CDP 客户端（同步 ws；每连接独占——s23 钉形态，无 evaluate：冻结窗内该
/// 通道同样被扣压，不作观测面）。
struct CdpClient {
    ws: tungstenite::WebSocket<TcpStream>,
    next_id: u64,
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
        let (ws, _resp) = tungstenite::client(format!("ws://127.0.0.1:{port}"), stream).expect("ws handshake");
        Self { ws, next_id: 0 }
    }

    /// 发送命令并等待响应（读超时 = 断言失败）。
    fn call(&mut self, method: &str, params: Value, timeout: Duration) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let text = serde_json::json!({ "id": id, "method": method, "params": params }).to_string();
        self.ws.send(Message::Text(text.into())).expect("ws send");
        let deadline = Instant::now() + timeout;
        loop {
            assert!(Instant::now() < deadline, "timeout waiting for {method} response");
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let msg: Value = serde_json::from_str(&text).expect("cdp json");
                    if msg.get("id").and_then(Value::as_u64) == Some(id) {
                        return msg;
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

    /// CDP 点击：mousePressed + mouseReleased(clickCount:1)。
    fn click(&mut self, x: f64, y: f64) {
        self.call(
            "Input.dispatchMouseEvent",
            serde_json::json!({ "type": "mousePressed", "x": x, "y": y, "button": "left", "clickCount": 1 }),
            Duration::from_secs(10),
        );
        self.call(
            "Input.dispatchMouseEvent",
            serde_json::json!({ "type": "mouseReleased", "x": x, "y": y, "button": "left", "clickCount": 1 }),
            Duration::from_secs(10),
        );
    }
}

/// 主钉（T27-P35）：在途字体 fetch 冻结主循环期间，点击 → 导航开始须有界。
/// 前提自证：点击时字体臂已开（started ≥1）未完（done =0）——冻结窗确实在途，
/// 前提不成立即失败（防假绿）。观测口径：fixture 服务端 `/p2` 计数增量距点击
/// 发送的耗时 < 8s（红态 ≈13.7s 翻红；绿态实测 ≤3s）。收尾：字体臂最终完成
/// （防「页面根本没加载」的空洞通过）。
#[test]
fn t27p35_click_nav_starts_within_bound_during_font_fetch() {
    let site = DelaySite::spawn();
    let (port, _browser) = spawn_browser(&site.url("/"));
    let mut cdp = CdpClient::connect(port);

    // 等链接页就绪（fixture 口径；主循环内 evaluate 不可用作就绪探测——冻结窗）。
    let deadline = Instant::now() + Duration::from_secs(30);
    while site.index_served.load(Ordering::Relaxed) < 1 {
        assert!(Instant::now() < deadline, "fixture index page not served within 30s");
        std::thread::sleep(Duration::from_millis(50));
    }
    // 字体取在 load+200ms 发起；+1.5s 点击，确保落在冻结窗内。
    std::thread::sleep(Duration::from_millis(1500));
    assert!(
        site.font_started.load(Ordering::Relaxed) >= 1,
        "premise: font fetch must have started before the click"
    );
    assert_eq!(
        site.font_done.load(Ordering::Relaxed),
        0,
        "premise: font fetch must still be in flight (main loop frozen window open)"
    );
    let p2_before = site.p2_served.load(Ordering::Relaxed);

    cdp.click(100.0, 30.0); // 链接热区（left:10 top:10 300x60）
    let clicked_at = Instant::now();

    // /p2 计数增量 = 点击已派发且导航已发起（browser 侧 zero-net 取回）。
    let deadline = clicked_at + Duration::from_secs(60);
    while site.p2_served.load(Ordering::Relaxed) <= p2_before {
        assert!(
            Instant::now() < deadline,
            "fixture /p2 never requested after click within 60s"
        );
        std::thread::sleep(Duration::from_millis(25));
    }
    let nav_start_elapsed = clicked_at.elapsed();
    println!("t27p35: nav start {nav_start_elapsed:?} after click (bound {NAV_START_BOUND:?})");
    assert!(
        nav_start_elapsed < NAV_START_BOUND,
        "click feedback must start navigation within {NAV_START_BOUND:?} while a blocking fetch is in flight \
         (got {nav_start_elapsed:?}) — pre-fix shape parks the click until the {FONT_DELAY:?} font fetch completes"
    );

    // 收尾前提：字体臂最终完成（否则上述测量发生在页面未加载的空洞上）。
    let deadline = Instant::now() + Duration::from_secs(60);
    while site.font_done.load(Ordering::Relaxed) < 1 {
        assert!(Instant::now() < deadline, "font fetch never completed within 60s");
        std::thread::sleep(Duration::from_millis(50));
    }
}
