//! DnD 兼容 goal（dnd-compat M1 / DC-1）——上游 html/editing/dnd/ corpus 的
//! testharness 可执行子集通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_page_state
//! 先例），经 `pub(crate)` 复用其执行底座（`run_testharness_html_inner` /
//! `corpus_case_timeout`），行为与既有通道一致。
//!
//! 用例由 `scripts/goals/28-dnd-compat.sh` 按需拉到 `wpt-data/`（gitignored，
//! WPT pin 315976933870b34d6ea30e3f6643403edae678ba）。语料全量 957 文件做
//! census（git trees 深挖拉取），运行面 = [`DND_TEST_SUBDIRS`] 目录扫描 +
//! [`dnd_case_skipped`] 内容规则筛减（skip 域 = goal 契约排除面，两侧同源防漂移）。
//!
//! 语料形态注记（2026-10-18 M1 盘点）：
//! - 上游**无顶层 `dnd/` 目录**（Contents API 404 实证）——拖放语料在
//!   `html/editing/dnd/`；立项时 goal 脚本的 `DIRS=("dnd")` 为路径错误，已修正。
//! - 957 文件中绝大多数为**真实拖拽交互面**（`-manual` 后缀 184 文件 + IE
//!   testcenter 遗产自校验页——无 testharness 引用、PASS/FAIL 文案由真实拖拽落
//!   DOM；以及 testdriver Actions 指针链依赖案）——全数属 M3 用户门控面
//!   （host drag-input 管线挂账，web-api-batch2 先例），不在本通道分母。
//! - 纯 JS 可执行面：`dom/`（draggable 反射 + ondrag* handler IDL 面）、
//!   `synthetic/001.html`（DragEvent 构造器/init*/合成派发）、`datastore/*.html`
//!   （DataTransfer 构造器/types FrozenArray 语义/ItemList remove 语义）、
//!   `the-draggable-attribute/`、`the-dropzone-attribute/dropzone_attribute.html`、
//!   顶层 `historical.html`（ondragexit 移除断言）。
//! - 三 interface 目录（the-datatransfer-interface 9 案 / the-datatransferitem-
//!   interface 1 案 / the-dragevent-interface 1 案）上游**全为 `-manual`**——
//!   DataTransfer/DragEvent 构造器面的上游 testharness 覆盖即 `synthetic/001.html`
//!   + `datastore/*.html` 三案；目录仍在扫描面内以使 skip 记账可见。
//! - `.xhtml`（datastore/events/dropzone 的 IE 遗产编号案）不进扫描面
//!   （testharness 通道无 xhtml 页管道——page_state 只扫 .html 同款）。
//! - `js-file-*.html`（顶层）虽为 testharness，但断言经 `test_driver.Actions()`
//!   指针链驱动真实拖放——按内容规则排除（M3 门控面，约束 2「不预建交互管线
//!   占位」）。

use crate::testharness::{HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_testharness_html_inner};
use std::path::Path;

/// dnd corpus 扫描子目录（可执行面 + 三 interface 目录供 skip 记账；helper
/// 目录 `resources/` 与交互面目录不进扫描面）。
pub const DND_TEST_SUBDIRS: &[&str] = &[
    "html/editing/dnd",
    "html/editing/dnd/dom",
    "html/editing/dnd/synthetic",
    "html/editing/dnd/datastore",
    "html/editing/dnd/the-datatransfer-interface",
    "html/editing/dnd/the-datatransferitem-interface",
    "html/editing/dnd/the-dragevent-interface",
    "html/editing/dnd/the-draggable-attribute",
    "html/editing/dnd/the-dropzone-attribute",
];

/// `the-draggable-attribute/draggable_attribute.html` 引用的元素清单 helper
/// （`/html/semantics/interfaces.js` 绝对路径——`inline_local_scripts` 只内联相对
/// 路径，绝对路径经 `inline_extras` 注入，canvas-tests.js 先例）。
const INTERFACES_JS_PATH: &str = "html/semantics/interfaces.js";
const INTERFACES_JS_SRC: &str = "/html/semantics/interfaces.js";

/// [`DND_TEST_SUBDIRS`] 用例排除规则（goal 契约排除面的 runner 侧镜像）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文。
///
/// - `-manual.` 后缀（真实用户拖拽——timing/page_state 先例；中缀匹配）：
///   三 interface 目录全案 + synthetic 005/006 + the-dropzone-attribute 20 案 +
///   datastore/015 等
/// - 无 testharness 引用（IE testcenter 遗产自校验页——PASS/FAIL 文案由真实拖拽
///   落 DOM；datastore/events/dropzone 编号案 + helper-drop-box-here.xhtml 形态；
///   本通道只扫 .html，此规则兜住 datastore/053~058 无 harness 引用的 .html）
/// - 内容标记 `test_driver.Actions`（指针链拖拽输入注入——host drag-input 管线
///   M3 用户门控面，goal 约束 2 不预建占位）：js-file-fallback-text-only、
///   js-file-image-drag
pub(crate) fn dnd_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.contains("-manual.") {
        return true;
    }
    if !source.contains("testharness.js") {
        return true;
    }
    if source.contains("test_driver.Actions") {
        return true;
    }
    false
}

/// Run the imported upstream dnd testharness executable subset
/// （dnd-compat goal M1 / DC-1）。filter 按路径子串过滤（如 `datastore/`、
/// `dom/`——基线按套件分类）。
pub fn run_dnd_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
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
    // 元素清单 helper（draggable_attribute.html 依赖）；缺失时用例照跑——
    // `elements` 未定义的 ReferenceError 如实记 Fail，不静默吞。
    let interfaces_source = std::fs::read_to_string(wpt_root.join(INTERFACES_JS_PATH)).ok();
    let mut cases = Vec::new();
    for subdir in DND_TEST_SUBDIRS {
        let Ok(entries) = std::fs::read_dir(wpt_root.join(subdir)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                continue;
            }
            let file_name = entry.file_name().to_string_lossy().to_string();
            if !file_name.ends_with(".html") {
                continue;
            }
            let relative = format!("{subdir}/{file_name}");
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            let Ok(source) = std::fs::read_to_string(&path) else {
                continue;
            };
            if dnd_case_skipped(&relative, &source) {
                continue;
            }
            // 绝对路径 helper 内联（canvas-tests.js inline_extras 先例）。
            let extras: Vec<(&str, &str)> = if source.contains(INTERFACES_JS_SRC) {
                interfaces_source
                    .as_deref()
                    .map(|src| vec![(INTERFACES_JS_SRC, src)])
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            let results = run_testharness_html_inner(
                wpt_root,
                &relative,
                &source,
                &harness_source,
                &extras,
                corpus_case_timeout(),
            );
            cases.push((relative, results));
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}
