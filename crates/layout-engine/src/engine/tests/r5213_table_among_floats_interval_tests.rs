//! R5213：table-among-floats 区间化 fit + 扩高上传基线 + 浮动 caption 豁免。
//!
//! 三个 load-bearing 锚（对应 floats-wrap-bfc-002/004/006 上游案与本轮新回归）：
//! 1. 区间化 fit：右 float 左缘给区间右界（旧 max_right 语义把右 float 右缘当阻挡物，
//!    混合 float 场景可用区间误判为 0 → 表错误推 below）。
//! 2. D 步扩高上传基线取 root.height（tr/tbody 中间层 content_height=0，用
//!    content_height 会把增量逐层放大，60→136→196 复合过冲）。
//! 3. x 归位臂仅非浮动 caption：float:right 的浮动 caption（float-applies-to-015）
//!    其 x 是合法 float 落位，归位到 margin_left 会把它从右缘拉回左缘。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_style_system::ComputedStyle;

/// 混合 float 容器：左 float 100×20 + 右 float 100×20 + BFC table（内容 80×20）。
fn build_mixed_float_table() -> (
    zero_dom::Document,
    HashMap<zero_dom::NodeId, ComputedStyle>,
    zero_dom::NodeId,
    zero_dom::NodeId,
) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(container, float_l).unwrap();
    let float_r = doc.create_element("div");
    doc.append_child(container, float_r).unwrap();
    let table = doc.create_element("div");
    doc.append_child(container, table).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();
    let cell_content = doc.create_element("div");
    doc.append_child(td, cell_content).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(20.0);
    styles.insert(float_l, fl);

    let mut fr = ComputedStyle::default();
    fr.display = DisplayValue::Block;
    fr.float = FloatValue::Right;
    fr.width = LengthValue::Px(100.0);
    fr.height = LengthValue::Px(20.0);
    styles.insert(float_r, fr);

    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    styles.insert(table, t);
    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);
    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    styles.insert(td, td_s);
    // 内容宽 80：区间 [left_edge=100, right_edge=200] gap=100 ≥ 80 → beside；
    // 旧 max_right 语义（右 float 右缘 300 作阻挡物）gap=0 → 错误推 below。
    let mut cc = ComputedStyle::default();
    cc.display = DisplayValue::Block;
    cc.width = LengthValue::Px(80.0);
    cc.height = LengthValue::Px(20.0);
    styles.insert(cell_content, cc);

    (doc, styles, container, table)
}

/// 区间化 fit：混合左/右 float 下 table beside 于左 float 右缘（x≈100），非推 below。
#[test]
fn r5213_interval_mixed_float_table_beside_not_below() {
    let (doc, styles, _container, table) = build_mixed_float_table();
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let table_box = find_child_by_node_id(&result.root, table).expect("table found");
    assert!(
        table_box.x >= 95.0 && table_box.x <= 105.0,
        "table 应 beside 左 float 右缘（x≈100），实际 x={}",
        table_box.x
    );
    assert!(
        table_box.y < 5.0,
        "table 应 beside（y≈0），旧 max_right 语义误判无空间推 below（y≈20），实际 y={}",
        table_box.y
    );
    // beside 填区间宽：[100, 200] gap=100。
    assert!(
        table_box.width >= 95.0 && table_box.width <= 105.0,
        "beside 表应填区间宽 ≈100，实际 {}",
        table_box.width
    );
}

/// 外表(auto 高) td 内：float 100×50 + 内表（内容 250×60）→ 内表推 below float，
/// D 步扩高 cell +50 上传行/行组/外表：外表高 ≈110，非 content_height 基线的逐层
/// 放大值（≥170）。
#[test]
fn r5213_pushed_table_grows_outer_table_height() {
    let (mut doc, body) = make_doc_with_body();
    let outer = doc.create_element("div");
    doc.append_child(body, outer).unwrap();
    let o_tbody = doc.create_element("div");
    doc.append_child(outer, o_tbody).unwrap();
    let o_tr = doc.create_element("div");
    doc.append_child(o_tbody, o_tr).unwrap();
    let o_td = doc.create_element("div");
    doc.append_child(o_tr, o_td).unwrap();
    let float_l = doc.create_element("div");
    doc.append_child(o_td, float_l).unwrap();
    let inner = doc.create_element("div");
    doc.append_child(o_td, inner).unwrap();
    let i_tr = doc.create_element("div");
    doc.append_child(inner, i_tr).unwrap();
    let i_td = doc.create_element("div");
    doc.append_child(i_tr, i_td).unwrap();
    let cell_content = doc.create_element("div");
    doc.append_child(i_td, cell_content).unwrap();

    let mut styles = HashMap::new();
    let mut outer_s = ComputedStyle::default();
    outer_s.display = DisplayValue::Table;
    outer_s.width = LengthValue::Px(300.0);
    styles.insert(outer, outer_s);
    for id in [o_tbody, o_tr] {
        let mut s = ComputedStyle::default();
        s.display = if id == o_tbody {
            DisplayValue::TableRowGroup
        } else {
            DisplayValue::TableRow
        };
        styles.insert(id, s);
    }
    let mut o_td_s = ComputedStyle::default();
    o_td_s.display = DisplayValue::TableCell;
    styles.insert(o_td, o_td_s);
    let mut fl = ComputedStyle::default();
    fl.display = DisplayValue::Block;
    fl.float = FloatValue::Left;
    fl.width = LengthValue::Px(100.0);
    fl.height = LengthValue::Px(50.0);
    styles.insert(float_l, fl);
    let mut inner_s = ComputedStyle::default();
    inner_s.display = DisplayValue::Table;
    styles.insert(inner, inner_s);
    let mut i_tr_s = ComputedStyle::default();
    i_tr_s.display = DisplayValue::TableRow;
    styles.insert(i_tr, i_tr_s);
    let mut i_td_s = ComputedStyle::default();
    i_td_s.display = DisplayValue::TableCell;
    styles.insert(i_td, i_td_s);
    // 内容 250×60：250 > beside gap 200 → 推 below float（y=50）→ 内表底 110，
    // D 步扩高外表 td（60→110）并沿 tr/tbody/outer 上传。
    let mut cc = ComputedStyle::default();
    cc.display = DisplayValue::Block;
    cc.width = LengthValue::Px(250.0);
    cc.height = LengthValue::Px(60.0);
    styles.insert(cell_content, cc);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let outer_box = find_child_by_node_id(&result.root, outer).expect("outer found");
    assert!(
        outer_box.height >= 105.0 && outer_box.height <= 120.0,
        "外表高应含推下内表（≈110），实际 {}",
        outer_box.height
    );
    assert!(
        outer_box.height < 150.0,
        "扩高上传基线须取 root.height；content_height 基线会逐层放大（≥170 复合过冲），实际 {}",
        outer_box.height
    );
}

/// 浮动 caption（float:right 96×96）x 保持 float 落位（右缘），不被归位臂拉回左缘。
#[test]
fn r5213_floating_caption_x_preserved() {
    let (mut doc, body) = make_doc_with_body();
    let table = doc.create_element("div");
    doc.append_child(body, table).unwrap();
    let caption = doc.create_element("div");
    doc.append_child(table, caption).unwrap();
    let tr = doc.create_element("div");
    doc.append_child(table, tr).unwrap();
    let td = doc.create_element("div");
    doc.append_child(tr, td).unwrap();

    let mut styles = HashMap::new();
    let mut t = ComputedStyle::default();
    t.display = DisplayValue::Table;
    styles.insert(table, t);
    let mut cap = ComputedStyle::default();
    cap.display = DisplayValue::TableCaption;
    cap.caption_side = zero_style_system::property::CaptionSideValue::Bottom;
    cap.float = FloatValue::Right;
    cap.width = LengthValue::Px(96.0);
    cap.height = LengthValue::Px(96.0);
    styles.insert(caption, cap);
    let mut tr_s = ComputedStyle::default();
    tr_s.display = DisplayValue::TableRow;
    styles.insert(tr, tr_s);
    let mut td_s = ComputedStyle::default();
    td_s.display = DisplayValue::TableCell;
    styles.insert(td, td_s);
    let mut cc = ComputedStyle::default();
    cc.display = DisplayValue::Block;
    cc.width = LengthValue::Px(50.0);
    cc.height = LengthValue::Px(20.0);
    styles.insert(td, cc);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let cap_box = find_child_by_node_id(&result.root, caption).expect("caption found");
    assert!(
        cap_box.x > 600.0,
        "float:right caption 应保持 float 落位（x≈704），归位臂误拉回左缘则 x≈0，实际 x={}",
        cap_box.x
    );
}
