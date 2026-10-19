//! R5188 回归：随形 box-shadow——border-shape 活跃时零模糊阴影随形状（inset 环带
//! 随描边带内缘 / outset 平移外扩 + punch-out；css-borders-4 §border-shape，判例
//! two-shapes-shadow / combined-shadow / shadow-transparent），及 border-radius
//! outset 阴影随圆角对称臂（css-backgrounds-3 §7.1，chromium oracle
//! box-shadow-radius-002 判例；box-shadow-border-radius-001 / css-box-shadow-001）。

use super::super::border_shape::border_shape_shadow_ring;
use super::super::painter::Painter;
use zero_layout_engine::LayoutBox;
use zero_style_system::{ComputedStyle, StyleSystem};

/// 水平扫描线 even-odd 边交点（严格 > 比较避开顶点行歧义；末点→首点显式闭合）。
fn scan_crossings(flat: &[f32], y: f32) -> Vec<f32> {
    let pts: Vec<(f32, f32)> = flat.chunks(2).map(|c| (c[0], c[1])).collect();
    let n = pts.len();
    let mut xs = Vec::new();
    for i in 0..n {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % n];
        if (y1 > y) != (y2 > y) {
            xs.push(x1 + (x2 - x1) * (y - y1) / (y2 - y1));
        }
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    xs
}

fn has_vert(verts: &[f32], px: f32, py: f32) -> bool {
    verts
        .chunks(2)
        .any(|c| (c[0] - px).abs() < 0.1 && (c[1] - py).abs() < 0.1)
}

/// body{margin:0} 下渲染单 #t 页面（元素贴原点，坐标断言直读），返回 (ComputedStyle, LayoutBox)。
fn setup(css: &str) -> (ComputedStyle, LayoutBox) {
    let css = format!("body {{ margin: 0 }} {css}");
    let html = r#"<html><head><style></style></head><body><div id="t"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(&css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    fn find_id<'a>(box_node: &'a LayoutBox, doc: &zero_dom::Document) -> Option<&'a LayoutBox> {
        let matched = box_node
            .node_id
            .and_then(|nid| {
                doc.get(nid).and_then(|node| match &node.kind {
                    zero_dom::NodeKind::Element(el) => el.get_attribute("id").map(|v| v == "t"),
                    _ => None,
                })
            })
            .unwrap_or(false);
        if matched {
            return Some(box_node);
        }
        box_node.children.iter().find_map(|c| find_id(c, doc))
    }
    let found = find_id(&result.root, &doc).expect("#t 盒应存在").clone();
    let style = styles
        .get(&found.node_id.expect("#t 盒应有 node_id"))
        .cloned()
        .expect("#t 应有样式");
    (style, found)
}

/// 判例 two-shapes-shadow：fill mode circle(50%) circle(25%) outset 阴影
/// `10px 10px 0 5px` → 阴影形状 = 外圆平移 (10,10) 外扩 5（r105 @ (110,110)），
/// punch-out = 外圆自身（r100 @ (100,100)）。y=110 行 even-odd 段 = 基圆独有月牙
/// [0.5..5] 与阴影独有月牙 [199.5..215]。
#[test]
fn test_r5188_two_shapes_outset_ring_row() {
    let (style, box_node) = setup("#t { width: 200px; height: 200px; border-shape: circle(50%) circle(25%); }");
    let ring = border_shape_shadow_ring(&style, &box_node, 0.0, 0.0, false, 10.0, 10.0, 5.0)
        .expect("fill mode outset 环带应可得");
    let flat: Vec<f32> = ring.iter().flat_map(|&(x, y)| [x, y]).collect();
    let xs = scan_crossings(&flat, 110.0);
    assert_eq!(xs.len(), 4, "y=110 应 4 交点：{xs:?}");
    for (got, want) in xs.iter().zip([0.5, 5.0, 199.5, 215.0]) {
        assert!((got - want).abs() < 0.2, "交点 {got} 应 ≈ {want}：{xs:?}");
    }
}

/// 判例 two-shapes-shadow inset 半：`0 0 0 5px purple inset` → 环带 = 内形状（r50
/// 圆）与再内缩 spread（r45）之间。y=100 行段 = [50..55] 与 [145..150]。
#[test]
fn test_r5188_two_shapes_inset_ring_row() {
    let (style, box_node) = setup("#t { width: 200px; height: 200px; border-shape: circle(50%) circle(25%); }");
    let ring =
        border_shape_shadow_ring(&style, &box_node, 0.0, 0.0, true, 0.0, 0.0, 5.0).expect("fill mode inset 环带应可得");
    let flat: Vec<f32> = ring.iter().flat_map(|&(x, y)| [x, y]).collect();
    let xs = scan_crossings(&flat, 100.0);
    assert_eq!(xs.len(), 4, "y=100 应 4 交点：{xs:?}");
    for (got, want) in xs.iter().zip([50.0, 55.0, 145.0, 150.0]) {
        assert!((got - want).abs() < 0.2, "交点 {got} 应 ≈ {want}：{xs:?}");
    }
}

/// 判例 combined-shadow：stroke mode 全盒形状（ZW 解析 inset(0)；xywh 未入解析）+ border 10 + outset
/// `10px 20px 0 10px` → 阴影形状自边框墨迹外缘（路径外扩 width/2）起量：base =
/// (0,0,220,220)（border-box），外扩 10 平移 (10,20) → (0..240)×(10..250)。
#[test]
fn test_r5188_stroke_outset_from_border_outer_edge() {
    let (style, box_node) = setup(
        "#t { width: 200px; height: 200px; border: 10px solid black; \
         border-shape: inset(0); }",
    );
    let ring = border_shape_shadow_ring(&style, &box_node, 0.0, 0.0, false, 10.0, 20.0, 10.0)
        .expect("stroke mode outset 环带应可得");
    let flat: Vec<f32> = ring.iter().flat_map(|&(x, y)| [x, y]).collect();
    // 移动外圈角 (0,10)/(240,250)；punch 基圈角 (0,0)/(220,220)。
    assert!(has_vert(&flat, 0.0, 10.0), "移动外圈 TL (0,10)：{ring:?}");
    assert!(has_vert(&flat, 240.0, 250.0), "移动外圈 BR (240,250)：{ring:?}");
    assert!(has_vert(&flat, 0.0, 0.0), "punch 基圈 TL (0,0)：{ring:?}");
    assert!(has_vert(&flat, 220.0, 220.0), "punch 基圈 BR (220,220)：{ring:?}");
}

/// 内缩退化：inset spread 60 超内切半径（r50 圆）→ 偏移多边形翻转 → None（回退
/// 方环）。负 spread 与带偏移 inset 同不进形状路径。
#[test]
fn test_r5188_inset_degenerate_and_guard_none() {
    let (style, box_node) = setup("#t { width: 200px; height: 200px; border-shape: circle(50%) circle(25%); }");
    assert!(
        border_shape_shadow_ring(&style, &box_node, 0.0, 0.0, true, 0.0, 0.0, 60.0).is_none(),
        "spread 60 > 内切半径应 None"
    );
    assert!(
        border_shape_shadow_ring(&style, &box_node, 0.0, 0.0, true, 0.0, 0.0, -5.0).is_none(),
        "负 spread inset 应 None"
    );
    assert!(
        border_shape_shadow_ring(&style, &box_node, 0.0, 0.0, true, 5.0, 0.0, 5.0).is_none(),
        "带偏移 inset 应 None"
    );
}

/// R5180 对称臂：border-radius 50px + `0 0 0 20px black`（透明背景）→ 黑 path_fill
/// 环 = 圆 r70 − 圆 r50（box-shadow-border-radius-001 / css-box-shadow-001 /
/// shadow-transparent 判例；chromium oracle box-shadow-radius-002 同语义）。
#[test]
fn test_r5188_radius_outset_shadow_rounded_ring() {
    let css = "body { margin: 0 } #t { width: 100px; height: 100px; border-radius: 50px; \
               box-shadow: 0 0 0 20px black; background: transparent; }";
    let html = r#"<html><head><style></style></head><body><div id="t"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));
    let prims = painter.into_primitives();

    let rings: Vec<&Vec<f32>> = prims
        .path_fills
        .iter()
        .filter(|p| p.color.r < 30 && p.color.g < 30 && p.color.b < 30)
        .map(|p| &p.vertices)
        .collect();
    assert_eq!(rings.len(), 1, "黑环应为单条 path_fill，got {}", rings.len());
    let xs = scan_crossings(rings[0], 50.0);
    assert_eq!(xs.len(), 4, "y=50 应 4 交点：{xs:?}");
    for (got, want) in xs.iter().zip([-20.0, 0.0, 100.0, 120.0]) {
        assert!((got - want).abs() < 0.2, "交点 {got} 应 ≈ {want}：{xs:?}");
    }
}

/// corner-shape 非初始（bevel 声明）→ 随形角阴影域未实现，回退 ShadowPrimitive
/// 方环（无黑 path_fill，有 shadow 图元）——false-green 对（corner-shape-overflow-
/// clip-margin）维持基线，不出错误圆角。
#[test]
fn test_r5188_corner_shape_non_round_outset_falls_back() {
    let css = "body { margin: 0 } #t { width: 100px; height: 100px; border-radius: 50px; \
               corner-shape: bevel; box-shadow: 0 0 0 20px black; }";
    let html = r#"<html><head><style></style></head><body><div id="t"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    let mut engine = zero_layout_engine::LayoutEngine::new(800.0, 600.0);
    let result = engine.compute(&doc, &styles);
    let mut painter = Painter::new();
    painter.paint(&result.root, &styles, Some(&doc));
    let prims = painter.into_primitives();
    assert!(
        !prims
            .path_fills
            .iter()
            .any(|p| p.color.r < 30 && p.color.g < 30 && p.color.b < 30),
        "corner-shape 非初始不应出黑 path_fill 环"
    );
    assert!(!prims.shadows.is_empty(), "应回退 ShadowPrimitive 方环");
}
