//! 页面内文本查找（Ctrl+F）——对最近渲染帧的 glyph 图元做匹配定位。
//!
//! 最小 API 面：零 engine/renderer 改动，消费 browser 侧已有的 `last_render`
//! glyph 图元（`glyph_id` 保留源码点，用于选择/命中测试/文本恢复）。匹配给出
//! glyph 区间与包围盒，供计数（find_set_matches）、当前项滚动定位与高亮绘制。
//!
//! 已知最小面局限（记账）：连字/复杂 shaping 场景下源码点经 `source` cluster
//! 恢复才完整，此处按逐 glyph 码点匹配——纯 ASCII 查询语义完整，含连字语言的
//! 完整匹配属后续切片。全字匹配边界为启发式（见 [`is_separated`]）：依赖渲染
//! 流不产出空白 glyph 时 x 间隙近似空格宽，等宽字体页与 CJK 相邻字（advance
//! 1em）会高估边界；RTL 流 x 递减不判边界。

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
/// 匹配两端为词边界——见 [`is_separated`] 的边界信号说明）。连续 glyph 逐一取
/// 源码点组成匹配文本。
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
        if whole_word && (!is_word_start(glyphs, &chars, start) || !is_word_end(glyphs, &chars, end)) {
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

/// 匹配起点是否处于词首：流首或前邻非词字符（合成流含空白 glyph 时）即为边界；
/// 前邻是词字符时再按 [`is_separated`] 判断中间是否夹了分隔。
fn is_word_start(glyphs: &[GlyphPrimitive], chars: &[Option<char>], start: usize) -> bool {
    start == 0 || !is_wordish(chars[start - 1]) || is_separated(glyphs, start - 1)
}

/// 匹配终点是否处于词尾：流尾或后邻非词字符即为边界；后邻是词字符时再按
/// [`is_separated`] 判断末 glyph 与后邻之间是否夹了分隔。
fn is_word_end(glyphs: &[GlyphPrimitive], chars: &[Option<char>], end: usize) -> bool {
    end >= chars.len() || !is_wordish(chars[end]) || is_separated(glyphs, end - 1)
}

/// 相邻 `idx` 与 `idx+1` 两 glyph 之间是否夹了词分隔（空格/换行/元素边界）。
///
/// 真实渲染流不产出空白 glyph（空格只贡献 advance，painter/整形层即丢弃），
/// 逐级取信号：跨行（y 突变）必为分隔；同 run 源 cluster 跳字节为精确信号
///（复杂 shaping 路径携带 `source`）；否则用 x 间隙估算——观测 advance 超过
/// 前字符常规 advance 估算一档余量，即中间存在空格宽的额外位移。
fn is_separated(glyphs: &[GlyphPrimitive], idx: usize) -> bool {
    let (prev, cur) = match (glyphs.get(idx), glyphs.get(idx + 1)) {
        (Some(prev), Some(cur)) => (prev, cur),
        _ => return true,
    };
    // 跨行必是分隔（行盒间 y 差 ≥ 半个 em 量级）。
    if (cur.y - prev.y).abs() > prev.font_size * 0.5 {
        return true;
    }
    // 同 run 源 cluster 跳字节：中间源字符（空格/换行）未产 glyph，精确。
    if let (Some(prev_source), Some(cur_source)) = (&prev.source, &cur.source)
        && prev_source.same_text_run(cur_source)
    {
        return cur_source.start > prev_source.end;
    }
    let prev_char = char::from_u32(prev.glyph_id);
    let observed = cur.x - prev.x;
    observed > prev.font_size * (estimate_advance_em(prev_char) + WORD_SEPARATION_EXTRA_EM)
}

/// 相邻 glyph 判定「夹了分隔」所需的最小额外间隙（em）。常规字体空格宽
/// ≈0.25–0.35em，三档 advance 估算的残差上限 ≈0.1em，取 0.12em 兼顾两侧。
const WORD_SEPARATION_EXTRA_EM: f32 = 0.12;

/// 常规比例字体的字符 advance 估算（em），三档分类：窄字母（真实 ≈0.28em）、
/// 宽字母（≈0.84em）、其余默认。仅用于 [`is_separated`] 的间隙估算，不参与
/// 渲染度量（度量单一 owner 在 font/shaping 层）。
fn estimate_advance_em(ch: Option<char>) -> f32 {
    match ch {
        Some('i' | 'j' | 'l' | 't' | 'f' | 'r' | 'I' | '!' | '.' | ',' | ':' | ';' | '|' | '\'' | '\u{2019}') => 0.32,
        Some('m' | 'w' | 'M' | 'W') => 0.85,
        _ => GLYPH_WIDTH_FACTOR,
    }
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

    /// 构造无空白 glyph 的渲染流（真实 painter 形态：空白只贡献 advance）。
    /// `gaps[i]` 为第 i 与第 i+1 个字符的 x 间距（px，字号 16）。
    fn stream_without_spaces(text: &str, gaps: &[f32]) -> Vec<GlyphPrimitive> {
        let mut x = 0.0;
        text.chars()
            .enumerate()
            .map(|(i, c)| {
                if i > 0 {
                    x += gaps[i - 1];
                }
                glyph(x, 20.0, c)
            })
            .collect()
    }

    #[test]
    fn whole_word_on_stream_without_space_glyphs() {
        // "concat"（词内均匀步进）：默认匹配 1 处，W• 全滤。
        let concat = stream_without_spaces("concat", &[8.0; 5]);
        assert_eq!(find_matches(&concat, "cat", false, false).len(), 1);
        assert_eq!(find_matches(&concat, "cat", false, true).len(), 0);
        // "cat cat"：空格 glyph 被 painter 丢弃，跨空格 x 间距 24px（8 + 空格宽 16）。
        // 两处 cat 均为词首词尾 → W• 匹配 2 处。
        let spaced = stream_without_spaces("catcat", &[8.0, 8.0, 24.0, 8.0, 8.0]);
        assert_eq!(find_matches(&spaced, "cat", false, false).len(), 2);
        assert_eq!(find_matches(&spaced, "cat", false, true).len(), 2);
        // 窄字母（t ≈0.28em）后跨空格的间隙也须检出：8px 步进 + 16px 空格 ≈ 0.23em 余量。
        let narrow = stream_without_spaces("atcat", &[8.0, 24.0, 8.0, 8.0, 8.0]);
        assert_eq!(find_matches(&narrow, "cat", false, true).len(), 1);
    }

    #[test]
    fn whole_word_line_break_is_boundary() {
        // 同为 "catcat" 字符流，但第二个 cat 换行（y 突变）：跨行必为边界。
        let glyphs = vec![
            glyph(0.0, 20.0, 'c'),
            glyph(8.0, 20.0, 'a'),
            glyph(16.0, 20.0, 't'),
            glyph(0.0, 60.0, 'c'),
            glyph(8.0, 60.0, 'a'),
            glyph(16.0, 60.0, 't'),
        ];
        assert_eq!(find_matches(&glyphs, "cat", false, true).len(), 2);
    }

    #[test]
    fn whole_word_same_run_source_cluster_gap() {
        // 复杂 shaping 路径携带同 run source：跳字节（空格未产 glyph）为精确信号，
        // 优先于几何估算（此处几何上无间隙也按词边界处理）。
        // 同一 run 必须共享同一 Arc 文本（same_text_run 按 Arc::ptr_eq 判定）。
        let run: std::sync::Arc<str> = "cat cat".into();
        let source = |start: u32, end: u32| {
            Some(zero_render_foundation::primitive::GlyphSource::new(run.clone(), start, end).unwrap())
        };
        let mut glyphs: Vec<GlyphPrimitive> = "catcat"
            .chars()
            .enumerate()
            .map(|(i, c)| {
                let mut g = glyph(i as f32 * 5.0, 20.0, c);
                g.source = match i {
                    0 => source(0, 1),
                    1 => source(1, 2),
                    2 => source(2, 3),
                    // 第二个 cat：cluster 从 4 跳到 5（跳过源空格 cluster 3）。
                    3 => source(4, 5),
                    4 => source(5, 6),
                    _ => source(6, 7),
                };
                g
            })
            .collect();
        assert_eq!(find_matches(&glyphs, "cat", false, true).len(), 2);
        // cluster 连续（无跳字节）时两个 cat 都在词中：同 "concat" 语义，W• 全滤。
        for (i, g) in glyphs.iter_mut().enumerate() {
            let start = i as u32;
            g.source = source(start, start + 1);
        }
        assert_eq!(find_matches(&glyphs, "cat", false, true).len(), 0);
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
