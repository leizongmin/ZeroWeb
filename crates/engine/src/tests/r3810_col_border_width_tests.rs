//! R3810：col/colgroup 边框参与列宽 accounting + 左右缘冲突真实生效。
//!
//! 两段：① `col_border_width_halves` 把覆盖元素的边框半宽计入列宽 floor（CSS2
//! §17.5.2.1 列宽从左 border 中心到右 border 中心）；② R3805 左右缘解析的
//! `resolve_element_side` 调用传 `&mut` **临时元组**，col/colgroup 边框的胜出结果随
//! 临时值丢弃，从未真正进入 winner（border-*-width-applies-to-005/006 失权根因）。
//! chromium CDP 实证：colgroup border-left 1in + 空 cell → 表宽 96、cell0 区域
//! [48,96]（右半 border 即列宽）。
#[test]
fn r3810_col_border_width_and_edge_resolution() {
    let mut pipeline = crate::pipeline::RenderPipeline::new(800.0, 600.0);
    let html = r#"<html><head><style>
    #test { border-left-style: solid; border-left-width: 1in; display: table-column-group; }
    #table { border-collapse: collapse; display: table; }
    .column { display: table-column; }
    .row { display: table-row; }
    .cell { display: table-cell; height: 0.5in; }
    </style></head><body style="margin:0">
    <div id="table">
      <div id="test"><div class="column"></div><div class="column"></div></div>
      <div class="row"><div class="cell"></div><div class="cell"></div></div>
      <div class="row"><div class="cell"></div><div class="cell"></div></div>
    </div>
    </body></html>"#;
    let _ = pipeline.render_html(html, "");
    let layout = pipeline.layout().expect("layout");
    // 收集 4 个 cell（叶子、高 48）
    fn collect(b: &zero_layout_engine::types::LayoutBox, out: &mut Vec<(f32, f32, f32, f32)>) {
        if b.height == 48.0 && b.width <= 48.0 {
            out.push((b.x, b.y, b.width, b.border_left));
        }
        for c in &b.children {
            collect(c, out);
        }
    }
    let mut cells = Vec::new();
    collect(&layout.root.clone(), &mut cells);
    assert!(cells.len() >= 4, "应找到 4 个 cell（48 高），实际 {}", cells.len());
    // cell0（每行首列）：左缘 override 生效 → border_left = 96、solid 样式覆盖
    assert!(
        (cells[0].3 - 96.0).abs() < 0.5,
        "colgroup border-left 96 应覆盖 cell0 左缘，实际 {}",
        cells[0].3
    );
    // 列宽 floor 生效：每 cell 48 宽（右半 96/2）
    assert!(
        (cells[0].2 - 48.0).abs() < 0.5 && (cells[1].2 - 48.0).abs() < 0.5,
        "空 cell 列宽应含 col 边框半宽（48），实际 {:?} {:?}",
        cells[0].2,
        cells[1].2
    );
}
