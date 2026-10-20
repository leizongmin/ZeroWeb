//! Dedicated Worker 兼容 goal（workers-compat M1 / DC-1）—— `workers/` +
//! `dedicated-workers/` 上游 testharness window 可执行子集通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_webmessaging
//! 先例），经 `pub(crate)` 复用其执行底座（`run_any_js_window_case` /
//! `run_testharness_html_inner` / `corpus_case_timeout`），行为与既有通道一致。
//!
//! 用例由 `scripts/goals/70-workers-compat.sh` 按需拉到 `wpt-data/workers/` 与
//! `wpt-data/dedicated-workers/`（gitignored，WPT pin
//! 315976933870b34d6ea30e3f6643403edae678ba）。运行面 = 目录扫描（webmessaging
//! 先例，skip 规则两侧同源防漂移）+ [`workers_case_skipped`] 内容规则筛减。
//!
//! 语料形态注记（2026-10-20 M1 盘点）：workers corpus 主体是 window 侧观测面——
//! `new Worker(url)` 派发 + postMessage 往返断言（`*.worker.js` in-worker harness
//! 形态在 fetch 层排除，M2/M3 worker 运行时落地后按 worker harness 通道评估重入）。
//! 本 goal 的 `new Worker(` 面是验收主体（webmessaging M1 曾把该面挂本 goal，本轮
//! 归位），不作为 skip 标记；仅 SharedWorker / module / iframe / multi-global /
//! secure-context 面按 goal 契约排除。

use crate::testharness::{
    HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_any_js_window_case, run_testharness_html_inner,
};
use std::path::Path;

/// workers/dedicated-workers corpus 扫描子目录（top-level + 一层子目录——语料两
/// 层深：`baseurl/alpha/`、`constructors/Worker/`、`interfaces/WorkerUtils/` 等，
/// 与 lib.sh fetch_dir_html depth<1 递归同布局；helper 目录 `resources/`
/// `support/` 在扫描内跳过）。`workers/modules`（module worker——goal 契约
/// 「启动时评估，不承诺」）、`workers/multi-globals`、`workers/non-automated`
/// （manual）、`workers/same-site-cookies`（cookie 域 + .headers 驱动，cookies
/// goal 面）不扫描，排除记账见 [`workers_case_skipped`] 与 goal evidence。
pub const WORKERS_TEST_SUBDIRS: &[&str] = &["workers", "dedicated-workers"];

/// [`WORKERS_TEST_SUBDIRS`] 用例排除规则（goal 契约排除面的 runner 侧镜像）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文（内容规则按标记筛减）。
///
/// - `*.https.*`（secure-context 前提 runner 无——fileapi/webmessaging 先例）
/// - `.html`/`.htm` 不含 `testharness`（同目录无 harness 支撑页——`examples/`
///   演示页、iframe 导航空白靶页；`.window.js`/`.any.js` 无此引用属正常形态，
///   规则只作用于 html/htm）
/// - 内容标记 `new SharedWorker(`（SharedWorker——goal Mission 排除，挂账）
/// - 内容标记 `iframe`（frame 树面——runner 无 frame 管道，navigation-compat M3
///   门槛后随其重入，webmessaging 同规则）
/// - 内容标记 `window.open`（第二文档面，同 iframe 门槛）
/// - 内容标记 `type: 'module'` / `type:"module"`（module worker——goal 契约
///   「classic 先行，module 评估记账」；`workers/modules/` 目录级不扫描，此处兜
///   底顶层零散 module 用例）
pub(crate) fn workers_case_skipped(relative: &str, source: &str) -> bool {
    if relative.contains(".https.") {
        return true;
    }
    if (relative.ends_with(".html") || relative.ends_with(".htm")) && !source.contains("testharness") {
        return true;
    }
    source.contains("new SharedWorker(")
        || source.contains("iframe")
        || source.contains("window.open")
        || source.contains("type: 'module'")
        || source.contains("type:'module'")
        || source.contains("type: \"module\"")
        || source.contains("type:\"module\"")
}

/// 目录枚举辅助：`dir`（wpt_root 相对）及其全部子目录加入 `out`（wpt_root 相对
/// 字符串）；`resources`/`support` helper 目录逐层跳过。
fn collect_scan_dirs(wpt_root: &Path, dir: &Path, out: &mut Vec<String>) {
    out.push(dir.to_string_lossy().to_string());
    let Ok(entries) = std::fs::read_dir(wpt_root.join(dir)) else {
        return;
    };
    let mut subdirs = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_dir() && name != "resources" && name != "support" {
            subdirs.push(name);
        }
    }
    subdirs.sort();
    for name in subdirs {
        collect_scan_dirs(wpt_root, &dir.join(&name), out);
    }
}

/// Run the imported upstream workers window subset
/// （workers-compat goal M1 / DC-1）。filter 按路径子串过滤（如
/// `constructors/`、`semantics/`——基线按面分类）。
pub fn run_workers_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
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
    // 目录枚举：扫描根 + 全部子目录（语料最深三层：`workers/interfaces/WorkerGlobalScope/close/`
    // 等；fetch 侧 GOAL_FETCH_DEEP=1 全量树同布局）；helper 子目录（resources/support）
    // 逐层跳过。
    let mut scan_dirs = Vec::new();
    for subdir in WORKERS_TEST_SUBDIRS {
        collect_scan_dirs(wpt_root, Path::new(subdir), &mut scan_dirs);
    }
    scan_dirs.sort();
    scan_dirs.dedup();
    let mut cases = Vec::new();
    for subdir in &scan_dirs {
        let Ok(entries) = std::fs::read_dir(wpt_root.join(subdir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_html = file_name.ends_with(".html") || file_name.ends_with(".htm");
            // .window.js 与 .any.js 同为 window 变体形态（webmessaging/fileapi 通道
            // 先例——any_js_window_wrapper 内联执行，META script 依赖自动解析）。
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
            if workers_case_skipped(&relative, &source) {
                continue;
            }
            let results = if is_html {
                run_testharness_html_inner(
                    wpt_root,
                    &relative,
                    &source,
                    &harness_source,
                    &[],
                    corpus_case_timeout(),
                )
            } else {
                run_any_js_window_case(wpt_root, &relative, &source, &harness_source)
            };
            cases.push((relative, results));
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}
