//! R5509（CSS2 §17.5.3 valign × §9.2.1.1 匿名块拆分）回归锚：
//! block-in-inline-001/002 签名——双格表（block-in-inline 格 + 参照格）下
//! valign:middle 按载体盒高（taffy 片段叶陈旧值 18）求和得假 available=36 →
//! dy=18 内容下移露 td 底色，参照格随后被增长 delta 推下 18。修复：valign
//! 子盒求和对 r109 载体改用片段流范围（max(y+h)=真值 54）→ avail=0 不位移。
//!
//! 锚定结构不变量（行高 18 由 Legacy 度量给出，两态同源，断言取相对值）：
//! 两格 td 等高、block-in-inline 格的载体子盒 y=0（无 valign 位移）、行高=
//! 单元格高（无增长 delta 膨胀）。chromium oracle：两 td 等高 60px 内容顶齐
//!（tests/wpt-runner wpt-data css/CSS2/box-display/block-in-inline-001）。

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

const CELL1: &str = r#"<span class="inline">Line 1<span class="block">Line 2</span>Line 3</span>"#;
const CELL2: &str = r#"<div class="a">Line 1</div><div class="b">Line 2</div><div class="c">Line 3</div>"#;

/// 布并返回（root 盒树, styles）。
fn layout(html: &str) -> (std::sync::Arc<LayoutBox>, HashMap<NodeId, ComputedStyle>) {
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(400.0, 600.0);
    // 作者样式表须显式传入（管线走 collect_stylesheets 提取 <style> 后交
    // compute_styles；直接传 &[] 只有 UA 默认，.block{display:block} 不生效，
    // inline_has_block_child 恒 false——R5509 首轮探针假阳性教训）。
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

#[test]
fn r5509_bii_two_cells_carrier_not_shifted() {
    let html = format!(
        r#"<html><head><style>{STYLE}</style></head><body><table><tr><td>{CELL1}</td><td>{CELL2}</td></tr></table></body></html>"#
    );
    let (root, styles) = layout(&html);
    let tr = find_display(&root, &styles, |d| {
        matches!(d, zero_style_system::property::DisplayValue::TableRow)
    })
    .expect("table row box must exist");
    let tds: Vec<&LayoutBox> = tr
        .children
        .iter()
        .filter(|c| {
            c.node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.display, zero_style_system::property::DisplayValue::TableCell))
        })
        .collect();
    assert_eq!(tds.len(), 2, "two cells in the row");
    let (td1, td2) = (tds[0], tds[1]);
    // ① 两格等高（chromium oracle 均 60px）：valign 假 available 不再抬行高。
    assert!(
        (td1.height - td2.height).abs() < 0.5,
        "cells must be equal height, got td1={} td2={}",
        td1.height,
        td2.height
    );
    // ② 参照格不被增长 delta 推下（修复前 y=18）。
    assert!(
        td2.y.abs() < 0.5,
        "reference cell must stay at row top, got y={}",
        td2.y
    );
    // ③ block-in-inline 格的载体子盒 y=0（修复前 valign middle dy=18）。
    let carrier = td1
        .children
        .iter()
        .find(|c| c.is_r109_split)
        .expect("cell1 must contain an r109 split carrier");
    assert!(
        carrier.y.abs() < 0.5,
        "carrier must not be valign-shifted, got y={}",
        carrier.y
    );
    // ④ 行高不膨胀（修复前 tr 被增长 delta 推到 Σ taffy 堆叠高 138）。
    assert!(
        (tr.height - td1.height).abs() < 0.5,
        "row height must equal cell height, got row={} cell={}",
        tr.height,
        td1.height
    );
}

#[test]
fn r5509_bii_single_cell_baseline_unchanged() {
    // 单格态修复前后本就正确（avail<0 不触发 valign 位移）——防修复回归。
    let html = format!(
        r#"<html><head><style>{STYLE}</style></head><body><table><tr><td>{CELL1}</td></tr></table></body></html>"#
    );
    let (root, styles) = layout(&html);
    let tr = find_display(&root, &styles, |d| {
        matches!(d, zero_style_system::property::DisplayValue::TableRow)
    })
    .expect("table row box must exist");
    let td1 = tr
        .children
        .iter()
        .find(|c| {
            c.node_id
                .and_then(|id| styles.get(&id))
                .is_some_and(|s| matches!(s.display, zero_style_system::property::DisplayValue::TableCell))
        })
        .expect("cell box must exist");
    // R5509 时点的既有现象（stash 证伪新旧同值）：单格态 tr 高 78 = cell 60 + 一行
    // 18——行高多计一行；R5510 已修复（表内盒高度作者补写 content_height，见
    // r5510_table_row_backfill_invariant_tests），此处仍不锚行高（行高契约归 R5510
    // 锚），仅锚无 valign 位移。
    let carrier = td1
        .children
        .iter()
        .find(|c| c.is_r109_split)
        .expect("cell must contain an r109 split carrier");
    assert!(
        carrier.y.abs() < 0.5,
        "carrier must not be valign-shifted, got y={}",
        carrier.y
    );
}
