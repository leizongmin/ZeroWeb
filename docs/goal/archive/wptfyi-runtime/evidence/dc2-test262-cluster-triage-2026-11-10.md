# DC-2 · test262 失败簇定性（2026-11-10）

接 [DC-1 全量基线](dc1-full-baseline-2026-11-10.md)的归簇收敛：对三个失败大簇用
**真实语料文件 + 三个独立引擎**（ZW 内嵌 V8 15.0.245.2 / node 24.21.0·V8
13.6.233.17 / Chrome 156.0.8078.79 headless）交叉定性。结论：三簇均为 **V8
引擎侧差异**（上游未实现或未发布），ZW 行为与最新 Chrome 逐字一致，非 runner
缺陷、非 ZW 引擎引入，本 goal 不回收。

## 簇 1 · sec-evaldeclarationinstantiation 早期错误（~88 sync 变体）

- 形态：`function f(p = eval("var arguments")) {}` +
  `assert.throws(SyntaxError, f)`——sloppy 函数 param-init direct eval 声明
  `arguments`/`eval` 应抛 SyntaxError（案 esid 即 sec-evaldeclarationinstantiation）。
- 实证链（`test/language/eval-code/direct/func-decl-*-declare-arguments.js` 家族）：
  - ZW runner func-decl/gen/async-gen 变体 Fail（DC-1 基线 + 真语料复跑一致）；
  - node 24（V8 13.6，`vm.runInThisContext` classic 语义 + assert 桩）同案 Fail；
  - **Chrome 156 headless 同案 Fail**（file:// 加载真实语料，`assert.throws`
    报 "Expected a SyntaxError to be thrown but no exception was thrown at all"）；
  - V8 主线 `test262.status` 仅排除 non-definable-function 4 案，该家族不在表；
  - Firefox `jstests.list` 不排除该家族（SpiderMonkey 实现该检查）——spec 要求
    成立，V8 上游长期不实现（引擎 conformance 缺口，非语料错误）。
- async 变体通过属空转：案形 `f().then($DONE, error => { assert(error
  instanceof SyntaxError); }).then($DONE, $DONE)`——不抛的引擎 f() 正常
  resolve → `$DONE`（所有标准 harness 同行为）；实现检查的引擎走 reject 路径
  过。两引擎族皆报 Pass，runner 无 bug。

## 簇 2 · source phase imports 未发布（expressions/dynamic-import 90 + module-code/source-phase-import 子集）

- 形态：`import.source(obj)` 动态调用（features `[source-phase-imports]`，
  上游 .case 无 `module` flag，classic 变体展开合规）。
- 实证：Chrome 156 在 **classic 与 module 上下文均** `SyntaxError: Unexpected
  identifier 'source'`（headless 实测）——V8 未发布 source phase imports；
  V8 主线 `test262.status` 对 `language/module-code/source-phase-import/*`
  与 `staging/source-phase-imports/*` 记 `[SKIP]`（926-927、1088 行）。
- 嵌入引擎 V8 15.0.245.2（v8 crate 150.2.0）与 Chrome 156（V8 15.6）行为一致。

## 簇 3 · import defer 未发布（~70）

- 形态：`import.defer(...)` 动态调用（deferred module evaluation）。
  Chrome 156 实测 module 上下文 `SyntaxError: Unexpected identifier 'defer'`
  ——动态 defer 形态未发布（至少在本仓 pin 语料所用形态上）。与簇 2 同桶。

## 处置与账

- 三簇 ~250 失败案定性引擎侧：修复 = V8 特性实现或实验 flag 决策（flag 存在
  与否未验证），超出本 goal（通过率推进）授权——记引擎挂账，不进 DC-3 回收面。
- 净增账不变（~75,032 / 76,000）；本批案在 DC-1 基线 96.39% 内、不在已计净增中。
- test262 剩余待归簇：module early-error 132、SAB clone 106、ShadowRealm 57、
  expressions 负例 76（"Expected SyntaxError but got This statement should
  not be evaluated"）、"Assignment to constant variable" AsyncTestFailure 18
  （疑引擎行为异常，待查）、递归深度 34 等。
- 临时探针已清理（eval-code 目录 zw-probe-*.js）；调查中间产物在 /tmp
  （V8/Firefox 状态文件、Chrome 测试页），关键输出已内联本文。
