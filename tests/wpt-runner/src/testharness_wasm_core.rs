//! WebAssembly 兼容扩面 goal（wasm-compat M1 / DC-2）——`wasm/core` 全量 +
//! `wasm/webapi` 上游 testharness 通道（docs/goal/archive/wasm-compat.md）。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_workers
//! 先例），经 `pub(crate)` 复用其执行底座（`run_any_js_window_case` /
//! `run_testharness_html_inner` / `corpus_case_timeout`）。既有 jsapi 通道
//! （`testharness::WASM_CASES` 固定清单 + `testharness-wasm` 子命令）是 DC-4
//! 回归底线，**保持字节不动**；本模块只覆盖扩面语料。
//!
//! 用例由 `scripts/goals/32-wasm-compat.sh`（GOAL_FETCH_DEEP=1 git trees 深挖 +
//! GOAL_PULL_ANY_JS=1 + GOAL_PULL_WASM=1）按 pin 315976933870b34d6ea30e3f6643403edae678ba
//! 拉到 `wpt-data/wasm/core/`、`wpt-data/wasm/webapi/`（gitignored）。
//!
//! 语料形态（pin 盘点，530 + 90 blobs）：
//! - `wasm/core/**/*.<name>.wast.js.html`——spec 解释器套件编译产物（wrapper 页）：
//!   加载 `/resources/testharness.js` + `./js/harness/{async,sync}_index.js`（WAST
//!   harness：`module()`/`instance()`/`call()`/`assert_return()` 等，wasm 二进制
//!   以 `\xNN` 转义字面内嵌于 `.wast.js`，无外部资产依赖）+ `./js/**.wast.js` 用例
//!   正文。相对 src 由页面 script fetcher 按 case 目录解析（wpt_data_script_fetcher）。
//!   顶层 MVP+已定稿 proposal；`bulk-memory/`、`exceptions/`、`gc/`、`memory64/`、
//!   `multi-memory/`、`relaxed-simd/`、`simd/` 子目录为 proposal 族——**不设目录级
//!   排除**，基线按 proposal 归簇（M2 主目标）。
//! - `wasm/webapi/*.any.js`——compile/instantiateStreaming 与 fetch 集成面；
//!   `resources/*.wasm` 二进制资产 + `.headers` sidecar（Content-Type 覆盖，
//!   wptserve 语义 fetch 通道已支持——`wpt_sidecar_content_type`）。
//!
//! 运行面 = 目录递归扫描（workers 先例）+ [`wasm_core_case_skipped`] 内容规则筛减。
//! `.any.js` 走 window 变体通道（goal 契约「含 .any.js window 变体通道」；
//! `run_any_js_window_case`），`.html`/`.htm`（含 wrapper 页）走标准 html 通道。
//! dedicated/shared/service worker-only 变体（`// META: global=` 无 window token）
//! M1 不设通道——skip 记账（jsapi 通道同口径先例），挂账 M2 归簇评估。

use crate::testharness::{
    HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_any_js_window_case, run_testharness_html_inner,
};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// wasm-compat 扩面 corpus 扫描根（wpt-data 相对）。
const WASM_CORE_TEST_DIR: &str = "wasm/core";
const WASM_WEBAPI_TEST_DIR: &str = "wasm/webapi";

/// [`WASM_CORE_TEST_DIR`]/[`WASM_WEBAPI_TEST_DIR`] 用例排除规则（goal 契约排除面
/// 的 runner 侧镜像；fetch 侧全量拉取，两侧同源防漂移——workers 先例）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文。
///
/// - `esm-integration/`（wasm/webapi）：WASM ESM module integration——module
///   script/source-phase import/worker 变体，runner 无 module-script 管道
///   （page-wasm goal fetch 脚本 skip 域同款先例，M1 挂账）。
/// - `idlharness.*`：依赖 /resources/WebIDLParser.js build 期生成资产
///   （compression/webcrypto/timing 先例）。
/// - `.html`/`.htm` 不含 `testharness` 引用：reftest 参照页/manual/demo
///   （workers 先例——wrapper 页自带 `/resources/testharness.js` 引用，天然放行）。
/// - `.any.js` `// META: global=` 无 `window` token：worker-only 变体，M1 window
///   变体通道之外（模块头注记），skip 记账。
fn wasm_core_case_skipped(relative: &str, source: &str) -> bool {
    if relative.contains("esm-integration/") {
        return true;
    }
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.starts_with("idlharness.") {
        return true;
    }
    if (relative.ends_with(".html") || relative.ends_with(".htm")) && !source.contains("testharness") {
        return true;
    }
    if relative.ends_with(".any.js") && !wpt_meta_global_includes_window(source) {
        return true;
    }
    false
}

/// Parse `// META: global=` tokens（workers 模块 `wpt_meta_globals` 同语义——
/// 多行/逗号并集 + `//META:` 无空格兼容）。此处只回答「含 window 变体」；
/// 完整分道（dedicated 通道）挂 M2。无 `global=` META 的 any.js 上游默认
/// window 变体（wptserve AnyHtmlHandler 无条件面），放行。
fn wpt_meta_global_includes_window(source: &str) -> bool {
    let mut any_global_meta = false;
    for line in source.lines() {
        let trimmed = line.trim();
        let Some(meta_value) = trimmed
            .strip_prefix("// META: ")
            .or_else(|| trimmed.strip_prefix("//META: "))
        else {
            continue;
        };
        let Some(value) = meta_value.strip_prefix("global=") else {
            continue;
        };
        any_global_meta = true;
        if value.split(',').any(|token| token.trim() == "window") {
            return true;
        }
    }
    !any_global_meta
}

/// Run the pinned upstream wasm/core + wasm/webapi window subset
/// （wasm-compat goal M1 / DC-2——语料扩面基线通道）。目录递归扫描（workers 先例；
/// 排序收集保证基线顺序跨轮可复现），`.any.js`/`.window.js` 走
/// [`run_any_js_window_case`] window 变体，`.html`/`.htm` 走标准 html 通道。
/// skip 规则见 [`wasm_core_case_skipped`]。filter 按路径子串过滤
/// （如 `wasm/core/simd`、`instantiateStreaming`——基线按 proposal/面分类）。
pub fn run_wasm_core_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{WASM_CORE_TEST_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    // 递归收集用例路径（语料两层深：`wasm/core/simd/`、`wasm/core/js/`——js/ 是
    // wrapper 页的支撑产物目录，`.wast.js` 非 `.any.js` 形态天然不进扫描面）。
    let mut case_paths = Vec::new();
    for root_dir in [WASM_CORE_TEST_DIR, WASM_WEBAPI_TEST_DIR] {
        let mut stack = vec![wpt_root.join(root_dir)];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".any.js")
                    || name.ends_with(".window.js")
                    || name.ends_with(".html")
                    || name.ends_with(".htm")
                {
                    case_paths.push(path);
                }
            }
        }
    }
    case_paths.sort();
    let mut cases = Vec::new();
    for path in case_paths {
        let relative = match path.strip_prefix(wpt_root) {
            Ok(relative) => relative.to_string_lossy().to_string(),
            Err(_) => continue,
        };
        if filter.is_some_and(|filter| !relative.contains(filter)) {
            continue;
        }
        // lossy 解码（workers 同款）——语料理论合法 UTF-8，防御性兜底不静默丢案。
        let Ok(source) = read_corpus_to_string(&path) else {
            continue;
        };
        if wasm_core_case_skipped(&relative, &source) {
            continue;
        }
        let results = if relative.ends_with(".html") || relative.ends_with(".htm") {
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
    cases
}

#[cfg(test)]
mod tests {
    use super::*;

    /// wrapper 页形态：testharness 引用放行（spec 解释器套件执行单元）。
    #[test]
    fn wrapper_html_with_harness_runs() {
        let source = "<script src=\"/resources/testharness.js\"></script>\n\
                      <script src=\"./js/harness/async_index.js\"></script>\n\
                      <script src=\"./js/address.wast.js\"></script>";
        assert!(!wasm_core_case_skipped("wasm/core/address.wast.js.html", source));
    }

    /// 无 testharness 引用的 html（reftest 参照/manual 页）排除。
    #[test]
    fn plain_html_without_harness_skipped() {
        assert!(wasm_core_case_skipped("wasm/core/foo-ref.html", "<div>ref</div>"));
    }

    /// esm-integration 域整域排除（module-script 管道缺口，M1 挂账）。
    #[test]
    fn esm_integration_skipped() {
        assert!(wasm_core_case_skipped(
            "wasm/webapi/esm-integration/source-phase.tentative.html",
            "<script type=\"module\">import source</script> testharness"
        ));
    }

    /// idlharness 排除（WebIDLParser build 期资产，compression/webcrypto 先例）。
    #[test]
    fn idlharness_skipped() {
        assert!(wasm_core_case_skipped(
            "wasm/webapi/idlharness.any.js",
            "// META: global=window\nidlharness();"
        ));
    }

    /// `.any.js` window 变体分道：含 window token 放行；worker-only skip；
    /// 无 global= META 默认 window 放行；`//META:` 无空格兼容（upstream serve.py
    /// meta 解析，workers 模块同款）。
    #[test]
    fn any_js_window_lane_split() {
        assert!(!wasm_core_case_skipped(
            "wasm/webapi/abort.any.js",
            "// META: global=window,dedicatedworker,sharedworker\npromise_test(t => {});"
        ));
        assert!(wasm_core_case_skipped(
            "wasm/webapi/worker-only.any.js",
            "// META: global=dedicatedworker,sharedworker\npromise_test(t => {});"
        ));
        assert!(!wasm_core_case_skipped(
            "wasm/webapi/no-meta.any.js",
            "promise_test(t => {});"
        ));
        assert!(!wasm_core_case_skipped(
            "wasm/webapi/no-space.any.js",
            "//META: global=window\npromise_test(t => {});"
        ));
    }
}
