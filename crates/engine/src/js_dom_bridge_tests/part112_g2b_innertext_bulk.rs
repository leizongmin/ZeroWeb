// part112 — t-g2b R6：innerText 批量样式通道钉面。契约：`_zwGetInnerText` 经
// `__zw_innertext_styles`（`innertext_bulk_styles_doc` 记录流）一次宿主往返取子树
// display/visibility，与逐元素 getComputedStyle 查询逐位一致（同代际键同序列化）；
// hidden/visibility-hidden 子树排除、script/style/template 排除、br 换行、表单
// value（shim IDL 面，bulk 不承载）维持。
// 已知近似边界（R3028/R3260 v2 既有，非本钉面）：sel 根存在同 turn 写后读文本分歧
// 时（part04 innerText getter B1 快捷臂 latest-wins ≠ 快照）直接返原文拼接、绕过
// 渲染感知收集器——动态 append 场景经该臂，与本通道无关（part112 前期诊断实证
// gcsLog 空 + 原文包含同时成立）。故本钉面第二测经 handle 臂容器（getter 无 B1
// 快捷臂，walk 必跑）钉「批量 miss → 逐元素回落」路径：动态（createElement）节点
// 不在批量记录内，逐元素 getComputedStyle 回落仍参与 walk，隐藏判定同语义。
// 沙盒模式沿用 part63 m3s2_sandbox 先例。

/// part112 公共沙盒（part63 m3s2_sandbox 同构）。
fn p112_sandbox(page_url: &str, dom_html: &str) -> zero_script_sandbox::V8Sandbox {
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

/// 批量通道端到端：body.innerText 走批量路径，渲染感知语义逐项维持。
#[test]
fn p112_innertext_bulk_channel_render_aware_semantics() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = p112_sandbox(
        "https://wpt.test/g2b/innertext-bulk.html",
        "<html><body>\
         <p>para1</p>\
         <div class=\"h\"><span>hidtext</span></div>\
         <span class=\"v\">vistext<span>deepv</span></span>\
         <script>var p112x = 1;</script>\
         <style>.h{display:none}.v{visibility:hidden}</style>\
         <br>\
         <input value=\"typedval\">\
         <textarea>taval</textarea>\
         <select><option>optval</option></select>\
         <template><div>tpltext</div></template>\
         </body></html>",
    );
    let out = sandbox
        .execute(
            "var r = { it: document.body.innerText };\
             var sel = document.body.__zwSelector || '';\
             try { r.bulk = typeof __zw_innertext_styles === 'function' ? __zw_innertext_styles(sel) : 'NOFN'; } catch (e) { r.bulk = 'ERR:' + e.message; }\
             r.sel = sel;\
             JSON.stringify(r)",
        )
        .unwrap()
        .value;
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("json");
    let it = parsed["it"].as_str().unwrap_or_default();
    // 包含面：可见文本 + 表单 value（shim IDL 面——bulk 只承载样式，不承载 value）。
    assert!(it.contains("para1"), "可见文本包含（innerText 基面）");
    assert!(it.contains("typedval"), "input value 经 IDL getter 包含（bulk 不承载 value）");
    assert!(it.contains("taval"), "textarea value 包含");
    assert!(it.contains("optval"), "select value 包含");
    assert!(it.contains('\n'), "br/block 换行面包含");
    // 排除面：display:none 子树、visibility:hidden 子树、script/style/template 内容。
    assert!(!it.contains("hidtext"), "display:none 子树排除（批量值判定）");
    assert!(!it.contains("vistext"), "visibility:hidden 排除");
    assert!(!it.contains("deepv"), "visibility:hidden 子树整体排除");
    assert!(!it.contains("p112x"), "script 内容排除");
    assert!(!it.contains(".h{"), "style 内容排除");
    assert!(!it.contains("tpltext"), "template 内容排除");
    // 批量通道在册：记录流含 hidden 元素的 display='none' 记录（\x1f 分隔三段）。
    let bulk = parsed["bulk"].as_str().unwrap_or_default();
    assert!(bulk != "NOFN", "__zw_innertext_styles 回调已注册");
    let hidden_rec = bulk
        .split('|')
        .find(|rec| rec.split('\x1f').nth(1) == Some("none"))
        .expect("批量记录含 display:none 元素");
    assert!(hidden_rec.split('\x1f').nth(2) == Some("visible"), "display:none 记录 visibility 段在册");
    let vis_rec = bulk
        .split('|')
        .find(|rec| rec.split('\x1f').nth(2) == Some("hidden"))
        .expect("批量记录含 visibility:hidden 元素");
    assert!(vis_rec.split('\x1f').nth(1).is_some(), "visibility:hidden 记录 display 段在册");
}

/// 批量 miss 回落：createElement 动态节点不在批量记录（host doc 快照外；sel 根同
/// turn 分歧场景由 getter B1 快捷臂短路、不经本通道——见文件头注）→ 经 handle 臂
/// 容器（getter 无快捷臂，walk 必跑、根无 sel → bulk 通道缺席）钉逐元素回落路径：
/// 可见动态文本包含、动态 display:none 子树按 handle gCS 通道判定排除。
#[test]
fn p112_innertext_bulk_miss_falls_back_per_element() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = p112_sandbox(
        "https://wpt.test/g2b/innertext-bulk-fallback.html",
        "<html><body><p>base</p></body></html>",
    );
    let out = sandbox
        .execute(
            "var cont = document.createElement('div');\
             document.body.appendChild(cont);\
             var d1 = document.createElement('div');\
             d1.textContent = 'dynvisible';\
             cont.appendChild(d1);\
             var d2 = document.createElement('div');\
             d2.textContent = 'dynhidden';\
             d2.style.display = 'none';\
             cont.appendChild(d2);\
             var r = { it: cont.innerText };\
             r.hasDynVis = r.it.indexOf('dynvisible') >= 0;\
             r.hasDynHid = r.it.indexOf('dynhidden') >= 0;\
             JSON.stringify(r)",
        )
        .unwrap()
        .value;
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(parsed["hasDynVis"], serde_json::json!(true), "动态可见节点经逐元素回落路径包含");
    assert_eq!(
        parsed["hasDynHid"], serde_json::json!(false),
        "动态 display:none 子树经 handle gCS 回落判定排除（与逐元素查询同语义）"
    );
}
