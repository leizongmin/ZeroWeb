// part102 — html-semantics M2 切片 400（2026-11-11）：object scheme 取回面 +
// window.length。
//
// s399 接通动态加载链后新显形 6F 的两面根因修复钉：
// - data:/blob:/bare-origin scheme 取回（object-allowed-schemas 4F 根因）：
//   data: 经 _zwDataURLProcessor 解码（fetch() 通路同源；text/html → 嵌套上下文，
//   其余 → 替换内容 success 无 navigable）；blob: 经 _zwBlobStore 命中（同源门，
//   fetch() scheme-fetch 同语义；miss/跨源 → error）；bare-origin http(s)（路径空
//   或仅 /）→ 合成空 text/html 嵌套上下文（FIXME 服务端目录语义）；
// - window.length（object-fallback-failed-cross-origin-navigation 2F 根因）：
//   LegacyUnforgeable getter——iframe（_iframeDocCache win 有且未 destroy）+
//   object（_zwObjectNavCache win 有且元素连接）计数；error 态/移除不计。
// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element
// https://html.spec.whatwg.org/multipage/window-object.html#dom-length

/// 桥测读回辅具（文件级自由函数——闭包持 `&mut sandbox` 跨 execute 触发 E0499）。
fn s400_read(sandbox: &mut zero_script_sandbox::V8Sandbox, id: &str) -> String {
    sandbox
        .execute(&format!("globalThis.__s400{id}"))
        .unwrap()
        .value
}

/// 钉①②：data:/blob: 七面——text/plain 替换内容 load + null 保持；text/html 嵌套
/// 上下文（doc.location.href 印记）；无效 data: error；blob 同源 text/html 物化；
/// blob text/plain null；跨源 blob error；revoked blob error。断言翻转：data:/blob:
/// 误落 host fetch（fetch log 非空）；text/html 未物化（doc null）；error/load 错派
///（events 序列错）。
#[test]
fn object_scheme_fetch_faces_s400() {
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
        Arc::new(Mutex::new("https://wpt.test/s400/x.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // host fetch 调用日志——data:/blob: 七面应全走 shim 内解码，log 恒空。
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

    // 第一段：七面布置（data: 四 + blob: 三），事件经插入钩子 drain 后可读。
    sandbox
        .execute(
            "(function () {\n\
             function mk(id, url) {\n\
               var o = document.createElement('object');\n\
               o.id = id;\n\
               var r = { events: [] };\n\
               o.onload = function () { r.events.push('load'); };\n\
               o.onerror = function () { r.events.push('error'); };\n\
               o.setAttribute('data', url);\n\
               document.body.appendChild(o);\n\
               return r;\n\
             }\n\
             // A：data: text/plain（无 MIME 缺省 US-ASCII）——替换内容 success。\n\
             globalThis.__s400rA = mk('s400a', 'data:,Hello%2C%20World%21');\n\
             // B：data: text/html——嵌套上下文物化。\n\
             globalThis.__s400rB = mk('s400b', 'data:text/html,<p>s400</p>');\n\
             // C：无效 data:（无 ','）——RFC 2397 failure → error。\n\
             globalThis.__s400rC = mk('s400c', 'data:s400-invalid');\n\
             // D：blob: text/html（createObjectURL 注册表命中）。\n\
             globalThis.__s400bD = URL.createObjectURL(new Blob(['<p>blob-doc</p>'], { type: 'text/html' }));\n\
             globalThis.__s400rD = mk('s400d', globalThis.__s400bD);\n\
             // E：blob: text/plain——替换内容 success 无 navigable。\n\
             globalThis.__s400rE = mk('s400e', URL.createObjectURL(new Blob(['plain'], { type: 'text/plain' })));\n\
             // F：跨源 blob URL（origin 门）→ error。\n\
             globalThis.__s400rF = mk('s400f', 'blob:https://elsewhere.test/s400-f');\n\
             // G：revoked blob（store 清理）→ error。\n\
             var bG = URL.createObjectURL(new Blob(['gone'], { type: 'text/html' }));\n\
             URL.revokeObjectURL(bG);\n\
             globalThis.__s400rG = mk('s400g', bG);\n\
             })();",
        )
        .unwrap();
    sandbox
        .execute(
            "(function () {\n\
             function ev(r) { return r.events.join(','); }\n\
             globalThis.__s400fA = ev(globalThis.__s400rA);\n\
             globalThis.__s400fB = ev(globalThis.__s400rB);\n\
             globalThis.__s400fC = ev(globalThis.__s400rC);\n\
             globalThis.__s400fD = ev(globalThis.__s400rD);\n\
             globalThis.__s400fE = ev(globalThis.__s400rE);\n\
             globalThis.__s400fF = ev(globalThis.__s400rF);\n\
             globalThis.__s400fG = ev(globalThis.__s400rG);\n\
             function face(id) {\n\
               var o = document.getElementById(id);\n\
               var cw = null, cd = null, href = '';\n\
               try { cw = o.contentWindow; } catch (e1) { cw = 'ERR'; }\n\
               try { cd = o.contentDocument; } catch (e2) { cd = 'ERR'; }\n\
               try { href = cd ? cd.location.href : ''; } catch (e3) { href = 'ERR:' + e3.message; }\n\
               return String(cw === null) + '/' + String(cd === null) + '/' + href;\n\
             }\n\
             globalThis.__s400nA = face('s400a');\n\
             globalThis.__s400nB = face('s400b');\n\
             globalThis.__s400nD = face('s400d');\n\
             globalThis.__s400nE = face('s400e');\n\
             })();",
        )
        .unwrap();
    assert_eq!(s400_read(&mut sandbox, "fA"), "load", "(A) data: text/plain → 替换内容 success（load，allowed-schemas data: 面口径）");
    assert_eq!(s400_read(&mut sandbox, "fB"), "load", "(B) data: text/html → load");
    assert_eq!(s400_read(&mut sandbox, "fC"), "error", "(C) 无效 data:（无 ','）→ RFC 2397 failure → error");
    assert_eq!(s400_read(&mut sandbox, "fD"), "load", "(D) blob: text/html → load");
    assert_eq!(s400_read(&mut sandbox, "fE"), "load", "(E) blob: text/plain → 替换内容 success");
    assert_eq!(s400_read(&mut sandbox, "fF"), "error", "(F) 跨源 blob URL → origin 门 → error");
    assert_eq!(s400_read(&mut sandbox, "fG"), "error", "(G) revoked blob URL → store miss → error");
    assert_eq!(
        s400_read(&mut sandbox, "nA"),
        "true/true/",
        "(A2) text/plain 无嵌套 navigable——contentWindow/contentDocument 保 null"
    );
    assert_eq!(
        s400_read(&mut sandbox, "nB"),
        "false/false/data:text/html,<p>s400</p>",
        "(B2) data: text/html 嵌套上下文——win+doc 物化 + location.href = data URL 印记"
    );
    assert_eq!(
        s400_read(&mut sandbox, "nD"),
        format!("false/false/{}", s400_blob_url(&mut sandbox)),
        "(D2) blob: text/html 嵌套上下文——win+doc 物化 + location.href = blob URL 印记"
    );
    assert_eq!(
        s400_read(&mut sandbox, "nE"),
        "true/true/",
        "(E2) blob: text/plain 无嵌套 navigable"
    );
    assert!(
        fetch_log.lock().unwrap().is_empty(),
        "data:/blob: 七面全走 shim 内解码——host fetch 零调用"
    );
}

/// 钉② blob: URL 读回辅具——createObjectURL 产物（uuid 随机，经 globalThis 桥读）。
fn s400_blob_url(sandbox: &mut zero_script_sandbox::V8Sandbox) -> String {
    sandbox.execute("globalThis.__s400bD").unwrap().value
}

/// 钉③：window.length LegacyUnforgeable getter 计数面 + bare-origin 合成——
/// 0（初始）→ 1（iframe win 物化）→ 2（object data:text/html）→ 3（bare-origin
/// object 合成 doc）→ error 态 object 不计 → 移除 object 递减 → 移除 iframe
///（destroyed 印记）递减；descriptor enumerable/configurable:false 钉。
#[test]
fn window_length_count_faces_s400() {
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
        Arc::new(Mutex::new("https://wpt.test/s400/y.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let fetch_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let fetch_log_c = fetch_log.clone();
    Sandbox::register_callback(
        &mut sandbox,
        "__zw_fetch",
        Box::new(move |args: &[String]| -> String {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            fetch_log_c.lock().unwrap().push(url.to_string());
            if url.contains("missing.html") {
                return "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/plain\u{1f}".to_string();
            }
            "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Blank</body></html>".to_string()
        }),
    );

    // 第一段：L0 初始 + L1 iframe（读 contentWindow 物化 R115 entry）+ L2/L3/L4
    // 三 object（data:text-html / bare-origin / 404）。插入钩子 _defer——win 物化
    // 在 execute 末 drain，计数第二段读。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s400L = [];\n\
             function len() { return globalThis.length; }\n\
             globalThis.__s400L.push(len());\n\
             var ifr = document.createElement('iframe');\n\
             ifr.id = 's400ifr';\n\
             ifr.src = '/common/blank.html';\n\
             document.body.appendChild(ifr);\n\
             globalThis.__s400ifrCW = String(ifr.contentWindow !== null);\n\
             globalThis.__s400L.push(len());\n\
             function mkObj(id, url) {\n\
               var o = document.createElement('object');\n\
               o.id = id;\n\
               o.setAttribute('data', url);\n\
               document.body.appendChild(o);\n\
               return o;\n\
             }\n\
             mkObj('s400objH', 'data:text/html,<p>x</p>');\n\
             mkObj('s400objB', 'https://wpt.test/');\n\
             mkObj('s400objE', '/missing.html');\n\
             })();",
        )
        .unwrap();
    // 第二段：drain 后计数 + bare-origin doc 面 + descriptor。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s400L.push(globalThis.length);\n\
             var ob = document.getElementById('s400objB');\n\
             var bd = null;\n\
             try { bd = ob.contentDocument; } catch (e1) { bd = 'ERR'; }\n\
             globalThis.__s400bare = String(bd && bd !== null ? bd.location.href : String(bd));\n\
             var oe = document.getElementById('s400objE');\n\
             globalThis.__s400errFace = String(oe.contentWindow === null) + '/' + String(oe.contentDocument === null);\n\
             var d = Object.getOwnPropertyDescriptor(globalThis, 'length');\n\
             globalThis.__s400desc = String(typeof d.get === 'function') + '/' + String(d.enumerable) + '/' + String(d.configurable) + '/' + String(!('writable' in d) || d.writable === undefined);\n\
             globalThis.__s400in = String('length' in globalThis);\n\
             })();",
        )
        .unwrap();
    assert_eq!(
        s400_read(&mut sandbox, "ifrCW"),
        "true",
        "(L1) iframe contentWindow 物化（R115 entry win 在位前提）"
    );
    assert_eq!(
        s400_read(&mut sandbox, "L"),
        "0,1,3",
        "(L) 计数相位：初始 0 → iframe win 同步物化 1 → drain 后 3（text/html object + bare-origin object 计入，404 error 不计）"
    );
    assert_eq!(s400_read(&mut sandbox, "bare"), "https://wpt.test/", "(L2) bare-origin 合成空 text/html——doc.location.href = 原点 URL");
    assert_eq!(s400_read(&mut sandbox, "errFace"), "true/true", "(L3) 404 error 态无嵌套 navigable——不计入 length");
    assert_eq!(
        s400_read(&mut sandbox, "desc"),
        "true/true/false/true",
        "(L4) LegacyUnforgeable descriptor：getter + enumerable:true + configurable:false + 访问器无 writable"
    );
    assert_eq!(s400_read(&mut sandbox, "in"), "true", "(L5) 'length' in window");
    // 第三段：移除 data:text/html object + iframe（destroyed 印记）。
    sandbox
        .execute(
            "(function () {\n\
             document.body.removeChild(document.getElementById('s400objH'));\n\
             document.body.removeChild(document.getElementById('s400ifr'));\n\
             })();",
        )
        .unwrap();
    sandbox
        .execute("globalThis.__s400L2 = globalThis.length;")
        .unwrap();
    assert_eq!(
        s400_read(&mut sandbox, "L2"),
        "1",
        "(L6) 移除后计数递减：object isConnected=false 不计 + iframe destroyed 印记不计，bare-origin object 保持"
    );
    assert!(
        !fetch_log
            .lock()
            .unwrap()
            .iter()
            .any(|u| u == "https://wpt.test/"),
        "bare-origin object 不走 host fetch（shim 内合成，FIXME 目录语义）"
    );
}
