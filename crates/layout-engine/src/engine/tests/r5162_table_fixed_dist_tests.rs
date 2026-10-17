//! R5162（CSS 2.2 §17.5.2.1）：table-layout:fixed + 显式表宽的列宽分布。
//!
//! 旧路径把单元格内容固有宽混进列宽再按 fixed_capped 比例收缩：空 cell 的 strut
//! 残留 min-content（0.6em）与内容宽一起把 specified 列摊薄——contain-paint-cell-001
//! （col 4ch=200 的三列表）[0,200,0] 变 [21.43,157.14,21.43]。chromium：specified 列
//! 保持（步骤 1-2），剩余表宽 auto 列均分（步骤 3「equally divide」），内容不参与。
//! 真 `<col>` 元素的 CSS 宽由 collect_col_widths 盒循环拾取（R2045 口径，
//! contain-paint-cell-002 驱动）。kill-switch `ZW_TABLE_FIXED_DIST=0`。
//!
//! 探针无 FontLoader：ch = 0.5em 兜底（100px 字号 → 4ch = 200px，确定值）。

use crate::engine::LayoutEngine;
use zero_css_parser::Parser;
use zero_style_system::StyleSystem;

/// 布局并按 id 收集单元格 (x, width)（按收集顺序；id 形如 c1/c2/c3）。
fn probe_cells(html: &str, css: &str, ids: &[&str]) -> Vec<(f32, f32)> {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut found: Vec<(String, f32, f32)> = Vec::new();
    fn walk(
        box_: &crate::types::LayoutBox,
        doc: &zero_dom::Document,
        ids: &[&str],
        found: &mut Vec<(String, f32, f32)>,
    ) {
        if let Some(nid) = box_.node_id
            && let Some(el) = doc.get(nid)
            && let zero_dom::NodeKind::Element(e) = &el.kind
            && let Some(id_attr) = e.get_attribute("id")
            && ids.contains(&id_attr.as_str())
        {
            found.push((id_attr, box_.x, box_.width));
        }
        for c in &box_.children {
            walk(c, doc, ids, found);
        }
    }
    walk(&result.root, &doc, ids, &mut found);
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found.into_iter().map(|(_, x, w)| (x, w)).collect()
}

/// specified 列保持其宽：col 4ch=200 的三列 fixed 表（表宽 200）→ [0, 200, 0]。
/// contain-paint-cell-001 复现形（旧路径 [21.43, 157.14, 21.43]）。
#[test]
fn test_fixed_specified_column_not_diluted() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
        <div style="display: table; table-layout: fixed; width: 200px; font-size: 100px">
        <div style="display: table-column"></div><div style="display: table-column; width: 4ch"></div><div style="display: table-column"></div>
        <div style="display: table-row"><div id="c1" style="display: table-cell"></div><div id="c2" style="display: table-cell"></div><div id="c3" style="display: table-cell"></div></div>
        </div>
        </body></html>"#,
        "",
        &["c1", "c2", "c3"],
    );
    assert_eq!(cells.len(), 3, "three cells expected, got {cells:?}");
    assert_eq!(cells[0], (0.0, 0.0), "auto col with no leftover gets 0");
    assert_eq!(cells[1], (0.0, 200.0), "specified 4ch=200 column keeps its width");
    assert_eq!(cells[2], (200.0, 0.0), "auto col with no leftover gets 0");
}

/// 剩余表宽由 auto 列均分（§17.5.2.1 步骤 3「equally divide」，非按内容比例）：
/// 中列 2ch=100、表宽 200 → 两侧各 50。
#[test]
fn test_fixed_remaining_split_equally_among_auto() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
        <div style="display: table; table-layout: fixed; width: 200px; font-size: 100px">
        <div style="display: table-column"></div><div style="display: table-column; width: 2ch"></div><div style="display: table-column"></div>
        <div style="display: table-row"><div id="c1" style="display: table-cell"></div><div id="c2" style="display: table-cell"></div><div id="c3" style="display: table-cell"></div></div>
        </div>
        </body></html>"#,
        "",
        &["c1", "c2", "c3"],
    );
    assert_eq!(
        cells,
        vec![(0.0, 50.0), (50.0, 100.0), (150.0, 50.0)],
        "specified 100 kept, remaining 100 split equally, got {cells:?}"
    );
}

/// 单元格内容不参与 fixed 列宽（溢出由 cell overflow 呈现）：中列 4ch=200、内容
/// PASS+span（旧路径内容宽 440 混入 → 列被摊薄），列宽仍 [0, 200, 0]。
#[test]
fn test_fixed_content_does_not_dilate_columns() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
        <div style="display: table; table-layout: fixed; width: 200px; font-size: 100px">
        <div style="display: table-column"></div><div style="display: table-column; width: 4ch"></div><div style="display: table-column"></div>
        <div style="display: table-row"><div id="c1" style="display: table-cell"></div><div id="c2" style="display: table-cell">PASS<span>FAIL</span></div><div id="c3" style="display: table-cell"></div></div>
        </div>
        </body></html>"#,
        "",
        &["c1", "c2", "c3"],
    );
    assert_eq!(
        cells,
        vec![(0.0, 0.0), (0.0, 200.0), (200.0, 0.0)],
        "cell content must not dilute the specified column, got {cells:?}"
    );
}

/// 真 `<col>` 元素的 CSS width 拾取（collect_col_widths 盒循环，R2045 口径）：
/// `<colgroup><col><col id=mid>{width:200px}</colgroup>` + fixed 表 200 → 中列 200。
/// contain-paint-cell-002 复现形。
#[test]
fn test_real_col_css_width_applies_in_fixed_table() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
        <table id="t" style="table-layout: fixed; width: 200px; border-spacing: 0">
        <colgroup><col><col id="mid"><col></colgroup>
        <tr><td id="c1"></td><td id="c2"></td><td id="c3"></td></tr>
        </table>
        </body></html>"#,
        "col#mid { width: 200px; }",
        &["c1", "c2", "c3"],
    );
    assert_eq!(
        cells,
        vec![(0.0, 0.0), (0.0, 200.0), (200.0, 0.0)],
        "real <col> CSS width must set the column track, got {cells:?}"
    );
}

/// auto 宽 fixed 表不走新分支（§17.5.2.1 CAP/MIN 交互未案驱动，维持旧路径）：
/// 无表宽时收缩到列内容和（单 cell 内容 120 → 表 120）。
#[test]
fn test_fixed_auto_width_table_keeps_legacy_shrink() {
    let doc = zero_dom::parse_html(
        r#"<html><body style="margin:0">
        <div style="display: table; table-layout: fixed"><div style="display: table-row"><div style="display: table-cell"><div style="width: 120px; height: 10px"></div></div></div></div>
        </body></html>"#,
    );
    let stylesheets = vec![Parser::parse_stylesheet("")];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let table = &result.root.children[1].children[0];
    assert!(
        (table.width - 120.0).abs() < 2.0,
        "auto-width fixed table shrinks to column sum (~120), got {}",
        table.width
    );
}
