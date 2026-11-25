// wptfyi-css k134 刀 常驻钉测（2026-11-26）：gCS 第二参 handle 通道透传——
// 动态元素（createElement 产物，handle 身份）伪元素读形经 shim handle 臂
// 第三参透传 → host `__zw_get_computed_style_by_handle` 归一分派 → 视图
// ComputedStyle 伪元素存储槽投影（CSSOM-1 §dom-window-getcomputedstyle，
// k133 selector 通道透传的 handle 臂镜像）。
//   K1 handle 通道五槽投影：doc 表 `div::before` 规则 + 动态 div →
//      gCS(el,'::before').color 投影槽值（视图级联含伪元素槽计算）；
//   K1b 零漂移对照：同元素本体读不带第二参 → 本体值不变；
//   K2 静态 part 五槽 selector 通道沙箱端到端：doc 表 `::part()::after`
//      胜 shadow 内常规规则（k133 树上下文轴）——shim 白名单→host 归一→
//      投影全链（k133 钉测为直接 API 形，本钉补 shim 级接线）；
//   K3 槽 miss 宽容面：`::marker` 非 li 槽 None → originating 元素自身
//      计算值（css-pseudo-4 §2，values 域第三参读形键依赖的 shim 级钉）。
// shadow 树内动态 part 元素（interaction-with-pseudo-elements runner 形）的
// handle→forward-map→shadow 查找链由电池判读（裸沙箱 view 物化不含生产
// drain 面，沙箱级不可复现）。
// https://drafts.csswg.org/cssom-1/#dom-window-getcomputedstyle

/// 沙箱装具（part56 同款）：base html 携静态规则，JS 动态建元素。
fn k134_sandbox_with_html(html: &str) -> zero_script_sandbox::V8Sandbox {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::V8Sandbox;
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    sandbox
}

const K134_DIV_BEFORE_HTML: &str = "<html><head><style>div::before { color: rgb(1, 2, 3); content: \"x\"; }</style>\
</head><body></body></html>";

/// K1：动态元素（handle 通道）`::before` 读形投影槽值。
#[test]
fn test_kc5r134_handle_pseudo_projection() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k134_sandbox_with_html(K134_DIV_BEFORE_HTML);
    let r = sb
        .execute(
            r#"
        var d = document.createElement('div');
        document.body.appendChild(d);
        getComputedStyle(d, '::before').color
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(r, "rgb(1, 2, 3)", "K1 handle-channel ::before reads slot value, got {r:?}");
}

/// K1b：同元素本体读（不带第二参）零漂移——本体色不受伪元素规则影响。
#[test]
fn test_kc5r134_handle_own_read_undrifted() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k134_sandbox_with_html(K134_DIV_BEFORE_HTML);
    let r = sb
        .execute(
            r#"
        var d = document.createElement('div');
        document.body.appendChild(d);
        getComputedStyle(d).color
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(r, "rgb(0, 0, 0)", "K1b own read stays default, got {r:?}");
}

const K134_PART_AFTER_HTML: &str = "<html><head><style>\
#h::part(ab-p)::after { color: rgb(4, 5, 6); content: \"y\"; }</style></head><body>\
<div id=\"h\"><template shadowrootmode=\"open\">\
<style>#pa { color: rgb(7, 8, 9); } #pa::after { content: \"z\"; color: rgb(255, 0, 0); }</style>\
<span id=\"pa\" part=\"ab-p\">x</span></template></div></body></html>";

/// K2：静态 part 五槽 selector 通道沙箱端到端——doc 表 `::part(ab-p)::after`
/// 胜 shadow 内 `#pa::after` 常规规则（k133 树上下文轴 shim 级钉）。
#[test]
fn test_kc5r134_part_after_selector_channel() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k134_sandbox_with_html(K134_PART_AFTER_HTML);
    let r = sb
        .execute(
            r#"
        var el = document.getElementById('h').shadowRoot.getElementById('pa');
        el ? getComputedStyle(el, '::after').color : 'NO-EL'
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(r, "rgb(4, 5, 6)", "K2 doc-table part ::after wins, got {r:?}");
}

/// K3：槽 miss 宽容面——`::marker` 非 li 槽 None → originating 元素自身
/// 计算值（元素色 rgb(7, 8, 9)），非默认/非空。
#[test]
fn test_kc5r134_marker_slot_miss_fallback() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k134_sandbox_with_html(K134_PART_AFTER_HTML);
    let r = sb
        .execute(
            r#"
        var el = document.getElementById('h').shadowRoot.getElementById('pa');
        el ? getComputedStyle(el, '::marker').color : 'NO-EL'
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(r, "rgb(7, 8, 9)", "K3 marker slot miss falls back to element, got {r:?}");
}
