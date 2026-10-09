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
