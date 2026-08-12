use super::FontLoader;

impl FontLoader {
    /// 返回字体 OS/2 `sxHeight / unitsPerEm`。
    pub fn x_height_aspect(&self, font_id: u32) -> Option<f32> {
        self.font_metric_aspect(font_id, crate::font::FontSizeAdjustMetric::ExHeight)
    }

    /// 返回指定 CSS `font-size-adjust` metric 相对于 em 的 aspect value。
    pub fn font_metric_aspect(&self, font_id: u32, metric: crate::font::FontSizeAdjustMetric) -> Option<f32> {
        // https://drafts.csswg.org/css-fonts-4/#font-size-adjust-prop
        let data = self.font_data.get(&font_id)?;
        let face = rustybuzz::ttf_parser::Face::parse(data, 0).ok()?;
        let units_per_em = f32::from(face.units_per_em());
        if units_per_em <= 0.0 {
            return None;
        }
        let units = match metric {
            crate::font::FontSizeAdjustMetric::ExHeight => f32::from(face.x_height()?),
            crate::font::FontSizeAdjustMetric::CapHeight => f32::from(face.capital_height()?),
            crate::font::FontSizeAdjustMetric::ChWidth => f32::from(face.glyph_hor_advance(face.glyph_index('0')?)?),
            crate::font::FontSizeAdjustMetric::IcWidth => face
                .glyph_index('\u{6c34}')
                .and_then(|glyph_id| face.glyph_hor_advance(glyph_id))
                .map_or(units_per_em, f32::from),
            crate::font::FontSizeAdjustMetric::IcHeight => face
                .glyph_index('\u{6c34}')
                .and_then(|glyph_id| face.glyph_ver_advance(glyph_id))
                .map_or(units_per_em, f32::from),
        };
        (units > 0.0).then_some(units / units_per_em)
    }

    /// 按 primary target aspect 和 resolved face aspect 计算实际字号。
    pub fn adjusted_font_size(
        &self,
        primary_font_id: u32,
        resolved_font_id: u32,
        font_size: f32,
        adjustment: crate::font::FontSizeAdjustment,
    ) -> f32 {
        // https://drafts.csswg.org/css-fonts-4/#font-size-adjust-prop
        let (metric, target) = match adjustment {
            crate::font::FontSizeAdjustment::None => return font_size,
            crate::font::FontSizeAdjustment::Adjust { metric, target } => {
                let target = match target {
                    Some(value) if value.is_finite() && value >= 0.0 => value,
                    Some(_) => return font_size,
                    None => match self.font_metric_aspect(primary_font_id, metric) {
                        Some(value) => value,
                        None => return font_size,
                    },
                };
                (metric, target)
            }
        };
        let Some(aspect) = self.font_metric_aspect(resolved_font_id, metric) else {
            return font_size;
        };
        let adjusted = font_size * target / aspect;
        if adjusted.is_finite() && adjusted >= 0.0 {
            adjusted
        } else {
            font_size
        }
    }

    /// 构建 family 的行度量与字体相对单位 aspect 映射。
    ///
    /// https://drafts.csswg.org/css-fonts-4/#first-available-font
    ///
    /// 显式 `@font-face` family 只发布首个可匹配 U+0020 的 face；若整个 family
    /// 都不能匹配空格，则不发布，让调用方继续检查 CSS family 列表中的下一项。
    pub fn build_line_metric_map(&self) -> std::collections::HashMap<String, (u32, f32, f32, f32, f32, f32)> {
        self.build_font_resolver()
            .into_iter()
            .filter_map(|(family, base_id)| {
                // 跳过 weight/style/stretch 与内部 :face=N 变体键。
                if family.contains(':') {
                    return None;
                }
                let id = if self.family_aliases.contains(&family) {
                    self.family_map.get(&family)?.iter().copied().find(|font_id| {
                        self.font_allows_code_point(*font_id, ' ')
                            && self.fonts.get(font_id).is_some_and(|font| font.has_glyph(' '))
                    })?
                } else {
                    base_id
                };
                let (ascent, descent, line_gap) = self.line_metrics_full(id, 1.0)?;
                let ex_height = self
                    .font_metric_aspect(id, crate::font::FontSizeAdjustMetric::ExHeight)
                    .unwrap_or(0.5);
                let ch_width = self
                    .font_metric_aspect(id, crate::font::FontSizeAdjustMetric::ChWidth)
                    .unwrap_or(0.5);
                Some((family, (id, ascent, descent, line_gap, ex_height, ch_width)))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metric_map_uses_first_face_matching_space() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        let mut loader = FontLoader::new();
        let no_space = loader.load_font(LATO_TTF).expect("no-space face");
        let with_space = loader.load_font(LATO_TTF).expect("space face");
        loader.register_unicode_ranges(no_space, vec![(0x41, 0x5A)]);
        loader.register_family_alias("SplitMetrics", no_space);
        loader.register_family_alias("SplitMetrics", with_space);

        let metrics = loader.build_line_metric_map();
        assert_eq!(metrics.get("SplitMetrics").map(|entry| entry.0), Some(with_space));
    }
}
