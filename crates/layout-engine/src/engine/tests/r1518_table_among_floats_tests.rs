//! R1518d/R1518 V2 回归测试：table-among-floats scoped iterative fix。
//!
//! 背景（table-among-floats-001）：BFC 容器（overflow:hidden width:200）含
//!   - float1 75×100 float:left clear:left
//!   - float2 125×100 float:left clear:left
//!   - table > tr > td 内含 inner-float-A 75×100 + inner-float-B 75×100（均 float:left）
//!
//! 期望（CSS §9.5）：table shrink-to-fit 到 75 宽（inner float 堆叠 → 75×200）并被 §9.5
//! 推到 float 右侧（avoidance_x = max(75, 125) = 125），容器高度 = MAX(float 底 200,
//! table 底 200) = 200。
//!
//! 修复前症状（R1518d 定位）：step8 `adjust_table_layout` 把 shrink-to-fit 后仍堆在 float
//! 下方的 table 高度经 `reflow_siblings_after_table_height_change` +delta 到容器（200→300），
//! 容器下方露红；table 未 §9.5 推开重叠 float。本 pass（V2）scoped 修：A re-wrap inner
//! float + B 重算 table 高 + C 手动 §9.5 push + D 重算容器高度。

use super::*;
use zero_css_parser::values::{ClearValue, DisplayValue, FloatValue, LengthValue, OverflowValue};
use zero_style_system::ComputedStyle;

/// 构造 table-among-floats-001 核心结构：BFC 容器（overflow:hidden width:200）含
/// float1(75×100) + float2(125×100)（均 clear:left）+ table>tr>td 内含 2 个 75×100 float。
fn build_table_among_floats() -> (
    zero_dom::Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
    zero_dom::NodeId,
) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float1 = doc.create_element("div");
    doc.append_child(container, float1).unwrap();
    let float2 = doc.create_element("div");
    doc.append_child(container, float2).unwrap();
    let table = doc.create_element("div");
    doc.append_child(container, table).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();
    let inner_a = doc.create_element("div");
    doc.append_child(td, inner_a).unwrap();
    let inner_b = doc.create_element("div");
    doc.append_child(td, inner_b).unwrap();

    let mut styles = HashMap::new();

    // BFC 容器：overflow:hidden width:200（auto height）
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.overflow_x = OverflowValue::Hidden;
    cont.overflow_y = OverflowValue::Hidden;
    cont.width = LengthValue::Px(200.0);
    styles.insert(container, cont);

    let mk_float = |w: f64, h: f64| {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.float = FloatValue::Left;
        s.clear = ClearValue::Left;
        s.width = LengthValue::Px(w);
        s.height = LengthValue::Px(h);
        s
    };
    styles.insert(float1, mk_float(75.0, 100.0));
    styles.insert(float2, mk_float(125.0, 100.0));

    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    styles.insert(table, t);
    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);
    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    styles.insert(td, td_s);
    styles.insert(inner_a, mk_float(75.0, 100.0));
    // inner float 不需 clear（堆叠由 td 收窄驱动）
    styles.insert(inner_b, mk_float(75.0, 100.0));

    (doc, styles, container, table)
}

/// R1518d：table 被 §9.5 推到 float 右侧（avoidance_x ≈ 125），容器高度收回到 ≈200
///（修复前 table 堆叠 float 下方致容器 300）。load-bearing：关闭 fix（env=0）则
/// table.x ≈ 0 且容器 ≈300，断言失败。
#[test]
fn r1518_table_among_floats_pushed_aside_and_container_collapsed() {
    let (doc, styles, container, table) = build_table_among_floats();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let container_box = find_child_by_node_id(&result.root, container).expect("container found");
    let table_box = find_child_by_node_id(&result.root, table).expect("table found");

    // table 应被 §9.5 推到 float2 右侧（avoidance_x = max(75, 125) = 125）。
    assert!(
        table_box.x >= 120.0,
        "table should be §9.5-pushed beside floats (x≈125), got x={}",
        table_box.x
    );
    // table 应收缩到能放进 float 右侧空间（≤ 200-125=75）。
    assert!(
        table_box.width <= 80.0,
        "table width should shrink-to-fit beside floats (≤75), got {}",
        table_box.width
    );
    // 容器高度应收回 ≈200（修复前 reflow_siblings 把堆叠 table 高度 +delta 致 300）。
    assert!(
        container_box.height <= 230.0,
        "container height should collapse to ~200 (not stack table below floats), got {}",
        container_box.height
    );
    assert!(
        container_box.height >= 180.0,
        "container height should still contain floats (~200), got {}",
        container_box.height
    );
}

/// R1518d 守护：clear 的 table（clear-applies-to-013）不应被 §9.5 推到 float 右侧，
/// 应由 clear 逻辑清到 float 下方。验证 is_cleared 守卫生效。
#[test]
fn r1518_cleared_table_not_pushed_aside() {
    let (doc, mut styles, _container, table) = build_table_among_floats();
    // 给 table 加 clear:both —— 它应清到 float 下方，不 §9.5 推开。
    if let Some(s) = styles.get_mut(&table) {
        s.clear = ClearValue::Both;
    }
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table_box = find_child_by_node_id(&result.root, table).expect("table found");
    // clear:both 的 table 应清到 float1/float2 下方（y ≥ ~150），不应被推到 float 右侧（x≈125）。
    // 即使 §9.5 push 未触发，clear 也应使其 y 较大；关键是 x 不被推到 avoidance_x。
    let pushed_aside = table_box.x >= 120.0 && table_box.y < 50.0;
    assert!(
        !pushed_aside,
        "cleared table should NOT be §9.5-pushed beside floats, got table at ({},{})",
        table_box.x, table_box.y
    );
}

/// R1609：per-y §9.5 BFC-avoid（floats-wrap-bfc-006 谱系）。table 太宽放不进 natural_y 处
/// 最宽 float 右侧（fits=false）时，原 fits-only 逻辑不推 → table 卡顶部重叠 float。新 per-y
/// 逻辑从 natural_y 起按 float bottom 递进，把 table 推到首个整高避开 float 的 y。
///
/// 结构：BFC 容器（overflow:hidden width:300）含 float1(200×10) + float2(50×10)（均 clear:left）
/// 以及 table(width 150, td height 30)。table 150 放不进 float1(200) 右（剩 100）→ per-y 推过
/// float1 bottom 到 y=10，放得进 float2(50) 右（剩 250）→ x≈50。
/// load-bearing：关 fix（env=0）则 table 卡顶部 y≈0 重叠 float1，断言失败。
#[test]
fn r1609_wide_table_pushed_down_past_blocking_float() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float1 = doc.create_element("div");
    doc.append_child(container, float1).unwrap();
    let float2 = doc.create_element("div");
    doc.append_child(container, float2).unwrap();
    let table = doc.create_element("div");
    doc.append_child(container, table).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.overflow_x = OverflowValue::Hidden;
    cont.overflow_y = OverflowValue::Hidden;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    let mk_float = |w: f64, h: f64| {
        let mut s = ComputedStyle::default();
        s.display = DisplayValue::Block;
        s.float = FloatValue::Left;
        s.clear = ClearValue::Left;
        s.width = LengthValue::Px(w);
        s.height = LengthValue::Px(h);
        s
    };
    styles.insert(float1, mk_float(200.0, 10.0));
    styles.insert(float2, mk_float(50.0, 10.0));

    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    styles.insert(table, t);
    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);
    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    td_s.width = LengthValue::Px(150.0);
    td_s.height = LengthValue::Px(30.0);
    styles.insert(td, td_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let table_box = find_child_by_node_id(&result.root, table).expect("table found");

    // table(150) 放不进 float1(200) 右侧（剩 100 < 150）→ per-y 推到 float1 下方（y≥10），
    // 放得进 float2(50) 右侧（剩 250）→ x≈50。
    assert!(
        table_box.y >= 8.0,
        "wide table should be pushed DOWN past blocking float1 (y≈10), got y={}",
        table_box.y
    );
    assert!(
        (table_box.x - 50.0).abs() <= 8.0,
        "table should sit beside float2 (x≈50), got x={}",
        table_box.x
    );
}

/// R1613：auto-width BFC table 放进 float 右侧时填满可用 avoidance 宽度（非 shrink-to-fit
/// 内容宽）。floats-wrap-bfc-002-left-table 谱系：ref 用显式 width:200 的 div 模拟，test 的
/// auto table 须填 200（=container 300 − float 100）非内容宽。
/// 结构：BFC 容器（overflow:hidden width:300）含 float(100×100) + table(无显式宽，td 内容 50
/// 宽)。期望 table 在 float 右 x≈100、宽≈200（填满），非 50（shrink-to-fit）。
#[test]
fn r1613_beside_float_table_fills_available_width() {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float1 = doc.create_element("div");
    doc.append_child(container, float1).unwrap();
    let table = doc.create_element("div");
    doc.append_child(container, table).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.overflow_x = OverflowValue::Hidden;
    cont.overflow_y = OverflowValue::Hidden;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    let mut f = ComputedStyle::default();
    f.display = DisplayValue::Block;
    f.float = FloatValue::Left;
    f.width = LengthValue::Px(100.0);
    f.height = LengthValue::Px(100.0);
    styles.insert(float1, f);

    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    styles.insert(table, t);
    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);
    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    td_s.width = LengthValue::Px(50.0); // 内容仅 50 宽（< 可用 200）
    td_s.height = LengthValue::Px(100.0);
    styles.insert(td, td_s);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let table_box = find_child_by_node_id(&result.root, table).expect("table found");

    // table 放进 float(100) 右侧 → 填满可用宽 200（=300−100），非 shrink-to-fit 内容 50。
    assert!(
        (table_box.x - 100.0).abs() <= 8.0,
        "table should sit beside float (x≈100), got x={}",
        table_box.x
    );
    assert!(
        table_box.width >= 190.0,
        "beside-float auto table should FILL available width (~200, not shrink-to-fit 50), got {}",
        table_box.width
    );
}
