//! R5185 回归：background-clip:border-area × border-shape 带环（css-backgrounds-4
//! §2.1 × css-borders-4 §border-shape）。
//!
//! 旧行为：border-area 的形状元素背景可见区错用填充多边形（R5184 stroke = 路径、
//! fill = 内形状）——环带外半被裁掉、带内 padding 区漏出（clip-border-area-border-
//! shape-background-position 11.33% 判例）；矩形环带条带（R4529）截掉 fill mode
//! 环带内区段。R5185：带环 = even-odd 外+内串联（stroke = 凸路径 ±半宽 miter 偏移，
//! fill = 外/内形状），背景图像/底色统一裁到带环。

use super::super::border_shape::offset_convex_polygon;
use super::super::painter::Painter;
use zero_style_system::StyleSystem;

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

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.6
}

/// 矩形偏移精确：inset(0) 路径 (5,5,190,90) ±5 → 外 (0,0,200,100) / 内 (10,10,180,80)
///（background-position 判例带环 = border-box 减 padding-box）。
#[test]
fn test_r5185_offset_convex_polygon_rect() {
    let rect = vec![(5.0, 5.0), (195.0, 5.0), (195.0, 95.0), (5.0, 95.0)];
    let outer = offset_convex_polygon(&rect, 5.0).expect("rect 凸，偏移应成功");
    assert_eq!(outer.len(), 4);
    for (x, y) in &outer {
        assert!(near(*x, 0.0) || near(*x, 200.0), "外圈 x∈{{0,200}}：{outer:?}");
        assert!(near(*y, 0.0) || near(*y, 100.0), "外圈 y∈{{0,100}}：{outer:?}");
    }
    let inner = offset_convex_polygon(&rect, -5.0).expect("rect 凸，内缩应成功");
    for (x, y) in &inner {
        assert!(near(*x, 10.0) || near(*x, 190.0), "内圈 x∈{{10,190}}：{inner:?}");
        assert!(near(*y, 10.0) || near(*y, 90.0), "内圈 y∈{{10,90}}：{inner:?}");
    }
}

/// 菱形 miter 偏移：顶点 T(80,0) 内角 θ ≈ 106.26°（sin(θ/2) = 0.8），miter 顶点 =
/// T 沿角平分线外延 d/sin(θ/2)——d=12 外圈顶点 (80,−15)、内缩 (80,15)（join =
/// 偏移边交点，clip-border-area 判例 diamond 几何）。
#[test]
fn test_r5185_offset_convex_polygon_diamond_miter() {
    let diamond = vec![(80.0, 0.0), (160.0, 60.0), (80.0, 120.0), (0.0, 60.0)];
    let outer = offset_convex_polygon(&diamond, 12.0).expect("凸菱形应成功");
    assert!(
        outer.iter().any(|(x, y)| near(*x, 80.0) && near(*y, -15.0)),
        "外圈顶缘 miter 顶点 (80,−15)：{outer:?}"
    );
    let inner = offset_convex_polygon(&diamond, -12.0).expect("凸菱形内缩应成功");
    assert!(
        inner.iter().any(|(x, y)| near(*x, 80.0) && near(*y, 15.0)),
        "内圈顶缘 miter 顶点 (80,15)：{inner:?}"
    );
}

/// 尖刺拒绝：顶点 (0,10) 内角 ≈ 1°（miter 比 1/sin(0.5°) ≫ 4）→ None。
#[test]
fn test_r5185_offset_needle_rejected() {
    let needle = vec![(0.0, 10.0), (100.0, 0.0), (100.0, 1.0)];
    assert!(offset_convex_polygon(&needle, 5.0).is_none(), "尖刺角应超 miter 比拒绝");
}

/// 非凸（L 形，cross 变号）拒绝 → None（调用方回退既有行为，不出错带）。
#[test]
fn test_r5185_offset_concave_rejected() {
    let l_shape = vec![
        (0.0, 0.0),
        (100.0, 0.0),
        (100.0, 50.0),
        (50.0, 50.0),
        (50.0, 100.0),
        (0.0, 100.0),
    ];
    assert!(offset_convex_polygon(&l_shape, 5.0).is_none(), "L 形应拒绝");
}

/// e2e：border-area 渐变 + inset(0) → 带环可见（padding 中心区不涂）。
/// 旧行为（填充窗）渐变裁到中线路径矩形 (5,5,90,40)——中心 (50,30) 被涂。
#[test]
fn test_r5185_border_area_gradient_band_excludes_padding_center() {
    let css = "body { margin: 0 } #t { width: 100px; height: 60px; box-sizing: border-box; \
               border: 10px solid transparent; \
               background-image: linear-gradient(blue, blue); background-clip: border-area; \
               border-shape: inset(0); }";
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

    // 中心 (50,30)（padding 区）不应有任何蓝色覆盖：fill 矩形、渐变（含 clip 窗）、
    // path_fill（带环 even-odd 探针）逐一检查。
    for f in &prims.fills {
        let r = f.rect;
        let covers = r.origin.x <= 50.0
            && r.origin.y <= 30.0
            && r.origin.x + r.size.width > 50.0
            && r.origin.y + r.size.height > 30.0;
        assert!(
            !(covers && f.color.b >= 250 && f.color.r < 90),
            "蓝色 fill 覆盖 padding 中心：{:?}",
            r
        );
    }
    for g in &prims.gradients {
        let (r, clip) = (g.rect, g.clip);
        let (x0, y0, x1, y1) = match clip {
            Some(c) => (
                c.origin.x.max(r.origin.x),
                c.origin.y.max(r.origin.y),
                (c.origin.x + c.size.width).min(r.origin.x + r.size.width),
                (c.origin.y + c.size.height).min(r.origin.y + r.size.height),
            ),
            None => (
                r.origin.x,
                r.origin.y,
                r.origin.x + r.size.width,
                r.origin.y + r.size.height,
            ),
        };
        assert!(
            !(x0 <= 50.0 && y0 <= 30.0 && x1 > 50.0 && y1 > 30.0),
            "蓝色渐变（含窗）覆盖 padding 中心：rect {r:?} clip {clip:?}"
        );
    }
    for p in &prims.path_fills {
        if !(p.color.b >= 250 && p.color.r < 90) {
            continue;
        }
        let xs = scan_crossings(&p.vertices, 30.0);
        let covered = xs.chunks(2).any(|c| c.len() == 2 && c[0] <= 50.0 && c[1] > 50.0);
        assert!(!covered, "蓝色 path_fill y=30 行覆盖 x=50（应为环带双段不达中心）");
    }
}

/// e2e：border-area 底色 + inset(0) → 带环 path_fill，y=30 探针交点恰
/// [0,10,90,100]（外 (0,0,100,60) 内 (10,10,80,40)；填充窗旧行为 = [5,95]）。
#[test]
fn test_r5185_border_area_color_band_ring_probe() {
    let css = "body { margin: 0 } #t { width: 100px; height: 60px; box-sizing: border-box; \
               border: 10px solid transparent; background-color: blue; \
               background-clip: border-area; border-shape: inset(0); }";
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

    let blue: Vec<&Vec<f32>> = prims
        .path_fills
        .iter()
        .filter(|p| p.color.b >= 250 && p.color.r < 90)
        .map(|p| &p.vertices)
        .collect();
    assert_eq!(blue.len(), 1, "底色应为单条带环 path_fill，got {}", blue.len());
    let xs = scan_crossings(blue[0], 30.0);
    assert!(
        xs.len() == 4 && near(xs[0], 0.0) && near(xs[1], 10.0) && near(xs[2], 90.0) && near(xs[3], 100.0),
        "y=30 带环交点应为 [0,10,90,100]，got {xs:?}"
    );
}
