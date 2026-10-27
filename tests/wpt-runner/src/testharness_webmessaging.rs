//! Web Messaging 兼容 goal（webmessaging-compat M1 / DC-1）—— postMessage /
//! MessageChannel / MessagePort / BroadcastChannel 上游 testharness window
//! 可执行子集通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_fileapi
//! 先例），经 `pub(crate)` 复用其执行底座（`run_any_js_window_case` /
//! `run_testharness_html_inner` / `corpus_case_timeout`），行为与既有通道一致。
//!
//! 用例由 `scripts/goals/17-webmessaging-compat.sh` 按需拉到
//! `wpt-data/webmessaging/`（gitignored，WPT pin
//! 315976933870b34d6ea30e3f6643403edae678ba）。运行面 = 目录扫描（web-components
//! 先例，skip 规则两侧同源防漂移）+ [`webmessaging_case_skipped`] 内容规则筛减。
//! skip 域（goal Mission 排除面 + 支撑页）见该函数文档。
//!
//! 语料形态注记（2026-10-12 M1 盘点）：webmessaging corpus 主体是
//! `without-ports/` + `with-ports/` 的同 document postMessage 语义面（targetOrigin
//! 解析/校验、message 事件属性）与 `.any.js` 的 MessageChannel/Port 同全局面；
//! 跨 document 面（016-021、broadcastchannel 多数、multi-globals）依赖 iframe
//! 管道，按 goal 契约挂 navigation-compat M3 门槛排除。

use crate::testharness::{
    HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_any_js_window_case, run_testharness_html_inner,
};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// webmessaging corpus 扫描子目录（top-level 一并纳入——Channel_postMessage_* /
/// MessageEvent / MessagePort 的 `.any.js` window 变体在顶层；helper 目录
/// `resources/` `support/` `multi-globals/` 不在扫描面）。
pub const WEBMESSAGING_TEST_SUBDIRS: &[&str] = &[
    "webmessaging",
    "webmessaging/with-options",
    "webmessaging/without-ports",
    "webmessaging/with-ports",
    "webmessaging/broadcastchannel",
    "webmessaging/message-channels",
    "webmessaging/message-channels/close-event",
];

/// [`WEBMESSAGING_TEST_SUBDIRS`] 用例排除规则（goal 契约排除面的 runner 侧镜像）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文（内容规则按标记筛减）。
///
/// - `*.https.*`（secure-context / 存储分区 / bfcache 前提 runner 无——fileapi
///   先例）：broadcastchannel/cross-partition、service-worker、close-event
///   entangled-after-bfcache
/// - 内容标记 `iframe`（跨 document / detached-iframe / sandboxed-iframe /
///   opaque-origin 面——runner 无 frame 树管道，navigation-compat M3 门槛后随其
///   重入）：without-ports + with-ports 016-021、broadcastchannel 多数、
///   message-channels/cross-document、xsite 两案
/// - 内容标记 `window.open`（第二文档面，同 iframe 门槛）：
///   postMessage_CryptoKey_insecure
/// - 内容标记 `new Worker(`/`importScripts(`（dedicated worker 面——
///   workers-compat 管道域，goal Mission 排除）：worker.any.js、
///   worker-post-after-close、025.html（fetch_tests_from_worker）、
///   broadcastchannel/blobs（worker 内 BC blob 面）、workers.html
/// - 内容标记 `remote-context-helper`（dispatcher/remote-context-helper 第二
///   window 基建，META 依赖未导入且属跨文档面）：close-event 三个 `.window.js`
/// - `.html` 不含 `testharness`（同目录无 harness 支撑页——without-ports/019-1、
///   020-1 的 iframe 导航空白靶页；`.window.js`/`.any.js` 无此引用属正常形态，
///   规则只作用于 html）
pub(crate) fn webmessaging_case_skipped(relative: &str, source: &str) -> bool {
    if relative.contains(".https.") {
        return true;
    }
    if relative.ends_with(".html") && !source.contains("testharness") {
        return true;
    }
    source.contains("iframe")
        || source.contains("window.open")
        || source.contains("new Worker(")
        || source.contains("importScripts(")
        || source.contains("remote-context-helper")
}

/// Run the imported upstream webmessaging window subset
/// （webmessaging-compat goal M1 / DC-1）。filter 按路径子串过滤（如
/// `with-ports/`、`broadcastchannel/`——基线按面分类）。
pub fn run_webmessaging_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
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
    let mut cases = Vec::new();
    for subdir in WEBMESSAGING_TEST_SUBDIRS {
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
            let Ok(source) = read_corpus_to_string(&path) else {
                continue;
            };
            if webmessaging_case_skipped(&relative, &source) {
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
