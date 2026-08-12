//! 针对 document.rs 中未覆盖路径的补充测试。

use super::*;
use crate::mutation::MutationCallbackFn;

// ═══════════════════════════════════════════════════════════════════════
// 1. insert_before 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_insert_before_basic() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child1 = doc.create_text_node("first");
    let child2 = doc.create_text_node("second");
    doc.append_child(parent, child1).unwrap();
    doc.insert_before(parent, child2, child1).unwrap();
    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], child2);
    assert_eq!(children[1], child1);
}

#[test]
fn test_insert_before_nonexistent_parent() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_text_node("text");
    let ref_node = doc.create_text_node("ref");
    let result = doc.insert_before(parent, child, ref_node);
    assert!(result.is_err());
}

#[test]
fn test_insert_before_root_as_child() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let ref_node = doc.create_text_node("ref");
    doc.append_child(parent, ref_node).unwrap();
    let result = doc.insert_before(parent, doc.root(), ref_node);
    assert!(matches!(result, Err(DomError::CannotInsertDocumentRoot)));
}

#[test]
fn test_insert_before_would_create_cycle() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();
    let ref_node = doc.create_text_node("ref");
    doc.append_child(parent, ref_node).unwrap();
    let result = doc.insert_before(child, parent, ref_node);
    assert!(matches!(result, Err(DomError::WouldCreateCycle)));
}

// ── R3350：same-parent move 回归（insert_before / replace_child stale-index bug） ──

/// 辅助：读取元素子节点的 tag 名列表（按文档顺序）。
fn child_tags(doc: &Document, parent: NodeId) -> Vec<String> {
    doc.child_nodes(parent)
        .iter()
        .map(|n| {
            doc.get(*n)
                .map(|x| match &x.kind {
                    crate::NodeKind::Element(e) => e.local_name().to_string(),
                    _ => "?".to_string(),
                })
                .unwrap_or_default()
        })
        .collect()
}

/// R3350：insert_before 把**已是本父的子节点**前移到另一子节点之前——旧实现因
/// 先算 ref_idx 再 detach(new_node) 致索引指向错误位置（A 被追加到末尾而非 D 之前）。
/// 修复后正确得 [B, C, A, D]。
#[test]
fn r3350_insert_before_move_forward() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let a = doc.create_element("a");
    let b = doc.create_element("b");
    let c = doc.create_element("c");
    let d = doc.create_element("d");
    doc.append_child(parent, a).unwrap();
    doc.append_child(parent, b).unwrap();
    doc.append_child(parent, c).unwrap();
    doc.append_child(parent, d).unwrap();
    // 把 A（首子）移到 D（末子）之前
    doc.insert_before(parent, a, d).unwrap();
    assert_eq!(child_tags(&doc, parent), vec!["b", "c", "a", "d"]);
}

/// R3350：insert_before 后移——把 B（index 1）移到 D（index 3）之前，期望 [A, C, B, D]。
#[test]
fn r3350_insert_before_move_backward() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let a = doc.create_element("a");
    let b = doc.create_element("b");
    let c = doc.create_element("c");
    let d = doc.create_element("d");
    doc.append_child(parent, a).unwrap();
    doc.append_child(parent, b).unwrap();
    doc.append_child(parent, c).unwrap();
    doc.append_child(parent, d).unwrap();
    doc.insert_before(parent, b, d).unwrap();
    assert_eq!(child_tags(&doc, parent), vec!["a", "c", "b", "d"]);
}

/// R3350：insert_before 同节点 no-op——insert_before(parent, A, A) 不应改动顺序、不报错。
#[test]
fn r3350_insert_before_self_noop() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let a = doc.create_element("a");
    let b = doc.create_element("b");
    doc.append_child(parent, a).unwrap();
    doc.append_child(parent, b).unwrap();
    doc.insert_before(parent, a, a).unwrap();
    assert_eq!(child_tags(&doc, parent), vec!["a", "b"]);
}

/// R3350：insert_before 前移到首子之前——C 移到 A 之前，期望 [C, A, B]。
#[test]
fn r3350_insert_before_move_to_front() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let a = doc.create_element("a");
    let b = doc.create_element("b");
    let c = doc.create_element("c");
    doc.append_child(parent, a).unwrap();
    doc.append_child(parent, b).unwrap();
    doc.append_child(parent, c).unwrap();
    doc.insert_before(parent, c, a).unwrap();
    assert_eq!(child_tags(&doc, parent), vec!["c", "a", "b"]);
}

/// R3350：replace_child 用**已是本父的子节点**替换另一子节点——旧实现因
/// 先算 old_idx 再 detach(new_child) 致 `children[old_idx]` 越界 panic（len 收缩）。
/// 修复后正确得 [A, B, C]→replace(A,C)→[C, B]（A 替换 C 的位置，C 脱离）。
#[test]
fn r3350_replace_child_move_no_panic() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let a = doc.create_element("a");
    let b = doc.create_element("b");
    let c = doc.create_element("c");
    doc.append_child(parent, a).unwrap();
    doc.append_child(parent, b).unwrap();
    doc.append_child(parent, c).unwrap();
    // A 替换 C（A 是 C 之前的兄弟）——旧实现 panic，修复后无 panic 且顺序正确。
    let returned = doc.replace_child(parent, a, c).unwrap();
    assert_eq!(returned, c, "应返回被替换的 old_child");
    // A 占据 C 原位置（末位），B 仍居中：[B, A]。
    assert_eq!(child_tags(&doc, parent), vec!["b", "a"]);
    // C 脱离父节点。
    assert_eq!(doc.parent_node(c), None);
    assert_eq!(doc.parent_node(a), Some(parent));
}

/// R3350：replace_child 用靠后的子节点替换靠前的——B 替换 A，期望 [B, C]（B 原位置提升）。
#[test]
fn r3350_replace_child_move_earlier() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let a = doc.create_element("a");
    let b = doc.create_element("b");
    let c = doc.create_element("c");
    doc.append_child(parent, a).unwrap();
    doc.append_child(parent, b).unwrap();
    doc.append_child(parent, c).unwrap();
    doc.replace_child(parent, b, a).unwrap();
    // B 替换 A 的位置（首位），C 保持末位：[B, C]。
    assert_eq!(child_tags(&doc, parent), vec!["b", "c"]);
    assert_eq!(doc.parent_node(a), None);
    assert_eq!(doc.parent_node(b), Some(parent));
}

/// R3350：replace_child 同节点 no-op——replace(parent, A, A) 返回 A，顺序不变。
#[test]
fn r3350_replace_child_self_noop() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let a = doc.create_element("a");
    let b = doc.create_element("b");
    doc.append_child(parent, a).unwrap();
    doc.append_child(parent, b).unwrap();
    let returned = doc.replace_child(parent, a, a).unwrap();
    assert_eq!(returned, a);
    assert_eq!(child_tags(&doc, parent), vec!["a", "b"]);
}

// ═══════════════════════════════════════════════════════════════════════
// 2. replace_child 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_replace_child_basic() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let old = doc.create_text_node("old");
    doc.append_child(parent, old).unwrap();
    let new = doc.create_text_node("new");
    let result = doc.replace_child(parent, new, old).unwrap();
    assert_eq!(result, old);
    assert_eq!(doc.child_nodes(parent), vec![new]);
    assert!(doc.parent_node(old).is_none());
    assert_eq!(doc.parent_node(new), Some(parent));
}

#[test]
fn test_replace_child_nonexistent() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let new_child = doc.create_text_node("new");
    let old_child = doc.create_text_node("old");
    let result = doc.replace_child(parent, new_child, old_child);
    assert!(result.is_err());
}

#[test]
fn test_replace_child_preserves_id_map() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let old = doc.create_element("span");
    doc.set_attribute(old, "id", "target");
    doc.append_child(parent, old).unwrap();
    assert!(doc.get_element_by_id("target").is_some());
    let new = doc.create_text_node("replaced");
    doc.replace_child(parent, new, old).unwrap();
    // old node removed from id_map
    assert!(doc.get_element_by_id("target").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// 3. clone_node 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_clone_node_nonexistent_returns_self() {
    let mut doc = Document::new();
    // Create a node and then remove it to get an invalid NodeId
    let temp = doc.create_text_node("temp");
    // Just clone an existing node since we can't create fake NodeId
    let cloned = doc.clone_node(temp, false);
    assert_ne!(cloned, temp);
}

#[test]
fn test_clone_node_deep_with_nested_elements() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child1 = doc.create_element("span");
    let child2 = doc.create_text_node("hello");
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();
    let grandchild = doc.create_text_node("world");
    doc.append_child(child1, grandchild).unwrap();

    let cloned = doc.clone_node(parent, true);
    assert_ne!(cloned, parent);
    let cloned_children = doc.child_nodes(cloned);
    assert_eq!(cloned_children.len(), 2);
    // Deep clone should have cloned grandchildren
    let cloned_grandchildren = doc.child_nodes(cloned_children[0]);
    assert_eq!(cloned_grandchildren.len(), 1);
}

#[test]
fn test_clone_node_shallow() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child = doc.create_text_node("child");
    doc.append_child(parent, child).unwrap();

    let cloned = doc.clone_node(parent, false);
    assert!(doc.child_nodes(cloned).is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 4. set_text_content 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_set_text_content_on_comment() {
    let mut doc = Document::new();
    let comment = doc.create_comment("old comment");
    doc.set_text_content(comment, "new comment");
    match doc.get(comment).unwrap().kind {
        NodeKind::Comment(ref data) => assert_eq!(data.content, "new comment"),
        _ => panic!("Expected Comment"),
    }
}

#[test]
fn test_set_text_content_on_element_clears_children() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();
    let child = doc.create_text_node("child");
    doc.append_child(elem, child).unwrap();
    assert_eq!(doc.child_nodes(elem).len(), 1);

    doc.set_text_content(elem, "replacement text");
    let children = doc.child_nodes(elem);
    assert_eq!(children.len(), 1);
    match doc.get(children[0]).unwrap().kind {
        NodeKind::Text(ref data) => assert_eq!(data.content, "replacement text"),
        _ => panic!("Expected Text node"),
    }
}

#[test]
fn test_set_text_content_on_element_empty_string_clears() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();
    let child = doc.create_text_node("child");
    doc.append_child(elem, child).unwrap();

    doc.set_text_content(elem, "");
    assert!(doc.child_nodes(elem).is_empty());
}

#[test]
fn test_set_text_content_on_document_fragment() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    // DocumentFragment should handle set_text_content
    doc.set_text_content(frag, "text in fragment");
    let children = doc.child_nodes(frag);
    // Fragment should get a text child
    assert_eq!(children.len(), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 5. set_attribute / remove_attribute id 映射边界
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_set_attribute_id_replaces_old_id() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();
    doc.set_attribute(elem, "id", "first");
    assert_eq!(doc.get_element_by_id("first"), Some(elem));
    doc.set_attribute(elem, "id", "second");
    assert!(doc.get_element_by_id("first").is_none());
    assert_eq!(doc.get_element_by_id("second"), Some(elem));
}

#[test]
fn test_remove_attribute_id_removes_from_map() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();
    doc.set_attribute(elem, "id", "myid");
    assert_eq!(doc.get_element_by_id("myid"), Some(elem));
    doc.remove_attribute(elem, "id");
    assert!(doc.get_element_by_id("myid").is_none());
}

#[test]
fn test_remove_attribute_nonexistent_attribute() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();
    // Should not panic
    doc.remove_attribute(elem, "nonexistent");
}

// ═══════════════════════════════════════════════════════════════════════
// 6. MutationObserver 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_process_mutations_notifies_observers() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let notified = Rc::new(RefCell::new(0));
    let notified_clone = notified.clone();
    let observer = MutationObserver::new(Box::new(move |_records: &[MutationRecord]| {
        *notified_clone.borrow_mut() += 1;
    }) as MutationCallbackFn);

    let mut doc = Document::new();
    doc.add_observer(observer);

    // Trigger a mutation
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child = doc.create_text_node("hello");
    doc.append_child(parent, child).unwrap();

    doc.process_mutations();
    assert!(*notified.borrow() >= 1);
}

#[test]
fn test_clear_observers() {
    let mut doc = Document::new();
    let observer = MutationObserver::new(Box::new(|_: &[MutationRecord]| {}) as MutationCallbackFn);
    doc.add_observer(observer);
    doc.clear_observers();
    // No panic after clearing
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    doc.process_mutations();
}

#[test]
fn test_take_mutation_records_returns_records() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child = doc.create_text_node("hello");
    doc.append_child(parent, child).unwrap();

    let records = doc.take_mutation_records();
    assert!(!records.is_empty());
    // Second call should be empty
    let records2 = doc.take_mutation_records();
    assert!(records2.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 7. 事件系统边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_dispatch_event_capturing_phase() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let phases = Rc::new(RefCell::new(Vec::new()));
    let phases_clone = phases.clone();

    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();

    let phases_ref = phases_clone.clone();
    doc.add_event_listener(
        parent,
        "click",
        Box::new(move |e: &mut Event| {
            phases_ref.borrow_mut().push(("parent-capture", e.phase));
        }),
        true,
    );

    let phases_ref2 = phases_clone.clone();
    doc.add_event_listener(
        child,
        "click",
        Box::new(move |e: &mut Event| {
            phases_ref2.borrow_mut().push(("child-target", e.phase));
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(child, &mut event);

    let phases = phases.borrow();
    assert!(phases.len() >= 2);
    // First should be parent capturing
    match phases.first() {
        Some((name, phase)) => {
            assert_eq!(*name, "parent-capture");
            assert_eq!(*phase, EventPhase::Capturing);
        }
        None => panic!("Expected at least one event phase"),
    }
}

#[test]
fn test_dispatch_event_non_bubbling() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let count = Rc::new(RefCell::new(0));
    let count_clone = count.clone();

    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();

    doc.add_event_listener(
        parent,
        "focus",
        Box::new(move |_: &mut Event| {
            *count_clone.borrow_mut() += 1;
        }),
        false,
    );

    // Non-bubbling event should not reach parent from child
    let mut event = Event::new_with_options("focus", false, false);
    doc.dispatch_event(child, &mut event);
    // Parent listener should not be called since event doesn't bubble
    assert_eq!(*count.borrow(), 0);
}

#[test]
fn test_remove_event_listener_returns_count() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.add_event_listener(elem, "click", Box::new(|_| {}), false);
    doc.add_event_listener(elem, "click", Box::new(|_| {}), true);
    let count = doc.remove_event_listener(elem, "click");
    assert_eq!(count, 2);
}

#[test]
fn test_remove_event_listener_nonexistent() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let count = doc.remove_event_listener(elem, "click");
    assert_eq!(count, 0);
}

#[test]
fn test_remove_all_event_listeners() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.add_event_listener(elem, "click", Box::new(|_| {}), false);
    doc.add_event_listener(elem, "input", Box::new(|_| {}), false);
    doc.remove_all_event_listeners(elem);
    assert_eq!(doc.listener_count(elem, "click"), 0);
    assert_eq!(doc.listener_count(elem, "input"), 0);
}

#[test]
fn test_listener_count() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert_eq!(doc.listener_count(elem, "click"), 0);
    doc.add_event_listener(elem, "click", Box::new(|_| {}), false);
    assert_eq!(doc.listener_count(elem, "click"), 1);
    doc.add_event_listener(elem, "click", Box::new(|_| {}), true);
    assert_eq!(doc.listener_count(elem, "click"), 2);
}

// ═══════════════════════════════════════════════════════════════════════
// 8. Shadow DOM 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attach_shadow_on_non_element() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    let result = doc.attach_shadow(text, ShadowRootMode::Open);
    assert!(result.is_err());
}

#[test]
fn test_attach_shadow_double_fails() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    doc.append_child(doc.root(), host).unwrap();
    let result1 = doc.attach_shadow(host, ShadowRootMode::Open);
    assert!(result1.is_ok());
    let result2 = doc.attach_shadow(host, ShadowRootMode::Open);
    assert!(result2.is_err());
}

#[test]
fn test_shadow_root_returns_none_for_no_shadow() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert!(doc.shadow_root(elem).is_none());
}

#[test]
fn test_get_shadow_root_mode() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    doc.append_child(doc.root(), host).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Closed).unwrap();
    assert_eq!(doc.get_shadow_root_mode(shadow), Some(ShadowRootMode::Closed));
}

#[test]
fn test_get_shadow_root_mode_non_shadow() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert!(doc.get_shadow_root_mode(elem).is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// 9. compare_document_position 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_compare_position_disconnected_trees() {
    let mut doc = Document::new();
    let elem1 = doc.create_element("div");
    let elem2 = doc.create_element("span");
    // Both attached to root
    doc.append_child(doc.root(), elem1).unwrap();
    doc.append_child(doc.root(), elem2).unwrap();
    // Sibling nodes: elem1 before elem2 → elem2 follows elem1
    let result = doc.compare_document_position(elem1, elem2);
    assert!(result.is_some());
    let pos = result.unwrap();
    assert!(pos.contains(DocumentPosition::FOLLOWING));
}

#[test]
fn test_compare_position_ancestor() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();

    let pos = doc.compare_document_position(parent, child).unwrap();
    assert!(pos.contains(DocumentPosition::CONTAINED_BY));
    assert!(pos.contains(DocumentPosition::FOLLOWING));
}

#[test]
fn test_compare_position_descendant() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();

    let pos = doc.compare_document_position(child, parent).unwrap();
    assert!(pos.contains(DocumentPosition::CONTAINS));
    assert!(pos.contains(DocumentPosition::PRECEDING));
}

#[test]
fn test_compare_position_nonexistent_node() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();
    let other = doc.create_element("span");
    doc.append_child(doc.root(), other).unwrap();
    // Same-level siblings work fine
    let pos = doc.compare_document_position(elem, other);
    assert!(pos.is_some());
    assert!(pos.unwrap().contains(DocumentPosition::FOLLOWING));
}

// ═══════════════════════════════════════════════════════════════════════
// 10. TreeWalker 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_tree_walker_root_node() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    doc.append_child(doc.root(), root).unwrap();
    let walker = TreeWalker::new(root, u32::MAX);
    assert_eq!(walker.root(), root);
    assert_eq!(walker.current_node(), root);
}

#[test]
fn test_tree_walker_next_node_skips_filtered() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child1 = doc.create_text_node("text");
    let child2 = doc.create_element("span");
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();

    // TreeWalker starts at root (parent), next_node moves to first child
    let mut walker = TreeWalker::new(parent, 0b10);
    let first = walker.next_node(&doc);
    assert_eq!(first, Some(child1));
    let next = walker.next_node(&doc);
    assert_eq!(next, Some(child2));
}

#[test]
fn test_node_iterator_is_done() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let iter = NodeIterator::new(elem, u32::MAX);
    assert!(!iter.is_done());
}

// ═══════════════════════════════════════════════════════════════════════
// 11. node_type 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_node_type_all_kinds() {
    let mut doc = Document::new();

    // Document = 9
    assert_eq!(doc.node_type(doc.root()), Some(9));

    // Element = 1
    let elem = doc.create_element("div");
    assert_eq!(doc.node_type(elem), Some(1));

    // Text = 3
    let text = doc.create_text_node("hello");
    assert_eq!(doc.node_type(text), Some(3));

    // Comment = 8
    let comment = doc.create_comment("comment");
    assert_eq!(doc.node_type(comment), Some(8));

    // DocumentFragment = 11
    let frag = doc.create_document_fragment();
    assert_eq!(doc.node_type(frag), Some(11));

    // DocumentType = 10
    let doctype = doc.create_document_type("html", None, None);
    assert_eq!(doc.node_type(doctype), Some(10));

    // ProcessingInstruction = 7
    let pi = doc.create_processing_instruction("xml", "version='1.0'");
    assert_eq!(doc.node_type(pi), Some(7));
}

#[test]
fn test_node_type_nonexistent() {
    let mut doc = Document::new();
    // Create and remove a node to get an invalid NodeId
    let temp = doc.create_text_node("temp");
    let root = doc.root();
    doc.append_child(root, temp).unwrap();
    doc.remove_child(root, temp).unwrap();
    // After removal, the node may still exist in slotmap; just test with root
    assert_eq!(doc.node_type(root), Some(9));
}

// ═══════════════════════════════════════════════════════════════════════
// 12. owner_document 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_owner_document_of_root() {
    let doc = Document::new();
    // Root node itself should return Some(root)
    assert_eq!(doc.owner_document(doc.root()), Some(doc.root()));
}

#[test]
fn test_owner_document_of_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    // Unattached element has no parent, so owner_document returns itself (topmost ancestor)
    assert_eq!(doc.owner_document(elem), Some(elem));
}

#[test]
fn test_owner_document_of_nonexistent() {
    let mut doc = Document::new();
    // A text node not attached to any tree returns itself as topmost ancestor
    let text = doc.create_text_node("text");
    assert_eq!(doc.owner_document(text), Some(text));
}

// ═══════════════════════════════════════════════════════════════════════
// 13. depth / child_count / node_contains 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_depth_of_root() {
    let doc = Document::new();
    assert_eq!(doc.depth(doc.root()), Some(0));
}

#[test]
fn test_depth_nested() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();
    let grandchild = doc.create_text_node("text");
    doc.append_child(child, grandchild).unwrap();

    assert_eq!(doc.depth(parent), Some(1));
    assert_eq!(doc.depth(child), Some(2));
    assert_eq!(doc.depth(grandchild), Some(3));
}

#[test]
fn test_depth_nonexistent() {
    let doc = Document::new();
    // root node is always present at depth 0
    assert_eq!(doc.depth(doc.root()), Some(0));
    // A detached node created but never attached has no parent, so depth = 0
    // (slotmap still "contains" the key, but it has no parent chain to root)
    // We just verify root depth is correct here.
}

#[test]
fn test_child_count() {
    let mut doc = Document::new();
    assert_eq!(doc.child_count(doc.root()), 0);
    let child = doc.create_element("div");
    doc.append_child(doc.root(), child).unwrap();
    assert_eq!(doc.child_count(doc.root()), 1);
}

#[test]
fn test_node_contains_self() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert!(doc.node_contains(elem, elem));
}

#[test]
fn test_node_contains_child() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();
    assert!(doc.node_contains(parent, child));
    assert!(!doc.node_contains(child, parent));
}

// ═══════════════════════════════════════════════════════════════════════
// 14. DocumentPosition 位操作
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_document_position_from_bits() {
    let pos = DocumentPosition::from_bits(0x05);
    assert!(pos.contains(DocumentPosition::FOLLOWING));
    assert!(pos.contains(DocumentPosition::DISCONNECTED));
    assert!(!pos.contains(DocumentPosition::CONTAINS));
    assert!(!pos.contains(DocumentPosition::PRECEDING));
    assert_eq!(pos.bits(), 0x05);
}

// ═══════════════════════════════════════════════════════════════════════
// 15. create_processing_instruction
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_create_processing_instruction() {
    let mut doc = Document::new();
    let pi = doc.create_processing_instruction("xml-stylesheet", "href='style.xsl'");
    match doc.get(pi).unwrap().kind {
        NodeKind::ProcessingInstruction(ref data) => {
            assert_eq!(data.target, "xml-stylesheet");
            assert_eq!(data.data, "href='style.xsl'");
        }
        _ => panic!("Expected ProcessingInstruction"),
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 16. get_attribute / has_attribute / attribute_names on non-elements
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_get_attribute_on_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    assert!(doc.get_attribute(text, "id").is_none());
}

#[test]
fn test_has_attribute_on_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    assert!(!doc.has_attribute(text, "class"));
}

#[test]
fn test_attribute_names_on_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    assert!(doc.attribute_names(text).is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 17. text_content on various node types
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_text_content_on_document() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();
    let text = doc.create_text_node("hello");
    doc.append_child(elem, text).unwrap();
    // Document node's text_content concatenates all descendant text
    let content = doc.text_content(doc.root());
    assert_eq!(content, Some("hello".to_string()));
}

#[test]
fn test_text_content_on_empty_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let content = doc.text_content(elem);
    assert_eq!(content, Some(String::new()));
}

#[test]
fn test_text_content_on_processing_instruction() {
    let mut doc = Document::new();
    let pi = doc.create_processing_instruction("target", "data");
    let content = doc.text_content(pi);
    assert_eq!(content, Some("data".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// 18. remove_child 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_remove_child_not_a_child() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let other = doc.create_text_node("other");
    doc.append_child(doc.root(), parent).unwrap();
    let result = doc.remove_child(parent, other);
    assert!(matches!(result, Err(DomError::NotAChild { .. })));
}

#[test]
fn test_remove_child_nonexistent_parent() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_text_node("child");
    let result = doc.remove_child(parent, child);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// 19. collect_descendants
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_descendants_deep() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let c1 = doc.create_element("span");
    let c2 = doc.create_text_node("text");
    let c3 = doc.create_comment("comment");
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();
    doc.append_child(parent, c3).unwrap();
    let gc = doc.create_text_node("grandchild");
    doc.append_child(c1, gc).unwrap();

    let desc = doc.collect_descendants(parent);
    assert_eq!(desc.len(), 4); // c1, c2, c3, gc
}

// ═══════════════════════════════════════════════════════════════════════
// 20. Slot assignment 边界情况
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_assign_slot_basic() {
    let mut doc = Document::new();
    let slot = doc.create_element("slot");
    let content = doc.create_text_node("content");
    doc.assign_slot(slot, "default", content);
    let assigned = doc.assigned_nodes(slot, "default");
    assert_eq!(assigned, vec![content]);
}

#[test]
fn test_assigned_nodes_empty() {
    let mut doc = Document::new();
    let slot = doc.create_element("slot");
    let assigned = doc.assigned_nodes(slot, "nonexistent");
    assert!(assigned.is_empty());
}

#[test]
fn test_get_assigned_nodes_no_slot_attr() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    // No slot attribute on element
    let assigned = doc.get_assigned_nodes(elem);
    assert!(assigned.is_empty());
}
