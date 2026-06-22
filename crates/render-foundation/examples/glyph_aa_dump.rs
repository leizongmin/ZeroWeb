//! 诊断 example：直接用 fontdue 光栅化单字符 → PGM，供与 chromium 同字号同字符对比，
//! 量化纯 AA + glyph 轮廓差异（DC-14 字体攻坚 AA 基准）。不经 HTML/CSS/布局。
use std::io::Write;
use zero_render_foundation::font::FontLoader;

fn main() {
    let mut loader = FontLoader::new();
    let paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/usr/share/fonts/dejavu/DejaVuSans.ttf",
    ];
    let font_id = paths
        .iter()
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

    // R387 诊断：Ahem（reftest 字体）多字号光栅化度量——R174 仅测 DejaVu 48px，
    // 未覆盖 100px 大字号 Ahem（ifc-008 等大字体簇）。Ahem 每 glyph 应 ≈1em×1em（advance≈font_size）。
    let ahem_paths = [
        "tests/wpt-runner/fonts/Ahem.ttf",
        "tests/wpt-runner/wpt-data/fonts/Ahem.ttf",
    ];
    let ahem_id = ahem_paths
        .iter()
        .find_map(|p| std::fs::read(p).ok().and_then(|d| loader.load_font(&d).ok()));
    if let Some(aid) = ahem_id {
        eprintln!("--- Ahem (expect width≈height≈advance≈font_size, 1em square) ---");
        for &fs in &[16.0f32, 48.0, 100.0] {
            for &ch in &['X', 'x', ' '] {
                match loader.rasterize_glyph(aid, ch, fs) {
                    Ok(bm) => eprintln!(
                        "Ahem fs={fs} '{ch}': width={} height={} advance={:.1} (w/fs={:.3} adv/fs={:.3})",
                        bm.width,
                        bm.height,
                        bm.advance,
                        bm.width as f32 / fs,
                        bm.advance / fs
                    ),
                    Err(e) => eprintln!("Ahem fs={fs} '{ch}' ERR {e}"),
                }
            }
        }
    } else {
        eprintln!("Ahem.ttf not found");
    }
}
