//! R5221：多 float 纵向堆叠的下绕流游标修正（CSS2 §9.5 × §10.3.3）。
//!
//! chromium 探针定谳（floats-wrap-bfc-with-margin-007 tentative，puppeteer 几何两跑
//! 一致）：auto 宽 BFC 的 beside fit 用 margin box 对 beside 带判定，装不下即下绕——
//! bfc1（w=CB−mr=100、margin box 50 > 带 25）下绕 float 底 y=50 x=0 w=100；后继
//! float 流位置 ≥ 下绕 BFC 新底（float2 y=100）；definite 宽 BFC 顺序下推（bfc2
//! y=150 w=75）；wrapper h=200。旧臂（sibling 门回退）由 taffy 用 beside 带解
//! §10.3.3 → bfc1 beside x=25 w=75，全链少一行。注：tentative ref 期望 beside
//! 形态，与 chromium 堆叠行为分叉（chromium 自身亦不过该 ref），以 chromium 为准。

use super::*;
use zero_css_parser::values::{ClearValue, DisplayValue, FloatValue, LengthValue, OverflowValue};
use zero_style_system::ComputedStyle;

/// 007 最小形态：容器 50 宽 > float1(25×50, clear:left) > bfc1(overflow:hidden,
/// h=50, mr=−50, auto 宽) > float2(同 float1) > bfc2(overflow:hidden, h=50, w=75)。
#[test]
fn r5221_multifloat_wrap_floor_cascade() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float1 = doc.create_element("div");
    doc.append_child(container, float1).unwrap();
    let bfc1 = doc.create_element("div");
    doc.append_child(container, bfc1).unwrap();
    let float2 = doc.create_element("div");
    doc.append_child(container, float2).unwrap();
    let bfc2 = doc.create_element("div");
    doc.append_child(container, bfc2).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(50.0);
    styles.insert(container, cont);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.clear = ClearValue::Left;
    fl.width = LengthValue::Px(25.0);
    fl.height = LengthValue::Px(50.0);
    styles.insert(float1, fl.clone());
    styles.insert(float2, fl);

    let mut b1 = ComputedStyle::default();
    b1.display = DisplayValue::Block;
    b1.overflow_x = OverflowValue::Hidden;
    b1.overflow_y = OverflowValue::Hidden;
    b1.margin_right = LengthValue::Px(-50.0); // §10.3.3：w = CB − mr = 100
    b1.height = LengthValue::Px(50.0);
    styles.insert(bfc1, b1);

    let mut b2 = ComputedStyle::default();
    b2.display = DisplayValue::Block;
    b2.overflow_x = OverflowValue::Hidden;
    b2.overflow_y = OverflowValue::Hidden;
    b2.width = LengthValue::Px(75.0);
    b2.height = LengthValue::Px(50.0);
    styles.insert(bfc2, b2);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let f1 = find_child_by_node_id(&result.root, float1).expect("float1 found");
    let b1_box = find_child_by_node_id(&result.root, bfc1).expect("bfc1 found");
    let f2 = find_child_by_node_id(&result.root, float2).expect("float2 found");
    let b2_box = find_child_by_node_id(&result.root, bfc2).expect("bfc2 found");
    let cont_box = find_child_by_node_id(&result.root, container).expect("container found");

    // bfc1：下绕 float1 底（margin box 100−50=50 > beside 带 25），全 CB 重解。
    assert!(
        (b1_box.y - 50.0).abs() < 0.5 && (b1_box.x - 0.0).abs() < 0.5 && (b1_box.width - 100.0).abs() < 0.5,
        "bfc1 应下绕 y=50 x=0 w=100（chromium 堆叠形态），实际 y={} x={} w={}",
        b1_box.y,
        b1_box.x,
        b1_box.width
    );
    // 流游标修正：float2 流位置 ≥ bfc1 新底 100。
    assert!(
        (f2.y - 100.0).abs() < 0.5,
        "float2 应落 bfc1 底 y=100（修前滞留 50 与 bfc1 重叠），实际 {}",
        f2.y
    );
    // bfc2：definite 宽按 float2 新带顺序下推。
    assert!(
        (b2_box.y - 150.0).abs() < 0.5 && (b2_box.width - 75.0).abs() < 0.5,
        "bfc2 应 y=150 w=75，实际 y={} w={}",
        b2_box.y,
        b2_box.width
    );
    // 容器高含全部四行（修前 150）。
    assert!(
        (cont_box.height - 200.0).abs() < 0.5 && (f1.y - 0.0).abs() < 0.5,
        "容器 h 应 200（修前 150），实际 {}（float1 y={}）",
        cont_box.height,
        f1.y
    );
}
