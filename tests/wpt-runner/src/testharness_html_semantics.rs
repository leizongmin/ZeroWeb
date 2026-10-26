//! HTML Semantics 兼容 goal（html-semantics-compat M1 / DC-1）—— `html/semantics`
//! 九子域 DOM/语义/API 面 testharness window 可执行通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_webmessaging
//! 先例），经 `pub(crate)` 复用其执行底座（`run_any_js_window_case` /
//! `run_testharness_html` / `corpus_case_timeout`），行为与既有通道一致。
//!
//! 用例由 `scripts/goals/25-html-semantics-compat.sh` 按需拉到
//! `wpt-data/html/semantics/`（gitignored，WPT pin
//! 315976933870b34d6ea30e3f6643403edae678ba）。运行面 = 九域顶层递归扫描
//! （web-components 先例，skip 规则两侧同源防漂移）+ [`html_semantics_case_skipped`]
//! 内容规则筛减。skip 域（goal 契约排除面 + 非 window 可执行形态）见该函数文档。
//!
//! 域边界注记（goal 契约，2026-10-15 M1 盘点）：
//! - microdata：立项清单十域之一，pin 上游不存在（html/microdata 已移除，404 实测）
//!   ——记空域，不设扫描目录。
//! - media-elements/：corpus 已由归档 media goal 通道资产化收口（209 案 640/653，
//!   docs/compat/trends/wpt-suites.csv）——防重复计账，本通道零执行记账。
//! - the-iframe-element/ + the-frame-element/：frame 树深结构面挂 navigation-compat
//!   M3 门控（其通道 M1 已资产化 the-iframe-element 零执行记账）。
//! - the-template-element/：web-components 通道 corpus（WC_TEST_SUBDIRS 第三目录）。
//!
//! 渲染差异面（table 布局等）归 rendering-compat，本通道只做语义面。

use crate::testharness::{
    HarnessStatus, HarnessSubtestResult, case_variants, corpus_case_timeout, run_any_js_window_case,
    run_testharness_html,
};
use std::path::Path;

/// html-semantics corpus 扫描根目录（fetch 脚本 ②段九域同域——递归扫描覆盖全部
/// depth-N 叶子；microdata 上游不存在不列）。域内排除面按
/// [`html_semantics_case_skipped`] 路径/内容规则筛减。
pub const HTML_SEMANTICS_TEST_SUBDIRS: &[&str] = &[
    "html/semantics/forms",
    "html/semantics/embedded-content",
    "html/semantics/document-metadata",
    "html/semantics/grouping-content",
    "html/semantics/text-level-semantics",
    "html/semantics/tabular-data",
    "html/semantics/interactive-elements",
    "html/semantics/scripting-1",
    "html/semantics/links",
];

/// [`HTML_SEMANTICS_TEST_SUBDIRS`] 用例排除规则（fetch 脚本头注释排除域的 runner 侧
/// 镜像，双保险）。`relative` 为 wpt-data 相对路径，`source` 为用例原文。
///
/// - `*-manual.html`（用户交互面）/ `*-ref.html` / `*-notref.html`（reftest 参照页）/
///   `.https.html`（secure-context 前提）/ `idlharness.*`（WebIDLParser build 期资产
///   repo 内不存在）——net-api/encoding/html-syntax 先例
/// - 助资产目录 `*/resources/` `*/support/`（META script / inline 消费，不按案跑）、
///   `*/crashtests/`（崩溃面）、`*/bfcache/`（多文档回退缓存面——runner 单文档通道外）
/// - 排除域（零执行记账，防跨 goal 重复计账）：`/media-elements/`（media goal 已收口）、
///   `/the-iframe-element/` `/the-frame-element/`（navigation M3 门控）、
///   `/the-template-element/`（web-components 通道 corpus）
/// - 无 `testharness.js` 引用页（历史页 / 索引页——非 testharness 形态，html-syntax 先例）
/// - rel=match reftest 形态页、HTTP 头编码面（`content='http'`——file 通道无法复现，
///   html-syntax 先例）
/// - iframe/frame 依赖面（runner 无多 frame 文档管道，net-api/html-syntax 先例）——
///   按内容判定（observers 先例：名字无 iframe 但内容用的案面同样命中）
/// - 多窗口/popup（`window.open`）与 worker 执行面（`new Worker(` / `SharedWorker(` /
///   `navigator.serviceWorker`）——runner 单文档无窗口管理；workers 为独立 goal 域
pub(crate) fn html_semantics_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html")
        || name.ends_with("-ref.html")
        || name.ends_with("-notref.html")
        || name.ends_with(".https.html")
        || name.starts_with("idlharness.")
    {
        return true;
    }
    for dir in [
        "/resources/",
        "/support/",
        "/crashtests/",
        "/bfcache/",
        "/media-elements/",
        "/the-iframe-element/",
        "/the-frame-element/",
        "/the-template-element/",
    ] {
        if relative.contains(dir) {
            return true;
        }
    }
    if relative.ends_with(".html") && !source.contains("testharness") {
        return true;
    }
    // FIXME(tabular M2): processing-model-1/span-limits.html 同步循环挂死 runner
    // （innerHTML 追加 65533 行 + rowspan=65534 槽位填充，2026-10-15 M1 实测
    // 2s 单案超时与 240s 守卫墙钟下均 rc=124；超时网对此类同步循环无效）。
    // 65k 行表算法面归 rendering-compat 域，修复前挂账排除（零执行记账）。
    if relative.ends_with("tabular-data/processing-model-1/span-limits.html") {
        return true;
    }
    // FIXME(links M2): downloading-resources/ 全目录挂账排除——单案可跑（逐案
    // rc=1 秒级返回），同进程全集必挂（~7 分钟无输出后守卫 rc=124；按
    // header-origin*/header-referrer* 前缀拆分两半均稳定完成，M1 基线数字取自
    // 两次拆分半跑合并，见 evidence）。疑似前案 download/navigation 遗留状态
    // 污染后案且超时网失效，根因待 links 域 M2 排查（2026-10-15 M1 实测）。
    if relative.contains("links/downloading-resources/") {
        return true;
    }
    // FIXME(navigation M3): moving-between-documents 多文档面挂账排除——用例经
    // 同目录 resources/moving-between-documents-helper.js 运行期
    // createElement("iframe") 开第二文档（.html 主源无 iframe 字面，上方内容
    // 规则不可见；2026-10-25 s86 实测 53 案各 1P+1T：首子测试过，其后 iframe
    // load 等待挂满超时网）。runner 单文档通道外，归 navigation-compat M3 多
    // frame 文档管线门控（the-iframe-element 排除先例）；同目录 ordering/ 五案
    // 不引 helper，可跑面保留。
    if source.contains("moving-between-documents-helper.js") {
        return true;
    }
    source.contains("rel=match")
        || source.contains("content='http'")
        || source.contains("<iframe")
        || source.contains("createElement('iframe')")
        || source.contains("createElement(\"iframe\")")
        || source.contains("window.open")
        || source.contains("new Worker(")
        || source.contains("SharedWorker(")
        || source.contains("navigator.serviceWorker")
}

/// Run the pinned upstream html/semantics window subset
/// （html-semantics-compat goal M1 / DC-1——分域语义基线）。九域顶层递归扫描，
/// filter 按路径子串过滤（如 `forms/`、`tabular-data/`——基线分域收集）。
pub fn run_html_semantics_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
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
    for subdir in HTML_SEMANTICS_TEST_SUBDIRS {
        collect_html_semantics_cases(wpt_root, subdir, &harness_source, filter, &mut cases);
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}

/// 递归收集一个语义子域下的 window 可执行用例（[`html_semantics_case_skipped`] 排除）
/// 并执行（web-components `collect_wc_cases` 先例；语料为 depth-N 叶子布局，
/// the-script-element 用例在 depth-3）。
fn collect_html_semantics_cases(
    wpt_root: &Path,
    subdir: &str,
    harness_source: &str,
    filter: Option<&str>,
    cases: &mut Vec<(String, Vec<HarnessSubtestResult>)>,
) {
    let dir = wpt_root.join(subdir);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let nested = format!("{subdir}/{}", entry.file_name().to_string_lossy());
            collect_html_semantics_cases(wpt_root, &nested, harness_source, filter, cases);
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        let is_html = file_name.ends_with(".html");
        // .window.js 与 .any.js 同为 window 变体形态（webmessaging 通道先例——
        // any_js_window_wrapper 内联执行，META script 依赖自动解析）。
        let is_window_js = file_name.ends_with(".window.js");
        let is_any_js = file_name.ends_with(".any.js");
        if !is_html && !is_window_js && !is_any_js {
            continue;
        }
        let relative = format!("{subdir}/{file_name}");
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        if html_semantics_case_skipped(&relative, &source) {
            continue;
        }
        // R3906（html-semantics M2）：variant meta 展开（uievents R329 既有机制复用，
        // `case_variants` 同源）——声明 variant 的用例只按 variant 矩阵跑、不再跑基础
        // URL（上游语义：无 query 的裸跑对 `location.search` 消费方是伪形态；
        // textfieldselection/selection-not-application.html `?default`/`?week,month`
        // 两 variant 裸跑时 nonApplicableTypes 落 [''] 伪清单断言面）。
        let variants = case_variants(&source);
        if variants.is_empty() {
            let results = if is_html {
                run_testharness_html(wpt_root, &relative, &source, harness_source, corpus_case_timeout())
            } else {
                run_any_js_window_case(wpt_root, &relative, &source, harness_source)
            };
            cases.push((relative, results));
            continue;
        }
        for variant in variants {
            let case_name = format!("{relative}{variant}");
            let results = if is_html {
                run_testharness_html(wpt_root, &case_name, &source, harness_source, corpus_case_timeout())
            } else {
                run_any_js_window_case(wpt_root, &case_name, &source, harness_source)
            };
            cases.push((case_name, results));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // s86：moving-between-documents 多文档面按 helper 标记排除（FIXME(navigation
    // M3) 臂）——主源只带 `<script src>` 不含 iframe 字面，须按 helper 文件名命中；
    // 同目录 ordering/ 等不引 helper 的可跑面不得误伤。
    #[test]
    fn moving_between_documents_helper_cases_are_skipped() {
        let helper_case = r#"
<script src="/resources/testharness.js"></script>
<script src="resources/moving-between-documents-helper.js"></script>
<script>runTest("move-back", "iframe", "success", "inline", "classic");</script>
"#;
        assert!(html_semantics_case_skipped(
            "html/semantics/scripting-1/the-script-element/moving-between-documents/move-back-iframe-success-inline-classic.html",
            helper_case,
        ));

        let ordering_case = r#"
<script src="/resources/testharness.js"></script>
<script src="/resources/testharnessreport.js"></script>
<script>var t = async_test("in-order");</script>
"#;
        assert!(!html_semantics_case_skipped(
            "html/semantics/scripting-1/the-script-element/moving-between-documents/ordering/in-order.html",
            ordering_case,
        ));
    }
}
