// 缺陷2（M3，2026-11-04，R100 宿主半边接线钉）：renderer automation drain 后 handle 属性
// 读——收件箱桥。生产链：`apply_recorded_mutations`（renderer 主线程）drain →
// `append_applied_mutation_history`（收件箱；宿主半边此前未接线，见 callbacks.rs
// `__zw_get_attr_handle` 第 2 层注）→ apply；js_worker 线程读回调当前批 miss →
// `sync_mutation_history_inbox` 收箱进 MUTATION_HISTORY → 历史回落。接线前该读恒落空
//（probe7 实证：dataset.seq 同任务内翻转 null；基准 mut.replace.pool 语义 FAIL 228.8x）。
// 单线程测试按角色模拟：本线程 = worker 线程（publish 收件箱 + 执行读）；宿主角色 =
// drain 队列后对收件箱 Arc append（生产 `RendererJsWorker::append_applied_mutation_history`
// 同参同锁，装配序 drain → append → apply 镜像 part46/48 钉①）。
// https://dom.spec.whatwg.org/#dom-element-getattribute
// https://dom.spec.whatwg.org/#dom-element-dataset

/// 钉①：dataset/getAttribute/hasAttribute 三面跨 drain+apply 存活；dataset 枚举
///（`__zw_attr_names_handle` 历史层合并）同批收口。同批正向臂（apply 前）既有语义对照。
#[test]
fn handle_attr_read_survives_drain_via_history_inbox_m3d2() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/m3d2".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // worker 角色发布收件箱（js_worker_main 启动臂同调用）。
    let inbox: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(Vec::new()));
    crate::js_dom_bridge::publish_mutation_history_inbox(Arc::clone(&inbox));

    // 批 N：createElement → class/data-seq 写（dataset 面）→ appendChild。同批正向臂：
    // 当前批 latest-wins 即时可见（既有语义，apply 前不变——probe7 after-set/after-replace-sync）。
    sandbox
        .execute(
            "var d = document.createElement('div');\
             d.className = 'item';\
             d.dataset.seq = '7';\
             document.body.appendChild(d);\
             globalThis.__m3d2_same = d.getAttribute('data-seq') + '|' + Object.keys(d.dataset).join(',') + '|' + d.className;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__m3d2_same").unwrap().value,
        "7|seq|item",
        "同批 dataset/getAttribute/枚举读即时可见"
    );

    // 宿主角色装配：drain → append 收件箱（生产 append_applied_mutation_history 同体）→
    // apply 落快照（handle→selector 持久表 + 换代）→ 代际 bump → 下一 execute 前
    // clear_mutations_fresh 等价收口（drain ⇒ bump 不变式）。
    let batch: Vec<DomMutation> = {
        let mut guard = mutations.lock().unwrap_or_else(|e| e.into_inner());
        guard.drain(..).collect()
    };
    assert!(
        batch.iter().any(|m| matches!(m, DomMutation::SetAttrOnHandle { handle: _, name, .. } if name == "data-seq")),
        "批 N 必须含 dataset 写（SetAttrOnHandle{{data-seq}}）"
    );
    {
        // 生产宿主侧（RendererJsWorker::append_applied_mutation_history 体）：锁内 extend。
        inbox
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .extend(batch.iter().cloned());
    }
    let mut persistent: std::collections::HashMap<String, String> = std::collections::HashMap::new();
    {
        let mut doc = zero_dom::parse_html(&dom_html.lock().unwrap_or_else(|e| e.into_inner()).clone());
        let hs = crate::js_dom_bridge::apply_dom_mutations_with_persistent(&mut doc, &batch, Some(&persistent))
            .expect("批次边界 apply：createElement/class/data-seq 落 host 快照");
        persistent.extend(hs.iter().map(|(k, v)| (k.clone(), v.clone())));
        *dom_html.lock().unwrap_or_else(|e| e.into_inner()) = doc.outer_html(doc.root());
    }
    sandbox
        .execute("globalThis.__zw_apply_generation_bump && globalThis.__zw_apply_generation_bump();")
        .unwrap();
    {
        mutations.lock().unwrap_or_else(|e| e.into_inner()).clear();
        crate::js_dom_bridge::bump_mut_drain_gen();
    }

    // 批 N+1：当前批空——读回调须收宿主已应用批进历史后回落（接线前此处恒落空）。
    sandbox
        .execute(
            "globalThis.__m3d2_after = d.getAttribute('data-seq') + '|' + Object.keys(d.dataset).join(',') + '|' + d.className;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__m3d2_after").unwrap().value,
        "7|seq|item",
        "drain+apply 后 dataset/getAttribute/枚举读经收件箱→历史回落存活（缺陷2 语义）"
    );
    // has 面：公共 API 内部走 `__zw_has_attr_handle` 同回落链。
    let has = sandbox.execute("String(d.hasAttribute('data-seq'))").unwrap().value;
    assert_eq!(has, "true", "hasAttribute 跨 drain+apply 存活（has 面同回落链）");
    // 收件箱已搬空（mem::take）——二次读走纯历史，幂等不重放错值。
    assert!(
        inbox.lock().unwrap_or_else(|e| e.into_inner()).is_empty(),
        "收件箱须在读回调入口整批搬空（sync_mutation_history_inbox 消费语义）"
    );
    assert_eq!(
        sandbox.execute("d.getAttribute('data-seq')").unwrap().value,
        "7",
        "二次读走纯历史（收件箱已空）幂等"
    );
}
