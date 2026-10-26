// part80 — html-semantics M2 切片 49（R5253，2026-10-24）：fragment 导航 details 展开。
//
// spec「scroll to the fragment」steps：target 自身是 details → 设 open；否则逐个设
// target 链上未 open 的 details 祖先（target 落于 closed details 内时 fragment 导航
// 自动展开，展开后才有几何可量，随后既有 scrollIntoView/窗口位同步步正常落位）。
// WPT auto-expand-details-element-fragment 断言面：`location.hash = '#target'` 后
// closed details 应 open 且 pageYOffset 离开 0。修复落在 part02
// `_scrollToAnchorForHash`（几何量测前插展开步，经 setAttribute 走既有属性通道——
// toggle 事件 / name 排他组链自动）。
// https://html.spec.whatwg.org/multipage/browsing-the-web.html#scroll-to-the-fragment-spec

/// 钉：fragment 导航 details 展开——(a) closed details 内 target 经 hash 导航后
/// details open；(b) target 链外的 closed details 不被误开（展开步限 target 祖先链）；
/// (c) 无匹配 hash 不炸（top-fallback 路径保持）。
/// 滚动落位（pageYOffset 离开 0）不在本契约：bridge sandbox 无帧驱动/布局供给面，
/// 展开后几何不可量、锚点滚动不落位（展开步经 rAF 补偿臂在真帧重入落位）；该面由
/// WPT auto-expand-details-element-fragment 单案承载（runner RAF_FRAME_DRIVEN 已 Pass）。
/// 断言翻转：展开步被撤 → (a) details 保持 closed（toggle 链不触发）。
#[test]
fn fragment_nav_expands_details_r5253() {
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s49".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var sp = document.createElement('div');\
             sp.setAttribute('style', 'height:2000px');\
             document.body.appendChild(sp);\
             var det = document.createElement('details');\
             var tgt = document.createElement('div');\
             tgt.id = 'r253target';\
             det.appendChild(tgt);\
             document.body.appendChild(det);\
             globalThis.location.hash = '#r253target';\
             globalThis.__r5253a = String(det.hasAttribute('open'));\
             var det2 = document.createElement('details');\
             document.body.appendChild(det2);\
             var d2 = document.createElement('div');\
             d2.id = 'r253plain';\
             document.body.appendChild(d2);\
             globalThis.location.hash = '#r253plain';\
             globalThis.__r5253b = String(!det2.hasAttribute('open'));\
             globalThis.location.hash = '#r5253nomatch';\
             globalThis.__r5253c = 'ok';",
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__r5253a").unwrap().value,
        "true",
        "(a) hash 导航展开 closed details"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5253b").unwrap().value,
        "true",
        "(b) 裸 target 导航不误开链外 closed details（展开步限 target 链）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5253c").unwrap().value,
        "ok",
        "(c) 无匹配 hash 不炸（top-fallback 路径保持）"
    );
}
