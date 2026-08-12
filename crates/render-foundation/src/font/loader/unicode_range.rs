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
