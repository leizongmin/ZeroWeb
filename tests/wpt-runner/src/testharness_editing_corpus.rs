//! editing 命令面（editing-compat goal M1）——`editing/run` + `editing/other`
//! 上游 execCommand/queryCommand 语料通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_css_corpus
//! 先例），经 `pub(crate)` 复用其执行底座（`run_testharness_html_inner` /
//! `corpus_case_timeout`）。目标面是**编辑命令语义断言面**
//! （goal 契约 docs/goal/editing-compat.md）；uievents/selection 归各自通道，
//! 本通道不含渲染断言面。
//!
//! 用例由 `scripts/goals/33-editing-compat.sh`（GOAL_FETCH_DEEP=1 git trees
//! 深挖）按 pin 315976933870b34d6ea30e3f6643403edae678ba 拉到 `wpt-data/editing/`
//! （gitignored）。pin 处上游全树（2026-10-30 清单）：run 43 案平铺 +
//! other 96 案平铺，共享依赖 `include/`（implementation.js/tests.js/
//! editor-test-utils.js/reset.css）与 `data/`（per-command 期望矩阵）随目录拉取；
//! `crashtests/`（145 案病态输入）与 `manual/`（9 案真交互）不入通道
//! （fuzz-repro-guard 口径 / manual 通行规则），`edit-context/`、
//! `plaintext-only/`、`whitespaces/`、`chrome-compat/` 不在 DC-1 语料范围。
//!
//! 运行面 = 目录全递归扫描（css-testharness M0 先例，skip 规则两侧同源防漂移）
//! + 只收 `.html`/`.htm` + [`editing_case_skipped`] 内容规则筛减：
//! - `*-manual.html`（需真实用户交互；corpus 通行规则）
//! - `.https.`（secure-context 前提 runner 无——svg 先例）
//! - helper / 资产目录前缀（[`HELPER_DIRS`] 双保险；collect_scan_dirs 已跳）
//! - 无 `testharness` 引用（demo / ref 页；css M0 同规则）
//! - source 含 `<iframe`（iframe 依赖面——runner 无 iframe 文档管道，
//!   css-variables/fullscreen 同规则；other/ 9 案，占比入 M1 evidence）
//!
//! run/ 用例带 `<meta name=variant>`（?1-1000 分片子测试矩阵，subset-tests.js），
//! 逐 variant 跑（R329 同款；case 名带 query，与上游 dashboard 对齐）。

use crate::testharness::{HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_testharness_html_inner};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// helper / 资产目录（collect_scan_dirs 逐层跳过；skip 谓词前缀双保险同表）：
/// run/、other/ 在 pin 处均平铺无子目录，此表按 corpus 通行规则防御性收录
/// （未来 re-pin 上游引入 support/ 资产子目录时不扩面）。
const HELPER_DIRS: &[&str] = &["resources", "support"];

/// [`run_editing_corpus_dir`] 用例排除规则（fetch 侧 33 号脚本头注释同域，双保险）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文。
pub(crate) fn editing_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html") {
        return true;
    }
    if relative.contains(".https.") {
        return true;
    }
    for helper in HELPER_DIRS {
        if relative.contains(&format!("/{helper}/")) {
            return true;
        }
    }
    if !source.contains("testharness") {
        return true;
    }
    source.contains("<iframe")
}

/// 目录枚举辅助：`dir`（wpt_root 相对）及其全部子目录加入 `out`；helper 目录
/// 逐层跳过（css-testharness collect_scan_dirs 同构，跳过表按 editing 面收录）。
fn collect_scan_dirs(wpt_root: &Path, dir: &Path, out: &mut Vec<String>) {
    out.push(dir.to_string_lossy().to_string());
    let Ok(entries) = std::fs::read_dir(wpt_root.join(dir)) else {
        return;
    };
    let mut subdirs = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if entry.path().is_dir() && !HELPER_DIRS.contains(&name.as_str()) {
            subdirs.push(name);
        }
    }
    subdirs.sort();
    for name in subdirs {
        collect_scan_dirs(wpt_root, &dir.join(&name), out);
    }
}

/// Run one upstream editing corpus directory（run / other 共用底座）。`scan_root`
/// 为 wpt_root 相对目录（如 `editing/run`）。filter 按路径子串过滤。
fn run_editing_corpus_dir(
    wpt_root: &Path,
    scan_root: &str,
    filter: Option<&str>,
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
    let mut scan_dirs = Vec::new();
    collect_scan_dirs(wpt_root, Path::new(scan_root), &mut scan_dirs);
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
            if !is_html {
                continue;
            }
            let relative = format!("{subdir}/{file_name}");
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let Ok(source) = read_corpus_to_string(&path) else {
                continue;
            };
            if editing_case_skipped(&relative, &source) {
                continue;
            }
            // variant 用例逐 query 跑（R329；基础 URL 上游 harness 不注册 = 空转）。
            let variants = crate::testharness::case_variants(&source);
            if variants.is_empty() {
                let results = run_testharness_html_inner(
                    wpt_root,
                    &relative,
                    &source,
                    &harness_source,
                    &[],
                    corpus_case_timeout(),
                );
                cases.push((relative, results));
                continue;
            }
            for variant in variants {
                let case_name = format!("{relative}{variant}");
                let results = run_testharness_html_inner(
                    wpt_root,
                    &case_name,
                    &source,
                    &harness_source,
                    &[],
                    corpus_case_timeout(),
                );
                cases.push((case_name, results));
            }
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}

/// Run the pinned upstream editing/run execCommand corpus
/// （editing-compat goal M1 / DC-2）。filter 按路径子串过滤。
pub fn run_editing_run_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_editing_corpus_dir(wpt_root, "editing/run", filter)
}

/// Run the pinned upstream editing/other corpus
/// （editing-compat goal M1 / DC-2）。filter 按路径子串过滤。
pub fn run_editing_other_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_editing_corpus_dir(wpt_root, "editing/other", filter)
}
