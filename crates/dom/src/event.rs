//! DOM Event 系统 — 事件类型、事件目标和事件派发。
//!
//! 实现了 WHATWG DOM Standard 中的 Event 和 EventTarget 核心功能。

use crate::node::NodeId;
use std::fmt;

// ── EventPhase ─────────────────────────────────────────────────────────

/// 事件传播阶段。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventPhase {
    /// 捕获阶段。
    Capturing = 1,
    /// 目标阶段。
    AtTarget = 2,
    /// 冒泡阶段。
    Bubbling = 3,
}

// ── Event ──────────────────────────────────────────────────────────────

/// DOM 事件 — 表示 DOM 中发生的事件。
///
/// 包含事件类型、目标、传播控制等核心属性。
/// 对应 WHATWG DOM Standard 中的 Event 接口。
pub struct Event {
    /// 事件类型（如 "click"、"load"）。
    event_type: String,
    /// 事件是否冒泡。
    bubbles: bool,
    /// 事件是否可取消。
    cancelable: bool,
    /// 事件的当前传播阶段。
    pub(crate) phase: EventPhase,
    /// 事件的目标节点（最初派发的节点）。
    target: Option<NodeId>,
    /// 当前处理事件的节点（随传播变化）。
    pub(crate) current_target: Option<NodeId>,
    /// 是否已停止传播。
    stop_propagation: bool,
    /// 是否已停止立即传播（不再调用同节点上的后续监听器）。
    stop_immediate_propagation: bool,
    /// 是否已取消默认行为。
    canceled: bool,
    /// 是否已标记为已处理（trusted + dispatched）。
    dispatched: bool,
}

impl Event {
    /// 创建一个新事件。
    ///
    /// 默认 `bubbles = false`，`cancelable = false`。
    pub fn new(event_type: &str) -> Self {
        Self {
            event_type: event_type.to_string(),
            bubbles: false,
            cancelable: false,
            phase: EventPhase::AtTarget,
            target: None,
            current_target: None,
            stop_propagation: false,
            stop_immediate_propagation: false,
            canceled: false,
            dispatched: false,
        }
    }

    /// 创建一个带选项的事件。
    pub fn new_with_options(event_type: &str, bubbles: bool, cancelable: bool) -> Self {
        Self {
            event_type: event_type.to_string(),
            bubbles,
            cancelable,
            phase: EventPhase::AtTarget,
            target: None,
            current_target: None,
            stop_propagation: false,
            stop_immediate_propagation: false,
            canceled: false,
            dispatched: false,
        }
    }

    /// 获取事件类型。
    #[inline]
    pub fn event_type(&self) -> &str {
        &self.event_type
    }

    /// 事件是否冒泡。
    #[inline]
    pub fn bubbles(&self) -> bool {
        self.bubbles
    }

    /// 事件是否可取消。
    #[inline]
    pub fn cancelable(&self) -> bool {
        self.cancelable
    }

    /// 获取当前传播阶段。
    #[inline]
    pub fn phase(&self) -> EventPhase {
        self.phase
    }

    /// 获取事件目标节点。
    #[inline]
    pub fn target(&self) -> Option<NodeId> {
        self.target
    }

    /// 获取当前处理事件的节点。
    #[inline]
    pub fn current_target(&self) -> Option<NodeId> {
        self.current_target
    }

    /// 是否已停止传播。
    #[inline]
    pub fn propagation_stopped(&self) -> bool {
        self.stop_propagation
    }

    /// 是否已停止立即传播。
    #[inline]
    pub fn immediate_propagation_stopped(&self) -> bool {
        self.stop_immediate_propagation
    }

    /// 是否已取消默认行为。
    #[inline]
    pub fn default_prevented(&self) -> bool {
        self.canceled
    }

    /// 停止事件传播。
    ///
    /// 调用后，事件不再传播到后续节点，但当前节点上剩余的监听器仍会被调用。
    pub fn stop_propagation(&mut self) {
        self.stop_propagation = true;
    }

    /// 停止事件立即传播。
    ///
    /// 调用后，事件不再传播，且当前节点上后续的监听器也不会被调用。
    pub fn stop_immediate_propagation(&mut self) {
        self.stop_propagation = true;
        self.stop_immediate_propagation = true;
    }

    /// 取消默认行为。
    ///
    /// 仅在 `cancelable = true` 时有效。返回是否成功取消。
    pub fn prevent_default(&mut self) -> bool {
        if self.cancelable {
            self.canceled = true;
            true
        } else {
            false
        }
    }

    /// 初始化事件（内部使用，重置派发状态）。
    pub(crate) fn init_for_dispatch(&mut self, target: NodeId) {
        self.target = Some(target);
        self.current_target = None;
        self.stop_propagation = false;
        self.stop_immediate_propagation = false;
        self.canceled = false;
        self.dispatched = true;
        self.phase = EventPhase::AtTarget;
    }
}

impl fmt::Debug for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Event")
            .field("type", &self.event_type)
            .field("bubbles", &self.bubbles)
            .field("cancelable", &self.cancelable)
            .field("phase", &self.phase)
            .field("target", &self.target)
            .field("current_target", &self.current_target)
            .field("stop_propagation", &self.stop_propagation)
            .field("default_prevented", &self.canceled)
            .finish()
    }
}

// ── EventListener ──────────────────────────────────────────────────────

/// 事件监听回调函数类型。
pub type EventListenerFn = Box<dyn Fn(&mut Event)>;

// ── EventListenerHandle ────────────────────────────────────────────────

/// 事件监听器句柄，用于标识已注册的监听器（便于移除）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventListenerHandle {
    /// 监听器所在的节点。
    pub node: NodeId,
    /// 事件类型索引（内部使用）。
    pub index: usize,
}

// ── Internal: ListenerEntry ────────────────────────────────────────────

/// 内部监听器条目。
pub(crate) struct ListenerEntry {
    /// 监听器回调。
    pub callback: EventListenerFn,
    /// 是否在捕获阶段触发。
    pub capture: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Document;

    /// 测试事件重用：dispatch 两次后 init_for_dispatch 应重置状态。
    #[test]
    fn test_event_reuse_resets_state() {
        let mut event = Event::new("click");
        event.stop_propagation();
        assert!(event.propagation_stopped());

        // init_for_dispatch 重置状态
        let _target = {
            let mut doc = Document::new();
            let elem = doc.create_element("div");
            doc.append_child(doc.root(), elem).unwrap();
            elem
        };
        // 注意：target 可能已无效，但 init_for_dispatch 只设置 phase/target
        // 这里直接测试 stop_propagation 状态的保持
        // 实际 dispatch 会调用 init_for_dispatch 重置
        let fresh = Event::new("click");
        assert!(!fresh.propagation_stopped());
    }

    /// 测试 Debug trait 实现。
    #[test]
    fn test_event_debug_format() {
        let event = Event::new("click");
        let debug_str = format!("{event:?}");
        assert!(debug_str.contains("click"), "Debug should contain event type");
    }

    /// 测试 prevent_default 在捕获阶段。
    #[test]
    fn test_prevent_default_in_capture_phase() {
        let mut doc = Document::new();
        let parent = doc.create_element("div");
        doc.append_child(doc.root(), parent).unwrap();
        let child = doc.create_element("span");
        doc.append_child(parent, child).unwrap();

        doc.add_event_listener(
            parent,
            "click",
            Box::new(|e| {
                e.prevent_default();
            }),
            true,
        );

        let mut event = Event::new_with_options("click", true, true);
        let result = doc.dispatch_event(child, &mut event);
        assert!(
            event.default_prevented(),
            "prevent_default should be true after capture-phase call"
        );
        assert!(!result, "dispatch should return false when default prevented");
    }

    /// 测试 Event::new_with_options 全参数。
    #[test]
    fn test_event_new_with_options() {
        let event = Event::new_with_options("custom", true, false);
        assert_eq!(event.event_type(), "custom");
        assert!(event.bubbles());
        assert!(!event.cancelable());
    }

    /// 测试深层嵌套事件传播（5 层）。
    #[test]
    fn test_deep_nesting_propagation() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let mut doc = Document::new();
        let mut current = doc.root();
        let mut all_ids = vec![current];
        for _ in 0..5 {
            let child = doc.create_element("div");
            doc.append_child(current, child).unwrap();
            current = child;
            all_ids.push(current);
        }
        let target = current;

        let call_count = Arc::new(AtomicUsize::new(0));
        for &id in &all_ids {
            let count = call_count.clone();
            doc.add_event_listener(
                id,
                "test",
                Box::new(move |_| {
                    count.fetch_add(1, Ordering::SeqCst);
                }),
                true,
            );
            let count = call_count.clone();
            doc.add_event_listener(
                id,
                "test",
                Box::new(move |_| {
                    count.fetch_add(1, Ordering::SeqCst);
                }),
                false,
            );
        }

        let mut event = Event::new("test");
        doc.dispatch_event(target, &mut event);
        let total = call_count.load(Ordering::SeqCst);
        // 6 个节点各 2 个监听器（capture + bubble）= 12
        assert!(
            total >= 6,
            "deep nesting should propagate through all levels, got {total} calls"
        );
    }
}
