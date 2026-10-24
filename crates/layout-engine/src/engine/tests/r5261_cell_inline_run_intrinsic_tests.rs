//! R5261（CSS2 §17.5.2 cell max-content + css-sizing-3 inline 轴求和）：
//! cell intrinsic 宽的 inline 级显式宽子按**相邻 run 求和**计入列宽；cell 内
//! inline 级子（含 va≠baseline 原子盒）按行内流排布，不折行竖叠。
//!
//! 旧实现两处缺陷叠加出 img-swatch 参照表竖条形态（css/CSS2/tables/
//! border-conflict-element-001d/001e，`<td><img>×9</td>` + `img{vertical-align:bottom}`）：
//! ① 列宽逐子取 max = 单图宽（20px），行内逐图折行；
//! ② cell 内容行位同步走 remeasure 复用臂，全 baseline 门把 va≠baseline 原子 run
//!    拒之门外，img 滞留 taffy 块堆叠位。
//! run 只在 DOM 相邻盒子间累积：`<br>` 强制断行、间隔文本节点均断 run
//!（该形态 max-content 语义是 max 而非和）。

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

fn cell_width(html: &str) -> f32 {
    let (doc, result) = layout(html);
    let tds = doc.get_elements_by_tag_name("td");
    let td = *tds.first().expect("td");
    find_box(&result.root, td).expect("td box").0
}

/// 按 DOM 序收集 img 盒的 (x, y)。
fn img_positions(html: &str) -> Vec<(f32, f32)> {
    fn walk(b: &LayoutBox, id: NodeId) -> Option<(f32, f32)> {
        if b.node_id == Some(id) {
            return Some((b.x, b.y));
        }
        b.children.iter().find_map(|c| walk(c, id))
    }
    let (doc, result) = layout(html);
    doc.get_elements_by_tag_name("img")
        .iter()
        .filter_map(|&id| walk(&result.root, id))
        .collect()
}

/// 相邻 inline img run 求和：3×20px img → 列宽 60（旧 max = 20，逐图折行）。
#[test]
fn r5261_adjacent_inline_img_run_sums_column() {
    let w = cell_width(
        r#"<html><body style="margin:0"><table><tr><td><img style="width:20px;height:20px"><img style="width:20px;height:20px"><img style="width:20px;height:20px"></td></tr></table></body></html>"#,
    );
    assert!(
        (w - 60.0).abs() < 0.5,
        "R5261: 相邻 inline img run 应求和列宽 60，实际 {w}"
    );
}

/// `<br>` 强制断行断 run：两图列宽 = max(20) 而非和 40。
#[test]
fn r5261_br_breaks_inline_run() {
    let w = cell_width(
        r#"<html><body style="margin:0"><table><tr><td><img style="width:20px;height:20px"><br><img style="width:20px;height:20px"></td></tr></table></body></html>"#,
    );
    assert!(
        (w - 20.0).abs() < 0.5,
        "R5261: <br> 断 run 后列宽应保持单图 max 20，实际 {w}"
    );
}

/// 间隔文本节点断 run（保守：不求和也不低于旧 max 语义）。
#[test]
fn r5261_text_gap_breaks_inline_run() {
    let w = cell_width(
        r#"<html><body style="margin:0"><table><tr><td><img style="width:20px;height:20px">x<img style="width:20px;height:20px"></td></tr></table></body></html>"#,
    );
    assert!(
        (w - 20.0).abs() < 0.5,
        "R5261: 文本间隔断 run，列宽 = 单图 max 20，实际 {w}"
    );
}

/// cell 内相邻 img 共享行盒（baseline 对照）。
#[test]
fn r5261_td_baseline_imgs_flow_inline() {
    let pos = img_positions(
        r#"<html><body style="margin:0"><table><tr><td><img style="width:20px;height:20px"><img style="width:20px;height:20px"><img style="width:20px;height:20px"></td></tr></table></body></html>"#,
    );
    assert_eq!(pos, vec![(0.0, 0.0), (20.0, 0.0), (40.0, 0.0)]);
}

/// cell 内 va≠baseline 原子 run 不折行竖叠（img-swatch 条带表形态）。
/// 旧复用臂全 baseline 门把定位同步整体跳过，img 滞留 taffy 块堆叠位；
/// 尺寸一致时消费 IFC 存储行位（va 偏移布局期已施加，CSS2 §9.4.2/#inline-formatting）。
#[test]
fn r5261_td_valign_bottom_imgs_flow_inline() {
    let pos = img_positions(
        r#"<html><body style="margin:0"><table><tr><td><img style="width:20px;height:20px;vertical-align:bottom"><img style="width:20px;height:20px;vertical-align:bottom"><img style="width:20px;height:20px;vertical-align:bottom"></td></tr></table></body></html>"#,
    );
    assert_eq!(
        pos,
        vec![(0.0, 0.0), (20.0, 0.0), (40.0, 0.0)],
        "R5261: cell 内 va≠baseline 原子 run 应共享行盒（禁竖叠）"
    );
}
