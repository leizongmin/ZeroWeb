//! R1752：anonymous-table-cell-margin-collapsing（css-tables，3.40% diff）已知 bug characterization。
//!
//! chromium ref = filled-green-100px-square（table 100×100）。ZW 渲染 table **110×150**
//!（行溢出 height:100px）。根因（LAYOUT_DUMP 实证）：ZW **不生成匿名 table-cell 盒**
//!（CSS2 §17.2.1：table-row 的 block 子应被包成 anon TableCell）——block div 直接作
//! TableRow 子 → table model 破坏 → margin 不穿过 anon cell 折叠 → 行高累加溢出。
//!
//! **结构性 table-model 改动，高风险**（同 R1630 broad-table-regresses 谱系），须
//! dedicated RFC + slices，非 quick fix。本测试 characterize 当前 buggy 行为（table h≈150），
//! 当 anon table-cell generation 实现后 h 应回到 ~100，此断言失败提示更新。
use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::values::DisplayValue;

#[test]
fn r1752_anon_table_margin_collapsing_current_behavior() {
    let html = r#"<html><body>
<div style="display:table; height:100px; background:red;">
  <div style="display:table-row; background:green;">
    <div style="width:100px; margin:50px 0;">
      <div style="margin:50px 0;"></div>
    </div>
    <div style="margin:50px 0;"></div>
  </div>
  <div style="display:table-row; background:green;">
    <div style="margin:50px 0;"></div>
  </div>
  <div style="display:table-row;"></div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    // 找 table 盒。
    let mut table: Option<&LayoutBox> = None;
    let mut stack: Vec<&LayoutBox> = vec![&result.root];
    while let Some(b) = stack.pop() {
        if let Some(id) = b.node_id
            && let Some(s) = styles.get(&id)
            && matches!(s.display, DisplayValue::Table)
        {
            table = Some(b);
            break;
        }
        stack.extend(b.children.iter());
    }
    let t = table.expect("table box");
    // KNOWN BUG（characterization）：当前 ZW table h≈150（行溢出 height:100px，因 anon
    // table-cell 未生成 + margin 不折叠）。chromium 应 h≈100。当 anon table-cell generation
    // RFC 落地，此断言会失败 → 改为 assert h≈100。
    assert!(
        (t.height - 150.0).abs() < 10.0,
        "R1752 characterization: 当前 ZW table h≈{:.0}（已知 bug：anon table-cell 未生成，\
         行溢出 height:100px；chromium 应 ~100）。修复后更新为 ~100。",
        t.height
    );
}
