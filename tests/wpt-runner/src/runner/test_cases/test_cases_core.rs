//! 原始测试用例 + CSS 基础测试。
//!
//! 包含 Original 20、CSS Color、Display/Position/Visibility、
//! Text/Font、Box Model、Flexbox、Transform/Variables/Media/Selectors 测试。

use super::TestCase;

/// 返回原始测试用例和 CSS 基础测试用例。
pub fn core_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
                //  ORIGINAL 20 TEST CASES
                // ═══════════════════════════════════════════════════════════════

                // ── 简单文本渲染 ──
                TestCase {
                    id: "text/simple-text".to_string(),
                    description: "Simple text rendering".to_string(),
                    category: "html".to_string(),
                    html: "<html><body>Hello World</body></html>".to_string(),
                    css: String::new(),
                    assertions: vec![
                        "dom_has_body".to_string(),
                        "dom_has_text".to_string(),
                        "render_completes".to_string(),
                        "nonzero_primitives".to_string(),
                    ],
                },
                // ── CSS 颜色属性 ──
                TestCase {
                    id: "css/background-color".to_string(),
                    description: "CSS background-color property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div id="box">Colored</div></body></html>"#.to_string(),
                    css: "#box { background-color: red; width: 200px; height: 100px; }".to_string(),
                    assertions: vec![
                        "dom_has_body".to_string(),
                        "css_background_applied".to_string(),
                        "has_fill_primitives".to_string(),
                    ],
                },
                // ── CSS 尺寸属性 ──
                TestCase {
                    id: "css/width-height".to_string(),
                    description: "CSS width and height properties".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div id="sized">Sized</div></body></html>"#.to_string(),
                    css: "#sized { width: 300px; height: 150px; background-color: blue; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "layout_has_children".to_string()],
                },
                // ── Block 布局 ──
                TestCase {
                    id: "layout/block-basic".to_string(),
                    description: "Block layout with multiple divs".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div>Block 1</div>
                        <div>Block 2</div>
                        <div>Block 3</div>
                    </body></html>"#
                        .to_string(),
                    css: "div { width: 100px; height: 50px; background-color: green; }".to_string(),
                    assertions: vec![
                        "block_layout".to_string(),
                        "has_fill_primitives".to_string(),
                        "dom_has_element".to_string(),
                    ],
                },
                // ── Inline 布局 ──
                TestCase {
                    id: "layout/inline-text".to_string(),
                    description: "Inline text layout".to_string(),
                    category: "layout".to_string(),
                    html: "<html><body><p>Some inline text content here</p></body></html>".to_string(),
                    css: "p { font-size: 16px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── Flexbox 布局 ──
                TestCase {
                    id: "layout/flex-basic".to_string(),
                    description: "Basic flexbox layout".to_string(),
                    category: "layout".to_string(),
                    html: r#"<html><body>
                        <div id="flex-container">
                            <div class="item">A</div>
                            <div class="item">B</div>
                            <div class="item">C</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: r#"
                        #flex-container { display: flex; width: 300px; height: 100px; }
                        .item { flex: 1; background-color: orange; }
                    "#
                    .to_string(),
                    assertions: vec![
                        "flex_layout".to_string(),
                        "render_completes".to_string(),
                        "layout_has_children".to_string(),
                    ],
                },
                // ── 链接元素 ──
                TestCase {
                    id: "dom/link-element".to_string(),
                    description: "Link element exists in DOM".to_string(),
                    category: "dom".to_string(),
                    html: r#"<html><body><a href="https://example.com">Link</a></body></html>"#.to_string(),
                    css: String::new(),
                    assertions: vec!["dom_has_link".to_string(), "dom_has_body".to_string()],
                },
                // ── 表单元素 ──
                TestCase {
                    id: "dom/form-element".to_string(),
                    description: "Form element exists in DOM".to_string(),
                    category: "dom".to_string(),
                    html: r#"<html><body>
                        <form action="/submit">
                            <input type="text" name="q" />
                            <button type="submit">Go</button>
                        </form>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec![
                        "dom_has_form".to_string(),
                        "dom_has_input".to_string(),
                        "dom_has_button".to_string(),
                    ],
                },
                // ── 图片元素 ──
                TestCase {
                    id: "dom/img-element".to_string(),
                    description: "Image element exists in DOM".to_string(),
                    category: "dom".to_string(),
                    html: r#"<html><body><img src="test.png" alt="test" /></body></html>"#.to_string(),
                    css: String::new(),
                    assertions: vec!["dom_has_img".to_string(), "render_completes".to_string()],
                },
                // ── 嵌套元素 ──
                TestCase {
                    id: "html/nested-elements".to_string(),
                    description: "Nested HTML elements".to_string(),
                    category: "html".to_string(),
                    html: r#"<html><body>
                        <div class="outer">
                            <div class="inner">
                                <p>Deep text</p>
                            </div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".outer { width: 400px; height: 300px; background-color: #eee; }".to_string(),
                    assertions: vec![
                        "dom_has_element".to_string(),
                        "has_fill_primitives".to_string(),
                        "layout_has_children".to_string(),
                    ],
                },
                // ── CSS 边框 ──
                TestCase {
                    id: "css/border".to_string(),
                    description: "CSS border properties".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div id="bordered">Border</div></body></html>"#.to_string(),
                    css: "#bordered { border: 2px solid black; width: 200px; height: 100px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── CSS margin/padding ──
                TestCase {
                    id: "css/margin-padding".to_string(),
                    description: "CSS margin and padding".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div id="spaced">Spaced</div></body></html>"#.to_string(),
                    css: "#spaced { margin: 20px; padding: 10px; background-color: yellow; width: 200px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "layout_has_children".to_string()],
                },
                // ── 多种 CSS 颜色 ──
                TestCase {
                    id: "css/multiple-colors".to_string(),
                    description: "Multiple CSS background colors".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="red">R</div>
                        <div class="green">G</div>
                        <div class="blue">B</div>
                    </body></html>"#
                        .to_string(),
                    css: r#"
                        .red { background-color: red; width: 100px; height: 50px; }
                        .green { background-color: green; width: 100px; height: 50px; }
                        .blue { background-color: blue; width: 100px; height: 50px; }
                    "#
                    .to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "nonzero_primitives".to_string()],
                },
                // ── Select 元素 ──
                TestCase {
                    id: "dom/select-element".to_string(),
                    description: "Select element exists in DOM".to_string(),
                    category: "dom".to_string(),
                    html: r#"<html><body>
                        <select name="color">
                            <option value="red">Red</option>
                            <option value="blue">Blue</option>
                        </select>
                    </body></html>"#
                        .to_string(),
                    css: String::new(),
                    assertions: vec!["dom_has_select".to_string(), "dom_has_body".to_string()],
                },
                // ── Table 元素 ──
                TestCase {
                    id: "dom/table-element".to_string(),
                    description: "Table element exists in DOM".to_string(),
                    category: "dom".to_string(),
                    html: r#"<html><body>
                        <table>
                            <tr><td>A</td><td>B</td></tr>
                        </table>
                    </body></html>"#
                        .to_string(),
                    css: "table { background-color: #f0f0f0; width: 200px; }".to_string(),
                    assertions: vec!["dom_has_table".to_string(), "has_fill_primitives".to_string()],
                },
                // ── 空 HTML ──
                TestCase {
                    id: "html/empty".to_string(),
                    description: "Empty HTML document".to_string(),
                    category: "html".to_string(),
                    html: String::new(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── 畸形 HTML ──
                TestCase {
                    id: "html/malformed".to_string(),
                    description: "Malformed HTML document".to_string(),
                    category: "html".to_string(),
                    html: "<div><p>unclosed<span>no closing".to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── Unicode 内容 ──
                TestCase {
                    id: "html/unicode".to_string(),
                    description: "Unicode text content".to_string(),
                    category: "html".to_string(),
                    html: "<html><body>こんにちは世界 Grüße 🌍</body></html>".to_string(),
                    css: String::new(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── CSS 圆角 ──
                TestCase {
                    id: "css/border-radius".to_string(),
                    description: "CSS border-radius property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div id="rounded">Rounded</div></body></html>"#.to_string(),
                    css: "#rounded { border-radius: 10px; background-color: purple; width: 200px; height: 100px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── 带视口的布局验证 ──
                TestCase {
                    id: "layout/viewport".to_string(),
                    description: "Layout viewport is valid".to_string(),
                    category: "layout".to_string(),
                    html: "<html><body><div>Viewport test</div></body></html>".to_string(),
                    css: "div { width: 100%; height: 100px; background-color: teal; }".to_string(),
                    assertions: vec![
                        "layout_valid_viewport".to_string(),
                        "has_fill_primitives".to_string(),
                        "layout_has_children".to_string(),
                    ],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS COLOR TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── CSS hex colors ──
                TestCase {
                    id: "css/color-hex".to_string(),
                    description: "CSS hex color values".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="hex1">Red</div>
                        <div class="hex2">Green</div>
                    </body></html>"#
                        .to_string(),
                    css: ".hex1 { background-color: #ff0000; width: 100px; height: 50px; } .hex2 { background-color: #00ff00; width: 100px; height: 50px; }".to_string(),
                    assertions: vec![
                        "has_multiple_fills".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── CSS rgb() colors ──
                TestCase {
                    id: "css/color-rgb".to_string(),
                    description: "CSS rgb() color values".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="rgb1">Coral</div>
                        <div class="rgb2">Gold</div>
                    </body></html>"#
                        .to_string(),
                    css: ".rgb1 { background-color: rgb(255,127,80); width: 100px; height: 50px; } .rgb2 { background-color: rgb(255,215,0); width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── CSS hsl() colors ──
                TestCase {
                    id: "css/color-hsl".to_string(),
                    description: "CSS hsl() color values".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="hsl1">HSL Color</div></body></html>"#.to_string(),
                    css: ".hsl1 { background-color: hsl(120, 100%, 50%); width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── CSS named colors ──
                TestCase {
                    id: "css/color-named".to_string(),
                    description: "CSS named color values".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="n1">Crimson</div>
                        <div class="n2">Teal</div>
                    </body></html>"#
                        .to_string(),
                    css: ".n1 { background-color: crimson; width: 100px; height: 50px; } .n2 { background-color: teal; width: 100px; height: 50px; }".to_string(),
                    assertions: vec![
                        "has_fill_primitives".to_string(),
                        "render_completes".to_string(),
                    ],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS DISPLAY / POSITION / VISIBILITY TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── display:none ──
                TestCase {
                    id: "css/display-none".to_string(),
                    description: "CSS display:none property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="visible">Visible</div>
                        <div class="hidden">Hidden</div>
                    </body></html>"#
                        .to_string(),
                    css: ".visible { background-color: red; width: 100px; height: 50px; } .hidden { display: none; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── display:inline-block ──
                TestCase {
                    id: "css/display-inline-block".to_string(),
                    description: "CSS display:inline-block".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="ib">A</div>
                        <div class="ib">B</div>
                    </body></html>"#
                        .to_string(),
                    css: ".ib { display: inline-block; background-color: steelblue; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── position:absolute ──
                TestCase {
                    id: "css/position-absolute".to_string(),
                    description: "CSS position:absolute".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="container"><div class="abs">Absolute</div></div>
                    </body></html>"#
                        .to_string(),
                    css: ".container { position: relative; width: 300px; height: 200px; background-color: #eee; } .abs { position: absolute; top: 10px; left: 20px; background-color: red; width: 100px; height: 50px; }".to_string(),
                    assertions: vec![
                        "has_fill_primitives".to_string(),
                        "has_multiple_fills".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── position:relative ──
                TestCase {
                    id: "css/position-relative".to_string(),
                    description: "CSS position:relative".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="rel">Shifted</div>
                    </body></html>"#
                        .to_string(),
                    css: ".rel { position: relative; top: 20px; left: 30px; background-color: orange; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── position:fixed ──
                TestCase {
                    id: "css/position-fixed".to_string(),
                    description: "CSS position:fixed".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="fixed">Fixed</div>
                    </body></html>"#
                        .to_string(),
                    css: ".fixed { position: fixed; top: 0; left: 0; background-color: navy; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── overflow:hidden ──
                TestCase {
                    id: "css/overflow-hidden".to_string(),
                    description: "CSS overflow:hidden".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="clip">Content that overflows the box</div>
                    </body></html>"#
                        .to_string(),
                    css: ".clip { overflow: hidden; width: 100px; height: 50px; background-color: gray; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── z-index ──
                TestCase {
                    id: "css/z-index".to_string(),
                    description: "CSS z-index stacking".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="z1">Bottom</div>
                        <div class="z2">Top</div>
                    </body></html>"#
                        .to_string(),
                    css: ".z1 { position: absolute; z-index: 1; background-color: red; width: 100px; height: 100px; } .z2 { position: absolute; z-index: 2; background-color: blue; width: 80px; height: 80px; }".to_string(),
                    assertions: vec![
                        "has_fill_primitives".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── opacity ──
                TestCase {
                    id: "css/opacity".to_string(),
                    description: "CSS opacity property".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="faded">Half</div></body></html>"#.to_string(),
                    css: ".faded { opacity: 0.5; background-color: red; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── visibility:hidden ──
                TestCase {
                    id: "css/visibility-hidden".to_string(),
                    description: "CSS visibility:hidden".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="seen">Visible</div>
                        <div class="unseen">Invisible</div>
                    </body></html>"#
                        .to_string(),
                    css: ".seen { background-color: green; width: 100px; height: 50px; } .unseen { visibility: hidden; background-color: red; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS TEXT / FONT TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── text-align ──
                TestCase {
                    id: "css/text-align".to_string(),
                    description: "CSS text alignment".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="center">Centered</p>
                        <p class="right">Right</p>
                    </body></html>"#
                        .to_string(),
                    css: ".center { text-align: center; } .right { text-align: right; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── font-size ──
                TestCase {
                    id: "css/font-size".to_string(),
                    description: "CSS font sizes".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="big">Large</p>
                        <p class="tiny">Small</p>
                    </body></html>"#
                        .to_string(),
                    css: ".big { font-size: 32px; } .tiny { font-size: 10px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── font-weight ──
                TestCase {
                    id: "css/font-weight".to_string(),
                    description: "CSS font weights".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <p class="bold">Bold</p>
                        <p class="light">Light</p>
                    </body></html>"#
                        .to_string(),
                    css: ".bold { font-weight: bold; } .light { font-weight: 200; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS BOX MODEL TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── box-sizing:border-box ──
                TestCase {
                    id: "css/box-sizing".to_string(),
                    description: "CSS box-sizing:border-box".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="box">Box</div></body></html>"#.to_string(),
                    css: ".box { box-sizing: border-box; border: 10px solid black; width: 200px; height: 100px; background-color: cyan; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── max-width ──
                TestCase {
                    id: "css/max-width".to_string(),
                    description: "CSS max-width constraint".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="maxw">Constrained</div></body></html>"#.to_string(),
                    css: ".maxw { max-width: 300px; width: 100%; background-color: coral; height: 50px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── min-height ──
                TestCase {
                    id: "css/min-height".to_string(),
                    description: "CSS min-height constraint".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="minh">Minimum</div></body></html>"#.to_string(),
                    css: ".minh { min-height: 200px; background-color: gold; width: 200px; }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "layout_has_children".to_string()],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS FLEXBOX TESTS
                // ═══════════════════════════════════════════════════════════════

                // ── flex-wrap ──
                TestCase {
                    id: "css/flex-wrap".to_string(),
                    description: "CSS flex-wrap".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="wrap-container">
                            <div class="w-item">1</div>
                            <div class="w-item">2</div>
                            <div class="w-item">3</div>
                            <div class="w-item">4</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".wrap-container { display: flex; flex-wrap: wrap; width: 200px; } .w-item { width: 80px; height: 50px; background-color: salmon; }".to_string(),
                    assertions: vec![
                        "layout_has_children".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── flex-direction:column ──
                TestCase {
                    id: "css/flex-direction".to_string(),
                    description: "CSS flex-direction:column".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="col-container">
                            <div class="c-item">A</div>
                            <div class="c-item">B</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".col-container { display: flex; flex-direction: column; width: 200px; height: 200px; } .c-item { background-color: mediumpurple; height: 80px; }".to_string(),
                    assertions: vec![
                        "layout_has_children".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── justify-content ──
                TestCase {
                    id: "css/justify-content".to_string(),
                    description: "CSS justify-content:center".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="jc">
                            <div class="jc-item">X</div>
                            <div class="jc-item">Y</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".jc { display: flex; justify-content: center; width: 300px; height: 100px; background-color: #ddd; } .jc-item { width: 50px; height: 50px; background-color: tomato; }".to_string(),
                    assertions: vec![
                        "has_fill_primitives".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── align-items ──
                TestCase {
                    id: "css/align-items".to_string(),
                    description: "CSS align-items:center".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="ai">
                            <div class="ai-item">X</div>
                        </div>
                    </body></html>"#
                        .to_string(),
                    css: ".ai { display: flex; align-items: center; width: 300px; height: 200px; background-color: #ccc; } .ai-item { width: 50px; height: 50px; background-color: olive; }".to_string(),
                    assertions: vec![
                        "has_fill_primitives".to_string(),
                        "render_completes".to_string(),
                    ],
                },

                // ═══════════════════════════════════════════════════════════════
                //  CSS TRANSFORM / VARIABLES / MEDIA / SELECTORS
                // ═══════════════════════════════════════════════════════════════

                // ── transform:translate ──
                TestCase {
                    id: "css/transform-translate".to_string(),
                    description: "CSS transform translate".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="shifted">Moved</div></body></html>"#.to_string(),
                    css: ".shifted { transform: translate(50px, 30px); background-color: peru; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── CSS custom properties (variables) ──
                TestCase {
                    id: "css/css-variables".to_string(),
                    description: "CSS custom properties".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="var-test">Variable</div></body></html>"#.to_string(),
                    css: ":root { --main-color: #ff6600; } .var-test { background-color: var(--main-color); width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── @media rules ──
                TestCase {
                    id: "css/media-query".to_string(),
                    description: "CSS @media rules".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="mq">Media</div></body></html>"#.to_string(),
                    css: ".mq { width: 200px; height: 100px; background-color: pink; } @media (min-width: 500px) { .mq { background-color: violet; } }".to_string(),
                    assertions: vec!["has_fill_primitives".to_string(), "render_completes".to_string()],
                },
                // ── nested selectors ──
                TestCase {
                    id: "css/nested-selectors".to_string(),
                    description: "CSS complex selectors".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="outer"><p class="inner">Text</p></div>
                    </body></html>"#
                        .to_string(),
                    css: "div.outer > p.inner { background-color: khaki; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── class selector ──
                TestCase {
                    id: "css/class-selector".to_string(),
                    description: "CSS .class selectors".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="a">A</div>
                        <div class="b">B</div>
                    </body></html>"#
                        .to_string(),
                    css: ".a { background-color: red; width: 100px; height: 50px; } .b { background-color: blue; width: 100px; height: 50px; }".to_string(),
                    assertions: vec![
                        "has_multiple_fills".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── id selector ──
                TestCase {
                    id: "css/id-selector".to_string(),
                    description: "CSS #id selectors".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div id="foo">Foo</div>
                        <div id="bar">Bar</div>
                    </body></html>"#
                        .to_string(),
                    css: "#foo { background-color: red; width: 100px; height: 50px; } #bar { background-color: blue; width: 100px; height: 50px; }".to_string(),
                    assertions: vec![
                        "has_multiple_fills".to_string(),
                        "render_completes".to_string(),
                    ],
                },
                // ── attribute selector ──
                TestCase {
                    id: "css/attribute-selector".to_string(),
                    description: "CSS [attr] selectors".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div data-active="yes">Active</div>
                        <div data-active="no">Inactive</div>
                    </body></html>"#
                        .to_string(),
                    css: "[data-active=\"yes\"] { background-color: green; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── descendant selector ──
                TestCase {
                    id: "css/descendant-selector".to_string(),
                    description: "CSS descendant combinator".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div class="outer"><p>Descendant</p></div>
                    </body></html>"#
                        .to_string(),
                    css: "div p { background-color: lavender; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── child selector (>) ──
                TestCase {
                    id: "css/child-selector".to_string(),
                    description: "CSS child combinator >".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body>
                        <div><p>Direct child</p></div>
                    </body></html>"#
                        .to_string(),
                    css: "div > p { font-size: 20px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
                },
                // ── :hover pseudo-class ──
                TestCase {
                    id: "css/pseudo-class-hover".to_string(),
                    description: "CSS :hover pseudo-class parsing".to_string(),
                    category: "css".to_string(),
                    html: "<html><body><a href=\"#\" class=\"hover-link\">Hover me</a></body></html>"
                        .to_string(),
                    css: ".hover-link:hover { color: red; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
                // ── multiple classes ──
                TestCase {
                    id: "css/multiple-classes".to_string(),
                    description: "Multiple class names on one element".to_string(),
                    category: "css".to_string(),
                    html: r#"<html><body><div class="bold red bg">Multi</div></body></html>"#.to_string(),
                    css: ".bold { font-weight: bold; } .red { color: red; } .bg { background-color: pink; width: 100px; height: 50px; }".to_string(),
                    assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
                },
    ]
}
