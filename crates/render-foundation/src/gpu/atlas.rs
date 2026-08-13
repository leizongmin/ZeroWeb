//! Glyph Atlas — GPU 纹理图集，用于缓存已光栅化的 glyph 位图
//!
//! 基于 OmniTerm 的 GlyphAtlas 设计：
//! - 2048×2048 R8Unorm 纹理
//! - 行式打包（从左到右、从上到下）
//! - 图集满时清空重建（递增 generation 计数器）

use hashbrown::HashMap;

/// Atlas 纹理尺寸（像素）
const ATLAS_SIZE: u32 = 2048;

/// 图集中单个 glyph 的放置信息
#[derive(Debug, Clone)]
pub struct AtlasPlacement {
    /// 纹理中的 X 偏移（像素）
    pub x: u32,
    /// 纹理中的 Y 偏移（像素）
    pub y: u32,
    /// glyph 位图宽度
    pub width: u32,
    /// glyph 位图高度
    pub height: u32,
    /// glyph 的水平偏移
    pub x_offset: i16,
    /// glyph 的垂直偏移（向上为负）
    pub y_offset: i16,
    /// 水平推进宽度
    pub advance: f32,
}

impl AtlasPlacement {
    /// 计算 UV 坐标（带半纹素内缩，避免采样到相邻 glyph）
    pub fn uv(&self) -> (f32, f32, f32, f32) {
        let s = ATLAS_SIZE as f32;
        let half = 0.5 / s;
        (
            self.x as f32 / s + half,
            self.y as f32 / s + half,
            (self.x + self.width) as f32 / s - half,
            (self.y + self.height) as f32 / s - half,
        )
    }
}

/// Atlas 条目键
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GlyphAtlasKey {
    /// 字体 ID
    pub font_id: u32,
    /// 字符码点
    pub codepoint: u32,
    /// 当前字体内部 glyph index；与 Unicode 码点命名空间隔离。
    pub font_glyph_index: Option<u16>,
    /// 字体大小（像素，取整）
    pub size_px: u16,
    /// OpenType axis 坐标；浮点值按 IEEE-754 bits 参与键比较。
    variation_coordinates: Box<[([u8; 4], u32)]>,
}

impl GlyphAtlasKey {
    /// 创建新的 atlas 键
    pub fn new(font_id: u32, codepoint: u32, size_px: f32) -> Self {
        Self::new_with_variations(font_id, codepoint, size_px, &[])
    }

    /// 创建包含 OpenType axis 坐标的 atlas 键。
    pub fn new_with_variations(
        font_id: u32,
        codepoint: u32,
        size_px: f32,
        variations: &[crate::font::OpenTypeVariation],
    ) -> Self {
        Self {
            font_id,
            codepoint,
            font_glyph_index: None,
            size_px: size_px.round() as u16,
            variation_coordinates: variations
                .iter()
                .copied()
                .map(crate::font::OpenTypeVariation::cache_key)
                .collect(),
        }
    }

    /// 创建字体内部 glyph index 的 atlas 键。
    pub fn new_indexed(font_id: u32, glyph_index: u16, size_px: f32) -> Self {
        Self::new_indexed_with_variations(font_id, glyph_index, size_px, &[])
    }

    /// 创建包含 OpenType axis 坐标的字体内部 glyph index atlas 键。
    pub fn new_indexed_with_variations(
        font_id: u32,
        glyph_index: u16,
        size_px: f32,
        variations: &[crate::font::OpenTypeVariation],
    ) -> Self {
        Self {
            font_id,
            codepoint: glyph_index as u32,
            font_glyph_index: Some(glyph_index),
            size_px: size_px.round() as u16,
            variation_coordinates: variations
                .iter()
                .copied()
                .map(crate::font::OpenTypeVariation::cache_key)
                .collect(),
        }
    }
}

/// `place()` 的结果
#[derive(Debug, Clone)]
pub struct PlacementResult {
    /// 放置信息
    pub placement: AtlasPlacement,
    /// 是否是新插入的（需要上传纹理数据）
    pub is_new: bool,
}

/// Glyph Atlas — 在 GPU 纹理中缓存 glyph 位图
pub struct GlyphAtlas {
    /// 条目：glyph 键 → 放置信息
    entries: HashMap<GlyphAtlasKey, AtlasPlacement>,
    /// 当前行中的当前 X 位置
    cursor_x: u32,
    /// 当前行中的当前 Y 位置
    cursor_y: u32,
    /// 当前行的高度（用于换行）
    row_height: u32,
    /// 版本号，每次清空重建时递增
    generation: u64,
    /// 统计：已插入 glyph 数量
    glyph_count: usize,
}

impl GlyphAtlas {
    /// 创建新的空 Glyph Atlas
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            generation: 0,
            glyph_count: 0,
        }
    }

    /// Atlas 纹理尺寸
    pub fn atlas_size() -> u32 {
        ATLAS_SIZE
    }

    /// 当前 generation
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// 已缓存的 glyph 数量
    pub fn glyph_count(&self) -> usize {
        self.glyph_count
    }

    /// 查找已缓存的 glyph
    pub fn get(&self, key: &GlyphAtlasKey) -> Option<&AtlasPlacement> {
        self.entries.get(key)
    }

    /// 尝试在 atlas 中放置一个 glyph 位图。
    ///
    /// 如果 key 已存在，返回现有的放置信息（`is_new = false`）。
    /// 如果 atlas 已满，返回 `None`（调用者应调用 `clear` 后重试）。
    ///
    /// 返回 `PlacementResult` 包含拥有的放置信息。
    pub fn place(
        &mut self,
        key: GlyphAtlasKey,
        width: u32,
        height: u32,
        x_offset: i16,
        y_offset: i16,
        advance: f32,
    ) -> Option<PlacementResult> {
        // 已存在？
        if let Some(placement) = self.entries.get(&key) {
            return Some(PlacementResult {
                placement: placement.clone(),
                is_new: false,
            });
        }

        // 行式打包
        let (px, py) = if self.cursor_x + width <= ATLAS_SIZE {
            (self.cursor_x, self.cursor_y)
        } else {
            // 换行
            let new_y = self.cursor_y + self.row_height;
            if new_y + height > ATLAS_SIZE {
                return None; // Atlas 已满
            }
            self.cursor_y = new_y;
            self.cursor_x = 0;
            self.row_height = 0;
            (0, self.cursor_y)
        };

        let placement = AtlasPlacement {
            x: px,
            y: py,
            width,
            height,
            x_offset,
            y_offset,
            advance,
        };

        self.cursor_x = px + width;
        self.row_height = self.row_height.max(height);
        self.glyph_count += 1;

        self.entries.insert(key, placement.clone());

        Some(PlacementResult {
            placement,
            is_new: true,
        })
    }

    /// 清空 atlas（准备重建）。
    ///
    /// 返回旧的 generation。
    pub fn clear(&mut self) -> u64 {
        let old_gen = self.generation;
        self.entries.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_height = 0;
        self.generation += 1;
        self.glyph_count = 0;
        old_gen
    }

    /// 生成用于创建 wgpu 纹理的描述符
    pub fn texture_descriptor() -> wgpu::TextureDescriptor<'static> {
        wgpu::TextureDescriptor {
            label: Some("Glyph Atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        }
    }

    /// 生成纹理视图描述符
    pub fn view_descriptor() -> wgpu::TextureViewDescriptor<'static> {
        wgpu::TextureViewDescriptor::default()
    }

    /// 计算上传 glyph 位图数据时的行步幅（对齐到 256 字节）
    pub fn row_stride(width: u32) -> wgpu::TexelCopyBufferLayout {
        let bytes_per_row = width;
        let padded = bytes_per_row.next_multiple_of(256).max(256);
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(padded),
            rows_per_image: None,
        }
    }

    /// 为上传 glyph 位图数据创建临时缓冲区（含对齐填充）
    pub fn create_upload_buffer(bitmap_data: &[u8], width: u32) -> Vec<u8> {
        if width == 0 || bitmap_data.is_empty() {
            return Vec::new();
        }

        let bytes_per_row = width.next_multiple_of(256).max(256) as usize;
        let height = bitmap_data.len() / width as usize;
        let mut buf = vec![0u8; bytes_per_row * height];
        for (row, chunk) in bitmap_data.chunks_exact(width as usize).enumerate() {
            buf[row * bytes_per_row..row * bytes_per_row + chunk.len()].copy_from_slice(chunk);
        }
        buf
    }
}

impl Default for GlyphAtlas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atlas_new() {
        let atlas = GlyphAtlas::new();
        assert_eq!(atlas.glyph_count(), 0);
        assert_eq!(atlas.generation(), 0);
    }

    #[test]
    fn test_atlas_place_first_glyph() {
        let mut atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(0, 'A' as u32, 32.0);
        let result = atlas.place(key.clone(), 20, 30, 0, 0, 20.0);

        assert!(result.is_some());
        let r = result.unwrap();
        assert!(r.is_new);
        assert_eq!(r.placement.x, 0);
        assert_eq!(r.placement.y, 0);
        assert_eq!(r.placement.width, 20);
        assert_eq!(r.placement.height, 30);
        assert_eq!(atlas.glyph_count(), 1);
    }

    #[test]
    fn test_atlas_place_same_key_returns_existing() {
        let mut atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(0, 'A' as u32, 32.0);
        atlas.place(key.clone(), 20, 30, 0, 0, 20.0);

        let result = atlas.place(key.clone(), 20, 30, 0, 0, 20.0);
        assert!(result.is_some());
        assert!(!result.unwrap().is_new);
        assert_eq!(atlas.glyph_count(), 1);
    }

    #[test]
    fn test_atlas_place_multiple_glyphs_same_row() {
        let mut atlas = GlyphAtlas::new();
        for i in 0..10u32 {
            let key = GlyphAtlasKey::new(0, 'A' as u32 + i, 16.0);
            let result = atlas.place(key, 10, 16, 0, 0, 10.0);
            assert!(result.is_some());
            let r = result.unwrap();
            assert!(r.is_new);
            assert_eq!(r.placement.y, 0); // 都在同一行
        }
        assert_eq!(atlas.glyph_count(), 10);
    }

    #[test]
    fn test_atlas_place_wraps_to_next_row() {
        let mut atlas = GlyphAtlas::new();
        let key1 = GlyphAtlasKey::new(0, 1, 16.0);
        atlas.place(key1, 2040, 20, 0, 0, 2040.0);

        let key2 = GlyphAtlasKey::new(0, 2, 16.0);
        let result = atlas.place(key2, 10, 20, 0, 0, 10.0);
        assert!(result.is_some());
        assert!(result.unwrap().placement.y > 0);
    }

    #[test]
    fn test_atlas_full_returns_none() {
        let mut atlas = GlyphAtlas::new();
        for i in 0..200u32 {
            let key = GlyphAtlasKey::new(0, i, 16.0);
            if atlas.place(key, 256, 256, 0, 0, 256.0).is_none() {
                return; // Atlas 满了
            }
        }
        panic!("Atlas should have been full before 200 glyphs of 256x256");
    }

    #[test]
    fn test_atlas_clear_resets_state() {
        let mut atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(0, 'A' as u32, 32.0);
        atlas.place(key, 20, 30, 0, 0, 20.0);
        assert_eq!(atlas.glyph_count(), 1);

        let old_gen = atlas.clear();
        assert_eq!(old_gen, 0);
        assert_eq!(atlas.generation(), 1);
        assert_eq!(atlas.glyph_count(), 0);
    }

    #[test]
    fn test_atlas_uv_half_texel_inset() {
        let mut atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(0, 'A' as u32, 32.0);
        let r = atlas.place(key, 100, 100, 0, 0, 100.0).unwrap();

        let (u0, v0, u1, v1) = r.placement.uv();
        assert!(u0 > 0.0 && u0 < 0.001);
        assert!(v0 > 0.0 && v0 < 0.001);
        assert!(u1 > 0.04 && u1 < 0.06);
        assert!(v1 > 0.04 && v1 < 0.06);
    }

    #[test]
    fn test_atlas_texture_descriptor() {
        let desc = GlyphAtlas::texture_descriptor();
        assert_eq!(desc.size.width, 2048);
        assert_eq!(desc.size.height, 2048);
        assert_eq!(desc.format, wgpu::TextureFormat::R8Unorm);
    }

    #[test]
    fn test_atlas_upload_buffer_padding() {
        let data = vec![128u8; 100];
        let buf = GlyphAtlas::create_upload_buffer(&data, 10);
        let bytes_per_row = 10u32.next_multiple_of(256).max(256) as usize;
        assert_eq!(buf.len(), bytes_per_row * 10);
    }

    #[test]
    fn test_atlas_upload_buffer_zero_width_is_empty() {
        let buf = GlyphAtlas::create_upload_buffer(&[], 0);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_atlas_default() {
        let atlas = GlyphAtlas::default();
        assert_eq!(atlas.glyph_count(), 0);
        assert_eq!(atlas.generation(), 0);
    }

    #[test]
    fn test_atlas_key_creation() {
        let key = GlyphAtlasKey::new(1, 65, 32.0);
        assert_eq!(key.font_id, 1);
        assert_eq!(key.codepoint, 65);
        assert_eq!(key.size_px, 32);
    }

    #[test]
    fn test_atlas_key_size_rounding() {
        let key = GlyphAtlasKey::new(0, 65, 16.7);
        assert_eq!(key.size_px, 17); // round(16.7) = 17
    }

    #[test]
    fn test_atlas_place_then_clear_then_place() {
        let mut atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(0, 'A' as u32, 32.0);
        atlas.place(key.clone(), 20, 30, 0, 0, 20.0);
        assert_eq!(atlas.glyph_count(), 1);

        let old_gen = atlas.clear();
        assert_eq!(old_gen, 0);
        assert_eq!(atlas.glyph_count(), 0);

        // Same key can be placed again after clear
        let result = atlas.place(key, 20, 30, 0, 0, 20.0);
        assert!(result.is_some());
        assert!(result.unwrap().is_new);
        assert_eq!(atlas.glyph_count(), 1);
    }

    #[test]
    fn test_atlas_get_existing() {
        let mut atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(0, 'Z' as u32, 16.0);
        atlas.place(key.clone(), 10, 12, 0, 0, 10.0);

        let placement = atlas.get(&key);
        assert!(placement.is_some());
        assert_eq!(placement.unwrap().width, 10);
        assert_eq!(placement.unwrap().height, 12);
    }

    #[test]
    fn test_atlas_get_nonexistent() {
        let atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(0, 'X' as u32, 16.0);
        assert!(atlas.get(&key).is_none());
    }

    #[test]
    fn test_atlas_row_stride_alignment() {
        let layout = GlyphAtlas::row_stride(10);
        assert_eq!(layout.offset, 0);
        assert!(layout.bytes_per_row.unwrap() >= 256);
        assert!(layout.bytes_per_row.unwrap().is_multiple_of(256));
    }

    #[test]
    fn test_atlas_view_descriptor() {
        let desc = GlyphAtlas::view_descriptor();
        assert!(desc.label.is_none());
    }

    #[test]
    fn test_placement_advance_preserved() {
        let mut atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(0, 'A' as u32, 16.0);
        let result = atlas.place(key, 10, 10, 0, 0, 15.5);
        let p = result.unwrap();
        assert!((p.placement.advance - 15.5).abs() < f32::EPSILON);
    }

    /// 测试 atlas 坐标映射：连续放置的 glyph 的 x 坐标单调递增
    ///
    /// 验证同一行内 glyph 的 x 坐标严格递增（行式打包）。
    #[test]
    fn test_atlas_coordinates_monotonically_increase() {
        let mut atlas = GlyphAtlas::new();
        let mut prev_x = 0u32;

        for i in 0..20u32 {
            let key = GlyphAtlasKey::new(0, i + 100, 16.0);
            let result = atlas.place(key, 8, 8, 0, 0, 8.0).unwrap();
            if i > 0 {
                assert!(
                    result.placement.x > prev_x || result.placement.y > 0,
                    "同一行 x 应递增或换行"
                );
            }
            prev_x = result.placement.x;
        }
    }

    /// 测试 atlas 放置完整图集后返回 None
    ///
    /// 使用超大尺寸填满 atlas，验证 place 返回 None。
    #[test]
    fn test_atlas_full_with_single_large_glyph() {
        let mut atlas = GlyphAtlas::new();
        // 放一个 2048x2048 的 glyph 恰好占满整个 atlas
        let key = GlyphAtlasKey::new(0, 1, 16.0);
        let result = atlas.place(key, 2048, 2048, 0, 0, 2048.0);
        assert!(result.is_some(), "2048x2048 应刚好放得下");
        let r = result.unwrap();
        assert_eq!(r.placement.x, 0);
        assert_eq!(r.placement.y, 0);

        // 再放一个应失败
        let key2 = GlyphAtlasKey::new(0, 2, 16.0);
        let result2 = atlas.place(key2, 1, 1, 0, 0, 1.0);
        assert!(result2.is_none(), "图集已满应返回 None");
    }

    /// 测试 atlas clear 递增 generation
    ///
    /// 连续清空 atlas 多次，验证 generation 每次递增。
    #[test]
    fn test_atlas_clear_generation_increments() {
        let mut atlas = GlyphAtlas::new();
        assert_eq!(atlas.generation(), 0);

        atlas.clear();
        assert_eq!(atlas.generation(), 1);

        atlas.clear();
        assert_eq!(atlas.generation(), 2);

        atlas.clear();
        assert_eq!(atlas.generation(), 3);
    }

    /// 测试 atlas 放置后通过 get 查找的一致性
    ///
    /// 放置 glyph 后，通过 get 查找应返回完全一致的放置信息。
    #[test]
    fn test_atlas_place_and_get_consistency() {
        let mut atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(5, 'M' as u32, 20.0);
        let result = atlas.place(key.clone(), 15, 18, -2, 3, 12.0).unwrap();

        let got = atlas.get(&key).expect("应能查找到已放置的 glyph");
        assert_eq!(got.x, result.placement.x);
        assert_eq!(got.y, result.placement.y);
        assert_eq!(got.width, 15);
        assert_eq!(got.height, 18);
        assert_eq!(got.x_offset, -2);
        assert_eq!(got.y_offset, 3);
        assert!((got.advance - 12.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_unicode_and_indexed_atlas_keys_do_not_collide() {
        let unicode = GlyphAtlasKey::new(5, 65, 20.0);
        let indexed = GlyphAtlasKey::new_indexed(5, 65, 20.0);

        assert_ne!(unicode, indexed);
    }

    #[test]
    fn variation_coordinates_isolate_atlas_keys() {
        let condensed = GlyphAtlasKey::new_indexed_with_variations(
            5,
            65,
            20.0,
            &[crate::font::OpenTypeVariation::new(*b"wdth", 75.0)],
        );
        let expanded = GlyphAtlasKey::new_indexed_with_variations(
            5,
            65,
            20.0,
            &[crate::font::OpenTypeVariation::new(*b"wdth", 125.0)],
        );

        assert_ne!(condensed, expanded);
    }

    /// 测试 atlas 放置 0 高度 glyph 仍然成功
    ///
    /// 高度为 0 的 glyph 应被放置在 atlas 中，不导致 panic。
    #[test]
    fn test_atlas_place_zero_height_glyph() {
        let mut atlas = GlyphAtlas::new();
        let key = GlyphAtlasKey::new(0, 1, 16.0);
        let result = atlas.place(key, 10, 0, 0, 0, 10.0);
        assert!(result.is_some(), "0 高度 glyph 应能放置");
        assert_eq!(result.unwrap().placement.height, 0);
    }
}
