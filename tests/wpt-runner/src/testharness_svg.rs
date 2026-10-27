//! SVG 文档面兼容 goal（svg-compat M1 导入）——`svg/` 上游 testharness 通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_workers
//! 先例），经 `pub(crate)` 复用其执行底座（`run_testharness_html_inner` /
//! `corpus_case_timeout`）。目标面是 **SVG 文档模型 JS API**（SVGElement 类层级 /
//! SVGSVGElement viewBox/CTM / SVGGeometryElement 几何 / 事件目标化——goal
//! 契约 docs/goal/archive/svg-compat.md）；SVG 呈现正确性归 rendering-compat，本通道
//! 不含渲染断言面。
//!
//! 用例由 `scripts/goals/80-svg-compat.sh`（GOAL_FETCH_DEEP=1 git trees 深挖，
//! workers 先例）按 pin 315976933870b34d6ea30e3f6643403edae678ba 拉到
//! `wpt-data/svg/`（gitignored）。上游全树 2224 blobs——`.svg` 文档（渲染断言面
//! 大头，SVG 1.1 `import/` 套件 + 各域 reftests）与 `.html`（testharness DOM 面
//! + reftest 页混布）。
//!
//! 运行面 = 目录全递归扫描（workers 先例，skip 规则两侧同源防漂移）+ 只收
//! `.html`/`.htm` + [`svg_case_skipped`] 内容规则筛减：
//! - 无 `testharness` 引用的 html/htm 是 reftest 页 / ref 页 / manual / demo
//!   ——渲染断言面自动排除（占比盘点在 M1 evidence 按内容分类入档）；
//! - `.svg` 文档不进通道：runner testharness 管线按 HTML 解析页面，XML 文档
//!   解析路径缺位——计入口账（契约「XML 底座复用」评估项，M2 重入条件）；
//! - SMIL（`animations/`）不设目录级排除：SVGAnimationElement 接口面属 DOM
//!   类层级（M2 面），SMIL 时序面按 goal 契约评估后挂账（M4 定稿）。

use crate::testharness::{HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_testharness_html_inner};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// [`SVG_TEST_SUBDIRS`] 用例排除规则（goal 契约排除面的 runner 侧镜像）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文。
///
/// - `*.https.*`（secure-context 前提 runner 无——workers/fileapi 先例）
/// - `.html`/`.htm` 不含 `testharness`（reftest 页 / ref 页 / manual / demo
///   ——渲染断言面，本通道不收；`.svg` 文档在扫描层已按扩展名排除）
/// - 内容标记 `iframe`（frame 树面——runner 无 frame 管线，navigation-compat
///   M3 门槛后随其重入，workers/webmessaging 同规则）
/// - 内容标记 `window.open`（第二文档面，同 iframe 门槛）
pub(crate) fn svg_case_skipped(relative: &str, source: &str) -> bool {
    if relative.contains(".https.") {
        return true;
    }
    if (relative.ends_with(".html") || relative.ends_with(".htm")) && !source.contains("testharness") {
        return true;
    }
    source.contains("iframe") || source.contains("window.open")
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

/// Run the imported upstream svg testharness subset
/// （svg-compat goal M1 / DC-1）。filter 按路径子串过滤（如 `types/`、
/// `struct/`——基线按面分类）。
pub fn run_svg_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
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
    // 目录枚举：扫描根 + 全部子目录（语料最深三层：`types/scripted/`、
    // `struct/reftests/` 等；fetch 侧 GOAL_FETCH_DEEP=1 全量树同布局）；
    // helper 子目录（resources/support）逐层跳过。
    let mut scan_dirs = Vec::new();
    collect_scan_dirs(wpt_root, Path::new("svg"), &mut scan_dirs);
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
            // 只收 html/htm 页（testharness 管线按 HTML 解析；`.svg` 文档执行
            // 路径缺位——模块头注记记账，占比盘点入 evidence）。
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
            if svg_case_skipped(&relative, &source) {
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
