//! R5318 替换叶盒 cell intrinsic 固有宽（css-sizing-3 §5.1 max-content + CSS2 §10.3.2）。
//!
//! td 内 `display:block` img（CSS 宽 auto、attr/解码固有尺寸）此前经 R4028 臂
//! `box_content_max_width` 叶盒回退（own_explicit=0 + text_w=0）测 0 → 列宽塌 0、
//! 整格消失（c42-ibx-pad-000 格2 2.60% 实证：control 格红底 + 图全缺席，表格总宽
//! 224 应 428）。`box_content_max_width_inner` 叶盒臂现对替换叶读第一趟固有宽
//!（taffy definite，替换盒主轴永不 fill-stretch，无拉伸伪影）。
//! kill-switch `ZW_REPLACED_LEAF_INTRINSIC=0`（进程级 OnceLock）：带此 env 跑本文件
//! 应回退旧行为使 `r5318_block_replaced_leaf_cell_intrinsic_is_natural_width` 失败——
//! 双向可判别。

use std::collections::HashMap;

use zero_css_parser::values::DisplayValue;
use zero_dom::Document;
use zero_style_system::ComputedStyle;

use crate::table::compute_cell_intrinsic_width;
use crate::types::LayoutBox;

#[test]
fn r5318_block_replaced_leaf_cell_intrinsic_is_natural_width() {
    let mut doc = Document::new();
    let root = doc.root();
    let table_id = doc.create_element("table");
    let cell_id = doc.create_element("td");
    let img_id = doc.create_element("img");
    let _ = doc.append_child(root, table_id);
    let _ = doc.append_child(table_id, cell_id);
    let _ = doc.append_child(cell_id, img_id);

    let mut styles = HashMap::new();
    let mut ts = ComputedStyle::default();
    ts.display = DisplayValue::Table;
    styles.insert(table_id, ts);
    let mut cs = ComputedStyle::default();
    cs.display = DisplayValue::TableCell;
    styles.insert(cell_id, cs);
    let mut is = ComputedStyle::default();
    // c42 形态：`img { display: block; }`（UA inline → 块级），CSS 宽 auto。
    is.display = DisplayValue::Block;
    styles.insert(img_id, is);

    // 第一趟 taffy：attr/解码固有尺寸已写为 definite 宽（tree.rs
    // apply_replaced_element_sizing），cell 测量时点读到的即固有值。
    let img_box = LayoutBox {
        node_id: Some(img_id),
        is_replaced: true,
        width: 210.0,
        height: 120.0,
        ..Default::default()
    };
    // p5 实测形态：taffy 对 cell 的 junk/0 宽（列宽未决时塌 0）。
    let cell_box = LayoutBox {
        node_id: Some(cell_id),
        width: 0.0,
        children: vec![img_box],
        ..Default::default()
    };

    let intrinsic = compute_cell_intrinsic_width(&cell_box, &styles, &doc, Default::default(), None);
    // 修复前 R4028 臂叶盒回退双 0 → intrinsic 塌 ~0（列宽 0、格消失）。
    assert!(
        intrinsic >= 209.5,
        "替换叶盒（display:block img, CSS 宽 auto）cell intrinsic 应取固有宽 210，got {}",
        intrinsic
    );
}
