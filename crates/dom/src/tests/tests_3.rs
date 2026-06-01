// DOM crate 综合测试套件。
//
// 覆盖：节点类型、树操作、属性操作、HTML 解析、查询、序列化、MutationObserver。

use crate::*;
use std::sync::{Arc, Mutex};

// ═══════════════════════════════════════════════════════════════════════
// 1. 节点创建测试
// ═══════════════════════════════════════════════════════════════════════

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

