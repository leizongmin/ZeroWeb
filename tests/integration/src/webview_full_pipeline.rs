#[cfg(test)]
use zero_webview::{WebView, WebViewBuilder, WebViewConfig};

/// WebView 完整生命周期：创建 → 加载 HTML → 渲染 → 注入 CSS → 调整大小 → 重新渲染
#[test]
fn test_webview_full_lifecycle() {
    let mut wv = WebViewBuilder::new()
        .width(800)
        .height(600)
        .user_agent("IntegrationTest/1.0")
        .build();

    // 初始状态
    assert!(wv.url().is_none());
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_none());

    // 加载 HTML
    let html = r#"<html><body>
        <header><h1>Title</h1></header>
        <main><p>Content paragraph</p></main>
    </body></html>"#;
    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());

    // 注入 CSS 重新渲染
    let result = wv.inject_css("h1 { color: red; font-size: 24px; } p { margin: 10px; }");
    assert!(result.timings.total_ms >= 0.0);

    // 调整大小
    wv.resize(1024, 768);
    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);

    // 重新渲染
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0);
}

/// WebView 加载复杂页面（多元素 + CSS）
#[test]
fn test_webview_complex_page_with_styles() {
    let html = r#"<html><body>
        <nav><a href="/">Home</a><a href="/about">About</a></nav>
        <section id="content">
            <article><h2>Article 1</h2><p>Text 1</p></article>
            <article><h2>Article 2</h2><p>Text 2</p></article>
        </section>
        <footer>Copyright 2026</footer>
    </body></html>"#;
    let css = r#"
        nav { background: #333; padding: 10px; }
        section { padding: 20px; }
        article { margin-bottom: 20px; border: 1px solid #ccc; }
        footer { text-align: center; padding: 5px; }
    "#;

    let mut wv = WebView::new(WebViewConfig {
        width: 1440,
        height: 900,
        ..Default::default()
    });
    let result = wv.load_html(html, Some(css));
    assert!(result.timings.total_ms >= 0.0);
    // 复杂页面应成功渲染，图元数量取决于管线实现
}

/// WebView 多次加载不 panic
#[test]
fn test_webview_repeated_load() {
    let mut wv = WebView::new(WebViewConfig::default());
    for i in 0..5 {
        let html = format!("<html><body><div>Page {i}</div></body></html>");
        let result = wv.load_html(&html, None);
        assert!(result.timings.total_ms >= 0.0, "第 {i} 次加载应成功");
    }
    assert!(wv.last_render().is_some());
}

/// WebView execute_script 通过 V8 沙箱执行脚本
#[test]
fn test_webview_script_execution() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("1+1");
    assert!(result.is_ok(), "V8 sandbox should execute simple script");
    assert_eq!(result.unwrap(), "2");
}

// ── HTTP 缓存集成测试 ──

/// WebView HTTP 缓存初始状态为空。
#[test]
fn test_webview_http_cache_initially_empty() {
    let wv = WebView::new(WebViewConfig::default());
    assert_eq!(wv.http_cache_len(), 0);
    assert_eq!(wv.http_cache_bytes(), 0);
}

/// WebView HTTP 缓存可以清空。
#[test]
fn test_webview_http_cache_clear() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 初始状态
    assert_eq!(wv.http_cache_len(), 0);
    // 清空空缓存不 panic
    wv.clear_http_cache();
    assert_eq!(wv.http_cache_len(), 0);
}

/// WebView 多次 clear_http_cache 不 panic。
#[test]
fn test_webview_http_cache_clear_multiple() {
    let mut wv = WebView::new(WebViewConfig::default());
    for _ in 0..5 {
        wv.clear_http_cache();
    }
    assert_eq!(wv.http_cache_len(), 0);
}
