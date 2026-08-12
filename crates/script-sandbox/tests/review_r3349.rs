//! R3349 deep-review 修复回归测试（zero-script-sandbox es_module.rs）。
//!
//! 本轮 deep-review 发现并修复的 `extract_dynamic_import_specifiers` bug 常驻断言：
//!
//! **动态 import() 标识符提取恒空（中危）**：`extract_dynamic_import_specifiers` 找到
//! `import(` 后，把紧跟的 `rest`（如 `'./x.js')`）直接喂入 `extract_string_literal`。
//! 该 helper 要求字符串以闭合引号结尾，但动态 import 在引号后有 `)`（及可选 `;`）→
//! `ends_with(close)` 为 false → 恒返回 Err → 标识符被丢弃。**所有动态 import 形式
//! 皆失效**：`import('./x.js')` / `import('./x.js');` / `await import('./mod.js')` /
//! `import( "./a.js" )` 全提取为 `[]`。
//!
//! **生产影响**：`extract_module_import_specifiers`（调用 dynamic 版）被 renderer
//! `js_worker.rs:564` 与 browser `tab_js_worker.rs:542` 用于**预取/预注册**模块依赖——
//! 动态 import 漏提致对应模块未预取/预注册，运行时 `__zw_load_module` 须延迟到 fetch
//! 兜底（若兜底缺失则动态 import 失败）。真转换正确性 bug。
//!
//! 修复：`extract_dynamic_import_specifiers` 在 `extract_string_literal` 前剥离尾随 `)`
//!（及 `;`），仅改 dynamic 路径，不动 `extract_string_literal`（静态 import 依赖其当前
//! 行为）。

#![cfg(any(feature = "v8", feature = "quickjs"))]

use zero_script_sandbox::{extract_dynamic_import_specifiers, extract_module_import_specifiers};

// ── Bug：动态 import() 须提取标识符，不得恒空 ─────────────────────────

#[test]
fn test_extract_dynamic_import_basic_r3349() {
    // `import('./x.js')` 最基本形式——修复前恒返 []（extract_string_literal 因尾随 ) 报 unclosed）。
    let specs = extract_dynamic_import_specifiers("import('./x.js')");
    assert_eq!(specs, vec!["./x.js".to_string()], "动态 import('./x.js') 须提取 ./x.js");
}

#[test]
fn test_extract_dynamic_import_with_semicolon_r3349() {
    let specs = extract_dynamic_import_specifiers("import('./x.js');");
    assert_eq!(specs, vec!["./x.js".to_string()]);
}

#[test]
fn test_extract_dynamic_import_await_r3349() {
    // await 动态 import（最常见真实用法）。
    let specs = extract_dynamic_import_specifiers("const m = await import('./mod.js')");
    assert_eq!(specs, vec!["./mod.js".to_string()]);
}

#[test]
fn test_extract_dynamic_import_double_quote_r3349() {
    let specs = extract_dynamic_import_specifiers("import(\"./a.js\")");
    assert_eq!(specs, vec!["./a.js".to_string()]);
}

#[test]
fn test_extract_dynamic_import_spaces_r3349() {
    // import( 与引号间有空格。
    let specs = extract_dynamic_import_specifiers("import( './b.js' )");
    assert_eq!(specs, vec!["./b.js".to_string()]);
}

#[test]
fn test_extract_dynamic_import_multiple_r3349() {
    let specs = extract_dynamic_import_specifiers("import('./a.js'); import('./b.js')");
    assert_eq!(specs, vec!["./a.js".to_string(), "./b.js".to_string()]);
}

#[test]
fn test_extract_dynamic_import_dedup_r3349() {
    let specs = extract_dynamic_import_specifiers("import('./a.js'); import('./a.js')");
    assert_eq!(specs, vec!["./a.js".to_string()]);
}

// ── 经 extract_module_import_specifiers 端到端（含静态+动态混合）──────

#[test]
fn test_extract_module_imports_dynamic_via_public_api_r3349() {
    // 静态 + 动态混合：两者都须被提取（修复前动态部分漏）。
    let src = "import { x } from './static.js'\nconst y = import('./dynamic.js')";
    let specs = extract_module_import_specifiers(src);
    assert!(
        specs.contains(&"./static.js".to_string()),
        "静态 import 须提取：{specs:?}"
    );
    assert!(
        specs.contains(&"./dynamic.js".to_string()),
        "动态 import 须提取（修复前漏）：{specs:?}"
    );
}

// ── 静态 import 回归保护（修复不得破坏既有静态提取）──────────────────

#[test]
fn test_extract_module_imports_static_unchanged_r3349() {
    let src = "import { x } from './a.js'\nimport y from './b.js'\nimport * as z from './c.js'";
    let specs = extract_module_import_specifiers(src);
    assert!(specs.contains(&"./a.js".to_string()));
    assert!(specs.contains(&"./b.js".to_string()));
    assert!(specs.contains(&"./c.js".to_string()));
}

#[test]
fn test_extract_dynamic_import_no_false_positive_r3349() {
    // 无 import() 不应误提；含 "import(" 子串的字符串字面量不属动态导入（保守，可接受）。
    let specs = extract_dynamic_import_specifiers("var x = 1");
    assert!(specs.is_empty());
}
