//! R1728：float:left 占满宽致其右可用宽 < BFC **声明**宽 → 推到 float 下方（非 shrink）。
//!
//! `floats-wrap-top-below-bfc-002r` span2：容器 400，float:left 300（右可用仅 100），
//! BFC span（overflow:hidden）声明宽 200 放不下 float 右侧 → 应 pushdown 到 float 下方
//! 保持 width=200（chromium 行为）。ZW 旧：R1369 gate 仅查「溢出容器」（200 < 400 不溢出）
//! → 漏判，走到 squeeze 分支缩到 w=100 留在 beside（错）。
//!
//! 关键区分：仅对「声明宽（非 auto）」BFC 触发——auto 宽 BFC（floats-bfc-003 #bfc、
//! new-fc-beside-float）须 shrink-to-fit 旁置（spec：BFC 占 float 旁可用宽）。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue, OverflowValue};
use zero_style_system::ComputedStyle;

/// R1728：float:left 300 + 声明宽 200 的 BFC（overflow:hidden）放不下 float 右侧可用宽 100
/// → 推到 float 下方保持 width=200，非 shrink 到 100 留 beside。
/// load-bearing：关闭 fix（env ZW_BFC_LEFT_FIT_PUSHBELOW=0）则 span 被 squeeze 到 100 留 beside。
#[test]
fn r1728_declared_width_bfc_pushed_below_fullwidth_left_float() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let bfc_span = doc.create_element("span");
    doc.append_child(container, bfc_span).unwrap();

    let mut styles = HashMap::new();
    // 容器 width=400（auto height）。
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(400.0);
    styles.insert(container, cont);

    // float:left 300×75（占满宽，右可用仅 [300,400]=100）。
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(300.0);
    fl.height = LengthValue::Px(75.0);
    styles.insert(float_l, fl);

    // BFC span（display:block + overflow:hidden）声明宽 200（> 100 可用 → 应推下非 shrink）。
    let mut bs = ComputedStyle::default();
    bs.display = DisplayValue::Block;
    bs.overflow_x = OverflowValue::Hidden;
    bs.overflow_y = OverflowValue::Hidden;
    bs.width = LengthValue::Px(200.0);
    bs.height = LengthValue::Px(50.0);
    styles.insert(bfc_span, bs);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc_box = find_child_by_node_id(&result.root, bfc_span).expect("bfc span found");
    // BFC span 应保持声明宽 200（推下非 shrink），不应被 squeeze 到 ≤100。
    assert!(
        bfc_box.width >= 195.0,
        "声明宽 BFC 应保持 width=200（推下非 shrink），实际 {}",
        bfc_box.width
    );
    // BFC span 应推到 float 下方（y≈75）非 beside（y≈0）。
    assert!(
        bfc_box.y > 60.0,
        "BFC 应推到 float 下方（y≈75）非 beside（y≈0），实际 y={}",
        bfc_box.y
    );
}

/// R1728 回归守卫：auto 宽 BFC（无声明 width）旁 float:left 仍 shrink-to-fit 旁置，
/// **不**被 R1728 推下（floats-bfc-003 / new-fc-beside-float 行为）。
#[test]
fn r1728_auto_width_bfc_still_shrinks_beside_left_float() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let bfc_div = doc.create_element("div");
    doc.append_child(container, bfc_div).unwrap();

    let mut styles = HashMap::new();
    // 容器 width=300。
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    // float:left 200×50（右可用 [200,300]=100）。
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(200.0);
    fl.height = LengthValue::Px(50.0);
    styles.insert(float_l, fl);

    // BFC div（overflow:hidden）无声明 width（auto）→ shrink-to-fit 旁置，y≈0 非推下。
    let mut bd = ComputedStyle::default();
    bd.display = DisplayValue::Block;
    bd.overflow_x = OverflowValue::Hidden;
    bd.overflow_y = OverflowValue::Hidden;
    bd.height = LengthValue::Px(50.0);
    styles.insert(bfc_div, bd);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc_box = find_child_by_node_id(&result.root, bfc_div).expect("bfc div found");
    // auto 宽 BFC 应旁置（y≈0），不应被 R1728 推到 float 下方（y≈50）。
    assert!(
        bfc_box.y < 25.0,
        "auto 宽 BFC 应 shrink-to-fit 旁置（y≈0）非推下（y≈50），实际 y={}",
        bfc_box.y
    );
}
