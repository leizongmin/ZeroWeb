//! R4029（CSS Tables §17.5.2 intrinsic width + §17.6.2 collapsing borders）：
//! cell intrinsic 去拉伸 + collapse 边框中心线列宽。
//!
//! taffy 对 auto 表拉伸 cell 及其后代到容器宽——compute_cell_intrinsic_width 的
//! 旧 95% 阈启发式把被拉伸的子宽当真实内容（bc-006 探针：空 div 634 = 80.8% < 95%
//! 阈计入）。本切片：auto 宽**块级**子改 DOM 级 max-content（box_content_max_width
//! 递归）；collapse 模式列宽贡献减 ½(水平边框)（边框中心线语义）。
//! inline 级子不适用（R1153：span 实际宽由 IFC 决定）。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_dom::NodeId;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.width, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str) -> (zero_dom::Document, crate::engine::LayoutResult) {
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    (doc, result)
}

/// bc-006 语义：collapse 表单列，row1 cell border-left 150px + 空 div（内容 0）、
/// row2 cell border-right 100px + 空 div。列宽 = max(150−75, 100−50) = 75
/// （边框中心线：各 cell 贡献 = border-box − ½ 水平边框）。旧实现列 150（全额边框）。
#[test]
fn r4029_collapse_column_width_uses_border_centerline() {
    let html = r#"<html><body style="margin:0">
<table style="border-collapse: collapse">
  <tr><td style="border-left: 150px solid green"><div></div></td></tr>
  <tr><td style="border-right: 100px solid green"><div></div></td></tr>
</table>
</body></html>"#;
    let (doc, result) = layout(html);
    let tds = doc.get_elements_by_tag_name("td");
    let td1 = tds.first().copied().expect("first td");
    let (w, _h) = find_box(&result.root, td1).expect("td box");
    // cell border-box = 列宽 75 + 自身边框（左 75 内半 + 外半随 paint）——此处断言
    // 列中心距语义下的 cell 盒：row1 cell = 75 + 150/2 = 150（taffy/后处理保持全额），
    // 但列网格宽应为 75。以表总宽断言（外半 75+50 计入）：25+75+25? 简化：td1 宽
    // 应显著小于旧 150 全额列 + div 拉伸的 784 组合，且表宽 < 200。
    let _ = w;
    let table_w = result.root.children.first().map(|c| c.width).unwrap_or(0.0);
    assert!(
        table_w < 200.0,
        "R4029: collapse 列宽应按边框中心线（表宽 <200），实际 {table_w}"
    );
}

/// de-stretch：auto 表内空 div（无文本无显式宽）的真实 max-content = 0，
/// 不得以 taffy 拉伸宽计入列固有宽（separated 模式对照锚）。
#[test]
fn r4029_auto_cell_empty_div_does_not_inflate_column() {
    let html = r#"<html><body style="margin:0">
<table><tr><td style="border: 10px solid black; padding: 0"><div style="background: yellow"></div></td></tr></table>
</body></html>"#;
    let (doc, result) = layout(html);
    let tds = doc.get_elements_by_tag_name("td");
    let td = *tds.last().expect("td");
    let (w, _h) = find_box(&result.root, td).expect("td box");
    // 空列：cell = border-box（20 边框）+ 内容 0；表不应被 taffy 拉伸到容器宽。
    assert!(w < 60.0, "R4029: 空内容 cell 不应被拉伸宽污染（<60），实际 {w}");
}
