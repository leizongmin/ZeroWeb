//! R1722：float:right definite-width BFC 放不下 float 左侧可用宽 → 推到 float 下方
//!（mirror of R1369 float:left overflows 推下）。
//!
//! `floats-wrap-bfc-005` 子案 4：overflow:hidden width:50%（=150）旁 float:right 200（300 容器）。
//! chromium：BFC 不重叠 float，150 > 100 可用 → 推到 float 下方（y=20, w=150 保持）。
//! ZW 旧：float:right 分支仅 shrink（w=100 beside），不下沉。R1722 mirror R1369 推下。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue, OverflowValue};
use zero_style_system::ComputedStyle;

/// R1722：float:right + definite-width BFC（overflow:hidden）放不下 → 推到 float 下方，
/// 非 shrink beside。load-bearing：关闭 fix（env ZW_BFC_RIGHT_PUSHBELOW=0）则 div 被 shrink
/// 到 100 留在 beside。
#[test]
fn r1722_right_bfc_definite_width_pushed_below_float() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_r = doc.create_element("div");
    doc.append_child(container, float_r).unwrap();
    let bfc_div = doc.create_element("div");
    doc.append_child(container, bfc_div).unwrap();

    let mut styles = HashMap::new();
    // 容器 width=300（auto height）。
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    // float:right 200×20（贴右侧，left_edge=100）。
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Right;
    fl.width = LengthValue::Px(200.0);
    fl.height = LengthValue::Px(20.0);
    styles.insert(float_r, fl);

    // BFC div（overflow:hidden）width=150（=50%，> 100 可用 beside → 应推下非 shrink）。
    let mut bd = ComputedStyle::default();
    bd.display = DisplayValue::Block;
    bd.overflow_x = OverflowValue::Hidden;
    bd.overflow_y = OverflowValue::Hidden;
    bd.width = LengthValue::Px(150.0);
    bd.height = LengthValue::Px(20.0);
    styles.insert(bfc_div, bd);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc_box = find_child_by_node_id(&result.root, bfc_div).expect("bfc div found");
    // BFC div 应推到 float 下方（y≈20）保持 width=150，非 shrink 到 100 留 beside（y≈0）。
    assert!(
        bfc_box.width >= 145.0,
        "float:right definite-width BFC 应保持 width=150（推下非 shrink），实际 {}",
        bfc_box.width
    );
    assert!(
        bfc_box.y > 10.0,
        "BFC 应推到 float 下方（y≈20）非 beside（y≈0），实际 y={}",
        bfc_box.y
    );
}

/// R3610：right-float BFC avoidance 进入条件也要使用声明宽的 used-value。
/// `width:8em;font-size:20px` = 160px，float:right 200 后左侧只剩 100px；
/// 旧实现用 taffy 残留 `Em(8)` 的 raw width=8 做重叠判断，直接跳过 avoidance。
#[test]
fn r3610_right_bfc_relative_declared_width_pushed_below_float() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_r = doc.create_element("div");
    doc.append_child(container, float_r).unwrap();
    let bfc_div = doc.create_element("div");
    doc.append_child(container, bfc_div).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Right;
    fl.width = LengthValue::Px(200.0);
    fl.height = LengthValue::Px(20.0);
    styles.insert(float_r, fl);

    let mut bd = ComputedStyle::default();
    bd.display = DisplayValue::Block;
    bd.overflow_x = OverflowValue::Hidden;
    bd.overflow_y = OverflowValue::Hidden;
    bd.font_size = LengthValue::Px(20.0);
    bd.width = LengthValue::Em(8.0);
    bd.height = LengthValue::Px(20.0);
    styles.insert(bfc_div, bd);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc_box = find_child_by_node_id(&result.root, bfc_div).expect("bfc div found");
    assert!(
        bfc_box.width >= 155.0,
        "float:right relative declared-width BFC 应保持 width≈160（推下非 shrink），实际 {}",
        bfc_box.width
    );
    assert!(
        bfc_box.y > 10.0,
        "relative declared-width BFC 应推到 right float 下方（y≈20）非 beside（y≈0），实际 y={}",
        bfc_box.y
    );
}
