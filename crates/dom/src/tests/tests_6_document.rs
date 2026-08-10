// DOM Document 模块测试。
//
// 覆盖：ProcessingInstruction、text_content、属性操作、quirks mode、
//       normalize、import_node、get_elements_by_tag_name_ns、TreeWalker、NodeIterator。

use crate::*;

// ═══════════════════════════════════════════════════════════════════════
// 基础功能测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 ProcessingInstruction 内容可正确读取。
#[test]
fn test_processing_instruction_content() {
    let mut doc = Document::new();
    let pi = doc.create_processing_instruction("xml-stylesheet", "href=\"style.css\"");
    assert_eq!(doc.node_type(pi), Some(7));
    // 验证 text_content 返回 data 字段
    assert_eq!(doc.text_content(pi), Some("href=\"style.css\"".to_string()));
}

/// 测试 set_text_content 在 Comment 节点上。
#[test]
fn test_set_text_content_on_comment() {
    let mut doc = Document::new();
    let comment = doc.create_comment("original");
    assert_eq!(doc.text_content(comment), Some("original".to_string()));
    doc.set_text_content(comment, "updated");
    assert_eq!(doc.text_content(comment), Some("updated".to_string()));
}

/// 测试 set_text_content 在 DocumentFragment 上（应替换子节点为文本）。
#[test]
fn test_set_text_content_on_fragment() {
    let mut doc = Document::new();
    let frag = doc.create_document_fragment();
    let child = doc.create_element("div");
    doc.append_child(frag, child).unwrap();
    assert!(doc.has_child_nodes(frag));

    doc.set_text_content(frag, "new text");
    assert_eq!(doc.text_content(frag), Some("new text".to_string()));
    // 子元素应被替换为文本节点
    assert_eq!(doc.child_count(frag), 1);
}

/// 测试 text_content 在 DocumentType 上返回 None。
#[test]
fn test_text_content_doctype_returns_none() {
    let mut doc = Document::new();
    let dt = doc.create_document_type("html", None, None);
    assert_eq!(doc.text_content(dt), None);
}

/// 测试 get_elements_by_class_name 支持多 class 元素。
#[test]
fn test_get_elements_by_class_name_multi_class() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "foo bar baz");
    doc.append_child(doc.root(), elem).unwrap();

    // 搜索任意一个 class 都应匹配
    let results = doc.get_elements_by_class_name("bar");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], elem);
}

/// 测试 set_attribute id 为空字符串不加入 id_map。
#[test]
fn test_set_attribute_empty_id_not_in_id_map() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "id", "");
    doc.append_child(doc.root(), elem).unwrap();

    // 通过非空 id 查找应返回 None
    let found = doc.get_element_by_id("nonexistent");
    assert!(found.is_none(), "nonexistent id should not be in id_map");
}

/// 测试 compare_document_position 同一节点。
#[test]
fn test_compare_document_position_same_node() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let pos = doc.compare_document_position(elem, elem);
    assert!(pos.is_some(), "same node should return Some");
    // 同一节点的位标志应为 0
    assert_eq!(pos.unwrap().bits(), 0, "same node should have zero position bits");
}

/// 测试 quirks_mode 设置和获取。
#[test]
fn test_quirks_mode_set_get() {
    let mut doc = Document::new();
    assert_eq!(doc.quirks_mode(), QuirksMode::NoQuirks);
    doc.set_quirks_mode(QuirksMode::Quirks);
    assert_eq!(doc.quirks_mode(), QuirksMode::Quirks);
    doc.set_quirks_mode(QuirksMode::LimitedQuirks);
    assert_eq!(doc.quirks_mode(), QuirksMode::LimitedQuirks);
}

/// 测试 document URL 注入/读取（R3169，导航层 → Document，`document.URL`/`documentURI` 读）。
#[test]
fn test_document_url_set_get() {
    let mut doc = Document::new();
    // 默认未注入 → None。
    assert_eq!(doc.url(), None);
    // 注入页面 URL → 读回。
    doc.set_url(Some("https://example.com/page".to_string()));
    assert_eq!(doc.url(), Some("https://example.com/page"));
    // 清除。
    doc.set_url(None);
    assert_eq!(doc.url(), None);
}

/// 测试 document referrer 注入/读取（R3176，导航层 → Document，`document.referrer` 读，
/// 来源页 URL = 导航前的页面地址）。
#[test]
fn test_document_referrer_set_get() {
    let mut doc = Document::new();
    // 默认未注入 → None（直接打开页面无来源）。
    assert_eq!(doc.referrer(), None);
    // 注入来源页 URL → 读回。
    doc.set_referrer(Some("https://ref.example.com/prev".to_string()));
    assert_eq!(doc.referrer(), Some("https://ref.example.com/prev"));
    // 清除。
    doc.set_referrer(None);
    assert_eq!(doc.referrer(), None);
}

/// 测试 remove_attribute 在非元素节点上不 panic。
#[test]
fn test_remove_attribute_on_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    doc.remove_attribute(text, "id");
}

/// 测试 attribute_names 在非元素节点返回空。
#[test]
fn test_attribute_names_on_text_node() {
    let mut doc = Document::new();
    let text = doc.create_text_node("hello");
    assert!(doc.attribute_names(text).is_empty());
}

// ═══════════════════════════════════════════════════════════════════════
// normalize 测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试合并两个相邻文本节点。
#[test]
fn test_normalize_merges_adjacent_text_nodes() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let t1 = doc.create_text_node("hello");
    let t2 = doc.create_text_node(" ");
    let t3 = doc.create_text_node("world");

    doc.append_child(parent, t1).unwrap();
    doc.append_child(parent, t2).unwrap();
    doc.append_child(parent, t3).unwrap();

    // 规格化前：3 个子节点
    assert_eq!(doc.child_count(parent), 3);

    doc.normalize(parent);

    // 规格化后：合并为 1 个文本节点，内容为 "hello world"
    assert_eq!(doc.child_count(parent), 1);
    assert_eq!(doc.text_content(parent), Some("hello world".to_string()));
}

/// 测试移除元素之间的空文本节点。
#[test]
fn test_normalize_removes_empty_text_nodes() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let span1 = doc.create_element("span");
    let empty_text = doc.create_text_node("");
    let span2 = doc.create_element("span");

    doc.append_child(parent, span1).unwrap();
    doc.append_child(parent, empty_text).unwrap();
    doc.append_child(parent, span2).unwrap();

    // 规格化前：3 个子节点
    assert_eq!(doc.child_count(parent), 3);

    doc.normalize(parent);

    // 空文本节点被移除，只剩 2 个元素
    assert_eq!(doc.child_count(parent), 2);
    let children = doc.child_nodes(parent);
    assert_eq!(doc.node_type(children[0]), Some(1)); // Element
    assert_eq!(doc.node_type(children[1]), Some(1)); // Element
}

/// 测试规格化包含混合内容的子树（相邻文本 + 空文本 + 嵌套元素）。
#[test]
fn test_normalize_mixed_subtree() {
    let mut doc = Document::new();
    let root = doc.create_element("div");

    // root 的子节点：text("a") + text("b") + empty_text + span + text("c") + text("d")
    let t1 = doc.create_text_node("a");
    let t2 = doc.create_text_node("b");
    let empty = doc.create_text_node("");
    let span = doc.create_element("span");
    let t3 = doc.create_text_node("c");
    let t4 = doc.create_text_node("d");

    doc.append_child(root, t1).unwrap();
    doc.append_child(root, t2).unwrap();
    doc.append_child(root, empty).unwrap();
    doc.append_child(root, span).unwrap();
    doc.append_child(root, t3).unwrap();
    doc.append_child(root, t4).unwrap();

    // span 内部也有相邻文本
    let s1 = doc.create_text_node("x");
    let s2 = doc.create_text_node("y");
    doc.append_child(span, s1).unwrap();
    doc.append_child(span, s2).unwrap();

    doc.normalize(root);

    // root 层：text("ab") + span + text("cd") = 3 个子节点
    assert_eq!(doc.child_count(root), 3);
    let children = doc.child_nodes(root);
    assert_eq!(doc.text_content(children[0]), Some("ab".to_string()));
    assert_eq!(doc.node_type(children[1]), Some(1)); // span 元素
    assert_eq!(doc.text_content(children[2]), Some("cd".to_string()));

    // span 内部：text("xy") = 1 个子节点
    assert_eq!(doc.child_count(children[1]), 1);
    assert_eq!(doc.text_content(children[1]), Some("xy".to_string()));
}

/// 测试对已规格化的树调用 normalize 无变化。
#[test]
fn test_normalize_already_normalized_is_noop() {
    let mut doc = Document::new();
    let parent = doc.create_element("p");
    let t1 = doc.create_text_node("hello");
    let elem = doc.create_element("br");
    let t2 = doc.create_text_node("world");

    doc.append_child(parent, t1).unwrap();
    doc.append_child(parent, elem).unwrap();
    doc.append_child(parent, t2).unwrap();

    // 已经是规格化的状态
    assert_eq!(doc.child_count(parent), 3);

    doc.normalize(parent);

    // 无变化
    assert_eq!(doc.child_count(parent), 3);
    assert_eq!(doc.text_content(parent), Some("helloworld".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// import_node 测试
// ═══════════════════════════════════════════════════════════════════════

/// 浅导入元素节点，不应包含子节点。
#[test]
fn test_import_node_shallow() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    let child = doc.create_element("span");
    doc.append_child(parent, child).unwrap();
    doc.set_attribute(parent, "class", "outer");

    let imported = doc.import_node(parent, false);

    // 导入节点保留属性但无子节点
    assert_eq!(doc.get_attribute(imported, "class"), Some("outer".to_string()));
    assert!(!doc.has_child_nodes(imported));
}

/// 深导入保留完整子树。
#[test]
fn test_import_node_deep_preserves_subtree() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    let text = doc.create_text_node("hello");
    doc.append_child(child1, text).unwrap();
    doc.append_child(root, child1).unwrap();
    doc.append_child(root, child2).unwrap();

    let imported = doc.import_node(root, true);

    // 导入节点应包含完整的子树结构
    assert_eq!(doc.child_count(imported), 2);
    let children = doc.child_nodes(imported);
    let imported_p = children[0];
    let imported_span = children[1];
    assert_eq!(doc.child_count(imported_p), 1);
    assert_eq!(doc.text_content(imported_p), Some("hello".to_string()));
    assert_eq!(doc.child_count(imported_span), 0);
}

/// 导入的节点没有父节点（独立于原文档树）。
#[test]
fn test_import_node_has_no_parent() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let imported = doc.import_node(elem, true);

    // 导入节点没有父节点
    assert!(doc.parent_node(imported).is_none());
}

/// 导入的节点是独立的，修改原始节点不影响导入副本。
#[test]
fn test_import_node_is_independent() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "original");
    let child = doc.create_text_node("text");
    doc.append_child(elem, child).unwrap();

    let imported = doc.import_node(elem, true);

    // 修改原始节点
    doc.set_attribute(elem, "class", "modified");

    // 导入副本不受影响
    assert_eq!(doc.get_attribute(imported, "class"), Some("original".to_string()));
    assert_eq!(doc.text_content(imported), Some("text".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════
// get_elements_by_tag_name_ns 测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试按命名空间和标签名查找元素，应匹配正确命名空间中的元素。
#[test]
fn test_get_elements_by_tag_name_ns_finds_correct_namespace() {
    use markup5ever::{LocalName, Namespace, QualName};

    let mut doc = Document::new();
    let container = doc.create_element("div");

    // 创建 XHTML 命名空间的 div
    let xhtml_div = doc.create_element_with_qname(
        QualName::new(
            None,
            Namespace::from("http://www.w3.org/1999/xhtml"),
            LocalName::from("div"),
        ),
        vec![],
    );
    // 创建 SVG 命名空间的 rect
    let svg_rect = doc.create_element_with_qname(
        QualName::new(
            None,
            Namespace::from("http://www.w3.org/2000/svg"),
            LocalName::from("rect"),
        ),
        vec![],
    );

    doc.append_child(container, xhtml_div).unwrap();
    doc.append_child(container, svg_rect).unwrap();
    doc.append_child(doc.root(), container).unwrap();

    // 按 SVG 命名空间查找 rect
    let results = doc.get_elements_by_tag_name_ns(Some("http://www.w3.org/2000/svg"), "rect");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], svg_rect);

    // 按 XHTML 命名空间查找 div（只匹配 XHTML 命名空间的）
    let results = doc.get_elements_by_tag_name_ns(Some("http://www.w3.org/1999/xhtml"), "div");
    assert_eq!(results.len(), 2); // container + xhtml_div
}

/// 测试按命名空间查找时忽略错误命名空间中的元素。
#[test]
fn test_get_elements_by_tag_name_ns_ignores_wrong_namespace() {
    use markup5ever::{LocalName, Namespace, QualName};

    let mut doc = Document::new();

    // 创建 SVG 命名空间的 div
    let svg_div = doc.create_element_with_qname(
        QualName::new(
            None,
            Namespace::from("http://www.w3.org/2000/svg"),
            LocalName::from("div"),
        ),
        vec![],
    );
    // 创建 XHTML 命名空间的 div
    let xhtml_div = doc.create_element("div");

    doc.append_child(doc.root(), svg_div).unwrap();
    doc.append_child(doc.root(), xhtml_div).unwrap();

    // 按 SVG 命名空间查找 div — 不应匹配 XHTML 的 div
    let results = doc.get_elements_by_tag_name_ns(Some("http://www.w3.org/2000/svg"), "div");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], svg_div);

    // 按 XHTML 命名空间查找 div — 不应匹配 SVG 的 div
    let results = doc.get_elements_by_tag_name_ns(Some("http://www.w3.org/1999/xhtml"), "div");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0], xhtml_div);
}

/// 测试通配命名空间（None）匹配所有命名空间中指定标签名的元素。
#[test]
fn test_get_elements_by_tag_name_ns_wildcard_namespace() {
    use markup5ever::{LocalName, Namespace, QualName};

    let mut doc = Document::new();

    let svg_rect = doc.create_element_with_qname(
        QualName::new(
            None,
            Namespace::from("http://www.w3.org/2000/svg"),
            LocalName::from("rect"),
        ),
        vec![],
    );
    let xhtml_rect = doc.create_element_with_qname(
        QualName::new(
            None,
            Namespace::from("http://www.w3.org/1999/xhtml"),
            LocalName::from("rect"),
        ),
        vec![],
    );

    doc.append_child(doc.root(), svg_rect).unwrap();
    doc.append_child(doc.root(), xhtml_rect).unwrap();

    // namespace=None 应匹配所有命名空间的 rect
    let results = doc.get_elements_by_tag_name_ns(None, "rect");
    assert_eq!(results.len(), 2);
}

/// 测试通配标签名（"*"）匹配指定命名空间中的所有元素。
#[test]
fn test_get_elements_by_tag_name_ns_wildcard_local_name() {
    use markup5ever::{LocalName, Namespace, QualName};

    let mut doc = Document::new();
    let container = doc.create_element("div");

    let svg_rect = doc.create_element_with_qname(
        QualName::new(
            None,
            Namespace::from("http://www.w3.org/2000/svg"),
            LocalName::from("rect"),
        ),
        vec![],
    );
    let svg_circle = doc.create_element_with_qname(
        QualName::new(
            None,
            Namespace::from("http://www.w3.org/2000/svg"),
            LocalName::from("circle"),
        ),
        vec![],
    );
    let xhtml_span = doc.create_element("span");

    doc.append_child(container, svg_rect).unwrap();
    doc.append_child(container, svg_circle).unwrap();
    doc.append_child(container, xhtml_span).unwrap();
    doc.append_child(doc.root(), container).unwrap();

    // "*" 在 SVG 命名空间中应只匹配 SVG 元素
    let results = doc.get_elements_by_tag_name_ns(Some("http://www.w3.org/2000/svg"), "*");
    assert_eq!(results.len(), 2);
    assert!(results.contains(&svg_rect));
    assert!(results.contains(&svg_circle));
}

/// 测试双重通配（None 命名空间 + "*" 标签名）返回文档中所有元素。
#[test]
fn test_get_elements_by_tag_name_ns_double_wildcard() {
    use markup5ever::{LocalName, Namespace, QualName};

    let mut doc = Document::new();
    let container = doc.create_element("div");

    let svg_rect = doc.create_element_with_qname(
        QualName::new(
            None,
            Namespace::from("http://www.w3.org/2000/svg"),
            LocalName::from("rect"),
        ),
        vec![],
    );
    let xhtml_span = doc.create_element("span");

    doc.append_child(container, svg_rect).unwrap();
    doc.append_child(container, xhtml_span).unwrap();
    doc.append_child(doc.root(), container).unwrap();

    // 双重通配应返回所有元素
    let results = doc.get_elements_by_tag_name_ns(None, "*");
    // container + svg_rect + xhtml_span = 3
    assert!(results.len() >= 3);
    assert!(results.contains(&container));
    assert!(results.contains(&svg_rect));
    assert!(results.contains(&xhtml_span));
}

// ═══════════════════════════════════════════════════════════════════════
// TreeWalker 测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 TreeWalker 遍历 3 层树的所有节点。
#[test]
fn test_tree_walker_traverse_all() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    let text = doc.create_text_node("hello");
    doc.append_child(doc.root(), root).unwrap();
    doc.append_child(root, span).unwrap();
    doc.append_child(root, p).unwrap();
    doc.append_child(span, text).unwrap();

    let mut walker = TreeWalker::new(root, 0xFFFFFFFF);
    assert_eq!(walker.current_node(), root);

    let mut visited = vec![walker.current_node()];
    while let Some(node) = walker.next_node(&doc) {
        visited.push(node);
    }

    // root, span, text, p = 4 个节点
    assert_eq!(visited.len(), 4);
    assert_eq!(visited[0], root);
    assert_eq!(visited[1], span);
    assert_eq!(visited[2], text);
    assert_eq!(visited[3], p);
}

/// 测试 TreeWalker first_child 返回正确的子节点。
#[test]
fn test_tree_walker_first_child() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, span).unwrap();
    doc.append_child(root, p).unwrap();

    let mut walker = TreeWalker::new(root, 0xFFFFFFFF);
    assert_eq!(walker.current_node(), root);

    let child = walker.first_child(&doc);
    assert_eq!(child, Some(span));
    assert_eq!(walker.current_node(), span);
}

/// 测试 TreeWalker next_sibling 在兄弟节点间正确移动。
#[test]
fn test_tree_walker_next_sibling() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    doc.append_child(root, span).unwrap();
    doc.append_child(root, p).unwrap();

    let mut walker = TreeWalker::new(root, 0xFFFFFFFF);
    // 先移到第一个子节点
    walker.first_child(&doc);
    assert_eq!(walker.current_node(), span);

    // 移到下一个兄弟
    let sibling = walker.next_sibling(&doc);
    assert_eq!(sibling, Some(p));
    assert_eq!(walker.current_node(), p);

    // p 没有下一个兄弟
    assert_eq!(walker.next_sibling(&doc), None);
    assert_eq!(walker.current_node(), p);
}

/// 测试 TreeWalker 在单节点（无子节点）时 next_node 返回 None。
#[test]
fn test_tree_walker_empty_tree() {
    let mut doc = Document::new();
    let sole = doc.create_element("div");

    let mut walker = TreeWalker::new(sole, 0xFFFFFFFF);
    assert_eq!(walker.current_node(), sole);

    // 没有子节点，没有兄弟，没有父节点 → next_node 应返回 None
    assert_eq!(walker.next_node(&doc), None);
    assert_eq!(walker.current_node(), sole);
}

/// 测试 TreeWalker current_node 在每步遍历后返回正确的节点。
#[test]
fn test_tree_walker_current_node() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let a = doc.create_element("a");
    let b = doc.create_text_node("text");
    let c = doc.create_element("span");
    doc.append_child(root, a).unwrap();
    doc.append_child(a, b).unwrap();
    doc.append_child(root, c).unwrap();

    let mut walker = TreeWalker::new(root, 0xFFFFFFFF);

    // 初始：root
    assert_eq!(walker.current_node(), root);

    // next_node → a（root 的第一个子节点）
    walker.next_node(&doc);
    assert_eq!(walker.current_node(), a);

    // next_node → b（a 的第一个子节点）
    walker.next_node(&doc);
    assert_eq!(walker.current_node(), b);

    // next_node → c（b 无子无兄弟，回溯到 a 再到 root，root 的下一个兄弟是 c）
    walker.next_node(&doc);
    assert_eq!(walker.current_node(), c);

    // next_node → None（c 无子节点，回溯到 root 后无更多兄弟）
    assert_eq!(walker.next_node(&doc), None);
    assert_eq!(walker.current_node(), c);
}

// ═══════════════════════════════════════════════════════════════════════
// NodeIterator 测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 NodeIterator 遍历所有节点，计数正确。
#[test]
fn test_node_iterator_traverse() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let span = doc.create_element("span");
    let p = doc.create_element("p");
    let text = doc.create_text_node("hello");
    doc.append_child(doc.root(), root).unwrap();
    doc.append_child(root, span).unwrap();
    doc.append_child(root, p).unwrap();
    doc.append_child(span, text).unwrap();

    let mut iter = NodeIterator::new(root, 0xFFFFFFFF);
    assert_eq!(iter.current_node(), root);

    let mut visited = vec![iter.current_node()];
    while let Some(node) = iter.next_node(&doc) {
        visited.push(node);
    }

    // root, span, text, p = 4 个节点
    assert_eq!(visited.len(), 4);
    assert_eq!(visited[0], root);
    assert_eq!(visited[1], span);
    assert_eq!(visited[2], text);
    assert_eq!(visited[3], p);
    assert!(iter.is_done());
}

/// 测试 NodeIterator 向前再向后遍历，验证位置正确。
#[test]
fn test_node_iterator_next_previous() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let a = doc.create_element("a");
    let b = doc.create_text_node("text");
    let c = doc.create_element("span");
    doc.append_child(root, a).unwrap();
    doc.append_child(a, b).unwrap();
    doc.append_child(root, c).unwrap();

    let mut iter = NodeIterator::new(root, 0xFFFFFFFF);

    // next: root → a
    let n = iter.next_node(&doc);
    assert_eq!(n, Some(a));
    assert_eq!(iter.current_node(), a);

    // next: a → b
    let n = iter.next_node(&doc);
    assert_eq!(n, Some(b));
    assert_eq!(iter.current_node(), b);

    // next: b → c
    let n = iter.next_node(&doc);
    assert_eq!(n, Some(c));
    assert_eq!(iter.current_node(), c);

    // previous: c → b
    let p = iter.previous_node(&doc);
    assert_eq!(p, Some(b));
    assert_eq!(iter.current_node(), b);

    // previous: b → a
    let p = iter.previous_node(&doc);
    assert_eq!(p, Some(a));
    assert_eq!(iter.current_node(), a);

    // previous: a → 已在 root 下，无法继续
    let p = iter.previous_node(&doc);
    assert_eq!(p, None);
}

/// 测试 NodeIterator 在单节点（无子节点）时 next 返回 None。
#[test]
fn test_node_iterator_single_node() {
    let mut doc = Document::new();
    let sole = doc.create_element("div");

    let mut iter = NodeIterator::new(sole, 0xFFFFFFFF);
    assert_eq!(iter.current_node(), sole);

    // 没有子节点 → next_node 应返回 None
    assert_eq!(iter.next_node(&doc), None);
    assert_eq!(iter.current_node(), sole);
    assert!(iter.is_done());
}

/// 测试 NodeIterator current_node 在每步遍历后返回正确的节点。
#[test]
fn test_node_iterator_current_node() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let a = doc.create_element("a");
    let b = doc.create_text_node("text");
    let c = doc.create_element("span");
    doc.append_child(root, a).unwrap();
    doc.append_child(a, b).unwrap();
    doc.append_child(root, c).unwrap();

    let mut iter = NodeIterator::new(root, 0xFFFFFFFF);

    // 初始：root
    assert_eq!(iter.current_node(), root);

    // next → a
    iter.next_node(&doc);
    assert_eq!(iter.current_node(), a);

    // next → b
    iter.next_node(&doc);
    assert_eq!(iter.current_node(), b);

    // next → c
    iter.next_node(&doc);
    assert_eq!(iter.current_node(), c);

    // next → None，current 停留在 c
    assert_eq!(iter.next_node(&doc), None);
    assert_eq!(iter.current_node(), c);
}

/// 测试 NodeIterator 对无子节点的根节点遍历。
#[test]
fn test_node_iterator_empty_children() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    // parent 没有子节点

    let mut iter = NodeIterator::new(parent, 0xFFFFFFFF);
    assert_eq!(iter.current_node(), parent);
    assert!(!iter.is_done());

    // next_node 返回 None（无子节点）
    assert_eq!(iter.next_node(&doc), None);
    assert!(iter.is_done());

    // 再次调用仍然返回 None
    assert_eq!(iter.next_node(&doc), None);
}
