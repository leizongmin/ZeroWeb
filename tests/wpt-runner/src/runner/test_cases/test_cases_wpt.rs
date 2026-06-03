//! WPT 扩展测试用例。
//!
//! 包含 WPT Expansion 各阶段的测试：CSS Selectors、CSS Properties、
//! HTML Elements、CSS @rules、Layout edge cases。

use super::TestCase;

/// 返回 WPT 扩展测试用例。
pub fn wpt_expansion_tests() -> Vec<TestCase> {
    vec![
        //  WPT EXPANSION — CSS Selectors
                // ═══════════════════════════════════════════════════════════════

                // ── :lang selector ──
                TestCase {
                    id: "css/lang-selector".to_string(),
                    description: ":lang pseudo-class selector".to_string(),
                    category: "css".to_string(),
                    html: r#"<html lang="en"><body>
                        <p>English</p>
                        <p lang="fr">French</p>
                    </body></html>"#.to_string(),
                    css: "p:lang(en) { color: blue; } p:lang(fr) { color: red; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── :nth-of-type selector ──
                TestCase {
                    id: "css/nth-of-type".to_string(),
                    description: ":nth-of-type selector".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div><p>First</p><span>Second</span><p>Third</p></div>
                    </body></html>"#.to_string(),
                    css: "p:nth-of-type(odd) { color: red; } span:nth-of-type(1) { color: blue; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── :nth-last-child selector ──
                TestCase {
                    id: "css/nth-last-child".to_string(),
                    description: ":nth-last-child selector".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><ul>
                        <li>A</li><li>B</li><li>C</li><li>D</li>
                    </ul></body></html>"#.to_string(),
                    css: "li:nth-last-child(2) { color: green; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── Universal selector ──
                TestCase {
                    id: "css/universal-selector".to_string(),
                    description: "Universal * selector".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div><p>Text</p></div>
                    </body></html>"#.to_string(),
                    css: "* { margin: 0; padding: 0; } div { width: 200px; height: 100px; background-color: navy; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
                },
                // ── Descendant combinator ──
                TestCase {
                    id: "css/descendant-combinator".to_string(),
                    description: "Descendant combinator (space)".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="outer"><div class="inner"><p>Nested</p></div></div>
                    </body></html>"#.to_string(),
                    css: ".outer .inner p { color: orange; font-size: 16px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── Child combinator ──
                TestCase {
                    id: "css/child-combinator".to_string(),
                    description: "Child combinator (>)".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="parent"><p>Direct</p><div><p>Indirect</p></div></div>
                    </body></html>"#.to_string(),
                    css: ".parent > p { color: crimson; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── Adjacent sibling combinator ──
                TestCase {
                    id: "css/adjacent-sibling".to_string(),
                    description: "Adjacent sibling combinator (+)".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <h2>Heading</h2><p>First after h2</p><p>Second paragraph</p>
                    </body></html>"#.to_string(),
                    css: "h2 + p { color: purple; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── Attribute selectors ──
                TestCase {
                    id: "css/attribute-selectors".to_string(),
                    description: "Attribute selectors [attr], [attr=value]".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <input type="text" value="text input"/>
                        <input type="submit" value="Submit"/>
                    </body></html>"#.to_string(),
                    css: r#"input[type="submit"] { background-color: green; color: white; }"#.to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_input".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  WPT EXPANSION — CSS Properties
                // ═══════════════════════════════════════════════════════════════

                // ── CSS custom properties (var) ──
                TestCase {
                    id: "css/custom-properties".to_string(),
                    description: "CSS custom properties with var()".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="themed">Themed Box</div>
                    </body></html>"#.to_string(),
                    css: ":root { --main-color: #336699; --spacing: 20px; } .themed { color: var(--main-color); padding: var(--spacing); background-color: wheat; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string(), "has_fill_primitives".to_string()],
                },
                // ── CSS calc() ──
                TestCase {
                    id: "css/calc-function".to_string(),
                    description: "CSS calc() function".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="calc-box">Calculated</div>
                    </body></html>"#.to_string(),
                    css: ".calc-box { width: calc(100% - 40px); height: calc(50px + 20px); background-color: peru; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
                },
                // ── CSS min()/max()/clamp() ──
                TestCase {
                    id: "css/min-max-clamp".to_string(),
                    description: "CSS min(), max(), clamp() functions".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="clamped">Responsive</div>
                    </body></html>"#.to_string(),
                    css: ".clamped { width: clamp(200px, 50%, 500px); height: min(300px, 50vh); background-color: salmon; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
                },
                // ── CSS opacity ──
                TestCase {
                    id: "css/opacity".to_string(),
                    description: "CSS opacity property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="opaque">Opaque</div>
                        <div class="translucent">Translucent</div>
                    </body></html>"#.to_string(),
                    css: ".opaque { opacity: 1; background-color: blue; } .translucent { opacity: 0.5; background-color: red; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_multiple_fills".to_string()],
                },
                // ── CSS visibility ──
                TestCase {
                    id: "css/visibility".to_string(),
                    description: "CSS visibility property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="visible">Visible</div>
                        <div class="hidden">Hidden</div>
                    </body></html>"#.to_string(),
                    css: ".visible { visibility: visible; background-color: olive; width: 100px; height: 50px; } .hidden { visibility: hidden; background-color: purple; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
                },
                // ── CSS z-index ──
                TestCase {
                    id: "css/z-index".to_string(),
                    description: "CSS z-index stacking order".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="bottom">Bottom</div>
                        <div class="top">Top</div>
                    </body></html>"#.to_string(),
                    css: ".bottom { position: absolute; z-index: 1; background-color: gray; width: 200px; height: 100px; top: 10px; left: 10px; } .top { position: absolute; z-index: 2; background-color: silver; width: 200px; height: 100px; top: 50px; left: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_multiple_fills".to_string()],
                },
                // ── CSS font shorthand ──
                TestCase {
                    id: "css/font-shorthand".to_string(),
                    description: "CSS font shorthand property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="bold">Bold text</p>
                        <p class="italic">Italic text</p>
                    </body></html>"#.to_string(),
                    css: ".bold { font: bold 18px serif; } .italic { font: italic 14px sans-serif; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── CSS letter-spacing ──
                TestCase {
                    id: "css/letter-spacing".to_string(),
                    description: "CSS letter-spacing property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="spaced">Spaced out text</p>
                    </body></html>"#.to_string(),
                    css: ".spaced { letter-spacing: 5px; color: teal; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── CSS word-spacing ──
                TestCase {
                    id: "css/word-spacing".to_string(),
                    description: "CSS word-spacing property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="wide">Wide word spacing here</p>
                    </body></html>"#.to_string(),
                    css: ".wide { word-spacing: 10px; color: navy; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── CSS line-height ──
                TestCase {
                    id: "css/line-height".to_string(),
                    description: "CSS line-height property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="tall">Line one
        Line two
        Line three</p>
                    </body></html>"#.to_string(),
                    css: ".tall { line-height: 2; color: maroon; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── CSS text-align ──
                TestCase {
                    id: "css/text-align".to_string(),
                    description: "CSS text-align values".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="center">Centered</p>
                        <p class="right">Right aligned</p>
                    </body></html>"#.to_string(),
                    css: ".center { text-align: center; } .right { text-align: right; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── CSS white-space ──
                TestCase {
                    id: "css/white-space".to_string(),
                    description: "CSS white-space property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="nowrap">This is a very long line that should not wrap.</p>
                    </body></html>"#.to_string(),
                    css: ".nowrap { white-space: nowrap; color: darkblue; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  WPT EXPANSION — HTML Elements
                // ═══════════════════════════════════════════════════════════════

                // ── HTML article/section/nav ──
                TestCase {
                    id: "html/semantic-elements".to_string(),
                    description: "HTML5 semantic elements".to_string(),
                    category: "html".to_string(),
                    html: r##"<html><body>
                        <header><h1>Title</h1></header>
                        <nav><a href="#">Home</a></nav>
                        <main>
                            <article><h2>Article</h2><p>Content</p></article>
                            <aside><p>Sidebar</p></aside>
                        </main>
                        <footer><p>Footer</p></footer>
                    </body></html>"##.to_string(),
                    css: "article { background-color: lightyellow; padding: 10px; } aside { background-color: lightblue; padding: 10px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string(), "has_fill_primitives".to_string()],
                },
                // ── HTML mark/abbr/code ──
                TestCase {
                    id: "html/inline-semantic".to_string(),
                    description: "HTML inline semantic elements".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <p>Use <code>println!</code> to output text</p>
                        <p>This is <mark>highlighted</mark> text</p>
                        <p><abbr title="HyperText Markup Language">HTML</abbr> is a standard</p>
                    </body></html>"#.to_string(),
                    css: "code { background-color: #f0f0f0; padding: 2px 4px; } mark { background-color: yellow; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── HTML dl/dt/dd ──
                TestCase {
                    id: "html/definition-list".to_string(),
                    description: "HTML definition list".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <dl>
                            <dt>Term 1</dt><dd>Definition 1</dd>
                            <dt>Term 2</dt><dd>Definition 2</dd>
                        </dl>
                    </body></html>"#.to_string(),
                    css: "dt { font-weight: bold; } dd { margin-left: 20px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── HTML blockquote ──
                TestCase {
                    id: "html/blockquote".to_string(),
                    description: "HTML blockquote element".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <blockquote cite="https://example.com">
                            <p>To be or not to be.</p>
                            <footer>— Shakespeare</footer>
                        </blockquote>
                    </body></html>"#.to_string(),
                    css: "blockquote { border-left: 4px solid gray; padding-left: 16px; margin: 16px 0; color: #333; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── HTML pre ──
                TestCase {
                    id: "html/pre-element".to_string(),
                    description: "HTML preformatted text".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <pre>fn main() {
            println!("Hello");
        }</pre>
                    </body></html>"#.to_string(),
                    css: "pre { background-color: #2d2d2d; color: #f8f8f2; padding: 16px; overflow: auto; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string(), "has_fill_primitives".to_string()],
                },
                // ── HTML fieldset/legend ──
                TestCase {
                    id: "html/fieldset".to_string(),
                    description: "HTML fieldset and legend".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <fieldset>
                            <legend>Personal Info</legend>
                            <label>Name: <input type="text"/></label>
                            <label>Email: <input type="email"/></label>
                        </fieldset>
                    </body></html>"#.to_string(),
                    css: "fieldset { border: 2px solid gray; padding: 10px; } legend { font-weight: bold; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_input".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  WPT EXPANSION — CSS @rules
                // ═══════════════════════════════════════════════════════════════

                // ── @keyframes animation ──
                TestCase {
                    id: "css/keyframes-animation".to_string(),
                    description: "CSS @keyframes animation".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="animated-box">Animated</div>
                    </body></html>"#.to_string(),
                    css: "@keyframes pulse { from { opacity: 1; } to { opacity: 0.5; } } .animated-box { animation: pulse 2s ease-in-out infinite alternate; width: 200px; height: 100px; background-color: coral; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
                },
                // ── @layer basic ──
                TestCase {
                    id: "css/layer-basic".to_string(),
                    description: "CSS @layer cascade ordering".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="text">Layered text</p>
                    </body></html>"#.to_string(),
                    css: "@layer base { .text { color: red; } } @layer override { .text { color: blue; } }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── @supports rule ──
                TestCase {
                    id: "css/supports-display-grid".to_string(),
                    description: "CSS @supports with display: grid".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="container"><div class="item">Grid item</div></div>
                    </body></html>"#.to_string(),
                    css: "@supports (display: grid) { .container { display: grid; grid-template-columns: 1fr; } } @supports not (display: grid) { .container { display: block; } }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── @media rule ──
                TestCase {
                    id: "css/media-query".to_string(),
                    description: "CSS @media rule".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="responsive">Responsive box</div>
                    </body></html>"#.to_string(),
                    css: ".responsive { background-color: blue; color: white; width: 200px; height: 100px; } @media (max-width: 600px) { .responsive { background-color: red; } }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string(), "has_glyph_primitives".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  WPT EXPANSION — Layout edge cases
                // ═══════════════════════════════════════════════════════════════

                // ── Flexbox wrap ──
                TestCase {
                    id: "layout/flex-wrap".to_string(),
                    description: "Flexbox wrap behavior".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="flex-container">
                            <div class="item">A</div><div class="item">B</div>
                            <div class="item">C</div><div class="item">D</div>
                        </div>
                    </body></html>"#.to_string(),
                    css: ".flex-container { display: flex; flex-wrap: wrap; width: 300px; } .item { width: 100px; height: 50px; background-color: teal; margin: 5px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_multiple_fills".to_string()],
                },
                // ── Grid auto rows ──
                TestCase {
                    id: "layout/grid-auto-rows".to_string(),
                    description: "Grid with auto rows".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="grid">
                            <div>R1C1</div><div>R1C2</div>
                            <div>R2C1</div><div>R2C2</div>
                        </div>
                    </body></html>"#.to_string(),
                    css: ".grid { display: grid; grid-template-columns: 1fr 1fr; grid-auto-rows: 100px; gap: 10px; } .grid > div { background-color: lavender; padding: 10px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_multiple_fills".to_string(), "has_glyph_primitives".to_string()],
                },
                // ── Nested flexbox ──
                TestCase {
                    id: "layout/nested-flex-columns".to_string(),
                    description: "Nested flexbox with column direction".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="outer">
                            <div class="inner">
                                <div>Child 1</div><div>Child 2</div>
                            </div>
                        </div>
                    </body></html>"#.to_string(),
                    css: ".outer { display: flex; flex-direction: row; height: 300px; } .inner { display: flex; flex-direction: column; background-color: mistyrose; } .inner > div { background-color: lavenderblush; padding: 10px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "layout_has_deep_children".to_string()],
                },
                // ── Absolute in relative ──
                TestCase {
                    id: "layout/absolute-in-relative".to_string(),
                    description: "Absolute positioning in relative container".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="relative-box">
                            <div class="absolute-box">Overlapping</div>
                        </div>
                    </body></html>"#.to_string(),
                    css: ".relative-box { position: relative; width: 300px; height: 200px; background-color: lightgray; } .absolute-box { position: absolute; top: 50px; left: 50px; width: 100px; height: 80px; background-color: tomato; color: white; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_multiple_fills".to_string()],
                },
                // ── Multiple CSS transforms ──
                TestCase {
                    id: "css/transform-multiple".to_string(),
                    description: "Multiple CSS transform functions".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="transformed">Transformed</div>
                    </body></html>"#.to_string(),
                    css: ".transformed { width: 100px; height: 100px; background-color: mediumpurple; transform: translateX(50px) rotate(15deg) scale(1.2); }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
                },
                // ── CSS gradient stops ──
                TestCase {
                    id: "css/gradient-with-stops".to_string(),
                    description: "Gradient with explicit color stops".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="gradient">Gradient</div>
                    </body></html>"#.to_string(),
                    css: ".gradient { width: 300px; height: 200px; background: linear-gradient(to right, red 0%, yellow 50%, green 100%); color: white; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
                },
                // ── Complex page layout ──
                TestCase {
                    id: "layout/complex-page".to_string(),
                    description: "Complex page with header, sidebar, main, footer".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="page">
                            <header class="hdr">Header</header>
                            <div class="body-row">
                                <nav class="sidebar">Nav</nav>
                                <main class="content">
                                    <h1>Main Content</h1>
                                    <p>Paragraph text here.</p>
                                </main>
                            </div>
                            <footer class="ftr">Footer</footer>
                        </div>
                    </body></html>"#.to_string(),
                    css: ".page { display: flex; flex-direction: column; min-height: 100vh; } .hdr { background: #333; color: white; padding: 10px; } .body-row { display: flex; flex: 1; } .sidebar { width: 200px; background: #f4f4f4; padding: 10px; } .content { flex: 1; padding: 20px; } .ftr { background: #333; color: white; padding: 10px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "layout_has_deep_children".to_string(), "has_glyph_primitives".to_string()],
                },
    ]
}
