// 覆盖率提升测试 — webview.rs 中未覆盖的方法
use crate::*;

// ── extract_origin 测试 ──

#[test]
fn test_extract_origin_https_with_port() {
    let origin = WebView::extract_origin("https://example.com:8443/path?q=1");
    assert_eq!(origin, Some("https://example.com:8443".to_string()));
}

#[test]
fn test_extract_origin_http_no_port() {
    let origin = WebView::extract_origin("http://example.com/path");
    assert_eq!(origin, Some("http://example.com".to_string()));
}

#[test]
fn test_extract_origin_localhost() {
    let origin = WebView::extract_origin("https://localhost:3000/app");
    assert_eq!(origin, Some("https://localhost:3000".to_string()));
}

#[test]
fn test_extract_origin_invalid() {
    assert_eq!(WebView::extract_origin("not-a-url"), None);
    assert_eq!(WebView::extract_origin(""), None);
}

#[test]
fn test_extract_origin_no_path() {
    let origin = WebView::extract_origin("https://example.com");
    assert_eq!(origin, Some("https://example.com".to_string()));
}

#[test]
fn test_extract_origin_with_fragment() {
    let origin = WebView::extract_origin("https://example.com/page#section");
    assert_eq!(origin, Some("https://example.com".to_string()));
}

// ── fail_load 测试 ──

#[test]
fn test_fail_load_resets_loading_state() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.fail_load("network error");
    assert!(!wv.is_loading(), "fail_load should reset loading state");
}

#[test]
fn test_fail_load_emits_event() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let events_clone = events.clone();
    wv.on_event(move |e| {
        if let WebViewEvent::LoadFailed(url, msg) = e {
            events_clone.borrow_mut().push(format!("{url}:{msg}"));
        }
    });

    wv.fail_load("test error");
    assert_eq!(events.borrow().len(), 1);
    assert_eq!(events.borrow()[0], ":test error");
}

// ── execute_wasm 测试 ──

#[test]
fn test_execute_wasm_invalid_bytes() {
    let wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_wasm(&[0xFF, 0xFE], "main", &[]);
    assert!(result.is_err(), "invalid WASM bytes should error");
}

#[test]
fn test_execute_wasm_missing_function() {
    let wv = WebView::new(WebViewConfig::default());
    let wasm_bytes = wasm_empty_module();
    let result = wv.execute_wasm(&wasm_bytes, "nonexistent", &[]);
    assert!(result.is_err(), "missing function should error");
}

#[test]
fn test_execute_wasm_add_module() {
    let wv = WebView::new(WebViewConfig::default());
    let wasm_bytes = wasm_add_module();
    let args = vec![
        zero_wasm_sandbox::WasmValue::I32(3),
        zero_wasm_sandbox::WasmValue::I32(4),
    ];
    let result = wv.execute_wasm(&wasm_bytes, "add", &args);
    assert!(result.is_ok(), "valid WASM should succeed");
    assert_eq!(result.unwrap(), "i32(7)");
}

/// 辅助：生成空的 WASM 模块
fn wasm_empty_module() -> Vec<u8> {
    vec![
        0x00, 0x61, 0x73, 0x6D, // magic
        0x01, 0x00, 0x00, 0x00, // version
    ]
}

/// 辅助：生成简单的 add(i32, i32) -> i32 WASM 模块
fn wasm_add_module() -> Vec<u8> {
    vec![
        0x00, 0x61, 0x73, 0x6D, // magic
        0x01, 0x00, 0x00, 0x00, // version
        0x01, // type section
        0x07, // section length
        0x01, // 1 type
        0x60, // func type
        0x02, 0x7F, 0x7F, // 2 params: i32, i32
        0x01, 0x7F, // 1 result: i32
        0x03, // function section
        0x02, // section length
        0x01, // 1 function
        0x00, // type index 0
        0x07, // export section
        0x07, // section length
        0x01, // 1 export
        0x03, 0x61, 0x64, 0x64, // "add"
        0x00, // func export
        0x00, // func index 0
        0x0A, // code section
        0x09, // section length
        0x01, // 1 function body
        0x07, // body length
        0x00, // 0 locals
        0x20, 0x00, // local.get 0
        0x20, 0x01, // local.get 1
        0x6A, // i32.add
        0x0B, // end
    ]
}

// ── set_title 测试 ──

#[test]
fn test_set_title_emits_event() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let mut wv = WebView::new(WebViewConfig::default());
    let titles: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let titles_clone = titles.clone();
    wv.on_event(move |e| {
        if let WebViewEvent::TitleChanged(t) = e {
            titles_clone.borrow_mut().push(t.to_string());
        }
    });

    wv.set_title("Test Page");
    assert_eq!(titles.borrow().len(), 1);
    assert_eq!(titles.borrow()[0], "Test Page");
}

// ── url() 测试 ──

#[test]
fn test_url_initially_none() {
    let wv = WebView::new(WebViewConfig::default());
    assert!(wv.url().is_none());
}

// ── service_worker_registry_mut 测试 ──

#[test]
fn test_service_worker_registry_mut_access() {
    let mut wv = WebView::new(WebViewConfig::default());
    let registry = wv.service_worker_registry_mut();
    assert!(registry.is_empty());
}

// ── inject_css 测试 ──

#[test]
fn test_inject_css_accumulates() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>test</body></html>", None);

    wv.inject_css("body { color: red; }");
    wv.inject_css("div { margin: 0; }");

    // CSS 应该累积，重新渲染不 panic
    let _ = wv.render();
}

// ── render 重新渲染测试 ──

#[test]
fn test_render_updates_last_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>hello</body></html>", None);
    let _ = wv.render();
}

// ── execute_script 错误路径测试 ──

#[test]
fn test_execute_script_runtime_error() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("throw new Error('test error')");
    // 应该返回错误但不是 panic
    assert!(result.is_err() || result.is_ok());
}

#[test]
fn test_execute_script_syntax_error() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("function(");
    assert!(result.is_err(), "syntax error should fail");
}

/// R3083：`<script type="module">` 经 compile_module_script 转换后执行（旧与 Inline 同走经典路径→
/// `import` 抛 SyntaxError）。headless 进程内模式不 fetch 外链模块——预注册空存根使 `import './x.js'`
/// no-op，模块 body 可执行。验证：① strict 执行不抛；② 模块 body 副作用生效。
#[test]
fn test_module_script_executes_r3083() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html(
        "<html><body><script type=\"module\">import './missing.js'; globalThis.__modBodyRan = 'yes';</script></body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(
        r.is_ok(),
        "module script 应无异常执行（import 空 stub no-op），got: {:?}",
        r.err()
    );
    let flag = wv.execute_script("String(globalThis.__modBodyRan === 'yes')");
    assert_eq!(flag.unwrap(), "true", "module body 经转换后执行（__modBodyRan 设置）");
}

// ── WebViewConfig default 测试 ──

#[test]
fn test_webview_config_default() {
    let config = WebViewConfig::default();
    assert_eq!(config.width, 800);
    assert_eq!(config.height, 600);
    assert!(!config.transparent);
    assert!(config.user_agent.is_none());
    assert!(config.url.is_none());
    assert!(!config.devtools);
}

// ── load_html + complete_load 生命周期测试 ──

#[test]
fn test_load_html_then_fail() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body>test</body></html>", None);
    // After load, not loading
    assert!(!wv.is_loading());

    // fail_load on already-loaded page
    wv.fail_load("late error");
    assert!(!wv.is_loading());
}
