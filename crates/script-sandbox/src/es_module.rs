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

        let transformed = compile_module_script(source, url, url, &self.registry)?;

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

/// 文档级模块缓存复位脚本——宿主在**每文档脚本阶段起点**执行一次，使模块缓存按文档
/// 隔离（HTML navigate：跨文档导航创建新 global object 与新 module map，同 URL 模块在
/// 新文档重新求值；https://html.spec.whatwg.org/multipage/browsing-the-web.html#navigate）。
///
/// R5182-D1 三宿主复位面：renderer 路径经 `ResetDocumentState` 臂 `reset_context` 销毁
/// 重建全局对象（缓存随之消失，无须本脚本）；tab worker 路径（`tab_scripts::script_phase_begin`）
/// 与 webview 进程内路径（`run_page_scripts_impl`，无 reset_context）持持久 context，
/// 必须显式执行本脚本——不复位则同 URL 模块在新文档零次求值（D1 回归）。
pub const MODULE_CACHE_RESET_SCRIPT: &str = "globalThis.__moduleCache = {};\nglobalThis.__moduleFailures = {};\nglobalThis.__zwModuleFetchKeys = {};\nglobalThis.__zwDeferRecords = {};\nglobalThis.__zwDeferFetchKeys = {};\n";

/// s339（iframe 域动态 import 模块体 window 身份桥）——模块体求值包裹片（open/close
/// 夹住模块体源码）。两处求值臂（prelude eval 臂与 v8_runtime native settle 臂）在
/// **构建期**判定全局 `__zwActiveIframeWin` 桥位：在场才夹包裹，缺席按原样求值
/// （worker/裸 sandbox 等无 window 全局面零漂移——无条件形参遮蔽会把裸 `window`
/// 从 ReferenceError 变为 globalThis，worker module 语义面不可接受）。
/// 桥位由引擎 shim 在子文档 harness 段后注入的注册包装置位（js_dom_shim
/// `_zwRunIframeScripts` 注入段——test/async_test/promise_test 体执行窗 try/finally
/// 换入换出）。native settle 臂同步发生于 import() 求值窗内（classic 直调无跳代）
/// ——构建期 = 桥位窗 ✓；prelude eval 臂经 s337 双跳，构建落在体返回后的微任务
/// （桥位已清）→ 恒走原样面——桥位经该臂仅同步直调形态可达（语义保守，corpus
/// 子文档面全走 classic native 臂）。在场包裹语义：模块体裸 `window`/`document`/
/// `self` 形参遮蔽到 importing document 的 win/doc——本引擎单 isolate 共享全局，
/// 子文档 realm 以合并 Function 形参承载，模块体求值不落形参作用域，window 写
/// 此前错落主窗（s338 定谳根因链）。spec：模块的 settings object/realm 归属
/// importing document。
/// 内容约束：不得含单引号/反斜杠（prelude 以 `'…'` JS 字符串字面量内嵌）。
/// https://html.spec.whatwg.org/multipage/webappapis.html#module-scripts
pub const MODULE_EVAL_WINDOW_SCOPE_OPEN: &str =
    "function(){var __zwW=globalThis.__zwActiveIframeWin;return (function(window,document,self){return ";
/// s339 桥包裹闭合片（形参遮蔽调用 + 外层 IIFE 立即执行；见 [`MODULE_EVAL_WINDOW_SCOPE_OPEN`]）。
pub const MODULE_EVAL_WINDOW_SCOPE_CLOSE: &str =
    ";}).call(__zwW,__zwW,globalThis.__zwActiveIframeDoc||__zwW.document||__zwW,__zwW);}()";

/// 把依赖 IIFE 包成 `__moduleCache` 记忆化赋值表达式——模块体求值恰好一次（HTML module
/// map 命中即复用，https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script）。
/// R5182：prelude 急切求值、入口体内联、后续 module script 元素三条发射路径共享同一缓存实例；
/// 调用方保证 `globalThis.__moduleCache` 已初始化（`compile_module_script` 输出与 prelude 的
/// 首行 init 均已就位后才轮到本表达式求值）。命中判定用 `=== undefined`（contains 语义，
/// TV-4：不依赖「导出对象恒真值」这一未强制不变量）。
/// s344：求值失败记忆——体抛错时原值记入 `__moduleFailures[key]` 后重抛（传播语义不变）；
/// 后续同键求值复抛**同对象**不重求值（spec：parse error 与 evaluation error 均缓存在
/// module map，跨 script 元素/动态 import 同源——WPT choice-of-error-3 判据
/// `log[1] === log[3]`、evaluation-error-1 判据每元素一 error 事件共 9 元素）。入口复放
/// 路径经宿主包装 catch 置 s255 哨兵 → post-check 原值上报 window error 事件 + 元素
/// fire load（体尾 load 投递因抛错跳过，无双发）。
/// s344-r2（中间模块错误记账）：`ancestors` 为该依赖在入口图 DFS 路径上的祖先键
/// （[`module_graph_postorder`] 逐模块携带）——失败同时记入各祖先键（未记录时），
/// 即 spec 中间模块记录同样持有该求值错误：choice-of-error-3 入口 3a 的图
/// [throw2?3d←3b←3a] 首元素在 throw2?3d 抛 `{bar}` 中止 prelude，3b 自身发射未达，
/// 旧版只记 throw2?3d 键 → 第二元素（入口 3b）短路面落空、prelude 重跑图外模块
/// throw?3c（log 翻倍 6≠5）；祖先记账后第二元素入口键命中 → 宿主 s344-r2 预检
/// 复抛同对象，`log[1] === log[3]` 同源。
/// https://html.spec.whatwg.org/multipage/webappapis.html#run-a-module-script
fn memoized_dep_expr(key: &str, code: &str, ancestors: &[String]) -> String {
    let key_js = json_stringify(key);
    let ancestors_js = ancestors
        .iter()
        .map(|a| json_stringify(a))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "(globalThis.__moduleFailures[{key_js}] !== undefined ? (function(){{ throw globalThis.__moduleFailures[{key_js}]; }})() : (globalThis.__moduleCache[{key_js}] = globalThis.__moduleCache[{key_js}] === undefined ? (function(){{ try {{ return ({code}); }} catch (__zwModMemoErr) {{ globalThis.__moduleFailures[{key_js}] = __zwModMemoErr; for (var __zwModAncIdx = 0; __zwModAncIdx < [{ancestors_js}].length; __zwModAncIdx++) {{ var __zwModAncKey = [{ancestors_js}][__zwModAncIdx]; if (globalThis.__moduleFailures[__zwModAncKey] === undefined) {{ globalThis.__moduleFailures[__zwModAncKey] = __zwModMemoErr; }} }} throw __zwModMemoErr; }} }})() : globalThis.__moduleCache[{key_js}]))"
    )
}

/// 将 ES Module 源码编译为可在 V8 中执行的 IIFE 脚本（内联依赖）。
///
/// `url` 为模块自身 URL（相对 import 解析基准 + `import.meta.url` 发射值）；
/// `cache_key` 为 `__moduleCache` 文档级记忆化键——两者分离：同文档多个 module script
/// 元素共享缓存（module map 语义），但内联 module script 每元素是独立模块记录、键不得
/// 落在页面 URL 上（否则第二枚被首枚记忆化命中短路，模块体零次求值——s94 附带修复，
/// currentScript-null 双内联 module 定谳）。外链 module 以 src 绝对 URL 为键（同 src
/// 两元素恰好一次，spec 语义）。
/// https://html.spec.whatwg.org/multipage/webappapis.html#module-map
pub fn compile_module_script(
    source: &str,
    url: &str,
    cache_key: &str,
    registry: &ModuleRegistry,
) -> Result<String, ScriptError> {
    compile_module_script_with_credentials(source, url, cache_key, registry, "same-origin")
}

/// [`compile_module_script`] 的 credentials 变体（s338——module 元素 `crossorigin`
/// 属性决定其模块脚本的 credentials mode，动态 `import()` 取回沿用该 mode：
/// `use-credentials` → `include`，其余（缺失/anonymous/非法值按 spec invalid
/// value default = anonymous）→ `same-origin`）。经重写发射的 IIFE 局部
/// `_importCred` 穿透到 `__zw_dynamic_import`（spec fetch a single module script
/// 的 credentials mode 步骤；动态 import 的请求 credentials = 定义其的模块脚本
/// 元素 credentials mode）。
/// https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
pub fn compile_module_script_with_credentials(
    source: &str,
    url: &str,
    cache_key: &str,
    registry: &ModuleRegistry,
    credentials: &str,
) -> Result<String, ScriptError> {
    let uses_await = source.contains("await");
    // s269：async 包裹判定与消费上下文（依赖 await 发射）同源——`import(`（动态
    // import 重写后的 helper 调用态）也触发 async 包裹（prelude 契约既有行为），此时
    // TLA 依赖的消费点同样可合法 await。s361：静态 defer import 声明同传染（判定
    // 理由见 build_dep_iife 同名注记）。
    let async_body = source.contains("import(") || uses_await || source_has_deferred_import_decl(source);
    let mut path = vec![url.to_string()];
    let body = build_module_script(
        source,
        url,
        registry,
        async_body,
        credentials,
        &mut HashSet::new(),
        &mut path,
    )?;
    // R5182：首行初始化文档级模块缓存（幂等）——独立调用路径（EsModuleSandbox、SW 编译）
    // 不经过 prelude 也能安全使用记忆化发射点。
    let init = "globalThis.__moduleCache = globalThis.__moduleCache || {};\nglobalThis.__moduleFailures = globalThis.__moduleFailures || {};\n";
    if async_body {
        // FIXME(R5182)：async 入口体暂不记忆化——缓存实例会成 Promise（依赖读取方期望导出
        // 对象）。共享 async 模块的跨元素恰好一次留待后续切片（活体证据：MDN 入口链均同步）。
        // s269：TLA 依赖的缓存实例即 Promise（build_dep_iife async 包裹），消费方 await
        // 归一（await_prefix）——缓存 Promise 与缓存导出对象两态消费面自洽。
        // s337（TLA rejection 上报面）：async 入口的 rejection 是模块求值失败的唯一
        // 错误通路——await 掷与体同步 throw 全走 promise 拒绝，宿主包装 try/catch
        // （webview s255 面）接不到。spec run-a-module-script 求值 rejection →
        // report the exception（window error 事件，ev.error 为原值拒绝理由）。
        // build_module_script 已发射完整 async IIFE（尾 `})()`），不得再包一层：
        // 双层包裹令内层 promise 沦为语句求值即弃，外层以 undefined resolve，
        // rejection 永不达 catch（红态实测序对而 reportError 缺席）。catch 桥直附
        // 调用尾：非页面宿主无 shim reportError → 守卫空转。不设 __zwModEvalErrObj
        // 哨兵：哨兵由宿主 post-check 同步读清（s255），异步置位会陈旧滞留并被下一
        // 模块元素误报。元素 load 不补发（runner 投递尾微任务 fire / webview s107
        // 成功臂已覆盖，补发即双发）。
        // https://html.spec.whatwg.org/multipage/webappapis.html#run-a-module-script
        let entry_expr = body.trim_end();
        Ok(format!(
            "{init}{entry_expr}.catch(function (__zwModTlaErr) {{ try {{ if (typeof reportError === 'function') {{ reportError(__zwModTlaErr); }} }} catch (_zwTlaRpt) {{}} }});\n"
        ))
    } else {
        // 记忆化入口体：本模块若已作为他元素依赖求值过（缓存命中）→ 复用实例，模块体
        // 恰好执行一次（红态实测入口计数 2：prelude 急切 + 转换体各一次）。入口自身无
        // 导入者 → 祖先恒空（失败只记入口键；中间传播由入口体内联发射携带的祖先链承担，
        // s345——prelude 不再急切求值入口图，内联发射为唯一记账位）。
        Ok(format!("{init}{}", memoized_dep_expr(cache_key, &body, &[])))
    }
}

/// 编译依赖模块为可求值的 IIFE 表达式（返回 exports 对象）。
pub fn compile_dependency_iife(specifier: &str, registry: &ModuleRegistry) -> Result<String, ScriptError> {
    let mut path = vec![specifier.to_string()];
    build_dep_iife(specifier, registry, "same-origin", &mut HashSet::new(), &mut path)
}

/// s360：入口模块的延迟求值编译（[`build_module_script_deferred`] 变体——宿主
/// `__zw_compile_module` 第 5 参 defer 标记分派；产物为 `{exports, depsPromise,
/// runBody}` 记录对象 IIFE，依赖图急切求值、入口体延迟到门面属性访问）。
/// https://tc39.es/proposal-defer-import-eval/
pub fn compile_dependency_iife_deferred(specifier: &str, registry: &ModuleRegistry) -> Result<String, ScriptError> {
    let source = registry
        .get(specifier)
        .ok_or_else(|| ScriptError::RuntimeError(format!("Module not found: {specifier}")))?
        .to_string();
    let mut path = vec![specifier.to_string()];
    build_module_script_deferred(
        &source,
        specifier,
        registry,
        "same-origin",
        &mut HashSet::new(),
        &mut path,
    )
}

/// s348（html-semantics-compat）：包装级 parse 失败的**记录归属扫描**——沿入口图
/// postorder（[`module_graph_postorder`]，求值序）定位首个 parse 失败模块，返回其
/// 注册键（错误记录属主）。spec：parse error 属失败模块自身的 module record（module
/// map 失败记账），入口/中间模块不加新记录；多失败图（循环双根 choice-of-error-1）
/// 各根按各自 DFS 序报告**不同记录**的错误——旧 s102 臂按入口键新建 + 字典序
/// dep-scan 复报，双根并成同对象（assert_not_equals 红态实证）。parse 优先于既有
/// 记录（spec：图内 parse 失败在求值前的 instantiate 阶段暴露，求值错误记录不抢）。
/// 逐键：
/// ① 转换单元（[`compile_dependency_iife`]，与装配脚本同 classic 语法面；raw 源含
///    import/export 不可 classic 编译）parse-check 失败 → 即返该键；
/// ② 体合法但键带既有失败记录（前序元素求值/parse 记账，`recorded_failures` 快照）
///    → 记为回落候选，扫描继续；
/// ③ 空/占位源（网络失败预存根）与 build Err（解析/链接失败类，非 parse 面）跳过。
/// 扫描完仍无 parse 失败：返回首个回落候选（非 parse 失败语义=旧 dep-scan 的求值序
/// 版）；全无 → None（调用方回落旧 s102 记账臂）。调用方经
/// [`crate::Sandbox::parse_check`] 注入 compile-only 校验。
/// https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
pub fn module_parse_failure_owner(
    entry: Option<&str>,
    registry: &ModuleRegistry,
    recorded_failures: &HashSet<String>,
    parse_check: &mut dyn FnMut(&str) -> Result<(), String>,
) -> Option<String> {
    let mut recorded = None;
    for (key, _) in module_graph_postorder(entry, registry) {
        let Some(source) = registry.get(&key) else {
            continue;
        };
        if source.trim().is_empty() {
            continue;
        }
        let Ok(unit) = compile_dependency_iife(&key, registry) else {
            continue;
        };
        if parse_check(&unit).is_err() {
            return Some(key);
        }
        if recorded.is_none() && recorded_failures.contains(&key) {
            recorded = Some(key);
        }
    }
    recorded
}

/// s255：入口图 depth-first post-order 注册序——模块体求值序按依赖图后序、同层按
/// 源内声明序（ECMA-262 EvaluateModule 图遍历语义）。旧实现在 registry 哈希序注册
/// （HashMap 迭代序），多依赖模块的副作用顺序不确定——instantiation-error-7 实测
/// 7d/7e 颠倒。入口图不可达的 registry 余量按字典序追加（保持全量急切注册覆盖，
/// 仅定序）；未注册依赖与解析失败说明符跳过（定序尽力而为，链接/语法错误仍由
/// build 路径如实报告）。
/// s344-r2：逐模块携带 DFS 祖先键（入口→自身路径，不含自身；入口/余量 = 空）——
/// 供 [`memoized_dep_expr`] 失败记账传播到中间模块（spec 求值错误中间缓存语义，
/// choice-of-error-3 判据）。余量模块不在入口图内、无导入者关系 → 祖先恒空。
/// https://tc39.es/ecma262/#sec-moduleevaluation
fn module_graph_postorder(entry: Option<&str>, registry: &ModuleRegistry) -> Vec<(String, Vec<String>)> {
    let mut ordered = Vec::new();
    let mut visited = HashSet::new();
    let mut path = Vec::new();
    if let Some(entry) = entry {
        postorder_visit(entry, registry, &mut visited, &mut ordered, &mut path);
    }
    let mut leftovers: Vec<&str> = registry
        .specifiers()
        .into_iter()
        .filter(|spec| !visited.contains(*spec))
        .collect();
    leftovers.sort_unstable();
    ordered.extend(leftovers.into_iter().map(|spec| (spec.to_string(), Vec::new())));
    ordered
}

fn postorder_visit(
    specifier: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
    ordered: &mut Vec<(String, Vec<String>)>,
    path: &mut Vec<String>,
) {
    if !visited.insert(specifier.to_string()) {
        return;
    }
    let Some(source) = registry.get(specifier) else {
        return;
    };
    path.push(specifier.to_string());
    for dep in static_dependency_specifiers(source, specifier, registry) {
        postorder_visit(&dep, registry, visited, ordered, path);
    }
    path.pop();
    ordered.push((specifier.to_string(), path.clone()));
}

/// s255：模块源静态依赖说明符（import / export-from 的 module request，源内出现序）。
/// 动态 `import()` 非链接依赖不入（spec：运行时取回，不影响静态图求值序）。解析与
/// transform 同口径（`resolve_registered_specifier`，过页面 import map）。
fn static_dependency_specifiers(source: &str, importer_url: &str, registry: &ModuleRegistry) -> Vec<String> {
    let mut raw = Vec::new();
    for stmt in split_statements(source) {
        let trimmed = stmt.trim();
        let head = trimmed.split(';').next().unwrap_or(trimmed).trim();
        if let Some(clause) = strip_import_keyword(trimmed) {
            let clause_head = clause.split(';').next().unwrap_or(clause).trim();
            if clause_head.starts_with('\'') || clause_head.starts_with('"') || clause_head.starts_with('`') {
                if let Ok(spec) = extract_string_literal(clause_head) {
                    raw.push(spec);
                }
            } else if let Some((_before, spec_part)) = split_from_clause(clause_head)
                && let Ok(spec) = extract_import_specifier_from_rest(spec_part)
            {
                raw.push(spec);
            }
        } else if strip_export_keyword(trimmed).is_some()
            && (head.starts_with('*') || head.starts_with('{'))
            && let Some((_before, spec_part)) = split_from_clause(head)
            && let Ok(spec) = extract_import_specifier_from_rest(spec_part)
        {
            raw.push(spec);
        }
    }
    // s259：Blocked（null 条目）依赖不入定序图（module_graph_postorder 尽力
    // 而为契约不变——链接/解析错误由 transform/build 路径经 ? 如实报告）。
    raw.into_iter()
        .filter_map(|spec| resolve_registered_specifier(&spec, importer_url, registry).ok())
        .collect()
}

/// s361：动态 import 运行时 helper 段（三表 init + `__zw_load_module` +
/// `__zw_dynamic_import` + `__zwImportMapResolve` 全局定义，registry 无关）——
/// [`build_module_runtime_prelude`] 前缀复用；webview 动态/defer 契约产物同拼：
/// 契约 eval 上下文在非 module 页面（prelude 从未执行）缺 helper——嵌套
/// `import()` 内联体 ReferenceError（cyclic-tla-defer-eval 红态实证面）。
/// 重复定义幂等（同形覆盖，页面包 + 契约包双发射零行为漂移）。
pub fn module_dynamic_import_helpers_prelude() -> String {
    let mut out = String::from(
        "globalThis.__moduleCache = globalThis.__moduleCache || {};\nglobalThis.__moduleFailures = globalThis.__moduleFailures || {};\nglobalThis.__zwModuleFetchKeys = globalThis.__zwModuleFetchKeys || {};\n",
    );
    // t8j：helper（__zw_load_module / __zw_dynamic_import）必须先于 `__moduleCache[spec] = IIFE`
    // 注册求值——注册右侧的 IIFE **立即执行模块体**（依赖预热 + collect_module_deps 把入口模块
    // 自身也注册进 registry，见 js_worker/tab_js_worker collect_module_deps）。模块体顶层的
    // `import()`（Vite 现代浏览器探测 `import("_").catch(()=>1)`）在求值瞬间需要 helper 已定义；
    // 反序时入口模块体在 prelude 第 2 条语句执行 → `__zw_dynamic_import is not defined` ReferenceError，
    // 探测中断、后续语句永不执行 → 站点误回落 legacy 加载路径。
    // s338：第 4 参 credHint = 动态 import() 调用点模块的 credentials mode（重写发射
    // 经 `.call({referrer, credentials})` 穿入，s338 面；缺省 same-origin——classic
    // 直调形态/旧发射无此槽）。穿给宿主第 4 参（renderer/tab 旧宿主契约忽略多余
    // 参数，行为同 same-origin 缺省；webview 侧以 `__zw_cred` 查询标记穿 fetcher，
    // 见 webview __zw_compile_module）。
    // s363（mixed eager/defer 同 URL 记录共享）：eager import 命中 defer 共享记录的
    // 统一等待 helper——先等记录依赖图完成（depsPromise；spec eager import 语义 =
    // 全图求值完成），再触发体求值（runBody 恰一次守卫内建于 deferred 编译产物），
    // resolve 导出面。cyclic TLA 死锁面（s361）依赖此等待序：deps 挂起时 promise
    // 不落定，体不提前跑。
    out.push_str("globalThis.__zwEagerFromDefer = function (drec) {\n");
    out.push_str("  return Promise.resolve(drec.depsPromise).then(function () {\n");
    out.push_str("    drec.runBody();\n");
    out.push_str("    return drec.exports;\n");
    out.push_str("  });\n");
    out.push_str("};\n");
    out.push_str("globalThis.__zw_load_module = function(spec, parentHint, typeHint, credHint) {\n");
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
    // s87（html-semantics-compat dynamic-import must-reject 面）失败四态（HTML spec
    // https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
    // module map 失败记账语义；WPT dynamic-imports-fetch-error/script-error 判据）：
    // ① 宿主返空 = 顶层取回失败（404/500/MIME/跨源/解析失败）→ **每次**调用抛新
    //   TypeError（不入缓存）——fetch 失败不入 module map，逐次新错误对象；
    // ② 宿主返 `resolved\u{1f}`（空码标记）= 顶层已取回、依赖图编译失败 → 首抛新
    //   TypeError 且记入 `__moduleFailures`，重调复抛**同对象**（parse 期失败入 map 记账）；
    // ③ 取回成功但模块体 parse/求值抛错 → 错误对象记入 `__moduleFailures` 复抛同对象
    //   （spec：parse error 与 evaluation error 均缓存）。未发标记的宿主（renderer/tab）
    //   ②退化为 ①，行为不劣化。
    // ④ s252：宿主返 `3\u{1f}resolved\u{1f}message` = 链接/instantiation 期失败
    //   （缺导出等 CompileError）→ **每次**调用抛新 SyntaxError（不入缓存）——spec：
    //   link 失败不完成模块记录、不入 module map，逐次新错误对象（WPT
    //   dynamic-imports-script-error「instantiation error」判据 SyntaxError +
    //   differentErrorObjects；s87 ③ 旧口径「暂并入」翻案独立成态）。
    // s267（动态 import 取回缓存面）：`(spec, parent, type)` → resolved 记账后，
    // 同键复 import 且命名空间在缓存时**跳过宿主取回**直接回缓存实例——HTML
    // module map 语义（fetch-a-single-module-script 命中即复用，复 import 不再
    // 发网络请求；repeated-import 三页判据：成功复 import 服务端计数 0、串行/
    // 并行 ×5 计数 1）。失败态不记账（宿主返空 ① / 图编译失败 ② 空码 /
    // parse-eval 抛 ③ / 链接失败 ④ 均不入 `__zwModuleFetchKeys`）——取回失败
    // 不入 module map，重 import 重新发请求（同上 WPT 判据：404/MIME 步计数 1）。
    // t8j-r2「webview 进程内路径无宿主缓存，重复动态 import 重复取回——转池候选」
    // 在此收口：缓存查访在宿主调用之前，宿主重复取回面消除。
    // https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
    out.push_str(
        "  var fk = spec + '\\u001f' + parent + '\\u001f' + (typeof typeHint === 'string' ? typeHint : '');\n",
    );
    out.push_str("  var kr = __zwModuleFetchKeys[fk];\n");
    // s363：fk 缓存命中的 defer 共享键（同 URL 先被 import.defer 取过）走等待 helper
    // ——复 import 与首次同形态（Promise，等依赖图完成后触发体），spec eager import
    // 等全图求值完成语义在复取路径保持。
    out.push_str("  if (kr !== undefined && __moduleCache[kr]) {\n");
    out.push_str("    if (globalThis.__zwDeferRecords && globalThis.__zwDeferRecords[kr]) return __zwEagerFromDefer(globalThis.__zwDeferRecords[kr]);\n");
    out.push_str("    return __moduleCache[kr];\n");
    out.push_str("  }\n");
    out.push_str(
        "  var r = __zw_compile_module(spec, parent, typeof typeHint === 'string' ? typeHint : '', credHint || '');\n",
    );
    out.push_str("  if (!r) throw new TypeError(\"Failed to fetch module: '\" + spec + \"'\");\n");
    out.push_str("  var sep = r.indexOf('\\u001f');\n");
    // s252：tag 3（链接失败）判定须先于缓存/失败表读取（逐次新对象，不入两表）。
    out.push_str("  if (sep === 1 && r.charCodeAt(0) === 51) {\n");
    out.push_str("    var p3 = r.slice(2);\n");
    out.push_str("    var m3 = p3.indexOf('\\u001f');\n");
    out.push_str("    throw new SyntaxError(m3 >= 0 ? p3.slice(m3 + 1) : p3);\n");
    out.push_str("  }\n");
    out.push_str("  var resolved = sep >= 0 ? r.slice(0, sep) : spec;\n");
    // s363（mixed eager/defer 同 URL 记录共享——spec module map 单键，两视图同一记录）：
    // defer 先取的 URL 再走 eager import 时，**先等 defer 记录依赖图完成**（depsPromise，
    // spec eager import 语义 = 全图求值完成）再触发体求值（runBody 恰一次守卫内建）
    // 并以其导出面入 eager 缓存——此后同 URL eager 复 import 走既有缓存快速路径。
    // 返回 Promise 形态：消费方 __zw_dynamic_import 以 `Promise.resolve(...)` 展开
    // （es_module 625 发射面），既有同步直调面（单测）无 defer 记录落穿不受扰。
    // 守卫 `__zwDeferRecords &&`：契约 eval 上下文（s361 拼接 helpers）无 part01、
    // defer 表缺位时按原路径落穿零漂移。cyclic TLA 死锁面（s361）依赖此等待序——
    // 红态实证：同步 runBody 在 deps 挂起时提前跑体，`module A's body should never
    // run` 断言红（P→F），死锁时序破坏。记账（fk+缓存，exports 引用同源）同步做——
    // 复 import 经 fk/resolved 快速路径命中 defer 表后走同一等待 helper（形态一致，
    // 生产消费面 __zw_dynamic_import 以 Promise.resolve 包裹展开两形态通吃）。
    out.push_str("  if (globalThis.__zwDeferRecords && globalThis.__zwDeferRecords[resolved]) {\n");
    out.push_str("    var drec = globalThis.__zwDeferRecords[resolved];\n");
    out.push_str("    __moduleCache[resolved] = drec.exports;\n");
    out.push_str("    __zwModuleFetchKeys[fk] = resolved;\n");
    out.push_str("    return __zwEagerFromDefer(drec);\n");
    out.push_str("  }\n");
    // s363：resolved 层复取命中 defer 共享键——同款等待 helper（fk 不同键同 URL 时）。
    out.push_str("  if (__moduleCache[resolved]) {\n");
    out.push_str("    if (globalThis.__zwDeferRecords && globalThis.__zwDeferRecords[resolved]) return __zwEagerFromDefer(globalThis.__zwDeferRecords[resolved]);\n");
    out.push_str("    return __moduleCache[resolved];\n");
    out.push_str("  }\n");
    out.push_str("  if (__moduleFailures[resolved]) throw __moduleFailures[resolved];\n");
    out.push_str("  var code = sep >= 0 ? r.slice(sep + 1) : r;\n");
    out.push_str("  if (sep >= 0 && code === '') {\n");
    out.push_str("    var graphError = new TypeError(\"Failed to fetch module graph: '\" + resolved + \"'\");\n");
    out.push_str("    __moduleFailures[resolved] = graphError;\n");
    out.push_str("    throw graphError;\n");
    out.push_str("  }\n");
    out.push_str("  try {\n");
    // s253（CSP eval 门禁互斥）：模块求值仿真不走页面 eval 门禁。webview 侧 CSP
    // `script-src` 无 unsafe-eval 时 per-script 装 eval/Function 包装拦页面字符串
    // 编译 API；spec 上 module evaluation 不是 eval()——script-src 管模块 fetch
    // （nonce 校验已在宿主 resolver 完成），不管求值机制（propagate-nonce-
    // inline-module 实测：包装在场时此 eval 抛 EvalError，import 被自家 CSP 误拒）。
    // 包装在场时经 `__zwRealEval`（包装安装点保存的真实 eval，webview M2-s6）求值
    // ——仅此编译调用豁免；模块体内嵌套 eval()/Function 仍解析到 globalThis.eval
    // （包装）照拦，WPT string-compilation-* 页面级 eval 判据不受扰。未装包装
    // （renderer/tab 宿主、无 CSP 页）回落 eval 同形。
    // https://w3c.github.io/webappsec-csp/#can-compile-strings
    // https://html.spec.whatwg.org/multipage/webappapis.html#hostimportmoduledynamically-retrieval
    // s339：桥位在场（构建期判定——同步直调形态）才夹 `MODULE_EVAL_WINDOW_SCOPE_OPEN/
    // CLOSE` 窗口身份包裹；缺席按原样求值（与 v8_runtime settle 臂同形，worker/裸
    // sandbox 面零漂移——见常量文档）。
    out.push_str(
        "    __moduleCache[resolved] = (function() { var __zwEvalSrc = globalThis.__zwActiveIframeWin ? ('(' + '",
    );
    out.push_str(MODULE_EVAL_WINDOW_SCOPE_OPEN);
    out.push_str("' + code + '");
    out.push_str(MODULE_EVAL_WINDOW_SCOPE_CLOSE);
    out.push_str("' + ')') : ('(' + code + ')'); return (typeof __zwRealEval === 'function' ? __zwRealEval : eval)(__zwEvalSrc); })();\n");
    out.push_str("  } catch (e) {\n");
    out.push_str("    __moduleFailures[resolved] = e;\n");
    out.push_str("    throw e;\n");
    out.push_str("  }\n");
    out.push_str("  __zwModuleFetchKeys[fk] = resolved;\n");
    out.push_str("  return __moduleCache[resolved];\n");
    out.push_str("};\n");
    // t8j：动态 import() 失败须以 rejected promise 呈现，不得同步 throw——
    // https://tc39.es/ecma262/#sec-import-calls（ImportCall 恒返回 promise；取回/编译失败
    // 走 promise rejection）。同步 throw 发生在调用方 `.catch()` 挂上之前（`import(x).catch(…)`
    // 表达式求值即抛），Vite 现代浏览器探测模块 `import("_").catch(()=>1)` 由此整体中断、
    // 后续语句（`__vite_is_modern_browser=true`）永不执行 → 站点误回落 legacy 加载路径。
    // s249（动态 import 属性）：第二实参（attributes 对象）旧版被丢弃——`import(x,
    // { with: { type: "css" } })` 与 `import(x)` 对宿主不可分，css/text/json 数据
    // 模块无法记账（WPT css-module/text-module/json-module 动态面全族红）。此处按
    // TC39 import calls phase 2 解析：attributes 在场非对象 → TypeError；`with`
    // 缺省 → 无属性（JS 模块语义）；`with` 非对象 → TypeError；`type` 值 ∉ module
    // type map（json/css/text——与 import_attributes.rs VALID_MODULE_TYPES 同集）
    // → TypeError。校验失败同步 throw 由 `__zw_dynamic_import` 的 try 捕获转
    // rejected promise（ImportCall 恒返 promise，同上）。type 以第三参穿宿主
    //（`__zw_compile_module(spec, parent, type)`——合成臂按类型分派）。
    // https://tc39.es/proposal-import-attributes/#sec-import-calls
    // https://html.spec.whatwg.org/multipage/webappapis.html#module-type-map
    out.push_str("globalThis.__zw_dynamic_import = function(spec, attributes) {\n  var parent = (this && this.referrer) || ((typeof _importMeta !== 'undefined' && _importMeta.url) || 'about:blank');\n  // s338：credentials mode = 调用点发射穿入的 `this.credentials`（模块 IIFE 局部\n  // _importCred，元素 crossorigin 面），缺省 same-origin（classic 直调/旧发射）。\n  // 恒在 try 外先读——`this` 在后续 then 链内失效。\n  var _diCred = (this && this.credentials) || 'same-origin';\n  try {\n    var _diType = '';\n    if (attributes !== undefined && attributes !== null) {\n      if (typeof attributes !== 'object') {\n        throw new TypeError('Import attributes argument must be an object');\n      }\n      var _diWith = attributes.with;\n      if (_diWith !== undefined && _diWith !== null) {\n        if (typeof _diWith !== 'object') {\n          throw new TypeError('Import attributes \\'with\\' value must be an object');\n        }\n        var _diRawType = _diWith.type;\n        if (_diRawType !== undefined) {\n          _diType = String(_diRawType);\n          if (_diType !== 'json' && _diType !== 'css' && _diType !== 'text') {\n            throw new TypeError('Unsupported module type \"' + _diType + '\"');\n          }\n        }\n      }\n    }\n    // s337：双跳取回——单跳时取回体与元素 load 尾片同代竞争（同步块排队序先于
    // load 尾片），evaluation-order-4 判据 load 须先于动态依赖体（Chrome 取回跨
    // 宏任务，body 反应晚于 load 尾片代）；双跳后 hop1 空代让位 load 尾片，依赖
    // 体落次代。order-3 序不变（hop1 无日志，依赖体本就后于 mt-3.1）。
    // https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
    return Promise.resolve().then(function () {\n      return Promise.resolve().then(function () {\n        return Promise.resolve(__zw_load_module(spec, parent, _diType, _diCred));\n      });\n    });\n  } catch (e) {\n    return Promise.reject(e);\n  }\n};\n");
    // s254：import.meta.resolve 宿主桥（HTML #dom-import-meta-resolve——spec 入参经
    // ToString 后走 resolve a module specifier 全算法：map 命中 → 映射 URL 绝对化；
    // URL-like 未命中 → asURL 回退）。宿主返空 = resolution failure（裸说明符未映射/
    // 阻断条目）→ TypeError——spec 面较动态 import 的 join 降级更严（裸未映射本就
    // 不可解析，动态面 join 行为既有不扰）。回调未注册的宿主（renderer/tab 多进程
    // 接线未达，FIXME 挂账）同样回落 TypeError 而非 ReferenceError。
    // https://html.spec.whatwg.org/multipage/webappapis.html#dom-import-meta-resolve
    out.push_str("globalThis.__zwImportMapResolve = function (spec, base) {\n  if (typeof __zw_importmap_resolve !== 'function') throw new TypeError(\"Failed to resolve module specifier '\" + spec + \"'\");\n  var r = __zw_importmap_resolve(spec, base);\n  if (r === '') throw new TypeError(\"Failed to resolve module specifier '\" + spec + \"'\");\n  return r;\n};\n");
    out
}

/// 生成模块运行时 prelude（`__moduleCache` + 动态 `import()` 支持）。
///
/// R5182（模块恰好一次求值）：`__moduleCache` 挂 `globalThis` 持久——同文档多个 module
/// script 元素共享一份模块缓存（HTML module map 语义，fetch-a-single-module-script 命中
/// 即复用）。**文档边界由宿主负责**：renderer 路径经 `reset_context` 销毁重建全局对象自然
/// 复位；tab worker 与 webview 进程内路径持持久 context，须在每文档脚本阶段起点执行
/// [`MODULE_CACHE_RESET_SCRIPT`]（否则跨文档导航后同 URL 模块零次求值——D1 回归）。
/// `entry` 为入口模块 URL：入口图内模块（含入口）不在 prelude 急切执行——求值由
/// 调用方 `compile_module_script` 转换体的内联图承担（s345：循环依赖须在入口 IIFE 内
/// 闭合——导出 getter 先于提升 import 发射，依赖体经活对象读未初始化 let/const 自然
/// TDZ；prelude 急切求值会把入口源码内联进循环依赖 IIFE，体序倒置且 TDZ 失明）。
/// 入口图不可达的 registry 余量照旧急切求值，序为字典序（全量预热覆盖契约不变）。
pub fn build_module_runtime_prelude(registry: &ModuleRegistry, entry: Option<&str>) -> Result<String, ScriptError> {
    // s361：helper 段抽离（[module_dynamic_import_helpers_prelude]）——契约产物
    // 同拼面，见该函数注记。
    let mut out = module_dynamic_import_helpers_prelude();
    // s345：入口图内模块（含入口）不再 prelude 急切求值——求值由入口转换体的**内联图**
    // 承担（体首提升 import 的 memoized 发射，内联递归本身即入口图后序；循环依赖在入口
    // IIFE 内闭合，导出 getter 先于提升 import 发射 → 依赖体经活对象读到未初始化
    // let/const 自然 TDZ 抛 ReferenceError，spec ill-founded cycle 语义，evaluation-
    // error-2 实测修复）。prelude 急切求值在循环依赖面把入口源码内联进依赖 IIFE（s344
    // 入口入册后 registry 对 prelude 构建可见入口源）→ 入口体先于依赖体完成 → TDZ 失明
    // + 体序倒置（红态实测 got 体序倒置 5 项）。入口图不可达的 registry 余量保持急切
    // 求值（全量预热覆盖契约不变；entry = None 契约即全量，v8_runtime 助手面同型）。
    // https://tc39.es/ecma262/#sec-moduleevaluation
    // https://html.spec.whatwg.org/multipage/webappapis.html#run-a-module-script
    let mut reachable = HashSet::new();
    if let Some(root) = entry {
        let mut ordered = Vec::new();
        let mut path = Vec::new();
        postorder_visit(root, registry, &mut reachable, &mut ordered, &mut path);
    }
    for (spec, ancestors) in module_graph_postorder(entry, registry) {
        if reachable.contains(&spec) {
            continue;
        }
        let mut path = vec![spec.clone()];
        let iife = build_dep_iife(&spec, registry, "same-origin", &mut HashSet::new(), &mut path)?;
        // R5182：已缓存（本文档更早的 module script 元素已求值）→ 不再执行——同文档模块
        // 恰好一次（红态实测共享依赖跨元素计数 4；HTML module map 命中即复用）。判定用
        // `=== undefined`（contains 语义，TV-4）。s269：TLA 依赖缓存 Promise——消费方
        // await 归一（await_prefix），动态 import() 面 Promise.resolve 天然解包。
        // s344：记忆化发射统一走 `memoized_dep_expr`（失败表先行：命中失败记记录复抛
        // 同对象不重求值——裸守卫形态下已失败依赖在后续元素 prelude 重跑体、抛新对象，
        // choice-of-error-3 `log[1] === log[3]` 判据翻转）。
        out.push_str(&format!("{};\n", memoized_dep_expr(&spec, &iife, &ancestors)));
    }
    Ok(out)
}

/// JS 标识符续字符的 ASCII 字节面（`[A-Za-z0-9_$]`，ECMA-262 §sec-identifier-names
/// ID_Continue 的 ASCII 子集）——非 ASCII 续字符前置时该守卫失明（轻量扫描器既限）。
fn is_identifier_continue_byte(b: u8) -> bool {
    b == b'_' || b == b'$' || b.is_ascii_alphanumeric()
}

pub(crate) fn rewrite_dynamic_imports(source: &str) -> String {
    rewrite_imports_impl(source, ImportRewriteMode::Module)
}

/// s360（html-semantics-compat import.defer 面）：重写目标档位。
/// - [`ImportRewriteMode::Module`]：模块管线（`import(` → `__zw_dynamic_import`、
///   `import.defer(` → `__zw_import_defer`——发射体引用模块 IIFE 局部 `_importMeta`/
///   `_importCred`，与既有 eager 臂同槽）。
/// - [`ImportRewriteMode::ClassicDeferOnly`]：classic 页面脚本面——**只**重写
///   `import.defer(`（→ `__zwClassicImportDefer`，helper 自读 s90 referrer 槽）；
///   `import(` 不重写（classic 上下文走 V8 原生宿主回调 s88，重写反而失去槽位
///   referrer——模块 lane 的 `_importMeta` 局部量在 classic 源不存在）。
#[derive(Clone, Copy, PartialEq, Eq)]
enum ImportRewriteMode {
    Module,
    ClassicDeferOnly,
}

/// classic 页面脚本源的 import.defer 重写（s360——defer 动态形式在 bundled V8 默认
/// 关旗（`--js-import-defer` 未开）下 parse 即 SyntaxError「Unexpected identifier
/// 'defer'」，须在源码面重写到 JS helper；`import(` 原生通道不受扰）。
pub fn rewrite_classic_deferred_imports(source: &str) -> String {
    if !source.contains("import.defer") {
        return source.to_string();
    }
    rewrite_imports_impl(source, ImportRewriteMode::ClassicDeferOnly)
}

/// import 动态调用重写核心（[`rewrite_dynamic_imports`] 与
/// [`rewrite_classic_deferred_imports`] 共享的上下文感知扫描器）。
fn rewrite_imports_impl(source: &str, mode: ImportRewriteMode) -> String {
    // s91（string-compilation module 面）：字符串/模板/注释上下文感知——旧版纯文本
    // 扫描把**字符串字面量内**的 `import(`（eval/Function/setTimeout 字符串、事件处
    // 理器属性体的源码形态）也重写并发射 `_importMeta.url`，该绑定是模块 IIFE 局部
    // 量，字符串运行时经 Function/事件处理器编译到全局作用域 → `ReferenceError:
    // _importMeta`（探针实证：s91probe-fn/-reflected sync-throw、-timer 静默抛）。
    // 上下文内不重写 → 运行时编译体走 V8 原生动态 import() 宿主回调（classic/module
    // 页面 s88/s90 已接通槽位 referrer；SW 上下文动态 import 本就 spec 禁止，prelude
    // stub 维持 TypeError 语义）。
    // 已知限制：正则字面量内的引号（如 `/['"]/`）会把扫描器带入字符串态——后续真实
    // `import(` 漏重写降级走原生回调（页面上下文行为正确，仅 referrer 取槽位而非词法
    // 穿参）；全量 JS 词法器不做，按 t8j-r2 轻量精神取舍。
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    let mut stack: Vec<RewriteCtx> = Vec::new();
    let mut i = 0;
    while i < source.len() {
        if stack.is_empty() {
            // s267：标识符边界守卫——前邻标识符续字符时 `import(` 是标识符后缀
            // （repeated-import 页 `removeScriptReimport()` 探针实证 mid-identifier
            // 误重写 → ReferenceError），非 import 关键字，原样拷贝继续。
            if source[i..].starts_with("import(")
                && (i == 0 || !is_identifier_continue_byte(bytes[i - 1]))
                && mode == ImportRewriteMode::Module
            {
                // t8j：经 `.call({referrer})` 把调用点 referrer 穿给 helper——重写点在模块 IIFE
                // 内（build_module_script / build_dep_iife 均定义 `var _importMeta`；service_worker
                // 经典脚本路径在宿主侧设置 `globalThis._importMeta`），而 `__zw_dynamic_import`
                // 本体在全局作用域执行，读不到 IIFE 局部的 `_importMeta`（旧实现恒回退
                // 'about:blank' → 相对 spec 解析失败、按原始 specifier 发起网络请求）。
                // ECMA-262 §sec-module-specifiers：动态 import 的 specifier 相对
                // **active script/module**（即调用点 referrer）解析。
                // t8j-r2（D2）：发射串不得含任何引号字符——发射进宿主单引号字符串
                //（如 `throw new Error('import() failed')`）时引号提前终止字符串 → 整脚
                // 本 SyntaxError（base 发射串无引号仅值污染）。发射体只引用调用点词法
                // 作用域的 `_importMeta`（代码态重写目标已保证定义），零引号。
                // s338：credentials 槽同理引用 IIFE 局部 `_importCred`（build_module_script
                // / build_dep_iife 头部发射，元素 crossorigin 面；两处发射点覆盖全部重写
                // 落点），零引号零字面值。
                out.push_str("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, ");
                i += "import(".len();
                continue;
            }
            // s360：`import.defer(` 动态延迟导入重写——bundled V8 默认关 `--js-import-defer`
            // 旗，该语法 parse 即 SyntaxError「Unexpected identifier 'defer'」。Module 档
            // 发射体引用模块 IIFE 局部 `_importMeta`/`_importCred`（同 eager 臂槽位纪律，
            // 零引号）；ClassicDeferOnly 档发射 `__zwClassicImportDefer`（helper 自读
            // s90 referrer 槽 / s340 credentials 槽，classic 源无 IIFE 局部量可引）。
            // spec 延迟求值语义见 part01 shim `__zw_import_defer` 注记。
            // https://tc39.es/proposal-defer-import-eval/
            if source[i..].starts_with("import.defer(") && (i == 0 || !is_identifier_continue_byte(bytes[i - 1])) {
                match mode {
                    ImportRewriteMode::Module => {
                        out.push_str(
                            "__zw_import_defer.call({ referrer: _importMeta.url, credentials: _importCred }, ",
                        );
                    }
                    ImportRewriteMode::ClassicDeferOnly => {
                        out.push_str("__zwClassicImportDefer(");
                    }
                }
                i += "import.defer(".len();
                continue;
            }
            match bytes[i] {
                b'\'' | b'"' | b'`' => stack.push(RewriteCtx::Literal(bytes[i])),
                b'/' if bytes.get(i + 1) == Some(&b'/') => {
                    stack.push(RewriteCtx::LineComment);
                    out.push_str("//");
                    i += 2;
                    continue;
                }
                b'/' if bytes.get(i + 1) == Some(&b'*') => {
                    stack.push(RewriteCtx::BlockComment);
                    out.push_str("/*");
                    i += 2;
                    continue;
                }
                _ => {}
            }
            let step = source[i..].chars().next().map_or(1, char::len_utf8);
            out.push_str(&source[i..i + step]);
            i += step;
            continue;
        }
        let c = bytes[i];
        // 栈空分支已在上方 continue；此处栈非空，`expect` 恒成立。
        match stack.last_mut().expect("ctx stack non-empty") {
            RewriteCtx::Literal(q) => {
                if c == b'\\' {
                    // 转义对：拷转义符与被转义字符（UTF-8 逐字符，防切多字节序列）。
                    let step = source[i..].chars().next().map_or(1, char::len_utf8);
                    out.push_str(&source[i..i + step]);
                    i += step;
                    if i < source.len() {
                        let step2 = source[i..].chars().next().map_or(1, char::len_utf8);
                        out.push_str(&source[i..i + step2]);
                        i += step2;
                    }
                    continue;
                }
                if c == *q {
                    stack.pop();
                } else if *q == b'`' && c == b'$' && bytes.get(i + 1) == Some(&b'{') {
                    // 模板表达式态：`${` 内是代码——真实 `import(` 须照常重写。
                    stack.push(RewriteCtx::TemplateExpr(0));
                    out.push_str("${");
                    i += 2;
                    continue;
                }
                let step = source[i..].chars().next().map_or(1, char::len_utf8);
                out.push_str(&source[i..i + step]);
                i += step;
            }
            RewriteCtx::TemplateExpr(depth) => {
                // 模板 `${}` 表达式区内仍是代码——真实 `import(` 照常重写（仍在模块
                // IIFE 词法作用域内，发射体 `_importMeta`/`_importCred` 合法；标识符
                // 边界守卫同上，s338 credentials 槽同规）。
                if source[i..].starts_with("import(")
                    && (i == 0 || !is_identifier_continue_byte(bytes[i - 1]))
                    && mode == ImportRewriteMode::Module
                {
                    out.push_str("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, ");
                    i += "import(".len();
                    continue;
                }
                // s360：模板表达式区仍是模块 IIFE 词法作用域内代码——defer 臂同
                // 栈空分支（发射体局部量合法、标识符边界守卫同规）。
                if source[i..].starts_with("import.defer(") && (i == 0 || !is_identifier_continue_byte(bytes[i - 1])) {
                    match mode {
                        ImportRewriteMode::Module => {
                            out.push_str(
                                "__zw_import_defer.call({ referrer: _importMeta.url, credentials: _importCred }, ",
                            );
                        }
                        ImportRewriteMode::ClassicDeferOnly => {
                            out.push_str("__zwClassicImportDefer(");
                        }
                    }
                    i += "import.defer(".len();
                    continue;
                }
                match c {
                    b'\'' | b'"' | b'`' => stack.push(RewriteCtx::Literal(c)),
                    b'{' => *depth += 1,
                    b'}' => {
                        if *depth == 0 {
                            stack.pop(); // `}` 闭合 `${` → 回模板串
                        } else {
                            *depth -= 1;
                        }
                    }
                    _ => {}
                }
                let step = source[i..].chars().next().map_or(1, char::len_utf8);
                out.push_str(&source[i..i + step]);
                i += step;
            }
            RewriteCtx::LineComment => {
                if c == b'\n' {
                    stack.pop();
                }
                let step = source[i..].chars().next().map_or(1, char::len_utf8);
                out.push_str(&source[i..i + step]);
                i += step;
            }
            RewriteCtx::BlockComment => {
                if c == b'*' && bytes.get(i + 1) == Some(&b'/') {
                    stack.pop();
                    out.push_str("*/");
                    i += 2;
                    continue;
                }
                let step = source[i..].chars().next().map_or(1, char::len_utf8);
                out.push_str(&source[i..i + step]);
                i += step;
            }
        }
    }
    out
}

/// `rewrite_dynamic_imports` 的词法上下文栈元素（代码态 = 栈空）。
#[derive(Clone, Copy)]
enum RewriteCtx {
    /// 单/双引号字符串或模板串（值 = 定界符字节）。
    Literal(u8),
    /// 模板串 `${` 表达式态（值 = 未闭合 `{` 深度；0 时 `}` 回模板串）。
    TemplateExpr(u32),
    /// `//` 行注释（至行尾）。
    LineComment,
    /// `/* */` 块注释（至 `*/`）。
    BlockComment,
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
pub(crate) fn strip_import_keyword(stmt: &str) -> Option<&str> {
    let rest = stmt.strip_prefix("import")?;
    match rest.chars().next()? {
        c if c.is_whitespace() => Some(rest.trim_start()),
        '{' | '"' | '\'' | '`' | '*' => Some(rest),
        _ => None,
    }
}

/// s361：剥离静态 deferred namespace 声明的 `defer` 前缀（tc39
/// proposal-defer-import-eval 静态形式 `import defer * as ns from` → 名空间子句）。
/// 发射面（[`transform_import`]）与图收集面（[`extract_static_module_imports_with_attributes`]
/// / [`extract_static_module_import_specifiers`]）共用同一归一——漏归一则
/// [`import_clause_shape_ok`] 按 default 绑定形态判 `defer * as ns` 失败 → 依赖
/// 漏收（Module not found，cyclic 页红态实证面）。守卫：`defer` 后须 `*`，排除
/// `import defer from 'm'`（default 绑定名恰为 defer）与 `import deferredX`
/// （绑定名 defer 前缀重叠）。
fn strip_defer_prefix(clause: &str) -> &str {
    match clause.strip_prefix("defer") {
        Some(after) if after.trim_start().starts_with('*') => after.trim_start(),
        _ => clause,
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
pub(crate) fn split_from_clause(clause: &str) -> Option<(&str, &str)> {
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

// https://tc39.es/ecma262/#sec-imports — ImportClause 仅允许 ImportedDefaultBinding、
// `*` NameSpaceImport、NamedImports（前两者可 `,` 组合）。识别到 import 关键字不代表
// 语句形态合法：非绑定形态语句（webpack bundle 的 `export const __webpack_modules__=
// {…}` 巨型赋值实测，react-core 2026-10-09）会把正文里错误消息字符串的
// `from \`react-router/dom\`` 文本当 from 子句切出幻影依赖 → 404 → 模块编译整体失败
// → 站点 chunk 加载三连重试后报错。绑定/名字部分只含标识符与标点，先验形态再扫 from。
pub(crate) fn import_clause_shape_ok(clause: &str) -> bool {
    match clause.chars().next() {
        // 字符串形态（import"m"）、命名空间、命名导入
        Some('{') | Some('*') | Some('\'') | Some('"') | Some('`') => true,
        Some(first) if first.is_alphanumeric() || first == '_' || first == '$' => {
            // ImportedDefaultBinding：IDENT 后只允许空白与 `from`/`,`
            let ident_end = clause
                .char_indices()
                .find(|(_, c)| !(c.is_alphanumeric() || *c == '_' || *c == '$'))
                .map_or(clause.len(), |(i, _)| i);
            let after_ident = clause[ident_end..].trim_start();
            after_ident.starts_with("from") || after_ident.starts_with(',')
        }
        _ => false,
    }
}

// https://tc39.es/ecma262/#sec-exports — 再导出仅 `export {…}` 与 `export *`（可带
// from 子句）。`export const …` 等声明形态不是再导出，须整句跳过（防线上移到形态
// 检查，理由同 [`import_clause_shape_ok`]）。
fn reexport_clause_shape_ok(clause: &str) -> bool {
    matches!(clause.chars().next(), Some('{') | Some('*'))
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
            let clause = strip_defer_prefix(clause);
            if !import_clause_shape_ok(clause) {
                continue;
            }
            extract_import_specifier(clause)
        } else if let Some(clause) = strip_export_keyword(trimmed) {
            if !reexport_clause_shape_ok(clause) {
                continue;
            }
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

/// s107：静态 import 语句的 (specifier, with-type 属性) 对——JSON module 图收集按
/// type 属性分派数据模块合成（webview collect_module_deps_recursive）。复用 s105
/// 属性子句解析（suffix_after_specifier_literal / parse_with_clause）；属性块语法
/// 错误由校验面（validate_static_import_attributes）先行报告，此处按无属性处理。
/// export-from 再导出同为静态依赖（s116：图收集缺它 → 纯 re-export wrapper 的命名
/// 导入链接期查空——referrer insecure wrapper 实证）；with 属性在该形态语料未出现，
/// 一律按 None 顺延（FIXME：`export … from 'm' with {…}` 属性面）。
/// 动态 import() 不在此面（s105 同口径）。
pub fn extract_static_module_imports_with_attributes(source: &str) -> Vec<(String, Option<String>)> {
    let mut pairs: Vec<(String, Option<String>)> = Vec::new();
    for stmt in split_statements(source) {
        let Some(clause) = strip_import_keyword(stmt.trim()) else {
            // s116：export-from 再导出依赖臂（形态先验同
            // [`extract_static_module_import_specifiers`]——防正文消息字符串幻影依赖）。
            if let Some(export_clause) = strip_export_keyword(stmt.trim())
                && reexport_clause_shape_ok(export_clause)
                && let Ok(spec) = extract_reexport_specifier(export_clause)
                && !pairs.iter().any(|(existing, _)| existing == &spec)
            {
                pairs.push((spec, None));
            }
            continue;
        };
        let clause = strip_defer_prefix(clause);
        if !import_clause_shape_ok(clause) {
            continue;
        }
        let Ok(spec) = extract_import_specifier(clause) else {
            continue;
        };
        let type_attr = crate::import_attributes::suffix_after_specifier_literal(clause)
            .and_then(|suffix| crate::import_attributes::parse_with_clause(suffix).ok())
            .flatten()
            .and_then(|attrs| attrs.into_iter().find(|(key, _)| key == "type").map(|(_, v)| v));
        if !pairs.iter().any(|(existing, _)| existing == &spec) {
            pairs.push((spec, type_attr));
        }
    }
    pairs
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

/// 构建完整的模块执行脚本，内联所有依赖。入口（[`compile_module_script`]）与依赖
/// IIFE（[`build_dep_iife`]）共享的统一发射体。
///
/// s345 两遍扫描（导出活绑定 + import 提升 + TDZ 面）：
/// - 首遍分类语句：import 子句收集（发射提升至体首——spec：import 声明静态提升，
///   绑定在实例化即存在，源内 import 在体尾而先导使用合法
///   https://tc39.es/ecma262/#sec-static-semantics-staticallyinfiniteloops 无关面，
///   提升 = https://tc39.es/ecma262/#sec-moduledeclarationinstantiation）；
///   本地导出（const/let/var/function/class/解构/`export {}`）收集（导出名，本地名）。
/// - 体首导出 getter：`Object.defineProperty(_exports, K, { get: () => 本地名 })`——
///   导出绑定是活绑定（读时取当前值，duplicated-imports set_foo(43) 后读 43）；
///   未初始化 let/const 闭包读自然 TDZ 抛 ReferenceError（cycle-tdz-access
///   ill-founded cycle），var/function 提升先于 getter 可调用（undefined/函数值）。
///   spec https://tc39.es/ecma262/#sec-exports。
/// - 具名/默认导入绑定安装 globalThis getter（读时穿透依赖导出活对象——语句位
///   快照只读实例化值）；命名空间导入仍绑定导出对象本身（属性经 getter 活读）。
fn build_module_script(
    source: &str,
    url: &str,
    registry: &ModuleRegistry,
    async_body: bool,
    credentials: &str,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Result<String, ScriptError> {
    build_module_script_inner(source, url, registry, async_body, false, credentials, visited, path)
}

/// s360（html-semantics-compat import.defer 面）：入口模块的**延迟求值**发射变体——
/// 依赖图急切求值（`depsPromise`，async 承载 TLA 依赖 await 归一）、入口体延迟
/// （`runBody`，首次命名空间属性访问触发、恰好一次）。产物 IIFE 返回
/// `{ exports, depsPromise, runBody }` 记录对象（非导出面），由 shim
/// `__zw_load_module_deferred` 求值记账、`__zw_make_defer_facade` 建 Proxy 门面。
/// 体延迟作用域近似：导出声明经关键词剥离提升为 IIFE 顶层 `var` 赋值（getter 闭包
/// 目标）、`export function/class` 声明提前到顶层（函数提升即 spec 瞬时初始化语义；
/// class 提前求值为近似，语料无钉面 FIXME）、提升 import 发射的 `var` 剥离为赋值
/// （deps fn 局部 `var` 不可见顶层 getter/体）。TLA 入口体 runBody 为 async——属性
/// 访问在体 settle 前读到 undefined（spec 延迟命名空间对 TLA 模块的访问挂起语义未
/// 建模，FIXME，语料无钉面——tla 钉面入口体本身无 TLA）。
/// https://tc39.es/proposal-defer-import-eval/
pub fn build_module_script_deferred(
    source: &str,
    url: &str,
    registry: &ModuleRegistry,
    credentials: &str,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Result<String, ScriptError> {
    let async_body = module_source_has_top_level_await(source);
    build_module_script_inner(source, url, registry, async_body, true, credentials, visited, path)
}

/// [`build_module_script`] / [`build_module_script_deferred`] 共享发射核。
/// `deferred = true` 时 deps/body 拆分发射（见上方变体注记）；`false` 逐字节同旧路径。
#[allow(clippy::too_many_arguments)] // s360：deferred 穿参 +1 形参（8 参——既有图收集签名面）
fn build_module_script_inner(
    source: &str,
    url: &str,
    registry: &ModuleRegistry,
    async_body: bool,
    deferred: bool,
    credentials: &str,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
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
    // s338：本模块 credentials mode 的 IIFE 局部绑定——动态 import() 重写发射读此
    // 局部量穿 `__zw_dynamic_import`（spec：动态 import 请求 credentials = 定义模块
    // 脚本元素的 credentials mode；依赖 IIFE 同发射继承入口 mode，静态图取回同
    // mode 语义对齐）。值恒内部常量（same-origin/include），json_stringify 仅卫生。
    // https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
    output.push_str(&format!("  var _importCred = {};\n", json_stringify(credentials)));
    // s254：import.meta.resolve（HTML #import-metadata——入参 ToString 后经宿主回调走
    // resolve a module specifier 全算法；本模块基址穿入）。恒发射：无 map 时即纯
    // URL 解析（spec 语义），无 map 页面行为面不受扰。
    // https://html.spec.whatwg.org/multipage/webappapis.html#dom-import-meta-resolve
    output.push_str(
        "  _importMeta.resolve = function (spec) { return __zwImportMapResolve(String(spec), _importMeta.url); };\n",
    );
    // 入口自导入（github.com environment 入口真实形态 `import*as i from"./environment-*.js"`
    // 后 `e.C(i)`，2026-10-08 home-reload 线上证据）：入口自身作为依赖重入时绑定到自身
    // _exports 活对象——不得再内联一份入口体（内层拷贝执行期 typeof 守卫落空 → 空 stub →
    // webpack C(ns) 读 `__rspack_esm_ids.length` 抛 TypeError）。
    output.push_str(&format!("  var _mod_{} = _exports;\n", safe_ident(url)));
    visited.insert(url.to_string());

    // s345 首遍：分类语句——import 子句收集（体首提升）、本地导出名收集（体首 getter）。
    let mut import_clauses: Vec<&str> = Vec::new();
    let mut export_getters: Vec<(String, String)> = Vec::new();
    let stmts = split_statements(source);
    for stmt in &stmts {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(clause) = strip_import_keyword(trimmed) {
            import_clauses.push(clause);
            continue;
        }
        if let Some(clause) = strip_export_keyword(trimmed) {
            for (exported, local) in local_export_bindings(clause) {
                export_getters.push((exported, local));
            }
        }
    }
    // s345 体首导出 getter（先于提升 import——循环依赖中依赖体经 import 绑定或
    // 自导入活对象读到的导出必须已就位，未初始化 let/const 自然 TDZ 抛 ReferenceError）。
    for (exported, local) in &export_getters {
        output.push_str(&export_getter_line(exported, local));
    }
    // s360（deferred）：提升发射拆分——deps 段（急切）与体段（延迟）分缓冲装配。
    // deps fn 内提升发射**原样内联**（`var _mod_…` 为 deps fn 局部——import 绑定
    // getter 是 globalThis defineProperty，闭包捕获 fn 局部量即可见；体经 getter 读
    // 绑定，无须顶层可见性）。命名空间导入绑定（`var ns = _mod_x;` 体裸引用形）
    // 补 globalThis getter（命名空间绑定 spec 不可变，getter 恒等透传）。
    let mut hoisted = String::new();
    let mut body = String::new();
    let mut top_decls = String::new();
    // s345 提升 import：全部 import 语句发射于体首（源内声明序），先于普通语句——
    // 依赖图急切求值（prelude postorder）或体首 memoized 发射承接，绑定先于使用。
    // s360（deferred）：deps fn 为 async（TLA 依赖 await 归一）→ 提升发射恒按
    // enclosing_async=true。
    for clause in &import_clauses {
        let mut emitted = transform_import(
            clause,
            url,
            registry,
            visited,
            path,
            if deferred { true } else { async_body },
            credentials,
        )?;
        if deferred {
            emitted.push_str(&namespace_binding_getters(&emitted));
        }
        hoisted.push_str(&emitted);
    }
    // s360（deferred）：顶层预声明——导出 getter 闭包目标（体声明臂剥离关键词后的
    // 赋值落点）。恒 `var`（TDZ 近似同 eager 形态：体运行前读导出得 undefined 非
    // ReferenceError，见 export_getter_line 注记）。
    if deferred {
        output.push_str("  var __zwBodyRan = false;\n");
        let mut promoted: Vec<&str> = export_getters.iter().map(|(_, l)| l.as_str()).collect();
        promoted.sort_unstable();
        promoted.dedup();
        for local in &promoted {
            if is_plain_ident(local) {
                output.push_str(&format!("  var {local};\n"));
            }
        }
    }

    // 次遍：非 import 语句按源序发射（export 声明只发裸声明，接线由体首 getter 承担）。
    for stmt in &stmts {
        let trimmed = stmt.trim();
        if trimmed.is_empty() {
            continue;
        }
        if strip_import_keyword(trimmed).is_some() {
            continue;
        }
        if let Some(clause) = strip_export_keyword(trimmed) {
            let emitted = transform_export(clause, url, registry, visited, path, async_body, credentials)?;
            if deferred {
                // s360：`export function/class` 声明提升到顶层（函数提升 = spec 瞬时
                // 初始化；class 提前求值近似，语料无钉面）。const/let/var 声明臂
                // 剥离关键词为顶层 var 赋值（getter 目标可见性，见上方预声明）。
                let t = emitted.trim_start();
                if t.starts_with("function ") || t.starts_with("class ") {
                    top_decls.push_str(&emitted);
                    continue;
                }
                if let Some(rest) = t
                    .strip_prefix("const ")
                    .or_else(|| t.strip_prefix("let "))
                    .or_else(|| t.strip_prefix("var "))
                {
                    body.push_str(&assignment_from_decl(rest));
                    continue;
                }
            }
            // s360：体缓冲统一（提升 import 后追加）——原 eager 直写 output 会把
            // 导出/普通语句插到提升 import 之前，体序倒置（e.C(self) 实证红）。
            body.push_str(&emitted);
        } else {
            // 普通语句：替换 import.meta 与动态 import()（s360：defer 臂同扫）
            let s = rewrite_dynamic_imports(&trimmed.replace("import.meta", "_importMeta"));
            let line = format!("  {s};\n");
            body.push_str(&line);
        }
    }

    if deferred {
        // s360：顶层声明（export function/class 提升——见次遍剥离注记）。
        output.push_str(&top_decls);
        // deps fn（急切依赖求值——async 承载 TLA 依赖 await 归一）+ 体 fn
        //（延迟、恰好一次——__zwBodyRan 守卫）。记录对象由 shim
        // `__zw_load_module_deferred` 求值记账、`__zw_make_defer_facade` 建门面。
        output.push_str("  var __zwDepsPromise = (async function() {\n");
        output.push_str(&hoisted);
        output.push_str("  })();\n");
        if async_body {
            output.push_str("  var __zwRunBody = async function () {\n");
        } else {
            output.push_str("  var __zwRunBody = function () {\n");
        }
        output.push_str("    if (__zwBodyRan) { return; }\n    __zwBodyRan = true;\n");
        output.push_str(&body);
        output.push_str("  };\n");
        output.push_str("  return { exports: _exports, depsPromise: __zwDepsPromise, runBody: __zwRunBody };\n");
        output.push_str("})()\n");
        return Ok(output);
    }

    output.push_str(&hoisted);
    output.push_str(&body);
    output.push_str("  return _exports;\n");
    output.push_str("})()\n");
    Ok(output)
}

/// s360：命名空间导入绑定（`import * as ns from …`）的体可见性 getter——deferred
/// 发射把提升 import 内联进 deps fn（`var ns = _mod_x;` 为 fn 局部），体经裸标识符
/// 引用 `ns` 须落 globalThis getter（命名空间绑定 spec 不可变 → getter 恒等透传
/// dep 导出面）。只匹配整行紧凑形态 `  var NAME = _mod[_ref]_ID;`（deps fn 局部
/// 发射自 Shape；memoized 多行表达式内部不匹配——整行边界守卫）。
fn namespace_binding_getters(emitted: &str) -> String {
    let mut out = String::new();
    for line in emitted.lines() {
        let t = line.trim_start();
        if !line.starts_with("  var ") {
            continue;
        }
        let Some((name, rhs)) = t
            .strip_prefix("var ")
            .map(|r| r.strip_suffix(';').unwrap_or(r))
            .and_then(|r| r.split_once(" = "))
        else {
            continue;
        };
        let name = name.trim();
        let rhs = rhs.trim();
        let is_dep_local = rhs.starts_with("_mod_") || rhs.starts_with("_modref_");
        if is_dep_local
            && !name.starts_with("_mod")
            && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
        {
            out.push_str(&format!(
                "  try {{ Object.defineProperty(globalThis, {}, {{ get: function() {{ return {rhs}; }}, configurable: true }}); }} catch (_zwImpErr) {{}}\n",
                json_stringify(name)
            ));
        }
    }
    out
}

/// s360：transform_export 声明臂产物（`value = tlaValue;\n` / `{a, b} = x;` 剥关键词
/// 后余段）转顶层赋值语句——解构对象形态语句位 `{` 为块语句歧义，加括号成赋值表达
/// 式（spec Statement vs ExpressionStatement 歧义消除）。
fn assignment_from_decl(rest: &str) -> String {
    let t = rest.trim_end();
    let t = t.strip_suffix(';').unwrap_or(t);
    if t.trim_start().starts_with('{') {
        return format!("  ({t});\n");
    }
    format!("  {t};\n")
}

/// 为依赖模块构建内联的 IIFE（返回其导出对象）。
/// s269：依赖源含顶层 await（TLA，[`module_source_has_top_level_await`]）时以
/// **async IIFE** 包裹——`await` 落在同步 IIFE 内是 SyntaxError（module/
/// module-import-referrer.js `export const {…} = await import(…)` 实证，页脚
/// 整案零注册 Timeout）。async IIFE 求值产物是 Promise；消费方 await 归一
///（[`await_prefix`]），同步消费上下文拿 Promise 本体（FIXME，语料无钉面）。
/// s345：体发射与入口共享 [`build_module_script`]（导出 getter 体首 + import 提升
/// 同型——依赖体同样被循环依赖方经活对象读导出，TDZ/活绑定面一致）。
fn build_dep_iife(
    specifier: &str,
    registry: &ModuleRegistry,
    credentials: &str,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Result<String, ScriptError> {
    let source = registry
        .get(specifier)
        .ok_or_else(|| ScriptError::RuntimeError(format!("Module not found: {specifier}")))?
        .to_string();
    // s361：静态 defer import 声明传染 async（IIFE 须 async 承接 defer 目标 TLA
    // 的 await 归一发射——sync IIFE 中 `await memoized` 是语法错误且死锁时序不成立：
    // B 的动态 import(A) 触发的 A eager 编译若为 sync 形态，体会在 deps 挂起前同步
    // 跑（cyclic-tla-defer-eval flag 断言红态实证面）。
    let is_async = module_source_has_top_level_await(&source) || source_has_deferred_import_decl(&source);
    build_module_script(&source, specifier, registry, is_async, credentials, visited, path)
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
/// `bool` = 依赖源含顶层 await（TLA）→ IIFE 以 async 包裹，求值产物是
/// **Promise**（resolve 为导出对象）——消费方须 await（仅在 async 消费
/// 上下文发射；同步消费方拿到 Promise 本体，绑定近似 undefined——spec
/// 的 TDZ 语义未建模，FIXME 挂账，语料无钉面）。
enum DepInline {
    /// 首次内联：IIFE 源 + TLA 态 + DFS 祖先链（入口→导入者路径，含导入者不含自身——
    /// s345：记忆化发射失败记账传播面；prelude 不再急切求值入口图后此链由内联发射承担）。
    Fresh(String, bool, Vec<String>),
    /// 循环/菱形重入：safe 变量名（首次内联的 `_mod_{safe}`）+ 记忆化缓存键（s345
    /// 跨 IIFE 菱形回落面——首份内联在兄弟 IIFE 作用域时 `typeof` 守卫不可见）。
    Visited(String, bool, String),
}

impl DepInline {
    /// 依赖 IIFE 是否 Promise 值（TLA 依赖）。
    fn is_async(&self) -> bool {
        match self {
            DepInline::Fresh(_, a, _) | DepInline::Visited(_, a, _) => *a,
        }
    }

    /// Fresh 内联的失败记账祖先链（[`memoized_dep_expr`] 第 3 参）。
    fn ancestors(&self) -> &[String] {
        match self {
            DepInline::Fresh(_, _, ancestors) => ancestors,
            DepInline::Visited(..) => &[],
        }
    }

    /// 求值为该模块导出对象的表达式。Visited 用 `typeof` 守卫：变量在当前作用域可见
    /// （自导入/循环重入——目标 IIFE 在词法外层链上）→ 共享同一导出实例；深层菱形
    /// 首份内联在兄弟 IIFE 作用域不可见 → s345 回落 `__moduleCache[key]`（首份内联
    /// 先行已完成，命中同一实例——imports.html 三元素菱形 INC_A === INC_AB_A 判据）；
    /// 仍缺席（求值中环经外层未初始化 var 的退化形）→ 空对象（与旧占位行为一致）。
    fn exports_expr(&self) -> String {
        match self {
            DepInline::Fresh(code, _, _) => code.clone(),
            DepInline::Visited(safe, _, key) => {
                let key_js = json_stringify(key);
                format!(
                    "(typeof _mod_{safe} !== 'undefined' ? _mod_{safe} : (globalThis.__moduleCache[{key_js}] !== undefined ? globalThis.__moduleCache[{key_js}] : (function(){{return {{}};}})()))"
                )
            }
        }
    }
}

/// TLA 依赖在 async 消费上下文的 await 前缀（`memoized_dep_expr` 对 TLA 依赖缓存
/// Promise——await 归一 Promise/导出对象两态；同步依赖发射串逐字节不变，零回归面）。
fn await_prefix(dep_is_async: bool, enclosing_async: bool) -> &'static str {
    if dep_is_async && enclosing_async { "await " } else { "" }
}

fn inline_dep_once(
    specifier: &str,
    registry: &ModuleRegistry,
    credentials: &str,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
) -> Result<DepInline, ScriptError> {
    if !visited.contains(specifier) {
        visited.insert(specifier.to_string());
        let source = registry
            .get(specifier)
            .ok_or_else(|| ScriptError::RuntimeError(format!("Module not found: {specifier}")))?;
        let is_async = module_source_has_top_level_await(source);
        // s345：祖先链 = 入口→本导入者路径（含导入者不含自身，与 postorder_visit 语义
        // 一致——依赖体失败时其记忆化 catch 记账全链，中间模块记录同持失败对象，spec
        // 求值错误中间缓存）。自身入链后递归构建，依赖的内联发射携带含自身的链。
        // https://tc39.es/ecma262/#sec-moduleevaluation
        let ancestors = path.clone();
        path.push(specifier.to_string());
        let iife = build_dep_iife(specifier, registry, credentials, visited, path)?;
        path.pop();
        return Ok(DepInline::Fresh(iife, is_async, ancestors));
    }
    // 已访问（循环/菱形重入）→ 引用首份内联导出，不重复内联。
    let is_async = registry
        .get(specifier)
        .map(module_source_has_top_level_await)
        .unwrap_or(false);
    Ok(DepInline::Visited(
        safe_ident(specifier),
        is_async,
        specifier.to_string(),
    ))
}

/// 转换 import 声明。
/// 转换 import 语句（`import` 关键字已由 [`strip_import_keyword`] 剥离，传入子句）。
/// 同时支持常规与压缩（无关键字后空格）形态。
fn transform_import(
    clause: &str,
    importer_url: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
    enclosing_async: bool,
    credentials: &str,
) -> Result<String, ScriptError> {
    // s361：静态 deferred namespace 归一（见 [`strip_defer_prefix`]）。目标模块
    // 求值语义：提案明文带 TLA 的模块不被 defer——图急切 evaluate 阶段照常跑体至
    // TLA 挂起（tc39 issue #83 的循环 TLA 死锁时序即建立在此急切求值上），名空间
    // 臂 await 归一发射天然承接；sync 目标经本臂 eager 内联（体提前求值，spec
    // 违背——deferred record 形态与动态面门面统一为后续切片，语料无钉面 FIXME）。
    // https://tc39.es/proposal-defer-import-eval/
    let clause = strip_defer_prefix(clause);
    // import 'module' — 副作用导入
    if clause.starts_with('\'') || clause.starts_with('"') || clause.starts_with('`') {
        let raw_specifier = extract_string_literal(clause.split(';').next().unwrap_or(clause).trim())?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry)?;
        let safe = safe_ident(&specifier);
        let dep = inline_dep_once(&specifier, registry, credentials, visited, path)?;
        return match &dep {
            // R5182：Fresh 内联包记忆化——与 prelude 急切求值共享实例，不重复执行模块体。
            DepInline::Fresh(code, _, _) => Ok(format!(
                "  var _mod_{safe} = {}{};\n",
                await_prefix(dep.is_async(), enclosing_async),
                memoized_dep_expr(&specifier, code, dep.ancestors())
            )),
            DepInline::Visited(..) => Ok(String::new()),
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
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry)?;
        // R3398：防循环/菱形 import 无限递归（仅首次访问时内联依赖 IIFE；重入 → 引用首份
        // 导出，避免 a↔b 循环致栈溢出 abort）。镜像 import 'm' 副作用导入的 visited 守卫。
        let safe = safe_ident(&specifier);
        let dep = inline_dep_once(&specifier, registry, credentials, visited, path)?;
        let exports_var = match &dep {
            DepInline::Fresh(..) => format!("_mod_{safe}"),
            DepInline::Visited(..) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code, _, _) => {
                // R5182：Fresh 内联包记忆化（与 prelude / 其他元素共享实例）。
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    memoized_dep_expr(&specifier, code, dep.ancestors())
                ));
            }
            DepInline::Visited(..) => {
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    dep.exports_expr()
                ));
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
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry)?;
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
        let dep = inline_dep_once(&specifier, registry, credentials, visited, path)?;
        let exports_var = match &dep {
            DepInline::Fresh(..) => format!("_mod_{safe}"),
            // 重入：首份导出在既有 `_mod_{safe}` 中；守卫引用存入独立名避免遮蔽外层声明
            DepInline::Visited(..) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code, _, _) => {
                // R5182：Fresh 内联包记忆化（与 prelude / 其他元素共享实例）。
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    memoized_dep_expr(&specifier, code, dep.ancestors())
                ));
            }
            DepInline::Visited(..) => {
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    dep.exports_expr()
                ));
            }
        }
        result.push_str(&install_import_binding_getters(bindings, &exports_var));
        return Ok(result);
    }

    // import X from 'module' — 默认导入
    if let Some((name, spec_part)) = split_from_clause(clause) {
        let name = name.trim();
        let raw_specifier = extract_import_specifier_from_rest(spec_part)?;
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry)?;
        ensure_module_export(&specifier, "default", registry)?;
        let safe = safe_ident(&specifier);
        let dep = inline_dep_once(&specifier, registry, credentials, visited, path)?;
        let exports_var = match &dep {
            DepInline::Fresh(..) => format!("_mod_{safe}"),
            DepInline::Visited(..) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code, _, _) => {
                // R5182：Fresh 内联包记忆化（与 prelude / 其他元素共享实例）。
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    memoized_dep_expr(&specifier, code, dep.ancestors())
                ));
            }
            DepInline::Visited(..) => {
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    dep.exports_expr()
                ));
            }
        }
        result.push_str(&import_binding_getter_line(name, &format!("({exports_var}).default")));
        return Ok(result);
    }

    Ok(String::new())
}

fn resolve_registered_specifier(
    specifier: &str,
    importer_url: &str,
    registry: &ModuleRegistry,
) -> Result<String, ScriptError> {
    if registry.get(specifier).is_some() {
        return Ok(specifier.to_string());
    }
    // 注册表直击未命中时先过页面 import map（HTML 规范 resolve a module specifier
    // 的 map 阶段）：Resolved 按映射 URL 继续；Blocked（null 条目）→ 解析错误
    // 终止全部回退（s259——URL 形态说明符不得落回 asURL/相对解析继续抓取）；
    // Miss 按原相对路径解析继续。键迭代顺序为 serde_json Map 排序序（非
    // JSON 插入序），仅嵌套尾斜杠前缀键 / 多 scope 重叠时可观察（见 import_map.rs
    // 模块注释）。
    // https://html.spec.whatwg.org/multipage/webappapis.html#resolve-a-module-specifier
    if let Some(map) = registry.import_map() {
        match map.resolve(specifier, importer_url) {
            ImportMapResolution::Resolved(mapped) => return Ok(mapped),
            ImportMapResolution::Blocked => {
                return Err(ScriptError::RuntimeError(format!(
                    "Module specifier '{specifier}' is blocked by a null entry in the import map"
                )));
            }
            ImportMapResolution::Miss => {}
        }
    }
    let Ok(base) = url::Url::parse(importer_url) else {
        return Ok(specifier.to_string());
    };
    let Ok(mut resolved) = base.join(specifier) else {
        return Ok(specifier.to_string());
    };
    resolved.set_fragment(None);
    let resolved = resolved.to_string();
    if registry.get(&resolved).is_some() {
        Ok(resolved)
    } else {
        Ok(specifier.to_string())
    }
}

fn ensure_module_export(specifier: &str, name: &str, registry: &ModuleRegistry) -> Result<(), ScriptError> {
    if registry.get(specifier).is_none() {
        return Err(ScriptError::RuntimeError(format!("Module not found: {specifier}")));
    }
    match module_provides_export(specifier, name, registry, &mut HashSet::new()) {
        ExportResolution::Provides => Ok(()),
        // s255：星号歧义 = 链接期 SyntaxError（tc39 Instantiate：歧义名不进入导出
        // 面也不得导入——WPT instantiation-error-6/7 判据；webview 侧 CompileError
        // 统一映射 SyntaxError 报告面，s102 通道）。
        // https://tc39.es/ecma262/#sec-moduledeclarationlinking
        ExportResolution::Ambiguous => Err(ScriptError::CompileError(format!(
            "Module {specifier} contains conflicting star exports for name '{name}'"
        ))),
        ExportResolution::Missing => Err(ScriptError::CompileError(format!(
            "Module {specifier} does not provide an export named {name}"
        ))),
    }
}

/// s255：具名导出解析三态（tc39 ResolveExport 语义的静态近似）——`Provides` 命中、
/// `Ambiguous` 星号重导出歧义（同名经 ≥2 条星号路径可达，或子链歧义经星号传播——
/// Node 实证：7a 经 7b 的星号发现 7c 内 7d/7e 歧义，仍 link 失败）、`Missing` 未提供。
/// 循环（visited 重入）按 Missing（既有口径——自导入报 not provided）。
/// https://tc39.es/ecma262/#sec-resolveexport
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExportResolution {
    Provides,
    Ambiguous,
    Missing,
}

fn module_provides_export(
    specifier: &str,
    name: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
) -> ExportResolution {
    if !visited.insert(specifier.to_string()) {
        return ExportResolution::Missing;
    }
    let Some(source) = registry.get(specifier) else {
        return ExportResolution::Missing;
    };
    let mut star_provides = 0usize;
    let mut star_ambiguous = false;
    for statement in split_statements(source) {
        let Some(rest) = strip_export_keyword(statement.trim()) else {
            continue;
        };
        if name == "default"
            && let Some(expr) = rest.strip_prefix("default")
            && !expr.starts_with(|c: char| c.is_alphanumeric() || c == '_' || c == '$')
        {
            return ExportResolution::Provides;
        }
        for declaration in ["const ", "let ", "var ", "function ", "class "] {
            if let Some(value) = rest.strip_prefix(declaration) {
                if extract_binding_name(value) == name {
                    return ExportResolution::Provides;
                }
                // s269：解构声明绑定名（`export const { a: b } = …` 导出 b，
                // tc39 BoundName——module-import-referrer.js 钉面）。
                if extract_destructuring_export_names(value)
                    .is_some_and(|names| names.iter().any(|exported| exported == name))
                {
                    return ExportResolution::Provides;
                }
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
                    return ExportResolution::Provides;
                }
                continue;
            }
            if name != "default"
                && let Some((_, spec_part)) = split_from_clause(after)
                && let Ok(raw) = extract_import_specifier_from_rest(spec_part)
            {
                // s259：依赖说明符被 null 条目阻断 → 该星号子链按 Missing 处理
                //（提供性判定不下探；链路本身在 transform 期经 ? 终止）。
                let Ok(dependency) = resolve_registered_specifier(&raw, specifier, registry) else {
                    continue;
                };
                match module_provides_export(&dependency, name, registry, visited) {
                    ExportResolution::Provides => star_provides += 1,
                    // s255：星号子链歧义传播——spec ResolveExport 星号臂子解析为
                    // null（歧义）时整体即 null，不因其他星号路径可命中而消解
                    //（Node 24 实证：7c 内 7d/7e 歧义经 7b 星号上浮，7a 仍 link
                    // 失败，尽管 7f 直出同名）。
                    // https://tc39.es/ecma262/#sec-resolveexport
                    ExportResolution::Ambiguous => star_ambiguous = true,
                    ExportResolution::Missing => {}
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
                if exported != name {
                    continue;
                }
                match &from_specifier {
                    None => return ExportResolution::Provides,
                    // s255：显式重导出子链歧义按未绑定继续——spec ResolveExport
                    // 显式重导出臂 resolution 为 null 时落入后续子句/星号，不就
                    // 地判歧义（与星号臂传播相对）。
                    // s259：依赖说明符被 null 条目阻断 → 该重导出链不提供该名，
                    // 继续后续子句/星号（链路本身在 transform_export 期经 ? 终止）。
                    Some(Ok(dependency)) => {
                        if module_provides_export(dependency, imported, registry, visited) == ExportResolution::Provides
                        {
                            return ExportResolution::Provides;
                        }
                    }
                    Some(Err(_)) => {}
                }
            }
        }
    }
    if star_ambiguous || star_provides >= 2 {
        ExportResolution::Ambiguous
    } else if star_provides == 1 {
        ExportResolution::Provides
    } else {
        ExportResolution::Missing
    }
}

/// 从 `from '...'` 部分提取模块标识符。
fn extract_import_specifier_from_rest(s: &str) -> Result<String, ScriptError> {
    let s = s.split(';').next().unwrap_or(s).trim();
    extract_string_literal(s)
}

/// s345：具名导入绑定 globalThis getter 安装发射（`import { a as b, c }` 每绑定一条）。
/// bare 标识符在 IIFE 局部无声明 → 作用域链落 globalThis 访问器 → 活读依赖导出。
fn install_import_binding_getters(bindings: &str, exports_var: &str) -> String {
    let inner = bindings.trim_start_matches('{').trim_end_matches('}');
    let mut result = String::new();
    for item in inner.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }
        let (src, name) = match item.split_once(" as ") {
            Some((src, alias)) => (src.trim(), alias.trim()),
            None => (item, item),
        };
        result.push_str(&import_binding_getter_line(name, &format!("{exports_var}.{src}")));
    }
    result
}

/// s345：单条导入绑定 getter 发射。安装失败（同名全局不可配置，如 `window.location`）
/// 时 bare 读回落既有全局值（降级不致命，FIXME）；无 setter → strict 下赋值抛
/// TypeError（spec 导入绑定不可赋值面近似）。
fn import_binding_getter_line(name: &str, source_expr: &str) -> String {
    format!(
        "  try {{ Object.defineProperty(globalThis, {}, {{ get: function() {{ return {source_expr}; }}, configurable: true }}); }} catch (_zwImpErr) {{}}\n",
        json_stringify(name)
    )
}

/// 转换 export 声明（`export` 关键字已由 [`strip_export_keyword`] 剥离，传入子句）。
/// 同时支持常规与压缩（无关键字后空格）形态。
fn transform_export(
    clause: &str,
    importer_url: &str,
    registry: &ModuleRegistry,
    visited: &mut HashSet<String>,
    path: &mut Vec<String>,
    enclosing_async: bool,
    credentials: &str,
) -> Result<String, ScriptError> {
    // export * [as N] from 'module'（压缩形态 export*from"m" / export*as N from"m" 同样命中）
    if let Some(after_star) = clause.strip_prefix('*') {
        let after = after_star.trim_start();
        if let Some(after_as) = after.strip_prefix("as").map(str::trim_start) {
            let Some((namespace, spec_part)) = split_from_clause(after_as) else {
                return Err(ScriptError::CompileError(format!("unsupported export: {clause}")));
            };
            let raw_specifier = extract_import_specifier_from_rest(spec_part)?;
            let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry)?;
            let safe = safe_ident(&specifier);
            let dep = inline_dep_once(&specifier, registry, credentials, visited, path)?;
            let exports_var = match &dep {
                DepInline::Fresh(..) => format!("_mod_{safe}"),
                DepInline::Visited(..) => format!("_modref_{safe}"),
            };
            let mut result = String::new();
            match &dep {
                DepInline::Fresh(code, _, _) => {
                    // s255：Fresh 内联包记忆化（star-as 同星号臂注记）。
                    result.push_str(&format!(
                        "  var {exports_var} = {}{};\n",
                        await_prefix(dep.is_async(), enclosing_async),
                        memoized_dep_expr(&specifier, code, dep.ancestors())
                    ));
                }
                DepInline::Visited(..) => {
                    result.push_str(&format!(
                        "  var {exports_var} = {}{};\n",
                        await_prefix(dep.is_async(), enclosing_async),
                        dep.exports_expr()
                    ));
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
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry)?;
        let safe = safe_ident(&specifier);
        let dep = inline_dep_once(&specifier, registry, credentials, visited, path)?;
        let exports_var = match &dep {
            DepInline::Fresh(..) => format!("_mod_{safe}"),
            DepInline::Visited(..) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code, _, _) => {
                // s255：Fresh 内联包记忆化——与 prelude 急切注册/入口体共享实例，
                // 依赖体恰好一次求值（R5182 import 侧同型；star 内联此前裸 IIFE，
                // 急切臂已缓存后仍重跑——ierr-6 got 6c/6d 双份实证）。
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    memoized_dep_expr(&specifier, code, dep.ancestors())
                ));
            }
            DepInline::Visited(..) => {
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    dep.exports_expr()
                ));
            }
        }
        result.push_str(&format!(
            // s345：星号再导出 = 委托 getter（读时穿透依赖导出活对象——语句位值拷贝
            // 只读当时值，导出侧 let 改写后消费方经星号读仍旧值）；具名导出（体首
            // getter 已在案）不被星号覆盖（spec 星号不遮蔽具名
            // https://tc39.es/ecma262/#sec-moduleexports-creategetters；同名多星号
            // 首见优先 vs 旧末见覆盖，语料无钉面 FIXME）。'default' 不透传（spec）。
            "  var _reexport_{safe} = {exports_var};\n  Object.keys(_reexport_{safe}).forEach(function(key) {{ if (key !== 'default' && !Object.prototype.hasOwnProperty.call(_exports, key)) Object.defineProperty(_exports, key, {{ get: function() {{ return _reexport_{safe}[key]; }}, configurable: true, enumerable: true }}); }});\n",
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
        let specifier = resolve_registered_specifier(&raw_specifier, importer_url, registry)?;
        let safe = safe_ident(&specifier);
        for item in before[1..end].split(',').map(str::trim).filter(|item| !item.is_empty()) {
            let imported = item.split_once(" as ").map_or(item, |(name, _)| name).trim();
            ensure_module_export(&specifier, imported, registry)?;
        }
        let dep = inline_dep_once(&specifier, registry, credentials, visited, path)?;
        let exports_var = match &dep {
            DepInline::Fresh(..) => format!("_mod_{safe}"),
            DepInline::Visited(..) => format!("_modref_{safe}"),
        };
        let mut result = String::new();
        match &dep {
            DepInline::Fresh(code, _, _) => {
                // s255：Fresh 内联包记忆化——与 prelude 急切注册/入口体共享实例，
                // 依赖体恰好一次求值（R5182 import 侧同型；star 内联此前裸 IIFE，
                // 急切臂已缓存后仍重跑——ierr-6 got 6c/6d 双份实证）。
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    memoized_dep_expr(&specifier, code, dep.ancestors())
                ));
            }
            DepInline::Visited(..) => {
                result.push_str(&format!(
                    "  var {exports_var} = {}{};\n",
                    await_prefix(dep.is_async(), enclosing_async),
                    dep.exports_expr()
                ));
            }
        }
        result.push_str(&format!("  var _reexport_{safe} = {exports_var};\n"));
        for item in before[1..end].split(',').map(str::trim).filter(|item| !item.is_empty()) {
            // s345：具名再导出 = 委托 getter（活读依赖导出；语句位值拷贝同上）
            if let Some(pos) = item.find(" as ") {
                let imported = item[..pos].trim();
                let exported = item[pos + 4..].trim();
                result.push_str(&format!(
                    "  Object.defineProperty(_exports, {exported_key}, {{ get: function() {{ return _reexport_{safe}.{imported}; }}, configurable: true, enumerable: true }});\n",
                    exported_key = json_stringify(exported)
                ));
            } else {
                result.push_str(&format!(
                    "  Object.defineProperty(_exports, {exported_key}, {{ get: function() {{ return _reexport_{safe}.{item}; }}, configurable: true, enumerable: true }});\n",
                    exported_key = json_stringify(item)
                ));
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
    // s345：声明形态只发裸声明——导出接线由体首 getter（[`local_export_bindings`] 收集）
    // 承担。声明位 `_exports.name = name` 快照赋值废止（活绑定破面 + getter-only
    // 访问器 strict 赋值 TypeError）。
    if let Some(decl) = clause.strip_prefix("const ") {
        let decl_rw = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  const {decl_rw};\n"));
    }
    if let Some(decl) = clause.strip_prefix("let ") {
        let decl_rw = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  let {decl_rw};\n"));
    }
    if let Some(decl) = clause.strip_prefix("var ") {
        let decl_rw = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  var {decl_rw};\n"));
    }
    if let Some(decl) = clause.strip_prefix("function ") {
        let decl = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  function {decl}\n"));
    }
    if let Some(decl) = clause.strip_prefix("class ") {
        let decl = rewrite_dynamic_imports(&decl.replace("import.meta", "_importMeta"));
        return Ok(format!("  class {decl}\n"));
    }

    // export { X, Y as Z } — 本地列表导出，接线由体首 getter 承担（语句位零发射）
    if clause.starts_with('{') {
        return Ok(String::new());
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
/// s116：语句以完整标识符 `from` 收尾判定——import/export-from 跨行续行守卫
/// （深度 0 下以 `from` 收尾的合法语句仅 import/export-from 形，行尾不得切分）。
fn statement_ends_with_from(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() >= 4
        && &b[b.len() - 4..] == b"from"
        && !b[..b.len() - 4]
            .last()
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'$')
}

/// s351：import/export 声明在 specifier 字面量收尾处、后续行以 `with {` 起头时不切句
///（attributes 子句续行，见 [`split_statements`] s351 注）。三重形状先验防误吞：
/// 声明关键词开头（整词）+ `'`/`"` 收尾（module specifier 必为 string literal，
/// 反引号模板形非法且未闭合模板不续行）+ 跨空白前瞻 `with` 整词 + `{`。
fn statement_continues_with_attributes(statement: &str, lookahead: &[char]) -> bool {
    let t = statement.trim_start();
    // 关键词整词门：import/export 同为 6 字节。
    if !(t.starts_with("import") || t.starts_with("export"))
        || t.as_bytes()
            .get(6)
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'$')
    {
        return false;
    }
    // specifier 收尾门：module specifier 必为 `'…'`/`"…"` string literal。
    if !matches!(t.chars().last(), Some('\'') | Some('"')) {
        return false;
    }
    // 跨空白前瞻 `with` 整词 + 非空白 `{`（`with (obj)` legacy 语句形不续行）。
    let rest = lookahead.iter().collect::<String>();
    let rest = rest.trim_start();
    if !rest.starts_with("with")
        || rest
            .as_bytes()
            .get(4)
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'$')
    {
        return false;
    }
    rest[4..].trim_start().starts_with('{')
}

/// 模块源是否含**顶层** await（TLA——函数体/对象字面量/类体内的 await 不算，
/// spec：仅模块体直接 await 是模块级求值暂停点）。
/// https://tc39.es/ecma262/#sec-moduleevaluation
/// 判定策略（轻量扫描器，**宁欠勿过**——欠检测维持现状（该面今日即红），过检测
/// 会把同步依赖 IIFE 翻成 Promise 值、同步消费方绑定静默 undefined）：
/// - 逐顶层语句（[`split_statements`]，切准后 prev 上下文按语句复位）语句内
///   上下文感知扫描：字符串/模板/注释/正则内 await 跳过（[`is_regex_literal_start`]
///   同口径）；模板插值内欠检测不计；
/// - 深度门：`{`/`(`/`[` 加深，await 仅深度 0 计——函数声明体/箭头块体/对象字面量
///   （含方法体）/类体天然在括号内被排除；
/// - `=>` 守卫：语句深度 0 出现箭头后本语句 await 一律不计（表达式体箭头
///   `async () => await x` 的 await 属函数内；跨逗号同语句欠检测可接受）；
/// - 前邻 `.`/标识符续字符（成员访问 `obj.await` / 词缀 `xawait`）不计。
///
/// 欠检测残面（文档化，均维持现状）：括号内 await（`(await p)` 形）、模板插值内、
/// 块语句 `{ await x; }`、深度 0 `=>` 后逗号续项。
fn module_source_has_top_level_await(source: &str) -> bool {
    split_statements(source)
        .iter()
        .any(|statement| statement_scan_has_top_level_await(statement))
}

/// s361：模块源含**静态** `import defer *` deferred namespace 声明（tc39
/// proposal-defer-import-eval 静态形式）。async_body 传染谓词——含该声明的模块
/// 编译为 async IIFE（求值时序与 TLA 目标 await 挂起耦合，见
/// [`transform_import`] defer 归一臂注记）。误判排除：`import defer from 'm'`
/// （default 绑定名恰为 defer——`defer` 后须 `*`）、`import deferredX`（标识符
/// 续字符）、`import.defer(` 动态形态（[`strip_import_keyword`] 对 `.` 起始返
/// None 天然排除）。
pub fn source_has_deferred_import_decl(source: &str) -> bool {
    split_statements(source).iter().any(|statement| {
        strip_import_keyword(statement.trim())
            .and_then(|clause| clause.strip_prefix("defer"))
            .is_some_and(|after| after.trim_start().starts_with('*'))
    })
}

/// 单语句扫描（字节面——UTF-8 续字节不与 ASCII 引号/反斜杠/斜杠冲突，
/// rewrite_dynamic_imports 同纪律）。
fn statement_scan_has_top_level_await(statement: &str) -> bool {
    let bytes = statement.as_bytes();
    let mut i = 0usize;
    let mut str_q: Option<u8> = None;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;
    let mut in_regex = false;
    let mut regex_in_class = false;
    let mut depth = 0usize;
    let mut arrow_guard = false;
    let mut prev: Option<u8> = None;
    let mut prev_word = String::new();
    while i < bytes.len() {
        let b = bytes[i];
        let next = bytes.get(i + 1).copied();
        if line_comment {
            if b == b'\n' {
                line_comment = false;
                prev_word.clear();
            }
            i += 1;
            continue;
        }
        if block_comment {
            if b == b'*' && next == Some(b'/') {
                block_comment = false;
                i += 2;
                continue;
            }
            i += 1;
            continue;
        }
        if in_regex {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == b'[' {
                regex_in_class = true;
            } else if b == b']' {
                regex_in_class = false;
            } else if b == b'/' && !regex_in_class {
                // 旗标字母一并消费
                while matches!(bytes.get(i + 1), Some(f) if f.is_ascii_alphabetic()) {
                    i += 1;
                }
                in_regex = false;
                prev = Some(b')');
                prev_word.clear();
            }
            i += 1;
            continue;
        }
        if let Some(q) = str_q {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == q {
                str_q = None;
                prev = Some(q);
                prev_word.clear();
            }
            i += 1;
            continue;
        }
        // 代码态
        if b == b'/' && next == Some(b'/') {
            line_comment = true;
            i += 2;
            continue;
        }
        if b == b'/' && next == Some(b'*') {
            block_comment = true;
            i += 2;
            continue;
        }
        if b == b'/' && is_regex_literal_start(prev.map(|p| p as char).as_ref(), &prev_word) {
            in_regex = true;
            regex_in_class = false;
            i += 1;
            continue;
        }
        if b == b'\'' || b == b'"' || b == b'`' {
            str_q = Some(b);
            i += 1;
            continue;
        }
        // await token（词边界 + 前邻 `.`/标识符续字符排除）；被深度门/箭头守卫
        // 排除的 token 跳过继续扫（同语句后续深度 0 await 仍可命中）。
        if bytes[i..].starts_with(b"await")
            && !matches!(prev, Some(p) if p == b'.' || is_identifier_continue_byte(p))
            && !bytes
                .get(i + 5)
                .is_some_and(|after| is_identifier_continue_byte(*after))
        {
            if arrow_guard || depth > 0 {
                prev = Some(b't');
                prev_word = "await".to_string();
                i += 5;
                continue;
            }
            return true;
        }
        match b {
            b'(' | b'[' | b'{' => {
                depth += 1;
                prev = Some(b);
                prev_word.clear();
            }
            b')' | b']' | b'}' => {
                depth = depth.saturating_sub(1);
                prev = Some(b);
                prev_word.clear();
            }
            b'=' if next == Some(b'>')
                && depth == 0
                && !matches!(prev, Some(p) if p == b'=' || p == b'<' || p == b'>' || p == b'!') =>
            {
                arrow_guard = true;
                prev = Some(b'>');
                prev_word.clear();
                i += 1;
            }
            _ => {
                if is_identifier_continue_byte(b) {
                    prev_word.push(b as char);
                } else {
                    prev_word.clear();
                }
                prev = Some(b);
            }
        }
        i += 1;
    }
    false
}

pub(crate) fn split_statements(source: &str) -> Vec<String> {
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
                    // s116：`from` 行尾续行（跨行 import/export-from，注释同界）
                    if statement_ends_with_from(statement) {
                        current.push(ch);
                    } else {
                        if !statement.is_empty() {
                            stmts.push(statement.to_string());
                        }
                        current.clear();
                    }
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
            // s116：import/export-from 跨行续行——`import { x } from\n    "spec";`
            //（WPT referrer 族书写形）不能在 `from` 行尾切语句：切成 `{ x } from` /
            // `"spec";` 两段后提取与 transform（同吃本扫描）双双漏配 import 形。
            // 深度 0 下以完整标识符 `from` 收尾的合法语句仅 import/export-from 形，
            // 续行守卫安全。
            // s351：specifier 行尾 + 后续行 with 子句续行——`import x from\n"spec"\n
            // with { type: "css" };`（css-module/redirect-relative-urls 静态 import
            // 书写形）：attributes 子句是 import/export-from 声明文法的一部分
            //（https://tc39.es/proposal-import-attributes/ ，换行不终止声明），
            // 在 specifier 行尾切句会把 with 子句独立成句——提取丢属性面（无类型
            // 取回）、transform 发射孤儿 `with { … };` 残句（非合法 JS 语句形）。
            // 守卫三条件与Gate：import/export 开头 + specifier 字面量收尾 + 前瞻
            // `with {`（`with (obj)` legacy 语句形不续行）。
            if ch == '\n'
                && (statement_ends_with_from(statement)
                    || statement_continues_with_attributes(statement, &chars[index..]))
            {
                current.push(ch);
            } else {
                if !statement.is_empty() {
                    stmts.push(statement.to_string());
                }
                current.clear();
            }
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
pub(crate) fn extract_string_literal(s: &str) -> Result<String, ScriptError> {
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

/// s345：本地导出绑定收集——export 子句（关键字已剥离）为本地声明形态
/// （const/let/var/function/class/解构/`export { a, b as c }`）时返回
///（导出名，本地名）对（声明形态两名相同；列表形态 `as` 改名）；再导出/
/// default 形态返回空（接线不在体首 getter 承担面）。
fn local_export_bindings(clause: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for prefix in ["const ", "let ", "var ", "function ", "class "] {
        if let Some(decl) = clause.strip_prefix(prefix) {
            if let Some(names) = extract_destructuring_export_names(decl) {
                for name in names {
                    pairs.push((name.clone(), name));
                }
            } else {
                let name = extract_binding_name(decl);
                if is_plain_ident(name) {
                    pairs.push((name.to_string(), name.to_string()));
                }
            }
            return pairs;
        }
    }
    // export { X, Y as Z } — 本地列表（无 from；from 形已在 strip 前分流）
    if clause.starts_with('{')
        && split_from_clause(clause).is_none()
        && let Some(end) = clause.find('}')
    {
        for item in clause[1..end].split(',').map(str::trim).filter(|i| !i.is_empty()) {
            let (local, exported) = match item.split_once(" as ") {
                Some((local, exported)) => (local.trim(), exported.trim()),
                None => (item, item),
            };
            if is_plain_ident(local) && is_plain_ident(exported) {
                pairs.push((exported.to_string(), local.to_string()));
            }
        }
    }
    pairs
}

/// s345：体首导出 getter 单条发射——getter 闭包读本地名（活绑定 + 自然 TDZ）。
/// 导出名经 json_stringify（非标识符导出名亦可）；本地名非法标识符时跳过
/// （getter 无法引用，语义回落属性缺席 undefined——与旧接线形非法名同坏）。
fn export_getter_line(exported: &str, local: &str) -> String {
    if !is_plain_ident(local) {
        return String::new();
    }
    format!(
        "  Object.defineProperty(_exports, {}, {{ get: function() {{ return {local}; }}, configurable: true, enumerable: true }});\n",
        json_stringify(exported)
    )
}

/// 标识符校验（非空、标识符组成字符）——getter 闭包本地名发射资格。
fn is_plain_ident(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// s269：解构声明的导出名提取（`export const { a, b: c, d = 1 } = …` →
/// `["a", "c", "d"]`——导出名 = 模式目标绑定名，tc39 BoundName；
/// https://tc39.es/ecma262/#prod-BindingElement）。对象模式顶层条目形态：
/// `name` / `key: name` / `name = default`（键引号形 `"k": name` 同析）；
/// 字符串感知扫描定位条目边界。嵌套模式/数组模式/rest/计算键/非法目标 →
/// None（回落 extract_binding_name 现状路径）。非模式声明 → None。
/// 语料钉面：module/module-import-referrer.js `export const { referrer:
/// referrerExternalDynamic } = await import(…)`（旧发射 `_exports.unknown =
/// unknown` 在 strict IIFE 内 ReferenceError——模块体零注册面）。
fn extract_destructuring_export_names(decl: &str) -> Option<Vec<String>> {
    let mut chars = decl.char_indices().peekable();
    if chars.peek().map(|(_, c)| *c) != Some('{') {
        return None;
    }
    chars.next();
    let mut names = Vec::new();
    let mut entry = String::new();
    let mut str_q: Option<char> = None;
    let mut escaped = false;
    let mut nested = 0usize;
    let mut closed = false;
    for (_, c) in chars {
        if let Some(q) = str_q {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                str_q = None;
            }
            entry.push(c);
            continue;
        }
        match c {
            '\'' | '"' | '`' => {
                str_q = Some(c);
                entry.push(c);
            }
            '{' | '[' | '(' => {
                nested += 1;
                entry.push(c);
            }
            '}' if nested == 0 => {
                closed = true;
                break;
            }
            '}' | ']' | ')' => {
                nested -= 1;
                entry.push(c);
            }
            ',' if nested == 0 => {
                if !entry.trim().is_empty() {
                    names.push(pattern_entry_target(entry.trim())?);
                }
                entry.clear();
            }
            _ => entry.push(c),
        }
    }
    if !closed {
        return None;
    }
    let tail = entry.trim();
    if !tail.is_empty() {
        names.push(pattern_entry_target(tail)?);
    }
    Some(names)
}

/// 解构单条目的目标绑定名：默认值剥除（首个字符串外 `=`，取左侧）→
/// `key: name` 分裂（首个字符串外 `:`，取右侧——引号键 `"k": v` 同析）→
/// 标识符校验（非空、标识符组成字符、首字符非数字）。任一步失配 → None。
fn pattern_entry_target(entry: &str) -> Option<String> {
    let mut cur = entry;
    if let Some(idx) = top_level_unquoted_char(cur, '=') {
        cur = cur[..idx].trim_end();
    }
    if let Some(idx) = top_level_unquoted_char(cur, ':') {
        cur = cur[idx + 1..].trim_start();
    }
    let target = cur.trim();
    if target.is_empty() || !is_plain_identifier(target) {
        return None;
    }
    Some(target.to_string())
}

/// 字符串感知的首个裸字符定位（供默认值剥除）。
fn top_level_unquoted_char(s: &str, needle: char) -> Option<usize> {
    let mut str_q: Option<char> = None;
    let mut escaped = false;
    for (idx, c) in s.char_indices() {
        if let Some(q) = str_q {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                str_q = None;
            }
            continue;
        }
        match c {
            '\'' | '"' | '`' => str_q = Some(c),
            c if c == needle => return Some(idx),
            _ => {}
        }
    }
    None
}

/// 标识符校验（非空、全标识符组成字符、首字符非数字——ASCII 与 Unicode
/// 字母面同 extract_binding_name 宽松口径）。
fn is_plain_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if !c.is_ascii_digit() && (c.is_alphanumeric() || c == '_' || c == '$') => {}
        _ => return false,
    }
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
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

    // s348（包装级 parse 失败记录归属）：postorder 求值序定位首个 parse 失败模块
    // （非入口键）；parse 失败优先于既有记录；无 parse 失败回落首个既有记录键
    // （求值序，旧 dep-scan 的有序版）；全无 → None（调用方回落旧记账臂）。
    #[test]
    fn test_module_parse_failure_owner_postorder_s348() {
        let mut registry = ModuleRegistry::new();
        registry.register(
            "https://z.test/root.js",
            "import './dep_ok.js';\nimport './dep_bad.js';\nlog.push(1);",
        );
        registry.register("https://z.test/dep_ok.js", "globalThis.x = 1;");
        registry.register("https://z.test/dep_bad.js", "log.push(0);\n%!#$@#$@#$@");
        let stub_parse = |unit: &str| -> Result<(), String> {
            if unit.contains("%!#$") {
                Err("Compile error: SyntaxError: Unexpected token '%'".into())
            } else {
                Ok(())
            }
        };
        // ① parse 失败优先于既有记录（recorded 只挂 root = 模拟旧入口键记账残留）：
        // postorder [dep_ok, dep_bad, root]，dep_bad 命中即返、root 记录不探测。
        let recorded: HashSet<String> = ["https://z.test/root.js".to_string()].into_iter().collect();
        let mut checks = 0usize;
        let mut stub_count = |unit: &str| {
            checks += 1;
            stub_parse(unit)
        };
        let owner = module_parse_failure_owner(Some("https://z.test/root.js"), &registry, &recorded, &mut stub_count);
        assert_eq!(
            owner.as_deref(),
            Some("https://z.test/dep_bad.js"),
            "postorder 首失败=失败模块自身键"
        );
        assert_eq!(checks, 2, "dep_ok+dep_bad 两次校验即返");
        // ② 无 parse 失败：回落首个既有记录键（求值序——dep_ok 先于 root）。
        let recorded2: HashSet<String> = ["https://z.test/dep_ok.js".to_string()].into_iter().collect();
        let mut all_ok = |_: &str| Ok::<(), String>(());
        let owner2 = module_parse_failure_owner(Some("https://z.test/root.js"), &registry, &recorded2, &mut all_ok);
        assert_eq!(
            owner2.as_deref(),
            Some("https://z.test/dep_ok.js"),
            "回落候选取求值序首记录键"
        );
        // ③ 全无 → None（调用方回落旧记账臂）。
        let owner3 =
            module_parse_failure_owner(Some("https://z.test/root.js"), &registry, &HashSet::new(), &mut all_ok);
        assert_eq!(owner3, None, "无 parse 失败无记录 → 扫描不达");
    }

    // s91（string-compilation module 面）：rewrite 的字符串/模板/注释上下文感知——
    // 字面量内的 `import(` 是 eval/Function/setTimeout 字符串的**运行时代码**，编译到
    // 全局作用域后 `_importMeta`（IIFE 局部量）不可见 → ReferenceError 探针实证。
    // 仅代码态的真实 `import(` 重写；字面量内保持原样走运行时 V8 原生宿主回调。
    #[test]
    fn test_rewrite_skips_import_inside_string_literal() {
        let r =
            rewrite_dynamic_imports("var a = \"import('./x.js')\"; var b = 'import(\\'y.js\\')'; import('./z.js');");
        assert!(
            !r.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './x.js')"),
            "双引号串内 import( 不重写：{r}"
        );
        assert!(
            !r.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, 'y.js')"),
            "单引号串（内嵌转义引号）内 import( 不重写：{r}"
        );
        assert!(
            r.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, "),
            "代码态真实 import( 照常重写：{r}"
        );
        assert_eq!(r.matches("import(").count(), 2, "字面量内两处原样保留：{r}");
    }

    // s360（import.defer 面）：Module 档 defer 臂——`import.defer(` 重写为
    // `__zw_import_defer.call({referrer, credentials}, `（发射体零引号、引用模块
    // IIFE 局部量，同 eager 臂槽位纪律）；字面量内不重写；eager `import(` 臂共存。
    #[test]
    fn test_rewrite_defer_import_module_lane_s360() {
        let r =
            rewrite_dynamic_imports("var s = \"import.defer('./lit.js')\"; import.defer('./a.js'); import('./b.js');");
        assert!(
            r.contains("__zw_import_defer.call({ referrer: _importMeta.url, credentials: _importCred }, './a.js')"),
            "代码态 import.defer( 重写：{r}"
        );
        assert_eq!(
            r.matches("import.defer").count(),
            1,
            "字面量内 import.defer( 原样保留唯一一处（走原生 parse 面）：{r}"
        );
        assert!(
            r.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './b.js')"),
            "eager import( 臂共存不扰：{r}"
        );
    }

    // s360：ClassicDeferOnly 档——classic 页面脚本只重写 `import.defer(`（→
    // __zwClassicImportDefer，helper 自读槽位）；`import(` 走 V8 原生宿主回调不重写；
    // 无匹配源逐字节原样（classic 面零漂移守卫）。
    #[test]
    fn test_rewrite_defer_import_classic_lane_s360() {
        let src = "promise_test(async () => { const ns = await import.defer('./a.js'); import('./b.js'); });";
        let r = rewrite_classic_deferred_imports(src);
        assert!(
            r.contains("__zwClassicImportDefer('./a.js')"),
            "classic import.defer( 重写：{r}"
        );
        assert!(
            r.contains("import('./b.js')"),
            "classic import( 原样保留（native 宿主回调通道）：{r}"
        );
        assert!(!r.contains("__zw_dynamic_import"), "classic 面 eager 臂禁入：{r}");
        // 无匹配：逐字节原样。
        let plain = "var x = 1; function f() { return import.meta; }";
        assert_eq!(rewrite_classic_deferred_imports(plain), plain, "无匹配源逐字节原样");
    }

    // s360：延迟求值发射形态——deps fn（async，依赖 memoized 内联 + `var` 剥离为
    // 赋值）、体 fn（runBody 守卫 + 导出声明关键词剥离为顶层赋值）、导出 getter
    // 顶层预声明、记录对象三臂（exports/depsPromise/runBody）。
    #[test]
    fn test_deferred_emission_shape_s360() {
        let mut registry = ModuleRegistry::new();
        registry.register(
            "https://z.test/entry.js",
            "import { v } from './dep.js';\nexport const value = v;\nglobalThis.__entryRan = true;",
        );
        registry.register(
            "https://z.test/dep.js",
            "globalThis.__depRan = true;\nexport const v = 42;",
        );
        let out = compile_dependency_iife_deferred("https://z.test/entry.js", &registry).unwrap();
        assert!(
            out.contains("var __zwDepsPromise = (async function() {"),
            "deps fn 急切段发射：{out}"
        );
        assert!(
            out.contains("return { exports: _exports, depsPromise: __zwDepsPromise, runBody: __zwRunBody };"),
            "记录对象三臂：{out}"
        );
        assert!(out.contains("if (__zwBodyRan) { return; }"), "体恰好一次守卫：{out}");
        // 提升发射原样内联 deps fn（var 局部 + import 绑定 globalThis getter 闭包捕获）。
        assert!(
            out.contains("var __zwDepsPromise = (async function() {\n  var _mod_"),
            "deps fn 内提升发射原样（var 局部）：{out}"
        );
        assert!(
            out.contains("Object.defineProperty(globalThis, \"v\""),
            "import 绑定 getter（deps fn 内发射，闭包捕获局部量）：{out}"
        );
        // 导出声明关键词剥离 + 顶层 var 预声明（getter 闭包目标可见性）。
        assert!(
            out.contains("  var value;\n") && out.contains("value = v;"),
            "export const 剥离为顶层 var + 赋值：{out}"
        );
        assert!(
            out.contains("Object.defineProperty(_exports, \"value\""),
            "导出 getter 照常发射：{out}"
        );
        // eager 路径不受扰（同源编译无 defer 语义产物）。
        let eager = compile_dependency_iife("https://z.test/entry.js", &registry).unwrap();
        assert!(!eager.contains("__zwDepsPromise"), "eager 发射零 defer 产物：{eager}");
        assert!(eager.contains("return _exports;"), "eager 发射导出面不变：{eager}");
    }

    // s361：静态 deferred namespace（`import defer * as ns from`）——defer 前缀归一
    // 名空间臂（async TLA 目标 await 归一急切求值、活导出面绑定）；default 绑定名
    // 恰为 `defer` 与标识符前缀重叠（`deferredX`）不误归一。
    #[test]
    fn test_static_defer_namespace_arm_s361() {
        let mut registry = ModuleRegistry::new();
        registry.register(
            "https://z.test/a.js",
            "import defer * as nsB from './b.js';\nexport const value = nsB.tlaValue;",
        );
        registry.register(
            "https://z.test/b.js",
            "globalThis.__bRan = true;\nawait Promise.resolve();\nexport const tlaValue = 'tla';",
        );
        let out = compile_dependency_iife_deferred("https://z.test/a.js", &registry).unwrap();
        assert!(
            out.contains("= await (globalThis.__moduleFailures"),
            "async TLA 目标 await 归一发射（deps fn 急切求值，死锁时序面）：{out}"
        );
        assert!(
            out.contains("var nsB = _mod_") || out.contains("var nsB = _modref_"),
            "defer 归一名空间臂绑定发射：{out}"
        );
        assert!(!out.contains("import defer"), "归一后源内 defer 声明不再出现：{out}");
        // 边界：default 绑定名恰为 `defer`——不归一，走 default 导入臂。
        let mut registry2 = ModuleRegistry::new();
        registry2.register(
            "https://z.test/c.js",
            "import defer from './d.js';\nglobalThis.__got = defer;",
        );
        registry2.register("https://z.test/d.js", "export default 7;");
        let out2 = compile_dependency_iife_deferred("https://z.test/c.js", &registry2).unwrap();
        assert!(
            out2.contains("Object.defineProperty(globalThis, \"defer\""),
            "default 绑定名 defer 走 default 臂（getter 名 defer）：{out2}"
        );
        // 边界：标识符前缀重叠 `deferredX`——default 绑定不误归一。
        let mut registry3 = ModuleRegistry::new();
        registry3.register(
            "https://z.test/e.js",
            "import deferredX from './d.js';\nglobalThis.__got2 = deferredX;",
        );
        registry3.register("https://z.test/d.js", "export default 9;");
        let out3 = compile_dependency_iife_deferred("https://z.test/e.js", &registry3).unwrap();
        assert!(
            out3.contains("Object.defineProperty(globalThis, \"deferredX\""),
            "deferredX 绑定不误归一：{out3}"
        );
    }

    // s361：静态 defer 声明传染 async IIFE——eager 编译（build_dep_iife /
    // compile_module_script）对含 `import defer *` 的源以 async 包裹（承接 defer
    // 目标 TLA 的 await 归一发射；sync 形态下 cyclic 页 A eager 编译体提前跑红态面）。
    #[test]
    fn test_defer_import_async_contagion_s361() {
        assert!(
            source_has_deferred_import_decl("import defer * as ns from './b.js';"),
            "标准静态形态命中"
        );
        assert!(
            source_has_deferred_import_decl("import defer*as ns from './b.js';"),
            "压缩形态命中"
        );
        assert!(
            !source_has_deferred_import_decl("import defer from './d.js';"),
            "default 绑定名 defer 不命中"
        );
        assert!(
            !source_has_deferred_import_decl("import deferredX from './d.js';"),
            "标识符前缀重叠不命中"
        );
        assert!(
            !source_has_deferred_import_decl("import.defer('./b.js');"),
            "动态形态不命中"
        );
        let mut registry = ModuleRegistry::new();
        registry.register(
            "https://z.test/a.js",
            "import defer * as nsB from './b.js';\nexport const value = nsB.tlaValue;",
        );
        registry.register(
            "https://z.test/b.js",
            "globalThis.__bRan = true;\nawait Promise.resolve();\nexport const tlaValue = 'tla';",
        );
        // eager 编译传染：A 无自身 TLA，但含 defer 声明 → async IIFE。
        let eager = compile_dependency_iife("https://z.test/a.js", &registry).unwrap();
        assert!(
            eager.starts_with("(async function() {"),
            "含 defer 声明的 eager 编译传染 async 包裹：{eager}"
        );
        assert!(
            eager.contains("= await (globalThis.__moduleFailures"),
            "async 包裹下 await 合法（deps 挂起时序成立面）：{eager}"
        );
        // 非 defer 源零传染（回归守卫）。
        let mut registry4 = ModuleRegistry::new();
        registry4.register("https://z.test/plain.js", "globalThis.__p = 1;");
        let plain = compile_dependency_iife("https://z.test/plain.js", &registry4).unwrap();
        assert!(
            plain.starts_with("(function() {"),
            "无 defer 声明源保持 sync IIFE（零传染回归面）：{plain}"
        );
    }

    #[test]
    fn test_rewrite_skips_import_suffix_of_identifier() {
        // s267（repeated-import 页探针实证）：`import(` 是标识符后缀时（
        // `removeScriptReimport()`、`x.myimport(y)` 词法非法仅取合法形）为普通
        // 标识符引用，非 import 关键字——mid-identifier 误重写发射
        // `removeScriptRe__zw_dynamic_import...` → 定义/调用分裂 ReferenceError。
        // 前邻非标识符续字符（`(`、空白、运算符）的真实 import( 照常重写。
        let r = rewrite_dynamic_imports("removeScriptReimport(); myImport(z); $import(q); import('./real.js');");
        assert!(r.contains("removeScriptReimport()"), "标识符后缀原样保留: {r}");
        assert!(
            r.contains("myImport(z)") && r.contains("$import(q)"),
            "含 import 前缀的标识符调用原样保留: {r}"
        );
        assert!(
            r.contains(
                "__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './real.js')"
            ),
            "前邻空白的真实 import( 照常重写: {r}"
        );
        assert_eq!(r.matches("__zw_dynamic_import").count(), 1, "恰一处重写: {r}");
        // 模板表达式区内同守卫（`${ removeScriptReimport() }` 不误重写）。
        let t = rewrite_dynamic_imports("var t = `${ removeScriptReimport() } ${ import('./b.js') }`;");
        assert!(
            t.contains("removeScriptReimport()"),
            "模板表达式区内标识符后缀原样保留: {t}"
        );
        assert!(
            t.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './b.js')"),
            "模板表达式区内真实 import( 照常重写: {t}"
        );
    }

    #[test]
    fn test_rewrite_rewrites_import_inside_template_expression_only() {
        // 模板文本区不重写；`${}` 表达式区照常重写（含嵌套对象花括号深度）。
        let r = rewrite_dynamic_imports("var t = `text import('./a.js') ${ {n: 1}.n } ${ import('./b.js') }`;");
        assert!(
            !r.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './a.js')"),
            "模板文本区内不重写：{r}"
        );
        assert!(
            r.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './b.js')"),
            "模板表达式区内照常重写：{r}"
        );
    }

    #[test]
    fn test_rewrite_skips_import_inside_comments() {
        let r = rewrite_dynamic_imports("// import('./c.js')\n/* import('./d.js') */ import('./e.js');");
        assert!(
            !r.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './c.js')")
                && !r.contains(
                    "__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './d.js')"
                ),
            "行/块注释内不重写：{r}"
        );
        assert!(
            r.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './e.js')"),
            "注释后代码态照常重写：{r}"
        );
    }

    #[test]
    fn test_rewrite_survives_multibyte_characters() {
        // s91 返工钉：逐字节 `source[i..i+1]` 切进多字节字符 panic（WPT 语料
        // module 页含 'ś' 实证，runner 整批 rc=101）——拷贝按整字符步进。
        let r = rewrite_dynamic_imports(r#"var s = 'zażółć import("x.js") gęślą'; import('./y.js'); // importé"#);
        assert!(
            r.contains("zażółć") && r.contains("gęślą") && r.contains(r#"import("x.js")"#),
            "多字节字符串原样保留：{r}"
        );
        assert!(!r.contains(r#"_importMeta.url }, "x.js")"#), "串内 import( 未重写：{r}");
        assert!(
            r.contains("__zw_dynamic_import.call({ referrer: _importMeta.url, credentials: _importCred }, './y.js')"),
            "代码态照常重写：{r}"
        );
        assert!(r.ends_with("// importé"), "多字节行注释原样保留：{r}");
    }

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
            resolve_registered_specifier("react", "https://github.com/page", &reg).unwrap(),
            "https://assets.test/react.js"
        );
    }

    #[test]
    fn test_without_import_map_bare_specifier_unchanged() {
        let mut reg = ModuleRegistry::new();
        reg.register("https://github.com/react.js", "export default 1;");
        // 无 map：裸说明符按相对路径解析未命中 → 原样返回（既有行为）。
        assert_eq!(
            resolve_registered_specifier("react", "https://github.com/page", &reg).unwrap(),
            "react"
        );
        // 相对路径命中注册模块仍走原路径（无 map 回归保护）。
        assert_eq!(
            resolve_registered_specifier("./react.js", "https://github.com/page", &reg).unwrap(),
            "https://github.com/react.js"
        );
    }

    // P6C 回归（2026-10-09 github.com pricing 实测）：动态 import 路径的
    // `__zw_compile_module` 回调把依赖图 registry 接上本代页面 map 后，chunk 静态
    // `import*as r from"react"` 的裸说明符才能在 transform 阶段解析内联；漏接 map 时
    // `registry.get("react")` miss → "Module not found: react"（回调吞为空串 → 上层
    // 重试 3 次报 Loading chunk failed）。
    #[test]
    fn test_compile_dependency_iife_resolves_bare_specifier_via_import_map() {
        let base = url::Url::parse("https://github.com/").unwrap();
        let map = ImportMap::parse(r#"{"imports": {"react": "https://assets.test/react.js"}}"#, &base).unwrap();
        let chunk = r#"import*as r from"react";export const x = r.version;"#;
        let react = r#"export const version = "19.1.0";"#;

        // 无 map（P6C 前动态路径形态）：编译失败，错误点名裸说明符。
        let mut reg = ModuleRegistry::new();
        reg.register("https://github.test/chunk.js", chunk);
        reg.register("https://assets.test/react.js", react);
        let err = compile_dependency_iife("https://github.test/chunk.js", &reg).unwrap_err();
        assert!(matches!(err, ScriptError::RuntimeError(ref m) if m.contains("Module not found: react")));

        // 有 map（P6C 后）：react 按映射 URL 解析命中，依赖源码内联进 IIFE。
        let mut reg = ModuleRegistry::new();
        reg.set_import_map(map);
        reg.register("https://github.test/chunk.js", chunk);
        reg.register("https://assets.test/react.js", react);
        let iife = compile_dependency_iife("https://github.test/chunk.js", &reg).unwrap();
        assert!(iife.contains("19.1.0"));
    }

    // s259：null 条目（Blocked）终止全部回退——映射键与 URL 形态键两臂均以
    // null-entry 解析错误终止模块编译，不得降级为 Miss 后按相对解析/asURL
    // 继续内联抓取。
    // https://html.spec.whatwg.org/multipage/webappapis.html#resolve-a-module-specifier
    #[test]
    fn test_compile_dependency_iife_blocked_entry_terminates_fallbacks_s259() {
        let base = url::Url::parse("https://x.test/").unwrap();
        // 臂 1：裸键 null 条目——旧版降级 Miss → 相对解析未命中 →
        // "Module not found: blocked"。
        let map = ImportMap::parse(r#"{"imports": {"blocked": null}}"#, &base).unwrap();
        let mut reg = ModuleRegistry::new();
        reg.set_import_map(map);
        reg.register("https://x.test/chunk.js", "import'blocked';export const x = 1;");
        let err = compile_dependency_iife("https://x.test/chunk.js", &reg).unwrap_err();
        assert!(
            matches!(err, ScriptError::RuntimeError(ref m) if m.contains("blocked") && m.contains("null entry")),
            "裸键 null 条目以 null-entry 错误终止：{err:?}"
        );

        // 臂 2：URL 形态键 null 条目——URL 形态说明符命中阻断后不得落回
        // asURL/相对解析继续（旧版降级 Miss → 按 import 方基址 join 继续）。
        let map = ImportMap::parse(r#"{"imports": {"https://x.test/gone.js": null}}"#, &base).unwrap();
        let mut reg = ModuleRegistry::new();
        reg.set_import_map(map);
        reg.register(
            "https://x.test/chunk.js",
            "import'https://x.test/gone.js';export const x = 1;",
        );
        let err = compile_dependency_iife("https://x.test/chunk.js", &reg).unwrap_err();
        assert!(
            matches!(err, ScriptError::RuntimeError(ref m) if m.contains("gone.js") && m.contains("null entry")),
            "URL 形态键 null 条目终止全部回退：{err:?}"
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
    /// s87：失败原因钉 TypeError（HTML spec fetch-a-single-module-script——取回失败以
    /// TypeError reject；WPT dynamic-imports-fetch-error `promise_rejects_js(t, TypeError, …)`
    /// 判据），消息携带 spec。
    #[test]
    fn test_dynamic_import_failure_rejects_not_throws() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        // 模拟宿主取回失败：__zw_compile_module 返空 → __zw_load_module 抛 TypeError。
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
        let prelude = build_module_runtime_prelude(&ModuleRegistry::new(), None).unwrap();
        // 忠实复刻 execute_module_in_sandbox：prelude + 调用方代码单次 execute；
        // 微任务检查点在 execute 内跑 → .then 落定发生在返回前，__zw_report 已被调。
        sandbox
            .execute(&format!(
                "{prelude}\n\
                     __zw_dynamic_import('_nope').then(\n\
                       function () {{ __zw_report('fulfilled'); }},\n\
                       function (e) {{ __zw_report('rejected:' + (e && e.constructor.name) + ':' + (e && e.message)); }});\n"
            ))
            .expect("动态 import 失败不得同步 throw");
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("rejected:TypeError:Failed to fetch module: '_nope'"),
            "失败以 TypeError rejection 呈现且原因携带 spec（红态：同步 throw 后首个 execute 即 Err）"
        );
    }

    /// s363（mixed eager/defer 同 URL 记录共享——defer 先取、eager 后取臂）：prelude
    /// `__zw_load_module` 在 resolved 层命中 defer 表时**等待记录依赖图完成**
    /// （depsPromise——spec eager import 语义 = 全图求值完成）再触发体求值（runBody，
    /// 恰一次守卫内建于 deferred 编译产物）并以其导出面入 eager 缓存。HTML spec
    /// module map 单键——两视图同一记录，体不得二次求值。红态两段：修复前 eager 面
    /// 只查 `__moduleCache`，defer 先取的 URL 走宿主重新编译+求值，体跑两次；首版
    /// 钩子同步 runBody 在 deps 挂起时提前跑体，破坏 cyclic TLA 死锁时序（s361
    /// `module A's body should never run` P→F）。
    /// 两段 execute（s256 同款）：钩子 promise 链跨 execute 由后段末尾 checkpoint
    /// 排空（persistent context 存活）。
    #[test]
    fn test_mixed_eager_hits_defer_record_s363() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::with_config(crate::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::with_config(crate::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        // 宿主契约 stub：若被调说明共享钩子未命中 defer 表（红态 = IIFE 求值计数走
        // eager 面 +1，runs 与 evals 双计）。
        sandbox.register_callback(
            "__zw_compile_module",
            Box::new(|args| {
                let spec = args.first().cloned().unwrap_or_default();
                format!(
                    "{spec}\u{1f}(function(){{ globalThis.__s363e = (globalThis.__s363e|0) + 1; \
                 return {{ v: globalThis.__s363e }}; }})()"
                )
            }),
        );
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
        let prelude = module_dynamic_import_helpers_prelude();
        // 第一段：prelude + defer 表预置 + eager import 挂链（checkpoint 排首跳）。
        sandbox
            .execute(&format!(
                "{prelude}\n\
                 globalThis.__s363runs = 0;\n\
                 globalThis.__s363ran = false;\n\
                 globalThis.__zwDeferRecords = {{ 'https://z.test/m.js': {{\n\
                   exports: {{ w: 'deferred' }}, depsPromise: Promise.resolve(),\n\
                   runBody: function () {{ if (globalThis.__s363ran) return; globalThis.__s363ran = true; \
                   globalThis.__s363runs = (globalThis.__s363runs|0) + 1; }} }} }};\n\
                 globalThis.__s363p = __zw_load_module('https://z.test/m.js', 'about:blank', '', '');\n"
            ))
            .expect("eager 命中 defer 记录挂链不得同步 throw");
        // 第二段：消费链（末尾 checkpoint 排空整链）。
        sandbox
            .execute(
                "globalThis.__s363p.then(function (ns) {{\n\
                   var once = (globalThis.__s363runs === 1);\n\
                   var wv = !!(ns && ns.w === 'deferred');\n\
                   var notEager = (globalThis.__s363e === undefined);\n\
                   Promise.resolve(__zw_load_module('https://z.test/m.js', 'about:blank', '', '')).then(function (ns2) {{\n\
                     var stillOnce = (globalThis.__s363runs === 1);\n\
                     var cached = !!(ns2 && ns2 === ns);\n\
                     __zw_report([once, wv, notEager, stillOnce, cached].join(','));\n\
                   }}, function (e2) {{ __zw_report('ERR2:' + (e2 && e2.message)); }});\n\
                 }}, function (e1) {{ __zw_report('ERR1:' + (e1 && e1.message)); }});\n",
            )
            .expect("消费链不得同步 throw");
        assert_eq!(
            settled.lock().unwrap().as_deref(),
            Some("true,true,true,true,true"),
            "defer 先取记录被 eager import 等图完成后触发恰一次：runBody 计 1、导出面同源、宿主编译未被调、复 import 走缓存同引用"
        );
    }

    /// s87（html-semantics-compat dynamic-import must-reject 面）：取回失败四态身份钉。
    /// HTML spec fetch-a-single-module-script：fetch 失败**不入** module map（逐次新
    /// 错误对象——WPT fetch-error `assert_not_equals(error1, error2)`）；parse 期失败
    /// （顶层已取回、依赖图断）与 parse/求值期错误**入 map 记账**（重调同对象——WPT
    /// script-error `assert_equals(error1, error2)`）；s252：链接/instantiation 失败
    /// （tag 3）**不入** map（逐次新 SyntaxError——WPT script-error
    /// 「instantiation error」SyntaxError + differentErrorObjects 判据）。红态：全走
    /// `Module not found` 普通 Error、逐次新对象、parse/求值错误不缓存。
    #[test]
    fn test_dynamic_import_failure_identity_semantics() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        // 宿主模拟：`missing.js` 返空（①顶层取回失败）；`graphcut.js` 返 `resolved\x1f`
        // 空码标记（②依赖图断）；`parsebad.js` 返可编译 IIFE 但体为语法错；`throwing.js`
        // 返体抛错的 IIFE（③求值错误）；`linkfail.js` 返 tag 3 标记（④链接失败）。
        // IIFE 形态复刻 compile_dependency_iife 输出
        // （eval('(' + code + ')') 消费的括号包裹函数调用表达式）。
        sandbox.register_callback("__zw_compile_module", Box::new(|args| {
            let spec = args.first().map(String::as_str).unwrap_or("");
            match spec {
                "missing.js" => String::new(),
                "graphcut.js" => "https://zero.test/graphcut.js\u{1f}".to_string(),
                "parsebad.js" => "https://zero.test/parsebad.js\u{1f}(function() { return (function() { export const x =; })(); })()".to_string(),
                "throwing.js" => {
                    "https://zero.test/throwing.js\u{1f}(function() { return (function() { globalThis.__s87_eval_count = (globalThis.__s87_eval_count || 0) + 1; throw new Error('boom'); })(); })()".to_string()
                }
                "linkfail.js" => "3\u{1f}https://zero.test/linkfail.js\u{1f}Compile error: Module https://zero.test/linkfail.js does not provide an export named default".to_string(),
                _ => String::new(),
            }
        }));
        let reported: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let reported_cb = reported.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                if let Some(v) = args.first() {
                    reported_cb.lock().unwrap().push(v.clone());
                }
                String::new()
            }),
        );
        let prelude = build_module_runtime_prelude(&ModuleRegistry::new(), None).unwrap();
        // 同一 execute 内逐 case 回传（execute 间 global 不共享）。probe 串行两次
        // import 同 spec，回传 `spec:首错构造器:(首错 === 次错)`——身份语义直接落在
        // 错误对象比较上（不借缓存表反查：求值后缓存已就位，反查恒真无区分力）。
        sandbox
            .execute(&format!(
                "{prelude}\n\
                var out = [];\n\
                function probe(spec) {{\n\
                  return __zw_dynamic_import(spec).then(\n\
                    function () {{ out.push(spec + ':fulfilled'); }},\n\
                    function (e1) {{\n\
                      return __zw_dynamic_import(spec).then(\n\
                        function () {{ out.push(spec + ':second-fulfilled'); }},\n\
                        function (e2) {{ out.push(spec + ':' + e1.constructor.name + ':' + (e1 === e2)); }});\n\
                    }});\n\
                }}\n\
                Promise.all([probe('missing.js'), probe('graphcut.js'), probe('parsebad.js'), probe('throwing.js'), probe('linkfail.js')]).then(function () {{ __zw_report(out.join('|') + '|count=' + (globalThis.__s87_eval_count || 0)); }});\n"
            ))
            .expect("失败身份探针不得同步 throw");
        let lines = reported.lock().unwrap();
        let joined = lines.join("");
        let parts: Vec<&str> = joined.split('|').collect();
        assert_eq!(parts.len(), 6, "五案探针 + 求值计数全部落定：{joined}");
        // ① 顶层取回失败：TypeError × 逐次**新对象**（fetch 失败不入 module map）。
        assert_eq!(
            parts[0], "missing.js:TypeError:false",
            "取回失败恒 TypeError 且逐次新对象：{joined}"
        );
        // ② 依赖图断（宿主空码标记）：TypeError × **同对象**（parse 期失败入 map 记账）。
        assert_eq!(
            parts[1], "graphcut.js:TypeError:true",
            "图断失败恒 TypeError 且复抛同对象：{joined}"
        );
        // ③ parse 错误：SyntaxError × 同对象。
        assert_eq!(
            parts[2], "parsebad.js:SyntaxError:true",
            "parse 错误恒 SyntaxError 且复抛同对象：{joined}"
        );
        // ③' 求值错误：Error × 同对象。
        assert_eq!(
            parts[3], "throwing.js:Error:true",
            "求值错误复抛同一错误对象（入 map 记账判据）：{joined}"
        );
        // ④ 链接/instantiation 失败（tag 3）：SyntaxError × 逐次**新对象**（不入 map）。
        assert_eq!(
            parts[4], "linkfail.js:SyntaxError:false",
            "链接失败恒 SyntaxError 且逐次新对象：{joined}"
        );
        // 求值错误模块体恰好执行一次（重调不再跑体——module map 记账后零重复求值）。
        assert_eq!(parts[5], "count=1", "求值错误模块体只跑一次：{joined}");
    }

    /// s253（CSP eval 门禁互斥）：模块求值仿真豁免 eval 包装，页面级 eval 照拦。
    /// webview M2-s6 在 CSP `script-src` 无 unsafe-eval 时 per-script 装 eval/Function
    /// 包装（拦页面字符串编译 API）；prelude 的模块编译 eval 经 `__zwRealEval` 豁免
    /// （spec：module evaluation 非 eval()，script-src 只管模块 fetch——nonce 校验在
    /// 宿主 resolver）。红态：prelude 直呼 eval → 包装在场时动态 import 抛 EvalError
    /// （propagate-nonce-inline-module 失败签名）。窄豁免三层钉：①模块体求值豁免
    /// 照常落地；②模块体内嵌套 eval() 仍拦（页面语义不外溢）；③页面 eval() 照拦。
    #[test]
    fn test_dynamic_import_eval_gate_exemption_s253() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        // 模块 IIFE 复刻 compile_dependency_iife 括号包裹形：置位标记 → 体内 try 嵌套
        // eval（应被包装拦为 EvalError）→ 返回 'ok'。
        sandbox.register_callback("__zw_compile_module", Box::new(|args| {
            let spec = args.first().map(String::as_str).unwrap_or("");
            match spec {
                "mod.js" => "https://zero.test/mod.js\u{1f}(function() { return (function() { globalThis.__s253_module_ran = true; try { eval('1'); globalThis.__s253_nested_eval_ok = true; } catch (e) { globalThis.__s253_nested_eval_blocked = (e.constructor.name === 'EvalError'); } return 'ok'; })(); })()".to_string(),
                _ => String::new(),
            }
        }));
        let reported: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let reported_cb = reported.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                if let Some(v) = args.first() {
                    reported_cb.lock().unwrap().push(v.clone());
                }
                String::new()
            }),
        );
        // 复刻 webview M2-s6 包装 eval 半边（Function proxy 同形不涉本钉）：
        // `__zwCspEvalBlocked` 恒返 '1'（webview 侧为原生违例回调，此处 JS 桩同判）。
        let wrapper = "\n    globalThis.__zwCspEvalBlocked = function () { return '1'; };\n    globalThis.__zwRealEval = globalThis.eval;\n    globalThis.eval = function (code) { if (typeof __zwCspEvalBlocked === 'function' && __zwCspEvalBlocked() === '1') { throw new EvalError('EvalError: call to eval() blocked by CSP'); } return globalThis.__zwRealEval.apply(this, arguments); };\n";
        let prelude = build_module_runtime_prelude(&ModuleRegistry::new(), None).unwrap();
        sandbox
            .execute(&format!(
                "{wrapper}\n{prelude}\n\
                var out = [];\n\
                __zw_dynamic_import('mod.js').then(\n\
                  function (m) {{ out.push('resolved:' + m + ':ran=' + !!globalThis.__s253_module_ran + ':nested-blocked=' + !!globalThis.__s253_nested_eval_blocked); }},\n\
                  function (e) {{ out.push('rejected:' + e.constructor.name + ':' + e.message); }}).then(function () {{\n\
                  try {{ eval('1'); out.push('page-eval:unblocked'); }} catch (e) {{ out.push('page-eval:' + e.constructor.name); }}\n\
                  __zw_report(out.join('|'));\n\
                }});\n"
            ))
            .expect("包装在场探针不得同步 throw");
        let lines = reported.lock().unwrap();
        let joined = lines.join("");
        assert_eq!(
            joined, "resolved:ok:ran=true:nested-blocked=true|page-eval:EvalError",
            "模块编译 eval 豁免 + 模块体内嵌套 eval 照拦 + 页面 eval 照拦：{joined}"
        );
    }

    /// s254（import maps 进程内接线）：import.meta.resolve 面——transform 恒发射 +
    /// prelude 宿主桥 + 模块 IIFE 局部基址穿入三钉。宿主回调桩按 base+spec 合成
    /// 映射 URL；返空 = resolution failure（裸说明符未映射——spec 面较动态 import
    /// 的 join 降级更严）→ TypeError。红态：transform 无 resolve 发射 →
    /// `import.meta.resolve is not a function`。
    #[test]
    fn test_import_meta_resolve_s254() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        sandbox.register_callback(
            "__zw_importmap_resolve",
            Box::new(|args| {
                let spec = args.first().map(String::as_str).unwrap_or("");
                let base = args.get(1).map(String::as_str).unwrap_or("");
                match spec {
                    "bare" => format!("https://x.test/mapped?from={base}"),
                    _ => String::new(),
                }
            }),
        );
        let reported: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let reported_cb = reported.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                if let Some(v) = args.first() {
                    reported_cb.lock().unwrap().push(v.clone());
                }
                String::new()
            }),
        );
        let registry = ModuleRegistry::new();
        let prelude = build_module_runtime_prelude(&registry, None).unwrap();
        let source = "\
__zw_report('resolve=' + import.meta.resolve('bare'));\n\
try { import.meta.resolve(); } catch (e) { __zw_report('noarg=' + e.constructor.name); }";
        let module = build_module_script(
            source,
            "https://zero.test/m.js",
            &registry,
            false,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/m.js".to_string()],
        )
        .expect("模块编译不得失败");
        // transform 恒发射 resolve 面（无 map 页面即纯 URL 解析语义，行为不受扰）。
        assert!(
            module.contains("_importMeta.resolve = function"),
            "transform 恒发射 import.meta.resolve"
        );
        sandbox
            .execute(&format!("{prelude}\n{module}"))
            .expect("import.meta.resolve 探针不得同步 throw");
        let lines = reported.lock().unwrap();
        let joined = lines.join("|");
        assert_eq!(
            joined, "resolve=https://x.test/mapped?from=https://zero.test/m.js|noarg=TypeError",
            "映射命中经宿主桥返回 + IIFE 局部 _importMeta.url 穿入 + ToString 语义（no-arg → 'undefined' 裸键未映射 → TypeError）：{joined}"
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
        let prelude = build_module_runtime_prelude(&registry, None).unwrap();
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
        let prelude = build_module_runtime_prelude(&registry, None).unwrap();
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
        let prelude = build_module_runtime_prelude(&ModuleRegistry::new(), None).unwrap();
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
        let prelude = build_module_runtime_prelude(&registry, None).unwrap();
        // 单引号字符串含 `import(`（MDN 式特性检测/错误消息常见形态）+ 合法动态 import
        // （dep 缺失走 rejection）——两语句都必须存活。
        let source = "\
var s = 'import() failed';\n\
import('./dep.js').then(function () { __zw_report('dep-ok'); }, function (e) { __zw_report('slen=' + s.length + '|rej=' + (e && e.message)); });";
        let module = build_module_script(
            source,
            "https://zero.test/m.js",
            &registry,
            false,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/m.js".to_string()],
        )
        .expect("模块编译不得失败");
        sandbox
            .execute(&format!("{prelude}\n{module}"))
            .expect("单引号字符串含 import( 时改写不得产生 SyntaxError（红态：整脚本解析失败）");
        let text = reported.lock().unwrap().clone();
        let text = text.as_deref().unwrap_or_default();
        assert!(
            text.starts_with("slen=") && text.contains("rej=Failed to fetch module: './dep.js'"),
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
            let error = compile_module_script(
                source,
                "https://example.test/sw.js",
                "https://example.test/sw.js",
                &registry,
            )
            .unwrap_err();
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
            "https://example.test/entry.js",
            &registry,
        )
        .unwrap_err();
        assert!(error.to_string().contains("does not provide an export named k"));
        // 命名空间绑定本身仍可导入。
        let ok = compile_module_script(
            "import { ns } from './mid.js';",
            "https://example.test/entry.js",
            "https://example.test/entry.js",
            &registry,
        );
        assert!(ok.is_ok(), "export * as ns 应满足同名具名导入");
    }

    // s116 探针钉：export-from 纯 re-export 链（referrer insecure wrapper 语料形）——
    // 图注册语义 = raw spec 键 + 替换后体；页面经相对 raw spec 导入时链接器应沿
    // from 子句递归命中 checker 的 const 导出。
    #[test]
    fn test_export_from_reexport_chain_absolute_dep_s116() {
        let checker = "http://wpt.test:8000/html/semantics/scripting-1/the-script-element/module/resources/referrer-checker.py?name=insecure_import";
        let mut registry = ModuleRegistry::new();
        registry.register(checker, "export const referrer = 'http://wpt.test/';");
        registry.register(
            "./resources/import-referrer-checker-insecure.sub.js?name=insecure_import",
            "export { referrer } from 'http://wpt.test:8000/html/semantics/scripting-1/the-script-element/module/resources/referrer-checker.py?name=insecure_import';",
        );
        let result = compile_module_script(
            "import { referrer as v } from \"./resources/import-referrer-checker-insecure.sub.js?name=insecure_import\";\nexport default v;",
            "http://wpt.test/html/semantics/scripting-1/the-script-element/module/referrer-strict-policies.sub.html",
            "http://wpt.test/html/semantics/scripting-1/the-script-element/module/referrer-strict-policies.sub.html",
            &registry,
        );
        assert!(result.is_ok(), "export-from 链应满足具名导入: {result:?}");
    }

    // s116 回归钉：图收集提取器（with 属性变体）必须吐出 export-from 再导出依赖；
    // 正文消息字符串的 export-from 形文本不得产生幻影依赖（形态先验）。
    #[test]
    fn test_attributes_extractor_includes_export_from_reexport_s116() {
        let specs = extract_static_module_imports_with_attributes(
            "export { referrer } from 'http://wpt.test:8000/x/referrer-checker.py?name=iv';\nimport { a } from './b.js';",
        );
        assert!(
            specs
                .iter()
                .any(|(s, _)| s == "http://wpt.test:8000/x/referrer-checker.py?name=iv")
        );
        assert!(specs.iter().any(|(s, _)| s == "./b.js"));
        let phantom = extract_static_module_imports_with_attributes(
            "export const msg = 'see export { x } from \\'not-a-module.js\\' here';",
        );
        assert!(phantom.is_empty(), "幻影依赖: {phantom:?}");
    }

    // s351 回归钉：跨行 with 子句续行（css-module/redirect-relative-urls 静态 import
    // 书写形——specifier 与 with 子句分行）必须吐出完整 (spec, type) 对；独立成句的
    // `with { ... }` 不得污染前句无属性 import 的提取（type 缺省面不变）。
    #[test]
    fn test_attributes_extractor_multiline_with_clause_s351() {
        let specs = extract_static_module_imports_with_attributes(
            "import redirected from\n    \"/common/redirect.py?location=/t/redirect-target.css\"\n    with { type: \"css\" };",
        );
        assert_eq!(
            specs,
            vec![(
                "/common/redirect.py?location=/t/redirect-target.css".to_string(),
                Some("css".to_string())
            )],
            "跨行 with 子句须并入前句 import: {specs:?}"
        );
        // 无属性 import 后随无关 with 语句（legacy sloppy 形）：子句不匹配属性语法
        //（`(obj)` 括号形）→ 不吞句、属性面维持 None。
        let plain = extract_static_module_imports_with_attributes("import x from \"./a.js\"\nwith (obj) { y(); }");
        assert_eq!(plain, vec![("./a.js".to_string(), None)]);
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
            "https://example.test/module.js",
            &registry,
        )
        .unwrap();
        // R5182：首行为模块缓存初始化（幂等，s87 并入 __moduleFailures 初始化），
        // 其后的入口体保持 async IIFE 包裹（async 入口体暂不记忆化——见
        // compile_module_script FIXME）。
        assert!(compiled.starts_with(
            "globalThis.__moduleCache = globalThis.__moduleCache || {};\nglobalThis.__moduleFailures = globalThis.__moduleFailures || {};\n(async function() {"
        ));
        assert!(compiled.contains("await Promise.resolve();"));
    }

    // s269：TLA 探测矩阵——命中面（语句头 await / 解构绑定初始化 await）与
    // 排除面（async 函数体/async 块体箭头/promise_test 回调形/字符串与注释与
    // 正则内/成员访问/表达式箭头守卫）。欠检测残面（(await p) 形）钉现状值。
    #[test]
    fn test_top_level_await_detection_faces_s269() {
        assert!(module_source_has_top_level_await(
            "await import('./x.js');\nexport const y = 1;"
        ));
        // module-import-referrer.js 本体形
        assert!(module_source_has_top_level_await(
            "export const { referrer: referrerExternalDynamic } = await import('./resources/referrer-checker.py?name=x');"
        ));
        // async 函数体内 await（blob-url-workers.window.js promise_test 形）
        assert!(!module_source_has_top_level_await(
            "promise_test(async (t) => {\n  await import(moduleBlobUrl);\n  await new Promise((resolve) => {});\n});"
        ));
        // async 声明函数体
        assert!(!module_source_has_top_level_await(
            "async function f() { await g(); }\nf();"
        ));
        // 表达式体箭头（=> 守卫）
        assert!(!module_source_has_top_level_await(
            "const f = async () => await g();\nexport { f };"
        ));
        // 字符串/注释/正则内
        assert!(!module_source_has_top_level_await(
            "const s = \"await later\";\n// await the fetch\nconst re = /await/;\nexport { s, re };"
        ));
        // 成员访问
        assert!(!module_source_has_top_level_await(
            "const x = obj.await;\nexport { x };"
        ));
        // 对象字面量方法体（深度门）
        assert!(!module_source_has_top_level_await(
            "const o = { async m() { await g(); } };\nexport { o };"
        ));
        // 欠检测残面钉现状：括号内 await（(await p) 形）不计（维持同步编译）。
        assert!(!module_source_has_top_level_await(
            "const v = (await p).q;\nexport { v };"
        ));
    }

    // s269：TLA 依赖 IIFE async 包裹 + export 解构绑定接线
    //（module/module-import-referrer.js 钉形）。
    #[test]
    fn test_tla_dependency_iife_async_and_destructuring_export_s269() {
        let mut registry = ModuleRegistry::new();
        registry.register(
            "https://example.test/dep.js",
            "export { r as referrerExternalStatic } from './checker.js?name=external-static'\nexport const { referrer: referrerExternalDynamic } = await import('./checker.js?name=external-dynamic');",
        );
        registry.register(
            "https://example.test/checker.js?name=external-static",
            "export const r = 'https://example.test/';",
        );
        registry.register(
            "https://example.test/checker.js?name=external-dynamic",
            "export const r = 'https://example.test/';",
        );
        let iife = compile_dependency_iife("https://example.test/dep.js", &registry).unwrap();
        assert!(iife.starts_with("(async function() {"), "TLA 依赖须 async 包裹: {iife}");
        // s345：解构导出名接线 = 体首 getter（旧发射 `_exports.unknown = unknown` 快照
        // 在 strict IIFE 内 ReferenceError——module-import-referrer.js 实证零注册）。
        assert!(iife.contains(
            "Object.defineProperty(_exports, \"referrerExternalDynamic\", { get: function() { return referrerExternalDynamic; }"
        ));
        assert!(
            iife.contains(
                "Object.defineProperty(_exports, \"referrerExternalStatic\", { get: function() { return _reexport"
            ),
            "具名再导出委托 getter: {iife}"
        );
        // await 仍在 async IIFE 内合法（动态 import 已重写为宿主桥调用）
        assert!(iife.contains("await __zw_dynamic_import.call("));
        // 同步依赖 IIFE 逐字节不变（零回归面）
        let sync = compile_dependency_iife("https://example.test/checker.js?name=external-static", &registry).unwrap();
        assert!(sync.starts_with("(function() {"));
    }

    // s269：TLA 依赖在 async 入口的消费 await 发射（缓存 Promise 归一）。
    #[test]
    fn test_tla_dependency_consumed_with_await_in_async_entry_s269() {
        let mut registry = ModuleRegistry::new();
        registry.register(
            "https://example.test/dep.js",
            "export const { x } = await import('./x.js');",
        );
        registry.register("https://example.test/x.js", "export const x = 1;");
        let compiled = compile_module_script(
            "import { x } from './dep.js';\nglobalThis.v = x;\nawait import('./x.js');",
            "https://example.test/entry.js",
            "https://example.test/entry.js",
            &registry,
        )
        .unwrap();
        assert!(compiled.contains("(async function() {"));
        assert!(
            // s344：记忆化表达式失败表先行——await 前缀包住整个表达式（失败复抛
            // 分支在 async 上下文照旧落 promise 拒绝，语义不变）。
            compiled.contains("= await (globalThis.__moduleFailures["),
            "async 入口对 TLA 依赖消费须 await: {compiled}"
        );
    }

    // s269：解构导出名的 provide 判定（ensure_module_export 链接门不再假报
    // does not provide an export）。
    #[test]
    fn test_provides_export_destructuring_names_s269() {
        let mut registry = ModuleRegistry::new();
        registry.register(
            "https://example.test/dep.js",
            "export const { a, b: c, d = 1 } = src;\nexport const { e: f } = await import('./x.js');",
        );
        assert_eq!(
            module_provides_export("https://example.test/dep.js", "a", &registry, &mut HashSet::new()),
            ExportResolution::Provides
        );
        // `b: c` 导出名是目标 c，键 b 不导出
        assert_eq!(
            module_provides_export("https://example.test/dep.js", "c", &registry, &mut HashSet::new()),
            ExportResolution::Provides
        );
        assert_eq!(
            module_provides_export("https://example.test/dep.js", "b", &registry, &mut HashSet::new()),
            ExportResolution::Missing
        );
        assert_eq!(
            module_provides_export("https://example.test/dep.js", "d", &registry, &mut HashSet::new()),
            ExportResolution::Provides
        );
        assert_eq!(
            module_provides_export("https://example.test/dep.js", "f", &registry, &mut HashSet::new()),
            ExportResolution::Provides
        );
        assert_eq!(
            module_provides_export("https://example.test/dep.js", "e", &registry, &mut HashSet::new()),
            ExportResolution::Missing
        );
        // 非导出名仍 Missing
        assert_eq!(
            module_provides_export("https://example.test/dep.js", "src", &registry, &mut HashSet::new()),
            ExportResolution::Missing
        );
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

    // s107：with type 属性随 specifier 提取；无属性/非 type 键返 None；重复 specifier
    // 去重（首个 type 胜）；export-from 不在此面。
    #[test]
    fn test_static_import_extraction_with_type_attribute_s107() {
        assert_eq!(
            extract_static_module_imports_with_attributes(
                "import v from './module.json' with { type: \"json\" };\nimport './side.js';\nimport d from './dep.js' with { charset: \"utf-8\" };"
            ),
            [
                ("./module.json".to_string(), Some("json".to_string())),
                ("./side.js".to_string(), None),
                ("./dep.js".to_string(), None),
            ]
        );
        // 重复 specifier 去重 + 压缩形态（corpus module.html 同形）
        assert_eq!(
            extract_static_module_imports_with_attributes(
                "import a from'./m.json'with{type:\"json\"};\nimport b from './m.json' with { type: \"json\" };"
            ),
            [("./m.json".to_string(), Some("json".to_string()))]
        );
        // s116 修订：export-from 再导出是静态依赖（图收集缺它 → 纯 re-export wrapper
        // 链接期查空——referrer insecure wrapper 实证）；with 属性面暂不解析（None
        // 顺延，提取器 doc FIXME）。动态 import() 仍不在此面。
        assert_eq!(
            extract_static_module_imports_with_attributes(
                "export { x } from './m.json' with { type: \"json\" };\nconst m = import('./m.json', { with: { type: \"json\" } });"
            ),
            [("./m.json".to_string(), None)]
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
        let result = compile_module_script("import { a } from 'a';", "http://a/", "http://a/", &reg);
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
        let result = compile_module_script(
            "import { a } from 'a';\nimport { b } from 'b';",
            "http://root/",
            "http://root/",
            &reg,
        );
        assert!(result.is_ok(), "菱形 import 须编译成功: {:?}", result.err());
    }

    // R3398：默认导入旧实现把依赖 IIFE 字符串拼接 3 次 → 副作用执行 3 次。修复后求值一次。
    #[test]
    fn test_default_import_evaluated_once_r3398() {
        let mut reg = ModuleRegistry::new();
        // 依赖模块副作用：模块顶层表达式（计数）。若求值 3 次，输出会含 3 个相同语句。
        reg.register("dep", "export default 1; 2;");
        let script = compile_module_script("import d from 'dep';", "http://x/", "http://x/", &reg).unwrap();
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

    // s116：import/export-from 跨行续行——`from` 行尾不切语句（WPT referrer 族
    // 书写形：`import { x } from\n    "spec";`）。切分后提取与 transform 双双漏配
    // import 形 → 内联模块零测试注册（Timeout 假形态）。深度 0 下以完整标识符
    // `from` 收尾的合法语句仅 import/export-from 形。
    // https://tc39.es/ecma262/#sec-imports
    #[test]
    fn test_split_statements_multiline_import_from_continuation() {
        let src = "import { referrer as r } from\n    \"./a.js?name=x\";\nimport * as ns from\n  \"./b.js\";\nexport { r as rr } from\n  \"./a.js\";\nvar done = 1;";
        let stmts = split_statements(src);
        assert_eq!(stmts.len(), 4, "stmts={stmts:?}");
        assert!(stmts[0].starts_with("import { referrer as r } from"), "stmts={stmts:?}");
        assert!(stmts[0].contains("./a.js?name=x"), "stmts={stmts:?}");
        assert!(stmts[1].starts_with("import * as ns from"), "stmts={stmts:?}");
        assert!(stmts[2].starts_with("export { r as rr } from"), "stmts={stmts:?}");
        assert_eq!(stmts[3], "var done = 1", "stmts={stmts:?}");
        // 提取器同面：跨行形可提（此前漏提）。export-from 再导出同为静态依赖
        //（./a.js 双提去重 → 3 spec）。
        let specs = extract_static_module_import_specifiers(src);
        assert_eq!(specs.len(), 3, "specs={specs:?}");
        assert!(specs.contains(&"./a.js?name=x".to_string()), "specs={specs:?}");
        assert!(specs.contains(&"./b.js".to_string()), "specs={specs:?}");
        // `from` 前缀非标识符边界不误判（`platformfrom` 整词）。
        let glued = "var platformfrom = 1;\nvar x = 2;";
        assert_eq!(split_statements(glued).len(), 2, "glued={glued:?}");
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
        // s345：import 提升后依赖图求值先于入口体语句（spec
        // https://tc39.es/ecma262/#sec-moduleevaluation）——marker 快照读不到体中
        // `state.n=5`，判别读改为函数调用（活绑定语义：调用点在体中，读时取当前值）。
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/shared.js", "export const state={n:0};");
        sb.register_module(
            "https://a.test/chunk.js",
            concat!(
                "import{state}from\"./shared.js\";",
                "export const getMarker=function(){ return (state&&state.n===5)?'ok':'split'; };",
            ),
        );
        let r = sb
            .execute_module(
                concat!(
                    "import\"./shared.js\";",
                    "import{state}from\"./shared.js\";",
                    "state.n=5;",
                    "import*as c from\"./chunk.js\";",
                    "export const out=c.getMarker();",
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
        let compiled = compile_module_script(src, "zero://module", "zero://module", &ModuleRegistry::new()).unwrap();
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
    fn test_export_declaration_body_string_not_extracted_as_reexport() {
        // github react-core 真实形态（2026-10-09 pricing 活体）：`export const __webpack_modules__=
        // {…}` 巨型赋值的正文含错误消息字符串，串内 `from \`react-router/dom\`` 文本被
        // split_from_clause 当 from 子句 → 幻影依赖 → fetch 404 → 模块编译整体失败 →
        // 站点 chunk（cmi）三次重试后报 `Loading chunk cmi failed`。声明形态须整句跳过。
        let src = concat!(
            "export const __webpack_modules__={e(){aR(!1,'You provided the `flushSync` option ",
            "to a router update, but you are not using the `<RouterProvider>` from ",
            "`react-router/dom` so `ReactDOM.flushSync()` is unavailable.  Please update your ",
            "app to `import { RouterProvider } from \"react-router/dom\"` and ensure.');}};"
        );
        assert!(
            extract_static_module_import_specifiers(src).is_empty(),
            "红态：声明语句正文字符串里的 from 形态被当再导出切出幻影依赖"
        );
    }

    #[test]
    fn test_import_clause_shape_guard_rejects_non_binding_forms() {
        // import 侧同防线：IDENT 后跟 `=`/`(` 等非 from/`,` 形态不是 ImportClause
        let src = "import d={from\"phantom\"};import x(from'phantom2');";
        assert!(extract_static_module_import_specifiers(src).is_empty());
    }

    #[test]
    fn test_clause_shape_guard_keeps_real_import_forms() {
        // 形态守卫不得误伤真实 import 形态：默认/组合/命名空间/字符串/压缩形态
        let src = concat!(
            "import d from\"./d.js\";",
            "import x, {a as b} from './c.js';",
            "import*as ns from\"./ns.js\";",
            "import{p}from\"./p.js\";",
            "import\"./side.js\";"
        );
        assert_eq!(
            extract_static_module_import_specifiers(src),
            vec!["./d.js", "./c.js", "./ns.js", "./p.js", "./side.js"]
        );
    }

    #[test]
    fn test_reexport_shape_guard_keeps_real_reexport_forms() {
        let src = "export{a}from\"./x.js\";export*from\"./y.js\";export*as ns from\"./z.js\";export const k=1;";
        assert_eq!(
            extract_static_module_import_specifiers(src),
            vec!["./x.js", "./y.js", "./z.js"]
        );
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

    // ── R5182（site-search-availability，MDN 搜索弹层链上游根因）：模块恰好一次求值 ──
    // HTML module map：同一文档的模块记录只求值一次——fetch-a-single-module-script 命中
    // module map 直接复用（https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script）；
    // ECMA-262 module record evaluate 不得重入（https://tc39.es/ecma262/#sec-moduleevaluation）。
    // 活体最小复现（2026-10-19，本地 HTTP 页，域无关）：zw 单 module script 元素执行 2 次、
    // 多元素无共享（Chrome 均 1 次）；MDN 5 个 module 元素 → webpack 运行时多代执行 →
    // customElements.define "already used" ×14 + lit-html 命名空间导入 undefined → 搜索模态
    // 永不渲染。红态两支：(1) 入口自注册进 registry（collect_module_deps 语义）被 prelude
    // 急切执行 + 转换体再执行；(2) `var __moduleCache = {}` 每次 execute_module 重置，跨
    // 元素重跑整张依赖子图。复位语义（R5182-D1 返修）：缓存按文档隔离——renderer 路径经
    // `reset_context` 销毁重建全局对象；tab worker（`script_phase_begin`）与 webview 进程内
    // （`run_page_scripts_impl`）持持久 context，由宿主在每文档脚本阶段起点执行
    // [`MODULE_CACHE_RESET_SCRIPT`] 复位（三宿主一致：每文档恰好一次）。

    /// R5182 测试用持久上下文沙箱（renderer/tab worker 生产形态；part60 同款配置）。
    /// 双后端按本文件既定模式 cfg 分支（TV-1：quickjs 腿与 CI linux-x86_64-quickjs /
    /// windows-aarch64 走 `--no-default-features --features quickjs`，V8Sandbox 不存在）。
    #[cfg(any(feature = "v8", feature = "quickjs"))]
    fn r5182_sandbox() -> Box<dyn crate::Sandbox> {
        let config = crate::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        #[cfg(feature = "v8")]
        let mut sandbox: Box<dyn crate::Sandbox> = Box::new(crate::V8Sandbox::with_config(config).unwrap());
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox: Box<dyn crate::Sandbox> = Box::new(crate::QuickJSSandbox::with_config(config).unwrap());
        // 动态 import 取回面在纯沙箱无宿主：注册空 stub（本组用例只走静态 import 面）。
        sandbox.register_callback("__zw_compile_module", Box::new(|_args| String::new()));
        sandbox
    }

    #[test]
    fn test_r5182_entry_module_evaluates_once_per_element() {
        let mut sandbox = r5182_sandbox();
        let body = "globalThis.__r5182=(globalThis.__r5182||0)+1;";
        let mut registry = ModuleRegistry::new();
        // 入口自注册（collect_module_deps 把入口 URL 也放进 registry——t8j 语义）。
        registry.register("https://zero.test/entry.js", body);
        // 复刻宿主执行形（renderer/tab execute_module_in_sandbox）：prelude(入口跳过急切) + 转换体单次 execute。
        let prelude = build_module_runtime_prelude(&registry, Some("https://zero.test/entry.js")).unwrap();
        let transformed = compile_module_script(
            body,
            "https://zero.test/entry.js",
            "https://zero.test/entry.js",
            &registry,
        )
        .unwrap();
        sandbox.execute(&format!("{prelude}\n{transformed}")).unwrap();
        assert_eq!(
            sandbox.execute("String(globalThis.__r5182)").unwrap().value,
            "1",
            "入口模块每元素只求值一次（红态：prelude 急切执行入口 + 转换体再执行 = 2）"
        );
    }

    #[test]
    fn test_r5182_shared_dep_evaluates_once_across_elements() {
        let mut sandbox = r5182_sandbox();
        let dep_src = "globalThis.__r5182dep=(globalThis.__r5182dep||0)+1;";
        // 元素 1：入口 = 共享模块本体（MDN 形态：runtime.js 既是 script 元素又是依赖）。
        let mut reg1 = ModuleRegistry::new();
        reg1.register("https://zero.test/a.js", dep_src);
        let p1 = build_module_runtime_prelude(&reg1, Some("https://zero.test/a.js")).unwrap();
        let t1 = compile_module_script(dep_src, "https://zero.test/a.js", "https://zero.test/a.js", &reg1).unwrap();
        sandbox.execute(&format!("{p1}\n{t1}")).unwrap();
        // 元素 2：内联入口静态 import 共享模块（同文档第二个 module script 元素）。
        let mut reg2 = ModuleRegistry::new();
        reg2.register("https://zero.test/a.js", dep_src);
        let p2 = build_module_runtime_prelude(&reg2, Some("https://zero.test/e2.js")).unwrap();
        let t2 = compile_module_script(
            "import 'https://zero.test/a.js';",
            "https://zero.test/e2.js",
            "https://zero.test/e2.js",
            &reg2,
        )
        .unwrap();
        sandbox.execute(&format!("{p2}\n{t2}")).unwrap();
        assert_eq!(
            sandbox.execute("String(globalThis.__r5182dep)").unwrap().value,
            "1",
            "共享依赖跨 module script 元素只求值一次（红态：__moduleCache 每调用重置 + 内联重执行 = 4）"
        );
    }

    #[test]
    fn test_s94_inline_module_cache_key_separate_from_url() {
        // s94 附带修复（module 缓存键）：R5182 记忆化键与模块 URL 分离——同文档两枚
        // **内联** module script 旧以同一页面 URL 为键，第二枚被首枚的记忆化命中短路
        //（模块体零次求值 → currentScript-null "completed without reporting"，
        // probe-csn-d 双纯内联最小复现）。键分离后各自求值；同键（外链同 src 形态）
        // 仍恰好一次。
        // https://html.spec.whatwg.org/multipage/webappapis.html#module-map
        let mut sandbox = r5182_sandbox();
        let t1 = compile_module_script(
            "globalThis.__s94k1=(globalThis.__s94k1||0)+1;",
            "https://zero.test/page.html",
            "https://zero.test/page.html#!zw-inline-module-0",
            &ModuleRegistry::new(),
        )
        .unwrap();
        sandbox.execute(&t1).unwrap();
        let t2 = compile_module_script(
            "globalThis.__s94k2=(globalThis.__s94k2||0)+1;",
            "https://zero.test/page.html",
            "https://zero.test/page.html#!zw-inline-module-1",
            &ModuleRegistry::new(),
        )
        .unwrap();
        sandbox.execute(&t2).unwrap();
        assert_eq!(sandbox.execute("String(globalThis.__s94k1)").unwrap().value, "1");
        assert_eq!(
            sandbox.execute("String(globalThis.__s94k2)").unwrap().value,
            "1",
            "第二枚内联 module 须求值（红态：同键记忆化命中短路，零次求值）"
        );
        let ext = compile_module_script(
            "globalThis.__s94k3=(globalThis.__s94k3||0)+1;",
            "https://zero.test/a.js",
            "https://zero.test/a.js",
            &ModuleRegistry::new(),
        )
        .unwrap();
        sandbox.execute(&ext).unwrap();
        sandbox.execute(&ext).unwrap();
        assert_eq!(
            sandbox.execute("String(globalThis.__s94k3)").unwrap().value,
            "1",
            "同 cache_key（外链同 src 形态）恰好一次"
        );
    }

    #[test]
    fn test_r5182_named_import_shared_dep_via_cache_read() {
        // 边界变体（证明非仅副作用 import 形态有效）：具名导入走记忆化缓存读——依赖
        // 只在首个发射点求值一次，后续元素读同一导出实例。
        let mut sandbox = r5182_sandbox();
        let dep_src = "globalThis.__r5182n=(globalThis.__r5182n||0)+1;export const val = 41;";
        let mut reg1 = ModuleRegistry::new();
        reg1.register("https://zero.test/n.js", dep_src);
        let p1 = build_module_runtime_prelude(&reg1, Some("https://zero.test/ne1.js")).unwrap();
        let t1 = compile_module_script(
            "import { val } from 'https://zero.test/n.js';globalThis.__v1 = val;",
            "https://zero.test/ne1.js",
            "https://zero.test/ne1.js",
            &reg1,
        )
        .unwrap();
        sandbox.execute(&format!("{p1}\n{t1}")).unwrap();
        let mut reg2 = ModuleRegistry::new();
        reg2.register("https://zero.test/n.js", dep_src);
        let p2 = build_module_runtime_prelude(&reg2, Some("https://zero.test/ne2.js")).unwrap();
        let t2 = compile_module_script(
            "import { val } from 'https://zero.test/n.js';globalThis.__v2 = val;",
            "https://zero.test/ne2.js",
            "https://zero.test/ne2.js",
            &reg2,
        )
        .unwrap();
        sandbox.execute(&format!("{p2}\n{t2}")).unwrap();
        assert_eq!(
            sandbox.execute("String(globalThis.__r5182n)").unwrap().value,
            "1",
            "具名导入共享依赖只求值一次"
        );
        assert_eq!(
            sandbox
                .execute("String(globalThis.__v1 + globalThis.__v2)")
                .unwrap()
                .value,
            "82",
            "两个元素读到同一导出实例（val=41×2）"
        );
    }

    #[test]
    fn test_r5182_cache_resets_across_document_reset() {
        // 复位边界：跨文档导航 reset_context 销毁重建全局对象——缓存不得泄漏到新文档
        // （同 URL 再导航必须重新求值；HTML navigate "creates a new global object"）。
        let mut sandbox = r5182_sandbox();
        let body = "globalThis.__r5182r=(globalThis.__r5182r||0)+1;";
        let mut registry = ModuleRegistry::new();
        registry.register("https://zero.test/doc.js", body);
        let prelude = build_module_runtime_prelude(&registry, Some("https://zero.test/doc.js")).unwrap();
        let transformed =
            compile_module_script(body, "https://zero.test/doc.js", "https://zero.test/doc.js", &registry).unwrap();
        sandbox.execute(&format!("{prelude}\n{transformed}")).unwrap();
        assert_eq!(
            sandbox.execute("String(globalThis.__r5182r)").unwrap().value,
            "1",
            "文档 1：模块求值一次"
        );
        sandbox.reset_context();
        // TV-2：中间断言钉住「新全局里计数器未定义」——若复位失效（global 保留），此处读到
        // "1"（泄漏的记忆化实例命中并跳过模块体，下方断言也会假绿）。
        assert_eq!(
            sandbox.execute("String(globalThis.__r5182r)").unwrap().value,
            "undefined",
            "reset_context 后新全局不含上一文档的模块副作用（泄漏抑制重求值会被此断言抓住）"
        );
        sandbox.execute(&format!("{prelude}\n{transformed}")).unwrap();
        assert_eq!(
            sandbox.execute("String(globalThis.__r5182r)").unwrap().value,
            "1",
            "文档 2（reset_context 后新全局）：重新求值一次（缓存未跨文档泄漏）"
        );
    }

    #[test]
    fn test_r5182_host_reset_script_reenables_per_document() {
        // R5182-D1：tab worker（script_phase_begin）与 webview 进程内（run_page_scripts_impl）
        // 两条持久 context 路径的文档边界 = 宿主执行 MODULE_CACHE_RESET_SCRIPT。同一沙箱
        // （不 reset_context）模拟跨文档导航：复位前共享（第二文档零次求值＝D1 回归形态），
        // 复位后新文档重新求值一次。renderer 路径由 reset_context 覆盖（上一用例）。
        let mut sandbox = r5182_sandbox();
        let body = "globalThis.__r5182h=(globalThis.__r5182h||0)+1;";
        let mut registry = ModuleRegistry::new();
        registry.register("https://zero.test/page.js", body);
        let prelude = build_module_runtime_prelude(&registry, Some("https://zero.test/page.js")).unwrap();
        let transformed = compile_module_script(
            body,
            "https://zero.test/page.js",
            "https://zero.test/page.js",
            &registry,
        )
        .unwrap();
        let run_page = |sandbox: &mut Box<dyn crate::Sandbox>| {
            let prelude = prelude.clone();
            let transformed = transformed.clone();
            sandbox.execute(&format!("{prelude}\n{transformed}")).unwrap();
        };
        run_page(&mut sandbox);
        assert_eq!(
            sandbox.execute("String(globalThis.__r5182h)").unwrap().value,
            "1",
            "文档 1：模块求值一次"
        );
        // 文档 2 无复位（D1 回归形态对照）：持久 context 下记忆化命中 → 零次求值。
        run_page(&mut sandbox);
        assert_eq!(
            sandbox.execute("String(globalThis.__r5182h)").unwrap().value,
            "1",
            "无复位的持久 context：共享缓存命中，模块体不重执行（本用例钉住缓存持久性）"
        );
        // 文档 2 复位（宿主在脚本阶段起点执行 MODULE_CACHE_RESET_SCRIPT）：重新求值一次。
        sandbox.execute(crate::es_module::MODULE_CACHE_RESET_SCRIPT).unwrap();
        run_page(&mut sandbox);
        assert_eq!(
            sandbox.execute("String(globalThis.__r5182h)").unwrap().value,
            "2",
            "宿主执行复位脚本后：同 URL 模块在新文档重新求值（恰好一次，D1 修复语义）"
        );
    }

    // s255（instantiation-error-6/7 歧义面）：星号重导出歧义 = 链接期 SyntaxError。
    // ① 6 形：同名经同链两条星号路径可达；② 7 形：歧义子链经星号臂传播——兄弟星号臂
    // （7f）可消解不豁免（Node oracle：仍 SyntaxError）；③ 对照：单星号路径照常消解。
    // 红态：无三态判定时 ①② build 成功（歧义名漏进导出面）。
    #[test]
    fn test_star_export_ambiguity_s255() {
        let mut reg = ModuleRegistry::new();
        reg.register("./ierr-6a.js", "import { foo } from './ierr-6b.js';");
        reg.register(
            "./ierr-6b.js",
            "export * from './ierr-6c.js';\nexport * from './ierr-6d.js';",
        );
        reg.register("./ierr-6c.js", "export const foo = 'c';");
        reg.register("./ierr-6d.js", "export const foo = 'd';");
        let err = build_module_script(
            "import { foo } from './ierr-6b.js';",
            "https://zero.test/6a.js",
            &reg,
            false,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/6a.js".to_string()],
        )
        .expect_err("6 形双星号同名必须链接失败");
        assert!(
            matches!(&err, ScriptError::CompileError(m) if m.contains("conflicting star exports")),
            "歧义走 CompileError（宿主映射 SyntaxError 报告面）：{err:?}"
        );

        // 7 形（语料 instantiation-error-7a-7f）：7b 星号发现 7c，7c 星号发现 7d/7e
        // 歧义；7f 臂可消解——ResolveExport 星号臂子解析歧义即整体歧义，传播。
        // https://tc39.es/ecma262/#sec-resolveexport
        let mut reg = ModuleRegistry::new();
        reg.register("./ierr-7a.js", "import { foo } from './ierr-7b.js';");
        reg.register(
            "./ierr-7b.js",
            "export * from './ierr-7c.js';\nexport * from './ierr-7f.js';",
        );
        reg.register(
            "./ierr-7c.js",
            "export * from './ierr-7d.js';\nexport * from './ierr-7e.js';",
        );
        reg.register("./ierr-7d.js", "export let foo = 'd';");
        reg.register("./ierr-7e.js", "export let foo = 'e';");
        reg.register("./ierr-7f.js", "export let foo = 'f';");
        let err = build_module_script(
            "import { foo } from './ierr-7b.js';",
            "https://zero.test/7a.js",
            &reg,
            false,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/7a.js".to_string()],
        )
        .expect_err("7 形星号子链歧义必须传播为链接失败");
        assert!(
            matches!(&err, ScriptError::CompileError(m) if m.contains("conflicting star exports")),
            "传播歧义同走 CompileError：{err:?}"
        );

        // 对照（over-block 防回归）：单星号路径命中照常 build。
        let mut reg = ModuleRegistry::new();
        reg.register("./ok-a.js", "import { foo } from './ok-b.js';");
        reg.register("./ok-b.js", "export * from './ok-c.js';");
        reg.register("./ok-c.js", "export const foo = 'c';");
        build_module_script(
            "import { foo } from './ok-b.js';",
            "https://zero.test/ok-a.js",
            &reg,
            false,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/ok-a.js".to_string()],
        )
        .expect("单星号路径必须照常消解");
    }

    // s255 Fix B（star/re-export 内联记忆化钉）：入口 `import {x}` + `export * from`
    // 同一依赖——prelude 急切注册求值一次后，import 臂（R5182）与星号内联臂共享
    // __moduleCache 记忆化，依赖体恰好一次求值。红态：星号臂裸 IIFE → 第二次求值。
    #[test]
    fn test_star_inline_dep_evaluated_once_s255() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        let reported: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let reported_cb = reported.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                if let Some(v) = args.first() {
                    reported_cb.lock().unwrap().push(v.clone());
                }
                String::new()
            }),
        );
        let mut reg = ModuleRegistry::new();
        reg.register(
            "./s255-dep.js",
            "globalThis.__s255count = (globalThis.__s255count || 0) + 1;\nexport const foo = 'dep';",
        );
        let prelude = build_module_runtime_prelude(&reg, None).expect("prelude 构建不得失败");
        let entry = "import { foo } from './s255-dep.js';\nexport * from './s255-dep.js';";
        let module = build_module_script(
            entry,
            "https://zero.test/entry.js",
            &reg,
            false,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/entry.js".to_string()],
        )
        .expect("入口编译不得失败");
        sandbox
            .execute(&format!(
                "{prelude}\n{module}\n__zw_report('count=' + globalThis.__s255count);"
            ))
            .expect("探针执行不得同步 throw");
        let lines = reported.lock().unwrap();
        let joined = lines.join("|");
        assert_eq!(
            joined, "count=1",
            "依赖体恰好一次求值（急切注册 + import 臂 + 星号内联臂共享实例）：{joined}"
        );
    }

    // s255 Fix B 配套（注册序钉）：prelude 急切注册序 = 入口图 depth-first post-order
    // ——依赖先于导入方求值，同层按源内声明序。红态：registry 哈希序（HashMap 迭代）
    // 顺序不定——instantiation-error-7 实测 7d/7e 颠倒。
    #[test]
    fn test_prelude_registration_postorder_s255() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
        let reported: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let reported_cb = reported.clone();
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args| {
                if let Some(v) = args.first() {
                    reported_cb.lock().unwrap().push(v.clone());
                }
                String::new()
            }),
        );
        let marker = "globalThis.__s255ord.push(";
        let mut reg = ModuleRegistry::new();
        // a 导入 b、c（源内声明序）；入口导入 a——后序 b, c, a。
        reg.register(
            "./ord-a.js",
            &format!(
                "import {{ b }} from './ord-b.js';\nimport {{ c }} from './ord-c.js';\n{marker}'a');\nexport const a = 1;"
            ),
        );
        reg.register("./ord-b.js", &format!("{marker}'b');\nexport const b = 1;"));
        reg.register("./ord-c.js", &format!("{marker}'c');\nexport const c = 1;"));
        let prelude = build_module_runtime_prelude(&reg, Some("./ord-a.js")).expect("prelude 构建不得失败");
        let entry = "import { a } from './ord-a.js';";
        let module = build_module_script(
            entry,
            "https://zero.test/main.js",
            &reg,
            false,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/main.js".to_string()],
        )
        .expect("入口编译不得失败");
        sandbox
            .execute(&format!(
                "globalThis.__s255ord = [];\n{prelude}\n{module}\n__zw_report(globalThis.__s255ord.join(','));"
            ))
            .expect("探针执行不得同步 throw");
        let lines = reported.lock().unwrap();
        let joined = lines.join("|");
        assert_eq!(
            joined, "b,c,a",
            "急切注册序 = 入口图后序（依赖先于导入方，同层按源内声明序）：{joined}"
        );
    }

    // s336（动态 import 异步求值跳——HTML fetch a single module script 的异步性）：
    // import() 的模块体求值属后续 job，不占调用点同步块——helper 返回 Promise，取回
    // +求值经微任务调度。序判据（WPT microtasks/evaluation-order-3 全序列的 sandbox
    // 级钉）：入口体余下语句先完成 → 已排微任务先排空 → 子模块体及其微任务按 FIFO
    // 落位 → rejection 续体收尾。同步求值（旧态）把子模块体插进调用点同步块 → 序错。
    // https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script
    #[test]
    #[cfg(feature = "v8")]
    fn test_dynamic_import_evaluation_deferred_microtask_s336() {
        let fetched: std::sync::Arc<std::sync::Mutex<u32>> = std::sync::Arc::new(std::sync::Mutex::new(0));
        let fetched_cb = std::sync::Arc::clone(&fetched);
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        let contract = "https://zero.test/resources/dep.mjs\u{1f}(function() { \
            'use strict'; var _exports = {}; \
            globalThis.__log.push('step-dep-1'); \
            Promise.resolve().then(function () { globalThis.__log.push('microtask-dep'); }); \
            globalThis.__log.push('step-dep-2'); \
            throw new Error('error'); \
            return _exports; })()";
        let contract_for_cb = contract.to_string();
        sandbox.register_callback(
            "__zw_compile_module",
            Box::new(move |_args: &[String]| -> String {
                *fetched_cb.lock().unwrap() += 1;
                contract_for_cb.clone()
            }),
        );
        let lines: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let lines_cb = std::sync::Arc::clone(&lines);
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args: &[String]| -> String {
                lines_cb.lock().unwrap().push(args.first().cloned().unwrap_or_default());
                String::new()
            }),
        );
        let registry = ModuleRegistry::new();
        let prelude = build_module_runtime_prelude(&registry, None).unwrap_or_default();
        sandbox
            .execute(&format!(
                "{prelude}\nglobalThis.__log = [];\n\
                 __log.push('entry-1');\n\
                 Promise.resolve().then(function () {{ __log.push('microtask-entry'); }});\n\
                 __log.push('entry-2');\n\
                 __zw_dynamic_import.call({{ referrer: 'https://zero.test/resources/entry.mjs' }}, './dep.mjs').then(\n\
                     function () {{ __log.push('import-fulfilled'); }},\n\
                     function (e) {{\n\
                     __log.push('import-catch', e && e.message);\n\
                     __zw_report(__log.join(','));\n\
                 }});\n\
                 __log.push('entry-3');"
            ))
            .expect("执行不得整体失败");
        let joined = lines.lock().unwrap().join("|");
        let fetched_n = *fetched.lock().unwrap();
        assert_eq!(
            joined, "entry-1,entry-2,entry-3,microtask-entry,step-dep-1,step-dep-2,microtask-dep,import-catch,error",
            "动态 import 求值不占调用点同步块，序循 spec 异步 job 面：{joined}"
        );
        assert_eq!(fetched_n, 1, "宿主取回恰好一次");
    }

    // s337（TLA rejection 上报面——WPT microtasks/evaluation-order-4 的 sandbox 级钉）：
    // 入口体 `await import()` 拒绝（依赖体 throw）→ async IIFE promise 拒绝，此为模块
    // 求值失败的唯一错误通路（宿主包装 try/catch 接不到 promise 拒绝）。旧态无人接→
    // window error 不触达（runner 面 `window.onerror = testDone` 永不触发→Timeout）。
    // 修后 catch 桥 reportError 恰一次、携原值拒绝理由（ev.error 身份面）。全序列判据：
    // 入口同步块→入口微任务→依赖体及其微任务→rejection 续体（reportError）——
    // 依赖求值不占调用点同步块（s336 面），`step-4.1-3` 不执行（await 捷后余体）。
    // https://html.spec.whatwg.org/multipage/webappapis.html#run-a-module-script
    #[test]
    #[cfg(feature = "v8")]
    fn test_module_tla_rejection_reported_to_window_s337() {
        let fetched: std::sync::Arc<std::sync::Mutex<u32>> = std::sync::Arc::new(std::sync::Mutex::new(0));
        let fetched_cb = std::sync::Arc::clone(&fetched);
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        // evaluation-order-4.2.mjs：体 throw → import() promise 拒绝（语料原序）。
        let contract = concat!(
            "https://zero.test/resources/evaluation-order-4.2.mjs\u{1f}",
            "(function() { 'use strict'; var _exports = {}; ",
            "globalThis.__log.push('step-4.2-1'); ",
            "Promise.resolve().then(function () { globalThis.__log.push('microtask-4.2'); }); ",
            "globalThis.__log.push('step-4.2-2'); ",
            "throw new Error('error'); ",
            "return _exports; })()",
        );
        let contract_for_cb = contract.to_string();
        sandbox.register_callback(
            "__zw_compile_module",
            Box::new(move |_args: &[String]| -> String {
                *fetched_cb.lock().unwrap() += 1;
                contract_for_cb.clone()
            }),
        );
        let lines: std::sync::Arc<std::sync::Mutex<Vec<String>>> =
            std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let lines_cb = std::sync::Arc::clone(&lines);
        sandbox.register_callback(
            "__zw_report",
            Box::new(move |args: &[String]| -> String {
                lines_cb.lock().unwrap().push(args.first().cloned().unwrap_or_default());
                String::new()
            }),
        );
        let registry = ModuleRegistry::new();
        // evaluation-order-4.1.mjs（语料原序）：队列微任务后 await import。
        let source = concat!(
            "globalThis.__log.push('step-4.1-1');\n",
            "queueMicrotask(function () { globalThis.__log.push('microtask-4.1'); });\n",
            "globalThis.__log.push('step-4.1-2');\n",
            "await import('./evaluation-order-4.2.mjs');\n",
            "globalThis.__log.push('step-4.1-3');\n",
        );
        let transformed = compile_module_script(
            source,
            "https://zero.test/resources/evaluation-order-4.1.mjs",
            "https://zero.test/resources/evaluation-order-4.1.mjs",
            &registry,
        )
        .expect("入口编译不得失败");
        let prelude = build_module_runtime_prelude(&registry, None).unwrap_or_default();
        // 五跳梯：读态续体排在全部 rejection 链代际之后（链深 ≤ 4）。
        let ladder = concat!(
            "Promise.resolve().then(function(){}).then(function(){}).then(function(){})",
            ".then(function(){}).then(function(){ __zw_report('LOG:' + __log.join(',')); ",
            "__zw_report('ERR:' + __errs.join(',')); });",
        );
        let full_js = format!(
            "{prelude}\n\
globalThis.__log = [];\n\
globalThis.__errs = [];\n\
globalThis.queueMicrotask = globalThis.queueMicrotask || function (f) {{ Promise.resolve().then(f); }};\n\
globalThis.reportError = function (e) {{ globalThis.__errs.push(e && e.message); }};\n\
{transformed}\n\
{ladder}"
        );
        sandbox.execute(&full_js).expect("执行不得整体失败");
        let all = lines.lock().unwrap().join("|");
        let fetched_n = *fetched.lock().unwrap();
        assert_eq!(
            all, "LOG:step-4.1-1,step-4.1-2,microtask-4.1,step-4.2-1,step-4.2-2,microtask-4.2|ERR:error",
            "TLA rejection 经 catch 桥 reportError 恰一次且序循 spec（step-4.1-3 不执行）：{all}"
        );
        assert_eq!(fetched_n, 1, "宿主取回恰好一次");
    }

    // s338（dynamic-import credentials 面）发射契约钉：credentials 穿参 → IIFE 局部
    // `_importCred` 发射 + 重写发射 `credentials: _importCred` 引用（零引号零字面值，
    // t8j-r2 同规）；缺省路径（compile_module_script）发射 same-origin；prelude 双面
    // 穿参（__zw_load_module 第 4 参 credHint → __zw_compile_module 第 4 参，
    // __zw_dynamic_import 读 this.credentials）。
    #[test]
    fn test_module_credentials_emission_s338() {
        let registry = ModuleRegistry::new();
        let source = "const p = import('./dep.mjs');\n";
        let include = compile_module_script_with_credentials(
            source,
            "https://zero.test/resources/entry.mjs",
            "https://zero.test/resources/entry.mjs",
            &registry,
            "include",
        )
        .expect("入口编译不得失败");
        assert!(
            include.contains("var _importCred = \"include\";"),
            "include credentials 的 IIFE 局部发射缺席：{include}"
        );
        assert!(
            include.contains("credentials: _importCred"),
            "重写发射 credentials 槽缺席：{include}"
        );
        let default = compile_module_script(
            source,
            "https://zero.test/resources/entry.mjs",
            "https://zero.test/resources/entry.mjs",
            &registry,
        )
        .expect("入口编译不得失败");
        assert!(
            default.contains("var _importCred = \"same-origin\";"),
            "缺省 credentials 发射非 same-origin：{default}"
        );
        let prelude = build_module_runtime_prelude(&registry, None).expect("prelude 构建不得失败");
        assert!(
            prelude.contains("__zw_load_module = function(spec, parentHint, typeHint, credHint)"),
            "prelude __zw_load_module 第 4 参缺席"
        );
        assert!(
            prelude.contains(
                "__zw_compile_module(spec, parent, typeof typeHint === 'string' ? typeHint : '', credHint || '')"
            ),
            "prelude 宿主调用 credentials 第 4 参缺席"
        );
        assert!(
            prelude.contains("this.credentials"),
            "prelude 调用点 credentials 读缺席"
        );
    }

    // s338 行为钉：include credentials 入口的动态 import() → __zw_dynamic_import
    // `.call({referrer, credentials})` → prelude `_diCred` → __zw_load_module 第 4 参 →
    // 宿主 __zw_compile_module 第 4 参 == "include"（webview 侧据该值追加 `__zw_cred`
    // 取回标记——runner check-cookie 臂消费）。
    #[test]
    #[cfg(feature = "v8")]
    fn test_dynamic_import_credentials_passthrough_s338() {
        let captured: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>> = std::sync::Arc::default();
        let captured_cb = std::sync::Arc::clone(&captured);
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        sandbox.register_callback(
            "__zw_compile_module",
            Box::new(move |args: &[String]| -> String {
                captured_cb.lock().unwrap().push(args.to_vec());
                "https://zero.test/resources/dep.mjs\u{1f}(function() { return {}; })()".to_string()
            }),
        );
        let registry = ModuleRegistry::new();
        let transformed = compile_module_script_with_credentials(
            "const p = import('./dep.mjs');\n",
            "https://zero.test/resources/entry.mjs",
            "https://zero.test/resources/entry.mjs",
            &registry,
            "include",
        )
        .expect("入口编译不得失败");
        let prelude = build_module_runtime_prelude(&registry, None).unwrap_or_default();
        sandbox
            .execute(&format!("{prelude}\n{transformed}"))
            .expect("执行不得整体失败");
        let captured_args = captured.lock().unwrap();
        assert_eq!(captured_args.len(), 1, "宿主取回恰好一次");
        assert_eq!(
            captured_args[0].len(),
            4,
            "宿主调用带 credentials 第 4 参：{:?}",
            captured_args[0]
        );
        assert_eq!(captured_args[0][3], "include", "credentials mode 穿入宿主");
    }

    /// s339（iframe 域动态 import 模块体 window 身份桥——prelude eval 臂）：桥位
    /// 在场（**同步直调** `__zw_load_module` 形态；promise 双跳微任务面桥已清、
    /// 回落逐字求值——见 [`MODULE_EVAL_WINDOW_SCOPE_OPEN`] 文档）时模块体裸
    /// `window` 形参遮蔽到桥位对象（写落桥位、`globalThis` 仍主全局）；桥位缺席
    /// 按原样求值——裸 `window` 仍 ReferenceError（worker/裸 sandbox 零漂移语义
    /// 钉）。两 spec 分钉：模块 map 记忆化（s124）会让同 spec 二次 import 不重
    /// 求值，同体断言会被首 case 缓存掩盖。单 execute 全量（sandbox 缺省
    /// persistent_context=false——每次 execute 新 Context，跨 execute 全局不持久）。
    #[test]
    #[cfg(feature = "v8")]
    fn test_dynamic_import_window_scope_bridge_prelude_s339() {
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        sandbox.register_callback(
            "__zw_compile_module",
            Box::new(|args: &[String]| -> String {
                let (file, body) = match args.first().map(String::as_str) {
                    Some("./dep.mjs") => ("dep.mjs", "(function() { window.__zwProbeFromModule = 'hit'; globalThis.__zwProbeMain = 'main-hit'; return { ns: true }; })()"),
                    _ => ("dep2.mjs", "(function() { window.__zwProbe2FromModule = 'hit2'; return { ns2: true }; })()"),
                };
                format!("https://zero.test/resources/{file}\u{1f}{body}")
            }),
        );
        let registry = ModuleRegistry::new();
        let prelude = build_module_runtime_prelude(&registry, None).unwrap_or_default();
        // 桥位在场：同步直调（与 classic 面原生 settle 臂同窗形态）——模块体
        // `window` 写落桥位对象、`globalThis` 仍主全局、命名空间落缓存；随后桥位
        // 清空 + 第二 spec（躲开首 case 缓存）：逐字求值臂——裸 `window`
        // ReferenceError（无条件形参遮蔽缺席的语义钉），IIFE 首语句即抛、零写入。
        let probe = format!(
            "{prelude}\n\
             globalThis.__zwActiveIframeWin = {{ frame: true }};\n\
             globalThis.__zwActiveIframeDoc = {{ doc: true }};\n\
             var nsA = __zw_load_module('./dep.mjs', 'https://zero.test/resources/entry.html');\n\
             var rBridge = String(globalThis.__zwActiveIframeWin.__zwProbeFromModule);\n\
             var rMain = String(globalThis.__zwProbeMain);\n\
             var rNs = String(nsA && nsA.ns);\n\
             globalThis.__zwActiveIframeWin = undefined;\n\
             globalThis.__zwActiveIframeDoc = undefined;\n\
             var rB;\n\
             try {{ __zw_load_module('./dep2.mjs', 'https://zero.test/resources/entry.html'); rB = 'no-throw'; }}\n\
             catch (e) {{ rB = e && e.name; }}\n\
             [rBridge, rMain, rNs, rB, String(globalThis.__zwProbe2FromModule)].join('|');\n"
        );
        let result = sandbox.execute(&probe).expect("单回合探针不得整体失败");
        assert_eq!(
            result.value, "hit|main-hit|true|ReferenceError|undefined",
            "桥位在场 window 写落桥位/globalThis 落主全局/命名空间落缓存；桥位缺席裸 window ReferenceError 零半写"
        );
    }

    /// s344（module 记录求值恰一次 + 失败记忆）：同步入口体发射失败表先行复抛 +
    /// try/catch 记忆化形态（evaluation-error/compilation-error 跨元素同对象判据的
    /// 发射面）；TLA async 入口维持不记忆化（FIXME(R5182) 语义不变）。
    #[test]
    fn test_memoized_entry_failure_memory_s344() {
        let registry = ModuleRegistry::new();
        let compiled = compile_module_script(
            "globalThis.__side = 1;",
            "https://zero.test/e.js",
            "https://zero.test/e.js",
            &registry,
        )
        .unwrap();
        assert!(
            compiled.contains(r#"globalThis.__moduleFailures["https://zero.test/e.js"] !== undefined"#),
            "入口体失败表先行检查（s344 复抛形态）: {compiled}"
        );
        assert!(
            compiled.contains(r#"globalThis.__moduleFailures["https://zero.test/e.js"] = __zwModMemoErr"#),
            "入口体抛错记忆原值后重抛: {compiled}"
        );
        let tla = compile_module_script(
            "await 1;",
            "https://zero.test/tla.js",
            "https://zero.test/tla.js",
            &registry,
        )
        .unwrap();
        assert!(
            !tla.contains("__zwModMemoErr"),
            "TLA async 入口不记忆化（FIXME(R5182) 维持）: {tla}"
        );
    }

    /// s344/s345：入口图内模块的依赖求值由入口转换体**内联图**承担（体首提升 import
    /// 统一走记忆化表达式，失败表先行 + try/catch 记录；求值序 = 入口图源内声明序），
    /// prelude 不再急切发射入口可达模块——s345 循环依赖面：prelude 急切求值把入口源码
    /// 内联进循环依赖 IIFE（s344 入口入册后 registry 对 prelude 可见入口源）→ 入口体
    /// 先于依赖体完成 → TDZ 失明 + 体序倒置。prelude 只发射入口图不可达余量。
    #[test]
    fn test_prelude_dep_emission_memoized_and_source_order_s344() {
        let mut registry = ModuleRegistry::new();
        registry.register("https://zero.test/za.js", "globalThis.__za = 1;");
        registry.register("https://zero.test/ab.js", "globalThis.__ab = 1;");
        // 入口自身入册（s344 webview 同法）——import 声明序 ab 在前、za 在后。
        registry.register(
            "https://zero.test/entry.js",
            "import './ab.js';\nimport './za.js';\nglobalThis.__entry = 1;",
        );
        let prelude = build_module_runtime_prelude(&registry, Some("https://zero.test/entry.js")).unwrap();
        assert!(
            !prelude.contains(r#""https://zero.test/ab.js"] !== undefined"#)
                && !prelude.contains(r#""https://zero.test/za.js"] !== undefined"#)
                && !prelude.contains(r#""https://zero.test/entry.js"] !== undefined"#),
            "入口图内模块（依赖+入口）不入 prelude 急切求值（s345 内联图承担）: {prelude}"
        );
        // 入口转换体：依赖发射统一记忆化（裸守卫形态废止），源内声明序 ab 先于 za。
        let module = build_module_script(
            "import './ab.js';\nimport './za.js';\nglobalThis.__entry = 1;",
            "https://zero.test/entry.js",
            &registry,
            false,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/entry.js".to_string()],
        )
        .expect("入口编译不得失败");
        assert!(
            !module.contains("=== undefined) __moduleCache['"),
            "裸守卫形态废止（依赖发射统一记忆化）: {module}"
        );
        assert!(module.contains("__zwModMemoErr"), "依赖发射带失败记忆: {module}");
        let pos_ab = module
            .find(r#""https://zero.test/ab.js"] !== undefined"#)
            .expect("ab 依赖发射在场");
        let pos_za = module
            .find(r#""https://zero.test/za.js"] !== undefined"#)
            .expect("za 依赖发射在场");
        assert!(
            pos_ab < pos_za,
            "依赖求值序 = 入口源内声明序（ab 先于 za；旧字典序 leftovers 反转）"
        );
    }

    /// s344：记忆化表达式行为钉——体抛错 → 失败原值入表；同键复调复抛**同对象**
    /// 不重跑体（体副作用计数恒 1）；成功 → 缓存实例复用。
    #[test]
    fn test_memoized_expr_replay_same_object_s344() {
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
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
        registry.register(
            "https://zero.test/thr.js",
            "globalThis.__runs = (globalThis.__runs||0)+1; throw {bar: true};",
        );
        let iife = build_dep_iife(
            "https://zero.test/thr.js",
            &registry,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/thr.js".to_string()],
        )
        .unwrap();
        let expr = memoized_dep_expr("https://zero.test/thr.js", &iife, &[]);
        let probe = format!(
            "globalThis.__moduleCache = {{}};globalThis.__moduleFailures = {{}};\n\
             var e1, e2;\n\
             try {{ {} }} catch (e) {{ e1 = e; }}\n\
             try {{ {} }} catch (e) {{ e2 = e; }}\n\
             __zw_report([String(globalThis.__runs), String(e1 !== undefined && e1 === e2), String(e1 && e1.bar)].join('|'));\n",
            expr, expr
        );
        sandbox.execute(&probe).expect("探针求值成功");
        assert_eq!(
            reported.lock().unwrap().as_deref(),
            Some("1|true|true"),
            "体恰好跑一次（__runs=1）+ 复调复抛同对象（e1===e2）+ 原值保真（bar=true）"
        );
    }

    /// s344-r2（中间模块错误记账）：记忆化发射携带入口图 DFS 祖先键——体抛错时失败
    /// 原值**同时记入各祖先键**（spec 求值错误中间缓存：choice-of-error-3 首元素在
    /// 叶依赖 throw2?3d 抛错中止 prelude，中间模块 3b 自身发射未达，第二元素入口
    /// 3b 的短路面依赖 3b 键已记账——宿主 s344-r2 预检复抛同对象）。
    #[test]
    fn test_memoized_expr_records_ancestor_failures_s344r2() {
        let mut registry = ModuleRegistry::new();
        registry.register("https://zero.test/sb.js", "import './sthrow.js';\nglobalThis.__sb = 1;");
        registry.register("https://zero.test/sthrow.js", "throw {bar: true};");
        registry.register("https://zero.test/sa.js", "import './sb.js';\nglobalThis.__sa = 1;");
        // s345：祖先链由**入口体**内联发射携带（prelude 不再急切求值入口图）——叶依赖
        // sthrow 的祖先链 = [sa（入口）, sb]（发射内嵌于记忆化 catch 循环）；sb 的
        // 祖先链 = [sa]。
        let entry = build_module_script(
            "import './sb.js';\nglobalThis.__sa = 1;",
            "https://zero.test/sa.js",
            &registry,
            false,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/sa.js".to_string()],
        )
        .unwrap();
        assert!(
            entry.contains(r#"["https://zero.test/sa.js","https://zero.test/sb.js"]"#),
            "叶依赖发射携带全祖先链: {entry}"
        );
        assert!(
            entry.contains(r#"["https://zero.test/sa.js"]"#),
            "中间依赖发射携带入口祖先: {entry}"
        );
        #[cfg(feature = "v8")]
        let mut sandbox = crate::V8Sandbox::new().unwrap();
        #[cfg(all(feature = "quickjs", not(feature = "v8")))]
        let mut sandbox = crate::QuickJSSandbox::new().unwrap();
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
        let iife = build_dep_iife(
            "https://zero.test/sthrow.js",
            &registry,
            "same-origin",
            &mut HashSet::new(),
            &mut vec!["https://zero.test/sthrow.js".to_string()],
        )
        .unwrap();
        let expr = memoized_dep_expr(
            "https://zero.test/sthrow.js",
            &iife,
            &[
                "https://zero.test/sa.js".to_string(),
                "https://zero.test/sb.js".to_string(),
            ],
        );
        let probe = format!(
            "globalThis.__moduleCache = {{}};globalThis.__moduleFailures = {{}};\n\
             try {{ {} }} catch (_e) {{}}\n\
             __zw_report([\n\
               String(globalThis.__moduleFailures['https://zero.test/sthrow.js'] !== undefined),\n\
               String(globalThis.__moduleFailures['https://zero.test/sb.js'] !== undefined),\n\
               String(globalThis.__moduleFailures['https://zero.test/sa.js'] !== undefined),\n\
               String(globalThis.__moduleFailures['https://zero.test/sb.js'] === globalThis.__moduleFailures['https://zero.test/sthrow.js'])\n\
             ].join('|'));\n",
            expr
        );
        sandbox.execute(&probe).expect("探针求值成功");
        assert_eq!(
            reported.lock().unwrap().as_deref(),
            Some("true|true|true|true"),
            "失败原值同记叶键与全祖先键（同对象跨键共享）"
        );
    }

    /// s345：导出活绑定行为钉——导出侧 let 被本模块 setter 改写后，消费方经导入
    /// 绑定（globalThis getter）与命名空间读到的都是当前值（旧声明位快照接线
    /// 恒读实例化值 42——duplicated-imports 判据 `assert_equals(foo, 43)` 残红根因）。
    /// spec：导出绑定是活绑定 https://tc39.es/ecma262/#sec-exports。
    #[test]
    fn test_live_export_bindings_s345() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module(
            "https://a.test/shared.js",
            "export let n = 1;\nexport function setN(x) { n = x; }",
        );
        let r = sb
            .execute_module(
                concat!(
                    "import { n, setN } from './shared.js';",
                    "const r1 = n;",
                    "setN(7);",
                    "const r2 = n;",
                    "export const out1 = r1;",
                    "export const out2 = r2;",
                    "export const out3 = n;",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        assert_eq!(
            (
                r.namespace_json.contains("\"out1\":1"),
                r.namespace_json.contains("\"out2\":7"),
                r.namespace_json.contains("\"out3\":7"),
            ),
            (true, true, true),
            "活绑定：改写前读 1、改写后读 7（导入绑定与命名空间一致）：{}",
            r.namespace_json
        );
    }

    /// s345：import 提升行为钉——源内 import 在体尾而先导使用（late-namespace-request
    /// 判据形：`log.push(m.foo)` 在 import 语句之前）不再 ReferenceError。
    /// spec：import 声明静态提升 https://tc39.es/ecma262/#sec-moduledeclarationinstantiation。
    #[test]
    fn test_import_hoisting_s345() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module(
            "https://a.test/ns.js",
            "export let v = 5;\nexport function bump() { v = 9; }",
        );
        let r = sb
            .execute_module(
                concat!(
                    "const r1 = m.v;",
                    "m.bump();",
                    "const r2 = m.v;",
                    "import * as m from './ns.js';",
                    "export const out1 = r1;",
                    "export const out2 = r2;",
                    "export const out3 = m.v;",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        assert_eq!(
            (
                r.namespace_json.contains("\"out1\":5"),
                r.namespace_json.contains("\"out2\":9"),
                r.namespace_json.contains("\"out3\":9"),
            ),
            (true, true, true),
            "先导使用合法（提升）+ 命名空间活读：{}",
            r.namespace_json
        );
    }

    /// s345：循环依赖 TDZ 行为钉——ill-founded cycle（cycle-tdz-access 判据形）：
    /// 依赖体先于入口体求值（postorder），依赖读入口未初始化 `let` 导出经活对象
    /// getter 自然 TDZ 抛 ReferenceError（旧快照接线读 undefined 不抛）；入口体
    /// 不再执行（求值中止）。spec https://tc39.es/ecma262/#sec-moduleevaluation。
    /// 沙箱缺省非持久 context——错误与日志在单次探针执行内收集。
    #[test]
    fn test_tdz_cycle_reference_error_s345() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module(
            "https://a.test/a.js",
            "globalThis.__log = (globalThis.__log || []); globalThis.__log.push('a');\nimport { Y } from './entry.js';\nexport var X = Y;",
        );
        // 入口入册（依赖 a 的循环 import 解析基准；真实流程 collect_module_deps 同法）。
        sb.register_module(
            "https://a.test/entry.js",
            "import { X } from './a.js';\nglobalThis.__log.push('entry');\nexport let Y = X;",
        );
        let compiled = compile_module_script(
            "import { X } from './a.js';\nglobalThis.__log.push('entry');\nexport let Y = X;",
            "https://a.test/entry.js",
            "https://a.test/entry.js",
            &sb.registry,
        )
        .unwrap();
        let probe = format!(
            "var __zwErr = null;\ntry {{ {compiled} }} catch (e) {{ __zwErr = (e && e.constructor && e.constructor.name) || String(e); }}\n({{ err: __zwErr, log: (typeof globalThis.__log === 'undefined' ? null : globalThis.__log) }})"
        );
        let r = sb.sandbox.execute_json(&probe).unwrap();
        assert!(
            r.value.contains("\"err\":\"ReferenceError\"") && r.value.contains("\"log\":[\"a\"]"),
            "TDZ 抛 ReferenceError + 依赖体先执行、入口体中止：{}",
            r.value
        );
    }

    /// s345：星号再导出活委托行为钉——消费方经 `export * from` 委托 getter 读依赖
    /// 改写后的当前值（旧语句位值拷贝恒读委托时快照）；具名导出不被星号覆盖。
    #[test]
    fn test_star_export_live_delegation_s345() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module(
            "https://a.test/base.js",
            "export let flag = 'a';\nexport function setFlag(x) { flag = x; }",
        );
        sb.register_module("https://a.test/star.js", "export * from './base.js';");
        let r = sb
            .execute_module(
                concat!(
                    "import { flag, setFlag } from './star.js';",
                    "const s1 = flag;",
                    "setFlag('z');",
                    "const s2 = flag;",
                    "export const out1 = s1;",
                    "export const out2 = s2;",
                    "export const out3 = flag;",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        assert_eq!(
            (
                r.namespace_json.contains("\"out1\":\"a\""),
                r.namespace_json.contains("\"out2\":\"z\""),
                r.namespace_json.contains("\"out3\":\"z\""),
            ),
            (true, true, true),
            "星号委托活读：{}",
            r.namespace_json
        );
    }

    /// s345：发射形态钉——声明位快照接线废止、体首导出 getter 在场、导入绑定
    /// globalThis getter 在场（destructure_bindings/`var alias = _mod.src` 快照读
    /// 旧发射形废止）。
    #[test]
    fn test_binding_getter_shapes_s345() {
        let mut registry = ModuleRegistry::new();
        registry.register(
            "https://a.test/base.js",
            "export let flag='a';\nexport function setFlag(x){ flag=x; }",
        );
        let compiled = compile_module_script(
            "import { flag, setFlag } from './base.js';\nglobalThis.__r = flag;",
            "https://a.test/entry.js",
            "https://a.test/entry.js",
            &registry,
        )
        .unwrap();
        assert!(
            !compiled.contains("_exports.flag = flag;"),
            "声明位快照接线废止: {compiled}"
        );
        assert!(
            compiled.contains("Object.defineProperty(_exports, \"flag\", { get: function() { return flag; }"),
            "体首导出 getter: {compiled}"
        );
        assert!(
            compiled.contains("Object.defineProperty(globalThis, \"flag\", { get: function() { return _mod_"),
            "导入绑定 globalThis getter: {compiled}"
        );
    }

    /// s345：深层菱形重入共享实例钉——共享模块首份内联在兄弟 IIFE 作用域（入口不直接
    /// 依赖），后到分支的 Visited 臂 `typeof _mod_` 守卫不可见 → 回落 `__moduleCache`
    /// 命中同一实例（imports.html「Import the same module multiple times」判据形：
    /// INC_A === INC_AB_A 同一对象 + 值正确；旧空占位回落得 undefined.from TypeError，
    /// s345 全域 join 抓到的 P→F 回归，修复后翻绿）。spec：同 URL = 同一模块记录，
    /// https://html.spec.whatwg.org/multipage/webappapis.html#module-map。
    #[test]
    fn test_deep_diamond_visited_cache_fallback_s345() {
        let mut sb = EsModuleSandbox::new().unwrap();
        sb.register_module("https://a.test/base.js", "export const obj = { from: 'base.js' };");
        // 兄弟分支 1：base 首份内联落本 IIFE 局部 `_mod_base_`（入口不可见）。
        sb.register_module(
            "https://a.test/branch-a.js",
            "import { obj } from './base.js';\nexport { obj as OBJ_A };",
        );
        // 兄弟分支 2：base 重入（Visited）——typeof 守卫在自身/入口作用域均不可见，
        // 须经 `__moduleCache` 回落命中分支 1 的同一导出实例。
        sb.register_module(
            "https://a.test/branch-b.js",
            "import { obj } from './base.js';\nexport { obj as OBJ_B };",
        );
        let r = sb
            .execute_module(
                concat!(
                    "import { OBJ_A } from './branch-a.js';",
                    "import { OBJ_B } from './branch-b.js';",
                    "export const same = OBJ_A === OBJ_B;",
                    "export const from = OBJ_B && OBJ_B.from;",
                ),
                Some("https://a.test/entry.js"),
            )
            .unwrap();
        assert_eq!(
            (
                r.namespace_json.contains("\"same\":true"),
                r.namespace_json.contains("\"from\":\"base.js\""),
            ),
            (true, true),
            "深层菱形经缓存回落共享同一导出实例：{}",
            r.namespace_json
        );
    }
}
