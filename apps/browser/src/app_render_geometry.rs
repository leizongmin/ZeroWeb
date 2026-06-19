// --- 渲染工具函数 ---

/// 创建填充矩形图元
fn rect_fill(x: f32, y: f32, w: f32, h: f32, color: Color) -> FillPrimitive {
    FillPrimitive {
        rect: zero_render_foundation::geometry::Rect::new(x, y, w, h),
        color,
    }
}

/// 圆角矩形在指定行的水平可见区间 `(x_start, x_end)`。
fn rounded_rect_x_span_at_y(yf: f32, x: f32, y: f32, w: f32, h: f32, radius: f32) -> Option<(f32, f32)> {
    if yf < y || yf >= y + h {
        return None;
    }
    let r = radius.min(w * 0.5).min(h * 0.5);
    if r <= f32::EPSILON {
        return Some((x, x + w));
    }

    let r_sq = r * r;
    let mut x_start = x;
    let mut x_end = x + w;

    let dy_top = (y + r) - yf;
    if dy_top > 0.0 {
        let dx = (r_sq - dy_top * dy_top).max(0.0).sqrt();
        x_start = x + r - dx;
        x_end = x + w - r + dx;
    } else {
        let dy_bottom = yf - (y + h - r);
        if dy_bottom > 0.0 {
            let dx = (r_sq - dy_bottom * dy_bottom).max(0.0).sqrt();
            x_start = x + r - dx;
            x_end = x + w - r + dx;
        }
    }

    if x_end <= x_start {
        None
    } else {
        Some((x_start, x_end))
    }
}

/// 将轴对齐矩形裁剪到圆角矩形内，按行写入 fill。
#[allow(clippy::too_many_arguments)]
fn push_fill_clipped_to_rounded_rect(
    fills: &mut Vec<FillPrimitive>,
    fx: f32,
    fy: f32,
    fw: f32,
    fh: f32,
    color: Color,
    rx: f32,
    ry: f32,
    rw: f32,
    rh: f32,
    radius: f32,
) {
    let ix0 = fx.max(rx);
    let iy0 = fy.max(ry);
    let ix1 = (fx + fw).min(rx + rw);
    let iy1 = (fy + fh).min(ry + rh);
    if ix0 >= ix1 || iy0 >= iy1 {
        return;
    }

    let min_row = iy0.floor() as i32;
    let max_row = iy1.ceil() as i32;
    for row in min_row..max_row {
        let yf = row as f32 + 0.5;
        if yf < iy0 || yf >= iy1 {
            continue;
        }
        let Some((mut xs, mut xe)) = rounded_rect_x_span_at_y(yf, rx, ry, rw, rh, radius) else {
            continue;
        };
        xs = xs.max(ix0);
        xe = xe.min(ix1);
        if xe > xs {
            fills.push(rect_fill(xs, row as f32, xe - xs, 1.0, color));
        }
    }
}

/// 轴对齐矩形是否与圆角矩形有交集（用于 glyph 裁剪）。
#[allow(clippy::too_many_arguments)]
fn axis_rect_intersects_rounded_rect(
    ax: f32,
    ay: f32,
    aw: f32,
    ah: f32,
    rx: f32,
    ry: f32,
    rw: f32,
    rh: f32,
    radius: f32,
) -> bool {
    if ax >= rx + rw || ax + aw <= rx || ay >= ry + rh || ay + ah <= ry {
        return false;
    }
    let sample_y = (ay + ah * 0.5).clamp(ry, ry + rh - f32::EPSILON);
    let Some((xs, xe)) = rounded_rect_x_span_at_y(sample_y, rx, ry, rw, rh, radius) else {
        return false;
    };
    ax + aw > xs && ax < xe
}

/// 将圆角矩形外、轴对齐包围盒内的区域用指定颜色覆盖（清除四角溢出）。
fn push_rounded_rect_outside_corner_masks(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: Color,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = radius.min(w * 0.5).min(h * 0.5);
    if r <= f32::EPSILON {
        return;
    }

    let min_row = y.floor() as i32;
    let max_row = (y + h).ceil() as i32;

    for row in min_row..max_row {
        let yf = row as f32 + 0.5;
        if yf < y || yf >= y + h {
            continue;
        }
        let Some((xs, xe)) = rounded_rect_x_span_at_y(yf, x, y, w, h, r) else {
            continue;
        };
        if xs > x {
            fills.push(rect_fill(x, row as f32, xs - x, 1.0, color));
        }
        if x + w > xe {
            fills.push(rect_fill(xe, row as f32, x + w - xe, 1.0, color));
        }
    }
}

/// 圆角矩形描边（在内容之上绘制，仅输出边框环）。
#[allow(clippy::too_many_arguments)]
fn push_rounded_rect_border(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    border: f32,
    color: Color,
) {
    if w <= 0.0 || h <= 0.0 || border <= 0.0 {
        return;
    }
    let outer_r = radius.min(w * 0.5).min(h * 0.5);
    let inner_r = (outer_r - border).max(0.0);
    let min_row = y.floor() as i32;
    let max_row = (y + h).ceil() as i32;

    for row in min_row..max_row {
        let yf = row as f32 + 0.5;
        if yf < y || yf >= y + h {
            continue;
        }
        let Some((ox0, ox1)) = rounded_rect_x_span_at_y(yf, x, y, w, h, outer_r) else {
            continue;
        };
        if inner_r <= f32::EPSILON {
            fills.push(rect_fill(ox0, row as f32, ox1 - ox0, 1.0, color));
            continue;
        }
        let Some((ix0, ix1)) =
            rounded_rect_x_span_at_y(yf, x + border, y + border, w - 2.0 * border, h - 2.0 * border, inner_r)
        else {
            fills.push(rect_fill(ox0, row as f32, ox1 - ox0, 1.0, color));
            continue;
        };
        if ix0 > ox0 {
            fills.push(rect_fill(ox0, row as f32, ix0 - ox0, 1.0, color));
        }
        if ox1 > ix1 {
            fills.push(rect_fill(ix1, row as f32, ox1 - ix1, 1.0, color));
        }
    }
}

/// 四角圆角矩形（`radius = h/2` 时为胶囊形地址栏）。
fn push_rounded_rect_fill(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    radius: f32,
    color: Color,
) {
    if w <= 0.0 || h <= 0.0 {
        return;
    }
    let r = radius.min(w * 0.5).min(h * 0.5);
    if r <= f32::EPSILON {
        fills.push(rect_fill(x, y, w, h, color));
        return;
    }

    let r_sq = r * r;
    let min_y = y.floor() as i32;
    let max_y = (y + h).ceil() as i32;

    for row in min_y..max_y {
        let yf = row as f32 + 0.5;
        if yf < y || yf >= y + h {
            continue;
        }

        let mut x_start = x;
        let mut x_end = x + w;

        let dy_top = (y + r) - yf;
        if dy_top > 0.0 {
            let dx = (r_sq - dy_top * dy_top).max(0.0).sqrt();
            x_start = x + r - dx;
            x_end = x + w - r + dx;
        } else {
            let dy_bottom = yf - (y + h - r);
            if dy_bottom > 0.0 {
                let dx = (r_sq - dy_bottom * dy_bottom).max(0.0).sqrt();
                x_start = x + r - dx;
                x_end = x + w - r + dx;
            }
        }

        if x_end > x_start {
            fills.push(rect_fill(x_start, row as f32, x_end - x_start, 1.0, color));
        }
    }
}

fn draw_hollow_square(fills: &mut Vec<FillPrimitive>, x: f32, y: f32, size: f32, thickness: f32, color: Color) {
    fills.push(rect_fill(x, y, size, thickness, color));
    fills.push(rect_fill(x, y + size - thickness, size, thickness, color));
    fills.push(rect_fill(x, y, thickness, size, color));
    fills.push(rect_fill(x + size - thickness, y, thickness, size, color));
}

/// 实心圆盘（用于图标 hover 背景等）
fn push_circle_fill(fills: &mut Vec<FillPrimitive>, cx: f32, cy: f32, diameter: f32, color: Color) {
    let r = diameter * 0.5;
    if r <= 0.0 {
        return;
    }
    let min_y = (cy - r).floor() as i32;
    let max_y = (cy + r).ceil() as i32;
    let r_sq = r * r;

    for y in min_y..=max_y {
        let yf = y as f32 + 0.5;
        let dy = yf - cy;
        let dx_max = (r_sq - dy * dy).max(0.0).sqrt();
        if dx_max <= f32::EPSILON {
            continue;
        }
        fills.push(rect_fill(cx - dx_max, y as f32, dx_max * 2.0, 1.0, color));
    }
}

/// 按真实字体 advance 重新排列 WebView 文本 glyph（与 UI 文本一致）
pub(crate) fn reflow_webview_glyphs(
    glyphs: &mut [zero_render_foundation::primitive::GlyphPrimitive],
    font_loader: &FontLoader,
    primary_id: u32,
) {
    use std::collections::HashMap;

    if glyphs.is_empty() {
        return;
    }

    let mut lines: HashMap<i32, Vec<usize>> = HashMap::new();
    for (i, glyph) in glyphs.iter().enumerate() {
        if glyph.glyph_id == 0 {
            continue;
        }
        let Some(ch) = char::from_u32(glyph.glyph_id) else {
            continue;
        };
        if ch == '\0' {
            continue;
        }
        let key = (glyph.y * 2.0).round() as i32;
        lines.entry(key).or_default().push(i);
    }

    for indices in lines.into_values() {
        let mut indices = indices;
        indices.sort_by(|&a, &b| {
            glyphs[a]
                .x
                .partial_cmp(&glyphs[b].x)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut cursor_x = glyphs[indices[0]].x;
        let mut i = 0;
        while i < indices.len() {
            let cluster_x = glyphs[indices[i]].x;
            let font_size = glyphs[indices[i]].font_size;
            let Some(ch) = char::from_u32(glyphs[indices[i]].glyph_id) else {
                i += 1;
                continue;
            };

            let mut j = i + 1;
            while j < indices.len() && (glyphs[indices[j]].x - cluster_x).abs() < 1.0 {
                j += 1;
            }

            for idx in &indices[i..j] {
                let offset = glyphs[*idx].x - cluster_x;
                glyphs[*idx].x = cursor_x + offset;
            }

            cursor_x += font_loader.measure_advance(primary_id, ch, font_size);
            i = j;
        }
    }
}
