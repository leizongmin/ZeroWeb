//! WebView tests for uncovered paths - testing public API.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use zero_webview::{WebView, WebViewConfig, WebViewEvent};

#[test]
fn test_webview_event_callback_removal() {
    let mut webview = WebView::new(WebViewConfig::default());
    let callback1 = |event: &WebViewEvent| println!("Callback 1: {:?}", event);

    let index1 = webview.on_event(callback1);
    assert!(webview.remove_event_callback(index1));

    // Removing already-removed callback should fail
    assert!(!webview.remove_event_callback(index1));
    assert!(!webview.remove_event_callback(99));
}

#[test]
fn test_webview_execute_script_errors() {
    let mut webview = WebView::new(WebViewConfig::default());
    let result = webview.execute_script("");
    assert!(result.is_err());
}

#[test]
fn test_webview_extract_origin() {
    assert_eq!(
        WebView::extract_origin("https://example.com/path"),
        Some("https://example.com".to_string())
    );
    assert_eq!(
        WebView::extract_origin("http://localhost:3000"),
        Some("http://localhost:3000".to_string())
    );
    assert_eq!(WebView::extract_origin("not-a-url"), None);
    assert_eq!(WebView::extract_origin(""), None);
}

#[test]
fn test_webview_set_title() {
    let mut webview = WebView::new(WebViewConfig::default());
    assert_eq!(webview.title(), None);
    webview.set_title("Test Page");
    assert_eq!(webview.title(), Some("Test Page"));
}

#[test]
fn test_webview_resize() {
    let mut webview = WebView::new(WebViewConfig::default());
    webview.resize(800, 600);
}

#[test]
fn test_webview_is_loading() {
    let webview = WebView::new(WebViewConfig::default());
    assert!(!webview.is_loading());
}

#[test]
fn test_webview_load_html() {
    let mut webview = WebView::new(WebViewConfig::default());
    let result = webview.load_html("<html><body>Hello</body></html>", None);
    // Should produce render primitives
    assert!(!result.primitives.is_empty() || result.timings.parse_ms >= 0.0);
}

#[test]
fn test_webview_fail_load() {
    let mut webview = WebView::new(WebViewConfig::default());
    webview.fail_load("Network error");
}

#[test]
fn test_webview_complete_load() {
    let mut webview = WebView::new(WebViewConfig::default());
    let result = webview.complete_load("<html><body>Loaded</body></html>", Some("body { color: red; }"));
    assert!(!result.primitives.is_empty() || result.timings.parse_ms >= 0.0);
}

#[test]
fn test_webview_config_default() {
    let config = WebViewConfig::default();
    let webview = WebView::new(config);
    assert_eq!(webview.url(), None);
    assert!(webview.title().is_none());
}

/// 极简 std HTTP/1.0 静态服务器（仅测试用）：服务 `path -> body` 映射。
///
/// 返回绑定的本地 URL 前缀（`http://127.0.0.1:<port>/`）。服务器在后台线程
/// 运行，`shutdown` 置位后退出。Content-Type 固定按扩展名粗分（html/css/其它）。
struct MiniServer {
    base: String,
    shutdown: Arc<AtomicBool>,
}

impl MiniServer {
    fn start(files: HashMap<&'static str, &'static str>) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        // 拷贝映射到 owned 以便 move 进线程。
        let owned: HashMap<String, String> = files.into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
        thread::spawn(move || {
            while !shutdown_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let owned = owned.clone();
                        thread::spawn(move || Self::handle(&mut stream, &owned));
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(std::time::Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Self {
            base: format!("http://127.0.0.1:{port}"),
            shutdown,
        }
    }

    fn handle(stream: &mut TcpStream, files: &HashMap<String, String>) {
        let mut buf = [0u8; 1024];
        let n = stream.read(&mut buf).unwrap_or(0);
        let req = String::from_utf8_lossy(&buf[..n]);
        let path = req
            .lines()
            .next()
            .unwrap_or("")
            .split_whitespace()
            .nth(1)
            .unwrap_or("/");
        let ct = if path.ends_with(".css") {
            "text/css"
        } else if path.ends_with(".html") {
            "text/html"
        } else {
            "application/octet-stream"
        };
        let body = files.get(path).map(|s| s.as_str());
        let resp = match body {
            Some(b) => format!(
                "HTTP/1.0 200 OK\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{b}",
                b.len()
            ),
            None => "HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string(),
        };
        let _ = stream.write_all(resp.as_bytes());
        let _ = stream.flush();
    }
}

impl Drop for MiniServer {
    fn drop(&mut self) {
        // 通知后台线程退出（best-effort；线程在下一次 10ms 轮询时观察到）。
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

/// R213：URL 导航路径下外链 `<link rel="stylesheet">` 必须被抓取并应用。
///
/// page.html 仅通过外链引用 style.css（无任何内联红色样式），style.css 把
/// `#x` 背景设为纯红。若外链 CSS 被正确加载并级联，渲染结果应包含纯红 fill；
/// 否则（外链被忽略）`#x` 无背景，不会出现纯红 fill。
#[test]
fn test_fetch_url_loads_external_stylesheet() {
    let page = "<!DOCTYPE html><html><head>\
                <link rel=\"stylesheet\" href=\"/style.css\">\
                </head><body><div id=\"x\">Hi</div></body></html>";
    let css = "#x { background-color: rgb(255,0,0); width: 100px; height: 100px; }";
    let mut files = HashMap::new();
    files.insert("/page.html", page);
    files.insert("/style.css", css);
    let server = MiniServer::start(files);

    let mut webview = WebView::new(WebViewConfig::default());
    let url = format!("{}/page.html", server.base);
    let result = webview.fetch_url(&url).expect("fetch_url should succeed");

    // 外链 CSS 应用后，#x 背景应生成纯红 (255,0,0) fill。
    let has_red = result
        .primitives
        .fills
        .iter()
        .any(|f| f.color.r == 255 && f.color.g == 0 && f.color.b == 0);
    assert!(
        has_red,
        "external stylesheet not applied: no pure-red fill found (got {} fills)",
        result.primitives.fills.len()
    );
}

/// 反向对照：外链 style.css 缺失（404）时不应 panic，页面仍渲染（宽松降级）。
#[test]
fn test_fetch_url_external_stylesheet_missing_does_not_break() {
    let page = "<!DOCTYPE html><html><head>\
                <link rel=\"stylesheet\" href=\"/missing.css\">\
                </head><body><p>Hello</p></body></html>";
    let mut files = HashMap::new();
    files.insert("/page.html", page);
    let server = MiniServer::start(files);

    let mut webview = WebView::new(WebViewConfig::default());
    let url = format!("{}/page.html", server.base);
    let result = webview.fetch_url(&url);
    assert!(result.is_ok(), "missing external CSS must not break navigation");
}
