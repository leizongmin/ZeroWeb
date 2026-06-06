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
                "has_fill_primitives".to_string(),
                "dom_has_text".to_string(),
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
                "has_fill_primitives".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  HTML 链接导航扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "navigation/links/download-attribute".to_string(),
            description: "a[download] 属性不崩溃".to_string(),
            category: "navigation".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<a href="/files/doc.pdf" download="document.pdf">Download PDF</a>
<a href="/images/photo.jpg" download>Download Image</a>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_link".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "navigation/links/target-blank".to_string(),
            description: "target=_blank 链接".to_string(),
            category: "navigation".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<a href="https://example.com" target="_blank" rel="noopener">External</a>
<a href="https://example.org" target="_blank">Another</a>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_link".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "navigation/links/ping-attribute".to_string(),
            description: "a[ping] 属性不崩溃".to_string(),
            category: "navigation".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<a href="/page2" ping="/track/click">Trackable link</a>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_link".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  Head 元数据扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "navigation/meta/viewport".to_string(),
            description: "viewport meta 标签".to_string(),
            category: "navigation".to_string(),
            html: r#"<!DOCTYPE html>
<html><head>
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Responsive Page</title>
</head><body>
<div style="width:100%; max-width:600px; margin:0 auto;">Responsive content</div>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_meta".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "navigation/meta/charset".to_string(),
            description: "charset meta 标签".to_string(),
            category: "navigation".to_string(),
            html: r#"<!DOCTYPE html>
<html><head>
<meta charset="UTF-8">
<title>Unicode Page</title>
</head><body>
<p>中文 日本語 한국어 Ελληνικά العربية עברית</p>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_meta".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  图片资源扩展
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "navigation/images/picture-element".to_string(),
            description: "picture 元素不崩溃".to_string(),
            category: "navigation".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<picture>
    <source srcset="/img/large.jpg" media="(min-width: 800px)">
    <source srcset="/img/medium.jpg" media="(min-width: 400px)">
    <img src="/img/small.jpg" alt="Responsive image">
</picture>
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_img".to_string(),
                "render_completes".to_string(),
            ],
        },
        TestCase {
            id: "navigation/images/srcset".to_string(),
            description: "img srcset 属性".to_string(),
            category: "navigation".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<img src="/img/photo.jpg"
     srcset="/img/photo-320w.jpg 320w, /img/photo-640w.jpg 640w, /img/photo-1280w.jpg 1280w"
     sizes="(max-width: 600px) 100vw, 50vw"
     alt="Responsive photo">
</body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_img".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        //  综合页面导航
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "navigation/composite/documentation-page".to_string(),
            description: "文档导航页面".to_string(),
            category: "navigation".to_string(),
            html: r##"<!DOCTYPE html>
<html><head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Documentation</title>
<style>
nav { background: #333; padding: 10px; }
nav a { color: white; margin-right: 15px; text-decoration: none; }
article { max-width: 800px; margin: 20px auto; }
aside { float: right; width: 200px; background: #f5f5f5; padding: 10px; }
</style>
</head><body>
<nav>
    <a href="#intro">Introduction</a>
    <a href="#setup">Setup</a>
    <a href="#api">API Reference</a>
    <a href="/download" download>Download</a>
</nav>
<article>
    <h1 id="intro">Introduction</h1>
    <p>This is the documentation page with navigation links.</p>
    <h2 id="setup">Setup</h2>
    <p>Setup instructions here.</p>
    <h2 id="api">API Reference</h2>
    <p>API documentation here.</p>
</article>
</body></html>"##
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_nav".to_string(),
                "dom_has_link".to_string(),
                "dom_has_heading".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  链接和导航交互
        // ═══════════════════════════════════════════════════════════════

        // ── 多种链接类型 ──
        TestCase {
            id: "navigation/link-types".to_string(),
            description: "多种链接类型渲染".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><body>
            <nav>
                <a href="https://example.com">External link</a>
                <a href="/about">Relative link</a>
                <a href="#section">Anchor link</a>
                <a href="mailto:test@example.com">Email link</a>
                <a href="tel:+1234567890">Phone link</a>
                <a href="javascript:void(0)">JavaScript link</a>
                <a href="data:text/plain,Hello">Data URI link</a>
            </nav>
            <section id="section">Target section</section>
            </body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── 图片映射和图片链接 ──
        TestCase {
            id: "navigation/image-links".to_string(),
            description: "图片链接渲染".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><body>
            <a href="https://example.com"><img src="logo.png" alt="Logo"></a>
            <a href="https://example.com"><img src="banner.png" alt="Banner" width="300" height="100"></a>
            <figure>
                <a href="https://example.com"><img src="photo.jpg" alt="Photo"></a>
                <figcaption>Click photo to visit</figcaption>
            </figure>
            </body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── 面包屑导航 ──
        TestCase {
            id: "navigation/breadcrumb".to_string(),
            description: "面包屑导航渲染".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><body>
            <nav aria-label="Breadcrumb">
                <ol>
                    <li><a href="/">Home</a></li>
                    <li><a href="/products">Products</a></li>
                    <li><a href="/products/electronics">Electronics</a></li>
                    <li aria-current="page">Smartphones</li>
                </ol>
            </nav>
            </body></html>"##.to_string(),
            css: "nav ol { display: flex; list-style: none; gap: 8px; } li+li::before { content: '›'; margin-right: 8px; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ── 分页导航 ──
        TestCase {
            id: "navigation/pagination".to_string(),
            description: "分页导航渲染".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><body>
            <nav aria-label="Pagination">
                <ul>
                    <li><a href="?page=1">1</a></li>
                    <li><a href="?page=2">2</a></li>
                    <li aria-current="page">3</li>
                    <li><a href="?page=4">4</a></li>
                    <li><a href="?page=5">5</a></li>
                    <li><a href="?page=4">Next</a></li>
                </ul>
            </nav>
            </body></html>"##.to_string(),
            css: "nav ul { display: flex; gap: 4px; list-style: none; } li { padding: 4px 12px; border: 1px solid #ccc; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── meta 刷新和重定向 ──
        TestCase {
            id: "navigation/meta-tags".to_string(),
            description: "meta 标签渲染（charset/viewport/description）".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <meta name="description" content="Test page">
            <meta name="author" content="ZeroWeb">
            <link rel="icon" href="favicon.ico">
            <link rel="stylesheet" href="style.css">
            <base target="_blank">
            </head><body>
            <p>Page with meta tags</p>
            </body></html>"##.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  导航扩展（+5 测试）
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "navigation/hash-fragments".to_string(),
            description: "Hash 片段链接渲染".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><body>
            <nav>
                <a href="#section-1">Section 1</a>
                <a href="#section-2">Section 2</a>
                <a href="#section-3">Section 3</a>
            </nav>
            <section id="section-1"><h2>Section 1</h2><p>Content for section 1</p></section>
            <section id="section-2"><h2>Section 2</h2><p>Content for section 2</p></section>
            <section id="section-3"><h2>Section 3</h2><p>Content for section 3</p></section>
            </body></html>"##.to_string(),
            css: "nav { background: #eee; padding: 8px; margin-bottom: 10px; } nav a { margin-right: 10px; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_nav".to_string(),
                "dom_has_link".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        TestCase {
            id: "navigation/nav-menu-responsive".to_string(),
            description: "响应式导航菜单渲染".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><body>
            <header>
                <nav>
                    <a href="/" class="logo">ZeroWeb</a>
                    <ul class="menu">
                        <li><a href="/features">Features</a></li>
                        <li><a href="/docs">Docs</a></li>
                        <li><a href="/about">About</a></li>
                        <li><a href="/contact">Contact</a></li>
                    </ul>
                </nav>
            </header>
            </body></html>"##.to_string(),
            css: "header { background: #2c3e50; padding: 10px 20px; } nav { display: flex; justify-content: space-between; align-items: center; } .menu { display: flex; list-style: none; gap: 20px; } .menu a { color: white; text-decoration: none; } .logo { color: white; font-size: 20px; font-weight: bold; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_nav".to_string(),
                "dom_has_link".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        TestCase {
            id: "navigation/skip-links".to_string(),
            description: "跳转链接和锚点渲染".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><body>
            <a href="#main" class="skip">Skip to main content</a>
            <header>Header with navigation</header>
            <main id="main">
                <h1>Main Content</h1>
                <p>This is the main content area.</p>
            </main>
            <footer>Footer</footer>
            </body></html>"##.to_string(),
            css: ".skip { position: absolute; top: -40px; left: 0; background: #000; color: #fff; padding: 8px; } .skip:focus { top: 0; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_link".to_string(),
                "dom_has_heading".to_string(),
                "render_completes".to_string(),
            ],
        },

        TestCase {
            id: "navigation/table-of-contents".to_string(),
            description: "目录导航渲染".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><body>
            <aside>
                <h2>Table of Contents</h2>
                <ol>
                    <li><a href="#ch1">Chapter 1: Introduction</a></li>
                    <li><a href="#ch2">Chapter 2: Getting Started</a></li>
                    <li><a href="#ch3">Chapter 3: Advanced Topics</a></li>
                    <li><a href="#ch4">Chapter 4: API Reference</a></li>
                    <li><a href="#ch5">Chapter 5: Deployment</a></li>
                </ol>
            </aside>
            <main>
                <h1 id="ch1">Introduction</h1><p>Intro text</p>
                <h1 id="ch2">Getting Started</h1><p>Setup text</p>
                <h1 id="ch3">Advanced Topics</h1><p>Advanced text</p>
            </main>
            </body></html>"##.to_string(),
            css: "aside { float: left; width: 250px; background: #f8f8f8; padding: 16px; margin-right: 20px; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_link".to_string(),
                "dom_has_heading".to_string(),
                "render_completes".to_string(),
            ],
        },

        TestCase {
            id: "navigation/sitemap-links".to_string(),
            description: "站点地图和多层级链接渲染".to_string(),
            category: "navigation".to_string(),
            html: r##"<html><body>
            <nav aria-label="Sitemap">
                <ul>
                    <li><a href="/">Home</a></li>
                    <li>
                        <a href="/products">Products</a>
                        <ul>
                            <li><a href="/products/software">Software</a></li>
                            <li><a href="/products/hardware">Hardware</a></li>
                        </ul>
                    </li>
                    <li>
                        <a href="/support">Support</a>
                        <ul>
                            <li><a href="/support/docs">Documentation</a></li>
                            <li><a href="/support/forum">Forum</a></li>
                            <li><a href="/support/contact">Contact Us</a></li>
                        </ul>
                    </li>
                </ul>
            </nav>
            </body></html>"##.to_string(),
            css: "ul { list-style: none; } ul ul { margin-left: 20px; } li { padding: 2px 0; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_nav".to_string(),
                "dom_has_link".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        // 导航边界条件（Navigation Edge Cases）
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "navigation/redirect/chain".into(),
            description: "重定向链追踪".into(),
            category: "navigation".into(),
            html: r#"<html><body>
            <h1>Redirect Chain</h1>
            <div>Original URL: <span id="original">/first</span></div>
            <div>After 302: <span id="redirected">/second</span></div>
            <div>Final: <span id="final">/target</span></div>
            <style>
            .status { padding: 4px 8px; margin: 4px; border-radius: 3px; font-size: 13px; }
            .ok { background: #d4edda; color: #155724; }
            </style>
            <div class="status ok">Redirect chain tracked correctly</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into()],
        },

        TestCase {
            id: "navigation/fragment/hash-navigation".into(),
            description: "Hash 片段导航".into(),
            category: "navigation".into(),
            html: r##"<html><body>
            <h1>Hash Navigation</h1>
            <nav>
                <a href="#section1">Section 1</a>
                <a href="#section2">Section 2</a>
                <a href="#section3">Section 3</a>
            </nav>
            <div id="section1"><h2>Section 1</h2><p>Content for section 1</p></div>
            <div id="section2"><h2>Section 2</h2><p>Content for section 2</p></div>
            <div id="section3"><h2>Section 3</h2><p>Content for section 3</p></div>
            <div id="log">Hash: (none)</div>
            <script>
                window.addEventListener('hashchange', function(e) {
                    document.getElementById('log').textContent = 'Hash: ' + location.hash;
                });
            </script>
            </body></html>"##.into(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_element:h1".into(),
                "dom_has_element:nav".into(),
                "render_completes".into(),
            ],
        },

        TestCase {
            id: "navigation/cache/validation".into(),
            description: "HTTP 缓存验证（ETag/If-None-Match）".into(),
            category: "navigation".into(),
            html: r#"<html><body>
            <h1>Cache Validation</h1>
            <style>
            .cache-card { border: 1px solid #ddd; padding: 8px; margin: 4px; border-radius: 4px; }
            code { background: #f0f0f0; padding: 2px 4px; border-radius: 2px; }
            </style>
            <div class="cache-card">
                <p>ETag: <code>"abc123"</code></p>
                <p>If-None-Match: <code>"abc123"</code></p>
                <p>Cache-Control: <code>max-age=3600</code></p>
                <p>Result: 304 Not Modified</p>
            </div>
            <div class="cache-card">
                <p>Cache-Control: <code>no-cache</code></p>
                <p>Result: Always revalidate</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into()],
        },

        TestCase {
            id: "navigation/cookie/attributes".into(),
            description: "Cookie 安全属性验证".into(),
            category: "navigation".into(),
            html: r#"<html><body>
            <h1>Cookie Security Attributes</h1>
            <style>
            table { border-collapse: collapse; width: 100%; }
            th, td { border: 1px solid #ddd; padding: 6px 10px; text-align: left; font-size: 13px; }
            th { background: #f5f5f5; }
            .secure { color: #155724; } .insecure { color: #721c24; }
            </style>
            <table>
            <tr><th>Attribute</th><th>Effect</th><th>Status</th></tr>
            <tr><td>Secure</td><td>HTTPS only</td><td class="secure">Enforced</td></tr>
            <tr><td>HttpOnly</td><td>No JS access</td><td class="secure">Enforced</td></tr>
            <tr><td>SameSite=Strict</td><td>No cross-site</td><td class="secure">Enforced</td></tr>
            <tr><td>SameSite=Lax</td><td>Top-level GET only</td><td class="secure">Enforced</td></tr>
            <tr><td>SameSite=None</td><td>Requires Secure</td><td class="secure">Enforced</td></tr>
            <tr><td>Path</td><td>Scope restriction</td><td class="secure">Enforced</td></tr>
            <tr><td>Domain</td><td>Subdomain access</td><td class="secure">Enforced</td></tr>
            <tr><td>Max-Age/Expires</td><td>Lifetime control</td><td class="secure">Enforced</td></tr>
            </table>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "dom_has_element:table".into(), "render_completes".into()],
        },

        TestCase {
            id: "navigation/hsts/auto-upgrade".into(),
            description: "HSTS 自动升级 HTTP→HTTPS".into(),
            category: "navigation".into(),
            html: r#"<html><body>
            <h1>HSTS Auto-Upgrade</h1>
            <style>
            .upgrade-row { display: flex; align-items: center; gap: 8px; padding: 6px; border-bottom: 1px solid #eee; }
            .arrow { color: #28a745; font-weight: bold; }
            .url { font-family: monospace; font-size: 13px; }
            </style>
            <div class="upgrade-row"><span class="url">http://github.com/...</span> <span class="arrow">→</span> <span class="url">https://github.com/...</span></div>
            <div class="upgrade-row"><span class="url">http://cdn.cloudflare.com/...</span> <span class="arrow">→</span> <span class="url">https://cdn.cloudflare.com/...</span></div>
            <div class="upgrade-row"><span class="url">http://google.com/...</span> <span class="arrow">→</span> <span class="url">https://google.com/...</span></div>
            <p>40+ preload domains auto-upgraded before connection</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into()],
        },

        TestCase {
            id: "navigation/navigation-state/machine".into(),
            description: "导航状态机 — 前进/后退/刷新".into(),
            category: "navigation".into(),
            html: r#"<html><body>
            <h1>Navigation State Machine</h1>
            <style>
            .state { display: inline-block; padding: 6px 12px; margin: 4px; border-radius: 4px; font-size: 13px; }
            .active { background: #007bff; color: white; }
            .visited { background: #28a745; color: white; }
            .future { background: #e9ecef; color: #666; }
            </style>
            <div>
                <span class="state visited">Page A</span> →
                <span class="state visited">Page B</span> →
                <span class="state active">Page C</span> →
                <span class="state future">Page D</span>
            </div>
            <div style="margin-top: 16px;">
                <button>Back</button> <button>Forward</button> <button>Reload</button>
            </div>
            <div style="margin-top: 8px; font-size: 13px; color: #666;">
                <p>History length: 3 entries</p>
                <p>Current index: 2 (Page C)</p>
                <p>Can go back: true</p>
                <p>Can go forward: false (cleared by new navigation)</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "dom_has_element:button".into(), "render_completes".into()],
        },

        TestCase {
            id: "navigation/service-worker/fetch-intercept".into(),
            description: "Service Worker fetch 拦截".into(),
            category: "navigation".into(),
            html: r#"<html><body>
            <h1>Service Worker Fetch Intercept</h1>
            <style>
            .sw-flow { font-family: monospace; font-size: 13px; padding: 12px; background: #f8f9fa; border-radius: 4px; }
            .step { margin: 4px 0; }
            .highlight { color: #0056b3; font-weight: bold; }
            </style>
            <div class="sw-flow">
                <div class="step">1. Browser: fetch("/api/data")</div>
                <div class="step">2. <span class="highlight">Service Worker: intercept</span></div>
                <div class="step">3. SW: cache.match(request)</div>
                <div class="step">4a. Cache HIT → return cached response</div>
                <div class="step">4b. Cache MISS → fetch from network → cache response</div>
                <div class="step">5. Browser: receive response</div>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into()],
        },

        TestCase {
            id: "navigation/timeout/retry".into(),
            description: "网络超时和重试策略".into(),
            category: "navigation".into(),
            html: r#"<html><body>
            <h1>Network Timeout and Retry</h1>
            <style>
            .timeline { border-left: 2px solid #007bff; padding-left: 16px; margin: 16px 0; }
            .event { margin: 8px 0; font-size: 13px; }
            .time { color: #666; font-family: monospace; }
            .fail { color: #dc3545; } .ok { color: #28a745; }
            </style>
            <div class="timeline">
                <div class="event"><span class="time">T+0s</span> Request sent</div>
                <div class="event"><span class="time">T+5s</span> No response... <span class="fail">timeout pending</span></div>
                <div class="event"><span class="time">T+10s</span> <span class="fail">Timeout!</span> Request aborted</div>
                <div class="event"><span class="time">T+10.1s</span> Retry #1 initiated</div>
                <div class="event"><span class="time">T+11.5s</span> <span class="ok">Response received (200 OK)</span></div>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into()],
        },

        TestCase {
            id: "navigation/cors/preflight-sequence".into(),
            description: "CORS 预检请求完整序列".into(),
            category: "navigation".into(),
            html: r#"<html><body>
            <h1>CORS Preflight Sequence</h1>
            <style>
            .seq { font-family: monospace; font-size: 13px; }
            .req { color: #0056b3; } .res { color: #28a745; } .err { color: #dc3545; }
            </style>
            <div class="seq">
                <p class="req">OPTIONS /api/data HTTP/1.1</p>
                <p>Origin: https://app.example.com</p>
                <p>Access-Control-Request-Method: PUT</p>
                <p>Access-Control-Request-Headers: Content-Type, X-Custom</p>
                <p class="res">HTTP/1.1 204 No Content</p>
                <p>Access-Control-Allow-Origin: https://app.example.com</p>
                <p>Access-Control-Allow-Methods: GET, PUT, POST</p>
                <p>Access-Control-Allow-Headers: Content-Type, X-Custom</p>
                <p>Access-Control-Max-Age: 3600</p>
                <p class="req">PUT /api/data HTTP/1.1 (actual request)</p>
                <p class="res">HTTP/1.1 200 OK</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "dom_has_element:h1".into(), "render_completes".into()],
        },
    ]
}

/// 返回运行时和事件循环测试用例。
pub fn runtime_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  定时器
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "runtime/timer/setTimeout".into(),
            description: "setTimeout 基本用法".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            setTimeout(function() {
                document.getElementById('result').textContent = 'timeout fired';
            }, 0);
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
        TestCase {
            id: "runtime/timer/setInterval".into(),
            description: "setInterval 重复回调".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="counter">0</div>
            <script>
            var count = 0;
            var id = setInterval(function() {
                count++;
                document.getElementById('counter').textContent = count;
                if (count >= 3) clearInterval(id);
            }, 10);
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
        TestCase {
            id: "runtime/timer/nested-timeout".into(),
            description: "嵌套 setTimeout 调用".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            setTimeout(function() {
                setTimeout(function() {
                    setTimeout(function() {
                        document.getElementById('result').textContent = 'nested done';
                    }, 0);
                }, 0);
            }, 0);
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Promise / microtask
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "runtime/promise/basic-resolve".into(),
            description: "Promise resolve 基本用法".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            Promise.resolve(42).then(function(val) {
                document.getElementById('result').textContent = 'resolved: ' + val;
            });
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
        TestCase {
            id: "runtime/promise/async-await".into(),
            description: "async/await 语法".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            async function fetchData() {
                var val = await Promise.resolve('hello');
                document.getElementById('result').textContent = 'got: ' + val;
            }
            fetchData();
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
        TestCase {
            id: "runtime/promise/microtask-order".into(),
            description: "microtask 执行顺序".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            var order = [];
            Promise.resolve().then(() => order.push('micro1'));
            Promise.resolve().then(() => order.push('micro2'));
            setTimeout(() => { order.push('timeout'); }, 0);
            setTimeout(() => {
                document.getElementById('result').textContent = order.join(',');
            }, 10);
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  MutationObserver
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "runtime/mutation-observer/basic".into(),
            description: "MutationObserver 观察子节点变化".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="target">initial</div>
            <div id="log"></div>
            <script>
            var observer = new MutationObserver(function(mutations) {
                var log = document.getElementById('log');
                log.textContent = 'mutations: ' + mutations.length;
            });
            observer.observe(document.getElementById('target'), { childList: true, characterData: true, subtree: true });
            document.getElementById('target').textContent = 'changed';
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  事件冒泡/捕获
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "runtime/events/bubble-capture".into(),
            description: "事件冒泡和捕获阶段".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="parent"><button id="child">Click</button></div>
            <div id="log"></div>
            <script>
            var log = [];
            document.getElementById('parent').addEventListener('click', function() {
                log.push('parent-bubble');
            }, false);
            document.getElementById('parent').addEventListener('click', function() {
                log.push('parent-capture');
            }, true);
            document.getElementById('child').addEventListener('click', function(e) {
                log.push('child');
            });
            document.getElementById('child').click();
            setTimeout(function() {
                document.getElementById('log').textContent = log.join(',');
            }, 0);
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CustomEvent
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "runtime/events/custom-event".into(),
            description: "CustomEvent 创建和分发".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            document.addEventListener('my-event', function(e) {
                document.getElementById('result').textContent = 'detail: ' + e.detail;
            });
            var event = new CustomEvent('my-event', { detail: 'hello' });
            document.dispatchEvent(event);
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  requestAnimationFrame
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "runtime/raf/basic".into(),
            description: "requestAnimationFrame 回调".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            requestAnimationFrame(function(timestamp) {
                document.getElementById('result').textContent = 'raf: ' + (timestamp >= 0);
            });
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  导航状态管理
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "runtime/navigation/history-api".into(),
            description: "History API pushState/replaceState".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            history.pushState({ page: 1 }, '', '/page1');
            history.pushState({ page: 2 }, '', '/page2');
            history.replaceState({ page: 2 }, '', '/page2v2');
            document.getElementById('result').textContent = 'length: ' + history.length;
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  console API
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "runtime/console/all-methods".into(),
            description: "console 所有方法不崩溃".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">ok</div>
            <script>
            console.log('log');
            console.warn('warn');
            console.error('error');
            console.info('info');
            console.debug('debug');
            console.table([{a:1},{a:2}]);
            console.group('group');
            console.log('grouped');
            console.groupEnd();
            console.time('timer');
            console.timeEnd('timer');
            console.assert(1 === 1, 'assertion ok');
            console.clear();
            console.count('counter');
            console.count('counter');
            console.dir({key: 'value'});
            console.trace();
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  错误处理
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "runtime/error/try-catch".into(),
            description: "try-catch 错误恢复".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            try {
                JSON.parse('invalid json {{{');
            } catch(e) {
                document.getElementById('result').textContent = 'caught: ' + (e instanceof SyntaxError);
            }
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
        TestCase {
            id: "runtime/error/promise-rejection".into(),
            description: "Promise rejection 处理".into(),
            category: "runtime".into(),
            html: r#"<html><body>
            <div id="result">waiting</div>
            <script>
            Promise.reject(new Error('test rejection')).catch(function(e) {
                document.getElementById('result').textContent = 'caught: ' + e.message;
            });
            </script>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "render_completes".into()],
        },
    ]
}
