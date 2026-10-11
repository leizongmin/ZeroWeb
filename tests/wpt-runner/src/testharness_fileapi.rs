//! FileAPI 兼容 goal（fileapi-compat M1 / DC-1）—— Blob/File/FileReader/objectURL
//! 上游 testharness window 可执行子集通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（后者已 ~10.5k 行，goal 通道自 2000 行
//! 约束起新开文件），经 `pub(crate)` 复用其执行底座（`run_any_js_window_case` /
//! `run_testharness_html_inner` / `corpus_case_timeout`），行为与既有通道一致。
//!
//! 用例由 `scripts/goals/16-fileapi-compat.sh` 按需拉到 `wpt-data/FileAPI/`
//! （gitignored，WPT pin 315976933870b34d6ea30e3f6643403edae678ba）。运行面 =
//! 目录递归扫描（非逐案常量——web-components 先例，skip 规则两侧同源防漂移）+
//! [`fileapi_case_skipped`] 内容规则筛减。skip 域（Support Envelope 排除项 +
//! goal Mission 排除面）见该函数文档。

use crate::testharness::{
    HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_any_js_window_case, run_testharness_html_inner,
};
use std::path::Path;

/// FileAPI corpus 扫描子目录（top-level 一并纳入——fileReader/unicode 两个 .any.js
/// 在顶层；skip 规则排除 idlharness/historical/manual/detached-frame 面）。
pub const FILEAPI_TEST_SUBDIRS: &[&str] = &[
    "FileAPI",
    "FileAPI/blob",
    "FileAPI/file",
    "FileAPI/FileReader",
    "FileAPI/reading-data-section",
    "FileAPI/url",
    "FileAPI/filelist-section",
];

/// [`FILEAPI_TEST_SUBDIRS`] 用例排除规则（fetch 脚本头注释 skip 域的 runner 侧镜像）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文（内容规则按标记筛减）。
///
/// - `*-manual.html` / `*-ref.html` / `*-notref.html`（真实用户交互 / reftest 参照页，
///   observers 先例）
/// - `idlharness*`（依赖 /resources/WebIDLParser.js build 期生成资产，window 可执行
///   面之外——clipboard idlharness 先例）
/// - `*.https.*`（BlobURL 分区面 + historical——存储分区/HTTPS/Service Worker 前提
///   runner 无）
/// - `*-crash*`（crashtest 崩溃回归面，非语义簇——net-api crashtests/ 先例）
/// - `Blob-methods-from-detached-frame.html`（detached iframe 面依赖第二 document）
/// - 内容标记 `iframe`/`window.open`（multi-global/第二文档面：url/ 的
///   cross-global-revoke、multi-global-origin-serialization、sandboxed-iframe、
///   unicode-origin、url-lifetime、url-charset、url-in-tags*、url-reload 全族——
///   runner 无 iframe/window.open 管道，navigation-compat 管道落地后随其重入；
///   基线预跑实证：该族 5 案 Fail/Timeout 全部落在 iframe 装载/新窗口语义上，
///   而健康面（blob/reading-data/url-with-fetch）源码零 iframe 字样，标记无误伤）
/// - 内容标记 `new Worker(`/`importScripts(`（dedicated worker 面——workers-compat
///   管道域：FileReader/workers.html、BlobURL/opaque-origin）
/// - 内容标记 `ReadableStream`/`getReader(`/`pipeTo(`（streams 消费端——goal Mission
///   排除面，记账回流 net-api streams 域：blob/Blob-stream.any.js、
///   blob/Blob-textStream.any.js）
/// - 名字 `send-file-form*` / 内容标记 `.py`（表单提交回显端点依赖 wptserve——
///   file/ 的 send-file-form*.html 七案 + send-file-formdata*.any.js 三案，runner
///   无服务器回环；formdata 上传面记账挂账）
pub(crate) fn fileapi_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html")
        || name.ends_with("-ref.html")
        || name.ends_with("-notref.html")
        || name.starts_with("idlharness")
        || relative.contains(".https.")
        || name.contains("-crash")
        || name == "Blob-methods-from-detached-frame.html"
        || name.starts_with("send-file-form")
    {
        return true;
    }
    source.contains("iframe")
        || source.contains("window.open")
        || source.contains("new Worker(")
        || source.contains("importScripts(")
        || source.contains("ReadableStream")
        || source.contains("getReader(")
        || source.contains("pipeTo(")
        || source.contains(".py")
}

/// Run the imported upstream FileAPI window subset
/// （fileapi-compat goal M1 / DC-1）。filter 按路径子串过滤（如 `blob/`、
/// `reading-data-section/`——基线按面分类）。
pub fn run_fileapi_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
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
    for subdir in FILEAPI_TEST_SUBDIRS {
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
            // .window.js 与 .any.js 同为 window 变体形态（webstorage 通道先例——
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
            if fileapi_case_skipped(&relative, &source) {
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
