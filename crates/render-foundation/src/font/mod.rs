//! 字体渲染 — 字体加载、Glyph 缓存、字体 fallback

pub mod cache;
pub mod loader;

pub use cache::GlyphCache;
pub use loader::FontLoader;

/// 字体描述
#[derive(Debug, Clone)]
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
