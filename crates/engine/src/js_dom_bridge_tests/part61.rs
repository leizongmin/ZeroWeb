// part61 — workers-compat M2 切片：importScripts 三相语义与 worker 全局最小面；
// M3 切片补充：postMessage transfer/ports + structured-clone 语义（DC-3）。
// 本文件经 js_dom_bridge_tests.rs 的 include! 并入同一模块（part31 先例——
// part21 4071 行超 2000 行上限后新工作落新文件）。契约：docs/goal/workers-compat.md
// DC-2/DC-3；WPT 案面 workers/interfaces/WorkerUtils/importScripts/{002-012}、
// WorkerNavigator.any.js、semantics/interface-objects/001.worker.js、
// semantics/structured-clone/dedicated.html battery、Worker-multi-port。
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

/// worker 未捕获错误管线（report the exception）：onmessage handler 抛与
/// setTimeout 回调抛均异步上报到 page 侧 Worker 实例，message 格式化
/// `Uncaught <name>: <message>`（message-helper 断言面）。
///
/// 顶层抛同管线（throw.js toplevel 位置——worker 全局 onerror 已先安装、返回
/// false 不阻断 page 侧上报）。
#[test]
fn test_worker_toplevel_throw_reports_to_page() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/onerr/top.html", |_page, src| {
        if src == "top-worker.js" {
            r#"onerror = function () { return false; };
throw new TypeError('Throw in toplevel');"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__msgs = [];\
             var worker = new Worker('top-worker.js');\
             worker.onerror = function (event) { globalThis.__msgs.push(String(event.message)); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__msgs)").unwrap().value,
        r#"["Uncaught TypeError: Throw in toplevel"]"#,
        "顶层抛经管线上报 page 侧（worker 内 onerror 返 false 不阻断）"
    );
}
#[test]
fn test_worker_uncaught_error_report_pipeline_formats_message() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/onerr/pipe.html", |_page, src| {
        if src == "pipe-worker.js" {
            r#"onmessage = function () { throw new Error('Throw in message handler'); };
setTimeout(function () { throw new TypeError('Throw in timer'); }, 0);"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__msgs = [];\
             var worker = new Worker('pipe-worker.js');\
             worker.onerror = function (event) { globalThis.__msgs.push(String(event.message)); };\
             worker.postMessage('x');",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 8);
    // 序无关断言：message 派发（微任务泵驱动）与宿主定时器回调的相对顺序不作约定。
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__msgs.slice().sort())")
            .unwrap()
            .value,
        r#"["Uncaught Error: Throw in message handler","Uncaught TypeError: Throw in timer"]"#,
        "onmessage/timer 抛均上报且 message 含原名与原消息"
    );
}

/// worker 全局 onerror handler 再抛 → 原错误与再抛错误各上报一次（throw.js
/// onerror 位置第二错语义），内部再抛不回环 worker 全局。
#[test]
fn test_worker_internal_onerror_rethrow_reports_both_errors() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/onerr/two.html", |_page, src| {
        if src == "two-worker.js" {
            r#"onerror = function () { throw new Error('Throw in error handler'); };
onmessage = function () { throw new Error('Throw in message handler'); };"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__msgs = [];\
             var worker = new Worker('two-worker.js');\
             worker.onerror = function (event) { globalThis.__msgs.push(String(event.message)); };\
             worker.postMessage('x');",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 8);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__msgs)").unwrap().value,
        r#"["Uncaught Error: Throw in message handler","Uncaught Error: Throw in error handler"]"#,
        "原错误与 onerror 再抛各上报一次、顺序保持"
    );
}


/// handled.html 面：worker 全局 onerror（legacy 5 参签名）返 true → handled，
/// 不再向 page 侧上报（ErrorEvent 不派发、__zw_report_error 不触）。
#[test]
fn test_worker_internal_onerror_return_true_suppresses_page_error() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/onerr/handled.html", |_page, src| {
        if src == "handled-worker.js" {
            r#"onerror = function (message, filename, lineno, colno, error) {
  postMessage([typeof message, typeof filename, typeof lineno, typeof colno, error instanceof Error]);
  return true;
};
throw new Error('swallowed');"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__msgs = [];\
             globalThis.__pageErr = null;\
             var worker = new Worker('handled-worker.js');\
             worker.onerror = function () { globalThis.__pageErr = 'fired'; };\
             worker.onmessage = function (e) { globalThis.__msgs.push(e.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__pageErr").unwrap().value,
        "null",
        "onerror 返 true 应抑制 page 侧上报（handled）"
    );
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__msgs)").unwrap().value,
        r#"[["string","string","number","number",true]]"#,
        "legacy onerror 应按 5 参签名调用（message/filename/lineno/colno/error）"
    );
}

/// dedicated-worker-parse-error-failure 面：脚本语法错（编译期抛）→ Worker 对象
/// error 事件为普通 Event——无 message 成员、非 ErrorEvent（worker-processing-model
/// parse/fetch 分型；fetch 失败为 ErrorEvent，既有测试另案）。
#[test]
fn test_worker_parse_error_dispatches_plain_error_event() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/parse/bad.html", |_page, src| {
        if src == "bad-worker.js" {
            "this is not valid javascript (".to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__pe = null;\
             var worker = new Worker('bad-worker.js');\
             worker.onerror = function (ev) { globalThis.__pe = [ev instanceof ErrorEvent, ev.message === undefined]; };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__pe)").unwrap().value,
        r#"[false,true]"#,
        "解析失败应派发普通 Event（非 ErrorEvent、无 message）"
    );
}

/// 顶层 function 声明暴露到 worker 全局（Annex B 函数落 Function 作用域的补偿）——
/// 字符串定时器按全局作用域编译（with(self)）须能解析到（throw.js setTimeout-string 面）。
#[test]
fn test_worker_toplevel_function_reachable_from_string_timer() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/timer/str.html", |_page, src| {
        if src == "str-worker.js" {
            r#"function workerFn() { postMessage('from-string-timer'); }
setTimeout('workerFn()', 10);"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__msgs = [];\
             var worker = new Worker('str-worker.js');\
             worker.onmessage = function (e) { globalThis.__msgs.push(e.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 8);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__msgs)").unwrap().value,
        r#"["from-string-timer"]"#,
        "字符串定时器应解析到顶层 function 声明（暴露补偿生效）"
    );
}

/// run-a-worker/001 面：worker 顶层 this === self（new Function 体以 wctx 为 this 调用）。
#[test]
fn test_worker_toplevel_this_is_self() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/this/top.html", |_page, src| {
        if src == "this-worker.js" {
            "postMessage(this === self);".to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = null;\
             var worker = new Worker('this-worker.js');\
             worker.onmessage = function (e) { globalThis.__r = e.data; };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "true",
        "worker 顶层 this 应等于 self"
    );
}

/// self 只读（getter-only defineProperty）——sloppy 模式赋值静默忽略、self 不变。
#[test]
fn test_worker_self_assignment_silently_ignored() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/self/ro.html", |_page, src| {
        if src == "ro-worker.js" {
            "var result;\ntry { self = 1; result = 'no-throw:' + (typeof self); } catch (e) { result = 'throw:' + e.name; }\npostMessage(result);"
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = null;\
             var worker = new Worker('ro-worker.js');\
             worker.onmessage = function (e) { globalThis.__r = e.data; };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "no-throw:object",
        "sloppy 模式对 getter-only self 赋值应静默忽略且 self 不变"
    );
}

/// WorkerGlobalScope-close 语义：close() 不拦 worker→page 出站投递（同任务
/// close 前后 postMessage 均达——「Should be delivered」面）；close 拦的是入站
/// 投递与新任务（页侧无二次 pong，见 test_worker_close_page_flow）。
#[test]
fn test_worker_close_postmessage_still_delivers() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/close/late.html", |_page, src| {
        if src == "late-worker.js" {
            r#"onmessage = function () {
  postMessage('first');
  close();
  postMessage('second');
};"#
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__msgs = [];\
             var worker = new Worker('late-worker.js');\
             worker.onmessage = function (e) { globalThis.__msgs.push(e.data); };\
             worker.postMessage('go');",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 8);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__msgs)").unwrap().value,
        r#"["first","second"]"#,
        "close() 前后 worker→page 投递均达（出站口不被 closing 关闭）"
    );
}

/// postMessage_DataCloneErr 面：worker 内平台对象（WorkerNavigator 实例）不可
/// 结构化克隆 → DataCloneError（prototype 注册表拒绝点）。
#[test]
fn test_worker_structured_clone_rejects_platform_objects() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/clone/platform.html", |_page, src| {
        if src == "platform-worker.js" {
            "try { postMessage(navigator); postMessage('no-throw'); } catch (e) { postMessage('clone-err:' + e.name); }"
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__msgs = [];\
             var worker = new Worker('platform-worker.js');\
             worker.onmessage = function (e) { globalThis.__msgs.push(e.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__msgs)").unwrap().value,
        r#"["clone-err:DataCloneError"]"#,
        "postMessage(navigator) 应抛 DataCloneError 且首条消息不入队"
    );
}

/// worker performance 最小面：now/mark/measure 可用而 timing/navigation 不可见
///（page performance 经 with(self) 回退泄漏的修正——worker-performance.worker.js 面）。
#[test]
fn test_worker_performance_faces() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/perf/faces.html", |_page, src| {
        if src == "perf-worker.js" {
            "postMessage([typeof performance.timing, typeof performance.navigation, typeof performance.now(), typeof performance.mark]);"
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = null;\
             var worker = new Worker('perf-worker.js');\
             worker.onmessage = function (e) { globalThis.__r = JSON.stringify(e.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        r#"["undefined","undefined","number","function"]"#,
        "worker performance 应有 now/mark 而无 timing/navigation"
    );
}

/// MessageEvent WebIDL 面：data 缺省 null（'data' in options 判定）、
/// Symbol.toStringTag → '[object MessageEvent]'（assert_class_string 面）。
#[test]
fn test_worker_messageevent_faces() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/ev/faces.html", |_page, src| {
        if src == "ev-worker.js" {
            "var ev = new MessageEvent('message', {});\npostMessage([Object.prototype.toString.call(ev), ev.data === null]);"
                .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = null;\
             var worker = new Worker('ev-worker.js');\
             worker.onmessage = function (e) { globalThis.__r = JSON.stringify(e.data); };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        r#"["[object MessageEvent]",true]"#,
        "MessageEvent 缺省 data 为 null 且 toStringTag 为 MessageEvent"
    );
}

/// Worker-location.sub 面：String(WorkerLocation) 为 native 形态（own toString 掩码）。
#[test]
fn test_worker_location_string_mask() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/loc/mask.html", |_page, src| {
        if src == "mask-worker.js" {
            "postMessage(String(WorkerLocation));".to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__r = null;\
             var worker = new Worker('mask-worker.js');\
             worker.onmessage = function (e) { globalThis.__r = e.data; };",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "function WorkerLocation() { [native code] }",
        "String(WorkerLocation) 应为 native 掩码形态"
    );
}

/// WorkerGlobalScope-close 页面流复刻：close 后 worker→page postMessage 仍达
///（「Should be delivered」面——closing 只拦新任务不入 worker 事件循环，不拦
/// worker→page 投递），page 在 close 入队后的 ping 不再被 worker 处理（无二次 pong）。
#[test]
fn test_worker_close_page_flow() {
    use zero_script_sandbox::Sandbox;
    let mut sandbox = worker_m2_sandbox("https://wpt.test/close/flow.html", |_page, src| {
        if src == "flow-worker.js" {
            r#"onmessage = function (evt) {
  if (evt.data == "close") {
    close();
    postMessage("Should be delivered");
  } else if (evt.data == "ping") {
    postMessage("pong");
  }
};"#
            .to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__msgs = [];\
             var worker = new Worker('flow-worker.js');\
             worker.onmessage = function (evt) {\
               globalThis.__msgs.push(evt.data);\
               if (evt.data === 'pong') {\
                 worker.postMessage('close');\
                 worker.postMessage('ping');\
               }\
             };\
             worker.postMessage('ping');",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 10);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__msgs)").unwrap().value,
        r#"["pong","Should be delivered"]"#,
        "close 后 worker→page 投递仍达；close 入队后的 ping 不被处理（无二次 pong）"
    );
}

/// M3（DC-3）：结构化克隆包装对象面——Boolean/Number/String/BigInt 包装实例克隆保持
/// 类型（spec serialize Boolean/Number/String/BigInt object 情形；battery
/// compare_Boolean/compare_obj「instanceof」面）+ 稀疏数组洞保持（spec 逐 index 判
/// item present；battery「Array sparse」length 面）。
#[test]
fn test_m3_structured_clone_wrappers_and_sparse_array() {
    let mut sandbox = worker_m2_sandbox("https://wpt.test/m3/clone-wrap.html", |_page, _src| String::new());
    sandbox
        .execute(
            "globalThis.__r = [];\
             __r.push(structuredClone(Object(true)) instanceof Boolean);\
             __r.push(structuredClone(Object(42)) instanceof Number);\
             __r.push(structuredClone(Object('x')) instanceof String);\
             __r.push(structuredClone(Object(10n)) instanceof BigInt);\
             var sp = structuredClone(new Array(10));\
             __r.push(sp.length === 10 && !(0 in sp) && !(9 in sp));\
             var nested = structuredClone([{}, [1, 2]]);\
             __r.push(Array.isArray(nested) && nested[1][1] === 2 && typeof nested[0] === 'object');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__r)").unwrap().value,
        r#"[true,true,true,true,true,true]"#,
        "包装对象克隆保持类型 + 稀疏数组 length/洞保持"
    );
}

/// M3（DC-3）：Error 克隆三相——constructor/name/message 保持、cause 递归克隆、
/// 非 own message 不安装、自定义 own 属性不迁（battery compare_Error 五面）。
#[test]
fn test_m3_structured_clone_error_face() {
    let mut sandbox = worker_m2_sandbox("https://wpt.test/m3/clone-err.html", |_page, _src| String::new());
    sandbox
        .execute(
            "globalThis.__r = [];\
             var e = new TypeError('m1'); e.cause = new RangeError('c');\
             var c = structuredClone(e);\
             __r.push(c instanceof TypeError && c.message === 'm1' && c.name === 'TypeError');\
             __r.push(c.cause instanceof RangeError && c.cause.message === 'c');\
             var c2 = structuredClone(new Error());\
             __r.push(!c2.hasOwnProperty('message'));\
             var e3 = new TypeError('x'); e3.foo = 1;\
             __r.push(structuredClone(e3).foo === undefined);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__r)").unwrap().value,
        r#"[true,true,true,true]"#,
        "Error 克隆保持构造器/message/cause，自定义属性不迁"
    );
}

/// M3（DC-3）：视图克隆面——DataView 经 fresh buffer 三元组重建（buffer 非同一）；
/// resizable buffer 身份保真（maxByteLength/resizable 随迁）；OOB DataView 序列化抛
/// DataCloneError；SharedArrayBuffer 非 COI 拒绝克隆（battery catch 面）。
#[test]
fn test_m3_structured_clone_view_faces() {
    let mut sandbox = worker_m2_sandbox("https://wpt.test/m3/clone-view.html", |_page, _src| String::new());
    sandbox
        .execute(
            "globalThis.__r = [];\
             var dv = new DataView(new ArrayBuffer(8)); dv.setUint8(0, 7);\
             var dc = structuredClone(dv);\
             __r.push(dc instanceof DataView && dc.buffer !== dv.buffer && dc.getUint8(0) === 7);\
             var rab = new ArrayBuffer(16, { maxByteLength: 1024 });\
             var tc = structuredClone(new Uint8Array(rab));\
             __r.push(tc.buffer.maxByteLength === 1024 && tc.buffer.resizable === true);\
             var f = new Uint8Array([1, 2, 3]);\
             var fc = structuredClone(f);\
             __r.push(fc instanceof Uint8Array && fc[0] === 1 && fc.buffer !== f.buffer);\
             var ab2 = new ArrayBuffer(16, { maxByteLength: 1024 });\
             var dv2 = new DataView(ab2, 8); ab2.resize(0);\
             try { structuredClone(dv2); __r.push('oob-no-throw'); } catch (e) { __r.push(e.name === 'DataCloneError' && e.code === 25); }\
             try { structuredClone(new SharedArrayBuffer(8)); __r.push('sab-no-throw'); } catch (e) { __r.push(e.name === 'DataCloneError'); }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__r)").unwrap().value,
        r#"[true,true,true,true,true]"#,
        "DataView 重建 + resizable 保真 + OOB/SAB DataCloneError 面"
    );
}

/// M3（DC-3）：FileList 克隆保持类型 + Blob 在 global 构造器被 delete 后仍克隆为
/// Blob 实例（spec 反序列化走 realm 注册表，delete 不触及——battery「interface
/// deleted from the global must still deserialize」面）。
#[test]
fn test_m3_structured_clone_filelist_and_deleted_blob_global() {
    let mut sandbox = worker_m2_sandbox("https://wpt.test/m3/clone-del.html", |_page, _src| String::new());
    sandbox
        .execute(
            "globalThis.__r = [];\
             var fl = new FileList();\
             var fc = structuredClone(fl);\
             __r.push(fc instanceof FileList && fc !== fl && fc.length === 0);\
             var b = new Blob(['x']);\
             var BC = globalThis.Blob;\
             delete globalThis.Blob;\
             var cb = structuredClone(b);\
             __r.push(cb instanceof BC && cb.size === 1);\
             globalThis.Blob = BC;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__r)").unwrap().value,
        r#"[true,true]"#,
        "FileList instanceof 保持 + delete 后 Blob 仍克隆为 Blob"
    );
}

/// M3（DC-3）：ImageBitmap 全局接口面——非法构造 TypeError、wire 几何解析、克隆保持
/// 类型与内容、toStringTag、close() 释放、transfer 副作用（源 close）。
#[test]
fn test_m3_imagebitmap_faces() {
    let mut sandbox = worker_m2_sandbox("https://wpt.test/m3/bitmap.html", |_page, _src| String::new());
    sandbox
        .execute(
            "globalThis.__r = [];\
             try { new ImageBitmap(); __r.push('ctor-no-throw'); } catch (e) { __r.push(e instanceof TypeError); }\
             var bm = new ImageBitmap('2:1;,1,2,3,4,5,6,7,8');\
             __r.push(bm instanceof ImageBitmap && bm.width === 2 && bm.height === 1);\
             var bc = structuredClone(bm);\
             __r.push(bc instanceof ImageBitmap && bc !== bm && bc.width === 2 && bc._zwBitmapWire === bm._zwBitmapWire);\
             __r.push(Object.prototype.toString.call(bc) === '[object ImageBitmap]');\
             bm.close();\
             __r.push(bm._closed === true && bm.width === 0);\
             var bm2 = new ImageBitmap('1:1;,1,2,3,4');\
             structuredClone(bm2, { transfer: [bm2] });\
             __r.push(bm2._closed === true);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__r)").unwrap().value,
        r#"[true,true,true,true,true,true]"#,
        "ImageBitmap 接口/克隆/close/transfer 面"
    );
}

/// M3（DC-3）：detached buffer transfer 面——首转移后源 byteLength 0（真 detach）+
/// 再转移抛 DataCloneError（battery「A detached ArrayBuffer cannot be transferred」）；
/// OffscreenCanvas 克隆保持类型与尺寸（battery transfer 面同参）。
#[test]
fn test_m3_transfer_detached_buffer_and_offscreen_canvas() {
    let mut sandbox = worker_m2_sandbox("https://wpt.test/m3/detach.html", |_page, _src| String::new());
    sandbox
        .execute(
            "globalThis.__r = [];\
             var ab = new ArrayBuffer(8);\
             structuredClone(ab, { transfer: [ab] });\
             __r.push(ab.byteLength === 0);\
             try { structuredClone(ab, { transfer: [ab] }); __r.push('re-no-throw'); } catch (e) { __r.push(e.name === 'DataCloneError' && e.code === 25); }\
             var oc = new OffscreenCanvas(10, 10);\
             var occ = structuredClone(oc);\
             __r.push(occ instanceof OffscreenCanvas && occ.width === 10 && occ.height === 10 && occ !== oc);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__r)").unwrap().value,
        r#"[true,true,true]"#,
        "detached buffer 再转移拒绝 + OffscreenCanvas 克隆尺寸面"
    );
}

/// M3（DC-3）：MessagePort 转移错误面——closed/source port、payload port 无列表、
/// 不可转移项、null 元素（TypeError）各形态（spec serialize-with-transfer 校验序 +
/// Worker-multi-port「failed postMessage 后重发仍可用」面）。
#[test]
fn test_m3_port_transfer_errors() {
    let mut sandbox = worker_m2_sandbox("https://wpt.test/m3/port-err.html", |_page, _src| String::new());
    sandbox
        .execute(
            "globalThis.__r = [];\
             var ch = new MessageChannel();\
             var ch2 = new MessageChannel(); ch2.port2.close();\
             try { ch.port1.postMessage(1, [ch2.port2]); __r.push('closed-no-throw'); } catch (e) { __r.push(e.name === 'DataCloneError'); }\
             try { ch.port1.postMessage(ch.port2); __r.push('payload-no-throw'); } catch (e) { __r.push(e.name === 'DataCloneError'); }\
             try { ch.port1.postMessage(1, [{}]); __r.push('nt-no-throw'); } catch (e) { __r.push(e.name === 'DataCloneError'); }\
             try { ch.port1.postMessage(1, [null]); __r.push('null-no-throw'); } catch (e) { __r.push(e instanceof TypeError); }\
             try { ch.port1.postMessage(1, [ch.port1]); __r.push('self-no-throw'); } catch (e) { __r.push(e.name === 'DataCloneError'); }\
             ch.port1.postMessage('still-ok');\
             __r.push(true);",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 3);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__r)").unwrap().value,
        r#"[true,true,true,true,true,true]"#,
        "port 转移五错误形态 + 失败 postMessage 后源 port 仍可用"
    );
}

/// M3（DC-3）：payload port + transfer 列表同一转移体（battery transfer「MessagePort」
/// 面）——page→worker 转移 port2，worker echo 回 page，转移体与 port1 通信可达。
#[test]
fn test_m3_port_transfer_same_clone_reaches_peer() {
    let mut sandbox = worker_m2_sandbox("https://wpt.test/m3/port-flow.html", |_page, src| {
        if src == "echo-worker.js" {
            "onmessage = ev => postMessage(ev.data, ev.data.transfer);".to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__msgs = [];\
             var ch = new MessageChannel();\
             globalThis.__ch = ch;\
             var worker = new Worker('echo-worker.js');\
             worker.onmessage = function (ev) {\
               globalThis.__port = ev.data.data;\
               globalThis.__msgs.push('echo');\
             };\
             worker.postMessage({ data: ch.port2, transfer: [ch.port2] }, [ch.port2]);",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    sandbox
        .execute(
            "globalThis.__ch.port1.onmessage = function (ev) { globalThis.__msgs.push(ev.data); };\
             globalThis.__port.postMessage('ohai');",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__msgs)").unwrap().value,
        r#"["echo","ohai"]"#,
        "转移体克隆与原配对 port 通信可达（serialize-with-transfer 同一转移体语义）"
    );
}

/// M3（DC-3）：postMessage 第二参联合形态——null → 字典分支缺省（second-argument-null
/// 面）、options 字典 {transfer} 形态、[null] 元素 TypeError。
#[test]
fn test_m3_postmessage_union_second_arg() {
    let mut sandbox = worker_m2_sandbox("https://wpt.test/m3/union.html", |_page, src| {
        if src == "echo-worker.js" {
            "onmessage = ev => postMessage(ev.data, ev.data.transfer);".to_string()
        } else {
            String::new()
        }
    });
    sandbox
        .execute(
            "globalThis.__m = [];\
             var w = new Worker('echo-worker.js');\
             w.onmessage = function (ev) { globalThis.__m.push(ev.data); };\
             w.postMessage(1, null);\
             w.postMessage(2, { transfer: [] });\
             w.postMessage(3, undefined);\
             try { w.postMessage(4, [null]); } catch (e) { globalThis.__m.push(e instanceof TypeError); }",
        )
        .unwrap();
    worker_m2_pump(&mut sandbox, 6);
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__m)").unwrap().value,
        r#"[true,1,2,3]"#,
        "[null] 元素 TypeError 同步抛（先入）；null/字典/undefined 三形态经 task 送达"
    );
}

