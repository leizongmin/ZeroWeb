// part74 — html-semantics M2 切片 42（R5245，2026-10-23）：close-the-dialog 组栈
// 卫生——模态/非模态一律同步出栈（spec close watcher 随 dialog 关闭销毁、其组离栈）。
// R4300 只在模态臂 splice，modeless close() 残留 stale 项（s23 挂账「非模态 close
// 不出组栈——读侧 purge 兜底」收口）。栈内部态无 JS 观测面，本契约钉行为不变量：
// show/close 循环 + 栈序 + 全关 no-op + 模态臂保持。
// https://html.spec.whatwg.org/multipage/interactive-elements.html#set-the-dialog-close-watcher

/// 断言反转面：splice 误伤活项/漏项 → R1/R2 栈序目标错位红；模态臂回退 → R4 红；
/// 出栈误杀未开组 → R3 假阳性红。
#[test]
fn test_dialog_group_stack_hygiene_r5245() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // R1：modeless show/close ×3 循环（旧实现每轮残留一个 stale 项）后重开——ESC 照常
    // 命中尾组（closedby=any），cancel 事件照常派发，close() 后栈无残留干扰下轮。
    sandbox
        .execute(
            "var d1 = document.createElement('dialog'); d1.setAttribute('closedby', 'any'); document.body.appendChild(d1); globalThis.__hD1 = d1; globalThis.__hSeq = []; d1.addEventListener('cancel', function () { globalThis.__hSeq.push('cancel'); }); d1.addEventListener('close', function () { globalThis.__hSeq.push('close'); }); d1.show(); d1.close(); d1.show(); d1.close(); d1.show(); d1.close(); d1.show();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("globalThis.__hSeq = []; var r = __zw_esc_dialog_cancel() + ',' + Number(globalThis.__hD1.hasAttribute('open')) + ',' + globalThis.__hSeq.join('|'); globalThis.__hD1.close(); r")
            .unwrap()
            .value,
        "true,0,cancel",
        "R1：show/close 三循环后重开，ESC 照常命中尾组（cancel 同步派发）"
    );

    // R2：栈序——d1/d2 依次 show（d2 尾组），首次 ESC 关 d2、再次关 d1（栈顶优先序）。
    sandbox
        .execute(
            "var d1 = document.createElement('dialog'); d1.setAttribute('closedby', 'any'); document.body.appendChild(d1); var d2 = document.createElement('dialog'); d2.setAttribute('closedby', 'any'); document.body.appendChild(d2); globalThis.__hD1 = d1; globalThis.__hD2 = d2; d1.show(); d2.show(); globalThis.__hR2a = __zw_esc_dialog_cancel() + ',' + Number(d2.hasAttribute('open')) + ',' + Number(d1.hasAttribute('open')); globalThis.__hR2b = __zw_esc_dialog_cancel() + ',' + Number(d2.hasAttribute('open')) + ',' + Number(d1.hasAttribute('open'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hR2a + ';' + globalThis.__hR2b").unwrap().value,
        "true,0,1;true,0,0",
        "R2：双 modeless 栈序——ESC 先关尾组 d2 再关 d1"
    );

    // R3：全关后 ESC no-op（返 false——无活组可命中，出栈不产生假阳性）。
    assert_eq!(
        sandbox.execute("String(__zw_esc_dialog_cancel())").unwrap().value,
        "false",
        "R3：全关后 ESC 无命中"
    );

    // R4：模态臂保持——showModal 入栈、close() 出栈（模态态三件套齐清），重开照常。
    sandbox
        .execute(
            "var d3 = document.createElement('dialog'); d3.setAttribute('closedby', 'any'); document.body.appendChild(d3); globalThis.__hD3 = d3; d3.showModal(); globalThis.__hR4a = __zw_esc_dialog_cancel() + ',' + Number(d3.hasAttribute('open')); d3.showModal(); globalThis.__hR4b = Number(d3.hasAttribute('open')); d3.close(); globalThis.__hR4c = __zw_esc_dialog_cancel();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__hR4a + ';' + globalThis.__hR4b + ';' + globalThis.__hR4c").unwrap().value,
        "true,0;1;false",
        "R4：模态 close() 出栈后重开照常，二次 close 后 ESC no-op"
    );
}
