use super::FontLoader;
use hashbrown::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

/// hmtx 字符测量缓存键：完整字体链 + 字号 bits + 字符。
type HmtxCacheKey = (Arc<[u32]>, u32, char);
pub(super) type HmtxCache = Arc<Mutex<HashMap<HmtxCacheKey, f32>>>;

impl FontLoader {
    /// 批量测量整段文本的 hmtx advance（无 shaping 上下文）。
    ///
    /// 与 rustybuzz 的 `unshaped_advance_x` 同源（同一 hmtx 表 × size/upem）。
    /// hmtx advance 可逐字符相加，因此缓存字符而不是完整文本，使不同 run 可共享
    /// 重复字符，同时避免为每次测量分配文本 `String`。
    pub fn measure_text_hmtx(&self, font_ids: &[u32], text: &str, font_size: f32) -> f32 {
        if font_size <= 0.0 || text.is_empty() {
            return 0.0;
        }

        let mut chain = font_ids.to_vec();
        for &id in &self.fallback_chain {
            if !chain.contains(&id) {
                chain.push(id);
            }
        }
        if !character_cache_enabled() {
            return text
                .chars()
                .map(|ch| measure_hmtx_char(self, &chain, ch, font_size))
                .sum();
        }
        let chain_key: Arc<[u32]> = chain.clone().into();
        let mut cache = self.hmtx_cache.lock().expect("hmtx cache poisoned");
        text.chars()
            .map(|ch| {
                let key = (Arc::clone(&chain_key), font_size.to_bits(), ch);
                if let Some(&width) = cache.get(&key) {
                    return width;
                }
                let width = measure_hmtx_char(self, &chain, ch, font_size);
                if cache.len() >= HMTX_CACHE_MAX {
                    cache.clear();
                }
                cache.insert(key, width);
                width
            })
            .sum()
    }

    pub(super) fn clear_hmtx_cache(&self) {
        self.hmtx_cache.lock().expect("hmtx cache poisoned").clear();
    }
}

/// hmtx 测量用的 face 缓存：ttf_parser::Face 借用 `bytes`，条目内 Arc 保活字节。
struct HmtxCachedFace {
    _bytes: Arc<Vec<u8>>,
    face: rustybuzz::ttf_parser::Face<'static>,
}

thread_local! {
    static HMTX_FACE_CACHE: std::cell::RefCell<std::collections::HashMap<u64, HmtxCachedFace>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}

const HMTX_FACE_CACHE_MAX: usize = 16;
const HMTX_CACHE_MAX: usize = 4096;

fn character_cache_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZW_HMTX_CHAR_CACHE").as_deref() != Ok("0"))
}

fn measure_hmtx_char(loader: &FontLoader, font_ids: &[u32], ch: char, size: f32) -> f32 {
    HMTX_FACE_CACHE.with(|cache_cell| {
        let mut cache = cache_cell.borrow_mut();
        for &font_id in font_ids {
            let Some(&key) = loader.font_instance_ids.get(&font_id) else {
                continue;
            };
            if cache.contains_key(&key) {
                continue;
            }
            if cache.len() >= HMTX_FACE_CACHE_MAX {
                cache.clear();
            }
            let Some(bytes) = loader.font_data.get(&font_id).cloned() else {
                continue;
            };
            // SAFETY: Face 借用 bytes；同一缓存条目持有 Arc，Face 会先于 bytes drop。
            let slice: &[u8] = &bytes;
            let static_bytes: &'static [u8] = unsafe { std::mem::transmute(slice) };
            let Ok(face) = rustybuzz::ttf_parser::Face::parse(static_bytes, loader.face_index(font_id)) else {
                continue;
            };
            cache.insert(key, HmtxCachedFace { _bytes: bytes, face });
        }

        font_ids
            .iter()
            .filter(|&&font_id| loader.font_allows_code_point(font_id, ch))
            .find_map(|&font_id| {
                let cached = cache.get(loader.font_instance_ids.get(&font_id)?)?;
                let upem = f32::from(cached.face.units_per_em());
                if upem <= 0.0 {
                    return None;
                }
                let glyph_id = cached.face.glyph_index(ch)?;
                let advance = f32::from(cached.face.glyph_hor_advance(glyph_id)?);
                Some(advance * size / upem)
            })
            .unwrap_or(size * 0.5)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const LATO_TTF: &[u8] = include_bytes!("../../../../../tests/wpt-runner/fonts/Lato-Medium.ttf");

    #[test]
    fn character_cache_reuses_widths_across_runs() {
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("register bundled Lato font");
        loader.measure_text_hmtx(&[font_id], "AB", 16.0);
        assert_eq!(loader.hmtx_cache.lock().unwrap().len(), 2);
        loader.measure_text_hmtx(&[font_id], "BA", 16.0);
        assert_eq!(
            loader.hmtx_cache.lock().unwrap().len(),
            2,
            "reordered text should reuse per-character widths"
        );
    }

    #[test]
    fn fallback_and_unicode_range_changes_clear_character_cache() {
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("register bundled Lato font");
        loader.measure_text_hmtx(&[font_id], "A", 16.0);
        loader.set_fallback_chain(vec![font_id]);
        assert!(loader.hmtx_cache.lock().unwrap().is_empty());

        loader.measure_text_hmtx(&[font_id], "A", 16.0);
        loader.register_unicode_ranges(font_id, vec![(0x41, 0x5A)]);
        assert!(loader.hmtx_cache.lock().unwrap().is_empty());
    }

    #[test]
    fn face_cache_uses_font_instance_identity() {
        HMTX_FACE_CACHE.with(|cache| cache.borrow_mut().clear());

        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("register bundled Lato font");
        let duplicate = loader.duplicate();
        loader.measure_text_hmtx(&[font_id], "first run", 16.0);
        duplicate.measure_text_hmtx(&[font_id], "duplicate run", 16.0);
        HMTX_FACE_CACHE.with(|cache| {
            assert_eq!(
                cache.borrow().len(),
                1,
                "duplicate loader should reuse the same font instance"
            );
        });

        let mut independent = FontLoader::new();
        let independent_id = independent.load_font(LATO_TTF).expect("register independent Lato font");
        independent.measure_text_hmtx(&[independent_id], "independent run", 16.0);
        HMTX_FACE_CACHE.with(|cache| {
            assert_eq!(
                cache.borrow().len(),
                2,
                "independent loader should use a distinct font instance"
            );
        });
    }

    #[test]
    fn measurement_matches_shaping_unshaped_advance() {
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("register bundled Lato font");
        for text in ["Hello world", "AVATAR", "The quick brown fox"] {
            let measured = loader.measure_text_hmtx(&[font_id], text, 16.0);
            let shaped = crate::font::TextShaper::new(&loader, Some(crate::primitive::FontId(font_id)))
                .shape_single_line(text, 16.0);
            let unshaped: f32 = shaped.iter().map(|glyph| glyph.unshaped_advance_x).sum();
            let tolerance = text.chars().count() as f32 / 64.0 + 0.01;
            assert!(
                (measured - unshaped).abs() <= tolerance,
                "hmtx({measured:.3}) differs from shaping unshaped({unshaped:.3}): {text:?}"
            );
        }
    }

    #[cfg(feature = "freetype-raster")]
    #[test]
    fn freetype_measurement_matches_shaping_after_no_hinting() {
        let mut loader = FontLoader::new();
        let font_id = loader.load_font(LATO_TTF).expect("register bundled Lato font");
        for text in ["Hello", "WELCOME", "AVATAR", "The quick brown fox"] {
            let measured: f32 = text.chars().map(|ch| loader.measure_advance(font_id, ch, 16.0)).sum();
            let shaped = crate::font::TextShaper::new(&loader, Some(crate::primitive::FontId(font_id)))
                .shape_single_line(text, 16.0);
            let unshaped: f32 = shaped.iter().map(|glyph| glyph.unshaped_advance_x).sum();
            let tolerance = text.chars().count() as f32 / 64.0 + 0.01;
            assert!(
                (measured - unshaped).abs() <= tolerance,
                "measure({measured:.3}) differs from shaping hmtx({unshaped:.3}), tolerance {tolerance:.3}: {text:?}"
            );
        }
    }
}
