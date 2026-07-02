//! 具体 fontdue + rustybuzz 文本后端（spec IF-008 / DC-11 / TBD-8）。
//!
//! 给 [`FontProvider`] / [`TextShaper`] / [`TextMeasurer`] trait 提供**真实**实现：
//! - fontdue：字体解析、glyph 度量（advance/ascent/descent）、光栅化数据来源；
//! - rustybuzz：OpenType shaping（GSUB/GPOS，连字/kerning/mark）。
//!
//! 本 crate 不依赖任何 UI/浏览器业务 crate（DC-1），因此 `ui/render` 与 `zero-webview`
//! 可同时复用本后端，得到一致的 fallback chain 与 shaping 结果（DC-11 关键不变量）。
//!
//! M2 范围：单主字体 shaping + 通用族 fallback chain + fontdue 行度量；完整 per-glyph
//! 字体级 fallback（逐字符切换字体）与 bidi 自动方向在后续里程碑补全。

use crate::diagnostics::TextError;
use crate::font_database::{FontMatch, FontProvider, FontSource};
use crate::font_request::{FontFamily, FontId, FontRequest, FontStretch, TextDirection};
use crate::shaping::{GlyphRun, PositionedGlyph, ShapeInput, ShapedText, TextShaper};
use crate::text_measure::{TextMeasureInput, TextMeasurer, TextMetrics};

/// 已加载字体条目。`data` 保留原始字节供 rustybuzz `Face::from_slice` 使用。
struct LoadedFont {
    id: FontId,
    /// 调用方声明的族名（CSS/UI token → FontProvider 匹配依据）。
    family: FontFamily,
    data: Vec<u8>,
    font: fontdue::Font,
}

/// fontdue + rustybuzz 文本后端。
///
/// 持有已加载字体，同时实现 [`FontProvider`] / [`TextShaper`] / [`TextMeasurer`]，
/// 是 UI SDK 与 WebView 共享的统一文本能力对象（DC-11）。
#[derive(Default)]
pub struct FontdueBackend {
    fonts: Vec<LoadedFont>,
    next_id: u32,
}

impl FontdueBackend {
    pub fn new() -> FontdueBackend {
        FontdueBackend::default()
    }

    /// 已加载字体数。
    pub fn len(&self) -> usize {
        self.fonts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fonts.is_empty()
    }

    /// 加载内存字体，用调用方声明的族名注册（CSS `@font-face` / 应用打包字体均走此路径）。
    /// 返回分配的稳定 [`FontId`]。
    pub fn load_family(&mut self, family: &str, data: &[u8]) -> Result<FontId, TextError> {
        if data.is_empty() {
            return Err(TextError::InvalidRequest("empty font data".into()));
        }
        let font = fontdue::Font::from_bytes(data, fontdue::FontSettings::default())
            .map_err(|_| TextError::InvalidRequest("fontdue parse failed".into()))?;
        let id = FontId(self.next_id);
        self.next_id += 1;
        self.fonts.push(LoadedFont {
            id,
            family: FontFamily::new(family),
            data: data.to_vec(),
            font,
        });
        Ok(id)
    }

    /// 按 [`FontRequest`] 的候选族顺序匹配首个已加载字体；无精确匹配时回退到首个已加载字体。
    fn best_match(&self, request: &FontRequest) -> Option<&LoadedFont> {
        for fam in &request.families {
            if let Some(f) = self.fonts.iter().find(|f| f.family.0.eq_ignore_ascii_case(&fam.0)) {
                return Some(f);
            }
        }
        self.fonts.first()
    }

    /// 字体是否覆盖该字符（glyph_index != 0）。
    fn covers(font: &LoadedFont, ch: char) -> bool {
        font.font.lookup_glyph_index(ch) != 0
    }

    fn match_of(font: &LoadedFont) -> FontMatch {
        FontMatch {
            id: font.id,
            family: font.family.clone(),
            // M2：weight/style 取默认（fontdue 不暴露 OS/2 weight；调用方可后续按 id 覆盖）。
            weight: crate::font_request::FontWeight::NORMAL,
            style: crate::font_request::FontStyle::Normal,
            stretch: FontStretch::NORMAL,
            source: FontSource::Memory,
        }
    }
}

impl FontProvider for FontdueBackend {
    fn query(&self, request: &FontRequest) -> Result<FontMatch, TextError> {
        let font = self.best_match(request).ok_or(TextError::FontNotFound)?;
        Ok(Self::match_of(font))
    }

    fn fallback_chain(&self, text: &str, request: &FontRequest) -> Result<Vec<FontMatch>, TextError> {
        let primary = self.best_match(request).ok_or(TextError::FontNotFound)?;
        let mut chain = vec![Self::match_of(primary)];
        // 追加其它能覆盖文本中字符的已加载字体（去重），保证 shaping 有 fallback 候选。
        for f in &self.fonts {
            if f.id == primary.id {
                continue;
            }
            if text.chars().any(|c| Self::covers(f, c)) {
                chain.push(Self::match_of(f));
            }
        }
        Ok(chain)
    }
}

impl TextShaper for FontdueBackend {
    fn shape(&self, input: &ShapeInput) -> Result<ShapedText, TextError> {
        if input.size_px <= 0.0 {
            return Err(TextError::InvalidRequest("size_px must be > 0".into()));
        }
        if input.text.is_empty() {
            return Ok(ShapedText {
                runs: Vec::new(),
                total_advance_x: 0.0,
                total_advance_y: 0.0,
            });
        }
        let primary = self.best_match(&input.font_request).ok_or(TextError::FontNotFound)?;
        let glyphs = shape_with_font(primary, &input.text, input.size_px, input.direction);
        let total_advance_x = glyphs.iter().map(|g| g.x_advance).sum();
        let total_advance_y = glyphs.iter().map(|g| g.y_advance).sum();
        Ok(ShapedText {
            runs: vec![GlyphRun {
                font: Self::match_of(primary),
                font_size_px: input.size_px,
                glyphs,
            }],
            total_advance_x,
            total_advance_y,
        })
    }
}

impl TextMeasurer for FontdueBackend {
    fn measure(&self, input: &TextMeasureInput) -> Result<TextMetrics, TextError> {
        if input.size_px <= 0.0 {
            return Err(TextError::InvalidRequest("size_px must be > 0".into()));
        }
        let font = self.best_match(&input.font_request).ok_or(TextError::FontNotFound)?;
        let size = input.size_px;

        let glyphs = shape_with_font(font, &input.text, size, input.direction);
        // 宽度 + 行数（可选自动换行：超宽硬折行）。
        let (width, line_count) = match input.max_width {
            Some(max_w) if max_w > 0.0 => wrap_width_and_lines(&glyphs, max_w),
            _ => {
                let w = glyphs.iter().map(|g| g.x_advance).sum::<f32>();
                (w, if input.text.is_empty() { 0 } else { 1 })
            }
        };

        // fontdue 行度量：ascent 为正（基线上），descent 为负（基线下）。
        let (ascent, descent) = match font.font.horizontal_line_metrics(size) {
            Some(m) => (m.ascent, -m.descent),
            None => (size * 0.8, size * 0.2),
        };
        let line_height = ascent + descent;
        Ok(TextMetrics {
            width,
            height: line_count as f32 * line_height,
            ascent,
            descent,
            line_count,
        })
    }
}

/// 用 rustybuzz 对单字体 shaping；Face 解析失败时回退到 fontdue 逐字符映射。
fn shape_with_font(font: &LoadedFont, text: &str, size_px: f32, direction: TextDirection) -> Vec<PositionedGlyph> {
    let Some(face) = rustybuzz::Face::from_slice(&font.data, 0) else {
        return shape_fallback_per_char(font, text, size_px);
    };

    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(match direction {
        TextDirection::Rtl => rustybuzz::Direction::RightToLeft,
        TextDirection::Ltr | TextDirection::Auto => rustybuzz::Direction::LeftToRight,
    });
    let glyph_buffer = rustybuzz::shape(&face, &[], buffer);
    let infos = glyph_buffer.glyph_infos();
    let positions = glyph_buffer.glyph_positions();

    let upem = font.font.units_per_em();
    let px_per_unit = if upem > 0.0 { size_px / upem } else { 0.0 };

    let mut out = Vec::with_capacity(infos.len());
    for (info, pos) in infos.iter().zip(positions.iter()) {
        // glyph_id == 0 → .notdef（字体缺字），用 0.6·size 估算宽度避免塌陷。
        let x_advance = if info.glyph_id == 0 {
            size_px * 0.6
        } else {
            font.font
                .metrics_indexed(info.glyph_id.min(u16::MAX as u32) as u16, size_px)
                .advance_width
        };
        out.push(PositionedGlyph {
            glyph_id: info.glyph_id,
            cluster: info.cluster,
            x_advance,
            y_advance: pos.y_advance as f32 * px_per_unit,
            x_offset: pos.x_offset as f32 * px_per_unit,
            y_offset: pos.y_offset as f32 * px_per_unit,
        });
    }
    out
}

/// fontdue 逐字符 shaping 回退（rustybuzz 无法解析字体表时）。
fn shape_fallback_per_char(font: &LoadedFont, text: &str, size_px: f32) -> Vec<PositionedGlyph> {
    text.char_indices()
        .map(|(byte_offset, ch)| {
            let glyph_index = font.font.lookup_glyph_index(ch);
            let x_advance = if glyph_index == 0 {
                size_px * 0.6
            } else {
                font.font.metrics_indexed(glyph_index, size_px).advance_width
            };
            PositionedGlyph {
                glyph_id: glyph_index as u32,
                cluster: byte_offset as u32,
                x_advance,
                y_advance: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
            }
        })
        .collect()
}

/// 贪心硬折行：累计 advance 超过 `max_width` 即折行。返回 (最宽行宽, 行数)。
fn wrap_width_and_lines(glyphs: &[PositionedGlyph], max_width: f32) -> (f32, u32) {
    let mut lines = 1u32;
    let mut line_w = 0.0f32;
    let mut widest = 0.0f32;
    for g in glyphs {
        if line_w + g.x_advance > max_width && line_w > 0.0 {
            widest = widest.max(line_w);
            line_w = g.x_advance;
            lines += 1;
        } else {
            line_w += g.x_advance;
        }
    }
    widest = widest.max(line_w);
    (widest, lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font_request::{FontRequest, TextDirection};
    use crate::shaping::ShapeInput;
    use crate::text_measure::TextMeasureInput;

    /// WPT 标准测试字体（每个字符为 1em 实心方块），跨平台确定性。
    /// 路径相对 crate 根：foundation/text → ../../../tests/wpt-runner/fonts/Ahem.ttf。
    const AHEM: &[u8] = include_bytes!("../../../tests/wpt-runner/fonts/Ahem.ttf");

    fn backend_with_ahem() -> FontdueBackend {
        let mut b = FontdueBackend::new();
        b.load_family("Ahem", AHEM).expect("Ahem.ttf should parse via fontdue");
        b
    }

    #[test]
    fn load_family_assigns_stable_increasing_ids() {
        let mut b = backend_with_ahem();
        let id2 = b.load_family("sans-serif", AHEM).unwrap();
        assert_eq!(id2, FontId(1));
        assert_eq!(b.len(), 2);
    }

    #[test]
    fn load_family_rejects_empty_data() {
        let mut b = FontdueBackend::new();
        assert!(matches!(b.load_family("x", &[]), Err(TextError::InvalidRequest(_))));
    }

    #[test]
    fn query_resolves_loaded_family_and_falls_back() {
        let b = backend_with_ahem();
        let m = b.query(&FontRequest::new("Ahem")).unwrap();
        assert_eq!(m.family, FontFamily::new("Ahem"));
        assert_eq!(m.source, FontSource::Memory);
        // 未加载的族 → 回退到首个已加载字体（不报错）。
        let m2 = b.query(&FontRequest::new("DoesNotExist")).unwrap();
        assert_eq!(m2.id, FontId(0));
    }

    #[test]
    fn query_errors_when_no_font_loaded() {
        let b = FontdueBackend::new();
        assert!(matches!(b.query(&FontRequest::new("x")), Err(TextError::FontNotFound)));
    }

    #[test]
    fn fallback_chain_starts_with_primary() {
        let mut b = backend_with_ahem();
        let _ = b.load_family("sans-serif", AHEM).unwrap();
        let chain = b.fallback_chain("ABC", &FontRequest::new("Ahem")).unwrap();
        assert_eq!(chain[0].family, FontFamily::new("Ahem"));
        // 其它能覆盖 ABC 的已加载字体也进 chain。
        assert!(!chain.is_empty());
    }

    #[test]
    fn shape_ahem_one_glyph_per_ascii_char() {
        let b = backend_with_ahem();
        let shaped = b
            .shape(&ShapeInput {
                text: "ABC".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: 16.0,
                direction: TextDirection::Ltr,
                script: None,
                scale_factor: 1.0,
            })
            .unwrap();
        assert_eq!(shaped.glyph_count(), 3, "ASCII 3 字符 → 3 glyph");
        assert!(shaped.total_advance_x > 0.0);
        // Ahem 每字符 advance ≈ font_size（1em 方块）。
        for g in &shaped.runs[0].glyphs {
            assert!(g.x_advance > 0.0);
            assert!(g.glyph_id != 0, "Ahem 应覆盖 ASCII");
        }
    }

    #[test]
    fn shape_rejects_nonpositive_size() {
        let b = backend_with_ahem();
        let result = b.shape(&ShapeInput {
            text: "A".into(),
            font_request: FontRequest::new("Ahem"),
            size_px: 0.0,
            direction: TextDirection::Ltr,
            script: None,
            scale_factor: 1.0,
        });
        assert!(matches!(result, Err(TextError::InvalidRequest(_))));
    }

    #[test]
    fn shape_empty_text_yields_no_runs() {
        let b = backend_with_ahem();
        let shaped = b
            .shape(&ShapeInput {
                text: "".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: 16.0,
                direction: TextDirection::Ltr,
                script: None,
                scale_factor: 1.0,
            })
            .unwrap();
        assert_eq!(shaped.glyph_count(), 0);
        assert_eq!(shaped.total_advance_x, 0.0);
    }

    #[test]
    fn shape_wider_at_larger_size() {
        let b = backend_with_ahem();
        let mk = |size| {
            b.shape(&ShapeInput {
                text: "Hello".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: size,
                direction: TextDirection::Ltr,
                script: None,
                scale_factor: 1.0,
            })
            .unwrap()
            .total_advance_x
        };
        let w16 = mk(16.0);
        let w32 = mk(32.0);
        assert!(w32 > w16, "32px 应宽于 16px: {w32} vs {w16}");
    }

    #[test]
    fn measure_returns_positive_metrics_single_line() {
        let b = backend_with_ahem();
        let m = b
            .measure(&TextMeasureInput {
                text: "Hello".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: 16.0,
                max_width: None,
                direction: TextDirection::Ltr,
            })
            .unwrap();
        assert!(m.width > 0.0);
        assert!(m.ascent > 0.0);
        assert!(m.descent >= 0.0);
        assert!(m.height >= m.ascent);
        assert_eq!(m.line_count, 1);
    }

    #[test]
    fn measure_wraps_when_max_width_exceeded() {
        let b = backend_with_ahem();
        // "AAAAAAAAAA"（10 字符）at 16px → 约 160px 宽；max_width 50 → 应折多行。
        let m = b
            .measure(&TextMeasureInput {
                text: "AAAAAAAAAA".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: 16.0,
                max_width: Some(50.0),
                direction: TextDirection::Ltr,
            })
            .unwrap();
        assert!(m.line_count >= 2, "应至少 2 行，实际 {}", m.line_count);
        assert!(m.width <= 50.0 + 16.0, "最宽行不应超 max_width+1 字符");
    }

    #[test]
    fn measure_rejects_nonpositive_size() {
        let b = backend_with_ahem();
        let result = b.measure(&TextMeasureInput {
            text: "A".into(),
            font_request: FontRequest::new("Ahem"),
            size_px: -1.0,
            max_width: None,
            direction: TextDirection::Ltr,
        });
        assert!(matches!(result, Err(TextError::InvalidRequest(_))));
    }

    #[test]
    fn wrap_helper_breaks_on_overflow() {
        // 每字符 advance 10，max_width 25 → 3 个字符后（30>25）折行。
        let glyphs: Vec<PositionedGlyph> = (0..5)
            .map(|i| PositionedGlyph {
                glyph_id: i,
                cluster: i,
                x_advance: 10.0,
                y_advance: 0.0,
                x_offset: 0.0,
                y_offset: 0.0,
            })
            .collect();
        let (w, lines) = wrap_width_and_lines(&glyphs, 25.0);
        assert!(lines >= 2, "应折行，实际 {lines}");
        assert!(w <= 25.0 + 10.0);
    }

    #[test]
    fn glyph_run_advance_x_and_metrics_line_height() {
        // 覆盖 shaping.rs::GlyphRun::advance_x 与 text_measure.rs::TextMetrics::line_height。
        let run = GlyphRun {
            font: FontMatch {
                id: FontId(0),
                family: FontFamily::new("Ahem"),
                weight: crate::font_request::FontWeight::NORMAL,
                style: crate::font_request::FontStyle::Normal,
                stretch: FontStretch::NORMAL,
                source: FontSource::Memory,
            },
            font_size_px: 16.0,
            glyphs: vec![
                PositionedGlyph {
                    glyph_id: 1,
                    cluster: 0,
                    x_advance: 10.0,
                    y_advance: 0.0,
                    x_offset: 0.0,
                    y_offset: 0.0,
                },
                PositionedGlyph {
                    glyph_id: 2,
                    cluster: 1,
                    x_advance: 12.0,
                    y_advance: 0.0,
                    x_offset: 0.0,
                    y_offset: 0.0,
                },
            ],
        };
        assert_eq!(run.advance_x(), 22.0);
        let metrics = TextMetrics {
            width: 22.0,
            height: 20.0,
            ascent: 14.0,
            descent: 4.0,
            line_count: 1,
        };
        assert_eq!(metrics.line_height(), 18.0);
    }

    #[test]
    fn text_blob_caret_mapping_from_real_shaped() {
        // 用真实 backend shape 出 "ABC"，构造 TextBlob，验证 caret 映射（覆盖 text_blob.rs）。
        let b = backend_with_ahem();
        let input = ShapeInput {
            text: "ABC".into(),
            font_request: FontRequest::new("Ahem"),
            size_px: 16.0,
            direction: TextDirection::Ltr,
            script: None,
            scale_factor: 1.0,
        };
        let shaped = b.shape(&input).unwrap();
        let metrics = b
            .measure(&TextMeasureInput {
                text: "ABC".into(),
                font_request: FontRequest::new("Ahem"),
                size_px: 16.0,
                max_width: None,
                direction: TextDirection::Ltr,
            })
            .unwrap();
        let blob = crate::text_blob::TextBlob::new(shaped.clone(), metrics);
        // 起始 caret 在最左。
        assert_eq!(blob.caret_x_for_byte(0), 0.0);
        // 超出文本末尾的 caret ≤ 总 advance。
        let end = blob.caret_x_for_byte(usize::MAX);
        assert!(end <= shaped.total_advance_x + 1.0);
        // 中间偏移的 caret 介于起止之间。
        let mid = blob.caret_x_for_byte(1);
        assert!(mid > 0.0 && mid < shaped.total_advance_x);
    }
}
