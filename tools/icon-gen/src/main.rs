//! ZeroBrowser 图标生成器：从源 SVG 生成各平台所需图标资产。
//!
//! 用法：
//!   cargo run -p zero-icon-gen -- [--svg <path>] [--out <dir>]
//!
//! 默认：
//!   源 SVG:  apps/browser/assets/app-icon.svg
//!   输出目录: apps/browser/assets/icons-gen/
//!
//! 产物：
//!   - icon-16.png ... icon-512.png   (Linux .desktop + 运行时窗口图标)
//!   - zero-browser.ico               (Windows，多尺寸)
//!   - iconset/icon_*.png             (macOS，配合 iconutil 生成 .icns)
//!   - window-icon.rgba               (运行时 winit 窗口图标，256px RGBA)
//!
//! macOS .icns 需在 macOS 上额外执行（见 package-macos.sh）：
//!   iconutil -c icns <out>/iconset -o <out>/zero-browser.icns

use std::fs;
use std::path::PathBuf;

use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;

const DEFAULT_SVG: &str = "apps/browser/assets/app-icon.svg";
const DEFAULT_OUT: &str = "apps/browser/assets/icons-gen";

/// ICO 内嵌尺寸（Windows 资源管理器 / 任务栏常用）。
const ICO_SIZES: [u32; 5] = [16, 32, 48, 64, 256];

/// Linux hicolor + 运行时窗口图标所需尺寸。
const PNG_SIZES: [u32; 6] = [16, 32, 48, 128, 256, 512];

/// macOS .iconset 尺寸（含 @2x，对应 16/32/128/256/512 的 1x 与 2x）。
const MAC_ICONSET: &[(u32, &str)] = &[
    (16, "icon_16x16.png"),
    (32, "icon_16x16@2x.png"),
    (32, "icon_32x32.png"),
    (64, "icon_32x32@2x.png"),
    (128, "icon_128x128.png"),
    (256, "icon_128x128@2x.png"),
    (256, "icon_256x256.png"),
    (512, "icon_256x256@2x.png"),
    (512, "icon_512x512.png"),
];

fn main() {
    let mut svg_path = PathBuf::from(DEFAULT_SVG);
    let mut out_dir = PathBuf::from(DEFAULT_OUT);

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--svg" => {
                svg_path = PathBuf::from(args.next().expect("--svg 缺少参数"));
            }
            "--out" => {
                out_dir = PathBuf::from(args.next().expect("--out 缺少参数"));
            }
            "-h" | "--help" => {
                eprintln!(
                    "用法: zero-icon-gen [--svg <path>] [--out <dir>]\n  默认 svg={DEFAULT_SVG}\n  默认 out={DEFAULT_OUT}"
                );
                return;
            }
            other => {
                eprintln!("未知参数: {other}");
                std::process::exit(1);
            }
        }
    }

    let svg_bytes = fs::read(&svg_path).unwrap_or_else(|e| {
        eprintln!("读取 SVG 失败 {}: {e}", svg_path.display());
        std::process::exit(1);
    });

    let tree = usvg::Tree::from_data(&svg_bytes, &usvg::Options::default()).unwrap_or_else(|e| {
        eprintln!("解析 SVG 失败: {e}");
        std::process::exit(1);
    });
    let view = tree.size().width().max(tree.size().height());
    if view <= 0.0 {
        eprintln!("SVG 尺寸无效");
        std::process::exit(1);
    }

    fs::create_dir_all(&out_dir).unwrap_or_else(|e| {
        eprintln!("创建输出目录失败: {e}");
        std::process::exit(1);
    });
    let iconset_dir = out_dir.join("iconset");
    fs::create_dir_all(&iconset_dir).unwrap_or_else(|e| {
        eprintln!("创建 iconset 目录失败: {e}");
        std::process::exit(1);
    });

    // 预渲染所有需要的尺寸为 RGBA 像素，避免重复光栅化。
    let all_sizes: Vec<u32> = {
        let s: std::collections::BTreeSet<u32> = ICO_SIZES
            .iter()
            .chain(PNG_SIZES.iter())
            .chain(MAC_ICONSET.iter().map(|(sz, _)| sz))
            .copied()
            .collect();
        s.into_iter().collect()
    };
    let mut rgba_cache: std::collections::HashMap<u32, (u32, u32, Vec<u8>)> = std::collections::HashMap::new();
    for size in &all_sizes {
        rgba_cache.insert(*size, rasterize(&tree, view, *size));
    }

    // 1. PNG 各尺寸
    for size in PNG_SIZES {
        let (w, h, rgba) = rgba_cache.get(&size).expect("cached");
        let png_bytes = encode_png(*w, *h, rgba);
        let path = out_dir.join(format!("icon-{size}.png"));
        fs::write(&path, &png_bytes).unwrap_or_else(|e| {
            eprintln!("写入 {} 失败: {e}", path.display());
            std::process::exit(1);
        });
    }

    // 2. Windows .ico
    let mut ico_entries: Vec<ico::IconImage> = Vec::new();
    for size in ICO_SIZES {
        let (w, h, rgba) = rgba_cache.get(&size).expect("cached");
        let png_bytes = encode_png(*w, *h, rgba);
        let entry = ico::IconImage::read_png(std::io::Cursor::new(&png_bytes)).unwrap_or_else(|e| {
            eprintln!("组装 ICO 尺寸 {size} 失败: {e}");
            std::process::exit(1);
        });
        ico_entries.push(entry);
    }
    let ico_dir = ico::IconDir::new(ico::ResourceType::Icon);
    let mut ico_dir = ico_dir;
    for entry in ico_entries {
        ico_dir.add_entry(ico::IconDirEntry::encode(&entry).unwrap_or_else(|e| {
            eprintln!("编码 ICO entry 失败: {e}");
            std::process::exit(1);
        }));
    }
    let ico_path = out_dir.join("zero-browser.ico");
    {
        let mut f = fs::File::create(&ico_path).unwrap_or_else(|e| {
            eprintln!("创建 {} 失败: {e}", ico_path.display());
            std::process::exit(1);
        });
        ico_dir.write(&mut f).unwrap_or_else(|e| {
            eprintln!("写入 ICO 失败: {e}");
            std::process::exit(1);
        });
    }

    // 3. macOS iconset（PNG 文件，供 iconutil -c icns 使用）
    for (size, name) in MAC_ICONSET {
        let (w, h, rgba) = rgba_cache.get(size).expect("cached");
        let png_bytes = encode_png(*w, *h, rgba);
        let path = iconset_dir.join(name);
        fs::write(&path, &png_bytes).unwrap_or_else(|e| {
            eprintln!("写入 {} 失败: {e}", path.display());
            std::process::exit(1);
        });
    }

    // 4. 运行时窗口图标（256px RGBA，winit::window::Icon 使用）
    let (_w, _h, rgba) = rgba_cache.get(&256).expect("cached");
    let win_icon_path = out_dir.join("window-icon-256.rgba");
    fs::write(&win_icon_path, rgba).unwrap_or_else(|e| {
        eprintln!("写入 {} 失败: {e}", win_icon_path.display());
        std::process::exit(1);
    });

    println!("图标资产已生成至 {}", out_dir.display());
    println!("  - PNG:   {} 个尺寸", PNG_SIZES.len());
    println!("  - ICO:   {} (含 {} 尺寸)", ico_path.display(), ICO_SIZES.len());
    println!(
        "  - iconset: {} (在 macOS 上用 iconutil 生成 .icns)",
        iconset_dir.display()
    );
    println!("  - 窗口图标: {}", win_icon_path.display());
}

/// 超采样倍数：在 4× 分辨率光栅化后 box-filter 降采样到目标尺寸，
/// 显著减少小尺寸（16/32px）下的锯齿与半透明边缘损失。
const SUPERSAMPLE: u32 = 4;

fn rasterize(tree: &usvg::Tree, view: f32, size: u32) -> (u32, u32, Vec<u8>) {
    let side = size.max(1);
    if side <= 64 {
        // 小尺寸：超采样降采样，提升清晰度。
        let hi = side * SUPERSAMPLE;
        let mut hi_pixmap = Pixmap::new(hi, hi).expect("hi pixmap");
        let hi_scale = hi as f32 / view;
        resvg::render(tree, Transform::from_scale(hi_scale, hi_scale), &mut hi_pixmap.as_mut());
        let rgba = downsample_box(&hi_pixmap, side);
        (side, side, rgba)
    } else {
        // 大尺寸：直接光栅化已足够清晰。
        let mut pixmap = Pixmap::new(side, side).expect("pixmap");
        let scale = side as f32 / view;
        resvg::render(tree, Transform::from_scale(scale, scale), &mut pixmap.as_mut());
        let rgba: Vec<u8> = pixmap
            .pixels()
            .iter()
            .flat_map(|p| {
                let c = p.demultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect();
        (side, side, rgba)
    }
}

/// Box-filter 降采样：把 hi×hi 的预乘 RGBA 位图按 SS×SS 的核平均到 side×side。
fn downsample_box(hi: &Pixmap, side: u32) -> Vec<u8> {
    let ss = SUPERSAMPLE as usize;
    let hi_side = hi.width() as usize;
    let hi_pixels = hi.pixels();
    let mut out = Vec::with_capacity((side as usize * side as usize) * 4);
    for oy in 0..side as usize {
        for ox in 0..side as usize {
            let mut r_acc = 0u32;
            let mut g_acc = 0u32;
            let mut b_acc = 0u32;
            let mut a_acc = 0u32;
            for dy in 0..ss {
                for dx in 0..ss {
                    let hx = ox * ss + dx;
                    let hy = oy * ss + dy;
                    let idx = hy * hi_side + hx;
                    let p = hi_pixels[idx].demultiply();
                    r_acc += p.red() as u32;
                    g_acc += p.green() as u32;
                    b_acc += p.blue() as u32;
                    a_acc += p.alpha() as u32;
                }
            }
            let n = (ss * ss) as u32;
            out.push((r_acc / n) as u8);
            out.push((g_acc / n) as u8);
            out.push((b_acc / n) as u8);
            out.push((a_acc / n) as u8);
        }
    }
    out
}

fn encode_png(w: u32, h: u32, rgba: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("png header");
        writer.write_image_data(rgba).expect("png write");
    }
    out
}
