//! css testharness 面（css-testharness-compat goal M0 试点）——`css/css-values` +
//! `css/selectors` + `css/css-variables` 上游 testharness 通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_svg
//! 先例），经 `pub(crate)` 复用其执行底座（`run_testharness_html_inner` /
//! `corpus_case_timeout`）。目标面是 **CSSOM/style API 断言面**
//! （`getComputedStyle` / `CSSStyleDeclaration` / 支持性判断——goal 契约
//! docs/goal/css-testharness-compat.md）；reftest 渲染账归 rendering-compat，
//! 本通道不含渲染断言面。
//!
//! 用例由 `scripts/goals/35-css-testharness-compat.sh`（GOAL_FETCH_DEEP=1
//! git trees 深挖，svg/workers 先例）按 pin 315976933870b34d6ea30e3f6643403edae678ba
//! 拉到 `wpt-data/css/`（gitignored）。试点两目录上游全树：css-values 672 blobs /
//! selectors 996 blobs（2026-10-29 树清单）——`.html` 主体 + support 资产
//! （png/css/woff）+ `.xht` XML 页；两目录均无 `.any.js`（形态分簇实测，
//! `.any.js` window 变体通道为后续扩面目录预留，net-api 先例）。
//! css-variables 数据双源（M2）：fetch-wpt-data 精选仓（v1.10）只有 reftest
//! 口径子集（harness 形态 59 页），上游全量 harness 面由 35 号脚本深挖补拉。
//!
//! 运行面 = 目录全递归扫描（svg/workers 先例，skip 规则两侧同源防漂移）+
//! 只收 `.html`/`.htm` + [`css_testharness_case_skipped`] 内容规则筛减：
//! - 无 `testharness` 引用的 html/htm 是 reftest 页 / ref 页 / manual / demo
//!   ——渲染断言面自动排除（rendering-compat 流域；占比盘点入 M0 evidence）；
//! - `.xht` 文档不进通道：runner testharness 管线按 HTML 解析页面，XML 文档
//!   解析路径缺位（svg `.svg` 同口径）——计入口账；
//! - source 含 `<iframe`——iframe 依赖面（runner 无 iframe 文档管道，
//!   css-variables/fullscreen 同规则）。

use crate::testharness::{HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_testharness_html_inner};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// helper / 资产目录（collect_scan_dirs 逐层跳过；skip 谓词前缀双保险同表）：
/// - `resources`/`support`：helper 资产，不按案跑（corpus 通行规则）
/// - `reference`：reftest ref 页目录（css-values 27 页，渲染断言面）
/// - `crashtests`：病态输入 crash 回归页（fuzz-repro-guard 口径不入常驻运行面，
///   css-variables `*-crash.html` 同口径）
const HELPER_DIRS: &[&str] = &["resources", "support", "reference", "crashtests"];

/// 病态内存案名级排除（fuzz-repro-guard 口径不入常驻运行面；runner 谓词单点，
/// 直接子命令调用与分片脚本两路同享）：
/// - `if-conditionals.html`：css-values，CSS if() 条件面半支持状态指数膨胀，
///   单案 Oilpan CagedHeap OOM（fuzz-repro-guard 口径复现实证，M0）
/// - `variable-exponential-blowup.html`：css-variables，自定义属性引用链指数
///   展开，单案 4.9GB 实测杀树（css-testharness-compat M1 归因实证；M2 解禁
///   css-variables 的前置排除）
const PATHO_CASES: &[&str] = &["if-conditionals.html", "variable-exponential-blowup.html"];

/// [`run_css_corpus_dir`] 用例排除规则（fetch 侧 35 号脚本头注释同域，双保险）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文。
///
/// - `*-manual.html`（需真实用户交互；css-variables 先例）
/// - `*-crash.html`（病态输入 crash 回归——fuzz-repro-guard 口径）
/// - 病态内存案名（[`PATHO_CASES`]）
/// - `.https.`（secure-context 前提 runner 无——svg 先例）
/// - helper / 资产目录前缀（[`HELPER_DIRS`] 双保险；collect_scan_dirs 已跳）
/// - 无 `testharness` 引用（reftest 页 / ref 页 / manual / demo——渲染断言面
///   自动排除，svg 先例）
/// - source 含 `<iframe`（iframe 依赖面——runner 无 iframe 文档管道）
pub(crate) fn css_testharness_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("-manual.html") || name.ends_with("-crash.html") {
        return true;
    }
    if PATHO_CASES.contains(&name) {
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
/// 逐层跳过（[`HELPER_DIRS`]，svg `collect_scan_dirs` 同构、跳过表按 css 面
/// 扩 reference/crashtests）。
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

/// Run one upstream css testharness corpus directory（试点 css-values /
/// selectors 共用底座，M3 扩面目录复用）。`scan_root` 为 wpt_root 相对目录
/// （如 `css/css-values`）。filter 按路径子串过滤。
fn run_css_corpus_dir(
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
    // 目录枚举：扫描根 + 全部子目录（fetch 侧 GOAL_FETCH_DEEP=1 全量树同布局）；
    // helper 子目录逐层跳过。
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
            // 只收 html/htm 页（testharness 管线按 HTML 解析；`.xht` XML 文档
            // 执行路径缺位——模块头注记记账，占比盘点入 M0 evidence）。
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
            if css_testharness_case_skipped(&relative, &source) {
                continue;
            }
            let results = run_testharness_html_inner(
                wpt_root,
                &relative,
                &source,
                &harness_source,
                &[],
                corpus_case_timeout(),
            );
            cases.push((relative, results));
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}

/// Run the pinned upstream css-values testharness subset
/// （css-testharness-compat goal M0 / DC-1）。filter 按路径子串过滤。
pub fn run_css_values_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-values", filter)
}

/// Run the pinned upstream css/selectors testharness subset
/// （css-testharness-compat goal M0 / DC-1）。filter 按路径子串过滤。
pub fn run_css_selectors_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/selectors", filter)
}

/// Run the pinned upstream css-variables testharness subset（css-testharness-compat
/// goal M2 / DC-3——自 site-optimizer R5114 通道（`testharness` 模块
/// `run_css_variables_cases`）迁入共享底座：skip 规则单点防漂移，非 testharness
/// reftest 页出运行面，病态内存案谓词排除）。filter 按路径子串过滤。
pub fn run_css_variables_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-variables", filter)
}

/// Run the pinned upstream css-color testharness subset（css-testharness-compat
/// goal M3 扩面——getComputedStyle 颜色序列化/解析断言面，top15 分母
/// 12,714 子测试）。filter 按路径子串过滤。
pub fn run_css_color_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-color", filter)
}

/// Run the pinned upstream css-align testharness subset（css-testharness-compat
/// goal M3 扩面——box alignment 属性值解析/序列化断言面，top15 分母
/// 5,311 子测试）。filter 按路径子串过滤。
pub fn run_css_align_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-align", filter)
}

/// Run the pinned upstream css-sizing testharness subset（css-testharness-compat
/// goal M3 扩面——sizing 属性解析/计算值断言面，top15 分母 6,258 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_sizing_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-sizing", filter)
}

/// Run the pinned upstream css-typed-om testharness subset（css-testharness-compat
/// goal M3 扩面批次 2——CSS Typed OM API 断言面（CSSStyleValue/CSSNumericValue
/// 等），top15 分母 13,445 子测试）。filter 按路径子串过滤。
pub fn run_css_typed_om_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-typed-om", filter)
}

/// Run the pinned upstream css-fonts testharness subset（css-testharness-compat
/// goal M3 扩面批次 2——@font-face/font-family 匹配断言面，top15 分母
/// 8,665 子测试）。filter 按路径子串过滤。
pub fn run_css_fonts_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-fonts", filter)
}

/// Run the pinned upstream css-shapes testharness subset（css-testharness-compat
/// goal M3 扩面批次 2——shape-outside 解析/计算值断言面，top15 分母
/// 7,527 子测试）。filter 按路径子串过滤。
pub fn run_css_shapes_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-shapes", filter)
}
