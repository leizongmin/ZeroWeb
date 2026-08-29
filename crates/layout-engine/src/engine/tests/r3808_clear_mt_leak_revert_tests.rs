//! R3808：float-then-clear 容器的 taffy 父子 margin 折叠 phantom 撤销（margin-collapse-142）。
//!
//! 现象：td > container（无 border/padding-top）> float + .clear（clear:left、mt 4em）。
//! taffy 0.12 把 .clear 的 mt 折叠进 container（§8.3.1 parent-child collapse），但
//! clearance 打断折叠链（CSS §9.5.2）——折叠本不应发生。taffy 折叠后 container 被
//! 多推下 64px，td 内露红底（142 test cell 24.34%）。chromium：container 顶在 cell
//! content 顶、.clear 落 float 底（mt 被清除吸收）。
//!
//! 修（两段）：① tree.rs 构树期对「float 前置 + 后随 cleared 块子 + 无 border/padding-top
//! 的普通块容器」设 taffy overflow:Hidden 抑制 taffy 内部父子折叠（沿 R3755 先例，
//! kill-switch `ZW_CLEAR_MT_TAFFY_GUARD=0`；R1318 案带 border-top 故不命中）；
//! ② float_positioning.rs clearance 臂的泄漏签名撤销（kill-switch
//! `ZW_CLEAR_MT_LEAK_REVERT=0`），兜住 taffy 抑制未覆盖的路径。

use super::*;

#[test]
fn r3808_clear_mt_leak_reverted() {
    let (mut doc, body) = make_doc_with_body();
    let table = doc.create_element("div");
    doc.append_child(body, table).unwrap();
    let row = doc.create_element("div");
    doc.append_child(table, row).unwrap();
    let cell = doc.create_element("div");
    doc.append_child(row, cell).unwrap();
    let container = doc.create_element("div");
    doc.append_child(cell, container).unwrap();
    let float_d = doc.create_element("div");
    doc.append_child(container, float_d).unwrap();
    let clear_d = doc.create_element("div");
    doc.append_child(container, clear_d).unwrap();

    let mut styles = HashMap::new();
    let mut table_style = ComputedStyle::default();
    table_style.display = zero_css_parser::values::DisplayValue::Table;
    styles.insert(table, table_style);
    let mut row_style = ComputedStyle::default();
    row_style.display = zero_css_parser::values::DisplayValue::TableRow;
    styles.insert(row, row_style);
    let mut cell_style = ComputedStyle::default();
    cell_style.display = zero_css_parser::values::DisplayValue::TableCell;
    cell_style.width = zero_css_parser::values::LengthValue::Px(134.0);
    styles.insert(cell, cell_style);
    let mut cont = ComputedStyle::default();
    cont.display = zero_css_parser::values::DisplayValue::Block;
    cont.width = zero_css_parser::values::LengthValue::Px(128.0);
    styles.insert(container, cont);

    let mut fl = ComputedStyle::default();
    fl.display = zero_css_parser::values::DisplayValue::Block;
    fl.float = zero_css_parser::values::FloatValue::Left;
    fl.width = zero_css_parser::values::LengthValue::Px(64.0);
    fl.height = zero_css_parser::values::LengthValue::Px(64.0);
    styles.insert(float_d, fl);

    let mut cl = ComputedStyle::default();
    cl.display = zero_css_parser::values::DisplayValue::Block;
    cl.clear = zero_css_parser::values::ClearValue::Left;
    cl.margin_top = zero_css_parser::values::LengthValue::Px(64.0);
    cl.margin_bottom = zero_css_parser::values::LengthValue::Px(64.0);
    cl.height = zero_css_parser::values::LengthValue::Px(64.0);
    styles.insert(clear_d, cl);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let c = find_child_by_node_id(&result.root, container).expect("container");
    let cl_box = find_child_by_node_id(&result.root, clear_d).expect("clear");
    // container 顶须在 cell content 顶（y=0、mt=0）——折叠 phantom 被撤销。
    assert!(
        (c.y - 0.0).abs() < 0.5 && (c.margin_top - 0.0).abs() < 0.5,
        "clearance 打断折叠后 container 不应携带折叠 phantom（y=0 mt=0），实际 y={} mt={}",
        c.y,
        c.margin_top
    );
    // .clear 落 float 底（mt 被清除吸收）。
    assert!(
        (cl_box.y - 64.0).abs() < 0.5,
        ".clear 应落 float 底（y=64），实际 y={}",
        cl_box.y
    );
}
