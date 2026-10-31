// t-p126 S1c：worker 查询物化 memo（线程本地，按视图代键）的语义钉定。
//
// memo 是纯函数缓存：同视图代 token 内命中必须与 uncached 重算逐值等价——唯一
// 选择器是 shim 代理层的寻址身份（R2663），错值即错绑。本组钉：cached/uncached
// 全元素对拍（重复 id / 重复 tag.class / name/target 锚 / 深链无 id/class）、
// token 失效（换 doc 换 token 必须重算）、封顶清空后仍正确、find_by_selector
// cached 对拍。各测试用互异 html_ptr 假值，防同进程 thread_local memo 串场。

#[test]
fn s1c_uniq_selector_cached_parity_with_uncached() {
    use crate::js_dom_bridge::{find_by_selector, unique_selector_for_node, unique_selector_for_node_cached, UniqSelCtx};
    // 重复 id、重复 tag.class、同名 input、唯一 target 锚、无 id/class 深链。
    let html = "<html><body>\
        <div id=\"dup\">d1</div><div id=\"dup\">d2</div>\
        <div class=\"card\">a</div><div class=\"card\">b</div>\
        <form><input name=\"q\"></form><form><input name=\"q\"></form>\
        <section id=\"root-anchor\"><ul><li><p><span>deep</span></p></li></ul></section>\
        <div><span>shallow</span></div>\
        <a target=\"unique-target\">t</a>\
        </body></html>";
    let doc = crate::js_dom_bridge::parse_html(html);
    let ctx = UniqSelCtx::new(0xdead, 0, 7, 9, doc.node_count());
    let all = doc.query_selector_all(doc.root(), "*");
    assert!(all.len() >= 12);
    for id in all {
        let uncached = unique_selector_for_node(&doc, id);
        let cached = unique_selector_for_node_cached(&doc, ctx, id);
        assert_eq!(cached, uncached, "parity broken at {id:?}");
        // R2663 语义：物化出的 selector 必须唯一定位回原节点。
        if let Some(sel) = &cached {
            assert_eq!(find_by_selector(&doc, sel), Some(id), "selector {sel} 未能回定原节点");
        }
    }
    // 重复 id 的唯一性门必须拒绝 `#dup`（两元素都回落）。
    let dups = doc.query_selector_all(doc.root(), "#dup");
    assert_eq!(dups.len(), 2);
    for id in dups {
        assert_ne!(unique_selector_for_node_cached(&doc, ctx, id).as_deref(), Some("#dup"));
    }
    // 重复 tag.class 门拒绝 `div.card`。
    let cards = doc.query_selector_all(doc.root(), ".card");
    assert_eq!(cards.len(), 2);
    for id in cards {
        assert_ne!(unique_selector_for_node_cached(&doc, ctx, id).as_deref(), Some("div.card"));
    }
    // 深链 span 借唯一祖先锚落根。
    let span = doc.query_selector(doc.root(), "span").expect("span");
    let sel = unique_selector_for_node_cached(&doc, ctx, span).expect("deep span 可寻址");
    assert!(sel.starts_with("#root-anchor >"), "deep span sel = {sel}");
    // 同 token 二次取值：命中 memo，与首算逐值等价。
    let again = unique_selector_for_node_cached(&doc, ctx, span);
    assert_eq!(again, unique_selector_for_node(&doc, span));
}

#[test]
fn s1c_memo_keys_on_view_generation() {
    use crate::js_dom_bridge::{parse_html, unique_selector_for_node_cached, UniqSelCtx};
    // 代 A：唯一 `.card` → stable `div.card`。
    let doc1 = parse_html("<html><body><div class=\"card\">a</div></body></html>");
    let node1 = doc1.query_selector(doc1.root(), "div").expect("div in doc1");
    let ctx_a = UniqSelCtx::new(1, 0, 0, 0, doc1.node_count());
    assert_eq!(unique_selector_for_node_cached(&doc1, ctx_a, node1).as_deref(), Some("div.card"));
    // 代 B：第二个 `.card` 进场，唯一性门必须重算拒绝 stable 形态（不得命中 A 代
    // 缓存的 "div.card"）→ 回落结构路径。
    let doc2 = parse_html("<html><body><div class=\"card\">a</div><div class=\"card\">b</div></body></html>");
    let node2 = doc2.query_selector(doc2.root(), "div").expect("div in doc2");
    let ctx_b = UniqSelCtx::new(1, 1, 0, 0, doc2.node_count());
    let b = unique_selector_for_node_cached(&doc2, ctx_b, node2).expect("结构路径可寻址");
    assert!(b.starts_with("html:nth-child(1)"), "代 B 应回落结构路径，实得 {b}");
    // 同 doc2 同 ctx_b 复取：命中 memo，与首算逐值等价。
    assert_eq!(unique_selector_for_node_cached(&doc2, ctx_b, node2), Some(b));
}

#[test]
fn s1c_memo_cap_overflow_stays_correct() {
    use crate::js_dom_bridge::{parse_html, unique_selector_for_node_cached, UniqSelCtx};
    let doc = parse_html("<html><body><div class=\"c\">x</div></body></html>");
    let node = doc.query_selector(doc.root(), "div").expect("div");
    // 越过封顶（UNIQ_MEMO_CAP=8192，溢出整体清空——性能换语义无损）。
    for i in 0..8210usize {
        let ctx = UniqSelCtx::new(2, i, 0, 0, doc.node_count());
        assert_eq!(
            unique_selector_for_node_cached(&doc, ctx, node).as_deref(),
            Some("div.c"),
            "iter {i}"
        );
    }
}

#[test]
fn s1c_find_by_selector_cached_parity() {
    use crate::js_dom_bridge::{find_by_selector, find_by_selector_cached, parse_html, UniqSelCtx};
    let html = "<html><body>\
        <div id=\"dup\">d1</div><div id=\"dup\">d2</div>\
        <div class=\"card\">a</div>\
        <a target=\"unique-target\">t</a>\
        </body></html>";
    let doc = parse_html(html);
    let ctx = UniqSelCtx::new(0xbeef, 3, 1, 2, doc.node_count());
    for sel in ["#dup", ".card", "a[target=\"unique-target\"]", "#nope", "div.card"] {
        assert_eq!(find_by_selector_cached(&doc, ctx, sel), find_by_selector(&doc, sel), "sel={sel}");
        // 命中路径复取同值（含 None 缓存）。
        assert_eq!(find_by_selector_cached(&doc, ctx, sel), find_by_selector_cached(&doc, ctx, sel));
    }
}

// ── t-p126 S1d：DocUniqIndex（按视图代懒建的文档唯一性索引）──────────────────
//
// 门计数从索引读（O(1)），索引键取 `SimpleSelector::matches` 同源原始字段——对拍
// 钉定：cached（索引路径）与 uncached（引擎 DFS 路径）全元素逐值等价，覆盖索引
// 建键与门查找键的已知错位面（trim 空白 id、Unicode 空白 class token、跨 tag 同
// class、重复 name/target）。

#[test]
fn s1d_gate_parity_torture_vs_engine() {
    use crate::js_dom_bridge::{find_by_selector, parse_html, unique_selector_for_node, unique_selector_for_node_cached, UniqSelCtx};
    // 空白 padding id（门键 trim、索引键 raw）、重复 id、数字首字符 id、
    // 多 class 单例、跨 tag 同 class、NBSP class（Unicode/ASCII 空白分词差异面）、
    // 重复 name、唯一 target。
    let html = "<html><body>\
        <div id=\" spaced \">ws</div>\
        <div id=\"dup\">d1</div><div id=\"dup\">d2</div>\
        <span id=\"1a\">digit</span>\
        <div class=\"card pill\">solo</div>\
        <span class=\"hl cross\">s1</span><i class=\"hl cross\">i1</i>\
        <p class=\"a\u{00a0}b\">nbsp</p>\
        <form><input name=\"q\"></form><form><input name=\"q\"></form>\
        <a href=\"#\">plain</a><a target=\"t9\">anchor</a>\
        </body></html>";
    let doc = parse_html(html);
    let ctx = UniqSelCtx::new(0x5eed, 5, 2, 3, doc.node_count());
    // 全元素 parity：索引门与引擎门逐值等价，且物化 selector 且回定原节点。
    let all = doc.query_selector_all(doc.root(), "*");
    assert!(all.len() >= 13);
    for id in all {
        let uncached = unique_selector_for_node(&doc, id);
        let cached = unique_selector_for_node_cached(&doc, ctx, id);
        assert_eq!(cached, uncached, "parity broken at {id:?}");
        if let Some(sel) = &cached {
            assert_eq!(find_by_selector(&doc, sel), Some(id), "selector {sel} 未能回定原节点");
        }
    }
    // 空白 id：门键 trim 后 `#spaced` 在 raw id 域 0 匹配（引擎同）→ 拒 stable 形态。
    let ws_div = doc.query_selector_all(doc.root(), "div")[0];
    let ws_sel = unique_selector_for_node_cached(&doc, ctx, ws_div).expect("ws div 可寻址");
    assert_ne!(ws_sel, "#spaced", "trim 后 id 不得命中 raw id 索引");
    // NBSP class：门 token（Unicode 分词 "a"）在 class_list（ASCII 分词 ["a\u{a0}b"]）
    // 键索引 miss → 拒 stable 形态（与引擎 `p.a` 0 匹配一致）。
    let nbsp_p = doc.query_selector(doc.root(), "p").expect("p");
    let nbsp_sel = unique_selector_for_node_cached(&doc, ctx, nbsp_p).expect("nbsp p 可寻址");
    assert!(nbsp_sel.starts_with("html:nth-child(1)"), "nbsp class 应拒 stable 形态，实得 {nbsp_sel}");
    // 跨 tag 同 class：`span.hl`/`i.hl` 各自 tag 域唯一 → stable 直取。
    let span = doc.query_selector(doc.root(), "span.hl").expect("span.hl");
    assert_eq!(unique_selector_for_node_cached(&doc, ctx, span).as_deref(), Some("span.hl"));
    let itag = doc.query_selector(doc.root(), "i.hl").expect("i.hl");
    assert_eq!(unique_selector_for_node_cached(&doc, ctx, itag).as_deref(), Some("i.hl"));
    // 重复 name 锚拒绝；双 `<a>` 使 bare tag 歧义 → 唯一 target 锚直取。
    for input in doc.query_selector_all(doc.root(), "input") {
        assert_ne!(
            unique_selector_for_node_cached(&doc, ctx, input).as_deref(),
            Some("input[name=\"q\"]")
        );
    }
    let anchor = doc.query_selector(doc.root(), "a[target=\"t9\"]").expect("anchor");
    assert_eq!(
        unique_selector_for_node_cached(&doc, ctx, anchor).as_deref(),
        Some("a[target=\"t9\"]")
    );
}

#[test]
fn s1d_primitive_find_parity_and_fallback() {
    use crate::js_dom_bridge::{find_by_selector, find_by_selector_cached, parse_html, UniqSelCtx};
    let html = "<html><body>\
        <div id=\"dup\">d1</div><div id=\"dup\">d2</div>\
        <div id=\"solo\">s</div>\
        <span id=\"a.b\">dot</span>\
        <span id=\"1a\">digit</span>\
        <div class=\"card\">c1</div>\
        <p>para</p>\
        <a target=\"t9\">t</a>\
        </body></html>";
    let doc = parse_html(html);
    let ctx = UniqSelCtx::new(0xf00d, 1, 4, 5, doc.node_count());
    // 原始字符集形态（索引 O(1) 直查）与回落形态（转义/复合/组合器/伪类）全部
    // 与引擎逐值等价。
    for sel in [
        "#dup", "#solo", "#1a", "#nope", // 索引直查（含 miss）
        "div", "html", "body", "p", // 裸 tag
        "div.card", "div.pill", "span.hl", // tag.class（含 miss）
        "DIV", "#a\\.b", ".card", "div.card.x", // 大写/转义/复合 → 回落
        "a[target=\"t9\"]", "ul > li", "p:nth-child(1)", "*", // 属性/组合器/结构路径/通用
    ] {
        assert_eq!(find_by_selector_cached(&doc, ctx, sel), find_by_selector(&doc, sel), "sel={sel}");
    }
    // 重复 id：索引 first = 文档序首个（与 query_selector 先序 DFS 首个同值）。
    let dups = doc.query_selector_all(doc.root(), "#dup");
    assert_eq!(dups.len(), 2);
    assert_eq!(find_by_selector_cached(&doc, ctx, "#dup"), Some(dups[0]));
}

#[test]
fn s1d_index_rebuilds_per_token() {
    use crate::js_dom_bridge::{find_by_selector_cached, parse_html, unique_selector_for_node_cached, UniqSelCtx};
    // 代 A：唯一 `.card`，find 索引直查命中。
    let doc1 = parse_html("<html><body><div class=\"card\">a</div></body></html>");
    let node1 = doc1.query_selector(doc1.root(), "div").expect("div in doc1");
    let ctx_a = UniqSelCtx::new(9, 0, 0, 0, doc1.node_count());
    assert_eq!(find_by_selector_cached(&doc1, ctx_a, "div.card"), Some(node1));
    // 代 B：第二个 `.card` 进场——索引必须按 token 重建（陈旧 first/count 会错绑）。
    let doc2 = parse_html("<html><body><div class=\"card\">a</div><div class=\"card\">b</div></body></html>");
    let all2 = doc2.query_selector_all(doc2.root(), "div");
    assert_eq!(all2.len(), 2);
    let ctx_b = UniqSelCtx::new(9, 1, 0, 0, doc2.node_count());
    assert_eq!(find_by_selector_cached(&doc2, ctx_b, "div.card"), Some(all2[0]));
    // 门随新索引重算：stable 形态被拒 → 结构路径。
    for node in all2 {
        assert_ne!(
            unique_selector_for_node_cached(&doc2, ctx_b, node).as_deref(),
            Some("div.card"),
            "node {node:?}"
        );
    }
}

#[test]
fn s1d_index_rebuilds_on_node_count_change() {
    use crate::js_dom_bridge::{parse_html, query_match_selector_doc_cached, UniqSelCtx};
    // r5010 形状钉定：iadj 锚查询先于 mutation 入队（count=0）建索引 → 宿主直写
    // html（不 bump view_gen）+ 手工 drain（不 bump drain_gen）→ 四元组复现而内容
    // 已变。node_count 指纹必须单独触发 memo/index 失效（S1c 四元组下此形状静默
    // 踩陈旧 None/计数，S1d 每 token 全量快照放大暴露面后由 r5010 揪出）。
    let pre = parse_html("<html><body><div id='host'></div></body></html>");
    let ctx_pre = UniqSelCtx::new(0xabcd, 0, 0, 0, pre.node_count());
    assert_eq!(
        query_match_selector_doc_cached(&pre, ctx_pre, "[id=\"s30-verdict\"]"),
        "",
        "pre-apply 应 miss"
    );
    let post = parse_html(
        "<html><body><div id='host'><div id=\"s30-verdict\">S30-MULTIMATCH: PASS</div></div></body></html>",
    );
    assert_ne!(pre.node_count(), post.node_count(), "结构性插入必改节点数");
    let ctx_post = UniqSelCtx::new(0xabcd, 0, 0, 0, post.node_count());
    assert_eq!(
        query_match_selector_doc_cached(&post, ctx_post, "[id=\"s30-verdict\"]"),
        "#s30-verdict",
        "同四元组异 node_count 必须重建索引并命中引擎一致结果"
    );
}
