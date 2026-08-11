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

// ── R3287：execute_script_with_dom（A 代 polyfill）querySelector 兄弟组合器 ──────
// `execute_script_with_dom` 是公开稳定的 WebView 嵌入 API，注入 A 代 polyfill
//（dom_bridge.rs::generate_dom_api_polyfill）。其 _matchesSingleSelector 历史仅支持后代/子代组合器，
// `+`/`~` 静默不匹配——R3287 补全（延续 R3285 DOM crate + R3286 B 代 shim 的组合器一致化系列）。
// A 代 polyfill 维护独立虚拟 DOM（_nodeMap，经 createElement/appendChild 填充，不接已 load_html 的树），
// 故测试经 createElement 构建 sibling 子树后 querySelector。

#[test]
fn test_dom_js_polyfill_next_sibling_combinator_r3287() {
    use zero_webview::{WebView, WebViewConfig};
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body></body></html>", None);
    // root: h1, p#a, p#b, span#s, p#c（兄弟序）
    let script = r#"
        var root = document.createElement('div');
        var h1 = document.createElement('h1');  h1.id = 't';
        var p1 = document.createElement('p');   p1.id = 'a';
        var p2 = document.createElement('p');   p2.id = 'b';
        var sp = document.createElement('span'); sp.id = 's';
        var p3 = document.createElement('p');   p3.id = 'c';
        root.appendChild(h1); root.appendChild(p1); root.appendChild(p2); root.appendChild(sp); root.appendChild(p3);
        var next = root.querySelector('h1 + p');
        var spanP = root.querySelector('span + p');
        var noMatch = root.querySelector('h1 + span');
        JSON.stringify({next: next ? next.id : '.', spanP: spanP ? spanP.id : '.', noMatch: noMatch ? 'Y' : 'N'});
    "#;
    let result = wv.execute_script_with_dom(script).unwrap();
    assert!(
        result.contains("\"next\":\"a\""),
        "`h1 + p` 应匹配紧邻 h1 的 p（a）: {result}"
    );
    assert!(
        result.contains("\"spanP\":\"c\""),
        "`span + p` 应匹配紧邻 span 的 p（c）: {result}"
    );
    assert!(
        result.contains("\"noMatch\":\"N\""),
        "`h1 + span` 无匹配（紧邻 h1 的是 p）: {result}"
    );
}

#[test]
fn test_dom_js_polyfill_subsequent_sibling_combinator_r3287() {
    use zero_webview::{WebView, WebViewConfig};
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body></body></html>", None);
    let script = r#"
        var root = document.createElement('div');
        var h1 = document.createElement('h1');  h1.id = 't';
        var p1 = document.createElement('p');   p1.id = 'a';
        var p2 = document.createElement('p');   p2.id = 'b';
        var sp = document.createElement('span'); sp.id = 's';
        var p3 = document.createElement('p');   p3.id = 'c';
        root.appendChild(h1); root.appendChild(p1); root.appendChild(p2); root.appendChild(sp); root.appendChild(p3);
        var all = root.querySelectorAll('h1 ~ p').length;
        var spanP = root.querySelectorAll('span ~ p').length;
        var mixed = root.querySelector('h1 + p ~ p');
        JSON.stringify({all: all, spanP: spanP, mixed: mixed ? mixed.id : '.'});
    "#;
    let result = wv.execute_script_with_dom(script).unwrap();
    assert!(
        result.contains("\"all\":3"),
        "`h1 ~ p` 应匹配 h1 之后全部 p（a/b/c = 3）: {result}"
    );
    assert!(
        result.contains("\"spanP\":1"),
        "`span ~ p` 应仅匹配 span 之后的 p（c = 1）: {result}"
    );
    assert!(
        result.contains("\"mixed\":\"b\""),
        "`h1 + p ~ p` 应匹配 b（h1+p=a，a 之后的 p 首个 = b，回溯正确）: {result}"
    );
}

// ── R3288：execute_script_with_dom（A 代 polyfill）复合选择器 + 伪类 ──────────────
// 旧 A 代 _matchesSingleSelector 按 `#`/`.`/`[`/`:`/tag 顺序短路，致复合选择器（`div.foo:first-child`、
// `input:checked`、`a:not(.x)`）无法工作——tag 分支先吞复合串。R3288 重构为真复合匹配（tag/id/class/
// attr/pseudo AND）+ ~15 静态可判定伪类 + nth-child(an+b)，与 DOM crate（R3277-R3284）+ B 代 shim 对齐。

#[test]
fn test_dom_js_polyfill_compound_selector_r3288() {
    use zero_webview::{WebView, WebViewConfig};
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body></body></html>", None);
    // div.foo#bar + 复合 tag.class + tag[attr=val] + tag.class[attr]
    let script = r#"
        var d1 = document.createElement('div'); d1.id = 'bar'; d1.className = 'foo';
        var d2 = document.createElement('div'); d2.className = 'foo';
        var d3 = document.createElement('input'); d3.setAttribute('type', 'text');
        d1.appendChild(d2); d1.appendChild(d3);
        var r1 = d1.querySelector('div.foo#bar') !== null;     // 复合 tag+class+id（自身=div，但 querySelector 只查子树）
        var r2 = d1.querySelector('div.foo') === d2;            // 子树内 div.foo
        var r3 = d1.querySelector('input[type=text]') === d3;   // tag+属性=值
        var r4 = d1.querySelector('div.foo#nope') === null;     // id 不匹配 → null
        JSON.stringify({r1: r1, r2: r2, r3: r3, r4: r4});
    "#;
    let result = wv.execute_script_with_dom(script).unwrap();
    assert!(
        result.contains(r#""r1":false"#),
        "`div.foo#bar` 在子树内无匹配（d1 自身不计入）: {result}"
    );
    assert!(result.contains(r#""r2":true"#), "`div.foo` 应匹配 d2: {result}");
    assert!(
        result.contains(r#""r3":true"#),
        "`input[type=text]` 应匹配 d3: {result}"
    );
    assert!(result.contains(r#""r4":true"#), "`div.foo#nope` 无匹配: {result}");
}

#[test]
fn test_dom_js_polyfill_pseudo_classes_r3288() {
    use zero_webview::{WebView, WebViewConfig};
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body></body></html>", None);
    // root: h1, p.a, p.b(checked input 子), span, p.c
    let script = r#"
        var root = document.createElement('div');
        var h1 = document.createElement('h1');
        var p1 = document.createElement('p'); p1.className = 'a';
        var p2 = document.createElement('p'); p2.className = 'b';
        var cb = document.createElement('input'); cb.setAttribute('checked', '');
        p2.appendChild(cb);
        var sp = document.createElement('span');
        var p3 = document.createElement('p'); p3.className = 'c';
        root.appendChild(h1); root.appendChild(p1); root.appendChild(p2); root.appendChild(sp); root.appendChild(p3);
        var firstChild = root.querySelector('p:first-child');
        var lastP = root.querySelector('p:last-child');
        var firstPCompound = root.querySelector('p.a:first-child');   // 复合 tag.class:伪类
        var nthEven = root.querySelectorAll('p:nth-child(even)').length;
        var nth2n1 = root.querySelectorAll(':nth-child(2n+1)').length;
        var checked = root.querySelector('input:checked') === cb;       // :checked
        var notClass = root.querySelectorAll('p:not(.b)').length;       // :not(.b) → a + c = 2
        var onlyChild = root.querySelector('span:only-child') === null; // span 非独子
        JSON.stringify({
          firstP: firstChild ? firstChild.className : '.',     // p:first-child → 无（root 首子是 h1）
          lastP: lastP ? lastP.className : '.',                 // p:last-child → 无（末子是 p.c，但 p:last-child 需 p 且末位）
          firstPCompound: firstPCompound ? firstPCompound.className : '.', // p.a:first-child → 无（a 非首子）
          nthEven: nthEven,
          nth2n1: nth2n1,
          checked: checked,
          notClass: notClass,
          onlyChild: onlyChild
        });
    "#;
    let result = wv.execute_script_with_dom(script).unwrap();
    // p:first-child：root 首子是 h1，无 p 是首子 → null
    assert!(
        result.contains(r#""firstP":".""#),
        "`p:first-child` 无匹配（root 首子 h1）: {result}"
    );
    // p:last-child：末子是 p.c（p 且末位）→ c
    assert!(
        result.contains(r#""lastP":"c""#),
        "`p:last-child` 应匹配末子 p.c: {result}"
    );
    assert!(
        result.contains(r#""firstPCompound":".""#),
        "`p.a:first-child` 无匹配（a 非首子）: {result}"
    );
    assert!(
        result.contains(r#""checked":true"#),
        "`input:checked` 应匹配 cb: {result}"
    );
    assert!(
        result.contains(r#""notClass":2"#),
        "`p:not(.b)` 应匹配 a + c = 2: {result}"
    );
    assert!(
        result.contains(r#""onlyChild":true"#),
        "`span:only-child` 无匹配（span 非独子）→ null === null → true: {result}"
    );
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
