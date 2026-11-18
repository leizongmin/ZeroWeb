//! R5516（CSS2 §17.4 × §17.2.1 匿名 cell × §17.5.3 行高分配）：
//! - table-row 的块级子按匿名 table-cell 入网格，margin 不应用（§17.4 margin
//!   applies-to——converter 按自身 display=TableCell 判，匿名 cell 旁路），且须
//!   立独立格式化上下文（R5281 同口径，子 margin 不塌穿 cell→row→table）。
//!   驱动案 css-tables/anonymous-table-cell-margin-collapsing（R5099 记档候选）：
//!   ZW 旧态表 150×109.6、首 cell margin-top 塌至表顶（应 100×100 绿方块）。
//! - 行高分配输入与装配度量同源：taffy 隔离盒高漏计末子 margin-bottom，
//!   content_row_heights 按 taffy 摊 extras、装配按 R1390 aware 兜底 → 差额双计
//!   （真 td 同病复现：rows 125/25 表 150，应 100/0 表 100）。

use crate::types::LayoutBox;
use std::sync::Arc;
use zero_dom::{Document, NodeKind};

fn layout(html: &str, css: &str) -> crate::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = crate::LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// 找指定 id 的 LayoutBox（深度优先）。
fn find_id<'a>(b: &'a LayoutBox, doc: &'a Document, id: &str) -> Option<&'a LayoutBox> {
    if let Some(nid) = b.node_id
        && let Some(node) = doc.get(nid)
        && let NodeKind::Element(e) = &node.kind
        && e.get_attribute("id").as_deref() == Some(id)
    {
        return Some(b);
    }
    b.children.iter().find_map(|c| find_id(c, doc, id))
}

/// 驱动案最小形：display:table（height:100px）> display:table-row > 块级 div
///（margin:50px 0）内含空 div（margin:50px 0）。抑制 + 隔离 + 分配对齐后：
/// 表盒 height=100、自身 margin_top=0（margin 不塌至表顶）、行 1=100/行 2=0。
/// 旧态：表 150、mt=50（子 margin 塌穿链）、行 100/50。
#[test]
fn r5516_anon_cell_margin_suppressed_row_height_exact() {
    let css = "div.table { display: table; height: 100px; } \
               div.row { display: table-row; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<div class="table" id="tbl">
 <div class="row" id="row1">
  <div id="cell-a" style="margin: 50px 0;">
    <div style="margin: 50px 0;"></div>
  </div>
  <div id="cell-b" style="margin: 50px 0;"></div>
 </div>
 <div class="row" id="row2">
  <div id="cell-c" style="margin: 50px 0;"></div>
 </div>
</div>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let result = layout(&html, css);
    let root = Arc::try_unwrap(result.root).unwrap_or_else(|arc| (*arc).clone());
    let tbl = find_id(&root, &doc, "tbl").expect("表盒应存在");
    assert!(
        (tbl.height - 100.0).abs() < 0.5,
        "表盒高应 100（definite，行 100+0），实得 {}",
        tbl.height
    );
    assert!(
        (tbl.margin_top - 0.0).abs() < 0.01,
        "表盒自身 margin_top 应 0（匿名 cell margin 抑制后无塌穿逃逸），实得 {}",
        tbl.margin_top
    );
    let row1 = find_id(&root, &doc, "row1").expect("row1 应存在");
    assert!(
        (row1.height - 100.0).abs() < 0.5,
        "行 1 高应 100（cell A 内容含子 margin box），实得 {}",
        row1.height
    );
    assert!(
        (row1.margin_top - 0.0).abs() < 0.01,
        "行 1 自身 margin_top 应 0（无塌穿），实得 {}",
        row1.margin_top
    );
    let row2 = find_id(&root, &doc, "row2").expect("row2 应存在");
    assert!(
        row2.height.abs() < 0.5,
        "行 2 高应 0（cell C margin 抑制后无内容，extras 不再错摊），实得 {}",
        row2.height
    );
}

/// 真 td 同病锚（分配输入 aware floor）：table height:100px，td 内空 div
///（margin:50px 0）+ 第二行空 td。旧态 taffy 隔离盒高漏末子 mb → 分配摊
/// extras 25、装配 aware 兜底 → rows 125/25、表 150。修后行 100/0、表 100。
#[test]
fn r5516_td_last_child_mb_distribution_exact() {
    let css = "table { height: 100px; border-spacing: 0; border-collapse: collapse; } \
               td { padding: 0; }";
    let html = format!(
        r#"<html><head><style>{css}</style></head><body>
<table id="tbl">
 <tr><td id="td1" style="width:100px"><div style="margin: 50px 0;"></div></td></tr>
 <tr><td id="td2"></td></tr>
</table>
</body></html>"#
    );
    let doc = zero_dom::parse_html(&html);
    let result = layout(&html, css);
    let root = Arc::try_unwrap(result.root).unwrap_or_else(|arc| (*arc).clone());
    let tbl = find_id(&root, &doc, "tbl").expect("table 应存在");
    assert!(
        (tbl.height - 100.0).abs() < 0.5,
        "表盒高应 100（definite，行 100+0 不再超额 150），实得 {}",
        tbl.height
    );
    let td1 = find_id(&root, &doc, "td1").expect("td1 应存在");
    assert!(
        (td1.height - 100.0).abs() < 0.5,
        "td1 高应 100（BFC 内子 margin 留盒内），实得 {}",
        td1.height
    );
    let td2 = find_id(&root, &doc, "td2").expect("td2 应存在");
    assert!(
        td2.height.abs() < 0.5,
        "td2 高应 0（无内容行不吸收 extras），实得 {}",
        td2.height
    );
    // 行帧推进已由 dump 取证（行 2 盒顶 = 表顶 8 + 行 1 高 100）；cell y 为
    // 行帧相对量，不入断言。
}
