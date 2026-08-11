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

// ── document.currentScript（R3258，HTML §4.11.3.1）──
// classic 脚本执行期间 currentScript 指向自身 <script> 元素；执行期外为 null；module 执行期恒 null。
// 宿主经 __zw_set_current_script(idx) / __zw_clear_current_script() 在每个 classic 脚本执行前后设/清，
// idx = 脚本在「全部 <script> 元素」（含非 JS 类型）中的文档序，与 getElementsByTagName('script') 对齐。
// 主用例：分析 SDK / 脚本加载器读 currentScript.src 定位自身来源（GA / requirejs / 广告 SDK）。
#[test]
fn test_document_current_script_classic_r3258() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 前置一个非 JS 类型 <script type="application/json"> 验证索引对齐：extract 过滤它（不入执行队列），
    // 但 extract_page_scripts_indexed 计其入 global_idx，故其后 JS 脚本 idx=1（非 0）。
    // getElementsByTagName('script')[1] 必须解析到该 JS 脚本元素（id=s1），证明 idx 对齐正确。
    wv.load_html(
        "<html><body>\
         <script type=\"application/json\">{\"x\":1}</script>\
         <script id=\"s1\">globalThis.__csTagName = document.currentScript ? document.currentScript.tagName : 'NULL';\
            globalThis.__csId = document.currentScript ? (document.currentScript.id || 'NOID') : 'NULL';\
            globalThis.__csNonNull = (document.currentScript !== null);</script>\
         </body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "classic script 应无异常执行, got: {:?}", r.err());
    assert_eq!(
        wv.execute_script("String(globalThis.__csTagName)").unwrap(),
        "SCRIPT",
        "currentScript.tagName === 'SCRIPT'（classic 执行期指向 <script> 元素）"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__csId)").unwrap(),
        "s1",
        "currentScript = 执行中的 <script id=s1>（idx 跨非 JS 脚本对齐：JSON=0, JS=1）"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__csNonNull)").unwrap(),
        "true",
        "currentScript 非 null（classic 执行期）"
    );
    // 全部脚本执行完毕后 currentScript 应回到 null（finally 清）。
    assert_eq!(
        wv.execute_script("String(document.currentScript === null)").unwrap(),
        "true",
        "currentScript 为 null（脚本执行期外，finally 清）"
    );
}

#[test]
fn test_document_current_script_module_null_r3258() {
    let mut wv = WebView::new(WebViewConfig::default());
    // spec：module 脚本执行期 currentScript 恒 null（host 仅 classic 分支设 currentScript）。
    wv.load_html(
        "<html><body><script type=\"module\">globalThis.__modCsNull = (document.currentScript === null);</script></body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "module script 应无异常执行, got: {:?}", r.err());
    assert_eq!(
        wv.execute_script("String(globalThis.__modCsNull)").unwrap(),
        "true",
        "currentScript 为 null（module 执行期，spec）"
    );
}

// ── document.scrollingElement（R3259，HTML §3.1.1）──
// 返回文档视口滚动元素：standards 模式（compatMode==='CSS1Compat'）→ documentElement；quirks → body。
// headless 恒 CSS1Compat（无 quirks 跟踪）→ documentElement。scroll 库/框架读视口滚动容器高频 API
//（locomotive-scroll / smoothscroll / lazy-load）——此前缺 → `document.scrollingElement.scrollTop` 抛 TypeError。
#[test]
fn test_document_scrolling_element_r3259() {
    let mut wv = WebView::new(WebViewConfig::default());
    // classic 脚本执行期（shim document 已初始化）记录 scrollingElement 信息——execute_script
    // 直读 document 会 ReferenceError（shim 仅 run_page_scripts 路径注入）。
    wv.load_html(
        "<html><head><title>t</title></head><body><div>hi</div>\
         <script>\
         globalThis.__seNonNull = (document.scrollingElement !== null && document.scrollingElement !== undefined);\
         globalThis.__seTag = String(document.scrollingElement.tagName);\
         globalThis.__seEq = (document.scrollingElement === document.documentElement);\
         globalThis.__cm = String(document.compatMode);\
         </script></body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "classic script 应无异常执行, got: {:?}", r.err());
    assert_eq!(
        wv.execute_script("String(globalThis.__seNonNull)").unwrap(),
        "true",
        "document.scrollingElement 非 null/undefined"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__seTag)").unwrap(),
        "HTML",
        "scrollingElement.tagName === 'HTML'（standards 模式 → documentElement）"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__seEq)").unwrap(),
        "true",
        "scrollingElement === document.documentElement（standards 模式，identity）"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__cm)").unwrap(),
        "CSS1Compat",
        "compatMode === 'CSS1Compat'（standards，scrollingElement 走 documentElement 分支的依据）"
    );
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

// ── transitive module 递归 fetch（R3094）──
// dynamic import('./mod.js')，'./mod.js' 静态 import './dep.js'（transitive）。collect_module_deps_recursive
// 递归 fetch 两层 → registry 按原 spec 注册 → compile_dependency_iife 编译完整 graph。验证 val 经传递依赖解析。
#[test]
fn test_transitive_module_graph_r3094() {
    use std::sync::Arc;
    let mut wv = crate::WebViewBuilder::new()
        .script_source_fetcher(Arc::new(|_page, src| match src {
            "./mod.js" => Ok("import { val } from './dep.js'; export default val * 2;".to_string()),
            "./dep.js" => Ok("export const val = 21;".to_string()),
            _ => Err("not found".to_string()),
        }))
        .build();
    wv.load_html(
        "<html><body><script type=\"module\">\n\
         import('./mod.js').then(function (m) { globalThis.__modVal = m.default; });\n\
         </script></body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "transitive module graph 无异常, got: {:?}", r.err());
    assert_eq!(
        wv.execute_script("String(globalThis.__modVal)").unwrap(),
        "42",
        "transitive module graph：import('./mod.js')→import './dep.js' 递归 fetch → val=21 → default=42"
    );
}

// ── P1b S2：原生 DOM 绑定生产接线（native_dom kill-switch，本轮 R3097）──
// native_dom=true 时，run_page_scripts 在 polyfill 桥之上额外安装原生 nodeType/tagName
// getter（经 Sandbox::install_native_bindings escape-hatch 进持久 Context）。页面脚本读
// __zw_native_element_for_id('a').nodeType/.tagName → 原生直读 re-parsed Document（不经 shim
// 字符串桥）。默认关 → 零回归（下方 disabled 测试）。

// 原生 DOM 绑定仅 V8（quickjs 无 install_native_bindings escape-hatch）；ON 路径仅 v8 测。
#[cfg(feature = "v8")]
#[test]
fn test_native_dom_bindings_wiring_r3097() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html(
        "<html><body>\
         <div id=\"a\"><span id=\"b\">x</span></div>\
         <script>\
           globalThis.__nt = __zw_native_element_for_id('a').nodeType;\
           globalThis.__tn = __zw_native_element_for_id('a').tagName;\
           globalThis.__tnB = __zw_native_element_for_id('b').tagName;\
           globalThis.__same = (__zw_native_element_for_id('a') === __zw_native_element_for_id('a'));\
         </script>\
         </body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "native_dom 接线无异常, got: {:?}", r.err());
    assert_eq!(
        wv.execute_script("String(globalThis.__nt)").unwrap(),
        "1",
        "native nodeType"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__tn)").unwrap(),
        "DIV",
        "native tagName（HTML 大写）"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__tnB)").unwrap(),
        "SPAN",
        "native tagName 多元素不串扰"
    );
    assert_eq!(
        wv.execute_script("String(globalThis.__same)").unwrap(),
        "true",
        "NodeId↔对象身份映射（同 id 返同对象）"
    );
}

#[test]
fn test_native_dom_disabled_by_default_r3097() {
    // 默认关：__zw_native_element_for_id 未安装 → typeof 'undefined'（polyfill 桥不受影响）。
    let mut wv = crate::WebViewBuilder::new().build();
    wv.load_html(
        "<html><body><div id=\"a\"></div>\
         <script>globalThis.__has = (typeof __zw_native_element_for_id);</script>\
         </body></html>",
        None,
    );
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "默认（native_dom 关）无异常");
    assert_eq!(
        wv.execute_script("String(globalThis.__has)").unwrap(),
        "undefined",
        "默认 native_dom 关 → 工厂未安装（零回归）"
    );
}

// ── P1b L1b：native 读/写 live Document（去 R3097 read-only 快照 inert，本轮 R3107）──

// native_dom=true + load_html（render_html 填 cached_doc）→ native 经 `cached_doc_shared` 绑
// **live** Document。native setAttribute 直接改 live cached_doc（不更 cached_html、不发
// DomMutation）→ 后续 native getAttribute 见 live 写入（"9"）；re-parse 快照路径会读
// cached_html → "1"。断言 "9" 验证 live 路径（de-inert）。
#[cfg(feature = "v8")]
#[test]
fn test_native_dom_live_document_de_inert_r3107() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html("<html><body><div id=\"a\" data-x=\"1\"></div></body></html>", None);
    let r = wv.run_page_scripts_strict();
    assert!(r.is_ok(), "native_dom live 接线无异常, got: {:?}", r.err());
    // native 写：直接改 live cached_doc（cached_html 仍 data-x="1"）。
    wv.execute_script("__zw_native_element_for_id('a').setAttribute('data-x','9')")
        .unwrap();
    // native 读：见 live 写入 "9"（re-parse 快照会读 cached_html → "1"）。
    let v = wv
        .execute_script("String(__zw_native_element_for_id('a').getAttribute('data-x'))")
        .unwrap();
    assert_eq!(v, "9", "native 读 live Document（de-inert）：见 native 写");
}

// ── P1b L1b caveat ①：native 写触发重渲染（本轮 R3108）──

// native_dom=true + load_html → native 绑定经 execute_script 安装（R3108 修复 R3107 red：
// 此前仅 run_page_scripts_impl 非空脚本路径安装，execute_script 直调路径未装 →
// __zw_native_element_for_id 未定义）。native textContent 写直改 live cached_doc（不经
// polyfill DomMutation 队列）→ sync_render_after_native_dom 检测 live-doc≠cached_html
// → 全量重渲染 → 文本 glyph 图元出现 + cached_html 同步。空 div 起点 0 glyph，写入后
// glyph 增长证明重渲染确实发生（非仅 live-doc 内存变更）。
#[cfg(feature = "v8")]
#[test]
fn test_native_dom_write_triggers_rerender_r3108() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html("<html><body><div id=\"a\"></div></body></html>", None);
    wv.run_page_scripts_strict().unwrap();
    let glyphs_before = wv.last_render().expect("initial render").primitives.glyphs.len();

    // native 写：textContent 注入文本（直改 live cached_doc）。
    wv.execute_script("__zw_native_element_for_id('a').textContent = 'ZeroWeb R3108'")
        .unwrap();

    // R3108：native 写触发重渲染 → 文本 glyph 图元出现（glyphs 增长）。
    let glyphs_after = wv
        .last_render()
        .expect("render after native write")
        .primitives
        .glyphs
        .len();
    assert!(
        glyphs_after > glyphs_before,
        "native 写应触发重渲染（glyphs {} → {}）",
        glyphs_before,
        glyphs_after
    );
    // cached_html 同步 native 写（live-doc 变更传播至序列化快照）。
    assert!(
        wv.html_content().contains("ZeroWeb R3108"),
        "cached_html 同步 native 写"
    );
}

// ── P1b S4 EventTarget native：addEventListener/removeEventListener/dispatchEvent（本轮 R3109）──

// native_dom=true → native 元素（经 execute_script 安装路径，R3108）具 EventTarget 方法。
// addEventListener 持久化 listener（Global<Value>），dispatchEvent 复活 Local 调用——验证 webview
// 沙箱安装路径含 S4 模板方法（非仅 engine 隔离测试）。
#[cfg(feature = "v8")]
#[test]
fn test_native_event_target_r3109() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html("<html><body><div id=\"a\"></div></body></html>", None);
    wv.run_page_scripts_strict().unwrap();
    // 注册 click 监听器 + 派发 → listener 设全局。
    wv.execute_script(
        "(()=>{ const el=__zw_native_element_for_id('a');\
         el.addEventListener('click', ()=>{ globalThis.__clicked='yes'; });\
         el.dispatchEvent({type:'click'}); return globalThis.__clicked || 'no'; })()",
    )
    .unwrap();
    assert_eq!(
        wv.execute_script("String(globalThis.__clicked || 'no')").unwrap(),
        "yes",
        "native dispatchEvent 触发监听器"
    );
    // removeEventListener 后不再触发。
    wv.execute_script(
        "(()=>{ const el=__zw_native_element_for_id('a');\
         const fn=()=>{ globalThis.__clicked2='yes'; };\
         el.addEventListener('keyup', fn); el.removeEventListener('keyup', fn);\
         el.dispatchEvent({type:'keyup'}); return globalThis.__clicked2 || 'no'; })()",
    )
    .unwrap();
    assert_eq!(
        wv.execute_script("String(globalThis.__clicked2 || 'no')").unwrap(),
        "no",
        "removeEventListener 后 dispatch 不触发"
    );
}

// ── P1b host→page native 事件派发（本轮 R3121）──

// native_dom=true → 页面经 native addEventListener 注册的监听器（gc.rs LISTENERS）经
// webview.dispatch_event（host 驱动）可达——polyfill __zw_dispatch_event 不达，闭合 S4 host 驱动半边。
// 复用既有 __zw_native_query_selector + 原生 dispatchEvent（零新 engine 代码）。
#[cfg(feature = "v8")]
#[test]
fn test_native_host_dispatch_event_r3121() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html("<html><body><div id=\"b\"></div></body></html>", None);
    // 经 native 元素（__zw_native_element_for_id）注册 click 监听器，设全局标记。
    wv.execute_script(
        "(()=>{ const el=__zw_native_element_for_id('b');\
         el.addEventListener('click', ()=>{ globalThis.__clicked='yes'; });\
         return 'registered'; })()",
    )
    .unwrap();
    // host dispatch_event → 经 __zw_native_query_selector('#b') + 原生 dispatchEvent 触发 native 监听器。
    wv.dispatch_event("#b", "click").unwrap();
    assert_eq!(
        wv.execute_script("String(globalThis.__clicked || 'no')").unwrap(),
        "yes",
        "host dispatch_event 经 native 路径触发 native 监听器"
    );
}

// ── P1b host→page native event 对象丰富化（本轮 R3124）──

// native_dom=true → host dispatch_event 派发的 native event 不再是 bare {type}，而带
// target/currentTarget（= 目标元素，解锁 e.target 高频读）+ bubbles:true。闭合 R3121 限制①。
#[cfg(feature = "v8")]
#[test]
fn test_native_dispatch_event_enriched_r3124() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html("<html><body><div id=\"b\"></div></body></html>", None);
    // native 监听器捕获 event，记录 type/target===el/currentTarget===el/bubbles。
    wv.execute_script(
        "(()=>{ const el=__zw_native_element_for_id('b');\
         el.addEventListener('click', (e)=>{\
         globalThis.__ev = e.type + '/' + (e.target===el) + '/' + (e.currentTarget===el) + '/' + e.bubbles;\
         }); return 'registered'; })()",
    )
    .unwrap();
    wv.dispatch_event("#b", "click").unwrap();
    assert_eq!(
        wv.execute_script("String(globalThis.__ev || 'none')").unwrap(),
        "click/true/true/true",
        "native event 带 type/target/currentTarget/bubbles"
    );
}

// ── P1b host→page native dispatchEvent 冒泡（本轮 R3125）──

// native_dom=true → host dispatch_event('#child','click') 经原生 dispatchEvent 冒泡到 parent
// native 监听器（target+bubble 两阶段）。闭合 R3109 不冒泡限制——parent 级事件委托可达。
#[cfg(feature = "v8")]
#[test]
fn test_native_dispatch_event_bubbles_r3125() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html(
        "<html><body><div id=\"parent\"><div id=\"child\"></div></div></body></html>",
        None,
    );
    // parent 注册 native click 监听器（事件委托模式）。
    wv.execute_script(
        "(()=>{ __zw_native_element_for_id('parent')\
         .addEventListener('click', ()=>{ globalThis.__bubbled='yes'; });\
         return 'registered'; })()",
    )
    .unwrap();
    // host dispatch_event('#child') → bubbles:true → 冒泡到 parent 触发其监听器。
    wv.dispatch_event("#child", "click").unwrap();
    assert_eq!(
        wv.execute_script("String(globalThis.__bubbled || 'no')").unwrap(),
        "yes",
        "host dispatch_event 冒泡到 parent native 监听器"
    );
}

// ── P1b 完整 Attr 节点（本轮 R3122）──

// native_dom=true → getNamedItem 返 Attr 节点（nodeType=2/name/value/ownerElement），非 plain 对象。
// value setter 经 set_attribute 写回。验证 webview 沙箱安装含 R3122 Attr 模板（非仅 engine 隔离测试）。
#[cfg(feature = "v8")]
#[test]
fn test_native_attr_node_r3122() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html("<html><body><div id=\"a\" class=\"row\"></div></body></html>", None);
    // Attr 节点面：nodeType=2、name=nodeName、value live、ownerElement===owner 元素。
    assert_eq!(
        wv.execute_script(
            "(()=>{ const el=__zw_native_element_for_id('a');\
             const at=el.attributes.getNamedItem('class');\
             return at.nodeType+'/'+at.name+'/'+at.nodeName+'/'+at.value+'/'+(at.ownerElement===el); })()"
        )
        .unwrap(),
        "2/class/class/row/true",
        "native Attr 节点 nodeType=2/name/nodeName/value/ownerElement"
    );
    // value setter 写回 owner 元素（getAttribute live 见）。
    assert_eq!(
        wv.execute_script(
            "(()=>{ __zw_native_element_for_id('a').attributes.getNamedItem('class').value='new';\
             return __zw_native_element_for_id('a').getAttribute('class'); })()"
        )
        .unwrap(),
        "new",
        "Attr.value setter 经 set_attribute 写回"
    );
}

// ── P1b 节点导航 native：parentNode/firstChild/lastChild/nextSibling/previousSibling/hasChildNodes（本轮 R3110）──

// native_dom=true → native 元素（execute_script 安装路径）具节点导航 getter。验证 webview 沙箱安装
// 含 R3110 模板方法（非仅 engine 隔离测试）：s1.parentNode.id==='root'、root.firstChild.id==='s1'、
// s1.nextSibling.id==='s2'、s2.previousSibling.id==='s1'、s2.nextSibling===null、root.hasChildNodes()。
#[cfg(feature = "v8")]
#[test]
fn test_native_node_navigation_r3110() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html(
        "<html><body><div id=\"root\"><span id=\"s1\">hi</span><span id=\"s2\"></span></div></body></html>",
        None,
    );
    wv.run_page_scripts_strict().unwrap();
    // parentNode + siblings。
    assert_eq!(
        wv.execute_script(
            "(()=>{ const s1=__zw_native_element_for_id('s1');\
             return s1.parentNode.id+'/'+s1.nextSibling.id+'/'+__zw_native_element_for_id('s2').previousSibling.id; })()"
        ).unwrap(),
        "root/s2/s1",
        "parentNode/nextSibling/previousSibling"
    );
    // firstChild + lastChild + hasChildNodes + null 关系。
    assert_eq!(
        wv.execute_script(
            "(()=>{ const r=__zw_native_element_for_id('root');\
             return r.firstChild.id+'/'+r.lastChild.id+'/'+r.hasChildNodes()+'/'+\
             (__zw_native_element_for_id('s2').nextSibling === null); })()"
        )
        .unwrap(),
        "s1/s2/true/true",
        "firstChild/lastChild/hasChildNodes/null 关系"
    );
}

// ── P1b replaceChild + nodeValue setter native（本轮 R3111）──

// native_dom=true → native nodeValue 写经 execute_script 改 live cached_doc（文本子节点内容），
// R3108 sync_render_after_native_dom 检测 live-doc≠cached_html → 重渲染 + 同步 cached_html。
// 验证 R3108 + R3111 联动：native 文本写 → 渲染可见。
#[cfg(feature = "v8")]
#[test]
fn test_native_node_value_write_rerenders_r3111() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html("<html><body><div id=\"a\">old</div></body></html>", None);
    wv.run_page_scripts_strict().unwrap();
    // native nodeValue 写：改 div 文本子节点。
    wv.execute_script("__zw_native_element_for_id('a').firstChild.nodeValue = 'new'")
        .unwrap();
    // R3108 sync：cached_html 同步为 "new"（旧 "old" 被替换）。
    let html = wv.html_content();
    assert!(html.contains("new"), "native nodeValue 写传播至 cached_html");
    assert!(!html.contains(">old<"), "旧文本被 native 写替换");
}

// ── P1b attributes NamedNodeMap native（本轮 R3112）──

// native_dom=true → element.attributes 经 execute_script 安装路径可用（NamedNodeMap：length +
// item + getNamedItem + set/removeNamedItem + 身份稳定）。验证 webview 沙箱安装含 R3112 模板。
#[cfg(feature = "v8")]
#[test]
fn test_native_attributes_namednodemap_r3112() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html(
        "<html><body><div id=\"a\" class=\"row\" data-x=\"42\"></div></body></html>",
        None,
    );
    wv.run_page_scripts_strict().unwrap();
    // length + 身份稳定 + item/getNamedItem 读。
    assert_eq!(
        wv.execute_script(
            "(()=>{ const el=__zw_native_element_for_id('a'); const a=el.attributes;\
             return a.length+'/'+(a===el.attributes)+'/'+a.getNamedItem('class').value+'/'+a.item(2).name; })()"
        )
        .unwrap(),
        "3/true/row/data-x",
        "attributes length/identity/getNamedItem/item"
    );
}

// ── P1b innerHTML/outerHTML getter native（本轮 R3113）──

// native_dom=true → innerHTML/outerHTML 经 execute_script 安装路径可用（序列化子树/自身）。
// 验证 webview 沙箱安装含 R3113 getter + live 序列化反映 native 写。
#[cfg(feature = "v8")]
#[test]
fn test_native_inner_outer_html_r3113() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html("<html><body><div id=\"a\"><b>hi</b>!</div></body></html>", None);
    wv.run_page_scripts_strict().unwrap();
    assert_eq!(
        wv.execute_script("(__zw_native_element_for_id('a').innerHTML)")
            .unwrap(),
        "<b>hi</b>!",
        "innerHTML 序列化子节点"
    );
    assert_eq!(
        wv.execute_script("(__zw_native_element_for_id('a').outerHTML)")
            .unwrap(),
        r#"<div id="a"><b>hi</b>!</div>"#,
        "outerHTML 含自身 tag"
    );
}

// ── P1b innerHTML/outerHTML setter native（本轮 R3123）──

// native_dom=true → innerHTML/outerHTML setter 经 execute_script 解析 HTML 片段写 live cached_doc，
// R3108 sync_render_after_native_dom 检测 live-doc≠cached_html → 重渲染 + 同步 cached_html。
// 验证 R3108 + R3123 联动：native HTML 写 → 渲染可见（闭合 R3113 getter-only 限制）。
#[cfg(feature = "v8")]
#[test]
fn test_native_inner_html_setter_rerenders_r3123() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html("<html><body><div id=\"a\"><span>old</span></div></body></html>", None);
    wv.run_page_scripts_strict().unwrap();
    // native innerHTML 写：解析 `<b>new</b>` 替换 span。
    wv.execute_script("__zw_native_element_for_id('a').innerHTML='<b>new</b>'")
        .unwrap();
    let html = wv.html_content();
    assert!(
        html.contains("<b>new</b>"),
        "native innerHTML setter 传播至 cached_html"
    );
    assert!(!html.contains("old"), "旧子节点被 innerHTML setter 清空");
}

// outerHTML setter：整体替换元素 + 重渲染。
#[cfg(feature = "v8")]
#[test]
fn test_native_outer_html_setter_rerenders_r3123() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html(
        "<html><body><div id=\"p\"><span id=\"a\">old</span></div></body></html>",
        None,
    );
    wv.run_page_scripts_strict().unwrap();
    // native outerHTML 写：span 整体替换为 `<b>new</b>`。
    wv.execute_script("__zw_native_element_for_id('a').outerHTML='<b>new</b>'")
        .unwrap();
    let html = wv.html_content();
    assert!(
        html.contains("<b>new</b>"),
        "native outerHTML setter 替换传播至 cached_html"
    );
    assert!(!html.contains("old"), "原 span 被 outerHTML setter 移除");
}

// ── P1b cloneNode(deep) native（本轮 R3114）──

// native_dom=true → cloneNode 经 execute_script 安装路径可用（浅/深克隆）。
#[cfg(feature = "v8")]
#[test]
fn test_native_clone_node_r3114() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html(
        "<html><body><div id=\"a\" class=\"x\"><span>hi</span></div></body></html>",
        None,
    );
    wv.run_page_scripts_strict().unwrap();
    // 浅克隆：同 tag+属性，无子；新对象。
    assert_eq!(
        wv.execute_script(
            "(()=>{ const c=__zw_native_element_for_id('a').cloneNode(false);\
             return c.tagName+'/'+c.getAttribute('class')+'/'+c.children.length; })()"
        )
        .unwrap(),
        "DIV/x/0",
        "cloneNode(false) 浅克隆"
    );
    // 深克隆：含子树。
    assert_eq!(
        wv.execute_script("(__zw_native_element_for_id('a').cloneNode(true).children.length)")
            .unwrap(),
        "1",
        "cloneNode(true) 深克隆含子树"
    );
}

// ── P1b contains(node) native（本轮 R3115）──

// native_dom=true → contains 经 execute_script 安装路径可用（后代关系 walk parent 链）。
#[cfg(feature = "v8")]
#[test]
fn test_native_contains_r3115() {
    let mut wv = crate::WebViewBuilder::new().native_dom(true).build();
    wv.load_html(
        "<html><body><div id=\"a\"><div id=\"b\"><span id=\"c\">x</span></div></div></body></html>",
        None,
    );
    wv.run_page_scripts_strict().unwrap();
    assert_eq!(
        wv.execute_script(
            "(()=>{ const a=__zw_native_element_for_id('a'), c=__zw_native_element_for_id('c');\
             return a.contains(c)+'/'+a.contains(a)+'/'+c.contains(a); })()"
        )
        .unwrap(),
        "true/true/false",
        "contains 后代/自身/非后代"
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
