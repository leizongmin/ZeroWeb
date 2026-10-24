// part82 — html-semantics M2 切片 51（R5256，2026-10-24）：dialog open 属性移除面
// 关闭步 + detached 文档 dialog 三方法。
//
// spec the-dialog-element attribute change steps（open 移除）→ dialog closing
// algorithm：close/toggle 事件排队、无 cancel、无 beforetoggle、焦点不恢复、
// returnValue 不变（dialog-close-via-attribute.tentative 断言契约； shim 旧注记
// 「直接 removeAttribute 不派 close」已被 2025 规范变更推翻）。detached 文档
//（无 browsing context）的 dialog 产物：showModal() 抛 InvalidStateError
//（dialog-active-document——旧产物无方法致 TypeError 非 DOMException）、show()
// 静默返、close() no-op。
// https://html.spec.whatwg.org/multipage/interactive-elements.html#the-dialog-element

/// 钉：(a) 模态 dialog removeAttribute('open') —— 立即非 :modal、close 事件异步
/// 恰一次、cancel 零次、toggle 'open->closed' 恰一次（分腿驱动下不合并）；
/// (b) IDL `open = false` 同面；(c) 已 closed dialog 再移除零事件（wasOpen 门）；
/// (d) 同任务 showModal + 移除 → toggle 合并单发（R4200 去重语义）；
/// (e) detached 文档 dialog showModal 抛 InvalidStateError + matches('[open]')
/// false + show/close 不抛。时序模拟：记录式 `__zw_setTimeout` stub + 手动
/// `__zw_fire`（runner/browser 宿主为真 timer）。
#[test]
fn dialog_attr_removal_closing_r5256() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // 记录式 host timer stub（part81 契约同款）——task 登记、手动驱动。
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
        "<html><body><button id=\"b1\">b</button>\
         <dialog id=\"dg\"><button id=\"db\">in</button></dialog></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s51".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var dg = document.getElementById('dg');\
             globalThis.__rc = 0; globalThis.__cc = 0; globalThis.__tc = 0;\
             globalThis.__tst = '';\
             dg.addEventListener('close', function () { globalThis.__rc++; });\
             dg.addEventListener('cancel', function () { globalThis.__cc++; });\
             dg.addEventListener('toggle', function (ev) {\
               globalThis.__tc++;\
               globalThis.__tst = ev.oldState + '->' + ev.newState; });\
             // (a) 模态开 → 移除 open 属性（分腿：showModal 的 toggle 先排空）。
             dg.showModal();\
             globalThis.__zw_fire();\
             globalThis.__t1 = globalThis.__tst; // closed->open
             dg.removeAttribute('open');\
             globalThis.__modImm = String(dg.matches(':modal'));\
             globalThis.__rcImm = globalThis.__rc; // close 异步——同步点未派",
        )
        .unwrap();
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    sandbox.execute("globalThis.__t2 = globalThis.__tst;").unwrap();
    // (b) IDL falsy 同面；(c) closed 再移除 wasOpen 门零事件。
    sandbox
        .execute(
            "var dg = document.getElementById('dg');\
             dg.showModal();\
             globalThis.__zw_fire();\
             dg.open = false;\
             globalThis.__zw_fire();\
             globalThis.__rcB = globalThis.__rc;\
             dg.removeAttribute('open');\
             globalThis.__zw_fire();\
             globalThis.__rcC = globalThis.__rc;\
             // (d) 同任务 showModal + 移除 → toggle 合并（R4200 去重：单发）。
             dg.showModal();\
             dg.removeAttribute('open');\
             globalThis.__tcD0 = globalThis.__tc;\
             globalThis.__zw_fire();\
             globalThis.__tcD1 = globalThis.__tc;\
             // (e) detached 文档 dialog 三方法。
             var fdoc = document.implementation.createHTMLDocument();\
             var fdg = fdoc.createElement('dialog');\
             var _sm = false;\
             try { fdg.showModal(); } catch (e) { _sm = (e.name === 'InvalidStateError'); }\
             globalThis.__rsm = String(_sm);\
             globalThis.__rsmatch = String(fdg.matches('[open]'));\
             var _sh = true; try { fdg.show(); } catch (e) { _sh = false; }\
             var _cl = true; try { fdg.close(); } catch (e) { _cl = false; }\
             globalThis.__rsc = String(_sh && _cl);\
             // (f) 属性直开（未经 show，无 close watcher）→ 移除仅隐藏零事件
             //（dialog-open「IDL false 不派 close」断言面）。
             var d3 = document.createElement('dialog');\
             document.body.appendChild(d3);\
             d3.addEventListener('close', function () { globalThis.__rcF++; });\
             globalThis.__rcF = 0;\
             d3.open = true;\
             d3.open = false;\
             globalThis.__zw_fire();\
             globalThis.__rcFd = String(globalThis.__rcF);",
        )
        .unwrap();

    // (a) 移除面：立即非模态、close 异步恰一次、cancel 零次、toggle 形态。
    assert_eq!(
        sandbox.execute("globalThis.__modImm").unwrap().value,
        "false",
        "(a) 移除后立即非 :modal"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rcImm)").unwrap().value,
        "0",
        "(a) close 异步排队——移除同步点未派"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cc)").unwrap().value,
        "0",
        "(a) cancel 零次"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__rc >= 1)").unwrap().value,
        "true",
        "(a) close 恰派发"
    );
    assert_eq!(sandbox.execute("globalThis.__t1").unwrap().value, "closed->open", "(a) 开腿 toggle 形态");
    assert_eq!(
        sandbox.execute("globalThis.__t2").unwrap().value,
        "open->closed",
        "(a) 移除腿 toggle open->closed"
    );
    // (b) IDL falsy 同面（close 计数累计）。
    assert_eq!(
        sandbox.execute("String(globalThis.__rcB >= 2)").unwrap().value,
        "true",
        "(b) IDL open=false 面 close 同样派发"
    );
    // (c) wasOpen 门：closed dialog 再移除零 close。
    assert_eq!(
        sandbox.execute("String(globalThis.__rcC)").unwrap().value,
        sandbox.execute("String(globalThis.__rcB)").unwrap().value,
        "(c) closed 再移除零事件"
    );
    // (d) 同任务合并：showModal+移除仅一次 toggle。
    assert_eq!(
        sandbox.execute("String(globalThis.__tcD1 - globalThis.__tcD0)").unwrap().value,
        "1",
        "(d) 同任务开+移 toggle 合并单发"
    );
    // (e) detached 文档面。
    assert_eq!(
        sandbox.execute("globalThis.__rsm").unwrap().value,
        "true",
        "(e) detached showModal 抛 InvalidStateError"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rsmatch").unwrap().value,
        "false",
        "(e) detached 产物 matches('[open]') false"
    );
    assert_eq!(
        sandbox.execute("globalThis.__rsc").unwrap().value,
        "true",
        "(e) detached show/close 不抛"
    );
    // (f) 属性直开移除面：零 close（watcher 门）。
    assert_eq!(
        sandbox.execute("globalThis.__rcFd").unwrap().value,
        "0",
        "(f) 属性直开（未 show）移除零 close"
    );
}
