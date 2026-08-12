//! GPU 渲染器 — 区域后处理滤镜 pass（DC-9 ping-pong 双纹理后处理）
//!
//! 从 `mod.rs` 拆分以控制单文件体积（2000 行规则）。包含 headless 模式下三类
//! 区域后处理 pass：
//! - `apply_color_filters_headless` — opacity / brightness / contrast 颜色矩阵
//! - `apply_transform_filters_headless` — 2D 仿射变换（逆变换重采样）
//! - `apply_blur_filters_headless` — 高斯模糊（separable 两趟）
//!
//! 三者均由 `render_full_scene_gpu` 在主场景绘制完成后调用，对 `headless_texture`
//! 做 ping-pong（A↔B 双纹理）区域后处理。
//!
//! 通过 split-impl（`impl super::GpuRenderer`）实现：子模块可访问父模块私有字段，
//! 无可见性变更，纯代码移动。`use super::*;` 复用 `mod.rs` 的全部导入与局部项
//!（Rect、TransformPost、create_headless_texture、run_blur_pass 等）。

use super::*;

impl super::GpuRenderer {
    /// DC-9 单通道颜色滤镜区域后处理（opacity/brightness/contrast，仅 headless）。
    ///
    /// 对每个 `(rect, mode, param)`：copy A→B（B 获得完整场景基底）→ scissor pass 采样 A
    /// 写 B（rect 内按 mode 应用滤镜）→ copy B→A（结果回 A 供 read_pixels）。多滤镜逐个
    /// ping-pong，最终结果在 headless_texture(A)。匹配 CPU `apply_filter` 语义。
    pub(super) fn apply_color_filters_headless(
        &mut self,
        width: u32,
        height: u32,
        filters: &[(Rect, f32, f32)],
        scale: f32,
    ) {
        use wgpu::util::DeviceExt;

        let Some(tex_a) = self.headless_texture.as_ref() else {
            return;
        };
        // 确保 ping-pong 纹理 B 存在且尺寸匹配
        let need_recreate = self
            .headless_texture_b
            .as_ref()
            .map(|t| t.size().width != width || t.size().height != height)
            .unwrap_or(true);
        if need_recreate {
            self.headless_texture_b = Some(create_headless_texture(
                &self.device,
                width,
                height,
                self.surface_format,
            ));
        }
        let Some(tex_b) = self.headless_texture_b.as_ref() else {
            return;
        };

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // 共享源采样器（ClampToEdge + Linear）
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Color Filter Src Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        for &(rect, mode, param) in filters {
            // scissor 钳制到 [0, width]×[0, height]（wgpu 要求非零且在界内）
            let sx = ((rect.left() * scale).floor().max(0.0) as u32).min(width);
            let sy = ((rect.top() * scale).floor().max(0.0) as u32).min(height);
            let sw = ((rect.right() - rect.left()).max(0.0) * scale).ceil() as u32;
            let sh = ((rect.bottom() - rect.top()).max(0.0) * scale).ceil() as u32;
            let sw = sw.min(width.saturating_sub(sx));
            let sh = sh.min(height.saturating_sub(sy));
            if sw == 0 || sh == 0 {
                continue; // 退化 rect，跳过（避免 wgpu 零尺寸 scissor 错误）
            }

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Color Filter Encoder"),
            });

            // 1. copy A→B（B 获得完整场景内容作为基底，保 rect 外像素不变）
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: tex_a,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: tex_b,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                extent,
            );

            // 2. uniform buffer [width, height, mode, param]（16 字节匹配 UNIFORM_SIZE）
            let uniform_data: [f32; 4] = [width as f32, height as f32, mode, param];
            let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Color Filter Uniform Buffer"),
                contents: bytemuck::cast_slice(&uniform_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let uniform_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Color Filter Uniform BG"),
                layout: &self.uniform_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });
            let src_view = tex_a.create_view(&wgpu::TextureViewDescriptor::default());
            let src_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Color Filter Src BG"),
                layout: &self.blur_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

            // 3. scissor pass: 采样 A → 写 B（rect 内 RGB *= amount；rect 外保留 copy 的 A 内容）
            let view_b = tex_b.create_view(&wgpu::TextureViewDescriptor::default());
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Color Filter Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view_b,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_scissor_rect(sx, sy, sw, sh);
                pass.set_pipeline(&self.color_filter_pipeline);
                pass.set_bind_group(0, &uniform_bg, &[]);
                pass.set_bind_group(1, &src_bg, &[]);
                pass.draw(0..3, 0..1);
            }

            // 4. copy B→A（结果回 A，read_pixels 读 A）
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: tex_b,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: tex_a,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                extent,
            );

            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    /// DC-9 transform 区域后处理（2D 仿射逆矩阵重采样，仅 headless）。
    ///
    /// 对每个 `TransformPost`：copy A→B（B 获得完整场景基底，保 rect 外像素）→ scissor pass
    /// 用 transform_pipeline 采样 A、按预计算逆矩阵把目标像素映射回源位置写 B（rect 内）→
    /// copy B→A（结果回 A 供 read_pixels）。逆映射落在 rect 外的像素写白（匹配 CPU clear-to-white）。
    /// uniform = 16 个 f32（64 字节，与 `TRANSFORM_SHADER` 的 `TransformUniforms` 对齐）。
    pub(super) fn apply_transform_filters_headless(
        &mut self,
        width: u32,
        height: u32,
        transforms: &[TransformPost],
        scale: f32,
    ) {
        use wgpu::util::DeviceExt;

        let Some(tex_a) = self.headless_texture.as_ref() else {
            return;
        };
        let need_recreate = self
            .headless_texture_b
            .as_ref()
            .map(|t| t.size().width != width || t.size().height != height)
            .unwrap_or(true);
        if need_recreate {
            self.headless_texture_b = Some(create_headless_texture(
                &self.device,
                width,
                height,
                self.surface_format,
            ));
        }
        let Some(tex_b) = self.headless_texture_b.as_ref() else {
            return;
        };

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        // 源采样器：Nearest 匹配 CPU apply_transform_post 的 `.round()` 整数采样
        //（Linear 会做双线性插值，与 CPU 逐字逐句的最近邻语义不一致）。
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Transform Src Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        for t in transforms {
            // scissor 钳制到 [0, width]×[0, height]（wgpu 要求非零且在界内）
            let sx = ((t.rect.left() * scale).floor().max(0.0) as u32).min(width);
            let sy = ((t.rect.top() * scale).floor().max(0.0) as u32).min(height);
            let sw = ((t.rect.right() - t.rect.left()).max(0.0) * scale).ceil() as u32;
            let sh = ((t.rect.bottom() - t.rect.top()).max(0.0) * scale).ceil() as u32;
            let sw = sw.min(width.saturating_sub(sx));
            let sh = sh.min(height.saturating_sub(sy));
            if sw == 0 || sh == 0 {
                continue; // 退化 rect，跳过
            }

            let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Transform Filter Encoder"),
            });

            // 1. copy A→B（B 获得完整场景内容，保 rect 外像素不变）
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: tex_a,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: tex_b,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                extent,
            );

            // 2. uniform（64 字节 = 16 f32，布局对齐 TransformUniforms）
            let ox = t.origin_x * scale;
            let oy = t.origin_y * scale;
            let uniform_data: [f32; 16] = [
                width as f32,
                height as f32,
                ox,
                oy,
                t.inv_a,
                t.inv_b,
                t.inv_c,
                t.inv_d,
                t.inv_tx,
                t.inv_ty,
                t.rect.left() * scale,
                t.rect.top() * scale,
                t.rect.right() * scale,
                t.rect.bottom() * scale,
                0.0,
                0.0,
            ];
            let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Transform Uniform Buffer"),
                contents: bytemuck::cast_slice(&uniform_data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
            let uniform_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Transform Uniform BG"),
                layout: &self.transform_uniform_bgl,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });
            let src_view = tex_a.create_view(&wgpu::TextureViewDescriptor::default());
            let src_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Transform Src BG"),
                layout: &self.blur_bgl,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&src_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            });

            // 3. scissor pass: 采样 A → 写 B（rect 内逆映射采样；rect 外白）
            let view_b = tex_b.create_view(&wgpu::TextureViewDescriptor::default());
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Transform Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view_b,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                    multiview_mask: None,
                });
                pass.set_scissor_rect(sx, sy, sw, sh);
                pass.set_pipeline(&self.transform_pipeline);
                pass.set_bind_group(0, &uniform_bg, &[]);
                pass.set_bind_group(1, &src_bg, &[]);
                pass.draw(0..3, 0..1);
            }

            // 4. copy B→A（结果回 A，read_pixels 读 A）
            encoder.copy_texture_to_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: tex_b,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyTextureInfo {
                    texture: tex_a,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                extent,
            );

            self.queue.submit(std::iter::once(encoder.finish()));
        }
    }

    /// DC-9 filter:blur 区域后处理（2-pass H+V 高斯，仅 headless）。
    ///
    /// 对每个 `(rect, radius)`：可分离高斯 = 水平 1D + 垂直 1D 两趟。每趟用 blur_pipeline
    /// ping-pong：copy src→dst（保 rect 外像素）→ scissor pass 采样 src、按方向 blur 写 dst
    ///（rect 内）→ 进入下一趟时 dst 变 src。最终结果在 headless_texture(A)。
    /// uniform = `{screen_w, screen_h, blur_radius, direction}`（direction 0=H, 1=V，与
    /// BLUR_SHADER 对齐）。
    pub(super) fn apply_blur_filters_headless(&mut self, width: u32, height: u32, filters: &[(Rect, f32)], scale: f32) {
        let Some(tex_a) = self.headless_texture.as_ref() else {
            return;
        };
        let need_recreate = self
            .headless_texture_b
            .as_ref()
            .map(|t| t.size().width != width || t.size().height != height)
            .unwrap_or(true);
        if need_recreate {
            self.headless_texture_b = Some(create_headless_texture(
                &self.device,
                width,
                height,
                self.surface_format,
            ));
        }
        let Some(tex_b) = self.headless_texture_b.as_ref() else {
            return;
        };

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        // 克隆 Arc 设备/队列，避免在循环中调用 run_blur_pass（自由函数，借用域分离）时
        // 与 tex_a/tex_b（borrow self.headless_texture/_b）冲突。
        let device = self.device.clone();
        let queue = self.queue.clone();
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Blur Src Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        for &(rect, radius) in filters {
            if radius < 1.0 {
                continue; // 零半径模糊 = no-op
            }
            let sx = ((rect.left() * scale).floor().max(0.0) as u32).min(width);
            let sy = ((rect.top() * scale).floor().max(0.0) as u32).min(height);
            let sw = ((rect.right() - rect.left()).max(0.0) * scale).ceil() as u32;
            let sh = ((rect.bottom() - rect.top()).max(0.0) * scale).ceil() as u32;
            let sw = sw.min(width.saturating_sub(sx));
            let sh = sh.min(height.saturating_sub(sy));
            if sw == 0 || sh == 0 {
                continue;
            }
            let scissor = (sx, sy, sw, sh);

            // Pass 1 水平：copy A→B，blur_pipeline 采样 A（direction=0）写 B（rect 内）。
            // B = H-blurred(rect) + original(outside)。
            run_blur_pass(
                &device,
                &queue,
                &self.blur_pipeline,
                &self.uniform_bgl,
                &self.blur_bgl,
                tex_a,
                tex_b,
                extent,
                &sampler,
                radius,
                0.0,
                scissor,
                "Blur H",
            );
            // Pass 2 垂直：copy B→A，blur_pipeline 采样 B（direction=1）写 A（rect 内）。
            // A = V-blur(H-blurred)(rect) + original(outside) = 2D blur。
            run_blur_pass(
                &device,
                &queue,
                &self.blur_pipeline,
                &self.uniform_bgl,
                &self.blur_bgl,
                tex_b,
                tex_a,
                extent,
                &sampler,
                radius,
                1.0,
                scissor,
                "Blur V",
            );
        }
    }
}
