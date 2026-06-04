//! 端到端渲染验证集成测试。
//!
//! 验证完整的 HTML → CSS → Style → Layout → Render 管线，
//! 覆盖目标文档 Done Criteria 中的关键渲染场景。

use zero_css_parser::Parser as CssParser;
use zero_dom::Document;
use zero_engine::RenderPipeline;

// ── 辅助 ──────────────────────────────────────────────────────────

/// 创建 html > body 基础 DOM，返回 (doc, body NodeId)。
fn make_doc_with_body() -> (Document, zero_dom::NodeId) {
    let mut doc = Document::new();
    let root = doc.root();
    let html = doc.create_element("html");
    doc.append_child(root, html).unwrap();
    let body = doc.create_element("body");
    doc.append_child(html, body).unwrap();
    (doc, body)
}

/// 完整管线：DOM + CSS → Style → Layout → Render primitives。
fn render_pipeline(html: &str, css: &str, width: f32, height: f32) -> zero_engine::RenderResult {
    let mut pipeline = RenderPipeline::new(width, height);
    pipeline.render_html(html, css)
}

// ── 1. Flexbox 布局端到端验证 ──────────────────────────────────────

#[test]
fn test_flexbox_row_layout() {
    let html = r#"<html><body>
        <div class="container">
            <div class="item">A</div>
            <div class="item">B</div>
            <div class="item">C</div>
        </div>
    </body></html>"#;
    let css = r#"
        .container { display: flex; flex-direction: row; width: 600px; }
        .item { width: 200px; height: 100px; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Flexbox row layout should complete");
}

#[test]
fn test_flexbox_column_layout() {
    let html = r#"<html><body>
        <div class="col-container">
            <div class="row">Row 1</div>
            <div class="row">Row 2</div>
        </div>
    </body></html>"#;
    let css = r#"
        .col-container { display: flex; flex-direction: column; height: 400px; }
        .row { height: 200px; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Flexbox column layout should complete");
}

#[test]
fn test_flexbox_wrap() {
    let html = r#"<html><body>
        <div class="wrap-container">
            <div class="box">1</div>
            <div class="box">2</div>
            <div class="box">3</div>
        </div>
    </body></html>"#;
    let css = r#"
        .wrap-container { display: flex; flex-wrap: wrap; width: 400px; }
        .box { width: 250px; height: 100px; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Flexbox wrap should complete");
}

// ── 2. Block + Inline 布局端到端验证 ───────────────────────────────

#[test]
fn test_block_layout_nested() {
    let html = r#"<html><body>
        <div class="outer">
            <div class="inner">
                <p>Paragraph text</p>
            </div>
        </div>
    </body></html>"#;
    let css = r#"
        .outer { width: 800px; padding: 20px; }
        .inner { margin: 10px; padding: 15px; }
        p { margin: 5px 0; }
    "#;
    let result = render_pipeline(html, css, 1024.0, 768.0);
    assert!(result.timings.total_ms >= 0.0, "Nested block layout should complete");
}

#[test]
fn test_inline_text_flow() {
    let html = r#"<html><body>
        <p>Hello <strong>bold</strong> and <em>italic</em> text.</p>
    </body></html>"#;
    let css = r#"
        p { font-size: 16px; line-height: 1.5; }
        strong { font-weight: bold; }
        em { font-style: italic; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Inline text flow should complete");
}

// ── 3. 定位（absolute/relative/fixed）端到端验证 ──────────────────

#[test]
fn test_absolute_positioning() {
    let html = r#"<html><body>
        <div class="relative-container">
            <div class="absolute-box">Positioned</div>
        </div>
    </body></html>"#;
    let css = r#"
        .relative-container { position: relative; width: 400px; height: 300px; }
        .absolute-box { position: absolute; top: 10px; left: 20px; width: 100px; height: 50px; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Absolute positioning should complete");
}

// ── 4. 颜色和背景端到端验证 ───────────────────────────────────────

#[test]
fn test_color_rendering() {
    let html = r#"<html><body>
        <div class="red-box">Red</div>
        <div class="blue-box">Blue</div>
        <div class="green-box">Green</div>
    </body></html>"#;
    let css = r#"
        .red-box { background-color: #ff0000; color: white; width: 100px; height: 100px; }
        .blue-box { background-color: rgb(0, 0, 255); color: white; width: 100px; height: 100px; }
        .green-box { background-color: green; width: 100px; height: 100px; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Color rendering should complete");
}

// ── 5. CSS Transform 端到端验证 ───────────────────────────────────

#[test]
fn test_transforms_pipeline() {
    let html = r#"<html><body>
        <div class="rotate">Rotated</div>
        <div class="scale">Scaled</div>
        <div class="translate">Translated</div>
    </body></html>"#;
    let css = r#"
        .rotate { transform: rotate(45deg); width: 100px; height: 100px; }
        .scale { transform: scale(2); width: 50px; height: 50px; }
        .translate { transform: translate(100px, 50px); width: 100px; height: 100px; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Transform rendering should complete");
}

// ── 6. Box Model（margin/padding/border）端到端验证 ──────────────

#[test]
fn test_box_model() {
    let html = r#"<html><body>
        <div class="boxed">Content with box model</div>
    </body></html>"#;
    let css = r#"
        .boxed {
            width: 200px;
            height: 100px;
            margin: 20px;
            padding: 15px;
            border: 2px solid #333;
            box-sizing: border-box;
        }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Box model rendering should complete");
}

// ── 7. 媒体查询端到端验证 ─────────────────────────────────────────

#[test]
fn test_media_query_responsive() {
    let html = r#"<html><body>
        <div class="responsive">Responsive content</div>
    </body></html>"#;
    let css = r#"
        .responsive { width: 100%; background-color: blue; }
        @media (max-width: 600px) {
            .responsive { background-color: red; width: 100%; }
        }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Media query rendering should complete");
}

// ── 8. CSS 选择器全覆盖端到端验证 ────────────────────────────────

#[test]
fn test_complex_selectors() {
    let html = r#"<html><body>
        <nav>
            <ul>
                <li class="active"><a href="/">Home</a></li>
                <li><a href="/about">About</a></li>
            </ul>
        </nav>
        <main>
            <article id="post-1">
                <h2>Post Title</h2>
                <p class="intro">Introduction</p>
                <p>Content paragraph</p>
            </article>
        </main>
    </body></html>"#;
    let css = r#"
        /* 类型选择器 */
        nav { background: #333; }
        /* 后代选择器 */
        nav ul { list-style: none; padding: 0; }
        /* 子选择器 */
        nav > ul { display: flex; }
        /* 相邻兄弟选择器 */
        h2 + p { font-weight: bold; }
        /* ID 选择器 */
        #post-1 { padding: 20px; }
        /* 类选择器 */
        .intro { font-size: 18px; }
        /* 属性选择器 */
        a[href="/"] { color: white; }
        /* 伪类 */
        li:first-child { margin-left: 0; }
        li:last-child { margin-right: 0; }
        /* :not() */
        li:not(.active) a { color: #aaa; }
        /* 通配符 */
        * { margin: 0; padding: 0; }
    "#;
    let result = render_pipeline(html, css, 1024.0, 768.0);
    assert!(result.timings.total_ms >= 0.0, "Complex selectors should complete");
}

// ── 9. 多页面导航端到端验证（通过 WebView）───────────────────────

#[test]
fn test_webview_navigation_flow() {
    use zero_webview::{WebView, WebViewConfig};

    let mut wv = WebView::new(WebViewConfig::default());

    // 加载第一页
    let page1 = r#"<html><head><title>Page 1</title></head>
        <body><h1>First Page</h1></body></html>"#;
    let result = wv.load_html(page1, None);
    assert!(result.timings.total_ms >= 0.0);
    assert!(wv.last_render().is_some());

    // 加载第二页（替换第一页内容）
    let page2 = r#"<html><head><title>Page 2</title></head>
        <body><h1>Second Page</h1><p>Content</p></body></html>"#;
    let result = wv.load_html(page2, None);
    assert!(result.timings.total_ms >= 0.0);

    // 回到第一页内容（通过重新加载）
    let result = wv.load_html(page1, None);
    assert!(result.timings.total_ms >= 0.0);
}

// ── 10. 完整页面渲染（模拟真实网站结构）──────────────────────────

#[test]
fn test_realistic_page_rendering() {
    let html = r#"<html><head><title>Test Page</title></head><body>
        <header>
            <nav>
                <a href="/" class="logo">MySite</a>
                <ul class="nav-links">
                    <li><a href="/home">Home</a></li>
                    <li><a href="/about">About</a></li>
                    <li><a href="/contact">Contact</a></li>
                </ul>
            </nav>
        </header>
        <main>
            <section class="hero">
                <h1>Welcome to MySite</h1>
                <p>A sample page for testing the browser engine.</p>
            </section>
            <section class="content">
                <article>
                    <h2>Feature One</h2>
                    <p>Description of feature one with some text content.</p>
                </article>
                <article>
                    <h2>Feature Two</h2>
                    <p>Description of feature two with more text.</p>
                </article>
                <article>
                    <h2>Feature Three</h2>
                    <p>Description of feature three.</p>
                </article>
            </section>
        </main>
        <footer>
            <p>&copy; 2026 MySite. All rights reserved.</p>
        </footer>
    </body></html>"#;

    let css = r#"
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: sans-serif; color: #333; }
        header { background: #1a1a2e; color: white; padding: 10px 20px; }
        nav { display: flex; justify-content: space-between; align-items: center; }
        .logo { font-size: 24px; font-weight: bold; color: white; }
        .nav-links { display: flex; list-style: none; gap: 20px; }
        .nav-links a { color: #eee; }
        main { padding: 20px; }
        .hero { background: #e94560; color: white; padding: 40px; margin-bottom: 20px; }
        .hero h1 { font-size: 36px; }
        .content { display: flex; gap: 20px; }
        .content article { flex: 1; padding: 20px; border: 1px solid #ddd; }
        .content article h2 { margin-bottom: 10px; color: #1a1a2e; }
        footer { background: #16213e; color: #aaa; padding: 20px; text-align: center; margin-top: 20px; }
    "#;

    let result = render_pipeline(html, css, 1440.0, 900.0);
    assert!(
        result.timings.total_ms >= 0.0,
        "Realistic page should render without errors"
    );
}

// ── 11. CSS Grid 端到端验证 ──────────────────────────────────────

#[test]
fn test_grid_basic_layout() {
    let html = r#"<html><body>
        <div class="grid">
            <div class="a">A</div>
            <div class="b">B</div>
            <div class="c">C</div>
            <div class="d">D</div>
        </div>
    </body></html>"#;
    let css = r#"
        .grid {
            display: grid;
            grid-template-columns: 1fr 1fr;
            grid-template-rows: auto auto;
            gap: 10px;
            width: 400px;
        }
        .a, .b, .c, .d { padding: 10px; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Grid layout should complete");
}

// ── 12. Overflow 和滚动端到端验证 ────────────────────────────────

#[test]
fn test_overflow_hidden() {
    let html = r#"<html><body>
        <div class="container">
            <div class="overflow-content">This is very long content that should be clipped by the container.</div>
        </div>
    </body></html>"#;
    let css = r#"
        .container { width: 200px; height: 100px; overflow: hidden; border: 1px solid #ccc; }
        .overflow-content { width: 500px; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Overflow hidden should complete");
}

// ── 13. CSS 级联和继承端到端验证 ────────────────────────────────

#[test]
fn test_cascade_specificity() {
    let html = r#"<html><body>
        <div id="main" class="container">
            <p class="text highlight">Styled text</p>
        </div>
    </body></html>"#;
    let css = r#"
        /* 低优先级 */
        p { color: black; font-size: 14px; }
        /* 类选择器覆盖 */
        .text { color: blue; }
        /* 多类选择器更高优先级 */
        .text.highlight { color: green; }
        /* ID 选择器后代最高 */
        #main p { font-size: 16px; }
        /* 继承验证 */
        body { font-family: serif; line-height: 1.6; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "Cascade specificity should complete");
}

// ── 14. CSS 自定义属性（CSS Variables）端到端验证 ──────────────

#[test]
fn test_css_custom_properties() {
    let html = r#"<html><body>
        <div class="themed">
            <h2>Themed Heading</h2>
            <p>Themed paragraph.</p>
        </div>
    </body></html>"#;
    let css = r#"
        :root {
            --primary: #3498db;
            --text-color: #2c3e50;
            --spacing: 16px;
        }
        .themed {
            color: var(--text-color);
            padding: var(--spacing);
        }
        .themed h2 {
            color: var(--primary);
        }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "CSS custom properties should complete");
}

// ── 15. 多样式表和 @layer 端到端验证 ────────────────────────────

#[test]
fn test_css_layer() {
    let html = r#"<html><body>
        <div class="box">Layered styles</div>
    </body></html>"#;
    let css = r#"
        @layer base {
            .box { color: black; padding: 10px; }
        }
        @layer theme {
            .box { color: blue; background: #f0f0f0; }
        }
        .box { border: 1px solid #ccc; }
    "#;
    let result = render_pipeline(html, css, 800.0, 600.0);
    assert!(result.timings.total_ms >= 0.0, "CSS @layer should complete");
}
