//! R5209（CSS2 §17.2.1：table-row-group 的非 table-row 子 → 匿名 cell 包裹）：
//! 孤立行组（retrofit 匿名表根）内夹在两真行之间的裸文本须合成匿名 cell、与
//! 裸 cell 合成中段匿名行，且中段 cell 自携行 y（不再压回首行）。三处修：
//! 1. `wrap_stray_text_runs_in_row_group`（table_anon_cell.rs）：行组直系 boxless
//!    文本 run 合成空匿名 cell（R5169 同款借 node_id + fragment 圈定）；
//! 2. `build_grid` is_orphan cell 臂按 is_anon_table_cell 旗标收网格；
//! 3. `position_cells`：孤立 retrofit 匿名行的 cell 自携 local_y（非首行不再
//!    与首行同 y 重叠）。
//! 行组级 paint_text 直绘抑制（paint 侧，engine/text.rs）由常驻 reftest
//! table-anonymous-objects-161~166 守护（修前 1.29-1.84% 红，修后全 0.00%），
//! corpus 定谳 999→993 净 −6。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout(body: &str) -> crate::engine::LayoutResult {
    let html = format!("<html><body style=\"margin:0;font-family:monospace;white-space:nowrap\">{body}</body></html>");
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

fn find_anon_table_root(root: &LayoutBox) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.is_anon_table_root && b.node_id.is_some() {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 161 变异后形态：row-group 内 [row1][裸文本 "Row 22, Col 1"][cell Col2]
/// [cell Col3][row333]。裸文本合成匿名 cell 落 col 0，与两裸 cell 合成中段
/// 匿名行（y = 首行高），row333 仍在其下（修前：文本整段丢失 + 中段 cell
/// 压回 y=0 与 row1 重叠）。
#[test]
fn r5209_row_group_stray_text_becomes_anon_cell_in_middle_row() {
    let result = layout(
        "<div style=\"font-size:2em\">\
         <span style=\"display:table-row-group\">\
         <span style=\"display:table-row\">\
         <span style=\"display:table-cell\">Row 1, Col 1</span>\
         <span style=\"display:table-cell\">Row 1, Col 2</span>\
         <span style=\"display:table-cell\">Row 1, Col 3</span>\
         </span>\
         Row 22, Col 1\
         <span style=\"display:table-cell\">Row 22, Col 2</span>\
         <span style=\"display:table-cell\">Row 22, Col 3</span>\
         <span style=\"display:table-row\">\
         <span style=\"display:table-cell\">Row 333, Col 1</span>\
         <span style=\"display:table-cell\">Row 333, Col 2</span>\
         <span style=\"display:table-cell\">Row 333, Col 3</span>\
         </span>\
         </span>\
         </div>",
    );
    let table = find_anon_table_root(&result.root).expect("行组 retrofit 匿名表根");
    // row1 盒高 = 首行行高；中段匿名行 cell 的 y 带应与 row1 底对齐。
    let row1 = table
        .children
        .iter()
        .find(|c| !c.is_anon_table_cell && c.height > 10.0)
        .expect("row1 盒");
    let mid_band = (row1.y + row1.height - 6.0, row1.y + row1.height + 6.0);
    let mid_cells: Vec<_> = table
        .children
        .iter()
        .filter(|c| c.y > mid_band.0 && c.y < mid_band.1)
        .collect();
    assert!(
        mid_cells.len() == 3,
        "中段匿名行应有 3 个 cell（含合成 cell），实得 {}：{:?}",
        mid_cells.len(),
        mid_cells
            .iter()
            .map(|c| (c.x, c.y, c.is_anon_table_cell))
            .collect::<Vec<_>>()
    );
    let cells_x: Vec<f32> = mid_cells.iter().map(|c| c.x).collect();
    assert!(
        cells_x[0] < cells_x[1] && cells_x[1] < cells_x[2],
        "中段三 cell 应按列序水平排列，实得 x={:?}",
        cells_x
    );
    assert!(
        mid_cells[0].is_anon_table_cell,
        "col 0 应为裸文本合成的匿名 cell（修前文本整段丢失）"
    );
}

/// 行组内多匿名行不再互相重叠：第二个匿名行 cell 的 y 必须等于首行高
/// （修前 cell_carries_row_y 门漏掉孤立 retrofit 分支，全部压回 y=0）。
#[test]
fn r5209_orphan_row_group_anon_row_cells_carry_row_y() {
    let result = layout(
        "<div style=\"font-size:2em\">\
         <span style=\"display:table-row-group\">\
         <span style=\"display:table-row\">\
         <span style=\"display:table-cell\">Row 1, Col 1</span>\
         </span>\
         <span style=\"display:table-cell\">Row 22, Col 1</span>\
         <span style=\"display:table-cell\">Row 22, Col 2</span>\
         <span style=\"display:table-row\">\
         <span style=\"display:table-cell\">Row 333, Col 1</span>\
         </span>\
         </span>\
         </div>",
    );
    let table = find_anon_table_root(&result.root).expect("行组 retrofit 匿名表根");
    let row1 = table
        .children
        .iter()
        .find(|c| !c.is_anon_table_cell && c.height > 10.0)
        .expect("row1 盒");
    let bare_cells: Vec<_> = table
        .children
        .iter()
        .filter(|c| !c.is_anon_table_cell && c.y > row1.y + row1.height - 0.5)
        .collect();
    assert!(
        bare_cells.len() >= 2,
        "两个裸 cell 应落在 row1 之下（y ≥ {}），实得 {:?}（修前压回 y=0 与 row1 重叠）",
        row1.y + row1.height,
        table.children.iter().map(|c| (c.x, c.y)).collect::<Vec<_>>()
    );
}
