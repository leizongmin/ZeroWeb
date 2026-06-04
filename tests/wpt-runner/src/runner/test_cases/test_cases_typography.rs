//! CSS 排版和高级视觉效果测试。
//!
//! 覆盖字体排版、文本渲染、高级 CSS 视觉效果、
//! 响应式布局、CSS 变量、Container Queries、高级选择器等。

use super::TestCase;

/// 返回排版和高级视觉效果测试用例。
pub fn typography_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        // 字体属性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/font-family-stack".into(),
            description: "多字体栈渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="font-family: Georgia, 'Times New Roman', serif">Serif text</p>
            <p style="font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif">Sans text</p>
            <p style="font-family: 'Courier New', Courier, monospace">Mono text</p>
            <p style="font-family: cursive">Cursive text</p>
            <p style="font-family: fantasy">Fantasy text</p>
            <p style="font-family: system-ui, -apple-system, sans-serif">System font</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/font-sizes".into(),
            description: "各种字号渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="font-size: 12px">12px text</p>
            <p style="font-size: 14px">14px text</p>
            <p style="font-size: 16px">16px text</p>
            <p style="font-size: 24px">24px text</p>
            <p style="font-size: 32px">32px text</p>
            <p style="font-size: 0.8em">0.8em text</p>
            <p style="font-size: 1.5em">1.5em text</p>
            <p style="font-size: 80%">80% text</p>
            <p style="font-size: 150%">150% text</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/font-weights".into(),
            description: "各种字重渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="font-weight: 100">Thin</p>
            <p style="font-weight: 300">Light</p>
            <p style="font-weight: 400">Normal</p>
            <p style="font-weight: 500">Medium</p>
            <p style="font-weight: 700">Bold</p>
            <p style="font-weight: 900">Black</p>
            <p style="font-weight: bold">Bold keyword</p>
            <p style="font-weight: normal">Normal keyword</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/font-shorthand".into(),
            description: "font 简写属性渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="font: italic bold 16px/1.5 Georgia, serif">Italic bold serif</p>
            <p style="font: normal normal 14px/1.2 Arial, sans-serif">Normal sans</p>
            <p style="font: small-caps 700 20px/2 monospace">Small-caps mono</p>
            <p style="font: 12px sans-serif">Size and family only</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 文本属性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/text-align".into(),
            description: "text-align 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="text-align: left">Left aligned text</p>
            <p style="text-align: center">Center aligned text</p>
            <p style="text-align: right">Right aligned text</p>
            <p style="text-align: justify">Justified text that should fill the entire width of its container when it is long enough to wrap to multiple lines.</p>
            <p style="text-align: start">Start aligned</p>
            <p style="text-align: end">End aligned</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/line-height".into(),
            description: "line-height 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="line-height: 1">Tight line height. This text should have minimal spacing between lines.</p>
            <p style="line-height: 1.5">Normal line height. This text should have comfortable spacing between lines.</p>
            <p style="line-height: 2">Double line height. This text should have generous spacing between lines.</p>
            <p style="line-height: 24px">Fixed line height at 24px.</p>
            <p style="line-height: 150%">Percentage line height at 150%.</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/text-decoration".into(),
            description: "text-decoration 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="text-decoration: underline">Underlined text</p>
            <p style="text-decoration: overline">Overlined text</p>
            <p style="text-decoration: line-through">Strikethrough text</p>
            <p style="text-decoration: underline overline">Under and overlined</p>
            <p style="text-decoration: underline wavy red">Wavy red underline</p>
            <p style="text-decoration: underline dotted blue">Dotted blue underline</p>
            <p style="text-decoration: underline dashed">Dashed underline</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/text-transform".into(),
            description: "text-transform 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="text-transform: uppercase">this should be uppercase</p>
            <p style="text-transform: lowercase">THIS SHOULD BE LOWERCASE</p>
            <p style="text-transform: capitalize">each word should start with capital</p>
            <p style="text-transform: none">No Transformation Applied</p>
            <p style="text-transform: full-width">Full Width Text</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/letter-word-spacing".into(),
            description: "letter-spacing 和 word-spacing 渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="letter-spacing: 0.1em">Widely spaced letters</p>
            <p style="letter-spacing: -0.05em">Tightly spaced letters</p>
            <p style="letter-spacing: 2px">2px letter spacing</p>
            <p style="word-spacing: 0.5em">Words with more space</p>
            <p style="word-spacing: -0.1em">Words with less space</p>
            <p style="word-spacing: 10px">Words with 10px spacing</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/white-space".into(),
            description: "white-space 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="white-space: normal">Normal   whitespace    handling</p>
            <p style="white-space: nowrap">No wrapping allowed for this text content</p>
            <p style="white-space: pre">Pre     formatted    text</p>
            <p style="white-space: pre-wrap">Pre     wrapped    text with long content that should wrap to the next line</p>
            <p style="white-space: pre-line">Pre     line    text</p>
            <p style="white-space: break-spaces">Break   spaces    text</p>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 颜色和背景
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/colors-named".into(),
            description: "命名颜色渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="color: red">Red text</p>
            <p style="color: dodgerblue">Dodgerblue text</p>
            <p style="color: mediumseagreen">Green text</p>
            <p style="color: orange">Orange text</p>
            <p style="color: rebeccapurple">Rebeccapurple text</p>
            <p style="color: coral">Coral text</p>
            <p style="color: teal">Teal text</p>
            <p style="color: crimson">Crimson text</p>
            <div style="background: gold; padding: 10px">Gold background</div>
            <div style="background: lightblue; color: navy; padding: 10px">Lightblue + navy</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/colors-functional".into(),
            description: "函数颜色渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="color: rgb(255, 0, 0)">RGB red</p>
            <p style="color: rgba(0, 128, 0, 0.8)">RGBA green</p>
            <p style="color: hsl(240, 100%, 50%)">HSL blue</p>
            <p style="color: hsla(60, 100%, 50%, 0.9)">HSLA yellow</p>
            <div style="background: rgb(100, 149, 237); padding: 10px">Cornflower blue bg</div>
            <div style="background: hsl(0, 0%, 90%); padding: 10px">Light gray bg</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/colors-hex".into(),
            description: "十六进制颜色渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <p style="color: #ff0000">Full hex red</p>
            <p style="color: #00ff00">Full hex green</p>
            <p style="color: #0000ff">Full hex blue</p>
            <p style="color: #f00">Short hex red</p>
            <p style="color: #0f0">Short hex green</p>
            <p style="color: #00f">Short hex blue</p>
            <div style="background: #333; color: #eee; padding: 10px">Dark bg light text</div>
            <div style="background: #ff660088; padding: 10px">8-digit hex with alpha</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 边框和圆角
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/border-styles".into(),
            description: "各种边框样式渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="border: 2px solid black; margin: 5px; padding: 5px">Solid border</div>
            <div style="border: 2px dashed black; margin: 5px; padding: 5px">Dashed border</div>
            <div style="border: 2px dotted black; margin: 5px; padding: 5px">Dotted border</div>
            <div style="border: 4px double black; margin: 5px; padding: 5px">Double border</div>
            <div style="border: 3px groove gray; margin: 5px; padding: 5px">Groove border</div>
            <div style="border: 3px ridge gray; margin: 5px; padding: 5px">Ridge border</div>
            <div style="border: 3px inset gray; margin: 5px; padding: 5px">Inset border</div>
            <div style="border: 3px outset gray; margin: 5px; padding: 5px">Outset border</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "layout_box_count_ge:8".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/border-radius".into(),
            description: "圆角边框渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="border: 2px solid black; border-radius: 10px; padding: 10px; margin: 5px">Rounded corners</div>
            <div style="border: 2px solid black; border-radius: 50%; width: 100px; height: 100px; margin: 5px">Circle</div>
            <div style="border: 2px solid black; border-radius: 10px 0 10px 0; padding: 10px; margin: 5px">Asymmetric radius</div>
            <div style="border: 2px solid black; border-radius: 20px / 10px; padding: 10px; margin: 5px">Elliptical radius</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 阴影
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/box-shadow".into(),
            description: "box-shadow 渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="box-shadow: 5px 5px 10px rgba(0,0,0,0.3); padding: 20px; margin: 20px; background: white">Simple shadow</div>
            <div style="box-shadow: 0 0 20px rgba(0,0,255,0.5); padding: 20px; margin: 20px; background: white">Spread shadow</div>
            <div style="box-shadow: inset 0 0 10px rgba(0,0,0,0.3); padding: 20px; margin: 20px; background: white">Inset shadow</div>
            <div style="box-shadow: 2px 2px 5px black, -2px -2px 5px gray; padding: 20px; margin: 20px; background: white">Multiple shadows</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "shadow_count_ge:1".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 渐变
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/gradients-linear".into(),
            description: "linear-gradient 渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="background: linear-gradient(to right, red, blue); height: 50px; margin: 5px"></div>
            <div style="background: linear-gradient(135deg, #ff0000, #00ff00, #0000ff); height: 50px; margin: 5px"></div>
            <div style="background: linear-gradient(to bottom, white, black); height: 50px; margin: 5px"></div>
            <div style="background: linear-gradient(90deg, rgba(255,0,0,0), rgba(255,0,0,1)); height: 50px; margin: 5px"></div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "gradient_count_ge:1".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/gradients-radial".into(),
            description: "radial-gradient 渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="background: radial-gradient(circle, red, blue); height: 100px; margin: 5px"></div>
            <div style="background: radial-gradient(ellipse at top left, red, blue); height: 100px; margin: 5px"></div>
            <div style="background: radial-gradient(circle closest-side, red, blue); height: 100px; margin: 5px"></div>
            <div style="background: radial-gradient(circle farthest-corner, #e66465, #9198e5); height: 100px; margin: 5px"></div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "gradient_count_ge:1".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // opacity 和 visibility
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/opacity".into(),
            description: "opacity 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="opacity: 1.0; background: blue; color: white; padding: 10px; margin: 5px">100% opacity</div>
            <div style="opacity: 0.75; background: blue; color: white; padding: 10px; margin: 5px">75% opacity</div>
            <div style="opacity: 0.5; background: blue; color: white; padding: 10px; margin: 5px">50% opacity</div>
            <div style="opacity: 0.25; background: blue; color: white; padding: 10px; margin: 5px">25% opacity</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/visibility".into(),
            description: "visibility 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="visibility: visible; background: green; padding: 10px; margin: 5px">Visible</div>
            <div style="visibility: hidden; background: red; padding: 10px; margin: 5px">Hidden (takes space)</div>
            <div style="visibility: visible; background: blue; padding: 10px; margin: 5px">Visible after hidden</div>
            <div style="visibility: collapse; background: yellow; padding: 10px; margin: 5px">Collapsed</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "layout_box_count_ge:4".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // overflow
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/overflow".into(),
            description: "overflow 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="width: 100px; height: 50px; overflow: visible; border: 1px solid red; margin: 5px">
                This text overflows the container and should be visible.
            </div>
            <div style="width: 100px; height: 50px; overflow: hidden; border: 1px solid blue; margin: 5px">
                This text is clipped by the container and should be hidden.
            </div>
            <div style="width: 100px; height: 50px; overflow: scroll; border: 1px solid green; margin: 5px">
                This container has scrollbars.
            </div>
            <div style="width: 100px; height: 50px; overflow: auto; border: 1px solid purple; margin: 5px">
                Auto overflow behavior.
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "layout_box_count_ge:4".into(), "has_fill_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // CSS 变量和自定义属性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/css-variables".into(),
            description: "CSS 自定义属性渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div class="card">Card with CSS variables</div>
            </body></html>"#.into(),
            css: r#"
                :root {
                    --primary: #0066cc;
                    --bg: #f5f5f5;
                    --radius: 8px;
                    --spacing: 16px;
                }
                .card {
                    background: var(--bg);
                    color: var(--primary);
                    border: 1px solid var(--primary);
                    border-radius: var(--radius);
                    padding: var(--spacing);
                    margin: 10px;
                }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/css-variables-fallback".into(),
            description: "CSS 自定义属性回退值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div class="item">Item with fallback</div>
            </body></html>"#.into(),
            css: r#"
                .item {
                    color: var(--undefined-color, #333);
                    background: var(--undefined-bg, white);
                    padding: var(--undefined-pad, 10px);
                    border: var(--undefined-border, 1px solid #ccc);
                    margin: 10px;
                }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/css-variables-calc".into(),
            description: "CSS 变量 + calc() 渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div class="fluid">Fluid typography with calc and variables</div>
            </body></html>"#.into(),
            css: r#"
                :root {
                    --base-size: 16px;
                    --scale: 1.5;
                }
                .fluid {
                    font-size: calc(var(--base-size) * var(--scale));
                    padding: calc(10px + 2%);
                    margin: 10px;
                    background: #eee;
                }
            "#.into(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面：博客文章
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/composite/blog-post".into(),
            description: "博客文章排版渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <article class="blog-post">
                <h1>The Art of Typography</h1>
                <p class="meta">Published on <time datetime="2026-06-01">June 1, 2026</time> by <em>Jane Doe</em></p>
                <p class="intro">Typography is the art and technique of arranging type to make written language legible, readable and appealing.</p>
                <h2>History</h2>
                <p>The <strong>history of typography</strong> dates back to ancient civilizations. From <em>clay tablets</em> to <em>movable type</em>, the evolution of text presentation has shaped human communication.</p>
                <blockquote>
                    <p>"Typography is what language looks like." — Ellen Lupton</p>
                </blockquote>
                <h2>Modern Practices</h2>
                <p>Key principles include:</p>
                <ul>
                    <li><strong>Hierarchy</strong> — using size and weight to guide the reader</li>
                    <li><strong>Contrast</strong> — creating visual interest through difference</li>
                    <li><strong>Spacing</strong> — giving text room to breathe</li>
                    <li><strong>Alignment</strong> — maintaining visual order</li>
                </ul>
                <h2>Code Example</h2>
                <pre><code>body {
    font-family: system-ui, sans-serif;
    line-height: 1.6;
    max-width: 65ch;
}</code></pre>
                <h2>Conclusion</h2>
                <p>Good typography is invisible — it enhances the reading experience without drawing attention to itself.</p>
            </article>
            </body></html>"#.into(),
            css: r#"
                body { font-family: Georgia, serif; max-width: 700px; margin: 0 auto; padding: 20px; color: #333; }
                h1 { font-size: 2em; line-height: 1.2; margin-bottom: 0.3em; }
                h2 { font-size: 1.5em; margin-top: 1.5em; color: #222; }
                .meta { color: #666; font-size: 0.9em; margin-bottom: 1.5em; }
                .intro { font-size: 1.1em; line-height: 1.7; }
                blockquote { border-left: 4px solid #ccc; margin: 1em 0; padding: 0.5em 1em; color: #555; background: #f9f9f9; }
                pre { background: #2d2d2d; color: #f8f8f2; padding: 1em; overflow-x: auto; border-radius: 4px; }
                code { font-family: 'Fira Code', monospace; }
                ul { padding-left: 1.5em; }
                li { margin: 0.3em 0; line-height: 1.5; }
                strong { font-weight: 700; }
                em { font-style: italic; }
            "#.into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_article".into(),
                "dom_has_element:blockquote".into(),
                "dom_has_element:pre".into(),
                "dom_has_element:code".into(),
                "has_fill_primitives".into(),
                "has_glyph_primitives".into(),
                "layout_has_children".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面：定价卡片
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/composite/pricing-cards".into(),
            description: "定价卡片渐变阴影渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div class="pricing">
                <div class="plan">
                    <h3>Basic</h3>
                    <div class="price">$9<span>/mo</span></div>
                    <ul>
                        <li>5 Projects</li>
                        <li>10GB Storage</li>
                        <li>Email Support</li>
                    </ul>
                    <button>Choose Plan</button>
                </div>
                <div class="plan featured">
                    <h3>Pro</h3>
                    <div class="price">$29<span>/mo</span></div>
                    <ul>
                        <li>Unlimited Projects</li>
                        <li>100GB Storage</li>
                        <li>Priority Support</li>
                    </ul>
                    <button>Choose Plan</button>
                </div>
                <div class="plan">
                    <h3>Enterprise</h3>
                    <div class="price">$99<span>/mo</span></div>
                    <ul>
                        <li>Unlimited Everything</li>
                        <li>1TB Storage</li>
                        <li>24/7 Support</li>
                    </ul>
                    <button>Contact Us</button>
                </div>
            </div>
            </body></html>"#.into(),
            css: r#"
                .pricing { display: flex; gap: 20px; padding: 40px; align-items: stretch; }
                .plan {
                    flex: 1; border: 1px solid #ddd; border-radius: 12px;
                    padding: 30px; text-align: center; background: white;
                }
                .plan.featured {
                    border-color: #0066cc;
                    box-shadow: 0 10px 30px rgba(0, 102, 204, 0.2);
                    background: linear-gradient(to bottom, #f0f7ff, white);
                }
                .plan h3 { margin: 0 0 10px; color: #333; }
                .price { font-size: 2.5em; font-weight: bold; color: #0066cc; margin: 15px 0; }
                .price span { font-size: 0.4em; color: #666; }
                .plan ul { list-style: none; padding: 0; margin: 20px 0; }
                .plan li { padding: 8px 0; border-bottom: 1px solid #eee; }
                .plan button { background: #0066cc; color: white; border: none; padding: 12px 30px; border-radius: 6px; font-size: 1em; }
                .featured button { background: linear-gradient(135deg, #0066cc, #004499); }
            "#.into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_button".into(),
                "dom_has_list".into(),
                "has_fill_primitives".into(),
                "has_glyph_primitives".into(),
                "shadow_count_ge:1".into(),
                "gradient_count_ge:1".into(),
                "layout_has_children".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // CSS box-sizing
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/box-sizing".into(),
            description: "box-sizing 渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="width: 200px; padding: 20px; border: 5px solid red; box-sizing: content-box; margin: 5px">Content-box (total width: 250px)</div>
            <div style="width: 200px; padding: 20px; border: 5px solid blue; box-sizing: border-box; margin: 5px">Border-box (total width: 200px)</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "layout_box_count_ge:2".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // display 变体
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/display-variants".into(),
            description: "display 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="display: block; background: #eee; padding: 5px; margin: 5px">Block</div>
            <span style="display: inline; background: #ddd; padding: 2px">Inline 1</span>
            <span style="display: inline; background: #ccc; padding: 2px">Inline 2</span>
            <div style="display: inline-block; background: #bbb; padding: 5px; width: 100px">Inline-block</div>
            <div style="display: none; background: red">This is hidden</div>
            <div style="display: flex; gap: 10px; margin: 5px">
                <div style="flex: 1; background: #aaf">Flex 1</div>
                <div style="flex: 2; background: #afa">Flex 2</div>
            </div>
            <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 5px; margin: 5px">
                <div style="background: #ffa">Grid 1</div>
                <div style="background: #faf">Grid 2</div>
                <div style="background: #aff">Grid 3</div>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // CSS calc()
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/calc".into(),
            description: "calc() 函数渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="width: calc(100% - 40px); background: #eee; padding: 10px; margin: 5px">calc(100% - 40px)</div>
            <div style="width: calc(50% + 20px); background: #ddd; padding: 10px; margin: 5px">calc(50% + 20px)</div>
            <div style="font-size: calc(16px + 0.5vw)">Fluid font size</div>
            <div style="height: calc(100px / 2); background: #ccc; margin: 5px">Height calc(100px / 2)</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // position 变体
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/position-variants".into(),
            description: "position 各种值渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="position: relative; height: 100px; background: #f0f0f0; margin: 10px">
                <div style="position: absolute; top: 10px; right: 10px; background: #ffcccc; padding: 5px">Absolute</div>
                <div style="position: relative; top: 20px; left: 20px; background: #ccffcc; padding: 5px">Relative offset</div>
            </div>
            <div style="position: sticky; top: 0; background: #ccccff; padding: 10px">Sticky header</div>
            <div style="height: 200px; background: #eee; margin: 5px">Spacer</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // z-index 层叠
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/z-index".into(),
            description: "z-index 层叠渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="position: relative; height: 150px; margin: 10px">
                <div style="position: absolute; top: 0; left: 0; width: 100px; height: 100px; background: rgba(255,0,0,0.7); z-index: 3">z:3</div>
                <div style="position: absolute; top: 20px; left: 20px; width: 100px; height: 100px; background: rgba(0,255,0,0.7); z-index: 2">z:2</div>
                <div style="position: absolute; top: 40px; left: 40px; width: 100px; height: 100px; background: rgba(0,0,255,0.7); z-index: 1">z:1</div>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // CSS 选择器
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/selectors-complex".into(),
            description: "复杂选择器渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div class="container">
                <ul class="list">
                    <li class="item active">Active item</li>
                    <li class="item">Normal item</li>
                    <li class="item">Normal item</li>
                </ul>
                <div id="special" data-type="highlight" lang="en">
                    <p class="text primary">Primary text</p>
                    <p class="text secondary">Secondary text</p>
                </div>
                <div class="box">
                    <p>Nested paragraph</p>
                    <span>Inline span</span>
                </div>
            </div>
            </body></html>"#.into(),
            css: r#"
                /* 类选择器 */
                .container { padding: 20px; }
                /* 后代选择器 */
                .container p { margin: 5px 0; }
                /* 子选择器 */
                .container > .box { border: 1px solid #ccc; padding: 10px; }
                /* 属性选择器 */
                [data-type="highlight"] { background: #fff3cd; padding: 10px; }
                /* ID 选择器 */
                #special { border-left: 3px solid #ffc107; }
                /* 伪类 */
                .item:first-child { font-weight: bold; }
                .item:last-child { font-style: italic; }
                .item.active { color: #0066cc; }
                /* 相邻兄弟选择器 */
                .item.active + .item { color: #666; }
                /* :lang 选择器 */
                [lang="en"] { font-style: normal; }
                /* :not 选择器 */
                .text:not(.secondary) { font-size: 1.2em; }
                /* 通用选择器 */
                .box > * { padding: 2px; }
            "#.into(),
            assertions: vec![
                "dom_has_body".into(),
                "has_fill_primitives".into(),
                "has_glyph_primitives".into(),
                "layout_has_children".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // cursor 属性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/cursor".into(),
            description: "cursor 属性不崩溃".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="cursor: pointer">Pointer</div>
            <div style="cursor: default">Default</div>
            <div style="cursor: text">Text</div>
            <div style="cursor: move">Move</div>
            <div style="cursor: not-allowed">Not allowed</div>
            <div style="cursor: crosshair">Crosshair</div>
            <div style="cursor: grab">Grab</div>
            <div style="cursor: wait">Wait</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // pointer-events
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/pointer-events".into(),
            description: "pointer-events 属性渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="pointer-events: auto; padding: 10px; margin: 5px; background: #eee">Auto</div>
            <div style="pointer-events: none; padding: 10px; margin: 5px; background: #eee">None</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // CSS transform
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/transform-2d".into(),
            description: "2D transform 渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="transform: translate(50px, 20px); background: #ffcccc; padding: 10px; margin: 10px">Translated</div>
            <div style="transform: rotate(45deg); background: #ccffcc; padding: 10px; margin: 10px; width: 50px; height: 50px">Rotated</div>
            <div style="transform: scale(1.5); background: #ccccff; padding: 10px; margin: 10px">Scaled</div>
            <div style="transform: skew(10deg, 5deg); background: #ffffcc; padding: 10px; margin: 10px">Skewed</div>
            <div style="transform: translateX(30px) rotate(15deg); background: #ffccff; padding: 10px; margin: 10px">Combined</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 综合页面：landing page
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/composite/landing".into(),
            description: "Landing page 综合渲染".into(),
            category: "typography".into(),
            html: r##"<html><body>
            <header class="hero">
                <nav>
                    <a href="#" class="logo">ZeroWeb</a>
                    <div class="nav-links">
                        <a href="#features">Features</a>
                        <a href="#pricing">Pricing</a>
                        <a href="#about">About</a>
                    </div>
                </nav>
                <h1 class="hero-title">Build the Web with Rust</h1>
                <p class="hero-subtitle">A fast, safe, and modern browser engine</p>
                <button class="cta">Get Started</button>
            </header>
            <section id="features">
                <h2>Features</h2>
                <div class="feature-grid">
                    <div class="feature">
                        <h3>Fast</h3>
                        <p>Written in Rust for maximum performance</p>
                    </div>
                    <div class="feature">
                        <h3>Safe</h3>
                        <p>Memory safety without garbage collection</p>
                    </div>
                    <div class="feature">
                        <h3>Standard</h3>
                        <p>Web standard compliant rendering engine</p>
                    </div>
                </div>
            </section>
            <footer>
                <p>&copy; 2026 ZeroWeb Project</p>
            </footer>
            </body></html>"##.into(),
            css: r##"
                :root { --primary: #4285f4; --dark: #1a1a2e; --light: #f5f5f5; }
                body { margin: 0; font-family: system-ui, sans-serif; color: #333; }
                .hero {
                    background: linear-gradient(135deg, var(--dark), #16213e);
                    color: white; padding: 60px 40px; text-align: center;
                }
                nav { display: flex; justify-content: space-between; align-items: center; max-width: 1000px; margin: 0 auto 40px; }
                .logo { font-size: 1.5em; font-weight: bold; color: white; text-decoration: none; }
                .nav-links a { color: #ccc; text-decoration: none; margin-left: 20px; }
                .hero-title { font-size: 3em; margin: 0; }
                .hero-subtitle { font-size: 1.3em; color: #aaa; margin: 10px 0 30px; }
                .cta {
                    background: var(--primary); color: white; border: none;
                    padding: 15px 40px; font-size: 1.1em; border-radius: 8px; cursor: pointer;
                    box-shadow: 0 4px 15px rgba(66, 133, 244, 0.4);
                }
                #features { padding: 60px 40px; max-width: 1000px; margin: 0 auto; }
                #features h2 { text-align: center; font-size: 2em; margin-bottom: 40px; }
                .feature-grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 30px; }
                .feature {
                    background: var(--light); padding: 30px; border-radius: 12px;
                    text-align: center; border: 1px solid #e0e0e0;
                }
                .feature h3 { color: var(--primary); margin-top: 0; }
                footer { background: var(--dark); color: #ccc; text-align: center; padding: 20px; margin-top: 60px; }
            "##.into(),
            assertions: vec![
                "dom_has_body".into(),
                "dom_has_header".into(),
                "dom_has_nav".into(),
                "dom_has_element:section".into(),
                "dom_has_element:footer".into(),
                "dom_has_button".into(),
                "has_fill_primitives".into(),
                "has_glyph_primitives".into(),
                "gradient_count_ge:1".into(),
                "shadow_count_ge:1".into(),
                "layout_has_children".into(),
                "no_panic".into(),
            ],
        },
        // ═══════════════════════════════════════════════════════════════
        // filter 属性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/filter".into(),
            description: "CSS filter 渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="filter: blur(2px); background: #eee; padding: 10px; margin: 5px">Blurred</div>
            <div style="filter: brightness(1.5); background: #eee; padding: 10px; margin: 5px">Bright</div>
            <div style="filter: grayscale(100%); background: #cc6633; padding: 10px; margin: 5px; color: white">Grayscale</div>
            <div style="filter: sepia(100%); background: #eee; padding: 10px; margin: 5px">Sepia</div>
            <div style="filter: contrast(200%); background: #eee; padding: 10px; margin: 5px">High contrast</div>
            <div style="filter: none; background: #eee; padding: 10px; margin: 5px">No filter</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 多列布局
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/multi-column".into(),
            description: "CSS 多列布局渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="column-count: 3; column-gap: 20px; column-rule: 1px solid #ccc; padding: 10px; margin: 10px">
                <p>Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.</p>
                <p>Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.</p>
                <p>Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.</p>
            </div>
            <div style="column-width: 150px; column-gap: 15px; padding: 10px; margin: 10px">
                <p>Column width based layout with automatic column count.</p>
                <p>More content to fill columns.</p>
            </div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_glyph_primitives".into(), "layout_has_children".into(), "no_panic".into()],
        },
        // ═══════════════════════════════════════════════════════════════
        // 混合 CSS 特性
        // ═══════════════════════════════════════════════════════════════
        TestCase {
            id: "typography/mix-blend-mode".into(),
            description: "mix-blend-mode 渲染不崩溃".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="position: relative; height: 100px; margin: 10px">
                <div style="position: absolute; width: 80px; height: 80px; background: red; top: 0; left: 0"></div>
                <div style="position: absolute; width: 80px; height: 80px; background: blue; top: 20px; left: 20px; mix-blend-mode: multiply"></div>
            </div>
            <div style="mix-blend-mode: screen; background: green; padding: 10px; margin: 5px">Screen blend</div>
            <div style="mix-blend-mode: overlay; background: yellow; padding: 10px; margin: 5px">Overlay blend</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "no_panic".into()],
        },
        TestCase {
            id: "typography/contain".into(),
            description: "CSS contain 属性渲染".into(),
            category: "typography".into(),
            html: r#"<html><body>
            <div style="contain: layout; padding: 10px; margin: 5px; background: #eee">Layout containment</div>
            <div style="contain: paint; padding: 10px; margin: 5px; background: #eee">Paint containment</div>
            <div style="contain: size; padding: 10px; margin: 5px; background: #eee">Size containment</div>
            <div style="contain: strict; padding: 10px; margin: 5px; background: #eee">Strict containment</div>
            <div style="contain: content; padding: 10px; margin: 5px; background: #eee">Content containment</div>
            <div style="contain: none; padding: 10px; margin: 5px; background: #eee">No containment</div>
            </body></html>"#.into(),
            css: String::new(),
            assertions: vec!["dom_has_body".into(), "has_fill_primitives".into(), "has_glyph_primitives".into(), "no_panic".into()],
        },
    ]
}
