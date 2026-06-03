//! CSS 扩展测试 + HTML/Form 扩展 + 布局扩展测试。
//!
//! 包含 CSS Grid、Transforms/Transitions/Shadows、Gradient、
//! @Rules/Custom Properties、Advanced Selectors、Additional Properties、
//! HTML/Form Extended、Layout Extended 测试用例。

use super::TestCase;

/// 返回 CSS 扩展、HTML/Form 扩展和布局扩展测试用例。
pub fn css_extended_tests() -> Vec<TestCase> {
    vec![
        //  CSS GRID EXTENDED TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── grid with gap ──
                TestCase {
                    id: "css/grid-gap".to_string(),
                    description: "CSS Grid with gap".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="grid-gap">
                            <div>A</div><div>B</div><div>C</div><div>D</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".grid-gap { display: grid; grid-template-columns: 1fr 1fr; gap: 20px; width: 400px; } .grid-gap div { background-color: lightcoral; height: 50px; }".to_string(),
                    assertions: vec!["layout_has_children".to_string(), "render_completes".to_string()],
                },
                // ── grid with named areas ──
                TestCase {
                    id: "css/grid-named-areas".to_string(),
                    description: "CSS Grid named grid areas".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="named-grid">
                            <div class="hd">H</div>
                            <div class="sd">S</div>
                            <div class="mn">M</div>
                            <div class="ft">F</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".named-grid { display: grid; grid-template-areas: \"hd hd\" \"sd mn\" \"ft ft\"; grid-template-columns: 100px 1fr; grid-template-rows: 40px 1fr 40px; width: 300px; height: 200px; } .hd { grid-area: hd; background-color: #eee; } .sd { grid-area: sd; background-color: #ddd; } .mn { grid-area: mn; background-color: #ccc; } .ft { grid-area: ft; background-color: #bbb; }".to_string(),
                    assertions: vec!["layout_has_children".to_string(), "render_completes".to_string()],
                },
                // ── grid auto-fill/minmax ──
                TestCase {
                    id: "css/grid-auto-fill".to_string(),
                    description: "CSS Grid auto-fill with minmax".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="auto-grid">
                            <div>1</div><div>2</div><div>3</div><div>4</div><div>5</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".auto-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(100px, 1fr)); width: 400px; } .auto-grid div { background-color: thistle; height: 40px; }".to_string(),
                    assertions: vec!["layout_has_children".to_string(), "render_completes".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS TRANSFORMS / TRANSITIONS / SHADOWS
                // ═══════════════════════════════════════════════════════════════

                // ── transform:rotate ──
                TestCase {
                    id: "css/transform-rotate".to_string(),
                    description: "CSS transform rotate".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="rotated">Rotated</div></body></html>"#.to_string(),
                    css: ".rotated { transform: rotate(45deg); background-color: tomato; width: 100px; height: 100px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── transform:scale ──
                TestCase {
                    id: "css/transform-scale".to_string(),
                    description: "CSS transform scale".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="scaled">Scaled</div></body></html>"#.to_string(),
                    css: ".scaled { transform: scale(1.5); background-color: turquoise; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── CSS transition ──
                TestCase {
                    id: "css/transition".to_string(),
                    description: "CSS transition property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="trans">Transition</div></body></html>"#.to_string(),
                    css: ".trans { transition: all 0.3s ease; background-color: slateblue; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── box-shadow ──
                TestCase {
                    id: "css/box-shadow".to_string(),
                    description: "CSS box-shadow property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="shadow">Shadow</div></body></html>"#.to_string(),
                    css: ".shadow { box-shadow: 5px 5px 10px rgba(0,0,0,0.5); background-color: white; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── text-shadow ──
                TestCase {
                    id: "css/text-shadow".to_string(),
                    description: "CSS text-shadow property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><p class="tshadow">Shadow Text</p></body></html>"#.to_string(),
                    css: ".tshadow { text-shadow: 2px 2px 4px rgba(0,0,0,0.3); font-size: 24px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS GRADIENT / BACKGROUND-IMAGE
                // ═══════════════════════════════════════════════════════════════

                // ── linear-gradient ──
                TestCase {
                    id: "css/linear-gradient".to_string(),
                    description: "CSS linear-gradient".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="grad">Gradient</div></body></html>"#.to_string(),
                    css: ".grad { background: linear-gradient(to right, red, blue); width: 200px; height: 100px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── radial-gradient ──
                TestCase {
                    id: "css/radial-gradient".to_string(),
                    description: "CSS radial-gradient".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="rg">Radial</div></body></html>"#.to_string(),
                    css: ".rg { background: radial-gradient(circle, white, black); width: 200px; height: 200px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS @RULES / CUSTOM PROPERTIES
                // ═══════════════════════════════════════════════════════════════

                // ── @supports rule ──
                TestCase {
                    id: "css/supports-rule".to_string(),
                    description: "CSS @supports rule".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="sup">Supports</div></body></html>"#.to_string(),
                    css: "@supports (display: flex) { .sup { display: flex; background-color: teal; width: 200px; height: 50px; } }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── @layer rule ──
                TestCase {
                    id: "css/layer-rule".to_string(),
                    description: "CSS @layer rule".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="layered">Layered</div></body></html>"#.to_string(),
                    css: "@layer base { .layered { background-color: olive; width: 200px; height: 50px; } }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── CSS custom properties with fallback ──
                TestCase {
                    id: "css/var-fallback".to_string(),
                    description: "CSS var() with fallback".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="fb">Fallback</div></body></html>"#.to_string(),
                    css: ".fb { background-color: var(--undefined, #333); width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── CSS custom properties inheritance ──
                TestCase {
                    id: "css/var-inheritance".to_string(),
                    description: "CSS custom properties inheritance".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="parent"><div class="child">Child</div></div>
                    </body></html>"#
                        .to_string(),
                    css: ".parent { --my-bg: #ff6600; } .child { background-color: var(--my-bg); width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS ADVANCED SELECTORS
                // ═══════════════════════════════════════════════════════════════

                // ── :not() selector ──
                TestCase {
                    id: "css/not-selector".to_string(),
                    description: "CSS :not() selector".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="a">A</div>
                        <div class="b">B</div>
                    </body></html>"#
                        .to_string(),
                    css: "div:not(.b) { background-color: crimson; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── :is() selector ──
                TestCase {
                    id: "css/is-selector".to_string(),
                    description: "CSS :is() selector".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <h1>H1</h1>
                        <h2>H2</h2>
                        <p>P</p>
                    </body></html>"#
                        .to_string(),
                    css: ":is(h1, h2) { color: navy; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── :where() selector ──
                TestCase {
                    id: "css/where-selector".to_string(),
                    description: "CSS :where() selector".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="x">X</div>
                        <p class="x">Y</p>
                    </body></html>"#
                        .to_string(),
                    css: ":where(div, p).x { background-color: salmon; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── :has() selector ──
                TestCase {
                    id: "css/has-selector".to_string(),
                    description: "CSS :has() selector".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="container"><p>Has child</p></div>
                        <div class="empty"></div>
                    </body></html>"#
                        .to_string(),
                    css: "div:has(p) { background-color: gold; width: 200px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── :nth-child() selector ──
                TestCase {
                    id: "css/nth-child".to_string(),
                    description: "CSS :nth-child() selector".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <ul>
                            <li>1</li><li>2</li><li>3</li><li>4</li><li>5</li>
                        </ul>
                    </body></html>"#
                        .to_string(),
                    css: "li:nth-child(odd) { background-color: lightblue; } li { height: 20px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS ADDITIONAL PROPERTIES
                // ═══════════════════════════════════════════════════════════════

                // ── cursor property ──
                TestCase {
                    id: "css/cursor".to_string(),
                    description: "CSS cursor property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="cur">Pointer</div></body></html>"#.to_string(),
                    css: ".cur { cursor: pointer; background-color: #eee; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── overflow-wrap ──
                TestCase {
                    id: "css/overflow-wrap".to_string(),
                    description: "CSS overflow-wrap property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><p class="wrap">Superlongwordthatshouldwrap overflow-wrap: break-word</p></body></html>"#.to_string(),
                    css: ".wrap { overflow-wrap: break-word; width: 100px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── text-transform ──
                TestCase {
                    id: "css/text-transform".to_string(),
                    description: "CSS text-transform property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="upper">upper</p>
                        <p class="lower">LOWER</p>
                        <p class="cap">capitalize me</p>
                    </body></html>"#
                        .to_string(),
                    css: ".upper { text-transform: uppercase; } .lower { text-transform: lowercase; } .cap { text-transform: capitalize; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── text-decoration ──
                TestCase {
                    id: "css/text-decoration".to_string(),
                    description: "CSS text-decoration property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="under">Underline</p>
                        <p class="line">Line-through</p>
                    </body></html>"#
                        .to_string(),
                    css: ".under { text-decoration: underline; } .line { text-decoration: line-through; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── border-radius with percentage ──
                TestCase {
                    id: "css/border-radius-percent".to_string(),
                    description: "CSS border-radius with percentage".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="circle">Circle</div></body></html>"#.to_string(),
                    css: ".circle { border-radius: 50%; background-color: crimson; width: 100px; height: 100px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── outline property ──
                TestCase {
                    id: "css/outline".to_string(),
                    description: "CSS outline property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="outlined">Outline</div></body></html>"#.to_string(),
                    css: ".outlined { outline: 2px solid blue; width: 100px; height: 50px; background-color: #eee; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  HTML/FORM EXTENDED TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── form with all input types ──
                TestCase {
                    id: "html/input-types".to_string(),
                    description: "HTML input type variants".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <form>
                            <input type="text" name="t" />
                            <input type="password" name="p" />
                            <input type="email" name="e" />
                            <input type="number" name="n" />
                            <input type="checkbox" name="c" />
                            <input type="radio" name="r" />
                            <input type="submit" value="Go" />
                        </form>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["dom_has_form".to_string(), "dom_has_input".to_string(), "render_completes".to_string()],
                },
                // ── textarea element ──
                TestCase {
                    id: "html/textarea".to_string(),
                    description: "HTML textarea element".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <form>
                            <textarea name="msg" rows="4" cols="30">Hello</textarea>
                            <button type="submit">Send</button>
                        </form>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["dom_has_form".to_string(), "dom_has_button".to_string(), "render_completes".to_string()],
                },
                // ── figure and figcaption ──
                TestCase {
                    id: "html/figure".to_string(),
                    description: "HTML figure and figcaption".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <figure>
                            <img src="photo.jpg" alt="Photo" />
                            <figcaption>A nice photo</figcaption>
                        </figure>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["dom_has_img".to_string(), "dom_has_text".to_string(), "render_completes".to_string()],
                },
                // ── details and summary ──
                TestCase {
                    id: "html/details-summary".to_string(),
                    description: "HTML details and summary elements".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <details>
                            <summary>Click to expand</summary>
                            <p>Hidden content revealed</p>
                        </details>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── table with thead/tbody ──
                TestCase {
                    id: "html/table-full".to_string(),
                    description: "Full HTML table with thead/tbody".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <table>
                            <thead><tr><th>Name</th><th>Age</th></tr></thead>
                            <tbody>
                                <tr><td>Alice</td><td>30</td></tr>
                                <tr><td>Bob</td><td>25</td></tr>
                            </tbody>
                        </table>
                    </body></html>"#
                        .to_string(),
                    css: "table { border-collapse: collapse; width: 200px; } th, td { border: 1px solid black; padding: 4px; }".to_string(),
                    assertions: vec!["dom_has_table".to_string(), "dom_has_text".to_string(), "render_completes".to_string()],
                },
                // ── iframe element ──
                TestCase {
                    id: "html/iframe".to_string(),
                    description: "HTML iframe element".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <iframe src="https://example.com" width="300" height="200"></iframe>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── video element (layout placeholder) ──
                TestCase {
                    id: "html/video-element".to_string(),
                    description: "HTML video element".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <video src="test.mp4" width="320" height="240" controls></video>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── audio element ──
                TestCase {
                    id: "html/audio-element".to_string(),
                    description: "HTML audio element".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <audio src="test.mp3" controls></audio>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── canvas element ──
                TestCase {
                    id: "html/canvas-element".to_string(),
                    description: "HTML canvas element".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <canvas id="c" width="200" height="100"></canvas>
                    </body></html>"#
                        .to_string(),
                    css: "#c { border: 1px solid black; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  LAYOUT EXTENDED TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── aspect-ratio layout ──
                TestCase {
                    id: "layout/aspect-ratio".to_string(),
                    description: "CSS aspect-ratio property".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="ar">16:9</div>
                    </body></html>"#
                        .to_string(),
                    css: ".ar { aspect-ratio: 16 / 9; width: 320px; background-color: mediumseagreen; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── sticky positioning ──
                TestCase {
                    id: "layout/sticky".to_string(),
                    description: "CSS position:sticky".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="scroll-container">
                            <div class="sticky">Sticky</div>
                            <div class="tall">Content</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".scroll-container { height: 300px; overflow: auto; } .sticky { position: sticky; top: 0; background-color: orange; height: 30px; } .tall { height: 600px; background-color: #eee; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── flexbox gap ──
                TestCase {
                    id: "layout/flex-gap".to_string(),
                    description: "Flexbox with gap".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="flex-gap">
                            <div>A</div><div>B</div><div>C</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".flex-gap { display: flex; gap: 10px; width: 300px; } .flex-gap div { background-color: dodgerblue; width: 80px; height: 50px; }".to_string(),
                    assertions: vec!["layout_has_children".to_string(), "render_completes".to_string()],
                },
                // ── nested flexbox ──
                TestCase {
                    id: "layout/nested-flex".to_string(),
                    description: "Nested flex containers".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="outer">
                            <div class="inner">
                                <div class="item">1</div>
                                <div class="item">2</div>
                            </div>
                            <div class="inner">
                                <div class="item">3</div>
                                <div class="item">4</div>
                            </div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".outer { display: flex; width: 400px; } .inner { display: flex; flex-direction: column; flex: 1; } .item { background-color: lightpink; height: 40px; margin: 2px; }".to_string(),
                    assertions: vec!["layout_has_deep_children".to_string(), "render_completes".to_string()],
                },
                // ── inline-block flow ──
                TestCase {
                    id: "layout/inline-block-flow".to_string(),
                    description: "Inline-block elements flowing and wrapping".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="ib1">Block A</div>
                        <div class="ib1">Block B</div>
                        <div class="ib1">Block C</div>
                        <div class="ib1">Block D</div>
                    </body></html>"#
                        .to_string(),
                    css: ".ib1 { display: inline-block; width: 100px; height: 80px; background-color: wheat; margin: 5px; }".to_string(),
                    assertions: vec!["has_multiple_fills".to_string(), "render_completes".to_string()],
                },
                // ── zero-dimension element ──
                TestCase {
                    id: "layout/zero-dimension".to_string(),
                    description: "Zero width/height element".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div class="zero">Invisible</div>
                        <div class="normal">Visible</div>
                    </body></html>"#
                        .to_string(),
                    css: ".zero { width: 0; height: 0; } .normal { width: 100px; height: 50px; background-color: teal; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
    ]
}
