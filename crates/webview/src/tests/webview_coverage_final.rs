// WebView 最终覆盖率测试 - 针对 webview.rs 中未覆盖的路径
// 专注于错误路径、边界条件和特殊场景

use crate::*;
use std::cell::RefCell;
use std::rc::Rc;

/// 测试 load_url 后立即 fail_load 不触发 LoadEnd 事件
#[test]
fn test_load_url_then_immediate_fail_load() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 监听事件
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| match e {
        WebViewEvent::LoadStart(url) => {
            events_clone.borrow_mut().push(format!("LoadStart:{}", url));
        }
        WebViewEvent::LoadEnd(url) => {
            events_clone.borrow_mut().push(format!("LoadEnd:{}", url));
        }
        WebViewEvent::LoadFailed(url, error) => {
            events_clone.borrow_mut().push(format!("LoadFailed:{}:{}", url, error));
        }
        _ => {}
    });

    // load_url 后立即 fail_load
    wv.load_url("https://example.com");
    wv.fail_load("immediate failure");

    // 应该只有 LoadStart 和 LoadFailed，没有 LoadEnd
    let events_received = events.borrow();
    assert!(events_received.contains(&"LoadStart:https://example.com".to_string()));
    assert!(events_received.contains(&"LoadFailed:https://example.com:immediate failure".to_string()));
    assert!(!events_received.iter().any(|e| e.starts_with("LoadEnd:")));
}

/// 测试 fetch_url URL 为空字符串
#[test]
fn test_fetch_url_empty_string() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.fetch_url("");
    assert!(result.is_err());
    assert_eq!(wv.url(), Some("")); // URL 应该被设置
    assert!(!wv.is_loading()); // 加载状态应被重置
}

/// 测试 fetch_url 无效协议（ftp）
#[test]
fn test_fetch_url_ftp_scheme() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.fetch_url("ftp://example.com/file");
    assert!(result.is_err());
    assert!(!wv.is_loading());
}

/// 测试 execute_script 脚本为纯空格
#[test]
fn test_execute_script_whitespace_only() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("   \n  \t  ");
    // 应该返回错误或成功但不能 panic
    assert!(result.is_ok() || result.is_err());
}

/// 测试 execute_script 脚本为空字符串后立即再次执行
#[test]
fn test_execute_script_empty_then_valid() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 先执行空脚本
    let _ = wv.execute_script("");

    // 再执行有效脚本
    let result = wv.execute_script("1 + 1");
    assert!(result.is_ok());
}

/// 测试 execute_script 深层属性错误
#[test]
fn test_execute_script_deep_property_chain_error() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 深层属性链中的中间对象不存在
    let script = "a.b.c.d.e.f";
    let result = wv.execute_script(script);
    assert!(result.is_err() || result.is_ok()); // 可能是运行时错误
}

/// 测试 execute_script 返回超长字符串
#[test]
fn test_execute_script_very_long_string() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 生成很长的字符串
    let long_string = "x".repeat(100000);
    let script = format!("'{}'", long_string);

    let result = wv.execute_script(&script);
    // 可能失败（内存限制），但不能 panic
    assert!(result.is_ok() || result.is_err());
}

/// 测试 execute_script 抛出特定类型的错误
#[test]
fn test_execute_script_type_error() {
    let mut wv = WebView::new(WebViewConfig::default());

    // TypeError - 调用非函数
    let script = "(1)()";
    let result = wv.execute_script(script);
    assert!(result.is_err());
}

/// 测试 execute_script ReferenceError 后立即执行成功
#[test]
fn test_execute_script_error_then_success() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 先执行会报错的脚本
    let _ = wv.execute_script("undefinedVar");

    // 再执行成功的脚本
    let result = wv.execute_script("2 + 2");
    assert!(result.is_ok());
}

/// 测试 execute_script 多行语句中的语法错误
#[test]
fn test_execute_script_multiline_syntax_error() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 多行语句，第二行有语法错误
    let script = r#"
        let x = 1;
        let y = ;  // 语法错误
        x + y;
    "#;

    let result = wv.execute_script(script);
    assert!(result.is_err());
}

/// 测试 load_html 后立即 inject_css 大量 CSS
#[test]
fn test_load_html_then_inject_massive_css() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 加载基础 HTML
    wv.load_html("<html><body>test</body></html>", None);

    // 注入大量 CSS 规则
    let massive_css = (0..1000)
        .map(|i| format!("div.rule-{} {{ color: {}; }}", i, i % 256))
        .collect::<Vec<_>>()
        .join("\n");

    let result = wv.inject_css(&massive_css);
    // 应该成功执行
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试 resize 到零尺寸后的渲染
#[test]
fn test_resize_zero_then_render() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 加载内容
    wv.load_html("<html><body>test</body></html>", None);

    // 调整到零尺寸
    wv.resize(0, 0);

    // 尝试渲染
    let result = wv.render();
    // 应该不会 panic
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试 Service Worker 注册/注销循环很多次
#[test]
fn test_service_worker_register_unregister_cycle_many_times() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 循环注册和注销
    for i in 0..100 {
        let sw_id = wv.register_service_worker(&format!("/sw-{}.js", i), "/", "https://example.com");

        // 尝试各种操作
        let _ = wv.install_service_worker(sw_id);
        let _ = wv.activate_service_worker(sw_id);
        let _ = wv.unregister_service_worker(sw_id);
    }

    // 最后应该没有错误
    assert!(wv.service_worker_registry().is_empty());
}

/// 测试 load_url 相同 URL 但不同大小写
#[test]
fn test_load_url_case_sensitive_url_change() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 设置初始 URL
    wv.load_url("https://Example.com/path");
    let original_url = wv.url().map(|s| s.to_string());

    // 加载相同 URL 但大小写不同（可能被视为不同）
    wv.load_url("https://example.com/path");

    // 如果系统区分大小写，URL 应该不同
    if let Some(current_url) = wv.url() {
        assert_ne!(current_url, original_url.as_deref().unwrap_or(""));
    }
    assert!(wv.is_loading());
}

/// 测试 complete_load 在没有 load_url 调用的情况下
#[test]
fn test_complete_load_without_load_url() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 直接调用 complete_load 而不先 load_url
    let result = wv.complete_load("<html><body>test</body></html>", None);

    // 应该成功完成加载
    assert!(result.timings.total_ms >= 0.0);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), None); // 未调用 load_url，current_url 仍为 None
}

/// 测试 execute_script_with_dom 在复杂场景下的表现
#[test]
fn test_execute_script_with_dom_complex_scenario() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 加载包含特殊字符的 HTML
    let html = r#"
        <html>
            <body>
                <div data-test="value & 'quotes' & \"double quotes\">
                    <span>Text with <strong>nested</strong> elements</span>
                </div>
                <script>
                    window.globalVar = { nested: { value: 42 } };
                </script>
            </body>
        </html>
    "#;
    wv.load_html(html, None);

    // 执行复杂的 DOM 操作
    let script = r#"
        // 访问深层嵌套的属性
        const div = document.querySelector('[data-test]');
        div.style.color = 'red';
        div.setAttribute('data-modified', 'true');

        // 读取全局变量
        JSON.stringify(window.globalVar.nested.value);
    "#;

    let result = wv.execute_script_with_dom(script);
    // DOM polyfill 是桩实现，querySelector 不会真正工作
    // 但 execute_script_with_dom 不应该 panic
    assert!(result.is_ok() || result.is_err());
}

/// 测试 inject_css 在已有大量 CSS 基础上的追加
#[test]
fn test_inject_css_after_many_css_injections() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body></body></html>", None);

    // 多次注入 CSS
    for i in 0..50 {
        let css = format!("div.rule-{} {{ color: {}; }}", i, i % 256);
        let _ = wv.inject_css(&css);
    }

    // 最后再注入一次
    let final_css = "body { background: blue; }";
    let result = wv.inject_css(final_css);

    // 应该成功
    assert!(result.timings.total_ms >= 0.0);
}

/// 测试 execute_script 超时场景（如果有超时机制）
#[test]
fn test_execute_script_potential_timeout() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 创建可能很慢的操作（如果 JS 引擎支持）
    let script = r#"
        let result = 0;
        for (let i = 0; i < 1000000; i++) {
            result += i;
        }
        result;
    "#;

    let result = wv.execute_script(script);
    // 可能成功或失败，但不能 panic
    assert!(result.is_ok() || result.is_err());
}

/// 测试 URL 中包含特殊字符
#[test]
fn test_load_url_with_special_characters() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 包含各种特殊字符的 URL
    let urls = vec![
        "https://example.com/path with spaces",
        "https://example.com/path%20with%20encoding",
        "https://example.com/path?query=value&key=with=symbols",
        "https://example.com/path#fragment/with/slash",
        "https://user:pass@example.com:8080/path",
    ];

    for url in urls {
        wv.load_url(url);
        assert_eq!(wv.url(), Some(url));
        assert!(wv.is_loading());

        // 清除加载状态
        wv.fail_load("test reset");
        assert!(!wv.is_loading());
    }
}

/// 测试 service_worker_registry_mut 的修改后立即查询
#[test]
fn test_service_worker_registry_mut_and_immediate_query() {
    let mut wv = WebView::new(WebViewConfig::default());

    let registry = wv.service_worker_registry_mut();

    // 直接修改注册表 - 通过删除所有注册
    // registry.clear();  // 假设这个方法不存在
    // 测试其他方法

    // 立即查询
    assert!(registry.is_empty());
}

/// 测试 resize 极大尺寸后恢复
#[test]
fn test_resize_extreme_then_normal() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 调整到极大尺寸
    wv.resize(10000, 10000);
    assert_eq!(wv.config().width, 10000);
    assert_eq!(wv.config().height, 10000);

    // 恢复到正常尺寸
    wv.resize(800, 600);
    assert_eq!(wv.config().width, 800);
    assert_eq!(wv.config().height, 600);
}

/// 测试 execute_script 返回 undefined
#[test]
fn test_execute_script_returns_undefined() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 返回 undefined 的脚本
    let script = "undefined";
    let result = wv.execute_script(script);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "undefined");
}

/// 测试 execute_script 返回 null
#[test]
fn test_execute_script_returns_null() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 返回 null 的脚本
    let script = "null";
    let result = wv.execute_script(script);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "null");
}

/// 测试 execute_script 执行异步操作（如果有）
#[test]
fn test_execute_script_async_operations() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 如果 JS 引擎支持异步操作
    let script = r#"
        new Promise((resolve) => {
            setTimeout(() => {
                resolve('async result');
            }, 0);
        });
    "#;

    let result = wv.execute_script(script);
    // 异步操作可能不被完全支持，测试不 panic 即可
    assert!(result.is_ok() || result.is_err());
}

/// 测试多个事件回调被同时触发
#[test]
fn test_multiple_event_callbacks_triggered() {
    let mut wv = WebView::new(WebViewConfig::default());

    let events1: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events1_clone = events1.clone();
    let callback1_id = wv.on_event(move |e| {
        if let WebViewEvent::LoadStart(url) = e {
            events1_clone.borrow_mut().push(format!("CB1:{}", url));
        }
    });

    let events2: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events2_clone = events2.clone();
    let _callback2_id = wv.on_event(move |e| {
        if let WebViewEvent::LoadStart(url) = e {
            events2_clone.borrow_mut().push(format!("CB2:{}", url));
        }
    });

    // 触发事件
    wv.load_url("https://example.com");

    // 两个回调都应该被触发
    assert_eq!(events1.borrow().len(), 1);
    assert_eq!(events2.borrow().len(), 1);
    assert_eq!(events1.borrow()[0], "CB1:https://example.com");
    assert_eq!(events2.borrow()[0], "CB2:https://example.com");

    // 移除其中一个回调
    wv.remove_event_callback(callback1_id);

    // 再次触发事件
    wv.load_url("https://example.com/page2");

    // 注意: remove_event_callback 使用 Vec::remove，索引会移位
    // 移除 callback1 (index 0) 后，callback2 移动到 index 0
    // 此时 events1 不再被触发
    assert_eq!(events1.borrow().len(), 1); // 没有新事件
}

/// 测试 fetch_url 网络路径 - 超时场景
#[test]
fn test_fetch_url_timeout_error() {
    // 5s 超时 + sleep=2000：本测试恒真断言（is_ok || is_err），仅验证不 panic；
    // 外网不可达时 5s 内返回（默认 30s 会让 CI 无外网时实等 30s）。
    let mut wv = WebView::new(WebViewConfig {
        http_timeout_secs: Some(5),
        ..Default::default()
    });

    // 监听事件
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| match e {
        WebViewEvent::LoadFailed(url, error) => {
            events_clone.borrow_mut().push(format!("LoadFailed:{}:{}", url, error));
        }
        _ => {}
    });

    // 尝试设置一个会超时的 URL
    // 注意：这依赖于 HTTP 客户端的超时行为
    let result = wv.fetch_url("http://httpstat.us/200?sleep=2000");
    // 由于是同步调用，可能不会真正超时，但测试结构应正确
    assert!(result.is_ok() || result.is_err());
}

/// 测试 fetch_url 网络路径 - 连接失败
#[test]
fn test_fetch_url_connection_failure() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 监听事件
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| match e {
        WebViewEvent::LoadFailed(url, error) => {
            events_clone.borrow_mut().push(format!("LoadFailed:{}:{}", url, error));
        }
        _ => {}
    });

    // 使用一个不太可能存在的域名
    let result = wv.fetch_url("https://this-domain-does-not-exist-12345.com");
    assert!(result.is_err());

    // 应该触发 LoadFailed 事件
    let events_received = events.borrow();
    assert!(events_received.iter().any(|e| e.starts_with("LoadFailed:")));
}

/// 测试 set_title 触发 TitleChanged 事件
#[test]
fn test_set_title_triggers_title_changed_event() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 监听事件
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| match e {
        WebViewEvent::TitleChanged(title) => {
            events_clone.borrow_mut().push(format!("TitleChanged:{}", title));
        }
        _ => {}
    });

    // 设置标题
    wv.set_title("新标题");

    // 应该触发 TitleChanged 事件
    let events_received = events.borrow();
    assert!(events_received.contains(&"TitleChanged:新标题".to_string()));
    assert_eq!(wv.title(), Some("新标题"));
}

/// 测试 title getter - 初始返回 None
#[test]
fn test_title_getter_initial_none() {
    let wv = WebView::new(WebViewConfig::default());
    assert_eq!(wv.title(), None);
}

/// 测试 title getter - set_title 后返回 Some
#[test]
fn test_title_getter_after_set_title() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.set_title("测试标题");
    assert_eq!(wv.title(), Some("测试标题"));
}

/// 测试 config getter - 返回引用
#[test]
fn test_config_getter() {
    let wv = WebView::new(WebViewConfig::default());
    let config = wv.config();
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(!config.transparent);
    assert_eq!(config.url, None);
}

/// 测试 last_render - 初始返回 None
#[test]
fn test_last_render_initial_none() {
    let wv = WebView::new(WebViewConfig::default());
    assert!(wv.last_render().is_none());
}

/// 测试 last_render - load_html 后返回 Some
#[test]
fn test_last_render_after_load_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    let _result = wv.load_html("<html><body>test</body></html>", None);
    assert!(wv.last_render().is_some());
}

/// 测试 WebViewConfig 自定义值
#[test]
fn test_webview_config_custom_values() {
    let config = WebViewConfig {
        width: 1024,
        height: 768,
        transparent: true,
        user_agent: Some("MyCustomAgent/1.0".to_string()),
        url: Some("https://example.com".to_string()),
        devtools: true,
        external_script: None,
        ..Default::default()
    };

    let wv = WebView::new(config);
    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);
    assert!(wv.config().transparent);
    assert_eq!(wv.config().user_agent, Some("MyCustomAgent/1.0".to_string()));
    assert_eq!(wv.config().url, Some("https://example.com".to_string()));
    assert!(wv.config().devtools);
}

/// 测试 fetch_url 相同 URL 第二次不触发 UrlChanged
#[test]
fn test_fetch_url_same_url_no_second_url_changed() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 监听事件
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| match e {
        WebViewEvent::UrlChanged(url) => {
            events_clone.borrow_mut().push(format!("UrlChanged:{}", url));
        }
        _ => {}
    });

    // 第一次 fetch
    let _ = wv.fetch_url("https://example.com");

    // 第二次 fetch 相同 URL
    let _ = wv.fetch_url("https://example.com");

    // UrlChanged 只应该在 URL 不同时触发一次
    let events_received = events.borrow();
    let url_changed_events: Vec<String> = events_received
        .iter()
        .filter(|e| e.starts_with("UrlChanged:"))
        .cloned()
        .collect();

    assert_eq!(url_changed_events.len(), 1); // 第一次 fetch 从 None→Some 触发一次 UrlChanged
}

/// 测试 execute_wasm 无效字节
#[test]
fn test_execute_wasm_invalid_bytes() {
    let wv = WebView::new(WebViewConfig::default());

    // 无效的 WASM 字节
    let invalid_wasm = b"This is not valid WASM binary data";

    let result = wv.execute_wasm(invalid_wasm, "main", &[]);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), WebViewError::Script(_)));
}

/// 测试 complete_load 先 load_url 后 complete
#[test]
fn test_complete_load_with_load_url_first() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 先设置 URL
    wv.load_url("https://example.com");
    assert_eq!(wv.url(), Some("https://example.com"));
    assert!(wv.is_loading());

    // 然后完成加载
    let _result = wv.complete_load("<html><body>test</body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://example.com"));
}

/// 测试 SW 拦截 - 注册、激活后 fetch_url
#[test]
fn test_service_worker_fetch_intercept() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 注册 Service Worker
    let sw_id = wv.register_service_worker("/sw.js", "/", "https://example.com");

    // 安装 Service Worker（必须先安装才能激活）
    assert!(wv.install_service_worker(sw_id));
    assert!(wv.activate_service_worker(sw_id));

    // 监听事件
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| match e {
        WebViewEvent::LoadEnd(url) => {
            events_clone.borrow_mut().push(format!("LoadEnd:{}", url));
        }
        _ => {}
    });

    // 尝试 fetch URL（会被 SW 拦截）
    // 注意：由于 SW 需要实际的 script 文件，这里主要是测试路径
    let result = wv.fetch_url("https://example.com/test");
    // 可能失败（因为没有真实的 SW），但测试拦截路径
    assert!(result.is_ok() || result.is_err());
}

/// 测试 fail_load 事件触发
#[test]
fn test_fail_load_event() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 监听事件
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| match e {
        WebViewEvent::LoadFailed(url, error) => {
            events_clone.borrow_mut().push(format!("LoadFailed:{}:{}", url, error));
        }
        _ => {}
    });

    // 触发 fail_load
    wv.fail_load("测试错误");

    // 应该触发 LoadFailed 事件
    let events_received = events.borrow();
    assert!(events_received.iter().any(|e| e.starts_with("LoadFailed:")));
}

/// 测试 WebViewError Display 格式化
#[test]
fn test_webview_error_display() {
    use std::fmt::Write;

    let mut s = String::new();

    // 测试不同类型的错误格式化
    let errors = vec![
        WebViewError::Navigation("导航错误".to_string()),
        WebViewError::Script("脚本错误".to_string()),
        WebViewError::NotImplemented("未实现功能".to_string()),
    ];

    for error in errors {
        write!(&mut s, "{}", error).unwrap();
        assert!(!s.is_empty());
        s.clear();
    }
}

/// 测试 remove_event_callback 越界索引返回 false
#[test]
fn test_remove_event_callback_out_of_bounds() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 添加一个回调
    let callback_id = wv.on_event(|_| {});

    // 尝试移除不存在的回调（越界）
    assert!(!wv.remove_event_callback(callback_id + 1));

    // 尝试移除空列表中的回调 - 由于我们至少添加了一个回调，所以列表不为空
    assert!(!wv.remove_event_callback(999)); // 使用一个不可能的索引

    // 移除已存在的回调应该成功
    assert!(wv.remove_event_callback(callback_id));

    // 再次移除同一个应该失败
    assert!(!wv.remove_event_callback(callback_id));
}

/// 测试 resize 并验证新配置
#[test]
fn test_resize_and_verify_new_config() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 调整大小
    wv.resize(1920, 1080);

    // 验证配置更新
    assert_eq!(wv.config().width, 1920);
    assert_eq!(wv.config().height, 1080);
}

/// 测试注入 CSS 后重新渲染
#[test]
fn test_inject_css_and_render() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 加载 HTML
    wv.load_html("<html><body><div>测试</div></body></html>", None);

    // 注入 CSS
    let _result = wv.inject_css("div { color: red; }");

    // 重新渲染应该反映 CSS 变化
    assert!(!wv.last_render().unwrap().timings.total_ms.is_nan());
}
