//! 绘制辅助 — corner-shape / border-shape 形状几何家族（自 helpers.rs 按职责拆分，T59；纯移动零行为变化）。

use zero_css_parser::values::{BorderShapeGeometryBox, BorderShapeValue, ClipPathRadius, ClipPathValue};
use zero_layout_engine::LayoutBox;
use zero_render_foundation::color::Color;
use zero_style_system::{ComputedStyle, property::types::BorderStyleValue};

use crate::paint::helpers::BorderRadiusSpec;

use super::clip::{circle_to_polygon, ellipse_to_polygon};
use super::color::resolve_color_current;
use super::helpers::resolve_inset_length;

/// R5184 kill-switch（`ZW_BORDER_SHAPE=0` 整体禁用 border-shape 路径）。
// OPTIMIZATION：进程内只读一次——下方三个消费端均在 painter 逐元素热路径，
// `env::var` 每次上锁扫描 environ，500 元素页一次 paint 数千次查询，实测
// CI bench-gate `paint_complex_page_500_elements` +34% 超预算。kill-switch
// 约定进程启动前设定，读一次与逐次读语义等价。
fn border_shape_disabled() -> bool {
    static KILL: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *KILL.get_or_init(|| std::env::var("ZW_BORDER_SHAPE").as_deref() == Ok("0"))
}

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
    if border_shape_disabled() {
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
    if border_shape_disabled() {
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

/// R5186（css-borders-4 §border-shape，判例 inner-outer）：形状活跃即视为「有边框
/// 可绘」——绘制入口的布局边框宽门禁放行 fill mode 环带（环带宽 = 外/内形状围成
/// 域，与 border 宽解耦；border-style none 判例环带 = 外形状到内形状整段）。
/// stroke mode 环带仍由 plan 侧 `width > 0` 门禁压空（style none → 宽 0 → 无带）。
pub(crate) fn border_shape_border_path_active(style: &ComputedStyle) -> bool {
    !matches!(style.border_shape, BorderShapeValue::None)
}

/// R5186（css-borders-4 §border-shape，判例 polygon-miter-limit）：形状描边的 miter
/// join 楔形。CPU path stroke = 逐段粗线无 join 几何——凸顶点处两段端帽间缺口、
/// 尖角顶点丢失 miter 尖刺（ref `stroke-miterlimit=1000` 保留尖刺，ratio ≈ 5）。
/// 逐顶点发射 [外偏移点 A, miter 尖点 M, 外偏移点 B] 楔形（一次 even-odd
/// path_fill）：凹顶点楔形落于带内属同色冗余覆盖无害；自交路径（bowtie）交叉处
/// 无顶点不受影响。外侧按局部转向判定（cross 符号 → 右/左法线族），不依赖全局
/// 有向面积（自交时无意义）。miter 不设限（判例语义「keeps miter joins」）；
/// 近平行边（|cross| 过小，t 发散）与超界 t 跳过——FIXME(miter-bevel-limit)：
/// bevel/miter-limit 拓扑独立轮评估。
pub(crate) fn stroke_miter_wedges(pts: &[(f32, f32)], width: f32) -> Vec<Vec<(f32, f32)>> {
    let n = pts.len();
    if n < 3 || width <= 0.0 {
        return Vec::new();
    }
    let h = width * 0.5;
    let mut wedges = Vec::new();
    for i in 0..n {
        let (px, py) = pts[(i + n - 1) % n];
        let (cx, cy) = pts[i];
        let (qx, qy) = pts[(i + 1) % n];
        let (ux, uy) = (cx - px, cy - py);
        let (wx, wy) = (qx - cx, qy - cy);
        let ul = (ux * ux + uy * uy).sqrt();
        let wl = (wx * wx + wy * wy).sqrt();
        if ul < 1e-9 || wl < 1e-9 {
            continue;
        }
        let (ux, uy) = (ux / ul, uy / ul);
        let (wx, wy) = (wx / wl, wy / wl);
        let cross = ux * wy - uy * wx;
        if cross.abs() < 1e-9 {
            continue; // 共线：无 join 缺口
        }
        let (n1x, n1y, n2x, n2y) = if cross > 0.0 {
            (uy, -ux, wy, -wx)
        } else {
            (-uy, ux, -wy, wx)
        };
        let a = (cx + n1x * h, cy + n1y * h);
        let b = (cx + n2x * h, cy + n2y * h);
        // M = 直线 (A, u) ∩ (B, w)：t·u − s·w = B−A，叉乘 w 消 s。
        let t = ((b.0 - a.0) * wy - (b.1 - a.1) * wx) / cross;
        if !t.is_finite() || t.abs() > 1e4 {
            continue;
        }
        wedges.push(vec![a, (a.0 + t * ux, a.1 + t * uy), b]);
    }
    wedges
}

/// R5184（css-borders-4 §border-shape + WPT 判例）：stroke mode `inset(0 round R)`
/// 与默认 half-border-box 的 border-radius 等价判定。CSS Backgrounds §5.5 单角钳制
/// 使描边带与 border-radius: R 渲染逐弧重合（outline-auto-with-border R=60→外弧
/// 50、outline-auto-forms R=25→25 判例；chromium 同构复用圆角栅格）。命中时
/// painter 按 border-radius 同款图元路径施绘——同图元 = 同栅格 = 配对像素一致。
/// 非 0 inset / 非 Length 圆角 / 显式其他 geometry-box 不在等价域（无判例，维持
/// 形状路径施绘）。kill-switch 同 `ZW_BORDER_SHAPE`。
pub(crate) fn border_shape_round_inset_equivalent_radii(
    style: &ComputedStyle,
    box_node: &LayoutBox,
) -> Option<BorderRadiusSpec> {
    if border_shape_disabled() {
        return None;
    }
    let BorderShapeValue::Stroke { shape, geometry_box } = &style.border_shape else {
        return None;
    };
    if !matches!(geometry_box, BorderShapeGeometryBox::HalfBorderBox) {
        return None;
    }
    let ClipPathValue::Inset {
        top,
        right,
        bottom,
        left,
        round,
    } = shape
    else {
        return None;
    };
    let font_size = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None) as f32;
    let zero = |v: &zero_css_parser::values::LengthValue| resolve_inset_length(v, 0.0, font_size) == 0.0;
    if !(zero(top) && zero(right) && zero(bottom) && zero(left)) {
        return None;
    }
    let Some(ClipPathRadius::Length(v)) = round else {
        return None;
    };
    // 单角钳制 min(R, min(w,h)/2)——R5203 起 border-radius 解析（from_style_with_box）
    // 已去半短边钳制，clip-path inset round 仍保留（规范两处同为 §5.5 口径无钳制；
    // 无 corpus 驱动案，行为分歧挂账不改，仅此注释记录）。
    let half_min = box_node.width.min(box_node.height) / 2.0;
    let declared = resolve_inset_length(v, half_min, font_size);
    let r = declared.min(half_min).max(0.0);
    if r <= 0.0 {
        return None;
    }
    Some(BorderRadiusSpec {
        top_left: r,
        top_right: r,
        bottom_right: r,
        bottom_left: r,
    })
}

/// R5185（css-backgrounds-4 §2.1 background-clip:border-area × css-borders-4
/// §border-shape）：border-area 环带多边形（even-odd 外+内串联，[`super::helpers::concat_ring_vertices`]
/// keyhole 切点）——border-area 的背景可见区 = 边框墨迹带，形状元素即描边带环
/// （fill mode = 外/内形状间环带）。带不可得（非凸路径 / miter 比 > 4 / 退化）→
/// None（调用方回退既有行为：R4529 矩形环带 + R5184 填充窗）。FIXME：带环出盒
/// 部分（如 polygon 顶点外扩越 border-box）受 painting area = border-box 限制不
/// 绘——overflow 判例 poking ~250px 在 fuzzy totalPixels 容差内。
pub(crate) fn border_shape_border_area_band(
    style: &ComputedStyle,
    box_node: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
) -> Option<Vec<(f32, f32)>> {
    match border_shape_plan(style, box_node, abs_x, abs_y)? {
        BorderShapePlan::Fill { outer, inner, .. } => Some(super::helpers::concat_ring_vertices(outer, inner)),
        BorderShapePlan::Stroke { vertices, width, .. } => {
            if width <= 0.0 {
                return None;
            }
            let outer = offset_convex_polygon(&vertices, width / 2.0)?;
            let inner = offset_convex_polygon(&vertices, -width / 2.0)?;
            Some(super::helpers::concat_ring_vertices(outer, inner))
        }
    }
}

/// R5185：背景图像层形状裁剪多边形选择——border-area 层 = 带环（上函数），其余层
/// = 填充多边形（R5184：stroke = 路径、fill = 内形状，≡ R4534 背景色语义）。
/// border-shape 不活跃 → None。带不可得时 border-area 回退填充多边形（维持现态）。
pub(crate) fn border_shape_bg_image_clip_polygon(
    style: &ComputedStyle,
    box_node: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
) -> Option<Vec<(f32, f32)>> {
    use zero_style_system::property::types::BackgroundClipComputedValue;
    let plan = border_shape_plan(style, box_node, abs_x, abs_y)?;
    let border_area = style
        .background_clip
        .iter()
        .any(|c| matches!(c, BackgroundClipComputedValue::BorderArea));
    if border_area {
        Some(
            border_shape_border_area_band(style, box_node, abs_x, abs_y)
                .unwrap_or_else(|| plan.background_vertices().to_vec()),
        )
    } else {
        Some(plan.background_vertices().to_vec())
    }
}

/// R5188（css-borders-4 §border-shape，判例 two-shapes-shadow / combined-shadow）：
/// 形状活跃时零模糊 box-shadow 的随形 even-odd 环带顶点。
/// - inset（零偏移、spread > 0）：环带 = 描边带内缘（fill mode = 内形状；stroke
///   mode = 路径内缩 relevant side width/2）与再内缩 spread 之间（ref 语义 = SVG
///   stroke-width=2·spread clip 到内缘）。
/// - outset（任意偏移/spread）：阴影形状 = 边框墨迹外缘（fill mode = 外形状；
///   stroke mode = 路径外扩 width/2——border-shape-shadow ref 实证 spread 自边框
///   外缘量起）平移 (dx,dy) 后外扩 spread，与原位置形状 even-odd 对称差 =
///   punch-out（CSS Backgrounds §7.1「drawn outside the border edge only」）。
///
/// 非凸路径（offset miter 比 > 4）/ 内缩退化（spread 超内切半径，偏移多边形翻转）/
/// 顶点 < 3 → None（调用方回退 ShadowPrimitive 方环既有行为）。FIXME：outset 对称
/// 差在透明背景元素上会把「盒内非阴影区」误涂（even-odd XOR ≠ 布尔差）——corpus
/// 判例均为不透明背景/边框整盖、绘制序在背景之下，像素等价；布尔差拓扑独立轮评估。
pub(crate) fn border_shape_shadow_ring(
    style: &ComputedStyle,
    box_node: &LayoutBox,
    abs_x: f32,
    abs_y: f32,
    inset: bool,
    offset_x: f32,
    offset_y: f32,
    spread: f32,
) -> Option<Vec<(f32, f32)>> {
    let plan = border_shape_plan(style, box_node, abs_x, abs_y)?;
    if inset {
        if offset_x != 0.0 || offset_y != 0.0 || spread <= 0.0 {
            return None;
        }
        let perimeter = match &plan {
            BorderShapePlan::Fill { inner, .. } => inner.clone(),
            BorderShapePlan::Stroke { vertices, width, .. } => {
                if *width > 0.0 {
                    offset_convex_polygon(vertices, -width / 2.0)?
                } else {
                    vertices.clone()
                }
            }
        };
        if perimeter.len() < 3 {
            return None;
        }
        // spread ≥ 内切半径 → 环带吞没全域（偏移线过质心后翻转成伪小多边形）→
        // 形状路径不可得，回退方环既有行为。
        if spread >= polygon_inradius(&perimeter) {
            return None;
        }
        let band = offset_convex_polygon(&perimeter, -spread)?;
        // 内缩退化（spread 超内切半径）→ 偏移多边形翻转/越界：面积符号翻转、不再
        // 严格小于内缘、或带顶点越出内缘 → 回退方环。
        let (a_edge, a_band) = (polygon_area2(&perimeter), polygon_area2(&band));
        if a_edge.signum() != a_band.signum() || a_band.abs() >= a_edge.abs() || !convex_contains_all(&perimeter, &band)
        {
            return None;
        }
        Some(super::helpers::concat_ring_vertices(perimeter, band))
    } else {
        let base = match &plan {
            BorderShapePlan::Fill { outer, .. } => outer.clone(),
            BorderShapePlan::Stroke { vertices, width, .. } => {
                if *width > 0.0 {
                    offset_convex_polygon(vertices, *width / 2.0)?
                } else {
                    vertices.clone()
                }
            }
        };
        if base.len() < 3 {
            return None;
        }
        // 负 spread 内缩同内切半径守卫（正 spread 外扩无翻转域，不需守卫）。
        if spread < 0.0 && -spread >= polygon_inradius(&base) {
            return None;
        }
        let grown = if spread != 0.0 {
            let g = offset_convex_polygon(&base, spread)?;
            // 外扩/内缩方向校验：偏移多边形翻转（符号变号）、与 spread 方向相悖、
            // 或负 spread 内缩带顶点越出基圈 → 退化回退方环。
            let (a_base, a_grown) = (polygon_area2(&base), polygon_area2(&g));
            if a_base.signum() != a_grown.signum()
                || (spread > 0.0 && a_grown.abs() <= a_base.abs())
                || (spread < 0.0 && (a_grown.abs() >= a_base.abs() || !convex_contains_all(&base, &g)))
            {
                return None;
            }
            g
        } else {
            base.clone()
        };
        let moved: Vec<(f32, f32)> = grown.into_iter().map(|(x, y)| (x + offset_x, y + offset_y)).collect();
        Some(super::helpers::concat_ring_vertices(moved, base))
    }
}

/// shoelace 有向面积 ×2（顶点序保留符号；退化共线 ≈ 0）。环带退化判定用。
fn polygon_area2(pts: &[(f32, f32)]) -> f32 {
    pts.iter()
        .enumerate()
        .map(|(i, (x1, y1))| {
            let (x2, y2) = pts[(i + 1) % pts.len()];
            x1 * y2 - x2 * y1
        })
        .sum()
}

/// 凸多边形内切半径（质心到各边线距离最小值；质心在凸域内，距离恒正）。
/// spread 超内切半径 → 内缩偏移线过质心，偏移多边形翻转成伪小多边形。
fn polygon_inradius(pts: &[(f32, f32)]) -> f32 {
    let n = pts.len();
    let (cx, cy) = (
        pts.iter().map(|p| p.0).sum::<f32>() / n as f32,
        pts.iter().map(|p| p.1).sum::<f32>() / n as f32,
    );
    let mut min = f32::MAX;
    for i in 0..n {
        let (x1, y1) = pts[i];
        let (x2, y2) = pts[(i + 1) % n];
        let (ex, ey) = (x2 - x1, y2 - y1);
        let len = (ex * ex + ey * ey).sqrt();
        if len < 1e-9 {
            continue;
        }
        let d = ((ex * (cy - y1) - ey * (cx - x1)) / len).abs();
        min = min.min(d);
    }
    min
}

/// 凸多边形包含判定（逐点边界叉积同号；共线段跳过）。内缩退化（偏移多边形翻转
/// 越界）判定用——翻转路径的顶点必落基圈外。
fn convex_contains_all(outer: &[(f32, f32)], pts: &[(f32, f32)]) -> bool {
    let n = outer.len();
    pts.iter().all(|&(px, py)| {
        let mut sign = 0i8;
        for i in 0..n {
            let (x1, y1) = outer[i];
            let (x2, y2) = outer[(i + 1) % n];
            let c = (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1);
            if c.abs() < 1e-6 {
                continue;
            }
            let s = if c > 0.0 { 1 } else { -1 };
            if sign == 0 {
                sign = s;
            } else if s != sign {
                return false;
            }
        }
        true
    })
}

/// 凸多边形 miter 偏移（d > 0 外扩 / d < 0 内缩）——逐边法向平移线，相邻偏移线
/// 交点 = 偏移顶点（join = miter，css-borders-4 §border-shape 描边带边界）。
/// 非凸（相邻边 cross 变号）/ 平行边退化 / miter 比 > 4（sin(θ/2) < 1/4，斜切
/// 限制域 bevel 拓扑独立轮评估）→ None。
pub(crate) fn offset_convex_polygon(pts: &[(f32, f32)], d: f32) -> Option<Vec<(f32, f32)>> {
    let n = pts.len();
    if n < 3 || d == 0.0 {
        return None;
    }
    // 有向面积定外向法线符号（屏幕坐标 y 向下，代数自洽不依赖视觉朝向）。
    let area2: f32 = (0..n)
        .map(|i| {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[(i + 1) % n];
            x1 * y2 - x2 * y1
        })
        .sum();
    if area2.abs() < 1e-9 {
        return None;
    }
    let outward_pos = area2 > 0.0;
    let edges: Vec<(f32, f32)> = (0..n)
        .map(|i| {
            let (x1, y1) = pts[i];
            let (x2, y2) = pts[(i + 1) % n];
            (x2 - x1, y2 - y1)
        })
        .collect();
    // 凸性：相邻边 cross 全同号（共线段跳过）。
    let mut sign = 0i8;
    for i in 0..n {
        let (ax, ay) = edges[i];
        let (bx, by) = edges[(i + 1) % n];
        let c = ax * by - ay * bx;
        if c.abs() < 1e-6 {
            continue;
        }
        let s = if c > 0.0 { 1 } else { -1 };
        if sign == 0 {
            sign = s;
        } else if s != sign {
            return None;
        }
    }
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        // 偏移线 L_j：过 pts[j] + n_j·d、方向 e_j；n_j = e_j 的外向单位法线。
        let line = |j: usize| -> Option<((f32, f32), (f32, f32))> {
            let (ex, ey) = edges[j];
            let len = (ex * ex + ey * ey).sqrt();
            if len < 1e-9 {
                return None;
            }
            let (ex, ey) = (ex / len, ey / len);
            let (nx, ny) = if outward_pos { (ey, -ex) } else { (-ey, ex) };
            let (px, py) = pts[j];
            Some(((px + nx * d, py + ny * d), (ex, ey)))
        };
        let ((ax, ay), (e1x, e1y)) = line((i + n - 1) % n)?;
        let ((bx, by), (e2x, e2y)) = line(i)?;
        let denom = e1x * e2y - e1y * e2x;
        if denom.abs() < 1e-9 {
            return None;
        }
        // miter 比 = 1/sin(θ内/2) = 1/cos(转角/2) > 4 ⟺ cos_t < −0.875（尖刺域
        // bevel 拓扑独立轮评估）。
        let cos_t = (e1x * e2x + e1y * e2y).clamp(-1.0, 1.0);
        if cos_t < -0.875 {
            return None;
        }
        let t = ((bx - ax) * e2y - (by - ay) * e2x) / denom;
        out.push((ax + t * e1x, ay + t * e1y));
    }
    Some(out)
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

/// basic-shape → 绝对坐标多边形顶点（css-shapes-1 解析语义）。circle/ellipse 64 段近似。
fn border_shape_vertices(shape: &ClipPathValue, bx: f32, by: f32, bw: f32, bh: f32, font_size: f32) -> Vec<(f32, f32)> {
    const SEGMENTS: usize = 64;
    match shape {
        ClipPathValue::None => Vec::new(),
        ClipPathValue::Inset {
            top,
            right,
            bottom,
            left,
            round,
        } => {
            let t = resolve_inset_length(top, bh, font_size);
            let r = resolve_inset_length(right, bw, font_size);
            let b = resolve_inset_length(bottom, bh, font_size);
            let l = resolve_inset_length(left, bw, font_size);
            let (iw, ih) = (bw - l - r, bh - t - b);
            // R5184（css-shapes-1 inset() round；css-borders-4 §border-shape）：
            // 单一圆角按 CSS Backgrounds §5.5 单角钳制 min(r, min(iw,ih)/2)（均匀
            // 半径相邻必不重叠）出圆角矩形多边形——half-border-box 描边带经此钳制
            // 与 border-radius 渲染逐弧重合（outline-auto-with-border R=60→45+5=50、
            // outline-auto-forms R=25→20+5=25 判例）。圆角百分比按内矩形短边解析：
            // FIXME 规范为横/纵各相对宽/高的椭圆角，ZW 圆角模型取单值近似。
            if let Some(ClipPathRadius::Length(v)) = round
                && iw > 0.0
                && ih > 0.0
            {
                let corner = resolve_inset_length(v, iw.min(ih), font_size).min(iw.min(ih) / 2.0);
                if corner > 0.0 {
                    return super::helpers::rounded_corner_polygon_in_box(bx + l, by + t, iw, ih, [corner; 4]);
                }
            }
            // FIXME: round 关键词半径与非均匀椭圆角未支持，回退方角四顶点。
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
    fn test_r2314_border_radius_percentage_no_half_side_clamp() {
        // R5203：50% on 200×100 → 0.5×200=100，不再按半短边钳制（§5.5 角重叠
        // 收缩才是规范口径，单角 100 无重叠保持原值；修前钳到 50 与 px 不同形）
        let mut style = ComputedStyle::default();
        style.border_top_left_radius = LengthValue::Percentage(50.0);
        let spec = BorderRadiusSpec::from_style_with_box(&style, 200.0, 100.0);
        assert_eq!(spec.top_left, 100.0);
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

    // ── R5184: inset() round 圆角多边形 ─────────────────────────────────

    fn inset_shape(round: Option<ClipPathRadius>) -> ClipPathValue {
        ClipPathValue::Inset {
            top: LengthValue::Px(0.0),
            right: LengthValue::Px(0.0),
            bottom: LengthValue::Px(0.0),
            left: LengthValue::Px(0.0),
            round,
        }
    }

    /// R5184：inset(0 round 60px) on half-border rect (5,5,90,90)——半径钳制
    /// min(60, 45) = 45，弧心 (50,50)，顶缘弧顶 (50,5) 应为顶点（描边带外缘
    /// = r50 弧心 (50,50)，与 border-radius:60px 渲染逐弧重合的判例几何）。
    #[test]
    fn test_r5184_inset_round_clamped_polygon() {
        let verts = border_shape_vertices(
            &inset_shape(Some(ClipPathRadius::Length(LengthValue::Px(60.0)))),
            5.0,
            5.0,
            90.0,
            90.0,
            16.0,
        );
        assert!(verts.len() >= 16, "圆角多边形应远多于 4 顶点：{}", verts.len());
        // 弧顶 (50, 5)：钳制后 45 半径弧的最高点。
        assert!(
            verts
                .iter()
                .any(|(x, y)| (x - 50.0).abs() < 0.01 && (y - 5.0).abs() < 0.01),
            "顶缘弧顶 (50,5) 应为顶点：{verts:?}"
        );
        // 无顶点落在盒角 (5,5)（方角已被弧替代）。
        assert!(
            !verts
                .iter()
                .any(|(x, y)| (x - 5.0).abs() < 0.5 && (y - 5.0).abs() < 0.5),
            "盒角 (5,5) 不应有顶点：{verts:?}"
        );
    }

    /// 无 round → 方角四顶点 byte-identical（既有用户零扰动契约）。
    #[test]
    fn test_r5184_inset_without_round_square_unchanged() {
        let verts = border_shape_vertices(&inset_shape(None), 5.0, 5.0, 90.0, 90.0, 16.0);
        assert_eq!(verts.len(), 4);
        assert_eq!(verts[0], (5.0, 5.0));
        assert_eq!(verts[1], (95.0, 5.0));
        assert_eq!(verts[2], (95.0, 95.0));
        assert_eq!(verts[3], (5.0, 95.0));
    }

    /// 负尺寸内矩形（collapsed shape，如 inset(100px) on 197×197）回退方角——
    /// 退化 bowtie 语义保持（扫描线近似零面积）。
    #[test]
    fn test_r5184_inset_round_degenerate_falls_back() {
        let shape = ClipPathValue::Inset {
            top: LengthValue::Px(100.0),
            right: LengthValue::Px(100.0),
            bottom: LengthValue::Px(100.0),
            left: LengthValue::Px(100.0),
            round: Some(ClipPathRadius::Length(LengthValue::Px(20.0))),
        };
        let verts = border_shape_vertices(&shape, 0.5, 0.5, 197.0, 197.0, 16.0);
        assert_eq!(verts.len(), 4, "负尺寸内矩形应回退 4 顶点：{verts:?}");
    }

    /// 非正方形盒钳制：forms 判例几何——inset(0 round 25px) on (5,5,190,40)：
    /// 半径钳到 min(25, 40/2) = 20，弧心 (25,25)，顶/底弧顶 (25,5)/(25,45)。
    #[test]
    fn test_r5184_inset_round_clamp_non_square() {
        let verts = border_shape_vertices(
            &inset_shape(Some(ClipPathRadius::Length(LengthValue::Px(25.0)))),
            5.0,
            5.0,
            190.0,
            40.0,
            16.0,
        );
        for apex in [(25.0_f32, 5.0_f32), (25.0, 45.0)] {
            assert!(
                verts
                    .iter()
                    .any(|(x, y)| (x - apex.0).abs() < 0.01 && (y - apex.1).abs() < 0.01),
                "弧顶 {apex:?} 应为顶点：{verts:?}"
            );
        }
    }
}
