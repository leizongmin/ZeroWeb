// part61 — workers-compat M2 切片：importScripts 三相语义与 worker 全局最小面。
// 本文件经 js_dom_bridge_tests.rs 的 include! 并入同一模块（part31 先例——
// part21 4071 行超 2000 行上限后新工作落新文件）。契约：docs/goal/workers-compat.md
// DC-2；WPT 案面 workers/interfaces/WorkerUtils/importScripts/{002-012}、
// WorkerNavigator.any.js、semantics/interface-objects/001.worker.js。
// 沙盒模式沿用 part21 嵌套 worker 先例：persistent_context + __zw_fetch_script
// 注册 + microtask 泵（重复 execute 触发 postMessage 投递）。

/// workers-compat M2 公共沙盒搭建：shim 装载 + 脚本 fetch 回调 + DOM 回调注册。
fn worker_m2_sandbox<F>(page_url: &str, fetch: F) -> zero_script_sandbox::V8Sandbox
where
    F: Fn(&str, &str) -> String + Send + Sync + 'static,
{
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.register_callback(
        "__zw_fetch_script",
        Box::new(move |args| {
            let page = args.first().map(String::as_str).unwrap_or("");
            let src = args.get(1).map(String::as_str).unwrap_or("");
            fetch(page, src)
        }),
    );
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page: Arc<Mutex<String>> = Arc::new(Mutex::new(page_url.to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page, &canvas_registry, None);
    sandbox
}

/// postMessage 经 queueMicrotask 投递——重复 execute 触发微task 泵（part21 先例）。
fn worker_m2_pump(sandbox: &mut zero_script_sandbox::V8Sandbox, rounds: usize) {
    use zero_script_sandbox::Sandbox;
    for i in 0..rounds {
        sandbox
            .execute(&format!("globalThis.__workerM2Pump = {i};"))
            .unwrap();
    }
}

/// 003 案：data: 脚本顺序执行且顶层定义/赋值与 worker 脚本共享作用域
///（direct eval 同 var-env）——块内首语句（try { 后）同样走内联路径。
#[test]
fn test_worker_importscripts_data_urls_share_scope_and_run_sequentially() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/importScripts/003.html", |page, src| {
        assert_eq!(page, "https://wpt.test/importScripts/003.html");
        if src == "003-worker.js" {
            r#"var x = 'a';
try {
  importScripts('data:text/javascript,x+="b"',
                'data:text/javascript,x+="c"');
} catch(e) {
  x += "d"
}
postMessage(x);"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             var worker = new Worker('003-worker.js');\
             worker.onmessage = function (event) { globalThis.__r = event.data; };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "abc",
        "importScripts 应顺序执行且与 worker 脚本共享作用域（catch 未触发）"
    );
}

/// 004 案：实参表中段脚本语法错（含行注释的实参表）——语法错在执行位抛出、
/// 首脚本已执行的效果保留、后续脚本不执行；异常被 instanceof SyntaxError 命中。
#[test]
fn test_worker_importscripts_syntax_error_mid_list_aborts_remaining() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/importScripts/004.html", |_page, src| {
        if src == "004-worker.js" {
            r#"var x = '';
var exception;
try {
  importScripts('data:text/javascript,x+="first script successful. "',
                'data:text/javascript,x+="FAIL (second script). "; for(;) break;', // doesn't compile
                'data:text/javascript,x+="FAIL (third script)"');
} catch(ex) {
  if (ex instanceof SyntaxError)
    exception = true;
  else
    exception = String(ex);
}
postMessage([x, exception]);"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             var worker = new Worker('004-worker.js');\
             worker.onmessage = function (event) { globalThis.__r = JSON.stringify(event.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        r#"["first script successful. ",true]"#,
        "语法错中止后续脚本且保留首脚本效果（实参表内注释不破坏解析）"
    );
}

/// 002 案：任一参数 URL 解析失败 → SyntaxError DOMException（原生构造器——
/// R384 wrong-global），且任何脚本都不执行（相 ① 先于全部执行）。
#[test]
fn test_worker_importscripts_invalid_url_throws_native_syntaxerror_and_runs_nothing() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/importScripts/002.html", |_page, src| {
        if src == "002-worker.js" {
            r#"var ran = false;
var name = '';
var nativeCtor = false;
try {
  importScripts('data:text/javascript,ran=true', 'http://foo bar');
} catch (e) {
  name = e.name;
  nativeCtor = e.constructor === DOMException;
}
postMessage([ran, name, nativeCtor]);"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             var worker = new Worker('002-worker.js');\
             worker.onmessage = function (event) { globalThis.__r = JSON.stringify(event.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        r#"[false,"SyntaxError",true]"#,
        "URL 解析失败抛 SyntaxError DOMException 且不执行任何脚本"
    );
}

/// fetch 相失败（同源脚本取不到）→ NetworkError DOMException，且任何脚本都不
/// 执行（相 ② 先于全部执行——blob-url「前脚本 revoke」案钉住的全量先取语义）。
#[test]
fn test_worker_importscripts_fetch_failure_throws_networkerror_and_runs_nothing() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/importScripts/net.html", |_page, src| {
        if src == "net-worker.js" {
            r#"var ran = false;
var name = '';
var nativeCtor = false;
try {
  importScripts('data:text/javascript,ran=true', 'missing-script.js');
} catch (e) {
  name = e.name;
  nativeCtor = e.constructor === DOMException;
}
postMessage([ran, name, nativeCtor]);"#
                .to_string()
        } else {
            String::new() // missing-script.js 取不到 → NetworkError
        }
    });
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             var worker = new Worker('net-worker.js');\
             worker.onmessage = function (event) { globalThis.__r = JSON.stringify(event.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        r#"[false,"NetworkError",true]"#,
        "fetch 失败抛 NetworkError 且不执行任何脚本"
    );
}

/// 006 案：执行相运行时 throw 在执行位透传——首脚本效果保留（x=1）、
/// 异常被 worker try/catch 命中（y=2）、后续脚本不执行（z undefined）。
#[test]
fn test_worker_importscripts_runtime_throw_aborts_remaining_scripts() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/importScripts/006.html", |_page, src| {
        if src == "006-worker.js" {
            r#"var x;
var y;
var z;
try {
  importScripts('data:text/javascript,x=1',
                'data:text/javascript,throw 2',
                'data:text/javascript,z=3');
} catch(e) {
  y = e;
}
postMessage([x, y, z]);"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             var worker = new Worker('006-worker.js');\
             worker.onmessage = function (event) { globalThis.__r = JSON.stringify(event.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "[1,2,null]",
        "运行时 throw 中止后续脚本、保留已完成脚本效果"
    );
}

/// 010 案：常量实参（undefined）按 String() 归一解析——运行时 importScripts
/// 与字面量路径同语义，`undefined` 作为相对 URL 取脚本并执行。
#[test]
fn test_worker_importscripts_constant_arg_undefined_resolves_and_runs() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/importScripts/010.html", |_page, src| {
        if src == "undefined" {
            "var got = 'undefined';".to_string()
        } else if src == "010-worker.js" {
            r#"var got = 'unset';
try {
  importScripts(undefined);
} catch (ex) {
  got = 'caught ' + ex;
}
postMessage(got);"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             var worker = new Worker('010-worker.js');\
             worker.onmessage = function (event) { globalThis.__r = event.data; };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "undefined",
        "常量实参 undefined 按 String() 归一后正常解析取脚本"
    );
}

/// worker 全局最小面（DC-2）：WorkerNavigator 只读属性（赋值 TypeError）、
/// instanceof WorkerNavigator/WorkerLocation 命中、接口对象为 self own property
///（interface-objects 001 案断言面）。
#[test]
fn test_worker_global_navigator_location_and_interface_own_properties() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/interfaces/globals.html", |_page, src| {
        if src == "globals-worker.js" {
            r#"var ro = false;
try { navigator.userAgent = 'x'; } catch (e) { ro = e instanceof TypeError; }
postMessage({
  nav: typeof navigator,
  navInstance: navigator instanceof WorkerNavigator,
  appName: typeof navigator.appName === 'string' && navigator.appName.length > 0,
  ro: ro,
  loc: location instanceof WorkerLocation,
  locHref: typeof location.href === 'string' && location.href.indexOf('https://wpt.test/') === 0,
  ownDomException: Object.getOwnPropertyNames(self).indexOf('DOMException') !== -1,
  ownFormData: Object.getOwnPropertyNames(self).indexOf('FormData') !== -1,
  ownWorkerNavigator: Object.getOwnPropertyNames(self).indexOf('WorkerNavigator') !== -1,
  ownWorkerLocation: Object.getOwnPropertyNames(self).indexOf('WorkerLocation') !== -1
});"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = 'pending';\
             var worker = new Worker('globals-worker.js');\
             worker.onmessage = function (event) { globalThis.__r = JSON.stringify(event.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    // JSON.stringify 键序按源码插入序确定——整串比对（引擎测试无 serde_json 依赖）。
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        r#"{"nav":"object","navInstance":true,"appName":true,"ro":true,"loc":true,"locHref":true,"ownDomException":true,"ownFormData":true,"ownWorkerNavigator":true,"ownWorkerLocation":true}"#,
        "worker 全局最小面：navigator/location 只读、接口 own property（interface-objects 001 面）"
    );
}

/// worker 脚本 fetch 失败 → 异步 error 事件派发到 Worker 实例（spec
/// worker-processing-model；host 未注册 fetcher 时的静默语义不受影响）。
#[test]
fn test_worker_script_fetch_failure_dispatches_error_event() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/fail/error.html", |_page, _src| String::new());
    sandbox
        .execute(
            "globalThis.__err = null;\
             var worker = new Worker('missing-worker.js');\
             worker.onerror = function (event) { globalThis.__err = String(event.message); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    let msg = sandbox.execute("globalThis.__err").unwrap().value;
    assert!(
        msg.contains("Failed to fetch worker script"),
        "fetch 失败应派发 error 事件且 message 指明失败原因，实际: {msg}"
    );
}
