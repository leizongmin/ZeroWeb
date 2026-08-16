# M4 — QuickJS execute 全局代码语义修复（R78）

**日期**: 2026-08-16
**Commit**: `d4eb0641`
**前置**: R76（quickjs WPT 矩阵基线 `68f09748`）+ R77（L2 第二刀负结果归档）
**证据**: [evidence/2026-08-16-r78-quickjs-global-eval-fix.json](../evidence/2026-08-16-r78-quickjs-global-eval-fix.json)

## 背景

R76 建立 quickjs WPT dom 矩阵双路径基线后，「quickjs 矩阵剩余缺口聚类」列为 R76 后下一步首选 ⑤。最大异常：**dom/traversal quickjs 7P/47F vs v8 953P**——两引擎同一用例集、同一 shim、差 130 倍。这不是 DOM 语义缺口（那类两引擎会共有），是 quickjs 特有根因。

## 根因定位（三轮 probe）

1. **失败聚类**：traversal 47F 几乎全部 `assert_node is not defined` / `testNodes is not defined`——都是内联 support .js（assert-node.js / common.js）里声明的全局符号。v8 同用例 Pass。
2. **最小复现**（script-sandbox 临时测试）：`sb.execute("function f(){}; var v=1;")` 后 `sb.execute("typeof f")` → `"undefined"`——**连裸声明都跨 execute 丢失**，与 try/finally 包装无关。
3. **C 层确认**（rquickjs-sys quickjs.c）：`js_parse_program` 的 `fd->is_global_var = (eval_type == JS_EVAL_TYPE_GLOBAL) || (MODULE) || !is_strict_mode`。`QuickJSSandbox::execute` 的 JS 包装 `String(eval(code))` 中 eval 是 JS 层**直接 eval**（JS_EVAL_TYPE_DIRECT），声明进 eval 临时词法环境。V8 的 `Script::compile+run` 是全局代码语义——两引擎分歧的源头。

**方案否决记录**：JS 间接 eval `(0,eval)(code)`（ECMAScript 全局环境语义）probe 通过了非严格体，但对 `'use strict'` 脚本体声明仍落临时环境（跨 eval `typeof s5` = undefined）——WPT 用例大量使用 strict（TreeWalker.html 自带 "use strict"），否决。唯一让 `is_global_var` 恒真的形态是 `JS_EVAL_TYPE_GLOBAL` 本身。

## 修复

`crates/script-sandbox/src/quickjs_runtime.rs` `eval_in_ctx`：

- 改用 C API `qjs::JS_Eval(ctx, code, len, "execute", JS_EVAL_TYPE_GLOBAL)`（经 `rquickjs::qjs` 重导出 + `Ctx::as_raw()`）——与 V8 `Script::run` 等价的全局代码执行。
- 不加 `JS_EVAL_FLAG_STRICT`：页面 classic 脚本默认非严格；脚本体自带 `'use strict'` 指令时 parser 自行进入严格模式且声明仍全局（rquickjs `EvalOptions::default()` 的 `strict:true` 会强制 strict 且其结构体 `#[non_exhaustive]` 无法外部构造——本就不该用）。
- 结果经 `JS_ToString` + `JS_ToCStringLen2` 字符串化（`String(result)` 契约不变）；异常路径保持 `ctx.catch()` 提取 message 的既有语义。
- Worker 路径（quickjs_worker.rs）本就用裸 `ctx.eval`（rquickjs GLOBAL）——主 execute 路径现与其一致。

## 效果（quickjs WPT dom 矩阵）

| 子目录 | R76 | R78 | v8 基线 |
|--------|-----|-----|---------|
| traversal | 7P/47F (12.96%) | **953P** | 953P（**对齐**） |
| collections | 38P/10F | **48P/0F** | 48P（对齐） |
| nodes | 2201P/1113F | **3113P** | ~3096P+ |
| events | 156P/99F | **185P** | ~189P |
| ranges（抽样） | 40P/72F | 15P/11F（Range-attribute-nodes）| 逐字节一致 |

- native 路径（ZW_NATIVE_DOM=1）traversal 953P/627F 与 polyfill 完全一致——R76 双路径对等性保持。
- ranges 全量被 Range-mutations-insertBefore 慢用例吃满 test-guard 2400s（M1 L2 已知遗留，v8 在 R76 同样吃满 1800s——非本切片引入）。
- nodes/events 剩余 fail 聚类核实与 v8 共有（compareDocumentPosition 语义缺口两引擎同败）——无新增引擎特有分歧。

## 验证

- 新回归测试 `tests/quickjs_scope_repro_r78.rs`：裸声明跨 execute 存活 + classic try/finally 包装（webview 生产形态同构）+ strict 脚本体声明全局可见 + strict undeclared 赋值抛错 + 返回值契约。
- script-sandbox quickjs 76+1 全绿；engine quickjs 1424；wpt-runner quickjs 106；script-sandbox v8 155；integration quickjs 747P/2F（html_compat default_actions 两失败 clean HEAD 同败——并行流既存，stash 验证）。
- clippy（quickjs 矩阵）零警告；cargo fmt 无 diff；pre-commit-guard PASS。

## 过程教训

1. **双引擎架构下「同一 shim」不等于「同一行为」**——shim 之下的执行原语（eval 形态）是行为面的一部分。R76 的 0.02pp 对等结论只在「用例侧 document 走 shim」前提下成立，脚本装载层此前从未被 WPT 级对拍。
2. **rquickjs 高层 API 的默认值有坑**：`EvalOptions::default()` 强制 strict 且 `#[non_exhaustive]`；`ctx.eval` 的文档说 "Evaluate a script in global context" 但 JS 包装字符串里再嵌 eval 就完全不是那回事。
3. **probe 否决要留档**：间接 eval 方案在非严格用例全对，只有 strict 脚本暴露差异——测试矩阵必须含 strict 用例才能否决一个 eval 方案。
