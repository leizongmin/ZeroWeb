// DOM crate 综合测试套件（第二部分）。
//
// 覆盖：Event 系统。

use crate::*;
use std::sync::{Arc, Mutex};

// ═══════════════════════════════════════════════════════════════════════
// 16. Event 系统测试
// ═══════════════════════════════════════════════════════════════════════

// ── Event 创建和属性 ─────────────────────────────────────────────────

/// 测试 Event 基本创建。
#[test]
fn test_event_creation() {
    let event = Event::new("click");
    assert_eq!(event.event_type(), "click");
    assert!(!event.bubbles());
    assert!(!event.cancelable());
    assert!(!event.default_prevented());
    assert!(!event.propagation_stopped());
    assert_eq!(event.target(), None);
    assert_eq!(event.current_target(), None);
}

/// 测试 Event 带选项创建。
#[test]
fn test_event_creation_with_options() {
    let event = Event::new_with_options("submit", true, true);
    assert_eq!(event.event_type(), "submit");
    assert!(event.bubbles());
    assert!(event.cancelable());
}

/// 测试不同事件类型名称。
#[test]
fn test_event_various_types() {
    for event_type in &["click", "input", "keydown", "load", "scroll", "custom"] {
        let event = Event::new(event_type);
        assert_eq!(event.event_type(), *event_type);
    }
}

/// 测试 EventPhase 枚举值。
#[test]
fn test_event_phase_values() {
    assert_eq!(EventPhase::Capturing as i32, 1);
    assert_eq!(EventPhase::AtTarget as i32, 2);
    assert_eq!(EventPhase::Bubbling as i32, 3);
}

// ── Event preventDefault ─────────────────────────────────────────────

/// 测试 preventDefault 在 cancelable 事件上生效。
#[test]
fn test_prevent_default_cancelable() {
    let mut event = Event::new_with_options("click", true, true);
    assert!(!event.default_prevented());

    let result = event.prevent_default();
    assert!(result, "preventDefault should return true for cancelable event");
    assert!(event.default_prevented());
}

/// 测试 preventDefault 在不可取消事件上无效。
#[test]
fn test_prevent_default_not_cancelable() {
    let mut event = Event::new("load"); // bubbles=false, cancelable=false
    let result = event.prevent_default();
    assert!(!result, "preventDefault should return false for non-cancelable event");
    assert!(!event.default_prevented());
}

// ── Event stopPropagation ────────────────────────────────────────────

/// 测试 stopPropagation 设置标志。
#[test]
fn test_stop_propagation() {
    let mut event = Event::new("click");
    assert!(!event.propagation_stopped());
    event.stop_propagation();
    assert!(event.propagation_stopped());
}

/// 测试 stopImmediatePropagation 同时设置两个标志。
#[test]
fn test_stop_immediate_propagation() {
    let mut event = Event::new("click");
    assert!(!event.propagation_stopped());
    assert!(!event.immediate_propagation_stopped());

    event.stop_immediate_propagation();

    assert!(event.propagation_stopped());
    assert!(event.immediate_propagation_stopped());
}

// ── EventTarget add/remove/dispatch ──────────────────────────────────

/// 测试 add_event_listener 和 dispatch_event 基本流程。
#[test]
fn test_add_and_dispatch_event() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.append_child(root, elem).unwrap();

    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();
    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |_event| {
            *called_clone.lock().unwrap() = true;
        }),
        false,
    );

    assert_eq!(doc.listener_count(elem, "click"), 1);

    let mut event = Event::new("click");
    doc.dispatch_event(elem, &mut event);
    assert!(*called.lock().unwrap(), "event listener should have been called");
}

/// 测试 dispatch_event 设置事件 target。
#[test]
fn test_dispatch_sets_target() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.append_child(root, elem).unwrap();

    let target_received = Arc::new(Mutex::new(None));
    let target_clone = target_received.clone();
    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |event| {
            *target_clone.lock().unwrap() = event.target();
        }),
        false,
    );

    let mut event = Event::new("click");
    doc.dispatch_event(elem, &mut event);

    assert_eq!(*target_received.lock().unwrap(), Some(elem));
    assert_eq!(event.target(), Some(elem));
}

/// 测试 dispatch_event 返回值反映 defaultPrevented。
#[test]
fn test_dispatch_returns_prevented() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    // 可取消事件，监听器中 preventDefault
    doc.add_event_listener(
        elem,
        "click",
        Box::new(|event| {
            let _ = event.prevent_default();
        }),
        false,
    );

    let mut event = Event::new_with_options("click", false, true);
    let not_prevented = doc.dispatch_event(elem, &mut event);
    assert!(
        !not_prevented,
        "dispatch should return false when preventDefault is called"
    );
}

/// 测试 dispatch_event 返回 true 当没有 preventDefault。
#[test]
fn test_dispatch_returns_not_prevented() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    doc.add_event_listener(
        elem,
        "click",
        Box::new(|_event| {
            // 不调用 preventDefault
        }),
        false,
    );

    let mut event = Event::new_with_options("click", false, true);
    let not_prevented = doc.dispatch_event(elem, &mut event);
    assert!(not_prevented, "dispatch should return true when no preventDefault");
}

/// 测试 add_event_listener 多个监听器。
#[test]
fn test_multiple_listeners_same_type() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let count = Arc::new(Mutex::new(0usize));
    for _ in 0..3 {
        let count_clone = count.clone();
        doc.add_event_listener(
            elem,
            "click",
            Box::new(move |_event| {
                *count_clone.lock().unwrap() += 1;
            }),
            false,
        );
    }

    assert_eq!(doc.listener_count(elem, "click"), 3);

    let mut event = Event::new("click");
    doc.dispatch_event(elem, &mut event);
    assert_eq!(*count.lock().unwrap(), 3, "all 3 listeners should fire");
}

/// 测试不同事件类型的监听器互不影响。
#[test]
fn test_different_event_types() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let click_called = Arc::new(Mutex::new(false));
    let input_called = Arc::new(Mutex::new(false));
    let click_clone = click_called.clone();
    let input_clone = input_called.clone();

    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |_| {
            *click_clone.lock().unwrap() = true;
        }),
        false,
    );
    doc.add_event_listener(
        elem,
        "input",
        Box::new(move |_| {
            *input_clone.lock().unwrap() = true;
        }),
        false,
    );

    // 派发 click，只有 click 监听器触发
    let mut event = Event::new("click");
    doc.dispatch_event(elem, &mut event);
    assert!(*click_called.lock().unwrap());
    assert!(
        !*input_called.lock().unwrap(),
        "input listener should not fire for click event"
    );
}

/// 测试 remove_event_listener。
#[test]
fn test_remove_event_listener() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();
    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |_| {
            *called_clone.lock().unwrap() = true;
        }),
        false,
    );

    assert_eq!(doc.listener_count(elem, "click"), 1);
    let removed = doc.remove_event_listener(elem, "click");
    assert_eq!(removed, 1);
    assert_eq!(doc.listener_count(elem, "click"), 0);

    let mut event = Event::new("click");
    doc.dispatch_event(elem, &mut event);
    assert!(!*called.lock().unwrap(), "removed listener should not fire");
}

/// 测试 remove_event_listener 不存在的类型返回 0。
#[test]
fn test_remove_nonexistent_event_listener() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    let removed = doc.remove_event_listener(elem, "click");
    assert_eq!(removed, 0, "removing nonexistent listener should return 0");
}

/// 测试 remove_all_event_listeners。
#[test]
fn test_remove_all_event_listeners() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    doc.add_event_listener(elem, "click", Box::new(|_| {}), false);
    doc.add_event_listener(elem, "input", Box::new(|_| {}), false);

    assert_eq!(doc.listener_count(elem, "click"), 1);
    assert_eq!(doc.listener_count(elem, "input"), 1);

    doc.remove_all_event_listeners(elem);

    assert_eq!(doc.listener_count(elem, "click"), 0);
    assert_eq!(doc.listener_count(elem, "input"), 0);
}

/// 测试没有监听器时 dispatch_event 正常完成。
#[test]
fn test_dispatch_without_listeners() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    let mut event = Event::new("click");
    let not_prevented = doc.dispatch_event(elem, &mut event);
    assert!(not_prevented, "dispatch without listeners should return true");
    assert_eq!(event.target(), Some(elem));
}

// ── Event 冒泡 ───────────────────────────────────────────────────────

/// 测试事件冒泡通过 DOM 树。
#[test]
fn test_event_bubbling() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(span, p).unwrap();

    let bubble_path = Arc::new(Mutex::new(Vec::new()));
    let bp_clone = bubble_path.clone();
    let p_id = p;

    // 在每个节点上注册监听器，记录 current_target
    for node_id in [div, span, p] {
        let bp = bp_clone.clone();
        doc.add_event_listener(
            node_id,
            "click",
            Box::new(move |event| {
                bp.lock().unwrap().push(event.current_target());
            }),
            false,
        );
    }

    // 派发冒泡事件到最深节点 p
    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(p_id, &mut event);

    let path = bubble_path.lock().unwrap();
    // p (target) -> span -> div（不冒泡到 document root）
    assert_eq!(
        *path,
        vec![Some(p), Some(span), Some(div)],
        "event should bubble from target through ancestors"
    );
}

/// 测试非冒泡事件不冒泡。
#[test]
fn test_non_bubbling_event() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();

    let call_log = Arc::new(Mutex::new(Vec::new()));
    let log_clone = call_log.clone();

    for node_id in [div, span] {
        let log = log_clone.clone();
        doc.add_event_listener(
            node_id,
            "load",
            Box::new(move |event| {
                log.lock().unwrap().push(event.current_target());
            }),
            false,
        );
    }

    // 非冒泡事件
    let mut event = Event::new("load"); // bubbles = false
    doc.dispatch_event(span, &mut event);

    let log = call_log.lock().unwrap();
    // 只有 span (target) 触发
    assert_eq!(*log, vec![Some(span)], "non-bubbling event should only fire on target");
}

// ── stopPropagation ──────────────────────────────────────────────────

/// 测试 stopPropagation 阻止继续冒泡。
#[test]
fn test_stop_propagation_during_bubble() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(span, p).unwrap();

    let call_log = Arc::new(Mutex::new(Vec::new()));
    let log_p = call_log.clone();
    let log_span = call_log.clone();
    let log_div = call_log.clone();

    // p 监听器：stopPropagation
    doc.add_event_listener(
        p,
        "click",
        Box::new(move |event| {
            log_p.lock().unwrap().push("p");
            event.stop_propagation();
        }),
        false,
    );

    // span 监听器
    doc.add_event_listener(
        span,
        "click",
        Box::new(move |_event| {
            log_span.lock().unwrap().push("span");
        }),
        false,
    );

    // div 监听器
    doc.add_event_listener(
        div,
        "click",
        Box::new(move |_event| {
            log_div.lock().unwrap().push("div");
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(p, &mut event);

    let log = call_log.lock().unwrap();
    assert_eq!(*log, vec!["p"], "stopPropagation should prevent bubbling to ancestors");
}

/// 测试 stopImmediatePropagation 阻止同节点上的后续监听器。
#[test]
fn test_stop_immediate_propagation_same_node() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let call_log = Arc::new(Mutex::new(Vec::new()));
    let log1 = call_log.clone();
    let log2 = call_log.clone();
    let log3 = call_log.clone();

    // 第一个监听器：stopImmediatePropagation
    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |event| {
            log1.lock().unwrap().push("first");
            event.stop_immediate_propagation();
        }),
        false,
    );

    // 第二个监听器：不应触发
    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |_event| {
            log2.lock().unwrap().push("second");
        }),
        false,
    );

    // 第三个监听器：不应触发
    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |_event| {
            log3.lock().unwrap().push("third");
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(elem, &mut event);

    let log = call_log.lock().unwrap();
    assert_eq!(
        *log,
        vec!["first"],
        "stopImmediatePropagation should prevent remaining listeners on same node"
    );
}

/// 测试 stopPropagation（非 immediate）允许同节点上的后续监听器继续执行。
#[test]
fn test_stop_propagation_allows_same_node_listeners() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let call_log = Arc::new(Mutex::new(Vec::new()));
    let log1 = call_log.clone();
    let log2 = call_log.clone();

    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |event| {
            log1.lock().unwrap().push("first");
            event.stop_propagation(); // 非立即停止，同节点后续仍执行
        }),
        false,
    );

    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |_event| {
            log2.lock().unwrap().push("second");
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(elem, &mut event);

    let log = call_log.lock().unwrap();
    assert_eq!(
        *log,
        vec!["first", "second"],
        "stopPropagation (not immediate) should allow remaining listeners on same node"
    );
}

// ── 捕获阶段 ─────────────────────────────────────────────────────────

/// 测试捕获阶段监听器在祖先节点上先于目标触发。
#[test]
fn test_capture_phase() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();

    let call_log = Arc::new(Mutex::new(Vec::new()));
    let log_capture = call_log.clone();
    let log_bubble = call_log.clone();

    // div 上的捕获监听器
    doc.add_event_listener(
        div,
        "click",
        Box::new(move |_event| {
            log_capture.lock().unwrap().push("div-capture");
        }),
        true, // capture
    );

    // span 上的冒泡监听器（目标阶段）
    doc.add_event_listener(
        span,
        "click",
        Box::new(move |_event| {
            log_bubble.lock().unwrap().push("span-target");
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(span, &mut event);

    let log = call_log.lock().unwrap();
    assert_eq!(
        *log,
        vec!["div-capture", "span-target"],
        "capture listener on ancestor should fire before target listener"
    );
}

/// 测试完整的三阶段事件传播。
#[test]
fn test_full_event_propagation_phases() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();

    let call_log = Arc::new(Mutex::new(Vec::new()));

    // div 捕获
    let log = call_log.clone();
    doc.add_event_listener(
        div,
        "click",
        Box::new(move |event| {
            log.lock()
                .unwrap()
                .push(format!("div-capture(phase={:?})", event.phase()));
        }),
        true,
    );

    // div 冒泡
    let log = call_log.clone();
    doc.add_event_listener(
        div,
        "click",
        Box::new(move |event| {
            log.lock()
                .unwrap()
                .push(format!("div-bubble(phase={:?})", event.phase()));
        }),
        false,
    );

    // span 目标（capture=true）
    let log = call_log.clone();
    doc.add_event_listener(
        span,
        "click",
        Box::new(move |event| {
            log.lock()
                .unwrap()
                .push(format!("span-target-cap(phase={:?})", event.phase()));
        }),
        true,
    );

    // span 目标（capture=false）
    let log = call_log.clone();
    doc.add_event_listener(
        span,
        "click",
        Box::new(move |event| {
            log.lock()
                .unwrap()
                .push(format!("span-target(phase={:?})", event.phase()));
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(span, &mut event);

    let log = call_log.lock().unwrap();
    assert_eq!(log.len(), 4, "all 4 listeners should fire");
    // 顺序：div-capture -> span-target-cap -> span-target -> div-bubble
    assert!(log[0].contains("div-capture"), "first should be div capture");
    assert!(
        log[1].contains("span-target-cap"),
        "second should be span capture at target"
    );
    assert!(log[2].contains("span-target"), "third should be span at target");
    assert!(log[3].contains("div-bubble"), "fourth should be div bubble");
}

/// 测试捕获阶段 stopPropagation 阻止目标阶段。
#[test]
fn test_stop_propagation_in_capture_phase() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();

    let call_log = Arc::new(Mutex::new(Vec::new()));

    // div 捕获：stopPropagation
    let log = call_log.clone();
    doc.add_event_listener(
        div,
        "click",
        Box::new(move |event| {
            log.lock().unwrap().push("div-capture");
            event.stop_propagation();
        }),
        true,
    );

    // span 目标：不应触发
    let log = call_log.clone();
    doc.add_event_listener(
        span,
        "click",
        Box::new(move |_| {
            log.lock().unwrap().push("span-target");
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(span, &mut event);

    let log = call_log.lock().unwrap();
    assert_eq!(
        *log,
        vec!["div-capture"],
        "stopPropagation in capture should prevent target phase"
    );
}
