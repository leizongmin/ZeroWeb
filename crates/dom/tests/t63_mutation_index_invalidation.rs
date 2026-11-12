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
//!
//! # 判别力（TV F1 返修强化）
//!
//! 每例断言在「删除对应 [`zero_dom::Document`] 失效调用」反事实下必须失败：
//! - **契约级判别形**：children 写点后直断 `query_pos_index_len() == 0`。新节点
//!   首评会 miss 触发整父批量重建、覆盖过期表洗白后续查询断言；replace/
//!   set_text_content/normalize 三写点的元素位次与计数在失效缺失下查询级恒
//!   不变——内态断言是其唯一判别形；
//! - **查询级判别形**：幸存旧兄弟**先行**评估 count 敏感形态（`:nth-last-child`/
//!   `:last-child`，过期表 count=旧值 vs 新树必异），且先评被移位/幸存节点再评
//!   新节点，消除 miss 重建覆盖的洗白效应；
//! - 跨文档隔离例用双向 count 敏感断言（过期表 (3,5) 对 nth-child(3) 仍真，
//!   仅查 child_index 不判别）。
//!
//! 判别面外如实标注：`selectedcontent` `clear_children`（解析路径）无对应例；
//! 被移除/被换出节点的 detached 位置评估走 merge-base 根元素分支恒评 (1,1)、
//! shadow 直子按 host 元素子面评恒值（p125 quirk，`p125_shadow_position_pseudo.rs`
//! 冻结同源语义）、`:has` args 嵌套括号 parse 不支持恒 false（探针实证
//! base=fix 一致）——三者对失效缺失均不敏感，断言「位置改变」形非法，不钉。

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

/// 失效契约直断：children 写点完成后位置索引必须为空表（父键条目计数清零）。
/// 查询级断言可被「miss 重建覆盖」洗白，内态断言直接钉失效调用本身。
fn assert_index_invalidated(doc: &zero_dom::Document, write_point: &str) {
    assert_eq!(
        doc.query_pos_index_len(),
        0,
        "{write_point}: position index must be invalidated after children mutation"
    );
}

/// append_child 失效：新 li 的 nth-child 位置随 mutation 反映，旧 li 位置不变。
#[test]
fn t63_append_updates_nth_child_positions() {
    let mut doc = parse_html(&list_doc(2));
    let root = doc.root();

    let nth2 = doc.query_selector(root, "ul > li:nth-child(2)").unwrap();
    let new_li = doc.create_element("li");
    doc.append_child(body_id(&doc, "ul#list"), new_li).unwrap();

    // 契约级判别（反事实：删 append_child 的 invalidate 调用即在此失败）。
    assert_index_invalidated(&doc, "append_child");
    // 查询级判别：幸存旧兄弟 count 敏感形态先行评估——过期表 (2,2) 评
    // nth-last-child(1)/last-child，新树 (2,3) 评 nth-last-child(2)。
    assert!(
        doc.matches(nth2, "li:nth-last-child(2)"),
        "surviving li must be nth-last-child(2) after append (count-sensitive)"
    );
    assert!(!doc.matches(nth2, "li:last-child"));
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

    // 契约级判别。
    assert_index_invalidated(&doc, "remove_child");
    // 查询级判别（原生判别例）：second_li 过期 HIT (2,3) 对 nth-child(1) 为假。
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

    // 契约级判别。
    assert_index_invalidated(&doc, "insert_before");
    // 查询级判别：先评被移位旧节点（过期 HIT (2,2) 对 nth-child(3) 为假、对
    // nth-child(2) 为真），再评新节点——次序颠倒会被 mid 的 miss 重建洗白。
    assert!(
        doc.matches(second_li, "li:nth-child(3)"),
        "displaced li must shift to nth-child(3)"
    );
    assert!(!doc.matches(second_li, "li:nth-child(2)"));
    assert!(doc.matches(mid, "li:nth-child(2)"));
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

    // 契约级判别：replace 保元素计数、幸存兄弟位次不变，被换出节点 detached 走
    // merge-base 根元素分支恒评 (1,1)——查询级判别形不可构造（fresh 节点 miss
    // 重建还会洗白任何后置查询断言），内态断言是该写点唯一判别形。
    assert_index_invalidated(&doc, "replace_child");
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
    // 先建 ul 父键位置索引条目（count 敏感查询触发批量回填），供失效内态断言。
    assert!(doc.query_selector(root, "ul > li:nth-child(2)").is_some());

    doc.set_text_content(ul, "flat text");

    // 契约级判别：被清 li 脱树后走 merge-base 根元素分支、ul 自身兄弟位次不变
    // ——查询级判别形不可构造，内态断言是该写点判别形。
    assert_index_invalidated(&doc, "set_text_content");
    // li 全部移除：任何 li 位置伪类不再命中。
    let old_li = doc.query_selector(root, "ul > li");
    assert!(old_li.is_none(), "set_text_content must clear element children");
    // ul 自身仍是 body 第 2 个元素子（p 在前）。
    assert!(doc.matches(ul, "ul:nth-child(2)"));
}

/// normalize 失效（条件式——有移除才清）：相邻文本合并产生节点移除后，位置
/// 索引必须清空；元素位次在文本合并前后不变。
#[test]
fn t63_normalize_keeps_positions_consistent() {
    let html = "<html><body><div id=d><span>s1</span>t<span>s2</span></div></body></html>";
    let mut doc = parse_html(html);
    let root = doc.root();
    let d = body_id(&doc, "div#d");
    let s1 = doc.query_selector(root, "span:nth-child(1)").unwrap();
    let s2 = doc.query_selector(root, "span:nth-child(2)").unwrap();

    // 运行时拼装相邻文本对（解析期文本已合并，运行时插入才能触发 normalize 的
    // 移除分支）；插入自身失效后重评 s1 重建 d 父键条目，使 normalize 条件式
    // 失效的缺失可被内态断言捕获。
    let extra = doc.create_text_node("t");
    doc.insert_before(d, extra, s2).unwrap();
    assert!(doc.matches(s1, "span:nth-child(1)"));

    doc.normalize(d);

    // 契约级判别：文本合并不改变元素位次与计数——查询级判别形不可构造，
    // 内态断言是该写点（条件式分支）判别形。
    assert_index_invalidated(&doc, "normalize");
    // span 是 div 仅有的两个元素子（text 参与合并/移除，元素位次不变）。
    assert!(doc.matches(s1, "span:nth-child(1)"));
    assert!(doc.matches(s2, "span:nth-child(2)"));
}

/// selector 解析缓存语义：同文本重复查询一致；mutation 后同文本反映新树态
/// （缓存 parse 产物而非结果）；非法 selector 稳定 miss。本例写点（append_child）
/// 同时钉失效契约。
#[test]
fn t63_selector_cache_semantics_across_mutation() {
    let mut doc = parse_html(&list_doc(2));
    let root = doc.root();
    let ul = body_id(&doc, "ul#list");

    let sel = "ul > li:nth-child(2)";
    let second_li = doc.query_selector(root, sel).unwrap();
    let b = doc.query_selector(root, sel).unwrap();
    assert_eq!(second_li, b, "repeated query with identical selector text must agree");

    let fresh = doc.create_element("li");
    doc.append_child(ul, fresh).unwrap();

    // 契约级判别（本例写点=append_child）。
    assert_index_invalidated(&doc, "append_child (cache case)");
    // 查询级判别：幸存旧兄弟 count 敏感形态先行评估（过期表 (2,2) 评
    // nth-last-child(1)，新树 (2,3) 评 nth-last-child(2)）。
    assert!(
        doc.matches(second_li, "li:nth-last-child(2)"),
        "surviving li must be nth-last-child(2) after append"
    );
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
/// 恒 false（探针实证 pre/post 同值、base=fix 一致），位置相关 `:has` 形态不在
/// 此钉——判别力由幸存旧兄弟 count 敏感断言 + 失效内态断言承担。
#[test]
fn t63_has_and_nth_of_revalidate_after_mutation() {
    let mut doc = parse_html(&list_doc(2));
    let root = doc.root();
    let ul = body_id(&doc, "ul#list");
    // 先建 ul 父键位置索引条目（:has DFS 不吃位置索引，需 count 敏感查询触发）。
    let second_li = doc.query_selector(root, "ul > li:nth-child(2)").unwrap();

    assert!(doc.query_selector(root, "ul:has(em)").is_none());
    let third = doc.create_element("li");
    doc.append_child(ul, third).unwrap();

    // 契约级判别 + 查询级判别（幸存旧兄弟先行评估，过期表 (2,2) 评
    // nth-last-child(1)，新树 (2,3) 评 nth-last-child(2)）。
    assert_index_invalidated(&doc, "append_child (:has case)");
    assert!(
        doc.matches(second_li, "li:nth-last-child(2)"),
        "surviving li must be nth-last-child(2) after append"
    );

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
    let a_li3 = doc_a.query_selector(root_a, "ul > li:nth-child(3)").unwrap();
    assert!(doc_b.query_selector(root_b, "ul > li:nth-child(3)").is_some());
    assert!(doc_b.query_selector(root_b, "ul > li:nth-child(5)").is_some());
    let b_li3 = doc_b.query_selector(root_b, "ul > li:nth-child(3)").unwrap();
    assert!(doc_a.query_selector(root_a, "ul > li:nth-child(5)").is_none());
    // A 再查一次（B 查询不得污染 A 的表）。
    assert!(doc_a.query_selector(root_a, "ul > li:nth-child(3)").is_some());

    // count 敏感形态双向判别（仅查 child_index 的 nth-child 断言对串扰不敏感
    // ——过期表 (3,5) 对 nth-child(3) 仍真）：A 侧 count=3，a_li3=(3,3) 评
    // nth-last-child(1)，吃 B 表 (3,5) 则必假；B 侧 count=5，b_li3=(3,5) 评
    // nth-last-child(3)，吃 A 表 (3,3) 则必假。
    assert!(
        doc_a.matches(a_li3, "li:nth-last-child(1)"),
        "doc A count must stay 3 (isolated from B's position table)"
    );
    assert!(
        doc_b.matches(b_li3, "li:nth-last-child(3)"),
        "doc B count must stay 5 (isolated from A's position table)"
    );
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
    // 先建 host 父键位置索引条目（p 元素子面），供 attach_shadow 失效内态断言。
    let p = doc.query_selector(root, "p").unwrap();
    assert!(doc.matches(p, "p:nth-child(1)"));

    let shadow_root = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    // 契约级判别：attach_shadow 直推 children 不经 append_child（独立写点），
    // 反事实（删 shadow.rs 的 invalidate 调用）即在此失败。
    assert_index_invalidated(&doc, "attach_shadow");

    let s1 = doc.create_element("span");
    let s2 = doc.create_element("span");
    doc.append_child(shadow_root, s1).unwrap();
    doc.append_child(shadow_root, s2).unwrap();

    // host 元素子 = [p]：shadow 直接子评 {ci=1, cc=1}。
    assert!(doc.matches(s2, "span:nth-child(1)"));
    assert!(doc.matches(s2, "span:first-child"));
    let s3 = doc.create_element("span");
    doc.append_child(shadow_root, s3).unwrap();
    // 契约级判别：shadow 内 mutation 走 append_child 失效契约，此前 shadow
    // 查询重建的 shadow_root 父键条目必须再清空。
    assert_index_invalidated(&doc, "shadow append");
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
    // 契约级判别：快照=解析初态，位置索引必须复位（反事实：携带派生表即失败）。
    assert_eq!(snap.query_pos_index_len(), 0, "snapshot must reset the position index");
    let snap_root = snap.root();
    assert_eq!(
        snap.query_selector_all(snap_root, "ul > li:nth-child(2)").len(),
        1,
        "snapshot must re-derive identical query results from empty caches"
    );
    assert!(snap.query_selector(snap_root, "ul > li:nth-child(4)").is_some());
    assert!(snap.query_selector(snap_root, "ul > li:nth-child(5)").is_none());
}
