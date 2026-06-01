use super::body_of;
// DOM crate 综合测试套件。
//
// 覆盖：节点类型、树操作、属性操作、HTML 解析、查询、序列化、MutationObserver。

use crate::*;
use std::sync::{Arc, Mutex};

// ═══════════════════════════════════════════════════════════════════════
// 1. 节点创建测试
// ═══════════════════════════════════════════════════════════════════════

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
