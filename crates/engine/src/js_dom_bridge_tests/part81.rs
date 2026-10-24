// part81 — html-semantics M2 切片 50（R5254，2026-10-24）：parser 初始 open 的
// details toggle 补偿。
//
// spec the-details-element：attribute change steps 随 parser 逐元素运行——解析器添加
// `open` 也 queue details toggle event task。主文档初始 HTML（host 解析直建，不经 JS
// 写路径）由注册完成点扫描补偿（callbacks 注册尾调 `__zwDetailsScan4100([document],
// true)`，与 innerHTML/createElement-HTML/包装世界解析面同链）；DOMParser 产物在
// parseFromString 返回前扫描快照（part02——快照架构无宿主派发链，task 直调 `ontoggle`
// 属性 + `open` IDL getter）。toggleEvent.html t9（inline ontoggle promise）与末案
//（parseFromString 产物 ontoggle）依赖此面；缺补偿时两案永挂 → 文件级 Timeout。
// https://html.spec.whatwg.org/multipage/interactive-elements.html#the-details-element

/// 钉：parser 初始 open 的 toggle 补偿——(a) 主文档初始 `<details open>` 恰好派一次
/// toggle（oldState closed/newState open，target open 真值），同态写 `open = true`
/// 不再派；(b) 初始 closed details 不派（翻转：扫描被撤 → (a) 计数 0）；(c)
/// DOMParser 产物 `<details open>` 的 ontoggle 收到事件且 e.target.open 为 true。
/// 时序模拟：契约无宿主 timer——赋 `__zw_setTimeout` 记录式 stub，handler 就位后
/// `__zw_fire` 手动驱动（runner `__zw_test_setTimeout` / browser TimerBridge 为真
/// timer，task 恒后于页面脚本）。
#[test]
fn parser_open_details_toggle_r5254() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // 记录式 host timer stub（part01 契约同款）——task 登记、手动驱动。
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
        "<html><body><details id=\"r254open\" open></details>\
         <details id=\"r254closed\"></details></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s50".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    // register 尾部跑主文档初始扫描（刀一）——task 被 stub 登记、暂不派发。
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__r254a = 0;\
             globalThis.__r254b = 0;\
             globalThis.__r254st = '';\
             var dOpen = document.getElementById('r254open');\
             var dClosed = document.getElementById('r254closed');\
             dOpen.ontoggle = function (ev) {\
               globalThis.__r254a++;\
               globalThis.__r254st = ev.oldState + '->' + ev.newState + ':' + String(dOpen.open);\
             };\
             dClosed.ontoggle = function () { globalThis.__r254b++; };\
             // DOMParser 产物面（刀二）：parse → handler → 等 task。
             var pdoc = new DOMParser().parseFromString('<details open></details>', 'text/html');\
             var pel = pdoc.querySelector('details');\
             globalThis.__r254c = 0;\
             globalThis.__r254ct = '';\
             pel.ontoggle = function (ev) {\
               globalThis.__r254c++;\
               globalThis.__r254ct = String(ev.target && ev.target.open);\
             };",
        )
        .unwrap();
    // 宿主生命周期模拟：页面脚本跑完后派 DCL（扫描监听在此触发——解析完成点，
    // handler/resolver 已就位），toggle task 被 stub 登记后手动驱动。
    sandbox
        .execute(
            "document.dispatchEvent(new Event('DOMContentLoaded', { bubbles: true }));",
        )
        .unwrap();
    // 驱动 task（runner/browser 宿主为真 timer 的时序等价）。
    sandbox.execute("globalThis.__zw_fire();").unwrap();
    // 同态写不重派（t9 断言面）。
    sandbox
        .execute(
            "document.getElementById('r254open').open = true;\
             var d2 = new DOMParser().parseFromString('<div></div>', 'text/html');\
             globalThis.__r254d = d2.querySelectorAll('details[open]').length;",
        )
        .unwrap();
    sandbox.execute("globalThis.__zw_fire();").unwrap();

    assert_eq!(
        sandbox.execute("String(globalThis.__r254a)").unwrap().value,
        "1",
        "(a) 初始 open details 恰好一次 toggle"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r254st").unwrap().value,
        "closed->open:true",
        "(a) toggle 形态 oldState/newState/target.open"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r254b)").unwrap().value,
        "0",
        "(b) 初始 closed details 不派 toggle"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r254c)").unwrap().value,
        "1",
        "(c) DOMParser 产物 open details 收到 toggle"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r254ct").unwrap().value,
        "true",
        "(c) 事件 target.open 反射真值"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__r254d)").unwrap().value,
        "0",
        "(c2) 无 open details 的解析产物零 task（扫描窄域）"
    );
}
