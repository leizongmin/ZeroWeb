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
    // 对齐到整数像素，保证四条边等宽且锐利（避免 CPU backend floor/ceil 造成上下/左右边宽度不一致）。
    let x = x.round();
    let y = y.round();
    let size = size.round();
    let thickness = thickness.round().max(1.0);
    fills.push(rect_fill(x, y, size, thickness, color));
    fills.push(rect_fill(x, y + size - thickness, size, thickness, color));
    fills.push(rect_fill(x, y, thickness, size, color));
    fills.push(rect_fill(x + size - thickness, y, thickness, size, color));
}

/// 绘制与 `assets/app-icon.svg` 一致的 ZeroWeb 三色环形标识。
fn push_zero_web_icon(fills: &mut Vec<FillPrimitive>, cx: f32, cy: f32, diameter: f32, background: Color) {
    let radius = diameter * 0.5;
    let radius_sq = radius * radius;
    let min_x = (cx - radius).floor() as i32;
    let max_x = (cx + radius).ceil() as i32;
    let min_y = (cy - radius).floor() as i32;
    let max_y = (cy + radius).ceil() as i32;
    let blue = Color { r: 74, g: 158, b: 255, a: 255 };
    let teal = Color { r: 20, g: 184, b: 166, a: 255 };
    let indigo = Color { r: 30, g: 79, b: 208, a: 255 };

    for y in min_y..max_y {
        for x in min_x..max_x {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            if dx * dx + dy * dy > radius_sq {
                continue;
            }
            let angle = dy.atan2(dx).to_degrees();
            let color = if (-90.0..30.0).contains(&angle) {
                blue
            } else if (30.0..150.0).contains(&angle) {
                teal
            } else {
                indigo
            };
            fills.push(rect_fill(x as f32, y as f32, 1.0, 1.0, color));
        }
    }

    push_circle_fill(fills, cx, cy, diameter * 0.39, background);
    push_circle_fill(
        fills,
        cx,
        cy,
        diameter * 0.30,
        Color { r: 24, g: 88, b: 200, a: 255 },
    );
}

#[derive(Clone, Copy)]
struct RoundedSquareStyle {
    thickness: f32,
    radius: f32,
    color: Color,
    background: Color,
}

fn draw_hollow_rounded_square(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    size: f32,
    style: RoundedSquareStyle,
) {
    if style.radius <= f32::EPSILON {
        draw_hollow_square(fills, x, y, size, style.thickness, style.color);
        return;
    }
    push_rounded_rect_fill(fills, x, y, size, size, style.radius, style.color);
    let inset = style.thickness.max(1.0);
    push_rounded_rect_fill(
        fills,
        x + inset,
        y + inset,
        (size - 2.0 * inset).max(0.0),
        (size - 2.0 * inset).max(0.0),
        (style.radius - inset).max(0.0),
        style.background,
    );
}

fn draw_hollow_rounded_square_top_right_only(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    size: f32,
    style: RoundedSquareStyle,
) {
    if style.radius <= f32::EPSILON {
        draw_hollow_square_top_right_only(fills, x, y, size, style.thickness, style.color);
        return;
    }
    draw_hollow_rounded_square(fills, x, y, size, style);
    let inset = style.thickness.max(1.0);
    fills.push(rect_fill(x, y + size - inset, size, inset, style.background));
    fills.push(rect_fill(x, y, inset, size, style.background));
}

/// 只画方框的上边和右边（用于还原图标中被前框遮挡的后框，露出右上角）。
fn draw_hollow_square_top_right_only(
    fills: &mut Vec<FillPrimitive>,
    x: f32,
    y: f32,
    size: f32,
    thickness: f32,
    color: Color,
) {
    let x = x.round();
    let y = y.round();
    let size = size.round();
    let thickness = thickness.round().max(1.0);
    fills.push(rect_fill(x, y, size, thickness, color));
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
