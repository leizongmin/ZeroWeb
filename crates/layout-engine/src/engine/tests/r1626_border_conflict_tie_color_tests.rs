//! R1626：collapsed-border cell-vs-cell 平局 + 异色时不推送颜色 override 回归测试。
//!
//! border-conflict-element-001a 谱系：4×4 `border-collapse:collapse` 表，每格 20px solid
//! border，相邻格共享边的颜色由 §17.6.2.1 冲突解析决定（同宽同 style 平局 → 最左/最上格
//! 颜色胜出）。旧 `resolve_collapsed_borders` 在 cell-vs-cell 内部边上，override 触发条件
//! 仅看 `(win_w - lo_w).abs() > 0.001 || win_style != lo_style`（宽/style 差），**不看颜色**。
//! 同宽同 style 异色平局 → 不推 override → 相邻两格各按自身颜色各画半宽（左半 A 色 / 右半
//! B 色），违反 §17.6.2.1（应整条共享边为胜出色）。且平局裁决 `>` 选了右/下格（应左/上格）。
//!
//! 修复两处（table_borders.rs）：① tie-break `>` → `>=`（左/上格平局胜出）；② override
//! 触发条件增加颜色维度（胜出色 != 败者自身色时也推 override）。本测试覆盖竖向（左|右）
//! 与横向（上|下）两条 cell-vs-cell 内部边。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_style_system::StyleSystem;

/// 递归收集表格内所有「叶子 cell 盒」（有 border 且无进一步 table-cell 子盒）。
fn collect_cells<'a>(box_node: &'a LayoutBox, out: &mut Vec<&'a LayoutBox>) {
    let has_cell_child = box_node
        .children
        .iter()
        .any(|c| c.border_top > 0.0 || c.border_left > 0.0);
    if (box_node.border_top > 0.0 || box_node.border_left > 0.0) && !has_cell_child {
        out.push(box_node);
    }
    for child in &box_node.children {
        collect_cells(child, out);
    }
}

/// green = Rgba(0,128,0,255) = 0x008000FF
const GREEN: u32 = 0x008000FF;

fn layout(html: &str) -> Vec<LayoutBox> {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut cells: Vec<&LayoutBox> = Vec::new();
    collect_cells(&result.root, &mut cells);
    cells.into_iter().cloned().collect()
}

/// 2 列 collapsed 表：左格右边 green，右格左边 red（均 20px solid，平局）。
/// 期望：共享竖边 → 最左（green）胜出 → 右格的左边被 override 成 green。
#[test]
fn r1626_vertical_tie_leftmost_color_overrides_right_cell() {
    let html = r#"<html><body style="margin:0"><table style="border-collapse: collapse"><tr>
        <td style="border: 20px solid green; width: 40px; height: 40px"></td>
        <td style="border: 20px solid red; width: 40px; height: 40px"></td>
    </tr></table></body></html>"#;
    let mut cells = layout(html);
    cells.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    assert!(cells.len() >= 2, "应至少有 2 个 cell 盒，实际 {}", cells.len());

    let right_cell = &cells[1];
    // 共享竖边：左格(green) 胜出 → 右格左边(side 3)应被 override 成 green
    let left_override = right_cell.collapsed_border_color_overrides[3];
    assert_eq!(
        left_override,
        Some(GREEN),
        "右格左边应被 override 成 green(0x008000FF，最左格胜出)，实际 {:?}",
        left_override
    );
}

/// 2 行 collapsed 表：上行底边 green，下行顶边 red（均 20px solid，平局）。
/// 期望：共享横边 → 最上（green）胜出 → 下行的顶边被 override 成 green。
#[test]
fn r1626_horizontal_tie_topmost_color_overrides_bottom_cell() {
    let html = r#"<html><body style="margin:0"><table style="border-collapse: collapse">
        <tr><td style="border: 20px solid green; width: 40px; height: 40px"></td></tr>
        <tr><td style="border: 20px solid red; width: 40px; height: 40px"></td></tr>
    </table></body></html>"#;
    let mut cells = layout(html);
    cells.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap_or(std::cmp::Ordering::Equal));
    assert!(cells.len() >= 2, "应至少有 2 个 cell 盒，实际 {}", cells.len());

    let bottom_cell = &cells[1];
    // 共享横边：上格(green) 胜出 → 下格顶边(side 0)应被 override 成 green
    let top_override = bottom_cell.collapsed_border_color_overrides[0];
    assert_eq!(
        top_override,
        Some(GREEN),
        "下格顶边应被 override 成 green(0x008000FF，最上格胜出)，实际 {:?}",
        top_override
    );
}

/// 宽度不平局时（旧路径已工作）：左格 30px green，右格 20px red → 左格更宽胜出，
/// 右格左边 override 成 green。此案在修复前后都应通过，守防回归。
#[test]
fn r1626_width_mismatch_winner_overrides_unchanged() {
    let html = r#"<html><body style="margin:0"><table style="border-collapse: collapse"><tr>
        <td style="border: 30px solid green; width: 40px; height: 40px"></td>
        <td style="border: 20px solid red; width: 40px; height: 40px"></td>
    </tr></table></body></html>"#;
    let mut cells = layout(html);
    cells.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    assert!(cells.len() >= 2);
    let right_cell = &cells[1];
    let left_override = right_cell.collapsed_border_color_overrides[3];
    assert_eq!(
        left_override,
        Some(GREEN),
        "宽度不平局时左格(green,30px)胜出，右格左边应 override 成 green"
    );
}
