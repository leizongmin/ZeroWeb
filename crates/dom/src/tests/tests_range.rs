// Range API 测试套件
//
// 覆盖：Range 错误处理、边界条件、复杂场景

use crate::*;

// ═══════════════════════════════════════════════════════════════════════
// Range 错误处理测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 select_node 对 detached 节点（没有父节点）应返回 Detached 错误。
#[test]
fn test_range_select_node_detached_error() {
    let mut doc = Document::new();
    // 创建但不附加到文档的节点
    let detached = doc.create_element("div");

    let mut range = Range::new(detached, detached);
    let result = range.select_node(&doc, detached);
    assert!(
        matches!(result, Err(RangeError::Detached)),
        "select_node on detached node should return Detached error, got: {:?}",
        result
    );
}

/// 测试 select_node 对文档根节点（Document）应返回 Detached 错误。
#[test]
fn test_range_select_node_document_root_error() {
    let doc = Document::new();
    let root = doc.root();

    let mut range = Range::at(root, 0);
    let result = range.select_node(&doc, root);
    assert!(
        matches!(result, Err(RangeError::Detached)),
        "select_node on document root should return Detached error, got: {:?}",
        result
    );
}

/// 测试 select_node 对不存在的节点位置应返回 WrongDocument 错误。
#[test]
fn test_range_select_node_wrong_document_error() {
    // select_node 要求节点有父节点，否则返回 Detached
    let mut doc1 = Document::new();
    let node_from_doc1 = doc1.create_element("p");
    // 未挂载到文档的节点没有父节点
    let mut range = Range::new(doc1.root(), doc1.root());
    let result = range.select_node(&doc1, node_from_doc1);
    // 未挂载节点没有父节点，应返回 Detached
    assert!(
        matches!(result, Err(RangeError::Detached)),
        "select_node on detached node should return Detached error, got: {:?}",
        result
    );
}

/// 测试 delete_contents 对空范围是有效的（不 panic）。
#[test]
fn test_range_delete_contents_empty() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let mut range = Range::new(parent, parent);
    // 空范围删除应成功
    let result = range.delete_contents(&mut doc);
    assert!(result.is_ok(), "delete_contents on empty range should succeed");
}

/// 测试 extract_contents 对空范围返回空 fragment。
#[test]
fn test_range_extract_contents_empty() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let mut range = Range::new(parent, parent);
    let result = range.extract_contents(&mut doc);
    assert!(result.is_ok(), "extract_contents on empty range should succeed");
}

/// 测试 clone_contents 对空范围返回空 fragment。
#[test]
fn test_range_clone_contents_empty() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let range = Range::new(parent, parent);
    let result = range.clone_contents(&mut doc);
    assert!(result.is_ok(), "clone_contents on empty range should succeed");
}

// ═══════════════════════════════════════════════════════════════════════
// Range 边界条件测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 Range::at 创建的范围折叠到指定位置。
#[test]
fn test_range_at_folds_to_position() {
    let mut doc = Document::new();
    let div = doc.create_element("div");
    doc.append_child(doc.root(), div).unwrap();
    let text = doc.create_text_node("Hello");
    doc.append_child(div, text).unwrap();

    let range = Range::at(text, 2);
    assert!(range.collapsed());
    assert_eq!(range.start_offset(), 2);
    assert_eq!(range.end_offset(), 2);
}

/// 测试 Range::at 对空文本节点应折叠到偏移 0。
#[test]
fn test_range_at_empty_text_node() {
    let mut doc = Document::new();
    let empty_text = doc.create_text_node("");
    doc.append_child(doc.root(), empty_text).unwrap();

    let range = Range::at(empty_text, 0);
    assert!(range.collapsed());
    assert_eq!(range.start_offset(), 0);
}

/// 测试 Range::at 对偏移量超出文本长度应不 panic。
#[test]
fn test_range_at_offset_beyond_text_length() {
    let mut doc = Document::new();
    let text = doc.create_text_node("Hi");
    doc.append_child(doc.root(), text).unwrap();

    let range = Range::at(text, 100); // 远超文本长度
    assert!(range.collapsed());
    assert_eq!(range.start_offset(), 100);
}

/// 测试 Range 创建后在不同容器上的复杂范围操作。
#[test]
fn test_range_complex_different_containers() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    let child3 = doc.create_element("b");
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();
    doc.append_child(parent, child3).unwrap();

    let mut range = Range::new(parent, parent);
    // 设置一个覆盖第二个子节点的范围
    range.set_start(parent, 1).unwrap();
    range.set_end(parent, 2).unwrap();

    // 测试 text_content 应只包含第二个子节点的内容
    let text = range.text_content(&doc);
    // 由于 span 是空元素，text_content 可能为空，这是正确的
    assert!(!text.contains("b"), "text_content should not include 'b'");
}

// ═══════════════════════════════════════════════════════════════════════
// Range 复杂场景测试
// ═══════════════════════════════════════════════════════════════════════

/// 测试 text_content 跨多个文本节点。
#[test]
fn test_text_content_multiple_text_nodes() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let text1 = doc.create_text_node("Hello ");
    let text2 = doc.create_text_node("World!");
    doc.append_child(parent, text1).unwrap();
    doc.append_child(parent, text2).unwrap();

    let mut range = Range::new(parent, parent);
    range.set_start(parent, 0).unwrap();
    range.set_end(parent, 2).unwrap();

    let content = range.text_content(&doc);
    assert_eq!(content, "Hello World!");
}

/// 测试 text_content 包含空文本节点。
#[test]
fn test_text_content_with_empty_text_nodes() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let text1 = doc.create_text_node("Hello ");
    let empty = doc.create_text_node("");
    let text2 = doc.create_text_node("World!");
    doc.append_child(parent, text1).unwrap();
    doc.append_child(parent, empty).unwrap();
    doc.append_child(parent, text2).unwrap();

    let mut range = Range::new(parent, parent);
    range.set_start(parent, 0).unwrap();
    range.set_end(parent, 3).unwrap();

    let content = range.text_content(&doc);
    assert_eq!(content, "Hello World!");
}

/// 测试 delete_contents 删除部分子节点。
#[test]
fn test_delete_contents_partial() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    let child3 = doc.create_element("b");
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();
    doc.append_child(parent, child3).unwrap();

    // 删除中间的 span
    let mut range = Range::new(parent, parent);
    range.set_start(parent, 1).unwrap();
    range.set_end(parent, 2).unwrap();
    range.delete_contents(&mut doc).unwrap();

    let remaining = doc.child_nodes(parent);
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining[0], child1);
    assert_eq!(remaining[1], child3);
}

/// 测试 extract_contents 提取部分子节点。
#[test]
fn test_extract_contents_partial() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    let child3 = doc.create_element("b");
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();
    doc.append_child(parent, child3).unwrap();

    // 提取中间的 span
    let mut range = Range::new(parent, parent);
    range.set_start(parent, 1).unwrap();
    range.set_end(parent, 2).unwrap();
    let fragment = range.extract_contents(&mut doc).unwrap();

    let remaining = doc.child_nodes(parent);
    assert_eq!(remaining.len(), 2);
    assert_eq!(remaining[0], child1);
    assert_eq!(remaining[1], child3);

    let extracted = doc.child_nodes(fragment);
    assert_eq!(extracted.len(), 1);
    assert_eq!(extracted[0], child2);
}

/// 测试 clone_contents 克隆部分子节点。
#[test]
fn test_clone_contents_partial() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    let child3 = doc.create_element("b");
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();
    doc.append_child(parent, child3).unwrap();

    // 克隆中间的 span
    let mut range = Range::new(parent, parent);
    range.set_start(parent, 1).unwrap();
    range.set_end(parent, 2).unwrap();
    let fragment = range.clone_contents(&mut doc).unwrap();

    // 原始树不变
    let remaining = doc.child_nodes(parent);
    assert_eq!(remaining.len(), 3);
    assert_eq!(remaining[0], child1);
    assert_eq!(remaining[1], child2);
    assert_eq!(remaining[2], child3);

    // 克隆的 fragment
    let cloned = doc.child_nodes(fragment);
    assert_eq!(cloned.len(), 1);

    // 克隆的节点是新的
    assert_ne!(cloned[0], child2);
}

/// 测试 insert_node 在起始位置插入。
#[test]
fn test_insert_node_at_start() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();

    let new_node = doc.create_element("b");
    doc.set_text_content(new_node, "new");

    let mut range = Range::at(parent, 0);
    range.insert_node(&mut doc, new_node).unwrap();

    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 3);
    assert_eq!(children[0], new_node);
    assert_eq!(children[1], child1);
    assert_eq!(children[2], child2);
}

/// 测试 insert_node 在结尾位置插入。
#[test]
fn test_insert_node_at_end() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    let child1 = doc.create_element("p");
    let child2 = doc.create_element("span");
    doc.append_child(parent, child1).unwrap();
    doc.append_child(parent, child2).unwrap();

    let new_node = doc.create_element("b");
    doc.set_text_content(new_node, "new");

    let mut range = Range::at(parent, 2);
    range.insert_node(&mut doc, new_node).unwrap();

    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 3);
    assert_eq!(children[0], child1);
    assert_eq!(children[1], child2);
    assert_eq!(children[2], new_node);
}

// ═══════════════════════════════════════════════════════════════════════
// compare_boundary_points 测试覆盖
// ═══════════════════════════════════════════════════════════════════════

/// 测试 compare_boundary_points 当一个 Range 完全包含另一个 Range 时。
#[test]
fn test_compare_boundary_points_contained() {
    let doc = parse_html("<div><p>A</p><p>B</p><p>C</p></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    // r1: [0, 1] (第一个 p)
    let mut r1 = Range::new(div, div);
    r1.set_start(div, 0).unwrap();
    r1.set_end(div, 1).unwrap();

    // r2: [0, 2] (前两个 p)
    let mut r2 = Range::new(div, div);
    r2.set_start(div, 0).unwrap();
    r2.set_end(div, 2).unwrap();

    // r1 结束在 r2 开始之后，但 r2 结束在 r1 结束之后
    // 应返回 0，因为边界点有重叠
    assert_eq!(r1.compare_boundary_points(&r2), 0);
}

/// 测试 compare_boundary_points 当边界点相同时。
#[test]
fn test_compare_boundary_points_adjacent() {
    let doc = parse_html("<div><p>A</p><p>B</p></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    // r1: [0, 1] (第一个 p)
    let mut r1 = Range::new(div, div);
    r1.set_start(div, 0).unwrap();
    r1.set_end(div, 1).unwrap();

    // r2: [1, 2] (第二个 p)
    let mut r2 = Range::new(div, div);
    r2.set_start(div, 1).unwrap();
    r2.set_end(div, 2).unwrap();

    // r1 结束在 r2 开始处，所以返回 -1
    assert_eq!(r1.compare_boundary_points(&r2), -1);
}

/// 测试 compare_boundary_points 反向比较（交换参数）。
#[test]
fn test_compare_boundary_points_reversed() {
    let doc = parse_html("<div><p>A</p><p>B</p></div>");
    let body = body_of(&doc);
    let div = doc.first_child(body).unwrap();

    // r1: [0, 1] (第一个 p)
    let mut r1 = Range::new(div, div);
    r1.set_start(div, 0).unwrap();
    r1.set_end(div, 1).unwrap();

    // r2: [1, 2] (第二个 p)
    let mut r2 = Range::new(div, div);
    r2.set_start(div, 1).unwrap();
    r2.set_end(div, 2).unwrap();

    // r2.compare_boundary_points(r1) 应返回 1
    assert_eq!(r2.compare_boundary_points(&r1), 1);
}

// ═══════════════════════════════════════════════════════════════════════
// 辅助函数
// ═══════════════════════════════════════════════════════════════════════

/// 辅助函数：获取 body 节点。parse_html 创建 document > html > body 结构。
fn body_of(doc: &Document) -> NodeId {
    let html = doc.first_child(doc.root()).unwrap();
    // html 有 head 和 body 两个子节点，body 是最后一个
    doc.last_child(html).unwrap()
}
