# DC-1 全量基线——test262 通道终账与 CI 接入（2026-11-10）

wrapper t-collision 修复（commit `2214db000`：用例源顶层 `var/let t` 与折叠页 outer
句柄碰撞曾整页 SyntaxError）后全量重跑。原始 JSON 在 `/tmp/test262-baseline/`
（分片脚本 `run-shards.sh` + 聚合 `aggregate.py`，test-guard 包裹
`--per-proc-mem 4 --total-mem 8 --time-limit 7200`，顺序总墙钟 ~3.7h）。

## 全量终账（vendored pin：test262 @ b66872a9，WPT_REV 315976933）

| 分片 | 执行数 | Pass | Fail | Timeout | 墙钟 | 通过率 |
|---|---:|---:|---:|---:|---|---:|
| staging | 1,484 | 1,362 | 117 | 5 | 6.5min | 91.8% |
| annexB | 1,086 | 1,032 | 54 | 0 | 3.1min | 95.0% |
| intl402 | 3,323 | 3,273 | 49 | 1 | 11.0min | 98.5% |
| language | 23,645 | 22,648 | 992 | 5 | 65.0min | 95.8% |
| built-ins | 23,512 | 22,809 | 607 | 96 | 107.0min | 97.0% |
| test/harness | 116 | 111 | 3 | 2 | 12.3min | 95.7% |
| **合计** | **53,166** | **51,235** | **1,822** | **109** | ~3.7h | **96.36%** |

- 执行数 = 53,441 源文件 − 275 skip 面（fixture / 无 frontmatter support）。
- **探针污染修复校正后净账：51,250 Pass / 1,807 Fail / 109 Timeout = 96.39%**
  （见下「runner 探针污染缺陷」——15 案 Fail 系 runner 侧解析缺陷，修复后重跑
  15/15 转 Pass；shift/pop set-length 4 案 Timeout 修复前后同状态，非回归）。
- 分母口径：DC-3 对账以同 revision 归一（上游漂移见末节）。

## 变体拆分（全量）

- default：51,656 案 97.3%（50,252P / 1,298F / 106T）
- strict：678 案 90.0%（610P / 68F）
- module：832 案 44.8%（373P / 456F / 3T）——module 面失败大头是 V8 语法/语义
  缺口（syntax-parse diffs ~195、dynamic-import/default-export 语义 ~192），
  通道侧可归簇的 module 内 early-error tripwire ~61（全变体口径 ~133，
  `$DONOTEVALUATE` 捕获——引擎未检模块早期错误）。

## 失败归簇（DC-2 候选定量，修复校正后 1,807F + 109T）

1. **other ~1,446**——提案语法/API 缺口为主，可细分：
   - ~91 Expected SyntaxError 未抛（early-error 负例检出缺口）
   - ~91 `import source` phase 提案（`Unexpected identifier 'source'`）
   - ~70 import defer 提案（`SameValue(1, 0)`）
   - ~57 ShadowRealm is not defined（提案）
   - ~50 Expected ReferenceError 未抛（early-error 负例）
   - ~34 Maximum call stack size exceeded（递归深度）
   - ~31 top-level await 负例、~26 Proxy -realm 案（createRealm 同根）
   - ~54 Iterator.zipKeyed/.zip（Iterator helpers 提案补齐）
   - ~25 module fetch 语义（Failed to fetch module）
   - ~76 语法解析缺口杂项（Unexpected token/number/string、missing `)`）
   - ~14 transferToImmutable 提案、~16 module Compile error 杂项
2. module early-error tripwire ~133（`$DONOTEVALUATE` 捕获，引擎未检模块早期错误）
3. agent/SAB 108（SharedArrayBuffer cross-compartment / structured clone）
4. timeout 109（30s 单案预算吃满；含 shift/pop set-length 4 案——
   Array.prototype 索引 accessor 污染 testharness 完成判定，见下）
5. createRealm/installAPI 78（iframe realm 语义）
6. evalScript/otherGlobal.eval 31（`g.eval is not a function`——$262 eval 面）
7. gc/TestUtils 11

## runner 探针污染缺陷（本轮修复，两形态）

test262 用例可合法污染全局原型，runner 的结果记录/探针序列化被劫持：

1. **结果数组空洞**（15 案 Fail，修复后 15/15 转 Pass）：`15.4.4.x-7-c-i` 族、
   `set-length-{frozen,non-writable}`（push/unshift）案在 `Array.prototype` 上
   定义索引 accessor（仅 setter）。result 回调在用例体执行**后**触发，shim
   `__zw_harness_results.push(obj)` 走 `[[Set]]` 命中继承 setter 丢值 → 数组空洞
   → 探针 `JSON.stringify` 出 `[null]` → serde 反序列化 Fail。修复：改用
   `Object.defineProperty`（`[[DefineOwnProperty]]` 写 own 索引）。
2. **探针 stringify 被劫持**（1 案）：`no-boolean-toJSON` 把
   `Object.prototype.toJSON` 赋为返回标量的函数 → 探针整体 `JSON.stringify`
   被劫持返回 `2`。修复：probe 包 IIFE，stringify 前暂删
   `Object.prototype.toJSON`、`finally` 恢复；delete 撞 non-configurable 时
   静默回到原行为。
3. 未修：shift/pop set-length 4 案 Timeout——testharness.js（pinned 语料，
   不改）内部数组 push 同样走 `[[Set]]`，完成判定被污染挂死。上游浏览器从
   DOM 读结果不受影响；归 timeout 簇，DC-2 再评估。

验证：修复后 15/15 目标案 Pass（各含 3 连跑），`cargo test -p zero-wpt-runner`
318 passed / 0 failed，encoding `api-basics`（6 子测试）与 dom
`attributes-are-nodes`（4 子测试）代表案全 Pass——probe 面为全通道共享，无回归。
clippy `-D warnings` 干净。

## CI 接入

`ci-shards.yml` 手调三片（单分片 CI 超 360min job 硬顶，按一级目录拆）：

- `test262-language` / `test262-builtins`：filter 各一级目录，time_limit 9000，
  job_timeout 240。
- `test262-misc`：filter `|` 多段合并 staging/annexB/intl402/test-harness 四目录，
  time_limit 5400，job_timeout 150。
- runner filter 多段语义（`split('|').any(contains)`）落在
  `testharness_test262.rs` filter 判据一行，其他套件 filter 不含 `|` 行为不变；
  多段并集已实证（单段 1 案 → 双段并集 5 案）。
- fetch 复用 `fetch-wpt-test262`（48 号脚本幂等，CI fresh checkout 每周
  partial clone ~225MB）。
- discover 验证：37 分片，test262 三片入 matrix，catch-all 不再收它（此前
  未手调时会被 catch-all 打包成单分片全量跑，CI 必超时记 0）。
- 本地实测墙钟 ×2 + 余量定 time_limit：language/built-ins 65/107min ×2 ≈ 9000s；
  misc 合片 20.6min ×2 ≈ 5400s（CI ≈ 本地 3–4×，余量见 CI 分片经验注释）。

## 上游漂移（DC-3 对账口径）

本地 pin vs wpt.fyi universe 792c7512e3：module 832 vs 843、总用例 53,441 vs
53,794（~350-600 文件漂移）。DC-3 全量对账以同 revision 分母归一，漂移另报。

## 对目标净增的账（DC-2 输入）

test262 通道接入后全量分子净增 ≈ **51,250**（当前执行且通过）。目标净增 76,000，
余 ~24,750 从 encoding（缺口 23,847）+ wasm（缺口 2,803）补——需回收两域缺口
~93%，DC-2 主战场在 encoding；test262 module 面（44.8%）与探针同类的通道侧
修正仍可零星增量。

## 待办

- encoding / wasm 两域 DC-2 通道现状与缺口定量（下一步）
