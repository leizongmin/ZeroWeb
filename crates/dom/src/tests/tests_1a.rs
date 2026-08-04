// DOM crate 综合测试套件（第一部分）。
//
// 覆盖：节点类型、树操作、属性操作、HTML 解析、查询、序列化、MutationObserver。

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
fn test_query_selector_nth_child() {
    let doc = parse_html("<html><body><div>a</div><div>b</div><div>c</div></body></html>");
    let root = doc.root();
    // :nth-child(2) → 第 2 个 div（"b"）。
    let r = doc.query_selector(root, "div:nth-child(2)").expect("nth-child(2)");
    assert_eq!(doc.text_content(r), Some("b".to_string()));
    // :nth-child(odd) → 第 1、3 个。
    let odds = doc.query_selector_all(root, "div:nth-child(odd)");
    assert_eq!(odds.len(), 2);
    // :first-child / :last-child。
    assert_eq!(
        doc.text_content(doc.query_selector(root, "div:first-child").unwrap()),
        Some("a".to_string())
    );
    assert_eq!(
        doc.text_content(doc.query_selector(root, "div:last-child").unwrap()),
        Some("c".to_string())
    );
}

#[test]
fn test_query_selector_nth_of_type() {
    // 混合 tag：p/div 交替；nth-of-type 只在同 tag 中计数。
    let doc = parse_html("<html><body><p>1</p><div>D1</div><p>2</p><div>D2</div><p>3</p></body></html>");
    let root = doc.root();
    // p:nth-of-type(2) → 第 2 个 p（"2"），尽管它是父的第 3 个元素子。
    let r = doc.query_selector(root, "p:nth-of-type(2)").expect("p:nth-of-type(2)");
    assert_eq!(doc.text_content(r), Some("2".to_string()));
    // div:last-of-type → "D2"。
    assert_eq!(
        doc.text_content(doc.query_selector(root, "div:last-of-type").unwrap()),
        Some("D2".to_string())
    );
    // p:first-of-type → "1"。
    assert_eq!(
        doc.text_content(doc.query_selector(root, "p:first-of-type").unwrap()),
        Some("1".to_string())
    );
}

#[test]
fn test_query_selector_structural_path_uniqueness() {
    // 多个无 id/class 的同 tag 元素——nth-child 路径唯一锁定。
    let doc = parse_html("<html><body><ul><li>1</li><li>2</li><li>3</li></ul></body></html>");
    let root = doc.root();
    let r = doc.query_selector(root, "li:nth-child(3)").expect("li:nth-child(3)");
    assert_eq!(doc.text_content(r), Some("3".to_string()));
    // 组合 combinator + nth-child：ul > li:nth-child(2)。
    let r2 = doc
        .query_selector(root, "ul > li:nth-child(2)")
        .expect("ul>li:nth-child(2)");
    assert_eq!(doc.text_content(r2), Some("2".to_string()));
}

#[test]
fn test_query_selector_not_pseudo() {
    let doc = parse_html(
        "<html><body><ul>\
         <li class='skip'>1</li><li>2</li><li class='skip'>3</li><li>4</li>\
         </ul></body></html>",
    );
    let root = doc.root();
    // li:not(.skip) → 第 2、4 个 li（"2"、"4"）。
    let matched = doc.query_selector_all(root, "li:not(.skip)");
    assert_eq!(matched.len(), 2, ":not(.skip) 应匹配 2 个");
    let texts: Vec<_> = matched.iter().filter_map(|id| doc.text_content(*id)).collect();
    assert_eq!(texts, vec!["2".to_string(), "4".to_string()]);
    // :not(:first-child) → 非首子的 li（"2"、"3"、"4"）。
    let not_first = doc.query_selector_all(root, "li:not(:first-child)");
    assert_eq!(not_first.len(), 3);
}

#[test]
fn test_query_selector_nth_last() {
    let doc = parse_html("<html><body><ul><li>1</li><li>2</li><li>3</li><li>4</li></ul></body></html>");
    let root = doc.root();
    // :nth-last-child(1) = :last-child → "4"。
    assert_eq!(
        doc.text_content(doc.query_selector(root, "li:nth-last-child(1)").unwrap()),
        Some("4".to_string())
    );
    // :nth-last-child(2) → "3"。
    assert_eq!(
        doc.text_content(doc.query_selector(root, "li:nth-last-child(2)").unwrap()),
        Some("3".to_string())
    );
    // :nth-last-of-type(odd) → 倒数第 1、3 → "4"、"2"。
    let odds = doc.query_selector_all(root, "li:nth-last-of-type(odd)");
    let texts: Vec<_> = odds.iter().filter_map(|id| doc.text_content(*id)).collect();
    assert_eq!(texts, vec!["2".to_string(), "4".to_string()]);
}

#[test]
fn test_query_selector_is_where() {
    let doc = parse_html(
        "<html><body>\
         <p class='a'>P-A</p><span class='b'>S-B</span><div class='a'>D-A</div><p class='c'>P-C</p>\
         </body></html>",
    );
    let root = doc.root();
    // :is(.a, .b) → class a 或 b（P-A, S-B, D-A）。
    let matched = doc.query_selector_all(root, ":is(.a, .b)");
    assert_eq!(matched.len(), 3, ":is(.a,.b) 应匹配 3 个");
    // :where(.c) → class c（P-C）。
    let c = doc.query_selector(root, ":where(.c)").expect(":where(.c)");
    assert_eq!(doc.text_content(c), Some("P-C".to_string()));
    // 组合：p:is(.a) → p 且 class a（P-A；排除 div.a）。
    let pa = doc.query_selector_all(root, "p:is(.a)");
    assert_eq!(pa.len(), 1);
    assert_eq!(doc.text_content(pa[0]), Some("P-A".to_string()));
}

#[test]
fn test_query_selector_has() {
    // 注意：p3 用 <section> 包裹（section 为块级、不触发 p 自动闭合、非 div），
    // 使 .child 真正成为 p3 的孙节点。若用 <p> 包裹，HTML5 解析器会在 <div> 前自动闭合 <p>，
    // 致 .child 升格为 p3 的直接子（与「孙」语义相悖）。
    let doc = parse_html(
        "<html><body>\
         <div class='parent' id='p1'><span class='child'>c</span></div>\
         <div class='parent' id='p2'>no child</div>\
         <div class='parent' id='p3'><section><div class='child'>x</div></section></div>\
         </body></html>",
    );
    let root = doc.root();
    let ids_of =
        |sels: &[NodeId]| -> Vec<String> { sels.iter().filter_map(|id| doc.get_attribute(*id, "id")).collect() };
    // div:has(.child) → p1（直接子 .child）、p3（后代 .child）。p2 无 .child → 不匹配。
    let has_child = doc.query_selector_all(root, "div:has(.child)");
    let ids = ids_of(&has_child);
    assert!(ids.contains(&"p1".to_string()), "p1 直接子 .child 应匹配 :has(.child)");
    assert!(ids.contains(&"p3".to_string()), "p3 后代 .child 应匹配 :has(.child)");
    assert!(!ids.contains(&"p2".to_string()), "p2 无 .child 不应匹配 :has(.child)");
    // :has(> .child) → 仅直接子为 .child 的（p1）。p3 的直接子是 section（非 .child）→ 不匹配。
    let has_direct = doc.query_selector_all(root, "div:has(> .child)");
    let direct_ids = ids_of(&has_direct);
    assert!(
        direct_ids.contains(&"p1".to_string()),
        "p1 直接子 .child 应匹配 :has(> .child)"
    );
    assert!(
        !direct_ids.contains(&"p3".to_string()),
        "p3 的 .child 是孙（在 section 内），不应匹配 :has(> .child)"
    );
    // 后代作用域内嵌组合器：:has(section .child) → 仅 p3（section 内有 .child）。
    let has_combinator = doc.query_selector_all(root, "div:has(section .child)");
    let comb_ids = ids_of(&has_combinator);
    assert!(comb_ids.contains(&"p3".to_string()), "p3 应匹配 :has(section .child)");
    assert!(
        !comb_ids.contains(&"p1".to_string()),
        "p1 无 section 不应匹配 :has(section .child)"
    );
    // 负向：无任何后代匹配 → 空集。
    assert!(doc.query_selector_all(root, "div:has(.nonexistent)").is_empty());
    assert!(doc.query_selector_all(root, "div:has(> .nonexistent)").is_empty());
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
