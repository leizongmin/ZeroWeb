//! R5170 回归：rtl 表列序镜像（css-writing-modes-3 §7.1）。
//!
//! direction:rtl 的表把首列排到网格右侧（direction-applies-to-013/014：100px 固定
//! 布局 2×2 表的首 cell "X" 应落右半，旧实现落左半 1.04%）。实现为 position_cells
//! 落位处镜像（cell_x 仍按 ltr 推进，占位/折叠/跨列逻辑不变），水平域限定。
//!
//! kill-switch `ZW_TABLE_RTL_COL_ORDER=0` 回退由 corpus OFF 轮承担（LazyLock 进程级
//! 快照，进程内 set_var 不可靠——R3858 教训）。

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

fn find_box(root: &crate::types::LayoutBox, id: zero_dom::NodeId) -> Option<&crate::types::LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        for c in b.children.iter() {
            stack.push(c);
        }
    }
    None
}

/// 013 形态：direction:rtl 的 display:table（固定布局 100px、两列各 50px、
/// border-spacing:0）——首 cell 应落右列 x=50，次 cell 落左列 x=0（旧实现首 cell
/// x=0，列序未镜像）。行序不镜像（仅列序）。
#[test]
fn r5170_rtl_table_first_cell_lands_right_column() {
    let html = r#"<html><head><style>
#table {
  direction: rtl;
  display: table;
  table-layout: fixed;
  width: 100px;
  border-spacing: 0;
}
.row { display: table-row; }
.cell { display: table-cell; width: 50px; height: 50px; }
</style></head><body>
<div id="table">
  <div class="row">
    <div class="cell" id="c1">X</div>
    <div class="cell" id="c2"></div>
  </div>
  <div class="row">
    <div class="cell" id="c3">X</div>
    <div class="cell" id="c4"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    for (row, ids) in [("1", ("c1", "c2")), ("2", ("c3", "c4"))] {
        let first = doc.get_element_by_id(ids.0).expect("首 cell 应存在");
        let second = doc.get_element_by_id(ids.1).expect("次 cell 应存在");
        let fx = find_box(&result.root, first).expect("首 cell 盒应存在").x;
        let sx = find_box(&result.root, second).expect("次 cell 盒应存在").x;
        assert!(
            (fx - 50.0).abs() < 0.5,
            "行 {row} rtl 首 cell 应落右列 x=50，实际 {fx}（旧缺陷 x=0 列序未镜像）"
        );
        assert!(sx.abs() < 0.5, "行 {row} rtl 次 cell 应落左列 x=0，实际 {sx}");
    }
}

/// ltr 对照组：同标记去 direction——首 cell x=0（镜像恒等式还原）。
#[test]
fn r5170_ltr_table_first_cell_lands_left_column() {
    let html = r#"<html><head><style>
#table {
  display: table;
  table-layout: fixed;
  width: 100px;
  border-spacing: 0;
}
.row { display: table-row; }
.cell { display: table-cell; width: 50px; height: 50px; }
</style></head><body>
<div id="table">
  <div class="row">
    <div class="cell" id="c1">X</div>
    <div class="cell" id="c2"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let first = doc.get_element_by_id("c1").expect("首 cell 应存在");
    let second = doc.get_element_by_id("c2").expect("次 cell 应存在");
    let fx = find_box(&result.root, first).expect("首 cell 盒应存在").x;
    let sx = find_box(&result.root, second).expect("次 cell 盒应存在").x;
    assert!(fx.abs() < 0.5, "ltr 首 cell 应落左列 x=0，实际 {fx}");
    assert!((sx - 50.0).abs() < 0.5, "ltr 次 cell 应落右列 x=50，实际 {sx}");
}

/// 014 形态：direction:rtl 的 display:inline-table 同走 position_cells——首 cell
/// 同样落右列（镜像与 display 外层无关）。
#[test]
fn r5170_rtl_inline_table_first_cell_lands_right_column() {
    let html = r#"<html><head><style>
#table {
  direction: rtl;
  display: inline-table;
  table-layout: fixed;
  width: 100px;
  border-spacing: 0;
}
.row { display: table-row; }
.cell { display: table-cell; width: 50px; height: 50px; }
</style></head><body>
<div id="table">
  <div class="row">
    <div class="cell" id="c1">X</div>
    <div class="cell" id="c2"></div>
  </div>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);

    let first = doc.get_element_by_id("c1").expect("首 cell 应存在");
    let second = doc.get_element_by_id("c2").expect("次 cell 应存在");
    let fx = find_box(&result.root, first).expect("首 cell 盒应存在").x;
    let sx = find_box(&result.root, second).expect("次 cell 盒应存在").x;
    assert!(
        (fx - 50.0).abs() < 0.5,
        "inline-table rtl 首 cell 应落右列 x=50，实际 {fx}（旧缺陷 x=0）"
    );
    assert!(sx.abs() < 0.5, "inline-table rtl 次 cell 应落左列 x=0，实际 {sx}");
}
