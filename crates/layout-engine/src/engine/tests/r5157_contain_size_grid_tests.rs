//! R5157（css-contain-1 §3.1 × css-align-3 gap legacy alias）：contain:size 的 grid
//! 容器按「无内容」尺寸化时 track 几何仍参与——grid-template-columns/rows 与 gap 是
//! 尺寸声明非内容。此前两处缺口：① R4034b containment 抑制臂把 contained abs grid
//! 塌成 0×0（shrink-to-fit 直接 continue）；② `grid-gap` legacy 别名未 canonical 化，
//! computed row_gap/column_gap 全落初始值（style-system shorthand 层修复，有独立单测）。
//! driving: css/css-contain/contain-size-grid-002（font-size:800px 令 auto track 经
//! 内容路径污染 → 修复后必为 100×100 纯绿）。kill-switch `ZW_GRID_CONTAIN_SIZE_TRACKS=0`。

use crate::engine::LayoutEngine;
use crate::types::LayoutBox;
use zero_css_parser::Parser;
use zero_dom::NodeId;
use zero_style_system::StyleSystem;

/// 返回 #test 盒几何 (x, y, w, h)。
fn layout_test_box(html: &str, css: &str) -> (f32, f32, f32, f32) {
    let doc = zero_dom::parse_html(html);
    let stylesheets = vec![Parser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let id = doc.get_element_by_id("test").unwrap();
    fn find(root: &LayoutBox, id: NodeId) -> Option<(f32, f32, f32, f32)> {
        let mut stack = vec![root];
        while let Some(b) = stack.pop() {
            if b.node_id == Some(id) {
                return Some((b.x, b.y, b.width, b.height));
            }
            stack.extend(b.children.iter());
        }
        None
    }
    find(&result.root, id).expect("#test 无布局盒")
}

/// corpus case 形：contained abs grid（grid-gap 20 + auto 80px 列 / 40px×2 行）应为
/// 100×100——auto track 贡献 0（containment 抑制内容）、gap 计 n-1 次、definite px
/// track 计入；无 CIS 声明故 track 几何是唯一尺寸来源。
#[test]
fn r5157_contained_abs_grid_tracks_and_gap_extent() {
    let (_x, _y, w, h) = layout_test_box(
        r#"<html><body style="margin:0"><div id="test">&nbsp;</div></body></html>"#,
        r#"div { position: absolute; }
#test { background: green; contain: size; display: grid; grid-gap: 20px;
grid-template-columns: auto 80px; grid-template-rows: 40px 40px; font-size: 800px; }"#,
    );
    assert!((w - 100.0).abs() < 0.5, "宽应 0+20+80=100，实际 {w}");
    assert!((h - 100.0).abs() < 0.5, "高应 40+20+40=100，实际 {h}");
}

/// gap 别名回归守：普通（无 containment）grid 的 `grid-gap` 同样生效——两列 50px +
/// gap 20，第二列单元 x 应落在 70（此前别名断线 gap 落 0 时 x=50）。
#[test]
fn r5157_grid_gap_alias_applies_to_plain_grid() {
    let doc = zero_dom::parse_html(
        r#"<html><body style="margin:0"><div id="test"><span>a</span><span>b</span></div></body></html>"#,
    );
    let css = r#"#test { display: grid; grid-gap: 20px; grid-template-columns: 50px 50px; }
span { height: 20px; }"#;
    let stylesheets = vec![Parser::parse_stylesheet(css)];
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &stylesheets);
    let mut engine = LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let id = doc.get_element_by_id("test").unwrap();
    fn find(root: &LayoutBox, id: NodeId) -> Option<&LayoutBox> {
        if root.node_id == Some(id) {
            return Some(root);
        }
        root.children.iter().find_map(|c| find(c, id))
    }
    let second = find(&result.root, id)
        .and_then(|g| g.children.get(1))
        .expect("第二列单元缺布局盒");
    assert!(
        (second.x - 70.0).abs() < 0.5,
        "第二列 x 应 50+20=70（gap 生效），实际 {}",
        second.x
    );
}
