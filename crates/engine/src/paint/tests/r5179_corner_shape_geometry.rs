//! R5179 回归：corner-shape 真几何（css-borders-4 §3.7/§3.9.4.2）。
//!
//! R4248 定界（scoop/负 K superellipse → None 回退 round）摘除，三处修复：
//! 解析层 superellipse(±∞)→Square/Notch 映射 + squircle 关键字；几何层按 K 分
//! regime——凸侧 K≥0 心 = 内角点 C 指数 n=2K、凹侧 K≤−1 心 = 盒角点 P 的
//! T^(1/n) 参数化（scoop ≡ K=−1 圆弧）、−1<K<0 切线交点族；边框环内圈 notch
//! 保留全半径（chromium 内描边语义，阶梯腿 = 内盒角 + R）。
//!
//! kill-switch 回退由 corpus OFF 轮承担（无独立开关；R4248 门控删除，回退需还原
//! helpers.rs 几何 + border.rs 内圈半径 + 解析映射三处）。

use zero_css_parser::values::{CornerShapeKind, CornerShapeValue, parse_corner_shape};
use zero_style_system::StyleSystem;

use super::super::helpers::shaped_corner_polygon_in_box;
use super::super::painter::Painter;

/// 计算 #t 元素样式的最小管线（100×100 盒、四角半径 25，案形族共用）。
fn style_with(shape: &str) -> zero_style_system::ComputedStyle {
    let css = format!("#t {{ width: 100px; height: 100px; border-radius: 25px; corner-shape: {shape}; }}");
    let html = r#"<html><body><div id="t"></div></body></html>"#;
    let doc = zero_dom::parse_html(html);
    let sheet = zero_css_parser::Parser::parse_stylesheet(&css);
    let mut sys = StyleSystem::new();
    sys.set_viewport(800.0, 600.0);
    let styles = sys.compute_styles(&doc, &[sheet]);
    styles
        .iter()
        .find(|(id, _)| doc.get_attribute(**id, "id").as_deref() == Some("t"))
        .map(|(_, s)| s.clone())
        .expect("目标元素样式")
}

/// 顶点扁平数组中是否存在 ≈(px,py)（容差 0.6px）。
fn has_vert(verts: &[f32], px: f32, py: f32) -> bool {
    verts
        .chunks(2)
        .any(|c| (c[0] - px).abs() < 0.6 && (c[1] - py).abs() < 0.6)
}

/// scoop（≡ superellipse(−1)）：采样点全部落在以盒角点 P 为圆心、r 为半径的
/// 圆上（css-borders-4 §3.9.4.2 step 37：K=−1 → n=2，心 = P），且方向自 start
/// 扫到 end（旧实现反向采样成 bowtie 自交多边形——被 R4248 门控掩盖未渲染）。
#[test]
fn test_r5179_scoop_samples_on_circle_at_box_corner() {
    let style = style_with("scoop");
    assert!(!style.corner_shape.is_all_round());
    let pts = shaped_corner_polygon_in_box(&style, 0.0, 0.0, 100.0, 100.0, [25.0; 4]).expect("scoop 出多边形");
    // TL 角段：start=(0,25)，采样 7 点，end=(25,0)。中段点检验圆方程。
    assert_eq!(pts[0], (0.0, 25.0), "TL start = 左缘端点");
    for (x, y) in &pts[1..8] {
        let d = ((*x as f64).powi(2) + (*y as f64).powi(2)).sqrt();
        assert!(
            (d - 25.0).abs() < 1e-3,
            "scoop 采样点应距盒角点 P=(0,0) 25px（圆弧），got ({x},{y}) d={d}"
        );
        assert!(
            *x >= 0.0 && *x <= 25.0 && *y >= 0.0 && *y <= 25.0,
            "采样点应落在 TL 角区，got ({x},{y})"
        );
    }
}

/// superellipse(-100)（K=−100 ≤ −1，n=200 ≥ 64）：深凹曲线收拢为 notch 阶梯——
/// 每角恰 [start, C, end] 三点、中间点 = 内角点 C=(25,25)（曲线对阶梯偏差
/// ≤ 0.38px，低于既有弦高容差；亚像素贴边经无 AA path_fill 会放大成整像素
/// 腿位溢出，见 helpers.rs R5179 注）。
#[test]
fn test_r5179_superellipse_negative_100_collapses_to_notch() {
    let style = style_with("superellipse(-100)");
    let pts = shaped_corner_polygon_in_box(&style, 0.0, 0.0, 100.0, 100.0, [25.0; 4]).expect("负 K 出多边形");
    assert_eq!(pts.len(), 12, "四角 × [start, C, end]，got {pts:?}");
    assert_eq!(pts[1], (25.0, 25.0), "TL 中间点应恰为 C=(25,25)");
}

/// superellipse(0)（≡ bevel）：无中间采样点，四角每角恰 [start, end] 直弦；
/// 与 bevel 关键字输出逐点一致。
#[test]
fn test_r5179_superellipse_zero_equals_bevel() {
    let se0 = style_with("superellipse(0)");
    let bevel = style_with("bevel");
    let a = shaped_corner_polygon_in_box(&se0, 0.0, 0.0, 100.0, 100.0, [25.0; 4]).expect("superellipse(0) 出多边形");
    let b = shaped_corner_polygon_in_box(&bevel, 0.0, 0.0, 100.0, 100.0, [25.0; 4]).expect("bevel 出多边形");
    assert_eq!(a.len(), 8, "四角 × [start,end]，无中间点，got {a:?}");
    assert_eq!(a, b, "superellipse(0) ≡ bevel");
}

/// superellipse(24)（K=24，n=48）：凸侧近直角——采样点贴近盒角点 P（square
/// 案形 TR 角的指数语义；旧实现把 K 当指数用，形状显著偏圆）。
#[test]
fn test_r5179_superellipse_positive_near_square() {
    let style = style_with("superellipse(24)");
    let pts = shaped_corner_polygon_in_box(&style, 0.0, 0.0, 100.0, 100.0, [25.0; 4]).expect("出多边形");
    // 多边形顺时针：TL 9 点（start+7+end）→ TR 9 点自 idx 9 起。
    let tr = &pts[9..17];
    assert_eq!(tr[0], (75.0, 0.0), "TR start = 顶缘端点");
    for (x, y) in &tr[1..8] {
        assert!(
            *x >= 75.0 && *x <= 100.0 && *y >= 0.0 && *y <= 25.0,
            "采样点应落在 TR 角区，got ({x},{y})"
        );
        // 直接断言超椭圆方程：角内坐标 (u,v) 满足 u^n + v^n = r^n（n = 2K = 48）。
        // 比距角点阈值强且与采样参数化无关（凸侧保留极角采样，K 大时轴向
        // 采样点距角点可达 ~2.3px，是采样分布而非曲线偏差）。
        let (u, v) = (*x - 75.0, 25.0 - *y);
        let n = 48.0_f64;
        let lhs = (u as f64).powf(n) + (v as f64).powf(n);
        let rhs = 25.0_f64.powf(n);
        assert!(
            (lhs - rhs).abs() / rhs < 1e-3,
            "K=24 采样点应落在超椭圆 u^48+v^48=r^48 上，got ({x},{y}) lhs={lhs} rhs={rhs}"
        );
    }
}

/// 边框环内圈：notch 案形（100×100 r25 border 5px red bg green，body margin 0）
/// 的内圈多边形阶梯腿 = 内盒角 + 全半径 25 → C=(30,30)（chromium 内描边语义；
/// 旧实现 r−bw=20 → C=(25,25) 腿与外圈重合、沿阶梯腿带宽归零）。
#[test]
fn test_r5179_notch_border_ring_inner_full_radius() {
    let css = "body { margin: 0 } #t { width: 100px; height: 100px; border-radius: 25px; \
               border: 5px solid red; background: green; corner-shape: notch; box-sizing: border-box; }";
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

    // 边框环 = 外圈红 path_fill + 内圈绿 path_fill；背景绿 path_fill（外圈十字）
    // 亦在场——内圈以「含 (30,30) 阶梯点」判别（外圈十字阶梯在 (25,25)）。
    let red: Vec<&Vec<f32>> = prims
        .path_fills
        .iter()
        .filter(|p| p.color.r >= 250 && p.color.g < 90)
        .map(|p| &p.vertices)
        .collect();
    let green: Vec<&Vec<f32>> = prims
        .path_fills
        .iter()
        .filter(|p| p.color.g >= 100 && p.color.r < 90)
        .map(|p| &p.vertices)
        .collect();
    assert!(red.iter().any(|v| has_vert(v, 25.0, 25.0)), "外圈红阶梯应过 C=(25,25)");
    let inner = green.iter().find(|v| has_vert(v, 30.0, 30.0)).unwrap_or_else(|| {
        panic!(
            "内圈绿阶梯应过 C=(30,30)（内盒角 (5,5) + 全半径 25），绿 path 数 {}",
            green.len()
        )
    });
    assert!(
        !has_vert(inner, 25.0, 25.0),
        "内圈不应再出现减边框后的 C=(25,25)，got {inner:?}"
    );
}

/// superellipse(-infinity) 经解析映射为 Notch（计算值携带；几何层走 notch 阶梯
/// + 内圈全半径，与 notch 关键字同构）。
#[test]
fn test_r5179_superellipse_negative_infinity_computes_notch() {
    assert!(matches!(
        parse_corner_shape("superellipse(-infinity)"),
        Some(CornerShapeValue::All(CornerShapeKind::Notch))
    ));
    let style = style_with("superellipse(-infinity)");
    assert!(
        matches!(style.corner_shape, CornerShapeValue::All(CornerShapeKind::Notch)),
        "计算值应为 Notch，got {:?}",
        style.corner_shape
    );
}
