//! Web 平台扩展合规性测试。
//!
//! 覆盖 CSS 滤镜、变换 3D、表单元素、ARIA 可访问性、
//! 安全策略、响应式设计、CSS 容器查询、交互伪类等。

use super::TestCase;

/// 返回 Web 平台扩展合规性测试用例。
pub fn web_platform_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  CSS 滤镜
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-filter-blur".to_string(),
            description: "CSS blur filter".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#e74c3c;filter:blur(2px);">Blurred</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-filter-brightness".to_string(),
            description: "CSS brightness filter".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#333;filter:brightness(2);">Bright</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-filter-grayscale".to_string(),
            description: "CSS grayscale filter".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#e74c3c;filter:grayscale(100%);">Gray</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-filter-sepia".to_string(),
            description: "CSS sepia filter".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#fff;filter:sepia(100%);">Sepia</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-filter-drop-shadow".to_string(),
            description: "CSS drop-shadow filter".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#e74c3c;filter:drop-shadow(4px 4px 6px rgba(0,0,0,0.5));">Shadow</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS Transform 3D
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-transform-rotateX".to_string(),
            description: "CSS rotateX 3D transform".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#3498db;transform:rotateX(45deg);">3D X</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-transform-rotateY".to_string(),
            description: "CSS rotateY 3D transform".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#2ecc71;transform:rotateY(30deg);">3D Y</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-transform-perspective".to_string(),
            description: "CSS perspective property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="perspective:500px;">
                <div style="width:200px;height:100px;background:#9b59b6;transform:rotateY(45deg);">Perspective</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-transform-origin".to_string(),
            description: "CSS transform-origin".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#e67e22;transform:rotate(30deg);transform-origin:top left;">Origin</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 混合模式
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-mix-blend-mode".to_string(),
            description: "CSS mix-blend-mode".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#e74c3c;">
                <div style="width:100px;height:50px;background:#3498db;mix-blend-mode:multiply;">Blend</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_multiple_fills".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  表单元素
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/form-input-types".to_string(),
            description: "HTML input types (email, tel, date, number)".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <form>
                <input type="email" placeholder="Email">
                <input type="tel" placeholder="Phone">
                <input type="date">
                <input type="number" min="0" max="100">
                <input type="password" placeholder="Password">
                <input type="search" placeholder="Search">
                <input type="url" placeholder="URL">
                <input type="color">
            </form>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_form".to_string(), "dom_has_input".to_string()],
        },
        TestCase {
            id: "web-platform/form-textarea-select".to_string(),
            description: "HTML textarea and select elements".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <form>
                <textarea rows="4" cols="30">Multi-line text input</textarea>
                <select>
                    <option value="a">Option A</option>
                    <option value="b">Option B</option>
                    <option value="c">Option C</option>
                </select>
            </form>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_form".to_string()],
        },
        TestCase {
            id: "web-platform/form-fieldset-legend".to_string(),
            description: "HTML fieldset and legend elements".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <fieldset>
                <legend>Personal Info</legend>
                <label>Name: <input type="text"></label>
                <label>Email: <input type="email"></label>
            </fieldset>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "web-platform/form-datalist-output".to_string(),
            description: "HTML datalist and output elements".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <form>
                <input list="colors" type="text">
                <datalist id="colors">
                    <option value="Red">
                    <option value="Green">
                    <option value="Blue">
                </datalist>
                <output name="result">0</output>
            </form>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_form".to_string()],
        },
        TestCase {
            id: "web-platform/form-progress-meter".to_string(),
            description: "HTML progress and meter elements".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <label>Progress: <progress value="70" max="100">70%</progress></label>
            <label>Score: <meter min="0" max="100" low="30" high="80" optimum="90" value="85">85</meter></label>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  ARIA 可访问性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/aria-roles".to_string(),
            description: "ARIA role attributes".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
            <div role="navigation" aria-label="Main">
                <a href="/home">Home</a>
            </div>
            <div role="main">
                <article role="article">
                    <h2>Title</h2>
                    <p role="text">Content</p>
                </article>
            </div>
            <div role="contentinfo">
                <p>Footer</p>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_link".to_string()],
        },
        TestCase {
            id: "web-platform/aria-live-region".to_string(),
            description: "ARIA live region attributes".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
            <div aria-live="polite" aria-atomic="true">
                <p>Status message</p>
            </div>
            <div role="alert" aria-live="assertive">
                <p>Warning message</p>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "web-platform/aria-expanded-controls".to_string(),
            description: "ARIA expanded/controls attributes".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
            <button aria-expanded="false" aria-controls="panel1">Toggle</button>
            <div id="panel1" role="region" aria-hidden="true">
                <p>Hidden content</p>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_button".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  安全相关 HTML
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/security-meta-csp".to_string(),
            description: "Content-Security-Policy meta tag".to_string(),
            category: "html".to_string(),
            html: r#"<html><head>
            <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self'">
            </head><body>
            <p>CSP via meta tag</p>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "web-platform/security-sandbox-iframe".to_string(),
            description: "Sandboxed iframe element".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <iframe sandbox="allow-scripts" srcdoc="<p>Sandboxed</p>" width="300" height="100"></iframe>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "web-platform/security-referrer-policy".to_string(),
            description: "Referrer-Policy meta tag".to_string(),
            category: "html".to_string(),
            html: r#"<html><head>
            <meta name="referrer" content="no-referrer">
            </head><body>
            <a href="https://example.com" rel="noopener noreferrer">Link</a>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_link".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS Container Queries
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-container-inline-size".to_string(),
            description: "CSS container-type inline-size".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div class="card-container" style="container-type:inline-size;container-name:card;width:300px;background:#eee;padding:10px;">
                <div class="card" style="background:#3498db;color:white;padding:10px;">
                    <p>Card content</p>
                </div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-container-query-style".to_string(),
            description: "CSS @container style query".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="container-type:inline-size;container-name:sidebar;width:200px;background:#f0f0f0;padding:10px;">
                <div style="background:#e74c3c;color:white;padding:10px;">Sidebar item</div>
            </div>
            </body></html>"#.to_string(),
            css: r#"@container sidebar (min-width: 100px) { div { font-size: 14px; } }"#.to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS Scroll Snap
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-scroll-snap-container".to_string(),
            description: "CSS scroll-snap-type on container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:300px;height:200px;overflow:auto;scroll-snap-type:y mandatory;">
                <div style="height:200px;background:#e74c3c;scroll-snap-align:start;">Slide 1</div>
                <div style="height:200px;background:#3498db;scroll-snap-align:start;">Slide 2</div>
                <div style="height:200px;background:#2ecc71;scroll-snap-align:start;">Slide 3</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 自定义属性高级用法
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-custom-props-cascade".to_string(),
            description: "CSS custom properties cascade through DOM tree".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="--spacing:20px;--color:#e74c3c;background:#f0f0f0;padding:var(--spacing);">
                <div style="background:var(--color);padding:var(--spacing);color:white;">Nested</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-custom-props-calc".to_string(),
            description: "CSS custom properties with calc()".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="--base:100px;width:calc(var(--base) * 2);height:var(--base);background:#9b59b6;color:white;">
                Double width
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 高级 Grid 布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-grid-auto-fill".to_string(),
            description: "CSS grid with auto-fill and minmax".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(120px,1fr));gap:10px;padding:10px;">
                <div style="background:#e74c3c;height:80px;">1</div>
                <div style="background:#3498db;height:80px;">2</div>
                <div style="background:#2ecc71;height:80px;">3</div>
                <div style="background:#f39c12;height:80px;">4</div>
                <div style="background:#9b59b6;height:80px;">5</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-grid-span".to_string(),
            description: "CSS grid items spanning multiple columns".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:5px;padding:10px;">
                <div style="grid-column:span 2;background:#e74c3c;height:60px;">Wide</div>
                <div style="background:#3498db;height:60px;">Normal</div>
                <div style="background:#2ecc71;height:60px;">1</div>
                <div style="background:#f39c12;height:60px;">2</div>
                <div style="background:#9b59b6;height:60px;">3</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 响应式布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-responsive-card-grid".to_string(),
            description: "Responsive card grid with auto-fill".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:15px;padding:20px;">
                <div style="background:#fff;border:1px solid #ddd;border-radius:8px;padding:15px;">
                    <h3>Card 1</h3><p>Description</p>
                </div>
                <div style="background:#fff;border:1px solid #ddd;border-radius:8px;padding:15px;">
                    <h3>Card 2</h3><p>Description</p>
                </div>
                <div style="background:#fff;border:1px solid #ddd;border-radius:8px;padding:15px;">
                    <h3>Card 3</h3><p>Description</p>
                </div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string(), "has_glyph_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  HTML5 语义结构
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/html5-complete-page".to_string(),
            description: "Complete HTML5 semantic page structure".to_string(),
            category: "html".to_string(),
            html: r#"<!DOCTYPE html>
            <html lang="en"><head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Complete Page</title>
                <link rel="stylesheet" href="style.css">
            </head><body>
                <header>
                    <nav><a href="/">Home</a><a href="/about">About</a></nav>
                </header>
                <main>
                    <article>
                        <h1>Article Title</h1>
                        <p>Article content with <strong>bold</strong> and <em>italic</em>.</p>
                        <section><h2>Section</h2><p>Section content</p></section>
                    </article>
                    <aside><h3>Related</h3><p>Sidebar</p></aside>
                </main>
                <footer><p>&copy; 2026</p></footer>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "dom_has_head".to_string(),
                "dom_has_title".to_string(),
                "dom_has_link".to_string(),
                "dom_has_nav".to_string(),
                "dom_has_article".to_string(),
                "dom_has_section".to_string(),
                "dom_has_footer".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 高级视觉效果
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-gradient-linear-angle".to_string(),
            description: "CSS linear-gradient with angle".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:300px;height:100px;background:linear-gradient(135deg,#667eea 0%,#764ba2 100%);"></div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-gradient-radial-custom".to_string(),
            description: "CSS radial-gradient with custom size".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:300px;height:200px;background:radial-gradient(ellipse at 30% 50%,#f093fb 0%,#f5576c 100%);"></div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-gradient-multi-stop".to_string(),
            description: "CSS gradient with multiple color stops".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:300px;height:100px;background:linear-gradient(90deg,#ff0000 0%,#ff7f00 14%,#ffff00 28%,#00ff00 42%,#0000ff 57%,#4b0082 71%,#9400d3 100%);"></div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 定位和层叠
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-sticky-header".to_string(),
            description: "CSS position:sticky header".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="height:400px;overflow:auto;">
                <div style="position:sticky;top:0;background:#2d3436;color:white;padding:10px;z-index:10;">Sticky Header</div>
                <div style="height:600px;background:#f0f0f0;padding:10px;">
                    <p>Scrollable content area</p>
                </div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string(), "has_glyph_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-fixed-footer".to_string(),
            description: "CSS position:fixed footer bar".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="min-height:100vh;padding:20px;">
                <p>Page content</p>
            </div>
            <div style="position:fixed;bottom:0;left:0;right:0;background:#2d3436;color:white;padding:10px;text-align:center;">
                Fixed Footer Bar
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS contain 属性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-contain-strict".to_string(),
            description: "CSS contain:strict".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="contain:strict;width:200px;height:100px;background:#e74c3c;color:white;padding:10px;">
                Contained
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-contain-layout".to_string(),
            description: "CSS contain:layout".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="contain:layout;width:300px;background:#f0f0f0;padding:20px;">
                <div style="background:#3498db;height:50px;">Child</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS will-change 和 isolation
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-will-change".to_string(),
            description: "CSS will-change property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="will-change:transform;width:200px;height:100px;background:#9b59b6;">Animated</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-isolation-isolate".to_string(),
            description: "CSS isolation:isolate".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="isolation:isolate;background:#f0f0f0;padding:20px;">
                <div style="background:#e74c3c;mix-blend-mode:multiply;">Isolated</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  HTML details/summary 和 dialog
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/html-details-summary".to_string(),
            description: "HTML details/summary disclosure element".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <details>
                <summary>Click to expand</summary>
                <p>Hidden content revealed on click</p>
            </details>
            <details open>
                <summary>Already open</summary>
                <p>Visible content</p>
            </details>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "web-platform/html-dialog-element".to_string(),
            description: "HTML dialog element".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <dialog open>
                <form method="dialog">
                    <p>Dialog content</p>
                    <button value="ok">OK</button>
                </form>
            </dialog>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_form".to_string(), "dom_has_button".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  HTML template 和 slot
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/html-template-slot".to_string(),
            description: "HTML template element".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <template id="my-template">
                <div class="card">
                    <h3>Template Title</h3>
                    <p>Template content</p>
                </div>
            </template>
            <p>Template defined above</p>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_paragraph".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  HTML picture 和 source
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/html-picture-source".to_string(),
            description: "HTML picture and source elements".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <picture>
                <source media="(min-width: 800px)" srcset="large.jpg">
                <source media="(min-width: 400px)" srcset="medium.jpg">
                <img src="small.jpg" alt="Responsive image" width="300" height="200">
            </picture>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_img".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS @supports
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-supports-display-grid".to_string(),
            description: "CSS @supports for display:grid".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="padding:10px;background:#eee;">No grid support</div>
            </body></html>"#.to_string(),
            css: r#"@supports (display: grid) { div { background: #2ecc71 !important; } }"#.to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  综合布局测试
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/layout-admin-dashboard".to_string(),
            description: "Admin dashboard layout with sidebar + grid cards".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body style="margin:0;font-family:sans-serif;">
            <div style="display:flex;min-height:100vh;">
                <nav style="width:200px;background:#2d3436;color:white;padding:15px;">
                    <h3>Dashboard</h3>
                    <ul><li>Home</li><li>Users</li><li>Settings</li></ul>
                </nav>
                <main style="flex:1;padding:20px;background:#f5f6fa;">
                    <h1>Welcome</h1>
                    <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:15px;">
                        <div style="background:white;padding:15px;border-radius:8px;"><h4>Stat 1</h4><p>1,234</p></div>
                        <div style="background:white;padding:15px;border-radius:8px;"><h4>Stat 2</h4><p>567</p></div>
                        <div style="background:white;padding:15px;border-radius:8px;"><h4>Stat 3</h4><p>89%</p></div>
                        <div style="background:white;padding:15px;border-radius:8px;"><h4>Stat 4</h4><p>42</p></div>
                    </div>
                </main>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        TestCase {
            id: "web-platform/layout-magazine".to_string(),
            description: "Magazine layout with multi-column text".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
            <div style="padding:20px;">
                <h1 style="text-align:center;">Magazine Title</h1>
                <div style="column-count:3;column-gap:20px;column-rule:1px solid #ddd;">
                    <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
                    <p>Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.</p>
                    <p>Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.</p>
                </div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string(), "has_glyph_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS overflow 和 text-overflow
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-text-overflow-ellipsis".to_string(),
            description: "CSS text-overflow:ellipsis on truncated text".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;background:#f0f0f0;padding:10px;">
                This is a very long text that should be truncated with an ellipsis at the end
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string(), "has_glyph_primitives".to_string()],
        },
        TestCase {
            id: "web-platform/css-overflow-auto".to_string(),
            description: "CSS overflow:auto on container".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:300px;height:100px;overflow:auto;background:#f0f0f0;padding:10px;">
                <div style="width:500px;height:300px;background:#e74c3c;">Large content that overflows</div>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 多重背景和背景属性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-background-size-cover".to_string(),
            description: "CSS background-size:cover".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:300px;height:200px;background:#e74c3c;background-size:cover;background-repeat:no-repeat;"></div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  JS Web API 存在性检测
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/js-api-notification".to_string(),
            description: "Notification API availability".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
            <script>
            var hasNotification = typeof Notification !== 'undefined';
            var hasPermission = typeof Notification !== 'undefined' ? Notification.permission : 'unavailable';
            </script>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "web-platform/js-api-geolocation".to_string(),
            description: "Geolocation API availability".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
            <script>
            var hasGeo = 'geolocation' in navigator;
            </script>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "web-platform/js-api-clipboard".to_string(),
            description: "Clipboard API availability".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
            <script>
            var hasClipboard = typeof navigator.clipboard !== 'undefined';
            </script>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "web-platform/js-api-performance".to_string(),
            description: "Performance API availability".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
            <script>
            var hasPerf = typeof performance !== 'undefined';
            if (hasPerf) {
                var now = performance.now();
                var entries = performance.getEntries();
            }
            </script>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        TestCase {
            id: "web-platform/js-api-mutation-observer".to_string(),
            description: "MutationObserver API".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
            <div id="target"></div>
            <script>
            if (typeof MutationObserver !== 'undefined') {
                var observer = new MutationObserver(function(mutations) {
                    mutations.forEach(function(m) { var type = m.type; });
                });
                var target = document.getElementById('target');
                if (target) {
                    observer.observe(target, {childList: true, attributes: true});
                }
            }
            </script>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS @layer 级联层
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-layer-cascade".to_string(),
            description: "CSS @layer cascade order".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div class="box" style="width:200px;height:100px;padding:10px;">Layered</div>
            </body></html>"#.to_string(),
            css: r#"@layer base, components;
            @layer base { .box { background: #ccc; padding: 5px; } }
            @layer components { .box { background: #3498db; color: white; } }"#.to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS aspect-ratio
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-aspect-ratio".to_string(),
            description: "CSS aspect-ratio property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:300px;aspect-ratio:16/9;background:#9b59b6;color:white;">
                16:9 Box
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string(), "has_glyph_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  HTML table 完整结构
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/html-table-complete".to_string(),
            description: "Complete HTML table with caption, thead, tfoot".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <table border="1">
                <caption>Sales Report 2026</caption>
                <colgroup>
                    <col style="background:#f0f0f0">
                    <col span="2">
                </colgroup>
                <thead>
                    <tr><th>Product</th><th>Q1</th><th>Q2</th></tr>
                </thead>
                <tbody>
                    <tr><td>Widget A</td><td>$1,200</td><td>$1,500</td></tr>
                    <tr><td>Widget B</td><td>$800</td><td>$950</td></tr>
                </tbody>
                <tfoot>
                    <tr><td>Total</td><td>$2,000</td><td>$2,450</td></tr>
                </tfoot>
            </table>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "dom_has_table".to_string(), "has_glyph_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 边框和圆角组合
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-border-radius-asymmetric".to_string(),
            description: "CSS asymmetric border-radius".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width:200px;height:100px;background:#e74c3c;border-radius:20px 5px 20px 5px;"></div>
            <div style="width:200px;height:100px;background:#3498db;border-radius:50px/20px;margin-top:10px;"></div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 伪元素
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/css-pseudo-before-after".to_string(),
            description: "CSS ::before and ::after pseudo-elements".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div class="quoted" style="padding:20px;background:#f0f0f0;">Real content</div>
            </body></html>"#.to_string(),
            css: r#".quoted::before { content: "«"; font-size: 24px; color: #e74c3c; }
            .quoted::after { content: "»"; font-size: 24px; color: #e74c3c; }"#.to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  HTML 多媒体占位
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "web-platform/html-media-placeholder".to_string(),
            description: "HTML video and audio placeholder elements".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
            <video width="320" height="240" controls>
                <source src="video.mp4" type="video/mp4">
                <p>Your browser does not support video.</p>
            </video>
            <audio controls>
                <source src="audio.mp3" type="audio/mpeg">
                <p>Your browser does not support audio.</p>
            </audio>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
    ]
}
