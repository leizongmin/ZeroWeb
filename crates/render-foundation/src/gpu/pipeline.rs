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
/// 顶点格式：8 个 float = [x, y, u, v, r, g, b, a]（32 字节步幅；P2-8 加 alpha）
/// - pos (x, y): 像素空间坐标
/// - uv (u, v): atlas UV 坐标（填充矩形使用 -1,-1）
/// - color (r, g, b, a): 逐顶点 RGBA 颜色
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
    @location(1) color: vec4f,
};

@vertex
fn vs_main(
    @location(0) pos: vec2f,
    @location(1) uv: vec2f,
    @location(2) color: vec4f,
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
    // P2-8：顶点携带 alpha（color.a），与覆盖率 alpha（glyph atlas 或 1.0）相乘——
    // 半透明填充/glyph/描边不再被画成不透明（旧实现恒 alpha=1.0）
    return vec4f(in.color.rgb, in.color.a * alpha);
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
    @location(0) color: vec4f,
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
    @location(2) color: vec4f,
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

    return in.color;
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

// R3289：repeating 渐变色标周期参数（first = 首色标 offset，period = last - first；
// 非 repeating 时未使用）。纹理经色标重映射（周期铺满 [0,1]），repeating 折叠：
// t_cycle = fract((t - first) / period) 归一化到 [0,1] 采样——与 CPU 折叠等效。
struct GradUniforms {
    first: f32,
    period: f32,
    _pad0: f32,
    _pad1: f32,
};
@group(2) @binding(0) var<uniform> grad_uniforms: GradUniforms;

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
        // CSS Images 4 §4.3.4 角度约定（与 CPU gradient.rs 对齐）：0deg = 正上方
        // 顺时针递增，屏幕 y 向下 → θ = atan2(dx, -dy)。旧 atan2(dy, dx) = 正右
        // 逆时针，差 90°+反向（P2-8 修复）。
        let dx = in.world_pos.x - in.param0;
        let dy = in.world_pos.y - in.param1;
        let angle = atan2(dx, -dy);
        t = fract((angle - in.param2) / 6.283185307179586);
    }
    if (!is_repeating) {
        t = clamp(t, 0.0, 1.0);
    } else {
        // R3289：折叠到 [first, last] 周期并归一化到纹理 [0,1]（纹理已重映射为周期）
        let period = max(grad_uniforms.period, 1e-6);
        t = fract((t - grad_uniforms.first) / period);
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

/// WGSL 着色器 — DC-9 单通道颜色滤镜后处理（opacity / brightness / contrast）。
///
/// 采样源纹理，按 `mode` 应用滤镜（线性空间，sRGB 自动解码→计算→编码）：
/// - mode 0 = opacity(amount)：RGB *= clamp(amount, 0, 1)
/// - mode 1 = brightness(amount)：RGB *= amount（>1 可超过 1，写入时钳制）
/// - mode 2 = contrast(amount)：(RGB - 0.5) * amount + 0.5
///
/// 与 CPU `apply_filter` 对齐（区域后处理，rect 由 render pass scissor 限定）。
/// 独立 WGSL 模块，自含 vs_fullscreen + VertexOutput。
pub const COLOR_FILTER_SHADER: &str = r#"
struct Uniforms {
    screen_width: f32,
    screen_height: f32,
    mode: f32,
    param: f32,
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
fn fs_color_filter(in: VertexOutput) -> @location(0) vec4f {
    let c = textureSample(src_texture, src_sampler, in.uv);
    let p = uniforms.param;
    let m = uniforms.mode;
    // 在线性空间计算（render target 为 sRGB，textureSample 自动解码）。
    // 各 mode 公式与 CPU apply_filter（[0,255] 空间）等价，仅空间不同 →
    // 覆盖达标，非像素精确（同 opacity/brightness/contrast 的 sRGB parity caveat，R279）。
    var out: vec3f;
    if (m < 0.5) {
        // 0 = opacity
        let a = clamp(p, 0.0, 1.0);
        out = c.rgb * a;
    } else if (m < 1.5) {
        // 1 = brightness
        out = c.rgb * p;
    } else if (m < 2.5) {
        // 2 = contrast
        out = (c.rgb - vec3(0.5)) * p + vec3(0.5);
    } else if (m < 3.5) {
        // 3 = grayscale(p)：lerp 向 Rec601 luma（CPU: c + (gray-c)*amt）
        let gray = dot(c.rgb, vec3(0.299, 0.587, 0.114));
        out = mix(c.rgb, vec3(gray), p);
    } else if (m < 4.5) {
        // 4 = hue-rotate(p degrees)：CSS 色相旋转循环矩阵（CPU hue_rotate 同矩阵）
        let angle = radians(p);
        let cos_a = cos(angle);
        let sin_a = sin(angle);
        let sq3 = sqrt(3.0);
        let inv3 = 1.0 / 3.0;
        let ma = cos_a + (1.0 - cos_a) * inv3;
        let mb = (1.0 - cos_a) * inv3 - sq3 * sin_a * inv3;
        let mc = (1.0 - cos_a) * inv3 + sq3 * sin_a * inv3;
        out = vec3(
            ma * c.r + mb * c.g + mc * c.b,
            mc * c.r + ma * c.g + mb * c.b,
            mb * c.r + mc * c.g + ma * c.b,
        );
    } else if (m < 5.5) {
        // 5 = invert(p)：CPU c + (255-2c)*amt ≡ mix(c, 1-c, p)
        out = mix(c.rgb, vec3(1.0) - c.rgb, p);
    } else if (m < 6.5) {
        // 6 = saturate(p)：lerp 从 luma 向原色（CPU gray + (c-gray)*amt ≡ mix(gray, c, p)）
        let gray = dot(c.rgb, vec3(0.299, 0.587, 0.114));
        out = mix(vec3(gray), c.rgb, p);
    } else {
        // 7 = sepia(p)：sepia 矩阵后 lerp（CPU c + (s-c)*amt，s 经 .min(255) 钳制）
        let sr = min(0.393 * c.r + 0.769 * c.g + 0.189 * c.b, 1.0);
        let sg = min(0.349 * c.r + 0.686 * c.g + 0.168 * c.b, 1.0);
        let sb = min(0.272 * c.r + 0.534 * c.g + 0.131 * c.b, 1.0);
        out = mix(c.rgb, vec3(sr, sg, sb), p);
    }
    out = clamp(out, vec3(0.0), vec3(1.0));
    return vec4f(out, 1.0);
}
"#;

// ─── 通用常量 ──────────────────────────────────────────────────

/// Fill/Glyph 顶点步幅（字节）
pub const FILL_VERTEX_STRIDE: u64 = 32;

/// Fill/Glyph 每 vertex float 数量
pub const FILL_FLOATS_PER_VERTEX: usize = 8;

/// Uniform 缓冲区大小（16 字节 = 4 个 f32）
pub const UNIFORM_SIZE: u64 = 16;

/// Transform uniform 缓冲区大小（64 字节 = 16 个 f32，16 字节对齐）。
///
/// 布局（WGSL std140，与 `TRANSFORM_SHADER` 的 `TransformUniforms` 对齐）：
/// screen(vec2) + origin(vec2) + inv_row0(vec4) + inv_row1(vec2) + rect_min(vec2) +
/// rect_max(vec2) + pad(vec2) = 8+8+16+8+8+8+8 = 64。
pub const TRANSFORM_UNIFORM_SIZE: u64 = 64;

/// WGSL 着色器 — DC-9 filter/transform 区域后处理（逆变换重采样）。
///
/// 与 CPU `apply_transform_post` 对齐：对 rect 内每个目标像素，用**预计算逆矩阵**把目标
/// 位置映射回源位置，采样源纹理；逆映射落在 rect 外的位置输出白色（匹配 CPU 的 clear-to-white）。
/// 逆矩阵在 CPU 侧计算后传入（避免 shader 内做 det/除法）。独立 WGSL 模块。
pub const TRANSFORM_SHADER: &str = r#"
struct TransformUniforms {
    screen: vec2f,
    origin: vec2f,
    inv_row0: vec4f,
    inv_row1: vec2f,
    rect_min: vec2f,
    rect_max: vec2f,
    _pad: vec2f,
};

@group(0) @binding(0) var<uniform> u: TransformUniforms;
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
fn fs_transform(in: VertexOutput) -> @location(0) vec4f {
    // 目标像素坐标（uv * screen）
    let dst_px = vec2f(in.uv.x * u.screen.x, in.uv.y * u.screen.y);
    let rel = dst_px - u.origin;
    // 逆变换：src = inv_row0(a,b,c,d) ⊗ rel + inv_row1(tx,ty) + origin
    let src_x = u.inv_row0.x * rel.x + u.inv_row0.z * rel.y + u.inv_row1.x + u.origin.x;
    let src_y = u.inv_row0.y * rel.x + u.inv_row0.w * rel.y + u.inv_row1.y + u.origin.y;
    // textureSample 必须在 uniform control flow 中调用，故无条件采样后用 select
    //（匹配 CPU apply_transform_post：逆映射落在 rect 内取源像素，否则 clear-to-white）。
    let src_uv = vec2f(src_x / u.screen.x, src_y / u.screen.y);
    let sampled = textureSample(src_texture, src_sampler, src_uv);
    let inside = src_x >= u.rect_min.x && src_x < u.rect_max.x
                 && src_y >= u.rect_min.y && src_y < u.rect_max.y;
    return select(vec4f(1.0, 1.0, 1.0, 1.0), sampled, inside);
}
"#;

/// RoundedRect 顶点步幅（字节）— 16 个 float（color 为 rgba）
pub const ROUNDED_RECT_VERTEX_STRIDE: u64 = 64;

/// RoundedRect 每 vertex float 数量
pub const ROUNDED_RECT_FLOATS_PER_VERTEX: usize = 16;

/// Gradient 顶点步幅（字节）— 10 个 float（x, y, world_x, world_y, grad_type+p0, p1, p2, p3 不对...)
/// 实际：[x, y, world_x, world_y, grad_type, p0, p1, p2, p3, _pad] = 10 floats
pub const GRADIENT_VERTEX_STRIDE: u64 = 40;

/// Gradient 每 vertex float 数量
pub const GRADIENT_FLOATS_PER_VERTEX: usize = 10;

/// Image 顶点步幅（字节）— 固定 7 个 float（P2-8 后 fill 布局已 8 float，image 独立）
pub const IMAGE_VERTEX_STRIDE: u64 = 28;

/// Image 每 vertex float 数量
pub const IMAGE_FLOATS_PER_VERTEX: usize = 7;

/// Image 顶点属性（独立于 fill 的 7-float 格式：[x, y, u, v, r, g, b]）
const IMAGE_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
    0 => Float32x2,   // pos [x, y]
    1 => Float32x2,   // uv  [u, v]
    2 => Float32x3,   // color [r, g, b]
];

/// 返回 Image 顶点缓冲区布局
pub fn image_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: IMAGE_VERTEX_STRIDE,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &IMAGE_VERTEX_ATTRIBUTES,
    }
}

/// Blur Uniform 缓冲区大小（16 字节 = 4 个 f32）
pub const BLUR_UNIFORM_SIZE: u64 = 16;

// ─── Fill + Glyph 管线创建 ─────────────────────────────────────

/// Fill+Glyph 顶点属性
const FILL_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 3] = wgpu::vertex_attr_array![
    0 => Float32x2,   // pos [x, y]
    1 => Float32x2,   // uv  [u, v]
    2 => Float32x4,   // color [r, g, b, a]
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
        bind_group_layouts: &[Some(uniform_bgl), Some(atlas_bgl)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Fill Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(fill_vertex_buffer_layout())],
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
        cache: None,
        multiview_mask: None,
    })
}

// ─── RoundedRect 管线创建 ──────────────────────────────────────

/// RoundedRect 顶点属性 — 15 个 float
const ROUNDED_RECT_VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![
    0 => Float32x2,   // pos [x, y]
    1 => Float32x2,   // uv (unused, but kept for layout compatibility)
    2 => Float32x4,   // color [r, g, b, a]
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
        bind_group_layouts: &[Some(uniform_bgl)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Rounded Rect Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(rounded_rect_vertex_buffer_layout())],
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
        cache: None,
        multiview_mask: None,
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
    grad_uniform_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Gradient Shader"),
        source: wgpu::ShaderSource::Wgsl(GRADIENT_SHADER.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Gradient Pipeline Layout"),
        bind_group_layouts: &[Some(uniform_bgl), Some(gradient_bgl), Some(grad_uniform_bgl)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Gradient Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(gradient_vertex_buffer_layout())],
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
        cache: None,
        multiview_mask: None,
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
        bind_group_layouts: &[Some(uniform_bgl), Some(image_bgl)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Image Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[Some(image_vertex_buffer_layout())],
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
        cache: None,
        multiview_mask: None,
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
        bind_group_layouts: &[Some(uniform_bgl), Some(src_bgl)],
        immediate_size: 0,
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
        cache: None,
        multiview_mask: None,
    })
}

/// 创建颜色滤镜后处理管线（DC-9：opacity / brightness / contrast）。
///
/// 结构同 `create_blur_pipeline`：group 0 = uniform（UNIFORM_SIZE=16，4 f32 =
/// {screen_w, screen_h, mode, param}），group 1 = 源纹理+采样器（`create_texture_bind_group_layout`）。
/// entry_point = `fs_color_filter`，按 uniform.mode 分派滤镜；区域由 pass scissor 限定。
pub fn create_color_filter_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bgl: &wgpu::BindGroupLayout,
    src_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Color Filter Shader"),
        source: wgpu::ShaderSource::Wgsl(COLOR_FILTER_SHADER.into()),
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Color Filter Pipeline Layout"),
        bind_group_layouts: &[Some(uniform_bgl), Some(src_bgl)],
        immediate_size: 0,
    });

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Color Filter Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_color_filter"),
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
        cache: None,
        multiview_mask: None,
    })
}

/// 创建 transform uniform 绑定组布局（group 0，64 字节 uniform，DC-9 transform）。
pub fn create_transform_uniform_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Transform Uniform BGL"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: Some(NonZeroU64::new(TRANSFORM_UNIFORM_SIZE).unwrap()),
            },
            count: None,
        }],
    })
}

/// 创建 transform 后处理管线（DC-9）。
///
/// group 0 = transform uniform（TRANSFORM_UNIFORM_SIZE=64，预计算逆矩阵+origin+rect 边界），
/// group 1 = 源纹理+采样器（`create_texture_bind_group_layout`）。entry_point = `fs_transform`，
/// 逆变换重采样，区域由 pass scissor 限定。
pub fn create_transform_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    transform_uniform_bgl: &wgpu::BindGroupLayout,
    src_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Transform Shader"),
        source: wgpu::ShaderSource::Wgsl(TRANSFORM_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Transform Pipeline Layout"),
        bind_group_layouts: &[Some(transform_uniform_bgl), Some(src_bgl)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Transform Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_transform"),
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
        cache: None,
        multiview_mask: None,
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

// ─── Blend 管线（C/R3278：mix-blend-mode 双 pass 合成）──────────────────

/// WGSL 着色器 — blend 合成（CSS Compositing-1 §5.1 16 模式）。
/// 全屏 quad（scissor 限制 blend 区域），采样 source（元素层）与 backdrop（主帧拷贝），
/// 按 uniform.mode 应用公式输出。B(Cb, Cs)：Cb=backdrop（背景），Cs=source（元素）。
pub const BLEND_SHADER: &str = r#"
struct BlendUniforms {
    mode: f32,
    screen_w: f32,
    screen_h: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> uniforms: BlendUniforms;
@group(1) @binding(0) var source_tex: texture_2d<f32>;
@group(1) @binding(1) var backdrop_tex: texture_2d<f32>;
@group(1) @binding(2) var samp: sampler;

@vertex
fn vs_fullscreen(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4f {
    var pos = array<vec2f, 3>(vec2f(-1.0, -1.0), vec2f(3.0, -1.0), vec2f(-1.0, 3.0));
    return vec4f(pos[vi], 0.0, 1.0);
}

fn lum(c: vec3f) -> f32 {
    return 0.3 * c.r + 0.59 * c.g + 0.11 * c.b;
}

fn set_lum(c: vec3f, l: f32) -> vec3f {
    let d = l - lum(c);
    return clamp(c + vec3f(d), vec3f(0.0), vec3f(1.0));
}

fn sat(c: vec3f) -> f32 {
    return max(max(c.r, c.g), c.b) - min(min(c.r, c.g), c.b);
}

fn set_sat(c: vec3f, s: f32) -> vec3f {
    let mn = min(min(c.r, c.g), c.b);
    let mx = max(max(c.r, c.g), c.b);
    if (mx > mn) {
        let mid = c.r + c.g + c.b - mn - mx;
        let mid2 = (mid - mn) * s / (mx - mn);
        return vec3f(
            select(mid2, 0.0, c.r == mn),
            select(mid2, 0.0, c.g == mn),
            select(mid2, 0.0, c.b == mn),
        ) + vec3f(select(0.0, s, c.r == mx), select(0.0, s, c.g == mx), select(0.0, s, c.b == mx));
    }
    return c;
}

fn hard_light(cb: vec3f, cs: vec3f) -> vec3f {
    return select(cb * cs * 2.0, 1.0 - 2.0 * (1.0 - cb) * (1.0 - cs), cs > vec3f(0.5));
}

fn soft_light(cb: f32, cs: f32) -> f32 {
    if (cs <= 0.5) {
        return cb - (1.0 - 2.0 * cs) * cb * (1.0 - cb);
    }
    let d = select(sqrt(cb), ((16.0 * cb - 12.0) * cb + 4.0) * cb, cb <= 0.25);
    return cb + (2.0 * cs - 1.0) * (d - cb);
}

fn blend_channel(cb: f32, cs: f32, m: i32) -> f32 {
    if (m == 1) { return cb * cs; }                        // multiply
    if (m == 2) { return 1.0 - (1.0 - cb) * (1.0 - cs); }  // screen
    if (m == 4) { return min(cb, cs); }                    // darken
    if (m == 5) { return max(cb, cs); }                    // lighten
    if (m == 6) { return select(1.0, cb / (1.0 - cs), cs < 1.0); }  // color-dodge
    if (m == 7) { return select(0.0, 1.0 - (1.0 - cb) / cs, cs > 0.0); }  // color-burn
    if (m == 10) { return abs(cb - cs); }                  // difference
    if (m == 11) { return cb + cs - 2.0 * cb * cs; }       // exclusion
    return cb; // 其余通道级模式由 blend() 处理
}

fn blend(cb: vec3f, cs: vec3f, m: i32) -> vec3f {
    if (m == 0) { return cs; }                             // normal
    if (m == 3) { return hard_light(cs, cb); }             // overlay
    if (m == 8) { return hard_light(cb, cs); }             // hard-light
    if (m == 9) {                                          // soft-light
        return vec3f(soft_light(cb.r, cs.r), soft_light(cb.g, cs.g), soft_light(cb.b, cs.b));
    }
    if (m == 12) { return set_lum(set_sat(cb, sat(cs)), lum(cb)); }  // hue
    if (m == 13) { return set_lum(set_sat(cs, sat(cb)), lum(cb)); }  // saturation
    if (m == 14) { return set_lum(cs, lum(cb)); }          // color
    if (m == 15) { return set_lum(cb, lum(cs)); }          // luminosity
    return vec3f(blend_channel(cb.r, cs.r, m), blend_channel(cb.g, cs.g, m), blend_channel(cb.b, cs.b, m));
}

@fragment
fn fs_blend(@builtin(position) pos: vec4f) -> @location(0) vec4f {
    let uv = vec2f(pos.x / uniforms.screen_w, pos.y / uniforms.screen_h);
    let cs = textureSample(source_tex, samp, uv).rgb;
    let cb = textureSample(backdrop_tex, samp, uv).rgb;
    let mixed = blend(cb, cs, i32(uniforms.mode));
    // alpha：src-over 合成（CSS §5.2）
    let sa = textureSample(source_tex, samp, uv).a;
    let da = textureSample(backdrop_tex, samp, uv).a;
    return vec4f(mixed, sa + da * (1.0 - sa));
}
"#;

/// Blend 管线绑定组布局（uniform + source/backdrop 纹理 + 采样器）。
pub fn create_blend_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Blend BG Layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

/// 创建 Blend 合成管线（全屏 pass，无顶点缓冲）。
pub fn create_blend_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    uniform_bgl: &wgpu::BindGroupLayout,
    blend_bgl: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Blend Shader"),
        source: wgpu::ShaderSource::Wgsl(BLEND_SHADER.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Blend Pipeline Layout"),
        bind_group_layouts: &[Some(uniform_bgl), Some(blend_bgl)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Blend Pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader_module,
            entry_point: Some("vs_fullscreen"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader_module,
            entry_point: Some("fs_blend"),
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
        cache: None,
        multiview_mask: None,
    })
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_sources_not_empty() {
        assert!(FILL_GLYPH_SHADER.contains("vs_main"));
        assert!(FILL_GLYPH_SHADER.contains("fs_main"));
        assert!(ROUNDED_RECT_SHADER.contains("vs_main"));
        assert!(GRADIENT_SHADER.contains("vs_main"));
        assert!(IMAGE_SHADER.contains("vs_main"));
        // BLUR_SHADER 用全屏三角形 pass，vertex entry point 命名为 vs_fullscreen（非 vs_main）
        assert!(BLUR_SHADER.contains("vs_fullscreen"));
        assert!(BLUR_SHADER.contains("fs_blur"));
    }

    #[test]
    fn test_vertex_constants() {
        assert_eq!(FILL_FLOATS_PER_VERTEX, 8);
        assert_eq!(FILL_VERTEX_STRIDE, 32);
        assert_eq!(UNIFORM_SIZE, 16);
        assert_eq!(ROUNDED_RECT_FLOATS_PER_VERTEX, 16);
        assert_eq!(ROUNDED_RECT_VERTEX_STRIDE, 64);
        assert_eq!(GRADIENT_FLOATS_PER_VERTEX, 10);
        assert_eq!(GRADIENT_VERTEX_STRIDE, 40);
    }
}
