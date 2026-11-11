// part107 — html-semantics M2 切片 407（2026-11-12）：静态 embed 链 + fallback
// 子内容激活。
//
// embed-document.html / embed-document-under-content-visibility 双案红根修复钉
//（s405 FIXME(embed-static-events) 收口——静态 embed 无人启航，子文档永不物化
// → `parent.childLoaded` 恒 false）+ object-in-object-fallback-2 红根修复钉
//（平扫误启 loaded 祖先内子 object → assert_unreached）+ embed-in-object-fallback
// 双案（无 data 祖先 / 不可渲染 essence 祖先的子 embed 须激活）：
// - part04 `__zwStartParserObjects` 扩 EMBED 循环（src + being rendered 门）+
//   object/embed 共用祖先门（media 祖先 / 渲染成功 object 祖先 → 跳过）+ object
//   循环同步直调（祖先门须读先行元素终态——defer 会打散启动序）；
// - part01 data: 不可渲染 essence（非 text/*、非 image/*）→ 'fallback' 态 +
//   事件映射（object 渲染失败派 error、embed 恒 load）。
// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element

/// 桥测读回辅具（文件级自由函数——闭包持 `&mut sandbox` 跨 execute 触发 E0499）。
fn s407_read(sandbox: &mut zero_script_sandbox::V8Sandbox, id: &str) -> String {
    sandbox.execute(&format!("globalThis.__s407{id}")).unwrap().value
}

/// 钉①：静态 embed 启航链（parent 旗标通道）+ media 祖先门 + loaded-object
/// 祖先门 + 静默面。断言翻转面：静态 embed 不启航（旗标 false）；media 祖先
/// 误启航（video 内子 embed 物化——embed-ignored-in-media-element 钉面破）；
/// loaded 祖先误放行（object-in-object-fallback-2 同款 unreached 面）。
#[test]
fn embed_static_chain_faces_s407() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // 静态面三拓扑：顶层 embed（启航链）/ video 祖先 embed（media 门）/ loaded
    // object 祖先 embed（loaded 门——ldoc text/html 渲染成功）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <video><embed id=\"vEmb\" src=\"/s407/vchild.html\"></video>\
         <object id=\"ldoc\" data=\"/s407/ldoc.html\">\
         <embed id=\"nEmb\" src=\"/s407/nchild.html\"></object>\
         <embed id=\"tEmb\" src=\"/s407/tchild.html\">\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/s407/page.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // host fetch 路由：tchild 子文档置父旗标；ldoc 渲染成功 text/html；其余 URL
    // 恒 404（误启航经 fetch log 显形）。
    let fetch_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let fetch_log_c = fetch_log.clone();
    Sandbox::register_callback(
        &mut sandbox,
        "__zw_fetch",
        Box::new(move |args: &[String]| -> String {
            let url = args.get(2).cloned().unwrap_or_default();
            fetch_log_c.lock().unwrap().push(url.clone());
            if url.contains("/s407/tchild.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body><script>parent.__s407t = true;</script></body></html>".to_string();
            }
            if url.contains("/s407/ldoc.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Loaded</body></html>"
                    .to_string();
            }
            "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/html\u{1f}".to_string()
        }),
    );

    // 页脚本：静态 embed 元素事件监听（静默面断言——fireEvents=false 不派）。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s407t = false;\n\
             globalThis.__s407ev = [];\n\
             var t = document.getElementById('tEmb');\n\
             t.onload = function () { __s407ev.push('load:' + t.getAttribute('src')); };\n\
             t.onerror = function () { __s407ev.push('error'); };\n\
             })();",
        )
        .unwrap();

    // parser 静态启航（part06 window load 分支扫描位同款直调——同步链：object
    // 循环终态入缓存 → embed 门读祖先态 → 启航 → 子文档物化 → 子脚本置旗标）。
    sandbox.execute("__zwStartParserObjects();").unwrap();

    // 启航链：顶层 embed 子脚本已在 load 监听器可读的窗口置旗标（embed-document
    // 断言形——`addEventListener('load', () => assert_true(window.childLoaded))`）。
    let flag = s407_read(&mut sandbox, "t");
    assert_eq!(flag, "true", "顶层 embed 子文档 parent 旗标通道实得 {flag}");
    // 门面：video 祖先与 loaded-object 祖先的子 embed 零取回（fetch log 显形）。
    let log = fetch_log.lock().unwrap();
    let fetched_t = log.iter().filter(|u| u.contains("/s407/tchild")).count();
    let fetched_v = log.iter().filter(|u| u.contains("/s407/vchild")).count();
    let fetched_n = log.iter().filter(|u| u.contains("/s407/nchild")).count();
    let fetched_l = log.iter().filter(|u| u.contains("/s407/ldoc")).count();
    assert_eq!(fetched_t, 1, "顶层 embed 恰一取回实得 {fetched_t}");
    assert_eq!(fetched_l, 1, "loaded object 自身取回实得 {fetched_l}");
    assert_eq!(fetched_v, 0, "video 祖先 embed 零取回实得 {fetched_v}");
    assert_eq!(fetched_n, 0, "loaded 祖先子 embed 零取回实得 {fetched_n}");
    // 静默面：静态 sweep 不派元素终态事件（FIXME(static-events) 族记档）。
    let events = s407_read(&mut sandbox, "ev.join('|')");
    assert_eq!(events, "", "静态 embed sweep 不派 load/error 实得 {events}");
}

/// 钉②：fallback 子内容激活矩阵 + 不可渲染 essence 事件映射 + 可渲染 parity。
/// 拓扑镜像 embed-in-object-fallback{,-2}/object-in-object-fallback-2：
/// o1 无 data → 子 e1 激活；o2 不可渲染 essence → 子 e2 激活；o3 渲染成功 →
/// 子 e3 惰性；o5 不可渲染 → 子 oi（渲染成功）激活 + oi 的子 ei 惰性 + oi2
///（不可渲染）激活 → 子 ei2 激活。动态 pin：不可渲染 essence object 派 error
///（渲染失败映射）、text/plain essence object 派 load + contentDocument null
///（allowed-schemas 口径保持）。
#[test]
fn embed_object_fallback_activation_faces_s407() {
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
        "<html><body>\
         <object id=\"o1\"><embed id=\"e1\" src=\"/s407/f1.html\"></object>\
         <object id=\"o2\" data=\"data:application/x-does-not-exist,test\">\
         <embed id=\"e2\" src=\"/s407/f2.html\"></object>\
         <object id=\"o3\" data=\"/s407/doc3.html\">\
         <embed id=\"e3\" src=\"/s407/f3.html\"></object>\
         <object id=\"o5\" data=\"data:application/x-does-not-exist,test\">\
         <object id=\"oi\" data=\"/s407/docO.html\">\
         <embed id=\"ei\" src=\"/s407/fi.html\"></object>\
         <object id=\"oi2\" data=\"data:application/x-does-not-exist,test\">\
         <embed id=\"ei2\" src=\"/s407/fi2.html\"></object>\
         </object>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/s407/page.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // host fetch：三渲染成功 text/html（doc3/docO）+ 四子文档（f1/f2/fi2 激活置
    // 旗标；f3/fi 误启航显形用——同型置旗标，断言恒 false）。
    let fetch_log: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));
    let fetch_log_c = fetch_log.clone();
    Sandbox::register_callback(
        &mut sandbox,
        "__zw_fetch",
        Box::new(move |args: &[String]| -> String {
            let url = args.get(2).cloned().unwrap_or_default();
            fetch_log_c.lock().unwrap().push(url.clone());
            for tag in ["f1", "f2", "f3", "fi", "fi2"] {
                if url.contains(&format!("/s407/{tag}.html")) {
                    return format!(
                        "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body><script>parent.__s407{tag} = true;</script></body></html>"
                    );
                }
            }
            if url.contains("/s407/doc3.html") || url.contains("/s407/docO.html") {
                return "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body>Doc</body></html>"
                    .to_string();
            }
            "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/html\u{1f}".to_string()
        }),
    );

    // 页脚本：激活旗标初值 + 动态事件 pin 目标。
    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s407f1 = false;\n\
             globalThis.__s407f2 = false;\n\
             globalThis.__s407f3 = false;\n\
             globalThis.__s407fi = false;\n\
             globalThis.__s407fi2 = false;\n\
             globalThis.__s407dev = [];\n\
             })();",
        )
        .unwrap();

    // 静态启航（扫描位同款直调）。
    sandbox.execute("__zwStartParserObjects();").unwrap();

    // 激活矩阵：o1（无 data）→ e1 ✓；o2（不可渲染 essence）→ e2 ✓；o3（渲染
    // 成功）→ e3 ✗；o5 → oi ✓ → ei ✗；oi2（不可渲染）→ ei2 ✓。
    let f1 = s407_read(&mut sandbox, "f1");
    let f2 = s407_read(&mut sandbox, "f2");
    let f3 = s407_read(&mut sandbox, "f3");
    let fi = s407_read(&mut sandbox, "fi");
    let fi2 = s407_read(&mut sandbox, "fi2");
    assert_eq!(f1, "true", "无 data 祖先的子 embed 激活实得 {f1}");
    assert_eq!(f2, "true", "不可渲染 essence 祖先的子 embed 激活实得 {f2}");
    assert_eq!(f3, "false", "渲染成功祖先的子 embed 惰性实得 {f3}");
    assert_eq!(fi, "false", "渲染成功祖先（嵌套 oi）的子 embed 惰性实得 {fi}");
    assert_eq!(fi2, "true", "嵌套不可渲染 oi2 的子 embed 激活实得 {fi2}");

    // 动态 pin A：不可渲染 essence → fallback 态 → object 派 error（渲染失败
    // 映射——原 image 臂误派 load 的翻转面）。
    sandbox
        .execute(
            "(function () {\n\
             var d = document.createElement('object');\n\
             d.onerror = function () { __s407dev.push('error'); };\n\
             d.onload = function () { __s407dev.push('load'); };\n\
             d.data = 'data:application/x-does-not-exist,dyn';\n\
             document.body.append(d);\n\
             })();",
        )
        .unwrap();

    // 动态 pin B：text/plain essence → image 态 parity（allowed-schemas 口径——
    // load 派 + 无嵌套 navigable）。
    sandbox
        .execute(
            "(function () {\n\
             var d2 = document.createElement('object');\n\
             d2.onload = function () { __s407dev.push('load-plain:' + (d2.contentDocument === null)); };\n\
             d2.onerror = function () { __s407dev.push('error-plain'); };\n\
             d2.data = 'data:,Hello%2C%20World%21';\n\
             document.body.append(d2);\n\
             })();",
        )
        .unwrap();

    // _defer 微任务 drain：事件派发在各 execute 末 checkpoint 全量排空（含嵌套
    // queueMicrotask），pin A 事件序先于 pin B 代码执行落数组。

    let dev = s407_read(&mut sandbox, "dev.join('|')");
    assert_eq!(
        dev, "error|load-plain:true",
        "不可渲染 essence object 派 error + text/plain parity（load + null doc）实得 {dev}"
    );
    // 取回面：不可渲染 data: 不走网络（in-shim 解码臂）、f3/fi 惰性零取回、
    // doc3/docO/f1/f2/fi2 各恰一。
    let log = fetch_log.lock().unwrap();
    let count = |pat: &str| log.iter().filter(|u| u.contains(pat)).count();
    assert_eq!(
        count("/s407/f3"),
        0,
        "渲染成功祖先子 embed 零取回实得 {}",
        count("/s407/f3")
    );
    assert_eq!(
        count("/s407/fi.html"),
        0,
        "嵌套 oi 子 embed 零取回实得 {}",
        count("/s407/fi.html")
    );
    for pat in ["/s407/f1", "/s407/f2", "/s407/fi2", "/s407/doc3", "/s407/docO"] {
        assert_eq!(count(pat), 1, "{pat} 恰一取回实得 {}", count(pat));
    }
    assert!(
        !log.iter().any(|u| u.contains("data:")),
        "data: 资源零网络取回实得 {:?}",
        log.iter().filter(|u| u.contains("data:")).collect::<Vec<_>>()
    );
}

/// 钉③：s407 首轮终局 join 双红回归面（embed-represent-nothing-04 /
/// object-ignored-in-media-element P→F 翻转，门修复后复绿钉）。
/// - rendered 门：display:none 内静态 embed 不启航（spec never-rendered 不加载，
///   子文档 `parent.childLoaded` 置旗面不得发生）；可见孪生 embed 照启。
/// - object 版 media 门：video 内 object 不启航（旧 defer 启航时子文档
///   step-after-done 被 harness 吞掉的假绿面——同步启航会踩活测试
///   assert_unreached）。
#[test]
fn embed_rendered_gate_media_object_gates_s407() {
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
        "<html><body>\
         <embed id=\"hEmb\" src=\"/s407/hchild.html\" style=\"display:none\">\
         <embed id=\"sEmb\" src=\"/s407/schild.html\">\
         <video><object id=\"mo\" data=\"/s407/mobj.html\"></object></video>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/s407/page.html".to_string()));
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
            for tag in ["hchild", "schild", "mobj"] {
                if url.contains(&format!("/s407/{tag}.html")) {
                    return format!(
                        "__zwfr:200\u{1f}OK\u{1f}content-type\u{1e}text/html\u{1f}<html><body><script>parent.__s407{tag} = true;</script></body></html>"
                    );
                }
            }
            "__zwfr:404\u{1f}Not Found\u{1f}content-type\u{1e}text/html\u{1f}".to_string()
        }),
    );

    sandbox
        .execute(
            "(function () {\n\
             globalThis.__s407hchild = false;\n\
             globalThis.__s407schild = false;\n\
             globalThis.__s407mobj = false;\n\
             })();",
        )
        .unwrap();

    sandbox.execute("__zwStartParserObjects();").unwrap();

    let h = s407_read(&mut sandbox, "hchild");
    let s = s407_read(&mut sandbox, "schild");
    let m = s407_read(&mut sandbox, "mobj");
    assert_eq!(h, "false", "display:none embed 不启航置旗面实得 {h}");
    assert_eq!(s, "true", "可见 embed 启航链实得 {s}");
    assert_eq!(m, "false", "video 内 object 不启航置旗面实得 {m}");
    let log = fetch_log.lock().unwrap();
    let count = |pat: &str| log.iter().filter(|u| u.contains(pat)).count();
    assert_eq!(count("/s407/hchild"), 0, "display:none embed 零取回实得 {}", count("/s407/hchild"));
    assert_eq!(count("/s407/schild"), 1, "可见 embed 恰一取回实得 {}", count("/s407/schild"));
    assert_eq!(count("/s407/mobj"), 0, "video 内 object 零取回实得 {}", count("/s407/mobj"));
}
