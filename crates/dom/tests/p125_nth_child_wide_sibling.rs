//! p125 回归：`:nth-child` 宽兄弟选择器查询复杂度（github.com/trending renderer 楔死）。
//!
//! 根因：`element_matches_selector` 对带伪类选择器在链选择器 DFS 的每个访问节点急切
//! 计算 `compute_element_position`（全兄弟两次遍历 + `to_vec` 分配）；N 个同父 `<a>`
//! 时单次 miss 查询 O(全树节点 × N) 退化——846 锚点形状实测 64ms/查询恒定（无伪类
//! 同形状 509µs，125×），SetDomSnapshot 每 publish 对 846 锚点逐个寻址 = 20.3s 楔死。
//!
//! 修复：①廉价成分先行（`selector.matches(elem)` 失败即返）；②每查询位置索引（RAII
//! guard 装载，`compute_element_position` miss 时 per-parent 批量回填）。本测试钉两层：
//! 功能 oracle（nth-child 结果 == 文档序直接算术——CSS Selectors L4 §nth-child-pseudo、
//! DOM §match-a-selectors-chain 行为零变化）+ 渐进上界（5000 兄弟 miss 查询 < 1s；
//! 修前该形状秒级，修后毫秒级）。

use std::time::{Duration, Instant};
use zero_dom::parse_html;

/// dm-2 fixture 语义（github select-menu 语言菜单）：同父锚点各带一个 span 子，
/// 846 形状放大到 5000。
fn wide_sibling_doc(n: usize) -> String {
    let mut html = String::from("<html><body><div id=\"menu\">");
    for i in 0..n {
        html.push_str(&format!("<a href=\"/lang/{i}\" data-i=\"{i}\"><span>s{i}</span></a>"));
    }
    html.push_str("</div></body></html>");
    html
}

/// 功能 oracle：nth-child 链查询结果必须与文档序直接算术逐项一致。
#[test]
fn p125_nth_child_wide_sibling_matches_document_order() {
    let doc = parse_html(&wide_sibling_doc(5000));
    let root = doc.root();

    let all = doc.query_selector_all(root, "#menu > a");
    assert_eq!(all.len(), 5000, "baseline qsa must find all 5000 anchors");

    // nth-child(k) 命中 == 文档序第 k 个锚点。
    for &k in &[1usize, 2, 4999, 5000] {
        let hit = doc
            .query_selector(root, &format!("#menu > a:nth-child({k})"))
            .unwrap_or_else(|| panic!("nth-child({k}) must hit"));
        assert_eq!(
            hit,
            all[k - 1],
            "nth-child({k}) must be the {k}-th anchor in document order"
        );
    }
    // 越界 miss。
    assert!(doc.query_selector(root, "#menu > a:nth-child(5001)").is_none());
    // an+b 全量计数：1..=5000 内 3 的倍数 = 1666。
    let nth_3n = doc.query_selector_all(root, "#menu > a:nth-child(3n)");
    assert_eq!(nth_3n.len(), 1666, "nth-child(3n) over 5000 anchors");
    // 单点 matches（查询索引未装载）行为不变。
    assert!(doc.matches(all[42], "a:nth-child(43)"));
    assert!(!doc.matches(all[42], "a:nth-child(44)"));
}

/// 渐进上界：宽兄弟 nth-child 查询不得随兄弟数二次退化（p125 楔死形状）。
#[test]
fn p125_nth_child_wide_sibling_query_complexity_bound() {
    let doc = parse_html(&wide_sibling_doc(5000));
    let root = doc.root();

    // 全 DFS miss（越界 nth-child 不提前退出）——修前 O(节点×兄弟) 秒级退化形状。
    let t = Instant::now();
    assert!(doc.query_selector(root, "#menu > a:nth-child(9999)").is_none());
    let miss_elapsed = t.elapsed();

    // 全量 an+b qsa（候选全判定，索引一次批量回填）。
    let t = Instant::now();
    assert_eq!(doc.query_selector_all(root, "#menu > a:nth-child(3n)").len(), 1666);
    let qsa_elapsed = t.elapsed();

    // 修前 846 兄弟实测 64ms/查询恒定、5000 兄弟秒级；修后摊销 O(1)/节点（毫秒
    // 级）。1s 上界留 ~100× 裕度防慢机噪声误报。
    println!("p125 bound: miss {miss_elapsed:?}, qsa(3n) {qsa_elapsed:?}");
    assert!(
        miss_elapsed < Duration::from_secs(1),
        "wide-sibling nth-child miss query took {miss_elapsed:?} — p125 pathological complexity"
    );
    assert!(
        qsa_elapsed < Duration::from_secs(1),
        "wide-sibling nth-child(3n) qsa took {qsa_elapsed:?} — p125 pathological complexity"
    );
}
