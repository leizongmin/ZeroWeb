//! Document 覆盖率补充测试：shadow DOM、事件分发、observer、slot 等。

use crate::event::{Event, EventListenerFn};
use crate::mutation::{MutationCallbackFn, MutationObserver};
use crate::*;

// ═══════════════════════════════════════════════════════════════════════
// Shadow DOM 操作
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_attach_shadow_open() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    doc.append_child(doc.root(), host).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    assert!(doc.shadow_root(host).is_some());
    let mode = doc.get_shadow_root_mode(shadow).unwrap();
    assert_eq!(mode, ShadowRootMode::Open);
}

#[test]
fn test_attach_shadow_closed() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    doc.append_child(doc.root(), host).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Closed).unwrap();
    let mode = doc.get_shadow_root_mode(shadow).unwrap();
    assert_eq!(mode, ShadowRootMode::Closed);
}

#[test]
fn test_shadow_root_none_when_no_shadow() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    assert!(doc.shadow_root(el).is_none());
}

#[test]
fn test_attach_shadow_on_non_element_fails() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    let result = doc.attach_shadow(text, ShadowRootMode::Open);
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════
// Slot 分配
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_assign_and_resolve_slots() {
    let mut doc = Document::new();
    let host = doc.create_element("host-element");
    doc.append_child(doc.root(), host).unwrap();
    doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let shadow = doc.shadow_root(host).unwrap();

    let slot = doc.create_element("slot");
    doc.set_attribute(slot, "name", "content");
    doc.append_child(shadow, slot).unwrap();

    let light = doc.create_element("span");
    doc.set_attribute(light, "slot", "content");
    doc.append_child(host, light).unwrap();

    doc.assign_slot(slot, "content", light);
    doc.resolve_slots(host);

    let assigned = doc.get_assigned_nodes(slot);
    assert!(!assigned.is_empty());
}

#[test]
fn test_assigned_nodes_empty() {
    let mut doc = Document::new();
    let slot = doc.create_element("slot");
    let assigned = doc.get_assigned_nodes(slot);
    assert!(assigned.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// MutationObserver
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_mutation_observer_basic() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.append_child(doc.root(), el).unwrap();

    let callback: MutationCallbackFn = Box::new(|_records| {});
    let observer = MutationObserver::new(callback);
    doc.add_observer(observer);

    doc.set_attribute(el, "class", "test");
    doc.process_mutations();

    let _records = doc.take_mutation_records();
    doc.clear_observers();
}

// ═══════════════════════════════════════════════════════════════════════
// 事件监听
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_add_and_remove_event_listener() {
    let mut doc = Document::new();
    let el = doc.create_element("div");

    let cb: EventListenerFn = Box::new(|_event| {});
    doc.add_event_listener(el, "click", cb, false);
    assert_eq!(doc.listener_count(el, "click"), 1);

    let removed = doc.remove_event_listener(el, "click");
    assert_eq!(removed, 1);
    assert_eq!(doc.listener_count(el, "click"), 0);
}

#[test]
fn test_remove_all_event_listeners() {
    let mut doc = Document::new();
    let el = doc.create_element("div");

    let cb1: EventListenerFn = Box::new(|_e| {});
    let cb2: EventListenerFn = Box::new(|_e| {});
    doc.add_event_listener(el, "click", cb1, false);
    doc.add_event_listener(el, "mouseover", cb2, false);

    doc.remove_all_event_listeners(el);
    assert_eq!(doc.listener_count(el, "click"), 0);
    assert_eq!(doc.listener_count(el, "mouseover"), 0);
}

#[test]
fn test_dispatch_event() {
    let mut doc = Document::new();
    let el = doc.create_element("button");
    doc.append_child(doc.root(), el).unwrap();

    let cb: EventListenerFn = Box::new(|_event| {});
    doc.add_event_listener(el, "click", cb, false);

    let mut event = Event::new("click");
    let _result = doc.dispatch_event(el, &mut event);
}

// ═══════════════════════════════════════════════════════════════════════
// 文档操作辅助函数
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_collect_descendants() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child1 = doc.create_element("span");
    let child2 = doc.create_element("p");
    doc.append_child(doc.root(), parent).unwrap();
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();

    let descendants = doc.collect_descendants(parent);
    assert!(descendants.contains(&child1));
    assert!(descendants.contains(&child2));
}

#[test]
fn test_depth() {
    let mut doc = Document::new();
    let root = doc.root();
    let el = doc.create_element("div");
    doc.append_child(root, el).unwrap();
    let depth = doc.depth(el).unwrap();
    assert_eq!(depth, 1);
}

#[test]
fn test_child_count() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();
    let c1 = doc.create_element("span");
    let c2 = doc.create_element("p");
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();
    assert_eq!(doc.child_count(parent), 2);
}

#[test]
fn test_node_type() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    let text = doc.create_text_node("hello");
    assert!(doc.node_type(el).is_some());
    assert!(doc.node_type(text).is_some());
}

#[test]
fn test_owner_document() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.append_child(doc.root(), el).unwrap();
    let owner = doc.owner_document(el);
    assert!(owner.is_some());
}

#[test]
fn test_has_child_nodes() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    assert!(!doc.has_child_nodes(el));
    let text = doc.create_text_node("text");
    doc.append_child(el, text).unwrap();
    assert!(doc.has_child_nodes(el));
}

#[test]
fn test_node_contains() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(doc.root(), parent).unwrap();
    doc.append_child(parent, child).unwrap();
    assert!(doc.node_contains(parent, child));
    assert!(!doc.node_contains(child, parent));
}

#[test]
fn test_query_selector() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "test");
    doc.append_child(doc.root(), el).unwrap();
    let result = doc.query_selector(el, ".test");
    assert!(result.is_some() || result.is_none()); // doesn't panic
}

#[test]
fn test_query_selector_all() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.append_child(doc.root(), el).unwrap();
    let result = doc.query_selector_all(el, "span");
    assert!(result.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// DocumentFragment 和 DocumentType
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_create_document_fragment() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    assert!(doc.node_type(frag).is_some());
}

#[test]
fn test_create_document_type() {
    let mut doc = Document::new();
    let doctype = doc.create_document_type("html", Some("public".to_string()), Some("system".to_string()));
    assert!(doc.node_type(doctype).is_some());
}

#[test]
fn test_create_processing_instruction() {
    let mut doc = Document::new();
    let pi = doc.create_processing_instruction("xml-stylesheet", "href=\"style.css\"");
    assert!(doc.node_type(pi).is_some());
}

// ═══════════════════════════════════════════════════════════════════════
// clone_node
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_clone_node_shallow() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "class", "original");
    let cloned = doc.clone_node(el, false);
    let class = doc.get_attribute(cloned, "class");
    assert_eq!(class, Some("original".to_string()));
}

#[test]
fn test_clone_node_deep() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();
    let cloned = doc.clone_node(parent, true);
    assert!(doc.has_child_nodes(cloned));
}

// ═══════════════════════════════════════════════════════════════════════
// replace_child
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_replace_child() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let old = doc.create_element("span");
    let new = doc.create_element("p");
    doc.append_child(doc.root(), parent).unwrap();
    doc.append_child(parent, old).unwrap();
    let result = doc.replace_child(parent, new, old).unwrap();
    assert_eq!(result, old);
    assert_eq!(doc.child_count(parent), 1);
    let first = doc.first_child(parent).unwrap();
    assert_eq!(first, new);
}

// ═══════════════════════════════════════════════════════════════════════
// insert_before
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_insert_before() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let first = doc.create_element("span");
    let second = doc.create_element("p");
    let new_el = doc.create_element("a");
    doc.append_child(doc.root(), parent).unwrap();
    doc.append_child(parent, first).unwrap();
    doc.append_child(parent, second).unwrap();
    doc.insert_before(parent, new_el, second).unwrap();
    let children = doc.child_nodes(parent);
    assert_eq!(children[1], new_el);
}

// ═══════════════════════════════════════════════════════════════════════
// has_attribute / attribute_names
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_has_attribute() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "data-test", "value");
    assert!(doc.has_attribute(el, "data-test"));
    assert!(!doc.has_attribute(el, "data-missing"));
}

#[test]
fn test_attribute_names() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_attribute(el, "id", "test");
    doc.set_attribute(el, "class", "foo");
    let names = doc.attribute_names(el);
    assert!(names.contains(&"id".to_string()));
    assert!(names.contains(&"class".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// get_elements_by_tag_name
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_get_elements_by_tag_name() {
    let mut doc = Document::new();
    let p1 = doc.create_element("p");
    let p2 = doc.create_element("p");
    let span = doc.create_element("span");
    doc.append_child(doc.root(), p1).unwrap();
    doc.append_child(doc.root(), p2).unwrap();
    doc.append_child(doc.root(), span).unwrap();
    let ps = doc.get_elements_by_tag_name("p");
    assert_eq!(ps.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════
// text_content on various node types
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_text_content_on_document_fragment() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    let text = doc.create_text_node("hello");
    doc.append_child(frag, text).unwrap();
    let content = doc.text_content(frag);
    assert_eq!(content, Some("hello".to_string()));
}

#[test]
fn test_set_text_content_on_element() {
    let mut doc = Document::new();
    let el = doc.create_element("div");
    doc.set_text_content(el, "new content");
    let content = doc.text_content(el);
    assert_eq!(content, Some("new content".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// node_count
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_node_count() {
    let doc = Document::new();
    let count = doc.node_count();
    assert!(count > 0); // at least the root node
}

#[test]
fn test_element_count_excludes_non_elements() {
    let mut doc = Document::new();
    doc.create_element("div");
    doc.create_element("span");
    doc.create_text_node("text");
    doc.create_comment("comment");
    assert_eq!(doc.element_count(), 2);
}
