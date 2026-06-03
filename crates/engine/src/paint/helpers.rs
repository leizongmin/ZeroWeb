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
            font_id: FontId(1),
            bitmap_width: None,
            bitmap_height: None,
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
            font_id: FontId(1),
            bitmap_width: None,
            bitmap_height: None,
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
                font_id: FontId(1),
                bitmap_width: None,
                bitmap_height: None,
            },
            GlyphPrimitive {
                x: 10.0,
                y: 10.0,
                font_size: 16.0,
                color: Color::rgb(0, 0, 0),
                glyph_id: 2,
                font_id: FontId(1),
                bitmap_width: None,
                bitmap_height: None,
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
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
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
            direction: GradientDirection::ToBottom,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(255, 0, 0, 255),
                position: None,
            }],
            repeating: false,
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect).expect("linear gradient should convert");
        assert_eq!(prim.rect, rect);
        assert!(matches!(prim.kind, GradientKind::Linear { .. }));
        assert_eq!(prim.stops.len(), 1);
    }

    #[test]
    fn test_radial_gradient_to_primitive() {
        let grad = GradientValue::Radial(RadialGradient {
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
        let prim = gradient_to_primitive(&grad, &rect).expect("radial gradient should convert");
        assert!(matches!(prim.kind, GradientKind::Radial { .. }));
    }

    #[test]
    fn test_conic_gradient_returns_none() {
        let grad = GradientValue::Conic(ConicGradient {
            repeating: false,
            from_angle: 0.0,
            position_x: LengthValue::Percentage(50.0),
            position_y: LengthValue::Percentage(50.0),
            stops: vec![],
        });
        let rect = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(gradient_to_primitive(&grad, &rect).is_none());
    }

    #[test]
    fn test_radial_closest_side() {
        // Use Px values since length_to_f32 only handles Px
        // cx = rect.left() + 50/100*200 = 0 + 100 = 100
        // cy = rect.top() + 50/100*100 = 0 + 50 = 50
        let grad = GradientValue::Radial(RadialGradient {
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Px(50.0),
            position_y: LengthValue::Px(50.0),
            size: RadialSize::ClosestSide,
            repeating: false,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            }],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect).unwrap();
        if let GradientKind::Radial {
            cx, cy, outer_radius, ..
        } = prim.kind
        {
            // cx = 0 + 50/100*200 = 100, cy = 0 + 50/100*100 = 50
            assert!((cx - 100.0).abs() < 0.1);
            assert!((cy - 50.0).abs() < 0.1);
            // closest side from (100, 50): min(100, 100, 50, 50) = 50
            assert!((outer_radius - 50.0).abs() < 0.1);
        } else {
            panic!("expected radial gradient");
        }
    }

    #[test]
    fn test_radial_farthest_side() {
        // cx = 0 + 50/100*200 = 100, cy = 0 + 50/100*100 = 50
        let grad = GradientValue::Radial(RadialGradient {
            shape: RadialShape::Ellipse,
            position_x: LengthValue::Px(50.0),
            position_y: LengthValue::Px(50.0),
            size: RadialSize::FarthestSide,
            repeating: false,
            stops: vec![GradientColorStop {
                color: ColorValue::Rgba(0, 0, 0, 255),
                position: None,
            }],
        });
        let rect = Rect::new(0.0, 0.0, 200.0, 100.0);
        let prim = gradient_to_primitive(&grad, &rect).unwrap();
        if let GradientKind::Radial { outer_radius, .. } = prim.kind {
            // center at (100, 50), farthest side = max(100, 100, 50, 50) = 100
            assert!((outer_radius - 100.0).abs() < 0.1);
        } else {
            panic!("expected radial gradient");
        }
    }

    #[test]
    fn test_radial_length_size() {
        let grad = GradientValue::Radial(RadialGradient {
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
        let prim = gradient_to_primitive(&grad, &rect).unwrap();
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
        let result = convert_color_stops(&stops);
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
        let result = convert_color_stops(&stops);
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
        let result = convert_color_stops(&stops);
        assert_eq!(result.len(), 1);
        assert!((result[0].offset - 0.0).abs() < 0.01);
    }
}
