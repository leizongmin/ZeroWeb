//! T63 churn 治理回归钉：位置索引字段化驻留（thread_local 每查询装/卸 → Document
//! 常驻字段）+ 结构 mutation 全清失效 + selector 解析缓存（Arc 共享）。
//!
//! 位置索引跨查询驻留后，**失效正确性是安全措施**（宁过勿漏）：漏清则伪类位置
//! 评估（`:nth-child`/`:first-child`/`:has`/`:nth-child(of)`）读到 mutation 前的
//! sibling 序。本文件以 mutation 前后查询语义零变化为 oracle，钉每类 children
//! 结构写点的失效契约：
//! - `append_child`/`remove_child`/`insert_before`/`replace_child`/`set_text_content`
//!   （元素类分支）/`normalize`（有移除时）/`attach_shadow` + shadow 内 mutation；
//! - selector 解析缓存无失效需求（键=文本，parse 纯函数）——同文本查询在
//!   mutation 后必须反映新树态（缓存的是 parse 产物，不是查询结果）；
//! - 跨 Document 实例隔离（slotmap NodeId 世代键跨文档可数值碰撞——字段随
//!   Document 生命周期，禁全局/TLS 驻留）；
//! - `pristine_snapshot` 缓存面复位（快照=解析初态，携带派生表会被写穿语义反噬）。

use zero_dom::{ShadowRootMode, parse_html};

/// `<ul id="list">` 下 N 个 `<li>` 文本 a/b/c…，body 前置一个 sibling `<p>` 干扰项。
fn list_doc(items: usize) -> String {
    let mut html = String::from("<html><body><p>lead</p><ul id=\"list\">");
    for i in 0..items {
        html.push_str(&format!("<li>c{i}</li>"));
    }
    html.push_str("</ul></body></html>");
    html
}

fn body_id(doc: &zero_dom::Document, selector: &str) -> zero_dom::NodeId {
    doc.query_selector(doc.root(), selector)
        .unwrap_or_else(|| panic!("{selector} must resolve in fixture"))
}

/// append_child 失效：新 li 的 nth-child 位置随 mutation 反映，旧 li 位置不变。
#[test]
fn t63_append_updates_nth_child_positions() {
    let mut doc = parse_html(&list_doc(2));
    let root = doc.root();

    let nth2 = doc.query_selector(root, "ul > li:nth-child(2)").unwrap();
    let new_li = doc.create_element("li");
    doc.append_child(body_id(&doc, "ul#list"), new_li).unwrap();

    // 失效后重评：新 li 是第 3 个元素子（p 与 ul 是 body 的元素子，li 们是 ul 的）。
    assert!(
        doc.matches(new_li, "li:nth-child(3)"),
        "appended li must evaluate as nth-child(3) after index invalidation"
    );
    assert!(!doc.matches(new_li, "li:nth-child(2)"));
    // 旧 li 位置不变。
    assert!(doc.matches(nth2, "li:nth-child(2)"));
    // qSA 全量重评一致。
    assert_eq!(doc.query_selector_all(root, "ul > li:nth-child(3)").len(), 1);
    assert_eq!(doc.query_selector_all(root, "ul > li:nth-child(2)").len(), 1);
}

/// remove_child 失效：被删元素出列，后续元素序号前移。
#[test]
fn t63_remove_shifts_nth_child_positions() {
    let mut doc = parse_html(&list_doc(3));
    let root = doc.root();
    let ul = body_id(&doc, "ul#list");
    let first_li = doc.query_selector(root, "ul > li:nth-child(1)").unwrap();
    let second_li = doc.query_selector(root, "ul > li:nth-child(2)").unwrap();

    doc.remove_child(ul, first_li).unwrap();

    assert!(
        doc.matches(second_li, "li:nth-child(1)"),
        "second li must become nth-child(1) after removal"
    );
    assert!(!doc.matches(second_li, "li:nth-child(2)"));
    assert_eq!(doc.query_selector_all(root, "ul > li:nth-child(1)").len(), 1);
    // 被删节点脱离树后的位置评估走 merge-base 根元素分支（评成 nth-child(1)），
    // 非本切片语义（探针实证 base=fix 一致），不在此钉。
}

/// insert_before 失效：中间插入使后半序号右移。
#[test]
fn t63_insert_before_shifts_positions() {
    let mut doc = parse_html(&list_doc(2));
    let root = doc.root();
    let ul = body_id(&doc, "ul#list");
    let second_li = doc.query_selector(root, "ul > li:nth-child(2)").unwrap();

    let mid = doc.create_element("li");
    doc.insert_before(ul, mid, second_li).unwrap();

    assert!(doc.matches(mid, "li:nth-child(2)"));
    assert!(
        doc.matches(second_li, "li:nth-child(3)"),
        "displaced li must shift to nth-child(3)"
    );
    assert_eq!(doc.query_selector_all(root, "ul > li:nth-child(3)").len(), 1);
}

/// replace_child 失效：新节点顶替旧位的 nth-child 位置。
#[test]
fn t63_replace_child_takes_over_position() {
    let mut doc = parse_html(&list_doc(2));
    let root = doc.root();
    let ul = body_id(&doc, "ul#list");
    let first_li = doc.query_selector(root, "ul > li:nth-child(1)").unwrap();

    let fresh = doc.create_element("li");
    doc.replace_child(ul, fresh, first_li).unwrap();

    assert!(doc.matches(fresh, "li:nth-child(1)"));
    // 被换出节点评 nth-child(1) 同为 merge-base 根元素分支语义，不在此钉（同上）。
}

/// set_text_content（元素类分支）失效：子树整体替换后位置评估反映新结构。
#[test]
fn t63_set_text_content_resets_children_positions() {
    let mut doc = parse_html(&list_doc(2));
    let root = doc.root();
    let ul = body_id(&doc, "ul#list");
    assert_eq!(doc.query_selector_all(root, "ul > li").len(), 2);

    doc.set_text_content(ul, "flat text");

    // li 全部移除：任何 li 位置伪类不再命中。
    let old_li = doc.query_selector(root, "ul > li");
    assert!(old_li.is_none(), "set_text_content must clear element children");
    // ul 自身仍是 body 第 2 个元素子（p 在前）。
    assert!(doc.matches(ul, "ul:nth-child(2)"));
}

/// normalize 失效：相邻文本合并后同文档查询仍逐项一致。
#[test]
fn t63_normalize_keeps_positions_consistent() {
    let html = "<html><body><div id=d><span>s1</span>t<span>s2</span></div></body></html>";
    let mut doc = parse_html(html);
    let root = doc.root();
    let d = body_id(&doc, "div#d");
    let s1 = doc.query_selector(root, "span:nth-child(1)").unwrap();
    let s2 = doc.query_selector(root, "span:nth-child(2)").unwrap();

    doc.normalize(d);

    // span 是 div 仅有的两个元素子（text 参与合并/移除，元素位次不变）。
    assert!(doc.matches(s1, "span:nth-child(1)"));
    assert!(doc.matches(s2, "span:nth-child(2)"));
}

/// selector 解析缓存语义：同文本重复查询一致；mutation 后同文本反映新树态
/// （缓存 parse 产物而非结果）；非法 selector 稳定 miss。
#[test]
fn t63_selector_cache_semantics_across_mutation() {
    let mut doc = parse_html(&list_doc(2));
    let root = doc.root();
    let ul = body_id(&doc, "ul#list");

    let sel = "ul > li:nth-child(2)";
    let a = doc.query_selector(root, sel).unwrap();
    let b = doc.query_selector(root, sel).unwrap();
    assert_eq!(a, b, "repeated query with identical selector text must agree");

    let fresh = doc.create_element("li");
    doc.append_child(ul, fresh).unwrap();
    // 同文本（缓存命中路径）必须反映 mutation 后的树态。
    assert!(doc.matches(fresh, "li:nth-child(3)"));
    assert_eq!(
        doc.query_selector_all(root, sel).len(),
        1,
        "cached parse must not freeze results"
    );

    // 非法 selector：解析失败稳定 miss（不缓存、不 panic）。
    assert!(doc.query_selector(root, "###").is_none());
    assert!(doc.query_selector(root, "###").is_none());
    assert!(!doc.matches(root, "###"));
}

/// `:nth-child(of)` 延后复评吃失效后的索引；`:has` 后代路径随树变化。
/// 基线限制：`:has` args 内嵌套括号（如 `:has(> li:nth-child(3))`）parse 不支持
/// 恒 false（探针实证 pre/post 同值、base=fix 一致），位置相关 `:has` 形态不在此钉。
#[test]
fn t63_has_and_nth_of_revalidate_after_mutation() {
    let mut doc = parse_html(&list_doc(2));
    let root = doc.root();
    let ul = body_id(&doc, "ul#list");

    assert!(doc.query_selector(root, "ul:has(em)").is_none());
    let third = doc.create_element("li");
    doc.append_child(ul, third).unwrap();
    let em = doc.create_element("em");
    doc.append_child(third, em).unwrap();
    // :has 后代路径（qSA 求值）必须看到 mutation 后子树。
    assert!(
        doc.query_selector(root, "ul:has(em)").is_some(),
        ":has descendant path must see post-mutation subtree"
    );
    // nth-child(of) 延后复评：新 li 在仅计 li 兄弟中位置=3。
    assert!(
        doc.matches(third, "li:nth-child(3 of li)"),
        "nth-child(of) must revalidate after invalidation"
    );
    assert!(!doc.matches(third, "li:nth-child(2 of li)"));
}

/// 跨 Document 实例隔离：两文档同形状各自查询正确（字段随实例，无 TLS/全局
/// 驻留——slotmap NodeId 世代键跨文档可数值碰撞）。
#[test]
fn t63_index_isolated_per_document() {
    let doc_a = parse_html(&list_doc(3));
    let doc_b = parse_html(&list_doc(5));
    let root_a = doc_a.root();
    let root_b = doc_b.root();

    // 先在 A 建索引，再查 B（若索引跨文档串扰，B 的第 3/5 位置判定会假值）。
    assert!(doc_a.query_selector(root_a, "ul > li:nth-child(3)").is_some());
    assert!(doc_b.query_selector(root_b, "ul > li:nth-child(3)").is_some());
    assert!(doc_b.query_selector(root_b, "ul > li:nth-child(5)").is_some());
    assert!(doc_a.query_selector(root_a, "ul > li:nth-child(5)").is_none());
    // A 再查一次（B 查询不得污染 A 的表）。
    assert!(doc_a.query_selector(root_a, "ul > li:nth-child(3)").is_some());
}

/// shadow：attachShadow 失效 + shadow 树内 mutation 失效（shadow 查询走同一
/// 常驻索引）。shadow 直接子位置按 merge-base 冻结语义评 host 元素子面
/// （parent_element_node 越过 ShadowRoot 取 host——p125_shadow_position_pseudo
/// 钉的怪癖），与 shadow 兄弟序无关，故 fixture 的 host 需带 light 元素子。
#[test]
fn t63_shadow_queries_track_mutation() {
    let mut doc = parse_html("<html><body><div id=host><p>light</p></div></body></html>");
    let root = doc.root();
    let host = body_id(&doc, "div#host");

    let shadow_root = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let s1 = doc.create_element("span");
    let s2 = doc.create_element("span");
    doc.append_child(shadow_root, s1).unwrap();
    doc.append_child(shadow_root, s2).unwrap();

    // host 元素子 = [p]：shadow 直接子评 {ci=1, cc=1}。
    assert!(doc.matches(s2, "span:nth-child(1)"));
    assert!(doc.matches(s2, "span:first-child"));
    let s3 = doc.create_element("span");
    doc.append_child(shadow_root, s3).unwrap();
    // shadow 内 append 后失效重评同值（host 元素子面未变）。
    assert!(
        doc.matches(s2, "span:nth-child(1)"),
        "position must stay after sibling append"
    );
    assert!(
        doc.matches(s3, "span:nth-child(1)"),
        "new shadow child must evaluate post-invalidation"
    );

    // shadow 查询 API（走 parse 缓存 + 常驻索引）与 matches 同值。
    assert_eq!(doc.query_selector_all_shadow(shadow_root, "span:nth-child(1)").len(), 3);
    assert_eq!(doc.query_selector_shadow(shadow_root, "span:nth-child(1)"), Some(s1));
    // light 子位置不受 shadow attach / shadow 内 mutation 影响。
    let p = doc.query_selector(root, "p").unwrap();
    assert!(doc.matches(p, "p:nth-child(1)"));
}

/// pristine_snapshot 缓存面复位：快照文档首次查询（缓存空起算）与原文档一致。
#[test]
fn t63_snapshot_restarts_caches() {
    let doc = parse_html(&list_doc(4));
    let root = doc.root();
    // 原文档先建满缓存面。
    let hits = doc.query_selector_all(root, "ul > li:nth-child(2)");
    assert_eq!(hits.len(), 1);

    let snap = doc.pristine_snapshot();
    let snap_root = snap.root();
    assert_eq!(
        snap.query_selector_all(snap_root, "ul > li:nth-child(2)").len(),
        1,
        "snapshot must re-derive identical query results from empty caches"
    );
    assert!(snap.query_selector(snap_root, "ul > li:nth-child(4)").is_some());
    assert!(snap.query_selector(snap_root, "ul > li:nth-child(5)").is_none());
}
