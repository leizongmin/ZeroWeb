// T68（site-compat juejin feed 归因轮，2026-11-14）：sel/handle 双族注册表 sync
// 常驻钉——t67-minrepro 复现页核心四案例语义（caseA class 形态 QSA 归并 /
// caseC removeChild 序列化空串 / caseDAsync drain 后 childNodes identity /
// fragment 视图 handle 族子宿主 wire）。四案同指 T67 定谳缺口「JS 指针视图
// 已挂而宿主注册表（文档级 QSA/TreeWalker）不同步」。
// 规范锚：https://dom.spec.whatwg.org/#concept-node-pre-insert
//         https://dom.spec.whatwg.org/#concept-node-remove
//         https://dom.spec.whatwg.org/#concept-tree-child（live 集合 identity）

/// 案 A 钉：handle 族子（createElement 产物）insertBefore 进 sel-only 解析容器
///（querySelector 产物）——同步窗口内（宿主 apply 未 drain）文档级
/// `querySelectorAll('.cls')` 的 **class 形态**经 R3902b pending 归并命中
///（修复前无归并臂恒 miss），childNodes/innerHTML 融合视图同窗一致。
#[test]
fn t68_selhandle_insert_qsa_class_merge_case_a() {
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
        "<html><body><div id=\"t68a\" class=\"wrap\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/t68a".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var wrap = document.getElementById('t68a');\
             var kid = document.createElement('div');\
             kid.className = 't68mk';\
             wrap.insertBefore(kid, null);\
             globalThis.__kid = kid; globalThis.__wrap = wrap;",
        )
        .unwrap();

    // 同步窗口视图一致性：指针视图 + 文档级 class 查询 + 融合序列化。
    assert_eq!(
        sandbox.execute("String(__wrap.childNodes.length)").unwrap().value,
        "1",
        "sel-only 容器 childNodes 同窗含 handle 子"
    );
    assert_eq!(
        sandbox.execute("String(__wrap.childNodes[0] === __kid)").unwrap().value,
        "true",
        "childNodes 成员 = 页面持有的原 handle proxy（spec live identity）"
    );
    assert_eq!(
        sandbox.execute("String(__kid.parentNode === __wrap)").unwrap().value,
        "true",
        "parentNode 反链指容器"
    );
    assert_eq!(
        sandbox.execute("String(document.querySelectorAll('.t68mk').length)").unwrap().value,
        "1",
        "文档级 QSA class 形态同窗命中（R3902b pending 归并——T68 扩 class 形态前恒 0）"
    );
    assert_eq!(
        sandbox
            .execute("String(document.querySelectorAll('.t68mk')[0] === __kid)")
            .unwrap()
            .value,
        "true",
        "QSA 命中成员 identity = 原 handle proxy"
    );
    assert!(
        sandbox
            .execute("String(__wrap.innerHTML)")
            .unwrap()
            .value
            .contains("t68mk"),
        "融合 innerHTML 含 pending 子（R380 walk 序列化）"
    );

    // 宿主 wire 在册：AppendChild { parent=容器 sel, child=handle }。
    let q = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    assert!(
        q.iter()
            .any(|m| matches!(m, DomMutation::AppendChild { .. })),
        "插入必须产出 AppendChild wire（宿主落地凭据）；queue={q:?}"
    );
}

/// 案 C 钉：sel-only 容器 removeChild 唯一子后 innerHTML 同窗为**空串**（融合
/// 序列化合法结果）——修复前 `_zwFusedInnerHtmlSel` 尾部 `!== '' → null` 折叠 +
/// getter `||` 回落把宿主 stale 快照交回（htmlGone=false 根因）。
#[test]
fn t68_remove_child_fused_empty_serialization_case_c() {
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
        "<html><body><div id=\"t68c\"><p class=\"oldc\">x</p></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/t68c".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var w = document.getElementById('t68c');\
             var c = w.firstElementChild;\
             globalThis.__w = w; globalThis.__c = c;\
             w.removeChild(c);",
        )
        .unwrap();

    // 指针/集合视图移除即时收敛（spec concept-node-remove：父无子）。
    assert_eq!(
        sandbox.execute("String(__w.childNodes.length)").unwrap().value,
        "0",
        "childNodes 移除即时可见"
    );
    assert_eq!(
        sandbox.execute("String(__w.contains(__c))").unwrap().value,
        "false",
        "contains 移除后 false"
    );
    // 核心：innerHTML 空串（不回落 stale host 快照）。
    assert_eq!(
        sandbox.execute("String(__w.innerHTML)").unwrap().value,
        "",
        "移除唯一子后 innerHTML 同窗为空串（T68 空串保留——修复前回落宿主 stale 串）"
    );
    // 宿主快照在 drain 前仍含旧子（异步 apply 语义未变——移除以 Remove mutation 在册）。
    let q = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    assert!(
        q.iter().any(|m| matches!(m, DomMutation::Remove { .. })),
        "removeChild 必须产出 Remove wire；queue={q:?}"
    );
}

/// 案 DAsync 钉：drain（apply + identity 桥 + 代际 bump）后 sel-only 容器
/// childNodes 成员 identity = 页面持有的原 handle proxy——`_wrapNodeEntry` 'E'
/// 臂接 `_zwQueryWrapIdentity` 反查（修复前裸 `_wrapSelector` 产新 proxy，
/// indexOf 按对象恒等判 miss）。part45 批次边界模式（drain → mutation 历史 →
/// persistent 表 → 快照落定 → 双向 identity 桥 → bump）。
#[test]
fn t68_drain_after_identity_reverse_lookup_case_d_async() {
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
        "<html><body><div id=\"t68d\" class=\"wrap\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/t68d".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // turn 1：handle 子插入 sel-only 容器（vue-like swap 面）。
    sandbox
        .execute(
            "var wrap = document.getElementById('t68d');\
             var kid = document.createElement('div');\
             kid.className = 't68mk';\
             wrap.insertBefore(kid, null);\
             globalThis.__kid = kid; globalThis.__wrap = wrap;",
        )
        .unwrap();

    // 批次边界：drain → mutation 历史 → persistent handle→sel 表 → 快照落定
    // → 双向 identity 桥（webview register_{,forward_}identity_bridge 镜像）→ 代际 bump。
    let batch1: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch1.is_empty(), "批 N 必须产出 mutation 记录");
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
        .expect("批次边界 apply：handle 子落宿主快照");
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
        .execute("if (typeof globalThis.__zw_apply_generation_bump === 'function') globalThis.__zw_apply_generation_bump();{}")
        .unwrap();

    // turn 2：drain 后双面一致——宿主快照含子 + childNodes identity 保持。
    assert_eq!(
        sandbox
            .execute("String(document.querySelectorAll('.t68mk').length)")
            .unwrap()
            .value,
        "1",
        "drain 后宿主注册表 class 查询命中"
    );
    assert_eq!(
        sandbox
            .execute("String(__wrap.childNodes.length === 1 && __wrap.childNodes[0] === __kid)")
            .unwrap()
            .value,
        "true",
        "drain 后 childNodes 成员 identity = 原 handle proxy（'E' 臂 R331 反查——T68 修复前裸 wrapSelector 产新 proxy 恒 false）"
    );
    assert_eq!(
        sandbox.execute("String(__kid.parentNode === __wrap)").unwrap().value,
        "true",
        "drain 后 parentNode 反链保持"
    );
    assert_eq!(
        sandbox.execute("String(document.querySelector('.t68mk') === __kid)").unwrap().value,
        "true",
        "QS 命中 identity = 原 handle proxy（R331 既有语义对照臂）"
    );
    assert!(
        sandbox
            .execute("String(__wrap.innerHTML)")
            .unwrap()
            .value
            .contains("t68mk"),
        "drain 后 innerHTML 含子（宿主直读或融合一致）"
    );
}

/// 修法 2 钉（t8i sel-only 容器臂）：无 handle fragment 视图（createContextualFragment
/// 产物）携 handle 族子 insertBefore 进 sel-only 容器——handle 子补发宿主 wire
/// `AppendChild { parent=容器 sel, child=handle }`（修复前全臂 noWire 静默缺落），
/// 同窗视图/文档级查询经 pending 归并一致，apply 后子落宿主快照。
#[test]
fn t68_fragment_view_handle_child_wire_t8i_sel_container() {
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
        "<html><body><div id=\"t68f\" class=\"wrap\"></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/t68f".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var wrap = document.getElementById('t68f');\
             var range = document.createRange();\
             range.selectNodeContents(document.body);\
             var frag = range.createContextualFragment('<span class=\"t68p\">p</span>');\
             var hk = document.createElement('b');\
             hk.className = 't68fk';\
             frag.appendChild(hk);\
             globalThis.__hkHandle = String(hk.__zwHandle);\
             wrap.insertBefore(frag, null);\
             globalThis.__hk = hk; globalThis.__fw = wrap;",
        )
        .unwrap();

    // 同步窗口：fragment 子展开可见（解析 span + handle b）。
    assert_eq!(
        sandbox.execute("String(__fw.childNodes.length)").unwrap().value,
        "2",
        "sel-only 容器 childNodes 同窗含 fragment 两子（overlay 承载）"
    );
    assert_eq!(
        sandbox.execute("String(__fw.childNodes[1] === __hk)").unwrap().value,
        "true",
        "handle 子 identity 保持"
    );
    assert_eq!(
        sandbox
            .execute("String(document.querySelectorAll('.t68fk').length)")
            .unwrap()
            .value,
        "1",
        "文档级 class 查询同窗命中（pending 归并）"
    );
    // 修法 2 核心：handle 子的宿主 wire 在册。
    let hk_handle = sandbox
        .execute("globalThis.__hkHandle")
        .unwrap()
        .value
        .to_string();
    let q = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    assert!(
        q.iter().any(|m| matches!(
            m,
            DomMutation::AppendChild { child_handle, .. } if *child_handle == hk_handle
        )),
        "t8i handle 子必须产出 AppendChild wire（修复前 noWire 静默）；queue={q:?}"
    );

    // 批次边界 apply：wire 可消费，handle 子落宿主快照（「宿主落地」证明）。
    let batch: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    let mut doc = zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
    crate::js_dom_bridge::apply_dom_mutations(&mut doc, &batch)
        .expect("t8i wire 批 apply 必须成功（ref/handle 在批内 CreateElement 先序注册）");
    let out = doc.outer_html(doc.root());
    assert!(
        out.contains("t68fk"),
        "apply 后宿主快照含 handle 子；html={out}"
    );
}

/// 修法 2 钉（R97 handle 容器臂）：无 handle fragment 视图携 handle 族子
/// insertBefore 进 handle 容器（createElement 产物）——handle 子补发宿主 wire
/// `AppendChildByHandle { parent=容器 handle, child=handle }`（修复前只做 JS
/// registry splice，宿主树永久缺席），同批 CreateElement 先序注册下 apply 成功。
#[test]
fn t68_fragment_view_handle_child_wire_r97_handle_container() {
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/t68r".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var hc = document.createElement('ul');\
             var range = document.createRange();\
             range.selectNodeContents(document.body);\
             var frag = range.createContextualFragment('<li class=\"t68p\">p</li>');\
             var hk = document.createElement('li');\
             hk.className = 't68fk2';\
             frag.appendChild(hk);\
             globalThis.__hkHandle2 = String(hk.__zwHandle);\
             globalThis.__hcHandle = String(hc.__zwHandle);\
             hc.insertBefore(frag, null);\
             globalThis.__hk2 = hk; globalThis.__hc = hc;",
        )
        .unwrap();

    // JS registry 权威视图（handle 容器以 `_handleChildren` 为准）。
    assert_eq!(
        sandbox.execute("String(__hc.childNodes.length)").unwrap().value,
        "2",
        "handle 容器 registry 同窗含 fragment 两子"
    );
    assert_eq!(
        sandbox.execute("String(__hc.childNodes[1] === __hk2)").unwrap().value,
        "true",
        "handle 子 identity 保持"
    );
    // 修法 2 核心：`AppendChildByHandle` wire 在册。
    let hk_handle = sandbox
        .execute("globalThis.__hkHandle2")
        .unwrap()
        .value
        .to_string();
    let hc_handle = sandbox
        .execute("globalThis.__hcHandle")
        .unwrap()
        .value
        .to_string();
    let q = mutations.lock().unwrap_or_else(|e| e.into_inner()).clone();
    assert!(
        q.iter().any(|m| matches!(
            m,
            DomMutation::AppendChildByHandle { parent_handle, child_handle }
                if *child_handle == hk_handle && *parent_handle == hc_handle
        )),
        "R97 handle 子必须产出 AppendChildByHandle wire（修复前仅 JS registry）；queue={q:?}"
    );

    // 批内先序 CreateElement 注册后 apply 成功（wire 可消费）。
    let batch: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    let mut doc = zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
    crate::js_dom_bridge::apply_dom_mutations(&mut doc, &batch)
        .expect("R97 wire 批 apply 必须成功");
}

/// 修法 5 钉（s165 跨批 ref 失效降级 append）：`InsertBefore { parent_selector,
/// child_handle, ref_selector }` 的 ref 三查全 miss（selector / 批内预解析 /
/// handle 表——ref handle 经 `refresh_if_html_changed` 全量重建换代后失效的
/// 形态）时**降级 append 尾部落地**，不再 warn + skip 整条丢弃。对齐
/// [`DomMutation::InsertBeforeByHandleHandle`] 的 ref miss lenient 约定
///（「ref 不在树中的语义等价——insertBefore(node, null) == appendChild」）。
/// 活体锚：掘金 feed 挂载窗口 22 次 `insert_before ref no match, skipped:
/// __n69`（vue entry-list/卡片挂接走 s165 臂，宿主零落地而 JS pending 桶视图
/// 稳定呈现——视图/宿主分裂最终一环）。同批对照面：ref 有效时仍精确前插。
/// 规范锚：https://dom.spec.whatwg.org/#concept-node-pre-insert
#[test]
fn t68_s165_stale_ref_degrade_append_lands() {
    // 活体形态：ref 填 handle 串但 handle 表无此条目（跨批换代失效）——
    // selector 查找 miss、ins_targets miss、handles miss。
    let html = "<html><body><div id=\"t68s165p\"><span class=\"head\">h</span></div></body></html>";
    let mutations = vec![
        DomMutation::CreateElement {
            handle: "__nkid".into(),
            tag: "p".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__nkid".into(),
            name: "class".into(),
            value: "t68s165kid".into(),
        },
        // s165 臂 wire 形态：ref_selector = handle 字面（跨批后表内无此条目）。
        DomMutation::InsertBefore {
            parent_selector: "#t68s165p".into(),
            child_handle: "__nkid".into(),
            ref_selector: "__n69".into(),
        },
    ];
    let out = crate::js_dom_bridge::apply_mutations_to_html(html, &mutations)
        .expect("stale ref 必须降级 append 而非整批失败");
    assert!(
        out.contains("t68s165kid"),
        "ref 三查全 miss 时子必须降级 append 落地（修复前 skip 静默丢弃）；html={out}"
    );
    assert!(
        out.contains("class=\"t68s165kid\"></p></div>"),
        "降级 append 落父尾部（ref 不在树中无锚序可保）；html={out}"
    );

    // 对照面：ref 有效（selector 可解析）时仍精确前插，不被降级影响。
    let mutations_ok = vec![
        DomMutation::CreateElement {
            handle: "__nkid2".into(),
            tag: "p".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__nkid2".into(),
            name: "class".into(),
            value: "t68s165ok".into(),
        },
        DomMutation::InsertBefore {
            parent_selector: "#t68s165p2".into(),
            child_handle: "__nkid2".into(),
            ref_selector: "#t68s165ref".into(),
        },
    ];
    let html2 = "<html><body><div id=\"t68s165p2\"><span id=\"t68s165ref\">r</span></div></body></html>";
    let out2 = crate::js_dom_bridge::apply_mutations_to_html(html2, &mutations_ok)
        .expect("ref 有效时精确 insert 必须成功");
    let kid_pos = out2.find("t68s165ok").expect("子落地");
    let ref_pos = out2.find("t68s165ref").expect("ref 存在");
    assert!(
        kid_pos < ref_pos,
        "ref 有效时子前插于 ref（精确 insertBefore 语义保持）；html={out2}"
    );
}

/// 案 G 钉（T68 第六面）：handle miss lenient 化不毒化批——跨批 refresh 重建使
/// handle 表换代清空是活体常态（掘金 feed `unknown handle`/`unknown parent
/// handle` 每次硬错中止整批，批内后续 sel 域 mutation 全部丢弃，宿主/视图
/// 分裂放大）。 miss 条目 warn+跳过后，批内其余 mutation 必须照常落地。
#[test]
fn t68_handle_miss_lenient_no_batch_poison() {
    // 属性面：批首 SetAttrOnHandle 引用跨批失效 handle——旧版整批 Err，
    // 批尾 sel 域 AppendChild 随批全丢；新版前条 skip、后条落地。
    let html = "<html><body><div id=\"t68g1\"><span class=\"keep\">k</span></div></body></html>";
    let mutations = vec![
        DomMutation::SetAttrOnHandle {
            handle: "__nstale2224".into(),
            name: "data-x".into(),
            value: "v".into(),
        },
        DomMutation::CreateElement {
            handle: "__nkid".into(),
            tag: "p".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__nkid".into(),
            name: "class".into(),
            value: "t68gkid".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#t68g1".into(),
            child_handle: "__nkid".into(),
        },
    ];
    let out = crate::js_dom_bridge::apply_mutations_to_html(html, &mutations)
        .expect("handle miss 必须跳过该条而非中止整批");
    assert!(
        out.contains("t68gkid"),
        "批内 miss 条目后的 sel 域 mutation 必须照常落地；html={out}"
    );
    assert!(
        !out.contains("data-x"),
        "miss 条目本身不产出属性（skip 语义，非静默成功）；html={out}"
    );

    // 结构面：批首 AppendChildByHandle parent handle miss（活体 __n232 形态），
    // 同批后条 sel 域插入不受毒化。
    let mutations2 = vec![
        DomMutation::AppendChildByHandle {
            parent_handle: "__nstale232".into(),
            child_handle: "__norphan".into(),
        },
        DomMutation::CreateElement {
            handle: "__nkid2".into(),
            tag: "p".into(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "__nkid2".into(),
            name: "class".into(),
            value: "t68gkid2".into(),
        },
        DomMutation::AppendChild {
            parent_selector: "#t68g2".into(),
            child_handle: "__nkid2".into(),
        },
    ];
    let html2 = "<html><body><div id=\"t68g2\"><span>i</span></div></body></html>";
    let out2 = crate::js_dom_bridge::apply_mutations_to_html(html2, &mutations2)
        .expect("parent handle miss 必须跳过该条而非中止整批");
    assert!(
        out2.contains("t68gkid2"),
        "parent handle miss 后同批 sel 域插入必须落地；html={out2}"
    );
}
