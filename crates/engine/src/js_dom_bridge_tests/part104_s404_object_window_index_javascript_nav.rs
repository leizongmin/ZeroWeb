// part104 — html-semantics M2 切片 404（2026-11-11）：`window[N]` 索引访问 +
// object javascript: URL 导航面。
//
// object-javascript-url.html 文件级 Timeout 的三面根因修复钉（s227 同族勘根先行：
// /resources/blank.html 语料资产缺席 → 404 → error 代替 load → await onload 恒挂；
// 资产已补，剩余两面为引擎通道）：
// - `window[N]` 索引属性（WindowProperties 异对象层，own accessor 0..15——
//   `window[0].location.href` 断言入口；活跃集合与 _zwWindowLengthCount 同口径）；
// - `_zwLoadObjectResource` javascript: data 属性臂（子测1「should do nothing」——
//   无 load，error/fallback 语义，不 fetch）；
// - `_zwObjectNavigateEntry` javascript: 臂（#javascript-protocol：目标上下文求值，
//   完成值字符串 → 重物化 + fired 复位 + load 重派；非字符串/抛错 → 静默中止；
//   opaque path percent-decode 后作脚本源）。
// https://html.spec.whatwg.org/multipage/window-object.html#windowproxy
// https://html.spec.whatwg.org/multipage/browsing-the-web.html#javascript-protocol

/// 桥测读回辅具（文件级自由函数——闭包持 `&mut sandbox` 跨 execute 触发 E0499）。
fn s404_read(sandbox: &mut zero_script_sandbox::V8Sandbox, id: &str) -> String {
    sandbox
        .execute(&format!("globalThis.__s404{id}"))
        .unwrap()
        .value
}

/// 钉①：`window[N]` 索引访问——iframe+object 活跃集合按物化序索引（树序近似）+
/// descriptor 面（own accessor、enumerable:false 防 for-in/keys 漏出、
/// configurable:true）+ 越界 undefined + 移除动态重排。断言翻转面：索引层误落
/// host fetch（零取回）；移除后索引不重排（window[0] 残留旧 win）；descriptor
/// enumerable:true（Object.keys 漏出数值键——for-in 污染面）。
#[test]
fn window_indexed_access_faces_s404() {
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
        Arc::new(Mutex::new("https://wpt.test/s404/x.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
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
            "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Blank</body></html>".to_string()
        }),
    );

    // 第一段：iframe（contentWindow 同步物化——s400 L1 口径）+ object（data: text/html，
    // 插入钩子 drain 物化）。索引物化序 = iframe 先、object 后。
    sandbox
        .execute(
            "(function () {\n\
             var ifr = document.createElement('iframe');\n\
             ifr.id = 's404ifr';\n\
             ifr.src = '/common/blank.html';\n\
             document.body.appendChild(ifr);\n\
             globalThis.__s404i0 = String(ifr.contentWindow !== null);\n\
             var o = document.createElement('object');\n\
             o.id = 's404obj';\n\
             o.setAttribute('data', 'data:text/html,<p>obj</p>');\n\
             document.body.appendChild(o);\n\
             globalThis.__s404obj = o;\n\
             })();",
        )
        .unwrap();

    // 第二段：drain 后索引面 + descriptor + 越界。
    sandbox
        .execute(
            "(function () {\n\
             var ifr = document.getElementById('s404ifr');\n\
             var obj = globalThis.__s404obj;\n\
             globalThis.__s404n = [\n\
               String(globalThis.length),\n\
               String(window[0] === ifr.contentWindow),\n\
               String(window[1] === obj.contentWindow),\n\
               String(window[2] === undefined),\n\
               String(window[15] === undefined),\n\
               String(typeof window[0]),\n\
               String(getOwnPropertyKind())\n\
             ].join('|');\n\
             function getOwnPropertyKind() {\n\
               var d = Object.getOwnPropertyDescriptor(globalThis, '0');\n\
               if (!d) return 'missing';\n\
               return 'get:' + String(!!d.get) + '/enum:' + String(d.enumerable) + '/cfg:' + String(d.configurable) + '/keys:' + String(Object.keys(globalThis).indexOf('0') === -1);\n\
             }\n\
             })();",
        )
        .unwrap();
    assert_eq!(s404_read(&mut sandbox, "i0"), "true", "(I0) iframe contentWindow 同步物化（索引前置）");
    assert_eq!(
        s404_read(&mut sandbox, "n"),
        "2|true|true|true|true|object|get:true/enum:false/cfg:true/keys:true",
        "(I1) window[N] 索引：length=2 → [0]=iframe（物化序≈树序近似）→ [1]=object → 越界 undefined → descriptor own accessor（enumerable:false 不漏 keys、configurable:true 可被页面数值全局遮蔽）实得"
    );

    // 第三段：移除 iframe（destroyed 印记）→ object 升 [0]；再移除 object → 越界。
    sandbox
        .execute(
            "(function () {\n\
             document.body.removeChild(document.getElementById('s404ifr'));\n\
             globalThis.__s404r1 = String(window[0] === globalThis.__s404obj.contentWindow) + '/' + String(globalThis.length);\n\
             document.body.removeChild(globalThis.__s404obj);\n\
             globalThis.__s404r2 = String(window[0] === undefined) + '/' + String(globalThis.length);\n\
             })();",
        )
        .unwrap();
    assert_eq!(
        s404_read(&mut sandbox, "r1"),
        "true/1",
        "(I2) 移除 iframe 后索引动态重排：object 升 [0]，length 递减"
    );
    assert_eq!(
        s404_read(&mut sandbox, "r2"),
        "true/0",
        "(I3) 全移除后越界 undefined + length 0"
    );
    assert!(
        !fetch_log
            .lock()
            .unwrap()
            .iter()
            .any(|u| u.contains("javascript:")),
        "索引/length 读面零 host fetch（活跃集合纯缓存口径）实得 {:?}",
        *fetch_log.lock().unwrap()
    );
}

/// 钉②③④：javascript: 三面（object-javascript-url 三子测镜像）——
/// ①data 属性 javascript:"foo" → 无 load（error 臂，零 fetch）；②已载 object
/// `window[0].location.href='javascript:"<b>x</b>"'` → 完成值字符串重物化 +
/// load 重派 + 内容落 doc + URL 槽不动；③`javascript:1` 非字符串完成值 → 静默
/// 中止（无 load、doc 不动）。断言翻转面：data 臂误 fetch（log 入 javascript:）；
/// 求值通道无完成值（load 不派/内容空）；percent-decode 缺（%22%3Cb%3E 序列入
/// doc）；非字符串误清文档。
#[test]
fn object_javascript_url_faces_s404() {
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
        Arc::new(Mutex::new("https://wpt.test/s404b/page.html".to_string()));
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
            if url.contains("/s404b/blank.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Blank</body></html>".to_string();
            }
            "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/html\u{1f}".to_string()
        }),
    );

    // 第一段：obj1（data=javascript:"foo"——子测1镜像）+ obj2（data=blank.html——
    // insertObjectNavigable 镜像，onload 计数）。drain 后 obj2 初载 load1。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s404b = [];\n\
             globalThis.__s404bev1 = [];\n\
             globalThis.__s404bloads = 0;\n\
             var o1 = document.createElement('object');\n\
             o1.id = 's404obj1';\n\
             o1.onload = function () { globalThis.__s404bev1.push('load'); };\n\
             o1.onerror = function () { globalThis.__s404bev1.push('error'); };\n\
             o1.setAttribute('data', 'javascript:\"foo\"');\n\
             document.body.appendChild(o1);\n\
             var o2 = document.createElement('object');\n\
             o2.id = 's404obj2';\n\
             o2.onload = function () { globalThis.__s404bloads++; };\n\
             o2.setAttribute('data', '/s404b/blank.html');\n\
             document.body.appendChild(o2);\n\
             globalThis.__s404bo2 = o2;\n\
             })();",
        )
        .unwrap();

    // 第二段：子测1面（obj1 error 无 load）+ 索引面 + 子测2导航（javascript:
    // 完成值字符串——href setter 经 URL 序列化 %22%3Cb%3E 编码，求值通道
    // percent-decode 还原）。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s404bp1 = globalThis.__s404bev1.join(',') || 'none';\n\
             globalThis.__s404bp2 = String(globalThis.length) + '/' + String(window[0] === globalThis.__s404bo2.contentWindow);\n\
             window[0].location.href = 'javascript:\"<b>x</b>\"';\n\
             })();",
        )
        .unwrap();

    // 第三段：子测2断言（load2 + 内容 + URL 槽）+ 子测3导航（非字符串完成值）。
    sandbox
        .execute(
            "(function () {\n\
             var o2 = globalThis.__s404bo2;\n\
             var doc = o2.contentDocument;\n\
             globalThis.__s404bp3 = [\n\
               String(globalThis.__s404bloads),\n\
               String(doc.body.textContent),\n\
               String(doc.location.href.indexOf('blank.html') !== -1)\n\
             ].join('|');\n\
             window[0].location.href = 'javascript:1';\n\
             })();",
        )
        .unwrap();

    // 第四段：子测3断言（无第三 load、doc 不动）+ 索引 live 解析保持。
    // FIXME：跨导航 WindowProxy identity 不钉——重物化建新 win（s403 同型，
    // named access 重 morph；spec 同一 navigable 保 WindowProxy identity，shim
    // 近似，corpus 断言面均为 fresh 读不触该面）。
    sandbox
        .execute(
            "(function () {\n\
             var o2 = globalThis.__s404bo2;\n\
             globalThis.__s404bp4 = [\n\
               String(globalThis.__s404bloads),\n\
               String(o2.contentDocument.body.textContent),\n\
               String(window[0] === o2.contentWindow)\n\
             ].join('|');\n\
             })();",
        )
        .unwrap();

    assert_eq!(
        s404_read(&mut sandbox, "bp1"),
        "error",
        "(J1) data=javascript:\"foo\" → 无 load（error 臂 fallback——子测1「should do nothing」）"
    );
    assert_eq!(
        s404_read(&mut sandbox, "bp2"),
        "1/true",
        "(J2) obj1 error 态不计 length；window[0] = obj2 contentWindow（初载 load1 后）"
    );
    assert_eq!(
        s404_read(&mut sandbox, "bp3"),
        "2|x|true",
        "(J3) javascript:\"<b>x</b>\" → 完成值字符串重物化：load 重派（1→2）+ 内容落 doc（percent-decode 还原）+ URL 槽不动（停 blank.html）"
    );
    assert_eq!(
        s404_read(&mut sandbox, "bp4"),
        "2|x|true",
        "(J4) javascript:1 非字符串完成值 → 静默中止：无第三 load、doc 不动、window[0] identity 保持"
    );
    let log = fetch_log.lock().unwrap();
    assert_eq!(
        log.iter().filter(|u| u.contains("blank.html")).count(),
        1,
        "obj2 初载恰一取回（javascript: 臂零 fetch——导航不走网络）实得 {:?}",
        *log
    );
    assert!(
        !log.iter().any(|u| u.contains("javascript:")),
        "javascript: URL 不落 host fetch（求值通道 shim 内闭环）实得 {:?}",
        *log
    );
}
