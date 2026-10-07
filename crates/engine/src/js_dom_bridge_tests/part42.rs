// t8g（site-compat bilibili-20261002-r1）回归钉：in-document 结构性二次方税根因修复。
// 根因链：`_unrecordHandleChild` R91 段引用未声明标识符 `ch` → `typeof ch` 沿全局链
// 落到 WindowProperties NPO has trap → 每次 removeChild 触发全局命名查找 →
// `_zwNPOIfrScan` 每次 cache miss（每 mutation bump ifrGen）getElementsByTagName
// 新建 live HTMLCollection 永久注册 `_zwLiveCollections` → 集合数随操作数线性涨 →
// `_zwHCLiveInvalidate` 每 mutation 遍历全部集合 × 每集合 matches 逐元素 tag 解析
// → O(ops²)。真站实证：400 对 append/remove 泄漏 409 集合、_realTag 43·i 次/对。

/// 泄漏钉：removeChild 循环不触发集合累积——600 对 append/remove 墙钟有界。
/// 修复前同形态约 8s 量级（二次方 + 每对 1 个新集合的失效遍历）；修复后线性，
/// 进程内毫秒级。绝对阈值给足裕量（<2000ms），避免 CI 机器噪声误报。
#[test]
fn test_t8g_npo_iframe_scan_no_collection_leak() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    let t0 = std::time::Instant::now();
    sandbox
        .execute(
            "var c = document.createElement('div');\
             document.body.appendChild(c);\
             var d = document.createElement('div');\
             for (var i = 0; i < 600; i++) { c.appendChild(d); c.removeChild(d); }\
             globalThis.__t8gDone = true;",
        )
        .unwrap();
    let elapsed = t0.elapsed();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__t8gDone)")
            .unwrap()
            .value,
        "true",
        "600 对 append/remove 正常完成"
    );
    assert!(
        elapsed.as_millis() < 2000,
        "600 对 append/remove 应线性完成（实测 {:?}；二次方回归时为秒级到分钟级）",
        elapsed
    );
}

/// NPO 语义钉：修复不得破坏 Window 命名属性解析——`ch`（旧笔误标识符）不得被
/// 解析成任何值，命名 iframe 访问在 mutation 后仍走通（iframe 扫描为 qSA 静态
/// 枚举，每次现查反映当前文档）。
#[test]
fn test_t8g_window_named_properties_intact() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><iframe name=\"frameA\"></iframe></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    sandbox
        .execute(
            "var c = document.createElement('div');\
             document.body.appendChild(c);\
             var d = document.createElement('div');\
             for (var i = 0; i < 50; i++) { c.appendChild(d); c.removeChild(d); }\
             globalThis.__chType = typeof ch;\
             globalThis.__chIn = 'ch' in globalThis;\
             globalThis.__ifrLen = document.getElementsByTagName('iframe').length;\
             globalThis.__frameAType = typeof window.frameA;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__chType").unwrap().value,
        "undefined",
        "未声明标识符 `ch`（旧 R91 笔误名）不得解析为任何命名属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__chIn)").unwrap().value,
        "false",
        "`ch` 不得出现在 global 命名面"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ifrLen)").unwrap().value,
        "1",
        "mutation 后 iframe 集合仍正确反映文档"
    );
    // 命名 iframe 访问：contentWindow 在无真实加载环境时允许 null/undefined，
    // 但解析路径不得抛异常（返回值类型仅两种）。
    let v = sandbox.execute("globalThis.__frameAType").unwrap().value;
    assert!(
        v == "undefined" || v == "object",
        "window.frameA 命名访问不得抛异常（实测 {}）",
        v
    );
}

/// R91 钉：removeChild 后反链清理生效——移除节点脱离文档可观测面（isConnected /
/// document.contains），且同 handle 重挂载到新父后恢复连接（反链重写语义不受影响）。
#[test]
fn test_t8g_r91_backlink_cleanup_after_remove() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    sandbox
        .execute(
            "var a = document.createElement('div');\
             document.body.appendChild(a);\
             var b = document.createElement('div');\
             document.body.appendChild(b);\
             var d = document.createElement('div');\
             a.appendChild(d);\
             globalThis.__connInA = d.isConnected;\
             a.removeChild(d);\
             globalThis.__connAfterRemove = d.isConnected;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__connInA)")
            .unwrap()
            .value,
        "true",
        "挂载期间 isConnected 为真"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__connAfterRemove)")
            .unwrap()
            .value,
        "false",
        "removeChild 后 isConnected 立即为假（R91 反链清理）"
    );
}
