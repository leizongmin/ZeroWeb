//! WebView Demo — "Hello ZeroBrowser"
//!
//! M1 里程碑 demo：创建桌面窗口，使用 CPU 渲染 "Hello ZeroBrowser" 文本。
//! 演示 render-foundation + host-runtime 的集成。

use zero_host_runtime::event::AppEvent;
use zero_host_runtime::window::{HostRuntime, WindowConfig};
use zero_render_foundation::font::cache::{GlyphCache, GlyphKey};
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::surface::FrameBuffer;

/// 将 glyph 位图 blit 到帧缓冲
fn blit_glyph(
    fb: &mut FrameBuffer,
    bitmap: &zero_render_foundation::font::GlyphBitmap,
    x: f32,
    baseline_y: f32,
) {
    let start_x = (x as i32 + bitmap.x_offset as i32).max(0) as u32;
    let start_y = (baseline_y as i32 + bitmap.y_offset as i32).max(0) as u32;

    for row in 0..bitmap.height {
        for col in 0..bitmap.width {
            let px = start_x + col as u32;
            let py = start_y + row as u32;
            if px >= fb.width || py >= fb.height {
                continue;
            }

            let alpha = bitmap.data[(row as usize * bitmap.width as usize) + col as usize];
            if alpha == 0 {
                continue;
            }

            let a = alpha as f32 / 255.0;
            let existing = fb.get_pixel(px, py);
            let r = (existing[0] as f32 * (1.0 - a)) as u8;
            let g = (existing[1] as f32 * (1.0 - a)) as u8;
            let b = (existing[2] as f32 * (1.0 - a)) as u8;
            fb.set_pixel(px, py, [r, g, b, 255]);
        }
    }
}

/// 使用 5x7 点阵渲染文本（无字体时的后备方案）
fn render_text_fallback(fb: &mut FrameBuffer, text: &str, start_x: u32, center_y: usize) {
    let scale = 4u32;
    let y = center_y - (7 * scale as usize) / 2;

    for (i, ch) in text.chars().enumerate() {
        let pattern = get_font5x7(ch);
        let ox = start_x + (i as u32 * 6 * scale);

        for (row, &byte) in pattern.iter().enumerate() {
            for col in 0u8..5 {
                if byte & (1 << (4 - col)) != 0 {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let px = ox + col as u32 * scale + dx;
                            let py = y as u32 + row as u32 * scale + dy;
                            if px < fb.width && py < fb.height {
                                fb.set_pixel(px, py, [33, 33, 33, 255]);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 5x7 点阵字体
fn get_font5x7(ch: char) -> [u8; 7] {
    match ch {
        'A' => [0x04, 0x0A, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1C, 0x12, 0x11, 0x11, 0x11, 0x12, 0x1C],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0E, 0x11, 0x10, 0x0E, 0x01, 0x11, 0x0E],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    }
}

/// 尝试加载系统字体并渲染演示文本到帧缓冲
fn render_demo(fb: &mut FrameBuffer) {
    fb.clear(255, 255, 255, 255);

    let text = "Hello ZeroBrowser!";
    let font_size = 32.0f32;

    let mut font_loader = FontLoader::new();
    let mut glyph_cache = GlyphCache::new(8192);

    // 尝试加载系统字体
    let font_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];

    let font_id = font_paths.iter().find_map(|path| {
        std::fs::read(path).ok().and_then(|data| font_loader.load_font(&data).ok())
    });

    if let Some(fid) = font_id {
        let mut x = 40.0f32;
        let baseline_y = fb.height as f32 / 2.0;

        for ch in text.chars() {
            let key = GlyphKey::new(fid, ch as u32, font_size);
            if let Ok(bitmap) = glyph_cache.get_or_insert_with(key, || {
                font_loader.rasterize_glyph(fid, ch, font_size)
            }) {
                blit_glyph(fb, bitmap, x, baseline_y);
                x += bitmap.advance;
            }
        }
    } else {
        render_text_fallback(fb, text, 40, fb.height as usize / 2);
    }
}

fn main() {
    println!("ZeroBrowser WebView Demo");
    println!("正在初始化...");

    // 先演示 CPU 渲染到帧缓冲
    let mut fb = FrameBuffer::new(800, 600);
    render_demo(&mut fb);
    println!("已渲染演示文本到 800x600 帧缓冲");

    // 保存为 PPM 文件（简单的无压缩图片格式）
    let ppm = std::fs::File::create("demo_output.ppm");
    if let Ok(mut file) = ppm {
        use std::io::Write;
        let _ = writeln!(file, "P6");
        let _ = writeln!(file, "{} {}", fb.width, fb.height);
        let _ = writeln!(file, "255");
        for chunk in fb.data.chunks_exact(4) {
            let _ = file.write_all(&[chunk[0], chunk[1], chunk[2]]);
        }
        println!("已保存帧缓冲到 demo_output.ppm");
    }

    // 启动窗口事件循环
    let config = WindowConfig::new("ZeroBrowser Demo").with_size(800, 600);
    let runtime = HostRuntime::new(config);

    println!("进入事件循环...");
    if let Err(e) = runtime.run(|event| {
        match event {
            AppEvent::Resized { width, height } => {
                println!("窗口大小变更: {}x{}", width, height);
            }
            AppEvent::CloseRequested => {
                println!("窗口关闭请求");
            }
            AppEvent::Focused => {
                println!("窗口获得焦点");
            }
            AppEvent::Unfocused => {
                println!("窗口失去焦点");
            }
            AppEvent::KeyboardInput { key, pressed } => {
                if pressed {
                    println!("按键: {}", key);
                }
            }
            AppEvent::RedrawRequested => {}
        }
    }) {
        eprintln!("事件循环错误: {}", e);
        std::process::exit(1);
    }

    println!("ZeroBrowser WebView Demo 已退出");
}
