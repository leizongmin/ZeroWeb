//! css testharness 面（css-testharness-compat goal M0 试点）——`css/css-values` +
//! `css/selectors` + `css/css-variables` 上游 testharness 通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_svg
//! 先例），经 `pub(crate)` 复用其执行底座（`run_testharness_html_inner` /
//! `corpus_case_timeout`）。目标面是 **CSSOM/style API 断言面**
//! （`getComputedStyle` / `CSSStyleDeclaration` / 支持性判断——goal 契约
//! docs/goal/archive/css-testharness-compat.md）；reftest 渲染账归 rendering-compat，
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

/// Run the pinned upstream css-anchor-position testharness subset
/// （css-testharness-compat goal M3 扩面批次 3——CSS anchor positioning API
/// 断言面（anchor()/anchor-size()/position-anchor 等），top15 分母
/// 13,657 子测试）。filter 按路径子串过滤。
pub fn run_css_anchor_position_cases(
    wpt_root: &Path,
    filter: Option<&str>,
) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-anchor-position", filter)
}

/// Run the pinned upstream css-masking testharness subset（css-testharness-compat
/// goal M3 扩面批次 3——mask/clip-path/mask-border 解析与计算值断言面，top15
/// 分母 6,399 子测试）。filter 按路径子串过滤。
pub fn run_css_masking_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-masking", filter)
}

/// Run the pinned upstream css-backgrounds testharness subset
/// （css-testharness-compat goal M3 扩面批次 3——background/border 解析、
/// 序列化与计算值断言面，top15 分母 6,223 子测试）。filter 按路径子串过滤。
pub fn run_css_backgrounds_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-backgrounds", filter)
}

/// Run the pinned upstream css-transforms testharness subset
/// （css-testharness-compat goal M3 扩面批次 4——transform/rotate/scale/
/// translate 解析、序列化与计算值断言面，top15 分母 5,516 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_transforms_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-transforms", filter)
}

/// Run the pinned upstream css motion path testharness subset（扫描根
/// `css/motion`——上游目录名无 css- 前缀；css-testharness-compat goal M3
/// 扩面批次 4——offset-* 运动路径解析与计算值断言面，top15 分母 4,959
/// 子测试）。filter 按路径子串过滤。
pub fn run_css_motion_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/motion", filter)
}

/// Run the pinned upstream css-grid testharness subset（css-testharness-compat
/// goal M3 扩面批次 5——grid 布局属性解析/计算值断言面，top15 分母
/// 15,841 子测试）。filter 按路径子串过滤。
pub fn run_css_grid_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-grid", filter)
}

/// Run the pinned upstream css-text testharness subset（css-testharness-compat
/// goal M3 扩面批次 5——text 排版属性（line-break/hyphens/text-transform 等）
/// 解析与计算值断言面，top15 分母 5,714 子测试）。filter 按路径子串过滤。
pub fn run_css_text_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-text", filter)
}

// css-testharness-compat-v2 goal M1 批次 1（css 二期 B1 值/属性解析序列化簇
// 首批 6 目录，分母 16,564 子测试——M0 甄别表 docs/goal/archive/css-testharness-compat-v2/
// evidence/2026-11-04-m0-triage-and-gate.md §3；通道契约与一期 M3 扩面一致）。

/// Run the pinned upstream cssom testharness subset（css 二期 M1 批次 1——
/// CSSOM 声明值解析/序列化断言面，M0 分母 4,456 子测试）。filter 按路径子串过滤。
pub fn run_cssom_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/cssom", filter)
}

/// Run the pinned upstream css-images testharness subset（css 二期 M1 批次 1——
/// image 值类型（gradient/object-fit 等）解析与序列化断言面，M0 分母 3,595
/// 子测试）。filter 按路径子串过滤。
pub fn run_css_images_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-images", filter)
}

/// Run the pinned upstream filter-effects testharness subset（css 二期 M1
/// 批次 1——filter/backdrop-filter/drop-shadow 解析与序列化断言面，M0 分母
/// 3,007 子测试）。filter 按路径子串过滤。
pub fn run_filter_effects_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/filter-effects", filter)
}

/// Run the pinned upstream css-ui testharness subset（css 二期 M1 批次 1——
/// UI 交互属性（outline/cursor/pointer-events 等）解析与计算值断言面，M0
/// 分母 2,457 子测试）。filter 按路径子串过滤。
pub fn run_css_ui_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-ui", filter)
}

/// Run the pinned upstream css-borders testharness subset（css 二期 M1 批次
/// 1——border 四边/圆角/图像边框解析与序列化断言面，M0 分母 1,601 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_borders_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-borders", filter)
}

/// Run the pinned upstream css-logical testharness subset（css 二期 M1 批次
/// 1——逻辑属性（margin-block/padding-inline 等）解析与计算值断言面，M0
/// 分母 1,448 子测试）。filter 按路径子串过滤。
pub fn run_css_logical_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-logical", filter)
}

// css-testharness-compat-v2 goal M1 批次 2（css 二期 B1 值/属性解析序列化簇
// 余 18 目录，分母 7,318 子测试——M0 甄别表 §3；通道契约与批次 1 一致，
// fill-stroke / compositing 上游目录名无 css- 前缀走 motion 先例）。

/// Run the pinned upstream css-text-decor testharness subset（css 二期 M1
/// 批次 2——text-decoration 属性族解析与序列化断言面，M0 分母 1,376 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_text_decor_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-text-decor", filter)
}

/// Run the pinned upstream css-box testharness subset（css 二期 M1 批次 2——
/// 盒模型边距属性（margin/padding 等）解析与计算值断言面，M0 分母 1,063
/// 子测试）。filter 按路径子串过滤。
pub fn run_css_box_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-box", filter)
}

/// Run the pinned upstream css-pseudo testharness subset（css 二期 M1 批次
/// 2——伪元素（::before/::first-line 等）生成内容断言面，M0 分母 865
/// 子测试）。filter 按路径子串过滤。
pub fn run_css_pseudo_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-pseudo", filter)
}

/// Run the pinned upstream css-display testharness subset（css 二期 M1 批次
/// 2——display 值与盒类型解析断言面，M0 分母 567 子测试）。filter 按路径
/// 子串过滤。
pub fn run_css_display_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-display", filter)
}

/// Run the pinned upstream css-mixins testharness subset（css 二期 M1 批次
/// 2——混入规则（@mixin）自定义原语解析面，M0 分母 618 子测试）。filter
/// 按路径子串过滤。
pub fn run_css_mixins_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-mixins", filter)
}

/// Run the pinned upstream css-syntax testharness subset（css 二期 M1 批次
/// 2——CSS 语法解析与错误恢复断言面，M0 分母 444 子测试）。filter 按路径
/// 子串过滤。
pub fn run_css_syntax_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-syntax", filter)
}

/// Run the pinned upstream fill-stroke testharness subset（css 二期 M1 批次
/// 2——SVG 填充/描边属性解析与序列化断言面，M0 分母 409 子测试）。filter
/// 按路径子串过滤。
pub fn run_fill_stroke_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/fill-stroke", filter)
}

/// Run the pinned upstream css-shadow testharness subset（css 二期 M1 批次
/// 2——box-shadow/text-shadow 解析与序列化断言面，M0 分母 377 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_shadow_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-shadow", filter)
}

/// Run the pinned upstream css-size-adjust testharness subset（css 二期 M1
/// 批次 2——font-size-adjust 解析与计算值断言面，M0 分母 215 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_size_adjust_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-size-adjust", filter)
}

/// Run the pinned upstream css-content testharness subset（css 二期 M1 批次
/// 2——content 属性值解析与序列化断言面，M0 分母 253 子测试）。filter 按
/// 路径子串过滤。
pub fn run_css_content_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-content", filter)
}

/// Run the pinned upstream css-will-change testharness subset（css 二期 M1
/// 批次 2——will-change 属性解析断言面，M0 分母 173 子测试）。filter 按
/// 路径子串过滤。
pub fn run_css_will_change_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-will-change", filter)
}

/// Run the pinned upstream css-color-adjust testharness subset（css 二期 M1
/// 批次 2——颜色调整属性（print-color-adjust/color-scheme）解析断言面，M0
/// 分母 171 子测试）。filter 按路径子串过滤。
pub fn run_css_color_adjust_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-color-adjust", filter)
}

/// Run the pinned upstream css-easing testharness subset（css 二期 M1 批次
/// 2——缓动函数（linear()/cubic-bezier()/steps()）解析与序列化断言面，M0
/// 分母 167 子测试）。filter 按路径子串过滤。
pub fn run_css_easing_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-easing", filter)
}

/// Run the pinned upstream compositing testharness subset（css 二期 M1 批次
/// 2——混合合成（mix-blend-mode/isolation）解析与计算值断言面，M0 分母
/// 167 子测试）。filter 按路径子串过滤。
pub fn run_compositing_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/compositing", filter)
}

/// Run the pinned upstream css-counter-styles testharness subset（css 二期
/// M1 批次 2——@counter-style 规则与 counter() 序列化断言面，M0 分母 158
/// 子测试）。filter 按路径子串过滤。
pub fn run_css_counter_styles_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-counter-styles", filter)
}

/// Run the pinned upstream css-nesting testharness subset（css 二期 M1 批次
/// 2——CSS 嵌套规则解析断言面，M0 分母 126 子测试）。filter 按路径子串
/// 过滤。
pub fn run_css_nesting_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-nesting", filter)
}

/// Run the pinned upstream css-color-hdr testharness subset（css 二期 M1
/// 批次 2——HDR 颜色（dynamic-range 等）解析断言面，M0 分母 116 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_color_hdr_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-color-hdr", filter)
}

/// Run the pinned upstream css-env testharness subset（css 二期 M1 批次 2——
/// env() 环境变量解析断言面，M0 分母 53 子测试）。filter 按路径子串过滤。
pub fn run_css_env_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-env", filter)
}

// css-testharness-compat-v2 goal M2 批次 3（css 二期 B2 规则/条件/级联簇
// 7 目录，分母 9,751 子测试——M0 甄别表 §3；CSS2 数值门按 html 面折算 300，
// .xht XML 面不在通道——模块头注记；mediaqueries / CSS2 上游目录名无
// css- 前缀走 motion 先例）。

/// Run the pinned upstream css-conditional testharness subset（css 二期 M2
/// 批次 3——条件规则（@media/@supports）解析与匹配断言面，M0 分母 3,258
/// 子测试）。filter 按路径子串过滤。
pub fn run_css_conditional_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-conditional", filter)
}

/// Run the pinned upstream mediaqueries testharness subset（css 二期 M2 批次
/// 3——媒体查询解析与匹配断言面，M0 分母 1,768 子测试）。filter 按路径
/// 子串过滤。
pub fn run_mediaqueries_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/mediaqueries", filter)
}

/// Run the pinned upstream css-cascade testharness subset（css 二期 M2 批次
/// 3——级联与继承（@import/important/origin）断言面，M0 分母 1,750 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_cascade_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-cascade", filter)
}

/// Run the pinned upstream CSS2 testharness subset（扫描根 `css/CSS2`——上游
/// 目录名大写无 css- 前缀；css 二期 M2 批次 3——CSS2.1 html 面属性解析与
/// 计算值断言面，M0 分母 2,548 子测试，数值门按 html 面折算 300——.xht
/// XML 面不在通道）。filter 按路径子串过滤。
pub fn run_css2_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/CSS2", filter)
}

/// Run the pinned upstream css-forms testharness subset（css 二期 M2 批次
/// 3——表单元素 UI 样式断言面，M0 分母 93 子测试）。filter 按路径子串过滤。
pub fn run_css_forms_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-forms", filter)
}

/// Run the pinned upstream css-link-params testharness subset（css 二期 M2
/// 批次 3——link 元素参数传递断言面，M0 分母 93 子测试）。filter 按路径
/// 子串过滤。
pub fn run_css_link_params_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-link-params", filter)
}

/// Run the pinned upstream css-font-loading testharness subset（css 二期 M2
/// 批次 3——FontFace/FontFaceSet 加载 API 断言面，M0 分母 241 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_font_loading_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-font-loading", filter)
}

// css-testharness-compat-v2 goal M2 批次 4（css 二期 B3 混合布局簇 13 目录，
// 分母 19,286 子测试——M0 甄别表 §3；M0 划界：解析/序列化份额入本账，布局
// 探测面失败归口布局账〔rendering-compat 控制面 / 独立 layout goal 待立项〕，
// 本通道修到序列化/声明值层为止）。

/// Run the pinned upstream css-gaps testharness subset（css 二期 M2 批次 4——
/// 间距属性（gap/row-gap/column-gap）解析断言面，M0 分母 4,157 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_gaps_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-gaps", filter)
}

/// Run the pinned upstream css-flexbox testharness subset（css 二期 M2 批次
/// 4——flexbox 属性（flex/flex-direction 等）解析与计算值断言面，M0 分母
/// 4,782 子测试）。filter 按路径子串过滤。
pub fn run_css_flexbox_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-flexbox", filter)
}

/// Run the pinned upstream css-multicol testharness subset（css 二期 M2 批次
/// 4——多列布局属性（columns/column-gap 等）解析断言面，M0 分母 1,614
/// 子测试）。filter 按路径子串过滤。
pub fn run_css_multicol_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-multicol", filter)
}

/// Run the pinned upstream css-position testharness subset（css 二期 M2 批次
/// 4——定位属性（position/inset/z-index）解析与计算值断言面，M0 分母
/// 1,507 子测试）。filter 按路径子串过滤。
pub fn run_css_position_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-position", filter)
}

/// Run the pinned upstream css-overflow testharness subset（css 二期 M2 批次
/// 4——溢出属性（overflow/overflow-clip-margin 等）解析断言面，M0 分母
/// 1,200 子测试）。filter 按路径子串过滤。
pub fn run_css_overflow_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-overflow", filter)
}

/// Run the pinned upstream css-tables testharness subset（css 二期 M2 批次
/// 4——表格布局属性（border-collapse/table-layout 等）解析断言面，M0 分母
/// 939 子测试）。filter 按路径子串过滤。
pub fn run_css_tables_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-tables", filter)
}

/// Run the pinned upstream css-lists testharness subset（css 二期 M2 批次 4——
/// 列表属性（list-style/counter）解析与序列化断言面，M0 分母 972 子测试）。
/// filter 按路径子串过滤。
pub fn run_css_lists_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-lists", filter)
}

/// Run the pinned upstream css-inline testharness subset（css 二期 M2 批次 4——
/// 行内布局属性（vertical-align/line-height 等）解析断言面，M0 分母 857
/// 子测试）。filter 按路径子串过滤。
pub fn run_css_inline_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-inline", filter)
}

/// Run the pinned upstream css-scroll-snap testharness subset（css 二期 M2
/// 批次 4——scroll-snap 属性族解析断言面，M0 分母 845 子测试）。filter 按
/// 路径子串过滤。
pub fn run_css_scroll_snap_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-scroll-snap", filter)
}

/// Run the pinned upstream css-viewport testharness subset（css 二期 M2 批次
/// 4——视口描述符与 zoom 解析断言面，M0 分母 780 子测试）。filter 按路径
/// 子串过滤。
pub fn run_css_viewport_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-viewport", filter)
}

/// Run the pinned upstream css-break testharness subset（css 二期 M2 批次 4——
/// 分页/断行属性（break-inside/box-decoration-break 等）解析断言面，M0
/// 分母 717 子测试）。filter 按路径子串过滤。
pub fn run_css_break_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-break", filter)
}

/// Run the pinned upstream css-contain testharness subset（css 二期 M2 批次
/// 4——contain/content-visibility 解析断言面，M0 分母 497 子测试）。filter
/// 按路径子串过滤。
pub fn run_css_contain_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-contain", filter)
}

/// Run the pinned upstream css-writing-modes testharness subset（css 二期 M2
/// 批次 4——书写模式属性（writing-mode/direction 等）解析断言面，M0 分母
/// 419 子测试）。filter 按路径子串过滤。
pub fn run_css_writing_modes_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_css_corpus_dir(wpt_root, "css/css-writing-modes", filter)
}
