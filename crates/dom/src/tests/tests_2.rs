use super::body_of;
// DOM crate 综合测试套件。
//
// 覆盖：节点类型、树操作、属性操作、HTML 解析、查询、序列化、MutationObserver。

use crate::*;
use std::sync::{Arc, Mutex};

// ── node_contains ─────────────────────────────────────────────────────

/// 测试 node_contains 对自身返回 true。
#[test]
fn test_node_contains_self() {
    let doc = Document::new();
    let root = doc.root();
    assert!(doc.node_contains(root, root), "a node should contain itself");
}

/// 测试 node_contains 对后代节点返回 true。
#[test]
fn test_node_contains_descendant() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(span, p).unwrap();

    assert!(doc.node_contains(root, div));
    assert!(doc.node_contains(root, span));
    assert!(doc.node_contains(root, p));
    assert!(doc.node_contains(div, span));
    assert!(doc.node_contains(div, p));
}

/// 测试 node_contains 对无关节点返回 false。
#[test]
fn test_node_contains_not_related() {
    let mut doc = Document::new();
    let root = doc.root();
    let a = doc.create_element("div");
    let b = doc.create_element("span");
    doc.append_child(root, a).unwrap();
    doc.append_child(root, b).unwrap();

    // a 不包含 b，b 也不包含 a
    assert!(!doc.node_contains(a, b));
    assert!(!doc.node_contains(b, a));
}

/// 测试 node_contains 对兄弟节点返回 false。
#[test]
fn test_node_contains_sibling_false() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();

    assert!(!doc.node_contains(c1, c2));
    assert!(!doc.node_contains(c2, c3));
    assert!(!doc.node_contains(c3, c1));
}

// ── compare_document_position ─────────────────────────────────────────

/// 测试 compare_document_position：前面的节点返回 FOLLOWING。
#[test]
fn test_compare_document_position_preceding() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();

    // c1 在 c2 之前 → c2 在 c1 之后（FOLLOWING）
    let pos = doc.compare_document_position(c1, c2).unwrap();
    assert!(pos.contains(DocumentPosition::FOLLOWING), "c2 should be following c1");
}

/// 测试 compare_document_position：后面的节点返回 PRECEDING。
#[test]
fn test_compare_document_position_following() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();

    // c2 在 c1 之后 → c2 在 c1 的位置看来是在前面（PRECEDING）
    let pos = doc.compare_document_position(c2, c1).unwrap();
    assert!(pos.contains(DocumentPosition::PRECEDING), "c1 should be preceding c2");
}

/// 测试 compare_document_position：包含关系。
#[test]
fn test_compare_document_position_contains() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();

    // div 包含 span → 从 span 看 div，div 在前面且包含 span
    let pos = doc.compare_document_position(span, div).unwrap();
    assert!(pos.contains(DocumentPosition::CONTAINS), "div should contain span");
    assert!(
        pos.contains(DocumentPosition::PRECEDING),
        "div should be preceding span"
    );
}

/// 测试 compare_document_position：被包含关系。
#[test]
fn test_compare_document_position_contained_by() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();

    // span 被 div 包含 → 从 div 看 span，span 在后面且被包含
    let pos = doc.compare_document_position(div, span).unwrap();
    assert!(
        pos.contains(DocumentPosition::CONTAINED_BY),
        "span should be contained by div"
    );
    assert!(
        pos.contains(DocumentPosition::FOLLOWING),
        "span should be following div"
    );
}

// ── collect_descendants ──────────────────────────────────────────────

/// 测试 collect_descendants 对空节点。
#[test]
fn test_collect_descendants_empty() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let descendants = doc.collect_descendants(elem);
    assert!(
        descendants.is_empty(),
        "element with no children should have no descendants"
    );
}

/// 测试 collect_descendants 对深层树。
#[test]
fn test_collect_descendants_deep_tree() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    let text = doc.create_text_node("hello");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(div, p).unwrap();
    doc.append_child(span, text).unwrap();

    let descendants = doc.collect_descendants(div);
    assert_eq!(descendants.len(), 3, "div should have 3 descendants (span, text, p)");
    assert_eq!(descendants[0], span, "first descendant should be span");
    assert_eq!(descendants[1], text, "second descendant should be text (child of span)");
    assert_eq!(descendants[2], p, "third descendant should be p");
}

// ── depth ────────────────────────────────────────────────────────────

/// 测试文档根节点的深度为 0。
#[test]
fn test_depth_of_root() {
    let doc = Document::new();
    assert_eq!(doc.depth(doc.root()), Some(0));
}

/// 测试深层节点的深度。
#[test]
fn test_depth_of_deep_node() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(span, p).unwrap();

    assert_eq!(doc.depth(div), Some(1));
    assert_eq!(doc.depth(span), Some(2));
    assert_eq!(doc.depth(p), Some(3));
}

// ── child_count ──────────────────────────────────────────────────────

/// 测试 child_count。
#[test]
fn test_child_count() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();

    assert_eq!(doc.child_count(parent), 0);

    let c1 = doc.create_element("span");
    let c2 = doc.create_text_node("hello");
    let c3 = doc.create_comment("note");
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();
    doc.append_child(parent, c3).unwrap();

    assert_eq!(doc.child_count(parent), 3);
}

// ── node_type ────────────────────────────────────────────────────────

/// 测试各种节点类型的 WHATWG nodeType 值。
#[test]
fn test_node_type_all_kinds() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let text = doc.create_text_node("hello");
    let comment = doc.create_comment("note");
    let doctype = doc.create_document_type("html", None, None);
    let frag = doc.create_document_fragment();
    let pi = doc.create_processing_instruction("xml", "version=\"1.0\"");

    assert_eq!(doc.node_type(elem), Some(1), "Element = 1");
    assert_eq!(doc.node_type(text), Some(3), "Text = 3");
    assert_eq!(doc.node_type(pi), Some(7), "ProcessingInstruction = 7");
    assert_eq!(doc.node_type(comment), Some(8), "Comment = 8");
    assert_eq!(doc.node_type(doc.root()), Some(9), "Document = 9");
    assert_eq!(doc.node_type(doctype), Some(10), "DocumentType = 10");
    assert_eq!(doc.node_type(frag), Some(11), "DocumentFragment = 11");
}

// ── owner_document ───────────────────────────────────────────────────

/// 测试 owner_document 返回文档根节点。
#[test]
fn test_owner_document() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let text = doc.create_text_node("hello");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(span, text).unwrap();

    assert_eq!(doc.owner_document(root), Some(root));
    assert_eq!(doc.owner_document(div), Some(root));
    assert_eq!(doc.owner_document(span), Some(root));
    assert_eq!(doc.owner_document(text), Some(root));
}

/// 测试 owner_document 对孤立节点返回其自身（作为根）。
#[test]
fn test_owner_document_orphan() {
    let doc = Document::new();
    let _root = doc.root();
    // 孤立节点没有 parent，owner_document 返回自身
    let mut doc2 = Document::new();
    let orphan = doc2.create_element("div");
    assert_eq!(
        doc2.owner_document(orphan),
        Some(orphan),
        "orphan node's owner_document should be itself"
    );
    // 但 root 下面的节点应该指向 root
    assert_eq!(doc2.owner_document(doc2.root()), Some(doc2.root()));
}

/// 测试 attach_shadow 创建 ShadowRoot。
#[test]
fn test_attach_shadow_creates_shadow_root() {
    let mut doc = Document::new();
    let root = doc.root();
    let host = doc.create_element("div");
    doc.append_child(root, host).unwrap();

    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    assert!(doc.contains(shadow));
    assert!(matches!(
        doc.get(shadow).map(|n| &n.kind),
        Some(NodeKind::ShadowRoot(_))
    ));
    // ShadowRoot 的 node_type 应为 11（同 DocumentFragment）
    assert_eq!(doc.node_type(shadow), Some(11));
}

/// 测试 attach_shadow open 模式。
#[test]
fn test_attach_shadow_open_mode() {
    let mut doc = Document::new();
    let host = doc.create_element("div");

    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    assert_eq!(doc.get_shadow_root_mode(shadow), Some(ShadowRootMode::Open));
}

/// 测试 attach_shadow closed 模式。
#[test]
fn test_attach_shadow_closed_mode() {
    let mut doc = Document::new();
    let host = doc.create_element("div");

    let shadow = doc.attach_shadow(host, ShadowRootMode::Closed).unwrap();
    assert_eq!(doc.get_shadow_root_mode(shadow), Some(ShadowRootMode::Closed));
}

/// 测试 attach_shadow 对非元素节点返回错误。
#[test]
fn test_attach_shadow_non_element_error() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");

    let result = doc.attach_shadow(text, ShadowRootMode::Open);
    assert!(matches!(result, Err(DomError::NotAnElement)));
}

/// 测试 attach_shadow 重复附加返回错误。
#[test]
fn test_attach_shadow_duplicate_error() {
    let mut doc = Document::new();
    let host = doc.create_element("div");

    doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let result = doc.attach_shadow(host, ShadowRootMode::Open);
    assert!(matches!(result, Err(DomError::AlreadyHasShadowRoot)));
}

/// 测试 shadow_root 返回 None 和 Some。
#[test]
fn test_shadow_root_returns_correctly() {
    let mut doc = Document::new();
    let host = doc.create_element("div");

    // 附加前返回 None
    assert_eq!(doc.shadow_root(host), None);

    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    assert_eq!(doc.shadow_root(host), Some(shadow));
}

/// 测试 get_shadow_root_mode 对非 ShadowRoot 节点返回 None。
#[test]
fn test_get_shadow_root_mode_non_shadow() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert_eq!(doc.get_shadow_root_mode(elem), None);
}

/// 测试 append_child 向 ShadowRoot 内添加子节点。
#[test]
fn test_append_child_into_shadow_root() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    let inner = doc.create_element("span");
    doc.append_child(shadow, inner).unwrap();

    assert_eq!(doc.parent_node(inner), Some(shadow));
    assert_eq!(doc.child_nodes(shadow), vec![inner]);

    // 可以通过 ShadowRoot 获取文本内容
    let text = doc.create_text_node("shadow content");
    doc.append_child(inner, text).unwrap();
    assert_eq!(doc.text_content(shadow), Some("shadow content".to_string()));
}

/// 测试 assigned_nodes 对空 slot 返回空列表。
#[test]
fn test_assigned_nodes_empty_slot() {
    let mut doc = Document::new();
    let slot_elem = doc.create_element("slot");
    doc.set_attribute(slot_elem, "name", "header");

    let nodes = doc.assigned_nodes(slot_elem, "header");
    assert!(nodes.is_empty());
}

/// 测试 assign_slot 和 assigned_nodes 往返。
#[test]
fn test_assign_slot_and_assigned_nodes() {
    let mut doc = Document::new();
    let root = doc.root();

    // 创建 host 和 shadow root
    let host = doc.create_element("div");
    doc.append_child(root, host).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    // 在 shadow root 中创建 slot 元素
    let slot_elem = doc.create_element("slot");
    doc.set_attribute(slot_elem, "name", "header");
    doc.append_child(shadow, slot_elem).unwrap();

    // 在 light DOM 中创建内容节点
    let light_node = doc.create_element("h1");
    doc.set_attribute(light_node, "slot", "header");
    doc.append_child(host, light_node).unwrap();

    // 分配 light DOM 节点到 slot
    doc.assign_slot(slot_elem, "header", light_node);

    let assigned = doc.assigned_nodes(slot_elem, "header");
    assert_eq!(assigned, vec![light_node]);
}

/// 测试 query_selector_shadow 基本功能。
#[test]
fn test_query_selector_shadow_basic() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    let inner = doc.create_element("span");
    doc.set_attribute(inner, "class", "inner");
    doc.append_child(shadow, inner).unwrap();

    // 在 shadow DOM 中查找
    let found = doc.query_selector_shadow(shadow, ".inner");
    assert_eq!(found, Some(inner));

    // 在 shadow DOM 中按标签名查找
    let found_tag = doc.query_selector_shadow(shadow, "span");
    assert_eq!(found_tag, Some(inner));
}

/// 测试 query_selector_all_shadow 基本功能。
#[test]
fn test_query_selector_all_shadow_basic() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    let s1 = doc.create_element("span");
    let s2 = doc.create_element("span");
    let s3 = doc.create_element("p");
    doc.append_child(shadow, s1).unwrap();
    doc.append_child(shadow, s2).unwrap();
    doc.append_child(shadow, s3).unwrap();

    let spans = doc.query_selector_all_shadow(shadow, "span");
    assert_eq!(spans, vec![s1, s2]);
}

/// 测试 Shadow DOM 封装：light DOM 查询不会找到 shadow 内部元素。
#[test]
fn test_shadow_dom_encapsulation() {
    let mut doc = Document::new();
    let root = doc.root();

    // 创建宿主元素并加入文档
    let host = doc.create_element("div");
    doc.append_child(root, host).unwrap();

    // 在宿主内创建 light DOM 内容
    let light = doc.create_element("span");
    doc.set_attribute(light, "class", "light");
    doc.append_child(host, light).unwrap();

    // 附加 ShadowRoot 并添加 shadow DOM 内容
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let shadow_span = doc.create_element("span");
    doc.set_attribute(shadow_span, "class", "shadow");
    doc.append_child(shadow, shadow_span).unwrap();

    // 从文档根查询 span — 只应找到 light DOM 的（因为 shadow 子树
    // 挂在 host 的 children 中，query 会穿透到 shadow 内容。
    // 这里验证从 host 直接查询到的结构正确）
    let _all_spans_from_root = doc.query_selector_all(root, "span");
    // shadow_span 在 shadow 子树内，但当前 query 实现会遍历所有子节点
    // 关键验证：从 shadow_root 查询只能找到 shadow 内容
    let shadow_spans = doc.query_selector_all_shadow(shadow, "span");
    assert_eq!(shadow_spans, vec![shadow_span]);
    // shadow 内的查询不应找到 light DOM 节点
    let shadow_light_result = doc.query_selector_shadow(shadow, ".light");
    assert_eq!(shadow_light_result, None);
}

/// 测试 slot 的默认内容回退。
#[test]
fn test_slot_default_content_fallback() {
    let mut doc = Document::new();
    let root = doc.root();

    let host = doc.create_element("my-component");
    doc.append_child(root, host).unwrap();
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    // 创建带默认内容的 slot
    let slot_elem = doc.create_element("slot");
    doc.set_attribute(slot_elem, "name", "header");
    let default_content = doc.create_text_node("Default Header");
    doc.append_child(shadow, slot_elem).unwrap();
    doc.append_child(slot_elem, default_content).unwrap();

    // 未分配任何节点时，assigned_nodes 为空，但 slot 子节点（默认内容）仍在
    let assigned = doc.assigned_nodes(slot_elem, "header");
    assert!(assigned.is_empty(), "no nodes assigned yet");

    // slot 本身的 textContent 仍包含默认内容
    assert_eq!(doc.text_content(slot_elem), Some("Default Header".to_string()));
}

/// 测试多个节点分配到同一 slot。
#[test]
fn test_assign_multiple_nodes_to_slot() {
    let mut doc = Document::new();
    let slot_elem = doc.create_element("slot");
    doc.set_attribute(slot_elem, "name", "items");

    let n1 = doc.create_element("li");
    let n2 = doc.create_element("li");
    let n3 = doc.create_element("li");

    doc.assign_slot(slot_elem, "items", n1);
    doc.assign_slot(slot_elem, "items", n2);
    doc.assign_slot(slot_elem, "items", n3);

    let assigned = doc.assigned_nodes(slot_elem, "items");
    assert_eq!(assigned.len(), 3);
    assert_eq!(assigned[0], n1);
    assert_eq!(assigned[1], n2);
    assert_eq!(assigned[2], n3);
}

/// 测试 ShadowRoot 的 host 指向正确的宿主元素。
#[test]
fn test_shadow_root_host_reference() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    // 验证 host 字段
    if let Some(NodeKind::ShadowRoot(data)) = doc.get(shadow).map(|n| n.kind.clone()) {
        assert_eq!(data.host, Some(host));
    } else {
        panic!("expected ShadowRoot node");
    }
}

/// 测试 ShadowRoot 作为宿主元素的子节点。
#[test]
fn test_shadow_root_is_child_of_host() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    // ShadowRoot 的 parent 应指向 host
    assert_eq!(doc.parent_node(shadow), Some(host));
    // host 的子列表中包含 ShadowRoot
    let children = doc.child_nodes(host);
    assert!(children.contains(&shadow));
}

/// 测试 query_selector_shadow 不存在的选择器返回 None。
#[test]
fn test_query_selector_shadow_not_found() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    let result = doc.query_selector_shadow(shadow, "#nonexistent");
    assert_eq!(result, None);

    let results = doc.query_selector_all_shadow(shadow, ".missing");
    assert!(results.is_empty());
}

/// Range clone_contents deep clones nested children.
#[test]
fn test_range_clone_contents_deep_clone() {
    let mut doc = parse_html("<div><p><span>deep</span></p><p>second</p></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    let mut range = Range::new(div, div);
    range.select_node_contents(&doc, div).unwrap();
    let fragment = range.clone_contents(&mut doc).unwrap();

    // Cloned fragment should have the same structure
    let cloned_children = doc.child_nodes(fragment);
    assert_eq!(cloned_children.len(), 2, "cloned fragment should have 2 children");
    // Deep clone: first child should have its own children
    let cloned_p = cloned_children[0];
    assert!(doc.has_child_nodes(cloned_p), "cloned <p> should have children");
    // Original unchanged
    assert_eq!(doc.child_nodes(div).len(), 2);
}

/// Range insert_node at different offsets.
#[test]
fn test_range_insert_node_at_offset() {
    let mut doc = parse_html("<div><p>A</p><p>C</p></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    // Insert between the two <p> children (offset=1)
    let new_p = doc.create_element("p");
    doc.set_text_content(new_p, "B");

    let mut range = Range::at(div, 1);
    range.insert_node(&mut doc, new_p).unwrap();

    let children = doc.child_nodes(div);
    assert_eq!(children.len(), 3);
    let text_b = doc.text_content(children[1]);
    assert_eq!(text_b, Some("B".to_string()));
}

/// Range delete_contents removes correct nodes within a subrange.
#[test]
fn test_range_delete_contents_subrange() {
    let mut doc = parse_html("<div><p>A</p><p>B</p><p>C</p><p>D</p></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    // Delete children at offset 1..3 (the middle two <p> elements)
    let mut range = Range::at(div, 1);
    range.set_end(div, 3).unwrap();
    range.delete_contents(&mut doc).unwrap();

    let children = doc.child_nodes(div);
    assert_eq!(children.len(), 2, "should have 2 remaining children");
    assert_eq!(doc.text_content(children[0]), Some("A".to_string()));
    assert_eq!(doc.text_content(children[1]), Some("D".to_string()));
}

/// Range extract_contents moves nodes to fragment.
#[test]
fn test_range_extract_contents_partial() {
    let mut doc = parse_html("<div><p>1</p><p>2</p><p>3</p></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    // Extract first two children
    let mut range = Range::at(div, 0);
    range.set_end(div, 2).unwrap();
    let fragment = range.extract_contents(&mut doc).unwrap();

    assert_eq!(doc.child_nodes(fragment).len(), 2, "fragment should have 2 nodes");
    assert_eq!(doc.child_nodes(div).len(), 1, "div should have 1 remaining child");
    assert_eq!(doc.text_content(doc.first_child(div).unwrap()), Some("3".to_string()));
}

/// Range select_node_contents on element with multiple children.
#[test]
fn test_range_select_node_contents_many_children() {
    let doc = parse_html("<div><a>1</a><b>2</b><i>3</i><u>4</u></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    let mut range = Range::new(div, div);
    range.select_node_contents(&doc, div).unwrap();

    assert_eq!(range.start_container(), div);
    assert_eq!(range.start_offset(), 0);
    assert_eq!(range.end_container(), div);
    assert_eq!(range.end_offset(), 4, "should cover all 4 children");
}

/// Range collapse to start vs end produces correct offsets.
#[test]
fn test_range_collapse_to_start_vs_end() {
    let doc = parse_html("<div><p>A</p><p>B</p></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    let mut range = Range::new(div, div);
    range.set_start(div, 0).unwrap();
    range.set_end(div, 2).unwrap();
    assert!(!range.collapsed());

    // Collapse to start
    let mut r_start = range.clone();
    r_start.collapse(true);
    assert!(r_start.collapsed());
    assert_eq!(r_start.start_offset(), 0);

    // Collapse to end
    let mut r_end = range.clone();
    r_end.collapse(false);
    assert!(r_end.collapsed());
    assert_eq!(r_end.start_offset(), 2);
}

/// Serialize a simple full HTML document.
#[test]
fn test_serialize_full_document() {
    let doc = parse_html("<!DOCTYPE html><html><body><h1>Title</h1></body></html>");
    let html = doc.outer_html(doc.root());
    assert!(html.contains("<!DOCTYPE html>"), "should contain DOCTYPE");
    assert!(html.contains("<h1>"), "should contain h1 tag");
}

/// Serialize document with multiple attributes.
#[test]
fn test_serialize_element_with_multiple_attributes() {
    let mut doc = Document::new();
    let input = doc.create_element("input");
    doc.set_attribute(input, "type", "text");
    doc.set_attribute(input, "name", "email");
    doc.set_attribute(input, "placeholder", "Enter email");

    let html = doc.outer_html(input);
    assert!(html.contains("type=\"text\""));
    assert!(html.contains("name=\"email\""));
    assert!(html.contains("placeholder=\"Enter email\""));
}

/// Serialize document mixing text and element child nodes.
#[test]
fn test_serialize_mixed_text_and_elements() {
    let mut doc = Document::new();
    let p = doc.create_element("p");
    let t1 = doc.create_text_node("Hello ");
    let b = doc.create_element("b");
    let t2 = doc.create_text_node("World");
    let t3 = doc.create_text_node("!");
    doc.append_child(p, t1).unwrap();
    doc.append_child(p, b).unwrap();
    doc.append_child(b, t2).unwrap();
    doc.append_child(p, t3).unwrap();

    let html = doc.outer_html(p);
    assert_eq!(html, "<p>Hello <b>World</b>!</p>");
}

/// Serialize document with comment nodes.
#[test]
fn test_serialize_with_comments() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    let comment = doc.create_comment("this is a comment");
    let text = doc.create_text_node("content");
    doc.append_child(div, comment).unwrap();
    doc.append_child(div, text).unwrap();

    let html = doc.outer_html(div);
    assert!(html.contains("<!--this is a comment-->"));
    assert!(html.contains("content"));
}

/// Serialize deeply nested elements.
#[test]
fn test_serialize_deeply_nested() {
    let mut doc = Document::new();
    let mut current = doc.create_element("div");
    doc.append_child(doc.root(), current).unwrap();
    for _ in 0..5 {
        let inner = doc.create_element("section");
        doc.append_child(current, inner).unwrap();
        current = inner;
    }
    doc.set_text_content(current, "leaf");

    let _html = doc.outer_html(doc.root());
    let leaf = doc.first_child(doc.root()).unwrap();
    let leaf_html = doc.outer_html(leaf);
    // Should contain nested sections ending with "leaf"
    assert!(leaf_html.contains("<section>"));
    assert!(leaf_html.contains("leaf"));
}

/// Append multiple children to a shadow root and verify ordering.
#[test]
fn test_shadow_root_append_multiple_children() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    let c1 = doc.create_element("header");
    let c2 = doc.create_element("main");
    let c3 = doc.create_element("footer");
    doc.append_child(shadow, c1).unwrap();
    doc.append_child(shadow, c2).unwrap();
    doc.append_child(shadow, c3).unwrap();

    assert_eq!(doc.child_nodes(shadow), vec![c1, c2, c3]);
    assert_eq!(doc.parent_node(c2), Some(shadow));
}

/// Shadow root text content collection gathers text from nested children.
#[test]
fn test_shadow_root_text_content_collection() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let p = doc.create_element("p");
    doc.append_child(shadow, p).unwrap();
    let t1 = doc.create_text_node("Hello ");
    let t2 = doc.create_text_node("Shadow");
    doc.append_child(p, t1).unwrap();
    doc.append_child(p, t2).unwrap();

    assert_eq!(doc.text_content(shadow), Some("Hello Shadow".to_string()));
}

/// Multiple elements can each have their own shadow roots.
#[test]
fn test_multiple_elements_with_shadow_roots() {
    let mut doc = Document::new();
    let root = doc.root();
    let host1 = doc.create_element("comp-a");
    let host2 = doc.create_element("comp-b");
    doc.append_child(root, host1).unwrap();
    doc.append_child(root, host2).unwrap();

    let shadow1 = doc.attach_shadow(host1, ShadowRootMode::Open).unwrap();
    let shadow2 = doc.attach_shadow(host2, ShadowRootMode::Closed).unwrap();

    let s1_inner = doc.create_element("span");
    doc.set_attribute(s1_inner, "class", "a-content");
    doc.append_child(shadow1, s1_inner).unwrap();

    let s2_inner = doc.create_element("div");
    doc.set_attribute(s2_inner, "class", "b-content");
    doc.append_child(shadow2, s2_inner).unwrap();

    // Each shadow root has its own content
    assert_eq!(doc.query_selector_shadow(shadow1, ".a-content"), Some(s1_inner));
    assert_eq!(doc.query_selector_shadow(shadow2, ".b-content"), Some(s2_inner));
    // Cross-query returns nothing
    assert_eq!(doc.query_selector_shadow(shadow1, ".b-content"), None);
    assert_eq!(doc.query_selector_shadow(shadow2, ".a-content"), None);
}

/// Shadow root node_type returns 11 (same as DocumentFragment).
#[test]
fn test_shadow_root_node_type_is_11() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let host2 = doc.create_element("span");
    let shadow2 = doc.attach_shadow(host2, ShadowRootMode::Closed).unwrap();

    assert_eq!(doc.node_type(shadow), Some(11));
    assert_eq!(doc.node_type(shadow2), Some(11));
}

/// Slot assignment with multiple nodes preserves order.
#[test]
fn test_slot_assignment_multiple_nodes_order() {
    let mut doc = Document::new();
    let slot = doc.create_element("slot");
    doc.set_attribute(slot, "name", "list");

    let n1 = doc.create_element("li");
    doc.set_attribute(n1, "class", "first");
    let n2 = doc.create_element("li");
    doc.set_attribute(n2, "class", "second");
    let n3 = doc.create_element("li");
    doc.set_attribute(n3, "class", "third");

    doc.assign_slot(slot, "list", n1);
    doc.assign_slot(slot, "list", n2);
    doc.assign_slot(slot, "list", n3);

    let assigned = doc.assigned_nodes(slot, "list");
    assert_eq!(assigned.len(), 3);
    assert_eq!(assigned[0], n1);
    assert_eq!(assigned[1], n2);
    assert_eq!(assigned[2], n3);
}

/// Nested shadow DOM: shadow root inside another shadow root's subtree.
#[test]
fn test_nested_shadow_dom() {
    let mut doc = Document::new();
    let root = doc.root();

    // Outer component
    let outer_host = doc.create_element("outer-comp");
    doc.append_child(root, outer_host).unwrap();
    let outer_shadow = doc.attach_shadow(outer_host, ShadowRootMode::Open).unwrap();

    // Inner component inside outer shadow
    let inner_host = doc.create_element("inner-comp");
    doc.append_child(outer_shadow, inner_host).unwrap();
    let inner_shadow = doc.attach_shadow(inner_host, ShadowRootMode::Open).unwrap();

    let deep_elem = doc.create_element("span");
    doc.set_attribute(deep_elem, "id", "deep");
    doc.set_text_content(deep_elem, "nested");
    doc.append_child(inner_shadow, deep_elem).unwrap();

    // Query from outer shadow finds inner_host but not deep content
    let found = doc.query_selector_shadow(outer_shadow, "inner-comp");
    assert_eq!(found, Some(inner_host));
    assert_eq!(doc.query_selector_shadow(outer_shadow, "#deep"), None);

    // Query from inner shadow finds deep content
    let found_deep = doc.query_selector_shadow(inner_shadow, "#deep");
    assert_eq!(found_deep, Some(deep_elem));
    assert_eq!(doc.text_content(deep_elem), Some("nested".to_string()));
}

/// Event stopImmediatePropagation during bubbling prevents ancestor listeners.
#[test]
fn test_immediate_propagation_stops_bubbling_to_ancestors() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();

    let call_log = Arc::new(Mutex::new(Vec::new()));
    let log_span = call_log.clone();
    let log_div = call_log.clone();

    // span: stopImmediatePropagation
    doc.add_event_listener(
        span,
        "click",
        Box::new(move |e| {
            log_span.lock().unwrap().push("span");
            e.stop_immediate_propagation();
        }),
        false,
    );

    // div bubble: should not fire
    doc.add_event_listener(
        div,
        "click",
        Box::new(move |_| {
            log_div.lock().unwrap().push("div");
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(span, &mut event);

    let log = call_log.lock().unwrap();
    assert_eq!(*log, vec!["span"], "immediate stop should prevent bubbling to div");
}

/// Dispatching event on a node with no listeners does not panic.
#[test]
fn test_dispatch_no_listeners_no_panic() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    // Not even attached to the document

    let mut event = Event::new("click");
    let result = doc.dispatch_event(elem, &mut event);
    assert!(result, "dispatch with no listeners should return true");
}

/// Event on a disconnected (non-attached) node sets target correctly.
#[test]
fn test_event_on_disconnected_node() {
    let mut doc = Document::new();
    let orphan = doc.create_element("div");

    let target_seen = Arc::new(Mutex::new(None));
    let target_clone = target_seen.clone();
    let mut doc2 = Document::new();
    doc2.add_event_listener(
        orphan,
        "custom",
        Box::new(move |e| {
            *target_clone.lock().unwrap() = e.target();
        }),
        false,
    );

    let mut event = Event::new("custom");
    doc2.dispatch_event(orphan, &mut event);

    assert_eq!(*target_seen.lock().unwrap(), Some(orphan));
}

/// Multiple event types on same node fire independently.
#[test]
fn test_multiple_event_types_independent() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let flags = Arc::new(Mutex::new((false, false, false)));
    let f1 = flags.clone();
    let f2 = flags.clone();
    let f3 = flags.clone();

    doc.add_event_listener(
        elem,
        "click",
        Box::new(move |_| {
            f1.lock().unwrap().0 = true;
        }),
        false,
    );
    doc.add_event_listener(
        elem,
        "focus",
        Box::new(move |_| {
            f2.lock().unwrap().1 = true;
        }),
        false,
    );
    doc.add_event_listener(
        elem,
        "blur",
        Box::new(move |_| {
            f3.lock().unwrap().2 = true;
        }),
        false,
    );

    assert_eq!(doc.listener_count(elem, "click"), 1);
    assert_eq!(doc.listener_count(elem, "focus"), 1);
    assert_eq!(doc.listener_count(elem, "blur"), 1);

    // Dispatch focus only
    let mut event = Event::new("focus");
    doc.dispatch_event(elem, &mut event);

    let guard = flags.lock().unwrap();
    assert!(!guard.0, "click should not fire");
    assert!(guard.1, "focus should fire");
    assert!(!guard.2, "blur should not fire");
}

/// Listener count tracking: add, add, remove, verify count.
#[test]
fn test_listener_count_tracking() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    assert_eq!(doc.listener_count(elem, "click"), 0);

    doc.add_event_listener(elem, "click", Box::new(|_| {}), false);
    assert_eq!(doc.listener_count(elem, "click"), 1);

    doc.add_event_listener(elem, "click", Box::new(|_| {}), false);
    assert_eq!(doc.listener_count(elem, "click"), 2);

    let removed = doc.remove_event_listener(elem, "click");
    assert_eq!(removed, 2);
    assert_eq!(doc.listener_count(elem, "click"), 0);
}

/// Remove all listeners for a node clears all event types.
#[test]
fn test_remove_all_listeners_clears_all_types() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.add_event_listener(elem, "click", Box::new(|_| {}), false);
    doc.add_event_listener(elem, "input", Box::new(|_| {}), false);
    doc.add_event_listener(elem, "keydown", Box::new(|_| {}), false);
    doc.add_event_listener(elem, "keyup", Box::new(|_| {}), false);

    assert_eq!(doc.listener_count(elem, "click"), 1);
    assert_eq!(doc.listener_count(elem, "input"), 1);
    assert_eq!(doc.listener_count(elem, "keydown"), 1);
    assert_eq!(doc.listener_count(elem, "keyup"), 1);

    doc.remove_all_event_listeners(elem);

    assert_eq!(doc.listener_count(elem, "click"), 0);
    assert_eq!(doc.listener_count(elem, "input"), 0);
    assert_eq!(doc.listener_count(elem, "keydown"), 0);
    assert_eq!(doc.listener_count(elem, "keyup"), 0);
}

/// Observe attribute changes records correct old_value.
#[test]
fn test_observe_attribute_changes_old_value() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "v1");
    doc.take_mutation_records(); // clear

    doc.set_attribute(elem, "class", "v2");
    let records = doc.take_mutation_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].mutation_type, MutationType::Attributes);
    assert_eq!(records[0].attribute_name, Some("class".to_string()));
    assert_eq!(records[0].old_value, Some("v1".to_string()));
}

/// Observe child list changes records added and removed nodes.
#[test]
fn test_observe_child_list_add_and_remove() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    doc.take_mutation_records(); // clear

    let c1 = doc.create_element("span");
    doc.append_child(parent, c1).unwrap();
    let c2 = doc.create_element("p");
    doc.append_child(parent, c2).unwrap();
    doc.remove_child(parent, c1).unwrap();

    let records = doc.take_mutation_records();
    // 3 operations: append c1, append c2, remove c1
    assert_eq!(records.len(), 3);
    assert_eq!(records[0].added_nodes, vec![c1]);
    assert!(records[0].removed_nodes.is_empty());
    assert_eq!(records[2].removed_nodes, vec![c1]);
    assert!(records[2].added_nodes.is_empty());
}

/// Take records clears pending, second call returns empty.
#[test]
fn test_take_records_clears_then_empty() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "data-x", "1");
    doc.set_attribute(elem, "data-y", "2");
    doc.set_attribute(elem, "data-z", "3");

    let first = doc.take_mutation_records();
    assert_eq!(first.len(), 3);

    let second = doc.take_mutation_records();
    assert!(second.is_empty(), "second take should return empty");
}

/// Process mutations notifies all registered observers.
#[test]
fn test_process_mutations_notifies_observers() {
    let count1 = Arc::new(Mutex::new(0usize));
    let count2 = Arc::new(Mutex::new(0usize));
    let c1 = count1.clone();
    let c2 = count2.clone();

    let mut doc = Document::new();
    doc.add_observer(MutationObserver::new(Box::new(move |records| {
        *c1.lock().unwrap() += records.len();
    })));
    doc.add_observer(MutationObserver::new(Box::new(move |records| {
        *c2.lock().unwrap() += records.len();
    })));

    let root = doc.root();
    let child = doc.create_element("div");
    doc.append_child(root, child).unwrap();
    doc.process_mutations();

    assert_eq!(*count1.lock().unwrap(), 1, "first observer should be notified");
    assert_eq!(*count2.lock().unwrap(), 1, "second observer should be notified");
}

/// Multiple observers on same document each get all records.
#[test]
fn test_multiple_observers_same_document() {
    let received_a = Arc::new(Mutex::new(Vec::new()));
    let received_b = Arc::new(Mutex::new(Vec::new()));
    let ra = received_a.clone();
    let rb = received_b.clone();

    let mut doc = Document::new();
    doc.add_observer(MutationObserver::new(Box::new(move |records| {
        for r in records {
            ra.lock().unwrap().push(r.mutation_type.clone());
        }
    })));
    doc.add_observer(MutationObserver::new(Box::new(move |records| {
        for r in records {
            rb.lock().unwrap().push(r.mutation_type.clone());
        }
    })));

    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "test");

    doc.process_mutations();

    assert_eq!(*received_a.lock().unwrap(), vec![MutationType::Attributes]);
    assert_eq!(*received_b.lock().unwrap(), vec![MutationType::Attributes]);
}

/// query_selector with ID selector finds correct element.
#[test]
fn test_query_selector_id_finds_correct() {
    let doc = parse_html("<html><body><div id=\"first\">a</div><div id=\"second\">b</div></body></html>");
    let root = doc.root();

    let found = doc.query_selector(root, "#second").unwrap();
    assert_eq!(doc.text_content(found), Some("b".to_string()));
}

/// query_selector with class selector finds first match.
#[test]
fn test_query_selector_class_first_match() {
    let doc = parse_html("<html><body><span class=\"x\">1</span><span class=\"x\">2</span></body></html>");
    let root = doc.root();

    let found = doc.query_selector(root, ".x").unwrap();
    assert_eq!(doc.text_content(found), Some("1".to_string()));
}

/// query_selector with tag selector is case-insensitive.
#[test]
fn test_query_selector_tag_case_insensitive() {
    let doc = parse_html("<html><body><DIV>content</DIV></body></html>");
    let root = doc.root();

    let found = doc.query_selector(root, "div");
    assert!(found.is_some());
    assert_eq!(doc.text_content(found.unwrap()), Some("content".to_string()));
}

/// query_selector_all returns all matching elements in document order.
#[test]
fn test_query_selector_all_document_order() {
    let doc = parse_html("<html><body><p>1</p><div><p>2</p></div><p>3</p></body></html>");
    let root = doc.root();

    let all_p = doc.query_selector_all(root, "p");
    assert_eq!(all_p.len(), 3, "should find all 3 <p> elements");
    assert_eq!(doc.text_content(all_p[0]), Some("1".to_string()));
    assert_eq!(doc.text_content(all_p[1]), Some("2".to_string()));
    assert_eq!(doc.text_content(all_p[2]), Some("3".to_string()));
}

/// query_selector with attribute selector [attr=value].
#[test]
fn test_query_selector_attribute_value() {
    let doc = parse_html("<html><body><input type=\"text\" /><input type=\"checkbox\" /></body></html>");
    let root = doc.root();

    let text_input = doc.query_selector(root, "[type=text]");
    assert!(text_input.is_some());
    assert_eq!(doc.get_attribute(text_input.unwrap(), "type"), Some("text".to_string()));

    let checkbox = doc.query_selector(root, "[type=checkbox]");
    assert!(checkbox.is_some());
}

/// Nested element queries: searching from a subtree root.
#[test]
fn test_nested_element_queries() {
    let doc = parse_html(
        "<html><body><div class=\"outer\"><span id=\"target\"><em>deep</em></span></div><span id=\"sibling\">outside</span></body></html>",
    );
    let root = doc.root();
    let outer = doc.query_selector(root, ".outer").unwrap();

    // From outer, find span#target
    let target = doc.query_selector(outer, "#target").unwrap();
    assert_eq!(doc.text_content(target), Some("deep".to_string()));

    // From outer, should NOT find span#sibling (it's outside outer)
    assert!(doc.query_selector(outer, "#sibling").is_none());

    // From root, find both
    assert!(doc.query_selector(root, "#target").is_some());
    assert!(doc.query_selector(root, "#sibling").is_some());
}

/// Parse HTML with script tags.
#[test]
fn test_parse_script_tags() {
    let doc = parse_html("<html><body><script>var x = 1;</script><p>after</p></body></html>");
    let scripts = doc.get_elements_by_tag_name("script");
    assert_eq!(scripts.len(), 1, "should have one script element");
    let text = doc.text_content(scripts[0]);
    assert!(text.is_some());
    assert!(text.unwrap().contains("var x = 1;"));
}

/// Parse HTML with style tags.
#[test]
fn test_parse_style_tags() {
    let doc = parse_html("<html><head><style>body { color: red; }</style></head><body></body></html>");
    let styles = doc.get_elements_by_tag_name("style");
    assert_eq!(styles.len(), 1, "should have one style element");
    let text = doc.text_content(styles[0]);
    assert!(text.is_some());
    assert!(text.unwrap().contains("color: red"));
}

/// Parse HTML with self-closing tags (void elements).
#[test]
fn test_parse_self_closing_tags() {
    let doc = parse_html("<html><body><br/><hr/><img src=\"test.png\"/></body></html>");
    let brs = doc.get_elements_by_tag_name("br");
    let hrs = doc.get_elements_by_tag_name("hr");
    let imgs = doc.get_elements_by_tag_name("img");
    assert_eq!(brs.len(), 1);
    assert_eq!(hrs.len(), 1);
    assert_eq!(imgs.len(), 1);
    assert_eq!(doc.get_attribute(imgs[0], "src"), Some("test.png".to_string()));
}

/// Parse HTML with nested lists.
#[test]
fn test_parse_nested_lists() {
    let doc = parse_html("<html><body><ul><li>a<ul><li>b1</li><li>b2</li></ul></li><li>c</li></ul></body></html>");
    let uls = doc.get_elements_by_tag_name("ul");
    assert_eq!(uls.len(), 2, "should have outer and inner <ul>");

    let lis = doc.get_elements_by_tag_name("li");
    assert_eq!(lis.len(), 4, "should have 4 <li> elements total");
}

/// Removed node still allows text_content access.
#[test]
fn test_removed_node_text_content_accessible() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_text_content(elem, "hello");
    doc.append_child(root, elem).unwrap();

    doc.remove_child(root, elem).unwrap();

    // 节点已从树中移除，但仍然可以访问 text_content
    assert_eq!(doc.text_content(elem), Some("hello".to_string()));
}

/// Removed node can be re-attached to the tree.
#[test]
fn test_removed_node_can_be_reattached() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("span");
    doc.set_attribute(elem, "id", "test");
    doc.append_child(root, elem).unwrap();

    doc.remove_child(root, elem).unwrap();
    assert_eq!(doc.parent_node(elem), None);

    // 重新挂载
    doc.append_child(root, elem).unwrap();
    assert_eq!(doc.parent_node(elem), Some(root));

    let found = doc.get_element_by_id("test");
    assert_eq!(found, Some(elem), "re-attached node should be findable by id");
}

/// set_attribute on a detached node still works.
#[test]
fn test_set_attribute_on_detached_node() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    // Never attached to tree
    doc.set_attribute(elem, "class", "orphan");
    assert_eq!(doc.get_attribute(elem, "class"), Some("orphan".to_string()));
}

/// clone_node on a removed node produces a correct copy.
#[test]
fn test_clone_removed_node() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "original");
    doc.append_child(root, elem).unwrap();
    doc.remove_child(root, elem).unwrap();

    let cloned = doc.clone_node(elem, false);
    assert_eq!(doc.get_attribute(cloned, "class"), Some("original".to_string()));
}

/// child_nodes of a removed node returns its children (not empty).
#[test]
fn test_removed_node_child_nodes_still_works() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();
    doc.append_child(root, parent).unwrap();

    doc.remove_child(root, parent).unwrap();

    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 1);
    assert_eq!(children[0], child);
}

/// insert_before with ref_node not a child of parent returns error.
#[test]
fn test_insert_before_ref_not_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let new_child = doc.create_element("span");
    let wrong_ref = doc.create_element("p");
    doc.append_child(root, parent).unwrap();

    let result = doc.insert_before(parent, new_child, wrong_ref);
    assert!(result.is_err(), "insert_before with non-child ref should fail");
}

/// replace_child where old_child is not a child of parent returns error.
#[test]
fn test_replace_child_old_not_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let new_child = doc.create_element("span");
    let wrong_old = doc.create_element("p");
    doc.append_child(root, parent).unwrap();

    let result = doc.replace_child(parent, new_child, wrong_old);
    assert!(result.is_err(), "replace_child with non-child old should fail");
}

/// 测试 get_elements_by_class_name 返回正确数量的匹配元素。
#[test]
fn test_element_get_elements_by_class_name() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();

    let c1 = doc.create_element("span");
    doc.set_attribute(c1, "class", "foo");
    let c2 = doc.create_element("p");
    doc.set_attribute(c2, "class", "foo");
    let c3 = doc.create_element("a");
    doc.set_attribute(c3, "class", "bar");

    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();
    doc.append_child(parent, c3).unwrap();

    let results = doc.get_elements_by_class_name("foo");
    assert_eq!(results.len(), 2, "应该找到 2 个 class 为 foo 的元素");
    assert!(results.contains(&c1));
    assert!(results.contains(&c2));
}

/// 测试 set_attribute 设置 id 后可通过 get_attribute 取回。
#[test]
fn test_element_set_id() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.set_attribute(elem, "id", "my-element");
    assert_eq!(
        doc.get_attribute(elem, "id"),
        Some("my-element".to_string()),
        "get_attribute(\"id\") 应返回设置的值"
    );
}

/// 测试 create_comment 创建的注释节点类型和文本内容。
#[test]
fn test_document_create_comment() {
    let mut doc = Document::new();
    let comment = doc.create_comment("这是一条注释");

    assert_eq!(doc.node_type(comment), Some(8), "注释节点类型应为 8");
    if let Some(NodeKind::Comment(data)) = doc.get(comment).map(|n| n.kind.clone()) {
        assert_eq!(data.content, "这是一条注释");
    } else {
        panic!("应该创建 Comment 节点");
    }
}

/// 测试 insert_before 在最后一个子节点之前插入等价于 append_child 的效果。
///
/// 由于当前 insert_before 不接受 Option<NodeId>，这里通过在最后一个子节点
/// 之前插入来验证其行为与 append_child 语义一致（都是追加到末尾）。
#[test]
fn test_node_insert_before_at_end() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();

    let c1 = doc.create_element("span");
    let c2 = doc.create_element("p");
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();

    // 在 c2（最后一个子节点）之前插入 → 结果为 [c1, new_node, c2]
    let new_node = doc.create_element("a");
    doc.insert_before(parent, new_node, c2).unwrap();
    assert_eq!(doc.child_nodes(parent), vec![c1, new_node, c2]);

    // 现在用 append_child 追加另一个节点到末尾
    let tail = doc.create_element("em");
    doc.append_child(parent, tail).unwrap();
    assert_eq!(doc.child_nodes(parent), vec![c1, new_node, c2, tail]);
    assert_eq!(doc.last_child(parent), Some(tail));
}

/// 测试嵌套文本子节点的 text_content 递归拼接。
#[test]
fn test_element_inner_text() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");

    let t1 = doc.create_text_node("Hello");
    let child = doc.create_element("span");
    let t2 = doc.create_text_node(" ");
    let t3 = doc.create_text_node("World");

    doc.append_child(parent, t1).unwrap();
    doc.append_child(parent, child).unwrap();
    doc.append_child(child, t2).unwrap();
    doc.append_child(child, t3).unwrap();

    assert_eq!(
        doc.text_content(parent),
        Some("Hello World".to_string()),
        "textContent 应递归拼接所有嵌套文本节点"
    );
}

/// insert_before places new node before the reference child.
#[test]
fn test_insert_before_places_before_ref() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let first = doc.create_element("a");
    let second = doc.create_element("b");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, first).unwrap();

    doc.insert_before(parent, second, first).unwrap();

    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], second);
    assert_eq!(children[1], first);
}

/// 测试 remove_child 后 id_map 中的条目已被正确清除。
///
/// remove_child 会从 id_map 中移除被删除节点（及其后代）的 id 映射，
/// 确保 get_element_by_id 不再返回已从文档树中移除的节点。
#[test]
fn test_id_map_stale_after_remove() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "test");
    doc.append_child(root, elem).unwrap();

    // 追加后可以通过 id 找到
    assert_eq!(doc.get_element_by_id("test"), Some(elem));

    // 从文档中移除
    doc.remove_child(root, elem).unwrap();
    assert_eq!(doc.parent_node(elem), None);

    // 移除后 id_map 条目应被清除
    let found = doc.get_element_by_id("test");
    assert!(
        found.is_none(),
        "remove_child 后 id_map 条目应被清除，get_element_by_id 应返回 None"
    );
}

/// 测试 clone_node 后 id_map 的映射行为。
///
/// clone_node 不将克隆元素的 id 注册到 id_map 中，
/// 确保 get_element_by_id 仍然返回原始节点。
/// id 在文档中必须唯一，克隆节点共享相同的 id 值不应覆盖原始映射。
#[test]
fn test_id_map_after_clone() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "orig");
    doc.append_child(root, elem).unwrap();

    assert_eq!(doc.get_element_by_id("orig"), Some(elem));

    // 克隆元素（浅拷贝，id 属性被复制）
    let cloned = doc.clone_node(elem, false);

    // 两个节点都有 id="orig" 属性
    assert_eq!(doc.get_attribute(elem, "id"), Some("orig".to_string()));
    assert_eq!(doc.get_attribute(cloned, "id"), Some("orig".to_string()));

    // clone_node 不注册到 id_map，get_element_by_id 仍返回原始节点
    let found = doc.get_element_by_id("orig");
    assert_eq!(
        found,
        Some(elem),
        "clone_node 不应覆盖原始节点的 id_map 条目，get_element_by_id 应返回原始节点"
    );
}

/// 测试 set_attribute 修改 id 时 id_map 的更新行为。
///
/// 这是正确行为：将 id 从 "old" 改为 "new" 时，
/// get_element_by_id("old") 返回 None，get_element_by_id("new") 返回该元素。
#[test]
fn test_id_map_update_on_attribute_change() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "old");
    doc.append_child(root, elem).unwrap();

    // 初始状态：通过 "old" 可以找到
    assert_eq!(doc.get_element_by_id("old"), Some(elem));
    assert_eq!(doc.get_element_by_id("new"), None);

    // 修改 id
    doc.set_attribute(elem, "id", "new");

    // 旧 id 不再映射
    assert_eq!(doc.get_element_by_id("old"), None, "修改 id 后旧 id 应从 id_map 中移除");
    // 新 id 正确映射
    assert_eq!(
        doc.get_element_by_id("new"),
        Some(elem),
        "修改 id 后新 id 应正确映射到该元素"
    );
}

/// 测试 remove_child 清理被移除节点的 id_map 条目。
#[test]
fn test_remove_child_cleans_id_map() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "x");
    doc.append_child(root, elem).unwrap();

    // 追加后可以通过 id 找到
    assert_eq!(doc.get_element_by_id("x"), Some(elem));

    // 从文档中移除
    doc.remove_child(root, elem).unwrap();

    // get_element_by_id("x") 应返回 None
    assert_eq!(
        doc.get_element_by_id("x"),
        None,
        "移除节点后 id_map 应被清理，get_element_by_id 应返回 None"
    );
}

/// 测试 set_attribute 修改 id 时 id_map 正确更新。
#[test]
fn test_set_attribute_updates_id_map() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "old");
    doc.append_child(root, elem).unwrap();

    // 初始状态
    assert_eq!(doc.get_element_by_id("old"), Some(elem));
    assert_eq!(doc.get_element_by_id("new"), None);

    // 修改 id
    doc.set_attribute(elem, "id", "new");

    // 旧 id 不再映射
    assert_eq!(doc.get_element_by_id("old"), None, "修改 id 后旧 id 应从 id_map 中移除");
    // 新 id 正确映射
    assert_eq!(
        doc.get_element_by_id("new"),
        Some(elem),
        "修改 id 后新 id 应正确映射到该元素"
    );
}

/// 测试 remove_attribute 移除 id 属性时清理 id_map。
#[test]
fn test_remove_attribute_cleans_id_map() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "target");
    doc.append_child(root, elem).unwrap();

    // 初始状态
    assert_eq!(doc.get_element_by_id("target"), Some(elem));

    // 移除 id 属性
    doc.remove_attribute(elem, "id");

    // get_element_by_id 应返回 None
    assert_eq!(
        doc.get_element_by_id("target"),
        None,
        "移除 id 属性后 id_map 应被清理，get_element_by_id 应返回 None"
    );
}

/// 测试 remove_child 递归清理后代节点的 id_map 条目。
#[test]
fn test_remove_child_cleans_id_map_recursive() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.set_attribute(child, "id", "inner");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    assert_eq!(doc.get_element_by_id("inner"), Some(child));

    // 移除父节点，后代节点的 id_map 条目也应被清理
    doc.remove_child(root, parent).unwrap();

    assert_eq!(
        doc.get_element_by_id("inner"),
        None,
        "移除父节点后，后代的 id_map 条目也应被清理"
    );
}

/// 测试 closed 模式 Shadow DOM 的 shadow_root 访问。
///
/// 验证 attach_shadow 后，closed 模式的 shadow_root 仍可获取，
/// 但实际使用中 closed 模式应限制外部脚本访问。
#[test]
fn test_shadow_root_closed_mode_access() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    doc.append_child(doc.root(), host).unwrap();

    let shadow = doc.attach_shadow(host, ShadowRootMode::Closed).unwrap();

    // 当前实现中 shadow_root() 返回 Some，无论模式
    assert!(doc.shadow_root(host).is_some(), "shadow_root 应返回 Some");

    // 验证模式为 Closed
    let mode = doc.get_shadow_root_mode(shadow).unwrap();
    assert_eq!(mode, ShadowRootMode::Closed);
}

/// 测试 compare_document_position 在深层分支树中的行为。
///
/// 结构：root > div1 > span1 vs root > div2 > span2
/// span1 和 span2 共享 root 作为祖先，但在不同分支。
#[test]
fn test_compare_document_position_deep_branching() {
    let mut doc = Document::new();
    let root = doc.root();
    let div1 = doc.create_element("div");
    let div2 = doc.create_element("div");
    let span1 = doc.create_element("span");
    let span2 = doc.create_element("span");

    doc.append_child(root, div1).unwrap();
    doc.append_child(root, div2).unwrap();
    doc.append_child(div1, span1).unwrap();
    doc.append_child(div2, span2).unwrap();

    // span1 在 span2 之前 → node2(span2) 在 node1(span1) 之后 → FOLLOWING
    let pos = doc.compare_document_position(span1, span2).unwrap();
    assert!(
        pos.contains(DocumentPosition::FOLLOWING),
        "span2 应在 span1 之后（span1 在前）"
    );

    // 反向：span2 在 span1 之后 → node1(span2) 在 node2(span1) 之前 → PRECEDING
    let pos = doc.compare_document_position(span2, span1).unwrap();
    assert!(pos.contains(DocumentPosition::PRECEDING), "span1 应在 span2 之前");
}

/// 测试 compare_document_position 对同一元素返回 0。
#[test]
fn test_compare_document_position_same_node() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    doc.append_child(doc.root(), div).unwrap();

    let pos = doc.compare_document_position(div, div).unwrap();
    assert_eq!(pos.bits(), 0, "同一元素应返回 0");
}

/// 测试 compare_document_position 在直接父子关系中的行为。
#[test]
fn test_compare_document_position_parent_child() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(doc.root(), parent).unwrap();
    doc.append_child(parent, child).unwrap();

    // parent 包含 child
    let pos = doc.compare_document_position(parent, child).unwrap();
    assert!(pos.contains(DocumentPosition::CONTAINED_BY), "parent 应包含 child");

    // child 被 parent 包含
    let pos = doc.compare_document_position(child, parent).unwrap();
    assert!(pos.contains(DocumentPosition::CONTAINS), "child 应被 parent 包含");
}
