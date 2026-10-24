// T5/P10 宿主历史桥测试（part01-83 满载后扩展，控制单文件 <2000 行）。经 js_dom_bridge_tests.rs include。

#[test]
fn test_history_traverse_bridge_hands_off_t5() {
    // T5/P10：history.back()/forward()/go(delta) 在 shim 本地栈越界时经 HistoryBridge
    // __zw_request_history_traverse 移交宿主（spec：页面 back()/forward() 遍历 traversable
    // navigable 的会话历史，跨文档 entry 不在每文档本地栈）。有本地 entry 时仍本地消费
    //（pushState/hash 面），不投递。spec
    // https://html.spec.whatwg.org/multipage/nav-history-apis.html#the-history-interface
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    let hist_bridge = crate::HistoryBridge::new();
    let hist_queue = hist_bridge.queue();
    hist_bridge.register(&mut sandbox);
    let drain = || hist_queue.lock().map(|mut q| std::mem::take(&mut *q)).unwrap_or_default();

    // ① 初始单 entry：back() 越界 → 移交 delta=-1。
    sandbox.execute("history.back();").unwrap();
    assert_eq!(drain(), vec![-1], "初始单 entry history.back() 越界 → 移交 -1");

    // ② forward() 同理越界（cursor 0，无前向 entry）→ 移交 +1。
    sandbox.execute("history.forward();").unwrap();
    assert_eq!(drain(), vec![1], "初始单 entry history.forward() 越界 → 移交 +1");

    // ③ go(-2) 越界 → 移交原始 delta（判界在宿主）。
    sandbox.execute("history.go(-2);").unwrap();
    assert_eq!(drain(), vec![-2], "history.go(-2) 越界 → 移交 -2");

    // ④ 有本地 entry：pushState 后 back() 本地消费（cursor 1→0），不投递。
    sandbox.execute("history.pushState(null, '', '/spa');").unwrap();
    sandbox.execute("history.back();").unwrap();
    assert!(drain().is_empty(), "本地 entry 可达的 back() 本地消费 → 不移交");

    // ⑤ 本地栈耗尽再 back()：本地 cursor 已在 0，越界 → 移交。
    sandbox.execute("history.back();").unwrap();
    assert_eq!(drain(), vec![-1], "本地栈耗尽后 back() 越界 → 移交 -1");

    // ⑥ 同文档遍历到位后宿主重置（真导航 __zw_reset_history 契约）→ 单 entry 重建，移交恢复。
    sandbox.execute("__zw_reset_history && __zw_reset_history();").unwrap();
    assert!(drain().is_empty(), "重置不产生遍历请求");
    sandbox.execute("history.forward();").unwrap();
    assert_eq!(drain(), vec![1], "重置后 forward() 越界 → 移交 +1");
}

#[test]
fn test_history_traverse_no_bridge_noop_t5() {
    // T5/P10 零漂移守卫：无桥环境（headless/WPT/单测 sandbox 不注册 __zw_request_history_traverse）
    // 越界 back()/go() 保持 R3004 静默 no-op——不 panic、location/length 不变、队列概念不存在。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/page".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 无桥 feature-detect 为假：越界 back/go 静默（R3004 语义不变）。
    sandbox
        .execute(
            "globalThis.__len0 = history.length;\
             history.back();\
             history.go(-2);\
             globalThis.__len1 = history.length;\
             globalThis.__href1 = location.href;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len0)").unwrap().value, "1");
    assert_eq!(sandbox.execute("String(globalThis.__len1)").unwrap().value, "1", "无桥越界 back/go 后 length 不变（R3004 no-op）");
    assert_eq!(
        sandbox.execute("globalThis.__href1").unwrap().value,
        "https://example.com/page",
        "无桥越界 back/go 后 location 不变"
    );
}
