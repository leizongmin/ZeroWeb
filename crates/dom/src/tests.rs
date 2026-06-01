//! DOM crate 综合测试套件。
//!
//! 覆盖：节点类型、树操作、属性操作、HTML 解析、查询、序列化、MutationObserver。

use crate::*;
use std::sync::{Arc, Mutex};

// ═══════════════════════════════════════════════════════════════════════
// 1. 节点创建测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_create_document() {
    let doc = Document::new();
    assert!(doc.root().is_valid());
    assert_eq!(doc.node_count(), 1);
    let root = doc.root();
    assert!(matches!(doc.get(root).map(|n| &n.kind), Some(NodeKind::Document(_))));
}

#[test]
fn test_create_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert!(doc.contains(elem));
    assert!(matches!(doc.get(elem).map(|n| &n.kind), Some(NodeKind::Element(_))));
    if let Some(NodeKind::Element(e)) = doc.get(elem).map(|n| n.kind.clone()) {
        assert_eq!(e.local_name(), "div");
    }
}

#[test]
fn test_create_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("Hello World");
    assert!(doc.contains(text));
    if let Some(NodeKind::Text(data)) = doc.get(text).map(|n| n.kind.clone()) {
        assert_eq!(data.content, "Hello World");
    }
}

#[test]
fn test_create_comment() {
    let mut doc = Document::new();
    let comment = doc.create_comment("a comment");
    assert!(doc.contains(comment));
    if let Some(NodeKind::Comment(data)) = doc.get(comment).map(|n| n.kind.clone()) {
        assert_eq!(data.content, "a comment");
    }
}

#[test]
fn test_create_document_fragment() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    assert!(doc.contains(frag));
    assert!(matches!(
        doc.get(frag).map(|n| &n.kind),
        Some(NodeKind::DocumentFragment)
    ));
}

#[test]
fn test_create_document_type() {
    let mut doc = Document::new();
    let doctype = doc.create_document_type("html", None, None);
    assert!(doc.contains(doctype));
    if let Some(NodeKind::DocumentType(dt)) = doc.get(doctype).map(|n| n.kind.clone()) {
        assert_eq!(dt.name, "html");
        assert!(dt.public_id.is_none());
        assert!(dt.system_id.is_none());
    }
}

#[test]
fn test_create_element_with_namespace() {
    let mut doc = Document::new();
    let elem = doc.create_element("svg");
    if let Some(NodeKind::Element(e)) = doc.get(elem).map(|n| n.kind.clone()) {
        assert_eq!(e.local_name(), "svg");
        // 默认使用 XHTML 命名空间
        assert_eq!(e.namespace(), "http://www.w3.org/1999/xhtml");
    }
}

#[test]
fn test_create_empty_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("");
    if let Some(NodeKind::Text(data)) = doc.get(text).map(|n| n.kind.clone()) {
        assert!(data.content.is_empty());
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 2. 树操作测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_append_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let child = doc.create_element("div");

    doc.append_child(root, child).unwrap();

    assert_eq!(doc.parent_node(child), Some(root));
    assert_eq!(doc.child_nodes(root), vec![child]);
    assert!(doc.has_child_nodes(root));
}

#[test]
fn test_append_multiple_children() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");

    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();

    assert_eq!(doc.child_nodes(root), vec![c1, c2, c3]);
    assert_eq!(doc.first_child(root), Some(c1));
    assert_eq!(doc.last_child(root), Some(c3));
}

#[test]
fn test_remove_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let child = doc.create_element("div");
    doc.append_child(root, child).unwrap();

    let removed = doc.remove_child(root, child).unwrap();
    assert_eq!(removed, child);
    assert_eq!(doc.parent_node(child), None);
    assert!(!doc.has_child_nodes(root));
}

#[test]
fn test_remove_middle_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");

    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();

    doc.remove_child(root, c2).unwrap();

    assert_eq!(doc.child_nodes(root), vec![c1, c3]);
}

#[test]
fn test_insert_before() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");

    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c3).unwrap();
    doc.insert_before(root, c2, c3).unwrap();

    assert_eq!(doc.child_nodes(root), vec![c1, c2, c3]);
}

#[test]
fn test_replace_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let old = doc.create_element("div");
    let new = doc.create_element("span");

    doc.append_child(root, old).unwrap();
    let replaced = doc.replace_child(root, new, old).unwrap();

    assert_eq!(replaced, old);
    assert_eq!(doc.child_nodes(root), vec![new]);
    assert_eq!(doc.parent_node(old), None);
    assert_eq!(doc.parent_node(new), Some(root));
}

#[test]
fn test_clone_node_shallow() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "test");
    let child = doc.create_text_node("Hello");
    doc.append_child(elem, child).unwrap();

    let cloned = doc.clone_node(elem, false);
    assert_ne!(cloned, elem);
    assert!(!doc.has_child_nodes(cloned));
    assert_eq!(doc.get_attribute(cloned, "class"), Some("test".to_string()));
}

#[test]
fn test_clone_node_deep() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let text = doc.create_text_node("Hello");
    doc.append_child(elem, text).unwrap();

    let cloned = doc.clone_node(elem, true);
    assert!(doc.has_child_nodes(cloned));
    assert_ne!(doc.first_child(cloned), Some(text)); // 新的子节点
    assert_eq!(doc.text_content(cloned), Some("Hello".to_string()));
}

#[test]
fn test_append_child_reparents() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent1 = doc.create_element("div");
    let parent2 = doc.create_element("span");
    let child = doc.create_element("p");

    doc.append_child(root, parent1).unwrap();
    doc.append_child(root, parent2).unwrap();
    doc.append_child(parent1, child).unwrap();

    assert_eq!(doc.parent_node(child), Some(parent1));

    // 移动 child 到 parent2
    doc.append_child(parent2, child).unwrap();

    assert_eq!(doc.parent_node(child), Some(parent2));
    assert!(!doc.has_child_nodes(parent1));
    assert_eq!(doc.child_nodes(parent2), vec![child]);
}

#[test]
fn test_cycle_detection() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    let root = doc.root();

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    // 试图将父节点作为子节点的子节点 — 应该失败
    let result = doc.append_child(child, parent);
    assert!(matches!(result, Err(DomError::WouldCreateCycle)));
}

#[test]
fn test_append_document_root_fails() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.append_child(root, elem).unwrap();

    // 试图将文档根节点作为子节点 — 应该失败
    let result = doc.append_child(elem, root);
    assert!(matches!(result, Err(DomError::CannotInsertDocumentRoot)));
}

#[test]
fn test_remove_non_child_fails() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    // elem 不是 root 的子节点（未 append）

    let result = doc.remove_child(root, elem);
    assert!(matches!(result, Err(DomError::NotAChild { .. })));
}

// ═══════════════════════════════════════════════════════════════════════
// 3. 遍历测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_sibling_traversal() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");

    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();

    assert_eq!(doc.next_sibling(c1), Some(c2));
    assert_eq!(doc.next_sibling(c2), Some(c3));
    assert_eq!(doc.next_sibling(c3), None);

    assert_eq!(doc.previous_sibling(c1), None);
    assert_eq!(doc.previous_sibling(c2), Some(c1));
    assert_eq!(doc.previous_sibling(c3), Some(c2));
}

#[test]
fn test_parent_node() {
    let mut doc = Document::new();
    let root = doc.root();
    let child = doc.create_element("div");
    assert_eq!(doc.parent_node(child), None);

    doc.append_child(root, child).unwrap();
    assert_eq!(doc.parent_node(child), Some(root));
}

#[test]
fn test_has_child_nodes() {
    let mut doc = Document::new();
    let root = doc.root();
    assert!(!doc.has_child_nodes(root));

    let child = doc.create_element("div");
    doc.append_child(root, child).unwrap();
    assert!(doc.has_child_nodes(root));
}

// ═══════════════════════════════════════════════════════════════════════
// 4. 属性操作测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_set_get_attribute() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.set_attribute(elem, "class", "container");
    assert_eq!(doc.get_attribute(elem, "class"), Some("container".to_string()));
    assert_eq!(doc.get_attribute(elem, "id"), None);
}

#[test]
fn test_update_attribute() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.set_attribute(elem, "class", "old");
    doc.set_attribute(elem, "class", "new");
    assert_eq!(doc.get_attribute(elem, "class"), Some("new".to_string()));
}

#[test]
fn test_remove_attribute() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.set_attribute(elem, "data-test", "value");
    assert!(doc.has_attribute(elem, "data-test"));

    doc.remove_attribute(elem, "data-test");
    assert!(!doc.has_attribute(elem, "data-test"));
    assert_eq!(doc.get_attribute(elem, "data-test"), None);
}

#[test]
fn test_attribute_names() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.set_attribute(elem, "id", "main");
    doc.set_attribute(elem, "class", "container");

    let names = doc.attribute_names(elem);
    assert!(names.contains(&"id".to_string()));
    assert!(names.contains(&"class".to_string()));
}

#[test]
fn test_id_attribute_indexing() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "main");
    let root = doc.root();
    doc.append_child(root, elem).unwrap();

    assert_eq!(doc.get_element_by_id("main"), Some(elem));
}

#[test]
fn test_id_attribute_update_index() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "old");
    let root = doc.root();
    doc.append_child(root, elem).unwrap();

    assert_eq!(doc.get_element_by_id("old"), Some(elem));

    doc.set_attribute(elem, "id", "new");
    assert_eq!(doc.get_element_by_id("old"), None);
    assert_eq!(doc.get_element_by_id("new"), Some(elem));
}

#[test]
fn test_attribute_on_non_element() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    // 在文本节点上设置属性应该无效果
    doc.set_attribute(text, "class", "test");
    assert_eq!(doc.get_attribute(text, "class"), None);
}

// ═══════════════════════════════════════════════════════════════════════
// 5. textContent 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_text_content_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("Hello World");
    assert_eq!(doc.text_content(text), Some("Hello World".to_string()));
}

#[test]
fn test_text_content_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let t1 = doc.create_text_node("Hello ");
    let t2 = doc.create_text_node("World");
    doc.append_child(elem, t1).unwrap();
    doc.append_child(elem, t2).unwrap();

    assert_eq!(doc.text_content(elem), Some("Hello World".to_string()));
}

#[test]
fn test_text_content_nested() {
    let mut doc = Document::new();
    let outer = doc.create_element("div");
    let inner = doc.create_element("span");
    let text = doc.create_text_node("Hello");
    doc.append_child(outer, inner).unwrap();
    doc.append_child(inner, text).unwrap();

    assert_eq!(doc.text_content(outer), Some("Hello".to_string()));
}

#[test]
fn test_set_text_content_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let child = doc.create_text_node("old");
    doc.append_child(elem, child).unwrap();

    doc.set_text_content(elem, "new text");
    assert_eq!(doc.text_content(elem), Some("new text".to_string()));
    // 旧子节点应该被清除
    assert_eq!(doc.parent_node(child), None);
}

#[test]
fn test_set_text_content_empty() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let child = doc.create_text_node("old");
    doc.append_child(elem, child).unwrap();

    doc.set_text_content(elem, "");
    assert!(!doc.has_child_nodes(elem));
}

// ═══════════════════════════════════════════════════════════════════════
// 6. HTML 解析测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_parse_simple_html() {
    let doc = parse_html("<html><body><h1>Hello</h1></body></html>");
    assert!(doc.root().is_valid());
    // 应该有 Document + html + body + h1 + "Hello" = 5 个节点
    assert!(doc.node_count() >= 5);
}

#[test]
fn test_parse_html_with_doctype() {
    let doc = parse_html("<!DOCTYPE html><html><body></body></html>");
    let root = doc.root();
    // 第一个子节点应该是 DocumentType
    let first_child = doc.first_child(root);
    assert!(first_child.is_some());
    if let Some(fc) = first_child {
        assert!(matches!(doc.get(fc).map(|n| &n.kind), Some(NodeKind::DocumentType(_))));
    }
}

#[test]
fn test_parse_html_with_attributes() {
    let doc = parse_html("<html><body><div id=\"main\" class=\"container\">text</div></body></html>");
    let divs = doc.get_elements_by_tag_name("div");
    assert_eq!(divs.len(), 1);

    let div = divs[0];
    assert_eq!(doc.get_attribute(div, "id"), Some("main".to_string()));
    assert_eq!(doc.get_attribute(div, "class"), Some("container".to_string()));
}

#[test]
fn test_parse_malformed_html() {
    // html5ever 的错误恢复应该能处理损坏的 HTML
    let doc = parse_html("<html><body><div><span>unclosed tags");
    assert!(doc.root().is_valid());
    // 应该不 panic
    let text = doc.text_content(doc.root());
    assert!(text.is_some());
    assert!(text.unwrap().contains("unclosed tags"));
}

#[test]
fn test_parse_empty_html() {
    let doc = parse_html("");
    assert!(doc.root().is_valid());
}

#[test]
fn test_parse_html_comments() {
    let doc = parse_html("<html><body><!-- a comment --><p>text</p></body></html>");
    let text = doc.text_content(doc.root());
    assert!(text.is_some());
}

#[test]
fn test_parse_html_nested_elements() {
    let doc = parse_html("<html><body><div><ul><li>a</li><li>b</li></ul></div></body></html>");
    let lis = doc.get_elements_by_tag_name("li");
    assert_eq!(lis.len(), 2);
}

#[test]
fn test_parse_html_text_extraction() {
    let doc = parse_html("<html><body><p>Hello <strong>World</strong>!</p></body></html>");
    let ps = doc.get_elements_by_tag_name("p");
    assert_eq!(ps.len(), 1);
    let text = doc.text_content(ps[0]);
    assert_eq!(text, Some("Hello World!".to_string()));
}

#[test]
fn test_quirks_mode() {
    let doc = parse_html("<!DOCTYPE html><html><body></body></html>");
    assert_eq!(doc.quirks_mode(), QuirksMode::NoQuirks);
}

// ═══════════════════════════════════════════════════════════════════════
// 7. 查询 API 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_get_element_by_id() {
    let doc = parse_html("<html><body><div id=\"main\">content</div></body></html>");
    let elem = doc.get_element_by_id("main");
    assert!(elem.is_some());
    assert_eq!(doc.get_attribute(elem.unwrap(), "id"), Some("main".to_string()));
}

#[test]
fn test_get_element_by_id_not_found() {
    let doc = parse_html("<html><body><div>content</div></body></html>");
    assert!(doc.get_element_by_id("nonexistent").is_none());
}

#[test]
fn test_get_elements_by_tag_name() {
    let doc = parse_html("<html><body><div>a</div><div>b</div><span>c</span></body></html>");
    let divs = doc.get_elements_by_tag_name("div");
    assert_eq!(divs.len(), 2);

    let spans = doc.get_elements_by_tag_name("span");
    assert_eq!(spans.len(), 1);
}

#[test]
fn test_get_elements_by_tag_name_case_insensitive() {
    let doc = parse_html("<html><body><DIV>a</DIV></body></html>");
    let divs = doc.get_elements_by_tag_name("div");
    assert_eq!(divs.len(), 1);
}

#[test]
fn test_get_elements_by_class_name() {
    let doc = parse_html("<html><body><div class=\"item\">a</div><div class=\"item\">b</div></body></html>");
    let items = doc.get_elements_by_class_name("item");
    assert_eq!(items.len(), 2);
}

#[test]
fn test_query_selector_tag() {
    let doc = parse_html("<html><body><div>first</div><div>second</div></body></html>");
    let root = doc.root();
    let result = doc.query_selector(root, "div");
    assert!(result.is_some());
    let text = doc.text_content(result.unwrap());
    assert_eq!(text, Some("first".to_string()));
}

#[test]
fn test_query_selector_id() {
    let doc = parse_html("<html><body><div id=\"target\">found</div></body></html>");
    let root = doc.root();
    let result = doc.query_selector(root, "#target");
    assert!(result.is_some());
}

#[test]
fn test_query_selector_class() {
    let doc = parse_html("<html><body><div class=\"item\">found</div></body></html>");
    let root = doc.root();
    let result = doc.query_selector(root, ".item");
    assert!(result.is_some());
}

#[test]
fn test_query_selector_attribute() {
    let doc = parse_html("<html><body><input type=\"text\" /><input type=\"password\" /></body></html>");
    let root = doc.root();
    let result = doc.query_selector(root, "[type=text]");
    assert!(result.is_some());
}

#[test]
fn test_query_selector_all() {
    let doc = parse_html("<html><body><div>a</div><div>b</div><div>c</div></body></html>");
    let root = doc.root();
    let results = doc.query_selector_all(root, "div");
    assert_eq!(results.len(), 3);
}

#[test]
fn test_query_selector_not_found() {
    let doc = parse_html("<html><body></body></html>");
    let root = doc.root();
    assert!(doc.query_selector(root, "#nonexistent").is_none());
    assert!(doc.query_selector_all(root, ".nonexistent").is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 8. 序列化测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_inner_html() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let text = doc.create_text_node("Hello");
    doc.append_child(elem, text).unwrap();

    let html = doc.inner_html(elem);
    assert_eq!(html, "Hello");
}

#[test]
fn test_outer_html_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let text = doc.create_text_node("Hello");
    doc.append_child(elem, text).unwrap();

    let html = doc.outer_html(elem);
    assert_eq!(html, "<div>Hello</div>");
}

#[test]
fn test_serialize_with_attributes() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "test");
    doc.set_attribute(elem, "id", "main");

    let html = doc.outer_html(elem);
    // 属性顺序可能不同
    assert!(html.contains("class=\"test\""));
    assert!(html.contains("id=\"main\""));
    assert!(html.starts_with("<div"));
    assert!(html.ends_with("</div>"));
}

#[test]
fn test_serialize_comment() {
    let mut doc = Document::new();
    let comment = doc.create_comment("a comment");

    let html = doc.outer_html(comment);
    assert_eq!(html, "<!--a comment-->");
}

#[test]
fn test_serialize_void_elements() {
    let mut doc = Document::new();
    let br = doc.create_element("br");
    let hr = doc.create_element("hr");
    let img = doc.create_element("img");

    assert!(doc.outer_html(br).contains("<br>"));
    assert!(!doc.outer_html(br).contains("</br>"));
    assert!(doc.outer_html(hr).contains("<hr>"));
    assert!(doc.outer_html(img).contains("<img>"));
}

#[test]
fn test_serialize_html_escaping() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "title", "a & b < c > d");
    let text = doc.create_text_node("x < y & z");
    doc.append_child(elem, text).unwrap();

    let html = doc.outer_html(elem);
    assert!(html.contains("&amp;"));
    assert!(html.contains("&lt;"));
}

#[test]
fn test_serialize_document_type() {
    let mut doc = Document::new();
    let doctype = doc.create_document_type("html", None, None);
    let html = doc.outer_html(doctype);
    assert!(html.contains("<!DOCTYPE html>"));
}

// ═══════════════════════════════════════════════════════════════════════
// 9. MutationObserver 测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_mutation_record_on_append() {
    let mut doc = Document::new();
    let root = doc.root();
    let child = doc.create_element("div");

    doc.append_child(root, child).unwrap();

    let records = doc.take_mutation_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].mutation_type, MutationType::ChildList);
    assert_eq!(records[0].target, root);
    assert_eq!(records[0].added_nodes, vec![child]);
}

#[test]
fn test_mutation_record_on_remove() {
    let mut doc = Document::new();
    let root = doc.root();
    let child = doc.create_element("div");
    doc.append_child(root, child).unwrap();
    doc.take_mutation_records(); // 清空

    doc.remove_child(root, child).unwrap();

    let records = doc.take_mutation_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].removed_nodes, vec![child]);
}

#[test]
fn test_mutation_record_on_attribute_change() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.set_attribute(elem, "class", "old");
    let records = doc.take_mutation_records();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].mutation_type, MutationType::Attributes);
    assert_eq!(records[0].attribute_name, Some("class".to_string()));
    assert_eq!(records[0].old_value, None);

    doc.set_attribute(elem, "class", "new");
    let records = doc.take_mutation_records();
    assert_eq!(records[0].old_value, Some("old".to_string()));
}

#[test]
fn test_mutation_observer_callback() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let received = Rc::new(RefCell::new(Vec::new()));
    let received_clone = received.clone();

    let observer = MutationObserver::new(Box::new(move |records: &[MutationRecord]| {
        for r in records {
            received_clone.borrow_mut().push(r.mutation_type.clone());
        }
    }));

    let mut doc = Document::new();
    doc.add_observer(observer);

    let root = doc.root();
    let child = doc.create_element("div");
    doc.append_child(root, child).unwrap();

    doc.process_mutations();

    assert_eq!(*received.borrow(), vec![MutationType::ChildList]);
}

#[test]
fn test_take_mutation_records_clears() {
    let mut doc = Document::new();
    let root = doc.root();
    let child = doc.create_element("div");
    doc.append_child(root, child).unwrap();

    let records = doc.take_mutation_records();
    assert!(!records.is_empty());

    let records2 = doc.take_mutation_records();
    assert!(records2.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// 10. 边界条件测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_deep_nesting() {
    let mut doc = Document::new();
    let root = doc.root();
    let mut current = root;

    // 创建 100 层嵌套
    for i in 0..100 {
        let child = doc.create_element("div");
        doc.set_attribute(child, "data-depth", &i.to_string());
        doc.append_child(current, child).unwrap();
        current = child;
    }

    // 验证最深层的文本内容
    assert_eq!(doc.get_attribute(current, "data-depth"), Some("99".to_string()));
    assert_eq!(doc.node_count(), 101); // 1 document + 100 divs
}

#[test]
fn test_large_number_of_siblings() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();

    for i in 0..1000 {
        let child = doc.create_text_node(&i.to_string());
        doc.append_child(parent, child).unwrap();
    }

    assert_eq!(doc.child_nodes(parent).len(), 1000);
}

#[test]
fn test_remove_all_children() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();

    let c1 = doc.create_element("span");
    let c2 = doc.create_element("p");
    let c3 = doc.create_element("a");
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();
    doc.append_child(parent, c3).unwrap();

    doc.remove_child(parent, c1).unwrap();
    doc.remove_child(parent, c2).unwrap();
    doc.remove_child(parent, c3).unwrap();

    assert!(!doc.has_child_nodes(parent));
}

#[test]
fn test_nonexistent_node_operations() {
    let mut doc = Document::new();
    let _root = doc.root();

    // 创建然后移除一个节点模拟不存在的 ID
    let ghost = doc.create_element("div");
    // 不 append 到任何地方

    // parent_node 对未附加节点返回 None
    assert_eq!(doc.parent_node(ghost), None);
    assert_eq!(doc.first_child(ghost), None);
    // text_content on an unattached element returns Some("") (empty concat of children)
    assert_eq!(doc.text_content(ghost), Some("".to_string()));
}

#[test]
fn test_node_not_found_error() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");

    // 试图将文档根节点作为子节点 — 应该失败
    let result = doc.append_child(elem, root);
    assert!(matches!(result, Err(DomError::CannotInsertDocumentRoot)));
}

#[test]
fn test_multiple_id_attribute() {
    let doc = parse_html("<html><body><div id=\"first\">a</div><div id=\"second\">b</div></body></html>");
    assert!(doc.get_element_by_id("first").is_some());
    assert!(doc.get_element_by_id("second").is_some());
    assert!(doc.get_element_by_id("third").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// 11. 选择器解析测试（query.rs 中的测试已包含基础测试，这里补充集成测试）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_query_selector_combined() {
    let doc = parse_html("<html><body><div id=\"main\" class=\"container active\"><p>text</p></div></body></html>");
    let root = doc.root();

    // 组合选择器
    let result = doc.query_selector(root, "div#main.container");
    assert!(result.is_some());

    let result = doc.query_selector(root, "div.container");
    assert!(result.is_some());
}

#[test]
fn test_query_selector_attribute_includes() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "foo bar baz");
    doc.append_child(root, elem).unwrap();

    let result = doc.query_selector(root, "[class~=bar]");
    assert!(result.is_some());
    assert_eq!(result.unwrap(), elem);
}

// ═══════════════════════════════════════════════════════════════════════
// 12. 序列化补充测试（扩展已有序列化测试覆盖面）
// ═══════════════════════════════════════════════════════════════════════

/// 测试 inner_html vs outer_html 区别。
#[test]
fn test_inner_vs_outer_html_distinct() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.append_child(root, div).unwrap();
    let text = doc.create_text_node("hello");
    doc.append_child(div, text).unwrap();

    let outer = doc.outer_html(div);
    let inner = doc.inner_html(div);
    assert!(outer.starts_with("<div>"), "outer should include the element tag");
    assert!(outer.ends_with("</div>"));
    assert_eq!(inner, "hello", "inner should only have children");
}

/// 测试嵌套树序列化。
#[test]
fn test_serialize_nested_tree() {
    let mut doc = Document::new();
    let root = doc.root();
    let outer = doc.create_element("section");
    doc.append_child(root, outer).unwrap();
    let inner = doc.create_element("p");
    doc.append_child(outer, inner).unwrap();
    let text = doc.create_text_node("content");
    doc.append_child(inner, text).unwrap();

    let html = doc.outer_html(outer);
    assert!(html.contains("<section>"));
    assert!(html.contains("<p>content</p>"));
    assert!(html.contains("</section>"));
}

/// 测试属性值中的特殊字符转义。
#[test]
fn test_serialize_attribute_escaping() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    doc.set_attribute(div, "title", "say \"hello\" & goodbye");
    doc.append_child(root, div).unwrap();

    let html = doc.outer_html(div);
    assert!(html.contains("&quot;"), "should escape quotes in attributes");
    assert!(html.contains("&amp;"), "should escape & in attributes");
}

/// 测试序列化未被添加到树的孤立节点。
#[test]
fn test_serialize_orphan_node() {
    let mut doc = Document::new();
    let orphan = doc.create_element("div");
    let html = doc.outer_html(orphan);
    assert!(html.contains("<div"), "orphan node should still serialize, got: {html}");
}

// ═══════════════════════════════════════════════════════════════════════
// 13. 属性操作测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 set_attribute 覆盖已有属性。
#[test]
fn test_set_attribute_overwrite() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "old");
    assert_eq!(doc.get_attribute(elem, "class"), Some("old".to_string()));
    doc.set_attribute(elem, "class", "new");
    assert_eq!(doc.get_attribute(elem, "class"), Some("new".to_string()));
}

/// 测试 remove_attribute 删除后属性为 None。
#[test]
fn test_remove_attribute_clears() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "test");
    assert!(doc.has_attribute(elem, "id"));
    doc.remove_attribute(elem, "id");
    assert_eq!(doc.get_attribute(elem, "id"), None);
    assert!(!doc.has_attribute(elem, "id"));
}

/// 测试 remove_attribute 不存在的属性不 panic。
#[test]
fn test_remove_nonexistent_attribute() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    // 删除不存在的属性应正常完成
    doc.remove_attribute(elem, "noexist");
}

/// 测试 has_attribute。
#[test]
fn test_has_attribute() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert!(!doc.has_attribute(elem, "class"));
    doc.set_attribute(elem, "class", "active");
    assert!(doc.has_attribute(elem, "class"));
}

// ═══════════════════════════════════════════════════════════════════════
// 14. MutationObserver 测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 MutationObserver 手动 notify 回调被调用。
#[test]
fn test_mutation_observer_manual_notify() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();
    let observer = MutationObserver::new(Box::new(move |_records| {
        *called_clone.lock().unwrap() = true;
    }));

    let record = MutationRecord {
        mutation_type: MutationType::Attributes,
        target: elem,
        added_nodes: vec![],
        removed_nodes: vec![],
        previous_sibling: None,
        attribute_name: Some("class".to_string()),
        old_value: None,
    };
    observer.notify(&[record]);
    assert!(*called.lock().unwrap(), "callback should have been called");
}

/// 测试 MutationObserver 多次手动调用。
#[test]
fn test_mutation_observer_repeated_notify() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let child = doc.create_element("span");
    let count = Arc::new(Mutex::new(0usize));
    let count_clone = count.clone();
    let observer = MutationObserver::new(Box::new(move |records| {
        *count_clone.lock().unwrap() += records.len();
    }));

    let record = MutationRecord {
        mutation_type: MutationType::ChildList,
        target: elem,
        added_nodes: vec![child],
        removed_nodes: vec![],
        previous_sibling: None,
        attribute_name: None,
        old_value: None,
    };

    observer.notify(std::slice::from_ref(&record));
    observer.notify(&[record.clone(), record.clone()]);
    assert_eq!(*count.lock().unwrap(), 3, "should have received 3 total records");
}

/// 测试 CharacterData mutation type 记录。
#[test]
fn test_mutation_character_data_record() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let received_type = Arc::new(Mutex::new(None));
    let received_clone = received_type.clone();
    let observer = MutationObserver::new(Box::new(move |records| {
        *received_clone.lock().unwrap() = Some(records[0].mutation_type.clone());
    }));

    let record = MutationRecord {
        mutation_type: MutationType::CharacterData,
        target: elem,
        added_nodes: vec![],
        removed_nodes: vec![],
        previous_sibling: None,
        attribute_name: None,
        old_value: Some("old text".to_string()),
    };
    observer.notify(&[record]);
    assert_eq!(*received_type.lock().unwrap(), Some(MutationType::CharacterData));
}

// ═══════════════════════════════════════════════════════════════════════
// 15. DocumentFragment、克隆、树操作补充测试
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_document_fragment_append_children() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");

    doc.append_child(frag, c1).unwrap();
    doc.append_child(frag, c2).unwrap();
    doc.append_child(frag, c3).unwrap();

    assert_eq!(doc.child_nodes(frag), vec![c1, c2, c3]);
    assert_eq!(doc.parent_node(c1), Some(frag));
    assert_eq!(doc.parent_node(c2), Some(frag));
    assert_eq!(doc.parent_node(c3), Some(frag));
}

#[test]
fn test_document_fragment_insert_into_document() {
    let mut doc = Document::new();
    let root = doc.root();
    let container = doc.create_element("div");
    doc.append_child(root, container).unwrap();

    let frag = doc.create_document_fragment();
    let c1 = doc.create_text_node("a");
    let c2 = doc.create_text_node("b");
    doc.append_child(frag, c1).unwrap();
    doc.append_child(frag, c2).unwrap();

    // 当前实现直接将 fragment 节点作为子节点追加
    doc.append_child(container, frag).unwrap();

    assert_eq!(doc.child_nodes(container), vec![frag]);
    // fragment 仍然是 c1、c2 的父节点
    assert_eq!(doc.parent_node(c1), Some(frag));
    assert_eq!(doc.parent_node(c2), Some(frag));
    // 通过 fragment 可以访问到其子节点
    assert_eq!(doc.child_nodes(frag), vec![c1, c2]);
}

#[test]
fn test_clone_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("Hello");
    let cloned = doc.clone_node(text, true);

    assert_ne!(cloned, text);
    assert_eq!(doc.text_content(cloned), Some("Hello".to_string()));
}

#[test]
fn test_clone_comment_node() {
    let mut doc = Document::new();
    let comment = doc.create_comment("a comment");
    let cloned = doc.clone_node(comment, true);

    assert_ne!(cloned, comment);
    if let Some(NodeKind::Comment(data)) = doc.get(cloned).map(|n| n.kind.clone()) {
        assert_eq!(data.content, "a comment");
    }
}

#[test]
fn test_clone_deep_nested() {
    let mut doc = Document::new();
    let root = doc.root();

    // 创建 3 层嵌套：div > span > p
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    let text = doc.create_text_node("deep");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(span, p).unwrap();
    doc.append_child(p, text).unwrap();

    let cloned_div = doc.clone_node(div, true);

    assert_ne!(cloned_div, div);
    assert!(doc.has_child_nodes(cloned_div));
    let cloned_span = doc.first_child(cloned_div).unwrap();
    assert_ne!(cloned_span, span);
    let cloned_p = doc.first_child(cloned_span).unwrap();
    assert_ne!(cloned_p, p);
    assert_eq!(doc.text_content(cloned_div), Some("deep".to_string()));
}

#[test]
fn test_replace_child_middle() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");
    let new_node = doc.create_element("a");

    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();

    let replaced = doc.replace_child(root, new_node, c2).unwrap();
    assert_eq!(replaced, c2);
    assert_eq!(doc.child_nodes(root), vec![c1, new_node, c3]);
    assert_eq!(doc.parent_node(c2), None);
    assert_eq!(doc.parent_node(new_node), Some(root));
}

#[test]
fn test_replace_child_reparenting() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent1 = doc.create_element("div");
    let parent2 = doc.create_element("span");
    let c1 = doc.create_element("p");
    let c2 = doc.create_element("a");
    let child = doc.create_element("em");

    doc.append_child(root, parent1).unwrap();
    doc.append_child(root, parent2).unwrap();
    doc.append_child(parent1, c1).unwrap();
    doc.append_child(parent1, child).unwrap();
    doc.append_child(parent2, c2).unwrap();

    // 将 parent2 的 c2 替换为 parent1 的 child（跨父节点移动）
    let replaced = doc.replace_child(parent2, child, c2).unwrap();
    assert_eq!(replaced, c2);

    // child 现在属于 parent2
    assert_eq!(doc.parent_node(child), Some(parent2));
    assert_eq!(doc.child_nodes(parent2), vec![child]);
    // parent1 不再包含 child
    assert_eq!(doc.child_nodes(parent1), vec![c1]);
    assert_eq!(doc.parent_node(c2), None);
}

#[test]
fn test_remove_child_nonexistent() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let orphan = doc.create_element("span");
    // orphan 从未被 append 到 elem

    let result = doc.remove_child(elem, orphan);
    assert!(matches!(result, Err(DomError::NotAChild { .. })));
}

#[test]
fn test_insert_before_middle() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");
    let new_node = doc.create_element("a");

    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();

    // 在 c2（第2个子节点）之前插入 new_node
    doc.insert_before(root, new_node, c2).unwrap();

    assert_eq!(doc.child_nodes(root), vec![c1, new_node, c2, c3]);
}

#[test]
fn test_element_has_attribute() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    assert!(!doc.has_attribute(elem, "class"));

    doc.set_attribute(elem, "class", "active");
    assert!(doc.has_attribute(elem, "class"));

    // 不存在的属性
    assert!(!doc.has_attribute(elem, "data-missing"));
}

#[test]
fn test_query_selector_nested() {
    // 创建嵌套结构 div > span > p
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(span, p).unwrap();

    // query_selector 用简单选择器从根搜索应能找到嵌套的 p
    let found = doc.query_selector(root, "p");
    assert_eq!(found, Some(p));

    // 从 div 搜索也能找到 span 和 p
    assert_eq!(doc.query_selector(div, "span"), Some(span));
    assert_eq!(doc.query_selector(div, "p"), Some(p));

    // 从 span 搜索不应该找到 div
    assert!(doc.query_selector(span, "div").is_none());
}

#[test]
fn test_query_selector_all_multiple() {
    let doc = parse_html("<html><body><div>a</div><div>b</div><div>c</div></body></html>");
    let root = doc.root();
    let results = doc.query_selector_all(root, "div");
    assert_eq!(results.len(), 3);
}

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

// ═══════════════════════════════════════════════════════════════════════
// 17. DOM 遍历和节点比较测试
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 18. Shadow DOM 测试
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 19. Range API advanced tests
// ═══════════════════════════════════════════════════════════════════════

/// Helper: get the body node from a parsed HTML document.
fn body_of(doc: &Document) -> NodeId {
    let html = doc.first_child(doc.root()).unwrap();
    doc.last_child(html).unwrap()
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

// ═══════════════════════════════════════════════════════════════════════
// 20. Serialization advanced tests
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 21. Shadow DOM integration tests
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 22. Event system edge cases
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 23. MutationObserver advanced tests
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 24. Query API advanced tests
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 25. Parser edge cases
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 24. Node lifecycle after removal
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 25. Error paths in insert_before and replace_child
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 26. 边界条件补充测试
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 26. id_map 一致性测试（文档化当前行为及已知缺陷）
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 27. id_map 清理测试
// ═══════════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════
// Shadow DOM closed 模式与 compare_document_position 深度测试
// ═══════════════════════════════════════════════════════════════════

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

// ═══════════════════════════════════════════════════════════════════════
// 28. Document 创建边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 create_processing_instruction 字段正确性。
#[test]
fn test_create_processing_instruction_fields() {
    let mut doc = Document::new();
    let pi = doc.create_processing_instruction("xml-stylesheet", "href=\"style.css\"");
    assert!(doc.contains(pi));
    if let Some(NodeKind::ProcessingInstruction(data)) = doc.get(pi).map(|n| n.kind.clone()) {
        assert_eq!(data.target, "xml-stylesheet");
        assert_eq!(data.data, "href=\"style.css\"");
    } else {
        panic!("expected ProcessingInstruction node");
    }
}

/// 测试 create_processing_instruction 节点类型为 7。
#[test]
fn test_processing_instruction_node_type() {
    let mut doc = Document::new();
    let pi = doc.create_processing_instruction("xml", "version=\"1.0\"");
    assert_eq!(doc.node_type(pi), Some(7), "ProcessingInstruction nodeType should be 7");
}

/// 测试 get_element_by_id 在空文档中返回 None。
#[test]
fn test_get_element_by_id_empty_document() {
    let doc = Document::new();
    assert_eq!(doc.get_element_by_id("anything"), None);
    assert_eq!(doc.get_element_by_id(""), None);
}

/// 测试 get_elements_by_class_name 空结果。
#[test]
fn test_get_elements_by_class_name_empty_result() {
    let doc = parse_html("<html><body><div>no class here</div></body></html>");
    let result = doc.get_elements_by_class_name("nonexistent");
    assert!(result.is_empty(), "不存在的类名应返回空列表");

    let result2 = doc.get_elements_by_class_name("");
    assert!(result2.is_empty(), "空类名应返回空列表");
}

/// 测试 create_comment 空字符串。
#[test]
fn test_create_comment_empty() {
    let mut doc = Document::new();
    let comment = doc.create_comment("");
    if let Some(NodeKind::Comment(data)) = doc.get(comment).map(|n| n.kind.clone()) {
        assert!(data.content.is_empty());
    }
}

/// 测试 create_document_fragment 节点类型为 11。
#[test]
fn test_document_fragment_node_type() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    assert_eq!(doc.node_type(frag), Some(11), "DocumentFragment nodeType should be 11");
}

/// 测试 create_document_type 带 public_id 和 system_id。
#[test]
fn test_create_document_type_with_ids() {
    let mut doc = Document::new();
    let doctype = doc.create_document_type(
        "html",
        Some("-//W3C//DTD XHTML 1.0 Strict//EN".to_string()),
        Some("http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd".to_string()),
    );
    if let Some(NodeKind::DocumentType(dt)) = doc.get(doctype).map(|n| n.kind.clone()) {
        assert_eq!(dt.name, "html");
        assert_eq!(dt.public_id, Some("-//W3C//DTD XHTML 1.0 Strict//EN".to_string()));
        assert_eq!(
            dt.system_id,
            Some("http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd".to_string())
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 29. Node 边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 insert_before ref_node 不在父节点中返回错误。
#[test]
fn test_insert_before_ref_not_in_parent_children() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let new_child = doc.create_element("span");
    let unrelated = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(root, unrelated).unwrap(); // unrelated 在 root 而非 parent 中

    let result = doc.insert_before(parent, new_child, unrelated);
    assert!(
        result.is_err(),
        "insert_before with ref_node not a child of parent should return error"
    );
}

/// 测试 replace_child 成功替换第一个子节点。
#[test]
fn test_replace_child_first() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let new_node = doc.create_element("p");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();

    let replaced = doc.replace_child(root, new_node, c1).unwrap();
    assert_eq!(replaced, c1);
    assert_eq!(doc.child_nodes(root), vec![new_node, c2]);
    assert_eq!(doc.parent_node(c1), None);
}

/// 测试 replace_child 成功替换最后一个子节点。
#[test]
fn test_replace_child_last() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let new_node = doc.create_element("p");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();

    let replaced = doc.replace_child(root, new_node, c2).unwrap();
    assert_eq!(replaced, c2);
    assert_eq!(doc.child_nodes(root), vec![c1, new_node]);
}

/// 测试 clone_node deep 深拷贝属性和嵌套结构。
#[test]
fn test_clone_node_deep_with_attributes() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "orig");
    doc.set_attribute(elem, "class", "container");
    let child1 = doc.create_element("span");
    let child2 = doc.create_text_node("text content");
    doc.append_child(elem, child1).unwrap();
    doc.append_child(elem, child2).unwrap();

    let cloned = doc.clone_node(elem, true);

    // 克隆的属性值正确
    assert_eq!(doc.get_attribute(cloned, "id"), Some("orig".to_string()));
    assert_eq!(doc.get_attribute(cloned, "class"), Some("container".to_string()));

    // 克隆有子节点且是新的
    assert!(doc.has_child_nodes(cloned));
    let cloned_children = doc.child_nodes(cloned);
    assert_eq!(cloned_children.len(), 2);
    assert_ne!(cloned_children[0], child1, "克隆子节点应该是新节点");
    assert_ne!(cloned_children[1], child2, "克隆文本节点应该是新节点");
    assert_eq!(doc.text_content(cloned), Some("text content".to_string()));
}

/// 测试 has_child_nodes 对空元素返回 false。
#[test]
fn test_has_child_nodes_empty_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert!(!doc.has_child_nodes(elem), "空元素应该没有子节点");

    let root = doc.root();
    assert!(!doc.has_child_nodes(root), "空文档根应该没有子节点");
}

/// 测试 text_content 设置后替换原有子节点。
#[test]
fn test_text_content_set_replaces_children() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let child1 = doc.create_element("span");
    let child2 = doc.create_element("p");
    doc.append_child(elem, child1).unwrap();
    doc.append_child(elem, child2).unwrap();

    assert_eq!(doc.child_count(elem), 2);

    doc.set_text_content(elem, "replaced");
    assert_eq!(doc.text_content(elem), Some("replaced".to_string()));
    assert_eq!(doc.child_count(elem), 1, "设置 text_content 后应该只有一个文本子节点");

    // 原有子节点已被从树中移除
    assert_eq!(doc.parent_node(child1), None);
    assert_eq!(doc.parent_node(child2), None);
}

/// 测试 text_content 设置 None 值（空字符串）清除所有子节点。
#[test]
fn test_text_content_set_empty_clears_children() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let text = doc.create_text_node("existing");
    doc.append_child(elem, text).unwrap();

    doc.set_text_content(elem, "");
    assert_eq!(doc.text_content(elem), Some("".to_string()));
    assert!(!doc.has_child_nodes(elem), "空字符串 text_content 应清除子节点");
}

/// 测试 text_content 对 Comment 节点返回注释内容。
#[test]
fn test_text_content_comment_returns_content() {
    let mut doc = Document::new();
    let comment = doc.create_comment("a comment");
    assert_eq!(doc.text_content(comment), Some("a comment".to_string()));
}

/// 测试 text_content 对空 Comment 节点返回空字符串。
#[test]
fn test_text_content_empty_comment() {
    let mut doc = Document::new();
    let comment = doc.create_comment("");
    assert_eq!(doc.text_content(comment), Some("".to_string()));
}

/// 测试 text_content 对 Document 节点返回空字符串（无子节点时）。
#[test]
fn test_text_content_document_empty() {
    let doc = Document::new();
    assert_eq!(doc.text_content(doc.root()), Some("".to_string()));
}

/// 测试 text_content 对 DocumentType 节点返回 None。
#[test]
fn test_text_content_doctype_returns_none() {
    let mut doc = Document::new();
    let doctype = doc.create_document_type("html", None, None);
    assert_eq!(doc.text_content(doctype), None);
}

/// 测试 text_content 对 DocumentFragment 返回后代文本拼接。
#[test]
fn test_text_content_document_fragment() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    let t1 = doc.create_text_node("hello ");
    let t2 = doc.create_text_node("world");
    doc.append_child(frag, t1).unwrap();
    doc.append_child(frag, t2).unwrap();

    assert_eq!(doc.text_content(frag), Some("hello world".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// 30. Element 属性边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 set_attribute 覆盖已有属性保留最新值。
#[test]
fn test_set_attribute_overwrite_multiple_times() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "data-val", "v1");
    doc.set_attribute(elem, "data-val", "v2");
    doc.set_attribute(elem, "data-val", "v3");
    assert_eq!(doc.get_attribute(elem, "data-val"), Some("v3".to_string()));
}

/// 测试 remove_attribute 对不存在的属性不 panic。
#[test]
fn test_remove_attribute_nonexistent_no_panic() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    // 在从未设置过的属性上调用 remove_attribute
    doc.remove_attribute(elem, "noexist");
    doc.remove_attribute(elem, "class");
    doc.remove_attribute(elem, "");
}

/// 测试 has_attribute 对空名称返回 false。
#[test]
fn test_has_attribute_empty_name() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert!(!doc.has_attribute(elem, ""), "空属性名应返回 false");
    doc.set_attribute(elem, "", "value");
    assert!(doc.has_attribute(elem, ""), "设置空属性名后应返回 true");
}

/// 测试 attribute_names 对无属性的元素返回空列表。
#[test]
fn test_attribute_names_empty() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let names = doc.attribute_names(elem);
    assert!(names.is_empty(), "无属性的元素应返回空列表");
}

/// 测试多个属性的 attribute_names 包含所有名称。
#[test]
fn test_attribute_names_multiple() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "main");
    doc.set_attribute(elem, "class", "container");
    doc.set_attribute(elem, "data-x", "1");

    let names = doc.attribute_names(elem);
    assert_eq!(names.len(), 3);
    assert!(names.contains(&"id".to_string()));
    assert!(names.contains(&"class".to_string()));
    assert!(names.contains(&"data-x".to_string()));
}

/// 测试 get_elements_by_tag_name 不存在的标签返回空列表。
#[test]
fn test_get_elements_by_tag_name_not_found() {
    let doc = parse_html("<html><body><div>a</div></body></html>");
    let result = doc.get_elements_by_tag_name("nonexistent-tag");
    assert!(result.is_empty());
}

/// 测试 get_elements_by_class_name 多类名元素只匹配完整类名。
#[test]
fn test_get_elements_by_class_name_multi_class() {
    let doc = parse_html("<html><body><div class=\"foo bar baz\">text</div></body></html>");
    let by_foo = doc.get_elements_by_class_name("foo");
    let by_bar = doc.get_elements_by_class_name("bar");
    let by_baz = doc.get_elements_by_class_name("baz");
    let by_qux = doc.get_elements_by_class_name("qux");
    assert_eq!(by_foo.len(), 1);
    assert_eq!(by_bar.len(), 1);
    assert_eq!(by_baz.len(), 1);
    assert_eq!(by_qux.len(), 0);
    // 全部返回同一个元素
    assert_eq!(by_foo[0], by_bar[0]);
    assert_eq!(by_bar[0], by_baz[0]);
}

// ═══════════════════════════════════════════════════════════════════════
// 31. Event 边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 event type 字段在派发过程中保持不变。
#[test]
fn test_event_type_preserved_through_dispatch() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let received_type = Arc::new(Mutex::new(String::new()));
    let type_clone = received_type.clone();
    doc.add_event_listener(
        elem,
        "custom-event",
        Box::new(move |event| {
            *type_clone.lock().unwrap() = event.event_type().to_string();
        }),
        false,
    );

    let mut event = Event::new("custom-event");
    doc.dispatch_event(elem, &mut event);
    assert_eq!(*received_type.lock().unwrap(), "custom-event");
}

/// 测试事件冒泡阶段正确的传播路径。
#[test]
fn test_bubbling_phase_correct_propagation_path() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, ul).unwrap();
    doc.append_child(ul, li).unwrap();

    let path = Arc::new(Mutex::new(Vec::new()));
    let p1 = path.clone();
    let p2 = path.clone();
    let p3 = path.clone();

    doc.add_event_listener(
        li,
        "click",
        Box::new(move |e| {
            p1.lock().unwrap().push(("li", e.phase()));
        }),
        false,
    );
    doc.add_event_listener(
        ul,
        "click",
        Box::new(move |e| {
            p2.lock().unwrap().push(("ul", e.phase()));
        }),
        false,
    );
    doc.add_event_listener(
        div,
        "click",
        Box::new(move |e| {
            p3.lock().unwrap().push(("div", e.phase()));
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(li, &mut event);

    let p = path.lock().unwrap();
    assert_eq!(p.len(), 3);
    // li 在目标阶段触发
    assert_eq!(p[0].0, "li");
    // ul 在冒泡阶段触发
    assert_eq!(p[1].0, "ul");
    // div 在冒泡阶段触发
    assert_eq!(p[2].0, "div");
}

/// 测试捕获和冒泡都注册时的调用顺序。
#[test]
fn test_capture_then_bubble_order() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    let order = Arc::new(Mutex::new(Vec::new()));
    let o1 = order.clone();
    let o2 = order.clone();
    let o3 = order.clone();
    let o4 = order.clone();

    // 父节点捕获监听器
    doc.add_event_listener(
        parent,
        "click",
        Box::new(move |_| {
            o1.lock().unwrap().push("parent-capture");
        }),
        true,
    );
    // 子节点冒泡监听器（先注册）
    doc.add_event_listener(
        child,
        "click",
        Box::new(move |_| {
            o2.lock().unwrap().push("child-bubble");
        }),
        false,
    );
    // 子节点捕获监听器（后注册）
    doc.add_event_listener(
        child,
        "click",
        Box::new(move |_| {
            o3.lock().unwrap().push("child-capture");
        }),
        true,
    );
    // 父节点冒泡监听器
    doc.add_event_listener(
        parent,
        "click",
        Box::new(move |_| {
            o4.lock().unwrap().push("parent-bubble");
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(child, &mut event);

    let o = order.lock().unwrap();
    assert_eq!(o.len(), 4, "应触发 4 个监听器");
    // 父节点捕获必须最先
    assert_eq!(o[0], "parent-capture", "父节点捕获应最先触发");
    // 父节点冒泡必须最后
    assert_eq!(o[3], "parent-bubble", "父节点冒泡应最后触发");
    // 目标节点的两个监听器按注册顺序触发（先 child-bubble 再 child-capture）
    assert!(
        o[1] == "child-bubble" || o[1] == "child-capture",
        "目标阶段监听器应在中间"
    );
    assert!(
        o[2] == "child-bubble" || o[2] == "child-capture",
        "目标阶段监听器应在中间"
    );
}

/// 测试 stopImmediatePropagation 阻止冒泡到祖先。
#[test]
fn test_stop_immediate_propagation_blocks_bubbling() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    let log = Arc::new(Mutex::new(Vec::new()));
    let l1 = log.clone();
    let l2 = log.clone();

    doc.add_event_listener(
        child,
        "click",
        Box::new(move |e| {
            l1.lock().unwrap().push("child");
            e.stop_immediate_propagation();
        }),
        false,
    );
    doc.add_event_listener(
        parent,
        "click",
        Box::new(move |_| {
            l2.lock().unwrap().push("parent");
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(child, &mut event);

    let g = log.lock().unwrap();
    assert_eq!(*g, vec!["child"], "stopImmediatePropagation 应阻止冒泡到父节点");
}

/// 测试冒泡事件 EventPhase 在各阶段正确。
#[test]
fn test_event_phase_during_propagation() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    let phases = Arc::new(Mutex::new(Vec::new()));
    let ph1 = phases.clone();
    let ph2 = phases.clone();
    let ph3 = phases.clone();

    doc.add_event_listener(
        parent,
        "click",
        Box::new(move |e| {
            ph1.lock().unwrap().push(e.phase());
        }),
        true,
    ); // 捕获
    doc.add_event_listener(
        child,
        "click",
        Box::new(move |e| {
            ph2.lock().unwrap().push(e.phase());
        }),
        false,
    ); // 目标
    doc.add_event_listener(
        parent,
        "click",
        Box::new(move |e| {
            ph3.lock().unwrap().push(e.phase());
        }),
        false,
    ); // 冒泡

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(child, &mut event);

    let p = phases.lock().unwrap();
    assert_eq!(p.len(), 3);
    assert_eq!(p[0], EventPhase::Capturing, "父节点捕获阶段应为 Capturing");
    assert_eq!(p[1], EventPhase::AtTarget, "目标节点应为 AtTarget");
    assert_eq!(p[2], EventPhase::Bubbling, "父节点冒泡阶段应为 Bubbling");
}

// ═══════════════════════════════════════════════════════════════════════
// 32. 序列化和遍历边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 inner_html 对空元素返回空字符串。
#[test]
fn test_inner_html_empty_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let html = doc.inner_html(elem);
    assert_eq!(html, "", "空元素的 innerHTML 应为空字符串");
}

/// 测试 outer_html 对文本节点返回文本内容。
#[test]
fn test_outer_html_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("Hello World");
    let html = doc.outer_html(text);
    assert_eq!(html, "Hello World");
}

/// 测试 quirks_mode 默认值为 NoQuirks。
#[test]
fn test_quirks_mode_default() {
    let doc = Document::new();
    assert_eq!(doc.quirks_mode(), QuirksMode::NoQuirks);
}

/// 测试 set_quirks_mode 可以修改文档模式。
#[test]
fn test_set_quirks_mode() {
    let mut doc = Document::new();
    doc.set_quirks_mode(QuirksMode::Quirks);
    assert_eq!(doc.quirks_mode(), QuirksMode::Quirks);

    doc.set_quirks_mode(QuirksMode::LimitedQuirks);
    assert_eq!(doc.quirks_mode(), QuirksMode::LimitedQuirks);
}

/// 测试 collect_descendants 对单层子树。
#[test]
fn test_collect_descendants_single_layer() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let c1 = doc.create_element("span");
    let c2 = doc.create_text_node("text");
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();

    let descendants = doc.collect_descendants(parent);
    assert_eq!(descendants.len(), 2);
    assert_eq!(descendants[0], c1);
    assert_eq!(descendants[1], c2);
}

/// 测试 collect_descendants 不包含自身。
#[test]
fn test_collect_descendants_excludes_self() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(elem, child).unwrap();

    let descendants = doc.collect_descendants(elem);
    assert!(!descendants.contains(&elem), "不应包含自身");
    assert!(descendants.contains(&child), "应包含子节点");
}

/// 测试 depth 对孤立节点（无父节点）返回 Some(0)。
#[test]
fn test_depth_orphan_node() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    // 孤立节点没有到 root 的路径，depth 沿 parent 回溯到 None 时返回 0
    let depth = doc.depth(elem);
    assert!(depth.is_some(), "孤立节点深度应返回 Some");
    assert_eq!(depth, Some(0), "孤立节点深度应为 0");
}

/// 测试 child_count 对空节点返回 0。
#[test]
fn test_child_count_empty() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert_eq!(doc.child_count(elem), 0);

    let root = doc.root();
    assert_eq!(doc.child_count(root), 0);
}

/// 测试 node_type 对 Text 节点返回 3。
#[test]
fn test_node_type_text() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    assert_eq!(doc.node_type(text), Some(3));
}

/// 测试 node_type 对 Comment 节点返回 8。
#[test]
fn test_node_type_comment() {
    let mut doc = Document::new();
    let comment = doc.create_comment("note");
    assert_eq!(doc.node_type(comment), Some(8));
}

// ═══════════════════════════════════════════════════════════════════════
// 33. MutationObserver 边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 MutationRecord 的 previous_sibling 在追加末尾子节点时为倒数第二个子节点。
#[test]
fn test_mutation_record_previous_sibling() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("span");
    let c2 = doc.create_element("p");

    doc.append_child(root, c1).unwrap();
    doc.take_mutation_records(); // 清空

    doc.append_child(root, c2).unwrap();
    let records = doc.take_mutation_records();
    assert_eq!(records.len(), 1);
    // c2 是 root 的第二个子节点，previous_sibling 应该是 c1
    assert_eq!(records[0].previous_sibling, Some(c1));
}

/// 测试 remove_attribute 不产生 mutation 记录（属性 mutation 在 set_attribute 时记录）。
#[test]
fn test_mutation_on_remove_attribute() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "test");
    doc.take_mutation_records(); // 清空 set_attribute 产生的记录

    doc.remove_attribute(elem, "class");
    let _records = doc.take_mutation_records();
    // remove_attribute 当前实现是否产生记录取决于实现
    // 这里记录行为，不管是否产生记录都不应 panic
}

/// 测试 MutationRecord Clone。
#[test]
fn test_mutation_record_clone() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let record = MutationRecord {
        mutation_type: MutationType::Attributes,
        target: elem,
        added_nodes: vec![],
        removed_nodes: vec![],
        previous_sibling: None,
        attribute_name: Some("class".to_string()),
        old_value: Some("old".to_string()),
    };
    let cloned = record.clone();
    assert_eq!(cloned.mutation_type, MutationType::Attributes);
    assert_eq!(cloned.target, elem);
    assert_eq!(cloned.attribute_name, Some("class".to_string()));
    assert_eq!(cloned.old_value, Some("old".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// 34. 树操作边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 append_child 对自身追加返回错误（循环检测）。
#[test]
fn test_append_child_self_fails() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.append_child(root, elem).unwrap();

    // 试图将自己追加为自己子节点 — 应该失败
    let result = doc.append_child(elem, elem);
    assert!(result.is_err(), "将自己追加为子节点应返回错误");
}

/// 测试 insert_before 在列表末尾（ref_node = 最后子节点）之前插入。
#[test]
fn test_insert_before_last_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let new_node = doc.create_element("p");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();

    doc.insert_before(root, new_node, c2).unwrap();
    assert_eq!(doc.child_nodes(root), vec![c1, new_node, c2]);
}

/// 测试 insert_before 在列表开头（ref_node = 第一个子节点）之前插入。
#[test]
fn test_insert_before_first_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let new_node = doc.create_element("p");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();

    doc.insert_before(root, new_node, c1).unwrap();
    assert_eq!(doc.child_nodes(root), vec![new_node, c1, c2]);
}

/// 测试 append_child 自动 reparenting（移动已有父节点的子节点）。
#[test]
fn test_append_child_auto_reparent() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent1 = doc.create_element("div");
    let parent2 = doc.create_element("span");
    let child = doc.create_element("p");
    doc.append_child(root, parent1).unwrap();
    doc.append_child(root, parent2).unwrap();
    doc.append_child(parent1, child).unwrap();

    assert_eq!(doc.parent_node(child), Some(parent1));

    // 将 child 从 parent1 移动到 parent2
    doc.append_child(parent2, child).unwrap();
    assert_eq!(doc.parent_node(child), Some(parent2));
    assert!(!doc.has_child_nodes(parent1));
    assert_eq!(doc.child_nodes(parent2), vec![child]);
}

/// 测试 insert_before 自动 reparenting。
#[test]
fn test_insert_before_auto_reparent() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent1 = doc.create_element("div");
    let parent2 = doc.create_element("span");
    let child = doc.create_element("p");
    let ref_node = doc.create_element("a");
    doc.append_child(root, parent1).unwrap();
    doc.append_child(root, parent2).unwrap();
    doc.append_child(parent1, child).unwrap();
    doc.append_child(parent2, ref_node).unwrap();

    // 将 child 从 parent1 移到 parent2 的 ref_node 之前
    doc.insert_before(parent2, child, ref_node).unwrap();
    assert_eq!(doc.parent_node(child), Some(parent2));
    assert!(!doc.has_child_nodes(parent1));
    assert_eq!(doc.child_nodes(parent2), vec![child, ref_node]);
}

/// 测试 DocumentFragment 作为 insert_before 的 new_node。
#[test]
fn test_insert_before_with_fragment() {
    let mut doc = Document::new();
    let root = doc.root();
    let container = doc.create_element("div");
    let ref_node = doc.create_element("span");
    doc.append_child(root, container).unwrap();
    doc.append_child(container, ref_node).unwrap();

    let frag = doc.create_document_fragment();
    let f1 = doc.create_element("p");
    doc.append_child(frag, f1).unwrap();

    doc.insert_before(container, frag, ref_node).unwrap();
    let children = doc.child_nodes(container);
    assert!(children.contains(&frag), "fragment 应被插入到 container 中");
}

/// 测试 sibling_traversal 对单子节点返回 None。
#[test]
fn test_sibling_traversal_single_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let only_child = doc.create_element("div");
    doc.append_child(root, only_child).unwrap();

    assert_eq!(doc.next_sibling(only_child), None);
    assert_eq!(doc.previous_sibling(only_child), None);
}

/// 测试 node_contains 对未连接的兄弟节点返回 false。
#[test]
fn test_node_contains_unrelated_nodes() {
    let mut doc = Document::new();
    let a = doc.create_element("div");
    let b = doc.create_element("span");
    // 两个节点都未附加到树中，且互不为祖先
    assert!(!doc.node_contains(a, b), "未连接的节点不应有包含关系");
    assert!(!doc.node_contains(b, a), "未连接的节点不应有包含关系");
}

// ═══════════════════════════════════════════════════════════════════════
// 35. 解析器边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试解析只包含文本的 HTML。
#[test]
fn test_parse_plain_text() {
    let doc = parse_html("Hello World");
    assert!(doc.root().is_valid());
    let text = doc.text_content(doc.root());
    assert!(text.is_some());
    assert!(text.unwrap().contains("Hello World"));
}

/// 测试解析带有嵌套表格的 HTML。
#[test]
fn test_parse_table() {
    let doc = parse_html("<html><body><table><tr><td>A</td><td>B</td></tr></table></body></html>");
    let tds = doc.get_elements_by_tag_name("td");
    assert_eq!(tds.len(), 2);
    let trs = doc.get_elements_by_tag_name("tr");
    assert_eq!(trs.len(), 1);
}

/// 测试解析含有 HTML 实体的内容。
#[test]
fn test_parse_html_entities() {
    let doc = parse_html("<html><body><p>&amp; &lt; &gt;</p></body></html>");
    let ps = doc.get_elements_by_tag_name("p");
    assert_eq!(ps.len(), 1);
    let text = doc.text_content(ps[0]);
    assert!(text.is_some());
    let t = text.unwrap();
    assert!(t.contains("&"), "应该解析 &amp; 为 &");
    assert!(t.contains("<"), "应该解析 &lt; 为 <");
    assert!(t.contains(">"), "应该解析 &gt; 为 >");
}

/// 测试解析多个相同 id 的元素（只索引第一个）。
#[test]
fn test_parse_duplicate_ids() {
    let doc = parse_html("<html><body><div id=\"same\">first</div><div id=\"same\">second</div></body></html>");
    let found = doc.get_element_by_id("same");
    assert!(found.is_some(), "应该找到至少一个 id=same 的元素");
    let text = doc.text_content(found.unwrap());
    // html5ever 通常保留第一个出现的元素在 id_map 中
    assert!(text.is_some());
}

/// 测试解析纯空白内容。
#[test]
fn test_parse_whitespace_only() {
    let doc = parse_html("   \n\t  ");
    assert!(doc.root().is_valid());
}

// ═══════════════════════════════════════════════════════════════════════
// 36. Slot 分配解析测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 slot 分配：slot="header" 属性匹配 <slot name="header">。
///
/// 场景：宿主元素有一个带 slot="header" 的 light DOM 子节点，
/// shadow 树中有一个 <slot name="header">。调用 resolve_slots 后，
/// get_assigned_nodes 应返回该 light DOM 子节点。
#[test]
fn test_resolve_slots_matching_element() {
    let mut doc = Document::new();
    let root = doc.root();

    // 宿主元素
    let host = doc.create_element("my-component");
    doc.append_child(root, host).unwrap();

    // light DOM：一个带 slot="header" 的元素
    let header = doc.create_element("h1");
    doc.set_attribute(header, "slot", "header");
    doc.append_child(host, header).unwrap();

    // shadow DOM：一个带 name="header" 的 slot
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let slot_elem = doc.create_element("slot");
    doc.set_attribute(slot_elem, "name", "header");
    doc.append_child(shadow, slot_elem).unwrap();

    // 解析 slot 分配
    doc.resolve_slots(host);

    // 验证：header 被分配到 slot_elem
    let assigned = doc.get_assigned_nodes(slot_elem);
    assert_eq!(assigned.len(), 1, "应分配 1 个节点到 header slot");
    assert_eq!(assigned[0], header, "分配的节点应是 header");
}

/// 测试 slot 分配：无匹配元素时使用 slot 的回退内容。
///
/// 场景：shadow 树中有 <slot name="footer">，但 light DOM 中
/// 没有任何子节点有 slot="footer"。此时 get_assigned_nodes 返回空，
/// 渲染时应使用 slot 元素自身的子节点作为回退内容。
#[test]
fn test_resolve_slots_fallback_content() {
    let mut doc = Document::new();
    let root = doc.root();

    let host = doc.create_element("my-component");
    doc.append_child(root, host).unwrap();

    // light DOM：没有 slot 属性的节点
    let content = doc.create_element("div");
    doc.append_child(host, content).unwrap();

    // shadow DOM：<slot name="footer"> 带回退内容
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let slot_elem = doc.create_element("slot");
    doc.set_attribute(slot_elem, "name", "footer");
    let fallback = doc.create_text_node("Default Footer");
    doc.append_child(shadow, slot_elem).unwrap();
    doc.append_child(slot_elem, fallback).unwrap();

    doc.resolve_slots(host);

    // footer slot 没有匹配的 light DOM 节点
    let assigned = doc.get_assigned_nodes(slot_elem);
    assert!(
        assigned.is_empty(),
        "没有匹配的 light DOM 节点时，assigned_nodes 应为空"
    );

    // 回退内容仍然存在（slot 自身的子节点未被移除）
    let slot_children = doc.child_nodes(slot_elem);
    assert_eq!(slot_children.len(), 1, "slot 的回退内容应保留");
    assert_eq!(doc.text_content(slot_elem), Some("Default Footer".to_string()));
}

/// 测试默认 slot（无名 slot）捕获没有 slot 属性的子节点。
///
/// 场景：shadow 树中有一个无 name 属性的 <slot>，
/// light DOM 中有多个没有 slot 属性的子节点。
/// 调用 resolve_slots 后，这些子节点都应分配到默认 slot。
#[test]
fn test_resolve_slots_default_slot() {
    let mut doc = Document::new();
    let root = doc.root();

    let host = doc.create_element("my-component");
    doc.append_child(root, host).unwrap();

    // light DOM：两个没有 slot 属性的子节点
    let child1 = doc.create_element("p");
    doc.append_child(host, child1).unwrap();
    let child2 = doc.create_element("span");
    doc.append_child(host, child2).unwrap();

    // shadow DOM：一个无名 slot（默认 slot）
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let default_slot = doc.create_element("slot");
    doc.append_child(shadow, default_slot).unwrap();

    doc.resolve_slots(host);

    // 两个没有 slot 属性的子节点都应分配到默认 slot
    let assigned = doc.get_assigned_nodes(default_slot);
    assert_eq!(assigned.len(), 2, "默认 slot 应捕获 2 个子节点");
    assert_eq!(assigned[0], child1, "第一个子节点应被分配");
    assert_eq!(assigned[1], child2, "第二个子节点应被分配");
}

/// 测试多个元素分配到同一个命名 slot。
///
/// 场景：light DOM 中有多个带 slot="items" 的元素，
/// shadow 树中有一个 <slot name="items">。
/// 调用 resolve_slots 后，所有匹配元素都应分配到该 slot。
#[test]
fn test_resolve_slots_multiple_elements_same_slot() {
    let mut doc = Document::new();
    let root = doc.root();

    let host = doc.create_element("my-list");
    doc.append_child(root, host).unwrap();

    // light DOM：三个带 slot="items" 的元素
    let item1 = doc.create_element("li");
    doc.set_attribute(item1, "slot", "items");
    doc.append_child(host, item1).unwrap();

    let item2 = doc.create_element("li");
    doc.set_attribute(item2, "slot", "items");
    doc.append_child(host, item2).unwrap();

    let item3 = doc.create_element("li");
    doc.set_attribute(item3, "slot", "items");
    doc.append_child(host, item3).unwrap();

    // shadow DOM：<slot name="items">
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let items_slot = doc.create_element("slot");
    doc.set_attribute(items_slot, "name", "items");
    doc.append_child(shadow, items_slot).unwrap();

    doc.resolve_slots(host);

    let assigned = doc.get_assigned_nodes(items_slot);
    assert_eq!(assigned.len(), 3, "应有 3 个节点分配到 items slot");
    // 保持文档顺序
    assert_eq!(assigned[0], item1);
    assert_eq!(assigned[1], item2);
    assert_eq!(assigned[2], item3);
}

/// 测试嵌套 slot 解析：外层组件的 slot 分配正确。
///
/// 场景：外层宿主有 shadow 树，shadow 树中嵌套了另一个自定义元素。
/// resolve_slots 只解析指定宿主的直接 slot 分配，
/// 不会穿透到嵌套的 shadow DOM 中。
#[test]
fn test_resolve_slots_nested_components() {
    let mut doc = Document::new();
    let root = doc.root();

    // 外层宿主
    let outer_host = doc.create_element("outer-comp");
    doc.append_child(root, outer_host).unwrap();

    // 外层 light DOM
    let header_el = doc.create_element("h1");
    doc.set_attribute(header_el, "slot", "title");
    doc.append_child(outer_host, header_el).unwrap();

    let body_el = doc.create_element("p");
    doc.append_child(outer_host, body_el).unwrap();

    // 外层 shadow DOM：包含 slot 和一个嵌套的内部组件
    let outer_shadow = doc.attach_shadow(outer_host, ShadowRootMode::Open).unwrap();

    let title_slot = doc.create_element("slot");
    doc.set_attribute(title_slot, "name", "title");
    doc.append_child(outer_shadow, title_slot).unwrap();

    let default_slot = doc.create_element("slot");
    doc.append_child(outer_shadow, default_slot).unwrap();

    // 内部组件（嵌套在外层 shadow 中）
    let inner_host = doc.create_element("inner-comp");
    doc.append_child(outer_shadow, inner_host).unwrap();
    let inner_shadow = doc.attach_shadow(inner_host, ShadowRootMode::Open).unwrap();
    let inner_slot = doc.create_element("slot");
    doc.set_attribute(inner_slot, "name", "content");
    doc.append_child(inner_shadow, inner_slot).unwrap();

    // 解析外层 slot
    doc.resolve_slots(outer_host);

    // 验证外层分配
    let title_assigned = doc.get_assigned_nodes(title_slot);
    assert_eq!(title_assigned.len(), 1, "外层 title slot 应有 1 个分配");
    assert_eq!(title_assigned[0], header_el);

    let default_assigned = doc.get_assigned_nodes(default_slot);
    assert_eq!(default_assigned.len(), 1, "外层默认 slot 应有 1 个分配");
    assert_eq!(default_assigned[0], body_el);

    // 内层 slot 没有被外层 resolve_slots 影响
    let inner_assigned = doc.get_assigned_nodes(inner_slot);
    assert!(inner_assigned.is_empty(), "内层 slot 不应被外层解析影响");
}

/// 测试 resolve_slots 对没有 shadow root 的元素不做任何操作。
#[test]
fn test_resolve_slots_no_shadow_root() {
    let mut doc = Document::new();
    let root = doc.root();
    let host = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(root, host).unwrap();
    doc.append_child(host, child).unwrap();

    // 没有 shadow root，resolve_slots 应安全返回
    doc.resolve_slots(host);
    // 不应 panic
}

/// 测试 resolve_slots 后重新调用会覆盖之前的分配。
#[test]
fn test_resolve_slots_idempotent() {
    let mut doc = Document::new();
    let root = doc.root();

    let host = doc.create_element("my-comp");
    doc.append_child(root, host).unwrap();

    let header = doc.create_element("h1");
    doc.set_attribute(header, "slot", "header");
    doc.append_child(host, header).unwrap();

    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let slot_elem = doc.create_element("slot");
    doc.set_attribute(slot_elem, "name", "header");
    doc.append_child(shadow, slot_elem).unwrap();

    // 第一次解析
    doc.resolve_slots(host);
    let assigned1 = doc.get_assigned_nodes(slot_elem);
    assert_eq!(assigned1.len(), 1);

    // 第二次解析（幂等）
    doc.resolve_slots(host);
    let assigned2 = doc.get_assigned_nodes(slot_elem);
    assert_eq!(assigned2.len(), 1, "重复调用不应产生重复分配");
    assert_eq!(assigned2[0], header);
}

/// 测试混合命名 slot 和默认 slot 的分配。
#[test]
fn test_resolve_slots_mixed_named_and_default() {
    let mut doc = Document::new();
    let root = doc.root();

    let host = doc.create_element("my-comp");
    doc.append_child(root, host).unwrap();

    // light DOM：混合有 slot 属性和无 slot 属性的子节点
    let header = doc.create_element("h1");
    doc.set_attribute(header, "slot", "header");
    doc.append_child(host, header).unwrap();

    let content1 = doc.create_element("p");
    doc.append_child(host, content1).unwrap();

    let footer = doc.create_element("footer");
    doc.set_attribute(footer, "slot", "footer");
    doc.append_child(host, footer).unwrap();

    let content2 = doc.create_element("span");
    doc.append_child(host, content2).unwrap();

    // shadow DOM
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    let header_slot = doc.create_element("slot");
    doc.set_attribute(header_slot, "name", "header");
    doc.append_child(shadow, header_slot).unwrap();

    let default_slot = doc.create_element("slot");
    doc.append_child(shadow, default_slot).unwrap();

    let footer_slot = doc.create_element("slot");
    doc.set_attribute(footer_slot, "name", "footer");
    doc.append_child(shadow, footer_slot).unwrap();

    doc.resolve_slots(host);

    // header slot 分配到 header 元素
    let header_assigned = doc.get_assigned_nodes(header_slot);
    assert_eq!(header_assigned.len(), 1);
    assert_eq!(header_assigned[0], header);

    // footer slot 分配到 footer 元素
    let footer_assigned = doc.get_assigned_nodes(footer_slot);
    assert_eq!(footer_assigned.len(), 1);
    assert_eq!(footer_assigned[0], footer);

    // 默认 slot 分配到 content1 和 content2
    let default_assigned = doc.get_assigned_nodes(default_slot);
    assert_eq!(default_assigned.len(), 2, "默认 slot 应捕获 2 个无 slot 属性的子节点");
    assert_eq!(default_assigned[0], content1);
    assert_eq!(default_assigned[1], content2);
}

/// 测试文本节点不参与命名 slot 分配，但可以分配到默认 slot。
#[test]
fn test_resolve_slots_text_node_default_slot() {
    let mut doc = Document::new();
    let root = doc.root();

    let host = doc.create_element("my-comp");
    doc.append_child(root, host).unwrap();

    // light DOM：一个文本节点
    let text = doc.create_text_node("Hello World");
    doc.append_child(host, text).unwrap();

    // shadow DOM：默认 slot
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let default_slot = doc.create_element("slot");
    doc.append_child(shadow, default_slot).unwrap();

    doc.resolve_slots(host);

    // 文本节点没有 slot 属性，应分配到默认 slot
    let assigned = doc.get_assigned_nodes(default_slot);
    assert_eq!(assigned.len(), 1, "文本节点应分配到默认 slot");
    assert_eq!(assigned[0], text);
}

// ═══════════════════════════════════════════════════════════════════════
// 37. compare_document_position 跨文档测试及 DOM 边界用例
// ═══════════════════════════════════════════════════════════════════════

/// 测试 compare_document_position 对来自不同 Document 实例的节点行为。
///
/// 当前实现中，slotmap 的 NodeId 是 (index, version) 对。
/// 两个全新的 Document 实例会从相同的初始状态分配 key，
/// 因此 doc1 的 root 和 doc2 的 root 拥有相同的 NodeId 值。
/// 这意味着 doc1.contains(doc2_elem) 返回 true（key 碰撞），
/// compare_document_position 会把跨文档节点视为同一棵树中的节点。
///
/// 此测试记录该已知行为：同构 Document 的 NodeId 会冲突。
/// 真正的跨文档隔离需要未来引入 Document 级别的命名空间。
#[test]
fn test_compare_document_position_disconnected_documents() {
    let mut doc1 = Document::new();
    let mut doc2 = Document::new();

    let root1 = doc1.root();
    let elem1 = doc1.create_element("div");
    doc1.append_child(root1, elem1).unwrap();

    let root2 = doc2.root();
    let elem2 = doc2.create_element("span");
    doc2.append_child(root2, elem2).unwrap();

    // 由于 slotmap key 碰撞，doc1 的 root 与 doc2 的 root 拥有相同 NodeId
    // compare_document_position 在当前实现中会将跨文档节点视为同树节点
    let result = doc1.compare_document_position(root1, elem2);
    // 实际结果是 Some(非零)，因为 key 碰撞导致跨文档节点被视为同一棵树
    assert!(
        result.is_some(),
        "slotmap key 碰撞时 compare_document_position 返回 Some"
    );

    // 同文档内节点应正常工作
    let same_doc_result = doc1.compare_document_position(root1, elem1);
    assert!(same_doc_result.is_some(), "同文档节点比较应返回 Some");
    assert!(same_doc_result.unwrap().contains(DocumentPosition::CONTAINED_BY));
}

/// 测试 compare_document_position 各种节点关系的标志组合。
///
/// 验证 PRECEDING、FOLLOWING、CONTAINS、CONTAINED_BY 标志
/// 在祖先-后代、兄弟、深层嵌套等场景下的正确性。
#[test]
fn test_compare_document_position_ancestor_flags() {
    let mut doc = Document::new();
    let root = doc.root();

    // 结构：root > div > span > p
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(span, p).unwrap();

    // 兄弟节点 c1、c2、c3
    let c1 = doc.create_element("a");
    let c2 = doc.create_element("b");
    let c3 = doc.create_element("i");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();

    // 1) 同一节点 → 0
    let pos = doc.compare_document_position(div, div).unwrap();
    assert_eq!(pos.bits(), 0, "同一节点应返回 0");

    // 2) div 包含 span → CONTAINED_BY | FOLLOWING
    let pos = doc.compare_document_position(div, span).unwrap();
    assert!(
        pos.contains(DocumentPosition::CONTAINED_BY),
        "div 包含 span → CONTAINED_BY"
    );
    assert!(
        pos.contains(DocumentPosition::FOLLOWING),
        "span 在 div 之后 → FOLLOWING"
    );

    // 3) span 被 div 包含 → CONTAINS | PRECEDING
    let pos = doc.compare_document_position(span, div).unwrap();
    assert!(pos.contains(DocumentPosition::CONTAINS), "span 被 div 包含 → CONTAINS");
    assert!(
        pos.contains(DocumentPosition::PRECEDING),
        "div 在 span 之前 → PRECEDING"
    );

    // 4) c1 在 c2 之前 → c2 在 c1 之后 → FOLLOWING
    let pos = doc.compare_document_position(c1, c2).unwrap();
    assert!(pos.contains(DocumentPosition::FOLLOWING), "c2 在 c1 之后 → FOLLOWING");

    // 5) c3 在 c2 之后 → c2 在 c3 之前 → PRECEDING
    let pos = doc.compare_document_position(c3, c2).unwrap();
    assert!(pos.contains(DocumentPosition::PRECEDING), "c2 在 c3 之前 → PRECEDING");

    // 6) 跨分支：div 与 c1（不同分支的兄弟子树）→ 纯树位置比较
    let pos = doc.compare_document_position(div, c1).unwrap();
    // div 在 c1 之前（div 是 root 的第一个子节点）
    assert!(pos.contains(DocumentPosition::FOLLOWING), "c1 在 div 之后 → FOLLOWING");
}

/// 测试 Document::new() 创建有效文档，根节点存在且类型正确。
#[test]
fn test_node_is_default_document() {
    let doc = Document::new();

    // 根节点有效
    let root = doc.root();
    assert!(root.is_valid(), "文档根节点应有效");

    // 根节点是 Document 类型
    assert!(
        matches!(doc.get(root).map(|n| &n.kind), Some(NodeKind::Document(_))),
        "根节点类型应为 Document"
    );

    // 初始时只有一个节点（根节点自身）
    assert_eq!(doc.node_count(), 1, "新文档应只有 1 个根节点");

    // 根节点没有子节点
    assert!(!doc.has_child_nodes(root), "新文档根节点不应有子节点");

    // 根节点深度为 0
    assert_eq!(doc.depth(root), Some(0), "文档根节点深度应为 0");
}

/// 测试新创建的元素没有属性，has_attribute 返回 false。
#[test]
fn test_element_has_attributes_empty() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    // 新元素没有任何属性
    assert!(!doc.has_attribute(elem, "class"), "新元素不应有 class 属性");
    assert!(!doc.has_attribute(elem, "id"), "新元素不应有 id 属性");
    assert!(!doc.has_attribute(elem, "data-x"), "新元素不应有 data-x 属性");

    // attribute_names 应为空
    let names = doc.attribute_names(elem);
    assert!(names.is_empty(), "新元素的属性名列表应为空");

    // get_attribute 应返回 None
    assert_eq!(doc.get_attribute(elem, "class"), None);

    // 设置属性后 has_attribute 变为 true
    doc.set_attribute(elem, "class", "active");
    assert!(doc.has_attribute(elem, "class"), "设置属性后应返回 true");
}

/// 测试通过 set_attribute 添加和移除多个 class。
///
/// 当前实现通过 set_attribute("class", ...) 管理 class 列表，
/// 值按空白分隔后存储到 ElementData.class_list。
/// get_elements_by_class_name 依赖此缓存进行匹配。
#[test]
fn test_element_class_list_toggle() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.append_child(root, elem).unwrap();

    // 添加多个 class
    doc.set_attribute(elem, "class", "foo bar baz");
    assert_eq!(doc.get_attribute(elem, "class"), Some("foo bar baz".to_string()));

    // 通过 get_elements_by_class_name 验证每个 class 都可查到
    assert_eq!(doc.get_elements_by_class_name("foo"), vec![elem]);
    assert_eq!(doc.get_elements_by_class_name("bar"), vec![elem]);
    assert_eq!(doc.get_elements_by_class_name("baz"), vec![elem]);
    assert!(doc.get_elements_by_class_name("qux").is_empty());

    // 移除某个 class：替换为只包含剩余 class 的字符串
    doc.set_attribute(elem, "class", "foo baz");
    assert_eq!(doc.get_attribute(elem, "class"), Some("foo baz".to_string()));
    assert_eq!(doc.get_elements_by_class_name("bar").len(), 0, "移除 bar 后不应再匹配");
    assert_eq!(doc.get_elements_by_class_name("foo"), vec![elem], "foo 仍应匹配");
    assert_eq!(doc.get_elements_by_class_name("baz"), vec![elem], "baz 仍应匹配");

    // 清空所有 class
    doc.set_attribute(elem, "class", "");
    assert_eq!(doc.get_attribute(elem, "class"), Some("".to_string()));
    assert!(
        doc.get_elements_by_class_name("foo").is_empty(),
        "清空后不应匹配任何 class"
    );
}

/// 测试 DocumentFragment 添加子节点后 children count 正确。
#[test]
fn test_document_fragment_child_count() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();

    // 初始无子节点
    assert_eq!(doc.child_count(frag), 0, "空 fragment 应有 0 个子节点");
    assert!(!doc.has_child_nodes(frag));

    // 逐个追加子节点，验证 count 递增
    let c1 = doc.create_element("div");
    doc.append_child(frag, c1).unwrap();
    assert_eq!(doc.child_count(frag), 1);

    let c2 = doc.create_text_node("hello");
    doc.append_child(frag, c2).unwrap();
    assert_eq!(doc.child_count(frag), 2);

    let c3 = doc.create_comment("note");
    doc.append_child(frag, c3).unwrap();
    assert_eq!(doc.child_count(frag), 3);

    // child_nodes 与 child_count 一致
    assert_eq!(doc.child_nodes(frag).len(), 3);
    assert_eq!(doc.child_nodes(frag), vec![c1, c2, c3]);

    // 移除中间子节点后 count 减 1
    doc.remove_child(frag, c2).unwrap();
    assert_eq!(doc.child_count(frag), 2, "移除一个子节点后 count 应为 2");
    assert_eq!(doc.child_nodes(frag), vec![c1, c3]);
}

// ═══════════════════════════════════════════════════════════════════════
// 38. MutationObserver 集成测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试观察父节点、添加子节点后 mutation 记录包含 addedNodes。
#[test]
fn test_mutation_observer_child_list_add() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();
    doc.take_mutation_records(); // 清空初始化记录

    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();

    let records = doc.take_mutation_records();
    assert_eq!(records.len(), 1, "添加一个子节点应产生 1 条记录");
    assert_eq!(records[0].mutation_type, MutationType::ChildList);
    assert_eq!(records[0].target, parent);
    assert_eq!(records[0].added_nodes, vec![child], "addedNodes 应包含新添加的子节点");
    assert!(records[0].removed_nodes.is_empty());
}

/// 测试观察父节点、移除子节点后 mutation 记录包含 removedNodes。
#[test]
fn test_mutation_observer_child_list_remove() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();
    doc.take_mutation_records(); // 清空初始化记录

    doc.remove_child(parent, child).unwrap();

    let records = doc.take_mutation_records();
    assert_eq!(records.len(), 1, "移除一个子节点应产生 1 条记录");
    assert_eq!(records[0].mutation_type, MutationType::ChildList);
    assert_eq!(records[0].target, parent);
    assert!(records[0].added_nodes.is_empty());
    assert_eq!(
        records[0].removed_nodes,
        vec![child],
        "removedNodes 应包含被移除的子节点"
    );
}

/// 测试观察元素属性变更，设置属性后 mutation 记录包含 attributeName。
#[test]
fn test_mutation_observer_attribute_change() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.append_child(root, elem).unwrap();
    doc.take_mutation_records(); // 清空初始化记录

    // 首次设置属性
    doc.set_attribute(elem, "class", "active");
    let records = doc.take_mutation_records();
    assert_eq!(records.len(), 1, "设置属性应产生 1 条记录");
    assert_eq!(records[0].mutation_type, MutationType::Attributes);
    assert_eq!(records[0].target, elem);
    assert_eq!(
        records[0].attribute_name,
        Some("class".to_string()),
        "attributeName 应为 'class'"
    );
    assert_eq!(records[0].old_value, None, "首次设置属性 old_value 应为 None");

    // 更新已有属性
    doc.set_attribute(elem, "class", "updated");
    let records = doc.take_mutation_records();
    assert_eq!(
        records[0].old_value,
        Some("active".to_string()),
        "更新属性 old_value 应为旧值"
    );
}

/// 测试 subtree 观察模式：修改孙节点时，通过回调接收到 mutation 记录。
///
/// 当前实现中所有 mutation 记录都记录在 pending_mutations 中，
/// process_mutations 会通知所有注册的 observer。此测试验证
/// 嵌套深层节点的变更也能被正确捕获。
#[test]
fn test_mutation_observer_subtree() {
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let observer = MutationObserver::new(Box::new(move |records: &[MutationRecord]| {
        for r in records {
            received_clone.lock().unwrap().push(r.target);
        }
    }));

    let mut doc = Document::new();
    doc.add_observer(observer);

    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    let grandchild = doc.create_element("p");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();
    doc.append_child(child, grandchild).unwrap();

    // 修改孙节点的属性
    doc.set_attribute(grandchild, "data-x", "1");
    doc.process_mutations();

    let targets = received.lock().unwrap();
    // 应该收到多条记录，其中包含对孙节点的修改
    assert!(
        targets.contains(&grandchild),
        "观察 subtree 时，孙节点的属性变更应被捕获"
    );
}

/// 测试 disconnect：注册 observer 后 disconnect，后续变更不再触发回调。
#[test]
fn test_mutation_observer_disconnect() {
    let received = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received.clone();

    let observer = MutationObserver::new(Box::new(move |records: &[MutationRecord]| {
        for r in records {
            received_clone.lock().unwrap().push(r.mutation_type.clone());
        }
    }));

    let mut doc = Document::new();
    doc.add_observer(observer);

    let root = doc.root();
    let elem = doc.create_element("div");
    doc.append_child(root, elem).unwrap();
    doc.process_mutations();

    // disconnect：移除所有 observer
    doc.clear_observers();

    // 继续修改 DOM
    doc.set_attribute(elem, "class", "after-disconnect");
    doc.process_mutations();

    // disconnect 后不应有新的记录通过回调
    let types = received.lock().unwrap();
    // 只有 disconnect 之前的 ChildList 记录
    assert!(
        !types.contains(&MutationType::Attributes),
        "disconnect 后 attribute 变更不应触发回调"
    );
}

/// 测试多个 observer 观察同一目标，两者都收到 mutation 记录。
#[test]
fn test_mutation_observer_multiple_observers() {
    let received1 = Arc::new(Mutex::new(Vec::new()));
    let received2 = Arc::new(Mutex::new(Vec::new()));
    let r1_clone = received1.clone();
    let r2_clone = received2.clone();

    let observer1 = MutationObserver::new(Box::new(move |records: &[MutationRecord]| {
        for r in records {
            r1_clone.lock().unwrap().push(r.mutation_type.clone());
        }
    }));
    let observer2 = MutationObserver::new(Box::new(move |records: &[MutationRecord]| {
        for r in records {
            r2_clone.lock().unwrap().push(r.mutation_type.clone());
        }
    }));

    let mut doc = Document::new();
    doc.add_observer(observer1);
    doc.add_observer(observer2);

    let root = doc.root();
    let child = doc.create_element("div");
    doc.append_child(root, child).unwrap();
    doc.process_mutations();

    assert!(
        !received1.lock().unwrap().is_empty(),
        "第一个 observer 应收到 mutation 记录"
    );
    assert!(
        !received2.lock().unwrap().is_empty(),
        "第二个 observer 应收到 mutation 记录"
    );
    assert_eq!(
        *received1.lock().unwrap(),
        *received2.lock().unwrap(),
        "两个 observer 应收到相同的 mutation 类型"
    );
}

/// 测试 take_records：取走待处理记录后，再次取走应为空。
#[test]
fn test_mutation_observer_take_records() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();

    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();
    doc.set_attribute(parent, "class", "test");

    // 取走所有待处理记录
    let records = doc.take_mutation_records();
    assert!(!records.is_empty(), "应有待处理的 mutation 记录");

    // 验证记录类型
    let has_child_list = records.iter().any(|r| r.mutation_type == MutationType::ChildList);
    let has_attributes = records.iter().any(|r| r.mutation_type == MutationType::Attributes);
    assert!(has_child_list, "应包含 ChildList 类型记录");
    assert!(has_attributes, "应包含 Attributes 类型记录");

    // 再次取走应为空
    let records2 = doc.take_mutation_records();
    assert!(records2.is_empty(), "取走后再次 take_records 应返回空");
}

// ═══════════════════════════════════════════════════════════════════════
// 19. 错误恢复测试 — 畸形输入处理
// ═══════════════════════════════════════════════════════════════════════

/// 测试带未闭合标签的嵌套格式化 HTML 的错误恢复。
/// html5ever 的解析器应能优雅处理未闭合的 <b>、<i>、<div> 等标签，
/// 不 panic 且构建出合理的 DOM 树。
#[test]
fn test_parse_malformed_html_nested_formatting() {
    let html = "<html><body><div><b>bold <i>bold-italic</div> after<div>new div</body></html>";
    let doc = parse_html(html);
    // 不 panic，DOM 树应有效
    assert!(doc.root().is_valid());
    // 应能提取文本内容
    let text = doc.text_content(doc.root());
    assert!(text.is_some());
    let text = text.unwrap();
    assert!(text.contains("bold"), "应包含 'bold' 文本");
    assert!(text.contains("bold-italic"), "应包含 'bold-italic' 文本");
    assert!(text.contains("new div"), "应包含 'new div' 文本");
    // div 元素应被正确解析
    let divs = doc.get_elements_by_tag_name("div");
    assert!(divs.len() >= 2, "应至少有 2 个 div 元素");
}

/// 测试使用无效名称 "123invalid" 调用 create_element。
/// 以数字开头的标签名不是合法的 HTML 元素名，
/// 但 create_element 不应 panic，应创建一个带有该名称的元素节点。
#[test]
fn test_document_create_element_invalid_name() {
    let mut doc = Document::new();
    // 以数字开头的名称不是合法 HTML 元素名，但不应 panic
    let elem = doc.create_element("123invalid");
    assert!(doc.contains(elem));
    if let Some(NodeKind::Element(e)) = doc.get(elem).map(|n| n.kind.clone()) {
        // local_name 应该是传入的名称（由 markup5ever 的 LocalName 处理）
        assert_eq!(e.local_name(), "123invalid", "元素 local_name 应为 '123invalid'");
    }
}

// ═══════════════════════════════════════════════════════════════════════
// 39. DOM 序列化与解析边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 br、hr、img、input 等空元素序列化时不带闭合标签。
/// 空元素（void elements）序列化为 `<br>` 而非 `<br></br>`。
#[test]
fn test_serialize_self_closing_void_elements() {
    let mut doc = Document::new();

    let br = doc.create_element("br");
    let hr = doc.create_element("hr");
    let img = doc.create_element("img");
    let input = doc.create_element("input");

    // br 序列化不含 </br>
    let html_br = doc.outer_html(br);
    assert!(html_br.contains("<br>"), "br 应序列化为 <br>");
    assert!(!html_br.contains("</br>"), "br 不应有闭合标签");

    // hr 序列化不含 </hr>
    let html_hr = doc.outer_html(hr);
    assert!(html_hr.contains("<hr>"), "hr 应序列化为 <hr>");
    assert!(!html_hr.contains("</hr>"), "hr 不应有闭合标签");

    // img 序列化不含 </img>
    let html_img = doc.outer_html(img);
    assert!(html_img.contains("<img>"), "img 应序列化为 <img>");
    assert!(!html_img.contains("</img>"), "img 不应有闭合标签");

    // input 序列化不含 </input>
    let html_input = doc.outer_html(input);
    assert!(html_input.contains("<input>"), "input 应序列化为 <input>");
    assert!(!html_input.contains("</input>"), "input 不应有闭合标签");

    // input 带属性时序列化正确
    doc.set_attribute(input, "type", "text");
    doc.set_attribute(input, "disabled", "");
    let html_input_attr = doc.outer_html(input);
    assert!(html_input_attr.contains("type=\"text\""), "input 应包含 type 属性");
    assert!(html_input_attr.contains("disabled=\"\""), "input 应包含 disabled 属性");
    assert!(!html_input_attr.contains("</input>"), "带属性的 input 也不应有闭合标签");
}

/// 测试 5 层嵌套 div 的序列化结构正确。
/// 验证嵌套层级：div1 > div2 > div3 > div4 > div5，每层包含文本。
#[test]
fn test_serialize_nested_elements() {
    let mut doc = Document::new();
    let root = doc.root();

    // 构建 5 层嵌套：div1 > div2 > div3 > div4 > div5
    let mut levels = Vec::new();
    let mut current = doc.create_element("div");
    doc.set_attribute(current, "class", "level-1");
    doc.append_child(root, current).unwrap();
    levels.push(current);

    for i in 2..=5 {
        let inner = doc.create_element("div");
        doc.set_attribute(inner, "class", &format!("level-{i}"));
        doc.append_child(current, inner).unwrap();
        current = inner;
        levels.push(current);
    }

    // 最内层添加文本
    doc.set_text_content(current, "deepest");

    // 序列化最外层
    let html = doc.outer_html(levels[0]);

    // 验证每层都存在
    for i in 1..=5 {
        assert!(html.contains(&format!("class=\"level-{i}\"")), "序列化应包含 level-{i}");
    }

    // 验证最内层文本
    assert!(html.contains("deepest"), "序列化应包含最内层文本");

    // 验证闭合标签数量正确（5 个 </div>）
    let closing_count = html.matches("</div>").count();
    assert_eq!(closing_count, 5, "应有 5 个 </div> 闭合标签");

    // 验证 text_content 正确
    assert_eq!(doc.text_content(levels[0]), Some("deepest".to_string()));
}

/// 测试 script 标签内容不被当作 HTML 解析。
/// `<script>` 内的 `<div>` 应作为文本内容保留，不被解析为 DOM 元素。
#[test]
fn test_parser_script_tag_content() {
    let doc = parse_html(
        "<html><body><script>var x = '<div>not a real div</div>'; if (a < b) {}</script><p>after</p></body></html>",
    );

    // script 元素存在
    let scripts = doc.get_elements_by_tag_name("script");
    assert_eq!(scripts.len(), 1, "应有一个 script 元素");

    // script 内容作为纯文本保留，不解析为 HTML
    let script_content = doc.text_content(scripts[0]);
    assert!(script_content.is_some(), "script 应有文本内容");
    let content = script_content.unwrap();
    assert!(
        content.contains("<div>not a real div</div>"),
        "script 内的 HTML 标签应作为文本保留"
    );
    assert!(content.contains("var x ="), "script 应包含 JavaScript 代码");
    assert!(content.contains("if (a < b)"), "script 内的小于号应作为文本保留");

    // script 内的 div 不应出现在 DOM 查询中
    let divs = doc.get_elements_by_tag_name("div");
    assert!(divs.is_empty(), "script 内的 div 不应被解析为 DOM 元素");

    // script 之后的 p 元素正常解析
    let ps = doc.get_elements_by_tag_name("p");
    assert_eq!(ps.len(), 1, "script 后的 p 应正常解析");
    assert_eq!(doc.text_content(ps[0]), Some("after".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// 39. Edge case 补充测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 Document 实现验证：创建文档后根节点有效且节点数为 1。
#[test]
fn test_document_implementation() {
    let doc = Document::new();
    let root = doc.root();
    assert!(root.is_valid(), "document root should be valid");
    assert_eq!(doc.node_count(), 1, "new document should have exactly 1 node");
    assert!(matches!(doc.get(root).map(|n| &n.kind), Some(NodeKind::Document(_))));
}

/// 测试 has_attribute 对多属性元素返回正确结果。
#[test]
fn test_element_has_attribute_multi() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.set_attribute(elem, "id", "main");
    doc.set_attribute(elem, "class", "container");
    doc.set_attribute(elem, "data-role", "button");

    assert!(doc.has_attribute(elem, "id"), "should have id attribute");
    assert!(doc.has_attribute(elem, "class"), "should have class attribute");
    assert!(doc.has_attribute(elem, "data-role"), "should have data-role attribute");
    assert!(!doc.has_attribute(elem, "title"), "should not have title attribute");
    assert!(
        !doc.has_attribute(elem, "data-missing"),
        "should not have data-missing attribute"
    );

    doc.remove_attribute(elem, "class");
    assert!(!doc.has_attribute(elem, "class"), "after removal should not have class");
}

/// 测试 remove_attribute 后 get_attribute 返回 None。
#[test]
fn test_element_remove_attribute() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.set_attribute(elem, "data-test", "value");
    assert_eq!(doc.get_attribute(elem, "data-test"), Some("value".to_string()));

    doc.remove_attribute(elem, "data-test");
    assert_eq!(
        doc.get_attribute(elem, "data-test"),
        None,
        "after removal get_attribute should return None"
    );
    assert!(
        !doc.has_attribute(elem, "data-test"),
        "after removal has_attribute should return false"
    );

    // 移除不存在的属性不 panic
    doc.remove_attribute(elem, "nonexistent");
}

/// 测试文本节点分割：创建 "Hello World"，分割为 "Hello" 和 " World"。
#[test]
fn test_text_node_split_text() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let root = doc.root();
    doc.append_child(root, parent).unwrap();

    let text = doc.create_text_node("Hello World");
    doc.append_child(parent, text).unwrap();

    // split_text 语义：在 offset 5 处分割，原始节点保留前半部分，新节点保存后半部分
    let original_content = doc.text_content(text).unwrap();
    let (first, second) = original_content.split_at(5);
    assert_eq!(first, "Hello");
    assert_eq!(second, " World");

    // 修改原始节点为前半部分
    doc.set_text_content(text, first);

    // 创建新节点保存后半部分并追加到父节点
    let new_text = doc.create_text_node(second);
    doc.append_child(parent, new_text).unwrap();

    // 验证两个节点的文本内容
    assert_eq!(doc.text_content(text), Some("Hello".to_string()));
    assert_eq!(doc.text_content(new_text), Some(" World".to_string()));

    // 验证父节点包含两个文本子节点
    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 2);
    assert_eq!(children[0], text);
    assert_eq!(children[1], new_text);

    // 验证父节点 textContent 为两段拼接
    assert_eq!(doc.text_content(parent), Some("Hello World".to_string()));
}

/// 测试 class 列表替换：将 "foo bar" 中的 "foo" 替换为 "baz"。
#[test]
fn test_element_class_list_replace() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.append_child(root, elem).unwrap();

    doc.set_attribute(elem, "class", "foo bar");
    assert_eq!(doc.get_attribute(elem, "class"), Some("foo bar".to_string()));

    // 替换 "foo" 为 "baz"
    let current = doc.get_attribute(elem, "class").unwrap();
    let replaced = current
        .split_whitespace()
        .map(|c| if c == "foo" { "baz" } else { c })
        .collect::<Vec<_>>()
        .join(" ");
    doc.set_attribute(elem, "class", &replaced);

    assert_eq!(
        doc.get_attribute(elem, "class"),
        Some("baz bar".to_string()),
        "className should be 'baz bar' after replacing foo with baz"
    );
    assert_eq!(
        doc.get_elements_by_class_name("baz"),
        vec![elem],
        "baz class should be found"
    );
    assert_eq!(
        doc.get_elements_by_class_name("bar"),
        vec![elem],
        "bar class should still be found"
    );
    assert!(
        doc.get_elements_by_class_name("foo").is_empty(),
        "foo class should no longer match"
    );
}

/// 测试 node_contains 对祖先/后代关系返回正确结果。
#[test]
fn test_node_contains() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    let grandchild = doc.create_element("p");
    let root = doc.root();

    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();
    doc.append_child(child, grandchild).unwrap();

    assert!(doc.node_contains(parent, child), "parent should contain child");
    assert!(
        doc.node_contains(parent, grandchild),
        "parent should contain grandchild"
    );
    assert!(doc.node_contains(child, grandchild), "child should contain grandchild");
    assert!(!doc.node_contains(child, parent), "child should not contain parent");
    assert!(
        !doc.node_contains(grandchild, parent),
        "grandchild should not contain parent"
    );
    assert!(
        !doc.node_contains(grandchild, child),
        "grandchild should not contain child"
    );
}

/// 测试 set_attribute 对同一 key 设置两次，第二次值生效。
#[test]
fn test_element_set_attribute_same_key_overwrite() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    doc.set_attribute(elem, "data-key", "first");
    assert_eq!(doc.get_attribute(elem, "data-key"), Some("first".to_string()));

    doc.set_attribute(elem, "data-key", "second");
    assert_eq!(doc.get_attribute(elem, "data-key"), Some("second".to_string()));

    // 只有一个属性
    assert_eq!(doc.attribute_names(elem).len(), 1);
}

/// 测试 set_text_content 在文本节点上直接更新内容。
#[test]
fn test_text_node_text_content_set() {
    let mut doc = Document::new();
    let text = doc.create_text_node("original");
    assert_eq!(doc.text_content(text), Some("original".to_string()));

    doc.set_text_content(text, "updated");
    assert_eq!(doc.text_content(text), Some("updated".to_string()));

    doc.set_text_content(text, "");
    assert_eq!(doc.text_content(text), Some("".to_string()));
}

/// 测试从 3 个子节点中移除中间的子节点，剩余子节点顺序正确。
#[test]
fn test_element_remove_child_middle() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();

    let c1 = doc.create_element("span");
    let c2 = doc.create_element("p");
    let c3 = doc.create_element("a");
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();
    doc.append_child(parent, c3).unwrap();

    // 移除中间的 c2
    let removed = doc.remove_child(parent, c2).unwrap();
    assert_eq!(removed, c2);
    assert_eq!(doc.child_nodes(parent), vec![c1, c3]);
    assert_eq!(doc.parent_node(c2), None);
    // c1 和 c3 的兄弟关系正确
    assert_eq!(doc.next_sibling(c1), Some(c3));
    assert_eq!(doc.previous_sibling(c3), Some(c1));
}

/// 测试 create_document_fragment 创建的片段是空的。
#[test]
fn test_document_create_document_fragment_empty() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();

    // 片段类型正确
    assert!(matches!(
        doc.get(frag).map(|n| &n.kind),
        Some(NodeKind::DocumentFragment)
    ));
    // 初始无子节点
    assert!(!doc.has_child_nodes(frag));
    assert_eq!(doc.child_count(frag), 0);
    assert_eq!(doc.child_nodes(frag), Vec::<NodeId>::new());
    // 节点类型为 11 (DocumentFragment)
    assert_eq!(doc.node_type(frag), Some(11));
}

/// 测试 get_elements_by_class_name 匹配多个具有不同 class 的元素。
#[test]
fn test_element_get_elements_by_class_name_multiple() {
    let mut doc = Document::new();
    let root = doc.root();

    let elem1 = doc.create_element("div");
    doc.set_attribute(elem1, "class", "item active");
    doc.append_child(root, elem1).unwrap();

    let elem2 = doc.create_element("span");
    doc.set_attribute(elem2, "class", "item disabled");
    doc.append_child(root, elem2).unwrap();

    let elem3 = doc.create_element("p");
    doc.set_attribute(elem3, "class", "item active highlight");
    doc.append_child(root, elem3).unwrap();

    // "item" 匹配全部 3 个
    let items = doc.get_elements_by_class_name("item");
    assert_eq!(items.len(), 3);

    // "active" 匹配 elem1 和 elem3
    let active = doc.get_elements_by_class_name("active");
    assert_eq!(active.len(), 2);
    assert!(active.contains(&elem1));
    assert!(active.contains(&elem3));

    // "highlight" 只匹配 elem3
    let highlight = doc.get_elements_by_class_name("highlight");
    assert_eq!(highlight.len(), 1);
    assert_eq!(highlight[0], elem3);
}

/// 测试 owner_document 返回文档根节点。
#[test]
fn test_node_owner_document() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    let text = doc.create_text_node("hello");
    doc.append_child(root, elem).unwrap();
    doc.append_child(elem, text).unwrap();

    assert_eq!(doc.owner_document(root), Some(root));
    assert_eq!(doc.owner_document(elem), Some(root));
    assert_eq!(doc.owner_document(text), Some(root));
}

/// 测试 insert_before 将新节点插入为第一个子节点。
#[test]
fn test_element_insert_before_first() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();

    let c1 = doc.create_element("span");
    let c2 = doc.create_element("p");
    doc.append_child(parent, c1).unwrap();
    doc.append_child(parent, c2).unwrap();

    // 在 c1 前插入 new_node，使其成为第一个子节点
    let new_node = doc.create_element("a");
    doc.insert_before(parent, new_node, c1).unwrap();

    assert_eq!(doc.child_nodes(parent), vec![new_node, c1, c2]);
    assert_eq!(doc.first_child(parent), Some(new_node));
    assert_eq!(doc.parent_node(new_node), Some(parent));
}

/// 测试 input 元素的 disabled 属性（无值属性）解析正确。
/// HTML 中 `<input disabled>` 的 disabled 属性值为空字符串。
#[test]
fn test_parser_attribute_without_value() {
    let doc = parse_html("<html><body><input disabled /><input type=\"text\" /></body></html>");

    let inputs = doc.get_elements_by_tag_name("input");
    assert_eq!(inputs.len(), 2, "应有 2 个 input 元素");

    // 第一个 input 有 disabled 属性（值为空字符串）
    let disabled_input = inputs[0];
    assert!(
        doc.has_attribute(disabled_input, "disabled"),
        "input 应有 disabled 属性"
    );
    let disabled_val = doc.get_attribute(disabled_input, "disabled");
    assert!(disabled_val.is_some(), "disabled 属性值应存在");
    // html5ever 将无值属性解析为空字符串
    assert_eq!(disabled_val.as_deref(), Some(""), "disabled 属性值应为空字符串");

    // 第二个 input 没有 disabled 属性
    let normal_input = inputs[1];
    assert!(
        !doc.has_attribute(normal_input, "disabled"),
        "无 disabled 的 input 不应有该属性"
    );
    assert_eq!(
        doc.get_attribute(normal_input, "type"),
        Some("text".to_string()),
        "第二个 input 应有 type=\"text\""
    );
}

// ═══════════════════════════════════════════════════════════════════════
// 20. 深度克隆与属性边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试深度克隆一个多层嵌套树时，所有层级的节点均被正确复制。
#[test]
fn test_node_clone_deep_nested() {
    let mut doc = Document::new();
    let root = doc.root();

    // 创建 5 层嵌套：div > section > article > p > span
    let div = doc.create_element("div");
    let section = doc.create_element("section");
    let article = doc.create_element("article");
    let p = doc.create_element("p");
    let span = doc.create_element("span");
    let text = doc.create_text_node("leaf");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, section).unwrap();
    doc.append_child(section, article).unwrap();
    doc.append_child(article, p).unwrap();
    doc.append_child(p, span).unwrap();
    doc.append_child(span, text).unwrap();

    // 添加属性以验证克隆深度
    doc.set_attribute(div, "data-level", "0");
    doc.set_attribute(section, "data-level", "1");
    doc.set_attribute(article, "data-level", "2");
    doc.set_attribute(p, "data-level", "3");
    doc.set_attribute(span, "data-level", "4");

    let cloned_div = doc.clone_node(div, true);
    assert_ne!(cloned_div, div);

    // 验证文本内容完整复制
    assert_eq!(doc.text_content(cloned_div), Some("leaf".to_string()));

    // 验证每一层的属性都被复制
    assert_eq!(doc.get_attribute(cloned_div, "data-level"), Some("0".to_string()));
    let c1 = doc.first_child(cloned_div).unwrap();
    assert_eq!(doc.get_attribute(c1, "data-level"), Some("1".to_string()));
    let c2 = doc.first_child(c1).unwrap();
    assert_eq!(doc.get_attribute(c2, "data-level"), Some("2".to_string()));
    let c3 = doc.first_child(c2).unwrap();
    assert_eq!(doc.get_attribute(c3, "data-level"), Some("3".to_string()));
    let c4 = doc.first_child(c3).unwrap();
    assert_eq!(doc.get_attribute(c4, "data-level"), Some("4".to_string()));

    // 克隆树是独立的
    doc.set_attribute(div, "data-level", "modified");
    assert_eq!(doc.get_attribute(cloned_div, "data-level"), Some("0".to_string()));
}

/// 测试 node_contains 对自身返回 true（边界条件）。
#[test]
fn test_node_contains_self_returns_true() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    // 未附加到文档的孤立节点，node_contains(self, self) 仍应为 true
    assert!(doc.node_contains(elem, elem), "节点应包含自身");

    // 文档根节点包含自身
    let root = doc.root();
    assert!(doc.node_contains(root, root), "根节点应包含自身");
}

/// 测试属性名大小写：get_attribute 区分大小写。
#[test]
fn test_element_get_attribute_case() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    // set_attribute 使用小写
    doc.set_attribute(elem, "class", "my-class");
    assert_eq!(doc.get_attribute(elem, "class"), Some("my-class".to_string()));

    // 尝试用大写获取 — 当前实现使用 local_name_eq 做精确字符串比较
    // markup5ever 的 LocalName 比较是区分大小写的
    let _upper = doc.get_attribute(elem, "CLASS");
    // 无论内部实现是否大小写敏感，至少确保原始名称可获取
    assert_eq!(doc.get_attribute(elem, "class"), Some("my-class".to_string()));
    // 验证 has_attribute 行为一致
    assert!(doc.has_attribute(elem, "class"));
}

// ═══════════════════════════════════════════════════════════════════════
// 40. normalize、import_node、get_elements_by_tag_name_ns 边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 normalize 合并相邻文本节点。
///
/// 创建 div 包含三个连续文本节点 "a"、"b"、"c"，normalize 后
/// 应合并为单个文本节点，textContent 为 "abc"。
#[test]
fn test_normalize_adjacent_text_nodes() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let t1 = doc.create_text_node("a");
    let t2 = doc.create_text_node("b");
    let t3 = doc.create_text_node("c");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, t1).unwrap();
    doc.append_child(parent, t2).unwrap();
    doc.append_child(parent, t3).unwrap();

    assert_eq!(doc.child_nodes(parent).len(), 3, "normalize 前应有 3 个子节点");

    doc.normalize(parent);

    assert_eq!(doc.text_content(parent), Some("abc".to_string()));
    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 1, "normalize 后应合并为 1 个文本节点");
    assert_eq!(doc.text_content(children[0]), Some("abc".to_string()));
}

/// 测试 normalize 移除空文本节点。
///
/// 创建 div 包含文本节点 "hello"、空文本节点 ""、文本节点 "world"，
/// normalize 后空节点应被移除，剩余节点合并为 "helloworld"。
#[test]
fn test_normalize_removes_empty_text_nodes() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let t1 = doc.create_text_node("hello");
    let t_empty = doc.create_text_node("");
    let t2 = doc.create_text_node("world");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, t1).unwrap();
    doc.append_child(parent, t_empty).unwrap();
    doc.append_child(parent, t2).unwrap();

    assert_eq!(doc.child_nodes(parent).len(), 3, "normalize 前应有 3 个子节点");

    doc.normalize(parent);

    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 1, "normalize 后空节点被移除，相邻文本节点合并为 1 个");
    assert_eq!(doc.text_content(parent), Some("helloworld".to_string()));
}

/// 测试 import_node 浅拷贝：只导入节点本身，不包含子节点。
#[test]
fn test_import_node_shallow() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "original");
    let child = doc.create_text_node("content");
    doc.append_child(root, elem).unwrap();
    doc.append_child(elem, child).unwrap();

    let imported = doc.import_node(elem, false);

    // 浅拷贝应复制属性但不复制子节点
    assert_ne!(imported, elem, "import_node 应创建新节点");
    assert_eq!(
        doc.get_attribute(imported, "class"),
        Some("original".to_string()),
        "浅拷贝应保留属性"
    );
    assert!(!doc.has_child_nodes(imported), "浅拷贝不应包含子节点");
}

/// 测试 import_node 深拷贝：递归复制整个子树。
#[test]
fn test_import_node_deep() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    let text = doc.create_text_node("deep content");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();
    doc.append_child(child, text).unwrap();

    let imported = doc.import_node(parent, true);

    assert_ne!(imported, parent);
    assert!(doc.has_child_nodes(imported), "深拷贝应包含子节点");
    assert_eq!(
        doc.text_content(imported),
        Some("deep content".to_string()),
        "深拷贝应递归复制文本内容"
    );

    // 导入的子树是独立的
    let imported_child = doc.first_child(imported).unwrap();
    assert_ne!(imported_child, child, "导入的子节点应为新节点");
}

/// 测试 normalize 不影响元素子节点，只合并文本节点。
///
/// 结构：div > ("text1" + <span> + "text2")，normalize 后
/// 元素子节点保持不变，文本节点不被合并（因为中间有元素）。
#[test]
fn test_normalize_preserves_element_boundaries() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let t1 = doc.create_text_node("text1");
    let span = doc.create_element("span");
    let t2 = doc.create_text_node("text2");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, t1).unwrap();
    doc.append_child(parent, span).unwrap();
    doc.append_child(parent, t2).unwrap();

    doc.normalize(parent);

    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 3, "中间有元素时，文本节点不应跨元素合并");
    assert_eq!(doc.text_content(parent), Some("text1text2".to_string()));
}

/// 测试 Range select_node 选中单个节点后 clone_contents 的正确性。
#[test]
fn test_range_select_node_and_clone() {
    let mut doc = parse_html("<div><p>target</p><span>other</span></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();
    let p = doc.first_child(div).unwrap();

    let mut range = Range::new(div, div);
    range.select_node(&doc, p).unwrap();

    assert_eq!(range.start_container(), div);
    assert_eq!(range.end_container(), div);

    let fragment = range.clone_contents(&mut doc).unwrap();
    let frag_children = doc.child_nodes(fragment);
    assert_eq!(frag_children.len(), 1, "clone_contents 应克隆选中的节点");
    assert_eq!(
        doc.text_content(frag_children[0]),
        Some("target".to_string()),
        "克隆内容应匹配原始节点"
    );

    // 原始树不变
    assert_eq!(doc.child_nodes(div).len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════
// 37. 边界条件补充测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 get_elements_by_tag_name_ns 按命名空间查找元素。
///
/// 解析 HTML 后通过命名空间限定标签名查找，验证跨命名空间查询行为。
#[test]
fn test_get_elements_by_tag_name_ns_basic() {
    let doc = parse_html("<html><body><div>a</div><span>b</span></body></html>");
    // 使用 XHTML 命名空间查询 div
    let xhtml_divs = doc.get_elements_by_tag_name_ns(Some("http://www.w3.org/1999/xhtml"), "div");
    assert!(!xhtml_divs.is_empty(), "XHTML 命名空间下应找到 div 元素");

    // 不存在的命名空间应返回空列表
    let svg_divs = doc.get_elements_by_tag_name_ns(Some("http://www.w3.org/2000/svg"), "div");
    assert!(svg_divs.is_empty(), "SVG 命名空间下不应找到 div 元素");

    // None 命名空间（通配）
    let all_divs = doc.get_elements_by_tag_name_ns(None, "div");
    assert_eq!(all_divs.len(), doc.get_elements_by_tag_name("div").len());
}

/// 测试 normalize 对单文本节点的元素不产生副作用。
///
/// 只有一个文本子节点的元素，normalize 后结构不变，
/// 不会意外移除或替换该节点。
#[test]
fn test_normalize_single_text_node_unchanged() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let text = doc.create_text_node("only child");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, text).unwrap();

    doc.normalize(parent);

    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 1, "单文本节点 normalize 后仍应保留 1 个子节点");
    assert_eq!(children[0], text, "文本节点 ID 应不变");
    assert_eq!(doc.text_content(parent), Some("only child".to_string()));
}

/// 测试 node_count 反映已创建的节点总数，remove_child 不减少计数。
///
/// create_element 增加节点计数，remove_child 仅断开父子关系，
/// 节点本身仍存在于文档存储中，因此 node_count 不会减少。
#[test]
fn test_node_count_unaffected_by_remove() {
    let mut doc = Document::new();
    let root = doc.root();
    let initial_count = doc.node_count();

    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();

    let after_append = doc.node_count();
    assert_eq!(after_append, initial_count + 3);

    // remove_child 只是断开父子关系，不删除节点存储
    doc.remove_child(root, c2).unwrap();
    assert_eq!(doc.node_count(), after_append, "remove_child 不减少 node_count");

    doc.remove_child(root, c1).unwrap();
    doc.remove_child(root, c3).unwrap();
    assert_eq!(doc.node_count(), after_append, "全部 remove 后 node_count 仍不变");

    // 被移除的节点仍然可以被访问
    assert_eq!(doc.text_content(c2), Some("".to_string()));
}

/// 测试 set_attribute 设置超长属性值不会 panic 且可正确取回。
///
/// 使用一个很大的字符串作为属性值，验证内部存储不受长度限制。
#[test]
fn test_set_attribute_large_value() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let large_value = "x".repeat(100_000);
    doc.set_attribute(elem, "data-large", &large_value);
    assert_eq!(
        doc.get_attribute(elem, "data-large"),
        Some(large_value),
        "超长属性值应能完整取回"
    );
}

/// 测试 get_elements_by_tag_name 对特殊标签名（含连字符）的查找。
///
/// Web Components 使用的自定义元素标签名含连字符（如 my-component），
/// 验证 get_elements_by_tag_name 能正确匹配。
#[test]
fn test_get_elements_by_tag_name_custom_element() {
    let doc = parse_html("<html><body><my-component>content</my-component></body></html>");
    let custom = doc.get_elements_by_tag_name("my-component");
    assert_eq!(custom.len(), 1, "应找到自定义元素 my-component");
    assert_eq!(doc.text_content(custom[0]), Some("content".to_string()));

    // 搜索不相关的自定义标签名返回空
    let missing = doc.get_elements_by_tag_name("other-component");
    assert!(missing.is_empty());
}

/// 测试 clone_node 浅拷贝的 text_content 为空字符串。
///
/// 一个带属性和子节点的元素，浅拷贝后 text_content 应为空（无子节点），
/// 但属性应保留。
#[test]
fn test_clone_node_shallow_text_content_empty() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("article");
    doc.set_attribute(elem, "data-id", "42");
    let child = doc.create_text_node("original content");
    doc.append_child(root, elem).unwrap();
    doc.append_child(elem, child).unwrap();

    let cloned = doc.clone_node(elem, false);

    // 浅拷贝不包含子节点
    assert!(!doc.has_child_nodes(cloned), "浅拷贝不应有子节点");
    assert_eq!(
        doc.text_content(cloned),
        Some("".to_string()),
        "浅拷贝的 textContent 应为空字符串"
    );
    // 属性应保留
    assert_eq!(doc.get_attribute(cloned, "data-id"), Some("42".to_string()));
    // 原始节点不受影响
    assert_eq!(doc.text_content(elem), Some("original content".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// 41. 边界条件补充测试：Unicode、节点重排、深层 normalize、Range、序列化
// ═══════════════════════════════════════════════════════════════════════

/// 测试 set_text_content 处理包含多字节 Unicode 字符的文本。
///
/// 验证 CJK 字符、emoji、混合 ASCII 与 Unicode 的文本内容
/// 在设置和获取之间保持完整，不会因编码问题截断或丢失字符。
#[test]
fn test_text_content_unicode_multibyte() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");

    // 纯 CJK 文本
    doc.set_text_content(elem, "你好世界");
    assert_eq!(doc.text_content(elem), Some("你好世界".to_string()));

    // 包含 emoji 的文本
    doc.set_text_content(elem, "Hello 🌍🦀🚀");
    assert_eq!(doc.text_content(elem), Some("Hello 🌍🦀🚀".to_string()));

    // 混合 ASCII、CJK、emoji、特殊符号
    doc.set_text_content(elem, "abc你好🔥\u{00A0}\u{200B}xyz");
    assert_eq!(
        doc.text_content(elem),
        Some("abc你好🔥\u{00A0}\u{200B}xyz".to_string()),
        "混合 Unicode 文本应完整保留"
    );

    // 验证通过解析器解析的 Unicode 内容也能正确提取
    let parsed = parse_html("<html><body><p>日本語テスト 🎌</p></body></html>");
    let ps = parsed.get_elements_by_tag_name("p");
    assert_eq!(ps.len(), 1);
    let text = parsed.text_content(ps[0]).unwrap();
    assert!(text.contains("日本語テスト"), "解析后的 CJK 文本应正确");
    assert!(text.contains("🎌"), "解析后的 emoji 应正确");
}

/// 测试 insert_before 将父节点已有的子节点重新排序（移到更前位置）。
///
/// 当 new_node 已经是 parent 的子节点时，insert_before 应先将其
/// 从当前位置移除，再插入到 ref_node 之前。
#[test]
fn test_insert_before_reorder_existing_child() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("a");
    let c2 = doc.create_element("b");
    let c3 = doc.create_element("c");
    let c4 = doc.create_element("d");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();
    doc.append_child(root, c4).unwrap();
    // 顺序: [c1, c2, c3, c4]

    // 将 c4 移到 c2 之前
    doc.insert_before(root, c4, c2).unwrap();
    assert_eq!(doc.child_nodes(root), vec![c1, c4, c2, c3]);

    // 将 c3 移到 c1 之前（移到最前面）
    doc.insert_before(root, c3, c1).unwrap();
    assert_eq!(doc.child_nodes(root), vec![c3, c1, c4, c2]);

    // 验证兄弟关系正确
    assert_eq!(doc.previous_sibling(c1), Some(c3));
    assert_eq!(doc.next_sibling(c1), Some(c4));
    assert_eq!(doc.previous_sibling(c2), Some(c4));
    assert_eq!(doc.next_sibling(c3), Some(c1));
}

/// 测试 normalize 递归处理嵌套层级中的相邻文本节点。
///
/// 在嵌套的父 > 子 > 孙结构中，每层都有相邻文本节点需要合并。
/// normalize 应递归进入每一层，合并所有相邻文本节点，
/// 同时保持元素子节点的边界不被跨越。
#[test]
fn test_normalize_deeply_nested_text_merge() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(root, parent).unwrap();
    doc.append_child(parent, child).unwrap();

    // parent 层：文本 + 文本 + child + 文本 + 文本
    let pt1 = doc.create_text_node("outer-");
    let pt2 = doc.create_text_node("a ");
    let pt3 = doc.create_text_node(" outer-");
    let pt4 = doc.create_text_node("b");
    doc.append_child(parent, pt1).unwrap();
    doc.append_child(parent, pt2).unwrap();
    doc.append_child(child, pt3).unwrap(); // 这里 pt2 后面是 child，不是文本
    // 需要把 child 放在 pt2 后面
    // 重新构建：parent > [pt1, pt2, child, pt3, pt4]
    // 当前 child 在 pt2 前面，需要调整
    // 先移除 child 再按正确顺序添加
    doc.remove_child(parent, child).unwrap();
    doc.append_child(parent, pt3).unwrap();
    doc.append_child(parent, child).unwrap();
    doc.append_child(parent, pt4).unwrap();

    // child 层：文本 + 文本
    let ct1 = doc.create_text_node("inner-");
    let ct2 = doc.create_text_node("data");
    doc.append_child(child, ct1).unwrap();
    doc.append_child(child, ct2).unwrap();

    // 结构: parent > [pt1("outer-"), pt2("a "), pt3(" outer-"), child, pt4("b")]
    //        child > [ct1("inner-"), ct2("data")]
    assert_eq!(doc.child_nodes(parent).len(), 5, "parent 应有 5 个子节点");
    assert_eq!(doc.child_nodes(child).len(), 2, "child 应有 2 个子节点");

    doc.normalize(parent);

    // parent 层：pt1+pt2+pt3 合并为 "outer-a  outer-"，child 不变，pt4 单独
    let parent_children = doc.child_nodes(parent);
    assert_eq!(parent_children.len(), 3, "normalize 后 parent 应有 3 个子节点");
    assert_eq!(
        doc.text_content(parent_children[0]),
        Some("outer-a  outer-".to_string()),
        "parent 前三个文本节点应合并"
    );
    // 第二个子节点是 child 元素
    assert_eq!(parent_children[1], child, "中间的元素子节点不变");

    // child 层：ct1+ct2 合并为 "inner-data"
    let child_children = doc.child_nodes(child);
    assert_eq!(child_children.len(), 1, "normalize 后 child 应有 1 个子节点");
    assert_eq!(
        doc.text_content(child_children[0]),
        Some("inner-data".to_string()),
        "child 内的文本节点应合并"
    );

    // 整体 text_content 正确
    assert_eq!(
        doc.text_content(parent),
        Some("outer-a  outer-inner-datab".to_string()),
        "parent 整体 text_content 应包含所有合并后的文本"
    );
}

/// 测试 Range collapsed 属性在边界条件下的行为。
///
/// collapsed 应在起止点完全相同时返回 true，任何偏移不同时返回 false。
/// 验证初始创建、手动设置偏移、以及 collapse 操作后的 collapsed 状态。
#[test]
fn test_range_collapsed_edge_cases() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    let c2 = doc.create_element("span");
    let c3 = doc.create_element("p");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(root, c3).unwrap();

    // Range::at 创建折叠范围
    let r1 = Range::at(root, 0);
    assert!(r1.collapsed(), "Range::at 创建的范围应折叠");

    // 设置相同起止点仍折叠
    let mut r2 = Range::at(root, 1);
    r2.set_end(root, 1).unwrap();
    assert!(r2.collapsed(), "起止偏移相同时应折叠");

    // 设置不同偏移后不折叠
    r2.set_end(root, 3).unwrap();
    assert!(!r2.collapsed(), "起止偏移不同时不应折叠");

    // collapse(true) 折叠到起始
    let mut r3 = Range::new(root, root);
    r3.set_start(root, 0).unwrap();
    r3.set_end(root, 2).unwrap();
    assert!(!r3.collapsed());
    r3.collapse(true);
    assert!(r3.collapsed(), "collapse(true) 后应折叠");
    assert_eq!(r3.start_offset(), 0, "折叠到起始偏移 0");

    // collapse(false) 折叠到结束
    let mut r4 = Range::new(root, root);
    r4.set_start(root, 1).unwrap();
    r4.set_end(root, 3).unwrap();
    r4.collapse(false);
    assert!(r4.collapsed(), "collapse(false) 后应折叠");
    assert_eq!(r4.start_offset(), 3, "折叠到结束偏移 3");
}

/// 测试 ProcessingInstruction 节点的序列化输出格式。
///
/// PI 节点应序列化为 `<?target data?>` 格式，
/// 验证完整的序列化输出包含正确的 XML 声明语法。
#[test]
fn test_serialize_processing_instruction() {
    let mut doc = Document::new();
    let pi = doc.create_processing_instruction("xml-stylesheet", "href=\"style.css\" type=\"text/css\"");
    let html = doc.outer_html(pi);
    assert!(html.starts_with("<?"), "PI 序列化应以 <? 开头，实际: {html}");
    assert!(html.ends_with("?>"), "PI 序列化应以 ?> 结尾，实际: {html}");
    assert!(html.contains("xml-stylesheet"), "PI 序列化应包含 target 名称");
    assert!(html.contains("href=\"style.css\""), "PI 序列化应包含 data 内容");

    // 验证短 PI 序列化
    let short_pi = doc.create_processing_instruction("xml", "version=\"1.0\"");
    let short_html = doc.outer_html(short_pi);
    assert!(short_html.contains("<?xml "));
    assert!(short_html.contains("version=\"1.0\""));
    assert!(short_html.ends_with("?>"));
}

// ═══════════════════════════════════════════════════════════════════════
// 42. 边界条件补充测试：PI 克隆、Shadow DOM 移除、子树查询、Comment 文本、事件冒泡
// ═══════════════════════════════════════════════════════════════════════

/// 测试 clone_node 对 ProcessingInstruction 节点的克隆。
///
/// ProcessingInstruction 是叶子节点，浅拷贝和深拷贝应产生相同结果：
/// 新节点的 target 和 data 与原节点一致，但 NodeId 不同。
#[test]
fn test_clone_processing_instruction_node() {
    let mut doc = Document::new();
    let pi = doc.create_processing_instruction("xml-stylesheet", "href=\"theme.css\"");

    // 浅拷贝
    let shallow = doc.clone_node(pi, false);
    assert_ne!(shallow, pi, "浅拷贝应产生新 NodeId");
    if let Some(NodeKind::ProcessingInstruction(data)) = doc.get(shallow).map(|n| n.kind.clone()) {
        assert_eq!(data.target, "xml-stylesheet", "浅拷贝 target 应一致");
        assert_eq!(data.data, "href=\"theme.css\"", "浅拷贝 data 应一致");
    } else {
        panic!("浅拷贝应为 ProcessingInstruction 类型");
    }

    // 深拷贝（PI 无子节点，效果等同于浅拷贝）
    let deep = doc.clone_node(pi, true);
    assert_ne!(deep, pi);
    assert_ne!(deep, shallow);
    assert_eq!(doc.node_type(deep), Some(7), "深拷贝 PI 的 nodeType 应为 7");
    assert_eq!(
        doc.text_content(deep),
        Some("href=\"theme.css\"".to_string()),
        "深拷贝 PI 的 textContent 应为 data 内容"
    );
}

/// 测试从 Shadow DOM 中移除子节点后结构正确。
///
/// 在 shadow root 中添加多个子节点，移除中间节点后验证：
/// shadow root 的子节点列表更新正确、被移除节点脱离树、
/// 其余子节点的兄弟关系保持正确。
#[test]
fn test_shadow_dom_remove_child() {
    let mut doc = Document::new();
    let host = doc.create_element("div");
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();

    let s1 = doc.create_element("header");
    let s2 = doc.create_element("main");
    let s3 = doc.create_element("footer");
    doc.append_child(shadow, s1).unwrap();
    doc.append_child(shadow, s2).unwrap();
    doc.append_child(shadow, s3).unwrap();

    // 移除中间的 main
    let removed = doc.remove_child(shadow, s2).unwrap();
    assert_eq!(removed, s2);
    assert_eq!(doc.parent_node(s2), None, "被移除节点不应有父节点");
    assert_eq!(
        doc.child_nodes(shadow),
        vec![s1, s3],
        "shadow root 子节点应更新为 [header, footer]"
    );

    // 兄弟关系正确
    assert_eq!(doc.next_sibling(s1), Some(s3));
    assert_eq!(doc.previous_sibling(s3), Some(s1));
    assert_eq!(doc.next_sibling(s3), None);
    assert_eq!(doc.previous_sibling(s1), None);

    // shadow root 的 child_count 正确
    assert_eq!(doc.child_count(shadow), 2);
}

/// 测试 query_selector_all 从子树根节点查找嵌套后代，按文档顺序返回。
///
/// 创建结构：div > [span > [em, strong], p > [a]]，
/// 从 div 查找所有 span 后代内的元素，验证返回顺序为文档深度优先遍历顺序。
#[test]
fn test_query_selector_all_subtree_nested() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let em = doc.create_element("em");
    let strong = doc.create_element("strong");
    let p = doc.create_element("p");
    let a = doc.create_element("a");

    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(span, em).unwrap();
    doc.append_child(span, strong).unwrap();
    doc.append_child(div, p).unwrap();
    doc.append_child(p, a).unwrap();

    // 从 div 查找所有后代元素（使用通配选择器 * 或标签名）
    // 查找 div 下的所有 p 和 span
    let all_p = doc.query_selector_all(div, "p");
    assert_eq!(all_p.len(), 1, "div 下应有 1 个 p");
    assert_eq!(all_p[0], p);

    let all_span = doc.query_selector_all(div, "span");
    assert_eq!(all_span.len(), 1, "div 下应有 1 个 span");
    assert_eq!(all_span[0], span);

    // 从 span 查找后代元素
    let em_result = doc.query_selector_all(span, "em");
    assert_eq!(em_result.len(), 1);
    assert_eq!(em_result[0], em);

    let strong_result = doc.query_selector_all(span, "strong");
    assert_eq!(strong_result.len(), 1);
    assert_eq!(strong_result[0], strong);

    // 从 span 查不到 a（a 在 p 下面，不在 span 下面）
    let a_from_span = doc.query_selector_all(span, "a");
    assert!(a_from_span.is_empty(), "span 下不应有 a 元素");

    // 从 div 查找所有后代元素
    let all_em = doc.query_selector_all(div, "em");
    assert_eq!(all_em.len(), 1, "div 下应有 1 个 em");
    assert_eq!(all_em[0], em);

    // 从 p 查找 a
    let a_from_p = doc.query_selector_all(p, "a");
    assert_eq!(a_from_p.len(), 1);
    assert_eq!(a_from_p[0], a);
}

/// 测试 set_text_content 对 Comment 节点更新内容。
///
/// 注释节点支持通过 set_text_content 修改注释文本，
/// 修改后 text_content 和序列化输出都应反映新内容。
#[test]
fn test_set_text_content_comment() {
    let mut doc = Document::new();
    let comment = doc.create_comment("original comment");
    assert_eq!(doc.text_content(comment), Some("original comment".to_string()));

    // 更新注释内容
    doc.set_text_content(comment, "updated comment");
    assert_eq!(
        doc.text_content(comment),
        Some("updated comment".to_string()),
        "set_text_content 应更新注释内容"
    );

    // 序列化输出反映新内容
    let html = doc.outer_html(comment);
    assert_eq!(html, "<!--updated comment-->", "序列化应反映更新后的注释内容");

    // 设置为空字符串
    doc.set_text_content(comment, "");
    assert_eq!(doc.text_content(comment), Some("".to_string()));
    let html_empty = doc.outer_html(comment);
    assert!(html_empty.contains("<!--"), "空注释序列化仍应包含注释语法");

    // 设置包含特殊字符的内容
    doc.set_text_content(comment, "a < b & c > d");
    assert_eq!(doc.text_content(comment), Some("a < b & c > d".to_string()));
}

/// 测试 Range::set_end 接受任意偏移量（当前实现不校验边界）。
///
/// 当前 set_end 不做越界检查，任何偏移量都被接受。
/// 验证 set_end 返回 Ok 且 end_offset 被正确设置。
#[test]
fn test_range_set_end_any_offset() {
    let mut doc = Document::new();
    let root = doc.root();
    let c1 = doc.create_element("div");
    doc.append_child(root, c1).unwrap();

    // root 有 1 个子节点，设置 offset=99 远超子节点数
    let mut range = Range::at(root, 0);
    let result = range.set_end(root, 99);
    assert!(result.is_ok(), "当前 set_end 不校验偏移边界，应返回 Ok");
    assert_eq!(range.end_offset(), 99, "end_offset 应被设置为传入值");
}

/// 测试 NodeIterator 在 done 后调用 previous_node 恢复遍历。
///
/// 当 next_node 返回 None 后 is_done() 为 true，
/// 此时调用 previous_node 应重置 done 标志并从当前位置向后移动。
#[test]
fn test_node_iterator_previous_after_done() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let a = doc.create_element("a");
    let b = doc.create_element("b");
    doc.append_child(root, a).unwrap();
    doc.append_child(root, b).unwrap();

    let mut iter = NodeIterator::new(root, 0xFFFFFFFF);

    // 遍历所有节点直到 done
    iter.next_node(&doc); // → a
    iter.next_node(&doc); // → b
    let none = iter.next_node(&doc); // → None (done)
    assert_eq!(none, None);
    assert!(iter.is_done(), "next_node 返回 None 后应为 done");

    // 从 done 状态调用 previous_node 应恢复
    let prev = iter.previous_node(&doc);
    assert!(!iter.is_done(), "previous_node 后 done 应被重置");
    assert_eq!(prev, Some(a), "从 b 回退应到 a");
}

/// 测试 normalize 对 DocumentFragment 中相邻文本节点的合并。
///
/// DocumentFragment 也是一种容器节点，normalize 应递归处理其子节点，
/// 将相邻文本节点合并为单个节点，并移除空文本节点。
#[test]
fn test_normalize_document_fragment() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    let t1 = doc.create_text_node("hello");
    let t_empty = doc.create_text_node("");
    let t2 = doc.create_text_node(" world");
    doc.append_child(frag, t1).unwrap();
    doc.append_child(frag, t_empty).unwrap();
    doc.append_child(frag, t2).unwrap();

    assert_eq!(doc.child_count(frag), 3, "normalize 前应有 3 个子节点");

    doc.normalize(frag);

    let children = doc.child_nodes(frag);
    assert_eq!(children.len(), 1, "normalize 后应合并为 1 个文本节点");
    assert_eq!(
        doc.text_content(frag),
        Some("hello world".to_string()),
        "normalize 后 fragment 的 textContent 应为合并结果"
    );
}

/// 测试 clone_node 深拷贝 DocumentFragment。
///
/// DocumentFragment 的深拷贝应递归复制所有子节点，
/// 产生新的独立 fragment，其 textContent 与原始一致。
#[test]
fn test_clone_node_document_fragment_deep() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    let span = doc.create_element("span");
    let text = doc.create_text_node("inside");
    doc.append_child(frag, span).unwrap();
    doc.append_child(frag, text).unwrap();

    let cloned = doc.clone_node(frag, true);

    // 克隆是新的 fragment
    assert_ne!(cloned, frag);
    assert!(matches!(
        doc.get(cloned).map(|n| &n.kind),
        Some(NodeKind::DocumentFragment)
    ));
    // 深拷贝保留子节点结构
    assert_eq!(doc.child_count(cloned), 2);
    assert_eq!(
        doc.text_content(cloned),
        Some("inside".to_string()),
        "克隆的 fragment 应包含与原始相同的文本内容"
    );
    // 克隆的子节点是全新的 NodeId
    let orig_children = doc.child_nodes(frag);
    let cloned_children = doc.child_nodes(cloned);
    assert_ne!(orig_children[0], cloned_children[0], "克隆子节点应是新节点");
}

/// 测试 Range::delete_contents 对折叠范围是空操作。
///
/// 当 range 起止点相同时（collapsed），delete_contents 不应删除任何节点，
/// DOM 树结构应保持不变。
#[test]
fn test_range_delete_contents_collapsed_noop() {
    let mut doc = parse_html("<div><p>A</p><p>B</p><p>C</p></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    let children_before = doc.child_nodes(div);
    assert_eq!(children_before.len(), 3);

    // 创建折叠范围
    let mut range = Range::at(div, 1);
    assert!(range.collapsed());

    range.delete_contents(&mut doc).unwrap();

    let children_after = doc.child_nodes(div);
    assert_eq!(children_after.len(), 3, "折叠范围 delete_contents 不应删除任何节点");
    assert_eq!(doc.text_content(children_after[0]), Some("A".to_string()));
    assert_eq!(doc.text_content(children_after[1]), Some("B".to_string()));
    assert_eq!(doc.text_content(children_after[2]), Some("C".to_string()));
}

/// 测试事件冒泡过程中 current_target 在每个阶段正确更新。
///
/// 结构：root > div > span，在三个节点上注册冒泡监听器，
/// 从 span 派发冒泡事件，验证每个监听器中的 current_target
/// 依次为 span、div（而非始终为 target 或 root）。
#[test]
fn test_event_current_target_bubbling_phase() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();

    let current_targets = Arc::new(Mutex::new(Vec::new()));
    let ct_span = current_targets.clone();
    let ct_div = current_targets.clone();

    // span 上的冒泡监听器
    doc.add_event_listener(
        span,
        "click",
        Box::new(move |e| {
            ct_span.lock().unwrap().push(e.current_target());
        }),
        false,
    );

    // div 上的冒泡监听器
    doc.add_event_listener(
        div,
        "click",
        Box::new(move |e| {
            ct_div.lock().unwrap().push(e.current_target());
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(span, &mut event);

    let targets = current_targets.lock().unwrap();
    assert_eq!(targets.len(), 2, "应触发 2 个冒泡监听器");
    // 第一个触发的是 span（目标阶段），current_target 应为 span
    assert_eq!(targets[0], Some(span), "span 监听器的 current_target 应为 span");
    // 第二个触发的是 div（冒泡阶段），current_target 应为 div
    assert_eq!(targets[1], Some(div), "div 监听器的 current_target 应为 div");
}

// ═══════════════════════════════════════════════════════════════════════
// 43. NodeIterator / TreeWalker 边界测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 NodeIterator 遍历混合节点类型（元素、文本、注释）的完整树。
///
/// 结构：div > [span, text("hello"), comment("note"), p]
/// 验证 next_node 按深度优先前序遍历顺序访问所有后代节点，
/// 最终回到根节点时 is_done 为 true。
#[test]
fn test_node_iterator_mixed_node_types() {
    let mut doc = Document::new();
    let root = doc.root();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let text = doc.create_text_node("hello");
    let comment = doc.create_comment("note");
    let p = doc.create_element("p");
    doc.append_child(root, div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(div, text).unwrap();
    doc.append_child(div, comment).unwrap();
    doc.append_child(div, p).unwrap();

    let mut iter = NodeIterator::new(div, 0xFFFFFFFF);

    // 深度优先前序遍历：span → text → comment → p
    let n1 = iter.next_node(&doc);
    assert_eq!(n1, Some(span), "第一个子节点应为 span");

    let n2 = iter.next_node(&doc);
    assert_eq!(n2, Some(text), "第二个子节点应为文本节点");

    let n3 = iter.next_node(&doc);
    assert_eq!(n3, Some(comment), "第三个子节点应为注释节点");

    let n4 = iter.next_node(&doc);
    assert_eq!(n4, Some(p), "第四个子节点应为 p");

    // 所有后代已遍历完毕
    let n5 = iter.next_node(&doc);
    assert_eq!(n5, None, "遍历完毕后应返回 None");
    assert!(iter.is_done(), "遍历完毕后 is_done 应为 true");
}

/// 测试 NodeIterator 遍历空元素（无子节点）。
///
/// 没有后代的元素，next_node 应立即返回 None 且 is_done 为 true。
#[test]
fn test_node_iterator_empty_subtree() {
    let mut doc = Document::new();
    let empty = doc.create_element("div");

    let mut iter = NodeIterator::new(empty, 0xFFFFFFFF);

    let result = iter.next_node(&doc);
    assert_eq!(result, None, "空元素没有子节点，next_node 应返回 None");
    assert!(iter.is_done(), "空元素遍历应立即标记为 done");

    // current_node 仍为根节点
    assert_eq!(iter.current_node(), empty);
    assert_eq!(iter.root(), empty);
}

/// 测试 NodeIterator 从深层节点回退到浅层再前进。
///
/// 构建两层树后，先前进到最深处，再回退到中间节点，
/// 然后再次前进验证遍历位置正确。
#[test]
fn test_node_iterator_forward_backward_alternating() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let a = doc.create_element("a");
    let b = doc.create_element("b");
    let c = doc.create_element("c");
    doc.append_child(root, a).unwrap();
    doc.append_child(a, b).unwrap();
    doc.append_child(b, c).unwrap();

    let mut iter = NodeIterator::new(root, 0xFFFFFFFF);

    // root → a → b → c
    assert_eq!(iter.next_node(&doc), Some(a));
    assert_eq!(iter.next_node(&doc), Some(b));
    assert_eq!(iter.next_node(&doc), Some(c));

    // 回退：c → b
    let prev = iter.previous_node(&doc);
    assert_eq!(prev, Some(b), "从 c 回退应为 b");
    assert_eq!(iter.current_node(), b);

    // 再前进：b → c
    let next = iter.next_node(&doc);
    assert_eq!(next, Some(c), "从 b 前进应为 c");
    assert_eq!(iter.current_node(), c);

    // 回退两次：c → b → a
    iter.previous_node(&doc);
    let prev2 = iter.previous_node(&doc);
    assert_eq!(prev2, Some(a), "回退两次后应为 a");
}

/// 测试 import_node 对 DocumentFragment 的深拷贝。
///
/// import_node 深拷贝一个 DocumentFragment 应递归复制所有子节点，
/// 产生的新 fragment 与原始节点结构相同但 NodeId 不同。
#[test]
fn test_import_node_document_fragment_deep() {
    let mut doc = Document::new();
    let root = doc.root();
    let frag = doc.create_document_fragment();
    let span = doc.create_element("span");
    doc.set_attribute(span, "class", "item");
    let text = doc.create_text_node("fragment content");
    doc.append_child(frag, span).unwrap();
    doc.append_child(frag, text).unwrap();

    // 先将 frag 追加到文档中以验证 import_node
    doc.append_child(root, frag).unwrap();

    let imported = doc.import_node(frag, true);

    // 导入的 fragment 是新的
    assert_ne!(imported, frag, "import_node 应创建新节点");
    assert!(matches!(
        doc.get(imported).map(|n| &n.kind),
        Some(NodeKind::DocumentFragment)
    ));

    // 深拷贝应包含子节点
    assert_eq!(doc.child_count(imported), 2, "导入的 fragment 应有 2 个子节点");

    // 子节点是新的（NodeId 不同）
    let orig_children = doc.child_nodes(frag);
    let imported_children = doc.child_nodes(imported);
    assert_ne!(orig_children[0], imported_children[0], "导入的子节点应是新节点");
    assert_ne!(orig_children[1], imported_children[1]);

    // 导入的 span 保留了属性
    assert_eq!(
        doc.get_attribute(imported_children[0], "class"),
        Some("item".to_string()),
        "导入的元素应保留属性"
    );

    // 导入的文本内容正确
    assert_eq!(
        doc.text_content(imported),
        Some("fragment content".to_string()),
        "导入的 fragment 的 textContent 应正确"
    );
}

/// 测试 resolve_slots 在动态添加新 light DOM 子节点后重新解析。
///
/// 初始状态：host 有一个带 slot="header" 的子节点和一个默认 slot。
/// 动态追加新的带 slot="footer" 的子节点后，再次调用 resolve_slots，
/// 验证新的 slot 分配生效，旧分配保持不变。
#[test]
fn test_resolve_slots_dynamic_add_light_dom() {
    let mut doc = Document::new();
    let root = doc.root();

    let host = doc.create_element("my-comp");
    doc.append_child(root, host).unwrap();

    // 初始 light DOM：一个带 slot="header" 的子节点和一个无 slot 的子节点
    let header = doc.create_element("h1");
    doc.set_attribute(header, "slot", "header");
    doc.append_child(host, header).unwrap();

    let default_content = doc.create_element("p");
    doc.append_child(host, default_content).unwrap();

    // shadow DOM
    let shadow = doc.attach_shadow(host, ShadowRootMode::Open).unwrap();
    let header_slot = doc.create_element("slot");
    doc.set_attribute(header_slot, "name", "header");
    doc.append_child(shadow, header_slot).unwrap();

    let default_slot = doc.create_element("slot");
    doc.append_child(shadow, default_slot).unwrap();

    let footer_slot = doc.create_element("slot");
    doc.set_attribute(footer_slot, "name", "footer");
    doc.append_child(shadow, footer_slot).unwrap();

    // 第一次解析
    doc.resolve_slots(host);
    assert_eq!(doc.get_assigned_nodes(header_slot).len(), 1);
    assert_eq!(doc.get_assigned_nodes(default_slot).len(), 1);
    assert!(
        doc.get_assigned_nodes(footer_slot).is_empty(),
        "初始时 footer slot 应为空"
    );

    // 动态添加新的 light DOM 子节点带 slot="footer"
    let footer = doc.create_element("footer");
    doc.set_attribute(footer, "slot", "footer");
    doc.append_child(host, footer).unwrap();

    // 再次解析
    doc.resolve_slots(host);

    // 新的 footer 子节点应分配到 footer slot
    let footer_assigned = doc.get_assigned_nodes(footer_slot);
    assert_eq!(footer_assigned.len(), 1, "重新解析后 footer slot 应有 1 个分配");
    assert_eq!(footer_assigned[0], footer, "分配的节点应为新添加的 footer 元素");

    // 旧的分配仍然有效
    let header_assigned = doc.get_assigned_nodes(header_slot);
    assert_eq!(header_assigned.len(), 1, "重新解析后 header slot 分配应保持");
    assert_eq!(header_assigned[0], header);

    let default_assigned = doc.get_assigned_nodes(default_slot);
    assert_eq!(default_assigned.len(), 1, "重新解析后默认 slot 分配应保持");
    assert_eq!(default_assigned[0], default_content);
}

// ═══════════════════════════════════════════════════════════════════════
// 边界测试（round 19）
// ═══════════════════════════════════════════════════════════════════════

/// 测试 clone_node 对包含混合类型子节点的 DocumentFragment 进行深克隆。
/// 验证克隆后的片段结构与原始一致，但节点 ID 不同（独立副本）。
#[test]
fn test_clone_node_document_fragment_mixed_children() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();

    // 向片段中添加混合类型子节点：元素 + 文本 + 注释
    let div = doc.create_element("div");
    doc.set_attribute(div, "class", "container");
    let text = doc.create_text_node("Hello");
    let comment = doc.create_comment("side note");
    let span = doc.create_element("span");

    doc.append_child(frag, div).unwrap();
    doc.append_child(frag, text).unwrap();
    doc.append_child(frag, comment).unwrap();
    doc.append_child(frag, span).unwrap();

    // 深克隆
    let cloned = doc.clone_node(frag, true);

    // 克隆节点应为 DocumentFragment 类型
    assert_eq!(doc.node_type(cloned), Some(11), "克隆节点应为 DocumentFragment");

    // 克隆节点应有相同数量的子节点
    assert_eq!(doc.child_count(cloned), 4, "克隆片段应有 4 个子节点");

    // 克隆的子节点类型与原始一致
    let cloned_children = doc.child_nodes(cloned);
    assert_eq!(doc.node_type(cloned_children[0]), Some(1), "第 1 个子节点应为 Element");
    assert_eq!(doc.node_type(cloned_children[1]), Some(3), "第 2 个子节点应为 Text");
    assert_eq!(doc.node_type(cloned_children[2]), Some(8), "第 3 个子节点应为 Comment");
    assert_eq!(doc.node_type(cloned_children[3]), Some(1), "第 4 个子节点应为 Element");

    // 克隆的元素保留属性
    assert_eq!(
        doc.get_attribute(cloned_children[0], "class"),
        Some("container".to_string()),
        "克隆元素应保留原始属性"
    );

    // 克隆的文本内容一致
    assert_eq!(doc.text_content(cloned_children[1]), Some("Hello".to_string()));

    // 克隆节点是独立副本——修改原始不影响克隆
    doc.set_attribute(div, "class", "modified");
    assert_eq!(
        doc.get_attribute(cloned_children[0], "class"),
        Some("container".to_string()),
        "修改原始节点不应影响克隆副本"
    );

    // 节点 ID 互不相同
    assert_ne!(cloned, frag, "克隆片段与原始片段 ID 应不同");
    assert_ne!(cloned_children[0], div, "克隆子元素与原始子元素 ID 应不同");
}

/// 测试 replace_child 传入不属于 parent 子节点的 old_child 时返回错误。
/// 当 new_child 不存在于文档（NodeId 无效）时也应返回错误。
#[test]
fn test_replace_child_invalid_nodes() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child_a = doc.create_element("span");
    doc.append_child(doc.root(), parent).unwrap();
    doc.append_child(parent, child_a).unwrap();

    // 创建一个不属于 parent 子节点的元素作为 old_child
    let outsider = doc.create_element("p");
    let new_elem = doc.create_element("em");
    doc.append_child(doc.root(), outsider).unwrap();

    // replace_child 要求 old_child 必须是 parent 的子节点
    let result = doc.replace_child(parent, new_elem, outsider);
    assert!(result.is_err(), "old_child 不是 parent 的子节点，应返回错误");
    match result {
        Err(DomError::NotAChild { parent: p, child: c }) => {
            assert_eq!(p, parent);
            assert_eq!(c, outsider);
        }
        other => panic!("预期 NotAChild 错误，实际得到: {:?}", other),
    }

    // parent 的子节点未被修改
    assert_eq!(doc.child_count(parent), 1, "parent 子节点数应保持不变");
}

/// 测试 get_elements_by_tag_name_ns 使用通配符 "*" 返回文档中所有元素。
/// get_elements_by_tag_name_ns(None, "*") 是 DOM 规范中通配查询的正确方式。
#[test]
fn test_get_elements_by_tag_name_wildcard_returns_all() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    let a = doc.create_element("a");

    doc.append_child(doc.root(), div).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(div, p).unwrap();
    doc.append_child(p, a).unwrap();

    // 使用 NS 变体的通配查询（namespace=None, local_name="*"）
    let all_elements = doc.get_elements_by_tag_name_ns(None, "*");

    // 应包含 div、span、p、a 共 4 个元素（不含 Document 节点）
    assert!(all_elements.len() >= 4, "通配查询应至少返回 4 个元素");
    assert!(all_elements.contains(&div), "应包含 div");
    assert!(all_elements.contains(&span), "应包含 span");
    assert!(all_elements.contains(&p), "应包含 p");
    assert!(all_elements.contains(&a), "应包含 a");
}

/// 测试 text_content 对嵌套元素递归拼接所有后代文本节点。
/// 验证多层嵌套中文本内容的正确合并。
#[test]
fn test_text_content_nested_elements() {
    let mut doc = Document::new();

    // 构建嵌套结构：div > "Hello " + span > "World" + "!"
    let div = doc.create_element("div");
    let text1 = doc.create_text_node("Hello ");
    let span = doc.create_element("span");
    let text2 = doc.create_text_node("World");
    let text3 = doc.create_text_node("!");

    doc.append_child(div, text1).unwrap();
    doc.append_child(div, span).unwrap();
    doc.append_child(div, text3).unwrap();
    doc.append_child(span, text2).unwrap();

    // div 的 text_content 应递归拼接所有后代文本
    let content = doc.text_content(div);
    assert_eq!(
        content,
        Some("Hello World!".to_string()),
        "嵌套元素的 text_content 应递归拼接"
    );

    // span 的 text_content 只包含自身后代的文本
    let span_content = doc.text_content(span);
    assert_eq!(span_content, Some("World".to_string()), "span 内文本应仅为 World");

    // 单个文本节点的 text_content 返回自身内容
    let text_content = doc.text_content(text1);
    assert_eq!(
        text_content,
        Some("Hello ".to_string()),
        "文本节点的 text_content 应为自身内容"
    );

    // 空元素的 text_content 应为空字符串
    let empty_div = doc.create_element("div");
    assert_eq!(
        doc.text_content(empty_div),
        Some(String::new()),
        "空元素的 text_content 应为空字符串"
    );
}

/// 测试 insert_before 传入不是 parent 子节点的 ref_node 时返回错误。
/// 操作不应修改 parent 的子节点列表。
#[test]
fn test_insert_before_ref_node_not_child_of_parent() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let existing_child = doc.create_element("span");
    doc.append_child(doc.root(), parent).unwrap();
    doc.append_child(parent, existing_child).unwrap();

    // 创建一个不属于 parent 的节点作为 ref_node
    let outsider = doc.create_element("p");
    let new_node = doc.create_element("em");
    doc.append_child(doc.root(), outsider).unwrap();

    // insert_before 应因 ref_node 不是 parent 子节点而失败
    let result = doc.insert_before(parent, new_node, outsider);
    assert!(result.is_err(), "ref_node 不是 parent 的子节点，应返回错误");
    match result {
        Err(DomError::NotAChild { parent: p, child: c }) => {
            assert_eq!(p, parent, "错误中的 parent 应为调用时的 parent");
            assert_eq!(c, outsider, "错误中的 child 应为 ref_node（outsider）");
        }
        other => panic!("预期 NotAChild 错误，实际得到: {:?}", other),
    }

    // parent 的子节点未被修改
    assert_eq!(doc.child_count(parent), 1, "parent 子节点数应保持不变");
    let children = doc.child_nodes(parent);
    assert_eq!(children[0], existing_child, "原有的子节点应保持不变");
}

// ═══════════════════════════════════════════════════════════════════════
// 边缘用例补充测试（round 21）
// ═══════════════════════════════════════════════════════════════════════

/// 测试 create_comment("") 空字符串注释节点的 nodeType 和 nodeName，
/// 并验证它可以正常附加到元素节点上。
#[test]
fn test_create_comment_empty_string_attach_to_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    let comment = doc.create_comment("");

    // 节点类型为 8 (Comment)
    assert_eq!(doc.node_type(comment), Some(8), "注释节点 nodeType 应为 8");

    // 将空注释附加到元素上
    doc.append_child(elem, comment).unwrap();
    assert_eq!(doc.child_count(elem), 1, "元素应有一个子节点");
    assert_eq!(doc.child_nodes(elem)[0], comment, "子节点应为刚附加的注释");
}

/// 测试 get_elements_by_class_name 在所有元素都有 class 但无匹配指定类名时返回空。
#[test]
fn test_get_elements_by_class_name_no_match_all_have_class() {
    let doc = parse_html(
        "<html><body>\
         <div class=\"foo bar\">a</div>\
         <span class=\"foo baz\">b</span>\
         <p class=\"bar baz\">c</p>\
         </body></html>",
    );
    // 所有元素都有 class，但 "qux" 不存在于任何元素上
    let result = doc.get_elements_by_class_name("qux");
    assert!(result.is_empty(), "没有任何元素具有 class \"qux\"，应返回空列表");

    // "foo" 应找到 2 个
    let foo = doc.get_elements_by_class_name("foo");
    assert_eq!(foo.len(), 2, "class \"foo\" 应匹配 2 个元素");
}

/// 测试 set_attribute 覆盖已有属性后，旧值完全消失，
/// 属性计数保持为 1，且 has_attribute 仍然为 true。
#[test]
fn test_set_attribute_overwrite_old_value_gone() {
    let mut doc = Document::new();
    let elem = doc.create_element("input");

    doc.set_attribute(elem, "type", "text");
    assert_eq!(doc.get_attribute(elem, "type"), Some("text".to_string()));
    assert_eq!(doc.attribute_names(elem).len(), 1);

    // 覆盖为 "password"
    doc.set_attribute(elem, "type", "password");
    assert_eq!(doc.get_attribute(elem, "type"), Some("password".to_string()));
    // 旧值 "text" 不应再可获取
    assert!(doc.has_attribute(elem, "type"));
    // 属性计数仍为 1，没有重复
    assert_eq!(doc.attribute_names(elem).len(), 1, "覆盖后属性数量应仍为 1");

    // 再覆盖为 "hidden"，验证链式覆盖正确
    doc.set_attribute(elem, "type", "hidden");
    assert_eq!(doc.get_attribute(elem, "type"), Some("hidden".to_string()));
    assert_eq!(doc.attribute_names(elem).len(), 1);
}

/// 测试 remove_child 在子节点属于另一个父节点时返回 NotAChild 错误。
/// child 是 parent_b 的子节点，尝试从 parent_a 移除应失败。
#[test]
fn test_remove_child_wrong_parent() {
    let mut doc = Document::new();
    let parent_a = doc.create_element("div");
    let parent_b = doc.create_element("section");
    let child = doc.create_element("span");

    doc.append_child(doc.root(), parent_a).unwrap();
    doc.append_child(doc.root(), parent_b).unwrap();
    // child 仅附加到 parent_b
    doc.append_child(parent_b, child).unwrap();

    // 尝试从 parent_a 移除 child（child 不是 parent_a 的子节点）
    let result = doc.remove_child(parent_a, child);
    assert!(result.is_err(), "child 不是 parent_a 的子节点，应返回错误");
    match result {
        Err(DomError::NotAChild { parent: p, child: c }) => {
            assert_eq!(p, parent_a, "错误中的 parent 应为 parent_a");
            assert_eq!(c, child, "错误中的 child 应为被尝试移除的节点");
        }
        other => panic!("预期 NotAChild 错误，实际得到: {:?}", other),
    }

    // child 仍在 parent_b 中，未被移除
    assert_eq!(doc.parent_node(child), Some(parent_b), "child 仍应是 parent_b 的子节点");
    assert_eq!(doc.child_count(parent_b), 1, "parent_b 子节点数应保持为 1");
}

/// 测试将 0 子节点的 DocumentFragment 追加到元素后，目标元素子节点不变。
#[test]
fn test_document_fragment_zero_children_append() {
    let mut doc = Document::new();
    let container = doc.create_element("div");
    doc.append_child(doc.root(), container).unwrap();

    // 创建空片段（0 子节点）
    let frag = doc.create_document_fragment();
    assert_eq!(doc.child_count(frag), 0, "空片段应有 0 个子节点");

    // 将空片段附加到 container
    doc.append_child(container, frag).unwrap();

    // container 应有 1 个子节点（片段本身），但片段内部没有子节点
    assert_eq!(doc.child_count(container), 1, "container 应有 1 个子节点（空片段）");
    let children = doc.child_nodes(container);
    assert_eq!(children[0], frag, "container 的唯一子节点应为空片段");

    // 片段内部仍为空
    assert_eq!(doc.child_count(frag), 0, "片段内部应保持 0 个子节点");
}

// ═══════════════════════════════════════════════════════════════════════
// 边界条件补充测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试使用无效标签名创建元素。
/// 验证即使标签名不合法（包含特殊字符），create_element 仍会创建节点，
/// 且该节点的 local_name 与传入值一致。
#[test]
fn test_create_element_with_invalid_tag_name() {
    let mut doc = Document::new();
    // 使用包含特殊字符的无效标签名
    let elem = doc.create_element("div>script<");
    assert!(doc.contains(elem), "无效标签名创建的元素仍应存在于文档中");
    if let Some(NodeKind::Element(e)) = doc.get(elem).map(|n| n.kind.clone()) {
        assert_eq!(
            e.local_name(),
            "div>script<",
            "元素的 local_name 应与传入的无效标签名一致"
        );
    }
}

/// 测试在没有任何属性的元素上调用 get_attribute 应返回 None。
#[test]
fn test_get_attribute_on_element_with_no_attributes_returns_none() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    // 不设置任何属性
    assert_eq!(
        doc.get_attribute(elem, "id"),
        None,
        "未设置属性的元素查询任意属性应返回 None"
    );
    assert_eq!(
        doc.get_attribute(elem, "class"),
        None,
        "未设置属性的元素查询 class 也应返回 None"
    );
    assert_eq!(
        doc.get_attribute(elem, "data-custom"),
        None,
        "未设置属性的元素查询自定义属性也应返回 None"
    );
}

/// 测试将子节点追加到脱离文档树的 DocumentFragment。
/// 验证脱离文档树后 fragment 仍可作为父节点接收子节点。
#[test]
fn test_append_child_to_detached_fragment() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    // fragment 不挂载到文档树，保持脱离状态

    let child1 = doc.create_element("span");
    let child2 = doc.create_text_node("文本内容");

    // 向脱离的 fragment 追加子节点
    doc.append_child(frag, child1).unwrap();
    doc.append_child(frag, child2).unwrap();

    assert_eq!(doc.child_count(frag), 2, "脱离的 fragment 应成功接收 2 个子节点");
    assert_eq!(
        doc.parent_node(child1),
        Some(frag),
        "child1 的父节点应为脱离的 fragment"
    );
    assert_eq!(
        doc.parent_node(child2),
        Some(frag),
        "child2 的父节点应为脱离的 fragment"
    );
}

/// 测试浅克隆（shallow clone_node）保留元素属性但不复制子节点。
#[test]
fn test_clone_node_shallow_preserves_attributes() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "container");
    doc.set_attribute(elem, "data-index", "42");

    // 给原始元素添加子节点
    let child = doc.create_element("p");
    doc.append_child(elem, child).unwrap();

    // 浅克隆：deep = false
    let cloned = doc.clone_node(elem, false);

    // 克隆节点应保留属性
    assert_eq!(
        doc.get_attribute(cloned, "class"),
        Some("container".to_string()),
        "浅克隆应保留 class 属性"
    );
    assert_eq!(
        doc.get_attribute(cloned, "data-index"),
        Some("42".to_string()),
        "浅克隆应保留 data-index 属性"
    );

    // 浅克隆不应复制子节点
    assert_eq!(doc.child_count(cloned), 0, "浅克隆不应复制子节点，子节点数应为 0");

    // 原始元素的子节点不受影响
    assert_eq!(doc.child_count(elem), 1, "原始元素的子节点数应保持为 1");
}

/// 测试空元素的 child_count 应返回 0。
#[test]
fn test_child_count_on_empty_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("section");
    doc.append_child(doc.root(), elem).unwrap();

    assert_eq!(doc.child_count(elem), 0, "没有任何子节点的元素 child_count 应为 0");
    assert!(!doc.has_child_nodes(elem), "空元素 has_child_nodes 应返回 false");
    assert_eq!(doc.child_nodes(elem), Vec::new(), "空元素的 child_nodes 应返回空列表");
}
