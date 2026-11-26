// wptfyi-css k135 刀 常驻钉测（2026-11-26）：innerHTML 物化 shadow 子元素 gCS
// 读路径＋伪元素存储槽 background 判据扩面。
//   缺口：shim `shadow.innerHTML=` 产物经 t8r `_wcRebuildAsHandle` 物化为
//   detached 双子 handle（引擎真树节点无 handle 绑定）——k134 后 handle 通道
//   `nodes.get` miss＋forward 表无条目双落空＝gCS 恒 ''（元素级与伪元素级
//   皆然；k134 已过键全走 cloneNode 产物 in-tree 注册形）。
//   修形：shim handle 臂按 `_zwNodeParent`/`_handleChildren` 随读上传单跳
//   shadow 地址（"<rootHandle>:<childOrdinal>"，s45-M1 focus 铸址同源寻址
//   原语）→ host 第 4 参 → 视图槽 root→ShadowRoot→直接子元素序解析真树
//   nid；style-system before/after/placeholder 门扩 Transparent 值探针
//   background 判据（style 级钉见 style-system kc5r135_slot_gate）。
//   E1 shadow innerHTML 子 ::selection bg 读（槽已存形，纯读路径修复面）；
//   E2 同子 ::before bg 读（Part A 门扩＋Part B 读路径复合面）；
//   E3 同子元素级读（无伪规则本体值，通道可达性面）；
//   E4 零漂移对照：createElement 直挂 body 元素（无 shadow 父＝无地址）
//      伪元素读回落本体值，旧链行为不变。
// 深链（shadow 根下嵌套元素容器）FIXME(k135+) 逐跳扩链，电池判读域。
// https://drafts.csswg.org/cssom-1/#dom-window-getcomputedstyle

/// 沙箱装具（k134 同款）：空基座，页面脚本全动态建树。
fn k135_sandbox_with_html(html: &str) -> zero_script_sandbox::V8Sandbox {
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

/// after-part 判据页 run_part_test 页形：createElement 容器＋attachShadow＋
/// shadow.innerHTML part 元素＋JS 注入规则（innerHTML 物化＝detached 双子缺口形）。
const K135_SETUP_JS: &str = r#"
        var c = document.createElement('div');
        var sr = c.attachShadow({mode:'open'});
        sr.innerHTML = "<span part='p'></span><slot></slot>";
        document.body.appendChild(c);
        var st = document.createElement('style');
        st.innerText = "::part(p)::before { background: rgb(1, 2, 3); } ::part(p)::selection { background: rgb(4, 5, 6); }";
        document.head.appendChild(st);
    "#;

/// E1：shadow innerHTML 子 `::selection` bg 读——selection 槽 style 级已存
///（bg 判据 R5065 先例），本钉主验读路径地址解析（k134 后恒 '' 的缺口面）。
#[test]
fn test_kc5r135_shadow_innerhtml_selection_read() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k135_sandbox_with_html("<html><head></head><body></body></html>");
    let r = sb
        .execute(&format!(
            r#"
        {K135_SETUP_JS}
        getComputedStyle(sr.firstElementChild, '::selection').backgroundColor
        "#
        ))
        .unwrap()
        .value;
    assert_eq!(r, "rgb(4, 5, 6)", "E1 shadow innerHTML child ::selection reads slot, got {r:?}");
}

/// E2：同子 `::before` bg 读——Part A 门扩（style-system bg-only 存槽）＋
/// Part B 读路径复合面（k135 刀两半缺一即红）。
#[test]
fn test_kc5r135_shadow_innerhtml_before_read() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k135_sandbox_with_html("<html><head></head><body></body></html>");
    let r = sb
        .execute(&format!(
            r#"
        {K135_SETUP_JS}
        getComputedStyle(sr.firstElementChild, '::before').backgroundColor
        "#
        ))
        .unwrap()
        .value;
    assert_eq!(r, "rgb(1, 2, 3)", "E2 shadow innerHTML child ::before reads slot, got {r:?}");
}

/// E3：同子元素级读（无伪规则本体值）——地址解析不限伪元素投影臂，
/// 通道可达性面（k134 后元素级亦恒 ''）。
#[test]
fn test_kc5r135_shadow_innerhtml_own_read() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k135_sandbox_with_html("<html><head></head><body></body></html>");
    let r = sb
        .execute(&format!(
            r#"
        {K135_SETUP_JS}
        getComputedStyle(sr.firstElementChild).backgroundColor
        "#
        ))
        .unwrap()
        .value;
    assert_eq!(r, "rgba(0, 0, 0, 0)", "E3 own read reachable via addr, got {r:?}");
}

/// E4：零漂移对照——createElement 直挂 body 元素（无 shadow 父＝不产地址）
/// `::selection` 读回落本体值（透明），旧链行为不变。
#[test]
fn test_kc5r135_plain_child_no_addr_undrifted() {
    use zero_script_sandbox::Sandbox;
    let mut sb = k135_sandbox_with_html("<html><head></head><body></body></html>");
    let r = sb
        .execute(
            r#"
        var d = document.createElement('div');
        document.body.appendChild(d);
        getComputedStyle(d, '::selection').backgroundColor
        "#,
        )
        .unwrap()
        .value;
    assert_eq!(r, "rgba(0, 0, 0, 0)", "E4 no-addr plain child falls back to own value, got {r:?}");
}
