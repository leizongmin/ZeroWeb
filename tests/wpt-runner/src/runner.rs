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
        "render_completes" => assert_render_completes(output),
        "has_fill_primitives" => assert_has_fills(output),
        "has_glyph_primitives" => assert_has_glyphs(output),
        "layout_has_children" => assert_layout_has_children(output),
        "layout_valid_viewport" => assert_layout_valid_viewport(output),
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

/// 返回所有内置测试用例。
pub fn builtin_tests() -> Vec<TestCase> {
    vec![
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
            tests.len() >= 20,
            "Should have at least 20 builtin tests, got {}",
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
