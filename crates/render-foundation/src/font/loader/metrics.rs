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
}
