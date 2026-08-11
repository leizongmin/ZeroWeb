//! R939 Batch 1 / R940 细化：R109 §9.2.1.1 匿名块盒高度回填（spec FR-001）单测。
//!
//! 验证 `backfill_r109_anon_block_heights`：
//! ① 匿名块盒（fragment_node_ids.is_some）从 inline_layout 行盒回填自身 content_height
//!    （仅增大，不收缩）；
//! ② auto-height 容器含匿名块子 → 重算 content_height = max in-flow 非 float 子盒 border-box
//!    底（仅增大）——覆盖「anon 自身欠计」与「容器未把已正确的 anon 计入」两种（R940 max-bottom）；
//! ③ 非 auto-height 容器（显式 height）不扩展；
//! ④ 非 anon 块的 content_height 不被 inline_layout 改写；不收缩。
//!
//! NodeId 是 slotmap opaque key，须经 Document 取有效值；backfill 本身不消费 Document，
//! 仅用 NodeId 作 styles HashMap 的键。

use super::*;
use crate::types::{InlineLayoutFragment, InlineLayoutLine};
use std::collections::HashMap;
use zero_css_parser::values::LengthValue;
use zero_dom::{Document, NodeId};
use zero_style_system::ComputedStyle;

/// 取一个有效 NodeId（经 Document）。
fn fresh_id() -> NodeId {
    Document::new().create_element("div")
}

/// 构造一行内布局行盒（y, height）。
fn make_line(y: f32, height: f32) -> InlineLayoutLine {
    InlineLayoutLine {
        y,
        height,
        fragments: vec![InlineLayoutFragment {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            height,
            font_size: 20.0,
            is_ahem: false,
            is_ahem_font: false,
            text: String::new(),
            source: None,
            node_id: None,
            baseline_y: height,
        }],
        baseline_y: height,
        ascent: height * 0.8,
        descent: height * 0.2,
    }
}

#[test]
fn test_backfill_anon_block_own_height_from_inline_layout() {
    // ① 匿名块：taffy 经 ctx_node 欠计 content_height=20，inline_layout 真实 = 2 行 × 20 = 40。
    let mut anon = LayoutBox::default();
    anon.fragment_node_ids = Some(vec![]);
    anon.content_height = 20.0;
    anon.height = 20.0;
    anon.is_block_level = true;
    anon.inline_layout = Some(vec![make_line(0.0, 20.0), make_line(20.0, 20.0)]);

    let styles = HashMap::new();
    let mut wrapper = LayoutBox::default();
    wrapper.children.push(anon);
    let _ = backfill_r109_anon_block_heights(&mut wrapper, &styles);

    assert!(
        (wrapper.children[0].content_height - 40.0).abs() < 0.01,
        "anon content_height backfilled to 40"
    );
    assert!(
        (wrapper.children[0].height - 40.0).abs() < 0.01,
        "anon height backfilled to 40"
    );
}

#[test]
fn test_backfill_container_recompute_max_bottom() {
    // ② auto-height 容器：taffy 漏算 anon 子（content_height=20），但 anon 自身已正确 h=40
    //    位于 y=40（inserted block 之后）。max-bottom = 40+40 = 80 > 20 → 容器长到 80。
    let mut container = LayoutBox::default();
    let cid = fresh_id();
    container.node_id = Some(cid);
    container.content_height = 20.0;
    container.height = 20.0;
    container.is_block_level = true;

    let mut anon = LayoutBox::default();
    anon.fragment_node_ids = Some(vec![]);
    anon.y = 40.0;
    anon.content_height = 40.0; // 已正确（非欠计）
    anon.height = 40.0;
    anon.is_block_level = true;
    container.children.push(anon);

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.height = LengthValue::Auto;
    styles.insert(cid, s);

    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    assert!(
        (container.content_height - 80.0).abs() < 0.01,
        "container content_height recomputed to max-bottom 80"
    );
    assert!(
        (container.height - 80.0).abs() < 0.01,
        "container height recomputed to 80"
    );
}

#[test]
fn test_backfill_skips_explicit_height_container() {
    // ③ 容器显式 height:100px → 即使含 anon 子也不重算（auto_h=false）。
    let mut container = LayoutBox::default();
    let cid = fresh_id();
    container.node_id = Some(cid);
    container.content_height = 100.0;
    container.height = 100.0;
    container.is_block_level = true;

    let mut anon = LayoutBox::default();
    anon.fragment_node_ids = Some(vec![]);
    anon.y = 40.0;
    anon.content_height = 40.0;
    anon.height = 40.0;
    anon.is_block_level = true;
    container.children.push(anon);

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.height = LengthValue::Px(100.0);
    styles.insert(cid, s);

    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    assert!(
        (container.content_height - 100.0).abs() < 0.01,
        "explicit container unchanged"
    );
    assert!(
        (container.height - 100.0).abs() < 0.01,
        "explicit container height unchanged"
    );
}

#[test]
fn test_backfill_does_not_shrink() {
    // ④ 匿名块 content_height 已 ≥ inline_layout → 不收缩；容器 max-bottom ≤ 当前 → 不收缩。
    let mut anon = LayoutBox::default();
    anon.fragment_node_ids = Some(vec![]);
    anon.content_height = 60.0;
    anon.height = 60.0;
    anon.is_block_level = true;
    anon.inline_layout = Some(vec![make_line(0.0, 20.0)]); // inline 仅 20 < 60

    let styles = HashMap::new();
    let mut container = LayoutBox::default();
    container.children.push(anon);
    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    assert!(
        (container.children[0].content_height - 60.0).abs() < 0.01,
        "anon not shrunk"
    );
    assert!(
        (container.children[0].height - 60.0).abs() < 0.01,
        "anon height not shrunk"
    );
}

#[test]
fn test_backfill_ignores_non_anon_block() {
    // 非 anon 块（fragment_node_ids=None）即使有 inline_layout 也不被 Part 1 改写。
    let mut block = LayoutBox::default();
    block.fragment_node_ids = None;
    block.content_height = 10.0;
    block.height = 10.0;
    block.is_block_level = true;
    block.inline_layout = Some(vec![make_line(0.0, 40.0)]);

    let styles = HashMap::new();
    let mut container = LayoutBox::default();
    container.children.push(block);
    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    assert!(
        (container.children[0].content_height - 10.0).abs() < 0.01,
        "non-anon not touched by Part 1"
    );
}

#[test]
fn test_backfill_shifts_subsequent_sibling_on_growth() {
    // R941 兄弟位移：child[0]（anon）增高 delta → 后续 in-flow 非 float 非 abspos 兄弟
    // child[1].y 下移 delta（post-taffy 改高度不会自动重定位兄弟，否则重叠）。
    let mut container = LayoutBox::default();
    let cid = fresh_id();
    container.node_id = Some(cid);
    container.content_height = 30.0;
    container.height = 30.0;
    container.is_block_level = true;

    // child[0]：anon，taffy 欠计 content_height=20，inline_layout 真实=40 → Part 1 增高 20。
    let mut anon = LayoutBox::default();
    anon.fragment_node_ids = Some(vec![]);
    anon.y = 0.0;
    anon.content_height = 20.0;
    anon.height = 20.0;
    anon.is_block_level = true;
    anon.inline_layout = Some(vec![make_line(0.0, 20.0), make_line(20.0, 20.0)]);
    container.children.push(anon);

    // child[1]：常规块兄弟，位于 y=20（child[0] 原高之后）。
    let mut sib = LayoutBox::default();
    sib.y = 20.0;
    sib.content_height = 10.0;
    sib.height = 10.0;
    sib.is_block_level = true;
    container.children.push(sib);

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.height = LengthValue::Auto;
    styles.insert(cid, s);

    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    // child[0] 增高 20 → cumulative_shift=20 → child[1].y 从 20 移到 40。
    assert!(
        (container.children[1].y - 40.0).abs() < 0.01,
        "subsequent sibling shifted down by prior growth (20→40)"
    );
}

#[test]
fn test_backfill_does_not_shift_abspos_sibling() {
    // abspos 兄弟不应被位移（非 in-flow，独立定位）。
    let mut container = LayoutBox::default();
    container.is_block_level = true;

    let mut anon = LayoutBox::default();
    anon.fragment_node_ids = Some(vec![]);
    anon.content_height = 20.0;
    anon.height = 20.0;
    anon.is_block_level = true;
    anon.inline_layout = Some(vec![make_line(0.0, 20.0), make_line(20.0, 20.0)]);
    container.children.push(anon);

    let mut abspos = LayoutBox::default();
    abspos.y = 20.0;
    abspos.is_absolute = true;
    abspos.is_block_level = true;
    container.children.push(abspos);

    let styles = HashMap::new();
    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    assert!(
        (container.children[1].y - 20.0).abs() < 0.01,
        "abspos sibling not shifted"
    );
}

#[test]
fn test_backfill_container_grows_for_r109_split_child() {
    // ② R1164：auto-height 容器，直系子是 R109 拆分 inline 父盒（is_r109_split=true，
    //    fragment_node_ids=None）而非匿名块。taffy 欠计 split 子盒高度（content_height=20
    //    应 60），但 split 子盒自身高度已正确（60，无 ① 增长 → descendant_growth=0），
    //    且无匿名块直接子（has_anon_child=false）——旧 gate（has_anon_child||descendant_growth）
    //    不触发致容器残留矮 + bg 露白（block-in-inline-relpos-001，2.56→0.97% 实证）。
    //    新 gate has_r109_split_child 触发 max-bottom 重算 → 容器长到 60。welcome 无 R109
    //    split 故零回归（区别 R1163 broad「全容器」gate 致 welcome +12.57pp 回归）。
    let mut container = LayoutBox::default();
    let cid = fresh_id();
    container.node_id = Some(cid);
    container.content_height = 20.0;
    container.height = 20.0;
    container.is_block_level = true;

    let mut split_inline = LayoutBox::default();
    split_inline.is_r109_split = true; // R109 拆分 inline 父盒（非匿名块）
    split_inline.fragment_node_ids = None; // 父盒非片段
    split_inline.y = 0.0;
    split_inline.content_height = 60.0; // 已正确（自身不增长 → descendant_growth=0）
    split_inline.height = 60.0;
    split_inline.is_block_level = true;
    container.children.push(split_inline);

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.height = LengthValue::Auto;
    styles.insert(cid, s);

    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    assert!(
        (container.content_height - 60.0).abs() < 0.01,
        "container grows to fit R109 split child: got {}",
        container.content_height
    );
    assert!(
        (container.height - 60.0).abs() < 0.01,
        "container height grows: got {}",
        container.height
    );
}

#[test]
fn test_backfill_no_growth_for_normal_container_without_r109() {
    // ② 守卫：普通 auto-height 容器（无匿名块子 / 无 R109 split 子 / 无后代增长）不触发
    //    max-bottom 重算 —— 即便某 in-flow 子盒 y+height 恰 > 容器 content_height（如负 margin
    //    /margin-collapse-through 的合法情形），也不强扩。这是 narrow gate 的安全性核心：
    //    避免 R1163 broad gate 在 welcome 上 +12.57pp 回归。
    let mut container = LayoutBox::default();
    let cid = fresh_id();
    container.node_id = Some(cid);
    container.content_height = 20.0;
    container.height = 20.0;
    container.is_block_level = true;

    let mut normal_child = LayoutBox::default();
    normal_child.y = 0.0;
    normal_child.content_height = 60.0; // y+height=60 > 容器 20，但无 R109 标记
    normal_child.height = 60.0;
    normal_child.is_block_level = true;
    container.children.push(normal_child);

    let mut styles = HashMap::new();
    let mut s = ComputedStyle::default();
    s.height = LengthValue::Auto;
    styles.insert(cid, s);

    let _ = backfill_r109_anon_block_heights(&mut container, &styles);

    assert!(
        (container.content_height - 20.0).abs() < 0.01,
        "normal container without R109 split child is NOT grown: got {}",
        container.content_height
    );
}
