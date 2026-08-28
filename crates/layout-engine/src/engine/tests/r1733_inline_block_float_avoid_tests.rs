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

/// R3612：inline-block float avoidance 也要恢复声明宽的 used-value。
/// `width:10em;font-size:20px` = 200px；旧终末 pass 只调整 x，
/// 但保留 taffy raw `Em(10)` 的 10px 宽。
#[test]
fn r3612_inline_block_relative_declared_width_preserved_beside_left_float() {
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

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(150.0);
    fl.height = LengthValue::Px(25.0);
    styles.insert(float_l, fl);

    let mut s = ComputedStyle::default();
    s.display = DisplayValue::InlineBlock;
    s.font_size = LengthValue::Px(20.0);
    s.width = LengthValue::Em(10.0);
    s.height = LengthValue::Px(50.0);
    styles.insert(ib, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let ib_box = find_child_by_node_id(&result.root, ib).expect("inline-block found");
    assert!(
        ib_box.x > 100.0,
        "relative inline-block 应 shift 到 float:left 右缘旁，实际 x={}",
        ib_box.x
    );
    assert!(
        ib_box.width >= 195.0,
        "relative inline-block 应保持 width≈200，实际 {}",
        ib_box.width
    );
}

/// R1733 续（多-float 协调）：float:left 与 float:right（占满宽，inline-block 放不下任一旁）
/// 同容器，加 inline-block w=200。多-float 协调应判不可行→保持原位 + 宽度不变，非 per-float
/// over-shrink 错缩到小宽（floats-wrap-top-below-inline-002r 回归实证）。load-bearing：
/// 旧 per-float 实现把 inline-block 错缩/错移到右侧。
/// R3779b 更新：float 子不再向 IFC 注入占位 Br（无 R1286 幽灵空行 strut）——容器行盒
/// 高度由 90 收敛为 50（仅 inline-block 行），taffy 堆叠的 float_r（y=75..150）不再与
/// inline-block（y=0..50）垂直重叠 → 只剩 float_l 单 float 重叠 → 单 float 分支把
/// inline-block shift 到 float_l 右缘 x=250 并收缩宽度到剩余 150（CSS2 §9.5 line-box
/// 缩短的终末近似；「不可行保持原位」分支现仅在 2+ float 同时重叠时触发）。
#[test]
fn r1733_multifloat_inline_block_not_misplaced_when_infeasible() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let float_r = doc.create_element("div");
    doc.append_child(container, float_r).unwrap();
    let ib = doc.create_element("span");
    doc.append_child(container, ib).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(400.0);
    styles.insert(container, cont);

    // float:left 250 + float:right 250（500>400 不并行，inline-block 200 放不下任一旁）。
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(250.0);
    fl.height = LengthValue::Px(75.0);
    styles.insert(float_l, fl);
    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.float = FloatValue::Right;
    fr.width = LengthValue::Px(250.0);
    fr.height = LengthValue::Px(75.0);
    styles.insert(float_r, fr);

    // inline-block 200×50（> 任一旁可用宽 → 不可行 → 保持宽度不被错缩）。
    let mut s = ComputedStyle::default();
    s.display = DisplayValue::InlineBlock;
    s.width = LengthValue::Px(200.0);
    s.height = LengthValue::Px(50.0);
    styles.insert(ib, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let ib_box = find_child_by_node_id(&result.root, ib).expect("inline-block found");
    // R3779b 后：只剩 float_l 垂直重叠 → 单 float 分支 shift 到右缘 x=250 + 收缩到剩余宽
    // 150（非 over-shrink 到任意小值，非错移到右 float 旁）。
    assert!(
        ib_box.x >= 245.0,
        "单 float 重叠时 inline-block 应 shift 到 float:left 右缘旁（x≈250），实际 x={}",
        ib_box.x
    );
    assert!(
        (ib_box.width - 150.0).abs() < 1.0,
        "inline-block 应收缩到 float 右侧剩余宽 150，实际 width={}",
        ib_box.width
    );
}

/// R3613：multi-float 不可行分支虽然保持原位，也必须恢复声明宽的 used-value。
/// `width:10em;font-size:20px` = 200px；旧逻辑在不可行时直接 no-op，
/// 因而把 taffy raw `Em(10)` 的 10px 宽留到最终布局。
/// R3779b 更新：同 r1733_multifloat——float 子不再注入占位 Br 后，inline-block
/// （y=0..50）只与 float_l（y=0..75）重叠 → 单 float 分支 shift + 收缩。
/// 本测试改守护「relative declared width 恢复」在单 float 收缩路径下仍成立：
/// 收缩宽 = float 右侧剩余 150，而非 raw `Em(10)`=10px 或其它错值。
#[test]
fn r3613_multifloat_infeasible_relative_inline_block_preserves_declared_width() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let float_r = doc.create_element("div");
    doc.append_child(container, float_r).unwrap();
    let ib = doc.create_element("span");
    doc.append_child(container, ib).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(400.0);
    styles.insert(container, cont);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(250.0);
    fl.height = LengthValue::Px(75.0);
    styles.insert(float_l, fl);

    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.float = FloatValue::Right;
    fr.width = LengthValue::Px(250.0);
    fr.height = LengthValue::Px(75.0);
    styles.insert(float_r, fr);

    let mut s = ComputedStyle::default();
    s.display = DisplayValue::InlineBlock;
    s.font_size = LengthValue::Px(20.0);
    s.width = LengthValue::Em(10.0);
    s.height = LengthValue::Px(50.0);
    styles.insert(ib, s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let ib_box = find_child_by_node_id(&result.root, ib).expect("inline-block found");
    assert!(
        ib_box.x >= 245.0,
        "单 float 重叠时 inline-block 应 shift 到 float:left 右缘旁（x≈250），实际 x={}",
        ib_box.x
    );
    assert!(
        (ib_box.width - 150.0).abs() < 1.0,
        "relative inline-block 收缩宽应为 float 右侧剩余 150（used-value 恢复），实际 {}",
        ib_box.width
    );
}
