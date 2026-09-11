# M3-S1 — runner timer 泵 per-task 边界（kill-switch + A/B，2026-09-11）

## 改动面

`tests/wpt-runner/src/testharness.rs`（runner 侧，生产路径零触碰）：

- 注入 stub `__zw_fire_due_timers` 增 per-task 模式：runner env
  `ZW_TESTHARNESS_TIMER_PER_TASK=1` 置位页面全局 `__ZW_TIMER_PER_TASK` → 每次
  `__zw_fire_due_timers()` 调用只派发**首个**到期 timer，未派发的到期 timer 保留队列
  （下一 probe 迭代再派，FIFO 保序）。
- 效果：probe 循环每迭代一个 execute → **一 timer 一 task 一 microtask checkpoint**
  （spec event loop processing model step 3–6），消除「N timer 一个 execute、checkpoint
  只在批尾」的批量派发违反（evidence/2026-09-11-m1-event-loop-gap-list.md §1.2 第一行）。
- kill-switch 默认 OFF：维持整批排空，全部既有套件零行为变化。生产路径（renderer/
  browser js_worker 的 host TimerBridge 真 timer）不经此 stub，零触碰——
  reftest 单渲染路径（同步 stub 约束）不受影响（reftest 不走 testharness probe 循环）。

## A/B 证据（kill-switch OFF vs ON，逐 subtest diff）

| corpus | OFF | ON | delta |
|---|---|---|---|
| dom/nodes 全量（最大 timer-heavy 面） | 54,324P / 163F / 13T | 54,324P / 163F / 13T | **0** |
| dom/nodes MutationObserver 12 文件 | 135P / 3F / 0T | 135P / 3F / 0T | **0** |
| intersection-observer | 94P / 122F / 1T | 94P / 122F / 1T | **0** |
| resize-observer | 19P / 32F / 6T | 19P / 32F / 6T | **0** |

（163F/13T 为各 corpus 既有已知失败，双臂完全一致。）

## 质量门禁

- `cargo test -p zero-wpt-runner`：213P / 0F
- `cargo clippy -p zero-wpt-runner --all-targets -- -D warnings`：零警告
- `cargo fmt --all -- --check`：无 diff

## 结论与后续

- per-task timer 边界在 kill-switch 下零回归成立；default-on 待 M3 整体（task queue
  结构 + renderer tick_observers 同款重构）完成后一并评估（须用户点名）。
- M3 后续切片：② renderer `tick_observers` per-task 重构（生产面，需渲染 A/B）；③
  显式 task queue（多队列 oldest-first，timer/network/UI 分源）。
