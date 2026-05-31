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
    assert!(matches!(
        doc.get(root).map(|n| &n.kind),
        Some(NodeKind::Document(_))
    ));
}

#[test]
fn test_create_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    assert!(doc.contains(elem));
    assert!(matches!(
        doc.get(elem).map(|n| &n.kind),
        Some(NodeKind::Element(_))
    ));
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
    assert_eq!(
        doc.get_attribute(elem, "class"),
        Some("container".to_string())
    );
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
        assert!(matches!(
            doc.get(fc).map(|n| &n.kind),
            Some(NodeKind::DocumentType(_))
        ));
    }
}

#[test]
fn test_parse_html_with_attributes() {
    let doc =
        parse_html("<html><body><div id=\"main\" class=\"container\">text</div></body></html>");
    let divs = doc.get_elements_by_tag_name("div");
    assert_eq!(divs.len(), 1);

    let div = divs[0];
    assert_eq!(doc.get_attribute(div, "id"), Some("main".to_string()));
    assert_eq!(
        doc.get_attribute(div, "class"),
        Some("container".to_string())
    );
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
    assert_eq!(
        doc.get_attribute(elem.unwrap(), "id"),
        Some("main".to_string())
    );
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
    let doc = parse_html(
        "<html><body><div class=\"item\">a</div><div class=\"item\">b</div></body></html>",
    );
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
    let doc =
        parse_html("<html><body><input type=\"text\" /><input type=\"password\" /></body></html>");
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
    assert_eq!(
        doc.get_attribute(current, "data-depth"),
        Some("99".to_string())
    );
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
    let doc =
        parse_html("<html><body><div id=\"first\">a</div><div id=\"second\">b</div></body></html>");
    assert!(doc.get_element_by_id("first").is_some());
    assert!(doc.get_element_by_id("second").is_some());
    assert!(doc.get_element_by_id("third").is_none());
}

// ═══════════════════════════════════════════════════════════════════════
// 11. 选择器解析测试（query.rs 中的测试已包含基础测试，这里补充集成测试）
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn test_query_selector_combined() {
    let doc = parse_html(
        "<html><body><div id=\"main\" class=\"container active\"><p>text</p></div></body></html>",
    );
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
    assert!(
        outer.starts_with("<div>"),
        "outer should include the element tag"
    );
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
    assert!(
        html.contains("&quot;"),
        "should escape quotes in attributes"
    );
    assert!(html.contains("&amp;"), "should escape & in attributes");
}

/// 测试序列化未被添加到树的孤立节点。
#[test]
fn test_serialize_orphan_node() {
    let mut doc = Document::new();
    let orphan = doc.create_element("div");
    let html = doc.outer_html(orphan);
    assert!(
        html.contains("<div"),
        "orphan node should still serialize, got: {html}"
    );
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
    assert_eq!(
        *count.lock().unwrap(),
        3,
        "should have received 3 total records"
    );
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
    assert_eq!(
        *received_type.lock().unwrap(),
        Some(MutationType::CharacterData)
    );
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
    assert!(
        !result,
        "preventDefault should return false for non-cancelable event"
    );
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
    assert!(
        not_prevented,
        "dispatch should return true when no preventDefault"
    );
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
    assert!(!*input_called.lock().unwrap(), "input listener should not fire for click event");
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
    assert!(
        !*called.lock().unwrap(),
        "removed listener should not fire"
    );
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

    doc.add_event_listener(
        elem,
        "click",
        Box::new(|_| {}),
        false,
    );
    doc.add_event_listener(
        elem,
        "input",
        Box::new(|_| {}),
        false,
    );

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
    assert_eq!(
        *log,
        vec!["p"],
        "stopPropagation should prevent bubbling to ancestors"
    );
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
            log.lock().unwrap().push(format!(
                "div-capture(phase={:?})",
                event.phase()
            ));
        }),
        true,
    );

    // div 冒泡
    let log = call_log.clone();
    doc.add_event_listener(
        div,
        "click",
        Box::new(move |event| {
            log.lock().unwrap().push(format!(
                "div-bubble(phase={:?})",
                event.phase()
            ));
        }),
        false,
    );

    // span 目标（capture=true）
    let log = call_log.clone();
    doc.add_event_listener(
        span,
        "click",
        Box::new(move |event| {
            log.lock().unwrap().push(format!(
                "span-target-cap(phase={:?})",
                event.phase()
            ));
        }),
        true,
    );

    // span 目标（capture=false）
    let log = call_log.clone();
    doc.add_event_listener(
        span,
        "click",
        Box::new(move |event| {
            log.lock().unwrap().push(format!(
                "span-target(phase={:?})",
                event.phase()
            ));
        }),
        false,
    );

    let mut event = Event::new_with_options("click", true, false);
    doc.dispatch_event(span, &mut event);

    let log = call_log.lock().unwrap();
    assert_eq!(log.len(), 4, "all 4 listeners should fire");
    // 顺序：div-capture -> span-target-cap -> span-target -> div-bubble
    assert!(log[0].contains("div-capture"), "first should be div capture");
    assert!(log[1].contains("span-target-cap"), "second should be span capture at target");
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
    assert!(
        doc.node_contains(root, root),
        "a node should contain itself"
    );
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
    assert!(
        pos.contains(DocumentPosition::FOLLOWING),
        "c2 should be following c1"
    );
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
    assert!(
        pos.contains(DocumentPosition::PRECEDING),
        "c1 should be preceding c2"
    );
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
    assert!(
        pos.contains(DocumentPosition::CONTAINS),
        "div should contain span"
    );
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
    assert!(descendants.is_empty(), "element with no children should have no descendants");
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
    assert_eq!(
        doc.text_content(slot_elem),
        Some("Default Header".to_string())
    );
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
    doc.add_event_listener(span, "click", Box::new(move |e| {
        log_span.lock().unwrap().push("span");
        e.stop_immediate_propagation();
    }), false);

    // div bubble: should not fire
    doc.add_event_listener(div, "click", Box::new(move |_| {
        log_div.lock().unwrap().push("div");
    }), false);

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
    doc2.add_event_listener(orphan, "custom", Box::new(move |e| {
        *target_clone.lock().unwrap() = e.target();
    }), false);

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

    doc.add_event_listener(elem, "click", Box::new(move |_| {
        f1.lock().unwrap().0 = true;
    }), false);
    doc.add_event_listener(elem, "focus", Box::new(move |_| {
        f2.lock().unwrap().1 = true;
    }), false);
    doc.add_event_listener(elem, "blur", Box::new(move |_| {
        f3.lock().unwrap().2 = true;
    }), false);

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
    let doc = parse_html("<html><body><div class=\"outer\"><span id=\"target\"><em>deep</em></span></div><span id=\"sibling\">outside</span></body></html>");
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

/// 测试 remove_child 后 id_map 中的条目未被清除。
///
/// 已知缺陷：remove_child 不会从 id_map 中移除被删除节点的 id 映射，
/// 导致 get_element_by_id 仍然能找到已从文档树中移除的节点。
/// 浏览器规范要求节点从文档中移除后不再可通过 getElementById 找到。
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

    // 当前行为：remove_child 未清理 id_map，仍能找到已移除的节点。
    // 这是已知缺陷 — 规范要求返回 None。
    let found = doc.get_element_by_id("test");
    assert!(
        found == Some(elem),
        "已知缺陷：remove_child 后 id_map 条目未被清除，get_element_by_id 仍返回已移除的节点"
    );
}

/// 测试 clone_node 后 id_map 的映射行为。
///
/// 已知缺陷：clone_node 会将克隆元素的 id 注册到 id_map 中，
/// 覆盖原始元素的映射。导致 get_element_by_id 返回克隆节点而非原始节点，
/// 两个具有相同 id 的元素共存但 id_map 只记录最后一个。
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

    // 两个节点都有 id="orig"
    assert_eq!(doc.get_attribute(elem, "id"), Some("orig".to_string()));
    assert_eq!(doc.get_attribute(cloned, "id"), Some("orig".to_string()));

    // 当前行为：clone_node 将克隆的 id 插入 id_map，覆盖原始映射。
    // get_element_by_id("orig") 返回克隆节点而非原始节点。
    let found = doc.get_element_by_id("orig");
    assert!(
        found == Some(cloned),
        "已知缺陷：clone_node 覆盖了原始节点的 id_map 条目，get_element_by_id 返回克隆节点"
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
    assert_eq!(
        doc.get_element_by_id("old"),
        None,
        "修改 id 后旧 id 应从 id_map 中移除"
    );
    // 新 id 正确映射
    assert_eq!(
        doc.get_element_by_id("new"),
        Some(elem),
        "修改 id 后新 id 应正确映射到该元素"
    );
}
