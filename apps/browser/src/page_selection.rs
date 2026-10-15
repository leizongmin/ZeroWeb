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

/// 判断同一行上前后两个字形之间的空白 advance 区域是否对应被布局折叠掉的词间空格。
///
/// 行内折行把词间空格消费为片段间推进、不发射 glyph（字形序列只剩前进间隙），
/// 复制文本时按「超出前字形 advance 的富余量 ≈ 空格宽度」补插一个空格：
/// 下限避开 kerning/字距噪声，上限避开无关元素同行摆放的布局级大间隙。
fn glyph_gap_has_space(previous: &GlyphPrimitive, previous_advance: f32, current: &GlyphPrimitive) -> bool {
    // 仅同一行基线（基线 y 相同）且 x 前进的相邻字形之间补插；换行与 RTL 视觉序不适用。
    if current.x <= previous.x || (current.y - previous.y).abs() > 0.5 {
        return false;
    }
    let font_size = previous.font_size.max(1.0);
    let extra = current.x - previous.x - previous_advance;
    extra >= font_size * 0.12 && extra < font_size * 0.75
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

    /// 重构选区文本。`measure_advance` 返回字形内单字符的实际前进宽度（须与
    /// paint 布局同源），用于识别词间被折叠的空格 advance 区域。
    pub fn selected_text(
        glyphs: &[GlyphPrimitive],
        sel: &GlyphSelection,
        measure_advance: impl Fn(&GlyphPrimitive, char) -> f32,
    ) -> String {
        if glyphs.is_empty() || sel.is_collapsed() {
            return String::new();
        }
        let std::ops::Range { start, end } = sel.glyph_range(glyphs.len());
        if start >= end {
            return String::new();
        }
        let mut text = String::new();
        let mut source_run = Vec::new();
        // 上一个已计位字形与其已消费 advance；词间空格不发射 glyph，仅在 x 推进中
        // 留下空格宽度（glyph_gap_has_space 补插）。
        let mut previous: Option<(&GlyphPrimitive, f32)> = None;
        for glyph in &glyphs[start..end] {
            let gap_space = previous.is_some_and(|(previous_glyph, previous_advance)| {
                glyph_gap_has_space(previous_glyph, previous_advance, glyph)
            });
            if let Some(source) = &glyph.source {
                // 簇文本按逻辑源字节序在 run 结束时统一输出；补插空格前先落盘 run，
                // 保证空格落在已输出的 run 文本之后。
                if source_run
                    .first()
                    .is_some_and(|first: &&GlyphSource| !first.same_text_run(source))
                    || gap_space
                {
                    append_source_run(&mut text, &mut source_run);
                }
                if gap_space {
                    text.push(' ');
                }
                source_run.push(source);
                let cluster_advance: f32 = source.as_str().chars().map(|ch| measure_advance(glyph, ch)).sum();
                previous = Some((glyph, cluster_advance));
            } else {
                append_source_run(&mut text, &mut source_run);
                if let Some(ch) = char::from_u32(glyph.glyph_id) {
                    if gap_space {
                        text.push(' ');
                    }
                    text.push(ch);
                    previous = Some((glyph, measure_advance(glyph, ch)));
                } else {
                    // 无效码点无法估计宽度，重置锚点避免误判下一间隙。
                    previous = None;
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
            font_variation_id: None,
            bitmap_width: None,
            bitmap_height: None,
            rotation: 0.0,
            synthetic_italic: false,
        }
    }

    /// 测量闭包：与布局估算同源的按字符 advance（无字体数据时 paint 用同一比例表）。
    fn ratio_advance(glyph: &GlyphPrimitive, ch: char) -> f32 {
        glyph.font_size * glyph_advance_ratio(ch)
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
        assert_eq!(GlyphSelection::selected_text(&glyphs, &sel, ratio_advance), "abc");
    }

    #[test]
    fn selected_text_reconstructs_spaces_from_advance_gaps() {
        // 行内折行把词间空格消费为片段间推进、不发射 glyph（gui 探针页 h1 实证）；
        // 复制文本须按 advance 富余量补插空格。
        let mut glyphs = Vec::new();
        let mut x = 0.0;
        for (i, word) in ["ZeroBrowser", "GUI", "Probe"].iter().enumerate() {
            if i > 0 {
                x += 16.0 * 0.25; // 词间空格 advance（无 glyph）
            }
            for ch in word.chars() {
                glyphs.push(sample_glyph(x, ch));
                x += 16.0 * 0.55;
            }
        }
        let sel = GlyphSelection {
            anchor: 0,
            focus: glyphs.len(),
        };
        assert_eq!(
            GlyphSelection::selected_text(&glyphs, &sel, ratio_advance),
            "ZeroBrowser GUI Probe"
        );
    }

    #[test]
    fn selected_text_does_not_insert_space_for_contiguous_or_layout_scale_gaps() {
        // 紧邻字形（kerning 级微差）与大布局间隙（同行无关元素）都不补插空格。
        let glyphs = vec![
            sample_glyph(0.0, 'a'),
            sample_glyph(16.0 * 0.55, 'b'),
            sample_glyph(100.0, 'c'),
        ];
        let sel = GlyphSelection {
            anchor: 0,
            focus: glyphs.len(),
        };
        assert_eq!(GlyphSelection::selected_text(&glyphs, &sel, ratio_advance), "abc");
    }

    #[test]
    fn selected_text_inserts_space_between_source_runs_from_advance_gap() {
        // 跨片段的 sourced 簇同样受词间空格折叠影响；补插空格落在已输出的 run 之后。
        let text_a: Arc<str> = Arc::from("AB");
        let text_b: Arc<str> = Arc::from("CD");
        let mut first = sample_glyph(0.0, 'A');
        first.source = GlyphSource::new(text_a.clone(), 0, 1);
        let mut second = sample_glyph(16.0 * 0.55, 'B');
        second.source = GlyphSource::new(text_a, 1, 2);
        let mut third = sample_glyph(16.0 * (0.55 + 0.55 + 0.25), 'C');
        third.source = GlyphSource::new(text_b, 0, 2);

        let sel = GlyphSelection { anchor: 0, focus: 3 };
        assert_eq!(
            GlyphSelection::selected_text(&[first, second, third], &sel, ratio_advance),
            "AB CD"
        );
    }

    #[test]
    fn selected_text_restores_and_deduplicates_source_clusters() {
        let mut ligature = sample_glyph(0.0, 'f');
        ligature.source = GlyphSource::new(Arc::from("fi"), 0, 2);
        let sel = GlyphSelection { anchor: 0, focus: 1 };
        assert_eq!(GlyphSelection::selected_text(&[ligature], &sel, ratio_advance), "fi");

        let source = GlyphSource::new(Arc::from("A\u{301}"), 0, 3).expect("valid source");
        let mut base = sample_glyph(0.0, 'A');
        base.source = Some(source.clone());
        let mut mark = sample_glyph(10.0, '\u{301}');
        mark.source = Some(source);
        let sel = GlyphSelection { anchor: 0, focus: 2 };
        assert_eq!(
            GlyphSelection::selected_text(&[base, mark], &sel, ratio_advance),
            "A\u{301}"
        );
    }

    #[test]
    fn selected_text_keeps_adjacent_equal_clusters_from_independent_runs() {
        let mut first = sample_glyph(0.0, 'A');
        first.source = GlyphSource::new(Arc::from("A\u{301}"), 0, 3);
        let mut second = sample_glyph(10.0, 'A');
        second.source = GlyphSource::new(Arc::from("A\u{301}"), 0, 3);

        let sel = GlyphSelection { anchor: 0, focus: 2 };
        assert_eq!(
            GlyphSelection::selected_text(&[first, second], &sel, ratio_advance),
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
            GlyphSelection::selected_text(&glyphs, &GlyphSelection { anchor: 0, focus: 3 }, ratio_advance),
            "אבג"
        );
        assert_eq!(
            GlyphSelection::selected_text(&glyphs, &GlyphSelection { anchor: 0, focus: 2 }, ratio_advance),
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
