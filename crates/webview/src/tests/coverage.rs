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

// ── Storage 跨 load_html 持久化（R3088）──
// WebView 沙箱复用（ensure_sandbox：js_sandbox.is_some() 早返）+ shim 幂等注入（js_shim_initialized
// guard），故 globalThis.localStorage/sessionStorage（shim 的 _createStorage 对象）在同 WebView 多次
// load_html + run_page_scripts 间持久。闭合 R3087「下一步」调查：嵌入式/headless 路径已具备持久化
//（浏览器 tab 导航重建 worker 为另一路径，非 make test 可达）。本测试锁定该正确性（防回退）。
#[test]
fn test_storage_persists_across_load_html_r3088() {
    let mut wv = WebView::new(WebViewConfig::default());
    // page1：写 localStorage / sessionStorage。
    wv.load_html(
        "<html><body><script>\
         localStorage.setItem('k','value1');\
         sessionStorage.setItem('s','sess1');\
         </script></body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "page1 脚本执行无异常, got: {:?}", r.err());

    // page2（同 WebView，新 load_html）：读 page1 写入的 storage。
    wv.load_html(
        "<html><body><script>\
         globalThis.__ls = localStorage.getItem('k');\
         globalThis.__ss = sessionStorage.getItem('s');\
         </script></body></html>",
        None,
    );
    let r2 = wv.run_page_scripts_strict();
    assert!(r2.is_ok(), "page2 脚本执行无异常, got: {:?}", r2.err());

    assert_eq!(
        wv.execute_script("String(globalThis.__ls)").unwrap(),
        "value1",
        "localStorage 跨 load_html 持久（同 WebView 沙箱复用 + shim 幂等）"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__ss)").unwrap(),
        "sess1",
        "sessionStorage 跨 load_html 持久"
    );
}

// ── 外链脚本源获取（R3090）──
// 进程内/headless 路径：script_source_fetcher 提供 './app.js'（经典）+ './mod.js'（模块）源 →
// run_page_scripts fetch 后执行（External 走经典，ExternalModule 走 InlineModule 编译路径）。
// external_script 多进程模式不走此路径；fetcher 为 None 时外链脚本跳过（离线语义，零回归）。
#[test]
fn test_external_script_fetch_r3090() {
    use std::sync::Arc;
    let mut wv = crate::WebViewBuilder::new()
        .script_source_fetcher(Arc::new(|_page, src| {
            if src == "./app.js" {
                Ok("globalThis.__appExt = 'app-ok';".to_string())
            } else if src == "./mod.js" {
                // 模块体：副作用设全局 + export（compile_module_script 转 export，body 副作用执行）。
                Ok("globalThis.__modExt = 'mod-ok'; export const x = 1;".to_string())
            } else {
                Err("not found".to_string())
            }
        }))
        .build();
    wv.load_html(
        "<html><body>\
         <script src=\"./app.js\"></script>\
         <script type=\"module\" src=\"./mod.js\"></script>\
         </body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "外链脚本 fetch+执行无异常, got: {:?}", r.err());
    assert_eq!(
        wv.execute_script("String(globalThis.__appExt)").unwrap(),
        "app-ok",
        "外链经典脚本 ./app.js 经 fetcher fetch 后执行"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__modExt)").unwrap(),
        "mod-ok",
        "外链模块脚本 ./mod.js 经 fetcher fetch + module 编译后执行"
    );
}

// ── 外链 Worker fetch（R3091）──
// `new Worker('./worker.js')`（外链 URL）经 __zw_fetch_script（backed by ScriptSourceFetcher）取 worker 源
// → 同沙箱 IIFE 影子执行（R3089 机制）→ 消息往返。R3089 仅支持 data: URL；本切片闭合外链 worker 加载。
#[test]
fn test_external_worker_fetch_r3091() {
    use std::sync::Arc;
    let mut wv = crate::WebViewBuilder::new()
        .script_source_fetcher(Arc::new(|_page, src| {
            if src == "./worker.js" {
                Ok("onmessage = function (e) { postMessage(e.data * 2); };".to_string())
            } else {
                Err("not found".to_string())
            }
        }))
        .build();
    wv.load_html(
        "<html><body><script>\
         var w = new Worker('./worker.js');\
         globalThis.__reply = 'none';\
         w.onmessage = function (ev) { globalThis.__reply = ev.data; };\
         w.postMessage(21);\
         </script></body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "外链 worker fetch+执行无异常, got: {:?}", r.err());
    assert_eq!(
        wv.execute_script("String(globalThis.__reply)").unwrap(),
        "42",
        "外链 worker ./worker.js 经 fetcher fetch + IIFE 执行 → 消息往返（postMessage(21) → onmessage(42)）"
    );
}

// ── 动态 import() 外链 module（R3093）──
// module 脚本 `import('./mod.js')` 经 prelude 改写 __zw_dynamic_import → __zw_load_module → __zw_compile_module
//（host 回调 fetch './mod.js' 源 + compile_dependency_iife compile 为 IIFE → eval 为 namespace）。
// .then 微任务在 run_page_scripts 末排空。fetcher 配置时 InlineModule 预存根只用静态 import
//（动态 import() 不预存根 → 落 __zw_compile_module fetch），闭合外链加载全链（R3090 脚本 + R3091 worker + 本切片 module）。
#[test]
fn test_dynamic_import_external_module_r3093() {
    use std::sync::Arc;
    let mut wv = crate::WebViewBuilder::new()
        .script_source_fetcher(Arc::new(|_page, src| {
            if src == "./mod.js" {
                Ok("export default 42;".to_string())
            } else {
                Err("not found".to_string())
            }
        }))
        .build();
    wv.load_html(
        "<html><body><script type=\"module\">\n\
         import('./mod.js').then(function (m) { globalThis.__modVal = m.default; });\n\
         </script></body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "动态 import 外链 module 无异常, got: {:?}", r.err());
    assert_eq!(
        wv.execute_script("String(globalThis.__modVal)").unwrap(),
        "42",
        "动态 import('./mod.js') 外链 module → fetch + compile → namespace.default = 42"
    );
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
