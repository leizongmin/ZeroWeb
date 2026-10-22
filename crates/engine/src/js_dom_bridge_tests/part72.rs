// part72 — Page Visibility 宿主驱动面（goal background-resource-lifecycle M3，2026-10-23）：
// shim 可见性 getter 读宿主全局 `__ZW_HIDDEN`（原硬编码 visible）+ visibilitychange
// 派发钩子 `__zw_dispatch_visibilitychange`（照 _firePointerLockEvent document 级形态，
// HTML Page Visibility §visibilitychange：fire at the Document、不冒泡不可取消）+
// script_gen 两命令（裸 state 跨导航重注入 / 赋值+派发原子单串）+ onvisibilitychange
// IDL handler（_defineDocOnHandler 列表）。
// https://html.spec.whatwg.org/multipage/interaction.html#visibility-statens

/// 钉①初态 + ②过渡派发 + ③跨导航裸 state 无事件。断言反转面：getter 回硬编码 →
/// 过渡后 hidden/visibilityState/webkit* 断言红；派发钩子缺 → 事件计数 0 断言红；
/// 裸 state 误带派发 → 跨导航重注入后计数多 1 断言红。
#[test]
fn page_visibility_state_and_visibilitychange_dispatch_s72() {
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
        "<html><head></head><body><p>s72</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s72".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 初态（未注入宿主全局）：可见——与旧硬编码行为一致（headless 单 tab 零回归面）。
    assert_eq!(
        sandbox
            .execute("[String(document.hidden), document.visibilityState, String(document.webkitHidden), document.webkitVisibilityState].join(',')")
            .unwrap()
            .value,
        "false,visible,false,visible",
        "初态未注入 __ZW_HIDDEN 恒可见（缺省语义同旧硬编码）"
    );

    // 页面脚本：doc/window 监听 + 事件契约 + onvisibilitychange IDL。
    sandbox
        .execute(
            "window.__evts = [];\
             document.addEventListener('visibilitychange', function () { window.__evts.push('vis:' + document.visibilityState); });\
             window.addEventListener('visibilitychange', function () { window.__evts.push('vis-win'); });\
             globalThis.__visFlags = {};\
             document.addEventListener('visibilitychange', function (e) { globalThis.__visFlags.bubbles = e.bubbles; globalThis.__visFlags.cancelable = e.cancelable; globalThis.__visFlags.targetIsDoc = (e.target === document); globalThis.__visFlags.trusted = e.isTrusted; });\
             globalThis.__visIdl = 0;\
             document.onvisibilitychange = function () { globalThis.__visIdl++; };",
        )
        .unwrap();

    // 宿主过渡：赋值 + 派发原子单串（script_set_visibility）。handler 内读新值。
    sandbox.execute(&script_set_visibility(true)).unwrap();
    assert_eq!(
        sandbox
            .execute("[String(document.hidden), document.visibilityState, String(document.webkitHidden), document.webkitVisibilityState].join(',')")
            .unwrap()
            .value,
        "true,hidden,true,hidden",
        "宿主隐藏过渡后四 getter 全 hidden"
    );
    assert_eq!(
        sandbox.execute("window.__evts.join('|')").unwrap().value,
        "vis:hidden|vis-win",
        "visibilitychange 恰 1 次达 doc+window，handler 内读新值（原子单串面）"
    );
    assert_eq!(
        sandbox
            .execute("[String(globalThis.__visFlags.bubbles), String(globalThis.__visFlags.cancelable), String(globalThis.__visFlags.targetIsDoc), String(globalThis.__visFlags.trusted)].join(',')")
            .unwrap()
            .value,
        "false,false,true,true",
        "visibilitychange 不冒泡不可取消 target=document isTrusted=true（HTML Page Visibility）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__visIdl)").unwrap().value,
        "1",
        "onvisibilitychange IDL handler 随过渡触发 1 次"
    );

    // 恢复可见：计数 2、状态回 visible。
    sandbox.execute(&script_set_visibility(false)).unwrap();
    assert_eq!(
        sandbox.execute("window.__evts.join('|')").unwrap().value,
        "vis:hidden|vis-win|vis:visible|vis-win",
        "恢复可见再派 1 次，序为 hidden → visible"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__visIdl)").unwrap().value,
        "2",
        "IDL handler 计数 2（两次过渡各 1 次）"
    );

    // 跨导航重注入面：裸 state 脚本只改态不派事件（新 Document 无过渡语义）。
    sandbox.execute(&script_set_visibility_state(true)).unwrap();
    assert_eq!(
        sandbox.execute("[String(document.hidden), document.visibilityState].join(',')").unwrap().value,
        "true,hidden",
        "裸 state 脚本置 hidden"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__visIdl)").unwrap().value,
        "2",
        "裸 state 无事件（重注入初态不派 visibilitychange）"
    );
}
