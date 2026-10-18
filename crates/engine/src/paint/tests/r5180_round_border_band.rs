//! R5180 回归：plain border-radius 弧带（css-backgrounds-3 §5.5）。
//!
//! 旧行为：常规 radius+border 的边框带按四边直角条带绘制（背景圆、边带直角），
//! self-source corpus 两侧同错自洽掩盖（learnings「border-radius-band-sharp-strips」）。
//!
//! 新行为：非 collapse、四边 Solid 同色等宽、有非零半径时，外圈（规范口径解析）
//! 与内圈（padding edge max(0, r−bw) 同心）串联单次 even-odd path_fill；其余维持
//! 条带。kill-switch `ZW_ROUND_BORDER_BAND=0` 恢复条带（OFF 反证由 corpus 承担）。

use super::super::helpers::rounded_corner_polygon_in_box;
use super::super::painter::Painter;
use zero_style_system::StyleSystem;

/// 顶点扁平数组中是否存在 ≈(px,py)（容差 0.6px）。
fn has_vert(verts: &[f32], px: f32, py: f32) -> bool {
    verts
        .chunks(2)
        .any(|c| (c[0] - px).abs() < 0.6 && (c[1] - py).abs() < 0.6)
}

/// plain border-radius 多边形：与 shaped Round 臂同一 8 段采样——100×100 r25 的
/// TL start = (0,25)，采样点落在以 C=(25,25) 为心的圆上（弦线中点在弧内侧）。
#[test]
fn test_r5180_rounded_polygon_arc_samples() {
    let pts = rounded_corner_polygon_in_box(0.0, 0.0, 100.0, 100.0, [25.0; 4]);
    assert_eq!(pts.len(), 36, "四角 × [start, 7 采样, end]，got {pts:?}");
    assert_eq!(pts[0], (0.0, 25.0), "TL start = 左缘端点");
    for (x, y) in &pts[1..8] {
        let d = (((*x - 25.0) as f64).powi(2) + ((*y - 25.0) as f64).powi(2)).sqrt();
        assert!(
            (d - 25.0).abs() < 1e-3,
            "round 采样点应落在 C=(25,25) r=25 圆上，got ({x},{y}) d={d}"
        );
    }
}

/// 弧带管线判例：100×100 r25 border 5px solid red（bg 默认透明）→ 恰一条红
/// path_fill 同时含外圈弧中点 (7.32,7.32) 与内圈弧中点 (10.86,10.86)（even-odd
/// 环）；旧条带路径红边全为 add_fill 矩形（无 path_fill），此断言即回归探针。
#[test]
fn test_r5180_round_band_emits_even_odd_ring() {
    let css = "body { margin: 0 } #t { width: 100px; height: 100px; border-radius: 25px; \
               border: 5px solid red; box-sizing: border-box; }";
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
    assert_eq!(red.len(), 1, "弧带应为单条红 path_fill，got {}", red.len());
    let ring = red[0];
    // 外圈 TL 弧 8 段中点 = C(25,25) + 225°·25 ≈ (7.32,7.32)；内圈 r−bw=20 同心
    // （圆心不变 (25,25)）→ 中点 ≈ (25,25) − 20/√2 ≈ (10.86,10.86)。
    assert!(has_vert(ring, 7.32, 7.32), "环应含外圈弧中点，got {ring:?}");
    assert!(
        has_vert(ring, 10.86, 10.86),
        "环应含内圈（r−bw=20 同心）弧中点，got {ring:?}"
    );
    // 内圈起点 = 内盒角 (5,5) + r_inner 20 → (5,25)；外圈起点 (0,25) 同环在场。
    assert!(
        has_vert(ring, 0.0, 25.0) && has_vert(ring, 5.0, 25.0),
        "环应含外/内圈 TL 起点"
    );
}

/// 门控反例：dotted 边框（非 Solid）不进弧带，维持条带（无红 path_fill）。
#[test]
fn test_r5180_dotted_falls_back_to_strips() {
    let css = "body { margin: 0 } #t { width: 100px; height: 100px; border-radius: 25px; \
               border: 5px dotted red; box-sizing: border-box; }";
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
        !prims.path_fills.iter().any(|p| p.color.r >= 250 && p.color.g < 90),
        "dotted 不应产生弧带 path_fill"
    );
}
