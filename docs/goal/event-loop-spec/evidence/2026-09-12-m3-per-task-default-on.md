# M3 收口 — per-task kill-switch default-on（②a timer 泵，2026-09-12）

**决策项**：goal 待用户决策清单「M3 per-task（timer 泵/renderer tick）与 task queue 的
default-on 时机」按分期建议执行。本文件记 ②a（runner timer 泵）；②b（MO host trigger）
另记；renderer tick 压后（product-smoke 基线恢复 + 三面 A/B，见 master.md 决策记账）。

## ②a：runner timer 泵 default-on（opt-out 化）

- `tests/wpt-runner/src/testharness.rs`：`ZW_TESTHARNESS_TIMER_PER_TASK` 语义从
  「`=1` 开启（默认 OFF）」翻转为「`=0` 关闭（默认 ON）」——probe 循环每迭代一个
  execute → 一 timer 一 task 一 checkpoint（spec event loop processing model step 3-6）。
- 翻默认依据：M3-S1 A/B 零回归证据（evidence/2026-09-11-m3-s1-timer-per-task.md，
  dom 全量 / dom/nodes MO 12 文件 / IO / RO 四 corpus 双臂逐 subtest 零 delta）+ 生产
  路径零触碰（runner 侧开关，reftest 同步 stub 约束不受影响）。
- 注：`ZW_TESTHARNESS_RAF_FRAME_DRIVEN` 保持 `=1` opt-in 不变（goal 约束 1——reftest
  同步 stub 约束，不随本决策翻）。

## A/B（④ viewport 桥接后的新基线上复验，ON=新默认 vs OFF=`ZW_TESTHARNESS_TIMER_PER_TASK=0`）

| corpus | OFF（④ after 臂） | ON（新默认） | 翻转明细 |
|---|---|---|---|
| intersection-observer | 100P/116F/2T | 100P/116F/2T | **零 delta** |
| resize-observer | 18P/33F/6T | 18P/33F/6T | **零 delta** |
| dom 全量（54,413 subtests） | 54303P/90F/6PCF/14T | 54302P/90F/6PCF/15T | 仅 `dom/events/handler-count.html?element` case 级 Pass→Timeout（见下：arm 无关 flake，非 per-task 因果） |
| html（forms/focus） | 14P | 14P | **零 delta** |

**handler-count 定性**：定向双臂各复跑 ×2（`testharness-dom handler-count`）——
OFF 臂全 Timeout、ON 臂全 Timeout。该 case 贴 test-guard 超时边界固有抖动：④ 差分
中它的 Timeout→Pass 翻转已是侥幸臂，本轮回摆非 per-task 派发开销所致。零可复现 delta。

## 质量门禁

- `cargo test -p zero-wpt-runner` 213P/0F；clippy `-D warnings` 零警告；fmt 无 diff
- 全量 `make test` 与 ②b ON 臂轮一并汇报（runner crate 层面门禁先行全绿）

## 结论

- per-task timer 边界自本提交起为 testharness runner 默认行为；如需旧行为
  （整批排空）设 `ZW_TESTHARNESS_TIMER_PER_TASK=0`。
- M3 剩余：显式 task queue（多队列 oldest-first）维持缓行记档（gap-list §1.3 已被
  R2952 部分推翻、WPT 可观测面稀薄、无 driving 用例——重入条件见 master.md）。
