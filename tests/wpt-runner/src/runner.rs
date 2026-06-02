//! 测试执行器 — 加载、解析并运行单个 HTML 测试。
//!
//! 使用 ZeroWeb 引擎的 RenderPipeline 在无头模式下执行渲染测试。
//! 通过检查 DOM 结构、布局结果和渲染图元来判定测试通过/失败。

use zero_dom::parse_html;
use zero_engine::RenderPipeline;
use zero_render_foundation::primitive::RenderPrimitives;

use crate::report::TestResult;

/// 单个 WPT 测试用例的定义。
#[derive(Debug, Clone)]
pub struct TestCase {
    /// 测试唯一标识符。
    pub id: String,
    /// 测试描述。
    pub description: String,
    /// 测试分类（如 html、css、layout）。
    pub category: String,
    /// HTML 内容。
    pub html: String,
    /// CSS 内容。
    pub css: String,
    /// 测试断言函数名（用于报告）。
    pub assertions: Vec<String>,
}

/// 测试执行上下文 — 每个测试用例共享的配置。
pub struct TestContext {
    /// 视口宽度。
    pub viewport_width: f32,
    /// 视口高度。
    pub viewport_height: f32,
}

impl Default for TestContext {
    fn default() -> Self {
        Self {
            viewport_width: 800.0,
            viewport_height: 600.0,
        }
    }
}

/// 单个测试的渲染输出 — 用于断言判断。
#[allow(dead_code)]
pub struct RenderOutput {
    /// 渲染图元。
    pub primitives: RenderPrimitives,
    /// DOM 文档。
    pub document: zero_dom::Document,
    /// 布局结果。
    pub layout: zero_layout_engine::LayoutResult,
    /// 视口宽度。
    pub viewport_width: f32,
    /// 视口高度。
    pub viewport_height: f32,
}

/// 渲染 HTML 并返回渲染输出（无头模式）。
#[allow(dead_code)]
pub fn render_test_html(html: &str, css: &str, ctx: &TestContext) -> RenderOutput {
    let mut pipeline = RenderPipeline::new(ctx.viewport_width, ctx.viewport_height);
    let result = pipeline.render_html(html, css);
    let doc = parse_html(html);

    RenderOutput {
        primitives: result.primitives,
        document: doc,
        layout: result.layout,
        viewport_width: ctx.viewport_width,
        viewport_height: ctx.viewport_height,
    }
}

/// 运行单个测试用例，返回结果。
pub fn run_single(case: &TestCase, ctx: &TestContext) -> TestResult {
    let mut pipeline = RenderPipeline::new(ctx.viewport_width, ctx.viewport_height);

    // 执行渲染 — 不应 panic
    let render_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pipeline.render_html(&case.html, &case.css)
    }));

    match render_result {
        Ok(result) => {
            let doc = parse_html(&case.html);
            let output = RenderOutput {
                primitives: result.primitives,
                document: doc,
                layout: result.layout,
                viewport_width: ctx.viewport_width,
                viewport_height: ctx.viewport_height,
            };
            // 运行内联断言
            let assertion_results: Vec<(String, Result<(), String>)> = case
                .assertions
                .iter()
                .map(|name| (name.clone(), check_assertion(name, &output)))
                .collect();

            let failed: Vec<&str> = assertion_results
                .iter()
                .filter(|(_, r)| r.is_err())
                .map(|(name, _)| name.as_str())
                .collect();

            if failed.is_empty() {
                TestResult::pass(&case.id, &case.description, result.timings.total_ms)
            } else {
                let msg = format!("Failed assertions: {}", failed.join(", "));
                TestResult::fail(&case.id, &case.description, &msg, result.timings.total_ms)
            }
        }
        Err(_) => TestResult::fail(&case.id, &case.description, "Rendering panicked", 0.0),
    }
}

/// 按断言名称分发到对应的检查函数。
fn check_assertion(name: &str, output: &RenderOutput) -> Result<(), String> {
    match name {
        // DOM assertions
        "dom_has_body" => assert_dom_has_body(output),
        "dom_has_text" => assert_dom_has_text(output),
        "dom_has_element" => assert_dom_has_element(output, "div"),
        "dom_has_link" => assert_dom_has_element(output, "a"),
        "dom_has_form" => assert_dom_has_element(output, "form"),
        "dom_has_input" => assert_dom_has_element(output, "input"),
        "dom_has_img" => assert_dom_has_element(output, "img"),
        "dom_has_button" => assert_dom_has_element(output, "button"),
        "dom_has_select" => assert_dom_has_element(output, "select"),
        "dom_has_table" => assert_dom_has_element(output, "table"),
        "dom_has_head" => assert_dom_has_element(output, "head"),
        "dom_has_title" => assert_dom_has_element(output, "title"),
        "dom_has_meta" => assert_dom_has_element(output, "meta"),
        "dom_has_list" => assert_dom_has_list(output),
        "dom_has_heading" => assert_dom_has_heading(output),
        // Render assertions
        "render_completes" => assert_render_completes(output),
        "has_fill_primitives" => assert_has_fills(output),
        "has_glyph_primitives" => assert_has_glyphs(output),
        "has_multiple_fills" => assert_has_multiple_fills(output),
        // Layout assertions
        "layout_has_children" => assert_layout_has_children(output),
        "layout_has_deep_children" => assert_layout_has_deep_children(output),
        "layout_valid_viewport" => assert_layout_valid_viewport(output),
        "layout_width_positive" => assert_layout_width_positive(output),
        "layout_height_positive" => assert_layout_height_positive(output),
        // Aliases for convenience
        "css_background_applied" => assert_has_fills(output),
        "block_layout" => assert_layout_has_children(output),
        "inline_layout" => assert_has_glyphs(output),
        "flex_layout" => assert_layout_has_children(output),
        "nonzero_primitives" => assert_nonzero_primitives(output),
        "no_panic" => Ok(()),
        _ => Err(format!("Unknown assertion: {name}")),
    }
}

// ── 断言函数 ─────────────────────────────────────────────────────

fn assert_dom_has_body(output: &RenderOutput) -> Result<(), String> {
    let body = output.document.get_elements_by_tag_name("body");
    if body.is_empty() {
        Err("DOM does not contain <body> element".to_string())
    } else {
        Ok(())
    }
}

fn assert_dom_has_text(output: &RenderOutput) -> Result<(), String> {
    let root = output.document.root();
    let has_text = has_text_recursive(&output.document, root);
    if has_text {
        Ok(())
    } else {
        Err("DOM does not contain any text nodes".to_string())
    }
}

fn has_text_recursive(doc: &zero_dom::Document, node_id: zero_dom::NodeId) -> bool {
    if let Some(data) = doc.get(node_id)
        && let zero_dom::NodeKind::Text(_) = data.kind
    {
        return true;
    }
    for child in doc.child_nodes(node_id) {
        if has_text_recursive(doc, child) {
            return true;
        }
    }
    false
}

fn assert_dom_has_element(output: &RenderOutput, tag: &str) -> Result<(), String> {
    let elements = output.document.get_elements_by_tag_name(tag);
    if elements.is_empty() {
        Err(format!("DOM does not contain <{tag}> element"))
    } else {
        Ok(())
    }
}

fn assert_render_completes(_output: &RenderOutput) -> Result<(), String> {
    // 如果能到达这里，渲染已成功完成
    Ok(())
}

fn assert_has_fills(output: &RenderOutput) -> Result<(), String> {
    if output.primitives.fills.is_empty() {
        Err("No fill primitives generated".to_string())
    } else {
        Ok(())
    }
}

fn assert_has_glyphs(output: &RenderOutput) -> Result<(), String> {
    if output.primitives.glyphs.is_empty() {
        Err("No glyph primitives generated (expected text rendering)".to_string())
    } else {
        Ok(())
    }
}

fn assert_layout_has_children(output: &RenderOutput) -> Result<(), String> {
    if output.layout.root.children.is_empty() {
        Err("Layout root has no children".to_string())
    } else {
        Ok(())
    }
}

fn assert_layout_valid_viewport(output: &RenderOutput) -> Result<(), String> {
    if output.layout.viewport_width <= 0.0 || output.layout.viewport_height <= 0.0 {
        Err(format!(
            "Invalid viewport: {}x{}",
            output.layout.viewport_width, output.layout.viewport_height
        ))
    } else {
        Ok(())
    }
}

fn assert_nonzero_primitives(output: &RenderOutput) -> Result<(), String> {
    if output.primitives.is_empty() {
        Err("No primitives generated at all".to_string())
    } else {
        Ok(())
    }
}

fn assert_dom_has_list(output: &RenderOutput) -> Result<(), String> {
    let ul = output.document.get_elements_by_tag_name("ul");
    let ol = output.document.get_elements_by_tag_name("ol");
    if ul.is_empty() && ol.is_empty() {
        Err("DOM does not contain <ul> or <ol> element".to_string())
    } else {
        Ok(())
    }
}

fn assert_dom_has_heading(output: &RenderOutput) -> Result<(), String> {
    let headings = ["h1", "h2", "h3", "h4", "h5", "h6"];
    for tag in &headings {
        if !output.document.get_elements_by_tag_name(tag).is_empty() {
            return Ok(());
        }
    }
    Err("DOM does not contain any heading element (h1-h6)".to_string())
}

fn assert_layout_has_deep_children(output: &RenderOutput) -> Result<(), String> {
    fn max_depth(layout: &zero_layout_engine::LayoutBox, current: u32) -> u32 {
        let child_depths: Vec<u32> = layout.children.iter().map(|c| max_depth(c, current + 1)).collect();
        child_depths.into_iter().max().unwrap_or(current)
    }
    let depth = max_depth(&output.layout.root, 0);
    if depth >= 2 {
        Ok(())
    } else {
        Err(format!("Layout tree depth is {depth}, expected at least 2"))
    }
}

fn assert_has_multiple_fills(output: &RenderOutput) -> Result<(), String> {
    let count =
        output.primitives.fills.len() + output.primitives.rounded_rects.len() + output.primitives.path_fills.len();
    if count > 1 {
        Ok(())
    } else {
        Err(format!("Expected >1 fill primitives, got {count}"))
    }
}

fn assert_layout_width_positive(output: &RenderOutput) -> Result<(), String> {
    if output.layout.root.width > 0.0 {
        Ok(())
    } else {
        Err(format!(
            "Root layout width is {} (expected > 0)",
            output.layout.root.width
        ))
    }
}

fn assert_layout_height_positive(output: &RenderOutput) -> Result<(), String> {
    if output.layout.root.height > 0.0 {
        Ok(())
    } else {
        Err(format!(
            "Root layout height is {} (expected > 0)",
            output.layout.root.height
        ))
    }
}

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
    ]
}

/// 按分类过滤测试用例。
pub fn filter_tests_by_category(tests: &[TestCase], category: &str) -> Vec<TestCase> {
    tests.iter().filter(|t| t.category == category).cloned().collect()
}

/// 按路径模式过滤测试用例。
pub fn filter_tests_by_pattern(tests: &[TestCase], pattern: &str) -> Vec<TestCase> {
    tests.iter().filter(|t| t.id.contains(pattern)).cloned().collect()
}

/// 运行所有给定的测试用例，返回结果列表。
pub fn run_all(cases: &[TestCase], ctx: &TestContext) -> Vec<TestResult> {
    cases.iter().map(|case| run_single(case, ctx)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_tests_count() {
        let tests = builtin_tests();
        assert!(
            tests.len() >= 80,
            "Should have at least 80 builtin tests, got {}",
            tests.len()
        );
    }

    #[test]
    fn test_builtin_tests_have_valid_ids() {
        let tests = builtin_tests();
        for t in &tests {
            assert!(!t.id.is_empty(), "Test ID should not be empty");
            assert!(!t.description.is_empty(), "Test description should not be empty");
            assert!(
                !t.assertions.is_empty(),
                "Test should have at least one assertion: {}",
                t.id
            );
        }
    }

    #[test]
    fn test_filter_by_category() {
        let tests = builtin_tests();
        let html_tests = filter_tests_by_category(&tests, "html");
        let css_tests = filter_tests_by_category(&tests, "css");
        let layout_tests = filter_tests_by_category(&tests, "layout");
        let dom_tests = filter_tests_by_category(&tests, "dom");

        assert!(!html_tests.is_empty(), "Should have html tests");
        assert!(!css_tests.is_empty(), "Should have css tests");
        assert!(!layout_tests.is_empty(), "Should have layout tests");
        assert!(!dom_tests.is_empty(), "Should have dom tests");
    }

    #[test]
    fn test_filter_by_pattern() {
        let tests = builtin_tests();
        let filtered = filter_tests_by_pattern(&tests, "css/");
        for t in &filtered {
            assert!(t.id.contains("css/"));
        }
    }

    #[test]
    fn test_render_test_html_simple() {
        let ctx = TestContext::default();
        let output = render_test_html("<html><body><div>Hello</div></body></html>", "", &ctx);
        assert!(output.viewport_width > 0.0);
        assert!(output.viewport_height > 0.0);
    }

    #[test]
    fn test_run_single_pass() {
        let ctx = TestContext::default();
        let case = TestCase {
            id: "test/pass".to_string(),
            description: "Passing test".to_string(),
            category: "html".to_string(),
            html: "<html><body>Pass</body></html>".to_string(),
            css: String::new(),
            assertions: vec!["dom_has_body".to_string()],
        };
        let result = run_single(&case, &ctx);
        assert!(result.passed, "Expected pass, got: {}", result.message);
    }

    #[test]
    fn test_run_single_fail_unknown_assertion() {
        let ctx = TestContext::default();
        let case = TestCase {
            id: "test/fail".to_string(),
            description: "Failing test".to_string(),
            category: "html".to_string(),
            html: "<html><body>Fail</body></html>".to_string(),
            css: String::new(),
            assertions: vec!["nonexistent_assertion".to_string()],
        };
        let result = run_single(&case, &ctx);
        assert!(!result.passed);
    }

    #[test]
    fn test_run_all_collects_results() {
        let ctx = TestContext::default();
        let cases: Vec<TestCase> = builtin_tests().into_iter().take(3).collect();
        let results = run_all(&cases, &ctx);
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_categories_are_valid() {
        let tests = builtin_tests();
        let valid_categories = ["html", "css", "layout", "dom"];
        for t in &tests {
            assert!(
                valid_categories.contains(&t.category.as_str()),
                "Invalid category '{}' for test '{}'",
                t.category,
                t.id
            );
        }
    }
}
