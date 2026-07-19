//! R1733：终末 inline-block float 排斥 pass。
//!
//! atomic inline-level BFC（inline-block，`is_flow_root && !is_block_level`）与同容器 float
//! 垂直重叠时，终末 pass（compute() 末，所有重定位 pass 之后）shift x 到 float 旁，使
//! border-box 不重叠 float（近似 IFC line-box shortening）。floats-wrap-top-below-bfc l 变体
//! REF（inline-block 旁 float 应 x>float 右缘，非 content_left）。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_style_system::ComputedStyle;

/// R1733：float:left + inline-block 同容器 → inline-block x 应 shift 到 float 右缘旁
///（avoidance_x = float 右 margin-box 边），非 content_left。load-bearing：关闭 fix
///（env ZW_BFC_INLINEBLOCK_AVOID=0）则 inline-block 留 content_left（x≈0）。
#[test]
fn r1733_inline_block_shifted_beside_left_float() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let ib = doc.create_element("span");
    doc.append_child(container, ib).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(400.0);
    styles.insert(container, cont);

    // float:left 150×25（占左，右缘=150）。
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(150.0);
    fl.height = LengthValue::Px(25.0);
    styles.insert(float_l, fl);

    // inline-block（atomic inline-level BFC）200×50。
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::InlineBlock;
    s.width = LengthValue::Px(200.0);
    s.height = LengthValue::Px(50.0);
    styles.insert(ib, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let ib_box = find_child_by_node_id(&result.root, ib).expect("inline-block found");
    // inline-block 应 shift 到 float 右缘旁（x≈150，相对 container content），非 content_left（x≈0）。
    assert!(
        ib_box.x > 100.0,
        "inline-block 应 shift 到 float:left 右缘旁（x≈150），非 content_left（x≈0），实际 x={}",
        ib_box.x
    );
}
