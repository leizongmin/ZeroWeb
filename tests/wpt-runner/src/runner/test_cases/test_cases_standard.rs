//! WPT 标准合规性测试用例。
//!
//! 包含更多 CSS 属性、HTML 元素和 DOM 标准测试：
//! - CSS Box Model 扩展（box-shadow、outline、border-style）
//! - CSS Table 布局
//! - CSS List 样式
//! - CSS Writing Mode
//! - CSS Object Fit
//! - HTML 表单变体
//! - HTML 媒体元素变体
//! - 复杂布局组合

use super::TestCase;

/// 返回标准合规性扩展测试用例。
pub fn standard_compliance_tests() -> Vec<TestCase> {
    vec![
        // ═══════════════════════════════════════════════════════════════
        //  CSS BOX MODEL EXTENDED
        // ═══════════════════════════════════════════════════════════════

        // ── border-style variants ──
        TestCase {
            id: "css/border-style-dashed".to_string(),
            description: "CSS border-style dashed".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="box">Dashed border</div></body></html>"#.to_string(),
            css: ".box { border: 2px dashed red; width: 200px; height: 100px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "stroke_count_ge:1".to_string()],
        },
        TestCase {
            id: "css/border-style-dotted".to_string(),
            description: "CSS border-style dotted".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="box">Dotted border</div></body></html>"#.to_string(),
            css: ".box { border: 3px dotted blue; width: 200px; height: 100px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "stroke_count_ge:1".to_string()],
        },
        TestCase {
            id: "css/border-style-double".to_string(),
            description: "CSS border-style double".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="box">Double border</div></body></html>"#.to_string(),
            css: ".box { border: 4px double green; width: 200px; height: 100px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ── border individual sides ──
        TestCase {
            id: "css/border-individual-sides".to_string(),
            description: "CSS individual border sides".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="box">Sides</div></body></html>"#.to_string(),
            css: ".box { border-top: 2px solid red; border-right: 3px solid blue; border-bottom: 4px solid green; border-left: 1px solid orange; width: 200px; height: 100px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ── multiple box-shadow ──
        TestCase {
            id: "css/multiple-box-shadow".to_string(),
            description: "Multiple box-shadows".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="shadowed">Shadowed</div></body></html>"#.to_string(),
            css: ".shadowed { box-shadow: 2px 2px 4px rgba(0,0,0,0.3), -2px -2px 8px rgba(255,0,0,0.2); width: 200px; height: 100px; background-color: white; }".to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ── outline-offset ──
        TestCase {
            id: "css/outline-offset".to_string(),
            description: "CSS outline with offset".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="outlined">Outline</div></body></html>"#.to_string(),
            css: ".outlined { outline: 3px solid blue; outline-offset: 4px; width: 200px; height: 100px; background-color: lightgray; }".to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS TABLE LAYOUT
        // ═══════════════════════════════════════════════════════════════

        // ── table with fixed layout ──
        TestCase {
            id: "css/table-fixed-layout".to_string(),
            description: "CSS table-layout fixed".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <table class="fixed">
                    <tr><td>A</td><td>B</td><td>C</td></tr>
                    <tr><td>D</td><td>E</td><td>F</td></tr>
                </table>
            </body></html>"#.to_string(),
            css: ".fixed { table-layout: fixed; width: 300px; border-collapse: collapse; } td { border: 1px solid black; padding: 8px; }".to_string(),
            assertions: vec!["dom_has_table".to_string(), "render_completes".to_string()],
        },
        // ── table with caption ──
        TestCase {
            id: "html/table-caption".to_string(),
            description: "HTML table with caption".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <table>
                    <caption>Sales Data</caption>
                    <thead><tr><th>Product</th><th>Amount</th></tr></thead>
                    <tbody>
                        <tr><td>Widget</td><td>$100</td></tr>
                        <tr><td>Gadget</td><td>$200</td></tr>
                    </tbody>
                    <tfoot><tr><td>Total</td><td>$300</td></tr></tfoot>
                </table>
            </body></html>"#.to_string(),
            css: "table { border-collapse: collapse; width: 300px; } th, td { border: 1px solid gray; padding: 4px; } caption { font-weight: bold; }".to_string(),
            assertions: vec!["dom_has_table".to_string(), "dom_has_text".to_string(), "render_completes".to_string()],
        },
        // ── table cell span ──
        TestCase {
            id: "html/table-cellspan".to_string(),
            description: "HTML table with colspan/rowspan".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <table>
                    <tr><td colspan="2">Wide</td></tr>
                    <tr><td>A</td><td>B</td></tr>
                </table>
            </body></html>"#.to_string(),
            css: "table { width: 200px; border-collapse: collapse; } td { border: 1px solid black; }".to_string(),
            assertions: vec!["dom_has_table".to_string(), "render_completes".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS LIST STYLES
        // ═══════════════════════════════════════════════════════════════

        // ── list-style-type ──
        TestCase {
            id: "css/list-style-type".to_string(),
            description: "CSS list-style-type variants".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <ul class="disc"><li>A</li><li>B</li></ul>
                <ol class="roman"><li>I</li><li>II</li></ol>
            </body></html>"#.to_string(),
            css: ".disc { list-style-type: disc; } .roman { list-style-type: upper-roman; }".to_string(),
            assertions: vec!["dom_has_list".to_string(), "dom_has_text".to_string(), "render_completes".to_string()],
        },
        // ── list-style-position ──
        TestCase {
            id: "css/list-style-position".to_string(),
            description: "CSS list-style-position inside".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <ul class="inside"><li>Inside marker</li><li>Second item</li></ul>
            </body></html>"#.to_string(),
            css: ".inside { list-style-position: inside; }".to_string(),
            assertions: vec!["dom_has_list".to_string(), "dom_has_text".to_string(), "render_completes".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS WRITING MODE
        // ═══════════════════════════════════════════════════════════════

        // ── vertical writing mode ──
        TestCase {
            id: "css/writing-mode-vertical".to_string(),
            description: "CSS writing-mode vertical-rl".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="vertical">Vertical text</div></body></html>"#.to_string(),
            css: ".vertical { writing-mode: vertical-rl; background-color: wheat; width: 200px; height: 200px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS OBJECT FIT
        // ═══════════════════════════════════════════════════════════════

        // ── object-fit cover ──
        TestCase {
            id: "css/object-fit-cover".to_string(),
            description: "CSS object-fit cover".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><img class="cover" src="test.png" alt="test" /></body></html>"#.to_string(),
            css: ".cover { width: 200px; height: 200px; object-fit: cover; }".to_string(),
            assertions: vec!["dom_has_img".to_string(), "render_completes".to_string()],
        },
        // ── object-fit contain ──
        TestCase {
            id: "css/object-fit-contain".to_string(),
            description: "CSS object-fit contain".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><img class="contain" src="test.png" alt="test" /></body></html>"#.to_string(),
            css: ".contain { width: 300px; height: 200px; object-fit: contain; background-color: gray; }".to_string(),
            assertions: vec!["dom_has_img".to_string(), "render_completes".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  HTML FORM VARIANTS
        // ═══════════════════════════════════════════════════════════════

        // ── range input ──
        TestCase {
            id: "html/input-range".to_string(),
            description: "HTML range input".to_string(),
            category: "html".to_string(),
            html: r##"<html><body>
                <form>
                    <input type="range" min="0" max="100" value="50" />
                    <input type="color" value="#ff0000" />
                    <input type="date" />
                    <input type="time" />
                </form>
            </body></html>"##.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_form".to_string(), "dom_has_input".to_string(), "render_completes".to_string()],
        },
        // ── fieldset with disabled ──
        TestCase {
            id: "html/fieldset-disabled".to_string(),
            description: "HTML fieldset disabled state".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <fieldset disabled>
                    <legend>Disabled Section</legend>
                    <input type="text" value="disabled" />
                </fieldset>
            </body></html>"#.to_string(),
            css: "fieldset { border: 2px solid gray; padding: 10px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "dom_has_input".to_string()],
        },
        // ── datalist ──
        TestCase {
            id: "html/datalist".to_string(),
            description: "HTML datalist element".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <form>
                    <input list="browsers" name="browser" />
                    <datalist id="browsers">
                        <option value="Chrome">
                        <option value="Firefox">
                        <option value="Safari">
                    </datalist>
                </form>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_form".to_string(), "dom_has_input".to_string(), "render_completes".to_string()],
        },
        // ── progress and meter ──
        TestCase {
            id: "html/progress-meter".to_string(),
            description: "HTML progress and meter elements".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <progress value="70" max="100">70%</progress>
                <meter value="0.7" min="0" max="1">70%</meter>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
        },
        // ── output element ──
        TestCase {
            id: "html/output-element".to_string(),
            description: "HTML output element".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <form oninput="result.value=parseInt(a.value)+parseInt(b.value)">
                    <input type="number" id="a" value="10" /> +
                    <input type="number" id="b" value="20" /> =
                    <output name="result" for="a b">30</output>
                </form>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_form".to_string(), "dom_has_input".to_string(), "dom_has_text".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  HTML MEDIA ELEMENTS
        // ═══════════════════════════════════════════════════════════════

        // ── picture element ──
        TestCase {
            id: "html/picture-element".to_string(),
            description: "HTML picture element".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <picture>
                    <source srcset="wide.jpg" media="(min-width: 600px)" />
                    <img src="narrow.jpg" alt="Responsive image" />
                </picture>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_img".to_string(), "render_completes".to_string()],
        },
        // ── map and area ──
        TestCase {
            id: "html/map-area".to_string(),
            description: "HTML image map".to_string(),
            category: "html".to_string(),
            html: r##"<html><body>
                <img src="planets.jpg" alt="Planets" usemap="#planetmap" width="400" height="300" />
                <map name="planetmap">
                    <area shape="rect" coords="0,0,100,100" href="sun.htm" alt="Sun" />
                    <area shape="circle" coords="200,150,50" href="mercury.htm" alt="Mercury" />
                </map>
            </body></html>"##.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_img".to_string(), "render_completes".to_string(), "no_panic".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  COMPLEX LAYOUT COMBINATIONS
        // ═══════════════════════════════════════════════════════════════

        // ── Holy Grail layout ──
        TestCase {
            id: "layout/holy-grail".to_string(),
            description: "Holy Grail layout (header, footer, 3-column)".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
                <div class="container">
                    <header class="hdr">Header</header>
                    <div class="body-row">
                        <nav class="left">Nav</nav>
                        <main class="center">Main Content Area</main>
                        <aside class="right">Sidebar</aside>
                    </div>
                    <footer class="ftr">Footer</footer>
                </div>
            </body></html>"#.to_string(),
            css: ".container { display: flex; flex-direction: column; min-height: 100vh; } .hdr { background: #333; color: white; padding: 10px; } .body-row { display: flex; flex: 1; } .left { width: 150px; background: #eee; padding: 10px; } .center { flex: 1; padding: 20px; } .right { width: 200px; background: #eee; padding: 10px; } .ftr { background: #333; color: white; padding: 10px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "layout_has_deep_children".to_string(), "has_glyph_primitives".to_string()],
        },
        // ── Card grid layout ──
        TestCase {
            id: "layout/card-grid".to_string(),
            description: "Responsive card grid".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
                <div class="grid">
                    <div class="card"><h3>Card 1</h3><p>Description</p></div>
                    <div class="card"><h3>Card 2</h3><p>Description</p></div>
                    <div class="card"><h3>Card 3</h3><p>Description</p></div>
                    <div class="card"><h3>Card 4</h3><p>Description</p></div>
                    <div class="card"><h3>Card 5</h3><p>Description</p></div>
                    <div class="card"><h3>Card 6</h3><p>Description</p></div>
                </div>
            </body></html>"#.to_string(),
            css: ".grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 16px; padding: 20px; } .card { background: white; border: 1px solid #ddd; border-radius: 8px; padding: 16px; } .card h3 { margin: 0 0 8px 0; }".to_string(),
            assertions: vec!["render_completes".to_string(), "has_multiple_fills".to_string(), "has_glyph_primitives".to_string()],
        },
        // ── Dashboard layout with mixed layouts ──
        TestCase {
            id: "layout/dashboard".to_string(),
            description: "Dashboard with sidebar, header, and grid content".to_string(),
            category: "layout".to_string(),
            html: r##"<html><body>
                <div class="dashboard">
                    <aside class="sidebar">
                        <nav><a href="#">Home</a><a href="#">Stats</a><a href="#">Settings</a></nav>
                    </aside>
                    <div class="main">
                        <header class="topbar"><h1>Dashboard</h1></header>
                        <div class="stats">
                            <div class="stat">1,234</div>
                            <div class="stat">567</div>
                            <div class="stat">89%</div>
                            <div class="stat">$12.3K</div>
                        </div>
                    </div>
                </div>
            </body></html>"##.to_string(),
            css: ".dashboard { display: flex; height: 100vh; } .sidebar { width: 200px; background: #1a1a2e; color: white; padding: 20px; } .sidebar a { display: block; color: white; padding: 8px; } .main { flex: 1; display: flex; flex-direction: column; } .topbar { background: white; padding: 16px; border-bottom: 1px solid #eee; } .stats { display: grid; grid-template-columns: repeat(4, 1fr); gap: 16px; padding: 20px; } .stat { background: white; border: 1px solid #eee; border-radius: 8px; padding: 20px; font-size: 24px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "layout_has_deep_children".to_string(), "has_glyph_primitives".to_string()],
        },
        // ── Nested grid layout ──
        TestCase {
            id: "layout/nested-grid".to_string(),
            description: "Grid within grid layout".to_string(),
            category: "layout".to_string(),
            html: r#"<html><body>
                <div class="outer-grid">
                    <div class="span-full">Full Width</div>
                    <div class="inner-grid">
                        <div>A</div><div>B</div><div>C</div><div>D</div>
                    </div>
                    <div class="sidebar">Side</div>
                </div>
            </body></html>"#.to_string(),
            css: ".outer-grid { display: grid; grid-template-columns: 3fr 1fr; gap: 10px; padding: 10px; } .span-full { grid-column: 1 / -1; background: #333; color: white; padding: 10px; } .inner-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 10px; } .inner-grid div { background: lightblue; padding: 20px; } .sidebar { background: lightyellow; padding: 20px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "layout_has_deep_children".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS CUSTOM PROPERTIES EXTENDED
        // ═══════════════════════════════════════════════════════════════

        // ── CSS custom properties with multiple vars ──
        TestCase {
            id: "css/custom-properties-multi".to_string(),
            description: "Multiple CSS custom properties".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="themed-box">Themed Content</div>
            </body></html>"#.to_string(),
            css: ":root { --primary: #336699; --bg: #f5f5f5; --radius: 8px; --spacing: 16px; } .themed-box { background-color: var(--bg); color: var(--primary); border-radius: var(--radius); padding: var(--spacing); border: 1px solid var(--primary); }".to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string(), "has_glyph_primitives".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS FILTER EFFECTS
        // ═══════════════════════════════════════════════════════════════

        // ── CSS blur filter ──
        TestCase {
            id: "css/filter-blur".to_string(),
            description: "CSS filter blur".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="blur">Blurred</div></body></html>"#.to_string(),
            css: ".blur { filter: blur(5px); width: 200px; height: 100px; background-color: tomato; }".to_string(),
            assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
        },
        // ── CSS brightness filter ──
        TestCase {
            id: "css/filter-brightness".to_string(),
            description: "CSS filter brightness".to_string(),
            category: "css".to_string(),
            html: r#"<html><body><div class="bright">Bright</div></body></html>"#.to_string(),
            css: ".bright { filter: brightness(1.5); width: 200px; height: 100px; background-color: orange; }".to_string(),
            assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  HTML ACCESSIBILITY ATTRIBUTES
        // ═══════════════════════════════════════════════════════════════

        // ── ARIA attributes ──
        TestCase {
            id: "html/aria-attributes".to_string(),
            description: "HTML elements with ARIA attributes".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <nav aria-label="Main navigation">
                    <ul><li><a href="/" aria-current="page">Home</a></li><li><a href="/about">About</a></li></ul>
                </nav>
                <main aria-labelledby="title">
                    <h1 id="title">Welcome</h1>
                    <p role="status" aria-live="polite">Loading complete.</p>
                </main>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec!["dom_has_link".to_string(), "dom_has_text".to_string(), "render_completes".to_string()],
        },
        // ── HTML data attributes ──
        TestCase {
            id: "html/data-attributes".to_string(),
            description: "HTML data-* attributes".to_string(),
            category: "html".to_string(),
            html: r#"<html><body>
                <div data-user-id="42" data-role="admin" data-active="true">
                    <span>Admin User</span>
                </div>
            </body></html>"#.to_string(),
            css: "[data-role=\"admin\"] { background-color: gold; padding: 10px; }".to_string(),
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  CSS PSEUDO-ELEMENTS AND CONTENT
        // ═══════════════════════════════════════════════════════════════

        // ── ::first-line and ::first-letter ──
        TestCase {
            id: "css/pseudo-first-line".to_string(),
            description: "CSS ::first-line pseudo-element".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <p class="styled">First line of text. Second line of text. Third line of text.</p>
            </body></html>"#.to_string(),
            css: ".styled::first-line { font-weight: bold; color: navy; }".to_string(),
            assertions: vec!["render_completes".to_string(), "dom_has_text".to_string()],
        },

        // ═══════════════════════════════════════════════════════════════
        //  EDGE CASES — LARGE AND STRESS TESTS
        // ═══════════════════════════════════════════════════════════════

        // ── Many CSS rules ──
        TestCase {
            id: "css/many-rules".to_string(),
            description: "Stylesheet with 50 CSS rules".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
                <div class="c1">1</div><div class="c2">2</div><div class="c3">3</div>
                <div class="c4">4</div><div class="c5">5</div>
            </body></html>"#.to_string(),
            css: {
                let mut css = String::new();
                for i in 1..=50 {
                    css.push_str(&format!(".c{i} {{ width: 100px; height: 50px; background-color: hsl({}, 50%, 50%); }}\n", i * 7));
                }
                css
            },
            assertions: vec!["render_completes".to_string(), "has_fill_primitives".to_string()],
        },
        // ── Deep nesting 20 levels ──
        TestCase {
            id: "layout/deep-nesting-20".to_string(),
            description: "Deeply nested elements (20 levels)".to_string(),
            category: "layout".to_string(),
            html: {
                let mut html = String::from("<html><body>");
                for _ in 0..20 {
                    html.push_str("<div>");
                }
                html.push_str("Deep");
                for _ in 0..20 {
                    html.push_str("</div>");
                }
                html.push_str("</body></html>");
                html
            },
            css: "div { width: 400px; height: 300px; background-color: #f0f0f0; }".to_string(),
            assertions: vec!["render_completes".to_string(), "no_panic".to_string()],
        },
        // ── 综合标准渲染测试 ──
        TestCase {
            id: "standard/css-spacing-render".to_string(),
            description: "CSS spacing 属性组合渲染标准验证".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width: 300px; color: #222;">
                <p style="font-size: 16px; letter-spacing: 3px;">Wide letter spacing text</p>
                <p style="font-size: 16px; word-spacing: 10px;">Wide word spacing text content</p>
                <p style="font-size: 16px; letter-spacing: -1px; word-spacing: -3px;">Tight spacing text</p>
                <p style="font-size: 16px; letter-spacing: 0; word-spacing: 0;">Normal spacing text</p>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
                "glyph_count_ge:20".to_string(),
            ],
        },
        TestCase {
            id: "standard/text-overflow-standard".to_string(),
            description: "text-overflow 标准合规性验证".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="width: 150px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: #333; font-size: 14px; border: 1px solid #ddd; padding: 4px;">
                This text should be truncated with ellipsis because it is too long
            </div>
            <div style="width: 150px; white-space: nowrap; overflow: hidden; text-overflow: clip; color: #333; font-size: 14px; border: 1px solid #ddd; padding: 4px;">
                This text should be clipped without ellipsis
            </div>
            <div style="width: 400px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; color: #333; font-size: 14px; border: 1px solid #ddd; padding: 4px;">
                Short
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        TestCase {
            id: "standard/filter-all-functions".to_string(),
            description: "CSS filter 所有函数类型渲染验证".to_string(),
            category: "css".to_string(),
            html: r#"<html><body>
            <div style="filter: blur(2px); background: #e0e0e0; padding: 8px; margin: 4px; color: #333; font-size: 14px;">Blur</div>
            <div style="filter: brightness(1.5); background: #e0e0e0; padding: 8px; margin: 4px; color: #333; font-size: 14px;">Brightness</div>
            <div style="filter: contrast(1.2); background: #e0e0e0; padding: 8px; margin: 4px; color: #333; font-size: 14px;">Contrast</div>
            <div style="filter: grayscale(1); background: #e0e0e0; padding: 8px; margin: 4px; color: #333; font-size: 14px;">Grayscale</div>
            <div style="filter: hue-rotate(90deg); background: #e0e0e0; padding: 8px; margin: 4px; color: #333; font-size: 14px;">Hue Rotate</div>
            <div style="filter: invert(1); background: #e0e0e0; padding: 8px; margin: 4px; color: #333; font-size: 14px;">Invert</div>
            <div style="filter: saturate(2); background: #e0e0e0; padding: 8px; margin: 4px; color: #333; font-size: 14px;">Saturate</div>
            <div style="filter: sepia(1); background: #e0e0e0; padding: 8px; margin: 4px; color: #333; font-size: 14px;">Sepia</div>
            <div style="filter: none; background: #e0e0e0; padding: 8px; margin: 4px; color: #333; font-size: 14px;">No Filter</div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
        TestCase {
            id: "standard/combined-css-features".to_string(),
            description: "CSS 多属性组合渲染验证".to_string(),
            category: "css".to_string(),
            html: r#"<html><body style="margin: 0; padding: 16px;">
            <div style="max-width: 500px; margin: 0 auto;">
                <h1 style="font-size: 24px; letter-spacing: 1px; color: #1a1a1a; margin: 0 0 12px; text-align: center;">Combined Features</h1>
                <div style="display: flex; gap: 12px; margin-bottom: 16px;">
                    <div style="flex: 1; padding: 12px; background: #E3F2FD; border-radius: 4px; filter: brightness(1.1);">
                        <h3 style="font-size: 14px; color: #1565C0; margin: 0 0 4px; letter-spacing: 0.5px;">Card A</h3>
                        <p style="font-size: 12px; color: #333; margin: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">Long text that should be truncated in this flex card</p>
                    </div>
                    <div style="flex: 1; padding: 12px; background: #E8F5E9; border-radius: 4px; filter: grayscale(0.2);">
                        <h3 style="font-size: 14px; color: #2E7D32; margin: 0 0 4px; letter-spacing: 0.5px;">Card B</h3>
                        <p style="font-size: 12px; color: #333; margin: 0;">Normal text in second card</p>
                    </div>
                </div>
                <p style="font-size: 13px; color: #666; letter-spacing: 0.3px; word-spacing: 1px; line-height: 1.6;">
                    Footer text with subtle spacing adjustments for readability. Testing multiple CSS properties working together.
                </p>
            </div>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
                "glyph_count_ge:10".to_string(),
            ],
        },
        TestCase {
            id: "standard/form-styled-spacing".to_string(),
            description: "表单 + 间距 + overflow 组合".to_string(),
            category: "html".to_string(),
            html: r#"<html><body style="padding: 20px;">
            <form style="max-width: 350px;">
                <label style="display: block; font-size: 14px; color: #333; letter-spacing: 0.5px; margin-bottom: 4px;">Username</label>
                <input style="width: 100%; padding: 8px; font-size: 14px; border: 1px solid #ccc; box-sizing: border-box; letter-spacing: 1px;" value="user@example.com">
                <div style="height: 8px;"></div>
                <label style="display: block; font-size: 14px; color: #333; letter-spacing: 0.5px; margin-bottom: 4px;">Password</label>
                <input style="width: 100%; padding: 8px; font-size: 14px; border: 1px solid #ccc; box-sizing: border-box;" type="password" value="secret">
                <div style="height: 12px;"></div>
                <div style="white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-size: 12px; color: #888;">
                    Error message that is quite long and should be truncated with an ellipsis marker
                </div>
            </form>
            </body></html>"#.to_string(),
            css: String::new(),
            assertions: vec![
                "dom_has_body".to_string(),
                "render_completes".to_string(),
                "has_fill_primitives".to_string(),
                "has_glyph_primitives".to_string(),
            ],
        },
    ]
}
