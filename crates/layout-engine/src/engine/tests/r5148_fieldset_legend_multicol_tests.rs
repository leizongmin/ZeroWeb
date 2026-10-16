//! R5148（HTML 渲染规范 fieldset/legend 特殊布局 × css-multicol）：fieldset 自身是
//! multicol 容器时，渲染 legend 不入列流——chromium oracle（multicol-span-all-fieldset-001）
//! legend 独占容器顶部行，列区平衡其余子；ZW 旧版把 legend 作普通列项
//! （[legend|block1|block2]），与 ref 页（legend 在 multicol 容器外）自源失配。
//! kill-switch `ZW_MULTICOL_FIELDSET_LEGEND=0`。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::Parser;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

fn find_box(root: &LayoutBox, node_id: NodeId) -> Option<(f32, f32, f32)> {
    let mut stack = vec![root];
    while let Some(b) = stack.pop() {
        if b.node_id == Some(node_id) {
            return Some((b.x, b.y, b.height));
        }
        stack.extend(b.children.iter());
    }
    None
}

fn layout(html: &str, css: &str) -> crate::engine::LayoutResult {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    engine.compute(&doc, &styles)
}

/// span-all 结构（对齐 multicol-span-all-fieldset-001）：fieldset column-count:3 +
/// legend + block1/block2 + h3 spanner + block3/block4。legend 应独占顶部行，
/// block1/block2 平衡进前两列（不再 [legend|block1|block2] 三列分布）。
#[test]
fn r5148_fieldset_multicol_legend_not_a_column_item() {
    let html = r#"<html><body style="margin:0">
<fieldset><legend>Legend</legend>
<div>block1</div><div>block2</div>
<h3>spanner</h3>
<div>block3</div><div>block4</div>
</fieldset>
</body></html>"#;
    let css = r#"fieldset { column-count: 3; width: 400px; margin: 0; }
h3 { column-span: all; margin: 0; }
div { margin: 0; }"#;
    let doc = zero_dom::parse_html(html);
    let fieldset = doc
        .get_elements_by_tag_name("fieldset")
        .into_iter()
        .next()
        .expect("fieldset");
    let legend = doc
        .get_elements_by_tag_name("legend")
        .into_iter()
        .next()
        .expect("legend");
    let divs = doc.get_elements_by_tag_name("div");
    let (d1, d3) = (*divs.first().expect("block1"), *divs.get(2).expect("block3"));
    let h3 = doc.get_elements_by_tag_name("h3").into_iter().next().expect("h3");
    let result = layout(html, css);
    let (_, lg_y, lg_h) = find_box(&result.root, legend).expect("legend box");
    let (_, d1_y, _) = find_box(&result.root, d1).expect("block1 box");
    let (_, _, d2_h) = {
        let d2 = *divs.get(1).expect("block2");
        find_box(&result.root, d2).expect("block2 box")
    };
    let (_, h3_y, _) = find_box(&result.root, h3).expect("spanner box");
    let (_, d3_y, _) = find_box(&result.root, d3).expect("block3 box");
    let (_, _, fs_h) = find_box(&result.root, fieldset).expect("fieldset box");
    // legend 独占顶部行：列流原点 = legend 外尺寸，block1 在 legend 底之下。
    assert!(
        lg_y + lg_h <= d1_y + 0.5,
        "R5148: legend 应在列流行之上（lg_y={lg_y}+{lg_h} vs block1_y={d1_y}）——旧版 legend 作列项与 block1 同行"
    );
    // block1/block2 同一列行（区域平衡），spanner 在其下，block3 更在下。
    assert!(
        h3_y >= d1_y + d2_h.max(lg_h),
        "R5148: spanner 应在首区域列行之下（h3_y={h3_y}, d1_y={d1_y}, d2_h={d2_h}）"
    );
    assert!(
        d3_y > h3_y,
        "R5148: 尾区域应在 spanner 之下（d3_y={d3_y} vs h3_y={h3_y}）"
    );
    // 容器高含 legend 行（排除 legend 不丢其高度贡献）。
    assert!(
        fs_h >= lg_h + d2_h + 1.0,
        "R5148: fieldset 高度应含 legend 行 + 列行（fs_h={fs_h}, lg_h={lg_h}, 行高={d2_h}）"
    );
}

/// 非 spanner 平衡结构：fieldset column-count:3 + legend + 2 块 → 2 块平衡进前两列，
/// legend 不占列槽（旧版 [legend|block1|block2] 三列各一子）。
#[test]
fn r5148_fieldset_multicol_balance_excludes_legend() {
    let html = r#"<html><body style="margin:0">
<fieldset><legend>Legend</legend>
<div>block1</div><div>block2</div>
</fieldset>
</body></html>"#;
    let css = r#"fieldset { column-count: 3; width: 400px; margin: 0; }
div { margin: 0; }"#;
    let doc = zero_dom::parse_html(html);
    let legend = doc
        .get_elements_by_tag_name("legend")
        .into_iter()
        .next()
        .expect("legend");
    let divs = doc.get_elements_by_tag_name("div");
    let (d1, d2) = (*divs.first().expect("block1"), *divs.get(1).expect("block2"));
    let result = layout(html, css);
    let (lg_x, lg_y, lg_h) = find_box(&result.root, legend).expect("legend box");
    let (d1_x, d1_y, _) = find_box(&result.root, d1).expect("block1 box");
    let (d2_x, d2_y, _) = find_box(&result.root, d2).expect("block2 box");
    // legend 顶部行首（列流外），block1/block2 同行、分列 1/2。
    assert!(
        lg_y + lg_h <= d1_y + 0.5,
        "R5148: legend 应在块行之上（lg_y={lg_y}+{lg_h} vs d1_y={d1_y}）"
    );
    assert!(
        (d1_y - d2_y).abs() < 0.5 && d2_x > d1_x && d1_x >= lg_x,
        "R5148: block1/block2 应同行分列（d1=({d1_x},{d1_y}) d2=({d2_x},{d2_y})）——旧版 block2 落第 3 列"
    );
}
