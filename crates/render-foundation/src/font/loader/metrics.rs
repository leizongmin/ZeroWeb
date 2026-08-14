use super::FontLoader;

fn ex_height_units(os2_x_height: Option<i16>, glyph_y_max: Option<i16>) -> Option<f32> {
    os2_x_height
        .or_else(|| glyph_y_max.filter(|value| *value > 0))
        .map(f32::from)
}

impl FontLoader {
    /// 返回 face 的 `@font-face size-adjust` 缩放因子；未声明时为 1。
    pub fn font_size_scale(&self, font_id: u32) -> f32 {
        self.font_size_adjustments.get(&font_id).copied().unwrap_or(1.0)
    }

    /// 返回字体 OS/2 `sxHeight / unitsPerEm`。
    pub fn x_height_aspect(&self, font_id: u32) -> Option<f32> {
        self.font_metric_aspect(font_id, crate::font::FontSizeAdjustMetric::ExHeight)
    }

    /// 返回指定 CSS `font-size-adjust` metric 相对于 em 的 aspect value。
    pub fn font_metric_aspect(&self, font_id: u32, metric: crate::font::FontSizeAdjustMetric) -> Option<f32> {
        // https://drafts.csswg.org/css-fonts-4/#font-size-adjust-prop
        let data = self.font_data.get(&font_id)?;
        let face = rustybuzz::ttf_parser::Face::parse(data, self.face_index(font_id)).ok()?;
        let units_per_em = f32::from(face.units_per_em());
        if units_per_em <= 0.0 {
            return None;
        }
        let units = match metric {
            crate::font::FontSizeAdjustMetric::ExHeight => {
                let glyph_y_max = if std::env::var("ZW_FONT_METRIC_GLYPH_FALLBACK").as_deref() == Ok("0") {
                    None
                } else {
                    // https://drafts.csswg.org/css-values-4/#ex
                    // Older OS/2 tables may omit sxHeight. Use the actual `x` glyph top
                    // instead of abandoning font-size-adjust and diverging from ex/ch geometry.
                    face.glyph_index('x')
                        .and_then(|glyph_id| face.glyph_bounding_box(glyph_id))
                        .map(|bounds| bounds.y_max)
                };
                ex_height_units(face.x_height(), glyph_y_max)?
            }
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
            crate::font::FontSizeAdjustment::None => {
                // https://drafts.csswg.org/css-fonts-5/#descdef-font-face-size-adjust
                // The property preempts the descriptor; only apply face size-adjust
                // when no font-size-adjust property is active.
                if std::env::var("ZW_FONT_FACE_SIZE_ADJUST").as_deref() == Ok("0") {
                    return font_size;
                }
                return self
                    .font_size_adjustments
                    .get(&resolved_font_id)
                    .map_or(font_size, |scale| font_size * scale);
            }
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
    pub fn build_line_metric_map(&self) -> crate::font::FontFamilyMetricMap {
        self.build_font_resolver()
            .into_iter()
            .filter_map(|(family, base_id)| {
                // 跳过 weight/style/stretch 与内部 :face=N 变体键。
                if family.contains(':') {
                    return None;
                }
                let id = if self.family_aliases.contains(&family) {
                    self.family_map.get(&family)?.iter().copied().find(|font_id| {
                        self.font_allows_code_point(*font_id, ' ') && self.font_has_glyph(*font_id, ' ')
                    })?
                } else {
                    base_id
                };
                let (ascent, descent, line_gap) = self.line_metrics_full(id, 1.0)?;
                let ex_height = self
                    .font_metric_aspect(id, crate::font::FontSizeAdjustMetric::ExHeight)
                    .unwrap_or(0.5);
                let cap_height = self
                    .font_metric_aspect(id, crate::font::FontSizeAdjustMetric::CapHeight)
                    .unwrap_or(0.8);
                let ch_width = self
                    .font_metric_aspect(id, crate::font::FontSizeAdjustMetric::ChWidth)
                    .unwrap_or(0.5);
                let ic_width = self
                    .font_metric_aspect(id, crate::font::FontSizeAdjustMetric::IcWidth)
                    .unwrap_or(1.0);
                let size_adjust = self.font_size_scale(id);
                Some((
                    family,
                    crate::font::FontFamilyMetrics {
                        font_id: id,
                        ascent,
                        descent,
                        line_gap,
                        ex_height,
                        cap_height,
                        ch_width,
                        ic_width,
                        size_adjust,
                    },
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ex_height_falls_back_to_x_glyph_when_os2_metric_is_missing() {
        assert_eq!(ex_height_units(Some(700), Some(500)), Some(700.0));
        assert_eq!(ex_height_units(None, Some(500)), Some(500.0));
        assert_eq!(ex_height_units(None, Some(0)), None);
        assert_eq!(ex_height_units(None, None), None);
    }

    #[test]
    fn font_size_adjust_property_preempts_face_size_adjust_descriptor() {
        let data = include_bytes!("../../../../../tests/wpt-runner/fonts/Ahem.ttf");
        let mut loader = FontLoader::new();
        let id = loader.load_font(data).expect("Ahem");
        loader.register_font_size_adjust(id, 1.5);

        assert_eq!(
            loader.adjusted_font_size(id, id, 40.0, crate::font::FontSizeAdjustment::None),
            60.0
        );
        assert_eq!(
            loader.adjusted_font_size(
                id,
                id,
                40.0,
                crate::font::FontSizeAdjustment::Adjust {
                    metric: crate::font::FontSizeAdjustMetric::ExHeight,
                    target: Some(0.8),
                },
            ),
            40.0
        );
    }

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
        assert_eq!(metrics.get("SplitMetrics").map(|entry| entry.font_id), Some(with_space));
    }

    #[test]
    fn metric_map_exposes_first_available_face_size_adjust() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("font");
        loader.register_family_alias("AdjustedMetrics", font_id);
        loader.register_font_size_adjust(font_id, 0.5);

        let metrics = loader.build_line_metric_map();
        let metrics = metrics.get("AdjustedMetrics").expect("family metrics");
        assert_eq!(metrics.size_adjust, 0.5);
        assert_eq!(
            Some(metrics.cap_height),
            loader.font_metric_aspect(font_id, crate::font::FontSizeAdjustMetric::CapHeight)
        );
        assert_eq!(
            Some(metrics.ic_width),
            loader.font_metric_aspect(font_id, crate::font::FontSizeAdjustMetric::IcWidth)
        );
    }
}
