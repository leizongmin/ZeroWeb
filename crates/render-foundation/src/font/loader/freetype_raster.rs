//! FreeType glyph rasterization with cached variable-font faces.
//!
//! `GlyphBitmap` uses fontdue-compatible coordinates. Paint computes the bitmap
//! top-left as `(x + x_offset, baseline_y - y_offset - height)`, so FreeType's
//! `bitmap_top` maps to `y_offset = bitmap_top - height`.

use crate::font::{FontError, GlyphBitmap, OpenTypeVariation};
use std::cell::{OnceCell, RefCell};
use std::collections::HashMap;

enum GlyphSelector {
    CodePoint(char),
    GlyphIndex(u16),
}

#[derive(Clone, Copy)]
struct VariationAxis {
    tag: [u8; 4],
    min_value: f32,
    default_value: f32,
    max_value: f32,
}

struct CachedFace {
    face: freetype::Face<Vec<u8>>,
    axes: Box<[VariationAxis]>,
    applied_variations: Box<[([u8; 4], u32)]>,
}

impl CachedFace {
    fn new(face: freetype::Face<Vec<u8>>, font_bytes: &[u8], face_index: u32) -> Self {
        let axes = rustybuzz::ttf_parser::Face::parse(font_bytes, face_index)
            .map(|face| {
                face.variation_axes()
                    .into_iter()
                    .map(|axis| VariationAxis {
                        tag: axis.tag.to_bytes(),
                        min_value: axis.min_value,
                        default_value: axis.def_value,
                        max_value: axis.max_value,
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice()
            })
            .unwrap_or_default();
        Self {
            face,
            axes,
            applied_variations: Box::default(),
        }
    }

    fn apply_variations(&mut self, variations: &[OpenTypeVariation]) -> Result<(), FontError> {
        let key = variations
            .iter()
            .copied()
            .map(OpenTypeVariation::cache_key)
            .collect::<Vec<_>>()
            .into_boxed_slice();
        if self.applied_variations == key {
            return Ok(());
        }
        if variations.iter().any(|variation| !variation.value.is_finite()) {
            return Err(FontError::ParseFailed("non-finite variation coordinate".into()));
        }
        if self.axes.is_empty() {
            self.applied_variations = key;
            return Ok(());
        }

        let mut coordinates = self.axes.iter().map(|axis| axis.default_value).collect::<Vec<_>>();
        for variation in variations {
            if let Some((index, axis)) = self.axes.iter().enumerate().find(|(_, axis)| axis.tag == variation.tag) {
                coordinates[index] = variation.value.clamp(axis.min_value, axis.max_value);
            }
        }
        let coordinates = coordinates
            .into_iter()
            .map(|value| (f64::from(value) * 65536.0).round() as freetype::ffi::FT_Fixed)
            .collect::<Vec<_>>();
        let raw_face = self.face.raw() as *const _ as freetype::ffi::FT_Face;
        // https://freetype.org/freetype2/docs/reference/ft2-multiple_masters.html#ft_set_var_design_coordinates
        let error = unsafe {
            freetype::ffi::FT_Set_Var_Design_Coordinates(
                raw_face,
                coordinates.len() as freetype::ffi::FT_UInt,
                coordinates.as_ptr(),
            )
        };
        if error != freetype::ffi::FT_Err_Ok {
            return Err(FontError::ParseFailed(format!(
                "FreeType set variation coordinates: error {error}"
            )));
        }
        self.applied_variations = key;
        Ok(())
    }
}

thread_local! {
    static FT_LIB: OnceCell<freetype::Library> = const { OnceCell::new() };
    // Face<Vec<u8>> owns its font bytes. Reusing it avoids reparsing large TTC
    // files for every glyph; low-frequency font rotation clears the small cache.
    static FT_FACE_CACHE: RefCell<HashMap<u64, CachedFace>> = RefCell::new(HashMap::new());
    static RASTER_STAT: RefCell<(u64, f64)> = const { RefCell::new((0, 0.0)) };
}

const FACE_CACHE_MAX: usize = 8;

/// Sampled FNV-1a hash. The first 4 KiB and total length distinguish loaded
/// fonts without hashing multi-megabyte CJK collections on every glyph.
fn bytes_hash(bytes: &[u8], face_index: u32) -> u64 {
    let window = &bytes[..bytes.len().min(4096)];
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in window {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash ^= bytes.len() as u64;
    hash ^= u64::from(face_index) << 32;
    hash
}

fn with_lib<R>(f: impl FnOnce(&freetype::Library) -> R) -> R {
    FT_LIB.with(|cell| {
        let lib = cell.get_or_init(|| freetype::Library::init().expect("FreeType Library::init failed"));
        f(lib)
    })
}

pub(crate) fn rasterize(
    font_bytes: &[u8],
    face_index: u32,
    code_point: char,
    size: f32,
    variations: &[OpenTypeVariation],
) -> Result<GlyphBitmap, FontError> {
    if size <= 0.0 {
        return Err(FontError::NotFound(format!("non-positive size {size}")));
    }
    let raster_started = std::time::Instant::now();
    let result = rasterize_inner(
        font_bytes,
        face_index,
        GlyphSelector::CodePoint(code_point),
        size,
        variations,
    );
    static RASTER_STAT_ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let raster_stat_enabled = *RASTER_STAT_ENABLED.get_or_init(|| std::env::var("CJK_RASTER_STAT").is_ok());
    if raster_stat_enabled {
        RASTER_STAT.with(|stats| {
            let mut stats = stats.borrow_mut();
            stats.0 += 1;
            stats.1 += raster_started.elapsed().as_secs_f64() * 1000.0;
            if stats.0 % 50 == 0 {
                tracing::info!(
                    target: "zero_render_foundation::raster",
                    count = stats.0,
                    total_ms = stats.1,
                    average_ms = stats.1 / stats.0 as f64,
                    "FreeType raster statistics"
                );
            }
        });
    }
    result
}

pub(crate) fn rasterize_indexed(
    font_bytes: &[u8],
    face_index: u32,
    glyph_index: u16,
    size: f32,
    variations: &[OpenTypeVariation],
) -> Result<GlyphBitmap, FontError> {
    if size <= 0.0 {
        return Err(FontError::NotFound(format!("non-positive size {size}")));
    }
    rasterize_inner(
        font_bytes,
        face_index,
        GlyphSelector::GlyphIndex(glyph_index),
        size,
        variations,
    )
}

fn rasterize_inner(
    font_bytes: &[u8],
    face_index: u32,
    selector: GlyphSelector,
    size: f32,
    variations: &[OpenTypeVariation],
) -> Result<GlyphBitmap, FontError> {
    FT_FACE_CACHE.with(|cache_cell| {
        let mut cache = cache_cell.borrow_mut();
        with_lib(|lib| {
            let key = bytes_hash(font_bytes, face_index);
            if !cache.contains_key(&key) {
                if cache.len() >= FACE_CACHE_MAX {
                    cache.clear();
                }
                let face = lib
                    .new_memory_face2(font_bytes.to_vec(), isize::try_from(face_index).unwrap_or(0))
                    .map_err(|error| FontError::ParseFailed(format!("FreeType new_memory_face: {error:?}")))?;
                cache.insert(key, CachedFace::new(face, font_bytes, face_index));
            }
            let cached = cache.get_mut(&key).expect("face inserted above");
            cached.apply_variations(variations)?;
            let face = &cached.face;
            face.set_char_size((size * 64.0) as isize, (size * 64.0) as isize, 0, 0)
                .map_err(|error| FontError::ParseFailed(format!("FreeType set_char_size: {error:?}")))?;
            let glyph_index = match selector {
                GlyphSelector::CodePoint(code_point) => face
                    .get_char_index(code_point as usize)
                    .ok_or_else(|| FontError::NotFound(format!("no glyph index for {code_point:?}")))?,
                GlyphSelector::GlyphIndex(glyph_index) => u32::from(glyph_index),
            };
            // Full normal hinting is the established Chromium-matching path.
            face.load_glyph(glyph_index, freetype::face::LoadFlag::DEFAULT)
                .map_err(|error| FontError::ParseFailed(format!("FreeType load_glyph: {error:?}")))?;
            let glyph = face.glyph();
            glyph
                .render_glyph(freetype::RenderMode::Normal)
                .map_err(|error| FontError::ParseFailed(format!("FreeType render_glyph: {error:?}")))?;
            let bitmap = glyph.bitmap();
            let width = bitmap.width().max(0) as u16;
            let height = bitmap.rows().max(0) as u16;
            let pitch = bitmap.pitch().unsigned_abs() as usize;
            let mut data = vec![0u8; usize::from(width) * usize::from(height)];
            if width > 0 && height > 0 && pitch > 0 {
                let source = bitmap.buffer();
                let copy_width = usize::from(width).min(pitch).min(source.len());
                for row in 0..usize::from(height) {
                    let source_offset = row * pitch;
                    if source_offset + copy_width <= source.len() {
                        let destination_offset = row * usize::from(width);
                        data[destination_offset..destination_offset + copy_width]
                            .copy_from_slice(&source[source_offset..source_offset + copy_width]);
                    }
                }
            }
            let top = glyph.bitmap_top();
            Ok(GlyphBitmap {
                data,
                width,
                height,
                x_offset: glyph.bitmap_left() as i16,
                y_offset: (top - i32::from(height)) as i16,
                advance: (glyph.advance().x as f64 / 64.0) as f32,
            })
        })
    })
}

/// Measures the default-instance advance without producing a bitmap.
pub(crate) fn measure_advance(
    font_bytes: &[u8],
    face_index: u32,
    code_point: char,
    size: f32,
) -> Result<f32, FontError> {
    if size <= 0.0 {
        return Err(FontError::NotFound(format!("non-positive size {size}")));
    }
    FT_FACE_CACHE.with(|cache_cell| {
        let mut cache = cache_cell.borrow_mut();
        with_lib(|lib| {
            let key = bytes_hash(font_bytes, face_index);
            if !cache.contains_key(&key) {
                if cache.len() >= FACE_CACHE_MAX {
                    cache.clear();
                }
                let face = lib
                    .new_memory_face2(font_bytes.to_vec(), isize::try_from(face_index).unwrap_or(0))
                    .map_err(|error| FontError::ParseFailed(format!("FreeType new_memory_face: {error:?}")))?;
                cache.insert(key, CachedFace::new(face, font_bytes, face_index));
            }
            let cached = cache.get_mut(&key).expect("face inserted above");
            cached.apply_variations(&[])?;
            let face = &cached.face;
            face.set_char_size((size * 64.0) as isize, (size * 64.0) as isize, 0, 0)
                .map_err(|error| FontError::ParseFailed(format!("FreeType set_char_size: {error:?}")))?;
            let glyph_index = face
                .get_char_index(code_point as usize)
                .ok_or_else(|| FontError::NotFound(format!("no glyph index for {code_point:?}")))?;
            face.load_glyph(glyph_index, freetype::face::LoadFlag::DEFAULT)
                .map_err(|error| FontError::ParseFailed(format!("FreeType load_glyph: {error:?}")))?;
            Ok((face.glyph().advance().x as f64 / 64.0) as f32)
        })
    })
}
