//! Event dispatch — hit-test + 冒泡派发 + 焦点路由（P0-2 拆分）。
//!
//! host 主入口 `dispatch_event` 在本模块外，模块内提供：
//! - [`dispatch_node`]：递归命中测试 + 局部坐标派发，收集 widget 发出的 action。
//! - [`dispatch_to_widget`]：按 id 直接派发（hover 追踪用，发合成 Exited）。
//! - [`localize_position`]：把事件位置平移为相对节点 origin 的局部坐标。
//! - 树形查询：[`find_node`] / [`find_node_mut`] / [`find_rect`] / [`find_epoch`]。
//! - 命中测试：[`deepest_node_at`] / [`deepest_focusable_at`] / [`deepest_scroll_vertical_at`]。
//! - [`collect_emit`] / [`collect_focusables`]：辅助收集。

use zero_ui_core::action::{ActionId, ActionPayload, EventResult};
use zero_ui_core::event::{Modifiers, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Point, Rect};
use zero_ui_core::widget::{EventCtx, WidgetId};

use super::{ContainerKind, EmittedAction, HostNode, layout};

/// 把 `EventResult` 中的 action 收集进 `emitted`。
pub(super) fn collect_emit(res: EventResult, emitted: &mut Vec<EmittedAction>) {
    match res {
        EventResult::Ignored | EventResult::Consumed => {}
        EventResult::Emit(action) => emitted.push(EmittedAction { action, payload: None }),
        EventResult::EmitWithPayload(action, payload) => emitted.push(EmittedAction {
            action,
            payload: Some(payload),
        }),
    }
}

/// 按 id 查可变节点。
pub(super) fn find_node_mut<'a>(node: &'a mut HostNode, id: &WidgetId) -> Option<&'a mut HostNode> {
    if &node.id == id {
        return Some(node);
    }
    node.children.iter_mut().find_map(|c| find_node_mut(c, id))
}

/// 按 id 查不可变节点。
pub(super) fn find_node<'a>(node: &'a HostNode, id: &WidgetId) -> Option<&'a HostNode> {
    if &node.id == id {
        return Some(node);
    }
    node.children.iter().find_map(|c| find_node(c, id))
}

/// 查 `WidgetId` 节点的绝对 rect（layout 后有效；IME rect / 断言用）。
pub(super) fn find_rect(node: &HostNode, id: &WidgetId) -> Option<Rect> {
    if &node.id == id {
        return Some(node.cached_rect);
    }
    node.children.iter().find_map(|c| find_rect(c, id))
}

/// 查 `WidgetId` 节点的创建 epoch（reconcile 复用判断用）。
pub(super) fn find_epoch(node: &HostNode, id: &WidgetId) -> Option<u32> {
    if &node.id == id {
        return Some(node.epoch);
    }
    node.children.iter().find_map(|c| find_epoch(c, id))
}

/// 命中点下最深的 focusable 节点 id（click-to-focus 用，DC-8 phase-2）。
pub(super) fn deepest_focusable_at(node: &HostNode, point: Point) -> Option<WidgetId> {
    if !node.cached_rect.contains(point) {
        return None;
    }
    for child in node.children.iter().rev() {
        if let Some(id) = deepest_focusable_at(child, point) {
            return Some(id);
        }
    }
    if node.focusable { Some(node.id.clone()) } else { None }
}

/// 命中点下最深的「垂直滚动容器」节点 id（DC-16 gallery scroll）。
pub(super) fn deepest_scroll_vertical_at(node: &HostNode, point: Point) -> Option<WidgetId> {
    if !node.cached_rect.contains(point) {
        return None;
    }
    for child in node.children.iter().rev() {
        if let Some(id) = deepest_scroll_vertical_at(child, point) {
            return Some(id);
        }
    }
    if layout::node_container_kind(node) == Some(ContainerKind::ScrollVertical) {
        Some(node.id.clone())
    } else {
        None
    }
}

/// 命中点下最深节点 id（hover 追踪用，不论是否 focusable）。
pub(super) fn deepest_node_at(node: &HostNode, point: Point) -> Option<WidgetId> {
    if !node.cached_rect.contains(point) {
        return None;
    }
    for child in node.children.iter().rev() {
        if let Some(id) = deepest_node_at(child, point) {
            return Some(id);
        }
    }
    Some(node.id.clone())
}

/// 按声明（前序）顺序收集所有 focusable 节点的 id。
pub(super) fn collect_focusables(node: &HostNode, out: &mut Vec<WidgetId>) {
    if node.focusable {
        out.push(node.id.clone());
    }
    for c in &node.children {
        collect_focusables(c, out);
    }
}

/// 向指定 id 的节点（非 hit-test 定位）派发合成 `Exited` 事件（F1 hover 追踪）。
///
/// 与 [`dispatch_node`] 不同：本函数不遍历命中测试，而是按 id 精确查找目标节点
/// 并直接派发，用以通知旧悬停节点清除交互态（pressed/hover）。
/// 无 widget 或不命中 rect 时直接返回（不报错）。
pub(super) fn dispatch_to_widget(
    node: &mut HostNode,
    target: &WidgetId,
    phase: PointerPhase,
    _emitted: &mut Vec<EmittedAction>,
) {
    if let Some(target_node) = find_node_mut(node, target)
        && let Some(w) = target_node.widget.as_mut()
    {
        let exited = UiEvent::Pointer {
            phase,
            button: None,
            position: Point::ZERO,
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        };
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let res = w.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &exited,
        );
        target_node.invalidation |= flags;
        let _ = res;
    }
}

pub(super) fn dispatch_node(node: &mut HostNode, event: &UiEvent, emitted: &mut Vec<EmittedAction>) -> bool {
    let Some(abs_pos) = event.position() else {
        return false;
    };
    if !node.cached_rect.contains(abs_pos) {
        return false;
    }
    for child in node.children.iter_mut().rev() {
        if dispatch_node(child, event, emitted) {
            return true;
        }
    }
    if let Some(w) = node.widget.as_mut() {
        let local_event = localize_position(event, node.cached_rect.origin);
        let mut flags = zero_ui_core::invalidation::InvalidationFlags::CLEAN;
        let result = w.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &local_event,
        );
        node.invalidation |= flags;
        match result {
            EventResult::Ignored => false,
            EventResult::Consumed => true,
            EventResult::Emit(action) => {
                emitted.push(EmittedAction { action, payload: None });
                true
            }
            EventResult::EmitWithPayload(action, payload) => {
                emitted.push(EmittedAction {
                    action,
                    payload: Some(payload),
                });
                true
            }
        }
    } else {
        false
    }
}

/// 克隆事件并把位置平移为相对 `origin` 的局部坐标（widget 内部按自身坐标系处理）。
pub(super) fn localize_position(event: &UiEvent, origin: Point) -> UiEvent {
    match event {
        UiEvent::Pointer {
            phase,
            button,
            position,
            modifiers,
            pointer_id,
        } => UiEvent::Pointer {
            phase: *phase,
            button: *button,
            position: Point::new(position.x - origin.x, position.y - origin.y),
            modifiers: *modifiers,
            pointer_id: *pointer_id,
        },
        UiEvent::Scroll {
            delta,
            phase,
            position,
            modifiers,
        } => UiEvent::Scroll {
            delta: *delta,
            phase: *phase,
            position: Point::new(position.x - origin.x, position.y - origin.y),
            modifiers: *modifiers,
        },
        other => other.clone(),
    }
}

// 保持 ActionId / ActionPayload 引入，避免因模块独立后未使用警告（统一从顶层带过来）。
#[allow(dead_code)]
fn _action_unused(_: ActionId, _: ActionPayload) {}
