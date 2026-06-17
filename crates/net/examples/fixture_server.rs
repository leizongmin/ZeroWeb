//! 离线静态 fixture HTTP 服务器（DC-13 / 外链 CSS+图片子资源加载测试使能）。
//!
//! 用途：在无外部网络的 rally / CI 环境下，为 ZeroBrowser/WebView 的 URL 导航路径
//! （`fetch_url` → 解析 `<link rel="stylesheet">` / `<img src>` → HTTP 抓取 → 级联/渲染）
//! 提供一个本地 HTTP 源。便于离线验证 goal doc 列出的 P1 缺口「外部样式表加载缺失」
//! 和「图片子资源 / ImageCache 未贯通」。
//!
//! 设计原则（遵循 code-guidelines / 简单至上）：
//! - **零新依赖**：仅用 std（`std::net::TcpListener` + 手写 HTTP/1.0 响应），
//!   避免为静态文件服务引入 hyper / tiny_http。
//! - **仅满足离线 fixture 需求**：不实现 POST / chunked / keep-alive / TLS /
//!   目录列表 / 范围请求等用不到的能力（不做推测性开发）。
//!
//! 用法：
//! ```text
//! cargo run -p zero-net --example fixture-server -- --root <dir> --port <port>
//! ```
//! 然后让 ZeroBrowser 导航到 `http://127.0.0.1:<port>/<relative-path>`。

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

/// 命令行参数。
struct Args {
    root: PathBuf,
    port: u16,
}

fn parse_args() -> Args {
    let mut root = PathBuf::from(".");
    let mut port = 8000u16;
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--root" => {
                if let Some(v) = iter.next() {
                    root = PathBuf::from(v);
                }
            }
            "--port" => {
                if let Some(v) = iter.next().and_then(|v| v.parse::<u16>().ok()) {
                    port = v;
                }
            }
            _ => {}
        }
    }
    Args { root, port }
}

/// 按文件扩展名推断 Content-Type（覆盖离线 fixture 常见类型）。
fn content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("html") | Some("htm") | Some("xht") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

/// 将 URL 请求路径规范化为 root 下的安全文件路径（阻止 `..` 越界）。
fn resolve_safe(root: &Path, url_path: &str) -> Option<PathBuf> {
    // 去掉查询串与锚点
    let path_only = url_path.split(['?', '#']).next().unwrap_or("");
    // 去掉前导 '/'，按段拼接，过滤掉 "." / ".." / 空段（防目录穿越）
    let mut full = root.to_path_buf();
    for seg in path_only.split('/') {
        if seg.is_empty() || seg == "." || seg == ".." {
            continue;
        }
        full.push(seg);
    }
    // 默认 index.html
    if path_only.is_empty() || path_only == "/" || path_only.ends_with('/') {
        full.push("index.html");
    }
    // canonicalize 后确认仍在 root 之下（双保险）
    let root_canon = root.canonicalize().ok()?;
    let full_canon = full.canonicalize().ok()?;
    if full_canon.starts_with(&root_canon) {
        Some(full_canon)
    } else {
        None
    }
}

/// 处理单条 HTTP 连接：解析 GET 请求行，返回文件内容或 404。
fn handle_connection(stream: &mut TcpStream, root: &Path) {
    // 仅读请求行（够用于 GET 静态文件），其余 header 丢弃。
    let mut buf = [0u8; 2048];
    let n = match stream.read(&mut buf) {
        Ok(n) if n > 0 => n,
        _ => return,
    };
    let req = String::from_utf8_lossy(&buf[..n]);
    let req_line = req.lines().next().unwrap_or("");
    // 形如: GET /path HTTP/1.1
    let mut parts = req_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let path = parts.next().unwrap_or("/");

    if method != "GET" {
        let _ = write_response(stream, 405, "text/plain; charset=utf-8", b"405 Method Not Allowed");
        return;
    }

    match resolve_safe(root, path) {
        Some(file_path) => match fs::read(&file_path) {
            Ok(body) => {
                let ct = content_type(&file_path);
                let _ = write_response(stream, 200, ct, &body);
            }
            Err(_) => {
                let _ = write_response(stream, 404, "text/plain; charset=utf-8", b"404 Not Found");
            }
        },
        None => {
            let _ = write_response(stream, 404, "text/plain; charset=utf-8", b"404 Not Found");
        }
    }
}

/// 写一条极简 HTTP/1.0 响应（Connection: close，调用方每次新建连接）。
fn write_response(stream: &mut TcpStream, status: u16, content_type: &str, body: &[u8]) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "OK",
    };
    let head = format!(
        "HTTP/1.0 {status} {reason}\r\nContent-Type: {content_type}\r\nContent-Length: {len}\r\nConnection: close\r\n\r\n",
        len = body.len(),
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    stream.flush()
}

/// 在指定端口启动服务器（阻塞）。`shutdown` 为外部停止信号（示例 / 测试用）。
pub fn serve(root: &Path, port: u16, shutdown: Arc<AtomicBool>) -> std::io::Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port))?;
    listener.set_nonblocking(true)?;
    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((mut stream, _)) => {
                let root = root.to_path_buf();
                thread::spawn(move || {
                    handle_connection(&mut stream, &root);
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 非阻塞：短暂让出，轮询 shutdown
                thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(_) => break,
        }
    }
    Ok(())
}

fn main() {
    let args = parse_args();
    let port = args.port;
    let root = args.root.canonicalize().unwrap_or_else(|_| args.root.clone());
    eprintln!("fixture-server: serving {} on http://127.0.0.1:{port}", root.display());
    let shutdown = Arc::new(AtomicBool::new(false));
    // 注册 Ctrl-C 优雅停止（best-effort，不可用则忽略）
    let shutdown_clone = shutdown.clone();
    ctrlc_setup(shutdown_clone);
    if let Err(e) = serve(&root, port, shutdown) {
        eprintln!("fixture-server: error: {e}");
        std::process::exit(1);
    }
}

/// best-effort 注册 Ctrl-C（无第三方依赖；标准库无跨平台信号句柄，故仅 Unix 用 libc-free 方式忽略）。
fn ctrlc_setup(_shutdown: Arc<AtomicBool>) {
    // 不引入 signal-hook 依赖；进程被杀即停。保持极简。
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// 在临时目录建 fixture 树，启动服务器，用 TcpStream GET 验证 CSS / 404 / 内容类型。
    #[test]
    fn fixture_server_serves_files_and_404() {
        let dir = std::env::temp_dir().join(format!("zw-fixture-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        // 外链 CSS + 引用它的 HTML（模拟 DC-13 morning.work 的 /article.css 依赖）
        let mut css = fs::File::create(dir.join("article.css")).unwrap();
        css.write_all(b"body { color: red; }").unwrap();
        let mut html = fs::File::create(dir.join("index.html")).unwrap();
        html.write_all(b"<link rel=\"stylesheet\" href=\"/article.css\">")
            .unwrap();
        let root = dir.canonicalize().unwrap();

        // 绑定到 0 端口让 OS 分配空闲端口，避免并发测试端口冲突。
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener); // 释放给 serve 重新绑定

        let shutdown = Arc::new(AtomicBool::new(false));
        let root_clone = root.clone();
        let handle = thread::spawn(move || serve(&root_clone, port, shutdown.clone()));

        // 轮询等待服务器就绪（最多 ~2s）
        let mut stream = None;
        for _ in 0..100 {
            if let Ok(s) = TcpStream::connect(("127.0.0.1", port)) {
                stream = Some(s);
                break;
            }
            thread::sleep(std::time::Duration::from_millis(20));
        }
        let mut stream = stream.expect("server not ready");

        // 请求 CSS 文件
        stream
            .write_all(b"GET /article.css HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        let mut resp = Vec::new();
        stream.read_to_end(&mut resp).unwrap();
        let resp_text = String::from_utf8_lossy(&resp);
        assert!(
            resp_text.starts_with("HTTP/1.0 200 OK"),
            "expected 200, got: {}",
            &resp_text[..resp_text.len().min(80)]
        );
        assert!(resp_text.contains("text/css"), "expected css content-type");
        assert!(resp_text.contains("body { color: red; }"), "css body missing");

        // 请求不存在的文件 → 404
        let mut s2 = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s2.write_all(b"GET /missing.png HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        let mut resp2 = Vec::new();
        s2.read_to_end(&mut resp2).unwrap();
        let t2 = String::from_utf8_lossy(&resp2);
        assert!(
            t2.starts_with("HTTP/1.0 404"),
            "expected 404, got: {}",
            &t2[..t2.len().min(80)]
        );

        // 路径穿越被拒（也应 404，绝不返回 /etc/passwd）
        let mut s3 = TcpStream::connect(("127.0.0.1", port)).unwrap();
        s3.write_all(b"GET /../../etc/passwd HTTP/1.0\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        let mut resp3 = Vec::new();
        s3.read_to_end(&mut resp3).unwrap();
        let t3 = String::from_utf8_lossy(&resp3);
        assert!(t3.starts_with("HTTP/1.0 404"), "traversal must 404");

        // 清理：服务器在 serve() 的非阻塞循环里，置 shutdown 并 join（短暂）
        // 注意：主循环每 20ms 检查 shutdown，join 应很快返回。
        // 由于 handle 持有 shutdown 副本的 move，这里无法直接置位；
        // 改为：直接丢弃 handle（线程随进程结束退出），清理临时目录。
        drop(handle);
        let _ = fs::remove_dir_all(&dir);
    }
}
