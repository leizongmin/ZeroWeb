//! R5168 回归：CSS2 §17.2.1 匿名 table-cell 合成（table-row 内连续 inline 子包裹）。
//!
//! 背景：`build_row` 无 display 过滤，行内裸 inline span 被误当独立 cell 各占一列
//!（table-anonymous-objects-088/091/092：4 列 vs ref [匿名 cell, td, td] 3 列），
//! 且裸 span 盒几何为 taffy 交换帧 junk。本组测试钉住：
//! 1. 行内 [span, span, cell] 的前两个 span 合并为单个匿名 cell（行子 2 个盒）；
//! 2. 合成 cell 借宿主行 node_id + fragment_node_ids 圈定内容，compute_final 存储
//!    32px IFC 字号度量（paint Path B 字号来源）；
//! 3. `ZW_ANON_CELL_WRAP=0` kill-switch 完整回退（行子保持 3 个裸盒）。

use zero_css_parser::values::LengthValue;

/// 探针用：抽取 HTML 内联 <style> 块文本并解析（与 r5167 探针同口径）。
fn sheets(html: &str) -> Vec<zero_css_parser::Stylesheet> {
    let mut out = Vec::new();
    let mut rest = html;
    while let Some(start) = rest.find("<style") {
        let Some(tail_start) = rest[start..].find('>') else {
            break;
        };
        let after = &rest[start + tail_start + 1..];
        let Some(end) = after.find("</style>") else { break };
        out.push(zero_css_parser::Parser::parse_stylesheet(&after[..end]));
        rest = &after[end + 8..];
    }
    out
}

fn find_anon_cell(root: &crate::types::LayoutBox) -> Option<&crate::types::LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.is_anon_table_cell {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

fn find_box(root: &crate::types::LayoutBox, id: zero_dom::NodeId) -> Option<&crate::types::LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) && !b.is_anon_table_cell {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 088 形态：多行孤儿 row（sibling 经 merge_orphan_table_run 合入匿名表 → 逐行
/// build_row）。row1 = [inline span, inline span, table-cell span] → 显式 cell 落
/// col1（连续 inline run 占 col0 匿名列），不被挤到 col0（旧盒序缺陷：c1 叠在
/// 匿名内容上、列数少一）。裸 inline 走 R2160 multi-inline 跳过、step 8 无盒——
/// 本形态的修复在网格列预留 + 匿名列宽（fragment max-content），不产生合成盒。
#[test]
fn r5168_row_explicit_cell_lands_after_inline_run_column() {
    let html = r#"<html><head><style>
body { font-family: monospace; font-size: 2em; }
span.cell { display: table-cell; border: none; padding: 0; }
span.row { display: table-row; }
</style></head><body>
<div>
  <span class="row" id="row1">
    <span id="s1">Row 1, Col 1</span>
    <span id="s2">Row 1, Col 1</span>
    <span class="cell" id="c1">Row 1, Col 2</span>
  </span>
  <span class="row" id="row2">
    <span class="cell" id="c21">R2C1</span>
    <span class="cell" id="c22">R2C2</span>
    <span class="cell" id="c23">R2C3</span>
  </span>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let c1 = doc.get_element_by_id("c1").expect("c1 应存在");
    let c1_box = find_box(&result.root, c1).expect("c1 盒应存在");
    // run 文本 "Row 1, Col 1"（monospace 32px）max-content ≈ 230px → c1 应在其右
    //（col1）。旧盒序把 c1 放 col0（x≈0，与匿名内容重叠）。
    assert!(
        c1_box.x > 100.0,
        "c1 应落在 inline run 匿名列之后（col1）：x={}",
        c1_box.x
    );
}

/// 合成 cell 走 compute_final：存储 32px IFC 字号度量（paint Path B 的字号来源，
/// 修复前裸 span 文本落 16px 默认字号）。
#[test]
fn r5168_anon_cell_stores_ifc_font_metrics() {
    let html = r#"<html><head><style>
body { font-family: monospace; }
div { font-size: 2em; }
span.row { display: table-row; }
</style></head><body>
<div>
  <span class="row" id="row1">
    <span id="s1">Text</span>
    <span style="display: table-cell" id="c1">Cell</span>
  </span>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    assert!(
        matches!(
            styles
                .get(&doc.get_element_by_id("row1").unwrap())
                .map(|s| &s.font_size),
            Some(LengthValue::Px(32.0))
        ),
        "行应继承 div 32px 字号"
    );
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let row_node = doc.get_element_by_id("row1").expect("row 应存在");
    let row_box = find_box(&result.root, row_node).expect("row 盒应存在");
    let anon = find_anon_cell(row_box).expect("应存在合成匿名 cell");
    let stored: Vec<f32> = anon.text_node_font_sizes.values().copied().collect();
    assert_eq!(
        stored,
        vec![32.0],
        "合成 cell 应存储 32px IFC 度量供 paint Path A 使用（裸 span 直绘为 16px 默认）"
    );
}

// kill-switch 判别（`ZW_ANON_CELL_WRAP=0` 下两案即红：行子 3 裸盒、无合成 cell、
// 无存储度量）经外部 env 运行验证，不在进程内 set_var（并行测试线程污染）。
