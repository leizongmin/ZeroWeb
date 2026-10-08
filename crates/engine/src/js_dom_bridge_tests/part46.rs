// slice46（sel-form 消零门同族收口 + `_elKeyOf` 死引用修复）：
// ①sel-form（快照 sel 节点）R334 移挂 → apply 后 PA 保留（R379 代际 bump 全量保留
//   带 `__zwSelector` 条目——K3 清除仅针对无 handle 无 sel 的 parse 补偿节点）→
//   R125 remove 时 R51c 消零门（part05.js `_zwHCLiveInvalidate` 消零位点）第 3 合
//   取元 `_rv45.__zwHandle` 对 sel-form 恒假 → 消零保持 → 不入 `_zwPendingRemoved`
//   → 至下次 apply 前 document QS/QSA 对该 sel stale 命中（slice45 缺陷轮 xI-1 补
//   申报的读码推演，本轮常驻钉实证自然红后按三态处置）。
//   https://dom.spec.whatwg.org/#concept-node-list-alive
//   https://dom.spec.whatwg.org/#concept-node-remove
//   https://dom.spec.whatwg.org/#dom-parentnode-queryselectorall
// ②part06.js R161 QSA pending-tag 回退 dedup 引用 `_elKeyOf` 全 shim 无定义 →
//   ReferenceError 被 catch 吞 → seen161 恒空 → dedup 恒死 → apply 后 tag 形查询
//   重复计入 pending 条目（R331 identity 反查把快照命中升格为原 handle proxy +
//   R379 PA 保留同一节点 → 双计）。修复 = 按调用意图定义 `_elKeyOf`（与 pending
//   键 k161 同构：'@'+handle / 'id:'+id）。（slice45 交付卡遗留申报收口）
// ③slice27 静态单命中面 live 评估为评估件：判定=边界保持（行为钉固化现状 + 申报
//   收窄，不强修）——钉 `static_single_hit_named_access_stale_boundary_s46` 在本文件。
//
// 批次边界 apply 镜像生产链（part45 钉①同款，r5010/part25 先例扩展）：drain 队列
// → `append_mutation_history`（webview apply 前同款）→ 持久 handle→selector 表 +
// `apply_dom_mutations_with_persistent` 落快照 → `__zw_handle_for_selector` /
// `__zw_selector_for_handle` 双向 identity 桥（webview register_identity_bridge_
// callback / register_forward_identity_bridge_callback 镜像）→
// `__zw_apply_generation_bump`。

/// slice46 钉①：sel-form（快照 sel 节点）R334 移挂 → apply#1 → R125 remove 后，
/// document 面 QS/QSA 不得再返回该节点（spec 查询面限当下 document tree）；
/// raw `__zw_query_all` 作 host 快照真值对照臂（不经包装/dedup 面）；换代后闭合。
#[test]
fn sel_form_move_apply_remove_document_query_stale_s46() {
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
        "<html><body><div id='w46'><img id='mv46'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s46".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 批 N：sel-form 移挂——getElementById 取快照节点 sel proxy（有 __zwSelector 无
    // __zwHandle），appendChild 到 body（R334 分支：removed 归旧父 div#w46 + added
    // 归 body → 记账对冲后入 PA，`_zwPaGen45` 盖登记代际）。
    sandbox
        .execute(
            "var el = document.getElementById('mv46');\
             document.body.appendChild(el);",
        )
        .unwrap();

    // 批次边界 apply #1：InsertAdjacentSelElement 落快照（body 直含 img）+ 代际 bump。
    // PA 中的 sel 条目跨 apply 保留（R379；K3 清除仅针对无 handle 无 sel 的 parse
    // 补偿节点）。
    let batch1: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(
        !batch1.is_empty(),
        "批 N 必须产出 mutation 记录（RemoveChildAt/InsertAdjacentSelElement）"
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
        .expect("批次边界 apply #1：sel-form 移挂落 host 快照");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
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

    // 批 N+1：apply 已落窗口内 remove → document 面观测。raw 对照臂读 host 快照真值
    //（QSA 聚合计数在 tag 形 + pending 保留场景受 `_elKeyOf` 缺陷污染——slice45 交付
    // 卡遗留申报，本钉不对 apply 前/后未 remove 的 QSA 计数下断言）。
    sandbox
        .execute(
            "globalThis.__r46_st_raw = String(__zw_query_all('img'));\
             el.remove();\
             globalThis.__r46_after_raw = String(__zw_query_all('img'));\
             globalThis.__r46_after_qsa = document.querySelectorAll('img').length;\
             globalThis.__r46_after_qs = String(document.querySelector('img') === null);",
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
        .execute("globalThis.__r46_gen2_qsa = document.querySelectorAll('img').length;")
        .unwrap();

    // 单点聚合断言：红 log 一次带出全部观测值（slice44 打红名单不可追溯教训）。
    let mut read = |name: &str| -> String {
        sandbox
            .execute(&format!("String(globalThis.{name})"))
            .unwrap()
            .value
    };
    let st_raw = read("__r46_st_raw");
    let after_raw = read("__r46_after_raw");
    let after_qsa = read("__r46_after_qsa");
    let after_qs = read("__r46_after_qs");
    let gen2_qsa = read("__r46_gen2_qsa");
    let mut fails: Vec<String> = Vec::new();
    // 对照臂：host 快照真值（remove 前 img 在文档，remove 的 Remove mutation 落快照
    // 前仍在）。raw 值形态为唯一 selector 串（'|' 分隔），内容随 selector 生成规则
    // 定——此处只锚「remove 前非空、remove 后至换代前仍含」的 stale 前提与换代闭合。
    if st_raw.is_empty() || st_raw == "undefined" || st_raw == "null" {
        fails.push(format!("__r46_st_raw: want 非空（host 快照含 img）, got {st_raw:?}"));
    }
    if after_raw.is_empty() || after_raw == "undefined" || after_raw == "null" {
        fails.push(format!(
            "__r46_after_raw: want 非空（Remove mutation 未落快照，host 仍命中——stale 前提）, got {after_raw:?}"
        ));
    }
    // 主红臂：spec 查询面限当下 document tree——remove 后 QS/QSA 不得命中。
    if after_qsa != "0" {
        fails.push(format!("__r46_after_qsa: want \"0\", got {after_qsa:?}"));
    }
    if after_qs != "true" {
        fails.push(format!("__r46_after_qs: want \"true\", got {after_qs:?}"));
    }
    // 换代闭合观测（非红臂——E2 换代清表 + Remove 落快照）。
    if gen2_qsa != "0" {
        fails.push(format!("__r46_gen2_qsa: want \"0\", got {gen2_qsa:?}"));
    }
    assert!(
        fails.is_empty(),
        "sel-form remove 后 document 面查询 stale 观测（诊断：after_qsa/after_qs 红 ⇒ \
         R51c 消零门第 3 合取元（__zwHandle）对 sel-form 恒假，移除不入 _zwPendingRemoved，\
         漏斗无剔除依据；gen2 红 ⇒ 换代不自愈，另案）：\n{}",
        fails.join("\n")
    );
}

/// slice46 钉②：apply 后 tag 形 QSA 对 pending 保留条目不得双计（R161 回退 dedup）。
/// 场景：createElement handle 节点 append → apply#1 落快照（R331 反查把快照命中升格
/// 为原 handle proxy）+ R379 PA 保留同节点 → QSA('div') 快照命中与 pending 回落同
/// 指一节点，`seen161[_elKeyOf(...)]` dedup 须命中。修前 `_elKeyOf` 无定义
/// （ReferenceError 被 catch 吞）→ dedup 恒死 → length 2（双计）。
#[test]
fn qsa_pending_tag_dedup_identity_s46() {
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s46k".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 批 N：createElement（handle-form）→ append（mutation 入队，host 快照未落）。
    sandbox
        .execute("var p = document.createElement('div'); document.body.appendChild(p);")
        .unwrap();

    // 批次边界 apply #1：CreateElement + AppendChild 落快照 + 持久 handle→selector 表
    // + 双向 identity 桥 + 代际 bump。PA 的 handle 条目跨 apply 保留（R379 re-append
    // 移动语义）。
    let batch1: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch1.is_empty(), "批 N 必须产出 mutation 记录（createElement/append）");
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
        .expect("批次边界 apply #1：handle 节点落 host 快照");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    assert!(
        !persistent.is_empty(),
        "apply #1 须产出持久 handle→selector 表（R331 反查前提）"
    );
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

    // 批 N+1：apply 已落窗口内 tag 形 QSA——快照命中（经 R331 反查升格 handle proxy）
    // 与 pending 回落同指 p，dedup 须剔除。
    sandbox
        .execute(
            "globalThis.__r46k_raw = String(__zw_query_all('div'));\
             globalThis.__r46k_len = document.querySelectorAll('div').length;",
        )
        .unwrap();

    let mut read = |name: &str| -> String {
        sandbox
            .execute(&format!("String(globalThis.{name})"))
            .unwrap()
            .value
    };
    let mut fails: Vec<String> = Vec::new();
    // 对照臂：host 快照真值 1 命中（raw 无包装无回退）。
    let raw = read("__r46k_raw");
    if raw.is_empty() || raw == "undefined" || raw == "null" {
        fails.push(format!("__r46k_raw: want 非空（host 快照含 div）, got {raw:?}"));
    }
    // 主红臂：dedup 修复面——apply 后 tag 形 QSA 不双计 pending 保留条目。
    // 修前 got "2"（`_elKeyOf` 无定义 → ReferenceError 被吞 → seen161 恒空）。
    {
        let got = read("__r46k_len");
        if got != "1" {
            fails.push(format!(
                "__r46k_len: want \"1\", got {got:?}（诊断：>1 ⇒ R161 回退 dedup 恒死——\
                 `_elKeyOf` 死引用或键不同构，快照命中与 pending 条目双计）"
            ));
        }
    }
    assert!(
        fails.is_empty(),
        "apply 后 tag 形 QSA pending dedup 观测：\n{}",
        fails.join("\n")
    );
}

/// slice46 钉③（行为钉/申报收窄——固化现状，非 spec 断言）：slice27 静态单命中
/// named access 面不 live。初始快照 id 元素经 `_installNamedAccess`（register_dom_
/// callbacks 注册时自调用，见 callbacks.rs `__zw_collect_ids` 注册后）以数据属性
/// 装全局；元素移除且 Remove 落快照 + 代际 bump 后，属性保持 stale——直至下一次
/// 快照安装（renderer SetDomSnapshot 换代回收/重装链路闭合）。spec 取值算法每读按
/// 当下 named objects 求值（移除后属性应缺席，typeof 应为 "undefined"）——现状为
/// 已知收窄边界，本钉固化防无意识翻转；翻转此断言 = live 化改造落地时点（需
/// WindowProperties exotic 每读求值改造或静态面失格账本，波及 slice27/30/32/33/
/// 36/40/42/43/45 九轮钉网，成本依据见交付卡评估件段）。对照臂：GEBI 同时刻已不
/// 命中（host 快照真值）——查询面 live 而命名面 stale 的边界差即钉面。上游 WPT
/// removing.html 只覆盖 createElement 动态面（slice36/40 已收口），静态安装面无
/// 上游直接用例。
/// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
#[test]
fn static_single_hit_named_access_stale_boundary_s46() {
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
        "<html><body><img id='st46'></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s46g".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    // register_dom_callbacks 尾部自调用 __zwInstallNamedAccess()：静态单命中名
    // st46 以数据属性装上（安装链即被评面）。
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 批 N：remove 入队（host 快照未动）；pre 臂确认安装面在位。
    sandbox
        .execute(
            "globalThis.__r46g_pre_typeof = typeof window.st46;\
             document.getElementById('st46').remove();",
        )
        .unwrap();

    // 批次边界 apply：Remove 落快照（host 真值：img 已不在文档）+ 代际 bump。
    let batch: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch.is_empty(), "remove 必须产出 Remove mutation 记录");
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch);
    let persistent: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        crate::js_dom_bridge::apply_dom_mutations_with_persistent(&mut doc, &batch, Some(&persistent))
            .expect("批次边界 apply：Remove 落 host 快照");
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    // 观测：apply + 换代后——GEBI（host 真值）已不命中；window.st46 数据属性仍
    // 在（stale 至下次快照安装）。
    sandbox
        .execute(
            "globalThis.__r46g_gebi = String(document.getElementById('st46') === null);\
             globalThis.__r46g_typeof = typeof window.st46;\
             globalThis.__r46g_in = String('st46' in window);",
        )
        .unwrap();

    let mut read = |name: &str| -> String {
        sandbox
            .execute(&format!("String(globalThis.{name})"))
            .unwrap()
            .value
    };
    let mut fails: Vec<String> = Vec::new();
    // 安装面 sanity：remove 前属性已装（object）。
    let pre = read("__r46g_pre_typeof");
    if pre != "object" {
        fails.push(format!("__r46g_pre_typeof: want \"object\"（静态安装面在位）, got {pre:?}"));
    }
    // 对照臂：host 快照真值已剔除（查询面 live）。
    let gebi = read("__r46g_gebi");
    if gebi != "true" {
        fails.push(format!(
            "__r46g_gebi: want \"true\"（apply 后 GEBI 不命中——对照臂）, got {gebi:?}"
        ));
    }
    // 主钉臂（固化现状，非 spec 断言）：命名面 stale 保持——属性仍在、typeof 仍
    // "object"。spec 每读求值语义下应为 "undefined"/"false"；翻转 = live 化改造
    // 落地时点（申报收窄边界）。
    let typeof_after = read("__r46g_typeof");
    if typeof_after != "object" {
        fails.push(format!(
            "__r46g_typeof: want \"object\"（现状固化：stale 至下次快照安装）, got \
             {typeof_after:?}（变 \"undefined\" ⇒ 静态面已 live 化，须按交付卡评估件段 \
             重验九轮钉网并更新收窄申报）"
        ));
    }
    let in_after = read("__r46g_in");
    if in_after != "true" {
        fails.push(format!(
            "__r46g_in: want \"true\"（现状固化：属性保持）, got {in_after:?}"
        ));
    }
    assert!(
        fails.is_empty(),
        "slice27 静态单命中面 stale 边界观测（行为钉）：\n{}",
        fails.join("\n")
    );
}
