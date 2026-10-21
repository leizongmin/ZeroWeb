// part63 — navigation-compat M3-S2 切片：子文档 location 合流 + hash 写侧 + 合成树
// innerText 单测。契约：docs/goal/archive/navigation-compat.md DC-3（M3 最小面续——
// contentDocument.location 与 contentWindow.location 同一对象（spec
// #dom-document-location）、子窗 location.hash 写侧同文档片段导航、detached/iframe
// 文档 body.innerText 合成树近似）。WPT 案面 srcdoc_process_attributes /
// iframe_remove_src / srcdoc-anchor / srcdoc-attribute-reset / srcdoc_change_hash
// （前两步）。沙盒模式沿用 part62 先例。

/// M3-S2 公共沙盒搭建（同 part62 m3s1_sandbox）。
fn m3s2_sandbox(page_url: &str, dom_html: &str) -> zero_script_sandbox::V8Sandbox {
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

/// M3-S2 doc.location ↔ win.location 同一对象（spec
/// https://html.spec.whatwg.org/multipage/nav-history-apis.html#dom-document-location
/// ——browsing context 内 document.location 与 window.location 返回同一 Location）。
/// win 物化后 contentDocument.location 具全 URL 解析面（protocol——WPT
/// srcdoc_process_attributes / iframe_remove_src 断言面）；blob:-src 档案
/// protocol 'blob:'。未物化文档 toString 'about:blank' 面维持（成熟度门读法）。
#[test]
fn m3s2_doc_location_same_object_as_window_location() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/doc-location.html",
        "<html><body><iframe srcdoc='<p>s</p>'></iframe>\
         <iframe id=\"n\"></iframe></body></html>",
    );
    // blob:-src iframe 走 host fetch 同步契约（R115）——回显 200 + Final-URL 头
    // （R360 消费面），使 doc._zwURL 记账 blob: 档案。
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetch_log = fetches.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).cloned().unwrap_or_default();
            fetch_log.lock().unwrap().push(url.clone());
            format!("__zwfr:200\x1fOK\x1fx-zero-final-url\x1e{}\x1e\x1fsrc", url)
        }),
    );
    let out = sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             var d = f.contentDocument;\
             var w = f.contentWindow;\
             var never = document.getElementById('n');\
             var r = {\
               same: d.location === w.location,\
               docProto: d.location.protocol,\
               winProto: w.location.protocol,\
               docHref: d.location.href,\
               neverToString: never.contentDocument.location.toString()\
             };\
             try {\
               var bu = URL.createObjectURL(new Blob(['x'], { type: 'text/html' }));\
               var bf = document.createElement('iframe');\
               bf.src = bu;\
               document.body.appendChild(bf);\
               bf.contentDocument;\
               r.blobProto = bf.contentDocument.location.protocol;\
             } catch (e) { r.blobProto = 'ERR:' + e.message; }\
             JSON.stringify(r)",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"same":true,"docProto":"about:","winProto":"about:","docHref":"about:srcdoc","neverToString":"about:blank","blobProto":"blob:"}"#,
        "doc.location 与 win.location 同一对象 + protocol 解析面（about:/blob:）+ 未物化 toString 维持"
    );
}

/// M3-S2 子窗 location.hash 写侧（spec `#the-location-interface` hash setter——
/// 同文档片段导航：URL 更新 + 子会话条目 push + 异步 hashchange（UA 生成
/// isTrusted true）；'#' 前缀可省；同值 no-op 不派）。srcdoc_change_hash 前两步。
#[test]
fn m3s2_child_hash_write_updates_url_and_fires_hashchange() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/child-hash.html",
        "<html><body><iframe srcdoc='srcdoc document'></iframe></body></html>",
    );
    sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             globalThis.__r = { href0: null, sync: null, events: [], hist: null };\
             f.contentWindow.onhashchange = function (e) {\
               __r.events.push(e.oldURL + '>' + e.newURL + ':' + (e.isTrusted === true));\
             };\
             __r.href0 = f.contentWindow.location.href;\
             f.contentWindow.location.hash = '1';\
             __r.sync = f.contentWindow.location.href + '#' + f.contentWindow.location.hash;\
             __r.hist = f.contentWindow.history._entries.length;",
        )
        .unwrap();
    // 同步面：URL 槽即时更新（同文档导航无重载）。
    assert_eq!(
        sandbox.execute("globalThis.__r.href0").unwrap().value,
        "about:srcdoc",
        "srcdoc 子文档初始地址 about:srcdoc"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r.sync").unwrap().value,
        "about:srcdoc#1##1",
        "hash 写侧同步生效（href 带 #1、hash getter 返回 '#1'——spec 含前缀）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r.hist").unwrap().value,
        "2",
        "子会话条目 push（spec session history）"
    );
    // 异步 hashchange：微任务泵后恰一次。
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r.events)")
            .unwrap()
            .value,
        r#"["about:srcdoc>about:srcdoc#1:true"]"#,
        "hashchange 异步恰一次（UA 生成 isTrusted true）"
    );
    // 同值写 no-op：URL 不变、不派事件。
    sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             f.contentWindow.location.hash = '1';",
        )
        .unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox
            .execute(
                "var f = document.querySelector('iframe');\
                 JSON.stringify({ href: f.contentWindow.location.href, n: __r.events.length })"
            )
            .unwrap()
            .value,
        r#"{"href":"about:srcdoc#1","n":1}"#,
        "同 hash 写 no-op（不派 hashchange）"
    );
}

/// M3-S2 detached/iframe 文档 body.innerText 合成树近似（R3260 语义——合成树无
/// 渲染信息，innerText ≈ textContent；spec
/// https://html.spec.whatwg.org/#the-innertext-idl-attribute 渲染依赖步在合成树
/// 不可达）。WPT srcdoc-anchor / srcdoc-attribute-reset 经 body.innerText 断言。
#[test]
fn m3s2_child_body_inner_text() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/body-innertext.html",
        "<html><body><iframe srcdoc='srcdoc_text'></iframe></body></html>",
    );
    let out = sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             var d = f.contentDocument;\
             var r = { it: d.body.innerText, tc: d.body.textContent };\
             d.body.innerText = 'changed';\
             r.afterSet = d.body.textContent;\
             JSON.stringify(r)",
        )
        .unwrap()
        .value;
    assert_eq!(
        out,
        r#"{"it":"srcdoc_text","tc":"srcdoc_text","afterSet":"changed"}"#,
        "合成 body innerText 读 ≈ textContent、写落 textContent"
    );
    // srcdoc 移除 → 重处理回 about:blank 空文档，新 body.innerText ''（srcdoc-attribute-reset 尾段）。
    sandbox
        .execute("document.querySelector('iframe').removeAttribute('srcdoc');")
        .unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox
            .execute(
                "var f = document.querySelector('iframe');\
                 f.contentDocument.body.innerText"
            )
            .unwrap()
            .value,
        "",
        "srcdoc 移除成熟后新文档 body.innerText 为空串"
    );
}

/// M3-S4 子文档载入中自导航（spec
/// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element
/// ——pending 初始导航被载入中自导航取代：被取代文档不派元素 load，最终只派一次
/// load 且地址为取代后文档）。replace-before-load 族 waitForLoad 首断言面。
#[test]
fn m3s4_child_during_load_self_nav_single_load() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/browsers/browsing-the-web/navigating-across-documents/replace-before-load/probe.html",
        "<html><body></body></html>",
    );
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let log_fetch = log.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).cloned().unwrap_or_default();
            log_fetch.lock().unwrap().push(format!("fetch:{}", url));
            if url.contains("injector") {
                "__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1e\x1f<script>location.href = 'https://wpt.test/common/blank.html?repl';</script>".to_string()
            } else {
                "__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1e\x1f<p>t</p>".to_string()
            }
        }),
    );
    let log_js = log.clone();
    sandbox.register_callback(
        "__zw_report",
        Box::new(move |args| {
            log_js.lock().unwrap().push(args.first().cloned().unwrap_or_default());
            "".to_string()
        }),
    );
    sandbox
        .execute(
            "var iframe = document.createElement('iframe');\
             iframe.src = 'https://wpt.test/html/scratch/injector.html';\
             var n = 0;\
             iframe.addEventListener('load', function () {\
               n++;\
               __zw_report('load' + n + ':' + iframe.contentWindow.location.href);\
             });\
             document.body.appendChild(iframe);",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }
    sandbox
        .execute(
            "__zw_report('final:' + iframe.contentWindow.location.href + '#n=' + n +\
             '#hlen=' + iframe.contentWindow.history._entries.length + '#plen=' + history.length);",
        )
        .unwrap();
    let seq = log.lock().unwrap().clone();
    let loads: Vec<&String> = seq.iter().filter(|l| l.starts_with("load")).collect();
    assert_eq!(
        loads.len(),
        1,
        "被取代文档不派元素 load——恰一次（实际序列 {seq:?}）"
    );
    assert!(
        loads[0].ends_with("blank.html?repl"),
        "load 事件时点地址已是取代后文档（实际 {seq:?}）"
    );
    assert!(
        seq.last().is_some_and(|l| l == "final:https://wpt.test/common/blank.html?repl#n=1#hlen=1#plen=1"),
        "终态地址 + 计数 + 子局部历史折叠为 1 条（replace 不 push）+ 父窗长度不膨胀（实际 {seq:?}）"
    );
}

/// M3-S4 父窗 back 的 frame 局部回退（spec
/// https://html.spec.whatwg.org/multipage/nav-history-apis.html#joint-session-history
/// 联合遍历最小近似——顶层账本无更早条目时落最近活动 frame 的局部前一条目）。
/// replace-before-load checkSentinelIframe 断言面（ONE history.back() 必须把
/// 哨兵 iframe 驱回前一条目）：两段属性载入（src 更换 push 子局部历史）后父窗
/// back → frame 以 {history:true} 重载回首段地址 + 元素 load 派发 + 子局部
/// 历史弹出收缩；父窗 history.length 不膨胀。
#[test]
fn m3s4_parent_back_falls_back_to_frame_local_history() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/browsers/browsing-the-web/navigating-across-documents/replace-before-load/sentinel.html",
        "<html><body></body></html>",
    );
    let log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let log_fetch = log.clone();
    sandbox.register_callback(
        "__zw_fetch",
        Box::new(move |args| {
            let url = args.get(2).cloned().unwrap_or_default();
            log_fetch.lock().unwrap().push(format!("fetch:{}", url));
            format!("__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1e\x1f<p>{}</p>", url)
        }),
    );
    let log_js = log.clone();
    sandbox.register_callback(
        "__zw_report",
        Box::new(move |args| {
            log_js.lock().unwrap().push(args.first().cloned().unwrap_or_default());
            "".to_string()
        }),
    );
    sandbox
        .execute(
            "var iframe = document.createElement('iframe');\
             iframe.src = 'https://wpt.test/x/s-start.html';\
             var n = 0;\
             iframe.addEventListener('load', function () {\
               n++;\
               __zw_report('load' + n + ':' + iframe.contentWindow.location.href);\
             });\
             document.body.appendChild(iframe);",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }
    // 第二段属性载入：src 更换（载入后 push 子局部历史 → [s-start, s-end]）。
    sandbox
        .execute("document.querySelector('iframe').src = 'https://wpt.test/x/s-end.html';")
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }
    // 父窗 back：顶层账本无更早条目（history.length 1）→ frame 局部回退。
    sandbox
        .execute("__zw_report('pre:' + history.length + ':' + iframe.contentWindow.history._entries.length); history.back();")
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }
    sandbox
        .execute(
            "__zw_report('post:' + iframe.contentWindow.location.href + ':' +\
             iframe.contentWindow.history._entries.length + ':' + history.length);",
        )
        .unwrap();
    let seq = log.lock().unwrap().clone();
    assert!(
        seq.iter().any(|l| l == "pre:1:2"),
        "回退前父窗长度 1、子局部历史 2 条（实际序列 {seq:?}）"
    );
    let loads: Vec<&String> = seq.iter().filter(|l| l.starts_with("load")).collect();
    assert_eq!(
        loads.len(),
        3,
        "三段载入各派一次元素 load（s-start / s-end / back 重载）（实际序列 {seq:?}）"
    );
    assert!(
        seq.iter().any(|l| l == "load3:https://wpt.test/x/s-start.html"),
        "父窗 back 重载落子局部前一条目（实际序列 {seq:?}）"
    );
    assert!(
        seq.iter().any(|l| l == "post:https://wpt.test/x/s-start.html:1:1"),
        "回退后子局部历史弹出收缩为 1 条、父窗长度恒 1（实际序列 {seq:?}）"
    );
}

/// M3-S6 子窗局部 history 遍历（spec
/// https://html.spec.whatwg.org/multipage/nav-history-apis.html#session-history
/// ——子文档 pushState 后 back 落同文档条目 = 子窗异步 trusted popstate +
/// state 还原，父窗零 popstate；越界 back/forward = no-op（
/// iframe_sandbox_navigate_history_go_back/-forward 断言面：sandbox 子窗
/// 无局部可达条目时不触父窗账本）；子文档裸 `onpopstate =` 落子窗）。
#[test]
fn m3s6_child_local_history_traversal_pops_child_only() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/sandbox-nav-history.html",
        "<html><body><iframe src=\"support/child-history.html\"></iframe></body></html>",
    );
    sandbox
        .register_callback(
            "__zw_fetch",
            Box::new(|args| {
                let url = args.get(2).cloned().unwrap_or_default();
                let body = if url.contains("child-history") {
                    "<script>onpopstate = function (e) {\
                     parent.postMessage('popped:' + JSON.stringify(e.state) + ':' + (e.isTrusted === true), '*'); };\n\
                     onmessage = function (e) {\
                     if (e.data === 'go') { history.pushState({n: 1}, ''); history.back(); } }</script>"
                } else {
                    "<p>t</p>"
                };
                format!("__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1e\x1f{body}")
            }),
        );
    sandbox
        .execute(
            "var iframe = document.querySelector('iframe');\
             var w = iframe.contentWindow;\
             globalThis.__r = { got: [], parentPops: 0 };\
             window.addEventListener('message', function (e) { __r.got.push('' + e.data); });\
             window.onpopstate = function () { __r.parentPops++; };",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }
    // 子 pushState + back：子窗局部同文档遍历——子窗收 trusted popstate（state
    // 还原 {n:1}），父窗零 popstate。
    sandbox.execute("w.postMessage('go', '*');").unwrap();
    for _ in 0..10 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r.got)")
            .unwrap()
            .value,
        r#"["popped:null:true"]"#,
        "子窗局部 back 落初始条目 = 子窗 trusted popstate（e.state 为目标条目 null）"
    );
    assert_eq!(
        sandbox
            .execute("globalThis.__r.parentPops + '/' + history.state")
            .unwrap()
            .value,
        "0/null",
        "父窗零 popstate、父 history.state 不受子遍历影响"
    );
    assert_eq!(
        sandbox
            .execute("w.history.state + '/' + w.history.length")
            .unwrap()
            .value,
        "null/2",
        "子窗 back 后 state 落初始条目（null）、局部长度 2"
    );
    // forward 重入 pushState 条目（state {n:1}）；再 forward 越界 no-op；back 仍在界内。
    sandbox.execute("w.history.forward();").unwrap();
    for _ in 0..10 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r.got)")
            .unwrap()
            .value,
        r#"["popped:null:true","popped:{\"n\":1}:true"]"#,
        "子窗 forward 重入 pushState 条目（e.state = {{n:1}}）"
    );
    assert_eq!(
        sandbox
            .execute("JSON.stringify(w.history.state)")
            .unwrap()
            .value,
        r#"{"n":1}"#,
        "遍历后子窗 history.state 随游标还原"
    );
    // 越界 forward（游标已顶）= no-op（无第三次 popstate）。
    sandbox.execute("w.history.forward();").unwrap();
    for _ in 0..10 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r.got)")
            .unwrap()
            .value,
        r#"["popped:null:true","popped:{\"n\":1}:true"]"#,
        "越界 forward no-op（游标已顶，无局部可达条目）"
    );
}

/// M3-S6 parser 期 src iframe 自主载入 + 元素 load 双断言面（spec
/// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-iframe-element
/// ——解析期带的 iframe 无人读 contentWindow 也须取资源并派 trusted 元素 load：
/// `<iframe src="…" onload="doNavigation()">` 内联处理器与 onload property 双通道，
/// iframe_sandbox_navigate_history_go_back/-forward 载体）。loading=lazy 不补启。
#[test]
fn m3s6_parser_src_iframe_autoload_fires_element_load() {
    use std::sync::{Arc, Mutex};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/parser-iframe-load.html",
        "<html><body>\
         <iframe id=\"a\" src=\"support/child.html\" onload=\"globalThis.__attrLoad = (globalThis.__attrLoad || 0) + 1\"></iframe>\
         <iframe id=\"b\" src=\"support/lazy.html\" loading=\"lazy\"></iframe>\
         <iframe id=\"c\" src=\"support/child.html\"></iframe>\
         <iframe id=\"d\" src=\"support/scripted-child.html\" sandbox=\"allow-same-origin\"></iframe>\
         </body></html>",
    );
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetch_log = fetches.clone();
    sandbox
        .register_callback(
            "__zw_fetch",
            Box::new(move |args| {
                let url = args.get(2).cloned().unwrap_or_default();
                fetch_log.lock().unwrap().push(url.clone());
                let body = if url.contains("scripted-child.html") {
                    "<p>s</p><script>parent.postMessage('ran', '*');</script>"
                } else {
                    "<p>child</p>"
                };
                format!("__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1e\x1f{body}")
            }),
        );
    sandbox
        .execute(
            "globalThis.__propLoad = 0;\
             globalThis.__msgLog = [];\
             window.addEventListener('message', function (e) { globalThis.__msgLog.push('' + e.data); });\
             var c = document.querySelector('#c');\
             c.onload = function (e) { globalThis.__propLoad += (e.isTrusted === true) ? 1 : 0; };",
        )
        .unwrap();
    // window load 派发触发补启扫描（part06 load 分支）；此前零 contentWindow 读。
    sandbox
        .execute("globalThis.dispatchEvent(new Event('load'));")
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox.execute("JSON.stringify(globalThis.__msgLog)").unwrap().value,
        "[]",
        "sandbox 无 allow-scripts：子文档脚本禁跑（spec attr-iframe-sandbox——\
         defer 通道 __zwRunInlineScripts 与 _zwRunIframeScripts 同门，\
         iframe_sandbox_allow_script 首消息时序断言面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__attrLoad").unwrap().value,
        "1",
        "parser iframe 内联 onload 属性处理器触发（元素 load 派发面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__propLoad").unwrap().value,
        "1",
        "parser iframe onload property 触发（IDL handler 面，trusted）"
    );
    let fetched = fetches.lock().unwrap().join("|");
    assert!(
        fetched.contains("child.html"),
        "子文档 fetch 已自发结算（无人预读 contentWindow 也自主载入）：{fetched}"
    );
    assert!(
        !fetched.contains("lazy.html"),
        "loading=lazy 不补启（spec 延迟至接近视口）：{fetched}"
    );
}

/// M3-S7 子文档嵌套 iframe 自主载入 + 子文档合成树 contentDocument 面（spec
/// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#process-the-iframe-attributes
/// 递归步 + #attr-iframe-loading——lazy 仅 script-enabled 文档适用）。三层断言链
/// 对齐 WPT iframe-loading-lazy-in-script-disabled-iframe：sandbox 无 allow-scripts
/// 文档的 lazy 子 iframe 仍加载；script-enabled 链 lazy 维持惰性；无 loading 嵌套
/// 常载；跨层 src 按所在文档 URL 解析。
#[test]
fn m3s7_nested_iframe_loads_and_content_document_face() {
    use std::sync::{Arc, Mutex};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/m3s7.html",
        "<html><body>\
         <iframe id=\"scriptless\" sandbox=\"allow-same-origin\" src=\"support/lazy-inner.html\"></iframe>\
         <iframe id=\"eager\" src=\"support/lazy-inner.html\"></iframe>\
         <iframe id=\"plain\" src=\"support/plain-inner.html\"></iframe>\
         </body></html>",
    );
    let fetches: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let fetch_log = fetches.clone();
    sandbox
        .register_callback(
            "__zw_fetch",
            Box::new(move |args| {
                let url = args.get(2).cloned().unwrap_or_default();
                fetch_log.lock().unwrap().push(url.clone());
                let body = if url.contains("lazy-inner.html") {
                    "<iframe id=\"sub\" loading=\"lazy\" src=\"subframe.html\"></iframe>"
                } else if url.contains("plain-inner.html") {
                    "<iframe id=\"sub\" src=\"subframe.html\"></iframe>"
                } else {
                    "<!DOCTYPE html><body>\
                     <p>Subframe</p>\
                     </body>"
                };
                format!("__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1e\x1f{body}")
            }),
        );
    // window load 派发触发主文档补启（S6）；子文档嵌套链随 finish 管线同步成立。
    sandbox
        .execute("globalThis.dispatchEvent(new Event('load'));")
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }
    sandbox
        .execute(
            "var s = document.querySelector('#scriptless');\
             var si = s.contentDocument.querySelector('iframe');\
             globalThis.__r = {\
               sub: si.contentDocument.body.textContent.trim(),\
               eager: document.querySelector('#eager').contentDocument.querySelector('iframe').contentDocument,\
               plain: document.querySelector('#plain').contentDocument.querySelector('iframe').contentDocument.body.textContent.trim()\
             };",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r)")
            .unwrap()
            .value,
        r#"{"sub":"Subframe","eager":null,"plain":"Subframe"}"#,
        "三层链：script-disabled 文档 lazy 不豁免（子 iframe 已载、body 文本可达）\
         + script-enabled 链 lazy 维持惰性（contentDocument null）+ 无 loading 常载"
    );
    let fetched = fetches.lock().unwrap().join("|");
    assert!(
        fetched.contains("subframe.html"),
        "嵌套子 iframe fetch 已结算（跨层 src 按所在文档 URL 解析）：{fetched}"
    );
    assert_eq!(
        fetched.matches("subframe.html").count(),
        2,
        "subframe 恰两载（scriptless 链 lazy 照载 + plain 链常载；eager 链 lazy 未载）"
    );
}

/// M3-S5 父→子 postMessage 跨窗口投递 + 子文档 onmessage（spec
/// https://html.spec.whatwg.org/multipage/web-messaging.html#dom-window-postmessage
/// ——targetOrigin 校验按目标窗 origin，'*' 放行；子文档裸 onmessage 赋值落
/// 子窗、消息经子窗 onmessage 直调派发；子回程 parent.postMessage 走父窗
/// self 投递既有面）。iframe_sandbox_block_modals 1-4 断言面（modal 桩即时
/// 返回 undefined/false/null + 回程值断言）。
#[test]
fn m3s5_parent_to_child_postmessage_roundtrip() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/sandbox-block-modals.html",
        "<html><body><iframe src=\"support/child-modals.html\"></iframe></body></html>",
    );
    sandbox
        .register_callback(
            "__zw_fetch",
            Box::new(|args| {
                let url = args.get(2).cloned().unwrap_or_default();
                let body = if url.contains("child-modals") {
                    "<script>function openModal(name){switch(name){case 'alert':return alert('M');\
                     case 'confirm':return confirm('M?');case 'prompt':return prompt('M:', 'D');\
                     case 'print':return print();}}\n\
                     onmessage = function(e){ parent.postMessage(openModal(e.data), '*'); }</script>"
                } else {
                    "<p>t</p>"
                };
                format!("__zwfr:200\x1fOK\x1fcontent-type\x1etext/html\x1e\x1f{body}")
            }),
        );
    sandbox
        .execute(
            "var iframe = document.querySelector('iframe');\
             globalThis.__r = { got: [] };\
             window.addEventListener('message', function (e) { __r.got.push('' + JSON.stringify(e.data)); });",
        )
        .unwrap();
    for _ in 0..8 {
        sandbox.execute("0").unwrap();
    }
    // 四段投递：alert/confirm/prompt/print 各一（子窗 onmessage → modal 桩 → 回程）。
    sandbox
        .execute(
            "var w = iframe.contentWindow;\
             w.postMessage('alert', '*');\
             w.postMessage('confirm', '*');\
             w.postMessage('prompt', '*');\
             w.postMessage('print', '*');",
        )
        .unwrap();
    for _ in 0..12 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r.got)")
            .unwrap()
            .value,
        r#"["undefined","false","null","undefined"]"#,
        "回程序列 = modal 桩返回值（alert undefined / confirm false / prompt null / print undefined）"
    );
    // 子文档裸 onmessage 落子窗（with(window) + 槽位——非父窗 globalThis 误注册）。
    assert_eq!(
        sandbox
            .execute(
                "JSON.stringify({ onChild: typeof iframe.contentWindow.onmessage,\
                 onParent: typeof window.onmessage })"
            )
            .unwrap()
            .value,
        r#"{"onChild":"function","onParent":"object"}"#,
        "onmessage 绑定子窗、父窗零注册（getter 返 null——typeof 'object'）"
    );
}

/// M3-S3 联合 iframe session history（spec
/// https://html.spec.whatwg.org/multipage/nav-history-apis.html#joint-session-history
/// ——父窗 history back/forward 覆盖子同文档条目；父窗 `_hist_entries`/
/// history.length 不膨胀——联合序独立）。srcdoc_change_hash step 3/4 断言面：子
/// 连写 #1/#2 后父 back/forward 落子条目、子窗收 trusted hashchange，父窗零
/// hashchange（顶层 URL 未变）。
#[test]
fn m3s3_parent_back_forward_routes_child_hash_entries() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = m3s2_sandbox(
        "https://wpt.test/html/semantics/embedded-content/the-iframe-element/joint-hash.html",
        "<html><body><iframe srcdoc='joint'></iframe></body></html>",
    );
    sandbox
        .execute(
            "var f = document.querySelector('iframe');\
             var w = f.contentWindow;\
             globalThis.__r = { parent: [], child: [], len: history.length };\
             w.onhashchange = function (e) { __r.child.push(e.oldURL + '>' + e.newURL + ':' + (e.isTrusted === true)); };\
             window.onhashchange = function (e) { __r.parent.push(e.oldURL + '>' + e.newURL); };\
             w.location.hash = '1';\
             w.location.hash = '2';\
             __r.lenAfter = history.length;",
        )
        .unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    // 子窗两连写各派一次（父窗零派——顶层 URL 未变）。
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r.child)")
            .unwrap()
            .value,
        r#"["about:srcdoc>about:srcdoc#1:true","about:srcdoc#1>about:srcdoc#2:true"]"#,
        "子写侧两连写各自异步派 trusted hashchange"
    );
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r.parent)")
            .unwrap()
            .value,
        "[]",
        "子 hash 写不派父窗 hashchange（顶层 URL 未变）"
    );
    // 联合播种不膨胀父窗长度（M2 面维持：恒 1——Navigation/history 顶层账本不变）。
    assert_eq!(
        sandbox
            .execute("globalThis.__r.len + '/' + globalThis.__r.lenAfter + '/' + history.length")
            .unwrap()
            .value,
        "1/1/1",
        "联合播种前后父窗 history.length 恒 1（顶层账本不膨胀）"
    );
    // 父窗 back → 落子条目 #1（子窗 href + trusted hashchange）；forward → #2。
    sandbox.execute("history.back();").unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox
            .execute("f.contentWindow.location.href")
            .unwrap()
            .value,
        "about:srcdoc#1",
        "父窗 back 落子同文档条目（联合序 frame 标记）"
    );
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r.child[2])")
            .unwrap()
            .value,
        r#""about:srcdoc#2>about:srcdoc#1:true""#,
        "父 back 驱动子窗 trusted hashchange #2→#1"
    );
    sandbox.execute("history.forward();").unwrap();
    for _ in 0..4 {
        sandbox.execute("0").unwrap();
    }
    assert_eq!(
        sandbox
            .execute("f.contentWindow.location.href")
            .unwrap()
            .value,
        "about:srcdoc#2",
        "父窗 forward 落子同文档条目"
    );
    assert_eq!(
        sandbox
            .execute("JSON.stringify(globalThis.__r.child[3])")
            .unwrap()
            .value,
        r#""about:srcdoc#1>about:srcdoc#2:true""#,
        "父 forward 驱动子窗 trusted hashchange #1→#2"
    );
}
