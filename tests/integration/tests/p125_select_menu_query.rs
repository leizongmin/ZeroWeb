//! t-p125 回归：github.com/trending select-menu 锚点页的 DOM 序列化/查询成本护栏。
//!
//! 真站形状（2026-10-30 探针链，形状与探针过程见 crates/engine/tests/p125_select_menu_anchors.rs
//! 头注；本文件 fixture `p125-select-menu-dm-2.html` = 真站语言菜单片段最小化）：
//! 根因为 `element_matches_selector` 对带伪类选择器在链选择器 DFS 的每个访问节点
//! 急切计算 `compute_element_position`（两次全兄弟遍历 + Vec 分配），846 同父锚点
//! 单次 miss 查询 64ms 恒定；SetDomSnapshot 每 publish 对 846 锚点逐个寻址 →
//! 20.3s renderer 楔死（gdb 栈 + WorkerCensus + 本电池修前 125× 对照三证）。修复：
//! 廉价成分先行 + 每查询位置索引（crates/dom/src/document/mod.rs）。
//!
//! 本文件钉两层（真 fixture）：
//! - snapshot parse 回环：cached_html（= `doc.outer_html` 序列化产物，webview
//!   sync_render_after_native_dom 同款）re-parse 回环稳定；
//! - 选择器成本电池：各形状单查询成本上界（修前 `:nth-child` 形状 64ms 恒定，
//!   修后 ~1ms debug build；合成 5000 兄弟渐进断言在
//!   crates/dom/tests/p125_nth_child_wide_sibling.rs）。

use std::time::Instant;

const FIXTURE_DM2: &str = include_str!("../../../crates/engine/tests/fixtures/p125-select-menu-dm-2.html");

/// 序列化 → re-parse 回环：元素数不变、成本有界（p125 修前回环曾被列为嫌疑，
/// 本测试钉住快照路径非病态）。
#[test]
#[serial_test::serial]
fn dm2_snapshot_parse_roundtrip() {
    let t0 = Instant::now();
    let doc = zero_dom::parse_html(FIXTURE_DM2);
    println!("dm-2: first parse {}ms", t0.elapsed().as_millis());
    let anchors = doc.query_selector_all(doc.root(), "details-menu a");
    assert_eq!(anchors.len(), 846, "dm-2 fixture 须含 846 个 menu 锚点");

    let t1 = Instant::now();
    let out = doc.outer_html(doc.root());
    println!("dm-2: serialize {}ms len={}", t1.elapsed().as_millis(), out.len());
    let t2 = Instant::now();
    let doc2 = zero_dom::parse_html(&out);
    println!("dm-2: SECOND parse {}ms", t2.elapsed().as_millis());
    let anchors2 = doc2.query_selector_all(doc2.root(), "details-menu a");
    assert_eq!(anchors2.len(), 846, "回环后锚点数须不变");
    let out2 = doc2.outer_html(doc2.root());
    println!(
        "dm-2: re-serialize len={} (roundtrip-stable={})",
        out2.len(),
        out2.len() == out.len()
    );
}

/// 真 fixture 选择器成本电池（p125 楔死形状）。
///
/// 核心对照：同形状带/不带 `:nth-child` 伪类——修前带伪类形状 64ms/查询恒定
///（与 N 无关 = 全 DFS miss，O(节点×兄弟)），无伪类同形状 509µs（125×）。500ms
/// 断言上界为 hang-guard（修前可通过——64ms < 500ms；渐进回归由 dom crate 的
/// 5000 兄弟测试以 1s 上界钉住）。
#[test]
#[serial_test::serial]
fn dm2_selector_cost_battery() {
    let doc = zero_dom::parse_html(FIXTURE_DM2);
    let root = doc.root();
    let cases = [
        "#select-menu-language",
        "#languages-menuitems",
        "#languages-menuitems > a",
        "#languages-menuitems > a:nth-child(3)",
        "#languages-menuitems > a:nth-child(400)",
        "#languages-menuitems > a:nth-child(846)",
        "a:nth-child(3)",
        "details-menu > a:nth-child(3)",
        "details > details-menu > div > div > a:nth-child(400)",
        "html > body > details:nth-child(1)",
    ];
    for sel in cases {
        let t = Instant::now();
        let hit = doc.query_selector(root, sel).map(|_| "hit").unwrap_or("miss");
        let us = t.elapsed().as_micros();
        println!("qs {sel:<60} -> {hit} in {us}µs");
        assert!(us < 500_000, "query_selector '{sel}' took {us}µs — pathological");
    }
    let t = Instant::now();
    let n = doc.query_selector_all(root, "#languages-menuitems > a").len();
    println!("qsa #languages-menuitems > a -> {n} in {}µs", t.elapsed().as_micros());
}
