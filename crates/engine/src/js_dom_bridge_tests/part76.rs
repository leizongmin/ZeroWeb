// part76 — html-semantics M2 切片 44（R5247，2026-10-23）：跨代 handle proxy 孤儿
// isConnected 假阴性修正 + activeElement 焦点链 retarget light 锚误穿收口。
//
// 背景（focus-after-close Complex 案，testdriver 多命令 execute_script 间宿主 apply
// 换代）：handle 形 proxy 的 `_zwNodeParent` 反链缺失（静态树节点本无 R91 记账，
// 跨代后 proxy 亦不补记）→ isConnected 走反链 + rect 探测兜底双假阴性 → R4600
// detached 臂误入；同一 stale 面（`_shadowHandles` 跨执行残留注册——runner 投影
// 容器以 hostSel:null 形态入表）使 activeElement getter 的 focus-chain retarget 把
// light 锚误穿投影边界回吐 host。
//
// 修复双刀（shim part01/part04/part06）：①handle 形 isConnected 前置复核——宿主
// 持久反查 `__zw_selector_for_handle` + querySelector 同一性核验（命中同 handle 且
// `__zw_contains` 在档才短路 true；`_zwRemovedHandles` 守门保持 remove 语义；shadow
// 树内锚宿主扁平 selector 命中别元素 → 不短路，R91 反链权威判定保持）；②activeElement
// retarget 语义前提门——锚经同一性核验确认身处 light 树则直返锚（spec 焦点链
// retargeting 仅当焦点元素在 shadow 树内才穿边界；light 锚无边界可穿）。
//
// 契约构造说明：identity 桥为纯映射回调（`__zw_handle_for_selector` /
// `__zw_selector_for_handle`），键值由本测单方面定义（"k47*" 任意稳定键即可，R331
// QS 升格以桥值为 handle 物化）——静态树升格 proxy 无反链记账，恰为生产跨代孤儿
// 的公开面同构。stale `_shadowHandles` 投影注册（ Complex 案 light 误穿的完整形态）
// 唯一写点 attachShadow 注册的是新建 fragment handle，公开面不可指向 light 锚父——
// 该面自然红钉由 corpus focus-after-close 全域对账钉住（s44 evidence GAIN=1 逐位），
// 本文件 shadow 臂钉 retarget 既有面不回归。
// https://dom.spec.whatwg.org/#dom-node-isconnected
// https://html.spec.whatwg.org/multipage/interaction.html#focus-chain-retargeting
// https://html.spec.whatwg.org/multipage/interaction.html#focusing-steps

/// 钉①：跨代孤儿 handle 形 proxy（无反链记账）isConnected 不得假阴性——前置复核
/// 经宿主反查 + querySelector 同一性核验短路 true；remove 后 isConnected false 语义
/// 保持（快照未换代、`__zw_contains` 仍在档，唯一拦截是 `_zwRemovedHandles` 守门）。
/// 断言翻转：(a) 臂翻转 = 前置复核被撤（孤儿面 rect 探测假阴性复活——focus-after-
/// close Complex 案 R4600 detached 臂误入回归）；(b) 臂翻转 = removed 守门被撤
/// （remove 后仍经复核短路 true——「Focus should be moved to the body if the
/// previously focused element is removed」案回归）。
#[test]
fn is_connected_handle_orphan_identity_recheck_r5247() {
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
        "<html><body><input id='i47a'><input id='i47b'></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s47a".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 双向 identity 桥（键值契约自定义——R331 QS 升格以桥值为 handle 物化）+ 代际 bump。
    sandbox.register_callback(
        "__zw_handle_for_selector",
        Box::new(|args: &[String]| -> String {
            match args.first().map(String::as_str).unwrap_or("") {
                "#i47a" => "k47a".to_string(),
                _ => String::new(),
            }
        }),
    );
    sandbox.register_callback(
        "__zw_selector_for_handle",
        Box::new(|args: &[String]| -> String {
            match args.first().map(String::as_str).unwrap_or("") {
                "k47a" => "#i47a".to_string(),
                _ => String::new(),
            }
        }),
    );
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    // 批 N+1：QS 升格 handle 形（R331 反查以桥值物化）→ 孤儿 isConnected 复核。
    sandbox
        .execute(
            "var ip = document.querySelector('#i47a');\
             globalThis.__r5247f = String(!!ip && ip.__zwHandle === 'k47a' && ip.__zwSelector == null);\
             globalThis.__r5247a = String(!!ip && ip.isConnected === true);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5247f").unwrap().value,
        "true",
        "前提：identity 桥在位时 QS 升格为 handle 形 proxy（R331，handle 形无 __zwSelector）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5247a").unwrap().value,
        "true",
        "(a) 无反链记账的跨代 handle proxy isConnected 须经宿主反查同一性核验短路 true（孤儿假阴性收口）"
    );

    // remove 后：快照未换代（__zw_contains 仍命中 #i47a），removed 守门是唯一拦截。
    sandbox
        .execute("ip.remove(); globalThis.__r5247b = String(ip.isConnected === false);")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5247b").unwrap().value,
        "true",
        "(b) remove 后 isConnected false 语义保持（_zwRemovedHandles 守门不被前置复核短路）"
    );
}

/// 钉②：activeElement 焦点链 retarget 面——真 attachShadow 注册在位时 light 锚
/// 聚焦直返锚、shadow 锚聚焦 retarget host（R4600 focus-chain retargeting 既有面）、
/// 双向切换稳定。light 锚门的完整误穿形态（stale `_shadowHandles` 投影注册）公开面
/// 不可构造（见文件头），自然红钉由 corpus focus-after-close 全域对账钉住；本钉
/// 断言翻转 = (b) 臂 light 门误杀 shadow 锚（retarget 面回归，R4600 双层嵌套
/// dialog-focus-shadow 断言面同型）。
#[test]
fn active_element_light_anchor_no_retarget_r5247() {
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
        "<html><body><input id='i47f'></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s47b".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox.register_callback(
        "__zw_handle_for_selector",
        Box::new(|args: &[String]| -> String {
            match args.first().map(String::as_str).unwrap_or("") {
                "#i47f" => "k47f".to_string(),
                _ => String::new(),
            }
        }),
    );
    sandbox.register_callback(
        "__zw_selector_for_handle",
        Box::new(|args: &[String]| -> String {
            match args.first().map(String::as_str).unwrap_or("") {
                "k47f" => "#i47f".to_string(),
                _ => String::new(),
            }
        }),
    );
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    // (a) light 锚聚焦直返（identity 桥在位、复核链可全通——门判定与焦点序解耦基线）；
    // (b) 真 attachShadow 后 shadow 锚聚焦 retarget host；(c) 焦点切回 light 锚再度直返。
    sandbox
        .execute(
            "var ip = document.querySelector('#i47f');\
             ip.focus();\
             globalThis.__r5247c = String(document.activeElement === ip);\
             var host2 = document.createElement('div'); host2.id = 'h47s'; document.body.appendChild(host2);\
             var sh = host2.attachShadow({ mode: 'open' });\
             var inp2 = document.createElement('input'); sh.appendChild(inp2);\
             inp2.focus();\
             globalThis.__r5247d = String(document.activeElement && document.activeElement.id === 'h47s');\
             ip.focus();\
             globalThis.__r5247e = String(document.activeElement === ip);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5247c").unwrap().value,
        "true",
        "(a) light 锚聚焦后 activeElement 直返锚（复核链全通面，门判定基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5247d").unwrap().value,
        "true",
        "(b) shadow 树内锚聚焦后 activeElement retarget host（R4600 focus-chain retargeting 面保持）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5247e").unwrap().value,
        "true",
        "(c) 焦点切回 light 锚后 activeElement 再度直返锚（双向稳定）"
    );
}
