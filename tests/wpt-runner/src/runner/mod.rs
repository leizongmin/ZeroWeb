//! 测试执行器 — 加载、解析并运行单个 HTML 测试。
//!
//! 使用 ZeroWeb 引擎的 RenderPipeline 在无头模式下执行渲染测试。
//! 通过检查 DOM 结构、布局结果和渲染图元来判定测试通过/失败。
//! 支持预期元数据（PASS/FAIL/SKIP）管理已知行为。

use std::collections::HashMap;

use zero_dom::parse_html;
use zero_engine::RenderPipeline;
use zero_render_foundation::primitive::RenderPrimitives;

use crate::report::TestResult;

/// 测试预期结果 — 用于管理已知行为。
#[allow(dead_code)]
///
/// - `Pass`：测试预期通过（默认）
/// - `Fail`：测试预期失败（已知 bug 或未实现功能），不阻断 CI
/// - `Skip`：测试跳过（需要 GPU/Display 等不可用资源）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestExpectation {
    /// 预期通过（默认）
    Pass,
    /// 预期失败（已知问题）
    Fail,
    /// 跳过（条件不满足）
    Skip,
}

/// 测试预期元数据表 — 按测试 ID 管理已知行为。
#[allow(dead_code)]
///
/// 未在表中登记的测试默认预期为 `Pass`。
///
/// # 示例
///
/// ```
/// use zero_wpt_runner::runner::TestExpectations;
/// let mut exp = TestExpectations::new();
/// exp.expect_fail("geometry/grid/auto-fill".to_string());
/// exp.skip("geometry/position/fixed".to_string());
/// ```
#[derive(Debug, Clone, Default)]
pub struct TestExpectations {
    entries: HashMap<String, TestExpectation>,
}

impl TestExpectations {
    /// 创建空的预期表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 标记测试预期失败。
    #[allow(dead_code)]
    pub fn expect_fail(&mut self, id: String) {
        self.entries.insert(id, TestExpectation::Fail);
    }

    /// 标记测试跳过。
    #[allow(dead_code)]
    pub fn skip(&mut self, id: String) {
        self.entries.insert(id, TestExpectation::Skip);
    }

    /// 获取测试的预期结果（未登记则返回 Pass）。
    pub fn get(&self, id: &str) -> TestExpectation {
        self.entries.get(id).copied().unwrap_or(TestExpectation::Pass)
    }
}

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

/// 经**共享页面运行时**（WebView）渲染 HTML，返回图元——WPT 三路径统一的 runtime 路径。
///
/// 与 `render_test_html`（engine-direct）对自包含 HTML 产出一致（见 `runtime_path_tests`）。
/// reftest 确定性门仍走 engine-direct（`render_test_html` / `reftest.rs`）；本函数供 runtime 一致性校验，
/// 让 WPT 具备调用同一套页面运行时的能力（T6）。
#[allow(dead_code)]
pub fn render_test_html_via_runtime(
    html: &str,
    css: &str,
    ctx: &TestContext,
) -> zero_render_foundation::primitive::RenderPrimitives {
    let mut wv = zero_webview::WebView::new(zero_webview::WebViewConfig {
        width: ctx.viewport_width as u32,
        height: ctx.viewport_height as u32,
        ..Default::default()
    });
    wv.load_html(html, if css.is_empty() { None } else { Some(css) });
    wv.last_render().map(|r| r.primitives.clone()).unwrap_or_default()
}

/// R3076：`js_executes_ok` 断言——经 WebView 运行时路径**真实执行**内联 `<script>`（strict：首个脚本抛异常即失败）。
/// 闭合 web_api/js_dom 测试用例「空洞通过」（既不执行内联 JS → API 真损/行为错不可见）。与纯渲染断言
/// （dom_has_body/no_panic，仅查渲染快照）互补——本断言验证脚本运行时无异常（API 存在 + 基本可调）。
fn check_js_executes_ok(html: &str, ctx: &TestContext) -> Result<(), String> {
    let mut wv = zero_webview::WebView::new(zero_webview::WebViewConfig {
        width: ctx.viewport_width as u32,
        height: ctx.viewport_height as u32,
        ..Default::default()
    });
    wv.load_html(html, None);
    match wv.run_page_scripts_strict() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("inline script threw: {e}")),
    }
}

#[cfg(test)]
mod runtime_path_tests {
    use super::*;

    /// WPT 的 runtime 路径（WebView）须与 engine-direct 路径产出一致图元——三路径统一的实证。
    #[test]
    fn runtime_path_matches_engine_direct() {
        let ctx = TestContext::default();
        let html = r#"<html><body><div style="width:200px;height:100px;background:red">Box</div></body></html>"#;
        let engine = render_test_html(html, "", &ctx);
        let runtime = render_test_html_via_runtime(html, "", &ctx);
        assert_eq!(engine.primitives.fills.len(), runtime.fills.len());
        assert_eq!(engine.primitives.rounded_rects.len(), runtime.rounded_rects.len());
    }

    /// R3076：`check_js_executes_ok` 机制验证——有效脚本 Ok，抛异常脚本 Err。
    #[test]
    fn js_executes_ok_detects_throw_r3076() {
        let ctx = TestContext::default();
        let ok = check_js_executes_ok(
            r#"<html><body><script>var x = 1 + 2; document.body.dataset.x = x;</script></body></html>"#,
            &ctx,
        );
        assert!(ok.is_ok(), "valid script → Ok, got: {ok:?}");
        let threw = check_js_executes_ok(
            r#"<html><body><script>undefinedFunctionR3076();</script></body></html>"#,
            &ctx,
        );
        assert!(threw.is_err(), "throwing script → Err, got: {threw:?}");
    }

    /// R3076：web-api 代表性用例经 run_single（含 js_executes_ok 断言）通过——闭合「空洞通过」回归门。
    /// 采样 8 个跨 API 类别（fetch/websocket/performance/console/timer/observer/wasm）用例，避免全量 80 用例
    /// 各建 WebView 在跨 crate 并行测试下触发 V8 资源压力 flake（顺序建 80 isolate + 并行 load 偶发 init 失败）。
    #[test]
    fn web_api_cases_pass_with_js_executes_ok_r3076() {
        let ctx = TestContext::default();
        let cases = filter_tests_by_category(&builtin_tests(), "web-api");
        let sampled: Vec<&TestCase> = cases
            .iter()
            .filter(|c| c.assertions.iter().any(|a| a == "js_executes_ok"))
            // 每 6 个取 1 个，跨 API 类别采样 ~8 个代表（fetch→wasm 全谱）。
            .step_by(6)
            .collect();
        assert!(
            sampled.len() >= 6,
            "采样 web-api js_executes_ok 用例 ≥6，got {}",
            sampled.len()
        );
        for case in &sampled {
            let result = run_single(case, &ctx);
            assert!(
                result.passed(),
                "web-api {} 应通过（含 js_executes_ok 真实执行内联脚本）: {}",
                case.id,
                result.message
            );
        }
    }

    /// R3079：所有 canvas 用例经 `js_executes_ok` 真实执行内联脚本无异常。闭合 R3077（getContext DOM 集成）、
    /// R3078（fillText/measureText/createImageData）、R3079（createLinearGradient/createRadialGradient/
    /// createConicGradient、addColorStop、fillStyle=gradient、fill/fillRect 逐像素光栅化）后，canvas 赛道
    /// 全部用例脚本可执行。canvas 用例原仅声明 render_completes（渲染完成即过——脚本抛 TypeError 不可见，
    /// 即「空洞通过」）；本回归门经 check_js_executes_ok strict 执行验证 API 全链路可用，落地 R3078 defer 的
    /// 「gradient 闭合后 39/39 统一应用 js_executes_ok」。
    #[test]
    fn canvas_cases_execute_scripts_r3079() {
        let ctx = TestContext::default();
        let cases = filter_tests_by_category(&builtin_tests(), "canvas");
        assert!(!cases.is_empty(), "canvas 用例集非空");
        let mut failed: Vec<String> = Vec::new();
        for case in &cases {
            if let Err(e) = check_js_executes_ok(&case.html, &ctx) {
                failed.push(format!("{}: {}", case.id, e));
            }
        }
        assert!(
            failed.is_empty(),
            "canvas 用例应全部 js_executes_ok 通过（{} 例失败）:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// R3080：所有 web-workers 用例经 `js_executes_ok` 真实执行内联脚本无异常。闭合 Worker API 表面
    ///（postMessage/terminate/onmessage/onerror，EventTarget-based 构造器）后，`new Worker()` + postMessage +
    /// terminate 生命周期不再抛 TypeError。web-workers 用例原仅声明 render_completes（脚本抛 TypeError 不可见）；
    /// 本回归门经 strict 执行验证 Worker API 表面可用。
    #[test]
    fn web_worker_cases_execute_scripts_r3080() {
        let ctx = TestContext::default();
        let cases = filter_tests_by_category(&builtin_tests(), "web-workers");
        assert!(!cases.is_empty(), "web-workers 用例集非空");
        let mut failed: Vec<String> = Vec::new();
        for case in &cases {
            if let Err(e) = check_js_executes_ok(&case.html, &ctx) {
                failed.push(format!("{}: {}", case.id, e));
            }
        }
        assert!(
            failed.is_empty(),
            "web-workers 用例应全部 js_executes_ok 通过（{} 例失败）:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// R3081：所有 storage 用例经 `js_executes_ok` 真实执行内联脚本无异常。闭合 IndexedDB 内存表面
    ///（open/onupgradeneeded/onsuccess/transaction/store CRUD/createIndex）后，`indexedDB.open()` 全链不再抛
    /// `indexedDB is not defined`。storage 用例原仅声明 render_completes/dom_has_body（脚本抛不可见）；
    /// 本回归门经 strict 执行验证 IndexedDB API 表面可用。
    #[test]
    fn storage_cases_execute_scripts_r3081() {
        let ctx = TestContext::default();
        let cases = filter_tests_by_category(&builtin_tests(), "storage");
        assert!(!cases.is_empty(), "storage 用例集非空");
        let mut failed: Vec<String> = Vec::new();
        for case in &cases {
            if let Err(e) = check_js_executes_ok(&case.html, &ctx) {
                failed.push(format!("{}: {}", case.id, e));
            }
        }
        assert!(
            failed.is_empty(),
            "storage 用例应全部 js_executes_ok 通过（{} 例失败）:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// R3082：所有 runtime 用例经 `js_executes_ok` 真实执行内联脚本无异常。闭合 document.dispatchEvent
    ///（转发 _elKey('html',null)，对称 addEventListener）后，`document.dispatchEvent(new CustomEvent(...))`
    /// 不再抛 TypeError。runtime 用例原仅声明 render_completes/dom_has_body；本回归门经 strict 执行验证。
    #[test]
    fn runtime_cases_execute_scripts_r3082() {
        let ctx = TestContext::default();
        let cases = filter_tests_by_category(&builtin_tests(), "runtime");
        assert!(!cases.is_empty(), "runtime 用例集非空");
        let mut failed: Vec<String> = Vec::new();
        for case in &cases {
            if let Err(e) = check_js_executes_ok(&case.html, &ctx) {
                failed.push(format!("{}: {}", case.id, e));
            }
        }
        assert!(
            failed.is_empty(),
            "runtime 用例应全部 js_executes_ok 通过（{} 例失败）:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }

    /// R3083：所有 interactive 用例经 `js_executes_ok` 真实执行内联脚本无异常。闭合 `<script type=module>`
    /// 执行路径（compile_module_script 转换 import/export）后，含 module 脚本的页面（interactive/
    /// script-variants）不再抛 `Cannot use import statement outside a module`。
    #[test]
    fn interactive_cases_execute_scripts_r3083() {
        let ctx = TestContext::default();
        let cases = filter_tests_by_category(&builtin_tests(), "interactive");
        assert!(!cases.is_empty(), "interactive 用例集非空");
        let mut failed: Vec<String> = Vec::new();
        for case in &cases {
            if let Err(e) = check_js_executes_ok(&case.html, &ctx) {
                failed.push(format!("{}: {}", case.id, e));
            }
        }
        assert!(
            failed.is_empty(),
            "interactive 用例应全部 js_executes_ok 通过（{} 例失败）:\n{}",
            failed.len(),
            failed.join("\n")
        );
    }
}
/// 根据预期元数据管理已知行为：
#[allow(dead_code)]
/// - `Pass`：正常执行，失败则报告为 FAILED
/// - `Fail`：正常执行，失败报告为 EXPECTED_FAIL（不阻断），意外通过报告为 UNEXPECTED_PASS
/// - `Skip`：跳过执行，直接报告为 SKIPPED
pub fn run_single(case: &TestCase, ctx: &TestContext) -> TestResult {
    run_single_with_expectations(case, ctx, &TestExpectations::new())
}

/// 运行单个测试用例（带预期元数据），返回结果。
#[allow(dead_code)]
pub fn run_single_with_expectations(case: &TestCase, ctx: &TestContext, expectations: &TestExpectations) -> TestResult {
    let expected = expectations.get(&case.id);

    // 跳过的测试直接返回
    if expected == TestExpectation::Skip {
        return TestResult::skip_with_category(&case.id, &case.description, &case.category);
    }

    let mut pipeline = RenderPipeline::new(ctx.viewport_width, ctx.viewport_height);

    // 执行渲染 — 不应 panic
    let render_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        pipeline.render_html(&case.html, &case.css)
    }));

    let actual_result = match render_result {
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
                .map(|name| {
                    // R3076：js_executes_ok 经 WebView 运行时路径真实执行内联脚本（非纯渲染快照），需 case.html。
                    let r = if name == "js_executes_ok" {
                        check_js_executes_ok(&case.html, ctx)
                    } else {
                        check_assertion(name, &output)
                    };
                    (name.clone(), r)
                })
                .collect();

            let failed: Vec<&str> = assertion_results
                .iter()
                .filter(|(_, r)| r.is_err())
                .map(|(name, _)| name.as_str())
                .collect();

            if failed.is_empty() {
                (true, String::new(), result.timings.total_ms)
            } else {
                (
                    false,
                    format!("Failed assertions: {}", failed.join(", ")),
                    result.timings.total_ms,
                )
            }
        }
        Err(_) => (false, "Rendering panicked".to_string(), 0.0),
    };

    let (actual_passed, message, duration_ms) = actual_result;

    match (expected, actual_passed) {
        (TestExpectation::Pass, true) => {
            TestResult::pass_with_category(&case.id, &case.description, &case.category, duration_ms)
        }
        (TestExpectation::Pass, false) => {
            TestResult::fail_with_category(&case.id, &case.description, &case.category, &message, duration_ms)
        }
        (TestExpectation::Fail, true) => {
            TestResult::unexpected_pass_with_category(&case.id, &case.description, &case.category, duration_ms)
        }
        (TestExpectation::Fail, false) => {
            TestResult::expected_fail_with_category(&case.id, &case.description, &case.category, &message, duration_ms)
        }
        (TestExpectation::Skip, _) => unreachable!(),
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
        // 精确布局断言：layout_box_count_ge:N — 总节点数 >= N
        _ if name.starts_with("layout_box_count_ge:") => {
            let n = name
                .strip_prefix("layout_box_count_ge:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_layout_box_count_ge(output, n)
        }
        // 精确布局断言：layout_nth_size:IDX:WxH — 第 N 个盒子尺寸匹配
        _ if name.starts_with("layout_nth_size:") => {
            parse_layout_nth_size(name).and_then(|(idx, w, h)| assert_layout_nth_size(output, idx, w, h))
        }
        // 精确布局断言：layout_nth_pos:IDX:X,Y — 第 N 个盒子位置匹配
        _ if name.starts_with("layout_nth_pos:") => {
            parse_layout_nth_pos(name).and_then(|(idx, x, y)| assert_layout_nth_pos(output, idx, x, y))
        }
        // 精确布局断言：layout_nth_width_ge:IDX:N — 第 N 个盒子宽度 >= N
        _ if name.starts_with("layout_nth_width_ge:") => parse_layout_nth_float_ge(name, "layout_nth_width_ge:")
            .and_then(|(idx, min_w)| assert_layout_nth_width_ge(output, idx, min_w)),
        // 精确布局断言：layout_nth_height_ge:IDX:N — 第 N 个盒子高度 >= N
        _ if name.starts_with("layout_nth_height_ge:") => parse_layout_nth_float_ge(name, "layout_nth_height_ge:")
            .and_then(|(idx, min_h)| assert_layout_nth_height_ge(output, idx, min_h)),
        // 精确图元断言：fill_count:N — 填充图元精确数量
        _ if name.starts_with("fill_count:") => {
            let n = name
                .strip_prefix("fill_count:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_fill_count(output, n)
        }
        // 精确图元断言：fill_count_ge:N — 填充图元数量 >= N
        _ if name.starts_with("fill_count_ge:") => {
            let n = name
                .strip_prefix("fill_count_ge:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_fill_count_ge(output, n)
        }
        // 精确图元断言：glyph_count_ge:N — 字形图元数量 >= N
        _ if name.starts_with("glyph_count_ge:") => {
            let n = name
                .strip_prefix("glyph_count_ge:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_glyph_count_ge(output, n)
        }
        // 精确图元断言：stroke_count_ge:N — 描边图元数量 >= N
        _ if name.starts_with("stroke_count_ge:") => {
            let n = name
                .strip_prefix("stroke_count_ge:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_stroke_count_ge(output, n)
        }
        // 精确图元断言：gradient_count_ge:N — 渐变图元数量 >= N
        _ if name.starts_with("gradient_count_ge:") => {
            let n = name
                .strip_prefix("gradient_count_ge:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_gradient_count_ge(output, n)
        }
        // 精确图元断言：image_count_ge:N — 图片图元数量 >= N
        _ if name.starts_with("image_count_ge:") => {
            let n = name
                .strip_prefix("image_count_ge:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_image_count_ge(output, n)
        }
        // 精确图元断言：shadow_count_ge:N — 阴影图元数量 >= N
        _ if name.starts_with("shadow_count_ge:") => {
            let n = name
                .strip_prefix("shadow_count_ge:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_shadow_count_ge(output, n)
        }
        // 精确图元断言：total_primitive_count_ge:N — 总图元数量 >= N
        _ if name.starts_with("total_primitive_count_ge:") => {
            let n = name
                .strip_prefix("total_primitive_count_ge:")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            assert_total_primitive_count_ge(output, n)
        }
        // 快照断言：layout_snapshot — 生成布局快照到失败消息（调试用）
        "layout_snapshot" => Ok(()),
        "primitive_snapshot" => Ok(()),
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

// ── 精确几何断言 ─────────────────────────────────────────────────

fn assert_layout_box_count_ge(output: &RenderOutput, min: usize) -> Result<(), String> {
    let count = output.layout.root.count_boxes();
    if count >= min {
        Ok(())
    } else {
        Err(format!("Layout tree has {count} boxes (expected >= {min})"))
    }
}

fn parse_layout_nth_size(name: &str) -> Result<(usize, f32, f32), String> {
    // Format: layout_nth_size:IDX:WxH
    let rest = name.strip_prefix("layout_nth_size:").ok_or("bad prefix")?;
    let parts: Vec<&str> = rest.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid format, expected layout_nth_size:IDX:WxH, got {name}"));
    }
    let idx = parts[0].parse::<usize>().map_err(|e| e.to_string())?;
    let dims: Vec<&str> = parts[1].split('x').collect();
    if dims.len() != 2 {
        return Err(format!("Invalid dims format, expected WxH, got {}", parts[1]));
    }
    let w = dims[0].parse::<f32>().map_err(|e| e.to_string())?;
    let h = dims[1].parse::<f32>().map_err(|e| e.to_string())?;
    Ok((idx, w, h))
}

fn assert_layout_nth_size(output: &RenderOutput, idx: usize, expected_w: f32, expected_h: f32) -> Result<(), String> {
    match output.layout.root.nth_box(idx) {
        Some((_x, _y, w, h)) => {
            let tol = 1.0;
            if (w - expected_w).abs() > tol {
                return Err(format!("Box[{idx}] width={w:.2} expected={expected_w:.2} (tol={tol})"));
            }
            if (h - expected_h).abs() > tol {
                return Err(format!("Box[{idx}] height={h:.2} expected={expected_h:.2} (tol={tol})"));
            }
            Ok(())
        }
        None => Err(format!(
            "Box[{idx}] not found (only {} boxes)",
            output.layout.root.count_boxes()
        )),
    }
}

fn parse_layout_nth_pos(name: &str) -> Result<(usize, f32, f32), String> {
    let rest = name.strip_prefix("layout_nth_pos:").ok_or("bad prefix")?;
    let parts: Vec<&str> = rest.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid format, expected layout_nth_pos:IDX:X,Y, got {name}"));
    }
    let idx = parts[0].parse::<usize>().map_err(|e| e.to_string())?;
    let coords: Vec<&str> = parts[1].split(',').collect();
    if coords.len() != 2 {
        return Err(format!("Invalid coords format, expected X,Y, got {}", parts[1]));
    }
    let x = coords[0].parse::<f32>().map_err(|e| e.to_string())?;
    let y = coords[1].parse::<f32>().map_err(|e| e.to_string())?;
    Ok((idx, x, y))
}

fn assert_layout_nth_pos(output: &RenderOutput, idx: usize, expected_x: f32, expected_y: f32) -> Result<(), String> {
    match output.layout.root.nth_box(idx) {
        Some((x, y, _w, _h)) => {
            let tol = 1.0;
            if (x - expected_x).abs() > tol {
                return Err(format!("Box[{idx}] x={x:.2} expected={expected_x:.2} (tol={tol})"));
            }
            if (y - expected_y).abs() > tol {
                return Err(format!("Box[{idx}] y={y:.2} expected={expected_y:.2} (tol={tol})"));
            }
            Ok(())
        }
        None => Err(format!(
            "Box[{idx}] not found (only {} boxes)",
            output.layout.root.count_boxes()
        )),
    }
}

fn parse_layout_nth_float_ge(name: &str, prefix: &str) -> Result<(usize, f32), String> {
    let rest = name.strip_prefix(prefix).ok_or("bad prefix")?;
    let parts: Vec<&str> = rest.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid format, expected {prefix}IDX:N, got {name}"));
    }
    let idx = parts[0].parse::<usize>().map_err(|e| e.to_string())?;
    let min_val = parts[1].parse::<f32>().map_err(|e| e.to_string())?;
    Ok((idx, min_val))
}

fn assert_layout_nth_width_ge(output: &RenderOutput, idx: usize, min_w: f32) -> Result<(), String> {
    match output.layout.root.nth_box(idx) {
        Some((_x, _y, w, _h)) => {
            if w >= min_w {
                Ok(())
            } else {
                Err(format!("Box[{idx}] width={w:.2} (expected >= {min_w})"))
            }
        }
        None => Err(format!("Box[{idx}] not found")),
    }
}

fn assert_layout_nth_height_ge(output: &RenderOutput, idx: usize, min_h: f32) -> Result<(), String> {
    match output.layout.root.nth_box(idx) {
        Some((_x, _y, _w, h)) => {
            if h >= min_h {
                Ok(())
            } else {
                Err(format!("Box[{idx}] height={h:.2} (expected >= {min_h})"))
            }
        }
        None => Err(format!("Box[{idx}] not found")),
    }
}

fn assert_fill_count(output: &RenderOutput, expected: usize) -> Result<(), String> {
    let count = output.primitives.fills.len();
    if count == expected {
        Ok(())
    } else {
        Err(format!("fill count={count} (expected {expected})"))
    }
}

fn assert_fill_count_ge(output: &RenderOutput, min: usize) -> Result<(), String> {
    let count = output.primitives.fills.len();
    if count >= min {
        Ok(())
    } else {
        Err(format!("fill count={count} (expected >= {min})"))
    }
}

fn assert_glyph_count_ge(output: &RenderOutput, min: usize) -> Result<(), String> {
    let count = output.primitives.glyphs.len();
    if count >= min {
        Ok(())
    } else {
        Err(format!("glyph count={count} (expected >= {min})"))
    }
}

fn assert_stroke_count_ge(output: &RenderOutput, min: usize) -> Result<(), String> {
    let count = output.primitives.strokes.len();
    if count >= min {
        Ok(())
    } else {
        Err(format!("stroke count={count} (expected >= {min})"))
    }
}

fn assert_gradient_count_ge(output: &RenderOutput, min: usize) -> Result<(), String> {
    let count = output.primitives.gradients.len();
    if count >= min {
        Ok(())
    } else {
        Err(format!("gradient count={count} (expected >= {min})"))
    }
}

fn assert_image_count_ge(output: &RenderOutput, min: usize) -> Result<(), String> {
    let count = output.primitives.images.len();
    if count >= min {
        Ok(())
    } else {
        Err(format!("image count={count} (expected >= {min})"))
    }
}

fn assert_shadow_count_ge(output: &RenderOutput, min: usize) -> Result<(), String> {
    let count = output.primitives.shadows.len();
    if count >= min {
        Ok(())
    } else {
        Err(format!("shadow count={count} (expected >= {min})"))
    }
}

fn assert_total_primitive_count_ge(output: &RenderOutput, min: usize) -> Result<(), String> {
    let count = output.primitives.len();
    if count >= min {
        Ok(())
    } else {
        Err(format!("total primitive count={count} (expected >= {min})"))
    }
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
    run_all_with_expectations(cases, ctx, &TestExpectations::new())
}

/// 运行所有给定的测试用例（带预期元数据），返回结果列表。
#[allow(dead_code)]
pub fn run_all_with_expectations(
    cases: &[TestCase],
    ctx: &TestContext,
    expectations: &TestExpectations,
) -> Vec<TestResult> {
    cases
        .iter()
        .map(|case| run_single_with_expectations(case, ctx, expectations))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_tests_count() {
        let tests = builtin_tests();
        assert!(
            tests.len() >= 500,
            "Should have at least 500 builtin tests, got {}",
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
        assert!(result.passed(), "Expected pass, got: {}", result.message);
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
        assert!(!result.passed());
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
            "geometry",
            "web-api",
            "security",
            "runtime",
            "a11y-i18n",
            "interactive",
            "typography",
            "navigation",
            "js-dom",
            "es-modules",
            "html-layout",
            "multiprocess",
            "css-layout-subset",
            "accessibility",
            "platform-input",
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
