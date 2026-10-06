// part36：E14 同族收口（PR #83 复核遗留）——per-selector `cs_cache` 键并入 drain_gen
// 与 `clear_generation_cache` 直接单测。

#[test]
fn test_get_computed_style_per_selector_cache_drain_collision() {
    // per-selector `cs_cache` 短路先于 doc 级代际缓存：drain 后队列重长回同 len、
    // html 未变、内容不同——键缺 drain_gen 时已缓存 selector 返上一批 stale 样式
    //（PR #83 复核遗留 minor，E14 同族）。沙箱级走真实 `__zw_get_computed_style` 通路。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id=\"d\"></div><style>#d { display: none }</style></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 批 1（drain_gen 0、len=1）：inline display:block 压过样式表 none，填 per-selector 缓存。
    mutations
        .lock()
        .unwrap()
        .push(DomMutation::SetStyle {
            selector: "#d".to_string(),
            property: "display".to_string(),
            value: "block".to_string(),
        });
    sandbox
        .execute("globalThis.__v1 = getComputedStyle(document.querySelector('#d')).display;")
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__v1").unwrap().value, "block");

    // drain：清队列 + bump 代际；批 2 重长回同 len=1、html 未变、内容不同。
    mutations.lock().unwrap().clear();
    crate::js_dom_bridge::bump_mut_drain_gen();
    mutations
        .lock()
        .unwrap()
        .push(DomMutation::SetStyle {
            selector: "#d".to_string(),
            property: "display".to_string(),
            value: "inline".to_string(),
        });
    sandbox
        .execute("globalThis.__v2 = getComputedStyle(document.querySelector('#d')).display;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__v2").unwrap().value,
        "inline",
        "drain 后同 len 不同内容：per-selector 缓存必须随 drain_gen 换代，不得返批 1 的 block"
    );
}

#[test]
fn test_clear_generation_cache_forces_recompute() {
    // `clear_generation_cache` 直接单测（PR #83 复核 nit）：清槽后同键查询必须重算，
    // 命中路径不得复用清槽前的 doc。用 cfg(test) 重算计数判定——地址比对有 ABA
    // 巧合风险（清槽先释放旧 doc，新 parse 理论上可落回同址），计数判定确定性强。
    let html = "<html><body><div id='a'></div></body></html>";
    crate::js_dom_bridge::clear_generation_cache();
    GENERATION_RECOMPUTE_COUNT.with(|c| c.set(0));
    with_cached_document_styles(html, 0, 0, &[], |_, _| {});
    with_cached_document_styles(html, 0, 0, &[], |_, _| {});
    GENERATION_RECOMPUTE_COUNT.with(|c| {
        assert_eq!(
            c.get(),
            1,
            "冷槽首查重算一次，同代际第二查命中缓存不重算"
        );
    });
    crate::js_dom_bridge::clear_generation_cache();
    with_cached_document_styles(html, 0, 0, &[], |_, _| {});
    GENERATION_RECOMPUTE_COUNT.with(|c| {
        assert_eq!(
            c.get(),
            2,
            "clear 后同键查询必须重算（register_dom_callbacks 装新快照依赖该语义）"
        );
    });
}
