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
fn test_query_selector_form_state_pseudo() {
    let doc = parse_html(
        "<html><body>\
         <input type='checkbox' id='cb-on' checked>\
         <input type='checkbox' id='cb-off'>\
         <input type='radio' id='rd-on' checked>\
         <input type='text' id='txt' checked>\
         <input type='text' id='txt-dis' disabled>\
         <button id='btn-dis' disabled>b</button>\
         <button id='btn'>b2</button>\
         <select id='sel' disabled>\
           <option id='opt-a' selected>a</option>\
           <option id='opt-b'>b</option>\
         </select>\
         </body></html>",
    );
    let root = doc.root();
    let ids_of =
        |sels: &[NodeId]| -> Vec<String> { sels.iter().filter_map(|id| doc.get_attribute(*id, "id")).collect() };
    // :checked → cb-on（checked checkbox）、rd-on（checked radio）、opt-a（selected option）。
    // text#txt 虽带 checked 属性但 type=text 非 checkbox/radio → 不匹配。
    let checked = ids_of(&doc.query_selector_all(root, ":checked"));
    assert!(
        checked.contains(&"cb-on".to_string()),
        "checked checkbox 应匹配 :checked"
    );
    assert!(checked.contains(&"rd-on".to_string()), "checked radio 应匹配 :checked");
    assert!(
        checked.contains(&"opt-a".to_string()),
        "selected option 应匹配 :checked"
    );
    assert!(
        !checked.contains(&"cb-off".to_string()),
        "未选 checkbox 不应匹配 :checked"
    );
    assert!(
        !checked.contains(&"txt".to_string()),
        "type=text 带 checked 属性不应匹配 :checked"
    );
    // :disabled → txt-dis、btn-dis、sel（带 disabled 的表单控件）。opt-a/opt-b 位于禁用 <select>
    // 内 → HTML spec §4.10.10 传播禁用（R3277 select→option 禁用传播）。
    let disabled = ids_of(&doc.query_selector_all(root, ":disabled"));
    assert!(disabled.contains(&"txt-dis".to_string()));
    assert!(disabled.contains(&"btn-dis".to_string()));
    assert!(disabled.contains(&"sel".to_string()));
    assert!(
        disabled.contains(&"opt-a".to_string()),
        "禁用 <select> 内的 <option> 经传播应匹配 :disabled"
    );
    assert!(
        disabled.contains(&"opt-b".to_string()),
        "禁用 <select> 内的 <option> 经传播应匹配 :disabled"
    );
    assert!(!disabled.contains(&"btn".to_string()), "启用 button 不应匹配 :disabled");
    // :enabled → 表单控件且非禁用：cb-on/cb-off/rd-on/txt/btn。
    let enabled = ids_of(&doc.query_selector_all(root, ":enabled"));
    for id in ["cb-on", "cb-off", "rd-on", "txt", "btn"] {
        assert!(enabled.contains(&id.to_string()), "{id} 应匹配 :enabled");
    }
    assert!(
        !enabled.contains(&"txt-dis".to_string()),
        "禁用 input 不应匹配 :enabled"
    );
    assert!(
        !enabled.contains(&"btn-dis".to_string()),
        "禁用 button 不应匹配 :enabled"
    );
    // 组合：input:checked + input:disabled 互补，input:text:enabled 命中。
    let enabled_text = doc.query_selector_all(root, "input:enabled");
    assert!(
        enabled_text
            .iter()
            .any(|id| doc.get_attribute(*id, "id") == Some("txt".to_string()))
    );
    assert!(
        !enabled_text
            .iter()
            .any(|id| doc.get_attribute(*id, "id") == Some("txt-dis".to_string())),
        "禁用 text 不应在 input:enabled 中"
    );
}

#[test]
fn test_query_selector_fieldset_disabled_propagation_r3277() {
    // R3277：HTML spec §4.10.18 禁用传播。
    // <fieldset disabled> 内的后代表单控件经传播匹配 :disabled，**首个 <legend> 内除外**；
    // <select disabled> 内的 <option> 同样传播（§4.10.10）；<optgroup disabled> 内 option 传播。
    let doc = parse_html(
        "<html><body>\
         <fieldset id='fs' disabled>\
           <legend><input id='legend-in' type='text'></legend>\
           <input id='body-in' type='text'>\
           <select id='inner-sel'>\
             <option id='inner-opt'>x</option>\
           </select>\
           <button id='body-btn'>b</button>\
         </fieldset>\
         <fieldset id='fs2'>\
           <input id='outside' type='text'>\
         </fieldset>\
         <select id='sel-dis' disabled>\
           <optgroup id='og-dis' disabled><option id='og-opt'>y</option></optgroup>\
           <option id='sel-opt'>z</option>\
         </select>\
         <button id='free-btn'>f</button>\
         </body></html>",
    );
    let root = doc.root();
    let ids_of =
        |sels: &[NodeId]| -> Vec<String> { sels.iter().filter_map(|id| doc.get_attribute(*id, "id")).collect() };
    let disabled = ids_of(&doc.query_selector_all(root, ":disabled"));
    let enabled = ids_of(&doc.query_selector_all(root, ":enabled"));

    // fieldset 自身 disabled（样式按属性直判；:disabled 选择器对 fieldset 不匹配——
    // fieldset 不在 is_disableable_tag，本测用控件断言传播）。
    // legend 内控件豁免：legend-in 启用。
    assert!(
        !disabled.contains(&"legend-in".to_string()),
        "首个 legend 内控件应豁免 fieldset disabled 传播"
    );
    assert!(
        enabled.contains(&"legend-in".to_string()),
        "首个 legend 内控件应匹配 :enabled"
    );
    // fieldset body 内控件传播禁用。
    assert!(
        disabled.contains(&"body-in".to_string()),
        "禁用 fieldset body 内 input 应传播匹配 :disabled"
    );
    assert!(
        disabled.contains(&"body-btn".to_string()),
        "禁用 fieldset body 内 button 应传播匹配 :disabled"
    );
    // fieldset 内 select（未自身 disabled）的 option 亦经 fieldset 传播禁用。
    assert!(
        disabled.contains(&"inner-opt".to_string()),
        "禁用 fieldset 内 select 的 option 应传播匹配 :disabled"
    );
    // 未禁用 fieldset 内控件正常启用。
    assert!(
        enabled.contains(&"outside".to_string()),
        "未禁用 fieldset 内控件应匹配 :enabled"
    );
    // select disabled → 其 option 传播禁用（§4.10.10）。
    assert!(
        disabled.contains(&"sel-opt".to_string()),
        "禁用 select 内 option 应传播匹配 :disabled"
    );
    // optgroup disabled → 其 option 传播禁用（即便 select 亦禁用，option 仍禁用）。
    assert!(
        disabled.contains(&"og-opt".to_string()),
        "禁用 optgroup 内 option 应传播匹配 :disabled"
    );
    // 自由控件启用。
    assert!(enabled.contains(&"free-btn".to_string()), "无关 button 应匹配 :enabled");
    assert!(
        !disabled.contains(&"free-btn".to_string()),
        "无关 button 不应匹配 :disabled"
    );
}

#[test]
fn test_query_selector_form_state_required_readonly_r3278() {
    // R3278：DOM `:required`/`:optional`/`:read-only`/`:read-write` 选择器（此前仅 style-system CSS 支持，
    // querySelector/querySelectorAll 不识别）。语义与 style-system 同源（HTML spec）：
    //   :required/:optional——可约束表单控件（input/select/textarea）带/不带 `required`；
    //   :read-write——文本可编辑 type 的 input 或 textarea，无 readonly/disabled；
    //   :read-only——非 :read-write（含所有非表单元素）。
    let doc = parse_html(
        "<html><body>\
         <input id='req-in' type='text' required>\
         <input id='opt-in' type='text'>\
         <input id='ro-in' type='text' readonly>\
         <input id='cb' type='checkbox' required>\
         <select id='sel' required><option>x</option></select>\
         <textarea id='ta'></textarea>\
         <input id='btn-in' type='button'>\
         <p id='p'>text</p>\
         <fieldset disabled><input id='fs-in' type='text'></fieldset>\
         </body></html>",
    );
    let root = doc.root();
    let ids_of =
        |sels: &[NodeId]| -> Vec<String> { sels.iter().filter_map(|id| doc.get_attribute(*id, "id")).collect() };

    // :required → 可约束控件带 required：req-in（text+required）、cb（checkbox+required）、sel（select+required）。
    let required = ids_of(&doc.query_selector_all(root, ":required"));
    assert!(
        required.contains(&"req-in".to_string()),
        "required text input 应匹配 :required"
    );
    assert!(
        required.contains(&"cb".to_string()),
        "required checkbox 应匹配 :required"
    );
    assert!(
        required.contains(&"sel".to_string()),
        "required select 应匹配 :required"
    );
    assert!(
        !required.contains(&"opt-in".to_string()),
        "无 required 的 input 不应匹配 :required"
    );
    assert!(
        !required.contains(&"btn-in".to_string()),
        "type=button 不可约束，即使有 required 也不应匹配 :required"
    );

    // :optional → 可约束控件无 required：opt-in、ta（textarea 可约束无 required）。
    let optional = ids_of(&doc.query_selector_all(root, ":optional"));
    assert!(
        optional.contains(&"opt-in".to_string()),
        "无 required input 应匹配 :optional"
    );
    assert!(
        optional.contains(&"ta".to_string()),
        "无 required textarea 应匹配 :optional"
    );
    assert!(
        !optional.contains(&"req-in".to_string()),
        "required input 不应匹配 :optional"
    );

    // :read-write → 文本可编辑 input/textarea 无 readonly/disabled：opt-in、ta、req-in（required 仍可编辑）。
    let read_write = ids_of(&doc.query_selector_all(root, ":read-write"));
    assert!(
        read_write.contains(&"opt-in".to_string()),
        "普通 text input 应匹配 :read-write"
    );
    assert!(read_write.contains(&"ta".to_string()), "textarea 应匹配 :read-write");
    assert!(
        read_write.contains(&"req-in".to_string()),
        "required text input 仍可编辑，应匹配 :read-write"
    );
    assert!(
        !read_write.contains(&"ro-in".to_string()),
        "readonly input 不应匹配 :read-write"
    );
    assert!(
        !read_write.contains(&"btn-in".to_string()),
        "type=button 非文本可编辑，不应匹配 :read-write"
    );
    // fieldset 传播禁用：fs-in 自身无 disabled，但位于禁用 fieldset 内 → 经传播禁用 → 只读。
    assert!(
        !read_write.contains(&"fs-in".to_string()),
        "禁用 fieldset 内 input 经传播禁用应为只读，不应匹配 :read-write"
    );

    // :read-only → 非 :read-write：ro-in（readonly）、btn-in（非文本）、cb（checkbox）、p（非表单）。
    let read_only = ids_of(&doc.query_selector_all(root, ":read-only"));
    assert!(
        read_only.contains(&"ro-in".to_string()),
        "readonly input 应匹配 :read-only"
    );
    assert!(read_only.contains(&"p".to_string()), "非表单 <p> 应匹配 :read-only");
    assert!(read_only.contains(&"cb".to_string()), "checkbox 应匹配 :read-only");
    assert!(
        read_only.contains(&"fs-in".to_string()),
        "禁用 fieldset 内 input 经传播禁用应匹配 :read-only"
    );
    assert!(
        !read_only.contains(&"opt-in".to_string()),
        "可编辑 text input 不应匹配 :read-only"
    );

    // 组合：input:required 与 input:optional 互补（验证多伪类 AND）。
    let req_text = ids_of(&doc.query_selector_all(root, "input:required"));
    assert!(req_text.contains(&"req-in".to_string()));
    assert!(req_text.contains(&"cb".to_string()));
    assert!(
        !req_text.contains(&"opt-in".to_string()),
        "input:required 不应含 optional input"
    );
}

#[test]
fn test_query_selector_placeholder_default_indeterminate_r3279() {
    // R3279：DOM `:placeholder-shown`/`:default`/`:indeterminate` 选择器（此前仅 CSS 侧实现）。
    // 语义与 style-system 同源（HTML spec），共享 Document 权威方法：
    //   :placeholder-shown——input/textarea 有 placeholder 且当前无值；
    //   :default——option selected / checkbox|radio checked / form 内首个 submit 按钮；
    //   :indeterminate——progress 无 value / radio 组（同 name+同 form）内无 checked 成员。
    let doc = parse_html(
        "<html><body>\
         <form id='form1'>\
           <input id='ph-empty' type='text' placeholder='hint'>\
           <input id='ph-filled' type='text' placeholder='hint' value='x'>\
           <textarea id='ph-ta-empty' placeholder='hint'></textarea>\
           <textarea id='ph-ta-filled' placeholder='hint'>text</textarea>\
           <input id='def-cb' type='checkbox' checked>\
           <input id='def-radio-a' type='radio' name='g' checked>\
           <input id='def-radio-b' type='radio' name='g'>\
           <option id='def-opt' selected>x</option>\
           <button id='def-submit' type='submit'>go</button>\
           <button id='def-btn-type' type='button'>no</button>\
         </form>\
         <input id='ind-radio1' type='radio' name='r'>\
         <input id='ind-radio2' type='radio' name='r'>\
         <input id='ind-radio-checked-grp' type='radio' name='c' checked>\
         <input id='ind-radio-other' type='radio' name='c'>\
         <progress id='ind-prog'></progress>\
         <progress id='det-prog' value='50'></progress>\
         </body></html>",
    );
    let root = doc.root();
    let ids_of =
        |sels: &[NodeId]| -> Vec<String> { sels.iter().filter_map(|id| doc.get_attribute(*id, "id")).collect() };

    // :placeholder-shown → 有 placeholder 且无值：ph-empty（input value 空）、ph-ta-empty（textarea 空）。
    let ph_shown = ids_of(&doc.query_selector_all(root, ":placeholder-shown"));
    assert!(
        ph_shown.contains(&"ph-empty".to_string()),
        "有 placeholder 且无值的 input 应匹配 :placeholder-shown"
    );
    assert!(
        ph_shown.contains(&"ph-ta-empty".to_string()),
        "有 placeholder 且无内容的 textarea 应匹配 :placeholder-shown"
    );
    assert!(
        !ph_shown.contains(&"ph-filled".to_string()),
        "有 value 的 input 不应匹配 :placeholder-shown"
    );
    assert!(
        !ph_shown.contains(&"ph-ta-filled".to_string()),
        "有内容的 textarea 不应匹配 :placeholder-shown"
    );

    // :default → def-cb（checkbox checked）、def-radio-a（radio checked）、def-opt（option selected）、
    // def-submit（form 内首个 submit 按钮）；def-btn-type（type=button）非 submit 候选不匹配；
    // def-radio-b（未 checked）不匹配。
    let default = ids_of(&doc.query_selector_all(root, ":default"));
    assert!(
        default.contains(&"def-cb".to_string()),
        "checked checkbox 应匹配 :default"
    );
    assert!(
        default.contains(&"def-radio-a".to_string()),
        "checked radio 应匹配 :default"
    );
    assert!(
        default.contains(&"def-opt".to_string()),
        "selected option 应匹配 :default"
    );
    assert!(
        default.contains(&"def-submit".to_string()),
        "form 内首个 submit 按钮应匹配 :default"
    );
    assert!(
        !default.contains(&"def-btn-type".to_string()),
        "type=button 非 submit 候选，不应匹配 :default"
    );
    assert!(
        !default.contains(&"def-radio-b".to_string()),
        "未 checked radio 不应匹配 :default"
    );

    // :indeterminate → ind-prog（progress 无 value）、ind-radio1/ind-radio2（组 r 无 checked）；
    // ind-radio-other（组 c 有 checked 成员 ind-radio-checked-grp）不匹配；det-prog（有 value）不匹配。
    let indeterminate = ids_of(&doc.query_selector_all(root, ":indeterminate"));
    assert!(
        indeterminate.contains(&"ind-prog".to_string()),
        "无 value 的 progress 应匹配 :indeterminate"
    );
    assert!(
        indeterminate.contains(&"ind-radio1".to_string()),
        "组内无 checked 的 radio 应匹配 :indeterminate"
    );
    assert!(
        indeterminate.contains(&"ind-radio2".to_string()),
        "组内无 checked 的 radio 应匹配 :indeterminate"
    );
    assert!(
        !indeterminate.contains(&"ind-radio-other".to_string()),
        "组内有 checked 成员时 radio 不应匹配 :indeterminate"
    );
    assert!(
        !indeterminate.contains(&"det-prog".to_string()),
        "有 value 的 progress 不应匹配 :indeterminate"
    );
}

#[test]
fn test_query_selector_anylink_scope_lang_dir_r3281() {
    // R3281：DOM `:any-link`/`:link`/`:visited`/`:scope`/`:lang()`/`:dir()` 选择器（此前仅 CSS 侧
    // 实现）→ querySelector 走 DOM 路径不识别。语义与 style-system 同源（CSS Selectors L4），
    // 共享 Document 权威方法（lang/dir/scope 提升至 document/lang_dir.rs）。
    let doc = parse_html(
        "<html lang='en' dir='ltr'><body>\
         <a id='link-a' href='https://example.com'>A</a>\
         <a id='link-nohref'>no href</a>\
         <area id='link-area' href='/x'>\
         <link id='link-elem' href='styles.css'>\
         <p id='en-p'>english</p>\
         <p id='en-us-p' lang='en-US'>american</p>\
         <p id='fr-p' lang='fr'>français</p>\
         <p id='rtl-p' dir='rtl'>مرحبا</p>\
         <p id='rtl-inherit' dir='rtl'><span id='rtl-child'>kid</span></p>\
         <div id='rtl-auto' dir='auto'>مرحبا ABC</div>\
         </body></html>",
    );
    let root = doc.root();
    let ids_of =
        |sels: &[NodeId]| -> Vec<String> { sels.iter().filter_map(|id| doc.get_attribute(*id, "id")).collect() };

    // :any-link / :link → a/area/link 带 href：link-a、link-area、link-elem；link-nohref 无 href 不匹配。
    for sel in [":any-link", ":link"] {
        let links = ids_of(&doc.query_selector_all(root, sel));
        assert!(links.contains(&"link-a".to_string()), "{sel} 应匹配带 href 的 <a>");
        assert!(
            links.contains(&"link-area".to_string()),
            "{sel} 应匹配带 href 的 <area>"
        );
        assert!(
            links.contains(&"link-elem".to_string()),
            "{sel} 应匹配带 href 的 <link>"
        );
        assert!(
            !links.contains(&"link-nohref".to_string()),
            "{sel} 不应匹配无 href 的 <a>"
        );
    }

    // :visited → 静态永不匹配（隐私安全，防历史探测）。
    let visited = ids_of(&doc.query_selector_all(root, ":visited"));
    assert!(
        visited.is_empty(),
        ":visited 静态应永不匹配（隐私安全），实际 {visited:?}"
    );

    // :scope → 文档根元素（<html>，等价 :root）。body 内元素不匹配。
    let scope = doc.query_selector_all(root, ":scope");
    assert_eq!(scope.len(), 1, ":scope 应仅匹配文档根元素 <html>，实际匹配 {scope:?}");

    // :lang(en) → 自身或祖先 lang 属性匹配：en-p（lang=en 继承）、en-us-p（lang=en-US，en 前缀匹配）；
    // fr-p（lang=fr）不匹配。:lang(en-US) → 仅 en-us-p（en 不匹配 en-US 范围，前缀语义）。
    let lang_en = ids_of(&doc.query_selector_all(root, ":lang(en)"));
    assert!(lang_en.contains(&"en-p".to_string()), ":lang(en) 应匹配 lang=en 的元素");
    assert!(
        lang_en.contains(&"en-us-p".to_string()),
        ":lang(en) 应匹配 lang=en-US（前缀语义）"
    );
    assert!(!lang_en.contains(&"fr-p".to_string()), ":lang(en) 不应匹配 lang=fr");
    let lang_en_us = ids_of(&doc.query_selector_all(root, ":lang(en-US)"));
    assert!(
        lang_en_us.contains(&"en-us-p".to_string()),
        ":lang(en-US) 应匹配 lang=en-US"
    );
    assert!(
        !lang_en_us.contains(&"en-p".to_string()),
        ":lang(en-US) 不应匹配 lang=en（范围子标签数 > 语言子标签数）"
    );

    // :dir(rtl) → dir 属性沿祖先继承：rtl-p（dir=rtl）、rtl-child（祖先 dir=rtl）匹配；
    // rtl-auto（dir=auto，子树首个强字符为阿拉伯文 RTL）匹配。
    let dir_rtl = ids_of(&doc.query_selector_all(root, ":dir(rtl)"));
    assert!(dir_rtl.contains(&"rtl-p".to_string()), ":dir(rtl) 应匹配 dir=rtl 元素");
    assert!(
        dir_rtl.contains(&"rtl-child".to_string()),
        ":dir(rtl) 应沿祖先继承匹配 dir=rtl 的后代"
    );
    assert!(
        dir_rtl.contains(&"rtl-auto".to_string()),
        ":dir(rtl) 应匹配 dir=auto 且子树首字符为 RTL 的元素"
    );
    assert!(
        !dir_rtl.contains(&"en-p".to_string()),
        ":dir(rtl) 不应匹配 LTR 元素（html dir=ltr 继承）"
    );

    // :dir(ltr) → en-p（html dir=ltr 继承）匹配；rtl-p 不匹配。
    let dir_ltr = ids_of(&doc.query_selector_all(root, ":dir(ltr)"));
    assert!(dir_ltr.contains(&"en-p".to_string()), ":dir(ltr) 应匹配 LTR 元素");
    assert!(
        !dir_ltr.contains(&"rtl-p".to_string()),
        ":dir(ltr) 不应匹配 dir=rtl 元素"
    );
}

#[test]
fn test_query_selector_nth_child_of_s_r3282() {
    // R3282：DOM `:nth-child(an+b of S)` / `:nth-last-child(an+b of S)`——Selectors L4 §16。
    // 此前 DOM `parse_nth` 忽略 `of` 子句 → `querySelectorAll(":nth-child(even of .item)")` 返空。
    // 语义：仅计匹配 S 的元素兄弟中的位置满足 an+b（与 style-system matcher 同源语义）。
    // 父 ul 含 5 个 li，仅 .item 标记的参与计数：c1(.item)=序1, c3(.item)=序2, c5(.item)=序3。
    // 非 .item 的 c2/c4 在 of-S 计数中不存在。
    let doc = parse_html(
        "<html><body>\
         <ul id='ul1'>\
           <li id='c1' class='item'>1</li>\
           <li id='c2'>2</li>\
           <li id='c3' class='item'>3</li>\
           <li id='c4'>4</li>\
           <li id='c5' class='item'>5</li>\
         </ul>\
         </body></html>",
    );
    let root = doc.root();
    let ids_of =
        |sels: &[NodeId]| -> Vec<String> { sels.iter().filter_map(|id| doc.get_attribute(*id, "id")).collect() };

    // :nth-child(odd of .item) → .item 序 1(c1)/3(c5) 奇 → c1/c5；c3 序 2 偶不匹配。
    let odd_of = ids_of(&doc.query_selector_all(root, "li:nth-child(odd of .item)"));
    assert!(
        odd_of.contains(&"c1".to_string()),
        ":nth-child(odd of .item) 应匹配 c1（.item 序 1）"
    );
    assert!(
        odd_of.contains(&"c5".to_string()),
        ":nth-child(odd of .item) 应匹配 c5（.item 序 3）"
    );
    assert!(
        !odd_of.contains(&"c3".to_string()),
        ":nth-child(odd of .item) 不应匹配 c3（.item 序 2 偶）"
    );
    assert!(!odd_of.contains(&"c2".to_string()), "非 .item 不应匹配 of .item");
    assert!(!odd_of.contains(&"c4".to_string()), "非 .item 不应匹配 of .item");

    // :nth-child(even of .item) → .item 序 2(c3) 偶 → 仅 c3。
    let even_of = ids_of(&doc.query_selector_all(root, "li:nth-child(even of .item)"));
    assert_eq!(
        even_of,
        vec!["c3".to_string()],
        ":nth-child(even of .item) 应仅匹配 c3（.item 序 2）"
    );

    // :nth-child(1 of .item) → .item 序首个 = c1。
    let first_of = doc.query_selector(root, "li:nth-child(1 of .item)");
    assert_eq!(
        first_of.and_then(|id| doc.get_attribute(id, "id")),
        Some("c1".to_string()),
        ":nth-child(1 of .item) 应为 c1"
    );

    // :nth-last-child(1 of .item) → .item 序末个 = c5。
    let last_of = doc.query_selector(root, "li:nth-last-child(1 of .item)");
    assert_eq!(
        last_of.and_then(|id| doc.get_attribute(id, "id")),
        Some("c5".to_string()),
        ":nth-last-child(1 of .item) 应为 c5"
    );

    // :nth-last-child(odd of .item) → 倒序 .item 序：c5=1,c3=2,c1=3 → 奇序 c5/c1。
    let last_odd_of = ids_of(&doc.query_selector_all(root, "li:nth-last-child(odd of .item)"));
    assert!(
        last_odd_of.contains(&"c1".to_string()),
        ":nth-last-child(odd of .item) 应匹配 c1（倒序序 3）"
    );
    assert!(
        last_odd_of.contains(&"c5".to_string()),
        ":nth-last-child(odd of .item) 应匹配 c5（倒序序 1）"
    );
    assert!(
        !last_odd_of.contains(&"c3".to_string()),
        ":nth-last-child(odd of .item) 不应匹配 c3（倒序序 2 偶）"
    );

    // 无 `of` 的纯 :nth-child(2) 仍正常（c2，回归保护）。
    let nth2 = doc.query_selector(root, "li:nth-child(2)");
    assert_eq!(
        nth2.and_then(|id| doc.get_attribute(id, "id")),
        Some("c2".to_string()),
        ":nth-child(2) 回归应匹配 c2"
    );

    // `of` 选择器列表（逗号分隔）：:nth-child(1 of .item, #c4) → 计 .item ∪ #c4 的首个 = c1。
    let first_of_list = doc.query_selector(root, "li:nth-child(1 of .item, #c4)");
    assert_eq!(
        first_of_list.and_then(|id| doc.get_attribute(id, "id")),
        Some("c1".to_string()),
        ":nth-child(1 of .item, #c4) 应为 c1（c1 是 .item∪#c4 序首）"
    );

    // `of` 后无选择器（非法）→ 不匹配（返空），不 panic。
    let malformed = doc.query_selector_all(root, "li:nth-child(2 of )");
    assert!(malformed.is_empty(), "非法 `of` 空列表应返空");
}

#[test]
fn test_query_selector_attribute_operators() {
    let doc = parse_html(
        "<html><body>\
         <a href='https://a.example.com/' id='a-https'>A</a>\
         <a href='http://b.example.com/x.pdf' id='b-pdf'>B</a>\
         <a href='/local/page' id='c-local'>C</a>\
         <div class='nav-item active' id='d-active'>D</div>\
         <div class='nav-item' id='d-nav'>D2</div>\
         <p lang='en-US' id='e-en-us'>E</p>\
         <p lang='en' id='e-en'>E2</p>\
         <p lang='fr' id='e-fr'>E3</p>\
         </body></html>",
    );
    let root = doc.root();
    let ids_of = |s: &[NodeId]| -> Vec<String> { s.iter().filter_map(|id| doc.get_attribute(*id, "id")).collect() };
    // ^= 前缀：href^="https" → 仅 https 链接（带引号值去引号后匹配）。
    let prefix = ids_of(&doc.query_selector_all(root, "[href^=\"https\"]"));
    assert!(prefix.contains(&"a-https".to_string()), "https 前缀应匹配");
    assert!(
        !prefix.iter().any(|i| i.starts_with('b') || i.starts_with('c')),
        "非 https 前缀不应匹配"
    );
    // $= 后缀：href$=pdf → .pdf 结尾。
    let suffix = ids_of(&doc.query_selector_all(root, "[href$=pdf]"));
    assert!(suffix.contains(&"b-pdf".to_string()));
    assert!(!suffix.contains(&"a-https".to_string()));
    // *= 子串：class*=active → 含 active 的 class。
    let sub = ids_of(&doc.query_selector_all(root, "[class*=active]"));
    assert!(sub.contains(&"d-active".to_string()));
    assert!(!sub.contains(&"d-nav".to_string()));
    // |= 连字符匹配：lang|=en → lang == en 或以 en- 开头（en-US 命中，fr 不命中）。
    let dash = ids_of(&doc.query_selector_all(root, "[lang|=en]"));
    assert!(dash.contains(&"e-en-us".to_string()), "en-US 应匹配 lang|=en");
    assert!(dash.contains(&"e-en".to_string()), "en 应匹配 lang|=en");
    assert!(!dash.contains(&"e-fr".to_string()), "fr 不应匹配 lang|=en");
    // 组合：div.nav-item[class*=active] → 仅 d-active。
    let combo = ids_of(&doc.query_selector_all(root, "div.nav-item[class*=active]"));
    assert_eq!(combo, vec!["d-active".to_string()]);
    // 仅存在性回归：[href] 匹配所有带 href 的 a。
    assert_eq!(doc.query_selector_all(root, "a[href]").len(), 3);
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

#[test]
fn test_query_selector_target_r3283() {
    // R3283：DOM `:target` 选择器（CSS Selectors L3 §6.6.2：当前文档 URL fragment 指向的唯一元素）。
    // 此前 CSS 解析器识别 `:target` 但 DOM `query.rs` 与 style-system matcher 双双走 `_ => false`
    // → querySelectorAll(":target") 恒空，而同选择器 CSS 侧不匹配——DOM/CSS 不一致。补全为 DOM/CSS
    // 同源（Document::is_target_element 读 url fragment，百分号解码，getElementById 查唯一元素）。
    let mut doc = parse_html(
        "<html><body>\
         <h1 id='top'>Title</h1>\
         <p id='note-1'>first</p>\
         <p id='sec2'>section 2</p>\
         <p>No id here</p>\
         </body></html>",
    );
    let root = doc.root();
    // 自由函数而非闭包——闭包会捕获 `&doc` 延长不可变借用到 set_url（&mut）调用点，致借用冲突。
    let ids_of = |doc: &Document, sels: &[NodeId]| -> Vec<String> {
        sels.iter().filter_map(|id| doc.get_attribute(*id, "id")).collect()
    };

    // 无 URL → 无 fragment → 无 :target。
    let none = doc.query_selector_all(root, ":target");
    assert!(none.is_empty(), "无 URL 时 :target 应无匹配，实际 {none:?}");

    // URL 无 fragment → 无 :target。
    doc.set_url(Some("https://example.com/page".to_string()));
    let none = doc.query_selector_all(root, ":target");
    assert!(none.is_empty(), "URL 无 fragment 时 :target 应无匹配，实际 {none:?}");

    // URL fragment=#top → 仅匹配 id=top 的元素。
    doc.set_url(Some("https://example.com/page#top".to_string()));
    let target = ids_of(&doc, &doc.query_selector_all(root, ":target"));
    assert_eq!(target, vec!["top".to_string()], "#top fragment 应仅命中 id=top");

    // URL fragment=#sec2 → 切换到 id=sec2。
    doc.set_url(Some("https://example.com/page#sec2".to_string()));
    let target = ids_of(&doc, &doc.query_selector_all(root, ":target"));
    assert_eq!(target, vec!["sec2".to_string()], "#sec2 fragment 应仅命中 id=sec2");

    // fragment 指向不存在的 id → 无 :target。
    doc.set_url(Some("https://example.com/page#missing".to_string()));
    let target = ids_of(&doc, &doc.query_selector_all(root, ":target"));
    assert!(target.is_empty(), "不存在的 fragment 应无 :target 匹配");

    // 百分号编码 fragment：#note-1 编码为 #note-%31（%31 = '1'），解码后 = "note-1" 命中。
    doc.set_url(Some("https://example.com/page#note-%31".to_string()));
    let target = ids_of(&doc, &doc.query_selector_all(root, ":target"));
    assert_eq!(
        target,
        vec!["note-1".to_string()],
        "百分号编码 fragment #note-%31 应解码为 note-1 命中"
    );

    // 空 fragment（page#）→ 无 :target。
    doc.set_url(Some("https://example.com/page#".to_string()));
    let target = ids_of(&doc, &doc.query_selector_all(root, ":target"));
    assert!(target.is_empty(), "空 fragment 应无 :target 匹配");

    // 直接调权威方法（DOM/CSS 共享，style-system matcher 同此源）。
    doc.set_url(Some("https://example.com/page#top".to_string()));
    let top = doc.get_element_by_id("top").unwrap();
    assert!(
        doc.is_target_element(top),
        "is_target_element 对 #top 指向的元素应返 true"
    );
    let note1 = doc.get_element_by_id("note-1").unwrap();
    assert!(
        !doc.is_target_element(note1),
        "is_target_element 对非目标元素应返 false"
    );
}
