//! R5242：float 收缩对 definite 宽 inline-table 子按行内级 Σ 求和（CSS2 §9.2.2 × §10.3.5）。
//!
//! engine 的 `LayoutBox.is_block_level` 含 InlineTable（块叠装配简化，postprocess.rs
//! R5011 注在案），float shrink 两处块叠口径（shrink_pure_text_floats 的
//! `has_block_or_replaced` 早退门 + adjust_float_positions 的 `content_child_widths`
//! 过滤）因此把 inline-table 当块级子：wrapper 有 inline-block 兄弟时不早退、落 R4920
//! Σ 臂，但块叠 max 又把 Σ 结果重收缩回 max(子)（修前实证 100→50）；step 10
//! adjust_inline_block_positions 按 IFC 忠实反映 50 宽 → 两原子换行（表落 y=150）。
//! 容器高同理被 6.5 收缩臂的 has_block_children 判块级（修前 200）。
//!
//! 修后四处修改共用 `float_sigma_eligible_inline_table` 谓词（CSS width definite 的
//! inline-table 才豁免；auto 宽维持旧块叠语义，Σ 贡献 0 的双向失真见谓词文档），
//! wrapper 收缩到行内级 Σ=100、两原子同行，容器高经 6.5 remeasure 的 ib_sizes 并入
//! InlineTable + 收缩臂豁免回收为行盒高 100。
//!
//! kill-switch：float 两处随既有 `ZW_FLOAT_INLINE_SUM=0`；inline_finalization 两处随
//! `ZW_INLINE_TABLE_IB_SIZING=0`。单测进程内不设 env（默认全 ON）。

use super::*;
use zero_css_parser::values::{DisplayValue, FloatValue, LengthValue};
use zero_style_system::ComputedStyle;

/// R5242 锚：float wrapper（无文本）内 inline-block 50×100 + definite 宽 inline-table
/// 50×100——wrapper 收缩到行内级 Σ=100×100，两原子同行 y、表 x=+50。
/// 修前签名：wrapper w=50（块叠 max 撤掉 Σ）、h=200（6.5 收缩臂判块级），表原子
/// 换行落 y=150。
#[test]
fn r5242_float_shrink_inline_table_sigma_sum() {
    let (mut doc, body) = make_doc_with_body();
    let wrapper = doc.create_element("div");
    doc.append_child(body, wrapper).unwrap();
    let ib = doc.create_element("div");
    doc.append_child(wrapper, ib).unwrap();
    let table = doc.create_element("table");
    doc.append_child(wrapper, table).unwrap();
    let td = doc.create_element("td");
    doc.append_child(table, td).unwrap();

    let mut styles = HashMap::new();

    let mut ws = ComputedStyle::default();
    ws.display = DisplayValue::Block;
    ws.float = FloatValue::Left;
    styles.insert(wrapper, ws);

    let mut ibs = ComputedStyle::default();
    ibs.display = DisplayValue::InlineBlock;
    ibs.width = LengthValue::Px(50.0);
    ibs.height = LengthValue::Px(100.0);
    styles.insert(ib, ibs);

    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::InlineTable;
    ts.width = LengthValue::Px(50.0);
    ts.height = LengthValue::Px(100.0);
    styles.insert(table, ts);

    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let wrapper_box = find_child_by_node_id(&result.root, wrapper).expect("wrapper found");

    assert!(
        (wrapper_box.width - 100.0).abs() < 0.5,
        "wrapper 应收缩到行内级 Σ=100（修前块叠 max=50），实际 {}",
        wrapper_box.width
    );
    assert!(
        (wrapper_box.height - 100.0).abs() < 0.5,
        "wrapper 高应为行盒高 100（修前块堆叠 200），实际 {}",
        wrapper_box.height
    );

    let ib_box = find_child_by_node_id(wrapper_box, ib).expect("inline-block found");
    let table_box = find_child_by_node_id(wrapper_box, table).expect("inline-table found");
    assert!(
        (table_box.x - ib_box.x - 50.0).abs() < 0.5,
        "inline-table 应紧随 inline-block 同行（x=+50，修前换行 x 回到行首），实际 dx={}",
        table_box.x - ib_box.x
    );
    assert!(
        (table_box.y - ib_box.y).abs() < 0.5,
        "两原子应同行 y（修前表换行落 y=+150），实际 dy={}",
        table_box.y - ib_box.y
    );
    assert!(
        (table_box.width - 50.0).abs() < 0.5 && (table_box.height - 100.0).abs() < 0.5,
        "inline-table 应保持 definite 尺寸 50×100，实际 {}×{}",
        table_box.width,
        table_box.height
    );
}
