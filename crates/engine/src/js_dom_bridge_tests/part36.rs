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

// t6：html 未变、drain_gen/style_version 前进走增量换代（复用 cached doc + 幂等
// replay + 变更子树 cascade）——结果必须与全量重算逐位一致（含继承链上的后代）。
#[test]
fn test_generation_cache_incremental_matches_full_compute() {
    let html = "<html><body><div id='d' class='c'><span id='s'>x</span></div>"
        .to_string()
        + "<style>.c { color: blue; display: none }</style></body></html>";
    let m1 = DomMutation::SetStyle {
        selector: "#d".to_string(),
        property: "display".to_string(),
        value: "block".to_string(),
    };
    let m2 = DomMutation::SetStyle {
        selector: "#d".to_string(),
        property: "color".to_string(),
        value: "red".to_string(),
    };
    let style_dbg = |doc: &zero_dom::Document,
                     styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
                     sel: &str| {
        find_by_selector(doc, sel)
            .and_then(|node| styles.get(&node))
            .map(|s| format!("{:?}", s))
            .unwrap_or_default()
    };
    crate::js_dom_bridge::clear_generation_cache();
    GENERATION_FULL_PARSE_COUNT.with(|c| c.set(0));
    // 代 A（冷槽全量）：队列 [m1]。
    with_cached_document_styles(&html, 0, 1, std::slice::from_ref(&m1), |doc, styles| {
        style_dbg(doc, styles, "#d")
    });
    let full_before = GENERATION_FULL_PARSE_COUNT.with(|c| c.get());
    // 代 B（html 同、键前进 → 增量路径）：队列累积为 [m1, m2]。
    let d_inc = with_cached_document_styles(&html, 0, 2, &[m1.clone(), m2.clone()], |doc, styles| {
        (style_dbg(doc, styles, "#d"), style_dbg(doc, styles, "#s"))
    });
    assert_eq!(
        GENERATION_FULL_PARSE_COUNT.with(|c| c.get()),
        full_before,
        "同 html 键前进必须走增量换代、零全量 parse（换代粒度修复的 revert 检出锚点；\
         本断言只经 pending replay 路径，无 DRAIN_RECORD 竞态）"
    );
    // 权威对照：同输入全量重算。
    let (doc, styles) = compute_document_styles_with_inline_overrides(&html, &[m1, m2]);
    assert_eq!(
        d_inc.0,
        style_dbg(&doc, &styles, "#d"),
        "增量换代目标元素样式必须与全量重算逐位一致（inline display/color 压过样式表）"
    );
    assert_eq!(
        d_inc.1,
        style_dbg(&doc, &styles, "#s"),
        "增量换代后代继承样式必须与全量重算逐位一致（color 经 #d 继承）"
    );
}

// t6：attr mutation（属性选择器场景）经增量 cascade 必须重样式化匹配后代——
// replay 后 changed 含 body，body 子树重算让 [data-on] .c 从不匹配转为匹配。
#[test]
fn test_generation_cache_incremental_attr_selector_restyle() {
    let html = "<html><body><div class='c'>x</div><style>[data-on] .c { color: green }</style></body></html>";
    let m = DomMutation::SetAttr {
        selector: "body".to_string(),
        name: "data-on".to_string(),
        value: "1".to_string(),
    };
    let style_dbg = |doc: &zero_dom::Document,
                     styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>| {
        find_by_selector(doc, ".c")
            .and_then(|node| styles.get(&node))
            .map(|s| format!("{:?}", s))
            .unwrap_or_default()
    };
    crate::js_dom_bridge::clear_generation_cache();
    GENERATION_FULL_PARSE_COUNT.with(|c| c.set(0));
    // 代 A（冷槽、无 mutation）：.c 不匹配 [data-on] .c。
    let before = with_cached_document_styles(html, 0, 0, &[], |doc, styles| style_dbg(doc, styles));
    let full_before = GENERATION_FULL_PARSE_COUNT.with(|c| c.get());
    // 代 B（html 同、style_version 前进 → 增量路径）：SetAttr 后 .c 应转绿。
    let after = with_cached_document_styles(html, 0, 1, std::slice::from_ref(&m), |doc, styles| {
        style_dbg(doc, styles)
    });
    assert_eq!(
        GENERATION_FULL_PARSE_COUNT.with(|c| c.get()),
        full_before,
        "同 html 键前进必须走增量换代、零全量 parse（无 DRAIN_RECORD 竞态的第二锚点）"
    );
    assert_ne!(before, after, "attr mutation 必须触发增量重样式化：{:?} -> {:?}", before, after);
    // 权威对照：同输入全量重算。
    let (doc, styles) = compute_document_styles_with_inline_overrides(html, &[m]);
    assert_eq!(
        after,
        style_dbg(&doc, &styles),
        "增量换代结果必须与全量重算逐位一致（属性选择器匹配面）"
    );
}

// t6：drain 记录同步主通道——轮询站点每 tick「结构写 → drain → 快照重序列化 →
// gCS」形态。html 变但 drain 记录衔接时，cached doc 经权威 applier 推进 + 增量
// cascade，结果必须与全量 parse 逐位一致（属性选择器匹配面随 body 属性变化）。
// 接受项（PR #88 复核）：本组测试不 pin「sync 被消费」——全局 DRAIN_RECORD 在并行
// 测试下可被其他测试线程的 clear 抢走，此时走全量兜底、断言仍绿（sync 永久退化为
// fallback 时测试全绿）；路径活性由测试 1/2 的零全量计数断言（无竞态路径）与
// select_option/stale_record 两测的兜底计数间接覆盖。
#[test]
fn test_generation_cache_drain_sync_matches_full_compute() {
    let html0 = "<html><body><div class='c'>x</div><style>[data-on] .c { color: green }</style></body></html>";
    let html1 = "<html><body data-on=\"1\"><div class='c'>x</div><style>[data-on] .c { color: green }</style></body></html>";
    let m = DomMutation::SetAttr {
        selector: "body".to_string(),
        name: "data-on".to_string(),
        value: "1".to_string(),
    };
    let style_dbg = |doc: &zero_dom::Document,
                     styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>| {
        find_by_selector(doc, ".c")
            .and_then(|node| styles.get(&node))
            .map(|s| format!("{:?}", s))
            .unwrap_or_default()
    };
    crate::js_dom_bridge::clear_generation_cache();
    let before = with_cached_document_styles(html0, 0, 0, &[], |doc, styles| style_dbg(doc, styles));
    // drain：apply SetAttr 产新快照 html1（drain_gen 前进、队列清空）→ 发布记录。
    crate::js_dom_bridge::publish_gcs_drain_record(html0, html1, std::slice::from_ref(&m));
    let after = with_cached_document_styles(html1, 1, 0, &[], |doc, styles| style_dbg(doc, styles));
    assert_ne!(before, after, "drain 同步必须反映批内 mutation（.c 转绿）：{:?} -> {:?}", before, after);
    let (doc, styles) = compute_document_styles_with_inline_overrides(html1, &[]);
    assert_eq!(
        after,
        style_dbg(&doc, &styles),
        "drain 同步结果必须与全量重算逐位一致（属性选择器匹配面）"
    );
}

// t6：同批 create+append handle 族经 drain 同步——新元素样式必须随 append 父子树
// 增量 cascade 进入缓存，与全量 parse 一致。
#[test]
fn test_generation_cache_drain_sync_create_append() {
    let html0 = "<html><body><style>.n { color: purple }</style></body></html>";
    let html1 = "<html><body><div class=\"n\">t</div><style>.n { color: purple }</style></body></html>";
    let batch = vec![
        DomMutation::CreateElement {
            handle: "h0".to_string(),
            tag: "div".to_string(),
        },
        DomMutation::SetAttrOnHandle {
            handle: "h0".to_string(),
            name: "class".to_string(),
            value: "n".to_string(),
        },
        DomMutation::AppendChild {
            parent_selector: "body".to_string(),
            child_handle: "h0".to_string(),
        },
    ];
    let style_dbg = |doc: &zero_dom::Document,
                     styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>| {
        find_by_selector(doc, ".n")
            .and_then(|node| styles.get(&node))
            .map(|s| format!("{:?}", s))
            .unwrap_or_default()
    };
    crate::js_dom_bridge::clear_generation_cache();
    with_cached_document_styles(html0, 0, 0, &[], |doc, styles| style_dbg(doc, styles));
    crate::js_dom_bridge::publish_gcs_drain_record(html0, html1, &batch);
    let after = with_cached_document_styles(html1, 1, 0, &[], |doc, styles| style_dbg(doc, styles));
    let (doc, styles) = compute_document_styles_with_inline_overrides(html1, &[]);
    assert_eq!(
        after,
        style_dbg(&doc, &styles),
        "同批 create+append 的新元素样式必须与全量重算一致（append 父子树 cascade 覆盖）"
    );
}

// t6：drain 记录缺失/不衔接（多代未消费、旁路快照变化）→ 全量 parse 兜底，正确性
// 不依赖同步成功。
#[test]
fn test_generation_cache_drain_sync_fallback_without_record() {
    let html0 = "<html><body><div id='d' class='c'>x</div><style>.c { color: blue }</style></body></html>";
    let html1 = "<html><body><div id='d' class='c'>y</div><style>.c { color: blue }</style></body></html>";
    let style_dbg = |doc: &zero_dom::Document,
                     styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>| {
        find_by_selector(doc, ".c")
            .and_then(|node| styles.get(&node))
            .map(|s| format!("{:?}", s))
            .unwrap_or_default()
    };
    crate::js_dom_bridge::clear_generation_cache();
    with_cached_document_styles(html0, 0, 0, &[], |doc, styles| style_dbg(doc, styles));
    // 不发布记录：html 变 → 全量兜底。
    GENERATION_FULL_PARSE_COUNT.with(|c| c.set(0));
    let after = with_cached_document_styles(html1, 1, 0, &[], |doc, styles| style_dbg(doc, styles));
    assert_eq!(
        GENERATION_FULL_PARSE_COUNT.with(|c| c.get()),
        1,
        "无记录的 html 变化必须走全量 parse 兜底"
    );
    let (doc, styles) = compute_document_styles_with_inline_overrides(html1, &[]);
    assert_eq!(
        after,
        style_dbg(&doc, &styles),
        "无 drain 记录时全量 parse 兜底，结果必须与权威一致"
    );
}

// PR #88 复核 D1 钉：sync 换代必须从推进后的 doc 重收集 stylesheets——SetInnerHtml
// 改 `<style>` 文本后新规则必须生效（revert 重收集则 sync 用旧代样式表返旧色）。
#[test]
fn test_generation_cache_sync_recollects_stylesheets() {
    let html0 = "<html><body><div class='n'>t</div><style id='s'>.n { color: purple }</style></body></html>";
    // 渲染器快照重序列化属性用双引号，html1 与之一致。
    let html1 = "<html><body><div class=\"n\">t</div><style id=\"s\">.n { color: green }</style></body></html>";
    let m = DomMutation::SetInnerHtml {
        selector: "#s".to_string(),
        html: ".n { color: green }".to_string(),
    };
    let style_dbg = |doc: &zero_dom::Document,
                     styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>| {
        find_by_selector(doc, ".n")
            .and_then(|node| styles.get(&node))
            .map(|s| format!("{:?}", s))
            .unwrap_or_default()
    };
    crate::js_dom_bridge::clear_generation_cache();
    let before = with_cached_document_styles(html0, 0, 0, &[], |doc, styles| style_dbg(doc, styles));
    crate::js_dom_bridge::publish_gcs_drain_record(html0, html1, std::slice::from_ref(&m));
    let after = with_cached_document_styles(html1, 1, 0, &[], |doc, styles| style_dbg(doc, styles));
    assert_ne!(before, after, "样式文本变更必须反映到换代结果：{:?} -> {:?}", before, after);
    let (doc, styles) = compute_document_styles_with_inline_overrides(html1, &[]);
    assert_eq!(
        after,
        style_dbg(&doc, &styles),
        "sync 换代后样式表必须与全量重算一致（新规则生效，不得沿用旧代 stylesheets）"
    );
}

// PR #88 复核 D2 钉：SelectOption 的权威 applier 改写目标 option 的 selected 属性并
// deselect 兄弟（:checked / option[selected] 匹配面超出目标子树）→ 必须回退全量，
// 结果与全量一致。批含 SelectOption 时 sync 恒拒（白名单外），兜底计数无竞态。
#[test]
fn test_generation_cache_select_option_falls_back_full_parse() {
    let html0 = "<html><body><select id=\"sel\"><option value=\"a\">a</option><option value=\"b\" id=\"b\">b</option></select><style>option:checked { color: green }</style></body></html>";
    let html1 = "<html><body><select id=\"sel\"><option value=\"a\">a</option><option value=\"b\" id=\"b\" selected=\"\">b</option></select><style>option:checked { color: green }</style></body></html>";
    let m = DomMutation::SelectOption {
        selector: "#sel".to_string(),
        value: "b".to_string(),
    };
    let style_dbg = |doc: &zero_dom::Document,
                     styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>| {
        find_by_selector(doc, "#b")
            .and_then(|node| styles.get(&node))
            .map(|s| format!("{:?}", s))
            .unwrap_or_default()
    };
    crate::js_dom_bridge::clear_generation_cache();
    let before = with_cached_document_styles(html0, 0, 0, &[], |doc, styles| style_dbg(doc, styles));
    GENERATION_FULL_PARSE_COUNT.with(|c| c.set(0));
    crate::js_dom_bridge::publish_gcs_drain_record(html0, html1, std::slice::from_ref(&m));
    let after = with_cached_document_styles(html1, 1, 0, &[], |doc, styles| style_dbg(doc, styles));
    assert_eq!(
        GENERATION_FULL_PARSE_COUNT.with(|c| c.get()),
        1,
        "SelectOption 在白名单外：sync 必须拒绝、走全量 parse 兜底"
    );
    assert_ne!(before, after, "选中态变化必须反映到 option 样式：{:?} -> {:?}", before, after);
    let (doc, styles) = compute_document_styles_with_inline_overrides(html1, &[]);
    assert_eq!(
        after,
        style_dbg(&doc, &styles),
        "SelectOption 兜底结果必须与全量重算一致（option:checked 匹配新选中项）"
    );
}

// PR #88 复核（测试面 3）：记录不衔接（old_html 与槽当前代不符 / 被他批覆盖）→
// 全量兜底且兜底计数 +1——「记录死亡」路径的确定性验证。
#[test]
fn test_generation_cache_stale_record_falls_back_full_parse() {
    let html0 = "<html><body><div class='c'>x</div><style>.c { color: blue }</style></body></html>";
    let html1 = "<html><body data-on=\"1\"><div class='c'>x</div><style>[data-on] .c { color: green }</style></body></html>";
    let m = DomMutation::SetAttr {
        selector: "body".to_string(),
        name: "data-on".to_string(),
        value: "1".to_string(),
    };
    let style_dbg = |doc: &zero_dom::Document,
                     styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>| {
        find_by_selector(doc, ".c")
            .and_then(|node| styles.get(&node))
            .map(|s| format!("{:?}", s))
            .unwrap_or_default()
    };
    crate::js_dom_bridge::clear_generation_cache();
    with_cached_document_styles(html0, 0, 0, &[], |doc, styles| style_dbg(doc, styles));
    GENERATION_FULL_PARSE_COUNT.with(|c| c.set(0));
    // 发布不衔接记录：old_html 与槽（html0）不符 → sync 拒绝 → 全量兜底。
    // 他线程抢走记录 / 覆盖记录同样落到本断言（兜底路径恒 +1）。
    crate::js_dom_bridge::publish_gcs_drain_record("<html><body></body></html>", html1, std::slice::from_ref(&m));
    let after = with_cached_document_styles(html1, 1, 0, &[], |doc, styles| style_dbg(doc, styles));
    assert_eq!(
        GENERATION_FULL_PARSE_COUNT.with(|c| c.get()),
        1,
        "不衔接记录必须被拒绝并走全量 parse 兜底"
    );
    let (doc, styles) = compute_document_styles_with_inline_overrides(html1, &[]);
    assert_eq!(
        after,
        style_dbg(&doc, &styles),
        "不衔接记录兜底结果必须与权威一致"
    );
}
