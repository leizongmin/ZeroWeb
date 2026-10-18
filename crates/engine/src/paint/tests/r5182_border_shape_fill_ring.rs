//! R5182 回归：border-shape fill mode 环带 keyhole 串联（css-borders-4 §7）。
//!
//! 旧行为：Fill 臂朴素 `outer.extend(inner)`——外末→内首直连桥边与闭合边以不同
//! x 交叉扫描线，环带内撕出背景色隙 / 孔内挤出楔形（R5180 corner-shape-square
//! 同根因；geometry-box corpus 案 1.40% 判例）。R5180 的闭合边切点原仅支持
//! shaped 多边形（首末同在左缘竖边），用户 polygon 顶点序任意 → 切点通用化为
//! 闭合边真中点（首末同 x 的既有调用方逐位不变）。

use super::super::helpers::{concat_ring_vertices, cut_polygon_at_closing_edge};
use super::super::painter::Painter;
use zero_style_system::StyleSystem;

/// 水平扫描线 even-odd 边交点（严格 > 比较避开顶点行歧义，水平边不计；末点→
/// 首点显式闭合）。
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

/// 切点通用化：任意顶点序（TR 起始 CW 矩形）闭合边 = 末点 (10,10)→首点 (90,10)，
/// 切点 = 中点 (50,10)；首末同 x（左缘竖边）切点 x 不变（R5180 判例逐位一致）。
#[test]
fn test_r5182_cut_at_closing_edge_midpoint() {
    let user = vec![(90.0, 10.0), (90.0, 90.0), (10.0, 90.0), (10.0, 10.0)];
    let cut = cut_polygon_at_closing_edge(&user);
    assert_eq!(cut.len(), 6, "n + 首尾切点");
    assert_eq!(cut[0], (50.0, 10.0), "切点 = 闭合边中点");
    assert_eq!(cut[5], (50.0, 10.0), "首尾切点重复");
    assert_eq!(&cut[1..5], &user[..], "原顶点序保持");

    let shaped = vec![(0.0, 25.0), (100.0, 0.0), (100.0, 100.0), (0.0, 75.0)];
    let cut = cut_polygon_at_closing_edge(&shaped);
    assert_eq!(cut[0], (0.0, 50.0), "首末同 x → 中点 x 不变");
}

/// 带完整性探针：外 (0,0,100,100) 内 (10,10,90,90) 矩形环（用户顶点序），y=50
/// 扫描线交点应恰为 [0,10,90,100]（环带两段）。朴素 extend 的两条对角桥边在
/// y=50 同交 x≈5.56 → 交点 [5.56,5.56,90,100]：左带整体缺失 + 孔内楔形。
#[test]
fn test_r5182_ring_concat_no_tear_at_cut_row() {
    let outer = vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0)];
    let inner = vec![(10.0, 10.0), (90.0, 10.0), (90.0, 90.0), (10.0, 90.0)];
    let ring = concat_ring_vertices(outer, inner);
    let flat: Vec<f32> = ring.iter().flat_map(|(x, y)| [*x, *y]).collect();
    let xs = scan_crossings(&flat, 50.0);
    let near = |a: f32, b: f32| (a - b).abs() < 0.6;
    assert!(
        xs.len() == 4 && near(xs[0], 0.0) && near(xs[1], 10.0) && near(xs[2], 90.0) && near(xs[3], 100.0),
        "y=50 交点应为 [0,10,90,100]（环带两段），got {xs:?}"
    );
}

/// Fill 臂路由判例：border-shape 双 polygon（border-box）+ solid 边 → 恰一条红
/// path_fill 且过 y=50 探针（回归朴素 extend 即断言红）。
#[test]
fn test_r5182_border_shape_fill_routes_through_keyhole_concat() {
    let css = "body { margin: 0 } #t { width: 100px; height: 100px; box-sizing: border-box; \
               border: 5px solid red; \
               border-shape: polygon(0% 0%, 100% 0%, 100% 100%, 0% 100%) border-box \
                             polygon(10% 10%, 90% 10%, 90% 90%, 10% 90%) border-box; }";
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

    let red: Vec<&Vec<f32>> = prims
        .path_fills
        .iter()
        .filter(|p| p.color.r >= 250 && p.color.g < 90)
        .map(|p| &p.vertices)
        .collect();
    assert_eq!(red.len(), 1, "fill mode 应为单条红 path_fill，got {}", red.len());
    let xs = scan_crossings(red[0], 50.0);
    let near = |a: f32, b: f32| (a - b).abs() < 0.6;
    assert!(
        xs.len() == 4 && near(xs[0], 0.0) && near(xs[1], 10.0) && near(xs[2], 90.0) && near(xs[3], 100.0),
        "渲染环 y=50 交点应为 [0,10,90,100]，got {xs:?}"
    );
}
