//! R1730 Slice 5（RFC §10.2）：多-float BFC 协调。BFC 子同时垂直重叠 ≥2 同容器 float 时，
//! per-float 循环独立 pushdown 会 over-push；协调找首个使 BFC 不重叠任何 float 的 y。
//!
//! `floats-wrap-top-below-bfc-003l`：容器 400，float:left 250×75 + float:right 250×75
//!（500>400 不并行，R 下沉到 L 底）+ 2× BFC span（block overflow:hidden w=100 h=50）。
//! chromium：span2 下到 float L 底（y≈89）旁 float R（x<161）；ZW 旧推到 float R 底（y≈164）。
//! R1730 协调：候选 y={64, 89(float L 底), 164(float R 底)}，y=89 处只剩 float R（[161,411]），
//! span2 w=100 放得下其左 [0,61] → 取 y=89 x=0。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue, OverflowValue};
use zero_style_system::ComputedStyle;

/// R1730：float:left 250 + float:right 250 + BFC span w=100，span2 协调到 float L 底旁 float R
///（y≈75 非 y≈150），不 over-push 到 float R 底。load-bearing：关闭 fix（env
/// ZW_BFC_MULTIFLOAT_COORD=0）则 span2 被推到 float R 底（y≈150）。
#[test]
fn r1730_multifloat_coord_span2_below_first_float_beside_second() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let float_r = doc.create_element("div");
    doc.append_child(container, float_r).unwrap();
    let span1 = doc.create_element("span");
    doc.append_child(container, span1).unwrap();
    let span2 = doc.create_element("span");
    doc.append_child(container, span2).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(400.0);
    styles.insert(container, cont);

    // float:left 250×75（占左，右可用 [250,400]=150）。
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(250.0);
    fl.height = LengthValue::Px(75.0);
    styles.insert(float_l, fl);

    // float:right 250×75（与 L 不并行 → 下沉到 L 底 y=75）。
    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.float = FloatValue::Right;
    fr.width = LengthValue::Px(250.0);
    fr.height = LengthValue::Px(75.0);
    styles.insert(float_r, fr);

    // 两 BFC span（block overflow:hidden）w=100 h=50。
    let mk_span = || {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.overflow_x = OverflowValue::Hidden;
        s.overflow_y = OverflowValue::Hidden;
        s.width = LengthValue::Px(100.0);
        s.height = LengthValue::Px(50.0);
        s
    };
    styles.insert(span1, mk_span());
    styles.insert(span2, mk_span());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let span2_box = find_child_by_node_id(&result.root, span2).expect("span2 found");
    // span2 应协调到 float L 底（y≈75）旁 float R，**不**推到 float R 底（y≈150）。
    assert!(
        span2_box.y < 120.0,
        "span2 应协调到 float L 底（y≈75）旁 float R，非 over-push 到 float R 底（y≈150），实际 y={}",
        span2_box.y
    );
}

/// R1730 续（margin_auto plumbing）：2× float:right clear:right（叠右侧）+ BFC span
/// `margin-left:auto` w=200。span2 应右对齐到最晚 float（div2）左缘旁（x≈111），与 span1
/// 相邻（y≈64），非推到两 float 下。load-bearing：无 margin_left_auto 字段则 span2 的 x_lo
/// 起始 = 解析后大 margin_left → 误判 y=64 不可行 → over-push 到 float 底。
#[test]
fn r1730_margin_auto_bfc_right_aligns_to_leftmost_obstructing_float() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let div1 = doc.create_element("div");
    doc.append_child(container, div1).unwrap();
    let div2 = doc.create_element("div");
    doc.append_child(container, div2).unwrap();
    let span1 = doc.create_element("span");
    doc.append_child(container, span1).unwrap();
    let span2 = doc.create_element("span");
    doc.append_child(container, span2).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(400.0);
    styles.insert(container, cont);

    // 2× float:right clear:right（叠右侧）：div1 50×75 @ right，div2 100×75 clear:right 下沉。
    let mut d1 = ComputedStyle::default();
    d1.display = DisplayValue::Block;
    d1.float = FloatValue::Right;
    d1.clear = zero_css_parser::values::ClearValue::Right;
    d1.width = LengthValue::Px(50.0);
    d1.height = LengthValue::Px(75.0);
    styles.insert(div1, d1);
    let mut d2 = ComputedStyle::default();
    d2.display = DisplayValue::Block;
    d2.float = FloatValue::Right;
    d2.clear = zero_css_parser::values::ClearValue::Right;
    d2.width = LengthValue::Px(100.0);
    d2.height = LengthValue::Px(75.0);
    styles.insert(div2, d2);

    // 两 BFC span（block overflow:hidden）w=200 h=50，margin-left:auto（右对齐）。
    let mk_span = || {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.overflow_x = OverflowValue::Hidden;
        s.overflow_y = OverflowValue::Hidden;
        s.width = LengthValue::Px(200.0);
        s.height = LengthValue::Px(50.0);
        s.margin_left = LengthValue::Auto;
        s
    };
    styles.insert(span1, mk_span());
    styles.insert(span2, mk_span());

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let span2_box = find_child_by_node_id(&result.root, span2).expect("span2 found");
    // span2 应右对齐到 div2 左缘旁（x≈111，右缘≈311=div2 左），y 与 span1 相邻（≈64），
    // 非 over-push 到两 float 下（y≈150）。
    assert!(
        span2_box.y < 120.0,
        "margin-auto BFC span2 应与 span1 相邻（y≈64）右对齐到 div2 左缘，非推到 float 下（y≈150），实际 y={}",
        span2_box.y
    );
    assert!(
        span2_box.x > 80.0,
        "span2 应右对齐到 div2 左缘旁（x≈111），实际 x={}",
        span2_box.x
    );
}

/// R3611：多-float coordination 的可行区间也要使用声明宽的 used-value。
/// 两侧 float 各 150px，中间只剩 100px；`width:6em;font-size:20px` = 120px，
/// BFC 不应被 raw `Em(6)` 当成 6px 后放进窄缝。
#[test]
fn r3611_multifloat_coord_relative_declared_width_drops_below() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let float_r = doc.create_element("div");
    doc.append_child(container, float_r).unwrap();
    let bfc = doc.create_element("span");
    doc.append_child(container, bfc).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(400.0);
    styles.insert(container, cont);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(150.0);
    fl.height = LengthValue::Px(50.0);
    styles.insert(float_l, fl);

    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.float = FloatValue::Right;
    fr.width = LengthValue::Px(150.0);
    fr.height = LengthValue::Px(50.0);
    styles.insert(float_r, fr);

    let mut s = ComputedStyle::default();
    s.display = DisplayValue::Block;
    s.overflow_x = OverflowValue::Hidden;
    s.overflow_y = OverflowValue::Hidden;
    s.font_size = LengthValue::Px(20.0);
    s.width = LengthValue::Em(6.0);
    s.height = LengthValue::Px(25.0);
    styles.insert(bfc, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc_box = find_child_by_node_id(&result.root, bfc).expect("bfc found");
    assert!(
        bfc_box.width >= 115.0,
        "multi-float relative declared-width BFC 应保持 width≈120，实际 {}",
        bfc_box.width
    );
    assert!(
        bfc_box.y > 40.0,
        "multi-float 中间 100px 不足容纳 120px BFC，应推到 float 下方，实际 y={}",
        bfc_box.y
    );
}
