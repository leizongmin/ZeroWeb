//! Paint 文本整形的受限接入与旧逐字符回退。

use std::collections::HashMap;
use std::sync::Arc;
use unicode_bidi::{BidiClass, bidi_class};
use zero_layout_engine::TextFragmentSource;
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

fn shaped_rtl_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_RTL").as_deref() != Ok("0"))
}

pub(super) fn shaped_uba_rtl_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_SHAPED_UBA_RTL").as_deref() != Ok("0"))
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

/// 可安全交给单方向 shaper 的逻辑文本片段。
pub(super) struct LogicalFragmentSource<'a> {
    text: &'a str,
    source_text: Arc<str>,
    source_start: u32,
}

pub(super) fn fragment_shape_direction(
    source: Option<&TextFragmentSource>,
    style_direction: TextDirection,
    uba_rtl_enabled: bool,
) -> TextDirection {
    // https://www.w3.org/TR/css-writing-modes-3/#bidi-algo
    if uba_rtl_enabled && source.and_then(TextFragmentSource::uniform_resolved_rtl) == Some(true) {
        TextDirection::RightToLeft
    } else {
        style_direction
    }
}

/// 从布局片段恢复 RTL logical shaping 输入。
pub(super) fn logical_fragment_source(
    source: Option<&TextFragmentSource>,
    direction: TextDirection,
    text_transform_none: bool,
) -> Option<LogicalFragmentSource<'_>> {
    if direction != TextDirection::RightToLeft || !text_transform_none {
        return None;
    }
    let source = source?;
    let range = source.logical_range()?;
    let text = source.text.get(range.clone())?;
    if !single_rtl_script(text) {
        return None;
    }
    Some(LogicalFragmentSource {
        text,
        source_text: source.text.clone(),
        source_start: u32::try_from(range.start).ok()?,
    })
}

fn single_rtl_script(text: &str) -> bool {
    let mut has_rtl = false;
    for ch in text.chars() {
        match bidi_class(ch) {
            BidiClass::R | BidiClass::AL => has_rtl = true,
            BidiClass::L
            | BidiClass::EN
            | BidiClass::AN
            | BidiClass::LRE
            | BidiClass::RLE
            | BidiClass::LRO
            | BidiClass::RLO
            | BidiClass::LRI
            | BidiClass::RLI
            | BidiClass::FSI
            | BidiClass::PDI
            | BidiClass::PDF => return false,
            _ => {}
        }
    }
    has_rtl
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
    logical_source: Option<LogicalFragmentSource<'a>>,
) -> FragmentGlyphs<'a> {
    let complex_enabled = complex_run_enabled(
        direction,
        logical_source.is_some(),
        shaped_complex_enabled(),
        shaped_rtl_enabled(),
    );
    let shape_direction = effective_shape_direction(direction, complex_enabled);
    let logical_source = logical_source.filter(|_| complex_enabled && direction == TextDirection::RightToLeft);
    let shaping_text = logical_source.as_ref().map_or(text, |source| source.text);
    if eligible
        && shaped_text_enabled()
        && (direction != TextDirection::RightToLeft || complex_enabled)
        && let Some(glyphs) = crate::shape_text_for_paint(font_id, shaping_text, font_size, shape_direction, &[])
    {
        let Some(complex_mapping) = mapping_mode(shaping_text, &glyphs, complex_enabled) else {
            return FragmentGlyphs::Legacy(text.chars());
        };
        let simple_mapping = !complex_mapping;
        let indexed = indexed_glyph_enabled();
        if complex_mapping && !indexed {
            return FragmentGlyphs::Legacy(text.chars());
        }
        let offsets_enabled = shaped_offsets_enabled();
        if simple_mapping
            && crate::text_metrics::source_mapping_requires_offsets(shaping_text, &glyphs)
            && !offsets_enabled
        {
            return FragmentGlyphs::Legacy(text.chars());
        }
        let sources = if let Some(source) = logical_source {
            glyph_sources_in_run(
                shaping_text,
                &glyphs,
                complex_mapping,
                source.source_text,
                source.source_start,
            )
        } else {
            glyph_sources(shaping_text, &glyphs, complex_mapping)
        };
        let Some(sources) = sources else {
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

fn complex_run_enabled(
    direction: TextDirection,
    has_logical_source: bool,
    complex_enabled: bool,
    rtl_enabled: bool,
) -> bool {
    complex_enabled || rtl_enabled && has_logical_source && direction == TextDirection::RightToLeft
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
    glyph_sources_in_run(text, glyphs, all_clusters, Arc::from(text), 0)
}

fn glyph_sources_in_run(
    text: &str,
    glyphs: &[ShapedGlyph],
    all_clusters: bool,
    source_text: Arc<str>,
    source_start: u32,
) -> Option<Vec<Option<GlyphSource>>> {
    if !all_clusters && !crate::text_metrics::source_mapping_requires_offsets(text, glyphs) {
        return Some(vec![None; glyphs.len()]);
    }
    if !source_clusters_valid(text, glyphs) {
        return None;
    }
    let text_len = u32::try_from(text.len()).ok()?;
    let source_end = source_start.checked_add(text_len)?;
    if source_text.get(source_start as usize..source_end as usize) != Some(text) {
        return None;
    }
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
                GlyphSource::new(
                    source_text.clone(),
                    source_start.checked_add(glyph.cluster)?,
                    source_start.checked_add(end)?,
                )
                .filter(|source| {
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
    fn rtl_gate_only_enables_runs_with_logical_source() {
        assert!(!complex_run_enabled(TextDirection::RightToLeft, true, false, false));
        assert!(complex_run_enabled(TextDirection::RightToLeft, true, false, true));
        assert!(!complex_run_enabled(TextDirection::RightToLeft, false, false, true));
        assert!(!complex_run_enabled(TextDirection::LeftToRight, true, false, true));
        assert!(complex_run_enabled(TextDirection::LeftToRight, false, true, false));
    }

    #[test]
    fn uba_rtl_gate_overrides_ltr_style_only_for_uniform_rtl_fragment() {
        let rtl = TextFragmentSource {
            text: Arc::<str>::from("aאבb"),
            visual_to_logical: vec![Some(3..5), Some(1..3)],
            visual_is_rtl: vec![true, true],
        };
        assert_eq!(
            fragment_shape_direction(Some(&rtl), TextDirection::LeftToRight, false),
            TextDirection::LeftToRight
        );
        assert_eq!(
            fragment_shape_direction(Some(&rtl), TextDirection::LeftToRight, true),
            TextDirection::RightToLeft
        );
        let logical = logical_fragment_source(
            Some(&rtl),
            fragment_shape_direction(Some(&rtl), TextDirection::LeftToRight, true),
            true,
        )
        .expect("uniform RTL fragment in LTR container");
        assert_eq!(logical.text, "אב");

        let mixed = TextFragmentSource {
            text: Arc::<str>::from("aאב"),
            visual_to_logical: vec![Some(0..1), Some(3..5), Some(1..3)],
            visual_is_rtl: vec![false, true, true],
        };
        assert_eq!(
            fragment_shape_direction(Some(&mixed), TextDirection::LeftToRight, true),
            TextDirection::LeftToRight
        );
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
    fn logical_fragment_source_requires_rtl_without_text_transform() {
        let text = Arc::<str>::from("xאבגy");
        let source = TextFragmentSource {
            text: text.clone(),
            visual_to_logical: vec![Some(5..7), Some(3..5), Some(1..3)],
            visual_is_rtl: vec![true, true, true],
        };
        let logical =
            logical_fragment_source(Some(&source), TextDirection::RightToLeft, true).expect("contiguous RTL source");
        assert_eq!(logical.text, "אבג");
        assert_eq!(logical.source_start, 1);
        assert!(Arc::ptr_eq(&logical.source_text, &text));
        assert!(logical_fragment_source(Some(&source), TextDirection::LeftToRight, true).is_none());
        assert!(logical_fragment_source(Some(&source), TextDirection::RightToLeft, false).is_none());
    }

    #[test]
    fn logical_fragment_source_rejects_mixed_mapping() {
        let source = TextFragmentSource {
            text: Arc::<str>::from("aאב"),
            visual_to_logical: vec![Some(0..1), Some(3..5), Some(1..3)],
            visual_is_rtl: vec![false, true, true],
        };
        assert!(logical_fragment_source(Some(&source), TextDirection::RightToLeft, true).is_none());

        let monotonic_mixed = TextFragmentSource {
            text: Arc::<str>::from("aאב"),
            visual_to_logical: vec![Some(3..5), Some(1..3), Some(0..1)],
            visual_is_rtl: vec![true, true, true],
        };
        assert!(logical_fragment_source(Some(&monotonic_mixed), TextDirection::RightToLeft, true).is_none());
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

        let full_run = Arc::<str>::from("xאבגy");
        let sources = glyph_sources_in_run("אבג", &rtl, true, full_run.clone(), 1).expect("full-run RTL sources");
        let gimel = sources[0].as_ref().expect("gimel full-run source");
        assert_eq!((gimel.start, gimel.end), (5, 7));
        assert!(Arc::ptr_eq(&gimel.text, &full_run));
        assert!(sources.iter().flatten().all(|source| source.same_text_run(gimel)));
        assert!(glyph_sources_in_run("אבג", &rtl, true, full_run, 0).is_none());
    }

    #[test]
    fn complex_glyph_sources_reject_non_boundary_cluster() {
        let mut invalid = glyph();
        invalid.cluster = 1;
        assert!(!source_clusters_valid("אב", &[invalid]));
    }
}
