//! Pointer Lock 兼容 goal（pointerlock-compat M1 / DC-1）——上游 `pointerlock/`
//! corpus 的可执行子集通道（门控 goal，学 dnd/webgl：盘点自主、管线门控）。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_dnd 先例），
//! 经 `pub` 复用其执行底座（[`crate::testharness::run_testharness_html`] /
//! [`crate::testharness::corpus_case_timeout`]），行为与既有通道一致。
//!
//! 用例由 `scripts/goals/29-pointerlock-compat.sh` 按需拉到 `wpt-data/`
//! （gitignored，WPT pin 315976933870b34d6ea30e3f6643403edae678ba）。语料为顶层
//! 平铺布局（27 文件 = 25 .html + idlharness.window.js + test-helper.js，
//! 2026-10-18 对上游 Contents API 清单核对无缺）。
//!
//! 语料形态注记（2026-10-18 M1 盘点）：
//! - **静态可执行面**：`constructor.html`（MouseEvent `movementX/Y` init 字典
//!   扩展默认值——无 testdriver、无 requestPointerLock，纯构造器断言）。
//! - **异常语义面**：`pointerlock_without_gesture.html`（无用户激活双请求须双
//!   NotAllowedError 拒绝）+ `pointerlock-requires-gesture.html`（load 期请求须
//!   pointerlockerror + 拒绝）——不需要真实捕获。M1 实测红：shim R2939 的
//!   `requestPointerLock`（part04，grant/deny 二分）无 transient activation 门，
//!   默认 grant——`pointerlockchange` 携带非空 `pointerLockElement` 派发使
//!   `unreached_func` 触发。红归门控实现面（M2/M3 RFC 范围），不是可自主收敛
//!   的引擎缺陷。
//! - **捕获管线门控面**（[`pointerlock_case_skipped`] 内容规则排除）：testdriver
//!   `test_driver.*` 调用（13 案）或 `test-helper.js` 引用（`lockPointerAndWait
//!   ForEvent`/`unlockPointerAndWaitForEvent` 真锁获取——api_behavior/
//!   remove_target 两案）驱动的真实锁定授予面：相对位移流、事件序、boundary
//!   事件抑制、unadjustedMovement、fullscreen 联动——**未经用户点名不实现**
//!   （goal 契约 DC-2/M2 门控）。
//! - `-manual.` 后缀（7 案）：真实用户交互（interact flag / ESC 释放 / 跨 Tab
//!   离开），照 uievents/timing 先例排除。
//! - `idlharness.window.js`：不在扫描面（只扫 .html）；即便拉平也依赖
//!   /resources/WebIDLParser.js build 期资产——clipboard/fileapi/console 先例
//!   记账排除。
//! - `test-helper.js`：非用例，helper 目录同 `resources/` 先例不进扫描面；
//!   引用它的用例按门控面内容规则处理（见上）。

use crate::testharness::{HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_testharness_html};
use std::path::Path;

/// pointerlock corpus 扫描目录（上游平铺布局，无子目录）。
pub const POINTERLOCK_TEST_SUBDIRS: &[&str] = &["pointerlock"];

/// [`POINTERLOCK_TEST_SUBDIRS`] 用例排除规则（goal 契约排除面的 runner 侧镜像）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文。
///
/// - `-manual.` 后缀（真实用户交互——uievents/timing 先例；中缀匹配）
/// - `test_driver.` 调用或 `test-helper.js` 引用（捕获管线门控面——真锁获取
///   依赖宿主光标管线，未经用户点名不实现；dnd「不预建交互管线占位」同款）
/// - 无 testharness 引用（自校验页兜底——dnd IE testcenter 遗产先例）
pub(crate) fn pointerlock_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.contains("-manual.") {
        return true;
    }
    // pointerlock-compat M3 s2：movementX_Y_basic 只断言 mousemove 位移流
    // （movementX == screenX 前后差，Pointer Lock spec #dom-mouseevent-movementx），
    // 不取真锁——其 test_driver.Actions 步走 runner 合成指针管线即满足，窄化放行
    // （捕获管线门控例外，同 s1 B 面两案先例）。
    if name == "movementX_Y_basic.html" {
        return false;
    }
    if source.contains("test_driver.") || source.contains("test-helper.js") {
        return true;
    }
    if !source.contains("testharness.js") {
        return true;
    }
    false
}

/// Run the imported upstream pointerlock testharness executable subset
/// （pointerlock-compat goal M1 / DC-1）。filter 按路径子串过滤。
pub fn run_pointerlock_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
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
    for subdir in POINTERLOCK_TEST_SUBDIRS {
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
            if pointerlock_case_skipped(&relative, &source) {
                continue;
            }
            let results = run_testharness_html(wpt_root, &relative, &source, &harness_source, corpus_case_timeout());
            cases.push((relative, results));
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}
