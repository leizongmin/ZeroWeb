//! 内置 WPT 测试用例定义。
//!
//! 包含所有 Web Platform Tests 测试用例，按类别分组：
//! HTML、CSS、布局、DOM、错误恢复等。

use super::TestCase;

/// 返回所有内置测试用例。
pub fn builtin_tests() -> Vec<TestCase> {
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

        // ═══════════════════════════════════════════════════════════════
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

        // ═══════════════════════════════════════════════════════════════
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
