//! R1752 / R5516：anonymous-table-cell-margin-collapsing（css-tables，R1752 记档 3.40%、
//! R5099 记档 2.77%）——chromium ref = filled-green-100px-square（table 100×100）。
//! ZW 旧态渲染 table ≈110×150（行溢出 height:100px）：table-row 的块级子按匿名
//! table-cell 入网格（CSS2 §17.2.1），converter 的 margin applies-to（§17.4）与
//! R5281 BFC 隔离均按自身 display=TableCell 判，匿名 cell 旁路——margin 全额入账
//! 且子 margin 塌穿 cell→row→table。R5516 落地匿名 cell margin 抑制 + 隔离 +
//! 行高分配输入 aware floor（table.rs content_row_heights），表高回到 ~100，
//! 本测试按原 characterization 自述翻转为修复锚。
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
    // R5516 修复锚（原 characterization 自述翻转）：表高应回到 ~100（chromium 同），
    // 行高分配吃满 definite target（100 + 0），行 2/3 无内容不吸收 extras。
    assert!(
        (t.height - 100.0).abs() < 10.0,
        "R1752/R5516: table h 应 ≈100（匿名 cell margin 抑制 + 隔离 + 分配对齐后），\
         实得 {:.0}",
        t.height
    );
    assert!(
        (t.margin_top - 0.0).abs() < 0.01,
        "R5516: table 自身 margin_top 应 0（首 cell margin 不再塌至表顶），实得 {}",
        t.margin_top
    );
}
