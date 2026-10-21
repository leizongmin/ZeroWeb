//! R5218：含 inline-flex/inline-grid 子的块容器高度按行盒合成（CSS2 §9.2.1 × §10.6.3）。
//!
//! engine.rs 的 `LayoutBox.is_block_level` 把 InlineFlex/InlineGrid 列为块级（taffy
//! Flex/Grid 映射视角），`remeasure_inline_only_containers` 的 inline-only 门（入口
//! `has_inline_children` / 收缩臂 `has_block_children`）据此把含这些子的块容器排除在
//! IFC 重测外 → 容器高度保留 taffy 块堆叠累加值（探针实证：2 个 40×20 inline-flex
//! 子同行排布、容器高却 40 = 2×20）。修后两门按 display 直判豁免 inline-flex/
//! inline-grid 子（与 postprocess `adjust_inline_block_positions` 的 ib_indices 判据
//! 同源），容器高收缩到 IFC 行盒高。InlineTable 不入豁免（表格管线专道，
//! margin-applies-to-014 回归实证）。

use super::*;
use zero_css_parser::values::{DisplayValue, LengthValue};
use zero_style_system::ComputedStyle;

/// 搭建 block 容器（width=300，无显式高）+ N 个显式尺寸原子行内级子（40×20）场景。
fn atomic_row_case(displays: &[DisplayValue]) -> (LayoutBox, LayoutEngine, Vec<zero_dom::NodeId>) {
    let (mut doc, body) = make_doc_with_body();
    let container = doc.create_element("div");
    doc.append_child(body, container).unwrap();

    let mut styles = HashMap::new();
    let mut cont = ComputedStyle::default();
    cont.display = DisplayValue::Block;
    cont.width = LengthValue::Px(300.0);
    styles.insert(container, cont);

    let mut ids = Vec::new();
    for display in displays {
        let child = doc.create_element("div");
        doc.append_child(container, child).unwrap();
        let mut st = ComputedStyle::default();
        st.display = display.clone();
        st.width = LengthValue::Px(40.0);
        st.height = LengthValue::Px(20.0);
        styles.insert(child, st);
        ids.push(child);
    }

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let container_box = find_child_by_node_id(&result.root, container).expect("container found");
    (container_box.clone(), engine, ids)
}

/// 纯 inline-flex 行：两子同行（x 递增、y 同行），容器高 = 单盒高 20（非 taffy 块
/// 堆叠累加 40）。修前签名：x 同行（IFC 定位 pass）而容器高 40——位置/高度两条
/// 路径判据不一致。
#[test]
fn r5218_inline_flex_children_row_height_not_stacked() {
    let (container_box, _, ids) = atomic_row_case(&[DisplayValue::InlineFlex, DisplayValue::InlineFlex]);
    assert!(
        (container_box.height - 20.0).abs() < 0.5,
        "容器高应为行盒高 20（修前 taffy 块堆叠 40），实际 {}",
        container_box.height
    );
    let b0 = find_child_by_node_id(&container_box, ids[0]).expect("child0 found");
    let b1 = find_child_by_node_id(&container_box, ids[1]).expect("child1 found");
    assert!(
        (b0.y - b1.y).abs() < 0.5,
        "两子应同行（y 相等），实际 {} vs {}",
        b0.y,
        b1.y
    );
    assert!(b1.x > b0.x + 39.0, "两子应水平并排，实际 x {} vs {}", b0.x, b1.x);
}

/// 混合行（inline-block + inline-flex）：容器高同样收缩到行盒高 20——收缩臂
/// `has_block_children` 不得把 inline-flex 子计作块级子拒绝收缩。
#[test]
fn r5218_mixed_inline_block_inline_flex_row_height_not_stacked() {
    let (container_box, _, _) = atomic_row_case(&[DisplayValue::InlineBlock, DisplayValue::InlineFlex]);
    assert!(
        (container_box.height - 20.0).abs() < 0.5,
        "混合行容器高应为行盒高 20（修前 40），实际 {}",
        container_box.height
    );
}

/// inline-grid 同族豁免：纯 inline-grid 行容器高 = 行盒高 20。
#[test]
fn r5218_inline_grid_children_row_height_not_stacked() {
    let (container_box, _, _) = atomic_row_case(&[DisplayValue::InlineGrid, DisplayValue::InlineGrid]);
    assert!(
        (container_box.height - 20.0).abs() < 0.5,
        "inline-grid 行容器高应为行盒高 20（修前 40），实际 {}",
        container_box.height
    );
}

/// InlineTable 子不入豁免（表格管线专道）：容器高维持块堆叠口径 40，
/// margin-applies-to-014（inline-table + absolute wrapper）依赖现状。
#[test]
fn r5218_inline_table_child_keeps_block_stacked_height() {
    let (container_box, _, _) = atomic_row_case(&[DisplayValue::InlineTable, DisplayValue::InlineTable]);
    assert!(
        (container_box.height - 40.0).abs() < 0.5,
        "inline-table 子容器高应维持块堆叠 40（不入豁免），实际 {}",
        container_box.height
    );
}
