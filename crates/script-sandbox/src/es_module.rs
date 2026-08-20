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

use crate::{SandboxConfig, ScriptError};
use std::collections::{HashMap, HashSet};

/// 模块注册表 — 存储已注册的 ES Module 源代码。
#[derive(Debug, Clone, Default)]
pub struct ModuleRegistry {
    modules: HashMap<String, String>,
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
pub struct EsModuleSandbox {
    /// 模块注册表。
    registry: ModuleRegistry,
    /// V8 沙箱（用于执行转换后的代码）。
    sandbox: Box<dyn crate::Sandbox>,
}

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
    let body = build_module_script(source, url, registry, &mut HashSet::new())?;
    if source.contains("import(") {
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
    for spec in registry.specifiers() {
        let iife = build_dep_iife(spec, registry, &mut HashSet::new())?;
        let escaped = spec.replace('\\', "\\\\").replace('\'', "\\'");
        out.push_str(&format!("__moduleCache['{escaped}'] = {iife};\n"));
    }
    out.push_str("globalThis.__zw_load_module = function(spec) {\n");
    out.push_str("  if (__moduleCache[spec]) return __moduleCache[spec];\n");
    out.push_str(
        "  var parent = (typeof _importMeta !== 'undefined' && _importMeta.url) ? _importMeta.url : 'about:blank';\n",
    );
    out.push_str("  var code = __zw_compile_module(spec, parent);\n");
    out.push_str("  if (!code) throw new Error('Module not found: ' + spec);\n");
    out.push_str("  __moduleCache[spec] = (function() { return eval('(' + code + ')'); })();\n");
    out.push_str("  return __moduleCache[spec];\n");
    out.push_str("};\n");
    out.push_str(
        "globalThis.__zw_dynamic_import = function(spec) { return Promise.resolve(__zw_load_module(spec)); };\n",
    );
    Ok(out)
}

fn rewrite_dynamic_imports(source: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < source.len() {
        if source[i..].starts_with("import(") {
            out.push_str("__zw_dynamic_import(");
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

/// 仅提取**静态** `import` 依赖标识符（不含 `import()` 动态导入）。
/// 供动态 import() 运行时 fetch 路径（R3093）：预注册空存根只用静态 import（headless 单遍，transitive defer），
/// 动态 import() 留给运行时 `__zw_load_module → __zw_compile_module` fetch——避免预存根（empty namespace）
/// 短路运行时 fetch。无 fetcher 路径仍用 `extract_module_import_specifiers`（动态 import 预存根返空 namespace）。
pub fn extract_static_module_import_specifiers(source: &str) -> Vec<String> {
    let mut specs = Vec::new();
    for stmt in split_statements(source) {
        let trimmed = stmt.trim();
        let specifier = if trimmed.starts_with("import ") {
            extract_import_specifier(trimmed)
        } else if trimmed.starts_with("export ") {
            extract_reexport_specifier(trimmed)
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

fn extract_import_specifier(line: &str) -> Result<String, ScriptError> {
    let rest = &line["import ".len()..];
    if rest.starts_with('\'') || rest.starts_with('"') || rest.starts_with('`') {
        return extract_string_literal(rest.split(';').next().unwrap_or(rest).trim());
    }
    if let Some(from_pos) = rest.find(" from ") {
        return extract_import_specifier_from_rest(&rest[from_pos + 6..]);
    }
    Err(ScriptError::CompileError(format!("unsupported import: {line}")))
}

fn extract_reexport_specifier(line: &str) -> Result<String, ScriptError> {
    let rest = line
        .strip_prefix("export ")
        .ok_or_else(|| ScriptError::CompileError(format!("unsupported re-export: {line}")))?;
    let from_pos = rest
        .find(" from ")
        .ok_or_else(|| ScriptError::CompileError(format!("unsupported re-export: {line}")))?;
    extract_import_specifier_from_rest(&rest[from_pos + 6..])
}

/// 构建完整的模块执行脚本，内联所有依赖。
fn build_module_script(
    source: &str,
    url: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> Result<String, ScriptError> {
    let mut output = String::with_capacity(source.len() * 3);
    output.push_str("(function() {\n");
    output.push_str("  'use strict';\n");
    output.push_str("  var _exports = {};\n");
    output.push_str(&format!("  var _importMeta = {{ url: {} }};\n", json_stringify(url)));

    // 处理当前模块的每个语句
    let stmts = split_statements(source);
    for stmt in &stmts {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("import ") {
            output.push_str(&transform_import(trimmed, url, registry, visited)?);
        } else if trimmed.starts_with("export ") {
            output.push_str(&transform_export(trimmed, url, registry, visited)?);
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

    let stmts = split_statements(source);
    for stmt in &stmts {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("import ") {
            output.push_str(&transform_import(trimmed, specifier, registry, visited)?);
        } else if trimmed.starts_with("export ") {
            output.push_str(&transform_export(trimmed, specifier, registry, visited)?);
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
fn inline_dep_once(
    specifier: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> Result<String, ScriptError> {
    if !visited.contains(specifier) {
        visited.insert(specifier.to_string());
        return build_dep_iife(specifier, registry, visited);
    }
    // 已访问（循环/菱形）→ 空对象占位，不递归。
    Ok("(function(){return {};})()".to_string())
}

/// 转换 import 声明。
fn transform_import(
    line: &str,
    importer_url: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> Result<String, ScriptError> {
    let rest = &line["import ".len()..];

    // import 'module' — 副作用导入
    if rest.starts_with('\'') || rest.starts_with('"') || rest.starts_with('`') {
        let raw_specifier = extract_string_literal(rest.split(';').next().unwrap_or(rest).trim())?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        // 执行副作用（内联执行模块体但不使用返回值）
        if !visited.contains(&specifier) {
            visited.insert(specifier.clone());
            let dep_code = build_dep_iife(&specifier, registry, visited)?;
            return Ok(format!("  {dep_code};\n"));
        }
        return Ok(String::new());
    }

    // import * as X from 'module'
    if let Some(as_pos) = rest.find("* as ")
        && let Some(from_pos) = rest.find(" from ")
    {
        let ns_name = rest[as_pos + 5..from_pos].trim();
        let raw_specifier = extract_import_specifier_from_rest(&rest[from_pos + 6..])?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        // R3398：防循环/菱形 import 无限递归（仅首次访问时内联依赖 IIFE；已访问 → 空对象占位，
        // 避免 a↔b 循环致栈溢出 abort）。镜像 import 'm' 副作用导入的 visited 守卫（line 378）。
        let dep_code = inline_dep_once(&specifier, registry, visited)?;
        return Ok(format!("  var {ns_name} = {dep_code};\n"));
    }

    // import { X, Y as Z } from 'module'
    if rest.starts_with('{')
        && let Some(from_pos) = rest.find(" from ")
    {
        let bindings = rest[..from_pos].trim();
        let raw_specifier = extract_import_specifier_from_rest(&rest[from_pos + 6..])?;
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
        let dep_code = inline_dep_once(&specifier, registry, visited)?;
        let mut result = format!("  var _mod_{safe} = {dep_code};\n");
        result.push_str(&destructure_bindings(bindings, &safe));
        return Ok(result);
    }

    // import X from 'module' — 默认导入
    if let Some(from_pos) = rest.find(" from ") {
        let name = rest[..from_pos].trim();
        let raw_specifier = extract_import_specifier_from_rest(&rest[from_pos + 6..])?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        ensure_module_export(&specifier, "default", registry)?;
        let dep_code = inline_dep_once(&specifier, registry, visited)?;
        return Ok(format!("  var {name} = {dep_code}.default;\n"));
    }

    Ok(String::new())
}

fn resolve_registered_specifier(specifier: &str, importer_url: &str, registry: &ModuleRegistry) -> String {
    if registry.get(specifier).is_some() {
        return specifier.to_string();
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
        let Some(rest) = statement.trim().strip_prefix("export ") else {
            continue;
        };
        if name == "default" && rest.starts_with("default ") {
            return true;
        }
        for declaration in ["const ", "let ", "var ", "function ", "class "] {
            if let Some(value) = rest.strip_prefix(declaration)
                && extract_binding_name(value) == name
            {
                return true;
            }
        }
        if rest.starts_with("* as ")
            && let Some((namespace, _)) = rest["* as ".len()..].split_once(" from ")
            && namespace.trim() == name
        {
            return true;
        }
        if rest.starts_with('{')
            && let Some(end) = rest.find('}')
        {
            let from_specifier = rest[end + 1..]
                .trim()
                .strip_prefix("from ")
                .and_then(|value| extract_import_specifier_from_rest(value).ok())
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
        if name != "default"
            && let Some(from_rest) = rest.strip_prefix("* from ")
            && let Ok(raw) = extract_import_specifier_from_rest(from_rest)
        {
            let dependency = resolve_registered_specifier(&raw, specifier, registry);
            if module_provides_export(&dependency, name, registry, visited) {
                return true;
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

/// 生成解构导入语句。
fn destructure_bindings(bindings: &str, safe_mod: &str) -> String {
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
            result.push_str(&format!("  var {alias} = _mod_{safe_mod}.{src};\n"));
        } else {
            result.push_str(&format!("  var {item} = _mod_{safe_mod}.{item};\n"));
        }
    }
    result
}

/// 转换 export 声明。
fn transform_export(
    line: &str,
    importer_url: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> Result<String, ScriptError> {
    let rest = &line["export ".len()..];

    if rest.starts_with("* as ")
        && let Some(from_pos) = rest.find(" from ")
    {
        let namespace = rest["* as ".len()..from_pos].trim();
        let raw_specifier = extract_import_specifier_from_rest(&rest[from_pos + 6..])?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        let dep_code = inline_dep_once(&specifier, registry, visited)?;
        return Ok(format!("  _exports.{namespace} = {dep_code};\n"));
    }
    if let Some(from_rest) = rest.strip_prefix("* from ") {
        let raw_specifier = extract_import_specifier_from_rest(from_rest)?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        let safe = safe_ident(&specifier);
        let dep_code = inline_dep_once(&specifier, registry, visited)?;
        return Ok(format!(
            "  var _reexport_{safe} = {dep_code};\n  Object.keys(_reexport_{safe}).forEach(function(key) {{ if (key !== 'default') _exports[key] = _reexport_{safe}[key]; }});\n"
        ));
    }
    if rest.starts_with('{')
        && let Some(from_pos) = rest.find(" from ")
    {
        let end = rest[..from_pos]
            .find('}')
            .ok_or_else(|| ScriptError::CompileError("invalid re-export list: missing }".into()))?;
        let raw_specifier = extract_import_specifier_from_rest(&rest[from_pos + 6..])?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry);
        let safe = safe_ident(&specifier);
        for item in rest[1..end].split(',').map(str::trim).filter(|item| !item.is_empty()) {
            let imported = item.split_once(" as ").map_or(item, |(name, _)| name).trim();
            ensure_module_export(&specifier, imported, registry)?;
        }
        let dep_code = inline_dep_once(&specifier, registry, visited)?;
        let mut result = format!("  var _reexport_{safe} = {dep_code};\n");
        for item in rest[1..end].split(',').map(str::trim).filter(|item| !item.is_empty()) {
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
    if let Some(expr) = rest.strip_prefix("default ") {
        let expr = expr.replace("import.meta", "_importMeta");
        return Ok(format!("  _exports.default = {expr};\n"));
    }
    if let Some(decl) = rest.strip_prefix("const ") {
        let name = extract_binding_name(decl);
        return Ok(format!("  const {decl};\n  _exports.{name} = {name};\n"));
    }
    if let Some(decl) = rest.strip_prefix("let ") {
        let name = extract_binding_name(decl);
        return Ok(format!("  let {decl};\n  _exports.{name} = {name};\n"));
    }
    if let Some(decl) = rest.strip_prefix("var ") {
        let name = extract_binding_name(decl);
        return Ok(format!("  var {decl};\n  _exports.{name} = {name};\n"));
    }
    if let Some(decl) = rest.strip_prefix("function ") {
        let name = extract_binding_name(decl);
        return Ok(format!("  function {decl}\n  _exports.{name} = {name};\n"));
    }
    if let Some(decl) = rest.strip_prefix("class ") {
        let name = extract_binding_name(decl);
        return Ok(format!("  class {decl}\n  _exports.{name} = {name};\n"));
    }

    // export { X, Y as Z }
    if rest.starts_with('{') {
        let end = rest
            .find('}')
            .ok_or_else(|| ScriptError::CompileError("invalid export list: missing }".into()))?;
        let list_str = &rest[1..end];
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

    Ok(format!("  {line};\n"))
}

// ── 辅助函数 ──

/// Split top-level module statements without breaking multiline function or arrow bodies.
fn split_statements(source: &str) -> Vec<String> {
    let mut stmts = Vec::new();
    let mut current = String::new();
    let mut braces = 0usize;
    let mut parens = 0usize;
    let mut brackets = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;
    let chars = source.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        let ch = chars[index];
        let next = chars.get(index + 1).copied();
        if line_comment {
            if ch == '\n' {
                line_comment = false;
                if braces == 0 && parens == 0 && brackets == 0 {
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
        if let Some(delimiter) = quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == delimiter {
                quote = None;
            }
            index += 1;
            continue;
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
        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '{' => braces += 1,
            '}' => braces = braces.saturating_sub(1),
            '(' => parens += 1,
            ')' => parens = parens.saturating_sub(1),
            '[' => brackets += 1,
            ']' => brackets = brackets.saturating_sub(1),
            _ => {}
        }
        let top_level = braces == 0 && parens == 0 && brackets == 0;
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
    let mut safe = String::new();
    for c in specifier.chars() {
        if c.is_alphanumeric() || c == '_' {
            safe.push(c);
        } else {
            safe.push('_');
        }
    }
    let safe = safe.trim_matches('_');
    if safe.is_empty() {
        return "_mod".to_string();
    }
    if safe.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
        format!("_{safe}")
    } else {
        safe.to_string()
    }
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
        assert_eq!(safe_ident("./utils.js"), "utils_js");
        assert_eq!(safe_ident("https://example.com/mod.js"), "https___example_com_mod_js");
        assert_eq!(safe_ident("123"), "_123");
        assert_eq!(safe_ident("abc"), "abc");
        assert_eq!(safe_ident("../a.js"), "a_js");
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
}
