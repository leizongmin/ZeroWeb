//! R5208（CSS2 §17.2.1「blocks divide tables」+ §17.5 表格几何回写边界）：
//! 块级分隔符把连续孤立 table-internal 兄弟切成**多个独立匿名表**——分隔符后
//! 的孤立 row 形成第二个匿名表根，须停在分隔符之下（正常流堆叠），而非被
//! position_cells 的网格偏移覆写拉回 y=0 与前表重叠。两处修：
//! 1. `position_cells`：row_box 即 table_box 自身（孤儿 retrofit，`row.is_anonymous
//!    && row_group_index.is_none()`）只更新宽高、保留流位置；
//! 2. `adjust_table_layout_inner` 普通 else 分支：块子内部匿名 table 包装使
//!    taffy 虚高塌缩（54→18）时，按**收缩**差位移后续兄弟（父高 definite Px
//!    不回填）。driving: table-anonymous-objects-081/082/083/084（修前
//!    2.95/3.79/1.36/1.84% 红，修后全 0.00%），corpus 定谳 1003→999 净 −4。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

fn layout(body: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let html = format!("<html><body style=\"margin:0;font-family:monospace;white-space:nowrap\">{body}</body></html>");
    let doc = zero_dom::parse_html(&html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

fn find_box(root: &LayoutBox, node_id: zero_dom::NodeId) -> Option<&LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 083 形态锚点：row / **block 分隔符** / row 三兄弟。第三个 row 的匿名表根
/// 必须整体位于分隔符之下（y = row1 底 + 分隔符高），与前表不重叠（修前
/// position_cells 把网格 local_y=0 写进其流位置槽 → Row 333 压 Row 1）。
#[test]
fn r5208_orphan_row_after_block_divider_stacks_below_divider() {
    let (doc, result) = layout(
        "<div style=\"color:red\">\
         <span id=\"row1\" style=\"display:table-row\">\
         <span style=\"display:table-cell\">Row 1, Col 1</span>\
         <span style=\"display:table-cell\">Row 1, Col 2</span>\
         <span style=\"display:table-cell\">Row 1, Col 3</span>\
         </span>\
         <span id=\"divider\" style=\"display:block\">\
         <span style=\"display:table-cell\">Row 22, Col 1</span>\
         <span style=\"display:table-cell\">Row 22, Col 2</span>\
         <span style=\"display:table-cell\">Row 22, Col 3</span>\
         </span>\
         <span id=\"row3\" style=\"display:table-row\">\
         <span style=\"display:table-cell\">Row 333, Col 1</span>\
         <span style=\"display:table-cell\">Row 333, Col 2</span>\
         <span style=\"display:table-cell\">Row 333, Col 3</span>\
         </span>\
         </div>",
    );
    let row1 = find_box(&result.root, doc.get_element_by_id("row1").expect("row1")).expect("row1 匿名表根");
    let divider = find_box(&result.root, doc.get_element_by_id("divider").expect("divider")).expect("分隔符盒");
    let row3 = find_box(&result.root, doc.get_element_by_id("row3").expect("row3")).expect("row3 匿名表根");
    assert!(
        row1.is_anon_table_root && row3.is_anon_table_root,
        "孤立 row 应为匿名表根"
    );
    assert!(
        row3.y >= divider.y + divider.height - 0.5,
        "row3 应在分隔符之下（≥ {}），实得 y={}（修前被覆写回 0 与 row1 重叠）",
        divider.y + divider.height,
        row3.y
    );
    assert!(
        (row3.y - (row1.y + row1.height + divider.height)).abs() < 0.5,
        "row3 应紧贴 row1 底 + 分隔符高，row1.h={} divider.h={} row3.y={}",
        row1.height,
        divider.height,
        row3.y
    );
}

/// 分隔符内连续 cell run 合并为独立匿名表后，taffy 虚高（cells 各自成块竖排
/// 54）塌缩为真实表高（单行 18）——后续兄弟须随收缩差上移（083 的 72→36 分量）。
#[test]
fn r5208_divider_height_shrink_propagates_to_following_sibling() {
    let (doc, result) = layout(
        "<div>\
         <span style=\"display:table-row\">\
         <span style=\"display:table-cell\">Row 1</span>\
         </span>\
         <span id=\"divider\" style=\"display:block\">\
         <span style=\"display:table-cell\">A</span>\
         <span style=\"display:table-cell\">B</span>\
         </span>\
         <span id=\"row3\" style=\"display:table-row\">\
         <span style=\"display:table-cell\">Row 3</span>\
         </span>\
         </div>",
    );
    let divider = find_box(&result.root, doc.get_element_by_id("divider").expect("divider")).expect("分隔符盒");
    let row3 = find_box(&result.root, doc.get_element_by_id("row3").expect("row3")).expect("row3 匿名表根");
    // 单行行高量级：分隔符塌缩为单行（< 2×行高），row3 不得停留在两倍行高之外。
    assert!(
        divider.height < row3.height * 2.0,
        "block 分隔符内 cell run 应塌缩为单行表高 {}（taffy 虚高为逐 cell 块叠）",
        divider.height
    );
    assert!(
        (row3.y - (divider.y + divider.height)).abs() < 0.5,
        "row3 应紧贴收缩后的分隔符底 {}，实得 y={}",
        divider.y + divider.height,
        row3.y
    );
}
