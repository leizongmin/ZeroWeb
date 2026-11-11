// part103 — html-semantics M2 切片 403（2026-11-11）：object 嵌套文档脚本通道
// （R206 移植）+ parser 静态启航 + 嵌套导航 + named access navigable 值。
//
// object-handler.html 文件级 Timeout 的三面根因修复钉：
// - parser 静态 object 启航（`__zwStartParserObjects`——**静默**物化：不派
//   load/error，display:none 双案「不渲染不派 load」面规避回归）；
// - 嵌套文档脚本执行（`_zwObjectMaterializeDoc` 接 `_zwRunIframeScripts`——
//   parent 回调链驱动案）；
// - 嵌套导航（`document.location.href` 写侧 → `_zwObjectNavigateEntry`：fetch +
//   重物化 + 脚本重放，data 属性不动）+ named access morph（`window[name]` →
//   contentWindow）+ 嵌套 `history.replaceState`（doc._zwURL 槽，不 fetch）。
// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element

/// 桥测读回辅具（文件级自由函数——闭包持 `&mut sandbox` 跨 execute 触发 E0499）。
fn s403_read(sandbox: &mut zero_script_sandbox::V8Sandbox, id: &str) -> String {
    sandbox
        .execute(&format!("globalThis.__s403{id}"))
        .unwrap()
        .value
}

/// 钉①②③：嵌套脚本通道 + named access + 嵌套导航 + replaceState——
/// 静态 object（parser 面）data=test0.html，静默启航后嵌套文档脚本 parent 回调：
/// href0（嵌套 doc URL 印记）→ na（window[name] 已 morph 到 contentWindow）→
/// rs（嵌套 history.replaceState 改 doc._zwURL，零 fetch）→ 嵌套自导航 test1
/// （重物化 + 脚本重放；data 属性不动）。断言翻转面：静态 object 不启航（回调
/// 链永不执行 → seq 空）；脚本不接（seq 空）；导航走 iframe 重载（data 属性被
/// 改写 / fetch 回环）；replaceState 误 fetch（fallback URL 入 fetch log）。
/// 静默面：object 元素 onload/onerror 计数恒空（sweep 不派终态事件）。
#[test]
fn object_nested_script_channel_s403() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 静态 object（parser 面——无 data 属性 Setter、无插入钩子路径）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><object id=\"test\" name=\"obj\" data=\"test0.html\" \
         type=\"text/html\"></object></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/s403/page.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // host fetch 路由：test0/test1 → text/html 嵌套文档（test0 内嵌回调 + 自导航
    // 脚本）；fallback URL 恒 404（replaceState 不得 fetch——log 断言面）。
    let fetch_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let fetch_log_c = fetch_log.clone();
    Sandbox::register_callback(
        &mut sandbox,
        "__zw_fetch",
        Box::new(move |args: &[String]| -> String {
            let url = args.get(2).cloned().unwrap_or_default();
            fetch_log_c.lock().unwrap().push(url.clone());
            if url.contains("/s403/test0.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><head></head><body><script>parent.callback('test0');document.location.href = 'test1.html';</script></body></html>".to_string();
            }
            if url.contains("/s403/test1.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><head></head><body><script>parent.callback('test1');</script></body></html>".to_string();
            }
            "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/html\u{1f}".to_string()
        }),
    );

    // 第一段：页脚本（parent 回调定义——复刻 object-handler.html 断言链）。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s403 = [];\n\
             globalThis.__s403ev = [];\n\
             var t = document.getElementById('test');\n\
             t.onload = function () { __s403ev.push('load'); };\n\
             t.onerror = function () { __s403ev.push('error'); };\n\
             globalThis.callback = function (d) {\n\
               if (d === 'test0') {\n\
                 __s403.push('href0:' + (t.contentDocument.location.href.indexOf('test0.html') !== -1));\n\
                 __s403.push('na:' + (window['obj'] === t.contentWindow));\n\
                 window['obj'].history.replaceState({s: 1}, '', 'object-fallback.html');\n\
                 __s403.push('rs:' + (t.contentDocument.location.href.indexOf('object-fallback.html') !== -1));\n\
               } else if (d === 'test1') {\n\
                 __s403.push('href1:' + (t.contentDocument.location.href.indexOf('test1.html') !== -1));\n\
                 __s403.push('data:' + (String(t.data).indexOf('test0.html') !== -1));\n\
               }\n\
             };\n\
             })();",
        )
        .unwrap();

    // 第二段：parser 静态启航（part06 window load 分支扫描位同款直调）——
    // _defer 微任务 execute 末 drain，回调链（物化→脚本→导航→重放）全链同步。
    sandbox.execute("__zwStartParserObjects();").unwrap();

    // 第三段：断言。回调序 = href0/na/rs（test0 面）+ href1/data（test1 面）。
    let seq = s403_read(&mut sandbox, ".join('|')");
    assert_eq!(
        seq,
        "href0:true|na:true|rs:true|href1:true|data:true",
        "嵌套脚本回调链断言序（A 链）实得 {seq}; fetch={:?}",
        fetch_log.lock().unwrap()
    );
    // 静默面：sweep 不派终态事件。
    let events = s403_read(&mut sandbox, "ev.join('|')");
    assert_eq!(events, "", "静态 sweep 不派 load/error（静默面）实得 {events}");
    // 导航后嵌套 doc 停在 test1（replaceState 被导航覆盖）；fetch log 恒两取回
    //（test0 初载 + test1 导航；fallback URL 零 fetch——replaceState 只动 URL 槽）。
    let href = s403_read(
        &mut sandbox,
        "href = document.getElementById('test').contentDocument.location.href",
    );
    assert!(
        href.contains("test1.html"),
        "导航后嵌套 doc URL 停在 test1 实得 {href}"
    );
    let log = fetch_log.lock().unwrap();
    let fetched: Vec<&String> = log.iter().filter(|u| u.contains("/s403/")).collect();
    assert_eq!(fetched.len(), 2, "恰两取回（test0 + test1）实得 {fetched:?}");
    assert!(
        !log.iter().any(|u| u.contains("object-fallback.html")),
        "replaceState 不 fetch（fallback URL 零取回）实得 {:?}",
        *log
    );
}

/// 钉④：sweep 幂等 + 静默 + 导航失败面——二次 sweep/消费读零重取回（entry 缓存
/// 早返）；navigation 到 404 → fallback 语义（contentDocument/contentWindow null，
/// 事件恒空——once-terminal fired 门近似）。
#[test]
fn object_sweep_silent_idempotent_nav_error_s403() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // obj1：稳态 text/html；obj2：嵌套脚本自导航到 404（失败臂）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <object id=\"obj1\" name=\"o1\" data=\"once.html\"></object>\
         <object id=\"obj2\" name=\"o2\" data=\"nav404.html\"></object>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/s403b/page.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let fetch_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let fetch_log_c = fetch_log.clone();
    Sandbox::register_callback(
        &mut sandbox,
        "__zw_fetch",
        Box::new(move |args: &[String]| -> String {
            let url = args.get(2).cloned().unwrap_or_default();
            fetch_log_c.lock().unwrap().push(url.clone());
            if url.contains("/s403b/once.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Once</body></html>".to_string();
            }
            if url.contains("/s403b/nav404.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body><script>document.location.href = 'missing404.html';</script></body></html>".to_string();
            }
            "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/html\u{1f}".to_string()
        }),
    );
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s403b = [];\n\
             globalThis.__s403bev = [];\n\
             var o1 = document.getElementById('obj1'), o2 = document.getElementById('obj2');\n\
             o1.onload = o1.onerror = function (e) { __s403bev.push('o1:' + e.type); };\n\
             o2.onload = o2.onerror = function (e) { __s403bev.push('o2:' + e.type); };\n\
             __zwStartParserObjects();\n\
             var w1a = o1.contentWindow;\n\
             __zwStartParserObjects();\n\
             var w1b = o1.contentWindow;\n\
             __s403b.push('idem:' + (w1a === w1b));\n\
             __s403b.push('o1doc:' + (o1.contentDocument.location.href.indexOf('once.html') !== -1));\n\
             __s403b.push('o2doc:' + String(o2.contentDocument === null));\n\
             __s403b.push('o2win:' + String(o2.contentWindow === null));\n\
             })();",
        )
        .unwrap();

    let seq = s403_read(&mut sandbox, "b.join('|')");
    assert_eq!(
        seq,
        "idem:true|o1doc:true|o2doc:true|o2win:true",
        "幂等 + 导航失败 fallback 语义实得 {seq}"
    );
    let events = s403_read(&mut sandbox, "bev.join('|')");
    assert_eq!(events, "", "静默 sweep 恒无事件实得 {events}");
    let log = fetch_log.lock().unwrap();
    let once_count = log.iter().filter(|u| u.contains("/s403b/once.html")).count();
    assert_eq!(once_count, 1, "二次 sweep 零重取回（entry 缓存早返）实得 {once_count}");
    let miss_count = log
        .iter()
        .filter(|u| u.contains("/s403b/missing404.html"))
        .count();
    assert_eq!(miss_count, 1, "嵌套导航 404 恰一取回实得 {miss_count}");
}
