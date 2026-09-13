//! Surface-local font imports. Numeric renderer IDs are never global resource IDs.

use std::{collections::HashMap, sync::Arc};
use zero_protocol::{
    IpcFontPayload, MAX_PAINT_FONT_BYTES, MAX_PAINT_FONTS, MAX_PAINT_FONTS_BYTES, PaintSnapshotParams,
};
use zero_render_foundation::font::FontLoader;

/// A bounded font registry owned by one surface/document. Dropping it releases imported bytes.
pub struct PaintFonts {
    base: Arc<FontLoader>,
    /// Immutable registry shared with the raster worker.
    pub loader: Arc<FontLoader>,
    mapping: HashMap<u32, u32>,
    /// Changes only when the resource set changes, for raster-cache invalidation.
    pub revision: u64,
}

impl PaintFonts {
    /// Start from the same platform font table as the renderer.
    pub fn new(base: Arc<FontLoader>) -> Self {
        Self {
            loader: base.clone(),
            base,
            mapping: HashMap::new(),
            revision: 0,
        }
    }

    /// Validate and atomically replace the complete downloaded-font set.
    pub fn update(&mut self, fonts: &[IpcFontPayload]) -> Result<(), String> {
        if fonts.len() > MAX_PAINT_FONTS {
            return Err("too many paint fonts".into());
        }
        let mut total = 0usize;
        let mut ids = std::collections::HashSet::new();
        for font in fonts {
            total = total.saturating_add(font.data.len());
            if font.data.is_empty() || font.data.len() > MAX_PAINT_FONT_BYTES || total > MAX_PAINT_FONTS_BYTES {
                return Err("paint font byte budget exceeded".into());
            }
            if self.base.get_font_data(font.font_id).is_some() || !ids.insert(font.font_id) {
                return Err("duplicate or platform font ID in paint resources".into());
            }
        }
        if self.mapping.len() == fonts.len()
            && fonts.iter().all(|font| {
                self.mapping.get(&font.font_id).is_some_and(|id| {
                    self.loader.face_index(*id) == font.face_index
                        && self.loader.get_font_data(*id) == Some(font.data.as_slice())
                })
            })
        {
            return Ok(());
        }
        let mut loader = self.base.duplicate();
        let mut mapping = HashMap::new();
        for font in fonts {
            let id = loader
                .load_font_at_index(&font.data, font.face_index)
                .map_err(|e| format!("invalid paint font: {e}"))?;
            mapping.insert(font.font_id, id);
        }
        self.loader = Arc::new(loader);
        self.mapping = mapping;
        self.revision = self.revision.wrapping_add(1);
        Ok(())
    }

    /// Rewrite only downloaded font IDs before converting the frame to raster primitives.
    pub fn remap(&self, paint: &mut PaintSnapshotParams) {
        for glyph in &mut paint.glyphs {
            if let Some(id) = self.mapping.get(&glyph.font_id) {
                glyph.font_id = *id;
            }
        }
    }

    /// System-only frames share the base raster namespace.
    pub fn has_downloaded_fonts(&self) -> bool {
        !self.mapping.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const AHEM: &[u8] = include_bytes!("../../../tests/wpt-runner/fonts/Ahem.ttf");

    #[test]
    fn imports_are_surface_local_atomic_and_released_by_empty_snapshot() {
        let base = Arc::new(FontLoader::new());
        let mut first = PaintFonts::new(base.clone());
        let mut second = PaintFonts::new(base);
        let font = IpcFontPayload {
            font_id: 42,
            face_index: 0,
            data: AHEM.to_vec(),
        };
        first.update(std::slice::from_ref(&font)).unwrap();
        second.update(std::slice::from_ref(&font)).unwrap();
        let old = first.loader.clone();
        first.update(std::slice::from_ref(&font)).unwrap();
        assert!(
            Arc::ptr_eq(&old, &first.loader),
            "unchanged frame must not reparse fonts"
        );
        let invalid = IpcFontPayload {
            data: vec![1, 2, 3],
            ..font.clone()
        };
        assert!(first.update(&[invalid]).is_err());
        assert!(
            Arc::ptr_eq(&old, &first.loader),
            "failed update must preserve last registry"
        );
        assert!(first.update(&[font.clone(), font]).is_err());
        assert_eq!(first.loader.measure_advance(first.mapping[&42], 'X', 40.0), 40.0);
        first.update(&[]).unwrap();
        assert!(!first.has_downloaded_fonts());
        assert!(second.has_downloaded_fonts());
    }

    #[test]
    fn same_source_id_in_two_surfaces_keeps_different_font_bytes() {
        let base = Arc::new(FontLoader::new());
        let mut first = PaintFonts::new(base.clone());
        let mut second = PaintFonts::new(base);
        first
            .update(&[IpcFontPayload {
                font_id: 42,
                face_index: 0,
                data: AHEM.to_vec(),
            }])
            .unwrap();
        let lato = include_bytes!("../../../tests/wpt-runner/fonts/Lato-Medium.ttf");
        second
            .update(&[IpcFontPayload {
                font_id: 42,
                face_index: 0,
                data: lato.to_vec(),
            }])
            .unwrap();
        assert_eq!(first.loader.measure_advance(first.mapping[&42], 'X', 40.0), 40.0);
        assert!(second.loader.measure_advance(second.mapping[&42], 'X', 40.0) < 35.0);
        assert_eq!(first.loader.get_font_data(first.mapping[&42]), Some(AHEM));
    }

    #[test]
    fn rejects_invalid_face_platform_id_empty_data_and_excess_font_count() {
        let mut loader = FontLoader::new();
        let platform_id = loader.load_font(AHEM).unwrap();
        let mut fonts = PaintFonts::new(Arc::new(loader));
        let payload = IpcFontPayload {
            font_id: 42,
            face_index: 0,
            data: AHEM.to_vec(),
        };
        assert!(
            fonts
                .update(&[IpcFontPayload {
                    font_id: platform_id,
                    ..payload.clone()
                }])
                .is_err()
        );
        assert!(
            fonts
                .update(&[IpcFontPayload {
                    face_index: u32::MAX,
                    ..payload.clone()
                }])
                .is_err()
        );
        assert!(
            fonts
                .update(&[IpcFontPayload {
                    data: vec![],
                    ..payload.clone()
                }])
                .is_err()
        );
        assert!(fonts.update(&vec![payload; MAX_PAINT_FONTS + 1]).is_err());
        assert!(!fonts.has_downloaded_fonts());
    }
}
