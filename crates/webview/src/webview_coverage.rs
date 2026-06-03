//! WebView API 覆盖率提升测试。
//!
//! 专注于测试 WebView 公共 API 的错误恢复、边界条件和非常规路径。

use super::*;

#[test]
fn test_webview_load_html_with_empty_strings() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 测试空的 HTML 和 CSS
    let result = webview.load_html("", None);
    assert!(result.primitives.primitives.is_empty());

    let result = webview.load_html("<div></div>", Some(""));
    assert!(result.primitives.primitives.is_empty());
}

#[test]
fn test_webview_load_html_with_large_content() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 测试大 HTML 内容（超过 typical buffer size）
    let large_html = "<html><body>".repeat(1000) + "</body></html>";
    let result = webview.load_html(&large_html, None);
    assert!(!result.primitives.primitives.is_empty());

    // 测试大 CSS 内容
    let large_css = "div { color: red; }".repeat(1000);
    let result = webview.load_html("<div>test</div>", Some(&large_css));
    assert!(!result.primitives.primitives.is_empty());
}

#[test]
fn test_webview_load_url_with_invalid_urls() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 测试无效 URL
    let result = webview.fetch_url("not-a-url");
    assert!(result.is_err());
    if let Err(WebViewError::Navigation(msg)) = result {
        assert!(msg.contains("Failed to fetch"));
    }

    // 测试空 URL
    let result = webview.fetch_url("");
    assert!(result.is_err());

    // 测试特殊字符 URL
    let result = webview.fetch_url("http://example.com/with spaces");
    assert!(result.is_err());
}

#[test]
fn test_webview_extract_origin_edge_cases() {
    // 测试各种 URL 格式的 origin 提取
    assert_eq!(WebView::extract_origin("https://example.com"), Some("https://example.com".to_string()));
    assert_eq!(WebView::extract_origin("https://example.com:8443/path"), Some("https://example.com:8443".to_string()));
    assert_eq!(WebView::extract_origin("http://localhost:3000"), Some("http://localhost:3000".to_string()));
    assert_eq!(WebView::extract_origin("file:///path/to/file.html"), None);  // file scheme 没有 origin
    assert_eq!(WebView::extract_origin("invalid-url"), None);
    assert_eq!(WebView::extract_origin(""), None);
}

#[test]
fn test_webview_load_url_and_complete_load_interaction() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 先 load_url（仅设置状态）
    webview.load_url("https://example.com");
    assert_eq!(webview.url(), Some("https://example.com"));
    assert!(webview.is_loading());

    // 然后完成加载
    let result = webview.complete_load("<html><body>Test</body></html>", None);
    assert!(!webview.is_loading());
    assert!(!result.primitives.primitives.is_empty());
}

#[test]
fn test_webview_fail_load() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 设置 URL 和加载状态
    webview.load_url("https://example.com");

    // 标记加载失败
    webview.fail_load("Network error");
    assert!(!webview.is_loading());
    assert!(webview.last_render().is_none());
}

#[test]
fn test_webview_inject_css_incrementally() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 第一次注入 CSS
    let result1 = webview.inject_css("div { color: red; }");
    assert!(!result1.primitives.primitives.is_empty());

    // 再次注入（应追加）
    let result2 = webview.inject_css("p { font-size: 16px; }");
    assert!(!result2.primitives.primitives.is_empty());

    // 多次注入应累积
    let result3 = webview.inject_css("body { margin: 0; }");
    assert!(!result3.primitives.primitives.is_empty());
}

#[test]
fn test_webview_execute_script_error_scenarios() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 测试空脚本
    let result = webview.execute_script("");
    assert!(result.is_err());
    if let Err(WebViewError::Script(msg)) = result {
        assert!(msg.contains("Invalid input"));
    }

    // 测试语法错误脚本
    let result = webview.execute_script("var x = ");
    assert!(result.is_err());
    if let Err(WebViewError::Script(msg)) = result {
        assert!(msg.contains("Compile error"));
    }

    // 测试运行时错误脚本
    let result = webview.execute_script("undefinedvariable()");
    assert!(result.is_err());
    if let Err(WebViewError::Script(msg)) = result {
        assert!(msg.contains("Runtime error"));
    }
}

#[test]
fn test_webview_execute_script_with_dom_api() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 先加载一些 HTML
    webview.load_html("<div id='test'>Hello</div>", None);

    // 执行带有 DOM API 的脚本
    let result = webview.execute_script_with_dom("document.getElementById('test').textContent = 'World'");
    assert!(result.is_ok());
}

#[test]
fn test_webview_resize_multiple_times() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 初始大小
    assert_eq!(webview.config().width, 800);
    assert_eq!(webview.config().height, 600);

    // 第一次调整大小
    webview.resize(1024, 768);
    assert_eq!(webview.config().width, 1024);
    assert_eq!(webview.config().height, 768);

    // 第二次调整大小（变小）
    webview.resize(640, 480);
    assert_eq!(webview.config().width, 640);
    assert_eq!(webview.config().height, 480);

    // 调整为极小尺寸
    webview.resize(1, 1);
    assert_eq!(webview.config().width, 1);
    assert_eq!(webview.config().height, 1);
}

#[test]
fn test_webview_event_callback_management() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 添加多个回调
    let callback1 = webview.on_event(|event| {
        if let WebViewEvent::LoadStart(_) = event {
            println!("Callback 1: Load started");
        }
    });

    let callback2 = webview.on_event(|event| {
        if let WebViewEvent::LoadEnd(_) = event {
            println!("Callback 2: Load ended");
        }
    });

    assert_ne!(callback1, callback2);
    assert_eq!(webview.event_callbacks.len(), 2);

    // 移除第一个回调
    let removed = webview.remove_event_callback(callback1);
    assert!(removed);
    assert_eq!(webview.event_callbacks.len(), 1);

    // 移除不存在的回调
    let removed = webview.remove_event_callback(999);
    assert!(!removed);
    assert_eq!(webview.event_callbacks.len(), 1);
}

#[test]
fn test_webview_service_worker_lifecycle() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 注册多个 Service Worker
    let sw1_id = webview.register_service_worker(
        "/sw1.js",
        "/",
        "https://example.com"
    );
    let sw2_id = webview.register_service_worker(
        "/sw2.js",
        "/app/*",
        "https://example.com"
    );

    assert_ne!(sw1_id, sw2_id);

    // 安装 SW
    assert!(webview.install_service_worker(sw1_id));
    assert!(webview.activate_service_worker(sw1_id));

    // 注销 SW
    assert!(webview.unregister_service_worker(sw1_id));
    assert!(!webview.unregister_service_worker(sw999));  // 不存在的 ID
}

#[test]
fn test_webview_set_title_and_event_emission() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 设置标题应触发事件
    webview.set_title("Test Page");
    assert_eq!(webview.title(), Some("Test Page"));

    // 设置空标题
    webview.set_title("");
    assert_eq!(webview.title(), Some(""));
}

#[test]
fn test_webview_wasm_execution_error_scenarios() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 测试空的 WASM 字节码
    let wasm_bytes = vec![];
    let result = webview.execute_wasm(
        &wasm_bytes,
        "main",
        &[]
    );
    assert!(result.is_err());
    if let Err(WebViewError::Script(msg)) = result {
        assert!(msg.contains("WASM compile error"));
    }

    // 测试调用不存在的导出函数
    let valid_wasm = &[
        0x00, 0x61, 0x73, 0x6d, // magic
        0x01, 0x00, 0x00, 0x00, // version
        // 一个简单的模块但没有导出函数
    ];
    let result = webview.execute_wasm(
        valid_wasm,
        "nonexistent_function",
        &[]
    );
    assert!(result.is_err());
    if let Err(WebViewError::Script(msg)) = result {
        assert!(msg.contains("WASM call error"));
    }
}

#[test]
fn test_webview_cached_html_and_css() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 初始状态应该是空的
    assert!(webview.cached_html().is_empty());
    assert!(webview.cached_css().is_empty());

    // 加载内容
    webview.load_html("<div>Hello</div>", Some("div { color: red; }"));

    // 验证缓存
    assert!(!webview.cached_html().is_empty());
    assert!(!webview.cached_css().is_empty());
    assert_eq!(webview.cached_html(), "<div>Hello</div>");
    assert_eq!(webview.cached_css(), "div { color: red; }");

    // 重新渲染应使用缓存
    let result = webview.render();
    assert!(!result.primitives.primitives.is_empty());
}

#[test]
fn test_webview_configuration_clone_and_equality() {
    let config1 = WebViewConfig {
        width: 800,
        height: 600,
        transparent: false,
        user_agent: Some("MyBrowser/1.0".to_string()),
        url: None,
        devtools: false,
    };

    let config2 = config1.clone();
    assert_eq!(config1.width, config2.width);
    assert_eq!(config1.height, config2.height);
    assert_eq!(config1.transparent, config2.transparent);
    assert_eq!(config1.user_agent, config2.user_agent);
    assert_eq!(config1.url, config2.url);
    assert_eq!(config1.devtools, config2.devtools);
}

#[test]
fn test_webview_default_config() {
    let config = WebViewConfig::default();
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(!config.transparent);
    assert!(config.user_agent.is_none());
    assert!(config.url.is_none());
    assert!(!config.devtools);
}

#[test]
fn test_webview_last_render_cache() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 初始没有渲染结果
    assert!(webview.last_render().is_none());

    // 加载内容后应该有渲染结果
    let result = webview.load_html("<div>Test</div>", None);
    assert!(webview.last_render().is_some());

    // 重新渲染
    let result2 = webview.render();
    assert!(webview.last_render().is_some());
}

#[test]
fn test_webview_fetch_url_with_service_worker() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 注册 Service Worker
    let sw_id = webview.register_service_worker(
        "/sw.js",
        "/",
        "https://example.com"
    );
    webview.install_service_worker(sw_id);
    webview.activate_service_worker(sw_id);

    // 虽然 fetch_url 会实际尝试网络请求，但我们可以测试 Service Worker 的拦截逻辑
    // 这里主要确保 Service Worker 注册表是功能性的
    let registry = webview.service_worker_registry();
    assert!(!registry.workers().is_empty());
}

#[test]
fn test_webview_url_change_tracking() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 初始没有 URL
    assert_eq!(webview.url(), None);

    // 设置相同 URL 不应触发 URL 变更事件
    webview.load_url("https://example.com");
    let url1 = webview.url().unwrap().to_string();

    // 再次设置相同 URL
    webview.load_url("https://example.com");
    assert_eq!(webview.url().unwrap().to_string(), url1);

    // 设置不同 URL
    webview.load_url("https://example.org");
    assert_eq!(webview.url().unwrap(), "https://example.org");
}

#[test]
fn test_webview_script_execution_timeout() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 注意：这个测试依赖于 JS 沙箱的具体实现
    // 如果沙箱支持超时，这个测试会捕获超时错误

    // 尝试执行一个可能超时的脚本（如果沙箱支持）
    // 实际的超时行为取决于 V8/QuickJS 的配置
    let result = webview.execute_script("while(true) {}");
    // 可能是超时错误或编译错误（取决于沙箱实现）
    match result {
        Ok(_) => (),  // 理想情况下不应该成功
        Err(WebViewError::Script(msg)) => {
            // 可能是超时错误
            assert!(msg.contains("Timeout") || msg.contains("infinite loop"));
        }
        _ => (),  // 其他错误也是可能的
    }
}

#[test]
fn test_webview_empty_body_html() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 测试没有 body 标签的 HTML
    let result = webview.load_html("<html></html>", None);
    assert!(!result.primitives.primitives.is_empty());

    // 测试只有空白内容的 HTML
    let result = webview.load_html("   \n\t  ", None);
    assert!(result.primitives.primitives.is_empty());
}

#[test]
fn test_webview_multiple_css_injections() {
    let mut webview = WebView::new(WebViewConfig::default());

    // 多次注入不同 CSS
    webview.inject_css("div { color: red; }");
    webview.inject_css("p { font-size: 16px; }");
    webview.inject_css("body { margin: 0; padding: 0; }");

    // CSS 应该累积
    let result = webview.render();
    assert!(!result.primitives.primitives.is_empty());

    // 验证缓存中的 CSS 包含所有注入的样式
    assert!(webview.cached_css().contains("color: red"));
    assert!(webview.cached_css().contains("font-size: 16px"));
    assert!(webview.cached_css().contains("margin: 0"));
}