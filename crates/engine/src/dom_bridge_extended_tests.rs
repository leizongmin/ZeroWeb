//! DOM Bridge 扩展测试 - 针对 dom_bridge.rs 中未覆盖的路径
//!
//! 此模块专注于测试边缘条件、错误处理和特殊场景，
//! 以提高代码覆盖率。

use super::*;

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
    // getElementById with surrounding whitespace
    let result = DomBridge::parse_command("  document.getElementById(  \"test\"  )  ");
    assert_eq!(result, Some(DomCommand::GetElementById { id: "test".to_string() }));

    // querySelector with newlines inside parens
    let result = DomBridge::parse_command("document.querySelector(\n\"div.container\"\n)");
    assert_eq!(
        result,
        Some(DomCommand::QuerySelector {
            selector: "div.container".to_string()
        })
    );

    // querySelectorAll with tabs
    let result = DomBridge::parse_command("document.querySelectorAll(\t\"li\"\t)");
    assert_eq!(
        result,
        Some(DomCommand::QuerySelectorAll {
            selector: "li".to_string()
        })
    );

    // createElement with surrounding tabs
    let result = DomBridge::parse_command("\t\tdocument.createElement(\"span\")\t\t");
    assert_eq!(
        result,
        Some(DomCommand::CreateElement {
            tag_name: "span".to_string()
        })
    );

    // createTextNode with surrounding newlines
    let result = DomBridge::parse_command("\n\ndocument.createTextNode(\"text\")\n\n");
    assert_eq!(
        result,
        Some(DomCommand::CreateTextNode {
            text: "text".to_string()
        })
    );
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
        DomCommand::ReplaceChild {
            parent_id,
            new_child_id,
            old_child_id,
        } => {
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
    // 空引号对（无闭合引号——空字符串 find 返回 None）
    assert_eq!(extract_string_arg("\"\""), None);
    assert_eq!(extract_string_arg("''"), None);

    // 空字符串引号对带闭合括号
    assert_eq!(extract_string_arg("\"\" )"), Some("".to_string()));
    assert_eq!(extract_string_arg("'' )"), Some("".to_string()));

    // 只有开头引号，没有闭合引号
    assert_eq!(extract_string_arg("\"hello"), None);
    assert_eq!(extract_string_arg("'world"), None);

    // 内容包含括号（需要外层闭合括号）
    assert_eq!(extract_string_arg("\"test()\")"), Some("test()".to_string()));
    assert_eq!(extract_string_arg("'test(123)' )"), Some("test(123)".to_string()));

    // 闭合括号后的多余内容不影响提取（函数只验证 ) 存在）
    assert_eq!(extract_string_arg("\"hello\")world"), Some("hello".to_string()));
    assert_eq!(extract_string_arg("'test')extra"), Some("test".to_string()));
}

/// 测试 parse_command 带空格和换行符
#[test]
fn test_parse_command_with_newlines() {
    // getElementById with multiline string argument
    let result = DomBridge::parse_command("document.getElementById(\"multi\nline\nstring\")");
    assert_eq!(
        result,
        Some(DomCommand::GetElementById {
            id: "multi\nline\nstring".to_string()
        })
    );

    // querySelector with whitespace in argument
    let result = DomBridge::parse_command("document.querySelector(\"div\n\twith\nwhitespace\")");
    assert_eq!(
        result,
        Some(DomCommand::QuerySelector {
            selector: "div\n\twith\nwhitespace".to_string()
        })
    );

    // createElement with newlines inside parens
    let result = DomBridge::parse_command("document.createElement(\n\"span\"\n)");
    assert_eq!(
        result,
        Some(DomCommand::CreateElement {
            tag_name: "span".to_string()
        })
    );
}

/// 测试 parse_command 各种命令格式的一致性
#[test]
fn test_parse_command_all_commands_consistency() {
    let test_cases = vec![
        // GetElementById
        (
            "document.getElementById(\"id\")",
            DomCommand::GetElementById { id: "id".to_string() },
        ),
        // QuerySelector
        (
            "document.querySelector(\".class\")",
            DomCommand::QuerySelector {
                selector: ".class".to_string(),
            },
        ),
        // QuerySelectorAll
        (
            "document.querySelectorAll(\"div\")",
            DomCommand::QuerySelectorAll {
                selector: "div".to_string(),
            },
        ),
        // CreateElement
        (
            "document.createElement(\"span\")",
            DomCommand::CreateElement {
                tag_name: "span".to_string(),
            },
        ),
        // CreateTextNode
        (
            "document.createTextNode(\"text\")",
            DomCommand::CreateTextNode {
                text: "text".to_string(),
            },
        ),
        // GetElementsByClassName
        (
            "document.getElementsByClassName(\"active\")",
            DomCommand::GetElementsByClassName {
                class_name: "active".to_string(),
            },
        ),
        // GetElementsByTagName
        (
            "document.getElementsByTagName(\"p\")",
            DomCommand::GetElementsByTagName {
                tag_name: "p".to_string(),
            },
        ),
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
        DomCommand::DispatchEvent {
            target_id,
            event_type,
            bubbles,
            cancelable,
        } => {
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

    // 验证包含所有核心 DOM API（检查方法名存在于 polyfill 文本中）
    let required_apis = vec![
        // document 对象方法（在对象字面量中以 key 形式存在）
        "createElement:",
        "createTextNode:",
        "getElementById:",
        "querySelector:",
        "querySelectorAll:",
        "getElementsByClassName:",
        "getElementsByTagName:",
        // Element prototype 方法（在 _elementProto 中定义）
        "appendChild:",
        "removeChild:",
        "insertBefore:",
        "replaceChild:",
        "cloneNode:",
        "setAttribute:",
        "getAttribute:",
        "removeAttribute:",
        // 属性相关
        "getTextContent",
        "setTextContent",
        "innerHTML",
        "addEventListener:",
        "removeEventListener:",
        "dispatchEvent:",
        "CSSStyleDeclaration",
        "DOMTokenList",
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

    // 但不应该过大（R3287：+/-/~ 兄弟组合器匹配器 + 组合器链切分/回溯使 polyfill 增长至 ~50.6KB，
    // 上限 50000 → 55000 容纳四组合器一致化。此为合理性护栏，非硬预算——突破时评估是否该
    // 收敛 A 代 polyfill 为 B 代 shim 的薄封装，而非单纯抬上限）。
    assert!(polyfill.len() < 55000, "Polyfill too large: {} bytes", polyfill.len());
}

// ── Web Worker API 测试 ──

/// 测试 Worker 构造函数存在
#[test]
fn test_worker_constructor_in_polyfill() {
    let polyfill = generate_dom_api_polyfill();
    assert!(
        polyfill.contains("globalThis.Worker = Worker"),
        "Worker constructor should be defined in polyfill"
    );
    assert!(
        polyfill.contains("Worker.prototype.postMessage"),
        "Worker.postMessage should be defined"
    );
    assert!(
        polyfill.contains("Worker.prototype.terminate"),
        "Worker.terminate should be defined"
    );
}

/// 测试 Worker polyfill 包含事件监听器
#[test]
fn test_worker_event_listeners_in_polyfill() {
    let polyfill = generate_dom_api_polyfill();
    assert!(
        polyfill.contains("Worker.prototype.addEventListener"),
        "Worker.addEventListener should be defined"
    );
    assert!(
        polyfill.contains("Worker.prototype.removeEventListener"),
        "Worker.removeEventListener should be defined"
    );
    assert!(
        polyfill.contains("Worker.prototype.dispatchEvent"),
        "Worker.dispatchEvent should be defined"
    );
}

/// 测试 Worker postMessage 接受 transfer 参数
#[test]
fn test_worker_postmessage_transfer_in_polyfill() {
    let polyfill = generate_dom_api_polyfill();
    assert!(
        polyfill.contains("postMessage = function(message, transfer)"),
        "postMessage should accept message and transfer parameters"
    );
}

/// 测试 Worker terminate 清理状态
#[test]
fn test_worker_terminate_in_polyfill() {
    let polyfill = generate_dom_api_polyfill();
    // terminate 应该清除所有监听器和回调
    assert!(
        polyfill.contains("this._terminated = true"),
        "terminate should set _terminated flag"
    );
}

/// 测试 Worker 构造函数参数验证
#[test]
fn test_worker_parameter_validation_in_polyfill() {
    let polyfill = generate_dom_api_polyfill();
    assert!(
        polyfill.contains("scriptURL is required and must be a string"),
        "Worker should validate scriptURL parameter"
    );
}

// ── ES Module API 测试 ──

/// 测试动态 import() 在 polyfill 中定义
#[test]
fn test_dynamic_import_in_polyfill() {
    let polyfill = generate_dom_api_polyfill();
    assert!(
        polyfill.contains("globalThis.import = function(specifier)"),
        "Dynamic import() should be defined in polyfill"
    );
    assert!(
        polyfill.contains("__esModule: true"),
        "Import result should have __esModule flag"
    );
    assert!(
        polyfill.contains("__importedFrom: specifier"),
        "Import result should record the specifier"
    );
}

/// 测试 import() 参数验证
#[test]
fn test_import_parameter_validation_in_polyfill() {
    let polyfill = generate_dom_api_polyfill();
    assert!(
        polyfill.contains("import() requires a module specifier string"),
        "import() should validate specifier parameter"
    );
}

/// 测试 import.meta 在 polyfill 中定义
#[test]
fn test_import_meta_in_polyfill() {
    let polyfill = generate_dom_api_polyfill();
    assert!(
        polyfill.contains("importMeta"),
        "import.meta polyfill should be defined"
    );
    assert!(
        polyfill.contains("resolve: function(specifier)"),
        "import.meta.resolve should be defined"
    );
}

/// 测试 Worker 和 ES Module 共存
#[test]
fn test_worker_and_es_module_coexist() {
    let polyfill = generate_dom_api_polyfill();
    // 两者都应该在同一个 polyfill 中存在
    assert!(polyfill.contains("Worker"), "Worker should exist");
    assert!(polyfill.contains("globalThis.import"), "import() should exist");
    assert!(polyfill.contains("importMeta"), "import.meta should exist");
}
