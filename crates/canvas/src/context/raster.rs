//! Canvas 2D 渲染上下文 — 私有光栅化辅助方法。

use zero_render_foundation::color::Color;
use zero_render_foundation::geometry::Rect;

use crate::path::{Path2D, PathCommand};

use super::types::*;

// ── Blend mode helpers（W3C Compositing and Blending Level 1 §9 separable + §10 non-separable）──
// R3239：blend 模式先把源色 Cs 与背景色 Cb 按 B(Cb,Cs) 混合得 blended source
// `Cs' = (1 - αb)·Cs + αb·B(Cb,Cs)`，再以 source-over 合成因子与背景合成（§13）。
// Porter-Duff 模式不 blend（Cs' = Cs）。

/// §9 separable 基础公式（per-channel）。
fn blend_multiply(cb: f32, cs: f32) -> f32 {
    cb * cs
}
fn blend_screen(cb: f32, cs: f32) -> f32 {
    cb + cs - cb * cs
}
fn blend_hard_light(cb: f32, cs: f32) -> f32 {
    if cs <= 0.5 {
        blend_multiply(cb, 2.0 * cs)
    } else {
        blend_screen(cb, 2.0 * cs - 1.0)
    }
}

/// §9 separable per-channel B(Cb, Cs)。调用方保证 op 为 separable 模式。
fn separable_blend(op: CompositeOperation, cb: f32, cs: f32) -> f32 {
    match op {
        CompositeOperation::Multiply => blend_multiply(cb, cs),
        CompositeOperation::Screen => blend_screen(cb, cs),
        // overlay(Cb,Cs) = hard_light(Cs,Cb)（spec 交换参数）。
        CompositeOperation::Overlay => blend_hard_light(cs, cb),
        CompositeOperation::Darken => cb.min(cs),
        CompositeOperation::Lighten => cb.max(cs),
        CompositeOperation::ColorDodge => {
            if cb <= 0.0 {
                0.0
            } else if cs >= 1.0 {
                1.0
            } else {
                (cb / (1.0 - cs)).min(1.0)
            }
        }
        CompositeOperation::ColorBurn => {
            if cb >= 1.0 {
                1.0
            } else if cs <= 0.0 {
                0.0
            } else {
                1.0 - ((1.0 - cb) / cs).min(1.0)
            }
        }
        CompositeOperation::HardLight => blend_hard_light(cb, cs),
        CompositeOperation::SoftLight => {
            if cs <= 0.5 {
                cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb)
            } else {
                let d = if cb <= 0.25 {
                    ((16.0 * cb - 12.0) * cb + 4.0) * cb
                } else {
                    cb.sqrt()
                };
                cb + (2.0 * cs - 1.0) * (d - cb)
            }
        }
        CompositeOperation::Difference => (cb - cs).abs(),
        CompositeOperation::Exclusion => cb + cs - 2.0 * cb * cs,
        // 非 separable 不应达此（调用方分支保证）；保险返 cs。
        _ => cs,
    }
}

type Rgb = [f32; 3];

/// §10.1 亮度（Rec. 601 系数）。
fn lum(c: Rgb) -> f32 {
    0.3 * c[0] + 0.59 * c[1] + 0.11 * c[2]
}

/// §10.2 ClipColor——裁剪到 [0,1]（保持亮度，min/max 取自原始 C，两 clip 序贯作用于 C）。
fn clip_color(c: Rgb) -> Rgb {
    let l = lum(c);
    let mn = c[0].min(c[1]).min(c[2]);
    let mx = c[0].max(c[1]).max(c[2]);
    let mut out = c;
    if mn < 0.0 {
        let d = l - mn;
        if d != 0.0 {
            out = [
                l + (out[0] - l) * l / d,
                l + (out[1] - l) * l / d,
                l + (out[2] - l) * l / d,
            ];
        }
    }
    if mx > 1.0 {
        let d = mx - l;
        if d != 0.0 {
            out = [
                l + (out[0] - l) * (1.0 - l) / d,
                l + (out[1] - l) * (1.0 - l) / d,
                l + (out[2] - l) * (1.0 - l) / d,
            ];
        }
    }
    out
}

/// §10.2 SetLum——设亮度为 l（加偏移后 ClipColor）。
fn set_lum(c: Rgb, l: f32) -> Rgb {
    let d = l - lum(c);
    clip_color([c[0] + d, c[1] + d, c[2] + d])
}

/// §10.1 Sat——饱和度（max - min）。
fn sat(c: Rgb) -> f32 {
    c[0].max(c[1]).max(c[2]) - c[0].min(c[1]).min(c[2])
}

/// R56c（M8/DC-8）：反转向量尾部 [start..] 的段序列环绕方向——每段 (a→b) 变
/// (b→a) 且整体倒序（子路径几何不变、绕组符号翻转；roundRect 单轴镜像用）。
fn reverse_subpath(vertices: &mut Vec<f32>, start: usize) {
    let segs = (vertices.len() - start) / 4;
    if segs == 0 {
        return;
    }
    let mut out = Vec::with_capacity(segs * 4);
    for i in (0..segs).rev() {
        let s = start + i * 4;
        // 段 (p1 → p2) 反转为 (p2 → p1)
        out.push(vertices[s + 2]);
        out.push(vertices[s + 3]);
        out.push(vertices[s]);
        out.push(vertices[s + 1]);
    }
    vertices.truncate(start);
    vertices.extend_from_slice(&out);
}

/// R56c（M8/DC-8）：填充规则（spec dom-context-2d-fill 的 fillRule 参数）。
/// https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-fill
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FillRule {
    /// 非零环绕数（默认）。
    NonZero,
    /// 奇偶规则。
    EvenOdd,
}

/// R56c（M8/DC-8）：按填充规则求扫描行填充区间。
/// flatten 段序列（x1,y1,x2,y2 × n）与扫描线 sy 的交点：
/// - NonZero（默认）：交点带方向（段向下 = +1 / 向上 = −1，屏幕 y 向下），按 x
///   排序后累计绕组，绕组非零区间填充。嵌套同向子路径（绕组 ±2）与对角连线
///   杂散交点在偶奇两两配对下都会破裂（挖假洞/漏填）。
/// - EvenOdd：交点计数，奇偶切换填充区间。
pub(crate) fn fill_rule_spans(vertices: &[f32], sy: f32, rule: FillRule) -> Vec<(f32, f32)> {
    let mut crossings: Vec<(f32, i32)> = Vec::new();
    for seg in vertices.chunks_exact(4) {
        let (x1, y1, x2, y2) = (seg[0], seg[1], seg[2], seg[3]);
        if (y1 <= sy && y2 > sy) || (y2 <= sy && y1 > sy) {
            let t = (sy - y1) / (y2 - y1);
            let dir = if y2 > y1 { 1 } else { -1 };
            crossings.push((x1 + t * (x2 - x1), dir));
        }
    }
    crossings.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut spans = Vec::new();
    match rule {
        FillRule::NonZero => {
            let mut winding: i32 = 0;
            let mut span_start: Option<f32> = None;
            for &(x, d) in &crossings {
                let was_zero = winding == 0;
                winding += d;
                if was_zero && winding != 0 {
                    span_start = Some(x);
                } else if !was_zero
                    && winding == 0
                    && let Some(s) = span_start.take()
                {
                    spans.push((s, x));
                }
            }
        }
        FillRule::EvenOdd => {
            let xs: Vec<f32> = crossings.iter().map(|&(x, _)| x).collect();
            for pair in xs.chunks_exact(2) {
                spans.push((pair[0], pair[1]));
            }
        }
    }
    spans
}

/// §10.3 SetSat——设饱和度为 s（保持各通道大小序，min→0/mid→插值/max→s）。
fn set_sat(c: Rgb, s: f32) -> Rgb {
    let mut idx = [0usize, 1, 2];
    idx.sort_by(|&a, &b| c[a].total_cmp(&c[b]));
    let (imin, imid, imax) = (idx[0], idx[1], idx[2]);
    let (cmin, cmid, cmax) = (c[imin], c[imid], c[imax]);
    let mut out = c;
    if cmax > cmin {
        out[imid] = ((cmid - cmin) * s) / (cmax - cmin);
        out[imax] = s;
    } else {
        out[imid] = 0.0;
        out[imax] = 0.0;
    }
    out[imin] = 0.0;
    out
}

/// §10 non-separable B(Cb, Cs)。调用方保证 op 为 non-separable 模式。
fn nonseparable_blend(op: CompositeOperation, cb: Rgb, cs: Rgb) -> Rgb {
    match op {
        CompositeOperation::Hue => set_lum(set_sat(cs, sat(cb)), lum(cb)),
        CompositeOperation::Saturation => set_lum(set_sat(cb, sat(cs)), lum(cb)),
        CompositeOperation::Color => set_lum(cs, lum(cb)),
        CompositeOperation::Luminosity => set_lum(cb, lum(cs)),
        _ => cs,
    }
}

/// 计算 composite_pixel 用的源色（blend 模式返回 Cs'，Porter-Duff 返回 Cs 原样）。
fn blend_source_color(op: CompositeOperation, da: f32, src: Rgb, dst: Rgb) -> Rgb {
    let b = match op {
        CompositeOperation::Multiply
        | CompositeOperation::Screen
        | CompositeOperation::Overlay
        | CompositeOperation::Darken
        | CompositeOperation::Lighten
        | CompositeOperation::ColorDodge
        | CompositeOperation::ColorBurn
        | CompositeOperation::HardLight
        | CompositeOperation::SoftLight
        | CompositeOperation::Difference
        | CompositeOperation::Exclusion => [
            separable_blend(op, dst[0], src[0]),
            separable_blend(op, dst[1], src[1]),
            separable_blend(op, dst[2], src[2]),
        ],
        CompositeOperation::Hue
        | CompositeOperation::Saturation
        | CompositeOperation::Color
        | CompositeOperation::Luminosity => nonseparable_blend(op, dst, src),
        // Porter-Duff：不 blend。
        _ => return src,
    };
    // Cs' = (1 - αb)·Cs + αb·B(Cb,Cs)
    [
        (1.0 - da) * src[0] + da * b[0],
        (1.0 - da) * src[1] + da * b[1],
        (1.0 - da) * src[2] + da * b[2],
    ]
}

/// R34xx：点在三角形内判定（重心同侧法——阴影 join 三角 mask 用）。
#[allow(clippy::too_many_arguments)]
pub(crate) fn point_in_triangle(px: f32, py: f32, ax: f32, ay: f32, bx: f32, by: f32, cx: f32, cy: f32) -> bool {
    let sign = |x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32| (x1 - x3) * (y2 - y3) - (x2 - x3) * (y1 - y3);
    let d1 = sign(px, py, ax, ay, bx, by);
    let d2 = sign(px, py, bx, by, cx, cy);
    let d3 = sign(px, py, cx, cy, ax, ay);
    let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
    let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
    !(has_neg && has_pos)
}

/// R3240：单遍可分离 box blur（边缘 clamp）作用于 alpha mask。半径 0 / 空 mask 为 no-op。
/// 窗口样本数恒为 `2r+1`（边缘重复采样），故除数固定 `2r+1`。
pub(crate) fn box_blur_alpha(buf: &mut [u8], w: usize, h: usize, radius: usize) {
    if radius == 0 || w == 0 || h == 0 {
        return;
    }
    let r = radius as isize;
    let win = (2 * radius + 1) as u32;
    // 水平 pass：buf → tmp
    let mut tmp = vec![0u8; w * h];
    for y in 0..h {
        let row = &buf[y * w..(y + 1) * w];
        for x in 0..w {
            let mut sum: u32 = 0;
            for dx in -r..=r {
                let xx = (x as isize + dx).clamp(0, w as isize - 1) as usize;
                sum += row[xx] as u32;
            }
            tmp[y * w + x] = (sum / win) as u8;
        }
    }
    // 垂直 pass：tmp → buf
    for y in 0..h {
        for x in 0..w {
            let mut sum: u32 = 0;
            for dy in -r..=r {
                let yy = (y as isize + dy).clamp(0, h as isize - 1) as usize;
                sum += tmp[yy * w + x] as u32;
            }
            buf[y * w + x] = (sum / win) as u8;
        }
    }
}

/// R3242：shadowBlur 几何参数——返回 `(radius_per_pass, pad, passes)`。
/// 3 遍 box blur（半径 `round(blur/2)`）≈ gaussian（W3C 阴影软度标准近似），比 R3240 单遍 triangle
/// 衰减更平滑。`pad = 3·radius` 覆盖 3 遍总扩散。`blur<=0` 返 `(0,0,0)`（硬边，no-op）。
///
/// R3355：半径封顶 `MAX_RADIUS`（8192）。shadowBlur 极大值（如 1e30）经 `(blur/2).round() as i32`
/// 饱和到 i32::MAX，致下游三重故障：① pad = 3·i32::MAX 经 `as i32` 饱和到 i32::MAX，region padding
/// 的 i32 加减法溢出（draw_shadow_*，已另改 saturating_add/sub）；② box_blur_alpha 的 `for dx in -r..=r`
/// 达 ~4.3e9 次迭代挂起；③ box_blur 的 `sum: u32` 在 ~17M 次累加后溢出 panic。封顶后 pad=24576 远在
/// i32 内、box_blur 窗口 16385 窗样本 sum 上限 16385×255≈4.2M（u32 安全）、迭代量可控。封顶值远超
/// 任何可见阴影软度（Chrome 实践上限同量级），spec 无强制上限，属合理实现限制。
pub(crate) const SHADOW_BLUR_MAX_RADIUS: usize = 8192;

pub(crate) fn shadow_blur_geom(blur: f32) -> (usize, i32, u32) {
    if blur <= 0.0 {
        return (0, 0, 0);
    }
    let r = (((blur / 2.0).round() as i32).max(1) as usize).min(SHADOW_BLUR_MAX_RADIUS);
    (r, (3 * r) as i32, 3)
}

/// R3241：把 canvas 坐标矩形 `rect` 填入 region 局部 mask（覆盖像素写 255）。供 stroke shadow
/// 足迹（每段 thick rect + 连接点方块）构 mask 用。`ox/oy` 为 region 左上角 canvas 坐标。
pub(crate) fn fill_rect_into_mask(mask: &mut [u8], rw: usize, rh: usize, ox: i32, oy: i32, rect: &Rect) {
    let cx_start = rect.left().ceil() as i32;
    let cy_start = rect.top().ceil() as i32;
    let cx_end = (rect.right().ceil() as i32) + 1;
    let cy_end = (rect.bottom().ceil() as i32) + 1;
    for cy in cy_start..cy_end {
        let ly = cy - oy;
        if ly < 0 || (ly as usize) >= rh {
            continue;
        }
        for cx in cx_start..cx_end {
            let lx = cx - ox;
            if lx >= 0 && (lx as usize) < rw {
                mask[(ly as usize) * rw + (lx as usize)] = 255;
            }
        }
    }
}

/// R57（M3）：flatten 段序列（每 4 个 f32 = 一段 (x1,y1,x2,y2)）→ 图元点序列
///（每 2 个 f32 = 一个顶点，闭合多边形）。RenderPrimitives 的 PathFill/PathStroke
/// 契约是点序列（mesh.rs push_path_fill_mesh / push_path_stroke_mesh 按每 2 个
/// f32 解析）——段序列的段间相邻重复点（A→B→C→A 环的 B、C 各出现两次）会生成
/// 退化三角形/零长线（R57 gpu_path 实测旋转三角形全白）。取每段起点并去重相邻
/// 重复，环 A→B→C→A → [A,B,C]。
pub(crate) fn segs_to_point_verts(segments: &[f32]) -> Vec<f32> {
    let mut out: Vec<f32> = Vec::with_capacity(segments.len() / 2 + 2);
    for seg in segments.chunks_exact(4) {
        let (x, y) = (seg[0], seg[1]);
        let n = out.len();
        if n >= 2 && (out[n - 2] - x).abs() <= f32::EPSILON && (out[n - 1] - y).abs() <= f32::EPSILON {
            continue;
        }
        out.push(x);
        out.push(y);
    }
    out
}

/// R3240：把路径顶点（canvas 坐标）扫描线光栅化为 alpha 覆盖 mask（region 局部）——
/// 覆盖像素写 255，其余 0。`ox/oy` 为 region 左上角 canvas 坐标（mask-local = canvas - origin）。
pub(crate) fn rasterize_path_coverage(vertices: &[f32], mask: &mut [u8], rw: usize, rh: usize, ox: i32, oy: i32) {
    if vertices.len() < 4 || rw == 0 || rh == 0 {
        return;
    }
    let points: Vec<(f32, f32)> = vertices.chunks_exact(2).map(|c| (c[0], c[1])).collect();
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;
    for &(_, y) in &points {
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    // 逐 mask 行扫描：sy（canvas 坐标）在路径 y 范围内时求交、填充覆盖。
    // R56：同 blit_path_to_pixels——vertices 是独立段序列，按段判定穿越（多子路径
    // 跨界虚假边会破坏 clip mask 交点配对）。
    let rwi = rw as i32;
    for ly in 0..rh as i32 {
        let sy = (oy + ly) as f32 + 0.5;
        if sy < min_y || sy > max_y {
            continue;
        }
        let mut xs: Vec<f32> = Vec::new();
        for seg in vertices.chunks_exact(4) {
            let (x1, y1, x2, y2) = (seg[0], seg[1], seg[2], seg[3]);
            if (y1 <= sy && y2 > sy) || (y2 <= sy && y1 > sy) {
                let t = (sy - y1) / (y2 - y1);
                xs.push(x1 + t * (x2 - x1));
            }
        }
        xs.sort_by(|a, b| a.total_cmp(b));
        for pair in xs.chunks_exact(2) {
            let ix_start = ((pair[0] - ox as f32).max(0.0) as i32).max(0);
            let ix_end = ((pair[1] - ox as f32).min(rw as f32) as i32).min(rwi);
            for lx in ix_start..ix_end {
                mask[(ly as usize) * rw + (lx as usize)] = 255;
            }
        }
    }
}

impl CanvasContext {
    // ── Private helpers ──

    /// 对矩形应用当前变换。
    /// R57（M3）：像素 (px, py) 的覆盖分数——非轴对齐 CTM 下旋转矩形边界像素的
    /// 亚像素覆盖率（4×4 超采样，ref Chromium AA 边的半色调）。轴对齐时恒 1.0
    ///（包围盒 == 矩形，零回归——整数矩形硬边正确）。composite.grid 的
    /// rotate(90°)+scale 变换后 fillRect 旋转边差（oracle A/B 24% 之一）。
    pub(crate) fn pixel_in_transformed_rect(&self, px: f32, py: f32, x: f32, y: f32, w: f32, h: f32) -> bool {
        self.rect_coverage(px, py, x, y, w, h) > 0.0
    }

    /// 像素 (px, py) 相对变换后矩形 (x,y,w,h) 的覆盖率 [0,1]。轴对齐恒 1（零回归）；
    /// 非轴对齐 4×4 子像素中心采样（覆盖率 = 子像素命中比例）。
    pub(crate) fn rect_coverage(&self, px: f32, py: f32, x: f32, y: f32, w: f32, h: f32) -> f32 {
        if self.transform.b.abs() < 1e-6 && self.transform.c.abs() < 1e-6 {
            return 1.0;
        }
        let inv = self.transform.inverse();
        let mut hit = 0u32;
        // 4×4 超采样：子像素中心偏移 (i+0.5)/4, (j+0.5)/4
        for i in 0..4 {
            for j in 0..4 {
                let (ux, uy) = inv.transform_point(px + (i as f32 + 0.5) / 4.0, py + (j as f32 + 0.5) / 4.0);
                if ux >= x && uy >= y && ux < x + w && uy < y + h {
                    hit += 1;
                }
            }
        }
        hit as f32 / 16.0
    }

    /// R57（M3）：路径边界像素的 AA 覆盖率——4×4 超采样 + 点内测试
    ///（`spans_hit`：与填充光栅化同一非零/偶奇判定，边界像素与内部像素
    /// 同一「扫描线 span 覆盖」语义）。轴对齐 CTM 恒 1（整数矩形硬边零回归）；
    /// 非轴对齐 CTM 下旋转路径边像素半色调（与 [`Self::rect_coverage`] 同模式）。
    fn path_pixel_coverage(&self, vertices: &[f32], px: f32, py: f32, rule: FillRule) -> f32 {
        if self.transform.b.abs() < 1e-6 && self.transform.c.abs() < 1e-6 {
            return 1.0;
        }
        let mut hit = 0u32;
        for i in 0..4 {
            for j in 0..4 {
                let (sx, sy) = (px + (i as f32 + 0.5) / 4.0, py + (j as f32 + 0.5) / 4.0);
                if super::context_impl::spans_hit(vertices, sx, sy, rule) {
                    hit += 1;
                }
            }
        }
        hit as f32 / 16.0
    }

    /// R57（M3）：路径 span 边界像素合成——coverage ∈ (0,1) 时源 alpha × coverage
    ///（AA 边半色调；全覆盖像素走调用方原逻辑）。clip + composite + f32 并行写
    /// 与内部像素一致（仅 alpha 缩放）。
    fn blit_path_edge_pixel(&mut self, scan_x: u32, scan_y: u32, color: Color, coverage: f32) {
        // 双写防护：coverage ≥ 1 的边界像素已被全覆盖循环写入（整数边界硬边）。
        if coverage <= 0.0 || coverage >= 1.0 {
            return;
        }
        // R34xx：clip 区域裁剪（与全覆盖像素同语义）。
        if !self.clip_applies(scan_x as f32, scan_y as f32) {
            return;
        }
        let canvas_w = self.width as usize;
        let idx = (scan_y as usize * canvas_w + scan_x as usize) * 4;
        if idx + 3 >= self.pixel_buffer.len() {
            return;
        }
        let mut c = color;
        c.a = (c.a as f32 * coverage).round() as u8;
        let (r, g, b, a) = self.composite_pixel(
            c,
            self.pixel_buffer[idx],
            self.pixel_buffer[idx + 1],
            self.pixel_buffer[idx + 2],
            self.pixel_buffer[idx + 3],
        );
        self.pixel_buffer[idx] = r;
        self.pixel_buffer[idx + 1] = g;
        self.pixel_buffer[idx + 2] = b;
        self.pixel_buffer[idx + 3] = a;
        // R34xx（f16 存储）：float16 画布并行 f32 写（与 blit_rect_to_pixels_checked
        // 同模式——coverage 只缩放 u8 主缓冲）。
        self.blit_pixel_f32(idx, color);
    }

    pub(crate) fn transform_rect(&self, x: f32, y: f32, width: f32, height: f32) -> Rect {
        let (x1, y1) = self.transform.transform_point(x, y);
        let (x2, y2) = self.transform.transform_point(x + width, y + height);
        let min_x = x1.min(x2);
        let min_y = y1.min(y2);
        let max_x = x1.max(x2);
        let max_y = y1.max(y2);
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// 应用 global_alpha 到颜色。
    pub(crate) fn apply_alpha(&self, color: Color) -> Color {
        let a = ((color.a as f32) * self.global_alpha) as u8;
        Color::rgba(color.r, color.g, color.b, a)
    }

    /// 将圆角矩形扁平化为线段顶点。
    /// 每个圆角使用 8 段线段近似四分之一圆弧。
    /// radii 遵循 Canvas 规范：1 个值用于全部角，2 个值为 [左上/右下, 右上/左下]，4 个值为 [左上, 右上, 右下, 左下]。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn flatten_round_rect(
        vertices: &mut Vec<f32>,
        _current_x: f32,
        _current_y: f32,
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        radii: &[(f32, f32)],
    ) -> (f32, f32) {
        // R56（M8/DC-8）：负 w/h 归一化——spec §roundrect 的角序 [tl,tr,br,bl] 相对
        // **参数坐标系**（tl 恒贴 (x,y) 参数角），负 w/h 翻转边走向即镜像矩形，角随边
        // 镜像到对侧（负 h：tl↔bl、tr↔br；负 w：tl↔tr、bl↔br）。归一到包围盒后把
        // 半径数组按翻转换回几何角序（2d.path.roundrect.negative：roundRect(0,50,50,-25,[10,..])
        // 的 tl 圆贴参数角 (0,50)，镜像后落在包围盒左下）。旧实现负 w/h 直接产出反向
        // 多边形（扫描线偶填充翻外）。
        // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-roundrect
        let (w_was_neg, h_was_neg) = (w < 0.0, h < 0.0);
        let (aw, ah) = (w.abs(), h.abs());
        let (x, y) = (x + w.min(0.0), y + h.min(0.0));
        let (w, h) = (aw, ah);
        // radii 参数角序 → 包围盒几何角序：参数 tl 角经镜像落在包围盒的新角位。
        // 垂直镜像（负 h）tl↔bl/tr↔br，水平镜像（负 w）tl↔tr/bl↔br，双向即 180° 旋转。
        // 注意：镜像换位须在「展开到 4 角」之后做（radii 可能只有 1-3 项——
        // [a,b] 展开序 tl,tr,br,bl = a,b,a,b，先换位再展开会错角）。
        //
        // R56c（M8/DC-8）：记录 emit 起始——单轴镜像（恰好一负）时输出段序在函数
        // 末尾整体反转（环绕方向翻转）。真浏览器 roundRect 沿**参数边方向**环绕：
        // 负 w/h 单轴镜像的矩形与正参数矩形**反向**（roundrect.winding 实证——
        // 左半顺 × 下半反 同区域绕组对消不填，偶奇光栅无方向语义故旧实现不暴露）。
        let emit_start = vertices.len();
        let mut r = [(0.0f32, 0.0f32); 4];
        match radii.len() {
            0 => {}
            1 => {
                r = [radii[0]; 4];
            }
            2 => {
                r = [radii[0], radii[1], radii[0], radii[1]];
            }
            3 => {
                r = [radii[0], radii[1], radii[2], radii[1]];
            }
            _ => {
                r = [radii[0], radii[1], radii[2], radii[3]];
            }
        }
        if w_was_neg != h_was_neg {
            // 单轴镜像：垂直（负 h）0↔3、1↔2；水平（负 w）0↔1、3↔2。
            if h_was_neg {
                r.swap(0, 3);
                r.swap(1, 2);
            } else {
                r.swap(0, 1);
                r.swap(3, 2);
            }
        } else if w_was_neg {
            // 180° 旋转：0↔2、1↔3。
            r.swap(0, 2);
            r.swap(1, 3);
        }
        // R34xx：按比例缩放（spec §roundrect：scale = min(w/2/maxRx, h/2/maxRy, 1)，
        // 保持半径纵横比——旧 clamp 到短边/2 把 (40,20) 压成 (25,25)）。
        let max_rx = r.iter().map(|&(rx, _)| rx).fold(0.0f32, f32::max);
        let max_ry = r.iter().map(|&(_, ry)| ry).fold(0.0f32, f32::max);
        let scale = if max_rx > 0.0 || max_ry > 0.0 {
            let sx = if max_rx > 0.0 { w / 2.0 / max_rx } else { f32::MAX };
            let sy = if max_ry > 0.0 { h / 2.0 / max_ry } else { f32::MAX };
            sx.min(sy).clamp(0.0, 1.0)
        } else {
            1.0
        };
        for radius in &mut r {
            radius.0 *= scale;
            radius.1 *= scale;
        }

        // 所有半径为 0 时退化为矩形
        if r.iter().all(|&(rx, ry)| rx < f32::EPSILON && ry < f32::EPSILON) {
            // R56：自包含子路径——与圆角分支/path.rs 同语义，不连 current（旧连接段
            // 在 current 恰等于 corner3 时与闭合边重合，段式扫描产生奇数交点）。
            let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
            for i in 0..4 {
                let next = (i + 1) % 4;
                vertices.push(corners[i].0);
                vertices.push(corners[i].1);
                vertices.push(corners[next].0);
                vertices.push(corners[next].1);
            }
            if w_was_neg != h_was_neg {
                reverse_subpath(vertices, emit_start);
            }
            return (corners[0].0, corners[0].1);
        }

        // R34xx：圆角中心坐标（角对 x=水平/ y=垂直半径——椭圆弧）。
        // 左上角 (x + r[0].0, y + r[0].1), 角度 π ~ 3π/2
        // 右上角 (x + w - r[1].0, y + r[1].1), 角度 3π/2 ~ 2π
        // 右下角 (x + w - r[2].0, y + h - r[2].1), 角度 0 ~ π/2
        // 左下角 (x + r[3].0, y + h - r[3].1), 角度 π/2 ~ π
        let corner_cx = [x + r[0].0, x + w - r[1].0, x + w - r[2].0, x + r[3].0];
        let corner_cy = [y + r[0].1, y + r[1].1, y + h - r[2].1, y + h - r[3].1];
        let corner_start = [
            std::f32::consts::PI,
            std::f32::consts::FRAC_PI_2 * 3.0,
            0.0,
            std::f32::consts::FRAC_PI_2,
        ];
        let corner_end = [
            std::f32::consts::FRAC_PI_2 * 3.0,
            std::f32::consts::TAU,
            std::f32::consts::FRAC_PI_2,
            std::f32::consts::PI,
        ];

        // R34xx：16 段/角（旧 8 段对椭圆弧近似过粗——圆角外像素被折线误填，
        // 2d.path.roundrect.* DOMPoint 用例 (20,1) 期望椭圆外）。
        const CORNER_SEGMENTS: usize = 16;

        // R34xx：roundRect 是自包含子路径（round_rect 已 push MoveTo）——不连当前点。
        // R56h：子路径起点对齐 spec（dom-context-2d-roundrect）——subpath 从
        // (x + tl, y)（上边左端）起环：顶边 → 右上角弧 → 右边 → 右下角弧 → 底边
        // → 左下角弧 → 左边 → 左上角弧（闭合回起点）。旧实现从左上角弧的 π 角点
        // (x, y+tl) 起环，roundRect 后 current 落在 (x, y+tl)——后续 lineTo 从错误
        // 点出发（2d.path.roundrect.end.3 的 lineTo(-1,-1) 离画布像素 97px 漏覆盖）。
        let start_x = x + r[0].0;
        let start_y = y;
        let emit_arc = |c: usize, vertices: &mut Vec<f32>| {
            let step = (corner_end[c] - corner_start[c]) / CORNER_SEGMENTS as f32;
            let mut px = corner_cx[c] + r[c].0 * corner_start[c].cos();
            let mut py = corner_cy[c] + r[c].1 * corner_start[c].sin();
            for i in 0..CORNER_SEGMENTS {
                let angle = corner_start[c] + step * (i + 1) as f32;
                let nx = corner_cx[c] + r[c].0 * angle.cos();
                let ny = corner_cy[c] + r[c].1 * angle.sin();
                vertices.push(px);
                vertices.push(py);
                vertices.push(nx);
                vertices.push(ny);
                px = nx;
                py = ny;
            }
            // 从圆角末尾连线到下一个圆角的起点（即直边段）
            let next = (c + 1) % 4;
            let next_start = corner_start[next];
            let next_x = corner_cx[next] + r[next].0 * next_start.cos();
            let next_y = corner_cy[next] + r[next].1 * next_start.sin();
            vertices.push(px);
            vertices.push(py);
            vertices.push(next_x);
            vertices.push(next_y);
        };
        // 顶边：(x + tl, y) → 右上角弧起点 (x + w − tr, y)。
        vertices.push(start_x);
        vertices.push(start_y);
        vertices.push(x + w - r[1].0);
        vertices.push(y);
        // 遍历 4 个圆角（椭圆弧：x 用水平半径 r[c].0，y 用垂直半径 r[c].1）。
        for c in 1..=4 {
            emit_arc(c % 4, vertices);
        }
        // R56c：单轴镜像 → 段序反转（环绕方向沿参数边，见函数头注释）。
        if w_was_neg != h_was_neg {
            reverse_subpath(vertices, emit_start);
        }

        (start_x, start_y)
    }

    /// 计算 arcTo 的几何信息：返回 (切点1x, 切点1y, 切点2x, 切点2y)。
    /// 将当前路径命令扁平化为顶点列表（x, y 交替）。
    /// 对于圆弧，使用线性近似（固定 16 段细分）。
    /// `close_open_subpaths`：fill/clip/isPointInPath 需 **closepath-on-fill** 隐式
    /// 闭合开放子路径（spec dom-context-2d-fill；R56 段式扫描线无隐式闭合会丢
    /// 开放子路径的回边）；stroke/isPointInStroke 传 false（开放路径 stroke 不闭合）。
    pub(crate) fn flatten_path_opts(&self, close_open_subpaths: bool) -> Vec<f32> {
        let mut vertices = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut subpath_start_x = 0.0f32;
        let mut subpath_start_y = 0.0f32;
        // 子路径已有几何命令（区别「首个 MoveTo 前」——此时无子路径可闭合）。
        let mut subpath_has_geometry = false;
        // R56：是否已有任何子路径（MoveTo 已发生）——spec arc 步骤「If the context
        // has any subpaths, line to arc start」的判据（区别 subpath_has_geometry：
        // moveTo 后 arc 前为 false，但 spec 语义应连线）。
        let mut has_any_subpath = false;
        const ARC_SEGMENTS: usize = 16;

        for cmd in self.current_path.commands() {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
                    has_any_subpath = true;
                    // 隐式闭合上一开放子路径（终点≠起点补闭合段）。
                    if close_open_subpaths
                        && subpath_has_geometry
                        && ((current_x - subpath_start_x).abs() > f32::EPSILON
                            || (current_y - subpath_start_y).abs() > f32::EPSILON)
                    {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(subpath_start_x);
                        vertices.push(subpath_start_y);
                    }
                    subpath_start_x = x;
                    subpath_start_y = y;
                    current_x = x;
                    current_y = y;
                    subpath_has_geometry = false;
                }
                PathCommand::LineTo(x, y) => {
                    // R56e（M8/DC-8）：spec dom-context-2d-lineto——无任何子路径时
                    // lineTo 等同 moveTo（只设起点，不画隐含 (0,0)→目标 连线——
                    // 2d.path.lineTo.ensuresubpath.1：beginPath 后 lineTo(100,50)
                    // + stroke 期望画布保持底色）。
                    if has_any_subpath {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(x);
                        vertices.push(y);
                        subpath_has_geometry = true;
                    } else {
                        has_any_subpath = true;
                        subpath_start_x = x;
                        subpath_start_y = y;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
                    // R56e：spec dom-context-2d-quadraticcurveto——无任何子路径时
                    // 第一控制点被加入（current := (cpx,cpy) 成子路径起点），曲线
                    // 从该点照画（ensuresubpath.2 期望退化直线横穿画布）。
                    let (sx0, sy0) = if has_any_subpath {
                        (current_x, current_y)
                    } else {
                        has_any_subpath = true;
                        subpath_start_x = cpx;
                        subpath_start_y = cpy;
                        (cpx, cpy)
                    };
                    subpath_has_geometry = true;
                    // R56h（M8/DC-8）：段数自适应——控制点折线长 / 8px（[8,512]）。
                    // 固定 8 段对巨坐标曲线（bezierCurveTo.shape 控制折线 ~13000px）
                    // 弦偏差达 48px；按折线长缩放后画布内窗口获得足够段密度。
                    // R56i 修正：曲线起点用 sx0（无子路径时 = 第一控制点，spec
                    // ensuresubpath.2——旧 current_x 在无子路径时仍为 (0,0)，曲线
                    // 从原点错误出发）。
                    let poly = ((cpx - sx0).hypot(cpy - sy0) + (x - cpx).hypot(y - cpy)).max(1.0);
                    let segments = ((poly / 8.0).ceil() as usize).clamp(8, 512);
                    let mut px = sx0;
                    let mut py = sy0;
                    for i in 1..=segments {
                        let t = i as f32 / segments as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * sx0 + 2.0 * mt * t * cpx + t * t * x;
                        let ny = mt * mt * sy0 + 2.0 * mt * t * cpy + t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
                    // R56e：spec dom-context-2d-beziercurveto——无任何子路径时第一
                    // 控制点被加入（current := (cp1x,cp1y) 成子路径起点），曲线照画。
                    if !has_any_subpath {
                        has_any_subpath = true;
                        subpath_start_x = cp1x;
                        subpath_start_y = cp1y;
                        current_x = cp1x;
                        current_y = cp1y;
                    }
                    subpath_has_geometry = true;
                    // R56h：段数自适应（同 quadratic——控制折线长 / 8px，[8,512]）。
                    let poly = ((cp1x - current_x).hypot(cp1y - current_y)
                        + (cp2x - cp1x).hypot(cp2y - cp1y)
                        + (x - cp2x).hypot(y - cp2y))
                    .max(1.0);
                    let segments = ((poly / 8.0).ceil() as usize).clamp(8, 512);
                    let mut px = current_x;
                    let mut py = current_y;
                    for i in 1..=segments {
                        let t = i as f32 / segments as f32;
                        let mt = 1.0 - t;
                        let nx = mt * mt * mt * current_x
                            + 3.0 * mt * mt * t * cp1x
                            + 3.0 * mt * t * t * cp2x
                            + t * t * t * x;
                        let ny = mt * mt * mt * current_y
                            + 3.0 * mt * mt * t * cp1y
                            + 3.0 * mt * t * t * cp2y
                            + t * t * t * y;
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = x;
                    current_y = y;
                }
                PathCommand::Arc(cx, cy, radius, start_angle, end_angle, anticlockwise) => {
                    subpath_has_geometry = true;
                    // R34xx：anticlockwise 方向（同 path.rs flatten）。
                    let dir = if anticlockwise { -1.0 } else { 1.0 };
                    // R56（M8/DC-8）：角度归一化对齐 spec dom-context-2d-arc——
                    // |span| ≥ 2π 整圆；否则按方向取**同向** mod 2π 弧：顺时针
                    // （acw=false）span ∈ [0,2π)、逆时针 span ∈ (−2π,0]。
                    // 旧 `raw % TAU` 对顺时针负差得负 span → 弧走向反向
                    // （2d.path.arc.angle.5：start=1023π end=512.5π 顺时针，
                    // 旧 span=−π/2 画成逆时针 π/2 弧，扇形翻到对侧）。
                    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-arc
                    let tau = std::f32::consts::TAU;
                    let raw_span = end_angle - start_angle;
                    let span = if !anticlockwise {
                        if raw_span >= tau {
                            tau
                        } else {
                            ((raw_span % tau) + tau) % tau
                        }
                    } else if raw_span <= -tau {
                        -tau
                    } else {
                        // R34xx：mod==0 且 raw≠0 → 整圆（arc(0, 2π, true) 逆时针
                        // 全圆——2d.line.join.round 的圆盘填充；Chromium 语义）。
                        let m = -(((-raw_span) % tau + tau) % tau);
                        if m == 0.0 && raw_span != 0.0 { -tau } else { m }
                    };
                    // R56：span 归一化已含方向（顺时针 ∈ [0,τ] / 逆时针 ∈ [−τ,0]），
                    // 不再乘 dir（旧 span 为同号绝对值需 dir 定向——双重取反会翻弧）。
                    let _ = dir;
                    let angle_span = span;
                    // R56g（M8/DC-8）：弧折线段数自适应——粗 stroke（lineWidth/radius
                    // 比大）下 16 段折线的斜段矩形会侧向掠入弧带外（2d.path.arc.shape.1：
                    // r=50 lw=50 半圆，16 段首段矩形覆盖 (20,48) 距弧 40px 区域；
                    // 掠深 ≈ half_lw·sin(Δθ)，Δθ=τ/N）。τ/16≈22.5° 在 lw≈r 时掠深
                    // ≈ 9.5px 超容差；τ/64≈5.6° 掠深 ≈ 2.4px 消除。fill 侧扫描线
                    // 逐像素判定不受段数影响（精度单调提升）。
                    let arc_segments = if radius > 1e-6 {
                        // R56g：粗线（lw/r > 0.25）时弦切向偏差 δ=τ/2N 把端面投影
                        // 翻正——径向距离 d_r 的像素需要 d_r·sin(τ/2N) < d_t（端面
                        // 外距离，最小 1px）→ N > τ/(2·asin(1/d_r))。d_r~画布尺度
                        // ~150px → N~470；按 ratio 缩放取 N = min(512, 64·lw/r)。
                        // 实证：lw/r=1（shape.1）→64 ✓；lw/r=2（shape.3）→128 ✓。
                        let ratio = self.line_width / radius;
                        if ratio > 0.25 {
                            ((64.0 * ratio).round() as usize).min(512)
                        } else {
                            ARC_SEGMENTS
                        }
                    } else {
                        ARC_SEGMENTS
                    };
                    // R56g（M8/DC-8）：各向异性 CTM（sx≠sy）——spec 弧是用户空间的
                    // 圆经 CTM 变换（= 设备空间椭圆）。arc() 只变换了圆心，flatten
                    // 时以未变换半径画**圆**（arc.scale.1 的 scale(2,0.5)：r=56 的
                    // 圆应成 x 112/y 28 椭圆）。同 R56f arcTo 模式：控制点逆变换
                    // 回用户空间 → 构造圆弧 → 输出点逐点正变换。
                    let t = &self.transform;
                    let identity_ctm = (t.a - 1.0).abs() < 1e-9
                        && t.d.abs() < 1e-9
                        && t.b.abs() < 1e-9
                        && t.c.abs() < 1e-9
                        && t.e.abs() < 1e-9
                        && t.f.abs() < 1e-9;
                    // R56g：非恒等 CTM 统一走用户空间构造——弧顶点 = 圆心(已变换)
                    // + 用户半径·(cos,sin) 从未做 CTM 缩放（均匀大缩放同样错：
                    // arc.scale.2 的 scale(100,100) r=0.6 应成 r=60 圆，旧路径画
                    // 0.6px 小点——非恒等即改走逆变换→圆弧→正变换）。
                    if !identity_ctm {
                        let inv = self.transform.inverse();
                        let (ucx, ucy) = inv.transform_point(cx, cy);
                        let (u_px, u_py) = (ucx + radius * start_angle.cos(), ucy + radius * start_angle.sin());
                        if has_any_subpath {
                            vertices.push(current_x);
                            vertices.push(current_y);
                            let (d_px, d_py) = self.transform.transform_point(u_px, u_py);
                            vertices.push(d_px);
                            vertices.push(d_py);
                        } else {
                            has_any_subpath = true;
                        }
                        let mut prev_dev = self.transform.transform_point(u_px, u_py);
                        for i in 0..arc_segments {
                            let angle_i = start_angle + angle_span / arc_segments as f32 * (i + 1) as f32;
                            let (u_nx, u_ny) = (ucx + radius * angle_i.cos(), ucy + radius * angle_i.sin());
                            let dev = self.transform.transform_point(u_nx, u_ny);
                            vertices.push(prev_dev.0);
                            vertices.push(prev_dev.1);
                            vertices.push(dev.0);
                            vertices.push(dev.1);
                            prev_dev = dev;
                        }
                        current_x = prev_dev.0;
                        current_y = prev_dev.1;
                        // 弧自闭合子路径起点重置（同 R56d 语义——用户空间起点
                        // 正变换后与设备空间末点比较）。
                        let (d_start_x, d_start_y) = self.transform.transform_point(u_px, u_py);
                        if subpath_start_x == 0.0
                            && subpath_start_y == 0.0
                            && (prev_dev.0 - d_start_x).abs() <= 1e-3
                            && (prev_dev.1 - d_start_y).abs() <= 1e-3
                        {
                            subpath_start_x = d_start_x;
                            subpath_start_y = d_start_y;
                        }
                        continue;
                    }
                    let step = angle_span / arc_segments as f32;
                    let mut angle = start_angle;
                    let mut px = cx + radius * angle.cos();
                    let mut py = cy + radius * angle.sin();
                    // R56（M8/DC-8）：spec dom-context-2d-arc——若已有子路径，从当前
                    // 点直线连到弧起点（spec 步骤「add a straight line from the
                    // current point to the start point of the arc」）。旧实现不 push
                    // 该段，moveTo+arc 整圆 fill 的多边形缺这条边，扫描线在弧首角
                    // 配对破裂（2d.path.arc.angle.4 的 (98,48) 缺口）。首个子路径的
                    // 首个命令（无 current）不连。
                    if has_any_subpath {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(px);
                        vertices.push(py);
                    } else {
                        has_any_subpath = true;
                    }
                    for i in 0..arc_segments {
                        angle = start_angle + step * (i + 1) as f32;
                        let nx = cx + radius * angle.cos();
                        let ny = cy + radius * angle.sin();
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                    // R56d：无 MoveTo 直接 arc（首个子路径）时 subpath_start 仍是
                    // 初值 (0,0)——closepath-on-fill 末尾会补出弧末→(0,0) 的杂散
                    // 对角线（bigarc 的 17 段）。弧自闭合（末角 ≡ 首角，浮点差
                    // < 1e-4）时以弧起点为子路径起点。
                    if (px - subpath_start_x).abs() > 1e-4 || (py - subpath_start_y).abs() > 1e-4 {
                        // 末点未回到子路径起点：若子路径起点仍是初值且弧自身近闭合，
                        // 视为弧自包含子路径——重置起点为弧首点。
                        let arc_start_x = cx + radius * start_angle.cos();
                        let arc_start_y = cy + radius * start_angle.sin();
                        if subpath_start_x == 0.0
                            && subpath_start_y == 0.0
                            && (px - arc_start_x).abs() <= 1e-3
                            && (py - arc_start_y).abs() <= 1e-3
                        {
                            subpath_start_x = arc_start_x;
                            subpath_start_y = arc_start_y;
                        }
                    }
                }
                PathCommand::ArcTo(x1, y1, x2, y2, radius) => {
                    // R56f（M8/DC-8）：arcTo 真切线弧（path.rs 共享几何——切点
                    // T1/T2 + t = r/tan(φ/2) + 弧展平；旧 flatten_arc_to 线段近似
                    // 对远切点/变换场景形状不准，2d.path.arcTo.shape.*/scale/
                    // transformation 实证）。无子路径语义内含（moveTo 首控制点）。
                    //
                    // 角缩放 CTM（sx≠sy）：spec 弧是**用户空间的圆**经 CTM 变换
                    // （= 设备空间椭圆）——切线几何须在用户空间构造后逐点正变换
                    //（arcTo.scale 的 scale(0.1,1)：圆弧→扁椭圆弧）。控制点参数
                    // 在追加时已变换到设备空间，先逆变换回用户空间做圆弧。
                    subpath_has_geometry = true;
                    let had_subpath = has_any_subpath;
                    has_any_subpath = true;
                    let sx = self.transform.a;
                    let sy = self.transform.d;
                    let anisotropic = (sx - sy).abs() > 1e-6 * sx.abs().max(sy.abs()).max(1.0);
                    if anisotropic {
                        let inv = self.transform.inverse();
                        let (ux0, uy0) = inv.transform_point(current_x, current_y);
                        let (ux1, uy1) = inv.transform_point(x1, y1);
                        let (ux2, uy2) = inv.transform_point(x2, y2);
                        let start = vertices.len();
                        let (unx, uny) = crate::path::arc_to_tangent_segments(
                            &mut vertices,
                            ux0,
                            uy0,
                            ux1,
                            uy1,
                            ux2,
                            uy2,
                            radius,
                            had_subpath,
                        );
                        // 输出段逐点正变换。
                        for seg in vertices[start..].chunks_exact_mut(4) {
                            let (ax, ay) = self.transform.transform_point(seg[0], seg[1]);
                            let (bx, by) = self.transform.transform_point(seg[2], seg[3]);
                            seg[0] = ax;
                            seg[1] = ay;
                            seg[2] = bx;
                            seg[3] = by;
                        }
                        let (nx, ny) = self.transform.transform_point(unx, uny);
                        current_x = nx;
                        current_y = ny;
                    } else {
                        let (nx, ny) = crate::path::arc_to_tangent_segments(
                            &mut vertices,
                            current_x,
                            current_y,
                            x1,
                            y1,
                            x2,
                            y2,
                            radius,
                            had_subpath,
                        );
                        current_x = nx;
                        current_y = ny;
                    }
                }
                PathCommand::Ellipse(cx, cy, rx, ry, rotation, start_angle, end_angle) => {
                    subpath_has_geometry = true;
                    // R56h：非恒等 CTM 同 Arc 模式——ellipse() 只变换圆心，用户半径/
                    // 旋转/角度未经 CTM 缩放（scale(20,20) 的 rx=ry=5 椭圆在设备空间
                    // 应成 r=100——旧路径画 5px 小圆，2d.path.isPointInStroke.
                    // scaleddashes 的命中点距 100px 判负）。逆变换圆心 → 用户空间
                    // 构造椭圆 → 输出逐点正变换。
                    // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-ellipse
                    let t = &self.transform;
                    let identity_ctm = (t.a - 1.0).abs() < 1e-9
                        && (t.d - 1.0).abs() < 1e-9
                        && t.b.abs() < 1e-9
                        && t.c.abs() < 1e-9
                        && t.e.abs() < 1e-9
                        && t.f.abs() < 1e-9;
                    let cos_r = rotation.cos();
                    let sin_r = rotation.sin();
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let compute_point = |angle: f32| -> (f32, f32) {
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();
                        let px = rx * cos_a;
                        let py = ry * sin_a;
                        (cx + px * cos_r - py * sin_r, cy + px * sin_r + py * cos_r)
                    };
                    if !identity_ctm {
                        let inv = self.transform.inverse();
                        let (ucx, ucy) = inv.transform_point(cx, cy);
                        let u_point = |angle: f32| -> (f32, f32) {
                            let cos_a = angle.cos();
                            let sin_a = angle.sin();
                            let px = rx * cos_a;
                            let py = ry * sin_a;
                            (ucx + px * cos_r - py * sin_r, ucy + px * sin_r + py * cos_r)
                        };
                        let u_start = u_point(start_angle);
                        if has_any_subpath {
                            let (d_sx, d_sy) = self.transform.transform_point(u_start.0, u_start.1);
                            vertices.push(current_x);
                            vertices.push(current_y);
                            vertices.push(d_sx);
                            vertices.push(d_sy);
                        } else {
                            has_any_subpath = true;
                        }
                        let mut prev_dev = self.transform.transform_point(u_start.0, u_start.1);
                        for i in 0..ARC_SEGMENTS {
                            let u = u_point(start_angle + step * (i + 1) as f32);
                            let dev = self.transform.transform_point(u.0, u.1);
                            vertices.push(prev_dev.0);
                            vertices.push(prev_dev.1);
                            vertices.push(dev.0);
                            vertices.push(dev.1);
                            prev_dev = dev;
                        }
                        current_x = prev_dev.0;
                        current_y = prev_dev.1;
                        continue;
                    }
                    let (mut px, mut py) = compute_point(start_angle);
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let (nx, ny) = compute_point(angle);
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::RoundRect(x, y, w, h, ref radii) => {
                    subpath_has_geometry = true;
                    let (nx, ny) = Self::flatten_round_rect(&mut vertices, current_x, current_y, x, y, w, h, radii);
                    // R56：roundRect 自包含子路径（path.rs 的 round_rect 同时 push
                    // MoveTo）——隐式闭合须以子路径自身起点为基准，不受外部 MoveTo
                    // 的原始参数坐标干扰（负 w/h 归一化后起点 ≠ MoveTo 参数点，
                    // 隐式闭合会补出斜穿矩形的虚假边）。
                    subpath_start_x = nx;
                    subpath_start_y = ny;
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::ClosePath => {
                    // 从当前点画线回到子路径起点
                    if (current_x - subpath_start_x).abs() > f32::EPSILON
                        || (current_y - subpath_start_y).abs() > f32::EPSILON
                    {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(subpath_start_x);
                        vertices.push(subpath_start_y);
                    }
                    current_x = subpath_start_x;
                    current_y = subpath_start_y;
                    subpath_has_geometry = false; // 已显式闭合
                }
            }
        }
        // 命令流末尾的末个开放子路径隐式闭合（fill 的 closepath-on-fill 语义）。
        if close_open_subpaths
            && subpath_has_geometry
            && ((current_x - subpath_start_x).abs() > f32::EPSILON
                || (current_y - subpath_start_y).abs() > f32::EPSILON)
        {
            vertices.push(current_x);
            vertices.push(current_y);
            vertices.push(subpath_start_x);
            vertices.push(subpath_start_y);
        }
        // R56h：stroke 语义零长段剪除（spec trace-a-path——零长线段不参与描边；
        // round/butt 帽下退化子路径什么都不画，2d.path.stroke.prune.line/curve/arc）。
        // fill 保持原样（零长段对扫描线填充无影响）。
        if !close_open_subpaths {
            let mut read = 0usize;
            let mut write = 0usize;
            while read + 3 < vertices.len() {
                let (x0, y0, x1, y1) = (
                    vertices[read],
                    vertices[read + 1],
                    vertices[read + 2],
                    vertices[read + 3],
                );
                if (x1 - x0).abs() > f32::EPSILON || (y1 - y0).abs() > f32::EPSILON {
                    vertices[write] = x0;
                    vertices[write + 1] = y0;
                    vertices[write + 2] = x1;
                    vertices[write + 3] = y1;
                    write += 4;
                }
                read += 4;
            }
            vertices.truncate(write);
        }
        vertices
    }

    /// fill/clip/hit-test 语义的扁平化（隐式闭合开放子路径——closepath-on-fill）。
    pub(crate) fn flatten_path(&self) -> Vec<f32> {
        self.flatten_path_opts(true)
    }

    /// stroke 语义的扁平化（开放子路径保持开放）。
    pub(crate) fn flatten_path_open(&self) -> Vec<f32> {
        self.flatten_path_opts(false)
    }

    /// 将指定 Path2D 的命令扁平化为顶点列表（x, y 交替）。
    pub(crate) fn flatten_path_for(&self, path: &Path2D) -> Vec<f32> {
        let mut vertices = Vec::new();
        let mut current_x = 0.0f32;
        let mut current_y = 0.0f32;
        let mut subpath_start_x = 0.0f32;
        let mut subpath_start_y = 0.0f32;
        // R56e：子路径存在标志（arcTo 无子路径连线守卫用）。
        let mut has_any_subpath = false;
        const ARC_SEGMENTS: usize = 16;

        for cmd in path.commands() {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
                    has_any_subpath = true;
                    subpath_start_x = x;
                    subpath_start_y = y;
                    current_x = x;
                    current_y = y;
                }
                PathCommand::LineTo(x, y) => {
                    vertices.push(current_x);
                    vertices.push(current_y);
                    vertices.push(x);
                    vertices.push(y);
                    current_x = x;
                    current_y = y;
                }
                PathCommand::QuadraticCurveTo(cpx, cpy, x, y) => {
                    // R56h：二次转三次 + 自适应细分（同 flatten_path_opts）。
                    let (sx0, sy0) = (current_x, current_y);
                    let c1x = sx0 + (cpx - sx0) * 2.0 / 3.0;
                    let c1y = sy0 + (cpy - sy0) * 2.0 / 3.0;
                    let c2x = x + (cpx - x) * 2.0 / 3.0;
                    let c2y = y + (cpy - y) * 2.0 / 3.0;
                    crate::path::flatten_bezier_adaptive(&mut vertices, sx0, sy0, c1x, c1y, c2x, c2y, x, y, 0.1, 0);
                    current_x = x;
                    current_y = y;
                }
                PathCommand::BezierCurveTo(cp1x, cp1y, cp2x, cp2y, x, y) => {
                    // R56h：自适应细分（同 flatten_path_opts）。
                    crate::path::flatten_bezier_adaptive(
                        &mut vertices,
                        current_x,
                        current_y,
                        cp1x,
                        cp1y,
                        cp2x,
                        cp2y,
                        x,
                        y,
                        0.1,
                        0,
                    );
                    current_x = x;
                    current_y = y;
                }
                PathCommand::Arc(cx, cy, radius, start_angle, end_angle, anticlockwise) => {
                    // R34xx：anticlockwise 方向（同 flatten_path_for——此前忽略致 ctx.fill
                    // 路径的下半弧画成上半弧，2d.line.cap.round 胶囊 fill 失败）。
                    let dir = if anticlockwise { -1.0 } else { 1.0 };
                    let angle_span = (end_angle - start_angle).abs() * dir;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let mut px = cx + radius * start_angle.cos();
                    let mut py = cy + radius * start_angle.sin();
                    // R34xx：当前点 ≠ 弧起点时先 lineTo 弧起点（同 flatten_path_for）。
                    if (px - current_x).abs() > 1e-4 || (py - current_y).abs() > 1e-4 {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(px);
                        vertices.push(py);
                    }
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let nx = cx + radius * angle.cos();
                        let ny = cy + radius * angle.sin();
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::ArcTo(x1, y1, x2, y2, radius) => {
                    // R56f：同 flatten_path_opts——共享真切线弧几何。
                    let had_subpath = has_any_subpath;
                    has_any_subpath = true;
                    let (nx, ny) = crate::path::arc_to_tangent_segments(
                        &mut vertices,
                        current_x,
                        current_y,
                        x1,
                        y1,
                        x2,
                        y2,
                        radius,
                        had_subpath,
                    );
                    current_x = nx;
                    current_y = ny;
                }
                PathCommand::Ellipse(cx, cy, rx, ry, rotation, start_angle, end_angle) => {
                    let cos_r = rotation.cos();
                    let sin_r = rotation.sin();
                    let angle_span = end_angle - start_angle;
                    let step = angle_span / ARC_SEGMENTS as f32;
                    let compute_point = |angle: f32| -> (f32, f32) {
                        let cos_a = angle.cos();
                        let sin_a = angle.sin();
                        let px = rx * cos_a;
                        let py = ry * sin_a;
                        (cx + px * cos_r - py * sin_r, cy + px * sin_r + py * cos_r)
                    };
                    let (mut px, mut py) = compute_point(start_angle);
                    for i in 0..ARC_SEGMENTS {
                        let angle = start_angle + step * (i + 1) as f32;
                        let (nx, ny) = compute_point(angle);
                        vertices.push(px);
                        vertices.push(py);
                        vertices.push(nx);
                        vertices.push(ny);
                        px = nx;
                        py = ny;
                    }
                    current_x = px;
                    current_y = py;
                }
                PathCommand::RoundRect(x, y, w, h, ref radii) => {
                    let (nx, ny) = Self::flatten_round_rect(&mut vertices, current_x, current_y, x, y, w, h, radii);
                    current_x = nx;
                    current_y = ny;
                    // R56：同 flatten_path——子路径起点以 roundRect 自身边界为准。
                    subpath_start_x = nx;
                    subpath_start_y = ny;
                }
                PathCommand::ClosePath => {
                    if (current_x - subpath_start_x).abs() > f32::EPSILON
                        || (current_y - subpath_start_y).abs() > f32::EPSILON
                    {
                        vertices.push(current_x);
                        vertices.push(current_y);
                        vertices.push(subpath_start_x);
                        vertices.push(subpath_start_y);
                    }
                    current_x = subpath_start_x;
                    current_y = subpath_start_y;
                }
            }
        }
        vertices
    }

    /// 使用当前合成操作模式，将源颜色与目标像素进行合成。
    /// 返回合成后的 RGBA 值（0-255）。
    /// 参考 Porter-Duff alpha compositing 规范实现。
    pub(crate) fn composite_pixel(&self, src: Color, dst_r: u8, dst_g: u8, dst_b: u8, dst_a: u8) -> (u8, u8, u8, u8) {
        let sa = src.a as f32 / 255.0;
        let da = dst_a as f32 / 255.0;
        let sr = src.r as f32 / 255.0;
        let sg = src.g as f32 / 255.0;
        let sb = src.b as f32 / 255.0;
        let dr = dst_r as f32 / 255.0;
        let dg = dst_g as f32 / 255.0;
        let db = dst_b as f32 / 255.0;

        // Porter-Duff 合成因子 (Fa, Fb)
        let (fa, fb) = match self.composite_operation {
            CompositeOperation::SourceOver => (1.0, 1.0 - sa),
            CompositeOperation::DestinationOver => (1.0 - da, 1.0),
            CompositeOperation::SourceIn => (da, 0.0),
            CompositeOperation::DestinationIn => (0.0, sa),
            CompositeOperation::DestinationOut => (0.0, 1.0 - sa),
            // R34xx：标准 Porter-Duff source-out = (1-da, 0)（源画在目标外）——旧实现误用
            // destination-out 的 (0, 1-sa)（2d.composite.transparent.source-out 期望
            // 半透明目标的形状内输出源）。
            CompositeOperation::SourceOut => (1.0 - da, 0.0),
            CompositeOperation::SourceAtop => (da, 1.0 - sa),
            CompositeOperation::DestinationAtop => (1.0 - da, sa),
            CompositeOperation::Copy => (1.0, 0.0),
            CompositeOperation::Xor => (1.0 - da, 1.0 - sa),
            CompositeOperation::Lighter => (1.0, 1.0),
            CompositeOperation::Clear => (0.0, 0.0),
            // 其余混合模式使用 source-over 的合成因子
            _ => (1.0, 1.0 - sa),
        };

        // R3237：out_a 须先 clamp 到 [0,1] 再作 un-premultiply 除数——Lighter（plus）下
        // out_a = sa+da 可达 2.0，旧实现除以 2 致颜色减半（红+绿得 [128,128,0] 非 spec [255,255,0]）。
        // Porter-Duff 模式 out_a ≤ 1，clamp 为 no-op；destination-out 全擦除 out_a=0 不受影响。
        let out_a = (sa * fa + da * fb).clamp(0.0, 1.0);
        if out_a <= 0.0 {
            return (0, 0, 0, 0);
        }
        // R3239：blend 模式（§9 separable + §10 non-separable）用 blended source Cs' 替代 Cs；
        // Porter-Duff 模式 Cs' = Cs（blend_source_color 原样返回）。
        let cs = blend_source_color(self.composite_operation, da, [sr, sg, sb], [dr, dg, db]);
        let out_r = (cs[0] * sa * fa + dr * da * fb) / out_a;
        let out_g = (cs[1] * sa * fa + dg * da * fb) / out_a;
        let out_b = (cs[2] * sa * fa + db * da * fb) / out_a;

        (
            (out_r * 255.0).round().clamp(0.0, 255.0) as u8,
            (out_g * 255.0).round().clamp(0.0, 255.0) as u8,
            (out_b * 255.0).round().clamp(0.0, 255.0) as u8,
            (out_a * 255.0).round().clamp(0.0, 255.0) as u8,
        )
    }

    /// R3240：把（已 blur 的）shadow alpha mask 经当前 composite_operation 合成到 pixel_buffer。
    /// mask 为 region `(rx,ry,rw,rh)`（canvas 坐标）局部；`(off_x,off_y)` 整体偏移阴影；
    /// 每像素源色 alpha = `shadow_color.a · coverage · global_alpha`。
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn composite_shadow_mask(
        &mut self,
        mask: &[u8],
        rx: i32,
        ry: i32,
        rw: usize,
        rh: usize,
        off_x: f32,
        off_y: f32,
        color: Color,
        global_alpha: f32,
        shape_alpha: f32,
    ) {
        let cw = self.width as i32;
        let ch = self.height as i32;
        let dox = off_x.round() as i32;
        let doy = off_y.round() as i32;
        let base_a = color.a as f32 / 255.0;
        for ly in 0..rh as i32 {
            for lx in 0..rw as i32 {
                let coverage = mask[(ly as usize) * rw + (lx as usize)] as f32 / 255.0;
                if coverage <= 0.0 {
                    continue;
                }
                let cx = rx + lx + dox;
                let cy = ry + ly + doy;
                if cx < 0 || cy < 0 || cx >= cw || cy >= ch {
                    continue;
                }
                // R34xx：阴影受 clip 区域裁剪（2d.shadow.clip.2——clip 外不画阴影）。
                if !self.clip_applies(cx as f32, cy as f32) {
                    continue;
                }
                let alpha = (base_a * coverage * global_alpha * shape_alpha).clamp(0.0, 1.0);
                if alpha <= 0.0 {
                    continue;
                }
                let src = Color::rgba(color.r, color.g, color.b, (alpha * 255.0).round() as u8);
                let idx = ((cy as usize) * (self.width as usize) + (cx as usize)) * 4;
                let (pr, pg, pb, pa) = self.composite_pixel(
                    src,
                    self.pixel_buffer[idx],
                    self.pixel_buffer[idx + 1],
                    self.pixel_buffer[idx + 2],
                    self.pixel_buffer[idx + 3],
                );
                self.pixel_buffer[idx] = pr;
                self.pixel_buffer[idx + 1] = pg;
                self.pixel_buffer[idx + 2] = pb;
                self.pixel_buffer[idx + 3] = pa;
            }
        }
    }

    /// R34xx：设备空间半线宽——lineWidth 是画布坐标单位，描边几何经 CTM 变换到设备
    /// 空间后，垂向偏移按 |T·n̂_c| 缩放（n̂_c 为画布空间法线）。旧实现用画布线宽直接光栅，
    /// scale 变换下的线宽失真（2d.line.width.scaledefault/transformed：scale(50,50) 的
    /// lineWidth 1 视觉应为 50px）。
    pub(crate) fn device_half_width(&self, ux_dev: f32, uy_dev: f32, half_lw: f32) -> f32 {
        let t = &self.transform;
        let det = t.a * t.d - t.b * t.c;
        if det.abs() < f32::EPSILON {
            return half_lw; // 退化变换（scale 0）：保持原值，由段矩形空区域自然消隐
        }
        // 画布空间线段方向 = T⁻¹ · u_dev（线性部分）
        let (i_a, i_b, i_c, i_d) = (t.d / det, -t.b / det, -t.c / det, t.a / det);
        let (ux_c, uy_c) = (i_a * ux_dev + i_c * uy_dev, i_b * ux_dev + i_d * uy_dev);
        let len_c = (ux_c * ux_c + uy_c * uy_c).sqrt();
        if len_c < f32::EPSILON {
            return half_lw;
        }
        // 画布空间单位法线 → 设备偏移向量 = T · n̂_c
        let (nx_c, ny_c) = (-uy_c / len_c, ux_c / len_c);
        let (ox, oy) = (t.a * nx_c + t.c * ny_c, t.b * nx_c + t.d * ny_c);
        half_lw * (ox * ox + oy * oy).sqrt()
    }

    /// R34xx：join 点是否产生可见连接。两条相邻段共线（180°/0° 角，含零长段）时
    /// Miter/Bevel 无可见延伸（上游 2d.strokeRect.zero.4：Nx0 闭合线端点 miter 不覆盖
    /// 端点外区域）；Round 总是画（圆连接，zero.5 lineJoin=round 期望端点外覆盖）。
    pub(crate) fn join_visible(&self, seg_a: &[f32; 4], seg_b: &[f32; 4]) -> bool {
        self.join_shape_visible(seg_a, seg_b).is_some()
    }

    /// join 可见性 + 角向（R56）：返回 None = 不画；Some(convex) = 画（convex = 凸角）。
    /// 凹角（reflex——路径右转，join 平切三角落在两段主体已覆盖区内再外溢到外角区，
    /// 2d.line.join.bevel 的 (84,16) 凹角外不得涂）Miter/Bevel 不画；Round 仍画（圆盘
    /// 与凸角同径，凹角侧圆盘被段主体覆盖语义不变，保持 R34xx zero.5 行为）。
    pub(crate) fn join_shape_visible(&self, seg_a: &[f32; 4], seg_b: &[f32; 4]) -> Option<bool> {
        match self.line_join {
            LineJoin::Round => Some(true),
            LineJoin::Miter | LineJoin::Bevel => {
                let (ax, ay) = (seg_a[2] - seg_a[0], seg_a[3] - seg_a[1]);
                let (bx, by) = (seg_b[2] - seg_b[0], seg_b[3] - seg_b[1]);
                let la = (ax * ax + ay * ay).sqrt();
                let lb = (bx * bx + by * by).sqrt();
                if la < f32::EPSILON || lb < f32::EPSILON {
                    return None; // 任一侧零长段 → 无可见连接
                }
                let dot = ax * bx + ay * by;
                if dot.abs() >= la * lb * 0.9999 {
                    return None; // 共线 → 无角
                }
                // 叉积符号 = 转向：屏幕 y 向下，ax*by - ay*bx < 0 为左转（凸角，
                // join 尖在外侧须画）；> 0 为右转（凹角，平切三角落在段主体已覆盖区
                // 且外溢外角区——2d.line.join.bevel 的 (84,16) 凹角外不得涂）。
                Some(ax * by - ay * bx < 0.0)
            }
        }
    }

    /// R34xx：按 lineJoin 真实几何绘制 join 点。旧实现三种 join 均画 half_lw 方块——
    /// 方块覆盖角点外大片区域（2d.line.cap.closed / join.open / miter.* 角落 (1,1) 失败），
    /// 且 miter 忽略 miter_limit（2d.line.miter.exceeded）。miter 尖角三角 / bevel 平切
    /// 三角经 blit_path_to_pixels 通用扫描线填充；round 画圆盘。
    pub(crate) fn blit_join(
        &mut self,
        seg_a: &[f32; 4],
        seg_b: &[f32; 4],
        jx: f32,
        jy: f32,
        half_lw: f32,
        color: Color,
    ) {
        // 两边方向（单位向量）：seg_a 指向 join 点，seg_b 从 join 点出发。
        let (dax, day) = (jx - seg_a[0], jy - seg_a[1]);
        let (dbx, dby) = (seg_b[2] - jx, seg_b[3] - jy);
        let la = (dax * dax + day * day).sqrt();
        let lb = (dbx * dbx + dby * dby).sqrt();
        if la < f32::EPSILON || lb < f32::EPSILON {
            return;
        }
        let (uax, uay) = (dax / la, day / la);
        let (ubx, uby) = (dbx / lb, dby / lb);
        // R34xx：外扩点用法线方向（rot90(u) = (-u.y, u.x)），并选择**角内侧**侧——尖点
        // 方向 m = u1 - u2（外角平分），外扩点选 (ext - jx)·m > 0 的一侧。旧实现固定
        // rot90 一侧，90° 角时外扩点落在角外侧，三角不覆盖角内侧（join.miter (38,12)
        // 等组合几何失败）。
        let (mx, my) = (uax - ubx, uay - uby);
        let ml = (mx * mx + my * my).sqrt();
        if ml < f32::EPSILON {
            return;
        }
        let (dx, dy) = (mx / ml, my / ml);
        let (a_ext_x, a_ext_y) = if mx * -uay + my * uax > 0.0 {
            (jx - uay * half_lw, jy + uax * half_lw)
        } else {
            (jx + uay * half_lw, jy - uax * half_lw)
        };
        let (b_ext_x, b_ext_y) = if mx * -uby + my * ubx > 0.0 {
            (jx - uby * half_lw, jy + ubx * half_lw)
        } else {
            (jx + uby * half_lw, jy - ubx * half_lw)
        };
        match self.line_join {
            LineJoin::Round => {
                let r2 = half_lw * half_lw;
                let x0 = (jx - half_lw).floor() as i32;
                let y0 = (jy - half_lw).floor() as i32;
                let x1 = (jx + half_lw).ceil() as i32;
                let y1 = (jy + half_lw).ceil() as i32;
                for py in y0..y1 {
                    for px in x0..x1 {
                        let dx = px as f32 + 0.5 - jx;
                        let dy = py as f32 + 0.5 - jy;
                        if dx * dx + dy * dy <= r2 {
                            self.blit_pixel(px, py, color);
                        }
                    }
                }
            }
            LineJoin::Bevel => {
                // R56：段格式三边闭合三角（j→a / a→b / b→j）——blit_path_to_pixels 消费独立段序列
                //（旧顶点链 8 floats 只 2 段缺 a→b 边，扫描线配对在部分行破裂）。
                let verts = [
                    jx, jy, a_ext_x, a_ext_y, a_ext_x, a_ext_y, b_ext_x, b_ext_y, b_ext_x, b_ext_y, jx, jy,
                ];
                self.blit_path_to_pixels(&verts, color);
            }
            LineJoin::Miter => {
                // R34xx：miter 尖点。方向 = 外角平分（u1 - u2 归一化——u1 指向 join、u2 离开
                // join；对 90° 角尖点在对角线方向）。miter 长度 = half_lw / sin(θ/2)，θ 取
                // 两线几何锐角（cos θ = |u1·u2|）——2d.line.miter.acute 的 45° 角期望
                // ratio = 1/sin(22.5°) = 2.6139 恰在 miterLimit 2.613/2.614 边界。
                // 超限判定 spec：ratio = miter_len / half_lw = 1/sin(θ/2) > miterLimit → bevel。
                let cos_theta = -(uax * ubx + uay * uby);
                let sin_half = ((1.0 - cos_theta) / 2.0).sqrt();
                if sin_half < f32::EPSILON {
                    return;
                }
                let miter_len = half_lw / sin_half;
                if miter_len / half_lw > self.miter_limit {
                    // 超 miter limit → 降级 bevel 平切
                    // R56：段格式三边闭合三角（j→a / a→b / b→j）——blit_path_to_pixels 消费独立段序列
                    //（旧顶点链 8 floats 只 2 段缺 a→b 边，扫描线配对在部分行破裂）。
                    let verts = [
                        jx, jy, a_ext_x, a_ext_y, a_ext_x, a_ext_y, b_ext_x, b_ext_y, b_ext_x, b_ext_y, jx, jy,
                    ];
                    self.blit_path_to_pixels(&verts, color);
                } else {
                    let (px, py) = (jx + dx * miter_len, jy + dy * miter_len);
                    // 扫掠四边形轮廓：{jx, a_ext, P, b_ext}
                    // R56：段格式四边闭合四边形（j→a→P→b→j）。
                    let verts = [
                        jx, jy, a_ext_x, a_ext_y, a_ext_x, a_ext_y, px, py, px, py, b_ext_x, b_ext_y, b_ext_x, b_ext_y,
                        jx, jy,
                    ];
                    self.blit_path_to_pixels(&verts, color);
                }
            }
        }
    }

    /// R34xx：单像素合成写入（round cap/join 圆盘循环用），含画布边界与 clip 检查。
    /// R34xx：把字体灰度位图（每字节一像素的 coverage）alpha 混合进 pixel_buffer——
    /// 每像素源色 = 绘制色 × coverage（2d.text.draw.* 真文本光栅；clip/composite 同其他 blit）。
    pub(crate) fn blit_glyph_bitmap(
        &mut self,
        bmp: &zero_render_foundation::font::GlyphBitmap,
        gx: f32,
        gy: f32,
        color: Color,
    ) {
        // R34xx（filters 渲染）：colorMatrix 滤镜作用于字形色。
        let color = self.apply_filter_color(color);
        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        let (ix, iy) = (gx.floor() as i32, gy.floor() as i32);
        for by in 0..bmp.height as i32 {
            for bx in 0..bmp.width as i32 {
                let coverage = bmp.data[(by as usize) * bmp.width as usize + (bx as usize)];
                if coverage == 0 {
                    continue;
                }
                let (px, py) = (ix + bx, iy + by);
                if px < 0 || py < 0 || px as usize >= canvas_w || py as usize >= canvas_h {
                    continue;
                }
                if !self.clip_applies(px as f32, py as f32) {
                    continue;
                }
                let idx = (py as usize * canvas_w + px as usize) * 4;
                let src = Color {
                    r: color.r,
                    g: color.g,
                    b: color.b,
                    a: ((color.a as f32 * coverage as f32 / 255.0) * self.global_alpha) as u8,
                };
                let (pr, pg, pb, pa) = self.composite_pixel(
                    src,
                    self.pixel_buffer[idx],
                    self.pixel_buffer[idx + 1],
                    self.pixel_buffer[idx + 2],
                    self.pixel_buffer[idx + 3],
                );
                self.pixel_buffer[idx] = pr;
                self.pixel_buffer[idx + 1] = pg;
                self.pixel_buffer[idx + 2] = pb;
                self.pixel_buffer[idx + 3] = pa;
            }
        }
    }

    pub(crate) fn blit_pixel(&mut self, px: i32, py: i32, color: Color) {
        if px < 0 || py < 0 || px >= self.width as i32 || py >= self.height as i32 {
            return;
        }
        if !self.clip_applies(px as f32, py as f32) {
            return;
        }
        // R34xx：stroke 去重——同一次 stroke 调用内已覆盖像素跳过（段/join/cap 重叠区
        // 只合成一次）。
        if let Some(mask) = self.stroke_dedup_mask.as_mut() {
            let mi = (py as usize) * (self.width as usize) + px as usize;
            if mask[mi] != 0 {
                return;
            }
            mask[mi] = 1;
        }
        let idx = ((py as usize) * (self.width as usize) + px as usize) * 4;
        let (r, g, b, a) = self.composite_pixel(
            color,
            self.pixel_buffer[idx],
            self.pixel_buffer[idx + 1],
            self.pixel_buffer[idx + 2],
            self.pixel_buffer[idx + 3],
        );
        self.pixel_buffer[idx] = r;
        self.pixel_buffer[idx + 1] = g;
        self.pixel_buffer[idx + 2] = b;
        self.pixel_buffer[idx + 3] = a;
    }

    /// R34xx：blit_join 的渐变采样版（join 三角经 blit_path_gradient 逐像素采样；round 圆盘
    /// 经 blit_pixel_gradient）。
    pub(crate) fn blit_join_gradient(
        &mut self,
        seg_a: &[f32; 4],
        seg_b: &[f32; 4],
        jx: f32,
        jy: f32,
        half_lw: f32,
        style: &CanvasStyle,
    ) {
        let (dax, day) = (jx - seg_a[0], jy - seg_a[1]);
        let (dbx, dby) = (seg_b[2] - jx, seg_b[3] - jy);
        let la = (dax * dax + day * day).sqrt();
        let lb = (dbx * dbx + dby * dby).sqrt();
        if la < f32::EPSILON || lb < f32::EPSILON {
            return;
        }
        let (uax, uay) = (dax / la, day / la);
        let (ubx, uby) = (dbx / lb, dby / lb);
        // R34xx：外扩点用法线方向并按角内侧选侧（同 blit_join）。
        let (mx, my) = (uax - ubx, uay - uby);
        let ml = (mx * mx + my * my).sqrt();
        if ml < f32::EPSILON {
            return;
        }
        let (dx, dy) = (mx / ml, my / ml);
        let (a_ext_x, a_ext_y) = if mx * -uay + my * uax > 0.0 {
            (jx - uay * half_lw, jy + uax * half_lw)
        } else {
            (jx + uay * half_lw, jy - uax * half_lw)
        };
        let (b_ext_x, b_ext_y) = if mx * -uby + my * ubx > 0.0 {
            (jx - uby * half_lw, jy + ubx * half_lw)
        } else {
            (jx + uby * half_lw, jy - ubx * half_lw)
        };
        match self.line_join {
            LineJoin::Round => {
                let r2 = half_lw * half_lw;
                let x0 = (jx - half_lw).floor() as i32;
                let y0 = (jy - half_lw).floor() as i32;
                let x1 = (jx + half_lw).ceil() as i32;
                let y1 = (jy + half_lw).ceil() as i32;
                for py in y0..y1 {
                    for px in x0..x1 {
                        let dx = px as f32 + 0.5 - jx;
                        let dy = py as f32 + 0.5 - jy;
                        if dx * dx + dy * dy <= r2 {
                            self.blit_pixel_gradient(px, py, style);
                        }
                    }
                }
            }
            LineJoin::Bevel => {
                // R56：段格式三边闭合三角（j→a / a→b / b→j）——blit_path_to_pixels 消费独立段序列
                //（旧顶点链 8 floats 只 2 段缺 a→b 边，扫描线配对在部分行破裂）。
                let verts = [
                    jx, jy, a_ext_x, a_ext_y, a_ext_x, a_ext_y, b_ext_x, b_ext_y, b_ext_x, b_ext_y, jx, jy,
                ];
                self.blit_path_gradient(&verts, style);
            }
            LineJoin::Miter => {
                // R34xx：同 blit_join 的 miter 几何（外角平分尖点 + 锐角 θ + spec ratio 判定）。
                let cos_theta = -(uax * ubx + uay * uby);
                let sin_half = ((1.0 - cos_theta) / 2.0).sqrt();
                if sin_half < f32::EPSILON {
                    return;
                }
                let miter_len = half_lw / sin_half;
                if miter_len / half_lw > self.miter_limit {
                    // R56：段格式三边闭合三角（j→a / a→b / b→j）——blit_path_to_pixels 消费独立段序列
                    //（旧顶点链 8 floats 只 2 段缺 a→b 边，扫描线配对在部分行破裂）。
                    let verts = [
                        jx, jy, a_ext_x, a_ext_y, a_ext_x, a_ext_y, b_ext_x, b_ext_y, b_ext_x, b_ext_y, jx, jy,
                    ];
                    self.blit_path_gradient(&verts, style);
                } else {
                    let (px, py) = (jx + dx * miter_len, jy + dy * miter_len);
                    // R56：段格式四边闭合四边形（j→a→P→b→j）。
                    let verts = [
                        jx, jy, a_ext_x, a_ext_y, a_ext_x, a_ext_y, px, py, px, py, b_ext_x, b_ext_y, b_ext_x, b_ext_y,
                        jx, jy,
                    ];
                    self.blit_path_gradient(&verts, style);
                }
            }
        }
    }

    /// R34xx：单像素渐变采样写入（round cap/join 圆盘循环用）。
    pub(crate) fn blit_pixel_gradient(&mut self, px: i32, py: i32, style: &CanvasStyle) {
        if px < 0 || py < 0 || px >= self.width as i32 || py >= self.height as i32 {
            return;
        }
        if !self.clip_applies(px as f32, py as f32) {
            return;
        }
        // R34xx：stroke 去重（同 blit_pixel——渐变 stroke 的段/join/cap 重叠只合成一次）。
        if let Some(mask) = self.stroke_dedup_mask.as_mut() {
            let mi = (py as usize) * (self.width as usize) + px as usize;
            if mask[mi] != 0 {
                return;
            }
            mask[mi] = 1;
        }
        let color = self.apply_alpha(style.sample_at(px as f32, py as f32));
        let idx = ((py as usize) * (self.width as usize) + px as usize) * 4;
        let (r, g, b, a) = self.composite_pixel(
            color,
            self.pixel_buffer[idx],
            self.pixel_buffer[idx + 1],
            self.pixel_buffer[idx + 2],
            self.pixel_buffer[idx + 3],
        );
        self.pixel_buffer[idx] = r;
        self.pixel_buffer[idx + 1] = g;
        self.pixel_buffer[idx + 2] = b;
        self.pixel_buffer[idx + 3] = a;
    }

    /// 将矩形区域的颜色写入像素缓冲区（光栅化填充），应用当前合成操作模式。
    pub(crate) fn blit_rect_to_pixels(&mut self, rect: &Rect, color: Color) {
        self.blit_rect_to_pixels_checked(rect, color, None)
    }

    /// R57（M3）：带原始矩形逆变换判定的矩形填充——非轴对齐 CTM 下 `transform_rect`
    /// 的包围盒含旋转矩形角外三角区（composite.grid 的 rotate(90°)+scale 变换后
    /// fillRect 边界差 ~8px——oracle A/B 24%）。orig = 未变换矩形 (x,y,w,h)；
    /// 轴对齐 CTM 下 `pixel_in_transformed_rect` 恒 true（零回归）。
    pub(crate) fn blit_rect_to_pixels_checked(
        &mut self,
        rect: &Rect,
        color: Color,
        orig: Option<(f32, f32, f32, f32)>,
    ) {
        // R34xx（filters 渲染）：colorMatrix 滤镜作用于源色。
        let color = self.apply_filter_color(color);
        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        let x_start = rect.left().max(0.0) as usize;
        let y_start = rect.top().max(0.0) as usize;
        // R34xx：上界用 ceil——亚像素矩形（如线宽 1 → 半宽 0.5 的描边矩形）bottom 截断会
        // 漏掉相交像素行（10.5 as usize = 10 → y ∈ [10,10) 空循环）。
        let x_end = (rect.right().min(self.width as f32).ceil() as usize).min(canvas_w);
        let y_end = (rect.bottom().min(self.height as f32).ceil() as usize).min(canvas_h);
        for y in y_start..y_end {
            for x in x_start..x_end {
                // R57：非轴对齐 CTM 下排除包围盒角外区域 + 边界覆盖率混合
                //（4×4 超采样——旋转矩形边 AA，ref Chromium 半色调）。
                let coverage = match orig {
                    Some((ox, oy, ow, oh)) => self.rect_coverage(x as f32, y as f32, ox, oy, ow, oh),
                    None => 1.0,
                };
                if coverage <= 0.0 {
                    continue;
                }
                // R34xx：clip 区域裁剪（clip_path 未设时零开销）。
                if !self.clip_applies(x as f32, y as f32) {
                    continue;
                }
                let idx = (y * canvas_w + x) * 4;
                let (r, g, b, a) = if coverage >= 1.0 {
                    self.composite_pixel(
                        color,
                        self.pixel_buffer[idx],
                        self.pixel_buffer[idx + 1],
                        self.pixel_buffer[idx + 2],
                        self.pixel_buffer[idx + 3],
                    )
                } else {
                    // 边界像素：源 alpha × coverage（source-over 近似；其余合成模式
                    // 亦乘 coverage——AA 边的主要场景为 source-over）。
                    let mut c = color;
                    c.a = (c.a as f32 * coverage).round() as u8;
                    self.composite_pixel(
                        c,
                        self.pixel_buffer[idx],
                        self.pixel_buffer[idx + 1],
                        self.pixel_buffer[idx + 2],
                        self.pixel_buffer[idx + 3],
                    )
                };
                self.pixel_buffer[idx] = r;
                self.pixel_buffer[idx + 1] = g;
                self.pixel_buffer[idx + 2] = b;
                self.pixel_buffer[idx + 3] = a;
                // R34xx（f16 存储）：float16 画布并行写 f32 缓冲（精确浮点颜色）。
                self.blit_pixel_f32(idx, color);
            }
        }
    }

    /// R34xx（f16 存储）：float16 画布的 f32 并行像素写（color 的精确浮点——
    /// fill_color_f32 优先，否则 u8/255）。
    pub(crate) fn blit_pixel_f32(&mut self, idx: usize, color: Color) {
        if self.pixel_buffer_f32.is_empty() {
            return;
        }
        let [r, g, b, a] = self.fill_color_f32.unwrap_or([
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            color.a as f32 / 255.0,
        ]);
        self.pixel_buffer_f32[idx] = r;
        self.pixel_buffer_f32[idx + 1] = g;
        self.pixel_buffer_f32[idx + 2] = b;
        self.pixel_buffer_f32[idx + 3] = a;
    }

    /// 矩形渐变填充：每像素按设备坐标采样样式颜色，应用 global_alpha + 当前合成操作。
    /// 与 `blit_rect_to_pixels` 对偶，供渐变样式（linear/radial/conic）的 `fill_rect` 路径使用。
    pub(crate) fn blit_rect_gradient(&mut self, rect: &Rect, style: &CanvasStyle) {
        self.blit_rect_gradient_checked(rect, style, None)
    }

    /// R57（M3）：渐变矩形填充的逆变换判定版（同 blit_rect_to_pixels_checked）。
    pub(crate) fn blit_rect_gradient_checked(
        &mut self,
        rect: &Rect,
        style: &CanvasStyle,
        orig: Option<(f32, f32, f32, f32)>,
    ) {
        let canvas_w = self.width as usize;
        let canvas_h = self.height as usize;
        let x_start = rect.left().max(0.0) as usize;
        let y_start = rect.top().max(0.0) as usize;
        // R34xx：上界用 ceil——亚像素矩形（如线宽 1 → 半宽 0.5 的描边矩形）bottom 截断会
        // 漏掉相交像素行（10.5 as usize = 10 → y ∈ [10,10) 空循环）。
        let x_end = (rect.right().min(self.width as f32).ceil() as usize).min(canvas_w);
        let y_end = (rect.bottom().min(self.height as f32).ceil() as usize).min(canvas_h);
        for y in y_start..y_end {
            for x in x_start..x_end {
                // R57：非轴对齐 CTM 下排除包围盒角外区域 + 边界覆盖率混合。
                let coverage = match orig {
                    Some((ox, oy, ow, oh)) => self.rect_coverage(x as f32, y as f32, ox, oy, ow, oh),
                    None => 1.0,
                };
                if coverage <= 0.0 {
                    continue;
                }
                // R34xx：clip 区域裁剪（clip_path 未设时零开销）。
                if !self.clip_applies(x as f32, y as f32) {
                    continue;
                }
                let mut color = self.apply_alpha(style.sample_at(x as f32, y as f32));
                if coverage < 1.0 {
                    color.a = (color.a as f32 * coverage).round() as u8;
                }
                let idx = (y * canvas_w + x) * 4;
                let (r, g, b, a) = self.composite_pixel(
                    color,
                    self.pixel_buffer[idx],
                    self.pixel_buffer[idx + 1],
                    self.pixel_buffer[idx + 2],
                    self.pixel_buffer[idx + 3],
                );
                self.pixel_buffer[idx] = r;
                self.pixel_buffer[idx + 1] = g;
                self.pixel_buffer[idx + 2] = b;
                self.pixel_buffer[idx + 3] = a;
            }
        }
    }

    /// 将路径填充写入像素缓冲区（扫描线光栅化）。
    pub(crate) fn blit_path_to_pixels(&mut self, vertices: &[f32], color: Color) {
        // R34xx（filters 渲染）：colorMatrix 滤镜作用于源色。
        let color = self.apply_filter_color(color);
        self.blit_path_to_pixels_rule(vertices, color, FillRule::NonZero)
    }

    /// R56c：带填充规则的路径填充（fill("evenodd") 透传；默认封装保持 NonZero）。
    pub(crate) fn blit_path_to_pixels_rule(&mut self, vertices: &[f32], color: Color, rule: FillRule) {
        if vertices.len() < 4 {
            return;
        }
        // 找出包围盒
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        for chunk in vertices.chunks_exact(2) {
            min_x = min_x.min(chunk[0]);
            min_y = min_y.min(chunk[1]);
            max_x = max_x.max(chunk[0]);
            max_y = max_y.max(chunk[1]);
        }
        // 收集所有唯一顶点用于扫描线
        let mut points: Vec<(f32, f32)> = Vec::new();
        for chunk in vertices.chunks_exact(2) {
            points.push((chunk[0], chunk[1]));
        }
        let canvas_w = self.width;
        let canvas_h = self.height;
        let y_start = min_y.max(0.0).ceil() as u32;
        let y_end = max_y.min(canvas_h as f32).ceil() as u32;
        // R57（M3）：非轴对齐 CTM 下 span 边界像素 AA（4×4 超采样点内测试——
        // 旋转路径边半色调，与 rect_coverage 同模式；轴对齐恒硬边零回归）。
        let rotated = self.transform.b.abs() > 1e-6 || self.transform.c.abs() > 1e-6;

        for scan_y in y_start..y_end {
            let sy = scan_y as f32 + 0.5;
            // R56c（M8/DC-8）：nonzero fill rule（spec dom-context-2d-fill 默认）——
            // 交点带方向符号（段向下 = +1 / 向上 = −1，屏幕 y 向下），按 x 排序后
            // 累计绕组，非零区间填充。旧偶奇配对（排序后两两成对）对嵌套同向
            // 子路径（winding.add：外矩形+同向内矩形，中心绕组 ±2）和对角连线
            // 杂散交点都会配对破裂（挖出假洞/漏填）。
            // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-fill
            for (sx, ex) in fill_rule_spans(vertices, sy, rule) {
                let ix_start_f = sx.max(0.0);
                let ix_end_f = ex.min(canvas_w as f32);
                let full_start = ix_start_f.ceil() as u32;
                let full_end = ix_end_f.floor() as u32;
                for scan_x in full_start..full_end {
                    // R34xx：clip 区域裁剪（clip_path 未设时零开销）。
                    if !self.clip_applies(scan_x as f32, scan_y as f32) {
                        continue;
                    }
                    let idx = ((scan_y * canvas_w + scan_x) * 4) as usize;
                    if idx + 3 < self.pixel_buffer.len() {
                        // R3236：路径填充消费 globalCompositeOperation（与 blit_rect_to_pixels 一致）——
                        // 旧实现覆盖写，致 destination-out/lighter/copy 等经 fill() 的路径填充失效。
                        let (r, g, b, a) = self.composite_pixel(
                            color,
                            self.pixel_buffer[idx],
                            self.pixel_buffer[idx + 1],
                            self.pixel_buffer[idx + 2],
                            self.pixel_buffer[idx + 3],
                        );
                        self.pixel_buffer[idx] = r;
                        self.pixel_buffer[idx + 1] = g;
                        self.pixel_buffer[idx + 2] = b;
                        self.pixel_buffer[idx + 3] = a;
                        // R34xx（f16 存储）：float16 画布并行 f32 写。
                        self.blit_pixel_f32(idx, color);
                    }
                }
                // R57（M3）：span 边界像素 AA——起点/终点非整数时所在像素部分覆盖
                //（sx<0 时画布左缘像素 0 兜底——coverage 超采样自动判定；整数边界
                // 硬边，与 Chromium 整数坐标矩形一致）。
                if rotated {
                    if sx - sx.floor() > 1e-6 {
                        let b = sx.floor().max(0.0) as u32;
                        if b < canvas_w {
                            let cov = self.path_pixel_coverage(vertices, b as f32, scan_y as f32, rule);
                            if cov > 0.0 && cov < 1.0 {
                                self.blit_path_edge_pixel(b, scan_y, color, cov);
                            }
                        }
                    }
                    if ex - ex.floor() > 1e-6 {
                        let b = ex.floor().max(0.0) as u32;
                        if b < canvas_w {
                            let cov = self.path_pixel_coverage(vertices, b as f32, scan_y as f32, rule);
                            if cov > 0.0 && cov < 1.0 {
                                self.blit_path_edge_pixel(b, scan_y, color, cov);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 路径渐变填充：扫描线光栅化，每像素按设备坐标采样样式颜色，应用 global_alpha + 当前合成操作
    ///（与 `blit_path_to_pixels` 同语义——R3236 起消费合成操作）。供渐变样式的 `fill` 路径使用。
    pub(crate) fn blit_path_gradient(&mut self, vertices: &[f32], style: &CanvasStyle) {
        self.blit_path_gradient_rule(vertices, style, FillRule::NonZero)
    }

    /// R56c：带填充规则的渐变填充（fill("evenodd") 透传）。
    pub(crate) fn blit_path_gradient_rule(&mut self, vertices: &[f32], style: &CanvasStyle, rule: FillRule) {
        if vertices.len() < 4 {
            return;
        }
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut points: Vec<(f32, f32)> = Vec::new();
        for chunk in vertices.chunks_exact(2) {
            let (px, py) = (chunk[0], chunk[1]);
            min_x = min_x.min(px);
            min_y = min_y.min(py);
            max_x = max_x.max(px);
            max_y = max_y.max(py);
            points.push((px, py));
        }
        let canvas_w = self.width;
        let canvas_h = self.height;
        let y_start = min_y.max(0.0).ceil() as u32;
        let y_end = max_y.min(canvas_h as f32).ceil() as u32;
        // R57（M3）：非轴对齐 CTM 下 span 边界像素 AA（与 blit_path_to_pixels_rule 同模式）。
        let rotated = self.transform.b.abs() > 1e-6 || self.transform.c.abs() > 1e-6;
        for scan_y in y_start..y_end {
            let sy = scan_y as f32 + 0.5;
            // R56c（M8/DC-8）：nonzero fill rule（spec dom-context-2d-fill 默认）——
            // 交点带方向符号（段向下 = +1 / 向上 = −1，屏幕 y 向下），按 x 排序后
            // 累计绕组，非零区间填充。旧偶奇配对（排序后两两成对）对嵌套同向
            // 子路径（winding.add：外矩形+同向内矩形，中心绕组 ±2）和对角连线
            // 杂散交点都会配对破裂（挖出假洞/漏填）。
            // https://html.spec.whatwg.org/multipage/canvas.html#dom-context-2d-fill
            for (sx, ex) in fill_rule_spans(vertices, sy, rule) {
                let ix_start_f = sx.max(0.0);
                let ix_end_f = ex.min(canvas_w as f32);
                let full_start = ix_start_f.ceil() as u32;
                let full_end = ix_end_f.floor() as u32;
                for scan_x in full_start..full_end {
                    // R34xx：clip 区域裁剪（clip_path 未设时零开销）。
                    if !self.clip_applies(scan_x as f32, scan_y as f32) {
                        continue;
                    }
                    let idx = ((scan_y * canvas_w + scan_x) * 4) as usize;
                    if idx + 3 < self.pixel_buffer.len() {
                        let color = self.apply_alpha(style.sample_at(scan_x as f32, sy));
                        // R3236：路径渐变填充消费合成操作（与 blit_path_to_pixels / blit_rect_gradient 一致）。
                        let (r, g, b, a) = self.composite_pixel(
                            color,
                            self.pixel_buffer[idx],
                            self.pixel_buffer[idx + 1],
                            self.pixel_buffer[idx + 2],
                            self.pixel_buffer[idx + 3],
                        );
                        self.pixel_buffer[idx] = r;
                        self.pixel_buffer[idx + 1] = g;
                        self.pixel_buffer[idx + 2] = b;
                        self.pixel_buffer[idx + 3] = a;
                    }
                }
                // R57（M3）：span 边界像素 AA（渐变采样位置 = 像素中心，与全覆盖
                // 像素同 scan_y 扫描线；coverage 超采样判定与纯色路径一致）。
                if rotated {
                    if sx - sx.floor() > 1e-6 {
                        let b = sx.floor().max(0.0) as u32;
                        if b < canvas_w {
                            let cov = self.path_pixel_coverage(vertices, b as f32, scan_y as f32, rule);
                            if cov > 0.0 && cov < 1.0 {
                                let color = self.apply_alpha(style.sample_at(b as f32 + 0.5, sy));
                                self.blit_path_edge_pixel(b, scan_y, color, cov);
                            }
                        }
                    }
                    if ex - ex.floor() > 1e-6 {
                        let b = ex.floor().max(0.0) as u32;
                        if b < canvas_w {
                            let cov = self.path_pixel_coverage(vertices, b as f32, scan_y as f32, rule);
                            if cov > 0.0 && cov < 1.0 {
                                let color = self.apply_alpha(style.sample_at(b as f32 + 0.5, sy));
                                self.blit_path_edge_pixel(b, scan_y, color, cov);
                            }
                        }
                    }
                }
            }
        }
    }

    /// 将路径描边写入像素缓冲区（考虑 line_join 和 line_cap 设置）。
    pub(crate) fn blit_stroke_to_pixels(&mut self, vertices: &[f32], color: Color, line_width: f32, closed: bool) {
        if vertices.len() < 4 {
            return;
        }

        let half_lw = line_width / 2.0;

        // 将线段顶点列表转为 (x1,y1,x2,y2) 段列表
        // R56i（M8/DC-8）：spec dom-context-2d-stroke——零长线段在 stroke 前**剪除**
        //（2d.path.stroke.prune.*：moveTo(50,25)+lineTo(50,25) 的零长段不得画
        // round cap 圆盘——剪除后段列表为空则无 cap/join）。
        let segments: Vec<[f32; 4]> = vertices
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .filter(|seg| {
                let len = (seg[2] - seg[0]).hypot(seg[3] - seg[1]);
                len > f32::EPSILON
            })
            .collect();
        if segments.is_empty() {
            return;
        }

        // 绘制每条线段的主体矩形。R34xx：闭合路径（closed）段矩形沿线段**垂直方向**扩
        // half_lw（线宽），端点方向不扩——闭合线无 cap，线帽外扩只存在于开放段端点
        //（上游 2d.strokeRect.zero.*：Nx0 退化矩形的闭合线无 cap，端点外区域不得被段矩形
        // 覆盖；同时线宽须应用于垂直方向，2d.strokeRect.transform 期望 scale 后线仍 5px 宽）。
        // R34xx：stroke 单次调用去重 mask（段矩形/join/cap 重叠像素只合成一次——
        // 2d.strokeStyle.colorObject.transparency 的 2px 高矩形 50px 线宽）。
        self.stroke_dedup_mask = Some(vec![0u8; (self.width as usize) * (self.height as usize)]);
        self.stroke_dedup_mask = Some(vec![0u8; (self.width as usize) * (self.height as usize)]);
        let mut dev_half_lw = half_lw;
        for seg in &segments {
            // R34xx：per-segment 设备空间半线宽（CTM 非均匀变换下随段方向变化）。
            let seg_half = self.device_half_width(seg[2] - seg[0], seg[3] - seg[1], half_lw);
            dev_half_lw = seg_half;
            let (ax, ay) = (seg[0], seg[1]);
            let (bx, by) = (seg[2], seg[3]);
            let (dx, dy) = (bx - ax, by - ay);
            let len = (dx * dx + dy * dy).sqrt();
            if len < f32::EPSILON {
                continue; // 零长段（退化矩形往返的重复点）无像素
            }
            // R34xx：逐像素精确判定（距中心线 ≤ seg_half 且投影 ∈ [0,1]）——旧轴对齐 bbox
            // 对斜线段覆盖端点前角落（2d.line.miter.acute 的 (48,48) 在红斜线起点前 72px
            // 仍被 bbox 覆盖）。bbox 仅作遍历范围。
            let (nx, ny) = (-dy / len * seg_half, dx / len * seg_half);
            let min_x = ax.min(bx).min(ax + nx).min(bx + nx).min(ax - nx).min(bx - nx);
            let max_x = ax.max(bx).max(ax + nx).max(bx + nx).max(ax - nx).max(bx - nx);
            let min_y = ay.min(by).min(ay + ny).min(by + ny).min(ay - ny).min(by - ny);
            let max_y = ay.max(by).max(ay + ny).max(by + ny).max(ay - ny).max(by - ny);
            let x0 = min_x.max(0.0).floor() as i32;
            let y0 = min_y.max(0.0).floor() as i32;
            let x1 = (max_x.min(self.width as f32).ceil() as i32).min(self.width as i32);
            let y1 = (max_y.min(self.height as f32).ceil() as i32).min(self.height as i32);
            let len2 = len * len;
            // R56h（M8/DC-8）：像素方形 vs 带相交（非中心点判定）——判定半径
            // half + 0.5（像素内切半径）。真实光栅是面积覆盖：临界像素（中心
            // 距中心线 half+0.4px 内，方形与带相交）须着色（2d.path.bezierCurveTo.
            // shape/scaled 的 (1,1)：像素中心距曲线 27.9、half 27.5，方形最近角
            // 27.2 < 27.5 相交——旧中心点判定 miss）。
            let h2 = (seg_half + 0.5) * (seg_half + 0.5);
            for py in y0..y1 {
                for px in x0..x1 {
                    let (qx, qy) = (px as f32 + 0.5 - ax, py as f32 + 0.5 - ay);
                    let t = (qx * dx + qy * dy) / len2;
                    if t < 0.0 || t > 1.0 {
                        continue;
                    }
                    let (rx, ry) = (qx - t * dx, qy - t * dy);
                    if rx * rx + ry * ry > h2 {
                        continue;
                    }
                    self.blit_pixel(px, py, color);
                }
            }
        }

        // 绘制连接点（相邻线段交汇处）。R34xx：共线角（Miter/Bevel）不画——Nx0 退化
        // 矩形往返线端点的 180° 角不得覆盖端点外区域（zero.4）。join 形状按 lineJoin 真实
        // 几何（miter 尖角三角 / bevel 平切三角 / round 圆盘）——旧方块近似把 90° miter
        // 画成 400×400 方块，覆盖角点外大片区域（2d.line.cap.closed/join.open 等失败）。
        // R56i：closed 时环绕 join（末段→首段——roundRect 闭合环的起点 join）。
        let join_count = if closed {
            segments.len()
        } else {
            segments.len().saturating_sub(1)
        };
        for i in 0..join_count {
            let seg_a = segments[i];
            let seg_b = segments[(i + 1) % segments.len()];
            if !self.join_visible(&seg_a, &seg_b) {
                continue;
            }
            // seg_a 的终点应与 seg_b 的起点相同
            let jx = seg_a[2];
            let jy = seg_a[3];
            self.blit_join(&seg_a, &seg_b, jx, jy, dev_half_lw, color);
        }

        // R34xx：闭合路径首尾段连接处画 join（最后段终点 = 第一段起点；退化矩形 flatten
        // 的往返线两端点均须 join——上游 2d.strokeRect.zero.5 lineJoin=round 期望端点外覆盖；
        // Miter 共线角不画——zero.4 lineCap=round 端点外透明）。
        if closed && segments.len() >= 2 {
            let last_seg = segments[segments.len() - 1];
            let first_seg = segments[0];
            if self.join_visible(&last_seg, &first_seg) {
                self.blit_join(&last_seg, &first_seg, last_seg[2], last_seg[3], dev_half_lw, color);
            }
        }

        // 绘制端点 cap（闭合路径无端点 → 不画）
        if !closed {
            let first_seg = segments[0];
            let last_seg = segments[segments.len() - 1];

            // 起点端 cap
            self.blit_line_cap(
                first_seg[0],
                first_seg[1],
                first_seg[2],
                first_seg[3],
                dev_half_lw,
                color,
            );
            // 终点端 cap
            self.blit_line_cap(last_seg[2], last_seg[3], last_seg[0], last_seg[1], dev_half_lw, color);
        }
        self.stroke_dedup_mask = None;
    }

    /// 绘制线段端点的 cap。
    /// `endpoint` 是端点位置，`other` 是线段另一端（用于确定方向）。
    pub(crate) fn blit_line_cap(
        &mut self,
        endpoint_x: f32,
        endpoint_y: f32,
        other_x: f32,
        other_y: f32,
        half_lw: f32,
        color: Color,
    ) {
        match self.line_cap {
            LineCap::Butt => {
                // 平头：不做额外处理（线段矩形已精确到端点）
            }
            LineCap::Round => {
                // R34xx：真圆盘（半径 half_lw）——旧方块覆盖圆外四角
                // （2d.line.cap.round 断言 (67,6) 距端点 12.04 > 半径 10 应透明）。
                let r2 = half_lw * half_lw;
                let x0 = (endpoint_x - half_lw).floor() as i32;
                let y0 = (endpoint_y - half_lw).floor() as i32;
                let x1 = (endpoint_x + half_lw).ceil() as i32;
                let y1 = (endpoint_y + half_lw).ceil() as i32;
                for py in y0..y1 {
                    for px in x0..x1 {
                        let dx = px as f32 + 0.5 - endpoint_x;
                        let dy = py as f32 + 0.5 - endpoint_y;
                        if dx * dx + dy * dy <= r2 {
                            self.blit_pixel(px, py, color);
                        }
                    }
                }
            }
            LineCap::Square => {
                // 方头：在端点方向延伸 half_lw 的矩形
                let dx = endpoint_x - other_x;
                let dy = endpoint_y - other_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len < f32::EPSILON {
                    return;
                }
                let ux = dx / len;
                let uy = dy / len;
                // 从端点沿方向延伸 half_lw
                let ext_x = endpoint_x + ux * half_lw;
                let ext_y = endpoint_y + uy * half_lw;
                // R34xx：cap 矩形 = 延伸段（endpoint→ext）垂直扩 half_lw——旧实现 min/max
                // 再 ±half_lw 双向外扩，覆盖端点反侧 half_lw（2d.line.cap.square (75,4)
                // 距端点 11 > 10 仍被红 cap 覆盖）。
                let rect = self.line_segment_rect(endpoint_x, endpoint_y, ext_x, ext_y, half_lw * 2.0);
                self.blit_rect_to_pixels(&rect, color);
            }
        }
    }

    /// 路径描边**渐变**光栅化（R3084）：与 `blit_stroke_to_pixels` 同几何（段主体 + 连接点 + 端点 cap），
    /// 但每矩形经 `blit_rect_gradient` 逐像素采样样式颜色（与 fill 渐变 R3079 对称）。供渐变 stroke_style 用。
    pub(crate) fn blit_stroke_to_pixels_gradient(
        &mut self,
        vertices: &[f32],
        style: &CanvasStyle,
        line_width: f32,
        closed: bool,
    ) {
        if vertices.len() < 4 {
            return;
        }
        let half_lw = line_width / 2.0;
        // R56i（M8/DC-8）：spec dom-context-2d-stroke——零长线段在 stroke 前**剪除**
        //（2d.path.stroke.prune.*：moveTo(50,25)+lineTo(50,25) 的零长段不得画
        // round cap 圆盘——剪除后段列表为空则无 cap/join）。
        let segments: Vec<[f32; 4]> = vertices
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .filter(|seg| {
                let len = (seg[2] - seg[0]).hypot(seg[3] - seg[1]);
                len > f32::EPSILON
            })
            .collect();
        if segments.is_empty() {
            return;
        }
        // 段主体（R34xx：closed 段矩形垂直扩线宽、端点不扩，同 blit_stroke_to_pixels；
        // per-segment 设备空间半线宽同 flat 版）。
        let mut dev_half_lw = half_lw;
        for seg in &segments {
            let seg_half = self.device_half_width(seg[2] - seg[0], seg[3] - seg[1], half_lw);
            dev_half_lw = seg_half;
            let (ax, ay) = (seg[0], seg[1]);
            let (bx, by) = (seg[2], seg[3]);
            let (dx, dy) = (bx - ax, by - ay);
            let len = (dx * dx + dy * dy).sqrt();
            if len < f32::EPSILON {
                continue;
            }
            // R34xx：逐像素精确判定（同 blit_stroke_to_pixels）。
            let (nx, ny) = (-dy / len * seg_half, dx / len * seg_half);
            let min_x = ax.min(bx).min(ax + nx).min(bx + nx).min(ax - nx).min(bx - nx);
            let max_x = ax.max(bx).max(ax + nx).max(bx + nx).max(ax - nx).max(bx - nx);
            let min_y = ay.min(by).min(ay + ny).min(by + ny).min(ay - ny).min(by - ny);
            let max_y = ay.max(by).max(ay + ny).max(by + ny).max(ay - ny).max(by - ny);
            let x0 = min_x.max(0.0).floor() as i32;
            let y0 = min_y.max(0.0).floor() as i32;
            let x1 = (max_x.min(self.width as f32).ceil() as i32).min(self.width as i32);
            let y1 = (max_y.min(self.height as f32).ceil() as i32).min(self.height as i32);
            let len2 = len * len;
            // R56h（M8/DC-8）：像素方形 vs 带相交（非中心点判定）——判定半径
            // half + 0.5（像素内切半径）。真实光栅是面积覆盖：临界像素（中心
            // 距中心线 half+0.4px 内，方形与带相交）须着色（2d.path.bezierCurveTo.
            // shape/scaled 的 (1,1)：像素中心距曲线 27.9、half 27.5，方形最近角
            // 27.2 < 27.5 相交——旧中心点判定 miss）。
            let h2 = (seg_half + 0.5) * (seg_half + 0.5);
            for py in y0..y1 {
                for px in x0..x1 {
                    let (qx, qy) = (px as f32 + 0.5 - ax, py as f32 + 0.5 - ay);
                    let t = (qx * dx + qy * dy) / len2;
                    if t < 0.0 || t > 1.0 {
                        continue;
                    }
                    let (rx, ry) = (qx - t * dx, qy - t * dy);
                    if rx * rx + ry * ry > h2 {
                        continue;
                    }
                    self.blit_pixel_gradient(px, py, style);
                }
            }
        }
        // 连接点（R34xx：同 blit_stroke_to_pixels 真实 join 几何；共线角不画）
        // R56i：closed 时环绕 join（末段→首段——roundRect 闭合环的起点 join）。
        let join_count = if closed {
            segments.len()
        } else {
            segments.len().saturating_sub(1)
        };
        for i in 0..join_count {
            let seg_a = segments[i];
            let seg_b = segments[(i + 1) % segments.len()];
            if !self.join_visible(&seg_a, &seg_b) {
                continue;
            }
            self.blit_join_gradient(&seg_a, &seg_b, seg_a[2], seg_a[3], dev_half_lw, style);
        }
        // R34xx：闭合路径首尾段连接处画 join（同 blit_stroke_to_pixels；共线角不画）。
        if closed && segments.len() >= 2 {
            let last_seg = segments[segments.len() - 1];
            let first_seg = segments[0];
            if self.join_visible(&last_seg, &first_seg) {
                self.blit_join_gradient(&last_seg, &first_seg, last_seg[2], last_seg[3], dev_half_lw, style);
            }
        }
        // 端点 cap（闭合路径无端点 → 不画）
        if !closed {
            let first_seg = segments[0];
            let last_seg = segments[segments.len() - 1];
            self.blit_line_cap_gradient(
                first_seg[0],
                first_seg[1],
                first_seg[2],
                first_seg[3],
                dev_half_lw,
                style,
            );
            self.blit_line_cap_gradient(last_seg[2], last_seg[3], last_seg[0], last_seg[1], dev_half_lw, style);
        }
        self.stroke_dedup_mask = None;
    }

    /// 线段端点 cap **渐变**光栅化（R3084）：与 `blit_line_cap` 同几何，每矩形经 `blit_rect_gradient`。
    pub(crate) fn blit_line_cap_gradient(
        &mut self,
        endpoint_x: f32,
        endpoint_y: f32,
        other_x: f32,
        other_y: f32,
        half_lw: f32,
        style: &CanvasStyle,
    ) {
        match self.line_cap {
            LineCap::Butt => {}
            LineCap::Round => {
                // R34xx：真圆盘（同 blit_line_cap）。
                let r2 = half_lw * half_lw;
                let x0 = (endpoint_x - half_lw).floor() as i32;
                let y0 = (endpoint_y - half_lw).floor() as i32;
                let x1 = (endpoint_x + half_lw).ceil() as i32;
                let y1 = (endpoint_y + half_lw).ceil() as i32;
                for py in y0..y1 {
                    for px in x0..x1 {
                        let dx = px as f32 + 0.5 - endpoint_x;
                        let dy = py as f32 + 0.5 - endpoint_y;
                        if dx * dx + dy * dy <= r2 {
                            self.blit_pixel_gradient(px, py, style);
                        }
                    }
                }
            }
            LineCap::Square => {
                let dx = endpoint_x - other_x;
                let dy = endpoint_y - other_y;
                let len = (dx * dx + dy * dy).sqrt();
                if len < f32::EPSILON {
                    return;
                }
                let ux = dx / len;
                let uy = dy / len;
                let ext_x = endpoint_x + ux * half_lw;
                let ext_y = endpoint_y + uy * half_lw;
                // R34xx：cap 矩形 = 延伸段垂直扩 half_lw（同 blit_line_cap）。
                let rect = self.line_segment_rect(endpoint_x, endpoint_y, ext_x, ext_y, half_lw * 2.0);
                self.blit_rect_gradient(&rect, style);
            }
        }
    }

    /// 计算线段的描边矩形（沿线段方向扩展 line_width / 2）。
    pub(crate) fn line_segment_rect(&self, x1: f32, y1: f32, x2: f32, y2: f32, line_width: f32) -> Rect {
        // R34xx：段矩形精确到端点（不向端点外扩）——端点延伸属 cap 的职责（butt 无延伸、
        // round/square 由 blit_line_cap 单独画）。旧实现两端各扩 half_lw 使 butt cap 覆盖
        // 端点外区域（2d.line.cap.butt/closed、join.open、miter.* 角落 (1,1) 全失败）。
        let half_lw = line_width / 2.0;
        let (dx, dy) = (x2 - x1, y2 - y1);
        let len = (dx * dx + dy * dy).sqrt();
        if len < f32::EPSILON {
            return Rect::new(x1, y1, 0.0, 0.0);
        }
        let (nx, ny) = (-dy / len * half_lw, dx / len * half_lw);
        let min_x = x1.min(x2).min(x1 + nx).min(x2 + nx).min(x1 - nx).min(x2 - nx);
        let max_x = x1.max(x2).max(x1 + nx).max(x2 + nx).max(x1 - nx).max(x2 - nx);
        let min_y = y1.min(y2).min(y1 + ny).min(y2 + ny).min(y1 - ny).min(y2 - ny);
        let max_y = y1.max(y2).max(y1 + ny).max(y2 + ny).max(y1 - ny).max(y2 - ny);
        Rect::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// 计算描边路径的顶点，包括 line_join 和 line_cap 产生的额外顶点。
    /// 返回一个包含 (x, y) 对的顶点列表，构成描边的轮廓多边形。
    pub fn stroke_outline_vertices(&self) -> Vec<f32> {
        // R56：轮廓提取是 stroke 语义（开放子路径不闭合）。
        let path_vertices = self.flatten_path_open();
        if path_vertices.len() < 4 {
            return Vec::new();
        }

        let half_lw = self.line_width / 2.0;
        let segments: Vec<[f32; 4]> = path_vertices
            .chunks_exact(4)
            .map(|c| [c[0], c[1], c[2], c[3]])
            .collect();
        let mut outline = Vec::new();

        for (i, seg) in segments.iter().enumerate() {
            let x1 = seg[0];
            let y1 = seg[1];
            let x2 = seg[2];
            let y2 = seg[3];
            let dx = x2 - x1;
            let dy = y2 - y1;
            let len = (dx * dx + dy * dy).sqrt();
            if len < f32::EPSILON {
                continue;
            }
            let nx = -dy / len * half_lw; // 法线方向
            let ny = dx / len * half_lw;

            // 线段主体：4 个角点
            outline.push(x1 + nx);
            outline.push(y1 + ny);
            outline.push(x2 + nx);
            outline.push(y2 + ny);
            outline.push(x2 - nx);
            outline.push(y2 - ny);
            outline.push(x1 - nx);
            outline.push(y1 - ny);

            // 起点端 cap（仅第一条线段）
            if i == 0 {
                match self.line_cap {
                    LineCap::Butt => {}
                    LineCap::Round => {
                        let cx = x1;
                        let cy = y1;
                        let dir_x = -dx / len;
                        let dir_y = -dy / len;
                        const CAP_SEGMENTS: usize = 4;
                        for j in 0..CAP_SEGMENTS {
                            let a1 = std::f32::consts::PI * j as f32 / CAP_SEGMENTS as f32;
                            let a2 = std::f32::consts::PI * (j + 1) as f32 / CAP_SEGMENTS as f32;
                            let base_angle = dir_y.atan2(dir_x) - std::f32::consts::FRAC_PI_2;
                            outline.push(cx);
                            outline.push(cy);
                            outline.push(cx + half_lw * (base_angle + a1).cos());
                            outline.push(cy + half_lw * (base_angle + a1).sin());
                            outline.push(cx + half_lw * (base_angle + a2).cos());
                            outline.push(cy + half_lw * (base_angle + a2).sin());
                        }
                    }
                    LineCap::Square => {
                        let dir_x = -dx / len;
                        let dir_y = -dy / len;
                        let ext = half_lw;
                        outline.push(x1 + nx);
                        outline.push(y1 + ny);
                        outline.push(x1 + nx + dir_x * ext);
                        outline.push(y1 + ny + dir_y * ext);
                        outline.push(x1 - nx + dir_x * ext);
                        outline.push(y1 - ny + dir_y * ext);
                        outline.push(x1 - nx);
                        outline.push(y1 - ny);
                    }
                }
            }

            // 连接点（与下一条线段之间）
            if i < segments.len() - 1 {
                let next = segments[i + 1];
                let ndx = next[2] - next[0];
                let ndy = next[3] - next[1];
                let nlen = (ndx * ndx + ndy * ndy).sqrt();

                if nlen >= f32::EPSILON {
                    let nnx = -ndy / nlen * half_lw;
                    let nny = ndx / nlen * half_lw;
                    let jx = x2;
                    let jy = y2;

                    match self.line_join {
                        LineJoin::Miter => {
                            // 尖角：延伸两侧法线的交点
                            let miter_len = Self::compute_miter_length(nx, ny, nnx, nny, half_lw);
                            let mx = nx + nnx;
                            let my = ny + nny;
                            let m = (mx * mx + my * my).sqrt();
                            if m > f32::EPSILON {
                                let miter_x = jx + mx / m * miter_len;
                                let miter_y = jy + my / m * miter_len;
                                outline.push(jx + nx);
                                outline.push(jy + ny);
                                outline.push(miter_x);
                                outline.push(miter_y);
                                outline.push(jx + nnx);
                                outline.push(jy + nny);
                            }
                        }
                        LineJoin::Round => {
                            // 圆角：在连接点画扇形
                            const JOIN_SEGMENTS: usize = 4;
                            let start_angle = ny.atan2(nx);
                            let end_angle = nny.atan2(nnx);
                            let step = {
                                let diff = end_angle - start_angle;
                                if diff > std::f32::consts::PI {
                                    diff - std::f32::consts::TAU
                                } else if diff < -std::f32::consts::PI {
                                    diff + std::f32::consts::TAU
                                } else {
                                    diff
                                }
                            } / JOIN_SEGMENTS as f32;
                            let mut angle = start_angle;
                            for _ in 0..JOIN_SEGMENTS {
                                let a1 = angle;
                                let a2 = angle + step;
                                outline.push(jx);
                                outline.push(jy);
                                outline.push(jx + half_lw * a1.cos());
                                outline.push(jy + half_lw * a1.sin());
                                outline.push(jx + half_lw * a2.cos());
                                outline.push(jy + half_lw * a2.sin());
                                angle = a2;
                            }
                        }
                        LineJoin::Bevel => {
                            // 斜角：三角形连接
                            outline.push(jx + nx);
                            outline.push(jy + ny);
                            outline.push(jx + nnx);
                            outline.push(jy + nny);
                        }
                    }
                }
            }

            // 终点端 cap（仅最后一条线段）
            if i == segments.len() - 1 {
                match self.line_cap {
                    LineCap::Butt => {}
                    LineCap::Round => {
                        let cx = x2;
                        let cy = y2;
                        let dir_x = dx / len;
                        let dir_y = dy / len;
                        const CAP_SEGMENTS: usize = 4;
                        for j in 0..CAP_SEGMENTS {
                            let a1 = std::f32::consts::PI * j as f32 / CAP_SEGMENTS as f32;
                            let a2 = std::f32::consts::PI * (j + 1) as f32 / CAP_SEGMENTS as f32;
                            let base_angle = dir_y.atan2(dir_x) - std::f32::consts::FRAC_PI_2;
                            outline.push(cx);
                            outline.push(cy);
                            outline.push(cx + half_lw * (base_angle + a1).cos());
                            outline.push(cy + half_lw * (base_angle + a1).sin());
                            outline.push(cx + half_lw * (base_angle + a2).cos());
                            outline.push(cy + half_lw * (base_angle + a2).sin());
                        }
                    }
                    LineCap::Square => {
                        let dir_x = dx / len;
                        let dir_y = dy / len;
                        let ext = half_lw;
                        outline.push(x2 + nx);
                        outline.push(y2 + ny);
                        outline.push(x2 + nx + dir_x * ext);
                        outline.push(y2 + ny + dir_y * ext);
                        outline.push(x2 - nx + dir_x * ext);
                        outline.push(y2 - ny + dir_y * ext);
                        outline.push(x2 - nx);
                        outline.push(y2 - ny);
                    }
                }
            }
        }

        outline
    }

    /// 计算 miter 连接的长度（从连接点到尖角顶点的距离）。
    pub(crate) fn compute_miter_length(nx: f32, ny: f32, nnx: f32, nny: f32, half_lw: f32) -> f32 {
        let mx = nx + nnx;
        let my = ny + nny;
        let m = (mx * mx + my * my).sqrt();
        if m < f32::EPSILON {
            return half_lw;
        }
        half_lw * 2.0 / m
    }
}
