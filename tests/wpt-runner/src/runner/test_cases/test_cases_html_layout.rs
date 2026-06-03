//! HTML/DOM 结构测试 + 布局测试 + 错误恢复测试。
//!
//! 包含 HTML/DOM Structural、Layout、Error Recovery 测试用例。

use super::TestCase;

/// 返回 HTML/DOM 结构、布局和错误恢复测试用例。
pub fn html_layout_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
                //  HTML/DOM STRUCTURAL TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── head with title and meta ──
                TestCase {
                    id: "html/head-elements".to_string(),
                    description: "HTML head with title and meta".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><head><title>Test</title><meta charset="utf-8"></head><body>Content</body></html>"#.to_string(),
                    css: String::new(),
                    assertions: vec![
                        "dom_has_head".to_string(),
                        "dom_has_title".to_string(),
                        "dom_has_meta".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── heading elements h1-h6 ──
                TestCase {
                    id: "html/headings".to_string(),
                    description: "HTML heading elements h1-h6".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <h1>Heading 1</h1>
                        <h2>Heading 2</h2>
                        <h3>Heading 3</h3>
                        <h4>Heading 4</h4>
                        <h5>Heading 5</h5>
                        <h6>Heading 6</h6>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec![
                        "dom_has_heading".to_string(),
                        "dom_has_text".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── unordered list ──
                TestCase {
                    id: "html/list-ul".to_string(),
                    description: "HTML unordered list".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <ul>
                            <li>Item A</li>
                            <li>Item B</li>
                            <li>Item C</li>
                        </ul>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec![
                        "dom_has_list".to_string(),
                        "dom_has_text".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── ordered list ──
                TestCase {
                    id: "html/list-ol".to_string(),
                    description: "HTML ordered list".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <ol>
                            <li>First</li>
                            <li>Second</li>
                            <li>Third</li>
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
                // ── nested lists ──
                TestCase {
                    id: "html/nested-lists".to_string(),
                    description: "HTML nested ul/ol".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <ul>
                            <li>Item 1
                                <ol>
                                    <li>Sub A</li>
                                    <li>Sub B</li>
                                </ol>
                            </li>
                            <li>Item 2</li>
                        </ul>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec![
                        "dom_has_list".to_string(),
                        "dom_has_text".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── definition list ──
                TestCase {
                    id: "html/dl-list".to_string(),
                    description: "HTML definition list".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <dl>
                            <dt>Term</dt>
                            <dd>Definition</dd>
                        </dl>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── strong and em ──
                TestCase {
                    id: "html/strong-em".to_string(),
                    description: "HTML inline formatting".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <p><strong>Bold</strong> and <em>italic</em> text</p>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── pre and code ──
                TestCase {
                    id: "html/pre-code".to_string(),
                    description: "HTML preformatted and code".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <pre><code>fn main() { println!("hi"); }</code></pre>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── blockquote ──
                TestCase {
                    id: "html/blockquote".to_string(),
                    description: "HTML blockquote element".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <blockquote>To be or not to be</blockquote>
                    </body></html>"#
                        .to_string(),
                    css: "blockquote { border-left: 3px solid gray; padding-left: 10px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── article and section ──
                TestCase {
                    id: "html/article-section".to_string(),
                    description: "HTML5 semantic elements".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <article>
                            <section>Section 1</section>
                            <section>Section 2</section>
                        </article>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec![
                        "render_completes".to_string(),
                        "dom_has_text".to_string(),
                        "layout_has_children".to_string(),
                    ],
                },
                // ── nav element ──
                TestCase {
                    id: "html/nav-element".to_string(),
                    description: "HTML nav element".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <nav><a href="/">Home</a> <a href="/about">About</a></nav>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec![
                        "dom_has_link".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── header and footer ──
                TestCase {
                    id: "html/header-footer".to_string(),
                    description: "HTML header and footer elements".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <header>Header</header>
                        <main>Main content</main>
                        <footer>Footer</footer>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec![
                        "render_completes".to_string(),
                        "dom_has_text".to_string(),
                        "layout_has_children".to_string(),
                    ],
                },
                // ── multiple style tags ──
                TestCase {
                    id: "html/multiple-stylesheets".to_string(),
                    description: "Multiple style tags".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><head><style>.a { color: red; }</style><style>.b { color: blue; }</style></head><body>
                        <div class="a">A</div>
                        <div class="b">B</div>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── script tag (should not crash) ──
                TestCase {
                    id: "html/script-tag".to_string(),
                    description: "Script tag should not crash".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <p>Before</p>
                        <script>var x = 1;</script>
                        <p>After</p>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── DOCTYPE declaration ──
                TestCase {
                    id: "html/doctype".to_string(),
                    description: "DOCTYPE declaration".to_string(),
                    category: "html".to_string(),
                    html: "<!DOCTYPE html><html><body>With DOCTYPE</body></html>".to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── HTML comments ──
                TestCase {
                    id: "html/comments".to_string(),
                    description: "HTML comments".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <!-- This is a comment -->
                        <p>Visible</p>
                        <!-- Another comment -->
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── HTML entities ──
                TestCase {
                    id: "html/entities".to_string(),
                    description: "HTML entities".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <p>&amp; &lt; &gt; &quot; &#169;</p>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  LAYOUT TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── CSS Grid basic ──
                TestCase {
                    id: "layout/grid-basic".to_string(),
                    description: "CSS Grid basic layout".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="grid">
                            <div class="g-item">1</div>
                            <div class="g-item">2</div>
                            <div class="g-item">3</div>
                            <div class="g-item">4</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".grid { display: grid; grid-template-columns: 1fr 1fr; width: 400px; } .g-item { background-color: lightblue; height: 50px; }".to_string(),
                    assertions: vec![
                        "layout_has_children".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── grid-template-areas ──
                TestCase {
                    id: "layout/grid-areas".to_string(),
                    description: "CSS grid-template-areas".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="grid-areas">
                            <div class="header">Header</div>
                            <div class="sidebar">Sidebar</div>
                            <div class="main">Main</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".grid-areas { display: grid; grid-template-areas: \"header header\" \"sidebar main\"; grid-template-columns: 200px 1fr; grid-template-rows: 50px 1fr; width: 400px; height: 200px; } .header { grid-area: header; background-color: #eee; } .sidebar { grid-area: sidebar; background-color: #ddd; } .main { grid-area: main; background-color: #ccc; }".to_string(),
                    assertions: vec![
                        "layout_has_children".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── multi-column content ──
                TestCase {
                    id: "layout/multi-column".to_string(),
                    description: "Multiple columns of content".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="cols">
                            <div class="col">Column 1</div>
                            <div class="col">Column 2</div>
                            <div class="col">Column 3</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".cols { display: flex; width: 600px; } .col { flex: 1; background-color: honeydew; height: 200px; }".to_string(),
                    assertions: vec![
                        "has_multiple_fills".to_string(),
                        "layout_has_children".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── deep nesting (10 levels) ──
                TestCase {
                    id: "layout/deep-nesting".to_string(),
                    description: "Deeply nested elements (10 levels)".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div id="l1"><div id="l2"><div id="l3"><div id="l4"><div id="l5">
                        <div id="l6"><div id="l7"><div id="l8"><div id="l9"><div id="l10">
                            Deep
                        </div></div></div></div></div>
                        </div></div></div></div></div>
                    </body></html>"#
                        .to_string(),
                    css: "div { width: 400px; height: 300px; background-color: #f0f0f0; }".to_string(),
                    assertions: vec![
                        "layout_has_deep_children".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── mixed inline and block ──
                TestCase {
                    id: "layout/mixed-content".to_string(),
                    description: "Mixed inline and block elements".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div>Block 1</div>
                        <p>Paragraph with <strong>bold</strong> and <em>italic</em></p>
                        <div>Block 2</div>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec![
                        "render_completes".to_string(),
                        "dom_has_text".to_string(),
                        "layout_has_children".to_string(),
                    ],
                },
                // ── wide content ──
                TestCase {
                    id: "layout/wide-content".to_string(),
                    description: "Content wider than viewport".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body><div class="wide">Wide</div></body></html>"#.to_string(),
                    css: ".wide { width: 2000px; height: 100px; background-color: plum; }".to_string(),
                    assertions: vec![
                        "has_fill_primitives".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── layout positive dimensions ──
                TestCase {
                    id: "layout/positive-dimensions".to_string(),
                    description: "Layout root has positive dimensions".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body><div class="box">Box</div></body></html>"#.to_string(),
                    css: ".box { width: 200px; height: 100px; background-color: sienna; }".to_string(),
                    assertions: vec![
                        "layout_width_positive".to_string(),
                        "layout_height_positive".to_string(),
                        "has_fill_primitives".to_string(),
                    ],
                },

                // ═══════════════════════════════════════════════════════════════
                //  ERROR RECOVERY TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── missing close tags ──
                TestCase {
                    id: "html/missing-close-tags".to_string(),
                    description: "Missing closing tags".to_string(),
                    category: "html".to_string(),
                    html: "<html><body><div><p>Missing close<div>Another".to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── extra close tags ──
                TestCase {
                    id: "html/extra-close-tags".to_string(),
                    description: "Extra closing tags".to_string(),
                    category: "html".to_string(),
                    html: "<html><body><div>OK</div></p></span></body></html>".to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── invalid attributes ──
                TestCase {
                    id: "html/invalid-attributes".to_string(),
                    description: "Malformed attributes".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body><div = "bad" 3attr="no" data-ok="yes">Attr test</div></body></html>"#.to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── empty/void elements ──
                TestCase {
                    id: "html/empty-elements".to_string(),
                    description: "Self-closing void elements".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <br />
                        <hr />
                        <img src="x.png" />
                        <input type="text" />
                        <meta name="test" />
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec![
                        "render_completes".to_string(),
                        "no_panic".to_string(),
                        "dom_has_body".to_string(),
                    ],
                },
                // ── very large document (1000 divs) ──
                TestCase {
                    id: "html/very-large".to_string(),
                    description: "Very large document (1000 divs)".to_string(),
                    category: "html".to_string(),
                    html: {
                        let mut h = String::from("<html><body>");
                        for i in 0..1000 {
                            h.push_str(&format!("<div>{i}</div>"));
                        }
                        h.push_str("</body></html>");
                        h
                    },
                    css: "div { width: 100px; height: 10px; background-color: #ddd; }".to_string(),
                    assertions: vec![
                        "render_completes".to_string(),
                        "no_panic".to_string(),
                        "layout_has_children".to_string(),
                    ],
                },
    ]
}
