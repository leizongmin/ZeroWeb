//! Dedicated Worker 兼容 goal（workers-compat M1 导入 / M2 worker harness 通道）——
//! `workers/` + `dedicated-workers/` 上游 testharness 通道。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_webmessaging
//! 先例），经 `pub(crate)` 复用其执行底座（`run_any_js_window_case` /
//! `run_testharness_html_inner` / `corpus_case_timeout` / `wpt_meta_scripts`），
//! 行为与既有通道一致。
//!
//! 用例由 `scripts/goals/70-workers-compat.sh` 按需拉到 `wpt-data/workers/` 与
//! `wpt-data/dedicated-workers/`（gitignored，WPT pin
//! 315976933870b34d6ea30e3f6643403edae678ba）。运行面 = 目录扫描（webmessaging
//! 先例，skip 规则两侧同源防漂移）+ [`workers_case_skipped`] 内容规则筛减
//! + `.any.js` 按 `// META: global=` 变体分道（M2）。
//!
//! 变体分道（M2——`tools/serve/serve.py` ClassicWorkerHandler 同构）：
//! - window 含 `window` global → window 变体（M1 面，`any_js_window_wrapper` 同款）；
//! - dedicated 变体（`worker` shorthand 或 `dedicatedworker`）→ dedicated worker
//!   通道：上游 `.any.worker.js` 是薄包裹（GLOBAL flags + importScripts
//!   testharness + importScripts 原文件 + done()）——runner 无文件服务面，包裹源以
//!   data: URL 内联（shim `_zwDecodeWorkerScript` 只解 data: 形态，canvas
//!   worker 通道先例的 fetch_tests_from_worker 聚合管道），语料正文经根绝对
//!   `importScripts` 由 `wpt_data_script_fetcher` 供源（根绝对路径与 base 无关）；
//! - 仅 `sharedworker` / `serviceworker` → 跳过（SharedWorker 挂账 / 超出 goal 面）。
//!
//! `.worker.js` in-worker harness 形态（21 案，M2 重入）：上游原样作 worker 脚本
//! 供源（WorkersHandler 外链形态），页面 `fetch_tests_from_worker(new Worker(url))`
//! 聚合——canvas 通道同款。
//!
//! 本 goal 的 `new Worker(` window 观测面是验收主体（webmessaging M1 曾把该面挂
//! 本 goal，M1 归位），不作为 skip 标记；iframe / multi-global / secure-context
//! 面按 goal 契约排除。

use crate::testharness::{
    HarnessStatus, HarnessSubtestResult, corpus_case_timeout, normalize_relative, run_any_js_window_case,
    run_testharness_html_inner, wpt_meta_scripts,
};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// workers/dedicated-workers corpus 扫描子目录（top-level + 一层子目录——语料两
/// 层深：`baseurl/alpha/`、`constructors/Worker/`、`interfaces/WorkerUtils/` 等，
/// 与 lib.sh fetch_dir_html depth<1 递归同布局；helper 目录 `resources/`
/// `support/` 在扫描内跳过）。`workers/modules`（module worker——goal 契约
/// 「启动时评估，不承诺」）、`workers/multi-globals`、`workers/non-automated`
/// （manual）、`workers/same-site-cookies`（cookie 域 + .headers 驱动，cookies
/// goal 面）不扫描，排除记账见 [`workers_case_skipped`] 与 goal evidence。
pub const WORKERS_TEST_SUBDIRS: &[&str] = &["workers", "dedicated-workers"];

/// [`WORKERS_TEST_SUBDIRS`] 用例排除规则（goal 契约排除面的 runner 侧镜像）。
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文（内容规则按标记筛减）。
///
/// - `*.https.*`（secure-context 前提 runner 无——fileapi/webmessaging 先例）
/// - `.html`/`.htm` 不含 `testharness`（同目录无 harness 支撑页——`examples/`
///   演示页、iframe 导航空白靶页；`.window.js`/`.any.js` 无此引用属正常形态，
///   规则只作用于 html/htm）
/// - 内容标记 `new SharedWorker(` 或引用 `SharedWorker-create-common.js`（SharedWorker
///   ——goal Mission 排除，挂账；后者是 SharedWorker 专用构造 helper，用例本体经
///   helper 构造，M2 补——SharedWorker-exception(-propagation).html 漏网修正）
/// - 内容标记 `iframe`（frame 树面——runner 无 frame 管道，navigation-compat M3
///   门槛后随其重入，webmessaging 同规则）
/// - 内容标记 `window.open`（第二文档面，同 iframe 门槛）
/// - 内容标记 `type: 'module'` / `type:"module"`（module worker——goal 契约
///   「classic 先行，module 评估记账」；`workers/modules/` 目录级不扫描，此处兜
///   底顶层零散 module 用例）
pub(crate) fn workers_case_skipped(relative: &str, source: &str) -> bool {
    if relative.contains(".https.") {
        return true;
    }
    if (relative.ends_with(".html") || relative.ends_with(".htm")) && !source.contains("testharness") {
        return true;
    }
    source.contains("new SharedWorker(")
        || source.contains("SharedWorker-create-common.js")
        || source.contains("iframe")
        || source.contains("window.open")
        || source.contains("type: 'module'")
        || source.contains("type:'module'")
        || source.contains("type: \"module\"")
        || source.contains("type:\"module\"")
}

/// `.any.js` 变体分道（`// META: global=` 解析——上游 tokens：`window`、`worker`
/// shorthand（dedicated+shared+service 全 worker 面）、`dedicatedworker`、
/// `sharedworker`、`serviceworker`）。goal 契约面：window 变体走 window 通道，
/// dedicated 变体走 dedicated worker 通道，SharedWorker/ServiceWorker 专属跳过。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AnyJsLane {
    Window,
    DedicatedWorker,
    Skip,
}

/// Parse `// META: global=` tokens（多行/逗号并集——上游 `global=` 单行多值 +
/// 多行同键均为合法形态）。前缀兼容 `//META:` 无空格形态（upstream serve.py
/// meta 解析按 `//\s*META:`——corpus 实例 Worker-constructor-proto.any.js）。
fn wpt_meta_globals(source: &str) -> Vec<String> {
    let mut globals = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        let meta_value = trimmed
            .strip_prefix("// META: ")
            .or_else(|| trimmed.strip_prefix("//META: "));
        let Some(meta_value) = meta_value else {
            continue;
        };
        let Some(value) = meta_value.strip_prefix("global=") else {
            continue;
        };
        for token in value.split(',') {
            let token = token.trim();
            if !token.is_empty() && !globals.iter().any(|existing| existing == token) {
                globals.push(token.to_string());
            }
        }
    }
    globals
}

/// [`wpt_meta_globals`] 分道判定（无 `global=` META 的 any.js 上游默认 window 变体
/// ——`AnyHtmlHandler` path_replace 无条件面，同 window 通道）。
pub(crate) fn workers_any_js_lane(source: &str) -> AnyJsLane {
    let globals = wpt_meta_globals(source);
    if globals.is_empty() {
        return AnyJsLane::Window;
    }
    if globals.iter().any(|global| global == "window") {
        return AnyJsLane::Window;
    }
    if globals
        .iter()
        .any(|global| global == "worker" || global == "dedicatedworker")
    {
        return AnyJsLane::DedicatedWorker;
    }
    AnyJsLane::Skip
}

/// `.any.js` dedicated 变体 wrapper 源合成（M2 worker harness 通道）——上游
/// serve.py ClassicWorkerHandler 同构：GLOBAL flags + importScripts testharness +
/// META script 依赖 + 语料正文 + done()。语料正文**内嵌**（非
/// `importScripts('/…case.any.js')` 自引用——该路径经 fetcher 会命中同款包装再包
/// 一层成环）。FIXME(inline-strict)：正文以 "use strict" 开头时内嵌于非指令位、
/// strict 语义失效（upstream importScripts 独立脚本指令位生效——corpus 边缘）。
///
/// 执行形态：页面 `new Worker('/…case.any.worker.js')` 外链（`.any.js` 源经
/// fetcher 映射；上游 AnyWorkerHandler 语义
/// ——worker 自真实合成 origin 服务，正文 importScripts 根绝对路径可解析；
/// 此前 data: URL wrapper 内联交付，data: base 对根绝对路径 URL 解析必抛 SyntaxError，
/// 相①整脚本中止 → dedicated 通道 importScripts 族全灭 `tests:0`）。
/// `done()` 显式收尾（DedicatedWorkerTestEnvironment `wait_for_finish = true`
/// ——worker 内 harness 无 load 事件，必须显式信号）。
pub(crate) fn synthesize_any_worker_source(relative: &str, source: &str) -> String {
    // META script 依赖（`wpt_meta_scripts` 相对形态归一为 wpt-data 根绝对）。
    let case_dir = Path::new(relative).parent().unwrap_or_else(|| Path::new(""));
    let mut imports = vec!["'/resources/testharness.js'".to_string()];
    for script in wpt_meta_scripts(relative, source) {
        let resolved = if let Some(root_relative) = script.strip_prefix('/') {
            root_relative.to_string()
        } else {
            normalize_relative(&format!("{}/{script}", case_dir.to_string_lossy()))
        };
        imports.push(format!("'/{}'", resolved));
    }
    let import_calls = imports
        .iter()
        .map(|import| format!("importScripts({import})"))
        .collect::<Vec<_>>()
        .join(";\n");
    format!(
        "self.GLOBAL = {{\n\
         \x20 isWindow: function() {{ return false; }},\n\
         \x20 isWorker: function() {{ return true; }},\n\
         \x20 isShadowRealm: function() {{ return false; }},\n\
         }};\n{import_calls};\n{source}\ndone();\n"
    )
}

/// `.worker.js` in-worker harness 形态执行（M2 重入）：原样作 worker 脚本外链
/// （上游 WorkersHandler 同构——语料文件自带 importScripts testharness + done()），
/// 页面 `fetch_tests_from_worker(new Worker(url))` 聚合（canvas 通道同款）。
fn run_worker_js_case(wpt_root: &Path, relative: &str, harness_source: &str) -> Vec<HarnessSubtestResult> {
    let page = format!(
        "<!doctype html><meta charset=\"utf-8\">\
         <script src=\"/resources/testharness.js\"></script>\
         <script>fetch_tests_from_worker(new Worker('/{relative}'));</script>"
    );
    run_testharness_html_inner(wpt_root, relative, &page, harness_source, &[], corpus_case_timeout())
}

/// `.any.js` dedicated 变体执行：页面 `new Worker('/…case.any.worker.js')`——
/// upstream wptserve dedicated 变体对外 URL 命名（examples/general.any.js 断言
/// self.location.pathname 带 `.any.worker.js`；fetcher 侧按 `.worker.js` 后缀
/// 映射回磁盘 `.any.js` 源）。上报案名仍为 `.any.js` 源路径（与 M1/M2 台账键
/// 一致，跨轮可比）。
fn run_any_js_dedicated_case(wpt_root: &Path, relative: &str, harness_source: &str) -> Vec<HarnessSubtestResult> {
    let worker_relative = relative.replacen(".any.js", ".any.worker.js", 1);
    let page = format!(
        "<!doctype html><meta charset=\"utf-8\">\
         <script src=\"/resources/testharness.js\"></script>\
         <script>fetch_tests_from_worker(new Worker('/{worker_relative}'));</script>"
    );
    run_testharness_html_inner(wpt_root, relative, &page, harness_source, &[], corpus_case_timeout())
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

/// Run the imported upstream workers window subset
/// （workers-compat goal M1 / DC-1）。filter 按路径子串过滤（如
/// `constructors/`、`semantics/`——基线按面分类）。
pub fn run_workers_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
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
    // 目录枚举：扫描根 + 全部子目录（语料最深三层：`workers/interfaces/WorkerGlobalScope/close/`
    // 等；fetch 侧 GOAL_FETCH_DEEP=1 全量树同布局）；helper 子目录（resources/support）
    // 逐层跳过。
    let mut scan_dirs = Vec::new();
    for subdir in WORKERS_TEST_SUBDIRS {
        collect_scan_dirs(wpt_root, Path::new(subdir), &mut scan_dirs);
    }
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
            // .window.js 为 window 变体形态（webmessaging/fileapi 通道先例——
            // any_js_window_wrapper 内联执行，META script 依赖自动解析）；.any.js 按
            // `// META: global=` 分道（M2——worker/dedicatedworker 变体走 dedicated
            // worker 通道）；.worker.js 为 in-worker harness 形态（M2 重入）。
            let is_window_js = file_name.ends_with(".window.js");
            let is_any_js = file_name.ends_with(".any.js");
            let is_worker_js = file_name.ends_with(".worker.js");
            if !is_html && !is_window_js && !is_any_js && !is_worker_js {
                continue;
            }
            let relative = format!("{subdir}/{file_name}");
            if filter.is_some_and(|filter| !relative.contains(filter)) {
                continue;
            }
            // lossy 解码（无效 UTF-8 → U+FFFD）——脚本源码浏览器按 UTF-8 带替换解码，
            // `semantics/encodings/004.worker.js`（原始无效字节用例）read_to_string 会
            // 失败静默丢案；lossy 后入集执行。DC-1 起走统一入口（BOM 语料按 BOM 解，
            // 缺省 UTF-8 lossy 与原 from_utf8_lossy 等价）。
            let Ok(source) = read_corpus_to_string(&path) else {
                continue;
            };
            if workers_case_skipped(&relative, &source) {
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
            } else if is_worker_js {
                run_worker_js_case(wpt_root, &relative, &harness_source)
            } else if is_any_js {
                match workers_any_js_lane(&source) {
                    // dedicated 变体同 `.worker.js` 外链通道——`.any.js` 的 worker 源
                    // 包装在 fetcher 侧（`synthesize_any_worker_source`，ClassicWorkerHandler
                    // 同构），worker URL 为 `.any.worker.js` 变体名（拿真合成 origin 作
                    // base + upstream location 命名面）。
                    AnyJsLane::DedicatedWorker => run_any_js_dedicated_case(wpt_root, &relative, &harness_source),
                    AnyJsLane::Window => run_any_js_window_case(wpt_root, &relative, &source, &harness_source),
                    // 仅 sharedworker / serviceworker 变体：goal 契约面之外（M1 排除表
                    // SharedWorker 面同域），不收集。
                    AnyJsLane::Skip => continue,
                }
            } else {
                run_any_js_window_case(wpt_root, &relative, &source, &harness_source)
            };
            cases.push((relative, results));
        }
    }
    cases.sort_by(|a, b| a.0.cmp(&b.0));
    cases
}
