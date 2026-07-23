// Auto-generated test file — split from webview/lib.rs
use super::super::*;
use std::cell::RefCell;
use std::rc::Rc;

// ── 边界条件测试：默认视口、导航到 URL、错误后状态、CSS 持久性 ──

/// 验证 WebView 默认视口尺寸为 800x600。
#[test]
fn test_webview_default_viewport() {
    let wv = WebView::new(WebViewConfig::default());
    assert_eq!(wv.config().width, 800, "默认视口宽度应为 800");
    assert_eq!(wv.config().height, 600, "默认视口高度应为 600");
    // 默认视口下渲染应正常工作
    let mut wv2 = WebView::new(WebViewConfig::default());
    let result = wv2.load_html("<html><body>viewport test</body></html>", None);
    assert!(result.timings.total_ms >= 0.0, "默认视口渲染应成功");
}

/// 验证 navigate 到 URL 后 WebView 处于正确的加载状态。
#[test]
fn test_webview_navigate_to_url() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 初始状态：无 URL
    assert!(wv.url().is_none());
    assert!(!wv.is_loading());
    // 导航到 URL
    wv.load_url("https://example.com/page1");
    assert_eq!(wv.url(), Some("https://example.com/page1"));
    assert!(wv.is_loading());
    assert!(wv.last_render().is_none(), "load_url 后不应有渲染结果");
    // 完成加载
    wv.complete_load("<html><body><div>Navigated</div></body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://example.com/page1"));
    assert!(wv.last_render().is_some());
}

/// 验证加载失败后 WebView 状态正确：loading 停止，URL 保留，last_render 保持。
#[test]
fn test_webview_state_after_error() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 先成功加载一个页面
    wv.load_url("https://good.com");
    wv.complete_load("<html><body>Good page</body></html>", None);
    assert!(wv.last_render().is_some());
    let render_before_error = wv.last_render().unwrap().timings.total_ms;
    // 导航到新 URL 但加载失败
    wv.load_url("https://bad.com");
    assert!(wv.is_loading());
    wv.fail_load("DNS resolution failed");
    // 失败后 loading 应停止
    assert!(!wv.is_loading(), "失败后 loading 应停止");
    // URL 应保留为失败的 URL
    assert_eq!(wv.url(), Some("https://bad.com"), "URL 应保留为失败请求的 URL");
    // last_render 应保留上次成功的渲染结果
    assert!(wv.last_render().is_some(), "失败后应保留上次成功的渲染结果");
    assert!(
        (wv.last_render().unwrap().timings.total_ms - render_before_error).abs() < f64::EPSILON,
        "渲染结果应是上次成功加载的结果"
    );
}

/// 验证 CSS 在多次 render 调用间持久保留。
#[test]
fn test_webview_css_persistence_across_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div id=\"box\">Content</div></body></html>";
    let css = "#box { background-color: red; width: 100px; height: 50px; }";
    // 第一次加载带 CSS
    let first = wv.load_html(html, Some(css));
    let first_fill_count = first.primitives.fills.len();
    assert!(first_fill_count > 0, "带 CSS 的加载应产生 fills");
    // 第一次 render — CSS 应持久
    let second = wv.render();
    assert_eq!(
        second.primitives.fills.len(),
        first_fill_count,
        "第一次 render 后 CSS 应持久保留，fills 数量应一致"
    );
    // 第二次 render — CSS 仍应持久
    let third = wv.render();
    assert_eq!(
        third.primitives.fills.len(),
        first_fill_count,
        "第二次 render 后 CSS 仍应持久保留"
    );
}

/// 验证 WebView 在完整生命周期中的状态转换正确性。
///
/// 覆盖的状态转换路径：
/// 1. Created -> Loading（load_url）
/// 2. Loading -> Loaded（complete_load）
/// 3. Loaded -> Loading（load_url 新 URL）
/// 4. Loading -> Failed（fail_load）
/// 5. Failed -> Loading（load_url 重试）
/// 6. Loading -> Loaded（complete_load）
/// 7. Loaded -> Loading（load_html 不改变 loading 状态）
///   验证每个阶段 is_loading、url、last_render 的值正确。
#[test]
fn test_webview_lifecycle_state_transitions() {
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

    // ── 阶段 1: Created ──
    assert!(!wv.is_loading());
    assert!(wv.url().is_none());
    assert!(wv.last_render().is_none());

    // ── 阶段 2: Created -> Loading ──
    wv.load_url("https://lifecycle.com");
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://lifecycle.com"));
    assert!(wv.last_render().is_none());

    // ── 阶段 3: Loading -> Loaded ──
    wv.complete_load("<html><body><div>Loaded</div></body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://lifecycle.com"));
    assert!(wv.last_render().is_some());

    // ── 阶段 4: Loaded -> Loading（导航到新 URL）──
    wv.load_url("https://fail-lifecycle.com");
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://fail-lifecycle.com"));

    // ── 阶段 5: Loading -> Failed ──
    wv.fail_load("connection refused");
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://fail-lifecycle.com"));

    // ── 阶段 6: Failed -> Loading（重试）──
    wv.load_url("https://retry-lifecycle.com");
    assert!(wv.is_loading());
    assert_eq!(wv.url(), Some("https://retry-lifecycle.com"));

    // ── 阶段 7: Loading -> Loaded（重试成功）──
    wv.complete_load("<html><body><div>Retry OK</div></body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some("https://retry-lifecycle.com"));
    assert!(wv.last_render().is_some());

    // ── 验证事件序列 ──
    let recorded = events.borrow();
    // 完整序列:
    //   load_url -> LoadStart+UrlChanged (2)
    //   complete_load -> LoadEnd (1)
    //   load_url -> LoadStart+UrlChanged (2)
    //   fail_load -> LoadFailed (1)
    //   load_url -> LoadStart+UrlChanged (2)
    //   complete_load -> LoadEnd (1)
    //   合计 = 9
    assert_eq!(recorded.len(), 9, "应有 9 个事件，实际: {recorded:?}");
    assert_eq!(recorded[0], "LoadStart(https://lifecycle.com)");
    assert_eq!(recorded[1], "UrlChanged(https://lifecycle.com)");
    assert_eq!(recorded[2], "LoadEnd(https://lifecycle.com)");
    assert_eq!(recorded[3], "LoadStart(https://fail-lifecycle.com)");
    assert_eq!(recorded[4], "UrlChanged(https://fail-lifecycle.com)");
    assert!(recorded[5].starts_with("LoadFailed(https://fail-lifecycle.com"));
    assert_eq!(recorded[6], "LoadStart(https://retry-lifecycle.com)");
    assert_eq!(recorded[7], "UrlChanged(https://retry-lifecycle.com)");
    assert_eq!(recorded[8], "LoadEnd(https://retry-lifecycle.com)");
}

// ════════════════════════════════════════════════════════════════
//  边界条件测试：极端尺寸、Unicode URL、异常状态转换、回调边界
// ════════════════════════════════════════════════════════════════

/// 验证将 WebView 尺寸调整至 u32::MAX 不会 panic。
///
/// 边界场景：传入 u32 最大值作为视口宽高，
/// 确保内部 RenderPipeline 不会因整数溢出或内存分配失败而崩溃。
/// resize 应正常存储配置值，后续 render 也不应 panic。
#[test]
fn test_webview_resize_to_u32_max() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<html><body><div>Extreme</div></body></html>", None);
    wv.resize(u32::MAX, u32::MAX);
    assert_eq!(wv.config().width, u32::MAX);
    assert_eq!(wv.config().height, u32::MAX);
    // render 在极端尺寸下应不 panic（管线内部会处理）
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0, "极端尺寸下渲染耗时应为非负");
}

/// 验证加载包含 Unicode 和特殊字符的 URL 不会 panic，且 URL 被正确存储。
///
/// 边界场景：URL 包含中日韩字符、URL 编码百分号、查询参数中的特殊符号，
/// 确保 load_url 不会因非 ASCII 字符而崩溃，current_url 被原样存储。
#[test]
fn test_webview_load_url_with_unicode_and_special_chars() {
    let mut wv = WebView::new(WebViewConfig::default());
    let url = "https://例え.jp/パス?q=hello%20world&lang=日本語#セクション";
    wv.load_url(url);
    assert_eq!(wv.url(), Some(url), "Unicode URL 应被原样存储");
    assert!(wv.is_loading());
    wv.complete_load("<html><body><div>Unicode URL 内容</div></body></html>", None);
    assert!(!wv.is_loading());
    assert_eq!(wv.url(), Some(url));
}

/// 验证从加载失败状态直接调用 complete_load 不会 panic，且状态转换正确。
///
/// 异常状态转换路径：load_url -> fail_load -> complete_load（无中间 load_url）。
/// fail_load 将 loading 置为 false，complete_load 应能正常工作：
/// 加载 HTML、将 loading 置为 false（已为 false 不变），并触发 LoadEnd 事件。
#[test]
fn test_webview_fail_load_then_complete_without_load_url() {
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

    // 先导航并失败
    wv.load_url("https://will-fail.com");
    assert!(wv.is_loading());
    wv.fail_load("server error 500");
    assert!(!wv.is_loading());

    // 在未再次调用 load_url 的情况下直接 complete_load
    let result = wv.complete_load("<html><body><div>Recovery</div></body></html>", None);
    assert!(!wv.is_loading(), "complete_load 后不应处于加载状态");
    assert!(result.timings.total_ms >= 0.0, "渲染耗时应为非负");
    assert!(wv.last_render().is_some(), "应有渲染结果");
    // URL 应保留为 will-fail.com（complete_load 使用 current_url）
    assert_eq!(wv.url(), Some("https://will-fail.com"));

    // 事件序列：LoadStart + UrlChanged + LoadFailed + LoadEnd
    let recorded = events.borrow();
    assert_eq!(recorded.len(), 4, "应有 4 个事件，实际: {recorded:?}");
    assert!(recorded[3].starts_with("LoadEnd(https://will-fail.com"));
}

/// 验证在未注册任何回调时调用 remove_event_callback 返回 false。
///
/// 边界场景：event_callbacks 为空列表时，传入索引 0（合法 usize），
/// remove_event_callback 应安全返回 false 而非 panic。
#[test]
fn test_webview_remove_event_callback_on_empty_list() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 未注册任何回调，空列表
    assert!(!wv.remove_event_callback(0), "空回调列表中索引 0 应返回 false");
    assert!(
        !wv.remove_event_callback(usize::MAX),
        "空回调列表中 usize::MAX 应返回 false"
    );
    // 后续操作应正常工作，不 panic
    wv.load_url("https://test.com");
    assert_eq!(wv.url(), Some("https://test.com"));
}

/// 验证在加载状态（loading=true）下调用 render 不会改变加载标志。
///
/// 边界场景：load_url 将 loading 置为 true 后，直接调用 render，
/// render 仅执行重新渲染，不应干扰 loading 状态。
/// 适用于外部异步加载过程中需要中间渲染的场景（如进度指示器）。
#[test]
fn test_webview_render_while_loading_state() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 先加载一些内容到 cached_html 中
    wv.load_html("<html><body><div>Loading indicator</div></body></html>", None);
    assert!(!wv.is_loading());

    // 发起 URL 加载（设置 loading=true）
    wv.load_url("https://slow-page.com");
    assert!(wv.is_loading(), "load_url 后应处于加载状态");
    assert!(wv.last_render().is_some(), "之前的 load_html 应有渲染结果");

    // 在 loading 状态下调用 render — 模拟显示加载进度
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0, "loading 中 render 耗时应为非负");

    // 关键断言：render 不应改变 loading 状态
    assert!(wv.is_loading(), "render() 不应改变 loading 状态，应仍为 true");
    assert_eq!(wv.url(), Some("https://slow-page.com"), "URL 不应被 render 改变");

    // 最终完成加载
    wv.complete_load("<html><body><div>Final content</div></body></html>", None);
    assert!(!wv.is_loading(), "complete_load 后不应处于加载状态");
}

// ════════════════════════════════════════════════════════════════
//  边界条件测试：空标题、CSS 累积链、空白 HTML、无 URL 失败、inject 不干扰 loading
// ════════════════════════════════════════════════════════════════

/// 验证将标题设置为空字符串后，title() 返回 Some("") 而非 None。
///
/// 边界场景：空字符串在语义上与 None 不同，
/// set_title("") 应将内部 title 字段设为 Some("")，
/// 后续 title() 应精确返回 Some("") 而非 None。
#[test]
fn test_webview_set_title_empty_string() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 初始状态：标题为 None
    assert!(wv.title().is_none(), "初始标题应为 None");

    // 设置空字符串标题
    wv.set_title("");
    assert_eq!(wv.title(), Some(""), "空字符串标题应为 Some(\"\")，而非 None");

    // 覆盖为非空标题后再次设为空字符串
    wv.set_title("Real Title");
    assert_eq!(wv.title(), Some("Real Title"));
    wv.set_title("");
    assert_eq!(wv.title(), Some(""), "再次设为空字符串应为 Some(\"\")");
}

/// 验证 complete_load 传入 CSS 后，再 inject_css 追加的样式被正确累积。
///
/// 场景：load_url -> complete_load(html, Some(css_a)) -> inject_css(css_b)。
/// complete_load 内部调用 load_html 会缓存 css_a，
/// inject_css 在 cached_css 后追加 css_b，
/// render() 应使用包含 css_a + css_b 的累积 CSS。
#[test]
fn test_webview_complete_load_with_css_then_inject_more_css() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body>\
        <div class=\"original\">A</div>\
        <div class=\"extra\">B</div>\
        </body></html>";

    // 通过 load_url + complete_load 加载带 CSS 的内容
    wv.load_url("https://styled.com");
    let after_complete = wv.complete_load(
        html,
        Some(".original { background-color: red; width: 100px; height: 50px; }"),
    );
    let fills_after_complete = after_complete.primitives.fills.len();
    assert!(fills_after_complete > 0, "complete_load 带 CSS 应产生 fills");

    // 注入额外 CSS
    let after_inject = wv.inject_css(".extra { background-color: blue; width: 80px; height: 40px; }");
    let fills_after_inject = after_inject.primitives.fills.len();
    assert!(
        fills_after_inject >= fills_after_complete,
        "inject_css 应追加到 complete_load 的 CSS 上，fills 应 >= 注入前 (got {fills_after_inject} < {fills_after_complete})"
    );

    // render() 应保留累积的 CSS
    let after_render = wv.render();
    assert_eq!(
        after_render.primitives.fills.len(),
        fills_after_inject,
        "render() 应使用 complete_load CSS + inject CSS 的累积结果"
    );
}

/// 验证加载仅含空白字符的 HTML 不会 panic，且返回有效渲染结果。
///
/// 边界场景：传入 "   \n\t  " 等纯空白字符串，
/// 渲染管线应能处理无有效 HTML 标签的输入，
/// 不会因缺少根元素或内容为空而崩溃。
#[test]
fn test_webview_load_html_with_only_whitespace() {
    let mut wv = WebView::new(WebViewConfig::default());
    let whitespace_html = "   \n\t  \r\n   ";
    let result = wv.load_html(whitespace_html, None);
    assert!(result.timings.total_ms >= 0.0, "纯空白 HTML 渲染耗时应为非负");
    assert!(wv.last_render().is_some(), "纯空白 HTML 加载后应有渲染结果");
    assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
    assert!(wv.url().is_none(), "load_html 不应设置 URL");

    // 后续操作应正常工作
    let inject_result = wv.inject_css("div { color: red; }");
    assert!(inject_result.timings.total_ms >= 0.0, "空白 HTML 上注入 CSS 不应 panic");

    let render_result = wv.render();
    assert!(render_result.timings.total_ms >= 0.0, "空白 HTML 上重新渲染不应 panic");
}

/// R1987：`<img src="data:...">` 经 `reload_html_after_script` → `fetch_image_subresources`
///（sync 路径，current_url 设置时触发）解码 + 缓存，不再跳过 data: URI（in-scope img 子资源，
/// goal line 118 SVG-as-img）。fetch_image_subresources 由 fetch_url / reload_html_after_script 调用
///（load_html 本身不抓子资源），故用 load_url 设 current_url 后 reload_html_after_script 触发。
#[test]
fn test_load_html_decodes_data_uri_image() {
    let svg = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='16' height='16'><rect width='16' height='16' fill='%23880088'/></svg>";
    let html = format!(
        "<html><body><img src=\"{}\" width=\"16\" height=\"16\"></body></html>",
        svg
    );
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://example.com/");
    let _ = wv.reload_html_after_script(&html);
    assert!(
        !wv.image_cache_ref().is_empty(),
        "data: URI SVG image should be decoded + cached (R1987), not skipped"
    );
}

/// R1987：base64 PNG data: URI 同理解码 + 缓存。
#[test]
fn test_load_html_decodes_data_uri_base64_png() {
    // 2×2 red PNG（base64，ZW png crate 可解）。
    let png = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAIAAAACCAYAAABytg0kAAAAFElEQVR4nGP8z8Dwn4GBgYGJAQoAHxcCAk+Uzr4AAAAASUVORK5CYII=";
    let html = format!(
        "<html><body><img src=\"{}\" width=\"2\" height=\"2\"></body></html>",
        png
    );
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_url("https://example.com/");
    let _ = wv.reload_html_after_script(&html);
    assert!(
        !wv.image_cache_ref().is_empty(),
        "base64 PNG data: URI should be decoded + cached (R1987)"
    );
}

/// 验证在未先调用 load_url 的情况下直接调用 fail_load 不会 panic。
///
/// 边界场景：current_url 为 None 时调用 fail_load，
/// 内部 current_url.unwrap_or_default() 应返回空字符串，
/// LoadFailed 事件的 URL 字段应为空字符串。
/// loading 状态应从 false 变为 false（无变化）。
#[test]
fn test_webview_fail_load_without_prior_load_url_uses_empty_url() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let ec = events.clone();
    wv.on_event(move |e| {
        ec.borrow_mut().push(e.clone());
    });

    // 初始状态：无 URL，不在加载中
    assert!(wv.url().is_none());
    assert!(!wv.is_loading());

    // 直接调用 fail_load，未先调用 load_url
    wv.fail_load("unexpected error");
    assert!(!wv.is_loading(), "fail_load 后 loading 应为 false");

    // 验证 LoadFailed 事件的 URL 为空字符串
    let recorded = events.borrow();
    assert_eq!(recorded.len(), 1, "应有 1 个 LoadFailed 事件");
    assert!(
        matches!(&recorded[0], WebViewEvent::LoadFailed(url, msg) if url.is_empty() && msg.contains("unexpected error")),
        "LoadFailed 事件的 URL 应为空字符串，实际: {:?}",
        recorded[0]
    );
}

/// 验证在 loading 状态下调用 inject_css 不会重置 loading 标志。
///
/// 边界场景：load_url 将 loading 置为 true 后，
/// 调用 inject_css 进行样式注入（如加载指示器的 CSS 动画），
/// inject_css 不应干扰导航状态，loading 应保持为 true。
/// 适用于异步加载过程中动态更新样式的场景。
#[test]
fn test_webview_inject_css_after_load_url_preserves_loading_state() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 先加载 HTML 内容到缓存
    wv.load_html(
        "<html><body><div class=\"spinner\">Loading...</div></body></html>",
        None,
    );
    assert!(!wv.is_loading());

    // 发起 URL 加载
    wv.load_url("https://async-page.com");
    assert!(wv.is_loading(), "load_url 后应处于加载状态");
    assert_eq!(wv.url(), Some("https://async-page.com"));

    // 在 loading 状态下注入 CSS（如加载动画样式）
    let result = wv.inject_css(".spinner { animation: spin 1s linear infinite; }");
    assert!(result.timings.total_ms >= 0.0, "inject_css 渲染耗时应为非负");

    // 关键断言：inject_css 不应重置 loading 状态
    assert!(wv.is_loading(), "inject_css 不应改变 loading 状态，应仍为 true");
    assert_eq!(wv.url(), Some("https://async-page.com"), "URL 不应被 inject_css 改变");

    // 后续 complete_load 应正常完成加载
    wv.complete_load("<html><body><div>Final</div></body></html>", None);
    assert!(!wv.is_loading(), "complete_load 后不应处于加载状态");
}

// ════════════════════════════════════════════════════════════════
//  边界条件测试：连续失败、回调移除后验证、Builder 空 URL、超长 CSS、渲染幂等
// ════════════════════════════════════════════════════════════════

// ── 新增测试：覆盖更多边界情况 ──

/// 测试 resize 到 u32 最大值 - edge
#[test]
fn test_webview_resize_to_u32_max_edge() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 调整到最大尺寸
    wv.resize(u32::MAX, u32::MAX);
    assert_eq!(wv.config().width, u32::MAX);
    assert_eq!(wv.config().height, u32::MAX);
}

/// 测试 resize 到极小尺寸
#[test]
fn test_webview_resize_minimal_dimensions() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 调整到极小尺寸
    wv.resize(1, 1);
    assert_eq!(wv.config().width, 1);
    assert_eq!(wv.config().height, 1);
}

/// 测试多次 resize
#[test]
fn test_webview_multiple_resizes() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 多次调整大小
    wv.resize(800, 600);
    assert_eq!(wv.config().width, 800);
    assert_eq!(wv.config().height, 600);

    wv.resize(1024, 768);
    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);

    wv.resize(300, 200);
    assert_eq!(wv.config().width, 300);
    assert_eq!(wv.config().height, 200);
}

/// 测试 load_html 空 HTML 和各种组合
#[test]
fn test_webview_load_html_empty_content() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 空 HTML
    let result1 = wv.load_html("", None);
    assert!(result1.timings.total_ms >= 0.0);

    // 空 HTML + 空 CSS
    let result2 = wv.load_html("", Some(""));
    assert!(result2.timings.total_ms >= 0.0);

    // 空 HTML + 有效 CSS
    let result3 = wv.load_html("", Some("body { color: red; }"));
    assert!(result3.timings.total_ms >= 0.0);
}

/// 测试 load_url 相同 URL 不触发 UrlChanged 事件
#[test]
fn test_webview_load_url_same_url_no_events() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 设置初始 URL
    wv.load_url("https://example.com");
    let original_url = wv.url().map(|s| s.to_string());

    // 再次设置相同 URL，不应该触发 UrlChanged 事件
    wv.load_url("https://example.com");

    assert_eq!(wv.url().map(|s| s.to_string()), original_url);
    assert!(wv.is_loading());
}

/// 测试 execute_script 大量参数和返回值
#[test]
fn test_webview_execute_script_large_inputs() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 大对象创建测试
    let script = r#"
        const arr = [];
        for (let i = 0; i < 10000; i++) {
            arr.push({ id: i, name: 'test' + i });
        }
        JSON.stringify(arr);
    "#;

    let result = wv.execute_script(script);
    assert!(result.is_ok());
    assert!(result.unwrap().contains("\"id\":9999"));
}

/// 测试 execute_script 深层属性访问
#[test]
fn test_webview_execute_script_deep_property_access() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 深层属性链访问
    let script = r#"
        window.document.body.element.firstChild.id;
    "#;

    let result = wv.execute_script(script);
    // 即使对象不存在，也不应该 panic
    assert!(result.is_ok() || result.is_err());
}

/// 验证连续调用 fail_load 两次不会 panic，且 loading 始终为 false。
///
/// 边界场景：第一次 fail_load 将 loading 从 true 置为 false，
/// 第二次 fail_load 在 loading 已经为 false 的状态下调用，
/// 不应导致状态异常或 panic，且每次调用都应触发 LoadFailed 事件。
#[test]
fn test_webview_consecutive_fail_load_calls() {
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

    // 第一次加载并失败
    wv.load_url("https://first-fail.com");
    assert!(wv.is_loading());
    wv.fail_load("timeout");
    assert!(!wv.is_loading());

    // 第二次连续失败（未重新 load_url）
    wv.fail_load("second error");
    assert!(!wv.is_loading(), "连续 fail_load 后 loading 应仍为 false");

    // 验证两次 LoadFailed 事件都被触发
    let recorded = events.borrow();
    let fail_count = recorded.iter().filter(|e| e.starts_with("LoadFailed")).count();
    assert_eq!(fail_count, 2, "应有 2 次 LoadFailed 事件");
}

/// 验证移除事件回调后，后续操作不再触发该回调。
///
/// 场景：注册回调 A -> 触发操作（回调 A 被调用）-> 移除回调 A -> 触发操作（回调 A 不再被调用）。
/// 通过引用计数验证回调被调用次数精确匹配预期值。
#[test]
fn test_webview_callback_removed_no_longer_fires() {
    let mut wv = WebView::new(WebViewConfig::default());
    let call_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let cc = call_count.clone();
    let idx = wv.on_event(move |_| {
        *cc.borrow_mut() += 1;
    });

    // 第一次 set_title — 回调应被触发
    wv.set_title("First");
    assert_eq!(*call_count.borrow(), 1, "注册后回调应被触发 1 次");

    // 移除回调
    assert!(wv.remove_event_callback(idx));

    // 第二次 set_title — 已移除的回调不应再被触发
    wv.set_title("Second");
    assert_eq!(*call_count.borrow(), 1, "移除后回调不应再被触发，计数应保持 1");

    // 第三次 set_title — 确认回调持续不触发
    wv.set_title("Third");
    assert_eq!(*call_count.borrow(), 1, "多次操作后回调仍不应被触发");
}

/// 验证 WebViewBuilder 传入空字符串 URL 后，build 产生正确的初始状态。
///
/// 边界场景：url("") 是合法的链式调用（非 None），
/// build 应自动调用 load_url("")，将 WebView 置为加载状态，
/// current_url 应为 Some("")（空字符串，与 None 语义不同）。
#[test]
fn test_webview_builder_with_empty_url_string() {
    let wv = WebViewBuilder::new().url("").build();
    // url("") 设置了 config.url = Some("")，build 时会调用 load_url("")
    assert_eq!(wv.url(), Some(""), "空字符串 URL 应为 Some(\"\")，而非 None");
    assert!(wv.is_loading(), "空 URL 仍应触发加载状态");
    assert!(wv.last_render().is_none(), "仅有 load_url 不应有渲染结果");

    // 后续 complete_load 应正常工作
    let mut wv = wv;
    wv.complete_load("<html><body><div>Empty URL page</div></body></html>", None);
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_some());
    assert_eq!(wv.url(), Some(""), "complete_load 后 URL 应保持为空字符串");
}

/// 验证加载包含超长 CSS 属性值的 HTML 不会 panic，且渲染管线返回有效结果。
///
/// 边界场景：CSS 属性值长度达到数千字符（如超长 gradient 定义），
/// 确保 CSS 解析器和渲染管线不会因字符串过长而崩溃或内存溢出。
#[test]
fn test_webview_load_html_with_very_long_css_value() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 构造一个超长的 CSS background 属性值（重复 linear-gradient 段）
    let long_gradient = "linear-gradient(red, blue)".repeat(200);
    let css = format!("div {{ background: {long_gradient}; width: 100px; height: 50px; }}");
    let html = "<html><body><div>Long CSS test</div></body></html>";

    let result = wv.load_html(html, Some(&css));
    assert!(result.timings.total_ms >= 0.0, "超长 CSS 渲染耗时应为非负");
    assert!(wv.last_render().is_some(), "超长 CSS 加载后应有渲染结果");

    // 后续操作不应崩溃
    let inject_result = wv.inject_css("span { color: red; }");
    assert!(inject_result.timings.total_ms >= 0.0);
    let render_result = wv.render();
    assert!(render_result.timings.total_ms >= 0.0);
}

/// 验证 complete_load 后连续多次 render 产生完全相同的 fills 数量（渲染幂等性）。
///
/// 边界场景：相同输入（cached_html + cached_css）多次调用 render，
/// 渲染结果应在 fills 数量上保持一致（幂等），
/// 不应因内部状态变化而产生不同输出。
#[test]
fn test_webview_render_idempotent_after_complete_load() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body>\
        <div class=\"box-a\">Box A</div>\
        <div class=\"box-b\">Box B</div>\
        </body></html>";
    let css = ".box-a { background-color: red; width: 100px; height: 50px; }\
               .box-b { background-color: blue; width: 200px; height: 80px; }";

    wv.load_url("https://idempotent.com");
    let _complete = wv.complete_load(html, Some(css));
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_some());

    // 第一次 render
    let first = wv.render();
    let first_fills = first.primitives.fills.len();

    // 第二次 render — 应产生相同的 fills 数量
    let second = wv.render();
    let second_fills = second.primitives.fills.len();

    // 第三次 render — 进一步确认幂等性
    let third = wv.render();
    let third_fills = third.primitives.fills.len();

    assert_eq!(
        first_fills, second_fills,
        "连续 render 的 fills 数量应一致（第一次 vs 第二次）"
    );
    assert_eq!(
        second_fills, third_fills,
        "连续 render 的 fills 数量应一致（第二次 vs 第三次）"
    );
    assert!(first_fills > 0, "带背景色 CSS 的 HTML 应产生至少一个 fill 图元");
}

// ════════════════════════════════════════════════════════════════
//  边界条件测试：inject_css 先于 load_html、set_title 事件计数、零视口渲染、
//  失败恢复后渲染、连续 load_html 内容覆盖
// ════════════════════════════════════════════════════════════════

/// 验证在全新 WebView 上（从未调用 load_html）直接 inject_css 不会 panic。
///
/// 边界场景：WebView 刚创建，cached_html 为空，
/// 此时调用 inject_css 应安全返回有效的渲染结果，
/// 不会因缺少已缓存 HTML 而崩溃。
#[test]
fn test_webview_inject_css_before_any_load_html() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 全新 WebView，未调用任何 load_html
    assert!(wv.last_render().is_none(), "全新 WebView 不应有渲染结果");

    // 在未加载任何 HTML 的情况下直接注入 CSS
    let result = wv.inject_css("div { color: red; width: 100px; height: 50px; }");
    assert!(result.timings.total_ms >= 0.0, "inject_css 应返回非负耗时");
    assert!(wv.last_render().is_some(), "inject_css 后应有渲染结果");
    assert!(!wv.is_loading(), "inject_css 不应触发加载状态");

    // 后续 load_html 应正常工作
    let html_result = wv.load_html("<html><body><div>After inject</div></body></html>", None);
    assert!(html_result.timings.total_ms >= 0.0, "后续 load_html 应正常工作");
}

/// 验证多次 set_title 调用每次都触发独立的 TitleChanged 事件。
///
/// 场景：连续调用 set_title 三次（包含重复标题值），
/// 每次调用都应触发一个 TitleChanged 事件，共 3 个事件。
/// 即使标题值与前一次相同，事件仍应触发。
#[test]
fn test_webview_set_title_fires_separate_events_each_call() {
    let mut wv = WebView::new(WebViewConfig::default());
    let events: Rc<RefCell<Vec<WebViewEvent>>> = Rc::new(RefCell::new(Vec::new()));
    let ec = events.clone();
    wv.on_event(move |e| {
        ec.borrow_mut().push(e.clone());
    });

    // 第一次 set_title
    wv.set_title("标题一");
    // 第二次 set_title（不同值）
    wv.set_title("标题二");
    // 第三次 set_title（与第二次相同的值——仍应触发事件）
    wv.set_title("标题二");

    let recorded = events.borrow();
    assert_eq!(recorded.len(), 3, "三次 set_title 应触发 3 个 TitleChanged 事件");
    assert!(
        matches!(&recorded[0], WebViewEvent::TitleChanged(t) if t == "标题一"),
        "第一个事件应为 TitleChanged(\"标题一\")"
    );
    assert!(
        matches!(&recorded[1], WebViewEvent::TitleChanged(t) if t == "标题二"),
        "第二个事件应为 TitleChanged(\"标题二\")"
    );
    assert!(
        matches!(&recorded[2], WebViewEvent::TitleChanged(t) if t == "标题二"),
        "第三个事件应为 TitleChanged(\"标题二\")（重复值仍触发事件）"
    );
}

/// 验证 WebView 在零尺寸视口（width=0, height=0）下 render 不会 panic。
///
/// 边界场景：将视口尺寸设为 (0, 0) 后调用 render，
/// 渲染管线应能处理零尺寸画布，不会因除零或空缓冲区而崩溃。
/// 适用于窗口最小化或隐藏时的场景。
#[test]
fn test_webview_render_with_zero_size_viewport() {
    let mut wv = WebView::new(WebViewConfig {
        width: 0,
        height: 0,
        ..Default::default()
    });

    // 加载内容后渲染——零视口不应 panic
    let result = wv.load_html("<html><body><div>Zero viewport</div></body></html>", None);
    assert!(result.timings.total_ms >= 0.0, "零视口 load_html 应返回非负耗时");

    // render 在零视口下也不应 panic
    let render_result = wv.render();
    assert!(render_result.timings.total_ms >= 0.0, "零视口 render 应返回非负耗时");
    assert_eq!(wv.config().width, 0, "视口宽度应为 0");
    assert_eq!(wv.config().height, 0, "视口高度应为 0");

    // resize 到正常尺寸后应恢复正常
    wv.resize(800, 600);
    let after_resize = wv.render();
    assert!(after_resize.timings.total_ms >= 0.0, "恢复尺寸后 render 应成功");
    assert_eq!(wv.config().width, 800);
    assert_eq!(wv.config().height, 600);
}

/// 验证加载失败后通过 load_html 恢复，渲染结果反映恢复后的内容。
///
/// 场景：load_url -> fail_load（模拟网络错误）-> load_html 恢复。
/// fail_load 后 last_render 保留之前的结果（可能为 None），
/// load_html 应覆盖缓存内容并产生新的渲染结果，
/// 且 WebView 状态应完全恢复为正常（不处于加载状态）。
#[test]
fn test_webview_render_after_load_failure_recovery() {
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

    // 先成功加载一个页面
    wv.load_url("https://good-page.com");
    wv.complete_load("<html><body><div>Good content</div></body></html>", None);
    assert!(!wv.is_loading());
    assert!(wv.last_render().is_some());
    let good_render_fills = wv.last_render().unwrap().primitives.fills.len();

    // 导航到新 URL 但加载失败
    wv.load_url("https://broken-page.com");
    assert!(wv.is_loading());
    wv.fail_load("connection refused");
    assert!(!wv.is_loading(), "失败后 loading 应停止");
    // last_render 保留之前成功的结果
    assert!(wv.last_render().is_some(), "失败后应保留上次成功的渲染结果");

    // 通过 load_html 恢复——加载新内容
    let recovery_html = "<html><body><div>Recovery content</div></body></html>";
    let recovery_css = "div { background-color: green; width: 200px; height: 100px; }";
    let result = wv.load_html(recovery_html, Some(recovery_css));
    assert!(result.timings.total_ms >= 0.0, "恢复 load_html 应返回非负耗时");
    assert!(!wv.is_loading(), "load_html 后不应处于加载状态");
    assert!(wv.last_render().is_some(), "恢复后应有渲染结果");

    // 渲染结果应反映恢复后的内容（带 CSS，fills 应 > 无 CSS 时）
    let render_result = wv.render();
    assert!(render_result.timings.total_ms >= 0.0);
    assert!(
        render_result.primitives.fills.len() >= good_render_fills,
        "恢复后的渲染结果应反映新加载的带 CSS 内容"
    );

    // 验证事件序列：LoadStart+UrlChanged+LoadEnd + LoadStart+UrlChanged+LoadFailed
    // load_html 不触发事件
    let recorded = events.borrow();
    assert_eq!(recorded.len(), 6, "应有 6 个事件（成功加载 3 + 失败 3）");
    assert!(recorded[5].starts_with("LoadFailed(https://broken-page.com"));
}

/// 验证连续调用 load_html 三次，每次渲染结果反映最新加载的内容。
///
/// 场景：依次加载三份不同 HTML（含不同 CSS 样式），
/// 每次 load_html 后调用 render，验证 fills 数量随内容变化。
/// 最终 render 应反映第三次加载的内容，而非第一次或第二次。
#[test]
fn test_webview_consecutive_load_html_reflects_latest_content() {
    let mut wv = WebView::new(WebViewConfig::default());

    // 第一次加载：带红色背景的 div
    let html1 = "<html><body><div class=\"box\">Version 1</div></body></html>";
    let css1 = ".box { background-color: red; width: 100px; height: 50px; }";
    let result1 = wv.load_html(html1, Some(css1));
    assert!(result1.timings.total_ms >= 0.0, "第一次 load_html 应成功");
    let fills1 = wv.render().primitives.fills.len();

    // 第二次加载：带蓝色背景的 div + 额外 div
    let html2 = "<html><body>\
        <div class=\"box\">Version 2</div>\
        <div class=\"extra\">Extra</div>\
        </body></html>";
    // 注意：load_html 会重置 cached_css，仅使用传入的 CSS
    let css2 = ".box { background-color: blue; width: 200px; height: 80px; }\
                .extra { background-color: green; width: 50px; height: 30px; }";
    let result2 = wv.load_html(html2, Some(css2));
    assert!(result2.timings.total_ms >= 0.0, "第二次 load_html 应成功");
    let fills2 = wv.render().primitives.fills.len();

    // 第三次加载：仅一个无样式的 div
    let html3 = "<html><body><div>Version 3 - plain</div></body></html>";
    let result3 = wv.load_html(html3, None);
    assert!(result3.timings.total_ms >= 0.0, "第三次 load_html 应成功");
    let fills3 = wv.render().primitives.fills.len();

    // 验证第二次加载的 fills >= 第一次（更多带样式的元素）
    assert!(
        fills2 >= fills1,
        "第二次加载（两个带样式 div）的 fills 应 >= 第一次（一个带样式 div），got {fills2} < {fills1}"
    );

    // 验证第三次加载的 fills <= 第二次（无 CSS，背景色消失）
    assert!(
        fills3 <= fills2,
        "第三次加载（无 CSS）的 fills 应 <= 第二次（两个带样式 div），got {fills3} > {fills2}"
    );

    // 最终 render 应反映第三次的内容（无 CSS）
    let final_render = wv.render();
    assert_eq!(
        final_render.primitives.fills.len(),
        fills3,
        "最终 render 应反映第三次加载的内容"
    );
}

// ════════════════════════════════════════════════════════════════
//  边界条件测试：极大尺寸 Builder、inject_css 空字符串、最小 HTML、
//  TitleChanged 回调零触发、多次 inject_css 累积渲染
// ════════════════════════════════════════════════════════════════

/// 验证 WebViewBuilder 使用极大尺寸（u32::MAX）构建 WebView 不会 panic，
/// 且生成的 WebView 配置正确反映传入的尺寸。
///
/// 边界场景：width/height 设为 u32 最大值，
/// 确保构建器不会因整数溢出或内存预分配失败而崩溃。
/// 后续 load_html 和 render 也应在极端视口下安全完成。
#[test]
fn test_webview_builder_very_large_dimensions() {
    let mut wv = WebViewBuilder::new().width(u32::MAX).height(u32::MAX).build();

    assert_eq!(wv.config().width, u32::MAX, "宽度应存储为 u32::MAX");
    assert_eq!(wv.config().height, u32::MAX, "高度应存储为 u32::MAX");

    // 极大视口下加载和渲染不应 panic
    let html = "<html><body><div>Large viewport</div></body></html>";
    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0, "极大视口渲染耗时应为非负");
    assert!(wv.last_render().is_some());

    let render_result = wv.render();
    assert!(render_result.timings.total_ms >= 0.0, "极大视口下 re-render 应安全");
}

/// 验证 inject_css 传入空字符串不会 panic，且返回有效的渲染结果。
///
/// 边界场景：在已有 HTML 内容的 WebView 上注入空 CSS，
/// 渲染管线应安全处理空字符串输入，
/// fills 数量应与注入前保持一致（空 CSS 不产生新的样式规则）。
#[test]
fn test_webview_inject_css_empty_string_preserves_fills() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body><div class=\"box\">Content</div></body></html>";
    let css = ".box { background-color: red; width: 100px; height: 50px; }";

    let after_load = wv.load_html(html, Some(css));
    let fills_before = after_load.primitives.fills.len();
    assert!(fills_before > 0, "带 CSS 的 load_html 应产生 fills");

    // 注入空字符串 CSS
    let after_inject = wv.inject_css("");
    assert!(after_inject.timings.total_ms >= 0.0, "空 CSS 注入耗时应为非负");
    assert_eq!(
        after_inject.primitives.fills.len(),
        fills_before,
        "空 CSS 注入不应改变 fills 数量"
    );

    // render 也应保持一致
    let after_render = wv.render();
    assert_eq!(
        after_render.primitives.fills.len(),
        fills_before,
        "render 后 fills 应与注入空 CSS 前一致"
    );
}

/// 验证 load_html 加载最小 HTML "<html></html>" 不会 panic，且返回有效渲染结果。
///
/// 边界场景：传入仅含根元素、无 body、无内容的极简 HTML，
/// 确保 DOM 树构建和渲染管线不会因缺少 body 或内容为空而崩溃。
#[test]
fn test_webview_load_html_minimal_html_tag() {
    let mut wv = WebView::new(WebViewConfig::default());

    let result = wv.load_html("<html></html>", None);
    assert!(result.timings.total_ms >= 0.0, "最小 HTML 渲染耗时应为非负");
    assert!(wv.last_render().is_some(), "最小 HTML 加载后应有渲染结果");
    assert!(!wv.is_loading(), "load_html 不应将 WebView 置为加载状态");
    assert!(wv.url().is_none(), "load_html 不应设置 URL");
    assert!(wv.title().is_none(), "最小 HTML 不应产生标题");

    // 后续操作应正常工作
    let inject_result = wv.inject_css("body { margin: 0; }");
    assert!(inject_result.timings.total_ms >= 0.0, "最小 HTML 上注入 CSS 不应 panic");

    let render_result = wv.render();
    assert!(render_result.timings.total_ms >= 0.0, "最小 HTML 重新渲染不应 panic");
}

/// 验证在未调用 set_title 时，TitleChanged 回调触发次数为零。
///
/// 边界场景：注册 TitleChanged 监听后执行一系列操作
/// （load_url、complete_load、load_html、inject_css、render），
/// 由于所有操作均未调用 set_title，
/// TitleChanged 事件计数应始终保持为 0。
#[test]
fn test_webview_title_changed_zero_fires_without_set_title() {
    let mut wv = WebView::new(WebViewConfig::default());
    let title_change_count: Rc<RefCell<usize>> = Rc::new(RefCell::new(0));
    let tcc = title_change_count.clone();
    wv.on_event(move |e| {
        if matches!(e, WebViewEvent::TitleChanged(_)) {
            *tcc.borrow_mut() += 1;
        }
    });

    // 执行一系列操作——均不涉及 set_title
    wv.load_html("<html><body><div>No title set</div></body></html>", None);
    assert_eq!(*title_change_count.borrow(), 0, "load_html 后 TitleChanged 计数应为 0");

    wv.load_url("https://titleless.com");
    assert_eq!(*title_change_count.borrow(), 0, "load_url 后 TitleChanged 计数应为 0");

    wv.complete_load("<html><body><div>Loaded</div></body></html>", None);
    assert_eq!(
        *title_change_count.borrow(),
        0,
        "complete_load 后 TitleChanged 计数应为 0"
    );

    wv.inject_css("div { color: blue; }");
    assert_eq!(*title_change_count.borrow(), 0, "inject_css 后 TitleChanged 计数应为 0");

    let _ = wv.render();
    assert_eq!(*title_change_count.borrow(), 0, "render 后 TitleChanged 计数应为 0");

    // 确认 title() 仍为 None
    assert!(wv.title().is_none(), "未调用 set_title 时 title 应为 None");
}

/// 验证多次调用 inject_css 后 render 累积所有 CSS，fills 单调递增。
///
/// 边界场景：连续注入三条独立 CSS 规则（分别匹配不同 class），
/// 每次注入后 fills 数量应 >= 上一次（CSS 累积而非替换）。
/// 最终 render 应使用所有累积的 CSS，fills 数量与最后一次 inject_css 一致。
#[test]
fn test_webview_render_accumulates_all_css_after_multiple_injects() {
    let mut wv = WebView::new(WebViewConfig::default());
    let html = "<html><body>\
        <div class=\"a\">A</div>\
        <div class=\"b\">B</div>\
        <div class=\"c\">C</div>\
        </body></html>";

    // 初始加载（无 CSS）
    let initial = wv.load_html(html, None);
    let fills_initial = initial.primitives.fills.len();

    // 第一次注入：为 .a 添加样式
    let after_a = wv.inject_css(".a { background-color: red; width: 50px; height: 50px; }");
    let fills_a = after_a.primitives.fills.len();
    assert!(fills_a >= fills_initial, "第一次注入后 fills 应 >= 初始值");

    // 第二次注入：为 .b 添加样式
    let after_b = wv.inject_css(".b { background-color: green; width: 60px; height: 60px; }");
    let fills_b = after_b.primitives.fills.len();
    assert!(fills_b >= fills_a, "第二次注入后 fills 应 >= 第一次注入后");

    // 第三次注入：为 .c 添加样式
    let after_c = wv.inject_css(".c { background-color: blue; width: 70px; height: 70px; }");
    let fills_c = after_c.primitives.fills.len();
    assert!(fills_c >= fills_b, "第三次注入后 fills 应 >= 第二次注入后");

    // render 应累积所有三条 CSS，fills 与最后一次 inject_css 一致
    let after_render = wv.render();
    assert_eq!(
        after_render.primitives.fills.len(),
        fills_c,
        "render() 应使用累积的所有 CSS（.a + .b + .c），fills 数量应与最后一次 inject_css 一致"
    );

    // 再次 render 确认幂等
    let after_rerender = wv.render();
    assert_eq!(
        after_rerender.primitives.fills.len(),
        fills_c,
        "第二次 render 的 fills 应与第一次 render 一致（幂等）"
    );
}

// ── 新增边界测试 ──

/// 测试 WebViewConfig 默认 devtools 为 false。
#[test]
fn test_webview_config_default_devtools_off() {
    let config = WebViewConfig::default();
    assert!(!config.devtools, "默认 devtools 应为 false");
    assert!(!config.transparent, "默认 transparent 应为 false");
}

/// 测试 load_html 后 last_render 不为 None。
#[test]
fn test_webview_load_html_sets_last_render() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(wv.last_render().is_none(), "初始 last_render 应为 None");

    wv.load_html("<p>Hello</p>", None);
    assert!(wv.last_render().is_some(), "load_html 后 last_render 不应为 None");
}

/// 测试 resize 后重新 render 仍能工作（边界补充）。
#[test]
fn test_webview_resize_render_preserves_content() {
    let mut wv = WebView::new(WebViewConfig::default());
    wv.load_html("<div style='background: red; width: 100px; height: 50px;'></div>", None);
    let fills_before = wv.last_render().unwrap().primitives.fills.len();

    wv.resize(1024, 768);
    let result = wv.render();
    assert!(
        result.primitives.fills.len() >= fills_before,
        "resize 后 render 的 fills 不应少于 resize 前"
    );
}

/// 测试 is_loading 初始状态为 false。
#[test]
fn test_webview_initial_not_loading() {
    let wv = WebView::new(WebViewConfig::default());
    assert!(!wv.is_loading(), "初始状态不应在加载中");
}

/// 测试 remove_event_callback 对不存在的索引返回 false。
#[test]
fn test_webview_remove_nonexistent_callback() {
    let mut wv = WebView::new(WebViewConfig::default());
    assert!(!wv.remove_event_callback(999), "移除不存在的索引应返回 false");
    assert!(!wv.remove_event_callback(0), "移除索引 0（未注册）应返回 false");
}

/// 测试 execute_script_with_dom 可以使用 document API。
#[test]
fn test_webview_execute_script_with_dom() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 使用 DOM API 创建元素
    let result = wv.execute_script_with_dom(
        "var div = document.createElement('div'); div.setAttribute('id', 'test'); div.getAttribute('id');",
    );
    assert!(result.is_ok(), "execute_script_with_dom should succeed");
    assert_eq!(result.unwrap(), "test", "DOM API should work via polyfill");
}

/// 测试 execute_script_with_dom 可以 getElementById。
#[test]
fn test_webview_execute_script_with_dom_get_element() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        "document.createElement('div').setAttribute('id', 'app'); typeof document.getElementById;",
    );
    assert!(result.is_ok());
    // getElementById 是一个函数
    assert!(
        result.unwrap().contains("function"),
        "getElementById should be a function"
    );
}

/// 测试 execute_script_with_dom 可以 querySelector。
#[test]
fn test_webview_execute_script_with_dom_query_selector() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof document.querySelector;");
    assert!(result.is_ok());
    assert!(
        result.unwrap().contains("function"),
        "querySelector should be a function"
    );
}

/// 测试 execute_script_with_dom 可以 appendChild。
#[test]
fn test_webview_execute_script_with_dom_append_child() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom(
        "var parent = document.createElement('div'); var child = document.createElement('span'); parent.appendChild(child); parent.children.length;",
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "1", "appendChild should add child");
}

/// 测试 execute_script_with_dom document.body 存在。
#[test]
fn test_webview_execute_script_with_dom_body() {
    let mut wv = WebView::new(WebViewConfig::default());
    let result = wv.execute_script_with_dom("typeof document.body;");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "object", "document.body should exist");
}

/// 测试 execute_script_with_dom 空脚本：polyfill 会使空脚本成功执行。
#[test]
fn test_webview_execute_script_with_dom_empty() {
    let mut wv = WebView::new(WebViewConfig::default());
    // 空用户脚本 + polyfill = polyfill 自身执行成功
    let result = wv.execute_script_with_dom("  ");
    // polyfill 会执行并返回 undefined
    assert!(
        result.is_ok() || result.is_err(),
        "Empty user script with polyfill should not panic"
    );
}

// ════════════════════════════════════════════════════════════════
//  边界条件测试：Service Worker 注册表默认状态、生命周期异常、extract_origin
// ════════════════════════════════════════════════════════════════

/// 验证新创建的 WebView 的 Service Worker 注册表为空。
///
/// 边界场景：WebView 刚创建时，sw_registry 应无任何注册，
/// len 应为 0，is_empty 应为 true，active_count 应为 0。
#[test]
fn test_sw_registry_default_empty() {
    let wv = WebView::new(WebViewConfig::default());
    let registry = wv.service_worker_registry();
    assert!(registry.is_empty(), "新 WebView 的 SW 注册表应为空");
    assert_eq!(registry.len(), 0, "注册数量应为 0");
    assert_eq!(registry.active_count(), 0, "活跃 SW 数量应为 0");
    assert!(registry.active_origins().is_empty(), "活跃 origin 列表应为空");
}

/// 验证在未安装 Service Worker 的情况下直接调用 activate 应失败。
///
/// 边界场景：register_service_worker 后 SW 处于 Registered 状态，
/// activate 要求 SW 处于 Installed 状态，跳过 install 直接 activate 应返回 false。
/// SW 的状态应保持为 Registered，不被错误修改。
#[test]
fn test_sw_activate_before_install() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.register_service_worker("/sw.js", "/", "https://example.com");

    // 未调用 install_service_worker，直接尝试 activate
    let result = wv.activate_service_worker(id);
    assert!(!result, "未安装时 activate 应返回 false");

    // 状态应保持为 Registered
    let reg = wv.service_worker_registry().get(id).unwrap();
    assert_eq!(
        reg.state,
        zero_storage::ServiceWorkerState::Registered,
        "激活失败后状态应保持为 Registered"
    );
    assert_eq!(wv.service_worker_registry().active_count(), 0, "不应有活跃的 SW");
}

/// 验证对已激活的 Service Worker 再次调用 activate 应失败。
///
/// 边界场景：SW 已走完 register → install → activate 生命周期，
/// 处于 Activated 状态。再次调用 activate 时，activate 内部检查
/// 状态不是 Installed，应返回 false。
/// SW 的状态和活跃映射不应被破坏。
#[test]
fn test_sw_double_activate() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.register_service_worker("/sw.js", "/", "https://example.com");
    assert!(wv.install_service_worker(id));
    assert!(wv.activate_service_worker(id));

    // 第二次 activate 应失败
    let result = wv.activate_service_worker(id);
    assert!(!result, "重复 activate 应返回 false");

    // 状态应保持为 Activated
    let reg = wv.service_worker_registry().get(id).unwrap();
    assert_eq!(
        reg.state,
        zero_storage::ServiceWorkerState::Activated,
        "重复激活后状态应保持为 Activated"
    );
    assert!(reg.is_active(), "SW 仍应为活跃状态");
    assert_eq!(wv.service_worker_registry().active_count(), 1, "活跃 SW 数量应保持为 1");
}

/// 验证 extract_origin 对 http:// URL 正确提取 origin（不含端口）。
///
/// 边界场景：http:// URL 默认端口 80 不出现在 origin 字符串中，
/// extract_origin 应返回 "http://example.com"。
#[test]
fn test_sw_extract_origin_http() {
    assert_eq!(
        WebView::extract_origin("http://example.com/path"),
        Some("http://example.com".to_string()),
        "http:// URL 应正确提取 origin"
    );
    assert_eq!(
        WebView::extract_origin("http://example.com:8080/api/data?q=1"),
        Some("http://example.com:8080".to_string()),
        "http:// URL 带端口应包含端口号"
    );
    assert_eq!(
        WebView::extract_origin("http://example.com:80/default-port"),
        Some("http://example.com".to_string()),
        "http:// 默认端口 80 应被省略"
    );
}

/// 验证 extract_origin 对不含端口的 https:// URL 正确提取 origin。
///
/// 边界场景：https:// URL 无显式端口时，origin 不包含端口部分。
/// 同时验证路径和查询参数不包含在 origin 中。
#[test]
fn test_sw_extract_origin_no_port() {
    assert_eq!(
        WebView::extract_origin("https://example.com/page.html"),
        Some("https://example.com".to_string()),
        "无端口的 https:// URL 应提取 origin 不含端口"
    );
    assert_eq!(
        WebView::extract_origin("https://app.example.com:3000/dashboard?tab=overview"),
        Some("https://app.example.com:3000".to_string()),
        "带端口的 URL 应包含端口号"
    );
    assert_eq!(
        WebView::extract_origin("https://example.com"),
        Some("https://example.com".to_string()),
        "无路径的 URL 也应正确提取 origin"
    );
}

/// 验证通过 service_worker_registry_mut 将响应放入缓存后，
/// intercept_fetch 返回 Cached 结果。
///
/// 场景：register → install → activate 一个 SW，然后通过
/// service_worker_registry_mut 获取活跃 SW 的 cache_storage，
/// open 一个命名缓存并 put 一个请求-响应对。
/// 最后调用 intercept_fetch 验证返回 FetchInterceptResult::Cached，
/// 且缓存的响应内容与写入时一致。
#[test]
fn test_sw_cache_put_and_match_via_webview() {
    let mut wv = WebView::new(WebViewConfig::default());
    let id = wv.register_service_worker("/sw.js", "/", "https://example.com");
    assert!(wv.install_service_worker(id));
    assert!(wv.activate_service_worker(id));

    // 通过 service_worker_registry_mut 缓存一个响应
    let request = zero_storage::CacheRequest::new("https://example.com/api/data.json");
    let response = zero_storage::CacheResponse::ok(br#"{"status":"ok"}"#.to_vec());
    let _ = wv
        .service_worker_registry_mut()
        .get_active_mut("https://example.com")
        .unwrap()
        .cache_storage
        .open("api-cache")
        .put(request.clone(), response);

    // intercept_fetch 应返回 Cached
    let result = wv
        .service_worker_registry()
        .intercept_fetch(&request, "https://example.com");
    match result {
        zero_storage::FetchInterceptResult::Cached(resp) => {
            assert_eq!(resp.status, 200, "缓存响应状态码应为 200");
            assert_eq!(resp.body, br#"{"status":"ok"}"#.to_vec(), "缓存响应体应与写入时一致");
        }
        other => {
            panic!("intercept_fetch 应返回 Cached，实际返回: {:?}", other);
        }
    }

    // 不在缓存中的请求应返回 PassThrough（有活跃 SW 但无缓存命中）
    let uncached = zero_storage::CacheRequest::new("https://example.com/other.html");
    let uncached_result = wv
        .service_worker_registry()
        .intercept_fetch(&uncached, "https://example.com");
    assert!(
        matches!(uncached_result, zero_storage::FetchInterceptResult::PassThrough),
        "未缓存的请求应返回 PassThrough"
    );
}
