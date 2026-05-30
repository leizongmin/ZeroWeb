//! WebView Demo — "Hello ZeroBrowser" wgpu GPU 渲染
//!
//! M1 里程碑 demo：创建桌面窗口，使用 wgpu GPU 渲染 "Hello ZeroBrowser" 文本。
//! 演示 render-foundation GPU 渲染器 + host-runtime 窗口管理的集成。

use std::sync::Arc;
use zero_host_runtime::event::AppEvent;
use zero_host_runtime::window::{HostRuntime, WindowConfig};
use zero_render_foundation::color::Color;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::{GpuRenderer, GlyphDraw};
use zero_render_foundation::primitive::FillPrimitive;
use zero_render_foundation::surface::FrameBuffer;

/// 尝试加载系统字体，返回字体 ID
fn load_system_font(font_loader: &mut FontLoader) -> Option<u32> {
    let font_paths = [
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        "/usr/share/fonts/TTF/DejaVuSans.ttf",
        "/System/Library/Fonts/Helvetica.ttc",
        "C:\\Windows\\Fonts\\arial.ttf",
    ];

    font_paths.iter().find_map(|path| {
        std::fs::read(path).ok().and_then(|data| font_loader.load_font(&data).ok())
    })
}

/// 使用 5x7 点阵渲染文本到 CPU 帧缓冲（后备方案）
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

/// 应用状态 — 在事件循环中持续维护
struct DemoState {
    /// GPU 渲染器（可选 — 如果 GPU 不可用则降级到 CPU 渲染）
    gpu_renderer: Option<GpuRenderer>,
    /// 字体加载器
    font_loader: FontLoader,
    /// Glyph 缓存
    glyph_cache: GlyphCache,
    /// 已加载的系统字体 ID
    font_id: Option<u32>,
    /// 是否已初始化 GPU 表面
    surface_configured: bool,
    /// 是否需要重绘
    needs_redraw: bool,
}

impl DemoState {
    fn new() -> Self {
        let mut font_loader = FontLoader::new();
        let font_id = load_system_font(&mut font_loader);

        if font_id.is_some() {
            println!("已加载系统字体");
        } else {
            println!("未找到系统字体，使用 5x7 点阵后备字体");
        }

        Self {
            gpu_renderer: None,
            font_loader,
            glyph_cache: GlyphCache::new(8192),
            font_id,
            surface_configured: false,
            needs_redraw: true,
        }
    }

    /// 尝试初始化 GPU 渲染器
    fn init_gpu(&mut self, window: &Arc<winit::window::Window>) {
        match GpuRenderer::new_for_window(Arc::clone(window)) {
            Ok(renderer) => {
                println!(
                    "wgpu GPU 渲染器初始化成功 (format: {:?})",
                    renderer.surface_format()
                );
                self.gpu_renderer = Some(renderer);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(e) => {
                eprintln!("GPU 渲染器初始化失败: {e}，将使用 CPU 后备渲染");
            }
        }
    }

    /// 执行渲染
    fn render(&mut self, width: u32, height: u32) {
        // 取出 gpu_renderer 以避免借用冲突
        let mut gpu = self.gpu_renderer.take();
        if let Some(ref mut renderer) = gpu {
            self.render_gpu(renderer, width, height);
        }
        self.gpu_renderer = gpu;
    }

    fn render_gpu(&mut self, gpu: &mut GpuRenderer, width: u32, height: u32) {
        let text = "Hello ZeroBrowser!";
        let font_size = 32.0f32;
        let text_color = Color::rgb(33, 33, 33); // 深灰色文本

        // 背景填充
        let fills = vec![FillPrimitive {
            rect: zero_render_foundation::geometry::Rect::new(
                0.0,
                0.0,
                width as f32,
                height as f32,
            ),
            color: Color::rgb(255, 255, 255), // 白色背景
        }];

        // 构建 glyph 数据
        let mut glyphs = Vec::new();

        if let Some(fid) = self.font_id {
            // 计算文本起始位置（居中）
            let mut total_width = 0.0f32;
            for ch in text.chars() {
                let key = zero_render_foundation::font::cache::GlyphKey::new(fid, ch as u32, font_size);
                if let Ok(bitmap) = self.glyph_cache.get_or_insert_with(key, || {
                    self.font_loader.rasterize_glyph(fid, ch, font_size)
                }) {
                    total_width += bitmap.advance;
                }
            }

            let start_x = (width as f32 - total_width) / 2.0;
            let baseline_y = height as f32 / 2.0;
            let mut x = start_x;

            for ch in text.chars() {
                glyphs.push(GlyphDraw {
                    ch,
                    x,
                    baseline_y,
                    color: text_color,
                    font_id: fid,
                    font_size,
                });
                let key = zero_render_foundation::font::cache::GlyphKey::new(fid, ch as u32, font_size);
                if let Ok(bitmap) = self.glyph_cache.get_or_insert_with(key, || {
                    self.font_loader.rasterize_glyph(fid, ch, font_size)
                }) {
                    x += bitmap.advance;
                }
            }
        }

        gpu.render_scene(&fills, &self.font_loader, &mut self.glyph_cache, &glyphs);
    }
}

fn main() {
    println!("ZeroBrowser WebView Demo (wgpu GPU 渲染)");
    println!("正在初始化...");

    // CPU 后备：仍然生成 PPM 文件
    let mut fb = FrameBuffer::new(800, 600);

    let mut font_loader = FontLoader::new();
    let font_id = load_system_font(&mut font_loader);

    if let Some(fid) = font_id {
        let mut glyph_cache = GlyphCache::new(8192);
        let text = "Hello ZeroBrowser!";
        let font_size = 32.0f32;
        let mut x = 40.0f32;
        let baseline_y = fb.height as f32 / 2.0;

        fb.clear(255, 255, 255, 255);
        for ch in text.chars() {
            let key = zero_render_foundation::font::cache::GlyphKey::new(fid, ch as u32, font_size);
            if let Ok(bitmap) = glyph_cache.get_or_insert_with(key, || {
                font_loader.rasterize_glyph(fid, ch, font_size)
            }) {
                // Blit glyph to framebuffer
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
                x += bitmap.advance;
            }
        }
    } else {
        fb.clear(255, 255, 255, 255);
        render_text_fallback(&mut fb, "Hello ZeroBrowser!", 40, 300);
    }

    // 保存 PPM
    let ppm = std::fs::File::create("demo_output.ppm");
    if let Ok(mut file) = ppm {
        use std::io::Write;
        let _ = writeln!(file, "P6");
        let _ = writeln!(file, "{} {}", fb.width, fb.height);
        let _ = writeln!(file, "255");
        for chunk in fb.data.chunks_exact(4) {
            let _ = file.write_all(&[chunk[0], chunk[1], chunk[2]]);
        }
        println!("已保存 CPU 渲染帧缓冲到 demo_output.ppm");
    }

    // 启动 GPU 窗口渲染
    let config = WindowConfig::new("ZeroBrowser Demo — wgpu GPU").with_size(800, 600);
    let runtime = HostRuntime::new(config);

    // 我们需要在事件循环中访问窗口来创建 GPU 表面
    // HostRuntime 现在传递窗口引用
    let mut state = DemoState::new();

    println!("进入事件循环...");
    if let Err(e) = runtime.run_with_window(move |event, window| {
        match event {
            AppEvent::RedrawRequested => {
                if !state.surface_configured {
                    if let Some(ref win) = window
                        && state.gpu_renderer.is_none()
                    {
                        state.init_gpu(win);
                    }
                    if let Some(ref mut gpu) = state.gpu_renderer {
                        gpu.configure_surface(800, 600);
                        state.surface_configured = true;
                    }
                }
                // 每帧都重绘（GPU 渲染需要持续刷新）
                state.render(800, 600);
                state.needs_redraw = false;
            }
            AppEvent::Resized { width, height } => {
                println!("窗口大小变更: {width}x{height}");
                if width > 0 && height > 0 {
                    if let Some(ref mut gpu) = state.gpu_renderer {
                        gpu.configure_surface(width, height);
                    }
                    state.needs_redraw = true;
                }
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
            _ => {}
        }
    }) {
        eprintln!("事件循环错误: {e}");
        std::process::exit(1);
    }

    println!("ZeroBrowser WebView Demo 已退出");
}
