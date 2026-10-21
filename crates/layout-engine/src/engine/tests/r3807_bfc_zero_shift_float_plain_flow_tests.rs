//! R3807：零位移 float 不构成 BFC 回避约束（CSS2 §9.5 + §10.3.3）。
//!
//! chromium 实证：零位移 float（左 float avoidance_x = 0 / 右 float float_x = 容器宽）
//! 的 float offset 与无 float 同值 → 不构成 BFC 回避约束，BFC 走普通 §10.3.3 流内求解：
//! 负 margin 扩张盒子（w = cb − ml − mr，border box 左 = ml）。
//! zero-width-floats.html 依赖此语义（两个 0 宽 float 夹 overflow:hidden 负 margin BFC）。
//!
//! R5217 起，真约束 float 旁的 BFC 改用 chromium FloatAvoider 钉位语义（68 行几何探针
//! 实证，见 `r5217_bfc_margin_pin_beside_tests.rs`）：受约束 BFC border box 钉在
//! float-free band [L, R] 内（rf ml=−50 @ gap=75 → x=0 w=75，探针实证；旧 shrink 的
//! [−50,125] 形状 chromium 不产生）。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue, OverflowValue};
use zero_style_system::ComputedStyle;

/// 零位移 float 不构成约束：0 宽 float 左（贴容器左缘）+ 0 宽 float 右（贴容器右缘）
/// 时，overflow:hidden BFC 保持普通流内几何 x=ml=−50、w=cb−ml−mr=200。
/// 旧行为：被左/右 float 排斥臂钉成 x=0、w=100（zero-width-floats.html 2.08% 根因）。
#[test]
fn r3807_zero_shift_floats_bfc_keeps_plain_flow_negative_margin_geometry() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let float_r = doc.create_element("div");
    doc.append_child(container, float_r).unwrap();
    let bfc_div = doc.create_element("div");
    doc.append_child(container, bfc_div).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(100.0);
    styles.insert(container, cont);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(0.0);
    fl.height = LengthValue::Px(50.0);
    styles.insert(float_l, fl);

    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.float = FloatValue::Right;
    fr.width = LengthValue::Px(0.0);
    fr.height = LengthValue::Px(150.0);
    styles.insert(float_r, fr);

    let mut bd = ComputedStyle::default();
    bd.display = DisplayValue::Block;
    bd.overflow_x = OverflowValue::Hidden;
    bd.overflow_y = OverflowValue::Hidden;
    bd.margin_left = LengthValue::Px(-50.0);
    bd.margin_right = LengthValue::Px(-50.0);
    bd.height = LengthValue::Px(100.0);
    styles.insert(bfc_div, bd);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc_box = find_child_by_node_id(&result.root, bfc_div).expect("bfc div found");
    // §10.3.3 流内求解：w = 100 − (−50) − (−50) = 200，border box 左 = ml = −50。
    assert!(
        (bfc_box.width - 200.0).abs() < 0.5,
        "零位移 float 旁 BFC 应保持流内 w=200（负 margin 扩张），实际 {}",
        bfc_box.width
    );
    assert!(
        (bfc_box.x - (-50.0)).abs() < 0.5,
        "零位移 float 旁 BFC border box 左应在 ml=−50，实际 {}",
        bfc_box.x
    );
}

/// 真约束右 float（可发生垂直重叠）下负 margin BFC 走 R5217 钉位语义：border box 左被钳
/// 到 band 左缘 0、右缘钉 float 左缘 75 → w=75。chromium 探针实证（2026-10-22，
/// `/tmp/r5217_r3807_probe.mjs`：rf ml=−50 → x=0 w=75 y=0；旧 shrink 的 x=−50 w=125
/// chromium 不产生）。零位移 guard（上一测试）不受影响。
#[test]
fn r3807_constrained_right_float_pins_negative_margin_bfc_into_band() {
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
    cont.width = LengthValue::Px(100.0);
    styles.insert(container, cont);

    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.float = FloatValue::Right;
    fr.width = LengthValue::Px(25.0);
    fr.height = LengthValue::Px(50.0);
    styles.insert(float_r, fr);

    let mut bd = ComputedStyle::default();
    bd.display = DisplayValue::Block;
    bd.overflow_x = OverflowValue::Hidden;
    bd.overflow_y = OverflowValue::Hidden;
    bd.margin_left = LengthValue::Px(-50.0);
    bd.height = LengthValue::Px(60.0);
    styles.insert(bfc_div, bd);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let bfc_box = find_child_by_node_id(&result.root, bfc_div).expect("bfc div found");
    // R5217 钉位：band = [0, float mb 左缘 75]（gap=75，margin_h=−50 阈内），左缘钳 0、
    // 右缘钉 75 → w=75（chromium 探针同值）。
    assert!(
        (bfc_box.x - 0.0).abs() < 0.5,
        "负 margin BFC 左缘应被钳到 band 左缘 0，实际 {}",
        bfc_box.x
    );
    assert!(
        (bfc_box.width - 75.0).abs() < 0.5,
        "右缘应钉 float 左缘（w=75），实际 {}",
        bfc_box.width
    );
}
