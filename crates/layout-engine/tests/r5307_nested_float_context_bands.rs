//! R5307：同 BFC 上下文内前序嵌套 float 带集透传与 §9.5.1 墙模型回归钉。
//!
//! 两层缺陷的布局层复现（原为 position-absolute-dynamic-relayout-004 驱动）：
//! 1. **透传/消费**：非 BFC 兄弟块内的嵌套 float（float1 在早序 wrapper 内）对
//!    后继兄弟块内的 float（float2）不可见 → float2 从 y=0 落位叠压 float1。
//!    修复：递归收集非 BFC 后代 float 几何（`nested_float_geoms`）并入后继子的
//!    inherited 带集，earliest-fit 试探按文档序过滤后约束落位。
//! 2. **墙模型**：earliest-fit 对 §9.5.1 规则 1/2/3 的容器壁钳制过强——左系
//!    float 仅当存在相交左系带（「another left-floating box to its left」）时
//!    右缘才受容器右缘约束，否则可溢出容器右缘（规则 3）。修复前无条件下
//!    `min(container_width)`，同 BFC 外层 float（425 宽目标 vs 400 宽容器）被
//!    误下移，floats-rule3/7-outside-* 家族翻红。
//!
//! 另注（已知挂账，非本修复范围）：同 walk 内直接兄弟 float（flat 形态）
//! line_y=0 时仍叠压——taffy 期无带集可收集（Phase 1），与修复前逐值一致，
//! 测试钉住现状防意外回归。
//! 诊断链记档：docs/goal/rendering-compat/evidence/r5307-nested-float-context-bands-2026-10-28.md
//!
//! https://www.w3.org/TR/CSS22/visuren.html#float-position

use std::collections::HashMap;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_dom::{Document, NodeId};
use zero_layout_engine::{LayoutBox, LayoutEngine};
use zero_style_system::ComputedStyle;

/// 块级无浮动样式（w/h px）。
fn block(w: f64, h: f64) -> ComputedStyle {
    ComputedStyle {
        display: DisplayValue::Block,
        width: LengthValue::Px(w),
        height: LengthValue::Px(h),
        ..Default::default()
    }
}

/// 块级浮动样式（w/h px）。
fn block_float(direction: FloatValue, w: f64, h: f64) -> ComputedStyle {
    ComputedStyle {
        display: DisplayValue::Block,
        float: direction,
        width: LengthValue::Px(w),
        height: LengthValue::Px(h),
        ..Default::default()
    }
}

/// 按 DOM 节点 ID 在布局树中递归查找对应 LayoutBox。
fn find_box(box_tree: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
    if box_tree.node_id == Some(id) {
        return Some(box_tree);
    }
    box_tree.children.iter().find_map(|c| find_box(c, id))
}

/// 驱动案结构：outer(100×100) → [w1 → f1(100×50 float:left)，后继兄弟块
/// mid → f2(100×50 float:left)]；`nested=false` 时 f2 直接挂在 outer 下。
fn build(second_float_nested: bool) -> (Document, HashMap<NodeId, ComputedStyle>, NodeId, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();

    let mut styles = HashMap::new();
    styles.insert(outer, block(100.0, 100.0));

    // float1 包装块 + float1（100×50）
    let w1 = doc.create_element("div");
    doc.append_child(outer, w1).unwrap();
    styles.insert(w1, block(0.0, 0.0));
    let f1 = doc.create_element("div");
    doc.append_child(w1, f1).unwrap();
    styles.insert(f1, block_float(FloatValue::Left, 100.0, 50.0));

    let (mid, f2);
    if second_float_nested {
        // float2 在后继兄弟块内（驱动案形态）
        let m = doc.create_element("div");
        doc.append_child(outer, m).unwrap();
        styles.insert(m, block(0.0, 0.0));
        let f = doc.create_element("div");
        doc.append_child(m, f).unwrap();
        mid = m;
        f2 = f;
    } else {
        // float2 与 float1 的包装块同层（flat 形态）
        let f = doc.create_element("div");
        doc.append_child(outer, f).unwrap();
        mid = outer;
        f2 = f;
    }
    styles.insert(f2, block_float(FloatValue::Left, 100.0, 50.0));
    (doc, styles, mid, f2)
}

/// 修复目标：后继兄弟块内 float2 须避开前序嵌套 float1（y=50），不再从 y=0 叠压。
#[test]
fn nested_sibling_float_descends_below_prior_nested_float() {
    let (doc, styles, mid, f2) = build(true);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mid_box = find_box(&result.root, mid).expect("mid 布局盒存在");
    let f2_box = find_box(&result.root, f2).expect("f2 布局盒存在");
    assert_eq!(
        (f2_box.x, f2_box.y),
        (0.0, 50.0),
        "float2 应落在前序嵌套 float1（高 50）之下；mid 内子数 {}",
        mid_box.children.len()
    );
}

/// flat 形态现状钉（已知挂账）：同 walk 直接兄弟 float 在 line_y=0 时仍叠压，
/// 与修复前逐值一致——透传修复不得改变该形态（taffy 期无带集可收集）。
#[test]
fn flat_sibling_floats_layout_unchanged() {
    let (doc, styles, _outer, f2) = build(false);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let f2_box = find_box(&result.root, f2).expect("f2 布局盒存在");
    assert_eq!(
        (f2_box.x, f2_box.y),
        (0.0, 0.0),
        "flat 形态现状：float2 从 y=0 落位（挂账，Phase 1 带集收集后修复）"
    );
}

/// floats-rule3-outside-left-001 结构：root float 500 → [R float:right 50×300，
/// wrapper mr:100（内容 0..400）→ C float:left `target_width`×10]。
fn rule3_left_case(target_width: f64) -> (Document, HashMap<NodeId, ComputedStyle>, NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let rf = doc.create_element("div");
    doc.append_child(body, rf).unwrap();
    let mut styles = HashMap::new();
    styles.insert(rf, block_float(FloatValue::Left, 500.0, 500.0));

    let r = doc.create_element("div");
    doc.append_child(rf, r).unwrap();
    styles.insert(r, block_float(FloatValue::Right, 50.0, 300.0));

    let w = doc.create_element("div");
    doc.append_child(rf, w).unwrap();
    styles.insert(
        w,
        ComputedStyle {
            margin_right: LengthValue::Px(100.0),
            ..block(0.0, 0.0)
        },
    );

    let c = doc.create_element("div");
    doc.append_child(w, c).unwrap();
    styles.insert(c, block_float(FloatValue::Left, target_width, 10.0));
    (doc, styles, c)
}

/// rule3-outside-left-001：规则 3——C 无相交左系带，右缘可溢出容器右缘
///（425 > 400），只需避开 R 带（450..500）→ 落 (0,0) 不下移。
#[test]
fn rule3_left_float_overflows_container_without_left_neighbor() {
    let (doc, styles, c) = rule3_left_case(425.0);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let c_box = find_box(&result.root, c).expect("目标 float 布局盒存在");
    assert_eq!(
        (c_box.x, c_box.y),
        (0.0, 0.0),
        "425 宽左 float 应在 y=0 溢出 400 宽容器右缘"
    );
}

/// rule3-outside-left-002 同构（475 宽）：右缘溢出会撞 R 带（450..500），
/// 须下移至 R 带（高 300）之下 → (0,300)。
#[test]
fn rule3_left_float_descends_when_right_band_blocks() {
    let (doc, styles, c) = rule3_left_case(475.0);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let c_box = find_box(&result.root, c).expect("目标 float 布局盒存在");
    assert_eq!(
        (c_box.x, c_box.y),
        (0.0, 300.0),
        "475 宽左 float 避不开右系带应下移到 y=300"
    );
}

/// floats-rule7-outside-left-001 结构：root float 500 → [A float:left 50×300，
/// wrapper ml:100（内容 0..400）→ C float:left 425×10]。规则 2：存在相交左系带
/// （A 带）时 C 右缘不得越过容器右缘（400），425 放不下 → 下移至 A 带之下 (0,300)。
#[test]
fn rule7_left_float_descends_below_prior_left_float() {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    let rf = doc.create_element("div");
    doc.append_child(body, rf).unwrap();
    let mut styles = HashMap::new();
    styles.insert(rf, block_float(FloatValue::Left, 500.0, 500.0));

    let a = doc.create_element("div");
    doc.append_child(rf, a).unwrap();
    styles.insert(a, block_float(FloatValue::Left, 50.0, 300.0));

    let w = doc.create_element("div");
    doc.append_child(rf, w).unwrap();
    styles.insert(
        w,
        ComputedStyle {
            margin_left: LengthValue::Px(100.0),
            ..block(0.0, 0.0)
        },
    );

    let c = doc.create_element("div");
    doc.append_child(w, c).unwrap();
    styles.insert(c, block_float(FloatValue::Left, 425.0, 10.0));

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let c_box = find_box(&result.root, c).expect("目标 float 布局盒存在");
    assert_eq!(
        (c_box.x, c_box.y),
        (0.0, 300.0),
        "425 宽左 float 在相交左系带下应下移（规则 2 容器右缘钳制）"
    );
}
