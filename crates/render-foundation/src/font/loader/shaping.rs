use super::FontLoader;
use hashbrown::HashMap;
use std::sync::Arc;

type ShapeCacheKey = (
    Vec<(
        u32,
        u64,
        u32,
        Vec<(u32, u32)>,
        Vec<crate::font::OpenTypeFeature>,
        Vec<([u8; 4], u32)>,
    )>,
    u32,
    (u8, u32),
    crate::font::TextDirection,
    Option<String>,
    String,
);
pub(super) type ShapeCache = Arc<std::sync::Mutex<HashMap<ShapeCacheKey, Vec<crate::font::ShapedGlyph>>>>;

impl FontLoader {
    /// 使用指定 face 整形文本，并缓存跨帧重复结果。
    pub fn shape_text_cached(&self, font_id: u32, text: &str, font_size: f32) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_direction(font_id, text, font_size, crate::font::TextDirection::Auto)
    }

    /// 使用指定 face 与方向整形文本，并缓存跨帧重复结果。
    pub fn shape_text_cached_with_direction(
        &self,
        font_id: u32,
        text: &str,
        font_size: f32,
        direction: crate::font::TextDirection,
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_features(font_id, text, font_size, direction, &[])
    }

    /// 使用指定 face、方向与 OpenType feature 整形文本，并缓存跨帧重复结果。
    /// R34xx：`language` 为 BCP47 标签（canvas ctx.lang 等，None = 字体默认语言系统）。
    pub fn shape_text_cached_with_features_lang(
        &self,
        font_id: u32,
        text: &str,
        font_size: f32,
        direction: crate::font::TextDirection,
        features: &[crate::font::OpenTypeFeature],
        language: Option<&str>,
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_font_ids_and_options(
            &[font_id],
            text,
            font_size,
            crate::font::TextShapingOptions {
                direction,
                features,
                language,
                ..crate::font::TextShapingOptions::default()
            },
        )
    }

    /// 使用指定 face、方向与 OpenType feature 整形文本，并缓存跨帧重复结果。
    pub fn shape_text_cached_with_features(
        &self,
        font_id: u32,
        text: &str,
        font_size: f32,
        direction: crate::font::TextDirection,
        features: &[crate::font::OpenTypeFeature],
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_features_lang(font_id, text, font_size, direction, features, None)
    }

    /// 使用有序 CSS face 列表整形文本，并缓存跨帧重复结果。
    pub fn shape_text_cached_with_font_ids(
        &self,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        direction: crate::font::TextDirection,
        features: &[crate::font::OpenTypeFeature],
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_font_ids_and_adjustment(
            font_ids,
            text,
            font_size,
            direction,
            features,
            crate::font::FontSizeAdjustment::None,
        )
    }

    /// 使用有序 CSS face 列表和 per-face `font-size-adjust` 整形文本，并缓存结果。
    pub fn shape_text_cached_with_font_ids_and_adjustment(
        &self,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        direction: crate::font::TextDirection,
        features: &[crate::font::OpenTypeFeature],
        adjustment: crate::font::FontSizeAdjustment,
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        self.shape_text_cached_with_font_ids_and_options(
            font_ids,
            text,
            font_size,
            crate::font::TextShapingOptions {
                direction,
                features,
                adjustment,
                ..crate::font::TextShapingOptions::default()
            },
        )
    }

    /// 使用有序 CSS face 列表、variation axes 和 per-face `font-size-adjust` 整形文本，并缓存结果。
    pub fn shape_text_cached_with_font_ids_and_options(
        &self,
        font_ids: &[u32],
        text: &str,
        font_size: f32,
        options: crate::font::TextShapingOptions<'_>,
    ) -> Option<Vec<crate::font::ShapedGlyph>> {
        let crate::font::TextShapingOptions {
            direction,
            features,
            variations,
            adjustment,
            language,
        } = options;
        let &primary_id = font_ids.first()?;
        let per_face_features = crate::font::shaper::per_face_features_enabled();
        let primary_features = self.resolved_font_features(primary_id, features);
        // OPTIMIZATION: 缓存 key 用字体字节 hash 而非 instance_id——instance_id 全局
        // 递增，perf/reftest 每帧新建 FontLoader 重新加载字体（id 重新分配）→ key 每帧
        // 不同 → 缓存全 miss（`word-break: break-word` 中文长文逐字 fragment 每帧全量
        // 重排，perf-gate morning paint 回归）。按内容寻址跨帧/跨 loader 稳定；
        // loader-local font ID、face descriptor scale 与 unicode-range 同时入 key：
        // cache value 保留 font ID，且同字节 @font-face 可携带不同元数据，均不能跨
        // 不兼容的 loader 实例复用。
        let font_faces = font_ids
            .iter()
            .map(|font_id| {
                self.get_font_data(*font_id).map(|data| {
                    let descriptor_scale = self.font_size_adjustments.get(font_id).copied().unwrap_or(1.0);
                    let unicode_ranges = self.font_unicode_ranges.get(font_id).cloned().unwrap_or_default();
                    (
                        *font_id,
                        crate::font::font_bytes_hash(data) ^ (u64::from(self.face_index(*font_id)) << 32),
                        descriptor_scale.to_bits(),
                        unicode_ranges,
                        if per_face_features {
                            self.resolved_font_features(*font_id, features)
                        } else {
                            primary_features.clone()
                        },
                        self.resolved_font_variations(*font_id, variations)
                            .into_iter()
                            .map(crate::font::OpenTypeVariation::cache_key)
                            .collect(),
                    )
                })
            })
            .collect::<Option<Vec<_>>>()?;
        let key = (
            font_faces,
            font_size.to_bits(),
            adjustment.cache_key(),
            direction,
            language.map(str::to_string),
            text.to_string(),
        );
        if let Some(glyphs) = self.shape_cache.lock().expect("shape cache poisoned").get(&key) {
            return Some(glyphs.clone());
        }

        let glyphs = crate::font::TextShaper::new(self, Some(crate::primitive::FontId(primary_id)))
            .shape_single_line_with_font_ids_and_options(
                font_ids,
                text,
                font_size,
                crate::font::TextShapingOptions {
                    features: if per_face_features { features } else { &primary_features },
                    ..options
                },
            );
        let mut cache = self.shape_cache.lock().expect("shape cache poisoned");
        // OPTIMIZATION: 4096 上限在 `word-break: break-word` 中文长文下逐字 fragment
        //（每汉字一 key）数帧即清空 → 每帧全量 miss 重排（perf-gate morning paint
        // 回归，R3243-F 扩大 fallback 调用面）。65536 覆盖长文全帧 key 集，跨帧命中。
        if cache.len() >= 65536 {
            cache.clear();
        }
        cache.insert(key, glyphs.clone());
        Some(glyphs)
    }

    /// 注册 face 级 OpenType feature 默认值。
    pub fn register_font_features(&mut self, font_id: u32, features: Vec<crate::font::OpenTypeFeature>) {
        if self.fonts.contains_key(&font_id) {
            self.font_features.insert(font_id, features);
            self.shape_cache.lock().expect("shape cache poisoned").clear();
        }
    }

    pub(in crate::font) fn resolved_font_features(
        &self,
        font_id: u32,
        caller_features: &[crate::font::OpenTypeFeature],
    ) -> Vec<crate::font::OpenTypeFeature> {
        let mut resolved = self.font_features.get(&font_id).cloned().unwrap_or_default();
        for feature in caller_features {
            if let Some(existing) = resolved.iter_mut().find(|existing| existing.tag == feature.tag) {
                *existing = *feature;
            } else {
                resolved.push(*feature);
            }
        }
        resolved
    }

    /// 注册 face 级 OpenType variation axis 默认值。
    pub fn register_font_variations(&mut self, font_id: u32, variations: Vec<crate::font::OpenTypeVariation>) {
        if self.fonts.contains_key(&font_id) {
            self.font_variations.insert(font_id, variations);
            self.shape_cache.lock().expect("shape cache poisoned").clear();
        }
    }

    /// 合并 face descriptor defaults 与元素 caller axes；caller 同 tag 覆盖 descriptor。
    pub fn resolved_font_variations(
        &self,
        font_id: u32,
        caller_variations: &[crate::font::OpenTypeVariation],
    ) -> Vec<crate::font::OpenTypeVariation> {
        let mut resolved = self.font_variations.get(&font_id).cloned().unwrap_or_default();
        for variation in caller_variations {
            if let Some(existing) = resolved.iter_mut().find(|existing| existing.tag == variation.tag) {
                *existing = *variation;
            } else {
                resolved.push(*variation);
            }
        }
        resolved
    }

    /// 注册 face 级 `size-adjust` 缩放因子。
    pub fn register_font_size_adjust(&mut self, font_id: u32, scale: f32) {
        if self.fonts.contains_key(&font_id) && scale.is_finite() && scale >= 0.0 {
            self.font_size_adjustments.insert(font_id, scale);
            self.shape_cache.lock().expect("shape cache poisoned").clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{FontSizeAdjustment, TextDirection};

    #[test]
    fn shared_cache_separates_face_size_adjust_descriptors() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");

        let mut base = FontLoader::new();
        let font_id = base.load_font(LATO_TTF).expect("load bundled Lato");
        let mut adjusted = base.duplicate();
        let plain = base.duplicate();
        adjusted.register_font_size_adjust(font_id, 1.5);

        let adjusted_glyphs = adjusted
            .shape_text_cached_with_font_ids_and_adjustment(
                &[font_id],
                "A",
                16.0,
                TextDirection::LeftToRight,
                &[],
                FontSizeAdjustment::None,
            )
            .expect("shape adjusted face");
        let plain_glyphs = plain
            .shape_text_cached_with_font_ids_and_adjustment(
                &[font_id],
                "A",
                16.0,
                TextDirection::LeftToRight,
                &[],
                FontSizeAdjustment::None,
            )
            .expect("shape plain face");

        assert_eq!(adjusted_glyphs[0].font_size, 24.0);
        assert_eq!(plain_glyphs[0].font_size, 16.0);
    }

    #[test]
    fn shared_cache_preserves_loader_local_font_ids() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        const AHEM_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Ahem.ttf");

        let mut first = FontLoader::new();
        let first_lato = first.load_font(LATO_TTF).expect("load Lato first");
        let mut second = first.duplicate();
        let first_ahem = first.load_font(AHEM_TTF).expect("load Ahem second");
        let second_ahem = second.load_font(AHEM_TTF).expect("load Ahem first");
        let second_lato = second.load_font(LATO_TTF).expect("load Lato second");
        assert_eq!(first_ahem, second_ahem);
        assert_ne!(first_lato, second_lato);

        let first_glyphs = first
            .shape_text_cached_with_font_ids(&[first_lato], "A", 16.0, TextDirection::LeftToRight, &[])
            .expect("shape first loader");
        let second_glyphs = second
            .shape_text_cached_with_font_ids(&[second_lato], "A", 16.0, TextDirection::LeftToRight, &[])
            .expect("shape second loader");

        assert_eq!(first_glyphs[0].font_id.0, first_lato);
        assert_eq!(second_glyphs[0].font_id.0, second_lato);
    }

    #[test]
    fn fallback_face_uses_its_descriptor_features_and_caller_override() {
        const LATO_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");

        let mut base = FontLoader::new();
        let primary = base.load_font(LATO_TTF).expect("load primary Lato");
        let secondary = base.load_font(LATO_TTF).expect("load secondary Lato");
        base.register_unicode_ranges(primary, vec![(u32::from('A'), u32::from('Z'))]);
        base.register_unicode_ranges(secondary, vec![(u32::from('a'), u32::from('z'))]);
        let mut disabled = base.duplicate();
        let mut enabled = base.duplicate();
        disabled.register_font_features(secondary, vec![crate::font::OpenTypeFeature::new(*b"liga", 0)]);
        enabled.register_font_features(secondary, vec![crate::font::OpenTypeFeature::new(*b"liga", 1)]);

        let descriptor_glyphs = disabled
            .shape_text_cached_with_font_ids(&[primary, secondary], "fi", 16.0, TextDirection::LeftToRight, &[])
            .expect("shape with secondary descriptor");
        assert_eq!(descriptor_glyphs.len(), 2);
        assert!(descriptor_glyphs.iter().all(|glyph| glyph.font_id.0 == secondary));

        let enabled_glyphs = enabled
            .shape_text_cached_with_font_ids(&[primary, secondary], "fi", 16.0, TextDirection::LeftToRight, &[])
            .expect("shape with enabled secondary descriptor");
        assert_eq!(enabled_glyphs.len(), 1);
        assert_eq!(enabled_glyphs[0].font_id.0, secondary);

        let caller_glyphs = disabled
            .shape_text_cached_with_font_ids(
                &[primary, secondary],
                "fi",
                16.0,
                TextDirection::LeftToRight,
                &[crate::font::OpenTypeFeature::new(*b"liga", 1)],
            )
            .expect("shape with caller override");
        assert_eq!(caller_glyphs.len(), 1);
        assert_eq!(caller_glyphs[0].font_id.0, secondary);
    }

    #[test]
    fn variation_axes_change_advance_and_isolate_cache_entries() {
        const ROBOTO_EXTREMO: &[u8] =
            include_bytes!("../../../../../tests/wpt-runner/fonts/RobotoExtremo-VF.subset.ttf");

        let mut loader = FontLoader::new();
        let font_id = loader
            .load_font(ROBOTO_EXTREMO)
            .expect("load RobotoExtremo variable font");
        loader.register_font_variations(font_id, vec![crate::font::OpenTypeVariation::new(*b"wdth", 75.0)]);
        let condensed = loader
            .shape_text_cached_with_font_ids_and_options(
                &[font_id],
                "text",
                32.0,
                crate::font::TextShapingOptions {
                    direction: TextDirection::LeftToRight,
                    ..crate::font::TextShapingOptions::default()
                },
            )
            .expect("shape descriptor-default condensed instance");
        let expanded = loader
            .shape_text_cached_with_font_ids_and_options(
                &[font_id],
                "text",
                32.0,
                crate::font::TextShapingOptions {
                    direction: TextDirection::LeftToRight,
                    variations: &[crate::font::OpenTypeVariation::new(*b"wdth", 125.0)],
                    ..crate::font::TextShapingOptions::default()
                },
            )
            .expect("shape caller-overridden expanded instance");

        let condensed_width: f32 = condensed.iter().map(|glyph| glyph.advance_x).sum();
        let expanded_width: f32 = expanded.iter().map(|glyph| glyph.advance_x).sum();
        assert!(
            expanded_width > condensed_width,
            "wdth axis must affect shaping advance: condensed={condensed_width}, expanded={expanded_width}"
        );
    }
}
