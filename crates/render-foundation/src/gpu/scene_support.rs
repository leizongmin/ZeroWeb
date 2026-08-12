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
/// - 滤镜/变换已支持窗口模式（D/R3279：离屏纹理后处理 + blit 回 surface）
///
/// 不受影响、无需回退：半透明颜色（P2-8 后顶点携带 alpha，shader 输出
/// `color.a × 覆盖率`）、渐变（纹理 RGBA）、图片、硬边不透明阴影。
pub fn scene_supported(primitives: &RenderPrimitives) -> bool {
    // C（R3278）+ R3284：clip 全路径支持（draw_order 白 rect 原位 / 分桶末尾擦白，
    // 均对齐 CPU 语义）；blend 需 draw_order 顺序（双 pass 分段）——分桶路径无顺序
    // 语义 → 仍拒绝回退（blend 分桶影响面≈0：painter 默认产 draw_order）。
    if !primitives.blend_modes.is_empty() && primitives.draw_order.is_empty() {
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
    // D/R3279：filter/transform 后处理已支持窗口模式（离屏纹理 ping-pong + blit 回
    // surface）——不再回退。
    true
}
