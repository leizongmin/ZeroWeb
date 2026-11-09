# DC-1 通道试跑证据——test262 runner 通道接入与抽样（2026-11-09）

代码落点 commit `79862fbb3`（feat(wpt-runner): test262 通道接入——上游 serve 包装链折叠单页执行）。

## 通道构成

- 语料 vendoring：`tests/wpt-runner/scripts/goals/48-test262-runtime.sh`——git partial clone
  + cone sparse-checkout，pin = `lib.sh` WPT_REV `315976933870b34d6ea30e3f6643403edae678ba`；
  vendored.toml 记 test262 上游 `tc39/test262 @ b66872a92487694396fb082343e08dd7cca5ddf4`。
  落盘 `tests/wpt-runner/wpt-data/third_party/test262/`（test/ 53,441 文件 225MB + harness/ 41 库）；
  `resources/test262/{test262-reporter,test262-provider}.js` 同 pin raw 拉取。幂等判据
  vendored.toml 在位，`FORCE=1` 重做。
- runner 通道：`zero-wpt-runner testharness-test262`（`testharness_test262.rs`）——上游
  serve.py 包装链折叠单页（`/tmp` 核对 serve.py @ pin 的 Test262Window*Handler 全文逐字对齐）：
  - 变体推导 `module > onlyStrict > default`，每用例恰一变体（语料无 META: variant，全库
    grep 零命中）；变体名与 wpt.fyi summary 路径逐字对齐（`.test262.html` /
    `.test262.strict.html` / `.test262-module.html`）。
  - skip 面 = 上游 manifest support 判据镜像：`*_FIXTURE.js`、无 frontmatter .js。
  - 折叠隔离：bootstrap `setup({allow_uncaught_exception: true})`——上游两页结构中 inner
    页异常不达 outer testharness；折叠页经此关闭 testharness 全局 error 毒化（testharness.js
    error_handler 的 `!tests.allow_uncaught_exception` 分支整段跳过），reporter 为唯一分类者。
  - module 拒绝臂：直接派发携带 error 对象的 ErrorEvent（上游 = `setTimeout(() => throw)`
    依赖浏览器把 timer 未捕获异常报成 window error——**ZW 引擎未实现该报告**，见挂账）。
- 单元测试 14 项全过（frontmatter 两形态、变体优先级、包装序、strict 前缀、module import、
  `</script` 转义、skip 规则等）。

## 抽样验证（DC-1 契约：同步/async/module/负例/realm 样本）

| 簇 | 用例 | 变体 | 结果 |
|---|---|---|---|
| 同步正例 | language/types/boolean/S8.3_A1_T1.js | default | Pass |
| strict 正例 | language/types/reference/8.7.2-3-a-2gs.js | onlyStrict | Pass |
| 负例 parse | language/types/boolean/S8.3_A2.1.js（`true=1` → SyntaxError） | default | Pass |
| 负例 runtime | 探针（inline script throw TypeError） | default | Pass |
| async | built-ins/Promise/exception-after-resolve-in-executor.js | default | Pass |
| module 正例 | language/eval-code/indirect/export.js | module | Pass |
| module 负例 parse | 探针（`export const x = ;`） | module | Pass |
| module 负例 runtime | 探针（module body throw TypeError） | module | Pass |
| module 负例（语料） | language/module-code/early-dup-export-dflt-id.js | module | **Fail**（见缺口 ②） |

## 全簇批跑与预算

`language/types/` 全簇：113/113 用例全执行（与目录 frontmatter 文件数逐位相等——skip/filter
记账零偏差），109 Pass / 4 Fail / 0 Timeout，墙钟 16.1s（含 53k 文件全量扫描）≈ **140ms/案**。
全量 53,441 案线性外推 ≈ 2.1h，可按目录分片入 CI（与 wasm-core 3600s envelope 同型，
`make testharness-test262 FILTER=<dir>` 透传）。探针目录验证后已删（gitignored 语料区，无残留）。

## 真实缺口（DC-2 归簇候选，通道如实记账不压制）

1. **ReferenceError 语义**：`8.7.2-1-s` / `8.7.2-3-a-1gs`（strict 删除属性应抛
   ReferenceError 未抛）——language/types/reference 2 案。
2. **`$262.createRealm` 面**：`get-value-prop-base-primitive-realm` /
   `put-value-prop-base-primitive-realm`——provider createRealm 走 iframe，runner 折叠页
   `Cannot read properties of undefined (reading 'prototype')`（census：createRealm 281 案、
   agent 112 案、gc 15 案——规模待全量基线确认）。
3. **module 早期错误检出**：`early-dup-export-dflt-id`（重复 `export default` 应 parse
   SyntaxError）——ZW 模块管道未检出早期错误，模块体求值触发 sta.js
   `$DONOTEVALUATE()` 陷阱抛串 → 负例不匹配。module 变体全量 832 案（上游 843）的
   早期错误子簇规模待全量基线。

## 引擎挂账（FIXME，跨通道基线重验后再修）

- **timer 未捕获异常无 window error 报告**：shim `part01.js` setTimeout/setInterval
  wrapper `catch (_e) {}` 静默吞异常（spec: clean up after running callback steps →
  report the exception）；async 探针（async flag + setTimeout throw）Timeout 实证。
  通道已绕开（module 拒绝臂直接派发 ErrorEvent）；共享路径修复会使既有通道
  「吞掉的 timer 异常」开始计 Fail（testharness 默认毒化），需全通道基线重验，不入本通道提交。
- **`unhandledrejection` 事件未实现**：reporter 兼听该事件；test262 async 用例以
  $DONE/print 收口为主，规模待 DC-2 全量基线确认后再评估。

## 与 wpt.fyi 记账对齐

- 上游变体规则（tools/manifest/sourcefile.py @ pin 逐字核对）：suffix = `.test262` +
  (`-module` | `.strict`) + `.html`，单变体。通道 case 名逐字对齐 → coverage exact match 可命中。
- 上游 vendored rev 漂移（本地 pin vs wpt.fyi universe 792c7512e3 runs）：onlyStrict 678
  逐位相等；module 832 vs 843、总用例 53,441 vs 53,794——约 350-600 文件上游漂移，
  按任务要求「另报上游漂移」，DC-3 对账时以同 revision 分母归一。
- wpt.fyi summary 实证：每 test262 路径恰 1 子测试（c=[1,1] 或 [0,1]）——通道每变体恰报
  1 子测试（async_test 完成态），分母 `max(浏览器 c[1], ZW total)` 逐文件对齐。
