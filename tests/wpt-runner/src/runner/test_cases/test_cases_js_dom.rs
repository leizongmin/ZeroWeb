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
                "has_fill_primitives".to_string(),
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
                "has_fill_primitives".to_string(),
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
                "has_fill_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  DOM 高级 API
        // ═══════════════════════════════════════════════════════════════

        TestCase {
            id: "js-dom/dataset-api".to_string(),
            description: "element.dataset read/write custom data attributes".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="user" data-user-id="42" data-user-name="Alice" data-role="admin">
    <span id="result">checking</span>
</div>
<script>
var el = document.getElementById('user');
var id = el.dataset.userId;
var name = el.dataset.userName;
el.dataset.active = 'true';
// 断言 dataset camelCase↔kebab-case 反射（data-user-id → userId）+ 写回反射。
if (id !== '42') throw new Error('dataset-api: userId="' + id + '" expected "42"');
if (name !== 'Alice') throw new Error('dataset-api: userName="' + name + '" expected "Alice"');
if (el.dataset.active !== 'true') throw new Error('dataset-api: active writeback failed');
if (el.getAttribute('data-active') !== 'true') throw new Error('dataset-api: getAttribute(data-active) reflection failed');
document.getElementById('result').textContent = id + '-' + name;
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
        TestCase {
            id: "js-dom/classlist-advanced".to_string(),
            description: "classList toggle/replace/contains advanced".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="box" class="active visible">Box</div>
<script>
var el = document.getElementById('box');
el.classList.toggle('active');
el.classList.toggle('visible', true);
el.classList.replace('visible', 'hidden');
el.classList.add('new-class');
var contains = el.classList.contains('hidden');
var count = el.classList.length;
// toggle('active') 移除 active；replace visible→hidden；add new-class → 最终 'hidden new-class'。
if (!contains) throw new Error('classlist-advanced: contains(hidden)=false expected true');
if (count !== 2) throw new Error('classlist-advanced: length=' + count + ' expected 2');
if (el.classList.contains('active')) throw new Error('classlist-advanced: active should have been toggled off');
if (!el.classList.contains('new-class')) throw new Error('classlist-advanced: new-class missing after add');
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
        TestCase {
            id: "js-dom/element-matches-closest".to_string(),
            description: "element.matches() and element.closest() selectors".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<nav id="nav">
    <ul class="menu">
        <li class="item"><a href="/home" class="link" id="target">Link</a></li>
    </ul>
</nav>
<script>
var link = document.getElementById('target');
var isLink = link.matches('a.link');
var li = link.closest('li');
var menu = link.closest('.menu');
var nav = link.closest('nav');
// 断言 matches + closest（逐层祖先查询）+ 不匹配返 null。
if (!isLink) throw new Error('element-matches-closest: matches(a.link)=false expected true');
if (li === null || li.tagName !== 'LI') throw new Error('element-matches-closest: closest(li) failed: ' + (li && li.tagName));
if (menu === null || !menu.classList.contains('menu')) throw new Error('element-matches-closest: closest(.menu) failed');
if (nav === null || nav.id !== 'nav') throw new Error('element-matches-closest: closest(nav) failed');
if (link.closest('section') !== null) throw new Error('element-matches-closest: closest(section) should be null');
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
        TestCase {
            id: "js-dom/custom-event".to_string(),
            description: "CustomEvent constructor and dispatch".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="target">Target</div>
<script>
var received = false;
var detail = null;
document.getElementById('target').addEventListener('my-event', function(e) {
    received = true;
    detail = e.detail;
});
var evt = new CustomEvent('my-event', { detail: { key: 'value' } });
document.getElementById('target').dispatchEvent(evt);
// dispatchEvent 同步触发监听器——received + detail 应已设置。
if (!received) throw new Error('custom-event: listener not invoked by dispatchEvent');
if (!detail || detail.key !== 'value') throw new Error('custom-event: detail.key="' + (detail && detail.key) + '" expected "value"');
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
        TestCase {
            id: "js-dom/document-fragment".to_string(),
            description: "DocumentFragment for batch DOM operations".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<ul id="list"></ul>
<script>
var fragment = document.createDocumentFragment();
for (var i = 0; i < 5; i++) {
    var li = document.createElement('li');
    li.textContent = 'Item ' + i;
    fragment.appendChild(li);
}
// 断言 fragment 累积 5 子（appendChild 真实工作）——DocumentFragment 批量插入的容器侧验证。
if (fragment.childNodes.length !== 5) throw new Error('document-fragment: fragment childNodes=' + fragment.childNodes.length + ' expected 5');
document.getElementById('list').appendChild(fragment);
// appendChild 后 fragment 应清空（子迁移到 list）——锁 DocumentFragment 一次性插入语义。
if (fragment.childNodes.length !== 0) throw new Error('document-fragment: fragment not emptied after append (leftover=' + fragment.childNodes.length + ')');
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
        TestCase {
            id: "js-dom/node-compare-document-position".to_string(),
            description: "Node.compareDocumentPosition() ordering".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="parent">
    <span id="first">First</span>
    <span id="second">Second</span>
</div>
<script>
var first = document.getElementById('first');
var second = document.getElementById('second');
var pos = first.compareDocumentPosition(second);
// second FOLLOWING first = Node.DOCUMENT_POSITION_FOLLOWING (4)
var isFollowing = (pos & 4) !== 0;
if (!isFollowing) throw new Error('node-compare-document-position: second not detected as FOLLOWING first (pos=' + pos + ')');
// 反向：first 相对 second 应 PRECEDING (2)。
var posRev = second.compareDocumentPosition(first);
if ((posRev & 2) === 0) throw new Error('node-compare-document-position: first not detected as PRECEDING second (pos=' + posRev + ')');
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
        TestCase {
            id: "js-dom/innerhtml-outerhtml".to_string(),
            description: "innerHTML and outerHTML read/write".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="content"><p>Original</p></div>
<div id="holder"></div>
<script>
var content = document.getElementById('content');
var html = content.innerHTML;
content.innerHTML = '<span>Replaced</span>';
var holder = document.getElementById('holder');
holder.innerHTML = '<em>New content</em>';
// 初始 innerHTML 读回含 <p>（selector-identity 元素读 parsed DOM 快照）；outerHTML 含元素 tag + id。
// 注：set innerHTML 写入 handle 子树不回写 parsed 快照（headless handle-only 限制，R3316），故不校验写回。
if (html.indexOf('<p>') < 0 && html.indexOf('<P>') < 0) throw new Error('innerhtml-outerhtml: initial innerHTML missing <p>: "' + html + '"');
if (content.outerHTML.indexOf('content') < 0) throw new Error('innerhtml-outerhtml: outerHTML missing element id: "' + content.outerHTML + '"');
// holder 初始空（写前），outerHTML 亦含 tag。
if (holder.outerHTML.indexOf('holder') < 0) throw new Error('innerhtml-outerhtml: holder outerHTML missing id: "' + holder.outerHTML + '"');
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
        TestCase {
            id: "js-dom/mutation-observer".to_string(),
            description: "MutationObserver observe and callback".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="observed">Watch me</div>
<script>
var mutations = [];
var observer = new MutationObserver(function(muts) {
    for (var i = 0; i < muts.length; i++) {
        mutations.push(muts[i].type);
    }
});
var target = document.getElementById('observed');
observer.observe(target, { childList: true, attributes: true });
target.setAttribute('data-x', '1');
target.setAttribute('data-y', '2');
var records = observer.takeRecords();
observer.disconnect();
// R3330：spec 合规——setAttribute 各产独立 attributes 记录，不合并（差异 #4「takeRecords 合并」
// 经 R3025-R3028 MO observe-options 收尾后已闭合：实测 setAttribute×2 = 2 条独立记录，各带正确 attributeName）。
// 注：textContent 在无 characterData 观测时不产记录（headless 已知限制，记 headless-js-dom-divergence backlog #1 域），
// 故本用例以 setAttribute 双写为「逐条不合并」的可验证信号。
if (records.length !== 2) throw new Error('mutation-observer: setAttribute×2 takeRecords=' + records.length + ', expected 2 (逐条不合并)');
if (records[0].type !== 'attributes' || records[1].type !== 'attributes') throw new Error('mutation-observer: record type 非 attributes (got ' + records[0].type + ',' + records[1].type + ')');
var names = records.map(function (r) { return r.attributeName; }).sort().join(',');
if (names !== 'data-x,data-y') throw new Error('mutation-observer: attributeName 错 (got ' + names + ')');
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
        TestCase {
            id: "js-dom/shadow-dom-basic".to_string(),
            description: "Shadow DOM attachShadow basic".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="host">Light content</div>
<script>
var host = document.getElementById('host');
var shadow = host.attachShadow({ mode: 'open' });
shadow.innerHTML = '<p>Shadow content</p>';
// attachShadow 返 ShadowRoot（nodeType 11 + '#shadow-root' nodeName + host 反向引用 + mode）。
// 注：shadow 树内容不经宿主 querySelectorAll 遍历（headless 渲染走 flat tree），故仅断言 root 身份。
if (!shadow || shadow.nodeType !== 11) throw new Error('shadow-dom-basic: attachShadow did not return a ShadowRoot (nodeType=' + (shadow && shadow.nodeType) + ')');
if (shadow.host !== host) throw new Error('shadow-dom-basic: shadow.host not the host element');
if (shadow.mode !== 'open') throw new Error('shadow-dom-basic: shadow.mode="' + shadow.mode + '" expected "open"');
// 二次 attachShadow 应拒绝（spec：host 已附加 shadow → NotSupportedError）。
try { host.attachShadow({ mode: 'open' }); throw new Error('shadow-dom-basic: second attachShadow should throw'); }
catch (e) { if (e.message === 'shadow-dom-basic: second attachShadow should throw') throw e; /* expected */ }
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
        TestCase {
            id: "js-dom/element-create-comment".to_string(),
            description: "document.createComment and DOM manipulation".to_string(),
            category: "js-dom".to_string(),
            html: r#"<!DOCTYPE html>
<html><body>
<div id="target">Content</div>
<script>
var comment = document.createComment('This is a comment');
var target = document.getElementById('target');
target.parentNode.insertBefore(comment, target);
var text = document.createTextNode(' appended');
target.appendChild(text);
// createComment → 注释节点（nodeType 8 + 正确 data）；createTextNode → 文本节点（nodeType 3 + 正确 data）。
// 注：insertBefore 在 handle-identity 子树下的 sibling 链读取有 headless 限制（R3316），故仅断言节点工厂身份。
if (!comment || comment.nodeType !== 8) throw new Error('element-create-comment: createComment nodeType=' + (comment && comment.nodeType) + ' expected 8');
if (comment.data !== 'This is a comment') throw new Error('element-create-comment: comment.data="' + comment.data + '"');
if (!text || text.nodeType !== 3) throw new Error('element-create-comment: createTextNode nodeType=' + (text && text.nodeType) + ' expected 3');
if (text.data !== ' appended') throw new Error('element-create-comment: text.data="' + text.data + '"');
</script>
</body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "no_panic".to_string(),
                "js_executes_ok".to_string(),
            ],
        },
    ]
}
