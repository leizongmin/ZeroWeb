//! 页面内文本查找（Ctrl+F）——对最近渲染帧的 glyph 图元做匹配定位。
//!
//! 最小 API 面：零 engine/renderer 改动，消费 browser 侧已有的 `last_render`
//! glyph 图元（`glyph_id` 保留源码点，用于选择/命中测试/文本恢复）。匹配给出
//! glyph 区间与包围盒，供计数（find_set_matches）、当前项滚动定位与高亮绘制。
//!
//! 已知最小面局限（记账）：连字/复杂 shaping 场景下源码点经 `source` cluster
//! 恢复才完整，此处按逐 glyph 码点匹配——纯 ASCII 查询语义完整，含连字语言的
//! 完整匹配属后续切片。

use std::ops::Range;

use zero_render_foundation::primitive::GlyphPrimitive;

/// 单个查找匹配：glyph 区间 + 文档坐标包围盒 `(x, top, w, h)`。
#[derive(Debug, Clone, PartialEq)]
pub struct PageFindMatch {
    /// 匹配覆盖的 glyph 下标区间（含头不含尾）。
    pub glyph_range: Range<usize>,
    /// 文档坐标包围盒（与 glyph 图元同坐标系）。
    pub rect: (f32, f32, f32, f32),
}

/// glyph 字元宽估算系数（与选区高亮同款：字号 × 0.55）。
const GLYPH_WIDTH_FACTOR: f32 = 0.55;

/// 在 glyph 序列中查找 `query` 的全部出现（默认大小写不敏感；`whole_word` 时要求
/// 匹配两端为非字母数字边界）。连续 glyph 逐一取源码点组成匹配文本。
pub fn find_matches(
    glyphs: &[GlyphPrimitive],
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
) -> Vec<PageFindMatch> {
    if query.is_empty() || glyphs.is_empty() {
        return Vec::new();
    }
    let needle: Vec<char> = if case_sensitive {
        query.chars().collect()
    } else {
        query.to_lowercase().chars().collect()
    };
    let chars: Vec<Option<char>> = glyphs
        .iter()
        .map(|glyph| char::from_u32(glyph.glyph_id).map(|c| if case_sensitive { c } else { c.to_ascii_lowercase() }))
        .collect();

    let mut matches = Vec::new();
    let last_start = chars.len().saturating_sub(needle.len());
    for start in 0..=last_start {
        if chars[start..start + needle.len()]
            .iter()
            .zip(&needle)
            .any(|(got, want)| *got != Some(*want))
        {
            continue;
        }
        let end = start + needle.len();
        if whole_word
            && (is_wordish(chars.get(start.wrapping_sub(1)).copied().flatten())
                || is_wordish(chars.get(end).copied().flatten()))
        {
            continue;
        }
        matches.push(PageFindMatch {
            glyph_range: start..end,
            rect: match_rect(&glyphs[start..end]),
        });
    }
    matches
}

fn is_wordish(c: Option<char>) -> bool {
    c.is_some_and(|c| c.is_alphanumeric() || c == '_')
}

fn match_rect(glyph_slice: &[GlyphPrimitive]) -> (f32, f32, f32, f32) {
    let mut min_x = f32::MAX;
    let mut min_top = f32::MAX;
    let mut max_right = f32::MIN;
    let mut max_bottom = f32::MIN;
    for glyph in glyph_slice {
        min_x = min_x.min(glyph.x);
        min_top = min_top.min(glyph.y - glyph.font_size);
        max_right = max_right.max(glyph.x + glyph.font_size * GLYPH_WIDTH_FACTOR);
        max_bottom = max_bottom.max(glyph.y);
    }
    if min_x > max_right {
        return (0.0, 0.0, 0.0, 0.0);
    }
    (min_x, min_top, max_right - min_x, max_bottom - min_top)
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_render_foundation::color::Color;
    use zero_render_foundation::primitive::FontId;

    fn glyph(x: f32, y: f32, c: char) -> GlyphPrimitive {
        GlyphPrimitive {
            x,
            y,
            font_size: 16.0,
            color: Color::WHITE,
            glyph_id: c as u32,
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

    fn sample_text() -> Vec<GlyphPrimitive> {
        "Needle in the haystack, needle twice."
            .chars()
            .enumerate()
            .map(|(i, c)| glyph(i as f32 * 8.0, 20.0, c))
            .collect()
    }

    #[test]
    fn finds_all_case_insensitive_occurrences() {
        let matches = find_matches(&sample_text(), "needle", false, false);
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].glyph_range, 0..6);
        assert_eq!(matches[1].glyph_range, 24..30);
    }

    #[test]
    fn case_sensitive_drops_lowercase_hits() {
        // 文本 = "Needle … needle twice."：小写查询只命中第二处。
        assert_eq!(find_matches(&sample_text(), "needle", true, false).len(), 1);
        assert_eq!(find_matches(&sample_text(), "Needle", true, false).len(), 1);
    }

    #[test]
    fn whole_word_requires_boundaries() {
        let text: Vec<GlyphPrimitive> = "needle needles"
            .chars()
            .enumerate()
            .map(|(i, c)| glyph(i as f32 * 8.0, 20.0, c))
            .collect();
        assert_eq!(find_matches(&text, "needle", false, true).len(), 1);
        assert_eq!(find_matches(&text, "needle", false, false).len(), 2);
    }

    #[test]
    fn empty_query_and_empty_page_yield_no_matches() {
        assert!(find_matches(&sample_text(), "", false, false).is_empty());
        assert!(find_matches(&[], "needle", false, false).is_empty());
    }

    #[test]
    fn rect_spans_the_match() {
        let matches = find_matches(&sample_text(), "Needle", true, false);
        let (x, top, w, h) = matches[0].rect;
        assert_eq!(x, 0.0);
        assert_eq!(top, 4.0);
        // 并集宽 = 末 glyph x(40) + 字宽 8.8 − 首 glyph x(0)。
        assert!((w - (5.0 * 8.0 + 16.0 * 0.55)).abs() < 0.01);
        assert_eq!(h, 16.0);
    }
}
