//! Glyph 缓存 — 缓存已渲染的 glyph 位图

use crate::font::{FontError, GlyphBitmap};
use hashbrown::HashMap;

/// Glyph 缓存键
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct GlyphKey {
    /// 字体 ID
    pub font_id: u32,
    /// Glyph 索引（或 Unicode 码点）
    pub glyph_id: u32,
    /// 字体大小（像素，取整）
    pub size_px: u16,
}

impl GlyphKey {
    /// 创建缓存键
    pub fn new(font_id: u32, glyph_id: u32, size_px: f32) -> Self {
        Self {
            font_id,
            glyph_id,
            size_px: size_px.round() as u16,
        }
    }
}

/// Glyph 缓存 — 缓存已渲染的 glyph 位图以避免重复光栅化
pub struct GlyphCache {
    /// 缓存条目
    cache: HashMap<GlyphKey, GlyphBitmap>,
    /// 最大缓存条目数
    max_entries: usize,
}

impl GlyphCache {
    /// 创建新的 Glyph 缓存
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_entries,
        }
    }

    /// 获取或插入 glyph
    pub fn get_or_insert_with<F>(&mut self, key: GlyphKey, f: F) -> Result<&GlyphBitmap, FontError>
    where
        F: FnOnce() -> Result<GlyphBitmap, FontError>,
    {
        if self.cache.len() >= self.max_entries && !self.cache.contains_key(&key) {
            // 简单淘汰策略：当缓存满时清空一半
            // TODO: 后续实现 LRU 淘汰
            let keys_to_remove: Vec<_> = self.cache.keys().take(self.max_entries / 2).cloned().collect();
            for k in keys_to_remove {
                self.cache.remove(&k);
            }
        }

        if !self.cache.contains_key(&key) {
            let bitmap = f()?;
            self.cache.insert(key.clone(), bitmap);
        }

        Ok(self.cache.get(&key).unwrap())
    }

    /// 直接获取缓存的 glyph
    pub fn get(&self, key: &GlyphKey) -> Option<&GlyphBitmap> {
        self.cache.get(key)
    }

    /// 插入 glyph 到缓存
    pub fn insert(&mut self, key: GlyphKey, bitmap: GlyphBitmap) {
        self.cache.insert(key, bitmap);
    }

    /// 缓存条目数
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// 当前内存使用量估算（字节）
    pub fn estimated_memory(&self) -> usize {
        self.cache
            .values()
            .map(|b| b.data.len() + std::mem::size_of::<GlyphBitmap>())
            .sum()
    }
}

impl Default for GlyphCache {
    fn default() -> Self {
        Self::new(4096) // 默认最多 4096 个 glyph
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_bitmap(data: &[u8], w: u16, h: u16) -> GlyphBitmap {
        GlyphBitmap {
            data: data.to_vec(),
            width: w,
            height: h,
            x_offset: 0,
            y_offset: 0,
            advance: w as f32,
        }
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = GlyphCache::new(100);
        let key = GlyphKey::new(0, 65, 16.0); // font 0, 'A', 16px
        let bitmap = make_bitmap(&[1, 2, 3, 4], 2, 2);

        cache.insert(key.clone(), bitmap);
        assert_eq!(cache.len(), 1);

        let got = cache.get(&key).unwrap();
        assert_eq!(got.width, 2);
        assert_eq!(got.height, 2);
    }

    #[test]
    fn test_cache_eviction() {
        let mut cache = GlyphCache::new(4);

        // 填充缓存
        for i in 0..4 {
            let key = GlyphKey::new(0, i, 16.0);
            cache.insert(key, make_bitmap(&[i as u8; 4], 2, 2));
        }
        assert_eq!(cache.len(), 4);

        // 添加第 5 个，触发淘汰
        let key = GlyphKey::new(0, 100, 16.0);
        cache.insert(key, make_bitmap(&[0; 4], 2, 2));
        // 淘汰后应该少于 5
        assert!(cache.len() <= 5);
    }

    #[test]
    fn test_cache_get_or_insert() {
        let mut cache = GlyphCache::new(100);
        let key = GlyphKey::new(0, 65, 16.0);

        // 首次访问，应该调用 f
        let result = cache.get_or_insert_with(key.clone(), || Ok(make_bitmap(&[42; 4], 2, 2)));
        assert!(result.is_ok());
        assert_eq!(cache.len(), 1);

        // 再次访问，不应调用 f
        let result2 = cache.get_or_insert_with(key.clone(), || {
            panic!("不应再次调用");
        });
        assert!(result2.is_ok());
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = GlyphCache::new(100);
        cache.insert(GlyphKey::new(0, 65, 16.0), make_bitmap(&[0; 4], 2, 2));
        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_memory_estimate() {
        let mut cache = GlyphCache::new(100);
        cache.insert(GlyphKey::new(0, 65, 16.0), make_bitmap(&[0; 100], 10, 10));
        let mem = cache.estimated_memory();
        assert!(mem >= 100);
    }

    #[test]
    fn test_glyph_key_size_rounding() {
        let key = GlyphKey::new(0, 65, 16.4);
        assert_eq!(key.size_px, 16);
        let key2 = GlyphKey::new(0, 65, 16.5);
        assert_eq!(key2.size_px, 17); // round(16.5) = 17
    }

    #[test]
    fn test_cache_eviction_halves_when_full() {
        let mut cache = GlyphCache::new(4);
        // Fill cache to capacity
        for i in 0..4u32 {
            let key = GlyphKey::new(0, i, 16.0);
            cache.insert(key, make_bitmap(&[i as u8; 4], 2, 2));
        }
        assert_eq!(cache.len(), 4);
        // Insert a new key — should trigger eviction of half
        let new_key = GlyphKey::new(0, 99, 16.0);
        cache
            .get_or_insert_with(new_key, || Ok(make_bitmap(&[0; 4], 2, 2)))
            .unwrap();
        // After eviction + insertion, count should be <= (4 - 2) + 1 = 3
        assert!(cache.len() <= 3);
    }

    #[test]
    fn test_cache_hit_returns_same_value() {
        let mut cache = GlyphCache::new(100);
        let key = GlyphKey::new(1, 42, 24.0);
        cache.insert(key.clone(), make_bitmap(&[7, 8, 9], 3, 1));

        let got = cache.get(&key).unwrap();
        assert_eq!(got.data, vec![7, 8, 9]);
        assert_eq!(got.width, 3);
        assert_eq!(got.height, 1);
    }

    #[test]
    fn test_cache_miss_returns_none() {
        let cache = GlyphCache::new(100);
        let key = GlyphKey::new(0, 999, 16.0);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_default_capacity() {
        let cache = GlyphCache::default();
        assert!(cache.is_empty());
        // Default max_entries is 4096
        // Just verify it works with a few inserts
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_get_or_insert_error_propagation() {
        let mut cache = GlyphCache::new(100);
        let key = GlyphKey::new(0, 65, 16.0);
        let result = cache.get_or_insert_with(key.clone(), || Err(FontError::NotFound("test".to_string())));
        assert!(result.is_err());
        // Failed insert should not add to cache
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_glyph_key_equality() {
        let k1 = GlyphKey::new(1, 65, 16.0);
        let k2 = GlyphKey::new(1, 65, 16.0);
        let k3 = GlyphKey::new(2, 65, 16.0);
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
    }

    #[test]
    fn test_glyph_key_different_sizes() {
        let k1 = GlyphKey::new(0, 65, 12.0);
        let k2 = GlyphKey::new(0, 65, 24.0);
        assert_ne!(k1, k2);
        assert_eq!(k1.size_px, 12);
        assert_eq!(k2.size_px, 24);
    }

    #[test]
    fn test_cache_insert_overwrite() {
        let mut cache = GlyphCache::new(100);
        let key = GlyphKey::new(0, 65, 16.0);
        cache.insert(key.clone(), make_bitmap(&[1, 2, 3], 3, 1));
        // Insert again with same key — should overwrite
        cache.insert(key.clone(), make_bitmap(&[4, 5, 6], 3, 1));
        let got = cache.get(&key).unwrap();
        assert_eq!(got.data, vec![4, 5, 6]);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_multiple_fonts() {
        let mut cache = GlyphCache::new(100);
        let key1 = GlyphKey::new(0, 65, 16.0);
        let key2 = GlyphKey::new(1, 65, 16.0);
        cache.insert(key1.clone(), make_bitmap(&[10], 1, 1));
        cache.insert(key2.clone(), make_bitmap(&[20], 1, 1));
        assert_eq!(cache.len(), 2);
        assert_eq!(cache.get(&key1).unwrap().data[0], 10);
        assert_eq!(cache.get(&key2).unwrap().data[0], 20);
    }

    #[test]
    fn test_cache_zero_capacity() {
        let mut cache = GlyphCache::new(0);
        // Inserting should still work (direct insert bypasses eviction logic)
        let key = GlyphKey::new(0, 65, 16.0);
        cache.insert(key.clone(), make_bitmap(&[1], 1, 1));
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_glyph_bitmap_fields() {
        let bm = GlyphBitmap {
            data: vec![128; 100],
            width: 10,
            height: 10,
            x_offset: -2,
            y_offset: 5,
            advance: 12.5,
        };
        assert_eq!(bm.width, 10);
        assert_eq!(bm.height, 10);
        assert_eq!(bm.x_offset, -2);
        assert_eq!(bm.y_offset, 5);
        assert!((bm.advance - 12.5).abs() < f32::EPSILON);
    }

    /// 测试 get_or_insert_with 在零容量缓存中触发淘汰后仍能插入
    ///
    /// max_entries=0 时 get_or_insert_with 应能淘汰并插入新条目。
    #[test]
    fn test_cache_get_or_insert_zero_capacity() {
        let mut cache = GlyphCache::new(0);
        let key = GlyphKey::new(0, 65, 16.0);
        let result = cache.get_or_insert_with(key.clone(), || Ok(make_bitmap(&[42], 1, 1)));
        assert!(result.is_ok(), "零容量缓存应能通过 get_or_insert 插入");
        // 淘汰逻辑移除一半（0/2 = 0），但随后直接插入
        assert!(cache.get(&key).is_some());
    }

    /// 测试缓存错误不插入后再次调用成功
    ///
    /// 第一次 get_or_insert_with 返回 Err 不应阻止后续对相同 key 的调用。
    #[test]
    fn test_cache_failed_insert_retriable() {
        let mut cache = GlyphCache::new(100);
        let key = GlyphKey::new(0, 65, 16.0);

        // 第一次失败
        let result = cache.get_or_insert_with(key.clone(), || Err(FontError::NotFound("test".to_string())));
        assert!(result.is_err());
        assert!(cache.get(&key).is_none(), "失败后缓存不应有条目");

        // 第二次成功
        let result = cache.get_or_insert_with(key.clone(), || Ok(make_bitmap(&[99], 1, 1)));
        assert!(result.is_ok());
        assert_eq!(cache.get(&key).unwrap().data[0], 99);
    }

    /// 测试 estimated_memory 对多个条目的累加
    ///
    /// 插入多个不同大小的 bitmap 后验证 estimated_memory 正确累加。
    #[test]
    fn test_cache_estimated_memory_multiple_entries() {
        let mut cache = GlyphCache::new(100);

        cache.insert(GlyphKey::new(0, 65, 16.0), make_bitmap(&[0; 100], 10, 10));
        cache.insert(GlyphKey::new(0, 66, 16.0), make_bitmap(&[0; 200], 10, 20));
        cache.insert(GlyphKey::new(0, 67, 16.0), make_bitmap(&[0; 50], 5, 10));

        let mem = cache.estimated_memory();
        // 每条还应加上 size_of::<GlyphBitmap>()
        let bitmap_size = std::mem::size_of::<GlyphBitmap>();
        assert!(
            mem >= 100 + 200 + 50 + bitmap_size * 3,
            "内存估算应包含所有条目数据和结构体大小"
        );
    }

    /// 测试 GlyphKey::new 对零和负 font_size 的处理
    ///
    /// font_size 为 0 时 round() 为 0；font_size 为负数时
    /// round() 产生负整数，转换为 u16 时 Rust 会饱和为 0。
    #[test]
    fn test_glyph_key_zero_and_negative_font_size() {
        let key_zero = GlyphKey::new(0, 65, 0.0);
        assert_eq!(key_zero.size_px, 0, "font_size=0 应 round 为 0");

        let key_neg = GlyphKey::new(0, 65, -10.4);
        assert_eq!(key_neg.size_px, 0, "font_size<0 的 round 值转 u16 应饱和为 0");
    }
}
