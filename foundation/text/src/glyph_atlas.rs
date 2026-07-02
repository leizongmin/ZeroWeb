//! Glyph atlas entry（spec IF-008 `GlyphAtlasEntry`）。
//!
//! 描述单个 glyph 在 atlas 中的位置与度量，供绘制后端采样。

use serde::{Deserialize, Serialize};

/// atlas 中的矩形（像素坐标）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AtlasRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// glyph atlas 条目。
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GlyphAtlasEntry {
    /// 所属 atlas 索引（支持多页 atlas）。
    pub atlas_index: u32,
    pub rect: AtlasRect,
    /// 基线左到 glyph 左边缘的承载偏移。
    pub bearing_x: f32,
    /// 基线到 glyph 顶部的承载偏移。
    pub bearing_y: f32,
    /// 该 glyph 的水平前进量。
    pub advance_x: f32,
    /// 光栅化时的字号（用于验证命中）。
    pub size_px: f32,
}
