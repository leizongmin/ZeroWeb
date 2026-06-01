// DOM crate 综合测试套件。
//
// 覆盖：节点类型、树操作、属性操作、HTML 解析、查询、序列化、MutationObserver。

use crate::*;

// ═══════════════════════════════════════════════════════════════════════
// 1. 节点创建测试
// ═══════════════════════════════════════════════════════════════════════

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

// ── 新增边界测试 ──

/// 测试连续 append_child 维持正确顺序。
#[test]
fn test_append_child_ordering() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("ul");
    doc.append_child(root, parent).unwrap();

    let a = doc.create_element("li");
    let b = doc.create_element("li");
    let c = doc.create_element("li");
    doc.append_child(parent, a).unwrap();
    doc.append_child(parent, b).unwrap();
    doc.append_child(parent, c).unwrap();

    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 3, "应有 3 个子节点");
    assert_eq!(children[0], a, "第一个子节点应为 a");
    assert_eq!(children[1], b, "第二个子节点应为 b");
    assert_eq!(children[2], c, "第三个子节点应为 c");
}

/// 测试 set_attribute 覆写已有值（边界补充）。
#[test]
fn test_set_attribute_overwrite_returns_new() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "data-val", "first");
    assert_eq!(doc.get_attribute(elem, "data-val"), Some("first".to_string()));

    doc.set_attribute(elem, "data-val", "second");
    assert_eq!(
        doc.get_attribute(elem, "data-val"),
        Some("second".to_string()),
        "覆写后应返回最新值"
    );
}

/// 测试 create_element 名称大小写保留。
#[test]
fn test_create_element_case_preserved() {
    let mut doc = Document::new();
    let elem = doc.create_element("MyComponent");
    // 元素创建后 tag_name 应保留原始大小写
    assert!(doc.outer_html(elem).contains("MyComponent"), "tag 名应保留大小写");
}

/// 测试 remove_child 从中间移除不影响兄弟顺序。
#[test]
fn test_remove_child_middle_sibling() {
    let mut doc = Document::new();
    let root = doc.root();
    let parent = doc.create_element("div");
    doc.append_child(root, parent).unwrap();

    let a = doc.create_element("span");
    let b = doc.create_element("span");
    let c = doc.create_element("span");
    doc.append_child(parent, a).unwrap();
    doc.append_child(parent, b).unwrap();
    doc.append_child(parent, c).unwrap();

    doc.remove_child(parent, b).unwrap();

    let children = doc.child_nodes(parent);
    assert_eq!(children.len(), 2, "移除后应有 2 个子节点");
    assert_eq!(children[0], a, "第一个应为 a");
    assert_eq!(children[1], c, "第二个应为 c");
}

/// 测试 create_text_node 空字符串不 panic。
#[test]
fn test_create_text_node_empty_string() {
    let mut doc = Document::new();
    let text = doc.create_text_node("");
    let root = doc.root();
    doc.append_child(root, text).unwrap();

    let html = doc.outer_html(text);
    // 空文本节点序列化后应为空字符串
    assert_eq!(html, "", "空文本节点序列化应为空字符串");
}
