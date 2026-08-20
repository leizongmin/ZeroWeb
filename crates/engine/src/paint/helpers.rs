//! 辅助工具 — 变换偏移、裁剪、opacity 应用、渐变转换等。

use zero_css_parser::values::{
    ColorHueMethod, ColorInterpolation, ColorInterpolationSpace, ColorValue, GradientColorStop, GradientDirection,
    GradientValue, LengthValue, RadialSize, TransformFunction, TransformValue, eval_calc,
};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{
    GradientColorSpace, GradientInterpolation, GradientKind, GradientPrimitive, GradientStop, HueMethod,
    RenderPrimitives, TransformPrimitive,
};
use zero_style_system::{ComputedStyle, TextTransformValue};

use super::color::resolve_color_current;

/// 从 ComputedStyle 的 transform 计算偏移量。
///
/// 返回 (dx, dy) 偏移，用于调整图元位置。
/// 仅提取 translate 分量；rotate/scale/skew 由 TransformPrimitive 处理。
pub fn apply_transform_offset(style: &ComputedStyle, _abs_x: f32, _abs_y: f32) -> (f32, f32) {
    match &style.transform {
        TransformValue::None => (0.0, 0.0),
        TransformValue::List(funcs) => {
            let mut dx = 0.0_f32;
            let mut dy = 0.0_f32;
            for f in funcs {
                match f {
                    TransformFunction::Translate(tx, ty) => {
                        dx += *tx as f32;
                        dy += *ty as f32;
                    }
                    TransformFunction::TranslateX(tx) => {
                        dx += *tx as f32;
                    }
                    TransformFunction::TranslateY(ty) => {
                        dy += *ty as f32;
                    }
                    // rotate, scale, skew 不产生偏移
                    _ => {}
                }
            }
            (dx, dy)
        }
    }
}

/// 计算 2D 仿射变换矩阵（含 transform-origin）。
///
/// 变换矩阵按 CSS 规范组合：
/// 1. 平移到 transform-origin
/// 2. 应用所有 transform 函数
/// 3. 平移回
///
/// 返回 None 如果变换为 None 或全部是 identity。
pub fn compute_transform_matrix(style: &ComputedStyle, rect: &Rect) -> Option<TransformPrimitive> {
    let funcs = match &style.transform {
        TransformValue::None => return None,
        TransformValue::List(f) => f,
    };

    // 检查是否只有 translate 函数（由 offset 处理，不需要 TransformPrimitive）
    let has_non_translate = funcs.iter().any(|f| {
        !matches!(
            f,
            TransformFunction::Translate(_, _) | TransformFunction::TranslateX(_) | TransformFunction::TranslateY(_)
        )
    });
    if !has_non_translate {
        return None;
    }

    // 计算 transform-origin（相对于视口绝对坐标）
    let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
    let origin_x =
        rect.origin.x + resolve_transform_origin_length(&style.transform_origin_x, rect.size.width, font_size_px);
    let origin_y =
        rect.origin.y + resolve_transform_origin_length(&style.transform_origin_y, rect.size.height, font_size_px);

    // 构建累积变换矩阵（3x3 仿射，存储为 [a, b, c, d, tx, ty]）
    // | a  c  tx |
    // | b  d  ty |
    // | 0  0   1 |
    let mut a = 1.0_f32;
    let mut b = 0.0_f32;
    let mut c = 0.0_f32;
    let mut d = 1.0_f32;
    let mut tx = 0.0_f32;
    let mut ty = 0.0_f32;

    for func in funcs {
        let (fa, fb, fc, fd, ftx, fty) = match func {
            TransformFunction::Translate(dx, dy) => (1.0, 0.0, 0.0, 1.0, *dx as f32, *dy as f32),
            // R2294：translate(%) 相对元素 border-box（rect.size）求值。has_non_translate 把
            // Mixed 变体当非-translate → 走本 matrix 路径（rect 可用），绕过无 rect 的 offset 路径。
            TransformFunction::TranslateMixed(tx, txp, ty, typ) => {
                let fx = if *txp {
                    rect.size.width * (*tx as f32) / 100.0
                } else {
                    *tx as f32
                };
                let fy = if *typ {
                    rect.size.height * (*ty as f32) / 100.0
                } else {
                    *ty as f32
                };
                (1.0, 0.0, 0.0, 1.0, fx, fy)
            }
            TransformFunction::TranslateXMixed(tx, _) => {
                (1.0, 0.0, 0.0, 1.0, rect.size.width * (*tx as f32) / 100.0, 0.0)
            }
            TransformFunction::TranslateYMixed(ty, _) => {
                (1.0, 0.0, 0.0, 1.0, 0.0, rect.size.height * (*ty as f32) / 100.0)
            }
            TransformFunction::TranslateX(dx) => (1.0, 0.0, 0.0, 1.0, *dx as f32, 0.0),
            TransformFunction::TranslateY(dy) => (1.0, 0.0, 0.0, 1.0, 0.0, *dy as f32),
            TransformFunction::Rotate(deg) => {
                let rad = deg.to_radians() as f32;
                let cos = rad.cos();
                let sin = rad.sin();
                (cos, sin, -sin, cos, 0.0, 0.0)
            }
            TransformFunction::Scale(sx, sy) => {
                let sy = sy.unwrap_or(*sx) as f32;
                (*sx as f32, 0.0, 0.0, sy, 0.0, 0.0)
            }
            TransformFunction::ScaleX(sx) => (*sx as f32, 0.0, 0.0, 1.0, 0.0, 0.0),
            TransformFunction::ScaleY(sy) => (1.0, 0.0, 0.0, *sy as f32, 0.0, 0.0),
            TransformFunction::Skew(ax, ay) => {
                let tan_ax = ax.to_radians().tan() as f32;
                let tan_ay = ay.map(|v| v.to_radians().tan() as f32).unwrap_or(0.0);
                (1.0, tan_ay, tan_ax, 1.0, 0.0, 0.0)
            }
            // 3D 变换函数降级为 2D 近似
            TransformFunction::Translate3d(dx, dy, _) => (1.0, 0.0, 0.0, 1.0, *dx as f32, *dy as f32),
            TransformFunction::RotateX(_) | TransformFunction::RotateY(_) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
            TransformFunction::RotateZ(deg) => {
                let rad = deg.to_radians() as f32;
                let cos = rad.cos();
                let sin = rad.sin();
                (cos, sin, -sin, cos, 0.0, 0.0)
            }
            TransformFunction::Rotate3d(_, _, _, _) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
            TransformFunction::Scale3d(sx, sy, _) => (*sx as f32, 0.0, 0.0, *sy as f32, 0.0, 0.0),
            TransformFunction::Perspective(_) => (1.0, 0.0, 0.0, 1.0, 0.0, 0.0),
            TransformFunction::Matrix(ma, mb, mc, md, me, mf) => {
                (*ma as f32, *mb as f32, *mc as f32, *md as f32, *me as f32, *mf as f32)
            }
        };

        // 矩阵乘法：current = current * func
        let new_a = a * fa + c * fb;
        let new_b = b * fa + d * fb;
        let new_c = a * fc + c * fd;
        let new_d = b * fc + d * fd;
        let new_tx = a * ftx + c * fty + tx;
        let new_ty = b * ftx + d * fty + ty;
        a = new_a;
        b = new_b;
        c = new_c;
        d = new_d;
        tx = new_tx;
        ty = new_ty;
    }

    // 应用 transform-origin 偏移
    // 最终变换：translate(origin) * matrix * translate(-origin)
    // 对 affine [a,b,c,d,tx,ty] 来说：
    // new_tx = origin_x * (1 - a) - origin_y * c + tx
    // new_ty = -origin_x * b + origin_y * (1 - d) + ty
    let final_tx = origin_x * (1.0 - a) - origin_y * c + tx;
    let final_ty = -origin_x * b + origin_y * (1.0 - d) + ty;

    // 检查是否为 identity 变换
    let is_identity = (a - 1.0).abs() < 1e-6
        && b.abs() < 1e-6
        && c.abs() < 1e-6
        && (d - 1.0).abs() < 1e-6
        && final_tx.abs() < 1e-6
        && final_ty.abs() < 1e-6;
    if is_identity {
        return None;
    }

    Some(TransformPrimitive {
        rect: *rect,
        origin_x,
        origin_y,
        a,
        b,
        c,
        d,
        tx: final_tx,
        ty: final_ty,
    })
}

fn resolve_transform_origin_length(value: &LengthValue, box_size: f32, font_size_px: f64) -> f32 {
    match value {
        LengthValue::Percentage(p) => box_size * (*p as f32 / 100.0),
        LengthValue::Auto | LengthValue::MinContent | LengthValue::MaxContent | LengthValue::FitContent(_) => {
            box_size / 2.0
        }
        other => zero_style_system::computed::resolve_length(other, font_size_px, None, None) as f32,
    }
}

/// 如果样式包含非平移变换，将 TransformPrimitive 添加到图元列表。
pub fn apply_transform(style: &ComputedStyle, rect: &Rect, primitives: &mut RenderPrimitives) {
    if let Some(tp) = compute_transform_matrix(style, rect) {
        primitives.add_transform(tp);
    }
}

/// 将填充矩形裁剪到指定区域内（原地修改）。
///
/// 从 `start` 索引开始的所有填充矩形会被裁剪到 `clip_rect` 内。
pub fn clip_fills(fills: &mut [zero_render_foundation::primitive::FillPrimitive], start: usize, clip_rect: &Rect) {
    for fill in fills.iter_mut().skip(start) {
        let r = &mut fill.rect;
        let left = r.left().max(clip_rect.left());
        let top = r.top().max(clip_rect.top());
        let right = r.right().min(clip_rect.right());
        let bottom = r.bottom().min(clip_rect.bottom());
        if right <= left || bottom <= top {
            // 完全在裁剪区域外，清零
            r.size.width = 0.0;
            r.size.height = 0.0;
        } else {
            r.origin.x = left;
            r.origin.y = top;
            r.size.width = right - left;
            r.size.height = bottom - top;
        }
    }
}

/// 将字形裁剪到指定区域内（原地修改）。
///
/// 从 `start` 索引开始的所有字形，如果完全在裁剪区域外则标记为 glyph_id=0。
pub fn clip_glyphs(glyphs: &mut [zero_render_foundation::primitive::GlyphPrimitive], start: usize, clip_rect: &Rect) {
    for g in glyphs.iter_mut().skip(start) {
        // 字形位置是左上角，假定宽高约等于 font_size
        let right = g.x + g.font_size;
        let bottom = g.y + g.font_size;
        if right <= clip_rect.left()
            || bottom <= clip_rect.top()
            || g.x >= clip_rect.right()
            || g.y >= clip_rect.bottom()
        {
            g.glyph_id = 0; // 标记为不可见
            g.font_size = 0.0;
        }
    }
}

/// 对快照之后新增的所有图元应用多边形裁剪（用于 clip-path: circle/ellipse/polygon）。
///
/// 使用凸多边形的扫描线裁剪：将每个填充矩形与多边形求交，
/// 结果为多个子矩形。简化处理：使用包围盒裁剪 + 丢弃完全在多边形外的图元。
pub fn clip_all_primitives_to_polygon(
    primitives: &mut RenderPrimitives,
    from: &PrimitiveCounts,
    polygon: &[(f32, f32)],
) {
    if polygon.len() < 3 {
        return;
    }

    // 计算多边形包围盒
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for &(x, y) in polygon {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    let bbox = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);

    // 第一步：用包围盒裁剪所有图元
    clip_all_primitives_to_rect(primitives, from, &bbox);

    // 第二步：对 fills 进行精确多边形裁剪
    // 将每个 fill 矩形与多边形求交，生成多个子矩形
    let mut new_fills = Vec::new();
    let fills_to_clip: Vec<_> = primitives.fills.drain(from.fills..).collect();
    for fill in fills_to_clip {
        let clipped = clip_fill_to_polygon(&fill, polygon);
        new_fills.extend(clipped);
    }
    primitives.fills.extend(new_fills);

    // 对 glyphs 进行精确裁剪（丢弃中心不在多边形内的字形）
    for g in primitives.glyphs.iter_mut().skip(from.glyphs) {
        if g.glyph_id == 0 {
            continue;
        }
        let cx = g.x + g.font_size / 2.0;
        let cy = g.y + g.font_size / 2.0;
        if !point_in_polygon(cx, cy, polygon) {
            g.glyph_id = 0;
            g.font_size = 0.0;
        }
    }
}

/// 将填充矩形裁剪到多边形内部。
///
/// 使用扫描线方法：对矩形的每行像素，计算与多边形边的交点，
/// 生成裁剪后的子矩形片段。
fn clip_fill_to_polygon(
    fill: &zero_render_foundation::primitive::FillPrimitive,
    polygon: &[(f32, f32)],
) -> Vec<zero_render_foundation::primitive::FillPrimitive> {
    use zero_render_foundation::primitive::FillPrimitive;

    let r = &fill.rect;
    if r.size.width <= 0.0 || r.size.height <= 0.0 {
        return vec![];
    }

    // 简化：使用逐行扫描线（步长为像素高度）
    // 对每一行，找到多边形在该行的覆盖区间
    let step = 1.0_f32.max(r.size.height / 20.0); // 最少 20 步
    let mut result = Vec::new();
    let mut y = r.top();
    while y < r.bottom() {
        let y_end = (y + step).min(r.bottom());
        // 找到该行与多边形的所有交点
        let mut intersections = Vec::new();
        let n = polygon.len();
        for i in 0..n {
            let (x1, y1) = polygon[i];
            let (x2, y2) = polygon[(i + 1) % n];
            if (y1 < y && y2 >= y) || (y2 < y && y1 >= y) {
                let t = (y - y1) / (y2 - y1);
                let ix = x1 + t * (x2 - x1);
                intersections.push(ix);
            }
        }
        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        // 成对取交点生成区间
        let mut idx = 0;
        while idx + 1 < intersections.len() {
            let left = intersections[idx].max(r.left());
            let right = intersections[idx + 1].min(r.right());
            if right > left {
                result.push(FillPrimitive {
                    rect: Rect::new(left, y, right - left, y_end - y),
                    color: fill.color,
                });
            }
            idx += 2;
        }
        y = y_end;
    }
    result
}

/// 判断点是否在多边形内部（射线法）。
fn point_in_polygon(px: f32, py: f32, polygon: &[(f32, f32)]) -> bool {
    let n = polygon.len();
    let mut inside = false;
    let mut j = n - 1;
    for i in 0..n {
        let (xi, yi) = polygon[i];
        let (xj, yj) = polygon[j];
        if ((yi > py) != (yj > py)) && (px < (xj - xi) * (py - yi) / (yj - yi) + xi) {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// 将圆形近似为多边形（用于 clip-path: circle()）。
pub fn circle_to_polygon(cx: f32, cy: f32, r: f32, segments: usize) -> Vec<(f32, f32)> {
    let mut points = Vec::with_capacity(segments);
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        points.push((cx + r * angle.cos(), cy + r * angle.sin()));
    }
    points
}

/// 将椭圆近似为多边形（用于 clip-path: ellipse()）。
pub fn ellipse_to_polygon(cx: f32, cy: f32, rx: f32, ry: f32, segments: usize) -> Vec<(f32, f32)> {
    let mut points = Vec::with_capacity(segments);
    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        points.push((cx + rx * angle.cos(), cy + ry * angle.sin()));
    }
    points
}

/// 渲染图元数量快照（用于 opacity 应用范围判断）。
pub struct PrimitiveCounts {
    /// 填充图元数量。
    pub fills: usize,
    /// 圆角矩形图元数量。
    pub rounded_rects: usize,
    /// 渐变图元数量。
    pub gradients: usize,
    /// 阴影图元数量。
    pub shadows: usize,
    /// 图片图元数量。
    pub images: usize,
    /// 字形图元数量。
    pub glyphs: usize,
    /// 描边图元数量。
    pub strokes: usize,
}

impl PrimitiveCounts {
    /// 从当前 RenderPrimitives 创建快照。
    pub fn snapshot(p: &RenderPrimitives) -> Self {
        Self {
            fills: p.fills.len(),
            rounded_rects: p.rounded_rects.len(),
            gradients: p.gradients.len(),
            shadows: p.shadows.len(),
            images: p.images.len(),
            glyphs: p.glyphs.len(),
            strokes: p.strokes.len(),
        }
    }
}

/// 对快照之后新增的所有图元应用矩形裁剪（用于 clip-path: inset()）。
///
/// 与 clip_fills/clip_glyphs 类似，但处理全部图元类型。
pub fn clip_all_primitives_to_rect(primitives: &mut RenderPrimitives, from: &PrimitiveCounts, clip_rect: &Rect) {
    clip_fills(&mut primitives.fills, from.fills, clip_rect);
    clip_glyphs(&mut primitives.glyphs, from.glyphs, clip_rect);
    // 裁剪圆角矩形
    for rr in primitives.rounded_rects.iter_mut().skip(from.rounded_rects) {
        let r = &mut rr.rect;
        let left = r.left().max(clip_rect.left());
        let top = r.top().max(clip_rect.top());
        let right = r.right().min(clip_rect.right());
        let bottom = r.bottom().min(clip_rect.bottom());
        if right <= left || bottom <= top {
            r.size.width = 0.0;
            r.size.height = 0.0;
        } else {
            r.origin.x = left;
            r.origin.y = top;
            r.size.width = right - left;
            r.size.height = bottom - top;
        }
    }
    // 裁剪渐变
    for grad in primitives.gradients.iter_mut().skip(from.gradients) {
        let r = &mut grad.rect;
        let left = r.left().max(clip_rect.left());
        let top = r.top().max(clip_rect.top());
        let right = r.right().min(clip_rect.right());
        let bottom = r.bottom().min(clip_rect.bottom());
        if right <= left || bottom <= top {
            r.size.width = 0.0;
            r.size.height = 0.0;
        } else {
            r.origin.x = left;
            r.origin.y = top;
            r.size.width = right - left;
            r.size.height = bottom - top;
        }
    }
    // 裁剪阴影（简化：矩形范围）
    for shadow in primitives.shadows.iter_mut().skip(from.shadows) {
        let r = &mut shadow.rect;
        let left = r.left().max(clip_rect.left());
        let top = r.top().max(clip_rect.top());
        let right = r.right().min(clip_rect.right());
        let bottom = r.bottom().min(clip_rect.bottom());
        if right <= left || bottom <= top {
            r.size.width = 0.0;
            r.size.height = 0.0;
        } else {
            r.origin.x = left;
            r.origin.y = top;
            r.size.width = right - left;
            r.size.height = bottom - top;
        }
    }
    // 裁剪图片：**crop 语义（非 rescale）**。
    // 关键：保持 img.rect 不变（source 始终映射到完整 rect，保持原始分辨率），
    // 仅把可见窗口 img.clip 收窄为「当前有效区域 ∩ clip_rect」。
    // render_image 据此窗口裁剪绘制；旧实现 shrink rect 会导致 renderer 把整张
    // source 重映射进缩小后的 rect（rescale），破坏 clip:rect / overflow:hidden
    // 的「裁剪=遮罩」语义（clip-rect-vrl 三联根因，R294）。
    for img in primitives.images.iter_mut().skip(from.images) {
        let rect = img.rect;
        let cur = img.clip.unwrap_or(rect);
        let left = cur.left().max(clip_rect.left());
        let top = cur.top().max(clip_rect.top());
        let right = cur.right().min(clip_rect.right());
        let bottom = cur.bottom().min(clip_rect.bottom());
        if right <= left || bottom <= top {
            // 完全在裁剪区外：零尺寸 clip 窗口（render_image 见空交集跳过）
            img.clip = Some(Rect::new(left, top, 0.0, 0.0));
        } else {
            img.clip = Some(Rect::new(left, top, right - left, bottom - top));
        }
    }
    // 裁剪描边线段：线段两端都在裁剪区域外时标记为不可见
    // 注意：点在裁剪区域外 = 超出任意一条边（|| 连接各边判断）
    for stroke in primitives.strokes.iter_mut().skip(from.strokes) {
        let p1_outside = stroke.x1 < clip_rect.left()
            || stroke.x1 > clip_rect.right()
            || stroke.y1 < clip_rect.top()
            || stroke.y1 > clip_rect.bottom();
        let p2_outside = stroke.x2 < clip_rect.left()
            || stroke.x2 > clip_rect.right()
            || stroke.y2 < clip_rect.top()
            || stroke.y2 > clip_rect.bottom();
        if p1_outside && p2_outside {
            stroke.width = 0.0;
        }
    }
}

/// 对快照之后新增的所有图元应用 opacity（alpha 衰减）。
pub fn apply_opacity_to_new_primitives(primitives: &mut RenderPrimitives, from: &PrimitiveCounts, opacity: f32) {
    for fill in primitives.fills.iter_mut().skip(from.fills) {
        fill.color.a = (fill.color.a as f32 * opacity).round() as u8;
    }
    for rr in primitives.rounded_rects.iter_mut().skip(from.rounded_rects) {
        rr.color.a = (rr.color.a as f32 * opacity).round() as u8;
    }
    for grad in primitives.gradients.iter_mut().skip(from.gradients) {
        for stop in &mut grad.stops {
            stop.color.a = (stop.color.a as f32 * opacity).round() as u8;
        }
    }
    for shadow in primitives.shadows.iter_mut().skip(from.shadows) {
        shadow.color.a = (shadow.color.a as f32 * opacity).round() as u8;
    }
    for img in primitives.images.iter_mut().skip(from.images) {
        // ImagePrimitive 没有 color 字段，opacity 通过绘制时应用
        let _ = img;
    }
    for glyph in primitives.glyphs.iter_mut().skip(from.glyphs) {
        glyph.color.a = (glyph.color.a as f32 * opacity).round() as u8;
    }
    for stroke in primitives.strokes.iter_mut().skip(from.strokes) {
        stroke.color.a = (stroke.color.a as f32 * opacity).round() as u8;
    }
}

/// 根据 CSS text-transform 转换文本。
///
/// 实际逻辑已在 `TextTransformValue::apply`（style-system）实现，供 layout-engine
/// 在 `collect_inline_items` 期与 paint 期共享同一转换（R1012 Phase A IFC 统一）。
/// 本函数保留为 paint 模块的稳定入口（`&TextTransformValue` 入参），委托到该方法。
pub fn apply_text_transform(text: &str, transform: &TextTransformValue) -> String {
    transform.apply(text)
}

/// 四角圆角半径集合。
#[derive(Debug, Clone, Copy)]
pub struct BorderRadiusSpec {
    /// 左上角半径。
    pub top_left: f32,
    /// 右上角半径。
    pub top_right: f32,
    /// 右下角半径。
    pub bottom_right: f32,
    /// 左下角半径。
    pub bottom_left: f32,
}

impl BorderRadiusSpec {
    /// 从 ComputedStyle 提取圆角半径。
    pub fn from_style(style: &ComputedStyle) -> Self {
        Self {
            top_left: length_to_f32(&style.border_top_left_radius),
            top_right: length_to_f32(&style.border_top_right_radius),
            bottom_right: length_to_f32(&style.border_bottom_right_radius),
            bottom_left: length_to_f32(&style.border_bottom_left_radius),
        }
    }

    /// 从 ComputedStyle 提取圆角半径，按 border-box 尺寸解析百分比与含百分比 calc。
    ///
    /// `box_w`/`box_h` 为元素 border-box 尺寸（layout 后已知）。CSS Backgrounds §5.1：
    /// border-radius 百分比相对于 border-box 对应轴。百分比与含百分比的
    /// calc()/min()/max()/clamp() 在 computed 阶段无法解析（需容器尺寸），此处按
    /// border-box 解析。不含百分比的长度（含 calc）已在 computed 阶段解析为 Px，
    /// 经此路径原值返回（byte-identical 于 [`Self::from_style`]）。
    ///
    /// driving：R2314 border-radius 百分比（`border-radius: 50%` 圆形）与含百分比 calc。
    pub fn from_style_with_box(style: &ComputedStyle, box_w: f32, box_h: f32) -> Self {
        let max_r = (box_w.min(box_h) / 2.0).max(0.0);
        let font_size_px = zero_style_system::computed::resolve_length(&style.font_size, 16.0, None, None);
        Self {
            top_left: resolve_radius_length(&style.border_top_left_radius, box_w, max_r, font_size_px),
            top_right: resolve_radius_length(&style.border_top_right_radius, box_w, max_r, font_size_px),
            bottom_right: resolve_radius_length(&style.border_bottom_right_radius, box_w, max_r, font_size_px),
            bottom_left: resolve_radius_length(&style.border_bottom_left_radius, box_w, max_r, font_size_px),
        }
    }

    /// 所有圆角都为零。
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0 && self.top_right == 0.0 && self.bottom_right == 0.0 && self.bottom_left == 0.0
    }
}

/// 解析 border-radius 单角长度值为像素半径。
///
/// - `Px`：已解析的绝对值（em/rem/vw 与不含百分比的 calc 已在 computed 阶段解析为 Px），
///   原值返回（不钳制，保持既有 px 行为 byte-identical）。
/// - `Percentage`：CSS Backgrounds §5.1，相对于 border-box 宽度（水平半径语义）。
/// - `Calc`：含百分比的 calc/min/max/clamp（无百分比的已在 computed 解析为 Px），
///   以 `box_w` 为百分比基准（parent_length）求值；不可解（如需 font 上下文）→ 回退 0.0。
///
/// 百分比与 calc 结果钳制到 `max_r`（= min(box_w, box_h) / 2）：CSS 规定单角半径不超过
/// 边长一半，避免圆角超出边框盒致视觉溢出。Px 值不钳制（既有行为不变）。
fn resolve_radius_length(v: &LengthValue, box_w: f32, max_r: f32, font_size_px: f64) -> f32 {
    match v {
        LengthValue::Px(p) => *p as f32,
        LengthValue::Percentage(pct) => {
            let r = (pct / 100.0) * box_w as f64;
            r.min(max_r as f64) as f32
        }
        LengthValue::Calc(expr) => eval_calc(expr, Some(box_w as f64))
            .map(|r| r.min(max_r as f64) as f32)
            .unwrap_or(0.0),
        // computed 阶段未解析的其余真实长度单位：按当前 font-size 在 paint 边界解析。
        _ => zero_style_system::computed::resolve_length(v, font_size_px, None, None) as f32,
    }
}

/// 将 LengthValue 转换为 f32（仅支持 Px）。
pub fn length_to_f32(v: &LengthValue) -> f32 {
    match v {
        LengthValue::Px(p) => *p as f32,
        _ => 0.0,
    }
}

/// 解析 clip-path basic-shape `<length-percentage>` 为像素（CSS Masking §inset/circle/ellipse/polygon）。
///
/// 通用解析器：Px 原值；Em/Rem 按 font-size 解析；Percentage 相对调用方传入的 `box_dim`。
/// clip-path 的长度值位于 `ClipPathValue::*` enum 内，computed 阶段的 `resolve_length_field`
/// 不触及，故在 paint 补解析。调用方按形状语义传 `box_dim`：
/// - inset：top/bottom→height、left/right→width（R2365）
/// - circle 半径：sqrt(w²+h²)/√2；ellipse rx→width、ry→height（R2366）
/// - polygon 顶点 / circle·ellipse 圆心 position：x→width、y→height（R2366）
///
/// vw/vh/ch 等视口/字体度量单位近似为 0（clip-path 极罕用，缺视口上下文）。
pub fn resolve_inset_length(v: &LengthValue, box_dim: f32, font_size_px: f32) -> f32 {
    match v {
        LengthValue::Px(p) => *p as f32,
        LengthValue::Em(e) => (*e as f32) * font_size_px,
        LengthValue::Rem(e) => (*e as f32) * 16.0,
        LengthValue::Percentage(p) => (*p as f32 / 100.0) * box_dim,
        _ => 0.0,
    }
}

/// 将位置 LengthValue 解析为相对于容器尺寸的像素偏移。
/// 支持百分比和绝对长度值。
fn resolve_position(v: &LengthValue, container_size: f32) -> f32 {
    match v {
        LengthValue::Percentage(p) => *p as f32 / 100.0 * container_size,
        LengthValue::Px(p) => *p as f32,
        _ => container_size / 2.0, // 默认居中
    }
}

/// 简单的字符串哈希函数（用于从 URL 字符串生成 ImageKey）。
pub fn simple_hash(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for byte in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
    }
    hash
}

/// 判断 `href` 是否应原样使用（已有 scheme、片段锚点等），不参与相对 base 解析。
fn is_non_relative_href(href: &str) -> bool {
    if href.starts_with('#') {
        return true;
    }
    if href.starts_with("data:")
        || href.starts_with("http://")
        || href.starts_with("https://")
        || href.starts_with("zero://")
        || href.starts_with("file://")
        || href.starts_with("ftp://")
    {
        return true;
    }
    if let Some(colon) = href.find(':') {
        let scheme = &href[..colon];
        if !scheme.is_empty()
            && scheme
                .chars()
                .all(|c| c.is_ascii_alphabetic() || c.is_ascii_digit() || matches!(c, '+' | '-' | '.'))
            && scheme.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        {
            return true;
        }
    }
    false
}

/// 将相对 URL 解析为绝对 URL（解析失败时原样返回 `href`）。
pub fn resolve_document_url(base_url: &str, href: &str) -> String {
    let href = href.trim();
    if href.is_empty() || is_non_relative_href(href) {
        return href.to_string();
    }
    match url::Url::parse(base_url).and_then(|base| base.join(href)) {
        Ok(abs) => abs.to_string(),
        Err(_) => href.to_string(),
    }
}

/// 图像子资源 lookup key：始终对**解析后的绝对 URL** 哈希，与 webview 抓取路径一致。
pub fn image_resource_key(src: &str, document_url: Option<&str>) -> u64 {
    let resolved = document_url
        .map(|base| resolve_document_url(base, src))
        .unwrap_or_else(|| src.to_string());
    simple_hash(&resolved)
}

/// 将 CSS GradientValue 转换为 GradientPrimitive。
///
/// 目前支持 linear-gradient 和 radial-gradient。
/// conic-gradient 暂不渲染（返回 None）。
pub fn gradient_to_primitive(
    gradient: &GradientValue,
    rect: &Rect,
    element_color: &ColorValue,
) -> Option<GradientPrimitive> {
    let w = rect.size.width;
    let h = rect.size.height;
    match gradient {
        GradientValue::Linear(lg) => {
            let kind = linear_direction_to_kind(&lg.direction, rect);
            let stops = convert_color_stops(&lg.stops, element_color);
            Some(GradientPrimitive {
                interpolation: map_interpolation(lg.interpolation),
                rect: *rect,
                kind,
                stops,
                repeating: lg.repeating,
            })
        }
        GradientValue::Radial(rg) => {
            let cx = rect.left() + resolve_position(&rg.position_x, w);
            let cy = rect.top() + resolve_position(&rg.position_y, h);
            let outer = match &rg.size {
                RadialSize::ClosestSide => (cx - rect.left())
                    .min(rect.right() - cx)
                    .min(cy - rect.top())
                    .min(rect.bottom() - cy),
                RadialSize::FarthestSide => (cx - rect.left())
                    .max(rect.right() - cx)
                    .max(cy - rect.top())
                    .max(rect.bottom() - cy),
                RadialSize::ClosestCorner => {
                    let tl = (cx - rect.left()).hypot(cy - rect.top());
                    let tr = (rect.right() - cx).hypot(cy - rect.top());
                    let bl = (cx - rect.left()).hypot(rect.bottom() - cy);
                    let br = (rect.right() - cx).hypot(rect.bottom() - cy);
                    tl.min(tr).min(bl).min(br)
                }
                RadialSize::FarthestCorner => {
                    let tl = (cx - rect.left()).hypot(cy - rect.top());
                    let tr = (rect.right() - cx).hypot(cy - rect.top());
                    let bl = (cx - rect.left()).hypot(rect.bottom() - cy);
                    let br = (rect.right() - cx).hypot(rect.bottom() - cy);
                    tl.max(tr).max(bl).max(br)
                }
                RadialSize::Length(lv) => length_to_f32(lv),
            };
            let stops = convert_color_stops(&rg.stops, element_color);
            Some(GradientPrimitive {
                interpolation: map_interpolation(rg.interpolation),
                rect: *rect,
                kind: GradientKind::Radial {
                    cx,
                    cy,
                    inner_radius: 0.0,
                    outer_radius: outer.max(0.01),
                },
                stops,
                repeating: rg.repeating,
            })
        }
        GradientValue::Conic(cg) => {
            let cx = rect.left() + resolve_position(&cg.position_x, w);
            let cy = rect.top() + resolve_position(&cg.position_y, h);
            let start_angle = cg.from_angle.to_radians() as f32;
            let stops = convert_color_stops(&cg.stops, element_color);
            Some(GradientPrimitive {
                interpolation: map_interpolation(cg.interpolation),
                rect: *rect,
                kind: GradientKind::Conic { cx, cy, start_angle },
                stops,
                repeating: cg.repeating,
            })
        }
    }
}

/// 将 css-parser `ColorInterpolation` 映射为 render-foundation `GradientInterpolation`
/// （CSS Color 4 `gradient in <colorspace>`，R2289）。
fn map_interpolation(i: ColorInterpolation) -> GradientInterpolation {
    let space = match i.space {
        ColorInterpolationSpace::Srgb => GradientColorSpace::Srgb,
        ColorInterpolationSpace::SrgbLinear => GradientColorSpace::SrgbLinear,
        ColorInterpolationSpace::Lab => GradientColorSpace::Lab,
        ColorInterpolationSpace::Oklab => GradientColorSpace::Oklab,
        ColorInterpolationSpace::Lch => GradientColorSpace::Lch,
        ColorInterpolationSpace::Oklch => GradientColorSpace::Oklch,
    };
    let hue = match i.hue {
        ColorHueMethod::Shorter => HueMethod::Shorter,
        ColorHueMethod::Longer => HueMethod::Longer,
        ColorHueMethod::Increasing => HueMethod::Increasing,
        ColorHueMethod::Decreasing => HueMethod::Decreasing,
    };
    GradientInterpolation { space, hue }
}

/// 将线性渐变方向转换为 GradientKind::Linear。
pub fn linear_direction_to_kind(dir: &GradientDirection, rect: &Rect) -> GradientKind {
    let w = rect.size.width;
    let h = rect.size.height;
    let cx = rect.left() + w / 2.0;
    let cy = rect.top() + h / 2.0;
    match dir {
        GradientDirection::ToBottom => GradientKind::Linear {
            x0: cx,
            y0: rect.top(),
            x1: cx,
            y1: rect.bottom(),
        },
        GradientDirection::ToTop => GradientKind::Linear {
            x0: cx,
            y0: rect.bottom(),
            x1: cx,
            y1: rect.top(),
        },
        GradientDirection::ToRight => GradientKind::Linear {
            x0: rect.left(),
            y0: cy,
            x1: rect.right(),
            y1: cy,
        },
        GradientDirection::ToLeft => GradientKind::Linear {
            x0: rect.right(),
            y0: cy,
            x1: rect.left(),
            y1: cy,
        },
        GradientDirection::ToTopRight => GradientKind::Linear {
            x0: rect.left(),
            y0: rect.bottom(),
            x1: rect.right(),
            y1: rect.top(),
        },
        GradientDirection::ToTopLeft => GradientKind::Linear {
            x0: rect.right(),
            y0: rect.bottom(),
            x1: rect.left(),
            y1: rect.top(),
        },
        GradientDirection::ToBottomRight => GradientKind::Linear {
            x0: rect.left(),
            y0: rect.top(),
            x1: rect.right(),
            y1: rect.bottom(),
        },
        GradientDirection::ToBottomLeft => GradientKind::Linear {
            x0: rect.right(),
            y0: rect.top(),
            x1: rect.left(),
            y1: rect.bottom(),
        },
        GradientDirection::Angle(deg) => {
            // 角度转坐标：0deg = to top, 90deg = to right, 180deg = to bottom
            let rad = (deg - 90.0).to_radians();
            let dx = rad.cos();
            let dy = rad.sin();
            let half_diag = w.hypot(h) / 2.0;
            GradientKind::Linear {
                x0: cx - dx as f32 * half_diag,
                y0: cy - dy as f32 * half_diag,
                x1: cx + dx as f32 * half_diag,
                y1: cy + dy as f32 * half_diag,
            }
        }
    }
}

/// 将 CSS 渐变色标转换为渲染层 GradientStop。
pub fn convert_color_stops(stops: &[GradientColorStop], element_color: &ColorValue) -> Vec<GradientStop> {
    let n = stops.len();
    stops
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let offset = s
                .position
                .as_ref()
                .map(|lv| match lv {
                    LengthValue::Percentage(p) => *p as f32 / 100.0,
                    LengthValue::Px(px) => *px as f32,
                    // calc/min/max/clamp 求值为 px（无 parent_length：百分比不可解→None→回退 0.0）。
                    // driving: css-images gradient-infinity（calc(1px/0) / calc(Infinity*1px)）。
                    LengthValue::Calc(expr) => eval_calc(expr, None).unwrap_or(0.0) as f32,
                    _ => 0.0,
                })
                .unwrap_or(if n <= 1 { 0.0 } else { i as f32 / (n - 1) as f32 });
            GradientStop {
                offset,
                // R2370：currentColor 按**使用该渐变的元素**自身 color 解析（CSS Color §resolving）。
                // 旧 color_value_to_render 无元素上下文 → currentColor 回落黑色。driving: color-stop-currentcolor。
                color: resolve_color_current(&s.color, element_color),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_css_parser::values::{
        ColorValue, ConicGradient, GradientColorStop, GradientDirection, GradientValue, LengthValue, LinearGradient,
        RadialGradient, RadialShape, RadialSize, TransformFunction, TransformValue,
    };
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{FillPrimitive, FontId, GlyphPrimitive, GradientKind, RenderPrimitives};
    use zero_style_system::ComputedStyle;

    // ── apply_transform_offset ──────────────────────────────────────────

    #[test]
    fn test_transform_offset_none() {
        let style = ComputedStyle::default();
        let (dx, dy) = apply_transform_offset(&style, 10.0, 20.0);
        assert_eq!((dx, dy), (0.0, 0.0));
    }

    #[test]
    fn test_transform_offset_translate() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Translate(50.0, 30.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (50.0, 30.0));
    }

    #[test]
    fn test_transform_offset_translate_x() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::TranslateX(100.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (100.0, 0.0));
    }

    #[test]
    fn test_transform_offset_translate_y() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::TranslateY(75.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (0.0, 75.0));
    }

    #[test]
    fn test_transform_offset_multiple_translates() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![
            TransformFunction::TranslateX(10.0),
            TransformFunction::TranslateY(20.0),
            TransformFunction::Translate(5.0, 15.0),
        ]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (15.0, 35.0)); // 10+5, 20+15
    }

    #[test]
    fn test_transform_offset_rotate_no_offset() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(45.0)]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (0.0, 0.0));
    }

    #[test]
    fn test_transform_offset_scale_no_offset() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Scale(2.0, Some(3.0))]);
        let (dx, dy) = apply_transform_offset(&style, 0.0, 0.0);
        assert_eq!((dx, dy), (0.0, 0.0));
    }

    // ── compute_transform_matrix ────────────────────────────────────────

    #[test]
    fn test_compute_transform_none_returns_none() {
        let style = ComputedStyle::default();
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(compute_transform_matrix(&style, &rect).is_none());
    }

    #[test]
    fn test_compute_transform_translate_only_returns_none() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Translate(10.0, 20.0)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        // translate-only should NOT generate TransformPrimitive
        assert!(compute_transform_matrix(&style, &rect).is_none());
    }

    #[test]
    fn test_compute_transform_rotate_90() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(90.0)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        // Rotate 90°: cos(90°) ≈ 0, sin(90°) ≈ 1
        assert!(tp.a.abs() < 0.01, "a should be ~0, got {}", tp.a);
        assert!((tp.b - 1.0).abs() < 0.01, "b should be ~1, got {}", tp.b);
        assert!((tp.c + 1.0).abs() < 0.01, "c should be ~-1, got {}", tp.c);
        assert!(tp.d.abs() < 0.01, "d should be ~0, got {}", tp.d);

        // origin at center (50, 50)
        assert!((tp.origin_x - 50.0).abs() < 0.1);
        assert!((tp.origin_y - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_transform_scale_2x() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Scale(2.0, None)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        assert!((tp.a - 2.0).abs() < 0.01);
        assert!((tp.d - 2.0).abs() < 0.01);
        assert!(tp.b.abs() < 0.01);
        assert!(tp.c.abs() < 0.01);
    }

    #[test]
    fn test_compute_transform_scale_x_y() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Scale(3.0, Some(0.5))]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        assert!((tp.a - 3.0).abs() < 0.01);
        assert!((tp.d - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_compute_transform_custom_origin() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(90.0)]);
        style.transform_origin_x = LengthValue::Px(0.0);
        style.transform_origin_y = LengthValue::Px(0.0);
        let rect = Rect::new(10.0, 20.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        // origin at top-left corner (10, 20)
        assert!((tp.origin_x - 10.0).abs() < 0.1);
        assert!((tp.origin_y - 20.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_transform_origin_relative_lengths() {
        let mut style = ComputedStyle::default();
        style.font_size = LengthValue::Px(20.0);
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(90.0)]);
        style.transform_origin_x = LengthValue::Em(1.0);
        style.transform_origin_y = LengthValue::Em(2.0);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        assert!((tp.origin_x - 20.0).abs() < 0.1);
        assert!((tp.origin_y - 40.0).abs() < 0.1);
    }

    #[test]
    fn test_compute_transform_combined_translate_rotate() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![
            TransformFunction::Translate(10.0, 20.0),
            TransformFunction::Rotate(45.0),
        ]);
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        // rotate 45°: cos = sin ≈ 0.707
        let cos45 = 45.0_f64.to_radians().cos() as f32;
        let sin45 = 45.0_f64.to_radians().sin() as f32;
        assert!((tp.a - cos45).abs() < 0.01);
        assert!((tp.b - sin45).abs() < 0.01);
        assert!((tp.c + sin45).abs() < 0.01);
        assert!((tp.d - cos45).abs() < 0.01);
    }

    #[test]
    fn test_compute_transform_skew_x() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Skew(45.0, None)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tp = compute_transform_matrix(&style, &rect).expect("should have transform");

        assert!((tp.a - 1.0).abs() < 0.01);
        assert!(tp.b.abs() < 0.01);
        assert!((tp.c - 1.0).abs() < 0.01, "tan(45°) = 1.0, got {}", tp.c);
        assert!((tp.d - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_compute_transform_identity_skew_0() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Skew(0.0, None)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        // skew(0) is identity → should return None
        assert!(compute_transform_matrix(&style, &rect).is_none());
    }

    #[test]
    fn test_apply_transform_noop_for_none() {
        let style = ComputedStyle::default();
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut p = RenderPrimitives::default();
        apply_transform(&style, &rect, &mut p);
        assert!(p.transforms.is_empty());
    }

    #[test]
    fn test_apply_transform_adds_for_rotate() {
        let mut style = ComputedStyle::default();
        style.transform = TransformValue::List(vec![TransformFunction::Rotate(30.0)]);
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut p = RenderPrimitives::default();
        apply_transform(&style, &rect, &mut p);
        assert_eq!(p.transforms.len(), 1);
    }

    // ── clip_fills ──────────────────────────────────────────────────────

    #[test]
    fn test_clip_fills_inside_rect() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut fills = vec![FillPrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::rgb(255, 0, 0),
        }];
        clip_fills(&mut fills, 0, &clip);
        assert_eq!(fills[0].rect.origin.x, 10.0);
        assert_eq!(fills[0].rect.size.width, 50.0);
    }

    #[test]
    fn test_clip_fills_partially_outside() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut fills = vec![FillPrimitive {
            rect: Rect::new(80.0, 80.0, 50.0, 50.0),
            color: Color::rgb(0, 255, 0),
        }];
        clip_fills(&mut fills, 0, &clip);
        assert_eq!(fills[0].rect.origin.x, 80.0);
        assert_eq!(fills[0].rect.origin.y, 80.0);
        assert_eq!(fills[0].rect.size.width, 20.0); // 100 - 80
        assert_eq!(fills[0].rect.size.height, 20.0); // 100 - 80
    }

    #[test]
    fn test_clip_fills_fully_outside() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut fills = vec![FillPrimitive {
            rect: Rect::new(200.0, 200.0, 50.0, 50.0),
            color: Color::rgb(0, 0, 255),
        }];
        clip_fills(&mut fills, 0, &clip);
        assert_eq!(fills[0].rect.size.width, 0.0);
        assert_eq!(fills[0].rect.size.height, 0.0);
    }

    #[test]
    fn test_clip_fills_skip_start() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut fills = vec![
            FillPrimitive {
                rect: Rect::new(200.0, 200.0, 50.0, 50.0),
                color: Color::rgb(0, 0, 255),
            },
            FillPrimitive {
                rect: Rect::new(10.0, 10.0, 50.0, 50.0),
                color: Color::rgb(255, 0, 0),
            },
        ];
        clip_fills(&mut fills, 1, &clip);
        // First fill untouched
        assert_eq!(fills[0].rect.origin.x, 200.0);
        // Second fill clipped (but stays inside)
        assert_eq!(fills[1].rect.origin.x, 10.0);
    }

    // ── clip_glyphs ─────────────────────────────────────────────────────

    #[test]
    fn test_clip_glyphs_inside() {
        let clip = Rect::new(0.0, 0.0, 200.0, 200.0);
        let mut glyphs = vec![GlyphPrimitive {
            x: 10.0,
            y: 10.0,
            font_size: 16.0,
            color: Color::rgb(0, 0, 0),
            glyph_id: 42,
            font_glyph_index: None,
            source: None,
            font_id: FontId(1),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        }];
        clip_glyphs(&mut glyphs, 0, &clip);
        assert_eq!(glyphs[0].glyph_id, 42); // untouched
    }

    #[test]
    fn test_clip_glyphs_outside() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut glyphs = vec![GlyphPrimitive {
            x: 200.0,
            y: 200.0,
            font_size: 16.0,
            color: Color::rgb(0, 0, 0),
            glyph_id: 42,
            font_glyph_index: None,
            source: None,
            font_id: FontId(1),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        }];
        clip_glyphs(&mut glyphs, 0, &clip);
        assert_eq!(glyphs[0].glyph_id, 0); // marked invisible
        assert_eq!(glyphs[0].font_size, 0.0);
    }

    #[test]
    fn test_clip_glyphs_partial_skip() {
        let clip = Rect::new(0.0, 0.0, 100.0, 100.0);
        let mut glyphs = vec![
            GlyphPrimitive {
                x: 200.0,
                y: 200.0,
                font_size: 16.0,
                color: Color::rgb(0, 0, 0),
                glyph_id: 1,
                font_glyph_index: None,
                source: None,
                font_id: FontId(1),
                font_variation_id: None,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
                synthetic_italic: false,
            },
            GlyphPrimitive {
                x: 10.0,
                y: 10.0,
                font_size: 16.0,
                color: Color::rgb(0, 0, 0),
                glyph_id: 2,
                font_glyph_index: None,
                source: None,
                font_id: FontId(1),
                font_variation_id: None,
                bitmap_width: None,
                bitmap_height: None,
                rotation: 0.0,
                synthetic_italic: false,
            },
        ];
        clip_glyphs(&mut glyphs, 1, &clip);
        assert_eq!(glyphs[0].glyph_id, 1); // untouched (before start)
        assert_eq!(glyphs[1].glyph_id, 2); // inside clip
    }

    // ── PrimitiveCounts / apply_opacity ─────────────────────────────────

    #[test]
    fn test_primitive_counts_snapshot() {
        let mut p = RenderPrimitives::default();
        p.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::rgba(255, 0, 0, 255),
        });
        p.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 20.0, 20.0),
            color: Color::rgba(0, 255, 0, 255),
        });
        let snap = PrimitiveCounts::snapshot(&p);
        assert_eq!(snap.fills, 2);
        assert_eq!(snap.glyphs, 0);
    }

    #[test]
    fn test_apply_opacity_reduces_alpha() {
        let mut p = RenderPrimitives::default();
        let before = PrimitiveCounts::snapshot(&p);
        p.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            color: Color::rgba(255, 0, 0, 200),
        });
        p.glyphs.push(GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 16.0,
            color: Color::rgba(0, 0, 0, 128),
            glyph_id: 1,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        });
        apply_opacity_to_new_primitives(&mut p, &before, 0.5);
        assert_eq!(p.fills[0].color.a, 100); // 200 * 0.5
        assert_eq!(p.glyphs[0].color.a, 64); // 128 * 0.5
    }

    #[test]
    fn test_apply_opacity_zero() {
        let mut p = RenderPrimitives::default();
        let before = PrimitiveCounts::snapshot(&p);
        p.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::rgba(255, 0, 0, 255),
        });
        apply_opacity_to_new_primitives(&mut p, &before, 0.0);
        assert_eq!(p.fills[0].color.a, 0);
    }

    #[test]
    fn test_apply_opacity_full() {
        let mut p = RenderPrimitives::default();
        let before = PrimitiveCounts::snapshot(&p);
        p.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::rgba(255, 0, 0, 128),
        });
        apply_opacity_to_new_primitives(&mut p, &before, 1.0);
        assert_eq!(p.fills[0].color.a, 128);
    }

    #[test]
    fn test_apply_opacity_skips_before_snapshot() {
        let mut p = RenderPrimitives::default();
        p.fills.push(FillPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::rgba(255, 0, 0, 200),
        });
        let snap = PrimitiveCounts::snapshot(&p);
        apply_opacity_to_new_primitives(&mut p, &snap, 0.5);
        // No new primitives added, so existing one should be untouched
        assert_eq!(p.fills[0].color.a, 200);
    }

    // ── apply_text_transform ────────────────────────────────────────────

    #[test]
    fn test_text_transform_none() {
        assert_eq!(
            apply_text_transform("hello World", &TextTransformValue::None),
            "hello World"
        );
    }

    #[test]
    fn test_text_transform_uppercase() {
        assert_eq!(apply_text_transform("hello", &TextTransformValue::Uppercase), "HELLO");
    }

    #[test]
    fn test_text_transform_lowercase() {
        assert_eq!(apply_text_transform("HELLO", &TextTransformValue::Lowercase), "hello");
    }

    #[test]
    fn test_text_transform_capitalize() {
        assert_eq!(
            apply_text_transform("hello world", &TextTransformValue::Capitalize),
            "Hello World"
        );
    }

    #[test]
    fn test_text_transform_capitalize_with_spaces() {
        assert_eq!(
            apply_text_transform("  multiple  spaces", &TextTransformValue::Capitalize),
            "  Multiple  Spaces"
        );
    }

    #[test]
    fn test_text_transform_capitalize_empty() {
        assert_eq!(apply_text_transform("", &TextTransformValue::Capitalize), "");
    }

    #[test]
    fn test_text_transform_capitalize_numbers() {
        assert_eq!(
            apply_text_transform("abc123def", &TextTransformValue::Capitalize),
            "Abc123def"
        );
    }

    // ── R2327：text-transform full-width / full-size-kana（CSS Text 3 §3.1）──

    #[test]
    fn test_r2327_text_transform_full_width() {
        // CSS Text 3 §3.1：ASCII 可打印 U+0021–U+007E → 全角 U+FF01–U+FF5E（+0xFEE0）；
        // 空格（U+0020）与非 ASCII 不变。driving: css-text text-transform-fullwidth-001/009。
        assert_eq!(
            apply_text_transform("Hello!", &TextTransformValue::FullWidth),
            "\u{FF28}\u{FF45}\u{FF4C}\u{FF4C}\u{FF4F}\u{FF01}", // Ｈｅｌｌｏ！
            "ASCII letters + punct -> fullwidth"
        );
        // 数字与符号（空格不转换）
        assert_eq!(
            apply_text_transform("A1 #", &TextTransformValue::FullWidth),
            "\u{FF21}\u{FF11} \u{FF03}", // Ａ１ ＃（空格保留，# → U+FF03）
            "digits/symbols -> fullwidth, space preserved"
        );
        // 空格（U+0020）保留不转换，两侧字母转全角
        let r = apply_text_transform("A B", &TextTransformValue::FullWidth);
        assert_eq!(
            r, "\u{FF21} \u{FF22}",
            "space U+0020 preserved (ASCII), letters fullwidth'd"
        );
        // 非 ASCII（中文）不变
        assert_eq!(apply_text_transform("中文", &TextTransformValue::FullWidth), "中文");
    }

    #[test]
    fn test_r2327_text_transform_full_size_kana() {
        // CSS Text 3 §3.1：小書き仮名 → 普通仮名。driving: css-text text-transform-full-size-kana-005。
        assert_eq!(
            apply_text_transform("ぁぃぅぇぉっゃゅょゎ", &TextTransformValue::FullSizeKana),
            "あいうえおつやゆよわ",
            "hiragana small -> regular"
        );
        assert_eq!(
            apply_text_transform("ァィゥェォッャュョォ", &TextTransformValue::FullSizeKana),
            "アイウエオツヤユヨオ",
            "katakana small -> regular (ォ->オ)"
        );
        // ヵ ヶ → カ ケ
        assert_eq!(apply_text_transform("ヶ", &TextTransformValue::FullSizeKana), "ケ");
        // 普通仮名不变
        assert_eq!(apply_text_transform("あい", &TextTransformValue::FullSizeKana), "あい");
    }

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

    // ── length_to_f32 ───────────────────────────────────────────────────

    #[test]
    fn test_length_to_f32_px() {
        assert_eq!(length_to_f32(&LengthValue::Px(42.0)), 42.0);
    }

    #[test]
    fn test_length_to_f32_non_px() {
        assert_eq!(length_to_f32(&LengthValue::Percentage(50.0)), 0.0);
    }

    // ── simple_hash ─────────────────────────────────────────────────────

    #[test]
    fn test_simple_hash_deterministic() {
        let h1 = simple_hash("https://example.com/image.png");
        let h2 = simple_hash("https://example.com/image.png");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_simple_hash_different_inputs() {
        let h1 = simple_hash("abc");
        let h2 = simple_hash("def");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_simple_hash_empty() {
        let h = simple_hash("");
        assert_eq!(h, 5381); // initial value with no bytes processed
    }

    // ── gradient_to_primitive ───────────────────────────────────────────

    #[test]
    fn test_linear_gradient_to_primitive() {
        let grad = GradientValue::Linear(LinearGradient {
            interpolation: Default::default(),
            direction: GradientDirection::ToBottom,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            }],
            repeating: false,
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255))
            .expect("linear gradient should convert");
        assert_eq!(prim.rect, rect);
        assert!(matches!(prim.kind, GradientKind::Linear { .. }));
        assert_eq!(prim.stops.len(), 1);
    }

    /// R2370：gradient color-stop 的 currentColor 解析为**使用该渐变的元素**自身 color。
    /// 旧 convert_color_stops 用 color_value_to_render 无元素上下文 → currentColor 回落黑色。
    /// driving: css-images color-stop-currentcolor。
    #[test]
    fn test_gradient_to_primitive_currentcolor_resolves_to_element_color() {
        let grad = GradientValue::Linear(LinearGradient {
            interpolation: Default::default(),
            direction: GradientDirection::ToBottom,
            stops: vec![GradientColorStop {
                color: ColorValue::CurrentColor,
                position: None,
            }],
            repeating: false,
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let element_color = ColorValue::Rgba(255, 0, 0, 255); // red
        let prim = gradient_to_primitive(&grad, &rect, &element_color).expect("gradient should convert");
        assert_eq!(
            prim.stops[0].color,
            zero_render_foundation::color::Color::rgb(255, 0, 0),
            "gradient currentcolor stop 应解析为元素 color（red），非黑色"
        );
    }

    #[test]
    fn test_radial_gradient_to_primitive() {
        let grad = GradientValue::Radial(RadialGradient {
            interpolation: Default::default(),
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            size: RadialSize::FarthestCorner,
            repeating: false,
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 255, 255, 255),
                    position: None,
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 0, 255),
                    position: None,
                },
            ],
        });
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255))
            .expect("radial gradient should convert");
        assert!(matches!(prim.kind, GradientKind::Radial { .. }));
    }

    #[test]
    fn test_conic_gradient_to_primitive() {
        let grad = GradientValue::Conic(ConicGradient {
            interpolation: Default::default(),
            repeating: false,
            from_angle: 90.0,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            stops: vec![
                GradientColorStop {
                    color: ColorValue::Rgba(255, 0, 0, 255),
                    position: None,
                },
                GradientColorStop {
                    color: ColorValue::Rgba(0, 0, 255, 255),
                    position: None,
                },
            ],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255))
            .expect("conic gradient should convert");
        assert!(matches!(prim.kind, GradientKind::Conic { .. }));
        assert_eq!(prim.stops.len(), 2);
        if let GradientKind::Conic {
            cx, cy, start_angle, ..
        } = prim.kind
        {
            assert!((cx - 100.0).abs() < 0.1);
            assert!((cy - 50.0).abs() < 0.1);
            assert!((start_angle - 90.0_f64.to_radians() as f32).abs() < 0.01);
        }
    }

    #[test]
    fn test_radial_closest_side() {
        let grad = GradientValue::Radial(RadialGradient {
            interpolation: Default::default(),
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            size: RadialSize::ClosestSide,
            repeating: false,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            }],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255)).unwrap();
        if let GradientKind::Radial {
            cx, cy, outer_radius, ..
        } = prim.kind
        {
            // 50% of 200 = 100, 50% of 100 = 50
            assert!((cx - 100.0).abs() < 0.1);
            assert!((cy - 50.0).abs() < 0.1);
            assert!((outer_radius - 50.0).abs() < 0.1);
        } else {
            panic!("expected radial gradient");
        }
    }

    #[test]
    fn test_radial_farthest_side() {
        let grad = GradientValue::Radial(RadialGradient {
            interpolation: Default::default(),
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            size: RadialSize::FarthestSide,
            repeating: false,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            }],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255)).unwrap();
        if let GradientKind::Radial { outer_radius, .. } = prim.kind {
            assert!((outer_radius - 100.0).abs() < 0.1);
        } else {
            panic!("expected radial gradient");
        }
    }

    #[test]
    fn test_radial_length_size() {
        let grad = GradientValue::Radial(RadialGradient {
            interpolation: Default::default(),
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            size: RadialSize::Length(LengthValue::Px(75.0)),
            repeating: false,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            }],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect, &ColorValue::Rgba(0, 0, 0, 255)).unwrap();
        if let GradientKind::Radial { outer_radius, .. } = prim.kind {
            assert!((outer_radius - 75.0).abs() < 0.1);
        } else {
            panic!("expected radial gradient");
        }
    }

    // ── linear_direction_to_kind ────────────────────────────────────────

    #[test]
    fn test_linear_direction_to_bottom() {
        let rect = Rect::new(0.0, 0.0, 100.0, 200.0);
        let kind = linear_direction_to_kind(&GradientDirection::ToBottom, &rect);
        if let GradientKind::Linear { x0, y0, x1: _, y1: _ } = kind {
            assert!((x0 - 50.0).abs() < 0.01);
            assert!((y0 - 0.0).abs() < 0.01);
        } else {
            panic!("expected linear gradient");
        }
    }

    #[test]
    fn test_linear_direction_to_top() {
        let rect = Rect::new(0.0, 0.0, 100.0, 200.0);
        let kind = linear_direction_to_kind(&GradientDirection::ToTop, &rect);
        if let GradientKind::Linear { x0, y0, x1: _, y1 } = kind {
            assert!((x0 - 50.0).abs() < 0.01);
            assert!((y0 - 200.0).abs() < 0.01);
            assert!((y1 - 0.0).abs() < 0.01);
        } else {
            panic!("expected linear gradient");
        }
    }

    #[test]
    fn test_linear_direction_to_right() {
        let rect = Rect::new(0.0, 0.0, 100.0, 200.0);
        let kind = linear_direction_to_kind(&GradientDirection::ToRight, &rect);
        if let GradientKind::Linear { x0, y0: _, x1, y1: _ } = kind {
            assert!((x0 - 0.0).abs() < 0.01);
            assert!((x1 - 100.0).abs() < 0.01);
        } else {
            panic!("expected linear gradient");
        }
    }

    #[test]
    fn test_linear_direction_to_left() {
        let rect = Rect::new(0.0, 0.0, 100.0, 200.0);
        let kind = linear_direction_to_kind(&GradientDirection::ToLeft, &rect);
        if let GradientKind::Linear { x0, y0: _, x1, y1: _ } = kind {
            assert!((x0 - 100.0).abs() < 0.01);
            assert!((x1 - 0.0).abs() < 0.01);
        } else {
            panic!("expected linear gradient");
        }
    }

    #[test]
    fn test_linear_direction_angle() {
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        // 90deg = to right
        let kind = linear_direction_to_kind(&GradientDirection::Angle(90.0), &rect);
        if let GradientKind::Linear { x0, x1, .. } = kind {
            assert!(x0 < x1); // moves rightward
        } else {
            panic!("expected linear gradient");
        }
    }

    // ── convert_color_stops ─────────────────────────────────────────────

    #[test]
    fn test_convert_stops_with_positions() {
        let stops = vec![
            GradientColorStop {
                color: ColorValue::Rgba(255, 0, 0, 255),
                position: Some(LengthValue::Percentage(0.0)),
            },
            GradientColorStop {
                color: ColorValue::Rgba(0, 0, 255, 255),
                position: Some(LengthValue::Percentage(100.0)),
            },
        ];
        let result = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(result.len(), 2);
        assert!((result[0].offset - 0.0).abs() < 0.01);
        assert!((result[1].offset - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_stops_auto_distribute() {
        let stops = vec![
            GradientColorStop {
                color: ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            },
            GradientColorStop {
                color: ColorValue::Rgba(0, 255, 0, 255),
                position: None,
            },
            GradientColorStop {
                color: ColorValue::Rgba(0, 0, 255, 255),
                position: None,
            },
        ];
        let result = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));
        assert!((result[0].offset - 0.0).abs() < 0.01);
        assert!((result[1].offset - 0.5).abs() < 0.01);
        assert!((result[2].offset - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_convert_stops_single_stop_offset_zero() {
        let stops = vec![GradientColorStop {
            color: ColorValue::Rgba(128, 128, 128, 255),
            position: None,
        }];
        let result = convert_color_stops(&stops, &ColorValue::Rgba(0, 0, 0, 255));
        assert_eq!(result.len(), 1);
        assert!((result[0].offset - 0.0).abs() < 0.01);
    }

    #[test]
    fn resolve_document_url_root_relative_path() {
        assert_eq!(
            resolve_document_url("https://example.com/page", "/aaa"),
            "https://example.com/aaa"
        );
    }

    #[test]
    fn resolve_document_url_relative_path() {
        assert_eq!(
            resolve_document_url("https://example.com/dir/page", "other.html"),
            "https://example.com/dir/other.html"
        );
    }

    #[test]
    fn resolve_document_url_preserves_special_schemes() {
        assert_eq!(
            resolve_document_url("https://example.com", "mailto:test@example.com"),
            "mailto:test@example.com"
        );
        assert_eq!(
            resolve_document_url("https://example.com", "javascript:void(0)"),
            "javascript:void(0)"
        );
    }
}
