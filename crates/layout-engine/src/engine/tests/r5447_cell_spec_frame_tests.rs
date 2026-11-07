//! R5447（CSS2 §17.5.2 cell content-box 语义）：cell 指定绝对宽度的列贡献 =
//! width + padding + border（separated 显式臂 + fixed 分支 col_cell_spec 两处）。
//!
//! 旧径 specified 直接当列宽——c5525-fltmult-000 ref 侧 td width:100 + border:20×2
//! 列塌 100（chrome 140），与 test 侧 float div（60+40+40=140 正确）错位 → 整案
//! ✗10.32%。chrome oracle 探针矩阵定谳（A-G/H 形）：auto/fixed、有无显式表宽全部
//! content-box 同口径。
//!
//! kill-switch `ZW_CELL_SPEC_FRAME=0` 回退旧行为（OFF 轮不设单测——进程内 set_var
//! 不可靠，R3858 教训；corpus A/B 覆盖）。探针无 FontLoader：estimate 字体度量
//! （R5434 同口径），断言用几何不受字体影响的裸 td。

use crate::engine::LayoutEngine;
use zero_css_parser::Parser;
use zero_style_system::StyleSystem;

/// 布局并按 id 收集单元格 (x, width)（按收集顺序；id 形如 c1）。
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

/// chrome A 形（c5525-fltmult-000 ref 侧同构）：separated auto 表，td width:100 +
/// border 左右 20px → cell border-box 140（旧径塌 100）。
#[test]
fn r5447_separated_cell_spec_width_adds_frame() {
    let (x, w) = probe_cells(
        r#"<html><body style="margin:0">
<div style="display: table; border-spacing: 0"><div style="display: table-row">
<div id="c1" style="display: table-cell; width: 100px; border-left: 20px solid black; border-right: 20px solid black"></div>
</div></div></body></html>"#,
        "",
        &["c1"],
    )
    .remove(0);
    assert_eq!(x, 0.0);
    assert!(
        (w - 140.0).abs() < 1.0,
        "cell border-box 应 = 100 + 20×2 = 140, got {w}"
    );
}

/// chrome C 形：padding 同计（width:100 + border 20×2 + padding 10×2 → 160）。
#[test]
fn r5447_separated_cell_spec_width_adds_padding() {
    let (_, w) = probe_cells(
        r#"<html><body style="margin:0">
<div style="display: table; border-spacing: 0"><div style="display: table-row">
<div id="c1" style="display: table-cell; width: 100px; padding-left: 10px; padding-right: 10px; border-left: 20px solid black; border-right: 20px solid black"></div>
</div></div></body></html>"#,
        "",
        &["c1"],
    )
    .remove(0);
    assert!(
        (w - 160.0).abs() < 1.0,
        "cell border-box 应 = 100 + 10×2 + 20×2 = 160, got {w}"
    );
}

/// chrome D/E 形回归卫：frame 为 0 的裸 td width:100 → 列 100（R5447 前后不变）。
#[test]
fn r5447_bare_spec_width_unchanged() {
    let (_, w) = probe_cells(
        r#"<html><body style="margin:0">
<div style="display: table; border-spacing: 0"><div style="display: table-row">
<div id="c1" style="display: table-cell; width: 100px"></div>
</div></div></body></html>"#,
        "",
        &["c1"],
    )
    .remove(0);
    assert!((w - 100.0).abs() < 1.0, "裸 specified 列应 = 100, got {w}");
}

/// chrome H 形：fixed + 显式表宽 400 + td width:100 + border 20×2 + auto td →
/// 列 140 / 400−140=260（col_cell_spec 加 frame 口径，fixed 分支消费点）。
#[test]
fn r5447_fixed_explicit_table_cell_spec_adds_frame() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
<div style="display: table; table-layout: fixed; width: 400px; border-spacing: 0"><div style="display: table-row">
<div id="c1" style="display: table-cell; width: 100px; border-left: 20px solid black; border-right: 20px solid black"></div>
<div id="c2" style="display: table-cell">x</div>
</div></div></body></html>"#,
        "",
        &["c1", "c2"],
    );
    assert_eq!(cells.len(), 2);
    let (x1, w1) = cells[0];
    let (x2, w2) = cells[1];
    assert!((w1 - 140.0).abs() < 1.0, "specified 列应 = 140, got {w1}");
    assert_eq!(x1, 0.0);
    assert!((x2 - 140.0).abs() < 1.0, "auto 列起点应 = 140, got {x2}");
    assert!((w2 - 260.0).abs() < 1.0, "auto 列应均分 260, got {w2}");
}
