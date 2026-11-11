// part100 — html-semantics M2 切片 398（2026-11-11）：OBJECT contentWindow 嵌套
// 浏览上下文面。
//
// R5274（part87）钉死的是「无 data 资源/无 fetch 通道 → 恒 null」尾步；本切片
// 接通 contentWindow 消费点的惰性物化管线（part01 _zwLoadObjectNavigable）：
// 成功加载 text/html 资源 → contentWindow 返回嵌套 navigable 的 window proxy
//（spec #dom-object-contentwindow）；fetch 失败（404）/替换内容类型（image/*）/
// 无 data 属性 → null（#the-object-element 处理模型尾步）。name 快照：嵌套
// navigable 命名取创建时点 name 内容属性值，之后内容属性增删不影响
//（object-attributes「name 改后 contentWindow.name 仍 'o'」断言链）。
// contentDocument 恒 null 边界保持（嵌套文档访问面散单另案）。
// https://html.spec.whatwg.org/multipage/iframe-embed-object.html#the-object-element

/// 钉①：四面分类 + name 快照 + 同 identity。
/// 断言翻转：OBJECT contentWindow 臂被撤（(a) 复落恒 null）；Content-Type 门失效
///（(d) image 资源误建 navigable）；name 快照失效（(a2/a3) 读到元素 name 反射
/// 动态值或 undefined）；缓存失效（(e) 二次读换 identity）。
#[test]
fn object_content_window_nested_navigable_s398() {
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
         <object id='ok' data='/common/blank.html' name='o'></object>\
         <object id='miss' data='/missing.html'></object>\
         <object id='nodata' type='image/png'></object>\
         <object id='img' data='/images/blue.png'></object>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/s398/x.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // 同步 `__zw_fetch` stub（part20 R115 先例）——wire：__zwfr: status \x1f
    // statusText \x1f headers \x1f body；headers 按 URL 分流（text/html 成功 /
    // 404 缺失 / image/png 替换内容）。
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
            "var okEl = document.getElementById('ok');\n\
             var win = okEl.contentWindow;\n\
             globalThis.__s398a = String(win !== null);\n\
             globalThis.__s398a2 = String(win !== null && win.name === 'o');\n\
             okEl.setAttribute('name', 'o1');\n\
             globalThis.__s398a3 = String(win !== null && win.name === 'o');\n\
             okEl.removeAttribute('name');\n\
             globalThis.__s398a4 = String(win !== null && win.name === 'o' && okEl.name === '');\n\
             globalThis.__s398b = String(document.getElementById('miss').contentWindow === null);\n\
             globalThis.__s398c = String(document.getElementById('nodata').contentWindow === null);\n\
             globalThis.__s398d = String(document.getElementById('img').contentWindow === null);\n\
             globalThis.__s398e = String(okEl.contentWindow === win);\n\
             globalThis.__s398f = String(okEl.contentDocument === null);",
        )
        .unwrap();
    let mut assert_s398 = |id: &str, expected: &str, msg: &str| {
        assert_eq!(
            sandbox.execute(&format!("globalThis.__s398{id}")).unwrap().value,
            expected,
            "{msg}"
        );
    };
    assert_s398("a", "true", "(a) 成功 text/html data → contentWindow 非 null（嵌套 navigable 物化）");
    assert_s398("a2", "true", "(a2) contentWindow.name 为创建时点 name 内容属性快照 'o'");
    assert_s398("a3", "true", "(a3) setAttribute('name','o1') 后快照仍 'o'（browsing context name 独立记账）");
    assert_s398("a4", "true", "(a4) removeAttribute 后快照仍 'o' 且 name IDL 反射为 ''（两记账分离）");
    assert_s398("b", "true", "(b) 404 fetch 失败 → fallback，contentWindow null");
    assert_s398("c", "true", "(c) 无 data 属性 → 不取回，contentWindow null（object-attributes obj2 面）");
    assert_s398("d", "true", "(d) image/* 替换内容类型 → 无嵌套 navigable，contentWindow null");
    assert_s398("e", "true", "(e) 二次读同 identity（entry 缓存）");
    assert_s398("f", "true", "(f) contentDocument 恒 null 边界保持（R5274 尾步不扩）");
}

/// 钉②：SVG data: URL（opaque origin 替换内容）与无 fetch 通道保 R5274 恒 null
/// 面——part87 钉①(b) 语义在新管线下的两路保持（非 text/html Content-Type /
/// __zw_fetch 缺席）。
/// 断言翻转：Content-Type 门漏放 data: URL（误建 navigable）；fetch 缺席分支
/// 误物化空 win。
#[test]
fn object_content_window_null_faces_kept_s398() {
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
        "<html><body><object id='svg' data='data:image/svg+xml,<svg xmlns=\"http://www.w3.org/2000/svg\"><rect height=\"100\" width=\"100\"/></svg>'></object></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s398b".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    // 无 __zw_fetch 注册（part87 同环境口径）——fetch 缺席分支保 null。
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox
        .execute(
            "var svgEl = document.getElementById('svg');\n\
             globalThis.__s398g = String(svgEl.contentWindow === null);\n\
             globalThis.__s398h = String(svgEl.contentWindow === null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__s398g").unwrap().value,
        "true",
        "(g) data: SVG（无 fetch 通道）→ contentWindow null（R5274 恒 null 面保持）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__s398h").unwrap().value,
        "true",
        "(h) error/loading 态 entry 缓存复用，二次读同 null"
    );
}
