//! WebCryptoAPI corpus testharness 通道（webcrypto-compat goal）——`.any.js`/
//! `.window.js` window 腿 + `.any.js` dedicated worker 腿（M2 worker 腿接线）。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束 + webcrypto 域内聚，
//! testharness_webmessaging/workers 先例），经 `pub(crate)` 复用其执行底座
//! （`run_testharness_html` / `any_js_window_wrapper` / `wpt_meta_scripts` /
//! `wpt_js_has_long_timeout` / `CASE_TIMEOUT`）。用例由
//! `scripts/goals/26-webcrypto-compat.sh` 按需拉到 `wpt-data/WebCryptoAPI/`
//! （gitignored，WPT pin 子树 `084495e783…`）。
//!
//! 双腿分道（M2 worker 腿接线——上游 wptserve `.any.js` 变体语义）：
//! - window 腿：`.any.js` / `.window.js` 全量（M1 起既有面，台账键不变——
//!   跨轮零回归对账锚）；
//! - dedicated worker 腿：`.any.js` 且 global tokens 含 dedicated 面（无
//!   `// META: global=` 缺省 window+dedicatedworker 双变体——上游 test_format
//!   语义；显式 tokens 含 `worker` shorthand 或 `dedicatedworker`）。台账键 =
//!   worker 变体 URL 名（`foo.https.any.worker.js`——upstream wptserve 对外
//!   命名，与磁盘文件无碰撞；window 键保持源文件名跨轮可比）。仅
//!   `sharedworker`/`serviceworker` → 无 dedicated 腿（通道不在 goal 面）。
//!
//! worker 腿执行形态（[`crate::testharness_workers`] 通道同构）：页面
//! `fetch_tests_from_worker(new Worker('/…any.worker.js'))` 聚合；worker 顶脚本
//! 经 fetcher `.any.worker.js` → 磁盘 `.any.js` 映射 + ClassicWorkerHandler 同构
//! 包装（GLOBAL flags + importScripts testharness/META 依赖 + 语料正文内嵌 +
//! `done()`）。engine 侧 worker 全局 crypto 绑定（part05 `wctx.crypto` +
//! CryptoKey/SubtleCrypto/TextEncoder/TextDecoder 镜像，`[SecureContext]` 门禁
//! 随页锚 scheme）为本切片配套。
//!
//! 上报案名 window 腿在前 worker 腿在后逐案相邻，排序收集保证基线顺序可复现。

use crate::testharness::{
    CASE_TIMEOUT, HarnessStatus, HarnessSubtestResult, any_js_window_wrapper, run_testharness_html,
    wpt_js_has_long_timeout, wpt_meta_scripts,
};
use crate::testharness_workers::wpt_meta_globals;
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// corpus 目录——fetch 脚本 `scripts/goals/26-webcrypto-compat.sh`（同 WPT_REV
/// pin，git trees 深挖 `lib.sh fetch_dir_deep`）拉取 `.any.js`/`.window.js` 用例
/// 与 vectors/support 到 `wpt-data/WebCryptoAPI/`（gitignored）。
const WEBCRYPTO_TEST_DIR: &str = "WebCryptoAPI";

/// [`WEBCRYPTO_TEST_DIR`] 用例名排除规则（runner 侧记账排除，fetch 侧全量拉取）。
/// - `idlharness.*`：依赖 /resources/WebIDLParser.js 与 idlharness.js build 期生成
///   资产，repo 内不存在，window 不可执行（compression idlharness 先例）。
/// - `secure_context/` 的 `.sub.html` 为 include 资产非用例（扫描只取 .js 形态，
///   天然排除）。
fn webcrypto_case_skipped(relative: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    name.starts_with("idlharness.")
}

/// WebCryptoAPI corpus 页面 scheme 判定（webcrypto-compat 残留收口）：上游按
/// 文件名 `.https.` 中缀双态——带中缀的案（digest/derive/sign_verify 等全部
/// 语义案）跑 https 安全上下文，无中缀的案（historical.any.js 断言非安全上下文
/// 无 crypto.subtle/CryptoKey、getRandomValues.any.js 与 scheme 无关）跑 http。
/// runner 页锚同口径翻转，shim 侧 [SecureContext] 门禁（part01b `_zwSubtle`）
/// 随之生效。worker 腿键继承同名判定（同 page 同源 worker）。
pub(crate) fn webcrypto_case_uses_http_page(case_name: &str) -> bool {
    let Some(name) = case_name.strip_prefix("WebCryptoAPI/") else {
        return false;
    };
    let file = name.rsplit('/').next().unwrap_or(name);
    !file.contains(".https.")
}

/// dedicated worker 腿判定（`// META: global=` 解析——[`wpt_meta_globals`] 同源）。
/// 无 `global=` META 的 `.any.js` 上游缺省 window+dedicatedworker 双变体（WPT
/// test_format「any.js runs in window and dedicated worker by default」）；
/// 显式 tokens 含 `worker`（shorthand，dedicated+shared+service 全 worker 面）
/// 或 `dedicatedworker` 即含 dedicated 腿。仅 `sharedworker`/`serviceworker`
/// → 无 dedicated 腿（runner 无该通道，goal 面外）。
fn webcrypto_has_dedicated_lane(source: &str) -> bool {
    let globals = wpt_meta_globals(source);
    if globals.is_empty() {
        return true;
    }
    globals
        .iter()
        .any(|global| global == "worker" || global == "dedicatedworker")
}

/// dedicated worker 腿执行：页面 `fetch_tests_from_worker(new Worker(url))`
/// 聚合（testharness_workers `.worker.js` 通道同款页面模板）。worker 顶脚本由
/// fetcher 侧包装（`.any.worker.js` → 磁盘 `.any.js`，ClassicWorkerHandler
/// 同构），页面只载 testharness.js + 聚合调用。`// META: timeout=long` 经
/// [`wpt_js_has_long_timeout`] 升 timeout_meta（window 腿
/// [`any_js_window_wrapper`] 同款——`run_testharness_html` 按 HTML meta 放宽）。
/// 案名传 worker 变体键（页锚 scheme 判定与 worker 聚合上报同键）。
fn run_webcrypto_worker_leg(
    wpt_root: &Path,
    worker_key: &str,
    case_source: &str,
    harness_source: &str,
) -> Vec<HarnessSubtestResult> {
    let timeout_meta = if wpt_js_has_long_timeout(case_source) {
        "<meta name=\"timeout\" content=\"long\">"
    } else {
        ""
    };
    let page = format!(
        "<!doctype html><meta charset=\"utf-8\">{timeout_meta}\
         <script src=\"/resources/testharness.js\"></script>\
         <script>fetch_tests_from_worker(new Worker('/{worker_key}'));</script>"
    );
    run_testharness_html(wpt_root, worker_key, &page, harness_source, CASE_TIMEOUT)
}

/// Run the pinned upstream WebCryptoAPI corpus（webcrypto-compat goal）——
/// window 腿（M1 / DC-1 分域基线）+ `.any.js` dedicated worker 腿（M2）。
/// 目录**递归**扫描 `.any.js` / `.window.js`（用例入口形态；语料埋二三层子目录），
/// META script 自动解析（vectors/support 与用例同树）。window 腿经
/// [`any_js_window_wrapper`] 内联执行；worker 腿页页面聚合（见模块注释）。
/// 排序收集保证基线顺序可复现。filter 按路径子串过滤（window 键命中即含其
/// worker 腿——worker 键为 window 键派生，`make testharness-webcrypto
/// FILTER=digest` 双腿同滤）。
pub fn run_webcrypto_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let harness_source = match read_corpus_to_string(wpt_root.join("resources/testharness.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{WEBCRYPTO_TEST_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error.to_string()),
                }],
            )];
        }
    };
    // 递归收集用例路径（含子目录；排序保证跨轮基线可复现）。
    let mut case_paths = Vec::new();
    let mut stack = vec![wpt_root.join(WEBCRYPTO_TEST_DIR)];
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
            if name.ends_with(".any.js") || name.ends_with(".window.js") {
                case_paths.push(path);
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
        if filter.is_some_and(|filter| !relative.contains(filter)) || webcrypto_case_skipped(&relative) {
            continue;
        }
        let Ok(case_source) = read_corpus_to_string(&path) else {
            continue;
        };
        let case_dir = Path::new(&relative).parent().unwrap_or_else(|| Path::new(""));
        let mut support_sources = Vec::new();
        let mut support_failed = false;
        let mut support_fail_results = Vec::new();
        for script in wpt_meta_scripts(&relative, &case_source) {
            let support_path = match script.strip_prefix('/') {
                Some(root_relative) => wpt_root.join(root_relative),
                None => wpt_root.join(case_dir).join(&script),
            };
            match read_corpus_to_string(support_path) {
                Ok(source) => support_sources.push((script, source)),
                Err(error) => {
                    support_fail_results.push(HarnessSubtestResult {
                        name: format!("load WebCryptoAPI support {script}"),
                        status: HarnessStatus::Fail,
                        message: Some(error.to_string()),
                    });
                    support_failed = true;
                    break;
                }
            }
        }
        let is_any_js = relative.ends_with(".any.js");
        let has_worker_lane = is_any_js && webcrypto_has_dedicated_lane(&case_source);
        if support_failed {
            // 支撑缺失记账双腿镜像（window 键既有行为；worker 键同记录保持
            // 确定性——不在 worker 内 importScripts 失败面悬置）。
            cases.push((relative.clone(), support_fail_results.clone()));
            if has_worker_lane {
                let worker_key = relative.replacen(".any.js", ".any.worker.js", 1);
                cases.push((worker_key, support_fail_results));
            }
            continue;
        }
        let support_refs = support_sources
            .iter()
            .map(|(name, source)| (name.as_str(), source.as_str()))
            .collect::<Vec<_>>();
        let html = any_js_window_wrapper(&relative, &support_refs, &case_source);
        let results = run_testharness_html(wpt_root, &relative, &html, &harness_source, CASE_TIMEOUT);
        cases.push((relative.clone(), results));
        if has_worker_lane {
            let worker_key = relative.replacen(".any.js", ".any.worker.js", 1);
            let worker_results = run_webcrypto_worker_leg(wpt_root, &worker_key, &case_source, &harness_source);
            cases.push((worker_key, worker_results));
        }
    }
    cases
}
