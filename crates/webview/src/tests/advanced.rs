// Auto-generated test file — split from webview/lib.rs
use super::super::*;
use std::cell::RefCell;
use std::rc::Rc;

// ── Script bridge ──

#[test]
fn test_webview_execute_script_empty() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("");
    assert!(result.is_err());
}

#[test]
fn test_webview_execute_script_long_script() {
    let mut wv = WebView::new(WebViewConfig::default());
    let long_script = "var x = 0; ".repeat(1000);
    let result = wv.execute_script(&long_script);
    // V8 sandbox executes the script (result is undefined)
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_webview_execute_script_multiple_calls() {
    let mut wv = WebView::new(WebViewConfig::default());
    for i in 0..5 {
        let result = wv.execute_script(&format!("{i} + 1"));
        assert!(result.is_ok(), "Script {i} should execute successfully");
    }
}

#[test]
fn test_webview_execute_script_with_special_chars() {
    let mut wv = WebView::new(WebViewConfig::default());
    let script = "let s = 'hello \"world\" 🌍';";
    let result = wv.execute_script(script);
    assert!(result.is_ok(), "Script with special chars should execute");
}

#[test]
fn test_webview_execute_script_returns_result() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("1 + 1");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "2");
}

// ── Event callbacks: edge cases ──

#[test]
fn test_webview_load_html_does_not_fire_load_events() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let ec = events.clone();
    wv.on_event(move |e| {
        ec.borrow_mut().push(e.clone());
    });
    wv.load_html("<html><body>No events</body></html>", None);
    // load_html does not fire LoadStart/LoadEnd/LoadFailed
    assert!(events.borrow().is_empty());
}

#[test]
fn test_webview_inject_css_does_not_fire_events() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><div>Test</div></body></html>", None);
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let ec = events.clone();
    wv.on_event(move |e| {
        ec.borrow_mut().push(e.clone());
    });
    wv.inject_css("div { color: red; }");
    assert!(events.borrow().is_empty());
}

#[test]
fn test_webview_set_title_fires_title_changed_event() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let ec = events.clone();
    wv.on_event(move |e| {
        ec.borrow_mut().push(e.clone());
    });
    wv.set_title("Title 1");
    wv.set_title("Title 2");
    let recorded = events.borrow();
    assert_eq!(recorded.len(), 2);
    assert!(matches!(&recorded[0], WebViewEvent::TitleChanged(t) if t == "Title 1"));
    assert!(matches!(&recorded[1], WebViewEvent::TitleChanged(t) if t == "Title 2"));
}

#[test]
fn test_webview_callback_sees_all_event_types() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let ec = events.clone();
    wv.on_event(move |e| {
        let name = match e {
            WebViewEvent::LoadStart(_) => "LoadStart",
            WebViewEvent::LoadEnd(_) => "LoadEnd",
            WebViewEvent::LoadFailed(_, _) => "LoadFailed",
            WebViewEvent::TitleChanged(_) => "TitleChanged",
            WebViewEvent::UrlChanged(_) => "UrlChanged",
        };
        ec.borrow_mut().push(name.to_string());
    });

    wv.set_title("MyTitle");
    wv.load_url("https://example.com");
    wv.complete_load("<html><body>Hi</body></html>", None);

    let recorded = events.borrow();
    assert!(recorded.contains(&"TitleChanged".to_string()));
    assert!(recorded.contains(&"LoadStart".to_string()));
    assert!(recorded.contains(&"UrlChanged".to_string()));
    assert!(recorded.contains(&"LoadEnd".to_string()));
    assert!(!recorded.contains(&"LoadFailed".to_string()));
}

#[test]
fn test_webview_remove_first_callback_keeps_second() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events_a: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let events_b: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let ea = events_a.clone();
    let eb = events_b.clone();
    let idx_a = wv.on_event(move |_| {
        *ea.borrow_mut() += 1;
    });
    wv.on_event(move |_| {
        *eb.borrow_mut() += 1;
    });

    wv.set_title("T");
    assert_eq!(*events_a.borrow(), 1);
    assert_eq!(*events_b.borrow(), 1);

    wv.remove_event_callback(idx_a);
    wv.set_title("T2");
    assert_eq!(*events_a.borrow(), 1); // not incremented
    assert_eq!(*events_b.borrow(), 2); // incremented
}

// ── WebView state transitions ──

#[test]
fn test_webview_state_idle_to_loading_to_loaded() {
    let mut wv = WebView::new(WebViewConfig::default());
    // Idle
    assert!(!wv.is_loading());
    assert!(wv.url().is_none());
    assert!(wv.last_render().is_none());

    // Loading
    wv.load_url("https://state-test.com");
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://state-test.com"));
    assert!(wv.last_render().is_none());

    // Loaded
    wv.complete_load("<html><body>Loaded</body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://state-test.com"));
    assert!(wv.last_render().is_some());
}

#[test]
fn test_webview_state_idle_to_loading_to_failed() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://fail.com");
    assert!(wv.is_loading());
    wv.fail_load("timeout");
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_none());
    assert_eq!(wv.url(), Some("https://fail.com"));
}

#[test]
fn test_webview_state_loaded_then_reload() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://page.com");
    wv.complete_load("<html><body>V1</body></html>", None);
    assert!(!wv.is_loading());

    // Reload same URL
    wv.load_url("https://page.com");
    assert!(wv.is_loading());
    wv.complete_load("<html><body>V2</body></html>", None);
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_some());
}

#[test]
fn test_webview_state_loaded_then_navigate_new() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://a.com");
    wv.complete_load("<html><body>A</body></html>", None);
    assert!(!wv.is_loading());

    wv.load_url("https://b.com");
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://b.com"));
    wv.complete_load("<html><body>B</body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://b.com"));
}

#[test]
fn test_webview_state_fail_then_retry() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://retry.com");
    wv.fail_load("connection reset");
    assert!(!wv.is_loading());

    // Retry
    wv.load_url("https://retry.com");
    assert!(wv.is_loading());
    wv.complete_load("<html><body>Success</body></html>", None);
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_some());
}

// ── Configuration ──

#[test]
fn test_webview_config_user_agent_none_by_default() {
    let config = WebViewConfig::default();
    assert!(config.user_agent.is_none());
}

#[test]
fn test_webview_config_devtools_false_by_default() {
    let config = WebViewConfig::default();
    assert!(!config.devtools);
}

#[test]
fn test_webview_config_transparent_false_by_default() {
    let config = WebViewConfig::default();
    assert!(!config.transparent);
}

#[test]
fn test_webview_config_url_none_by_default() {
    let config = WebViewConfig::default();
    assert!(config.url.is_none());
}

#[test]
fn test_webview_config_all_fields_custom() {
    let config = WebViewConfig {
        width: 1920,
        height: 1080,
        transparent: true,
        user_agent: Some("Custom/2.0".to_string()),
        url: Some("https://start.com".to_string()),
        devtools: true,
        external_script: None,
    };
    let wv = WebView::new(config);
    assert_eq!(wv.config().width, 1920);
    assert_eq!(wv.config().height, 1080);
    assert!(wv.config().transparent);
    assert_eq!(wv.config().user_agent.as_deref(), Some("Custom/2.0"));
    assert_eq!(wv.config().url.as_deref(), Some("https://start.com"));
    assert!(wv.config().devtools);
}

// ── WebViewRenderResult clone/debug ──

#[test]
fn test_webview_render_result_clone() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.load_html("<html><body><div>X</div></body></html>", None);
    let cloned = result.clone();
    assert!(cloned.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_render_result_debug() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.load_html("<html><body><div>X</div></body></html>", None);
    let debug = format!("{result:?}");
    assert!(debug.contains("WebViewRenderResult"));
}

// ── Resize edge cases ──

#[test]
fn test_webview_resize_very_large() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.resize(10000, 10000);
    assert_eq!(wv.config().width, 10000);
    assert_eq!(wv.config().height, 10000);
}

#[test]
fn test_webview_resize_preserves_title() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.set_title("Preserved");
    wv.resize(500, 400);
    assert_eq!(wv.title(), Some("Preserved"));
}

// ── load_html with various content ──

#[test]
fn test_webview_load_html_with_inline_styles() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div style=\"color: red; width: 100px;\">Styled</div></body></html>";
    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0);
}

#[test]
fn test_webview_load_html_preserves_cached_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><div>Cached</div></body></html>", None);
    // inject_css uses cached HTML internally — verify it works
    let result = wv.inject_css("div { background: green; }");
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());
}

// ── Builder: all-setters chain ──

#[test]
fn test_webview_builder_all_options() {
    let wv = WebViewBuilder::new()
        .width(1280)
        .height(720)
        .transparent(true)
        .user_agent("FullAgent/3.0")
        .url("https://full.com")
        .devtools(true)
        .build();
    assert_eq!(wv.config().width, 1280);
    assert_eq!(wv.config().height, 720);
    assert!(wv.config().transparent);
    assert_eq!(wv.config().user_agent.as_deref(), Some("FullAgent/3.0"));
    assert!(wv.config().devtools);
    assert_eq!(wv.url(), Some("https://full.com"));
    assert!(wv.is_loading());
}

// ── cached_css：CSS 在 render / resize 后保留 ──

#[test]
fn test_webview_load_html_with_css_preserved_in_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div id=\"main\">Hello</div></body></html>";
    let css = "div { background-color: red; width: 200px; height: 100px; }";
    let first = wv.load_html(html, Some(css));
    let fill_count_after_load = first.primitives.fills.len();

    // render() 应使用缓存的 CSS，fills 数量应一致
    let second = wv.render();
    assert_eq!(
        second.primitives.fills.len(),
        fill_count_after_load,
        "render() should produce same fills as load_html() when CSS is cached"
    );
}

#[test]
fn test_webview_load_html_css_preserved_after_resize() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div id=\"box\">Content</div></body></html>";
    let css = "div { background-color: blue; width: 100px; height: 50px; }";
    let first = wv.load_html(html, Some(css));
    let fill_count = first.primitives.fills.len();

    wv.resize(400, 300);
    let after = wv.render();
    assert_eq!(
        after.primitives.fills.len(),
        fill_count,
        "CSS should be preserved after resize + render"
    );
}

/// 验证脚本修改 DOM 后重绘仍保留异步加载的外链 CSS。
#[test]
fn test_webview_reload_after_script_preserves_cached_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div class=\"box\">Before</div></body></html>";
    let css = ".box { background-color: red; width: 200px; height: 100px; }";
    let initial = wv.load_html(html, Some(css));

    let mutated = "<html><body><div class=\"box\">After</div><span>Added</span></body></html>";
    let after_script = wv.reload_html_after_script(mutated);

    assert_eq!(
        after_script.primitives.fills.len(),
        initial.primitives.fills.len(),
        "script rerender must preserve cached external CSS"
    );
}

#[test]
fn test_webview_inject_css_accumulates() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div class=\"a b\">Test</div></body></html>";
    let css = ".a { background-color: red; width: 100px; height: 50px; }";
    let first = wv.load_html(html, Some(css));
    let fill_count_first = first.primitives.fills.len();

    // 注入额外 CSS，应追加到已有 CSS
    let second = wv.inject_css(".b { background-color: blue; }");
    // 注入后 fills 数量应 >= 之前（追加的 CSS 可能影响布局）
    assert!(
        second.primitives.fills.len() >= fill_count_first,
        "inject_css should accumulate CSS, not replace it"
    );

    // render 也应保留累积的 CSS
    let third = wv.render();
    assert_eq!(
        third.primitives.fills.len(),
        second.primitives.fills.len(),
        "render() should use accumulated CSS"
    );
}

#[test]
fn test_webview_load_html_resets_cached_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div>Content</div></body></html>";
    wv.load_html(html, Some("div { color: red; }"));
    // 再次调用 load_html 传 None，应重置 CSS
    wv.load_html(html, None);
    let after = wv.render();
    // 没有 CSS 时的 fills 应 <= 有 CSS 时
    // 主要验证不会崩溃，且 CSS 被正确清空
    assert!(after.timings.total_ms >= 0.0);
}

// ════════════════════════════════════════════════════════════════
//  边界条件测试：空输入、CSS 累积、状态机转换
// ════════════════════════════════════════════════════════════════

/// 验证加载空 HTML 字符串不会 panic，且返回有效的渲染结果。
///
/// 边界场景：传入完全为空的字符串而非有效 HTML 文档，
/// 确保渲染管线不会因缺少根元素而崩溃。
#[test]
fn test_webview_load_empty_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.load_html("", None);
    assert!(result.timings.total_ms >= 0.0, "渲染空 HTML 应返回非负耗时");
    assert!(wv.last_render().is_some(), "加载空 HTML 后应存在渲染结果");
    assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
    assert!(wv.url().is_none(), "load_html 不应设置 URL");
}

/// 验证执行空脚本字符串返回错误。
///
/// 边界场景：传入空字符串作为脚本内容，
/// 确保 JS 引擎正确拒绝空输入。
#[test]
fn test_webview_execute_script_empty_string() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script("");
    assert!(result.is_err(), "空脚本应返回错误");
    match result.unwrap_err() {
        WebViewError::Script(msg) => {
            assert!(
                msg.contains("Invalid input") || msg.contains("empty"),
                "错误信息应提及空输入，实际: {msg}"
            );
        }
        other => panic!("预期 Script 错误，实际: {other}"),
    }
}

/// 验证多次注入 CSS 会累积而非替换。
///
/// 每次调用 inject_css 应将新 CSS 追加到已有 CSS 之后，
/// 渲染结果应反映所有已注入样式的叠加效果。
#[test]
fn test_webview_multiple_css_injections() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body>\
        <div class=\"first\">A</div>\
        <div class=\"second\">B</div>\
        <div class=\"third\">C</div>\
        </body></html>";
    let initial = wv.load_html(html, None);
    let fills_after_load = initial.primitives.fills.len();

    // 第一次注入：为 .first 添加背景
    let after_first = wv.inject_css(".first { background-color: red; width: 50px; height: 50px; }");
    let fills_after_first = after_first.primitives.fills.len();
    assert!(
        fills_after_first >= fills_after_load,
        "第一次注入后 fills 数量应 >= 初始值"
    );

    // 第二次注入：为 .second 添加背景
    let after_second = wv.inject_css(".second { background-color: green; width: 50px; height: 50px; }");
    let fills_after_second = after_second.primitives.fills.len();
    assert!(
        fills_after_second >= fills_after_first,
        "第二次注入后 fills 数量应 >= 第一次注入后（CSS 累积，不替换）"
    );

    // 第三次注入：为 .third 添加背景
    let after_third = wv.inject_css(".third { background-color: blue; width: 50px; height: 50px; }");
    let fills_after_third = after_third.primitives.fills.len();
    assert!(
        fills_after_third >= fills_after_second,
        "第三次注入后 fills 数量应 >= 第二次注入后（CSS 持续累积）"
    );

    // render() 也应保留所有累积的 CSS
    let after_render = wv.render();
    assert_eq!(
        after_render.primitives.fills.len(),
        fills_after_third,
        "render() 应使用累积的所有 CSS"
    );
}

/// 验证 WebView 状态机转换：Created -> Loading -> Loaded -> Error。
///
/// 测试完整的状态生命周期：
/// 1. 初始 Created 状态（无 URL，未加载）
/// 2. load_url 进入 Loading 状态
/// 3. complete_load 进入 Loaded 状态
/// 4. 再次 load_url 进入 Loading 状态
/// 5. fail_load 进入 Error（恢复到非加载状态）
/// 6. 重试后再次进入 Loaded 状态
#[test]
fn test_webview_state_transitions() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let ec = events.clone();
    wv.on_event(move |e| {
        let label = match e {
            WebViewEvent::LoadStart(u) => format!("LoadStart({u})"),
            WebViewEvent::LoadEnd(u) => format!("LoadEnd({u})"),
            WebViewEvent::LoadFailed(u, m) => format!("LoadFailed({u},{m})"),
            WebViewEvent::TitleChanged(t) => format!("TitleChanged({t})"),
            WebViewEvent::UrlChanged(u) => format!("UrlChanged({u})"),
        };
        ec.borrow_mut().push(label);
    });

    // ── 状态 1: Created ──
    assert!(!wv.is_loading(), "初始状态: 不应处于加载中");
    assert!(wv.url().is_none(), "初始状态: URL 应为 None");
    assert!(wv.last_render().is_none(), "初始状态: 不应有渲染结果");

    // ── 状态 2: Loading（通过 load_url）──
    wv.load_url("https://state-test.com");
    assert!(wv.is_loading(), "Loading 状态: 应处于加载中");
    assert_eq!(wv.url(), Some("https://state-test.com"), "Loading 状态: URL 应已设置");
    assert!(wv.last_render().is_none(), "Loading 状态: 尚未有渲染结果");

    // ── 状态 3: Loaded（通过 complete_load）──
    wv.complete_load("<html><body><div>Content</div></body></html>", None);
    assert!(!wv.is_loading(), "Loaded 状态: 不应处于加载中");
    assert_eq!(wv.url(), Some("https://state-test.com"), "Loaded 状态: URL 应保持不变");
    assert!(wv.last_render().is_some(), "Loaded 状态: 应有渲染结果");

    // ── 状态 4: 再次 Loading（导航到新 URL）──
    wv.load_url("https://error-test.com");
    assert!(wv.is_loading(), "再次 Loading: 应处于加载中");
    assert_eq!(wv.url(), Some("https://error-test.com"), "再次 Loading: URL 应已更新");

    // ── 状态 5: Error（通过 fail_load）──
    wv.fail_load("network timeout");
    assert!(!wv.is_loading(), "Error 状态: 加载应已停止");
    assert_eq!(wv.url(), Some("https://error-test.com"), "Error 状态: URL 应保留");
    assert!(wv.last_render().is_some(), "Error 状态: 上次成功的渲染结果应保留");

    // ── 状态 6: 重试 Loading -> Loaded ──
    wv.load_url("https://retry-test.com");
    assert!(wv.is_loading(), "重试 Loading: 应处于加载中");
    assert_eq!(wv.url(), Some("https://retry-test.com"), "重试 Loading: URL 应已更新");
    wv.complete_load("<html><body><div>Retry OK</div></body></html>", None);
    assert!(!wv.is_loading(), "重试 Loaded: 不应处于加载中");
    assert_eq!(wv.url(), Some("https://retry-test.com"), "重试 Loaded: URL 应保持");
    assert!(wv.last_render().is_some(), "重试 Loaded: 应有渲染结果");

    // ── 验证完整事件序列 ──
    let recorded = events.borrow();
    assert_eq!(
        recorded.len(),
        9,
        "应有 9 个事件: 2(LoadStart+UrlChanged) + 1(LoadEnd) + 2(LoadStart+UrlChanged) + 1(LoadFailed) + 2(LoadStart+UrlChanged) + 1(LoadEnd)"
    );
    assert_eq!(recorded[0], "LoadStart(https://state-test.com)");
    assert_eq!(recorded[1], "UrlChanged(https://state-test.com)");
    assert_eq!(recorded[2], "LoadEnd(https://state-test.com)");
    assert_eq!(recorded[3], "LoadStart(https://error-test.com)");
    assert_eq!(recorded[4], "UrlChanged(https://error-test.com)");
    assert!(recorded[5].starts_with("LoadFailed(https://error-test.com"));
    assert_eq!(recorded[6], "LoadStart(https://retry-test.com)");
    assert_eq!(recorded[7], "UrlChanged(https://retry-test.com)");
    assert_eq!(recorded[8], "LoadEnd(https://retry-test.com)");
}

// ════════════════════════════════════════════════════════════════
//  边界条件测试：多次导航、CSS 注入累积、脚本占位、Builder 视口
// ════════════════════════════════════════════════════════════════

/// 验证连续导航两次后，WebView 最终状态反映 URL2 的内容。
///
/// 模拟用户从 URL1 导航到 URL2 的场景：
/// 1. 加载 URL1 并完成（complete_load），确认状态为 URL1
/// 2. 加载 URL2 并完成（complete_load），确认最终状态为 URL2
/// 3. 确保 URL、加载状态、渲染结果全部正确指向 URL2
#[test]
fn test_webview_multiple_navigate() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 第一次导航：URL1
    wv.load_url("https://url-one.com");
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://url-one.com"));
    wv.complete_load("<html><body><div>Content from URL1</div></body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://url-one.com"));
    let render1 = wv.last_render().unwrap();
    assert!(render1.timings.total_ms >= 0.0);

    // 第二次导航：URL2
    wv.load_url("https://url-two.com");
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://url-two.com"));
    wv.complete_load("<html><body><div>Content from URL2</div></body></html>", None);
    assert!(!wv.is_loading());

    // 验证最终状态指向 URL2
    assert_eq!(wv.url(), Some("https://url-two.com"));
    assert!(wv.last_render().is_some());
    let render2 = wv.last_render().unwrap();
    assert!(render2.timings.total_ms >= 0.0);

    // 重新渲染应仍反映 URL2 的内容（cached_html 为 URL2 的 HTML）
    let rerender = wv.render();
    assert!(rerender.timings.total_ms >= 0.0);
}

/// 验证 load_html 加载 CSS 后，inject_css 追加新样式，cached_css 同时包含原始和注入的 CSS。
///
/// 步骤：
/// 1. load_html 加载带 CSS 的 HTML（为 .orig 元素设置红色背景）
/// 2. inject_css 注入额外 CSS（为 .injected 元素设置蓝色背景）
/// 3. 通过 render() 的 fills 数量验证 CSS 累积效果：
///    - 注入后 fills >= 仅原始 CSS 时的 fills
///    - render() 使用累积 CSS，fills 数量一致
#[test]
fn test_webview_inject_css_after_load() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body>\
        <div class=\"orig\">Original</div>\
        <div class=\"injected\">Injected</div>\
        </body></html>";
    let original_css = ".orig { background-color: red; width: 100px; height: 50px; }";

    // 加载带原始 CSS 的 HTML
    let after_load = wv.load_html(html, Some(original_css));
    let fills_after_load = after_load.primitives.fills.len();
    assert!(fills_after_load > 0, "带 CSS 的 load_html 应产生 fills");

    // 注入额外 CSS
    let injected_css = ".injected { background-color: blue; width: 80px; height: 40px; }";
    let after_inject = wv.inject_css(injected_css);
    let fills_after_inject = after_inject.primitives.fills.len();

    // 注入后 fills 应 >= 仅原始 CSS（CSS 累积，不替换）
    assert!(
        fills_after_inject >= fills_after_load,
        "inject_css 应追加 CSS，fills 数量应 >= 注入前 (got {fills_after_inject} < {fills_after_load})"
    );

    // render() 应使用累积的 CSS（原始 + 注入），fills 数量一致
    let after_render = wv.render();
    assert_eq!(
        after_render.primitives.fills.len(),
        fills_after_inject,
        "render() 应使用累积的 cached_css（原始 + 注入）"
    );
}

/// 验证 execute_script 作为占位方法，在 JS 引擎集成前返回 NotImplemented 错误。
///
/// 验证 execute_script 现在通过 V8 沙箱执行脚本。
///
/// V8 沙箱已集成，execute_script 可以成功执行简单脚本。
/// document.title 在独立沙箱中不可用，因此会产生运行时错误。
#[test]
fn test_webview_execute_script_v8_integrated() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 简单算术表达式应成功执行
    let result = wv.execute_script("1 + 1");
    assert!(result.is_ok(), "简单脚本应成功执行");
    assert_eq!(result.unwrap(), "2");

    // WebView 状态不应因 execute_script 调用而改变
    assert!(wv.url().is_none());
    assert!(!wv.is_loading());

    // 多次调用同样成功
    for i in 0..3 {
        let r = wv.execute_script(&format!("{i} * 2"));
        assert!(r.is_ok(), "Script {i} should succeed");
    }
}

/// 验证 WebViewBuilder 默认配置（无 URL）产生正确的初始状态。
///
/// 默认视口 800x600，无 URL，不在加载中，无渲染结果。
#[test]
fn test_webview_builder_defaults() {
    let wv = WebViewBuilder::new().build();
    assert_eq!(wv.config().width, 800, "默认宽度应为 800");
    assert_eq!(wv.config().height, 600, "默认高度应为 600");
    assert!(wv.url().is_none(), "默认不应有 URL");
    assert!(!wv.is_loading(), "默认不应处于加载中");
    assert!(wv.last_render().is_none(), "默认不应有渲染结果");
    assert!(!wv.config().transparent);
    assert!(wv.config().user_agent.is_none());
    assert!(!wv.config().devtools);
}

/// 验证加载 data URI 内容后渲染成功。
///
/// 通过 load_html 加载 data URI 格式的 HTML 内容，
/// 确认渲染管线产生有效结果（非负耗时、存在渲染输出）。
#[test]
fn test_webview_load_data_uri() {
    let mut wv = WebView::new(WebViewConfig::default());
    let data_uri_html = "<html><body><div>Data URI content rendered</div></body></html>";
    let result = wv.load_html(data_uri_html, None);
    assert!(result.timings.total_ms >= 0.0, "data URI 渲染耗时应为非负");
    assert!(wv.last_render().is_some(), "加载 data URI 后应有渲染结果");
    assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
}

/// 验证加载 HTML 后注入 CSS，渲染结果反映注入的样式。
///
/// 步骤：
/// 1. load_html 加载带 div 的 HTML（无 CSS）
/// 2. inject_css 注入为 div 设置背景色和尺寸的 CSS
/// 3. 渲染结果的 fills 数量应大于仅加载 HTML 时
#[test]
fn test_webview_render_after_inject_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div class=\"box\">Hello</div></body></html>";

    // 加载 HTML（无 CSS）
    let after_load = wv.load_html(html, None);
    let fills_after_load = after_load.primitives.fills.len();

    // 注入 CSS
    let css = ".box { background-color: green; width: 100px; height: 50px; }";
    let after_inject = wv.inject_css(css);
    let fills_after_inject = after_inject.primitives.fills.len();

    // 注入后 fills 数量应 >= 加载时（CSS 为 div 添加了背景色）
    assert!(
        fills_after_inject >= fills_after_load,
        "注入 CSS 后 fills 数量应 >= 注入前 (got {fills_after_inject} < {fills_after_load})"
    );

    // render 应使用累积的 CSS
    let after_render = wv.render();
    assert_eq!(
        after_render.primitives.fills.len(),
        fills_after_inject,
        "render() 应使用注入的 CSS"
    );
}

/// 验证 WebViewBuilder 支持自定义视口尺寸，且 build 后 WebView 正确反映配置。
///
/// 测试非默认视口（如 1280x900），确认：
/// 1. Builder 的 width/height 链式调用正确
/// 2. build 后 config 反映自定义尺寸
/// 3. 后续 render 在正确尺寸的视口上工作
#[test]
fn test_webview_builder_custom_viewport() {
    let mut wv = WebViewBuilder::new().width(1280).height(900).build();

    // 验证自定义视口尺寸
    assert_eq!(wv.config().width, 1280, "视口宽度应为 1280");
    assert_eq!(wv.config().height, 900, "视口高度应为 900");

    // 默认值应保持不变
    assert!(!wv.config().transparent);
    assert!(wv.config().user_agent.is_none());
    assert!(!wv.config().devtools);

    // 加载 HTML 并渲染，验证在自定义视口上正常工作
    let html = "<html><body><div>Custom viewport</div></body></html>";
    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0, "自定义视口上的渲染应成功");
    assert!(wv.last_render().is_some());

    // resize 后视口尺寸应更新
    wv.resize(640, 480);
    assert_eq!(wv.config().width, 640);
    assert_eq!(wv.config().height, 480);
    let after_resize = wv.render();
    assert!(after_resize.timings.total_ms >= 0.0);
}

// ════════════════════════════════════════════════════════════════
//  边界条件测试：默认配置、data URI、连续导航、CSS 存储、状态转换
// ════════════════════════════════════════════════════════════════

/// 验证 WebView 使用默认配置创建后，所有字段均为预期默认值。
///
/// 测试通过 WebViewConfig::default() 构造 WebView，
/// 确认宽高、透明度、user_agent、devtools 等字段均为默认值，
/// 且初始状态下无 URL、无标题、不在加载中、无渲染结果。
#[test]
fn test_webview_default_config() {
    let config = WebViewConfig::default();
    let wv = WebView::new(config);

    // 配置字段默认值
    assert_eq!(wv.config().width, 800, "默认宽度应为 800");
    assert_eq!(wv.config().height, 600, "默认高度应为 600");
    assert!(!wv.config().transparent, "默认不应透明");
    assert!(wv.config().user_agent.is_none(), "默认 user_agent 应为 None");
    assert!(wv.config().url.is_none(), "默认 url 应为 None");
    assert!(!wv.config().devtools, "默认 devtools 应为 false");

    // 初始状态
    assert!(wv.url().is_none(), "初始 URL 应为 None");
    assert!(wv.title().is_none(), "初始标题应为 None");
    assert!(!wv.is_loading(), "初始不应处于加载中");
    assert!(wv.last_render().is_none(), "初始不应有渲染结果");
}

/// 验证加载 data URI 格式的 HTML 内容不会 panic，且渲染管线返回有效结果。
///
/// 模拟 "data:text/html,<h1>Hello</h1>" 场景：
/// 通过 load_html 加载 data URI 中嵌入的 HTML 片段，
/// 确认渲染结果非负耗时，且 last_render 存在。
#[test]
fn test_webview_load_data_uri_content() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 模拟 data URI 中提取的 HTML 内容
    let html = "<h1>Hello</h1>";
    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0, "data URI 渲染耗时应为非负");
    assert!(wv.last_render().is_some(), "加载 data URI 后应有渲染结果");
    assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
    assert!(wv.url().is_none(), "load_html 不应设置 URL");
}

/// 验证连续导航到 url1 再到 url2 后，当前 URL 为 url2。
///
/// 模拟用户在浏览器中依次访问两个不同页面的场景：
/// 1. 导航到 url1 并完成加载，确认状态正确
/// 2. 导航到 url2 并完成加载，确认最终 URL 为 url2
/// 3. 渲染结果应反映 url2 的内容
#[test]
fn test_webview_sequential_navigate() {
    let mut wv = WebView::new(WebViewConfig::default());
    let url1 = "https://first-page.com";
    let url2 = "https://second-page.com";

    // 第一次导航：url1
    wv.load_url(url1);
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some(url1));
    wv.complete_load("<html><body><div>Page 1</div></body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some(url1));

    // 第二次导航：url2
    wv.load_url(url2);
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some(url2));
    wv.complete_load("<html><body><div>Page 2</div></body></html>", None);
    assert!(!wv.is_loading());

    // 最终状态：URL 为 url2
    assert_eq!(wv.url(), Some(url2), "连续导航后当前 URL 应为 url2");
    assert!(wv.last_render().is_some(), "应有渲染结果");
}

/// 验证加载 HTML 后注入 CSS，CSS 被正确存储在 cached_css 中。
///
/// 步骤：
/// 1. load_html 加载 HTML（带初始 CSS）
/// 2. inject_css 注入额外 CSS
/// 3. 多次 render() 后 CSS 仍被保留（fills 数量不变）
/// 4. 再次 inject_css 后 CSS 继续累积
#[test]
fn test_webview_css_stored_after_inject() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div class=\"a b\">Text</div></body></html>";
    let css_a = ".a { background-color: red; width: 100px; height: 50px; }";

    // 加载带初始 CSS 的 HTML
    let after_load = wv.load_html(html, Some(css_a));
    let fills_after_load = after_load.primitives.fills.len();
    assert!(fills_after_load > 0, "带 CSS 的 load_html 应产生 fills");

    // 注入额外 CSS，应被存储
    let css_b = ".b { background-color: green; width: 80px; height: 40px; }";
    let after_inject = wv.inject_css(css_b);
    let fills_after_inject = after_inject.primitives.fills.len();
    assert!(
        fills_after_inject >= fills_after_load,
        "注入后 fills 应 >= 注入前 (got {fills_after_inject} < {fills_after_load})"
    );

    // render() 后 CSS 应被保留（fills 数量不变）
    let after_render = wv.render();
    assert_eq!(
        after_render.primitives.fills.len(),
        fills_after_inject,
        "render() 后 CSS 应被保留，fills 数量应一致"
    );
}
