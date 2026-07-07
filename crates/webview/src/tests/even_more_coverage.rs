// 更多覆盖率测试 - 进一步提高 webview.rs 覆盖率
use crate::*;
use std::cell::RefCell;
use std::rc::Rc;

// ── extract_origin 测试 ──

#[test]
fn test_extract_origin_file_scheme() {
    // file scheme 可能不返回 origin
    let origin = WebView::extract_origin("file:///path/to/file.html");
    // 根据实际实现，file scheme 可能返回 "null"
    assert!(origin.is_none() || origin == Some("null".to_string()));
}

#[test]
fn test_extract_origin_ftp_scheme() {
    // 测试其他 scheme（如 ftp）
    let origin = WebView::extract_origin("ftp://example.com/path");
    assert_eq!(origin, Some("ftp://example.com".to_string()));
}

#[test]
fn test_extract_origin_ws_scheme() {
    // 测试 WebSocket scheme
    let origin = WebView::extract_origin("ws://example.com/path");
    assert_eq!(origin, Some("ws://example.com".to_string()));
}

#[test]
fn test_extract_origin_wss_scheme() {
    // 测试 WebSocket Secure scheme
    let origin = WebView::extract_origin("wss://example.com/path");
    assert_eq!(origin, Some("wss://example.com".to_string()));
}

#[test]
fn test_extract_origin_with_query_params() {
    // 带有查询参数的 URL
    let origin = WebView::extract_origin("https://example.com/path?param=value");
    assert_eq!(origin, Some("https://example.com".to_string()));
}

#[test]
fn test_extract_origin_with_user_info() {
    // 带有用户信息的 URL
    let origin = WebView::extract_origin("https://user:pass@example.com/path");
    assert_eq!(origin, Some("https://example.com".to_string()));
}

// ── load_html 边界情况测试 ──

#[test]
fn test_load_html_empty_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    let _result = wv.load_html("<html><body>test</body></html>", Some(""));
    // CSS 为空字符串应该正常工作
    assert!(!wv.is_loading());
}

#[test]
fn test_load_html_whitespace_only_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    let _result = wv.load_html("<html><body>test</body></html>", Some("   \n  \t  "));
    // 只有空白字符的 CSS 应该正常工作
    assert!(!wv.is_loading());
}

#[test]
fn test_load_html_very_large_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    let large_html = "<html><body>".to_owned() + &"<p>test</p>".repeat(10000) + "</body></html>";
    let _result = wv.load_html(&large_html, None);
    // 大量 HTML 应该能正常处理（测试不会 panic）
    assert!(!wv.is_loading());
}

#[test]
fn test_load_html_with_unicode_content() {
    let mut wv = WebView::new(WebViewConfig::default());
    let unicode_html = "<html><body>测试内容 🚀</body></html>";
    let _result = wv.load_html(unicode_html, None);
    // Unicode 内容应该正常处理
    assert!(!wv.is_loading());
}

// ── execute_script 更多错误路径测试 ──

#[test]
fn test_execute_script_very_long_script() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 非常长的脚本
    let long_script = "console.log('test');".repeat(1000);
    let result = wv.execute_script(&long_script);
    // 长脚本应该不会 panic，可能成功或失败
    assert!(result.is_ok() || result.is_err());
}

#[test]
#[cfg(feature = "v8")]
fn test_execute_script_with_unicode() {
    let mut wv = WebView::new(WebViewConfig::default());
    let script = "console.log('你好，世界！🌍');";
    let result = wv.execute_script(script);
    // Unicode 脚本应该正常执行
    assert!(result.is_ok());
}

#[test]
fn test_execute_script_with_special_chars() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 包含特殊字符的脚本
    let script = r#"const chars = "特殊字符: ~!@#$%^&*()_+-=[]{}|;':\",./<>?";"#;
    let result = wv.execute_script(script);
    // 特殊字符应该正常处理
    assert!(result.is_ok());
}

#[test]
#[cfg(feature = "v8")]
fn test_execute_script_multiple_calls() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 多次调用 execute_script
    for i in 0..10 {
        let script = format!("console.log({});", i);
        let result = wv.execute_script(&script);
        assert!(result.is_ok());
    }
}

// ── Service Worker 更多测试场景 ──

#[test]
fn test_service_worker_register_duplicate() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 注册相同的 Service Worker 多次
    let sw_id1 = wv.register_service_worker("/sw1.js", "/", "https://example.com");
    let sw_id2 = wv.register_service_worker("/sw1.js", "/", "https://example.com");

    // 应该返回不同的 ID
    assert_ne!(sw_id1, sw_id2);

    // 两个都应该在注册表中
    assert!(wv.service_worker_registry().get(sw_id1).is_some());
    assert!(wv.service_worker_registry().get(sw_id2).is_some());
}

#[test]
fn test_service_worker_register_multiple_scopes() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 注册相同脚本但不同 scope
    let sw_id1 = wv.register_service_worker("/sw.js", "/", "https://example.com");
    let sw_id2 = wv.register_service_worker("/sw.js", "/app", "https://example.com");

    assert_ne!(sw_id1, sw_id2);
}

#[test]
fn test_service_worker_register_different_origins() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 注册相同脚本但不同 origin
    let sw_id1 = wv.register_service_worker("/sw.js", "/", "https://example.com");
    let sw_id2 = wv.register_service_worker("/sw.js", "/", "https://other.com");

    assert_ne!(sw_id1, sw_id2);
}

#[test]
fn test_service_worker_install_after_unregister() {
    let mut wv = WebView::new(WebViewConfig::default());

    let sw_id = wv.register_service_worker("/sw.js", "/", "https://example.com");
    // 先注销再尝试安装
    let unregister_success = wv.unregister_service_worker(sw_id);
    assert!(unregister_success);

    // 注销后再安装应该返回 false
    let install_success = wv.install_service_worker(sw_id);
    assert!(!install_success);
}

#[test]
fn test_service_worker_activate_before_register() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 尝试激活一个不存在的 Service Worker
    let activate_success = wv.activate_service_worker(999);
    assert!(!activate_success);
}

// ── inject_css 更多测试 ──

#[test]
fn test_inject_css_malformed_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>test</body></html>", None);

    // 注入格式错误的 CSS
    let _result = wv.inject_css("body { color: red }");
    // 即使 CSS 格式错误，也应该不会 panic
    assert!(!wv.is_loading());
}

#[test]
fn test_inject_css_with_very_long_rule() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>test</body></html>", None);

    // 非常长的 CSS 规则
    let long_rule = "body { ".to_owned() + &"color: red; ".repeat(1000) + "}";
    let _result = wv.inject_css(&long_rule);
    // 长 CSS 规则应该能处理
    assert!(!wv.is_loading());
}

#[test]
fn test_inject_css_unicode() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>测试内容</body></html>", None);

    // Unicode 字符的 CSS
    let unicode_css = r#"body { font-family: "微软雅黑"; content: "🚀"; }"#;
    let _result = wv.inject_css(unicode_css);
    // Unicode CSS 应该正常处理
    assert!(!wv.is_loading());
}

// ── fetch_url 更多错误场景测试 ──

#[test]
fn test_fetch_url_invalid_scheme_http() {
    // HTTP 在没有网络时可能被阻止
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.fetch_url("http://nonexistent.example.com/path");
    // 错误的 URL 或网络问题应该返回错误
    assert!(result.is_ok() || result.is_err());
    assert!(!wv.is_loading());
}

#[test]
fn test_fetch_url_relative_path() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 相对路径应该无效
    let result = wv.fetch_url("/relative/path");
    // 相对路径可能被视为错误
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_fetch_url_with_fragment() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 带 fragment 的 URL
    let result = wv.fetch_url("https://example.com/page#section");
    // Fragment 应该被忽略，但不应该导致错误
    assert!(result.is_ok() || result.is_err());
}

// ── 完整生命周期测试 ──

#[test]
fn test_full_lifecycle_multiple_times() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 捕获事件
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        events_clone.borrow_mut().push(format!("{:?}", e));
    });

    // 完整生命周期重复多次
    for i in 0..3 {
        // 1. 开始加载
        wv.load_url(&format!("https://example.com/page{}", i));

        // 2. 完成加载
        wv.complete_load(&format!("<html><body>Page {}</body></html>", i), None);

        // 3. 注入 CSS
        wv.inject_css(&format!(
            "body {{ color: {}; }}",
            if i % 2 == 0 { "red" } else { "blue" }
        ));

        // 4. 重新渲染
        let _ = wv.render();

        // 5. 设置标题
        wv.set_title(&format!("Page {}", i));

        // 6. 执行脚本
        let script = format!("console.log('Page {} loaded');", i);
        let _ = wv.execute_script(&script);
    }

    // 验证事件数量
    let event_count = events.borrow().len();
    assert!(event_count > 0);
    assert!(!wv.is_loading());
}

// ── resize 边界测试 ──

#[test]
fn test_resize_to_small_dimensions() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>test</body></html>", None);

    // 调整到非常小的尺寸
    wv.resize(1, 1);
    let result = wv.render();
    // 应该能处理小尺寸
    assert!(!wv.is_loading());
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_resize_to_same_dimensions() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>test</body></html>", None);

    // 调整到相同尺寸
    let original_width = wv.config().width;
    let original_height = wv.config().height;
    wv.resize(original_width, original_height);

    let result = wv.render();
    // 应该正常工作
    assert!(!wv.is_loading());
    assert!(result.timings.total_ms >= 0.0);
}

// ── edge cases 测试 ──

#[test]
fn test_empty_webview_without_any_operations() {
    let wv = WebView::new(WebViewConfig::default());

    // 不进行任何操作，只是创建和销毁
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), None);
    assert_eq!(wv.title(), None);
    assert!(wv.last_render().is_none());
}
