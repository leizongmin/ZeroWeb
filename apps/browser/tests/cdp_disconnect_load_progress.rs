//! CDP 客户端断开后载入推进不停摆钉（T93 headless 消息泵解耦）。
//!
//! 既有缺陷（T92 diag3 单变量实锤）：renderer 事件泵（FetchResponse 消费 +
//! renderer 消息处理）只活在活连接 WS 读 WouldBlock 分支（S12 只修「命令间歇
//! 期」）。CDP 客户端断开（read error/Close → `conn.alive=false` → retain 移
//! 除）后零连接，session 无人消费 renderer 侧 FetchResponse → inflight fetch
//! 永不落定 → 载入推进停摆 → `finish_page_load`（nav complete +
//! DOMContentLoaded/load lifecycle 派发）永不执行——headless/自动化场景异常断
//! 开使页面卡在载入中态。
//!
//! 钉面（真实进程：zero-browser + zero-renderer + 本地静态 HTTP，diag3 形制
//! 常驻化；断言走 `ZW_JS_WORKER_CENSUS` sink 的 `nav complete` 行——非 CDP 通
//! 道，连接存活与否不影响观察）：
//! 1. `nav_complete_lands_after_client_disconnect`——导航提交后客户端断开，
//!    nav complete 仍须落盘（base 上恒超时红，修复后绿）。
//! 2. `nav_complete_lands_with_connection_alive`——负控制：连接保持时 nav
//!    complete 正常落盘（守卫 TestSite/sink 链路自身正确性）。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::Value;
use tungstenite::Message;

/// CDP 强制 token 面的测试常量：spawn 显式传入、连接侧 query 携带（s21 同款）。
const TEST_CDP_TOKEN: &str = "zw-integ-test-cdp-token";

/// nav complete 落盘等待上限：慢脚本 2s + 载入收尾裕量。
const NAV_COMPLETE_CAP: Duration = Duration::from_secs(15);

/// 测试页服务：线程化静态 HTTP（进程生命周期 = 测试函数）。
/// `/` = 含 parser 阻塞外链慢脚本的页面；`/slowscript?ms=<n>` = 延迟应答脚本。
struct TestSite {
    port: u16,
    #[allow(dead_code)]
    shutdown: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl TestSite {
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
                let (content_type, body) = match path.split('?').next().unwrap_or("/") {
                    "/" => (
                        "text/html",
                        "<!DOCTYPE html><html><head><title>S93 Slow</title></head><body>\
                         <p>ready</p>\
                         <script src=\"/slowscript?ms=2000\"></script>\
                         </body></html>"
                            .to_string(),
                    ),
                    p if p.starts_with("/slowscript") => {
                        let ms = path
                            .split("ms=")
                            .nth(1)
                            .and_then(|v| v.parse::<u64>().ok())
                            .unwrap_or(2000);
                        std::thread::sleep(Duration::from_millis(ms));
                        ("text/javascript", "// slow script\n".to_string())
                    }
                    _ => ("text/plain", "not found".to_string()),
                };
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\n\
                     Content-Length: {}\r\nConnection: close\r\n\r\n{body}",
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

struct BrowserProcess(Child);

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// 启动 zero-browser --headless，`ZW_JS_WORKER_CENSUS` 指向测试私有 sink 文件
///（renderer 子进程继承 env；sink 模式（值为路径）启用 T91 导航相 mark 行）。
/// 返回（CDP 端口, sink 路径, 进程句柄）。
fn spawn_browser() -> (u16, PathBuf, BrowserProcess) {
    // 端口预分配：先占住再释放（窗口期极小；发现端点轮询兜底，s21 同款）。
    let probe = TcpListener::bind("127.0.0.1:0").expect("bind probe port");
    let port = probe.local_addr().expect("addr").port();
    drop(probe);

    let storage = std::env::temp_dir().join(format!("zw-s93-pin-{}-{}", std::process::id(), port));
    std::fs::create_dir_all(&storage).expect("storage dir");
    let sink = storage.join("census.sink");
    // renderer 与 browser 同 target 目录（dev profile），取同级（#12：测试前须
    // 显式构建两 bin，spawn 陈旧二进制产稳定假绿/假红）。
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
        .env("ZERO_RENDERER_PATH", &renderer)
        .env("ZW_JS_WORKER_CENSUS", &sink)
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
    (port, sink, BrowserProcess(child))
}

/// 最小 CDP 客户端（同步 ws，s21 形制精简：命令→响应，途经事件跳过）。
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
        let (ws, _resp) = tungstenite::client(format!("ws://127.0.0.1:{port}/?auth_token={TEST_CDP_TOKEN}"), stream)
            .expect("ws handshake");
        Self { ws, next_id: 0 }
    }

    /// 发送命令并等同名响应（读超时窗内轮询；超时即断言失败）。
    fn call(&mut self, method: &str, params: Value, timeout: Duration) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let text = serde_json::json!({ "id": id, "method": method, "params": params }).to_string();
        self.ws.send(Message::Text(text.into())).expect("ws send");
        let deadline = Instant::now() + timeout;
        loop {
            assert!(Instant::now() < deadline, "timeout waiting for response: {method}");
            match self.ws.read() {
                Ok(Message::Text(text)) => {
                    let msg: Value = serde_json::from_str(&text).expect("json");
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
}

/// sink 中 `nav complete` 行计数（基线判读用——启动 about:blank 载入也可能落
/// 一条 complete，存在性判断会假绿，必须数增量）。
fn count_nav_complete(sink: &Path) -> usize {
    std::fs::read_to_string(sink)
        .unwrap_or_default()
        .lines()
        .filter(|line| line.starts_with("nav complete"))
        .count()
}

/// 轮询 sink 文件直到 `nav complete` 行数超过 `baseline`（T91 导航相 mark，
/// page_scripts finish_page_load 收尾链发射——断连楔卡死的落点本身）。返回
/// 耗时；超时把 sink 尾部带进失败信息（sink 经即时 write_all 落盘，无行 =
/// 推进确实停摆，不是读缓存问题）。
fn wait_for_nav_complete(sink: &Path, baseline: usize, cap: Duration) -> Duration {
    let start = Instant::now();
    loop {
        let elapsed = start.elapsed();
        assert!(
            elapsed < cap,
            "nav complete not landed within {cap:?} (wedge regression?)"
        );
        let content = std::fs::read_to_string(sink).unwrap_or_default();
        if content.lines().filter(|line| line.starts_with("nav complete")).count() > baseline {
            return elapsed;
        }
        if elapsed > Duration::from_secs(3) && elapsed < Duration::from_secs(4) {
            // 诊断面：3s 后仍未闭合时打一次 sink 尾部（现场可读）。
            let tail: Vec<&str> = content.lines().rev().take(5).collect();
            eprintln!("[s93] sink tail @3s: {tail:?}");
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}

/// 钉 1：导航提交后客户端断开，nav complete 仍须落盘（diag3 形制常驻化）。
/// 断言走 sink 而非 CDP：重连会恢复 WouldBlock 泵，掩盖停摆面。
#[test]
fn nav_complete_lands_after_client_disconnect() {
    let site = TestSite::spawn();
    let (port, sink, _browser) = spawn_browser();
    let baseline = count_nav_complete(&sink);

    let mut conn = CdpClient::connect(port);
    conn.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/") }),
        Duration::from_secs(10),
    );
    // 400ms：页面已提交、外链脚本 fetch 已在途（慢脚本 2s，窗口落在载入中段）。
    std::thread::sleep(Duration::from_millis(400));

    // 客户端硬断开：drop 不发 Close 帧——宿主侧 TCP EOF/reset →
    // tungstenite `ResetWithoutClosingHandshake`（T92 腿窗观测形态同面）
    // → conn.alive=false → retain 移除 → 零连接。
    drop(conn);

    let content = std::fs::read_to_string(&sink).unwrap_or_default();
    assert!(
        content.lines().any(|line| line.starts_with("nav begin navigate")),
        "nav begin mark missing before disconnect — sink chain broken"
    );

    let elapsed = wait_for_nav_complete(&sink, baseline, NAV_COMPLETE_CAP);
    println!("nav complete landed {elapsed:?} after client disconnect");
}

/// 钉 2（负控制）：连接保持时 nav complete 正常落盘——守卫 TestSite/sink
/// 链路自身正确性（钉 1 的红只可能来自断连楔，不来自页面加载失败）。
#[test]
fn nav_complete_lands_with_connection_alive() {
    let site = TestSite::spawn();
    let (port, sink, _browser) = spawn_browser();
    let baseline = count_nav_complete(&sink);

    let mut conn = CdpClient::connect(port);
    conn.call(
        "Page.navigate",
        serde_json::json!({ "url": site.url("/") }),
        Duration::from_secs(10),
    );

    let elapsed = wait_for_nav_complete(&sink, baseline, NAV_COMPLETE_CAP);
    println!("nav complete landed {elapsed:?} with connection alive");
}
