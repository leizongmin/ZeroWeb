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
}
