# DC-2 · test262 import 形态严格化修复（2026-11-10）

接 [export default 簇](dc2-test262-export-default-fix-2026-11-10.md)后的 expressions
负例 76 案归簇——**67 案可修**（本仓重写层形态宽松），根因与修复如下；
9 案定性引擎/结构侧不回收。

## 归簇定性（expressions 负例 76 = 63 + 12 + 1）

**marker-Fail 判定原理**（全桶共用）：负例案 `negative: {phase: parse,
type: SyntaxError}`——引擎 parse 拒绝则负例满足 Pass；parse 放行则正文执行到
`$DONOTEVALUATE()` marker → Fail "Expected SyntaxError but got This statement
should not be evaluated"。

1. **63 案 dynamic-import/syntax/invalid import-defer 负例**：三形态（无参
   `import.defer()` / rest `import.defer(...a)` / new `new import.defer(...)`）
   × 21 嵌套上下文。s360 shim（ClassicDeferOnly 档）无条件重写 `import.defer(`
   前缀，非法形也被救活成合法调用 → parse 放行 → marker。bundled V8 原生对
   所有 defer 形 parse 报「Unexpected identifier 'defer'」（关旗）——无 shim
   时负例本应全过。
2. **12 案 assignmenttargettype**：4 importcall module 案（`import() = 1` /
   `(import()) = 1` / `import.defer() = 1` / `(import.defer()) = 1`）同根——
   Module 档重写把 `import(` 变 `__zw_dynamic_import()` 普通调用，丢失 V8
   原生 parse 检查（node 24 实证 `import() = 1` 报「import() requires a
   specifier」、defer 形报 Unexpected identifier）。8 strict callexpression
   案（`f() = 1` / `f()++` / `for (f() in …)` 等）**引擎侧不回收**：node 24
   （V8 13.6）+ Chrome 156（V8 15.6）+ ZW（V8 15.0.245.2）三引擎一致 parse
   容忍（Annex B web-compat），marker 先抛；for-in/of 形三引擎完全不抛。
   spec 要求 strict 下 parse 期 SyntaxError（#sec-static-semantics
   assignmenttargettype），V8 上游未按 parse 期实现——与 eval-code 簇同桶。
3. **1 案 escaped-await module**（expressions/class/class-name-ident-await-
   escaped-module）：`class await {}` StringValue 归一 "await"，module
   goal 下应 parse SyntaxError（node 实证抛）。ZW Fail 根因 = ESM 转换层 IIFE
   包装把 module 体降级为 function 上下文（await 非保留字）——结构性缺口，
   记入 module early-error 簇待归（该簇 ~132 案的定性输入）。

## 修复（s360-r2 形态严格化）

`es_module.rs` `rewrite_imports_impl` 四个重写落点（栈空×2 + TemplateExpr×2）
统一加两守卫，非法形保留原文交 V8 原生 parse 拒绝：

- `preceded_by_new_keyword`：`new` 关键字回扫（ImportCall 非
  MemberExpression，`new import(` 语法非法）；`renew import(` 前邻标识符续
  字符排除。
- `dynamic_import_args_well_formed`：参数列表预扫——非空、无顶层 `...`
  rest、无顶层 `,` 多参才重写；嵌套括号/字符串/模板/注释内字符不计顶层
  （`import((a, b))` / `import([1, 2])` / `import([...r])` 合法照重）；
  残缺未闭合保守不重写。

真实页面不受扰：合法单参形重写行为字节级不变；空参/rest/多参/new 形本为
语法非法，无页面依赖。ClassicDeferOnly 与 Module 两档同规。

## 验证

- 单测 3 新增（严格形态保留 / 预扫上下文精度 / new 边界回扫），script-sandbox
  lib **436 全绿**；clippy `-D warnings` 净；fmt 净（test-guard 包裹）。
- 真语料 test262 通道对拍（release runner，test-guard 包裹）：
  - `language/expressions/dynamic-import` 941 案：714 → **777 Pass（+63）**；
  - `assignmenttargettype`：12 Fail → **8**（4 importcall 转绿；8 strict
    callexpression 维持 Fail = 已定性引擎侧）；
  - `language/module-code` 596 案：318P/3T **不变**（0 回归）。
- html-semantics 受益面（s360 合法形 defer 页）：basic-deferred-evaluation /
  tla-deferred-evaluation / cyclic-tla-defer-eval 三页 **全 Pass**（cyclic
  优于 s360 记录的 bounded Fail——后续修复已覆盖该面）。
- SW 通道（service_worker.rs 共用重写层）：`testharness-service-workers-core
  FILTER=dynamic-import` 8 Pass / 0 Fail。

## 附带发现：s360 五件资源本地丢失

basic/tla 两页首跑报 "Failed to fetch module"——`dynamic-import/resources/`
五件（side-effect-module 等，s360 按 pin 315976933 手工补入、gitignore 不
入库）被本地语料刷新抹掉。按挂账 pin 重补（gh api contents），五件字节数
（58/118/97/121/109）与 update-wpt-data.sh 挂账逐件核对一致后页面转绿。
语料刷新后此五件需按挂账重补，已是第二次。

## 对目标净增的账

~75,059 + **67** ≈ **75,126 / 76,000**，余 ~874。

## 待办

- test262 剩余簇归簇：module early-error 132（含 escaped-await IIFE 降级
  输入）、SAB clone 106、ShadowRealm 57、递归深度 34
- encoding 残余可回收项（iframe 面 ~330 收益评估）
- DC-3 全量对账（同 revision 归一、验收线 1,393,955、邻域无回退）
