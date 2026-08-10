// DOM crate 综合测试套件。
//
// 覆盖：节点类型、树操作、属性操作、HTML 解析、查询、序列化、MutationObserver。

use crate::*;
use std::sync::{Arc, Mutex};

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

/// 测试 create_element 对 HTML 文档 ASCII 小写 tag 名（R3173）。
///
/// spec `dom-document-createelement` step 2：HTML 文档须将 localName ASCII 小写后创建。
/// `create_element("MyComponent")` → `local_name` "mycomponent"（小写，与真实浏览器一致）；
/// `tagName` getter HTML-uppercased → "MYCOMPONENT"；`outer_html` 原样输出 local_name →
/// "<mycomponent></mycomponent>"（serializer 不主动小写，小写职责在 create_element）。
#[test]
fn test_create_element_html_lowercased_r3173() {
    let mut doc = Document::new();
    let elem = doc.create_element("MyComponent");
    // DOM 层 local_name 已小写（DOM spec createElement step 2）。
    if let Some(NodeKind::Element(e)) = doc.get(elem).map(|n| n.kind.clone()) {
        assert_eq!(e.local_name(), "mycomponent", "local_name 应按 DOM spec 小写");
        assert_eq!(e.tag_name(), "MYCOMPONENT", "tagName getter 应 HTML-uppercased");
    }
    // serializer 原样输出 local_name（已小写），无需主动小写。
    assert_eq!(
        doc.outer_html(elem),
        "<mycomponent></mycomponent>",
        "outer_html 应输出已小写的 local_name"
    );
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

// ═══════════════════════════════════════════════════════════════════════
// 边界条件补充测试（round 22）
// ═══════════════════════════════════════════════════════════════════════

/// 测试 compare_document_position 对同一节点返回 0（bits 为空）。
///
/// 验证 node1 == node2 时结果为 DocumentPosition(0)，
/// 不包含任何 PRECEDING、FOLLOWING、CONTAINS 等标志。
#[test]
fn test_compare_document_position_same_node_zero_flags() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.append_child(root, elem).unwrap();

    let pos = doc.compare_document_position(elem, elem).unwrap();
    assert_eq!(pos.bits(), 0, "同一节点比较应返回 0");
    assert!(!pos.contains(DocumentPosition::PRECEDING));
    assert!(!pos.contains(DocumentPosition::FOLLOWING));
    assert!(!pos.contains(DocumentPosition::CONTAINS));
    assert!(!pos.contains(DocumentPosition::CONTAINED_BY));
    assert!(!pos.contains(DocumentPosition::DISCONNECTED));
}

/// 测试 create_element 使用含连字符的标签名（自定义元素命名规则）。
///
/// Web Components 规范要求自定义元素名必须含连字符。
/// 验证 create_element 正确保留连字符标签名。
#[test]
fn test_create_element_with_hyphenated_name() {
    let mut doc = Document::new();
    let elem = doc.create_element("my-widget");
    assert!(doc.contains(elem));
    if let Some(NodeKind::Element(e)) = doc.get(elem).map(|n| n.kind.clone()) {
        assert_eq!(e.local_name(), "my-widget", "应保留含连字符的标签名");
    }
}

/// 测试 DOMTokenList 边界：设置重复 class 后 get_elements_by_class_name 不重复返回。
///
/// set_attribute("class", "foo foo foo") 存储时 class_list 解析为
/// ["foo", "foo", "foo"]，但 get_elements_by_class_name 仍只返回元素一次。
#[test]
fn test_duplicate_class_in_attribute() {
    let mut doc = Document::new();
    let root = doc.root();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "foo foo foo");
    doc.append_child(root, elem).unwrap();

    // 元素在结果中只出现一次
    let result = doc.get_elements_by_class_name("foo");
    assert_eq!(result.len(), 1, "重复 class 不应导致元素重复出现");
    assert_eq!(result[0], elem);
}

/// 测试 DOMTokenList 边界：移除不存在的 class 后属性值不变。
#[test]
fn test_remove_nonexistent_class_preserves_others() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "foo bar baz");
    doc.append_child(doc.root(), elem).unwrap();

    // 手动移除一个不存在的 class "qux"，保留其他
    let current = doc.get_attribute(elem, "class").unwrap();
    let filtered: Vec<&str> = current.split_whitespace().filter(|c| *c != "qux").collect();
    doc.set_attribute(elem, "class", &filtered.join(" "));

    // 所有原有 class 仍然存在
    assert_eq!(doc.get_attribute(elem, "class"), Some("foo bar baz".to_string()));
    assert_eq!(doc.get_elements_by_class_name("foo"), vec![elem]);
    assert_eq!(doc.get_elements_by_class_name("bar"), vec![elem]);
    assert_eq!(doc.get_elements_by_class_name("baz"), vec![elem]);
}

/// 测试 DOMTokenList 边界：在空 class 属性上操作不 panic。
#[test]
fn test_class_operations_on_empty_class() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.set_attribute(elem, "class", "");
    doc.append_child(doc.root(), elem).unwrap();

    // 空 class 属性上查询任何 class 都不匹配
    assert!(doc.get_elements_by_class_name("foo").is_empty());
    assert!(doc.get_elements_by_class_name("").is_empty());

    // 设置为空白字符串也视为空
    doc.set_attribute(elem, "class", "   ");
    assert!(doc.get_elements_by_class_name("foo").is_empty());
}

/// 测试 Range select_node_contents 对空元素返回折叠范围（offset 0, 0）。
#[test]
fn test_range_select_node_contents_empty_element() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    let mut range = Range::new(elem, elem);
    range.select_node_contents(&doc, elem).unwrap();

    assert_eq!(range.start_offset(), 0, "空元素 start_offset 应为 0");
    assert_eq!(range.end_offset(), 0, "空元素 end_offset 应为 0");
    assert!(range.collapsed(), "空元素 select_node_contents 应产生折叠范围");
}

/// 测试 Range clone_contents 对嵌套多层元素深拷贝后结构独立。
#[test]
fn test_range_clone_contents_nested_independence() {
    let mut doc = Document::new();
    let parent = doc.create_element("div");
    doc.append_child(doc.root(), parent).unwrap();

    // 构建嵌套结构：div > ul > li > span > "text"
    let ul = doc.create_element("ul");
    let li = doc.create_element("li");
    let span = doc.create_element("span");
    let text = doc.create_text_node("text");
    doc.append_child(parent, ul).unwrap();
    doc.append_child(ul, li).unwrap();
    doc.append_child(li, span).unwrap();
    doc.append_child(span, text).unwrap();

    let mut range = Range::new(parent, parent);
    range.select_node_contents(&doc, parent).unwrap();
    let fragment = range.clone_contents(&mut doc).unwrap();

    // 修改原始树不影响克隆
    doc.set_text_content(span, "modified");
    // 克隆的 text_content 仍为 "text"
    let frag_children = doc.child_nodes(fragment);
    assert_eq!(frag_children.len(), 1, "克隆片段应有 1 个子节点");
    assert_eq!(
        doc.text_content(fragment),
        Some("text".to_string()),
        "克隆应为独立副本，不受原始修改影响"
    );
    // 原始已改变
    assert_eq!(doc.text_content(parent), Some("modified".to_string()));
}

/// 测试序列化带 public_id 和 system_id 的 DocumentType。
#[test]
fn test_serialize_doctype_with_public_and_system_id() {
    let mut doc = Document::new();
    let doctype = doc.create_document_type(
        "html",
        Some("-//W3C//DTD XHTML 1.0 Strict//EN".to_string()),
        Some("http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd".to_string()),
    );
    let html = doc.outer_html(doctype);
    assert!(html.contains("<!DOCTYPE"), "应包含 DOCTYPE 声明");
    assert!(html.contains("PUBLIC"), "应包含 PUBLIC 关键字");
    assert!(html.contains(r#"-//W3C//DTD XHTML 1.0 Strict//EN"#), "应包含 public_id");
    assert!(
        html.contains("http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd"),
        "应包含 system_id"
    );
}

/// 测试序列化仅带 system_id（无 public_id）的 DocumentType。
#[test]
fn test_serialize_doctype_system_id_only() {
    let mut doc = Document::new();
    let doctype = doc.create_document_type("html", None, Some("about:legacy-compat".to_string()));
    let html = doc.outer_html(doctype);
    assert!(html.contains("<!DOCTYPE"), "应包含 DOCTYPE 声明");
    // 仅 system_id 时输出 SYSTEM 关键字
    assert!(
        html.contains("SYSTEM"),
        "仅 system_id 时应使用 SYSTEM 关键字，实际: {html}"
    );
    assert!(html.contains("about:legacy-compat"), "应包含 system_id 值");
}

/// 测试 TreeWalker 遍历混合节点类型（元素、文本、注释）。
#[test]
fn test_tree_walker_mixed_node_types() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let span = doc.create_element("span");
    let text = doc.create_text_node("hello");
    let comment = doc.create_comment("note");
    let p = doc.create_element("p");
    doc.append_child(root, span).unwrap();
    doc.append_child(root, text).unwrap();
    doc.append_child(root, comment).unwrap();
    doc.append_child(root, p).unwrap();

    let mut walker = TreeWalker::new(root, 0xFFFFFFFF);

    // 深度优先前序遍历
    let n1 = walker.next_node(&doc);
    assert_eq!(n1, Some(span), "第一个子节点应为 span");

    let n2 = walker.next_node(&doc);
    assert_eq!(n2, Some(text), "第二个子节点应为文本节点");

    let n3 = walker.next_node(&doc);
    assert_eq!(n3, Some(comment), "第三个子节点应为注释节点");

    let n4 = walker.next_node(&doc);
    assert_eq!(n4, Some(p), "第四个子节点应为 p");

    // 遍历完毕
    let n5 = walker.next_node(&doc);
    assert_eq!(n5, None, "遍历完毕后应返回 None");
}

/// 测试 TreeWalker first_child 和 next_sibling 导航。
#[test]
fn test_tree_walker_child_and_sibling_navigation() {
    let mut doc = Document::new();
    let root = doc.create_element("div");
    let c1 = doc.create_element("a");
    let c2 = doc.create_element("b");
    let inner = doc.create_element("i");
    doc.append_child(root, c1).unwrap();
    doc.append_child(root, c2).unwrap();
    doc.append_child(c1, inner).unwrap();

    let mut walker = TreeWalker::new(root, 0xFFFFFFFF);

    // first_child 从 root 进入第一个子节点
    let child = walker.first_child(&doc);
    assert_eq!(child, Some(c1), "first_child 应为 a");
    assert_eq!(walker.current_node(), c1);

    // first_child 从 c1 进入其子节点
    let inner_node = walker.first_child(&doc);
    assert_eq!(inner_node, Some(inner), "a 的 first_child 应为 i");

    // next_sibling 从 inner 应该返回 None（无兄弟）
    let sibling = walker.next_sibling(&doc);
    assert_eq!(sibling, None, "i 没有兄弟节点");
}

/// 测试 dispatch_event 在断开连接（未附加到文档树）的节点上正常工作。
#[test]
fn test_dispatch_event_on_disconnected_node_sets_target() {
    let mut doc = Document::new();
    let orphan = doc.create_element("div");
    // 不 append 到文档树

    let called = Arc::new(Mutex::new(false));
    let called_clone = called.clone();
    doc.add_event_listener(
        orphan,
        "click",
        Box::new(move |event| {
            assert_eq!(event.target(), Some(orphan), "target 应为孤立节点自身");
            *called_clone.lock().unwrap() = true;
        }),
        false,
    );

    let mut event = Event::new("click");
    let result = doc.dispatch_event(orphan, &mut event);
    assert!(result, "dispatch 应正常完成");
    assert!(*called.lock().unwrap(), "监听器应被调用");
    assert_eq!(event.target(), Some(orphan), "事件 target 应被设置");
}

/// 测试 prevent_default 在不可取消事件上无效且 default_prevented 保持 false。
#[test]
fn test_prevent_default_on_non_cancelable_event_in_dispatch() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();

    doc.add_event_listener(
        elem,
        "load",
        Box::new(|event| {
            let result = event.prevent_default();
            assert!(!result, "不可取消事件上 prevent_default 应返回 false");
            assert!(!event.default_prevented(), "default_prevented 应保持 false");
        }),
        false,
    );

    let mut event = Event::new("load"); // cancelable=false
    let not_prevented = doc.dispatch_event(elem, &mut event);
    assert!(not_prevented, "不可取消事件的 dispatch 应返回 true");
    assert!(!event.default_prevented());
}

// ═══════════════════════════════════════════════════════════════════════
// R3174 属性名 HTML 命名空间 case-insensitive（DOM spec setAttribute step 2 /
// get-an-attribute-by-name step 1）
// ═══════════════════════════════════════════════════════════════════════

/// HTML 元素属性名 ASCII case-insensitive：`set_attribute("Foo")` 后任意大小写 `get_attribute`
/// 均命中，`attribute_names()` 返回小写存储名。DOM spec「get an attribute by name」step 1：
/// HTML 文档 name ASCII 小写。
#[test]
fn test_html_attribute_name_case_insensitive_r3174() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    // 大写设置
    doc.set_attribute(elem, "Data-Role", "button");
    // 任意大小写读取均命中（case-insensitive lookup）
    assert_eq!(doc.get_attribute(elem, "data-role"), Some("button".to_string()));
    assert_eq!(doc.get_attribute(elem, "DATA-ROLE"), Some("button".to_string()));
    assert_eq!(doc.get_attribute(elem, "Data-Role"), Some("button".to_string()));
    // 存储名为小写（setAttribute step 2 小写 qualifiedName）
    assert_eq!(doc.attribute_names(elem), vec!["data-role".to_string()]);
    // has_attribute / remove_attribute 同样 case-insensitive
    assert!(doc.has_attribute(elem, "DATA-ROLE"));
    doc.remove_attribute(elem, "data-ROLE");
    assert!(!doc.has_attribute(elem, "data-role"));
}

/// HTML 元素 `set_attribute("ID", ...)` 经 case-insensitive 小写后正确注册到 id_map，
/// `get_element_by_id` 可命中（id_map 检测用 effective name，HTML "ID"/"Id" 均算 id 属性）。
#[test]
fn test_html_attribute_id_uppercase_registers_in_id_map_r3174() {
    let mut doc = Document::new();
    let elem = doc.create_element("div");
    doc.append_child(doc.root(), elem).unwrap();
    // 大写 "ID" 应被识别为 id 属性并注册 id_map
    doc.set_attribute(elem, "ID", "main-btn");
    assert_eq!(doc.get_element_by_id("main-btn"), Some(elem));
    // 大写移除也应清理 id_map
    doc.remove_attribute(elem, "ID");
    assert_eq!(doc.get_element_by_id("main-btn"), None);
}

/// 非 HTML 命名空间（SVG）元素属性名**大小写敏感**：`set_attribute("viewBox")` 保留原样，
/// `get_attribute("viewbox")`（小写）不命中。SVG 属性大小写敏感是 SVG 整合的关键
///（`viewBox` vs `viewbox` 是不同属性）。
#[test]
fn test_svg_attribute_name_case_sensitive_r3174() {
    let mut doc = Document::new();
    let rect = doc.create_element_ns("http://www.w3.org/2000/svg", "rect");
    doc.set_attribute(rect, "viewBox", "0 0 100 50");
    // 精确大小写命中
    assert_eq!(doc.get_attribute(rect, "viewBox"), Some("0 0 100 50".to_string()));
    // 小写不命中（SVG 大小写敏感，未小写）
    assert_eq!(doc.get_attribute(rect, "viewbox"), None);
    assert!(!doc.has_attribute(rect, "VIEWBOX"));
    // 存储名保留原大小写
    assert_eq!(doc.attribute_names(rect), vec!["viewBox".to_string()]);
}
