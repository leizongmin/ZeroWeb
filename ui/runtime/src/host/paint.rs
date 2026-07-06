//! Paint — 遍历 retained widget 实例树，把每个 widget paint 进全局 `Scene`（P0-2 拆分）。
//!
//! 入口：[`paint_node`]。每个 widget 以局部坐标 paint 到 `SceneRecorder`，再按节点绝对
//! `cached_rect.origin` 平移后并入全局 `Scene`；clip 取祖先 clip 与节点 rect 的交集。
//!
//! 视口外 early-out（P3-1）：节点完全在 parent_clip 之外时跳过整棵子树。

use zero_ui_core::binding::Value;
use zero_ui_core::geometry::{Rect, Vec2};
use zero_ui_core::prop_keys;
use zero_ui_core::widget::{PaintCtx, WidgetId};
use zero_ui_render::{RenderPrimitive, Scene, SceneEntry, SceneRecorder};

use super::HostNode;

/// 递归遍历 widget 实例树，paint 进全局 `Scene`。
///
/// # Clip 链契约
///
/// 视口外节点（`parent_clip.intersect(cached_rect) == None`）产出 `own_clip = None`。
/// `None` 裁剪语义由后端 adapter 定义：`ui/adapters/render-foundation` 把 `None`
/// 回落为视口（viewport fallback），使节点仍可能渲染在视口内。这是**有意设计**：
/// - 根节点 `parent_clip = Some(viewport)` → 可见节点产 `Some(intersection)` →
///   不可见节点产 `None`。
/// - 若改为 `.unwrap_or(Rect::ZERO)`（视口外→零面积 clip），虽然语义更纯，
///   但 bridge stateful-clip 路径对此未经充分验证（O1 follow-up）。
/// - `Rect::intersect` 边相接（共享一条边，零面积）返 `None`（ui/core 已回归测试覆盖），
///   此时 bridge viewport fallback 使边缘节点正确渲染。
pub(super) fn paint_node(
    node: &mut HostNode,
    scene: &mut Scene,
    parent_clip: Option<Rect>,
    tokens: &zero_ui_core::theme::SemanticTokens,
    font_metrics: Option<(f32, f32)>,
    now_ms: Option<i64>,
    frame_requests: &std::cell::Cell<u64>,
) {
    // 视口外 early-out（P3-1）：节点完全在 parent_clip 之外时跳过整个子树。
    let own_clip = match parent_clip {
        Some(pc) => match pc.intersect(node.cached_rect) {
            Some(c) => Some(c),
            None => return,
        },
        None => None,
    };
    // 容器节点底色：无 widget 的容器（layout=column/row/stack）若声明 `bg` prop（**token 名**，
    // 如 "surface"/"background"），先铺底色再画子节点。
    if node.widget.is_none()
        && let Some(Value::Text(token)) = node.props.get(prop_keys::BG)
        && let Some(color) = tokens.color_for(token)
    {
        scene.push(SceneEntry {
            source: WidgetId::new(node.id.0.as_str()),
            clip: own_clip,
            primitive: RenderPrimitive::FillRect {
                rect: node.cached_rect,
                color,
                rounding: zero_ui_core::geometry::Rounding::ZERO,
            },
        });
    }
    if let Some(w) = node.widget.as_mut() {
        // widget 以节点局部坐标 paint；own_clip 是绝对坐标，需平移到局部后传给 recorder。
        let local_clip = own_clip.map(|c| c.translate(-node.cached_rect.origin.x, -node.cached_rect.origin.y));
        let mut rec = SceneRecorder::new(WidgetId::new(node.id.0.as_str()));
        rec.set_clip(local_clip);
        let mut ctx = PaintCtx {
            recorder: &mut rec,
            clip: local_clip,
            offset: Vec2::ZERO,
            tokens,
            font_metrics,
            now_ms,
            frame_requests,
        };
        w.paint(&mut ctx);
        let local = rec.finish();
        let abs_offset = Vec2::new(node.cached_rect.origin.x, node.cached_rect.origin.y);
        // P2-8：消费 local.entries，避免每个 primitive + source 都 clone 一次。
        scene.extend_translated(local, abs_offset);
    }
    for child in node.children.iter_mut() {
        paint_node(child, scene, own_clip, tokens, font_metrics, now_ms, frame_requests);
    }
}
