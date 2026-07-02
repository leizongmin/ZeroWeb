//! Glyph cache（spec IF-008 `GlyphCache`）。
//!
//! 同 FontId + glyph_id + size + subpixel 的 glyph 在 atlas 中只光栅化一次（DC-11 复用）。
//! M1 定义 `GlyphKey`/`GlyphCache` trait 与一个最小内存实现（无真实光栅，便于单测）；
//! M2 接入 render-foundation 的真实 glyph atlas + LRU eviction。

use crate::diagnostics::TextError;
use crate::font_request::FontId;
use crate::glyph_atlas::{AtlasRect, GlyphAtlasEntry};
use hashbrown::HashMap;

/// glyph 缓存键（决定 atlas 中是否复用同一 entry）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphKey {
    pub font_id: FontId,
    pub glyph_id: u32,
    /// 字号（量化到 1/4 px 以提高命中率；由调用方或实现负责）。
    pub size_q8: u32,
    /// 子像素偏移（0..3，对应 4 档）。
    pub subpixel_x: u8,
    pub subpixel_y: u8,
}

impl GlyphKey {
    pub fn new(font_id: FontId, glyph_id: u32, size_q8: u32) -> GlyphKey {
        GlyphKey {
            font_id,
            glyph_id,
            size_q8,
            subpixel_x: 0,
            subpixel_y: 0,
        }
    }
}

/// glyph cache（spec IF-008 `GlyphCache`）。
pub trait GlyphCache {
    fn get_or_insert(&mut self, glyph: GlyphKey) -> Result<GlyphAtlasEntry, TextError>;
    fn contains(&self, glyph: &GlyphKey) -> bool;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// M1 最小内存 cache：首次插入分配递增 atlas rect，再次命中直接返回（无光栅、无驱逐）。
/// 仅供单测与接口验证；真实 atlas 在 `ui/render` / render-foundation。
#[derive(Debug, Default)]
pub struct InMemoryGlyphCache {
    entries: HashMap<GlyphKey, GlyphAtlasEntry>,
    next_index: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
}

const ATLAS_TILE: u32 = 64;
const ATLAS_WIDTH: u32 = 1024;

impl InMemoryGlyphCache {
    pub fn new() -> InMemoryGlyphCache {
        InMemoryGlyphCache::default()
    }

    fn allocate(&mut self) -> AtlasRect {
        if self.cursor_x + ATLAS_TILE > ATLAS_WIDTH {
            self.cursor_x = 0;
            self.cursor_y += self.row_height.max(ATLAS_TILE);
            self.row_height = 0;
        }
        let rect = AtlasRect {
            x: self.cursor_x,
            y: self.cursor_y,
            w: ATLAS_TILE,
            h: ATLAS_TILE,
        };
        self.cursor_x += ATLAS_TILE;
        self.row_height = self.row_height.max(ATLAS_TILE);
        rect
    }
}

impl GlyphCache for InMemoryGlyphCache {
    fn get_or_insert(&mut self, glyph: GlyphKey) -> Result<GlyphAtlasEntry, TextError> {
        if let Some(entry) = self.entries.get(&glyph) {
            return Ok(*entry);
        }
        let index = self.next_index;
        self.next_index += 1;
        let rect = self.allocate();
        let entry = GlyphAtlasEntry {
            atlas_index: index,
            rect,
            bearing_x: 0.0,
            bearing_y: 0.0,
            advance_x: 0.0,
            size_px: 0.0,
        };
        self.entries.insert(glyph, entry);
        Ok(entry)
    }

    fn contains(&self, glyph: &GlyphKey) -> bool {
        self.entries.contains_key(glyph)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_key_reuses_atlas_entry() {
        // DC-11：同 key 的 glyph 复用同一 atlas entry（不重复分配）。
        let mut cache = InMemoryGlyphCache::new();
        let key = GlyphKey::new(FontId(1), 42, 14 * 256 / 14);
        let e1 = cache.get_or_insert(key).unwrap();
        let e2 = cache.get_or_insert(key).unwrap();
        assert_eq!(e1.atlas_index, e2.atlas_index);
        assert_eq!(e1.rect, e2.rect);
        assert_eq!(cache.len(), 1, "same key must not allocate twice");
    }

    #[test]
    fn different_keys_allocate_distinct_rects() {
        let mut cache = InMemoryGlyphCache::new();
        let a = cache.get_or_insert(GlyphKey::new(FontId(1), 1, 14)).unwrap();
        let b = cache.get_or_insert(GlyphKey::new(FontId(1), 2, 14)).unwrap();
        assert_ne!(a.rect, b.rect);
        assert_eq!(cache.len(), 2);
    }
}
