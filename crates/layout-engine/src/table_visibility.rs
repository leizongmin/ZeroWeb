//! visibility:collapse 的表格行折叠检测。
//!
//! CSS Tables §4.1：table-row 上 `visibility:collapse` 的行高度为 0，
//! 不参与布局，且不贡献与相邻行之间的 border-spacing。本模块检测哪些行被折叠，
//! 与列折叠（`table.rs::detect_collapsed_columns`）对称。

use std::collections::HashMap;

use zero_css_parser::values::VisibilityValue;
use zero_dom::NodeId;
use zero_style_system::ComputedStyle;

use crate::table::{TableRow, get_row_box};
use crate::types::LayoutBox;

/// 检测 table 中被 `visibility:collapse` 折叠的行。
///
/// 遍历 `grid_rows`，对每个非匿名 table-row 取其 LayoutBox 的 computed style，
/// 检查 `visibility` 是否为 `Collapse`。匿名行（直接 table-cell 子元素生成的行，
/// 无 table-row 元素）不可折叠——它们没有可设置 visibility 的元素。
///
/// 返回与 `grid_rows` 等长的布尔向量，`true` 表示该行被折叠（高度应为 0）。
pub(crate) fn detect_collapsed_rows(
    table_box: &LayoutBox,
    grid_rows: &[TableRow],
    styles: &HashMap<NodeId, ComputedStyle>,
) -> Vec<bool> {
    grid_rows
        .iter()
        .map(|row| {
            if row.is_anonymous {
                return false;
            }
            get_row_box(table_box, row)
                .and_then(|rb| rb.node_id)
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.visibility, VisibilityValue::Collapse))
        })
        .collect()
}
