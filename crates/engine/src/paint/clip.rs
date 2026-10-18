//! 绘制辅助 — 裁剪/opacity 家族（自 helpers.rs 按职责拆分，T59；纯移动零行为变化）。

use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::RenderPrimitives;

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

/// R4248/R4541：非轴对齐多边形裁剪统一入口（clip-path polygon 臂与 border-shape
/// overflow 裁剪共用）——被多边形完全覆盖的填充/圆角矩形改写为多边形 path_fill
/// （顶点均落在图元 rect 内 ≡ polygon ⊆ rect，改写不越界；R4539 起 op 原位替换保
/// z 序），其余图元走 v2 条带裁剪。矩形多边形跳过改写：矩形交集裁剪即精确（既有
/// 语义，测试依赖）。
pub fn clip_with_polygon_rewrite(primitives: &mut RenderPrimitives, from: &PrimitiveCounts, polygon: &[(f32, f32)]) {
    use zero_render_foundation::primitive::DrawOp;

    if polygon.len() < 3 {
        return;
    }
    let is_axis_aligned_rect = polygon.len() == 4 && {
        let xs: Vec<f32> = polygon.iter().map(|p| p.0).collect();
        let ys: Vec<f32> = polygon.iter().map(|p| p.1).collect();
        xs.iter().filter(|&&v| (v - xs[0]).abs() < 0.01).count() == 2
            && xs.iter().filter(|&&v| (v - xs[2]).abs() < 0.01).count() == 2
            && ys.iter().filter(|&&v| (v - ys[0]).abs() < 0.01).count() == 2
            && ys.iter().filter(|&&v| (v - ys[2]).abs() < 0.01).count() == 2
    };
    if is_axis_aligned_rect {
        clip_all_primitives_to_polygon(primitives, from, polygon);
        return;
    }
    // 被多边形完全覆盖的填充/圆角矩形改写为多边形 path_fill。kill-switch
    // `ZW_CLIP_REWRITE_INPLACE=0` 回退尾部追加旧行为（z 序倒挂，R4539 修复前形态）。
    let clip_rewrite_inplace = std::env::var("ZW_CLIP_REWRITE_INPLACE").as_deref() != Ok("0");
    let covered = |r: &Rect| {
        polygon
            .iter()
            .all(|&(x, y)| x >= r.left() - 0.5 && x <= r.right() + 0.5 && y >= r.top() - 0.5 && y <= r.bottom() + 0.5)
    };
    let mut rewritten: Vec<(usize, zero_render_foundation::color::Color)> = Vec::new();
    for i in from.fills..primitives.fills.len() {
        let rect = primitives.fills[i].rect;
        if covered(&rect) {
            rewritten.push((i, primitives.fills[i].color));
            primitives.fills[i].rect = Rect::new(0.0, 0.0, 0.0, 0.0);
        }
    }
    let mut rewritten_rr: Vec<(usize, zero_render_foundation::color::Color)> = Vec::new();
    for i in from.rounded_rects..primitives.rounded_rects.len() {
        let rect = primitives.rounded_rects[i].rect;
        if covered(&rect) {
            rewritten_rr.push((i, primitives.rounded_rects[i].color));
            primitives.rounded_rects[i].rect = Rect::new(0.0, 0.0, 0.0, 0.0);
        }
    }
    let verts: Vec<f32> = polygon.iter().flat_map(|&(x, y)| [x, y]).collect();
    for (i, color) in rewritten {
        primitives.add_path_fill(verts.clone(), color);
        if clip_rewrite_inplace
            && let Some(pos) = primitives
                .draw_order
                .iter()
                .position(|op| matches!(op, DrawOp::Fill(j) if *j == i))
            && let Some(tail) = primitives.draw_order.pop()
        {
            primitives.draw_order[pos] = tail;
        }
    }
    for (i, color) in rewritten_rr {
        primitives.add_path_fill(verts.clone(), color);
        if clip_rewrite_inplace
            && let Some(pos) = primitives
                .draw_order
                .iter()
                .position(|op| matches!(op, DrawOp::RoundedRect(j) if *j == i))
            && let Some(tail) = primitives.draw_order.pop()
        {
            primitives.draw_order[pos] = tail;
        }
    }
    clip_all_primitives_to_polygon(primitives, from, polygon);
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

    // 第二步：对 fills 进行精确多边形裁剪。
    // R4540 v2：旧法 drain/重灌 fills 向量使 from.fills 之后的全部 Fill op 索引失效
    // （draw_order 重放时条带被丢弃不绘制，R4537 记档的「条带无 op」根因），且索引
    // 位移令后续嵌套 clip 消费的快照失准（R4536 缺陷 C）。v2 改为原地条带化：原 fill
    // 清零、条带以 path_fill 发射并把 op 登记/插入到原 Fill op 槽位——op 插入不携带
    // 索引位移、fills 向量长度不变，任何时点的 PrimitiveCounts 快照与 op 索引永久
    // 有效。kill-switch `ZW_CLIP_V2=0` 回退旧 drain/重灌行为。
    let clip_v2 = std::env::var("ZW_CLIP_V2").as_deref() != Ok("0");
    if clip_v2 {
        clip_fills_to_polygon_inplace(primitives, from.fills, polygon, true);
        // R4542（css-borders-4 §border-shape）：replaced 元素（img 等）内容同样裁剪到
        // 多边形内缘——条带以 clip 窗口承载（crop 语义，source 仍映射完整 rect 不重采样）。
        clip_images_to_polygon_inplace(primitives, from.images, polygon, true);
        // R5184：渐变层同样条带裁剪（clip 窗口保持渐变相位，R3909 crop 语义）——
        // border-shape 背景渐变（collapsed-shape-clips-background）与 corner-shape/
        // clip-path 窗口的渐变层此前溢出多边形。
        clip_gradients_to_polygon_inplace(primitives, from.gradients, polygon, true);
    } else {
        // 将每个 fill 矩形与多边形求交，生成多个子矩形
        let mut new_fills = Vec::new();
        let fills_to_clip: Vec<_> = primitives.fills.drain(from.fills..).collect();
        for fill in fills_to_clip {
            let clipped = clip_fill_to_polygon(&fill, polygon, false);
            new_fills.extend(clipped);
        }
        primitives.fills.extend(new_fills);
    }

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
///
/// R4540（R4536 缺陷 B 修正）：`midpoint=true` 时在条带**中点**采样（条带顶采样在
/// 多边形顶点恰落在采样线上时按半开边规则漏计自该顶点起始的边——整数坐标测试页
/// 常态——整带丢失）。中点与整数顶点天然错开，交点计数恒成对。
fn clip_fill_to_polygon(
    fill: &zero_render_foundation::primitive::FillPrimitive,
    polygon: &[(f32, f32)],
    midpoint: bool,
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
        let y_sample = if midpoint { (y + y_end) * 0.5 } else { y };
        // 找到该行与多边形的所有交点
        let mut intersections = Vec::new();
        let n = polygon.len();
        for i in 0..n {
            let (x1, y1) = polygon[i];
            let (x2, y2) = polygon[(i + 1) % n];
            if (y1 < y_sample && y2 >= y_sample) || (y2 < y_sample && y1 >= y_sample) {
                let t = (y_sample - y1) / (y2 - y1);
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

/// R4540 v2：fills 多边形条带裁剪（原地条带化 + op 登记）。
///
/// 对 `[start..len)` 内每个 fill：
/// - 0 条带（fill 在多边形外）→ fill 原地清零（`DrawOp::Fill(i)` 重放零矩形为 no-op）；
/// - 条带与原 rect 完全一致（fill 含于多边形内部）→ 原样保留；
/// - 其余（部分相交）→ fill 原地清零，逐条带 `add_path_fill` 发射，并把原 `Fill(i)`
///   op 槽位替换为首个条带的 `PathFill` op、其余条带 op 依次插入其后——同一 fill 的
///   条带互不重叠（内部序无关），相对其他图元的 z 序 = 原 op 槽位（保序）。
///
/// 不变性：`draw_order` 插入/替换 op 只改顺序表本身，任何 op 的索引载荷无需更新；
/// fills 向量长度不变 → 既有 `PrimitiveCounts` 快照与全部 op 索引永久有效（嵌套
/// clip 消费可安全叠加，R4536 缺陷 C 根除）。找不到原 op 槽位（防御）时只清零不发射。
fn clip_fills_to_polygon_inplace(
    primitives: &mut RenderPrimitives,
    start: usize,
    polygon: &[(f32, f32)],
    midpoint: bool,
) {
    use zero_render_foundation::primitive::DrawOp;

    for i in start..primitives.fills.len() {
        let fill = primitives.fills[i].clone();
        let strips = clip_fill_to_polygon(&fill, polygon, midpoint);
        match strips.len() {
            0 => primitives.fills[i].rect = Rect::new(0.0, 0.0, 0.0, 0.0),
            1 if strips[0].rect == fill.rect => {}
            _ => {
                primitives.fills[i].rect = Rect::new(0.0, 0.0, 0.0, 0.0);
                let Some(op_pos) = primitives
                    .draw_order
                    .iter()
                    .position(|op| matches!(op, DrawOp::Fill(j) if *j == i))
                else {
                    continue;
                };
                for (k, strip) in strips.iter().enumerate() {
                    let r = &strip.rect;
                    let verts = vec![
                        r.left(),
                        r.top(),
                        r.right(),
                        r.top(),
                        r.right(),
                        r.bottom(),
                        r.left(),
                        r.bottom(),
                    ];
                    primitives.add_path_fill(verts, strip.color);
                    // add_path_fill 必在尾部追加一个 PathFill op；移到目标槽位。
                    let tail = primitives.draw_order.pop();
                    let Some(tail) = tail else { break };
                    if k == 0 {
                        primitives.draw_order[op_pos] = tail;
                    } else {
                        primitives.draw_order.insert(op_pos + k, tail);
                    }
                }
            }
        }
    }
}

/// R4542：images 多边形条带裁剪（原地 + op 登记）——与 fills v2 同一不变式。
///
/// 对 `[start..len)` 内每个 image：以 rect 为几何做扫描线条带，
/// - 0 条带（多边形外）→ rect 清零（`DrawOp::Image(i)` 重放为无绘制）；
/// - 1 条带 = 原 rect（含于多边形）→ 原样保留；
/// - 其余（部分相交）→ 原 image rect 清零，逐条带追加新 ImagePrimitive
///   （rect = 原始 rect（source 映射保持不重采样）+ `clip = Some(条带 rect)` crop
///   窗口），并把原 `Image(i)` op 槽位替换/插入为条带 op（z 序 = 原 op 槽位保序）。
///
/// images 向量只增不减，op 插入零索引位移（R4540 v2 不变式）。
/// `source: Some`（border-image 9-slice）的图元跳过（条带会破坏 slice 映射），
/// 仅保留包围盒裁剪旧行为。
fn clip_images_to_polygon_inplace(
    primitives: &mut RenderPrimitives,
    start: usize,
    polygon: &[(f32, f32)],
    midpoint: bool,
) {
    use zero_render_foundation::primitive::{DrawOp, FillPrimitive};

    for i in start..primitives.images.len() {
        if primitives.images[i].source.is_some() {
            continue;
        }
        let img = &primitives.images[i];
        let probe = FillPrimitive {
            rect: img.rect,
            color: zero_render_foundation::color::Color::BLACK,
        };
        let strips = clip_fill_to_polygon(&probe, polygon, midpoint);
        match strips.len() {
            0 => primitives.images[i].rect = Rect::new(0.0, 0.0, 0.0, 0.0),
            1 if strips[0].rect == img.rect => {}
            _ => {
                let img = &primitives.images[i];
                let rect = img.rect;
                let key = img.image_key.clone();
                primitives.images[i].rect = Rect::new(0.0, 0.0, 0.0, 0.0);
                let Some(op_pos) = primitives
                    .draw_order
                    .iter()
                    .position(|op| matches!(op, DrawOp::Image(j) if *j == i))
                else {
                    continue;
                };
                for (k, strip) in strips.iter().enumerate() {
                    let idx = primitives.images.len();
                    primitives
                        .images
                        .push(zero_render_foundation::primitive::ImagePrimitive {
                            rect,
                            image_key: key.clone(),
                            clip: Some(strip.rect),
                            source: None,
                            clip_radius: None,
                        });
                    primitives.draw_order.push(DrawOp::Image(idx));
                    let tail = primitives.draw_order.pop();
                    let Some(tail) = tail else { break };
                    if k == 0 {
                        primitives.draw_order[op_pos] = tail;
                    } else {
                        primitives.draw_order.insert(op_pos + k, tail);
                    }
                }
            }
        }
    }
}

/// R5184：gradients 多边形条带裁剪（原地 + op 登记）——与 images v2 同一不变式。
///
/// 对 `[start..len)` 内每个 gradient：以 rect 为几何做扫描线条带，
/// - 0 条带（多边形外/退化形状如 inset(100px)、circle(0%)）→ rect 清零
///   （`DrawOp::Gradient(i)` 重放为无绘制）；
/// - 1 条带 = 原 rect（含于多边形）→ 原样保留；
/// - 其余（部分相交）→ 原 rect 清零，逐条带追加新 GradientPrimitive（rect 保持
///   完整 rect（渐变相位不重映射）+ `clip = Some(条带)` crop 窗口，R3909 语义），
///   并把原 `Gradient(i)` op 槽位替换/插入为条带 op（z 序 = 原 op 槽位保序）。
///
/// gradients 向量只增不减，op 插入零索引位移（R4540 v2 不变式）。
fn clip_gradients_to_polygon_inplace(
    primitives: &mut RenderPrimitives,
    start: usize,
    polygon: &[(f32, f32)],
    midpoint: bool,
) {
    use zero_render_foundation::primitive::{DrawOp, FillPrimitive, GradientPrimitive};

    for i in start..primitives.gradients.len() {
        let probe = FillPrimitive {
            rect: primitives.gradients[i].rect,
            color: zero_render_foundation::color::Color::BLACK,
        };
        let strips = clip_fill_to_polygon(&probe, polygon, midpoint);
        match strips.len() {
            0 => primitives.gradients[i].rect = Rect::new(0.0, 0.0, 0.0, 0.0),
            1 if strips[0].rect == primitives.gradients[i].rect => {}
            _ => {
                let g = &primitives.gradients[i];
                let rect = g.rect;
                let kind = g.kind.clone();
                let stops = g.stops.clone();
                let repeating = g.repeating;
                let interpolation = g.interpolation;
                primitives.gradients[i].rect = Rect::new(0.0, 0.0, 0.0, 0.0);
                let Some(op_pos) = primitives
                    .draw_order
                    .iter()
                    .position(|op| matches!(op, DrawOp::Gradient(j) if *j == i))
                else {
                    continue;
                };
                for (k, strip) in strips.iter().enumerate() {
                    let idx = primitives.gradients.len();
                    primitives.gradients.push(GradientPrimitive {
                        rect,
                        kind: kind.clone(),
                        stops: stops.clone(),
                        repeating,
                        interpolation,
                        clip: Some(strip.rect),
                    });
                    primitives.draw_order.push(DrawOp::Gradient(idx));
                    let tail = primitives.draw_order.pop();
                    let Some(tail) = tail else { break };
                    if k == 0 {
                        primitives.draw_order[op_pos] = tail;
                    } else {
                        primitives.draw_order.insert(op_pos + k, tail);
                    }
                }
            }
        }
    }
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
    // 裁剪阴影。R4059：blur（3σ 外扩）会越过 `rect` 渲染，仅 shrink rect 挡不住 blur
    // 溢出——把可见窗口写 `shadow.clip`（语义同 img.clip crop，R294），renderer 端按
    // 窗口硬裁 blur 输出（cpu 逐像素 / gpu scissor）。rect 自身保持不变：rect 是阴影
    // 形状的参考框（spread/blur 从它外扩），shrink 会把剩余 blur 一并错位收缩。
    for shadow in primitives.shadows.iter_mut().skip(from.shadows) {
        // 参考框与裁剪区完全不相交 → 零尺寸 rect（renderer 见空面积早退）；否则保持
        // rect 原样，窗口以 clip_rect 为准（blur 可以在 rect 与 clip_rect 之间的缝隙区
        // 可见——如 offset 阴影）。
        let r = shadow.rect;
        let visible = r.right() > clip_rect.left()
            && r.left() < clip_rect.right()
            && r.bottom() > clip_rect.top()
            && r.top() < clip_rect.bottom();
        if !visible {
            let r = &mut shadow.rect;
            r.size.width = 0.0;
            r.size.height = 0.0;
        }
        shadow.clip = Some(*clip_rect);
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

/// R5155（css-contain-1 §3.1 paint containment 圆角裁剪）：对快照之后新增的
/// image 图元施加「矩形窗口 + 圆角」裁剪——窗口照旧矩形交集（crop 语义），
/// 另设 `clip_radius` 供渲染端逐像素圆角剔除。仅接图片内容（contain:paint 圆角
/// 裁剪驱动案 contain-paint-clip-015/016 的越界主体）；fill/rounded_rect 背景
/// 自身已按圆角绘制、glyph 文本圆角剔除无案面（FIXME 案驱动再接）。
/// `radii` = 四角半径 [tl, tr, br, bl]（CSS px，相对 clip_rect；border 宽已扣减）。
pub fn clip_images_to_rounded_rect(
    primitives: &mut RenderPrimitives,
    from: &PrimitiveCounts,
    clip_rect: &Rect,
    radii: [f32; 4],
) {
    for img in primitives.images.iter_mut().skip(from.images) {
        let rect = img.rect;
        let cur = img.clip.unwrap_or(rect);
        let left = cur.left().max(clip_rect.left());
        let top = cur.top().max(clip_rect.top());
        let right = cur.right().min(clip_rect.right());
        let bottom = cur.bottom().min(clip_rect.bottom());
        if right <= left || bottom <= top {
            img.clip = Some(Rect::new(left, top, 0.0, 0.0));
            continue;
        }
        img.clip = Some(Rect::new(left, top, right - left, bottom - top));
        img.clip_radius = Some(radii);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paint::border_shape::border_shape_overflow_polygon;
    use zero_css_parser::values::{BorderShapeValue, ClipPathValue, LengthValue, TransformValue};
    use zero_layout_engine::LayoutBox;
    use zero_render_foundation::color::Color;
    use zero_render_foundation::geometry::Rect;
    use zero_render_foundation::primitive::{DrawOp, FillPrimitive, FontId, GlyphPrimitive, RenderPrimitives};
    use zero_style_system::ComputedStyle;

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

    // ── R4540：v2 多边形条带裁剪（中点采样 + 原地 op 登记）────────────────

    /// R4536 缺陷 B 修正：多边形顶点恰落在条带采样线上时，顶采样按半开边规则漏计
    /// 自该顶点起始的边 → 整带丢失；中点采样与整数顶点天然错开。
    #[test]
    fn test_clip_fill_to_polygon_midpoint_sampling_keeps_vertex_row() {
        let fill = FillPrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            color: Color::BLACK,
        };
        // 轴对齐矩形多边形 [0,8]×[2,6]：四顶点全落在整数采样线上。
        let polygon = vec![(0.0_f32, 2.0_f32), (8.0, 2.0), (8.0, 6.0), (0.0, 6.0)];
        let area = |strips: &[FillPrimitive]| {
            strips
                .iter()
                .map(|s| s.rect.size.width * s.rect.size.height)
                .sum::<f32>()
        };
        let top = clip_fill_to_polygon(&fill, &polygon, false);
        let mid = clip_fill_to_polygon(&fill, &polygon, true);
        // 顶采样缺陷形态：首行 [2,3) 采样线 y=2 与顶边顶点重合被漏计（条带整体下移一行：
        // 起点 3.0），而底行 [6,7) 又被 y2>=y 半开规则多算——形状在顶点行扭曲。
        assert!(
            top.first().map(|s| s.rect.origin.y).unwrap_or(0.0) >= 3.0,
            "顶采样基线：首条带应被推到 y=3（漏计顶点行），实际 {:?}",
            top.first().map(|s| s.rect.origin.y)
        );
        // 中点采样：条带精确平铺 [2,6)，面积满覆盖 32。
        assert!(
            (area(&mid) - 32.0).abs() < 0.5,
            "中点采样应满覆盖 32，实际 {}",
            area(&mid)
        );
        assert_eq!(mid.first().map(|s| s.rect.origin.y), Some(2.0), "首条带应从 y=2 起");
    }

    /// R4540 v2：部分相交 fill 条带化后 op 原位登记——Fill op 被替换为条带 PathFill op，
    /// fills 向量长度不变（零索引位移）。
    #[test]
    fn test_clip_fills_inplace_registers_strip_ops_without_index_shift() {
        let mut p = RenderPrimitives::new();
        // fill[0] 在多边形外（不动）、fill[1] 与多边形部分相交（条带化）、fill[2] 无关（不动）。
        let from = PrimitiveCounts::snapshot(&p);
        p.add_fill(Rect::new(0.0, 0.0, 2.0, 2.0), Color::BLACK);
        p.add_fill(Rect::new(0.0, 0.0, 8.0, 8.0), Color::rgba(0, 128, 0, 255));
        p.add_fill(Rect::new(20.0, 20.0, 2.0, 2.0), Color::BLACK);
        // 矩形多边形 [0,8]×[2,6]：与 fill[1] 部分相交。
        let polygon = vec![(0.0_f32, 2.0_f32), (8.0, 2.0), (8.0, 6.0), (0.0, 6.0)];
        clip_fills_to_polygon_inplace(&mut p, from.fills, &polygon, true);
        assert_eq!(p.fills.len(), 3);

        // fills 向量长度不变（v2 核心不变式：零索引位移）。
        assert_eq!(p.fills.len(), 3, "v2 不得增删 fills（旧 drain/重灌会位移索引）");
        // fill[1] 清零（原 op 重放为 no-op），条带以 path_fill 承载。
        assert_eq!(p.fills[1].rect.size.width, 0.0, "部分相交 fill 应清零");
        assert!(
            p.path_fills.len() >= 4,
            "条带应以 path_fill 发射，实际 {}",
            p.path_fills.len()
        );
        // op 登记：draw_order 中原 Fill(1) 槽位被首个条带 PathFill op 替换，其余条带 op
        // 紧随其后插入。
        let fill1_pos = p.draw_order.iter().position(|op| matches!(op, DrawOp::Fill(1)));
        assert!(fill1_pos.is_none(), "Fill(1) op 应被替换，不得残留");
        let strip_pos = p.draw_order.iter().position(|op| matches!(op, DrawOp::PathFill(_)));
        assert!(strip_pos.is_some(), "条带 PathFill op 应在原 op 槽位附近");
        // 条带 PathFill op 全部落在 Fill(1) 原 op 位（=1）与 Fill(2) op 位之前（z 序保序）。
        let fill2_pos = p
            .draw_order
            .iter()
            .position(|op| matches!(op, DrawOp::Fill(2)))
            .expect("fill[2] 未被裁剪，op 应保留");
        let strip_count = p
            .draw_order
            .iter()
            .filter(|op| matches!(op, DrawOp::PathFill(_)))
            .count();
        assert!(strip_count >= 4);
        let first_strip = p
            .draw_order
            .iter()
            .position(|op| matches!(op, DrawOp::PathFill(_)))
            .unwrap();
        assert!(
            first_strip > 0 && first_strip < fill2_pos,
            "条带 op 应位于 fill[1] 原槽位（fill[0] 之后、fill[2] 之前）：first={first_strip} fill2={fill2_pos}"
        );
        // draw_order 全部 op 索引仍有效：Fill op 指向的 fill 存在。
        for op in &p.draw_order {
            if let DrawOp::Fill(j) = op {
                assert!(*j < p.fills.len(), "Fill op 索引 {} 越界", j);
            }
        }
    }

    /// R4542：images 多边形条带裁剪——clip 窗口条带 + op 槽位登记（images 向量只增不减）。
    #[test]
    fn test_clip_images_inplace_registers_strip_ops_without_index_shift() {
        use zero_render_foundation::image_cache::ImageKey;
        use zero_render_foundation::primitive::{DrawOp, ImagePrimitive};
        let mut p = RenderPrimitives::new();
        p.add_image(ImagePrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            image_key: ImageKey(1),
            clip: None,
            source: None,
            clip_radius: None,
        });
        // 矩形多边形 [0,8]×[2,6]：与 image 部分相交。
        let polygon = vec![(0.0_f32, 2.0_f32), (8.0, 2.0), (8.0, 6.0), (0.0, 6.0)];
        clip_images_to_polygon_inplace(&mut p, 0, &polygon, true);
        // 原 image 清零；条带以新 image（clip 窗口）追加在尾部（向量只增）。
        assert_eq!(p.images[0].rect.size.width, 0.0, "部分相交 image 应清零");
        assert!(p.images.len() >= 5, "条带 image 应追加：{}", p.images.len());
        assert_eq!(p.images[1].rect, Rect::new(0.0, 0.0, 8.0, 8.0), "条带 rect 保持源映射");
        assert!(p.images[1].clip.is_some(), "条带应以 clip 窗口承载");
        // op 登记：原 Image(0) op 被替换为条带 op；全部 Image op 索引有效。
        assert!(
            !p.draw_order.iter().any(|op| matches!(op, DrawOp::Image(0))),
            "Image(0) op 应被条带 op 替换"
        );
        for op in &p.draw_order {
            if let DrawOp::Image(j) = op {
                assert!(*j < p.images.len(), "Image op 索引 {} 越界", j);
            }
        }
    }

    /// R5184：gradients 多边形条带裁剪——clip 窗口条带 + op 槽位登记（gradients
    /// 向量只增不减，零索引位移）。
    #[test]
    fn test_clip_gradients_inplace_registers_strip_ops_without_index_shift() {
        use zero_render_foundation::primitive::{DrawOp, GradientKind, GradientPrimitive, GradientStop};
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 8.0,
                y1: 0.0,
            },
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::BLACK,
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::WHITE,
                },
            ],
            repeating: false,
            interpolation: Default::default(),
            clip: None,
        });
        // 矩形多边形 [0,8]×[2,6]：与 gradient 部分相交。
        let polygon = vec![(0.0_f32, 2.0_f32), (8.0, 2.0), (8.0, 6.0), (0.0, 6.0)];
        clip_gradients_to_polygon_inplace(&mut p, 0, &polygon, true);
        // 原 gradient 清零；条带以新 gradient（clip 窗口）追加在尾部（向量只增）。
        assert_eq!(p.gradients[0].rect.size.width, 0.0, "部分相交 gradient 应清零");
        assert!(p.gradients.len() >= 5, "条带 gradient 应追加：{}", p.gradients.len());
        assert_eq!(
            p.gradients[1].rect,
            Rect::new(0.0, 0.0, 8.0, 8.0),
            "条带 rect 保持完整 rect（相位不重映射）"
        );
        assert!(p.gradients[1].clip.is_some(), "条带应以 clip 窗口承载");
        // op 登记：原 Gradient(0) op 被替换为条带 op；全部 Gradient op 索引有效。
        assert!(
            !p.draw_order.iter().any(|op| matches!(op, DrawOp::Gradient(0))),
            "Gradient(0) op 应被条带 op 替换"
        );
        for op in &p.draw_order {
            if let DrawOp::Gradient(j) = op {
                assert!(*j < p.gradients.len(), "Gradient op 索引 {} 越界", j);
            }
        }
    }

    /// R5184：退化多边形（collapsed shape，circle(0%) 的 64 重合顶点）→ 0 条带 →
    /// rect 清零（背景层整体裁没的判例语义）。
    #[test]
    fn test_clip_gradients_inplace_degenerate_polygon_zeroes() {
        use zero_render_foundation::primitive::{GradientKind, GradientPrimitive, GradientStop};
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 8.0,
                y1: 0.0,
            },
            stops: vec![GradientStop {
                offset: 0.0,
                color: Color::BLACK,
            }],
            repeating: false,
            interpolation: Default::default(),
            clip: None,
        });
        let degenerate: Vec<(f32, f32)> = vec![(3.0, 3.0); 64];
        clip_gradients_to_polygon_inplace(&mut p, 0, &degenerate, true);
        assert_eq!(p.gradients[0].rect.size.width, 0.0, "退化多边形应清零 gradient");
    }

    /// R4541：border-shape overflow 裁剪多边形——stroke mode 半宽内缩与 fill mode 内形状。
    #[test]
    fn test_border_shape_overflow_polygon_inner_edge() {
        use zero_css_parser::values::{BorderShapeGeometryBox, ClipPathRadius};
        let mut style = ComputedStyle::default();
        // stroke mode：diamond polygon(50% 0, 100% 50%, 50% 100%, 0 50%)。
        style.border_shape = BorderShapeValue::Stroke {
            shape: ClipPathValue::Polygon {
                fill_rule: zero_css_parser::values::PolygonFillRule::NonZero,
                points: vec![
                    (LengthValue::Percentage(50.0), LengthValue::Percentage(0.0)),
                    (LengthValue::Percentage(100.0), LengthValue::Percentage(50.0)),
                    (LengthValue::Percentage(50.0), LengthValue::Percentage(100.0)),
                    (LengthValue::Percentage(0.0), LengthValue::Percentage(50.0)),
                ],
            },
            geometry_box: BorderShapeGeometryBox::HalfBorderBox,
        };
        // 100×100 盒 + 10px 边框：half-border box = [5,95]²，内缩半宽 5 → [10,90]²。
        let box_node = LayoutBox {
            width: 100.0,
            height: 100.0,
            border_top: 10.0,
            border_left: 10.0,
            border_bottom: 10.0,
            border_right: 10.0,
            ..Default::default()
        };
        let poly = border_shape_overflow_polygon(&style, &box_node, 0.0, 0.0).expect("stroke mode 应产出裁剪多边形");
        assert_eq!(poly.len(), 4);
        let xs: Vec<f32> = poly.iter().map(|p| p.0).collect();
        let ys: Vec<f32> = poly.iter().map(|p| p.1).collect();
        // 内缘 diamond 顶点：(50,10) (90,50) (50,90) (10,50)。
        assert!(
            (xs.iter().cloned().fold(f32::MIN, f32::max) - 90.0).abs() < 0.01,
            "x 最大应内缩到 90：{xs:?}"
        );
        assert!(
            (ys.iter().cloned().fold(f32::MIN, f32::max) - 90.0).abs() < 0.01,
            "y 最大应内缩到 90：{ys:?}"
        );
        assert!(
            (xs.iter().cloned().fold(f32::MAX, f32::min) - 10.0).abs() < 0.01,
            "x 最小应内缩到 10：{xs:?}"
        );

        // fill mode：内形状顶点直接作为裁剪多边形。
        style.border_shape = BorderShapeValue::Fill {
            outer: ClipPathValue::None,
            outer_box: BorderShapeGeometryBox::BorderBox,
            inner: ClipPathValue::Circle {
                radius: ClipPathRadius::Length(LengthValue::Px(30.0)),
                position: None,
            },
            inner_box: BorderShapeGeometryBox::PaddingBox,
        };
        let poly = border_shape_overflow_polygon(&style, &box_node, 0.0, 0.0).expect("fill mode 应产出裁剪多边形");
        assert!(poly.len() >= 3, "内形状（circle 64 段）应 ≥3 顶点：{}", poly.len());
        let cx = 50.0_f32;
        let r_max = poly
            .iter()
            .map(|(x, y)| ((x - cx).powi(2) + (y - cx).powi(2)).sqrt())
            .fold(f32::MIN, f32::max);
        assert!((r_max - 30.0).abs() < 1.0, "内形状半径应 ≈30（64 段逼近）：{r_max}");
    }
}
