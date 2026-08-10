//! WebView 页面文本选区（基于渲染 glyph 图元）。

use zero_render_foundation::primitive::GlyphPrimitive;

/// 页面 glyph 索引选区。
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

    pub fn selected_text(glyphs: &[GlyphPrimitive], sel: &GlyphSelection) -> String {
        if glyphs.is_empty() || sel.is_collapsed() {
            return String::new();
        }
        let (start, end) = sel.normalized();
        if start >= glyphs.len() {
            return String::new();
        }
        let end = end.min(glyphs.len().saturating_sub(1));
        glyphs[start..=end]
            .iter()
            .filter_map(|g| char::from_u32(g.glyph_id))
            .collect()
    }
}

/// 在文档坐标下命中 glyph 索引。
pub fn hit_test_glyph(glyphs: &[GlyphPrimitive], x: f32, y: f32) -> Option<usize> {
    let mut best: Option<(usize, f32)> = None;
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
            return Some(if x > mid && i + 1 < glyphs.len() { i + 1 } else { i });
        }
        let cx = glyph.x + width * 0.5;
        let cy = glyph.y - glyph.font_size * 0.5;
        let dist = (x - cx).powi(2) + (y - cy).powi(2);
        if best.is_none_or(|(_, d)| dist < d) {
            best = Some((i, dist));
        }
    }
    best.map(|(i, _)| i.min(glyphs.len().saturating_sub(1)))
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
    char::from_u32(glyph.glyph_id).unwrap_or('\0')
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
    use zero_render_foundation::color::Color;
    use zero_render_foundation::primitive::FontId;
    use zero_render_foundation::primitive::GlyphPrimitive;

    fn sample_glyph(x: f32, ch: char) -> GlyphPrimitive {
        GlyphPrimitive {
            x,
            y: 20.0,
            font_size: 16.0,
            color: Color::BLACK,
            glyph_id: ch as u32,
            font_glyph_index: None,
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
        assert_eq!(hit_test_glyph(&glyphs, 25.0, 20.0), Some(1));
    }

    #[test]
    fn selected_text_uses_glyph_range() {
        let glyphs = vec![sample_glyph(0.0, 'a'), sample_glyph(10.0, 'b'), sample_glyph(20.0, 'c')];
        let sel = GlyphSelection { anchor: 0, focus: 2 };
        assert_eq!(GlyphSelection::selected_text(&glyphs, &sel), "abc");
    }
}
