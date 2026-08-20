//! WebView Demo — "Hello ZeroWeb" wgpu GPU 渲染
//!
//! M1 里程碑 demo：创建桌面窗口，使用 wgpu GPU 渲染 "Hello ZeroWeb" 文本。
//! 演示 render-foundation GPU 渲染器 + host-runtime 窗口管理的集成。

use std::num::NonZeroU32;
use std::sync::Arc;

use softbuffer::{Context as SoftbufferContext, Surface as SoftbufferSurface};
use zero_host_runtime::event::AppEvent;
use zero_host_runtime::window::{HostRuntime, WindowConfig};
use zero_render_foundation::color::Color;
use zero_render_foundation::config::RenderMode;
use zero_render_foundation::cpu::render_scene_to_framebuffer;
use zero_render_foundation::font::cache::GlyphCache;
use zero_render_foundation::font::loader::FontLoader;
use zero_render_foundation::gpu::renderer::{GlyphDraw, GpuRenderer};
use zero_render_foundation::primitive::FillPrimitive;
use zero_render_foundation::surface::FrameBuffer;

type CpuSurface = SoftbufferSurface<Arc<winit::window::Window>, Arc<winit::window::Window>>;

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
        std::fs::read(path)
            .ok()
            .and_then(|data| font_loader.load_font(&data).ok())
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

fn parse_render_mode_from_args() -> Result<RenderMode, String> {
    let mut args = std::env::args().skip(1);
    let mut cli_mode = None;

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            print_usage();
            std::process::exit(0);
        }

        if let Some(value) = arg.strip_prefix("--renderer=") {
            cli_mode = Some(value.parse()?);
            continue;
        }

        if arg == "--renderer" {
            let value = args
                .next()
                .ok_or_else(|| format!("--renderer requires {}", RenderMode::values()))?;
            cli_mode = Some(value.parse()?);
        }
    }

    Ok(cli_mode.or(RenderMode::from_env()?).unwrap_or_default())
}

fn print_usage() {
    println!("Usage: webview-demo [--renderer {}]", RenderMode::values());
    println!("Environment: {}={}", RenderMode::ENV_VAR, RenderMode::values());
}

fn logical_size_from_window(window: &winit::window::Window) -> ((u32, u32), f32) {
    let physical = window.inner_size();
    let scale = normalized_window_scale(window.scale_factor());
    let logical_width = ((physical.width as f32 / scale).round() as u32).max(1);
    let logical_height = ((physical.height as f32 / scale).round() as u32).max(1);
    ((logical_width, logical_height), scale)
}

fn normalized_window_scale(scale: f64) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale as f32
    } else {
        1.0
    }
}

/// 应用状态 — 在事件循环中持续维护
struct DemoState {
    /// GPU 渲染器（可选 — 如果 GPU 不可用则降级到 CPU 渲染）
    gpu_renderer: Option<GpuRenderer>,
    /// CPU 软件渲染窗口 surface
    cpu_surface: Option<CpuSurface>,
    /// 渲染模式
    render_mode: RenderMode,
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
    /// 窗口逻辑尺寸
    logical_size: (u32, u32),
    /// 窗口物理尺寸
    physical_size: (u32, u32),
    /// 窗口缩放因子
    scale_factor: f32,
}

impl DemoState {
    fn new(render_mode: RenderMode) -> Self {
        let mut font_loader = FontLoader::new();
        let font_id = load_system_font(&mut font_loader);

        if font_id.is_some() {
            println!("已加载系统字体");
        } else {
            println!("未找到系统字体，使用 5x7 点阵后备字体");
        }

        Self {
            gpu_renderer: None,
            cpu_surface: None,
            render_mode,
            font_loader,
            glyph_cache: GlyphCache::new(8192),
            font_id,
            surface_configured: false,
            needs_redraw: true,
            logical_size: (800, 600),
            physical_size: (800, 600),
            scale_factor: 1.0,
        }
    }

    /// 尝试初始化 GPU 渲染器
    fn init_gpu(&mut self, window: &Arc<winit::window::Window>) {
        if matches!(self.render_mode, RenderMode::Cpu) {
            return;
        }

        match GpuRenderer::new_for_window(Arc::clone(window)) {
            Ok(renderer) => {
                println!("wgpu GPU 渲染器初始化成功 (format: {:?})", renderer.surface_format());
                self.gpu_renderer = Some(renderer);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(e) => {
                if matches!(self.render_mode, RenderMode::Gpu) {
                    eprintln!("GPU 渲染器初始化失败: {e}");
                } else {
                    eprintln!("GPU 渲染器初始化失败: {e}，将使用 CPU 后备渲染");
                }
            }
        }
    }

    /// 初始化 CPU 软件渲染 surface
    fn init_cpu_surface(&mut self, window: &Arc<winit::window::Window>) {
        if self.cpu_surface.is_some() {
            return;
        }

        match SoftbufferContext::new(Arc::clone(window))
            .and_then(|context| SoftbufferSurface::new(&context, Arc::clone(window)))
        {
            Ok(surface) => {
                println!("CPU 软件渲染器初始化成功");
                self.cpu_surface = Some(surface);
                self.surface_configured = false;
                self.needs_redraw = true;
            }
            Err(e) => {
                eprintln!("CPU 软件渲染器初始化失败: {e}");
            }
        }
    }

    /// 执行渲染
    fn render(&mut self, width: u32, height: u32) {
        // 取出 gpu_renderer 以避免借用冲突
        let mut gpu = self.gpu_renderer.take();
        if let Some(ref mut renderer) = gpu {
            self.render_gpu(renderer, width, height);
        } else if self.cpu_surface.is_some() {
            self.render_cpu(width, height);
        }
        self.gpu_renderer = gpu;
    }

    fn render_gpu(&mut self, gpu: &mut GpuRenderer, width: u32, height: u32) {
        let (fills, glyphs) = self.build_scene(width, height);
        gpu.render_scene_scaled(
            &fills,
            &self.font_loader,
            &mut self.glyph_cache,
            &glyphs,
            &[],
            self.scale_factor,
        );
    }

    fn render_cpu(&mut self, width: u32, height: u32) {
        let (fills, glyphs) = self.build_scene(width, height);
        let fb = render_scene_to_framebuffer(
            width,
            height,
            self.scale_factor,
            &fills,
            &[],
            &self.font_loader,
            &mut self.glyph_cache,
            &glyphs,
            &[],
            &[],
            &[],
        );
        let Some(surface) = self.cpu_surface.as_mut() else {
            return;
        };

        let sw = match NonZeroU32::new(fb.width) {
            Some(width) => width,
            None => return,
        };
        let sh = match NonZeroU32::new(fb.height) {
            Some(height) => height,
            None => return,
        };

        if let Err(e) = surface.resize(sw, sh) {
            eprintln!("CPU surface resize 失败: {e}");
            return;
        }
        let mut buffer = match surface.buffer_mut() {
            Ok(buffer) => buffer,
            Err(e) => {
                eprintln!("CPU surface buffer 失败: {e}");
                return;
            }
        };
        for (dst, rgba) in buffer.iter_mut().zip(fb.data.as_chunks::<4>().0.iter()) {
            *dst = ((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | rgba[2] as u32;
        }
        if let Err(e) = buffer.present() {
            eprintln!("CPU surface present 失败: {e}");
        }
    }

    fn build_scene(&mut self, width: u32, height: u32) -> (Vec<FillPrimitive>, Vec<GlyphDraw>) {
        let text = "Hello ZeroWeb!";
        let font_size = 32.0f32;
        let text_color = Color::rgb(33, 33, 33); // 深灰色文本

        // 背景填充
        let fills = vec![FillPrimitive {
            rect: zero_render_foundation::geometry::Rect::new(0.0, 0.0, width as f32, height as f32),
            color: Color::rgb(255, 255, 255), // 白色背景
        }];

        // 构建 glyph 数据
        let mut glyphs = Vec::new();

        if let Some(fid) = self.font_id {
            // 计算文本起始位置（居中）
            let mut total_width = 0.0f32;
            for ch in text.chars() {
                let key = zero_render_foundation::font::cache::GlyphKey::new(fid, ch as u32, font_size);
                if let Ok(bitmap) = self
                    .glyph_cache
                    .get_or_insert_with(key, || self.font_loader.rasterize_glyph(fid, ch, font_size))
                {
                    total_width += bitmap.advance;
                }
            }

            let start_x = (width as f32 - total_width) / 2.0;
            let baseline_y = height as f32 / 2.0;
            let mut x = start_x;

            for ch in text.chars() {
                glyphs.push(GlyphDraw {
                    ch,
                    font_glyph_index: None,
                    x,
                    baseline_y,
                    color: text_color,
                    font_id: fid,
                    font_variations: None,
                    font_size,
                    rotation: 0.0,
                });
                let key = zero_render_foundation::font::cache::GlyphKey::new(fid, ch as u32, font_size);
                if let Ok(bitmap) = self
                    .glyph_cache
                    .get_or_insert_with(key, || self.font_loader.rasterize_glyph(fid, ch, font_size))
                {
                    x += bitmap.advance;
                }
            }
        }

        (fills, glyphs)
    }
}

fn main() {
    println!("ZeroWeb WebView Demo (wgpu GPU 渲染)");
    println!("正在初始化...");

    let render_mode = match parse_render_mode_from_args() {
        Ok(mode) => mode,
        Err(err) => {
            eprintln!("{err}");
            print_usage();
            std::process::exit(2);
        }
    };
    println!("渲染模式: {render_mode}");

    // CPU 后备：仍然生成 PPM 文件
    let mut fb = FrameBuffer::new(800, 600);

    let mut font_loader = FontLoader::new();
    let font_id = load_system_font(&mut font_loader);

    if let Some(fid) = font_id {
        let mut glyph_cache = GlyphCache::new(8192);
        let text = "Hello ZeroWeb!";
        let font_size = 32.0f32;
        let mut x = 40.0f32;
        let baseline_y = fb.height as f32 / 2.0;

        fb.clear(255, 255, 255, 255);
        for ch in text.chars() {
            let key = zero_render_foundation::font::cache::GlyphKey::new(fid, ch as u32, font_size);
            if let Ok(bitmap) = glyph_cache.get_or_insert_with(key, || font_loader.rasterize_glyph(fid, ch, font_size))
            {
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
        render_text_fallback(&mut fb, "Hello ZeroWeb!", 40, 300);
    }

    // 保存 PPM
    let ppm = std::fs::File::create("demo_output.ppm");
    if let Ok(mut file) = ppm {
        use std::io::Write;
        let _ = writeln!(file, "P6");
        let _ = writeln!(file, "{} {}", fb.width, fb.height);
        let _ = writeln!(file, "255");
        for chunk in fb.data.as_chunks::<4>().0 {
            let _ = file.write_all(&[chunk[0], chunk[1], chunk[2]]);
        }
        println!("已保存 CPU 渲染帧缓冲到 demo_output.ppm");
    }

    // 启动 GPU 窗口渲染
    let config = WindowConfig::new("ZeroWeb Demo — wgpu GPU").with_size(800, 600);
    let runtime = HostRuntime::new(config);

    // 我们需要在事件循环中访问窗口来创建 GPU 表面
    // HostRuntime 现在传递窗口引用
    let mut state = DemoState::new(render_mode);

    println!("进入事件循环...");
    if let Err(e) = runtime.run_with_window(move |event, window| {
        match event {
            AppEvent::RedrawRequested => {
                if !state.surface_configured {
                    if let Some(ref win) = window
                        && state.gpu_renderer.is_none()
                        && state.cpu_surface.is_none()
                    {
                        let (logical_size, scale_factor) = logical_size_from_window(win);
                        let physical = win.inner_size();
                        state.logical_size = logical_size;
                        state.physical_size = (physical.width, physical.height);
                        state.scale_factor = scale_factor;

                        match state.render_mode {
                            RenderMode::Cpu => state.init_cpu_surface(win),
                            RenderMode::Gpu | RenderMode::Auto => {
                                state.init_gpu(win);
                                if state.gpu_renderer.is_none() && matches!(state.render_mode, RenderMode::Auto) {
                                    state.init_cpu_surface(win);
                                }
                            }
                        }
                    }
                    if let Some(ref mut gpu) = state.gpu_renderer {
                        let (w, h) = state.physical_size;
                        gpu.configure_surface(w, h);
                        state.surface_configured = true;
                    } else if state.cpu_surface.is_some() {
                        state.surface_configured = true;
                    }
                }
                // 每帧都重绘（GPU 渲染需要持续刷新）
                state.render(state.logical_size.0, state.logical_size.1);
                state.needs_redraw = false;
            }
            AppEvent::Resized { width, height } => {
                println!("窗口大小变更: {width}x{height}");
                if width > 0 && height > 0 {
                    state.physical_size = (width, height);
                    if let Some(ref win) = window {
                        let (logical_size, scale_factor) = logical_size_from_window(win);
                        state.logical_size = logical_size;
                        state.scale_factor = scale_factor;
                    } else {
                        state.logical_size = (width, height);
                        state.scale_factor = 1.0;
                    }
                    if let Some(ref mut gpu) = state.gpu_renderer {
                        gpu.configure_surface(width, height);
                    }
                    state.needs_redraw = true;
                }
            }
            AppEvent::ScaleFactorChanged { scale_factor } => {
                println!("窗口缩放因子变更: {scale_factor}");
                if let Some(ref win) = window {
                    let physical = win.inner_size();
                    let (logical_size, normalized_scale) = logical_size_from_window(win);
                    state.physical_size = (physical.width, physical.height);
                    state.logical_size = logical_size;
                    state.scale_factor = normalized_scale;
                    if let Some(ref mut gpu) = state.gpu_renderer {
                        gpu.configure_surface(physical.width, physical.height);
                    }
                } else {
                    state.scale_factor = normalized_window_scale(scale_factor);
                }
                state.needs_redraw = true;
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

        if state.needs_redraw
            && let Some(ref win) = window
        {
            win.request_redraw();
        }
    }) {
        eprintln!("事件循环错误: {e}");
        std::process::exit(1);
    }

    println!("ZeroWeb WebView Demo 已退出");
}
