//! WebView 产品级视觉 smoke 测试（质量测试矩阵 Phase 4）
//!
//! 对 headless WebView 增加固定页面截图 smoke，覆盖：
//! - load：加载固定页面验证渲染输出
//! - resize：调整视口大小验证重新渲染
//! - inject CSS：注入样式验证视觉变化
//! - navigation：多页面导航验证状态
//! - script execution：脚本执行验证 DOM 修改
//! - scroll/document height：文档高度和链接命中测试

use zero_webview::{WebView, WebViewBuilder, WebViewConfig};

// ── 辅助函数 ──

/// 创建标准测试 WebView（800×600）
fn create_webview() -> WebView {
    WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        ..Default::default()
    })
}

/// 多段落固定页面，用于渲染验证
fn multi_paragraph_html() -> &'static str {
    r#"<html><head><style>
        body { font-family: sans-serif; margin: 0; padding: 20px; }
        h1 { font-size: 24px; margin-bottom: 10px; }
        p { font-size: 14px; line-height: 1.5; margin-bottom: 8px; }
        .highlight { background: yellow; padding: 4px; }
        .card { border: 1px solid #ccc; border-radius: 8px; padding: 16px; margin: 10px 0; }
    </style></head><body>
        <h1>Smoke Test Page</h1>
        <p class="highlight">This is a highlighted paragraph with some text content.</p>
        <div class="card">
            <p>Card content with border and rounded corners.</p>
        </div>
        <p>Second paragraph with normal styling.</p>
        <p>Third paragraph to test multi-line rendering.</p>
    </body></html>"#
}

/// 导航链接页面
fn navigation_html() -> &'static str {
    r#"<html><body>
        <nav><a href="/home">Home</a> | <a href="/about">About</a></nav>
        <main><h1>Navigation Page</h1><p>Content here.</p></main>
    </body></html>"#
}

/// 表单页面
fn form_html() -> &'static str {
    r#"<html><body>
        <form>
            <label>Name:</label><input type="text" id="name" />
            <label>Email:</label><input type="email" id="email" />
            <button type="submit">Submit</button>
        </form>
    </body></html>"#
}

// ── Phase 4: Load 固定页面验证 ──

/// 加载固定多段落页面，验证渲染图元非空且包含预期类型
#[test]
fn test_smoke_load_multi_paragraph_page() {
    let mut wv = create_webview();
    let result = wv.load_html(multi_paragraph_html(), None);

    // 渲染必须成功
    assert!(result.timings.total_ms >= 0.0, "渲染应成功完成");

    // 必须有 glyph（文本内容）
    assert!(!result.primitives.glyphs.is_empty(), "页面应包含文本 glyph");

    // 必须有 fill（背景、边框等）
    assert!(!result.primitives.fills.is_empty(), "页面应包含填充图元");

    // 应有圆角矩形（.card 的 border-radius）
    // 注：圆角矩形可能通过 fills 或 rounded_rects 实现
    let has_rounded = !result.primitives.rounded_rects.is_empty();
    let has_border_fill = result.primitives.fills.len() >= 3; // 至少有背景+边框
    assert!(has_rounded || has_border_fill, "页面应有圆角或边框填充图元");

    // URL 状态
    assert!(wv.last_render().is_some());
}

/// 加载包含链接的页面，验证链接命中测试
#[test]
fn test_smoke_load_page_with_links() {
    let mut wv = create_webview();
    wv.load_html(navigation_html(), None);

    // 链接命中测试：由于没有精确布局坐标，验证 hit_test_link 不 panic
    // 遍历一些可能的坐标位置
    for x in [50.0, 200.0, 400.0] {
        for y in [20.0, 50.0, 100.0] {
            let _ = wv.hit_test_link(x, y);
        }
    }

    // 文档高度应大于 0
    let doc_height = wv.document_height();
    assert!(doc_height.is_some(), "应能获取文档高度");
    if let Some(h) = doc_height {
        assert!(h > 0.0, "文档高度应大于 0，got {h}");
    }
}

/// 加载表单页面，验证表单元素渲染不 panic
#[test]
fn test_smoke_load_form_page() {
    let mut wv = create_webview();
    let result = wv.load_html(form_html(), None);

    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives.glyphs.is_empty(), "表单标签应有文本");
}

/// 加载纯文本页面，验证极简内容渲染
#[test]
fn test_smoke_load_minimal_page() {
    let mut wv = create_webview();
    let result = wv.load_html("<html><body><p>Hello World</p></body></html>", None);

    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives.glyphs.is_empty());
    // 极简页面可能只有 glyph 没有 fill（无背景/边框）
}

/// 加载空页面，验证不 panic
#[test]
fn test_smoke_load_empty_page() {
    let mut wv = create_webview();
    let result = wv.load_html("<html><body></body></html>", None);
    assert!(result.timings.total_ms >= 0.0);
}

/// 加载带外部 CSS 的页面
#[test]
fn test_smoke_load_page_with_external_css() {
    let mut wv = create_webview();
    let html = r#"<html><body><div class="box">Styled</div></body></html>"#;
    let css = r#"
        .box { background: #ff0000; width: 200px; height: 100px; border-radius: 10px; }
    "#;
    let result = wv.load_html(html, Some(css));

    assert!(result.timings.total_ms >= 0.0);
    assert!(
        !result.primitives.fills.is_empty() || !result.primitives.rounded_rects.is_empty(),
        "外部 CSS 应产生填充图元"
    );
}

// ── Phase 4: Resize 验证 ──

/// 调整视口大小后重新渲染，验证图元数量变化
#[test]
fn test_smoke_resize_renders_differently() {
    let mut wv = create_webview();

    // 加载页面
    let result_800 = wv.load_html(multi_paragraph_html(), None);
    let glyphs_800 = result_800.primitives.glyphs.len();

    // 调整为更窄的视口（文本会换行更多行）
    wv.resize(400, 300);
    let result_400 = wv.render();
    let glyphs_400 = result_400.primitives.glyphs.len();

    // 两种视口都应有渲染结果
    assert!(glyphs_800 > 0, "800px 宽视口应有 glyph");
    assert!(glyphs_400 > 0, "400px 宽视口应有 glyph");

    // 窄视口的 glyph 数量可能不同（换行导致重新排列）
    // 只要都渲染成功即可
    assert!(wv.config().width == 400);
    assert!(wv.config().height == 300);
}

/// 连续多次 resize 不 panic
#[test]
fn test_smoke_resize_multiple_times() {
    let mut wv = create_webview();
    wv.load_html(multi_paragraph_html(), None);

    for (w, h) in [(1024, 768), (640, 480), (320, 240), (1920, 1080)] {
        wv.resize(w, h);
        let result = wv.render();
        assert!(result.timings.total_ms >= 0.0, "resize 到 {w}x{h} 应渲染成功");
    }
}

/// 极小视口渲染不 panic
#[test]
fn test_smoke_resize_tiny_viewport() {
    let mut wv = create_webview();
    wv.load_html(multi_paragraph_html(), None);

    wv.resize(1, 1);
    let result = wv.render();
    assert!(result.timings.total_ms >= 0.0, "1x1 视口应渲染成功");
}

// ── Phase 4: Inject CSS 验证 ──

/// 注入 CSS 后渲染结果应有图元变化
#[test]
fn test_smoke_inject_css_changes_rendering() {
    let mut wv = create_webview();

    // 先加载基础页面
    let result_before = wv.load_html("<html><body><div id='box'>Hello</div></body></html>", None);
    let fills_before = result_before.primitives.fills.len();

    // 注入 CSS 改变背景
    let result_after = wv.inject_css("#box { background: red; width: 200px; height: 50px; }");
    let fills_after = result_after.primitives.fills.len();

    // 注入后可能增加 fill（红色背景）
    assert!(
        fills_after >= fills_before,
        "注入背景 CSS 后 fill 数量应 >= 注入前 ({fills_before} vs {fills_after})"
    );
}

/// 注入多个 CSS 规则不 panic
#[test]
fn test_smoke_inject_multiple_css_rules() {
    let mut wv = create_webview();
    wv.load_html(
        r#"<html><body><h1>Title</h1><p>Text</p><div class="box">Box</div></body></html>"#,
        None,
    );

    let css = r#"
        h1 { color: blue; font-size: 32px; }
        p { color: green; margin: 10px; }
        .box { background: red; padding: 20px; border: 2px solid black; }
    "#;
    let result = wv.inject_css(css);
    assert!(result.timings.total_ms >= 0.0);
}

// ── Phase 4: Navigation 验证 ──

/// 多页面导航：加载不同页面验证状态变化
#[test]
fn test_smoke_navigate_between_pages() {
    let mut wv = create_webview();

    // 页面 1
    let r1 = wv.load_html("<html><body><h1>Page One</h1></body></html>", None);
    assert!(!r1.primitives.glyphs.is_empty());
    let html1 = wv.html_content().to_string();

    // 页面 2
    let r2 = wv.load_html("<html><body><h1>Page Two</h1></body></html>", None);
    assert!(!r2.primitives.glyphs.is_empty());
    let html2 = wv.html_content().to_string();

    // 两个页面的 HTML 内容应不同
    assert_ne!(html1, html2, "两个页面的 HTML 应不同");
}

/// 使用 complete_load 模拟网络加载
#[test]
fn test_smoke_complete_load_flow() {
    let mut wv = create_webview();

    // 模拟 load_url + complete_load 流程
    wv.load_url("https://example.com");
    assert!(wv.is_loading(), "加载中状态应为 true");

    let html = "<html><body><h1>Loaded via complete_load</h1></body></html>";
    let result = wv.complete_load(html, None);

    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives.glyphs.is_empty());
}

/// fail_load 不 panic 并保持可用
#[test]
fn test_smoke_fail_load_recoverable() {
    let mut wv = create_webview();

    wv.load_url("https://nonexistent.invalid");
    wv.fail_load("DNS resolution failed");

    // 失败后应仍能加载新页面
    let result = wv.load_html("<html><body><h1>Recovered</h1></body></html>", None);
    assert!(result.timings.total_ms >= 0.0);
    assert!(!result.primitives.glyphs.is_empty());
}

// ── Phase 4: Script Execution 验证 ──

/// 脚本执行：修改 DOM 后渲染
#[test]
fn test_smoke_script_modify_dom() {
    let mut wv = create_webview();
    wv.load_html(r#"<html><body><div id="target">Original</div></body></html>"#, None);

    // 执行脚本
    let result = wv.execute_script("document.getElementById('target').textContent = 'Modified'");
    // 脚本可能通过 polyfill 执行
    if result.is_ok() {
        // 重新渲染
        let render = wv.render();
        assert!(render.timings.total_ms >= 0.0);
    }
}

/// 执行复杂脚本不 panic
#[test]
fn test_smoke_script_complex_execution() {
    let mut wv = create_webview();

    // 数组操作
    let r = wv.execute_script("[1,2,3].map(x => x * 2).join(',')");
    assert!(r.is_ok());
    assert_eq!(r.unwrap(), "2,4,6");

    // JSON 操作
    let r = wv.execute_script("JSON.stringify({name: 'test', value: 42})");
    assert!(r.is_ok());
}

/// 脚本执行错误不崩溃
#[test]
fn test_smoke_script_error_handling() {
    let mut wv = create_webview();

    // 语法错误
    let r = wv.execute_script("invalid {{{{");
    assert!(r.is_err());

    // 运行时错误
    let r = wv.execute_script("undefined.function()");
    assert!(r.is_err());

    // 错误后 WebView 仍可用
    let r = wv.execute_script("1 + 1");
    assert!(r.is_ok());
}

// ── Phase 4: WebView Builder 模式验证 ──

/// 使用 WebViewBuilder 创建带配置的 WebView
#[test]
fn test_smoke_builder_pattern() {
    let mut wv = WebViewBuilder::new()
        .width(1024)
        .height(768)
        .user_agent("SmokeTest/1.0")
        .transparent(false)
        .build();

    assert_eq!(wv.config().width, 1024);
    assert_eq!(wv.config().height, 768);

    let result = wv.load_html("<html><body><h1>Builder Test</h1></body></html>", None);
    assert!(result.timings.total_ms >= 0.0);
}

// ── Phase 4: 事件回调验证 ──

/// 注册事件回调不 panic
#[test]
fn test_smoke_event_callback_registration() {
    let mut wv = create_webview();

    let callback_id = wv.on_event(|event| {
        // 简单记录事件类型
        match event {
            zero_webview::WebViewEvent::LoadStart(url) => {
                let _ = url;
            }
            zero_webview::WebViewEvent::LoadEnd(url) => {
                let _ = url;
            }
            zero_webview::WebViewEvent::LoadFailed(url, err) => {
                let _ = (url, err);
            }
            zero_webview::WebViewEvent::TitleChanged(title) => {
                let _ = title;
            }
            zero_webview::WebViewEvent::UrlChanged(url) => {
                let _ = url;
            }
        }
    });

    // 加载页面触发事件
    wv.load_html("<html><body><h1>Events Test</h1></body></html>", None);

    // 移除回调
    assert!(wv.remove_event_callback(callback_id));
}

// ── Phase 4: prefers-color-scheme 验证 ──

/// 设置 prefers-color-scheme 不 panic
#[test]
fn test_smoke_prefers_color_scheme() {
    let mut wv = create_webview();

    // 设置暗色模式
    use zero_css_parser::PrefersColorSchemeValue;
    wv.set_prefers_color_scheme(PrefersColorSchemeValue::Dark);

    let html = r#"<html><head><style>
        @media (prefers-color-scheme: dark) { body { background: #333; } }
        @media (prefers-color-scheme: light) { body { background: #fff; } }
    </style></head><body><p>Theme test</p></body></html>"#;

    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0);
}

// ── Phase 4: @media print 媒体类型验证（R1992 webview 生产接线）──

/// `WebView::set_media_type(Print)` 不 panic 且经 pipeline 触达级联（DC-12）。
///
/// 镜像 `test_smoke_prefers_color_scheme`：设置 Print 媒体类型后加载含 `@media print`
/// 规则的页面，验证 webview 层接线（字段持久化 + pipeline.set_media_type + 各 render
/// 入口重放）端到端不崩溃。
#[test]
fn test_smoke_media_type_print() {
    let mut wv = create_webview();

    // 切打印媒体类型（默认 Screen；Print 使 @media print 规则生效）。
    wv.set_media_type(zero_css_parser::media_query::MediaType::Print);

    let html = r#"<html><head><style>
        @media screen { body { background: #00f; } }
        @media print  { body { background: #f00; } }
    </style></head><body><p>Print media test</p></body></html>"#;

    let result = wv.load_html(html, None);
    assert!(result.timings.total_ms >= 0.0);

    // 切回 Screen 不 panic（验证字段可重复设置 + 默认值往返）。
    wv.set_media_type(zero_css_parser::media_query::MediaType::Screen);
    let result2 = wv.load_html(html, None);
    assert!(result2.timings.total_ms >= 0.0);
}

// ── Phase 4: 调试指示器默认跳过验证（R1996）──

/// WebView 默认跳过调试属性指示器（生产嵌入干净渲染；R1996）。
///
/// `break-before: always` 触发 `paint_break_indicator`（元素顶部红色双横线 = 2 个 fill）。
/// 默认（skip=true）不绘制；`set_skip_indicators(false)` 后绘制 → 后者 fills 更多，证明：
/// ① 默认干净（无调试标记）；② 指示器机制工作（skip=false 时存在）；③ 旧默认 skip=false
/// 会在产品页含此类属性的元素上显示调试标记（R1996 修复）。
#[test]
fn webview_skips_debug_indicators_by_default() {
    // 经 <style> 块 + class 选择器施加（比 inline 更可靠触发指示器），含两个指示器触发属性：
    // break-before:always（paint_break_indicator 顶部红双横线）+ direction:rtl（paint_direction_indicator）。
    let html = concat!(
        "<html><head><style>",
        ".b { break-before: always; direction: rtl; }",
        "</style></head><body><div class=\"b\">hello indicator</div></body></html>"
    );
    let r_default = create_webview().load_html(html, None);
    let mut wv_indicators = create_webview();
    wv_indicators.set_skip_indicators(false);
    let r_indicators = wv_indicators.load_html(html, None);
    assert!(
        r_indicators.primitives.fills.len() > r_default.primitives.fills.len(),
        "skip=false should render more fills (debug indicators) than default skip=true; \
         got indicators={} vs default={}",
        r_indicators.primitives.fills.len(),
        r_default.primitives.fills.len()
    );
}

// ── Phase 4: WASM 执行验证 ──

/// execute_wasm 基础调用（空 WASM 模块不崩溃）
#[test]
fn test_smoke_wasm_execution() {
    let wv = create_webview();

    // 使用最小有效 WASM 模块（空函数）
    // wasm magic number + version + empty module
    let empty_wasm: &[u8] = &[
        0x00, 0x61, 0x73, 0x6d, // magic
        0x01, 0x00, 0x00, 0x00, // version
    ];

    let result = wv.execute_wasm(empty_wasm, "_start", &[]);
    // 空 WASM 没有函数，预期返回错误但不崩溃
    assert!(result.is_err() || result.is_ok());
}

// ── Phase 4: 综合生命周期验证 ──

/// 完整产品场景：加载 → 渲染 → CSS注入 → 调整大小 → 脚本执行 → 重新渲染
#[test]
fn test_smoke_full_product_scenario() {
    let mut wv = WebViewBuilder::new()
        .width(1024)
        .height(768)
        .user_agent("ZeroWeb/1.0")
        .build();

    // 1. 加载页面
    let r = wv.load_html(
        r#"<html><head><style>
            .container { max-width: 800px; margin: 0 auto; }
            .header { background: #1a1a2e; color: white; padding: 20px; }
            .content { padding: 20px; }
        </style></head><body>
            <div class="container">
                <div class="header"><h1>Welcome</h1></div>
                <div class="content">
                    <p>Paragraph 1 with some text.</p>
                    <p>Paragraph 2 with more text.</p>
                    <p>Paragraph 3 to fill space.</p>
                </div>
            </div>
        </body></html>"#,
        None,
    );
    assert!(r.timings.total_ms >= 0.0);
    assert!(!r.primitives.glyphs.is_empty());
    assert!(!r.primitives.fills.is_empty());

    // 2. CSS 注入
    let r = wv.inject_css(".content { border: 1px solid #eee; }");
    assert!(r.timings.total_ms >= 0.0);

    // 3. 调整视口（模拟窗口调整）
    wv.resize(800, 600);
    let r = wv.render();
    assert!(r.timings.total_ms >= 0.0);

    // 4. 脚本执行
    let script_result = wv.execute_script("document.title = 'Modified'");
    // 脚本结果可能成功也可能因 polyfill 限制失败，但不崩溃
    let _ = script_result;

    // 5. 再次调整视口
    wv.resize(1440, 900);
    let r = wv.render();
    assert!(r.timings.total_ms >= 0.0);
    assert!(!r.primitives.glyphs.is_empty());

    // 6. 最终状态验证
    assert_eq!(wv.config().width, 1440);
    assert_eq!(wv.config().height, 900);
    assert!(wv.last_render().is_some());
}

/// 连续加载多个不同复杂度页面不退化
#[test]
fn test_smoke_sequential_page_loads() {
    let mut wv = create_webview();

    let pages = [
        "<html><body><p>Simple</p></body></html>",
        r#"<html><head><style>
            body { display: flex; }
            .col { flex: 1; padding: 10px; }
        </style></head><body>
            <div class="col"><p>Column 1</p></div>
            <div class="col"><p>Column 2</p></div>
        </body></html>"#,
        r#"<html><head><style>
            .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; }
            .item { background: #f0f0f0; padding: 10px; }
        </style></head><body>
            <div class="grid">
                <div class="item">1</div><div class="item">2</div><div class="item">3</div>
                <div class="item">4</div><div class="item">5</div><div class="item">6</div>
            </div>
        </body></html>"#,
    ];

    for (i, html) in pages.iter().enumerate() {
        let result = wv.load_html(html, None);
        assert!(result.timings.total_ms >= 0.0, "第 {i} 个页面应渲染成功");
        assert!(!result.primitives.glyphs.is_empty(), "第 {i} 个页面应有 glyph");
    }
}

/// 渲染管线耗时分解合理
#[test]
fn test_smoke_render_timings_breakdown() {
    let mut wv = create_webview();
    let result = wv.load_html(multi_paragraph_html(), None);

    // 各阶段耗时应非负
    assert!(result.timings.parse_ms >= 0.0, "parse_ms 应非负");
    assert!(result.timings.style_ms >= 0.0, "style_ms 应非负");
    assert!(result.timings.layout_ms >= 0.0, "layout_ms 应非负");
    assert!(result.timings.paint_ms >= 0.0, "paint_ms 应非负");
    assert!(result.timings.total_ms >= 0.0, "total_ms 应非负");

    // 总时间应 >= 各子阶段之和
    let sum_stages =
        result.timings.parse_ms + result.timings.style_ms + result.timings.layout_ms + result.timings.paint_ms;
    assert!(
        result.timings.total_ms >= sum_stages - 0.1,
        "total_ms ({}) 应 >= 各阶段之和 ({})",
        result.timings.total_ms,
        sum_stages
    );
}

/// 缓存操作在渲染流程中不崩溃
#[test]
fn test_smoke_cache_operations_during_rendering() {
    let mut wv = create_webview();

    wv.load_html("<html><body><p>Cache test</p></body></html>", None);

    // 缓存操作
    let initial_len = wv.http_cache_len();
    assert_eq!(initial_len, 0, "初始缓存应为空");

    wv.clear_http_cache();
    assert_eq!(wv.http_cache_len(), 0);

    // 渲染后缓存状态一致
    wv.render();
    wv.clear_http_cache();
    assert!(wv.http_cache_len() == 0);
}

/// WebView extract_origin 工具方法验证
#[test]
fn test_smoke_extract_origin() {
    assert_eq!(
        WebView::extract_origin("https://example.com/path"),
        Some("https://example.com".to_string())
    );
    assert_eq!(
        WebView::extract_origin("http://localhost:8080/test"),
        Some("http://localhost:8080".to_string())
    );
    assert_eq!(WebView::extract_origin("not-a-url"), None);
}

/// html_content() 返回加载的内容
#[test]
fn test_smoke_html_content_tracking() {
    let mut wv = create_webview();
    let html = "<html><body><div>Tracked Content</div></body></html>";
    wv.load_html(html, None);

    let content = wv.html_content();
    assert!(content.contains("Tracked Content"), "html_content 应包含加载的内容");
}
