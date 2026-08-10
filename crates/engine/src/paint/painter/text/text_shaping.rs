//! Paint 文本整形的受限接入与旧逐字符回退。

use zero_render_foundation::font::ShapedGlyph;

/// Paint 循环消费的单个字形。
pub(super) struct FragmentGlyph {
    pub(super) code_point: char,
    pub(super) font_glyph_index: Option<u16>,
    pub(super) x_offset: f32,
    pub(super) y_offset: f32,
    pub(super) advance_x: Option<f32>,
}

/// 整形结果或零分配的旧逐字符迭代器。
pub(super) enum FragmentGlyphs<'a> {
    Shaped(std::vec::IntoIter<ShapedGlyph>),
    Legacy(std::str::Chars<'a>),
}

impl Iterator for FragmentGlyphs<'_> {
    type Item = FragmentGlyph;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Shaped(glyphs) => {
                let glyph = glyphs.next()?;
                Some(FragmentGlyph {
                    code_point: glyph.code_point,
                    font_glyph_index: u16::try_from(glyph.glyph_id).ok(),
                    x_offset: glyph.x_offset,
                    y_offset: glyph.y_offset,
                    advance_x: Some(glyph.advance_x),
                })
            }
            Self::Legacy(chars) => Some(FragmentGlyph {
                code_point: chars.next()?,
                font_glyph_index: None,
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
pub(super) fn fragment_glyphs<'a>(font_id: u32, text: &'a str, font_size: f32, eligible: bool) -> FragmentGlyphs<'a> {
    if eligible
        && std::env::var("ZW_SHAPED_TEXT").as_deref() == Ok("1")
        && let Some(glyphs) = crate::shape_text_for_paint(font_id, text, font_size)
        && one_to_one_source_mapping(text, &glyphs)
    {
        return FragmentGlyphs::Shaped(glyphs.into_iter());
    }
    FragmentGlyphs::Legacy(text.chars())
}

fn one_to_one_source_mapping(text: &str, glyphs: &[ShapedGlyph]) -> bool {
    glyphs.len() == text.chars().count()
        && text.chars().zip(glyphs).all(|(source, glyph)| {
            source == glyph.code_point && glyph.glyph_id > 0 && u16::try_from(glyph.glyph_id).is_ok()
        })
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

    fn glyph(code_point: char, glyph_id: u32) -> ShapedGlyph {
        ShapedGlyph {
            glyph_id,
            font_id: FontId(1),
            advance_x: 8.0,
            x_offset: 0.0,
            y_offset: 0.0,
            code_point,
        }
    }

    #[test]
    fn one_to_one_mapping_accepts_plain_glyphs() {
        assert!(one_to_one_source_mapping("AV", &[glyph('A', 3), glyph('V', 4)]));
    }

    #[test]
    fn one_to_one_mapping_rejects_ligature_or_wrong_cluster() {
        assert!(!one_to_one_source_mapping("fi", &[glyph('f', 7)]));
        assert!(!one_to_one_source_mapping("éA", &[glyph('é', 8), glyph('é', 9)]));
    }
}
