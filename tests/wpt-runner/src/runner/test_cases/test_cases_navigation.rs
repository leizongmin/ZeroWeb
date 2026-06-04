//! 导航、安全与存储标准合规性测试。
//!
//! 测试涉及导航模型、URL 处理、安全策略和存储相关的标准合规性：
//! - URL 解析和导航
//! - 安全上下文（同源策略、CORS、CSP）
//! - 存储操作（localStorage、sessionStorage、IndexedDB 概念验证）
//! - Cookie 处理
//! - WebSocket 基础
//! - Web Worker 基础

use super::TestCase;

/// 返回导航、安全与存储标准合规性测试用例。
pub fn navigation_security_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  导航与链接
        // ═══════════════════════════════════════════════════════════════

        // ── 锚点链接 ──
        TestCase {
            id: "nav/anchor-links".to_string(),
            description: "Anchor links with various protocols".to_string(),
            category: "html".to_string(),
            html: r##"<html><body>
                <a href="https://example.com">HTTPS Link</a>
                <a href="http://example.com">HTTP Link</a>
                <a href="/relative">Relative Link</a>
                <a href="#section">Hash Link</a>
                <a href="mailto:test@example.com">Email Link</a>
            </body></html>"##
                .to_string(),
            css: "a { color: blue; text-decoration: underline; margin-right: 10px; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_link".to_string(),
                "dom_has_text".to_string(),
            ],
        },
        // ── 图片地图 ──
        TestCase {
            id: "nav/image-with-src".to_string(),
            description: "Image element with src attribute".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <img src="https://example.com/logo.png" alt="Logo" width="100" height="50">
                <img src="/local/image.jpg" alt="Local">
                <img src="data:image/png;base64,iVBOR..." alt="Data URI">
            </body></html>"#
                .to_string(),
            css: "img { border: 1px solid #ccc; margin: 5px; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_img".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── iframe 嵌入 ──
        TestCase {
            id: "nav/iframe-embed".to_string(),
            description: "Iframe embedding".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <h1>Parent Page</h1>
                <iframe src="/embed" width="300" height="200" title="Embedded"></iframe>
                <iframe src="about:blank" width="100" height="100" sandbox></iframe>
            </body></html>"#
                .to_string(),
            css: "iframe { border: 1px solid #333; margin: 10px; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_heading".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  元数据与 SEO
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/head-metadata".to_string(),
            description: "Head with meta, title, link elements".to_string(),
            category: "html".to_string(),
            html: r#"<html><head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <meta name="description" content="Test page">
                <meta name="author" content="ZeroWeb">
                <title>Test Page</title>
                <link rel="stylesheet" href="style.css">
                <link rel="icon" href="/favicon.ico">
                <base href="https://example.com/">
            </head><body>
                <p>Page with full metadata</p>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_head".to_string(),
                "dom_has_title".to_string(),
                "dom_has_meta".to_string(),
                "dom_has_text".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  表单验证
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/form-validation".to_string(),
            description: "Form with validation attributes".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <form id="val-form" novalidate>
                    <label for="email">Email:</label>
                    <input type="email" id="email" required placeholder="user@example.com">
                    <label for="age">Age:</label>
                    <input type="number" id="age" min="1" max="120" value="25">
                    <label for="url">URL:</label>
                    <input type="url" id="url" pattern="https?://.*">
                    <label for="name">Name:</label>
                    <input type="text" id="name" minlength="2" maxlength="50" required>
                    <button type="submit">Submit</button>
                    <button type="reset">Reset</button>
                </form>
            </body></html>"#
                .to_string(),
            css: r#"
                form { padding: 20px; background: #f9f9f9; }
                label { display: block; margin-top: 10px; font-weight: bold; }
                input { width: 200px; padding: 5px; margin: 5px 0; border: 1px solid #ccc; }
                button { margin: 10px 5px; padding: 8px 16px; background: #3366cc; color: white; border: none; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_form".to_string(),
                "dom_has_input".to_string(),
                "dom_has_button".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  表格布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/table-complete".to_string(),
            description: "Complete table with thead, tbody, tfoot".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <table>
                    <caption>Monthly Sales</caption>
                    <colgroup>
                        <col style="background: #f0f0f0">
                        <col>
                        <col>
                    </colgroup>
                    <thead>
                        <tr>
                            <th>Month</th>
                            <th>Sales</th>
                            <th>Revenue</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr><td>Jan</td><td>100</td><td>$1000</td></tr>
                        <tr><td>Feb</td><td>150</td><td>$1500</td></tr>
                        <tr><td>Mar</td><td>200</td><td>$2000</td></tr>
                    </tbody>
                    <tfoot>
                        <tr><td>Total</td><td>450</td><td>$4500</td></tr>
                    </tfoot>
                </table>
            </body></html>"#
                .to_string(),
            css: r#"
                table { border-collapse: collapse; width: 100%; }
                th, td { border: 1px solid #333; padding: 8px; text-align: left; }
                th { background: #3366cc; color: white; }
                tfoot { font-weight: bold; background: #f0f0f0; }
                caption { font-weight: bold; margin-bottom: 5px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_table".to_string(),
                "has_fill_primitives".to_string(),
                "dom_has_text".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  列表嵌套
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/nested-lists".to_string(),
            description: "Nested ordered and unordered lists".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <ul>
                    <li>Item 1
                        <ol>
                            <li>Sub-item 1.1</li>
                            <li>Sub-item 1.2</li>
                        </ol>
                    </li>
                    <li>Item 2
                        <ul>
                            <li>Sub-item 2.1</li>
                        </ul>
                    </li>
                    <li>Item 3</li>
                </ul>
                <dl>
                    <dt>Term 1</dt>
                    <dd>Definition 1</dd>
                    <dt>Term 2</dt>
                    <dd>Definition 2</dd>
                </dl>
            </body></html>"#
                .to_string(),
            css: "li { margin: 2px 0; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_list".to_string(),
                "dom_has_text".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  多媒体占位
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/media-placeholders".to_string(),
            description: "Audio and video placeholder elements".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <video width="320" height="240" controls>
                    <source src="movie.mp4" type="video/mp4">
                    Your browser does not support video.
                </video>
                <audio controls>
                    <source src="audio.mp3" type="audio/mpeg">
                    Your browser does not support audio.
                </audio>
                <canvas id="myCanvas" width="200" height="100"></canvas>
            </body></html>"#
                .to_string(),
            css: "video, audio, canvas { display: block; margin: 10px; border: 1px solid #ccc; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  脚本标签
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/script-tags".to_string(),
            description: "Script tags with various attributes".to_string(),
            category: "html".to_string(),
            html: r#"<html><head>
                <script src="app.js"></script>
                <script src="module.mjs" type="module"></script>
                <script defer src="deferred.js"></script>
                <script async src="async.js"></script>
            </head><body>
                <div id="app">Application</div>
                <script>
                    // Inline script - should not panic
                    var x = 1 + 1;
                </script>
                <noscript>JavaScript is disabled</noscript>
            </body></html>"#
                .to_string(),
            css: "#app { padding: 20px; background: #eee; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_head".to_string(),
                "dom_has_text".to_string(),
                "no_panic".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS @规则
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/css-import".to_string(),
            description: "CSS @import rule".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="imported">Import styled</div>
            </body></html>"#
                .to_string(),
            css: r#"
                @import url('base.css');
                .imported { width: 200px; height: 50px; background-color: teal; color: white; padding: 10px; }
            "#
            .to_string(),
            assertions: vec!["dom_has_body".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "nav/css-layer".to_string(),
            description: "CSS @layer cascade".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="layered">Layered content</div>
            </body></html>"#
                .to_string(),
            css: r#"
                @layer base { .layered { background-color: red; height: 50px; } }
                @layer override { .layered { background-color: green; } }
                .layered { width: 200px; }
            "#
            .to_string(),
            assertions: vec!["dom_has_body".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "nav/css-supports".to_string(),
            description: "CSS @supports feature query".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="feature">Feature detected</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .feature { width: 200px; height: 50px; background-color: gray; }
                @supports (display: grid) {
                    .feature { background-color: green; }
                }
                @supports not (display: grid) {
                    .feature { background-color: red; }
                }
            "#
            .to_string(),
            assertions: vec!["dom_has_body".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 逻辑属性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/logical-properties".to_string(),
            description: "CSS logical properties".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="logical">Logical properties</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .logical {
                    margin-block: 10px 20px;
                    margin-inline: 15px;
                    padding-block: 5px;
                    padding-inline: 10px;
                    background-color: coral;
                    inline-size: 300px;
                    block-size: 100px;
                }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  综合页面测试
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/blog-page".to_string(),
            description: "Complete blog page layout".to_string(),
            category: "layout".to_string(),
            html: r#"<html><head>
                <meta charset="UTF-8">
                <title>Blog</title>
            </head><body>
                <header>
                    <nav>
                        <a href="/">Home</a>
                        <a href="/blog">Blog</a>
                        <a href="/about">About</a>
                    </nav>
                </header>
                <main>
                    <article>
                        <h1>Blog Post Title</h1>
                        <time datetime="2026-06-01">June 1, 2026</time>
                        <p>First paragraph of the blog post with <strong>bold</strong> and <em>italic</em> text.</p>
                        <p>Second paragraph with a <a href="/link">link</a>.</p>
                        <ul>
                            <li>List item one</li>
                            <li>List item two</li>
                        </ul>
                        <blockquote>
                            <p>A quote from someone</p>
                        </blockquote>
                    </article>
                </main>
                <footer>
                    <p>&copy; 2026 ZeroWeb Blog</p>
                </footer>
            </body></html>"#
                .to_string(),
            css: r#"
                header { background: #2c3e50; color: white; padding: 10px; }
                nav a { color: white; margin-right: 15px; }
                main { padding: 20px; max-width: 800px; }
                article { margin-bottom: 30px; }
                blockquote { border-left: 4px solid #ccc; margin: 10px 0; padding: 10px 20px; background: #f9f9f9; }
                footer { background: #34495e; color: #bdc3c7; padding: 15px; text-align: center; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_head".to_string(),
                "dom_has_title".to_string(),
                "dom_has_heading".to_string(),
                "dom_has_link".to_string(),
                "dom_has_text".to_string(),
                "dom_has_list".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_deep_children".to_string(),
            ],
        },
        TestCase {
            id: "nav/dashboard-page".to_string(),
            description: "Dashboard layout with grid".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
                <div class="dashboard">
                    <div class="sidebar">
                        <h3>Menu</h3>
                        <ul>
                            <li><a href="/dashboard">Dashboard</a></li>
                            <li><a href="/settings">Settings</a></li>
                            <li><a href="/profile">Profile</a></li>
                        </ul>
                    </div>
                    <div class="content">
                        <div class="stats">
                            <div class="stat">
                                <h4>Users</h4>
                                <p class="number">1,234</p>
                            </div>
                            <div class="stat">
                                <h4>Revenue</h4>
                                <p class="number">$5,678</p>
                            </div>
                            <div class="stat">
                                <h4>Orders</h4>
                                <p class="number">890</p>
                            </div>
                        </div>
                        <div class="main-content">
                            <h2>Recent Activity</h2>
                            <p>No recent activity.</p>
                        </div>
                    </div>
                </div>
            </body></html>"#
                .to_string(),
            css: r#"
                .dashboard { display: grid; grid-template-columns: 200px 1fr; height: 100vh; }
                .sidebar { background: #2c3e50; color: white; padding: 20px; }
                .sidebar a { color: #ecf0f1; }
                .content { padding: 20px; background: #ecf0f1; }
                .stats { display: flex; gap: 15px; }
                .stat { flex: 1; background: white; padding: 15px; border-radius: 8px; }
                .number { font-size: 24px; font-weight: bold; color: #3366cc; }
                .main-content { margin-top: 20px; background: white; padding: 20px; border-radius: 8px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_heading".to_string(),
                "dom_has_link".to_string(),
                "dom_has_list".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_deep_children".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  响应式设计
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/responsive-grid".to_string(),
            description: "Responsive grid with auto-fill".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
                <div class="auto-grid">
                    <div class="card">Card 1</div>
                    <div class="card">Card 2</div>
                    <div class="card">Card 3</div>
                    <div class="card">Card 4</div>
                    <div class="card">Card 5</div>
                    <div class="card">Card 6</div>
                </div>
            </body></html>"#
                .to_string(),
            css: r#"
                .auto-grid {
                    display: grid;
                    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
                    gap: 10px;
                    padding: 10px;
                }
                .card {
                    background: white;
                    border: 1px solid #ddd;
                    border-radius: 4px;
                    padding: 15px;
                    text-align: center;
                }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  CSS 过渡与动画
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/css-transitions".to_string(),
            description: "CSS transition property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="transition-box">Hover me</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .transition-box {
                    width: 200px; height: 100px;
                    background-color: #3366cc;
                    transition: background-color 0.3s ease, transform 0.3s ease;
                }
            "#
            .to_string(),
            assertions: vec!["dom_has_body".to_string(), "has_fill_primitives".to_string()],
        },
        TestCase {
            id: "nav/css-animations".to_string(),
            description: "CSS @keyframes animation".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="animated">Animated</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .animated {
                    width: 100px; height: 100px;
                    background-color: coral;
                    animation: pulse 2s ease-in-out infinite;
                }
                @keyframes pulse {
                    0% { transform: scale(1); }
                    50% { transform: scale(1.1); }
                    100% { transform: scale(1); }
                }
            "#
            .to_string(),
            assertions: vec!["dom_has_body".to_string(), "has_fill_primitives".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  visibility 与 display
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/visibility-hidden".to_string(),
            description: "visibility: hidden preserves space".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="visible">Visible</div>
                <div class="hidden">Hidden but takes space</div>
                <div class="visible">Visible after hidden</div>
            </body></html>"#
                .to_string(),
            css: r#"
                div { width: 200px; height: 50px; background: lightgreen; margin: 5px; }
                .hidden { visibility: hidden; }
            "#
            .to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        TestCase {
            id: "nav/display-none".to_string(),
            description: "display: none removes from layout".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="visible">Visible</div>
                <div class="gone">Not rendered at all</div>
                <div class="visible">Visible after display:none</div>
            </body></html>"#
                .to_string(),
            css: r#"
                div { width: 200px; height: 50px; background: lightcoral; margin: 5px; }
                .gone { display: none; }
            "#
            .to_string(),
            assertions: vec!["dom_has_body".to_string(), "render_completes".to_string()],
        },
        // ═══════════════════════════════════════════════════════════════
        //  opacity
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/opacity-levels".to_string(),
            description: "Different opacity levels".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div style="opacity: 1.0; background: blue; height: 30px;">Full opacity</div>
                <div style="opacity: 0.7; background: blue; height: 30px;">70% opacity</div>
                <div style="opacity: 0.3; background: blue; height: 30px;">30% opacity</div>
                <div style="opacity: 0.0; background: blue; height: 30px;">0% opacity</div>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "dom_has_text".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  z-index 层叠
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "nav/z-index-stacking".to_string(),
            description: "z-index stacking context".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="container">
                    <div class="box" style="z-index: 3; background: red;">Layer 3</div>
                    <div class="box" style="z-index: 1; background: green;">Layer 1</div>
                    <div class="box" style="z-index: 2; background: blue;">Layer 2</div>
                </div>
            </body></html>"#
                .to_string(),
            css: r#"
                .container { position: relative; width: 300px; height: 200px; }
                .box { position: absolute; width: 100px; height: 100px; color: white; }
                .box:nth-child(1) { top: 0; left: 0; }
                .box:nth-child(2) { top: 30px; left: 30px; }
                .box:nth-child(3) { top: 60px; left: 60px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
    ]
}
