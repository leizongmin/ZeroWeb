use super::FontLoader;
use hashbrown::HashMap;
use std::sync::Arc;

type ShapeCacheKey = (
    Vec<u64>,
    u32,
    (u8, u32),
    crate::font::TextDirection,
    Vec<crate::font::OpenTypeFeature>,
    String,
);
pub(super) type ShapeCache = Arc<std::sync::Mutex<HashMap<ShapeCacheKey, Vec<crate::font::ShapedGlyph>>>>;

impl FontLoader {
    /// 使用指定 face 整形文本，并缓存跨帧重复结果。
    pub fn shape_text_cached(&self, font_id: u32, text: &str, font_size: f32) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_direction(font_id, text, font_size, crate::font::TextDirection::Auto)
    }

    /// 使用指定 face 与方向整形文本，并缓存跨帧重复结果。
    pub fn shape_text_cached_with_direction(
        &self,
        font_id: u32,
        text: &str,
        font_size: f32,
        direction: crate::font::TextDirection,
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_features(font_id, text, font_size, direction, &[])
    }

    /// 使用指定 face、方向与 OpenType feature 整形文本，并缓存跨帧重复结果。
    pub fn shape_text_cached_with_features(
        &self,
        font_id: u32,
        text: &str,
        font_size: f32,
        direction: crate::font::TextDirection,
        features: &[crate::font::OpenTypeFeature],
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_font_ids(&[font_id], text, font_size, direction, features)
    }

    /// 使用有序 CSS face 列表整形文本，并缓存跨帧重复结果。
    pub fn shape_text_cached_with_font_ids(
        &self,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        direction: crate::font::TextDirection,
        features: &[crate::font::OpenTypeFeature],
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_font_ids_and_adjustment(
            font_ids,
            text,
            font_size,
            direction,
            features,
            crate::font::FontSizeAdjustment::None,
        )
    }

    /// 使用有序 CSS face 列表和 per-face `font-size-adjust` 整形文本，并缓存结果。
    pub fn shape_text_cached_with_font_ids_and_adjustment(
        &self,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        direction: crate::font::TextDirection,
        features: &[crate::font::OpenTypeFeature],
        adjustment: crate::font::FontSizeAdjustment,
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        let &primary_id = font_ids.first()?;
        // OPTIMIZATION: 缓存 key 用字体字节 hash 而非 instance_id——instance_id 全局
        // 递增，perf/reftest 每帧新建 FontLoader 重新加载字体（id 重新分配）→ key 每帧
        // 不同 → 缓存全 miss（`word-break: break-word` 中文长文逐字 fragment 每帧全量
        // 重排，perf-gate morning paint 回归）。按内容寻址跨帧/跨 loader 稳定；
        // 语义等价（不同字体数据 → 不同 hash，同 instance 隔离效果）。
        let font_hashes = font_ids
            .iter()
            .map(|font_id| self.get_font_data(*font_id).map(crate::font::font_bytes_hash))
            .collect::<Option<Vec<_>>>()?;
        let mut resolved_features = self.font_features.get(&primary_id).cloned().unwrap_or_default();
        for feature in features {
            if let Some(existing) = resolved_features
                .iter_mut()
                .find(|existing| existing.tag == feature.tag)
            {
                *existing = *feature;
            } else {
                resolved_features.push(*feature);
            }
        }
        let key = (
            font_hashes,
            font_size.to_bits(),
            adjustment.cache_key(),
            direction,
            resolved_features.clone(),
            text.to_string(),
        );
        if let Some(glyphs) = self.shape_cache.lock().expect("shape cache poisoned").get(&key) {
            return Some(glyphs.clone());
        }

        let glyphs = crate::font::TextShaper::new(self, Some(crate::primitive::FontId(primary_id)))
            .shape_single_line_with_font_ids_and_adjustment(
                font_ids,
                text,
                font_size,
                direction,
                &resolved_features,
                adjustment,
            );
        let mut cache = self.shape_cache.lock().expect("shape cache poisoned");
        // OPTIMIZATION: 4096 上限在 `word-break: break-word` 中文长文下逐字 fragment
        //（每汉字一 key）数帧即清空 → 每帧全量 miss 重排（perf-gate morning paint
        // 回归，R3243-F 扩大 fallback 调用面）。65536 覆盖长文全帧 key 集，跨帧命中。
        if cache.len() >= 65536 {
            cache.clear();
        }
        cache.insert(key, glyphs.clone());
        Some(glyphs)
    }

    /// 注册 face 级 OpenType feature 默认值。
    pub fn register_font_features(&mut self, font_id: u32, features: Vec<crate::font::OpenTypeFeature>) {
        if self.fonts.contains_key(&font_id) {
            self.font_features.insert(font_id, features);
        }
    }

    /// 注册 face 级 `size-adjust` 缩放因子。
    pub fn register_font_size_adjust(&mut self, font_id: u32, scale: f32) {
        if self.fonts.contains_key(&font_id) && scale.is_finite() && scale >= 0.0 {
            self.font_size_adjustments.insert(font_id, scale);
            self.shape_cache.lock().expect("shape cache poisoned").clear();
        }
    }
}
