//! 绘制辅助 — corner-shape / border-shape 形状几何家族（自 helpers.rs 按职责拆分，T59；纯移动零行为变化）。

use zero_css_parser::values::{BorderShapeGeometryBox, BorderShapeValue, ClipPathRadius, ClipPathValue};
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_style_system::{ComputedStyle, property::types::BorderStyleValue};

use super::clip::{circle_to_polygon, ellipse_to_polygon};
use super::color::resolve_color_current;
use super::helpers::resolve_inset_length;

/// R4534（css-borders-4 §7）：border-shape 形状方案——形状 → 绝对坐标多边形 +
/// relevant side 边框色/宽（§7.6：block-start→inline-start→block-end→inline-end
/// 首个非 none 边；horizontal-tb LTR = top→left→bottom→right）。
pub(crate) enum BorderShapePlan {
    /// stroke mode：沿路径居中描边（宽 = relevant side border-width）。
    Stroke {
        /// 路径多边形顶点。
        vertices: Vec<(f32, f32)>,
        /// relevant side border-width。
        width: f32,
        /// relevant side border-color（currentColor 已解析）。
        color: Color,
    },
    /// fill mode：外/内路径之间面积填充（even-odd：外+内顶点串联仅填环带）。
    Fill {
        /// 外路径顶点。
        outer: Vec<(f32, f32)>,
        /// 内路径顶点。
        inner: Vec<(f32, f32)>,
        /// relevant side border-color。
        color: Color,
    },
}

impl BorderShapePlan {
    /// 背景色裁剪多边形（§7.5.3 背景裁剪至内形状；stroke mode = 路径多边形——
    /// 超出内缘的带由居中描边覆盖，与 oracle 语义一致）。
    pub(crate) fn background_vertices(&self) -> &[(f32, f32)] {
        match self {
            BorderShapePlan::Stroke { vertices, .. } => vertices,
            BorderShapePlan::Fill { inner, .. } => inner,
        }
    }
}

/// relevant side（§7.6：block-start→inline-start→block-end→inline-end 首个非 none
/// 边；全 none 回落 top）——R4541 从 border_shape_plan 抽出供 overflow 裁剪共用。
fn border_shape_relevant_side<'a>(
    style: &'a ComputedStyle,
    box_node: &LayoutBox,
) -> (f32, &'a zero_css_parser::values::ColorValue, &'a BorderStyleValue) {
    let sides = [
        (box_node.border_top, &style.border_top_color, &style.border_top_style),
        (box_node.border_left, &style.border_left_color, &style.border_left_style),
        (
            box_node.border_bottom,
            &style.border_bottom_color,
            &style.border_bottom_style,
        ),
        (
            box_node.border_right,
            &style.border_right_color,
            &style.border_right_style,
        ),
    ];
    let side = sides
        .iter()
        .find(|(_, _, s)| !matches!(s, BorderStyleValue::None | BorderStyleValue::Hidden))
        .copied()
        .unwrap_or((sides[0].0, sides[0].1, sides[0].2));
    (side.0, side.1, side.2)
}

/// R4541（css-borders-4 §border-shape）：border-shape 元素的 overflow 内容裁剪多边形
///（子内容裁剪到形状内缘，不得盖过边框或溢出形状）。fill mode（双形状）= 内形状顶点；
/// stroke mode（单形状）= 形状在引用盒各侧内缩 rel_width/2（描边居中于形状路径，
/// 内缘 = 半宽内缩；width=0 ≡ 形状本身）。kill-switch `ZW_BORDER_SHAPE=0` → None。
pub fn border_shape_overflow_polygon(
    style: &ComputedStyle,
    box_node: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
) -> Option<Vec<(f32, f32)>> {
    if std::env::var("ZW_BORDER_SHAPE").as_deref() == Ok("0") {
        return None;
    }
    let font_size = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None) as f32;
    let rel_width = border_shape_relevant_side(style, box_node).0;
    match &style.border_shape {
        BorderShapeValue::None => None,
        BorderShapeValue::Stroke { shape, geometry_box } => {
            let (bx, by, bw, bh) = border_shape_ref_box(*geometry_box, style, box_node, abs_x, abs_y, rel_width);
            let inset = rel_width / 2.0;
            let vertices = border_shape_vertices(
                shape,
                bx + inset,
                by + inset,
                (bw - 2.0 * inset).max(0.0),
                (bh - 2.0 * inset).max(0.0),
                font_size,
            );
            Some(vertices).filter(|v| v.len() >= 3)
        }
        BorderShapeValue::Fill { inner, inner_box, .. } => {
            let (ix, iy, iw, ih) = border_shape_ref_box(*inner_box, style, box_node, abs_x, abs_y, rel_width);
            let vertices = border_shape_vertices(inner, ix, iy, iw, ih, font_size);
            Some(vertices).filter(|v| v.len() >= 3)
        }
    }
}

/// 解析 border-shape 方案（kill-switch `ZW_BORDER_SHAPE=0` → None；none/解析失败
/// → None = 常规边框）。形状顶点为绝对坐标。
pub(crate) fn border_shape_plan(
    style: &ComputedStyle,
    box_node: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
) -> Option<BorderShapePlan> {
    if std::env::var("ZW_BORDER_SHAPE").as_deref() == Ok("0") {
        return None;
    }
    let border_shape = match &style.border_shape {
        BorderShapeValue::None => return None,
        v => v,
    };
    // relevant side（§7.6）
    let (rel_width, rel_color, _rel_style) = border_shape_relevant_side(style, box_node);
    let color = resolve_color_current(rel_color, &style.color);
    let font_size = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None) as f32;

    match border_shape {
        BorderShapeValue::None => unreachable!(),
        BorderShapeValue::Stroke { shape, geometry_box } => {
            let (bx, by, bw, bh) = border_shape_ref_box(*geometry_box, style, box_node, abs_x, abs_y, rel_width);
            let vertices = border_shape_vertices(shape, bx, by, bw, bh, font_size);
            Some(BorderShapePlan::Stroke {
                vertices,
                width: rel_width,
                color,
            })
        }
        BorderShapeValue::Fill {
            outer,
            outer_box,
            inner,
            inner_box,
        } => {
            let (ox, oy, ow, oh) = border_shape_ref_box(*outer_box, style, box_node, abs_x, abs_y, rel_width);
            let outer_v = border_shape_vertices(outer, ox, oy, ow, oh, font_size);
            let (ix, iy, iw, ih) = border_shape_ref_box(*inner_box, style, box_node, abs_x, abs_y, rel_width);
            let inner_v = border_shape_vertices(inner, ix, iy, iw, ih, font_size);
            Some(BorderShapePlan::Fill {
                outer: outer_v,
                inner: inner_v,
                color,
            })
        }
    }
}

/// geometry-box → 引用盒绝对矩形（§7.3；half-border-box = border-box 各侧内缩
/// 半个 relevant side border-width）。
fn border_shape_ref_box(
    geometry_box: BorderShapeGeometryBox,
    _style: &ComputedStyle,
    box_node: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
    rel_width: f32,
) -> (f32, f32, f32, f32) {
    match geometry_box {
        BorderShapeGeometryBox::ContentBox => (
            abs_x + box_node.border_left + box_node.padding_left,
            abs_y + box_node.border_top + box_node.padding_top,
            box_node.content_width,
            box_node.content_height,
        ),
        BorderShapeGeometryBox::PaddingBox => (
            abs_x + box_node.border_left,
            abs_y + box_node.border_top,
            box_node.width - box_node.border_left - box_node.border_right,
            box_node.height - box_node.border_top - box_node.border_bottom,
        ),
        BorderShapeGeometryBox::BorderBox => (abs_x, abs_y, box_node.width, box_node.height),
        BorderShapeGeometryBox::MarginBox => (
            abs_x - box_node.margin_left,
            abs_y - box_node.margin_top,
            box_node.width + box_node.margin_left + box_node.margin_right,
            box_node.height + box_node.margin_top + box_node.margin_bottom,
        ),
        BorderShapeGeometryBox::HalfBorderBox => {
            let inset = rel_width / 2.0;
            (
                abs_x + inset,
                abs_y + inset,
                (box_node.width - 2.0 * inset).max(0.0),
                (box_node.height - 2.0 * inset).max(0.0),
            )
        }
    }
}

/// basic-shape → 绝对坐标多边形顶点（css-shapes-1 解析语义；slice 1 忽略 inset()
/// round 圆角）。circle/ellipse 64 段近似。
fn border_shape_vertices(shape: &ClipPathValue, bx: f32, by: f32, bw: f32, bh: f32, font_size: f32) -> Vec<(f32, f32)> {
    const SEGMENTS: usize = 64;
    match shape {
        ClipPathValue::None => Vec::new(),
        ClipPathValue::Inset {
            top,
            right,
            bottom,
            left,
            ..
        } => {
            let t = resolve_inset_length(top, bh, font_size);
            let r = resolve_inset_length(right, bw, font_size);
            let b = resolve_inset_length(bottom, bh, font_size);
            let l = resolve_inset_length(left, bw, font_size);
            vec![
                (bx + l, by + t),
                (bx + bw - r, by + t),
                (bx + bw - r, by + bh - b),
                (bx + l, by + bh - b),
            ]
        }
        ClipPathValue::Circle { radius, position } => {
            let (cx, cy) = match position {
                Some((px, py)) => (
                    bx + resolve_inset_length(px, bw, font_size),
                    by + resolve_inset_length(py, bh, font_size),
                ),
                None => (bx + bw / 2.0, by + bh / 2.0),
            };
            let r = border_shape_circle_radius(radius, cx, cy, bx, by, bw, bh, font_size);
            circle_to_polygon(cx, cy, r, SEGMENTS)
        }
        ClipPathValue::Ellipse { rx, ry, position } => {
            let (cx, cy) = match position {
                Some((px, py)) => (
                    bx + resolve_inset_length(px, bw, font_size),
                    by + resolve_inset_length(py, bh, font_size),
                ),
                None => (bx + bw / 2.0, by + bh / 2.0),
            };
            let (erx, ery) = border_shape_ellipse_radii(rx, ry, cx, cy, bx, by, bw, bh, font_size);
            ellipse_to_polygon(cx, cy, erx, ery, SEGMENTS)
        }
        ClipPathValue::Polygon { points, .. } => points
            .iter()
            .map(|(x, y)| {
                (
                    bx + resolve_inset_length(x, bw, font_size),
                    by + resolve_inset_length(y, bh, font_size),
                )
            })
            .collect(),
    }
}

/// circle() 半径解析（css-shapes-1：% 相对 sqrt(w²+h²)/√2；closest/farthest-side
/// = 圆心到引用盒四边最近/最远距离）。
fn border_shape_circle_radius(
    radius: &ClipPathRadius,
    cx: f32,
    cy: f32,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
    font_size: f32,
) -> f32 {
    match radius {
        ClipPathRadius::Length(v) => {
            resolve_inset_length(v, (bw * bw + bh * bh).sqrt() / std::f32::consts::SQRT_2, font_size)
        }
        ClipPathRadius::ClosestSide => (cx - bx).min(bx + bw - cx).min(cy - by).min(by + bh - cy),
        ClipPathRadius::FarthestSide => (cx - bx).max(bx + bw - cx).max(cy - by).max(by + bh - cy),
        // closest/farthest-corner = 圆心到角的欧氏距离（dx、dy 组合 4 个角）。
        ClipPathRadius::ClosestCorner => {
            let dxs = [(cx - bx), (bx + bw - cx)];
            let dys = [(cy - by), (by + bh - cy)];
            let mut best = f32::MAX;
            for dx in dxs {
                for dy in dys {
                    best = best.min((dx * dx + dy * dy).sqrt());
                }
            }
            best
        }
        ClipPathRadius::FarthestCorner => {
            let dxs = [(cx - bx), (bx + bw - cx)];
            let dys = [(cy - by), (by + bh - cy)];
            let mut best = f32::MIN;
            for dx in dxs {
                for dy in dys {
                    best = best.max((dx * dx + dy * dy).sqrt());
                }
            }
            best
        }
    }
}

/// ellipse() 半径解析（% rx 相对引用盒宽、ry 相对高；closest/farthest-side 逐轴）。
fn border_shape_ellipse_radii(
    rx: &ClipPathRadius,
    ry: &ClipPathRadius,
    cx: f32,
    cy: f32,
    bx: f32,
    by: f32,
    bw: f32,
    bh: f32,
    font_size: f32,
) -> (f32, f32) {
    // *-corner 关键词双轴同值 = 圆心到最近/最远角的欧氏距离（csswg#14010 口径，
    // border-shape-ellipse-corner-keywords ref 实证：hypot(60,80)=100 双轴同值）。
    let corner = |closest: bool| -> f32 {
        let dxs = [(cx - bx), (bx + bw - cx)];
        let dys = [(cy - by), (by + bh - cy)];
        let mut best = if closest { f32::MAX } else { f32::MIN };
        for dx in dxs {
            for dy in dys {
                let d = (dx * dx + dy * dy).sqrt();
                best = if closest { best.min(d) } else { best.max(d) };
            }
        }
        best
    };
    if matches!(rx, ClipPathRadius::ClosestCorner | ClipPathRadius::FarthestCorner)
        || matches!(ry, ClipPathRadius::ClosestCorner | ClipPathRadius::FarthestCorner)
    {
        let d = match rx {
            ClipPathRadius::ClosestCorner => corner(true),
            ClipPathRadius::FarthestCorner => corner(false),
            _ => match ry {
                ClipPathRadius::ClosestCorner => corner(true),
                ClipPathRadius::FarthestCorner => corner(false),
                _ => 0.0,
            },
        };
        return (d, d);
    }
    let radii = |v: &ClipPathRadius, dim: f32, axis: u8| match v {
        ClipPathRadius::Length(v) => resolve_inset_length(v, dim, font_size),
        ClipPathRadius::ClosestSide => {
            if axis == 0 {
                (cx - bx).min(bx + bw - cx)
            } else {
                (cy - by).min(by + bh - cy)
            }
        }
        ClipPathRadius::FarthestSide => {
            if axis == 0 {
                (cx - bx).max(bx + bw - cx)
            } else {
                (cy - by).max(by + bh - cy)
            }
        }
        ClipPathRadius::ClosestCorner => corner(true),
        ClipPathRadius::FarthestCorner => corner(false),
    };
    (radii(rx, bw, 0), radii(ry, bh, 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::helpers::BorderRadiusSpec;
    use zero_css_parser::values::LengthValue;
    use zero_style_system::ComputedStyle;

    // ── BorderRadiusSpec ────────────────────────────────────────────────

    #[test]
    fn test_border_radius_from_style() {
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = LengthValue::Px(10.0);
        style.border_top_right_radius = LengthValue::Px(20.0);
        style.border_bottom_right_radius = LengthValue::Px(30.0);
        style.border_bottom_left_radius = LengthValue::Px(40.0);
        let spec = BorderRadiusSpec::from_style(&style);
        assert_eq!(spec.top_left, 10.0);
        assert_eq!(spec.top_right, 20.0);
        assert_eq!(spec.bottom_right, 30.0);
        assert_eq!(spec.bottom_left, 40.0);
    }

    #[test]
    fn test_border_radius_is_zero() {
        let spec = BorderRadiusSpec {
            top_left: 0.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
        };
        assert!(spec.is_zero());
    }

    #[test]
    fn test_border_radius_not_zero() {
        let spec = BorderRadiusSpec {
            top_left: 5.0,
            top_right: 0.0,
            bottom_right: 0.0,
            bottom_left: 0.0,
        };
        assert!(!spec.is_zero());
    }

    // ── R2314: border-radius 百分比 / 含百分比 calc 解析 ──────────────────

    /// 辅助：从字符串构造含百分比 calc 的 LengthValue。
    fn calc_radius(s: &str) -> LengthValue {
        LengthValue::Calc(Box::new(
            zero_css_parser::values::parse_math_function(s).expect("calc must parse"),
        ))
    }

    #[test]
    fn test_r2314_border_radius_px_byte_identical() {
        // px 值经 from_style_with_box 与 from_style 完全一致（不钳制）
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = LengthValue::Px(10.0);
        style.border_top_right_radius = LengthValue::Px(20.0);
        style.border_bottom_right_radius = LengthValue::Px(30.0);
        style.border_bottom_left_radius = LengthValue::Px(40.0);
        let with_box = BorderRadiusSpec::from_style_with_box(&style, 100.0, 100.0);
        let plain = BorderRadiusSpec::from_style(&style);
        assert_eq!(with_box.top_left, plain.top_left);
        assert_eq!(with_box.top_right, plain.top_right);
        assert_eq!(with_box.bottom_right, plain.bottom_right);
        assert_eq!(with_box.bottom_left, plain.bottom_left);
        assert_eq!(with_box.top_left, 10.0);
    }

    #[test]
    fn test_r2314_border_radius_percentage_circle() {
        // border-radius: 50% on 100×100 → 50（正圆）。此前 length_to_f32 丢弃为 0（方形）。
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = LengthValue::Percentage(50.0);
        let spec = BorderRadiusSpec::from_style_with_box(&style, 100.0, 100.0);
        assert_eq!(spec.top_left, 50.0);
    }

    #[test]
    fn test_r2314_border_radius_percentage_clamped() {
        // 50% on 200×100 → 0.5×200=100，钳制到 min(200,100)/2=50（CSS 单角半径不超边长一半）
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = LengthValue::Percentage(50.0);
        let spec = BorderRadiusSpec::from_style_with_box(&style, 200.0, 100.0);
        assert_eq!(spec.top_left, 50.0);
    }

    #[test]
    fn test_r2314_border_radius_percentage_small() {
        // 10% on 100×100 → 10（未触钳制）
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = LengthValue::Percentage(10.0);
        let spec = BorderRadiusSpec::from_style_with_box(&style, 100.0, 100.0);
        assert_eq!(spec.top_left, 10.0);
    }

    #[test]
    fn test_r2314_border_radius_calc_with_percentage() {
        // calc(50% - 5px) on 100×100 → 50 - 5 = 45
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = calc_radius("calc(50% - 5px)");
        let spec = BorderRadiusSpec::from_style_with_box(&style, 100.0, 100.0);
        assert!((spec.top_left - 45.0).abs() < 0.01, "got {}", spec.top_left);
    }

    #[test]
    fn test_r2314_border_radius_min_max_clamp_with_percentage() {
        // 用 200×200（max_r=100）避免 CSS 单角半径钳制（≤边长一半）干扰 min/max/clamp 数学验证
        // min(80%, 60px) on 200×200 → min(160, 60) = 60
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = calc_radius("min(80%, 60px)");
        let spec = BorderRadiusSpec::from_style_with_box(&style, 200.0, 200.0);
        assert!((spec.top_left - 60.0).abs() < 0.01, "got {}", spec.top_left);

        // max(10%, 30px) on 200×200 → max(20, 30) = 30
        let mut style2 = ComputedStyle::default();
        style2.border_top_right_radius = calc_radius("max(10%, 30px)");
        let spec2 = BorderRadiusSpec::from_style_with_box(&style2, 200.0, 200.0);
        assert!((spec2.top_right - 30.0).abs() < 0.01, "got {}", spec2.top_right);

        // clamp(20%, 90%, 70px) on 200×200 → clamp(40, 180, 70) = 70
        let mut style3 = ComputedStyle::default();
        style3.border_bottom_right_radius = calc_radius("clamp(20%, 90%, 70px)");
        let spec3 = BorderRadiusSpec::from_style_with_box(&style3, 200.0, 200.0);
        assert!((spec3.bottom_right - 70.0).abs() < 0.01, "got {}", spec3.bottom_right);
    }

    #[test]
    fn test_r2314_border_radius_default_zero() {
        // 无圆角 → 0
        let style = ComputedStyle::default();
        let spec = BorderRadiusSpec::from_style_with_box(&style, 100.0, 100.0);
        assert!(spec.is_zero());
    }

    // ── css-backgrounds-3 §5.5 相邻圆角重叠收缩 ─────────────────────────

    #[test]
    fn test_border_radius_pill_overlap_shrunk_to_half_height() {
        // pill 习语 `border-radius: 999px` on 66×27：四角收缩到 min(w,h)/2 = 13.5
        //（welcome 页 .version 徽章；修复前 Px 原样 999，圆角光栅化整体失效）
        let mut style = ComputedStyle::default();
        for r in [
            &mut style.border_top_left_radius,
            &mut style.border_top_right_radius,
            &mut style.border_bottom_right_radius,
            &mut style.border_bottom_left_radius,
        ] {
            *r = LengthValue::Px(999.0);
        }
        let spec = BorderRadiusSpec::from_style_with_box(&style, 66.0, 27.0);
        let half = 13.5;
        assert!((spec.top_left - half).abs() < 1e-4, "tl={}", spec.top_left);
        assert!((spec.top_right - half).abs() < 1e-4, "tr={}", spec.top_right);
        assert!((spec.bottom_right - half).abs() < 1e-4, "br={}", spec.bottom_right);
        assert!((spec.bottom_left - half).abs() < 1e-4, "bl={}", spec.bottom_left);
    }

    #[test]
    fn test_border_radius_overlapping_pair_scaled_proportionally() {
        // tl=tr=60 on 100×50：f = min(100/120, 50/60) = 0.8̄ → 两角各 50
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = LengthValue::Px(60.0);
        style.border_top_right_radius = LengthValue::Px(60.0);
        let spec = BorderRadiusSpec::from_style_with_box(&style, 100.0, 50.0);
        assert!((spec.top_left - 50.0).abs() < 1e-3, "tl={}", spec.top_left);
        assert!((spec.top_right - 50.0).abs() < 1e-3, "tr={}", spec.top_right);
        assert_eq!(spec.bottom_right, 0.0);
        assert_eq!(spec.bottom_left, 0.0);
    }

    #[test]
    fn test_border_radius_no_overlap_unchanged() {
        // 不重叠（各边之和 ≤ 边长）时保持原值——R2314 px byte-identical 契约的前提
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = LengthValue::Px(40.0);
        style.border_top_right_radius = LengthValue::Px(20.0);
        let spec = BorderRadiusSpec::from_style_with_box(&style, 100.0, 100.0);
        assert_eq!(spec.top_left, 40.0);
        assert_eq!(spec.top_right, 20.0);
        assert_eq!(spec.bottom_right, 0.0);
        assert_eq!(spec.bottom_left, 0.0);
    }
}
