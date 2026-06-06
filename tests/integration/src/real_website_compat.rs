//! 真实网站兼容性集成测试
//!
//! 验证浏览器引擎能加载并渲染真实网站内容。
//! 所有测试标记为 #[ignore]，需通过 `cargo test -p zero-integration-tests -- --ignored` 运行。
//! 测试需要网络连接。
//!
//! Top 20 目标网站（按复杂度分级）：
//! - Tier 1（极简）: example.com, info.cern.ch, httpbin.org/html, w3.org, whatwg.org
//! - Tier 2（文本为主）: lite.cnn.com, lobste.rs, curl.se, ietf.org, datatracker.ietf.org
//! - Tier 3（技术站）: rust-lang.org, python.org, nodejs.org, docs.rs, jsonplaceholder.typicode.com
//! - Tier 4（主流复杂）: github.com, stackoverflow.com, cloudflare.com, w3schools.com, pkg.go.dev

use zero_webview::{WebView, WebViewBuilder, WebViewConfig};

// ── 辅助函数 ──

/// 创建标准测试 WebView（1024×768 视口）
fn test_webview() -> WebView {
    WebViewBuilder::new()
        .width(1024)
        .height(768)
        .user_agent("ZeroWeb/1.0 Compatibility Test")
        .build()
}

/// 验证渲染结果的基本合理性：
/// - 总图元数 > 0（页面有可见内容）
/// - glyph 或 fill 至少有一个非空（文字或背景）
/// - 管线各阶段耗时 >= 0
fn assert_valid_render(result: &zero_webview::WebViewRenderResult, site: &str) {
    let total = result.primitives.len();
    assert!(
        total > 0,
        "{site}: 渲染结果应包含至少一个图元，实际 primitives.len() = {total}"
    );

    assert!(
        result.timings.total_ms >= 0.0,
        "{site}: 总渲染时间应为非负值"
    );

    // 至少应有文字（glyph）或填充（fill）图元
    let has_glyphs = !result.primitives.glyphs.is_empty();
    let has_fills = !result.primitives.fills.is_empty();
    assert!(
        has_glyphs || has_fills,
        "{site}: 页面应包含文字或填充图元（glyphs={}, fills={}",
        result.primitives.glyphs.len(),
        result.primitives.fills.len()
    );
}

/// 验证 HTML 内容的基本合理性：
/// - 非空
/// - 包含 <html> 或 <body> 标签
fn assert_valid_html(html: &str, site: &str) {
    assert!(
        !html.is_empty(),
        "{site}: HTML 内容不应为空"
    );
    let lower = html.to_lowercase();
    assert!(
        lower.contains("<html") || lower.contains("<body") || lower.contains("<!doctype"),
        "{site}: HTML 应包含 <html>、<body> 或 <!doctype> 标签，实际前 200 字符: {}",
        &html[..200.min(html.len())]
    );
}

/// 验证 HTML 包含指定文本片段
fn assert_html_contains(html: &str, needle: &str, site: &str) {
    let lower = html.to_lowercase();
    assert!(
        lower.contains(&needle.to_lowercase()),
        "{site}: HTML 应包含 '{needle}'，实际前 500 字符: {}",
        &html[..500.min(html.len())]
    );
}

// ════════════════════════════════════════════════════════════════
// Tier 1: 极简静态网站（几乎无 JS，纯 HTML）
// ════════════════════════════════════════════════════════════════

/// #1 example.com — IANA 维护的示例域名，最简单的标准 HTML 页面
#[test]
#[ignore = "需要网络连接"]
fn test_site_example_com() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://example.com").expect("example.com 应可访问");

    assert_valid_render(&result, "example.com");
    assert_valid_html(wv.html_content(), "example.com");
    assert_html_contains(wv.html_content(), "Example Domain", "example.com");
    assert_html_contains(wv.html_content(), "iana", "example.com");
}

/// #2 info.cern.ch — 世界上第一个网站（纯静态 HTML，无 CSS/JS）
#[test]
#[ignore = "需要网络连接"]
fn test_site_info_cern_ch() {
    let mut wv = test_webview();
    let result = wv.fetch_url("http://info.cern.ch").expect("info.cern.ch 应可访问");

    assert_valid_render(&result, "info.cern.ch");
    assert_valid_html(wv.html_content(), "info.cern.ch");
    // 世界上第一个网页应包含 hypertext 相关文字
    assert_html_contains(wv.html_content(), "hypertext", "info.cern.ch");
}

/// #3 httpbin.org/html — HTTP 测试服务的简单 HTML 页面
#[test]
#[ignore = "需要网络连接"]
fn test_site_httpbin_html() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://httpbin.org/html").expect("httpbin.org/html 应可访问");

    assert_valid_render(&result, "httpbin.org/html");
    assert_valid_html(wv.html_content(), "httpbin.org/html");
    // httpbin HTML 页面包含 Hermann Melville 的文字
    assert_html_contains(wv.html_content(), "Moby", "httpbin.org/html");
}

/// #4 w3.org — W3C 官方网站
#[test]
#[ignore = "需要网络连接"]
fn test_site_w3_org() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://www.w3.org").expect("w3.org 应可访问");

    assert_valid_render(&result, "w3.org");
    assert_valid_html(wv.html_content(), "w3.org");
    assert_html_contains(wv.html_content(), "W3C", "w3.org");
}

/// #5 whatwg.org — Web 超文本应用技术工作组
#[test]
#[ignore = "需要网络连接"]
fn test_site_whatwg() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://whatwg.org").expect("whatwg.org 应可访问");

    assert_valid_render(&result, "whatwg.org");
    assert_valid_html(wv.html_content(), "whatwg.org");
    assert_html_contains(wv.html_content(), "whatwg", "whatwg.org");
}

// ════════════════════════════════════════════════════════════════
// Tier 2: 文本为主的静态网站（有基础 CSS，少 JS）
// ════════════════════════════════════════════════════════════════

/// #6 lite.cnn.com — CNN 精简版（为低带宽设计的纯文本新闻）
#[test]
#[ignore = "需要网络连接"]
fn test_site_lite_cnn() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://lite.cnn.com").expect("lite.cnn.com 应可访问");

    assert_valid_render(&result, "lite.cnn.com");
    assert_valid_html(wv.html_content(), "lite.cnn.com");
    assert_html_contains(wv.html_content(), "cnn", "lite.cnn.com");
}

/// #7 lobste.rs — 技术新闻聚合（简洁 HTML，类似 HN）
#[test]
#[ignore = "需要网络连接"]
fn test_site_lobsters() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://lobste.rs").expect("lobste.rs 应可访问");

    assert_valid_render(&result, "lobste.rs");
    assert_valid_html(wv.html_content(), "lobste.rs");
    assert_html_contains(wv.html_content(), "lobste", "lobste.rs");
}

/// #8 curl.se — cURL 官网（简洁的文档站）
#[test]
#[ignore = "需要网络连接"]
fn test_site_curl() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://curl.se").expect("curl.se 应可访问");

    assert_valid_render(&result, "curl.se");
    assert_valid_html(wv.html_content(), "curl.se");
    assert_html_contains(wv.html_content(), "curl", "curl.se");
}

/// #9 ietf.org — 互联网工程任务组
#[test]
#[ignore = "需要网络连接"]
fn test_site_ietf() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://www.ietf.org").expect("ietf.org 应可访问");

    assert_valid_render(&result, "ietf.org");
    assert_valid_html(wv.html_content(), "ietf.org");
    assert_html_contains(wv.html_content(), "ietf", "ietf.org");
}

/// #10 datatracker.ietf.org — IETF 文档追踪器
#[test]
#[ignore = "需要网络连接"]
fn test_site_datatracker_ietf() {
    let mut wv = test_webview();
    let result = wv
        .fetch_url("https://datatracker.ietf.org")
        .expect("datatracker.ietf.org 应可访问");

    assert_valid_render(&result, "datatracker.ietf.org");
    assert_valid_html(wv.html_content(), "datatracker.ietf.org");
    assert_html_contains(wv.html_content(), "ietf", "datatracker.ietf.org");
}

// ════════════════════════════════════════════════════════════════
// Tier 3: 技术文档 / 编程语言官网（中等 CSS 复杂度）
// ════════════════════════════════════════════════════════════════

/// #11 rust-lang.org — Rust 编程语言官网
#[test]
#[ignore = "需要网络连接"]
fn test_site_rust_lang() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://www.rust-lang.org").expect("rust-lang.org 应可访问");

    assert_valid_render(&result, "rust-lang.org");
    assert_valid_html(wv.html_content(), "rust-lang.org");
    assert_html_contains(wv.html_content(), "rust", "rust-lang.org");
}

/// #12 python.org — Python 编程语言官网
#[test]
#[ignore = "需要网络连接"]
fn test_site_python() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://www.python.org").expect("python.org 应可访问");

    assert_valid_render(&result, "python.org");
    assert_valid_html(wv.html_content(), "python.org");
    assert_html_contains(wv.html_content(), "Python", "python.org");
}

/// #13 nodejs.org — Node.js 官网
#[test]
#[ignore = "需要网络连接"]
fn test_site_nodejs() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://nodejs.org").expect("nodejs.org 应可访问");

    assert_valid_render(&result, "nodejs.org");
    assert_valid_html(wv.html_content(), "nodejs.org");
    assert_html_contains(wv.html_content(), "node", "nodejs.org");
}

/// #14 docs.rs — Rust 文档托管
#[test]
#[ignore = "需要网络连接"]
fn test_site_docs_rs() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://docs.rs").expect("docs.rs 应可访问");

    assert_valid_render(&result, "docs.rs");
    assert_valid_html(wv.html_content(), "docs.rs");
    assert_html_contains(wv.html_content(), "docs", "docs.rs");
}

/// #15 jsonplaceholder.typicode.com — 免费在线 REST API（简单 HTML 页面）
#[test]
#[ignore = "需要网络连接"]
fn test_site_jsonplaceholder() {
    let mut wv = test_webview();
    let result = wv
        .fetch_url("https://jsonplaceholder.typicode.com")
        .expect("jsonplaceholder.typicode.com 应可访问");

    assert_valid_render(&result, "jsonplaceholder.typicode.com");
    assert_valid_html(wv.html_content(), "jsonplaceholder.typicode.com");
    assert_html_contains(wv.html_content(), "json", "jsonplaceholder.typicode.com");
}

// ════════════════════════════════════════════════════════════════
// Tier 4: 主流网站（较复杂 HTML/CSS，可能有大量 JS）
// ════════════════════════════════════════════════════════════════

/// #16 github.com — GitHub 首页
#[test]
#[ignore = "需要网络连接"]
fn test_site_github() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://github.com").expect("github.com 应可访问");

    assert_valid_render(&result, "github.com");
    assert_valid_html(wv.html_content(), "github.com");
    assert_html_contains(wv.html_content(), "github", "github.com");
}

/// #17 stackoverflow.com — Stack Overflow
#[test]
#[ignore = "需要网络连接"]
fn test_site_stackoverflow() {
    let mut wv = test_webview();
    let result = wv
        .fetch_url("https://stackoverflow.com")
        .expect("stackoverflow.com 应可访问");

    assert_valid_render(&result, "stackoverflow.com");
    assert_valid_html(wv.html_content(), "stackoverflow.com");
    assert_html_contains(wv.html_content(), "stack", "stackoverflow.com");
}

/// #18 cloudflare.com — Cloudflare 官网
#[test]
#[ignore = "需要网络连接"]
fn test_site_cloudflare() {
    let mut wv = test_webview();
    let result = wv
        .fetch_url("https://www.cloudflare.com")
        .expect("cloudflare.com 应可访问");

    assert_valid_render(&result, "cloudflare.com");
    assert_valid_html(wv.html_content(), "cloudflare.com");
    assert_html_contains(wv.html_content(), "cloudflare", "cloudflare.com");
}

/// #19 w3schools.com — W3Schools 在线教程
#[test]
#[ignore = "需要网络连接"]
fn test_site_w3schools() {
    let mut wv = test_webview();
    let result = wv
        .fetch_url("https://www.w3schools.com")
        .expect("w3schools.com 应可访问");

    assert_valid_render(&result, "w3schools.com");
    assert_valid_html(wv.html_content(), "w3schools.com");
    assert_html_contains(wv.html_content(), "w3schools", "w3schools.com");
}

/// #20 pkg.go.dev — Go 包文档站
#[test]
#[ignore = "需要网络连接"]
fn test_site_pkg_go_dev() {
    let mut wv = test_webview();
    let result = wv.fetch_url("https://pkg.go.dev").expect("pkg.go.dev 应可访问");

    assert_valid_render(&result, "pkg.go.dev");
    assert_valid_html(wv.html_content(), "pkg.go.dev");
    assert_html_contains(wv.html_content(), "go", "pkg.go.dev");
}

// ════════════════════════════════════════════════════════════════
// 综合兼容性测试：多个网站加载不崩溃
// ════════════════════════════════════════════════════════════════

/// 连续加载多个网站不 panic，验证 WebView 生命周期管理
#[test]
#[ignore = "需要网络连接"]
fn test_multi_site_sequential_loading() {
    let mut wv = test_webview();
    let sites = vec![
        "https://example.com",
        "https://whatwg.org",
        "https://www.w3.org",
        "https://curl.se",
    ];

    for site in &sites {
        let result = wv
            .fetch_url(site)
            .unwrap_or_else(|e| panic!("{site} 加载不应返回致命错误: {e:?}"));

        assert_valid_render(&result, site);
        assert_valid_html(wv.html_content(), site);

        // 重置状态准备下一个站点
        wv.clear_http_cache();
    }
}

/// 不同视口尺寸下加载同一网站不崩溃
#[test]
#[ignore = "需要网络连接"]
fn test_responsive_render_example_com() {
    let viewports: [(u32, u32); 4] = [(320, 480), (768, 1024), (1024, 768), (1920, 1080)];

    for &(w, h) in &viewports {
        let mut wv = WebViewBuilder::new()
            .width(w)
            .height(h)
            .user_agent("ZeroWeb/1.0")
            .build();

        let result = wv
            .fetch_url("https://example.com")
            .unwrap_or_else(|e| panic!("example.com 在 {w}x{h} 下加载失败: {e:?}"));

        assert_valid_render(&result, &format!("example.com@{w}x{h}"));
    }
}

/// 验证页面 HTML 结构完整性：标题、元数据、正文内容
#[test]
#[ignore = "需要网络连接"]
fn test_page_structure_w3_org() {
    let mut wv = test_webview();
    let result = wv
        .fetch_url("https://www.w3.org")
        .expect("w3.org 应可访问");

    assert_valid_render(&result, "w3.org");
    let html = wv.html_content();

    // W3C 页面应有完整的 HTML 结构
    let lower = html.to_lowercase();
    assert!(lower.contains("<html"), "应包含 <html> 标签");
    assert!(lower.contains("<head"), "应包含 <head> 标签");
    assert!(lower.contains("<body"), "应包含 <body> 标签");
    assert!(lower.contains("w3c"), "应包含 W3C 相关内容");

    // 渲染结果应有大量图元（W3C 页面内容丰富）
    let total = result.primitives.len();
    assert!(
        total > 10,
        "W3C 页面应产生大量渲染图元，实际: {total}"
    );
}

/// 性能验证：中等复杂页面首屏渲染时间
#[test]
#[ignore = "需要网络连接"]
fn test_performance_python_org() {
    let mut wv = test_webview();
    let result = wv
        .fetch_url("https://www.python.org")
        .expect("python.org 应可访问");

    assert_valid_render(&result, "python.org");

    // 首屏渲染应在合理时间内完成（Done Criteria: < 2000ms）
    assert!(
        result.timings.total_ms < 2000.0,
        "python.org 首屏渲染应 < 2000ms，实际: {:.2}ms (parse={:.2}, style={:.2}, layout={:.2}, paint={:.2})",
        result.timings.total_ms,
        result.timings.parse_ms,
        result.timings.style_ms,
        result.timings.layout_ms,
        result.timings.paint_ms
    );
}
