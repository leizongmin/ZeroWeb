//! 渲染图元 — 填充矩形、Glyph 图元等

use crate::color::Color;
use crate::geometry::Rect;

/// 填充图元 — 纯色矩形
#[derive(Debug, Clone)]
pub struct FillPrimitive {
    /// 矩形区域
    pub rect: Rect,
    /// 填充颜色
    pub color: Color,
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
    /// 填充矩形列表
    pub fills: Vec<FillPrimitive>,
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

    /// 添加一个 Glyph
    pub fn add_glyph(&mut self, glyph: GlyphPrimitive) {
        self.glyphs.push(glyph);
    }

    /// 图元总数
    pub fn len(&self) -> usize {
        self.fills.len() + self.glyphs.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.fills.is_empty() && self.glyphs.is_empty()
    }

    /// 获取所有图元的包围盒
    pub fn bounding_box(&self) -> Option<Rect> {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for fill in &self.fills {
            min_x = min_x.min(fill.rect.left());
            min_y = min_y.min(fill.rect.top());
            max_x = max_x.max(fill.rect.right());
            max_y = max_y.max(fill.rect.bottom());
        }

        for glyph in &self.glyphs {
            // Glyph 位置是左上角，假设最大尺寸
            min_x = min_x.min(glyph.x);
            min_y = min_y.min(glyph.y);
            max_x = max_x.max(glyph.x + glyph.font_size);
            max_y = max_y.max(glyph.y + glyph.font_size);
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
        p.add_fill(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            Color::RED,
        );
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
}
