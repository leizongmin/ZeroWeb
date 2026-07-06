//! FontdueBackend → UI core TextMeasure 适配器（P3-7 核心修复）。
//!
//! 桥接 foundation 层 `TextMeasurer`（签名复杂）和 UI core `TextMeasure`（签名简单），
//! 并实现 **per-character fallback**：
//!
//! `FontdueBackend::measure` 只用单个 best_match 字体 shape，缺字字符（.notdef）
//! 被估成 `0.6 * size_px`，导致 CJK 在拉丁字体里严重低估。本适配器在 measure 时
//! 按「主族能覆盖 vs 不能覆盖」把文本分段，对每段选首个能覆盖的字体单独 measure，
//! 累加宽度。这样 CJK 字符会用 CJK 字体度量，得到正确的全角宽度。
//!
//! 注入方式：`WinitRuntime::run` 创建 driver 后，把 `Arc<FontdueBackend>` 包成
//! `FontdueTextMeasure` 注入 `host.set_text_measure()`；同时取主字体的 line_metrics
//! 注入 `host.set_font_metrics()`。这样所有 widget 的 layout 都用真实字体度量。

use std::sync::Arc;

use zero_text_foundation::{FontRequest, FontdueBackend, TextDirection, TextMeasureInput, TextMeasurer};
use zero_ui_core::widget::{TextMeasure, TextSize};

/// 包裹 `FontdueBackend` 实现 UI core 的 `TextMeasure`，带 per-character fallback。
pub struct FontdueTextMeasure {
    backend: Arc<FontdueBackend>,
    /// 已加载字体的 family 列表（fallback 顺序，首个为主族）。
    families: Vec<String>,
}

impl FontdueTextMeasure {
    pub fn new(backend: Arc<FontdueBackend>) -> Self {
        let families = backend.family_names();
        FontdueTextMeasure { backend, families }
    }

    /// 取主字体（首个 family）的 line_metrics 比率，用于基线对齐。
    ///
    /// 返回 `(ascent_ratio, descent_ratio)`，host 会乘以 `font_size` 得到实际值。
    /// `descent_ratio` 为负值（fontdue 约定）。
    pub fn line_metrics_ratio(&self, size_px: f32) -> Option<(f32, f32)> {
        let req = self.request_for_family(self.families.first()?.as_str())?;
        let input = TextMeasureInput {
            text: "Mg".into(),
            font_request: req,
            size_px,
            max_width: None,
            direction: TextDirection::Auto,
        };
        let m = self.backend.measure(&input).ok()?;
        let descent = m.height - m.ascent;
        Some((m.ascent / size_px, -(descent / size_px)))
    }

    /// 构造指定 family 的 FontRequest；family 不存在返回 None。
    fn request_for_family(&self, family: &str) -> Option<FontRequest> {
        if self.families.iter().any(|f| f.eq_ignore_ascii_case(family)) {
            Some(FontRequest::new(family))
        } else {
            None
        }
    }

    /// 对单个字符找首个能覆盖它的 family（per-character fallback）。
    fn family_for_char(&self, ch: char) -> Option<&str> {
        // ASCII 控制字符（含空格、换行）直接用主族，避免无谓查找。
        if ch.is_ascii() {
            return self.families.first().map(|s| s.as_str());
        }
        for fam in &self.families {
            if self.backend.family_covers_char(fam, ch) {
                return Some(fam.as_str());
            }
        }
        // 全部字体都不覆盖 → 用主族（产生 .notdef，由 shape_with_font 估算宽度）。
        self.families.first().map(|s| s.as_str())
    }

    /// 把文本按「每段使用同一 family」切分，返回 `[(family, segment)]`。
    ///
    /// 连续使用同一 family 的字符合并成一段，避免每个字符单独 measure（性能）。
    fn segment_by_font(&self, text: &str) -> Vec<(String, String)> {
        let mut out: Vec<(String, String)> = Vec::new();
        for ch in text.chars() {
            let fam = self.family_for_char(ch).unwrap_or("").to_string();
            if let Some(last) = out.last_mut()
                && last.0 == fam
            {
                last.1.push(ch);
                continue;
            }
            out.push((fam, ch.to_string()));
        }
        out
    }

    /// 用指定 family measure 一段文本，返回宽度。
    fn measure_segment(&self, family: &str, text: &str, size_px: f32) -> f32 {
        if text.is_empty() {
            return 0.0;
        }
        let req = self
            .request_for_family(family)
            .unwrap_or_else(|| FontRequest::new("UI"));
        let input = TextMeasureInput {
            text: text.into(),
            font_request: req,
            size_px,
            max_width: None,
            direction: TextDirection::Auto,
        };
        match self.backend.measure(&input) {
            Ok(m) => m.width,
            Err(_) => text.chars().count() as f32 * size_px * 0.6,
        }
    }
}

impl TextMeasure for FontdueTextMeasure {
    fn measure(&self, text: &str, font_size: f32) -> TextSize {
        if text.is_empty() {
            return TextSize::default();
        }
        // 按字体分段累加宽度。
        let segments = self.segment_by_font(text);
        let total_width: f32 = segments
            .iter()
            .map(|(fam, seg)| self.measure_segment(fam, seg, font_size))
            .sum();
        // 行高用主字体度量；找不到则回落 heuristic。
        let height = match self.families.first() {
            Some(fam) => {
                let req = FontRequest::new(fam);
                let input = TextMeasureInput {
                    text: "Mg".into(),
                    font_request: req,
                    size_px: font_size,
                    max_width: None,
                    direction: TextDirection::Auto,
                };
                self.backend
                    .measure(&input)
                    .map(|m| m.height)
                    .unwrap_or(font_size * 1.2)
            }
            None => font_size * 1.2,
        };
        TextSize {
            width: total_width,
            height,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 加载 Noto Sans（拉丁）+ M+ 1P（CJK）的 backend，模拟 gallery 字体栈。
    fn backend_with_fonts() -> Arc<FontdueBackend> {
        let mut b = FontdueBackend::new();
        let noto = include_bytes!("../../../../tests/wpt-runner/wpt-data/fonts/noto/noto-sans-v8-latin-regular.woff");
        if let Some(bytes) = zero_render_foundation::font::decode_woff(noto) {
            let _ = b.load_family("UI", &bytes);
        }
        let mplus = include_bytes!("../../../../tests/wpt-runner/wpt-data/fonts/mplus-1p-regular.woff");
        if let Some(bytes) = zero_render_foundation::font::decode_woff(mplus) {
            let _ = b.load_family("CJK", &bytes);
        }
        Arc::new(b)
    }

    #[test]
    fn measure_ascii_returns_nonzero_width() {
        let tm = FontdueTextMeasure::new(backend_with_fonts());
        let s = tm.measure("hello", 14.0);
        assert!(s.width > 0.0, "ascii 宽度应 > 0, got {}", s.width);
        assert!(s.height > 0.0);
    }

    #[test]
    fn measure_empty_returns_zero() {
        let tm = FontdueTextMeasure::new(backend_with_fonts());
        let s = tm.measure("", 14.0);
        assert_eq!(s.width, 0.0);
    }

    #[test]
    fn measure_longer_text_has_larger_width() {
        let tm = FontdueTextMeasure::new(backend_with_fonts());
        let short = tm.measure("hi", 14.0).width;
        let long = tm.measure("hello world", 14.0).width;
        assert!(long > short, "长文本应更宽: {short} vs {long}");
    }

    #[test]
    fn measure_cjk_uses_cjk_font_not_notdef() {
        // 关键回归：CJK 字符必须用 CJK 字体度量，宽度 ≈ font_size（全角），
        // 而非被拉丁字体估成 0.6 * size。
        let tm = FontdueTextMeasure::new(backend_with_fonts());
        let s = tm.measure("组件", 14.0);
        assert!(s.width >= 14.0, "CJK 宽度应 ≥ 一个字宽 (14px), got {}", s.width);
    }

    #[test]
    fn measure_cjk_much_wider_than_ascii_of_same_length() {
        let tm = FontdueTextMeasure::new(backend_with_fonts());
        let cjk = tm.measure("组件画廊", 14.0).width;
        let ascii = tm.measure("abcd", 14.0).width;
        assert!(
            cjk > ascii * 1.5,
            "4 个 CJK 字符应远宽于 4 个 ASCII: cjk={cjk}, ascii={ascii}"
        );
    }

    #[test]
    fn measure_mixed_ascii_cjk_segments_correctly() {
        // 混合文本：ASCII 段用拉丁字体宽度，CJK 段用 CJK 字体宽度。
        let tm = FontdueTextMeasure::new(backend_with_fonts());
        let pure_ascii = tm.measure("abc", 14.0).width;
        let pure_cjk = tm.measure("组件画", 14.0).width;
        let mixed = tm.measure("abc组件画", 14.0).width;
        let sum = pure_ascii + pure_cjk;
        // 混合应 ≈ 两段之和（允许 shaping 误差，5% 容差）。
        assert!(
            (mixed - sum).abs() < sum * 0.05,
            "混合文本宽度应 ≈ 两段之和: mixed={mixed}, sum={sum}"
        );
    }

    #[test]
    fn family_covers_char_distinguishes_ascii_and_cjk() {
        let b = backend_with_fonts();
        // UI (Noto Sans latin) 覆盖 ASCII。
        assert!(b.family_covers_char("UI", 'a'), "UI 应覆盖 ASCII 'a'");
        // UI 不应覆盖中文。
        assert!(!b.family_covers_char("UI", '中'), "UI (latin) 不应覆盖中文 '中'");
    }

    #[test]
    fn segment_by_font_splits_by_family_coverage() {
        // 验证分段逻辑：混合 ASCII + 中文应至少产生 UI 段（ASCII 必走 UI）。
        let tm = FontdueTextMeasure::new(backend_with_fonts());
        assert!(
            tm.families.len() >= 2,
            "应加载 UI + CJK 两个字体, got {:?}",
            tm.families
        );
        let segs = tm.segment_by_font("ab中文");
        assert!(!segs.is_empty());
        // 第一段（ASCII）必是 UI。
        assert_eq!(segs[0].0, "UI", "ASCII 段应分到 UI");
        assert_eq!(segs[0].1, "ab");
    }
}
