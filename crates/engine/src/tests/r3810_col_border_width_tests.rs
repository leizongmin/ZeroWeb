//! R3810：col/colgroup 边框参与列宽 accounting + 左右缘冲突真实生效。
//!
//! 两段：① col/colgroup 边框半宽计入列尺寸（CSS2 §17.5.2.1 列宽从左 border 中心到
//! 右 border 中心；R5282 winner-center 起 `col_border_line_widths` 按线归属入账——
//! 组边落首/末列界线，只注首列左半，不再像旧 `col_border_width_halves` 给每个
//! 覆盖列 floor）；② R3805 左右缘解析的 `resolve_element_side` 调用传 `&mut`
//! **临时元组**，col/colgroup 边框的胜出结果随临时值丢弃，从未真正进入 winner
//!（border-*-width-applies-to-005/006 失权根因）。
//! chromium CDP 实证：colgroup border-left 1in + 空 cell → 表宽 96、cell0 区域
//! [48,96]（右半 border 即列宽）、col1 无面积。ZW 空 cell 固有宽有 1 字符 strut
//! 兜底（table_types.rs char_width），col1 保持 strut 宽——断言差值关系（col0−col1
//! = 胜者半宽 48、col0 起于格线 48）规避 strut 常量，reftest 面多出的空白不入差。
#[test]
fn r3810_col_border_width_and_edge_resolution() {
    // R5282 默认回退 border-center 后，本测锚定 winner-center 线归属几何，经测试缝启用。
    zero_layout_engine::table::set_winner_center_test(true);
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
    // 收集叶子 cell（空 cell 无子盒；行/行组/表盒有子不入选），坐标沿树累加成
    // 表框绝对值：每行 col0 = (x 48, w 9.6+48, bl 96)、col1 = (x 105.6, w 9.6, bl 0)
    fn collect(b: &zero_layout_engine::types::LayoutBox, ox: f32, oy: f32, out: &mut Vec<(f32, f32, f32, f32)>) {
        let ax = ox + b.x;
        let ay = oy + b.y;
        if b.children.is_empty() && b.height == 48.0 && b.width <= 96.0 {
            out.push((ax, ay, b.width, b.border_left));
        }
        for c in &b.children {
            collect(c, ax, ay, out);
        }
    }
    let mut cells = Vec::new();
    collect(&layout.root.clone(), 0.0, 0.0, &mut cells);
    assert!(cells.len() >= 4, "应找到 4 个 cell，实际 {}", cells.len());
    // cell0（每行首列）：左缘 override 生效 → border_left = 96、solid 样式覆盖
    assert!(
        (cells[0].3 - 96.0).abs() < 0.5 && (cells[2].3 - 96.0).abs() < 0.5,
        "colgroup border-left 96 应覆盖 col0 cell 左缘，实际 {:?} {:?}",
        cells[0].3,
        cells[2].3
    );
    // 线归属（R5282 col_border_line_widths）：组边 96 只落格线 0 → col0 槽含左半 48
    //（外半在表盒外、内半即槽），col1 不再借组边 floor（旧模型 col1 也得 48）。
    // 断差值 48 而非绝对宽——空 cell strut 兜底与 chromium 0 的偏差另行挂账。
    assert!(
        (cells[0].2 - cells[1].2 - 48.0).abs() < 0.5 && (cells[2].2 - cells[3].2 - 48.0).abs() < 0.5,
        "col0−col1 槽差应 = 组边左半 48，实际 {:?} {:?}",
        cells[0].2,
        cells[1].2
    );
    // cell0 起于格线 48（外半带在表盒外；chromium [48,96]）
    assert!(
        (cells[0].0 - 48.0).abs() < 0.5 && (cells[2].0 - 48.0).abs() < 0.5,
        "col0 cell 起于格线 48，实际 {:?} {:?}",
        cells[0].0,
        cells[2].0
    );
    assert!(
        (cells[1].3).abs() < 0.5 && (cells[3].3).abs() < 0.5,
        "col1 无组边线归属，左缘不获 override，实际 {:?} {:?}",
        cells[1].3,
        cells[3].3
    );
}
