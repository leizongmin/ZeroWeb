//! R5186 回归：border-shape 描边 miter join 楔形 + fill mode 零宽边框环带放行 +
//! outline 随形状（css-borders-4 §border-shape × css-ui §outline）。
//!
//! 旧行为：CPU path stroke 逐段粗线无 join 几何（尖角丢 miter 尖刺，polygon-
//! miter-limit 判例）；绘制入口仅在布局边框宽 > 0 时进入边框绘制（border-style
//! none 的 fill mode 判例 inner-outer 环带整条缺失）；outline 恒矩形（outline /
//! outline-double 判例圆形状外红环画成方框）。

use super::super::border_shape::{border_shape_border_path_active, stroke_miter_wedges};
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

/// miter 楔形 = 转向外侧两偏移点与尖点围成的三角形。矩形四顶点各一楔，尖点 =
/// 外推角点（局部转向定外侧，cross>0 左转 → 右法线族）。
#[test]
fn test_r5186_miter_wedges_rect() {
    let rect = vec![(10.0, 10.0), (110.0, 10.0), (110.0, 60.0), (10.0, 60.0)];
    let wedges = stroke_miter_wedges(&rect, 10.0);
    assert_eq!(wedges.len(), 4, "矩形 4 顶点各一楔形");
    // 顶点 (10,10)：入边 (10,60)→(10,10) 方向 (0,−1)，出边 →(110,10) 方向 (1,0)，
    // cross>0 左转 → 外侧 = 右法线族：(−1,0) 与 (0,−1) → A=(5,10) B=(10,5)，
    // 尖点 M = (5,5)。
    let w0 = &wedges[0];
    let has_tip = w0.iter().any(|&(x, y)| (x - 5.0).abs() < 0.1 && (y - 5.0).abs() < 0.1);
    assert!(has_tip, "顶点 (10,10) 楔形应含尖点 (5,5)，实际 {w0:?}");
}

/// 尖角 miter 尖刺：bowtie 顶点 (0,0)（楔形张开角 ≈23.7°）尖距 = h/sin(θ/2)
/// ≈ 24.3（w=10，h=5）；方向沿外侧角平分线（自交路径局部转向判定，不依赖全局
/// 有向面积——bowtie shoelace 面积恰为 0）。
#[test]
fn test_r5186_miter_wedges_bowtie_spike() {
    let bowtie = vec![(0.0, 0.0), (66.0, 150.0), (66.0, 0.0), (0.0, 150.0)];
    let wedges = stroke_miter_wedges(&bowtie, 10.0);
    assert_eq!(wedges.len(), 4, "bowtie 4 顶点各一楔形（交叉处无顶点）");
    let w0 = &wedges[0];
    let tip = w0
        .iter()
        .copied()
        .find(|&(x, y)| x < -0.5 && y < 0.0)
        .expect("顶点 (0,0) 楔形应有向上偏左尖点");
    let dist = (tip.0 * tip.0 + tip.1 * tip.1).sqrt();
    assert!((dist - 24.3).abs() < 0.5, "尖距 = h/sin(θ/2) ≈ 24.3，实际 {dist}");
}

/// 近平行边（共线段）与零宽跳过：退化输入不发楔形。
#[test]
fn test_r5186_miter_wedges_degenerate() {
    let collinear = vec![(0.0, 0.0), (50.0, 0.0), (100.0, 0.0)];
    assert!(stroke_miter_wedges(&collinear, 10.0).is_empty(), "共线多边形无楔形");
    let rect = vec![(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (0.0, 10.0)];
    assert!(stroke_miter_wedges(&rect, 0.0).is_empty(), "零宽无楔形");
}

/// 形状活跃判定：border-shape 解析成功即放行（fill mode 环带宽与 border 宽解耦）。
#[test]
fn test_r5186_border_path_active() {
    let html = r#"<html><head><style></style></head><body>
        <div style="border-shape: circle(50%)"></div>
        <div style="color: red"></div>
    </body></html>"#;
    let doc = zero_dom::parse_html(html);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[]);
    let mut active = 0;
    let mut plain = 0;
    for (nid, style) in &styles {
        if let Some(node) = doc.get(*nid)
            && let zero_dom::NodeKind::Element(el) = &node.kind
            && el.local_name() == "div"
        {
            if border_shape_border_path_active(style) {
                active += 1;
            } else {
                plain += 1;
            }
        }
    }
    assert_eq!(active, 1, "circle 形状活跃 → 边框路径可绘");
    assert_eq!(plain, 1, "无形状 → 不放行");
}

/// outline 随形状：circle 形状 + outline 4px solid → 环带 path_fill，y=圆心行
/// 交点外沿 = [46, 154]（带 = 路径外向 [0, width]，ref 判例 outline 内缘 =
/// 形状路径：circle r50 → 红环 [50,54]，x 极值 46/154）。
#[test]
fn test_r5186_outline_follows_shape_ring() {
    let css = "body { margin: 0 } #t { width: 100px; height: 100px; background: lightblue; \
               border-shape: circle(50px at 50px 50px); outline: 4px solid red; }";
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

    let reds: Vec<_> = prims
        .path_fills
        .iter()
        .filter(|p| p.color.r >= 200 && p.color.g < 80 && p.color.b < 80)
        .collect();
    assert_eq!(reds.len(), 1, "恰一红色环带 path_fill，实际 {}", reds.len());
    let xs = scan_crossings(&reds[0].vertices, 50.0);
    assert!(
        xs.len() == 4
            && (xs[0] + 4.0).abs() < 0.6
            && xs[1].abs() < 0.6
            && (xs[2] - 100.0).abs() < 0.6
            && (xs[3] - 104.0).abs() < 0.6,
        "y=50 行环带交点 = [-4,0,100,104]（圆心 50 r50 外扩 4），实际 {xs:?}"
    );
}

/// outline double 随形状：9px double → 两条子带（w/3 三等分，两线 + 中隙），
/// y=圆心行内带环交点 [47,50,150,153]（环 r50..53）、外带环 [41,44,156,159]
///（环 r56..59；ref 判例 outline-double r51.5/r57.5 ± 1.5）。
#[test]
fn test_r5186_outline_double_shape_bands() {
    let css = "body { margin: 0 } #t { width: 100px; height: 100px; background: lightblue; \
               border-shape: circle(50px at 50px 50px); outline: 9px double red; }";
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

    let reds: Vec<_> = prims
        .path_fills
        .iter()
        .filter(|p| p.color.r >= 200 && p.color.g < 80 && p.color.b < 80)
        .collect();
    assert_eq!(reds.len(), 2, "double = 两条子带 path_fill，实际 {}", reds.len());
    let near = |a: f32, b: f32| (a - b).abs() < 0.6;
    let mut has_inner = false;
    let mut has_outer = false;
    for p in &reds {
        let xs = scan_crossings(&p.vertices, 50.0);
        assert_eq!(xs.len(), 4, "每条子带 y=50 行 4 交点，实际 {xs:?}");
        // 内带环 r50..53：x ∈ [-3,0,100,103]；外带环 r56..59：x ∈ [-9,-6,106,109]。
        if near(xs[0], -3.0) && near(xs[1], 0.0) && near(xs[2], 100.0) && near(xs[3], 103.0) {
            has_inner = true;
        }
        if near(xs[0], -9.0) && near(xs[1], -6.0) && near(xs[2], 106.0) && near(xs[3], 109.0) {
            has_outer = true;
        }
    }
    assert!(has_inner, "缺内带环 [-3,0,100,103]");
    assert!(has_outer, "缺外带环 [-9,-6,106,109]");
}
