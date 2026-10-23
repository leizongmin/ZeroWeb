// part79 — html-semantics M2 切片 47（R5250，2026-10-23）：showModal fully-active 门。
//
// spec「show a modal dialog」步 2/3：subject 未连接抛 InvalidStateError（既有门）、
// node document 非 fully active 抛（本切片新增）。dialog-showModal「When the document
// is not attached to any pages」第二断言面根因：detached dialog 经
// `doc.body.appendChild(dg)` 移入 implementation.createHTMLDocument 文档后，
// M3 扩批 XLIII 的 `_zwUnmarkRemoved`（新父即脱离 removed 态）+ host 快照滞后
// `__zw_contains` 恒 1，连接门双信号均判连接 → showModal 误开。修复：detached/
// iframe doc body appendChild 通用路径（part03 `_r191adoptBody`）落 sel 键控收养章
// `__zwAdoptDocBySel`；showModal 读章（handle 形态走既有 `__zwAdoptDocByHandle`）
// owner doc ≠ active document → 抛；主文档 append 族（part04 appendChild/
// insertAdjacent/insertBefore）对称删章（回插恢复 fully active）。
// https://html.spec.whatwg.org/multipage/interactive-elements.html#dom-dialog-showmodal
// https://html.spec.whatwg.org/multipage/document-lifecycle.html#fully-active

/// 钉：showModal fully-active 门——(a) detached dialog showModal 抛 InvalidStateError
///（既有连接门防回归）；(b) 收养进 createHTMLDocument 文档后 showModal 仍抛
///（fully-active 新门——连接门已失效的形态）；(c) 回插主文档后 showModal 成功开
///（删章恢复 fully active，不误伤正常路径）；(d) 二次 close 后正常关（状态机无残留）。
/// 断言翻转：(b) = fully-active 门被撤（误开复活，open 属性出现）；(c) = 删章面被撤
///（回插后误抛——门过宽回归）。
#[test]
fn show_modal_fully_active_gate_r5250() {
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
        "<html><body><dialog id='dg'></dialog></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s47".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var dg = document.getElementById('dg');\
             dg.parentNode.removeChild(dg);\
             var _a = false;\
             try { dg.showModal(); } catch (e) { _a = (e.name === 'InvalidStateError'); }\
             globalThis.__r5250a = String(_a);\
             var fdoc = document.implementation.createHTMLDocument();\
             fdoc.body.appendChild(dg);\
             var _b = false;\
             try { dg.showModal(); } catch (e) { _b = (e.name === 'InvalidStateError'); }\
             globalThis.__r5250b = String(_b);\
             globalThis.__r5250b2 = String(dg.hasAttribute('open'));\
             document.body.appendChild(dg);\
             var _c = false;\
             try { dg.showModal(); _c = dg.hasAttribute('open'); } catch (e) { _c = false; }\
             globalThis.__r5250c = String(_c);\
             dg.close();\
             globalThis.__r5250d = String(!dg.hasAttribute('open'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5250a").unwrap().value,
        "true",
        "(a) detached showModal 抛 InvalidStateError（既有连接门防回归）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5250b").unwrap().value,
        "true",
        "(b) inert 文档收养后 showModal 抛 InvalidStateError（fully-active 门）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5250b2").unwrap().value,
        "false",
        "(b2) 抛后未写 open 属性（门先于打开步骤）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5250c").unwrap().value,
        "true",
        "(c) 回插主文档 showModal 成功开（删章恢复 fully active，不误伤）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5250d").unwrap().value,
        "true",
        "(d) close 正常关（open 属性移除，状态机无残留）"
    );
}
