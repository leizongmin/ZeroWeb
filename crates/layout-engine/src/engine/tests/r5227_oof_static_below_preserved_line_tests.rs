//! R5227（CSS2 §9.2.1.1 × §9.5.1 × §10.3.7）OOF 静态位/float 参与 preserved 行锚点。
//! OOF 子不触发 §9.2.1.1 匿名块拆分（chromium 探针 height-002/all-prop-002 实证
//! 拆分扰动行盒宿主几何与绘制层序），容器维持单 IFC：
//! - abspos（top/bottom 全 auto）静态位 = 前驱在流内容之后（css-position-3
//!   §static-position）；前驱为行内内容时 = 容器 IFC 末行底（abspos.rs R5227
//!   行底锚 pass，chromium 探针 rel_y=20）；
//! - float 外顶与前置行盒**顶边**齐平（§9.5.1 规则 4，R5215 同模型；拆分形态
//!   由 float_positioning Phase 1 flow_snap 快照 + Phase 2 floor 排除片段盒）。
//! chromium CDP 探针定谳（c1-c5，见 goal 证据）：nbsp 行存在、abspos/float
//! rel_y=20（行高 20）、无文本全 OOF rel_y=0、in-flow 块后 abspos rel_y=10。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

fn find_abs_positions(root: &LayoutBox, node_id: zero_dom::NodeId) -> Vec<(f32, f32)> {
    let mut hits = Vec::new();
    fn walk(b: &LayoutBox, abs_x: f32, abs_y: f32, node_id: zero_dom::NodeId, hits: &mut Vec<(f32, f32)>) {
        let ax = abs_x + b.x;
        let ay = abs_y + b.y;
        if b.node_id == Some(node_id) {
            hits.push((ax, ay));
        }
        for c in &b.children {
            walk(c, ax, ay, node_id, hits);
        }
    }
    walk(root, 0.0, 0.0, node_id, &mut hits);
    hits
}

fn assert_abs_y(positions: &[(f32, f32)], expect_y: f32, label: &str) {
    assert!(!positions.is_empty(), "{label} 盒应存在于布局树");
    for (i, (_, ay)) in positions.iter().enumerate() {
        assert!(
            (ay - expect_y).abs() <= 0.5,
            "{label} 第 {i} 个盒 abs_y 应为 {expect_y}，实际 {ay}"
        );
    }
}

/// between-float-and-text 形态：nbsp 行下 abspos 与 float 同落 rel_y=20——abspos
/// 静态位在匿名行后（=20），float 行顶锚 + margin-top（0+20=20），二者重合
///（chromium 探针 c1 同值；reftest 绿盖红 PASS 条件）。旧行为：容器不拆分，
/// abspos 被包进占位 anon 块、float 被行底推低（abs_y=70）。
#[test]
fn r5227_nbsp_abspos_float_both_below_line() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  &nbsp;
  <div id=ab style="position:absolute; width:200px; height:200px;"></div>
  <div id=fl style="float:left; margin-top:20px; width:200px; height:200px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let ab = doc.get_element_by_id("ab").expect("div#ab");
    let fl = doc.get_element_by_id("fl").expect("div#fl");
    assert_abs_y(&find_abs_positions(&result.root, ab), 20.0, "nbsp 行后 abspos");
    assert_abs_y(
        &find_abs_positions(&result.root, fl),
        20.0,
        "nbsp 行后 float（行顶+mt）",
    );
}

/// nbsp + abspos（无 float）：匿名行推进 abspos 静态位（§10.3.7 前驱流内块底
/// = 20；chromium 探针 c4 同值）。
#[test]
fn r5227_nbsp_abspos_static_below_line() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  &nbsp;
  <div id=ab style="position:absolute; width:200px; height:200px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let ab = doc.get_element_by_id("ab").expect("div#ab");
    assert_abs_y(&find_abs_positions(&result.root, ab), 20.0, "nbsp 行后 abspos 静态位");
}

/// 无文本纯 OOF 子守卫：不生成匿名行（可折叠白空间塌穿不变），abspos/float
/// 均在 y=0（chromium 探针 c3 同值；R4489 纯 OOF 不占位口径保留）。
#[test]
fn r5227_oof_only_no_phantom_line() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  <div id=ab style="position:absolute; width:200px; height:200px;"></div>
  <div id=fl style="float:left; width:200px; height:200px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let ab = doc.get_element_by_id("ab").expect("div#ab");
    let fl = doc.get_element_by_id("fl").expect("div#fl");
    assert_abs_y(&find_abs_positions(&result.root, ab), 0.0, "纯 OOF abspos（无幻影行）");
    assert_abs_y(&find_abs_positions(&result.root, fl), 0.0, "纯 OOF float（无幻影行）");
}

/// in-flow 块 + abspos 守卫：静态位取前驱 in-flow 块底（R4017 既有口径，
/// chromium 探针 c5 同值）——R5227 拆分扩展不得扰动。
#[test]
fn r5227_inflow_block_abspos_static_after_block() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  <div style="height:10px;"></div>
  <div id=ab style="position:absolute; width:200px; height:200px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let ab = doc.get_element_by_id("ab").expect("div#ab");
    assert_abs_y(
        &find_abs_positions(&result.root, ab),
        10.0,
        "in-flow 块后 abspos 静态位",
    );
}

/// nbsp + float（无 abspos）隔离锚：float 行顶锚不被匿名行底推低——y =
/// 行顶 + margin-top = 20（§9.5.1 规则 4；旧行为行底+mt = 40）。
#[test]
fn r5227_float_after_nbsp_line_top_anchor() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  &nbsp;
  <div id=fl style="float:left; margin-top:20px; width:200px; height:200px;"></div>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let fl = doc.get_element_by_id("fl").expect("div#fl");
    assert_abs_y(&find_abs_positions(&result.root, fl), 20.0, "nbsp 行后 float 行顶锚");
}

/// all-prop-002 形态守卫：abspos 是**首个**子（其后才有 inline 内容）——静态位
/// 维持容器内容顶（前驱无在流内容，行盒在 abspos 之后不后溯；chromium 探针
/// greenB rel_y=0 同值）。行底锚只对 DOM 序在前的在流内容生效。
#[test]
fn r5227_abspos_before_inline_content_stays_at_top() {
    let html = r#"<html><body style="margin:0">
<div style="line-height:20px;">
  <div id=ab style="position:absolute; width:100px; height:100px;"></div>
  <span>after</span>
</div>
</body></html>"#;
    let (doc, result) = layout(html);
    let ab = doc.get_element_by_id("ab").expect("div#ab");
    assert_abs_y(
        &find_abs_positions(&result.root, ab),
        0.0,
        "行内内容前的 abspos（容器顶）",
    );
}
