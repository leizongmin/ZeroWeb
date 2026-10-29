# M2 s149 · runner 终态判定收口——harness 页级超时早收形（文件级 Timeout 族 91/96 单根）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s148（input-type-change validity
type 读快照盲区收口）

## 入场定刀位

s148 下一步候选①：文件级 Timeout 族单根翻拣（link-multiple-load-events /
csp-blocked；刀 B 拆账余面 + s148 观察①同构——「per-test timeout 缺失面勘根」）。

承前注记：上一 session 在 s148 提交后、429 限流中断前（16:31–16:40）已在
form-submission-0/ 落 4 个勘根探针（zz-s149-probe-{timer,ladder,trace,ht}.html，
gitignored 临时语料），探向与本轮同一假设（timer 通道在 harness 页是否真实
fire / harness 级 10s 文件超时是否触发）；timer/ladder 探针 Pass、trace/ht 探针
文件级 Timeout。本轮从该现场接续，干净全量前已按惯例删除。

## 勘根（上游 harness 源码 + 定向复现定谳）

1. **定向复现**（link-multiple-load-events.html）：4 案 2s step_timeout 兜底
   正常翻 Fail（timer 通道无恙）+ 文件级 Timeout，completion state
   `{"loaded":true,"pending":2,"phase":4,"tests":6}`——**phase 已 COMPLETE、
   completion 回调已发，runner 却报「completion callback was not called」**。
2. **上游 harness 结构定谳**（pinned `wpt-data/resources/testharness.js`）：
   - **无默认 per-test 超时**——`settings.test_timeout=null`，`Test` 级
     `set_timeout()` 仅在显式 `setup({timeout})` 时才挂定时器；控制面原文
     「上游 8s 默认 per-test 超时」假设不成立。
   - 真实兜底是**页级 harness 超时**：`Tests` 构造即 arm
     `setTimeout(Tests.timeout, 10s)`（normal；`<meta name=timeout
     content=long>` 时 60s）。`Tests.prototype.timeout` 把**全部测试强制标
     COMPLETE** 后 `complete()` → `notify_complete()` 同步发 completion 回调。
   - 早收后果：未报测试的 `done()` 在强制 COMPLETE 后 no-op——**结果行不再
     变化**（partial rows 即终量），`num_pending` 恒 >0。
3. **runner 侧单点归因**：终态判定
   `terminal_harness_state` 要求 `phase==4 && pending==0 && tests==results`
   配平；早收形 pending 恒 >0、results 恒 <tests → 永不满足 → 探针空转至
   30s 案超时 → 文件级 Timeout 行。s148 全量 96 个文件级 Timeout 键中
   **91 例 state.phase==4**（harness 已自收）、5 例 phase==2 独立子面（见挂账）。
4. **次因（同修）**：probe JS 的 complete 计算把 1s timer grace 检查放在
   `__zw_harness_complete` 之前——页面存在持续定时器时 complete 恒 false，
   completion 已发也到不了终态。completion 后 `done()`/`step()` 全 no-op，
   不可能再有结果行，flag 检查前移语义安全。

## 施工（tests/wpt-runner/src/testharness.rs 3 处，净 +26/−3）

1. probe JS：`if (globalThis.__zw_harness_complete) return true;` 前移到
   timer grace 循环之前（completion = 文件终态）。
2. `harness_probe_is_terminal` 新增臂：`complete && state.phase == 4`
   （completion 已发 + harness phase COMPLETE = 终态，不再要求
   pending/results 配平）。
3. 终态收集块：早收形保留**文件级 Timeout 键**——upstream wptrunner 对
   harness status=TIMEOUT 文件记 TIMEOUT + partial rows；键/态与案超时旧
   路径同形（join 零位移），消息准确化
   （`page timeout completed the file with N test(s) unreported`）。

## 验证

- 定向 link-multiple-load-events：行集逐字节同形（4 Fail + 1 文件级
  Timeout），消息准确化；**墙钟 30s（案超时）→ 10.1s**（`ZW_TESTHARNESS_
  TIME_LOG=1`：`run=9995ms` = harness 页级 10s 超时即收）。
- 定向 input-type-change-submit：**20/20 全 Pass**（s148 主刀文件零回归）。
- **全量 join**（对 s148 /tmp/zw-s148-full.log，(key,state) 对 sort -u）：
  s149 **7087P/810F/96T = 7993 对**（s148 7085P/812F/96T = 7993 对）——
  **键对数与 96T 键集逐字节守恒**；唯一位移 = modulepreload-referrerpolicy
  2F→P（s148 在册全量负载 flake 本轮自然复绿，s124 族观察先例），**零真
  P→F、零键增减**。早收形 91 例 = 26 例（partial rows + 文件级 Timeout
  标记）+ 65 例（零行，既有「completed without reporting」路径）；5 例
  phase==2 走原 deadline 路径不变。
- 门禁：clippy workspace --all-targets -D warnings 零告警；runner bin
  **264/0F**（s148 基线 263/1F——trend_csv_check 本轮已绿，并行流归口）；
  fmt 过。零 .js/.shim 改动（纯 runner 终态面，产品路径零触碰）。

## 挂账（本切片更新）

- **候选① 文件级 Timeout 族「per-test timeout 缺失面」定谳收口**：缺失的
  不是 harness 超时（页级 10s 兜底在位且真实 fire），是 runner 终态判定
  对早收形的识别。键集报法漂移的结构根（挂起吞没键 / 30s 窗口随机性）
  随完成点确定性（10s 即收）收窄。
- **新记挂账：meta timeout=long × corpus_case_timeout 通道**——5 例
  phase==2（object-events / beforeinput.tentative / rel-attribute-clearing /
  target_blank_implicit_noopener ×2）全带 `<meta name=timeout content=long>`：
  `run_testharness_html` 的 long 放宽判定 `timeout == CASE_TIMEOUT`（10s）
  对 corpus 通道（30s）失效 → 60s 页级超时永远没有窗口 fire → 30s 案超时
  收场。T 分类与上游一致（body 挂死于未实现事件面，上游 60s 同判 TIMEOUT），
  仅墙钟差；放宽语义是否对 corpus 通道恢复（max(budget, 60s)）待后续切片
  单独定夺。
- 承袭不变：details 族 lib flake、无锚链碰撞理论面盘点、with(window) 包合
  分支影子旁路面、影子名单非注册类 API 不全、engine lib 环境面、R102
  live_aware pending 失明、promise_test 串行启动面（s148 观察①——本轮后
  仅影响「单案挂死冻结尾部」的文件内部，文件级完成已不受牵连）、待决三项
  （css/text 语料资源、green-2x2.png、parser form pointer 缓行 2026-10-23）。

## 下一步（s150 候选）

①早收形 91 文件的 partial rows 单根翻拣（2F→P 类真兼容面——如
link-multiple 4F 的 stylesheet.py 端点仿真面勘根，s138 echo 端点先例）；
②63F 目录基线面再翻拣（s148 候选④ 承袭）；③无锚链碰撞理论面影响面盘点
（s148 候选②承袭）；④encoding 流 M1 进度对账（s148 候选③承袭）。
