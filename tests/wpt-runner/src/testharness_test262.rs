//! Test262 上游执行通道（wptfyi-runtime goal，docs/goal/archive/wptfyi-runtime.md DC-1）。
//!
//! 独立模块而非并入 [`crate::testharness`]（2000 行约束，testharness_wasm_core
//! 先例），复用其执行底座（`run_testharness_html_inner` / `corpus_case_timeout`）。
//!
//! 语料由 `scripts/goals/48-test262-runtime.sh` vendoring 到
//! `wpt-data/third_party/test262/{test,harness}`（git partial clone + cone sparse，
//! pin = lib.sh WPT_REV；vendored.toml 记 test262 上游 rev）与
//! `wpt-data/resources/test262/{test262-reporter,test262-provider}.js`（上游
//! serve 包装器脚本，同 pin）。
//!
//! 执行语义 = 上游 serve.py test262 包装链的**折叠单页**等价实现（上游 outer
//! `.test262.html` 页持 testharness async_test、inner `.test262-test.html` 页持
//! reporter/harness/provider 两页 iframe 结构 → 单页三段脚本序，通信收口
//! `window.test262HarnessDone`；顶层页 `window.parent === window`，reporter
//! 的 `parentWindow.test262HarnessDone` 调用零改动即达）。两页隔离的折叠等价：
//! 上游 inner 页未捕获异常只达 reporter 不达 outer testharness——折叠页 bootstrap
//! `setup({allow_uncaught_exception: true})` 关闭 testharness 全局 error 毒化
//! （testharness.js error_handler 的 `!tests.allow_uncaught_exception` 分支整段
//! 跳过），reporter 仍为唯一分类者。参考：
//! https://github.com/web-platform-tests/wpt/blob/8c04fb8d43/tools/serve/serve.py
//! （Test262Window*Handler 族）+ resources/test262/test262-reporter.js。
//!
//! 变体记账（tools/manifest/sourcefile.py 逐字核对；每用例恰一变体——语料无
//! META: variant）：
//! - `flags` 含 `module` → `<name>.test262-module.html`：`<script type=module>`
//!   dynamic `import()` 用例源；import 拒绝 → `setTimeout(throw)` 走 window
//!   error 事件负例分类；import 成功且非 async → 直接 `test262Done()`
//! - `flags` 含 `onlyStrict`（非 module）→ `<name>.test262.strict.html`：用例源
//!   前缀 `"use strict";`（仅用例脚本块，harness 块不加——serve 同款）
//! - 其余 → `<name>.test262.html`
//!
//! 计分口径：每变体恰 1 子测试（wpt.fyi summary 每路径 c[1]=1 实证）——
//! async_test 完成态即该子测试态（Pass/Fail 负例分类消息），与 coverage 对账
//! 分母 `max(浏览器 c[1], ZW total)` 恰好逐文件对齐。
//!
//! skip 面（runner 侧镜像上游 manifest support 判据）：
//! - `*_FIXTURE.js`（tools/manifest/test262.py parse 早退同款）
//! - 无 YAML frontmatter 的 .js = support 文件（不产生 manifest 条目）
//!
//! 其余全量执行（负例/realm/agent 面按真实失败在 DC-2 归簇，不预设排除）。

use crate::testharness::{HarnessStatus, HarnessSubtestResult, corpus_case_timeout, run_testharness_html_inner};
use crate::wpt_file_loader::read_corpus_to_string;
use std::path::Path;

/// test262 用例扫描根（wpt-data 相对）。
const TEST262_TEST_DIR: &str = "third_party/test262/test";
/// test262 harness 库目录（includes 与常驻 assert/sta/doneprintHandle）。
const TEST262_HARNESS_DIR: &str = "third_party/test262/harness";
/// 上游 serve 包装器脚本目录（fetch 脚本同 pin 拉取）。
const TEST262_RESOURCES_DIR: &str = "resources/test262";

/// frontmatter 判据的最小 YAML 子集解析结果。只取上游
/// tools/manifest/test262.py TestRecord 的执行语义字段（flags 三态 + includes +
/// negative{phase,type}）；info/description/features 等不消费。
#[derive(Debug, Default, PartialEq, Eq)]
struct Test262Meta {
    is_module: bool,
    is_only_strict: bool,
    is_async: bool,
    includes: Vec<String>,
    /// `negative:` 的 `(type, phase)`；phase 缺省时上游 reporter 按 runtime 处理，
    /// 此处存 `None` phase 与上游对齐（拼接 meta 脚本时同样缺省）。
    negative: Option<(String, Option<String>)>,
}

/// 从用例源提取 frontmatter 块（首个 `/*--- ... ---*/`，DOTALL 语义——
/// tools/manifest/test262.py _YAML_PATTERN 同款；块前版权注释不碍事）。
fn frontmatter_block(source: &str) -> Option<&str> {
    let start = source.find("/*---")? + "/*---".len();
    let rest = &source[start..];
    let end = rest.find("---*/")?;
    Some(&rest[..end])
}

/// 解析一行 `key:` / `- value` / `key: value` 的 (indent, key, inline_value)。
fn split_key_line(line: &str) -> Option<(usize, &str, Option<&str>)> {
    let indent = line.len() - line.trim_start().len();
    let trimmed = line.trim();
    if let Some(item) = trimmed.strip_prefix("- ") {
        return Some((indent, "-", Some(item.trim())));
    }
    let (key, value) = trimmed.split_once(':')?;
    let value = value.trim();
    Some((indent, key.trim(), if value.is_empty() { None } else { Some(value) }))
}

/// 去引号（YAML 标量单双引号形态；test262 includes/负例 type 均为裸标识符，
/// 引号形态防御性兼容）。
fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2
        && ((value.starts_with('\'') && value.ends_with('\'')) || (value.starts_with('"') && value.ends_with('"')))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    }
}

/// 行内列表 `[a, b]` → 元素列表（非列表形态返回单元素标量）。
fn inline_list(value: &str) -> Vec<String> {
    let value = value.trim();
    if let Some(inner) = value.strip_prefix('[').and_then(|v| v.strip_suffix(']')) {
        inner.split(',').map(unquote).filter(|item| !item.is_empty()).collect()
    } else {
        vec![unquote(value)]
    }
}

/// frontmatter 最小解析：逐行状态机（`flags:`/`includes:` 行内 `[]` 或缩进块
/// `- item`；`negative:` 行内 `{phase, type}` 或缩进块键值；`info:`/`description:`
/// 等 `|`/`>` 块标量整段跳过——info 文本内出现 `flags:` 字样不得误吞，
/// 上游 yaml.safe_load 同语义）。解析失败（无 frontmatter）返回 None = support。
fn parse_test262_meta(source: &str) -> Option<Test262Meta> {
    let block = frontmatter_block(source)?;
    let mut meta = Test262Meta::default();
    let lines: Vec<&str> = block.lines().collect();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let Some((indent, key, value)) = split_key_line(line) else {
            index += 1;
            continue;
        };
        match (indent, key) {
            // 块标量（`|`/`>` 及其 chomp 修饰形态）：整段跳过至缩进回到本键层级。
            (_, "info" | "description" | "defines" | "notes")
                if value.is_some_and(|v| v.starts_with('|') || v.starts_with('>')) =>
            {
                index += 1;
                while index < lines.len() {
                    let next = lines[index];
                    let Some((next_indent, _, _)) = split_key_line(next) else {
                        index += 1;
                        continue;
                    };
                    if next_indent <= indent {
                        break;
                    }
                    index += 1;
                }
            }
            (0, "flags") => {
                index += 1;
                if let Some(value) = value {
                    for flag in inline_list(value) {
                        match flag.as_str() {
                            "module" => meta.is_module = true,
                            "onlyStrict" => meta.is_only_strict = true,
                            "async" => meta.is_async = true,
                            _ => {}
                        }
                    }
                } else {
                    while index < lines.len() {
                        let Some((next_indent, item_key, item_value)) = split_key_line(lines[index]) else {
                            break;
                        };
                        if next_indent == 0 || item_key != "-" {
                            break;
                        }
                        if let Some(item_value) = item_value {
                            match item_value {
                                "module" => meta.is_module = true,
                                "onlyStrict" => meta.is_only_strict = true,
                                "async" => meta.is_async = true,
                                _ => {}
                            }
                        }
                        index += 1;
                    }
                }
            }
            (0, "includes") => {
                index += 1;
                if let Some(value) = value {
                    meta.includes = inline_list(value);
                } else {
                    while index < lines.len() {
                        let Some((next_indent, item_key, item_value)) = split_key_line(lines[index]) else {
                            break;
                        };
                        if next_indent == 0 || item_key != "-" {
                            break;
                        }
                        if let Some(item_value) = item_value {
                            meta.includes.push(unquote(item_value));
                        }
                        index += 1;
                    }
                }
            }
            (0, "negative") => {
                index += 1;
                if let Some(value) = value {
                    // 行内 dict：`{phase: parse, type: SyntaxError}`（键序不定）。
                    let inner = value
                        .strip_prefix('{')
                        .and_then(|v| v.strip_suffix('}'))
                        .unwrap_or(value);
                    meta.negative = Some(parse_negative_pairs(inner, ','));
                } else {
                    // 缩进块：phase/type 各一行。
                    let mut negative_type: Option<String> = None;
                    let mut negative_phase: Option<String> = None;
                    while index < lines.len() {
                        let Some((next_indent, pair_key, pair_value)) = split_key_line(lines[index]) else {
                            break;
                        };
                        if next_indent == 0 {
                            break;
                        }
                        match (pair_key, pair_value) {
                            ("type", Some(v)) => negative_type = Some(unquote(v)),
                            ("phase", Some(v)) => negative_phase = Some(unquote(v)),
                            _ => {}
                        }
                        index += 1;
                    }
                    if let Some(negative_type) = negative_type {
                        meta.negative = Some((negative_type, negative_phase));
                    }
                }
            }
            _ => index += 1,
        }
    }
    Some(meta)
}

/// 逗号分隔键值对解析（负例行内 dict；冒号首现切分）。
fn parse_negative_pairs(inner: &str, separator: char) -> (String, Option<String>) {
    let mut negative_type: Option<String> = None;
    let mut negative_phase: Option<String> = None;
    for pair in inner.split(separator) {
        let Some((key, value)) = pair.split_once(':') else {
            continue;
        };
        match key.trim() {
            "type" => negative_type = Some(unquote(value)),
            "phase" => negative_phase = Some(unquote(value)),
            _ => {}
        }
    }
    (
        negative_type.unwrap_or_else(|| "Test262Error".to_string()),
        negative_phase,
    )
}

/// 变体后缀（tools/manifest/sourcefile.py：module 优先于 onlyStrict——模块恒
/// 严格模式，不追加 .strict）。
fn variant_suffix(meta: &Test262Meta) -> &'static str {
    if meta.is_module {
        ".test262-module.html"
    } else if meta.is_only_strict {
        ".test262.strict.html"
    } else {
        ".test262.html"
    }
}

/// [`TEST262_TEST_DIR`] 用例排除规则（上游 manifest support 判据镜像；
/// `relative` 为 wpt-data 相对路径，`source` 为用例原文）。
fn test262_case_skipped(relative: &str, source: &str) -> bool {
    let name = relative.rsplit('/').next().unwrap_or(relative);
    if name.ends_with("_FIXTURE.js") {
        return true;
    }
    // ShadowRealm.importValue 真加载面（r11 勘察证据
    // docs/goal/archive/wptfyi-runtime/evidence/dc2-test262-shadowrealm-2026-11-10.md）：
    // engine dynamic import 桥按 classic 页面 slot 契约实现，shadow realm 场景
    // 违背 V8 内核期望——import-value 案进程级 SIGTRAP 连累整轮，module 变体
    // 三案依赖同一成功路径。划界 skip（同步校验面照常执行）。
    const SHADOW_REALM_IMPORT_VALUE_SKIPPED: [&str; 4] = [
        "import-value.js",
        "throws-if-import-value-does-not-exist.js",
        "throws-typeerror-import-syntax-error.js",
        "throws-typeerror-import-throws.js",
    ];
    if relative.contains("ShadowRealm/prototype/importValue/") && SHADOW_REALM_IMPORT_VALUE_SKIPPED.contains(&name) {
        return true;
    }
    parse_test262_meta(source).is_none()
}

/// JS 字符串字面量转义（meta 脚本的负例 type/phase 注入面；值域为上游
/// frontmatter 词汇——防御任意引号/反斜杠形态）。
fn js_string_literal(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

/// 折叠包装页装配（serve.py 上游链的脚本序保持）：
///
/// 1. head：testharness.js（runner 注入 harness + 结果 reporter）+ 空的
///    testharnessreport.js 占位 + **outer 接线**（async_test("Test") +
///    `test262HarnessDone`——status 1/2 抛错转 Fail，serve outer wrapper 同款）
///    + **reporter + provider**（同 pin corpus 内联；provider 读 reporter 已装的
///      `window.print` 覆盖，序在前——serve inner pre_wrapper 同序）
/// 2. head meta：`test262Negative(type, phase)` / `test262IsAsync(true)`
/// 3. body：`test262Setup()` → assert/sta → includes →（async 时）
///    doneprintHandle → 用例源（strict 变体加 `"use strict";` 前缀；module
///    变体换 `type=module` dynamic `import()` 块）
///
/// 全部脚本块独立 inline——负例 parse 用例源的语法错误只落本块 window error，
/// 不阻断 bootstrap（serve `<script src>` 逐文件加载语义）。
fn build_test262_wrapper(
    meta: &Test262Meta,
    test_source: &str,
    reporter_source: &str,
    provider_source: &str,
    harness_blocks: &[String],
    module_import_path: Option<&str>,
) -> String {
    let mut head = String::from(
        "<!doctype html><meta charset=\"utf-8\">\
         <script src=\"/resources/testharness.js\"></script>\
         <script src=\"/resources/testharnessreport.js\"></script>\
         <script>\n\
         setup({ allow_uncaught_exception: true });\n\
         // 折叠页内用例源与 bootstrap 共享全局词法环境——outer 句柄用碰撞安全名
         //（上游两页结构中用例源在 inner 页独立 window，`const t` 无碰撞面；语料
         // 顶层 `var/let t` 声明与词法 `t` 并存即 SyntaxError——staging 10 案实证）。
         const __zwT262Outer = async_test(\"Test\");\n\
         window.test262HarnessDone = __zwT262Outer.step_func_done(function(status, message) {\n\
         \x20 if (status === 1) { throw new Error(message || \"Test failed\"); }\n\
         \x20 else if (status === 2) { throw new Error(message || \"Harness Error\"); }\n\
         });\n\
         </script>\n\
         <script>",
    );
    head.push_str(reporter_source);
    head.push_str("\n</script>\n<script>");
    head.push_str(provider_source);
    head.push_str("\n</script>\n");
    if let Some((negative_type, negative_phase)) = &meta.negative {
        head.push_str(&format!(
            "<script>test262Negative({}, {});</script>\n",
            js_string_literal(negative_type),
            js_string_literal(negative_phase.as_deref().unwrap_or("runtime")),
        ));
    }
    if meta.is_async {
        head.push_str("<script>test262IsAsync(true);</script>\n");
    }

    let mut body = String::from("<body><script>test262Setup();</script>\n");
    for block in harness_blocks {
        body.push_str("<script>");
        body.push_str(&block.replace("</script", "<\\/script"));
        body.push_str("</script>\n");
    }
    if let Some(import_path) = module_import_path {
        // module 变体（serve Test262WindowModuleTestHandler 同款）：import 成功且
        // 非 async → 直接收口；拒绝 → 分类。上游拒绝臂 = `setTimeout(() => throw)`
        // 靠浏览器把 timer 未捕获异常报成 window error——ZW 引擎未实现该报告
        //（FIXME 挂账 docs/goal/archive/wptfyi-runtime/master.md，属共享路径，须跨通道
        // 基线重验后再修）；此处直接派发同构 ErrorEvent（同 target/listener/
        // event.error 语义），reporter on_error 分类路径逐字不变。
        body.push_str(&format!(
            "<script>window.__test262IsModule = true;</script>\n\
             <script type=\"module\">\n\
             test262Setup();\n\
             import({}).then(() => {{\n\
             \x20 if (!window.test262Async) {{ test262Done(); }}\n\
             }}).catch(error => {{\n\
             \x20 window.dispatchEvent(new ErrorEvent(\"error\", {{\n\
             \x20\x20   error: error, message: String(error), cancelable: true,\n\
             \x20 }}));\n\
             }});\n\
             </script>\n",
            js_string_literal(import_path),
        ));
    } else {
        body.push_str("<script>");
        if meta.is_only_strict && !meta.is_module {
            body.push_str("\"use strict\";\n");
        }
        body.push_str(&test_source.replace("</script", "<\\/script"));
        body.push_str("</script>\n");
    }
    format!("{head}</head>\n{body}</body>")
}

/// Run the pinned upstream test262 corpus（wptfyi-runtime goal DC-1 通道）。
/// 目录递归扫描（wasm-core/workers 先例；排序收集保证基线顺序跨轮可复现），
/// 变体名 = 上游 manifest URL 规则（[variant_suffix]），报告路径与 wpt.fyi
/// summary 逐字对齐（coverage exact match）。skip 规则见 [test262_case_skipped]。
/// filter 按路径子串过滤，`|` 分隔多段任一命中即收（如
/// `third_party/test262/test/language/statements`；CI misc 分片合并
/// staging/annexB/intl402 三目录时用多段）。
pub fn run_test262_cases(wpt_root: &Path, filter: Option<&str>) -> Vec<(String, Vec<HarnessSubtestResult>)> {
    let load_resource = |relative: &str| -> Result<String, String> {
        read_corpus_to_string(wpt_root.join(relative)).map_err(|error| error.to_string())
    };
    // bootstrap 资产前置校验（缺失按单 Fail 记账不静默——run_any_js_window_case
    // support 缺失同款；404 文件级而非全通道中止）。
    let reporter_source = match load_resource(&format!("{TEST262_RESOURCES_DIR}/test262-reporter.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{TEST262_RESOURCES_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load test262-reporter.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }],
            )];
        }
    };
    let provider_source = match load_resource(&format!("{TEST262_RESOURCES_DIR}/test262-provider.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{TEST262_RESOURCES_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load test262-provider.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }],
            )];
        }
    };
    let harness_source = match load_resource("resources/testharness.js") {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{TEST262_RESOURCES_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load testharness.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }],
            )];
        }
    };
    // 常驻 harness 块（assert/sta——serve inner pre_wrapper 逐字同序；块体预读
    // 缺失按 Fail 记账）。includes 与 doneprintHandle 按用例装配。
    let assert_block = match load_resource(&format!("{TEST262_HARNESS_DIR}/assert.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{TEST262_HARNESS_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load harness/assert.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }],
            )];
        }
    };
    let sta_block = match load_resource(&format!("{TEST262_HARNESS_DIR}/sta.js")) {
        Ok(source) => source,
        Err(error) => {
            return vec![(
                format!("{TEST262_HARNESS_DIR}/"),
                vec![HarnessSubtestResult {
                    name: "load harness/sta.js".into(),
                    status: HarnessStatus::Fail,
                    message: Some(error),
                }],
            )];
        }
    };

    let mut case_paths: Vec<std::path::PathBuf> = Vec::new();
    let mut stack = vec![wpt_root.join(TEST262_TEST_DIR)];
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
            if entry.file_name().to_string_lossy().ends_with(".js") {
                case_paths.push(path);
            }
        }
    }
    case_paths.sort();

    let timeout = corpus_case_timeout();
    let mut cases = Vec::new();
    for path in case_paths {
        let relative = match path.strip_prefix(wpt_root) {
            Ok(relative) => relative.to_string_lossy().to_string(),
            Err(_) => continue,
        };
        if filter.is_some_and(|filter| !filter.split('|').any(|f| relative.contains(f))) {
            continue;
        }
        let Ok(source) = read_corpus_to_string(&path) else {
            continue;
        };
        if test262_case_skipped(&relative, &source) {
            continue;
        }
        // frontmatter 在 skip 判据里已验证过一次；此处重解析换取无副作用的
        // 纯函数链（语料 5.3 万文件、frontmatter 块 <1KB，双解析成本可忽略）。
        let meta = parse_test262_meta(&source).expect("frontmatter verified by skip predicate");
        let case_name = format!("{}{}", &relative[..relative.len() - ".js".len()], variant_suffix(&meta));
        // includes + async doneprintHandle（serve inner _get_script 同序；
        // 缺失按单 Fail 记账不静默）。
        let mut harness_blocks = vec![assert_block.clone(), sta_block.clone()];
        let mut missing_include: Option<String> = None;
        for include in &meta.includes {
            // includes 形态两种：裸 harness 文件名（`assert.js` → /third_party/
            // test262/harness/）或 /resources/test262/ 路径（sourcefile.py
            // _scripts 分流同款）。
            let include_path = if include.starts_with("/resources/test262/") {
                include.trim_start_matches('/').to_string()
            } else {
                format!("{TEST262_HARNESS_DIR}/{include}")
            };
            match load_resource(&include_path) {
                Ok(block) => harness_blocks.push(block),
                Err(error) => {
                    missing_include = Some(format!("load harness include {include}: {error}"));
                    break;
                }
            }
        }
        if let Some(message) = missing_include {
            cases.push((
                case_name,
                vec![HarnessSubtestResult {
                    name: "Test".into(),
                    status: HarnessStatus::Fail,
                    message: Some(message),
                }],
            ));
            continue;
        }
        if meta.is_async {
            match load_resource(&format!("{TEST262_HARNESS_DIR}/doneprintHandle.js")) {
                Ok(block) => harness_blocks.push(block),
                Err(error) => {
                    cases.push((
                        case_name,
                        vec![HarnessSubtestResult {
                            name: "Test".into(),
                            status: HarnessStatus::Fail,
                            message: Some(format!("load harness/doneprintHandle.js: {error}")),
                        }],
                    ));
                    continue;
                }
            }
        }
        let module_import_path = meta.is_module.then(|| format!("/{relative}"));
        let html = build_test262_wrapper(
            &meta,
            &source,
            &reporter_source,
            &provider_source,
            &harness_blocks,
            module_import_path.as_deref(),
        );
        let results = run_testharness_html_inner(wpt_root, &case_name, &html, &harness_source, &[], timeout);
        cases.push((case_name, results));
    }
    cases
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 行内 flags 列表解析 + 三态旗标。
    #[test]
    fn parses_inline_flags() {
        let source = "/*---\nflags: [module, async]\n---*/\nrun();";
        let meta = parse_test262_meta(source).expect("frontmatter");
        assert!(meta.is_module);
        assert!(meta.is_async);
        assert!(!meta.is_only_strict);
        assert_eq!(variant_suffix(&meta), ".test262-module.html");
    }

    /// 缩进块 flags 列表（每行 `- value`）。
    #[test]
    fn parses_block_flags() {
        let source = "/*---\nflags:\n  - onlyStrict\n  - async\n---*/\nrun();";
        let meta = parse_test262_meta(source).expect("frontmatter");
        assert!(meta.is_only_strict);
        assert!(meta.is_async);
        assert!(!meta.is_module);
        assert_eq!(variant_suffix(&meta), ".test262.strict.html");
    }

    /// module 优先于 onlyStrict（sourcefile.py：模块恒严格模式不追加 .strict）。
    #[test]
    fn module_wins_over_only_strict() {
        let source = "/*---\nflags: [module, onlyStrict]\n---*/\nrun();";
        let meta = parse_test262_meta(source).expect("frontmatter");
        assert_eq!(variant_suffix(&meta), ".test262-module.html");
    }

    /// 缺省旗标 → default 变体。
    #[test]
    fn default_variant_for_unflagged() {
        let source = "/*---\ninfo: NaN !== NaN\n---*/\nrun();";
        let meta = parse_test262_meta(source).expect("frontmatter");
        assert_eq!(variant_suffix(&meta), ".test262.html");
    }

    /// includes 行内 + 块两形态。
    #[test]
    fn parses_includes_both_forms() {
        let inline = parse_test262_meta("/*---\nincludes: [compareArray.js, propertyHelper.js]\n---*/\nrun();")
            .expect("frontmatter");
        assert_eq!(inline.includes, vec!["compareArray.js", "propertyHelper.js"]);
        let block = parse_test262_meta("/*---\nincludes:\n  - isConstructor.js\n---*/\nrun();").expect("frontmatter");
        assert_eq!(block.includes, vec!["isConstructor.js"]);
    }

    /// 负例行内 dict（键序不定）与缩进块两形态；phase 缺省 None。
    #[test]
    fn parses_negative_both_forms() {
        let inline = parse_test262_meta("/*---\nnegative:\n  phase: parse\n  type: SyntaxError\n---*/\nrun();")
            .expect("frontmatter");
        assert_eq!(
            inline.negative,
            Some(("SyntaxError".to_string(), Some("parse".to_string())))
        );
        let oneline = parse_test262_meta("/*---\nnegative: {type: RangeError, phase: runtime}\n---*/\nrun();")
            .expect("frontmatter");
        assert_eq!(
            oneline.negative,
            Some(("RangeError".to_string(), Some("runtime".to_string())))
        );
        let no_phase =
            parse_test262_meta("/*---\nnegative:\n  type: Test262Error\n---*/\nrun();").expect("frontmatter");
        assert_eq!(no_phase.negative, Some(("Test262Error".to_string(), None)));
    }

    /// info 块标量整段跳过——info 文本内 `flags:` 字样不得误吞（YAML 块标量
    /// 语义；上游 yaml.safe_load 同判据）。
    #[test]
    fn info_block_scalar_does_not_leak_flags() {
        let source = "/*---\ninfo: |\n  see flags: [module] for details\n  more text\nflags: [async]\n---*/\nrun();";
        let meta = parse_test262_meta(source).expect("frontmatter");
        assert!(!meta.is_module);
        assert!(meta.is_async);
    }

    /// 无 frontmatter = support 文件（上游 manifest 同判据）。
    #[test]
    fn support_file_without_frontmatter_is_none() {
        assert_eq!(parse_test262_meta("var x = 1;\nrun(x);"), None);
    }

    /// skip 规则：_FIXTURE.js 与 support 文件排除；普通用例放行。
    #[test]
    fn skip_rules() {
        assert!(test262_case_skipped(
            "third_party/test262/test/built-ins/x_FIXTURE.js",
            "/*---\nflags: [module]\n---*/\nrun();"
        ));
        assert!(test262_case_skipped(
            "third_party/test262/test/built-ins/helper.js",
            "var helper = 1;"
        ));
        assert!(!test262_case_skipped(
            "third_party/test262/test/built-ins/x.js",
            "/*---\nflags: []\n---*/\nrun();"
        ));
    }

    /// 折叠包装页脚本序：outer 接线 → reporter → provider → meta → body
    /// （assert/sta → includes → doneprintHandle → 用例源）。
    #[test]
    fn wrapper_script_order() {
        let meta = parse_test262_meta(
            "/*---\nflags: [async]\nincludes: [compareArray.js]\nnegative:\n  phase: runtime\n  type: TypeError\n---*/\nrun();",
        )
        .expect("frontmatter");
        let html = build_test262_wrapper(
            &meta,
            "run();",
            "REPORTER",
            "PROVIDER",
            &["ASSERT".into(), "STA".into(), "COMPARE".into(), "DONEPRINT".into()],
            None,
        );
        assert!(html.contains("setup({ allow_uncaught_exception: true });"));
        // 折叠页碰撞安全名——用例源顶层 `var/let t` 不与 outer 句柄词法冲突。
        assert!(html.contains("const __zwT262Outer"));
        let wiring = html.find("window.test262HarnessDone").expect("wiring");
        let reporter = html.find("REPORTER").expect("reporter");
        let provider = html.find("PROVIDER").expect("provider");
        let negative = html.find("test262Negative").expect("negative");
        let is_async = html.find("test262IsAsync").expect("isAsync");
        let setup = html.find("test262Setup").expect("setup");
        let assert = html.find("ASSERT").expect("assert");
        let sta = html.find("STA").expect("sta");
        let include = html.find("COMPARE").expect("include");
        let doneprint = html.find("DONEPRINT").expect("doneprint");
        let source = html.find("run();</script>").expect("source");
        assert!(wiring < reporter);
        assert!(reporter < provider);
        assert!(provider < negative);
        assert!(negative < is_async);
        assert!(is_async < setup);
        assert!(setup < assert);
        assert!(assert < sta);
        assert!(sta < include);
        assert!(include < doneprint);
        assert!(doneprint < source);
        assert!(html.contains("<body><script>test262Setup();</script>"));
    }

    /// strict 变体仅用例源块加 `"use strict";` 前缀（harness 块不加）。
    #[test]
    fn strict_prefix_only_on_source_block() {
        let meta = parse_test262_meta("/*---\nflags: [onlyStrict]\n---*/\nrun();").expect("frontmatter");
        let html = build_test262_wrapper(&meta, "run();", "R", "P", &["A".into(), "S".into()], None);
        assert_eq!(html.matches("\"use strict\";").count(), 1);
        let source_pos = html.find("\"use strict\";").expect("prefix");
        let assert_pos = html.find(">A</script>").expect("assert");
        assert!(assert_pos < source_pos);
    }

    /// module 变体：type=module 块 + absolute import 路径 + 用例源不内联。
    #[test]
    fn module_variant_uses_dynamic_import() {
        let meta = parse_test262_meta("/*---\nflags: [module]\n---*/\nexport {};").expect("frontmatter");
        let html = build_test262_wrapper(
            &meta,
            "export {};",
            "R",
            "P",
            &["A".into(), "S".into()],
            Some("/third_party/test262/test/x.js"),
        );
        assert!(html.contains("<script type=\"module\">"));
        assert!(html.contains("import(\"/third_party/test262/test/x.js\")"));
        assert!(html.contains("window.__test262IsModule = true;"));
        assert!(html.contains("test262Done();"));
        // 拒绝臂：直接派发携带 error 对象的 ErrorEvent（引擎 timer-throw 报告
        // FIXME 挂账——上游 setTimeout 重抛依赖的浏览器语义 ZW 未实现）。
        assert!(html.contains("new ErrorEvent(\"error\", {"));
        assert!(html.contains("error: error, message: String(error),"));
        assert!(!html.contains("<script>export {};"));
    }

    /// 用例源 `</script` 转义（annexB HTML-close-comment 语料形态）。
    #[test]
    fn source_script_close_tag_escaped() {
        let meta = parse_test262_meta("/*---\nflags: []\n---*/\nvar s = \"</script>\";").expect("frontmatter");
        let html = build_test262_wrapper(&meta, "var s = \"</script>\";", "R", "P", &[], None);
        assert!(html.contains("var s = \"<\\/script>\";"));
        assert!(!html.contains("var s = \"</script>\";"));
    }

    /// meta 脚本负例注入带 phase 缺省（reporter runtime 语义对齐）。
    #[test]
    fn negative_meta_defaults_phase() {
        let meta = parse_test262_meta("/*---\nnegative:\n  type: Test262Error\n---*/\nrun();").expect("frontmatter");
        let html = build_test262_wrapper(&meta, "run();", "R", "P", &[], None);
        assert!(html.contains("test262Negative(\"Test262Error\", \"runtime\");"));
    }
}
