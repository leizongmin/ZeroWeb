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

// part112 动画覆盖面一致性（双首轮返修钉）：批量通道与逐元素 gCS 在**动画/过渡
// 覆盖进行中**逐位一致——批量臂镜像四层 consult（query_style_override → js_anim →
// transition → cssa）+ transition before-change style 观测登记。
// 引擎现实与测试分型：canonicalize applier（animation.rs apply_single_property）
// 已覆盖 display（s560 全桥补臂）、未覆盖 visibility → js_anim/cssa 两通道对
// visibility 覆盖值恒 None（双端同回落级联，守卫钉防记录流缺元素/值漂移，consult
// 位镜像保证 applier 扩展后自动一致）；有产出的歧视形状是 transition
// allow-discrete display 离散对——覆盖值 "block" 反向于级联 "none"（fixture 样式
// 表钉级联），缺 consult 的批量臂落级联端值即判红。

/// part112 批量/逐元素记录比对 helper：从 bulk 记录流取 sel 段的 (display, visibility)。
fn p112_bulk_record<'a>(bulk: &'a str, sel: &str) -> Option<( &'a str, &'a str)> {
    bulk.split('|').find_map(|rec| {
        let mut parts = rec.split('\x1f');
        let s = parts.next()?;
        (s == sel).then(|| (parts.next().unwrap_or_default(), parts.next().unwrap_or_default()))
    })
}

/// WAAPI JS 动画面（守卫钉）：引擎面预置活跃动画条目（`js_anim_registry::register`，
/// 同 `__zw_js_anim_register` 引擎侧入口）。引擎现实：canonicalize 的 applier
/// （animation.rs `apply_single_property`）display 已有 s560 补臂、未覆盖
/// visibility → 两通道对
/// visibility 动画均无覆盖值、同回落级联——本钉防的是该边界下 bulk 记录流缺元素/
/// 值漂移；consult 位镜像（computed_style.rs 批量闭包与 gCS 同序）保证 applier
/// 扩展支持后覆盖值自动两通道一致。有产出的歧视形状由 transition display 离散对
/// 测试承担。
#[test]
fn p112_bulk_matches_gcs_under_js_animation() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = p112_sandbox(
        "https://wpt.test/g2b/innertext-bulk-jsanim.html",
        "<html><body><p id=\"t\">animtext</p></body></html>",
    );
    crate::js_dom_bridge::js_anim_registry::register(
        Some("#t"),
        None,
        r#"{"kf":[{"o":0,"p":{"visibility":"hidden"}},{"o":1,"p":{"visibility":"visible"}}],"dur":10000,"fill":"both"}"#,
    );
    let out = sandbox
        .execute(
            "var el = document.getElementById('t');\
             var gcs = getComputedStyle(el).visibility;\
             var bulk = __zw_innertext_styles('body');\
             var rec = bulk.split('|').map(function(r){return r.split('\\u001f');}).find(function(p){return p[0] === '#t';});\
             JSON.stringify({gcs: gcs, recVis: rec ? rec[2] : null})",
        )
        .unwrap()
        .value;
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(
        parsed["gcs"], serde_json::json!("visible"),
        "applier 未覆盖 visibility：gCS 动画窗内回落级联（canonicalize 能力边界）"
    );
    assert_eq!(
        parsed["recVis"], serde_json::json!("visible"),
        "批量记录 visibility == gCS（consult 双 None 一致；记录流不缺动画元素）"
    );
}

/// CSS transition 面（引擎面预置，同 transition_registry 自测装置）：display
/// Block→None + allow-discrete 离散对。fixture 样式表把 #t 级联钉在 display:none
/// （after-change 端点与活级联一致），过渡覆盖值 "block" 反向于级联 "none"——
/// 缺 transition consult 的批量臂落级联端值 "none" 即判红（歧视力承重面；级联
/// 为 UA 默认 Block 时覆盖 raw=from 与级联同值，canonicalize 恒 None、歧视力
/// 为零，recheck#1 major）。负 delay 大量级（-25s/100s）打 progress≈0.25 落
/// from，秒级窗口对执行延迟不敏感。visibility 线写形态不采用：visibility
/// 关键字在开区间走 fallback 不采纳通道（两通道同为 None，一致性行为已由
/// js_anim/cssa 面钉），display 离散对是引擎有产出的歧视形状。
#[test]
fn p112_bulk_matches_gcs_under_css_transition() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    use zero_style_system::ComputedStyle;
    let mut sandbox = p112_sandbox(
        "https://wpt.test/g2b/innertext-bulk-transition.html",
        "<html><head><style>#t{display:none}</style></head><body><p id=\"t\">tdis</p></body></html>",
    );
    let doc = zero_dom::parse_html(
        "<html><head><style>#t{display:none}</style></head><body><p id=\"t\">tdis</p></body></html>",
    );
    let nid = crate::find_by_selector(&doc, "#t").expect("target");
    let mut prev = ComputedStyle::default();
    let mut new = ComputedStyle::default();
    prev.display = zero_css_parser::values::DisplayValue::Block;
    new.display = zero_css_parser::values::DisplayValue::None;
    new.transition_property = vec!["display".to_string()];
    // 大量级时窗（100s/-25s）：progress≈0.25 对登记→JS 查询间隔秒级不敏感；
    // 短窗（100ms/-25ms）在间隔落 75ms 外时过渡已完、确定性翻红（recheck#1 flake 面）。
    new.transition_duration = vec![100000.0];
    new.transition_delay = vec![-25000.0];
    new.transition_timing_function = vec![zero_css_parser::values::TimingFunctionValue::Linear];
    new.transition_behavior = vec!["allow-discrete".to_string()];
    crate::js_dom_bridge::transition_registry::register_from_style_change(
        crate::js_dom_bridge::transition_registry::World::Gen,
        "#t",
        &prev,
        &new,
        &doc,
        nid,
    );
    let out = sandbox
        .execute(
            "var el = document.getElementById('t');\
             var gcs = getComputedStyle(el).display;\
             var bulk = __zw_innertext_styles('body');\
             var rec = bulk.split('|').map(function(r){return r.split('\\u001f');}).find(function(p){return p[0] === '#t';});\
             JSON.stringify({gcs: gcs, recDisp: rec ? rec[1] : null})",
        )
        .unwrap()
        .value;
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(
        parsed["gcs"], serde_json::json!("block"),
        "allow-discrete display 过渡 progress≈0.25 落 from——逐元素 gCS 返回覆盖值而非级联 none（级联由 fixture 样式表钉死）"
    );
    assert_eq!(
        parsed["recDisp"], serde_json::json!("block"),
        "批量 display == gCS 过渡覆盖值（transition consult 镜像；覆盖值 block 反向于级联 none，缺 consult 落级联判红）"
    );
}

/// CSS @keyframes 面（守卫钉）：animation-name 级联铺面（animationName 非 none，
/// 钉 note_stylesheets → cascade 活性链）+ 动画窗内批量记录 == 逐元素 gCS。
/// 引擎现实同 js_anim 面：canonicalize applier 未覆盖 visibility，cssa 覆盖值
/// None 两通道同回落级联；cssa consult 位镜像保证 applier 扩展后自动一致。
#[test]
fn p112_bulk_matches_gcs_under_css_animation() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = p112_sandbox(
        "https://wpt.test/g2b/innertext-bulk-cssa.html",
        "<html><head><style>\
         @keyframes p112kf{from{visibility:hidden}to{visibility:hidden}}\
         .ca{animation:p112kf 10s linear both}\
         </style></head><body><p class=\"ca\">catext</p></body></html>",
    );
    let out = sandbox
        .execute(
            "var el = document.querySelector('.ca');\
             var gcs = getComputedStyle(el).visibility;\
             var an = getComputedStyle(el).animationName;\
             var bulk = __zw_innertext_styles('body');\
             var rec = bulk.split('|').map(function(r){return r.split('\\u001f');}).find(function(p){return p[0] === el.__zwSelector;});\
             JSON.stringify({gcs: gcs, an: an, recVis: rec ? rec[2] : null})",
        )
        .unwrap()
        .value;
    let parsed: serde_json::Value = serde_json::from_str(&out).expect("json");
    assert_eq!(
        parsed["an"], serde_json::json!("p112kf"),
        "@keyframes 级联铺面（note → cascade 链活性）"
    );
    assert_eq!(
        parsed["gcs"], serde_json::json!("visible"),
        "applier 未覆盖 visibility：gCS 动画窗内回落级联（canonicalize 能力边界）"
    );
    assert_eq!(
        parsed["recVis"], serde_json::json!("visible"),
        "批量记录 visibility == gCS（consult 双 None 一致；记录流不缺动画元素）"
    );
}
