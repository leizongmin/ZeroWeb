//! R5448：walk 不稳帧（前序兄弟含 inline 内容 + float，块高 remeasure 步 6 才物化
//! h=0）内跳过跨兄弟 float 带集/底边继承——幻影带/幻影 floor 根治钉。
//!
//! 驱动案同 c5525-fltmrgn-000 test3 形态最小化：t2（文本 + float）walk h=0 → t3 帧
//! y stale，t2 子树 f2 带换算 `fy − cy` 下移 δ 成幻影带，把 f3（mt:30）从 30 压到
//! 45+30。修复 = 不稳帧内带集/底边不继承（kill-switch `ZW_FLOAT_STALE_FRAME_BAND_SKIP`
//! 默认开，回退钉在 r5448_float_stale_frame_band_skip_kill_switch.rs）。
//! 对照钉 ②：t2 无 inline 内容（仅 float）时 real h 亦 0（float 不参与非 BFC auto
//! 高，§9.7）→ 帧稳定 → R5307 带透传必须保留（f3 仍避让 f2 带）。
//!
//! 诊断链记档：docs/goal/rendering-compat/evidence/r5371-float-ifc-shortening-2026-11-01.md §2
//!
//! https://www.w3.org/TR/CSS22/visuren.html#float-position

use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::ComputedStyle;

/// 块级样式（margin 均可指定）。
fn block(mt: f64, w: f64, h: f64) -> ComputedStyle {
    ComputedStyle {
        display: DisplayValue::Block,
        margin_top: LengthValue::Px(mt),
        width: LengthValue::Px(w),
        height: LengthValue::Px(h),
        ..Default::default()
    }
}

/// 块级浮动样式。
fn block_float(mt: f64, w: f64, h: f64) -> ComputedStyle {
    ComputedStyle {
        display: DisplayValue::Block,
        float: FloatValue::Left,
        margin_top: LengthValue::Px(mt),
        width: LengthValue::Px(w),
        height: LengthValue::Px(h),
        ..Default::default()
    }
}

/// 按 DOM 节点 ID 在布局树中递归查找对应 LayoutBox。
fn find_box(box_tree: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    if box_tree.node_id == Some(id) {
        return Some(box_tree);
    }
    box_tree.children.iter().find_map(|c| find_box(c, id))
}

/// 驱动案：outer(width 128) → [t2(margin 16)：f2 float 60×45 + 文本 "x"，
/// t3(margin 16)：f3 float 60×45 mt:30 + 文本 "x"]。`prior_text=false` 时 t2
/// 无文本（δ=0 稳定帧对照形）。
fn build(prior_text: bool) -> (Document, HashMap<NodeId, ComputedStyle>, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();

    let mut styles = HashMap::new();
    styles.insert(outer, block(0.0, 128.0, 0.0));

    // t2：前序兄弟（不稳定帧源）
    let t2 = doc.create_element("div");
    doc.append_child(outer, t2).unwrap();
    styles.insert(t2, block(16.0, 0.0, 0.0));
    let f2 = doc.create_element("div");
    doc.append_child(t2, f2).unwrap();
    styles.insert(f2, block_float(0.0, 60.0, 45.0));
    if prior_text {
        let t = doc.create_text_node("x");
        doc.append_child(t2, t).unwrap();
    }

    // t3：受害兄弟
    let t3 = doc.create_element("div");
    doc.append_child(outer, t3).unwrap();
    styles.insert(t3, block(16.0, 0.0, 0.0));
    let f3 = doc.create_element("div");
    doc.append_child(t3, f3).unwrap();
    styles.insert(f3, block_float(30.0, 60.0, 45.0));
    let t = doc.create_text_node("x");
    doc.append_child(t3, t).unwrap();

    (doc, styles, f3, t3)
}

/// 驱动案：t2 文本+float（walk h=0 不稳帧）——f3 只受自身 mt:30，不被幻影带推压。
#[test]
fn phantom_band_does_not_push_float_in_stale_frame() {
    let (doc, styles, f3, _t3) = build(true);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let f3_box = find_box(&result.root, f3).expect("f3 布局盒存在");
    assert_eq!(f3_box.y, 30.0, "f3 应只受自身 margin-top:30，不被 t2 幻影带压到 45+30");
}

/// 对照钉 ②：t2 无 inline 内容（仅 float）→ real h 亦 0（§9.7 float 不参与非 BFC
/// auto 高）→ 帧稳定 → R5307 带透传保留，f3 须避让 f2 带（45）落 45+30。
#[test]
fn stable_frame_band_inheritance_preserved() {
    let (doc, styles, f3, _t3) = build(false);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let f3_box = find_box(&result.root, f3).expect("f3 布局盒存在");
    assert_eq!(
        f3_box.y, 75.0,
        "稳定帧带透传不回退：f3 避让 f2 带（高 45）应落 45+30（守 R5307）"
    );
}
