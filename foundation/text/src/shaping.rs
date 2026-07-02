//! 文本 shaping（spec IF-008 `TextShaper`）。
//!
//! M1 定义 `ShapeInput`/`ShapedText`/`GlyphRun`/`PositionedGlyph` 与 `TextShaper` trait；
//! 具体 HarfBuzz（rustybuzz）/swash 桥接在 M2。

use crate::diagnostics::TextError;
use crate::font_database::FontMatch;
use crate::font_request::{FontRequest, Script, TextDirection};
use serde::{Deserialize, Serialize};

/// shaping 输入（spec IF-008）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapeInput {
    pub text: String,
    pub font_request: FontRequest,
    pub size_px: f32,
    pub direction: TextDirection,
    pub script: Option<Script>,
    pub scale_factor: f32,
}

/// 单个定位 glyph（坐标为相对基线的前进量/偏移，em 或 px 由实现约定；此处用 px）。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PositionedGlyph {
    pub glyph_id: u32,
    /// 源文本字节偏移簇（用于 hit-test/caret/selection 回映）。
    pub cluster: u32,
    pub x_advance: f32,
    pub y_advance: f32,
    pub x_offset: f32,
    pub y_offset: f32,
}

/// 同一字体的一段 glyph 序列（一个 shape run）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphRun {
    pub font: FontMatch,
    pub font_size_px: f32,
    pub glyphs: Vec<PositionedGlyph>,
}

impl GlyphRun {
    pub fn advance_x(&self) -> f32 {
        self.glyphs.iter().map(|g| g.x_advance).sum()
    }
}

/// shaping 结果（一段文本的 glyph 序列 + 总前进量）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShapedText {
    pub runs: Vec<GlyphRun>,
    pub total_advance_x: f32,
    pub total_advance_y: f32,
}

impl ShapedText {
    pub fn glyph_count(&self) -> usize {
        self.runs.iter().map(|r| r.glyphs.len()).sum()
    }
}

/// 文本 shaper（spec IF-008 `TextShaper`）。
pub trait TextShaper {
    fn shape(&self, input: &ShapeInput) -> Result<ShapedText, TextError>;
}
