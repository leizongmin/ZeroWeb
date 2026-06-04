//! DOM Level 2+ API 标准合规性测试。
//!
//! 覆盖 DOM 核心操作、属性操作、文档方法、事件相关 DOM 结构、
//! 表单元素验证、语义化 HTML 元素。

use super::TestCase;

/// 返回 DOM API 标准合规性测试用例。
pub fn dom_api_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  DOM CORE — 文档结构
        // ═══════════════════════════════════════════════════════════════

        // ── 完整 HTML5 文档结构 ──
        TestCase {
            id: "dom/html5-full-structure".to_string(),
            description: "Complete HTML5 document structure".to_string(),
            category: "dom".to_string(),
            html: r#"<!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Test Page</title>
                    <link rel="stylesheet" href="style.css">
                </head>
                <body>
                    <header>Header</header>
                    <main>Main Content</main>
                    <footer>Footer</footer>
                </body>
                </html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_head".to_string(),
                "dom_has_title".to_string(),
                "dom_has_meta".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── HTML 注释 ──
        TestCase {
            id: "dom/html-comments".to_string(),
            description: "HTML comments preserved in DOM".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <!-- This is a comment -->
                <p>Text after comment</p>
                <!-- Another comment -->
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── 嵌套 div 结构 ──
        TestCase {
            id: "dom/nested-divs".to_string(),
            description: "Deeply nested div structure".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <div id="level1">
                    <div id="level2">
                        <div id="level3">
                            <div id="level4">
                                <span>Deep content</span>
                            </div>
                        </div>
                    </div>
                </div>
            </body></html>"#
                .to_string(),
            css: "div { padding: 10px; background-color: #f0f0f0; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "layout_has_deep_children".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM — 属性操作
        // ═══════════════════════════════════════════════════════════════

        // ── class 属性 ──
        TestCase {
            id: "dom/class-attribute".to_string(),
            description: "Multiple class attributes".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <div class="container main active">Multi-class element</div>
                <p class="text muted">Muted text</p>
            </body></html>"#
                .to_string(),
            css: ".container { width: 300px; } .main { background-color: white; } .active { color: green; } .muted { color: gray; }"
                .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── data-* 自定义属性 ──
        TestCase {
            id: "dom/data-attributes".to_string(),
            description: "Custom data-* attributes".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <div data-id="123" data-name="test" data-active="true">Data attr element</div>
            </body></html>"#
                .to_string(),
            css: "div { width: 200px; height: 100px; background-color: teal; color: white; }"
                .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── id 属性唯一性 ──
        TestCase {
            id: "dom/id-attribute".to_string(),
            description: "Unique id attributes".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <div id="header">Header</div>
                <div id="content">Content</div>
                <div id="footer">Footer</div>
            </body></html>"#
                .to_string(),
            css: "#header { background-color: navy; color: white; } #content { background-color: white; } #footer { background-color: #333; color: white; } div { padding: 10px; }"
                .to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM — 表单元素
        // ═══════════════════════════════════════════════════════════════

        // ── 完整表单 ──
        TestCase {
            id: "dom/form-complete".to_string(),
            description: "Complete form with multiple input types".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <form action="/submit" method="post">
                    <label for="name">Name:</label>
                    <input type="text" id="name" name="name" placeholder="Enter name">
                    <label for="email">Email:</label>
                    <input type="email" id="email" name="email">
                    <label for="pass">Password:</label>
                    <input type="password" id="pass" name="password">
                    <button type="submit">Submit</button>
                    <button type="reset">Reset</button>
                </form>
            </body></html>"#
                .to_string(),
            css: "form { padding: 20px; } label { display: block; margin-top: 10px; } input { width: 200px; padding: 5px; }"
                .to_string(),
            assertions: vec![
                "dom_has_form".to_string(),
                "dom_has_input".to_string(),
                "dom_has_button".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── 复杂表单控件 ──
        TestCase {
            id: "dom/form-advanced-controls".to_string(),
            description: "Advanced form controls (checkbox, radio, textarea, select)".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <form>
                    <input type="checkbox" id="cb1" checked>
                    <label for="cb1">Option A</label>
                    <input type="radio" name="choice" id="r1" value="a">
                    <label for="r1">Choice A</label>
                    <input type="radio" name="choice" id="r2" value="b">
                    <label for="r2">Choice B</label>
                    <textarea id="msg" rows="4" cols="30">Default text</textarea>
                    <select id="sel">
                        <option value="1">Option 1</option>
                        <option value="2" selected>Option 2</option>
                        <option value="3">Option 3</option>
                    </select>
                </form>
            </body></html>"#
                .to_string(),
            css: "form { padding: 10px; }".to_string(),
            assertions: vec![
                "dom_has_form".to_string(),
                "dom_has_select".to_string(),
                "dom_has_input".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── HTML5 input 类型 ──
        TestCase {
            id: "dom/html5-input-types".to_string(),
            description: "HTML5 input types".to_string(),
            category: "dom".to_string(),
            html: r##"<html><body>
                <form>
                    <input type="number" min="0" max="100" value="50">
                    <input type="range" min="0" max="100" value="75">
                    <input type="date">
                    <input type="time">
                    <input type="color" value="#ff0000">
                    <input type="search" placeholder="Search...">
                    <input type="url" placeholder="https://example.com">
                    <input type="tel" placeholder="+1-234-567-8900">
                </form>
            </body></html>"##
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_form".to_string(),
                "dom_has_input".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM — 语义化 HTML
        // ═══════════════════════════════════════════════════════════════

        // ── 语义化页面结构 ──
        TestCase {
            id: "dom/semantic-structure".to_string(),
            description: "Semantic HTML5 page structure".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <nav>
                    <a href="/">Home</a>
                    <a href="/about">About</a>
                    <a href="/contact">Contact</a>
                </nav>
                <article>
                    <h1>Article Title</h1>
                    <p>Article content paragraph.</p>
                    <section>
                        <h2>Section Title</h2>
                        <p>Section content.</p>
                    </section>
                </article>
                <aside>
                    <p>Sidebar content</p>
                </aside>
            </body></html>"#
                .to_string(),
            css: "nav { background-color: #333; color: white; padding: 10px; } article { padding: 20px; } aside { background-color: #f5f5f5; padding: 10px; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "dom_has_link".to_string(),
                "dom_has_heading".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── figure + figcaption ──
        TestCase {
            id: "dom/figure-figcaption".to_string(),
            description: "Figure with figcaption".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <figure>
                    <img src="photo.jpg" alt="A photo">
                    <figcaption>Caption for the photo</figcaption>
                </figure>
            </body></html>"#
                .to_string(),
            css: "figure { border: 1px solid #ccc; padding: 10px; }".to_string(),
            assertions: vec![
                "dom_has_img".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── details/summary ──
        TestCase {
            id: "dom/details-summary".to_string(),
            description: "Details and summary elements".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <details>
                    <summary>Click to expand</summary>
                    <p>Hidden content revealed on click</p>
                </details>
                <details open>
                    <summary>Already open</summary>
                    <p>This content is visible by default</p>
                </details>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM — 列表和表格
        // ═══════════════════════════════════════════════════════════════

        // ── 有序列表 ──
        TestCase {
            id: "dom/ordered-list".to_string(),
            description: "Ordered list with multiple items".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <ol>
                    <li>First item</li>
                    <li>Second item</li>
                    <li>Third item</li>
                </ol>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_list".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── 定义列表 ──
        TestCase {
            id: "dom/definition-list".to_string(),
            description: "Definition list (dl/dt/dd)".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <dl>
                    <dt>Term 1</dt>
                    <dd>Definition 1</dd>
                    <dt>Term 2</dt>
                    <dd>Definition 2</dd>
                </dl>
            </body></html>"#
                .to_string(),
            css: "dt { font-weight: bold; } dd { margin-left: 20px; }".to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── 复杂表格 ──
        TestCase {
            id: "dom/complex-table".to_string(),
            description: "Complex table with thead, tbody, tfoot".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <table>
                    <thead>
                        <tr><th>Name</th><th>Age</th><th>City</th></tr>
                    </thead>
                    <tbody>
                        <tr><td>Alice</td><td>30</td><td>NYC</td></tr>
                        <tr><td>Bob</td><td>25</td><td>LA</td></tr>
                    </tbody>
                    <tfoot>
                        <tr><td colspan="3">2 records</td></tr>
                    </tfoot>
                </table>
            </body></html>"#
                .to_string(),
            css: "table { border-collapse: collapse; width: 100%; } th, td { border: 1px solid black; padding: 8px; text-align: left; }"
                .to_string(),
            assertions: vec![
                "dom_has_table".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM — 文本元素
        // ═══════════════════════════════════════════════════════════════

        // ── 内联文本元素 ──
        TestCase {
            id: "dom/inline-text-elements".to_string(),
            description: "Various inline text elements".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <p>This is <strong>bold</strong> and <em>italic</em> text.</p>
                <p>This is <mark>highlighted</mark> text.</p>
                <p>This is <del>deleted</del> and <ins>inserted</ins> text.</p>
                <p>This is <code>code</code> and <small>small</small> text.</p>
                <p>H<sub>2</sub>O and E=mc<sup>2</sup></p>
                <p><abbr title="HyperText Markup Language">HTML</abbr> is a language.</p>
            </body></html>"#
                .to_string(),
            css: "mark { background-color: yellow; } code { background-color: #f0f0f0; }".to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── 引用和块引用 ──
        TestCase {
            id: "dom/blockquote-q".to_string(),
            description: "Blockquote and inline quote".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <blockquote cite="https://example.com">
                    <p>To be or not to be.</p>
                </blockquote>
                <p>He said <q>Hello world</q>.</p>
            </body></html>"#
                .to_string(),
            css: "blockquote { border-left: 3px solid gray; padding-left: 15px; margin: 15px 0; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── pre/code/kbd ──
        TestCase {
            id: "dom/pre-code-kbd".to_string(),
            description: "Preformatted, code and keyboard input".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <pre><code>function hello() {
    console.log("Hello!");
}</code></pre>
                <p>Press <kbd>Ctrl</kbd> + <kbd>C</kbd> to copy.</p>
            </body></html>"#
                .to_string(),
            css: "pre { background-color: #f5f5f5; padding: 10px; } code { font-family: monospace; } kbd { background-color: #eee; border: 1px solid #ccc; padding: 2px 4px; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM — 链接和导航
        // ═══════════════════════════════════════════════════════════════

        // ── 多种链接 ──
        TestCase {
            id: "dom/link-types".to_string(),
            description: "Various link types".to_string(),
            category: "dom".to_string(),
            html: r##"<html><body>
                <a href="https://example.com">External link</a>
                <a href="/about">Internal link</a>
                <a href="#section">Anchor link</a>
                <a href="mailto:test@example.com">Email link</a>
                <a href="tel:+1234567890">Phone link</a>
            </body></html>"##
                .to_string(),
            css: "a { color: blue; text-decoration: underline; margin-right: 10px; }".to_string(),
            assertions: vec![
                "dom_has_link".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM — 媒体元素
        // ═══════════════════════════════════════════════════════════════

        // ── 图片元素 ──
        TestCase {
            id: "dom/image-element".to_string(),
            description: "Image element with alt text".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <img src="photo.jpg" alt="A scenic photo" width="300" height="200">
                <img src="icon.png" alt="Icon" width="16" height="16">
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_img".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── video/audio 占位 ──
        TestCase {
            id: "dom/media-placeholder".to_string(),
            description: "Video and audio placeholders".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <video width="320" height="240" controls>
                    <source src="movie.mp4" type="video/mp4">
                    Your browser does not support video.
                </video>
                <audio controls>
                    <source src="audio.mp3" type="audio/mpeg">
                    Your browser does not support audio.
                </audio>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM — Unicode 和特殊字符
        // ═══════════════════════════════════════════════════════════════

        // ── Unicode 文本 ──
        TestCase {
            id: "dom/unicode-text".to_string(),
            description: "Unicode text content".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <p>English: Hello World</p>
                <p>Chinese: 你好世界</p>
                <p>Japanese: こんにちは</p>
                <p>Korean: 안녕하세요</p>
                <p>Arabic: مرحبا</p>
                <p>Emoji: 🌍 🚀 ❤️ 🎉</p>
                <p>Math: ∑ ∫ √ ∞ ≠ ≤ ≥</p>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── HTML 实体 ──
        TestCase {
            id: "dom/html-entities".to_string(),
            description: "HTML entity decoding".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <p>&amp; &lt; &gt; &quot; &apos;</p>
                <p>&nbsp;&nbsp;&nbsp;Non-breaking spaces</p>
                <p>&copy; &reg; &trade; &euro; &pound;</p>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM — 错误恢复
        // ═══════════════════════════════════════════════════════════════

        // ── 未闭合标签 ──
        TestCase {
            id: "dom/unclosed-tags".to_string(),
            description: "Unclosed tags error recovery".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <p>Paragraph 1
                <p>Paragraph 2
                <div><span>Inner
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── 无效嵌套 ──
        TestCase {
            id: "dom/invalid-nesting".to_string(),
            description: "Invalid nesting error recovery".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <b><i>Bold italic</b></i>
                <div><div><div>Deep nesting</div></div></div>
            </body></html>"#
                .to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── 空文档 ──
        TestCase {
            id: "dom/empty-document".to_string(),
            description: "Empty document handling".to_string(),
            category: "dom".to_string(),
            html: String::new(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
        // ── 仅文本 ──
        TestCase {
            id: "dom/text-only".to_string(),
            description: "Plain text without HTML tags".to_string(),
            category: "dom".to_string(),
            html: "Just plain text, no HTML tags at all.".to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string()],
        },
    ]
}
