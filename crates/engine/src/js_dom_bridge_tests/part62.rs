// part62 — navigation-compat M3-S1 切片：iframe 浏览上下文最小面单测。
// 契约：docs/goal/archive/navigation-compat.md DC-3（M3 最小面——contentWindow/contentDocument/
// frames/parent/top + iframe 元素属性语义）；WPT 案面 the-iframe-element/
// {sandbox-ascii-case-insensitive, iframe-load-event, content_document_changes_only_
// after_load_matures, document-getters-return-null-for-cross-origin, iframe-network-
// error.sub} 与 srcdoc 处理族。沙盒模式沿用 part21/23 先例：persistent_context +
// `__zw_fetch` 同步契约 + 微任务泵（R288 load 延迟派发与 M3 成熟度交换同在
// microtask 边界落定）。

/// M3-S1 公共沙盒搭建：shim 装载 + DOM 回调注册（page_url 可控——iframe 相对
/// src 解析与 referrer/location 断言依赖它）。
fn m3s1_sandbox(page_url: &str, dom_html: &str) -> zero_script_sandbox::V8Sandbox {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(dom_html.to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new(page_url.to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox
}

/// M3-S1 srcdoc 初始处理（spec `#process-the-iframe-attributes`——srcdoc 在场即导航
/// 到 about:srcdoc 文档，继承父文档 base/origin；无 src 也适用）。WPT 对应面：
/// srcdoc_processor 系列（_markup 断言 contentDocument 反映 srcdoc 标记）。
#[test]
fn m3s1_srcdoc_initial_processing_and_url() {
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/srcdoc-initial.html",
        "<html><body><iframe srcdoc='<p id=\"s\">hi</p>'></iframe></body></html>",
    );
    let out = sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             var d = f.contentDocument;\
             JSON.stringify({\
               el: d.getElementById('s') ? d.getElementById('s').textContent : null,\
               url: f.contentWindow.location.href\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"el":"hi","url":"about:srcdoc"}"#,
        "srcdoc 初始处理：contentDocument 反映标记、地址 about:srcdoc"
    );
}

/// M3-S1 srcdoc 优先于 src（spec `#process-the-iframe-attributes` 步序——srcdoc 在场
/// 时不发起 src fetch）。fetch 回调记录零调用为优先性证据。
#[test]
fn m3s1_srcdoc_precedes_src_no_fetch() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/srcdoc-precedence.html",
        "<html><body><iframe src='dummy.html' srcdoc='<b id=\"b\">x</b>'></iframe></body></html>",
    );
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetch_log = fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            fetch_log
                .lock()
                .unwrap()
                .push(args.get(2).cloned().unwrap_or_default());
            "__zwfr:200\x1fOK\x1f\x1fshould-not-load".to_string()
        }),
    );
    let out = sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             var d = f.contentDocument;\
             JSON.stringify({\
               el: d.getElementById('b') ? d.getElementById('b').textContent : null,\
               url: f.contentWindow.location.href\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"el":"x","url":"about:srcdoc"}"#,
        "srcdoc 与 src 同在场：内容取 srcdoc、地址 about:srcdoc"
    );
    assert!(
        fetches.lock().unwrap().is_empty(),
        "srcdoc 优先：不得发起 src fetch"
    );
}

/// M3-S1 srcdoc 属性变更 → 重处理（spec：srcdoc set/change/remove 都触发
/// process-the-iframe-attributes）+ 成熟度门（旧 Document 保持到新文档 mature——
/// content_document_changes_only_after_load_matures 的同步断言面）+ 元素 load
/// 为 trusted Event（iframe-load-event 断言面）。swap 与 load 同批次延后到
/// microtask（R288 语义复用）。
#[test]
fn m3s1_srcdoc_attr_change_matures_with_trusted_load() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/srcdoc-mutate.html",
        "<html><body></body></html>",
    );
    sandbox
        .execute(
            "var f = document.createElement('iframe');\
             document.body.appendChild(f);\
             var log = [];\
             f.contentDocument;\
             f.onload = function (e) {\
               log.push('load:' + (e.isTrusted === true) + ':' + (e.constructor === Event));\
             };\
             f.setAttribute('srcdoc', '<p id=\"s2\">new</p>');\
             log.push('sync-new:' + !!f.contentDocument.getElementById('s2'));\
             log.push('sync-url:' + f.contentDocument.location.toString());\
             globalThis.__m3log = log;",
        )
        .unwrap();
    // 微任务泵：swap + trusted load 同批次落定（沙箱在 execute 间冲刷微任务，
    // 同步期证据 = log 前两条——成熟度门内旧文档保持、无 load）。
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    let out = sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             JSON.stringify({\
               el: f.contentDocument.getElementById('s2')\
                 ? f.contentDocument.getElementById('s2').textContent : null,\
               url: f.contentDocument.location.toString(),\
               log: globalThis.__m3log.join('|')\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"el":"new","url":"about:srcdoc","log":"sync-new:false|sync-url:about:blank|load:true:true"}"#,
        "同步期旧文档保持（log 前两条），成熟后切 srcdoc 文档且 load 单次 trusted"
    );
}

/// M3-S1 srcdoc 移除 → 重处理回退空文档（spec：removeAttribute('srcdoc') 同样触发
/// 属性处理；此时 src 亦缺席 → about:blank 初始文档，srcdoc 内容不可达）。
#[test]
fn m3s1_srcdoc_removal_reverts_to_about_blank() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/srcdoc-remove.html",
        "<html><body><iframe srcdoc='<p id=\"s\">hi</p>'></iframe></body></html>",
    );
    sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             f.contentDocument;\
             f.removeAttribute('srcdoc');\
             globalThis.__m3sync = f.contentDocument.getElementById('s')\
               ? f.contentDocument.getElementById('s').textContent : 'gone';",
        )
        .unwrap();
    // 同步期：旧 srcdoc 文档保持（成熟度门）。
    assert_eq!(
        sandbox.execute("globalThis.__m3sync").unwrap().value,
        "hi",
        "srcdoc 移除后同步读仍见旧文档"
    );
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    let out = sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             JSON.stringify({\
               gone: !f.contentDocument.getElementById('s'),\
               url: f.contentDocument.location.toString()\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"gone":true,"url":"about:blank"}"#,
        "成熟后回退 about:blank 空文档"
    );
}

/// M3-S1 `sandbox.supports()` 固定 supported tokens 表（spec
/// `#attr-iframe-sandbox`——supports 只查标准 token 集合，与元素当前属性值无关；
/// sandbox-ascii-case-insensitive 对值写 İ 形近字仍问 supports('allow-same-origin')）。
#[test]
fn m3s1_sandbox_supports_fixed_token_list() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/sandbox-supports.html",
        "<html><body></body></html>",
    );
    let out = sandbox
        .execute(
            "var f = document.createElement('iframe');\
             var r = ['allow-scripts', 'allow-same-origin', 'allow-forms', 'allow-popups',\
                      'allow-top-navigation', 'allow-top-navigation-by-user-activation',\
                      'allow-presentation', 'allow-downloads', 'allow-modals'].map(\
                      function (t) { return f.sandbox.supports(t); });\
             r.push(f.sandbox.supports('allow-foo'));\
             r.push(f.sandbox.supports(''));\
             f.setAttribute('sandbox', 'allow-same-or\\u0130gin');\
             r.push(f.sandbox.supports('allow-same-origin'));\
             r.join('|')",
        )
        .unwrap()
        .value;
    assert_eq!(
        out, "true|true|true|true|true|true|true|true|true|false|false|true",
        "sandbox.supports 查固定 token 表（值无关、未知/空串 false）"
    );
}

/// M3-S1 sandbox 指令 ASCII 大小写不敏感解析 + 双门：`ALLOW-SAME-ORIGIN`（大写）
/// 命中 allow-same-origin → contentDocument 可达；无 allow-scripts → 子文档脚本
/// 不执行；`ALLOW-SCRIPTS` 命中 → 子脚本执行（spec `#parse-a-sandboxing-directive`
/// ——token 先 ASCII 小写再匹配；İ/ſ 形近字不经 ASCII 小写 → 不命中）。
/// WPT：sandbox-ascii-case-insensitive + sandboxed-scripts-not-executed 面。
#[test]
fn m3s1_sandbox_ascii_parse_gates_scripts_and_origin() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/sandbox-ascii.html",
        "<html><body>\
         <iframe id='a' sandbox='ALLOW-SAME-ORIGIN ALLOW-SCRIPTS'\
           srcdoc='<script>window.__ran = true;</script>'></iframe>\
         <iframe id='n' sandbox=''\
           srcdoc='<script>window.__ran = true;</script>'></iframe>\
         </body></html>",
    );
    let out = sandbox
        .execute(
            "var a = document.getElementById('a');\
             var n = document.getElementById('n');\
             JSON.stringify({\
               aRan: a.contentWindow.__ran === true,\
               aDoc: a.contentDocument != null,\
               nRan: n.contentWindow.__ran === true,\
               nDoc: n.contentDocument != null\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"aRan":true,"aDoc":true,"nRan":false,"nDoc":false}"#,
        "ASCII 大写 token 命中 allow-same-origin/allow-scripts；空 sandbox 断脚本与同源"
    );
}

/// M3-S1 sandbox 无 allow-same-origin → contentDocument/getSVGDocument null
/// （spec `#dom-iframe-contentdocument`——沙箱 opaque origin 即 cross-origin；
/// document-getters-return-null-for-cross-origin 用 data: URL 同面——data: 载入
/// 文档的 entry.url 以 data: 起头，同门返 null）。
#[test]
fn m3s1_cross_origin_content_document_null() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/doc-getters-null.html",
        "<html><body>\
         <iframe id='s' sandbox='allow-scripts' srcdoc='<p>x</p>'></iframe>\
         <iframe id='d' src='data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\"/>'></iframe>\
         </body></html>",
    );
    let out = sandbox
        .execute(
            "var s = document.getElementById('s');\
             var d = document.getElementById('d');\
             JSON.stringify({\
               sDoc: s.contentDocument === null,\
               sSvg: s.getSVGDocument() === null,\
               dDoc: d.contentDocument === null,\
               dSvg: d.getSVGDocument() === null\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"sDoc":true,"sSvg":true,"dDoc":true,"dSvg":true}"#,
        "opaque-origin 沙箱与 data: URL 的 contentDocument/getSVGDocument 均 null"
    );
}

/// M3-S1 `window.frames` named + indexed 访问（R177 索引面 + M3 named 面——
/// `frames.<name>`/`frames['name']` 按 iframe name 内容属性解析到 contentWindow，
/// iframe-network-error 的 `frames.existingIframe1.location.href` 断言入口）。
#[test]
fn m3s1_frames_named_and_indexed_access() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/frames-named.html",
        "<html><body><iframe name='f1' srcdoc='<p id=\"sp\">child</p>'></iframe></body></html>",
    );
    let out = sandbox
        .execute(
            "var w = window.frames;\
             var cw = document.querySelector('iframe').contentWindow;\
             JSON.stringify({\
               len: w.length,\
               zero: w[0] === cw,\
               named: w.f1 === cw,\
               bracket: w['f1'] === cw\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"len":1,"zero":true,"named":true,"bracket":true}"#,
        "frames 索引 + named 双形态解析到同一 contentWindow"
    );
}

/// M3-S1 src 赋值导航的成熟度序 + trusted load（content_document_changes_only_
/// after_load_matures 去 trickle 面：src 赋值后同步读仍 about:blank，load 微任务
/// 派发且 trusted）。fetch 同步契约下子文档即取即结（part23 先例）。
#[test]
fn m3s1_src_set_sync_view_unchanged_then_trusted_load() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/src-matures.html",
        "<html><body></body></html>",
    );
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |_args| {
            "__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1f<!doctype html><p id=\"m\">loaded</p>"
                .to_string()
        }),
    );
    sandbox
        .execute(
            "var f = document.createElement('iframe');\
             document.body.appendChild(f);\
             var log = [];\
             f.onload = function (e) { log.push('trusted:' + (e.isTrusted === true)); };\
             log.push('init:' + f.contentDocument.location.toString());\
             f.setAttribute('src', 'support/iframe-that-checks-contentDocument.html');\
             log.push('sync:' + f.contentDocument.location.toString());\
             globalThis.__m3 = log;",
        )
        .unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    let out = sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             JSON.stringify({\
               el: f.contentDocument.getElementById('m')\
                 ? f.contentDocument.getElementById('m').textContent : null,\
               url: f.contentDocument.location.toString(),\
               log: globalThis.__m3.join('|')\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"el":"loaded","url":"https://wpt.test/html/semantics/embedded-content/the-iframe-element/support/iframe-that-checks-contentDocument.html","log":"init:about:blank|sync:about:blank|trusted:true"}"#,
        "src 赋值同步期仍见 about:blank（log 前两条），成熟后切新文档且 load 单次 trusted"
    );
}

/// M3-S1 子文档 location.href 写 → iframe 重导航（part05 iframeLocation href
/// setter——viaNavigation 绕 srcdoc 属性处理，按 URL fetch；iframe-network-error
/// 的子导航入口面）。子脚本在父文档 load 管线内同步执行（R206 合并作用域），
/// 导航链：starter → child-target 两跳 fetch 均命中。
#[test]
fn m3s1_child_location_href_navigates_frame() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s1_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/child-nav.html",
        "<html><body><iframe src='starter.html'></iframe></body></html>",
    );
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetch_log = fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).cloned().unwrap_or_default();
            fetch_log.lock().unwrap().push(url.clone());
            if url.ends_with("child-target.html") {
                "__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1f<!doctype html><p id=\"ct\">arrived</p>"
                    .to_string()
            } else {
                "__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1f<!doctype html><script>location.href = 'child-target.html';</script>"
                    .to_string()
            }
        }),
    );
    sandbox.execute("document.querySelector('iframe').contentDocument;").unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    let out = sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             JSON.stringify({\
               el: f.contentDocument.getElementById('ct')\
                 ? f.contentDocument.getElementById('ct').textContent : null,\
               url: f.contentDocument.location.toString()\
             })",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"el":"arrived","url":"https://wpt.test/html/semantics/embedded-content/the-iframe-element/child-target.html"}"#,
        "子 location.href 写驱动 iframe 导航到目标文档"
    );
    let logged = fetches.lock().unwrap();
    assert!(
        logged.len() >= 2
            && logged.iter().any(|u| u.ends_with("starter.html"))
            && logged
                .iter()
                .any(|u| u.ends_with("child-target.html")),
        "两跳 fetch 均发起（starter → child-target），实得 {:?}",
        logged.iter().take(4).collect::<Vec<_>>()
    );
}
