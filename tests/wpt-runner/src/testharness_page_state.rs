//! Page 状态兼容 goal（page-state-compat M1 / DC-1）—— sendBeacon / Page Visibility /
//! navigator.onLine 三小 corpus 上游 testharness window 可执行子集通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_webmessaging
//! 先例），经 `pub(crate)` 复用其执行底座（`run_any_js_window_case` /
//! `run_testharness_html_inner` / `corpus_case_timeout`），行为与既有通道一致。
//!
//! 用例由 `scripts/goals/27-page-state-compat.sh` 按需拉到 `wpt-data/`
//! （gitignored，WPT pin 315976933870b34d6ea30e3f6643403edae678ba）。运行面 =
//! 目录扫描（web-components 先例，skip 规则两侧同源防漂移）+
//! [`page_state_case_skipped`] 内容规则筛减。skip 域（goal 契约排除面）见该函数文档。
//!
//! 语料形态注记（2026-10-17 M1 盘点）：
//! - 上游无顶层 `online/` 目录——navigator.onLine 用例在
//!   `html/browsers/offline/browser-state/`（Contents API 实证，goal 脚本已按此拉取）。
//! - beacon corpus 主体（4 个 `.https.window.js`）是 iframe.contentWindow.navigator
//!   跨文档面 + 服务端 `beacon.py` store/stat 回读——iframe 依赖按 webmessaging 先例
//!   排除；`headers/*.html` + `beacon-url-encoding-euc-kr.https.html` 无 iframe，
//!   依赖 runner fetch_bridge 的 `.py` fixture 端点等价实现（net-api upload.py 先例）
//!   + sendBeacon 真 POST 投递（M2 实现轮，基线先如实记 Fail）。

use crate::testharness::{
    HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_any_js_window_case, run_testharness_html_inner,
};
use std::path::Path;

/// page-state corpus 扫描子目录（三套件：beacon 本体 + headers 叶、page-visibility、
/// browser-state online 叶；helper 目录 `resources/` 不在扫描面）。
pub const PAGE_STATE_TEST_SUBDIRS: &[&str] = &[
    "beacon",
    "beacon/headers",
    "page-visibility",
    "html/browsers/offline/browser-state",
];

/// [`PAGE_STATE_TEST_SUBDIRS`] 用例排除规则（goal 契约排除面的 runner 侧镜像）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文（内容规则按标记筛减）。
///
/// - `*-manual.*`（真实用户交互/OS 窗口操作——timing 先例；中缀匹配——
///   navigator_online_event-manual.https.html 形态）：
///   navigator_online_event-manual、test_minimize-manual、test_tab_state_change-manual
/// - `idlharness.*`（依赖 /resources/WebIDLParser.js——上游 build 期生成资产，repo 内
///   不存在，timing/clipboard 先例）：beacon/idlharness.any.js
/// - 内容标记 `window_state_context`（真实 minimize/restore OS 窗口操作——headless 无
///   可见性变化源，goal 约束 2「按 corpus 可执行子集定分母」）：minimize.html、
///   visibility-state-entry.tentative.html（VisibilityStateEntry performance 面同依赖）
/// - 内容标记 `<iframe` / `createElement('iframe')`（runner 无多 frame 文档管道——
///   webmessaging/timing 先例）：4 个 beacon `.window.js`（iframe.contentWindow 面）、
///   test_child_document、onvisibilitychange、iframe-unload、iframe-session-history
/// - 内容标记 `window.open`（第二文档面——webmessaging 先例）：unload、unload-bubbles
pub(crate) fn page_state_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.contains("-manual.") || name.starts_with("idlharness.") {
        return true;
    }
    source.contains("window_state_context")
        || source.contains("<iframe")
        || source.contains("createElement('iframe')")
        || source.contains("createElement(\"iframe\")")
        || source.contains("window.open")
}

/// Run the imported upstream page-state window subset
/// （page-state-compat goal M1 / DC-1）。filter 按路径子串过滤（如 `beacon/`、
/// `page-visibility/`——基线按套件分类）。
pub fn run_page_state_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
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
    for subdir in PAGE_STATE_TEST_SUBDIRS {
        let Ok(entries) = std::fs::read_dir(wpt_root.join(subdir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            let is_html = file_name.ends_with(".html");
            // .window.js 与 .any.js 同为 window 变体形态（webstorage/fileapi 通道
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
            if page_state_case_skipped(&relative, &source) {
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
