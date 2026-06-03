//! 资源预加载覆盖率提升测试。
//!
!// 专注于测试资源预加载器的边界条件和非常规使用模式。

use super::*;

#[test]
fn test_resource_hint_parse_whitespace_handling() {
    // 测试 rel 属性中的空白字符处理
    assert_eq!(parse_resource_hint("preload"), Some(ResourceHintType::Preload));
    assert_eq!(parse_resource_hint("  PRELOAD  "), Some(ResourceHintType::Preload));
    assert_eq!(parse_resource_hint("\t\tpreload\n"), Some(ResourceHintType::Preload));
    assert_eq!(parse_resource_hint("PRELOAD"), Some(ResourceHintType::Preload));
    assert_eq!(parse_resource_hint("Preload"), Some(ResourceHintType::Preload));
    assert_eq!(parse_resource_hint(""), None);
    assert_eq!(parse_resource_hint("unknown"), None);
    assert_eq!(parse_resource_hint("preload prefetch"), None);  // 多个值不被支持
}

#[test]
fn test_resource_hint_type_from_link_attrs_unknown_rel() {
    // 测试未知的 rel 属性返回 None
    assert_eq!(
        ResourceHint::from_link_attrs("style.css", "stylesheet", Some("style"), false, None),
        None
    );
    assert_eq!(
        ResourceHint::from_link_attrs("script.js", "modulepreload", Some("script"), false, None),
        None
    );
}

#[test]
fn test_resource_hint_type_case_insensitive_as_value() {
    // 测试 as 属性值的大小写不敏感
    assert_eq!(parse_resource_type("SCRIPT"), ResourceType::Script);
    assert_eq!(parse_resource_type("  Script  "), ResourceType::Script);
    assert_eq!(parse_resource_type("script"), ResourceType::Script);
    assert_eq!(parse_resource_type("font"), ResourceType::Font);
    assert_eq!(parse_resource_type("FONT"), ResourceType::Font);
    assert_eq!(parse_resource_type("Font"), ResourceType::Font);
}

#[test]
fn test_resource_hint_from_link_attrs_all_types() {
    // 测试所有资源提示类型的创建
    let preload_hint = ResourceHint::from_link_attrs(
        "app.js",
        "preload",
        Some("script"),
        false,
        None
    ).unwrap();
    assert_eq!(preload_hint.hint_type, ResourceHintType::Preload);
    assert_eq!(preload_hint.resource_type, ResourceType::Script);
    assert_eq!(preload_hint.priority, LoadPriority::High);

    let prefetch_hint = ResourceHint::from_link_attrs(
        "next.js",
        "prefetch",
        None,
        false,
        None
    ).unwrap();
    assert_eq!(prefetch_hint.hint_type, ResourceHintType::Prefetch);
    assert_eq!(prefetch_hint.resource_type, ResourceType::Other);
    assert_eq!(prefetch_hint.priority, LoadPriority::Low);

    let preconnect_hint = ResourceHint::from_link_attrs(
        "https://cdn.example.com",
        "preconnect",
        None,
        false,
        None
    ).unwrap();
    assert_eq!(preconnect_hint.hint_type, ResourceHintType::Preconnect);
    assert_eq!(preconnect_hint.priority, LoadPriority::Medium);

    let dns_hint = ResourceHint::from_link_attrs(
        "https://api.example.com",
        "dns-prefetch",
        None,
        false,
        None
    ).unwrap();
    assert_eq!(dns_hint.hint_type, ResourceHintType::DnsPrefetch);
    assert_eq!(dns_hint.priority, LoadPriority::Low);
}

#[test]
fn test_resource_hint_priority_inference_edge_cases() {
    // 测试优先级推断的边界情况

    // Preload with as="document" (应该为 Critical)
    let doc_hint = ResourceHint::from_link_attrs(
        "page.html",
        "preload",
        Some("document"),
        false,
        None
    ).unwrap();
    assert_eq!(doc_hint.priority, LoadPriority::Critical);

    // Preload with as="embed"
    let embed_hint = ResourceHint::from_link_attrs(
        "object.swf",
        "preload",
        Some("embed"),
        false,
        None
    ).unwrap();
    assert_eq!(embed_hint.priority, LoadPriority::Medium);  // embed 的默认优先级

    // Prefetch 任何资源类型都应该是 Low
    let any_prefetch = ResourceHint::from_link_attrs(
        "data.json",
        "prefetch",
        Some("fetch"),
        false,
        None
    ).unwrap();
    assert_eq!(any_prefetch.priority, LoadPriority::Low);
}

#[test]
fn test_resource_hint_cors_integrity_combinations() {
    // 测试 CORS 和 integrity 属性的各种组合
    let hint1 = ResourceHint::from_link_attrs(
        "script.js",
        "preload",
        Some("script"),
        false,
        None
    ).unwrap();
    assert!(!hint1.cors);
    assert!(hint1.integrity.is_none());

    let hint2 = ResourceHint::from_link_attrs(
        "style.css",
        "preload",
        Some("style"),
        true,
        None
    ).unwrap();
    assert!(hint2.cors);
    assert!(hint2.integrity.is_none());

    let hint3 = ResourceHint::from_link_attrs(
        "font.woff2",
        "preload",
        Some("font"),
        false,
        Some("sha384-abc123")
    ).unwrap();
    assert!(!hint3.cors);
    assert_eq!(hint3.integrity, Some("sha384-abc123".to_string()));

    let hint4 = ResourceHint::from_link_attrs(
        "image.png",
        "preload",
        Some("image"),
        true,
        Some("sha256-def456")
    ).unwrap();
    assert!(hint4.cors);
    assert_eq!(hint4.integrity, Some("sha256-def456".to_string()));
}

#[test]
fn test_resource_preloader_register_duplicate_priority_logic() {
    let mut preloader = ResourcePreloader::new();

    // 先注册为 prefetch (Low)
    preloader.register_link(
        "app.js",
        "prefetch",
        None,
        false,
        None
    );

    // 注册相同的 URL 为 preload script (High) - 应该覆盖
    preloader.register_link(
        "app.js",
        "preload",
        Some("script"),
        false,
        None
    );

    // 注册相同的 URL 为 preload style (Critical) - 应该覆盖
    preloader.register_link(
        "app.js",
        "preload",
        Some("style"),
        false,
        None
    );

    // 最终应该是 Critical 优先级
    let hint = preloader.get("app.js").unwrap();
    assert_eq!(hint.hint_type, ResourceHintType::Preload);
    assert_eq!(hint.resource_type, ResourceType::Style);
    assert_eq!(hint.priority, LoadPriority::Critical);

    // 只有一个条目（已去重）
    assert_eq!(preloader.len(), 1);
}

#[test]
fn test_resource_preloader_register_manual_hint() {
    let mut preloader = ResourcePreloader::new();

    // 手动创建提示并注册
    let hint1 = ResourceHint {
        url: "style.css".to_string(),
        hint_type: ResourceHintType::Preload,
        resource_type: ResourceType::Style,
        priority: LoadPriority::Critical,
        cors: false,
        integrity: None,
        state: ResourceLoadState::Pending,
    };

    let hint2 = ResourceHint {
        url: "app.js".to_string(),
        hint_type: ResourceHintType::Prefetch,
        resource_type: ResourceType::Script,
        priority: LoadPriority::Low,
        cors: true,
        integrity: Some("sha384-abc".to_string()),
        state: ResourceLoadState::Pending,
    };

    preloader.register(hint1.clone());
    preloader.register(hint2.clone());

    assert_eq!(preloader.len(), 2);
    assert_eq!(preloader.get("style.css").unwrap().priority, LoadPriority::Critical);
    assert_eq!(preloader.get("app.js").unwrap().integrity, Some("sha384-abc".to_string()));
}

#[test]
fn test_resource_preloader_register_link_returns_consistent() {
    let mut preloader = ResourcePreloader::new();

    // 有效提示应返回 true
    assert!(preloader.register_link(
        "style.css",
        "preload",
        Some("style"),
        false,
        None
    ));

    // 无效提示应返回 false
    assert!(!preloader.register_link(
        "script.js",
        "unknown",
        Some("script"),
        false,
        None
    ));

    // 再次注册有效提示应返回 true
    assert!(preloader.register_link(
        "script.js",
        "preload",
        Some("script"),
        false,
        None
    ));
}

#[test]
fn test_resource_preloader_pending_resources_sorting() {
    let mut preloader = ResourcePreloader::new();

    // 添加不同优先级的资源
    preloader.register_link("style.css", "preload", Some("style"), false, None);  // Critical
    preloader.register_link("app.js", "preload", Some("script"), false, None);   // High
    preloader.register_link("font.woff2", "preload", Some("font"), false, None); // High
    preloader.register_link("next.js", "prefetch", None, false, None);           // Low
    preloader.register_link("data.json", "prefetch", None, false, None);         // Low

    let pending = preloader.pending_resources();
    assert_eq!(pending.len(), 5);

    // 验证优先级排序：Critical > High > High > Low > Low
    assert_eq!(pending[0].priority, LoadPriority::Critical);  // style.css
    assert_eq!(pending[1].priority, LoadPriority::High);     // app.js
    assert_eq!(pending[2].priority, LoadPriority::High);     // font.woff2
    assert_eq!(pending[3].priority, LoadPriority::Low);      // next.js
    assert_eq!(pending[4].priority, LoadPriority::Low);      // data.json

    // 同优先级的资源按注册顺序或 URL 排序（取决于实现）
    assert_eq!(pending[1].url, "app.js");
    assert_eq!(pending[2].url, "font.woff2");
}

#[test]
fn test_resource_preloader_mark_operations_nonexistent() {
    let mut preloader = ResourcePreloader::new();

    // 对不存在的 URL 进行标记操作
    assert!(!preloader.mark_loading("nonexistent.js"));
    preloader.mark_loaded("nonexistent.js");
    preloader.mark_failed("nonexistent.js");

    // 预加载器仍为空
    assert!(preloader.is_empty());
}

#[test]
fn test_resource_preloader_mark_cycle() {
    let mut preloader = ResourcePreloader::new();
    preloader.register_link("app.js", "preload", Some("script"), false, None);

    // 初始状态：Pending
    let hint = preloader.get("app.js").unwrap();
    assert_eq!(hint.state, ResourceLoadState::Pending);

    // 标记为 Loading
    assert!(preloader.mark_loading("app.js"));
    let hint = preloader.get("app.js").unwrap();
    assert_eq!(hint.state, ResourceLoadState::Loading);

    // 尝试再次标记为 Loading（应失败）
    assert!(!preloader.mark_loading("app.js"));

    // 标记为 Loaded
    preloader.mark_loaded("app.js");
    let hint = preloader.get("app.js").unwrap();
    assert_eq!(hint.state, ResourceLoadState::Loaded);

    // 标记为 Failed
    preloader.register_link("error.js", "preload", Some("script"), false, None);
    preloader.mark_loading("error.js");
    preloader.mark_failed("error.js");
    let hint = preloader.get("error.js").unwrap();
    assert_eq!(hint.state, ResourceLoadState::Failed);

    // 已加载或失败的资源不应出现在 pending 列表中
    let pending = preloader.pending_resources();
    assert_eq!(pending.len(), 0);
}

#[test]
fn test_resource_preloader_clear() {
    let mut preloader = ResourcePreloader::new();

    // 添加多个资源
    preloader.register_link("style.css", "preload", Some("style"), false, None);
    preloader.register_link("app.js", "preload", Some("script"), false, None);
    preloader.register_link("font.woff2", "preload", Some("font"), false, None);

    // 标记一些为已加载
    preloader.mark_loading("style.css");
    preloader.mark_loaded("style.css");

    // 清除所有资源
    preloader.clear();

    assert!(preloader.is_empty());
    assert_eq!(preloader.len(), 0);
    assert!(preloader.get("style.css").is_none());
}

#[test]
fn test_resource_preloader_empty_initially() {
    let preloader = ResourcePreloader::new();
    assert!(preloader.is_empty());
    assert_eq!(preloader.len(), 0);
    assert_eq!(preloader.pending_resources().len(), 0);
    assert!(preloader.get("any.js").is_none());
}

#[test]
fn test_load_priority_ordering_and_display() {
    // 测试优先级的排序和字符串表示
    assert!(LoadPriority::Critical > LoadPriority::High);
    assert!(LoadPriority::High > LoadPriority::Medium);
    assert!(LoadPriority::Medium > LoadPriority::Low);
    assert!(LoadPriority::Low > LoadPriority::Idle);

    // 测试 Display trait
    assert_eq!(LoadPriority::Critical.to_string(), "critical");
    assert_eq!(LoadPriority::High.to_string(), "high");
    assert_eq!(LoadPriority::Medium.to_string(), "medium");
    assert_eq!(LoadPriority::Low.to_string(), "low");
    assert_eq!(LoadPriority::Idle.to_string(), "idle");
}

#[test]
fn test_resource_hint_type_display() {
    // 测试资源提示类型的字符串表示
    assert_eq!(ResourceHintType::Preload.to_string(), "preload");
    assert_eq!(ResourceHintType::Prefetch.to_string(), "prefetch");
    assert_eq!(ResourceHintType::Preconnect.to_string(), "preconnect");
    assert_eq!(ResourceHintType::DnsPrefetch.to_string(), "dns-prefetch");
}

#[test]
fn test_resource_type_display() {
    // 测试资源类型的字符串表示
    assert_eq!(ResourceType::Script.to_string(), "script");
    assert_eq!(ResourceType::Style.to_string(), "style");
    assert_eq!(ResourceType::Image.to_string(), "image");
    assert_eq!(ResourceType::Font.to_string(), "font");
    assert_eq!(ResourceType::Audio.to_string(), "audio");
    assert_eq!(ResourceType::Video.to_string(), "video");
    assert_eq!(ResourceType::Fetch.to_string(), "fetch");
    assert_eq!(ResourceType::Document.to_string(), "document");
    assert_eq!(ResourceType::Embed.to_string(), "embed");
    assert_eq!(ResourceType::Other.to_string(), "other");
}

#[test]
fn test_resource_load_state_equality() {
    // 测试加载状态的相等性
    assert_eq!(ResourceLoadState::Pending, ResourceLoadState::Pending);
    assert_eq!(ResourceLoadState::Loading, ResourceLoadState::Loading);
    assert_eq!(ResourceLoadState::Loaded, ResourceLoadState::Loaded);
    assert_ne!(ResourceLoadState::Pending, ResourceLoadState::Loading);
    assert_ne!(ResourceLoadState::Loading, ResourceLoadState::Failed);
}

#[test]
fn test_resource_hint_debug_format() {
    // 测试调试格式
    let hint = ResourceHint {
        url: "test.js".to_string(),
        hint_type: ResourceHintType::Preload,
        resource_type: ResourceType::Script,
        priority: LoadPriority::High,
        cors: true,
        integrity: Some("sha384-test".to_string()),
        state: ResourceLoadState::Pending,
    };

    let debug_str = format!("{:?}", hint);
    assert!(debug_str.contains("url: \"test.js\""));
    assert!(debug_str.contains("hint_type: Preload"));
    assert!(debug_str.contains("resource_type: Script"));
    assert!(debug_str.contains("priority: High"));
    assert!(debug_str.contains("cors: true"));
    assert!(debug_str.contains("integrity: Some(\"sha384-test\")"));
    assert!(debug_str.contains("state: Pending"));
}

#[test]
fn test_resource_hint_clone() {
    // 测试资源提示的克隆
    let original = ResourceHint {
        url: "test.js".to_string(),
        hint_type: ResourceHintType::Preload,
        resource_type: ResourceType::Script,
        priority: LoadPriority::High,
        cors: true,
        integrity: Some("sha384-test".to_string()),
        state: ResourceLoadState::Pending,
    };

    let cloned = original.clone();
    assert_eq!(original.url, cloned.url);
    assert_eq!(original.hint_type, cloned.hint_type);
    assert_eq!(original.resource_type, cloned.resource_type);
    assert_eq!(original.priority, cloned.priority);
    assert_eq!(original.cors, cloned.cors);
    assert_eq!(original.integrity, cloned.integrity);
    assert_eq!(original.state, cloned.state);
}

#[test]
fn test_resource_preloader_large_number_of_hints() {
    let mut preloader = ResourcePreloader::new();

    // 注册大量资源
    for i in 0..100 {
        let resource_type = match i % 10 {
            0 => "script",
            1 => "style",
            2 => "image",
            3 => "font",
            4 => "audio",
            5 => "video",
            6 => "fetch",
            7 => "document",
            8 => "embed",
            _ => "other",
        };

        preloader.register_link(
            &format!("resource{}.{}", i, resource_type),
            if i % 2 == 0 { "preload" } else { "prefetch" },
            Some(resource_type),
            i % 3 == 0,  // CORS
            if i % 5 == 0 { Some("sha384-hash") } else { None }
        );
    }

    assert_eq!(preloader.len(), 100);
    let pending = preloader.pending_resources();
    assert_eq!(pending.len(), 100);

    // 验证排序（按优先级）
    let mut priorities: Vec<_> = pending.iter().map(|h| h.priority).collect();
    let mut sorted_priorities = priorities.clone();
    sorted_priorities.sort_by_key(|p| std::cmp::Reverse(*p));
    assert_eq!(priorities, sorted_priorities);
}

#[test]
fn test_resource_hint_with_empty_url() {
    // 测试空 URL 的情况
    let result = ResourceHint::from_link_attrs(
        "",
        "preload",
        Some("script"),
        false,
        None
    );
    // 空 URL 是有效的（尽管可能没有实际意义）
    assert!(result.is_some());
    let hint = result.unwrap();
    assert_eq!(hint.url, "");
    assert_eq!(hint.hint_type, ResourceHintType::Preload);
}

#[test]
fn test_resource_hint_same_different_priority() {
    let mut preloader = ResourcePreloader::new();

    // 注册相同 URL，不同优先级
    preloader.register_link("app.js", "prefetch", None, false, None);  // Low
    preloader.register_link("app.js", "preload", Some("script"), false, None); // High
    preloader.register_link("app.js", "preload", Some("style"), false, None);  // Critical

    // 应该保留最高优先级
    let hint = preloader.get("app.js").unwrap();
    assert_eq!(hint.priority, LoadPriority::Critical);
    assert_eq!(hint.resource_type, ResourceType::Style);
}