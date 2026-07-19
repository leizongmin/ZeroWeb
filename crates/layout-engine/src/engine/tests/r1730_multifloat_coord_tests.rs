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
