// part63 — navigation-compat M3-S2 切片：子文档 location 合流 + hash 写侧 + 合成树
// innerText 单测。契约：docs/goal/navigation-compat.md DC-3（M3 最小面续——
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
