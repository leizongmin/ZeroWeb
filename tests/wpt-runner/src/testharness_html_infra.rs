//! wptfyi-html goal（DC-2 批九）—— `html/infrastructure` + 小簇搭车
//! （obsolete/links/meta/scripting/select）testharness window 可执行通道。
//!
//! 独立模块（2000 行约束 + testharness_html_rendering 先例），经 `pub(crate)` 复用
//! [`crate::testharness`] 执行底座（`run_testharness_html` / `run_any_js_window_case` /
//! `corpus_case_timeout`）与 [`crate::testharness_workers`] dedicated 变体腿
//! （encoding 通道 worker_variants 先例）。
//!
//! 语料由 `scripts/goals/62-wptfyi-html-infra.sh` 拉到 `wpt-data/html/`（gitignored，
//! WPT pin 315976933870b34d6ea30e3f6643403edae678ba）。分母（summary-chrome
//! @4b26db8978）193 键 / 1,736 子测试：infrastructure 160 + obsolete 20 + links 7 +
//! meta 1 + scripting 3 + select 2。执行形态：
//! - `.html` 计分页（68 键）——variant meta 展开（js-dom R329 / html-semantics R3906
//!   先例：声明 variant 的用例只按矩阵跑、不再跑基础 URL，wpt.fyi 分母 variant
//!   URL 独立键同形）。query-encoding/ 9 基页 91 变体键（`?include=<面>` ×80 +
//!   `?encoding=<enc>` ×11）；编码变体页主文档/脚本按 wptserve per-request 模板
//!   注入语义替换 `{{GET[...]}}`（脚本侧 fetcher 既有同款，主文档侧
//!   [`crate::testharness::wpt_apply_get_query_substitutions`]——attributes.sub.html
//!   `<meta charset={{GET[encoding]}}>` 载体）
//! - `.window.js` window 变体源（15 键生成形 `.window.html`）与 `.any.js`（4 源 8 键
//!   生成形 `.any.html`/`.any.worker.html`）——any_js_window_wrapper /
//!   dedicated 变体腿（META script 依赖 wrapper 自动解析）
//!
//! 零执行记账面（fetch 脚本头注释同域，runner 侧镜像双保险）见
//! [`html_infra_case_skipped`] 文档。

use crate::testharness::{
    HarnessStatus, HarnessSubtestResult, apply_wpt_substitutions, case_variants, corpus_case_timeout,
    run_any_js_window_case, run_testharness_html, wpt_apply_get_query_substitutions,
};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// infrastructure + 小簇 corpus 扫描根目录（fetch 脚本 DIRS 同域双保险——递归
/// 扫描覆盖全部 depth-N 叶子）。
pub const HTML_INFRA_TEST_SUBDIRS: &[&str] = &[
    "html/infrastructure",
    "html/links",
    "html/meta",
    "html/obsolete",
    "html/scripting",
    "html/select",
];

/// 单案执行车道（collect 时按源形态钉死，execute 按车道分派）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum CaseLane {
    /// `.html` 计分页（variant 键已展开进 case_name）。
    Html,
    /// `.window.js` / `.any.js` window 腿（wrapper 内联执行）。
    WindowJs,
    /// `.any.js` dedicated 变体腿（`.any.worker.js` 键，对账映射
    /// `.any.worker.html` 分母变体；encoding 通道先例）。
    DedicatedWorkerJs,
}

/// 单案执行规格（case_name 与上报键逐字一致——含 variant 后缀 / 腿后缀）。
struct HtmlInfraCaseSpec {
    case_name: String,
    source: String,
    lane: CaseLane,
}

/// [`HTML_INFRA_TEST_SUBDIRS`] 用例排除规则（fetch 脚本头注释零执行记账面的
/// runner 侧镜像，双保险）。`relative` 为 wpt-data 相对路径，`source` 为用例原文。
///
/// - `*-manual.html`（用户交互面）/ `*-ref.html` / `*-notref.html`（reftest 参照页）/
///   `.xhtml`（historical.sub.xhtml 真文档——runner 无 XML MIME 文档载入管道，
///   FIXME(wptfyi-html DC-2+)，rendering 通道同款门控）
/// - 助资产目录 `*/resources/` `*/support/`（helper 脚本不按案跑）、`*/crashtests/`
///   （崩溃回归页）
/// - 无 `testharness` 引用页（reftest 主体页 / 崩溃页——非 testharness 形态，本口径
///   reftest 无子测试不进分母）。仅对 .html 生效：.window.js/.any.js 源不引
///   testharness（wrapper 注入）
/// - rel=match reftest 形态页（colours quirk 面——testharness 通道外）
/// - `.js` 源 `META: global=` 显式收窄对齐（批四先例——worker-only 源不生成
///   window 键；缺省保持全全局语义）
/// - iframe 依赖面（runner 无多 frame 文档管道，semantics/syntax 先例——按内容
///   判定，名字无 iframe 但内容用的案面同样命中；master 卡点 iframe 子文档管道
///   同根因挂账）；多窗口/popup（`window.open`）与 worker/sharedworker/
///   serviceworker 执行面（workers goal 域，semantics 先例）
pub(crate) fn html_infra_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html")
        || name.ends_with("-ref.html")
        || name.ends_with("-notref.html")
        || name.ends_with(".xhtml")
    {
        return true;
    }
    for dir in ["/resources/", "/support/", "/crashtests/"] {
        if relative.contains(dir) {
            return true;
        }
    }
    if relative.ends_with(".html") && !source.contains("testharness") {
        return true;
    }
    if source.contains("rel=match") || source.contains("rel=\"match\"") {
        return true;
    }
    if relative.ends_with(".js")
        && let Some((_, globals)) = source.split_once("META: global=")
        && !globals
            .split(['\n', '\r'])
            .next()
            .is_some_and(|line| line.split(',').any(|token| token.trim() == "window"))
    {
        return true;
    }
    source.contains("<iframe")
        || source.contains("createElement('iframe')")
        || source.contains("createElement(\"iframe\")")
        || source.contains("window.open")
        || source.contains("new Worker(")
        || source.contains("SharedWorker(")
        || source.contains("navigator.serviceWorker")
}

/// Run the pinned upstream html/infrastructure + small-cluster window subset
/// （wptfyi-html DC-2 批九）。filter 按路径子串过滤（如 `query-encoding`、
/// `base-url`）。
pub fn run_html_infra_cases(
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
    for subdir in HTML_INFRA_TEST_SUBDIRS {
        collect_html_infra_cases(wpt_root, subdir, filter, &mut specs);
    }
    let mut cases = crate::parallel_map(&specs, jobs, |spec| {
        let results = match spec.lane {
            CaseLane::Html => {
                // 主文档 wptserve 模板注入：token 全集（semantics s116 先例）+ GET
                // 查询模板（query-encoding 编码变体载体——上游 wptserve 对每请求
                // 资源按该请求查询串注入，变体 query 已在 case_name）。
                let substituted = apply_wpt_substitutions(&spec.source);
                let query = spec.case_name.split_once('?').map(|(_, query)| query).unwrap_or("");
                let substituted = wpt_apply_get_query_substitutions(substituted, query);
                run_testharness_html(
                    wpt_root,
                    &spec.case_name,
                    &substituted,
                    harness_source.as_str(),
                    corpus_case_timeout(),
                )
            }
            CaseLane::WindowJs => {
                run_any_js_window_case(wpt_root, &spec.case_name, &spec.source, harness_source.as_str())
            }
            CaseLane::DedicatedWorkerJs => {
                crate::testharness_workers::run_any_js_dedicated_case_as(
                    wpt_root,
                    &spec.case_name,
                    &spec.case_name,
                    harness_source.as_str(),
                )
            }
        };
        (spec.case_name.clone(), results)
    });
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}

/// 递归收集一个扫描根下的可执行用例规格（testharness_html_semantics 同构：
/// 同递归、同 skip 谓词、同 variant 展开、同 filter）。
fn collect_html_infra_cases(
    wpt_root: &Path,
    subdir: &str,
    filter: Option<&str>,
    specs: &mut Vec<HtmlInfraCaseSpec>,
) {
    let dir = wpt_root.join(subdir);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let nested = format!("{subdir}/{}", entry.file_name().to_string_lossy());
            collect_html_infra_cases(wpt_root, &nested, filter, specs);
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        let is_html = file_name.ends_with(".html");
        let is_window_js = file_name.ends_with(".window.js");
        let is_any_js = file_name.ends_with(".any.js");
        if !is_html && !is_window_js && !is_any_js {
            continue;
        }
        let relative = format!("{subdir}/{file_name}");
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let Ok(source) = read_corpus_to_string(&path) else {
            continue;
        };
        if html_infra_case_skipped(&relative, &source) {
            continue;
        }
        if is_html {
            // R3906（html-semantics M2）variant 语义：声明 variant 的用例只按矩阵
            // 跑、不再跑基础 URL（wpt.fyi 分母 variant URL 独立键同形）。
            let variants = case_variants(&source);
            if variants.is_empty() {
                specs.push(HtmlInfraCaseSpec {
                    case_name: relative,
                    source,
                    lane: CaseLane::Html,
                });
            } else {
                for variant in variants {
                    specs.push(HtmlInfraCaseSpec {
                        case_name: format!("{relative}{variant}"),
                        source: source.clone(),
                        lane: CaseLane::Html,
                    });
                }
            }
            continue;
        }
        // window 腿（.window.js 生成形 .window.html；.any.js 生成形 .any.html）。
        specs.push(HtmlInfraCaseSpec {
            case_name: relative.clone(),
            source: source.clone(),
            lane: CaseLane::WindowJs,
        });
        // dedicated 变体腿（encoding worker_variants 先例——分道判定
        // dedicated_worker_variant：META 缺省双变体 / 显式 worker token）。
        if is_any_js && crate::testharness_workers::dedicated_worker_variant(&source) {
            specs.push(HtmlInfraCaseSpec {
                case_name: relative.replacen(".any.js", ".any.worker.js", 1),
                source,
                lane: CaseLane::DedicatedWorkerJs,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 计分页可跑；助资产目录、reftest 参照/主体页、manual、.xhtml、iframe/worker
    // 内容门、META global 收窄各臂排除。
    #[test]
    fn infra_pages_run_and_assets_skip() {
        assert!(!html_infra_case_skipped(
            "html/infrastructure/common-dom-interfaces/collections/supported-property-names.html",
            r#"<script src="/resources/testharness.js"></script><div id=log></div>"#,
        ));
        // 变体页（query-encoding 载体）可跑——variant 展开在 collect 侧
        assert!(!html_infra_case_skipped(
            "html/infrastructure/urls/resolving-urls/query-encoding/utf-8.html",
            r#"<meta name="variant" content="?include=url">
<script src="/resources/testharness.js"></script>"#,
        ));
        assert!(html_infra_case_skipped(
            "html/infrastructure/safe-passing-of-structured-data/resources/echo-iframe.html",
            "",
        ));
        assert!(html_infra_case_skipped(
            "html/infrastructure/urls/dynamic-changes-to-base-urls/historical.sub.xhtml",
            r#"<script src="/resources/testharness.js"/>"#,
        ));
        assert!(html_infra_case_skipped(
            "html/links/manifest/mime-type-none-manual.html",
            "<script src=/resources/testharness.js>",
        ));
        // reftest 主体页（rel=match / 无 testharness 引用）零执行记账
        assert!(html_infra_case_skipped(
            "html/select/select-capitalize-sizing.html",
            r#"<link rel="match" href="select-capitalize-sizing-ref.html">"#,
        ));
        assert!(html_infra_case_skipped(
            "html/infrastructure/safe-passing-of-structured-data/structuredclone_0.html",
            r#"<script src="/resources/testharness.js"></script><iframe src="..."></iframe>"#,
        ));
        // .window.js/.any.js 不受 no-testharness 门约束（wrapper 注入），内容门与
        // META global 收窄仍生效
        assert!(!html_infra_case_skipped(
            "html/infrastructure/safe-passing-of-structured-data/window-postmessage.window.js",
            "async_test(t => { t.done(); });",
        ));
        assert!(html_infra_case_skipped(
            "html/infrastructure/urls/base-url/document-base-url-window-open-about-blank.https.window.js",
            "test(() => { const w = window.open('about:blank'); });",
        ));
        assert!(html_infra_case_skipped(
            "html/infrastructure/safe-passing-of-structured-data/worker-only.any.js",
            "// META: global=dedicatedworker\nasync_test(t => t.done());",
        ));
        // META 缺省 .any.js 双变体（window + dedicated 腿）不排除
        assert!(!html_infra_case_skipped(
            "html/infrastructure/safe-passing-of-structured-data/shared-array-buffers/serialization-via-idb.any.js",
            "// META: script=/IndexedDB/resources/support.js\nasync_test(t => t.done());",
        ));
    }

    // 批九 GET 模板注入：变体 query 从 case_name 解出，主文档编码变体载体
    // （attributes.sub.html `<meta charset={{GET[encoding]}}>`）按查询串替换；
    // 无查询残留占位符清空（上游 wptserve 缺参语义）。
    #[test]
    fn get_query_substitution_uses_case_query() {
        let source = r#"<!doctype html><meta charset={{GET[encoding]}}><script>var x = "{{GET[none]}}";</script>"#;
        let substituted = apply_wpt_substitutions(source);
        let substituted = wpt_apply_get_query_substitutions(substituted, "encoding=utf8");
        assert!(substituted.contains(r#"<meta charset=utf8>"#), "{substituted}");
        assert!(substituted.contains(r#"var x = "";"#), "{substituted}");
    }
}
