// part106 — html-semantics M2 切片 406（2026-11-11）：frames named access 扩
// embed/object + 嵌套导航失败终态（embed 重派 load）。
//
// embed-network-error.sub.html 子测 3/4（「navigating an existing embed」2F）根因
// 修复钉（s405 后余红——R177 帧代理 iframe-only + 导航失败臂静默）：
// - R177 frames Proxy named access 扩 embed/object：树序 querySelectorAll 扫描 +
//   name 内容属性匹配 + `_zwObjectNavCache` entry.win 解析（embed 无 contentWindow
//   IDL；spec named access 值算法「唯一 named 元素有嵌套浏览上下文 → 返其
//   WindowProxy」）；
// - `_zwObjectNavigateEntry` 失败三臂收拢 `_zwObjectNavFail`：embed entry 失败重派
//   终态事件（语料钉「失败 → load 派、error 恒不派」，fired 门复位），object 保持
//   静默（once-terminal 近似——part103 钉面不破）。
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object

/// 桥测读回辅具（文件级自由函数——闭包持 `&mut sandbox` 跨 execute 触发 E0499）。
fn s406_read(sandbox: &mut zero_script_sandbox::V8Sandbox, id: &str) -> String {
    sandbox
        .execute(&format!("globalThis.__s406{id}"))
        .unwrap()
        .value
}

/// 钉①②③④：embed named access + 导航失败 load 重派（driving 镜像）+ object
/// named access parity + object 导航失败静默保持。断言翻转面：frames.named 仍
/// iframe-only（existingEmbed1 undefined → 「Cannot read properties of undefined
/// (reading 'location')」红态）；导航失败误派 error（事件序污染）；object 误重派
/// 事件（part103 once-terminal 近似破面）。
#[test]
fn frames_named_embed_nav_fail_faces_s406() {
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
        Arc::new(Mutex::new("https://wpt.test/s406/page.html".to_string()));
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
            if url.contains("/s406/blank.html") || url.contains("/s406/blank2.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Blank</body></html>".to_string();
            }
            "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/html\u{1f}".to_string()
        }),
    );

    // 第一段：emb（name 属性面 + 子测3 镜像——onload 内 frames.named 解析 + 导航
    // 404 + onload/onerror 重赋）+ obj（name 面同构——parity + object 静默保持）。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s406ev = [];\n\
             var em = document.createElement('embed');\n\
             em.name = 'existingEmbed1';\n\
             em.src = '/s406/blank.html';\n\
             em.onload = function () {\n\
               globalThis.__s406ev.push('load1:' + String(globalThis.frames.existingEmbed1 !== undefined));\n\
               em.onload = function () { globalThis.__s406ev.push('load2'); };\n\
               em.onerror = function () { globalThis.__s406ev.push('error2'); };\n\
               globalThis.frames.existingEmbed1.location.href = '/s406/missing.html';\n\
             };\n\
             em.onerror = function () { globalThis.__s406ev.push('error1'); };\n\
             document.body.append(em);\n\
             var ob = document.createElement('object');\n\
             ob.name = 'existingObj1';\n\
             ob.data = '/s406/blank2.html';\n\
             ob.onload = function () {\n\
               globalThis.__s406ev.push('objload1:' + String(globalThis.frames.existingObj1 === ob.contentWindow));\n\
               ob.onload = function () { globalThis.__s406ev.push('objload2'); };\n\
               ob.onerror = function () { globalThis.__s406ev.push('objerror2'); };\n\
               globalThis.frames.existingObj1.location.href = '/s406/missing2.html';\n\
             };\n\
             ob.onerror = function () { globalThis.__s406ev.push('objerror1'); };\n\
             document.body.append(ob);\n\
             var em2 = document.createElement('embed');\n\
             em2.name = 'invalidNavEmbed';\n\
             em2.src = '/s406/blank.html';\n\
             em2.onload = function () {\n\
               globalThis.__s406ev.push('em2load1');\n\
               em2.onload = function () { globalThis.__s406ev.push('em2load2'); };\n\
               em2.onerror = function () { globalThis.__s406ev.push('em2error2'); };\n\
               globalThis.frames.invalidNavEmbed.location.href = '//{{hosts[][nonexistent]}}/';\n\
             };\n\
             em2.onerror = function () { globalThis.__s406ev.push('em2error1'); };\n\
             document.body.append(em2);\n\
             })();",
        )
        .unwrap();

    // 第二段：事件面（embed load 重派 + error 恒缺 + object parity + object 静默）
    // + 失败后 fallback 态（双 entry win 槽清空 → frames.named 不再解析 + length 0）。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s406p1 = [\n\
               String(globalThis.__s406ev.indexOf('load1:true') !== -1),\n\
               String(globalThis.__s406ev.indexOf('load2') !== -1),\n\
               String(globalThis.__s406ev.indexOf('objload1:true') !== -1),\n\
               String(globalThis.__s406ev.indexOf('em2load2') !== -1),\n\
               String(globalThis.__s406ev.filter(function (x) { return x.indexOf('error') === 0; }).length),\n\
               String(globalThis.__s406ev.length)\n\
             ].join('|');\n\
             globalThis.__s406p2 = String(globalThis.length) + '/' +\n\
               String(globalThis.frames.existingEmbed1 === undefined) + '/' +\n\
               String(globalThis.frames.existingObj1 === undefined);\n\
             })();",
        )
        .unwrap();

    assert_eq!(
        s406_read(&mut sandbox, "p1"),
        "true|true|true|true|0|5",
        "(M1) embed named access 解析（load1:true）+ 导航 404 → 第二 load（load2）+ error 恒不派（0）+ object parity（objload1:true）+ object 导航失败静默 + URL 解析失败容差回落（em2load2——`//{{hosts[][nonexistent]}}/` 字面 → _zwResolveIframeSrc 落取回失败 → load）"
    );
    assert_eq!(
        s406_read(&mut sandbox, "p2"),
        "0/true/true",
        "(M2) 失败后 fallback 态：双 entry win 槽清空 → length 0 + frames.named 不再解析（spec：失败无嵌套浏览上下文）"
    );
    let log = fetch_log.lock().unwrap();
    assert_eq!(
        log.iter().filter(|u| u.contains("/s406/blank.html")).count(),
        2,
        "双 embed 初载恰二取回实得 {:?}",
        *log
    );
    assert_eq!(
        log.iter().filter(|u| u.contains("/s406/blank2.html")).count(),
        1,
        "object 初载恰一取回实得 {:?}",
        *log
    );
    assert_eq!(
        log.iter().filter(|u| u.contains("/s406/missing")).count(),
        2,
        "双导航失败各恰一取回（objnav:<key>:nav 通道）实得 {:?}",
        *log
    );
    assert_eq!(
        log.iter().filter(|u| u.contains("{{hosts")).count(),
        1,
        "URL 解析失败容差：字面 token 落 _zwResolveIframeSrc（https://wpt.test 前缀）恰一取回实得 {:?}",
        *log
    );
}
