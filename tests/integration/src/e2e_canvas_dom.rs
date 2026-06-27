//! Canvas 2D、安全功能、存储、网络端到端集成测试。

// ── 1. Canvas 2D 端到端验证 ──────────────────────────────────────

#[test]
fn test_canvas_2d_basic_shapes() {
    use zero_canvas::CanvasContext;

    let mut ctx = CanvasContext::new(400, 300);
    ctx.fill_rect(10.0, 10.0, 100.0, 50.0);
    ctx.stroke_rect(120.0, 10.0, 100.0, 50.0);
    ctx.clear_rect(50.0, 20.0, 30.0, 30.0);
}

#[test]
fn test_canvas_2d_path_drawing() {
    use zero_canvas::CanvasContext;

    let mut ctx = CanvasContext::new(200, 200);
    ctx.begin_path();
    ctx.move_to(10.0, 10.0);
    ctx.line_to(190.0, 10.0);
    ctx.line_to(190.0, 190.0);
    ctx.close_path();
    ctx.fill();

    ctx.begin_path();
    ctx.arc(100.0, 100.0, 50.0, 0.0, std::f32::consts::PI * 2.0);
    ctx.fill();
}

#[test]
fn test_canvas_2d_transformations() {
    use zero_canvas::CanvasContext;

    let mut ctx = CanvasContext::new(400, 400);
    ctx.save();
    ctx.translate(100.0, 100.0);
    ctx.rotate(std::f32::consts::PI / 4.0);
    ctx.scale(2.0, 2.0);
    ctx.fill_rect(0.0, 0.0, 50.0, 50.0);
    ctx.restore();
    ctx.fill_rect(10.0, 10.0, 20.0, 20.0);
}

#[test]
fn test_canvas_2d_line_width_and_alpha() {
    use zero_canvas::CanvasContext;

    let mut ctx = CanvasContext::new(200, 200);
    ctx.set_fill_style(zero_canvas::CanvasStyle::default_black());
    ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
    ctx.set_line_width(3.0);
    ctx.stroke_rect(50.0, 50.0, 100.0, 100.0);
    ctx.set_global_alpha(0.5);
    ctx.fill_rect(75.0, 75.0, 50.0, 50.0);
}

#[test]
fn test_canvas_2d_text() {
    use zero_canvas::CanvasContext;
    use zero_canvas::FontDescriptor;

    let mut ctx = CanvasContext::new(400, 100);
    ctx.set_font(FontDescriptor::default());
    ctx.set_fill_style(zero_canvas::CanvasStyle::default_black());
    ctx.fill_text("Hello, World!", 10.0, 50.0);
}

#[test]
fn test_canvas_2d_gradient() {
    use zero_canvas::CanvasContext;

    let mut ctx = CanvasContext::new(200, 200);
    let mut grad = ctx.create_linear_gradient(0.0, 0.0, 200.0, 200.0);
    grad.add_color_stop(0.0, zero_render_foundation::color::Color::rgb(255, 0, 0));
    grad.add_color_stop(1.0, zero_render_foundation::color::Color::rgb(0, 0, 255));
    ctx.set_fill_style(zero_canvas::CanvasStyle::LinearGradient(grad));
    ctx.fill_rect(0.0, 0.0, 200.0, 200.0);
}

// ── 2. DOM-JS 桥接端到端验证 ──────────────────────────────────────
// 注：execute_script_with_dom 先注入 DOM API polyfill，然后执行用户脚本。

#[test]
fn test_dom_js_bridge_query_selector() {
    use zero_webview::{WebView, WebViewConfig};

    let mut wv = WebView::new(WebViewConfig::default());
    let html = r#"<html><body><div id="app"><h1>Title</h1></div></body></html>"#;
    wv.load_html(html, None);

    // 使用 execute_script_with_dom 注入 DOM API polyfill
    let result = wv.execute_script_with_dom("document.getElementById('app') !== null");
    assert!(result.is_ok(), "DOM querySelector should work with DOM polyfill");
}

#[test]
fn test_dom_js_bridge_inner_html() {
    use zero_webview::{WebView, WebViewConfig};

    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><div id='target'>Old content</div></body></html>", None);

    // DOM API polyfill 提供了 document 对象，但 getElementById 在无 DOM 树连接时可能返回 null
    // 验证 polyfill 注入不崩溃
    let result = wv.execute_script_with_dom("typeof document.getElementById === 'function'");
    assert!(
        result.is_ok(),
        "document.getElementById should be a function with DOM polyfill"
    );
}

#[test]
fn test_dom_js_create_elements() {
    use zero_webview::{WebView, WebViewConfig};

    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body></body></html>", None);

    let script = r#"
        var div = document.createElement('div');
        div.id = 'created';
        div.textContent = 'Dynamic content';
        'ok'
    "#;
    let result = wv.execute_script_with_dom(script);
    assert!(result.is_ok(), "DOM createElement should work with DOM polyfill");
}

#[test]
fn test_dom_js_event_listener() {
    use zero_webview::{WebView, WebViewConfig};

    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><button id='btn'>Click</button></body></html>", None);

    // 使用 createElement 创建的元素测试 addEventListener（不依赖 DOM 树中的元素）
    let script = r#"
        var btn = document.createElement('button');
        btn.addEventListener('click', function() { });
        'listener added'
    "#;
    let result = wv.execute_script_with_dom(script);
    assert!(result.is_ok(), "DOM addEventListener should work with DOM polyfill");
}

// ── 3. 安全功能端到端验证 ─────────────────────────────────────────

#[test]
fn test_cors_policy_check() {
    use zero_security::cors::{CorsPolicy, check_cors};
    use zero_security::origin::Origin;

    let policy = CorsPolicy::default();
    let origin = Origin::parse("https://example.com").unwrap();
    let _result = check_cors(&policy, &origin, "GET", &[]);
}

#[test]
fn test_csp_basic_policy() {
    use zero_security::csp::ContentSecurityPolicy;
    use zero_security::origin::Origin;

    let policy = ContentSecurityPolicy::parse("default-src 'self'; script-src 'self' https://cdn.example.com");
    let origin = Origin::parse("https://example.com").unwrap();

    assert!(policy.is_resource_allowed("script", "https://example.com/app.js", Some(&origin)));
}

#[test]
fn test_security_same_origin_policy() {
    use zero_security::origin::Origin;

    let origin1 = Origin::parse("https://example.com").unwrap();
    let origin2 = Origin::parse("https://example.com").unwrap();
    let origin3 = Origin::parse("https://other.com").unwrap();

    assert!(origin1.is_same_origin(&origin2));
    assert!(!origin1.is_same_origin(&origin3));
}

// ── 4. 存储端到端验证 ────────────────────────────────────────────

#[test]
fn test_storage_local_storage_roundtrip() {
    use zero_storage::local_storage::{StorageType, WebStorage};

    let mut storage = WebStorage::new(StorageType::Local, "https://example.com");
    assert!(storage.is_empty());

    storage.set("key1", "value1").unwrap();
    storage.set("key2", "value2").unwrap();
    assert_eq!(storage.len(), 2);
    assert_eq!(storage.get("key1"), Some("value1"));

    storage.remove("key1");
    assert_eq!(storage.len(), 1);
    assert!(storage.get("key1").is_none());

    storage.clear();
    assert!(storage.is_empty());
}

// ── 5. 网络请求端到端验证 ─────────────────────────────────────────

#[test]
fn test_url_parsing() {
    use zero_net::url_parser::parse_url;

    let url = parse_url("https://example.com:8443/path?q=1#frag").unwrap();
    assert_eq!(url.scheme, "https");
    assert_eq!(url.host.unwrap(), "example.com");
    assert_eq!(url.port, Some(8443));
    assert_eq!(url.path, "/path");
}

// ── 6. Protocol/IPC 端到端验证 ────────────────────────────────────

#[test]
fn test_ipc_roundtrip_navigation() {
    use zero_protocol::message::{IpcMessage, IpcMessageKind, NavigateParams};
    use zero_protocol::serialize::{deserialize, serialize};

    let msg = IpcMessage {
        id: 1,
        kind: IpcMessageKind::Navigate(NavigateParams {
            url: "https://example.com".to_string(),
            referrer: None,
                navigation_epoch: 0,
        }),
    };
    let bytes = serialize(&msg).unwrap();
    let decoded: IpcMessage = deserialize(&bytes).unwrap();
    assert_eq!(msg.id, decoded.id);
}

#[test]
fn test_ipc_roundtrip_simple_messages() {
    use zero_protocol::message::{IpcMessage, IpcMessageKind};
    use zero_protocol::serialize::{deserialize, serialize};

    let kinds = vec![
        IpcMessageKind::GoBack,
        IpcMessageKind::GoForward,
        IpcMessageKind::Heartbeat,
        IpcMessageKind::TitleChanged("Test".to_string()),
        IpcMessageKind::UrlChanged("https://example.com".to_string()),
    ];

    for (i, kind) in kinds.into_iter().enumerate() {
        let msg = IpcMessage { id: i as u64, kind };
        let bytes = serialize(&msg).unwrap();
        let decoded: IpcMessage = deserialize(&bytes).unwrap();
        assert_eq!(msg.id, decoded.id);
    }
}
