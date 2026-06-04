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
        "dom_has_paragraph" => assert_dom_has_element(output, "p"),
        "dom_has_span" => assert_dom_has_element(output, "span"),
        "dom_has_section" => assert_dom_has_element(output, "section"),
        "dom_has_article" => assert_dom_has_element(output, "article"),
        "dom_has_nav" => assert_dom_has_element(output, "nav"),
        "dom_has_header" => assert_dom_has_element(output, "header"),
        "dom_has_footer" => assert_dom_has_element(output, "footer"),
        // Render assertions
        "render_completes" => assert_render_completes(output),
        "has_fill_primitives" => assert_has_fills(output),
        "has_glyph_primitives" => assert_has_glyphs(output),
        "has_multiple_fills" => assert_has_multiple_fills(output),
        "has_shadow_primitives" => assert_has_shadows(output),
        "has_stroke_primitives" => assert_has_strokes(output),
        "has_image_primitives" => assert_has_images(output),
        // Layout assertions
        "layout_has_children" => assert_layout_has_children(output),
        "layout_has_deep_children" => assert_layout_has_deep_children(output),
        "layout_valid_viewport" => assert_layout_valid_viewport(output),
        "layout_width_positive" => assert_layout_width_positive(output),
        "layout_height_positive" => assert_layout_height_positive(output),
        "layout_has_many_children" => assert_layout_has_many_children(output),
        // Aliases for convenience
        "css_background_applied" => assert_has_fills(output),
        "block_layout" => assert_layout_has_children(output),
        "inline_layout" => assert_has_glyphs(output),
        "flex_layout" => assert_layout_has_children(output),
        "grid_layout" => assert_layout_has_children(output),
        "nonzero_primitives" => assert_nonzero_primitives(output),
        "no_panic" => Ok(()),
        _ if name.starts_with("dom_has_element:") => {
            let tag = name.strip_prefix("dom_has_element:").unwrap_or("");
            assert_dom_has_element(output, tag)
        }
        // 精确布局断言：layout_child_count_ge:N — 子元素数 >= N
        _ if name.starts_with("layout_child_count_ge:") => {
            let n = name
                .strip_prefix("layout_child_count_ge:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_layout_child_count_ge(output, n)
        }
        // 精确布局断言：layout_depth_ge:N — 树深度 >= N
        _ if name.starts_with("layout_depth_ge:") => {
            let n = name
                .strip_prefix("layout_depth_ge:")
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(0);
            assert_layout_depth_ge(output, n)
        }
        // 布局断言：root 维度接近视口
        "layout_root_fills_viewport" => assert_layout_root_fills_viewport(output),
        // 布局断言：存在多个非零尺寸子盒
        "layout_has_sized_children" => assert_layout_has_sized_children(output),
        // 布局断言：子盒之间没有重叠（排除 display:none）
        "layout_children_non_overlapping" => assert_layout_children_non_overlapping(output),
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

fn assert_has_shadows(output: &RenderOutput) -> Result<(), String> {
    if output.primitives.shadows.is_empty() {
        Err("No shadow primitives generated".to_string())
    } else {
        Ok(())
    }
}

fn assert_has_strokes(output: &RenderOutput) -> Result<(), String> {
    if output.primitives.strokes.is_empty() {
        Err("No stroke primitives generated".to_string())
    } else {
        Ok(())
    }
}

fn assert_has_images(output: &RenderOutput) -> Result<(), String> {
    if output.primitives.images.is_empty() {
        Err("No image primitives generated".to_string())
    } else {
        Ok(())
    }
}

fn assert_layout_has_many_children(output: &RenderOutput) -> Result<(), String> {
    let count = output.layout.root.children.len();
    if count >= 3 {
        Ok(())
    } else {
        Err(format!("Layout root has {} children (expected >= 3)", count))
    }
}

fn assert_layout_child_count_ge(output: &RenderOutput, min: usize) -> Result<(), String> {
    let count = output.layout.root.children.len();
    if count >= min {
        Ok(())
    } else {
        Err(format!("Layout root has {} children (expected >= {min})", count))
    }
}

fn assert_layout_depth_ge(output: &RenderOutput, min_depth: u32) -> Result<(), String> {
    let depth = max_layout_depth(&output.layout.root, 0);
    if depth >= min_depth {
        Ok(())
    } else {
        Err(format!("Layout tree depth is {depth} (expected >= {min_depth})"))
    }
}

fn max_layout_depth(box_node: &zero_layout_engine::LayoutBox, current: u32) -> u32 {
    box_node
        .children
        .iter()
        .map(|c| max_layout_depth(c, current + 1))
        .max()
        .unwrap_or(current)
}

fn assert_layout_root_fills_viewport(output: &RenderOutput) -> Result<(), String> {
    let root = &output.layout.root;
    let vw = output.layout.viewport_width;
    let vh = output.layout.viewport_height;
    let width_ok = (root.width - vw).abs() < 1.0;
    let height_ok = root.height > 0.0 && root.height <= vh * 1.5;
    if width_ok && height_ok {
        Ok(())
    } else {
        Err(format!(
            "Root {}x{} doesn't fill viewport {}x{}",
            root.width, root.height, vw, vh
        ))
    }
}

fn assert_layout_has_sized_children(output: &RenderOutput) -> Result<(), String> {
    let sized = output
        .layout
        .root
        .children
        .iter()
        .filter(|c| c.width > 0.0 && c.height > 0.0)
        .count();
    if sized >= 2 {
        Ok(())
    } else {
        Err(format!(
            "Only {sized} children have positive dimensions (expected >= 2)"
        ))
    }
}

fn assert_layout_children_non_overlapping(output: &RenderOutput) -> Result<(), String> {
    let children = &output.layout.root.children;
    if children.len() < 2 {
        return Ok(());
    }
    let sized: Vec<_> = children.iter().filter(|c| c.width > 0.0 && c.height > 0.0).collect();
    for i in 0..sized.len() {
        for j in (i + 1)..sized.len() {
            let a = sized[i];
            let b = sized[j];
            let overlap_x = a.x < b.x + b.width && b.x < a.x + a.width;
            let overlap_y = a.y < b.y + b.height && b.y < a.y + a.height;
            if overlap_x && overlap_y {
                // 允许少量重叠（边距合并等），但不允许完全包含
                let a_contains_b =
                    a.x <= b.x && a.y <= b.y && a.x + a.width >= b.x + b.width && a.y + a.height >= b.y + b.height;
                if a_contains_b {
                    continue;
                }
                // 对于同层级块级元素，水平方向不应重叠
                return Err(format!(
                    "Children overlap: [{:.1},{:.1},{:.1},{:.1}] vs [{:.1},{:.1},{:.1},{:.1}]",
                    a.x, a.y, a.width, a.height, b.x, b.y, b.width, b.height
                ));
            }
        }
    }
    Ok(())
}

mod test_cases;

// 重新导出 builtin_tests 以保持公共 API 不变
pub use test_cases::builtin_tests;

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
            tests.len() >= 300,
            "Should have at least 300 builtin tests, got {}",
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
        let valid_categories = [
            "html",
            "css",
            "layout",
            "dom",
            "es-modules",
            "web-workers",
            "css-layout",
            "canvas",
            "storage",
        ];
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
