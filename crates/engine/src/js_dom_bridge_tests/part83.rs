// part83 — html-semantics M2 切片 52（R5257/R5258，2026-10-24）：focus 滚动进视口 +
// 视口 scroll 事件 document 级派发 + Element.moveBefore。
//
// spec focusing steps 尾 UA 滚动面（CSSOM View scroll a target into view）：
// 程序 focus() 于视口外元素 → nearest 最小滚动 + scroll 事件（WPT
// focus-after-close「outViewPortButton.focus() 后 document 收 scroll 且
// scrollTop>0」断言面——此前 scroll 永挂文件级 Timeout）。no-scroll 变体：
// dialog 内部 focusing steps（show/showModal 聚焦、close-the-dialog 焦点恢复、
// inert fixup）不滚视口（同测试「close 后 scrollTop==0」断言面）。视口 scroll
// 事件 target 是 Document（CSSOM View）——document 监听可收且 window 监听不双计
//（非冒泡 bubble 腿 bubbles 门控）。Element.moveBefore（DOM standard）：免
// removal steps 的树内移动——dialog open 态与 close watcher 组栈原地保持
//（dialog-open-movebefore-setup 断言面：moveBefore 后 ESC 仍关组栈顶
// second_dialog）；child 非本父 → NotFoundError。
// https://html.spec.whatwg.org/multipage/interaction.html#focusing-steps
// https://drafts.csswg.org/cssom-view/#scrolling
// https://dom.spec.whatwg.org/#dom-parentnode-movebefore

/// 钉：(a) 视口外元素 focus() → 窗口滚动 nearest 位 + document 恰收一次 scroll +
/// window 恰收一次 scroll（无双计）+ documentElement.scrollTop 窗口镜像；
/// (b) dialog showModal/close 内部 focusing steps 全程零滚动 + close 恢复原焦点；
/// (c) moveBefore(dialog, null) → 归属新父、open 态双保、ESC 关组栈顶 second 而
/// first 保持；(d) refNode 非本父子 → NotFoundError；(e) 计数收口：全程仅 (a)
/// 一对 scroll 事件。时序模拟：记录式 `__zw_setTimeout` stub + 手动 `__zw_fire`
///（runner/browser 宿主为真 timer）；几何 stub gBCR（register_dom_callbacks 注册
/// 的进程内实现后覆写）。
#[test]
fn focus_scroll_into_view_movebefore_r5257() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // 记录式 host timer stub（part81/82 契约同款）——task 登记、手动驱动。
    sandbox
        .execute(
            "globalThis.__zw_setTimeout = function (id, delay) {\
               globalThis.__zw_armed.push(id); return id; };\
             globalThis.__zw_armed = [];\
             globalThis.__zw_fire = function () {\
               var armed = globalThis.__zw_armed; globalThis.__zw_armed = [];\
               for (var i = 0; i < armed.length; i++) {\
                 globalThis.__zwResolveCallback(armed[i], ''); } };",
        )
        .unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"np\"></div>\
         <dialog id=\"d1\" closedby=\"closerequest\"></dialog>\
         <dialog id=\"d2\" closedby=\"closerequest\"></dialog>\
         <button id=\"ob\">ob</button></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s52".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 几何 stub：register_dom_callbacks 的进程内 gBCR 后覆写——ob 位于视口
    //（innerHeight=800，part01 默认）下方 1200px 处、高 40。nearest 最小滚动期望
    // 底贴齐：scrollTo(0, 1200+40-800=440)。
    sandbox
        .execute(
            "globalThis.__zw_getBoundingClientRect = function () { return '0,1200,100,40'; };\
             globalThis.__docN = 0; globalThis.__winN = 0;\
             document.addEventListener('scroll', function () { globalThis.__docN++; });\
             window.addEventListener('scroll', function () { globalThis.__winN++; });\
             // (a) 视口外 focus → nearest 滚动 + 双通道恰一次 scroll。
             var ob = document.getElementById('ob');\
             ob.focus();\
             globalThis.__aScrollY = window.scrollY;\
             globalThis.__aDet = document.documentElement.scrollTop;\
             // (b) dialog 内部 focusing steps 零滚动（no-scroll 变体）。
             var d1 = document.getElementById('d1');\
             d1.showModal();\
             globalThis.__bAfterShow = window.scrollY;\
             d1.close();\
             globalThis.__bActive = String(document.activeElement === ob);\
             globalThis.__bScrollY = window.scrollY;\
             // (c) moveBefore：免 removal steps——open 态与组栈序原地保持。
             var d2 = document.getElementById('d2');\
             var np = document.getElementById('np');\
             d1.show();\
             d2.show();\
             np.moveBefore(d1, null);\
             globalThis.__cParent = String(d1.parentNode === np);\
             globalThis.__cOpen = String(d1.open) + ',' + String(d2.open);\
             globalThis.__zw_esc_dialog_cancel();\
             globalThis.__zw_fire();\
             globalThis.__cEsc = String(d1.open) + ',' + String(d2.open);\
             // (d) refNode 非本父子 → NotFoundError（ob.parent 是 body 非 np）。
             var _mbErr = '';\
             try { np.moveBefore(ob, ob); } catch (e) { _mbErr = e.name; }\
             globalThis.__dErr = _mbErr;",
        )
        .unwrap();

    // (a) nearest 滚动落位 + 双通道恰一次。
    assert_eq!(
        sandbox.execute("String(globalThis.__aScrollY)").unwrap().value,
        "440",
        "(a) focus 出界元素 → 底贴齐 nearest 滚动位"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__aDet)").unwrap().value,
        "440",
        "(a) documentElement.scrollTop 窗口镜像"
    );
    // (b) dialog 内部路径零滚动 + close 恢复原焦点。
    assert_eq!(
        sandbox.execute("String(globalThis.__bAfterShow)").unwrap().value,
        "440",
        "(b) showModal focusing steps 零滚动"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__bActive)").unwrap().value,
        "true",
        "(b) close 恢复 previously focused"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__bScrollY)").unwrap().value,
        "440",
        "(b) close 焦点恢复不滚（scrollTop 保持）"
    );
    // (c) moveBefore 后归属/open 双保；ESC 关组栈顶 second。
    assert_eq!(
        sandbox.execute("globalThis.__cParent").unwrap().value,
        "true",
        "(c) moveBefore 归属新父"
    );
    assert_eq!(
        sandbox.execute("globalThis.__cOpen").unwrap().value,
        "true,true",
        "(c) moveBefore 后双 dialog open 原地保持"
    );
    assert_eq!(
        sandbox.execute("globalThis.__cEsc").unwrap().value,
        "true,false",
        "(c) ESC 关组栈顶 second_dialog、first 保持"
    );
    // (d) spec 校验面。
    assert_eq!(
        sandbox.execute("globalThis.__dErr").unwrap().value,
        "NotFoundError",
        "(d) refNode 非本父子 → NotFoundError"
    );
    // (e) 全程仅 (a) 一对 scroll 事件（document/window 各恰一次）。
    assert_eq!(
        sandbox.execute("String(globalThis.__docN)").unwrap().value,
        "1",
        "(e) document 恰收一次视口 scroll"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__winN)").unwrap().value,
        "1",
        "(e) window 恰收一次视口 scroll（doc 派发不双计）"
    );
}
