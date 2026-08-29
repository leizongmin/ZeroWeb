//! R3809：原子行内级盒 top/bottom 对齐边是 margin box（CSS §10.8.1）。
//!
//! margin-applies-to-012/014 族：abspos shrink-wrap 内 inline-block（margin 50、
//! va:bottom）旧按 border box 对齐 → border 顶多下移 margin_bottom（+50px，2.08% 簇）。
//! chromium CDP 实证：va:top + margin-top:25 的 border box y=25（margin box 顶对齐行顶）；
//! va:bottom 的 border box 底 = 行底 − margin_bottom（margin box 底贴行底）。

use super::*;

#[test]
fn r3809_va_bottom_aligns_margin_box() {
    let (mut doc, body) = make_doc_with_body();
    let wrapper = doc.create_element("div");
    doc.append_child(body, wrapper).unwrap();
    let test = doc.create_element("div");
    doc.append_child(wrapper, test).unwrap();

    let mut styles = HashMap::new();
    let mut ws = ComputedStyle::default();
    ws.display = zero_css_parser::values::DisplayValue::Block;
    ws.position = zero_css_parser::values::PositionValue::Absolute;
    styles.insert(wrapper, ws);

    let mut ts = ComputedStyle::default();
    ts.display = zero_css_parser::values::DisplayValue::InlineBlock;
    ts.width = zero_css_parser::values::LengthValue::Px(200.0);
    ts.height = zero_css_parser::values::LengthValue::Px(200.0);
    ts.margin_top = zero_css_parser::values::LengthValue::Px(50.0);
    ts.margin_left = zero_css_parser::values::LengthValue::Px(50.0);
    ts.margin_right = zero_css_parser::values::LengthValue::Px(50.0);
    ts.margin_bottom = zero_css_parser::values::LengthValue::Px(50.0);
    ts.vertical_align = zero_css_parser::values::VerticalAlignValue::Bottom;
    styles.insert(test, ts);

    let mut engine = LayoutEngine::new(400.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let t = find_child_by_node_id(&result.root, test).expect("test");
    // va:bottom 对齐把 margin-box 底贴行底 → border box 顶 = margin_top = 50。
    // 旧行为：按 border box 对齐 → y=100（多下移一个 margin_bottom）。
    assert!(
        (t.y - 50.0).abs() < 0.5,
        "va:bottom 原子盒 border 顶应为 margin_top=50（margin-box 对齐），实际 {}",
        t.y
    );
}
