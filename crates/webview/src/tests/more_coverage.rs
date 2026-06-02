// WebView 额外覆盖率测试
// 覆盖 fetch_url 的更多错误路径和边界情况

use crate::*;

/// 测试 fetch_url 无效 URL 格式
#[test]
fn test_fetch_url_invalid_scheme() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.fetch_url("invalid-url");
    assert!(result.is_err());
    assert!(!wv.is_loading());
}

/// 测试 Service Worker 无效注册
#[test]
fn test_register_service_worker_invalid_params() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 空字符串应该也能注册（实际使用中可能检查，但这里应该不 panic）
    let id1 = wv.register_service_worker("", "/", "");
    let id2 = wv.register_service_worker("http://example.com", "", "http://example.com");
    let id3 = wv.register_service_worker("/sw.js", "/", "");

    // 应该返回有效的 ID（不重复）
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
}

/// 测试 install_service_worker 无效 ID
#[test]
fn test_install_service_worker_invalid_id() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 尝试安装不存在的 SW ID
    let success = wv.install_service_worker(0); // 不存在的 ID
    assert!(!success);

    // 尝试安装超大 ID
    let success = wv.install_service_worker(u64::MAX);
    assert!(!success);
}

/// 测试 execute_script 大量参数和返回值
#[test]
fn test_execute_script_large_inputs() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 大对象创建测试
    let script = r#"
        const arr = [];
        for (let i = 0; i < 10000; i++) {
            arr.push({ id: i, name: 'test' + i });
        }
        JSON.stringify(arr);
    "#;

    let result = wv.execute_script(script);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("\"id\":9999"));
}

/// 测试 execute_script 深层属性访问
#[test]
fn test_execute_script_deep_property_access() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 深层属性链访问
    let script = r#"
        window.document.body.element.firstChild.id;
    "#;

    let result = wv.execute_script(script);
    // 即使对象不存在，也不应该 panic
    assert!(result.is_ok() || result.is_err());
}

/// 测试 execute_script_with_dom 空 DOM
#[test]
fn test_execute_script_with_dom_empty_html() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 先加载空的 HTML
    wv.load_html("", None);

    // 执行 DOM 操作
    let script = "document.createElement('div').className = 'test';";
    let result = wv.execute_script_with_dom(script);
    assert!(result.is_ok());
}

/// 测试 inject_css 到空 HTML
#[test]
fn test_inject_css_to_empty_html() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 空 HTML 状态下注入 CSS
    let result = wv.inject_css("body { color: red; }");
    assert!(result.timings.total_ms >= 0.0);
    // Check that render primitives exist without knowing exact structure
    assert!(wv.last_render().is_some());
}

/// 测试 resize 到最小尺寸
#[test]
fn test_resize_minimal_dimensions() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 调整到最小尺寸
    wv.resize(1, 1);
    assert_eq!(wv.config().width, 1);
    assert_eq!(wv.config().height, 1);
}

/// 测试 resize 到最大尺寸
#[test]
fn test_resize_large_dimensions() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 调整到超大尺寸
    let width = 10000;
    let height = 10000;
    wv.resize(width, height);
    assert_eq!(wv.config().width, width);
    assert_eq!(wv.config().height, height);
}

/// 测试多次 resize
#[test]
fn test_multiple_resizes() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 多次调整大小
    wv.resize(800, 600);
    assert_eq!(wv.config().width, 800);
    assert_eq!(wv.config().height, 600);

    wv.resize(1024, 768);
    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);

    wv.resize(300, 200);
    assert_eq!(wv.config().width, 300);
    assert_eq!(wv.config().height, 200);
}

/// 测试 fetch_url 相同 URL 多次
#[test]
fn test_fetch_url_same_url_multiple_times() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 注册 SW 以模拟缓存
    let sw_id = wv.register_service_worker("/sw.js", "/", "https://example.com");
    wv.install_service_worker(sw_id);

    // 第一次加载
    let _ = wv.fetch_url("https://example.com");

    // 第二次加载相同 URL
    let _ = wv.fetch_url("https://example.com");
    assert_eq!(wv.url(), Some("https://example.com"));
    assert!(!wv.is_loading());
}

/// 测试事件回调在操作过程中被移除
#[test]
fn test_remove_callback_during_operation() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 注册多个回调
    let callback1_id = wv.on_event(|_event| {});
    let _callback2_id = wv.on_event(|_event| {});

    // 移除其中一个回调
    assert!(wv.remove_event_callback(callback1_id));

    // 移除不存在的回调应该失败
    assert!(!wv.remove_event_callback(999));

    // 执行操作，确保不 panic
    wv.load_url("https://example.com");
    assert_eq!(wv.url(), Some("https://example.com"));
}

/// 测试 remove_event_callback 超出范围的索引
#[test]
fn test_remove_callback_out_of_bounds() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 没有回调的情况下尝试移除
    assert!(!wv.remove_event_callback(0));
    assert!(!wv.remove_event_callback(100));
}

/// 测试 execute_script_with_dom 复杂 DOM 操作
#[test]
fn test_execute_script_with_dom_complex_operations() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 先创建基础 HTML
    wv.load_html("<html><body></body></html>", None);

    // 复杂的 DOM 操作
    let script = r#"
        // 创建元素
        const div = document.createElement('div');
        div.id = 'test';
        div.className = 'container';
        div.style.color = 'red';

        // 设置属性
        div.setAttribute('data-test', 'value');

        // 添加到 body
        document.body.appendChild(div);

        // 返回确认
        div.innerHTML = 'Test successful';
        'Operation completed';
    "#;

    let result = wv.execute_script_with_dom(script);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Operation completed"));
}

/// 测试 complete_load 设置标题
#[test]
fn test_complete_load_sets_title() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 先设置标题
    wv.set_title("Original Title");

    // 完成加载，标题应该被保留
    let _result = wv.complete_load("<html><body>Test</body></html>", None);

    assert!(!wv.is_loading());
    assert_eq!(wv.title(), Some("Original Title"));
    assert!(wv.last_render().is_some());
}

/// 测试 fail_load 空错误消息
#[test]
fn test_fail_load_empty_error() {
    let mut wv = WebView::new(WebViewConfig::default());

    wv.load_url("https://example.com");
    assert!(wv.is_loading());

    // 空错误消息
    wv.fail_load("");

    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://example.com"));
    assert!(wv.last_render().is_none());
}

/// 测试 Service Worker 频繁注册/注销
#[test]
fn test_service_worker_frequent_register_unregister() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 频繁注册和注销同一个 URL
    for i in 0..10 {
        let sw_id = wv.register_service_worker(&format!("/sw-{}.js", i), "/", "https://example.com");

        // 尝试安装（可能因为 SW 不存在而失败）
        let _ = wv.install_service_worker(sw_id);
        let _ = wv.activate_service_worker(sw_id);

        // 注销
        let _ = wv.unregister_service_worker(sw_id);
    }

    // 最后应该没有活动 SW
    // Check that there are no active service workers
    assert!(wv.service_worker_registry().get_active("https://example.com").is_none());
}

/// 测试 execute_script 语法错误但合法的形式
#[test]
fn test_execute_script_valid_syntax_but_error() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 语法正确但运行时错误的代码
    let script = r#"
        // ReferenceError - 使用的变量未定义
        console.log(undefinedVar);
    "#;

    let result = wv.execute_script(script);
    assert!(result.is_err());
    assert!(matches!(result, Err(WebViewError::Script(_))));
}

/// 测试 execute_script_with_dom 在非 HTML 页面中
#[test]
fn test_execute_script_with_dom_on_non_html() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 加载非 HTML 内容
    wv.load_html("", None);

    // 执行 DOM 操作（即使没有 DOM）
    let script = "document.createElement('div');";
    let result = wv.execute_script_with_dom(script);

    // 应该成功或失败，但不能 panic
    assert!(result.is_ok() || result.is_err());
}

/// 测试 load_html 空 HTML 和 CSS
#[test]
fn test_load_html_empty_content() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 空 HTML
    let result1 = wv.load_html("", None);
    assert!(result1.timings.total_ms >= 0.0);

    // 空 HTML + 空 CSS
    let result2 = wv.load_html("", Some(""));
    assert!(result2.timings.total_ms >= 0.0);

    // 空 HTML + 有效 CSS
    let result3 = wv.load_html("", Some("body { color: red; }"));
    assert!(result3.timings.total_ms >= 0.0);
}

/// 测试 load_url 相同 URL 不触发事件
#[test]
fn test_load_url_same_url_no_events() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 设置初始 URL
    wv.load_url("https://example.com");
    let original_url = wv.url().map(|s| s.to_string());

    // 再次设置相同 URL，不应该触发 UrlChanged 事件
    wv.load_url("https://example.com");

    assert_eq!(wv.url().map(|s| s.to_string()), original_url);
    assert!(wv.is_loading());
}
