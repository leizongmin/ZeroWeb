//! wptfyi-html goal（DC-1）—— `html/the-xhtml-syntax` XHTML 实体矩阵 testharness
//! window 可执行通道。
//!
//! 独立模块（2000 行约束 + testharness_html_semantics 先例），经 `pub(crate)` 复用
//! [`crate::testharness`] 执行底座（`run_testharness_html` / `corpus_case_timeout`）。
//!
//! 用例由 `scripts/goals/55-wptfyi-html.sh` 显式清单拉到
//! `wpt-data/html/the-xhtml-syntax/`（gitignored，WPT pin
//! 315976933870b34d6ea30e3f6643403edae678ba）。计分面 = 12 个实体矩阵外层
//! `.htm` 页（25,606 子测试——wpt.fyi aligned 三家 summary 并集口径，见
//! coverage-2026-11-20 `html/the-xhtml-syntax` 行）：外层页 iframe 载入
//! `support/xhtml-mathml-dtd-entity.htm`，support 页 XHR 拉 `entities.json`
//! （2,231 项命名实体）后逐实体 `DOMParser.parseFromString(doctype+串, mime)`
//! 断言 Text 节点 charCode；application/xhtml+xml 走真实 MIME 的 XML 文档
//! DOMParser 路径，doctype systemId 恒 `"foo"`（不存在的相对 URL——外部实体
//! 不得发起网络取回的安全边界锚）。
//!
//! 零执行记账面（与 fetch 脚本头注释同源）：
//! - `support/` 助资产目录（机器页 / 实体表 / CSS，不按案跑）
//! - rel=match reftest 形（data-xhtml-with-dtd.html、adopt-while-parsing-001.html
//!   及其 ref、adopt-while-parsing.xhtml）——本口径 reftest 无子测试不进分母
//! - `.xhtml` testharness 形（xml-stylesheet-blocking.xhtml）——runner 无 XML 文档
//!   载入管道（真实 MIME 文档路径），FIXME(wptfyi-html DC-2) 门控挂账

use crate::testharness::{HarnessSubtestResult, corpus_case_timeout, run_testharness_html};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// the-xhtml-syntax corpus 扫描根目录（上游该顶级目录唯一子树 parsing-xhtml-documents）。
pub const XHTML_SYNTAX_TEST_SUBDIR: &str = "html/the-xhtml-syntax";

/// [`XHTML_SYNTAX_TEST_SUBDIR`] 用例排除规则。`relative` 为 wpt-data 相对路径，
/// `source` 为用例原文。
pub(crate) fn xhtml_syntax_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    // 助资产与 reftest 参照页（html-semantics 通道同名先例）
    if name.ends_with("-ref.html") || name.ends_with("-notref.html") || relative.contains("/support/") {
        return true;
    }
    // rel=match reftest 形——无子测试分母贡献（coverage 口径 c=[0,0]）
    if source.contains("rel=match") {
        return true;
    }
    // .xhtml 真文档形态——runner 无 XML MIME 文档载入管道，DC-2 门控挂账
    // （xml-stylesheet-blocking.xhtml 唯一案； FIXME(wptfyi-html DC-2)）
    if name.ends_with(".xhtml") {
        return true;
    }
    // 无 testharness 引用页（reftest 主体页等非 testharness 形态，html-syntax 先例）
    if !source.contains("testharness") {
        return true;
    }
    false
}

/// Run the pinned upstream the-xhtml-syntax entity-matrix subset（wptfyi-html DC-1）。
/// filter 按路径子串过滤（如 `entity-10`）。
pub fn run_xhtml_syntax_cases(
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
    collect_xhtml_syntax_cases(wpt_root, XHTML_SYNTAX_TEST_SUBDIR, filter, &mut specs);
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

/// 递归收集可执行用例规格（.htm/.html testharness 形态；testharness_html_semantics
/// 先例——`.htm` 为 the-xhtml-syntax 实体矩阵上游扩展名）。
fn collect_xhtml_syntax_cases(
    wpt_root: &Path,
    subdir: &str,
    filter: Option<&str>,
    specs: &mut Vec<XhtmlSyntaxCaseSpec>,
) {
    let dir = wpt_root.join(subdir);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let nested = format!("{subdir}/{}", entry.file_name().to_string_lossy());
            collect_xhtml_syntax_cases(wpt_root, &nested, filter, specs);
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !file_name.ends_with(".html") && !file_name.ends_with(".htm") {
            continue;
        }
        let relative = format!("{subdir}/{file_name}");
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let Ok(source) = read_corpus_to_string(&path) else {
            continue;
        };
        if xhtml_syntax_case_skipped(&relative, &source) {
            continue;
        }
        specs.push(XhtmlSyntaxCaseSpec {
            case_name: relative,
            source,
        });
    }
}

/// 单案执行规格（testharness_html_semantics 的 HtmlSemanticsCaseSpec 简形——
/// 本通道无 .window.js/.any.js 形态，全走 [`run_testharness_html`]）。
struct XhtmlSyntaxCaseSpec {
    case_name: String,
    source: String,
}

use crate::testharness::HarnessStatus;

#[cfg(test)]
mod tests {
    use super::*;

    // 实体矩阵外层页可跑；support 资产与 reftest 形排除。
    #[test]
    fn entity_matrix_pages_run_and_assets_skip() {
        let outer = r#"
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<iframe id="test" src="support/xhtml-mathml-dtd-entity.htm"></iframe>
"#;
        assert!(!xhtml_syntax_case_skipped(
            "html/the-xhtml-syntax/parsing-xhtml-documents/xhtml-mathml-dtd-entity-1.htm",
            outer,
        ));
        assert!(xhtml_syntax_case_skipped(
            "html/the-xhtml-syntax/parsing-xhtml-documents/support/xhtml-mathml-dtd-entity.htm",
            "",
        ));
        assert!(xhtml_syntax_case_skipped(
            "html/the-xhtml-syntax/parsing-xhtml-documents/data-xhtml-with-dtd-ref.html",
            "<link rel=match href=x>",
        ));
        // .xhtml testharness 形（真 MIME 文档面）——DC-2 门控排除
        assert!(xhtml_syntax_case_skipped(
            "html/the-xhtml-syntax/parsing-xhtml-documents/xml-stylesheet-blocking.xhtml",
            "<script src=\"/resources/testharness.js\"/>",
        ));
    }
}
