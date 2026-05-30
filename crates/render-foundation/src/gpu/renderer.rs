//! GPU 渲染器 — 组合 wgpu 上下文、glyph atlas 和渲染管线
//!
//! 提供两种模式：
//! - **窗口模式**: 直接渲染到 wgpu Surface（GPU 合成到屏幕）
//! - **无头模式**: 渲染到纹理后回读像素（CPU 后备 / 测试用）

use crate::color::Color;
use crate::font::cache::GlyphCache;
use crate::font::loader::FontLoader;
use crate::gpu::atlas::{GlyphAtlas, GlyphAtlasKey};
use crate::gpu::pipeline::{
    create_atlas_bind_group_layout, create_render_pipeline, create_uniform_bind_group_layout,
};
use crate::primitive::FillPrimitive;

/// 渲染场景中的 glyph 文本参数
#[derive(Debug, Clone)]
pub struct GlyphDraw {
    /// 字符
    pub ch: char,
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
}
use std::sync::Arc;
use wgpu::util::DeviceExt;

/// GPU 渲染器 — 管理 wgpu 设备、atlas 和渲染管线
pub struct GpuRenderer {
    /// wgpu 设备
    device: Arc<wgpu::Device>,
    /// wgpu 队列
    queue: Arc<wgpu::Queue>,
    /// 渲染管线
    pipeline: wgpu::RenderPipeline,
    /// Uniform 绑定组布局
    uniform_bgl: wgpu::BindGroupLayout,
    /// Atlas 绑定组布局（保留用于 atlas 重建时重新创建绑定组）
    #[allow(dead_code)]
    atlas_bgl: wgpu::BindGroupLayout,
    /// Glyph Atlas（CPU 侧放置追踪）
    atlas: GlyphAtlas,
    /// Atlas 纹理
    atlas_texture: wgpu::Texture,
    /// Atlas 绑定组
    atlas_bind_group: wgpu::BindGroup,
    /// 当前表面尺寸
    surface_size: (u32, u32),
    /// 窗口表面（窗口模式）
    surface: Option<wgpu::Surface<'static>>,
    /// 表面格式
    surface_format: wgpu::TextureFormat,
    /// 无头渲染目标纹理
    headless_texture: Option<wgpu::Texture>,
}

impl GpuRenderer {
    /// 创建无头模式的 GPU 渲染器（用于测试和 CPU 回读）
    pub fn new_headless(width: u32, height: u32) -> Result<Self, String> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            force_fallback_adapter: true,
            compatible_surface: None,
        }))
        .ok_or("无法获取 wgpu 适配器")?;

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("ZeroBrowser GPU Device (headless)"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
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
                label: Some("ZeroBrowser GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
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
            .find(|f| {
                matches!(
                    f,
                    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
                )
            })
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
        let pipeline = create_render_pipeline(&device, format, &uniform_bgl, &atlas_bgl);
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
            uniform_bgl,
            atlas_bgl,
            atlas,
            atlas_texture,
            atlas_bind_group,
            surface_size,
            surface,
            surface_format: format,
            headless_texture,
        })
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
                desired_maximum_frame_latency: 2,
            };
            surface.configure(&self.device, &config);
        }
        self.surface_size = (w, h);

        // 更新无头纹理尺寸
        if self.headless_texture.is_some() {
            self.headless_texture = Some(create_headless_texture(
                &self.device,
                w,
                h,
                self.surface_format,
            ));
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
        match self.atlas.place(key.clone(), width, height, x_offset, y_offset, advance) {
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
                // Atlas 满了，清空重建
                self.atlas.clear();
                // 重试
                self.atlas
                    .place(key, width, height, x_offset, y_offset, advance)
                    .map(|result| {
                        if result.is_new {
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
                        result.placement
                    })
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
    ) {
        let mut vertices: Vec<f32> = Vec::new();

        // 1. 填充矩形
        for fill in fills {
            push_fill_quad(
                &mut vertices,
                fill.rect.left(),
                fill.rect.top(),
                fill.rect.right(),
                fill.rect.bottom(),
                fill.color,
            );
        }

        // 2. Glyph 文本
        // 先收集所有 glyph 位图数据，避免同时借用 glyph_cache 和 self
        let glyph_data: Vec<(char, f32, f32, Color, u32, f32, crate::font::GlyphBitmap)> = glyphs
            .iter()
            .filter_map(|gd| {
                let cache_key = crate::font::cache::GlyphKey::new(gd.font_id, gd.ch as u32, gd.font_size);
                glyph_cache
                    .get_or_insert_with(cache_key, || {
                        font_loader.rasterize_glyph(gd.font_id, gd.ch, gd.font_size)
                    })
                    .ok()
                    .map(|bitmap| (gd.ch, gd.x, gd.baseline_y, gd.color, gd.font_id, gd.font_size, bitmap.clone()))
            })
            .collect();

        for (ch, x, baseline_y, color, font_id, font_size, bitmap) in glyph_data {
            let atlas_key = GlyphAtlasKey::new(font_id, ch as u32, font_size);
            let placement = match self.upload_glyph_to_atlas(
                atlas_key,
                &bitmap.data,
                bitmap.width as u32,
                bitmap.height as u32,
                bitmap.x_offset,
                bitmap.y_offset,
                bitmap.advance,
            ) {
                Some(p) => p,
                None => continue,
            };

            let (u0, v0, u1, v1) = placement.uv();
            let gx = x + placement.x_offset as f32;
            let gy = baseline_y + placement.y_offset as f32;
            let gw = placement.width as f32;
            let gh = placement.height as f32;
            let (r, g, b) = color_to_f32(color);

            // 6 个顶点（2 个三角形）
            vertices.extend_from_slice(&[gx, gy, u0, v0, r, g, b]);
            vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
            vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
            vertices.extend_from_slice(&[gx + gw, gy, u1, v0, r, g, b]);
            vertices.extend_from_slice(&[gx + gw, gy + gh, u1, v1, r, g, b]);
            vertices.extend_from_slice(&[gx, gy + gh, u0, v1, r, g, b]);
        }

        self.render_vertices(&vertices);
    }

    /// 使用顶点数据执行渲染
    fn render_vertices(&self, vertices: &[f32]) {
        let (width, height) = self.surface_size;

        // Uniform 缓冲区
        let uniform_data: [f32; 4] = [
            width as f32,
            height as f32,
            GlyphAtlas::atlas_size() as f32,
            0.0,
        ];
        let uniform_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Uniform Buffer"),
                contents: bytemuck::cast_slice(&uniform_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let uniform_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Uniform Bind Group"),
            layout: &self.uniform_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 顶点缓冲区
        let vertex_buffer = if !vertices.is_empty() {
            Some(
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Vertex Buffer"),
                        contents: bytemuck::cast_slice(vertices),
                        usage: wgpu::BufferUsages::VERTEX,
                    }),
            )
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
                    Err(_) => return,
                };
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());

                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder"),
                    });

                self.run_render_pass(
                    &mut encoder,
                    &view,
                    &uniform_bg,
                    vertex_buffer.as_ref(),
                    vertex_count,
                );

                self.queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            (None, Some(tex)) => {
                // 无头模式
                let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder = self
                    .device
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("Render Encoder (headless)"),
                    });

                self.run_render_pass(
                    &mut encoder,
                    &view,
                    &uniform_bg,
                    vertex_buffer.as_ref(),
                    vertex_count,
                );

                self.queue.submit(std::iter::once(encoder.finish()));
            }
            _ => {}
        }
    }

    /// 执行渲染 pass
    fn run_render_pass(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        uniform_bg: &wgpu::BindGroup,
        vertex_buffer: Option<&wgpu::Buffer>,
        vertex_count: u32,
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

/// 推入一个填充矩形的 6 个顶点（2 个三角形）
fn push_fill_quad(
    vertices: &mut Vec<f32>,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
    color: Color,
) {
    let (r, g, b) = color_to_f32(color);
    let (u, v) = (-1.0f32, -1.0f32);

    // 三角形 1: 左上 → 右上 → 左下
    vertices.extend_from_slice(&[left, top, u, v, r, g, b]);
    vertices.extend_from_slice(&[right, top, u, v, r, g, b]);
    vertices.extend_from_slice(&[left, bottom, u, v, r, g, b]);
    // 三角形 2: 右上 → 右下 → 左下
    vertices.extend_from_slice(&[right, top, u, v, r, g, b]);
    vertices.extend_from_slice(&[right, bottom, u, v, r, g, b]);
    vertices.extend_from_slice(&[left, bottom, u, v, r, g, b]);
}

/// Color → (f32, f32, f32) 归一化到 [0, 1]
fn color_to_f32(color: Color) -> (f32, f32, f32) {
    (
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_push_fill_quad() {
        let mut vertices = Vec::new();
        push_fill_quad(
            &mut vertices,
            0.0,
            0.0,
            100.0,
            50.0,
            Color::rgba(255, 0, 0, 255),
        );
        // 6 个顶点 × 7 个 float = 42
        assert_eq!(vertices.len(), 42);
        assert_eq!(vertices[2], -1.0); // u
        assert_eq!(vertices[3], -1.0); // v
    }

    #[test]
    fn test_color_to_f32() {
        let (r, g, b) = color_to_f32(Color::rgba(128, 64, 255, 255));
        assert!((r - 128.0 / 255.0).abs() < f32::EPSILON);
        assert!((g - 64.0 / 255.0).abs() < f32::EPSILON);
        assert!((b).abs() > 0.99);
    }

    #[test]
    fn test_push_multiple_fills() {
        let mut vertices = Vec::new();
        for i in 0..5u32 {
            push_fill_quad(
                &mut vertices,
                i as f32 * 10.0,
                0.0,
                i as f32 * 10.0 + 10.0,
                10.0,
                Color::BLACK,
            );
        }
        // 5 × 6 × 7 = 210
        assert_eq!(vertices.len(), 210);
    }
}
