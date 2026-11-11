// part105 — html-semantics M2 切片 405（2026-11-11）：embed 动态加载链（s399 镜像）。
//
// embed-javascript-url.html 文件级 Timeout 根因修复钉（embed 元素无任何加载面——
// 无插入钩子/无加载核/无嵌套 navigable）：
// - `_zwStartConnectedEmbed` 动态插入启航（part04 两插入路同排）——共享
//   `_zwLoadObjectResource` 加载核（embed: 标记 → src 属性读），入共享
//   _zwObjectNavCache → window[N]/window.length 自动计入（s404 索引层）；
// - 失败终态映射 load（embed-network-error 四子测钉「失败 → load 派、error 恒不
//   派」——display no plugin 语义；object 保持 error）；
// - javascript: src 全静默（embed-javascript-url 子测1钉「should do nothing」）；
// - EMBED.src IDL setter 重启臂（part05，镜像 s399 OBJECT.data——已连接 embed 的
//   src 更新重启资源取回，embed-change-src 驱动案）。
// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-embed-element-setup-steps

/// 桥测读回辅具（文件级自由函数——闭包持 `&mut sandbox` 跨 execute 触发 E0499）。
fn s405_read(sandbox: &mut zero_script_sandbox::V8Sandbox, id: &str) -> String {
    sandbox
        .execute(&format!("globalThis.__s405{id}"))
        .unwrap()
        .value
}

/// 钉①②③：embed javascript: 三面（embed-javascript-url 三子测镜像）——
/// ①src=javascript:"foo" 全静默（无 load **亦无 error**——embed 失败映射 load
/// 但 javascript: 非取回失败零事件）；②已载 embed `window[0].location.href=
/// 'javascript:"<b>x</b>"'` → 完成值重物化 + load 重派 + 内容落嵌套 doc（embed 无
/// contentDocument IDL，经 window[0].document 读）+ URL 槽不动；③javascript:1
/// 非字符串完成值 → 静默中止。断言翻转面：javascript: 误派 error（事件序非空）；
/// embed 未入 window[N] 活跃集合（window[0] undefined）；javascript: 误 fetch。
#[test]
fn embed_javascript_url_faces_s405() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/s405/page.html".to_string()));
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
            if url.contains("/s405/blank.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Blank</body></html>".to_string();
            }
            "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/html\u{1f}".to_string()
        }),
    );

    // 第一段：emb1（src=javascript:"foo"——子测1镜像）+ emb2（src=blank.html——
    // insertEmbedNavigable 镜像）。drain 后 emb2 初载 load1。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s405ev1 = [];\n\
             globalThis.__s405loads = 0;\n\
             var e1 = document.createElement('embed');\n\
             e1.id = 's405e1';\n\
             e1.onload = function () { globalThis.__s405ev1.push('load'); };\n\
             e1.onerror = function () { globalThis.__s405ev1.push('error'); };\n\
             e1.src = 'javascript:\"foo\"';\n\
             document.body.append(e1);\n\
             var e2 = document.createElement('embed');\n\
             e2.id = 's405e2';\n\
             e2.onload = function () { globalThis.__s405loads++; };\n\
             e2.src = '/s405/blank.html';\n\
             document.body.append(e2);\n\
             globalThis.__s405e2 = e2;\n\
             })();",
        )
        .unwrap();

    // 第二段：子测1面（零事件）+ 索引面 + 子测2导航（完成值字符串）。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s405p1 = globalThis.__s405ev1.join(',') || 'none';\n\
             globalThis.__s405p2 = String(globalThis.length) + '/' + String(window[0] !== undefined && window[0] !== null);\n\
             globalThis.__s405w0 = window[0];\n\
             window[0].location.href = 'javascript:\"<b>x</b>\"';\n\
             })();",
        )
        .unwrap();

    // 第三段：子测2断言（load2 + 内容经 window[0].document + URL 槽）+ 子测3导航。
    sandbox
        .execute(
            "(function () {\n\
             var doc = window[0].document;\n\
             globalThis.__s405p3 = [\n\
               String(globalThis.__s405loads),\n\
               String(doc.body.textContent),\n\
               String(doc.location.href.indexOf('blank.html') !== -1)\n\
             ].join('|');\n\
             window[0].location.href = 'javascript:1';\n\
             })();",
        )
        .unwrap();

    // 第四段：子测3断言（无第三 load、doc 不动）+ 索引 live 解析保持。
    // FIXME：跨导航 WindowProxy identity 不钉（s404 同款记档——重物化建新 win）。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s405p4 = [\n\
               String(globalThis.__s405loads),\n\
               String(window[0].document.body.textContent),\n\
               String(window[0] !== undefined && window[0] !== null)\n\
             ].join('|');\n\
             })();",
        )
        .unwrap();

    assert_eq!(
        s405_read(&mut sandbox, "p1"),
        "none",
        "(K1) src=javascript:\"foo\" → 零事件（javascript: 全静默——load/error 均不派）"
    );
    assert_eq!(
        s405_read(&mut sandbox, "p2"),
        "1/true",
        "(K2) emb1 静默不派事件不计 length（error 态无 win）；window[0] = emb2 嵌套上下文（初载 load1）"
    );
    assert_eq!(
        s405_read(&mut sandbox, "p3"),
        "2|x|true",
        "(K3) javascript:\"<b>x</b>\" → 完成值重物化：load 重派（1→2）+ 内容落嵌套 doc（window[0].document 读——embed 无 contentDocument IDL）+ URL 槽不动"
    );
    assert_eq!(
        s405_read(&mut sandbox, "p4"),
        "2|x|true",
        "(K4) javascript:1 非字符串完成值 → 静默中止：无第三 load、doc 不动"
    );
    let log = fetch_log.lock().unwrap();
    assert_eq!(
        log.iter().filter(|u| u.contains("blank.html")).count(),
        1,
        "emb2 初载恰一取回（javascript: 臂零 fetch）实得 {:?}",
        *log
    );
    assert!(
        !log.iter().any(|u| u.contains("javascript:")),
        "javascript: URL 不落 host fetch 实得 {:?}",
        *log
    );
}

/// 钉④⑤⑥：失败映射 load + data: text/html 物化 + src 重启臂——
/// ①src=missing 404 → load 派（**error 不派**——embed-network-error 语义）+
/// error 态无 navigable 不计 length；②data:text/html → 嵌套上下文物化 + window[0]
/// （error 态 embed3 不计入）；③已连接 embed src IDL 更新 → 重启（旧 entry 作废
/// 重建 fired 门复位 → 第二 load 派 + 嵌套 doc 换新内容）。断言翻转面：失败误派
/// error（事件序错）；重启不删旧 entry（fired 门残留 → 第二 load 不派）。
#[test]
fn embed_failure_load_restart_faces_s405() {
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
        "<html><body></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> =
        Arc::new(Mutex::new("https://wpt.test/s405b/page.html".to_string()));
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
            "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/html\u{1f}".to_string()
        }),
    );

    // 第一段：emb3（404 失败面）+ emb4（data:text/html 物化面）。drain 后 emb3
    // failure→load、emb4 物化。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s405bev3 = [];\n\
             globalThis.__s405bloads4 = 0;\n\
             var e3 = document.createElement('embed');\n\
             e3.id = 's405be3';\n\
             e3.onload = function () { globalThis.__s405bev3.push('load'); };\n\
             e3.onerror = function () { globalThis.__s405bev3.push('error'); };\n\
             e3.src = '/s405b/missing404.html';\n\
             document.body.append(e3);\n\
             var e4 = document.createElement('embed');\n\
             e4.id = 's405be4';\n\
             e4.onload = function () { globalThis.__s405bloads4++; };\n\
             e4.src = 'data:text/html,<p>emb</p>';\n\
             document.body.append(e4);\n\
             globalThis.__s405be4 = e4;\n\
             })();",
        )
        .unwrap();

    // 第二段：失败面 + 物化面 + 重启（src IDL 更新）。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s405bp1 = globalThis.__s405bev3.join(',') + '/' + String(globalThis.length);\n\
             globalThis.__s405bp2 = String(window[0] !== undefined && window[0] !== null && String(window[0].document.body.textContent) === 'emb');\n\
             globalThis.__s405be4.src = 'data:text/html,<p>emb2</p>';\n\
             })();",
        )
        .unwrap();

    // 第三段：重启面（第二 load + doc 换新）。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s405bp3 = [\n\
               String(globalThis.__s405bloads4),\n\
               String(window[0].document.body.textContent)\n\
             ].join('|');\n\
             })();",
        )
        .unwrap();

    assert_eq!(
        s405_read(&mut sandbox, "bp1"),
        "load/1",
        "(L1) 404 失败 → load 派 error 不派（embed-network-error 语义）；length=1 仅物化的 emb4（emb3 error 态无 navigable 不计）"
    );
    assert_eq!(
        s405_read(&mut sandbox, "bp2"),
        "true",
        "(L2) data:text/html → 嵌套上下文物化；window[0] = emb4（emb3 error 态不计入索引）"
    );
    assert_eq!(
        s405_read(&mut sandbox, "bp3"),
        "2|emb2",
        "(L3) src IDL 更新 → 重启：旧 entry 作废重建（fired 门复位）→ 第二 load 派（1→2）+ 嵌套 doc 换新内容"
    );
    assert_eq!(
        fetch_log
            .lock()
            .unwrap()
            .iter()
            .filter(|u| u.contains("missing404.html"))
            .count(),
        1,
        "失败面恰一取回"
    );
}
