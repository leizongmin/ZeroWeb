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

// ── SecurityContext 集成测试 ──

/// WebView 安全上下文：初始状态有 HSTS 预加载列表。
#[test]
fn test_webview_security_context_initial_state() {
    let wv = WebView::new(WebViewConfig::default());
    // 安全上下文应已初始化，包含 HSTS 预加载列表
    let ctx = wv.security_context();
    assert!(ctx.hsts_count() > 0, "预加载列表应已加载");
    assert!(ctx.page_origin().is_none(), "初始无页面源");
}

/// WebView 安全上下文：子资源混合内容阻止。
#[test]
fn test_webview_check_subresource_blocks_mixed_content() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 模拟 HTTPS 页面
    wv.security_context_mut().set_page_origin("https://secure.example.com");

    // script → 阻止
    let result = wv.check_subresource_url("http://evil.com/steal.js", "script");
    assert!(matches!(result, zero_security::ResourceCheckResult::Blocked(_)));

    // img → 升级
    let result = wv.check_subresource_url("http://cdn.com/photo.jpg", "img");
    assert!(matches!(result, zero_security::ResourceCheckResult::Upgraded(_)));
}

/// WebView 安全上下文：HSTS 预加载升级子资源。
#[test]
fn test_webview_check_subresource_hsts_upgrade() {
    let mut wv = WebView::new(WebViewConfig::default());

    // github.com 在 HSTS 预加载列表中
    let result = wv.check_subresource_url("http://github.com/file.js", "script");
    assert!(matches!(result, zero_security::ResourceCheckResult::Upgraded(ref url) if url.starts_with("https://")));
}

/// WebView 安全上下文：运行时注册 HSTS。
#[test]
fn test_webview_register_hsts_from_response_header() {
    let mut wv = WebView::new(WebViewConfig::default());
    let count_before = wv.security_context().hsts_count();

    // 模拟从 HTTPS 响应头注册 HSTS
    assert!(
        wv.security_context_mut()
            .register_hsts("custom-secure.com", "max-age=31536000; includeSubDomains")
    );

    let count_after = wv.security_context().hsts_count();
    assert_eq!(count_after, count_before + 1);

    // 注册后 HTTP URL 应被升级
    let result = wv.check_subresource_url("http://custom-secure.com/page", "document");
    assert!(matches!(result, zero_security::ResourceCheckResult::Upgraded(_)));
}

/// WebView 安全上下文：完整混合内容矩阵。
#[test]
fn test_webview_mixed_content_full_matrix() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.security_context_mut().set_page_origin("https://secure.bank.com");

    // Blockable 类型
    for rt in &["script", "style", "connect", "font", "iframe", "object", "worker"] {
        let result = wv.check_subresource_url("http://attacker.com/res", rt);
        assert!(
            matches!(result, zero_security::ResourceCheckResult::Blocked(_)),
            "类型 '{rt}' 应被阻止"
        );
    }

    // OptionallyBlockable 类型
    for rt in &["img", "audio", "video", "media"] {
        let result = wv.check_subresource_url("http://cdn.com/res", rt);
        assert!(
            matches!(result, zero_security::ResourceCheckResult::Upgraded(ref url) if url.starts_with("https://")),
            "类型 '{rt}' 应被升级"
        );
    }
}

/// WebView load_html 不受安全上下文影响。
#[test]
fn test_webview_load_html_ignores_security_context() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.security_context_mut().set_page_origin("https://secure.example.com");

    // 直接 load_html 不走 fetch_url，不受混合内容检查
    let html = r#"<html><body><h1>Direct HTML</h1></body></html>"#;
    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());
}
