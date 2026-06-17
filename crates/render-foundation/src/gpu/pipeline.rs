//! GPU 渲染管线 — WGSL 着色器和 wgpu 管线配置
//!
//! 管线架构：
//! - **Fill+Glyph 管线**（现有）：处理填充矩形和 glyph 文本
//! - **RoundedRect 管线**：处理圆角矩形的 GPU 片段着色器渲染
//! - **Gradient 管线**：处理线性/径向/锥形渐变
//! - **Image 管线**：处理图片纹理采样
//! - **Blur 管线**：处理阴影模糊和滤镜模糊的后处理 pass

use std::num::NonZeroU64;

// ─── Fill + Glyph 管线 ────────────────────────────────────────

/// WGSL 着色器 — 统一处理填充矩形和 glyph 文本
///
/// 顶点格式：7 个 float = [x, y, u, v, r, g, b]（28 字节步幅）
/// - pos (x, y): 像素空间坐标
/// - uv (u, v): atlas UV 坐标（填充矩形使用 -1,-1）
/// - color (r, g, b): 逐顶点 RGB 颜色
pub const FILL_GLYPH_SHADER: &str = r#"
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
        alpha = 1.0;
    } else {
        alpha = textureSample(atlas_texture, atlas_sampler, in.uv).r;
    }
    return vec4f(in.color, alpha);
}
"#;

// ─── RoundedRect 管线 ──────────────────────────────────────────

/// WGSL 着色器 — 圆角矩形渲染
///
/// 顶点格式：15 个 float = [x, y, u, v, r, g, b, rect_l, rect_t, rect_r, rect_b, tl_r, tr_r, br_r, bl_r]
/// 片段着色器对每个像素做圆角检测：在四个角的扇形区域内，计算像素到角中心的距离，
/// 若距离大于半径则 discard。
pub const ROUNDED_RECT_SHADER: &str = r#"
struct Uniforms {
    screen_width: f32,
    screen_height: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) color: vec3f,
    @location(1) world_pos: vec2f,
    @location(2) rect_lt: vec2f,
    @location(3) rect_rb: vec2f,
    @location(4) radii_tl_tr: vec2f,
    @location(5) radii_br_bl: vec2f,
};

@vertex
fn vs_main(
    @location(0) pos: vec2f,
    @location(1) uv: vec2f,  // unused for rounded rect
    @location(2) color: vec3f,
    @location(3) rect_lt: vec2f,
    @location(4) rect_rb: vec2f,
    @location(5) radii_tl_tr: vec2f,
    @location(6) radii_br_bl: vec2f,
) -> VertexOutput {
    let x = (pos.x / uniforms.screen_width) * 2.0 - 1.0;
    let y = 1.0 - (pos.y / uniforms.screen_height) * 2.0;
    var out: VertexOutput;
    out.position = vec4f(x, y, 0.0, 1.0);
    out.color = color;
    out.world_pos = pos;
    out.rect_lt = rect_lt;
    out.rect_rb = rect_rb;
    out.radii_tl_tr = radii_tl_tr;
    out.radii_br_bl = radii_br_bl;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    let px = in.world_pos.x;
    let py = in.world_pos.y;
    let l = in.rect_lt.x;
    let t = in.rect_lt.y;
    let r = in.rect_rb.x;
    let b = in.rect_rb.y;
    let tl_r = in.radii_tl_tr.x;
    let tr_r = in.radii_tl_tr.y;
    let br_r = in.radii_br_bl.x;
    let bl_r = in.radii_br_bl.y;

    // 判断像素是否在某个角的扇形区域内
    // 左上角
    let tl_cx = l + tl_r;
    let tl_cy = t + tl_r;
    if (px < tl_cx && py < tl_cy && tl_r > 0.0) {
        let dx = px - tl_cx;
        let dy = py - tl_cy;
        if (dx * dx + dy * dy > tl_r * tl_r) {
            discard;
        }
    }
    // 右上角
    let tr_cx = r - tr_r;
    let tr_cy = t + tr_r;
    if (px > tr_cx && py < tr_cy && tr_r > 0.0) {
        let dx = px - tr_cx;
        let dy = py - tr_cy;
        if (dx * dx + dy * dy > tr_r * tr_r) {
            discard;
        }
    }
    // 右下角
    let br_cx = r - br_r;
    let br_cy = b - br_r;
    if (px > br_cx && py > br_cy && br_r > 0.0) {
        let dx = px - br_cx;
        let dy = py - br_cy;
        if (dx * dx + dy * dy > br_r * br_r) {
            discard;
        }
    }
    // 左下角
    let bl_cx = l + bl_r;
    let bl_cy = b - bl_r;
    if (px < bl_cx && py > bl_cy && bl_r > 0.0) {
        let dx = px - bl_cx;
        let dy = py - bl_cy;
        if (dx * dx + dy * dy > bl_r * bl_r) {
            discard;
        }
    }

    return vec4f(in.color, 1.0);
}
"#;

// ─── Gradient 管线 ─────────────────────────────────────────────

/// WGSL 着色器 — 渐变渲染（线性/径向/锥形）
///
/// 使用 1D 渐变纹理存储色标。顶点格式：
/// [x, y, u, v, r, g, b, grad_type, param0, param1, param2, param3]
/// - grad_type: 0=linear, 1=radial, 2=conic
/// - param0-3: 渐变参数
pub const GRADIENT_SHADER: &str = r#"
struct Uniforms {
    screen_width: f32,
    screen_height: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var grad_texture: texture_2d<f32>;
@group(1) @binding(1) var grad_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) world_pos: vec2f,
    @location(1) grad_type: f32,
    @location(2) param0: f32,
    @location(3) param1: f32,
    @location(4) param2: f32,
    @location(5) param3: f32,
};

@vertex
fn vs_main(
    @location(0) pos: vec2f,
    @location(1) world_pos: vec2f,
    @location(2) grad_type_and_p0: vec2f,
    @location(3) param12: vec2f,
    @location(4) param34: vec2f,
) -> VertexOutput {
    let x = (pos.x / uniforms.screen_width) * 2.0 - 1.0;
    let y = 1.0 - (pos.y / uniforms.screen_height) * 2.0;
    var out: VertexOutput;
    out.position = vec4f(x, y, 0.0, 1.0);
    out.world_pos = world_pos;
    out.grad_type = grad_type_and_p0.x;
    out.param0 = grad_type_and_p0.y;
    out.param1 = param12.x;
    out.param2 = param12.y;
    out.param3 = param34.x;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    var t: f32;
    let is_repeating = in.param3 < -0.5;
    if (in.grad_type < 0.5) {
        // 线性渐变: param0=x0, param1=y0, param2=x1, param3=packed(repeating,y1)
        let dx = in.world_pos.x - in.param0;
        let dy = in.world_pos.y - in.param1;
        let lx = in.param2 - in.param0;
        let ly = abs(in.param3) - in.param1;
        let len2 = lx * lx + ly * ly;
        if (len2 > 0.0) {
            t = (dx * lx + dy * ly) / len2;
        } else {
            t = 0.0;
        }
    } else if (in.grad_type < 1.5) {
        // 径向渐变: param0=cx, param1=cy, param2=inner_r, param3=packed(repeating,outer_r)
        let dx = in.world_pos.x - in.param0;
        let dy = in.world_pos.y - in.param1;
        let dist = sqrt(dx * dx + dy * dy);
        let inner_r = in.param2;
        let outer_r = abs(in.param3);
        let range = outer_r - inner_r;
        if (range > 0.0) {
            t = (dist - inner_r) / range;
        } else {
            t = 0.0;
        }
    } else {
        // 锥形渐变: param0=cx, param1=cy, param2=start_angle
        let dx = in.world_pos.x - in.param0;
        let dy = in.world_pos.y - in.param1;
        let angle = atan2(dy, dx);
        t = fract((angle - in.param2) / 6.283185307179586);
    }
    if (!is_repeating) {
        t = clamp(t, 0.0, 1.0);
    } else {
        t = fract(t);
    }
    return textureSample(grad_texture, grad_sampler, vec2f(t, 0.5));
}
"#;

// ─── Image 管线 ────────────────────────────────────────────────

/// WGSL 着色器 — 图片纹理采样
///
/// 顶点格式与 fill 相同：7 个 float = [x, y, u, v, r, g, b]
/// u/v 存储图片纹理坐标 [0,1]。
pub const IMAGE_SHADER: &str = r#"
struct Uniforms {
    screen_width: f32,
    screen_height: f32,
    _pad0: f32,
    _pad1: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var image_texture: texture_2d<f32>;
@group(1) @binding(1) var image_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_main(
    @location(0) pos: vec2f,
    @location(1) uv: vec2f,
    @location(2) color: vec3f,
) -> VertexOutput {
    let x = (pos.x / uniforms.screen_width) * 2.0 - 1.0;
    let y = 1.0 - (pos.y / uniforms.screen_height) * 2.0;
    var out: VertexOutput;
    out.position = vec4f(x, y, 0.0, 1.0);
    out.uv = uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return textureSample(image_texture, image_sampler, in.uv);
}
"#;

// ─── Blur 管线（后处理） ───────────────────────────────────────

/// WGSL 着色器 — 分离式高斯模糊（单次 box-blur pass）
///
/// 输入：源纹理 + 模糊方向参数
/// uniform: [screen_width, screen_height, blur_radius, direction(0=horizontal,1=vertical)]
pub const BLUR_SHADER: &str = r#"
struct Uniforms {
    screen_width: f32,
    screen_height: f32,
    blur_radius: f32,
    direction: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var src_texture: texture_2d<f32>;
@group(1) @binding(1) var src_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_fullscreen(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    // 全屏四边形：3 个顶点覆盖整个 NDC 空间
    var pos = array<vec2f, 3>(
        vec2f(-1.0, -1.0),
        vec2f(3.0, -1.0),
        vec2f(-1.0, 3.0),
    );
    var uv = array<vec2f, 3>(
        vec2f(0.0, 1.0),
        vec2f(2.0, 1.0),
        vec2f(0.0, -1.0),
    );
    var out: VertexOutput;
    out.position = vec4f(pos[vertex_index], 0.0, 1.0);
    out.uv = uv[vertex_index];
    return out;
}

@fragment
fn fs_blur(in: VertexOutput) -> @location(0) vec4f {
    let radius = max(uniforms.blur_radius, 1.0);
    let dir = select(
        vec2f(1.0 / uniforms.screen_width, 0.0),
        vec2f(0.0, 1.0 / uniforms.screen_height),
        uniforms.direction > 0.5,
    );
    let r = i32(radius);
    var color = vec4f(0.0);
    var total_weight: f32 = 0.0;
    for (var i: i32 = -r; i <= r; i++) {
        let weight = 1.0 - abs(f32(i)) / (radius + 1.0);
        let offset = dir * f32(i);
        color += textureSample(src_texture, src_sampler, in.uv + offset) * weight;
        total_weight += weight;
    }
    return color / total_weight;
}
"#;

/// WGSL 着色器 — DC-9 filter:opacity 后处理（区域 RGB 乘）。
///
/// 与 CPU `apply_filter` 的 `Opacity(amount)` 对齐：采样源纹理，RGB *= amount
///（framebuffer alpha 恒 255）。区域由 render pass 的 scissor rect 限定（仅影响
/// filter.rect 内像素）。独立 WGSL 模块，自含 vs_fullscreen + VertexOutput。
pub const OPACITY_SHADER: &str = r#"
struct Uniforms {
    screen_width: f32,
    screen_height: f32,
    amount: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var src_texture: texture_2d<f32>;
@group(1) @binding(1) var src_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4f,
    @location(0) uv: vec2f,
};

@vertex
fn vs_fullscreen(
    @builtin(vertex_index) vertex_index: u32,
) -> VertexOutput {
    // 全屏三角形：3 个顶点覆盖整个 NDC 空间
    var pos = array<vec2f, 3>(
        vec2f(-1.0, -1.0),
        vec2f(3.0, -1.0),
        vec2f(-1.0, 3.0),
    );
    var uv = array<vec2f, 3>(
        vec2f(0.0, 1.0),
        vec2f(2.0, 1.0),
        vec2f(0.0, -1.0),
    );
    var out: VertexOutput;
    out.position = vec4f(pos[vertex_index], 0.0, 1.0);
    out.uv = uv[vertex_index];
    return out;
}

@fragment
fn fs_opacity(in: VertexOutput) -> @location(0) vec4f {
    let c = textureSample(src_texture, src_sampler, in.uv);
    let a = max(0.0, min(1.0, uniforms.amount));
    return vec4f(c.r * a, c.g * a, c.b * a, 1.0);
}
"#;

// ─── 通用常量 ──────────────────────────────────────────────────

/// Fill/Glyph 顶点步幅（字节）
pub const FILL_VERTEX_STRIDE: u64 = 28;

/// Fill/Glyph 每 vertex float 数量
pub const FILL_FLOATS_PER_VERTEX: usize = 7;

/// Uniform 缓冲区大小（16 字节 = 4 个 f32）
pub const UNIFORM_SIZE: u64 = 16;

/// RoundedRect 顶点步幅（字节）— 15 个 float
pub const ROUNDED_RECT_VERTEX_STRIDE: u64 = 60;

/// RoundedRect 每 vertex float 数量
pub const ROUNDED_RECT_FLOATS_PER_VERTEX: usize = 15;

/// Gradient 顶点步幅（字节）— 10 个 float（x, y, world_x, world_y, grad_type+p0, p1, p2, p3 不对...)
/// 实际：[x, y, world_x, world_y, grad_type, p0, p1, p2, p3, _pad] = 10 floats
pub const GRADIENT_VERTEX_STRIDE: u64 = 40;

/// Gradient 每 vertex float 数量
pub const GRADIENT_FLOATS_PER_VERTEX: usize = 10;

/// Image 顶点步幅（字节）— 与 fill 相同 7 个 float
pub const IMAGE_VERTEX_STRIDE: u64 = 28;

/// Image 每 vertex float 数量
pub const IMAGE_FLOATS_PER_VERTEX: usize = 7;

/// Blur Uniform 缓冲区大小（16 字节 = 4 个 f32）
pub const BLUR_UNIFORM_SIZE: u64 = 16;

// ─── Fill + Glyph 管线创建 ─────────────────────────────────────

/// Fill+Glyph 顶点属性
const FILL_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
    0 => Float32x2,   // pos [x, y]
    1 => Float32x2,   // uv  [u, v]
    2 => Float32x3,   // color [r, g, b]
];

/// 返回 Fill+Glyph 顶点缓冲区布局
pub fn fill_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: FILL_VERTEX_STRIDE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &FILL_VERTEX_ATTRIBUTES,
    }
}

/// 创建 Fill+Glyph 渲染管线
pub fn create_fill_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bgl: &wgpu::BindGroupLayout,
    atlas_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Fill+Glyph Shader"),
        source: wgpu::ShaderSource::Wgsl(FILL_GLYPH_SHADER.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Fill Pipeline Layout"),
        bind_group_layouts: &[uniform_bgl, atlas_bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Fill Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[fill_vertex_buffer_layout()],
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

// ─── RoundedRect 管线创建 ──────────────────────────────────────

/// RoundedRect 顶点属性 — 15 个 float
const ROUNDED_RECT_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
    0 => Float32x2,   // pos [x, y]
    1 => Float32x2,   // uv (unused, but kept for layout compatibility)
    2 => Float32x3,   // color [r, g, b]
    3 => Float32x2,   // rect_lt [left, top]
    4 => Float32x2,   // rect_rb [right, bottom]
    5 => Float32x2,   // radii_tl_tr
    6 => Float32x2,   // radii_br_bl
];

/// 返回 RoundedRect 顶点缓冲区布局
pub fn rounded_rect_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: ROUNDED_RECT_VERTEX_STRIDE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &ROUNDED_RECT_VERTEX_ATTRIBUTES,
    }
}

/// 创建 RoundedRect 渲染管线
pub fn create_rounded_rect_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Rounded Rect Shader"),
        source: wgpu::ShaderSource::Wgsl(ROUNDED_RECT_SHADER.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Rounded Rect Pipeline Layout"),
        bind_group_layouts: &[uniform_bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Rounded Rect Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[rounded_rect_vertex_buffer_layout()],
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

// ─── Gradient 管线创建 ─────────────────────────────────────────

/// Gradient 顶点属性 — 10 个 float
const GRADIENT_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
    0 => Float32x2,   // pos [x, y]
    1 => Float32x2,   // world_pos [world_x, world_y]
    2 => Float32x2,   // grad_type + param0
    3 => Float32x2,   // param1 + param2
    4 => Float32x2,   // param3 + _padding
];

/// 返回 Gradient 顶点缓冲区布局
pub fn gradient_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: GRADIENT_VERTEX_STRIDE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &GRADIENT_VERTEX_ATTRIBUTES,
    }
}

/// 创建 Gradient 渲染管线
pub fn create_gradient_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bgl: &wgpu::BindGroupLayout,
    gradient_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Gradient Shader"),
        source: wgpu::ShaderSource::Wgsl(GRADIENT_SHADER.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Gradient Pipeline Layout"),
        bind_group_layouts: &[uniform_bgl, gradient_bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Gradient Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[gradient_vertex_buffer_layout()],
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

// ─── Image 管线创建 ────────────────────────────────────────────

/// 创建 Image 渲染管线
pub fn create_image_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bgl: &wgpu::BindGroupLayout,
    image_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Image Shader"),
        source: wgpu::ShaderSource::Wgsl(IMAGE_SHADER.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Image Pipeline Layout"),
        bind_group_layouts: &[uniform_bgl, image_bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Image Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[fill_vertex_buffer_layout()], // same layout as fill
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

// ─── Blur 管线创建 ─────────────────────────────────────────────

/// 创建 Blur 后处理管线
pub fn create_blur_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bgl: &wgpu::BindGroupLayout,
    src_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Blur Shader"),
        source: wgpu::ShaderSource::Wgsl(BLUR_SHADER.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Blur Pipeline Layout"),
        bind_group_layouts: &[uniform_bgl, src_bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Blur Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_blur"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
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

/// 创建 filter:opacity 后处理管线（DC-9）。
///
/// 结构同 `create_blur_pipeline`：group 0 = uniform（UNIFORM_SIZE=16，4 f32），
/// group 1 = 源纹理+采样器（`create_texture_bind_group_layout`）。entry_point =
/// `fs_opacity`（区域 RGB 乘，区域由 pass scissor 限定）。
pub fn create_opacity_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bgl: &wgpu::BindGroupLayout,
    src_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Opacity Filter Shader"),
        source: wgpu::ShaderSource::Wgsl(OPACITY_SHADER.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Opacity Pipeline Layout"),
        bind_group_layouts: &[uniform_bgl, src_bgl],
        push_constant_ranges: &[],
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Opacity Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_opacity"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent::REPLACE,
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
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

// ─── 通用 Bind Group Layout 创建 ───────────────────────────────

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
                min_binding_size: Some(NonZeroU64::new(UNIFORM_SIZE).unwrap()),
            },
            count: None,
        }],
    })
}

/// 创建 atlas 绑定组布局（group 1）— 用于 Fill+Glyph 管线
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

/// 创建通用纹理+采样器绑定组布局（用于 Gradient/Image/Blur）
pub fn create_texture_bind_group_layout(device: &wgpu::Device, label: &str) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some(label),
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

// ─── 兼容旧 API ────────────────────────────────────────────────

/// 旧名：WGSL 着色器（兼容）
#[allow(dead_code)]
pub const GLYPH_SHADER: &str = FILL_GLYPH_SHADER;

/// 旧名：每 vertex float 数量（兼容）
pub const VERTEX_FLOATS_PER_VERTEX: usize = FILL_FLOATS_PER_VERTEX;

/// 旧名：顶点步幅（兼容）
pub const VERTEX_STRIDE: u64 = FILL_VERTEX_STRIDE;

/// 旧名：顶点属性数组（兼容）
#[allow(dead_code)]
const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] = FILL_VERTEX_ATTRIBUTES;

/// 旧名：顶点缓冲区布局（兼容）
pub fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    fill_vertex_buffer_layout()
}

/// 旧名：创建渲染管线（兼容）
pub fn create_render_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bind_group_layout: &wgpu::BindGroupLayout,
    atlas_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    create_fill_pipeline(device, format, uniform_bind_group_layout, atlas_bind_group_layout)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_sources_not_empty() {
        assert!(!FILL_GLYPH_SHADER.is_empty());
        assert!(FILL_GLYPH_SHADER.contains("vs_main"));
        assert!(FILL_GLYPH_SHADER.contains("fs_main"));
        assert!(!ROUNDED_RECT_SHADER.is_empty());
        assert!(!GRADIENT_SHADER.is_empty());
        assert!(!IMAGE_SHADER.is_empty());
        assert!(!BLUR_SHADER.is_empty());
    }

    #[test]
    fn test_vertex_constants() {
        assert_eq!(FILL_FLOATS_PER_VERTEX, 7);
        assert_eq!(FILL_VERTEX_STRIDE, 28);
        assert_eq!(UNIFORM_SIZE, 16);
        assert_eq!(ROUNDED_RECT_FLOATS_PER_VERTEX, 15);
        assert_eq!(ROUNDED_RECT_VERTEX_STRIDE, 60);
        assert_eq!(GRADIENT_FLOATS_PER_VERTEX, 10);
        assert_eq!(GRADIENT_VERTEX_STRIDE, 40);
    }
}
