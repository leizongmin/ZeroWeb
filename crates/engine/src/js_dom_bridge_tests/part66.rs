// js_dom_bridge_tests part66（R4600）——dialog focusing steps 家族契约面。spec
// 「dialog focusing steps」（https://html.spec.whatwg.org/multipage/interactive-elements.html
// #dialog-focusing-steps）：show()/showModal() 加 open 属性后 Set previously focused
// element = focused element → control = subject 有 autofocus 属性 ? subject :
// focus delegate ?? subject → 对 control 跑 focusing steps。focus delegate 委派链
//（https://html.spec.whatwg.org/multipage/interaction.html#focus-delegate）：
// autofocus delegate → 树序后代（dialog 目标 → sequentially focusable，跳 tabindex=-1；
// 否则 focusable area；delegatesFocus shadow host 递归下探）。close-the-dialog
// previously focused 恢复尾步 + showModal 未连接 InvalidStateError（页面 global
// DOMException 构造）+ show() 断开不抛。WPT 受害面：dialog-autofocus /
// show-modal-focusing-steps / child-sequential-focus / dialog-focus-shadow(50F) /
// focus-after-close / dialog-focusing-steps-inert / dialog-focusing-steps-disconnected。

#[test]
fn test_dialog_focusing_steps_r4600() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body><input id='pf'></body></html>".to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // R1：autofocus 属性在 subject → control = dialog 自身（子级 autofocus 不夺——
    // child-sequential-focus autofocusdialog 断言面）。
    sandbox
        .execute(
            "var d1 = document.createElement('dialog'); d1.setAttribute('autofocus', ''); \
             var b1 = document.createElement('button'); b1.setAttribute('autofocus', ''); d1.appendChild(b1); \
             document.body.appendChild(d1); d1.showModal(); \
             globalThis.__r1 = document.activeElement === d1 ? 'dialog' : String(document.activeElement && document.activeElement.tagName);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r1").unwrap().value,
        "dialog",
        "R1：subject autofocus → 聚焦 dialog 本身"
    );
    sandbox.execute("d1.close();").unwrap();

    // R2：无 autofocus → 首个 sequentially focusable 后代；tabindex=-1 跳过
    //（child-sequential-focus keyboardfocusdialog 面）。
    sandbox
        .execute(
            "var d2 = document.createElement('dialog'); \
             var m1 = document.createElement('button'); m1.setAttribute('tabindex', '-1'); d2.appendChild(m1); \
             var m2 = document.createElement('button'); m2.setAttribute('tabindex', '0'); d2.appendChild(m2); \
             document.body.appendChild(d2); d2.showModal(); \
             globalThis.__r2 = document.activeElement === m2 ? 'second' \
               : (document.activeElement === m1 ? 'first' : 'other');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r2").unwrap().value,
        "second",
        "R2：tabindex=-1 跳过 → 首 sequential focusable 胜"
    );
    sandbox.execute("d2.close();").unwrap();

    // R3：全灭（hidden）→ 兜底聚焦 dialog 本身（show-modal-focusing-steps 尾臂）。
    sandbox
        .execute(
            "var d3 = document.createElement('dialog'); \
             var h1 = document.createElement('button'); h1.setAttribute('hidden', ''); d3.appendChild(h1); \
             document.body.appendChild(d3); d3.showModal(); \
             globalThis.__r3 = document.activeElement === d3 ? 'dialog' : 'other';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3").unwrap().value,
        "dialog",
        "R3：无可聚焦后代 → 兜底聚焦 dialog"
    );
    sandbox.execute("d3.close();").unwrap();

    // R4：autofocus delegate 跳过面——[autofocus disabled]/[autofocus hidden]/嵌套
    // closed dialog 内 [autofocus] 全跳；真 [autofocus] 胜（dialog-autofocus 面）。
    sandbox
        .execute(
            "var d4 = document.createElement('dialog'); \
             var a1 = document.createElement('input'); a1.setAttribute('autofocus', ''); a1.disabled = true; d4.appendChild(a1); \
             var a2 = document.createElement('textarea'); a2.setAttribute('autofocus', ''); a2.setAttribute('hidden', ''); d4.appendChild(a2); \
             var nd = document.createElement('dialog'); var nb = document.createElement('button'); nb.setAttribute('autofocus', ''); nd.appendChild(nb); d4.appendChild(nd); \
             var a3 = document.createElement('button'); a3.setAttribute('autofocus', ''); d4.appendChild(a3); \
             document.body.appendChild(d4); d4.showModal(); \
             globalThis.__r4 = document.activeElement === a3 ? 'real' : (document.activeElement === nb ? 'nested' : 'other');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r4").unwrap().value,
        "real",
        "R4：disabled/hidden/closed-dialog 子树 autofocus 全跳过"
    );
    sandbox.execute("d4.close();").unwrap();

    // R5：show() 断开不抛（spec show 步无未连接检查——toggle-events 断言面）：加 open，
    // toggle 异步排队跨 execute 到达；showModal() 断开抛 InvalidStateError 且
    // constructor 为页面 DOMException（wrong-global 断言面——
    // dialog-focusing-steps-disconnected）。
    sandbox
        .execute(
            "var d5 = document.createElement('dialog'); \
             globalThis.__r5 = []; \
             d5.addEventListener('toggle', function () { globalThis.__r5.push('toggle'); }); \
             try { d5.show(); globalThis.__r5.push(String(d5.hasAttribute('open'))); } catch (e) { globalThis.__r5.push('throw:' + e.name); } \
             var d5b = document.createElement('dialog'); \
             try { d5b.showModal(); globalThis.__r5.push('no-throw'); } catch (e) { \
               globalThis.__r5.push('throw:' + e.name + ':' + (e.constructor === globalThis.DOMException)); }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5.join('|')").unwrap().value,
        "true|throw:InvalidStateError:true|toggle",
        "R5：断开 show() 开启（toggle 异步补达）；showModal() 抛页面 DOMException"
    );

    // R6：close 恢复 prevFocused——非模态（show）与模态（showModal）双臂；
    // 「moved in between」移动后仍恢复（focus-after-close 面）。
    sandbox
        .execute(
            "globalThis.__r6 = []; \
             var pf = document.getElementById('pf'); \
             var d6 = document.createElement('dialog'); \
             var c1 = document.createElement('button'); d6.appendChild(c1); document.body.appendChild(d6); \
             pf.focus(); d6.show(); d6.close(); \
             globalThis.__r6.push(document.activeElement === pf ? 'show-ok' : 'show-miss'); \
             pf.focus(); d6.showModal(); d6.close(); \
             globalThis.__r6.push(document.activeElement === pf ? 'modal-ok' : 'modal-miss'); \
             pf.focus(); d6.showModal(); document.body.appendChild(pf); d6.close(); \
             globalThis.__r6.push(document.activeElement === pf ? 'moved-ok' : 'moved-miss');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r6.join('|')").unwrap().value,
        "show-ok|modal-ok|moved-ok",
        "R6：close 恢复 prevFocused（show/showModal/移动后）"
    );

    // R7：prevFocused 已移除 → focusing steps no-op → activeElement 回落 body
    //（focus-after-close move_to_body 面）。
    sandbox
        .execute(
            "var pf7 = document.createElement('input'); document.body.appendChild(pf7); \
             var d7 = document.createElement('dialog'); document.body.appendChild(d7); \
             pf7.focus(); d7.showModal(); d7.close(); pf7.remove(); \
             globalThis.__r7 = document.activeElement === document.body ? 'body' : 'other';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r7").unwrap().value,
        "body",
        "R7：prevFocused 移除 → activeElement 回落 body"
    );

    // R8：delegatesFocus 影子委派——焦点落 shadow 树内 [autofocus] →
    // document.activeElement retarget 到 host + shadowRoot.activeElement = 内层锚
    //（dialog-focus-shadow 50F 主断言面）；delegatesFocus=false → 兜底 dialog。
    sandbox
        .execute(
            "globalThis.__r8 = []; \
             var d8 = document.createElement('dialog'); document.body.appendChild(d8); \
             var host = document.createElement('div'); d8.appendChild(host); \
             var sr = host.attachShadow({ mode: 'open', delegatesFocus: true }); \
             var sb = document.createElement('button'); sb.setAttribute('autofocus', ''); sr.appendChild(sb); \
             d8.showModal(); \
             globalThis.__r8.push(document.activeElement === host ? 'host' : 'doc-miss'); \
             globalThis.__r8.push(sr.activeElement === sb ? 'inner' : 'sr-miss'); \
             d8.close(); \
             var d8b = document.createElement('dialog'); document.body.appendChild(d8b); \
             var hostB = document.createElement('div'); d8b.appendChild(hostB); \
             var srB = hostB.attachShadow({ mode: 'open', delegatesFocus: false }); \
             var sbB = document.createElement('button'); sbB.setAttribute('autofocus', ''); srB.appendChild(sbB); \
             d8b.showModal(); \
             globalThis.__r8.push(document.activeElement === d8b ? 'fallback-dialog' : 'fallback-miss'); \
             d8b.close();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r8.join('|')").unwrap().value,
        "host|inner|fallback-dialog",
        "R8：delegatesFocus true 委派 shadow 内锚 + false 兜底 dialog"
    );

    // R9：焦点恢复目标已移入 shadow root → 恢复到内层锚，document.activeElement =
    // host（focus-after-close move_to_shadow_root 面）。
    sandbox
        .execute(
            "var pf9 = document.createElement('input'); document.body.appendChild(pf9); \
             var d9 = document.createElement('dialog'); document.body.appendChild(d9); \
             var h9 = document.createElement('div'); document.body.appendChild(h9); \
             var sr9 = h9.attachShadow({ mode: 'open' }); \
             pf9.focus(); d9.showModal(); sr9.appendChild(pf9); d9.close(); \
             globalThis.__r9 = (sr9.activeElement === pf9 && document.activeElement === h9) ? 'ok' : 'miss';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r9").unwrap().value,
        "ok",
        "R9：prevFocused 移入 shadow → 恢复 + host retarget"
    );

    // R10：inert 子元素不作为 focusable area（dialog-focusing-steps-inert 面）——
    // 跳过 inert 候选兜底落 dialog。
    sandbox
        .execute(
            "var d10 = document.createElement('dialog'); document.body.appendChild(d10); \
             var i10 = document.createElement('button'); i10.setAttribute('inert', ''); d10.appendChild(i10); \
             d10.showModal(); \
             globalThis.__r10 = document.activeElement === d10 ? 'dialog' : 'other';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r10").unwrap().value,
        "dialog",
        "R10：inert 候选跳过 → 兜底 dialog"
    );
    sandbox.execute("d10.close();").unwrap();

    // R11：已开非模态再 showModal → 抛页面 DOMException（dialog-no-throw 反向臂：
    // 非模态已开时 showModal 的 InvalidStateError 身份）。
    sandbox
        .execute(
            "var d11 = document.createElement('dialog'); document.body.appendChild(d11); \
             d11.show(); globalThis.__r11 = 'open'; \
             try { d11.showModal(); globalThis.__r11 += '|no-throw'; } catch (e) { \
               globalThis.__r11 += '|throw:' + e.name + ':' + (e.constructor === globalThis.DOMException); }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r11").unwrap().value,
        "open|throw:InvalidStateError:true",
        "R11：非模态已开 showModal 抛页面 DOMException"
    );

    // R12：subject inert 门——inert IDL 属性反射 attribute（part04/part05 布尔反射）；
    // 非模态 show() 焦点不变；模态 showModal() focus fixup rule 落 body
    //（dialog-focusing-steps-inert 双臂断言面）。
    sandbox
        .execute(
            "globalThis.__r12 = []; \
             var pf12 = document.createElement('input'); document.body.appendChild(pf12); \
             var d12 = document.createElement('dialog'); \
             var i12 = document.createElement('input'); i12.setAttribute('autofocus', ''); d12.appendChild(i12); \
             document.body.appendChild(d12); \
             pf12.focus(); d12.inert = true; d12.show(); \
             globalThis.__r12.push(document.activeElement === pf12 ? 'show-keep' : 'show-miss'); \
             d12.close(); d12.showModal(); \
             globalThis.__r12.push(document.activeElement === document.body ? 'modal-body' : 'modal-miss'); \
             d12.close(); d12.inert = false;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r12.join('|')").unwrap().value,
        "show-keep|modal-body",
        "R12：inert 非模态焦点不变 + 模态 fixup 落 body"
    );

    // R13：subject 未连接 → focusing steps 跳过，焦点不变
    //（dialog-focusing-steps-disconnected show() 臂断言面；R5 已锁 showModal 抛）。
    sandbox
        .execute(
            "var pf13 = document.createElement('input'); document.body.appendChild(pf13); \
             var d13 = document.createElement('dialog'); \
             var i13 = document.createElement('input'); i13.setAttribute('autofocus', ''); d13.appendChild(i13); \
             pf13.focus(); d13.show(); \
             globalThis.__r13 = document.activeElement === pf13 ? 'keep' : 'miss';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r13").unwrap().value,
        "keep",
        "R13：断开 dialog show() 焦点不变"
    );
}

// s28（2026-10-23）——模态 inertness 焦点切面契约：栈顶 open+模态 dialog 的
// shadow-including 子树外元素经 _zwIsFocusableSel 拒焦（宿主 pre-focus 步与
// mousedown 默认动作共用的焦点门）。dialog-closedby-corner-cases「focusin calls
// showModal, modal」light dismiss 案根因面：pre-focus 抢跑把焦点漏到模态外 →
// closing steps restore no-op → 无 focusin → 处理器 re-entrant showModal 不触发
// → 端态 closed。真 Chrome 基准 29/30（runner 修复后同 29/30，唯余 esc-modeless
// 上游失败，Chrome 实测同款）。仅栈顶模态生效（stacked modal 内层子树可聚焦）。
// 元素全部落在初始 dom_html（变更追加节点的 selector 串解析不在本契约面内）。
// https://html.spec.whatwg.org/multipage/interactive-elements.html#modal-dialogs
#[test]
fn test_dialog_modal_focus_gate_s28() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body>\
          <button id='g1b'></button>\
          <dialog id='g1d'><button id='g2b'></button><button id='g2c'></button></dialog>\
          <button id='g4x'></button>\
          <dialog id='g4o'></dialog>\
          <dialog id='g4i'><button id='g4b'></button><button id='g4c'></button></dialog>\
        </body></html>"
            .to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // G1：模态开 → 模态外按钮 _zwFocusSel 拒焦（焦点保持在模态子树内；真 Chrome
    // mousedown hit-test 跳过模态外元素的同源行为，focusable-area 切片）。
    // showModal 后 dialog 无 autofocus → 首个 focusable 后代 g2b 获焦。
    sandbox
        .execute(
            "var g1d = document.querySelector('#g1d'); \
             var g1b = document.querySelector('#g1b'); \
             var g2b = document.querySelector('#g2b'); \
             g1d.showModal(); \
             globalThis.__g1pre = document.activeElement === g2b ? 'g2b' : 'other'; \
             globalThis._zwFocusSel('#g1b'); \
             globalThis.__g1 = document.activeElement === g1b ? 'leak' : 'kept';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__g1pre").unwrap().value,
        "g2b",
        "G1 前置：showModal 聚焦首个 focusable 后代"
    );
    assert_eq!(
        sandbox.execute("globalThis.__g1").unwrap().value,
        "kept",
        "G1：模态子树外 pre-focus 被门拒"
    );

    // G2：模态子树内元素仍可聚焦（focusable area 不受本门影响）。
    sandbox
        .execute(
            "globalThis._zwFocusSel('#g2c'); \
             globalThis.__g2 = document.activeElement === document.querySelector('#g2c') ? 'in' : 'blocked';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__g2").unwrap().value,
        "in",
        "G2：模态子树内可聚焦"
    );

    // G3：close 后模态外恢复可聚焦（closing steps restore 把焦点交还外部元素的
    // 依赖面——corner-cases click-modal 案的 fc:button 触发前提）。
    sandbox
        .execute(
            "g1d.close(); \
             globalThis._zwFocusSel('#g1b'); \
             globalThis.__g3 = document.activeElement === g1b ? 'restored' : 'stuck';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__g3").unwrap().value,
        "restored",
        "G3：模态关闭后模态外恢复可聚焦"
    );

    // G4：stacked modal——仅栈顶模态生效：外层模态外的元素拒焦，栈顶（内层）
    // 模态子树内仍可聚焦（Chromium 同款）。
    sandbox
        .execute(
            "var g4o = document.querySelector('#g4o'); \
             var g4i = document.querySelector('#g4i'); \
             var g4b = document.querySelector('#g4b'); \
             g4o.showModal(); g4i.showModal(); \
             globalThis.__g4pre = document.activeElement === g4b ? 'g4b' : 'other'; \
             globalThis._zwFocusSel('#g4x'); \
             globalThis.__g4a = document.activeElement === document.querySelector('#g4x') ? 'leak' : 'kept'; \
             globalThis._zwFocusSel('#g4c'); \
             globalThis.__g4b = document.activeElement === document.querySelector('#g4c') ? 'in' : 'blocked';",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__g4pre").unwrap().value,
        "g4b",
        "G4 前置：栈顶模态 focusing steps 聚焦其后代"
    );
    assert_eq!(
        sandbox.execute("globalThis.__g4a").unwrap().value,
        "kept",
        "G4a：栈顶模态子树外拒焦"
    );
    assert_eq!(
        sandbox.execute("globalThis.__g4b").unwrap().value,
        "in",
        "G4b：栈顶模态子树内可聚焦"
    );
    sandbox.execute("g4i.close(); g4o.close();").unwrap();
}
