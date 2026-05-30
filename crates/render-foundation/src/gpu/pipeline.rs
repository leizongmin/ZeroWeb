//! GPU 渲染管线 — WGSL 着色器和 wgpu 管线配置
//!
//! 基于 OmniTerm 的统一管线设计：
//! - 单管线同时处理填充矩形和 glyph 渲染
//! - 填充矩形使用 UV = (-1, -1) 作为标记，片段着色器中 alpha = 1.0
//! - Glyph 使用真实的 atlas UV 坐标采样 R8Unorm 纹理获取 alpha 遮罩

/// WGSL 着色器 — 统一处理填充矩形和 glyph 文本
///
/// 顶点格式：7 个 float = [x, y, u, v, r, g, b]（28 字节步幅）
/// - pos (x, y): 像素空间坐标
/// - uv (u, v): atlas UV 坐标（填充矩形使用 -1,-1）
/// - color (r, g, b): 逐顶点 RGB 颜色
pub const GLYPH_SHADER: &str = r#"
struct Uniforms {
    screen_width: f32,
    screen_height: f32,
    atlas_size: f32,
    _padding: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var atlas_texture: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
    @location(1) color: vec3f,
};

@vertex
fn vs_main(
    @location(0) pos: vec2f,
    @location(1) uv: vec2f,
    @location(2) color: vec3f,
) -> VertexOutput {
    // 像素坐标 → NDC 裁剪空间
    let x = (pos.x / uniforms.screen_width) * 2.0 - 1.0;
    let y = 1.0 - (pos.y / uniforms.screen_height) * 2.0;
    var out: VertexOutput;
    out.position = vec4f(x, y, 0.0, 1.0);
    out.uv = uv;
    out.color = color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var alpha: f32;
    if (in.uv.x < 0.0) {
        // 填充矩形：UV = (-1,-1) 表示纯色填充
        alpha = 1.0;
    } else {
        // Glyph：从 atlas 纹理采样 R 通道作为 alpha 遮罩
        alpha = textureSample(atlas_texture, atlas_sampler, in.uv).r;
    }
    return vec4f(in.color, alpha);
}
"#;

/// 顶点数据格式 — 7 个 float（28 字节）
///
/// ```text
/// offset 0:  x (f32)     — 像素空间 X
/// offset 4:  y (f32)     — 像素空间 Y
/// offset 8:  u (f32)     — atlas U（或 -1.0 表示填充）
/// offset 12: v (f32)     — atlas V（或 -1.0 表示填充）
/// offset 16: r (f32)     — 红色分量 [0,1]
/// offset 20: g (f32)     — 绿色分量 [0,1]
/// offset 24: b (f32)     — 蓝色分量 [0,1]
/// ```
pub const VERTEX_FLOATS_PER_VERTEX: usize = 7;

/// 顶点步幅（字节）
pub const VERTEX_STRIDE: u64 = 28;

/// Uniform 缓冲区大小（16 字节 = 4 个 f32）
pub const UNIFORM_SIZE: u64 = 16;

/// 顶点属性数组（const 以获取 'static 生命周期）
const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
    0 => Float32x2,   // pos [x, y]
    1 => Float32x2,   // uv  [u, v]
    2 => Float32x3,   // color [r, g, b]
];

/// 返回 wgpu 顶点缓冲区布局
pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: VERTEX_STRIDE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &VERTEX_ATTRIBUTES,
    }
}

/// 创建渲染管线（需要在 async 上下文中调用，因为 request_device 是异步的）
pub fn create_render_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    atlas_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Glyph Shader"),
        source: wgpu::ShaderSource::Wgsl(GLYPH_SHADER.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render Pipeline Layout"),
        bind_group_layouts: &[uniform_bind_group_layout, atlas_bind_group_layout],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[vertex_buffer_layout()],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
        cache: None,
    })
}

/// 创建 uniform 绑定组布局（group 0）
pub fn create_uniform_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Uniform Bind Group Layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(std::num::NonZeroU64::new(UNIFORM_SIZE).unwrap()),
            },
            count: None,
        }],
    })
}

/// 创建 atlas 绑定组布局（group 1）
pub fn create_atlas_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Atlas Bind Group Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_source_not_empty() {
        assert!(!GLYPH_SHADER.is_empty());
        assert!(GLYPH_SHADER.contains("vs_main"));
        assert!(GLYPH_SHADER.contains("fs_main"));
    }

    #[test]
    fn test_vertex_constants() {
        assert_eq!(VERTEX_FLOATS_PER_VERTEX, 7);
        assert_eq!(VERTEX_STRIDE, 28);
        assert_eq!(UNIFORM_SIZE, 16);
    }
}
