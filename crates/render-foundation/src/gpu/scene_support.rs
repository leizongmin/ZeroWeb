//! GPU 场景支持检测 — 判定当前帧能否被 GPU 渲染器正确绘制。
//!
//! GPU 生产路径（`render_full_scene_gpu`）对部分特性未实现或静默降级：不检测时
//! 会「静默画错」（测试通过但用户看到错误像素）。调用方（浏览器本地渲染 /
//! 合成器 GPU 光栅）在返回 `false` 时回退 CPU 整帧重画——慢但对。
//!
//! 基线：docs/learnings/bugs/cpu-gpu-path-divergence.md（P0-1 回退项）。

use crate::primitive::RenderPrimitives;

/// GPU 能否正确渲染本帧；`false` 时调用方应回退 CPU 渲染。
///
/// 检测 GPU 生产路径未实现、会静默丢弃或画错的特性：
/// - `clips` / `blend_modes`：`render_full_scene_gpu` 完全不消费
/// - 带模糊/spread/inset 的阴影：GPU 只画硬边矩形
/// - 重复渐变且首色标 offset ≠ 0：GPU shader `fract(t)` 与 CPU `[first,last]`
///   折叠语义不同，GPU 无 first/last 传参通道
/// - 窗口模式（`headless=false`）下的 filter/transform：后处理仅 headless 生效
///
/// 不受影响、无需回退：半透明颜色（P2-8 后顶点携带 alpha，shader 输出
/// `color.a × 覆盖率`）、渐变（纹理 RGBA）、图片、硬边不透明阴影。
pub fn scene_supported(primitives: &RenderPrimitives, headless: bool) -> bool {
    if !primitives.clips.is_empty() || !primitives.blend_modes.is_empty() {
        return false;
    }
    // 阴影：GPU 仅硬边矩形（collect_shadow_vertices），
    // 无模糊 / 无 spread / 无 inset 时才与 CPU 行为一致。
    if primitives
        .shadows
        .iter()
        .any(|s| s.blur_radius > 0.0 || s.spread_radius != 0.0 || s.inset)
    {
        return false;
    }
    // 重复渐变且首色标 offset ≠ 0：GPU shader `fract(t)` 折叠回 [0,1)（归一化纹理），
    // CPU 折叠回 [first,last]（原 offset 采样）——GPU 无 first/last 传参通道，回退。
    // （首 offset = 0 时两种语义一致，无需回退。）
    if primitives
        .gradients
        .iter()
        .any(|g| g.repeating && g.stops.first().is_some_and(|s| s.offset.abs() > 1e-6))
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
