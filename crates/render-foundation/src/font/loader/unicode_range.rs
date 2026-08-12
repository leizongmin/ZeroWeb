use super::FontLoader;

impl FontLoader {
    /// 注册 `@font-face unicode-range`；空列表表示 unrestricted。
    pub fn register_unicode_ranges(&mut self, font_id: u32, ranges: Vec<(u32, u32)>) {
        if std::env::var("ZW_FONT_UNICODE_RANGE").as_deref() == Ok("0") || ranges.is_empty() {
            self.font_unicode_ranges.remove(&font_id);
        } else {
            self.font_unicode_ranges.insert(font_id, ranges);
        }
        self.shape_cache.lock().expect("shape cache poisoned").clear();
    }

    pub(super) fn font_allows_code_point(&self, font_id: u32, code_point: char) -> bool {
        self.font_unicode_ranges.get(&font_id).is_none_or(|ranges| {
            ranges
                .iter()
                .any(|&(start, end)| (start..=end).contains(&(code_point as u32)))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{TextDirection, resolve_font_faces};

    #[test]
    fn same_family_ranges_select_distinct_faces() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        let mut loader = FontLoader::new();
        let uppercase = loader.load_font(LATO_TTF).expect("uppercase face");
        let lowercase = loader.load_font(LATO_TTF).expect("lowercase face");
        loader.register_unicode_ranges(uppercase, vec![(0x41, 0x5A)]);
        loader.register_unicode_ranges(lowercase, vec![(0x61, 0x7A)]);
        loader.register_family_alias("SplitFamily", uppercase);
        loader.register_family_alias("SplitFamily", lowercase);

        let resolver = loader.build_font_resolver();
        let (font_ids, _) =
            resolve_font_faces(&resolver, "splitfamily", false, false, 100.0).expect("same-family faces");
        assert_eq!(font_ids, vec![uppercase, lowercase]);

        let glyphs = loader
            .shape_text_cached_with_font_ids(&font_ids, "Aa", 16.0, TextDirection::LeftToRight, &[])
            .expect("split-family shape");
        assert_eq!(
            glyphs.iter().map(|glyph| glyph.font_id.0).collect::<Vec<_>>(),
            vec![uppercase, lowercase]
        );
    }
}
