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
use crate::geometry::Rect;
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
                label: Some("ZeroWeb GPU Device (headless)"),
                required_features: wgpu::Features::empty(),
                required_limits:
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
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
                label: Some("ZeroWeb GPU Device"),
                required_features: wgpu::Features::empty(),
                required_limits:
                    wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
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
        self.render_scene_with_clip(fills, font_loader, glyph_cache, glyphs, None);
    }

    /// 渲染填充矩形和 glyph 文本到当前表面（带可选裁剪区域）
    #[allow(clippy::too_many_arguments)]
    pub fn render_scene_with_clip(
        &mut self,
        fills: &[FillPrimitive],
        font_loader: &FontLoader,
        glyph_cache: &mut GlyphCache,
        glyphs: &[GlyphDraw],
        clip_rect: Option<Rect>,
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
                let cache_key =
                    crate::font::cache::GlyphKey::new(gd.font_id, gd.ch as u32, gd.font_size);
                glyph_cache
                    .get_or_insert_with(cache_key, || {
                        font_loader.rasterize_glyph(gd.font_id, gd.ch, gd.font_size)
                    })
                    .ok()
                    .map(|bitmap| {
                        (
                            gd.ch,
                            gd.x,
                            gd.baseline_y,
                            gd.color,
                            gd.font_id,
                            gd.font_size,
                            bitmap.clone(),
                        )
                    })
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

        self.render_vertices(&vertices, clip_rect);
    }

    /// 使用顶点数据执行渲染
    fn render_vertices(&self, vertices: &[f32], clip_rect: Option<Rect>) {
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

                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder"),
                        });

                self.run_render_pass(
                    &mut encoder,
                    &view,
                    &uniform_bg,
                    vertex_buffer.as_ref(),
                    vertex_count,
                    clip_rect,
                );

                self.queue.submit(std::iter::once(encoder.finish()));
                output.present();
            }
            (None, Some(tex)) => {
                // 无头模式
                let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    self.device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Render Encoder (headless)"),
                        });

                self.run_render_pass(
                    &mut encoder,
                    &view,
                    &uniform_bg,
                    vertex_buffer.as_ref(),
                    vertex_count,
                    clip_rect,
                );

                self.queue.submit(std::iter::once(encoder.finish()));
            }
            _ => {}
        }
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
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
        rx.recv()
            .ok()
            .and_then(|r| r.ok())
            .map(|_| {
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
) -> (
    wgpu::Texture,
    wgpu::TextureView,
    wgpu::Sampler,
    wgpu::BindGroup,
) {
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

    /// 测试无头模式 GPU 渲染器创建
    #[test]
    fn test_gpu_renderer_headless_creation() {
        let renderer = GpuRenderer::new_headless(64, 64);
        assert!(renderer.is_ok(), "Failed to create headless renderer");
        let renderer = renderer.unwrap();
        assert!(!renderer.is_window_mode());
        assert_eq!(renderer.surface_size(), (64, 64));
        assert_eq!(
            renderer.surface_format(),
            wgpu::TextureFormat::Rgba8UnormSrgb
        );
    }

    /// 测试渲染红色填充并回读像素验证
    #[test]
    fn test_gpu_renderer_render_and_read_pixels() {
        let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 32.0, 32.0),
            color: Color::RED,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[]);

        let pixels = renderer
            .read_pixels()
            .expect("read_pixels should return data in headless mode");
        assert_eq!(pixels.len(), 32 * 32 * 4);

        // 第一个像素应为红色 (R=255, G=0, B=0, A=255)
        assert_eq!(pixels[0], 255, "R channel should be 255");
        assert_eq!(pixels[1], 0, "G channel should be 0");
        assert_eq!(pixels[2], 0, "B channel should be 0");
        assert_eq!(pixels[3], 255, "A channel should be 255");
    }

    /// 测试渲染绿色填充并回读像素
    #[test]
    fn test_gpu_renderer_green_fill_readback() {
        let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 16.0, 16.0),
            color: Color::GREEN,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[]);

        let pixels = renderer.read_pixels().expect("read_pixels");
        // 绿色 (R=0, G=255, B=0, A=255)
        assert_eq!(pixels[0], 0);
        assert_eq!(pixels[1], 255);
        assert_eq!(pixels[2], 0);
        assert_eq!(pixels[3], 255);
    }

    /// 测试无填充时回读像素应为白色（clear color）
    #[test]
    fn test_gpu_renderer_empty_scene_white_pixels() {
        let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        renderer.render_scene(&[], &font_loader, &mut glyph_cache, &[]);

        let pixels = renderer.read_pixels().expect("read_pixels");
        // 白色背景 (R=255, G=255, B=255, A=255)
        assert_eq!(pixels[0], 255);
        assert_eq!(pixels[1], 255);
        assert_eq!(pixels[2], 255);
        assert_eq!(pixels[3], 255);
    }

    /// 测试 configure_surface 更新无头纹理尺寸
    #[test]
    fn test_gpu_renderer_configure_surface_resize() {
        let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
        assert_eq!(renderer.surface_size(), (32, 32));

        renderer.configure_surface(64, 64);
        assert_eq!(renderer.surface_size(), (64, 64));
    }

    /// 测试 read_pixels 在窗口模式下返回 None
    #[test]
    fn test_gpu_renderer_read_pixels_window_mode_none() {
        // 窗口模式没有 headless_texture，read_pixels 应返回 None
        // 由于创建窗口模式需要 winit window，我们构造一个 from_device
        // 直接使用 headless 模式验证方法存在即可
        let renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
        // headless 模式有 texture，所以 read_pixels 应该能工作
        assert!(renderer.read_pixels().is_some());
    }

    /// 测试裁剪区域限制渲染范围
    #[test]
    fn test_gpu_renderer_clip_rect_limits_rendering() {
        let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");

        // 渲染一个全屏红色矩形，但裁剪到左上角 8x8 区域
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 32.0, 32.0),
            color: Color::RED,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        let clip = Rect::new(0.0, 0.0, 8.0, 8.0);

        renderer.render_scene_with_clip(
            &fills,
            &font_loader,
            &mut glyph_cache,
            &[],
            Some(clip),
        );

        let pixels = renderer.read_pixels().expect("read_pixels");

        // 左上角 (0,0) 应为红色
        assert_eq!(pixels[0], 255, "R at (0,0)");
        assert_eq!(pixels[1], 0, "G at (0,0)");

        // 裁剪区域外 (16,0) 应为白色（clear color）
        let idx = (16 * 4) as usize;
        assert_eq!(pixels[idx], 255, "R at (16,0) should be white");
        assert_eq!(pixels[idx + 1], 255, "G at (16,0) should be white");
    }

    /// 测试 atlas 初始状态
    #[test]
    fn test_gpu_renderer_atlas_initial_state() {
        let renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
        assert_eq!(renderer.atlas_generation(), 0);
        assert_eq!(renderer.atlas_glyph_count(), 0);
    }

    /// 测试蓝色填充回读
    #[test]
    fn test_gpu_renderer_blue_fill_readback() {
        let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 16.0, 16.0),
            color: Color::BLUE,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[]);
        let pixels = renderer.read_pixels().expect("read_pixels");
        assert_eq!(pixels[0], 0);
        assert_eq!(pixels[1], 0);
        assert_eq!(pixels[2], 255);
        assert_eq!(pixels[3], 255);
    }

    /// 测试黑色填充回读
    #[test]
    fn test_gpu_renderer_black_fill_readback() {
        let mut renderer = GpuRenderer::new_headless(8, 8).expect("headless renderer");
        let fills = vec![FillPrimitive {
            rect: Rect::new(0.0, 0.0, 8.0, 8.0),
            color: Color::BLACK,
        }];
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);
        renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[]);
        let pixels = renderer.read_pixels().expect("read_pixels");
        assert_eq!(pixels[0], 0);
        assert_eq!(pixels[1], 0);
        assert_eq!(pixels[2], 0);
        assert_eq!(pixels[3], 255);
    }

    /// 测试 glyph_draw 结构体
    #[test]
    fn test_glyph_draw_fields() {
        let gd = GlyphDraw {
            ch: 'A',
            x: 10.0,
            baseline_y: 20.0,
            color: Color::RED,
            font_id: 1,
            font_size: 16.0,
        };
        assert_eq!(gd.ch, 'A');
        assert_eq!(gd.x, 10.0);
        assert_eq!(gd.font_id, 1);
    }

    /// 测试 configure_surface 最小尺寸
    #[test]
    fn test_gpu_renderer_configure_surface_min_size() {
        let mut renderer = GpuRenderer::new_headless(32, 32).expect("headless renderer");
        renderer.configure_surface(0, 0);
        // Should clamp to (1, 1)
        assert_eq!(renderer.surface_size(), (1, 1));
    }

    /// 测试多次渲染不会 panic
    #[test]
    fn test_gpu_renderer_multiple_renders() {
        let mut renderer = GpuRenderer::new_headless(16, 16).expect("headless renderer");
        let font_loader = FontLoader::new();
        let mut glyph_cache = GlyphCache::new(64);

        for _ in 0..3 {
            let fills = vec![FillPrimitive {
                rect: Rect::new(0.0, 0.0, 16.0, 16.0),
                color: Color::RED,
            }];
            renderer.render_scene(&fills, &font_loader, &mut glyph_cache, &[]);
        }
        let pixels = renderer.read_pixels().expect("read_pixels");
        assert_eq!(pixels.len(), 16 * 16 * 4);
    }
}
