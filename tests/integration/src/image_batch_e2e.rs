//! P14/TV-F1 真进程端到端钉：renderer 发布路径两阶段批量图片取回的并发回归。
//!
//! 真实 `zero-browser --headless`（拉起真实 `zero-renderer` 子进程）加载本地
//! fixture 页面（36 张同源图）；修复后发布路径先非阻塞 issue 全部缺图，browser
//! 侧 zero-net 调度器按每来源上限并发展开。双通道观测在途峰值：
//!
//! 1. fixture 服务端在途计数（读到 `/img-*` 请求 → 响应写毕）；
//! 2. browser stderr 的 zero-net `HTTP fetch start/done` 事件序重建
//!    （t6-p14 `measure.py` 口径：剥 ANSI 后按事件序扫深度，见
//!    `.acceptance/site-optimizer/perf-mem-20261023/bench/logs/t6-p14/measure.py`）。
//!
//! 峰值 ≥2 区分「发布路径批量」与回退逐图阻塞串行（max-in-flight 恒 1——旧实现
//! 或 runtime.rs 接线回退时本钉翻红，正是 TV-F1 要求的可察性）；≤ 每来源上限防
//! 调度器越界。不钉 ==上限：饱和判据由 zero-net 调度器级用例与
//! `network_loading::same_origin_image_burst_*` 区间钉负责（TV-F3/DF-F2）。
//!
//! 前置：workspace 共享 target 目录需有 `debug/zero-browser` 与
//! `debug/zero-renderer`（`cargo build -p zero-browser -p zero-renderer`），
//! 从测试可执行文件位置解析。

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use zero_net::resource_policy::max_connections_per_origin;

/// fixture 图片数（与一次性手工证据 t6-p14 同规模：>2×默认每来源上限）。
const IMAGE_COUNT: usize = 36;
/// 每张图片响应前的持槽时长：显著大于并发爬升间隔，在途峰值方可观测。
const HOLD: Duration = Duration::from_millis(150);
/// 4x4 PNG（t6-p14 fixture 同源字节，renderer 解码路径已验证）。
const PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00,
    0x04, 0x00, 0x00, 0x00, 0x04, 0x08, 0x02, 0x00, 0x00, 0x00, 0x26, 0x93, 0x09, 0x29, 0x00, 0x00, 0x00, 0x0c, 0x49,
    0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x60, 0x20, 0x1d, 0x00, 0x00, 0x00, 0x34, 0x00, 0x01, 0x76, 0x5e, 0xae, 0xc3,
    0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

fn index_html() -> String {
    let mut html = String::from("<!DOCTYPE html><html><head><title>t6-e2e</title></head><body>");
    for index in 0..IMAGE_COUNT {
        html.push_str(&format!(
            "<img src=\"img-{index}.png\" width=\"16\" height=\"16\" alt=\"i{index}\">"
        ));
    }
    html.push_str("</body></html>");
    html
}

/// 本地 fixture：线程化 HTTP，每连接一线程 + 读超时——浏览器会开预连接（不发
/// 数据），单线程同步读会卡死 accept 循环（仓库既有先例：cdp_* 测试 TestSite、
/// `network_loading` 突发 fixture）。
struct BurstSite {
    url: String,
    img_in_flight: Arc<AtomicUsize>,
    img_max_in_flight: Arc<AtomicUsize>,
    img_served: Arc<AtomicUsize>,
    html_served: Arc<AtomicUsize>,
    shutdown: Arc<AtomicBool>,
    accept_loop: Option<thread::JoinHandle<()>>,
}

impl BurstSite {
    fn spawn() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture");
        let port = listener.local_addr().expect("fixture address").port();
        listener.set_nonblocking(true).expect("nonblocking fixture");
        let img_in_flight = Arc::new(AtomicUsize::new(0));
        let img_max_in_flight = Arc::new(AtomicUsize::new(0));
        let img_served = Arc::new(AtomicUsize::new(0));
        let html_served = Arc::new(AtomicUsize::new(0));
        let shutdown = Arc::new(AtomicBool::new(false));
        let accept_loop = {
            let img_in_flight = Arc::clone(&img_in_flight);
            let img_max_in_flight = Arc::clone(&img_max_in_flight);
            let img_served = Arc::clone(&img_served);
            let html_served = Arc::clone(&html_served);
            let shutdown = Arc::clone(&shutdown);
            thread::spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let img_in_flight = Arc::clone(&img_in_flight);
                            let img_max_in_flight = Arc::clone(&img_max_in_flight);
                            let img_served = Arc::clone(&img_served);
                            let html_served = Arc::clone(&html_served);
                            // 每连接一线程：读有超时且不得阻塞 accept 循环。
                            thread::spawn(move || {
                                serve(stream, &img_in_flight, &img_max_in_flight, &img_served, &html_served);
                            });
                        }
                        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                            thread::sleep(Duration::from_millis(2));
                        }
                        Err(error) => panic!("fixture accept failed: {error}"),
                    }
                }
            })
        };
        Self {
            url: format!("http://127.0.0.1:{port}"),
            img_in_flight,
            img_max_in_flight,
            img_served,
            html_served,
            shutdown,
            accept_loop: Some(accept_loop),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.url)
    }
}

impl Drop for BurstSite {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(accept_loop) = self.accept_loop.take() {
            let _ = accept_loop.join();
        }
    }
}

/// 单连接服务：图片请求持槽 [`HOLD`] 后应答（Connection: close 逐连接释放，
/// 与 `network_loading` 突发 fixture 同型）；其余路径按 HTML 页面应答。
fn serve(
    mut stream: TcpStream,
    img_in_flight: &AtomicUsize,
    img_max_in_flight: &AtomicUsize,
    img_served: &AtomicUsize,
    html_served: &AtomicUsize,
) {
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(5))).ok();
    let Some(head) = read_request_head(&mut stream) else {
        return;
    };
    let request = String::from_utf8_lossy(&head);
    let path = request.split_whitespace().nth(1).unwrap_or("/");
    if path.starts_with("/img-") {
        img_in_flight.fetch_add(1, Ordering::Relaxed);
        img_max_in_flight.fetch_max(img_in_flight.load(Ordering::Relaxed), Ordering::Relaxed);
        thread::sleep(HOLD);
        write_response(&mut stream, "image/png", PNG);
        img_in_flight.fetch_sub(1, Ordering::Relaxed);
        img_served.fetch_add(1, Ordering::Relaxed);
    } else {
        write_response(&mut stream, "text/html; charset=utf-8", index_html().as_bytes());
        html_served.fetch_add(1, Ordering::Relaxed);
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

impl BrowserProcess {
    fn spawn(start_url: &str, stderr_log: &Path) -> Self {
        let browser_bin = workspace_bin("zero-browser");
        let renderer_bin = workspace_bin("zero-renderer");
        // 端口预分配：headless CDP 端点默认 9222，显式占空闲端口防并行/残留冲突。
        let probe = TcpListener::bind("127.0.0.1:0").expect("bind cdp probe port");
        let cdp_port = probe.local_addr().expect("cdp probe address").port();
        drop(probe);
        let storage = std::env::temp_dir().join(format!("zw-t6-e2e-storage-{}-{cdp_port}", std::process::id()));
        std::fs::create_dir_all(&storage).expect("storage dir");
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(stderr_log)
            .expect("open browser stderr log");
        let child = Command::new(&browser_bin)
            .arg("--headless")
            .arg(format!("--remote-debugging-port={cdp_port}"))
            .arg(start_url)
            .env("ZERO_STORAGE_DIR", &storage)
            .env("ZERO_RENDERER_PATH", &renderer_bin)
            .stdout(Stdio::null())
            .stderr(Stdio::from(log_file))
            .spawn()
            .unwrap_or_else(|error| panic!("spawn {} failed: {error}", browser_bin.display()));
        Self(child)
    }
}

impl Drop for BrowserProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// 定位 workspace 共享 target 目录下的二进制：测试可执行文件位于
/// `<target>/debug/deps/`，上两级即 `<target>/debug`。
fn workspace_bin(name: &str) -> PathBuf {
    let exe = std::env::current_exe().expect("current test executable");
    exe.ancestors()
        .nth(2)
        .map(|dir| dir.join(name))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| {
            panic!("{name} not found next to the test executable — build first: cargo build -p zero-browser -p zero-renderer")
        })
}

/// 从 browser stderr 日志重建图片取回在途峰值（t6-p14 measure.py 口径：剥 ANSI
/// 后按 `HTTP fetch start/done` 事件序扫深度——zero-net 调度器日志在 browser
/// 进程内逐事件单次 write 落盘，事件序即时间序）。返回 (img start 数, img done
/// 数, img 在途峰值)。
fn parse_fetch_events(log_path: &Path) -> (usize, usize, usize) {
    let raw = std::fs::read_to_string(log_path).expect("read browser stderr log");
    let mut img_starts = 0_usize;
    let mut img_dones = 0_usize;
    let mut depth = 0_usize;
    let mut peak = 0_usize;
    for line in raw.lines() {
        let line = strip_ansi(line);
        if let Some(rest) = line.split("HTTP fetch start url=").nth(1)
            && rest.contains("/img-")
        {
            img_starts += 1;
            depth += 1;
            peak = peak.max(depth);
        } else if let Some(rest) = line.split("HTTP fetch done url=").nth(1)
            && rest.contains("/img-")
        {
            img_dones += 1;
            depth = depth.saturating_sub(1);
        }
    }
    (img_starts, img_dones, peak)
}

/// 剥 ANSI SGR 序列（tracing fmt 落盘带色，measure.py 同款处理）。
fn strip_ansi(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(current) = chars.next() {
        if current != '\x1b' {
            out.push(current);
            continue;
        }
        // ESC 后应为 '[' 参数序列、终止于 'm'（tracing fmt 仅发 SGR）。
        if chars.peek() == Some(&'[') {
            let _ = chars.next();
            for follow in chars.by_ref() {
                if follow == 'm' {
                    break;
                }
            }
        } else {
            out.push(current);
        }
    }
    out
}

/// 主钉（TV-F1）：renderer 发布路径批量取回在真进程内并发展开——fixture 服务端
/// 与 zero-net 日志两口径的图片在途峰值都必须 ≥2（旧逐图串行形态恒 1，翻红），
/// 且 ≤ 每来源上限；36 张图全部取回、无丢失。
#[test]
fn renderer_publish_batch_image_fetch_reaches_concurrent_in_flight() {
    let site = BurstSite::spawn();
    let log_path = std::env::temp_dir().join(format!("zw-t6-e2e-{}-stderr.log", std::process::id()));
    let _browser = BrowserProcess::spawn(&site.url("/index.html"), &log_path);

    // 等 36 张图全部应答完毕（fixture 口径；超时视为页面未完成加载）。
    let deadline = Instant::now() + Duration::from_secs(60);
    while site.img_served.load(Ordering::Relaxed) < IMAGE_COUNT {
        assert!(
            Instant::now() < deadline,
            "fixture images not fully served within 60s (served={})",
            site.img_served.load(Ordering::Relaxed)
        );
        thread::sleep(Duration::from_millis(50));
    }
    // 尾窗：让 browser 侧 done 日志与调度器收尾落地，再关停进程读日志。
    thread::sleep(Duration::from_millis(500));

    let limit = max_connections_per_origin();
    assert!(
        site.html_served.load(Ordering::Relaxed) >= 1,
        "fixture page must be loaded at least once"
    );
    let served = site.img_served.load(Ordering::Relaxed);
    assert_eq!(served, IMAGE_COUNT, "every image must be fetched exactly once");
    let server_peak = site.img_max_in_flight.load(Ordering::Relaxed);
    assert!(
        server_peak >= 2,
        "same-origin images must be fetched concurrently through the real renderer publish \
         path (server-observed peak={server_peak}); pre-fix serial shape peaks at 1"
    );
    assert!(
        server_peak <= limit,
        "server-observed peak {server_peak} exceeds per-origin limit {limit}"
    );

    let (img_starts, img_dones, log_peak) = parse_fetch_events(&log_path);
    assert_eq!(img_starts, IMAGE_COUNT, "log fetch-start count mismatch");
    assert_eq!(img_dones, IMAGE_COUNT, "log fetch-done count mismatch");
    assert!(
        log_peak >= 2,
        "log-reconstructed max-in-flight must exceed serial shape (peak={log_peak})"
    );
    assert!(
        log_peak <= limit,
        "log-reconstructed peak {log_peak} exceeds per-origin limit {limit}"
    );
    println!("t6 e2e image peaks: server={server_peak} log={log_peak} limit={limit} served={served}");

    let _ = std::fs::remove_file(&log_path);
}
