//! WebView 页面文本选区（基于渲染 glyph 图元）。

use zero_render_foundation::primitive::{GlyphPrimitive, GlyphSource, TextControlBoundary};

/// 文本控件内一次 caret 边界命中。
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextControlCaretHit {
    pub utf16_offset: u32,
    pub x: f32,
    pub y: f32,
    pub height: f32,
}

/// 从 paint 生成的真实边界缓存中选择同控件、同一行且最接近指针的 caret。
pub fn hit_test_text_control_boundary(
    boundaries: &[TextControlBoundary],
    node_handle: u64,
    x: f32,
    y: f32,
) -> Option<TextControlCaretHit> {
    let line = boundaries
        .iter()
        .filter(|boundary| boundary.node_handle == node_handle)
        .min_by(|left, right| {
            let distance = |boundary: &&TextControlBoundary| {
                if y < boundary.y {
                    boundary.y - y
                } else if y > boundary.y + boundary.height {
                    y - (boundary.y + boundary.height)
                } else {
                    0.0
                }
            };
            distance(left).total_cmp(&distance(right))
        })?;
    boundaries
        .iter()
        .filter(|boundary| {
            boundary.node_handle == node_handle
                && boundary.y.to_bits() == line.y.to_bits()
                && boundary.height.to_bits() == line.height.to_bits()
        })
        .min_by(|left, right| (left.x - x).abs().total_cmp(&(right.x - x).abs()))
        .map(|boundary| TextControlCaretHit {
            utf16_offset: boundary.utf16_offset,
            x: boundary.x,
            y: boundary.y,
            height: boundary.height,
        })
}

fn append_source_run(text: &mut String, sources: &mut Vec<&GlyphSource>) {
    sources.sort_unstable_by_key(|source| (source.start, source.end));
    let mut previous = None;
    for source in sources.drain(..) {
        if previous.is_none_or(|previous: &GlyphSource| !previous.same_cluster(source)) {
            text.push_str(source.as_str());
        }
        previous = Some(source);
    }
}

/// 页面 glyph caret boundary 选区；端点范围为 `0..=glyphs.len()`。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphSelection {
    pub anchor: usize,
    pub focus: usize,
}

impl GlyphSelection {
    pub fn collapsed(index: usize) -> Self {
        Self {
            anchor: index,
            focus: index,
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.anchor == self.focus
    }

    pub fn normalized(&self) -> (usize, usize) {
        if self.anchor <= self.focus {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }

    /// 返回裁剪到 glyph 数量的半开选区范围。
    pub fn glyph_range(&self, glyph_count: usize) -> std::ops::Range<usize> {
        let (start, end) = self.normalized();
        start.min(glyph_count)..end.min(glyph_count)
    }

    pub fn selected_text(glyphs: &[GlyphPrimitive], sel: &GlyphSelection) -> String {
        if glyphs.is_empty() || sel.is_collapsed() {
            return String::new();
        }
        let std::ops::Range { start, end } = sel.glyph_range(glyphs.len());
        if start >= end {
            return String::new();
        }
        let mut text = String::new();
        let mut source_run = Vec::new();
        for glyph in &glyphs[start..end] {
            if let Some(source) = &glyph.source {
                if source_run
                    .first()
                    .is_some_and(|first: &&GlyphSource| !first.same_text_run(source))
                {
                    append_source_run(&mut text, &mut source_run);
                }
                source_run.push(source);
            } else {
                append_source_run(&mut text, &mut source_run);
                if let Some(ch) = char::from_u32(glyph.glyph_id) {
                    text.push(ch);
                }
            }
        }
        // https://drafts.csswg.org/css-writing-modes-4/#bidi-algo
        // Paint glyphs may be visual-order; copied text follows logical source byte order.
        append_source_run(&mut text, &mut source_run);
        text
    }
}

fn caret_boundary(glyphs: &[GlyphPrimitive], glyph_index: usize, after: bool) -> usize {
    let mut boundary = glyph_index + usize::from(after);
    if after {
        while boundary < glyphs.len()
            && glyphs[boundary - 1]
                .source
                .as_ref()
                .zip(glyphs[boundary].source.as_ref())
                .is_some_and(|(left, right)| left.same_cluster(right))
        {
            boundary += 1;
        }
    } else {
        while boundary > 0
            && glyphs[boundary - 1]
                .source
                .as_ref()
                .zip(glyphs[boundary].source.as_ref())
                .is_some_and(|(left, right)| left.same_cluster(right))
        {
            boundary -= 1;
        }
    }
    boundary
}

/// 在文档坐标下命中 glyph 间的 caret boundary。
pub fn hit_test_caret(glyphs: &[GlyphPrimitive], x: f32, y: f32) -> Option<usize> {
    let mut best: Option<(usize, f32, bool)> = None;
    for (i, glyph) in glyphs.iter().enumerate() {
        let Some(ch) = char::from_u32(glyph.glyph_id) else {
            continue;
        };
        if ch == '\0' {
            continue;
        }
        let top = glyph.y - glyph.font_size;
        let bottom = glyph.y + glyph.font_size * 0.25;
        let width = glyph.font_size * glyph_advance_ratio(ch);
        let right = glyph.x + width;
        if x >= glyph.x && x <= right && y >= top && y <= bottom {
            let mid = glyph.x + width * 0.5;
            return Some(caret_boundary(glyphs, i, x > mid));
        }
        let cx = glyph.x + width * 0.5;
        let cy = glyph.y - glyph.font_size * 0.5;
        let dist = (x - cx).powi(2) + (y - cy).powi(2);
        if best.is_none_or(|(_, best_dist, _)| dist < best_dist) {
            best = Some((i, dist, x > cx));
        }
    }
    best.map(|(i, _, after)| caret_boundary(glyphs, i, after))
}

/// 双击选词：扩展 glyph 索引到相邻“词”字符。
#[allow(dead_code)]
pub fn word_glyph_range(glyphs: &[GlyphPrimitive], index: usize) -> (usize, usize) {
    if glyphs.is_empty() {
        return (0, 0);
    }
    let index = index.min(glyphs.len() - 1);
    let mut start = index;
    let mut end = index;
    while start > 0 {
        let ch = glyph_char(&glyphs[start - 1]);
        if is_word_char(ch) {
            start -= 1;
        } else {
            break;
        }
    }
    while end + 1 < glyphs.len() {
        let ch = glyph_char(&glyphs[end + 1]);
        if is_word_char(ch) {
            end += 1;
        } else {
            break;
        }
    }
    (start, end + 1)
}

#[allow(dead_code)]
fn glyph_char(glyph: &GlyphPrimitive) -> char {
    glyph
        .source
        .as_ref()
        .and_then(|source| source.as_str().chars().next())
        .or_else(|| char::from_u32(glyph.glyph_id))
        .unwrap_or('\0')
}

#[allow(dead_code)]
fn is_word_char(c: char) -> bool {
    !c.is_whitespace() && c != '\0'
}

fn glyph_advance_ratio(c: char) -> f32 {
    if c.is_ascii_whitespace() {
        0.25
    } else if c.is_ascii_punctuation() {
        0.4
    } else {
        0.55
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use zero_render_foundation::color::Color;
    use zero_render_foundation::primitive::FontId;

    fn sample_glyph(x: f32, ch: char) -> GlyphPrimitive {
        GlyphPrimitive {
            x,
            y: 20.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: ch as u32,
            font_glyph_index: None,
            source: None,
            font_id: FontId(0),
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        }
    }

    #[test]
    fn hit_test_returns_nearest_glyph_outside_tight_bounds() {
        let glyphs = vec![sample_glyph(0.0, 'H'), sample_glyph(10.0, 'i')];
        assert_eq!(hit_test_caret(&glyphs, 25.0, 20.0), Some(2));
    }

    #[test]
    fn selected_text_uses_glyph_range() {
        let glyphs = vec![sample_glyph(0.0, 'a'), sample_glyph(10.0, 'b'), sample_glyph(20.0, 'c')];
        let sel = GlyphSelection { anchor: 0, focus: 3 };
        assert_eq!(sel.glyph_range(glyphs.len()), 0..3);
        assert_eq!(GlyphSelection::selected_text(&glyphs, &sel), "abc");
    }

    #[test]
    fn selected_text_restores_and_deduplicates_source_clusters() {
        let mut ligature = sample_glyph(0.0, 'f');
        ligature.source = GlyphSource::new(Arc::from("fi"), 0, 2);
        let sel = GlyphSelection { anchor: 0, focus: 1 };
        assert_eq!(GlyphSelection::selected_text(&[ligature], &sel), "fi");

        let source = GlyphSource::new(Arc::from("A\u{301}"), 0, 3).expect("valid source");
        let mut base = sample_glyph(0.0, 'A');
        base.source = Some(source.clone());
        let mut mark = sample_glyph(10.0, '\u{301}');
        mark.source = Some(source);
        let sel = GlyphSelection { anchor: 0, focus: 2 };
        assert_eq!(GlyphSelection::selected_text(&[base, mark], &sel), "A\u{301}");
    }

    #[test]
    fn selected_text_keeps_adjacent_equal_clusters_from_independent_runs() {
        let mut first = sample_glyph(0.0, 'A');
        first.source = GlyphSource::new(Arc::from("A\u{301}"), 0, 3);
        let mut second = sample_glyph(10.0, 'A');
        second.source = GlyphSource::new(Arc::from("A\u{301}"), 0, 3);

        let sel = GlyphSelection { anchor: 0, focus: 2 };
        assert_eq!(
            GlyphSelection::selected_text(&[first, second], &sel),
            "A\u{301}A\u{301}"
        );
    }

    #[test]
    fn selected_text_orders_rtl_clusters_by_logical_source_range() {
        let text: Arc<str> = Arc::from("אבג");
        let mut gimel = sample_glyph(0.0, 'ג');
        gimel.source = GlyphSource::new(text.clone(), 4, 6);
        let mut bet = sample_glyph(10.0, 'ב');
        bet.source = GlyphSource::new(text.clone(), 2, 4);
        let mut alef = sample_glyph(20.0, 'א');
        alef.source = GlyphSource::new(text, 0, 2);
        let glyphs = [gimel, bet, alef];

        assert_eq!(
            GlyphSelection::selected_text(&glyphs, &GlyphSelection { anchor: 0, focus: 3 }),
            "אבג"
        );
        assert_eq!(
            GlyphSelection::selected_text(&glyphs, &GlyphSelection { anchor: 0, focus: 2 }),
            "בג"
        );
    }

    #[test]
    fn hit_test_snaps_caret_outside_shared_cluster() {
        let source = GlyphSource::new(Arc::from("A\u{301}"), 0, 3).expect("valid source");
        let mut base = sample_glyph(0.0, 'A');
        base.source = Some(source.clone());
        let mut mark = sample_glyph(10.0, '\u{301}');
        mark.source = Some(source);
        let glyphs = [base, mark];

        assert_eq!(hit_test_caret(&glyphs, 8.0, 20.0), Some(2));
        assert_eq!(hit_test_caret(&glyphs, 10.1, 20.0), Some(0));
    }

    #[test]
    fn text_control_hit_uses_cached_utf16_boundaries() {
        let boundaries = [
            TextControlBoundary {
                node_handle: 7,
                utf16_offset: 0,
                x: 10.0,
                y: 20.0,
                height: 18.0,
            },
            TextControlBoundary {
                node_handle: 7,
                utf16_offset: 1,
                x: 14.0,
                y: 20.0,
                height: 18.0,
            },
            TextControlBoundary {
                node_handle: 7,
                utf16_offset: 3,
                x: 31.5,
                y: 20.0,
                height: 18.0,
            },
            TextControlBoundary {
                node_handle: 7,
                utf16_offset: 4,
                x: 47.0,
                y: 20.0,
                height: 18.0,
            },
        ];

        assert_eq!(
            hit_test_text_control_boundary(&boundaries, 7, 30.0, 25.0),
            Some(TextControlCaretHit {
                utf16_offset: 3,
                x: 31.5,
                y: 20.0,
                height: 18.0,
            })
        );
        assert!(hit_test_text_control_boundary(&boundaries, 8, 30.0, 25.0).is_none());
    }
}
