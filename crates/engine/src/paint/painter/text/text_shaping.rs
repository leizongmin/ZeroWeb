//! Paint 文本整形的受限接入与旧逐字符回退。

use std::collections::HashMap;
use std::sync::Arc;
use zero_render_foundation::font::{ShapedGlyph, TextDirection};
use zero_render_foundation::primitive::GlyphSource;

fn shaped_text_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_TEXT").as_deref() != Ok("0"))
}

fn indexed_glyph_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_GLYPH_INDEX").as_deref() != Ok("0"))
}

fn shaped_positioning_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_POSITIONING").as_deref() == Ok("1"))
}

fn shaped_advance_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| shaped_positioning_enabled() || std::env::var("ZW_SHAPED_ADVANCE").as_deref() != Ok("0"))
}

fn shaped_offsets_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| shaped_positioning_enabled() || std::env::var("ZW_SHAPED_OFFSETS").as_deref() != Ok("0"))
}

fn shaped_complex_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_COMPLEX").as_deref() == Ok("1"))
}

/// Paint 循环消费的单个字形。
pub(super) struct FragmentGlyph {
    pub(super) code_point: char,
    pub(super) font_glyph_index: Option<u16>,
    pub(super) source: Option<GlyphSource>,
    pub(super) x_offset: f32,
    pub(super) y_offset: f32,
    pub(super) advance_x: Option<f32>,
}

/// 整形结果或零分配的旧逐字符迭代器。
pub(super) enum FragmentGlyphs<'a> {
    Shaped {
        glyphs: std::vec::IntoIter<ShapedGlyph>,
        sources: std::vec::IntoIter<Option<GlyphSource>>,
        indexed: bool,
        advanced: bool,
        offset: bool,
    },
    Legacy(std::str::Chars<'a>),
}

impl Iterator for FragmentGlyphs<'_> {
    type Item = FragmentGlyph;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Shaped {
                glyphs,
                sources,
                indexed,
                advanced,
                offset,
            } => {
                let glyph = glyphs.next()?;
                Some(FragmentGlyph {
                    code_point: glyph.code_point,
                    font_glyph_index: (*indexed).then(|| u16::try_from(glyph.glyph_id).ok()).flatten(),
                    source: sources.next().flatten(),
                    x_offset: if *offset { glyph.x_offset } else { 0.0 },
                    y_offset: if *offset { glyph.y_offset } else { 0.0 },
                    advance_x: (*advanced).then_some(glyph.advance_x),
                })
            }
            Self::Legacy(chars) => Some(FragmentGlyph {
                code_point: chars.next()?,
                font_glyph_index: None,
                source: None,
                x_offset: 0.0,
                y_offset: 0.0,
                advance_x: None,
            }),
        }
    }
}

/// 返回受限 shaping 结果；未启用、不满足边界或 cluster 非一一映射时回退旧路径。
///
/// https://www.w3.org/TR/css-text-3/#text-shaping
pub(super) fn fragment_glyphs<'a>(
    font_id: u32,
    text: &'a str,
    font_size: f32,
    eligible: bool,
    direction: TextDirection,
    advance_eligible: bool,
) -> FragmentGlyphs<'a> {
    let complex_enabled = shaped_complex_enabled();
    let shape_direction = effective_shape_direction(direction, complex_enabled);
    if eligible
        && shaped_text_enabled()
        && (direction != TextDirection::RightToLeft || complex_enabled)
        && let Some(glyphs) = crate::shape_text_for_paint(font_id, text, font_size, shape_direction)
    {
        let Some(complex_mapping) = mapping_mode(text, &glyphs, complex_enabled) else {
            return FragmentGlyphs::Legacy(text.chars());
        };
        let simple_mapping = !complex_mapping;
        let indexed = indexed_glyph_enabled();
        if complex_mapping && !indexed {
            return FragmentGlyphs::Legacy(text.chars());
        }
        let offsets_enabled = shaped_offsets_enabled();
        if simple_mapping && crate::text_metrics::source_mapping_requires_offsets(text, &glyphs) && !offsets_enabled {
            return FragmentGlyphs::Legacy(text.chars());
        }
        let Some(sources) = glyph_sources(text, &glyphs, complex_mapping) else {
            return FragmentGlyphs::Legacy(text.chars());
        };
        return FragmentGlyphs::Shaped {
            glyphs: glyphs.into_iter(),
            sources: sources.into_iter(),
            indexed,
            advanced: complex_mapping || shaped_advance_enabled() && (advance_eligible || shaped_positioning_enabled()),
            offset: complex_mapping || offsets_enabled,
        };
    }
    FragmentGlyphs::Legacy(text.chars())
}

fn effective_shape_direction(direction: TextDirection, complex_enabled: bool) -> TextDirection {
    if complex_enabled {
        direction
    } else {
        TextDirection::Auto
    }
}

fn mapping_mode(text: &str, glyphs: &[ShapedGlyph], allow_complex: bool) -> Option<bool> {
    if crate::text_metrics::one_to_one_source_mapping(text, glyphs) {
        Some(false)
    } else if allow_complex && source_clusters_valid(text, glyphs) {
        Some(true)
    } else {
        None
    }
}

fn source_clusters_valid(text: &str, glyphs: &[ShapedGlyph]) -> bool {
    let Ok(text_len) = u32::try_from(text.len()) else {
        return false;
    };
    !glyphs.is_empty()
        && glyphs.iter().any(|glyph| glyph.cluster == 0)
        && glyphs.iter().all(|glyph| {
            glyph.glyph_id > 0
                && u16::try_from(glyph.glyph_id).is_ok()
                && glyph.cluster < text_len
                && text.is_char_boundary(glyph.cluster as usize)
        })
}

fn glyph_sources(text: &str, glyphs: &[ShapedGlyph], all_clusters: bool) -> Option<Vec<Option<GlyphSource>>> {
    if !all_clusters && !crate::text_metrics::source_mapping_requires_offsets(text, glyphs) {
        return Some(vec![None; glyphs.len()]);
    }
    if !source_clusters_valid(text, glyphs) {
        return None;
    }
    let text: Arc<str> = Arc::from(text);
    let text_len = u32::try_from(text.len()).ok()?;
    let mut starts: Vec<u32> = glyphs.iter().map(|glyph| glyph.cluster).collect();
    starts.sort_unstable();
    starts.dedup();
    let mut cluster_counts = HashMap::new();
    for glyph in glyphs {
        *cluster_counts.entry(glyph.cluster).or_insert(0usize) += 1;
    }
    Some(
        glyphs
            .iter()
            .map(|glyph| {
                let end = starts
                    .iter()
                    .copied()
                    .find(|start| *start > glyph.cluster)
                    .unwrap_or(text_len);
                GlyphSource::new(text.clone(), glyph.cluster, end).filter(|source| {
                    all_clusters || cluster_counts[&glyph.cluster] > 1 || source.as_str().chars().nth(1).is_some()
                })
            })
            .collect(),
    )
}

/// 判断是否为须绘制占位框的非空白 Cc 控制字符。
pub(super) fn is_cc_control_char(ch: char) -> bool {
    let cp = ch as u32;
    ((cp <= 0x1F) || (0x7F..=0x9F).contains(&cp)) && !matches!(cp, 0x09 | 0x0A | 0x0C | 0x0D)
}

/// Ahem 在 half-leading 近零时使用 em-box 定位。
pub(super) fn ahem_uses_embox_position(line_height: f32, font_size: f32) -> bool {
    let half_leading = (line_height - font_size) / 2.0;
    half_leading.abs() < 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::primitive::FontId;

    fn glyph() -> ShapedGlyph {
        ShapedGlyph {
            glyph_id: 7,
            font_id: FontId(1),
            advance_x: 8.0,
            unshaped_advance_x: 9.0,
            x_offset: 2.0,
            y_offset: 3.0,
            cluster: 0,
            code_point: 'A',
        }
    }

    #[test]
    fn shaped_advance_and_offsets_are_independent() {
        let mut offsets_only = FragmentGlyphs::Shaped {
            glyphs: vec![glyph()].into_iter(),
            sources: vec![None].into_iter(),
            indexed: true,
            advanced: false,
            offset: true,
        };
        let offset = offsets_only.next().expect("offset glyph");
        assert_eq!(offset.advance_x, None);
        assert_eq!((offset.x_offset, offset.y_offset), (2.0, 3.0));

        let mut advance_only = FragmentGlyphs::Shaped {
            glyphs: vec![glyph()].into_iter(),
            sources: vec![None].into_iter(),
            indexed: true,
            advanced: true,
            offset: false,
        };
        let advance = advance_only.next().expect("advance glyph");
        assert_eq!(advance.advance_x, Some(8.0));
        assert_eq!((advance.x_offset, advance.y_offset), (0.0, 0.0));
    }

    #[test]
    fn glyph_sources_share_utf8_cluster_ranges_within_one_text_run() {
        let mut base = glyph();
        base.code_point = 'A';
        let mut mark = glyph();
        mark.code_point = '\u{301}';
        let mut trailing = glyph();
        trailing.code_point = 'B';
        trailing.cluster = 3;

        let sources = glyph_sources("A\u{301}B", &[base, mark, trailing], false).expect("valid sources");
        let base = sources[0].as_ref().expect("base source");
        let mark = sources[1].as_ref().expect("mark source");

        assert_eq!(base.as_str(), "A\u{301}");
        assert!(base.same_cluster(mark));
        assert!(sources[2].is_none());
    }

    #[test]
    fn complex_glyph_sources_cover_ligature_and_decreasing_clusters() {
        assert_eq!(
            effective_shape_direction(TextDirection::RightToLeft, false),
            TextDirection::Auto
        );
        assert_eq!(
            effective_shape_direction(TextDirection::RightToLeft, true),
            TextDirection::RightToLeft
        );
        let mut ligature = glyph();
        ligature.code_point = 'f';
        assert!(source_clusters_valid("fi", &[ligature.clone()]));
        assert_eq!(mapping_mode("fi", &[ligature.clone()], false), None);
        assert_eq!(mapping_mode("fi", &[ligature.clone()], true), Some(true));
        let ligature_sources = glyph_sources("fi", &[ligature], true).expect("ligature sources");
        assert_eq!(ligature_sources[0].as_ref().expect("ligature source").as_str(), "fi");

        let mut gimel = glyph();
        gimel.cluster = 4;
        gimel.code_point = 'ג';
        let mut bet = glyph();
        bet.cluster = 2;
        bet.code_point = 'ב';
        let mut alef = glyph();
        alef.code_point = 'א';
        let rtl = [gimel, bet, alef];
        assert!(source_clusters_valid("אבג", &rtl));
        let sources = glyph_sources("אבג", &rtl, true).expect("RTL sources");
        assert_eq!(sources[0].as_ref().expect("gimel source").as_str(), "ג");
        assert_eq!(sources[1].as_ref().expect("bet source").as_str(), "ב");
        assert_eq!(sources[2].as_ref().expect("alef source").as_str(), "א");
    }

    #[test]
    fn complex_glyph_sources_reject_non_boundary_cluster() {
        let mut invalid = glyph();
        invalid.cluster = 1;
        assert!(!source_clusters_valid("אב", &[invalid]));
    }
}
