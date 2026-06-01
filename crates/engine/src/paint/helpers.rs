//! 辅助工具 — 变换偏移、裁剪、opacity 应用、渐变转换等。

use zero_css_parser::values::{
    GradientColorStop, GradientDirection, GradientValue, LengthValue, RadialSize, TransformFunction, TransformValue,
};
use zero_render_foundation::geometry::Rect;
use zero_render_foundation::primitive::{GradientKind, GradientPrimitive, GradientStop, RenderPrimitives};
use zero_style_system::{ComputedStyle, TextTransformValue};

use super::color::color_value_to_render;

/// 从 ComputedStyle 的 transform 计算偏移量。
///
/// 返回 (dx, dy) 偏移，用于调整图元位置。
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
pub fn apply_text_transform(text: &str, transform: &TextTransformValue) -> String {
    match transform {
        TextTransformValue::None => text.to_string(),
        TextTransformValue::Uppercase => text.to_uppercase(),
        TextTransformValue::Lowercase => text.to_lowercase(),
        TextTransformValue::Capitalize => {
            let mut result = String::with_capacity(text.len());
            let mut prev_is_boundary = true;
            for ch in text.chars() {
                if prev_is_boundary && ch.is_alphabetic() {
                    for c in ch.to_uppercase() {
                        result.push(c);
                    }
                } else {
                    result.push(ch);
                }
                prev_is_boundary = !ch.is_alphanumeric();
            }
            result
        }
    }
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

    /// 所有圆角都为零。
    pub fn is_zero(&self) -> bool {
        self.top_left == 0.0 && self.top_right == 0.0 && self.bottom_right == 0.0 && self.bottom_left == 0.0
    }
}

/// 将 LengthValue 转换为 f32（仅支持 Px）。
pub fn length_to_f32(v: &LengthValue) -> f32 {
    match v {
        LengthValue::Px(p) => *p as f32,
        _ => 0.0,
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

/// 将 CSS GradientValue 转换为 GradientPrimitive。
///
/// 目前支持 linear-gradient 和 radial-gradient。
/// conic-gradient 暂不渲染（返回 None）。
pub fn gradient_to_primitive(gradient: &GradientValue, rect: &Rect) -> Option<GradientPrimitive> {
    let w = rect.size.width;
    let h = rect.size.height;
    match gradient {
        GradientValue::Linear(lg) => {
            let kind = linear_direction_to_kind(&lg.direction, rect);
            let stops = convert_color_stops(&lg.stops);
            Some(GradientPrimitive {
                rect: *rect,
                kind,
                stops,
            })
        }
        GradientValue::Radial(rg) => {
            let cx = rect.left() + length_to_f32(&rg.position_x) / 100.0 * w;
            let cy = rect.top() + length_to_f32(&rg.position_y) / 100.0 * h;
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
            let stops = convert_color_stops(&rg.stops);
            Some(GradientPrimitive {
                rect: *rect,
                kind: GradientKind::Radial {
                    cx,
                    cy,
                    inner_radius: 0.0,
                    outer_radius: outer.max(0.01),
                },
                stops,
            })
        }
        GradientValue::Conic(_) => {
            // conic-gradient 暂不支持渲染
            None
        }
    }
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
pub fn convert_color_stops(stops: &[GradientColorStop]) -> Vec<GradientStop> {
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
                    _ => 0.0,
                })
                .unwrap_or(if n <= 1 { 0.0 } else { i as f32 / (n - 1) as f32 });
            GradientStop {
                offset,
                color: color_value_to_render(&s.color),
            }
        })
        .collect()
}
