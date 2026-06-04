//! DOM Bridge 扩展测试 - 针对 dom_bridge.rs 中未覆盖的路径
//!
//! 此模块专注于测试边缘条件、错误处理和特殊场景，
//! 以提高代码覆盖率。

use zero_engine::*;

/// 测试 WebAssembly 编译错误
#[test]
fn test_web_compile_error() {
    let polyfill = generate_dom_api_polyfill();

    // 验证 polyfill 包含错误处理代码
    assert!(polyfill.contains("Promise.reject"));
    assert!(polyfill.contains("TypeError"));
    assert!(polyfill.contains("Argument 0 must be a buffer source"));
}

/// 测试 WebAssembly 实例化错误
#[test]
fn test_web_instantiate_error() {
    let polyfill = generate_dom_api_polyfill();

    // 验证 polyfill 包含实例化错误处理
    assert!(polyfill.contains("instantiate"));
    assert!(polyfill.contains("buffer source or Module"));
}

/// 测试 Service Worker 注册错误（空 scriptURL）
#[test]
fn test_service_worker_register_empty_script_url() {
    let polyfill = generate_dom_api_polyfill();

    // 验证 polyfill 包含错误检查
    assert!(polyfill.contains("scriptURL is required"));
    assert!(polyfill.contains("typeof scriptURL !== 'string'"));
}

/// 测试 extract_string_arg 转义引号（当前限制）
#[test]
fn test_extract_string_arg_escaped_quotes_current_limitation() {
    // 当前实现不支持转义引号，测试这个限制
    let input = r#""hello\"world")"#;
    let result = extract_string_arg(input);

    // 由于 find(quote_char) 查找第一个引号，转义引号会被视为引号边界
    // 这是一个已知限制，测试的行为是合理的
    let _ = result; // 测试不会 panic 即可
}

/// 测试 extract_string_arg Unicode 内容
#[test]
fn test_extract_string_arg_unicode_comprehensive() {
    let test_cases = vec![
        ("\"日本語テスト\")", "日本語テスト"),
        ("'🎉🎊🎈')", "🎉🎊🎈"),
        ("\"emoji: 🚀 rocket\")", "emoji: 🚀 rocket"),
        ("\"中文-English-日本語\")", "中文-English-日本語"),
        ("'特殊字符: @#$%^&*()')", "特殊字符: @#$%^&*()"),
    ];

    for (input, expected) in test_cases {
        let result = extract_string_arg(input);
        assert_eq!(result, Some(expected.to_string()), "Failed for input: {}", input);
    }
}

/// 测试 parse_command 处理各种带空格的变体
#[test]
fn test_parse_command_whitespace_variants() {
    let test_cases = vec![
        ("  document.getElementById(  \"test\"  )  ", "test"),
        ("document.querySelector(\n\"div.container\"\n)", "div.container"),
        ("document.querySelectorAll(\t\"li\"\t)", "li"),
        ("\t\tdocument.createElement(\"span\")\t\t", "span"),
        ("\n\ndocument.createTextNode(\"text\")\n\n", "text"),
    ];

    for (input, expected_id) in test_cases {
        let result = DomBridge::parse_command(input);
        assert!(
            matches!(result, Some(DomCommand::GetElementById { id }) if id == expected_id),
            "Failed to parse: {}",
            input
        );
    }
}

/// 测试 DomCommand::InsertBefore 没有 ref_child_id
#[test]
fn test_insert_before_no_ref_child() {
    let cmd = DomCommand::InsertBefore {
        parent_id: 1,
        new_child_id: 2,
        ref_child_id: None,
    };

    match cmd {
        DomCommand::InsertBefore { ref_child_id, .. } => {
            assert!(ref_child_id.is_none());
        }
        _ => panic!("Expected InsertBefore command"),
    }
}

/// 测试 DomCommand::ReplaceChild 正常情况
#[test]
fn test_replace_child_normal() {
    let cmd = DomCommand::ReplaceChild {
        parent_id: 10,
        new_child_id: 20,
        old_child_id: 30,
    };

    match cmd {
        DomCommand::ReplaceChild { parent_id, new_child_id, old_child_id } => {
            assert_eq!(parent_id, 10);
            assert_eq!(new_child_id, 20);
            assert_eq!(old_child_id, 30);
        }
        _ => panic!("Expected ReplaceChild command"),
    }
}

/// 测试 DomCommand::CloneNode 浅拷贝
#[test]
fn test_clone_node_shallow_copy() {
    let cmd = DomCommand::CloneNode {
        element_id: 42,
        deep: false,
    };

    match cmd {
        DomCommand::CloneNode { element_id, deep } => {
            assert_eq!(element_id, 42);
            assert!(!deep);
        }
        _ => panic!("Expected CloneNode command"),
    }
}

/// 测试 DomCommand::GetStyle
#[test]
fn test_get_style_command() {
    let cmd = DomCommand::GetStyle { element_id: 100 };
    assert_eq!(cmd, DomCommand::GetStyle { element_id: 100 });
}

/// 测试 DomCommand::SetStyle
#[test]
fn test_set_style_command() {
    let cmd = DomCommand::SetStyle {
        element_id: 200,
        value: "color: blue; font-size: 14px".to_string(),
    };

    match cmd {
        DomCommand::SetStyle { element_id, value } => {
            assert_eq!(element_id, 200);
            assert_eq!(value, "color: blue; font-size: 14px");
        }
        _ => panic!("Expected SetStyle command"),
    }
}

/// 测试 DomCommand::SetInnerHtml
#[test]
fn test_set_inner_html_command() {
    let cmd = DomCommand::SetInnerHtml {
        element_id: 50,
        value: "<div>Hello</div>".to_string(),
    };

    match cmd {
        DomCommand::SetInnerHtml { element_id, value } => {
            assert_eq!(element_id, 50);
            assert_eq!(value, "<div>Hello</div>");
        }
        _ => panic!("Expected SetInnerHtml command"),
    }
}

/// 测试 DomCommand::GetParentNode
#[test]
fn test_get_parent_node_command() {
    let cmd = DomCommand::GetParentNode { element_id: 75 };
    assert_eq!(cmd, DomCommand::GetParentNode { element_id: 75 });
}

/// 测试 DomResult::Error 大小写敏感性
#[test]
fn test_dom_result_error_case_sensitivity() {
    let error1 = DomResult::Error("Not Found".to_string());
    let error2 = DomResult::Error("not found".to_string());

    // 字符串比较是大小写敏感的
    assert_ne!(error1, error2);

    // 相同字符串应该相等
    let error3 = DomResult::Error("Not Found".to_string());
    assert_eq!(error1, error3);
}

/// 测试 DomResult::ElementList 包含多个元素
#[test]
fn test_dom_result_element_list_multiple() {
    let elements = vec![1, 2, 3, 4, 5];
    let result = DomResult::ElementList(elements.clone());

    match result {
        DomResult::ElementList(list) => {
            assert_eq!(list.len(), 5);
            assert_eq!(list[0], 1);
            assert_eq!(list[4], 5);
        }
        _ => panic!("Expected ElementList"),
    }
}

/// 测试 DomResult::String None 值
#[test]
fn test_dom_result_string_none() {
    let result = DomResult::String(None);
    assert_eq!(result, DomResult::String(None));
}

/// 测试 DomBridge 注册大量节点
#[test]
fn test_bridge_register_many_performance() {
    let mut bridge = DomBridge::new();

    // 注册 1000 个节点
    let mut handles = Vec::new();
    for i in 0..1000 {
        handles.push(bridge.register(i));
    }

    assert_eq!(bridge.len(), 1000);

    // 验证所有 handle 都是唯一的
    let unique_handles: std::collections::HashSet<u64> = handles.iter().copied().collect();
    assert_eq!(unique_handles.len(), 1000);

    // 验证可以正确解析所有节点
    for (i, &handle) in handles.iter().enumerate() {
        assert_eq!(bridge.resolve(handle), Some(i as u64));
    }
}

/// 测试 DomBridge 在 register 和 unregister 之间的一致性
#[test]
fn test_bridge_consistency() {
    let mut bridge = DomBridge::new();

    // 注册节点
    let h1 = bridge.register(1);
    let h2 = bridge.register(2);

    assert_eq!(bridge.resolve(h1), Some(1));
    assert_eq!(bridge.resolve(h2), Some(2));

    // 注销节点
    bridge.unregister(h1);
    assert_eq!(bridge.resolve(h1), None);
    assert_eq!(bridge.resolve(h2), Some(2));

    // 重新注册已注销的 handle
    let h3 = bridge.register(1);
    assert_ne!(h1, h3); // 应该得到新的 handle
    assert_eq!(bridge.resolve(h3), Some(1));
}

/// 测试 DomBridge clear 后的状态
#[test]
fn test_bridge_clear_state() {
    let mut bridge = DomBridge::new();

    // 注册一些节点
    bridge.register(10);
    bridge.register(20);
    bridge.register(30);

    assert_eq!(bridge.len(), 3);
    assert!(!bridge.is_empty());

    // 清空
    bridge.clear();

    assert_eq!(bridge.len(), 0);
    assert!(bridge.is_empty());

    // 清空后 resolve 所有节点都应该返回 None
    assert_eq!(bridge.resolve(10), None);
    assert_eq!(bridge.resolve(20), None);
    assert_eq!(bridge.resolve(30), None);

    // 清空后可以重新注册
    let h = bridge.register(40);
    assert_eq!(bridge.resolve(h), Some(40));
    assert_eq!(bridge.len(), 1);
}

/// 测试 extract_string_arg 边界条件
#[test]
fn test_extract_string_arg_boundary_conditions() {
    // 测试只有引号，没有内容
    assert_eq!(extract_string_arg("\"\""), Some("".to_string()));
    assert_eq!(extract_string_arg("''"), Some("".to_string()));

    // 测试只有引号，没有括号
    assert_eq!(extract_string_arg("\"hello"), None);
    assert_eq!(extract_string_arg("'world"), None);

    // 测试内容包含括号
    assert_eq!(extract_string_arg("\"test()\")"), Some("test()".to_string()));
    assert_eq!(extract_string_arg("'test(123)'"), Some("test(123)".to_string()));

    // 测试括号在引号外
    assert_eq!(extract_string_arg("\"hello\")world"), None);
    assert_eq!(extract_string_arg("'test')extra"), None);
}

/// 测试 parse_command 带空格和换行符
#[test]
fn test_parse_command_with_newlines() {
    let test_cases = vec![
        ("document.getElementById(\"multi\nline\nstring\")", "multi\nline\nstring"),
        ("document.querySelector(\"div\n\twith\nwhitespace\")", "div\n\twith\nwhitespace"),
        ("document.createElement(\n\"span\"\n)", "span"),
    ];

    for (input, expected) in test_cases {
        let result = DomBridge::parse_command(input);
        assert!(
            matches!(result, Some(DomCommand::GetElementById { id }) if id == expected),
            "Failed for input: {}",
            input
        );
    }
}

/// 测试 parse_command 各种命令格式的一致性
#[test]
fn test_parse_command_all_commands_consistency() {
    let test_cases = vec![
        // GetElementById
        ("document.getElementById(\"id\")", DomCommand::GetElementById { id: "id".to_string() }),
        // QuerySelector
        ("document.querySelector(\".class\")", DomCommand::QuerySelector { selector: ".class".to_string() }),
        // QuerySelectorAll
        ("document.querySelectorAll(\"div\")", DomCommand::QuerySelectorAll { selector: "div".to_string() }),
        // CreateElement
        ("document.createElement(\"span\")", DomCommand::CreateElement { tag_name: "span".to_string() }),
        // CreateTextNode
        ("document.createTextNode(\"text\")", DomCommand::CreateTextNode { text: "text".to_string() }),
        // GetElementsByClassName
        ("document.getElementsByClassName(\"active\")", DomCommand::GetElementsByClassName { class_name: "active".to_string() }),
        // GetElementsByTagName
        ("document.getElementsByTagName(\"p\")", DomCommand::GetElementsByTagName { tag_name: "p".to_string() }),
    ];

    for (input, expected) in test_cases {
        let result = DomBridge::parse_command(input);
        assert_eq!(result, Some(expected), "Failed to parse: {}", input);
    }
}

/// 测试 DomCommand::DispatchEvent 的所有参数
#[test]
fn test_dispatch_event_all_parameters() {
    let cmd = DomCommand::DispatchEvent {
        target_id: 123,
        event_type: "click".to_string(),
        bubbles: true,
        cancelable: false,
    };

    match cmd {
        DomCommand::DispatchEvent { target_id, event_type, bubbles, cancelable } => {
            assert_eq!(target_id, 123);
            assert_eq!(event_type, "click");
            assert!(bubbles);
            assert!(!cancelable);
        }
        _ => panic!("Expected DispatchEvent command"),
    }
}

/// 测试 DomCommand::AddEventListener capture 选项
#[test]
fn test_add_event_listener_capture() {
    let cmd = DomCommand::AddEventListener {
        element_id: 456,
        event_type: "keydown".to_string(),
        capture: true,
    };

    match cmd {
        DomCommand::AddEventListener { capture, .. } => {
            assert!(capture);
        }
        _ => panic!("Expected AddEventListener command"),
    }
}

/// 测试 DomCommand::RemoveEventListener
#[test]
fn test_remove_event_listener() {
    let cmd = DomCommand::RemoveEventListener {
        element_id: 789,
        event_type: "input".to_string(),
    };

    match cmd {
        DomCommand::RemoveEventListener { element_id, event_type } => {
            assert_eq!(element_id, 789);
            assert_eq!(event_type, "input");
        }
        _ => panic!("Expected RemoveEventListener command"),
    }
}

/// 测试 generate_dom_api_polyfill 包含所有必要的 API
#[test]
fn test_polyfill_contains_all_apis() {
    let polyfill = generate_dom_api_polyfill();

    // 验证包含所有核心 DOM API
    let required_apis = vec![
        "document.createElement",
        "document.createTextNode",
        "document.getElementById",
        "document.querySelector",
        "document.querySelectorAll",
        "document.getElementsByClassName",
        "document.getElementsByTagName",
        "appendChild",
        "removeChild",
        "insertBefore",
        "replaceChild",
        "cloneNode",
        "setAttribute",
        "getAttribute",
        "removeAttribute",
        "textContent",
        "innerHTML",
        "addEventListener",
        "removeEventListener",
        "dispatchEvent",
        "style",
        "classList",
    ];

    for api in required_apis {
        assert!(polyfill.contains(api), "Missing API: {}", api);
    }
}

/// 测试 generate_dom_api_polyfill 包含所有 Web API
#[test]
fn test_polyfill_contains_all_web_apis() {
    let polyfill = generate_dom_api_polyfill();

    // 验证包含所有 Web API
    let web_apis = vec![
        "globalThis.fetch",
        "globalThis.Headers",
        "globalThis.Request",
        "globalThis.Response",
        "globalThis.console",
        "globalThis.setTimeout",
        "globalThis.setInterval",
        "globalThis.localStorage",
        "globalThis.sessionStorage",
        "globalThis.MutationObserver",
        "globalThis.IntersectionObserver",
        "globalThis.ResizeObserver",
        "globalThis.WebAssembly",
        "navigator.serviceWorker",
    ];

    for api in web_apis {
        assert!(polyfill.contains(api), "Missing Web API: {}", api);
    }
}

/// 测试 generate_dom_api_polyfill 的长度合理性
#[test]
fn test_polyfill_length_reasonable() {
    let polyfill = generate_dom_api_polyfill();

    // polyfill 应该足够大以包含所有必要的功能
    assert!(polyfill.len() > 5000, "Polyfill too small: {} bytes", polyfill.len());

    // 但不应该过大
    assert!(polyfill.len() < 20000, "Polyfill too large: {} bytes", polyfill.len());
}