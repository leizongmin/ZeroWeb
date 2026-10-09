//! ES Module 运行时 — 支持编译和执行 ES Module 格式的 JavaScript。
//!
//! 提供基本的 ES Module 支持：
//! - 源代码转换方式支持 `export`/`import` 语法
//! - 模块注册表管理已注册的模块
//! - `import.meta.url` 支持
//!
//! # 工作原理
//!
//! 将 ES Module 源代码转换为普通脚本：
//! - 依赖模块的内联转换代码直接嵌入导入模块
//! - `export` 声明转为 `_exports` 对象属性赋值
//! - `import` 声明转为对内联依赖模块导出对象的引用

#[cfg(any(feature = "v8", feature = "quickjs"))]
use crate::SandboxConfig;
use crate::ScriptError;
use std::collections::{HashMap, HashSet};

use crate::import_map::{ImportMap, ImportMapResolution};

/// 模块注册表 — 存储已注册的 ES Module 源代码。
#[derive(Debug, Clone, Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, String>,
    /// 页面 import map（HTML `<script type="importmap">` 解析产物）。
    /// `None` = 未设置，行为与无 import map 时完全一致。
    import_map: Option<ImportMap>,
}

impl ModuleRegistry {
    /// 创建空的模块注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册一个模块。
    pub fn register(&mut self, specifier: &str, source: &str) {
        self.modules.insert(specifier.to_string(), source.to_string());
    }

    /// 查询模块源代码。
    pub fn get(&self, specifier: &str) -> Option<&str> {
        self.modules.get(specifier).map(|s| s.as_str())
    }

    /// 移除一个已注册的模块。
    pub fn unregister(&mut self, specifier: &str) -> bool {
        self.modules.remove(specifier).is_some()
    }

    /// 获取已注册模块数量。
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// 注册表是否为空。
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// 列出所有已注册模块的标识符。
    pub fn specifiers(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }

    /// 设置页面 import map（HTML `<script type="importmap">` 解析产物）。
    pub fn set_import_map(&mut self, map: ImportMap) {
        self.import_map = Some(map);
    }

    /// 读取当前 import map（未设置时返回 `None`）。
    pub fn import_map(&self) -> Option<&ImportMap> {
        self.import_map.as_ref()
    }
}

/// ES Module 执行结果。
#[derive(Debug, Clone)]
pub struct ModuleResult {
    /// 模块命名空间对象的 JSON 字符串表示。
    pub namespace_json: String,
    /// 执行耗时（毫秒）。
    pub execution_time_ms: f64,
}

/// ES Module 沙箱 — 支持 `export`/`import` 语法的 JavaScript 执行环境。
///
/// 通过源代码转换将 ES Module 语法的代码在 V8 中执行。
/// 依赖模块以 IIFE 形式内联，导出通过共享的 `_exports` 对象传递。
#[cfg(any(feature = "v8", feature = "quickjs"))]
pub struct EsModuleSandbox {
    /// 模块注册表。
    registry: ModuleRegistry,
    /// V8 沙箱（用于执行转换后的代码）。
    sandbox: Box<dyn crate::Sandbox>,
}

#[cfg(any(feature = "v8", feature = "quickjs"))]
impl EsModuleSandbox {
    /// 创建新的 ES Module 沙箱。
    pub fn new() -> Result<Self, ScriptError> {
        // js-dom R84：v8+quickjs 组合态（workspace feature 并集）双分支都编译 → 变量重复
        // 绑定 + move 冲突。quickjs 分支 not(v8) 门控（v8 优先，与 lib.rs re-export 门控
        // 一致）；单 feature 语义不变。
        #[cfg(feature = "v8")]
        let sandbox: Box<dyn crate::Sandbox> = Box::new(crate::V8Sandbox::new()?);
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let sandbox: Box<dyn crate::Sandbox> = Box::new(crate::QuickJSSandbox::new()?);

        Ok(Self {
            registry: ModuleRegistry::new(),
            sandbox,
        })
    }

    /// 使用自定义配置创建 ES Module 沙箱。
    pub fn with_config(config: SandboxConfig) -> Result<Self, ScriptError> {
        #[cfg(feature = "v8")]
        let sandbox: Box<dyn crate::Sandbox> = Box::new(crate::V8Sandbox::with_config(config)?);
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let sandbox: Box<dyn crate::Sandbox> = Box::new(crate::QuickJSSandbox::with_config(config)?);

        Ok(Self {
            registry: ModuleRegistry::new(),
            sandbox,
        })
    }

    /// 获取模块注册表（可变引用）。
    pub fn registry_mut(&mut self) -> &mut ModuleRegistry {
        &mut self.registry
    }

    /// 获取模块注册表（只读引用）。
    pub fn registry(&self) -> &ModuleRegistry {
        &self.registry
    }

    /// 注册一个模块到注册表。
    pub fn register_module(&mut self, specifier: &str, source: &str) {
        self.registry.register(specifier, source);
    }

    /// 编译并执行 ES Module 代码。
    pub fn execute_module(&mut self, source: &str, url: Option<&str>) -> Result<ModuleResult, ScriptError> {
        if source.trim().is_empty() {
            return Err(ScriptError::InvalidInput("module source is empty".into()));
        }

        let start = std::time::Instant::now();
        let url = url.unwrap_or("zero://module");

        let transformed = compile_module_script(source, url, &self.registry)?;

        // 执行转换后的脚本；namespace_json 需要 JSON 序列化模块命名空间对象
        // （plain execute() 对对象返回 "[object Object]"，须用 execute_json()）
        let result = self.sandbox.execute_json(&transformed)?;

        let execution_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(ModuleResult {
            namespace_json: result.value,
            execution_time_ms,
        })
    }
}

#[cfg(any(feature = "v8", feature = "quickjs"))]
impl std::fmt::Debug for EsModuleSandbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EsModuleSandbox")
            .field("registry", &self.registry)
            .finish()
    }
}

// ── 核心转换逻辑（纯函数） ──

/// 将 ES Module 源码编译为可在 V8 中执行的 IIFE 脚本（内联依赖）。
pub fn compile_module_script(source: &str, url: &str, registry: &ModuleRegistry) -> Result<String, ScriptError> {
    let uses_await = source.contains("await");
    let body = build_module_script(source, url, registry, uses_await, &mut HashSet::new())?;
    if source.contains("import(") || uses_await {
        Ok(format!("(async function() {{\n{body}\n}})();\n"))
    } else {
        Ok(body)
    }
}

/// 编译依赖模块为可求值的 IIFE 表达式（返回 exports 对象）。
pub fn compile_dependency_iife(specifier: &str, registry: &ModuleRegistry) -> Result<String, ScriptError> {
    build_dep_iife(specifier, registry, &mut HashSet::new())
}

/// 生成模块运行时 prelude（`__moduleCache` + 动态 `import()` 支持）。
pub fn build_module_runtime_prelude(registry: &ModuleRegistry) -> Result<String, ScriptError> {
    let mut out = String::from("var __moduleCache = {};\n");
    // t8j：helper（__zw_load_module / __zw_dynamic_import）必须先于 `__moduleCache[spec] = IIFE`
    // 注册求值——注册右侧的 IIFE **立即执行模块体**（依赖预热 + collect_module_deps 把入口模块
    // 自身也注册进 registry，见 js_worker/tab_js_worker collect_module_deps）。模块体顶层的
    // `import()`（Vite 现代浏览器探测 `import("_").catch(()=>1)`）在求值瞬间需要 helper 已定义；
    // 反序时入口模块体在 prelude 第 2 条语句执行 → `__zw_dynamic_import is not defined` ReferenceError，
    // 探测中断、后续语句永不执行 → 站点误回落 legacy 加载路径。
    out.push_str("globalThis.__zw_load_module = function(spec, parentHint) {\n");
    // t8j：parent 优先取调用点穿入的 referrer（rewrite_dynamic_imports 的 `.call({referrer})`），
    // 回退全局 _importMeta（直接调用形态）——模块 IIFE 局部的 _importMeta 在此不可见。
    out.push_str(
        "  var parent = parentHint || ((typeof _importMeta !== 'undefined' && _importMeta.url) || 'about:blank');\n",
    );
    // t8j-r2（D1）：缓存键必须是 referrer×spec 的解析结果（ECMA-262 §sec-hostresolveimportedmodule）。
    // 旧实现以原始 specifier 为键——两个不同目录的模块各自 `import('./config.js')` 时后者命中
    // 前者条目，静默拿到错误模块。宿主（renderer/tab/webview）已按 parent 解析出绝对 URL，
    // 回传改为 `resolved\u{1f}code`：缓存键取宿主解析键，跨目录同名相对 spec 不再碰撞；
    // 旧契约（无分隔符的裸 code）回退 spec 键（base 语义）。renderer/tab 宿主有 runtime_iifes
    // 编译缓存，重复调用不重复取回；webview 进程内路径无宿主缓存（重复动态 import 同模块
    // 重复取回+编译，副作用不重复——转池候选）。
    out.push_str("  var r = __zw_compile_module(spec, parent);\n");
    out.push_str("  if (!r) throw new Error('Module not found: ' + spec);\n");
    out.push_str("  var sep = r.indexOf('\\u001f');\n");
    out.push_str("  var resolved = sep >= 0 ? r.slice(0, sep) : spec;\n");
    out.push_str("  if (__moduleCache[resolved]) return __moduleCache[resolved];\n");
    out.push_str("  var code = sep >= 0 ? r.slice(sep + 1) : r;\n");
    out.push_str("  __moduleCache[resolved] = (function() { return eval('(' + code + ')'); })();\n");
    out.push_str("  return __moduleCache[resolved];\n");
    out.push_str("};\n");
    // t8j：动态 import() 失败须以 rejected promise 呈现，不得同步 throw——
    // https://tc39.es/ecma262/#sec-import-calls（ImportCall 恒返回 promise；取回/编译失败
    // 走 promise rejection）。同步 throw 发生在调用方 `.catch()` 挂上之前（`import(x).catch(…)`
    // 表达式求值即抛），Vite 现代浏览器探测模块 `import("_").catch(()=>1)` 由此整体中断、
    // 后续语句（`__vite_is_modern_browser=true`）永不执行 → 站点误回落 legacy 加载路径。
    out.push_str(
        "globalThis.__zw_dynamic_import = function(spec) {\n  var parent = (this && this.referrer) || ((typeof _importMeta !== 'undefined' && _importMeta.url) || 'about:blank');\n  try {\n    return Promise.resolve(__zw_load_module(spec, parent));\n  } catch (e) {\n    return Promise.reject(e);\n  }\n};\n",
    );
    for spec in registry.specifiers() {
        let iife = build_dep_iife(spec, registry, &mut HashSet::new())?;
        let escaped = spec.replace('\\', "\\\\").replace('\'', "\\'");
        out.push_str(&format!("__moduleCache['{escaped}'] = {iife};\n"));
    }
    Ok(out)
}

pub(crate) fn rewrite_dynamic_imports(source: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < source.len() {
        if source[i..].starts_with("import(") {
            // t8j：经 `.call({referrer})` 把调用点 referrer 穿给 helper——重写点在模块 IIFE
            // 内（build_module_script / build_dep_iife 均定义 `var _importMeta`；service_worker
            // 经典脚本路径在宿主侧设置 `globalThis._importMeta`），而 `__zw_dynamic_import`
            // 本体在全局作用域执行，读不到 IIFE 局部的 `_importMeta`（旧实现恒回退
            // 'about:blank' → 相对 spec 解析失败、按原始 specifier 发起网络请求）。
            // ECMA-262 §sec-module-specifiers：动态 import 的 specifier 相对
            // **active script/module**（即调用点 referrer）解析。
            // t8j-r2（D2）：发射串不得含任何引号字符——子串扫描不识别字符串字面量上下文，
            // 发射进宿主单引号字符串（如 `throw new Error('import() failed')`）时引号提前
            // 终止字符串 → 整脚本 SyntaxError（base 发射串无引号仅值污染）。发射体只引用
            // 调用点词法作用域的 `_importMeta`（全部重写目标已保证定义），零引号。
            out.push_str("__zw_dynamic_import.call({ referrer: _importMeta.url }, ");
            i += "import(".len();
        } else {
            let ch = source[i..].chars().next().expect("char");
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// 从模块源码中提取全部 `import` 依赖标识符（静态 `import` + 动态 `import()`）。
pub fn extract_module_import_specifiers(source: &str) -> Vec<String> {
    let mut specs = extract_static_module_import_specifiers(source);
    push_unique_specs(&mut specs, extract_dynamic_import_specifiers(source));
    specs
}

// https://tc39.es/ecma262/#sec-imports
// https://tc39.es/ecma262/#sec-exports
// 识别 import 语句并剥离关键字，返回子句。识别 `import `（关键字后空白）与压缩形态
// `import{`、`import"`、`import'`、`` import` ``、`import*`——minifier 会剥掉关键字后
// 无语法歧义的空白。动态 `import(`、元属性 `import.`、`import` 前缀标识符不匹配。
fn strip_import_keyword(stmt: &str) -> Option<&str> {
    let rest = stmt.strip_prefix("import")?;
    match rest.chars().next()? {
        c if c.is_whitespace() => Some(rest.trim_start()),
        '{' | '"' | '\'' | '`' | '*' => Some(rest),
        _ => None,
    }
}

// https://tc39.es/ecma262/#sec-exports
// 识别 export 语句并剥离关键字。识别 `export ` 与压缩形态 `export{`、`export*`。
fn strip_export_keyword(stmt: &str) -> Option<&str> {
    let rest = stmt.strip_prefix("export")?;
    match rest.chars().next()? {
        c if c.is_whitespace() => Some(rest.trim_start()),
        '{' | '*' => Some(rest),
        _ => None,
    }
}

/// 在 import/export 子句中定位 `from` 关键字，返回（from 之前的绑定子句、from 后的
/// 模块标识符串起点）。支持常规 ` from '…'` 与压缩形态 `from'…'`：`from` 须为独立
/// token（前一字符非标识符组成、其后到字符串字面量间只允许空白）。
// https://tc39.es/ecma262/#sec-module-specifiers
fn split_from_clause(clause: &str) -> Option<(&str, &str)> {
    let bytes = clause.as_bytes();
    let is_ident_byte = |b: u8| b.is_ascii_alphanumeric() || b == b'_' || b == b'$';
    let mut search = 0;
    while let Some(rel) = clause[search..].find("from") {
        let at = search + rel;
        let before_ok = at == 0 || !is_ident_byte(bytes[at - 1]);
        let after = clause[at + 4..].trim_start();
        let after_ok = after.starts_with('"') || after.starts_with('\'') || after.starts_with('`');
        if before_ok && after_ok {
            // before 为绑定/名字子句，调用方按已去空白使用（{ a } 尾空格会让
            // trim_end_matches('}') 失效，切出 "a }" 这类脏绑定名）
            return Some((clause[..at].trim(), after));
        }
        search = at + 4;
    }
    None
}

/// 仅提取**静态** `import` 依赖标识符（不含 `import()` 动态导入）。
/// 供动态 import() 运行时 fetch 路径（R3093）：预注册空存根只用静态 import（headless 单遍，transitive defer），
/// 动态 import() 留给运行时 `__zw_load_module → __zw_compile_module` fetch——避免预存根（empty namespace）
/// 短路运行时 fetch。无 fetcher 路径仍用 `extract_module_import_specifiers`（动态 import 预存根返空 namespace）。
pub fn extract_static_module_import_specifiers(source: &str) -> Vec<String> {
    let mut specs = Vec::new();
    for stmt in split_statements(source) {
        let trimmed = stmt.trim();
        let specifier = if let Some(clause) = strip_import_keyword(trimmed) {
            extract_import_specifier(clause)
        } else if let Some(clause) = strip_export_keyword(trimmed) {
            extract_reexport_specifier(clause)
        } else {
            continue;
        };
        if let Ok(spec) = specifier {
            push_unique_spec(&mut specs, spec);
        }
    }
    specs
}

/// 从模块源码中提取 `import('...')` 动态依赖标识符。
pub fn extract_dynamic_import_specifiers(source: &str) -> Vec<String> {
    let mut specs = Vec::new();
    let mut search_from = 0;
    while let Some(rel) = source[search_from..].find("import(") {
        let start = search_from + rel + "import(".len();
        let rest = source[start..].trim_start();
        // extract_string_literal（R3349 已修为「在匹配闭合引号处停止，忽略其后的 `)`/`;`/剩余代码」）
        // 提取引号内的模块标识符。
        if let Ok(spec) = extract_string_literal(rest) {
            push_unique_spec(&mut specs, spec);
        }
        search_from = start;
    }
    specs
}

fn push_unique_spec(specs: &mut Vec<String>, spec: String) {
    if !specs.contains(&spec) {
        specs.push(spec);
    }
}

fn push_unique_specs(specs: &mut Vec<String>, more: Vec<String>) {
    for s in more {
        push_unique_spec(specs, s);
    }
}

/// 从 import 语句子句（关键字已剥离）提取模块标识符。
fn extract_import_specifier(clause: &str) -> Result<String, ScriptError> {
    if clause.starts_with('\'') || clause.starts_with('"') || clause.starts_with('`') {
        return extract_string_literal(clause.split(';').next().unwrap_or(clause).trim());
    }
    if let Some((_bindings, spec_part)) = split_from_clause(clause) {
        return extract_import_specifier_from_rest(spec_part);
    }
    Err(ScriptError::CompileError(format!("unsupported import: {clause}")))
}

/// 从 export 再导出语句子句（关键字已剥离）提取模块标识符。
fn extract_reexport_specifier(clause: &str) -> Result<String, ScriptError> {
    if let Some((_bindings, spec_part)) = split_from_clause(clause) {
        return extract_import_specifier_from_rest(spec_part);
    }
    Err(ScriptError::CompileError(format!("unsupported re-export: {clause}")))
}

/// 构建完整的模块执行脚本，内联所有依赖。
fn build_module_script(
    source: &str,
    url: &str,
    registry: &ModuleRegistry,
    async_body: bool,
    visited: &mut HashSet<String>,
) -> Result<String, ScriptError> {
    let mut output = String::with_capacity(source.len() * 3);
    if async_body {
        output.push_str("(async function() {\n");
    } else {
        output.push_str("(function() {\n");
    }
    output.push_str("  'use strict';\n");
    output.push_str("  var _exports = {};\n");
    output.push_str(&format!("  var _importMeta = {{ url: {} }};\n", json_stringify(url)));
    // 入口自导入（github.com environment 入口真实形态 `import*as i from"./environment-*.js"`
    // 后 `e.C(i)`，2026-10-08 home-reload 线上证据）：入口自身作为依赖重入时绑定到自身
    // _exports 活对象——不得再内联一份入口体（内层拷贝执行期 typeof 守卫落空 → 空 stub →
    // webpack C(ns) 读 `__rspack_esm_ids.length` 抛 TypeError）。
    output.push_str(&format!("  var _mod_{} = _exports;\n", safe_ident(url)));
    visited.insert(url.to_string());

    // 处理当前模块的每个语句
    let stmts = split_statements(source);
    for stmt in &stmts {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(clause) = strip_import_keyword(trimmed) {
            output.push_str(&transform_import(clause, url, registry, visited)?);
        } else if let Some(clause) = strip_export_keyword(trimmed) {
            output.push_str(&transform_export(clause, url, registry, visited)?);
        } else {
            // 普通语句：替换 import.meta 与动态 import()
            let s = rewrite_dynamic_imports(&trimmed.replace("import.meta", "_importMeta"));
            output.push_str("  ");
            output.push_str(&s);
            output.push_str(";\n");
        }
    }

    output.push_str("  return _exports;\n");
    output.push_str("})()\n");
    Ok(output)
}

/// 为依赖模块构建内联的 IIFE（返回其导出对象）。
fn build_dep_iife(
    specifier: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> Result<String, ScriptError> {
    let source = registry
        .get(specifier)
        .ok_or_else(|| ScriptError::RuntimeError(format!("Module not found: {specifier}")))?;

    let mut output = String::new();
    output.push_str("(function() {\n");
    output.push_str("  'use strict';\n");
    output.push_str("  var _exports = {};\n");
    output.push_str(&format!(
        "  var _importMeta = {{ url: {} }};\n",
        json_stringify(specifier)
    ));
    // 依赖模块自导入与入口自导入同型（见 build_module_script）：绑定到自身 _exports 活对象。
    output.push_str(&format!("  var _mod_{} = _exports;\n", safe_ident(specifier)));
    visited.insert(specifier.to_string());

    let stmts = split_statements(source);
    for stmt in &stmts {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(clause) = strip_import_keyword(trimmed) {
            output.push_str(&transform_import(clause, specifier, registry, visited)?);
        } else if let Some(clause) = strip_export_keyword(trimmed) {
            output.push_str(&transform_export(clause, specifier, registry, visited)?);
        } else {
            let s = rewrite_dynamic_imports(&trimmed.replace("import.meta", "_importMeta"));
            output.push_str("  ");
            output.push_str(&s);
            output.push_str(";\n");
        }
    }

    output.push_str("  return _exports;\n");
    output.push_str("})()");
    Ok(output)
}

/// 内联依赖模块 IIFE，**首次访问**时递归转换，**已访问**（循环 / 菱形 import）时返空对象占位
/// `(function(){return {};})()` 而非递归——防循环 import（a↔b）无限递归致栈溢出 abort（R3398
/// 实测 `thread '...' has overflowed its stack`）。已访问返空对象使 JS 仍可编译运行，循环依赖
/// 绑定解析为 undefined（转换式架构无 live binding，此为防崩溃的安全近似，非 spec 精确循环语义）。
/// `visited` 在整个模块图编译间共享（compile_module_script 起 `&mut HashSet` 透传）。
/// 依赖内联结果。Fresh 为模块 IIFE 代码（首次内联，调用方用 `var _mod_{safe}` 承接）；
/// Visited 为循环/菱形重入——模块体不得重复执行（webpack runtime 等有状态模块二次
/// 初始化会分裂实例，chunk 注册到 A 实例、require 走 B 实例 → 运行时 TypeError），
/// 改为对首份导出变量的守卫引用。
enum DepInline {
    Fresh(String),
    Visited(String), // safe 变量名（首次内联的 `_mod_{safe}`）
}

impl DepInline {
    /// 求值为该模块导出对象的表达式。Visited 用 `typeof` 守卫：变量在当前作用域可见
    /// （首份内联在外层，闭包可见）→ 共享同一导出实例；深层菱形不可见 → 回落空对象
    ///（与旧空占位行为一致，不劣于现状）。
    fn exports_expr(&self) -> String {
        match self {
            DepInline::Fresh(code) => code.clone(),
            DepInline::Visited(safe) => {
                format!("(typeof _mod_{safe} !== 'undefined' ? _mod_{safe} : (function(){{return {{}};}})())")
            }
        }
    }
}

fn inline_dep_once(
    specifier: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> Result<DepInline, ScriptError> {
    if !visited.contains(specifier) {
        visited.insert(specifier.to_string());
        return Ok(DepInline::Fresh(build_dep_iife(specifier, registry, visited)?));
    }
    // 已访问（循环/菱形重入）→ 引用首份内联导出，不重复内联。
    Ok(DepInline::Visited(safe_ident(specifier)))
}

/// 转换 import 声明。
/// 转换 import 语句（`import` 关键字已由 [`strip_import_keyword`] 剥离，传入子句）。
/// 同时支持常规与压缩（无关键字后空格）形态。
fn transform_import(
    clause: &str,
    importer_url: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> Result<String, ScriptError> {
    // import 'module' — 副作用导入
    if clause.starts_with('\'') || clause.starts_with('"') || clause.starts_with('`') {
        let raw_specifier = extract_string_literal(clause.split(';').next().unwrap_or(clause).trim())?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        let safe = safe_ident(&specifier);
        let dep = inline_dep_once(&specifier, registry, visited)?;
        return match &dep {
            DepInline::Fresh(code) => Ok(format!("  var _mod_{safe} = {code};\n")),
            DepInline::Visited(_) => Ok(String::new()),
        };
    }

    // import * as X from 'module'（压缩形态 import*as X from"m" 同样命中）
    if let Some(after_star) = clause.strip_prefix('*') {
        let Some(after_as) = after_star.trim_start().strip_prefix("as").map(str::trim_start) else {
            return Err(ScriptError::CompileError(format!("unsupported import: {clause}")));
        };
        let Some((ns_name, spec_part)) = split_from_clause(after_as) else {
            return Err(ScriptError::CompileError(format!("unsupported import: {clause}")));
        };
        let raw_specifier = extract_import_specifier_from_rest(spec_part)?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        // R3398：防循环/菱形 import 无限递归（仅首次访问时内联依赖 IIFE；重入 → 引用首份
        // 导出，避免 a↔b 循环致栈溢出 abort）。镜像 import 'm' 副作用导入的 visited 守卫。
        let safe = safe_ident(&specifier);
        let dep = inline_dep_once(&specifier, registry, visited)?;
        let exports_var = match &dep {
            DepInline::Fresh(_) => format!("_mod_{safe}"),
            DepInline::Visited(_) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code) => {
                result.push_str(&format!("  var {exports_var} = {code};\n"));
            }
            DepInline::Visited(_) => {
                result.push_str(&format!("  var {exports_var} = {};\n", dep.exports_expr()));
            }
        }
        result.push_str(&format!("  var {} = {exports_var};\n", ns_name.trim()));
        return Ok(result);
    }

    // import { X, Y as Z } from 'module'
    if clause.starts_with('{')
        && let Some((bindings, spec_part)) = split_from_clause(clause)
    {
        let raw_specifier = extract_import_specifier_from_rest(spec_part)?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        for item in bindings
            .trim_start_matches('{')
            .trim_end_matches('}')
            .split(',')
            .map(str::trim)
            .filter(|item| !item.is_empty())
        {
            let imported = item.split_once(" as ").map_or(item, |(name, _)| name).trim();
            ensure_module_export(&specifier, imported, registry)?;
        }
        let safe = safe_ident(&specifier);
        let dep = inline_dep_once(&specifier, registry, visited)?;
        let exports_var = match &dep {
            DepInline::Fresh(_) => format!("_mod_{safe}"),
            // 重入：首份导出在既有 `_mod_{safe}` 中；守卫引用存入独立名避免遮蔽外层声明
            DepInline::Visited(_) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code) => {
                result.push_str(&format!("  var {exports_var} = {code};\n"));
            }
            DepInline::Visited(_) => {
                result.push_str(&format!("  var {exports_var} = {};\n", dep.exports_expr()));
            }
        }
        result.push_str(&destructure_bindings(bindings, &exports_var));
        return Ok(result);
    }

    // import X from 'module' — 默认导入
    if let Some((name, spec_part)) = split_from_clause(clause) {
        let name = name.trim();
        let raw_specifier = extract_import_specifier_from_rest(spec_part)?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        ensure_module_export(&specifier, "default", registry)?;
        let safe = safe_ident(&specifier);
        let dep = inline_dep_once(&specifier, registry, visited)?;
        let exports_var = match &dep {
            DepInline::Fresh(_) => format!("_mod_{safe}"),
            DepInline::Visited(_) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code) => {
                result.push_str(&format!("  var {exports_var} = {code};\n"));
            }
            DepInline::Visited(_) => {
                result.push_str(&format!("  var {exports_var} = {};\n", dep.exports_expr()));
            }
        }
        result.push_str(&format!("  var {name} = ({exports_var}).default;\n"));
        return Ok(result);
    }

    Ok(String::new())
}

fn resolve_registered_specifier(specifier: &str, importer_url: &str, registry: &ModuleRegistry) -> String {
    if registry.get(specifier).is_some() {
        return specifier.to_string();
    }
    // 注册表直击未命中时先过页面 import map（HTML 规范 resolve a module specifier
    // 的 map 阶段）；无 map 或未命中（Miss）按原相对路径解析继续。
    // FIXME(spec)：Blocked（null 条目）规范要求抛 TypeError 终止全部回退，此处受
    // `String` 返回签名所限按 Miss 降级；键迭代顺序为 serde_json Map 排序序（非
    // JSON 插入序），仅嵌套尾斜杠前缀键 / 多 scope 重叠时可观察（见 import_map.rs 模块注释）。
    if let Some(map) = registry.import_map()
        && let ImportMapResolution::Resolved(mapped) = map.resolve(specifier, importer_url)
    {
        return mapped;
    }
    let Ok(base) = url::Url::parse(importer_url) else {
        return specifier.to_string();
    };
    let Ok(mut resolved) = base.join(specifier) else {
        return specifier.to_string();
    };
    resolved.set_fragment(None);
    let resolved = resolved.to_string();
    if registry.get(&resolved).is_some() {
        resolved
    } else {
        specifier.to_string()
    }
}

fn ensure_module_export(specifier: &str, name: &str, registry: &ModuleRegistry) -> Result<(), ScriptError> {
    if registry.get(specifier).is_none() {
        return Err(ScriptError::RuntimeError(format!("Module not found: {specifier}")));
    }
    if module_provides_export(specifier, name, registry, &mut HashSet::new()) {
        Ok(())
    } else {
        Err(ScriptError::CompileError(format!(
            "Module {specifier} does not provide an export named {name}"
        )))
    }
}

fn module_provides_export(
    specifier: &str,
    name: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> bool {
    if !visited.insert(specifier.to_string()) {
        return false;
    }
    let Some(source) = registry.get(specifier) else {
        return false;
    };
    for statement in split_statements(source) {
        let Some(rest) = strip_export_keyword(statement.trim()) else {
            continue;
        };
        if name == "default"
            && let Some(expr) = rest.strip_prefix("default")
            && !expr.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '$')
        {
            return true;
        }
        for declaration in ["const ", "let ", "var ", "function ", "class "] {
            if let Some(value) = rest.strip_prefix(declaration)
                && extract_binding_name(value) == name
            {
                return true;
            }
        }
        // export * [as N] from 'module'（压缩形态 export*from"m" 同样命中）
        if let Some(after_star) = rest.strip_prefix('*') {
            let after = after_star.trim_start();
            // export * as ns from 'dep' 仅提供命名空间绑定 ns 本身，不透传 dep 的
            // 具名导出（https://tc39.es/ecma262/#prod-ExportDeclaration
            // StarAsNamespaceExportClause）——不得落入下方 re-export-all 递归，
            // 否则 import{k} 会错误命中并运行时得到 undefined（审查 F5）。
            if let Some(after_as) = after.strip_prefix("as").map(str::trim_start) {
                if let Some((namespace, _)) = split_from_clause(after_as)
                    && namespace.trim() == name
                {
                    return true;
                }
                continue;
            }
            if name != "default"
                && let Some((_, spec_part)) = split_from_clause(after)
                && let Ok(raw) = extract_import_specifier_from_rest(spec_part)
            {
                let dependency = resolve_registered_specifier(&raw, specifier, registry);
                if module_provides_export(&dependency, name, registry, visited) {
                    return true;
                }
            }
        }
        if rest.starts_with('{')
            && let Some(end) = rest.find('}')
        {
            // from 子句支持常规与压缩形态（export{a}from"m"）
            let from_specifier = split_from_clause(rest[end + 1..].trim())
                .and_then(|(_, spec_part)| extract_import_specifier_from_rest(spec_part).ok())
                .map(|raw| resolve_registered_specifier(&raw, specifier, registry));
            for item in rest[1..end].split(',').map(str::trim).filter(|item| !item.is_empty()) {
                let (imported, exported) = item
                    .split_once(" as ")
                    .map_or((item, item), |(imported, exported)| (imported.trim(), exported.trim()));
                if exported == name
                    && from_specifier
                        .as_deref()
                        .is_none_or(|dependency| module_provides_export(dependency, imported, registry, visited))
                {
                    return true;
                }
            }
        }
    }
    false
}

/// 从 `from '...'` 部分提取模块标识符。
fn extract_import_specifier_from_rest(s: &str) -> Result<String, ScriptError> {
    let s = s.split(';').next().unwrap_or(s).trim();
    extract_string_literal(s)
}

/// 生成解构导入语句（从 `exports_var` 指向的模块导出对象解构）。
fn destructure_bindings(bindings: &str, exports_var: &str) -> String {
    let inner = bindings.trim_start_matches('{').trim_end_matches('}');
    let mut result = String::new();
    for item in inner.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        if let Some(pos) = item.find(" as ") {
            let src = item[..pos].trim();
            let alias = item[pos + 4..].trim();
            result.push_str(&format!("  var {alias} = {exports_var}.{src};\n"));
        } else {
            result.push_str(&format!("  var {item} = {exports_var}.{item};\n"));
        }
    }
    result
}

/// 转换 export 声明（`export` 关键字已由 [`strip_export_keyword`] 剥离，传入子句）。
/// 同时支持常规与压缩（无关键字后空格）形态。
fn transform_export(
    clause: &str,
    importer_url: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> Result<String, ScriptError> {
    // export * [as N] from 'module'（压缩形态 export*from"m" / export*as N from"m" 同样命中）
    if let Some(after_star) = clause.strip_prefix('*') {
        let after = after_star.trim_start();
        if let Some(after_as) = after.strip_prefix("as").map(str::trim_start) {
            let Some((namespace, spec_part)) = split_from_clause(after_as) else {
                return Err(ScriptError::CompileError(format!("unsupported export: {clause}")));
            };
            let raw_specifier = extract_import_specifier_from_rest(spec_part)?;
            let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
            let safe = safe_ident(&specifier);
            let dep = inline_dep_once(&specifier, registry, visited)?;
            let exports_var = match &dep {
                DepInline::Fresh(_) => format!("_mod_{safe}"),
                DepInline::Visited(_) => format!("_modref_{safe}"),
            };
            let mut result = String::new();
            match &dep {
                DepInline::Fresh(code) => {
                    result.push_str(&format!("  var {exports_var} = {code};\n"));
                }
                DepInline::Visited(_) => {
                    result.push_str(&format!("  var {exports_var} = {};\n", dep.exports_expr()));
                }
            }
            result.push_str(&format!("  _exports.{} = {exports_var};\n", namespace.trim()));
            // export * as N 同时产生 LocalName=N 的 import 绑定，模块体内可裸标识符
            // 访问（具名重导出 export {a as b} from 则不产生局部绑定）
            // https://tc39.es/ecma262/#sec-exports-static-semantics-importentries
            // 保留字无法被裸标识符引用，跳过局部声明以免语法错误。
            if !is_reserved_word(namespace.trim()) {
                result.push_str(&format!("  var {} = {exports_var};\n", namespace.trim()));
            }
            return Ok(result);
        }
        let Some((_empty, spec_part)) = split_from_clause(after) else {
            return Err(ScriptError::CompileError(format!("unsupported export: {clause}")));
        };
        let raw_specifier = extract_import_specifier_from_rest(spec_part)?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        let safe = safe_ident(&specifier);
        let dep = inline_dep_once(&specifier, registry, visited)?;
        let exports_var = match &dep {
            DepInline::Fresh(_) => format!("_mod_{safe}"),
            DepInline::Visited(_) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code) => {
                result.push_str(&format!("  var {exports_var} = {code};\n"));
            }
            DepInline::Visited(_) => {
                result.push_str(&format!("  var {exports_var} = {};\n", dep.exports_expr()));
            }
        }
        result.push_str(&format!(
            "  var _reexport_{safe} = {exports_var};\n  Object.keys(_reexport_{safe}).forEach(function(key) {{ if (key !== 'default') _exports[key] = _reexport_{safe}[key]; }});\n",
        ));
        return Ok(result);
    }
    if clause.starts_with('{')
        && let Some((before, spec_part)) = split_from_clause(clause)
    {
        let end = before
            .find('}')
            .ok_or_else(|| ScriptError::CompileError("invalid re-export list: missing }".into()))?;
        let raw_specifier = extract_import_specifier_from_rest(spec_part)?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        let safe = safe_ident(&specifier);
        for item in before[1..end].split(',').map(str::trim).filter(|item| !item.is_empty()) {
            let imported = item.split_once(" as ").map_or(item, |(name, _)| name).trim();
            ensure_module_export(&specifier, imported, registry)?;
        }
        let dep = inline_dep_once(&specifier, registry, visited)?;
        let exports_var = match &dep {
            DepInline::Fresh(_) => format!("_mod_{safe}"),
            DepInline::Visited(_) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code) => {
                result.push_str(&format!("  var {exports_var} = {code};\n"));
            }
            DepInline::Visited(_) => {
                result.push_str(&format!("  var {exports_var} = {};\n", dep.exports_expr()));
            }
        }
        result.push_str(&format!("  var _reexport_{safe} = {exports_var};\n"));
        for item in before[1..end].split(',').map(str::trim).filter(|item| !item.is_empty()) {
            if let Some(pos) = item.find(" as ") {
                let imported = item[..pos].trim();
                let exported = item[pos + 4..].trim();
                result.push_str(&format!("  _exports.{exported} = _reexport_{safe}.{imported};\n"));
            } else {
                result.push_str(&format!("  _exports.{item} = _reexport_{safe}.{item};\n"));
            }
        }
        return Ok(result);
    }
    // export default expr — 压缩形态 export default{…} 同样命中；`default` 后为
    // 标识符组成字符时不匹配（`export defaultx` 是非法语句，交回退路径原样报错）。
    if let Some(expr) = clause.strip_prefix("default")
        && !expr.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '$')
    {
        let expr = rewrite_dynamic_imports(&expr.trim_start().replace("import.meta", "_importMeta"));
        return Ok(format!("  _exports.default = {expr};\n"));
    }
    // 声明体可能含 import.meta 与动态 import()（github react-core 实测：
    // `export const __webpack_modules__={…import.meta.hot… await import(…)…}`，
    // 2026-10-09 线上 SyntaxError: Cannot use 'import.meta' outside a module），
    // 重发前须与普通语句路径（build_dep_iife else 分支）同步重写——IIFE 以经典
    // 脚本执行，import.meta 仅模块可用；动态 import() 须走宿主桥接。
    if let Some(decl) = clause.strip_prefix("const ") {
        let name = extract_binding_name(decl);
        let decl = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  const {decl};\n  _exports.{name} = {name};\n"));
    }
    if let Some(decl) = clause.strip_prefix("let ") {
        let name = extract_binding_name(decl);
        let decl = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  let {decl};\n  _exports.{name} = {name};\n"));
    }
    if let Some(decl) = clause.strip_prefix("var ") {
        let name = extract_binding_name(decl);
        let decl = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  var {decl};\n  _exports.{name} = {name};\n"));
    }
    if let Some(decl) = clause.strip_prefix("function ") {
        let name = extract_binding_name(decl);
        let decl = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  function {decl}\n  _exports.{name} = {name};\n"));
    }
    if let Some(decl) = clause.strip_prefix("class ") {
        let name = extract_binding_name(decl);
        let decl = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  class {decl}\n  _exports.{name} = {name};\n"));
    }

    // export { X, Y as Z }
    if clause.starts_with('{') {
        let end = clause
            .find('}')
            .ok_or_else(|| ScriptError::CompileError("invalid export list: missing }".into()))?;
        let list_str = &clause[1..end];
        let mut result = String::new();
        for item in list_str.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }
            if let Some(pos) = item.find(" as ") {
                let local = item[..pos].trim();
                let exported = item[pos + 4..].trim();
                result.push_str(&format!("  _exports.{exported} = {local};\n"));
            } else {
                result.push_str(&format!("  _exports.{item} = {item};\n"));
            }
        }
        return Ok(result);
    }

    // 未识别形态原样保留 export 关键字交 V8 报错（不静默丢语义）
    Ok(format!("  export {clause};\n"))
}

// ── 辅助函数 ──

/// `/` 是否为正则字面量起点（而非除号）：按前一个有效字符判定。
/// 无前文/运算符后 → 正则；标识符组成字符后仅关键字（return/typeof/in 等）→ 正则，
/// 其余标识符与数字 → 除号；`)`/`]`/`.`/`+`/`-`/引号后 → 除号；`}` 按块语句结尾 → 正则。
/// 两条路径都原样保留文本，误判只影响切分点位置，不破坏字面量。
///
/// 已知限制（审查 F2）：前缀归除号侧的字符后接正则时（如 `if(x)/re/.test(y)`、
/// `a-/re/.test(s)`）不原子消费——正则体按普通文本扫描，仅当体内含引号或
/// 语句边界字符时才产生错误切分；true 误判则把除法两侧当正则消费。
/// 两者都产生错误切分而非文本破坏，暂以穷举关键字清单控制误判面。
fn is_regex_literal_start(prev: Option<&char>, prev_word: &str) -> bool {
    match prev {
        None => true,
        Some(')' | ']' | '.' | '+' | '-' | '\'' | '"' | '`') => false,
        Some(c) if c.is_alphanumeric() || *c == '_' || *c == '$' => matches!(
            prev_word,
            "return"
                | "typeof"
                | "instanceof"
                | "in"
                | "of"
                | "new"
                | "delete"
                | "void"
                | "case"
                | "do"
                | "else"
                | "yield"
                | "await"
                | "throw"
        ),
        Some(_) => true,
    }
}

/// 语句切分扫描上下文。
/// - Str：字符串/模板字面量内部，直到闭合分隔符（模板字面量内的 `${` 转入 TemplateBrace）。
/// - TemplateBrace：模板字面量 `${ }` 插值代码内部（携带剩余嵌套深度，深度归零出插值）。
///
/// <https://tc39.es/ecma262/#prod-TemplateLiteral>
#[derive(Clone, Copy)]
enum ScanCtx {
    Str(char),
    TemplateBrace(u32),
}

/// Split top-level module statements without breaking multiline function or arrow bodies.
fn split_statements(source: &str) -> Vec<String> {
    let mut stmts = Vec::new();
    let mut current = String::new();
    let mut braces = 0usize;
    let mut parens = 0usize;
    let mut brackets = 0usize;
    let mut ctx_stack: Vec<ScanCtx> = Vec::new();
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;
    // 正则字面量判定上下文：最后一个非空白有效字符与其标识符 token
    let mut prev_significant: Option<char> = None;
    let mut prev_word = String::new();
    let mut after_whitespace = false;
    let chars = source.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        let next = chars.get(index + 1).copied();
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                if braces == 0 && parens == 0 && brackets == 0 && ctx_stack.is_empty() {
                    let statement = current.trim();
                    if !statement.is_empty() {
                        stmts.push(statement.to_string());
                    }
                    current.clear();
                }
            }
            index += 1;
            continue;
        }
        if block_comment {
            if ch == '*' && next == Some('/') {
                block_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if let Some(&top) = ctx_stack.last() {
            match top {
                ScanCtx::Str(delimiter) => {
                    current.push(ch);
                    if escaped {
                        escaped = false;
                    } else if ch == '\\' {
                        escaped = true;
                    } else if ch == delimiter {
                        ctx_stack.pop();
                    } else if delimiter == '`' && ch == '$' && next == Some('{') {
                        // `${` 进入插值代码上下文（插值里的嵌套模板/字符串另起 Str 层）
                        ctx_stack.pop();
                        ctx_stack.push(ScanCtx::TemplateBrace(1));
                        current.push('{');
                        index += 2;
                        prev_significant = Some('{');
                        prev_word.clear();
                        continue;
                    }
                    prev_significant = Some(ch);
                    index += 1;
                    continue;
                }
                ScanCtx::TemplateBrace(depth) => {
                    if ch == '/' && next == Some('/') {
                        line_comment = true;
                        index += 2;
                        continue;
                    }
                    if ch == '/' && next == Some('*') {
                        block_comment = true;
                        index += 2;
                        continue;
                    }
                    if ch == '/' && is_regex_literal_start(prev_significant.as_ref(), &prev_word) {
                        current.push(ch);
                        index += 1;
                        let mut in_class = false;
                        while index < chars.len() {
                            let rc = chars[index];
                            current.push(rc);
                            if rc == '\\' && index + 1 < chars.len() {
                                current.push(chars[index + 1]);
                                index += 2;
                                continue;
                            }
                            index += 1;
                            if rc == '[' {
                                in_class = true;
                            } else if rc == ']' {
                                in_class = false;
                            } else if rc == '/' && !in_class {
                                while index < chars.len() && chars[index].is_ascii_alphabetic() {
                                    current.push(chars[index]);
                                    index += 1;
                                }
                                break;
                            }
                        }
                        prev_significant = Some(')');
                        prev_word.clear();
                        continue;
                    }
                    current.push(ch);
                    match ch {
                        '{' => *ctx_stack.last_mut().expect("ctx") = ScanCtx::TemplateBrace(depth + 1),
                        '}' => {
                            if depth <= 1 {
                                ctx_stack.pop();
                                // https://tc39.es/ecma262/#prod-TemplateLiteral
                                // 出插值后回到模板正文扫描：否则模板正文里的裸换行/引号/`${`
                                // 会按顶层代码处理（github.com 多行错误消息模板实测踩坑）。
                                ctx_stack.push(ScanCtx::Str('`'));
                            } else {
                                *ctx_stack.last_mut().expect("ctx") = ScanCtx::TemplateBrace(depth - 1);
                            }
                        }
                        '\'' | '"' | '`' => ctx_stack.push(ScanCtx::Str(ch)),
                        _ => {}
                    }
                    if !ch.is_whitespace() {
                        if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                            if after_whitespace {
                                prev_word.clear();
                            }
                            prev_word.push(ch);
                        } else {
                            prev_word.clear();
                        }
                        prev_significant = Some(ch);
                        after_whitespace = false;
                    } else {
                        after_whitespace = true;
                    }
                    index += 1;
                    continue;
                }
            }
        }
        if ch == '/' && next == Some('/') {
            line_comment = true;
            index += 2;
            continue;
        }
        if ch == '/' && next == Some('*') {
            block_comment = true;
            index += 2;
            continue;
        }
        // https://tc39.es/ecma262/#prod-RegularExpressionLiteral
        // 正则字面量与除号共用 `/`——按前一个有效 token 判定（标识符/数字/`)`/`]`/`.` 后是
        // 除号；运算符、关键字后是正则；`}` 按块语句结尾视为正则起点）。正则整体原子消费
        //（内部引号/分号/括号不得影响引号跟踪与语句切分），文本原样保留。
        if ch == '/' && is_regex_literal_start(prev_significant.as_ref(), &prev_word) {
            current.push(ch);
            index += 1;
            let mut in_class = false;
            while index < chars.len() {
                let rc = chars[index];
                current.push(rc);
                if rc == '\\' && index + 1 < chars.len() {
                    current.push(chars[index + 1]);
                    index += 2;
                    continue;
                }
                index += 1;
                if rc == '[' {
                    in_class = true;
                } else if rc == ']' {
                    in_class = false;
                } else if rc == '/' && !in_class {
                    // flags（gimsuyvd 等单字母修饰符）跟随闭合 `/`
                    while index < chars.len() && chars[index].is_ascii_alphabetic() {
                        current.push(chars[index]);
                        index += 1;
                    }
                    break;
                }
            }
            // 正则字面量是值：其后的 `/` 是除号
            prev_significant = Some(')');
            prev_word.clear();
            continue;
        }
        match ch {
            '\'' | '"' | '`' => ctx_stack.push(ScanCtx::Str(ch)),
            '{' => braces += 1,
            '}' => braces = braces.saturating_sub(1),
            '(' => parens += 1,
            ')' => parens = parens.saturating_sub(1),
            '[' => brackets += 1,
            ']' => brackets = brackets.saturating_sub(1),
            _ => {}
        }
        let top_level = braces == 0 && parens == 0 && brackets == 0;
        if !ch.is_whitespace() {
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                if after_whitespace {
                    prev_word.clear();
                }
                prev_word.push(ch);
            } else {
                prev_word.clear();
            }
            prev_significant = Some(ch);
            after_whitespace = false;
        } else {
            after_whitespace = true;
        }
        if (ch == ';' || ch == '\n') && top_level {
            let statement = current.trim();
            if !statement.is_empty() {
                stmts.push(statement.to_string());
            }
            current.clear();
        } else {
            current.push(ch);
        }
        index += 1;
    }
    let statement = current.trim();
    if !statement.is_empty() {
        stmts.push(statement.to_string());
    }
    stmts
}

/// 提取字符串字面量（从首个引号起，到匹配的闭合引号止，忽略其后字符）。
///
/// R3349 deep-review：旧实现在首个 `;` 切分后要求整段以闭合引号**结尾**（`ends_with(close)`），
/// 对动态 `import('./x.js')`（引号后紧跟 `)`）恒判 unclosed → 标识符全被丢弃。改为按字符扫描到
/// 匹配的闭合引号即止，**忽略其后的 `)`/`;`/剩余代码**——既修动态 import，也保持静态 import
///（`'./a.js'` 后无非空白字符时行为不变）向后兼容。反斜杠转义引号不计为闭合。
fn extract_string_literal(s: &str) -> Result<String, ScriptError> {
    let s = s.trim();
    let mut chars = s.chars();
    let close = match chars.next() {
        Some('\'') => '\'',
        Some('"') => '"',
        Some('`') => '`',
        _ => return Err(ScriptError::CompileError(format!("expected string literal, got: {s}"))),
    };
    let mut out = String::new();
    let mut escaped = false;
    let mut closed = false;
    for c in chars {
        if escaped {
            out.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' {
            escaped = true;
            continue;
        }
        if c == close {
            closed = true;
            break;
        }
        out.push(c);
    }
    if closed {
        Ok(out)
    } else {
        Err(ScriptError::CompileError(format!("unclosed string literal: {s}")))
    }
}

/// 从声明中提取绑定名称。
fn extract_binding_name(decl: &str) -> &str {
    let decl = decl.trim();
    let end = decl
        .find(|c: char| c.is_whitespace() || c == '=' || c == '(' || c == '{')
        .unwrap_or(decl.len());
    let name = &decl[..end];
    if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') && !name.is_empty() {
        name
    } else {
        "unknown"
    }
}

pub(crate) fn expose_classic_script_lexicals(source: &str) -> String {
    let mut names = Vec::new();
    for statement in split_statements(source) {
        let statement = statement.trim_start();
        for prefix in ["const ", "let ", "class ", "function ", "async function "] {
            if let Some(declaration) = statement.strip_prefix(prefix) {
                let name = extract_binding_name(declaration);
                if name != "unknown" {
                    names.push(name.to_string());
                }
                break;
            }
        }
    }
    if names.is_empty() {
        return source.to_string();
    }
    let mut output = source.to_string();
    for name in names {
        output.push_str("\nglobalThis.");
        output.push_str(&name);
        output.push_str(" = ");
        output.push_str(&name);
        output.push(';');
    }
    output
}

/// JSON 字符串转义。
fn json_stringify(s: &str) -> String {
    let escaped = s
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

/// 将模块标识符转换为安全的 JS 标识符。
fn safe_ident(specifier: &str) -> String {
    // 折叠不可逆（"/"与"_"折叠出同一标识符），不同 URL 会碰撞到同一 _mod_ 变量，
    // 菱形重入改读 var _mod_{safe} 后会静默绑定到错误模块实例（recheck-b3 B3-1）——
    // 追加 FNV-1a 摘要后缀使 URL→标识符映射可注入。
    let mut safe = String::new();
    for c in specifier.chars() {
        if c.is_alphanumeric() || c == '_' {
            safe.push(c);
        } else {
            safe.push('_');
        }
    }
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for b in specifier.as_bytes() {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let suffix = format!("_h{hash:016x}");
    let safe = safe.trim_matches('_');
    if safe.is_empty() {
        return format!("_mod{suffix}");
    }
    if safe.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("_{safe}{suffix}")
    } else {
        format!("{safe}{suffix}")
    }
}

/// tc39 ModuleExportName 允许保留字（`export * as class from` 合法），但包裹体
/// 总在 `'use strict'` 下运行：局部 `var <name>` 对保留字及 `eval`/`arguments` 均为
/// 语法错误——这些名字不发局部声明（recheck-b3 B3-2）。
/// https://tc39.es/ecma262/#sec-keywords-and-reserved-words
/// https://tc39.es/ecma262/#sec-identifiers-static-semantics-early-errors
fn is_reserved_word(name: &str) -> bool {
    matches!(
        name,
        "arguments"
            | "await"
            | "eval"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "let"
            | "static"
            | "implements"
            | "interface"
            | "package"
            | "private"
            | "protected"
            | "public"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_registry_new() {
        let reg = ModuleRegistry::new();
        assert!(reg.is_empty());
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_module_registry_register_and_get() {
        let mut reg = ModuleRegistry::new();
        reg.register("./utils.js", "export const PI = 3.14;");
        assert_eq!(reg.len(), 1);
        assert_eq!(reg.get("./utils.js"), Some("export const PI = 3.14;"));
    }

    #[test]
    fn test_module_registry_unregister() {
        let mut reg = ModuleRegistry::new();
        reg.register("./a.js", "export const a = 1;");
        assert!(reg.unregister("./a.js"));
        assert!(!reg.unregister("./a.js"));
    }

    #[test]
    fn test_module_registry_specifiers() {
        let mut reg = ModuleRegistry::new();
        reg.register("./a.js", "");
        reg.register("./b.js", "");
        let mut specs = reg.specifiers();
        specs.sort();
        assert_eq!(specs, vec!["./a.js", "./b.js"]);
    }

    // P6 import map：裸说明符经 map 命中已注册模块；无 map 时保持既有行为（判别）。
    #[test]
    fn test_import_map_resolves_bare_specifier_to_registered_module() {
        let base = url::Url::parse("https://github.com/").unwrap();
        let map = ImportMap::parse(r#"{"imports": {"react": "https://assets.test/react.js"}}"#, &base).unwrap();
        let mut reg = ModuleRegistry::new();
        reg.set_import_map(map);
        reg.register("https://assets.test/react.js", "export default 1;");
        assert_eq!(
            resolve_registered_specifier("react", "https://github.com/page", &reg),
            "https://assets.test/react.js"
        );
    }

    #[test]
    fn test_without_import_map_bare_specifier_unchanged() {
        let mut reg = ModuleRegistry::new();
        reg.register("https://github.com/react.js", "export default 1;");
        // 无 map：裸说明符按相对路径解析未命中 → 原样返回（既有行为）。
        assert_eq!(
            resolve_registered_specifier("react", "https://github.com/page", &reg),
            "react"
        );
        // 相对路径命中注册模块仍走原路径（无 map 回归保护）。
        assert_eq!(
            resolve_registered_specifier("./react.js", "https://github.com/page", &reg),
            "https://github.com/react.js"
        );
    }

    #[test]
    fn test_es_module_sandbox_new() {
        assert!(EsModuleSandbox::new().is_ok());
    }

    #[test]
    fn test_es_module_sandbox_debug() {
        let sandbox = EsModuleSandbox::new().unwrap();
        assert!(format!("{sandbox:?}").contains("EsModuleSandbox"));
    }

    #[test]
    fn test_execute_module_empty() {
        let mut sb = EsModuleSandbox::new().unwrap();
        assert!(matches!(sb.execute_module("", None), Err(ScriptError::InvalidInput(_))));
    }

    #[test]
    fn test_export_const() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb.execute_module("export const x = 42;", None).unwrap();
        assert!(r.namespace_json.contains("42"));
    }

    #[test]
    fn test_export_default() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb.execute_module("export default 99;", None).unwrap();
        assert!(r.namespace_json.contains("99"));
    }

    #[test]
    fn test_export_function() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb
            .execute_module("export function add(a, b) { return a + b; }", None)
            .unwrap();
        assert!(!r.namespace_json.is_empty());
    }

    #[test]
    fn test_export_list() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb
            .execute_module("const a = 1\nconst b = 2\nexport { a, b as c }", None)
            .unwrap();
        assert!(!r.namespace_json.is_empty());
    }

    #[test]
    fn test_export_let() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb.execute_module("export let count = 100;", None).unwrap();
        assert!(!r.namespace_json.is_empty());
    }

    #[test]
    fn test_export_var() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb.execute_module("export var name = 'test';", None).unwrap();
        assert!(!r.namespace_json.is_empty());
    }

    #[test]
    fn test_export_class() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb.execute_module("export class MyClass {}", None).unwrap();
        assert!(!r.namespace_json.is_empty());
    }

    #[test]
    fn test_export_multiple() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb
            .execute_module("export const a = 1\nexport const b = 2\nexport default a + b", None)
            .unwrap();
        assert!(r.namespace_json.contains("3"));
    }

    #[test]
    fn test_import_meta() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb
            .execute_module("export default import.meta.url;", Some("https://example.com/module.js"))
            .unwrap();
        assert!(r.namespace_json.contains("https://example.com/module.js"));
    }

    /// t8j（site-compat bilibili laputa-home）：动态 import() 取回/编译失败须以 rejected
    /// promise 呈现（https://tc39.es/ecma262/#sec-import-calls——ImportCall 恒返回 promise），
    /// 不得同步 throw。同步 throw 发生在调用方 `.catch()` 挂上之前，Vite 现代浏览器探测
    /// `import("_").catch(()=>1)` 整体中断、后续语句永不执行 → 站点误回落 legacy 路径。
    /// 红态：`__zw_dynamic_import` 在 `__zw_compile_module` 返空（取回失败）时同步抛
    /// `Module not found`，本测试首个 execute 即 Err。
    #[test]
    fn test_dynamic_import_failure_rejects_not_throws() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        // 模拟宿主取回失败：__zw_compile_module 返空 → __zw_load_module 抛 Module not found。
        sandbox.register_callback("__zw_compile_module", Box::new(|_args| String::new()));
        // promise 落定结果经宿主回调回传（execute 间 global 不共享——非 persistent
        // context 每次 execute 新建；真实路径 prelude+模块同为单次 execute）。
        let settled: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let settled_cb = settled.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *settled_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        let prelude = build_module_runtime_prelude(&ModuleRegistry::new()).unwrap();
        // 忠实复刻 execute_module_in_sandbox：prelude + 调用方代码单次 execute；
        // 微任务检查点在 execute 内跑 → .then 落定发生在返回前，__zw_report 已被调。
        sandbox
            .execute(&format!(
                "{prelude}\n\
                     __zw_dynamic_import('_nope').then(\n\
                       function () {{ __zw_report('fulfilled'); }},\n\
                       function (e) {{ __zw_report('rejected:' + (e && e.message)); }});\n"
            ))
            .expect("动态 import 失败不得同步 throw");
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("rejected:Module not found: _nope"),
            "失败以 rejection 呈现且原因携带 spec（红态：同步 throw 后首个 execute 即 Err）"
        );
    }

    /// t8j F3（site-compat bilibili laputa-home）：prelude 语句顺序钉——helper 定义必须先于
    /// `__moduleCache[spec] = IIFE` 注册求值。collect_module_deps 把入口模块自身注册进 registry，
    /// 注册右侧 IIFE 立即执行模块体；模块体顶层 `import()`（Vite 探测形态）在 helper 未定义时
    /// 同步 ReferenceError（红态：反序 prelude 中模块体在 `__zw_dynamic_import` 定义前执行）。
    #[test]
    fn test_prelude_helpers_precede_module_registrations() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        sandbox.register_callback("__zw_compile_module", Box::new(|_args| String::new()));
        // execute 间 global 不共享（非 persistent context 每次 execute 新建）——模块体执行
        // 结果经宿主回调在同一 execute 内回传（与 F2 钉同法）。
        let reported: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let reported_cb = reported.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *reported_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        let mut registry = ModuleRegistry::new();
        // 入口模块自身进 registry（复刻 collect_module_deps 行为）+ 体顶层动态 import 探测。
        registry.register(
            "https://zero.test/m.js",
            "globalThis.__t8j_probe_ran = true;\nimport('_').catch(function(){});\nglobalThis.__t8j_after_import = true;\n__zw_report('after=' + (globalThis.__t8j_after_import === true));",
        );
        let prelude = build_module_runtime_prelude(&registry).unwrap();
        sandbox
            .execute(&prelude)
            .expect("注册求值阶段模块体顶层 import() 不得同步 throw（红态：helper 反序 → ReferenceError）");
        assert_eq!(
            reported.lock().unwrap().as_deref(),
            Some("after=true"),
            "模块体在注册求值阶段跑完 import() 之后的语句（探测被吞、流程继续）"
        );
    }

    /// t8j F4（site-compat bilibili laputa-home）：动态 import() 相对 spec 按调用点 referrer
    /// 解析钉——rewrite_dynamic_imports 经 `.call({referrer})` 把模块 IIFE 的 `_importMeta.url`
    /// 穿给 helper，`__zw_compile_module(spec, parent)` 收到的 parent 须为调用模块 URL 而非
    /// 'about:blank'（红态：旧 helper 读不到 IIFE 局部 _importMeta，相对 spec 按原样发起请求）。
    /// https://tc39.es/ecma262/#sec-module-specifiers（相对 specifier 相对 referrer 解析）。
    #[test]
    fn test_dynamic_import_referrer_resolves_relative_spec() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        let seen_parent: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let seen_parent_cb = seen_parent.clone();
        let reported: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let reported_cb = reported.clone();
        // 宿主取回调：（spec, parent）——parent 即断言面；相对 spec 命中已注册依赖时返回
        // `resolved\x1fcode`（t8j-r2 D1 契约，复刻 renderer/tab 宿主形态），否则返空
        // （走 Module not found rejection）。
        sandbox.register_callback(
            "__zw_compile_module",
            Box::new(move |args| {
                let parent = args.get(1).cloned().unwrap_or_default();
                let is_target =
                    args.first().map(String::as_str) == Some("./dep.js") && parent == "https://zero.test/m.js";
                *seen_parent_cb.lock().unwrap() = Some(parent);
                if is_target {
                    "https://zero.test/dep.js\u{1f}(function(){ globalThis.__t8j_dep_loaded = true; return { default: 42 }; })()"
                        .to_string()
                } else {
                    String::new()
                }
            }),
        );
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *reported_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        let mut registry = ModuleRegistry::new();
        registry.register(
            "https://zero.test/m.js",
            "import('./dep.js').then(function (m) { __zw_report('dep=' + m.default); }, function (e) { __zw_report('rej=' + e.message); });",
        );
        let prelude = build_module_runtime_prelude(&registry).unwrap();
        sandbox.execute(&prelude).expect("模块体注册求值不得同步 throw");
        assert_eq!(
            seen_parent.lock().unwrap().as_deref(),
            Some("https://zero.test/m.js"),
            "宿主取回调收到的 parent = 调用模块 URL（红态：'about:blank'，相对 spec 无法解析）"
        );
        assert_eq!(
            reported.lock().unwrap().as_deref(),
            Some("dep=42"),
            "相对 spec 命中依赖并加载（红态：Module not found rejection）"
        );
    }

    /// t8j-r2 D1（缺陷审查返修）：动态 import 缓存键 = referrer×spec 的解析结果——两个不同
    /// 目录的模块各自 `import('./config.js')` 时各得自己的模块（ECMA-262
    /// §sec-hostresolveimportedmodule：解析键为 referrer×specifier）。宿主回传
    /// `resolved\x1fcode`，JS 侧以宿主解析键为缓存键。红态（PR #109 首 版）：缓存以原始
    /// specifier 为键，B 命中 A 写入的条目，静默拿到 A 的模块且宿主只被取回一次（首版）。
    #[test]
    fn test_dynamic_import_cache_keyed_by_resolved_url() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        let seen: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let seen_cb = seen.clone();
        let reported: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let reported_cb = reported.clone();
        // 宿主解析 stub：`./x` 相对 parent 目录 join（resolve_document_url 语义子集），
        // 返回 `resolved\x1fiife`；config 内容按目录区分（A/B）。
        sandbox.register_callback(
            "__zw_compile_module",
            Box::new(move |args| {
                let spec = args.first().cloned().unwrap_or_default();
                let parent = args.get(1).cloned().unwrap_or_default();
                let dir = match parent.rfind('/') {
                    Some(i) => &parent[..=i],
                    None => "",
                };
                let resolved = match spec.strip_prefix("./") {
                    Some(rest) => format!("{dir}{rest}"),
                    None => spec.clone(),
                };
                seen_cb.lock().unwrap().push(resolved.clone());
                let tag = if resolved.contains("/app/a/") { "A" } else { "B" };
                format!("{resolved}\u{1f}(function(){{ return {{ default: '{tag}-config' }}; }})()")
            }),
        );
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *reported_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        let prelude = build_module_runtime_prelude(&ModuleRegistry::new()).unwrap();
        // prelude 与驱动代码单次 execute（非持久 context 跨 execute 全局不可见，复刻真实路径）。
        let driver = "\
var a = __zw_load_module('./config.js', 'https://zero.test/app/a/m.js');\n\
var b = __zw_load_module('./config.js', 'https://zero.test/app/b/m.js');\n\
__zw_report('A=' + a.default + '|B=' + b.default);";
        sandbox
            .execute(&format!("{prelude}\n{driver}"))
            .expect("跨目录同名相对 spec 取回不得失败");
        assert_eq!(
            reported.lock().unwrap().as_deref(),
            Some("A=A-config|B=B-config"),
            "各目录模块拿到各自的 config（红态：B 静默拿到 A-config）"
        );
        assert_eq!(
            seen.lock().unwrap().len(),
            2,
            "两个不同解析键各触发一次宿主取回（红态：B 命中缓存、宿主只被调用 1 次）"
        );
    }

    /// t8j-r2 D2（缺陷审查返修）：rewrite 发射体零引号——模块源码单引号字符串字面量含
    /// `import(` 时改写不得破坏语法。红态（PR #109 首版发射串含 4 个单引号）：发射进
    /// `'import() failed'` 提前终止字符串 → 整 execute SyntaxError（prelude+模块单脚本，
    /// 单模块坏全文死）。改写后的值污染（子串扫描既有限制）与 base 同级，可接受。
    #[test]
    fn test_rewrite_single_quoted_string_with_import_stays_parseable() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        sandbox.register_callback("__zw_compile_module", Box::new(|_args| String::new()));
        let reported: std::sync::Arc<std::sync::Mutex<Option<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let reported_cb = reported.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                *reported_cb.lock().unwrap() = args.first().cloned();
                String::new()
            }),
        );
        let registry = ModuleRegistry::new();
        let prelude = build_module_runtime_prelude(&registry).unwrap();
        // 单引号字符串含 `import(`（MDN 式特性检测/错误消息常见形态）+ 合法动态 import
        // （dep 缺失走 rejection）——两语句都必须存活。
        let source = "\
var s = 'import() failed';\n\
import('./dep.js').then(function () { __zw_report('dep-ok'); }, function (e) { __zw_report('slen=' + s.length + '|rej=' + (e && e.message)); });";
        let module = build_module_script(source, "https://zero.test/m.js", &registry, false, &mut HashSet::new())
            .expect("模块编译不得失败");
        sandbox
            .execute(&format!("{prelude}\n{module}"))
            .expect("单引号字符串含 import( 时改写不得产生 SyntaxError（红态：整脚本解析失败）");
        let text = reported.lock().unwrap().clone();
        let text = text.as_deref().unwrap_or_default();
        assert!(
            text.starts_with("slen=") && text.contains("rej=Module not found: ./dep.js"),
            "字符串字面量后续语句执行、动态 import 走 rejection（实际: {text:?}）"
        );
    }

    #[test]
    fn test_import_destructure() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./math.js", "export const PI = 3.14\nexport const E = 2.72");
        let r = sb
            .execute_module("import { PI } from './math.js'\nexport default PI", None)
            .unwrap();
        assert!(r.namespace_json.contains("3.14"));
    }

    #[test]
    fn test_import_default() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./config.js", "export default { name: 'ZeroWeb' }");
        let r = sb
            .execute_module("import config from './config.js'\nexport default config.name", None)
            .unwrap();
        assert!(r.namespace_json.contains("ZeroWeb"));
    }

    #[test]
    fn test_import_alias() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./utils.js", "export const value = 42");
        let r = sb
            .execute_module("import { value as v } from './utils.js'\nexport default v", None)
            .unwrap();
        assert!(r.namespace_json.contains("42"));
    }

    #[test]
    fn test_import_not_found() {
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb.execute_module("import { x } from './missing.js'\nexport default x", None);
        assert!(r.is_err());
        assert!(r.unwrap_err().to_string().contains("Module not found"));
    }

    #[test]
    fn test_import_missing_exports_fails_during_compilation() {
        let mut registry = ModuleRegistry::new();
        registry.register("./dependency.js", "export const present = 1;");
        for source in [
            "import missing from './dependency.js';",
            "import { missing } from './dependency.js';",
            "export { missing } from './dependency.js';",
        ] {
            let error = compile_module_script(source, "https://example.test/sw.js", &registry).unwrap_err();
            assert!(matches!(error, ScriptError::CompileError(_)));
            assert!(error.to_string().contains("does not provide an export named"));
        }
    }

    #[test]
    fn test_export_star_as_namespace_does_not_reexport_named() {
        // export * as ns 只提供命名空间绑定 ns 本身，不透传 dep 的具名导出
        // （审查 F5；红态：import{k} 错误命中 → 运行时 undefined）。
        let mut registry = ModuleRegistry::new();
        registry.register("./dep.js", "export const k = 1;");
        registry.register("./mid.js", "export*as ns from'./dep.js';");
        let error = compile_module_script(
            "import { k } from './mid.js';",
            "https://example.test/entry.js",
            &registry,
        )
        .unwrap_err();
        assert!(error.to_string().contains("does not provide an export named k"));
        // 命名空间绑定本身仍可导入。
        let ok = compile_module_script(
            "import { ns } from './mid.js';",
            "https://example.test/entry.js",
            &registry,
        );
        assert!(ok.is_ok(), "export * as ns 应满足同名具名导入");
    }

    #[test]
    fn imported_classic_script_exposes_top_level_lexical_binding() {
        assert_eq!(
            expose_classic_script_lexicals("const imported = 'value';"),
            "const imported = 'value';\nglobalThis.imported = imported;"
        );
        assert_eq!(
            expose_classic_script_lexicals("function load() { const nested = 1; }"),
            "function load() { const nested = 1; }\nglobalThis.load = load;"
        );
        assert_eq!(
            expose_classic_script_lexicals("async function prepare() { return 1; }"),
            "async function prepare() { return 1; }\nglobalThis.prepare = prepare;"
        );
    }

    #[test]
    fn test_namespace_import() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./math.js", "export const x = 10\nexport const y = 20");
        let r = sb
            .execute_module(
                "import * as math from './math.js'\nexport default math.x + math.y",
                None,
            )
            .unwrap();
        assert!(r.namespace_json.contains("30"));
    }

    #[test]
    fn test_side_effect_import() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./side.js", "var _ran = true");
        let r = sb
            .execute_module("import './side.js'\nexport default 'done'", None)
            .unwrap();
        assert!(r.namespace_json.contains("done"));
    }

    #[test]
    fn test_chain_imports() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./a.js", "export const val = 5");
        sb.register_module("./b.js", "import { val } from './a.js'\nexport const doubled = val * 2");
        let r = sb
            .execute_module("import { doubled } from './b.js'\nexport default doubled", None)
            .unwrap();
        assert!(r.namespace_json.contains("10"));
    }

    #[test]
    fn test_chain_imports_resolve_canonical_urls_per_importer() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module(
            "https://example.test/workers/lib/entry.js",
            "import { val } from './value.js'; export const doubled = val * 2",
        );
        sb.register_module("https://example.test/workers/lib/value.js", "export const val = 6");
        let result = sb
            .execute_module(
                "import { doubled } from './lib/entry.js'; export default doubled",
                Some("https://example.test/workers/sw.js"),
            )
            .unwrap();
        assert!(result.namespace_json.contains("12"));
    }

    #[test]
    fn test_multiline_arrow_function_is_not_split() {
        let mut sandbox = EsModuleSandbox::new().unwrap();
        let result = sandbox
            .execute_module(
                "export const imported = 'module';
                 globalThis.onmessage = msg => {
                   globalThis.received = msg;
                 };",
                Some("https://example.test/module.js"),
            )
            .unwrap();
        assert!(result.namespace_json.contains("module"));
    }

    #[test]
    fn test_leading_comment_does_not_hide_static_import() {
        let mut registry = ModuleRegistry::new();
        registry.register("https://example.test/dependency.js", "export const value = 9;");
        let compiled = compile_module_script(
            "// changing response marker\nimport { value } from './dependency.js';\nglobalThis.value = value;",
            "https://example.test/sw.js",
            &registry,
        )
        .unwrap();
        assert!(!compiled.contains("import {"));
    }

    #[test]
    fn test_top_level_await_wraps_module_in_async_iife() {
        let registry = ModuleRegistry::new();
        let compiled = compile_module_script(
            "await Promise.resolve();\nglobalThis.ready = true;",
            "https://example.test/module.js",
            &registry,
        )
        .unwrap();
        assert!(compiled.starts_with("(async function() {"));
        assert!(compiled.contains("await Promise.resolve();"));
    }

    #[test]
    fn test_static_dependency_extraction_includes_reexports() {
        assert_eq!(
            extract_static_module_import_specifiers(
                "export { value as renamed } from './named.js';\
                 \nexport * from './star.js';\
                 \nexport * as namespace from './namespace.js';"
            ),
            ["./named.js", "./star.js", "./namespace.js"]
        );
    }

    #[test]
    fn test_reexports_resolve_canonical_urls() {
        let mut sandbox = EsModuleSandbox::new().unwrap();
        sandbox.register_module(
            "https://example.test/modules/dep.js",
            "export const value = 11; export default 99;",
        );
        let named = sandbox
            .execute_module(
                "export { value as renamed } from './dep.js';",
                Some("https://example.test/modules/entry.js"),
            )
            .unwrap();
        assert!(named.namespace_json.contains("\"renamed\":11"));

        let star = sandbox
            .execute_module(
                "export * from './dep.js';",
                Some("https://example.test/modules/entry.js"),
            )
            .unwrap();
        assert!(star.namespace_json.contains("\"value\":11"));
        assert!(!star.namespace_json.contains("\"default\""));

        let namespace = sandbox
            .execute_module(
                "export * as dependency from './dep.js';",
                Some("https://example.test/modules/entry.js"),
            )
            .unwrap();
        assert!(namespace.namespace_json.contains("\"dependency\":{"));
        assert!(namespace.namespace_json.contains("\"value\":11"));
        assert!(namespace.namespace_json.contains("\"default\":99"));
    }

    // R3398：循环 import（a↔b）旧实现无限递归 → 栈溢出 abort（实测 `has overflowed its stack`）。
    // 修复后须编译成功（不再无限递归），循环绑定解析为 undefined（转换式无 live binding，安全近似）。
    #[test]
    fn test_circular_import_no_overflow_r3398() {
        let mut reg = ModuleRegistry::new();
        reg.register("a", "import { b } from 'b'; export const a = b;");
        reg.register("b", "import { a } from 'a'; export const b = a;");
        // 修复前：栈溢出 abort（fatal runtime error）；修复后：编译成功返 Ok。
        let result = compile_module_script("import { a } from 'a';", "http://a/", &reg);
        assert!(result.is_ok(), "循环 import 须编译成功（不栈溢出）: {:?}", result.err());
        let script = result.unwrap();
        // 确认输出含占位空对象（已访问分支）而非无限内联。
        assert!(script.contains("(function(){return {};})()"), "已访问依赖须空对象占位");
    }

    // R3398：菱形 import（root→a, root→b, a→shared, b→shared）——shared 不应被递归内联两次。
    #[test]
    fn test_diamond_import_no_duplicate_inline_r3398() {
        let mut reg = ModuleRegistry::new();
        reg.register("shared", "export const shared = 42;");
        reg.register("a", "import { shared } from 'shared'; export const a = shared;");
        reg.register("b", "import { shared } from 'shared'; export const b = shared;");
        // 无循环，应正常编译（菱形 shared 经 visited 守卫不被重复递归致冗余，且不栈溢出）。
        let result = compile_module_script("import { a } from 'a';\nimport { b } from 'b';", "http://root/", &reg);
        assert!(result.is_ok(), "菱形 import 须编译成功: {:?}", result.err());
    }

    // R3398：默认导入旧实现把依赖 IIFE 字符串拼接 3 次 → 副作用执行 3 次。修复后求值一次。
    #[test]
    fn test_default_import_evaluated_once_r3398() {
        let mut reg = ModuleRegistry::new();
        // 依赖模块副作用：模块顶层表达式（计数）。若求值 3 次，输出会含 3 个相同语句。
        reg.register("dep", "export default 1; 2;");
        let script = compile_module_script("import d from 'dep';", "http://x/", &reg).unwrap();
        // 修复前默认导入分支把 dep IIFE 字面拼 3 次（出现 3 处 `(function(){...2;...})()`）；
        // 修复后求值一次存 `_dep_d` 变量。断言不再三重拼接：依赖模块体 "2;" 应只出现一次
        // 在 IIFE 内（+ 一次在变量赋值的引用，但那是变量名非字面 "2;"）。
        let count = script.matches("  2;").count();
        assert_eq!(
            count, 1,
            "默认导入依赖 IIFE 须只求值一次（'2;' 出现 1 次），got {count}：\n{script}"
        );
    }

    #[test]
    fn test_import_function_and_call() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./greet.js", "export function greet(name) { return 'Hello, ' + name }");
        let r = sb
            .execute_module(
                "import { greet } from './greet.js'\nexport default greet('World')",
                None,
            )
            .unwrap();
        assert!(r.namespace_json.contains("Hello, World"));
    }

    #[test]
    fn test_safe_ident() {
        // 折叠名后追加 FNV-1a 摘要后缀："/"与"_"折叠同像，摘要使 URL→标识符可注入
        //（recheck-b3 B3-1：碰撞会令菱形重入静默绑到错误实例）
        assert_eq!(safe_ident("./utils.js"), "utils_js_h4234e0a339b229f0");
        assert_eq!(
            safe_ident("https://example.com/mod.js"),
            "https___example_com_mod_js_h01cece26c5dc6d41"
        );
        assert_eq!(safe_ident("123"), "_123_h456fc2181822c4db");
        assert_eq!(safe_ident("abc"), "abc_he71fa2190541574b");
        assert_eq!(safe_ident("../a.js"), "a_js_h88d6723821338496");
        // 可注入性：折叠同像的不同 URL 摘要不同
        assert_ne!(safe_ident("a/b.js"), safe_ident("a_b.js"));
    }

    #[test]
    fn test_extract_string_literal() {
        assert_eq!(extract_string_literal("'hello'").unwrap(), "hello");
        assert_eq!(extract_string_literal("\"world\"").unwrap(), "world");
        assert!(extract_string_literal("naked").is_err());
    }

    #[test]
    fn test_extract_binding_name() {
        assert_eq!(extract_binding_name("x = 1"), "x");
        assert_eq!(extract_binding_name("foo() {}"), "foo");
        assert_eq!(extract_binding_name("Bar {}"), "Bar");
    }

    #[test]
    fn test_module_result_debug_clone() {
        let r = ModuleResult {
            namespace_json: "{\"x\":1}".into(),
            execution_time_ms: 0.5,
        };
        assert!(format!("{r:?}").contains("namespace_json"));
        let c = r.clone();
        assert_eq!(c.namespace_json, r.namespace_json);
    }

    #[test]
    fn test_registry_clone() {
        let mut reg = ModuleRegistry::new();
        reg.register("./a.js", "source");
        assert_eq!(reg.clone().get("./a.js"), Some("source"));
    }

    #[test]
    fn test_with_config() {
        let config = SandboxConfig {
            heap_limit: 16 * 1024 * 1024,
            timeout_ms: 5000,
            persistent_context: false,
            ..Default::default()
        };
        assert!(EsModuleSandbox::with_config(config).is_ok());
    }

    // ── 压缩形态（minifier 剥离关键字后的空格）──
    // github.com 真实模块形态（2026-10-08 T2 探索证据）：压缩 bundle 的 import/export
    // 语句没有关键字后空格（`import{a as b}from"./m.js"`、`export{n as x}`），语句
    // 识别须按 token 边界而非 `import `/`export ` 前缀。ECMA-262 §sec-imports、§sec-exports。

    #[test]
    fn test_minified_named_import_with_minified_dep_export() {
        // github.com 入口→wp-runtime 实际配对形态
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module(
            "https://assets.test/wp-runtime.js",
            "var n=function(){return {k:41}};\nexport{n as __webpack_require__};",
        );
        let r = sb
            .execute_module(
                "import{__webpack_require__ as i}from\"./wp-runtime.js\";export const v=i().k;",
                Some("https://assets.test/entry.js"),
            )
            .unwrap();
        assert!(r.namespace_json.contains("41"), "namespace={}", r.namespace_json);
    }

    #[test]
    fn test_minified_export_list_local() {
        // 压缩本地导出列表（无 from）：export{a,b as c}
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb.execute_module("var a=1;var b=2;export{a,b as c};", None).unwrap();
        assert!(
            r.namespace_json.contains("\"a\":1") && r.namespace_json.contains("\"c\":2"),
            "namespace={}",
            r.namespace_json
        );
    }

    #[test]
    fn test_minified_side_effect_import() {
        // 压缩副作用导入：import"./x.js"
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./side.js", "globalThis.__sideEffect = 7;");
        let r = sb
            .execute_module("import\"./side.js\";export const v=globalThis.__sideEffect;", None)
            .unwrap();
        assert!(r.namespace_json.contains("7"), "namespace={}", r.namespace_json);
    }

    #[test]
    fn test_minified_namespace_import() {
        // 压缩命名空间导入：import*as ns from"./m.js"
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./m.js", "export const k = 5;");
        let r = sb
            .execute_module("import*as ns from\"./m.js\";export const v=ns.k;", None)
            .unwrap();
        assert!(r.namespace_json.contains("5"), "namespace={}", r.namespace_json);
    }

    #[test]
    fn test_minified_default_import() {
        // 默认导入 from 后无空格：import d from"./m.js"
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./m.js", "export default {name:'zw'};");
        let r = sb
            .execute_module("import d from\"./m.js\";export const v=d.name;", None)
            .unwrap();
        assert!(r.namespace_json.contains("zw"), "namespace={}", r.namespace_json);
    }

    #[test]
    fn test_template_interpolation_with_nested_template_splitting() {
        // 模板字面量 ${} 插值内含代码与嵌套模板（github.com app-runtime 真实形态，
        // 2026-10-08 environment 图编译证据）：插值内的 `;`/引号/反引号不得破坏切分与引号跟踪。
        let src = "var u=`x${(()=>{let t=`a`;if(w){t=t.replace(RegExp(`(^|[?&])${a[0]}($|=)`,`g`))}return t}())}`||b;var y=1;";
        let stmts = split_statements(src);
        assert_eq!(stmts.len(), 2, "stmts={stmts:?}");
        assert!(stmts[0].starts_with("var u=`x${"), "stmts={stmts:?}");
        assert_eq!(stmts[1], "var y=1");
    }

    #[test]
    fn test_multiline_template_body_stays_one_statement() {
        // 模板正文含裸换行 + 插值内含字符串（github.com fetch-utilities 真实形态，
        // 2026-10-08 environment 图编译证据）：插值闭合后必须恢复模板上下文，
        // 否则正文换行被当作语句边界、插值内引号开启虚假字符串。
        let src = concat!(
            "var s=`HTTP error (${e.status}): ${n||\"No additional text\"}.\n",
            "Error Info: ${JSON.stringify(o)}`;var t=1;",
        );
        let stmts = split_statements(src);
        assert_eq!(stmts.len(), 2, "stmts={stmts:?}");
        assert!(stmts[0].contains("Error Info"), "stmts={stmts:?}");
        assert_eq!(stmts[1], "var t=1");
    }

    #[test]
    fn test_template_interpolation_string_with_backtick_splits_after() {
        // 插值内字符串含反引号（PR #114 审查 A2 变异验证样例）：旧实现（引号交替、
        // 无插值上下文机）把整条切成 1 条（尾句被吞），插值上下文机须切成 2 条。
        let src = "var s=`a${f('`')}b`;var t=1;";
        let stmts = split_statements(src);
        assert_eq!(stmts.len(), 2, "stmts={stmts:?}");
        assert!(stmts[0].starts_with("var s=`a${f('`')}b`"), "stmts={stmts:?}");
        assert_eq!(stmts[1], "var t=1");
    }

    #[test]
    fn test_template_interpolation_regex_and_comments_dont_leak_context() {
        // 插值内正则字面量与注释（PR #114 审查 F1 回归：TemplateBrace 分支
        // 原先不消费正则、不识别注释——正则内引号开启幽灵字符串上下文，
        // 模块其余部分的 import/export 不再被识别/改写；HTML 转义习语
        // `${t.replace(/[' "]/g,…)}` 实测触发）。尾部 import/export 必须仍被识别。
        let src = concat!(
            "var tag=`<a title=${t.replace(/[' \"]/g,String.fromCharCode(95))}>`;",
            "import{a}from\"./dep.js\";",
            "export const v=a;",
        );
        let stmts = split_statements(src);
        assert_eq!(stmts.len(), 3, "stmts={stmts:?}");
        assert!(stmts[0].contains("String.fromCharCode(95)"), "stmts={stmts:?}");
        assert!(stmts[1].starts_with("import{a}"), "stmts={stmts:?}");
        assert!(stmts[2].starts_with("export const"), "stmts={stmts:?}");
    }

    #[test]
    fn test_template_trailing_newline_before_close_stays_one_statement() {
        // 插值内部裸换行的上下文恢复守卫：`${r\n}`——裸换行位于插值内部，
        // 随后的 `}` 闭合插值、反引号闭合模板。整体是单条 var 声明
        // （f 与 x 同声明），插值内换行不得触发切分（2026-10-08 模板
        // 上下文机设计样例）。
        let src = "var f=(e,t)=>`${e}${t}=${r\n}`,x=1;";
        let stmts = split_statements(src);
        assert_eq!(stmts.len(), 1, "stmts={stmts:?}");
        assert!(stmts[0].contains("x=1"), "stmts={stmts:?}");
        assert!(stmts[0].contains("=${r"), "stmts={stmts:?}");
    }

    #[test]
    fn test_split_statements_keyword_after_whitespace_starts_new_prev_word() {
        // 空白后新标识符单独成词、不与 prev_word 拼接（sib-5，217fbc298）：压缩形态
        // `else return /re/` 下，旧实现把 "else"+"return" 跨空白拼成 "elsereturn"，
        // 非关键词 → `/` 误判除号 → 正则不原子消费，内部 `;` 把语句切碎。
        // 正则判定依据 https://tc39.es/ecma262/#prod-RegularExpressionLiteral。
        let src = "if(c)return 1;else return /a;b/.test(c);var done=1;";
        let stmts = split_statements(src);
        assert_eq!(stmts.len(), 3, "stmts={stmts:?}");
        assert_eq!(stmts[0], "if(c)return 1", "stmts={stmts:?}");
        assert_eq!(stmts[1], "else return /a;b/.test(c)", "stmts={stmts:?}");
        assert_eq!(stmts[2], "var done=1", "stmts={stmts:?}");
    }

    #[test]
    fn test_diamond_dep_shares_first_inline_instance() {
        // webpack 菱形（github.com 真实结构，2026-10-08 home-reload 证据）：entry→wp-runtime
        // （命名导入）+ entry→chunk（命名空间导入）+ chunk→wp-runtime 重入。重入须共享首份
        // 实例——旧空占位使 chunk 内 __webpack_require__ 为 undefined → `e.C(ns)` 报
        // "Cannot read properties of undefined (reading 'C')"。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module(
            "https://a.test/wp.js",
            "var reg={};var req=function(id){return reg[id]};req.C=function(ns){reg[ns.id]=ns};export{req as __webpack_require__};",
        );
        sb.register_module(
            "https://a.test/chunk.js",
            "import{__webpack_require__ as e}from\"./wp.js\";var ns={id:'c1',v:9};e.C(ns);export const marker=(typeof e.C==='function')?'ok':'broken';",
        );
        let r = sb
            .execute_module(
                "import{__webpack_require__ as e}from\"./wp.js\";import*as c from\"./chunk.js\";export const v=c.marker;",
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        assert!(r.namespace_json.contains("ok"), "namespace={}", r.namespace_json);
    }

    #[test]
    fn test_namespace_first_visit_binding_shares_instance_with_reentry() {
        // sib-3（5c79691d1）判别测试：命名空间导入作为 shared 模块**首访者**时，Fresh 臂
        // 须绑定 `var _mod_{safe} = IIFE`——后续重入经 exports_expr() 的 `typeof _mod_`
        // 守卫读到的才是首份实例。无首访绑定时守卫回落空对象（旧故障形态：重入读空，
        // marker=broken-empty），且全套既有测试无报警（2026-10-09 复核 TA1 突变实证）。
        // 同入口 namespace+具名双导入须解析到同一实例（_modref_ 守卫不遮蔽首访绑定）。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/shared.js", "export const v=7;");
        sb.register_module(
            "https://a.test/chunk.js",
            "import{v as vv}from\"./shared.js\";export const marker=(vv===7)?'ok':'broken-empty';",
        );
        let r = sb
            .execute_module(
                concat!(
                    "import*as s from\"./shared.js\";",
                    "import*as c from\"./chunk.js\";",
                    "import{v as w}from\"./shared.js\";",
                    "export const out=c.marker;",
                    "export const same=(s.v===w&&w===7)?'same':'split';",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        assert!(
            r.namespace_json.contains("\"out\":\"ok\""),
            "namespace={}",
            r.namespace_json
        );
        assert!(
            r.namespace_json.contains("\"same\":\"same\""),
            "namespace={}",
            r.namespace_json
        );
    }

    #[test]
    fn test_default_and_export_star_first_visit_bindings() {
        // sib-3 另两臂：default 导入与 export * from 作为 shared **首访者**时同样须在
        // 入口作用域绑定 `var _mod_{safe}`，供后续重入（含嵌套 chunk 内双臂）读取；
        // 缺绑定时首访自身可读（旧实现直发 IIFE 表达式），但重入守卫回落空对象——
        // 断言须逐键核对，不能只看 contains("ok")。深层菱形（首访发生在嵌套模块
        // IIFE 内部）作用域不外溢、守卫回落空对象，为 exports_expr 文档化边界，
        // 不在本测试断言范围。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/shared.js", "export const v=7;export default 42;");
        sb.register_module(
            "https://a.test/chunk.js",
            concat!(
                "import{v as vv}from\"./shared.js\";",
                "import dd from\"./shared.js\";",
                "export const m1=(vv===7)?'ok':'broken';",
                "export const m2=(dd===42)?'ok':'broken';",
            ),
        );
        let r = sb
            .execute_module(
                concat!(
                    "import d from\"./shared.js\";",
                    "export*from\"./shared.js\";",
                    "import*as s from\"./shared.js\";",
                    "import*as c from\"./chunk.js\";",
                    "export const e1=(d===42)?'ok':'broken';",
                    "export const e2=(s.v===7)?'ok':'broken';",
                    "export const x1=c.m1;",
                    "export const x2=c.m2;",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        let ns = &r.namespace_json;
        assert!(
            ns.contains("\"e1\":\"ok\"")
                && ns.contains("\"e2\":\"ok\"")
                && ns.contains("\"x1\":\"ok\"")
                && ns.contains("\"x2\":\"ok\""),
            "namespace={ns}",
        );
    }

    #[test]
    fn test_side_effect_import_first_visit_binding() {
        // sib-4（270613038）判别测试：副作用导入（`import "m"`）作为 shared **首访者**时，
        // Fresh 臂须绑定 `var _mod_{safe} = IIFE`——后续具名重入读到的才是同一实例
        // （旧实现只 visited.insert 不绑定，重入守卫回落空对象；共享可变状态可见分裂）。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/shared.js", "export const state={n:0};");
        sb.register_module(
            "https://a.test/chunk.js",
            concat!(
                "import{state}from\"./shared.js\";",
                "export const marker=(state&&state.n===5)?'ok':'split';",
            ),
        );
        let r = sb
            .execute_module(
                concat!(
                    "import\"./shared.js\";",
                    "import{state}from\"./shared.js\";",
                    "state.n=5;",
                    "import*as c from\"./chunk.js\";",
                    "export const out=c.marker;",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        assert!(
            r.namespace_json.contains("\"out\":\"ok\""),
            "namespace={}",
            r.namespace_json
        );
    }

    #[test]
    fn test_reexport_arms_first_visit_bindings() {
        // sib-4 另两臂：`export * as ns from` 与具名重导出 `export { v as w } from` 作为
        // shared **首访者**时同样须绑定 `var _mod_{safe}`，供后续具名重入读取；缺绑定时
        // 重导出副本自身可读（旧实现直发 IIFE 表达式），但重入守卫回落空对象——断言
        // 逐键核对。
        // 局部绑定差异（tc39 #sec-exports-static-semantics-importentries）：
        // `export * as ns` 产生 LocalName=ns 的 import 绑定，模块体内可裸引用；
        // `export { v as w2 } from` 不产生局部绑定，w2 只经导出可见。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/shared.js", "export const v=7;");
        let r = sb
            .execute_module(
                concat!(
                    "export*as ns from\"./shared.js\";",
                    "export{v as w2}from\"./shared.js\";",
                    "import{v as w}from\"./shared.js\";",
                    "export const o1=(ns.v===w&&w===7)?'ok':'split';",
                    "export const o2=w;",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        let nsj = &r.namespace_json;
        assert!(
            nsj.contains("\"o1\":\"ok\"") && nsj.contains("\"o2\":7"),
            "namespace={nsj}",
        );
        assert!(nsj.contains("\"w2\":7"), "具名重导出导出绑定缺失: namespace={nsj}",);
    }

    #[test]
    fn test_export_star_as_default_no_local_var() {
        // `export * as default from` 的名字是保留字，不能发 `var default = …`
        //（会整体语法错误）；导出绑定 default 仍须指向依赖命名空间对象。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/shared.js", "export const v=7;");
        let entry = concat!(
            "export*as default from\"./shared.js\";",
            "import*as self from\"./entry.js\";",
            "export const o=(self.default&&self.default.v===7)?'ok':'split';",
        );
        // collect_module_deps 会把入口自身注册进 registry（js_worker.rs）——镜像该行为
        sb.register_module("https://a.test/entry.js", entry);
        let r = sb.execute_module(entry, Some("https://a.test/entry.js")).unwrap();
        let nsj = &r.namespace_json;
        assert!(
            nsj.contains("\"o\":\"ok\"") && nsj.contains("\"default\""),
            "namespace={nsj}",
        );
    }

    #[test]
    fn test_safe_ident_collision_binds_distinct_instances() {
        // safe_ident 折叠不可逆：a/b.js 与 a_b.js 折叠出同一标识符。菱形重入改读
        // var _mod_{safe} 后碰撞会静默绑定到错误实例（recheck-b3 B3-1：m1b 读到
        // M2 的导出）——摘要后缀使映射可注入后，重入须各自解析到正确实例。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/a/b.js", "export const who='M1';");
        sb.register_module("https://a.test/a_b.js", "export const who='M2';");
        let r = sb
            .execute_module(
                concat!(
                    "import*as m1 from\"./a/b.js\";",
                    "import*as m2 from\"./a_b.js\";",
                    "import*as m1b from\"./a/b.js\";",
                    "export const w1=m1.who;",
                    "export const w2=m2.who;",
                    "export const w1b=m1b.who;",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        let nsj = &r.namespace_json;
        assert!(
            nsj.contains("\"w1\":\"M1\"") && nsj.contains("\"w2\":\"M2\"") && nsj.contains("\"w1b\":\"M1\""),
            "碰撞实例串扰: namespace={nsj}",
        );
    }

    #[test]
    fn test_export_star_as_reserved_word_no_local_var() {
        // `export * as class from` 是合法 ESM（ModuleExportName 允许保留字），只是
        // 不能发 var class 局部声明（recheck-b3 B3-2）——语句须可执行且导出绑定在。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/shared.js", "export const v=7;");
        let r = sb
            .execute_module(
                concat!("export*as class from\"./shared.js\";", "export const o='ok';",),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        let nsj = &r.namespace_json;
        assert!(
            nsj.contains("\"o\":\"ok\"") && nsj.contains("\"class\""),
            "namespace={nsj}",
        );
    }

    #[test]
    fn test_export_star_as_eval_arguments_no_local_var() {
        // 包裹体总在 'use strict' 下运行：`var eval`/`var arguments` 是 strict 早
        // 错误（SyntaxError），与保留字同类（sib-6，c986462ad）——`export * as
        // eval/arguments from` 须跳过局部声明，仅保留导出绑定。
        // https://tc39.es/ecma262/#sec-identifiers-static-semantics-early-errors
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/shared.js", "export const v=7;");
        let r = sb
            .execute_module(
                concat!(
                    "export*as eval from\"./shared.js\";",
                    "export*as arguments from\"./shared.js\";",
                    "export const o='ok';",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        let nsj = &r.namespace_json;
        assert!(
            nsj.contains("\"o\":\"ok\"") && nsj.contains("\"eval\"") && nsj.contains("\"arguments\""),
            "namespace={nsj}",
        );
    }

    #[test]
    fn test_entry_self_import_shares_own_exports() {
        // 入口自导入（github.com environment 入口真实结构，2026-10-08 home-reload 证据：
        // `import*as i from"./environment-*.js";e.C(i)`）：自导入不得再内联一份入口体
        // （内层拷贝执行期守卫 typeof 落空 → 空 stub → `e.C({})` 报 reading 'length'），
        // 须解析到入口自身的导出。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module(
            "https://a.test/wp.js",
            // C 读 ns.__rspack_esm_ids.length——与 github wp-runtime 真实语义一致（stub 命名空间必须抛错）
            "var reg={};var req=function(id){return reg[id]};req.C=function(ns){var ids=ns.__rspack_esm_ids;for(var n=0;n<ids.length;n++)reg[ids[n]]=ns};export{req as __webpack_require__};",
        );
        // 真实入口语句序：export 声明在前（environment-*.js 实测位置 101），C 调用在后
        let entry_src = concat!(
            "export const __rspack_esm_ids=['e1'];export const tag='root';",
            "import{__webpack_require__ as e}from\"./wp.js\";",
            "import*as self from\"./entry.js\";",
            "e.C(self);",
        );
        // collect_module_deps 会把入口自身注册进 registry（js_worker.rs）——镜像该行为
        sb.register_module("https://a.test/entry.js", entry_src);
        let r = sb.execute_module(entry_src, Some("https://a.test/entry.js")).unwrap();
        assert!(r.namespace_json.contains("root"), "namespace={}", r.namespace_json);
    }

    #[test]
    fn test_export_declaration_body_rewrites_import_meta_and_dynamic_import() {
        // github.com react-core chunk 真实形态（2026-10-09 t2g 轮 "Loading chunk cmi
        // failed after 3 retries" 证据，275034 字节单条 export const 语句）：声明体内含
        // import.meta（react-router 懒加载 catch 分支 `isSpaMode&&import.meta.hot`）与
        // 动态 import()（`let r=await import(e.module)`）。transform_export 声明路径重发
        // 前未同步重写 → IIFE 经典脚本执行抛 SyntaxError: Cannot use 'import.meta'
        // outside a module → chunk 三次重试全败。
        let mut sb = EsModuleSandbox::new().unwrap();
        let src = concat!(
            "export const mods={",
            "seen:null,",
            "load(e){try{return import(e.m)}catch(t){if(import.meta.hot)throw t;return null}}",
            "};",
        );
        // 编译产物断言：声明体内动态 import() 须重写为宿主桥接调用，import.meta 须消解
        // （残留即经典脚本 IIFE SyntaxError）。
        let compiled = compile_module_script(src, "zero://module", &ModuleRegistry::new()).unwrap();
        assert!(
            compiled.contains("__zw_dynamic_import"),
            "dynamic import not rewritten: {}",
            &compiled[..compiled.len().min(400)]
        );
        assert!(!compiled.contains("import.meta"), "import.meta survived");
        // 执行断言（不含动态 import 的同型声明，避开 async 包装使 namespace 可序列化）：
        // import.meta 残留时此处直接 SyntaxError。
        let src2 = "export const mods={seen:null,load(e){if(import.meta.hot)throw e;return null}};";
        let r = sb.execute_module(src2, None).unwrap();
        // JSON 序列化省略函数值（load 是方法），断言属性键即可
        assert!(r.namespace_json.contains("\"mods\""), "namespace={}", r.namespace_json);
    }

    #[test]
    fn test_minified_star_reexport() {
        // 压缩星号再导出：export*from"./m.js"
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("./m.js", "export const k = 3;");
        let r = sb.execute_module("export*from\"./m.js\";", None).unwrap();
        assert!(r.namespace_json.contains("3"), "namespace={}", r.namespace_json);
    }

    #[test]
    fn test_extract_static_specifiers_minified() {
        let src = "import{__webpack_require__ as i}from\"./wp.js\";import\"./side.js\";var x=1;import y from\"./d.js\"";
        assert_eq!(
            extract_static_module_import_specifiers(src),
            vec!["./wp.js", "./side.js", "./d.js"]
        );
    }

    #[test]
    fn test_non_module_statements_not_misdetected() {
        // 不得误伤：动态 import( 表达式、import.meta 元属性、含 import 前缀的标识符
        let src = "const x = import.meta.url; important(); var y = import('./dyn.js');";
        assert!(extract_static_module_import_specifiers(src).is_empty());
    }

    #[test]
    fn test_regex_literal_with_quotes_survives_statement_splitting() {
        // github.com behaviors 模块真实形态（2026-10-08 T2 home-reload 证据）：压缩代码的
        // 正则字面量内含引号/分号（/[\s,']+/、/"/g），语句切分的引号跟踪不得进入正则。
        // 修复前：正则内 `"` 被当字符串边界 → 跨语句吞切分点 → V8 "Invalid regular
        // expression: missing /"。
        let mut sb = EsModuleSandbox::new().unwrap();
        let r = sb
            .execute_module(
                "export const f=(t)=>t.split(/[\\s,']+/);export const e=(s)=>s.replace(/\"/g,\"&quot;\").replace(/'/g,\"&#39;\");export const v=f(\"a,b\").length;",
                None,
            )
            .unwrap();
        assert!(r.namespace_json.contains("2"), "namespace={}", r.namespace_json);
    }

    #[test]
    fn test_regex_vs_division_splitting() {
        // 除号不得误判为正则（数字/标识符/`)` 后的 `/`），语句仍按 `;` 正确切分
        let src = "var a=6/2/1;var b=10;\nvar c=x.y/2;";
        let stmts = split_statements(src);
        assert_eq!(stmts, vec!["var a=6/2/1", "var b=10", "var c=x.y/2"]);
    }

    #[test]
    fn test_regex_after_keyword_starts_regex() {
        // 关键字后 `/` 是正则：return /re/.test(t)（块语句以 ; 结尾——压缩码恒有）
        let src = "function f(t){return /['\"]/g.test(t)};var x=1;";
        let stmts = split_statements(src);
        assert_eq!(stmts.len(), 2, "stmts={stmts:?}");
        assert!(stmts[0].contains("/['\"]/g"), "stmts={stmts:?}");
    }
}
