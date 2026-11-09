# M2 切片 361：静态 `import defer *` 声明面——归一臂 + async 传染 + 契约 helper 前置

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/module/dynamic-import/` cyclic-tla-defer-eval.tentative.html（s360 挂账残 5F 之 1，tc39 issue #83 循环 TLA 死锁案）
**结果**: 单案 F→P——死锁三断言（B 体已评估 / A 体未评估 / promise 永不落定）全绿；域 join 零意外位移，语料口径 the-script-element **5F→4F、5T 不变**。

## 勘根定谳

1. **静态形式语义**：tc39 proposal-defer-import-eval——`import defer * as ns from "spec"` 仅 namespace 导入；带 TLA 的模块**不被 defer**（issue #83：cyclic TLA 死锁为预期行为，promise never settles）。
2. **真根因（三层同源）**：`strip_import_keyword` 剥出 clause `defer * as nsB from ...` → ① transform_import 按 default 臂发射（绑定名判 defer）；② `import_clause_shape_ok` 对 ident 后非 `from`/`,` 判 default 形态失败 → 两个图收集函数 continue 跳过 → registry 无 B → inline_dep_once Module not found → 编译失败；③ 无 async 传染 → sync IIFE 中 `await memoized` 语法错且死锁时序不成立。

## 施工

1. **es_module.rs `strip_defer_prefix`**（strip_import_keyword 后）：clause 剥 `defer` 前缀归一名空间臂；守卫 `defer` 后须 `*`（排除 `import defer from 'm'` default 绑定名恰为 defer、`import deferredX` 前缀重叠）。三接入点：transform_import 头部、extract_static_module_imports_with_attributes / extract_static_module_import_specifiers 形态判定前（发射面与图收集面共用归一，漏一即依赖漏收）。sync 目标静态 defer eager 内联违背标 FIXME（语料无钉面）。
2. **es_module.rs `source_has_deferred_import_decl`**（pub）：split_statements 逐句 strip_import_keyword + defer 前缀 + `*` 守卫谓词。async 传染三点：build_dep_iife（dep IIFE async 化，承接 defer 目标 TLA await 归一发射 + 死锁时序成立的前提）、compile_module_script_with_credentials async_body、webview.rs load 派发面 async_module_face。
3. **es_module.rs prelude helper 段抽离**：原 build_module_runtime_prelude 头部 137 行（三表 init + __zw_load_module + __zw_dynamic_import + __zwImportMapResolve）抽独立 `pub fn module_dynamic_import_helpers_prelude()`，prelude 头部改调（同拼面，module 页面包零漂移）。
4. **webview.rs 两契约 Ok 臂拼接**：compile_module_contract / compile_module_contract_deferred 产物 = `helpers 段 + return (iife)` 包裹成**单表达式** IIFE——契约 eval 上下文是 `(code)` 表达式求值（part01.js 12304 window-scope 包裹 `(OPEN+code+CLOSE)`），语句拼接 `(A);(B)` 非法（红态 3 实证 SyntaxError）；return 形态整体单函数调用表达式，求值返回 iife 结果（eager exports 对象 / deferred 三臂 record）不变。helper 段三表 init 均为 `globalThis.X = globalThis.X || {}` 幂等守卫，反复执行不重置缓存。

## 红态轨迹（solo 全程）

1. 施工前：`assert_true: module B should have started evaluating` got false——collect 图收集漏收 B（shape check 失败 continue）。
2. 发射/图收集归一后：断言 1/2 过、断言 3 `state="rejected with ReferenceError"`——probe19 定谳 `__zw_dynamic_import is not defined`（stack at __zw_load_module_deferred）：B 体内联在契约 eval 上下文，helper 只随页面 module script 包发射，classic 页（cyclic 入口页）prelude 从未执行。
3. helper 前置拼接后：`SyntaxError: Unexpected token ';'`（at eval, 12304:103）——`(helpers})();(iife)` 在 `(code)` 表达式外壳内是两语句非表达式。
4. return 包裹形态后：promise 20×100ms 内无 reject 无 fulfill（`info="no-reject"`）= 死锁 pending 达成 → corpus 案三断言全绿。

## Blast radius / 回归定谳

- **归一臂零位移**：strip_defer_prefix 对无 `defer` 前缀 clause 原样返回；`defer` 后非 `*`（default 绑定名 / deferredX）不归一。sandbox 425 单测含归一臂发射断言（`= await (globalThis.__moduleFailures` memoized 形态、`var nsB = _mod_`、`Object.defineProperty(globalThis, "defer"` 边界）与 async 传染三态断言全绿。
- **契约拼接面**：对无静态 defer 依赖的模块同样前置 helpers——重定义同源同值 + 幂等表 init，域 join 1129 键双面 A-only/B-only 双零、唯一状态翻转 = cyclic F→P、P→F=0、T 双向 ∅ 实证零意外位移。
- **B 面 4F/5T 与 s360 预期残集逐名同构**：F=143/052 长挂 2 + probe12/13 自建 2；T=5 同名单。
- 域 join / clippy / make test 见下。

## 验证

- **solo**：cyclic-tla-defer-eval.tentative.html Pass（`import.defer() deadlocks on cyclic TLA dependency: the promise never settles`）。
- **域 join**（A=s360-b vs B=s361-b，LC_ALL=C + (key,state) 对 sort -u）：键集合 1129 对 1129 双零；STATE-DIFF 恰 1 = cyclic F→P。**det F→P=1（目标案）/ P→F=0 / T 双向 ∅**；语料口径 the-script-element **5F→4F、5T 不变**。
- **clippy + fmt**：zero-script-sandbox + zero-webview --all-targets 净；cargo fmt 一处单测断言换行重排（零语义），复跑 sandbox 425 全绿。
- **make test**：9324P/7F，失败 7 名与 s356-s360 基线（并行流组合态既有集）逐名同构，本流不入侵零新增红。
- **probe 清理**：probe19-defer-cyclic.html 跑后已删（wpt-data 零残留）。

## 挂账 / 下一步

- mixed eager/defer 同 URL 记录共享语义；TLA 入口体延迟属性访问挂起语义；sync 目标静态 defer eager 内联 spec 违背（FIXME 在册）。
- 非 webview 宿主（renderer/tab/SW）import.defer 面。
- 存量在册：143/052 流式物化长挂；probe12/13 自建单列；T=5 长尾。

## 日志

- evidence/logs/s361-b-script-element.log（域跑 B 面，join 基线 s360-b-script-element.log）
- evidence/logs/s361-make-test.log（全量回归）
