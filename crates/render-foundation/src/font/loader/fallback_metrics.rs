use super::FontLoader;
use hashbrown::HashMap;
use std::sync::{Arc, Mutex};

/// R4374：per-font 行度量缓存键 `(font_id, size_bits)` → `(ascent, descent, line_gap)`。
type FontMetricsKey = (u32, u32);
/// R4374：per-char 实际使用字体缓存键 `(primary_id, char)` → 链上首个覆盖字体。
/// 链/unicode-range 变更由调用方清缓存（与 shape/hmtx 缓存同失效门）。
type CharFontKey = (u32, char);

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
    pub(super) fn fallback_metrics_per_font(&self, font_id: u32, size: f32) -> Option<(f32, f32, f32)> {
        let key: FontMetricsKey = (font_id, size.to_bits());
        if let Some(hit) = self
            .fallback_metrics_cache
            .lock()
            .expect("fallback metrics cache poisoned")
            .per_font
            .get(&key)
        {
            return *hit;
        }
        let computed = self.line_metrics_full(font_id, size);
        self.fallback_metrics_cache
            .lock()
            .expect("fallback metrics cache poisoned")
            .per_font
            .insert(key, computed);
        computed
    }

    pub(super) fn fallback_metrics_char_font(&self, primary_id: u32, ch: char) -> Option<u32> {
        let key: CharFontKey = (primary_id, ch);
        if let Some(hit) = self
            .fallback_metrics_cache
            .lock()
            .expect("fallback metrics cache poisoned")
            .char_font
            .get(&key)
        {
            return *hit;
        }
        let chain = self.lookup_chain(primary_id);
        let computed = self.resolve_font_for_code_point_in_chain(&chain, ch);
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
}
