// part108 — html-semantics M2 切片 408（2026-11-12）：detached 文档 style 元素
// LinkStyle `sheet` + headEl 本地树三件套。
//
// non-parser-created-doc.html 红根修复钉（`createHTMLDocument().createElement
// ('style')` + `doc.head.appendChild` 后 `assert_true(!!style.sheet)`）：
// - part03 headEl 补本地树三件套（R126 docEl 同款缺口——appendChild 旧落全局
//   Node.prototype，plain 子节点 childNodes 不插、parentNode 不设）；
// - part03 detached createElement 的 STYLE 产物直挂 `sheet` 访问器（R5256
//   dialog / s69 img.decode 同款 createElement 时直挂——plain `_zwMEl` 产物无
//   属性读取层路由，main sheet getter 分支按 sel/handle 代理匹配不命中）；
// - part06 `_makeStyleSheet` R5271b 门扩 plain owner 臂（spec LinkStyle：sheet
//   在任意文档树内即存在——detached doc 树含；未挂载 walk 无根返 null 保持）。
// https://drafts.csswg.org/cssom-1/#dom-linkstyle-sheet
// https://html.spec.whatwg.org/multipage/semantics.html#update-a-style-block

/// 桥测读回辅具（文件级自由函数——闭包持 `&mut sandbox` 跨 execute 触发 E0499）。
fn s408_read(sandbox: &mut zero_script_sandbox::V8Sandbox, id: &str) -> String {
    sandbox.execute(&format!("globalThis.__s408{id}")).unwrap().value
}

/// 钉①：driving 面镜像 + headEl 树接线 + SameObject 身份 + 出树/回挂门。
/// 断言翻转面：未挂载 sheet 恒 null（LinkStyle not-in-a-Document 保持）；
/// appendChild 后树接线失效（head 子不插/父链不设 → sheet undefined——旧红根）；
/// removeChild 出树 null、回挂身份恢复（R5271 [SameObject] 缓存跨门读写一致）。
#[test]
fn detached_doc_style_sheet_faces_s408() {
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
        "<html><body><p>s408 detached style sheet pin</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/s408/page.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 时序镜像 corpus non-parser-created-doc.html：createHTMLDocument →
    // createElement('style') → head.appendChild → 读 sheet（CSS-in-JS 惯用时序）。
    sandbox
        .execute(
            "(function () {\n\
             var doc = document.implementation.createHTMLDocument('');\n\
             var style = doc.createElement('style');\n\
             var r = globalThis.__s408r = {};\n\
             r.sheetBefore = style.sheet === null ? 'null'\n\
               : (style.sheet === undefined ? 'undef' : 'obj');\n\
             doc.head.appendChild(style);\n\
             var sh = style.sheet;\n\
             r.sheetAfter = sh === null ? 'null' : (sh === undefined ? 'undef' : 'obj');\n\
             r.identity = sh !== null && sh !== undefined && sh === style.sheet ? 'same' : 'diff';\n\
             r.parent = style.parentNode === doc.head ? 'head' : 'other';\n\
             r.headKids = String(doc.head.childNodes.length);\n\
             r.rulesLen = '';\n\
             try { r.rulesLen = String(sh.cssRules.length); } catch (e) { r.rulesLen = 'ERR'; }\n\
             doc.head.removeChild(style);\n\
             r.sheetAfterRemove = style.sheet === null ? 'null' : 'truthy';\n\
             doc.head.appendChild(style);\n\
             r.sheetIdentityBack = style.sheet === sh ? 'same' : 'diff';\n\
             })();",
        )
        .unwrap();

    // 未挂载 → null（spec LinkStyle not-in-a-Document 面保持；旧实现此处恒
    // undefined——访问器缺位）。
    assert_eq!(
        s408_read(&mut sandbox, "r.sheetBefore"),
        "null",
        "未挂载 style.sheet 恒 null 实得 {}",
        s408_read(&mut sandbox, "r.sheetBefore")
    );
    // 挂载后 → truthy sheet（旧红根：headEl 无本地树方法 + plain 产物无访问器，
    // sheet undefined）。
    assert_eq!(
        s408_read(&mut sandbox, "r.sheetAfter"),
        "obj",
        "挂载后 sheet 对象实得 {}",
        s408_read(&mut sandbox, "r.sheetAfter")
    );
    assert_eq!(
        s408_read(&mut sandbox, "r.identity"),
        "same",
        "sheet [SameObject] 跨读同对象实得 {}",
        s408_read(&mut sandbox, "r.identity")
    );
    // headEl 树接线（R126 同款语义）：父链 + 子表（title + style = 2，R130 title
    // 子先在）。
    assert_eq!(
        s408_read(&mut sandbox, "r.parent"),
        "head",
        "style.parentNode === doc.head 实得 {}",
        s408_read(&mut sandbox, "r.parent")
    );
    assert_eq!(
        s408_read(&mut sandbox, "r.headKids"),
        "2",
        "head.childNodes = [title, style] 实得 {}",
        s408_read(&mut sandbox, "r.headKids")
    );
    // detached 树无 mutation 历史 → 空 wire 臂 cssRules 恒空
    //（FIXME(style-detached-rules)：textContent 内联规则未接 CSSOM）。
    assert_eq!(
        s408_read(&mut sandbox, "r.rulesLen"),
        "0",
        "detached sheet cssRules 恒空实得 {}",
        s408_read(&mut sandbox, "r.rulesLen")
    );
    // 出树 → null；回挂 → 同一对象（缓存身份恢复）。
    assert_eq!(
        s408_read(&mut sandbox, "r.sheetAfterRemove"),
        "null",
        "removeChild 出树 sheet 恒 null 实得 {}",
        s408_read(&mut sandbox, "r.sheetAfterRemove")
    );
    assert_eq!(
        s408_read(&mut sandbox, "r.sheetIdentityBack"),
        "same",
        "回挂后 sheet 身份恢复实得 {}",
        s408_read(&mut sandbox, "r.sheetIdentityBack")
    );
}

/// 钉②：detached STYLE type 门（s156/s160 精确匹配语义镜像——part04 R3036
/// getter 同规则）。断言翻转面：type 门缺位 → 不可渲染 type 误建 sheet。
#[test]
fn detached_style_type_gate_faces_s408() {
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
        "<html><body><p>s408 detached type gate pin</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://wpt.test/s408/page.html".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 五案：不可渲染 essence → null；精确 text/css → sheet；ASCII-CI → sheet
    //（s156 案面）；空串 → sheet（空串 = 无 type 语义，style_type_html test2）；
    // 带空白 → null（s160：无 trimming 精确匹配）。
    sandbox
        .execute(
            "(function () {\n\
             var doc = document.implementation.createHTMLDocument('');\n\
             var g = globalThis.__s408t = {};\n\
             function pin(key, type) {\n\
               var s = doc.createElement('style');\n\
               s.setAttribute('type', type);\n\
               doc.head.appendChild(s);\n\
               g[key] = s.sheet === null || s.sheet === undefined ? 'null' : 'obj';\n\
               doc.head.removeChild(s);\n\
             }\n\
             pin('jibberish', 'text/jibberish');\n\
             pin('exact', 'text/css');\n\
             pin('ci', 'TeXt/CsS');\n\
             pin('empty', '');\n\
             pin('spaced', ' text/css ');\n\
             })();",
        )
        .unwrap();

    assert_eq!(
        s408_read(&mut sandbox, "t.jibberish"),
        "null",
        "不可渲染 type 不建 sheet 实得 {}",
        s408_read(&mut sandbox, "t.jibberish")
    );
    assert_eq!(
        s408_read(&mut sandbox, "t.exact"),
        "obj",
        "text/css 建 sheet 实得 {}",
        s408_read(&mut sandbox, "t.exact")
    );
    assert_eq!(
        s408_read(&mut sandbox, "t.ci"),
        "obj",
        "TeXt/CsS ASCII-CI 建 sheet 实得 {}",
        s408_read(&mut sandbox, "t.ci")
    );
    assert_eq!(
        s408_read(&mut sandbox, "t.empty"),
        "obj",
        "空 type 不设门建 sheet 实得 {}",
        s408_read(&mut sandbox, "t.empty")
    );
    assert_eq!(
        s408_read(&mut sandbox, "t.spaced"),
        "null",
        "带空白 type 精确匹配不建 sheet 实得 {}",
        s408_read(&mut sandbox, "t.spaced")
    );
}
