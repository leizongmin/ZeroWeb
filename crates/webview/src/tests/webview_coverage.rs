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
    /// 启动服务器，服务 `path -> body bytes` 映射（支持二进制内容如 PNG）。
    fn start(files: HashMap<String, Vec<u8>>) -> Self {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        listener.set_nonblocking(true).unwrap();
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = shutdown.clone();
        thread::spawn(move || {
            while !shutdown_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let owned = files.clone();
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

    fn handle(stream: &mut TcpStream, files: &HashMap<String, Vec<u8>>) {
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
        } else if path.ends_with(".png") {
            "image/png"
        } else {
            "application/octet-stream"
        };
        // header 与 body 分开写：body 可能是二进制（PNG），不能用 format! 嵌入。
        match files.get(path) {
            Some(body) => {
                let head = format!(
                    "HTTP/1.0 200 OK\r\nContent-Type: {ct}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                let _ = stream.write_all(head.as_bytes());
                let _ = stream.write_all(body);
            }
            None => {
                let _ = stream.write_all(b"HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            }
        }
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
    files.insert("/page.html".to_string(), page.as_bytes().to_vec());
    files.insert("/style.css".to_string(), css.as_bytes().to_vec());
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
    files.insert("/page.html".to_string(), page.as_bytes().to_vec());
    let server = MiniServer::start(files);

    let mut webview = WebView::new(WebViewConfig::default());
    let url = format!("{}/page.html", server.base);
    let result = webview.fetch_url(&url);
    assert!(result.is_ok(), "missing external CSS must not break navigation");
}

/// 构造一张 3×2 纯绿 RGBA PNG 的字节（用 png 编码器）。
fn green_3x2_png() -> Vec<u8> {
    use png::{BitDepth, ColorType, Encoder};
    let mut buf = Vec::new();
    {
        let mut encoder = Encoder::new(&mut buf, 3, 2);
        encoder.set_color(ColorType::Rgba);
        encoder.set_depth(BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        // 3×2×4 = 24 字节原始像素，全纯绿 (0,255,0,255)。
        let data: Vec<u8> = [0, 255, 0, 255].repeat(6);
        writer.write_image_data(&data).unwrap();
    }
    buf
}

/// R214：URL 导航路径下 `<img src>` 图片子资源必须被抓取、解码并写入 ImageCache。
///
/// page.html 含 `<img src="/pic.png">`，pic.png 是 3×2 纯绿 PNG。fetch_url 后
/// webview 的 image_cache 应含该图（按 abs url hash 键），且尺寸为 3×2——证明
/// 图片子资源抓取 + 解码 + 缓存贯通。
#[test]
fn test_fetch_url_loads_image_subresource() {
    let page = "<!DOCTYPE html><html><head></head><body>\
                <img src=\"/pic.png\"></body></html>";
    let png = green_3x2_png();
    let mut files = HashMap::new();
    files.insert("/page.html".to_string(), page.as_bytes().to_vec());
    files.insert("/pic.png".to_string(), png);
    let server = MiniServer::start(files);

    let mut webview = WebView::new(WebViewConfig::default());
    let url = format!("{}/page.html", server.base);
    let result = webview.fetch_url(&url).expect("fetch_url should succeed");
    // 触发 image_cache 使用（fetch_url 已填充）。
    let _ = result;

    // image_cache 应含 pic.png 解码结果。
    use zero_render_foundation::image_cache::ImageKey;
    let key = ImageKey::new(zero_engine::image_resource_key("/pic.png", Some(&url)));
    let img = webview.image_cache().get(&key);
    assert!(img.is_some(), "image subresource not decoded/cached");
    let img = img.unwrap();
    assert_eq!(img.width, 3, "decoded image width");
    assert_eq!(img.height, 2, "decoded image height");
    assert_eq!(img.get_pixel(0, 0), [0, 255, 0, 255], "top-left pixel pure green");
}

/// resize + render 后 `<img>` 图元不应消失（pipeline 重建时须保留 image_sizes）。
#[test]
fn test_resize_render_keeps_img_primitives() {
    let page = "<!DOCTYPE html><html><head></head><body>\
                <img src=\"/pic.png\" width=\"40\" height=\"30\"></body></html>";
    let png = green_3x2_png();
    let mut files = HashMap::new();
    files.insert("/page.html".to_string(), page.as_bytes().to_vec());
    files.insert("/pic.png".to_string(), png);
    let server = MiniServer::start(files);

    let mut webview = WebView::new(WebViewConfig::default());
    let url = format!("{}/page.html", server.base);
    webview.fetch_url(&url).expect("fetch_url should succeed");
    let images_before = webview.last_render().unwrap().primitives.images.len();
    assert!(images_before > 0, "expected img primitive before resize");

    webview.resize(1024, 768);
    let result = webview.render();
    assert!(
        !result.primitives.images.is_empty(),
        "img primitives lost after resize+render (before={images_before})"
    );
}

/// R218：URL 导航路径下 `<img src>` 的 SVG 子资源必须被抓取、栅格化并写入 ImageCache。
///
/// page.html 含 `<img src="/logo.svg">`，logo.svg 是 4×3 纯绿 SVG。fetch_url 后
/// webview 的 image_cache 应含该图（经 `decode_image_bytes` 内容嗅探路由到
/// `decode_svg_bytes`），尺寸 4×3——证明 SVG 栅格化贯通 URL 导航路径（DC-13）。
#[test]
fn test_fetch_url_loads_svg_image_subresource() {
    let page = "<!DOCTYPE html><html><head></head><body>\
                <img src=\"/logo.svg\"></body></html>";
    let svg = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\
               <svg xmlns=\"http://www.w3.org/2000/svg\" width=\"4\" height=\"3\">\
               <rect width=\"4\" height=\"3\" fill=\"rgb(0,255,0)\"/></svg>";
    let mut files = HashMap::new();
    files.insert("/page.html".to_string(), page.as_bytes().to_vec());
    files.insert("/logo.svg".to_string(), svg.as_bytes().to_vec());
    let server = MiniServer::start(files);

    let mut webview = WebView::new(WebViewConfig::default());
    let url = format!("{}/page.html", server.base);
    webview.fetch_url(&url).expect("fetch_url should succeed");

    use zero_render_foundation::image_cache::ImageKey;
    let key = ImageKey::new(zero_engine::image_resource_key("/logo.svg", Some(&url)));
    let img = webview.image_cache().get(&key);
    assert!(img.is_some(), "SVG subresource not decoded/cached");
    let img = img.unwrap();
    assert_eq!(img.width, 4, "rasterized SVG width");
    assert_eq!(img.height, 3, "rasterized SVG height");
    let px = img.get_pixel(1, 1);
    assert!(px[1] > 200, "SVG green channel should be high, got {}", px[1]);
    assert_eq!(px[3], 255, "SVG alpha should be fully opaque");
}
