//! 字体渲染 — 字体加载、Glyph 缓存、字体 fallback

pub mod cache;
mod face_match;
pub mod loader;
pub mod shaper;
pub mod woff;

pub use cache::GlyphCache;
pub use face_match::{NORMAL_FONT_STRETCH, font_face_aliases, resolve_font_face, resolve_font_faces};
pub use loader::FontLoader;
pub use shaper::{
    FontSizeAdjustMetric, FontSizeAdjustment, OpenTypeFeature, ShapedGlyph, ShapedLine, TextDirection, TextShaper,
    measure_text_width,
};
pub use woff::{decode_woff, is_woff};

/// 字体描述
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontDesc {
    /// 字体族名称
    pub family: String,
    /// 字重（100-900，400 = normal，700 = bold）
    pub weight: u16,
    /// 是否斜体
    pub italic: bool,
}

impl FontDesc {
    /// 创建默认字体描述
    pub fn normal(family: &str) -> Self {
        Self {
            family: family.to_string(),
            weight: 400,
            italic: false,
        }
    }

    /// 创建粗体字体描述
    pub fn bold(family: &str) -> Self {
        Self {
            family: family.to_string(),
            weight: 700,
            italic: false,
        }
    }

    /// 创建斜体字体描述
    pub fn italic(family: &str) -> Self {
        Self {
            family: family.to_string(),
            weight: 400,
            italic: true,
        }
    }

    /// 创建自定义字体描述
    pub fn new(family: &str, weight: u16, italic: bool) -> Self {
        Self {
            family: family.to_string(),
            weight,
            italic,
        }
    }
}

/// Glyph 位图
#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    /// 位图数据（灰度，每个字节一个像素）
    pub data: Vec<u8>,
    /// 位图宽度
    pub width: u16,
    /// 位图高度
    pub height: u16,
    /// 水平偏移
    pub x_offset: i16,
    /// 垂直偏移（相对于基线）
    pub y_offset: i16,
    /// 水平前进宽度
    pub advance: f32,
}

/// 字体加载错误
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    /// 字体未找到
    #[error("字体未找到: {0}")]
    NotFound(String),
    /// 字体解析失败
    #[error("字体解析失败: {0}")]
    ParseFailed(String),
    /// Glyph 不存在
    #[error("Glyph 不存在: font_id={font_id}, glyph_id={glyph_id}")]
    GlyphNotFound {
        /// 字体 ID
        font_id: u32,
        /// Glyph ID
        glyph_id: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── FontDesc 测试 ──────────────────────────────────────

    #[test]
    fn test_font_desc_normal() {
        let desc = FontDesc::normal("Arial");
        assert_eq!(desc.family, "Arial");
        assert_eq!(desc.weight, 400);
        assert!(!desc.italic);
    }

    #[test]
    fn test_font_desc_bold() {
        let desc = FontDesc::bold("Helvetica");
        assert_eq!(desc.family, "Helvetica");
        assert_eq!(desc.weight, 700);
        assert!(!desc.italic);
    }

    #[test]
    fn test_font_desc_italic() {
        let desc = FontDesc::italic("Georgia");
        assert_eq!(desc.family, "Georgia");
        assert_eq!(desc.weight, 400);
        assert!(desc.italic);
    }

    #[test]
    fn test_font_desc_new_custom() {
        let desc = FontDesc::new("Roboto", 300, true);
        assert_eq!(desc.family, "Roboto");
        assert_eq!(desc.weight, 300);
        assert!(desc.italic);
    }

    #[test]
    fn test_font_desc_equality() {
        let a = FontDesc::normal("Arial");
        let b = FontDesc::normal("Arial");
        assert_eq!(a, b);

        let c = FontDesc::bold("Arial");
        assert_ne!(a, c);

        let d = FontDesc::normal("Helvetica");
        assert_ne!(a, d);
    }

    #[test]
    fn test_font_desc_clone() {
        let desc = FontDesc::new("Test", 600, true);
        let cloned = desc.clone();
        assert_eq!(desc, cloned);
    }

    #[test]
    fn test_font_desc_debug() {
        let desc = FontDesc::normal("Arial");
        let debug = format!("{:?}", desc);
        assert!(debug.contains("Arial"));
        assert!(debug.contains("400"));
    }

    // ── GlyphBitmap 测试 ──────────────────────────────────

    #[test]
    fn test_glyph_bitmap_fields() {
        let bitmap = GlyphBitmap {
            data: vec![128u8; 100],
            width: 10,
            height: 10,
            x_offset: 2,
            y_offset: -1,
            advance: 12.5,
        };
        assert_eq!(bitmap.data.len(), 100);
        assert_eq!(bitmap.width, 10);
        assert_eq!(bitmap.height, 10);
        assert_eq!(bitmap.x_offset, 2);
        assert_eq!(bitmap.y_offset, -1);
        assert!((bitmap.advance - 12.5).abs() < f32::EPSILON);
    }

    #[test]
    fn test_glyph_bitmap_clone() {
        let bitmap = GlyphBitmap {
            data: vec![255; 4],
            width: 2,
            height: 2,
            x_offset: 0,
            y_offset: 0,
            advance: 3.0,
        };
        let cloned = bitmap.clone();
        assert_eq!(cloned.data, bitmap.data);
        assert_eq!(cloned.width, bitmap.width);
    }

    #[test]
    fn test_glyph_bitmap_debug() {
        let bitmap = GlyphBitmap {
            data: vec![],
            width: 0,
            height: 0,
            x_offset: 0,
            y_offset: 0,
            advance: 0.0,
        };
        let debug = format!("{:?}", bitmap);
        assert!(debug.contains("GlyphBitmap"));
    }

    // ── FontError 测试 ─────────────────────────────────────

    #[test]
    fn test_font_error_not_found() {
        let e = FontError::NotFound("missing.ttf".into());
        assert!(e.to_string().contains("missing.ttf"));
    }

    #[test]
    fn test_font_error_parse_failed() {
        let e = FontError::ParseFailed("corrupt".into());
        assert!(e.to_string().contains("corrupt"));
    }

    #[test]
    fn test_font_error_glyph_not_found() {
        let e = FontError::GlyphNotFound {
            font_id: 1,
            glyph_id: 42,
        };
        let msg = e.to_string();
        assert!(msg.contains("1"));
        assert!(msg.contains("42"));
    }

    #[test]
    fn test_font_error_debug() {
        let e = FontError::NotFound("test".into());
        let debug = format!("{:?}", e);
        assert!(debug.contains("NotFound"));
    }
}
