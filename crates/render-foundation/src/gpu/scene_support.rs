//! GPU 场景支持检测 — 判定当前帧能否被 GPU 渲染器正确绘制。
//!
//! GPU 生产路径（`render_full_scene_gpu`）对部分特性未实现或静默降级：不检测时
//! 会「静默画错」（测试通过但用户看到错误像素）。调用方（浏览器本地渲染 /
//! 合成器 GPU 光栅）在返回 `false` 时回退 CPU 整帧重画——慢但对。
//!
//! 基线：docs/learnings/bugs/cpu-gpu-path-divergence.md（P0-1 回退项）。

use crate::gpu::renderer::GlyphDraw;
use crate::primitive::{FillPrimitive, RenderPrimitives, RoundedRectPrimitive};

/// GPU 能否正确渲染本帧；`false` 时调用方应回退 CPU 渲染。
///
/// 检测 GPU 生产路径未实现、会静默丢弃或画错的特性：
/// - `clips` / `blend_modes`：`render_full_scene_gpu` 完全不消费
/// - 半透明颜色：fill/rounded-rect/stroke/path/glyph shader 输出 alpha 恒 1
///   （顶点仅 RGB，`gpu/mesh.rs`；blend 开启但 shader alpha 恒 1）
/// - 带模糊/spread/inset 的阴影：GPU 只画硬边不透明矩形
/// - 窗口模式（`headless=false`）下的 filter/transform：后处理仅 headless 生效
///
/// 不受影响、无需回退：渐变（纹理 RGBA，shader 直接输出 alpha）、图片
/// （线性采样，alpha 保留）、无模糊/无 spread/不透明硬边阴影（GPU 与 CPU 一致）。
pub fn scene_supported(
    primitives: &RenderPrimitives,
    ui_glyphs: &[GlyphDraw],
    overlay_fills: &[FillPrimitive],
    overlay_glyphs: &[GlyphDraw],
    overlay_rounded_rects: &[RoundedRectPrimitive],
    headless: bool,
) -> bool {
    if !primitives.clips.is_empty() || !primitives.blend_modes.is_empty() {
        return false;
    }
    // 半透明颜色：GPU 顶点仅 RGB（mesh.rs color_to_f32 丢 alpha），
    // fill/rounded/stroke/path/glyph shader 输出 alpha 恒 1。
    if primitives.fills.iter().any(|p| p.color.a < 255)
        || overlay_fills.iter().any(|p| p.color.a < 255)
        || primitives.rounded_rects.iter().any(|p| p.color.a < 255)
        || overlay_rounded_rects.iter().any(|p| p.color.a < 255)
        || primitives.strokes.iter().any(|p| p.color.a < 255)
        || primitives.path_fills.iter().any(|p| p.color.a < 255)
        || primitives.path_strokes.iter().any(|p| p.color.a < 255)
        || primitives.glyphs.iter().any(|p| p.color.a < 255)
        || ui_glyphs.iter().any(|p| p.color.a < 255)
        || overlay_glyphs.iter().any(|p| p.color.a < 255)
    {
        return false;
    }
    // 阴影：GPU 仅硬边不透明矩形（collect_shadow_vertices），
    // 无模糊 / 无 spread / 无 inset / 不透明时才与 CPU 行为一致。
    if primitives
        .shadows
        .iter()
        .any(|s| s.blur_radius > 0.0 || s.spread_radius != 0.0 || s.inset || s.color.a < 255)
    {
        return false;
    }
    // 窗口模式：filter/transform 后处理仅 headless 生效（render_full_scene_gpu 的
    // headless_texture.is_some() 守卫），窗口模式下静默丢弃 → 回退。
    if !headless && (!primitives.filters.is_empty() || !primitives.transforms.is_empty()) {
        return false;
    }
    true
}
