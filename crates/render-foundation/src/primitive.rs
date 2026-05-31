//! 渲染图元 — 填充矩形、圆角矩形、路径填充、路径描边、裁剪区域、渐变、阴影、图片、Glyph 图元等

use crate::color::Color;
use crate::geometry::Rect;
use crate::image_cache::ImageKey;

/// 填充图元 — 纯色矩形
#[derive(Debug, Clone)]
pub struct FillPrimitive {
    /// 矩形区域
    pub rect: Rect,
    /// 填充颜色
    pub color: Color,
}

/// 圆角矩形图元 — 支持 border-radius 的填充矩形
#[derive(Debug, Clone)]
pub struct RoundedRectPrimitive {
    /// 矩形区域
    pub rect: Rect,
    /// 填充颜色
    pub color: Color,
    /// 左上角圆角半径
    pub top_left_radius: f32,
    /// 右上角圆角半径
    pub top_right_radius: f32,
    /// 右下角圆角半径
    pub bottom_right_radius: f32,
    /// 左下角圆角半径
    pub bottom_left_radius: f32,
}

impl RoundedRectPrimitive {
    /// 创建四个圆角相同的圆角矩形
    pub fn uniform(rect: Rect, color: Color, radius: f32) -> Self {
        Self {
            rect,
            color,
            top_left_radius: radius,
            top_right_radius: radius,
            bottom_right_radius: radius,
            bottom_left_radius: radius,
        }
    }
}

/// 路径填充图元 — 使用路径命令填充任意形状。
#[derive(Debug, Clone)]
pub struct PathFillPrimitive {
    /// 路径命令列表（扁平化的线段序列）。
    /// 每对 f32 表示一个顶点 (x, y)，构成闭合多边形。
    pub vertices: Vec<f32>,
    /// 填充颜色。
    pub color: Color,
}

/// 线段端点样式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineCap {
    /// 平头
    Butt,
    /// 圆头
    Round,
    /// 方头
    Square,
}

/// 描边线型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LineStyle {
    /// 实线
    Solid,
    /// 虚线（线段和间隔交替）
    Dashed,
    /// 点线
    Dotted,
}

/// 路径描边图元 — 使用路径命令描边任意形状。
#[derive(Debug, Clone)]
pub struct PathStrokePrimitive {
    /// 路径命令列表（扁平化的线段序列）。
    /// 每对 f32 表示一个顶点 (x, y)，构成折线/多边形。
    pub vertices: Vec<f32>,
    /// 描边颜色。
    pub color: Color,
    /// 线宽。
    pub line_width: f32,
    /// 是否闭合路径。
    pub closed: bool,
}

/// 描边线段图元 — 用于 border-style dashed/dotted 等单条线段
#[derive(Debug, Clone)]
pub struct StrokePrimitive {
    /// 线段起点
    pub x1: f32,
    /// 线段起点
    pub y1: f32,
    /// 线段终点
    pub x2: f32,
    /// 线段终点
    pub y2: f32,
    /// 线宽
    pub width: f32,
    /// 线条颜色
    pub color: Color,
    /// 线型
    pub style: LineStyle,
    /// 端点样式
    pub cap: LineCap,
}

/// 裁剪图元 — 限制后续绘制到指定矩形区域内
#[derive(Debug, Clone)]
pub struct ClipPrimitive {
    /// 裁剪矩形区域
    pub rect: Rect,
}

/// 渐变停止点
#[derive(Debug, Clone)]
pub struct GradientStop {
    /// 偏移量 [0.0, 1.0]
    pub offset: f32,
    /// 颜色
    pub color: Color,
}

/// 渐变类型
#[derive(Debug, Clone)]
pub enum GradientKind {
    /// 线性渐变：从起点到终点
    Linear {
        /// 起点 X
        x0: f32,
        /// 起点 Y
        y0: f32,
        /// 终点 X
        x1: f32,
        /// 终点 Y
        y1: f32,
    },
    /// 径向渐变：从内圆到外圆
    Radial {
        /// 内圆圆心 X
        cx: f32,
        /// 内圆圆心 Y
        cy: f32,
        /// 内圆半径
        inner_radius: f32,
        /// 外圆半径
        outer_radius: f32,
    },
}

/// 渐变图元 — 线性/径向渐变填充矩形
#[derive(Debug, Clone)]
pub struct GradientPrimitive {
    /// 渐变覆盖的矩形区域
    pub rect: Rect,
    /// 渐变类型
    pub kind: GradientKind,
    /// 颜色停止点列表
    pub stops: Vec<GradientStop>,
}

/// 阴影图元 — box-shadow 效果
#[derive(Debug, Clone)]
pub struct ShadowPrimitive {
    /// 阴影对应的矩形区域
    pub rect: Rect,
    /// 阴影颜色
    pub color: Color,
    /// 水平偏移
    pub offset_x: f32,
    /// 垂直偏移
    pub offset_y: f32,
    /// 模糊半径
    pub blur_radius: f32,
    /// 扩展半径
    pub spread_radius: f32,
}

/// 图片图元 — 在指定矩形区域内绘制图片
#[derive(Debug, Clone)]
pub struct ImagePrimitive {
    /// 目标绘制区域
    pub rect: Rect,
    /// 图片缓存键
    pub image_key: ImageKey,
}

/// Glyph 图元 — 字符渲染
#[derive(Debug, Clone)]
pub struct GlyphPrimitive {
    /// 在表面上的位置（左上角）
    pub x: f32,
    /// 在表面上的位置（基线）
    pub y: f32,
    /// 字体大小（像素）
    pub font_size: f32,
    /// 前景色
    pub color: Color,
    /// Glyph 索引
    pub glyph_id: u32,
    /// 字体 ID
    pub font_id: FontId,
    /// 预缓存的位图宽度（如果已缓存）
    pub bitmap_width: Option<u16>,
    /// 预缓存的位图高度
    pub bitmap_height: Option<u16>,
}

/// 字体 ID 标识
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontId(pub u32);

/// 渲染图元列表 — 由渲染管线生成，供 Backend 消费
#[derive(Debug, Clone, Default)]
pub struct RenderPrimitives {
    /// 裁剪区域列表（绘制其他图元前应应用裁剪）
    pub clips: Vec<ClipPrimitive>,
    /// 填充矩形列表
    pub fills: Vec<FillPrimitive>,
    /// 圆角矩形列表
    pub rounded_rects: Vec<RoundedRectPrimitive>,
    /// 路径填充列表
    pub path_fills: Vec<PathFillPrimitive>,
    /// 路径描边列表
    pub path_strokes: Vec<PathStrokePrimitive>,
    /// 描边线段列表
    pub strokes: Vec<StrokePrimitive>,
    /// 渐变列表
    pub gradients: Vec<GradientPrimitive>,
    /// 阴影列表
    pub shadows: Vec<ShadowPrimitive>,
    /// 图片列表
    pub images: Vec<ImagePrimitive>,
    /// Glyph 列表
    pub glyphs: Vec<GlyphPrimitive>,
}

impl RenderPrimitives {
    /// 创建空的图元列表
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加一个填充矩形
    pub fn add_fill(&mut self, rect: Rect, color: Color) {
        self.fills.push(FillPrimitive { rect, color });
    }

    /// 添加一个圆角矩形
    pub fn add_rounded_rect(&mut self, rounded: RoundedRectPrimitive) {
        self.rounded_rects.push(rounded);
    }

    /// 添加一个路径填充图元。
    pub fn add_path_fill(&mut self, vertices: Vec<f32>, color: Color) {
        self.path_fills.push(PathFillPrimitive { vertices, color });
    }

    /// 添加一个路径描边图元。
    pub fn add_path_stroke(&mut self, vertices: Vec<f32>, color: Color, line_width: f32, closed: bool) {
        self.path_strokes.push(PathStrokePrimitive {
            vertices,
            color,
            line_width,
            closed,
        });
    }

    /// 添加一个描边线段
    pub fn add_stroke(&mut self, stroke: StrokePrimitive) {
        self.strokes.push(stroke);
    }

    /// 添加一个裁剪区域
    pub fn add_clip(&mut self, rect: Rect) {
        self.clips.push(ClipPrimitive { rect });
    }

    /// 添加一个渐变
    pub fn add_gradient(&mut self, gradient: GradientPrimitive) {
        self.gradients.push(gradient);
    }

    /// 添加一个阴影
    pub fn add_shadow(&mut self, shadow: ShadowPrimitive) {
        self.shadows.push(shadow);
    }

    /// 添加一个图片图元
    pub fn add_image(&mut self, image: ImagePrimitive) {
        self.images.push(image);
    }

    /// 添加一个 Glyph
    pub fn add_glyph(&mut self, glyph: GlyphPrimitive) {
        self.glyphs.push(glyph);
    }

    /// 图元总数
    pub fn len(&self) -> usize {
        self.clips.len()
            + self.fills.len()
            + self.rounded_rects.len()
            + self.path_fills.len()
            + self.path_strokes.len()
            + self.strokes.len()
            + self.gradients.len()
            + self.shadows.len()
            + self.images.len()
            + self.glyphs.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.clips.is_empty()
            && self.fills.is_empty()
            && self.rounded_rects.is_empty()
            && self.path_fills.is_empty()
            && self.path_strokes.is_empty()
            && self.strokes.is_empty()
            && self.gradients.is_empty()
            && self.shadows.is_empty()
            && self.images.is_empty()
            && self.glyphs.is_empty()
    }

    /// 获取所有图元的包围盒
    pub fn bounding_box(&self) -> Option<Rect> {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        let mut expand = |left: f32, top: f32, right: f32, bottom: f32| {
            min_x = min_x.min(left);
            min_y = min_y.min(top);
            max_x = max_x.max(right);
            max_y = max_y.max(bottom);
        };

        for fill in &self.fills {
            expand(fill.rect.left(), fill.rect.top(), fill.rect.right(), fill.rect.bottom());
        }

        for rr in &self.rounded_rects {
            expand(rr.rect.left(), rr.rect.top(), rr.rect.right(), rr.rect.bottom());
        }

        for path_fill in &self.path_fills {
            for chunk in path_fill.vertices.chunks_exact(2) {
                expand(chunk[0], chunk[1], chunk[0], chunk[1]);
            }
        }

        for path_stroke in &self.path_strokes {
            for chunk in path_stroke.vertices.chunks_exact(2) {
                expand(chunk[0], chunk[1], chunk[0], chunk[1]);
            }
        }

        for stroke in &self.strokes {
            let half_w = stroke.width / 2.0;
            expand(
                stroke.x1.min(stroke.x2) - half_w,
                stroke.y1.min(stroke.y2) - half_w,
                stroke.x1.max(stroke.x2) + half_w,
                stroke.y1.max(stroke.y2) + half_w,
            );
        }

        for grad in &self.gradients {
            expand(grad.rect.left(), grad.rect.top(), grad.rect.right(), grad.rect.bottom());
        }

        for shadow in &self.shadows {
            let left = shadow.rect.left() + shadow.offset_x - shadow.spread_radius - shadow.blur_radius;
            let top = shadow.rect.top() + shadow.offset_y - shadow.spread_radius - shadow.blur_radius;
            let right = shadow.rect.right() + shadow.offset_x + shadow.spread_radius + shadow.blur_radius;
            let bottom = shadow.rect.bottom() + shadow.offset_y + shadow.spread_radius + shadow.blur_radius;
            expand(left, top, right, bottom);
        }

        for img in &self.images {
            expand(img.rect.left(), img.rect.top(), img.rect.right(), img.rect.bottom());
        }

        for glyph in &self.glyphs {
            expand(glyph.x, glyph.y, glyph.x + glyph.font_size, glyph.y + glyph.font_size);
        }

        if min_x < max_x && min_y < max_y {
            Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Point;

    #[test]
    fn test_primitives_empty() {
        let p = RenderPrimitives::new();
        assert!(p.is_empty());
        assert_eq!(p.len(), 0);
        assert!(p.bounding_box().is_none());
    }

    #[test]
    fn test_primitives_add_fill() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), Color::RED);
        assert!(!p.is_empty());
        assert_eq!(p.fills.len(), 1);
        assert_eq!(p.glyphs.len(), 0);
    }

    #[test]
    fn test_primitives_bounding_box() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(10.0, 20.0, 100.0, 50.0), Color::BLACK);
        p.add_fill(Rect::new(200.0, 100.0, 50.0, 50.0), Color::BLACK);

        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.origin, Point::new(10.0, 20.0));
        // 右边界 250, 下边界 150
        assert_eq!(bb.right(), 250.0);
        assert_eq!(bb.bottom(), 150.0);
    }

    #[test]
    fn test_fill_primitive_fields() {
        let fill = FillPrimitive {
            rect: Rect::new(1.0, 2.0, 3.0, 4.0),
            color: Color::BLUE,
        };
        assert_eq!(fill.rect.origin.x, 1.0);
        assert_eq!(fill.color, Color::BLUE);
    }

    #[test]
    fn test_rounded_rect_uniform() {
        let rr = RoundedRectPrimitive::uniform(Rect::new(0.0, 0.0, 100.0, 50.0), Color::RED, 10.0);
        assert_eq!(rr.top_left_radius, 10.0);
        assert_eq!(rr.top_right_radius, 10.0);
        assert_eq!(rr.bottom_right_radius, 10.0);
        assert_eq!(rr.bottom_left_radius, 10.0);
    }

    #[test]
    fn test_rounded_rect_in_primitives() {
        let mut p = RenderPrimitives::new();
        p.add_rounded_rect(RoundedRectPrimitive::uniform(
            Rect::new(10.0, 10.0, 80.0, 80.0),
            Color::GREEN,
            15.0,
        ));
        assert_eq!(p.rounded_rects.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_stroke_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_stroke(StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 100.0,
            y2: 100.0,
            width: 2.0,
            color: Color::BLACK,
            style: LineStyle::Dashed,
            cap: LineCap::Butt,
        });
        assert_eq!(p.strokes.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_clip_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 200.0, 200.0));
        assert_eq!(p.clips.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_gradient_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 100.0,
                y1: 0.0,
            },
            stops: vec![
                GradientStop {
                    offset: 0.0,
                    color: Color::RED,
                },
                GradientStop {
                    offset: 1.0,
                    color: Color::BLUE,
                },
            ],
        });
        assert_eq!(p.gradients.len(), 1);
    }

    #[test]
    fn test_shadow_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(10.0, 10.0, 80.0, 80.0),
            color: Color::rgba(0, 0, 0, 128),
            offset_x: 4.0,
            offset_y: 4.0,
            blur_radius: 8.0,
            spread_radius: 0.0,
        });
        assert_eq!(p.shadows.len(), 1);
    }

    #[test]
    fn test_image_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_image(ImagePrimitive {
            rect: Rect::new(0.0, 0.0, 50.0, 50.0),
            image_key: ImageKey::new(42),
        });
        assert_eq!(p.images.len(), 1);
    }

    #[test]
    fn test_path_fill_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_path_fill(vec![0.0, 0.0, 50.0, 0.0, 50.0, 50.0, 0.0, 50.0], Color::RED);
        assert_eq!(p.path_fills.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_path_stroke_primitive() {
        let mut p = RenderPrimitives::new();
        p.add_path_stroke(vec![0.0, 0.0, 100.0, 100.0], Color::BLACK, 2.0, false);
        assert_eq!(p.path_strokes.len(), 1);
    }

    #[test]
    fn test_bounding_box_with_rounded_rect() {
        let mut p = RenderPrimitives::new();
        p.add_rounded_rect(RoundedRectPrimitive::uniform(
            Rect::new(5.0, 5.0, 50.0, 50.0),
            Color::BLACK,
            10.0,
        ));
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 5.0);
        assert_eq!(bb.top(), 5.0);
        assert_eq!(bb.right(), 55.0);
        assert_eq!(bb.bottom(), 55.0);
    }

    #[test]
    fn test_bounding_box_with_stroke() {
        let mut p = RenderPrimitives::new();
        p.add_stroke(StrokePrimitive {
            x1: 10.0,
            y1: 20.0,
            x2: 50.0,
            y2: 60.0,
            width: 4.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 8.0); // 10 - 2
        assert_eq!(bb.top(), 18.0); // 20 - 2
        assert_eq!(bb.right(), 52.0); // 50 + 2
        assert_eq!(bb.bottom(), 62.0); // 60 + 2
    }

    #[test]
    fn test_bounding_box_with_shadow() {
        let mut p = RenderPrimitives::new();
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::BLACK,
            offset_x: 5.0,
            offset_y: 5.0,
            blur_radius: 3.0,
            spread_radius: 2.0,
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 10.0);
        assert_eq!(bb.top(), 10.0);
        assert_eq!(bb.right(), 70.0);
        assert_eq!(bb.bottom(), 70.0);
    }

    #[test]
    fn test_len_counts_all_types() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
        p.add_fill(Rect::new(0.0, 0.0, 10.0, 10.0), Color::RED);
        p.add_stroke(StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            width: 1.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        });
        assert!(p.len() >= 3);
    }

    #[test]
    fn test_line_style_equality() {
        assert_eq!(LineStyle::Solid, LineStyle::Solid);
        assert_ne!(LineStyle::Dashed, LineStyle::Dotted);
    }

    #[test]
    fn test_line_cap_equality() {
        assert_eq!(LineCap::Round, LineCap::Round);
        assert_ne!(LineCap::Butt, LineCap::Square);
    }

    #[test]
    fn test_gradient_kind_radial() {
        let kind = GradientKind::Radial {
            cx: 50.0,
            cy: 50.0,
            inner_radius: 0.0,
            outer_radius: 50.0,
        };
        if let GradientKind::Radial { outer_radius, .. } = kind {
            assert_eq!(outer_radius, 50.0);
        } else {
            panic!("Expected Radial");
        }
    }

    #[test]
    fn test_glyph_primitive_creation() {
        let g = GlyphPrimitive {
            x: 10.0,
            y: 20.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 42,
            font_id: FontId(1),
            bitmap_width: Some(12),
            bitmap_height: Some(16),
        };
        assert_eq!(g.x, 10.0);
        assert_eq!(g.font_id, FontId(1));
        assert_eq!(g.bitmap_width, Some(12));
    }

    #[test]
    fn test_glyph_in_render_primitives() {
        let mut p = RenderPrimitives::new();
        p.add_glyph(GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 65,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        assert_eq!(p.glyphs.len(), 1);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_font_id_equality() {
        assert_eq!(FontId(1), FontId(1));
        assert_ne!(FontId(1), FontId(2));
    }

    #[test]
    fn test_bounding_box_with_glyphs() {
        let mut p = RenderPrimitives::new();
        p.add_glyph(GlyphPrimitive {
            x: 5.0,
            y: 10.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: 0,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 5.0);
        assert_eq!(bb.top(), 10.0);
        assert_eq!(bb.right(), 21.0); // x + font_size
        assert_eq!(bb.bottom(), 26.0); // y + font_size
    }

    #[test]
    fn test_bounding_box_with_images() {
        let mut p = RenderPrimitives::new();
        p.add_image(ImagePrimitive {
            rect: Rect::new(50.0, 60.0, 100.0, 80.0),
            image_key: ImageKey::new(1),
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 50.0);
        assert_eq!(bb.top(), 60.0);
        assert_eq!(bb.right(), 150.0);
        assert_eq!(bb.bottom(), 140.0);
    }

    #[test]
    fn test_bounding_box_with_gradient() {
        let mut p = RenderPrimitives::new();
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 200.0, 100.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 200.0,
                y1: 0.0,
            },
            stops: vec![],
        });
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.right(), 200.0);
        assert_eq!(bb.bottom(), 100.0);
    }

    #[test]
    fn test_bounding_box_with_path_fill() {
        let mut p = RenderPrimitives::new();
        p.add_path_fill(vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0], Color::RED);
        let bb = p.bounding_box().unwrap();
        // Points: (10,20), (30,40), (50,60)
        assert_eq!(bb.left(), 10.0);
        assert_eq!(bb.top(), 20.0);
        assert_eq!(bb.right(), 50.0);
        assert_eq!(bb.bottom(), 60.0);
    }

    #[test]
    fn test_render_primitives_mixed_types_count() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
        p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::RED);
        p.add_fill(Rect::new(0.0, 0.0, 50.0, 50.0), Color::BLUE);
        p.add_stroke(StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            width: 1.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Round,
        });
        p.add_glyph(GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 0,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        assert_eq!(p.len(), 5);
        assert!(!p.is_empty());
    }

    #[test]
    fn test_rounded_rect_individual_radii() {
        let rr = RoundedRectPrimitive {
            rect: Rect::new(0.0, 0.0, 100.0, 100.0),
            color: Color::GREEN,
            top_left_radius: 5.0,
            top_right_radius: 10.0,
            bottom_right_radius: 15.0,
            bottom_left_radius: 20.0,
        };
        assert_eq!(rr.top_left_radius, 5.0);
        assert_eq!(rr.top_right_radius, 10.0);
        assert_eq!(rr.bottom_right_radius, 15.0);
        assert_eq!(rr.bottom_left_radius, 20.0);
    }

    // -- 边界条件测试 --
    /// 测试 bounding_box 只包含 clips 时返回 None
    #[test]
    fn test_bounding_box_clips_only_returns_none() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 100.0, 100.0));
        p.add_clip(Rect::new(50.0, 50.0, 100.0, 100.0));
        // clips 不参与 bounding_box 计算
        assert!(p.bounding_box().is_none());
    }

    /// 测试 RenderPrimitives::len 包含所有类型
    #[test]
    fn test_len_all_primitive_types() {
        let mut p = RenderPrimitives::new();
        p.add_clip(Rect::new(0.0, 0.0, 10.0, 10.0));
        p.add_fill(Rect::new(0.0, 0.0, 10.0, 10.0), Color::BLACK);
        p.add_rounded_rect(RoundedRectPrimitive::uniform(
            Rect::new(0.0, 0.0, 10.0, 10.0),
            Color::BLACK,
            5.0,
        ));
        p.add_path_fill(vec![0.0, 0.0, 10.0, 10.0], Color::BLACK);
        p.add_path_stroke(vec![0.0, 0.0, 10.0, 10.0], Color::BLACK, 1.0, false);
        p.add_stroke(StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            width: 1.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        });
        p.add_gradient(GradientPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            kind: GradientKind::Linear {
                x0: 0.0,
                y0: 0.0,
                x1: 10.0,
                y1: 0.0,
            },
            stops: vec![],
        });
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            color: Color::BLACK,
            offset_x: 0.0,
            offset_y: 0.0,
            blur_radius: 0.0,
            spread_radius: 0.0,
        });
        p.add_image(ImagePrimitive {
            rect: Rect::new(0.0, 0.0, 10.0, 10.0),
            image_key: ImageKey::new(0),
        });
        p.add_glyph(GlyphPrimitive {
            x: 0.0,
            y: 0.0,
            font_size: 12.0,
            color: Color::BLACK,
            glyph_id: 0,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
        });
        assert_eq!(p.len(), 10);
    }

    /// 测试 bounding_box 包含负坐标
    #[test]
    fn test_bounding_box_negative_coordinates() {
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(-50.0, -30.0, 100.0, 60.0), Color::BLACK);
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), -50.0);
        assert_eq!(bb.top(), -30.0);
        assert_eq!(bb.right(), 50.0);
        assert_eq!(bb.bottom(), 30.0);
    }

    /// 透明度 alpha=0.0 的图元应不可见（预乘 alpha 后所有通道为零）。
    #[test]
    fn test_composite_primitive_opacity_zero() {
        // alpha=0 的颜色（完全透明），预乘后 RGB 通道全部归零
        let invisible_color = Color::rgba(255, 0, 0, 0);
        let premultiplied = invisible_color.premultiplied();
        assert!(premultiplied[0].abs() < f32::EPSILON, "R 通道预乘后应为 0");
        assert!(premultiplied[1].abs() < f32::EPSILON, "G 通道预乘后应为 0");
        assert!(premultiplied[2].abs() < f32::EPSILON, "B 通道预乘后应为 0");
        assert!(premultiplied[3].abs() < f32::EPSILON, "A 通道预乘后应为 0");

        // 添加一个完全透明的 fill 图元
        let mut p = RenderPrimitives::new();
        p.add_fill(Rect::new(0.0, 0.0, 100.0, 100.0), invisible_color);
        // 图元本身存在（len=1），但颜色完全透明
        assert_eq!(p.fills.len(), 1);
        assert_eq!(p.fills[0].color.a, 0, "alpha 应为 0");

        // 同理：添加一个完全透明的阴影图元
        p.add_shadow(ShadowPrimitive {
            rect: Rect::new(10.0, 10.0, 50.0, 50.0),
            color: Color::TRANSPARENT,
            offset_x: 5.0,
            offset_y: 5.0,
            blur_radius: 3.0,
            spread_radius: 0.0,
        });
        let shadow = &p.shadows[0];
        assert_eq!(shadow.color.a, 0);
        let shadow_premul = shadow.color.premultiplied();
        assert!(shadow_premul.iter().all(|&c| c.abs() < f32::EPSILON));
    }

    /// 测试 path_fill 空 vertices 的 bounding_box
    #[test]
    fn test_bounding_box_empty_path_fill_vertices() {
        let mut p = RenderPrimitives::new();
        p.add_path_fill(vec![], Color::BLACK);
        // Empty vertices means nothing contributes to bounding box
        assert!(p.bounding_box().is_none());
    }

    /// 测试 StrokePrimitive width=0.0
    #[test]
    fn test_stroke_primitive_zero_width() {
        let s = StrokePrimitive {
            x1: 0.0,
            y1: 0.0,
            x2: 10.0,
            y2: 10.0,
            width: 0.0,
            color: Color::BLACK,
            style: LineStyle::Solid,
            cap: LineCap::Butt,
        };
        assert_eq!(s.width, 0.0);

        let mut p = RenderPrimitives::new();
        p.add_stroke(s);
        let bb = p.bounding_box().unwrap();
        assert_eq!(bb.left(), 0.0);
        assert_eq!(bb.top(), 0.0);
        assert_eq!(bb.right(), 10.0);
        assert_eq!(bb.bottom(), 10.0);
    }
}
