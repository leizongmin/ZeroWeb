//! R3807：零位移 float 不构成 BFC 回避约束（CSS2 §9.5 + §10.3.3）。
//!
//! chromium 实证：零位移 float（左 float avoidance_x = 0 / 右 float float_x = 容器宽）
//! 的 float offset 与无 float 同值 → 不构成 BFC 回避约束，BFC 走普通 §10.3.3 流内求解：
//! 负 margin 扩张盒子（w = cb − ml − mr，border box 左 = ml）。
//! zero-width-floats.html 依赖此语义（两个 0 宽 float 夹 overflow:hidden 负 margin BFC）。
//!
//! 钉位语义（chromium FloatAvoider 把受约束 BFC border box 钉在 float-free band [L, R]）
//! 实测存在：但套用会回归 floats-wrap-bfc-with-margin-006（ZW 与 chromium 的 float 垂直
//! 堆叠 y 模型不同——ZW 中 BFC 与 float 同行垂直重叠时旧 shrink 臂已是 reftest 正解），
//! 按净变更≥0 纪律不落。第三案锁定该范围（真约束右 float 不改变既有 shrink 行为）。

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

/// 真约束右 float（可发生垂直重叠）保持既有 shrink 语义：border box 左 = ml（负 margin
/// 不被钳到 band 左缘）、右缘收缩到 float 左缘。锁定 R3807 修复范围——零位移 guard 之外
/// 的回避臂不动（floats-wrap-bfc-with-margin-006 依赖，net≥0 纪律）。
#[test]
fn r3807_constrained_right_float_keeps_existing_shrink_semantics() {
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
    // 既有几何（taffy band 求解 + 回避臂右缘恰触 float 时跳过）：左缘 = ml = −50，
    // 右缘 = float 左缘 75 → w = 125。R3807 不改变此形状（006 reftest 依赖）。
    assert!(
        (bfc_box.x - (-50.0)).abs() < 0.5,
        "负 margin BFC 左缘应保持 ml=−50（不被钉位），实际 {}",
        bfc_box.x
    );
    assert!(
        (bfc_box.width - 125.0).abs() < 0.5,
        "右缘应抵 float 左缘（w=125 = 75−(−50)），实际 {}",
        bfc_box.width
    );
}
