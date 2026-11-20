use super::FontLoader;
use hashbrown::HashMap;
use std::sync::{Arc, Mutex};

/// R4374：per-font 行度量缓存键 `(font_id, size_bits)` → `(ascent, descent, line_gap)`。
type FontMetricsKey = (u32, u32);
/// R4378：per-char 实际使用字体缓存键 `(有序 face 链, char)` → 链上首个覆盖字体。
/// 链 = 元素 CSS face 序 + global fallback（R4378 起含多 @font-face 序，键须含全链
/// ——同首 face 异链的解析结果不同）。链/unicode-range 变更由调用方清缓存
/// （与 shape/hmtx 缓存同失效门）。
type CharFontKey = (std::sync::Arc<[u32]>, char);

#[derive(Default)]
pub(super) struct FallbackMetricsState {
    per_font: HashMap<FontMetricsKey, Option<(f32, f32, f32)>>,
    char_font: HashMap<CharFontKey, Option<u32>>,
}

impl FallbackMetricsState {
    fn clear(&mut self) {
        self.per_font.clear();
        self.char_font.clear();
    }
}

pub(super) type FallbackMetricsCache = Arc<Mutex<FallbackMetricsState>>;

impl FontLoader {
    /// R4378：per-font 行度量（含 `@font-face size-adjust` 缩放）。
    ///
    /// chromium 语义（css-fonts-5 §size-adjust）：size-adjust 缩放**所有**度量——
    /// 经 size-adjust face 解析的字形，其 ascent/descent 按缩放后有效字号取值
    /// （`size_adjusted_line_metrics` 同式）。漏乘则 size-adjust face 的行盒贡献
    /// 偏大（size-adjust.tentative oracle 翻红实证）。缓存键含缩放后 bits。
    pub(super) fn fallback_metrics_per_font(&self, font_id: u32, size: f32) -> Option<(f32, f32, f32)> {
        let scale = self.font_size_scale(font_id);
        let effective = size * scale;
        let key: FontMetricsKey = (font_id, effective.to_bits());
        if let Some(hit) = self
            .fallback_metrics_cache
            .lock()
            .expect("fallback metrics cache poisoned")
            .per_font
            .get(&key)
        {
            return *hit;
        }
        let computed = self.line_metrics_full(font_id, effective);
        self.fallback_metrics_cache
            .lock()
            .expect("fallback metrics cache poisoned")
            .per_font
            .insert(key, computed);
        computed
    }

    pub(super) fn fallback_metrics_char_font(&self, chain: &std::sync::Arc<[u32]>, ch: char) -> Option<u32> {
        let key: CharFontKey = (chain.clone(), ch);
        if let Some(hit) = self
            .fallback_metrics_cache
            .lock()
            .expect("fallback metrics cache poisoned")
            .char_font
            .get(&key)
        {
            return *hit;
        }
        let computed = self.resolve_font_for_code_point_in_chain(chain, ch);
        self.fallback_metrics_cache
            .lock()
            .expect("fallback metrics cache poisoned")
            .char_font
            .insert(key, computed);
        computed
    }

    pub(super) fn clear_fallback_metrics_cache(&self) {
        self.fallback_metrics_cache
            .lock()
            .expect("fallback metrics cache poisoned")
            .clear();
    }

    /// fallback 度量缓存常驻字节估算（T79 账外域仪拍：env 门测量读数，非行为路径）。
    ///
    /// 口径：per_font/char_font 两表 hashbrown 槽位容量（槽位数 × 单键值对
    /// size_of）+ char_font 键内 face 链 `Arc<[u32]>` 目标缓冲（按指针地址去重
    /// ——链 Arc 与调用方共享，只计本缓存持有的键引用面）。无字节上限（仅失效
    /// clear），估算服务域级归因（MB 量级排序），非 allocator 级精确核算。
    pub fn fallback_metrics_cache_bytes(&self) -> u64 {
        let cache = self.fallback_metrics_cache.lock().unwrap_or_else(|e| e.into_inner());
        let mut total: u64 = (std::mem::size_of::<FontMetricsKey>()
            .saturating_add(std::mem::size_of::<Option<(f32, f32, f32)>>())
            .saturating_mul(cache.per_font.capacity()))
        .saturating_add(
            std::mem::size_of::<CharFontKey>()
                .saturating_add(std::mem::size_of::<Option<u32>>())
                .saturating_mul(cache.char_font.capacity()),
        ) as u64;
        let mut chains: Vec<*const [u32]> = Vec::new();
        for key in cache.char_font.keys() {
            let ptr = Arc::as_ptr(&key.0);
            if chains.iter().all(|seen| !std::ptr::eq(*seen, ptr)) {
                chains.push(ptr);
                total = total.saturating_add((key.0.len() as u64) * 4);
            }
        }
        total
    }
}
