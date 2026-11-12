//! R5510（CSS2 §17.5.3 行高 × R109 backfill delta-add 不变式）回归锚：
//! 单格态（行内唯一 cell 含 block-in-inline 拆分载体）行高在表格 pass 吃载体的
//! taffy 片段叶陈旧值 18（真范围 54 要等 engine 步骤 12/12.1 回填）——行/行组
//! 盒高由 position_cells / update_row_group_positions 直接写入而 content_height
//! 留 taffy 旧值，破坏 backfill_r109_anon_block_heights arm② 依赖的
//! height = content_height + frame 不变式：delta 叠加在已含陈旧行高的 height 上
//! → tr 60+18=78、行组/表/body 复利（chromium oracle：tr=td=60，见 R5509 锚
//! wpt-data 引用）。修复 = 表内盒高度作者写点补写 content_height = height
//! （水平 position_cells / update_row_group_positions + vertical 孪生
//! position_cells_vertical 同款），delta 收敛为真实欠计。
//!
//! 门控＝taffy content_height 与网格行高不一致（>0.5，≤0.5 补写为亚像素
//! no-op）∧ 行子树含 r109 拆分载体。无条件补写会改变 arm② 的 delta 叠加与
//! 兄弟位移——tao-087/088（畸形表生成 unwrapped Inline 子行，arm② 兄弟位移
//! 恰为其与参照对齐的承担者）corpus solo×3 双二进制确定性回归，故收窄到
//! r109 载体已证域（首轮 corpus A/B 定谳后改门控，见控制面 R5510）。
//!
//! 锚定结构不变量（行高 18 由 Legacy 度量给出，两态同源，断言取相对值）：
//! 单格态全链等高收敛、双行表行 1 落行 0 底（兄弟位移恰为真实增长）、双格态
//! 参照 R5509 锚不回归（彼处全链早已一致，补写为 no-op）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use std::collections::HashMap;
use zero_dom::NodeId;
use zero_style_system::{ComputedStyle, StyleSystem};

const STYLE: &str = r#"
  td { border: solid black; padding: 0; background: red; }
  .inline { display: inline; background: lime; }
  .block { display: block; background: green; }
"#;

/// block-in-inline 拆分载体内容（R5509 同款三片段形态）。
const CARRIER_CELL: &str = r#"<span class="inline">Line 1<span class="block">Line 2</span>Line 3</span>"#;

/// 布并返回（root 盒树, styles）。作者样式表须显式传入（R5509 探针假阳性教训：
/// 传 &[] 只有 UA 默认，.block{display:block} 不生效 → 拆分不触发、盒树形态失真）。
fn layout(html: &str) -> (std::sync::Arc<LayoutBox>, HashMap<NodeId, ComputedStyle>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(400.0, 600.0);
    let sheet = zero_css_parser::Parser::parse_stylesheet(STYLE);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut eng = LayoutEngine::new(400.0, 600.0);
    let r = eng.compute(&doc, &styles);
    (r.root, styles)
}

/// 按 display 下探首个匹配盒。
fn find_display<'a>(
    b: &'a LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    want: fn(&zero_style_system::property::DisplayValue) -> bool,
) -> Option<&'a LayoutBox> {
    if b.node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| want(&s.display))
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_display(c, styles, want))
}

fn find_all_display<'a>(
    b: &'a LayoutBox,
    styles: &HashMap<NodeId, ComputedStyle>,
    want: fn(&zero_style_system::property::DisplayValue) -> bool,
    out: &mut Vec<&'a LayoutBox>,
) {
    if b.node_id
        .and_then(|id| styles.get(&id))
        .is_some_and(|s| want(&s.display))
    {
        out.push(b);
    }
    for c in &b.children {
        find_all_display(c, styles, want, out);
    }
}

fn is_row(d: &zero_style_system::property::DisplayValue) -> bool {
    matches!(d, zero_style_system::property::DisplayValue::TableRow)
}
fn is_row_group(d: &zero_style_system::property::DisplayValue) -> bool {
    matches!(d, zero_style_system::property::DisplayValue::TableRowGroup)
}
fn is_table(d: &zero_style_system::property::DisplayValue) -> bool {
    matches!(d, zero_style_system::property::DisplayValue::Table)
}
fn is_cell(d: &zero_style_system::property::DisplayValue) -> bool {
    matches!(d, zero_style_system::property::DisplayValue::TableCell)
}

/// 单格态：全链（td→tr→行组→表）高度收敛一致——backfill delta 不再叠加陈旧行高。
#[test]
fn r5510_single_cell_chain_collapses_to_cell_height() {
    let html = format!(
        r#"<html><head><style>{STYLE}</style></head><body><table><tr><td>{CARRIER_CELL}</td></tr></table></body></html>"#
    );
    let (root, styles) = layout(&html);
    let td = find_display(&root, &styles, is_cell).expect("cell box must exist");
    let tr = find_display(&root, &styles, is_row).expect("row box must exist");
    let rg = find_display(&root, &styles, is_row_group).expect("row group box must exist");
    let table = find_display(&root, &styles, is_table).expect("table box must exist");
    // 修复前：tr = 18(陈旧行高) + 60(delta 叠加) = 78，行组/表复利 96（应 60）。
    assert!(
        (tr.height - td.height).abs() < 0.5,
        "row must equal cell height, got row={} cell={}",
        tr.height,
        td.height
    );
    assert!(
        (rg.height - td.height).abs() < 0.5,
        "row group must equal cell height, got rg={} cell={}",
        rg.height,
        td.height
    );
    assert!(
        (table.height - td.height).abs() < 0.5,
        "table must equal cell height, got table={} cell={}",
        table.height,
        td.height
    );
    // 行盒自身不变式成立（height = content_height，行无自身框）。
    assert!(
        (tr.content_height - tr.height).abs() < 0.5,
        "row content_height must mirror height, got ch={} h={}",
        tr.content_height,
        tr.height
    );
}

/// 双行表：行 0 含拆分载体、行 1 普通内容——行 1 兄弟位移恰为行 0 真实增长
/// （修复前 tr 增长 delta 含陈旧行高 18，行 1 被多推 18）。
#[test]
fn r5510_two_row_table_row1_sits_at_row0_bottom() {
    let html = format!(
        r#"<html><head><style>{STYLE}</style></head><body><table><tr><td>{CARRIER_CELL}</td></tr><tr><td><div>Row 2</div></td></tr></table></body></html>"#
    );
    let (root, styles) = layout(&html);
    let mut rows = Vec::new();
    find_all_display(&root, &styles, is_row, &mut rows);
    assert_eq!(rows.len(), 2, "two row boxes must exist");
    let (row0, row1) = (rows[0], rows[1]);
    // 行 1 y = 行 0 高（行 0 回填增长后兄弟位移精确，行间不重叠不空带）。
    assert!(
        (row1.y - row0.height).abs() < 0.5,
        "row1 must sit at row0 bottom, got row1.y={} row0.h={}",
        row1.y,
        row0.height
    );
    // 表高 = 两行之和（不膨胀）。
    let table = find_display(&root, &styles, is_table).expect("table box must exist");
    assert!(
        (table.height - row0.height - row1.height).abs() < 0.5,
        "table height must equal sum of rows, got table={} rows={}+{}",
        table.height,
        row0.height,
        row1.height
    );
}

/// 双格态（参照格在场）：行高由参照格真值主导，全链早已一致——补写为 no-op，
/// 防 R5510 修复扰动 R5509 绿面。
#[test]
fn r5510_two_cell_reference_row_unchanged() {
    let html = format!(
        r#"<html><head><style>{STYLE}</style></head><body><table><tr><td>{CARRIER_CELL}</td><td><div>Line 1</div><div>Line 2</div><div>Line 3</div></td></tr></table></body></html>"#
    );
    let (root, styles) = layout(&html);
    let td = find_display(&root, &styles, is_cell).expect("cell box must exist");
    let tr = find_display(&root, &styles, is_row).expect("row box must exist");
    let rg = find_display(&root, &styles, is_row_group).expect("row group box must exist");
    let table = find_display(&root, &styles, is_table).expect("table box must exist");
    assert!(
        (tr.height - td.height).abs() < 0.5
            && (rg.height - td.height).abs() < 0.5
            && (table.height - td.height).abs() < 0.5,
        "chain must stay consistent, got tr={} rg={} table={} td={}",
        tr.height,
        rg.height,
        table.height,
        td.height
    );
}
