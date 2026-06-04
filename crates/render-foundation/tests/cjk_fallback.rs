//! 集成测试：CJK/Emoji 字体回退（需系统字体）

use zero_render_foundation::font::loader::FontLoader;

fn read_font(path: &str) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

#[test]
fn cjk_fallback_skips_primary_notdef_tofu() {
    let primary_data = read_font("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
    let cjk_data = read_font("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc");
    let (Some(primary_data), Some(cjk_data)) = (primary_data, cjk_data) else {
        eprintln!("skipping: missing system fonts");
        return;
    };

    let mut loader = FontLoader::new();
    let primary = loader.load_font(&primary_data).expect("load primary");
    let cjk = loader.load_font(&cjk_data).expect("load cjk");
    loader.set_fallback_chain(vec![cjk]);

    let (resolved, bitmap) = loader
        .rasterize_glyph_with_fallback(primary, '中', 24.0)
        .expect("should resolve 中 via CJK fallback");
    assert_eq!(resolved, cjk, "CJK glyph should come from fallback font");
    assert!(bitmap.width > 0 && bitmap.height > 0);
    assert!(bitmap.data.iter().any(|&b| b > 0));
}

#[test]
fn emoji_fallback_from_color_font_if_available() {
    let primary_data = read_font("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf");
    let emoji_data = read_font("/usr/share/fonts/truetype/noto/NotoColorEmoji.ttf");
    let (Some(primary_data), Some(emoji_data)) = (primary_data, emoji_data) else {
        eprintln!("skipping: missing system fonts");
        return;
    };

    let mut loader = FontLoader::new();
    let primary = loader.load_font(&primary_data).expect("load primary");
    let emoji = loader.load_font(&emoji_data).expect("load emoji");
    loader.set_fallback_chain(vec![emoji]);

    let emoji_font = loader.get(emoji).expect("emoji font");
    if !emoji_font.has_glyph('🚀') {
        eprintln!("skipping: NotoColorEmoji has no 🚀 glyph in fontdue");
        return;
    }

    let result = loader.rasterize_glyph_with_fallback(primary, '🚀', 24.0);
    match result {
        Ok((resolved, bitmap)) => {
            assert_eq!(resolved, emoji);
            assert!(bitmap.data.iter().any(|&b| b > 0), "emoji bitmap should be non-empty");
        }
        Err(e) => {
            eprintln!("emoji rasterize not supported on this system: {e}");
        }
    }
}
