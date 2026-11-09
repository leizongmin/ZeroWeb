//! R5448 kill-switch 回退锚（独立测试进程——`ZW_FLOAT_STALE_FRAME_BAND_SKIP=0` 时
//! walk 不稳帧内跨兄弟带集/底边继承恢复旧行为，幻影带重现；进程内 set_var 与并行
//! 单测互踩，R3858 教训——本文件仅此一测，独占进程环境）。
//!
//! 谱系与几何同 `r5448_float_stale_frame_band_skip.rs` 驱动案（t2 文本+float 不稳帧
//! → t3 帧 y stale → f2 带幻影下移 → f3 被 45+30 压位）。

use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::ComputedStyle;

#[test]
fn kill_switch_restores_phantom_band_push() {
    // Rust 2024：set_var 为 unsafe（进程全局可变）；OnceLock 首读前生效。
    unsafe { std::env::set_var("ZW_FLOAT_STALE_FRAME_BAND_SKIP", "0") };

    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();

    let mut styles = HashMap::new();
    let mk_block = |mt: f64, w: f64, h: f64| ComputedStyle {
        display: DisplayValue::Block,
        margin_top: LengthValue::Px(mt),
        width: LengthValue::Px(w),
        height: LengthValue::Px(h),
        ..Default::default()
    };
    let mk_float = |mt: f64, w: f64, h: f64| ComputedStyle {
        display: DisplayValue::Block,
        float: FloatValue::Left,
        margin_top: LengthValue::Px(mt),
        width: LengthValue::Px(w),
        height: LengthValue::Px(h),
        ..Default::default()
    };
    styles.insert(outer, mk_block(0.0, 128.0, 0.0));

    // t2：文本 + float（walk h=0 不稳帧源）
    let t2 = doc.create_element("div");
    doc.append_child(outer, t2).unwrap();
    styles.insert(t2, mk_block(16.0, 0.0, 0.0));
    let f2 = doc.create_element("div");
    doc.append_child(t2, f2).unwrap();
    styles.insert(f2, mk_float(0.0, 60.0, 45.0));
    let t2_text = doc.create_text_node("x");
    doc.append_child(t2, t2_text).unwrap();

    // t3：受害兄弟（f3 mt:30）
    let t3 = doc.create_element("div");
    doc.append_child(outer, t3).unwrap();
    styles.insert(t3, mk_block(16.0, 0.0, 0.0));
    let f3 = doc.create_element("div");
    doc.append_child(t3, f3).unwrap();
    styles.insert(f3, mk_float(30.0, 60.0, 45.0));
    let t3_text = doc.create_text_node("x");
    doc.append_child(t3, t3_text).unwrap();

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let find = |tree: &LayoutBox, id: NodeId| -> (f32, f32) {
        fn walk(tree: &LayoutBox, id: NodeId) -> Option<(f32, f32)> {
            if tree.node_id == Some(id) {
                return Some((tree.x, tree.y));
            }
            tree.children.iter().find_map(|c| walk(c, id))
        }
        walk(tree, id).expect("布局盒存在")
    };
    let (_, fy) = find(&result.root, f3);
    assert_eq!(
        fy, 75.0,
        "kill-switch=0 恢复旧行为：幻影带（f2 带下移 δ）把 f3 从 30 压到 45+30"
    );
}
