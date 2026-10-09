// slice48（attr-handle history 回落实钉）：
// ①apply 跨界属性读存活——IDL boolean / 反射串两面（slice45 申报残余⑨：__zw_get_attr_handle
//   无 history/快照回落，attr 读链跨 apply 落空为系统性边界）。宿主 `__zw_get_attr_handle` /
//   `__zw_has_attr_handle` 补第 2 层 MUTATION_HISTORY 重放（与 text/tag 面 R100 同构）后：
//   createElement 产物 `el.loop = true` / `el.role = 'button'` 经 apply 批次边界（drain 队列 →
//   append_mutation_history → apply 落快照）后，getAttribute/hasAttribute 仍返现行值。
// ②控制臂：setAttribute 面（R122 实例层既有覆盖，防过度修复破坏实例互操作）+ absent/remove
//   面（latest-wins 序：历史层 Remove 必须压过更早 Set；未设属性不得被他 handle 命中）。
//   批次边界装配镜像 part45 钉①（drain → append_mutation_history → apply_dom_mutations_
//   with_persistent → 快照换代 → 代际 bump）——harness 装配镜像 R100 契约（drain→append），
//   生产接线未落地（webview/renderer apply 链均无 append_mutation_history 调用方，残余在册）。
//   https://dom.spec.whatwg.org/#dom-element-getattribute
//   https://dom.spec.whatwg.org/#dom-element-hasattribute

/// slice48 钉①(a)：IDL boolean 反射写（`el.loop = true`，part05.js:164 真写支路不经
/// R122 实例 upsert）跨 apply 后 hasAttribute/getAttribute 存活。spec：属性是元素状态，
/// 无 alive/快照概念（属性面不适用 #concept-node-list-alive）。
#[test]
fn handle_attr_idl_boolean_survives_apply_s48() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s48".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 批 N：createElement video → IDL loop=true（SetAttrOnHandle{loop,""}）→ appendChild。
    // 同批正向臂：当前批 latest-wins 即时可见（既有语义，apply 前不变）。
    sandbox
        .execute(
            "var v48 = document.createElement('video');\
             v48.loop = true;\
             document.body.appendChild(v48);\
             globalThis.__r_s48_pre_has = String(v48.hasAttribute('loop'));",
        )
        .unwrap();

    // 批次边界 apply #1（harness 装配镜像 R100 契约：drain→append；生产接线未落地，见头注）。
    let batch1: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(
        batch1.iter().any(|m| matches!(m, DomMutation::SetAttrOnHandle { name, .. } if name == "loop")),
        "批 N 必须产出 SetAttrOnHandle{{loop}} 记录（IDL boolean 反射真写支路）"
    );
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch1);
    let mut persistent: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch1,
            Some(&persistent),
        )
        .expect("批次边界 apply #1：handle-form 节点落 host 快照");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    // 批 N+1：apply 已落窗口内读回——历史层（第 2 层）供给现行值。
    sandbox
        .execute(
            "globalThis.__r_s48_post_has = String(v48.hasAttribute('loop'));\
             globalThis.__r_s48_post_get = String(v48.getAttribute('loop'));",
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__r_s48_pre_has").unwrap().value,
        "true",
        "同批正向臂：apply 前 current-batch 层即时可见（既有语义）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_s48_post_has").unwrap().value,
        "true",
        "apply 跨界后 hasAttribute('loop') 存活（落空态：宿主队列空 → 恒 false）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_s48_post_get").unwrap().value,
        "",
        "apply 跨界后 getAttribute('loop') 返 present-empty（落空态：返 null）"
    );
}

/// slice48 钉①(b)：反射串写（`el.role = 'button'`，part04.js:10279 经缓存 + host 直写、
/// 无实例 upsert）跨 apply 后 getAttribute 存活。role 的 IDL getter 走 _reflectedAttrs
/// 缓存即时面，本钉钉的是**属性读面**（getAttribute 不读缓存——spec 同一 attribute list）。
#[test]
fn handle_attr_reflected_getattr_survives_apply_s48() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s48".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var r48 = document.createElement('div');\
             r48.role = 'button';\
             document.body.appendChild(r48);",
        )
        .unwrap();

    let batch1: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(
        batch1.iter().any(|m| matches!(m, DomMutation::SetAttrOnHandle { name, .. } if name == "role")),
        "批 N 必须产出 SetAttrOnHandle{{role}} 记录"
    );
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch1);
    let mut persistent: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch1,
            Some(&persistent),
        )
        .expect("批次边界 apply #1");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    sandbox
        .execute("globalThis.__r_s48_role = String(r48.getAttribute('role'));")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_s48_role").unwrap().value,
        "button",
        "apply 跨界后 getAttribute('role') 存活（落空态：返 null）"
    );
}

/// slice48 控制臂(a)：setAttribute 面跨 apply 读存活——R122 实例层既有覆盖（setAttribute
/// 经 `_zwAttrInstUpsert` 同步），修复前即绿。防过度修复（历史层接入）破坏实例互操作。
#[test]
fn handle_attr_setattr_face_control_s48() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s48".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var c48 = document.createElement('div');\
             c48.setAttribute('data-x', 'v1');\
             document.body.appendChild(c48);",
        )
        .unwrap();

    let batch1: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch1);
    let mut persistent: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch1,
            Some(&persistent),
        )
        .expect("批次边界 apply #1");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    sandbox
        .execute(
            "globalThis.__r_s48_c_get = String(c48.getAttribute('data-x'));\
             globalThis.__r_s48_c_has = String(c48.hasAttribute('data-x'));",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_s48_c_get").unwrap().value,
        "v1",
        "控制臂：setAttribute 面跨 apply 存活（R122 实例层既有覆盖）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_s48_c_has").unwrap().value,
        "true",
        "控制臂：hasAttribute 存活"
    );
}

/// slice48 控制臂(b)：absent/remove 面——(i) 未设属性不被他 handle 历史误命中（handle
/// 键隔离）；(ii) 跨 apply 后 remove 闭合：remove 入当前批即时 absent，apply#2 后历史层
/// latest-wins（Remove 晚于 Set）保持 absent。
#[test]
fn handle_attr_absent_and_remove_faces_s48() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s48".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 批 N：设 data-y（供 remove 面）；同批另一 handle 设 other-attr（误命中哨兵）。
    sandbox
        .execute(
            "var d48 = document.createElement('div');\
             d48.setAttribute('data-y', 'v2');\
             document.body.appendChild(d48);\
             var o48 = document.createElement('span');\
             o48.setAttribute('other-attr', 'zz');\
             document.body.appendChild(o48);",
        )
        .unwrap();

    let batch1: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch1);
    let mut persistent: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch1,
            Some(&persistent),
        )
        .expect("批次边界 apply #1");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    // 批 N+1：未设属性 absent + remove data-y（入当前批）。
    sandbox
        .execute(
            "globalThis.__r_s48_nope_get = String(d48.getAttribute('nope'));\
             globalThis.__r_s48_nope_has = String(d48.hasAttribute('nope'));\
             d48.removeAttribute('data-y');\
             globalThis.__r_s48_rm_now_has = String(d48.hasAttribute('data-y'));",
        )
        .unwrap();

    // 批次边界 apply #2：Remove 落历史。
    let batch2: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(
        batch2.iter().any(|m| matches!(m, DomMutation::RemoveAttrOnHandle { name, .. } if name == "data-y")),
        "批 N+1 必须产出 RemoveAttrOnHandle{{data-y}} 记录"
    );
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch2);
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch2,
            Some(&persistent),
        )
        .expect("批次边界 apply #2");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    // 批 N+2：历史层 latest-wins——Remove（晚）压 Set（早）保持 absent。
    sandbox
        .execute(
            "globalThis.__r_s48_rm_post_has = String(d48.hasAttribute('data-y'));\
             globalThis.__r_s48_rm_post_get = String(d48.getAttribute('data-y'));",
        )
        .unwrap();

    assert_eq!(
        sandbox.execute("globalThis.__r_s48_nope_get").unwrap().value,
        "null",
        "未设属性 getAttribute 恒 null（历史层不得跨 handle 误命中）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_s48_nope_has").unwrap().value,
        "false",
        "未设属性 hasAttribute 恒 false"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_s48_rm_now_has").unwrap().value,
        "false",
        "remove 入当前批即时 absent（既有 current-batch 语义）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_s48_rm_post_has").unwrap().value,
        "false",
        "apply#2 后历史层 latest-wins：Remove（晚）压 Set（早）保持 absent"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_s48_rm_post_get").unwrap().value,
        "null",
        "apply#2 后 getAttribute(data-y) 保持 null"
    );
}

// slice48（t8m 页面加载生命周期：document.readyState 三态 + readystatechange 双过渡派发）：
// 红证据（bilibili 运行 evidence/t8m-lifecycle/，2026-10-09）：native getter 硬编码
// "complete" → 页面脚本执行期（React/Next hydration 调度门控读点）恒 complete +
// 全页零 readystatechange——事件序「先 complete 后 DCL」在真实浏览器不可能出现，
// 框架生命周期假设错乱（#425/#418 水合错误簇根因）。Chrome oracle：脚本期 loading →
// rs(interactive) → DCL → rs(complete) → load，各恰 1 次。
// 修复面（三件）：①native getter 读 shim 全局 `__zwReadyState`（未注入/非字符串/越界值
// 缺省 "complete"——WPT run_script 模型零回归）；②script_gen `script_set_ready_state`/
// `script_transition_ready_state` 过渡命令（赋值 + 派发原子单串）；③shim
// `__zw_dispatch_event` readystatechange 分支——plain Event 不冒泡不可取消，经
// `_dispatchWithBubble` targetSlot='doc'（R40 document target 语义：event.target =
// document，doc 槽位监听在 target 站触发，window 虚站不触发）。
// 宿主序镜像（renderer `page_scripts::finish_page_load` / tab `PageScriptRunner::finish`
// + 阶段起点注入）：set('loading') → [页面脚本] → transition('interactive') → DCL →
// transition('complete') → load。
// https://html.spec.whatwg.org/multipage/dom.html#dom-document-readystate
// https://html.spec.whatwg.org/multipage/syntax.html#the-end

/// t8m 钉①：readyState 缺省 "complete" + 状态宿越界值不外泄。
/// 断言翻转语义：getter 改回硬编码 "complete" → ①臂脚本期断言红；白名单撤除 →
/// bogus/非字符串臂红（非标准态外泄）。WPT run_script 模型（无宿主过渡）由
/// 「未注入缺省」臂钉住（既有断言零回归面）。
#[test]
fn ready_state_default_complete_and_host_whitelist_s48() {
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
        "<html><head></head><body><p>s48</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s48".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 未注入（WPT run_script 模型）：缺省 "complete"，状态宿自身 undefined。
    assert_eq!(
        sandbox.execute("String(document.readyState)").unwrap().value,
        "complete",
        "未注入状态宿时 readyState 缺省 complete（run_script 模型零回归面）"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__zwReadyState)").unwrap().value,
        "undefined",
        "状态宿未注入时全局不存在"
    );

    // 宿主阶段起点：set('loading') → getter 读状态宿。
    sandbox.execute(&script_set_ready_state("loading")).unwrap();
    assert_eq!(
        sandbox.execute("String(document.readyState)").unwrap().value,
        "loading",
        "阶段起点后脚本期 readyState = loading（Chrome oracle）"
    );

    // 页面脚本篡改越界值 / 非字符串值 → 缺省 complete（不外泄非标准态）。
    sandbox.execute("globalThis.__zwReadyState = 'bogus'").unwrap();
    assert_eq!(
        sandbox.execute("String(document.readyState)").unwrap().value,
        "complete",
        "越界值按缺省处理（readystate 枚举仅三态）"
    );
    sandbox.execute("globalThis.__zwReadyState = 42").unwrap();
    assert_eq!(
        sandbox.execute("String(document.readyState)").unwrap().value,
        "complete",
        "非字符串值按缺省处理"
    );
}

/// t8m 钉②：三态过渡序 + readystatechange 派发契约（spec HTML §the end）。
/// 断言翻转语义：①getter 回硬编码 → 脚本期 loading / handler 内 rs 值断言红；
/// ②过渡命令撤除或派发与赋值分裂（分两次提交）→ 序列 join / rs 恰 2 次断言红；
/// ③shim 分支落泛型（bubbles:true）→ bubbles/cancelable/window-rs 监听断言红；
/// ④targetSlot='doc' 撤（落 html 元素 target）→ `e.target === document` 断言红。
#[test]
fn ready_state_lifecycle_transitions_and_rs_dispatch_s48() {
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
        "<html><head></head><body><p>s48</p></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s48".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 宿主阶段起点（先于页面脚本）。
    sandbox.execute(&script_set_ready_state("loading")).unwrap();

    // 页面脚本：注册 document/window 监听 + 记录脚本期 readyState。
    sandbox
        .execute(
            "window.__evts = [];\
             window.__rsScript = document.readyState;\
             document.addEventListener('readystatechange', function () { window.__evts.push('rs:' + document.readyState); });\
             window.addEventListener('readystatechange', function () { window.__evts.push('rs-win'); });\
             document.addEventListener('DOMContentLoaded', function () { window.__evts.push('DCL:' + document.readyState); });\
             window.addEventListener('load', function () { window.__evts.push('load:' + document.readyState); });\
             globalThis.__rsFlags = {};\
             document.addEventListener('readystatechange', function (e) { globalThis.__rsFlags.bubbles = e.bubbles; globalThis.__rsFlags.cancelable = e.cancelable; globalThis.__rsFlags.targetIsDoc = (e.target === document); globalThis.__rsFlags.trusted = e.isTrusted; });",
        )
        .unwrap();

    // 宿主序：DCL 前过渡 interactive → DCL；load 前过渡 complete → load。
    // 过渡命令 = 赋值 + readystatechange 派发原子单串（script_transition_ready_state）。
    sandbox.execute(&script_transition_ready_state("interactive")).unwrap();
    sandbox
        .execute(&script_dispatch_dom_event("html", "DOMContentLoaded", None))
        .unwrap();
    sandbox.execute(&script_transition_ready_state("complete")).unwrap();
    sandbox.execute(&script_dispatch_dom_event("html", "load", None)).unwrap();

    // 脚本期 readyState = loading（红证据反转面：修前恒 complete）。
    assert_eq!(
        sandbox.execute("window.__rsScript").unwrap().value,
        "loading",
        "页面脚本执行期 readyState = loading"
    );
    // 事件序与次数：rs(interactive) → DCL → rs(complete) → load，各恰 1 次；
    // handler 内读 readyState 与过渡值一致（原子单串面）。
    assert_eq!(
        sandbox.execute("window.__evts.join('|')").unwrap().value,
        "rs:interactive|DCL:interactive|rs:complete|load:complete",
        "事件序须为 rs(interactive) → DCL → rs(complete) → load，各恰 1 次"
    );
    // 终态 complete。
    assert_eq!(
        sandbox.execute("String(document.readyState)").unwrap().value,
        "complete",
        "load 前过渡后终态 complete"
    );
    // readystatechange 事件契约：不冒泡不可取消、target = document、UA 印章 isTrusted。
    assert_eq!(
        sandbox.execute("[String(globalThis.__rsFlags.bubbles), String(globalThis.__rsFlags.cancelable), String(globalThis.__rsFlags.targetIsDoc), String(globalThis.__rsFlags.trusted)].join(',')").unwrap().value,
        "false,false,true,true",
        "rs 事件 bubbles=false cancelable=false target=document isTrusted=true（HTML §the end + R312）"
    );
    // window 侧 rs 监听 0 次（fires at the Document——不冒泡到 window）。
    assert_eq!(
        sandbox.execute("window.__evts.indexOf('rs-win')").unwrap().value,
        "-1",
        "window 侧 readystatechange 监听不触发"
    );
}
