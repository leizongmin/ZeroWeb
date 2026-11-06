//! R5434（CSS2 §17.5.2.1 auto layout 列宽）：col 元素指定宽 = 列 used width。
//!
//! 旧径 col 宽只是 floor，cell max-content 直接 max 进槽 → 列被可断行内容撑破
//! （vertical-align-baseline-003 ref 侧：`col#middle{width:80px}` 六个 Filler Text
//! 单行 430px；chrome 同形 80px 换行 6 行）。chrome oracle 三探针定谳：
//! ① col 80 + max-content 430 → 列 80（内容换行）② col 2 + "xx" → 列 = min-content
//！（floor 由内容撑起，col 宽不压破内容）③ col 80 + cell 显式宽 200 → 列 200
//! （cell 显式宽与 col 宽同为作者 floor，大者胜）。
//!
//! kill-switch `ZW_TABLE_COL_USED_WIDTH=0` 回退 floor-only 旧行为（OFF 轮不设单测，
//! 进程内 set_var 不可靠，R3858 教训；corpus A/B 覆盖）。探针无 FontLoader：
//! estimate 字体度量（R5162 同口径，断言避开度量敏感的精确 floor 值）。

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

const COL80_CSS: &str = "";
const COL80_HTML: &str = r#"<html><body style="margin:0">
<div style="display: table; font-size: 20px"><div style="display: table-column; width: 80px"></div>
<div style="display: table-row"><div id="c1" style="display: table-cell">a b c d e f g h i j k l m n o p q r s t</div></div>
</div></body></html>"#;

/// 主判例（vertical-align-baseline-003 ref 侧同构）：col 80 + 可断行长文本
/// （max-content ≫ 80）→ 列 = 80（内容换行），不被 max-content 撑破。
#[test]
fn r5434_col_spec_width_caps_wrappable_cell_contribution() {
    let cells = probe_cells(COL80_HTML, COL80_CSS, &["c1"]);
    assert_eq!(cells.len(), 1, "one cell expected, got {cells:?}");
    assert_eq!(
        cells[0].1, 80.0,
        "col 指定宽列须保持 80（可断行内容不撑破 used width），got {}",
        cells[0].1
    );
}

/// min-content floor 守卫（chrome p1/p2 形）：col 2px + 不可断长词 → 列 = floor
/// （词宽）而非 2（col 宽不压破内容）。旧径该形也是词宽（max-content 直接进槽），
/// 本针保证钳臂不把 floor 钳掉。
#[test]
fn r5434_col_spec_min_content_floor_survives_cap() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
<div style="display: table; font-size: 20px"><div style="display: table-column; width: 2px"></div>
<div style="display: table-row"><div id="c1" style="display: table-cell">xxxxxxxxxx</div></div>
</div></body></html>"#,
        "",
        &["c1"],
    );
    assert_eq!(cells.len(), 1, "one cell expected, got {cells:?}");
    assert!(
        cells[0].1 > 30.0,
        "col 2px + 不可断词 → 列须被 min-content floor 撑起（>30），got {}",
        cells[0].1
    );
}

/// 负向守卫（chrome p3 形）：cell 自身显式宽 200 与 col 宽 80 同为作者 floor，大者
/// 胜——cell 显式宽不入钳臂（css_width_auto 门）。
#[test]
fn r5434_cell_explicit_width_not_capped_by_col_spec() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
<div style="display: table"><div style="display: table-column; width: 80px"></div>
<div style="display: table-row"><div id="c1" style="display: table-cell; width: 200px">a b</div></div>
</div></body></html>"#,
        "",
        &["c1"],
    );
    assert_eq!(cells.len(), 1, "one cell expected, got {cells:?}");
    assert_eq!(
        cells[0].1, 200.0,
        "cell 显式宽 200 须胜过 col 宽 80，got {}",
        cells[0].1
    );
}

/// 窄内容守卫：col 80 + 窄内容（min-content < 80）→ 列仍 = 80（col 宽是 floor，
/// 内容不缩列）。钳臂 w ≤ spec 不触发，floor 语义不变。
#[test]
fn r5434_col_spec_floors_narrow_content() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
<div style="display: table; font-size: 20px"><div style="display: table-column; width: 80px"></div>
<div style="display: table-row"><div id="c1" style="display: table-cell">x</div></div>
</div></body></html>"#,
        "",
        &["c1"],
    );
    assert_eq!(cells.len(), 1, "one cell expected, got {cells:?}");
    assert_eq!(
        cells[0].1, 80.0,
        "窄内容不缩列（col 宽 floor 保持 80），got {}",
        cells[0].1
    );
}

/// corpus 案形（真 `<table>` + `<col>`）：同主判例——col 80 + 可断行长文本 → 列 = 80。
/// R2045 CSS-width col 经 collect_col_widths 进同一 Pass 0 快照路径。
#[test]
fn r5434_real_table_col_spec_caps_wrappable_cell() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
<table style="border-spacing:0"><col style="width:80px">
<tr><td id="c1">a b c d e f g h i j k l m n o p q r s t</td></tr>
</table>
</body></html>"#,
        "",
        &["c1"],
    );
    assert_eq!(cells.len(), 1, "one cell expected, got {cells:?}");
    assert_eq!(
        cells[0].1, 80.0,
        "真 <col> 指定宽列须保持 80（可断行内容不撑破），got {}",
        cells[0].1
    );
}

/// corpus 案形（真 `<table>` + `<col>`）：min-content floor 守卫同形单测。
#[test]
fn r5434_real_table_col_spec_min_content_floor_survives_cap() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
<table style="border-spacing:0"><col style="width:2px">
<tr><td id="c1">xxxxxxxxxx</td></tr>
</table>
</body></html>"#,
        "",
        &["c1"],
    );
    assert_eq!(cells.len(), 1, "one cell expected, got {cells:?}");
    assert!(
        cells[0].1 > 30.0,
        "真 <col> 2px + 不可断词 → 列须被 min-content floor 撑起（>30），got {}",
        cells[0].1
    );
}

/// min-width 守卫（CSS Tables §17.5.3）：col 80 + cell min-width 200（width auto、
/// 窄内容）→ 列 = 200（钳臂不得倒扣 cell min-width）。
#[test]
fn r5434_cell_min_width_floor_survives_col_spec_cap() {
    let cells = probe_cells(
        r#"<html><body style="margin:0">
<div style="display: table"><div style="display: table-column; width: 80px"></div>
<div style="display: table-row"><div id="c1" style="display: table-cell; min-width: 200px">x</div></div>
</div></body></html>"#,
        "",
        &["c1"],
    );
    assert_eq!(cells.len(), 1, "one cell expected, got {cells:?}");
    assert_eq!(
        cells[0].1, 200.0,
        "cell min-width 200 须胜过 col 宽 80（钳臂不倒扣），got {}",
        cells[0].1
    );
}
