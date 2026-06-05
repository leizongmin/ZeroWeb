//! CSS 选择器和属性标准合规性测试。
//!
//! 覆盖复杂选择器、CSS 变量、@规则交互、伪类/伪元素、
//! CSS 数学函数、逻辑属性、渐变、变换等高级特性。

use super::TestCase;

/// 返回 CSS 选择器和属性合规性测试用例。
pub fn css_compliance_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  CSS 选择器 — 组合器
        // ═══════════════════════════════════════════════════════════════

        // ── 子代选择器 ──
        TestCase {
            id: "css/child-combinator".to_string(),
            description: "Child combinator (>) selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="parent">
                    <p>Direct child</p>
                    <span><p>Not direct child</p></span>
                </div>
            </body></html>"#
                .to_string(),
            css: ".parent > p { color: red; background-color: #ffe; padding: 5px; }".to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── 相邻兄弟选择器 ──
        TestCase {
            id: "css/adjacent-sibling".to_string(),
            description: "Adjacent sibling (+) selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <h2>Heading</h2>
                <p>First paragraph after heading</p>
                <p>Second paragraph</p>
            </body></html>"#
                .to_string(),
            css: "h2 + p { color: blue; font-weight: bold; }".to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── 通用兄弟选择器 ──
        TestCase {
            id: "css/general-sibling".to_string(),
            description: "General sibling (~) selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <h2>Heading</h2>
                <p>First</p>
                <div>Divider</div>
                <p>Second</p>
            </body></html>"#
                .to_string(),
            css: "h2 ~ p { color: green; }".to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── 属性选择器 ──
        TestCase {
            id: "css/attribute-selectors".to_string(),
            description: "Various attribute selectors".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <input type="text" placeholder="Text input">
                <input type="password" placeholder="Password">
                <input type="submit" value="Go">
                <a href="https://example.com">Link</a>
            </body></html>"#
                .to_string(),
            css: r#"
                input[type="text"] { border: 1px solid blue; }
                input[type="password"] { border: 1px solid red; }
                a[href^="https"] { color: green; }
                a[href$=".com"] { font-weight: bold; }
                [placeholder] { padding: 5px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_input".to_string(),
                "dom_has_link".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 选择器 — 伪类
        // ═══════════════════════════════════════════════════════════════

        // ── :first-child / :last-child ──
        TestCase {
            id: "css/first-last-child".to_string(),
            description: ":first-child and :last-child selectors".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <ul>
                    <li>First</li>
                    <li>Middle</li>
                    <li>Last</li>
                </ul>
            </body></html>"#
                .to_string(),
            css: "li:first-child { color: green; } li:last-child { color: red; }".to_string(),
            assertions: vec![
                "dom_has_list".to_string(),
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── :only-child ──
        TestCase {
            id: "css/only-child".to_string(),
            description: ":only-child selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div><p>Only child paragraph</p></div>
                <div><p>First</p><p>Second</p></div>
            </body></html>"#
                .to_string(),
            css: "p:only-child { color: purple; font-weight: bold; }".to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── :empty ──
        TestCase {
            id: "css/empty-selector".to_string(),
            description: ":empty selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="box"></div>
                <div class="box">Has content</div>
                <div class="box"></div>
            </body></html>"#
                .to_string(),
            css: ".box { width: 100px; height: 50px; border: 1px solid #ccc; } .box:empty { background-color: #f0f0f0; }"
                .to_string(),
            assertions: vec![
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── :not() ──
        TestCase {
            id: "css/not-selector".to_string(),
            description: ":not() negation selector".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <p class="special">Special</p>
                <p>Normal 1</p>
                <p>Normal 2</p>
            </body></html>"#
                .to_string(),
            css: "p:not(.special) { color: gray; } .special { color: red; }".to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── :is() / :where() ──
        TestCase {
            id: "css/is-where-selector".to_string(),
            description: ":is() and :where() selectors".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <h1>Heading 1</h1>
                <h2>Heading 2</h2>
                <h3>Heading 3</h3>
                <p>Paragraph</p>
            </body></html>"#
                .to_string(),
            css: ":is(h1, h2, h3) { color: navy; } :where(h1, h2) { text-decoration: underline; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "dom_has_heading".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — 自定义属性和变量
        // ═══════════════════════════════════════════════════════════════

        // ── CSS 自定义属性基础 ──
        TestCase {
            id: "css/custom-properties".to_string(),
            description: "CSS custom properties (--*)".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="card">Card content</div>
            </body></html>"#
                .to_string(),
            css: r#"
                :root {
                    --main-color: #3498db;
                    --padding: 20px;
                    --radius: 8px;
                }
                .card {
                    background-color: var(--main-color);
                    color: white;
                    padding: var(--padding);
                    border-radius: var(--radius);
                }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── CSS 自定义属性 fallback ──
        TestCase {
            id: "css/custom-properties-fallback".to_string(),
            description: "CSS custom properties with fallback values".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="fb">Fallback content</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .fb {
                    color: var(--undefined-color, #e74c3c);
                    padding: var(--undefined-padding, 15px);
                    background-color: var(--bg, lightgray);
                }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — 数学函数
        // ═══════════════════════════════════════════════════════════════

        // ── calc() 函数 ──
        TestCase {
            id: "css/calc-function".to_string(),
            description: "CSS calc() function".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="calc-box">Calc sized box</div>
            </body></html>"#
                .to_string(),
            css: ".calc-box { width: calc(100% - 40px); height: calc(50px + 20px); background-color: coral; padding: 10px; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── min()/max()/clamp() ──
        TestCase {
            id: "css/min-max-clamp".to_string(),
            description: "CSS min(), max(), clamp() functions".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="min-box">Min</div>
                <div class="max-box">Max</div>
                <div class="clamp-box">Clamp</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .min-box { width: min(200px, 50%); height: 50px; background-color: teal; }
                .max-box { width: max(100px, 30%); height: 50px; background-color: olive; }
                .clamp-box { width: clamp(100px, 25%, 300px); height: 50px; background-color: maroon; color: white; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — 渐变
        // ═══════════════════════════════════════════════════════════════

        // ── linear-gradient 方向变体 ──
        TestCase {
            id: "css/linear-gradient-directions".to_string(),
            description: "Linear gradient with various directions".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="grad-right">Right</div>
                <div class="grad-down">Down</div>
                <div class="grad-deg">45deg</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .grad-right { background: linear-gradient(to right, red, blue); height: 50px; margin: 5px; }
                .grad-down { background: linear-gradient(to bottom, green, yellow); height: 50px; margin: 5px; }
                .grad-deg { background: linear-gradient(45deg, purple, orange); height: 50px; margin: 5px; color: white; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },
        // ── radial-gradient ──
        TestCase {
            id: "css/radial-gradient".to_string(),
            description: "Radial gradient".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="radial">Radial</div>
            </body></html>"#
                .to_string(),
            css: ".radial { background: radial-gradient(circle, white, blue); width: 200px; height: 200px; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — 变换和过渡
        // ═══════════════════════════════════════════════════════════════

        // ── transform 多函数组合 ──
        TestCase {
            id: "css/transform-combined".to_string(),
            description: "Combined CSS transform functions".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="transformed">Transformed box</div>
            </body></html>"#
                .to_string(),
            css: ".transformed { width: 100px; height: 80px; background-color: purple; color: white; transform: translate(20px, 10px) rotate(5deg) scale(1.1); }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── transform-origin ──
        TestCase {
            id: "css/transform-origin".to_string(),
            description: "CSS transform-origin property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="origin-box">Origin</div>
            </body></html>"#
                .to_string(),
            css: ".origin-box { width: 100px; height: 80px; background-color: darkblue; color: white; transform: rotate(15deg); transform-origin: top left; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── transition 属性 ──
        TestCase {
            id: "css/transition-property".to_string(),
            description: "CSS transition property definition".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="transition-box">Hover me</div>
            </body></html>"#
                .to_string(),
            css: ".transition-box { width: 100px; height: 80px; background-color: steelblue; color: white; transition: background-color 0.3s ease, transform 0.3s ease-in-out; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — 布局属性
        // ═══════════════════════════════════════════════════════════════

        // ── box-sizing: border-box ──
        TestCase {
            id: "css/box-sizing-border-box".to_string(),
            description: "box-sizing: border-box".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="border-box">Border box</div>
                <div class="content-box">Content box</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .border-box { box-sizing: border-box; width: 200px; height: 100px; padding: 20px; border: 5px solid red; background-color: lightblue; }
                .content-box { box-sizing: content-box; width: 200px; height: 100px; padding: 20px; border: 5px solid blue; background-color: lightyellow; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── position: relative + absolute ──
        TestCase {
            id: "css/position-relative-absolute".to_string(),
            description: "Relative and absolute positioning".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="relative-parent">
                    <div class="absolute-child">Absolute</div>
                    <div class="static-sibling">Static</div>
                </div>
            </body></html>"#
                .to_string(),
            css: r#"
                .relative-parent { position: relative; width: 300px; height: 200px; background-color: #eee; }
                .absolute-child { position: absolute; top: 10px; right: 10px; background-color: coral; padding: 5px; }
                .static-sibling { background-color: lightgreen; padding: 5px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── position: fixed ──
        TestCase {
            id: "css/position-fixed".to_string(),
            description: "Fixed positioning".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="content">Page content</div>
                <div class="fixed-bar">Fixed bar</div>
            </body></html>"#
                .to_string(),
            css: ".content { height: 1000px; background-color: white; } .fixed-bar { position: fixed; bottom: 0; left: 0; right: 0; background-color: #333; color: white; padding: 10px; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── Flexbox 居中 ──
        TestCase {
            id: "css/flex-centering".to_string(),
            description: "Flexbox centering".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="flex-center">
                    <div>Centered</div>
                </div>
            </body></html>"#
                .to_string(),
            css: ".flex-center { display: flex; justify-content: center; align-items: center; width: 300px; height: 200px; background-color: #f0f0f0; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── Flexbox wrap ──
        TestCase {
            id: "css/flex-wrap".to_string(),
            description: "Flexbox wrap".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="flex-wrap">
                    <div class="item">1</div>
                    <div class="item">2</div>
                    <div class="item">3</div>
                    <div class="item">4</div>
                    <div class="item">5</div>
                </div>
            </body></html>"#
                .to_string(),
            css: ".flex-wrap { display: flex; flex-wrap: wrap; width: 200px; } .item { width: 80px; height: 40px; background-color: tomato; margin: 5px; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "layout_has_children".to_string(),
            ],
        },
        // ── Grid 响应式布局 ──
        TestCase {
            id: "css/grid-responsive".to_string(),
            description: "Responsive Grid layout".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="grid-responsive">
                    <div>A</div><div>B</div><div>C</div><div>D</div><div>E</div>
                </div>
            </body></html>"#
                .to_string(),
            css: ".grid-responsive { display: grid; grid-template-columns: repeat(auto-fill, minmax(100px, 1fr)); gap: 10px; } .grid-responsive > div { background-color: mediumpurple; color: white; padding: 10px; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "layout_has_children".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — 颜色格式
        // ═══════════════════════════════════════════════════════════════

        // ── 多种颜色格式 ──
        TestCase {
            id: "css/color-formats".to_string(),
            description: "Various CSS color formats".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="hex">Hex</div>
                <div class="rgb">RGB</div>
                <div class="rgba">RGBA</div>
                <div class="hsl">HSL</div>
                <div class="hsla">HSLA</div>
                <div class="named">Named</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .hex { background-color: #ff6347; height: 30px; margin: 2px; }
                .rgb { background-color: rgb(0, 128, 255); height: 30px; margin: 2px; }
                .rgba { background-color: rgba(255, 0, 0, 0.5); height: 30px; margin: 2px; }
                .hsl { background-color: hsl(120, 100%, 50%); height: 30px; margin: 2px; }
                .hsla { background-color: hsla(240, 100%, 50%, 0.7); height: 30px; margin: 2px; }
                .named { background-color: dodgerblue; height: 30px; margin: 2px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── opacity ──
        TestCase {
            id: "css/opacity".to_string(),
            description: "CSS opacity property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="opaque">Opaque</div>
                <div class="semi-transparent">Semi-transparent</div>
                <div class="nearly-invisible">Nearly invisible</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .opaque { opacity: 1.0; background-color: red; height: 30px; margin: 2px; }
                .semi-transparent { opacity: 0.5; background-color: green; height: 30px; margin: 2px; }
                .nearly-invisible { opacity: 0.1; background-color: blue; height: 30px; margin: 2px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — 字体和文本
        // ═══════════════════════════════════════════════════════════════

        // ── font shorthand ──
        TestCase {
            id: "css/font-shorthand".to_string(),
            description: "CSS font shorthand property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <p class="bold-italic">Bold italic text</p>
                <p class="small-caps">Small caps text</p>
                <p class="mono">Monospace text</p>
            </body></html>"#
                .to_string(),
            css: r#"
                .bold-italic { font: bold italic 16px/1.5 serif; }
                .small-caps { font: small-caps 14px sans-serif; }
                .mono { font: 12px monospace; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── text-decoration ──
        TestCase {
            id: "css/text-decoration".to_string(),
            description: "CSS text-decoration property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <p class="underline">Underlined text</p>
                <p class="line-through">Strikethrough text</p>
                <p class="overline">Overlined text</p>
            </body></html>"#
                .to_string(),
            css: r#"
                .underline { text-decoration: underline; }
                .line-through { text-decoration: line-through; }
                .overline { text-decoration: overline; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── text-transform ──
        TestCase {
            id: "css/text-transform".to_string(),
            description: "CSS text-transform property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <p class="uppercase">hello world</p>
                <p class="lowercase">HELLO WORLD</p>
                <p class="capitalize">hello world</p>
            </body></html>"#
                .to_string(),
            css: r#"
                .uppercase { text-transform: uppercase; }
                .lowercase { text-transform: lowercase; }
                .capitalize { text-transform: capitalize; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        // ── letter-spacing / word-spacing ──
        TestCase {
            id: "css/letter-word-spacing".to_string(),
            description: "CSS letter-spacing and word-spacing".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <p class="wide-letters">Wide letter spacing</p>
                <p class="wide-words">Wide word spacing</p>
            </body></html>"#
                .to_string(),
            css: r#"
                .wide-letters { letter-spacing: 5px; }
                .wide-words { word-spacing: 10px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — 边框和背景
        // ═══════════════════════════════════════════════════════════════

        // ── border-radius ──
        TestCase {
            id: "css/border-radius".to_string(),
            description: "CSS border-radius property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="rounded">Rounded</div>
                <div class="pill">Pill shape</div>
                <div class="circle">Circle</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .rounded { width: 100px; height: 60px; border-radius: 10px; background-color: tomato; margin: 5px; }
                .pill { width: 120px; height: 40px; border-radius: 20px; background-color: steelblue; margin: 5px; }
                .circle { width: 80px; height: 80px; border-radius: 50%; background-color: mediumseagreen; margin: 5px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },
        // ── box-shadow 多值 ──
        TestCase {
            id: "css/box-shadow-multiple".to_string(),
            description: "Multiple box-shadows".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="shadowed">Shadowed box</div>
            </body></html>"#
                .to_string(),
            css: ".shadowed { width: 150px; height: 100px; background-color: white; box-shadow: 0 2px 4px rgba(0,0,0,0.1), 0 4px 8px rgba(0,0,0,0.1), inset 0 1px 0 rgba(255,255,255,0.5); }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — 逻辑属性
        // ═══════════════════════════════════════════════════════════════

        // ── margin-block / padding-inline ──
        TestCase {
            id: "css/logical-properties".to_string(),
            description: "CSS logical properties".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="logical">Logical properties</div>
            </body></html>"#
                .to_string(),
            css: ".logical { margin-block: 10px 20px; padding-inline: 15px 25px; background-color: peru; color: white; }"
                .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — overflow
        // ═══════════════════════════════════════════════════════════════

        // ── overflow 各值 ──
        TestCase {
            id: "css/overflow-values".to_string(),
            description: "CSS overflow property values".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="overflow-hidden">This is a long text that should be clipped when it overflows the container boundary</div>
                <div class="overflow-scroll">Scrollable content that overflows the container</div>
                <div class="overflow-auto">Auto overflow behavior</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .overflow-hidden { width: 150px; height: 30px; overflow: hidden; background-color: #eee; margin: 5px; }
                .overflow-scroll { width: 150px; height: 30px; overflow: scroll; background-color: #ddd; margin: 5px; }
                .overflow-auto { width: 150px; height: 30px; overflow: auto; background-color: #ccc; margin: 5px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — visibility
        // ═══════════════════════════════════════════════════════════════

        // ── visibility: hidden vs collapse ──
        TestCase {
            id: "css/visibility".to_string(),
            description: "CSS visibility property".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="visible">Visible</div>
                <div class="hidden">Hidden (takes space)</div>
                <div class="visible-after">After hidden</div>
            </body></html>"#
                .to_string(),
            css: r#"
                .visible { background-color: green; color: white; height: 30px; }
                .hidden { visibility: hidden; background-color: red; height: 30px; }
                .visible-after { background-color: blue; color: white; height: 30px; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS — z-index 和层叠
        // ═══════════════════════════════════════════════════════════════

        // ── z-index 层叠 ──
        TestCase {
            id: "css/z-index-stacking".to_string(),
            description: "CSS z-index stacking context".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="stack-container">
                    <div class="z1">z-index: 1</div>
                    <div class="z3">z-index: 3</div>
                    <div class="z2">z-index: 2</div>
                </div>
            </body></html>"#
                .to_string(),
            css: r#"
                .stack-container { position: relative; width: 200px; height: 150px; }
                .z1 { position: absolute; top: 0; left: 0; width: 100px; height: 80px; background-color: rgba(255,0,0,0.7); z-index: 1; }
                .z3 { position: absolute; top: 30px; left: 30px; width: 100px; height: 80px; background-color: rgba(0,0,255,0.7); z-index: 3; }
                .z2 { position: absolute; top: 60px; left: 60px; width: 100px; height: 80px; background-color: rgba(0,128,0,0.7); z-index: 2; }
            "#
            .to_string(),
            assertions: vec![
                "dom_has_text".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 滤镜和混合模式
        // ═══════════════════════════════════════════════════════════════

        // ── CSS 滤镜效果 ──
        TestCase {
            id: "css/filter-effects".to_string(),
            description: "CSS filter effects on elements".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="container">
                <div class="box blur">Blur</div>
                <div class="box brightness">Bright</div>
                <div class="box grayscale">Gray</div>
                <div class="box sepia">Sepia</div>
                <div class="box none">Normal</div>
            </div></body></html>"#.to_string(),
            css: r#".container { display: flex; gap: 10px; padding: 20px; background: #f8f9fa; }
                     .box { width: 80px; height: 80px; background: #e74c3c; color: white; display: flex; align-items: center; justify-content: center; border-radius: 8px; }
                     .blur { filter: blur(2px); }
                     .brightness { filter: brightness(1.5); }
                     .grayscale { filter: grayscale(100%); }
                     .sepia { filter: sepia(100%); }
                     .none { filter: none; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── CSS mix-blend-mode ──
        TestCase {
            id: "css/mix-blend-mode".to_string(),
            description: "CSS mix-blend-mode overlay".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="container">
                <div class="bg"></div>
                <div class="overlay">Blended</div>
            </div></body></html>"#.to_string(),
            css: r#".container { position: relative; width: 200px; height: 150px; background: #3498db; margin: 10px; }
                     .bg { position: absolute; top: 20px; left: 20px; width: 120px; height: 80px; background: #e74c3c; }
                     .overlay { position: absolute; top: 40px; left: 60px; width: 120px; height: 80px; background: #2ecc71; mix-blend-mode: multiply; color: white; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 背景高级特性
        // ═══════════════════════════════════════════════════════════════

        // ── 多重背景 ──
        TestCase {
            id: "css/multiple-backgrounds".to_string(),
            description: "Multiple background layers".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="multi-bg">Multiple backgrounds</div>
                <div class="gradient-border">Gradient border effect</div>
            </body></html>"#.to_string(),
            css: r#".multi-bg { width: 300px; height: 200px; margin: 10px; background: linear-gradient(45deg, transparent 40%, rgba(255,0,0,0.3) 40%, rgba(255,0,0,0.3) 60%, transparent 60%), linear-gradient(-45deg, transparent 40%, rgba(0,0,255,0.3) 40%, rgba(0,0,255,0.3) 60%, transparent 60%), #f0f0f0; border-radius: 8px; padding: 20px; color: #333; }
                     .gradient-border { width: 280px; height: 100px; margin: 10px; background: linear-gradient(white, white) padding-box, linear-gradient(135deg, #667eea, #764ba2) border-box; border: 4px solid transparent; border-radius: 8px; padding: 20px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── background-clip 和 origin ──
        TestCase {
            id: "css/background-clip-origin".to_string(),
            description: "background-clip and background-origin".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="box clip-border">border-box</div>
                <div class="box clip-padding">padding-box</div>
                <div class="box clip-content">content-box</div>
                <div class="box clip-text">Text Clip</div>
            </body></html>"#.to_string(),
            css: r#".box { width: 150px; height: 80px; margin: 10px; padding: 20px; border: 8px dashed #adb5bd; display: inline-block; vertical-align: top; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; font-size: 14px; }
                     .clip-border { background-clip: border-box; }
                     .clip-padding { background-clip: padding-box; }
                     .clip-content { background-clip: content-box; }
                     .clip-text { background-clip: text; -webkit-background-clip: text; color: transparent; font-size: 24px; font-weight: bold; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 文本效果
        // ═══════════════════════════════════════════════════════════════

        // ── text-shadow 多效果 ──
        TestCase {
            id: "css/text-shadow-effects".to_string(),
            description: "Multiple text-shadow effects".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <p class="glow">Glowing Text</p>
                <p class="neon">Neon Effect</p>
                <p class="emboss">Embossed</p>
                <p class="retro">Retro Shadow</p>
            </body></html>"#.to_string(),
            css: r#"p { font-size: 32px; margin: 20px; text-align: center; }
                     .glow { color: #fff; text-shadow: 0 0 10px #fff, 0 0 20px #ff0, 0 0 40px #ff0; background: #000; padding: 10px; }
                     .neon { color: #0ff; text-shadow: 0 0 5px #0ff, 0 0 10px #0ff, 0 0 20px #0ff, 0 0 40px #0ff; background: #111; padding: 10px; }
                     .emboss { color: #ccc; text-shadow: -1px -1px 0 #666, 1px 1px 0 #fff; background: #999; padding: 10px; }
                     .retro { color: #e74c3c; text-shadow: 3px 3px 0 #2c3e50, 6px 6px 0 #34495e, 9px 9px 0 #7f8c8d; background: #ecf0f1; padding: 10px; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── box-shadow 高级 ──
        TestCase {
            id: "css/box-shadow-advanced".to_string(),
            description: "Advanced box-shadow effects".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="card">Card with shadow</div>
                <div class="layered">Layered shadows</div>
                <div class="neon-box">Neon border</div>
                <div class="inset">Inset shadow</div>
            </body></html>"#.to_string(),
            css: r#"div { width: 200px; height: 100px; margin: 15px; padding: 15px; border-radius: 8px; display: inline-block; vertical-align: top; }
                     .card { background: white; box-shadow: 0 2px 4px rgba(0,0,0,0.1), 0 8px 16px rgba(0,0,0,0.1); }
                     .layered { background: white; box-shadow: 0 1px 2px rgba(0,0,0,0.07), 0 4px 8px rgba(0,0,0,0.07), 0 16px 32px rgba(0,0,0,0.07); }
                     .neon-box { background: #111; box-shadow: 0 0 5px #0ff, 0 0 10px #0ff, 0 0 20px #0ff, inset 0 0 10px #0ff; color: #0ff; }
                     .inset { background: #f8f9fa; box-shadow: inset 0 2px 4px rgba(0,0,0,0.2); color: #495057; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS 边框和轮廓
        // ═══════════════════════════════════════════════════════════════

        // ── border-radius 组合 ──
        TestCase {
            id: "css/border-radius-shapes".to_string(),
            description: "Various border-radius shapes".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="pill">Pill Shape</div>
                <div class="circle">Circle</div>
                <div class="leaf">Leaf</div>
                <div class="asymmetric">Asymmetric</div>
            </body></html>"#.to_string(),
            css: r#"div { width: 120px; height: 80px; margin: 10px; background: #3498db; color: white; display: inline-flex; align-items: center; justify-content: center; }
                     .pill { border-radius: 40px; }
                     .circle { border-radius: 50%; width: 80px; height: 80px; }
                     .leaf { border-radius: 5px 40px 5px 40px; }
                     .asymmetric { border-radius: 20px 0 20px 0; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },

        // ── border-style 全变体 ──
        TestCase {
            id: "css/border-style-variants".to_string(),
            description: "All border-style variants".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="solid">solid</div>
                <div class="dashed">dashed</div>
                <div class="dotted">dotted</div>
                <div class="double">double</div>
                <div class="groove">groove</div>
                <div class="ridge">ridge</div>
                <div class="inset">inset</div>
                <div class="outset">outset</div>
            </body></html>"#.to_string(),
            css: r#"div { width: 120px; height: 40px; margin: 8px; padding: 8px; display: inline-block; background: #f8f9fa; text-align: center; line-height: 40px; }
                     .solid { border: 3px solid #333; }
                     .dashed { border: 3px dashed #333; }
                     .dotted { border: 3px dotted #333; }
                     .double { border: 5px double #333; }
                     .groove { border: 5px groove #888; }
                     .ridge { border: 5px ridge #888; }
                     .inset { border: 5px inset #888; }
                     .outset { border: 5px outset #888; }"#.to_string(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
            ],
        },
    ]
}
