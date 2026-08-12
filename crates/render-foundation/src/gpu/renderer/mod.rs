//! GPU 渲染器 — 组合 wgpu 上下文、glyph atlas 和渲染管线
//!
//! 提供两种模式：
//! - **窗口模式**: 直接渲染到 wgpu Surface（GPU 合成到屏幕）
//! - **无头模式**: 渲染到纹理后回读像素（CPU 后备 / 测试用）

use crate::color::Color;
use crate::cpu::glyph_top_left;
use crate::font::cache::GlyphCache;
use crate::font::loader::FontLoader;
use crate::geometry::Rect;
use crate::gpu::atlas::{GlyphAtlas, GlyphAtlasKey};
use crate::gpu::mesh::{color_to_f32, push_fill_quad, push_path_fill_mesh, push_path_stroke_mesh, push_stroke_mesh};
use crate::gpu::pipeline::{
    FILL_FLOATS_PER_VERTEX, GRADIENT_FLOATS_PER_VERTEX, IMAGE_FLOATS_PER_VERTEX, ROUNDED_RECT_FLOATS_PER_VERTEX,
    create_atlas_bind_group_layout, create_blur_pipeline, create_color_filter_pipeline, create_gradient_pipeline,
    create_image_pipeline, create_render_pipeline, create_rounded_rect_pipeline, create_texture_bind_group_layout,
    create_transform_pipeline, create_transform_uniform_bgl, create_uniform_bind_group_layout,
};
use crate::image_cache::ImageCache;
use crate::primitive::{FillPrimitive, FilterKind, GradientKind, RenderPrimitives, RoundedRectPrimitive};

mod filters;

/// GPU 渲染器创建互斥锁 — 防止并发 wgpu 实例初始化导致 SIGSEGV
///
/// wgpu 驱动在多个线程同时创建 Instance/Adapter/Device 时可能触发段错误，
/// 通过全局互斥锁序列化创建过程来解决。
static GPU_CREATE_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// 渲染场景中的 glyph 文本参数
#[derive(Debug, Clone)]
pub struct GlyphDraw {
    /// 源字符。
    pub ch: char,
    /// 当前字体内部的 OpenType glyph index；`None` 表示按 `ch` 查字符。
    pub font_glyph_index: Option<u16>,
    /// 表面上的 X 位置
    pub x: f32,
    /// 基线 Y 位置
    pub baseline_y: f32,
    /// 前景颜色
    pub color: Color,
    /// 字体 ID
    pub font_id: u32,
    /// 字体大小（像素）
    pub font_size: f32,
    /// 字形旋转弧度（0.0 = 不旋转；FRAC_PI_2 = 顺时针 90°，用于 vertical-rl/lr Latin 文本）。
    /// DC-9 GPU parity：CPU renderer `blit_glyph_bitmap` 已支持 is_rotated_90，GPU 此前缺（rotation
    /// 字段不存在），致 GPU-mode vertical 文本不旋转。R1595 接入。
    pub rotation: f32,
}

impl GlyphDraw {
    fn rasterize(&self, font_loader: &FontLoader, size: f32) -> Option<(u32, crate::font::GlyphBitmap)> {
        match self.font_glyph_index {
            Some(glyph_index) => font_loader
                .rasterize_glyph_index(self.font_id, glyph_index, size)
                .ok()
                .map(|bitmap| (self.font_id, bitmap)),
            None => font_loader
                .rasterize_glyph_with_fallback(self.font_id, self.ch, size)
                .ok(),
        }
    }

    fn cache_key(&self, resolved_font_id: u32, size: f32) -> crate::font::cache::GlyphKey {
        match self.font_glyph_index {
            Some(glyph_index) => crate::font::cache::GlyphKey::new_indexed(resolved_font_id, glyph_index, size),
            None => crate::font::cache::GlyphKey::new(resolved_font_id, self.ch as u32, size),
        }
    }

    fn atlas_key(&self, resolved_font_id: u32, size: f32) -> GlyphAtlasKey {
        match self.font_glyph_index {
            Some(glyph_index) => GlyphAtlasKey::new_indexed(resolved_font_id, glyph_index, size),
            None => GlyphAtlasKey::new(resolved_font_id, self.ch as u32, size),
        }
    }
}

use std::sync::Arc;
use wgpu::util::DeviceExt;

/// GPU 渲染器 — 管理 wgpu 设备、atlas 和渲染管线
pub struct GpuRenderer {
    /// wgpu 设备
    device: Arc<wgpu::Device>,
    /// wgpu 队列
    queue: Arc<wgpu::Queue>,
    /// Fill+Glyph 渲染管线
    pipeline: wgpu::RenderPipeline,
    /// RoundedRect 渲染管线
    rounded_rect_pipeline: wgpu::RenderPipeline,
    /// Gradient 渲染管线
    gradient_pipeline: wgpu::RenderPipeline,
    /// Image 渲染管线
    image_pipeline: wgpu::RenderPipeline,
    /// Blur 后处理管线（DC-9 filter:blur 2-pass）
    blur_pipeline: wgpu::RenderPipeline,
    /// 单通道颜色滤镜后处理管线（DC-9：opacity/brightness/contrast）
    #[allow(dead_code)]
    color_filter_pipeline: wgpu::RenderPipeline,
    /// Transform 后处理管线（DC-9：2D 仿射变换逆矩阵重采样）
    #[allow(dead_code)]
    transform_pipeline: wgpu::RenderPipeline,
    /// Transform uniform 绑定组布局（group 0，64 字节）
    transform_uniform_bgl: wgpu::BindGroupLayout,
    /// Uniform 绑定组布局
    uniform_bgl: wgpu::BindGroupLayout,
    /// Atlas 绑定组布局（保留用于 atlas 重建时重新创建绑定组）
    #[allow(dead_code)]
    atlas_bgl: wgpu::BindGroupLayout,
    /// Gradient 纹理绑定组布局
    gradient_bgl: wgpu::BindGroupLayout,
    /// Image 纹理绑定组布局
    image_bgl: wgpu::BindGroupLayout,
    /// Image 纹理采样器（所有 image primitive 共用）。
    image_sampler: wgpu::Sampler,
    /// 已上传图片纹理缓存，避免同一内容每帧重复创建 GPU 资源。
    image_texture_cache: std::collections::HashMap<GpuImageCacheKey, CachedImageResource>,
    /// 图片纹理缓存显存预算（R3254-M3：超过时按 last_used 逐出；条目数上限 8192 仍兜底）。
    image_texture_budget_bytes: usize,
    /// 图片纹理缓存访问代数（R3254-M3：命中刷新 last_used 用）。
    image_texture_tick: u64,
    /// Blur 源纹理绑定组布局
    #[allow(dead_code)]
    blur_bgl: wgpu::BindGroupLayout,
    /// Glyph Atlas（CPU 侧放置追踪）
    atlas: GlyphAtlas,
    /// Atlas 纹理
    atlas_texture: wgpu::Texture,
    /// Atlas 绑定组
    atlas_bind_group: wgpu::BindGroup,
    /// 持久化 Uniform 缓冲区（避免每帧重新分配）
    uniform_buffer: Option<wgpu::Buffer>,
    /// 持久化 Uniform 绑定组
    uniform_bind_group: Option<wgpu::BindGroup>,
    /// 当前表面尺寸
    surface_size: (u32, u32),
    /// 窗口表面（窗口模式）
    surface: Option<wgpu::Surface<'static>>,
    /// 表面格式
    surface_format: wgpu::TextureFormat,
    /// 无头渲染目标纹理
    headless_texture: Option<wgpu::Texture>,
    /// 无头 ping-pong 第二纹理（DC-9 后处理：filter:opacity 等区域读+写需双纹理）
    headless_texture_b: Option<wgpu::Texture>,
    /// 是否暂停向窗口 surface present（Wayland 失焦时使用）
    present_suspended: bool,
    /// Linux：compositor 导入纹理（P0 GPU 零拷贝 blit）。
    #[cfg(target_os = "linux")]
    compositor_import: Option<CompositorImport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct GpuImageCacheKey {
    image_key: crate::image_cache::ImageKey,
    width: u32,
    height: u32,
    content_hash: u64,
}

struct CachedImageResource {
    _texture: wgpu::Texture,
    bind_group: Arc<wgpu::BindGroup>,
    /// 最近访问代数（R3254-M3：预算逐出排序依据）。
    last_used: u64,
    /// 纹理显存字节数（R3254-M3：预算核算）。
    byte_size: usize,
}

/// compositor 导入帧（Browser 侧 wgpu 外部纹理）。
#[cfg(target_os = "linux")]
struct CompositorImport {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    dst_x: f32,
    dst_y: f32,
}

enum RenderTarget {
    Surface {
        output: wgpu::SurfaceTexture,
        view: wgpu::TextureView,
    },
    Headless {
        view: wgpu::TextureView,
    },
}

impl RenderTarget {
    fn view(&self) -> &wgpu::TextureView {
        match self {
            Self::Surface { view, .. } | Self::Headless { view } => view,
        }
    }

    fn present(self, device: &wgpu::Device) {
        if let Self::Surface { output, .. } = self {
            output.present();
            GpuRenderer::poll_after_present(device);
        }
    }
}

impl GpuRenderer {
    /// 创建无头模式的 GPU 渲染器（用于测试和 CPU 回读）
    pub fn new_headless(width: u32, height: u32) -> Result<Self, String> {
        let _guard = GPU_CREATE_MUTEX.lock().unwrap();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // 优先请求软件 fallback adapter（Linux lavapipe/LLVMpipe，输出确定性利于测试）；
        // 无软件 fallback 的平台（macOS 无 software Metal、部分 Windows CI）回退到真实 GPU adapter。
        // 历史：单用 force_fallback_adapter:true 在 macOS CI（2026-06-15 迁移 macOS 26 后）返回 None，
        // 致 54 个 gpu::renderer::tests panic、main CI 红逾月（2026-05-30 后全 failure）。
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: true,
            compatible_surface: None,
        }))
        .or_else(|| {
            pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: None,
            }))
        })
        .ok_or("无法获取 wgpu 适配器（软件 fallback 与真实 GPU 均不可用）")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("ZeroWeb GPU Device (headless)"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .map_err(|e| format!("设备请求失败: {e}"))?;

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        let format = wgpu::TextureFormat::Rgba8UnormSrgb;
        let headless_texture = create_headless_texture(&device, width, height, format);

        Self::from_device(device, queue, format, Some(headless_texture), None)
    }

    /// 创建窗口模式的 GPU 渲染器
    pub fn new_for_window(window: Arc<winit::window::Window>) -> Result<Self, String> {
        let _guard = GPU_CREATE_MUTEX.lock().unwrap();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("表面创建失败: {e}"))?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))
        .ok_or("无法获取支持表面的 wgpu 适配器")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("ZeroWeb GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
                memory_hints: wgpu::MemoryHints::Performance,
            },
            None,
        ))
        .map_err(|e| format!("设备请求失败: {e}"))?;

        // 选择表面格式
        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .find(|f| matches!(f, wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm))
            .copied()
            .unwrap_or(caps.formats[0]);

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        Self::from_device(device, queue, format, None, Some(surface))
    }

    /// 从已有设备构建渲染器
    fn from_device(
        device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>,
        format: wgpu::TextureFormat,
        headless_texture: Option<wgpu::Texture>,
        surface: Option<wgpu::Surface<'static>>,
    ) -> Result<Self, String> {
        let uniform_bgl = create_uniform_bind_group_layout(&device);
        let atlas_bgl = create_atlas_bind_group_layout(&device);
        let gradient_bgl = create_texture_bind_group_layout(&device, "Gradient BGL");
        let image_bgl = create_texture_bind_group_layout(&device, "Image BGL");
        let blur_bgl = create_texture_bind_group_layout(&device, "Blur BGL");
        let image_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Image Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let pipeline = create_render_pipeline(&device, format, &uniform_bgl, &atlas_bgl);
        let rounded_rect_pipeline = create_rounded_rect_pipeline(&device, format, &uniform_bgl);
        let gradient_pipeline = create_gradient_pipeline(&device, format, &uniform_bgl, &gradient_bgl);
        let image_pipeline = create_image_pipeline(&device, format, &uniform_bgl, &image_bgl);
        let blur_pipeline = create_blur_pipeline(&device, format, &uniform_bgl, &blur_bgl);
        // DC-9 filter:opacity 后处理管线（复用 blur_bgl 源纹理布局）
        let color_filter_pipeline = create_color_filter_pipeline(&device, format, &uniform_bgl, &blur_bgl);
        // DC-9 transform 后处理管线（独立 uniform bgl + 复用 blur_bgl 源纹理布局）
        let transform_uniform_bgl = create_transform_uniform_bgl(&device);
        let transform_pipeline = create_transform_pipeline(&device, format, &transform_uniform_bgl, &blur_bgl);

        let atlas = GlyphAtlas::new();
        let (atlas_texture, _atlas_view, _atlas_sampler, atlas_bind_group) =
            create_atlas_resources(&device, &atlas_bgl);

        let surface_size = if let Some(ref tex) = headless_texture {
            let size = tex.size();
            (size.width, size.height)
        } else {
            (800, 600)
        };

        Ok(Self {
            device,
            queue,
            pipeline,
            rounded_rect_pipeline,
            gradient_pipeline,
            image_pipeline,
            blur_pipeline,
            color_filter_pipeline,
            transform_pipeline,
            uniform_bgl,
            transform_uniform_bgl,
            atlas_bgl,
            gradient_bgl,
            image_bgl,
            image_sampler,
            image_texture_cache: std::collections::HashMap::new(),
            // R3254-M3：256MB 显存预算（默认；条目上限 8192 兜底）。
            image_texture_budget_bytes: 256 * 1024 * 1024,
            image_texture_tick: 0,
            blur_bgl,
            atlas,
            atlas_texture,
            atlas_bind_group,
            uniform_buffer: None,
            uniform_bind_group: None,
            surface_size,
            surface,
            surface_format: format,
            headless_texture,
            headless_texture_b: None,
            present_suspended: false,
            #[cfg(target_os = "linux")]
            compositor_import: None,
        })
    }

    /// 失焦时暂停 present，并排空 GPU 队列中已提交的 swapchain 帧。
    pub fn suspend_present(&mut self) {
        self.present_suspended = true;
        if self.surface.is_some() {
            self.device.poll(wgpu::Maintain::Wait);
        }
    }

    /// 重新获焦并完成 surface 配置后恢复 present。
    pub fn resume_present(&mut self) {
        self.present_suspended = false;
    }

    /// 是否已暂停 present
    pub fn is_present_suspended(&self) -> bool {
        self.present_suspended
    }

    fn wayland_frame_latency() -> u32 {
        #[cfg(target_os = "linux")]
        {
            let on_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
                || std::env::var("WINIT_UNIX_BACKEND")
                    .map(|v| v.eq_ignore_ascii_case("wayland"))
                    .unwrap_or(false);
            if on_wayland {
                return 1;
            }
        }
        2
    }

    /// 配置窗口表面（在首次渲染或窗口大小变更时调用）
    pub fn configure_surface(&mut self, width: u32, height: u32) {
        let w = width.max(1);
        let h = height.max(1);

        if let Some(surface) = &self.surface {
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: self.surface_format,
                width: w,
                height: h,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: Self::wayland_frame_latency(),
            };
            surface.configure(&self.device, &config);
        }
        self.surface_size = (w, h);

        // 更新无头纹理尺寸
        if self.headless_texture.is_some() {
            self.headless_texture = Some(create_headless_texture(&self.device, w, h, self.surface_format));
        }
    }

    /// 上传 glyph 位图到 atlas 纹理，返回放置信息（所有权）
    #[allow(clippy::too_many_arguments)]
    pub fn upload_glyph_to_atlas(
        &mut self,
        key: GlyphAtlasKey,
        bitmap_data: &[u8],
        width: u32,
        height: u32,
        x_offset: i16,
        y_offset: i16,
        advance: f32,
    ) -> Option<crate::gpu::atlas::AtlasPlacement> {
        if width == 0 || height == 0 {
            return None;
        }

        match self
            .atlas
            .place(key.clone(), width, height, x_offset, y_offset, advance)
        {
            Some(result) => {
                if result.is_new {
                    // 新 glyph — 上传到 GPU 纹理
                    let padded = GlyphAtlas::create_upload_buffer(bitmap_data, width);
                    self.queue.write_texture(
                        wgpu::TexelCopyTextureInfo {
                            texture: &self.atlas_texture,
                            mip_level: 0,
                            origin: wgpu::Origin3d {
                                x: result.placement.x,
                                y: result.placement.y,
                                z: 0,
                            },
                            aspect: wgpu::TextureAspect::All,
                        },
                        &padded,
                        GlyphAtlas::row_stride(width),
                        wgpu::Extent3d {
                            width,
                            height,
                            depth_or_array_layers: 1,
                        },
                    );
                }
                Some(result.placement)
            }
            None => {
                // Atlas 满了：清除 CPU 端 atlas 记录并递增 generation。
                // 不在这里重试放置——返回 None 让调用者知道
                // 之前已生成的顶点数据（引用旧 UV）已失效，
                // 整个 glyph 收集过程应从头重新开始。
                self.atlas.clear();
                None
            }
        }
    }

    /// 渲染填充矩形和 glyph 文本到当前表面
    #[allow(clippy::too_many_arguments)]
    pub fn render_scene(
        &mut self,
        fills: &[FillPrimitive],
        font_loader: &FontLoader,
        glyph_cache: &mut GlyphCache,
        glyphs: &[GlyphDraw],
        overlay_fills: &[FillPrimitive],
    ) {
        self.render_scene_scaled(fills, font_loader, glyph_cache, glyphs, overlay_fills, 1.0);
    }

    /// 渲染填充矩形和 glyph 文本到当前表面，并支持 overlay_glyphs。
    #[allow(clippy::too_many_arguments)]
    pub fn render_scene_ext(
        &mut self,
        fills: &[FillPrimitive],
        font_loader: &FontLoader,
        glyph_cache: &mut GlyphCache,
        glyphs: &[GlyphDraw],
        overlay_fills: &[FillPrimitive],
        overlay_glyphs: &[GlyphDraw],
    ) {
        self.render_scene_with_clip_scaled(
            fills,
            font_loader,
            glyph_cache,
            glyphs,
            overlay_fills,
            overlay_glyphs,
            None,
            1.0,
        );
    }

    /// 渲染填充矩形和 glyph 文本到当前表面，并应用逻辑像素到物理像素的缩放。
    #[allow(clippy::too_many_arguments)]
    pub fn render_scene_scaled(
        &mut self,
        fills: &[FillPrimitive],
        font_loader: &FontLoader,
        glyph_cache: &mut GlyphCache,
        glyphs: &[GlyphDraw],
        overlay_fills: &[FillPrimitive],
        scale_factor: f32,
    ) {
        self.render_scene_with_clip_scaled(
            fills,
            font_loader,
            glyph_cache,
            glyphs,
            overlay_fills,
            &[],
            None,
            scale_factor,
        );
    }

    /// 渲染填充矩形和 glyph 文本到当前表面（带可选裁剪区域）
    #[allow(clippy::too_many_arguments)]
    pub fn render_scene_with_clip(
        &mut self,
        fills: &[FillPrimitive],
        font_loader: &FontLoader,
        glyph_cache: &mut GlyphCache,
        glyphs: &[GlyphDraw],
        overlay_fills: &[FillPrimitive],
        clip_rect: Option<Rect>,
    ) {
        self.render_scene_with_clip_scaled(
            fills,
            font_loader,
            glyph_cache,
            glyphs,
            overlay_fills,
            &[],
            clip_rect,
            1.0,
        );
    }

    /// 渲染填充矩形和 glyph 文本到当前表面（带可选裁剪区域和缩放）。
    #[allow(clippy::too_many_arguments)]
    pub fn render_scene_with_clip_scaled(
        &mut self,
        fills: &[FillPrimitive],
        font_loader: &FontLoader,
        glyph_cache: &mut GlyphCache,
        glyphs: &[GlyphDraw],
        overlay_fills: &[FillPrimitive],
        overlay_glyphs: &[GlyphDraw],
        clip_rect: Option<Rect>,
        scale_factor: f32,
    ) {
        let scale = normalize_scale_factor(scale_factor);
        let mut vertices: Vec<f32> = Vec::new();

        // 1. 填充矩形
        for fill in fills {
            push_fill_quad(
                &mut vertices,
                fill.rect.left() * scale,
                fill.rect.top() * scale,
                fill.rect.right() * scale,
                fill.rect.bottom() * scale,
                fill.color,
            );
        }

        // 2. Glyph 文本
        // 先收集所有 glyph 位图数据，避免同时借用 glyph_cache 和 self
        type GlyphDataItem = (GlyphAtlasKey, f32, f32, Color, f32, crate::font::GlyphBitmap, f32);
        let glyph_data: Vec<GlyphDataItem> = glyphs
            .iter()
            .filter_map(|gd| {
                let physical_font_size = gd.font_size * scale;
                let (resolved_id, bitmap) = gd.rasterize(font_loader, physical_font_size)?;
                let cache_key = gd.cache_key(resolved_id, physical_font_size);
                let cached = glyph_cache.get_or_insert_with(cache_key, || Ok(bitmap)).ok()?;
                Some((
                    gd.atlas_key(resolved_id, physical_font_size),
                    gd.x * scale,
                    gd.baseline_y * scale,
                    gd.color,
                    physical_font_size,
                    cached.clone(),
                    gd.rotation,
                ))
            })
            .collect();

        // LAY-02: 预先收集 overlay glyph 位图数据（避免在重试循环内重复借用 glyph_cache）
        let og_data: Vec<GlyphDataItem> = if !overlay_glyphs.is_empty() {
            overlay_glyphs
                .iter()
                .filter_map(|gd| {
                    let physical_font_size = gd.font_size * scale;
                    let (resolved_id, bitmap) = gd.rasterize(font_loader, physical_font_size)?;
                    let cache_key = gd.cache_key(resolved_id, physical_font_size);
                    let cached = glyph_cache.get_or_insert_with(cache_key, || Ok(bitmap)).ok()?;
                    Some((
                        gd.atlas_key(resolved_id, physical_font_size),
                        gd.x * scale,
                        gd.baseline_y * scale,
                        gd.color,
                        physical_font_size,
                        cached.clone(),
                        gd.rotation,
                    ))
                })
                .collect()
        } else {
            vec![]
        };

        // LAY-02: atlas 溢出时需要丢弃已有 glyph 顶点（旧 UV 失效）并从头重新收集。
        // fill 顶点不使用 atlas UV，保留即可。
        let fill_vertex_end = vertices.len();
        let mut atlas_retries = 0u32;
        'retry_glyphs: loop {
            vertices.truncate(fill_vertex_end);

            // 2. 主 glyph 文本
            for (atlas_key, x, baseline_y, color, _font_size, bitmap, rotation) in &glyph_data {
                let placement = match self.upload_glyph_to_atlas(
                    atlas_key.clone(),
                    &bitmap.data,
                    bitmap.width as u32,
                    bitmap.height as u32,
                    bitmap.x_offset,
                    bitmap.y_offset,
                    bitmap.advance,
                ) {
                    Some(p) => p,
                    None => {
                        // Atlas 已清除，旧 UV 全部失效——丢弃所有 glyph 顶点并重试
                        atlas_retries += 1;
                        if atlas_retries > 3 {
                            break 'retry_glyphs;
                        }
                        continue 'retry_glyphs;
                    }
                };

                let (u0, v0, u1, v1) = placement.uv();
                let (gx, gy) = glyph_top_left(
                    *x,
                    *baseline_y,
                    placement.x_offset,
                    placement.y_offset,
                    placement.height as u16,
                );
                let gx = gx.round();
                let gy = gy.round();
                let gw = placement.width as f32;
                let gh = placement.height as f32;
                let (r, g, b) = color_to_f32(*color);

                // R1595 DC-9 GPU parity：90° CW 旋转（匹配 CPU cpu/mod.rs is_rotated_90）。
                // 旋转后 quad 尺寸 gh×gw（swap），UV 角点重映射（orig TR→out TL 等），使 GPU-mode
                // vertical-rl/lr Latin 文本字形旋转正确。
                if (*rotation - std::f32::consts::FRAC_PI_2).abs() < 0.1 {
                    vertices.extend_from_slice(&[gx, gy, u1, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx + gh, gy, u1, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx, gy + gw, u0, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx + gh, gy, u1, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx + gh, gy + gw, u0, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx, gy + gw, u0, v0, r, g, b]);
                } else {
                    vertices.extend_from_slice(&[gx, gy, u0, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx + gw, gy + gh, u1, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
                }
            }

            // 3. Overlay fills（圆角遮罩、边框等，绘制在 glyphs 之上）
            for fill in overlay_fills {
                push_fill_quad(
                    &mut vertices,
                    fill.rect.left() * scale,
                    fill.rect.top() * scale,
                    fill.rect.right() * scale,
                    fill.rect.bottom() * scale,
                    fill.color,
                );
            }

            // 4. Overlay glyphs（最顶层控制元素，如右键菜单的文字）
            for (atlas_key, x, baseline_y, color, _font_size, bitmap, rotation) in &og_data {
                let placement = match self.upload_glyph_to_atlas(
                    atlas_key.clone(),
                    &bitmap.data,
                    bitmap.width as u32,
                    bitmap.height as u32,
                    bitmap.x_offset,
                    bitmap.y_offset,
                    bitmap.advance,
                ) {
                    Some(p) => p,
                    None => {
                        atlas_retries += 1;
                        if atlas_retries > 3 {
                            break 'retry_glyphs;
                        }
                        continue 'retry_glyphs;
                    }
                };
                let (u0, v0, u1, v1) = placement.uv();
                let (gx, gy) = glyph_top_left(
                    *x,
                    *baseline_y,
                    placement.x_offset,
                    placement.y_offset,
                    placement.height as u16,
                );
                let gx = gx.round();
                let gy = gy.round();
                let gw = placement.width as f32;
                let gh = placement.height as f32;
                let (r, g, b) = color_to_f32(*color);
                // R1595 DC-9 GPU parity：90° CW 旋转（同主 glyph 循环）。
                if (*rotation - std::f32::consts::FRAC_PI_2).abs() < 0.1 {
                    vertices.extend_from_slice(&[gx, gy, u1, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx + gh, gy, u1, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx, gy + gw, u0, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx + gh, gy, u1, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx + gh, gy + gw, u0, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx, gy + gw, u0, v0, r, g, b]);
                } else {
                    vertices.extend_from_slice(&[gx, gy, u0, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
                    vertices.extend_from_slice(&[gx + gw, gy + gh, u1, v1, r, g, b]);
                    vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
                }
            }

            break 'retry_glyphs;
        }

        self.render_vertices(&vertices, clip_rect.map(|clip| scale_rect(clip, scale)));
    }

    /// 渲染全部 13 种图元类型到当前表面（GPU 全量渲染）
    ///
    /// 遵循 CSS painting order：shadows → backgrounds → borders → content → overlay → post-processing
    #[allow(clippy::too_many_arguments)]
    pub fn render_full_scene_gpu(
        &mut self,
        primitives: &RenderPrimitives,
        font_loader: &FontLoader,
        glyph_cache: &mut GlyphCache,
        image_cache: Option<&mut ImageCache>,
        ui_glyphs: &[GlyphDraw],
        overlay_fills: &[FillPrimitive],
        overlay_glyphs: &[GlyphDraw],
        overlay_rounded_rects: &[RoundedRectPrimitive],
        scale_factor: f32,
    ) -> bool {
        let scale = normalize_scale_factor(scale_factor);
        let (width, height) = self.surface_size;

        if self.present_suspended {
            return false;
        }

        // P0-1：GPU 生产路径未实现的特性（clips/blend_modes/半透明颜色/带模糊阴影/
        // 窗口模式滤镜变换）静默画错——返回 false 由调用方回退 CPU 整帧重画（慢但对）。
        // 基线：docs/learnings/bugs/cpu-gpu-path-divergence.md
        if !crate::gpu::scene_support::scene_supported(
            primitives,
            ui_glyphs,
            overlay_fills,
            overlay_glyphs,
            overlay_rounded_rects,
            self.headless_texture.is_some(),
        ) {
            return false;
        }

        // ── Phase 1: 收集所有顶点数据（不持有 GPU 资源借用） ──

        // 1. Shadows
        let shadow_verts = self.collect_shadow_vertices(&primitives.shadows, scale);
        // 2. Fills
        let fill_verts = self.collect_fill_vertices(&primitives.fills, scale);
        // 3. RoundedRects
        let rr_verts = self.collect_rounded_rect_vertices(&primitives.rounded_rects, scale);
        // 4. Gradients（预创建纹理和绑定组）
        let grad_resources = self.prepare_gradient_resources(&primitives.gradients, scale);
        // 5. Images（预创建纹理和绑定组）
        let img_resources = self.prepare_image_resources(&primitives.images, image_cache, scale);
        // 6-8. Strokes + PathFills + PathStrokes
        let stroke_verts = self.collect_stroke_vertices(&primitives.strokes, scale);
        let path_fill_verts = self.collect_path_fill_vertices(&primitives.path_fills, scale);
        let path_stroke_verts = self.collect_path_stroke_vertices(&primitives.path_strokes, scale);
        // 9. Glyphs
        let glyph_verts =
            self.collect_glyph_vertices_from_primitives(&primitives.glyphs, font_loader, glyph_cache, scale);
        // 10. Chrome / WebView 文字（GlyphDraw，在 overlay 之前）
        let ui_glyph_verts = self.collect_overlay_glyphs_data(ui_glyphs, font_loader, glyph_cache, scale);
        // 11. Overlay fills
        let overlay_fill_verts = self.collect_fill_vertices(overlay_fills, scale);
        // 11b. Overlay rounded rects（滚动条 thumb 圆角滑块；GPU 以 fill 近似绘制）
        let overlay_rr_verts = self.collect_rounded_rect_vertices(overlay_rounded_rects, scale);
        // 12. Overlay glyphs
        let overlay_glyph_verts = self.collect_overlay_glyphs_data(overlay_glyphs, font_loader, glyph_cache, scale);

        // ── Phase 2: 提交 GPU 命令 ──
        // Uniform buffer/bind group persist across frames; only dimensions are updated.
        let uniform_data: [f32; 4] = [width as f32, height as f32, 0.0, 0.0];
        if self.uniform_buffer.is_none() {
            self.uniform_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&uniform_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }));
        }
        let uniform_buffer = self.uniform_buffer.as_ref().expect("uniform buffer initialized");
        self.queue
            .write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&uniform_data));
        if self.uniform_bind_group.is_none() {
            self.uniform_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform BG"),
                layout: &self.uniform_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            }));
        }

        // 获取渲染目标
        let target = match self.acquire_render_target(width, height) {
            Some(target) => target,
            None => return false,
        };
        let device = self.device.clone();
        let queue = self.queue.clone();
        let uniform_bg = self
            .uniform_bind_group
            .as_ref()
            .expect("uniform bind group initialized");
        let view = target.view();

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Full Scene Encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Full Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // 1. Shadows
            self.draw_fill_pass(&mut pass, uniform_bg, &device, &shadow_verts, "Shadow");
            // 2. Fills
            self.draw_fill_pass(&mut pass, uniform_bg, &device, &fill_verts, "Fill");
            // 3. RoundedRects
            self.draw_rounded_rect_pass(&mut pass, uniform_bg, &device, &rr_verts);
            // 4. Gradients
            self.draw_gradient_pass(&mut pass, uniform_bg, &device, &grad_resources);
            // 4b. Compositor GPU 导入 blit（P0）/ CPU 回退帧 blit（P0-1，跨平台）
            self.draw_compositor_import_pass(&mut pass, uniform_bg, &device);
            // 5. Images
            self.draw_image_pass(&mut pass, uniform_bg, &device, &img_resources);
            // 6-8. Strokes + PathFills + PathStrokes
            self.draw_fill_pass(&mut pass, uniform_bg, &device, &stroke_verts, "Stroke");
            self.draw_fill_pass(&mut pass, uniform_bg, &device, &path_fill_verts, "PathFill");
            self.draw_fill_pass(&mut pass, uniform_bg, &device, &path_stroke_verts, "PathStroke");
            // 9. Glyphs
            self.draw_fill_pass(&mut pass, uniform_bg, &device, &glyph_verts, "Glyph");
            // 10. Chrome / WebView 文字
            self.draw_fill_pass(&mut pass, uniform_bg, &device, &ui_glyph_verts, "UiGlyph");
            // 11. Overlay fills
            self.draw_fill_pass(&mut pass, uniform_bg, &device, &overlay_fill_verts, "OverlayFill");
            // 11b. Overlay rounded rects（滚动条 thumb 等）
            self.draw_rounded_rect_pass(&mut pass, uniform_bg, &device, &overlay_rr_verts);
            // 12. Overlay glyphs
            self.draw_fill_pass(&mut pass, uniform_bg, &device, &overlay_glyph_verts, "OverlayGlyph");
        }

        queue.submit(std::iter::once(encoder.finish()));

        // DC-9 filter:opacity 后处理（仅 headless）。
        //
        // 无 filter 时不触发（零默认回归：438 CPU reftest 与无 filter 的 GPU case 全不受影响）。
        // 对每个单通道颜色滤镜（opacity/brightness/contrast）做 ping-pong 区域后处理
        //（匹配 CPU apply_filter），结果写回 headless_texture(A) 供后续 read_pixels 读取。
        let color_filters = collect_color_filters(&primitives.filters);
        if !color_filters.is_empty() && self.headless_texture.is_some() {
            self.apply_color_filters_headless(width, height, &color_filters, scale);
        }
        // filter:blur（2-pass H+V 高斯，复用 blur_pipeline），同样 ping-pong 区域后处理。
        let blur_filters = collect_blur_filters(&primitives.filters);
        if !blur_filters.is_empty() && self.headless_texture.is_some() {
            self.apply_blur_filters_headless(width, height, &blur_filters, scale);
        }
        // CSS transform（2D 仿射逆矩阵重采样，匹配 CPU apply_transform_post）。
        // 无 transform 时不触发（零默认回归；transform reftest footprint ≈0，仅满足 DC-9 覆盖）。
        let transforms = collect_transforms(&primitives.transforms);
        if !transforms.is_empty() && self.headless_texture.is_some() {
            self.apply_transform_filters_headless(width, height, &transforms, scale);
        }

        target.present(&device);
        true
    }

    /// 上传 CPU 渲染帧（RGBA 行优先）为 GPU 纹理，供 P0-1 回退路径 blit 呈现。
    pub fn upload_frame(&self, width: u32, height: u32, rgba: &[u8]) -> wgpu::Texture {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("CPU Fallback Frame"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            rgba,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        texture
    }

    /// 内部：在 render pass 中使用 fill pipeline 绘制顶点
    fn draw_fill_pass(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        uniform_bg: &wgpu::BindGroup,
        device: &wgpu::Device,
        vertices: &[f32],
        label: &str,
    ) {
        if vertices.is_empty() {
            return;
        }
        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, uniform_bg, &[]);
        pass.set_bind_group(1, &self.atlas_bind_group, &[]);
        pass.set_vertex_buffer(0, vb.slice(..));
        pass.draw(0..(vertices.len() as u32 / FILL_FLOATS_PER_VERTEX as u32), 0..1);
    }

    /// 内部：在 render pass 中绘制圆角矩形
    fn draw_rounded_rect_pass(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        uniform_bg: &wgpu::BindGroup,
        device: &wgpu::Device,
        vertices: &[f32],
    ) {
        if vertices.is_empty() {
            return;
        }
        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("RoundedRect VB"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        pass.set_pipeline(&self.rounded_rect_pipeline);
        pass.set_bind_group(0, uniform_bg, &[]);
        pass.set_vertex_buffer(0, vb.slice(..));
        pass.draw(0..(vertices.len() as u32 / ROUNDED_RECT_FLOATS_PER_VERTEX as u32), 0..1);
    }

    /// 内部：在 render pass 中绘制渐变
    fn draw_gradient_pass(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        uniform_bg: &wgpu::BindGroup,
        device: &wgpu::Device,
        resources: &[(wgpu::BindGroup, Vec<f32>)],
    ) {
        pass.set_pipeline(&self.gradient_pipeline);
        pass.set_bind_group(0, uniform_bg, &[]);
        let flat_vertices: Vec<f32> = resources
            .iter()
            .flat_map(|(_, vertices)| vertices.iter().copied())
            .collect();
        if flat_vertices.is_empty() {
            return;
        }
        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Gradient VB"),
            contents: bytemuck::cast_slice(&flat_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let mut byte_offset = 0u64;
        for (bg, verts) in resources {
            let byte_len = (verts.len() * std::mem::size_of::<f32>()) as u64;
            pass.set_bind_group(1, bg, &[]);
            pass.set_vertex_buffer(0, vb.slice(byte_offset..byte_offset + byte_len));
            pass.draw(0..6, 0..1);
            byte_offset += byte_len;
        }
    }

    /// 内部：在 render pass 中绘制 compositor 导入纹理 / CPU 回退帧
    fn draw_compositor_import_pass(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        uniform_bg: &wgpu::BindGroup,
        device: &wgpu::Device,
    ) {
        let Some(import) = self.compositor_import.as_ref() else {
            return;
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Compositor Import Sampler"),
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compositor Import BG"),
            layout: &self.image_bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&import.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let l = import.dst_x;
        let t = import.dst_y;
        let r = l + import.width as f32;
        let b = t + import.height as f32;
        let verts = vec![
            l, t, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, r, t, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, r, b, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
            l, t, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, r, b, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, l, b, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0,
        ];
        pass.set_pipeline(&self.image_pipeline);
        pass.set_bind_group(0, uniform_bg, &[]);
        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Compositor Import VB"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        pass.set_bind_group(1, &bg, &[]);
        pass.set_vertex_buffer(0, vb.slice(..));
        pass.draw(0..6, 0..1);
    }

    /// 内部：在 render pass 中绘制图片
    fn draw_image_pass(
        &self,
        pass: &mut wgpu::RenderPass<'_>,
        uniform_bg: &wgpu::BindGroup,
        device: &wgpu::Device,
        resources: &[(Arc<wgpu::BindGroup>, Vec<f32>)],
    ) {
        pass.set_pipeline(&self.image_pipeline);
        pass.set_bind_group(0, uniform_bg, &[]);
        let flat_vertices: Vec<f32> = resources
            .iter()
            .flat_map(|(_, vertices)| vertices.iter().copied())
            .collect();
        if flat_vertices.is_empty() {
            return;
        }
        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Image VB"),
            contents: bytemuck::cast_slice(&flat_vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let mut byte_offset = 0u64;
        for (bg, verts) in resources {
            let byte_len = (verts.len() * std::mem::size_of::<f32>()) as u64;
            pass.set_bind_group(1, bg.as_ref(), &[]);
            pass.set_vertex_buffer(0, vb.slice(byte_offset..byte_offset + byte_len));
            pass.draw(0..(verts.len() as u32 / IMAGE_FLOATS_PER_VERTEX as u32), 0..1);
            byte_offset += byte_len;
        }
    }

    // ── 顶点收集方法（纯数据操作，无 GPU 借用冲突） ──

    fn collect_shadow_vertices(&self, shadows: &[crate::primitive::ShadowPrimitive], scale: f32) -> Vec<f32> {
        let mut verts = Vec::new();
        for shadow in shadows {
            let sr = &shadow.rect;
            let spread = shadow.spread_radius * scale;
            let ox = shadow.offset_x * scale;
            let oy = shadow.offset_y * scale;
            let l = sr.left() * scale - spread + ox;
            let t = sr.top() * scale - spread + oy;
            let r = sr.right() * scale + spread + ox;
            let b = sr.bottom() * scale + spread + oy;
            let c = Color::rgba(shadow.color.r, shadow.color.g, shadow.color.b, shadow.color.a);
            push_fill_quad(&mut verts, l, t, r, b, c);
        }
        verts
    }

    fn collect_fill_vertices(&self, fills: &[FillPrimitive], scale: f32) -> Vec<f32> {
        let mut verts = Vec::new();
        for fill in fills {
            let r = &fill.rect;
            push_fill_quad(
                &mut verts,
                r.left() * scale,
                r.top() * scale,
                r.right() * scale,
                r.bottom() * scale,
                fill.color,
            );
        }
        verts
    }

    fn collect_rounded_rect_vertices(&self, rects: &[crate::primitive::RoundedRectPrimitive], scale: f32) -> Vec<f32> {
        let mut verts = Vec::new();
        for rr in rects {
            let r = &rr.rect;
            let l = r.left() * scale;
            let t = r.top() * scale;
            let right = r.right() * scale;
            let b = r.bottom() * scale;
            let (cr, cg, cb) = color_to_f32(rr.color);
            let ca = rr.color.a as f32 / 255.0;
            let tl = rr.top_left_radius * scale;
            let tr = rr.top_right_radius * scale;
            let br = rr.bottom_right_radius * scale;
            let bl = rr.bottom_left_radius * scale;
            let uv = (-1.0f32, -1.0f32);
            let make_v =
                |x: f32, y: f32| -> [f32; 16] { [x, y, uv.0, uv.1, cr, cg, cb, ca, l, t, right, b, tl, tr, br, bl] };
            let v0 = make_v(l, t);
            let v1 = make_v(right, t);
            let v2 = make_v(l, b);
            let v3 = make_v(right, t);
            let v4 = make_v(right, b);
            let v5 = make_v(l, b);
            for v in [&v0, &v1, &v2, &v3, &v4, &v5] {
                verts.extend_from_slice(v);
            }
        }
        verts
    }

    fn prepare_gradient_resources(
        &self,
        gradients: &[crate::primitive::GradientPrimitive],
        scale: f32,
    ) -> Vec<(wgpu::BindGroup, Vec<f32>)> {
        let mut resources = Vec::new();
        for grad in gradients {
            let tex_data = gradient_stops_to_texture(&grad.stops, grad.interpolation);
            let grad_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Gradient Texture"),
                size: wgpu::Extent3d {
                    width: 256,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &grad_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &tex_data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(256 * 4),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: 256,
                    height: 1,
                    depth_or_array_layers: 1,
                },
            );
            let grad_view = grad_texture.create_view(&wgpu::TextureViewDescriptor::default());
            let grad_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("Gradient Sampler"),
                address_mode_u: wgpu::AddressMode::ClampToEdge,
                address_mode_v: wgpu::AddressMode::ClampToEdge,
                address_mode_w: wgpu::AddressMode::ClampToEdge,
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                mipmap_filter: wgpu::FilterMode::Nearest,
                ..Default::default()
            });
            let grad_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Gradient BG"),
                layout: &self.gradient_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&grad_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&grad_sampler),
                    },
                ],
            });

            let r = &grad.rect;
            let l = r.left() * scale;
            let t = r.top() * scale;
            let right = r.right() * scale;
            let b = r.bottom() * scale;
            let (gt, p0, p1, p2, p3) = match &grad.kind {
                GradientKind::Linear { x0, y0, x1, y1 } => {
                    let y1_scaled = *y1 * scale;
                    // 用负值编码 repeating 标志
                    let p3 = if grad.repeating { -y1_scaled } else { y1_scaled };
                    (0.0f32, *x0 * scale, *y0 * scale, *x1 * scale, p3)
                }
                GradientKind::Radial {
                    cx,
                    cy,
                    inner_radius,
                    outer_radius,
                } => {
                    let outer_scaled = *outer_radius * scale;
                    let p3 = if grad.repeating { -outer_scaled } else { outer_scaled };
                    (1.0f32, *cx * scale, *cy * scale, *inner_radius * scale, p3)
                }
                GradientKind::Conic { cx, cy, start_angle } => (2.0f32, *cx * scale, *cy * scale, *start_angle, 0.0),
            };
            let make_gv =
                |x: f32, y: f32| -> [f32; GRADIENT_FLOATS_PER_VERTEX] { [x, y, x, y, gt, p0, p1, p2, p3, 0.0] };
            let verts: Vec<f32> = [
                make_gv(l, t),
                make_gv(right, t),
                make_gv(l, b),
                make_gv(right, t),
                make_gv(right, b),
                make_gv(l, b),
            ]
            .concat();
            resources.push((grad_bg, verts));
        }
        resources
    }

    fn prepare_image_resources(
        &mut self,
        images: &[crate::primitive::ImagePrimitive],
        image_cache: Option<&mut ImageCache>,
        scale: f32,
    ) -> Vec<(Arc<wgpu::BindGroup>, Vec<f32>)> {
        let ic = match image_cache {
            Some(c) => c,
            None => return Vec::new(),
        };
        let mut resources = Vec::new();
        for img in images {
            let image_data = match ic.get(&img.image_key) {
                Some(d) => d,
                None => continue,
            };
            let (iw, ih) = (image_data.width, image_data.height);
            if iw == 0 || ih == 0 {
                ic.release(&img.image_key);
                continue;
            }

            let bg = self.cached_image_bind_group(&img.image_key, image_data);

            // crop 语义（R294）：source 始终映射到完整 img.rect（保持原始分辨率）。
            // 绘制区域 = rect ∩ clip（None 时 = rect）；UV 取 clip 窗口在 rect 内的归一化
            // 位置，使裁剪=遮罩而非缩放（clip:rect / overflow:hidden / clip-path inset）。
            let r = img.rect;
            let rect_l = r.left();
            let rect_t = r.top();
            let rect_w = (r.right() - r.left()).max(1e-6);
            let rect_h = (r.bottom() - r.top()).max(1e-6);
            let (l, t, right, b, u0, v0, u1, v1) = match &img.clip {
                Some(clip) => {
                    let cl = clip.left();
                    let ct = clip.top();
                    let cr = clip.right();
                    let cb = clip.bottom();
                    (
                        cl * scale,
                        ct * scale,
                        cr * scale,
                        cb * scale,
                        ((cl - rect_l) / rect_w).clamp(0.0, 1.0),
                        ((ct - rect_t) / rect_h).clamp(0.0, 1.0),
                        ((cr - rect_l) / rect_w).clamp(0.0, 1.0),
                        ((cb - rect_t) / rect_h).clamp(0.0, 1.0),
                    )
                }
                None => (
                    rect_l * scale,
                    rect_t * scale,
                    r.right() * scale,
                    r.bottom() * scale,
                    0.0,
                    0.0,
                    1.0,
                    1.0,
                ),
            };
            let verts: Vec<f32> = vec![
                l, t, u0, v0, 1.0, 1.0, 1.0, right, t, u1, v0, 1.0, 1.0, 1.0, l, b, u0, v1, 1.0, 1.0, 1.0, right, t,
                u1, v0, 1.0, 1.0, 1.0, right, b, u1, v1, 1.0, 1.0, 1.0, l, b, u0, v1, 1.0, 1.0, 1.0,
            ];
            resources.push((bg, verts));
            ic.release(&img.image_key);
        }
        resources
    }

    fn cached_image_bind_group(
        &mut self,
        image_key: &crate::image_cache::ImageKey,
        image_data: &crate::image_cache::ImageData,
    ) -> Arc<wgpu::BindGroup> {
        let cache_key = GpuImageCacheKey {
            image_key: image_key.clone(),
            width: image_data.width,
            height: image_data.height,
            // R3254-M2：读插入时预存的像素摘要（每帧全量哈希是主线程 CPU 大头）。
            content_hash: image_data.content_hash,
        };
        if !self.image_texture_cache.contains_key(&cache_key) {
            // R3254-M3：条目数硬上限兜底（全清会引发一次整帧回传抖动——仅极端场景触发）。
            if self.image_texture_cache.len() >= 8192 {
                self.image_texture_cache.clear();
            }
            let byte_size = (image_data.width as usize) * (image_data.height as usize) * 4;
            // R3254-M3：显存字节预算——超预算按 last_used 逐出（wgpu Texture drop 即释放）。
            if !self.image_texture_cache.is_empty() {
                let budget = self.image_texture_budget_bytes;
                let used: usize = self.image_texture_cache.values().map(|r| r.byte_size).sum();
                if used + byte_size > budget {
                    let mut candidates: Vec<(u64, GpuImageCacheKey)> = self
                        .image_texture_cache
                        .iter()
                        .map(|(k, r)| (r.last_used, k.clone()))
                        .collect();
                    candidates.sort_by_key(|(last_used, _)| *last_used);
                    for (_, key) in candidates {
                        if self.image_texture_cache.values().map(|r| r.byte_size).sum::<usize>() + byte_size <= budget {
                            break;
                        }
                        self.image_texture_cache.remove(&key);
                    }
                }
            }
            let texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Image Texture"),
                size: wgpu::Extent3d {
                    width: image_data.width,
                    height: image_data.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                // ImageData stores already-colored byte-space pixels. Sampling
                // as sRGB would decode them again and darken translucent images.
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            self.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &image_data.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(image_data.width * 4),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: image_data.width,
                    height: image_data.height,
                    depth_or_array_layers: 1,
                },
            );
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let bind_group = Arc::new(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Image BG"),
                layout: &self.image_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.image_sampler),
                    },
                ],
            }));
            self.image_texture_tick = self.image_texture_tick.saturating_add(1);
            self.image_texture_cache.insert(
                cache_key.clone(),
                CachedImageResource {
                    _texture: texture,
                    bind_group,
                    last_used: self.image_texture_tick,
                    byte_size,
                },
            );
        }
        // R3254-M3：命中刷新 last_used（LRU 逐出排序依据）。
        if let Some(resource) = self.image_texture_cache.get_mut(&cache_key) {
            self.image_texture_tick = self.image_texture_tick.saturating_add(1);
            resource.last_used = self.image_texture_tick;
        }
        self.image_texture_cache
            .get(&cache_key)
            .expect("cached image resource inserted")
            .bind_group
            .clone()
    }

    /// R3254-M4：清空图片纹理缓存（导航 epoch 变化 / 标签切换时调用——旧页纹理滞留
    /// 会累积显存；清空后同内容图片下帧重建纹理，仅重传开销，无害）。
    pub fn clear_image_texture_cache(&mut self) {
        self.image_texture_cache.clear();
    }

    fn collect_stroke_vertices(&self, strokes: &[crate::primitive::StrokePrimitive], scale: f32) -> Vec<f32> {
        let mut verts = Vec::new();
        for stroke in strokes {
            push_stroke_mesh(&mut verts, stroke, scale);
        }
        verts
    }

    fn collect_path_fill_vertices(&self, paths: &[crate::primitive::PathFillPrimitive], scale: f32) -> Vec<f32> {
        let mut verts = Vec::new();
        for pf in paths {
            push_path_fill_mesh(&mut verts, pf, scale);
        }
        verts
    }

    fn collect_path_stroke_vertices(&self, paths: &[crate::primitive::PathStrokePrimitive], scale: f32) -> Vec<f32> {
        let mut verts = Vec::new();
        for ps in paths {
            push_path_stroke_mesh(&mut verts, ps, scale);
        }
        verts
    }

    fn collect_glyph_vertices_from_primitives(
        &mut self,
        glyphs: &[crate::primitive::GlyphPrimitive],
        font_loader: &FontLoader,
        glyph_cache: &mut GlyphCache,
        scale: f32,
    ) -> Vec<f32> {
        let mut vertices: Vec<f32> = Vec::new();
        for gp in glyphs {
            let physical_font_size = gp.font_size * scale;
            let font_id = gp.font_id.0;
            let (resolved_id, bitmap) = if let Some(glyph_index) = gp.font_glyph_index() {
                match font_loader.rasterize_glyph_index(font_id, glyph_index, physical_font_size) {
                    Ok(bitmap) => (font_id, bitmap),
                    Err(_) => continue,
                }
            } else {
                let Some(code_point) = gp.code_point() else {
                    continue;
                };
                match font_loader.rasterize_glyph_with_fallback(font_id, code_point, physical_font_size) {
                    Ok(result) => result,
                    Err(_) => continue,
                }
            };
            let cache_key = match gp.font_glyph_index() {
                Some(glyph_index) => {
                    crate::font::cache::GlyphKey::new_indexed(resolved_id, glyph_index, physical_font_size)
                }
                None => crate::font::cache::GlyphKey::new(resolved_id, gp.glyph_id, physical_font_size),
            };
            let cached = match glyph_cache.get_or_insert_with(cache_key, || Ok(bitmap)) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let atlas_key = match gp.font_glyph_index() {
                Some(glyph_index) => GlyphAtlasKey::new_indexed(resolved_id, glyph_index, physical_font_size),
                None => GlyphAtlasKey::new(resolved_id, gp.glyph_id, physical_font_size),
            };
            let placement = match self.upload_glyph_to_atlas(
                atlas_key,
                &cached.data,
                cached.width as u32,
                cached.height as u32,
                cached.x_offset,
                cached.y_offset,
                cached.advance,
            ) {
                Some(p) => p,
                None => continue,
            };
            let (u0, v0, u1, v1) = placement.uv();
            let (gx, gy) = glyph_top_left(
                gp.x * scale,
                gp.y * scale,
                placement.x_offset,
                placement.y_offset,
                placement.height as u16,
            );
            let (gx, gy) = (gx.round(), gy.round());
            let (gw, gh) = (placement.width as f32, placement.height as f32);
            let (r, g, b) = color_to_f32(gp.color);
            vertices.extend_from_slice(&[gx, gy, u0, v0, r, g, b]);
            vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
            vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
            vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
            vertices.extend_from_slice(&[gx + gw, gy + gh, u1, v1, r, g, b]);
            vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
        }
        vertices
    }

    fn collect_overlay_glyphs_data(
        &mut self,
        overlay_glyphs: &[GlyphDraw],
        font_loader: &FontLoader,
        glyph_cache: &mut GlyphCache,
        scale: f32,
    ) -> Vec<f32> {
        let mut vertices: Vec<f32> = Vec::new();
        for gd in overlay_glyphs {
            let physical_font_size = gd.font_size * scale;
            let Some((resolved_id, bitmap)) = gd.rasterize(font_loader, physical_font_size) else {
                continue;
            };
            let cache_key = gd.cache_key(resolved_id, physical_font_size);
            let cached = match glyph_cache.get_or_insert_with(cache_key, || Ok(bitmap)) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let atlas_key = gd.atlas_key(resolved_id, physical_font_size);
            let placement = match self.upload_glyph_to_atlas(
                atlas_key,
                &cached.data,
                cached.width as u32,
                cached.height as u32,
                cached.x_offset,
                cached.y_offset,
                cached.advance,
            ) {
                Some(p) => p,
                None => continue,
            };
            let (u0, v0, u1, v1) = placement.uv();
            let (gx, gy) = glyph_top_left(
                gd.x * scale,
                gd.baseline_y * scale,
                placement.x_offset,
                placement.y_offset,
                placement.height as u16,
            );
            let (gx, gy) = (gx.round(), gy.round());
            let (gw, gh) = (placement.width as f32, placement.height as f32);
            let (r, g, b) = color_to_f32(gd.color);
            // R1595 DC-9 GPU parity：90° CW 旋转（同主 glyph 循环）。
            if (gd.rotation - std::f32::consts::FRAC_PI_2).abs() < 0.1 {
                vertices.extend_from_slice(&[gx, gy, u1, v0, r, g, b]);
                vertices.extend_from_slice(&[gx + gh, gy, u1, v1, r, g, b]);
                vertices.extend_from_slice(&[gx, gy + gw, u0, v0, r, g, b]);
                vertices.extend_from_slice(&[gx + gh, gy, u1, v1, r, g, b]);
                vertices.extend_from_slice(&[gx + gh, gy + gw, u0, v1, r, g, b]);
                vertices.extend_from_slice(&[gx, gy + gw, u0, v0, r, g, b]);
            } else {
                vertices.extend_from_slice(&[gx, gy, u0, v0, r, g, b]);
                vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
                vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
                vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
                vertices.extend_from_slice(&[gx + gw, gy + gh, u1, v1, r, g, b]);
                vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
            }
        }
        vertices
    }

    /// 获取当前帧的渲染目标，并在窗口模式下持有 surface frame 直到 present。
    fn acquire_render_target(&mut self, width: u32, height: u32) -> Option<RenderTarget> {
        match (&self.surface, &self.headless_texture) {
            (Some(surface), _) => {
                let output = match surface.get_current_texture() {
                    Ok(output) => output,
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        self.configure_surface(width, height);
                        return None;
                    }
                    Err(_) => return None,
                };
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
                Some(RenderTarget::Surface { output, view })
            }
            (None, Some(tex)) => Some(RenderTarget::Headless {
                view: tex.create_view(&wgpu::TextureViewDescriptor::default()),
            }),
            _ => None,
        }
    }

    /// 使用顶点数据执行渲染
    fn render_vertices(&mut self, vertices: &[f32], clip_rect: Option<Rect>) {
        if self.present_suspended {
            return;
        }

        let (width, height) = self.surface_size;

        // Uniform 缓冲区（复用持久缓冲区，避免每帧分配）
        let uniform_data: [f32; 4] = [width as f32, height as f32, GlyphAtlas::atlas_size() as f32, 0.0];

        // 按需创建持久缓冲区
        if self.uniform_buffer.is_none() {
            self.uniform_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&uniform_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }));
        }
        let uniform_buffer = self.uniform_buffer.as_ref().unwrap();
        self.queue
            .write_buffer(uniform_buffer, 0, bytemuck::cast_slice(&uniform_data));

        // 按需创建持久绑定组
        if self.uniform_bind_group.is_none() {
            self.uniform_bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Uniform Bind Group"),
                layout: &self.uniform_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            }));
        }
        let uniform_bg = self.uniform_bind_group.as_ref().unwrap();

        // 顶点缓冲区
        let vertex_buffer = if !vertices.is_empty() {
            Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }))
        } else {
            None
        };

        let vertex_count = vertices.len() as u32 / 7;

        // 渲染
        match (&self.surface, &self.headless_texture) {
            (Some(surface), _) => {
                // 窗口模式
                let output = match surface.get_current_texture() {
                    Ok(tex) => tex,
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        self.configure_surface(width, height);
                        return;
                    }
                    Err(_) => return,
                };
                let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

                self.run_render_pass(
                    &mut encoder,
                    &view,
                    uniform_bg,
                    vertex_buffer.as_ref(),
                    vertex_count,
                    clip_rect,
                );

                self.queue.submit(std::iter::once(encoder.finish()));
                output.present();
                Self::poll_after_present(&self.device);
            }
            (None, Some(tex)) => {
                // 无头模式
                let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder (headless)"),
                });

                self.run_render_pass(
                    &mut encoder,
                    &view,
                    uniform_bg,
                    vertex_buffer.as_ref(),
                    vertex_count,
                    clip_rect,
                );

                self.queue.submit(std::iter::once(encoder.finish()));
            }
            _ => {}
        }
    }

    /// present 后驱动 GPU 队列，Wayland 上必须用 Poll 以免阻塞 winit 事件循环。
    fn poll_after_present(device: &wgpu::Device) {
        #[cfg(target_os = "linux")]
        {
            let on_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
                || std::env::var("WINIT_UNIX_BACKEND")
                    .map(|v| v.eq_ignore_ascii_case("wayland"))
                    .unwrap_or(false);
            if on_wayland {
                device.poll(wgpu::Maintain::Poll);
                return;
            }
        }
        device.poll(wgpu::Maintain::Wait);
    }

    /// 从无头纹理回读像素数据（RGBA8）
    ///
    /// 仅在无头模式下可用。返回 RGBA 像素数据（行优先，自上而下）。
    pub fn read_pixels(&self) -> Option<Vec<u8>> {
        let tex = self.headless_texture.as_ref()?;
        let size = tex.size();
        let width = size.width;
        let height = size.height;
        let bytes_per_pixel = 4u32;
        let unpadded_bytes_per_row = width * bytes_per_pixel;

        // 计算对齐后的行步幅（wgpu 要求 256 字节对齐）
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_bytes_per_row = unpadded_bytes_per_row.div_ceil(align) * align;

        // 创建输出缓冲区
        let buffer_size = (padded_bytes_per_row * height) as u64;
        let output_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Pixel Readback Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // 复制纹理到缓冲区
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Pixel Readback Encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &output_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_bytes_per_row),
                    rows_per_image: None,
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(std::iter::once(encoder.finish()));

        // 映射缓冲区并读取数据
        let buffer_slice = output_buffer.slice(..);
        // poll device to ensure copy is complete
        self.device.poll(wgpu::Maintain::Wait);

        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });

        // 等待映射完成
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().ok().and_then(|r| r.ok()).map(|_| {
            let data = buffer_slice.get_mapped_range();
            // 去除每行填充字节
            let mut result = Vec::with_capacity((width * height * bytes_per_pixel) as usize);
            for row in data.chunks(padded_bytes_per_row as usize) {
                result.extend_from_slice(&row[..unpadded_bytes_per_row as usize]);
            }
            drop(data);
            output_buffer.unmap();
            result
        })
    }

    /// 执行渲染 pass
    fn run_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        uniform_bg: &wgpu::BindGroup,
        vertex_buffer: Option<&wgpu::Buffer>,
        vertex_count: u32,
        clip_rect: Option<Rect>,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // 设置裁剪/剪刀区域
        if let Some(clip) = clip_rect {
            let (surface_w, surface_h) = self.surface_size;
            let x = clip.left().max(0.0) as u32;
            let y = clip.top().max(0.0) as u32;
            let right = clip.right().max(0.0).min(surface_w as f32) as u32;
            let bottom = clip.bottom().max(0.0).min(surface_h as f32) as u32;
            let w = right.saturating_sub(x);
            let h = bottom.saturating_sub(y);
            if w > 0 && h > 0 {
                pass.set_scissor_rect(x, y, w, h);
            }
        }

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, uniform_bg, &[]);
        pass.set_bind_group(1, &self.atlas_bind_group, &[]);

        if let Some(vb) = vertex_buffer {
            pass.set_vertex_buffer(0, vb.slice(..));
            pass.draw(0..vertex_count, 0..1);
        }
    }

    /// 获取渲染目标格式
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    /// 获取表面尺寸
    pub fn surface_size(&self) -> (u32, u32) {
        self.surface_size
    }

    /// 是否为窗口模式
    pub fn is_window_mode(&self) -> bool {
        self.surface.is_some()
    }

    /// 获取 atlas generation
    pub fn atlas_generation(&self) -> u64 {
        self.atlas.generation()
    }

    /// 获取已缓存 glyph 数量
    pub fn atlas_glyph_count(&self) -> usize {
        self.atlas.glyph_count()
    }

    /// 获取 wgpu 设备引用（Linux dma-buf 导入等）。
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    /// 获取 wgpu 队列引用。
    pub fn queue(&self) -> &Arc<wgpu::Queue> {
        &self.queue
    }

    /// Linux：设置 compositor 导入纹理并在页面区 blit。
    #[cfg(target_os = "linux")]
    pub fn set_compositor_import(&mut self, texture: wgpu::Texture, width: u32, height: u32, dst_x: f32, dst_y: f32) {
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.compositor_import = Some(CompositorImport {
            _texture: texture,
            view,
            width,
            height,
            dst_x,
            dst_y,
        });
    }

    /// Linux：清除 compositor 导入纹理。
    #[cfg(target_os = "linux")]
    pub fn clear_compositor_import(&mut self) {
        self.compositor_import = None;
    }
}

/// 收集 FilterPrimitive 中的 Opacity 变体 → `(rect, amount)`（DC-9 GPU 后处理输入）。
///
/// `FilterPrimitive.filters` 是按顺序应用的滤镜函数列表；提取所有 Opacity 条目，
/// 其他滤镜（blur/brightness/...）GPU 暂未实现，此处跳过。
/// 收集 FilterPrimitive 中的单通道颜色滤镜 → `(rect, mode, param)`（DC-9 GPU 后处理输入）。
///
/// `mode` 与 COLOR_FILTER_SHADER 对应：0=opacity, 1=brightness, 2=contrast,
/// 3=grayscale, 4=hue-rotate(degrees), 5=invert, 6=saturate, 7=sepia（R285 扩展 3-7，
/// 公式对齐 CPU apply_filter）。blur/drop-shadow 不在此收集（blur 走独立 blur pipeline，
/// drop-shadow CPU 亦 stub）。
fn collect_color_filters(filters: &[crate::primitive::FilterPrimitive]) -> Vec<(Rect, f32, f32)> {
    filters
        .iter()
        .flat_map(|f| {
            f.filters.iter().filter_map(|k| match k {
                FilterKind::Opacity(amount) => Some((f.rect, 0.0, *amount)),
                FilterKind::Brightness(amount) => Some((f.rect, 1.0, *amount)),
                FilterKind::Contrast(amount) => Some((f.rect, 2.0, *amount)),
                FilterKind::Grayscale(amount) => Some((f.rect, 3.0, *amount)),
                FilterKind::HueRotate(degrees) => Some((f.rect, 4.0, *degrees)),
                FilterKind::Invert(amount) => Some((f.rect, 5.0, *amount)),
                FilterKind::Saturate(amount) => Some((f.rect, 6.0, *amount)),
                FilterKind::Sepia(amount) => Some((f.rect, 7.0, *amount)),
                _ => None,
            })
        })
        .collect()
}

/// 收集 FilterPrimitive 中的 Blur 变体 → `(rect, radius_px)`（DC-9 GPU 后处理输入）。
fn collect_blur_filters(filters: &[crate::primitive::FilterPrimitive]) -> Vec<(Rect, f32)> {
    filters
        .iter()
        .flat_map(|f| {
            f.filters.iter().filter_map(|k| match k {
                FilterKind::Blur(radius) => Some((f.rect, *radius)),
                _ => None,
            })
        })
        .collect()
}

/// 一个 TransformPrimitive 经 CPU 侧预计算逆矩阵后的后处理输入（DC-9 GPU）。
///
/// `inv_*` 与 `apply_transform_post`（cpu/mod.rs）公式逐字一致：
/// `src = inv_a*px + inv_c*py + inv_tx + ox` 等（px=x-ox）。奇异矩阵（|det|<1e-10）被丢弃。
struct TransformPost {
    rect: Rect,
    origin_x: f32,
    origin_y: f32,
    inv_a: f32,
    inv_b: f32,
    inv_c: f32,
    inv_d: f32,
    inv_tx: f32,
    inv_ty: f32,
}

/// 收集 `TransformPrimitive` 列表 → 预计算逆矩阵的 `TransformPost`（DC-9 GPU 后处理输入）。
fn collect_transforms(transforms: &[crate::primitive::TransformPrimitive]) -> Vec<TransformPost> {
    transforms
        .iter()
        .filter_map(|t| {
            let det = t.a * t.d - t.b * t.c;
            if det.abs() < 1e-10 {
                return None; // 奇异矩阵，跳过（匹配 CPU apply_transform_post 早退）
            }
            let inv_det = 1.0 / det;
            Some(TransformPost {
                rect: t.rect,
                origin_x: t.origin_x,
                origin_y: t.origin_y,
                inv_a: t.d * inv_det,
                inv_b: -t.b * inv_det,
                inv_c: -t.c * inv_det,
                inv_d: t.a * inv_det,
                inv_tx: (t.c * t.ty - t.d * t.tx) * inv_det,
                inv_ty: (t.b * t.tx - t.a * t.ty) * inv_det,
            })
        })
        .collect()
}

/// 单趟 blur（自由函数，避免与 headless_texture 借用冲突）：copy src→dst（保 dst 的
/// rect 外像素）→ scissor pass 用 blur_pipeline 采样 src、按 `direction`（0=H,1=V）以
/// `radius` blur 写 dst（rect 内）。
#[allow(clippy::too_many_arguments)]
fn run_blur_pass(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    uniform_bgl: &wgpu::BindGroupLayout,
    blur_bgl: &wgpu::BindGroupLayout,
    src: &wgpu::Texture,
    dst: &wgpu::Texture,
    extent: wgpu::Extent3d,
    sampler: &wgpu::Sampler,
    radius: f32,
    direction: f32,
    scissor: (u32, u32, u32, u32),
    label: &str,
) {
    use wgpu::util::DeviceExt;

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("Blur Filter Encoder"),
    });
    // 1. copy src→dst（dst 获得 src 内容作为基底，保 rect 外像素）
    encoder.copy_texture_to_texture(
        wgpu::TexelCopyTextureInfo {
            texture: src,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyTextureInfo {
            texture: dst,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        extent,
    );
    // 2. uniform {screen_w, screen_h, blur_radius, direction}（与 BLUR_SHADER Uniforms 对齐）
    let (w, h) = (extent.width, extent.height);
    let uniform_data: [f32; 4] = [w as f32, h as f32, radius, direction];
    let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Blur Uniform Buffer"),
        contents: bytemuck::cast_slice(&uniform_data),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Blur Uniform BG"),
        layout: uniform_bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: uniform_buffer.as_entire_binding(),
        }],
    });
    let src_view = src.create_view(&wgpu::TextureViewDescriptor::default());
    let src_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Blur Src BG"),
        layout: blur_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&src_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    });
    // 3. scissor pass：采样 src、blur 写 dst（rect 内）
    let dst_view = dst.create_view(&wgpu::TextureViewDescriptor::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some(label),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &dst_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        let (sx, sy, sw, sh) = scissor;
        pass.set_scissor_rect(sx, sy, sw, sh);
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &uniform_bg, &[]);
        pass.set_bind_group(1, &src_bg, &[]);
        pass.draw(0..3, 0..1);
    }
    queue.submit(std::iter::once(encoder.finish()));
}

/// 创建无头渲染目标纹理
fn create_headless_texture(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Headless Render Target"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
            | wgpu::TextureUsages::COPY_SRC
            | wgpu::TextureUsages::COPY_DST
            | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}

/// 创建 atlas 纹理、视图、采样器和绑定组
fn create_atlas_resources(
    device: &wgpu::Device,
    atlas_bgl: &wgpu::BindGroupLayout,
) -> (wgpu::Texture, wgpu::TextureView, wgpu::Sampler, wgpu::BindGroup) {
    let texture = device.create_texture(&GlyphAtlas::texture_descriptor());
    let view = texture.create_view(&GlyphAtlas::view_descriptor());

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Atlas Sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Nearest,
        min_filter: wgpu::FilterMode::Nearest,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("Atlas Bind Group"),
        layout: atlas_bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    });

    (texture, view, sampler, bind_group)
}

/// 将渐变色标转换为 256×1 RGBA 纹理数据
fn gradient_stops_to_texture(
    stops: &[crate::primitive::GradientStop],
    interpolation: crate::primitive::GradientInterpolation,
) -> Vec<u8> {
    let mut tex = vec![0u8; 256 * 4];
    if stops.is_empty() {
        return tex;
    }

    for i in 0..256u32 {
        let t = i as f32 / 255.0;
        // 找到 t 所在的两个色标之间
        let color = if stops.len() == 1 {
            stops[0].color
        } else {
            let mut c = stops[0].color;
            for j in 0..stops.len() - 1 {
                if t >= stops[j].offset && t <= stops[j + 1].offset {
                    let range = stops[j + 1].offset - stops[j].offset;
                    let local_t = if range > 0.0 {
                        (t - stops[j].offset) / range
                    } else {
                        0.0
                    };
                    let local_t = local_t.clamp(0.0, 1.0);
                    c = crate::color_space::interp_pair(
                        stops[j].color,
                        stops[j + 1].color,
                        local_t as f64,
                        interpolation.space,
                        interpolation.hue,
                    );
                    break;
                }
            }
            c
        };
        let idx = i as usize * 4;
        tex[idx] = color.r;
        tex[idx + 1] = color.g;
        tex[idx + 2] = color.b;
        tex[idx + 3] = color.a;
    }
    tex
}

fn normalize_scale_factor(scale_factor: f32) -> f32 {
    if scale_factor.is_finite() && scale_factor > 0.0 {
        scale_factor
    } else {
        1.0
    }
}

fn scale_rect(rect: Rect, scale: f32) -> Rect {
    Rect::new(
        rect.origin.x * scale,
        rect.origin.y * scale,
        rect.size.width * scale,
        rect.size.height * scale,
    )
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod parity_tests;
