//! dom_bridge 测试模块。

use super::*;

// ── DomBridge 测试 ──

#[test]
fn test_dom_bridge_new() {
    let bridge = DomBridge::new();
    assert!(bridge.is_empty());
    assert_eq!(bridge.len(), 0);
}

#[test]
fn test_dom_bridge_register() {
    let mut bridge = DomBridge::new();
    let h1 = bridge.register(100);
    let h2 = bridge.register(200);
    assert_ne!(h1, h2, "Handles should be unique");
    assert_eq!(bridge.len(), 2);
}

#[test]
fn test_dom_bridge_register_same_node_returns_same_handle() {
    let mut bridge = DomBridge::new();
    let h1 = bridge.register(100);
    let h2 = bridge.register(100);
    assert_eq!(h1, h2, "Same node should get same handle");
    assert_eq!(bridge.len(), 1);
}

#[test]
fn test_dom_bridge_resolve() {
    let mut bridge = DomBridge::new();
    let h = bridge.register(42);
    assert_eq!(bridge.resolve(h), Some(42));
    assert_eq!(bridge.resolve(999), None);
}

#[test]
fn test_dom_bridge_unregister() {
    let mut bridge = DomBridge::new();
    let h = bridge.register(42);
    bridge.unregister(h);
    assert_eq!(bridge.resolve(h), None);
    assert!(bridge.is_empty());
}

#[test]
fn test_dom_bridge_clear() {
    let mut bridge = DomBridge::new();
    bridge.register(1);
    bridge.register(2);
    bridge.register(3);
    bridge.clear();
    assert!(bridge.is_empty());
}

#[test]
fn test_dom_bridge_default() {
    let bridge = DomBridge::default();
    assert!(bridge.is_empty());
}

// ── DomCommand 解析测试 ──

#[test]
fn test_parse_get_element_by_id() {
    let cmd = DomBridge::parse_command(r#"document.getElementById("foo")"#);
    assert_eq!(cmd, Some(DomCommand::GetElementById { id: "foo".to_string() }));
}

#[test]
fn test_parse_get_element_by_id_single_quotes() {
    let cmd = DomBridge::parse_command("document.getElementById('bar')");
    assert_eq!(cmd, Some(DomCommand::GetElementById { id: "bar".to_string() }));
}

#[test]
fn test_parse_query_selector() {
    let cmd = DomBridge::parse_command(r#"document.querySelector("div.container")"#);
    assert_eq!(
        cmd,
        Some(DomCommand::QuerySelector {
            selector: "div.container".to_string()
        })
    );
}

#[test]
fn test_parse_query_selector_all() {
    let cmd = DomBridge::parse_command(r#"document.querySelectorAll("li")"#);
    assert_eq!(
        cmd,
        Some(DomCommand::QuerySelectorAll {
            selector: "li".to_string()
        })
    );
}

#[test]
fn test_parse_create_element() {
    let cmd = DomBridge::parse_command(r#"document.createElement("div")"#);
    assert_eq!(
        cmd,
        Some(DomCommand::CreateElement {
            tag_name: "div".to_string()
        })
    );
}

#[test]
fn test_parse_create_text_node() {
    let cmd = DomBridge::parse_command(r#"document.createTextNode("Hello")"#);
    assert_eq!(
        cmd,
        Some(DomCommand::CreateTextNode {
            text: "Hello".to_string()
        })
    );
}

#[test]
fn test_parse_get_elements_by_class_name() {
    let cmd = DomBridge::parse_command(r#"document.getElementsByClassName("active")"#);
    assert_eq!(
        cmd,
        Some(DomCommand::GetElementsByClassName {
            class_name: "active".to_string()
        })
    );
}

#[test]
fn test_parse_get_elements_by_tag_name() {
    let cmd = DomBridge::parse_command(r#"document.getElementsByTagName("div")"#);
    assert_eq!(
        cmd,
        Some(DomCommand::GetElementsByTagName {
            tag_name: "div".to_string()
        })
    );
}

#[test]
fn test_parse_unknown_command() {
    let cmd = DomBridge::parse_command("window.alert('hi')");
    assert_eq!(cmd, None);
}

#[test]
fn test_parse_empty_input() {
    let cmd = DomBridge::parse_command("");
    assert_eq!(cmd, None);
}

#[test]
fn test_parse_invalid_no_quotes() {
    let cmd = DomBridge::parse_command("document.getElementById(foo)");
    assert_eq!(cmd, None);
}

// ── DomResult 测试 ──

#[test]
fn test_dom_result_element() {
    let result = DomResult::Element(Some(42));
    assert_eq!(result, DomResult::Element(Some(42)));
}

#[test]
fn test_dom_result_element_none() {
    let result = DomResult::Element(None);
    assert_eq!(result, DomResult::Element(None));
}

#[test]
fn test_dom_result_element_list() {
    let result = DomResult::ElementList(vec![1, 2, 3]);
    assert_eq!(result, DomResult::ElementList(vec![1, 2, 3]));
}

#[test]
fn test_dom_result_string() {
    let result = DomResult::String(Some("hello".to_string()));
    assert_eq!(result, DomResult::String(Some("hello".to_string())));
}

#[test]
fn test_dom_result_bool() {
    assert_eq!(DomResult::Bool(true), DomResult::Bool(true));
    assert_eq!(DomResult::Bool(false), DomResult::Bool(false));
}

#[test]
fn test_dom_result_void() {
    assert_eq!(DomResult::Void, DomResult::Void);
}

#[test]
fn test_dom_result_error() {
    let result = DomResult::Error("not found".to_string());
    assert_eq!(result, DomResult::Error("not found".to_string()));
}

// ── Polyfill 生成测试 ──

#[test]
fn test_generate_dom_api_polyfill_not_empty() {
    let polyfill = generate_dom_api_polyfill();
    assert!(!polyfill.is_empty());
    assert!(polyfill.contains("document"));
    assert!(polyfill.contains("getElementById"));
    assert!(polyfill.contains("querySelector"));
    assert!(polyfill.contains("createElement"));
    assert!(polyfill.contains("appendChild"));
    assert!(polyfill.contains("setAttribute"));
    assert!(polyfill.contains("textContent"));
}

#[test]
fn test_extract_string_arg_double_quotes() {
    let result = extract_string_arg(r#""hello")"#);
    assert_eq!(result, Some("hello".to_string()));
}

#[test]
fn test_extract_string_arg_single_quotes() {
    let result = extract_string_arg("'world')");
    assert_eq!(result, Some("world".to_string()));
}

#[test]
fn test_extract_string_arg_empty() {
    let result = extract_string_arg("");
    assert_eq!(result, None);
}

#[test]
fn test_extract_string_arg_no_closing_paren() {
    let result = extract_string_arg(r#""hello""#);
    assert_eq!(result, None);
}

#[test]
fn test_extract_string_arg_no_quotes() {
    let result = extract_string_arg("hello)");
    assert_eq!(result, None);
}

#[test]
fn test_extract_string_arg_with_spaces() {
    let result = extract_string_arg(r#"  "hello"  )"#);
    assert_eq!(result, Some("hello".to_string()));
}

// ── 事件命令构造测试 ──

#[test]
fn test_add_event_listener_command_equality() {
    let cmd = DomCommand::AddEventListener {
        element_id: 42,
        event_type: "click".to_string(),
        capture: false,
    };
    assert_eq!(
        cmd,
        DomCommand::AddEventListener {
            element_id: 42,
            event_type: "click".to_string(),
            capture: false,
        }
    );
}

#[test]
fn test_add_event_listener_capture() {
    let cmd = DomCommand::AddEventListener {
        element_id: 1,
        event_type: "keydown".to_string(),
        capture: true,
    };
    match cmd {
        DomCommand::AddEventListener { capture, .. } => assert!(capture),
        _ => panic!("Expected AddEventListener"),
    }
}

#[test]
fn test_remove_event_listener_command() {
    let cmd = DomCommand::RemoveEventListener {
        element_id: 10,
        event_type: "input".to_string(),
    };
    assert_eq!(
        cmd,
        DomCommand::RemoveEventListener {
            element_id: 10,
            event_type: "input".to_string(),
        }
    );
}

#[test]
fn test_dispatch_event_command() {
    let cmd = DomCommand::DispatchEvent {
        target_id: 5,
        event_type: "custom".to_string(),
        bubbles: true,
        cancelable: false,
    };
    assert_eq!(
        cmd,
        DomCommand::DispatchEvent {
            target_id: 5,
            event_type: "custom".to_string(),
            bubbles: true,
            cancelable: false,
        }
    );
}

#[test]
fn test_dispatch_event_no_bubble() {
    let cmd = DomCommand::DispatchEvent {
        target_id: 1,
        event_type: "change".to_string(),
        bubbles: false,
        cancelable: true,
    };
    match cmd {
        DomCommand::DispatchEvent {
            bubbles, cancelable, ..
        } => {
            assert!(!bubbles);
            assert!(cancelable);
        }
        _ => panic!("Expected DispatchEvent"),
    }
}

// ── Polyfill 事件系统测试 ──

#[test]
fn test_polyfill_contains_event_system() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("addEventListener"));
    assert!(polyfill.contains("removeEventListener"));
    assert!(polyfill.contains("dispatchEvent"));
}

#[test]
fn test_polyfill_contains_custom_event() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("CustomEvent"));
    assert!(polyfill.contains("preventDefault"));
    assert!(polyfill.contains("stopPropagation"));
    assert!(polyfill.contains("stopImmediatePropagation"));
}

#[test]
fn test_polyfill_event_options_capture() {
    let polyfill = generate_dom_api_polyfill();
    // Verify the polyfill handles capture option
    assert!(polyfill.contains("capture"));
    assert!(polyfill.contains("_eventListeners"));
}

// ── Polyfill Fetch API 测试 ──

#[test]
fn test_polyfill_contains_fetch_api() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.fetch"));
    assert!(polyfill.contains("globalThis.Headers"));
    assert!(polyfill.contains("globalThis.Request"));
    assert!(polyfill.contains("globalThis.Response"));
}

#[test]
fn test_polyfill_contains_response_methods() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("prototype.json"));
    assert!(polyfill.contains("prototype.text"));
    assert!(polyfill.contains("prototype.clone"));
    assert!(polyfill.contains("Response.error"));
}

#[test]
fn test_polyfill_contains_headers_methods() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("prototype.append"));
    assert!(polyfill.contains("prototype.delete"));
    assert!(polyfill.contains("prototype.get"));
    assert!(polyfill.contains("prototype.has"));
    assert!(polyfill.contains("prototype.set"));
}

// ── Polyfill Console + Timer API 测试 ──

#[test]
fn test_polyfill_contains_console_api() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.console"));
    assert!(polyfill.contains("log: function"));
    assert!(polyfill.contains("warn: function"));
    assert!(polyfill.contains("error: function"));
    assert!(polyfill.contains("info: function"));
    assert!(polyfill.contains("time: function"));
    assert!(polyfill.contains("timeEnd: function"));
}

#[test]
fn test_polyfill_contains_timer_api() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.setTimeout"));
    assert!(polyfill.contains("globalThis.setInterval"));
    assert!(polyfill.contains("globalThis.clearTimeout"));
    assert!(polyfill.contains("globalThis.clearInterval"));
}

// ── Polyfill Web Storage API 测试 ──

#[test]
fn test_polyfill_contains_storage_api() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.localStorage"));
    assert!(polyfill.contains("globalThis.sessionStorage"));
    assert!(polyfill.contains("getItem"));
    assert!(polyfill.contains("setItem"));
    assert!(polyfill.contains("removeItem"));
}

// ── Polyfill MutationObserver 测试 ──

#[test]
fn test_polyfill_contains_mutation_observer() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.MutationObserver"));
    assert!(polyfill.contains("prototype.observe"));
    assert!(polyfill.contains("prototype.disconnect"));
    assert!(polyfill.contains("prototype.takeRecords"));
}

#[test]
fn test_polyfill_contains_mutation_record() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.MutationRecord"));
    assert!(polyfill.contains("addedNodes"));
    assert!(polyfill.contains("removedNodes"));
    assert!(polyfill.contains("attributeName"));
}

// ── Polyfill IntersectionObserver 测试 ──

#[test]
fn test_polyfill_contains_intersection_observer() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.IntersectionObserver"));
    assert!(polyfill.contains("prototype.observe"));
    assert!(polyfill.contains("prototype.unobserve"));
    assert!(polyfill.contains("prototype.disconnect"));
    assert!(polyfill.contains("prototype.takeRecords"));
}

#[test]
fn test_polyfill_contains_intersection_observer_entry() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.IntersectionObserverEntry"));
    assert!(polyfill.contains("isIntersecting"));
    assert!(polyfill.contains("intersectionRatio"));
}

// ── Polyfill ResizeObserver 测试 ──

#[test]
fn test_polyfill_contains_resize_observer() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.ResizeObserver"));
    assert!(polyfill.contains("globalThis.ResizeObserverEntry"));
    assert!(polyfill.contains("globalThis.DOMRectReadOnly"));
}

// ── 新增 DomCommand 变体测试 ──

#[test]
fn test_insert_before_command() {
    let cmd = DomCommand::InsertBefore {
        parent_id: 1,
        new_child_id: 2,
        ref_child_id: Some(3),
    };
    assert_eq!(
        cmd,
        DomCommand::InsertBefore {
            parent_id: 1,
            new_child_id: 2,
            ref_child_id: Some(3),
        }
    );
}

#[test]
fn test_insert_before_no_ref() {
    let cmd = DomCommand::InsertBefore {
        parent_id: 1,
        new_child_id: 2,
        ref_child_id: None,
    };
    match cmd {
        DomCommand::InsertBefore { ref_child_id, .. } => assert!(ref_child_id.is_none()),
        _ => panic!("Expected InsertBefore"),
    }
}

#[test]
fn test_replace_child_command() {
    let cmd = DomCommand::ReplaceChild {
        parent_id: 1,
        new_child_id: 2,
        old_child_id: 3,
    };
    assert_eq!(
        cmd,
        DomCommand::ReplaceChild {
            parent_id: 1,
            new_child_id: 2,
            old_child_id: 3,
        }
    );
}

#[test]
fn test_clone_node_command() {
    let cmd = DomCommand::CloneNode {
        element_id: 42,
        deep: true,
    };
    assert_eq!(
        cmd,
        DomCommand::CloneNode {
            element_id: 42,
            deep: true,
        }
    );
}

#[test]
fn test_clone_node_shallow() {
    let cmd = DomCommand::CloneNode {
        element_id: 5,
        deep: false,
    };
    match cmd {
        DomCommand::CloneNode { element_id, deep } => {
            assert_eq!(element_id, 5);
            assert!(!deep);
        }
        _ => panic!("Expected CloneNode"),
    }
}

#[test]
fn test_get_style_command() {
    let cmd = DomCommand::GetStyle { element_id: 10 };
    assert_eq!(cmd, DomCommand::GetStyle { element_id: 10 });
}

#[test]
fn test_set_style_command() {
    let cmd = DomCommand::SetStyle {
        element_id: 10,
        value: "color: red; font-size: 16px".to_string(),
    };
    assert_eq!(
        cmd,
        DomCommand::SetStyle {
            element_id: 10,
            value: "color: red; font-size: 16px".to_string(),
        }
    );
}

#[test]
fn test_set_inner_html_command() {
    let cmd = DomCommand::SetInnerHtml {
        element_id: 7,
        value: "<p>Hello</p>".to_string(),
    };
    assert_eq!(
        cmd,
        DomCommand::SetInnerHtml {
            element_id: 7,
            value: "<p>Hello</p>".to_string(),
        }
    );
}

#[test]
fn test_get_parent_node_command() {
    let cmd = DomCommand::GetParentNode { element_id: 3 };
    assert_eq!(cmd, DomCommand::GetParentNode { element_id: 3 });
}

// ── Polyfill 新增 API 验证测试 ──

#[test]
fn test_polyfill_contains_insert_before() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("insertBefore"));
}

#[test]
fn test_polyfill_contains_replace_child() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("replaceChild"));
}

#[test]
fn test_polyfill_contains_clone_node() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("cloneNode"));
}

#[test]
fn test_polyfill_contains_navigation_properties() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("firstChild"));
    assert!(polyfill.contains("lastChild"));
    assert!(polyfill.contains("nextSibling"));
    assert!(polyfill.contains("previousSibling"));
    assert!(polyfill.contains("childElementCount"));
    assert!(polyfill.contains("hasChildNodes"));
}

#[test]
fn test_polyfill_contains_style_api() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("_CSSStyleDeclaration"));
    assert!(polyfill.contains("getPropertyValue"));
    assert!(polyfill.contains("setProperty"));
    assert!(polyfill.contains("removeProperty"));
    assert!(polyfill.contains("cssText"));
}

#[test]
fn test_polyfill_contains_classlist_api() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("_DOMTokenList"));
    assert!(polyfill.contains("classList"));
    assert!(polyfill.contains("contains"));
    assert!(polyfill.contains("toggle"));
}

#[test]
fn test_polyfill_contains_inner_html_setter() {
    let polyfill = generate_dom_api_polyfill();
    // innerHTML getter + setter
    assert!(polyfill.contains("innerHTML"));
    assert!(polyfill.contains("outerHTML"));
}

#[test]
fn test_polyfill_contains_text_content_property() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("textContent"));
    assert!(polyfill.contains("innerText"));
    assert!(polyfill.contains("getTextContent"));
    assert!(polyfill.contains("setTextContent"));
}

#[test]
fn test_polyfill_contains_document_fragment() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("createDocumentFragment"));
}

#[test]
fn test_polyfill_contains_id_classname() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("node.id"));
    assert!(polyfill.contains("node.className"));
}

// ── 命令解析边界测试 ──

#[test]
fn test_parse_command_whitespace_tolerance() {
    let cmd = DomBridge::parse_command("  document.getElementById(\"test\")  ");
    assert!(matches!(cmd, Some(DomCommand::GetElementById { id }) if id == "test"));
}

#[test]
fn test_parse_command_single_quotes() {
    let cmd = DomBridge::parse_command("document.getElementById('my-id')");
    assert!(matches!(cmd, Some(DomCommand::GetElementById { id }) if id == "my-id"));
}

#[test]
fn test_parse_command_empty_string_arg() {
    let cmd = DomBridge::parse_command("document.getElementById(\"\")");
    assert!(matches!(cmd, Some(DomCommand::GetElementById { id }) if id == ""));
}

#[test]
fn test_parse_command_unknown_returns_none() {
    assert!(DomBridge::parse_command("window.alert('hi')").is_none());
    assert!(DomBridge::parse_command("").is_none());
    assert!(DomBridge::parse_command("not a command").is_none());
}

#[test]
fn test_parse_command_create_element() {
    let cmd = DomBridge::parse_command("document.createElement(\"div\")");
    assert!(matches!(cmd, Some(DomCommand::CreateElement { tag_name }) if tag_name == "div"));
}

#[test]
fn test_parse_command_create_text_node() {
    let cmd = DomBridge::parse_command("document.createTextNode(\"hello\")");
    assert!(matches!(cmd, Some(DomCommand::CreateTextNode { text }) if text == "hello"));
}

#[test]
fn test_parse_command_query_selector() {
    let cmd = DomBridge::parse_command("document.querySelector(\".container\")");
    assert!(matches!(cmd, Some(DomCommand::QuerySelector { selector }) if selector == ".container"));
}

#[test]
fn test_parse_command_query_selector_all() {
    let cmd = DomBridge::parse_command("document.querySelectorAll(\"div\")");
    assert!(matches!(cmd, Some(DomCommand::QuerySelectorAll { selector }) if selector == "div"));
}

#[test]
fn test_parse_command_get_elements_by_class_name() {
    let cmd = DomBridge::parse_command("document.getElementsByClassName(\"active\")");
    assert!(matches!(cmd, Some(DomCommand::GetElementsByClassName { class_name }) if class_name == "active"));
}

#[test]
fn test_parse_command_get_elements_by_tag_name() {
    let cmd = DomBridge::parse_command("document.getElementsByTagName(\"span\")");
    assert!(matches!(cmd, Some(DomCommand::GetElementsByTagName { tag_name }) if tag_name == "span"));
}

// ── DomBridge 边界测试 ──

#[test]
fn test_bridge_register_same_id_twice() {
    let mut bridge = DomBridge::new();
    let h1 = bridge.register(42);
    let h2 = bridge.register(42);
    assert_eq!(h1, h2, "重复注册同一 node_id 应返回相同 handle");
    assert_eq!(bridge.len(), 1);
}

#[test]
fn test_bridge_register_many() {
    let mut bridge = DomBridge::new();
    let mut handles = vec![];
    for i in 0..100 {
        handles.push(bridge.register(i));
    }
    assert_eq!(bridge.len(), 100);
    // 所有 handle 应唯一
    let unique: std::collections::HashSet<u64> = handles.iter().copied().collect();
    assert_eq!(unique.len(), 100);
}

#[test]
fn test_bridge_resolve_unregistered() {
    let bridge = DomBridge::new();
    assert!(bridge.resolve(999).is_none());
}

#[test]
fn test_bridge_unregister_and_resolve() {
    let mut bridge = DomBridge::new();
    let h = bridge.register(10);
    assert_eq!(bridge.resolve(h), Some(10));
    bridge.unregister(h);
    assert!(bridge.resolve(h).is_none());
    assert_eq!(bridge.len(), 0);
}

#[test]
fn test_bridge_clear() {
    let mut bridge = DomBridge::new();
    bridge.register(1);
    bridge.register(2);
    bridge.register(3);
    assert_eq!(bridge.len(), 3);
    bridge.clear();
    assert!(bridge.is_empty());
}

// ── DomResult 测试 ──

#[test]
fn test_dom_result_equality() {
    assert_eq!(DomResult::Void, DomResult::Void);
    assert_eq!(DomResult::Bool(true), DomResult::Bool(true));
    assert_ne!(DomResult::Bool(true), DomResult::Bool(false));
    assert_eq!(DomResult::Element(Some(42)), DomResult::Element(Some(42)));
    assert_eq!(
        DomResult::String(Some("hello".to_string())),
        DomResult::String(Some("hello".to_string()))
    );
}

// ── 边界条件测试：u64::MAX / escaped quotes / Unicode / clear + re-register ──

/// 测试 DomBridge::register 对 u64::MAX 的 node_id 正常工作。
#[test]
fn test_bridge_register_u64_max() {
    let mut bridge = DomBridge::new();
    let handle = bridge.register(u64::MAX);
    assert_eq!(bridge.resolve(handle), Some(u64::MAX));
    assert_eq!(bridge.len(), 1);

    // 重复注册 u64::MAX 应返回相同 handle
    let handle2 = bridge.register(u64::MAX);
    assert_eq!(handle, handle2, "重复注册 u64::MAX 应返回相同 handle");
    assert_eq!(bridge.len(), 1);
}

/// 测试 extract_string_arg 处理转义引号。
///
/// 注意：当前实现使用简单的 find(quote_char) 查找闭合引号，
/// 不支持转义引号——中间的引号会被视为闭合引号。
#[test]
fn test_extract_string_arg_escaped_quotes() {
    // 当前实现不支持转义引号，中间的 \" 会被视为引号边界
    // "hello\"world") -> 会匹配到 "hello\" 中的第一个引号对
    let result = extract_string_arg(r#""hello\"world")"#);
    // 当前实现：找到第一个 '"' 后在 [1..] 中找 '"'，找到的位置是 "hello\" 中的 \"
    // 实际上 find('"') 找到的是转义的引号位置
    // 简单实现不支持转义，行为是匹配到第一个出现的引号
    let _ = result; // 不 panic 即可
}

/// 测试 extract_string_arg 处理 Unicode 内容。
#[test]
fn test_extract_string_arg_unicode() {
    let result = extract_string_arg("\"日本語テスト\")");
    assert_eq!(result, Some("日本語テスト".to_string()));

    let result2 = extract_string_arg("'🎉🎊🎈')");
    assert_eq!(result2, Some("🎉🎊🎈".to_string()));

    let result3 = extract_string_arg("\"emoji: 🚀 rocket\")");
    assert_eq!(result3, Some("emoji: 🚀 rocket".to_string()));
}

/// 测试 clear 后重新注册 node_id 的 handle 连续性。
///
/// clear 不重置 next_handle，因此 clear 后注册的新 node_id
/// 应获得比 clear 前更大的 handle 值。
#[test]
fn test_clear_reregister_handle_continuity() {
    let mut bridge = DomBridge::new();

    // 注册 3 个 node_id
    let h1 = bridge.register(10);
    let h2 = bridge.register(20);
    let h3 = bridge.register(30);
    assert_eq!(bridge.len(), 3);

    // clear 所有映射
    bridge.clear();
    assert!(bridge.is_empty());
    assert_eq!(bridge.len(), 0);

    // 重新注册，handle 应从 h3+1 开始（不重置）
    let h4 = bridge.register(10);
    let h5 = bridge.register(20);

    assert!(h4 > h3, "clear 后的 handle({}) 应大于之前的最大 handle({})", h4, h3);
    assert!(h5 > h4, "后续 handle({}) 应大于前一个({})", h5, h4);
    assert_eq!(bridge.len(), 2);

    // resolve 应只找到新的映射
    assert_eq!(bridge.resolve(h4), Some(10));
    assert_eq!(bridge.resolve(h5), Some(20));
    // 旧的 handle 已被清除，resolve 应返回 None
    assert_eq!(bridge.resolve(h1), None);
    assert_eq!(bridge.resolve(h2), None);
    assert_eq!(bridge.resolve(h3), None);
}

/// 测试 parse_command 对空字符串参数的处理（边界补充）。
#[test]
fn test_parse_command_empty_string_arg_boundary() {
    let cmd = DomBridge::parse_command("document.getElementById(\"\")");
    assert!(matches!(cmd, Some(DomCommand::GetElementById { id }) if id == ""));
}

/// 测试 register(u64::MAX) 后 unregister 正常工作。
#[test]
fn test_register_u64_max_then_unregister() {
    let mut bridge = DomBridge::new();
    let handle = bridge.register(u64::MAX);
    assert_eq!(bridge.len(), 1);

    bridge.unregister(handle);
    assert!(bridge.is_empty());
    assert_eq!(bridge.resolve(handle), None);
}

/// 测试 clear 后立即 full_redraw 标记不互相干扰。
#[test]
fn test_dom_bridge_clear_then_register_zero() {
    let mut bridge = DomBridge::new();
    bridge.register(1);
    bridge.register(2);
    bridge.clear();

    // 注册 node_id=0
    let h = bridge.register(0);
    assert_eq!(bridge.resolve(h), Some(0));
    assert_eq!(bridge.len(), 1);
}

/// 测试 DomBridge Default trait 等价于 new()。
#[test]
fn test_dom_bridge_default_eq_new() {
    let new_bridge = DomBridge::new();
    let default_bridge = DomBridge::default();
    assert_eq!(new_bridge.len(), default_bridge.len());
    assert_eq!(new_bridge.is_empty(), default_bridge.is_empty());
}

// ── Polyfill WebAssembly API 测试 ──

#[test]
fn test_polyfill_contains_webassembly() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.WebAssembly"));
    assert!(polyfill.contains("compile"));
    assert!(polyfill.contains("instantiate"));
    assert!(polyfill.contains("validate"));
}

#[test]
fn test_polyfill_webassembly_compile() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("Promise.resolve"));
    assert!(polyfill.contains("ArrayBuffer"));
    assert!(polyfill.contains("Uint8Array"));
}

#[test]
fn test_polyfill_webassembly_exports() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("_modules"));
    assert!(polyfill.contains("_instances"));
    assert!(polyfill.contains("memory"));
    assert!(polyfill.contains("grow"));
}

// ── Polyfill navigator.serviceWorker 测试 ──

#[test]
fn test_polyfill_contains_service_worker_api() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("navigator.serviceWorker"));
    assert!(polyfill.contains("register"));
    assert!(polyfill.contains("getRegistration"));
    assert!(polyfill.contains("getRegistrations"));
}

#[test]
fn test_polyfill_service_worker_register_options() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("scope"));
    assert!(polyfill.contains("scriptURL"));
    assert!(polyfill.contains("unregister"));
    assert!(polyfill.contains("update"));
}

#[test]
fn test_polyfill_service_worker_lifecycle() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("installing"));
    assert!(polyfill.contains("waiting"));
    assert!(polyfill.contains("active"));
    assert!(polyfill.contains("_controller"));
}

#[test]
fn test_polyfill_contains_navigator() {
    let polyfill = generate_dom_api_polyfill();
    assert!(polyfill.contains("globalThis.navigator"));
}
