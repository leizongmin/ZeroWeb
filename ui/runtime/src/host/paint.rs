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
    // U-3：ScrollVertical 容器画可选 scrollbar（content > viewport 时）。
    // 默认显示；prop `show_scrollbar = false` 可关闭。
    paint_scrollbar(node, scene, own_clip, tokens);
}

/// 为 ScrollVertical 容器节点画竖直 scrollbar（thumb 比例反映 scroll 进度）。
///
/// 仅当 `content_height > viewport_height` 且未显式关闭时绘制。thumb 颜色取
/// `on_background` 的半透明，track 不画（避免视觉负担）。宽度 6px，距右边缘 2px。
fn paint_scrollbar(
    node: &HostNode,
    scene: &mut Scene,
    parent_clip: Option<Rect>,
    tokens: &zero_ui_core::theme::SemanticTokens,
) {
    // 是否是 ScrollVertical 容器。
    let is_scroll = super::layout::node_container_kind(node)
        .map(|k| matches!(k, super::ContainerKind::ScrollVertical))
        .unwrap_or(false);
    if !is_scroll {
        return;
    }
    // prop show_scrollbar = false → 关闭。
    if let Some(Value::Bool(false)) = node.props.get(prop_keys::SHOW_SCROLLBAR) {
        return;
    }
    let viewport = node.cached_rect.size.height;
    let content = node.content_height;
    if content <= viewport + 1.0 {
        return; // 内容不超出，不画。
    }
    let max_scroll = (content - viewport).max(1.0);
    let ratio = node.scroll_offset / max_scroll;
    // thumb 高度 = viewport * (viewport/content)，最小 24px。
    let thumb_h = (viewport * viewport / content).max(24.0).min(viewport);
    let track_h = viewport - thumb_h;
    let thumb_y = node.cached_rect.origin.y + ratio * track_h;
    let thumb_x = node.cached_rect.origin.x + node.cached_rect.size.width - 8.0;
    let thumb_rect = Rect::from_origin_size(
        zero_ui_core::geometry::Point::new(thumb_x, thumb_y),
        zero_ui_core::geometry::Size::new(6.0, thumb_h),
    );
    // U3-2 修复：thumb 颜色与 SDK ScrollBarStyle::from_tokens 同口径
    // （on_surface.mix(surface, 0.5)，light→深 thumb 浅 track，dark→浅 thumb 深 track，
    // 自动适应明暗主题）。之前用 `Color::rgba(c.r,c.g,c.b, 0.5*255.0)` 是 bug：
    // Color::rgba 的 alpha 是 0..1 范围，0.5*255=127.5 被钳到 1.0 → 完全不透明的纯黑
    // （light 主题）/ 纯白（dark 主题），看似「纯黑滚动条」。
    let thumb_color = tokens.on_surface.mix(tokens.surface, 0.5);
    scene.push(SceneEntry {
        source: WidgetId::new(node.id.0.as_str()),
        clip: parent_clip,
        primitive: RenderPrimitive::FillRect {
            rect: thumb_rect,
            color: thumb_color,
            rounding: zero_ui_core::geometry::Rounding::all(3.0),
        },
    });
}
