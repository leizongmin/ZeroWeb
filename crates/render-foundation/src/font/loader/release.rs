//! Release document-owned fonts without reusing numeric IDs in raster caches.

use super::FontLoader;

impl FontLoader {
    /// Remove a font owned by the caller; existing duplicate registries remain valid.
    pub fn remove_font(&mut self, id: u32) {
        self.fonts.remove(&id);
        self.font_data.remove(&id);
        self.face_indices.remove(&id);
        self.font_instance_ids.remove(&id);
        self.font_features.remove(&id);
        self.font_variations.remove(&id);
        self.font_unicode_ranges.remove(&id);
        self.font_size_adjustments.remove(&id);
        self.family_map.retain(|_, ids| {
            ids.retain(|font| *font != id);
            !ids.is_empty()
        });
        self.family_aliases.retain(|name| self.family_map.contains_key(name));
        self.fallback_chain.retain(|font| *font != id);
        self.bitmap_glyphs.retain(|(font, _, _), _| *font != id);
        if self.ahem_font_id == Some(id) {
            self.ahem_font_id = None;
        }
        self.shape_cache.lock().expect("shape cache poisoned").clear();
        self.clear_hmtx_cache();
        self.clear_fallback_metrics_cache();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn release_removes_aliases_without_mutating_a_duplicate_or_reusing_ids() {
        let bytes = include_bytes!("../../../../../tests/wpt-runner/fonts/Ahem.ttf");
        let mut loader = FontLoader::new();
        let id = loader.load_font(bytes).unwrap();
        loader.register_family_alias("DocumentFace", id);
        let retained = loader.duplicate();
        loader.remove_font(id);
        assert!(loader.get_font_data(id).is_none());
        assert!(!loader.build_font_resolver().contains_key("DocumentFace"));
        assert!(retained.get_font_data(id).is_some());
        assert_eq!(retained.measure_advance(id, 'X', 40.0), 40.0);
        assert!(loader.load_font(bytes).unwrap() > id);
    }
}
