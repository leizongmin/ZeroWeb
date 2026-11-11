// part109 — html-semantics M2 切片 409（2026-11-12）：script-created link
// stylesheet 不阻塞 script（s163 延迟仿真对动态插入臂失效 + 阻塞计数门）。
//
// script-created-link-stylesheet-does-not-block-script.html /
// dynamic-render-blocking-link-stylesheet-does-not-block-script.html 双案红根
// 修复钉（`assert_false(styleExists(...), 'still pending')` 恒红）：
// - part01 `_zwFetchSimDelay` 发题前对页 base 归一相对 URL——script-created
//   link 插入臂 `_zwSsVisitCss(_ssHref, …)` 发题传原始 href（相对形），
//   s163 interactions 前缀分支按绝对形态设计 miss → delay 仿真不生效，瞬时
//   取回在脚本间微任务排空点完成（style 双胞胎 Pass 正因 @import 内层 URL
//   已绝对化）；
// - part03 `_ssDoVisit` 阻塞计数门控到 parser-inserted 面（noOverlay=true =
//   s152 静态交错扫描臂）——spec the-link-element：script-created stylesheet
//   不入 script-blocking 集，否则 s349 等待泵会把 delay 取回等完再放行后继
//   脚本，延迟仿真被泵消解。
// https://html.spec.whatwg.org/multipage/semantics.html#the-link-element
// https://fetch.spec.whatwg.org/#fetching

/// 桥测读回辅具（文件级自由函数——闭包持 `&mut sandbox` 跨 execute 触发 E0499）。
fn s409_read(sandbox: &mut zero_script_sandbox::V8Sandbox, id: &str) -> String {
    sandbox.execute(&format!("globalThis.__s409r.{id}")).unwrap().value
}

/// 钉①：script-created link 双形探针——插入后不入 script-blocking 集（计数门，
/// IIFE 内插后即时读 mid-window 态）+ 发题 URL 对页 base 归一（wire 面钉归一
/// 生效）+ settle 平衡无僵尸。断言翻转面：旧动态臂计数（relBlocking/absBlocking
/// =true）+ 旧相对形发题 URL 不归一（`stylesheet.py?delay=1` 裸形）。
///
/// 桥测窗口语义注记：V8 每次脚本求值末尾自动 perform microtask checkpoint——
/// 延迟发题链（`setTimeout` 无宿主回落 `_defer`）在**同一 execute**内泵完，
/// fetch log 在 execute 返回时即已齐写；runner timer wheel 的 1000ms 真窗由
/// 钉②（`_zwFetchSimDelay` 纯函数延迟值）+ driving solo（双案 F→P ×2 跑）
/// 承载，桥测不可观察。
#[test]
fn script_created_link_not_blocking_faces_s409() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><head></head><body><p>s409 script-created link pin</p></body></html>".to_string(),
    ));
    // 页锚 = interactions 目录（相对 href 归一后前缀命中 s163 延迟分支）。
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/html/semantics/document-metadata/interactions-of-styling-and-scripting/page.html".to_string(),
    ));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // fetch 计数桥（同步 wire——Rust 侧 log 计发题；settle 面 text/css load 形）。
    // 注：断言侧先 clone 再进 assert_eq!——guard 临时横跨宏求值 + 格式化参数
    // 二次 lock 会自锁（std Mutex 不可重入，挂起实测）。
    let fetch_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let fetch_log_c = fetch_log.clone();
    Sandbox::register_callback(
        &mut sandbox,
        "__zw_fetch",
        Box::new(move |args: &[String]| -> String {
            fetch_log_c
                .lock()
                .unwrap()
                .push(args.get(2).cloned().unwrap_or_default());
            "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/css\u{1f}h1 { color: purple; }".to_string()
        }),
    );

    // 同一 execute 内插两 link（相对/绝对 delay 端点）并在插后即时读
    // script-blocking mid-window 态——动态插入臂计数门断言面（spec：script-created
    // stylesheet 不入 script-blocking 集；旧动态臂 relBlocking/absBlocking=true）。
    sandbox
        .execute(
            "(function () {\n\
             var r = globalThis.__s409r = {};\n\
             var l1 = document.createElement('link');\n\
             l1.rel = 'stylesheet';\n\
             l1.href = 'stylesheet.py?delay=1';\n\
             document.head.appendChild(l1);\n\
             r.relBlocking = String(globalThis.__zwScriptBlockingSheetsPending && globalThis.__zwScriptBlockingSheetsPending());\n\
             var l2 = document.createElement('link');\n\
             l2.rel = 'stylesheet';\n\
             l2.href = 'https://wpt.test/html/semantics/document-metadata/interactions-of-styling-and-scripting/stylesheet.py?delay=1';\n\
             document.head.appendChild(l2);\n\
             r.absBlocking = String(globalThis.__zwScriptBlockingSheetsPending && globalThis.__zwScriptBlockingSheetsPending());\n\
             })();",
        )
        .unwrap();

    // 阻塞计数门：script-created 不入 script-blocking 集（旧动态臂计数=true）。
    assert_eq!(
        s409_read(&mut sandbox, "relBlocking"),
        "false",
        "script-created link 不入 blocking 集实得 {}",
        s409_read(&mut sandbox, "relBlocking")
    );
    assert_eq!(
        s409_read(&mut sandbox, "absBlocking"),
        "false",
        "script-created link（绝对形）不入 blocking 集实得 {}",
        s409_read(&mut sandbox, "absBlocking")
    );

    // 发题面：execute 末尾检查点已泵完延迟发题链——双 link 齐写且相对形 URL
    // 已归一为绝对 interactions 路径（s409 归一 wire 面钉；旧相对形发题裸
    // `stylesheet.py?delay=1` miss s163 前缀分支 → 延迟仿真不生效）。
    let log = fetch_log.lock().unwrap().clone();
    assert_eq!(log.len(), 2, "检查点后双 link 发题齐写实得 {log:?}");
    for url in log.iter() {
        assert!(
            url.contains("/interactions-of-styling-and-scripting/stylesheet.py?delay=1"),
            "发题 URL 已归一为绝对 interactions 路径实得 {url}"
        );
    }
}

/// 钉②：`_zwFetchSimDelay` 相对 URL 归一化单元钉（纯函数面）。断言翻转面：
/// 旧相对形 miss interactions 前缀 → 0。对照面：异目录同名端点不误延迟
///（the-link-element/stylesheet.py 语义不同——take-and-clear 计数）、
/// delay.py 形不变。
#[test]
fn fetch_sim_delay_relative_normalization_faces_s409() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body>s409 delay pin</body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "https://wpt.test/html/semantics/document-metadata/interactions-of-styling-and-scripting/page.html".to_string(),
    ));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "(function () {\n\
             var r = globalThis.__s409r = {};\n\
             var d = function (u) { return String(globalThis._zwFetchSimDelay(u)); };\n\
             r.rel = d('stylesheet.py?delay=1');\n\
             r.abs = d('https://wpt.test/html/semantics/document-metadata/interactions-of-styling-and-scripting/stylesheet.py?delay=1');\n\
             r.otherDir = d('other/dir/stylesheet.py?delay=1');\n\
             r.linkElemDir = d('https://wpt.test/html/semantics/document-metadata/the-link-element/stylesheet.py?delay=1');\n\
             r.delayPy = d('delay.py?ms=50');\n\
             r.plain = d('foo.css');\n\
             })();",
        )
        .unwrap();

    assert_eq!(
        s409_read(&mut sandbox, "rel"),
        "1000",
        "相对 delay 形归一后命中 s163 分支实得 {}",
        s409_read(&mut sandbox, "rel")
    );
    assert_eq!(
        s409_read(&mut sandbox, "abs"),
        "1000",
        "绝对 delay 形本态保持实得 {}",
        s409_read(&mut sandbox, "abs")
    );
    assert_eq!(
        s409_read(&mut sandbox, "otherDir"),
        "0",
        "归一后异目录前缀仍不命中实得 {}",
        s409_read(&mut sandbox, "otherDir")
    );
    assert_eq!(
        s409_read(&mut sandbox, "linkElemDir"),
        "0",
        "the-link-element 同名端点不误延迟实得 {}",
        s409_read(&mut sandbox, "linkElemDir")
    );
    assert_eq!(
        s409_read(&mut sandbox, "delayPy"),
        "50",
        "delay.py ms 形不变实得 {}",
        s409_read(&mut sandbox, "delayPy")
    );
    assert_eq!(
        s409_read(&mut sandbox, "plain"),
        "0",
        "无延迟资源恒 0 实得 {}",
        s409_read(&mut sandbox, "plain")
    );
}
