// part77 — html-semantics M2 切片 45（R5248，2026-10-23）：summary-less details 内
// 非激活后代点击翻转（s37 已知边界「点击 summary-less details 内非激活后代不翻转」
// 的双半边收口）。
//
// 修复双刀（shim part03/part04）：①click() 合成通道——part04 click 默认动作段新增
// nearest 上溯分支：receiver 非 LABEL/SUMMARY/DETAILS 且 `_zwFindClickActivation`
// 命中 act:details 时 `_zwDetailsActivateR4400` 翻转（spec the-details-element
// activation behaviour + concept-event-dispatch legacy-pre-activation「nearest
// ancestor (or self) with activation behavior」）；②dispatch 通道——part03 R112
// post act:details 门由 `_zwUaDispatch === true` 放宽为
// `(_zwUaDispatch === true || _zwSyntheticClick !== true)`，JS `new MouseEvent`
// dispatch（无 synthetic 标、无 UA 印记）进同款翻转。
//
// 单翻互斥：合成标使 R112 post act:details 门跳过 click() 通道（part04 内联段已翻
// ——双翻会抵消回初始态，(a)(e) 臂红钉）；dispatch 通道不走 part04 内联默认动作段
// （仅 R112 post 承接）。hasSum 门槛让位：`_zwDetailsActivateR4400` 对有 summary 的
// details no-op（summary 站优先，R4009 面不回归）；nearest 命中交互后代
// （act:button 等）时 type 非 act:details 自然豁免。
// https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element
// https://html.spec.whatwg.org/multipage/webappapis.html#event-handler-attributes
// （activation behavior 段：legacy-pre-activation behavioral steps）

/// 钉①：翻转面——(a) 静态 summary-less details 内 p.click() 经 nearest 上溯翻开；
/// (b) 再点翻回（toggle 语义 + 合成标互斥：R112 post act:details 门被跳过，若双翻
/// 则 (a) 即回初始态红）；(c) 后代 dispatch `new MouseEvent` 经 R112 post act:details
/// 放宽门翻开；(d) preventDefault 取消后 R112 消费块入口守卫拦截不翻转；(e) details
/// 本体 .click() 走 R4400 既有 DETAILS 分支翻转（nearest 分支按 receiver tag 让位，
/// 双翻则红）；(f) 动态 createElement details + 后代 click 同款翻转（R91 记账链
/// nearest 可达）。
/// 断言翻转：(a)(f) = nearest 上溯分支被撤（s37 边界回归）；(c) = R112 门放宽被撤
/// （dispatch 半边回归）；(d) = 消费块入口 preventDefault 守卫被撤（取消语义回归）；
/// (a)(e) 双翻 = 互斥标失效。
#[test]
fn summaryless_details_descendant_activation_r5248() {
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
        "<html><body><details id='d77a'><p id='d77ap'>t</p><span id='d77as'>s</span></details>\
         <details id='d77b'><span id='d77bs'>s</span></details></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s47c".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var da = document.querySelector('#d77a');\
             var dap = document.querySelector('#d77ap');\
             var db = document.querySelector('#d77b');\
             var dbs = document.querySelector('#d77bs');\
             globalThis.__g1 = String(da && dap && db && dbs && da.open === false && db.open === false);\
             dap.click();\
             globalThis.__r5248a = String(da.open === true);\
             dap.click();\
             globalThis.__r5248b = String(da.open === false);\
             dbs.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));\
             globalThis.__r5248c = String(db.open === true);\
             var cancelled = false;\
             var once = function (ev) { cancelled = true; ev.preventDefault(); };\
             dbs.addEventListener('click', once);\
             dbs.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));\
             dbs.removeEventListener('click', once);\
             globalThis.__r5248d = String(cancelled === true && db.open === true);\
             db.click();\
             globalThis.__r5248e = String(db.open === false);\
             var d8 = document.createElement('details');\
             var p8 = document.createElement('p');\
             d8.appendChild(p8);\
             document.body.appendChild(d8);\
             p8.click();\
             globalThis.__r5248f = String(d8.open === true);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__g1").unwrap().value,
        "true",
        "前提：静态树 summary-less details 初始 closed 且后代可达"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5248a").unwrap().value,
        "true",
        "(a) summary-less details 内非激活后代 p.click() 经 nearest 上溯翻转所属 details（s37 边界收口）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5248b").unwrap().value,
        "true",
        "(b) 二次 p.click() 翻回 closed（toggle 语义；单翻互斥——若 R112 post act:details 门未跳过合成通道则双翻抵消，(a) 臂同红）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5248c").unwrap().value,
        "true",
        "(c) 后代 dispatch new MouseEvent 经 R112 post act:details 放宽门翻转（dispatch 半边收口）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5248d").unwrap().value,
        "true",
        "(d) preventDefault 取消后 listener 收到事件且 R112 消费块入口守卫拦截（不翻转）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5248e").unwrap().value,
        "true",
        "(e) details 本体 .click() 走 R4400 既有 DETAILS 分支翻转（nearest 分支 receiver tag 让位，双翻则红）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5248f").unwrap().value,
        "true",
        "(f) 动态 createElement details + 后代 click 同款翻转（R91 记账链 nearest 可达）"
    );
}

/// 钉②：让位与豁免面——(a) 有 summary 的 details：非 summary 后代点击不翻转
/// （`_zwDetailsActivateR4400` hasSum 门槛让位，summary 站优先）；(b) summary.click()
/// 翻转保持（R4009 既有面）；(c) summary-less details 内交互后代（button，无 type
/// 属性——`_zwHasOwnActivationBehavior` BUTTON 判定 ty 为空/submit/reset）点击时
/// nearest 命中 act:button，type 非 act:details 自然豁免、details 不翻转。
/// 断言翻转：(a) = hasSum 门槛被撤（summary 让位面回归，R4009 互斥破坏）；
/// (b) = SUMMARY 分支被撤（R4009 回归）；(c) = BUTTON 自身 activation 判定或
/// nearest type 豁免被撤（交互后代误触 details 翻转）。
#[test]
fn summary_and_interactive_descendant_exemption_r5248() {
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
        "<html><body><details id='d77c'><summary id='d77sum'>sum</summary><p id='d77cp'>t</p></details>\
         <details id='d77d'><button id='d77btn'>b</button></details></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s47d".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var dc = document.querySelector('#d77c');\
             var dcp = document.querySelector('#d77cp');\
             var dd = document.querySelector('#d77d');\
             globalThis.__g2 = String(dc && dcp && dd && dc.open === false && dd.open === false);\
             dcp.click();\
             globalThis.__r5248g = String(dc.open === false);\
             document.querySelector('#d77sum').click();\
             globalThis.__r5248h = String(dc.open === true);\
             document.querySelector('#d77btn').click();\
             globalThis.__r5248i = String(dd.open === false);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__g2").unwrap().value,
        "true",
        "前提：有 summary 与含 button 的 details 初始均 closed"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5248g").unwrap().value,
        "true",
        "(a) 有 summary 时非 summary 后代点击不翻转（hasSum 门槛让位，summary 站优先）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5248h").unwrap().value,
        "true",
        "(b) summary.click() 翻转保持（R4009 既有面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5248i").unwrap().value,
        "true",
        "(c) 交互后代 button 点击 nearest 命中 act:button，details 不翻转（type 豁免）"
    );
}
