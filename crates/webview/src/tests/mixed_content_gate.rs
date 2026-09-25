//! Mixed Content 分级阻止 + HSTS 响应注册（security-hardening M3）：
//! `mixed_content_enforcement` kill-switch 零影响面 + HTTPS 页面子资源分级阻止/
//! 升级 + `Strict-Transport-Security` 响应头注册接线。

use crate::{ImageSourceFetcher, ScriptSourceFetcher, WebView, WebViewConfig};
use std::sync::{Arc, Mutex};

/// 记录 fetch 请求 URL 的 spy fetcher（img/stylesheet 共用形态）。
type FetchLog = Arc<Mutex<Vec<String>>>;

fn image_fetcher_stub(log: FetchLog) -> ImageSourceFetcher {
    Arc::new(move |url: &str| {
        log.lock().unwrap_or_else(|e| e.into_inner()).push(url.to_string());
        Some(Vec::new())
    })
}

fn script_fetcher_stub(log: FetchLog) -> ScriptSourceFetcher {
    Arc::new(move |page_url: &str, src: &str| {
        log.lock()
            .unwrap_or_else(|e| e.into_inner())
            .push(format!("{page_url} | {src}"));
        Ok("globalThis.__ext_ran = 1;".to_string())
    })
}

/// M3：HSTS 响应头注册接线——HTTPS 响应的 `Strict-Transport-Security` 头进
/// HSTS store，后续 `check_resource_url` 强制升级；includeSubDomains 随头传播。
#[test]
fn hsts_response_header_registered_and_upgrades_m3() {
    // 强制面显式 on（照 csp_gate.rs 口径）：Default 读 `ZW_MIXED_CONTENT_ENFORCEMENT`
    // env——off 臂 A/B 环境下用例不得随环境漂移。
    let mut wv = WebView::new(WebViewConfig {
        mixed_content_enforcement: true,
        ..WebViewConfig::default()
    });
    // HTTPS 响应 + includeSubDomains 头 → host 注册。
    wv.note_hsts_response(
        "https://secure.example.com/page",
        &[(
            "Strict-Transport-Security".into(),
            "max-age=31536000; includeSubDomains".into(),
        )],
    );
    // 后续 HTTP 子资源/导航强制升级。
    assert_eq!(
        wv.check_subresource_url("http://secure.example.com/api", "connect"),
        zero_security::ResourceCheckResult::Upgraded("https://secure.example.com/api".to_string()),
        "已注册 host 的 HTTP 资源必须升级 HTTPS"
    );
    // includeSubDomains 传播到子域。
    assert_eq!(
        wv.check_subresource_url("http://sub.secure.example.com/x", "script"),
        zero_security::ResourceCheckResult::Upgraded("https://sub.secure.example.com/x".to_string()),
        "includeSubDomains 须覆盖子域"
    );
    // 非 HTTPS 响应不注册。
    wv.note_hsts_response(
        "http://plain.example.com/",
        &[("Strict-Transport-Security".into(), "max-age=31536000".into())],
    );
    assert_eq!(
        wv.check_subresource_url("http://plain.example.com/x", "document"),
        zero_security::ResourceCheckResult::Allow,
        "HTTP 响应的 HSTS 头不得注册"
    );
    // 无 HSTS 头静默跳过（不 panic、不注册）。
    wv.note_hsts_response("https://other.example.com/", &[]);
    assert_eq!(
        wv.check_subresource_url("http://other.example.com/x", "document"),
        zero_security::ResourceCheckResult::Allow,
    );
    // 解析失败的头不注册（无效指令语义）。
    wv.note_hsts_response(
        "https://bad.example.com/",
        &[("Strict-Transport-Security".into(), "garbage".into())],
    );
    assert_eq!(
        wv.check_subresource_url("http://bad.example.com/x", "document"),
        zero_security::ResourceCheckResult::Allow,
    );
}

/// M3：HTTPS 页面 HTTP 外链样式表（阻塞型）不抓取；HTTPS 样式表照常抓取。
#[test]
fn mixed_content_blocks_http_stylesheet_on_https_page_m3() {
    let log: FetchLog = Arc::new(Mutex::new(Vec::new()));
    let mut wv = WebView::new(WebViewConfig {
        mixed_content_enforcement: true,
        image_source_fetcher: Some(image_fetcher_stub(Arc::clone(&log))),
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/mc/style.html");
    wv.complete_fetched_page(
        r#"<html><head>
<link rel="stylesheet" href="http://cdn.test/blocked.css">
<link rel="stylesheet" href="https://cdn.test/allowed.css">
</head><body><p>x</p></body></html>"#,
        "https://wpt.test/mc/style.html",
    );
    let fetched = log.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        !fetched.iter().any(|u| u.starts_with("http://cdn.test/blocked.css")),
        "HTTP 样式表必须被阻止不抓取: {fetched:?}"
    );
    assert!(
        fetched.iter().any(|u| u == "https://cdn.test/allowed.css"),
        "HTTPS 样式表须照常抓取: {fetched:?}"
    );
}

/// M3：HTTPS 页面 HTTP 图片（可选阻塞型）自动升级 HTTPS 抓取；缓存键仍按
/// markup 原始 URL（painter 按 src 解析查找）。
#[test]
fn mixed_content_upgrades_http_image_on_https_page_m3() {
    let log: FetchLog = Arc::new(Mutex::new(Vec::new()));
    let mut wv = WebView::new(WebViewConfig {
        mixed_content_enforcement: true,
        image_source_fetcher: Some(image_fetcher_stub(Arc::clone(&log))),
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/mc/img.html");
    wv.complete_fetched_page(
        r#"<html><body><img src="http://cdn.test/photo.png"></body></html>"#,
        "https://wpt.test/mc/img.html",
    );
    let fetched = log.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        fetched.iter().any(|u| u == "https://cdn.test/photo.png"),
        "HTTP 图片须升级 HTTPS 后抓取: {fetched:?}"
    );
    assert!(
        !fetched.iter().any(|u| u == "http://cdn.test/photo.png"),
        "升级后不得再按原始 HTTP URL 抓取: {fetched:?}"
    );
}

/// M3：HTTPS 页面 HTTP 外链脚本（阻塞型）不抓取不执行。
#[test]
fn mixed_content_blocks_http_script_on_https_page_m3() {
    let log: FetchLog = Arc::new(Mutex::new(Vec::new()));
    let mut wv = WebView::new(WebViewConfig {
        mixed_content_enforcement: true,
        script_source_fetcher: Some(script_fetcher_stub(Arc::clone(&log))),
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/mc/script.html");
    wv.complete_fetched_page(
        r#"<html><body><script src="http://evil.test/inject.js"></script></body></html>"#,
        "https://wpt.test/mc/script.html",
    );
    wv.run_page_scripts().expect("run page scripts");
    assert!(
        log.lock().unwrap_or_else(|e| e.into_inner()).is_empty(),
        "HTTP 外链脚本必须被阻止不抓取: {:?}",
        log.lock().unwrap_or_else(|e| e.into_inner())
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__ext_ran)").unwrap(),
        "undefined",
        "被阻止脚本不得执行"
    );
}

/// M3 kill-switch：`mixed_content_enforcement = false` 时 HTTPS 页面 HTTP 子资源
/// 照旧加载（宿主显式回退面）；HTTP 页面天然不受检（非安全页面零影响）。
#[test]
fn mixed_content_kill_switch_off_and_http_page_inert_m3() {
    // off 臂：HTTP 样式表照常抓取。
    let log: FetchLog = Arc::new(Mutex::new(Vec::new()));
    let mut wv = WebView::new(WebViewConfig {
        mixed_content_enforcement: false,
        image_source_fetcher: Some(image_fetcher_stub(Arc::clone(&log))),
        ..WebViewConfig::default()
    });
    wv.prepare_document_state("https://wpt.test/mc/off.html");
    wv.complete_fetched_page(
        r#"<html><head><link rel="stylesheet" href="http://cdn.test/off.css"></head><body></body></html>"#,
        "https://wpt.test/mc/off.html",
    );
    assert!(
        log.lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .any(|u| u == "http://cdn.test/off.css"),
        "kill-switch off 时 HTTP 样式表须照常抓取: {:?}",
        log.lock().unwrap_or_else(|e| e.into_inner())
    );

    // HTTP 页面（非安全上下文）：HTTP 子资源不受检、照常加载。
    let log2: FetchLog = Arc::new(Mutex::new(Vec::new()));
    let mut wv2 = WebView::new(WebViewConfig {
        mixed_content_enforcement: true,
        image_source_fetcher: Some(image_fetcher_stub(Arc::clone(&log2))),
        ..WebViewConfig::default()
    });
    wv2.prepare_document_state("http://wpt.test/mc/plain.html");
    wv2.complete_fetched_page(
        r#"<html><body><img src="http://cdn.test/plain.png"></body></html>"#,
        "http://wpt.test/mc/plain.html",
    );
    assert!(
        log2.lock()
            .unwrap_or_else(|e| e.into_inner())
            .iter()
            .any(|u| u == "http://cdn.test/plain.png"),
        "HTTP 页面的 HTTP 图片不受 Mixed Content 检查: {:?}",
        log2.lock().unwrap_or_else(|e| e.into_inner())
    );
}
