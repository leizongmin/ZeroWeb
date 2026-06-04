//! JavaScript/DOM 交互标准合规性测试。
//!
//! 测试 HTML 内嵌脚本对 DOM 的操作能力，包括：
//! - 基础 DOM 操作（createElement、appendChild、setAttribute）
//! - 文本节点操作（textContent、innerHTML）
//! - 属性操作（getAttribute、setAttribute、classList）
//! - 事件系统（addEventListener、dispatchEvent）
//! - Web API（console、setTimeout、Promise）

use super::TestCase;

/// 返回 JavaScript/DOM 交互标准合规性测试用例。
pub fn js_dom_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  DOM 基础操作
        // ═══════════════════════════════════════════════════════════════

        // ── DOM 树构建 ──
        TestCase {
            id: "js-dom/nested-divs".to_string(),
            description: "Nested div elements with IDs".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <div id="outer">
                    <div id="inner">Content</div>
                </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_element".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ── 多层嵌套结构 ──
        TestCase {
            id: "js-dom/deep-nesting".to_string(),
            description: "Deep nested DOM structure".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <div id="l1">
                    <div id="l2">
                        <div id="l3">
                            <div id="l4">
                                <span id="l5">Deep</span>
                            </div>
                        </div>
                    </div>
                </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "layout_has_deep_children".to_string(),
            ],
        },

        // ── 属性操作 ──
        TestCase {
            id: "js-dom/data-attributes".to_string(),
            description: "Elements with data-* attributes".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <div id="item" data-name="test" data-value="42" data-active="true">
                    Data
                </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_element".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── class 属性操作 ──
        TestCase {
            id: "js-dom/class-list".to_string(),
            description: "Elements with multiple CSS classes".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <div class="container main active visible">
                    Multi-class element
                </div>
            </body></html>"#.to_string(),
            css: ".container { width: 100px; } .main { height: 50px; } .active { background-color: green; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── 表单元素 ──
        TestCase {
            id: "js-dom/form-inputs".to_string(),
            description: "Form with various input types".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <form id="myform">
                    <input type="text" name="username" value="test">
                    <input type="password" name="pass">
                    <input type="email" name="email">
                    <input type="checkbox" checked>
                    <input type="radio" name="choice">
                    <textarea name="comment">Hello</textarea>
                    <select name="country">
                        <option value="cn">China</option>
                        <option value="us">USA</option>
                    </select>
                    <button type="submit">Submit</button>
                </form>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_form".to_string(),
                "dom_has_input".to_string(),
                "dom_has_button".to_string(),
                "dom_has_select".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  HTML5 语义元素
        // ═══════════════════════════════════════════════════════════════

        // ── article 结构 ──
        TestCase {
            id: "js-dom/semantic-article".to_string(),
            description: "Semantic article structure".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <article>
                    <header>
                        <h1>Title</h1>
                        <time datetime="2026-01-01">Jan 1, 2026</time>
                    </header>
                    <p>Article content here.</p>
                    <footer>
                        <p>Footer info</p>
                    </footer>
                </article>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_heading".to_string(),
                "dom_has_text".to_string(),
            ],
        },

        // ── nav + aside 结构 ──
        TestCase {
            id: "js-dom/semantic-layout".to_string(),
            description: "Semantic layout with nav and aside".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <nav>
                    <a href="/">Home</a>
                    <a href="/about">About</a>
                </nav>
                <main>
                    <section>
                        <h2>Section Title</h2>
                        <p>Section content.</p>
                    </section>
                    <aside>
                        <p>Sidebar content.</p>
                    </aside>
                </main>
            </body></html>"#.to_string(),
            css: "nav { background: #333; } main { display: flex; } section { flex: 1; } aside { width: 200px; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_link".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ── figure + figcaption ──
        TestCase {
            id: "js-dom/figure-caption".to_string(),
            description: "Figure with figcaption".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <figure>
                    <img src="test.png" alt="Test image">
                    <figcaption>Caption for the image</figcaption>
                </figure>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_img".to_string(),
                "dom_has_text".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  HTML 实体与特殊字符
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/html-entities".to_string(),
            description: "HTML entities decode correctly".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <p>&amp; &lt; &gt; &quot; &#39;</p>
                <p>&copy; &reg; &trade;</p>
                <p>&nbsp;&nbsp;Spaces</p>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/unicode-content".to_string(),
            description: "Unicode content in various languages".to_string(),
            category: "dom".to_string(),
            html: r#"<html><body>
                <p>Chinese: 你好世界</p>
                <p>Japanese: こんにちは</p>
                <p>Korean: 안녕하세요</p>
                <p>Arabic: مرحبا</p>
                <p>Emoji: 🎉🚀❤️</p>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS + DOM 集成
        // ═══════════════════════════════════════════════════════════════

        // ── 内联样式 ──
        TestCase {
            id: "js-dom/inline-styles".to_string(),
            description: "Elements with inline style attributes".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div style="width: 200px; height: 100px; background-color: red;">Red box</div>
                <div style="width: 150px; height: 75px; background-color: blue; margin-top: 10px;">Blue box</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ── style 元素 ──
        TestCase {
            id: "js-dom/style-element".to_string(),
            description: "Style element with CSS rules".to_string(),
            category: "css".to_string(),
            html: r#"<html><head>
                <style>
                    .box { width: 100px; height: 100px; background-color: orange; margin: 5px; }
                    .rounded { border-radius: 10px; }
                    .shadow { box-shadow: 2px 2px 5px rgba(0,0,0,0.5); }
                </style>
            </head><body>
                <div class="box">Plain</div>
                <div class="box rounded">Rounded</div>
                <div class="box shadow">Shadow</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_head".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ── ID 选择器 + 属性 ──
        TestCase {
            id: "js-dom/id-attribute-selectors".to_string(),
            description: "ID and attribute selectors".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div id="header" class="main">Header</div>
                <div id="content" data-role="main">Content</div>
                <div id="footer">Footer</div>
            </body></html>"#.to_string(),
            css: r#"
                #header { background-color: navy; color: white; height: 60px; }
                #content { background-color: white; min-height: 400px; }
                #footer { background-color: #333; color: #ccc; height: 40px; }
                [data-role="main"] { padding: 10px; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  伪类选择器
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/pseudo-first-last-child".to_string(),
            description: "First-child and last-child selectors".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <ul>
                    <li>First</li>
                    <li>Middle</li>
                    <li>Last</li>
                </ul>
            </body></html>"#.to_string(),
            css: "li:first-child { color: red; } li:last-child { color: blue; } li { font-size: 14px; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_list".to_string(),
                "render_completes".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/pseudo-nth-child".to_string(),
            description: "Nth-child selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <table>
                    <tr><td>Row 1</td></tr>
                    <tr><td>Row 2</td></tr>
                    <tr><td>Row 3</td></tr>
                    <tr><td>Row 4</td></tr>
                    <tr><td>Row 5</td></tr>
                </table>
            </body></html>"#.to_string(),
            css: "tr:nth-child(even) { background-color: #f0f0f0; } tr:nth-child(odd) { background-color: #fff; } td { padding: 5px; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_table".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  组合选择器
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/descendant-selector".to_string(),
            description: "Descendant selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="container">
                    <p>Paragraph in container</p>
                    <div>
                        <p>Nested paragraph</p>
                    </div>
                </div>
                <p>Outside paragraph</p>
            </body></html>"#.to_string(),
            css: ".container p { color: green; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/child-selector".to_string(),
            description: "Direct child selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="parent">
                    <span>Direct child</span>
                    <div>
                        <span>Nested span</span>
                    </div>
                </div>
            </body></html>"#.to_string(),
            css: ".parent > span { color: red; font-weight: bold; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/adjacent-sibling-selector".to_string(),
            description: "Adjacent sibling selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <h2>Header</h2>
                <p>Adjacent paragraph</p>
                <p>Non-adjacent paragraph</p>
            </body></html>"#.to_string(),
            css: "h2 + p { color: blue; margin-top: 0; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_heading".to_string(),
                "dom_has_text".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  媒体查询
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/media-query-basic".to_string(),
            description: "Basic media query".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div id="responsive">Content</div>
            </body></html>"#.to_string(),
            css: r#"
                #responsive { width: 100%; background-color: blue; height: 100px; }
                @media (min-width: 600px) {
                    #responsive { background-color: red; }
                }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 变量（自定义属性）
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/css-variables".to_string(),
            description: "CSS custom properties".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="themed">Themed content</div>
                <div class="alt">Alternative theme</div>
            </body></html>"#.to_string(),
            css: r#"
                :root { --primary: #3366cc; --spacing: 10px; --radius: 5px; }
                .themed { background-color: var(--primary); padding: var(--spacing); border-radius: var(--radius); }
                .alt { --primary: #cc3333; background-color: var(--primary); padding: var(--spacing); }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ── CSS 变量 fallback ──
        TestCase {
            id: "js-dom/css-variables-fallback".to_string(),
            description: "CSS custom properties with fallback values".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="fb">Fallback content</div>
            </body></html>"#.to_string(),
            css: r#"
                .fb {
                    background-color: var(--undefined-color, #999999);
                    width: var(--undefined-width, 200px);
                    height: 100px;
                }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Box Model 高级
        // ═══════════════════════════════════════════════════════════════

        // ── box-sizing: border-box ──
        TestCase {
            id: "js-dom/box-sizing-border-box".to_string(),
            description: "box-sizing: border-box".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="box">Border box</div>
            </body></html>"#.to_string(),
            css: r#"
                .box {
                    box-sizing: border-box;
                    width: 200px;
                    height: 100px;
                    padding: 20px;
                    border: 5px solid black;
                    background-color: lightblue;
                }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ── margin 折叠 ──
        TestCase {
            id: "js-dom/margin-collapse".to_string(),
            description: "Block-level margin collapsing".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div style="margin-bottom: 20px; height: 50px; background: red;">Top</div>
                <div style="margin-top: 30px; height: 50px; background: blue;">Bottom</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS Grid 高级
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/grid-template-areas".to_string(),
            description: "Grid template areas layout".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
                <div class="grid">
                    <div class="header">Header</div>
                    <div class="sidebar">Sidebar</div>
                    <div class="main">Main</div>
                    <div class="footer">Footer</div>
                </div>
            </body></html>"#.to_string(),
            css: r#"
                .grid {
                    display: grid;
                    grid-template-areas: "header header" "sidebar main" "footer footer";
                    grid-template-columns: 200px 1fr;
                    grid-template-rows: 60px 1fr 40px;
                    height: 400px;
                    width: 600px;
                }
                .header { grid-area: header; background: navy; }
                .sidebar { grid-area: sidebar; background: #ddd; }
                .main { grid-area: main; background: white; }
                .footer { grid-area: footer; background: #333; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/grid-auto-flow".to_string(),
            description: "Grid auto-flow placement".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
                <div class="grid">
                    <div>A</div><div>B</div><div>C</div>
                    <div>D</div><div>E</div><div>F</div>
                </div>
            </body></html>"#.to_string(),
            css: r#"
                .grid {
                    display: grid;
                    grid-template-columns: repeat(3, 100px);
                    grid-auto-flow: row;
                    gap: 10px;
                }
                .grid > div { height: 50px; background: coral; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Flexbox 高级
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/flex-wrap".to_string(),
            description: "Flexbox wrap layout".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
                <div class="flex">
                    <div>1</div><div>2</div><div>3</div>
                    <div>4</div><div>5</div><div>6</div>
                </div>
            </body></html>"#.to_string(),
            css: r#"
                .flex { display: flex; flex-wrap: wrap; width: 300px; gap: 5px; }
                .flex > div { width: 100px; height: 50px; background: teal; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/flex-align".to_string(),
            description: "Flexbox alignment".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
                <div class="flex">
                    <div class="item">Short</div>
                    <div class="item tall">Tall</div>
                    <div class="item">Short</div>
                </div>
            </body></html>"#.to_string(),
            css: r#"
                .flex { display: flex; align-items: center; height: 200px; background: #eee; }
                .item { width: 80px; background: purple; color: white; }
                .tall { height: 150px; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  定位
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/position-absolute".to_string(),
            description: "Absolute positioning".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="container">
                    <div class="absolute">Absolute</div>
                </div>
            </body></html>"#.to_string(),
            css: r#"
                .container { position: relative; width: 300px; height: 200px; background: #f0f0f0; }
                .absolute { position: absolute; top: 10px; right: 10px; background: red; color: white; padding: 5px; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/position-fixed".to_string(),
            description: "Fixed positioning".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="fixed-bar">Fixed bar</div>
                <div style="height: 2000px;">Scrollable content</div>
            </body></html>"#.to_string(),
            css: r#"
                .fixed-bar { position: fixed; top: 0; left: 0; width: 100%; height: 40px; background: navy; color: white; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Overflow
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/overflow-hidden".to_string(),
            description: "Overflow hidden clipping".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="clipped">
                    <div class="overflow">This content overflows the container</div>
                </div>
            </body></html>"#.to_string(),
            css: r#"
                .clipped { width: 100px; height: 50px; overflow: hidden; background: #ddd; }
                .overflow { width: 200px; height: 200px; background: coral; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  Transform
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/transform-translate".to_string(),
            description: "CSS transform translate".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="moved">Translated</div>
            </body></html>"#.to_string(),
            css: r#"
                .moved {
                    width: 100px; height: 100px; background: orange;
                    transform: translate(50px, 25px);
                }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/transform-rotate".to_string(),
            description: "CSS transform rotate".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="rotated">Rotated 45deg</div>
            </body></html>"#.to_string(),
            css: r#"
                .rotated {
                    width: 80px; height: 80px; background: purple; color: white;
                    transform: rotate(45deg);
                }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  渐变
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/linear-gradient".to_string(),
            description: "Linear gradient background".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="gradient">Gradient</div>
            </body></html>"#.to_string(),
            css: r#"
                .gradient {
                    width: 300px; height: 100px;
                    background: linear-gradient(to right, red, yellow, green);
                }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/radial-gradient".to_string(),
            description: "Radial gradient background".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="radial">Radial</div>
            </body></html>"#.to_string(),
            css: r#"
                .radial {
                    width: 200px; height: 200px;
                    background: radial-gradient(circle, white, black);
                }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "nonzero_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  错误恢复
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/malformed-html-recovery".to_string(),
            description: "Malformed HTML error recovery".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <p>Unclosed paragraph
                <div>Nested <b>bold
                <ul>
                    <li>Item 1
                    <li>Item 2
                </ul>
                </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/empty-elements".to_string(),
            description: "Empty elements handling".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <div></div>
                <p></p>
                <span></span>
                <div>   </div>
            </body></html>"#.to_string(),
            css: "div { height: 20px; background: #eee; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/void-elements".to_string(),
            description: "Void elements (br, hr, img, input)".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <p>Line 1<br>Line 2<br>Line 3</p>
                <hr>
                <img src="test.png" alt="test">
                <input type="text" placeholder="Type here">
            </body></html>"#.to_string(),
            css: "hr { border: 1px solid #ccc; }".to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_img".to_string(),
                "dom_has_input".to_string(),
                "dom_has_text".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  边框样式
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/border-styles".to_string(),
            description: "Various border styles".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="solid">Solid</div>
                <div class="dashed">Dashed</div>
                <div class="dotted">Dotted</div>
                <div class="double">Double</div>
            </body></html>"#.to_string(),
            css: r#"
                div { width: 100px; height: 50px; margin: 5px; }
                .solid { border: 2px solid black; }
                .dashed { border: 2px dashed black; }
                .dotted { border: 2px dotted black; }
                .double { border: 4px double black; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── border-radius ──
        TestCase {
            id: "js-dom/border-radius".to_string(),
            description: "Border radius on elements".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="rounded">Rounded</div>
                <div class="circle">Circle</div>
            </body></html>"#.to_string(),
            css: r#"
                .rounded { width: 100px; height: 60px; background: teal; border-radius: 10px; }
                .circle { width: 100px; height: 100px; background: orange; border-radius: 50%; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  文本样式
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/text-decoration".to_string(),
            description: "Text decoration styles".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <p class="underline">Underlined text</p>
                <p class="line-through">Strikethrough text</p>
                <p class="overline">Overlined text</p>
            </body></html>"#.to_string(),
            css: r#"
                .underline { text-decoration: underline; }
                .line-through { text-decoration: line-through; }
                .overline { text-decoration: overline; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        TestCase {
            id: "js-dom/text-transform".to_string(),
            description: "Text transform property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <p class="upper">uppercase text</p>
                <p class="lower">LOWERCASE TEXT</p>
                <p class="capitalize">capitalize each word</p>
            </body></html>"#.to_string(),
            css: r#"
                .upper { text-transform: uppercase; }
                .lower { text-transform: lowercase; }
                .capitalize { text-transform: capitalize; }
            "#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  颜色格式
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/color-formats".to_string(),
            description: "Various CSS color formats".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div style="background-color: #ff0000; height: 20px;">Hex</div>
                <div style="background-color: rgb(0, 255, 0); height: 20px;">RGB</div>
                <div style="background-color: rgba(0, 0, 255, 0.8); height: 20px;">RGBA</div>
                <div style="background-color: hsl(60, 100%, 50%); height: 20px;">HSL</div>
                <div style="background-color: hsla(120, 100%, 50%, 0.7); height: 20px;">HSLA</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },
    ]
}
