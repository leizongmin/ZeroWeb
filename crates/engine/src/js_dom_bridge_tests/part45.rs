// slice45（handle-form 查询 stale 收口三件）：
// ①document 面查询对 handle-form 移除条目 stale（slice44 verdict 残余⑤，缺陷轮 I-1
//   读码新证→本轮自然红实证）。实测机理**两层**（立项实验证伪「仅漏斗缺口」前提）：
//   层1 记账——apply 代际 bump（R379/pa2b）有意保留 pendingAdded 的 handle 条目
//   （re-append 移动语义），apply 后 remove() 走 R51c 消零对冲，移除**不入**
//   _zwPendingRemoved——identity 消费面（gEBI `_zwPRSet().has` / R125 祖先行走 /
//   s43 fresh tag 面）全部失去剔除依据（slice44 verdict「ID 面两态均覆盖无此缺口」
//   的对照前提只对 apply 前窗口与 sel-form 成立）；层2 漏斗——即使入表，
//   `_zwPendingRemovedSels()` 只收含 `__zwSelector` 的 sel-form 条目（createElement
//   产物 `_wrapHandle` → `_makeProxy(null, handle)` 的 `__zwSelector` trap 恒 null），
//   document QS/QSA（slice44 sel 串过滤）对 handle-form stale 命中不剔除。
//   批次边界模拟镜像生产 apply 路径：drain 队列 → `append_mutation_history`（webview
//   apply 前同款，供 handle attr 读回走 latest-wins 重放）→ 持久 handle→selector 表 +
//   `apply_dom_mutations_with_persistent` 落快照（webview render_with_dom_mutations_
//   persistent 同源）→ `__zw_handle_for_selector` / `__zw_selector_for_handle` 双向
//   identity 桥（webview register_identity_bridge_callback / register_forward_identity_
//   bridge_callback 镜像）→ `__zw_apply_generation_bump`。r5010（part25）先例扩展。
//   https://dom.spec.whatwg.org/#concept-node-list-alive
//   https://dom.spec.whatwg.org/#dom-element-remove
//   https://dom.spec.whatwg.org/#dom-parentnode-queryselector

/// slice45 钉①：handle-form（createElement 产物）节点 apply 后同批 remove，
/// document 面 QS/QSA 不得再返回该节点（spec 查询面限当下 document tree）；
/// ID 面作对照臂（identity 消费面与 document 面同窗闭合）；换代后换收。
#[test]
fn handle_form_remove_document_query_stale_s45() {
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
        Arc::new(Mutex::new("<html><body><img id='keep45'></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s45".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 批 N：createElement（handle-form）→ 赋 id → appendChild（mutation 入队，host 快照未落）。
    sandbox
        .execute(
            "var d = document.createElement('img');\
             d.id = 'dyn45';\
             document.body.appendChild(d);\
             globalThis.__r_s45_pre = document.querySelectorAll('img').length;",
        )
        .unwrap();

    // 批次边界 apply #1：mutation 历史 + 持久表 + 快照落定 + 双向 identity 桥 + 代际 bump。
    let batch1: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch1.is_empty(), "批 N 必须产出 mutation 记录（createElement/setAttr/append）");
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
    // identity 桥双向 stub（生产 webview/renderer 注册的镜像）：selector→handle（R100
    // 反查）与 handle→selector（R145 正置——层1 修复的快照真含判定依赖此向）。
    let sel_to_handle: std::collections::HashMap<String, String> = persistent
        .iter()
        .map(|(h, s)| (s.clone(), h.clone()))
        .collect();
    sandbox.register_callback(
        "__zw_handle_for_selector",
        Box::new(move |args: &[String]| -> String {
            sel_to_handle
                .get(args.first().map(String::as_str).unwrap_or(""))
                .cloned()
                .unwrap_or_default()
        }),
    );
    let fwd_map = persistent.clone();
    sandbox.register_callback(
        "__zw_selector_for_handle",
        Box::new(move |args: &[String]| -> String {
            fwd_map
                .get(args.first().map(String::as_str).unwrap_or(""))
                .cloned()
                .unwrap_or_default()
        }),
    );
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    // 批 N+1：apply 已落窗口内 remove → document 面观测（QS / QSA / gEBI 对照臂）。
    sandbox
        .execute(
            "globalThis.__r_s45_applied_raw = String(__zw_query_all('img'));\
             globalThis.__r_s45_applied_id = String(document.getElementById('dyn45') === d);\
             d.remove();\
             globalThis.__r_s45_after_qs = String(document.querySelector('#dyn45') === null);\
             globalThis.__r_s45_after_qsa = document.querySelectorAll('img').length;\
             globalThis.__r_s45_after_id = String(document.getElementById('dyn45') === null);",
        )
        .unwrap();

    // 批次边界 apply #2（换代）：Remove 落快照——stale 窗口闭合（换代即愈）。
    let batch2: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch2.is_empty(), "remove 必须产出 Remove mutation 记录");
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch2);
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch2,
            Some(&persistent),
        )
        .expect("批次边界 apply #2：Remove 落 host 快照");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();
    sandbox
        .execute(
            "globalThis.__r_s45_gen2_qs = String(document.querySelector('#dyn45') === null);\
             globalThis.__r_s45_gen2_qsa = document.querySelectorAll('img').length;",
        )
        .unwrap();

    // 单点聚合断言：红 log 一次带出全部观测值（slice44 打红名单不可追溯教训）。
    // applied_raw 断言 host 快照真值（QSA 聚合计数受既有 `_elKeyOf` 未定义缺陷污染——
    // R161 pending-tag 回退 dedup 恒死（ReferenceError 被吞），apply 后 tag 形查询重复
    // 计入 pending 条目，属相邻既有缺陷，本钉不覆盖不申报为修复面）。
    let mut read = |name: &str| -> String {
        sandbox
            .execute(&format!("String(globalThis.{name})"))
            .unwrap()
            .value
    };
    let mut fails: Vec<String> = Vec::new();
    let mut expect = |fails: &mut Vec<String>, name: &str, want: &str| {
        let got = read(name);
        if got != want {
            fails.push(format!("{name}: want {want:?}, got {got:?}"));
        }
    };
    expect(&mut fails, "__r_s45_pre", "2");
    expect(&mut fails, "__r_s45_applied_raw", "#keep45|#dyn45");
    expect(&mut fails, "__r_s45_applied_id", "true");
    expect(&mut fails, "__r_s45_after_qs", "true");
    expect(&mut fails, "__r_s45_after_qsa", "1");
    expect(&mut fails, "__r_s45_after_id", "true");
    expect(&mut fails, "__r_s45_gen2_qs", "true");
    expect(&mut fails, "__r_s45_gen2_qsa", "1");
    assert!(
        fails.is_empty(),
        "handle-form remove 后 document 面查询 stale 观测（诊断：after_id=false ⇒ 层1 消零对冲\
         未入 removed 表；after_id=true 且 QS/QSA stale ⇒ 层2 漏斗不收 handle-form）：\n{}",
        fails.join("\n")
    );
}

/// slice45 钉③：morph 跟随面 >1 命中升格（slice42 残余申报收口）。spec 取值算法
/// 每读按当下 named objects 求值：多命中返 HTMLCollection、单命中返元素。2→1 时
/// 集合 morph 成元素全局（既有基线，slice32/42），1→2 重入时 morph 产物跟随臂
/// `>1 命中不升格`（slice42 口径）使全局停留元素——spec 偏差（Chrome 同 spec：
/// window.x = HTMLCollection）。修后跟随臂镜像恢复臂（g === undefined 臂 ≥2 恢复
/// 集合）升格回 installed 集合。
/// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
#[test]
fn named_access_morph_follow_multi_hit_upgrade_s45() {
    let mut sandbox = s42_sandbox!(
        "<html><body><img name='uu' id='u1'><img name='uu' id='u2'></body></html>"
    );
    sandbox
        .execute(
            "var col0 = window.uu;
             globalThis.__r_len0 = col0.length;
             document.getElementById('u1').setAttribute('name', 'off');
             globalThis.__r_morph = window.uu === document.getElementById('u2');
             document.getElementById('u1').setAttribute('name', 'uu');
             globalThis.__r_is_col = window.uu === col0;
             globalThis.__r_len = window.uu.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "双 name=uu 安装集合（slice30 基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_morph").unwrap().value,
        "true",
        "跌破单命中 morph 元素（spec 取值算法，slice32 基线不回退）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_is_col").unwrap().value,
        "true",
        "1→2 重入升格回集合（spec：多命中 named objects 返 HTMLCollection；修前\
         morph 产物停留元素全局）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len").unwrap().value,
        "2",
        "升格后全局反映双命中（live 维护自然延续）"
    );
}
