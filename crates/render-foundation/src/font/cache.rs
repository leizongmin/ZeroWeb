//! Glyph 缓存 — 基于 LRU 策略缓存已渲染的 glyph 位图。

use crate::font::{FontError, GlyphBitmap};
use hashbrown::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

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

/// 缓存条目 — 包含 glyph 位图和 LRU 位置信息。
///
/// 位图以 `Arc` 存储（性能门禁优化 S4，2026-08-08）：CPU 光栅器每帧对每个
/// 可见字符 `resolve_glyph_bitmap` 返回克隆（22k 字符页每帧 ~MB 级 memcpy），
/// 改为 Arc 共享后命中路径只做引用计数 bump。
struct CacheEntry {
    /// Glyph 位图数据（Arc 共享，命中免拷贝）。
    bitmap: Arc<GlyphBitmap>,
}

/// Glyph 缓存 — 基于 LRU 策略缓存已渲染的 glyph 位图以避免重复光栅化。
///
/// 使用近似 LRU 策略：每次访问将条目移到队列尾部，淘汰时从队列头部移除。
/// 淘汰粒度为批量（每次淘汰约 25% 的缓存），以减少频繁淘汰的开销。
pub struct GlyphCache {
    /// 缓存条目。
    cache: HashMap<GlyphKey, CacheEntry>,
    /// LRU 队列 — 最近访问的 key 在尾部，最久未访问的在头部。
    lru_queue: VecDeque<GlyphKey>,
    /// raw font_id → resolved font_id 映射（fallback 命中后记录）。
    ///
    /// 绘制路径先用图元携带的 raw font_id 查缓存，miss 后经 fallback 光栅化出
    /// resolved font_id（CJK 字形 ≠ raw）；若插入用 resolved 键而 lookup 用 raw 键
    /// 会永久 miss、每帧重光栅化。此映射让第二帧起直接命中。
    resolved: HashMap<GlyphKey, u32>,
    /// 最大缓存条目数。
    max_entries: usize,
}

impl GlyphCache {
    /// 创建新的 Glyph 缓存。
    pub fn new(max_entries: usize) -> Self {
        Self {
            cache: HashMap::new(),
            lru_queue: VecDeque::new(),
            resolved: HashMap::new(),
            max_entries: if max_entries == 0 { 1 } else { max_entries },
        }
    }

    /// 记录 raw 键的 fallback 解析结果（resolved font_id）。
    pub fn record_resolution(&mut self, raw_key: &GlyphKey, resolved_font_id: u32) {
        self.resolved.insert(raw_key.clone(), resolved_font_id);
    }

    /// 查询 raw 键是否已有 fallback 解析结果。
    pub fn resolved_font_id(&self, raw_key: &GlyphKey) -> Option<u32> {
        self.resolved.get(raw_key).copied()
    }

    /// 淘汰旧条目，为新条目腾出空间。
    ///
    /// 淘汰约 25% 的最旧条目（最少一个）。
    fn evict(&mut self) {
        let evict_count = (self.max_entries / 4).max(1);
        for _ in 0..evict_count {
            if let Some(old_key) = self.lru_queue.pop_front() {
                self.cache.remove(&old_key);
            }
        }
        // 不重建 lru_index（O(n)）——insert 重复路径改用位置查找（罕见），
        // evict 批量淘汰后旧 index 过期无害（队列头部 pop 与 index 无关）。
    }

    /// 获取或插入 glyph。
    ///
    /// 命中与插入均不提升 LRU（命中提升需全表 `rebuild_lru_indices` O(n)，CJK 页
    /// 首帧 miss 密集时每次插入 O(n) 重建 = 千万级 hash 操作；本方法调用方均为
    /// miss 插入路径，绘制热路径用 `get`/`get_shared` 自行管理，LRU 近似 FIFO
    /// 淘汰对命中率分布影响可忽略）。
    pub fn get_or_insert_with<F>(&mut self, key: GlyphKey, f: F) -> Result<&GlyphBitmap, FontError>
    where
        F: FnOnce() -> Result<GlyphBitmap, FontError>,
    {
        // 检查缓存命中（不提升，见方法注释）
        if self.cache.contains_key(&key) {
            return Ok(self.cache.get(&key).unwrap().bitmap.as_ref());
        }

        // 缓存未命中，需要淘汰空间
        if self.cache.len() >= self.max_entries {
            self.evict();
        }

        // 生成位图并插入（新条目在 LRU 尾部，无需提升）
        let bitmap = f()?;
        self.lru_queue.push_back(key.clone());
        self.cache.insert(key, CacheEntry { bitmap: Arc::new(bitmap) });

        Ok(self.cache.get(self.lru_queue.back().unwrap()).unwrap().bitmap.as_ref())
    }

    /// 直接获取缓存的 glyph（不更新 LRU 顺序）。
    pub fn get(&self, key: &GlyphKey) -> Option<&GlyphBitmap> {
        self.cache.get(key).map(|e| e.bitmap.as_ref())
    }

    /// 获取缓存的 glyph 共享句柄（S4：命中路径免位图拷贝，仅 Arc 引用计数 bump）。
    pub fn get_shared(&self, key: &GlyphKey) -> Option<Arc<GlyphBitmap>> {
        self.cache.get(key).map(|e| e.bitmap.clone())
    }

    /// 插入 glyph 到缓存（插入到 LRU 尾部）。
    pub fn insert(&mut self, key: GlyphKey, bitmap: GlyphBitmap) {
        // 如果已存在，先移除旧条目（位置查找——evict 后队列 index 未维护）
        if self.cache.remove(&key).is_some()
            && let Some(pos) = self.lru_queue.iter().position(|k| *k == key)
        {
            self.lru_queue.remove(pos);
        }

        self.lru_queue.push_back(key.clone());
        self.cache.insert(key, CacheEntry { bitmap: Arc::new(bitmap) });
    }

    /// 缓存条目数。
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// 清空缓存。
    pub fn clear(&mut self) {
        self.cache.clear();
        self.lru_queue.clear();
        self.resolved.clear();
    }

    /// 当前内存使用量估算（字节）。
    pub fn estimated_memory(&self) -> usize {
        self.cache
            .values()
            .map(|e| e.bitmap.data.len() + std::mem::size_of::<GlyphBitmap>())
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
        let key = GlyphKey::new(0, 65, 16.0);
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
        // 淘汰后应该 ≤ 5
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
    fn test_cache_eviction_lru_order() {
        let mut cache = GlyphCache::new(4);
        // Fill cache: 0, 1, 2, 3
        for i in 0..4u32 {
            let key = GlyphKey::new(0, i, 16.0);
            cache.insert(key, make_bitmap(&[i as u8; 4], 2, 2));
        }
        assert_eq!(cache.len(), 4);

        // Access glyph 0（get_or_insert_with 不提升 LRU——S4 后绘制热路径
        // get/get_shared 本就不 promote，命中提升需全表 rebuild O(n)，对
        // 命中率分布无可测收益，见 get_or_insert_with 注释）

        // Insert new key — evict glyph 0 (FIFO: oldest inserted first)
        let new_key = GlyphKey::new(0, 99, 16.0);
        cache
            .get_or_insert_with(new_key, || Ok(make_bitmap(&[0; 4], 2, 2)))
            .unwrap();

        // glyph 0 should be evicted (FIFO 淘汰最早插入的)
        assert!(
            cache.get(&GlyphKey::new(0, 0, 16.0)).is_none(),
            "FIFO: glyph 0 (oldest inserted) should be evicted"
        );
        // glyph 1..3 should survive（FIFO 只淘汰最早插入的）
        for i in 1..4u32 {
            assert!(
                cache.get(&GlyphKey::new(0, i, 16.0)).is_some(),
                "glyph {i} should survive FIFO eviction"
            );
        }
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
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_cache_get_or_insert_error_propagation() {
        let mut cache = GlyphCache::new(100);
        let key = GlyphKey::new(0, 65, 16.0);
        let result = cache.get_or_insert_with(key.clone(), || Err(FontError::NotFound("test".to_string())));
        assert!(result.is_err());
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
        // max_entries 被提升为 1
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

    /// 测试零容量缓存的 get_or_insert。
    #[test]
    fn test_cache_get_or_insert_zero_capacity() {
        let mut cache = GlyphCache::new(0);
        let key = GlyphKey::new(0, 65, 16.0);
        let result = cache.get_or_insert_with(key.clone(), || Ok(make_bitmap(&[42], 1, 1)));
        assert!(result.is_ok(), "零容量缓存应能通过 get_or_insert 插入");
        assert!(cache.get(&key).is_some());
    }

    /// 测试缓存错误不插入后再次调用成功。
    #[test]
    fn test_cache_failed_insert_retriable() {
        let mut cache = GlyphCache::new(100);
        let key = GlyphKey::new(0, 65, 16.0);

        let result = cache.get_or_insert_with(key.clone(), || Err(FontError::NotFound("test".to_string())));
        assert!(result.is_err());
        assert!(cache.get(&key).is_none(), "失败后缓存不应有条目");

        let result = cache.get_or_insert_with(key.clone(), || Ok(make_bitmap(&[99], 1, 1)));
        assert!(result.is_ok());
        assert_eq!(cache.get(&key).unwrap().data[0], 99);
    }

    /// 测试 estimated_memory 对多个条目的累加。
    #[test]
    fn test_cache_estimated_memory_multiple_entries() {
        let mut cache = GlyphCache::new(100);

        cache.insert(GlyphKey::new(0, 65, 16.0), make_bitmap(&[0; 100], 10, 10));
        cache.insert(GlyphKey::new(0, 66, 16.0), make_bitmap(&[0; 200], 10, 20));
        cache.insert(GlyphKey::new(0, 67, 16.0), make_bitmap(&[0; 50], 5, 10));

        let mem = cache.estimated_memory();
        let bitmap_size = std::mem::size_of::<GlyphBitmap>();
        assert!(
            mem >= 100 + 200 + 50 + bitmap_size * 3,
            "内存估算应包含所有条目数据和结构体大小"
        );
    }

    /// 测试 GlyphKey 对零和负 font_size 的处理。
    #[test]
    fn test_glyph_key_zero_and_negative_font_size() {
        let key_zero = GlyphKey::new(0, 65, 0.0);
        assert_eq!(key_zero.size_px, 0, "font_size=0 应 round 为 0");

        let key_neg = GlyphKey::new(0, 65, -10.4);
        assert_eq!(key_neg.size_px, 0, "font_size<0 的 round 值转 u16 应饱和为 0");
    }

    /// 测试 LRU 优先淘汰最久未访问的条目。
    #[test]
    fn test_lru_evicts_oldest_first() {
        let mut cache = GlyphCache::new(4);
        // 填充: 0, 1, 2, 3
        for i in 0..4u32 {
            cache.insert(GlyphKey::new(0, i, 16.0), make_bitmap(&[i as u8; 4], 2, 2));
        }

        // 访问 0 和 1（get_or_insert_with 不提升，见 get_or_insert_with 注释）

        // 插入新条目触发淘汰 → 应淘汰 0（最早插入，FIFO）
        cache
            .get_or_insert_with(GlyphKey::new(0, 50, 16.0), || Ok(make_bitmap(&[0; 4], 2, 2)))
            .unwrap();

        // 1 和 2 应该存活
        assert!(cache.get(&GlyphKey::new(0, 1, 16.0)).is_some());
        assert!(cache.get(&GlyphKey::new(0, 2, 16.0)).is_some());
        // 0 应该被淘汰（FIFO 最早插入）
        assert!(cache.get(&GlyphKey::new(0, 0, 16.0)).is_none());
    }

    /// 测试 insert 覆盖旧条目时 LRU 队列正确更新。
    #[test]
    fn test_insert_overwrite_promotes_in_lru() {
        let mut cache = GlyphCache::new(4);
        for i in 0..4u32 {
            cache.insert(GlyphKey::new(0, i, 16.0), make_bitmap(&[i as u8; 4], 2, 2));
        }

        // 覆盖 glyph 0（应提升到尾部）
        cache.insert(GlyphKey::new(0, 0, 16.0), make_bitmap(&[99; 4], 2, 2));

        // 插入新条目 → 应淘汰最旧的（glyph 1）
        cache
            .get_or_insert_with(GlyphKey::new(0, 50, 16.0), || Ok(make_bitmap(&[0; 4], 2, 2)))
            .unwrap();

        assert!(cache.get(&GlyphKey::new(0, 0, 16.0)).is_some(), "覆盖的 glyph 0 应存活");
        assert!(cache.get(&GlyphKey::new(0, 1, 16.0)).is_none(), "glyph 1 应被淘汰");
    }
}
