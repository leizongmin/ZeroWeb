//! R5167 回归：水平 table-cell IFC 度量存储（paint Path A 字号来源）+ 匿名行 cell y。
//!
//! 背景：`compute_final_inline_layouts` 早退门（`is_block_level` 清单不含 TableCell）
//! 使水平 cell 从不存储 text_node_font_sizes，paint 回退 Path B 直系文本 16px 默认字号，
//! 而 cell 盒支柱保持真实字号（table-anonymous-objects-062：td 16px 字形 vs span cell
//! 32px 字形，行高均 46px）。本组测试钉住三件事：
//! 1. 样式层 font-size 继承正确（table{font-size:2em} → td/cell 32px，style 块规则生效）；
//! 2. cell 盒存储 IFC 度量（32px），paint Path A 可用；
//! 3. 匿名行 cell 的盒 y 落在自己所在网格行（Case A：table 直接子裸 cell；
//!    Case B：row-group 直接子裸 cell），不与首行重叠。

use zero_css_parser::values::LengthValue;

/// 探针用：抽取 HTML 内联 <style> 块文本并解析（管线 collect_stylesheets 同口径的
/// 最小等价——测试探针无 zero-engine 依赖，直接 parse_stylesheet）。
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

fn font_of(
    styles: &std::collections::HashMap<zero_dom::NodeId, zero_style_system::ComputedStyle>,
    doc: &zero_dom::Document,
    id: &str,
) -> f64 {
    let node = doc.get_element_by_id(id).expect("节点应存在");
    match styles.get(&node).map(|s| &s.font_size) {
        Some(LengthValue::Px(v)) => *v,
        other => panic!("#{id} font_size 非像素值: {other:?}"),
    }
}

fn find_box(root: &crate::types::LayoutBox, id: zero_dom::NodeId) -> Option<&crate::types::LayoutBox> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(id) {
            return Some(b);
        }
        stack.extend(b.children.iter());
    }
    None
}

/// 盒的绝对 y（累加祖先链——cell 在显式行帧内 y=0 是行相对值，比较须用绝对坐标）。
fn abs_y(root: &crate::types::LayoutBox, id: zero_dom::NodeId) -> f32 {
    fn go(b: &crate::types::LayoutBox, acc: f32, id: zero_dom::NodeId) -> Option<f32> {
        let next = acc + b.y;
        if b.node_id == Some(id) {
            return Some(next);
        }
        for c in &b.children {
            if let Some(v) = go(c, next, id) {
                return Some(v);
            }
        }
        None
    }
    go(root, 0.0, id).expect("盒应存在")
}

/// CSS2.1 §17.4/§17.2.1：table 上的 font-size 继承进 td / 匿名表 cell。
/// 钉住样式层行为——存储臂的字号来源前提（style 块规则须生效）。
#[test]
fn r5167_font_inheritance_through_table() {
    // ref 形态：table{font-size:2em} + tr>td（style 块规则）。
    let ref_html = r#"<html><head><style>
body { font-family: monospace; }
table { font-size: 2em; border-spacing: 0; }
td { padding: 0; }
</style></head><body>
<div>
  <table id=t><tr><td id=r1>x</td></tr></table>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(ref_html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(ref_html));
    assert_eq!(font_of(&styles, &doc, "t"), 32.0, "table font-size:2em 应 32px");
    assert_eq!(font_of(&styles, &doc, "r1"), 32.0, "td 继承 table 32px");

    // test 形态：裸 table-cell span + row-group>row>cell（inline style）。
    let test_html = r#"<html><body style="font-size: 2em">
<span style="display: table-cell" id=c11>x</span>
<span style="display: table-row-group"><span style="display: table-row">
  <span style="display: table-cell" id=c21>x</span>
</span></span>
</body></html>"#;
    let doc2 = zero_dom::parse_html(test_html);
    let mut sys2 = zero_style_system::StyleSystem::new();
    sys2.set_viewport(800.0, 600.0);
    let styles2 = sys2.compute_styles(&doc2, &sheets(test_html));
    assert_eq!(font_of(&styles2, &doc2, "c11"), 32.0, "裸 cell 继承 body 32px");
    assert_eq!(font_of(&styles2, &doc2, "c21"), 32.0, "row-group 内 cell 继承 32px");
}

/// ref 形态（table{font-size:2em}）跑完整 engine.compute 后，td 盒的
/// text_node_font_sizes 应存 32px（paint Path A 的字号来源；修复前为空）。
#[test]
fn r5167_td_stores_ifc_font_metrics() {
    let ref_html = r#"<html><head><style>
body { font-family: monospace; }
table { font-size: 2em; border-spacing: 0; }
td { padding: 0; }
</style></head><body>
<div>
  <table><tr><td id=r1>x</td></tr></table>
</div>
</body></html>"#;
    let doc = zero_dom::parse_html(ref_html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(ref_html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let td = doc.get_element_by_id("r1").expect("td 应存在");
    let box_node = find_box(&result.root, td).expect("td 盒应存在");
    let stored: Vec<f32> = box_node.text_node_font_sizes.values().copied().collect();
    assert_eq!(stored, vec![32.0], "td 应存储 32px IFC 度量供 paint Path A 使用");
}

/// CSS2.1 §17.2.1 匿名行：table 直接子裸 cell（Case A）落在第二行 y，
/// 不与首行重叠（table-anonymous-objects-063：修复前 "Row 22" 与 "Row 1" 同 y）。
#[test]
fn r5167_case_a_direct_cell_anon_row_y() {
    let html = r#"<html><body style="font-size: 2em">
<span style="display:table">
  <span style="display: table-row-group"><span style="display: table-row">
    <span style="display: table-cell" id=c11>Row 1</span>
  </span></span>
  <span style="display: table-cell" id=c22>Row 22</span>
  <span style="display: table-row-group"><span style="display: table-row">
    <span style="display: table-cell" id=c33>Row 333</span>
  </span></span>
</span>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let y = |id: &str| -> f32 {
        let node = doc.get_element_by_id(id).expect("节点应存在");
        abs_y(&result.root, node)
    };
    let (y11, y22, y33) = (y("c11"), y("c22"), y("c33"));
    assert!(
        y22 > y11 + 1.0,
        "Case A 裸 cell（第二行）y 应低于首行：c22={y22} c11={y11}"
    );
    assert!(y33 > y22 + 1.0, "第三行 y 应低于匿名行：c33={y33} c22={y22}");
}

/// CSS2.1 §17.2.1 匿名行：row-group 直接子裸 cell（Case B）同样落在自己网格行
///（table-anonymous-objects-133：修复前与首行同 y）。
#[test]
fn r5167_case_b_rg_direct_cell_anon_row_y() {
    let html = r#"<html><body style="font-size: 2em">
<span style="display:table">
  <span style="display: table-row-group">
    <span style="display: table-row">
      <span style="display: table-cell" id=b11>Row 1</span>
    </span>
    <span style="display: table-cell" id=b22>Row 22</span>
    <span style="display: table-row">
      <span style="display: table-cell" id=b33>Row 333</span>
    </span>
  </span>
</span>
</body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = zero_style_system::StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &sheets(html));
    let mut engine = crate::engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let y = |id: &str| -> f32 {
        let node = doc.get_element_by_id(id).expect("节点应存在");
        abs_y(&result.root, node)
    };
    let (y11, y22, y33) = (y("b11"), y("b22"), y("b33"));
    assert!(
        y22 > y11 + 1.0,
        "Case B 裸 cell（第二行）y 应低于首行：b22={y22} b11={y11}"
    );
    assert!(y33 > y22 + 1.0, "第三行 y 应低于匿名行：b33={y33} b22={y22}");
}
