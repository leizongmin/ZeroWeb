//! WPT testharness runner and minimal testdriver adapter for HTML interactions.

use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use zero_page_runtime::{HtmlActionRequest, HtmlUserAction};
use zero_webview::{WebView, WebViewConfig};

const CASE_TIMEOUT: Duration = Duration::from_secs(10);

/// First supported upstream HTML interaction cases.
pub const HTML_INTERACTION_CASES: &[&str] = &[
    "html/semantics/embedded-content/media-elements/networkState_initial.html",
    "html/semantics/embedded-content/media-elements/readyState_initial.html",
    "html/semantics/embedded-content/media-elements/interfaces/HTMLElement/HTMLTrackElement/readyState.html",
    "html/semantics/forms/the-output-element/output.html",
    "html/semantics/forms/the-input-element/input-whitespace.html",
    "html/interaction/focus/sequential-focus-navigation-and-the-tabindex-attribute/focus-tabindex-default-value.html",
    "uievents/constructors/inputevent-constructor.html",
];

/// Canvas 2D 专项（docs/goal/canvas-2d.md）M1 切片 1 导入的目录面。
///
/// 由 `scripts/fetch-canvas-subset.sh` 维护；新目录随切片扩展追加。
pub const CANVAS_TEST_SUBDIRS: &[&str] = &[
    "html/canvas/element/the-canvas-state",
    "html/canvas/element/drawing-rectangles-to-the-canvas",
    "html/canvas/element/transformations",
    "html/canvas/element/pixel-manipulation",
    "html/canvas/element/line-styles",
    "html/canvas/element/shadows",
    "html/canvas/element/compositing",
    "html/canvas/element/fill-and-stroke-styles",
    "html/canvas/element/text",
];

/// canvas-tests.js 的 WPT 内路径（prepare 时内联替换）。
const CANVAS_TESTS_JS_PATH: &str = "html/canvas/resources/canvas-tests.js";

/// DOM 专项（docs/goal/js-dom.md，M4 / DC-3）导入的上游 `dom/` 子目录面。
///
/// 由 `tests/wpt-runner/scripts/fetch-dom-subset.sh` 维护（wpt-data gitignored，
/// 用例按需 fetch、不入库）；新子目录随 M4 切片扩展追加。dom 用例只需
/// `resources/testharness.js`（runner 内联），不依赖 canvas-tests.js。
pub const DOM_TEST_SUBDIRS: &[&str] = &["dom/nodes"];

/// WPT subtest status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HarnessStatus {
    /// The subtest passed.
    Pass,
    /// The subtest failed.
    Fail,
    /// The case did not complete before the wall-clock deadline.
    Timeout,
    /// The case requires a testdriver API outside the declared support surface.
    Unsupported,
}

/// One WPT subtest result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HarnessSubtestResult {
    /// Subtest name.
    pub name: String,
    /// Stable result status.
    pub status: HarnessStatus,
    /// Optional assertion or infrastructure message.
    pub message: Option<String>,
}

/// Run the selected upstream HTML interaction cases under `wpt_root`.
pub fn run_html_interaction_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_path = wpt_root.join("resources/testharness.js");
    let harness_source = match std::fs::read_to_string(&harness_path) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                harness_path.display().to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    HTML_INTERACTION_CASES
        .iter()
        .filter(|path| filter.is_none_or(|filter| path.contains(filter)))
        .map(|path| {
            let source = std::fs::read_to_string(wpt_root.join(path));
            let results = match source {
                Ok(source) => run_testharness_html(wpt_root, path, &source, &harness_source, CASE_TIMEOUT),
                Err(error) => vec![HarnessSubtestResult {
                    name: "load WPT case".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            };
            ((*path).to_string(), results)
        })
        .collect()
}

/// Run the upstream `html/canvas` testharness cases under `wpt_root` (Canvas 2D goal M1).
///
/// 扫描 [`CANVAS_TEST_SUBDIRS`] 下全部主线程 .html 用例；`canvas-tests.js`（用例的
/// `_addTest` 驱动框架）与 testharness.js 一样内联执行。filter 按路径子串过滤。
pub fn run_canvas_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match std::fs::read_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    let canvas_tests_source = match std::fs::read_to_string(wpt_root.join(CANVAS_TESTS_JS_PATH)) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                CANVAS_TESTS_JS_PATH.to_string(),
                vec![HarnessSubtestResult {
                    name: "load canvas-tests.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    let mut cases = Vec::new();
    for subdir in CANVAS_TEST_SUBDIRS {
        let dir = wpt_root.join(subdir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "html") {
                continue;
            }
            let relative = format!("{}/{}", subdir, entry.file_name().to_string_lossy());
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let source = match std::fs::read_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    cases.push((
                        relative.clone(),
                        vec![HarnessSubtestResult {
                            name: "load WPT case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    ));
                    continue;
                }
            };
            let results = run_canvas_testharness_html(
                wpt_root,
                &relative,
                &source,
                &harness_source,
                &canvas_tests_source,
                CASE_TIMEOUT,
            );
            cases.push((relative, results));
        }
    }
    cases
}

/// Run the upstream `dom/` testharness cases under `wpt_root`（JS/DOM nativization goal M4 / DC-3）。
///
/// 扫描 [`DOM_TEST_SUBDIRS`] 下全部主线程 .html 用例；仅依赖 `testharness.js`（与
/// [`run_html_interaction_cases`] 同一底层 [`run_testharness_html`]，不经 canvas-tests.js）。
/// filter 按路径子串过滤。用例由 `fetch-dom-subset.sh` 按需拉取（wpt-data gitignored）。
pub fn run_dom_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match std::fs::read_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                "resources/testharness.js".to_string(),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };

    let mut cases = Vec::new();
    for subdir in DOM_TEST_SUBDIRS {
        let dir = wpt_root.join(subdir);
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "html") {
                continue;
            }
            let relative = format!("{}/{}", subdir, entry.file_name().to_string_lossy());
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let source = match std::fs::read_to_string(&path) {
                Ok(source) => source,
                Err(error) => {
                    cases.push((
                        relative.clone(),
                        vec![HarnessSubtestResult {
                            name: "load WPT case".into(),
                            status: HarnessStatus::Fail,
                            message: Some(error.to_string()),
                        }],
                    ));
                    continue;
                }
            };
            let results = run_testharness_html(wpt_root, &relative, &source, &harness_source, CASE_TIMEOUT);
            cases.push((relative, results));
        }
    }
    cases
}

/// Run one canvas testharness case with `canvas-tests.js` inlined.
fn run_canvas_testharness_html(
    wpt_root: &Path,
    case_name: &str,
    source: &str,
    harness_source: &str,
    canvas_tests_source: &str,
    timeout: Duration,
) -> Vec<HarnessSubtestResult> {
    let inline_extras = [(CANVAS_TESTS_JS_PATH, canvas_tests_source)];
    run_testharness_html_inner(wpt_root, case_name, source, harness_source, &inline_extras, timeout)
}

/// Run one HTML testharness case with the declared click/send_keys adapter.
pub fn run_testharness_html(
    wpt_root: &Path,
    case_name: &str,
    source: &str,
    harness_source: &str,
    timeout: Duration,
) -> Vec<HarnessSubtestResult> {
    run_testharness_html_inner(wpt_root, case_name, source, harness_source, &[], timeout)
}

/// R34xx：headless 图片源获取器——`https://wpt.test/<path>`（wpt-data 相对路径）→
/// `wpt_root/<path>` 本地文件读取（PNG 等解码由 webview decode_image 完成）。
fn wpt_data_image_fetcher(wpt_root: &std::path::Path) -> Option<zero_webview::ImageSourceFetcher> {
    let root = wpt_root.to_path_buf();
    Some(std::sync::Arc::new(move |url: &str| {
        // 仅 wpt.test 域名（测试资源）；其他 URL 回退网络。
        let path_part = url.strip_prefix("https://wpt.test")?;
        let path_part = path_part.strip_prefix('/').unwrap_or(path_part);
        // 去查询串/片段。
        let clean = path_part.split(['?', '#']).next()?;
        if clean.is_empty() {
            return None;
        }
        std::fs::read(root.join(clean)).ok()
    }))
}

fn run_testharness_html_inner(
    wpt_root: &Path,
    case_name: &str,
    source: &str,
    harness_source: &str,
    inline_extras: &[(&str, &str)],
    timeout: Duration,
) -> Vec<HarnessSubtestResult> {
    let unsupported = unsupported_testdriver_dependencies(source);
    if !unsupported.is_empty() {
        return vec![HarnessSubtestResult {
            name: case_name.to_string(),
            status: HarnessStatus::Unsupported,
            message: Some(format!("unsupported testdriver API: {}", unsupported.join(", "))),
        }];
    }

    let html = prepare_harness_html(source, harness_source, inline_extras, wpt_root, case_name);
    let scripts = zero_engine::extract_page_scripts(&html);
    let script_lengths = scripts
        .iter()
        .map(|script| match script {
            zero_engine::PageScript::Inline(source) | zero_engine::PageScript::InlineModule(source) => source.len(),
            zero_engine::PageScript::External(_) | zero_engine::PageScript::ExternalModule(_) => 0,
        })
        .collect::<Vec<_>>();
    // js-dom goal DC-3「native 路径对照」：env `ZW_NATIVE_DOM=1` 时 runner 走原生绑定路径
    //（WebViewConfig.native_dom=true），而非默认 polyfill 字符串桥。用于建立 native 通过率
    // 基线，让 R2/R3/R4 native 修复（classList/createElement/node mutation DOMException）的基线
    // 价值可见。env 进程级（testharness 一次跑一个路径，无混跑）。
    let native_dom = std::env::var("ZW_NATIVE_DOM").as_deref() == Ok("1");
    let mut webview = WebView::new(WebViewConfig {
        width: 800,
        height: 600,
        native_dom,
        // R34xx：headless 图片源——wpt.test/images/* 映射到本地 wpt-data 目录
        //（testharness 无网络；G5 DOM img 源解锁依赖图片加载）。
        // js-dom goal：dom 用例同样需要本地 .js 内联 + 图片资源，两条路径统一走 wpt_root。
        image_source_fetcher: wpt_data_image_fetcher(wpt_root),
        ..WebViewConfig::default()
    });
    webview.prepare_document_state(&format!("https://wpt.test/{case_name}"));
    let page_url = format!("https://wpt.test/{case_name}");
    let external_css = webview.fetch_page_images(&html, &page_url);
    webview.load_html(&html, Some(&external_css));
    if let Err(error) = webview.run_page_scripts_strict() {
        return vec![HarnessSubtestResult {
            name: case_name.to_string(),
            status: HarnessStatus::Fail,
            message: Some(format!("page script threw: {error}")),
        }];
    }

    let deadline = Instant::now() + timeout;
    let mut partial_results = Vec::new();
    let mut last_test_function = "unknown".to_string();
    let mut last_harness_hook = "unknown".to_string();
    let mut last_state = serde_json::Value::Null;
    loop {
        if Instant::now() >= deadline {
            let mut results = map_harness_results(partial_results);
            results.push(HarnessSubtestResult {
                name: case_name.to_string(),
                status: HarnessStatus::Timeout,
                message: Some(format!(
                    "testharness completion callback was not called (test={}, hook={}, scripts={script_lengths:?}, state={last_state})",
                    last_test_function, last_harness_hook
                )),
            });
            return results;
        }

        let probe = match take_probe(&mut webview) {
            Ok(probe) => probe,
            Err(error) => {
                return vec![HarnessSubtestResult {
                    name: case_name.to_string(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }];
            }
        };
        partial_results = probe.results;
        last_test_function = probe.test_function;
        last_harness_hook = probe.harness_hook;
        last_state = probe.state;
        for command in probe.commands {
            let result = apply_testdriver_command(&mut webview, &command);
            if let Err(error) = resolve_testdriver_command(&mut webview, command.id, result.as_deref()) {
                return vec![HarnessSubtestResult {
                    name: case_name.to_string(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }];
            }
        }
        if probe.complete {
            return map_harness_results(partial_results);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn map_harness_results(results: Vec<RawHarnessResult>) -> Vec<HarnessSubtestResult> {
    results
        .into_iter()
        .map(|result| HarnessSubtestResult {
            name: result.name,
            status: match result.status {
                0 => HarnessStatus::Pass,
                2 => HarnessStatus::Timeout,
                _ => HarnessStatus::Fail,
            },
            message: result.message,
        })
        .collect()
}

fn prepare_harness_html(
    source: &str,
    harness_source: &str,
    inline_extras: &[(&str, &str)],
    wpt_root: &Path,
    case_path: &str,
) -> String {
    let reporter = r#"
if (typeof setup === 'function') setup({output: false});
globalThis.__zw_harness_results = [];
globalThis.__zw_harness_complete = false;
add_result_callback(function(test) {
  globalThis.__zw_harness_results.push({
    name: String(test.name || ''),
    status: Number(test.status),
    message: test.message == null ? null : String(test.message)
  });
});
add_completion_callback(function() {
  globalThis.__zw_harness_complete = true;
});
"#;
    let harness_source = harness_source.replacen(
        "\n})(self);",
        "\nglobal_scope.__zw_mark_harness_loaded = function() {\n\
         test_environment.all_loaded = true;\n\
         if (tests.all_done()) tests.complete();\n\
         };\n\
         global_scope.__zw_harness_state = function() {\n\
         return {tests:tests.tests.length,pending:tests.num_pending,loaded:test_environment.all_loaded,phase:tests.phase};\n\
         };\n})(self);",
        1,
    );
    let harness = format!(
        "<script>\n\
         globalThis.__zw_setTimeout = function() {{}};\n\
         globalThis.__zw_clearTimeout = function() {{}};\n\
         {harness_source}\n{reporter}\n</script>"
    );
    let mut html = replace_script_source(source, "/resources/testharness.js", &harness);
    html = replace_script_source(&html, "/resources/testharnessreport.js", "");
    html = replace_script_source(&html, "/resources/testdriver.js", TESTDRIVER_STUB);
    html = replace_script_source(&html, "/resources/testdriver-vendor.js", "");
    html = replace_script_source(&html, "/resources/testdriver-actions.js", "");
    // canvas-tests.js 等用例框架脚本：与 testharness.js 同款内联（外部脚本提取器不加载 src）。
    for (script_src, inline_source) in inline_extras {
        html = replace_script_source(&html, script_src, &format!("<script>{inline_source}</script>"));
    }
    // js-dom goal：用例引用的本地 .js 测试体（如 <script src="attributes.js">、
    // <script src="Document-createProcessingInstruction.js">）——extract_page_scripts 不加载外部 src，
    // 故此处从 wpt-data 读文件内容内联。case_path 形如 "dom/nodes/attributes.html"，本地 .js 解析为
    // 同目录文件（相对 src 如 "attributes.js" / "./attributes.js" / "../constants.js"）。
    // 仅内联相对路径（非 /resources/、非 http(s):）；文件缺失则移除该 script 标签（不注入空）。
    html = inline_local_scripts(&html, wpt_root, case_path);
    html.push_str(
        "<script>\
         document.dispatchEvent(new Event('DOMContentLoaded'));\
         globalThis.dispatchEvent(new Event('load'));\
         if (typeof globalThis.__zw_mark_harness_loaded === 'function') {\
           globalThis.__zw_mark_harness_loaded();\
         }\
         </script>",
    );
    html
}

fn replace_script_source(source: &str, script_src: &str, replacement: &str) -> String {
    let mut output = String::with_capacity(source.len() + replacement.len());
    let mut remaining = source;
    loop {
        let Some(start) = remaining.find("<script") else {
            output.push_str(remaining);
            break;
        };
        output.push_str(&remaining[..start]);
        let candidate = &remaining[start..];
        let Some(open_end) = candidate.find('>') else {
            output.push_str(candidate);
            break;
        };
        let open = &candidate[..=open_end];
        let Some(close_offset) = candidate[open_end + 1..].find("</script>") else {
            output.push_str(candidate);
            break;
        };
        let end = open_end + 1 + close_offset + "</script>".len();
        if open.contains(script_src) {
            output.push_str(replacement);
        } else {
            output.push_str(&candidate[..end]);
        }
        remaining = &candidate[end..];
    }
    output
}

/// 内联用例引用的本地 .js 测试体（js-dom goal R8）。
///
/// `extract_page_scripts` 不加载外部 `<script src>`，故用例引用的同目录 .js（如 attributes.js、
/// Document-createProcessingInstruction.js）或相对路径（../constants.js）不会执行 → `attr_is`/
/// 测试体 not defined。本函数扫描剩余的 `<script src="...">`（相对路径，非 /resources/、非 http），
/// 从 wpt-data 读文件内容内联为 inline `<script>`；文件缺失则移除该标签（best-effort，不注入空）。
///
/// `case_path` 形如 "dom/nodes/attributes.html"；相对 src 解析为相对该 case 所在目录。
fn inline_local_scripts(html: &str, wpt_root: &Path, case_path: &str) -> String {
    // case 所在目录（相对 wpt_root），如 "dom/nodes"。
    let case_dir = case_path.rsplit_once('/').map(|(d, _)| d).unwrap_or("");
    let mut output = String::with_capacity(html.len());
    let mut remaining = html;
    loop {
        let Some(start) = remaining.find("<script") else {
            output.push_str(remaining);
            break;
        };
        let Some(open_end) = remaining[start..].find('>') else {
            output.push_str(remaining);
            break;
        };
        let open_end = start + open_end;
        let open = &remaining[start..=open_end];
        // 提取 src="..."（仅相对路径 .js）。
        let src = extract_script_src(open);
        let resolved = src.and_then(|s| {
            // 仅相对路径（不以 / 开头、非 http(s):、非 // ）。
            if s.starts_with('/') || s.starts_with("http://") || s.starts_with("https://") || s.starts_with("//") {
                return None;
            }
            // 规范化 "./" 前缀 + 相对 case_dir 解析（含 ../ 上溯）。
            let rel = s.strip_prefix("./").unwrap_or(s);
            let combined = if case_dir.is_empty() {
                rel.to_string()
            } else {
                normalize_relative(&format!("{case_dir}/{rel}"))
            };
            std::fs::read_to_string(wpt_root.join(&combined))
                .ok()
                .map(|c| (combined, c))
        });
        let rest_start = open_end + 1;
        match resolved {
            Some((combined, content)) => {
                output.push_str(&remaining[..start]);
                output.push_str("<script data-inline=\"");
                output.push_str(&combined);
                output.push_str("\">");
                output.push_str(&content);
                output.push_str("</script>");
            }
            None => {
                // 非本地 .js（/resources/、http、或文件缺失）：保留原标签（extract_page_scripts 处理）。
                output.push_str(&remaining[..rest_start]);
            }
        }
        remaining = &remaining[rest_start..];
    }
    output
}

/// 从 `<script src="...">` 标签提取 src 值（单/双引号）。
fn extract_script_src(open_tag: &str) -> Option<&str> {
    let key = "src=\"";
    if let Some(i) = open_tag.find(key) {
        let after = &open_tag[i + key.len()..];
        return after.split('"').next();
    }
    let key = "src='";
    if let Some(i) = open_tag.find(key) {
        let after = &open_tag[i + key.len()..];
        return after.split('\'').next();
    }
    None
}

/// 规范化相对路径（处理 `..` 上溯，如 "dom/nodes/../constants.js" → "dom/constants.js"）。
fn normalize_relative(path: &str) -> String {
    let mut stack: Vec<&str> = Vec::new();
    for seg in path.split('/') {
        match seg {
            "" | "." => {}
            ".." => {
                stack.pop();
            }
            s => stack.push(s),
        }
    }
    stack.join("/")
}

fn unsupported_testdriver_dependencies(source: &str) -> Vec<String> {
    let mut dependencies = Vec::new();
    let mut remaining = source;
    while let Some(index) = remaining.find("test_driver.") {
        let after = &remaining[index + "test_driver.".len()..];
        let name: String = after
            .chars()
            .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
            .collect();
        let name_len = name.len();
        if !name.is_empty() && name != "click" && name != "send_keys" && !dependencies.contains(&name) {
            dependencies.push(name);
        }
        remaining = after.get(name_len..).unwrap_or_default();
    }
    dependencies
}

#[derive(Deserialize)]
struct HarnessProbe {
    complete: bool,
    results: Vec<RawHarnessResult>,
    commands: Vec<TestdriverCommand>,
    test_function: String,
    harness_hook: String,
    state: serde_json::Value,
}

#[derive(Deserialize)]
struct RawHarnessResult {
    name: String,
    status: u8,
    message: Option<String>,
}

#[derive(Deserialize)]
struct TestdriverCommand {
    id: u64,
    operation: String,
    selector: Option<String>,
    text: Option<String>,
}

fn take_probe(webview: &mut WebView) -> Result<HarnessProbe, String> {
    let value = webview
        .execute_script(
            "JSON.stringify({\
             complete:!!globalThis.__zw_harness_complete,\
             results:globalThis.__zw_harness_results||[],\
             test_function:typeof globalThis.test,\
             harness_hook:typeof globalThis.__zw_mark_harness_loaded,\
             state:typeof globalThis.__zw_harness_state==='function'?globalThis.__zw_harness_state():null,\
             commands:(globalThis.__zw_td_queue||[]).splice(0)})",
        )
        .map_err(|error| error.to_string())?;
    serde_json::from_str(&value).map_err(|error| format!("invalid harness probe: {error}: {value}"))
}

fn apply_testdriver_command(webview: &mut WebView, command: &TestdriverCommand) -> Option<String> {
    let Some(selector) = command.selector.as_deref() else {
        return Some("testdriver target has no stable selector".into());
    };
    let target = match webview.page_node_ref_for_selector(selector) {
        Some(target) => target,
        None => return Some(format!("testdriver target not found: {selector}")),
    };
    match command.operation.as_str() {
        "click" => dispatch_action(webview, target, HtmlUserAction::Activate),
        "send_keys" => {
            let text = command.text.as_deref().unwrap_or_default();
            for character in text.chars() {
                let action = match character {
                    '\u{E003}' => HtmlUserAction::DeleteBackward,
                    '\u{E004}' => HtmlUserAction::MoveFocus { forward: true },
                    character if ('\u{E000}'..='\u{F8FF}').contains(&character) => {
                        return Some(format!("unsupported WebDriver key U+{:04X}", character as u32));
                    }
                    character => HtmlUserAction::InsertText {
                        text: character.to_string(),
                    },
                };
                if let Some(error) = dispatch_action(webview, target, action) {
                    return Some(error);
                }
            }
            None
        }
        operation => Some(format!("unsupported testdriver command: {operation}")),
    }
}

fn dispatch_action(
    webview: &mut WebView,
    target: zero_page_runtime::PageNodeRef,
    action: HtmlUserAction,
) -> Option<String> {
    match webview.dispatch_loaded_page_user_action(HtmlActionRequest {
        target,
        action,
        shift: false,
    }) {
        Ok(result) if result.noop_reason.is_none() => None,
        Ok(result) => Some(format!("action was not applicable: {:?}", result.noop_reason)),
        Err(error) => Some(error.to_string()),
    }
}

fn resolve_testdriver_command(webview: &mut WebView, id: u64, error: Option<&str>) -> Result<(), String> {
    let error = serde_json::to_string(&error).map_err(|error| error.to_string())?;
    webview
        .execute_script(&format!("globalThis.__zw_td_resolve({id},{error})"))
        .map(|_| ())
        .map_err(|error| error.to_string())
}

const TESTDRIVER_STUB: &str = r#"<script>
(function() {
  var nextId = 1;
  var pending = {};
  globalThis.__zw_td_queue = [];
  function selectorFor(element) {
    if (!element) return null;
    var id = element.getAttribute && element.getAttribute('id');
    if (id) return '#' + id;
    var tag = String(element.tagName || '').toLowerCase();
    if (!tag) return null;
    var matches = document.querySelectorAll(tag);
    return matches.length === 1 ? tag : null;
  }
  function enqueue(operation, element, text) {
    return new Promise(function(resolve, reject) {
      var id = nextId++;
      pending[id] = { resolve: resolve, reject: reject };
      globalThis.__zw_td_queue.push({
        id: id, operation: operation, selector: selectorFor(element),
        text: text == null ? null : String(text)
      });
    });
  }
  globalThis.__zw_td_resolve = function(id, error) {
    var entry = pending[id];
    if (!entry) return;
    delete pending[id];
    if (error == null) entry.resolve();
    else entry.reject(new Error(String(error)));
  };
  globalThis.test_driver = {
    click: function(element) { return enqueue('click', element, null); },
    send_keys: function(element, keys) { return enqueue('send_keys', element, keys); }
  };
})();
</script>"#;

#[cfg(test)]
mod tests {
    use super::*;

    const MINI_HARNESS: &str = r#"
var __resultCallbacks = [], __completionCallbacks = [], __pending = 0;
globalThis.add_result_callback = function(cb) { __resultCallbacks.push(cb); };
globalThis.add_completion_callback = function(cb) { __completionCallbacks.push(cb); };
function __emit(t) { __resultCallbacks.forEach(function(cb){ cb(t); }); }
function __completeSoon() {
  Promise.resolve().then(function() {
    if (__pending === 0) __completionCallbacks.forEach(function(cb){ cb([]); });
  });
}
globalThis.test = function(fn, name) {
  var t = {name:name, status:0, message:null};
  try { fn(); } catch (e) { t.status=1; t.message=String(e); }
  __emit(t); __completeSoon();
};
globalThis.promise_test = function(fn, name) {
  __pending++;
  Promise.resolve().then(fn).then(function() {
    __emit({name:name,status:0,message:null});
  }, function(e) {
    __emit({name:name,status:1,message:String(e)});
  }).then(function(){ __pending--; __completeSoon(); });
};
globalThis.assert_equals = function(a,b,m) { if (a !== b) throw new Error(m || (String(a)+' != '+String(b))); };
"#;

    #[test]
    fn runs_supported_html_interaction_subtests() {
        let html = r##"
<script src="/resources/testharness.js"></script>
<script src="/resources/testdriver.js"></script>
<input id="name">
<input id="check" type="checkbox">
<script>
promise_test(async function() {
  var input = document.getElementById('name');
  await test_driver.send_keys(input, 'ab');
  assert_equals(input.value, 'ab');
}, 'send keys updates the live input');
promise_test(async function() {
  var input = document.getElementById('check');
  await test_driver.click(input);
  assert_equals(input.checked, true);
}, 'click updates live checkedness');
</script>
"##;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "local-supported.html",
            html,
            MINI_HARNESS,
            Duration::from_secs(2),
        );
        assert_eq!(
            results,
            vec![
                HarnessSubtestResult {
                    name: "send keys updates the live input".into(),
                    status: HarnessStatus::Pass,
                    message: None,
                },
                HarnessSubtestResult {
                    name: "click updates live checkedness".into(),
                    status: HarnessStatus::Pass,
                    message: None,
                }
            ]
        );
    }

    #[test]
    fn unsupported_testdriver_command_is_explicit() {
        let html = "test_driver.set_permission({name:'clipboard-read'}, 'granted')";
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "unsupported.html",
            html,
            MINI_HARNESS,
            Duration::from_secs(1),
        );
        assert_eq!(results[0].status, HarnessStatus::Unsupported);
        assert!(results[0].message.as_deref().unwrap().contains("set_permission"));
    }

    #[test]
    fn missing_harness_completion_is_timeout() {
        let html = r#"<script src="/resources/testharness.js"></script>"#;
        let results = run_testharness_html(
            Path::new("/nonexistent-wpt-root-for-tests"),
            "timeout.html",
            html,
            "function add_result_callback(){} function add_completion_callback(){}",
            Duration::from_millis(10),
        );
        assert_eq!(results[0].status, HarnessStatus::Timeout);
    }
}
