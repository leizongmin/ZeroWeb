//! 诊断 example：直接用 fontdue 光栅化单字符 → PGM，供与 chromium 同字号同字符对比，
//! 量化纯 AA + glyph 轮廓差异（DC-14 字体攻坚 AA 基准）。不经 HTML/CSS/布局。
use zero_render_foundation::font::FontLoader;
use std::io::Write;

fn main() {
    let mut loader = FontLoader::new();
    let paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];
    let font_id = paths.iter()
        .find_map(|p| std::fs::read(p).ok().and_then(|d| loader.load_font(&d).ok()))
        .expect("DejaVuSans.ttf not found on system");

    for ch in ['W', 'a', 'i', 'M', 'g'] {
        let bm = loader.rasterize_glyph(font_id, ch, 48.0).expect("rasterize");
        let path = format!("/tmp/fontdue_{ch}.pgm");
        let mut f = std::fs::File::create(&path).unwrap();
        write!(f, "P5\n{} {}\n255\n", bm.width, bm.height).unwrap();
        // 黑字白底：alpha → 255-alpha
        let pixels: Vec<u8> = bm.data.iter().map(|&a| 255u8.saturating_sub(a)).collect();
        f.write_all(&pixels).unwrap();
        eprintln!("{ch}: {}x{} advance={:.1} -> {path}", bm.width, bm.height, bm.advance);
    }
}
