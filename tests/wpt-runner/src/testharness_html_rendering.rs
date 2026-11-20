//! wptfyi-html goal（DC-2 批二）—— `html/rendering` UA 样式/呈现提示 testharness
//! window 可执行通道。
//!
//! 独立模块（2000 行约束 + testharness_xhtml_syntax 先例），经 `pub(crate)` 复用
//! [`crate::testharness`] 执行底座（`run_testharness_html` / `corpus_case_timeout`）。
//!
//! 语料由 `scripts/goals/56-wptfyi-html-rendering.sh` 拉到
//! `wpt-data/html/rendering/`（gitignored，WPT pin 315976933870b34d6ea30e3f6643403edae678ba）。
//! 计分面 = 目录内 testharness 形 `.html` 页（11-20 快照口径 html/rendering
//! 11,705 子测试 / 158 计分文件；reftest 主体页无子测试不进分母）。断言主力是
//! UA 样式表与呈现提示的 getComputedStyle 对照（support/test-ua-stylesheet.js
//! 共享 helper——test 元素 vs fake clone ref 逐 CSS 属性比对）。
//!
//! 零执行记账面（与 fetch 脚本头注释同源）：
//! - `support/` `resources/` `reference/` `crashtests/` 助资产目录（helper 脚本 /
//!   参照页 / 崩溃回归页，不按案跑）
//! - `-ref.html` / `-notref.html` reftest 参照页（html-semantics/html-syntax 同名先例）
//! - `.xhtml` 形（cdata-001/002.xhtml 等 20 文件）——runner 无 XML 文档载入管道
//!   （FIXME(wptfyi-html DC-2+)，xhtml-syntax 通道同款门控）
//! - 无 `testharness` 引用页（reftest 主体页——本口径 reftest 无子测试不进分母）

use crate::testharness::{HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_testharness_html};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// rendering corpus 扫描根目录（wpt-data 相对路径）。
pub const HTML_RENDERING_TEST_SUBDIR: &str = "html/rendering";

/// [`HTML_RENDERING_TEST_SUBDIR`] 用例排除规则。`relative` 为 wpt-data 相对路径，
/// `source` 为用例原文。
pub(crate) fn html_rendering_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    // 助资产目录（helper / 参照页 / 崩溃回归页——fetch 全拉保语料完整，通道零执行）
    if relative.contains("/support/")
        || relative.contains("/resources/")
        || relative.contains("/reference/")
        || relative.contains("/crashtests/")
    {
        return true;
    }
    // reftest 参照页（html-semantics/html-syntax 通道同名先例）
    if name.ends_with("-ref.html") || name.ends_with("-notref.html") {
        return true;
    }
    // .xhtml 真文档形态——runner 无 XML MIME 文档载入管道（DC-1 xhtml-syntax 同款门控，
    // FIXME(wptfyi-html DC-2+)）
    if name.ends_with(".xhtml") {
        return true;
    }
    // 无 testharness 引用页（reftest 主体页等非 testharness 形态——本口径 reftest
    // 无子测试不进分母，cdata/frameset 参照页等）
    if !source.contains("testharness") {
        return true;
    }
    false
}

/// Run the pinned upstream html/rendering window subset（wptfyi-html DC-2 批二）。
/// filter 按路径子串过滤（如 `lists`、`dimension-attributes`）。
pub fn run_html_rendering_cases(
    wpt_root: &Path,
    filter: Option<&str>,
    jobs: usize,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
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
    let mut specs = Vec::new();
    collect_html_rendering_cases(wpt_root, HTML_RENDERING_TEST_SUBDIR, filter, &mut specs);
    let mut cases = crate::parallel_map(&specs, jobs, |spec| {
        let results = run_testharness_html(
            wpt_root,
            &spec.case_name,
            &spec.source,
            harness_source.as_str(),
            corpus_case_timeout(),
        );
        (spec.case_name.clone(), results)
    });
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}

/// 递归收集可执行用例规格（.html testharness 形态）。
fn collect_html_rendering_cases(
    wpt_root: &Path,
    subdir: &str,
    filter: Option<&str>,
    specs: &mut Vec<HtmlRenderingCaseSpec>,
) {
    let dir = wpt_root.join(subdir);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let nested = format!("{subdir}/{}", entry.file_name().to_string_lossy());
            collect_html_rendering_cases(wpt_root, &nested, filter, specs);
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.ends_with(".html") {
            continue;
        }
        let relative = format!("{subdir}/{file_name}");
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let Ok(source) = read_corpus_to_string(&path) else {
            continue;
        };
        if html_rendering_case_skipped(&relative, &source) {
            continue;
        }
        specs.push(HtmlRenderingCaseSpec {
            case_name: relative,
            source,
        });
    }
}

/// 单案执行规格（xhtml-syntax 的 XhtmlSyntaxCaseSpec 同形简壳）。
struct HtmlRenderingCaseSpec {
    case_name: String,
    source: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // testharness 形 UA 样式断言页可跑；助资产目录、reftest 参照/主体页、
    // .xhtml 真文档排除。
    #[test]
    fn rendering_pages_run_and_assets_skip() {
        assert!(!html_rendering_case_skipped(
            "html/rendering/non-replaced-elements/lists/lists-styles.html",
            r#"<script src="/resources/testharness.js"></script>
<script src="/html/rendering/support/test-ua-stylesheet.js"></script>"#,
        ));
        assert!(html_rendering_case_skipped(
            "html/rendering/support/test-ua-stylesheet.js",
            "",
        ));
        assert!(html_rendering_case_skipped(
            "html/rendering/non-replaced-elements/the-hr-element-0/hr-ref.html",
            "",
        ));
        assert!(html_rendering_case_skipped(
            "html/rendering/cdata-001.xhtml",
            r#"<script src="/resources/testharness.js"/>"#,
        ));
        // reftest 主体页（无 testharness 引用）零执行记账
        assert!(html_rendering_case_skipped(
            "html/rendering/replaced-elements/the-select-element/select-1-inline-flex.html",
            "<link rel=match href=select-1-inline-flex-ref.html>",
        ));
    }
}
