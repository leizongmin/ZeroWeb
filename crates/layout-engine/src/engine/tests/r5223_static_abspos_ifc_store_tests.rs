//! R5223：静态位（全 inset auto）abspos 盒的绘制度量存储（CSS2 §10.3.7 × §10.8）。
//!
//! chromium 探针 + 案面实证：display:inline + position:absolute 盒块化（block-level），
//! 其 Final Inline Pass 时未挂入 children（后处理链附加）、内部 IFC 度量无处存储；
//! 水平 IFC collect 期 OOF 子被跳过（skip_oof 臂），paint Path B（空 styles）丢继承
//! 字号——vertical-align-sub/super-001 的 "Filler Text" 1in→16px。后处理水平臂直接
//! 以子盒 style 合成 text_node_font_sizes 等 override maps（Path B 无需 inline_layout），
//! 非空 maps 同时使 abspos_self_paint（R1548 安全类）生效。同轮：根级文本 run 不继承
//! out-of-flow 容器自身 vertical-align（§10.8 va 仅适用 inline-level；sub/super 泄入
//! 曾把行盒撑高 0.3·fs——111→140，红绿覆盖层错位）。

use super::*;
use zero_css_parser::values::{DisplayValue, PositionValue, VerticalAlignValue};
use zero_style_system::ComputedStyle;

/// 静态位 abspos span（全 inset auto、继承 96px、va=super）应存绘制度量 override
/// maps（text_node_font_sizes 非空 = Path B 继承字号 + abspos_self_paint 前提），
/// 且盒高不受 va 影响。
#[test]
fn r5223_static_abspos_stores_metrics_and_ignores_own_va() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let red = doc.create_element("span");
    doc.append_child(container, red).unwrap();
    let green = doc.create_element("span");
    doc.append_child(container, green).unwrap();
    let t1 = doc.create_text_node("Filler Text");
    doc.append_child(red, t1).unwrap();
    let t2 = doc.create_text_node("Filler Text");
    doc.append_child(green, t2).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.font_size = zero_css_parser::values::LengthValue::Px(96.0);
    styles.insert(container, cont);

    let mut s_abs = ComputedStyle::default();
    s_abs.display = DisplayValue::Inline;
    s_abs.position = PositionValue::Absolute;
    s_abs.font_size = zero_css_parser::values::LengthValue::Px(96.0); // styles map 扁平，直接挂 span
    // top/right/bottom/left 全默认 Auto = 静态位盒
    styles.insert(red, s_abs.clone());

    let mut s_va = s_abs;
    s_va.vertical_align = VerticalAlignValue::Super;
    styles.insert(green, s_va);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let red_box = find_child_by_node_id(&result.root, red).expect("red span box found");
    let green_box = find_child_by_node_id(&result.root, green).expect("green span box found");

    // 存储面：静态位 abspos 盒存绘制度量 override maps（Path B 继承字号前提）。
    assert!(
        !red_box.text_node_font_sizes.is_empty(),
        "静态位 abspos span 应存 text_node_font_sizes（Path B 字号 + abspos_self_paint 前提），实际空（h={} w={} ch={}）",
        red_box.height,
        red_box.width,
        red_box.content_width
    );
    assert!(
        (red_box.text_node_font_sizes.values().copied().sum::<f32>() / red_box.text_node_font_sizes.len() as f32
            - 96.0)
            .abs()
            < 0.5,
        "override 字号应为继承的 96px，实际 {:?}",
        red_box.text_node_font_sizes
    );

    // 几何面：容器自身 va=super 不得影响内部行盒（两盒同高，无 0.3·fs 撑高）。
    assert!(
        (red_box.height - green_box.height).abs() < 0.5,
        "va=super 盒应与 va=baseline 盒同高（根级 run 不继承容器 va），实际 {} vs {}",
        red_box.height,
        green_box.height
    );
    // 字号面：96px 字号的行盒高应显著大于 16px 默认（继承字号进入度量）。
    assert!(
        red_box.height > 60.0,
        "abspos span 内部行盒应按继承 96px 度量，实际 h={}",
        red_box.height
    );
}
