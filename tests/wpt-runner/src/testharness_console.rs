//! Console 兼容 goal（console-compat M1 / DC-1）—— console IDL 面（namespace
//! 形态/历史遗留方法不存在）与格式化/状态语义（label 转换、参数容错）上游
//! testharness window 可执行子集通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_cookies
//! 先例），复用其 flat 扫描底座 `run_any_js_corpus_subdirs`（.html 直跑 +
//! .any.js window 变体，net-api/encoding 同构）。
//!
//! 用例由 `scripts/goals/21-console-compat.sh` 按需拉到 `wpt-data/console/`
//! （gitignored，WPT pin 315976933870b34d6ea30e3f6643403edae678ba）。
//!
//! 语料形态注记（2026-10-14 M1 盘点）：上游 console corpus 分两层——
//! **纯 testharness 面**（`.any.js`：namespace property descriptor /
//! @@toStringTag、count/time label 的 toString 转换与异常再抛、历史遗留方法
//! 不存在、Symbol/大数组参数容错）为可执行验收面；**WebDriver BiDi
//! `log.entryAdded` 面**（`.html`：count/countReset/timing/format-specifier
//! 的 logging 观测——`test_driver.bidi` 捕获通道）runner 侧未提供，经
//! `unsupported_testdriver_dependencies` 整案 Unsupported（诚实记账不 skip，
//! 分母可见；捕获通道落地后零改动自动重入）。甄别结论与基线见
//! docs/goal/archive/console-compat/evidence/2026-10-14-m1-baseline.md。

use crate::testharness::{HarnessSubtestResult, run_any_js_corpus_subdirs};
use std::path::Path;

/// console corpus 扫描子目录（上游 console/ flat 布局；`helper.js` 非
/// .html/.any.js 形态自滤，经 `<script src>` 原样加载）。
pub const CONSOLE_TEST_SUBDIRS: &[&str] = &["console"];

/// [`CONSOLE_TEST_SUBDIRS`] 用例排除规则（goal 契约排除面的 runner 侧镜像）。
///
/// - `idlharness.*`（依赖 /resources/WebIDLParser.js——上游 resources/webidl2 的
///   build 期生成资产，repo 内不存在，window 可执行面之外；timing/clipboard
///   idlharness 先例）
/// - `-manual.html` 后缀（人工目检形态——console-string-format-specifier-symbol-manual）
///
/// BiDi log.entryAdded 依赖案**不在本规则 skip**：交
/// `run_testharness_html_inner` 的 testdriver 依赖检查判 Unsupported
/// （`unsupported testdriver API: bidi`）——不可执行面进分母记账。
pub(crate) fn console_case_skipped(relative: &str, source: &str) -> bool {
    let _ = source;
    let name = relative.rsplit('/').next().unwrap_or(relative);
    name.starts_with("idlharness.") || name.ends_with("-manual.html")
}

/// Run the pinned upstream console window subset（console-compat goal M1 / DC-1）。
/// filter 按路径子串过滤（如 `count`、`timing`、`namespace`——基线按面分类）。
pub fn run_console_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    run_any_js_corpus_subdirs(wpt_root, CONSOLE_TEST_SUBDIRS, filter, console_case_skipped)
}
