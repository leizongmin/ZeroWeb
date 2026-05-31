//! 图片对象缓存与 GC — 管理已解码图片的缓存和生命周期
//!
//! 提供：
//! - 基于引用计数的图片缓存
//! - LRU 风格的垃圾回收
//! - 图片数据存储（RGBA 像素数据）

use crate::geometry::Size;
use hashbrown::HashMap;

/// 图片缓存键 — 唯一标识一张图片
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct ImageKey(pub u64);

impl ImageKey {
    /// 创建新的图片键
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 已解码的图片数据
#[derive(Debug, Clone)]
pub struct ImageData {
    /// RGBA 像素数据（行优先）
    pub pixels: Vec<u8>,
    /// 宽度（像素）
    pub width: u32,
    /// 高度（像素）
    pub height: u32,
}

impl ImageData {
    /// 从 RGBA 字节数据创建图片
    ///
    /// # Errors
    /// 如果数据长度不等于 `width * height * 4` 则返回错误
    pub fn from_rgba(pixels: Vec<u8>, width: u32, height: u32) -> Result<Self, String> {
        let expected = (width as usize) * (height as usize) * 4;
        if pixels.len() != expected {
            return Err(format!(
                "pixel data size mismatch: expected {expected}, got {}",
                pixels.len()
            ));
        }
        Ok(Self { pixels, width, height })
    }

    /// 创建指定尺寸的空（全透明）图片
    pub fn new_empty(width: u32, height: u32) -> Self {
        let pixels = vec![0u8; (width as usize) * (height as usize) * 4];
        Self { pixels, width, height }
    }

    /// 获取指定位置的像素 (R, G, B, A)
    ///
    /// # Panics
    /// 如果坐标越界则 panic
    pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let idx = ((y * self.width + x) * 4) as usize;
        [
            self.pixels[idx],
            self.pixels[idx + 1],
            self.pixels[idx + 2],
            self.pixels[idx + 3],
        ]
    }

    /// 获取图片尺寸
    pub fn size(&self) -> Size {
        Size::new(self.width as f32, self.height as f32)
    }

    /// 字节大小估算
    pub fn byte_size(&self) -> usize {
        self.pixels.len()
    }
}

/// 图片缓存条目
#[derive(Debug)]
struct CacheEntry {
    /// 图片数据
    data: ImageData,
    /// 引用计数
    ref_count: u32,
    /// 最近访问的代数（用于 GC）
    last_access_gen: u64,
}

/// 图片对象缓存 — 管理已解码图片的生命周期
///
/// 使用引用计数和代际标记实现 GC：
/// - 插入图片时 ref_count = 1
/// - 每次 `get` 时递增 ref_count 并更新 last_access_gen
/// - `gc()` 移除 ref_count == 0 或长时间未访问的条目
pub struct ImageCache {
    /// 缓存条目
    entries: HashMap<ImageKey, CacheEntry>,
    /// 下一个键 ID
    next_key: u64,
    /// 当前世代（每次 GC 递增）
    current_gen: u64,
    /// 最大缓存条目数
    max_entries: usize,
    /// 最大字节数
    max_bytes: usize,
}

impl ImageCache {
    /// 创建新的图片缓存
    ///
    /// - `max_entries`: 最大缓存条目数
    /// - `max_bytes`: 最大缓存字节数
    pub fn new(max_entries: usize, max_bytes: usize) -> Self {
        Self {
            entries: HashMap::new(),
            next_key: 0,
            current_gen: 0,
            max_entries,
            max_bytes,
        }
    }

    /// 插入图片数据，返回缓存键
    pub fn insert(&mut self, data: ImageData) -> ImageKey {
        let key = ImageKey::new(self.next_key);
        self.next_key += 1;

        let entry = CacheEntry {
            data,
            ref_count: 1,
            last_access_gen: self.current_gen,
        };
        self.entries.insert(key.clone(), entry);
        key
    }

    /// 获取图片数据的引用，并递增引用计数
    pub fn get(&mut self, key: &ImageKey) -> Option<&ImageData> {
        let entry = self.entries.get_mut(key)?;
        entry.ref_count = entry.ref_count.saturating_add(1);
        entry.last_access_gen = self.current_gen;
        Some(&entry.data)
    }

    /// 释放一次引用（递减引用计数）
    pub fn release(&mut self, key: &ImageKey) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.ref_count = entry.ref_count.saturating_sub(1);
        }
    }

    /// 执行垃圾回收
    ///
    /// 移除以下条目：
    /// - ref_count == 0 的条目
    /// - 如果总条目数或总字节数超过限制，按 LRU 淘汰最旧条目
    pub fn gc(&mut self) {
        self.current_gen += 1;

        // 先移除 ref_count == 0 的条目
        self.entries.retain(|_, entry| entry.ref_count > 0);

        // 如果仍然超限，按 last_access_gen 排序淘汰最旧条目
        while self.entries.len() > self.max_entries || self.total_bytes() > self.max_bytes {
            if self.entries.is_empty() {
                break;
            }
            // 找到最旧的条目并移除
            let oldest_key = self
                .entries
                .iter()
                .min_by_key(|(_, e)| e.last_access_gen)
                .map(|(k, _)| k.clone());

            if let Some(key) = oldest_key {
                self.entries.remove(&key);
            } else {
                break;
            }
        }
    }

    /// 当前缓存条目数
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 当前缓存字节总数
    pub fn total_bytes(&self) -> usize {
        self.entries.values().map(|e| e.data.byte_size()).sum()
    }

    /// 当前世代
    pub fn generation(&self) -> u64 {
        self.current_gen
    }

    /// 获取指定键的引用计数（用于测试）
    pub fn ref_count(&self, key: &ImageKey) -> Option<u32> {
        self.entries.get(key).map(|e| e.ref_count)
    }

    /// 清空所有缓存
    pub fn clear(&mut self) {
        self.entries.clear();
        self.current_gen += 1;
    }
}

impl Default for ImageCache {
    fn default() -> Self {
        Self::new(256, 64 * 1024 * 1024) // 256 entries, 64 MB
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_image(w: u32, h: u32, fill: u8) -> ImageData {
        let pixels = vec![fill; (w as usize) * (h as usize) * 4];
        ImageData::from_rgba(pixels, w, h).unwrap()
    }

    #[test]
    fn test_image_data_from_rgba() {
        let pixels = vec![255u8; 2 * 2 * 4];
        let img = ImageData::from_rgba(pixels, 2, 2).unwrap();
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.get_pixel(0, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn test_image_data_from_rgba_wrong_size() {
        let pixels = vec![255u8; 10];
        let result = ImageData::from_rgba(pixels, 2, 2);
        assert!(result.is_err());
    }

    #[test]
    fn test_image_data_new_empty() {
        let img = ImageData::new_empty(4, 4);
        assert_eq!(img.pixels.len(), 4 * 4 * 4);
        assert_eq!(img.get_pixel(0, 0), [0, 0, 0, 0]);
    }

    #[test]
    fn test_image_data_size() {
        let img = ImageData::new_empty(10, 20);
        let size = img.size();
        assert_eq!(size.width, 10.0);
        assert_eq!(size.height, 20.0);
    }

    #[test]
    fn test_image_data_byte_size() {
        let img = ImageData::new_empty(3, 4);
        assert_eq!(img.byte_size(), 3 * 4 * 4);
    }

    #[test]
    fn test_cache_insert_and_get() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 255));
        assert_eq!(cache.len(), 1);

        let img = cache.get(&key).unwrap();
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(cache.ref_count(&key), Some(2)); // insert gives 1, get adds 1
    }

    #[test]
    fn test_cache_release() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 128));
        assert_eq!(cache.ref_count(&key), Some(1));

        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
    }

    #[test]
    fn test_cache_gc_removes_zero_ref() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 100));
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));

        cache.gc();
        assert!(cache.ref_count(&key).is_none());
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_gc_keeps_referenced() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 200));
        // ref_count is 1, should be kept
        cache.gc();
        assert!(cache.ref_count(&key).is_some());
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_cache_gc_evicts_by_lru_when_over_max_entries() {
        let mut cache = ImageCache::new(2, 1024 * 1024);
        let _key1 = cache.insert(make_image(1, 1, 10)); // gen 0
        let _key2 = cache.insert(make_image(1, 1, 20));
        let _key3 = cache.insert(make_image(1, 1, 30)); // triggers over max_entries

        cache.gc();
        assert!(cache.len() <= 2);
    }

    #[test]
    fn test_cache_gc_evicts_by_lru_when_over_max_bytes() {
        let mut cache = ImageCache::new(100, 32); // 32 bytes max
        let key1 = cache.insert(make_image(2, 2, 10)); // 16 bytes
        let _key2 = cache.insert(make_image(2, 2, 20)); // 16 bytes = total 32
        assert_eq!(cache.total_bytes(), 32);

        // Access key1 so it's newer
        let _ = cache.get(&key1);

        // Insert another, total > 32
        let _key3 = cache.insert(make_image(2, 2, 30)); // total = 48
        cache.gc();
        assert!(cache.total_bytes() <= 32);
    }

    #[test]
    fn test_cache_clear() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        cache.insert(make_image(1, 1, 0));
        cache.insert(make_image(1, 1, 1));
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_default() {
        let cache = ImageCache::default();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_get_nonexistent() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = ImageKey::new(999);
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_cache_release_nonexistent_is_noop() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = ImageKey::new(999);
        // Should not panic
        cache.release(&key);
    }

    #[test]
    fn test_cache_generation_increases_on_gc() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert_eq!(cache.generation(), 0);
        cache.gc();
        assert_eq!(cache.generation(), 1);
        cache.gc();
        assert_eq!(cache.generation(), 2);
    }

    #[test]
    fn test_cache_total_bytes() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        cache.insert(make_image(2, 2, 0)); // 2*2*4 = 16 bytes
        cache.insert(make_image(3, 3, 0)); // 3*3*4 = 36 bytes
        assert_eq!(cache.total_bytes(), 52);
    }

    #[test]
    fn test_image_key_new() {
        let key = ImageKey::new(42);
        assert_eq!(key.0, 42);
    }

    #[test]
    fn test_image_data_get_pixel_various_positions() {
        // 2x2 image with distinct pixel values
        let pixels: Vec<u8> = vec![
            255, 0, 0, 255, // (0,0) red
            0, 255, 0, 255, // (1,0) green
            0, 0, 255, 255, // (0,1) blue
            255, 255, 0, 255, // (1,1) yellow
        ];
        let img = ImageData::from_rgba(pixels, 2, 2).unwrap();
        assert_eq!(img.get_pixel(0, 0), [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(1, 0), [0, 255, 0, 255]);
        assert_eq!(img.get_pixel(0, 1), [0, 0, 255, 255]);
        assert_eq!(img.get_pixel(1, 1), [255, 255, 0, 255]);
    }

    #[test]
    fn test_image_key_equality_and_hash() {
        let k1 = ImageKey::new(10);
        let k2 = ImageKey::new(10);
        let k3 = ImageKey::new(20);
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(k1.clone());
        set.insert(k2.clone());
        assert_eq!(set.len(), 1);
        set.insert(k3.clone());
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_cache_multiple_get_increments_ref_count() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 100));
        assert_eq!(cache.ref_count(&key), Some(1));

        let _ = cache.get(&key);
        assert_eq!(cache.ref_count(&key), Some(2));

        let _ = cache.get(&key);
        assert_eq!(cache.ref_count(&key), Some(3));
    }

    #[test]
    fn test_cache_release_saturating_sub() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(1, 1, 0));
        // ref_count starts at 1
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
        // Releasing below 0 should saturate at 0
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));
    }

    #[test]
    fn test_cache_clear_increases_generation() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert_eq!(cache.generation(), 0);
        cache.insert(make_image(1, 1, 0));
        cache.clear();
        assert_eq!(cache.generation(), 1);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_sequential_insert_unique_keys() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 10));
        let k2 = cache.insert(make_image(1, 1, 20));
        let k3 = cache.insert(make_image(1, 1, 30));
        assert_ne!(k1, k2);
        assert_ne!(k2, k3);
        assert_eq!(cache.len(), 3);
        // Each has ref_count 1
        assert_eq!(cache.ref_count(&k1), Some(1));
        assert_eq!(cache.ref_count(&k2), Some(1));
        assert_eq!(cache.ref_count(&k3), Some(1));
    }

    #[test]
    fn test_image_data_from_rgba_large() {
        let pixels = vec![128u8; 100 * 100 * 4];
        let img = ImageData::from_rgba(pixels, 100, 100).unwrap();
        assert_eq!(img.byte_size(), 100 * 100 * 4);
        assert_eq!(img.get_pixel(50, 50), [128, 128, 128, 128]);
    }

    /// 测试 LRU 淘汰顺序：先插入（最旧世代）的条目应被优先淘汰
    #[test]
    fn test_cache_gc_lru_eviction_order() {
        let mut cache = ImageCache::new(2, 1024 * 1024);
        let key1 = cache.insert(make_image(1, 1, 10)); // gen 0
        let key2 = cache.insert(make_image(1, 1, 20)); // gen 0
        // 推进世代后插入 key3，使 key1/key2 有更旧的 last_access_gen
        cache.gc(); // gen -> 1，key1 和 key2 的 last_access_gen 仍为 0
        let key3 = cache.insert(make_image(1, 1, 30)); // last_access_gen = 1

        cache.gc(); // gen -> 2，应淘汰 gen=0 的条目（key1 和 key2 之一）
        assert_eq!(cache.len(), 2);
        // key3 最新（gen=1），一定保留
        assert!(cache.ref_count(&key3).is_some(), "最新的 key3 应保留");
        // key1 和 key2 同为 gen=0，淘汰其中一个即可
        let remaining = cache.ref_count(&key1).is_some() as usize + cache.ref_count(&key2).is_some() as usize;
        assert_eq!(remaining, 1, "key1 和 key2 中应恰好保留一个");
    }

    /// 测试多次 GC 后访问时间更新使条目免于被淘汰
    #[test]
    fn test_cache_gc_get_updates_lru_and_protects_entry() {
        let mut cache = ImageCache::new(2, 1024 * 1024);
        let key1 = cache.insert(make_image(1, 1, 10));
        let key2 = cache.insert(make_image(1, 1, 20));

        // 先执行一次 GC 使世代推进到 1
        cache.gc();
        // 此时 key1 和 key2 的 last_access_gen 仍为 0

        // 访问 key1 使其 last_access_gen 更新为 1
        let _ = cache.get(&key1);
        // key2 的 last_access_gen 仍为 0（更旧）

        // 插入第三个，触发超限
        let key3 = cache.insert(make_image(1, 1, 30)); // gen=1

        cache.gc(); // 世代推进到 2
        assert_eq!(cache.len(), 2);
        // key1 被访问过（gen=1），应保留；key2 最旧（gen=0），应被淘汰
        assert!(cache.ref_count(&key1).is_some(), "被访问过的 key1 应保留");
        assert!(cache.ref_count(&key2).is_none(), "未被访问的 key2 应被淘汰");
        assert!(cache.ref_count(&key3).is_some(), "key3 应保留");
    }

    /// 测试混合 release + get 模式下 GC 的正确性
    #[test]
    fn test_cache_gc_mixed_release_and_get_pattern() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key1 = cache.insert(make_image(1, 1, 10));
        let key2 = cache.insert(make_image(1, 1, 20));
        let key3 = cache.insert(make_image(1, 1, 30));

        // key1: release → ref_count=0（应被 GC 移除）
        cache.release(&key1);
        // key2: get → ref_count=2（应保留）
        let _ = cache.get(&key2);
        // key3: 保持 ref_count=1（应保留）

        cache.gc();
        assert!(cache.ref_count(&key1).is_none(), "ref_count=0 的 key1 应被移除");
        assert_eq!(cache.ref_count(&key2), Some(2), "key2 ref_count 应为 2");
        assert_eq!(cache.ref_count(&key3), Some(1), "key3 ref_count 应为 1");
        assert_eq!(cache.len(), 2);
    }

    /// 测试超大图片超过 max_bytes 时被淘汰
    #[test]
    fn test_cache_gc_single_image_exceeds_max_bytes() {
        let mut cache = ImageCache::new(10, 16); // 仅允许 16 字节
        let key1 = cache.insert(make_image(1, 1, 10)); // 4 字节
        let key2 = cache.insert(make_image(1, 1, 20)); // 4 字节
        assert_eq!(cache.total_bytes(), 8);

        // 插入一个 4x4（64 字节）的图片，远超 max_bytes
        let key_big = cache.insert(make_image(4, 4, 255)); // 64 字节
        assert!(cache.total_bytes() > 16);

        cache.gc();
        // GC 应不断淘汰最旧条目直到总字节数 ≤ max_bytes
        // 即使淘汰所有旧条目后只剩 key_big (64 > 16)，也会继续淘汰
        // 最终 key_big 也会被淘汰（因为 64 > 16）
        assert!(cache.total_bytes() <= 16, "GC 后总字节数应不超过 max_bytes");
    }

    // -- 边界条件测试 --
    /// 测试 ImageKey 边界值
    #[test]
    fn test_image_key_boundary_values() {
        let k_min = ImageKey::new(0);
        assert_eq!(k_min.0, 0);
        let k_max = ImageKey::new(u64::MAX);
        assert_eq!(k_max.0, u64::MAX);
        assert_ne!(k_min, k_max);
    }

    /// 测试 ImageData::new_empty 零尺寸
    #[test]
    fn test_image_data_new_empty_zero_dims() {
        let img = ImageData::new_empty(0, 0);
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 0);
        assert!(img.pixels.is_empty());
    }

    /// 测试 ImageData::from_rgba 零宽度
    #[test]
    fn test_image_data_from_rgba_zero_width() {
        let result = ImageData::from_rgba(vec![], 0, 5);
        assert!(result.is_ok());
        let img = result.unwrap();
        assert_eq!(img.width, 0);
        assert_eq!(img.height, 5);
        assert!(img.pixels.is_empty());
    }

    /// 测试 ImageData get_pixel 1x1 图片
    #[test]
    fn test_image_data_get_pixel_1x1() {
        let pixels = vec![42, 84, 126, 255];
        let img = ImageData::from_rgba(pixels, 1, 1).unwrap();
        assert_eq!(img.get_pixel(0, 0), [42, 84, 126, 255]);
    }

    /// 测试 ImageCache gc 后再 insert
    #[test]
    fn test_cache_insert_after_gc() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 100));
        cache.release(&k1);
        cache.gc();
        assert!(cache.is_empty());

        let k2 = cache.insert(make_image(1, 1, 200));
        assert_eq!(cache.ref_count(&k2), Some(1));
        assert_eq!(cache.len(), 1);
    }

    /// 测试 ImageCache release 然后 get（ref_count 从 0 回到 1）
    #[test]
    fn test_cache_release_then_get() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        let key = cache.insert(make_image(2, 2, 128));
        cache.release(&key);
        assert_eq!(cache.ref_count(&key), Some(0));

        // get 仍然能取回数据（条目还在，只是 ref_count=0）
        let img = cache.get(&key);
        assert!(img.is_some());
        assert_eq!(cache.ref_count(&key), Some(1));
    }

    /// 测试 ImageCache gc 在空缓存上
    #[test]
    fn test_cache_gc_empty() {
        let mut cache = ImageCache::new(10, 1024 * 1024);
        assert!(cache.is_empty());
        assert_eq!(cache.generation(), 0);
        cache.gc();
        assert!(cache.is_empty());
        assert_eq!(cache.generation(), 1);
    }

    /// 测试 ImageData::new_empty 所有像素透明
    #[test]
    fn test_image_data_new_empty_all_transparent() {
        let img = ImageData::new_empty(2, 2);
        assert_eq!(img.get_pixel(0, 0), [0, 0, 0, 0]);
        assert_eq!(img.get_pixel(1, 0), [0, 0, 0, 0]);
        assert_eq!(img.get_pixel(0, 1), [0, 0, 0, 0]);
        assert_eq!(img.get_pixel(1, 1), [0, 0, 0, 0]);
    }

    /// 测试 ImageCache max_entries=1
    #[test]
    fn test_cache_max_entries_one() {
        let mut cache = ImageCache::new(1, 1024 * 1024);
        let k1 = cache.insert(make_image(1, 1, 10));
        // gc to advance generation so k1 has older gen
        cache.gc();
        let k2 = cache.insert(make_image(1, 1, 20));
        // Now len = 2 > max_entries = 1
        cache.gc();
        // Should evict oldest entry, keeping only 1
        assert!(cache.len() <= 1);
        assert!(cache.ref_count(&k2).is_some(), "较新的 k2 应保留");
        assert!(cache.ref_count(&k1).is_none(), "较旧的 k1 应被淘汰");
    }

    /// 测试连续插入相同内容的数据产生不同的缓存键，缓存中两个条目并存。
    ///
    /// 对 ImageCache 调用两次 insert 传入相同像素数据，应返回不同的 key，
    /// 缓存中两个条目独立存在，可通过各自的 key 分别访问。
    #[test]
    fn test_image_cache_double_insert_same_key() {
        let mut cache = ImageCache::new(10, 1024 * 1024);

        let data1 = make_image(2, 2, 128);
        let data2 = make_image(2, 2, 128); // 相同尺寸和填充值

        let key1 = cache.insert(data1);
        let key2 = cache.insert(data2);

        // 两次插入应返回不同的 key
        assert_ne!(key1, key2, "两次插入应返回不同的 key");
        assert_eq!(cache.len(), 2, "缓存中应有 2 个条目");

        // 两个 key 都能独立获取
        let img1 = cache.get(&key1);
        assert!(img1.is_some(), "key1 应能获取到图片");
        assert_eq!(img1.unwrap().width, 2);

        let img2 = cache.get(&key2);
        assert!(img2.is_some(), "key2 应能获取到图片");
        assert_eq!(img2.unwrap().width, 2);

        // 引用计数各被增加（insert=1 + get=1 = 2）
        assert_eq!(cache.ref_count(&key1), Some(2));
        assert_eq!(cache.ref_count(&key2), Some(2));

        // 释放 key1 后 GC，key1 被移除，key2 保留
        cache.release(&key1);
        cache.release(&key1); // release the extra ref from get
        cache.gc();
        assert!(cache.ref_count(&key1).is_none(), "key1 应被 GC 移除");
        assert!(cache.ref_count(&key2).is_some(), "key2 应保留");
    }

    /// 测试 max_entries=0 时 GC 会清除所有条目
    ///
    /// 当 max_entries 设置为 0 时，每次 GC 都会淘汰所有条目，
    /// 因为任何条目数（>0）都超过限制。insert 本身不会拒绝插入，
    /// 但 GC 后缓存一定为空。
    #[test]
    fn test_image_cache_zero_max_entries() {
        let mut cache = ImageCache::new(0, 1024 * 1024);

        // 插入多个条目
        let k1 = cache.insert(make_image(1, 1, 10));
        let k2 = cache.insert(make_image(2, 2, 20));
        let k3 = cache.insert(make_image(3, 3, 30));
        assert_eq!(cache.len(), 3, "插入后应有 3 个条目");

        // GC 后所有条目因 max_entries=0 被淘汰
        cache.gc();
        assert!(cache.is_empty(), "GC 后缓存应为空");
        assert_eq!(cache.len(), 0);
        assert!(cache.ref_count(&k1).is_none(), "k1 应被淘汰");
        assert!(cache.ref_count(&k2).is_none(), "k2 应被淘汰");
        assert!(cache.ref_count(&k3).is_none(), "k3 应被淘汰");

        // 再次插入后立即 GC，同样被淘汰
        let k4 = cache.insert(make_image(1, 1, 40));
        cache.gc();
        assert!(cache.is_empty(), "再次 GC 后缓存仍应为空");
        assert!(cache.ref_count(&k4).is_none());
    }
}
