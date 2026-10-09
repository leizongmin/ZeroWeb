// slice47（压实豁免行为钉 + 缺陷修复 + 漏斗 hoist 性能收口）：
// ①件②（slice45 测效 I3 留档）：`_zwAppl45` 压实豁免条目寿命 ≤1 apply 代际。
//   豁免标记语义（part05.js `_zwHCLiveInvalidate` remove 路径）：apply 后跨代际
//   remove 的 pending-added handle 条目经 host 快照真含判定（`_zwHandleInHostSnapshot`
//   ——`__zw_selector_for_handle` + `__zw_contains` + 反查归属三桥核对）确认「快照
//   真含、现被移除」后打标入 removed 表；该条目无 `__zwSelector`（handle-form），
//   512 压实若不豁免会当死数据丢弃 → identity 剔除漏斗失去依据（document QS/QSA
//   对已移除节点 stale）。
//   **修前缺陷（本钉 (a) 臂修前自然红实证，见 evidence green/pin1-*-s47.log）**：
//   豁免标记经代理 set trap R3069 fallthrough 落 per-element `_expando` 旁表，而
//   同一 invalidate 内后执行的 R52 消零清除无条件 `delete _expando['@'+handle]`
//   ——刚打的标记先于任何压实读被湮灭 → 豁免在压实实际发生（>512）时恒失效。
//   slice47 最小修复：R52 expando 删除加 `!_rv._zwAppl45` 守卫（纯消零热路径
//   falsy 照删行为不变，仅豁免条目保留旁表）。本钉固化两段寿命边界：
//   (a) 同代际窗口内豁免条目**跨压实存活**（非死数据）——600 sel-form 条目灌表跨
//       512 阈值触发压实后，handle 豁免条目与 sel 条目同享查询剔除；
//   (b) 下一代际 bump（R3254-E2 整表作废）后条目清空——以 **id 形态 QSA** 为可判别
//       观测面：parsed 条目 sel 为 id 形态（`#bN`，漏斗 dump 实测），换代后重建同 id
//       节点并落快照，漏斗须放行（E2 未清时 stale '#b300' 条目与重挂节点共享同一
//       sel 串 → 漏斗剔除 host 命中）。document QS/gEBI 面被镜像 pending-added 残留
//       回退先行救回（镜像 apply 链不清 `_zwPendingAddedById`，实测 paB300=1）不具
//       判别力。handle-form
//       的跨代际漏斗误用在生产 latest-wins 桥（selector→handle 最新锚定）下被钝化，
//       且测试镜像逐批 re-parse 文档不可表达同 handle 重挂（host 持久 doc 才有
//       handle 注册表；首跑实证 `unknown child handle __n0`，见 evidence
//       green/pin1-first-run-s47.log）——该子面申报收窄，不在此钉。
//   批次边界模拟镜像 part45 钉①装配（drain → append_mutation_history → 持久表
//   apply → 双向 identity 桥 → `__zw_apply_generation_bump`；`__zw_contains` 由
//   register_dom_callbacks 默认注册——js_dom_bridge/callbacks.rs）。
//   https://dom.spec.whatwg.org/#concept-node-list-alive
//   https://dom.spec.whatwg.org/#concept-node-remove
//   https://dom.spec.whatwg.org/#dom-parentnode-queryselector

/// slice47 钉①：`_zwAppl45` 压实豁免条目跨压实存活 + apply 代际边界 E2 清空。
/// 断言翻转语义：(a) 臂翻转 = 压实豁免被撤（R52 守卫被改回无条件删，或**全局表**
/// 压实 keep 谓词 `_zwAppl45` 析取元被删——handle 豁免条目当死数据丢弃，查询
/// stale 复发；**桶级** keep 谓词 part05.js:10437 站点未被本钉覆盖——豁免条目在
/// 本钉路径先进全局表，桶级需 add/remove 桶键形态分裂的窄路径才持有，TEFF-I-1
/// 登记，slice45 起既有缺口）；(b) 臂翻转 = E2 代际清空被撤（stale id 条目经漏斗
/// 误杀换代后重挂节点——id 形态 QSA 判别面）。R52 守卫的 falsy 分支（未打标
/// 照删）由钉② r52_purge_untagged_expando_deletion_s47 单独钉住；打标分支
/// （豁免条目旁表保留）由 (a) 臂端到端覆盖（钉语义不绑守卫读法——返修 F-1 后
/// 守卫为直接旁表读）。
#[test]
fn compaction_exemption_appl45_generation_lifecycle_s47() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut html = String::from("<html><body><img id='keep47'><div id='blk47'>");
    for i in 0..600 {
        html.push_str(&format!("<div id='b{}'>x</div>", i));
    }
    html.push_str("</div></body></html>");

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s47".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 批 N：createElement（handle-form）→ 赋 id → appendChild（mutation 入队）。
    sandbox
        .execute(
            "var d = document.createElement('img');\
             d.id = 'dyn47';\
             document.body.appendChild(d);",
        )
        .unwrap();

    // 批次边界 apply #1：mutation 历史 + 持久表 + 快照落定 + 双向 identity 桥 + 代际
    // bump（→ gen1）。装配镜像 part45 钉①（handle_form_remove_document_query_stale_s45）。
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
        .expect("批次边界 apply #1：handle-form 节点落 host 快照");
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

    // 批 N+1（gen1 窗口）：600 sel-form 条目灌表（跨 512 压实阈值）→ d 跨代际移除
    //（豁免谓词四合取：PA + handle + 代际差 + 快照真含——`__zw_contains` 默认桥在位）
    //→ 再补一笔 mutation 触发压实检查（检查在 invalidate 入口，d 入表后的最后一笔
    // mutation 才会带着 d 过阈值）。
    sandbox
        .execute(
            "document.getElementById('blk47').innerHTML = '';\
             d.remove();\
             var z = document.createElement('br');\
             document.body.appendChild(z);\
             globalThis.__r47_ex_qs = String(document.querySelector('#dyn47') === null);\
             globalThis.__r47_ex_qsa = document.querySelectorAll('img').length;\
             globalThis.__r47_ex_sel = String(document.querySelector('#b300') === null);\
             globalThis.__r47_id = String(document.getElementById('dyn47') === null);",
        )
        .unwrap();

    // 批次边界 apply #2（换代）：Remove 族落快照 → bump → gen2。R3254-E2 整表作废
    // 把 d 的豁免条目一并清空。
    let batch2: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch2.is_empty(), "批量 remove + append 必须产出 mutation 记录");
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch2);
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch2,
            Some(&persistent),
        )
        .expect("批次边界 apply #2：批量 Remove 落 host 快照");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();

    // 批 N+2（gen2）：换代后重建同 id 节点（sel-form；parsed 条目 sel 为 **id 形态**
    // `#bN`——实测漏斗 dump，非位置路径）→ apply #3 落快照 → bump → gen3 → 观测。
    // E2 已清 ⇒ 漏斗 sel 直比无 stale 条目、放行重挂节点；E2 未清 ⇒ stale '#b300'
    // 条目与重挂节点共享同一 sel 串 → id 形态 QSA 剔除（判别面选择见断言注释）。
    sandbox
        .execute(
            "var nb = document.createElement('div');\
             nb.id = 'b300';\
             document.body.appendChild(nb);",
        )
        .unwrap();
    let batch3: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(!batch3.is_empty(), "重挂必须产出 mutation 记录");
    crate::js_dom_bridge::callbacks::append_mutation_history(&batch3);
    {
        let mut doc =
            zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(
            &mut doc,
            &batch3,
            Some(&persistent),
        )
        .expect("批次边界 apply #3：重挂落 host 快照");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();
    sandbox
        .execute(
            "globalThis.__r47_gen_qsa = document.querySelectorAll('#b300').length;\
             globalThis.__r47_gen_img = document.querySelectorAll('img').length;",
        )
        .unwrap();

    // 单点聚合断言（slice44 打红名单不可追溯教训）。
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
    // (a) 豁免条目跨压实存活：d 与 sel-form 条目同享查询剔除（翻转 = 压实豁免被撤）。
    expect(&mut fails, "__r47_ex_qs", "true");
    expect(&mut fails, "__r47_ex_qsa", "1");
    expect(&mut fails, "__r47_ex_sel", "true");
    // ID 面对照臂（identity 消费面同窗闭合，part45 同法）。
    expect(&mut fails, "__r47_id", "true");
    // (b) E2 清空后漏斗放行换代后重挂节点：观测面 = **id 形态 QSA**（R161 pending
    // 回退合并仅纯 tag 形态——part06.js `tagM161` 门；document QS/gEBI 面被镜像
    // pending-added 残留回退先行救回——镜像 apply 链不清 `_zwPendingAddedById`，
    // 实测 paB300=1，不具判别力）。E2 未清 ⇒ stale '#b300' 条目经漏斗剔除 host
    // 命中且无回退 → 0；E2 已清 ⇒ 1。
    expect(&mut fails, "__r47_gen_qsa", "1");
    expect(&mut fails, "__r47_gen_img", "1");
    assert!(
        fails.is_empty(),
        "压实豁免条目寿命观测（诊断：ex_qs=false ⇒ 压实丢弃了 _zwAppl45 条目；\
         gen_qsa=0 ⇒ E2 未清、stale id 条目经漏斗误杀换代后重挂节点）：\n{}",
        fails.join("\n")
    );
}

/// slice47 钉②（TEFF-I-2，PR #115 测效首轮）：R52 守卫 **falsy 分支**——未打标
/// 条目（同 turn createElement→append→remove 消零，从未 apply，R51c 热类）照删
/// per-element `_expando` 旁表。观测面：消零后经原 proxy 读用户 expando 返
/// undefined（旁表删除后 get trap R3042 miss；后续读经 R93 回退取全新 proxy 无
/// expando）。翻转 = 守卫被改宽（如恒不删 expando）→ 用户 expando 存活可读——
/// 性能不变式（R52 原始动机面：消零节点强引用全清）回归的行为显面。打标分支
/// （豁免条目旁表保留）由钉① (a) 臂端到端覆盖，两臂正交：守卫改无条件删时本钉
/// 保持绿而钉① (a) 臂红；守卫改恒不删时本钉红而钉① (a) 臂保持绿。
/// https://dom.spec.whatwg.org/#concept-node-remove
#[test]
fn r52_purge_untagged_expando_deletion_s47() {
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
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s47c".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 同 turn 消零热类：createElement → 用户 expando（旁表落表，R3069）→ append
    //（PA 入表）→ remove（invalidate 内 R51c 消零 + R52 清理：proxy/expando 旁表
    // 照删——未打标 falsy 分支）。无 apply、无 identity 桥（消零路径纯 shim 侧）。
    sandbox
        .execute(
            "var d = document.createElement('img');\
             d.foo47 = 'bar';\
             globalThis.__r47c_pre = String(d.foo47);\
             document.body.appendChild(d);\
             d.remove();\
             globalThis.__r47c_post = String(d.foo47);",
        )
        .unwrap();

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
    // 前置 sanity：expando set/get 往返成立（排除观测面自身失真）。
    expect(&mut fails, "__r47c_pre", "bar");
    // falsy 分支语义：消零后旁表已删 → expando 读回 undefined（翻转 = 守卫被改宽
    // 恒不删 → 'bar' 存活）。变异 RED：M-C（删除行整体中和）唯一杀死本断言。
    expect(&mut fails, "__r47c_post", "undefined");
    assert!(
        fails.is_empty(),
        "R52 消零未打标条目 expando 照删观测（诊断：post='bar' ⇒ 守卫被改宽、\
         消零节点旁表强引用残留，R52 泄漏修复面回归）：\n{}",
        fails.join("\n")
    );
}
