// part101 — html-semantics M2 切片 399（2026-11-11）：object 动态加载链。
//
// s398 接通的是消费点惰性物化（contentWindow/contentDocument 首读即加载）；本
// 切片接通 spec #the-object-element 处理模型的**触发面**：
// - 插入步（object element insertion steps）：已连接 object 启动资源取回，终态
//   派 trusted load/error（bubbles/cancelable=false——object-events 四案断言面；
//   Event interface + target identity 同面）；
// - data IDL setter：已连接 object 重启取回（换资源旧 entry 作废——events 序列
//   error→load）；detached 只写内容属性不触发（插入钩子接入时再启动）；
// - about:blank：不 fetch 直建空文档嵌套浏览上下文 + load（iframe R130 同语义）；
// - 嵌套 window 暴露 Window interface 对象（_zwMakeIframeWin 挂 win.Window——
//   object-events about:blank 案 `contentWindow instanceof contentWindow.Window`
//   断言链）；
// - image/* 替换内容派 load（成功渲染）但无嵌套 navigable（contentWindow/
//   contentDocument 保 null）。
// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element

/// 桥测读回辅具（文件级自由函数——闭包持 `&mut sandbox` 跨 execute 触发 E0499）。
fn s399_read(sandbox: &mut zero_script_sandbox::V8Sandbox, id: &str) -> String {
    sandbox
        .execute(&format!("globalThis.__s399{id}"))
        .unwrap()
        .value
}

/// 钉①：插入钩子四事件面 + about:blank 嵌套上下文 + Window interface + detached
/// setter 只写属性。断言翻转：插入钩子被撤（rec1/rec2/rec3 恒空——load/error
/// 不派发）；事件 flags 失真（trusted/bubbles/cancelable/interface 任一翻）；
/// win.Window 挂接被撤（instanceof false 或 TypeError）；detached 触发（rec4
/// 误真——spec nothing 步被跳过）。
#[test]
fn object_dynamic_load_chain_s399() {
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
        Arc::new(Mutex::new("https://wpt.test/s399/x.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 同步 `__zw_fetch` stub——text/html 成功 / 404 缺失 / image/png 替换内容。
    Sandbox::register_callback(
        &mut sandbox,
        "__zw_fetch",
        Box::new(|args: &[String]| -> String {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.contains("missing.html") {
                return "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/plain\u{1f}".to_string();
            }
            if url.contains("blue.png") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}image/png\u{1f}\u{0}_png".to_string();
            }
            "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Blank</body></html>".to_string()
        }),
    );

    // 第一段 execute：布置四相位（插入钩子延迟到 execute 末 microtask checkpoint
    // drain——事件在第二段可读）。
    sandbox
        .execute(
            "(function () {\n\
             function rec() { return { fired: false }; }\n\
             // A：插入钩子 + trusted load（text/html）+ 嵌套文档 face。\n\
             var o1 = document.createElement('object');\n\
             o1.setAttribute('data', '/common/blank.html');\n\
             var r1 = rec();\n\
             o1.onload = function (e) {\n\
               r1.fired = true;\n\
               r1.iface = Object.getPrototypeOf(e).constructor === globalThis.Event;\n\
               r1.trusted = e.isTrusted === true;\n\
               r1.nobubble = e.bubbles === false;\n\
               r1.nocancel = e.cancelable === false;\n\
               r1.target = e.target === o1;\n\
               r1.doc = o1.contentDocument !== null;\n\
               r1.origin = !!(o1.contentDocument && o1.contentDocument.location\n\
                 && o1.contentDocument.location.origin === globalThis.location.origin);\n\
             };\n\
             document.body.appendChild(o1);\n\
             // B：插入钩子 + trusted error（404），load 不派。\n\
             var o2 = document.createElement('object');\n\
             o2.setAttribute('data', '/missing.html');\n\
             var r2 = rec();\n\
             r2.load = false;\n\
             o2.onerror = function (e) { r2.fired = true; r2.trusted = e.isTrusted === true; r2.type = e.type; };\n\
             o2.onload = function () { r2.load = true; };\n\
             document.body.appendChild(o2);\n\
             // C：about:blank——不 fetch 直建嵌套上下文 + Window interface instanceof。\n\
             var o3 = document.createElement('object');\n\
             o3.setAttribute('data', 'about:blank');\n\
             var r3 = rec();\n\
             o3.onload = function () {\n\
               r3.fired = true;\n\
               try { r3.instanceof = o3.contentWindow instanceof o3.contentWindow.Window; }\n\
               catch (e3) { r3.instanceof = 'ERR:' + e3.message; }\n\
               try { r3.doc = o3.contentDocument !== null; }\n\
               catch (e3b) { r3.doc = 'ERR:' + e3b.message; }\n\
             };\n\
             document.body.appendChild(o3);\n\
             // D：detached data setter——只写内容属性，无事件（spec nothing 步）。\n\
             var o4 = document.createElement('object');\n\
             var r4 = rec();\n\
             o4.onerror = function () { r4.fired = true; };\n\
             o4.onload = function () { r4.fired = true; };\n\
             o4.data = '/missing.html';\n\
             globalThis.__s399r1 = r1; globalThis.__s399r2 = r2;\n\
             globalThis.__s399r3 = r3; globalThis.__s399r4 = r4;\n\
             globalThis.__s399o4data = o4.data;\n\
             })();",
        )
        .unwrap();
    // 第二段 execute：drain 完成后读回。
    sandbox
        .execute(
            "(function () {\n\
             function flat(r) {\n\
               var out = [];\n\
               for (var k in r) out.push(k + '=' + r[k]);\n\
               return out.sort().join(',');\n\
             }\n\
             globalThis.__s399f1 = flat(globalThis.__s399r1);\n\
             globalThis.__s399f2 = flat(globalThis.__s399r2);\n\
             globalThis.__s399f3 = flat(globalThis.__s399r3);\n\
             globalThis.__s399f4 = flat(globalThis.__s399r4);\n\
             })();",
        )
        .unwrap();
    assert_eq!(
        s399_read(&mut sandbox, "f1"),
        "doc=true,fired=true,iface=true,nobubble=true,nocancel=true,origin=true,target=true,trusted=true",
        "(A) 插入钩子 load 面：trusted + 非 bubbles/cancelable + interface/target + 嵌套文档同源 origin"
    );
    assert_eq!(
        s399_read(&mut sandbox, "f2"),
        "fired=true,load=false,trusted=true,type=error",
        "(B) 插入钩子 error 面：404 → trusted error，load 不派"
    );
    assert_eq!(
        s399_read(&mut sandbox, "f3"),
        "doc=true,fired=true,instanceof=true",
        "(C) about:blank：直建嵌套上下文 + contentWindow instanceof contentWindow.Window（win.Window 挂接）"
    );
    assert_eq!(
        s399_read(&mut sandbox, "f4"),
        "fired=false",
        "(D) detached data setter 只写属性不触发（nothing 步）"
    );
    assert_eq!(
        s399_read(&mut sandbox, "o4data"),
        "https://wpt.test/missing.html",
        "(D2) detached setter 后 data getter URL 反射 round-trip"
    );
}

/// 钉②：image/* 替换内容 load + null 面保持；data setter 重启序列（error→load）；
/// 无 data 插入静默。断言翻转：image 分支误建 navigable（contentWindow 非_null）；
/// 重启语义失效（旧 error entry 复用——events 只 error 或序列错序）；无 data 插入
/// 误派事件（events 含 error/load）。
#[test]
fn object_dynamic_load_restart_faces_s399() {
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
        Arc::new(Mutex::new("https://wpt.test/s399/y.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    Sandbox::register_callback(
        &mut sandbox,
        "__zw_fetch",
        Box::new(|args: &[String]| -> String {
            let url = args.get(2).map(String::as_str).unwrap_or("");
            if url.contains("missing.html") {
                return "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/plain\u{1f}".to_string();
            }
            if url.contains("blue.png") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}image/png\u{1f}\u{0}_png".to_string();
            }
            "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Blank</body></html>".to_string()
        }),
    );

    sandbox
        .execute(
            "(function () {\n\
             // E：image/* 替换内容——load 派发（成功渲染）但无嵌套 navigable。\n\
             var o5 = document.createElement('object');\n\
             o5.setAttribute('data', '/images/blue.png');\n\
             var r5 = { fired: false, err: false };\n\
             o5.onload = function () { r5.fired = true; };\n\
             o5.onerror = function () { r5.err = true; };\n\
             document.body.appendChild(o5);\n\
             // F：data setter 重启——无 data 插入（静默）→ error 资源 → 换 text/html\n\
             //（旧 entry 作废，load 再派）→ 事件序列 error,load。\n\
             var o6 = document.createElement('object');\n\
             var r6 = { events: [] };\n\
             o6.onerror = function () { r6.events.push('error'); };\n\
             o6.onload = function () { r6.events.push('load'); };\n\
             document.body.appendChild(o6);\n\
             o6.data = '/missing.html';\n\
             o6.data = '/common/blank.html';\n\
             globalThis.__s399r5 = r5; globalThis.__s399r6 = r6;\n\
             })();",
        )
        .unwrap();
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s399g5 = globalThis.__s399r5.fired + '/' + globalThis.__s399r5.err;\n\
             globalThis.__s399g6 = globalThis.__s399r6.events.join(',');\n\
             })();",
        )
        .unwrap();
    assert_eq!(
        s399_read(&mut sandbox, "g5"),
        "true/false",
        "(E) image/* 替换内容派 load 不派 error（成功渲染语义）"
    );
    assert_eq!(
        s399_read(&mut sandbox, "g6"),
        "error,load",
        "(F) data setter 重启序列：无 data 静默 → error → 换资源旧 entry 作废再 load"
    );
    // image 分支 null 面保持（无嵌套 navigable）——drain 后读 o5。
    sandbox
        .execute(
            "(function () {\n\
             var o5b = document.querySelector('object[data=\"/images/blue.png\"]');\n\
             globalThis.__s399g5n = String(o5b.contentWindow === null) + '/' + String(o5b.contentDocument === null);\n\
             })();",
        )
        .unwrap();
    assert_eq!(
        s399_read(&mut sandbox, "g5n"),
        "true/true",
        "(E2) image/* 无嵌套 navigable——contentWindow/contentDocument 保 null（object-attributes obj2 口径）"
    );
}
