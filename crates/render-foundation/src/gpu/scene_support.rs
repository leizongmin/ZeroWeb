//! GPU 场景支持检测 — 判定当前帧能否被 GPU 渲染器正确绘制。
//!
//! GPU 生产路径（`render_full_scene_gpu`）对部分特性未实现或静默降级：不检测时
//! 会「静默画错」（测试通过但用户看到错误像素）。调用方（浏览器本地渲染 /
//! 合成器 GPU 光栅）在返回 `false` 时回退 CPU 整帧重画——慢但对。
//!
//! 基线：docs/learnings/bugs/2026-08/2026-08-12-cpu-gpu-path-divergence.md（P0-1 回退项）。

use crate::primitive::{DrawOp, RenderPrimitives};

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
    // R3287 + R3290：阴影模糊/inset 均 GPU 支持（离屏蒙版 + 区域 blur + alpha 混合）。
    // R3289：repeating 渐变色标重映射（周期 [first,last] → [0,1]）——GPU fract(t)
    // 与 CPU 折叠等效，不再回退。
    // D/R3279：filter/transform 后处理已支持窗口模式（离屏纹理 ping-pong + blit 回
    // surface）——不再回退。
    true
}

/// T34：合成器脏区 clip 路径（fill+glyph 双图元管线，`render_page_scene_with_clip_scaled`）
/// 能否正确绘制本帧；`false` 时调用方应回退 CPU 脏区渲染。
///
/// clip 路径只有 fill+glyph 一条管线，帧内出现其他图元类型时继续渲染会把脏区内
/// 对应内容静默抹掉（此前只守卫 images、glyphs 直接传空数组——脏区文字整块消失，
/// 渐变/圆角/描边同样会丢），故整帧拒绝回退 CPU（慢但对，同 P0-1 理由）。
///
/// z 序同时校验：clip 路径按「全部 fills → 全部 glyphs」类型分桶绘制，`draw_order`
/// 中存在 glyph 之后的 fill（fill 应盖住文字）时分桶序会画错，也须拒绝。
/// `draw_order` 为空（旧快照）时类型分桶即既定回退语义，视为安全。
pub fn clip_scene_supported(primitives: &RenderPrimitives) -> bool {
    let only_fills_glyphs = primitives.clips.is_empty()
        && primitives.rounded_rects.is_empty()
        && primitives.path_fills.is_empty()
        && primitives.path_strokes.is_empty()
        && primitives.strokes.is_empty()
        && primitives.gradients.is_empty()
        && primitives.shadows.is_empty()
        && primitives.images.is_empty()
        && primitives.filters.is_empty()
        && primitives.blend_modes.is_empty()
        && primitives.transforms.is_empty();
    if !only_fills_glyphs {
        return false;
    }
    let mut seen_glyph = false;
    for op in &primitives.draw_order {
        match op {
            DrawOp::Glyph(_) => seen_glyph = true,
            DrawOp::Fill(_) if seen_glyph => return false,
            _ => {}
        }
    }
    true
}
